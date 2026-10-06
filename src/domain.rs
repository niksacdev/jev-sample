//! Validated application values. No runtime, transport or provider calls.

use std::fmt;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const MAX_MESSAGE_BYTES: usize = 4000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    Claim,
    PolicyChange,
    CustomerDetails,
    Billing,
}

impl Intent {
    pub const ALL: [Self; 4] = [
        Self::Claim,
        Self::PolicyChange,
        Self::CustomerDetails,
        Self::Billing,
    ];
}

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidMessage;

impl fmt::Display for InvalidMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Message must be nonempty and at most {MAX_MESSAGE_BYTES} UTF-8 bytes"
        )
    }
}

impl std::error::Error for InvalidMessage {}

pub struct Message(String);

impl Message {
    pub fn new(value: String) -> Result<Self, InvalidMessage> {
        if value.trim().is_empty() || value.len() > MAX_MESSAGE_BYTES {
            return Err(InvalidMessage);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Probability(f64);

impl Probability {
    pub fn new(value: f64) -> Result<Self, InvalidAssessment> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(InvalidAssessment);
        }
        Ok(Self(value))
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

/// A keyword match is not a confidence score. Providers must preserve that distinction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Evidence {
    KeywordMatch(bool),
    YesProbability(Probability),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Observation {
    pub intent: Intent,
    pub evidence: Evidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TokenUsage {
    pub input: u32,
    pub output: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidAssessment;

impl fmt::Display for InvalidAssessment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Assessment must contain one valid observation per supported intent")
    }
}

impl std::error::Error for InvalidAssessment {}

#[derive(Debug)]
pub struct Assessment {
    observations: Vec<Observation>,
    usage: Option<TokenUsage>,
}

impl Assessment {
    pub fn new(
        mut observations: Vec<Observation>,
        usage: Option<TokenUsage>,
    ) -> Result<Self, InvalidAssessment> {
        observations.sort_by_key(|observation| observation.intent);
        if observations.len() != Intent::ALL.len()
            || !observations
                .iter()
                .map(|observation| observation.intent)
                .eq(Intent::ALL)
        {
            return Err(InvalidAssessment);
        }
        Ok(Self {
            observations,
            usage,
        })
    }

    pub fn observations(&self) -> &[Observation] {
        &self.observations
    }

    pub fn usage(&self) -> Option<TokenUsage> {
        self.usage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_validates_empty_and_utf8_byte_boundaries() {
        assert!(Message::new(" \n".into()).is_err());
        assert!(Message::new("x".repeat(4000)).is_ok());
        assert!(Message::new("x".repeat(4001)).is_err());
        assert!(Message::new("é".repeat(2000)).is_ok());
        assert!(Message::new("é".repeat(2001)).is_err());
    }

    #[test]
    fn probabilities_reject_nonfinite_and_out_of_range_values() {
        for value in [f64::NAN, f64::INFINITY, -0.01, 1.01] {
            assert!(Probability::new(value).is_err());
        }
        for value in [0.0, 0.8, 1.0] {
            assert_eq!(Probability::new(value).map(Probability::value), Ok(value));
        }
    }

    #[test]
    fn assessment_rejects_missing_and_duplicate_intents_and_normalizes_order() {
        let mut observations: Vec<_> = Intent::ALL
            .into_iter()
            .map(|intent| Observation {
                intent,
                evidence: Evidence::KeywordMatch(false),
            })
            .collect();
        assert!(Assessment::new(vec![], None).is_err());
        observations.reverse();
        let result = Assessment::new(observations.clone(), None);
        assert_eq!(result.map(|a| a.observations[0].intent), Ok(Intent::Claim));
        observations[0].intent = observations[1].intent;
        assert!(Assessment::new(observations, None).is_err());
    }
}
