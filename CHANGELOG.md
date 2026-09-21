# Changelog

All notable changes to Zumbra. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versions follow [Semantic Versioning](https://semver.org/). Nothing has been released yet.

## [Unreleased]

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
  approval path. The MCP server exposes 17 wallet tools; the CLI has 15 commands.
- The PCZT export subcommands, which had no policy check.
- The parent's swap affiliate key and partner token.

### Security
- `wallet restore` takes the seed phrase on stdin instead of argv, stores it only in the
  encrypted OWS vault instead of a plaintext `.seed` file, refuses to overwrite an existing vault
  wallet, and writes the default spending policy so a restored wallet is capped. The plaintext
  seed-file fallback in the CLI is gone. Four tests; two mutation checks recorded.
- CLI `send confirm` checks the policy against the engine's real send amount before reading the
  seed. Previously a `--max` proposal was never policy-checked and confirm only ran the rate
  limit, so an edited pending file was signed as written. Test plus mutation check.
- Inherited from upstream's 2026-09-10 remediation: MCP self-unlock and self-approve tools
  removed, durable daily-spend reservations, confirmations bound to unpredictable proposal IDs.
- Known open items are listed in `SECURITY.md` and are the next work.
