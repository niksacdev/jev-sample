# Harness working agreement

Standard version reflected here: **0.6.0**.

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

Create or identify the tracking GitHub issue before PR-bound implementation.
Use its acceptance criteria and keep progress there; link it in the PR body.
Follow the canonical [issue policy](engineering-standards.md#delivery).
Do not use docs/task-list.md as the active backlog.
Connect the issue's value hypothesis and metrics to architecture tradeoffs;
record technical feedback, deviations, and outcome-review ownership using the
[product/architecture feedback loop](docs/metrics.md#productarchitecture-feedback-loop).
Do not equate passing tests or a merged PR with demonstrated product value.

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
and native Git Cargo/npm hook exist; use docs/development.md for commands.
The local health API, servicing coordinator, provider-comparison UI, Jev adapter,
operator shared-key inspection and mocked contract tests exist.
See README for runnable commands and limitations.
No consequential claim workflow, MCP, production identity or durable storage exists yet.
The [candidate-review runbook](docs/candidate-review.md), pre-committer profile,
and Rust CI workflow are configured. Hosted CI passed for quality-gate commit
e2278bf; each changed candidate needs its own checks. Native profile discovery
remains unverified; branch protection is not configured by these files.
Update this setup note when controls exist.
Documentation-only changes need no application tests unless documentation tests exist.
