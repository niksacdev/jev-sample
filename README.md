# ZipClaim
AI-native insurance support: plan the work, evaluate bounded decisions, and keep people in control.

[Development setup](docs/development.md)

## Project decisions

ZipClaim explores autonomous insurance servicing, with separate customer,
employee and operator views. The first executable slice delegates a synthetic
customer message from React to a Rust servicing coordinator, which assesses
intents and prepares review-only tasks. Rust can use a keyword baseline or Jev.
No claim, policy, customer or payment changes are executed.

The default **Customer** view uses the AI-native slice: OpenAI Responses formulates
a bounded base plan, local read-only tools supply synthetic facts, and an injected
`DecisionProvider` evaluates atomic Predicate/Choice/Score judgments. Jev and an
optional OpenAI Decisions adapter implement the same port. Code supports only the
synthetic-reference check, not arbitrary insurance judgments. One base plan is
shared across selected providers; independent continuations can diverge.

Configure `OPENAI_API_KEY` and an explicit `OPENAI_PLANNER_MODEL` locally to enable
planning and conversational replies. Configure `OPENAI_DECISION_MODEL` separately
to enable the Decisions provider; no model name is silently selected. Account
availability and successful live calls are **not verified**. The old
intent-assessment experiment remains available in **Assessment lab** for comparison;
its disabled LLM option is not the new workflow planner.

**Employee** shows authenticated paused journeys and scoped clarification/review.
**Operator** shows matched-provider evidence and durable comparison inspection.
Keys, protected records and review drafts clear when changing persona; customer
public results persist. Legacy role inspection is explicitly labelled and
collapsed. Tabs are convenience views, not authorization or insurer assignments.

The workflow dashboard records protected SQLite comparisons, frozen plans,
versioned policy/questions, exact decision-input hashes, provider/model, normalized
answers, usage (unknown stays unknown), latency, failures and end-to-end events.
Restart preserves records and marks unfinished runs interrupted without replay.
Matched question/version/input hashes support decision comparisons; resumed flows
are marked divergent. Completion/agreement is not accuracy or business value.
M01–M14 still require their independent operational/label/finance evidence.

Both clarification and employee-review resumes require `REASSURE_OPERATOR_KEY`
in this local preview, to prevent unauthenticated run hijacking. Enter a fictional
clarification or review note in the workflow dashboard. Clarification asks the
planner for a new bounded plan; employee review applies only to the currently
paused task and expected plan revision, never future judgments or real business authority. Customer-scoped
identity, a real employee queue and insurer integrations are not implemented.

SQLite defaults to ignored `.local/workflows.sqlite3`. The Unix directory/file
must be private (700 / 600), not symlinks. `REASSURE_WORKFLOW_DB` can select another
private path. Limits are in `config/workflow.json`: 200 comparisons, 8 MiB database,
2 concurrent comparisons, 45-second deadline, 8 tasks per plan, 24 total task
steps per run and at most two resumes. Storage exhaustion fails explicitly;
nothing is auto-deleted. Stop the API before explicitly archiving the database
and its journal to reset retention. Do not restart merely to clear these records.
Normalized decision snapshots, not raw vendor response bodies, are retained.

## Run the local UI

Use Node 24+ and the pinned Rust toolchain. In two terminals:

```sh
cargo run --bin api --locked
```

```sh
npm ci --prefix web
npm run dev --prefix web
```

Open **http://127.0.0.1:5173**. The frontend proxies application requests to the loopback Rust API; it never
calls models directly. The Assessment lab can run the same message against
Code (the limited keyword baseline) and Jev in parallel. Jev appears when its
server-side key is configured. The LLM option remains disabled until its
provider/model is selected. Each result stays separate; failures are not
substituted with another provider's output.

Create the local environment file and fill it locally:

```sh
cp .env.example .env
```

Add your `TYPESAFE_API_KEY` and a private `REASSURE_OPERATOR_KEY` of at least
32 bytes to `.env`; run the API from the repository root so it loads that file.
The operator key unlocks workflow inspection/resume, and raw request/response
inspection in the Operator view's legacy Assessment lab section.
The browser keeps the key only in memory, and the API never logs it. Do not put
either key in chat, frontend configuration, source, or shell history. The file is
ignored by Git. Jev is disabled if its key is absent or blank; a nonempty but
invalid key will fail authentication at the vendor. Provider failures never silently fall
back to Code. Live calls may incur vendor charges.

Rust structured logs go to stderr. OpenTelemetry spans are exported locally to
stdout and include the HTTP, application and Jev assessment spans. `RUST_BACKTRACE=1`
enables Rust panic backtraces in the API process diagnostics. Narratives,
credentials and raw provider bodies are excluded from logs and spans; raw exchanges
are kept only in bounded process memory and returned by the authenticated operator
endpoint. The Operator view also presents a sanitized per-run execution trace
(admission, assessment and routing stages). Restart clears in-memory run data.
Application-deadline and worker-panic failures have no retained raw exchange;
their failure code and sanitized trace remain visible.

Only submit synthetic data. The Data Protection agreement is acknowledged once per
browser-tab session before potential external inference; its versioned acknowledgement
survives page refreshes in session storage. Editing a message, selecting another provider
or switching personas does not ask again. No text or key is stored in session storage.
If storage is unavailable, the UI explicitly reports that acknowledgement lasts only
until this page closes. No live inference was needed for tests.
Account access, current terms and actual live Jev behavior must be verified by
the user before relying on results.

This is a single local synthetic workspace: persona tabs are not authentication,
legacy assessment metadata history is capped at 100 runs and lost on restart, and MCP, RAG,
durable claims workflows and real customer systems are not implemented.
The full-claims [design mockup](docs/design/intake-workbench.html) remains a
separate simulation, not a representation of completed backend capabilities.
The top **Experimental Preview** notice explains the experiment, fictional-data
requirement, configured-provider disclosure (OpenAI, Jev or future OSS integrations)
and local storage. Customer-facing assessment results are descriptive and do not
imply that an insurer action has been executed or assigned to an employee.
The ZipClaim identity uses crops of the supplied ZC artwork, its orange/pink/violet
palette and a white default background. The brand tagline "File fast. Settle faster."
is aspirational, not a measured settlement result. Optional dark mode is explicit
(`?scoutTheme=dark`), not automatically selected from OS preferences.
**Geek Mode** is a global toggle: neon terminal theme, original ZipClaim logo,
no hero/marketing blocks, and the claim test
console immediately below the preview notice. Turn it off to restore white branding.
**Submit a Claim** is the preview's customer action, not actual insurer submission.
**Let's connect ZipClaim** collects missing connections directly in the chat:
a masked **ZipClaim token**, masked OpenAI/Jev API keys, and an explicit
**OpenAI model name** textbox. Already configured connections do not ask for keys
again and cannot be replaced. Setup keeps credentials only in API memory until
restart, never writes `.env`, and makes no inference or account-verification
call. **Agree and save** incorporates the Data Protection acknowledgement, then
automatically refreshes availability. Submission stays disabled until both OpenAI
and Jev are configured and acknowledgement is completed. The claim draft is preserved.
Use **Refresh availability** to recover from a lost setup response or another
operator tab. A failed refresh never unlocks submission; retry the availability
check without resending saved credentials. Restart to change configured connections.
The claim chat and Assessment lab share `web/src/claimSamples.json`: three
categories of fictional, realistic questions. Category buttons fill the message
without submitting it; repeated clicks cycle through that category's examples.
These are original, resolution-oriented development examples informed by the
[Galileo insurance servicing scenario](https://huggingface.co/datasets/galileo-ai/agent-leaderboard-v2/viewer/adaptive_tool_use/insurance?row=0)
referenced in ADR0010, not verbatim benchmark rows or real customer records.
Runtime configuration, model availability and recorded events appear in side panes.
Assessment lab animates the icon while a request is processing. Raw provider payloads remain
available only in the authenticated Operator view.

### Live instrumentation

Geek Mode embeds [xterm.js](https://xtermjs.org/) 6.0.0 and its official FitAddon
0.11.0 (MIT). Connect **Live logs** with the local operator key to watch actual
API request, planner/replan/reply, tool, decision-attempt and failure events as
Rust emits them—even before the originating request finishes. Separate read-only
terminals show requests, model calls and the combined execution log. No shell or
keyboard input is connected. Plain metadata is control-character stripped.

`GET /v1/operator/telemetry` streams authenticated SSE over the Vite proxy, with
Bearer credentials in a header, never the URL. Four concurrent streams, 15-minute
sessions, a 256-entry process ring and explicit slow-client gaps bound work.
Initial replay is only the recent ring; older events are not promised. Streams
do not automatically reconnect. Turning Geek Mode off or changing persona
aborts the connection and clears protected client logs/key. The durable workflow
events remain the inspection/audit source, not this diagnostic stream.
Only an allowlist of structured fields/events is forwarded—no arbitrary stdout,
prompts, narratives, raw model bodies, environment values or keys. Live vendor
operation is still unverified; ordinary tests use mocks and local baseline work.

For configuration compatibility, Rust crate/module names `jev-sample` / `jev_sample`
and `REASSURE_*` environment variables remain unchanged. They are technical
identifiers, not the product name. No existing `.env` needs rewriting.

## Checks and Rust learning

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo audit --deny warnings
npm run build --prefix web
npm test --prefix web
npm audit --prefix web
cd web && PLAYWRIGHT_SKIP_BROWSER_GC=1 npx --no-install playwright install chromium && npm run e2e
```

Browser E2E tests start their own baseline API and UI, so stop existing servers
on ports 3000/5173 before running them. They never call Jev. Browser downloads
live in Playwright's external cache; screenshots/results are ignored artifacts.

`src/contracts.rs` is the API type authority. Run `npm run contracts --prefix web`
after changing it; a Rust test rejects stale TypeScript bindings.
Read the Rust backend in this order:

[Sequence diagrams and type walkthrough](docs/development.md#implemented-servicing-flow-and-types)
trace startup, successful requests, adapter conversion, failures and inspection.

| File | Responsibility |
| --- | --- |
| `src/domain.rs` | Validated message/probability/assessment values and distinct keyword versus probabilistic evidence |
| `src/assessment.rs` | `Assessor` trait, provenance and typed failures; no concrete provider selection |
| `src/routing.rs` | Pure planning using validated, versioned `config/routing.json` |
| `src/baseline.rs` | Keyword comparator with explicit intent-to-keyword definitions |
| `src/jev.rs` / `src/rubric.rs` | Typed vendor HTTP mapping and versioned question definitions; no routing threshold |
| `src/application.rs` | Shared servicing use case, bounded execution, run lifecycle and customer/operator projections |
| `src/http.rs` | Axum extraction, response/status mapping and browser-origin checks only |
| `src/bin/api.rs` | Composition root: configuration, provider construction and socket startup |

The binary injects an `Arc<dyn Assessor>` into `ServicingService`; swapping the
baseline, Jev or a test assessor does not change HTTP or workflow code. The
object-safe asynchronous trait uses standard `Future`/`Pin` without an additional
macro framework. Keyword evidence is never represented as a probability.
`tests/workflow.rs` proves provider interchangeability, policy injection,
failure propagation, capacity, cancellation, deadlines and panic handling.
Source-boundary regression checks flag selected forbidden dependency references;
they are lightweight guards, not compiler-enforced crate isolation.
Provider tests use local wiremock servers and independent request fixtures.

`REASSURE_ROUTING_POLICY=/absolute/path/to/routing.json` can select a policy file
at startup. Invalid/unknown/missing values fail startup; changing a threshold
requires a new policy version and relevant reevaluation. The application-owned
capacity/history/deadline defaults are in `config/execution.json` and are validated
at startup. These settings do not grant financial authority.

Axum owns JSON extraction, body limits, state and routing; Serde owns wire
serialization; reqwest owns connection/TLS, request timeout and redirects; Tokio
owns synchronization, task execution and deadlines; Tower exercises the HTTP
service in tests. The application retains admission control because its permit
must cover work after a browser disconnect, not merely the HTTP future.
Once admitted, bounded work completes and records its outcome even if the
caller disconnects. It is still process-local, not crash-durable.

- [Jev capabilities and limitations](docs/jev-capabilities.md)
- [Proposed product scope and customer problems](docs/product-scope.md)
- [Proposed product requirements and acceptance](docs/product-spec.md)
- [Persona jobs and reference journeys](docs/persona-journeys.md)
- [Proposed full-claims scorecard and insurer/vendor economics](docs/metrics.md)
- [Approved comparison design](docs/evaluation-design.md)
- [Approved application architecture](docs/adr/0007-single-package-api-and-evaluation.md)
- [Engineering standards](engineering-standards.md)
- [Harness working agreement](AGENTS.md)
- [Engineering learning loop](docs/engineering-maintenance.md)
- [Architecture decision records](docs/adr/README.md)
- [Active tasks: GitHub issues](https://github.com/niksacdev/zipclaim/issues)
- [Historical milestone task list (deprecated)](docs/task-list.md)

Documents distinguish approved decisions from proposals. Work pauses for discussion
after each logical milestone.
