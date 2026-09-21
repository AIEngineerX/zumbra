# zumbra Command Reference

## Global Flags

| Flag | Description |
|------|-------------|
| `--data-dir <PATH>` | Wallet data directory (default: `~/.zumbra/mainnet`) |
| `--testnet` | Use Zcash testnet |
| `--server <URL>` | Override lightwalletd server URL |
| `--human` | Human-readable output instead of JSON |

## Commands

### info
Print version, engine, network, data directory, and server URL.
```
zumbra info
```

### wallet init
Create a new wallet. Outputs seed phrase — store it securely.
```
zumbra wallet init
```

### wallet restore
Restore from a seed phrase. Operator-only; the agent never runs this and never handles the phrase.
```
zumbra wallet restore --birthday <HEIGHT>
```

### wallet delete
Delete wallet data. Requires `--confirm` flag.
```
zumbra wallet delete --confirm
```

### sync start
Start syncing (blocks until complete or Ctrl+C).
```
zumbra sync start
```

### sync status
Show current sync height and birthday.
```
zumbra sync status
```

### balance
Show wallet balance by pool (orchard, sapling, transparent).
```
zumbra balance
```

### address
Show wallet addresses and their pool capabilities.
```
zumbra address
```

### transactions
Show recent transaction history.
```
zumbra transactions --limit 20
```

### send propose
Create a send proposal (no seed required). Saves to pending file.
```
zumbra send propose --to <ADDRESS> --amount <ZATOSHIS> [--memo <TEXT>] [--context-id <ID>]
```

### send confirm
Sign and broadcast the pending proposal. Requires seed.
```
zumbra send confirm
```

### send max
Show maximum sendable amount to an address.
```
zumbra send max --to <ADDRESS>
```

### shield
Shield transparent funds to shielded pool. Requires seed.
```
zumbra shield
```

### policy show
Display current spending policy.
```
zumbra policy show
```

### policy set
Set a policy field.
```
zumbra policy set --field <FIELD> --value <VALUE>
```
Fields: `max_per_tx`, `daily_limit`, `min_spend_interval_ms`, `approval_threshold`, `require_context_id`

### policy add-allowlist
Add an address to the spending allowlist.
```
zumbra policy add-allowlist --address <ADDRESS>
```

### policy remove-allowlist
Remove an address from the allowlist.
```
zumbra policy remove-allowlist --address <ADDRESS>
```

### audit
View the audit log.
```
zumbra audit --limit 50 [--since <ISO8601_TIMESTAMP>]
```

### daemon start
Start the daemon (foreground, sync loop + Unix socket IPC).
```
zumbra daemon start
```

### daemon status
Check if the daemon is running.
```
zumbra daemon status
```

### daemon stop
Ask the daemon to stop.
```
zumbra daemon stop
```

### daemon lock
Zeroize seed in daemon memory. Sync continues, spending disabled.
```
zumbra daemon lock
```

### daemon unlock
Re-provide seed to re-enable spending.
```
zumbra daemon unlock
```

### x402 propose
Parse an HTTP 402 response body and create a send proposal (no seed required).
```
zumbra x402 propose --body '<JSON>' [--context-id <ID>]
```
Reads from stdin if `--body` is omitted.

### x402 pay
Parse a 402 response, pay, and return the PAYMENT-SIGNATURE header. Requires seed.
```
zumbra x402 pay --body '<JSON>' [--context-id <ID>]
```
Returns `{ txid, payment_signature, amount, fee, address }`. The `payment_signature` is a base64 header value to include as `PAYMENT-SIGNATURE` when retrying the original request.
