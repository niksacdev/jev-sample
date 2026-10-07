//! The provider port used by the servicing workflow.

use std::{future::Future, pin::Pin};

use crate::domain::{Assessment, Message};

pub type AssessmentFuture<'a> = Pin<Box<dyn Future<Output = AssessmentAttempt> + Send + 'a>>;

#[derive(Clone, Debug, Default, serde::Serialize, ts_rs::TS)]
pub struct ProviderExchange {
    pub request_body: String,
    pub response_status: Option<u16>,
    pub response_body: Option<String>,
    pub response_truncated: bool,
}

pub struct AssessmentAttempt {
    pub result: Result<Assessment, AssessmentFailure>,
    pub exchange: ProviderExchange,
}

impl AssessmentAttempt {
    pub fn success(assessment: Assessment, exchange: ProviderExchange) -> Self {
        Self {
            result: Ok(assessment),
            exchange,
        }
    }

    pub fn failure(failure: AssessmentFailure, exchange: ProviderExchange) -> Self {
        Self {
            result: Err(failure),
            exchange,
        }
    }
}

/// Object-safe async port, implemented by the baseline, Jev and test assessors.
/// Output is validated domain evidence, never routing or authorization.
pub trait Assessor: Send + Sync {
    fn provenance(&self) -> Provenance;
    fn assess<'a>(&'a self, message: &'a Message) -> AssessmentFuture<'a>;
}

#[derive(Clone, Debug)]
pub struct Provenance {
    pub assessor: String,
    pub model: Option<String>,
    pub rubric_version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssessmentFailure {
    Timeout,
    Transport,
    Authentication,
    RateLimited,
    Provider,
    InvalidResponse,
    ResponseTooLarge,
    Execution,
}

impl AssessmentFailure {
    pub fn code(self) -> &'static str {
        match self {
            Self::Timeout => "provider_timeout",
            Self::Transport => "provider_transport",
            Self::Authentication => "provider_authentication",
            Self::RateLimited => "provider_rate_limited",
            Self::Provider => "provider_error",
            Self::InvalidResponse => "invalid_provider_response",
            Self::ResponseTooLarge => "provider_response_too_large",
            Self::Execution => "assessment_execution_failed",
        }
    }
}
