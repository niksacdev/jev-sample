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
    let tracer = tracer_provider.tracer("zipclaim-api");
    let live_telemetry = jev_sample::telemetry::LiveTelemetry::default();
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
        .with(live_telemetry.clone())
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
        "Synthetic local workspace; assessors={}; legacy assessment history is lost on restart.",
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
    use jev_sample::workflow::{
        connections::{RouterConnection, RouterEndpoints},
        contracts::{DecisionChoice, PlannerChoice, ProviderId, RouterCredential, RouterId},
        decision::{DecisionProvider, DeterministicProvider, VendorDecisions},
        gateway::{RouteModels, valid_model_id},
        planner::{OpenAiPlanner, Planner},
        policy::WorkflowPolicy,
        service::WorkflowService,
        store::WorkflowStore,
    };
    let workflow_policy = WorkflowPolicy::from_json(include_str!("../../config/workflow.json"))
        .map_err(io::Error::other)?;
    let provider_timeout = Duration::from_millis(u64::from(workflow_policy.deadline_ms));
    let mut decisions: BTreeMap<ProviderId, Arc<dyn DecisionProvider>> = BTreeMap::new();
    decisions.insert(ProviderId::Code, Arc::new(DeterministicProvider));
    if let Some(key) = optional_env("TYPESAFE_API_KEY")? {
        decisions.insert(
            ProviderId::Jev,
            Arc::new(VendorDecisions::jev(&key, provider_timeout).map_err(io::Error::other)?),
        );
    }
    let openai_key = optional_env("OPENAI_API_KEY")?;
    let planner_model = optional_env("OPENAI_PLANNER_MODEL")?;
    let decision_model = optional_env("OPENAI_DECISION_MODEL")?;
    if openai_key.is_none() && (planner_model.is_some() || decision_model.is_some()) {
        return Err(io::Error::other("OpenAI models require OPENAI_API_KEY"));
    }
    let planner: Option<Arc<dyn Planner>> = match (&openai_key, &planner_model) {
        (Some(key), Some(model)) => Some(Arc::new(
            OpenAiPlanner::new(key, model, provider_timeout).map_err(io::Error::other)?,
        )),
        _ => None,
    };
    if let (Some(key), Some(model)) = (&openai_key, &decision_model) {
        decisions.insert(
            ProviderId::Openai,
            Arc::new(
                VendorDecisions::openai(key, model, provider_timeout).map_err(io::Error::other)?,
            ),
        );
    }
    // Direct vendor keys win; Azure Foundry then fills the planner, and one OpenRouter key fills
    // whatever is still missing. Startup uses the same route table as runtime setup.
    let mut models = RouteModels::checked_in();
    for (name, slot) in [
        (
            "OPENROUTER_PLANNER_MODEL",
            &mut models.openrouter.planner_model,
        ),
        (
            "OPENROUTER_DECISION_MODEL",
            &mut models.openrouter.decision_model,
        ),
        ("OPENROUTER_JEV_MODEL", &mut models.openrouter.jev_model),
        (
            "AZURE_FOUNDRY_PLANNER_DEPLOYMENT",
            &mut models.azure_foundry.planner_deployment,
        ),
        (
            "AZURE_FOUNDRY_PLANNER_MODEL",
            &mut models.azure_foundry.planner_model,
        ),
    ] {
        if let Some(model) = optional_env(name)? {
            if !valid_model_id(&model) {
                return Err(io::Error::other(format!("{name} is not a valid model ID")));
            }
            *slot = model;
        }
    }
    let endpoints = RouterEndpoints::production();
    let route_error = |router: &str| {
        let router = router.to_string();
        move |e| io::Error::other(format!("{router} startup connection failed: {e:?}"))
    };
    let mut planner = planner;
    match (
        optional_env("AZURE_FOUNDRY_ENDPOINT")?,
        optional_env("AZURE_FOUNDRY_API_KEY")?,
    ) {
        (Some(endpoint), Some(api_key)) => {
            let azure = RouterConnection::new(
                &RouterCredential {
                    router: RouterId::AzureFoundry,
                    api_key,
                    endpoint: Some(endpoint),
                },
                &endpoints,
            )
            .map_err(route_error("AZURE_FOUNDRY"))?;
            if planner.is_none() {
                planner = Some(
                    azure
                        .planner(PlannerChoice::Openai, &models, &endpoints, provider_timeout)
                        .map_err(route_error("AZURE_FOUNDRY"))?,
                );
            }
        }
        (None, None) => {}
        _ => {
            return Err(io::Error::other(
                "Azure Foundry requires both AZURE_FOUNDRY_ENDPOINT and AZURE_FOUNDRY_API_KEY",
            ));
        }
    }
    if let Some(api_key) = optional_env("OPENROUTER_API_KEY")? {
        let openrouter = RouterConnection::new(
            &RouterCredential {
                router: RouterId::Openrouter,
                api_key,
                endpoint: None,
            },
            &endpoints,
        )
        .map_err(route_error("OPENROUTER"))?;
        if planner.is_none() {
            planner = Some(
                openrouter
                    .planner(PlannerChoice::Openai, &models, &endpoints, provider_timeout)
                    .map_err(route_error("OPENROUTER"))?,
            );
        }
        if let std::collections::btree_map::Entry::Vacant(slot) = decisions.entry(ProviderId::Jev) {
            slot.insert(
                openrouter
                    .decision(DecisionChoice::Jev, &models, &endpoints, provider_timeout)
                    .map_err(route_error("OPENROUTER"))?,
            );
        }
        if let std::collections::btree_map::Entry::Vacant(slot) =
            decisions.entry(ProviderId::Openai)
            && let Some(baseline) = openrouter.llm_baseline(&models, &endpoints, provider_timeout)
        {
            slot.insert(baseline.map_err(route_error("OPENROUTER"))?);
        }
    }
    let database =
        optional_env("REASSURE_WORKFLOW_DB")?.unwrap_or_else(|| ".local/workflows.sqlite3".into());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    let store = WorkflowStore::open(Path::new(&database), &workflow_policy)
        .await
        .map_err(io::Error::other)?;
    let workflows =
        WorkflowService::new(planner, decisions, workflow_policy, store).with_route_models(models);
    eprintln!(
        "Synthetic workflow planner configured={}; durable SQLite records enabled; no automatic replay.",
        workflows.options().planner.available
    );
    let app = jev_sample::http::router_with(service, operator_auth.clone())
        .merge(jev_sample::http::workflow_router(
            workflows,
            operator_auth.clone(),
        ))
        .merge(jev_sample::http::telemetry_router(
            live_telemetry,
            operator_auth,
        ));
    eprintln!("API listening on http://{}", listener.local_addr()?);
    let result = axum::serve(listener, app).await;
    tracer_provider.shutdown().map_err(io::Error::other)?;
    result
}

fn optional_env(name: &str) -> io::Result<Option<String>> {
    match env::var(name) {
        Ok(value) if !value.trim().is_empty() => Ok(Some(value)),
        Ok(_) | Err(env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(io::Error::other(error)),
    }
}
