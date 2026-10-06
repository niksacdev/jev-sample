//! HTTP transport for the loopback-only servicing workspace.

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use serde::Serialize;

use crate::{
    application::{ServiceError, ServicingService},
    contracts::{ApiError, CustomerMessage, CustomerReply, OperatorRun},
};

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

pub fn router_with(service: ServicingService) -> Router {
    Router::new()
        .route("/health", get(|| async { Json(Health { status: "ok" }) }))
        .route("/v1/messages", post(message))
        .route("/v1/operator/runs", get(runs))
        .layer(DefaultBodyLimit::max(8192))
        .with_state(service)
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
    }
}

async fn message(
    State(service): State<ServicingService>,
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
    service
        .submit(input.message)
        .await
        .map(Json)
        .map_err(service_error)
}

async fn runs(State(service): State<ServicingService>) -> Json<Vec<OperatorRun>> {
    Json(service.runs().await)
}
