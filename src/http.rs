//! HTTP transport for the loopback-only servicing workspace.

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{
        Response, Sse,
        sse::{Event, KeepAlive},
    },
    routing::{get, post},
};
use serde::Serialize;
use subtle::ConstantTimeEq;

use crate::{
    application::{ServiceError, ServicingService},
    contracts::{
        ApiError, AssessorOption, CustomerMessage, CustomerReply, OperatorRun, OperatorRunDetail,
    },
};

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

#[derive(Clone)]
struct WorkflowHttpState {
    service: crate::workflow::service::WorkflowService,
    auth: OperatorAuth,
}

pub fn workflow_router(
    service: crate::workflow::service::WorkflowService,
    auth: OperatorAuth,
) -> Router {
    Router::new()
        .route("/v1/workflows/options", get(workflow_options))
        .route("/v1/workflows", post(workflow_submit))
        .route("/v1/operator/workflows/setup", post(workflow_setup))
        .route("/v1/operator/workflows", get(workflow_list))
        .route("/v1/operator/workflows/{id}/resume", post(workflow_resume))
        .layer(middleware::from_fn(instrument_request))
        .layer(DefaultBodyLimit::max(8192))
        .with_state(WorkflowHttpState { service, auth })
}

async fn instrument_request(request: axum::extract::Request, next: Next) -> Response {
    let path = request.uri().path();
    let route = match path {
        "/v1/workflows/options" => "workflow_options",
        "/v1/workflows" => "workflow_submit",
        "/v1/operator/workflows" => "workflow_inspect",
        "/v1/operator/workflows/setup" => "workflow_setup",
        "/v1/messages" => "assessment_submit",
        "/v1/assessors" => "assessor_options",
        "/v1/employee/runs" => "assessment_employee",
        "/v1/operator/runs" => "assessment_operator",
        _ if path.starts_with("/v1/operator/workflows/") && path.ends_with("/resume") => {
            "workflow_resume"
        }
        _ => "other",
    };
    let start = std::time::Instant::now();
    tracing::info!(event = "http_request_received", route);
    let response = next.run(request).await;
    tracing::info!(
        event = "http_request_finished",
        route,
        status = response.status().as_u16(),
        elapsed_ms = start.elapsed().as_millis() as u64
    );
    response
}

#[derive(Clone)]
struct TelemetryState {
    logs: crate::telemetry::LiveTelemetry,
    auth: OperatorAuth,
}

pub fn telemetry_router(logs: crate::telemetry::LiveTelemetry, auth: OperatorAuth) -> Router {
    Router::new()
        .route("/v1/operator/telemetry", get(live_logs))
        .with_state(TelemetryState { logs, auth })
}

async fn live_logs(
    State(state): State<TelemetryState>,
    headers: HeaderMap,
) -> Result<Response, HttpError> {
    workflow_origin(&headers)?;
    if state.auth.key.is_none() {
        return Err(error(
            StatusCode::SERVICE_UNAVAILABLE,
            "operator_auth_unconfigured",
            "Configure REASSURE_OPERATOR_KEY to stream instrumentation.",
            None,
        ));
    }
    if !state.auth.allows(&headers) {
        return Err(error(
            StatusCode::UNAUTHORIZED,
            "operator_auth_required",
            "Operator authorization is required for live instrumentation.",
            None,
        ));
    }
    let permit = state
        .logs
        .clients
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            error(
                StatusCode::TOO_MANY_REQUESTS,
                "telemetry_capacity",
                "Four instrumentation streams are already open.",
                None,
            )
        })?;
    let (history, receiver) = state.logs.subscribe().map_err(|_| {
        error(
            StatusCode::SERVICE_UNAVAILABLE,
            "telemetry_unavailable",
            "Live instrumentation is unavailable.",
            None,
        )
    })?;
    let first_sequence = history.front().map(|e| e.sequence);
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(900);
    let stream = futures_util::stream::unfold(
        (history, receiver, Some(permit), false),
        move |(mut history, mut receiver, mut permit, finished)| async move {
            if finished {
                return None;
            }
            let (frame, finished) = if tokio::time::Instant::now() >= deadline {
                (
                    Ok(Event::default()
                        .event("closed")
                        .data("Stream reached its 15-minute limit. Reconnect explicitly.")),
                    true,
                )
            } else if let Some(entry) = history.pop_front() {
                (
                    Event::default()
                        .event("log")
                        .id(entry.sequence.to_string())
                        .json_data(entry),
                    false,
                )
            } else {
                tokio::select! {
                    biased;
                    _ = tokio::time::sleep_until(deadline) => (
                        Ok(Event::default().event("closed").data("Stream reached its 15-minute limit. Reconnect explicitly.")), true
                    ),
                    value = receiver.recv() => match value {
                        Ok(entry) => (Event::default().event("log").id(entry.sequence.to_string()).json_data(entry), false),
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(count)) => (
                            Ok(Event::default().event("gap").data(count.to_string())), false
                        ),
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => (
                            Ok(Event::default().event("closed").data("Instrumentation source stopped.")), true
                        ),
                    }
                }
            };
            if finished {
                drop(permit.take());
            }
            Some((
                frame.map_err(std::io::Error::other),
                (history, receiver, permit, finished),
            ))
        },
    );
    use futures_util::StreamExt;
    let ready = futures_util::stream::once(async move {
        Ok::<_, std::io::Error>(Event::default().event("ready").data(format!(
            "Connected. Recent process history begins at sequence {}. Older events may be outside the 256-event buffer; new events stream live.",
            first_sequence.map_or_else(|| "none".to_owned(), |s|s.to_string())
        )))
    });
    use axum::response::IntoResponse;
    let mut response = Sse::new(ready.chain(stream))
        .keep_alive(KeepAlive::default())
        .into_response();
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response.headers_mut().insert(
        "x-accel-buffering",
        axum::http::HeaderValue::from_static("no"),
    );
    Ok(response)
}

fn workflow_origin(headers: &HeaderMap) -> Result<(), HttpError> {
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
    Ok(())
}

fn workflow_auth(state: &WorkflowHttpState, headers: &HeaderMap) -> Result<(), HttpError> {
    workflow_origin(headers)?;
    if state.auth.key.is_none() {
        return Err(error(
            StatusCode::SERVICE_UNAVAILABLE,
            "operator_auth_unconfigured",
            "Configure REASSURE_OPERATOR_KEY to inspect or resume durable workflows.",
            None,
        ));
    }
    if !state.auth.allows(headers) {
        return Err(error(
            StatusCode::UNAUTHORIZED,
            "operator_auth_required",
            "Operator authorization is required to inspect or resume workflows.",
            None,
        ));
    }
    Ok(())
}

fn workflow_error(value: crate::workflow::service::WorkflowError) -> HttpError {
    use crate::workflow::service::WorkflowError;
    let (status, code, message) = match value {
        WorkflowError::Invalid => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_workflow",
            "Use a bounded synthetic message, unique configured providers, valid request key and a resumable run.",
        ),
        WorkflowError::Unavailable => (
            StatusCode::SERVICE_UNAVAILABLE,
            "workflow_unavailable",
            "Configure the OpenAI planner and selected decision providers before submitting.",
        ),
        WorkflowError::Capacity => (
            StatusCode::TOO_MANY_REQUESTS,
            "workflow_capacity",
            "Workflow capacity is full. No automatic retry was scheduled.",
        ),
        WorkflowError::Storage => (
            StatusCode::SERVICE_UNAVAILABLE,
            "workflow_storage",
            "Durable storage failed, exhausted its limits, or rejected a request-key collision. Stop the API and inspect the protected database; do not blindly retry.",
        ),
        WorkflowError::Execution => (
            StatusCode::BAD_GATEWAY,
            "workflow_execution",
            "Workflow execution failed. Inspect durable records before retrying.",
        ),
    };
    tracing::warn!(event = "workflow_request_failed", code);
    error(status, code, message, None)
}

async fn workflow_options(
    State(state): State<WorkflowHttpState>,
) -> Json<crate::workflow::contracts::WorkflowOptions> {
    Json(state.service.options())
}

async fn workflow_setup(
    State(state): State<WorkflowHttpState>,
    headers: HeaderMap,
    body: Result<Json<crate::workflow::contracts::ConnectionSetup>, JsonRejection>,
) -> Result<Response, HttpError> {
    workflow_auth(&state, &headers)?;
    let Json(input) = body.map_err(|r| {
        error(
            r.status(),
            "invalid_setup_request",
            "Choose a router and enter its connection details.",
            None,
        )
    })?;
    use crate::workflow::service::ConnectionSetupError;
    let options = state.service.configure_connections(input).map_err(|failure| {
        let (status, code, message) = match failure {
            ConnectionSetupError::Invalid => (StatusCode::UNPROCESSABLE_ENTITY, "invalid_planner_setup",
                "Enter a valid API key for each selected router (no spaces, up to 512 characters). Already connected services cannot be replaced."),
            ConnectionSetupError::UnsupportedRoute => (StatusCode::UNPROCESSABLE_ENTITY, "unsupported_route",
                "That provider isn't available through the selected router."),
            ConnectionSetupError::InvalidEndpoint => (StatusCode::UNPROCESSABLE_ENTITY, "invalid_router_endpoint",
                "Enter your Azure Foundry resource endpoint, e.g. https://your-resource.openai.azure.com."),
            ConnectionSetupError::AlreadyConfigured => (StatusCode::CONFLICT, "planner_already_configured",
                "Setup is already configured. Refresh availability to continue. Restart the API to change credentials."),
            ConnectionSetupError::TransportUnavailable => (StatusCode::SERVICE_UNAVAILABLE, "setup_transport_unavailable",
                "The API could not initialize the AI connection. Your credentials were not installed. Try again or check the API's local configuration."),
        };
        tracing::warn!(event = "workflow_request_failed", code);
        error(status, code, message, None)
    })?;
    use axum::response::IntoResponse;
    let mut response = Json(options).into_response();
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    Ok(response)
}
async fn workflow_submit(
    State(state): State<WorkflowHttpState>,
    headers: HeaderMap,
    body: Result<Json<crate::workflow::contracts::WorkflowSubmission>, JsonRejection>,
) -> Result<Json<crate::workflow::contracts::WorkflowComparison>, HttpError> {
    workflow_origin(&headers)?;
    let Json(input) = body.map_err(|r| {
        error(
            r.status(),
            "invalid_request",
            "Expected a bounded workflow JSON request.",
            None,
        )
    })?;
    state
        .service
        .submit(input)
        .await
        .map(Json)
        .map_err(workflow_error)
}
async fn workflow_list(
    State(state): State<WorkflowHttpState>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::workflow::contracts::OperatorWorkflow>>, HttpError> {
    workflow_auth(&state, &headers)?;
    state
        .service
        .details()
        .await
        .map(Json)
        .map_err(workflow_error)
}
async fn workflow_resume(
    State(state): State<WorkflowHttpState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    body: Result<Json<crate::workflow::contracts::WorkflowResume>, JsonRejection>,
) -> Result<Json<crate::workflow::contracts::WorkflowComparison>, HttpError> {
    workflow_auth(&state, &headers)?;
    let Json(input) = body.map_err(|r| {
        error(
            r.status(),
            "invalid_request",
            "Expected a bounded workflow resume JSON request.",
            None,
        )
    })?;
    state
        .service
        .resume(id, input)
        .await
        .map(Json)
        .map_err(workflow_error)
}

#[derive(Clone)]
pub struct OperatorAuth {
    key: Option<String>,
}

impl OperatorAuth {
    pub fn new(key: Option<String>) -> Result<Self, std::io::Error> {
        if key.as_ref().is_some_and(|value| {
            value
                .chars()
                .filter(|c| !c.is_whitespace())
                .map(char::len_utf8)
                .sum::<usize>()
                < 32
        }) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "REASSURE_OPERATOR_KEY must contain at least 32 non-whitespace bytes",
            ));
        }
        Ok(Self { key })
    }

    fn allows(&self, headers: &HeaderMap) -> bool {
        let Some(expected) = self.key.as_deref() else {
            return false;
        };
        let Some(provided) = headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
        else {
            return false;
        };
        let expected = expected.as_bytes();
        let provided = provided.as_bytes();
        expected.len() == provided.len() && expected.ct_eq(provided).unwrap_u8() == 1
    }
}

#[derive(Clone)]
struct AppState {
    service: ServicingService,
    operator_auth: OperatorAuth,
}

pub fn router_with(service: ServicingService, operator_auth: OperatorAuth) -> Router {
    Router::new()
        .route("/health", get(|| async { Json(Health { status: "ok" }) }))
        .route("/v1/assessors", get(assessors))
        .route("/v1/messages", post(message))
        .route("/v1/employee/runs", get(employee_runs))
        .route("/v1/operator/runs", get(operator_runs))
        .layer(middleware::from_fn(instrument_request))
        .layer(DefaultBodyLimit::max(8192))
        .with_state(AppState {
            service,
            operator_auth,
        })
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

fn service_error(error_value: ServiceError) -> HttpError {
    match error_value {
        ServiceError::InvalidMessage(_) => error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_message",
            "Use a nonempty synthetic message of at most 4000 UTF-8 bytes.",
            None,
        ),
        ServiceError::CapacityExceeded => error(
            StatusCode::TOO_MANY_REQUESTS,
            "capacity_exceeded",
            "The assessment capacity is full. Try again later.",
            None,
        ),
        ServiceError::RunLimit => error(
            StatusCode::TOO_MANY_REQUESTS,
            "run_limit",
            "This local session has reached its run limit. Restart the sample to clear its ephemeral history.",
            None,
        ),
        ServiceError::Assessment { failure, run_id } => error(
            StatusCode::BAD_GATEWAY,
            failure.code(),
            "The assessment failed. No servicing action was executed. Inspect the operator view; there is no automatic fallback or retry.",
            Some(run_id),
        ),
        ServiceError::AssessorUnavailable(_) => error(
            StatusCode::SERVICE_UNAVAILABLE,
            "assessor_unavailable",
            "The selected assessor is not configured.",
            None,
        ),
    }
}

#[tracing::instrument(skip_all)]
async fn message(
    State(state): State<AppState>,
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
    state
        .service
        .submit_with(input.message, input.assessor)
        .await
        .map(Json)
        .map_err(service_error)
}

async fn assessors(State(state): State<AppState>) -> Json<Vec<AssessorOption>> {
    Json(state.service.assessor_options())
}

async fn employee_runs(State(state): State<AppState>) -> Json<Vec<OperatorRun>> {
    Json(state.service.runs().await)
}

async fn operator_runs(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<OperatorRunDetail>>, HttpError> {
    if state.operator_auth.key.is_none() {
        return Err(error(
            StatusCode::SERVICE_UNAVAILABLE,
            "operator_auth_unconfigured",
            "Operator inspection is locked until REASSURE_OPERATOR_KEY is configured.",
            None,
        ));
    }
    if !state.operator_auth.allows(&headers) {
        return Err(error(
            StatusCode::UNAUTHORIZED,
            "operator_auth_required",
            "Enter the local operator key to inspect provider exchanges.",
            None,
        ));
    }
    Ok(Json(state.service.run_details().await))
}
