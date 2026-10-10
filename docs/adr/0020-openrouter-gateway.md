# ADR 0020: OpenRouter as the single-key model gateway

Date: 2026-10-09
Status: Accepted
Tracking issue: [20](https://github.com/niksacdev/zipclaim/issues/20)

## Context and user direction

Setup asked for separate OpenAI and Jev keys plus a model name. The user asked
to use OpenRouter instead, partly to learn it. OpenRouter serves
`typesafe/jev-1.13` through its alpha Decisions API
(`POST /api/alpha/decisions`, TypeSafe wire format) and OpenAI models through a
stateless Responses API. It bills prepaid credits and reports `usage.cost`.

## Decision

One `OPENROUTER_API_KEY` (startup) or one setup field fills every missing slot:

| Slot | Adapter | Default model (`config/openrouter.json`) |
|------|---------|------------------------------------------|
| Planner | `OpenAiPlanner` over OpenRouter Responses | `openai/gpt-6-astra` |
| Jev | `VendorDecisions` over OpenRouter Decisions | `typesafe/jev-1.13` |
| OpenAI comparison | `LlmDecisions`, an LLM-as-decider over Responses with a strict per-question JSON schema | `openai/gpt-5.4-mini` |

- Endpoints are fixed constants in `src/workflow/gateway.rs`; tests can inject
  loopback endpoints only. Model IDs are validated and can be overridden with
  `OPENROUTER_PLANNER_MODEL`, `OPENROUTER_DECISION_MODEL` and
  `OPENROUTER_JEV_MODEL`.
- Direct `OPENAI_*` and `TYPESAFE_API_KEY` settings still take precedence.
- Through the gateway, a served model may be the configured ID or a dated
  snapshot (`<configured>-<date>`); any other model is rejected.
- HTTP 402 maps to a typed `insufficient_credits` provider failure. No silent
  fallback.
- `WorkflowUsage.cost_usd` records reported cost; Geek Mode shows it.
- The LLM decider returns self-reported, uncalibrated probabilities with no
  confidence. Distributions must sum to 1 within 0.02 or the answer is rejected.

## Consequences

OpenRouter becomes an additional data processor; the agreement and disclaimer
say so. The Decisions API is alpha. Strict json_schema support and the served
model format for these models are not yet verified by a live call; tests use
mocked contracts only. The legacy Assessment lab Jev path is unchanged.
