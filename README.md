<p align="center"><img src="brand/dist/readme-banner-1280x320.png" alt="Zumbra — Shielded Zcash for humans and agents" width="100%"></p>

# Zumbra

[![ci](https://github.com/AIEngineerX/zumbra/actions/workflows/ci.yml/badge.svg)](https://github.com/AIEngineerX/zumbra/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-F2C14E.svg)](LICENSE.md)
[![Status: pre-alpha](https://img.shields.io/badge/status-pre--alpha-1A1F2B.svg)](SECURITY.md)

> Shielded Zcash for humans and agents. Keys on your machine,
> a spending policy the agent cannot touch.

**Status: pre-alpha.** The engine is a working Zcash light wallet. The agent-safety layer on top
is not finished, so run it on testnet, and do not hand an agent mainnet funds until the items in
[`SECURITY.md`](SECURITY.md) are closed. That file says exactly what holds today and what does not.

**Zumbra is a fork of [Zipher](https://github.com/atmospherelabs-dev/zipher-app) by Atmosphere
Labs, MIT-licensed, taken at their `feat/ironwood` branch on 2026-09-20 with full history.**
We kept their Rust engine, CLI and MCP server, which are good. We are changing the guarantees
on top: the spending policy and the human approval move to where an agent cannot edit or call
them, their hosted endpoints and affiliate key come out, and everything that is not the agent
wallet is switched off. Their copyright line stays in [`LICENSE.md`](LICENSE.md) alongside the
original YWallet author's and ours; [`NOTICE`](NOTICE) lists every bundled component. We do not
contribute upstream and we are not affiliated with them.

## What it will be

**The agent wallet.** A headless Zcash wallet: a Rust engine, a CLI, and an MCP server that any
MCP client (Hermes, OpenClaw, Claude, Cursor) can use. The spending policy and the human
approval step are enforced by something the agent cannot edit or call. Shielded sends only.
No cloud, no custody, no telemetry, no baked-in third-party endpoints.

## Scope

The mobile app, prediction markets, multi-party wallets, EVM paths and the merchant stack that
existed in the fork's parent are being removed, not paused. Zumbra is the agent wallet.

## Why a fork

The parent project shipped the right architecture and a solid Zcash engine. By its own published
reviews, the agent-safety layer on top was unfinished. We wanted the engine, a different set of
guarantees on top, and the freedom to narrow scope to the agent wallet. SECURITY.md lists exactly
what holds today and what does not.

## Repository map

| Path | What |
|---|---|
| `rust/crates/engine` | Wallet engine: sync, send, policy, audit, vault |
| `rust/crates/cli` | Headless CLI |
| `rust/crates/mcp-server` | MCP server over stdio |
| `rust/crates/rng-compat` | RNG shim required by the vendored Zcash crates |
| `rust/vendor` | Four patched Zcash wallet crates with SHA-256 provenance |
| `ows-core` | Open Wallet Standard vault and signer (submodule) |
| `skills/` | Operator skill file for agent harnesses |
| `npm/` | npm launcher that downloads and checksum-verifies release binaries |
| `docs/BRAND.md`, `brand/` | Brand tokens, the mark, and the exporter that generates every asset |
| `lib/`, `ios/`, `android/`, `assets/` | The parent's Flutter app; switched off, not maintained here |

## Building

Rust stable with `rustfmt` and `clippy`, plus `protoc`, `cmake`, `pkg-config` and OpenSSL
headers. Linux or macOS; on Windows use WSL (the daemon uses Unix sockets and there is no
Windows binary).

```bash
git clone --recurse-submodules https://github.com/AIEngineerX/zumbra.git
cd zumbra && bash scripts/setup-hooks.sh
cd rust && cargo build --locked --release -p zumbra -p zumbra-mcp
# binaries: rust/target/release/zumbra and rust/target/release/zumbra-mcp
```

First run, on testnet, with a real vault passphrase:

```bash
export OWS_PASSPHRASE='<a real passphrase>'
zumbra --testnet wallet init      # prints the seed phrase once; store it
zumbra --testnet sync start
zumbra --testnet address          # paste this at a testnet faucet to get TAZ, which has no value
zumbra --testnet balance
```

Testnet coins come from public faucets, not from this project. See
[`CONTRIBUTING.md`](CONTRIBUTING.md) for the release gate.

## Roadmap

In order, each with a done-criterion rather than a date:

1. **Run it as forked.** Build from source, shielded testnet sends through the CLI and the MCP
   server, unmodified.
2. **Make it ours.** Rename, remove the parent's endpoints and affiliate key, switch off what is
   not the agent wallet.
3. **Make it safe for an agent.** Close the listed gaps in [`SECURITY.md`](SECURITY.md), each
   with a test that fails on the fork point and passes here.
4. **Privacy posture.** Fresh addresses per swap, Tor/SOCKS for the light client, encrypted
   wallet database.
5. **First agent on mainnet.** Capped wallet, operator approval over a channel the agent cannot
   call, prompt-injection tests recorded as integration tests.

## License

MIT. See [`LICENSE.md`](LICENSE.md) for the three copyright lines this code carries and
[`NOTICE`](NOTICE) for every bundled component and where it came from.
