#!/usr/bin/env bash
# Point this clone at the repo's hooks. Run once after cloning.
set -euo pipefail
cd "$(dirname "$0")/.."
chmod +x .githooks/* scripts/*.sh 2>/dev/null || true
git config core.hooksPath .githooks
echo "hooks installed: core.hooksPath = $(git config core.hooksPath)"
command -v gitleaks >/dev/null 2>&1 || echo "note: gitleaks is not on PATH; the pre-commit hook will run pattern checks only."
