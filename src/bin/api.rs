use jev_sample::{
    application::{ExecutionLimits, ServicingService},
    assessment::Assessor,
    baseline::KeywordBaseline,
    contracts::AssessorId,
    http::OperatorAuth,
    routing::RoutingPolicy,
};
use opentelemetry::trace::TracerProvider as _;
use std::{collections::BTreeMap, env, io, path::Path, sync::Arc, time::Duration};
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> io::Result<()> {
    if Path::new(".env").exists() {
        dotenvy::from_path(".env").map_err(io::Error::other)?;
    }
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();
    let tracer = tracer_provider.tracer("reassure-api");
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_target(false)
                .with_current_span(true)
                .with_span_list(true)
                .with_writer(io::stderr),
        )
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .try_init()
        .map_err(io::Error::other)?;

    let mut assessors: BTreeMap<AssessorId, Arc<dyn Assessor>> = BTreeMap::new();
    assessors.insert(AssessorId::Code, Arc::new(KeywordBaseline));
    match env::var("TYPESAFE_API_KEY") {
        Ok(key) if !key.trim().is_empty() => {
            assessors.insert(
                AssessorId::Jev,
                Arc::new(
                    jev_sample::jev::Jev::new(
                        "https://api.typesafe.ai/v1/systemone".into(),
                        &key,
                        Duration::from_secs(15),
                    )
                    .map_err(io::Error::other)?,
                ),
            );
        }
        Ok(_) => {}
        Err(env::VarError::NotPresent) => {}
        Err(error) => return Err(io::Error::other(error)),
    }
    let operator_auth = OperatorAuth::new(match env::var("REASSURE_OPERATOR_KEY") {
        Ok(key) => Some(key),
        Err(env::VarError::NotPresent) => None,
        Err(error) => return Err(io::Error::other(error)),
    })?;
    eprintln!(
        "Synthetic local workspace; assessors={}; history is lost on restart.",
        assessors
            .keys()
            .map(|id| id.as_str())
            .collect::<Vec<_>>()
            .join(",")
    );
    let policy_json = match env::var("REASSURE_ROUTING_POLICY") {
        Err(env::VarError::NotPresent) => include_str!("../../config/routing.json").to_owned(),
        Ok(path) => std::fs::read_to_string(path)?,
        Err(error) => return Err(io::Error::other(error)),
    };
    let policy = RoutingPolicy::from_json(&policy_json).map_err(io::Error::other)?;
    let limits = ExecutionLimits::from_json(include_str!("../../config/execution.json"))
        .map_err(io::Error::other)?;
    let service = ServicingService::with_assessors(assessors, policy, limits);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    eprintln!("API listening on http://{}", listener.local_addr()?);
    let result = axum::serve(
        listener,
        jev_sample::http::router_with(service, operator_auth),
    )
    .await;
    tracer_provider.shutdown().map_err(io::Error::other)?;
    result
}
