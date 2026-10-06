# Development setup

## Rust environment

Install Rust through [rustup](https://rustup.rs/). The repository pins its compiler
and required formatting/lint components in rust-toolchain.toml. Rustup may download
that toolchain the first time a Rust command runs in this checkout.

```sh
rustc --version
cargo --version
cargo fmt --version
cargo clippy --version
```

Use rust-analyzer in your editor if desired; it is not a project dependency.
Axum/Tokio are selected for the HTTP server; wiremock is selected for provider
HTTP tests. Axum, Tokio, Serde, and the Tower test utilities are now used;
wiremock now exercises the Jev HTTP adapter with dummy keys and local responses.

## Repository hygiene

Track the existing Cargo.lock. Ignore generated
target/coverage files and local .env files. Only sanitized .env.example content
may be committed; ignore rules are not secret scanning.
Never put API keys or real claim narratives in fixtures or telemetry.

The library exposes an HTTP router and the `api` binary serves it locally.
The serving application now delegates synthetic servicing messages to a bounded
Rust coordinator and keyword/Jev assessment adapter. See the root README for the
React UI and opt-in Jev commands. No consequential actions are implemented.
Configured lints forbid project unsafe code and reject unwrap/expect.
The Rust CI workflow repeats checks; its hosted result must pass before the first
behavioral slice is accepted. Use the [candidate-review runbook](candidate-review.md)
for independent review and dependency-advisory checks.
Feature combinations and dependency
policy will be defined against the actual project rather than guessed now.

## Hooks and learning

The YAML hooks invoke Cargo directly, without a custom runner:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
pre-commit validate-config
pre-commit run --all-files
```

Hooks run for Rust sources, Cargo manifests/lockfiles, the pinned toolchain, and
hook configuration. `pass_filenames: false` checks the project rather than
individual changed files. Formatting checks do not rewrite code.
Frontend file changes also run the strict TypeScript build and component tests;
install declared dependencies with `npm ci --prefix web` first.
Pre-commit temporarily stashes unstaged tracked changes during normal commits;
checks must not rely on untracked/generated inputs. CI must check a clean candidate.

Install pre-commit outside the repository through its official instructions
(on macOS, `brew install pre-commit`). Register the hook with `pre-commit install`.
This repository's worktrees share the registered hook; each checkout needs the
configuration. Hooks are bypassable and do not replace CI or architectural review.

The [engineering learning loop](engineering-maintenance.md) is not wired into
these hooks. No custom Python runner or AI analysis runs during commits.

## Integration tests

Follow the [mocked contract testing decision](adr/0008-mocked-provider-contract-tests.md).
Provider tests use real adapters against per-test local wiremock servers;
inbound routes use in-process Axum/Tower tests. Inject endpoints and dummy
credentials explicitly. Ordinary tests require no real API keys or paid calls.
These tests will run under `cargo test --workspace --locked` as code is added.
Four integration tests cover health JSON, HEAD, unsupported POST, and unknown
routes. They exercise the real router without opening sockets.

## Implemented servicing flow and types

These diagrams describe the executable code, not the full-claims target
architecture. Start with [the assessor port](../src/assessment.rs), then follow
[the application service](../src/application.rs).

### Startup: choose dependencies once

```mermaid
sequenceDiagram
    participant Main as api.rs (composition root)
    participant Config as Configuration
    participant Provider as KeywordBaseline or Jev
    participant Service as ServicingService
    participant HTTP as Axum Router
    Main->>Config: Read REASSURE_ASSESSOR
    Main->>Provider: Construct selected implementation
    Provider-->>Main: Arc of dyn Assessor
    Main->>Config: Parse routing.json and execution.json
    Config-->>Main: Validated RoutingPolicy and ExecutionLimits
    Main->>Service: new(assessor, policy, limits)
    Service-->>Main: ServicingService
    Main->>HTTP: router_with(service)
    Main->>HTTP: Serve on loopback port 3000
    Note over Main,HTTP: Invalid configuration fails startup; no silent fallback
```

`dyn Assessor` means the concrete implementation is selected at runtime through
the trait. `Arc` shares ownership safely across concurrent requests; it does not
make arbitrary mutation safe. Tokio's `Mutex` protects the run store and its
`Semaphore` bounds admitted work. Provider selection is not repeated in handlers.

### Successful request: wire DTO to validated evidence to response DTO

```mermaid
sequenceDiagram
    actor Customer
    participant UI as React
    participant HTTP as http.rs
    participant Service as application.rs
    participant Domain as domain.rs
    participant Provider as dyn Assessor
    participant Policy as routing.rs
    participant Store as Process-local run store
    Customer->>UI: Enter message
    UI->>HTTP: POST /v1/messages (CustomerMessage JSON)
    HTTP->>Service: submit(message: String)
    Service->>Domain: Message::new(String)
    Domain-->>Service: Result of validated Message or InvalidMessage
    Service->>Service: Acquire permit and check history limit
    Service->>Provider: provenance()
    Provider-->>Service: Provenance (assessor, model, rubric version)
    Service->>Store: Insert OperatorRun with Assessing state
    Service->>Service: Spawn admitted-work task owning permit
    Service->>Provider: assess(reference to Message)
    Provider-->>Service: AssessmentFuture
    Note over Service,Provider: Await supervised worker within configured deadline
    Provider-->>Service: Result of Assessment or AssessmentFailure
    Service->>Policy: plan(reference to Assessment)
    Policy-->>Service: Plan (RoutedIntent signals and Intent tasks)
    Service->>Service: Map to IntentSignal and ServicingTask DTOs
    Service->>Store: Record ReviewRequired or ClarificationRequired
    Service-->>HTTP: Result of CustomerReply or ServiceError
    HTTP-->>UI: HTTP 200 + CustomerReply JSON
    UI-->>Customer: Reply, next steps and status
    Note over Service,Store: Permit released after work completes; history is not durable
```

`AssessmentFuture` is a boxed, pinned asynchronous computation whose result is
`Result<Assessment, AssessmentFailure>`. The same trait call works for the
baseline, Jev and a test assessor. It does not itself select a provider, apply
routing thresholds or grant authority.

### Inside the adapters: different evidence, same validated contract

```mermaid
sequenceDiagram
    participant Service as ServicingService
    participant Baseline as KeywordBaseline
    participant Jev as Jev adapter
    participant Vendor as TypeSafe HTTP API
    participant Domain as Domain constructors
    alt Keyword baseline selected at startup
        Service->>Baseline: assess(reference to Message)
        Baseline->>Baseline: Match explicitly associated intent keywords
        Baseline->>Domain: Assessment::new(KeywordMatch observations, no usage)
        Domain-->>Baseline: Complete, unique intent observations
        Baseline-->>Service: Validated Assessment
    else Jev selected at startup
        Service->>Jev: assess(reference to Message)
        Jev->>Jev: Build typed Request from private question rubric
        Jev->>Vendor: POST /v1/systemone (server-side credentials)
        Vendor-->>Jev: Provider Response DTO
        Jev->>Jev: Check status, response size, model, IDs and answer types
        Jev->>Domain: Probability::new(each Noul value)
        Domain-->>Jev: Finite probability in range 0 to 1
        Jev->>Domain: Assessment::new(YesProbability observations, TokenUsage)
        Domain-->>Jev: Complete, unique intent observations
        Jev-->>Service: Validated Assessment
    end
    Note over Service,Domain: RoutingPolicy applies its threshold afterwards
```

The baseline's `KeywordMatch(bool)` is not a probability. Jev's
`YesProbability(Probability)` preserves its Noul evidence. `Assessment` has
private fields and validates that every supported `Intent` occurs exactly once.
`TokenUsage` is optional: absence is not fabricated zero usage.
The adapter's private `Request`, `Response`, `Answer` and `Usage` types represent
the vendor protocol, not the application API.

### Failure, cancellation and operator inspection

```mermaid
sequenceDiagram
    participant HTTP as HTTP caller
    participant Service as Admitted-work task
    participant Worker as Assessor worker
    participant Store as Process-local run store
    actor Operator
    Service->>Worker: Start assessment under deadline
    alt Provider rejects request or returns invalid data
        Worker-->>Service: AssessmentFailure
    else Deadline expires
        Service->>Worker: abort()
        Service->>Worker: Await cancellation
        Worker-->>Service: Cancelled (or panic)
    else Provider panics
        Worker-->>Service: Tokio JoinError
    end
    Service->>Store: Record Failed + safe failure_code
    Service->>Service: Emit correlated attempt completion and release permit
    alt HTTP caller still connected
        Service-->>HTTP: ServiceError::Assessment with run ID
        HTTP-->>HTTP: Map to HTTP 502 + ApiError
    else HTTP caller disconnected
        Note over Service,Store: Work still records its final result; no response delivery
    end
    Operator->>HTTP: GET /v1/operator/runs
    HTTP->>Store: service.runs()
    Store-->>HTTP: OperatorRun DTOs
    HTTP-->>Operator: States, signals, usage and provenance
```

Invalid input returns 422; exhausted concurrency/history returns 429. Those
rejections do not invoke the assessor or create a run. Malformed JSON/body limits
are handled by Axum before the use case. Provider failures never become keyword
fallback or successful empty assessments. Disconnect survival is process-local:
stopping the server still loses history and interrupts work.

### Type ownership quick reference

| Type | Defined in | Meaning |
| --- | --- | --- |
| `CustomerMessage` | `contracts.rs` | Incoming JSON DTO; its string still needs domain validation |
| `Message`, `Probability` | `domain.rs` | Values with validated construction and private inner data |
| `Intent`, `Evidence`, `Observation` | `domain.rs` | Application category and provider-independent typed evidence |
| `Assessment`, `TokenUsage` | `domain.rs` | Validated complete evidence set and optional usage |
| `Assessor`, `AssessmentFuture`, `Provenance`, `AssessmentFailure` | `assessment.rs` | Provider behavioral port, asynchronous result and provenance/failure contract |
| `RoutingPolicy`, `Plan`, `RoutedIntent` | `routing.rs` | Pure configured decision logic and internal planning result |
| `ServicingService`, `ExecutionLimits`, `ServiceError` | `application.rs` | Use-case coordination, validated limits and transport-independent errors |
| `IntentSignal`, `ServicingTask`, `CustomerReply`, `OperatorRun`, `ApiError` | `contracts.rs` | Persona/API projections, serialized by Serde and exported to TypeScript |

`Intent` is re-exported through `contracts.rs` so the generated TypeScript schema
uses the same categories without moving the domain into the HTTP layer.
`IntentSignal.matched` is a routing result, not raw provider evidence.
Every prepared `ServicingTask` remains `ReviewRequired`; no type above represents
permission to execute a claim or financial action.

## Run the first API slice

```sh
cargo run --bin api --locked
```

In another terminal:

```sh
curl -i http://127.0.0.1:3000/health
cargo test --test health --locked
```

Expect HTTP 200, `content-type: application/json`, and `{"status":"ok"}`.
`GET /health` means the process can serve this route; it does not claim provider
readiness, successful triage, or deployment readiness. HEAD returns the same
status/content type with no body; POST returns 405; unknown routes return 404.
No authentication, consequential claims execution or readiness endpoint exists.
Provider calls are opt-in from the server; health remains process liveness only.

The prototype binds only `127.0.0.1:3000`; an occupied port causes a visible
startup error and nonzero exit, never a silent fallback. Stop with Ctrl+C.
Startup diagnostics and structured assessment-attempt telemetry go to stderr.
Provider/application deadlines and concurrency controls exist; graceful request
draining, alerts and managed telemetry retention remain unimplemented.
Do not expose or deploy this local learning slice as a production service.

`Router` lets the same API run in-process in tests and on a socket in the binary.
`Json<Health>` serializes a typed response; `#[derive(Serialize)]` generates that
serialization. `async` allows I/O to yield to Tokio. `?` propagates failures instead
of panicking; tests also return `Result`, so setup failures fail visibly without
using unwrap. Tower's `oneshot` sends a request through the router, not a fake
health handler.

## Next milestone

The health and servicing/provider slices are implemented. Current boundaries
are recorded in [ADR 0011](adr/0011-provider-port-and-servicing-use-case.md).
Review the code and exercise the UI before expanding into customer/policy tools,
durable workflows or consequential actions.
