use jev_sample::{
    application::{ExecutionLimits, ServicingService},
    assessment::Assessor,
    baseline::KeywordBaseline,
    routing::RoutingPolicy,
};
use std::{env, io, sync::Arc, time::Duration};

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_target(false)
        .with_writer(io::stderr)
        .try_init()
        .map_err(io::Error::other)?;
    let assessor: Arc<dyn Assessor> = match env::var("REASSURE_ASSESSOR") {
        Err(env::VarError::NotPresent) => Arc::new(KeywordBaseline),
        Ok(value) if value == "rules" => Arc::new(KeywordBaseline),
        Ok(value) if value == "jev" => {
            let key = env::var("TYPESAFE_API_KEY").map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Jev mode requires TYPESAFE_API_KEY",
                )
            })?;
            Arc::new(
                jev_sample::jev::Jev::new(
                    "https://api.typesafe.ai/v1/systemone".into(),
                    &key,
                    Duration::from_secs(15),
                )
                .map_err(io::Error::other)?,
            )
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "REASSURE_ASSESSOR must be rules or jev",
            ));
        }
    };
    eprintln!(
        "Synthetic local workspace; assessor={}; history is lost on restart.",
        assessor.provenance().assessor
    );
    let policy_json = match env::var("REASSURE_ROUTING_POLICY") {
        Err(env::VarError::NotPresent) => include_str!("../../config/routing.json").to_owned(),
        Ok(path) => std::fs::read_to_string(path)?,
        Err(error) => return Err(io::Error::other(error)),
    };
    let policy = RoutingPolicy::from_json(&policy_json).map_err(io::Error::other)?;
    let limits = ExecutionLimits::from_json(include_str!("../../config/execution.json"))
        .map_err(io::Error::other)?;
    let service = ServicingService::new(assessor, policy, limits);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    eprintln!("API listening on http://{}", listener.local_addr()?);
    axum::serve(listener, jev_sample::http::router_with(service)).await
}
