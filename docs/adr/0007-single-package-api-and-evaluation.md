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
