use std::{future::Future, pin::Pin, sync::Arc, time::SystemTime};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    contracts::{OperatorWorkflow, WorkflowEvent, WorkflowState},
    policy::WorkflowPolicy,
};

pub mod sqlite;

#[derive(Clone, Deserialize, Serialize)]
pub struct Snapshot {
    pub detail: OperatorWorkflow,
    pub policy: WorkflowPolicy,
    pub context: String,
    pub run_contexts: std::collections::BTreeMap<String, String>,
}

#[derive(Debug)]
pub struct StoreError;
impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("workflow storage failed or exhausted")
    }
}
impl std::error::Error for StoreError {}

pub enum Admission {
    New(String),
    Existing(Box<Snapshot>),
}

pub type RepositoryFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, StoreError>> + Send + 'a>>;

pub trait WorkflowRepository: Send + Sync {
    fn admit<'a>(
        &'a self,
        request: String,
        payload: String,
        snapshot: Snapshot,
    ) -> RepositoryFuture<'a, Admission>;
    fn reserve_resume<'a>(
        &'a self,
        snapshot: Snapshot,
        request: String,
        payload: String,
    ) -> RepositoryFuture<'a, bool>;
    fn prior_request<'a>(
        &'a self,
        request: String,
        payload: String,
        comparison: String,
    ) -> RepositoryFuture<'a, bool>;
    fn save<'a>(&'a self, snapshot: Snapshot) -> RepositoryFuture<'a, ()>;
    fn get<'a>(&'a self, id: String) -> RepositoryFuture<'a, Snapshot>;
    fn list<'a>(&'a self) -> RepositoryFuture<'a, Vec<Snapshot>>;
}

#[derive(Clone)]
pub struct WorkflowStore {
    repository: Arc<dyn WorkflowRepository>,
}

impl WorkflowStore {
    pub fn new(repository: Arc<dyn WorkflowRepository>) -> Self {
        Self { repository }
    }

    pub async fn open(path: &std::path::Path, policy: &WorkflowPolicy) -> Result<Self, StoreError> {
        let store = Self::new(Arc::new(
            sqlite::SqliteRepository::open(path, policy).await?,
        ));
        recover_interrupted(&store).await?;
        Ok(store)
    }

    pub async fn admit(
        &self,
        request: String,
        payload: String,
        snapshot: Snapshot,
    ) -> Result<Admission, StoreError> {
        self.repository.admit(request, payload, snapshot).await
    }
    pub async fn reserve_resume(
        &self,
        snapshot: Snapshot,
        request: String,
        payload: String,
    ) -> Result<bool, StoreError> {
        self.repository
            .reserve_resume(snapshot, request, payload)
            .await
    }
    pub async fn prior_request(
        &self,
        request: String,
        payload: String,
        comparison: String,
    ) -> Result<bool, StoreError> {
        self.repository
            .prior_request(request, payload, comparison)
            .await
    }
    pub async fn save(&self, snapshot: Snapshot) -> Result<(), StoreError> {
        self.repository.save(snapshot).await
    }
    pub async fn get(&self, id: String) -> Result<Snapshot, StoreError> {
        self.repository.get(id).await
    }
    pub async fn list(&self) -> Result<Vec<Snapshot>, StoreError> {
        self.repository.list().await
    }
}

pub async fn recover_interrupted(store: &WorkflowStore) -> Result<(), StoreError> {
    for mut snapshot in store.list().await? {
        let mut changed = false;
        for run in &mut snapshot.detail.comparison.runs {
            if run.state == WorkflowState::Running {
                changed = true;
                run.state = WorkflowState::Interrupted;
                run.failure_code = Some("process_interrupted".into());
                run.reply = "The process stopped before the workflow completed. No automatic replay was performed.".into();
                run.event_trace.push(WorkflowEvent {
                    sequence: u32::try_from(run.event_trace.len() + 1).map_err(|_| StoreError)?,
                    comparison_id: snapshot.detail.comparison.comparison_id.clone(),
                    run_id: run.run_id.clone(),
                    task_id: None,
                    provider: None,
                    model: None,
                    actor: "runtime".into(),
                    plan_id: (!run.plan_id.is_empty()).then(|| run.plan_id.clone()),
                    policy_version: snapshot.policy.version.clone(),
                    schema_version: "workflow-events-1".into(),
                    stage: "workflow_interrupted".into(),
                    outcome: "interrupted".into(),
                    occurred_at_ms: epoch_ms()?,
                    recorded_at_ms: None,
                });
            }
        }
        if snapshot.detail.comparison.base_plan.tasks.is_empty()
            && snapshot.detail.comparison.failure_code.is_none()
        {
            snapshot.detail.comparison.failure_code = Some("planning_interrupted".into());
            changed = true;
        }
        if changed {
            snapshot.detail.comparison.complete = false;
            store.save(snapshot).await?;
        }
    }
    Ok(())
}

pub(super) fn parse_id(id: &str) -> Result<i64, StoreError> {
    id.strip_prefix("comparison-")
        .and_then(|value| value.parse().ok())
        .filter(|value: &i64| *value > 0)
        .ok_or(StoreError)
}

pub fn identity(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

pub fn epoch_ms() -> Result<f64, StoreError> {
    Ok(SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| StoreError)?
        .as_millis() as f64)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
pub(crate) async fn repository_contract(store: WorkflowStore) {
    use super::contracts::{
        ProviderId, WorkflowComparison, WorkflowPlan, WorkflowRun, WorkflowUsage,
    };

    fn snapshot() -> Snapshot {
        Snapshot {
            detail: OperatorWorkflow {
                comparison: WorkflowComparison {
                    comparison_id: String::new(),
                    input_key: "input-key".into(),
                    base_plan: WorkflowPlan {
                        plan_id: "plan-1".into(),
                        tasks: vec![],
                    },
                    runs: vec![WorkflowRun {
                        run_id: String::new(),
                        plan_id: "plan-1".into(),
                        provider: ProviderId::Code,
                        model: None,
                        state: WorkflowState::Completed,
                        reply: "done".into(),
                        tasks: vec![],
                        event_trace: vec![],
                        usage: WorkflowUsage::default(),
                        elapsed_ms: 0,
                        failure_code: None,
                    }],
                    mode: "test".into(),
                    complete: true,
                    planner_model: None,
                    planner_usage: WorkflowUsage::default(),
                    failure_code: None,
                },
                message: "contract".into(),
                decisions: vec![],
                protected_context_json: "{}".into(),
            },
            policy: WorkflowPolicy::from_json(include_str!("../../../config/workflow.json"))
                .expect("fixture policy"),
            context: "context".into(),
            run_contexts: Default::default(),
        }
    }

    let first = snapshot();
    let admitted = store
        .admit("request-1".into(), "payload-1".into(), first.clone())
        .await
        .expect("new admission");
    let comparison = match admitted {
        Admission::New(id) => id,
        Admission::Existing(_) => panic!("expected new admission"),
    };
    assert!(
        store
            .prior_request("request-1".into(), "payload-1".into(), comparison.clone())
            .await
            .expect("prior request")
    );
    assert!(
        store
            .prior_request(
                "request-1".into(),
                "payload-other".into(),
                comparison.clone()
            )
            .await
            .is_err()
    );
    assert!(matches!(
        store
            .admit("request-1".into(), "payload-1".into(), first.clone())
            .await
            .expect("idempotent admission"),
        Admission::Existing(_)
    ));
    assert!(
        store
            .admit("request-1".into(), "different".into(), first.clone())
            .await
            .is_err()
    );

    let mut saved = store.get(comparison.clone()).await.expect("get admitted");
    let event = WorkflowEvent {
        sequence: 1,
        comparison_id: comparison.clone(),
        run_id: saved.detail.comparison.runs[0].run_id.clone(),
        task_id: None,
        provider: None,
        model: None,
        actor: "contract".into(),
        plan_id: None,
        policy_version: saved.policy.version.clone(),
        schema_version: "workflow-events-1".into(),
        stage: "contract".into(),
        outcome: "recorded".into(),
        occurred_at_ms: 1.0,
        recorded_at_ms: None,
    };
    saved.detail.comparison.runs[0].event_trace.push(event);
    store.save(saved.clone()).await.expect("append event");
    saved.detail.comparison.runs[0].event_trace[0].outcome = "mutated".into();
    assert!(
        store.save(saved).await.is_err(),
        "stored events must be immutable"
    );

    let mut second = snapshot();
    second.detail.message = "second".into();
    let second_id = match store
        .admit("request-2".into(), "payload-2".into(), second)
        .await
        .expect("second admission")
    {
        Admission::New(id) => id,
        Admission::Existing(_) => panic!("expected new admission"),
    };
    let listed = store.list().await.expect("list snapshots");
    assert_eq!(listed[0].detail.comparison.comparison_id, second_id);
    assert_eq!(listed[1].detail.comparison.comparison_id, comparison);

    let mut resumed = store.get(second_id.clone()).await.expect("get for resume");
    resumed.detail.message = "resumed".into();
    assert!(
        store
            .reserve_resume(resumed.clone(), "request-3".into(), "payload-3".into())
            .await
            .expect("reserve resume")
    );
    assert!(
        !store
            .reserve_resume(resumed, "request-3".into(), "payload-3".into())
            .await
            .expect("idempotent resume")
    );
    assert!(
        store
            .prior_request("request-3".into(), "payload-3".into(), second_id.clone())
            .await
            .expect("resume prior request")
    );

    let mut third = snapshot();
    third.detail.message = "third".into();
    assert!(
        store
            .admit("request-4".into(), "payload-4".into(), third)
            .await
            .is_err(),
        "comparison quota must be enforced"
    );
}
