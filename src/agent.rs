//! Pure servicing classification and planning, not coverage or action authority.

use crate::contracts::{Intent, IntentSignal, ServicingTask, TaskState};

pub const RUBRIC_VERSION: &str = "servicing-intents-v1";
pub const ROUTING_VERSION: &str = "experiment-review-only-v1";
const INTENT_DISPLAY_THRESHOLD: f64 = 0.8;

pub fn model_signal(intent: Intent, probability: f64) -> Option<IntentSignal> {
    if !probability.is_finite() || !(0.0..=1.0).contains(&probability) {
        return None;
    }
    Some(IntentSignal {
        intent,
        probability: Some(probability),
        matched: probability >= INTENT_DISPLAY_THRESHOLD,
    })
}
pub const INTENTS: [(Intent, &str, &str); 4] = [
    (
        Intent::Claim,
        "claim",
        "Does the customer ask to file or service an insurance claim?",
    ),
    (
        Intent::PolicyChange,
        "policy_change",
        "Does the customer request a policy, beneficiary, endorsement or coverage change?",
    ),
    (
        Intent::CustomerDetails,
        "customer_details",
        "Does the customer ask to change their contact details or address?",
    ),
    (
        Intent::Billing,
        "billing",
        "Does the customer request help with premiums, payment timing or discounts?",
    ),
];

pub fn validate_message(message: &str) -> bool {
    !message.trim().is_empty() && message.len() <= 4000
}

/// Deliberately limited keyword baseline; not a model or calibrated classifier.
pub fn rules(message: &str) -> Vec<IntentSignal> {
    let text = message.to_lowercase();
    let groups = [
        ["claim", "accident", "er visit", "collision"].as_slice(),
        ["beneficiary", "rider", "coverage", "policy change"].as_slice(),
        ["address", "contact", "phone number", "email"].as_slice(),
        ["premium", "discount", "payment", "billing"].as_slice(),
    ];
    INTENTS
        .iter()
        .zip(groups)
        .map(|((intent, _, _), words)| IntentSignal {
            intent: *intent,
            probability: None,
            matched: words.iter().any(|word| text.contains(word)),
        })
        .collect()
}

pub fn plan(signals: &[IntentSignal]) -> Vec<ServicingTask> {
    signals
        .iter()
        .filter(|signal| signal.matched)
        .map(|signal| ServicingTask {
            intent: signal.intent,
            state: TaskState::ReviewRequired,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_and_rules_are_bounded_and_review_only() {
        assert!(!validate_message(" \n"));
        assert!(!validate_message(&"x".repeat(4001)));
        assert!(validate_message(&"x".repeat(4000)));
        let tasks = plan(&rules("File a claim; update my address; delay my premium."));
        assert_eq!(tasks.len(), 3);
        assert!(
            tasks
                .iter()
                .all(|task| task.state == TaskState::ReviewRequired)
        );
        assert!(plan(&rules("Hello")).is_empty());
        assert!(model_signal(Intent::Claim, f64::NAN).is_none());
        assert!(model_signal(Intent::Claim, f64::INFINITY).is_none());
        assert!(model_signal(Intent::Claim, -0.01).is_none());
        assert!(model_signal(Intent::Claim, 1.01).is_none());
        assert_eq!(
            model_signal(Intent::Claim, 0.8).map(|s| s.matched),
            Some(true)
        );
        assert_eq!(
            model_signal(Intent::Claim, 0.799).map(|s| s.matched),
            Some(false)
        );
    }
}
