<p align="center"><img src="brand/dist/readme-banner-1280x320.png" alt="Zumbra — Shielded Zcash for humans and agents" width="100%"></p>

# Zumbra

> Shielded Zcash for humans and agents. Keys on your machine,
> a spending policy the agent cannot touch.

**Status: pre-alpha, not safe to fund.** The code builds a working Zcash light wallet, but the
agent-safety layer is not finished. Read [`SECURITY.md`](SECURITY.md) for exactly what holds and
what does not. The running plan is kept privately by the owner; the public summary is there too.

**Zumbra is a fork of [Zipher](https://github.com/atmospherelabs-dev/zipher-app) by Atmosphere
Labs, MIT-licensed, taken at their `feat/ironwood` branch on 2026-09-20 with full history.**
We kept their Rust engine, CLI and MCP server, which are good. We are changing the guarantees
on top: the spending policy and the human approval move to where an agent cannot edit or call
them, their hosted endpoints and affiliate key come out, and everything that is not the agent
wallet is switched off. Their copyright line stays in [`LICENSE.md`](LICENSE.md) alongside the
original YWallet author's and ours; [`NOTICE`](NOTICE) lists every bundled component. We do not
contribute upstream and we are not affiliated with them.

## What it will be

Two things, built in this order:

1. **The agent wallet.** A headless Zcash wallet: a Rust engine, a CLI, and an MCP server that any
   MCP client (Hermes, OpenClaw, Claude, Cursor) can use. The spending policy and the human
   approval step are enforced by something the agent cannot edit or call. Shielded sends only.
   No cloud, no custody, no telemetry, no baked-in third-party endpoints.
2. **A site.** One static page: a single "ask" input over a local knowledge base, a human/agent
   toggle, and links. No LLM behind it, nothing tracked.

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

Rust stable with `rustfmt` and `clippy`. Crate and binary names still carry the parent's until the
rename lands; build by path.

```bash
git clone --recurse-submodules https://github.com/AIEngineerX/zumbra.git
cd zumbra && bash scripts/setup-hooks.sh
cd rust/crates/cli && cargo build --locked --release          # the CLI
cd ../mcp-server && cargo build --locked --release            # the MCP server
```

Use testnet. See [`CONTRIBUTING.md`](CONTRIBUTING.md).

## License

MIT. See [`LICENSE.md`](LICENSE.md) for the three copyright lines this code carries and
[`NOTICE`](NOTICE) for every bundled component and where it came from.
