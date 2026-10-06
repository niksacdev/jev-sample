# Architecture decision records

ADRs preserve consequential project decisions for people and coding harnesses.
Read relevant accepted records before changing their affected behavior.

| ADR | Status | Decision |
| --- | --- | --- |
| [0001](0001-insurance-intake-scope.md) | Accepted | Synthetic insurance intake, not adjudication |
| [0002](0002-harness-skills-and-review-context.md) | Accepted | Harness implementation, same-context skills, separate-context pre-committer |
| [0003](0003-separate-principles-from-tools.md) | Superseded by 0004 | Initial separation of principles and tool candidates |
| [0004](0004-consolidate-engineering-guidance.md) | Accepted | One standard, one entry point, ADR memory, executable enforcement |
| [0005](0005-evidence-driven-instruction-maintenance.md) | Accepted | Explicit evidence-driven maintenance with human adoption |
| [0006](0006-direct-cargo-commit-checks.md) | Superseded by 0012 | Direct Cargo commit checks |
| [0007](0007-single-package-api-and-evaluation.md) | Accepted | One library workflow shared by Axum/Tokio HTTP and evaluation CLI |
| [0008](0008-mocked-provider-contract-tests.md) | Accepted | Wiremock provider contracts and in-process HTTP integration tests |
| [0009](0009-ci-and-independent-candidate-review.md) | Accepted | GitHub CI, advisory checks, and isolated separate-context review |
| [0010](0010-react-ui-and-rust-agent-boundary.md) | Accepted | React UI, Rust-owned servicing assessment and disposable local run inspection |
| [0011](0011-provider-port-and-servicing-use-case.md) | Accepted | Injected assessor port, shared servicing use case and validated versioned routing policy |
| [0012](0012-native-git-quality-hook.md) | Accepted | Repository-managed native Git hook invoking Cargo/npm |

Use sequential IDs and descriptive names. Include date, status, context, decision,
alternatives, consequences, and verification/follow-up. Statuses are Proposed,
Accepted, Superseded, or Rejected. Update the index in the same change.

Keep accepted rationale intact. A replacement needs a new record with reciprocal
links and a Superseded status on the old record. Dated corrections and
implementation evidence may update an existing record without rewriting history.
Decisions are not proof of implementation; document unresolved follow-up.
