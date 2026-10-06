# ADR 0010: React UI delegates servicing to Rust

Date: 2026-10-05
Status: Accepted

## Context and decision

The user selected TypeScript/React for the UI and Rust for backend execution.
The browser delegates customer messages to the application API. Rust owns
classification, task planning, provider calls and deterministic controls.
Customer contracts contain no provider/model selection or probability fields.
Operator inspection includes model/rubric/routing versions and measured usage.
Generate browser DTO types with ts-rs from Rust and check for drift in tests.

The first implementation is a local, synthetic servicing experiment, not the
full autonomous claims architecture. A customer submits a message; a bounded
coordinator assesses four servicing intents and prepares review-only tasks.
The process uses the keyword baseline by default; explicit server configuration
selects Jev. Errors never fall back to the baseline or fabricated results.
The baseline is not an LLM comparator or an evaluation result.

Scenario direction is informed by the user-selected
[Galileo insurance split](https://huggingface.co/datasets/galileo-ai/agent-leaderboard-v2/viewer/adaptive_tool_use/insurance?row=0):
one conversation can contain several servicing requests. The UI stories are
original synthetic development examples, not an imported benchmark or held-out
evaluation. Claims, policy changes, customer details and billing are coarse
intent groups, not a complete decomposition of the six goals in Galileo row 0.
This servicing rubric does not replace the approved auto-intake evaluation
rubric, dataset splits or statistical gates.

Jev uses four Noul questions, model jev-1.13.0 and rubric servicing-intents-v1.
The exploratory 0.8 probability threshold selects displayed tasks, never grants
authority. All tasks require review. No claim, policy, customer or payment action
is executed; no human assignment is claimed.

## Alternatives and consequences

A Rust/WASM UI would add a second Rust framework without improving our browser
workflow requirements. Browser-side inference would expose credentials and couple
the customer experience to provider contracts.

For this disposable experiment only, retain at most 100 process-local run
metadata records in memory; refuse further work at that bound. Restart loses
history. No customer messages are retained or logged. No durable scheduling,
background agent fleet, MCP implementation, login, RBAC, external customer
system or RAG exists in this slice. Persona tabs share one synthetic workspace
and are not access controls. Never expose this server beyond loopback.
Production workflow persistence/authentication still requires separate decisions.

Use one reused reqwest client, a 15-second whole-request deadline, a 32-KiB
response limit, no redirects, no automatic retries and four concurrent requests.
Validate exact model/question IDs, answer type, required usage and finite
probabilities in [0,1]. API input is limited to 8192 bytes; narrative to 4000
UTF-8 bytes. Provider failure returns 502 and a recorded failed run.
Browser writes accept only the local Vite origin; no cross-origin read permission.

## Verification and follow-up

Axum/Tower route tests, real adapter wiremock tests and frontend component tests
require no credentials or paid inference. Live Jev behavior remains unverified
until an authorized synthetic call succeeds with the user's account.
Build/type-check the frontend and generate contracts before serving.
The next milestone introduces scoped tool/context contracts and workflow
transitions, not fake completion of claims. Full-journey evaluation and a
structured-output LLM comparator remain pending.

Dependency review: the initial frontend audit found GHSA-68fv-2mgg-jv7q in
source-map-js 1.2.1. The configured package feed did not provide fixed 1.2.2.
Pin the upstream v1.2.2 release commit
0a1d334fd1e55a47df97fcd60a7915d46df3b08a through an npm override; the lockfile
records archive integrity. The upstream diff validates section offsets, bounds
nested offsets and fixes amplification. No advisory suppression is permitted.
Replace the archive override with registry 1.2.2 or newer once the feed supplies
it, preserving clean audits and frontend checks. This tracked follow-up belongs
to the sample maintainer and is triggered by package-feed availability.
