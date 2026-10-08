#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use jev_sample::{
    http::{OperatorAuth, workflow_router},
    workflow::{
        contracts::*,
        decision::{
            DecisionAttempt, DecisionFuture, DecisionProvider, DecisionValue,
            DeterministicProvider, ProviderFailure, Question, VendorDecisions,
        },
        planner::{OpenAiPlanner, Planner, PlannerFuture, validate_plan},
        policy::{Gate, WorkflowPolicy},
        service::WorkflowService,
        store::WorkflowStore,
    },
};
use serde_json::json;
use tower::ServiceExt;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_partial_json, method, path},
};

struct TestDb(PathBuf);
static NEXT_DB: AtomicUsize = AtomicUsize::new(0);
impl TestDb {
    fn new() -> Self {
        use std::os::unix::fs::PermissionsExt;
        let path = std::env::temp_dir().join(format!(
            "zipclaim-test-{}-{:?}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT_DB.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir(&path).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        Self(path)
    }
    fn path(&self) -> PathBuf {
        self.0.join("workflow.sqlite")
    }
}
impl Drop for TestDb {
    fn drop(&mut self) {
        for name in ["workflow.sqlite", "workflow.sqlite-journal"] {
            let file = self.0.join(name);
            if file.exists() {
                std::fs::remove_file(file).unwrap();
            }
        }
        std::fs::remove_dir(&self.0).unwrap();
    }
}
fn policy() -> WorkflowPolicy {
    WorkflowPolicy::from_json(include_str!("../config/workflow.json")).unwrap()
}
fn task(id: &str, kind: WorkflowTaskKind, name: &str, depends: &[&str]) -> WorkflowTask {
    WorkflowTask {
        id: id.into(),
        kind,
        name: name.into(),
        depends_on: depends.iter().map(|s| (*s).into()).collect(),
        state: "pending".into(),
    }
}
fn plan(route: bool) -> WorkflowPlan {
    let mut tasks = vec![
        task("lookup", WorkflowTaskKind::Tool, "synthetic_record", &[]),
        task(
            "complete",
            WorkflowTaskKind::Decision,
            "synthetic_complete",
            &["lookup"],
        ),
    ];
    if route {
        tasks.push(task(
            "route",
            WorkflowTaskKind::Decision,
            "synthetic_route",
            &["complete"],
        ));
    }
    WorkflowPlan {
        plan_id: String::new(),
        tasks,
    }
}
#[derive(Default)]
struct MockPlanner {
    calls: AtomicUsize,
    route: bool,
    wait: bool,
    panic: bool,
}
impl Planner for MockPlanner {
    fn model(&self) -> String {
        "mock-planner-pinned".into()
    }
    fn plan<'a>(&'a self, _context: &'a str) -> PlannerFuture<'a, WorkflowPlan> {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::SeqCst);
            assert!(!self.panic, "synthetic planner panic");
            if self.wait {
                std::future::pending::<()>().await;
            }
            Ok((
                plan(self.route),
                WorkflowUsage {
                    input_tokens: Some(12),
                    output_tokens: Some(8),
                    attempts: 1,
                },
            ))
        })
    }
    fn reply<'a>(&'a self, context: &'a str) -> PlannerFuture<'a, String> {
        Box::pin(async move {
            assert!(context.contains("synthetic_record"));
            Ok((
                "Synthetic results recorded; no external action.".into(),
                WorkflowUsage {
                    input_tokens: Some(4),
                    output_tokens: Some(5),
                    attempts: 1,
                },
            ))
        })
    }
}
struct MockDecision {
    fail: bool,
}
impl DecisionProvider for MockDecision {
    fn id(&self) -> ProviderId {
        ProviderId::Openai
    }
    fn model(&self) -> Option<String> {
        Some("mock-decision-pinned".into())
    }
    fn capability(&self) -> String {
        "Synthetic test provider".into()
    }
    fn evaluate<'a>(&'a self, _context: &'a str, questions: &'a [Question]) -> DecisionFuture<'a> {
        Box::pin(async move {
            let answers = if self.fail {
                Err(ProviderFailure::RateLimited)
            } else {
                Ok(questions
                    .iter()
                    .map(|q| {
                        (
                            q.id.clone(),
                            DecisionValue::Predicate {
                                probability: 0.99,
                                confidence: None,
                            },
                        )
                    })
                    .collect())
            };
            DecisionAttempt {
                answers,
                usage: WorkflowUsage {
                    attempts: 1,
                    input_tokens: Some(3),
                    output_tokens: None,
                },
                elapsed_ms: 2,
            }
        })
    }
}
async fn service(
    db: &TestDb,
    planner: Arc<dyn Planner>,
    with_mock: bool,
) -> (WorkflowService, WorkflowStore) {
    let mut providers: BTreeMap<ProviderId, Arc<dyn DecisionProvider>> = BTreeMap::new();
    providers.insert(ProviderId::Code, Arc::new(DeterministicProvider));
    if with_mock {
        providers.insert(ProviderId::Openai, Arc::new(MockDecision { fail: false }));
    }
    let policy = policy();
    let store = WorkflowStore::open(&db.path(), &policy).await.unwrap();
    (
        WorkflowService::new(Some(planner), providers, policy, store.clone()),
        store,
    )
}
fn submission(key: &str, message: &str, providers: Vec<ProviderId>) -> WorkflowSubmission {
    WorkflowSubmission {
        client_request_id: key.into(),
        message: message.into(),
        providers,
    }
}

#[tokio::test]
async fn setup_is_authorized_validated_redacted_once_and_memory_only() {
    let db = TestDb::new();
    let policy = policy();
    let store = WorkflowStore::open(&db.path(), &policy).await.unwrap();
    let workflow = WorkflowService::new(
        None,
        BTreeMap::from([(
            ProviderId::Code,
            Arc::new(DeterministicProvider) as Arc<dyn DecisionProvider>,
        )]),
        policy.clone(),
        store.clone(),
    );
    let app = workflow_router(
        workflow.clone(),
        OperatorAuth::new(Some("setup-operator-key-at-least-32-bytes".into())).unwrap(),
    );
    let body = json!({"api_key":"fixture-private-api-key", "model":"fixture-model", "jev_api_key":"fixture-private-jev-key"});
    let request = |value: serde_json::Value, authorized: bool, origin: &str| {
        let mut builder = Request::builder()
            .method("POST")
            .uri("/v1/operator/workflows/setup")
            .header("content-type", "application/json")
            .header("origin", origin);
        if authorized {
            builder = builder.header(
                "authorization",
                "Bearer setup-operator-key-at-least-32-bytes",
            );
        }
        builder.body(Body::from(value.to_string())).unwrap()
    };
    let unauthorized = app
        .clone()
        .oneshot(request(body.clone(), false, "http://127.0.0.1:5173"))
        .await
        .unwrap();
    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);
    let unconfigured = workflow_router(workflow.clone(), OperatorAuth::new(None).unwrap())
        .oneshot(request(body.clone(), true, "http://127.0.0.1:5173"))
        .await
        .unwrap();
    assert_eq!(unconfigured.status(), StatusCode::SERVICE_UNAVAILABLE);
    let denied = app
        .clone()
        .oneshot(request(body.clone(), true, "https://untrusted.example"))
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    for invalid in [
        json!({"api_key":"fixture-private-api-key", "model":"fixture-model", "jev_api_key":""}),
        json!({"api_key":"fixture-private-api-key", "model":"fixture-model", "jev_api_key":"bad\nkey"}),
        json!({"api_key":"fixture-private-api-key", "model":"fixture-model", "jev_api_key":"a".repeat(513)}),
        json!({"api_key":"", "model":"fixture-model", "jev_api_key":"fixture-jev-key"}),
        json!({"api_key":"fixture-private-api-key", "model":"bad model\n", "jev_api_key":"fixture-jev-key"}),
        json!({"api_key":"bad\nkey", "model":"fixture-model", "jev_api_key":"fixture-jev-key"}),
        json!({"api_key":"a".repeat(513), "model":"fixture-model", "jev_api_key":"fixture-jev-key"}),
        json!({"api_key":"fixture-private-api-key", "model":"a".repeat(101), "jev_api_key":"fixture-jev-key"}),
        json!({"api_key":"fixture-private-api-key", "model":"fixture-model", "jev_api_key":"fixture-jev-key", "endpoint":"https://untrusted.example"}),
    ] {
        let response = app
            .clone()
            .oneshot(request(invalid, true, "http://127.0.0.1:5173"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(!workflow.options().planner.available);
    }
    let response = app
        .clone()
        .oneshot(request(body.clone(), true, "http://127.0.0.1:5173"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get("cache-control").unwrap(), "no-store");
    let encoded = to_bytes(response.into_body(), 8192).await.unwrap();
    assert!(!String::from_utf8_lossy(&encoded).contains("fixture-private-api-key"));
    assert!(!String::from_utf8_lossy(&encoded).contains("fixture-private-jev-key"));
    assert!(workflow.options().planner.available);
    assert!(
        workflow
            .options()
            .providers
            .iter()
            .any(|p| p.id == ProviderId::Jev && p.available)
    );
    assert_eq!(
        workflow.options().planner.model.as_deref(),
        Some("fixture-model")
    );
    let response = app
        .oneshot(request(body, true, "http://127.0.0.1:5173"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert!(workflow.details().await.unwrap().is_empty());
    let restarted = WorkflowService::new(None, BTreeMap::new(), policy, store);
    assert!(!restarted.options().planner.available);
    assert!(
        !restarted
            .options()
            .providers
            .iter()
            .any(|p| p.id == ProviderId::Jev && p.available)
    );
    assert!(
        !std::fs::read(db.path())
            .unwrap()
            .windows(b"fixture-private-api-key".len())
            .any(|part| part == b"fixture-private-api-key")
    );
    assert!(
        !std::fs::read(db.path())
            .unwrap()
            .windows(b"fixture-private-jev-key".len())
            .any(|part| part == b"fixture-private-jev-key")
    );
}

#[tokio::test]
async fn concurrent_setup_accepts_exactly_one_planner_without_replacement() {
    let db = TestDb::new();
    let policy = policy();
    let store = WorkflowStore::open(&db.path(), &policy).await.unwrap();
    let workflow = WorkflowService::new(None, BTreeMap::new(), policy, store);
    let workers: Vec<_> = (0..4)
        .map(|index| {
            let service = workflow.clone();
            std::thread::spawn(move || {
                service.configure_planner(PlannerSetup {
                    api_key: "fixture-api-key".into(),
                    model: format!("fixture-model-{index}"),
                    jev_api_key: "fixture-jev-key".into(),
                })
            })
        })
        .collect();
    let mut accepted = 0;
    for worker in workers {
        match worker.join().unwrap() {
            Ok(_) => accepted += 1,
            Err(jev_sample::workflow::service::PlannerSetupError::AlreadyConfigured) => {}
            Err(other) => panic!("unexpected setup error: {other:?}"),
        }
    }
    assert_eq!(accepted, 1);
    assert!(workflow.options().planner.available);
    assert!(
        workflow
            .options()
            .providers
            .iter()
            .any(|p| p.id == ProviderId::Jev && p.available)
    );
}

#[tokio::test]
async fn setup_fills_only_missing_connections_without_replacing_startup_provenance() {
    let db = TestDb::new();
    let policy = policy();
    let store = WorkflowStore::open(&db.path(), &policy).await.unwrap();
    let workflow = WorkflowService::new(
        Some(Arc::new(MockPlanner::default())),
        BTreeMap::new(),
        policy.clone(),
        store.clone(),
    );
    let before = workflow.options().planner.model;
    assert!(matches!(
        workflow.configure_planner(PlannerSetup {
            api_key: "replacement".into(),
            model: "replacement".into(),
            jev_api_key: "fixture-jev-key".into(),
        }),
        Err(jev_sample::workflow::service::PlannerSetupError::Invalid)
    ));
    assert!(
        !workflow
            .options()
            .providers
            .iter()
            .any(|p| p.id == ProviderId::Jev && p.available)
    );
    workflow
        .configure_planner(PlannerSetup {
            api_key: "".into(),
            model: "".into(),
            jev_api_key: "fixture-jev-key".into(),
        })
        .unwrap();
    assert_eq!(workflow.options().planner.model, before);
    assert!(
        workflow
            .options()
            .providers
            .iter()
            .any(|p| p.id == ProviderId::Jev && p.available)
    );
    let jev = Arc::new(
        VendorDecisions::for_test(
            "http://127.0.0.1:1/unused".into(),
            "fixture-startup-key",
            "fixture-startup-jev",
            ProviderId::Jev,
            Duration::from_secs(1),
        )
        .unwrap(),
    ) as Arc<dyn DecisionProvider>;
    let startup_jev = WorkflowService::new(
        None,
        BTreeMap::from([(ProviderId::Jev, jev.clone())]),
        policy,
        store,
    );
    assert!(matches!(
        startup_jev.configure_planner(PlannerSetup {
            api_key: "fixture-api-key".into(),
            model: "fixture-model".into(),
            jev_api_key: "replacement".into(),
        }),
        Err(jev_sample::workflow::service::PlannerSetupError::Invalid)
    ));
    assert!(!startup_jev.options().planner.available);
    startup_jev
        .configure_planner(PlannerSetup {
            api_key: "fixture-api-key".into(),
            model: "fixture-model".into(),
            jev_api_key: "".into(),
        })
        .unwrap();
    assert_eq!(
        startup_jev
            .options()
            .providers
            .iter()
            .find(|p| p.id == ProviderId::Jev)
            .unwrap()
            .model,
        jev.model()
    );
}

#[test]
fn plans_reject_cycles_unknown_tools_duplicate_dependencies_and_unbounded_work() {
    let good = plan(false);
    assert!(validate_plan(&good, 8));
    let mut invalid = good.clone();
    invalid.tasks[0].depends_on.push("complete".into());
    assert!(!validate_plan(&invalid, 8));
    invalid = good.clone();
    invalid.tasks[0].name = "send_money".into();
    assert!(!validate_plan(&invalid, 8));
    invalid = good.clone();
    invalid.tasks[1].depends_on.push("lookup".into());
    assert!(!validate_plan(&invalid, 8));
    invalid = good.clone();
    invalid.tasks[1].id = "lookup".into();
    assert!(!validate_plan(&invalid, 8));
    assert!(!validate_plan(&good, 1));
}
#[test]
fn thresholds_and_negative_predicates_are_not_low_confidence_or_permission() {
    let p = policy();
    let q = Question::named("synthetic_complete").unwrap();
    for (value, gate) in [
        (0.899, Gate::Clarify),
        (0.9, Gate::Proceed),
        (0.02, Gate::Clarify),
    ] {
        assert_eq!(
            p.gate(
                ProviderId::Openai,
                &q,
                &DecisionValue::Predicate {
                    probability: value,
                    confidence: None
                }
            ),
            gate
        );
    }
    assert_eq!(
        p.gate(
            ProviderId::Code,
            &Question::named("synthetic_route").unwrap(),
            &DecisionValue::Unsupported
        ),
        Gate::Review
    );
    let mut bad = p;
    bad.rules[0].min_probability = f64::NAN;
    assert!(bad.validate().is_err());
}
#[tokio::test]
async fn shared_plan_decision_identity_usage_and_history_survive_reopen() {
    let db = TestDb::new();
    let planner = Arc::new(MockPlanner::default());
    let (service, store) = service(&db, planner.clone(), true).await;
    let result = service
        .submit(submission(
            "same",
            "Read synthetic SYN-42.",
            vec![ProviderId::Code, ProviderId::Openai],
        ))
        .await
        .unwrap();
    assert!(result.complete);
    assert_eq!(planner.calls.load(Ordering::SeqCst), 1);
    for run in &result.runs {
        for event in &run.event_trace {
            assert!(
                event
                    .recorded_at_ms
                    .is_some_and(|time| time >= event.occurred_at_ms)
            );
            if event.actor == "planner" {
                assert_eq!(event.provider, Some(ProviderId::Openai));
                assert_eq!(event.model.as_deref(), Some("mock-planner-pinned"));
            }
            if event.stage == "decision_completed" {
                assert_eq!(event.provider, Some(run.provider));
                assert_eq!(event.model, run.model);
            }
        }
    }
    let detail = service.details().await.unwrap().remove(0);
    assert_eq!(detail.decisions.len(), 2);
    assert_eq!(
        detail.decisions[0].decision_input_key,
        detail.decisions[1].decision_input_key
    );
    assert_eq!(result.runs[1].usage.output_tokens, None);
    assert!(
        result
            .runs
            .iter()
            .all(|r| r.event_trace.iter().any(|e| e.stage == "tool_completed"))
    );
    assert!(result.runs.iter().all(|r| {
        r.event_trace
            .iter()
            .enumerate()
            .all(|(i, e)| e.sequence == i as u32 + 1 && e.comparison_id == result.comparison_id)
    }));
    drop(service);
    drop(store);
    let reopened = WorkflowStore::open(&db.path(), &policy()).await.unwrap();
    assert_eq!(
        reopened.list().await.unwrap()[0]
            .detail
            .comparison
            .comparison_id,
        result.comparison_id
    );
    assert!(reopened.list().await.unwrap()[0].detail.comparison.complete);
}
#[tokio::test]
async fn duplicate_admissions_do_not_repeat_planning_and_collisions_fail() {
    let db = TestDb::new();
    let planner = Arc::new(MockPlanner::default());
    let (service, _) = service(&db, planner.clone(), false).await;
    let a = service
        .submit(submission(
            "idempotent",
            "Read SYN-42.",
            vec![ProviderId::Code],
        ))
        .await
        .unwrap();
    let b = service
        .submit(submission(
            "idempotent",
            "Read SYN-42.",
            vec![ProviderId::Code],
        ))
        .await
        .unwrap();
    assert_eq!(a.comparison_id, b.comparison_id);
    assert_eq!(planner.calls.load(Ordering::SeqCst), 1);
    assert!(
        service
            .submit(submission(
                "idempotent",
                "Read SYN-43.",
                vec![ProviderId::Code]
            ))
            .await
            .is_err()
    );
}
#[tokio::test]
async fn clarification_replans_only_selected_run_and_employee_review_is_scoped() {
    let db = TestDb::new();
    let planner = Arc::new(MockPlanner {
        route: true,
        ..Default::default()
    });
    let (service, _) = service(&db, planner.clone(), false).await;
    let a = service
        .submit(submission(
            "clarify",
            "Please show my fictional record.",
            vec![ProviderId::Code],
        ))
        .await
        .unwrap();
    assert_eq!(a.runs[0].state, WorkflowState::Clarification);
    let request = WorkflowResume {
        run_id: a.runs[0].run_id.clone(),
        expected_plan_id: a.runs[0].plan_id.clone(),
        expected_task_id: "complete".into(),
        message: "Reference SYN-42".into(),
        client_request_id: "resume1".into(),
        employee_review: false,
    };
    let b = service
        .resume(a.comparison_id.clone(), request.clone())
        .await
        .unwrap();
    assert_eq!(b.runs[0].state, WorkflowState::EmployeeReview);
    assert_ne!(a.runs[0].plan_id, b.runs[0].plan_id);
    assert!(
        service
            .resume(
                a.comparison_id.clone(),
                WorkflowResume {
                    run_id: b.runs[0].run_id.clone(),
                    expected_plan_id: a.runs[0].plan_id.clone(),
                    expected_task_id: "route".into(),
                    message: "Stale employee review".into(),
                    client_request_id: "stale".into(),
                    employee_review: true,
                }
            )
            .await
            .is_err()
    );
    assert_eq!(planner.calls.load(Ordering::SeqCst), 2);
    assert!(
        b.runs[0]
            .event_trace
            .iter()
            .any(|e| e.stage == "replan_completed")
    );
    let again = service
        .resume(a.comparison_id.clone(), request)
        .await
        .unwrap();
    assert_eq!(again.runs[0].event_trace.len(), b.runs[0].event_trace.len());
    let c = service
        .resume(
            a.comparison_id,
            WorkflowResume {
                run_id: b.runs[0].run_id.clone(),
                expected_plan_id: b.runs[0].plan_id.clone(),
                expected_task_id: "route".into(),
                message: "Employee confirms read-only demonstration continuation.".into(),
                client_request_id: "resume2".into(),
                employee_review: true,
            },
        )
        .await
        .unwrap();
    assert_eq!(c.runs[0].state, WorkflowState::Completed);
    assert_eq!(c.mode, "shared_base_plan_with_divergent_resumes");
    assert!(c.runs[0].tasks.iter().any(|t| t.state == "reviewed"));
}
#[tokio::test]
async fn technical_provider_failure_is_not_uncertainty_or_another_members_failure() {
    let db = TestDb::new();
    let policy = policy();
    let store = WorkflowStore::open(&db.path(), &policy).await.unwrap();
    let providers: BTreeMap<_, Arc<dyn DecisionProvider>> = BTreeMap::from([
        (
            ProviderId::Code,
            Arc::new(DeterministicProvider) as Arc<dyn DecisionProvider>,
        ),
        (
            ProviderId::Openai,
            Arc::new(MockDecision { fail: true }) as Arc<dyn DecisionProvider>,
        ),
    ]);
    let service = WorkflowService::new(
        Some(Arc::new(MockPlanner::default())),
        providers,
        policy,
        store,
    );
    let result = service
        .submit(submission(
            "failure",
            "SYN-42",
            vec![ProviderId::Code, ProviderId::Openai],
        ))
        .await
        .unwrap();
    assert_eq!(result.runs[0].state, WorkflowState::Completed);
    assert_eq!(result.runs[1].state, WorkflowState::Failed);
    assert_eq!(
        result.runs[1].failure_code.as_deref(),
        Some("decision_rate_limited")
    );
    assert!(!service.details().await.unwrap()[0].decisions[1].complete);
}
#[tokio::test]
async fn deadline_survives_http_caller_cancellation_and_panic_is_recorded() {
    let db = TestDb::new();
    let mut policy = policy();
    policy.deadline_ms = 100;
    let store = WorkflowStore::open(&db.path(), &policy).await.unwrap();
    let providers: BTreeMap<_, Arc<dyn DecisionProvider>> = BTreeMap::from([(
        ProviderId::Code,
        Arc::new(DeterministicProvider) as Arc<dyn DecisionProvider>,
    )]);
    let service = WorkflowService::new(
        Some(Arc::new(MockPlanner {
            wait: true,
            ..Default::default()
        })),
        providers.clone(),
        policy.clone(),
        store.clone(),
    );
    let client = tokio::spawn({
        let service = service.clone();
        async move {
            service
                .submit(submission("disconnect", "SYN-42", vec![ProviderId::Code]))
                .await
        }
    });
    for _ in 0..20 {
        if !store.list().await.unwrap().is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    client.abort();
    tokio::time::sleep(Duration::from_millis(200)).await;
    let data = store.list().await.unwrap();
    assert_eq!(
        data[0].detail.comparison.runs[0].state,
        WorkflowState::Failed
    );
    assert_eq!(
        data[0].detail.comparison.failure_code.as_deref(),
        Some("workflow_deadline")
    );
    let panic_service = WorkflowService::new(
        Some(Arc::new(MockPlanner {
            panic: true,
            ..Default::default()
        })),
        providers,
        policy,
        store,
    );
    let failed = panic_service
        .submit(submission("panic", "SYN-42", vec![ProviderId::Code]))
        .await
        .unwrap();
    assert_eq!(failed.runs[0].state, WorkflowState::Failed);
}
#[tokio::test]
async fn startup_marks_unfinished_work_interrupted_without_external_replay() {
    let db = TestDb::new();
    let (service, store) = service(&db, Arc::new(MockPlanner::default()), false).await;
    let result = service
        .submit(submission("restart", "SYN-42", vec![ProviderId::Code]))
        .await
        .unwrap();
    let mut unfinished = store.get(result.comparison_id.clone()).await.unwrap();
    unfinished.detail.comparison.runs[0].state = WorkflowState::Running;
    store.save(unfinished).await.unwrap();
    drop(service);
    drop(store);
    let reopened = WorkflowStore::open(&db.path(), &policy()).await.unwrap();
    let interrupted = reopened.get(result.comparison_id).await.unwrap();
    assert_eq!(
        interrupted.detail.comparison.runs[0].state,
        WorkflowState::Interrupted
    );
    assert!(
        interrupted.detail.comparison.runs[0]
            .event_trace
            .iter()
            .any(|e| e.stage == "workflow_interrupted")
    );
    assert!(!interrupted.detail.comparison.complete);
    let event = interrupted.detail.comparison.runs[0]
        .event_trace
        .last()
        .unwrap();
    assert_eq!(event.actor, "runtime");
    assert_eq!(event.provider, None);
    assert_eq!(event.model, None);
}
#[tokio::test]
async fn retention_and_file_permissions_fail_closed() {
    use std::os::unix::fs::PermissionsExt;
    let db = TestDb::new();
    let mut p = policy();
    p.max_comparisons = 1;
    let store = WorkflowStore::open(&db.path(), &p).await.unwrap();
    let providers: BTreeMap<_, Arc<dyn DecisionProvider>> = BTreeMap::from([(
        ProviderId::Code,
        Arc::new(DeterministicProvider) as Arc<dyn DecisionProvider>,
    )]);
    let service = WorkflowService::new(Some(Arc::new(MockPlanner::default())), providers, p, store);
    service
        .submit(submission("one", "SYN-42", vec![ProviderId::Code]))
        .await
        .unwrap();
    assert!(
        service
            .submit(submission("two", "SYN-42", vec![ProviderId::Code]))
            .await
            .is_err()
    );
    drop(service);
    assert_eq!(
        std::fs::metadata(db.path()).unwrap().permissions().mode() & 0o077,
        0
    );
    std::fs::set_permissions(db.path(), std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(WorkflowStore::open(&db.path(), &policy()).await.is_err());
}
#[tokio::test]
async fn public_routes_exclude_protected_context_and_resume_requires_authorization() {
    let db = TestDb::new();
    let (service, _) = service(&db, Arc::new(MockPlanner::default()), false).await;
    let app = workflow_router(
        service,
        OperatorAuth::new(Some("test-operator-key-at-least-32-bytes".into())).unwrap(),
    );
    let unauth = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/operator/workflows")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauth.status(), StatusCode::UNAUTHORIZED);
    let request = json!({"message":"SYN-42","providers":["code"],"client_request_id":"http1"});
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/workflows")
                .header("content-type", "application/json")
                .body(Body::from(request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 100000).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(result.get("message").is_none());
    assert!(result.get("decisions").is_none());
    assert!(result.get("protected_context_json").is_none());
    let resume = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/operator/workflows/comparison-1/resume")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resume.status(), StatusCode::UNAUTHORIZED);
    let cross_origin = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/workflows")
                .header("origin", "https://attacker.invalid")
                .header("content-type", "application/json")
                .body(Body::from(request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cross_origin.status(), StatusCode::FORBIDDEN);
}
#[tokio::test]
async fn deterministic_malformed_context_fails_and_unsupported_is_not_probability() {
    let provider = DeterministicProvider;
    let q = Question::named("synthetic_complete").unwrap();
    assert!(provider.evaluate("not json", &[q]).await.answers.is_err());
    let q = Question::named("synthetic_route").unwrap();
    assert!(matches!(
        provider
            .evaluate(r#"{"message":"SYN-42"}"#, &[q])
            .await
            .answers
            .unwrap()["synthetic_route"],
        DecisionValue::Unsupported
    ));
}
#[tokio::test]
async fn jev_and_openai_wire_shapes_are_vendor_specific_but_normalize_the_same_question() {
    let server = MockServer::start().await;
    let q = Question::named("synthetic_priority").unwrap();
    Mock::given(method("POST")).and(path("/jev")).and(body_partial_json(json!({"questions":{"synthetic_priority":{"type":"score","criteria":["routine","urgent"]}}})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"model":"jev-1.13.0","answers":{"synthetic_priority":{"type":"score","score":0.1,"probabilities":{"0":0.9,"1":0.1},"legend":{"0":"routine","1":"urgent"},"confidence":0.8}},"usage":{"input_tokens":10,"output_tokens":4}}))).expect(1).mount(&server).await;
    let jev = VendorDecisions::for_test(
        format!("{}/jev", server.uri()),
        "mock-key",
        "jev-1.13.0",
        ProviderId::Jev,
        Duration::from_secs(1),
    )
    .unwrap();
    let answer = jev
        .evaluate(r#"{"message":"SYN-42"}"#, std::slice::from_ref(&q))
        .await;
    assert!(
        matches!(answer.answers.unwrap()["synthetic_priority"],DecisionValue::Score{value,..} if value==0.1)
    );
    Mock::given(method("POST")).and(path("/openai")).and(body_partial_json(json!({"questions":[{"name":"synthetic_priority","type":"score","levels":[{"label":"routine","description":"routine"},{"label":"urgent","description":"urgent"}]}]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"model":"mock-decisions","answers":[{"name":"synthetic_priority","type":"score","score":0.1,"probabilities":[{"value":0,"label":"routine","probability":0.9},{"value":1,"label":"urgent","probability":0.1}],"confidence":0.8}],"usage":{"input_tokens":10}}))).expect(1).mount(&server).await;
    let openai = VendorDecisions::for_test(
        format!("{}/openai", server.uri()),
        "mock-key",
        "mock-decisions",
        ProviderId::Openai,
        Duration::from_secs(1),
    )
    .unwrap();
    let answer = openai.evaluate(r#"{"message":"SYN-42"}"#, &[q]).await;
    assert!(answer.answers.is_ok());
    assert_eq!(answer.usage.output_tokens, None);
}
#[tokio::test]
async fn responses_planner_requires_completed_named_structured_output_and_disables_remote_storage()
{
    let server = MockServer::start().await;
    Mock::given(method("POST")).and(path("/responses")).and(body_partial_json(json!({"model":"mock-planner","store":false,"text":{"format":{"type":"json_schema","strict":true}}})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status":"completed","model":"mock-planner","output":[{"type":"message","content":[{"type":"output_text","text":r#"{"tasks":[{"id":"check","kind":"decision","name":"synthetic_complete","depends_on":[]}]}"#}]}],"usage":{"input_tokens":1,"output_tokens":2}}))).expect(1).mount(&server).await;
    let planner = OpenAiPlanner::for_test(
        format!("{}/responses", server.uri()),
        "mock-key",
        "mock-planner",
        Duration::from_secs(1),
    )
    .unwrap();
    let (plan, usage) = planner.plan("synthetic input").await.unwrap();
    assert!(validate_plan(&plan, 8));
    assert_eq!(usage.output_tokens, Some(2));
}

#[tokio::test]
async fn recorded_events_are_immutable_and_oversized_updates_roll_back() {
    let db = TestDb::new();
    let (service, store) = service(&db, Arc::new(MockPlanner::default()), false).await;
    let result = service
        .submit(submission("immutable", "SYN-42", vec![ProviderId::Code]))
        .await
        .unwrap();
    let original = store.get(result.comparison_id.clone()).await.unwrap();
    let mut altered = original.clone();
    altered.detail.comparison.runs[0].event_trace[0].outcome = "changed".into();
    assert!(store.save(altered).await.is_err());
    let mut oversized = original;
    oversized.detail.message = "x".repeat(9 * 1024 * 1024);
    assert!(store.save(oversized).await.is_err());
    let remaining = store.get(result.comparison_id).await.unwrap();
    assert_eq!(remaining.detail.message, "SYN-42");
    assert_eq!(
        remaining.detail.comparison.runs[0].event_trace[0].outcome,
        "running"
    );
}

#[tokio::test]
async fn vendor_authentication_refusal_and_oversized_responses_are_distinct() {
    let server = MockServer::start().await;
    let q = Question::named("synthetic_complete").unwrap();
    for (endpoint, response) in [
        ("auth", ResponseTemplate::new(401)),
        (
            "refusal",
            ResponseTemplate::new(200).set_body_json(
                json!({"model":"mock-model","answers":[{"name":q.id,"type":"refusal"}]}),
            ),
        ),
        (
            "oversized",
            ResponseTemplate::new(200).set_body_string("x".repeat(65537)),
        ),
    ] {
        Mock::given(method("POST"))
            .and(path(format!("/{endpoint}")))
            .respond_with(response)
            .expect(1)
            .mount(&server)
            .await;
        let vendor = VendorDecisions::for_test(
            format!("{}/{endpoint}", server.uri()),
            "mock-key",
            "mock-model",
            ProviderId::Openai,
            Duration::from_secs(1),
        )
        .unwrap();
        let attempt = vendor.evaluate("SYN-42", std::slice::from_ref(&q)).await;
        match endpoint {
            "auth" => assert!(matches!(
                attempt.answers,
                Err(ProviderFailure::Authentication)
            )),
            "refusal" => assert!(matches!(
                attempt.answers.unwrap()[&q.id],
                DecisionValue::Refusal
            )),
            "oversized" => assert!(matches!(
                attempt.answers,
                Err(ProviderFailure::ResponseTooLarge)
            )),
            _ => unreachable!(),
        }
        assert_eq!(attempt.usage.input_tokens, None);
    }
}

#[tokio::test]
async fn authorized_operator_can_inspect_protected_decision_context() {
    let db = TestDb::new();
    let (service, _) = service(&db, Arc::new(MockPlanner::default()), false).await;
    service
        .submit(submission(
            "protected",
            "SYN-42 fictional-narrative",
            vec![ProviderId::Code],
        ))
        .await
        .unwrap();
    let app = workflow_router(
        service,
        OperatorAuth::new(Some("test-operator-key-at-least-32-bytes".into())).unwrap(),
    );
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/operator/workflows")
                .header(
                    "authorization",
                    "Bearer test-operator-key-at-least-32-bytes",
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 100000).await.unwrap();
    let text = std::str::from_utf8(&body).unwrap();
    assert!(text.contains("fictional-narrative"));
    assert!(text.contains("question_json"));
}
