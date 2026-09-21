# Zumbra — project contract

> Read `docs/PLAN.md` first. It says what we are building, in what order, and what is cut.
> `docs/AUDIT-2026-09-20.md` says why the base code cannot be trusted with an agent yet.
> Update the plan's decision table before ending any session that changes state.

**This repo is a hard fork** of `atmospherelabs-dev/zipher-app` (MIT) at `feat/ironwood`
`2e7ccd3`. We do not contribute upstream. Upstream history and both MIT copyright lines stay in
`LICENSE.md` forever. Every upstream name, host, key and affiliate ID is replaced in Phase 1.

## Rules

1. **Policy before signer, on every path, fail closed.** `check_proposal` runs before any seed or
   spending key is read, on every code path that can produce a signature. A path that cannot be
   gated is deleted. Missing, unparseable, zero or wrong-typed policy values refuse to spend.
2. **No LLM authorises a spend.** Approval above threshold is a protocol on a channel the agent
   cannot call: the operator's CLI as a different OS user, or an operator-held secret the agent
   never sees. An MCP tool that approves is a bug, not a feature.
3. **Destinations never come from untrusted text.** Not from memos, fetched pages, swap quotes,
   model memory or conversation history. Allowlist, or a human in the current turn.
4. **Keys stay on the machine.** Never to stdout by default, never on argv, never in a tool result,
   never in a log line. Passphrases are zeroised. Empty passphrase needs an `--unsafe-` flag.
5. **Untrusted strings are sanitised before they reach a tool result:** C0/C1 controls, ANSI,
   OSC 8, OSC 52 stripped; length capped by characters, never by bytes.
6. **Nothing leaves the machine that the operator did not configure.** No baked-in third-party
   hosts, affiliate keys or telemetry. Alternates and swaps are opt-in.
7. **Every claim on the site maps to a command that works.** Tool counts are generated from code.
8. **Tests are integration tests against real processes.** No mocks. Security fixes get a
   mutation check: break it, watch the test fail, restore.
9. **Scope is `docs/PLAN.md`.** Cut items are not pulled forward because they are more fun than
   finishing the current phase.

## Recurring defect classes to check before calling anything done

Taken from this codebase and its sibling repos, all of which shipped at least once:

- A test that passes while asserting nothing (`all([])`, loop over an empty list, `if exists:`).
- A scalar where a collection is expected (a `str` allowlist becomes a substring test).
- Fail-open on zero, missing or wrong-typed config.
- Byte-index slicing of UTF-8.
- Work reported complete but never committed. `git log -1` and `git status --short` are evidence.

## Git

Remote: `https://github.com/AIEngineerX/zumbra.git` · Local: `V:\zumbra`
Commit as: `AIEngineerX <195990077+AIEngineerX@users.noreply.github.com>`
Do not use the `github-griffin` identity here. `main` is the trunk; short-lived branches per slice.
`upstream` remote points at atmospherelabs-dev for reference only. Never push to it.
