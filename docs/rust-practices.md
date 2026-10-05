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

## Supporting toolchain

This is the proposed enforcement plan, not a claim that tools are installed or
checks already run. Configure it during the quality-gates milestone.

The complementary [AI-native workflow](ai-engineering-practices.md) covers
repository instructions, selective agents and skills, review, and debt controls.

| Layer | Tools and policy | What it supports |
| --- | --- | --- |
| Reproducible builds | rustup; exact stable version in rust-toolchain.toml; committed Cargo.lock; CI uses --locked | Consistent compiler, formatter, lints, and dependency resolution |
| Editor feedback | rust-analyzer with formatting and Clippy diagnostics | Fast type, ownership, and lint feedback while learning |
| Compilation | cargo check; forbid unsafe code in project crates | Type/ownership enforcement and explicit project safety policy |
| Formatting | cargo fmt --all -- --check | Consistent formatting without subjective review |
| Linting | cargo clippy --workspace --all-targets --all-features --locked -- -D warnings | Compiler-supported correctness and maintainability checks |
| Behavioral tests | cargo test --workspace --locked; documentation tests | Domain rules, API contracts, error paths, and executable examples |
| Boundary properties | proptest, where invariant testing adds value | Threshold boundaries, probability validation, and routing invariants |
| External service isolation | wiremock for HTTP adapter tests | Malformed responses, status codes, timeouts, retries, and no paid calls in ordinary tests |
| Dependency policy | cargo-deny with reviewed advisory, license, source, and duplicate-version rules | Supply-chain and licensing controls, not proof that dependencies are secure |
| Coverage diagnostics | cargo-llvm-cov when tests exist | Find untested branches; coverage is diagnostic, not a substitute for meaningful assertions |
| Design review | coding-standards.md, human review, pre-committer agent | Module boundaries, domain modeling, complexity, scope, and test adequacy |

Use the Clippy command above only while all feature combinations are compatible.
If mutually exclusive features are introduced, document and test a feature matrix
instead. Record each lint exception with a narrow scope and rationale. Do not
enable every pedantic lint blindly or impose arbitrary coverage percentages.

The unsafe-code restriction applies to our crates, not all transitive dependencies.
Any exception requires explicit review. Dependency policy must account for the
actual graph; do not globally suppress findings to obtain a green build.

## Where checks run

1. During editing, rust-analyzer supplies feedback; use focused Cargo tests for
   the behavior being changed.
2. Before a commit, a versioned Git hook invokes a deterministic check script.
   Start with formatting, Clippy, and tests. It must check the candidate commit,
   not accidentally certify unrelated unstaged changes; partial staging must be
   handled explicitly or rejected with an actionable message.
3. On pull requests, CI independently runs the same checks, dependency policy,
   and applicable property/contract tests. Required branch checks must be enabled
   separately before claiming merges are protected.
4. Coverage reports run in CI as diagnostics. Add performance benchmarks,
   fuzzing, or Miri only when concrete risks justify them; they are not default
   dependencies for the first API slice.

Git hooks can be bypassed, so CI is the authoritative automated gate.
Hosted CI and branch protection still need configuration; documenting them does
not enforce them. Any external AI review must respect repository access and
data-handling constraints.

## Pre-committer responsibilities

The pre-committer agent reviews the candidate changes against coding-standards.md,
reports actionable findings, and identifies checks it could not perform.
It must not silently rewrite files, approve its own changes, fabricate successful
checks, or hide failures.

Keep deterministic checks independent of AI availability. Prefer an explicit
agent review at milestone/PR boundaries over making every Git commit depend on
an AI service. Decide any Codex hook integration after verifying supported hook
events and failure behavior. Human review remains responsible for architectural
judgment; no toolchain enforces all design principles automatically.

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
