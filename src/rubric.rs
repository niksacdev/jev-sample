use crate::domain::Intent;

pub(crate) const VERSION: &str = "servicing-intents-v1";

pub(crate) struct IntentQuestion {
    pub intent: Intent,
    pub id: &'static str,
    pub instructions: &'static str,
}

/// Immutable versioned Jev question definitions, not a routing policy.
pub(crate) const QUESTIONS: [IntentQuestion; 4] = [
    IntentQuestion {
        intent: Intent::Claim,
        id: "claim",
        instructions: "Does the customer ask to file or service an insurance claim?",
    },
    IntentQuestion {
        intent: Intent::PolicyChange,
        id: "policy_change",
        instructions: "Does the customer request a policy, beneficiary, endorsement or coverage change?",
    },
    IntentQuestion {
        intent: Intent::CustomerDetails,
        id: "customer_details",
        instructions: "Does the customer ask to change their contact details or address?",
    },
    IntentQuestion {
        intent: Intent::Billing,
        id: "billing",
        instructions: "Does the customer request help with premiums, payment timing or discounts?",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn question_ids_and_intents_are_unique_and_complete() {
        assert_eq!(
            QUESTIONS.iter().map(|q| q.intent).collect::<BTreeSet<_>>(),
            Intent::ALL.into_iter().collect()
        );
        assert_eq!(
            QUESTIONS
                .iter()
                .map(|q| q.id)
                .collect::<BTreeSet<_>>()
                .len(),
            QUESTIONS.len()
        );
        assert!(QUESTIONS.iter().all(|q| !q.instructions.trim().is_empty()));
    }
}
