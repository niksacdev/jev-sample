//! Explicitly limited keyword comparator. Not insurance policy enforcement.

use crate::{
    assessment::{AssessmentFailure, AssessmentFuture, Assessor, Provenance},
    domain::{Assessment, Evidence, Intent, Message, Observation},
};

struct KeywordDefinition {
    intent: Intent,
    words: &'static [&'static str],
}

const DEFINITIONS: [KeywordDefinition; 4] = [
    KeywordDefinition {
        intent: Intent::Claim,
        words: &["claim", "accident", "er visit", "collision"],
    },
    KeywordDefinition {
        intent: Intent::PolicyChange,
        words: &["beneficiary", "rider", "coverage", "policy change"],
    },
    KeywordDefinition {
        intent: Intent::CustomerDetails,
        words: &["address", "contact", "phone number", "email"],
    },
    KeywordDefinition {
        intent: Intent::Billing,
        words: &["premium", "discount", "payment", "billing"],
    },
];

pub struct KeywordBaseline;

impl Assessor for KeywordBaseline {
    fn provenance(&self) -> Provenance {
        Provenance {
            assessor: "keyword_baseline".into(),
            model: None,
            rubric_version: "keyword-baseline-v1".into(),
        }
    }

    fn assess<'a>(&'a self, message: &'a Message) -> AssessmentFuture<'a> {
        Box::pin(async move {
            let text = message.as_str().to_lowercase();
            let observations = DEFINITIONS
                .iter()
                .map(|definition| Observation {
                    intent: definition.intent,
                    evidence: Evidence::KeywordMatch(
                        definition.words.iter().any(|word| text.contains(word)),
                    ),
                })
                .collect();
            Assessment::new(observations, None).map_err(|_| AssessmentFailure::InvalidResponse)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn keywords_preserve_intent_association_without_positional_pairing()
    -> Result<(), Box<dyn std::error::Error>> {
        let assessment = KeywordBaseline
            .assess(&Message::new(
                "File a claim; update my address; delay my premium.".into(),
            )?)
            .await
            .map_err(|e| format!("{e:?}"))?;
        assert_eq!(
            assessment
                .observations()
                .iter()
                .map(|o| (o.intent, o.evidence))
                .collect::<Vec<_>>(),
            vec![
                (Intent::Claim, Evidence::KeywordMatch(true)),
                (Intent::PolicyChange, Evidence::KeywordMatch(false)),
                (Intent::CustomerDetails, Evidence::KeywordMatch(true)),
                (Intent::Billing, Evidence::KeywordMatch(true)),
            ]
        );
        assert!(assessment.usage().is_none());
        Ok(())
    }
}
