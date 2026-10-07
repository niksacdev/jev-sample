use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use jev_sample::{
    application::{ExecutionLimits, ServiceError, ServicingService},
    assessment::{
        AssessmentAttempt, AssessmentFailure, AssessmentFuture, Assessor, Provenance,
        ProviderExchange,
    },
    contracts::{RunState, TaskState},
    domain::{Assessment, Evidence, Intent, Message, Observation, Probability},
    routing::RoutingPolicy,
};
use tokio::sync::{Notify, Semaphore};

mod support;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[tokio::test]
async fn same_http_transport_accepts_baseline_and_test_provider() -> TestResult {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    let injected = support::service(Arc::new(TestAssessor {
        calls: AtomicUsize::new(0),
        failure: None,
    }));
    for (router, expected_state) in [
        (support::router(), "clarification_required"),
        (support::router_for(injected), "review_required"),
    ] {
        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/messages")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"message":"test request"}"#))?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 8192).await?)?;
        assert_eq!(body["state"], expected_state);
        assert!(body.get("model").is_none());
    }
    Ok(())
}

struct TestAssessor {
    calls: AtomicUsize,
    failure: Option<AssessmentFailure>,
}

impl Assessor for TestAssessor {
    fn provenance(&self) -> Provenance {
        Provenance {
            assessor: "test_assessor".into(),
            model: Some("test-model-v1".into()),
            rubric_version: "test-rubric-v1".into(),
        }
    }

    fn assess<'a>(&'a self, message: &'a Message) -> AssessmentFuture<'a> {
        Box::pin(async move {
            assert_eq!(message.as_str(), "test request");
            self.calls.fetch_add(1, Ordering::SeqCst);
            let exchange = ProviderExchange {
                request_body: "test request".into(),
                response_status: None,
                response_body: Some("fixture response".into()),
                response_truncated: false,
            };
            if let Some(failure) = self.failure {
                return AssessmentAttempt::failure(failure, exchange);
            }
            let observations = Intent::ALL
                .into_iter()
                .map(|intent| {
                    Probability::new(if intent == Intent::Claim { 0.85 } else { 0.1 }).map(|p| {
                        Observation {
                            intent,
                            evidence: Evidence::YesProbability(p),
                        }
                    })
                })
                .collect::<Result<Vec<_>, _>>();
            let observations = match observations {
                Ok(observations) => observations,
                Err(_) => {
                    return AssessmentAttempt::failure(
                        AssessmentFailure::InvalidResponse,
                        exchange,
                    );
                }
            };
            match Assessment::new(observations, None) {
                Ok(assessment) => AssessmentAttempt::success(assessment, exchange),
                Err(_) => AssessmentAttempt::failure(AssessmentFailure::InvalidResponse, exchange),
            }
        })
    }
}

#[tokio::test]
async fn same_use_case_runs_through_an_injected_assessor_and_policy() -> TestResult {
    let assessor = Arc::new(TestAssessor {
        calls: AtomicUsize::new(0),
        failure: None,
    });
    let service = support::service(assessor.clone());
    let reply = service
        .submit("test request".into())
        .await
        .map_err(|e| format!("{e:?}"))?;
    assert_eq!(assessor.calls.load(Ordering::SeqCst), 1);
    assert_eq!(reply.state, RunState::ReviewRequired);
    assert_eq!(reply.tasks.len(), 1);
    assert_eq!(reply.tasks[0].state, TaskState::ReviewRequired);
    let runs = service.runs().await;
    assert_eq!(runs[0].assessor, "test_assessor");
    assert_eq!(runs[0].model.as_deref(), Some("test-model-v1"));
    assert_eq!(runs[0].rubric_version, "test-rubric-v1");
    assert!(runs[0].input_tokens.is_none());

    let stricter = ServicingService::new(
        assessor,
        RoutingPolicy::from_json(r#"{"version":"test-strict-v2","intent_display_threshold":0.9}"#)?,
        ExecutionLimits::from_json(include_str!("../config/execution.json"))?,
    );
    let reply = stricter
        .submit("test request".into())
        .await
        .map_err(|e| format!("{e:?}"))?;
    assert_eq!(reply.state, RunState::ClarificationRequired);
    assert!(reply.tasks.is_empty());
    assert_eq!(stricter.runs().await[0].routing_version, "test-strict-v2");
    Ok(())
}

#[tokio::test]
async fn rejected_message_does_not_call_assessor_and_provider_failure_does_not_fallback() {
    let assessor = Arc::new(TestAssessor {
        calls: AtomicUsize::new(0),
        failure: Some(AssessmentFailure::RateLimited),
    });
    let service = support::service(assessor.clone());
    assert!(matches!(
        service.submit(" ".into()).await,
        Err(ServiceError::InvalidMessage(_))
    ));
    assert_eq!(assessor.calls.load(Ordering::SeqCst), 0);
    assert!(service.runs().await.is_empty());
    assert!(matches!(
        service.submit("test request".into()).await,
        Err(ServiceError::Assessment {
            failure: AssessmentFailure::RateLimited,
            ..
        })
    ));
    assert_eq!(assessor.calls.load(Ordering::SeqCst), 1);
    let runs = service.runs().await;
    assert_eq!(runs[0].state, RunState::Failed);
    assert_eq!(
        runs[0].failure_code.as_deref(),
        Some("provider_rate_limited")
    );
    assert!(runs[0].tasks.is_empty());
}

struct ControlledAssessor {
    started: Notify,
    release: Semaphore,
}

impl Assessor for ControlledAssessor {
    fn provenance(&self) -> Provenance {
        Provenance {
            assessor: "controlled_test".into(),
            model: None,
            rubric_version: "v1".into(),
        }
    }
    fn assess<'a>(&'a self, _: &'a Message) -> AssessmentFuture<'a> {
        Box::pin(async move {
            self.started.notify_one();
            let permit = match self.release.acquire().await {
                Ok(permit) => permit,
                Err(_) => {
                    return AssessmentAttempt::failure(
                        AssessmentFailure::Execution,
                        ProviderExchange::default(),
                    );
                }
            };
            permit.forget();
            match Assessment::new(
                Intent::ALL
                    .into_iter()
                    .map(|intent| Observation {
                        intent,
                        evidence: Evidence::KeywordMatch(false),
                    })
                    .collect(),
                None,
            ) {
                Ok(assessment) => AssessmentAttempt::success(
                    assessment,
                    ProviderExchange {
                        request_body: "controlled request".into(),
                        response_body: Some("controlled response".into()),
                        ..ProviderExchange::default()
                    },
                ),
                Err(_) => AssessmentAttempt::failure(
                    AssessmentFailure::InvalidResponse,
                    ProviderExchange::default(),
                ),
            }
        })
    }
}

#[tokio::test]
async fn caller_cancellation_preserves_bounded_work_and_final_state() -> TestResult {
    let assessor = Arc::new(ControlledAssessor {
        started: Notify::new(),
        release: Semaphore::new(0),
    });
    let service = ServicingService::new(
        assessor.clone(),
        RoutingPolicy::from_json(include_str!("../config/routing.json"))?,
        ExecutionLimits::from_json(r#"{"max_concurrent":1,"max_runs":2,"deadline_seconds":15}"#)?,
    );
    let caller = tokio::spawn({
        let service = service.clone();
        async move { service.submit("request".into()).await }
    });
    assessor.started.notified().await;
    caller.abort();
    assert_eq!(service.runs().await[0].state, RunState::Assessing);
    assert!(matches!(
        service.submit("another".into()).await,
        Err(ServiceError::CapacityExceeded)
    ));
    assessor.release.add_permits(1);
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        while service.runs().await[0].state == RunState::Assessing {
            tokio::task::yield_now().await;
        }
    })
    .await?;
    assert_eq!(
        service.runs().await[0].state,
        RunState::ClarificationRequired
    );
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn workflow_deadline_cancels_stalled_provider_and_releases_capacity() -> TestResult {
    let assessor = Arc::new(ControlledAssessor {
        started: Notify::new(),
        release: Semaphore::new(0),
    });
    let service = support::service(assessor.clone());
    let reply = service.submit("request".into()).await;
    assert!(matches!(
        reply,
        Err(ServiceError::Assessment {
            failure: AssessmentFailure::Timeout,
            ..
        })
    ));
    let failed = &service.runs().await[0];
    assert_eq!(failed.state, RunState::Failed);
    assert_eq!(failed.failure_code.as_deref(), Some("provider_timeout"));
    assert!(failed.provider_exchange.is_none());
    assessor.release.add_permits(1);
    let next = service.submit("request".into()).await;
    assert!(next.is_ok());
    assert_eq!(assessor.release.available_permits(), 0);
    Ok(())
}

struct PanickingAssessor;
impl Assessor for PanickingAssessor {
    fn provenance(&self) -> Provenance {
        Provenance {
            assessor: "panic_test".into(),
            model: None,
            rubric_version: "v1".into(),
        }
    }
    fn assess<'a>(&'a self, _: &'a Message) -> AssessmentFuture<'a> {
        Box::pin(async { panic!("deliberate provider panic for regression test") })
    }
}

#[tokio::test]
async fn provider_panic_is_a_visible_failed_run() {
    let service = support::service(Arc::new(PanickingAssessor));
    assert!(matches!(
        service.submit("request".into()).await,
        Err(ServiceError::Assessment {
            failure: AssessmentFailure::Execution,
            ..
        })
    ));
    assert_eq!(service.runs().await[0].state, RunState::Failed);
}

#[test]
fn invalid_execution_limits_fail_configuration() {
    for config in [
        r#"{"max_concurrent":0,"max_runs":100,"deadline_seconds":15}"#,
        r#"{"max_concurrent":4,"max_runs":0,"deadline_seconds":15}"#,
        r#"{"max_concurrent":4,"max_runs":100,"deadline_seconds":0}"#,
        r#"{"max_concurrent":4,"max_runs":100,"deadline_seconds":3601}"#,
        r#"{"max_concurrent":4,"max_runs":100}"#,
    ] {
        assert!(ExecutionLimits::from_json(config).is_err());
    }
}

#[test]
fn workflow_and_domain_dependency_boundaries_stay_explicit() {
    let application = include_str!("../src/application.rs");
    for dependency in ["axum", "reqwest", "crate::jev", "crate::baseline", "env::"] {
        assert!(
            !application.contains(dependency),
            "Application must not depend on {dependency}"
        );
    }
    for pure_module in [
        include_str!("../src/domain.rs"),
        include_str!("../src/routing.rs"),
    ] {
        for dependency in ["tokio", "reqwest", "axum", "std::env", "std::time"] {
            assert!(
                !pure_module.contains(dependency),
                "Pure module must not depend on {dependency}"
            );
        }
    }
    let http = include_str!("../src/http.rs");
    for dependency in [
        "crate::jev",
        "crate::baseline",
        "assessment::Assessor",
        "tokio::spawn",
        ".plan(",
    ] {
        assert!(
            !http.contains(dependency),
            "HTTP must not coordinate {dependency}"
        );
    }
}
