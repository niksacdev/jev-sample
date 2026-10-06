# Reassure: persona journey specifications

Date: 2026-10-05. Status: design-review baseline, not production authority.
The user accepted the three persona screens as a starting point.
These requirements feed the [product spec](product-spec.md),
[system design](system-design.md), contract modeling and acceptance tests.
The [mockup](design/intake-workbench.html) is scripted; backend remains health-only.

## Shared journey rules

Autonomous agents progress permitted work; humans intervene when an explicit
authority, evidence, safety, operational or customer-response gate blocks it.
Routine observation is not an approval. A pause has a reason, owner, required
action, source/proposal revision and permitted resume condition.
Timeout or unavailable dependency must not become invented evidence or success.

Customer, employee and operator are distinct authenticated roles in production.
The mockup's persona switch does not enforce access. Customers see their claim
and safe explanations; employees see assigned/authorized cases and applicable
guidelines; operators see authorized operational metadata, not unrestricted
claim narratives or employee financial authority.

Every material transition records initiating/executing identity, delegation,
input revisions, artifact references, action/tool, model/policy/guideline versions,
time and outcome. This is decision lineage, not hidden model reasoning.
Corrections append revisions rather than erase history.
Confirmed external outcomes, not agent prose, establish payment or service completion.

## Customer: one conversation, accountable resolution

**Job:** report a loss, provide necessary information once, understand progress,
respond when needed, and receive an explained outcome with a review path.
**Entry:** customer is entitled to access the claim; source/session identity is
verified. Missing identity or policy association routes to a safe verification
step, not guessed coverage.
**Value hypothesis:** less chasing and repeated information, with correct outcomes.

| ID / scenario | Steps and agent behavior | Customer-visible exit / acceptance |
| --- | --- | --- |
| C-01 Report and follow a routine claim | Customer supplies narrative/facts; agent validates, creates durable claim identity, orchestrates evidence/policy/assessment work and publishes progress | One traceable claim; current stage, completed stages and next owner visible. Duplicate submission returns existing result; failed recording is not called received |
| C-02 Provide clarification/evidence | Agent identifies missing/contradictory facts and asks a specific question; customer supplies response or authorized document; agents reassess the new revision | Request explains what is needed and why; response acknowledged; earlier evidence preserved; stale proposal cannot be accepted |
| C-03 Human involvement | Agent explains a specialist need without disclosing sensitive internal signals; employee takes ownership; customer follows status without repeatedly resubmitting | Named team/role, waiting reason and next update visible. No claim that a human joined until assignment/acknowledgement is recorded |
| C-04 Review a proposed resolution | Customer receives amount/service or no-payment explanation, applicable basis and next options after required insurer authority review; accepts, rejects or requests clarification | Response is version-bound and distinct from insurer approval. Rejection does not authorize payment; explanation acknowledgement does not waive review rights |
| C-05 Confirm completion | Agent tracks authorized payment/service to external confirmation, then explains closure and provides permitted receipts | Pending/failed/unknown payment differs from confirmed; claim is not closed solely because an instruction was sent |
| C-06 Disagree or bring new evidence | Customer requests review/reopening, supplies reasons/evidence; authorized workflow assigns ownership and reassesses | Request receipt, reviewer and state visible; original outcome retained; no silent deletion or promise of reversal |
| C-07 Disconnect, retry or unsupported input | Customer returns to conversation/status; repeated writes use stable identity; unavailable automation produces actionable recovery | Persisted progress survives session loss; no duplicate claims/payments; unsupported input never receives fabricated analysis |

Customer acceptance includes screen-reader-readable stage updates and keyboard
operable conversation/actions. Estimated completion time must be sourced or
clearly unavailable, not invented. Attachments require size/type/security checks;
uploading a document does not prove its accuracy or authorization.
Recovery owners: servicing workflow for missing status; customer for requested
information; assigned specialist for escalation; payment operations for unknown
delivery; review team for disputes. Concrete teams and response SLAs need selection.

Measure report completion, repeat-information requests, customer effort/status
chasing, time waiting by owner, unresolved cases and complaints/review requests.
Faster closure is not a benefit if incorrect decisions or disputes increase.
No target or research result for these full-claim measures has been approved.

## Employee: resolve an exception without losing context

**Job:** inspect why automation stopped, review source evidence and current
guidelines, apply authorized judgement, and return control safely.
**Entry:** authenticated employee with assignment and scope matching the case;
financial authority is independently verified, not inferred from employee role.
**Value hypothesis:** focused judgement rather than repeated evidence assembly.

| ID / scenario | Steps and agent behavior | Required exit / acceptance |
| --- | --- | --- |
| E-01 Acquire an exception | Employee opens assigned work, claims ownership with a version check; Rue presents reason, required decision, customer context and linked artifacts | One owner or explicit collaboration model; competing assignment conflicts visibly; source/guideline versions shown |
| E-02 Investigate with agent chat | Employee asks evidence/policy/state questions; agent retrieves only authorized sources and references records; employee inspects documents and gaps | Answers distinguish facts, model suggestions and missing evidence. Chat cannot silently alter claim state or grant authority |
| E-03 Resolve urgent/ambiguous intake | Employee arranges permitted specialist handling or requests customer clarification; customer alone supplies their response | Intervention and remaining work recorded; urgency is not medical diagnosis/dispatch; employee does not impersonate customer clarification |
| E-04 Recover assessment failure | Employee sees failed attempts, enters authorized manual assessment or chooses approved recovery | Technical failure remains in lineage/metrics; manual assessment has actor/source/revision; retries obey budget and cannot erase failure |
| E-05 Review resolution | Employee examines authoritative policy, evidence, calculations and proposal; signs, changes, rejects or escalates within granted authority | Exact proposal/version bound to decision. Changed proposal invalidates prior approvals/customer response; outside-scope action is denied/escalated |
| E-06 Resume or transfer ownership | Server verifies resolved gate and current inputs; agent continues or exception moves to authorized owner | Employee observes recorded result and subsequent state; unresolved gate cannot be marked complete by chat text; handoff needs acknowledgement |
| E-07 Handle stale edits, disputes and unavailable tools | Refresh/reconcile conflicting changes; review previous decisions without rewriting them; preserve draft when write fails | No unchecked overwrite, silent reapproval or blind financial retry; escalation retains an owner and audit trail |

Required context: authorized customer identifiers, claim/policy association,
original and clarified input, current proposal and amounts, evidence provenance,
previous decisions, exception history, authority scope and applicable guideline.
Expose only necessary sensitive fields; identity tokens and unrelated cases are
never model context. Guideline text/version must come from an approved source.

Measure active review time, context-gathering effort, correction/rework,
exception age and downstream decision correctness. Review speed alone does not
establish quality. Named specialist/claims authority owns decisions; integration
and operations owners handle failed writes/dependencies, not the employee guessing.

## Operator: supervise the fleet and establish value

**Job:** understand which agents are working or blocked, diagnose operational
failure, coordinate safe recovery and measure quality-adjusted economics.
**Entry:** authenticated operator scoped to environment/tenant/workload.
Monitoring access does not grant claims decision or payment authority.
**Value hypothesis:** less opaque automation and faster safe recovery.

| ID / scenario | Steps and agent behavior | Required exit / acceptance |
| --- | --- | --- |
| O-01 Inspect fleet/workload | Filter by environment, time, agent role, case state, owner and failure; see active, queued, blocked and completed work | Defined collection window and freshness; aggregates match drill-down denominators. Missing data is visible, not zero |
| O-02 Inspect a run | Follow servicing/evidence/policy/assessment/resolution/payment delegation and tool/model invocations; inspect authorized artifact references | Correlation, revisions, versions, dependencies, budget and outcome traceable; no secret/raw-content exposure through telemetry |
| O-03 Diagnose failure or stalled work | Inspect timeout/invalid output/auth denial/unknown delivery, dependency health, retries and owner | Technical failures distinct from business exceptions; alerts have owner and runbook; no unbounded retry suggested |
| O-04 Coordinate recovery | Request permitted pause/retry/reassignment/reconciliation after checking exact scope and approval requirements | Server enforces control authority; in-flight actions/cancellation are explicit; retry cannot double-pay or bypass customer/employee gates |
| O-05 Review value and quality | Compare comparable baseline/assisted workloads, net labor effort, operating/model costs, rework and correctness | Measured results, assumptions and forecasts separated; denominator, window, provenance and uncertainty displayed; no modeled capacity labeled realized savings |
| O-06 Audit change or degraded monitoring | Inspect release/model/policy versions and access-controlled audit; detect missing/stale signals | Traceability survives restart/version changes; telemetry outage shown; restricted evidence follows review access, not dashboard-wide disclosure |

Fleet model: logical agent roles are separate from run instances. An agent can
have many simultaneous claims; aggregate states must not collapse them to one
ambiguous "working" badge. Per-run state/ownership and fleet counts share a
declared snapshot/window. The current single-case mockup does not implement this.

Value definitions: correct completed claims per staff-hour includes exception,
correction and oversight effort; compare matched workloads and report quality
and uncertainty. Cost per correct completion includes labor, model/tool costs
and declared infrastructure/implementation allocations. Gross capacity value
and realized spending reduction are different. Payment amounts are not savings.
Do not double-count correction time and avoided rework.
Full-claims quality, appeals/rework, payment errors and customer outcomes require
new labels/study design; the intake benchmark cannot supply their denominators.
Targets, metric owners, windows and alert thresholds remain approval decisions.

## Architecture and contract coverage matrix

This table is the review checklist, not proof of implementation. Architecture
must account for every row before contracts are accepted; a proposed endpoint
name alone is insufficient.

| Journeys | Required boundary/contracts | Durable artifacts/state | Acceptance evidence to create |
| --- | --- | --- | --- |
| C-01, C-07 | Identity/resource access; create/read claim; idempotent submission; agent session resume | Claim ID, source revision, conversation references, state version | Duplicate, unauthorized, write failure and reconnect tests |
| C-02, E-02, E-03 | Scoped evidence retrieval/upload; clarification request/response; revision and provenance | Evidence bundle, input revision, owned clarification, superseded assessments | Contradictions, malformed files, missing sources, stale-input tests |
| C-03, E-01, E-06 | Exception assignment/ownership; versioned intervention; resume gate | Exception, actor/scope, acknowledgement, required action | Competing owners, escalation failure, unresolved-gate resume tests |
| E-04, O-03 | Assessment request/result/attempt; bounded provider adapter | Valid judgments OR typed failure, manual assessment, attempts | Wiremock timeout/rate limit/invalid DTO, budget and manual recovery tests |
| C-04, E-05, E-07 | Policy/evidence access; versioned proposal/decision; customer acceptance/rejection | Policy snapshot, loss calculation, reviewer authority, separate customer response | Authority denied, changed offer, rejected response, wrong-role tests |
| C-05, O-04 | Authorized payment/service instruction; deduplication/reconciliation; closure gate | Payment identity, ledger/receipt, pending/unknown/confirmed result, closure | Crash, duplicate delivery, unknown payment, failure-before-closure tests |
| C-06, E-07 | Review/reopen request; access-controlled decision history | Review owner, new evidence, retained original decision | Reopen without history loss, denied access, assigned review tests |
| O-01, O-02, O-06 | Fleet/run queries; scoped lineage/artifact read; telemetry schema/freshness | Run/role IDs, delegations, invocations, versioned signals | Aggregate/drill-down consistency, stale data, redaction/access tests |
| O-04 | Separately authorized operations commands, scoped intent and cancellation | Operator identity, control approval, outcome and unfinished work | Denied financial authority, expired intent, safe cancellation/recovery tests |
| O-05 | Measurement datasets/aggregation and declared economic assumptions | Baselines, outcomes, effort/cost, windows and provenance | Formula/denominator tests, missing data, no double-counting, quality gates |

MCP tools map to validated application commands, not an alternate policy engine.
Agent task/result envelopes carry scoped identities, artifact versions, budgets
and explicit outcomes. Authorization is enforced outside models at every boundary.
Concrete tool schemas/agent transport, identity exchange, storage and deployment
choices still require decisions. Model selection must justify each role;
Jev is not assumed appropriate for every agent.

## Mockup coverage and design acceptance

Current simulation: C-01/C-03/C-05 routine path and selected exceptions; scripted
C-02/C-04; E-02 scripted chat and selected E-03/E-04/E-05 actions; O-02 single-case
lineage and O-05 illustrative calculator. These are partial UX demonstrations,
not contract/security tests. Real identity, uploads, durable state and external
acknowledgements are absent.

Missing interactive branches: offer rejection, employee proposal modification,
contested coverage, failed/unknown payment, reconnect, reopening, competing
assignment, fleet filtering, monitoring degradation and operator recovery.
Prioritize them during design review; do not represent them as built.

Accept this baseline only after walking the scenarios with customer/claims/
operations representatives or explicitly documenting the synthetic-review
limitation. Resolve action/financial authority, claimant verification,
guideline/policy sources, exception owners/SLAs, supported claim types and
jurisdiction, customer review rights, and role-specific data access.
Then update architecture with states/boundaries for each journey, record
consequential changes in ADRs, and define Rust contracts plus example JSON/tests.
No real claims decision, payment or deployment is authorized by this specification.
