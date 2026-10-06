# Intake analyst workbench: product discovery draft

Status: Proposed, not approved for contract implementation.
Direction update: autonomous-first intake-to-handoff requested by the user;
human intervention by exception, with continuous state/lineage/artifact access.
Date: 2026-10-05.
Primary persona approved: insurance intake analyst.
Design: [interactive mockup](design/intake-workbench.html).
System design: [first-level architecture and contracts](system-design.md).

## Problem and customer value

An intake analyst receives a narrative and needs to decide the next handling
queue, whether expedited human attention is needed, and whether the available
information supports that decision. Our hypothesis is that reading, interpreting,
and correcting routing consumes time and that ambiguous information is easy to
miss. This is a discovery assumption, not observed customer research.

The job: "Help me route this intake correctly without hiding uncertainty or
making me redo the assessment." The intended beneficiary is the intake team;
claimants may benefit from appropriate handling, but no reduction in claim
settlement time has been demonstrated.

Before assistance, the analyst reads the narrative, selects a category, assesses
urgency, identifies missing information, and records the next handling action.
With assistance, narrow judgments organize the same task; the analyst can inspect
the original narrative, correct suggestions, and retain review ownership.
Assistance is valuable only if reading/checking/correction effort is lower while
final correctness remains intact.

## End-to-end value creation map

All benefits below are hypotheses. No customer study or production outcome has
been measured. The mechanism must improve the complete job, not merely one screen.

| Journey stage / friction | Product mechanism | Immediate useful output | Intended customer value | Evidence and countermeasure |
| --- | --- | --- | --- | --- |
| Intake arrives; facts may be missing | Validate input and preserve source identity | Traceable intake or actionable validation error | Less downstream rework | Completeness/rework; do not reject useful partial narratives without an approved rule |
| Analyst selects work; responsibility unclear | Explicit assignment and work status | Named owner and visible next action | Fewer lost intakes | Unowned time/queue age; measure against current workflow |
| Analyst reads and categorizes | Bounded semantic assessment of the same narrative | Proposed category, urgent cue, review requirement | Less repetitive interpretation effort | Total read/check time and correctness; not inference speed alone |
| Suggestion is wrong or uncertain | Visible source and editable disposition | Corrected analyst decision or clarification request | Appropriate reliance, fewer misroutes | Corrections, unnoticed wrong suggestions, review burden |
| Urgent case needs attention | Expedited human-review disposition | Accountable priority handoff | Earlier appropriate human attention | Urgency recall plus false-alert load and time to human attention; no medical-response claim |
| Assessment fails | Distinct failure state and manual route | Intake remains actionable with failure recorded | Continuity without false certainty | Failure rate and manual completion effort; failure still counts |
| Analyst records next action | Versioned disposition with duplicate/stale-write protection | Durable decision and pending handoff | Less repeat work, clear accountability | Lost/duplicate decisions and recording failures |
| Receiving team takes ownership | Acknowledged handoff and visible rejection/recovery | Accepted destination or owned exception | Fewer silently dropped/misrouted intakes | Acknowledgement time, rejected handoffs, downstream rework |
| Team learns from outcomes | Separate operational facts from independently scored quality | Corrections/rework and effort measurements | Evidence for improvement or reduced automation | Correct triages per analyst-hour and final correctness; clicks/acceptance are not ground truth |

Value to the analyst is reduced total effort per correct handoff. Value to the
operations lead is accountable flow and less rework. Claimant benefit is only an
indirect hypothesis until downstream handling is measured. Model cost reductions
do not count as customer value if review labor or errors increase.

## Product review scope

Walk five complete stories from arrival to accepted handoff: clear incident,
urgent cue, ambiguous intake followed by clarification, wrong suggestion followed
by correction, and technical failure followed by manual handling. Include a
rejected handoff and a stale concurrent edit in these stories.
For every transition review actor, input, output, owner, recovery action, and
measurement. An analyst clicking "record" is not yet a completed customer outcome.

The updated mockup simulates submission, validation, assessment, policy checking,
recording, pending handoff and acknowledgement. Routine processing advances
automatically. Urgent attention, conflicting information and technical failures
pause with an owner and required action; a scripted intervention resumes the run.
Every transition exposes lineage and expandable input/assessment/exception/
disposition/handoff artifacts. These are observable audit facts, not private
model reasoning. Persistence, authorization and tool execution are simulated,
not implemented. No arbitrary-input interpretation or live model is provided.

The intended experience is agentic: the customer submits an intake; an agent
progresses permitted workflow without routine analyst confirmation. Customers or
operators intervene only when required by an explicit exception. Operators can
inspect current state, source revisions, tool outcomes and artifacts at all stages.
Structured assessment/action cards and recorded state remain visible alongside
conversation. The mockup includes scripted customer status updates, not an open-ended chat.
Presentation update: fictional insurer **Reassure**, with **Rue** as the
customer-servicing agent. Customers submit through an editable chat composer,
receive stage-specific messages, and follow a seven-stage intake progress tracker.
The design-review inspection drawer exposes lineage/artifacts, not private model
reasoning or an approved customer entitlement to internal audit data.
Only exact selected synthetic stories have scripted assessments. Changed text
is preserved as input and pauses for a human rather than inventing a judgment.
No real specialist is contacted. Changing the story resets the local simulation.
The agent must not claim completion from its own prose or act beyond delegated
authority. The [agent/MCP design](system-design.md#agent-and-mcp-boundaries)
proposes identity, tool, confirmation, and message boundaries for review.

## Mockup journey and decisions to review

The mockup contains synthetic scenarios, not model output or evaluation data.
Submit a selected synthetic customer narrative and observe autonomous progress.
Resolve the scripted exception when one appears, then inspect resumed processing
and acknowledged handoff. No real queue is updated. Switching cases resets the
simulation and cancels its pending timer; nothing persists across reloads.

| State | Intended experience | Value or safeguard |
| --- | --- | --- |
| Clear incident | Suggested category and source cue, editable disposition | Reduce repetitive categorization |
| Urgent cue | Explicit expedited human-review cue | Bring attention to a potentially time-sensitive intake, not emergency automation |
| Ambiguous/mixed incident | No automatic route; visible uncertainty and missing-information cue | Prevent confident-looking misrouting |
| Technical failure | Assessment unavailable, separate failure label and manual review | Preserve service continuity without disguising a failed model call |
| Analyst correction | Editable category/attention/review; original suggestion remains visible | Allow disagreement and measure correction burden |

The source cue in the mockup is hand-authored. Whether production supports
validated evidence spans remains undecided; do not ask a model to invent
explanations or treat these excerpts as a promised API field.

Autonomous-first operation is the requested product direction. The exact action
allowlist, predelegated scopes, automation eligibility and exception thresholds
still need approval and evidence. Routine autonomy is not authority for coverage,
settlement, payment, or emergency actions. The approved automatic-route quality
and coverage targets remain experiment gates, not proven deployment eligibility.

## Model role: why Jev is plausible, not yet proven

Jev can make bounded semantic judgments from the narrative: incident category,
urgency cues, and review-required ambiguity. A structured-output LLM must receive
the same evidence and judgment task; deterministic rules are a third comparator.
Provider choice is not an analyst responsibility and is absent from the workbench.

Rust validates contracts and applies explicit routing policy. A probability is
not proof of correctness, so the mockup displays no fabricated confidence score.
Provider-specific uncertainty must be evaluated before gating automation.

Jev is not selected here for arithmetic, coverage/payout decisions, generating
claim advice, or open-ended conversations. Optional generation would be a
different feature and comparison. The design must demonstrate a customer task,
not assume a fast narrow model automatically creates business value.

## Validation before contracts

Ask representative analysts to complete manual and assisted tasks on synthetic
development scenarios. Counterbalance order and avoid showing the same case
twice to the same person where memory would bias results. Include clear, urgent,
ambiguous, wrong-suggestion, and technical-failure tasks; measure reading,
checking, correction, and completion time, not only model latency.

Review questions: Is the next action clear? Does the analyst notice uncertainty
and failure? Can they reject a suggestion? Which information is missing from
their actual workflow? Does assistance add checking work instead of saving it?
Recruitment, labeling, task count, and study power need definition before execution.
No analyst study has occurred.

Use the [approved metrics](metrics.md): correct triages per analyst-hour,
final routing correctness, correction/review burden, and quality gates.
The mockup has no performance dashboard: synthetic examples cannot establish
the >=20% throughput hypothesis or model superiority. Offline model evaluation
and a user exercise answer different questions.

## Proposed acceptance and non-goals

- Original narrative remains visible through assessment and recording.
- Analyst can correct all suggested dispositions; suggestions are not authority.
- Ambiguity and technical failure have distinct visible states.
- A recorded decision exposes category, attention, and review disposition.
- No coverage, liability, payout, eligibility, or emergency actions.
- No live model calls, real personal data, persistence, backend API integration,
  telemetry measurements, or real queue assignment in this design artifact.

## Contract handoff after design approval

Resolve queue taxonomy, required narrative fields, "other" versus "unclear",
urgency semantics, review reasons, correction handling, automation policy,
technical failure/retry UX, and whether evidence spans are required.
Then define validated domain types and explicit HTTP outcomes around those
decisions. Derive domain, route, and wiremock contract tests from the approved
states. Do not reverse-engineer production contracts from the mockup's JS objects.

The [current scope](product-scope.md), [evaluation design](evaluation-design.md),
and [architecture](adr/0007-single-package-api-and-evaluation.md) still apply.
This draft refines the persona and workflow; it does not silently amend targets
or approve implementation. Review the mockup and product assumptions before
starting the next backend milestone.
