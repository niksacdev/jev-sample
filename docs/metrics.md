# Claims product metrics and financial value

Status: end-to-end scorecard and financial model **proposed for claims/finance
expert and product-owner review** under [issue #13](https://github.com/niksacdev/jev-sample/issues/13).
The separately identified intake-component targets remain approved as of
2026-10-05. No holistic numerical targets or financial results are approved.

## Proposed holistic product scorecard

### Product mechanism and decision boundaries

The insurer hypothesis is that accountable evidence gathering, bounded assessment,
correct exception handling and verified fulfilment reduce total handling effort
and customer uncertainty without degrading entitlement, correctness or recourse.
The vendor hypothesis is that customers will pay for that demonstrated net value
at a price supporting sustainable delivery economics. Neither is established.

Use two linked scorecards: insurer/customer outcomes and product-provider economics.
Insurance premium, claim indemnity, and vendor subscription revenue are different
flows. Lower payouts, fewer appeals, faster denial or fewer necessary interventions
are not themselves benefits. Coverage and payment decisions require independent
authority; confidence cannot authorize them.

### Proposed north star

**Quality-verified resolutions per total claims-handling staff-hour**, paired with
customer effort, appropriate decision/fulfilment quality, recourse and safety gates.
Report handling cost per quality-verified resolution beside it.

A resolution needs the current authorized decision, required customer response,
verified payment/service outcome or an authorized explained no-payment outcome,
and an available review channel. Sending a payment instruction or agent prose is
not completion. Closure remains revisable; report later correction/reopening by
cohort and observation age. A no-payment acknowledgement is not waiver of rights.
Expert sampling establishes quality; raw closures are not all presumed correct.
If only a sample is scored, disclose sampling/weighting and uncertainty rather
than presenting an unobserved population count as verified.

### Process-grounded measures

Owner roles below are proposals requiring named assignments before a pilot.
Windows and quality/safety tolerances require approval; they are not universal
insurance SLAs. Segment by line of business, jurisdiction, complexity/severity,
channel, catastrophe exposure and required exception; compare like workloads.

| Stage and customer/operational job | Measure and boundary | Proposed evidence owner |
| --- | --- | --- |
| Report and identity | Durable reports / eligible report attempts; duplicate claims / report attempts; time to acknowledgement; verification failures | Claims intake lead |
| Gather evidence | Repeat-information requests / claims; active collection effort; waiting time by customer/external source/insurer; completeness against reviewed requirements | Claims operations lead |
| Policy and loss assessment | Independently scored material decision/calculation errors / audited decisions; corrections and evidence gaps, split by severity | Claims quality lead |
| Exception handling | Required interventions correctly escalated / independently labeled required interventions; false escalations; unowned/overdue exceptions and acknowledged transfer time | Specialist operations lead |
| Offer and required decisions | Time to an actionable authorized proposal; customer comprehension/effort; rejection/review outcomes, without treating acceptance as correctness | Customer experience and claims authority |
| Payment or service | Confirmed correct fulfilments / authorized fulfilment attempts; unknown/failed/duplicate/wrong-recipient/wrong-amount outcomes separately; time to confirmation | Payment operations |
| Explained closure and recourse | Quality-verified resolution throughput; end-to-end resolution time; unresolved inventory age; subsequent upheld corrections/reopenings by mature cohort | Claims quality and operations |
| Customer service throughout | Status-chasing contacts and repeated input per claim; accessible completion; complaints and barriers to review, with survey nonresponse visible | Customer experience lead |
| Agent operations | Failed attempts / all attempted operations; lost/stalled claims; recovery time; budget exhaustion; missing telemetry, privacy and authority violations | Engineering operations lead |

Do not merge active human time with elapsed waiting. Show wall-clock stage and
journey p50/p95, open-case age and time-to-outcome analyses accounting for still-open
cases; completed-only averages create selection bias. Define calendar/business
time and externally constrained waits before comparison. A model-call latency
target is separate from a weeks-long claims resolution target.

### Quality-adjusted autonomy and guardrails

Report eligible claims completed within delegated authority without human
intervention / all predeclared eligible claims, together with the eligibility
share of all claims. Freeze eligibility before outcome inspection. Show necessary,
avoidable and safety-driven intervention separately; maximize useful resolution,
not autonomy alone. Manual recovery remains part of total cost and correctness.

Gate any autonomy or economic claim on independently assessed entitlement and
calculation correctness, appropriate urgent handling, fulfilment reconciliation,
privacy, authorization, accessibility and effective recourse. Specify severity,
breach response and accountable owner. No relaxed tolerance is implied by
"proposal"; consequential breaches require stop/containment and human review.
Unchanged complaint or appeal counts are not proof of preserved rights: inspect
accessibility, suppression and underlying outcomes.

## Proposed dual-sided financial model

### Shared accounting contract

Finance and claims experts must select cost boundaries, currency, period,
eligible volume, workload mix, quality gate, adoption/ramp and counterfactual
before ROI is calculated. Maintain an input register with units, provenance,
owner, date, range and observed/estimated/scenario status. Missing values are
unknown, not zero. Reconcile modeled volumes and costs to operational and financial
records. Compare cohorts at matched maturity; losses and handling expenses must
not be confused with paid cash or final ultimate cost.

For a property/casualty insurer, separate indemnity from loss-adjustment expense
and map handling costs to its finance-approved defense/cost-containment versus
adjusting/other categories where applicable. Do not assume this taxonomy applies
unchanged to life, health or every jurisdiction. Paid-to-date, case reserves,
incurred estimates and ultimate cost are different measures; reserving changes
are not realized savings. Actuarial review is required for claims-development
or indemnity-impact conclusions.

### Insurer economics

Measure fully loaded handling expense per eligible claim and per quality-verified
resolution, including human intake, specialist review, oversight, correction,
customer contacts and reconciliation. Include recurring vendor charges, models/
tools where not already bundled, infrastructure, security/compliance, support and
change operations. Track one-time integration, migration, training, procurement
and parallel-run costs separately; specify allocation horizon for unit costs.

For a comparable period and workload:

```text
Net operating benefit = addressable baseline operating cost
                      - retained assisted operating cost
                      - incremental recurring technology/vendor cost
ROI = (attributable benefits - incremental costs) / incremental costs
NPV = sum(incremental net cash flow[t] / (1 + discount_rate)^t), including t=0
```

The first formula excludes one-time investment; ROI/NPV include it in the declared
horizon without counting it twice. Define whether amounts are cash or allocated
expense; do not mix definitions. Zero investment makes ROI undefined, not infinite.
Payback is the first period cumulative incremental cash flow recovers investment;
report "not reached" if the modeled horizon never crosses zero.

Classify labor benefits as cash spend actually removed/avoided, capacity released,
or service improvements. Saved minutes times loaded wages is capacity value unless
an approved staffing/overtime/vendor-spend change realizes cash. Do not count both
the same capacity value and its later cash conversion. Incremental growth benefits
require demonstrated demand and attributable contribution margin, not gross premium.

Avoided rework labor already included in total effort cannot be added again.
Appropriate indemnity is not an automation expense saving; any independently
validated leakage/recovery hypothesis needs a separate expert-approved model that
also detects underpayment and preserves rights. Faster legitimate payment can
change insurer cash timing: model working capital separately from expense savings.
Vendor price is an insurer cost and vendor revenue, not a new combined surplus.

### Agent-product business economics

The commercial model is not chosen. Compare per-tenant subscription, usage and
per-verified-outcome pricing against customer value, allocation fairness and
incentives; do not price rewarded denials or unverified "completions".
Outcome pricing needs an auditable outcome definition, dispute treatment,
reversals/reopenings and attribution beyond the model alone.

```text
Gross profit = recognized product revenue - finance-approved cost of revenue
Gross margin = gross profit / recognized revenue
Unit contribution = net unit revenue - defined variable unit delivery costs
Customer acquisition payback = attributable acquisition cost /
                              comparable-period customer contribution
```

State the accounting policy and unit for each formula. Gross margin with zero
revenue is undefined; nonpositive contribution has no finite acquisition payback.
Monthly contribution implies payback in months. Bookings, invoicing, collected
cash and recognized revenue are not interchangeable.

Cost-to-serve includes inference and retries, tools/data licenses, infrastructure,
tenant security/compliance, human escalation where the vendor bears it, support,
onboarding and incident recovery. Finance decides expense/COGS treatment; report
operational cost-to-serve separately if it differs from statutory gross margin.
Distinguish development and sales spend from delivery costs without hiding them
from overall operating profit, cash runway or investment returns.

Track retention/churn, expansion, contract duration, acquisition/onboarding cost
and concentration. Do not claim lifetime value from a short synthetic pilot or
assume constant retention forever. Price sensitivity should show insurer net value
after fees and vendor contribution under the same volumes and workload mix.

### Sensitivity, attribution and expert acceptance

Use downside/base/upside scenarios, not a single precise forecast. Vary eligible
volume/mix, adoption ramp, exception/rework rate, realized cash conversion of labor,
price, inference/tool spend, support and integration effort. Show break-even
volume/price only when inputs are defined, with capacity and quality constraints.
Correlated assumptions cannot all improve independently in the optimistic case.

Use matched/manual-assisted or controlled rollout evidence; account for staffing,
seasonality, catastrophes and policy changes. Report causal limitations and
uncertainty. An illustrative capacity calculator is not measured ROI.

Before adopting the scorecard, human claims operations, finance, customer/recourse
and engineering representatives must validate process coverage, cost accounting,
baselines, legal/authority boundaries, metric owners and achievable target ranges.
The research agent provides published-source support, not credentialed financial
approval. Issue #13 holds review status and named owner assignments.

### Evidence basis and expert review boundaries

Sources ground definitions and review questions, not numerical benchmarks,
product effectiveness, or authorization to handle real claims:

| Source | Use and limitation |
| --- | --- |
| [NAIC Model Act 900](https://content.naic.org/sites/default/files/model-law-900.pdf) | Prompt, fair investigation and settlement principles; a model act, not universally applicable law. Counsel must confirm local enactment and deadlines. |
| [ASOP 43: Property/Casualty Unpaid Claim Estimates](https://www.actuarialstandardsboard.org/asops/propertycasualty-unpaid-claim-estimates/) | Unpaid-estimate scope, methods, assumptions and uncertainty; actuarial input for maturity/reserving comparisons, not a product-ROI standard. |
| [NAIC P/C annual statement instructions](https://content.naic.org/sites/default/files/publication-asi-pua-25.pdf) | Claims/loss-adjustment reporting taxonomy; finance must confirm applicable edition, accounting policy and allocation. |
| [KPMG summary of SEC KPI guidance](https://kpmg.com/us/en/frv/reference-library/2020/sec-issues-mda-guidance-kpi-metrics.html) | Clear definitions, calculations, management use, assumptions and methodology changes. Secondary summary; SEC primary text was inaccessible to the research agent. Public-company disclosure context is not a universal product regulation. |
| [OMB Circular A-94](https://www.whitehouse.gov/wp-content/uploads/legacy_drupal_files/omb/circulars/A94/a094.pdf) | Discounted benefits/costs as methodological reference; federal guidance is not a private-insurer discount-rate mandate. Finance selects horizon/rate. |

Research does not constitute actuarial, accounting, investment or legal advice.
Revenue recognition and cost-of-revenue classification require the vendor's
applicable accounting standard and qualified accountant review; no authoritative
ASC 606 text was verified in this research. Acquisition-payback and retention
definitions are disclosed management conventions, not universal statutory rules.

Expert acceptance must resolve: supported line/jurisdiction; independent outcome
adjudication and maturity windows; addressable expense and capacity-to-cash policy;
indemnity exclusion or separately justified study; horizon/discount/cash treatment;
pricing, revenue recognition and delivery-cost boundaries; baseline/attribution;
named metric owners and breach authority. None is silently filled with an industry
average or an invented expert sign-off.

## Applicability and current evidence

The approved scorecard below applies to the bounded auto-insurance intake
experiment, not end-to-end claims or all servicing intents. The runnable
provider-comparison app and expanded claims vision do not yet have an approved
matching product scorecard. Do not apply intake targets to them without review,
or treat UI improvements and passing tests as evidence of business value.
The buyer, commercial model, willingness to pay, and unit-economics assumptions
also remain unvalidated. Cost reductions need an explicit beneficiary and
measured total effort, not API spend alone.

### Runnable servicing comparison: measurement boundary

The current executable assesses `claim`, `policy_change`, `customer_details`,
and `billing`, using Jev rubric `servicing-intents-v1` and default routing policy
`experiment-review-only-v1`. Matching intents create review-required tasks;
no-match results request clarification. Neither outcome is an automatic intake
route, an urgency judgment, a completed human review, or a completed claim.

Operator run/failure counts describe process-local assessor runs, not distinct
customer requests or independently scored outcomes. Selecting two assessors
creates two runs for one submission. Restart loses history; rejected requests
create no run. These counts alone do not establish a failure-rate denominator.

`elapsed_ms` measures assessor execution through completion/failure, before
routing and response delivery. It is not submission-to-visible-result latency.
The browser displays comparison results after all selected requests settle.
Record these boundaries separately; do not compare either with the approved
response-latency target without a matching measurement.

Any servicing diagnostic report should declare its workload, window, provider,
versions, attempted/admitted/rejected/completed/failed counts, and missing
observations. Unknown usage is not zero cost. These are diagnostic definitions,
not new approved product targets. The approved intake scorecard is unchanged.

### Intake gate applicability to the runnable code

| Approved measure | Current evidence boundary |
| --- | --- |
| Correct triages per analyst-hour | Unmeasured: no timed manual/assisted exercise |
| Final routing correctness | Unmeasured: no independent labels or manual baseline |
| Automatic routing precision | Undefined for current review-only tasks: zero automatic-route denominator |
| Automation coverage | No automatic routes implemented; not an executed intake-benchmark result |
| Urgency escalation recall | Intake urgency judgment and expedited-review route not implemented |
| Ambiguity escalation recall | No-match clarification is not labeled ambiguity/contradiction detection |
| Response latency | Unmeasured end-to-end p95; assessor elapsed time is a different boundary |
| Technical failure rate | Unmeasured over a declared workload/window; failure tests are not a rate |
| Technical failure routing | Visible error/failed run exists; owned review queue/handoff not implemented |

## Approved intake-component benchmark

The following original hierarchy, targets and experiment procedure apply only
to intake. They are retained without changed thresholds and do not serve as the
holistic product scorecard.

### Intake north star

Correctly triaged claims per analyst-hour, paired with routing and urgency
quality gates. Include review and correction effort, not just model response time.

### Intake metric hierarchy

| Layer | Candidate measures | Purpose |
| --- | --- | --- |
| Business value, lagging | Cost per correctly triaged claim, analyst time per claim, routing rework | Establish actual workflow value |
| Product | Time to usable triage, completion, corrections, review burden, repeat usage | Measure usefulness and appropriate reliance |
| Operational, leading | Input completeness, contradictions, review queue age, approaching SLA, detected drift | Warn of likely problems |
| Operational, lagging | SLA attainment, technical failures, downstream misroutes, actual cost, incidents | Record realized outcomes |
| Model evaluation | Category macro-F1, precision, urgency/review recall, calibration, error versus coverage | Evaluate bounded judgments and gating |
| Diagnostics | Errors by category, urgency, ambiguity, length, paraphrase; option-order sensitivity; component failures | Explain performance |
| Guardrails | Unsupported coverage/payment claims, unauthorized actions, privacy exposure, missed review cases | Enforce scope and safety |

Leading and lagging are relative to a particular outcome, not intrinsic labels.
Predictive signals do not establish causation. Acceptance or adoption alone
does not establish correctness.

### Approved intake primary scorecard

| Metric | Target | Definition |
| --- | --- | --- |
| Correct triages per analyst-hour | At least 20% improvement over manual triage | Controlled user exercise including review/correction time |
| Final routing correctness | No reduction versus manual | Independently scored final decisions, with uncertainty reported |
| Automatic routing precision | At least 95% | Correct automatic routes / all automatic routes |
| Automation coverage | At least 50%, while meeting quality gates | Automatic routes / all attempted intakes |
| Urgency escalation recall | At least 98% | Urgent cases flagged for expedited human review / all labeled urgent cases |
| Ambiguity escalation recall | At least 95% | Labeled review-required cases escalated / all labeled review-required cases |
| Response latency | p95 at most two seconds | Submission to routing/review result, including retries |
| Technical failure rate | At most 1% | Failed or invalid evaluations / all attempted evaluations |
| Technical failure routing | 100% visibly routed to review | Failures routed to review / all technical failures |

Technical failures remain failures in reporting even if review routing succeeds.
Report counts and uncertainty, not percentages alone.

## Business value and comparison

Cost per correctly triaged claim includes API spend and measured analyst effort.
State labor-cost assumptions explicitly.

Compare Jev, a structured-output LLM, and deterministic rules at common quality
gates. Lower cost or latency does not count as an advantage if quality, urgency
recall, or useful coverage suffers. Inconclusive evidence is a valid result.

Required diagnostics include calibration, per-category results, false urgency
alerts, human-review workload, and robustness.

## Measurement stages and governance

1. Offline synthetic evaluation measures model judgments, routing, latency, and cost.
2. A manual-versus-assisted user exercise measures throughput and final correctness.
3. Real operation would be needed for sustained adoption, downstream rework,
   SLA outcomes, and realized savings; it is outside the initial sample.

Without a user exercise, time savings are unmeasured. Synthetic evaluation
cannot certify production use or prove real insurance business value.

Before execution, specify each metric's owner, measurement window, baseline,
instrumentation, target, and breach response. Record model, rubric, and policy
versions and thresholds without sensitive narratives in telemetry.

Failed gates require revision or narrower automation, not post-hoc target
lowering. Insufficient sample size means insufficient evidence.

The comparison design, dataset sizes, and statistical confidence criteria are
approved in [evaluation-design.md](evaluation-design.md). The precise rubric
and execution settings still need to be resolved.
No acceptable correctness degradation margin has been approved.

## Product/architecture feedback loop

The [engineering standard](../engineering-standards.md#change-discipline) defines
the obligation; GitHub issues and PRs hold live hypotheses, evidence, status,
and ownership. This document holds current approved metric definitions and
scope, not a second task backlog. ADRs hold architectural rationale.

1. **Frame in the issue.** Link the applicable metric and explain the customer
   problem, business-value mechanism, expected effect, and guardrails. Specify
   baseline, numerator/denominator, source, window, owner, and review trigger.
   Record indirect/no impact with rationale; do not force maintenance into a
   fabricated product experiment. Unapproved metrics are proposals.
2. **Use metrics to choose architecture.** Compare alternatives using total
   workflow effort/cost, quality, latency, authority, and operational constraints.
   Record assumptions and experiments in the issue; capture consequential choices
   in an ADR linked back to the issue and affected metric definitions.
3. **Feed technical evidence back.** When feasibility, observed behavior, or
   cost challenges a product assumption, record previous/proposed scope or metric,
   evidence and limitations, tradeoffs, and reevaluation. Mark the decision
   proposed, approved, rejected, or deferred. Obtain explicit user/product-owner
   approval before changing the approved scorecard. Preserve the old definition
   and rationale through Git history and decision links.
4. **Review delivery in the PR.** Compare the issue hypothesis with actual
   evidence. Report implementation checks separately from product measurements,
   with counts and uncertainty where relevant. Update affected definitions and
   ADRs in the same change for approved revisions. Partial results and unknowns
   remain visible; a merge establishes delivery, not product success.
5. **Review outcomes on the issue.** At the named date or milestone, compare
   observed outcomes with baseline and guardrails. The user/product owner chooses
   continue, revise, narrow, or stop and records why. A failed gate or insufficient
   evidence requires explicit response, not post-hoc relabeling as success.
   If delivery closes before measurement, keep an outcome-review follow-up issue
   with a named owner and trigger. Do not close that follow-up until the evidence
   review or an explicit decision to stop measurement is recorded.

Review is also triggered by material scope, provider/model, policy, authority,
architecture, or business-cost assumption changes and by guardrail breaches.
For each approved metric revision, record the decision date, originating issue,
approval reference, prior/new definition or target, applicable scope/version,
and reevaluation requirement here beside the affected definition.

### Current reconciliation needed

Before claiming value for broader servicing or end-to-end claims, agree the
beneficiary/buyer, business model assumptions, value mechanism, and corresponding
scorecard. Decide explicitly whether the intake experiment remains a separate
benchmark. Metric owners, measurement windows, and product exercise execution
remain unresolved; no measured product improvements are asserted here.
Track that reconciliation in an issue rather than copying historical milestones.

These controls are review-enforced. No automated linkage/metric gate, recurring
review automation, production measurement, or assigned metric owners are implied
by the templates or this procedure.
