# ADR 0012: Repository-managed native Git quality hook

Date: 2026-10-06
Status: Accepted
Supersedes: [ADR 0006](0006-direct-cargo-commit-checks.md) and the hook-manager
portion of [ADR 0005](0005-evidence-driven-instruction-maintenance.md).

## Context and decision

The user requested Git hooks using only the existing developer toolchains.
Use an executable, versioned `.githooks/pre-commit` Bash script registered with
`git config --local core.hooksPath .githooks`.
Keep direct Cargo formatting/Clippy/tests and npm build/tests with staged-path
selection, fail-fast execution and an explicit `--all-files` mode.

## Alternatives and consequences

An additional hook manager adds another developer dependency and is unnecessary
for this small mixed-language repository. The native hook uses tools already needed here.
It deliberately rejects unstaged tracked changes when checks run rather than
implementing risky automatic stashing/restoration.

The setting is normally shared across worktrees; each checkout must contain the
hook. Existing files in Git's default hooks directory are left untouched but are
no longer selected by Git. Shared machine packages are not uninstalled.
Local hooks are bypassable and are not a clean-checkout guarantee: untracked
inputs can still influence builds. Hosted clean-checkout CI remains authoritative.
AI pre-committer review and engineering learning remain separate controls.

## Verification and follow-up

`bash tests/git-hooks.sh` exercises native Git invocation, documentation-only
skipping, filenames with spaces, Rust/frontend selection, combined configuration
and contract changes, unstaged-change rejection, failure propagation and
all-files execution with synthetic command stubs. CI runs this regression check.
Run `.githooks/pre-commit --all-files` with real toolchains for the candidate.
