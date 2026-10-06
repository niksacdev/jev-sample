# ADR 0011: Inject assessment providers into a shared servicing use case

Date: 2026-10-06
Status: Accepted

## Context

The user requested architectural corrections after reviewing agent.rs.
The implementation had two assessors but no behavioral port, positional keyword
mapping, a module-embedded routing threshold and orchestration inside HTTP.
This did not implement ADR 0007's intended dependency boundaries or standard
0.4.0's centralized validated routing configuration. Passing tests did not prove
architectural compliance. The user authorized correcting these defects and
independent review; this is not permission to implement unrelated claims features.

## Decision

Use one object-safe `Assessor: Send + Sync` port with a standard-library boxed
async future. Both real implementations and test assessors have current callers;
this is not a speculative abstraction. The composition root chooses the provider.
`ServicingService` depends only on the port, validated domain assessment and
injected routing policy. HTTP depends on the service, never on Jev or keywords.
The evaluation CLI can later call the same use case; it is not implemented here.

Separate modules: validated domain values; provider port; keyword adapter; Jev
adapter and rubric; pure routing; use-case coordination; HTTP; API composition.
API DTOs remain separate from assessment evidence. The existing browser schema
is unchanged; `Intent` is re-exported from the domain into contracts.

Assessment requires exactly one observation per supported intent. Private
constructors validate message byte length and finite [0,1] probabilities.
Keyword observations have boolean evidence; Jev has yes probabilities.
An LLM must not map self-reported confidence into that probability without a
justified contract/calibration decision. Provider-specific wire DTOs stay inside
the adapter. Token usage is optional, not zero-fabricated for non-model assessors.

Versioned question definitions are immutable code metadata, not configurable
entitlement or routing policy. Named fields replace tuples; keyword definitions
contain their own intent rather than relying on paired array positions.
`config/routing.json` supplies validated version/threshold at startup, optionally
replaced by `REASSURE_ROUTING_POLICY`. Bad configuration fails explicitly.
`config/execution.json` supplies validated application limits.

The application keeps the permit for the full admitted work lifetime. It uses
Tokio tasks and timeout/cancellation, not a custom scheduler. Disconnecting the
HTTP caller does not leave a permanently assessing record: admitted work finishes
within its deadline, records success/failure and releases capacity. Supervision
maps a provider panic to a visible typed execution failure. No retries/fallback,
unbounded background loop, durable queue, crash recovery or new storage adapter
is introduced. Existing process-local history limitations remain.

## Library reuse and alternatives

| Requirement | Existing facility used |
| --- | --- |
| Inbound routing/JSON/body limit/state | Axum Router, Json, rejection mapping, DefaultBodyLimit, State |
| Async execution/admission/deadlines | Tokio spawn, task abort/join, timeout, Semaphore, Mutex |
| HTTP integration harness | Tower ServiceExt::oneshot |
| Wire contracts/configuration | Serde structs and tagged values, serde_json |
| Provider HTTP/TLS/client reuse | reqwest Client, timeout, redirect policy, streaming chunks |
| Provider contract tests | wiremock real HTTP mock servers |
| Structured operational events | tracing and tracing-subscriber JSON output |
| Browser contracts | ts-rs from Rust types |

Do not add async-trait: the small object-safe boxed future uses std directly.
Do not introduce repository/store abstractions without a second implementation
or concrete persistence requirement. Request-scoped Tower concurrency/timeout
layers cannot alone enforce admitted work lifetime after caller cancellation,
so those limits belong in the use case. reqwest streaming is bounded manually
because ordinary response JSON decoding does not enforce our 32-KiB limit.

## Verification and consequences

Preserve customer/employee/operator UI and JSON contracts. Cover a third test
assessor through the same use case and HTTP route, changed policy thresholds,
boundary validation, failures, cancellation, panic and capacity. Tokio test-util
provides paused time for workflow deadlines instead of real sleeps.
Vendor request assertions are independent fixtures rather than built from the
production question definitions. Module unit tests are separated by behavior.

Selected forbidden-dependency source checks provide a small regression guard;
they do not replace architectural review or fully enforce dependency boundaries.
A future requirements-driven Cargo workspace can strengthen compiler isolation.
The root engineering standard remains 0.4.0: this change complies with existing
obligations, not a silent instruction amendment. Maintainer analysis should
classify the observed lapse and propose enforcement only where justified.

Assessment attempt events mark one invocation of the assessor port. For Jev,
that invocation is exactly one HTTP provider attempt (no retries); for the
keyword comparator it is a deterministic assessment, not a model request.
Events identify the run, attempt, assessor/model, rubric/routing versions,
outcome/failure, elapsed milliseconds and usage where available. They exclude
message text, credentials and raw provider bodies. The API installs a JSON
tracing subscriber on stderr and fails startup if installation fails.
