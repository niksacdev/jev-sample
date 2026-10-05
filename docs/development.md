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
wiremock will be added with the first provider adapter.

## Repository hygiene

Track the existing Cargo.lock. Ignore generated
target/coverage files and local .env files. Only sanitized .env.example content
may be committed; ignore rules are not secret scanning.
Never put API keys or real claim narratives in fixtures or telemetry.

The library exposes an HTTP router and the `api` binary serves it locally.
Domain behavior and provider adapters are not implemented.
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
Provider tests will use real adapters against per-test local wiremock servers;
inbound routes will use in-process Axum/Tower tests. Inject endpoints and dummy
credentials explicitly. Ordinary tests require no real API keys or paid calls.
These tests will run under `cargo test --workspace --locked` as code is added.
Four integration tests cover health JSON, HEAD, unsupported POST, and unknown
routes. They exercise the real router without opening sockets.

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
No authentication, claim input, provider calls, or readiness endpoint exists yet.

The prototype binds only `127.0.0.1:3000`; an occupied port causes a visible
startup error and nonzero exit, never a silent fallback. Stop with Ctrl+C.
Startup diagnostics go to stderr; workflow telemetry, graceful request draining,
connection/deadline controls, alerts, and telemetry retention are not implemented.
Do not expose or deploy this local learning slice as a production service.

`Router` lets the same API run in-process in tests and on a socket in the binary.
`Json<Health>` serializes a typed response; `#[derive(Serialize)]` generates that
serialization. `async` allows I/O to yield to Tokio. `?` propagates failures instead
of panicking; tests also return `Result`, so setup failures fail visibly without
using unwrap. Tower's `oneshot` sends a request through the router, not a fake
health handler.

## Next milestone

Architecture is approved in [the application ADR](adr/0007-single-package-api-and-evaluation.md).
Pause after validating this health slice. Next define typed intake/routing
contracts and deterministic fixtures before provider integration.
