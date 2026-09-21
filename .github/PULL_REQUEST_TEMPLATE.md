## What and why

The diff already says what changed. Explain **why**, and which plan item it closes.

Closes #

## Checklist

- [ ] `bash scripts/release-gate.sh` passed. Paste the tail below.
- [ ] No mocks added. Tests run against real processes; chain tests use testnet.
- [ ] Every new test can actually **fail**. For each one I asked *"what would this assert if the
      function returned an empty result?"*
- [ ] If this touches `policy.rs`, `audit.rs`, `send.rs`, `hitl.rs`, the vault, or any MCP tool
      that can spend: **mutation-tested**. I broke it on purpose, watched a test fail, restored,
      and confirmed green.
- [ ] No new outbound host, affiliate key, or telemetry.
- [ ] No key material on stdout by default, on argv, in a tool result, or in a log line.
- [ ] No MCP tool that approves, unlocks, or raises a limit.
- [ ] If it changes what the README or site claims: the claim maps to a command that works.
- [ ] The plan's decision table updated if a decision was made (owner-private).

## Gate output

```
paste the real tail of scripts/release-gate.sh
```

## What I did NOT test

Be honest here. Name the weakest part of the change and what is most likely to break first.
