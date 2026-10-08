# ZipClaim: product requirements

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

## Agent planning and decision checkpoints

### Experimental setup acceptance

The preview collects missing OpenAI and Jev connections with masked API-key
fields and a masked **ZipClaim token**. An explicit **OpenAI model name**
textbox remains available; no model identifier is guessed. **Agree and save**
incorporates the session Data Protection acknowledgement and automatically
refreshes setup availability. **Submit a Claim** stays disabled until both
connections and acknowledgement are ready. Setup/refresh failures preserve the
draft and provide recovery without inference or automatic claim submission.
Credentials remain in API memory until restart, never browser storage or
workflow records. Configuration readiness does not verify vendor account access.

The conversational LLM interprets the customer request and produces a versioned
base plan with tasks, dependencies, required context and expected outputs.
The agent can revise that plan after tool results, decisions or human responses.
This is not a fixed intent-to-task script. Task execution uses deterministic
tools where facts, arithmetic or state checks suffice; judgment tasks invoke a
replaceable decision provider against bounded, typed questions.

Decision requests identify their question/rubric version, permitted outcomes and
exact evidence snapshot. Results preserve predicate probabilities, choice/score
distributions and provider confidence semantics; confidence is not authority or
proof of correctness. Jev Noul returns probability of yes, not a separate
confidence value. A confident no must not be treated as uncertain. OpenAI's
Decisions API is a separate capability from the Responses planning/chat API.
Refusal, unsupported question, missing evidence and technical failure remain
distinct. Validated decision-specific policy selects continuation, clarification
or employee review; mandatory authority gates remain outside the model.

Customer clarification supplies missing facts. Employee intervention resolves
judgment or authority questions and requires authenticated action. Both responses
are bound to the paused run/task and revision; stale responses cannot resume a
different plan. The runtime records who/what supplied evidence and prevents
duplicate task effects across repeated requests or restarts.

### Provider-comparison acceptance

A comparison has one query and one frozen base plan, with separately identified
runs for selected decision providers. A matched-decision comparison evaluates
the same question and evidence snapshot against each provider. Independent
continuations may diverge after those decisions; they are not matched inputs
merely because they started with the same customer message.

The dashboard must distinguish matched questions from divergent flow outcomes.
Each event identifies comparison/run/task/decision/attempt, sequence, timestamp,
provider/model and relevant question, plan and policy versions. Record plan
creation/revision, tool and decision starts/results/failures, pauses, resumptions
and terminal outcomes. Missing usage/cost is unknown, not zero. Durable capture
and controlled inspection are required for replay after restart.

Compare latency, usage, failure/refusal, answer completeness and workflow outcome
separately. Provider agreement measures consistency, not accuracy. Accuracy needs
independent versioned labels and adjudication; human handling effort, customer
outcomes and finance records remain additional inputs to M01-M14.
Synthetic tool completion does not establish an insurer outcome.

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

## Measurement is part of feature acceptance

R01-R11 must produce the business records needed by the
[M01-M14 measurement delivery contract](metrics.md#measurement-delivery-contract).
R12-R15 must join those facts with independent quality, human-effort, finance
and commercial evidence; application logs alone cannot establish these outcomes.
Missing evidence is an explicit result, not a default zero.

Before implementing a slice, specify its event/record schema, exact collection
boundary, source owner, calculation and expected-result fixtures. Acceptance
includes durable capture with business state, safe replay/deduplication,
late-arrival/correction handling, source reconciliation and controlled access.
Results expose calculation version, cohort/cutoff, coverage and freshness.
Attempt IDs are not claim IDs; elapsed workflow time is not staff effort;
instructions are not confirmed fulfilment.

R12 acceptance includes all M01-M09 fixture cases in the measurement contract;
R13 includes M10/M11 cost/cash reconciliation; R14 includes M12/M13 usage,
revenue and payback reconciliation; R15 includes M14 contract/cohort checks.
Until required independent sources are available, reports show unavailable,
estimated or scenario status and an owned evidence gap. Passing synthetic
calculation tests does not authorize measured-value or ROI claims.

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
