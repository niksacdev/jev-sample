# Metrics and success targets

Status: hierarchy and initial primary targets approved on 2026-10-05.
These are experiment targets, not demonstrated capabilities.

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
