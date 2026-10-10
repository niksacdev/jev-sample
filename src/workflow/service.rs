use std::{
    collections::BTreeMap,
    sync::{Arc, OnceLock},
    time::{Duration, Instant},
};

use serde_json::json;
use tokio::sync::{Mutex, Semaphore};
use tracing::{Instrument, instrument::WithSubscriber};

use super::{
    connections::{self, RouteError, RouterConnection, RouterEndpoints},
    contracts::*,
    decision::{DecisionProvider, DecisionValue, ProviderFailure, Question},
    gateway::RouteModels,
    planner::{Planner, validate_plan},
    policy::{Gate, WorkflowPolicy},
    store::{Admission, Snapshot, StoreError, WorkflowStore, epoch_ms, identity},
};
use crate::domain::Message;

#[derive(Debug)]
pub enum WorkflowError {
    Invalid,
    Unavailable,
    Capacity,
    Storage,
    Execution,
}
#[derive(Debug)]
pub enum ConnectionSetupError {
    Invalid,
    AlreadyConfigured,
    TransportUnavailable,
    UnsupportedRoute,
    InvalidEndpoint,
}
impl From<RouteError> for ConnectionSetupError {
    fn from(error: RouteError) -> Self {
        match error {
            RouteError::Unsupported => Self::UnsupportedRoute,
            RouteError::InvalidCredential => Self::Invalid,
            RouteError::InvalidEndpoint => Self::InvalidEndpoint,
            RouteError::Client => Self::TransportUnavailable,
        }
    }
}
impl From<StoreError> for WorkflowError {
    fn from(_: StoreError) -> Self {
        Self::Storage
    }
}

#[derive(Clone)]
pub struct WorkflowService {
    inner: Arc<Runtime>,
}
struct Runtime {
    planner: Option<Arc<dyn Planner>>,
    providers: BTreeMap<ProviderId, Arc<dyn DecisionProvider>>,
    setup: OnceLock<SetupConnections>,
    models: RouteModels,
    endpoints: RouterEndpoints,
    policy: WorkflowPolicy,
    store: WorkflowStore,
    capacity: Arc<Semaphore>,
    resume_lock: Mutex<()>,
}

/// Connections added once at runtime through the chosen routers; startup connections always win.
struct SetupConnections {
    planner: Option<Arc<dyn Planner>>,
    providers: BTreeMap<ProviderId, Arc<dyn DecisionProvider>>,
}

impl Runtime {
    fn planner(&self) -> Option<&Arc<dyn Planner>> {
        self.planner
            .as_ref()
            .or_else(|| self.setup.get()?.planner.as_ref())
    }
    fn provider(&self, id: &ProviderId) -> Option<&Arc<dyn DecisionProvider>> {
        self.providers
            .get(id)
            .or_else(|| self.setup.get()?.providers.get(id))
    }
}

impl WorkflowService {
    pub fn new(
        planner: Option<Arc<dyn Planner>>,
        providers: BTreeMap<ProviderId, Arc<dyn DecisionProvider>>,
        policy: WorkflowPolicy,
        store: WorkflowStore,
    ) -> Self {
        let capacity = Arc::new(Semaphore::new(policy.max_concurrent as usize));
        Self {
            inner: Arc::new(Runtime {
                planner,
                providers,
                setup: OnceLock::new(),
                models: RouteModels::checked_in(),
                endpoints: RouterEndpoints::production(),
                policy,
                store,
                capacity,
                resume_lock: Mutex::new(()),
            }),
        }
    }
    /// Overrides checked-in router models; only effective before the service is cloned.
    pub fn with_route_models(mut self, models: RouteModels) -> Self {
        if let Some(runtime) = Arc::get_mut(&mut self.inner) {
            runtime.models = models;
        }
        self
    }
    /// Points routers at a loopback mock; only effective before the service is cloned.
    pub fn with_router_endpoints(mut self, endpoints: RouterEndpoints) -> Self {
        if let Some(runtime) = Arc::get_mut(&mut self.inner) {
            runtime.endpoints = endpoints;
        }
        self
    }
    pub fn options(&self) -> WorkflowOptions {
        WorkflowOptions {
            planner: PlannerOption {
                available: self.inner.planner().is_some(),
                model: self.inner.planner().map(|p| p.model()),
            },
            providers: [ProviderId::Code, ProviderId::Jev, ProviderId::Openai]
                .into_iter()
                .map(|id| {
                    let provider = self.inner.provider(&id);
                    DecisionProviderOption {
                        id,
                        available: provider.is_some(),
                        model: provider.and_then(|p| p.model()),
                        capability: provider.map(|p| p.capability()).unwrap_or_else(|| {
                            "Not configured; connect it in setup or with a direct vendor key at startup.".into()
                        }),
                    }
                })
                .collect(),
            limits: WorkflowLimits {
                max_tasks: self.inner.policy.max_tasks,
                max_steps: self.inner.policy.max_steps,
                deadline_ms: self.inner.policy.deadline_ms,
            },
            synthetic_only: true,
            connections: connections::catalog(&self.inner.models),
        }
    }
    /// Installs the chosen routes atomically. Only missing slots may be filled, and credentials
    /// must cover exactly the routers the chosen routes use.
    pub fn configure_connections(
        &self,
        input: ConnectionSetup,
    ) -> Result<WorkflowOptions, ConnectionSetupError> {
        if self.inner.setup.get().is_some() {
            return Err(ConnectionSetupError::AlreadyConfigured);
        }
        if input.planner.is_none() && input.decisions.is_empty() {
            return Err(ConnectionSetupError::Invalid);
        }
        if input.planner.is_some() && self.inner.planner().is_some() {
            return Err(ConnectionSetupError::AlreadyConfigured);
        }
        // Setup installs once, so it must cover every required slot startup left empty.
        let covers_jev = input
            .decisions
            .iter()
            .any(|route| connections::decision_provider(route.choice) == Some(ProviderId::Jev));
        if (input.planner.is_none() && self.inner.planner().is_none())
            || (!covers_jev && self.inner.provider(&ProviderId::Jev).is_none())
        {
            return Err(ConnectionSetupError::Invalid);
        }
        let mut used = std::collections::BTreeSet::new();
        let mut slots = std::collections::BTreeSet::new();
        if let Some(route) = input.planner {
            if !connections::supports_planner(route.router, route.choice) {
                return Err(ConnectionSetupError::UnsupportedRoute);
            }
            used.insert(route.router);
        }
        for route in &input.decisions {
            if !connections::supports_decision(route.router, route.choice) {
                return Err(ConnectionSetupError::UnsupportedRoute);
            }
            let slot = connections::decision_provider(route.choice)
                .ok_or(ConnectionSetupError::UnsupportedRoute)?;
            if !slots.insert(slot) {
                return Err(ConnectionSetupError::Invalid);
            }
            if self.inner.provider(&slot).is_some() {
                return Err(ConnectionSetupError::AlreadyConfigured);
            }
            used.insert(route.router);
        }
        let supplied: std::collections::BTreeSet<_> =
            input.credentials.iter().map(|c| c.router).collect();
        if supplied.len() != input.credentials.len() || supplied != used {
            return Err(ConnectionSetupError::Invalid);
        }
        let endpoints = &self.inner.endpoints;
        let routers = input
            .credentials
            .iter()
            .map(|c| Ok((c.router, RouterConnection::new(c, endpoints)?)))
            .collect::<Result<BTreeMap<_, _>, RouteError>>()?;
        let timeout = Duration::from_millis(u64::from(self.inner.policy.deadline_ms));
        let models = &self.inner.models;
        let connection = |router| routers.get(&router).ok_or(ConnectionSetupError::Invalid);
        let planner = input
            .planner
            .map(|route| {
                connection(route.router)?
                    .planner(route.choice, models, endpoints, timeout)
                    .map_err(ConnectionSetupError::from)
            })
            .transpose()?;
        let mut providers: BTreeMap<ProviderId, Arc<dyn DecisionProvider>> = BTreeMap::new();
        for route in &input.decisions {
            if let Some(slot) = connections::decision_provider(route.choice) {
                providers.insert(
                    slot,
                    connection(route.router)?.decision(route.choice, models, endpoints, timeout)?,
                );
            }
        }
        // An OpenRouter key also fills the general-LLM comparison baseline when it is missing.
        if !providers.contains_key(&ProviderId::Openai)
            && self.inner.provider(&ProviderId::Openai).is_none()
            && let Some(openrouter) = routers.get(&RouterId::Openrouter)
            && let Some(baseline) = openrouter.llm_baseline(models, endpoints, timeout)
        {
            providers.insert(ProviderId::Openai, baseline?);
        }
        self.inner
            .setup
            .set(SetupConnections { planner, providers })
            .map_err(|_| ConnectionSetupError::AlreadyConfigured)?;
        tracing::info!(
            event = "connections_configured",
            routers = ?used,
            planner = input.planner.is_some(),
            decisions = input.decisions.len(),
        );
        Ok(self.options())
    }
    pub async fn details(&self) -> Result<Vec<OperatorWorkflow>, WorkflowError> {
        Ok(self
            .inner
            .store
            .list()
            .await?
            .into_iter()
            .map(|s| s.detail)
            .collect())
    }

    #[tracing::instrument(skip_all)]
    pub async fn submit(
        &self,
        request: WorkflowSubmission,
    ) -> Result<WorkflowComparison, WorkflowError> {
        let service = self.clone();
        tokio::spawn(
            async move { service.submit_admitted(request).await }
                .in_current_span()
                .with_current_subscriber(),
        )
        .await
        .map_err(|_| WorkflowError::Execution)?
    }

    async fn submit_admitted(
        &self,
        request: WorkflowSubmission,
    ) -> Result<WorkflowComparison, WorkflowError> {
        Message::new(request.message.clone()).map_err(|_| WorkflowError::Invalid)?;
        if !valid_key(&request.client_request_id)
            || request.providers.is_empty()
            || request.providers.len() > 3
            || request
                .providers
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != request.providers.len()
        {
            return Err(WorkflowError::Invalid);
        }
        let planner = self
            .inner
            .planner()
            .cloned()
            .ok_or(WorkflowError::Unavailable)?;
        if request
            .providers
            .iter()
            .any(|p| self.inner.provider(p).is_none())
        {
            return Err(WorkflowError::Unavailable);
        }
        let permit = self
            .inner
            .capacity
            .clone()
            .try_acquire_owned()
            .map_err(|_| WorkflowError::Capacity)?;
        let encoded = serde_json::to_string(&request).map_err(|_| WorkflowError::Invalid)?;
        let payload = identity(&encoded);
        let snapshot = Snapshot {
            detail: OperatorWorkflow {
                comparison: WorkflowComparison {
                    comparison_id: String::new(),
                    input_key: identity(&request.message),
                    base_plan: WorkflowPlan {
                        plan_id: String::new(),
                        tasks: Vec::new(),
                    },
                    runs: request
                        .providers
                        .iter()
                        .map(|id| WorkflowRun {
                            run_id: String::new(),
                            plan_id: String::new(),
                            provider: *id,
                            model: self.inner.provider(id).and_then(|p| p.model()),
                            state: WorkflowState::Running,
                            reply: String::new(),
                            tasks: Vec::new(),
                            event_trace: Vec::new(),
                            usage: WorkflowUsage::default(),
                            elapsed_ms: 0,
                            failure_code: None,
                        })
                        .collect(),
                    mode: "shared_base_plan_independent_continuations".into(),
                    complete: false,
                    planner_model: Some(planner.model()),
                    planner_usage: WorkflowUsage::default(),
                    failure_code: None,
                },
                message: request.message.clone(),
                decisions: Vec::new(),
                protected_context_json: String::new(),
            },
            policy: self.inner.policy.clone(),
            context: json!({"message":request.message,"facts":[],"judgments":[]}).to_string(),
            run_contexts: BTreeMap::new(),
        };
        let comparison_id = match self
            .inner
            .store
            .admit(
                format!("submit:{}", request.client_request_id),
                payload,
                snapshot,
            )
            .await?
        {
            Admission::Existing(old) => return Ok(old.detail.comparison),
            Admission::New(id) => id,
        };
        let service = self.clone();
        let id = comparison_id.clone();
        let worker = tokio::spawn(
            async move {
                let _permit = permit;
                let mut snapshot = service.inner.store.get(id.clone()).await?;
                for index in 0..snapshot.detail.comparison.runs.len() {
                    event(&mut snapshot, index, None, "workflow_admitted", "running")?;
                    event(&mut snapshot, index, None, "plan_started", "running")?;
                }
                service.inner.store.save(snapshot.clone()).await?;
                let context = snapshot.context.clone();
                let (mut plan, usage) = planner
                    .plan(&context)
                    .await
                    .map_err(|_| WorkflowError::Execution)?;
                if !validate_plan(&plan, snapshot.policy.max_tasks) {
                    return Err(WorkflowError::Execution);
                }
                plan.plan_id = identity(
                    &json!({"plan":plan,"context":context,"planner_model":planner.model()})
                        .to_string(),
                );
                snapshot.detail.comparison.base_plan = plan.clone();
                snapshot.detail.comparison.planner_usage = usage;
                for index in 0..snapshot.detail.comparison.runs.len() {
                    snapshot.detail.comparison.runs[index].tasks = plan.tasks.clone();
                    snapshot.detail.comparison.runs[index].plan_id = plan.plan_id.clone();
                    event(&mut snapshot, index, None, "plan_completed", "succeeded")?;
                }
                service.inner.store.save(snapshot.clone()).await?;
                for index in 0..snapshot.detail.comparison.runs.len() {
                    service
                        .run(&mut snapshot, index, context.clone(), false)
                        .await?;
                }
                snapshot.detail.comparison.complete = snapshot
                    .detail
                    .comparison
                    .runs
                    .iter()
                    .all(|r| r.state == WorkflowState::Completed);
                service.inner.store.save(snapshot.clone()).await?;
                Ok::<_, WorkflowError>(service.inner.store.get(id).await?.detail.comparison)
            }
            .in_current_span()
            .with_current_subscriber(),
        );
        let supervisor = self.clone();
        tokio::spawn(
            async move { supervisor.supervise(worker, comparison_id).await }
                .in_current_span()
                .with_current_subscriber(),
        )
        .await
        .map_err(|_| WorkflowError::Execution)?
    }

    async fn supervise(
        &self,
        mut worker: tokio::task::JoinHandle<Result<WorkflowComparison, WorkflowError>>,
        id: String,
    ) -> Result<WorkflowComparison, WorkflowError> {
        match tokio::time::timeout(
            Duration::from_millis(u64::from(self.inner.policy.deadline_ms)),
            &mut worker,
        )
        .await
        {
            Ok(Ok(Ok(result))) => Ok(result),
            outcome => {
                let code = match outcome {
                    Err(_) => "workflow_deadline",
                    Ok(Err(_)) => "workflow_worker_failed",
                    Ok(Ok(Err(WorkflowError::Storage))) => "workflow_storage_failed",
                    _ => "workflow_planning_failed",
                };
                worker.abort();
                // Completed handles must not be polled a second time.
                if code == "workflow_deadline" {
                    let _ = worker.await;
                }
                let mut snapshot = self.inner.store.get(id).await?;
                snapshot.detail.comparison.failure_code = Some(code.into());
                for index in 0..snapshot.detail.comparison.runs.len() {
                    if snapshot.detail.comparison.runs[index].state == WorkflowState::Running {
                        snapshot.detail.comparison.runs[index].state = WorkflowState::Failed;
                        snapshot.detail.comparison.runs[index].failure_code = Some(code.into());
                        snapshot.detail.comparison.runs[index].reply =
                            "Workflow failed; no external business action was executed.".into();
                        event(&mut snapshot, index, None, "workflow_failed", code)?;
                    }
                }
                self.inner.store.save(snapshot.clone()).await?;
                if code == "workflow_storage_failed" {
                    Err(WorkflowError::Storage)
                } else {
                    Ok(self
                        .inner
                        .store
                        .get(snapshot.detail.comparison.comparison_id)
                        .await?
                        .detail
                        .comparison)
                }
            }
        }
    }

    async fn run(
        &self,
        snapshot: &mut Snapshot,
        index: usize,
        context: String,
        review_override: bool,
    ) -> Result<(), WorkflowError> {
        let started = Instant::now();
        let provider_id = snapshot.detail.comparison.runs[index].provider;
        let provider = self
            .inner
            .provider(&provider_id)
            .ok_or(WorkflowError::Unavailable)?;
        let mut context: serde_json::Value =
            serde_json::from_str(&context).map_err(|_| WorkflowError::Execution)?;
        let mut steps = snapshot.detail.comparison.runs[index]
            .event_trace
            .iter()
            .filter(|e| matches!(e.stage.as_str(), "tool_started" | "decision_started"))
            .count() as u32;
        for task_index in 0..snapshot.detail.comparison.runs[index].tasks.len() {
            let task = snapshot.detail.comparison.runs[index].tasks[task_index].clone();
            if task.state == "completed" || task.state == "reviewed" {
                continue;
            }
            steps += 1;
            if steps > snapshot.policy.max_steps {
                return Err(WorkflowError::Execution);
            }
            if !task.depends_on.iter().all(|d| {
                snapshot.detail.comparison.runs[index]
                    .tasks
                    .iter()
                    .any(|t| t.id == *d && matches!(t.state.as_str(), "completed" | "reviewed"))
            }) {
                return Err(WorkflowError::Execution);
            }
            snapshot.detail.comparison.runs[index].tasks[task_index].state = "running".into();
            event(
                snapshot,
                index,
                Some(&task.id),
                if task.kind == WorkflowTaskKind::Tool {
                    "tool_started"
                } else {
                    "decision_started"
                },
                "running",
            )?;
            self.inner.store.save(snapshot.clone()).await?;
            if task.kind == WorkflowTaskKind::Tool {
                let fact = if task.name == "synthetic_record" {
                    json!({"tool":"synthetic_record","synthetic":true,"status":"demonstration_only","message":"Invented read-only record; no insurer system was contacted."})
                } else {
                    json!({"tool":"workflow_capabilities","synthetic":true,"external_actions":false,"requires_authorized_employee_resume":true})
                };
                context["facts"]
                    .as_array_mut()
                    .ok_or(WorkflowError::Execution)?
                    .push(fact);
                snapshot.detail.comparison.runs[index].tasks[task_index].state = "completed".into();
                event(
                    snapshot,
                    index,
                    Some(&task.id),
                    "tool_completed",
                    "succeeded",
                )?;
            } else {
                let question = Question::named(&task.name).ok_or(WorkflowError::Execution)?;
                let encoded = context.to_string();
                let input_key =
                    identity(&json!({"context":context,"question":question}).to_string());
                let attempt = provider
                    .evaluate(&encoded, std::slice::from_ref(&question))
                    .await;
                tracing::info!(
                    event="decision_attempt_finished",
                    comparison_id=%snapshot.detail.comparison.comparison_id,
                    run_id=%snapshot.detail.comparison.runs[index].run_id,
                    task_id=%task.id,question_id=%question.id,question_version=%question.version,
                    provider=?provider_id,model=provider.model().as_deref(),policy_version=%snapshot.policy.version,
                    attempt=1,elapsed_ms=attempt.elapsed_ms,
                    input_tokens=attempt.usage.input_tokens,output_tokens=attempt.usage.output_tokens,
                    cost_usd=attempt.usage.cost_usd,
                    failure=attempt.answers.as_ref().err().map(ProviderFailure::code),
                );
                add_usage(
                    &mut snapshot.detail.comparison.runs[index].usage,
                    &attempt.usage,
                )?;
                let result = attempt.answers.and_then(|mut answers| {
                    if answers.len() != 1 {
                        return Err(ProviderFailure::InvalidResponse);
                    }
                    let answer = answers
                        .remove(&question.id)
                        .ok_or(ProviderFailure::InvalidResponse)?;
                    if !answer.validate(&question) {
                        return Err(ProviderFailure::InvalidResponse);
                    }
                    Ok(answer)
                });
                let serialized = match &result {
                    Ok(value) => {
                        serde_json::to_string(value).map_err(|_| WorkflowError::Execution)?
                    }
                    Err(failure) => format!("failure:{}", failure.code()),
                };
                let semantics = match &result {
                    Ok(DecisionValue::Predicate { .. }) => {
                        Some("probability_of_yes_not_confidence".into())
                    }
                    Ok(
                        DecisionValue::Choice { confidence, .. }
                        | DecisionValue::Score { confidence, .. },
                    ) => confidence.as_ref().map(|c| c.semantics.clone()),
                    Ok(DecisionValue::Deterministic { .. }) => {
                        Some("deterministic_boolean_no_probability".into())
                    }
                    _ => None,
                };
                snapshot.detail.decisions.push(WorkflowDecisionRecord {
                    run_id: snapshot.detail.comparison.runs[index].run_id.clone(),
                    plan_id: snapshot.detail.comparison.runs[index].plan_id.clone(),
                    decision_input_key: input_key,
                    question_id: question.id.clone(),
                    question_version: question.version.clone(),
                    provider: provider_id,
                    model: provider.model(),
                    policy_version: snapshot.policy.version.clone(),
                    result: serialized,
                    confidence_semantics: semantics,
                    usage: attempt.usage,
                    elapsed_ms: attempt.elapsed_ms,
                    complete: result.as_ref().is_ok_and(|v| {
                        !matches!(v, DecisionValue::Refusal | DecisionValue::Unsupported)
                    }),
                    task_id: task.id.clone(),
                    attempt: 1,
                    context_json: encoded,
                    question_json: serde_json::to_string(&question)
                        .map_err(|_| WorkflowError::Execution)?,
                });
                match result {
                    Err(failure) => {
                        snapshot.detail.comparison.runs[index].state = WorkflowState::Failed;
                        snapshot.detail.comparison.runs[index].failure_code =
                            Some(format!("decision_{}", failure.code()));
                        snapshot.detail.comparison.runs[index].tasks[task_index].state =
                            "failed".into();
                        event(
                            snapshot,
                            index,
                            Some(&task.id),
                            "decision_failed",
                            failure.code(),
                        )?;
                        break;
                    }
                    Ok(answer) => {
                        event(
                            snapshot,
                            index,
                            Some(&task.id),
                            "decision_completed",
                            if matches!(answer, DecisionValue::Refusal) {
                                "refused"
                            } else if matches!(answer, DecisionValue::Unsupported) {
                                "unsupported"
                            } else {
                                "answered"
                            },
                        )?;
                        let gate = snapshot.policy.gate(provider_id, &question, &answer);
                        context["judgments"]
                            .as_array_mut()
                            .ok_or(WorkflowError::Execution)?
                            .push(json!({"question":question.id,"answer":answer}));
                        if gate != Gate::Proceed && !review_override {
                            snapshot.detail.comparison.runs[index].state = if gate == Gate::Clarify
                            {
                                WorkflowState::Clarification
                            } else {
                                WorkflowState::EmployeeReview
                            };
                            snapshot.detail.comparison.runs[index].tasks[task_index].state =
                                "paused".into();
                            event(
                                snapshot,
                                index,
                                Some(&task.id),
                                "workflow_paused",
                                if gate == Gate::Clarify {
                                    "clarification"
                                } else {
                                    "employee_review"
                                },
                            )?;
                            break;
                        }
                        snapshot.detail.comparison.runs[index].tasks[task_index].state =
                            "completed".into();
                    }
                }
            }
            snapshot.detail.protected_context_json = context.to_string();
            snapshot.run_contexts.insert(
                snapshot.detail.comparison.runs[index].run_id.clone(),
                context.to_string(),
            );
            self.inner.store.save(snapshot.clone()).await?;
        }
        if snapshot.detail.comparison.runs[index].state == WorkflowState::Running {
            snapshot.detail.comparison.runs[index].state = WorkflowState::Completed;
        }
        let terminal_state = snapshot.detail.comparison.runs[index].state;
        context["workflow_state"] = json!(terminal_state);
        snapshot.detail.comparison.runs[index].state = WorkflowState::Running;
        snapshot.detail.protected_context_json = context.to_string();
        snapshot.run_contexts.insert(
            snapshot.detail.comparison.runs[index].run_id.clone(),
            context.to_string(),
        );
        event(snapshot, index, None, "reply_started", "running")?;
        self.inner.store.save(snapshot.clone()).await?;
        let planner = self.inner.planner().ok_or(WorkflowError::Unavailable)?;
        match planner.reply(&context.to_string()).await {
            Ok((reply, usage)) => {
                snapshot.detail.comparison.runs[index].state = terminal_state;
                snapshot.detail.comparison.runs[index].reply = reply;
                add_usage(&mut snapshot.detail.comparison.planner_usage, &usage)?;
                event(snapshot, index, None, "reply_completed", "succeeded")?;
            }
            Err(failure) => {
                snapshot.detail.comparison.runs[index].state = WorkflowState::Failed;
                snapshot.detail.comparison.runs[index].failure_code =
                    Some(format!("reply_{}", failure.code()));
                snapshot.detail.comparison.runs[index].reply="Conversational response failed. Inspect the recorded workflow; no response was substituted.".into();
                event(snapshot, index, None, "reply_failed", failure.code())?;
            }
        }
        snapshot.detail.comparison.runs[index].elapsed_ms =
            u32::try_from(started.elapsed().as_millis()).unwrap_or(u32::MAX);
        let state = serde_json::to_string(&snapshot.detail.comparison.runs[index].state)
            .map_err(|_| WorkflowError::Execution)?;
        event(
            snapshot,
            index,
            None,
            "workflow_observed",
            state.trim_matches('"'),
        )?;
        self.inner.store.save(snapshot.clone()).await?;
        Ok(())
    }

    pub async fn resume(
        &self,
        id: String,
        input: WorkflowResume,
    ) -> Result<WorkflowComparison, WorkflowError> {
        let service = self.clone();
        tokio::spawn(
            async move { service.resume_admitted(id, input).await }
                .in_current_span()
                .with_current_subscriber(),
        )
        .await
        .map_err(|_| WorkflowError::Execution)?
    }

    async fn resume_admitted(
        &self,
        id: String,
        input: WorkflowResume,
    ) -> Result<WorkflowComparison, WorkflowError> {
        Message::new(input.message.clone()).map_err(|_| WorkflowError::Invalid)?;
        if !valid_key(&input.client_request_id) {
            return Err(WorkflowError::Invalid);
        }
        let lock = self.inner.resume_lock.lock().await;
        let mut snapshot = self.inner.store.get(id.clone()).await?;
        let index = snapshot
            .detail
            .comparison
            .runs
            .iter()
            .position(|r| r.run_id == input.run_id)
            .ok_or(WorkflowError::Invalid)?;
        let configured = self
            .inner
            .provider(&snapshot.detail.comparison.runs[index].provider)
            .ok_or(WorkflowError::Unavailable)?;
        if configured.model() != snapshot.detail.comparison.runs[index].model
            || self.inner.planner().map(|p| p.model()) != snapshot.detail.comparison.planner_model
            || serde_json::to_value(&snapshot.policy).map_err(|_| WorkflowError::Execution)?
                != serde_json::to_value(&self.inner.policy).map_err(|_| WorkflowError::Execution)?
        {
            return Err(WorkflowError::Unavailable);
        }
        let payload = identity(&serde_json::to_string(&input).map_err(|_| WorkflowError::Invalid)?);
        if self
            .inner
            .store
            .prior_request(
                format!("resume:{}", input.client_request_id),
                payload.clone(),
                id.clone(),
            )
            .await?
        {
            return Ok(snapshot.detail.comparison);
        }
        let state = snapshot.detail.comparison.runs[index].state;
        let paused_task = snapshot.detail.comparison.runs[index]
            .tasks
            .iter()
            .find(|t| t.state == "paused");
        if snapshot.detail.comparison.runs[index].plan_id != input.expected_plan_id
            || paused_task.map(|t| t.id.as_str()) != Some(input.expected_task_id.as_str())
        {
            return Err(WorkflowError::Invalid);
        }
        if !(state == WorkflowState::Clarification && !input.employee_review
            || state == WorkflowState::EmployeeReview && input.employee_review)
            || snapshot
                .detail
                .comparison
                .runs
                .iter()
                .any(|r| r.state == WorkflowState::Running)
            || snapshot.detail.comparison.runs[index]
                .event_trace
                .iter()
                .filter(|e| e.stage == "workflow_resumed")
                .count()
                >= 2
        {
            return Err(WorkflowError::Invalid);
        }
        let permit = self
            .inner
            .capacity
            .clone()
            .try_acquire_owned()
            .map_err(|_| WorkflowError::Capacity)?;
        if snapshot.detail.comparison.runs[index]
            .tasks
            .iter()
            .any(|t| t.state == "running")
        {
            return Err(WorkflowError::Invalid);
        }
        snapshot.detail.comparison.runs[index].state = WorkflowState::Running;
        event(
            &mut snapshot,
            index,
            None,
            "workflow_resumed",
            if input.employee_review {
                "authorized_employee"
            } else {
                "customer_clarification"
            },
        )?;
        if !self
            .inner
            .store
            .reserve_resume(
                snapshot.clone(),
                format!("resume:{}", input.client_request_id),
                payload,
            )
            .await?
        {
            return Ok(self.inner.store.get(id).await?.detail.comparison);
        }
        drop(lock);
        let service = self.clone();
        let worker = tokio::spawn(
            async move {
                let _permit = permit;
                let mut context: serde_json::Value = serde_json::from_str(
                    snapshot
                        .run_contexts
                        .get(&input.run_id)
                        .unwrap_or(&snapshot.context),
                )
                .map_err(|_| WorkflowError::Execution)?;
                context["message"] = json!(format!(
                    "{}\nClarification: {}",
                    context["message"]
                        .as_str()
                        .ok_or(WorkflowError::Execution)?,
                    input.message
                ));
                if input.employee_review {
                    // Only the currently paused task is reviewed; later judgments retain their own gates.
                    for task in &mut snapshot.detail.comparison.runs[index].tasks {
                        if task.state == "paused" {
                            task.state = "reviewed".into();
                        }
                    }
                    context["facts"]
                        .as_array_mut()
                        .ok_or(WorkflowError::Execution)?
                        .push(json!({"employee_review":input.message,"synthetic_only":true}));
                } else {
                    let planner = service
                        .inner
                        .planner()
                        .ok_or(WorkflowError::Unavailable)?;
                    event(&mut snapshot, index, None, "replan_started", "running")?;
                    service.inner.store.save(snapshot.clone()).await?;
                    let (mut plan, usage) = planner
                        .plan(&context.to_string())
                        .await
                        .map_err(|_| WorkflowError::Execution)?;
                    if !validate_plan(&plan, snapshot.policy.max_tasks) {
                        return Err(WorkflowError::Execution);
                    }
                    plan.plan_id = identity(&json!({
                        "plan":plan,"context":context,"previous_plan_id":snapshot.detail.comparison.runs[index].plan_id,
                        "resume_request_id":input.client_request_id,"planner_model":planner.model(),
                    }).to_string());
                    snapshot.detail.comparison.runs[index].plan_id=plan.plan_id.clone();
                    snapshot.detail.comparison.runs[index].tasks = plan.tasks;
                    add_usage(&mut snapshot.detail.comparison.planner_usage, &usage)?;
                    event(&mut snapshot, index, None, "replan_completed", "succeeded")?;
                }
                snapshot.detail.comparison.mode = "shared_base_plan_with_divergent_resumes".into();
                service
                    .run(&mut snapshot, index, context.to_string(), false)
                    .await?;
                snapshot.detail.comparison.complete = snapshot
                    .detail
                    .comparison
                    .runs
                    .iter()
                    .all(|r| r.state == WorkflowState::Completed);
                service.inner.store.save(snapshot.clone()).await?;
                Ok(service.inner.store.get(snapshot.detail.comparison.comparison_id).await?.detail.comparison)
            }
            .in_current_span()
            .with_current_subscriber(),
        );
        let supervisor = self.clone();
        tokio::spawn(
            async move { supervisor.supervise(worker, id).await }
                .in_current_span()
                .with_current_subscriber(),
        )
        .await
        .map_err(|_| WorkflowError::Execution)?
    }
}

fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 80
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
fn add_usage(total: &mut WorkflowUsage, usage: &WorkflowUsage) -> Result<(), WorkflowError> {
    total.attempts = total
        .attempts
        .checked_add(usage.attempts)
        .ok_or(WorkflowError::Execution)?;
    let add = |old: Option<u32>, next: Option<u32>| -> Result<Option<u32>, WorkflowError> {
        match (old, next) {
            (Some(a), Some(b)) => Ok(Some(a.checked_add(b).ok_or(WorkflowError::Execution)?)),
            (None, Some(b)) if total.attempts == usage.attempts => Ok(Some(b)),
            _ => Ok(None),
        }
    };
    total.input_tokens = add(total.input_tokens, usage.input_tokens)?;
    total.output_tokens = add(total.output_tokens, usage.output_tokens)?;
    total.cost_usd = match (total.cost_usd, usage.cost_usd) {
        (Some(a), Some(b)) => Some(a + b),
        (None, Some(b)) if total.attempts == usage.attempts => Some(b),
        _ => None,
    };
    let first = total.attempts == usage.attempts;
    total.cost_source = match (total.cost_usd, total.cost_source, usage.cost_source) {
        (None, ..) => None,
        (Some(_), _, Some(next)) if first => Some(next),
        (Some(_), Some(CostSource::Reported), Some(CostSource::Reported)) => {
            Some(CostSource::Reported)
        }
        (Some(_), Some(_), Some(_)) => Some(CostSource::Estimated),
        _ => None,
    };
    Ok(())
}
fn event(
    snapshot: &mut Snapshot,
    index: usize,
    task: Option<&str>,
    stage: &str,
    outcome: &str,
) -> Result<(), WorkflowError> {
    let now = epoch_ms()?;
    let planner_stage =
        stage.starts_with("plan_") || stage.starts_with("replan_") || stage.starts_with("reply_");
    let actor = if planner_stage {
        "planner"
    } else if stage.starts_with("decision_") {
        "decision_provider"
    } else if stage.starts_with("tool_") {
        "tool"
    } else {
        "runtime"
    };
    let run = &mut snapshot.detail.comparison.runs[index];
    run.event_trace.push(WorkflowEvent {
        sequence: u32::try_from(run.event_trace.len() + 1).map_err(|_| WorkflowError::Execution)?,
        comparison_id: snapshot.detail.comparison.comparison_id.clone(),
        run_id: run.run_id.clone(),
        task_id: task.map(str::to_owned),
        provider: if planner_stage {
            Some(ProviderId::Openai)
        } else if actor == "decision_provider" {
            Some(run.provider)
        } else {
            None
        },
        model: if planner_stage {
            snapshot.detail.comparison.planner_model.clone()
        } else if actor == "decision_provider" {
            run.model.clone()
        } else {
            None
        },
        actor: actor.into(),
        plan_id: (!run.plan_id.is_empty()).then(|| run.plan_id.clone()),
        policy_version: snapshot.policy.version.clone(),
        schema_version: "workflow-events-1".into(),
        stage: stage.into(),
        outcome: outcome.into(),
        occurred_at_ms: now,
        recorded_at_ms: None,
    });
    tracing::info!(event="workflow_event",comparison_id=%snapshot.detail.comparison.comparison_id,run_id=%run.run_id,task_id=task,
        run_provider=?run.provider,actor,provider=?run.event_trace.last().and_then(|e|e.provider),model=run.event_trace.last().and_then(|e|e.model.as_deref()),sequence=run.event_trace.len(),stage,outcome,policy_version=%snapshot.policy.version);
    Ok(())
}
