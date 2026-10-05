# Engineering learning loop

Status: integration pending. The user approved automatic analysis that generates
a separate proposed patch, with human review before adoption. No active hook or
Rust orchestration tool implements AI analysis yet. Direct Cargo hooks are
configured separately; see [development setup](development.md).

## Implementation boundary

Use the language-agnostic pre-commit manager for YAML configuration and Git hook
registration. Declare Rust formatting, linting, and test commands directly in
that configuration once the Cargo project exists. Any custom orchestration must
be Rust, not a Python runner or a second application toolchain.

The temporary Python implementation, tests, and dependency manifest were removed.
The new YAML configuration invokes Cargo directly with no custom checker.

## Required behavior

- Analyze the exact staged candidate and supplied real check/review evidence.
- Invoke a fresh harness context with the focused learning skill, bounded
  permissions, timeout, and explicit cost controls.
- Produce a report and optional proposed patch outside tracked source.
- Never apply, stage, commit, or approve generated instructions automatically.
- Preserve candidate identity and revalidate after accepted changes.
- Distinguish missing rules, stale guidance, missing enforcement, and noncompliance.
  Prefer regression tests over adding prompt text.

The [learning skill](../.github/skills/engineering-learning/SKILL.md) and
[maintainer profile](../.github/agents/engineering-maintainer.agent.md) describe
the proposal contract. Native discovery and live execution remain unverified.
They do not access session history automatically; redacted selected evidence must
be supplied explicitly.

## Adoption

Human review decides whether a proposal is useful. The harness applies accepted
changes as a separate candidate, synchronizes standard versions and affected ADRs,
and reruns applicable checks. Later reviews assess whether the correction helped.

Git hooks remain bypassable. Rust CI and explicit separate-context pre-committer
review are configured in the [candidate-review runbook](candidate-review.md);
they do not automate learning analysis. No hosted AI or Codex hook exists.
