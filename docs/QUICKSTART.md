# Quickstart

From a clone to an agent that can propose a shielded testnet send, in one sitting. Every command
here was run against the built binary on 2026-09-21; the flags are the binary's, not a summary.
Amounts are in zatoshi throughout: 1 ZEC = 100 000 000 zat, so `1000000` is 0.01 ZEC.

## 0. Get the binaries

Prebuilt for Linux (x64, arm64) and macOS (Intel, Apple Silicon) on the
[releases page](https://github.com/AIEngineerX/zumbra/releases/tag/cli-v0.3.1). Each file ships with a `.sha256` next to it; check it before running
anything, then put `zumbra` and `zumbra-mcp` on your PATH:

```bash
curl -LO https://github.com/AIEngineerX/zumbra/releases/download/cli-v0.3.1/zumbra-linux-x64
curl -LO https://github.com/AIEngineerX/zumbra/releases/download/cli-v0.3.1/zumbra-linux-x64.sha256
sha256sum -c zumbra-linux-x64.sha256 && chmod +x zumbra-linux-x64 && mv zumbra-linux-x64 ~/.local/bin/zumbra
# same for zumbra-mcp-linux-x64; on macOS use shasum -a 256 -c and the darwin-* files
```

Or build from source. Linux or macOS, or WSL on Windows. Rust stable, `protoc`, `cmake`,
`pkg-config`, OpenSSL headers.

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
seed, signs, broadcasts, and writes the audit row. Above the approval threshold, confirm waits
for you (next section).

```bash
zumbra --testnet audit --limit 20
zumbra --testnet audit --since 2026-09-01
```

## 4b. Sends above the threshold need you

Once, from your own terminal (the passphrase is typed, never stored or exported):

```bash
zumbra --testnet operator init
```

When a send is above `approval_threshold`, propose still works and prints a proposal id with
`approval_required: true`. Confirm is refused until you sign that exact proposal:

```bash
zumbra --testnet approve <proposal_id>          # shows to, amount, fee; asks for the passphrase
zumbra --testnet send confirm                    # now passes the gate, reads the seed, signs
```

An approval is for one proposal, is used once, and expires (10 minutes by default,
`--ttl-minutes` to change). The agent sees the same flow over MCP: propose_send returns
`next_step` naming the command you must run; confirm_send is refused with the same hint until
you have run it. Nothing the agent can call creates an approval.

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
Shielding transparent funds and approving a send are operator actions on the CLI. No tool
unlocks the wallet, approves a send, or changes the policy. `wallet_lock` clears the seed from memory; only restarting the server from
your terminal brings it back.

## 6. Keep it running

```bash
zumbra --testnet daemon start        # foreground: sync loop plus a local Unix socket
zumbra --testnet daemon status
zumbra --testnet daemon lock         # zeroise the seed in memory; reads keep working
zumbra --testnet daemon stop
```

## Testing round: what to try, what to expect

This is pre-alpha on testnet. Run the steps above with TAZ from a public faucet, then try to break
the two things the design promises: the seed stays on your machine, and the agent cannot spend
past the policy.

Expect these to work: init, restore from stdin, sync, balance, address, a two-step send within the
caps, the audit log, and every MCP read tool. Expect these to be refused, and report it if they
are not: a send above `max_per_tx` or `daily_limit`, a send to an address outside a non-empty
allowlist, a send above `approval_threshold` (refused outright today; operator approval is not
built yet), any spend with a missing or corrupt `policy.toml`, and any MCP call that tries to
unlock, approve, or change the policy.

Known gaps are listed in [`SECURITY.md`](../SECURITY.md); do not report those. Everything else,
including anything that prints, logs, or writes a seed where it should not, goes to the
repository's issues, or to a private security advisory if it touches keys or the spend path.
Never paste a seed phrase into an issue, even a testnet one.

## Global flags

| Flag | Environment | Meaning |
|---|---|---|
| `--testnet` | `ZUMBRA_TESTNET=1` | Zcash testnet. Start here. |
| `--data-dir <path>` | `ZUMBRA_DATA_DIR` | Wallet directory; default `~/.zumbra/<network>` |
| `--server <url>` | `ZUMBRA_SERVER` | lightwalletd or Zaino server; the default is a community server |
| `--human` | | Prose instead of JSON |

Before funding anything on mainnet, read [`SECURITY.md`](../SECURITY.md). It says what holds
today and what does not.
