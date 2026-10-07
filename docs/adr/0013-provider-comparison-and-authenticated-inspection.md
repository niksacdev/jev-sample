# ADR 0013: Provider comparison and authenticated exchange inspection

Date: 2026-10-06
Status: Accepted
Supersedes: The "single provider selected at startup", "no login" and "no
customer messages are retained or logged" implementation statements in ADR 0010
for the local experiment only.

## Context and decision

The user requested choosing Code, Jev, or combinations in the UI, with the same
query assessed by each selection. The LLM comparison remains disabled until its
provider/model is selected. The UI submits each selected provider separately and
concurrently; each response remains independent, including partial failures.
Rust validates the assessor ID and composes only configured implementations.
Code denotes the explicitly limited keyword baseline, not an LLM or formal
insurance rules engine.

Jev reads `TYPESAFE_API_KEY` from the API process environment. The API loads a
root `.env` file through dotenvy when present; `.env` remains ignored and no
credential is sent to the browser or chat. Jev is unavailable when its key is
absent. The user supplies the live key locally; tests use only wiremock fixtures.

The user requested raw provider exchanges in Operator inspection. Since persona
tabs are not authorization, expose them only on `GET /v1/operator/runs` after a
constant-time bearer-key comparison against `REASSURE_OPERATOR_KEY`, configured
locally with at least 32 non-whitespace bytes. The React UI retains the key only
in memory and clears it on lock/page reload. The separate employee endpoint
returns sanitized run summaries. If no operator key is configured, raw inspection
fails closed.

For returned assessment attempts, keep a bounded copy of the actual request body
and bounded provider response body in each process-local run detail.
An application deadline or worker panic interrupts the attempt before it returns;
these failed runs have no retained provider exchange. The failure code and
sanitized execution trace remain visible; do not fabricate missing payloads.
Retaining partial exchanges across interruption is not implemented.
Jev responses are truncated at the existing
32-KiB boundary and marked; a response too large remains a visible failure.
These payloads can contain customer narratives, are not durable, and are never
written to logs or spans. Only the authenticated operator endpoint returns them.
This local shared-key control is not production identity, RBAC or a network
deployment authorization system; keep the API loopback-only.

Instrument HTTP/application/Jev spans with tracing-opentelemetry and the official
OpenTelemetry stdout exporter. Structured application events remain JSON on
stderr. Neither logs nor spans contain narratives, API keys, raw request/response
bodies or operator bearer values. `RUST_BACKTRACE=1` enables panic backtraces in
the API process diagnostics. No remote collector is configured.

## Alternatives and consequences

One server-startup provider cannot support same-query comparison. Browser-side
provider calls expose credentials and are rejected. A shared-key gate is a small
local-demo control, not a substitute for Entra/OIDC/RBAC in a deployed product.
Raw bodies are limited and memory-only to avoid silently creating a durable
sensitive-data store. Users must submit synthetic data and understand that
authenticated operator inspection can reveal it.

OpenTelemetry spans provide execution hierarchy/correlation; Rust panic
backtraces remain process diagnostics and are not exposed in the browser. The
stdout exporter makes traces visible locally without a collector; centralized
export remains future work.

## Verification and follow-up

Offline tests verify provider availability and selection, same-query Jev request,
partial failures, exchange response bounds, employee redaction, operator
authentication, contract generation and telemetry redaction. Frontend tests
verify selecting Jev plus Code, disabled LLM, distinct outcomes and key-gated
inspection. A successful live Jev call and actual account behavior remain
unverified until the user configures `.env` and explicitly tests synthetic data.
Choose and configure the LLM provider/model before enabling the LLM option.
