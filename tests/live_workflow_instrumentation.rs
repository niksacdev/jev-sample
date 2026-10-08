use jev_sample::{
    telemetry::LiveTelemetry,
    workflow::{
        contracts::{
            ProviderId, WorkflowPlan, WorkflowSubmission, WorkflowTask, WorkflowTaskKind,
            WorkflowUsage,
        },
        decision::DeterministicProvider,
        planner::{Planner, PlannerFuture},
        policy::WorkflowPolicy,
        service::WorkflowService,
        store::WorkflowStore,
    },
};
use std::{collections::BTreeMap, os::unix::fs::PermissionsExt, sync::Arc, time::Duration};
use tracing::instrument::WithSubscriber;
use tracing_subscriber::prelude::*;

struct GatedPlanner(Arc<tokio::sync::Notify>);
impl Planner for GatedPlanner {
    fn model(&self) -> String {
        "mock-planner-pinned".into()
    }
    fn plan<'a>(&'a self, _: &'a str) -> PlannerFuture<'a, WorkflowPlan> {
        Box::pin(async move {
            self.0.notified().await;
            Ok((
                WorkflowPlan {
                    plan_id: String::new(),
                    tasks: vec![WorkflowTask {
                        id: "reference".into(),
                        kind: WorkflowTaskKind::Decision,
                        name: "synthetic_complete".into(),
                        depends_on: vec![],
                        state: "pending".into(),
                    }],
                },
                WorkflowUsage::default(),
            ))
        })
    }
    fn reply<'a>(&'a self, _: &'a str) -> PlannerFuture<'a, String> {
        Box::pin(async { Ok(("Fixture reply.".into(), WorkflowUsage::default())) })
    }
}

// Separate process isolates tracing callsite-interest caches from unrelated workflow tests.
#[tokio::test]
async fn actual_planner_start_streams_before_the_plan_returns()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let directory =
        std::env::temp_dir().join(format!("zipclaim-live-workflow-{}", std::process::id()));
    std::fs::create_dir(&directory)?;
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700))?;
    let policy = WorkflowPolicy::from_json(include_str!("../config/workflow.json"))?;
    let store = WorkflowStore::open(&directory.join("workflow.sqlite"), &policy).await?;
    let gate = Arc::new(tokio::sync::Notify::new());
    let planner: Arc<dyn Planner> = Arc::new(GatedPlanner(gate.clone()));
    let decisions: BTreeMap<_, Arc<dyn jev_sample::workflow::decision::DecisionProvider>> =
        BTreeMap::from([(
            ProviderId::Code,
            Arc::new(DeterministicProvider)
                as Arc<dyn jev_sample::workflow::decision::DecisionProvider>,
        )]);
    let workflow = WorkflowService::new(Some(planner), decisions, policy, store);
    let logs = LiveTelemetry::default();
    let (_, mut receiver) = logs.subscribe()?;
    let subscriber = tracing_subscriber::registry().with(logs);
    let worker = tokio::spawn(
        async move {
            workflow
                .submit(WorkflowSubmission {
                    client_request_id: "live-plan".into(),
                    message: "SYN-42".into(),
                    providers: vec![ProviderId::Code],
                })
                .await
        }
        .with_subscriber(subscriber),
    );
    let entry = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let entry = receiver.recv().await?;
            if entry
                .fields
                .get("stage")
                .is_some_and(|s| s == "plan_started")
            {
                return Ok::<_, tokio::sync::broadcast::error::RecvError>(entry);
            }
        }
    })
    .await??;
    assert_eq!(entry.fields.get("actor").ok_or("missing actor")?, "planner");
    assert_eq!(
        entry.fields.get("model").ok_or("missing model")?,
        "mock-planner-pinned"
    );
    assert!(!worker.is_finished());
    gate.notify_one();
    let result = worker
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| format!("{e:?}"))?;
    assert!(result.complete);
    assert!(
        result.runs[0]
            .event_trace
            .iter()
            .any(|e| e.stage == "plan_completed")
    );
    std::fs::remove_file(directory.join("workflow.sqlite"))?;
    std::fs::remove_dir(directory)?;
    Ok(())
}
