//! Fixed vendor routes. Runtime composition and setup can choose a route but never a URL.

use serde::Deserialize;

pub const OPENAI_RESPONSES: &str = "https://api.openai.com/v1/responses";
pub const OPENAI_DECISIONS: &str = "https://api.openai.com/v1/decisions";
pub const TYPESAFE_SYSTEMONE: &str = "https://api.typesafe.ai/v1/systemone";
pub const OPENROUTER_RESPONSES: &str = "https://openrouter.ai/api/v1/responses";
pub const OPENROUTER_DECISIONS: &str = "https://openrouter.ai/api/alpha/decisions";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gateway {
    Direct,
    OpenRouter,
    AzureFoundry,
}

impl Gateway {
    pub fn label(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::OpenRouter => "openrouter",
            Self::AzureFoundry => "azure_foundry",
        }
    }

    /// Recorded model identity. Routed adapters are qualified so a resumed workflow cannot
    /// silently continue through a different router; direct identities keep their stored form.
    pub fn identity(self, model: &str) -> String {
        match self {
            Self::Direct => model.into(),
            routed => format!("{}:{model}", routed.label()),
        }
    }

    /// Routers may answer an alias such as `typesafe/jev-1.13` with a dated snapshot
    /// (`typesafe/jev-1.13-20260917`). Direct vendors must echo the configured model exactly.
    pub fn served_model_matches(self, configured: &str, served: &str) -> bool {
        served == configured
            || (self != Self::Direct
                && served
                    .strip_prefix(configured)
                    .and_then(|rest| rest.strip_prefix('-'))
                    .is_some_and(is_snapshot_date))
    }
}

/// `YYYYMMDD` or `YYYY-MM-DD` with a plausible month and day.
fn is_snapshot_date(suffix: &str) -> bool {
    let digits: String = match suffix.len() {
        8 => suffix.to_owned(),
        10 if suffix.as_bytes()[4] == b'-' && suffix.as_bytes()[7] == b'-' => {
            suffix.replace('-', "")
        }
        _ => return false,
    };
    if digits.len() != 8 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    let month: u8 = digits[4..6].parse().unwrap_or(0);
    let day: u8 = digits[6..8].parse().unwrap_or(0);
    (1..=12).contains(&month) && (1..=31).contains(&day)
}

/// Test-only adapters may target an unauthenticated plain-HTTP mock on 127.0.0.1 only.
pub(crate) fn is_loopback_test_endpoint(endpoint: &str) -> bool {
    reqwest::Url::parse(endpoint).is_ok_and(|url| {
        url.scheme() == "http"
            && url.username().is_empty()
            && url.password().is_none()
            && url.host_str() == Some("127.0.0.1")
            && url.port().is_some()
    })
}

/// Checked-in OpenRouter model choices; environment variables may override planner and decision models.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenRouterModels {
    pub planner_model: String,
    pub decision_model: String,
    pub jev_model: String,
}

impl OpenRouterModels {
    pub fn from_json(json: &str) -> Result<Self, String> {
        let models: Self =
            serde_json::from_str(json).map_err(|_| "invalid_openrouter_models".to_string())?;
        if [
            &models.planner_model,
            &models.decision_model,
            &models.jev_model,
        ]
        .iter()
        .all(|model| valid_model_id(model))
        {
            Ok(models)
        } else {
            Err("invalid_openrouter_models".into())
        }
    }

    pub fn checked_in() -> Self {
        #[allow(clippy::expect_used)]
        Self::from_json(include_str!("../../config/openrouter.json"))
            .expect("config/openrouter.json is validated by tests")
    }
}

/// Checked-in Azure Foundry deployment. The deployment name is sent; the model is what the
/// response must report, because deployment names are chosen freely by the Azure owner.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AzureFoundryModels {
    pub planner_deployment: String,
    pub planner_model: String,
    pub planner_price: Option<TokenPrice>,
}

/// List price used to estimate spend when a router reports tokens but not cost. Cached-input
/// discounts are ignored, so estimates err high.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TokenPrice {
    pub input_usd_per_million: f64,
    pub output_usd_per_million: f64,
    pub source: String,
}

impl TokenPrice {
    fn valid(&self) -> bool {
        [self.input_usd_per_million, self.output_usd_per_million]
            .iter()
            .all(|p| p.is_finite() && *p >= 0.0 && *p <= 10_000.0)
            && !self.source.trim().is_empty()
    }

    pub fn estimate(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        (f64::from(input_tokens) * self.input_usd_per_million
            + f64::from(output_tokens) * self.output_usd_per_million)
            / 1_000_000.0
    }
}

impl AzureFoundryModels {
    pub fn from_json(json: &str) -> Result<Self, String> {
        let models: Self =
            serde_json::from_str(json).map_err(|_| "invalid_azure_foundry_models".to_string())?;
        if valid_model_id(&models.planner_deployment)
            && valid_model_id(&models.planner_model)
            && models.planner_price.as_ref().is_none_or(TokenPrice::valid)
        {
            Ok(models)
        } else {
            Err("invalid_azure_foundry_models".into())
        }
    }

    pub fn checked_in() -> Self {
        #[allow(clippy::expect_used)]
        Self::from_json(include_str!("../../config/azure-foundry.json"))
            .expect("config/azure-foundry.json is validated by tests")
    }
}

/// Model choices for every router.
#[derive(Clone, Debug)]
pub struct RouteModels {
    pub openrouter: OpenRouterModels,
    pub azure_foundry: AzureFoundryModels,
}

impl RouteModels {
    pub fn checked_in() -> Self {
        Self {
            openrouter: OpenRouterModels::checked_in(),
            azure_foundry: AzureFoundryModels::checked_in(),
        }
    }
}

/// Azure resource hosts documented to serve the v1 Responses API at `/openai/v1/responses`.
const AZURE_HOST_SUFFIXES: [&str; 2] = [".openai.azure.com", ".services.ai.azure.com"];

/// Builds the Responses URL from a user-supplied Azure endpoint. Only HTTPS Azure resource hosts
/// are accepted, so a setup value cannot redirect the key elsewhere. The path is ignored, which
/// lets operators paste either the resource or the project endpoint.
pub fn azure_responses_url(endpoint: &str) -> Option<String> {
    let url = reqwest::Url::parse(endpoint.trim()).ok()?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return None;
    }
    let host = url.host_str()?.to_ascii_lowercase();
    let resource = AZURE_HOST_SUFFIXES
        .iter()
        .find_map(|suffix| host.strip_suffix(suffix))?;
    let valid = (2..=64).contains(&resource.len())
        && resource
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !resource.starts_with('-')
        && !resource.ends_with('-');
    valid.then(|| format!("https://{host}/openai/v1/responses"))
}

pub fn valid_model_id(model: &str) -> bool {
    !model.is_empty()
        && model.len() <= 100
        && model
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_./:~".contains(&b))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn only_openrouter_accepts_dated_snapshots_of_the_configured_model() {
        let or = Gateway::OpenRouter;
        assert!(or.served_model_matches("typesafe/jev-1.13", "typesafe/jev-1.13"));
        assert!(or.served_model_matches("typesafe/jev-1.13", "typesafe/jev-1.13-20260917"));
        assert!(!or.served_model_matches("typesafe/jev-1.13", "typesafe/jev-1.13-preview"));
        assert!(or.served_model_matches("openai/gpt-6-astra", "openai/gpt-6-astra-2026-01-01"));
        assert!(!or.served_model_matches("openai/gpt-6-astra", "openai/gpt-6-astra-1"));
        assert!(!or.served_model_matches("openai/gpt-6-astra", "openai/gpt-6-astra-20261301"));
        assert!(!or.served_model_matches("typesafe/jev-1.13", "typesafe/jev-1.130"));
        assert!(!or.served_model_matches("typesafe/jev-1.13", "openai/gpt-6-astra"));
    }

    #[test]
    fn loopback_guard_parses_the_host() {
        assert!(is_loopback_test_endpoint(
            "http://127.0.0.1:8080/v1/responses"
        ));
        assert!(!is_loopback_test_endpoint(
            "http://127.0.0.1:8080@attacker.example/"
        ));
        assert!(!is_loopback_test_endpoint(
            "http://127.0.0.1.attacker.example:80/"
        ));
        assert!(!is_loopback_test_endpoint("https://127.0.0.1:8080/"));
        assert!(!is_loopback_test_endpoint("http://user@127.0.0.1:8080/"));
        assert!(!Gateway::Direct.served_model_matches("jev-1.13.0", "jev-1.13.0-20260917"));
    }

    #[test]
    fn azure_endpoints_are_limited_to_https_resource_hosts() {
        assert_eq!(
            azure_responses_url("https://contoso-ai.openai.azure.com/").as_deref(),
            Some("https://contoso-ai.openai.azure.com/openai/v1/responses")
        );
        assert_eq!(
            azure_responses_url("https://Contoso.services.ai.azure.com/api/projects/claims")
                .as_deref(),
            Some("https://contoso.services.ai.azure.com/openai/v1/responses")
        );
        for rejected in [
            "http://contoso.openai.azure.com",
            "https://contoso.openai.azure.com:8443",
            "https://user@contoso.openai.azure.com",
            "https://contoso.openai.azure.com.attacker.example",
            "https://attacker.example/contoso.openai.azure.com",
            "https://-bad.openai.azure.com",
            "https://openai.azure.com",
            "https://contoso.openai.azure.com/?x=1",
            "not a url",
        ] {
            assert!(azure_responses_url(rejected).is_none(), "{rejected}");
        }
    }

    #[test]
    fn routed_identities_are_qualified_but_direct_ones_are_unchanged() {
        assert_eq!(Gateway::Direct.identity("gpt-6-astra"), "gpt-6-astra");
        assert_eq!(
            Gateway::AzureFoundry.identity("gpt-6-astra"),
            "azure_foundry:gpt-6-astra"
        );
        assert_eq!(
            Gateway::OpenRouter.identity("typesafe/jev-1.13"),
            "openrouter:typesafe/jev-1.13"
        );
        assert!(
            Gateway::AzureFoundry.served_model_matches("gpt-6-astra", "gpt-6-astra-2026-03-01")
        );
        let azure = AzureFoundryModels::checked_in();
        assert!(valid_model_id(&azure.planner_deployment));
        let price = azure.planner_price.expect("checked-in Azure price");
        assert!((price.estimate(1_000_000, 100_000) - 15.0).abs() < 1e-9);
        assert!(
            AzureFoundryModels::from_json(
                r#"{"planner_deployment":"a","planner_model":"b","planner_price":{"input_usd_per_million":-1,"output_usd_per_million":1,"source":"x"}}"#
            )
            .is_err()
        );
        assert!(
            AzureFoundryModels::from_json(
                r#"{"planner_deployment":"a","planner_model":"b","endpoint":"x"}"#
            )
            .is_err()
        );
    }

    #[test]
    fn checked_in_models_are_valid_and_unknown_fields_fail() {
        let models = OpenRouterModels::checked_in();
        assert!(models.jev_model.starts_with("typesafe/"));
        assert!(
            OpenRouterModels::from_json(
                r#"{"planner_model":"a","decision_model":"b","jev_model":"c","url":"x"}"#
            )
            .is_err()
        );
        assert!(
            OpenRouterModels::from_json(
                r#"{"planner_model":"bad model","decision_model":"b","jev_model":"c"}"#
            )
            .is_err()
        );
    }
}
