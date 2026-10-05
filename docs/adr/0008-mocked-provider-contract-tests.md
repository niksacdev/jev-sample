# ADR 0008: Test provider contracts with local HTTP mocks

Date: 2026-10-05
Status: Accepted

## Context

The user requires Rust mocks and integration tests as implementation starts,
so the agent can exercise planned contracts without calling live model services.
Mocking only an assessment method would miss HTTP serialization, status handling,
and adapter validation errors.

## Decision

Use [wiremock](https://docs.rs/wiremock/) as a Rust development dependency for
provider HTTP integration tests. Add it with the first adapter test that uses it,
and track the resolved version in Cargo.lock. Each test starts its own local mock
server and injects its URL into the real adapter's client configuration.
Tests must never default to a live provider URL or load production credentials.

Run the real HTTP client, request serialization, response parsing, and domain
validation against synthetic provider responses. Assert request method, path,
model version, required question IDs, and relevant headers/body fields, using
dummy credentials. Verify expected request counts, including retry behavior.

Keep HTTP mocks outside the domain. Pure domain/routing tests use typed values;
workflow tests use small deterministic implementations of the assessment
contract when they do not need HTTP. Do not introduce a general trait-mocking
framework without a concrete need.

Test inbound Axum routes in-process using Tower's service testing utilities,
with injected workflow dependencies. Add those development dependencies when
the first route test uses them. Provider contract tests live in Cargo integration
tests; both test surfaces run through the existing Cargo test command.

## Required cases as contracts are implemented

- Valid responses and exact request/response mapping.
- Missing answer IDs, unknown labels, malformed JSON, and invalid probabilities
  or inconsistent distributions.
- Authentication errors, rate limits, and provider/server errors.
- Response size limits, deadlines, bounded retries, and cancellation behavior.
- Uncertain assessments versus technical failures, retaining distinct outcomes
  through workflow and HTTP mappings.

Specify accepted status codes, retry eligibility, timeout semantics, and output
shapes before writing each adapter. Tests assert those decisions, not merely
that a mocked request succeeds. Avoid timing-sensitive sleeps; use bounded test
deadlines and controlled delayed responses only for timeout scenarios.

## Alternatives and limitations

Trait-only mocks are faster but do not exercise the wire contract. Live services
introduce cost, secrets, nondeterminism, and availability dependencies; keep live
checks separately authorized and out of ordinary tests.

Mock fixtures encode our understanding of a contract, not proof that a vendor
currently honors it. Derive sanitized fixtures from documented schemas and
record their provenance alongside them. Version changes require contract review.
Do not create held-out evaluation cases as integration fixtures.

## Verification and follow-up

The user requested this testing approach on 2026-10-05. The framework is selected,
but no provider adapter or HTTP route exists yet, so no mock tests or development
dependencies are installed at this milestone.
Implement the first inbound integration tests with the API slice and provider
mock tests with each adapter, before accepting those respective milestones.
Independent review and CI must execute them with
`cargo test --workspace --locked`.

Implementation update 2026-10-05: the health API now has four in-process
Axum/Tower integration tests. Provider adapters and wiremock tests remain pending.
