# Quickstart

From a clone to an agent that can propose a shielded testnet send, in one sitting. Every command
here was run against the built binary on 2026-09-21; the flags are the binary's, not a summary.
Amounts are in zatoshi throughout: 1 ZEC = 100 000 000 zat, so `1000000` is 0.01 ZEC.

## 0. Build

Linux or macOS, or WSL on Windows. Rust stable, `protoc`, `cmake`, `pkg-config`, OpenSSL headers.

```bash
git clone --recurse-submodules https://github.com/AIEngineerX/zumbra.git
cd zumbra && bash scripts/setup-hooks.sh
cd rust && cargo build --locked --release -p zumbra -p zumbra-mcp
export PATH="$PWD/target/release:$PATH"
```

## 1. A wallet on testnet

The seed is generated inside an encrypted vault on this machine. Pick a real passphrase; the
vault is only as strong as it.

```bash
export OWS_WALLET=zumbra-testnet
export OWS_PASSPHRASE='<a real passphrase>'
zumbra --testnet wallet init
```

`init` prints the seed phrase once, on purpose, so you can write it down. It also writes the
default policy and prints an MCP config block. Nothing else ever prints the seed.

Restoring an existing seed takes it on stdin, never on the command line. Run the command, paste
the 24 words at the prompt, press Enter; or pipe it from a password manager. If anything after
the seed is stored fails (usually the server), the restore rolls itself back and tells you.

```bash
zumbra --testnet wallet restore --birthday <block height>
```

## 2. Sync and read

```bash
zumbra --testnet sync start          # blocks until synced; Ctrl+C to stop
zumbra --testnet sync status
zumbra --testnet address             # paste this at a testnet faucet for TAZ (no value)
zumbra --testnet balance
zumbra --testnet transactions --limit 10
```

Everything is JSON by default. Add `--human` for prose.

## 3. The policy is yours, not the agent's

```bash
zumbra --testnet policy show
zumbra --testnet policy set --field max_per_tx --value 1000000
zumbra --testnet policy set --field daily_limit --value 5000000
zumbra --testnet policy set --field approval_threshold --value 500000
zumbra --testnet policy add-allowlist --address <unified address>
```

Fields: `max_per_tx`, `daily_limit`, `approval_threshold`, `min_spend_interval_ms`,
`require_context_id`, and the allowlist. Values in zatoshi. There is no MCP tool for any of this.

## 4. A send, in two steps

```bash
zumbra --testnet send propose --to <unified address> --amount 1000000 --memo "hello"
zumbra --testnet send confirm
```

`propose` builds the transaction with no seed loaded and reports amount and fee. `confirm` runs
the policy against the amount the engine will actually send, and only if it passes reads the
seed, signs, broadcasts, and writes the audit row. A send above the approval threshold is refused
today; the operator approval step is the next thing being built.

```bash
zumbra --testnet audit --limit 20
zumbra --testnet audit --since 2026-09-01
```

## 5. The agent

`zumbra-mcp` speaks MCP over stdio. It reads the same environment as the CLI plus
`ZUMBRA_TESTNET=1` and, optionally, `ZUMBRA_DATA_DIR` and `ZUMBRA_SERVER`.

```json
{
  "mcpServers": {
    "zumbra": {
      "command": "zumbra-mcp",
      "env": {
        "ZUMBRA_TESTNET": "1",
        "OWS_WALLET": "zumbra-testnet",
        "OWS_PASSPHRASE": "<the same passphrase>"
      }
    }
  }
}
```

The tools mirror the CLI's read and send commands: `wallet_status`, `get_balance`,
`get_transactions`, `sync_status`, `validate_address`, `propose_send`, `confirm_send`,
`wallet_lock`, `pay_x402`, plus the Ironwood pool-migration tools and `vote_eligibility`.
Shielding transparent funds is an operator action: `zumbra shield` on the CLI. No tool unlocks
the wallet, approves a send, or changes the policy. `wallet_lock` clears the seed from memory; only restarting the server from
your terminal brings it back.

## 6. Keep it running

```bash
zumbra --testnet daemon start        # foreground: sync loop plus a local Unix socket
zumbra --testnet daemon status
zumbra --testnet daemon lock         # zeroise the seed in memory; reads keep working
zumbra --testnet daemon stop
```

## Global flags

| Flag | Environment | Meaning |
|---|---|---|
| `--testnet` | `ZUMBRA_TESTNET=1` | Zcash testnet. Start here. |
| `--data-dir <path>` | `ZUMBRA_DATA_DIR` | Wallet directory; default `~/.zumbra/<network>` |
| `--server <url>` | `ZUMBRA_SERVER` | lightwalletd or Zaino server; the default is a community server |
| `--human` | | Prose instead of JSON |

Before funding anything on mainnet, read [`SECURITY.md`](../SECURITY.md). It says what holds
today and what does not.
