# Metrics and success targets

Status: hierarchy and initial primary targets approved on 2026-10-05.
These are experiment targets, not demonstrated capabilities.

## Applicability and current evidence

The approved scorecard below applies to the bounded auto-insurance intake
experiment, not end-to-end claims or all servicing intents. The runnable
provider-comparison app and expanded claims vision do not yet have an approved
matching product scorecard. Do not apply intake targets to them without review,
or treat UI improvements and passing tests as evidence of business value.
The buyer, commercial model, willingness to pay, and unit-economics assumptions
also remain unvalidated. Cost reductions need an explicit beneficiary and
measured total effort, not API spend alone.

## North star

Correctly triaged claims per analyst-hour, paired with routing and urgency
quality gates. Include review and correction effort, not just model response time.

## Metric hierarchy

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

## Approved primary scorecard

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
