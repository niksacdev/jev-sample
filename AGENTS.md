# Harness working agreement

Engineering standard version reflected here: **0.2.0**.
Read the [version and change-control policy](engineering-standards.md#standard-version-and-change-control).
Compare this version with the current standard before implementation; resolve
any mismatch. This is a synchronization marker, not a frozen ruleset.

## Required context

Before implementation or review, read all of
[engineering-standards.md](engineering-standards.md), then the
[ADR index](docs/adr/README.md) and ADRs relevant to the change.
Follow all mandatory requirements in the current standard, including additions
and revisions. Do not rely on a remembered summary or a fixed list of rule IDs.
After context compaction, reread these before continuing implementation.

This repository is Rust-first. Read docs/product-scope.md for current product
boundaries; those are evolving requirements, not permanent engineering rules.
For requested scope changes, follow
[Decisions and scope](engineering-standards.md#1-decisions-and-scope)
and update affected decisions/documents.
The harness implements; skills provide procedures in its context; pre-committer
independently tests/reviews candidate evidence. All subagents have their own
context and a bounded task packet; isolation alone does not prove review quality.
Do not create an implementation-owner agent.

## Load only the supporting context needed

| Change | Read |
| --- | --- |
| Scope or user workflow | docs/product-scope.md |
| Provider integration | docs/jev-capabilities.md |
| Model questions, thresholds, datasets, results | docs/metrics.md and docs/evaluation-design.md |
| Agents, skills, review workflow | Relevant ADRs and actual agent/skill configuration when present |
| Tool selection/setup | Relevant ADRs, executable configuration, and verified runbooks when present |

Do not load every supporting document, external reference, or skill by default.
Links do not guarantee content was loaded: read applicable files explicitly.
Preserve locked evaluation isolation; do not inspect held-out cases while tuning.

## Before accepting a change

State acceptance criteria, the standard version, and applicable obligations;
implement one coherent slice.
Run verified applicable checks on the actual candidate and report evidence,
exceptions, and unavailable checks. Update affected docs and ADRs. Pause for user
discussion at the milestone; do not silently advance to the next one.

There is no Cargo project, verified Rust command runbook, configured pre-committer,
or CI yet. Do not claim these checks ran or invent setup commands. Once they
exist, use executable configuration and verified runbooks as command authority.
Documentation-only changes need no application tests unless documentation tests exist.
