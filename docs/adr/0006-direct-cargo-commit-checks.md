# ADR 0006: Run Cargo directly through pre-commit

Date: 2026-10-05
Status: Accepted

## Decision

Use local YAML hooks for non-mutating formatting, Clippy, and locked Cargo tests.
No custom checker or third-party Rust hook wrapper is needed. Start with a minimal
dependency-free library scaffold, not an API framework. Cargo.lock is tracked.

## Rationale and alternatives

The user requested established Rust pre-commit practice. Direct commands use the
pinned Rust environment without introducing a second implementation language or
wrapper dependency. Custom learning analysis is a separate pending integration.

## Consequences and evidence

All three hooks passed on the initial scaffold. The library has no behavior or
tests yet; passing checks do not establish domain correctness.
The user explicitly approved installing the hook shared with the main checkout
and other worktrees. Checkouts without the YAML file must obtain it before
committing with this hook. CI and independent architecture review remain pending.
