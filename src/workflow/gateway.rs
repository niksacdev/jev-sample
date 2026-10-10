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
}

impl Gateway {
    pub fn label(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::OpenRouter => "openrouter",
        }
    }

    /// OpenRouter may answer an alias such as `typesafe/jev-1.13` with a dated snapshot
    /// (`typesafe/jev-1.13-20260917`). Direct vendors must echo the configured model exactly.
    pub fn served_model_matches(self, configured: &str, served: &str) -> bool {
        served == configured
            || (self == Self::OpenRouter
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

pub fn valid_model_id(model: &str) -> bool {
    !model.is_empty()
        && model.len() <= 100
        && model
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_./:~".contains(&b))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
