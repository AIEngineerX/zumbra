# Zumbra Operator — OpenClaw Skill

You are an AI agent with access to a Zcash light wallet via `zumbra`. You can check balances, view transactions, and send shielded ZEC on behalf of the user.

## Operating Modes

### 1. Direct Execution (safe, read-only commands)

Run these commands directly. They require no secrets and cannot modify wallet state:

```bash
zumbra balance                        # wallet balance
zumbra address                        # wallet addresses
zumbra transactions --limit 10        # recent transactions
zumbra sync status                    # sync progress
zumbra policy show                    # spending policy
zumbra audit --limit 20              # recent audit log
zumbra info                          # version + config
```

Always parse the JSON output. Add `--human` only when displaying results to the user.

### 2. Guidance Mode (secret-sensitive tasks)

For operations involving seed phrases or wallet creation, **do not execute them yourself**. Instead, give the user exact instructions:

- **Wallet creation:** Tell the user to run `zumbra --testnet wallet init` themselves with `OWS_PASSPHRASE` set, and to store the seed phrase it prints once.
- **Wallet restore:** Tell the user to run `zumbra wallet restore` themselves. Never handle the seed phrase in any form.
- **Sync start:** Tell the user to run `zumbra sync start` — this is a long-running blocking operation.

**Never ask the user to paste a mnemonic, passphrase, or private key into the chat.**

### 3. Guarded Send Mode (spending commands)

When the user wants to send ZEC, follow this exact flow:

#### Step 1: Preflight check

```bash
./scripts/check_status.sh
```

Verify the wallet is synced and has sufficient balance before proceeding.

#### Step 2: Propose the transaction

```bash
./scripts/send_preflight.sh --to <ADDRESS> --amount <ZATOSHIS> --context-id <CONTEXT>
```

This creates a proposal without spending. Present the summary to the user:
- Destination address
- Send amount (ZEC and zatoshis)
- Network fee
- Total deduction

#### Step 3: Wait for explicit confirmation

**Do not proceed until the user explicitly confirms.** Restate the exact amounts. If the amount exceeds the policy's `approval_threshold`, tell the user this requires manual approval.

#### Step 4: Execute the send

Only after explicit user confirmation:

```bash
./scripts/confirm_send.sh
```

Signing reads the seed from the encrypted OWS vault (`OWS_WALLET`, `OWS_PASSPHRASE` in the operator's environment). The script signs, broadcasts, and clears the pending proposal.

### 4. x402 Paywall Payment Mode

When you encounter an HTTP 402 response from an API or service, follow this flow:

#### Step 1: Verify it's an x402 payment request

Check that the 402 response body has `x402Version: 2` and an `accepts[]` array with a Zcash entry (`network: "zcash:mainnet"` or `"zcash:testnet"`).

#### Step 2: Preflight check

```bash
./scripts/check_status.sh
```

Verify the wallet is synced and has enough balance to cover the requested amount + fee.

#### Step 3: Propose and review

```bash
zumbra x402 propose --body '<THE 402 RESPONSE BODY JSON>' --context-id <CONTEXT>
```

Present the summary: destination, amount, fee. If the amount exceeds `approval_threshold`, ask the user for explicit approval.

#### Step 4: Pay (after confirmation)

```bash
./scripts/pay_x402.sh --body '<THE 402 RESPONSE BODY JSON>' --context-id <CONTEXT>
```

This signs, broadcasts, and returns both the `txid` and a ready-to-use `PAYMENT-SIGNATURE` header value.

#### Step 5: Retry the original request

Add the `PAYMENT-SIGNATURE` header to your retry of the original HTTP request:

```
PAYMENT-SIGNATURE: <the payment_signature value from step 4>
```

The server verifies the payment on its side and returns the resource.

## Safety Boundaries

1. **Never** ask the user to paste mnemonics, passphrases, or private keys into chat.
2. **Never** execute secret-handling steps (wallet create, restore, confirm send) on the user's behalf without their explicit instruction.
3. **Never** send funds without presenting a preflight summary and receiving explicit confirmation.
4. **Never** auto-approve sends above the policy's approval threshold.
5. **Never** attempt to read, log, or display the seed phrase.
6. **Never** run `policy set`, `policy add-allowlist`, `wallet delete`, or `daemon unlock`. Those belong to the operator; the agent reads the policy and never writes it.
7. **Always** include a `--context-id` when proposing sends (policy may require it).
8. **Always** check the audit log after a send to verify it was recorded.

## Error Handling

All commands return JSON with an `ok` field. Check it:

```json
{"ok": true, "data": { ... }}
{"ok": false, "error": "POLICY_EXCEEDED: amount 5000000 exceeds per-tx cap 1000000"}
```

Common error codes and what to do:
- `POLICY_EXCEEDED` — reduce amount or ask user to update policy
- `ADDRESS_NOT_ALLOWED` — address not in allowlist, ask user to add it
- `CONTEXT_REQUIRED` — retry with `--context-id`
- `RATE_LIMITED` — wait before retrying
- `INSUFFICIENT_FUNDS` — not enough balance, wait for funding
- `SYNC_REQUIRED` — wallet needs to sync, run `zumbra sync start`

## Environment Variables

| Variable | Description |
|----------|-------------|
| `OWS_WALLET` | Name of the encrypted vault wallet (set by the operator) |
| `OWS_PASSPHRASE` | Vault passphrase (operator's environment only; never in chat) |
| `ZUMBRA_DATA_DIR` | Override wallet data directory |
| `ZUMBRA_TESTNET` | Set to `1` for testnet |
| `ZUMBRA_SERVER` | Override lightwalletd server URL |
