# Harness working agreement

Standard version reflected here: **0.3.0**.

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

No Cargo project, runbook, pre-committer profile, or CI exists yet. Do not claim
they ran or invent verified commands. Update this setup note when controls exist.
Documentation-only changes need no application tests unless documentation tests exist.
