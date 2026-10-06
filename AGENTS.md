# Harness working agreement

Standard version reflected here: **0.4.0**.

## Read before implementation or review

Read [engineering-standards.md](engineering-standards.md) in full, compare its
version with this marker, then read the [ADR index](docs/adr/README.md) and relevant
accepted ADRs. Resolve mismatches/conflicts; follow all current requirements.
After context compaction, reread required guidance before continuing.
Links do not automatically load their targets.

## Load only task-relevant context

| Task | Read |
| --- | --- |
| Scope or workflow | [Product scope](docs/product-scope.md) |
| Provider integration | [Jev capabilities](docs/jev-capabilities.md) |
| Questions, thresholds, datasets, results | [Metrics](docs/metrics.md), [evaluation design](docs/evaluation-design.md) |
| Agents, skills, tools | Relevant ADRs and actual profiles/configuration/runbooks |
| Local environment | [Development setup](docs/development.md) |
| Mistakes or stale guidance | [Engineering learning loop](docs/engineering-maintenance.md) and maintainer profile |

Requirements may evolve; follow the standard's
[change discipline](engineering-standards.md#change-discipline), not historical
scope as a permanent restriction. Do not load every document/reference/skill.
Do not inspect held-out cases while tuning implementation or prompts.

## Execute, verify, stop

State acceptance criteria; implement one coherent slice; validate the exact
candidate using actual executable checks and verified runbooks. Review against
applicable standard requirements and report candidate/version, evidence,
exceptions, and missing checks. Update affected docs and ADRs.
Pause for user discussion at the agreed milestone.
If no milestone is agreed, finish the requested coherent task and stop; do not
infer permission to begin another task, merge, or deploy. Follow the standard's
[delivery policies](engineering-standards.md#delivery) for commits, PRs, and releases.

The maintainer profile and learning skill exist; automatic hook integration is
pending. Read the learning-loop guide for the intended contract and limitations. Native agent
discovery and live CLI analysis are not yet verified. The minimal Cargo scaffold
and direct-Cargo hook configuration exist; use docs/development.md for commands.
The local health API and in-process contract tests now exist; no claim workflow
or provider adapters exist yet.
The [candidate-review runbook](docs/candidate-review.md), pre-committer profile,
and Rust CI workflow are configured. Hosted CI passed for quality-gate commit
e2278bf; each changed candidate needs its own checks. Native profile discovery
remains unverified; branch protection is not configured by these files.
Update this setup note when controls exist.
Documentation-only changes need no application tests unless documentation tests exist.
