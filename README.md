# jev-sample
Rust based sample to demonstrate use of Jev model in FSI applications

[Development setup](docs/development.md)

## Project decisions

Reassure explores autonomous insurance servicing, with separate customer,
employee and operator views. The first executable slice delegates a synthetic
customer message from React to a Rust servicing coordinator, which assesses
intents and prepares review-only tasks. Rust can use a keyword baseline or Jev.
No claim, policy, customer or payment changes are executed.

## Run the local UI

Use Node 24+ and the pinned Rust toolchain. In two terminals:

```sh
cargo run --bin api --locked
```

```sh
npm ci --prefix web
npm run dev --prefix web
```

Open **http://127.0.0.1:5173**. The frontend proxies application requests to the
loopback Rust API; it never calls models directly. Default mode is an explicitly
limited keyword baseline. The customer screen prepares tasks; the employee
screen reads them; the operator screen inspects real run metadata.

To experiment with Jev, stop the API and restart it with:

```sh
REASSURE_ASSESSOR=jev cargo run --bin api --locked
```

Set `TYPESAFE_API_KEY` in the server's environment first using your local secret
manager or a non-echoing terminal prompt. Do not put it in frontend configuration,
source, chat, or a command saved in shell history. The app does not load `.env`
files. Missing/invalid configuration fails startup; provider failures never
silently fall back to the baseline. Live calls may incur vendor charges.

Only submit synthetic data. Every message submission requires acknowledgement
before potential external inference. No live inference was needed for tests.
Account access, current terms and actual live Jev behavior must be verified by
the user before relying on results.

This is a single local synthetic workspace: persona tabs are not authentication,
metadata history is capped at 100 runs and lost on restart, and MCP, RAG,
durable claims workflows and real customer systems are not implemented.
The full-claims [design mockup](docs/design/intake-workbench.html) remains a
separate simulation, not a representation of completed backend capabilities.

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
`src/agent.rs` contains pure validation, baseline and planning.
`src/jev.rs` owns the real provider HTTP contract and validates every answer.
`src/http.rs` coordinates bounded execution and returns distinct customer and
operator projections. Provider tests exercise this adapter against local
wiremock servers with dummy keys, not paid inference.

- [Jev capabilities and limitations](docs/jev-capabilities.md)
- [Approved product scope](docs/product-scope.md)
- [Product discovery draft and clickable mockup](docs/product-spec.md)
- [Customer, employee and operator journey specifications](docs/persona-journeys.md)
- [Approved metrics and targets](docs/metrics.md)
- [Approved comparison design](docs/evaluation-design.md)
- [Approved application architecture](docs/adr/0007-single-package-api-and-evaluation.md)
- [Engineering standards](engineering-standards.md)
- [Harness working agreement](AGENTS.md)
- [Engineering learning loop](docs/engineering-maintenance.md)
- [Architecture decision records](docs/adr/README.md)
- [Milestone task list](docs/task-list.md)

Documents distinguish approved decisions from proposals. Work pauses for discussion
after each logical milestone.
