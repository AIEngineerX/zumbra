# zumbra

Headless Zcash light wallet for AI agents. Shielded sends, a spending policy the agent cannot
edit, and an MCP server. Pre-alpha: binaries are on the GitHub releases page; this npm package
is not published to npmjs yet, so `npm install -g zumbra` does not work until it is.

## Install (once the package is on npmjs)

```bash
npm install -g zumbra
```

Installs two binaries: `zumbra` (CLI) and `zumbra-mcp` (MCP server for AI agents). The
installer downloads the release binaries for your platform and verifies their SHA-256 checksums
before installing them.

## Quick start

```bash
# Testnet first. Creates an encrypted OWS vault and a wallet inside it.
export OWS_PASSPHRASE='<a real passphrase>'
zumbra --testnet wallet init

# Sync and read
zumbra --testnet sync start
zumbra --testnet balance
zumbra --testnet address

# Send: propose (no seed loaded), then confirm (policy runs, then the seed is read)
zumbra --testnet send propose --to <unified address> --amount 1000000   # zatoshi; 0.01 ZEC
zumbra --testnet send confirm
```

Testnet coins (TAZ) have no value and come from a public faucet; the wallet's own address is
what you paste there.

## MCP server

For any MCP client (Hermes, OpenClaw, Claude, Cursor), add to its config:

```json
{
  "mcpServers": {
    "zumbra": {
      "command": "zumbra-mcp"
    }
  }
}
```

The server loads the seed from the encrypted OWS vault created by `wallet init`, using
`OWS_WALLET` and `OWS_PASSPHRASE` from its environment.

### MCP tools

Generated from the server source by `.private/phase2/gen-mcp-manifest.py`; the same list is in
`mcp.json`. No tool can unlock the wallet, approve a spend above the threshold, or change the
policy.

| Tool | What it does |
|---|---|
| `wallet_status` | Sync height, balance, primary address, policy summary, seed source |
| `wallet_lock` | Clears the seed from memory; signing fails until the operator restarts the server |
| `get_balance` | Balance per pool: Orchard, Sapling, transparent, unconfirmed |
| `propose_send` | Builds a send proposal, returns fee and amount for review; no seed loaded |
| `confirm_send` | Runs the policy, then signs and broadcasts the pending proposal |
| `get_transactions` | Recent history with memos |
| `sync_status` | Synced height, latest height, whether syncing, connection errors |
| `validate_address` | Address validity and type |
| `pay_x402` | Pays an x402 paywall from a 402 response body, bounded by the policy |
| `vote_eligibility` | Governance eligibility for a round |
| `ironwood_plan`, `ironwood_status` | Read the NU6.3 Ironwood pool-migration plan and state |
| `ironwood_confirm`, `ironwood_pause`, `ironwood_resume` | Present for compatibility; return "unavailable" |

### Security model

- Seed at rest only in the encrypted OWS vault (scrypt + AES-256-GCM); never on stdout by default.
- Process hardening: core dumps disabled, ptrace blocked.
- Spending policy: per-transaction cap, rolling daily cap, destination allowlist, approval
  threshold. The agent reads it; only the operator writes it.
- Two-step sends with replay protection: confirm requires the exact proposal it was given.
- Audit log: every proposal, spend and refusal, locally, in SQLite.

The gaps that still exist are listed in the repository's SECURITY.md. Read it before funding
anything.

## Spending policy

Default policy written by `wallet init`:

```toml
max_per_tx = 1000000          # 0.01 ZEC per transaction
daily_limit = 10000000        # 0.1 ZEC per rolling day
approval_threshold = 5000000  # 0.05 ZEC: above this the operator must approve
allowlist = []                # empty = any address; set it
```

Edit `~/.zumbra/<network>/policy.toml` or use `zumbra policy set` as the operator.

## Supported platforms

| OS | Arch |
|---|---|
| macOS | ARM64, x64 |
| Linux | x64, ARM64 |

No Windows binary. Build in WSL.

## Links

- [GitHub](https://github.com/AIEngineerX/zumbra)
- [Site](https://zumbra.dev)
