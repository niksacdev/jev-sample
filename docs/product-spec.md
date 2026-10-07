# Claim of Thrones: product requirements

**Status:** proposed; pending human product, claims, customer/recourse, finance
and technical-risk signoff. Purpose: correct, accountable resolution with less
avoidable claimant and handling effort.
[Scope](product-scope.md) fixes boundaries; [jobs/scenarios](persona-journeys.md)
supply acceptance cases; [metrics](metrics.md) define measurement.
This specification does not approve real decisions/payments or prescribe APIs,
services or agent topology.

## Outcomes and operating model

The proposed first exercise is a synthetic motor-claims reference journey.
Success means independently supported outcomes and effective recourse, not
denials, accepted suggestions, fewer necessary interventions or raw closures.
M02 verified resolutions per total handling hour is gated by M01 material decision
error rate, M06 fulfilment, M07 recourse and safety. M03/M04 protect claimant
time/effort; M05-M09 protect escalation, reliability and bounded autonomy;
M10/M11 measure insurer value; M12-M14 measure vendor viability and commercial fit.

Agents prepare information and coordinate bounded work. Consequential judgement
requires verified insurer authority. Evidence, coverage/loss analysis, proposal,
insurer decision, claimant response and fulfilment confirmation remain distinct.
A conversational statement cannot establish receipt, ownership, payment or closure.

## Requirements and acceptance

Owners below are accountable roles, not assigned people. Assign teams and
obligations before a pilot. "Record" means recoverable, reviewable target
behaviour; the process-local sample does not implement durable claims.
Acceptance uses synthetic fixtures unless a separately approved pilot permits
real data.

### R01 - Usable, nonduplicated report

**Input/output:** entitled claimant/session, supplied loss facts and submission
reference -> acknowledged report with stable reference, preserved facts and next
owner; no coverage promise.
**Exception owner:** intake/verification handles invalid identity/input, failed
recording and uncertain duplicates. Failed write is not receipt.
**Acceptance:** S01/S03/S09 retain useful partial facts, distinguish other from
unclear, recover recorded receipt after reconnect and return prior result for
exact retry without creating another claim.
**Trace:** J-C1, J-O1; M03, M04, M08.

### R02 - Evidence without fabricated completeness

**Input/output:** facts, reviewed evidence needs and permitted source revisions
-> inventory of provenance, conflicts, missing items and purposeful requests.
**Exception owner:** handler records inaccessible/unsafe/contradictory material
and next inquiry; claimant inability is not invented evidence.
**Acceptance:** S02/S10 retain original sources and corrections, explain rejected
attachments, identify repeated requests and reassess affected analysis.
**Trace:** J-C1, J-H1; M01, M04, M05, M08.

### R03 - Reviewable coverage basis

**Input/output:** applicable policy/endorsement revision, loss and evidence ->
sourced analysis and authorized determination or unresolved question.
**Exception owner:** authorized coverage reviewer handles missing/conflicting
policy, uncertain applicability and disputes. Retrieval failure is not denial.
**Acceptance:** S04/S06 expose basis/uncertainty, independently verify authority
and retain disagreement. Without real policy/jurisdiction, analysis is synthetic.
**Trace:** J-C2, J-C3, J-H2, J-T1; M01, M05, M07.

### R04 - Supported loss assessment

**Input/output:** current valuation evidence, approved method and coverage
context -> traceable assessment, exact amounts/currency, assumptions and gaps.
**Exception owner:** authorized assessor/handler corrects unsupported valuation,
contradictions or calculation errors; model confidence cannot determine money.
**Acceptance:** S01/S05 expose calculation basis, investigate challenges and
recompute after material changes without overwriting prior assessment.
**Trace:** J-C2, J-H1, J-H2; M01, M04, M05.

### R05 - Proposal and decision bound to actual authority

**Input/output:** current coverage/loss basis and verified delegation ->
versioned payment/service/no-payment proposal and separate authorized decision.
**Exception owner:** claims-authority owner resolves absent/expired/insufficient
delegation; employee title, confidence and dashboard access cannot substitute.
**Acceptance:** S01/S04-S06 block consequence without authority; material changes
require fresh affected approvals; authorized terms remain inspectable.
**Trace:** J-H2, J-T1; M01, M05, M06, M09.

### R06 - Informed claimant response

**Input/output:** authorized proposal/basis and applicable response rules ->
accessible explanation, actual status, accept/reject/question/review choices
and revision-bound response where required.
**Exception owner:** handler/customer service assists with disputes,
inaccessibility, missing estimate or nonresponse; silence is not acceptance.
**Acceptance:** S04-S06/S11 distinguish insurer approval from claimant response;
revised offers cannot reuse acceptance; no-payment acknowledgement does not waive
review; status does not invent timing.
**Trace:** J-C2, J-C3; M01, M03, M04, M07.

### R07 - Confirm fulfilment and reconcile uncertainty

**Input/output:** current authorized terms, required response and verified
destination -> execution identity, pending/unknown/failed/confirmed result and
supported receipt.
**Exception owner:** fulfilment/reconciliation investigates unknown delivery
before resend and contains wrong amount/recipient or duplicate execution.
**Acceptance:** S07/S08 require confirmation, not sent instructions; retries
cannot duplicate effects; adjustments need fresh authority; unknown required
fulfilment blocks closure.
**Trace:** J-C2, J-O2, J-T1; M06, M08.

### R08 - Owned exceptions and recovery

**Input/output:** blocked work, specialist/urgent cues or dependency failure ->
reason, accountable owner, next action, acknowledgement and resume condition.
**Exception owner:** operations retains responsibility when receiving team is
unavailable or retries/assignment stall.
**Acceptance:** S02/S03/S07/S10 retain failures after recovery, preserve gates,
bound retries and never claim dispatch or completed unacknowledged handoff.
**Trace:** J-H1, J-O1, J-O2; M03, M05, M08.

### R09 - Revisions, conflict and restart

**Input/output:** claim/action reference, recorded revisions and update ->
recoverable current state/history or explicit reconciliation conflict.
**Exception owner:** case owner with technical support handles stale edits,
ambiguous duplicates, failed writes and interruption.
**Acceptance:** S05/S08-S10 preserve acknowledged state and prior decisions,
invalidate affected approvals/responses after material changes and prevent
silent overwrites, unsafe merges and duplicate financial effects.
**Trace:** J-C1, J-H2, J-O2, J-T2; M01, M06, M08.

### R10 - Evidence-based closure and effective review

**Input/output:** authorized decision, required response and correct confirmed
fulfilment or authorized no-payment basis -> explained closure and review route.
**Exception owner:** fulfilment/review owner handles missing confirmation,
challenge or new evidence; review does not require agreeing with the outcome.
**Acceptance:** S06-S08 acknowledge review before/after closure, retain original
decision and receipt, apply independence rules and record reassessment without
promising reversal or erasing financial history.
**Trace:** J-C3, J-H2, J-O1; M01, M06, M07.

### R11 - Inspectable, scoped operation

**Input/output:** authenticated scope, permitted metadata, versions and
observation window -> appropriate status/lineage/workload with freshness/gaps.
**Exception owner:** technical/risk and operations contain access/authority
breaches and arrange recovery for missing monitoring.
**Acceptance:** S10/S11 prevent cross-case disclosure and unauthorized controls;
counts reconcile to declared population; missing data is not zero; lineage is
observable evidence, not hidden reasoning. Monitoring grants no decision rights.
**Trace:** J-O1, J-T1, J-T2; M01, M08, M09.

### R12 - Quality and total-job improvement

**Input/output:** predeclared cohorts/scenarios, independent rubric, manual
comparator and measured effort/waits -> M01-M09 findings with counts/uncertainty.
**Exception owner:** claims-quality/product records insufficient evidence or
regression and revises/stops the exercise, not definitions after seeing results.
**Acceptance:** S12 includes adverse branches and all cohort effort, including
unresolved/failed/in-progress work, review, correction, oversight and recovery.
Later corrections remain visible; sampled quality is not population truth.
**Trace:** J-O2, J-B1, J-T2; M01-M09.

### R13 - Insurer value without double-counting

**Input/output:** matched workload, approved cost boundary, retained labour,
fees, implementation spend and attributable cash changes -> M10/M11 analysis
separating cash, capacity and service value.
**Exception owner:** insurer finance withholds ROI conclusions for unknown
inputs, unsupported attribution or inconsistent horizon/allocation.
**Acceptance:** S12 excludes appropriate indemnity as savings, includes relevant
support/rework, does not count avoided rework twice and never calls saved minutes
alone realized cash. Cohort maturity and actuarial limits remain explicit.
**Trace:** J-B1; M01, M02, M10, M11.

### R14 - Sustainable vendor delivery economics

**Input/output:** contract/price scenario, recognized revenue policy, delivery,
support, acquisition and onboarding costs -> M12/M13 margin/contribution/payback.
**Exception owner:** vendor finance labels unknown/undefined/not-reached results
for unselected pricing, zero revenue or nonpositive contribution.
**Acceptance:** S12 includes inference/retries, tools, hosting, vendor-paid human
support and incidents; separates bookings/cash/revenue and development/sales
from delivery cost; reconciles revenue basis across M12/M13. Insurer fee/vendor
revenue is a transfer, not twice the shared benefit.
**Trace:** J-B2; M12, M13.

### R15 - Commercial fit and governed expansion

**Input/output:** sponsor/payer, researched alternatives, scoped pilot,
candidate pricing and quality/value evidence -> willingness-to-pay evidence
and continue/revise/stop decision.
**Exception owner:** product/commercial obtains claims/finance/risk review for
absent value, harmful pricing incentives or expansion.
**Acceptance:** S12 separates interest from paid adoption/renewal, does not infer
retention from a demo, defines reversals/disputes for outcome pricing and does
not approve other lines/general servicing by implication.
**Trace:** J-B1, J-B2; M01, M10-M14.

## Validation and release decisions

Human representatives walk S01-S12 and approve unresolved assumptions before
implementation commitments. Deterministic calculations, source fixtures and
mocked failures verify requirements, not live-provider or real claims quality.
Matched manual/assisted exercises require approved eligibility, rubric,
measurement, sample and decision criteria.

Evaluate provider components separately. Code/Jev intent comparison cannot
establish coverage accuracy, payment quality or claims throughput.
Preserve the [approved intake benchmark](evaluation-design.md#approved-intake-component-scorecard)
without applying its targets to M01-M14. Assign metric owners, maturity rules,
tolerances and breach responses before a pilot; holistic numerical targets remain
unset. Material authority/privacy/fulfilment breaches require containment and
human review; insufficient evidence is not success.

Derive architecture from jobs and requirements, not mockup controls. Keep
implementation choices in [system design](system-design.md) and consequential
decisions in ADRs. Track product/architecture feedback through linked issues/PRs.
