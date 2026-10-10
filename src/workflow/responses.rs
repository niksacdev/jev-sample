//! Stateless structured-output calls over an OpenAI-compatible Responses API.

use std::time::Duration;

use serde_json::{Value, json};

use super::{
    contracts::CostSource,
    contracts::WorkflowUsage,
    decision::VendorAuth,
    decision::{JsonVendor, ProviderFailure, optional_cost},
    gateway::{Gateway, TokenPrice, valid_model_id},
};

#[derive(Clone)]
pub struct ResponsesClient {
    transport: JsonVendor,
    /// Sent as `model`: a model ID, or an Azure deployment name.
    model: String,
    /// What the response must report as served.
    expected: String,
    gateway: Gateway,
    price: Option<TokenPrice>,
}

impl ResponsesClient {
    pub(crate) fn new(
        endpoint: String,
        key: &str,
        model: &str,
        gateway: Gateway,
        timeout: Duration,
    ) -> Result<Self, String> {
        Self::routed(
            endpoint,
            key,
            VendorAuth::Bearer,
            model,
            model,
            gateway,
            None,
            timeout,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn routed(
        endpoint: String,
        key: &str,
        auth: VendorAuth,
        model: &str,
        expected: &str,
        gateway: Gateway,
        price: Option<TokenPrice>,
        timeout: Duration,
    ) -> Result<Self, String> {
        if !valid_model_id(model) || !valid_model_id(expected) {
            return Err("invalid_model".into());
        }
        Ok(Self {
            transport: JsonVendor::with_auth(endpoint, key, auth, timeout)?,
            model: model.into(),
            expected: expected.into(),
            gateway,
            price,
        })
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    /// Recorded provenance, qualified by router so resumes cannot switch routes.
    pub fn identity(&self) -> String {
        self.gateway.identity(&self.model)
    }

    pub fn gateway(&self) -> Gateway {
        self.gateway
    }

    /// Returns exactly one strict JSON-schema text output; refusals are provider failures.
    pub async fn structured(
        &self,
        context: &str,
        instructions: &str,
        name: &str,
        schema: Value,
        max_output_tokens: u32,
    ) -> Result<(Value, WorkflowUsage), ProviderFailure> {
        let response = self
            .transport
            .post(&json!({
                "model": self.model, "store": false, "max_output_tokens": max_output_tokens,
                "instructions": instructions, "input": context,
                "text": {"format": {"type":"json_schema","name":name,"strict":true,"schema":schema}}
            }))
            .await?;
        let served = response.get("model").and_then(Value::as_str);
        if response.get("status").and_then(Value::as_str) != Some("completed")
            || !served
                .is_some_and(|served| self.gateway.served_model_matches(&self.expected, served))
        {
            return Err(ProviderFailure::InvalidResponse);
        }
        let mut texts = Vec::new();
        for item in response
            .get("output")
            .and_then(Value::as_array)
            .ok_or(ProviderFailure::InvalidResponse)?
        {
            if item.get("type").and_then(Value::as_str) != Some("message") {
                continue;
            }
            for part in item
                .get("content")
                .and_then(Value::as_array)
                .ok_or(ProviderFailure::InvalidResponse)?
            {
                match part.get("type").and_then(Value::as_str) {
                    Some("refusal") => return Err(ProviderFailure::Provider),
                    Some("output_text") => texts.push(
                        part.get("text")
                            .and_then(Value::as_str)
                            .ok_or(ProviderFailure::InvalidResponse)?,
                    ),
                    _ => return Err(ProviderFailure::InvalidResponse),
                }
            }
        }
        if texts.len() != 1 {
            return Err(ProviderFailure::InvalidResponse);
        }
        let usage = response
            .get("usage")
            .ok_or(ProviderFailure::InvalidResponse)?;
        let tokens = |key| {
            usage
                .get(key)
                .and_then(Value::as_u64)
                .and_then(|n| u32::try_from(n).ok())
                .ok_or(ProviderFailure::InvalidResponse)
        };
        tracing::info!(
            event = "responses_call_finished",
            gateway = self.gateway.label(),
            configured_model = %self.model,
            expected_model = %self.expected,
            served_model = served,
        );
        let (input_tokens, output_tokens) = (tokens("input_tokens")?, tokens("output_tokens")?);
        let (cost_usd, cost_source) = match (optional_cost(usage)?, &self.price) {
            (Some(cost), _) => (Some(cost), Some(CostSource::Reported)),
            (None, Some(price)) => (
                Some(price.estimate(input_tokens, output_tokens)),
                Some(CostSource::Estimated),
            ),
            (None, None) => (None, None),
        };
        Ok((
            serde_json::from_str(texts[0]).map_err(|_| ProviderFailure::InvalidResponse)?,
            WorkflowUsage {
                input_tokens: Some(input_tokens),
                output_tokens: Some(output_tokens),
                attempts: 1,
                cost_usd,
                cost_source,
            },
        ))
    }
}
