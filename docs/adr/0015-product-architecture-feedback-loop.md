# ADR 0015: Connect product metrics and architecture through issues and PRs

Date: 2026-10-07
Status: Accepted
Tracking issue: [#7](https://github.com/niksacdev/jev-sample/issues/7)

## Context

The user requires a two-way feedback loop: metrics guide architecture, while
technical feasibility and architectural evidence can improve or challenge product
assumptions. Static metric documents without issue/PR traceability lose insight.
The approved intake scorecard does not cover the expanded servicing/claims vision,
and no measured business value has been established.

## Decision

Engineering standard 0.6.0 requires an issue-level value hypothesis, metric
relationship, measurement contract, and outcome-review ownership. PRs reconcile
product expectations with architectural evidence and distinguish implementation
verification from demonstrated product value.

Technical findings may propose new mechanisms, scope, or metrics. Explicit
user/product-owner approval is required before those proposals become approved
scorecard revisions. Preserve decision links and reevaluation requirements.
Use docs/metrics.md for definitions, ADRs for architecture rationale, and issues
for live evidence/progress, without parallel task lists.

## Alternatives

One-way compliance with frozen metrics hides useful feasibility feedback.
Informal target adjustments erase failed hypotheses and enable post-hoc success
claims. Requiring every maintenance task to demonstrate a direct product gain
encourages fabricated evidence; justified indirect/no impact is supported instead.

## Consequences

Issue and PR templates prompt the required review. Pending outcomes stay in owned,
triggered follow-up issues even if implementation is delivered. Product approval
is distinct from engineering checks and from permission to merge or deploy.
Existing targets remain unchanged; expanded-scope and commercial assumptions need
their own review. This adds no production instrumentation or automated merge gate.

## Verification and follow-up

Check standard/AGENTS version agreement and template coverage of both directions.
Review future issues/PRs for traceable hypotheses, decisions, and outcome evidence.
GitHub tracking is established after the earlier HTTP 500 failures were resolved.
Named metric owners and the expanded scorecard are not established by adopting
this procedure.
