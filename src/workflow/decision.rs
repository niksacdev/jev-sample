use super::{
    contracts::{ProviderId, WorkflowUsage},
    gateway::{Gateway, OPENAI_DECISIONS, OPENROUTER_DECISIONS, TYPESAFE_SYSTEMONE},
    policy::probability,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    time::{Duration, Instant},
};

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum QuestionKind {
    Predicate,
    Choice { options: Vec<String> },
    Score { levels: Vec<String> },
}
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct Question {
    pub id: String,
    pub version: String,
    pub instructions: String,
    pub kind: QuestionKind,
    pub review: bool,
}
impl Question {
    pub fn valid(&self) -> bool {
        let labels = match &self.kind {
            QuestionKind::Predicate => {
                return !self.id.is_empty()
                    && self.id.len() <= 80
                    && !self.version.is_empty()
                    && self.version.len() <= 80
                    && !self.instructions.trim().is_empty()
                    && self.instructions.len() <= 4000;
            }
            QuestionKind::Choice { options } => options,
            QuestionKind::Score { levels } => levels,
        };
        !self.id.is_empty()
            && self.id.len() <= 80
            && !self.version.is_empty()
            && self.version.len() <= 80
            && !self.instructions.trim().is_empty()
            && self.instructions.len() <= 4000
            && (2..=10).contains(&labels.len())
            && labels.iter().all(|l| !l.trim().is_empty() && l.len() <= 80)
            && labels
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == labels.len()
    }
    pub fn named(id: &str) -> Option<Self> {
        let (instructions, kind, review) = match id {
            "synthetic_complete" => (
                "Does the supplied synthetic message explicitly contain a synthetic reference in the form SYN- followed by digits?",
                QuestionKind::Predicate,
                false,
            ),
            "synthetic_route" => (
                "Choose customer when the synthetic request can continue as read-only information; choose employee when it asks for a consequential or unsupported action.",
                QuestionKind::Choice {
                    options: vec!["customer".into(), "employee".into()],
                },
                true,
            ),
            "synthetic_priority" => (
                "Classify the synthetic read-only request's urgency as routine or urgent. This is not a real insurance priority decision.",
                QuestionKind::Score {
                    levels: vec!["routine".into(), "urgent".into()],
                },
                true,
            ),
            _ => return None,
        };
        Some(Self {
            id: id.into(),
            version: "synthetic-questions-1".into(),
            instructions: instructions.into(),
            kind,
            review,
        })
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Confidence {
    pub value: f64,
    pub semantics: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DecisionValue {
    Predicate {
        probability: f64,
        confidence: Option<Confidence>,
    },
    Choice {
        selected: String,
        probabilities: BTreeMap<String, f64>,
        confidence: Option<Confidence>,
    },
    Score {
        value: f64,
        probabilities: BTreeMap<String, f64>,
        confidence: Option<Confidence>,
    },
    Deterministic {
        value: bool,
    },
    Unsupported,
    Refusal,
}
impl DecisionValue {
    pub fn validate(&self, question: &Question) -> bool {
        if !question.valid() {
            return false;
        }
        let confidence_valid = |c: &Option<Confidence>| {
            c.as_ref().is_none_or(|c| {
                probability(c.value) && c.semantics == "vendor_distribution_derived"
            })
        };
        let distribution_valid = |p: &BTreeMap<String, f64>, options: &[String]| {
            p.len() == options.len()
                && options
                    .iter()
                    .all(|o| p.get(o).is_some_and(|p| probability(*p)))
                && (p.values().sum::<f64>() - 1.0).abs() <= 0.001
        };
        match (self, &question.kind) {
            (
                Self::Predicate {
                    probability: p,
                    confidence,
                },
                QuestionKind::Predicate,
            ) => probability(*p) && confidence_valid(confidence),
            (
                Self::Choice {
                    selected,
                    probabilities,
                    confidence,
                },
                QuestionKind::Choice { options },
            ) => {
                options.contains(selected)
                    && distribution_valid(probabilities, options)
                    && confidence_valid(confidence)
                    && probabilities
                        .get(selected)
                        .is_some_and(|chosen| probabilities.values().all(|p| p <= chosen))
            }
            (
                Self::Score {
                    value,
                    probabilities,
                    confidence,
                },
                QuestionKind::Score { levels },
            ) => {
                value.is_finite()
                    && *value >= 0.0
                    && *value <= (levels.len() - 1) as f64
                    && distribution_valid(probabilities, levels)
                    && confidence_valid(confidence)
                    && (levels
                        .iter()
                        .enumerate()
                        .map(|(i, l)| i as f64 * probabilities[l])
                        .sum::<f64>()
                        - *value)
                        .abs()
                        <= 0.001
            }
            (Self::Deterministic { .. }, QuestionKind::Predicate) => {
                question.id == "synthetic_complete"
            }
            (Self::Unsupported | Self::Refusal, _) => true,
            _ => false,
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderFailure {
    Authentication,
    InsufficientCredits,
    RateLimited,
    Timeout,
    Transport,
    Provider,
    InvalidResponse,
    ResponseTooLarge,
    Execution,
}
impl ProviderFailure {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Authentication => "authentication",
            Self::InsufficientCredits => "insufficient_credits",
            Self::RateLimited => "rate_limited",
            Self::Timeout => "timeout",
            Self::Transport => "transport",
            Self::Provider => "provider",
            Self::InvalidResponse => "invalid_response",
            Self::ResponseTooLarge => "response_too_large",
            Self::Execution => "execution",
        }
    }
}
#[derive(Clone, Deserialize, Serialize)]
pub struct DecisionAttempt {
    pub answers: Result<BTreeMap<String, DecisionValue>, ProviderFailure>,
    pub usage: WorkflowUsage,
    pub elapsed_ms: u32,
}
pub type DecisionFuture<'a> = Pin<Box<dyn Future<Output = DecisionAttempt> + Send + 'a>>;
pub trait DecisionProvider: Send + Sync {
    fn id(&self) -> ProviderId;
    fn model(&self) -> Option<String>;
    fn capability(&self) -> String;
    fn evaluate<'a>(&'a self, context: &'a str, questions: &'a [Question]) -> DecisionFuture<'a>;
}
pub struct DeterministicProvider;
impl DecisionProvider for DeterministicProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Code
    }
    fn model(&self) -> Option<String> {
        None
    }
    fn capability(&self) -> String {
        "Only synthetic_complete predicate; deterministic boolean, no probability/confidence".into()
    }
    fn evaluate<'a>(&'a self, context: &'a str, questions: &'a [Question]) -> DecisionFuture<'a> {
        Box::pin(async move {
            let parsed = serde_json::from_str::<serde_json::Value>(context);
            let Some(message) = parsed
                .as_ref()
                .ok()
                .and_then(|v| v.get("message").and_then(|v| v.as_str()))
            else {
                return DecisionAttempt {
                    answers: Err(ProviderFailure::InvalidResponse),
                    usage: WorkflowUsage {
                        attempts: 1,
                        ..Default::default()
                    },
                    elapsed_ms: 0,
                };
            };
            let complete = message
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '-')
                .any(|word| {
                    word.strip_prefix("SYN-")
                        .is_some_and(|s| !s.is_empty() && s.bytes().all(|c| c.is_ascii_digit()))
                });
            let answers = questions
                .iter()
                .map(|q| {
                    (
                        q.id.clone(),
                        if Question::named(&q.id).as_ref() == Some(q)
                            && q.id == "synthetic_complete"
                        {
                            DecisionValue::Deterministic { value: complete }
                        } else {
                            DecisionValue::Unsupported
                        },
                    )
                })
                .collect();
            DecisionAttempt {
                answers: Ok(answers),
                usage: WorkflowUsage::default(),
                elapsed_ms: 0,
            }
        })
    }
}

#[derive(Clone)]
pub struct JsonVendor {
    pub(crate) client: reqwest::Client,
    endpoint: String,
}
impl JsonVendor {
    pub(crate) fn new(endpoint: String, key: &str, timeout: Duration) -> Result<Self, String> {
        if key.trim().is_empty() {
            return Err("missing_key".into());
        }
        let mut auth = reqwest::header::HeaderValue::from_str(&format!("Bearer {key}"))
            .map_err(|_| "invalid_key")?;
        auth.set_sensitive(true);
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::AUTHORIZATION, auth);
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(timeout)
            .connect_timeout(timeout.min(Duration::from_secs(5)))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| "client")?;
        Ok(Self { client, endpoint })
    }
    #[tracing::instrument(skip_all)]
    pub(crate) async fn post(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, ProviderFailure> {
        let failure = |e: reqwest::Error| {
            if e.is_timeout() {
                ProviderFailure::Timeout
            } else {
                ProviderFailure::Transport
            }
        };
        let mut response = self
            .client
            .post(&self.endpoint)
            .json(body)
            .send()
            .await
            .map_err(failure)?;
        let status = response.status().as_u16();
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(failure)? {
            if bytes.len() + chunk.len() > 65536 {
                return Err(ProviderFailure::ResponseTooLarge);
            }
            bytes.extend_from_slice(&chunk);
        }
        if status != 200 {
            return Err(match status {
                401 | 403 => ProviderFailure::Authentication,
                402 => ProviderFailure::InsufficientCredits,
                429 | 529 => ProviderFailure::RateLimited,
                _ => ProviderFailure::Provider,
            });
        }
        serde_json::from_slice(&bytes).map_err(|_| ProviderFailure::InvalidResponse)
    }
}
#[derive(Clone)]
pub struct VendorDecisions {
    transport: JsonVendor,
    id: ProviderId,
    model: String,
    gateway: Gateway,
}
impl VendorDecisions {
    pub fn openai(key: &str, model: &str, timeout: Duration) -> Result<Self, String> {
        Self::new(
            OPENAI_DECISIONS.into(),
            key,
            model,
            ProviderId::Openai,
            Gateway::Direct,
            timeout,
        )
    }
    pub fn jev(key: &str, timeout: Duration) -> Result<Self, String> {
        Self::new(
            TYPESAFE_SYSTEMONE.into(),
            key,
            "jev-1.13.0",
            ProviderId::Jev,
            Gateway::Direct,
            timeout,
        )
    }
    /// Jev through OpenRouter's alpha Decisions API, which keeps the TypeSafe wire format.
    pub fn jev_openrouter(key: &str, model: &str, timeout: Duration) -> Result<Self, String> {
        Self::new(
            OPENROUTER_DECISIONS.into(),
            key,
            model,
            ProviderId::Jev,
            Gateway::OpenRouter,
            timeout,
        )
    }
    /// Local loopback mocks only; runtime composition cannot configure vendor URLs.
    pub fn for_test(
        endpoint: String,
        key: &str,
        model: &str,
        id: ProviderId,
        timeout: Duration,
    ) -> Result<Self, String> {
        Self::for_test_via(endpoint, key, model, id, Gateway::Direct, timeout)
    }
    pub fn for_test_via(
        endpoint: String,
        key: &str,
        model: &str,
        id: ProviderId,
        gateway: Gateway,
        timeout: Duration,
    ) -> Result<Self, String> {
        if !crate::workflow::gateway::is_loopback_test_endpoint(&endpoint) {
            return Err("test_endpoint_must_be_loopback".into());
        }
        Self::new(endpoint, key, model, id, gateway, timeout)
    }
    fn new(
        endpoint: String,
        key: &str,
        model: &str,
        id: ProviderId,
        gateway: Gateway,
        timeout: Duration,
    ) -> Result<Self, String> {
        if model.is_empty() || model.len() > 100 || id == ProviderId::Code {
            return Err("invalid_model".into());
        }
        Ok(Self {
            transport: JsonVendor::new(endpoint, key, timeout)?,
            id,
            model: model.into(),
            gateway,
        })
    }
    async fn run(&self, context: &str, questions: &[Question]) -> DecisionAttempt {
        let started = Instant::now();
        let mut usage = WorkflowUsage {
            attempts: 1,
            ..Default::default()
        };
        let answers=async {
            if context.len()>32000 || questions.is_empty() || questions.len()>8
                || questions.iter().any(|q|!q.valid())
                || questions.iter().map(|q|&q.id).collect::<std::collections::BTreeSet<_>>().len()!=questions.len() {
                return Err(ProviderFailure::InvalidResponse)
            }
            let request=if self.id==ProviderId::Jev {
                let questions=questions.iter().map(|q|{
                    let value=match &q.kind {
                        QuestionKind::Predicate=>serde_json::json!({"type":"noul","instructions":q.instructions}),
                        QuestionKind::Choice{options}=>serde_json::json!({"type":"choice","instructions":q.instructions,"criteria":options.iter().map(|o|(o,o)).collect::<BTreeMap<_,_>>()}),
                        QuestionKind::Score{levels}=>serde_json::json!({"type":"score","instructions":q.instructions,"criteria":levels}),
                    };
                    (q.id.clone(),value)
                }).collect::<BTreeMap<_,_>>();
                serde_json::json!({"model":self.model,"state":context,"questions":questions})
            } else {
                let questions=questions.iter().map(|q|match &q.kind {
                    QuestionKind::Predicate=>serde_json::json!({"name":q.id,"type":"predicate","instructions":q.instructions}),
                    QuestionKind::Choice{options}=>serde_json::json!({"name":q.id,"type":"choice","instructions":q.instructions,"choices":options.iter().map(|o|serde_json::json!({"value":o,"description":o})).collect::<Vec<_>>()}),
                    QuestionKind::Score{levels}=>serde_json::json!({"name":q.id,"type":"score","instructions":q.instructions,"levels":levels.iter().map(|o|serde_json::json!({"label":o,"description":o})).collect::<Vec<_>>()}),
                }).collect::<Vec<_>>();
                serde_json::json!({"model":self.model,"input":context,"questions":questions})
            };
            let response=self.transport.post(&request).await?;
            let served=response.get("model").and_then(|v|v.as_str());
            if !served.is_some_and(|served|self.gateway.served_model_matches(&self.model,served)){return Err(ProviderFailure::InvalidResponse)}
            tracing::info!(event="vendor_decision_served",gateway=self.gateway.label(),configured_model=%self.model,served_model=served,served_by=response.get("provider").and_then(|v|v.as_str()));
            if let Some(wire_usage)=response.get("usage").filter(|v|!v.is_null()) {
                usage.input_tokens=optional_u32(wire_usage,"input_tokens")?;
                usage.output_tokens=optional_u32(wire_usage,"output_tokens")?;
                usage.cost_usd=optional_cost(wire_usage)?;
            }
            let answer_values: BTreeMap<String,serde_json::Value>=if self.id==ProviderId::Jev {
                serde_json::from_value(response.get("answers").cloned().ok_or(ProviderFailure::InvalidResponse)?).map_err(|_|ProviderFailure::InvalidResponse)?
            } else {
                let array=response.get("answers").and_then(|v|v.as_array()).ok_or(ProviderFailure::InvalidResponse)?;
                let mut values=BTreeMap::new();
                for answer in array {
                    let name=answer.get("name").and_then(|v|v.as_str()).ok_or(ProviderFailure::InvalidResponse)?;
                    if values.insert(name.to_owned(),answer.clone()).is_some(){return Err(ProviderFailure::InvalidResponse)}
                }
                values
            };
            if answer_values.len()!=questions.len(){return Err(ProviderFailure::InvalidResponse)}
            questions.iter().map(|q|{
                let value=answer_values.get(&q.id).ok_or(ProviderFailure::InvalidResponse)?;
                let answer=parse_answer(value,q,self.id)?;
                if !answer.validate(q){return Err(ProviderFailure::InvalidResponse)}
                Ok((q.id.clone(),answer))
            }).collect()
        }.await;
        DecisionAttempt {
            answers,
            usage,
            elapsed_ms: started.elapsed().as_millis().min(u32::MAX as u128) as u32,
        }
    }
}
/// Gateway-reported spend in US dollars; absent or null when the route does not report cost.
pub(crate) fn optional_cost(value: &serde_json::Value) -> Result<Option<f64>, ProviderFailure> {
    value
        .get("cost")
        .filter(|v| !v.is_null())
        .map(|v| {
            v.as_f64()
                .filter(|cost| cost.is_finite() && *cost >= 0.0)
                .ok_or(ProviderFailure::InvalidResponse)
        })
        .transpose()
}
fn optional_u32(value: &serde_json::Value, key: &str) -> Result<Option<u32>, ProviderFailure> {
    value
        .get(key)
        .filter(|v| !v.is_null())
        .map(|v| {
            v.as_u64()
                .and_then(|v| u32::try_from(v).ok())
                .ok_or(ProviderFailure::InvalidResponse)
        })
        .transpose()
}
fn parse_answer(
    value: &serde_json::Value,
    question: &Question,
    id: ProviderId,
) -> Result<DecisionValue, ProviderFailure> {
    if value.get("type").and_then(|v| v.as_str()) == Some("refusal") {
        return Ok(DecisionValue::Refusal);
    }
    if value
        .get("refusal")
        .is_some_and(|v| !v.is_null() && v != false)
    {
        if value
            .get("refusal")
            .is_some_and(|v| v.as_str().is_some() || v == true)
        {
            return Ok(DecisionValue::Refusal);
        }
        return Err(ProviderFailure::InvalidResponse);
    }
    let confidence = value
        .get("confidence")
        .filter(|v| !v.is_null())
        .map(|v| {
            v.as_f64()
                .ok_or(ProviderFailure::InvalidResponse)
                .map(|value| Confidence {
                    value,
                    semantics: "vendor_distribution_derived".into(),
                })
        })
        .transpose()?;
    if matches!(
        question.kind,
        QuestionKind::Choice { .. } | QuestionKind::Score { .. }
    ) && confidence.is_none()
    {
        return Err(ProviderFailure::InvalidResponse);
    }
    let number = |key: &str| {
        value
            .get(key)
            .and_then(|v| v.as_f64())
            .ok_or(ProviderFailure::InvalidResponse)
    };
    let probabilities = || -> Result<BTreeMap<String, f64>, ProviderFailure> {
        let source = value
            .get("probabilities")
            .ok_or(ProviderFailure::InvalidResponse)?;
        let options = match &question.kind {
            QuestionKind::Choice { options } => options,
            QuestionKind::Score { levels } => levels,
            _ => return Err(ProviderFailure::InvalidResponse),
        };
        let mut result = BTreeMap::new();
        if id == ProviderId::Openai {
            for entry in source.as_array().ok_or(ProviderFailure::InvalidResponse)? {
                let key = if matches!(question.kind, QuestionKind::Score { .. }) {
                    let index = entry
                        .get("value")
                        .and_then(|v| v.as_u64())
                        .ok_or(ProviderFailure::InvalidResponse)?;
                    let label = options
                        .get(usize::try_from(index).map_err(|_| ProviderFailure::InvalidResponse)?)
                        .ok_or(ProviderFailure::InvalidResponse)?
                        .clone();
                    if entry.get("label").and_then(|v| v.as_str()) != Some(label.as_str()) {
                        return Err(ProviderFailure::InvalidResponse);
                    }
                    label
                } else {
                    entry
                        .get("value")
                        .and_then(|v| v.as_str())
                        .ok_or(ProviderFailure::InvalidResponse)?
                        .into()
                };
                let p = entry
                    .get("probability")
                    .and_then(|v| v.as_f64())
                    .ok_or(ProviderFailure::InvalidResponse)?;
                if result.insert(key, p).is_some() {
                    return Err(ProviderFailure::InvalidResponse);
                }
            }
        } else {
            for (key, p) in source.as_object().ok_or(ProviderFailure::InvalidResponse)? {
                let key = if matches!(question.kind, QuestionKind::Score { .. }) {
                    let label = options
                        .get(
                            key.parse::<usize>()
                                .map_err(|_| ProviderFailure::InvalidResponse)?,
                        )
                        .ok_or(ProviderFailure::InvalidResponse)?
                        .clone();
                    if value
                        .get("legend")
                        .and_then(|v| v.get(key))
                        .and_then(|v| v.as_str())
                        != Some(label.as_str())
                    {
                        return Err(ProviderFailure::InvalidResponse);
                    }
                    label
                } else {
                    key.clone()
                };
                if result
                    .insert(key, p.as_f64().ok_or(ProviderFailure::InvalidResponse)?)
                    .is_some()
                {
                    return Err(ProviderFailure::InvalidResponse);
                }
            }
        }
        Ok(result)
    };
    let kind = value
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or(ProviderFailure::InvalidResponse)?;
    match &question.kind {
        QuestionKind::Predicate
            if kind
                == if id == ProviderId::Jev {
                    "noul"
                } else {
                    "predicate"
                } =>
        {
            Ok(DecisionValue::Predicate {
                probability: number(if id == ProviderId::Jev {
                    "noul"
                } else {
                    "probability"
                })?,
                confidence,
            })
        }
        QuestionKind::Choice { .. } if kind == "choice" => Ok(DecisionValue::Choice {
            selected: value
                .get("choice")
                .and_then(|v| v.as_str())
                .ok_or(ProviderFailure::InvalidResponse)?
                .into(),
            probabilities: probabilities()?,
            confidence,
        }),
        QuestionKind::Score { .. } if kind == "score" => Ok(DecisionValue::Score {
            value: number("score")?,
            probabilities: probabilities()?,
            confidence,
        }),
        _ => Err(ProviderFailure::InvalidResponse),
    }
}
impl DecisionProvider for VendorDecisions {
    fn id(&self) -> ProviderId {
        self.id
    }
    fn model(&self) -> Option<String> {
        Some(self.model.clone())
    }
    fn capability(&self) -> String {
        "Versioned bounded Predicate, Choice, Score questions; vendor probabilities are not locally calibrated".into()
    }
    fn evaluate<'a>(&'a self, context: &'a str, questions: &'a [Question]) -> DecisionFuture<'a> {
        Box::pin(self.run(context, questions))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn finite_complete_distributions_and_consistent_selected_values_are_required() {
        let q = Question::named("synthetic_route").unwrap();
        for malformed in [
            json!({"type":"choice","choice":"customer","confidence":0.8,"probabilities":{"customer":0.9}}),
            json!({"type":"choice","choice":"customer","confidence":0.8,"probabilities":{"customer":0.9,"employee":0.9}}),
            json!({"type":"choice","choice":"customer","confidence":0.8,"probabilities":{"customer":0.1,"employee":0.9}}),
            json!({"type":"choice","choice":"unknown","confidence":0.8,"probabilities":{"customer":0.9,"employee":0.1}}),
            json!({"type":"choice","choice":"customer","confidence":1.5,"probabilities":{"customer":0.9,"employee":0.1}}),
        ] {
            assert!(
                !parse_answer(&malformed, &q, ProviderId::Jev)
                    .unwrap()
                    .validate(&q)
            );
        }
        let missing = json!({"type":"choice","choice":"customer","probabilities":{"customer":0.9,"employee":0.1}});
        assert!(parse_answer(&missing, &q, ProviderId::Jev).is_err());
        let duplicate = json!({"type":"choice","choice":"customer","confidence":0.8,"probabilities":[{"value":"customer","probability":0.9},{"value":"customer","probability":0.1}]});
        assert!(parse_answer(&duplicate, &q, ProviderId::Openai).is_err());
    }

    #[test]
    fn empty_score_dimensions_do_not_panic_and_score_is_weighted_evidence() {
        let mut q = Question::named("synthetic_priority").unwrap();
        let answer = DecisionValue::Score {
            value: 0.1,
            probabilities: BTreeMap::from([("routine".into(), 0.9), ("urgent".into(), 0.1)]),
            confidence: None,
        };
        assert!(answer.validate(&q));
        assert!(
            !DecisionValue::Score {
                value: 0.9,
                probabilities: BTreeMap::from([("routine".into(), 0.9), ("urgent".into(), 0.1)]),
                confidence: None
            }
            .validate(&q)
        );
        q.kind = QuestionKind::Score { levels: vec![] };
        assert!(!answer.validate(&q));
    }

    #[test]
    fn refusals_remain_refusals_not_technical_failures_or_probabilities() {
        let q = Question::named("synthetic_complete").unwrap();
        assert!(matches!(
            parse_answer(
                &json!({"type":"refusal","name":q.id}),
                &q,
                ProviderId::Openai
            )
            .unwrap(),
            DecisionValue::Refusal
        ));
        assert!(
            !DecisionValue::Predicate {
                probability: f64::NAN,
                confidence: None
            }
            .validate(&q)
        );
        assert!(
            !DecisionValue::Predicate {
                probability: 1.1,
                confidence: None
            }
            .validate(&q)
        );
    }
}
