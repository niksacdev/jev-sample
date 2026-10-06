use std::{collections::BTreeMap, time::Duration};

use reqwest::{Client, header};
use serde::Deserialize;

pub use crate::assessment::AssessmentFailure as Failure;
use crate::{
    assessment::{AssessmentFuture, Assessor, Provenance},
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

    async fn evaluate(&self, message: &Message) -> Result<Assessment, Failure> {
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
        let mut response = self
            .client
            .post(&self.endpoint)
            .json(&Request {
                model: MODEL,
                state: message.as_str(),
                questions,
            })
            .send()
            .await
            .map_err(transport_failure)?;
        match response.status().as_u16() {
            200 => {}
            401 | 403 => return Err(Failure::Authentication),
            429 | 529 => return Err(Failure::RateLimited),
            _ => return Err(Failure::Provider),
        }
        if response
            .content_length()
            .is_some_and(|size| size > MAX_RESPONSE as u64)
        {
            return Err(Failure::ResponseTooLarge);
        }
        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(transport_failure)? {
            if body.len() + chunk.len() > MAX_RESPONSE {
                return Err(Failure::ResponseTooLarge);
            }
            body.extend_from_slice(&chunk);
        }
        let result: Response =
            serde_json::from_slice(&body).map_err(|_| Failure::InvalidResponse)?;
        if result.model != MODEL || result.answers.len() != QUESTIONS.len() {
            return Err(Failure::InvalidResponse);
        }
        let observations = QUESTIONS
            .iter()
            .map(|question| {
                let answer = result
                    .answers
                    .get(question.id)
                    .ok_or(Failure::InvalidResponse)?;
                if answer.kind != "noul" {
                    return Err(Failure::InvalidResponse);
                }
                let probability =
                    Probability::new(answer.noul).map_err(|_| Failure::InvalidResponse)?;
                Ok(Observation {
                    intent: question.intent,
                    evidence: Evidence::YesProbability(probability),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Assessment::new(
            observations,
            Some(TokenUsage {
                input: result.usage.input_tokens,
                output: result.usage.output_tokens,
            }),
        )
        .map_err(|_| Failure::InvalidResponse)
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
