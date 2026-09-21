# Security

Zumbra is a Zcash wallet that an AI agent will hold. The threat model starts from one assumption:
**the agent will be prompt-injected**, by an on-chain memo, a fetched web page, a swap quote, or a
document it was asked to read. Every control below exists so that a compromised agent still cannot
move funds beyond what its operator authorised.

## Reporting a vulnerability

Open a private security advisory on `AIEngineerX/zumbra` (Security tab, "Report a vulnerability").
Do not open a public issue for anything touching key handling, the spending policy, approvals,
the MCP server, or the vault. We will acknowledge within 72 hours.

No maintainer will ever ask you for a seed phrase, a passphrase, or a private key. Anyone who does
is attacking you.

## Status: what holds today, and what does not

This repository is a hard fork of Zipher at its `feat/ironwood` branch, staged 2026-09-20. It is
**pre-alpha**: run it on testnet, and do not hand an agent mainnet funds until the agent-safety work
below is done. A source audit was
done at fork time; its findings are summarised here and tracked privately by the owner.

Holds on the current code (verified by reading, not yet by running):
- Amounts are `u64` end to end; non-finite values cannot reach the policy check.
- On every ZEC send path the policy check runs before the seed is read; seed bytes are zeroised
  after key derivation; no key material is written to logs.
- MCP sends are two-step (propose, then confirm) with an unpredictable proposal ID, and the
  self-unlock and self-approve tools were removed upstream on 2026-09-10.
- Daily spending uses durable reservations; a corrupt policy file refuses to spend.
- The vault is scrypt + AES-256-GCM; the npm installer verifies SHA-256 checksums.

Does **not** hold yet (scheduled, in severity order):
- A max-amount send skips the policy check.
- Some spend paths (PCZT export, and the EVM and market paths being switched off) have no policy.
- Policy files, the audit database, and `policy set` are writable by the same OS user as the agent.
- Two MCP tools can fetch arbitrary URLs and return the body to the model.
- Untrusted strings (memos, fetched bodies) are not sanitised before reaching the model.
- Session bearer tokens appear in tool output.
- An empty vault passphrase is accepted by default.
- Swaps send a stable address and the deposit txid to the swap operator under upstream's
  affiliate key. Swaps are disabled until this is fixed.

## Non-goals

These will not be accepted as pull requests.

- **No MCP tool that approves, unlocks, or raises a limit.** Approval is a protocol on a channel
  the agent cannot call. If the model can reach it, it is not a control.
- **No destination from untrusted text.** Not memos, not fetched pages, not agent memory.
- **No custodial mode, no hosted keys.** We never hold anyone's funds.
- **No telemetry, no baked-in third-party hosts or affiliate keys.** Everything that leaves the
  machine is something the operator configured.
- **No key material on stdout by default, on argv, or in a tool result.**

## Controls we commit to keeping

- **Policy before signer, fail closed.** Missing, unparseable, zero, or wrong-typed policy values
  refuse to spend. Zero means zero.
- **Privilege separation.** The wallet runs as its own OS user or service; the agent's user cannot
  write the policy, the audit log, or the data directory.
- **Loopback by default.** Anything that listens binds `127.0.0.1` unless explicitly told
  otherwise, and rate limits never exempt loopback.
- **Sanitised output.** OSC 8, OSC 52, DCS, APC, ANSI and C0/C1 control sequences are stripped from
  every string that came from the network or the chain before it reaches a terminal or a model.
- **Supply chain.** `Cargo.lock` committed, `--locked` builds, actions pinned by SHA, `cargo deny`
  and `cargo audit` in CI, gitleaks over full history, and a pre-commit hook that blocks `.env`,
  key files and seed-shaped strings. Vendored Zcash crates carry SHA-256 provenance in
  `rust/vendor/sources.json`.

## Your responsibilities as an operator

- Use testnet first. Fund mainnet with what you can afford to lose.
- Set a real vault passphrase. Run the wallet as a different user from the agent.
- Keep the MCP server on stdio or loopback. Never expose it to a network.
- MIT's "AS IS" clause is a legal position, not protection from consequences.
