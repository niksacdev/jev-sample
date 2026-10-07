//! HTTP transport for the loopback-only servicing workspace.

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
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
