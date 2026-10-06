//! Pure routing; model providers do not decide thresholds or task authority.

use serde::Deserialize;

use crate::domain::{Assessment, Evidence, Intent, Probability};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyConfig {
    version: String,
    intent_display_threshold: f64,
}

pub struct RoutingPolicy {
    version: String,
    threshold: Probability,
}

#[derive(Debug)]
pub enum PolicyError {
    InvalidJson(serde_json::Error),
    EmptyVersion,
    InvalidThreshold,
}

impl std::fmt::Display for PolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJson(e) => write!(f, "Invalid routing configuration: {e}"),
            Self::EmptyVersion => f.write_str("Routing policy needs a nonempty version"),
            Self::InvalidThreshold => f.write_str("Routing threshold must be finite and in [0,1]"),
        }
    }
}

impl std::error::Error for PolicyError {}

impl RoutingPolicy {
    pub fn from_json(json: &str) -> Result<Self, PolicyError> {
        let config: PolicyConfig = serde_json::from_str(json).map_err(PolicyError::InvalidJson)?;
        if config.version.trim().is_empty() {
            return Err(PolicyError::EmptyVersion);
        }
        let threshold = Probability::new(config.intent_display_threshold)
            .map_err(|_| PolicyError::InvalidThreshold)?;
        Ok(Self {
            version: config.version,
            threshold,
        })
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn plan(&self, assessment: &Assessment) -> Plan {
        let signals: Vec<_> = assessment
            .observations()
            .iter()
            .map(|observation| {
                let (probability, matched) = match observation.evidence {
                    Evidence::KeywordMatch(matched) => (None, matched),
                    Evidence::YesProbability(value) => {
                        (Some(value.value()), value.value() >= self.threshold.value())
                    }
                };
                RoutedIntent {
                    intent: observation.intent,
                    probability,
                    matched,
                }
            })
            .collect();
        let tasks = signals
            .iter()
            .filter(|signal| signal.matched)
            .map(|signal| signal.intent)
            .collect();
        Plan { signals, tasks }
    }
}

pub struct Plan {
    pub signals: Vec<RoutedIntent>,
    pub tasks: Vec<Intent>,
}

pub struct RoutedIntent {
    pub intent: Intent,
    pub probability: Option<f64>,
    pub matched: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Intent, Observation};

    #[test]
    fn invalid_policy_is_rejected_not_defaulted() {
        for json in [
            r#"{"version":"","intent_display_threshold":0.8}"#,
            r#"{"version":"v1","intent_display_threshold":1.1}"#,
            r#"{"version":"v1","intent_display_threshold":-0.1}"#,
            r#"{"version":"v1"}"#,
            r#"{"version":"v1","intent_display_threshold":0.8,"unknown":1}"#,
        ] {
            assert!(RoutingPolicy::from_json(json).is_err());
        }
    }

    #[test]
    fn configured_threshold_is_inclusive_and_all_tasks_require_review()
    -> Result<(), Box<dyn std::error::Error>> {
        let assessment = Assessment::new(
            Intent::ALL
                .into_iter()
                .zip([0.799, 0.8, 0.801, 1.0])
                .map(|(intent, value)| {
                    Probability::new(value).map(|p| Observation {
                        intent,
                        evidence: Evidence::YesProbability(p),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
            None,
        )?;
        let normal = RoutingPolicy::from_json(include_str!("../config/routing.json"))?;
        assert_eq!(normal.plan(&assessment).tasks.len(), 3);
        assert_eq!(
            normal.plan(&assessment).tasks,
            vec![
                Intent::PolicyChange,
                Intent::CustomerDetails,
                Intent::Billing
            ]
        );
        let stricter =
            RoutingPolicy::from_json(r#"{"version":"test-v2","intent_display_threshold":0.9}"#)?;
        assert_eq!(stricter.plan(&assessment).tasks.len(), 1);
        assert_eq!(stricter.version(), "test-v2");
        Ok(())
    }
}
