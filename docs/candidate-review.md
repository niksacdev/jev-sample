# Independent candidate review

## Prepare an exact staged candidate

Stage only intended changes and inspect `git diff --cached`. From the repository:

```sh
(
set -eu
git diff --cached --check
candidate=$(git write-tree)
baseline=$(git rev-parse HEAD)
snapshot=$(mktemp -d "${TMPDIR:-/tmp}/jev-review.XXXXXX")
git archive "$candidate" -o "$snapshot.tar"
tar -xf "$snapshot.tar" -C "$snapshot"
git diff --binary "$baseline" "$candidate" > "$snapshot.diff"
printf 'candidate=%s\nbaseline=%s\nsnapshot=%s\n' "$candidate" "$baseline" "$snapshot"
)
```

`git write-tree` identifies the index, not unstaged/untracked inputs. The exported
snapshot has no Git metadata or production .env files unless someone improperly
tracked them. Inspect the packet before sending it; never provide secrets or
held-out datasets. An archive is not an execution sandbox. Dependencies/build
scripts can execute code and checks can access the network; only approved
synthetic checks may run, with no production secrets in the execution environment.

Start a separate-context reviewer using the
[pre-committer profile](../.github/agents/pre-committer.agent.md). Supply the printed
identities, snapshot/diff paths, scope, acceptance criteria, standard version,
relevant ADRs, approved commands, and missing controls. If native profile
discovery is unavailable, explicitly load the profile into a fresh agent context
and disclose that fallback. Do not claim automatic Git-hook AI execution.

Approved routine commands, executed from the snapshot:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo audit --deny warnings
```

Install pinned cargo-audit 0.22.2 outside review beforehand. Audit fetches the
public RustSec advisory database; its result depends on the database at run time.
Record the database revision when reporting results. Audit is not a complete
license/provenance or supply-chain assessment; review new dependencies separately.
No ignored advisories are configured. Warnings also fail the gate.

Re-export and rerun after any candidate changes. Before committing, ensure
`git write-tree` still matches the reviewed tree. Preserve the review result in
the milestone/PR evidence. Remove only the exact printed temporary snapshot,
archive, and diff paths after review; do not delete a parent directory.

## CI and limits

[Rust quality](../.github/workflows/rust.yml) runs the same checks on pull requests,
main pushes, or manual dispatch. It uses read-only repository permission, no
persisted checkout credential, no provider secrets, a pinned checkout action,
and the repository-pinned Rust toolchain. Its stable job name is `Rust checks`.

Branch protection must separately require that job; the workflow does not
configure protection or enforce independent human/agent review. Hosted execution
must be observed on an actual run before being reported as passing.
AI review is an explicit milestone/PR invocation, not an automatic commit hook.
Learning proposals remain a separate pending integration.
