# Jev capabilities and limitations

Status: documentation research completed on 2026-10-05. API behavior has not
been independently measured; account access and applicable account terms remain
unverified.

## Documented interface

Jev is TypeSafe AI's System One model for narrow judgments over supplied content,
not a conventional tabular predictor trained on our financial history.

`POST https://api.typesafe.ai/v1/systemone` accepts a model ID, a textual or JSON
`state`, and a map of typed questions. The response contains the model version,
answers under matching question IDs, and token usage.

| Primitive | Documented output |
| --- | --- |
| Choice | Selected option, option probabilities, derived confidence |
| Score | Probability-weighted rubric level, legend, probabilities, derived confidence |
| Noul | Probability that a yes/no statement is true; no separate confidence |

Questions evaluate the same state independently. Broad judgments should be
decomposed, with composition controlled by application code.

The documented version is `jev-1.13.0`. Pin it for evaluation rather than use a
moving alias. The documentation lists Python and JavaScript SDKs; Rust can call
the HTTP API directly.

Customer customization uses state and question criteria, not customer fine-tuning.
Inputs are text only; images, audio, and video require preprocessing outside Jev.
English is the primary training language.

## Limits and pricing at research time

- Context: 64k tokens for the full request; 32k for state plus the longest question.
- Choice: up to 255 options. Score: two to ten rubric levels.
- Price: $0.042 per million input tokens; output tokens are free.
- Published rate limits: 100k tokens/second and 80 requests/second, explicitly
  subject to change.

These are vendor documentation claims, not measured results or account guarantees.
Recheck pricing and limits before paid evaluation.

## Boundaries that affect this sample

Confidence is derived from the probability distribution, not an independent
guarantee of correctness. Domain calibration must be evaluated.

Documented weaknesses include arithmetic, counting, date comparison, complex
indirection, irrelevant context, adversarial state, and Choice option ordering.
Jev does not generate free-form text.

Exact arithmetic and routing policy belong in Rust. Generated explanations, if
added later, require a separate LLM task and evaluation. Typed output does not
guarantee factual correctness, safe decisions, or business value.

## Data handling and service terms

TypeSafe states that it does not train on customer requests or responses. This
does not mean zero retention; zero data retention is an enterprise offering.
The service is governed by customer agreements, not an assumed open-weight
license. The customer agreement restricts distillation and competing-model
training.

Use synthetic, non-sensitive fixtures initially. Do not submit credentials,
private claim files, or real claimant information as evaluation content.

## Agent and decision-provider integration

Research refreshed on 2026-10-07. TypeSafe's
[function-calling cookbook](https://docs.typesafe.ai/cookbooks/function_calling)
selects tools and closed-set arguments with typed questions. Its
[building guidance](https://docs.typesafe.ai/concepts/how-to-build-with-system-one)
keeps execution and deterministic checks in code; Jev is not an open-ended
planning/chat agent. The [OpenRouter agent-tool-gating example](https://openrouter.ai/docs/cookbook/building-agents/gate-tool-calls-with-jev)
combines an LLM-proposed call, narrow Jev predicates and approve/block/review
branches. Cookbook thresholds are examples, not validated insurance policy.

The [OpenAI Decisions API](https://developers.openai.com/api/docs/guides/decisions)
is a separate public-beta endpoint, `POST /v1/decisions`, documenting predicate,
choice and score questions. Its question/answer arrays and input formats differ
from Jev's question/answer maps. OpenAI Responses
[function calling](https://developers.openai.com/api/docs/guides/function-calling)
supports the planning/conversational loop and tool arguments; it is not the
Decisions endpoint. Keep adapters separate and explicitly report model/version
and supported capabilities.

Both providers return bounded judgments rather than a proof of correctness.
Jev's Choice/Score confidence summarizes the returned distribution; Noul has no
separate confidence field. Preserve probability of false as a confident negative,
not uncertainty. Provider substitutions require held-out domain evaluation and
separately justified thresholds, not schema similarity alone.

## Official source index

- [Documentation index](https://docs.typesafe.ai/llms.txt)
- [API reference](https://docs.typesafe.ai/api)
- [Models, pricing, limits, customization](https://docs.typesafe.ai/models)
- [State](https://docs.typesafe.ai/concepts/state)
- [Confidence](https://docs.typesafe.ai/confidence)
- [Known limitations](https://docs.typesafe.ai/model-jaggedness/jev-1.13)
- [SDKs](https://docs.typesafe.ai/sdk)
- [Legal documentation](https://docs.typesafe.ai/legal)
- [Privacy policy](https://typesafe.ai/legal/privacy-policy)
- [Master customer agreement](https://typesafe.ai/legal/mca)
- [Data processing addendum](https://typesafe.ai/legal/data-processing)
- [Acceptable use policy](https://typesafe.ai/legal/acceptable-use-policy)
