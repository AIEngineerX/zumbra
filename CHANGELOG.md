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

### Security
- Inherited from upstream's 2026-09-10 remediation: MCP self-unlock and self-approve tools
  removed, durable daily-spend reservations, confirmations bound to unpredictable proposal IDs.
- Known open items are listed in `SECURITY.md` and are the next work.
