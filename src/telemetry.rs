//! Bounded, allowlisted live instrumentation. Never forwards arbitrary log bodies.

use std::{
    collections::{BTreeMap, VecDeque},
    io,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use tokio::sync::{Semaphore, broadcast};
use tracing::{Event, Subscriber, field::Visit};
use tracing_subscriber::{Layer, layer::Context};
use ts_rs::TS;

pub const HISTORY_LIMIT: usize = 256;

#[derive(Clone, Debug, Serialize, TS)]
pub struct LiveLog {
    #[ts(type = "number")]
    pub sequence: u64,
    #[ts(type = "number")]
    pub occurred_at_ms: u64,
    pub level: String,
    pub fields: BTreeMap<String, String>,
}

struct Buffer {
    sequence: u64,
    history: VecDeque<LiveLog>,
}

#[derive(Clone)]
pub struct LiveTelemetry {
    buffer: Arc<Mutex<Buffer>>,
    sender: broadcast::Sender<LiveLog>,
    pub(crate) clients: Arc<Semaphore>,
}

impl Default for LiveTelemetry {
    fn default() -> Self {
        let (sender, _) = broadcast::channel(HISTORY_LIMIT);
        Self {
            buffer: Arc::new(Mutex::new(Buffer {
                sequence: 0,
                history: VecDeque::new(),
            })),
            sender,
            clients: Arc::new(Semaphore::new(4)),
        }
    }
}

impl LiveTelemetry {
    pub fn subscribe(&self) -> io::Result<(VecDeque<LiveLog>, broadcast::Receiver<LiveLog>)> {
        let buffer = self
            .buffer
            .lock()
            .map_err(|_| io::Error::other("live telemetry buffer unavailable"))?;
        let receiver = self.sender.subscribe();
        Ok((buffer.history.clone(), receiver))
    }

    fn record(&self, level: &str, fields: BTreeMap<String, String>) -> io::Result<()> {
        let now = u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(io::Error::other)?
                .as_millis(),
        )
        .map_err(io::Error::other)?;
        let mut buffer = self
            .buffer
            .lock()
            .map_err(|_| io::Error::other("live telemetry buffer unavailable"))?;
        buffer.sequence = buffer
            .sequence
            .checked_add(1)
            .ok_or_else(|| io::Error::other("live telemetry sequence exhausted"))?;
        let entry = LiveLog {
            sequence: buffer.sequence,
            occurred_at_ms: now,
            level: level.to_owned(),
            fields,
        };
        if buffer.history.len() == HISTORY_LIMIT {
            buffer.history.pop_front();
        }
        buffer.history.push_back(entry.clone());
        // No subscriber is normal; recent entries remain in the bounded buffer.
        let _ = self.sender.send(entry);
        Ok(())
    }
}

#[derive(Default)]
struct Fields(BTreeMap<String, String>);

impl Fields {
    fn insert(&mut self, name: &str, value: &str) {
        let name = match name {
            "failure" | "failure_code" => "code",
            name => name,
        };
        if matches!(
            name,
            "event"
                | "comparison_id"
                | "run_id"
                | "task_id"
                | "actor"
                | "provider"
                | "run_provider"
                | "model"
                | "sequence"
                | "stage"
                | "outcome"
                | "policy_version"
                | "assessor"
                | "rubric_version"
                | "routing_version"
                | "elapsed_ms"
                | "input_tokens"
                | "output_tokens"
                | "attempt"
                | "code"
                | "route"
                | "status"
        ) {
            let sanitized: String = value
                .chars()
                .filter(|c| !c.is_control())
                .take(256)
                .collect();
            self.0.insert(name.to_owned(), sanitized);
        }
    }
}

impl Visit for Fields {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.insert(field.name(), value);
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.insert(field.name(), &format!("{value:?}"));
    }
}

impl<S: Subscriber> Layer<S> for LiveTelemetry {
    fn on_event(&self, event: &Event<'_>, _: Context<'_, S>) {
        let mut fields = Fields::default();
        event.record(&mut fields);
        if !fields.0.get("event").is_some_and(|name| {
            matches!(
                name.as_str(),
                "workflow_event"
                    | "decision_attempt_finished"
                    | "assessment_attempt_started"
                    | "assessment_attempt_completed"
                    | "assessment_worker_panicked"
                    | "workflow_request_failed"
                    | "http_request_received"
                    | "http_request_finished"
            )
        }) {
            return;
        }
        if self
            .record(event.metadata().level().as_str(), fields.0)
            .is_err()
        {
            eprintln!("Live instrumentation unavailable: an event could not be recorded.");
        }
    }
}
