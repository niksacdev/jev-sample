//! The provider port used by the servicing workflow.

use std::{future::Future, pin::Pin};

use crate::domain::{Assessment, Message};

pub type AssessmentFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Assessment, AssessmentFailure>> + Send + 'a>>;

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
