# ADR 0016: Bounded agent workflows and durable synthetic comparisons

Date: 2026-10-07
Status: Accepted
Tracking issue: #16

## Context and approval

The user approved an AI-native planner, typed decision providers, application-owned
confidence gates, clarification/review/resume, and SQLite persistence. This is an
additive local synthetic experiment; ADRs 0011 and 0013's legacy experiment remains
available. Approval does not authorize paid inference, production deployment,
real insurance decisions or payment execution. Standard 0.6.0 applies.

Issue 16 enables repeatable comparison and visibility into M01–M14 measurement
contracts, not demonstrated improvement in those metrics. Comparable observation
denominators and completeness are recorded; no accuracy or business-success labels
are inferred. Product owner reviews outcome evidence after a separately approved,
labelled evaluation. Offline mock verification is implementation evidence only.

## Decision

An object-safe planner port uses OpenAI Responses structured output to propose
bounded DAGs of allowlisted local synthetic/read-only tools and versioned atomic
Predicate/Choice/Score decision questions. Rust validates IDs, dependencies,
cycles, questions and finite budgets before execution. Tool results are grounded
facts, not authority. The planner never selects thresholds, endpoints or money
actions. A grounded structured reply and optional bounded replan follow execution.
The planner requires explicit OPENAI_API_KEY and OPENAI_PLANNER_MODEL. Production
uses the fixed official endpoint; alternate URLs are test-only constructors.

DecisionProvider is separate from planning. Jev dynamically receives state and
questions; the OpenAI Decisions adapter uses predicate/choice/score and named
answers. OPENAI_DECISION_MODEL must be explicitly configured; the currently
documented gpt-6-luna is not a silently activated default. Probability,
distribution, optional confidence semantics, refusal, unsupported capabilities
and technical failures remain distinct. The deterministic baseline supports
only the synthetic completeness predicate and supplies boolean evidence, not
invented probability/confidence. Unknown questions are unsupported.

Validated versioned policy freezes provider/question-specific probability and
confidence thresholds at admission. Uncertain/refused/unsupported outcomes pause
for customer clarification or employee review according to question metadata.
Technical failure is failed, not uncertainty. Review authorization is code-owned
and does not grant real-world authority.

Each comparison freezes ONE planner-produced base plan. Independent runs use
the same initial context/question identity for matched-input decisions. Resume
is explicitly independent continuation: later contexts may diverge, and records
carry comparison mode/round identity rather than suggesting causal comparability.
Round changes are recorded as resume/replan events and per-decision task/input
identities; no separate numeric round DTO is implemented.
Each run carries its current plan revision; replanning derives a new identity
from context, proposed plan and prior revision. Resume requires the expected
plan/task identity and rejects stale reviews. Decision records/events retain
that revision. Planner lifecycle events identify OpenAI and the planner model,
separately from the run's selected decision provider.
No hidden retry or provider fallback is permitted.

## Persistence, privacy and authority

Use rusqlite with bundled SQLite and bounded spawn_blocking operations, not a
home-grown file store or blocking SQL on Tokio execution threads. A single local
connection serializes transactions. Tables hold admitted comparison/request
identities, protected snapshots, immutable scoped event sequences, and idempotent
resume admissions. Snapshots include plans/revisions, synthetic inputs, frozen
policy, questions/context identities, provider/model/usage/latency/failure/refusal
and completeness. Epoch timestamps are UTC milliseconds with distinct occurrence
and recording time. SHA-256 identities are correlation keys, not anonymization.
Occurrence time is captured by the runtime. Recording time remains unknown
until the SQLite transaction stamps new events; later saves preserve that
stamp. This is the recording transaction's timestamp, not a separately measured
filesystem fsync-completion timestamp; both fields use millisecond precision.

Transactions reserve admission before external work. Reusing a client key with
different payload fails; identical keys return durable observed state without
duplicating calls. Startup marks unfinished work interrupted, with no automatic
external retry. Persistence failures are explicit. Storage has finite record and
byte limits; exhaustion fails closed, never automatically deletes customer data.
The operator must stop the process and explicitly archive/remove the local DB
before restarting to reset retention.

The DB directory/file require owner-only Unix permissions and must not be
symlinks; the loopback-only composition root refuses insecure existing paths.
The default directory is ignored local state. Raw synthetic narratives/questions/
results are retained only in protected SQLite/operator inspection, never spans
or ordinary logs. No public employee route exposes this data. Existing OperatorAuth
protects list/detail and employee-review resume; browser origins are checked.
Public summaries have safe tool/task names and fixed events; generated reply is
the only narrative returned. Local shared-key auth is not production identity.
Both customer clarification and employee review currently require operator
authorization. A public customer-resume route is intentionally absent until a
customer-scoped session/continuation token is implemented. Review accepts only
the paused task; later decisions remain independently gated. No insurer queue
or assignment is implied.

Normalized decision answers, exact question/context snapshots and usage are
retained, not raw vendor HTTP response bodies. A failed or interrupted attempt
may have only its started/failed event and unknown usage. Database pages are
bounded as well as logical snapshot size; rollback-journal overhead is additional
bounded local storage. The process binds its loopback port before opening the
store, avoiding startup-interruption recovery by a duplicate API process.

## Alternatives and consequences

Process memory loses comparisons on restart. JSON snapshots lack transactional
admission/collision and append-only event semantics. A remote database/queue adds
operations without a current need. Bundled SQLite adds a build dependency but
provides the approved minimal durable local boundary. Per-provider independent
initial planning confounds decision comparisons, so matched-input admission
shares one base plan; independent continuation is separately identified.

LLM self-reported confidence is not a decision probability. Confidence gates
must preserve vendor semantics and remain calibration hypotheses. Deterministic
evidence is capability-labelled, not an insurance rules engine. Synthetic facts
and bounded choices are intentionally limited; no arbitrary URL, shell, payment,
policy mutation, real customer systems, production recovery or identity exists.

## Verification and follow-up

Offline vendor mocks and Rust tests cover contract/value validation, finite plan
execution, grounded tools, uncertainty/review/resume, matched inputs, partial
failures, authorization, durable restart interruption, deduplication/collisions,
retention, redaction and failure propagation. Run formatting, check, clippy and
tests on the exact candidate. Parent owns UI/integration and independent review.
Vendor account behavior and current API availability remain unverified without
separately authorized live testing. This ADR records direction before source
implementation; the delivery report must identify any incomplete requirements.
