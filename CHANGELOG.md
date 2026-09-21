# Changelog

All notable changes to Zumbra. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versions follow [Semantic Versioning](https://semver.org/). Nothing has been released yet.

## [0.3.0] - 2026-09-21

First public build: tag `cli-v0.3.0`, binaries for Linux (x64, arm64) and macOS (Intel, Apple
Silicon) with SHA-256 files. Pre-alpha, for the testnet testing round. Everything below shipped
in it.

### Added
- Fork of Zipher (MIT) at its `feat/ironwood` branch, 2026-09-20, with full history and both
  inherited copyright lines retained.
- Brand: the Negative Z eclipse mark, tokens, and a script that generates every asset from one SVG.
- Repository hygiene: security policy, contributing guide, code of conduct, NOTICE, pre-commit
  secret block, release gate, pre-public audit, cargo-deny policy, CI with actions pinned by
  commit, Dependabot.

### Changed
- README credits the parent project up front and states status precisely.
- Renamed throughout: crates `zumbra`, `zumbra-mcp`, `zumbra-engine`; env prefix `ZUMBRA_`;
  data directory `~/.zumbra`; npm package `zumbra`; skill folder `zumbra-operator`.
- Default mainnet light-client server is a community one; nothing points at the parent's hosts.
- MCP server describes only the tools it has; its website field points at this repository.

### Removed
- The parent's Flutter app, its bridge crate, and the wrapper workspace.
- Swaps, EVM payments and sweeps, prediction markets, FROST multi-party wallets, the merchant
  API, the research tools, the URL-fetching paywall tool, session tokens, and the relay-based
  approval path, and two MCP tools that could only answer "nothing" (`get_pending_approval`)
  or "refused" (`shield_funds`). The MCP server exposes 15 wallet tools; the CLI has 15 commands.
- The PCZT export subcommands, which had no policy check.
- The parent's swap affiliate key and partner token.

### Added
- Operator approval channel. `zumbra operator init` creates an Ed25519 key (private half
  encrypted under a passphrase typed at the terminal); `zumbra approve <proposal_id>` shows the
  proposal and signs it for a limited time. Sends above `approval_threshold` are proposed as
  usual, then refused at confirm until a valid, single-use approval exists. The agent gets a
  `proposal_id` and a `next_step` telling it what the operator must run; it has no tool to
  approve. Tests on the engine, the CLI gate and the MCP gate, with mutation checks.

### Changed
- Default policy: `max_per_tx` 0.05 ZEC, `approval_threshold` 0.01 ZEC (was 0.01 and 0.05, a
  threshold above the cap, so the approval step could never trigger on a fresh wallet).
  `daily_limit` stays 0.1 ZEC. Test.

### Fixed
- `zumbra-mcp` now advertises the `tools` capability in its `initialize` response. Without it,
  spec-following clients (Claude Code, Hermes) connected and listed zero tools; only a client that
  called `tools/list` regardless ever saw them. Test.

### Security
- `wallet restore` takes the seed phrase on stdin instead of argv, stores it only in the
  encrypted OWS vault instead of a plaintext `.seed` file, refuses to overwrite an existing vault
  wallet, and writes the default spending policy so a restored wallet is capped. The plaintext
  seed-file fallback in the CLI is gone. Four tests; two mutation checks recorded.
- A missing, unreadable or corrupt `policy.toml` refuses to spend on every CLI and MCP spend path
  instead of loading as "no limits". Engine and CLI tests.
- A restore whose network step fails rolls back the vault entry and the policy file it staged, so
  the next `wallet init` cannot "reuse" the half-restored seed; and `wallet init` no longer prints
  a seed it did not just generate. Tests.
- `wallet init` reaches the server before creating anything, and if the wallet cannot be built
  after a vault seed was created, that unseen seed is removed rather than "reused" by the next
  init. Restore and init share one rollback that also removes a partial database, keeps a policy
  file the operator already had, and reports anything it could not remove. Tests.
- `zumbra-mcp` reads `ZUMBRA_TESTNET` with the same values the CLI accepts (1/0, true/false,
  yes/no, on/off). Test.
- The pre-commit hook handles filenames with spaces and non-ASCII characters (NUL-separated
  listing); checked by staging such files.
- `wallet init` and `wallet restore` refuse an empty vault passphrase unless
  `ZUMBRA_UNSAFE_EMPTY_PASSPHRASE=1` is set. Test.
- The CLI reads `ZUMBRA_DATA_DIR`, `ZUMBRA_TESTNET` and `ZUMBRA_SERVER`, as its docs, the
  Dockerfile and CI already assumed. Test.
- The pre-commit hook no longer exempts `.txt` files from the seed-phrase check; `seed.txt` is
  ignored by git.
- CLI `send confirm` checks the policy against the engine's real send amount before reading the
  seed. Previously a `--max` proposal was never policy-checked and confirm only ran the rate
  limit, so an edited pending file was signed as written. Test plus mutation check.
- Inherited from upstream's 2026-09-10 remediation: MCP self-unlock and self-approve tools
  removed, durable daily-spend reservations, confirmations bound to unpredictable proposal IDs.
- Known open items are listed in `SECURITY.md` and are the next work.
