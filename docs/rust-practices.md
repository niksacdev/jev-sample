# Rust implementation direction

Status: Rust-first direction approved on 2026-10-05. Framework choices below are
recommendations to confirm during the architecture milestone, not installed dependencies.

## Goal

Build the backend using established Rust frameworks and engineering practices.
No Python backend or Rust-versus-Python experiment is required. Do not claim
language superiority from this implementation.

Rust's compiler contributes type and memory safety. Architecture quality, domain
correctness, adequate tests, and reliable model behavior require separate controls.

## Recommended tooling

| Concern | Proposed framework or tool |
| --- | --- |
| HTTP API | Axum |
| Async runtime | Tokio |
| Typed JSON contracts | Serde |
| Outbound model API requests | Reqwest |
| Typed domain/library errors | thiserror |
| Structured diagnostics | tracing |
| Formatting and linting | cargo fmt and cargo clippy |
| Unit, integration, and compile-fail examples | Cargo test and Rust documentation tests |

Select versions and additional test helpers during implementation based on
compatible maintained releases. Add dependencies only when used.

## Engineering practices

- Separate domain judgments, routing policy, HTTP transport, and model adapters.
- Represent distinct domain concepts with types and handle outcomes exhaustively.
- Validate external JSON beyond deserialization: answer completeness, known
  categories, finite in-range probabilities, and consistent distributions.
- Convert model results into validated domain values before routing.
- Make errors explicit; never disguise invalid responses or timeouts as successful
  model decisions. Review routing must preserve the technical failure reason.
- Use bounded timeouts/retries and propagate cancellation appropriately.
- Keep financial arithmetic and date operations in deterministic code if needed.
- Keep secrets and sensitive content out of source code and telemetry.
- Cover routing boundaries, urgency/review precedence, ambiguity, provider failures,
  and malformed responses with behavioral tests.
- Keep ordinary tests deterministic and independent of paid model calls.
- Run formatting, Clippy, compilation, and tests locally and in CI.

The future coding-standards.md will define the enforceable details. A pre-committer
agent complements deterministic checks; it does not replace them or guarantee
design correctness.

## Learning and evidence

Deliver small milestones with explanations of Rust concepts, runnable commands,
expected results, and tests. Pause for discussion after each logical milestone.

Use tests and compiler examples to show concrete safeguards. Defects caught by
compilation, linting, tests, and review can inform improvements, but are not a
controlled language comparison. Neither line count nor coverage alone proves quality.

## Next milestone

Agree on framework choices, module boundaries, API contracts, and the first
learning slice before adding application code.
