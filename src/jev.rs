use std::{collections::BTreeMap, time::Duration};

use reqwest::{Client, header};
use serde::Deserialize;
use tracing::instrument;

pub use crate::assessment::AssessmentFailure as Failure;
use crate::{
    assessment::{AssessmentAttempt, AssessmentFuture, Assessor, Provenance, ProviderExchange},
    domain::{Assessment, Evidence, Message, Observation, Probability, TokenUsage},
    rubric::{QUESTIONS, VERSION},
};

pub const MODEL: &str = "jev-1.13.0";
const MAX_RESPONSE: usize = 32 * 1024;

#[derive(Clone)]
pub struct Jev {
    client: Client,
    endpoint: String,
}

#[derive(Deserialize)]
struct Response {
    model: String,
    answers: BTreeMap<String, Answer>,
    usage: Usage,
}

#[derive(Deserialize)]
struct Answer {
    #[serde(rename = "type")]
    kind: String,
    noul: f64,
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(serde::Serialize)]
struct Question {
    #[serde(rename = "type")]
    kind: &'static str,
    instructions: &'static str,
}

#[derive(serde::Serialize)]
struct Request<'a> {
    model: &'static str,
    state: &'a str,
    questions: BTreeMap<&'static str, Question>,
}

impl Jev {
    /// Production supplies the fixed vendor URL; tests supply a local mock URL.
    pub fn new(
        endpoint: String,
        key: &str,
        timeout: Duration,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        if key.trim().is_empty() {
            return Err(
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "Empty Jev API key").into(),
            );
        }
        let mut auth = header::HeaderValue::from_str(&format!("Bearer {key}"))?;
        auth.set_sensitive(true);
        let mut headers = header::HeaderMap::new();
        headers.insert(header::AUTHORIZATION, auth);
        let client = Client::builder()
            .default_headers(headers)
            .timeout(timeout)
            .connect_timeout(timeout.min(Duration::from_secs(5)))
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        Ok(Self { client, endpoint })
    }

    #[instrument(skip_all, fields(assessor = "jev", model = MODEL))]
    async fn evaluate(&self, message: &Message) -> AssessmentAttempt {
        let questions: BTreeMap<_, _> = QUESTIONS
            .iter()
            .map(|question| {
                (
                    question.id,
                    Question {
                        kind: "noul",
                        instructions: question.instructions,
                    },
                )
            })
            .collect();
        let request_body = match serde_json::to_string(&Request {
            model: MODEL,
            state: message.as_str(),
            questions,
        }) {
            Ok(body) => body,
            Err(_) => {
                return AssessmentAttempt::failure(Failure::Execution, ProviderExchange::default());
            }
        };
        let mut exchange = ProviderExchange {
            request_body: request_body.clone(),
            ..ProviderExchange::default()
        };
        let mut response = match self
            .client
            .post(&self.endpoint)
            .header(header::CONTENT_TYPE, "application/json")
            .body(request_body)
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                return AssessmentAttempt::failure(transport_failure(error), exchange);
            }
        };
        let status = response.status().as_u16();
        exchange.response_status = Some(status);
        let mut body = Vec::new();
        loop {
            let chunk = match response.chunk().await {
                Ok(Some(chunk)) => chunk,
                Ok(None) => break,
                Err(error) => {
                    exchange.response_body = Some(String::from_utf8_lossy(&body).into_owned());
                    return AssessmentAttempt::failure(transport_failure(error), exchange);
                }
            };
            if body.len() + chunk.len() > MAX_RESPONSE {
                body.extend_from_slice(&chunk[..MAX_RESPONSE.saturating_sub(body.len())]);
                exchange.response_body = Some(String::from_utf8_lossy(&body).into_owned());
                exchange.response_truncated = true;
                return AssessmentAttempt::failure(Failure::ResponseTooLarge, exchange);
            }
            body.extend_from_slice(&chunk);
        }
        exchange.response_body = Some(String::from_utf8_lossy(&body).into_owned());
        if status != 200 {
            let failure = match status {
                401 | 403 => Failure::Authentication,
                429 | 529 => Failure::RateLimited,
                _ => Failure::Provider,
            };
            return AssessmentAttempt::failure(failure, exchange);
        }
        let result: Response = match serde_json::from_slice(&body) {
            Ok(response) => response,
            Err(_) => return AssessmentAttempt::failure(Failure::InvalidResponse, exchange),
        };
        if result.model != MODEL || result.answers.len() != QUESTIONS.len() {
            return AssessmentAttempt::failure(Failure::InvalidResponse, exchange);
        }
        let observations = QUESTIONS
            .iter()
            .map(|question| {
                let Some(answer) = result.answers.get(question.id) else {
                    return Err(Failure::InvalidResponse);
                };
                if answer.kind != "noul" {
                    return Err(Failure::InvalidResponse);
                }
                let Ok(probability) = Probability::new(answer.noul) else {
                    return Err(Failure::InvalidResponse);
                };
                Ok(Observation {
                    intent: question.intent,
                    evidence: Evidence::YesProbability(probability),
                })
            })
            .collect::<Result<Vec<_>, _>>();
        let observations = match observations {
            Ok(observations) => observations,
            Err(failure) => return AssessmentAttempt::failure(failure, exchange),
        };
        let assessment = Assessment::new(
            observations,
            Some(TokenUsage {
                input: result.usage.input_tokens,
                output: result.usage.output_tokens,
            }),
        );
        match assessment {
            Ok(assessment) => AssessmentAttempt::success(assessment, exchange),
            Err(_) => AssessmentAttempt::failure(Failure::InvalidResponse, exchange),
        }
    }
}

impl Assessor for Jev {
    fn provenance(&self) -> Provenance {
        Provenance {
            assessor: "jev".into(),
            model: Some(MODEL.into()),
            rubric_version: VERSION.into(),
        }
    }

    fn assess<'a>(&'a self, message: &'a Message) -> AssessmentFuture<'a> {
        Box::pin(self.evaluate(message))
    }
}
fn transport_failure(error: reqwest::Error) -> Failure {
    if error.is_timeout() {
        Failure::Timeout
    } else {
        Failure::Transport
    }
}
