# Product scorecard and economics

**Status:** proposed definitions for product, claims and finance review.
No holistic targets, baselines, ROI or product gains are established.
Scope: [product scope](product-scope.md); jobs: [persona journeys](persona-journeys.md);
behavior: [product specification](product-spec.md).

## Value hypothesis and north star

For claimants: less uncertainty and repeated effort while obtaining a fair,
explained outcome with effective recourse. For insurers: less total handling
effort per correct resolution without shifting cost or risk to customers.
For the product provider: customers buy that demonstrated net value at a price
supporting sustainable delivery economics.

**North star: verified resolutions per total claims-handling staff-hour (M02),
gated by independently audited decision quality (M01), fulfilment (M06),
recourse (M07) and safety.** Faster closure, higher denial rates and fewer human
interventions are not standalone successes.

A verified resolution has a recorded authorized decision, the required customer
response, externally confirmed payment/service or a properly authorized explained
no-payment outcome, and an available review path. Payment instruction sent is not
payment confirmed. No-payment acknowledgement is not acceptance or waiver.
Closure can be reopened; outcome quality requires independent review.

## Measurement contract

Before a pilot, assign named owners and approve supported claim type/jurisdiction,
baseline, target, tolerances, observation window, data sources and breach response.
Roles below are proposed accountable functions, not assigned people.

Use claim IDs for customer outcomes, task/attempt IDs for execution, and tenant/
contract IDs for vendor economics. Two assessor runs for one submission are not
two claims. Count retries, rejected attempts, failures, unresolved cases and missing
observations explicitly. Unknown is not zero.

Compare matched cohorts by complexity/severity, channel, jurisdiction, catastrophe
exposure and development age. Report counts and uncertainty, not just percentages.
Open cases remain in inventory and time-to-outcome analysis; completed-only cycle
times are biased. Separate active work, insurer wait, claimant wait and external
wait; declare calendar/business time.
Report open, partially resolved and litigated shares at each cohort checkpoint.
Disputes can precede closure: use the relevant decision cohort for appeals and
the closure cohort for reopening, never mismatched calendar-period counts.

## Metric dictionary

Targets are **to be established from baselines and approved before outcome
inspection**. Numerical thresholds from the intake benchmark do not transfer here.
Windows below specify boundaries; their durations remain pilot decisions.

| ID | Outcome and calculation | Collection boundary / owner | Interpretation and guardrail |
| --- | --- | --- | --- |
| M01 | Material decision error rate = independently adjudicated materially wrong coverage/valuation/resolution decisions / audited decisions; severity reported separately | Stratified independent sample with reviewer disagreement/adjudication; fixed decision cohort; claims quality | Audit entitlement, authority and no-payment basis as applicable. Reviewer agreement alone is not truth. Report sampling weights, uncertainty and missing labels; never certify production from synthetic labels. |
| M02 | Verified resolutions / total active handling hours, including intake, investigation, specialist review, correction, oversight and recovery | Same cohort and observation horizon for numerator and effort; claims operations | Denominator includes all cohort claims, including unresolved, failed and in-progress work. If only sampled quality is known, report raw throughput plus estimated quality with uncertainty, not a fabricated verified population count. Gate on M01/M06/M07. |
| M03 | Report-to-verified-resolution p50/p95; stage time and unresolved inventory age; time-to-outcome with still-open cases accounted for | Durable acknowledgement through verified outcome, split paid/service/no-payment; claims operations | No excluded slow cases or stopped clocks without disclosure; model execution time is a separate diagnostic. |
| M04 | Status-chasing contacts and repeat-information requests / eligible claims; accessible completion and customer-effort survey distribution | Whole journey and fixed post-outcome window, including nonresponse; customer experience | Fewer contacts may indicate barriers, not lower effort. Validate comprehension/accessibility; satisfaction is not correctness. |
| M05 | Required escalation recall = correctly escalated cases / independently labeled required-escalation cases; false escalation and review hours alongside | Define urgency/authority/evidence/technical exception classes, acknowledgement time and unowned age; specialist operations | No optimizing away warranted review. Detection and completed human handoff are distinct. |
| M06 | Correct confirmed fulfilments / authorized fulfilment attempts; wrong amount/recipient, duplicate, unknown, failed and late outcomes separately | Instruction identity linked to authoritative confirmation/reconciliation; payment operations | Unknown is not success. No-payment outcomes have their own decision-quality audit, not a payment denominator. |
| M07 | Decisions contested / eligible decisions; closures reopened / eligible closures; upheld material corrections / adjudicated reviews; receipt-to-review-response time and severity alongside | Disputes include pre-closure offers/renegotiations; decision cohorts for disputes, closure cohorts for reopenings; fixed observation age and pending-review inventory; claims quality/compliance | Open is not undisputed. Fewer appeals may indicate suppressed recourse. Inspect access and resolution quality; upheld decisions do not alone establish fairness. |
| M08 | Failed attempts / all attempted operations; lost/stalled cases, acknowledged recovery time and missing telemetry | Attempt ledger plus durable case state; engineering operations | Recovery does not erase failure. Privacy/authorization violations reported by severity, not hidden inside availability. |
| M09 | Quality-gated autonomy = eligible claims verified within delegated authority without intervention / predeclared eligible claims; eligibility share of all claims separately | Eligibility frozen before outcomes; cohort and maturity as M02; claims operations/risk | Paired quality and recourse gates; exclude no cases post hoc. Distinguish necessary from avoidable interventions. |
| M10 | Total handling expense / eligible claims, and expense / quality-verified resolutions | Reconciled labor + oversight + rework + technology + declared allocations; same horizon; insurer finance | Indemnity separate. Report open inventory/quality estimate; disclose amortization and allocation, not just marginal model cost. |
| M11 | Insurer attributable net cash benefit, ROI, NPV and payback; released capacity separately | Counterfactual matched workload, investment horizon and adoption ramp; insurer finance | No cash conversion of saved hours without a realizable spend change. No double counting with M10. |
| M12 | Vendor gross margin = (recognized revenue - finance-approved cost of revenue) / recognized revenue; operational cost-to-serve alongside | Tenant/product period reconciled to accounts; vendor finance | Accounting classification disclosed; revenue zero means undefined margin. Include retries, tools, support and vendor-paid review. |
| M13 | Unit contribution = net unit revenue - defined variable delivery cost; acquisition payback from matching-period attributable customer contribution | Declared claim/tenant/usage unit and acquisition cohort; vendor finance | Reconcile revenue basis to M12; name and explain billed/contracted-basis differences before combined reporting. Nonpositive contribution means no finite modeled payback. CAC conventions and onboarding allocation explicit. |
| M14 | Paid activation, retention/churn and expansion; documented buyer willingness to pay and procurement barriers | Defined contract cohort, revenue/usage basis and observation duration; product/commercial owner | Trial clicks or repeated demo use are not demand. Do not estimate lifetime value from a short pilot. |

## Measurement delivery contract

Every metric must ship with its collection and calculation mechanism, not just
a dashboard label. The plan below is a **required target capability**, not
implemented instrumentation. None of M01-M14 is currently measured end to end.
Application events supply workflow facts; qualified audits, human-effort records,
authoritative integrations and finance/commercial records supply the remaining
evidence. The accountable functions in the dictionary own completeness and
interpretation; engineering owns reliable capture and reproducible computation.

Before declaring a metric operational, its owner approves the source mapping,
event/record schema, eligible population, unit, computation version, freshness
budget, observation window, access/retention rules and reconciliation procedure.
Unknown baselines and targets remain unknown even when collection works.

| Metric | System collection and external evidence | Computation and completeness check |
| --- | --- | --- |
| M01 | Versioned decision records linked to evidence/authority; independent audit selection, rubric, labels, severity and adjudication records | Join audits to exact decision revisions; report wrong/audited counts, missing labels, sampling coverage/weights and uncertainty. Fixture: wrong decision, reviewer conflict, missing audit and subsequent correction |
| M02 | Resolution-verification records; active handling intervals or validated time-study imports by claim, worker and activity, including review/support/recovery | Sum all cohort effort, including unfinished cases; validate overlapping intervals and reconcile staff totals. Count verified outcomes once per declared resolution episode; do not turn sampled quality into census verification. Fixture: failed/open claim with effort and reopened episode |
| M03 | Report acknowledgement, stage/wait transitions, verified resolution and reopening, each linked to claim/episode and accountable party | Reconstruct intervals and inventory at cutoff; retain right-censored open cases and report outcome maturity. Fixture: still-open, external wait, late transition and reopening; p50/p95 from closed cases alone must be labelled conditional |
| M04 | Contact records across assisted/digital channels with purpose; evidence-request identity/revision; reviewed unnecessary-repeat labels; survey invitations/responses and accessibility feedback | Deduplicate contacts, separate necessary follow-up from repetition/status chasing, report channel coverage and survey nonresponse. Fixture: same request retried versus justified new evidence request; absent survey is not low effort |
| M05 | Escalation requested/acknowledged/resolved records, reason and staff effort; independent required-escalation labels on cases whether escalated or not | Join labelled population to requests and acknowledgements; compute detection recall separately from handoff completion, false escalation, workload and queue age. Fixture: missed necessary escalation and unacknowledged detected case |
| M06 | Authorized instruction/revision, external execution identity, authoritative confirmation and reconciliation records; expected amount/currency and protected destination reference | Match authorized terms to confirmed effect; reconcile attempt and instruction populations, including pending/unknown/wrong/duplicate effects. Fixture: duplicate callback, unknown delivery then confirmation, wrong amount; no-payment excluded from fulfilment denominator |
| M07 | Offer/decision revisions, dispute/review receipt, review outcome/correction, closure and reopening; assisted-channel review records | Link pre-closure disputes to decision cohorts and reopening to closure cohorts; retain pending reviews and observation age. Fixture: revised rejected offer, open review and upheld correction after reporting cutoff |
| M08 | Durable attempted-operation register, starts/completions/failures, owner/transfer acknowledgements and recovery; independent source/work-state reconciliation | Compare attempted work with outcomes and expected inventory; orphan starts remain visible. Detect missing/stale capture through independent checks, not the same event stream alone. Fixture: crash after effect, lost completion, unavailable telemetry and recovered failure |
| M09 | Pre-outcome eligibility decision/rule version, authority, human-intervention reason and M01/M06/M07 verification evidence | Join frozen eligibility to all outcomes; report eligibility share, interventions and unresolved cases; recheck quality gates. Fixture: post-outcome eligibility edit rejected, necessary review and unresolved eligible claim |
| M10 | M02 effort evidence plus approved labour rates; insurer expense/vendor-fee imports with source line IDs, period, currency and allocation policy | Reconcile handling expense to approved finance totals and cohort units; disclose unallocated/missing cost and implementation allocation separately. Fixture: duplicate invoice, shared cost, open claim and missing labour rate |
| M11 | Approved comparator/attribution study; insurer cash-flow and investment records; adoption assumptions and separately labelled capacity evidence | Reproduce cash ROI/NPV/payback for a declared horizon and discount rate; reconcile recurring costs with M10 without subtracting twice. Fixture: capacity-only benefit, zero investment, negative cash flows and payback never reached |
| M12 | Provider/tool usage by attempt, infrastructure/support records, vendor cost-of-revenue and recognized-revenue imports | Reconcile operational usage/cost estimates to invoices and accounts; disclose estimate-versus-actual adjustments and finance-approved classifications. Fixture: charged failed call, shared infrastructure, missing usage and zero revenue |
| M13 | M12 reconciled revenue/cost basis plus variable-cost classifications, acquisition/onboarding records and customer acquisition cohort | Compute declared-unit contribution and cumulative cohort payback; expose incompatible revenue basis rather than combine it. Fixture: nonpositive contribution, churn before payback and onboarding counted only once |
| M14 | Contract/payer identity, paid activation, renewal/expansion/churn records from billing/CRM; separately sourced buyer research | Reconcile contract and revenue cohorts, define churn/retention convention, expose immature renewal populations and missing contract data. Fixture: trial conversion, cancellation, expansion and renewal not yet due; demo activity cannot count as paid adoption |

### Evidence integrity and reporting

Business records/events require a unique identity, schema version, event type,
occurrence and recording timestamps, authoritative source/reference, relevant
tenant/claim/decision/attempt identifiers and applicable revisions. External
imports additionally record period, units/currency and approval/provenance.
Keep narratives, documents, secrets and payment destinations out of general
telemetry; protected references are sufficient. Case identifiers join controlled
records, never unbounded metric labels.

Record business facts with their state changes so successful writes cannot
silently lose measurement evidence; reconcile external actions whose outcome is
unknown. Exact retransmission must not double-count. A retry is a new attempt,
not a new claim. Preserve corrections and reversals with links to prior records,
rather than overwriting history. Late/out-of-order arrivals need deterministic
replay, versioned recalculation and a declared report cutoff/restatement policy.
Architecture chooses the durable mechanism through reviewed decisions.

Every result exposes metric/calculation version, cohort/cutoff, units,
numerator/denominator or source totals, coverage, missing evidence, source
freshness and uncertainty. Report status as observed, estimated, scenario,
unavailable or not applicable with reasons. Zero requires a complete observed
population; a zero denominator gives an undefined rate. Degraded completeness
blocks a "verified" claim and triggers the named owner's recovery procedure.

Release acceptance requires synthetic expected-result fixtures for every row,
duplicate/retry/restart/late-data tests, source-total reconciliation and an
authorized operator's ability to trace a result to its evidence. These verify
the measurement machinery, not real-world product effectiveness.
An outcome feature is not measurement-ready until its required records exist.
Where an integration or human evidence source is absent, ship the missing-data
status and owned follow-up, not a plausible-looking number.

## Insurer economics

Define addressable operating costs and comparable outcomes before calculating
benefits. Include retained human effort, exceptions, rework, customer contacts,
oversight, security/compliance, infrastructure and support. Add vendor/model/tool
costs only where not already included. One-time integration, training, migration,
procurement and parallel operation belong in the investment cash flows.

```text
Period net operating cash benefit =
    attributable counterfactual operating cash spend
  - assisted operating cash spend (including incremental recurring costs)

Horizon ROI = (attributable cash benefits - incremental investment/operating costs)
            / incremental investment/operating costs

NPV = sum(net incremental cash flow[t] / (1 + period_discount_rate)^t), t = 0..T
```

Use either a gross-benefit/incremental-cost ledger or net incremental cash flows;
reconcile them before reporting. Do not subtract assisted recurring costs twice.
Costs/benefits must use matching horizon, currency and timing; disclose taxes and
terminal value assumptions. Zero incremental investment makes ROI undefined.
Payback is the first period cumulative net incremental cash flow recovers the
initial investment; report not reached if the horizon never crosses zero.
Discounted and undiscounted payback are different; name the one used.

Classify labor effects as realized/avoided cash spend, released capacity, or
service improvement. Saved hours multiplied by wages is capacity value, not
cash savings unless staffing/overtime/vendor spend actually changes or a justified
counterfactual expenditure is avoided. Redeployment is not automatically cash.
Demand-supported incremental business needs attributable contribution, not gross
premium. Do not add saved correction time again if total effort already includes it.

For P/C insurance, distinguish indemnity from loss-adjustment expense, with insurer
finance selecting applicable defense/cost-containment and adjusting/other expense
categories. Do not generalize this taxonomy to every insurance line.
Paid cash, case reserves, incurred estimates and ultimate cost differ; compare
claim maturity and obtain actuarial review. Lower reserves/payouts are not proof
of savings. Any leakage/recovery study needs independent entitlement review,
underpayment detection and a separate attribution design.

Faster legitimate payment changes cash timing; model it separately from handling
expense savings. Payment delay is not an optimization goal.
The vendor fee is insurer cost and vendor revenue: a transfer, not extra joint value.

## Product-provider economics and pricing

Subscription, usage and verified-outcome pricing remain alternatives, not a chosen
business model. Test willingness to pay, procurement, liability allocation,
integration cost and customer value after fees. Outcome pricing needs agreed
eligibility, confirmation, disputed outcomes, reopenings and reversals; never
reward denied claims or agent-declared completion.

Recognized revenue, bookings, invoices and collected cash are distinct. Insurance
premium/indemnity flowing through integrations is not assumed vendor revenue.
A qualified accountant must determine applicable recognition and principal/agent
treatment from contracts; no accounting conclusion is made here.

Cost-to-serve includes inference/retries, licensed tools/data, infrastructure,
tenant controls, vendor-paid human escalation, support and recovery. Finance
sets cost-of-revenue versus operating-expense policy; separately disclose operational
unit economics. Do not hide development/sales/integration expense from total
profitability or runway because it sits outside gross margin.

For acquisition payback, divide attributable acquisition cost by comparable-period
customer contribution only under a declared stable-contribution assumption;
otherwise use cumulative cohort contribution. Monthly contribution gives months.
Define gross/net retention basis and expansion treatment; do not assume perpetual
retention. Price/volume scenarios must show both insurer net value and vendor margin.

## Evidence and acceptance plan

1. Claims/product/finance reviewers approve job relevance, supported context,
   authoritative outcome definitions and the input register.
2. Establish matched manual/assisted baselines and case-mix segmentation.
   Predeclare sampling, power, labels, targets and quality/safety tolerances.
3. Measure full journey effort and customer outcomes, including failures,
   contested offers, unknown fulfilment and reopened cases.
4. Reconcile financial and workflow evidence; publish downside/base/upside scenarios
   for volume, adoption, exceptions, cash conversion, integration, price and support.
5. Human owners decide continue/revise/narrow/stop; no ROI claims before validation.

The runnable app supplies servicing-topic judgments and process-local attempt
metadata, not these full-claims outcomes. No genuine claimant study, finance baseline
or production cohort has been collected. UI stories and passing tests are implementation
evidence only. The approved intake scorecard is maintained separately in
[evaluation design](evaluation-design.md#approved-intake-component-scorecard).

## Product/architecture feedback loop

### Decision-provider experiments

Measure each comparison at two different boundaries: matched decision questions
with identical evidence, and independent workflow runs originating from the same
base plan. Record comparison/run/task/question/attempt identifiers, input evidence
identity, provider/model, question/policy/plan versions, lifecycle events, usage
and latency. Flag divergent evidence rather than pooling unmatched decisions.
Keep refused, failed, interrupted and paused runs in denominators.

Answer completeness means the provider supplied all requested valid answers;
workflow completeness means a declared terminal state was reached with required
task evidence. Neither establishes correctness or verified claim resolution.
Accuracy is unavailable without independent labels tied to exact question/input
versions. M01-M14 cannot be inferred wholesale from model traces: staff effort,
claimant experience, external confirmations and financial evidence still apply.
Provider token counts do not establish cash cost without a versioned price basis
and invoice reconciliation.

Issues hold the problem, JTBD, requirement/metric IDs, baseline, expected mechanism,
measurement owner and review trigger. Architectural choices compare total effort,
cost, quality, authority and recovery, not model latency alone. PRs report actual
evidence and unresolved hypotheses, updating affected definitions and decisions.

Technical findings may propose changes to mechanism, scope or metrics with evidence,
tradeoffs and reevaluation. Only explicit human/product-owner acceptance approves
those revisions. Pending outcomes have owned issue follow-ups even after delivery.
This implements the [engineering standard](../engineering-standards.md#change-discipline);
it is not automatic measurement or an agent's permission to lower failed gates.

## Source grounding and review limits

| Source | Supports / limits |
| --- | --- |
| [NAIC Model Act 900](https://content.naic.org/sites/default/files/model-law-900.pdf) | Fair investigation/settlement principles, not a universal jurisdictional law or numeric SLA |
| [ASOP 43](https://www.actuarialstandardsboard.org/asops/propertycasualty-unpaid-claim-estimates/) | Unpaid-claim estimate methods/assumptions/uncertainty; actuarial review required, not a product-ROI benchmark |
| [NAIC P/C statement instructions](https://content.naic.org/sites/default/files/publication-asi-pua-25.pdf) | Expense taxonomy; insurer finance must confirm applicable edition and allocations |
| [KPMG summary of SEC KPI guidance](https://kpmg.com/us/en/frv/reference-library/2020/sec-issues-mda-guidance-kpi-metrics.html) | Definitions, assumptions and methodology changes; secondary summary, public-company disclosure context |
| [OMB Circular A-94](https://www.whitehouse.gov/wp-content/uploads/legacy_drupal_files/omb/circulars/A94/a094.pdf) | Discounted-cost/benefit methodology reference; not a private insurer's discount-rate mandate |

AI product/finance research is not credentialed legal, actuarial or accounting
approval. Named expert reviewers must validate jurisdiction, cost/revenue policy,
baselines and authority before acceptance. Sources do not prove our product's
effectiveness or willingness to pay; numerical targets remain unset.
