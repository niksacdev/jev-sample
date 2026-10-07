//! Servicing use case: bounded execution, routing and run projections.
//! Transport and concrete providers are composed outside this module.

use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};

use serde::Deserialize;
use tokio::sync::{Mutex, Semaphore};
use tracing::{Instrument, instrument::WithSubscriber};

use crate::{
    assessment::{AssessmentFailure, Assessor},
    contracts::{
        AssessorId, AssessorOption, CustomerReply, ExecutionTraceEvent, IntentSignal, OperatorRun,
        OperatorRunDetail, RunState, ServicingTask, TaskState,
    },
    domain::{InvalidMessage, Message},
    routing::RoutingPolicy,
};

pub struct ExecutionLimits {
    max_concurrent: usize,
    max_runs: usize,
    deadline_seconds: u64,
}

impl ExecutionLimits {
    pub fn from_json(json: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Config {
            max_concurrent: usize,
            max_runs: usize,
            deadline_seconds: u64,
        }
        let config: Config = serde_json::from_str(json)?;
        let limits = Self {
            max_concurrent: config.max_concurrent,
            max_runs: config.max_runs,
            deadline_seconds: config.deadline_seconds,
        };
        if limits.max_concurrent == 0
            || limits.max_concurrent > Semaphore::MAX_PERMITS
            || limits.max_runs == 0
            || limits.deadline_seconds == 0
            || limits.deadline_seconds > 3600
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Execution limits need positive capacities and a deadline of at most 3600 seconds",
            )
            .into());
        }
        Ok(limits)
    }
}

#[derive(Debug)]
pub enum ServiceError {
    InvalidMessage(InvalidMessage),
    CapacityExceeded,
    RunLimit,
    Assessment {
        failure: AssessmentFailure,
        run_id: String,
    },
    AssessorUnavailable(AssessorId),
}

#[derive(Clone)]
pub struct ServicingService {
    inner: Arc<Runtime>,
}

struct Runtime {
    assessors: BTreeMap<AssessorId, Arc<dyn Assessor>>,
    policy: RoutingPolicy,
    limits: ExecutionLimits,
    capacity: Arc<Semaphore>,
    runs: Mutex<BTreeMap<String, OperatorRun>>,
}

impl ServicingService {
    pub fn new(
        assessor: Arc<dyn Assessor>,
        policy: RoutingPolicy,
        limits: ExecutionLimits,
    ) -> Self {
        Self::with_assessors(
            BTreeMap::from([(AssessorId::Code, assessor)]),
            policy,
            limits,
        )
    }

    pub fn with_assessors(
        assessors: BTreeMap<AssessorId, Arc<dyn Assessor>>,
        policy: RoutingPolicy,
        limits: ExecutionLimits,
    ) -> Self {
        let capacity = Arc::new(Semaphore::new(limits.max_concurrent));
        Self {
            inner: Arc::new(Runtime {
                assessors,
                policy,
                limits,
                capacity,
                runs: Mutex::new(BTreeMap::new()),
            }),
        }
    }

    pub async fn submit(&self, text: String) -> Result<CustomerReply, ServiceError> {
        self.submit_with(text, AssessorId::Code).await
    }

    #[tracing::instrument(skip_all, fields(assessor = %assessor_id.as_str()))]
    pub async fn submit_with(
        &self,
        text: String,
        assessor_id: AssessorId,
    ) -> Result<CustomerReply, ServiceError> {
        let message = Message::new(text).map_err(ServiceError::InvalidMessage)?;
        let assessor = self
            .inner
            .assessors
            .get(&assessor_id)
            .cloned()
            .ok_or(ServiceError::AssessorUnavailable(assessor_id))?;
        let permit = self
            .inner
            .capacity
            .clone()
            .try_acquire_owned()
            .map_err(|_| ServiceError::CapacityExceeded)?;
        let mut store = self.inner.runs.lock().await;
        if store.len() >= self.inner.limits.max_runs {
            return Err(ServiceError::RunLimit);
        }
        let run_id = format!("run-{:04}", store.len() + 1);
        let provenance = assessor.provenance();
        let run = OperatorRun {
            run_id: run_id.clone(),
            state: RunState::Assessing,
            assessor: provenance.assessor,
            model: provenance.model,
            rubric_version: provenance.rubric_version,
            routing_version: self.inner.policy.version().into(),
            elapsed_ms: None,
            input_tokens: None,
            output_tokens: None,
            signals: Vec::new(),
            tasks: Vec::new(),
            failure_code: None,
            provider_exchange: None,
            execution_trace: vec![ExecutionTraceEvent {
                stage: "request_admitted".into(),
                outcome: "assessing".into(),
                elapsed_ms: None,
            }],
        };
        store.insert(run_id.clone(), run.clone());
        drop(store);

        // Once admitted, work owns its permit and completes even if HTTP disconnects.
        // This is bounded process-local work, not a durable queue.
        let service = self.clone();
        let handle = tokio::spawn(
            async move {
                let _permit = permit;
                service.execute(message, run, assessor).await
            }
            .in_current_span()
            .with_current_subscriber(),
        );
        match handle.await {
            Ok(result) => result,
            Err(_) => {
                let failure = AssessmentFailure::Execution;
                if let Some(run) = self.inner.runs.lock().await.get_mut(&run_id) {
                    run.state = RunState::Failed;
                    run.failure_code = Some(failure.code().into());
                    run.execution_trace.push(ExecutionTraceEvent {
                        stage: "supervisor_failed".into(),
                        outcome: failure.code().into(),
                        elapsed_ms: None,
                    });
                }
                tracing::error!(
                    event = "servicing_execution_failed",
                    run_id = %run_id,
                    failure_code = failure.code(),
                );
                Err(ServiceError::Assessment { failure, run_id })
            }
        }
    }

    #[tracing::instrument(skip_all, fields(run_id = %run.run_id, assessor = %run.assessor))]
    async fn execute(
        &self,
        message: Message,
        mut run: OperatorRun,
        assessor: Arc<dyn Assessor>,
    ) -> Result<CustomerReply, ServiceError> {
        let start = Instant::now();
        run.execution_trace.push(ExecutionTraceEvent {
            stage: "assessment_started".into(),
            outcome: "running".into(),
            elapsed_ms: Some(0),
        });
        tracing::info!(
            event = "assessment_attempt_started",
            run_id = %run.run_id,
            attempt = 1,
            assessor = %run.assessor,
            model = run.model.as_deref(),
            rubric_version = %run.rubric_version,
            routing_version = %run.routing_version,
        );
        let mut worker = tokio::spawn(
            async move { assessor.assess(&message).await }
                .in_current_span()
                .with_current_subscriber(),
        );
        let assessment = match tokio::time::timeout(
            Duration::from_secs(self.inner.limits.deadline_seconds),
            &mut worker,
        )
        .await
        {
            Ok(Ok(attempt)) => {
                run.provider_exchange = Some(attempt.exchange);
                attempt.result
            }
            Ok(Err(_)) => Err(AssessmentFailure::Execution),
            Err(_) => {
                worker.abort();
                // Wait for cancellation before releasing the admitted-work permit.
                match worker.await {
                    Err(error) if error.is_panic() => Err(AssessmentFailure::Execution),
                    _ => Err(AssessmentFailure::Timeout),
                }
            }
        };
        // The validated deadline bounds elapsed time well below u32 milliseconds.
        run.elapsed_ms = Some(u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX));
        let assessment_outcome = if assessment.is_ok() {
            "succeeded".to_owned()
        } else {
            assessment
                .as_ref()
                .err()
                .map(|failure| failure.code().to_owned())
                .unwrap_or_else(|| "failed".into())
        };
        run.execution_trace.push(ExecutionTraceEvent {
            stage: "assessment_completed".into(),
            outcome: assessment_outcome,
            elapsed_ms: run.elapsed_ms,
        });
        let outcome = match assessment {
            Ok(assessment) => {
                let plan = self.inner.policy.plan(&assessment);
                run.signals = plan
                    .signals
                    .into_iter()
                    .map(|signal| IntentSignal {
                        intent: signal.intent,
                        probability: signal.probability,
                        matched: signal.matched,
                    })
                    .collect();
                run.tasks = plan
                    .tasks
                    .into_iter()
                    .map(|intent| ServicingTask {
                        intent,
                        state: TaskState::ReviewRequired,
                    })
                    .collect();
                if let Some(usage) = assessment.usage() {
                    run.input_tokens = Some(usage.input);
                    run.output_tokens = Some(usage.output);
                }
                run.state = if run.tasks.is_empty() {
                    RunState::ClarificationRequired
                } else {
                    RunState::ReviewRequired
                };
                run.execution_trace.push(ExecutionTraceEvent {
                    stage: "routing_completed".into(),
                    outcome: match run.state {
                        RunState::ClarificationRequired => "clarification_required",
                        RunState::ReviewRequired => "review_required",
                        RunState::Assessing => "assessing",
                        RunState::Failed => "failed",
                    }
                    .into(),
                    elapsed_ms: run.elapsed_ms,
                });
                run.execution_trace.push(ExecutionTraceEvent {
                    stage: "run_completed".into(),
                    outcome: "succeeded".into(),
                    elapsed_ms: run.elapsed_ms,
                });
                Ok(CustomerReply {
                    run_id: run.run_id.clone(),
                    state: run.state,
                    reply: if run.tasks.is_empty() {
                        "We couldn't identify a specific insurance need in this message. Add details about your claim, policy, contact information, or payment question."
                    } else {
                        "This message may relate to the insurance areas listed below. Review the assessment; no insurance action has been taken."
                    }.into(),
                    tasks: run.tasks.clone(),
                    execution_trace: run.execution_trace.clone(),
                })
            }
            Err(failure) => {
                run.state = RunState::Failed;
                run.failure_code = Some(failure.code().into());
                run.execution_trace.push(ExecutionTraceEvent {
                    stage: "run_failed".into(),
                    outcome: failure.code().into(),
                    elapsed_ms: run.elapsed_ms,
                });
                Err(ServiceError::Assessment {
                    failure,
                    run_id: run.run_id.clone(),
                })
            }
        };
        tracing::info!(
            event = "assessment_attempt_completed",
            run_id = %run.run_id,
            attempt = 1,
            outcome = ?run.state,
            assessor = %run.assessor,
            model = run.model.as_deref(),
            rubric_version = %run.rubric_version,
            routing_version = %run.routing_version,
            elapsed_ms = run.elapsed_ms,
            failure_code = run.failure_code.as_deref(),
            input_tokens = run.input_tokens,
            output_tokens = run.output_tokens,
        );
        self.inner.runs.lock().await.insert(run.run_id.clone(), run);
        outcome
    }

    pub async fn runs(&self) -> Vec<OperatorRun> {
        self.inner
            .runs
            .lock()
            .await
            .values()
            .cloned()
            .rev()
            .collect()
    }

    pub async fn run_details(&self) -> Vec<OperatorRunDetail> {
        self.inner
            .runs
            .lock()
            .await
            .values()
            .cloned()
            .rev()
            .map(|run| OperatorRunDetail {
                provider_exchange: run.provider_exchange.clone(),
                execution_trace: run.execution_trace.clone(),
                run,
            })
            .collect()
    }

    pub fn assessor_options(&self) -> Vec<AssessorOption> {
        AssessorId::ALL
            .into_iter()
            .map(|id| {
                let available = self.inner.assessors.contains_key(&id);
                let (label, unavailable_reason) = match id {
                    AssessorId::Code => ("Code (keyword baseline)", None),
                    AssessorId::Jev => (
                        "Jev",
                        (!available).then_some("Configure TYPESAFE_API_KEY and restart the API."),
                    ),
                    AssessorId::Llm => (
                        "LLM",
                        Some("LLM provider has not been selected or configured."),
                    ),
                };
                AssessorOption {
                    id,
                    label: label.into(),
                    available,
                    unavailable_reason: unavailable_reason.map(str::to_owned),
                }
            })
            .collect()
    }
}
