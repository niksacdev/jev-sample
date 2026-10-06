//! Loopback-only synthetic experiment. Persona views are not authentication.

use std::{collections::BTreeMap, sync::Arc, time::Instant};

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use serde::Serialize;
use tokio::sync::{Mutex, Semaphore};

use crate::{
    agent::{self, ROUTING_VERSION, RUBRIC_VERSION},
    contracts::{ApiError, CustomerMessage, CustomerReply, OperatorRun, RunState},
    jev::{Jev, MODEL},
};

#[derive(Clone)]
pub struct Application {
    inner: Arc<Runtime>,
}

struct Runtime {
    jev: Option<Jev>,
    runs: Mutex<BTreeMap<String, OperatorRun>>,
    capacity: Semaphore,
}

impl Application {
    pub fn new(jev: Option<Jev>) -> Self {
        Self {
            inner: Arc::new(Runtime {
                jev,
                runs: Mutex::new(BTreeMap::new()),
                capacity: Semaphore::new(4),
            }),
        }
    }
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

pub fn router() -> Router {
    router_with(Application::new(None))
}

pub fn router_with(application: Application) -> Router {
    Router::new()
        .route("/health", get(|| async { Json(Health { status: "ok" }) }))
        .route("/v1/messages", post(message))
        .route("/v1/operator/runs", get(runs))
        .layer(DefaultBodyLimit::max(8192))
        .with_state(application)
}

type HttpError = (StatusCode, Json<ApiError>);

fn error(status: StatusCode, code: &str, message: &str, run_id: Option<String>) -> HttpError {
    (
        status,
        Json(ApiError {
            code: code.into(),
            message: message.into(),
            run_id,
        }),
    )
}

async fn message(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<CustomerMessage>, JsonRejection>,
) -> Result<Json<CustomerReply>, HttpError> {
    if headers.get("origin").is_some_and(|origin| {
        origin != "http://127.0.0.1:5173" && origin != "http://localhost:5173"
    }) {
        return Err(error(
            StatusCode::FORBIDDEN,
            "origin_denied",
            "Browser origin is not allowed.",
            None,
        ));
    }
    let Json(input) = body.map_err(|rejection| {
        error(
            rejection.status(),
            "invalid_request",
            "Expected JSON with one message field (maximum 8192 bytes).",
            None,
        )
    })?;
    if !agent::validate_message(&input.message) {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_message",
            "Use a nonempty synthetic message of at most 4000 UTF-8 bytes.",
            None,
        ));
    }
    let _permit = app.inner.capacity.try_acquire().map_err(|_| {
        error(
            StatusCode::TOO_MANY_REQUESTS,
            "capacity_exceeded",
            "Four assessments are already running. Try again later.",
            None,
        )
    })?;
    let mut store = app.inner.runs.lock().await;
    if store.len() >= 100 {
        return Err(error(
            StatusCode::TOO_MANY_REQUESTS,
            "run_limit",
            "This local session has reached its 100-run limit. Restart the sample to clear its ephemeral history.",
            None,
        ));
    }
    let run_id = format!("run-{:04}", store.len() + 1);
    let mut run = OperatorRun {
        run_id: run_id.clone(),
        state: RunState::Assessing,
        assessor: if app.inner.jev.is_some() {
            "jev"
        } else {
            "keyword_baseline"
        }
        .into(),
        model: app.inner.jev.as_ref().map(|_| MODEL.into()),
        rubric_version: RUBRIC_VERSION.into(),
        routing_version: ROUTING_VERSION.into(),
        elapsed_ms: None,
        input_tokens: None,
        output_tokens: None,
        signals: Vec::new(),
        tasks: Vec::new(),
        failure_code: None,
    };
    store.insert(run_id.clone(), run.clone());
    drop(store);
    let start = Instant::now();
    let assessment = if let Some(jev) = &app.inner.jev {
        jev.assess(&input.message).await
    } else {
        Ok(crate::jev::Assessment {
            signals: agent::rules(&input.message),
            input_tokens: 0,
            output_tokens: 0,
        })
    };
    run.elapsed_ms = Some(u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX));
    match assessment {
        Ok(assessment) => {
            run.signals = assessment.signals;
            if app.inner.jev.is_some() {
                run.input_tokens = Some(assessment.input_tokens);
                run.output_tokens = Some(assessment.output_tokens);
            }
            run.tasks = agent::plan(&run.signals);
            run.state = if run.tasks.is_empty() {
                RunState::ClarificationRequired
            } else {
                RunState::ReviewRequired
            };
            let reply = CustomerReply {
                run_id: run_id.clone(),
                state: run.state,
                reply: if run.tasks.is_empty() {
                    "I'd like to understand a little more. Please clarify whether you need help with a claim, your policy, contact details or a premium payment."
                } else {
                    "I've organized your request into the next steps below. They're ready for review, and you can follow their status here."
                }.into(),
                tasks: run.tasks.clone(),
            };
            eprintln!(
                "run_id={run_id} state={:?} assessor={} elapsed_ms={:?}",
                run.state, run.assessor, run.elapsed_ms
            );
            app.inner.runs.lock().await.insert(run_id, run);
            Ok(Json(reply))
        }
        Err(failure) => {
            run.state = RunState::Failed;
            run.failure_code = Some(failure.code().into());
            eprintln!("run_id={run_id} state=failed code={}", failure.code());
            app.inner.runs.lock().await.insert(run_id.clone(), run);
            Err(error(
                StatusCode::BAD_GATEWAY,
                failure.code(),
                "The assessment failed. No servicing action was executed. Inspect the operator view; there is no automatic fallback or retry.",
                Some(run_id),
            ))
        }
    }
}

async fn runs(State(app): State<Application>) -> Json<Vec<OperatorRun>> {
    Json(
        app.inner
            .runs
            .lock()
            .await
            .values()
            .cloned()
            .rev()
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fifth_active_request_is_rejected_without_a_run()
    -> Result<(), Box<dyn std::error::Error>> {
        let app = Application::new(None);
        let permits = (0..4)
            .map(|_| app.inner.capacity.try_acquire())
            .collect::<Result<Vec<_>, _>>()?;
        let result = message(
            State(app.clone()),
            HeaderMap::new(),
            Ok(Json(CustomerMessage {
                message: "synthetic claim".into(),
            })),
        )
        .await;
        assert!(matches!(result, Err((StatusCode::TOO_MANY_REQUESTS, _))));
        assert!(app.inner.runs.lock().await.is_empty());
        drop(permits);
        assert!(
            message(
                State(app),
                HeaderMap::new(),
                Ok(Json(CustomerMessage {
                    message: "synthetic claim".into()
                }))
            )
            .await
            .is_ok()
        );
        Ok(())
    }
}
