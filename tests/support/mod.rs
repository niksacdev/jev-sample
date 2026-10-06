use jev_sample::{
    application::{ExecutionLimits, ServicingService},
    assessment::Assessor,
    baseline::KeywordBaseline,
    routing::RoutingPolicy,
};
use std::sync::Arc;

pub fn service(assessor: Arc<dyn Assessor>) -> ServicingService {
    let policy = RoutingPolicy::from_json(include_str!("../../config/routing.json"))
        .unwrap_or_else(|error| panic!("Invalid checked-in test policy: {error}"));
    let limits = ExecutionLimits::from_json(include_str!("../../config/execution.json"))
        .unwrap_or_else(|error| panic!("Invalid checked-in test limits: {error}"));
    ServicingService::new(assessor, policy, limits)
}

pub fn router() -> axum::Router {
    jev_sample::http::router_with(service(Arc::new(KeywordBaseline)))
}
