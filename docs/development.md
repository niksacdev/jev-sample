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
No framework or application dependencies are selected yet.

## Repository hygiene

Track Cargo.lock when the application Cargo project is created. Ignore generated
target/coverage files and local .env files. Only sanitized .env.example content
may be committed; ignore rules are not secret scanning.
Never put API keys or real claim narratives in fixtures or telemetry.

No Cargo manifest exists yet, so application build/test commands cannot run.
The first scaffold must add compilation, formatting, linting, tests, and independent
CI before its behavioral slice is accepted. Feature combinations and dependency
policy will be defined against the actual project rather than guessed now.

## Hooks and learning

The [engineering learning loop](engineering-maintenance.md) describes the pending
pre-commit integration. Agent/skill files are proposal contracts, not installed
checks. No Python application or custom Python hook runner is part of this repo.
The ignored .venv-hooks directory is a leftover developer-tool installation;
no Git hook is registered by this setup.

## Next milestone

Agree the backend boundaries, API contracts, provider adapters, and first learning
slice. Record selected architecture/frameworks in ADRs before implementation.
