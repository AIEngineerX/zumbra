#!/usr/bin/env bash
# Pre-public audit. Run before this repo is made public, and in CI thereafter.
#
# Why: the repo was forked and developed privately. Its docs legitimately name the parent project,
# the parent's company, their hosts and affiliate IDs, and the owner's local drive paths. None of
# that is secret, but a public README that reads as a takedown of the parent, or a code comment
# that still says the parent's name, is not the repo we want to ship. A one-time cleanup rots;
# this is a gate, so it cannot regress silently.
#
# Implementation note: excludes are git pathspecs, not a file list. The first version expanded
# `git ls-files` onto the command line, which exceeds the argument limit on Windows; git grep then
# failed silently and the script printed PASS on a tree full of hits. A check that cannot fail is
# not a check. The self-test at the bottom guards against that regressing.
#
# Exit 0 = clean, 1 = findings (every hit printed as file:line).
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1

FINDINGS=0

# Files where naming the parent is the point: license, notices, the audit, captured references,
# the plan and contract that explain the fork, and this script.
EXCLUDE=(
  ':!LICENSE.md' ':!NOTICE'
  ':!README.md' ':!SECURITY.md' ':!CONTRIBUTING.md' ':!CLAUDE.md' ':!scripts/pre-public-audit.sh'
  ':!ows-core'
)

report() {
  local label="$1" pattern="$2" guidance="$3"
  local hits
  hits=$(git grep -inE "$pattern" -- . "${EXCLUDE[@]}" 2>/dev/null || true)
  if [ -n "$hits" ]; then
    FINDINGS=1
    echo ""
    echo "=============================================================="
    echo "FINDING: $label ($(printf '%s\n' "$hits" | wc -l | tr -d ' ') hits)"
    echo "GUIDANCE: $guidance"
    echo "--------------------------------------------------------------"
    printf '%s\n' "$hits" | head -40
    [ "$(printf '%s\n' "$hits" | wc -l)" -gt 40 ] && echo "... (truncated; run git grep for the full list)"
  fi
}

# Self-test: the tool must be able to find a known string, or every PASS below is meaningless.
if ! git grep -q -- 'Zumbra' README.md 2>/dev/null; then
  echo "SELF-TEST FAILED: git grep cannot find a known string; refusing to report PASS." >&2
  exit 1
fi

echo "Pre-public audit: scanning tracked files, excluding license/notice/audit/reference/plan."

report "Parent project names in code, config, or copy" \
  'atmosphere ?labs|cipherscan|cipherpay|zipher' \
  "The rename phase replaces every crate, binary, env var, host and skill name. These should only survive in LICENSE.md and NOTICE."

report "Parent hosts and affiliate identifiers" \
  'atmospherelabs\.dev|cipherscan\.app|cipherpay\.app|zipher\.(app|to)|cipherscan\.near|partner_id' \
  "No baked-in third-party host or affiliate key ships. Operator-configured endpoints only."

report "Owner's local paths" \
  'V:\\|V:/|C:\\Users|/v/[a-z]|footb' \
  "Use relative paths or a neutral description."

report "Secrets and key material" \
  '(AIza[0-9A-Za-z_-]{30,}|sk-[A-Za-z0-9]{20,}|gho_|ghp_|-----BEGIN [A-Z ]*PRIVATE KEY|eyJ[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{20,}\.)' \
  "Hard stop. Rotate first, then purge from history. Deleting in a new commit is not enough."

echo ""
if [ "$FINDINGS" -eq 0 ]; then
  echo "PASS: no findings in tracked files."
else
  echo "FINDINGS ABOVE: this repo is NOT ready to be public."
  echo "Git history is also public. Purging a file in a new commit leaves earlier revisions readable."
fi
exit "$FINDINGS"
