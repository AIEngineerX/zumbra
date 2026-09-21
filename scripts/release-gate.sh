#!/usr/bin/env bash
# One command that must pass before any push. Composes the checks that exist; stops at the first
# real failure. It does not try to be helpful about partial failures.
set -euo pipefail
cd "$(dirname "$0")/.."

fail() { echo "GATE FAILED: $1" >&2; exit 1; }

[ -z "$(git status --porcelain)" ] || fail "working tree is dirty"

command -v gitleaks >/dev/null || fail "gitleaks not installed"
gitleaks git . --no-banner --redact >/dev/null || fail "gitleaks found secrets in history"

# Tracked secret-shaped files. Same pattern as .githooks/pre-commit and ci.yml; keep them aligned.
bad=$(git ls-files | grep -iE '(^|/)\.env($|\.)|\.pem$|\.p12$|\.jks$|\.keystore$|key\.properties$|keypair.*\.json$|\.mnemonic$|\.seed$|(^|/)policy\.toml$' | grep -v '\.example' || true)
[ -z "$bad" ] || fail "secret-shaped file is tracked: $bad"

# Rust checks run only when cargo is present. Absent cargo is a loud notice, never a silent pass.
if command -v cargo >/dev/null; then
  ( cd rust
    cargo fmt --all --check >/dev/null 2>&1 || echo "NOTICE: rustfmt does not pass on the inherited tree yet; non-blocking here as in CI"
    cargo clippy --locked --workspace --all-targets -- -D warnings || fail "clippy"
    if command -v cargo-deny >/dev/null; then cargo deny check || fail "cargo deny"; else echo "NOTICE: cargo-deny not installed; CI runs it."; fi
    cargo test --locked --workspace || fail "cargo test"
  )
else
  echo "NOTICE: cargo is not installed on this machine. Rust checks were NOT run. CI runs them."
fi

# Pre-public audit: blocking only once the repo is public. While private it reports.
VISIBILITY=$(gh repo view --json isPrivate --jq '.isPrivate' 2>/dev/null || echo "unknown")
case "$VISIBILITY" in
  false) bash scripts/pre-public-audit.sh || fail "repo is PUBLIC and the pre-public audit found exposure" ;;
  true)  echo "NOTICE: repo is private; pre-public audit is reported, not enforced:"; bash scripts/pre-public-audit.sh || true ;;
  *)     echo "NOTICE: could not determine repo visibility. Run scripts/pre-public-audit.sh by hand before publishing." ;;
esac

echo "ALL GATES PASSED"
