//! The single route table for setup: which router serves which planner or decision choice.
//! The setup catalog and the adapter factories both derive from it, so the UI can never offer a
//! route the backend cannot build. Adding a router or provider is a table entry plus an adapter.

use std::{sync::Arc, time::Duration};

use super::{
    contracts::{
        ConnectionCatalog, DecisionChoice, DecisionChoiceOption, PlannerChoice,
        PlannerChoiceOption, ProviderId, RouterCredential, RouterId, RouterOption,
    },
    decision::{DecisionProvider, VendorAuth, VendorDecisions},
    gateway::{
        Gateway, OPENROUTER_DECISIONS, OPENROUTER_RESPONSES, RouteModels, azure_responses_url,
        is_loopback_test_endpoint,
    },
    llm_decision::LlmDecisions,
    planner::{OpenAiPlanner, Planner},
    responses::ResponsesClient,
};

pub const ROUTERS: [RouterId; 2] = [RouterId::Openrouter, RouterId::AzureFoundry];
pub const PLANNERS: [PlannerChoice; 1] = [PlannerChoice::Openai];
pub const DECISIONS: [DecisionChoice; 3] = [
    DecisionChoice::Jev,
    DecisionChoice::OpenaiDecisions,
    DecisionChoice::MicrosoftDecisions,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlannerRoute {
    OpenRouter,
    AzureFoundry,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DecisionRoute {
    OpenRouterJev,
}

fn planner_route(router: RouterId, choice: PlannerChoice) -> Option<PlannerRoute> {
    match (router, choice) {
        (RouterId::Openrouter, PlannerChoice::Openai) => Some(PlannerRoute::OpenRouter),
        (RouterId::AzureFoundry, PlannerChoice::Openai) => Some(PlannerRoute::AzureFoundry),
    }
}

/// Jev is not in the Azure Foundry catalog; OpenAI Decisions and Microsoft Decisions have no
/// adapter yet, so they are listed but not routable.
fn decision_route(router: RouterId, choice: DecisionChoice) -> Option<DecisionRoute> {
    match (router, choice) {
        (RouterId::Openrouter, DecisionChoice::Jev) => Some(DecisionRoute::OpenRouterJev),
        _ => None,
    }
}

pub fn supports_planner(router: RouterId, choice: PlannerChoice) -> bool {
    planner_route(router, choice).is_some()
}

pub fn supports_decision(router: RouterId, choice: DecisionChoice) -> bool {
    decision_route(router, choice).is_some()
}

/// The persisted workflow slot a decision choice fills.
pub fn decision_provider(choice: DecisionChoice) -> Option<ProviderId> {
    match choice {
        DecisionChoice::Jev => Some(ProviderId::Jev),
        DecisionChoice::OpenaiDecisions => Some(ProviderId::Openai),
        DecisionChoice::MicrosoftDecisions => None,
    }
}

pub fn router_label(router: RouterId) -> &'static str {
    match router {
        RouterId::Openrouter => "OpenRouter",
        RouterId::AzureFoundry => "Azure Foundry",
    }
}

pub fn catalog(models: &RouteModels) -> ConnectionCatalog {
    let routers_for = |supports: &dyn Fn(RouterId) -> bool| -> Vec<RouterId> {
        ROUTERS.into_iter().filter(|r| supports(*r)).collect()
    };
    ConnectionCatalog {
        routers: ROUTERS
            .into_iter()
            .map(|id| match id {
                RouterId::Openrouter => RouterOption {
                    id,
                    label: router_label(id).into(),
                    needs_endpoint: false,
                    endpoint_hint: None,
                    help: "One OpenRouter key; usage is billed to your OpenRouter credits.".into(),
                },
                RouterId::AzureFoundry => RouterOption {
                    id,
                    label: router_label(id).into(),
                    needs_endpoint: true,
                    endpoint_hint: Some("https://your-resource.openai.azure.com".into()),
                    help: format!(
                        "Your Azure Foundry resource key and endpoint, with a deployment named {}.",
                        models.azure_foundry.planner_deployment
                    ),
                },
            })
            .collect(),
        planners: PLANNERS
            .into_iter()
            .map(|id| PlannerChoiceOption {
                id,
                label: "OpenAI".into(),
                routers: routers_for(&|r| supports_planner(r, id)),
                note: None,
            })
            .collect(),
        decisions: DECISIONS
            .into_iter()
            .map(|id| {
                let routers = routers_for(&|r| supports_decision(r, id));
                DecisionChoiceOption {
                    id,
                    label: match id {
                        DecisionChoice::Jev => "Jev",
                        DecisionChoice::OpenaiDecisions => "OpenAI Decisions API",
                        DecisionChoice::MicrosoftDecisions => "Microsoft Decisions",
                    }
                    .into(),
                    provider: decision_provider(id),
                    note: match id {
                        DecisionChoice::Jev => Some(
                            "Available through OpenRouter; not in the Azure Foundry catalog."
                                .into(),
                        ),
                        _ if routers.is_empty() => Some("Coming soon".into()),
                        _ => None,
                    },
                    routers,
                }
            })
            .collect(),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RouteError {
    Unsupported,
    InvalidCredential,
    InvalidEndpoint,
    Client,
}

fn client_error(code: String) -> RouteError {
    if code == "client" {
        RouteError::Client
    } else {
        RouteError::InvalidCredential
    }
}

/// Fixed router URLs. Tests may swap in a loopback base; runtime setup never chooses a URL
/// except the validated Azure resource host.
#[derive(Clone, Debug)]
pub struct RouterEndpoints {
    openrouter_responses: String,
    openrouter_decisions: String,
    azure_base: Option<String>,
}

impl RouterEndpoints {
    pub fn production() -> Self {
        Self {
            openrouter_responses: OPENROUTER_RESPONSES.into(),
            openrouter_decisions: OPENROUTER_DECISIONS.into(),
            azure_base: None,
        }
    }

    /// Routes every router to a local mock; Azure endpoints are still validated before use.
    pub fn loopback(base: &str) -> Result<Self, String> {
        let base = base.trim_end_matches('/');
        if !is_loopback_test_endpoint(base) {
            return Err("test_endpoint_must_be_loopback".into());
        }
        Ok(Self {
            openrouter_responses: format!("{base}/api/v1/responses"),
            openrouter_decisions: format!("{base}/api/alpha/decisions"),
            azure_base: Some(base.into()),
        })
    }
}

/// A validated credential for one router.
pub struct RouterConnection {
    router: RouterId,
    key: String,
    azure_url: Option<String>,
}

impl RouterConnection {
    pub fn new(
        credential: &RouterCredential,
        endpoints: &RouterEndpoints,
    ) -> Result<Self, RouteError> {
        let key = credential.api_key.as_str();
        if key.is_empty() || key.len() > 512 || !key.bytes().all(|b| b.is_ascii_graphic()) {
            return Err(RouteError::InvalidCredential);
        }
        let azure_url = match credential.router {
            RouterId::Openrouter => {
                if credential.endpoint.is_some() {
                    return Err(RouteError::InvalidEndpoint);
                }
                None
            }
            RouterId::AzureFoundry => {
                let url = credential
                    .endpoint
                    .as_deref()
                    .and_then(azure_responses_url)
                    .ok_or(RouteError::InvalidEndpoint)?;
                Some(match &endpoints.azure_base {
                    Some(base) => format!("{base}/openai/v1/responses"),
                    None => url,
                })
            }
        };
        Ok(Self {
            router: credential.router,
            key: key.into(),
            azure_url,
        })
    }

    pub fn router(&self) -> RouterId {
        self.router
    }

    pub fn planner(
        &self,
        choice: PlannerChoice,
        models: &RouteModels,
        endpoints: &RouterEndpoints,
        timeout: Duration,
    ) -> Result<Arc<dyn Planner>, RouteError> {
        let client = match planner_route(self.router, choice).ok_or(RouteError::Unsupported)? {
            PlannerRoute::OpenRouter => ResponsesClient::new(
                endpoints.openrouter_responses.clone(),
                &self.key,
                &models.openrouter.planner_model,
                Gateway::OpenRouter,
                timeout,
            ),
            PlannerRoute::AzureFoundry => {
                let azure = &models.azure_foundry;
                ResponsesClient::routed(
                    self.azure_url.clone().ok_or(RouteError::InvalidEndpoint)?,
                    &self.key,
                    VendorAuth::ApiKeyHeader,
                    &azure.planner_deployment,
                    &azure.planner_model,
                    Gateway::AzureFoundry,
                    azure.planner_price.clone(),
                    timeout,
                )
            }
        }
        .map_err(client_error)?;
        Ok(Arc::new(OpenAiPlanner::from_client(client)))
    }

    pub fn decision(
        &self,
        choice: DecisionChoice,
        models: &RouteModels,
        endpoints: &RouterEndpoints,
        timeout: Duration,
    ) -> Result<Arc<dyn DecisionProvider>, RouteError> {
        match decision_route(self.router, choice).ok_or(RouteError::Unsupported)? {
            DecisionRoute::OpenRouterJev => Ok(Arc::new(
                VendorDecisions::new(
                    endpoints.openrouter_decisions.clone(),
                    &self.key,
                    &models.openrouter.jev_model,
                    ProviderId::Jev,
                    Gateway::OpenRouter,
                    timeout,
                )
                .map_err(client_error)?,
            )),
        }
    }

    /// The general-LLM comparison baseline answering the same questions; OpenRouter only.
    pub fn llm_baseline(
        &self,
        models: &RouteModels,
        endpoints: &RouterEndpoints,
        timeout: Duration,
    ) -> Option<Result<Arc<dyn DecisionProvider>, RouteError>> {
        (self.router == RouterId::Openrouter).then(|| {
            ResponsesClient::new(
                endpoints.openrouter_responses.clone(),
                &self.key,
                &models.openrouter.decision_model,
                Gateway::OpenRouter,
                timeout,
            )
            .map(|client| Arc::new(LlmDecisions::from_client(client)) as Arc<dyn DecisionProvider>)
            .map_err(client_error)
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn credential(router: RouterId, endpoint: Option<&str>) -> RouterCredential {
        RouterCredential {
            router,
            api_key: "test-key".into(),
            endpoint: endpoint.map(Into::into),
        }
    }

    #[test]
    fn every_advertised_route_constructs_and_nothing_else_does() {
        let models = RouteModels::checked_in();
        let endpoints = RouterEndpoints::production();
        let catalog = catalog(&models);
        let timeout = Duration::from_secs(1);
        for router in ROUTERS {
            let connection = RouterConnection::new(
                &credential(
                    router,
                    (router == RouterId::AzureFoundry)
                        .then_some("https://contoso.openai.azure.com"),
                ),
                &endpoints,
            )
            .unwrap();
            for option in &catalog.planners {
                let built = connection.planner(option.id, &models, &endpoints, timeout);
                assert_eq!(
                    built.is_ok(),
                    option.routers.contains(&router),
                    "{router:?}"
                );
            }
            for option in &catalog.decisions {
                let built = connection.decision(option.id, &models, &endpoints, timeout);
                assert_eq!(
                    built.is_ok(),
                    option.routers.contains(&router),
                    "{router:?}"
                );
                if !option.routers.contains(&router) {
                    assert_eq!(built.err(), Some(RouteError::Unsupported));
                }
            }
        }
    }

    #[test]
    fn catalog_lists_only_jev_as_routable_and_only_via_openrouter() {
        let catalog = catalog(&RouteModels::checked_in());
        assert_eq!(catalog.routers.len(), 2);
        assert_eq!(catalog.planners[0].routers, ROUTERS.to_vec());
        let routable: Vec<_> = catalog
            .decisions
            .iter()
            .filter(|d| !d.routers.is_empty())
            .map(|d| (d.id, d.routers.clone()))
            .collect();
        assert_eq!(
            routable,
            vec![(DecisionChoice::Jev, vec![RouterId::Openrouter])]
        );
        assert!(
            catalog.decisions[1..]
                .iter()
                .all(|d| d.note.as_deref() == Some("Coming soon"))
        );
    }

    #[test]
    fn credentials_are_validated_per_router() {
        let endpoints = RouterEndpoints::production();
        let new = |c: RouterCredential| RouterConnection::new(&c, &endpoints).err();
        assert_eq!(
            new(credential(RouterId::AzureFoundry, None)),
            Some(RouteError::InvalidEndpoint)
        );
        assert_eq!(
            new(credential(
                RouterId::AzureFoundry,
                Some("https://evil.example/openai")
            )),
            Some(RouteError::InvalidEndpoint)
        );
        assert_eq!(
            new(credential(
                RouterId::Openrouter,
                Some("https://contoso.openai.azure.com")
            )),
            Some(RouteError::InvalidEndpoint)
        );
        let mut blank = credential(RouterId::Openrouter, None);
        blank.api_key = "has space".into();
        assert_eq!(new(blank), Some(RouteError::InvalidCredential));
        assert!(RouterEndpoints::loopback("https://example.com").is_err());
    }
}
