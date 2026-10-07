use jev_sample::{
    application::{ExecutionLimits, ServicingService},
    assessment::Assessor,
    baseline::KeywordBaseline,
    routing::RoutingPolicy,
};
use std::sync::Arc;

pub const TEST_OPERATOR_KEY: &str = "test-operator-key-1234567890123456";

pub fn service(assessor: Arc<dyn Assessor>) -> ServicingService {
    let policy = RoutingPolicy::from_json(include_str!("../../config/routing.json"))
        .unwrap_or_else(|error| panic!("Invalid checked-in test policy: {error}"));
    let limits = ExecutionLimits::from_json(include_str!("../../config/execution.json"))
        .unwrap_or_else(|error| panic!("Invalid checked-in test limits: {error}"));
    ServicingService::new(assessor, policy, limits)
}

pub fn router() -> axum::Router {
    router_for(service(Arc::new(KeywordBaseline)))
}

pub fn router_for(service: ServicingService) -> axum::Router {
    let auth = jev_sample::http::OperatorAuth::new(Some(TEST_OPERATOR_KEY.into()))
        .unwrap_or_else(|error| panic!("Invalid test operator key: {error}"));
    jev_sample::http::router_with(service, auth)
}
