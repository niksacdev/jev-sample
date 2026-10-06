use std::{collections::BTreeMap, time::Duration};

use reqwest::{Client, header};
use serde::Deserialize;
use serde_json::json;

use crate::{
    agent::{INTENTS, model_signal},
    contracts::IntentSignal,
};

pub const MODEL: &str = "jev-1.13.0";
const MAX_RESPONSE: usize = 32 * 1024;

#[derive(Clone)]
pub struct Jev {
    client: Client,
    endpoint: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Failure {
    Timeout,
    Transport,
    Authentication,
    RateLimited,
    Provider,
    InvalidResponse,
    ResponseTooLarge,
}

impl Failure {
    pub fn code(self) -> &'static str {
        match self {
            Self::Timeout => "provider_timeout",
            Self::Transport => "provider_transport",
            Self::Authentication => "provider_authentication",
            Self::RateLimited => "provider_rate_limited",
            Self::Provider => "provider_error",
            Self::InvalidResponse => "invalid_provider_response",
            Self::ResponseTooLarge => "provider_response_too_large",
        }
    }
}

pub struct Assessment {
    pub signals: Vec<IntentSignal>,
    pub input_tokens: u32,
    pub output_tokens: u32,
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

    pub async fn assess(&self, message: &str) -> Result<Assessment, Failure> {
        let questions: BTreeMap<_, _> = INTENTS
            .iter()
            .map(|(_, id, instruction)| (*id, json!({"type": "noul", "instructions": instruction})))
            .collect();
        let mut response = self
            .client
            .post(&self.endpoint)
            .json(&json!({"model": MODEL, "state": message, "questions": questions}))
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
        if result.model != MODEL || result.answers.len() != INTENTS.len() {
            return Err(Failure::InvalidResponse);
        }
        let signals = INTENTS
            .iter()
            .map(|(intent, id, _)| {
                let answer = result.answers.get(*id).ok_or(Failure::InvalidResponse)?;
                if answer.kind != "noul" {
                    return Err(Failure::InvalidResponse);
                }
                model_signal(*intent, answer.noul).ok_or(Failure::InvalidResponse)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Assessment {
            signals,
            input_tokens: result.usage.input_tokens,
            output_tokens: result.usage.output_tokens,
        })
    }
}

fn transport_failure(error: reqwest::Error) -> Failure {
    if error.is_timeout() {
        Failure::Timeout
    } else {
        Failure::Transport
    }
}
