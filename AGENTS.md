# Harness working agreement

## Required context

Before implementation or review, read all of
[engineering-standards.md](engineering-standards.md), then the
[ADR index](docs/adr/README.md) and ADRs relevant to the change.
Treat ES-01 through ES-16 as mandatory. Do not rely on a remembered summary.
After context compaction, reread these before continuing implementation.

This repository is Rust-first. Read docs/product-scope.md for current product
boundaries; those are evolving requirements, not permanent engineering rules.
For requested scope changes, follow ES-03 and update affected decisions/documents.
The harness implements; skills provide procedures in its context; pre-committer
is an independent test/review context. Do not create an implementation-owner agent.

## Load only the supporting context needed

| Change | Read |
| --- | --- |
| Scope or user workflow | docs/product-scope.md |
| Rust design or implementation | docs/rust-practices.md |
| Provider integration | docs/jev-capabilities.md |
| Model questions, thresholds, datasets, results | docs/metrics.md and docs/evaluation-design.md |
| Agents, skills, review workflow | docs/ai-engineering-practices.md |
| Tool selection/setup | docs/tooling-options.md; distinguish candidates from accepted ADRs |

Do not load every supporting document, external reference, or skill by default.
Links do not guarantee content was loaded: read applicable files explicitly.
Preserve locked evaluation isolation; do not inspect held-out cases while tuning.

## Before accepting a change

State acceptance criteria and affected ES rule IDs; implement one coherent slice.
Run verified applicable checks on the actual candidate and report evidence,
exceptions, and unavailable checks. Update affected docs and ADRs. Pause for user
discussion at the milestone; do not silently advance to the next one.

There is no Cargo project, verified Rust command runbook, configured pre-committer,
or CI yet. Do not claim these checks ran or invent setup commands. Once they
exist, use executable configuration and verified runbooks as command authority.
Documentation-only changes need no application tests unless documentation tests exist.
