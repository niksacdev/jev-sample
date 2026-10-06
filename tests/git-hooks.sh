#!/usr/bin/env bash
set -euo pipefail

root=$(git rev-parse --show-toplevel)
mkdir -p "$root/target"
scratch=$(mktemp -d "$root/target/git-hook-test.XXXXXX")
trap 'rm -rf "$scratch"' EXIT
mkdir -p "$scratch/bin" "$scratch/repo"
export HOOK_TEST_LOG="$scratch/commands"
cat > "$scratch/bin/cargo" <<'EOF'
#!/usr/bin/env bash
printf 'cargo %s\n' "$*" >> "$HOOK_TEST_LOG"
if [ "${HOOK_TEST_FAIL:-}" = cargo ]; then exit 7; fi
EOF
cat > "$scratch/bin/npm" <<'EOF'
#!/usr/bin/env bash
printf 'npm %s\n' "$*" >> "$HOOK_TEST_LOG"
if [ "${HOOK_TEST_FAIL:-}" = npm ]; then exit 8; fi
EOF
chmod +x "$scratch/bin/cargo" "$scratch/bin/npm"
export PATH="$scratch/bin:$PATH"
cd "$scratch/repo"
git init -q
git config user.name "Hook test"
git config user.email "hook-test@example.invalid"
git config core.hooksPath "$root/.githooks"
printf 'initial\n' > README.md
git add README.md
git commit -qm "Documentation only"
test ! -e "$HOOK_TEST_LOG"

mkdir -p src web/src config
printf 'rust\n' > 'src/file with spaces.rs'
git add src
git commit -qm "Rust"
printf '%s\n' \
    'cargo fmt --all -- --check' \
    'cargo clippy --workspace --all-targets --locked -- -D warnings' \
    'cargo test --workspace --locked' > "$scratch/expected"
diff -u "$scratch/expected" "$HOOK_TEST_LOG"

: > "$HOOK_TEST_LOG"
printf 'typescript\n' > web/src/app.ts
git add web
git commit -qm "Frontend"
printf '%s\n' 'npm run build --prefix web' 'npm test --prefix web' > "$scratch/expected"
diff -u "$scratch/expected" "$HOOK_TEST_LOG"

: > "$HOOK_TEST_LOG"
printf '{}\n' > config/routing.json
printf 'contract\n' > web/src/contracts.ts
git add config web
"$root/.githooks/pre-commit"
test "$(wc -l < "$HOOK_TEST_LOG" | tr -d ' ')" = 5

: > "$HOOK_TEST_LOG"
printf 'unstaged\n' >> 'src/file with spaces.rs'
if "$root/.githooks/pre-commit"; then
    echo "Hook incorrectly accepted unstaged tracked changes" >&2; exit 1
fi
test ! -s "$HOOK_TEST_LOG"
git add src
if HOOK_TEST_FAIL=cargo git commit -qm "Must fail"; then
    echo "Hook incorrectly accepted failed Cargo check" >&2; exit 1
fi
test "$(wc -l < "$HOOK_TEST_LOG" | tr -d ' ')" = 1

: > "$HOOK_TEST_LOG"
if HOOK_TEST_FAIL=npm "$root/.githooks/pre-commit" --all-files; then
    echo "Hook incorrectly accepted failed npm check" >&2; exit 1
fi
test "$(wc -l < "$HOOK_TEST_LOG" | tr -d ' ')" = 4

: > "$HOOK_TEST_LOG"
"$root/.githooks/pre-commit" --all-files
test "$(wc -l < "$HOOK_TEST_LOG" | tr -d ' ')" = 5
echo "Git hook regression checks passed"
