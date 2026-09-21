# How Zumbra works

Two ways in, one engine, one wallet on disk. Nothing leaves the machine except light-client
traffic to a Zcash server you choose.

## The pieces

```mermaid
flowchart LR
    subgraph you["You (operator)"]
        CLI["zumbra<br/>CLI"]
    end
    subgraph agent["Your agent (Hermes, OpenClaw, Claude, Cursor)"]
        MCP["zumbra-mcp<br/>MCP server over stdio"]
    end
    subgraph engine["Wallet engine (Rust)"]
        POL["Spending policy<br/>caps · allowlist · threshold"]
        SEND["Send<br/>propose → confirm"]
        AUD["Audit log<br/>every proposal, spend, refusal"]
        SYNC["Light client<br/>sync · scan · broadcast"]
    end
    subgraph disk["On your disk"]
        VAULT[("Encrypted vault<br/>scrypt + AES-256-GCM<br/>holds the seed")]
        DB[("Wallet DB<br/>notes · history")]
        PT[("policy.toml<br/>operator-owned")]
    end
    NET["lightwalletd / Zaino<br/>(your choice of server)"]
    ZC["Zcash network"]

    CLI --> SEND
    MCP --> SEND
    SEND --> POL
    POL -->|"passes"| VAULT
    VAULT -->|"seed, only now"| SEND
    SEND --> SYNC
    SYNC <--> NET <--> ZC
    SEND --> AUD
    POL -.->|"reads"| PT
    CLI -->|"writes"| PT
    SYNC <--> DB
```

- The agent and the operator use the same engine. The difference is what each is allowed to
  touch: the agent can propose and confirm; the policy is written with the operator's CLI, and
  no MCP tool touches it.
- The seed is read from the vault only after the policy has passed, and only for the signing
  step. It is zeroised from memory afterwards.
- The audit log is local SQLite. The daily cap is computed from it, not from memory, so a
  restart cannot reset it.

## How a send works

```mermaid
sequenceDiagram
    autonumber
    participant A as Agent
    participant M as zumbra-mcp
    participant P as Policy
    participant O as Operator
    participant V as Vault
    participant N as Zcash

    A->>M: propose_send(address, amount)
    M->>M: build proposal (no seed loaded)
    M-->>A: fee, amount, proposal_id
    A->>M: confirm_send(proposal_id)
    M->>P: check: per-tx cap, daily cap, allowlist, threshold
    alt refused
        P-->>M: violation
        M->>M: audit row: refused
        M-->>A: error code, nothing signed
    else above threshold
        P-->>M: approval required
        M-->>A: refused: "operator runs zumbra approve <id>"
        O->>O: zumbra approve <id>: sees to/amount/fee, types passphrase, signs
        A->>M: confirm_send(proposal_id) again
        M->>M: verify signature with the operator's public key; consume the approval
    end
    M->>V: read seed
    V-->>M: seed
    M->>M: sign, zeroise seed bytes
    M->>N: broadcast
    M->>M: audit row: txid, amount, context
    M-->>A: txid
```

The order is the whole design: propose, policy, approve, sign. The seed is the last thing
touched, never the first. Above the approval threshold the operator signs the exact proposal at
their own terminal with a key the agent never holds; [`SECURITY.md`](../SECURITY.md) says exactly
what holds now.

![How a send works](how-a-send-works.svg)

## What the agent can and cannot do

| Can | Cannot |
|---|---|
| read balance, addresses, history, sync state | change the policy through any tool |
| propose a send and confirm it within the policy | approve its own above-threshold send (only `zumbra approve`, with the operator's passphrase, can) |
| validate an address | unlock a locked wallet |
| pay an x402 paywall within the policy | export, print or see the seed |

Shielding transparent funds is an operator action on the CLI. No MCP tool reads or writes the policy. The file itself is protected by the operating system,
not by Zumbra: if the agent runs as your OS user, it can edit the file with a text editor. Run
the wallet as a separate user from the agent; `SECURITY.md` lists this as an open item.

## Where things live

| Path | What |
|---|---|
| `~/.zumbra/<network>/` | wallet database, audit log, pending proposal, policy |
| `~/.ows/wallets/<name>.json` | the encrypted vault with the seed |
| `policy.toml` | per-tx cap, daily cap, allowlist, approval threshold, in zatoshi |
| `operator.key`, `operator.pub` | the operator's approval key: private half encrypted under a typed passphrase, public half in the clear |
| `proposals/`, `approvals/` | recorded proposals for `zumbra approve` to show; signed, single-use approvals |

Testnet and mainnet are separate directories and separate vault wallets. Start on testnet.
