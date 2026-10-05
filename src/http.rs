//! HTTP transport. Health indicates process liveness, not provider readiness.

use axum::{Json, Router, routing::get};
use serde::Serialize;

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

/// Construct the API without binding a socket, for both serving and route tests.
pub fn router() -> Router {
    Router::new().route("/health", get(health))
}

async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}
