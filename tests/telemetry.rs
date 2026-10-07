use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};

use jev_sample::{
    application::{ExecutionLimits, ServicingService},
    assessment::{
        AssessmentAttempt, AssessmentFailure, AssessmentFuture, Assessor, Provenance,
        ProviderExchange,
    },
    domain::{Assessment, Evidence, Intent, Message, Observation},
    routing::RoutingPolicy,
};
use opentelemetry::trace::TracerProvider as _;
use tracing::{Instrument, instrument::WithSubscriber};
use tracing_subscriber::prelude::*;

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);

impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .map_err(|_| io::Error::other("capture poisoned"))?
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct TelemetryAssessor {
    fail: bool,
}

impl Assessor for TelemetryAssessor {
    fn provenance(&self) -> Provenance {
        Provenance {
            assessor: "telemetry_test".into(),
            model: Some("fixture-model".into()),
            rubric_version: "fixture-rubric".into(),
        }
    }
    fn assess<'a>(&'a self, _: &'a Message) -> AssessmentFuture<'a> {
        Box::pin(
            async move {
                if self.fail {
                    return AssessmentAttempt::failure(
                        AssessmentFailure::Authentication,
                        ProviderExchange {
                            request_body: "private-narrative-marker".into(),
                            ..ProviderExchange::default()
                        },
                    );
                }
                match Assessment::new(
                    Intent::ALL
                        .into_iter()
                        .map(|intent| Observation {
                            intent,
                            evidence: Evidence::KeywordMatch(intent == Intent::Claim),
                        })
                        .collect(),
                    None,
                ) {
                    Ok(assessment) => AssessmentAttempt::success(
                        assessment,
                        ProviderExchange {
                            request_body: "private-narrative-marker".into(),
                            response_body: Some("fixture".into()),
                            ..ProviderExchange::default()
                        },
                    ),
                    Err(_) => AssessmentAttempt::failure(
                        AssessmentFailure::InvalidResponse,
                        ProviderExchange::default(),
                    ),
                }
            }
            .instrument(tracing::info_span!("provider_assessment")),
        )
    }
}

#[tokio::test]
async fn spawned_assessment_spans_preserve_execution_parentage()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let exporter = opentelemetry_sdk::trace::InMemorySpanExporter::default();
    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let subscriber = tracing_subscriber::registry()
        .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("test")));
    let service = ServicingService::new(
        Arc::new(TelemetryAssessor { fail: false }),
        RoutingPolicy::from_json(include_str!("../config/routing.json"))?,
        ExecutionLimits::from_json(include_str!("../config/execution.json"))?,
    );
    async {
        service
            .submit("private-narrative-marker".into())
            .instrument(tracing::info_span!("http_request"))
            .await
    }
    .with_subscriber(subscriber)
    .await
    .map_err(|error| format!("{error:?}"))?;
    provider.force_flush()?;
    let spans = exporter.get_finished_spans()?;
    let http = spans
        .iter()
        .find(|span| span.name == "http_request")
        .ok_or("missing HTTP span")?;
    let execution = spans
        .iter()
        .find(|span| span.name == "execute")
        .ok_or("missing execution span")?;
    let submission = spans
        .iter()
        .find(|span| span.name == "submit_with")
        .ok_or("missing submission span")?;
    let assessment = spans
        .iter()
        .find(|span| span.name == "provider_assessment")
        .ok_or("missing provider span")?;
    assert_eq!(submission.parent_span_id, http.span_context.span_id());
    assert_eq!(execution.parent_span_id, submission.span_context.span_id());
    assert_eq!(assessment.parent_span_id, execution.span_context.span_id());
    assert_eq!(
        assessment.span_context.trace_id(),
        http.span_context.trace_id()
    );
    assert!(
        execution
            .attributes
            .iter()
            .any(|attribute| attribute.key.as_str() == "run_id"
                && attribute.value.to_string() == "run-0001")
    );
    assert!(!format!("{spans:?}").contains("private-narrative-marker"));
    provider.shutdown()?;
    Ok(())
}

#[tokio::test]
async fn attempt_events_include_provenance_outcomes_and_no_narrative()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for fail in [false, true] {
        let capture = Capture::default();
        let writer = capture.clone();
        let subscriber = tracing_subscriber::fmt()
            .json()
            .with_ansi(false)
            .with_writer(move || writer.clone())
            .finish();
        let service = ServicingService::new(
            Arc::new(TelemetryAssessor { fail }),
            RoutingPolicy::from_json(include_str!("../config/routing.json"))?,
            ExecutionLimits::from_json(include_str!("../config/execution.json"))?,
        );
        let result = service
            .submit("private-narrative-marker".into())
            .with_subscriber(subscriber)
            .await;
        assert_eq!(result.is_err(), fail);
        let bytes = capture.0.lock().map_err(|_| "capture poisoned")?.clone();
        let text = String::from_utf8(bytes)?;
        assert!(!text.contains("private-narrative-marker"));
        let events = text
            .lines()
            .map(serde_json::from_str::<serde_json::Value>)
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["fields"]["event"], "assessment_attempt_started");
        assert_eq!(events[1]["fields"]["event"], "assessment_attempt_completed");
        for event in &events {
            let fields = &event["fields"];
            assert_eq!(fields["run_id"], "run-0001");
            assert_eq!(fields["attempt"], 1);
            assert_eq!(fields["model"], "fixture-model");
            assert_eq!(fields["assessor"], "telemetry_test");
            assert_eq!(fields["rubric_version"], "fixture-rubric");
            assert_eq!(fields["routing_version"], "experiment-review-only-v1");
            for forbidden in [
                "message",
                "state_text",
                "api_key",
                "authorization",
                "provider_body",
            ] {
                assert!(fields.get(forbidden).is_none());
            }
        }
        assert!(events[1]["fields"]["elapsed_ms"].is_number());
        if fail {
            assert_eq!(
                events[1]["fields"]["failure_code"],
                "provider_authentication"
            );
            assert_eq!(events[1]["fields"]["outcome"], "Failed");
        } else {
            assert_eq!(events[1]["fields"]["outcome"], "ReviewRequired");
        }
    }
    Ok(())
}
