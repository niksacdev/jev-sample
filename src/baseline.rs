//! Explicitly limited keyword comparator. Not insurance policy enforcement.

use std::collections::BTreeMap;

use crate::{
    assessment::{
        AssessmentAttempt, AssessmentFailure, AssessmentFuture, Assessor, Provenance,
        ProviderExchange,
    },
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
            let observations: Vec<_> = DEFINITIONS
                .iter()
                .map(|definition| Observation {
                    intent: definition.intent,
                    evidence: Evidence::KeywordMatch(
                        definition.words.iter().any(|word| text.contains(word)),
                    ),
                })
                .collect();
            let response_body = serde_json::json!({
                "observations": observations.iter().map(|observation| {
                    let evidence = match observation.evidence {
                        Evidence::KeywordMatch(matched) => serde_json::json!({"keyword_match": matched}),
                        Evidence::YesProbability(value) => serde_json::json!({"yes_probability": value.value()}),
                    };
                    (observation.intent, evidence)
                }).collect::<BTreeMap<_, _>>()
            })
            .to_string();
            match Assessment::new(observations, None) {
                Ok(assessment) => AssessmentAttempt::success(
                    assessment,
                    ProviderExchange {
                        request_body: message.as_str().into(),
                        response_status: None,
                        response_body: Some(response_body),
                        response_truncated: false,
                    },
                ),
                Err(_) => AssessmentAttempt::failure(
                    AssessmentFailure::InvalidResponse,
                    ProviderExchange {
                        request_body: message.as_str().into(),
                        ..ProviderExchange::default()
                    },
                ),
            }
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
            .result
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
