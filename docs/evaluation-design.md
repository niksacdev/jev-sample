# Model and workflow comparison design

Status: design approved on 2026-10-05. Execution details listed below remain
to be resolved before running the experiment.
This document does not authorize paid API calls.

## Approved intake-component scorecard

This benchmark is separate from the holistic claims product scorecard in
[metrics.md](metrics.md). These approved targets apply only to the defined intake
experiment; they are not demonstrated results or full-claims acceptance gates.

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

The runnable app assesses servicing intents with review-only tasks, not this
benchmark's incident category/urgency/ambiguity contracts. The AI-native
workflow's structured-output planner selects claim tasks and decision questions,
not these intake judgments, and the evaluation CLI is absent. The benchmark
remains unexecuted; see [current evidence boundaries](metrics.md#assessment-lab-measurement-boundary).
Before execution, map required judgments/routes to actual contracts, establish
labels, freeze settings and declare measurement boundaries. Tests and UI stories
are not held-out evidence. No correctness-degradation margin is approved.

## Providers and common task

Compare deterministic rules, Jev pinned to `jev-1.13.0`, and a versioned
structured-output LLM.

All receive the same narrative, supplied facts, definitions, and categories.
No provider receives additional documents, tools, or hidden labels.

Evaluate three atomic judgments:

- Incident category.
- Urgent-assistance cue.
- Insufficient or conflicting information requiring review.

Other incidents are not automatically ambiguous. Define mixed-incident handling
and the distinction between "other" and "unclear" in the labeling rubric.

Shared Rust policy converts judgments into a suggested route, standard review,
or expedited human review. Tune provider-specific uncertainty thresholds on
development data and freeze them before test execution. Jev confidence and
LLM self-reported confidence must not be assumed equivalent.

Exclude free-form summaries from this benchmark.

## Dataset proposal

| Set | Proposed size | Purpose |
| --- | --- | --- |
| Development | 120 cases | Rubric, prompts, rules, and thresholds |
| Locked workflow test | 600 cases | Routing scorecard on a declared synthetic workload |
| Safety supplement | Enough to reach 150 urgent and 150 review-required test cases | Rare safety outcomes; groups may overlap |
| Robustness suite | 80 base cases plus controlled variants | Paraphrase, negation, distraction, instruction-like state, option order |

Predeclare category and urgency mix before authoring the workflow test. Report
coverage and cost for that mix only; do not call it representative of real claims
without evidence.

Do not pool supplements or robustness variants into workflow coverage. Report
them separately. Keep related narratives and paraphrases in the same split.
Template-derived correlations weaken statistical evidence.

## Labels and leakage controls

Write the rubric first. Each scored case needs human-reviewed category, urgency,
and review-required labels. Model-generated draft cases may be used, but model
labels are not ground truth.

Prefer two independent reviewers followed by adjudication before test execution.
Disclose single-reviewer limitations. Unresolved labels must be handled explicitly,
not silently dropped to improve metrics.

Keep the locked test outside the implementation agent's prompt/rule-tuning
context. Freeze prompts, rules, thresholds, model versions, and policy before
running the held-out evaluation.

## Approved statistical interpretation

For routing precision and escalation recall, require the one-sided 95% exact
binomial lower confidence bound to meet the approved target. Report point
estimates and numerator/denominator counts as well.

For illustration, 150 independent urgent cases with zero misses yield a lower
bound slightly above 98%. This is not a guarantee of passing and does not
account for dependence among cases. Multiple-gate and provider-selection
interpretation must be specified before making joint confidence claims.

Report paired differences between providers on common cases. Small or uncertain
differences are inconclusive.

The approved final-correctness target is no reduction. Do not introduce a
noninferiority margin without explicit approval.

## Operating measurements

Interleave provider calls to reduce time-of-day bias, using the same timeout and
bounded retry budget. Specify actual settings before execution.

Record versions, route, review reason, usage, attempts, elapsed time, and
technical failures. Report cold/warm behavior and concurrency separately.
Timeouts and invalid outputs count as failures and visibly route to review;
they must remain visible in latency and cost reporting.

Compute calibration from probability outputs where available. Do not treat
uncalibrated rule scores as probabilities. Report false urgency alerts and
review workload alongside recall.

## Product exercise

Use counterbalanced manual and assisted conditions with distinct, matched claim
sets. Score final correctness independently and include all review/correction
time.

A small pilot checks the exercise and estimates variability. Participant count
and statistical power must be established before substantiating the 20%
throughput target. Offline results alone do not establish product gains.

## Decisions still needed

- Workflow mix, labeling rubric, reviewers, and label adjudication procedure.
- LLM model and access, API budget, timeout/retry/concurrency settings.
- Metric owners, measurement windows, and precise latency/failure reporting.
- Product pilot protocol and subsequent sample-size calculation.
