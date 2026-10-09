# ADR 0007: Share one workflow between HTTP and evaluation

Date: 2026-10-05
Status: Accepted

## Context

The sample compares deterministic rules, Jev, and a structured-output LLM on
synthetic insurance intake. HTTP behavior and offline evaluation must use the
same validation and routing policy. The user is learning Rust through small
milestones; separate services or crates would add unnecessary setup now.

## Decision

Use one Cargo package with a library and two binaries: an HTTP server built with
Axum/Tokio and an evaluation CLI. Both call the library's shared workflow.
The user approved this architecture on 2026-10-05.

Planned responsibilities, introduced only as callers need them:

| Surface | Responsibility |
| --- | --- |
| Domain and routing | Validated types, explicit outcomes, pure versioned policy |
| Workflow | Coordinate assessment and routing; preserve technical failures separately from uncertainty |
| Provider adapters | Rules, Jev, and LLM assessments; validate external DTOs into domain types |
| HTTP transport | Request/response mapping, input limits, typed error mapping |
| Evaluation CLI | Dataset orchestration and measurements using the same workflow and policy |

Dependencies flow inward: transports call workflow; workflow uses domain types
and a narrow assessment contract; adapters implement that contract. Domain code
does not depend on Axum, Tokio, provider DTOs, environment, or network access.
HTTP handlers and the evaluation runner must not duplicate routing decisions.

Axum provides typed handlers and testable routing; Tokio supplies the async
runtime for HTTP and provider I/O. Add these dependencies at the first API slice,
not in this documentation milestone. Select the provider HTTP client and CLI
parsing tools when their concrete requirements are known.

No persistence, microservices, caching, frontend, or multi-crate workspace in
this starting architecture. New requirements can change it through a new ADR.

## Alternatives

- CLI first: simpler initially, but defers the approved HTTP learning slice.
- Multiple crates: stronger package boundaries, but unnecessary complexity until
  actual coupling or reuse justifies them.
- Separate evaluation logic: rejected because it risks measuring a different
  routing system from the HTTP application.

## Consequences and verification

Module boundaries require review and tests; one package does not enforce them
automatically. Provider differences must not disappear behind the contract:
retain model/rubric/policy provenance and provider-specific uncertainty evidence.
Detailed assessment schemas and failure mapping remain implementation decisions
to resolve before the relevant slice.

Next configure deterministic CI and independent candidate review before the
first behavioral slice. That slice will be a minimal health endpoint with focused
HTTP tests, explicit health semantics, and runnable Rust explanations, not model
integration. Ordinary tests remain synthetic/mocked and require no paid inference.

Acceptance for this milestone is an approved decision, recorded boundaries, and
an updated task list. No runtime architecture has been implemented yet.

Implementation update 2026-10-05: CI and isolated review were configured in
[ADR 0009](0009-ci-and-independent-candidate-review.md) and exercised on e2278bf.
The next candidate implements the local Axum health router/API binary and four
in-process integration tests. The original future-work statements above describe
the architecture-approval milestone, not the current implementation status.
Domain/routing, provider adapters, and the evaluation CLI remain unimplemented.

Implementation evidence 2026-10-07: candidate
`c55cbe9b7fe127acfbcf7d861435c82cecb9970f` contains a shared servicing application,
pure routing and injected Code/Jev assessors. Its binaries are `api` and
`export-contracts`; the evaluation CLI remains unimplemented. ADRs 0010,
0011 and 0013 describe this review-only servicing slice and provider comparison.
It is not an implementation of the approved intake benchmark. Retain the
decision that a future evaluator calls the shared workflow rather than
duplicating routing; resolve benchmark-contract applicability before execution.

Implementation evidence 2026-10-09: `main` adds the AI-native workflow (OpenAI
structured-output planner with Jev/OpenAI/code decision providers, ADRs 0016-0019)
beside the Assessment lab servicing slice. Binaries remain `api` and
`export-contracts`; the evaluation CLI remains unimplemented and the intake
benchmark unexecuted. See [metrics](../metrics.md#assessment-lab-measurement-boundary).
