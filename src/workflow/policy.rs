use super::{
    contracts::ProviderId,
    decision::{DecisionValue, Question},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GateRule {
    pub provider: ProviderId,
    pub question_id: String,
    pub min_probability: f64,
    pub min_confidence: Option<f64>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowPolicy {
    pub version: String,
    pub max_tasks: u32,
    pub max_steps: u32,
    pub deadline_ms: u32,
    pub max_comparisons: u32,
    pub max_storage_bytes: u64,
    pub max_concurrent: u32,
    pub rules: Vec<GateRule>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gate {
    Proceed,
    Clarify,
    Review,
}

impl WorkflowPolicy {
    pub fn from_json(json: &str) -> Result<Self, String> {
        let value: Self = serde_json::from_str(json).map_err(|_| "invalid_policy".to_string())?;
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<(), String> {
        let mut seen = BTreeSet::new();
        if self.version.is_empty()
            || self.version.len() > 64
            || !(1..=16).contains(&self.max_tasks)
            || !(1..=64).contains(&self.max_steps)
            || !(100..=120_000).contains(&self.deadline_ms)
            || !(1..=1000).contains(&self.max_comparisons)
            || !(65536..=64 * 1024 * 1024).contains(&self.max_storage_bytes)
            || !(1..=8).contains(&self.max_concurrent)
            || self.rules.is_empty()
        {
            return Err("invalid_policy".into());
        }
        for rule in &self.rules {
            if Question::named(&rule.question_id).is_none()
                || !probability(rule.min_probability)
                || rule.min_probability <= 0.5
                || rule.min_confidence.is_some_and(|p| !probability(p))
                || !seen.insert((rule.provider, rule.question_id.as_str()))
            {
                return Err("invalid_policy".into());
            }
        }
        Ok(())
    }
    pub fn gate(&self, provider: ProviderId, question: &Question, result: &DecisionValue) -> Gate {
        let uncertain = if question.review {
            Gate::Review
        } else {
            Gate::Clarify
        };
        let Some(rule) = self
            .rules
            .iter()
            .find(|r| r.provider == provider && r.question_id == question.id)
        else {
            return uncertain;
        };
        let (p, confidence, affirmative) = match result {
            DecisionValue::Predicate {
                probability,
                confidence,
            } => (
                *probability,
                confidence.as_ref(),
                *probability >= rule.min_probability,
            ),
            DecisionValue::Choice {
                selected,
                probabilities,
                confidence,
            } => (
                probabilities.get(selected).copied().unwrap_or(0.0),
                confidence.as_ref(),
                selected == "customer",
            ),
            DecisionValue::Score {
                probabilities,
                confidence,
                ..
            } => (
                probabilities.get("routine").copied().unwrap_or(0.0),
                confidence.as_ref(),
                true,
            ),
            DecisionValue::Deterministic { value: true }
                if provider == ProviderId::Code && question.id == "synthetic_complete" =>
            {
                return Gate::Proceed;
            }
            _ => return uncertain,
        };
        if p < rule.min_probability
            || !affirmative
            || rule
                .min_confidence
                .is_some_and(|threshold| confidence.is_none_or(|c| c.value < threshold))
        {
            uncertain
        } else {
            Gate::Proceed
        }
    }
}
pub fn probability(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}
