use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};

use jev_sample::{
    application::{ExecutionLimits, ServicingService},
    assessment::{AssessmentFailure, AssessmentFuture, Assessor, Provenance},
    domain::{Assessment, Evidence, Intent, Message, Observation},
    routing::RoutingPolicy,
};
use tracing::instrument::WithSubscriber;

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
        Box::pin(async move {
            if self.fail {
                return Err(AssessmentFailure::Authentication);
            }
            Assessment::new(
                Intent::ALL
                    .into_iter()
                    .map(|intent| Observation {
                        intent,
                        evidence: Evidence::KeywordMatch(intent == Intent::Claim),
                    })
                    .collect(),
                None,
            )
            .map_err(|_| AssessmentFailure::InvalidResponse)
        })
    }
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
