# Plan — a privacy-first Zcash wallet for humans and agents

**Owner:** AIEngineerX · **Started:** 2026-09-20 · **Name:** TBD (starts with Z; decided in §5)
**Base:** `atmospherelabs-dev/zipher-app` at `feat/ironwood` `2e7ccd3` (2026-09-11), MIT.
We are not contributing upstream. This is a hard fork that keeps upstream history and both MIT
copyright lines, and replaces every upstream default, name and key.

## 1. What this is

Two deliverables, in this order:

1. **The agent wallet.** A headless Zcash wallet (Rust engine + CLI + MCP server) that an AI agent
   can hold and spend from, where the spending policy and the human approval are enforced by
   something the agent cannot edit or call. Shielded by default, local keys, no upstream
   affiliates, no phone app.
2. **The site.** One static page in the shape of `docs/reference/zipher-site/`: a single "ask"
   input over a local keyword knowledge base, a human/agent toggle, our mark, our links. No LLM,
   no analytics, no backend.

Not in scope and not to be pulled forward: the Flutter mobile app, Polymarket, FROST shared
wallets, EVM swaps, the CipherPay merchant stack, a token, hosting other people's wallets.

## 2. Why fork, and why this base

- `main` does not compile, predates the Ironwood network upgrade that mainnet has already
  activated, and lacks the September security fixes. See `docs/AUDIT-2026-09-20.md`.
- `feat/ironwood` compiles per upstream's remediation note, syncs post-Ironwood, removes the
  self-unlock and self-approve MCP tools, and adds durable spend reservations.
- The remaining gaps (max-send bypass, policy-less spend paths, fail-open policy semantics,
  no privilege separation, unsanitised memos, swap identity leak, unsigned installer) are ours.

## 3. Phases and done-criteria

No dates. Each phase ends when its done-criterion is met and recorded here.

### Phase 0 — Stage (this commit)
- Local `main` reset to ironwood, submodule synced, audit + plan + site reference committed.
- **Done when:** the name is chosen, the GitHub repo exists under that name, and this commit is
  pushed to it.

### Phase 1 — Cut and rename
- Delete `lib/`, `ios/`, `android/`, `macos/`, `assets/`, `l10n.yaml`, `pubspec.*`,
  `flutter_rust_bridge.yaml`, `rust_builder/`, the Flutter workflows, `scripts/build-*.sh`,
  `deploy/frostd/`, and the `rng-compat` crate if only the app needs it.
- Remove from the Rust crates: Polymarket (`polymarket.rs`, `cli/market.rs` order paths),
  FROST (`frost.rs`, `cli/frost.rs`), EVM swap and pay (`evm*.rs`), `voting.rs` unless it is the
  ZIP-based governance vote and we want it, `research.rs` (Firecrawl).
- Rename crates, binaries, npm package, env-var prefix, data dir, skill folder. Keep the two MIT
  copyright lines in `LICENSE.md` and add ours.
- Replace every baked-in upstream default: lightwalletd hosts, CipherPay API, FROST relay, the
  NEAR Intents partner JWT and `cipherscan.near` affiliate. Swaps are **off** until Phase 3.
- Pin `ows-core` to a commit we have read, or vendor the two crates we use.
- **Done when:** `cargo build --release -p <cli> -p <mcp-server>` succeeds on this PC from a clean
  clone, `cargo test` passes, and `grep -ri "atmosphere\|cipherscan\|cipherpay\|zipher"` over the
  tree returns only `LICENSE.md`, `docs/AUDIT-*.md` and `docs/reference/`.

### Phase 2 — Close the agent-safety gaps
Ordered by severity from the audit. Each item gets a failing test first, then the fix.
1. `send propose --max` runs `check_proposal` after the engine returns the real amount; `send
   confirm` never re-proposes from a file the agent can write. (C1)
2. Every path that touches the seed goes through the same policy gate; the `pczt` path included.
   Anything we cannot gate is deleted, not left. (H1)
3. Policy semantics: parse error or missing file refuses to spend; `0` means zero, not unlimited;
   allowlist is a set of strings and a scalar is rejected at load. (H3)
4. Privilege separation: the wallet runs as its own OS user or service; `policy.toml`,
   `audit.sqlite` and the data dir are not writable by the agent's user; `policy set` requires
   the operator. Approval for above-threshold sends happens over a channel the agent cannot
   call (CLI as the operator user, or a signed operator token the agent never sees). (H4, C2)
5. Remove `pay_url` URL fetching and `session_request` from the MCP surface, or restrict to an
   operator-set host allowlist with https only; never return raw bodies to the model. (H5, H6)
6. Redact bearer tokens from every tool output. (M2)
7. Sanitise every untrusted string before it reaches a tool result: strip C0/C1 controls, ANSI,
   OSC 8 and OSC 52; cap length by chars not bytes. (M1, M3)
8. Passphrase: refuse empty unless `--unsafe-empty-passphrase`; zeroise after use; set file
   permissions on Windows too. (M4)
9. Persist the rate limiter. (M7)
10. `wallet init` never prints the seed unless `--reveal-seed`; no key material on argv. (M8)
11. Installer verifies the `.sha256` it downloads; CI actions pinned by SHA; `--locked` builds;
    `cargo audit` in CI. (M9)
- **Done when:** each item has a test that fails on the ironwood base and passes on ours, and a
  mutation check (break the fix, watch the test fail) is recorded in the PR.

### Phase 3 — Privacy posture
- Fresh refund/recipient address per swap, or swaps stay off.
- SOCKS/Tor option for lightwalletd; alternates opt-in, not auto-enabled by "known server".
- SQLCipher on the headless wallet DB.
- Separate seeds per chain if any non-Zcash chain returns.
- **Done when:** a single wallet's swap and sync traffic cannot be joined to one identifier by
  the lightwalletd operator or the swap operator, and the reasoning is written in `docs/PRIVACY.md`.

### Phase 4 — The site
- Static page per `docs/reference/zipher-site/README.md` "take / change" list.
- Knowledge entries written against our binary's real tool list. Tool count generated from
  code, not typed.
- **Done when:** the page passes the checks in `docs/reference/zipher-site/README.md`, is verified
  at desktop and 390 px, and every claim on it maps to a command that works.

### Phase 5 — First agent
- Wire the MCP server into a Hermes profile (see the GLASS repo for the container boundary).
- Testnet only, then mainnet with a capped wallet.
- **Done when:** an agent completes a shielded testnet send that required operator approval, and
  a prompt-injected memo fails to move funds, both recorded as integration tests.

## 4. Rules that do not move

Short form; the contract is `CLAUDE.md`.

- The policy gate runs before signing material is loaded, on every path, and fails closed.
- No LLM authorises a spend. Approval is a protocol on a channel the agent cannot reach.
- Destinations come from the allowlist or from a human in the current turn, never from memos,
  documents, fetched pages or model memory.
- Keys never leave the machine, never reach stdout by default, never ride on argv.
- Every claim on the site maps to a command that works.
- Integration tests against real processes. No mocks.

## 5. Decisions

| Date | Decision | By |
|---|---|---|
| 2026-09-20 | Fork, do not contribute upstream. Keep history and MIT notices. | owner |
| 2026-09-20 | Base on `feat/ironwood` `2e7ccd3`, not `main`. | assistant, owner to confirm |
| 2026-09-20 | Two deliverables: agent wallet first, site second. | owner |
| 2026-09-20 | Mobile app, Polymarket, FROST, EVM, CipherPay stack out of scope. | assistant, owner to confirm |
| TBD | Name. Candidates checked 2026-09-20 for GitHub/npm/DNS: see the session notes. | owner |
| TBD | Own repo vs inside GLASS. | owner |
