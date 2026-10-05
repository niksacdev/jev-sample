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

The minimal Cargo library scaffold exists with no domain behavior or dependencies.
Configured lints forbid project unsafe code and reject unwrap/expect.
Independent CI is required before the first behavioral slice is accepted.
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

## Next milestone

Agree the backend boundaries, API contracts, provider adapters, and first learning
slice. Record selected architecture/frameworks in ADRs before implementation.
