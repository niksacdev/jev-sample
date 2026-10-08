# Engineering learning loop

Status: integration pending. The user approved automatic analysis that generates
a separate proposed patch, with human review before adoption. No active hook or
Rust orchestration tool implements AI analysis yet. Direct Cargo hooks are
configured separately; see [development setup](development.md).

## Implementation boundary

Use the repository-managed native Git hook for direct Cargo and npm checks; see
[ADR 0012](adr/0012-native-git-quality-hook.md).
The small Bash hook only selects and executes existing quality commands.
Any future application-level AI orchestration must be Rust, not another
application toolchain. No hook currently invokes an AI agent.

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

## Accepted application lesson: guided recovery (2026-10-08)

The user rejected missing-configuration messaging that disabled submission and
sent customers to Geek Mode with no setup interaction. They explicitly requested
graceful recovery and memory-only credential setup. Assessment: missing UI
recovery behavior/enforcement, not a reason to weaken typed failures or authority.
[ADR0019](adr/0019-guided-memory-only-planner-setup.md) records the bounded remedy;
AGENTS contains scoped guidance accepted by this user direction. Rust and
TypeScript regressions exercise setup, failure categories, draft preservation,
secret isolation and explicit retry. No standard amendment or automated learning
hook is claimed. Owner niksacdev assesses the lesson at the first setup walkthrough.
