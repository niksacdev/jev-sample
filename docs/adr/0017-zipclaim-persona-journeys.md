# ADR 0017: ZipClaim identity and persona-aligned AI-native journeys

Date: 2026-10-08
Status: Accepted
Tracking issue: [17](https://github.com/niksacdev/zipclaim/issues/17)

## Context

The user requested the supplied ZipClaim logo/design language, white background,
complete product renaming, and methodical holistic integration of ADR0016.
Hiding the AI-native runtime behind a separate technical tab left the primary
customer experience on the earlier classifier. Combining all protected evidence
and intervention in a customer console obscured role responsibilities.
This enabling work does not establish an M01-M14 improvement or change targets.

## Decision

Use original cropped ZC artwork, tokenized orange/pink/violet branding, readable
dark ink and white default surfaces. Optional dark mode remains explicit.
The brand tagline is aspirational; preview notices prohibit real customer data
and disclose absent insurer execution and unverified live-provider behavior.
Retain Rust crate/module and REASSURE_* configuration names for compatibility;
product surfaces, package identity and telemetry instrumentation use ZipClaim.

Customer defaults to message/consent, base plan, progress and reply. Employee
shows authenticated paused journeys with relevant decision records and
revision-bound review or clarification. Operator shows authenticated matched
provider evidence, immutable events and explicit unknown quality. Assessment lab
preserves prior APIs/experiments and labelled collapsible legacy inspection.

Reuse one typed persona-aware WorkflowConsole with a public journey state and
separate protected state. Role changes clear credentials/inspection/review drafts
and invalidate late responses. Resume updates public state only for the same
comparison. Public journey highlights use actual observed stages, not fabricated
success. React tabs do not authorize: Rust retains the shared-key check for both
resume types, task/revision validation, policy ownership and bounded execution.

## Alternatives and consequences

A replacement mockup would disconnect the UI from durable contracts. Duplicated
persona state machines would risk drift in locking, idempotency and revision
checks. Keeping the technical console primary would preserve the architecture
mismatch. Shared code preserves one implementation but still requires explicit
visibility and stale-response tests. This is not a production role system.

Raster crops faithfully preserve the supplied artwork but cannot provide
vector-quality scaling. A future supplied vector source can replace assets.
No extra dependencies or microservices are needed. External models remain
disabled without explicit configuration; offline tests cannot prove live account
capability, real claim correctness or economic value.

## Verification and follow-up

Frontend tests cover primary navigation, unavailable planning, consent,
paused-journey filtering, scoped resume, retry identity and lock/role-switch
stale-response protection. Browser checks cover the actual Rust-backed legacy
lab and unconfigured primary workflow without vendor calls, plus mobile overflow.
Delivery evidence and task progress belong on issue17, not a second backlog.
niksacdev owns the first synthetic persona walkthrough and subsequent usability
review; independent labels, operational effort and finance evidence remain #16/#15
follow-up before metric claims. New integration needs new candidate review;
the prior exact-tree runtime review does not certify this changed tree.

### 2026-10-08 refinement

[ADR0018](0018-live-instrumentation-and-session-preview-agreement.md) adds the
requested live terminal component dependencies, compact neon Geek Mode and
browser-tab agreement. The original no-extra-dependencies choice above describes
the initial persona redesign, not the subsequent live instrumentation requirement.
