# ADR 0009: Repeat Cargo gates in CI and isolated review

Date: 2026-10-05
Status: Accepted

## Context and decision

The user authorized the quality-gate milestone before application behavior.
Keep fast formatting, Clippy, and test checks in the existing local hook.
Repeat them in GitHub Actions and explicit separate-context candidate review.
Add cargo-audit 0.22.2 to CI/review for lockfile vulnerability advisories;
warnings fail and no advisory exceptions are configured.

Use an index tree exported with Git archive for review. The reviewer receives
tree/baseline identity, diff, acceptance criteria, instructions, and approved
commands; it executes checks without modifying source or delivering changes.
See the [runbook](../candidate-review.md) for reproducible commands and limitations.
Native custom-agent discovery is not assumed; an explicitly loaded profile in a
fresh agent context is a disclosed fallback.

## Alternatives and consequences

Testing the mutable checkout would include unstaged inputs and weaken candidate
identity. A new worktree could also isolate a committed candidate but is not
required for staged review. Neither approach is a security sandbox.

Keep AI review explicit at milestones/PRs, not in every deterministic Git hook.
Automatic learning analysis is separate and still pending. Review findings do
not authorize adoption, merge, deployment, or weakened checks.

CI uses a pinned checkout action, read-only repository permissions, no persisted
credentials, no provider secrets, and bounded run time. No cache or external
Rust setup action is necessary at this scale. The repository toolchain file
selects Rust and components in both environments.

The advisory database evolves, so audit results are time-specific. Network or
tool installation failures fail the check rather than return a clean report.
Cargo-audit does not replace dependency provenance/license review.
Broader dependency policy is deferred until dependencies are introduced and
reviewed. Branch protection is an administrative control, not implemented by
the workflow.

## Verification and follow-up

Acceptance requires valid workflow syntax, passing Cargo/advisory checks on the
exact exported candidate, and actual execution by a separate-context reviewer.
Record the tree and applied standard version in the review report.
Hosted CI must also be observed on a PR before accepting the first behavioral
slice. File creation alone is not evidence of a passing hosted run.
