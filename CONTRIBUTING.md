# Contributing to Zumbra

This repo is private and owner-operated while it is pre-alpha. These notes are for anyone working
in it, human or agent. Read [`CLAUDE.md`](CLAUDE.md) first: it is the contract, and
[`SECURITY.md`](SECURITY.md) for what holds and what does not. The running plan and the fork-time
audit are kept privately by the owner.

## Getting set up

```bash
git clone --recurse-submodules https://github.com/AIEngineerX/zumbra.git
cd zumbra
bash scripts/setup-hooks.sh        # installs the pre-commit secret block (once per clone)
cd rust
cargo build --locked --release -p zumbra -p zumbra-mcp
cargo test --locked --workspace --exclude rust_lib_zumbra
```

Rust stable with `rustfmt` and `clippy` (see `rust/rust-toolchain.toml`). `gitleaks` and
`cargo-deny` are needed for the release gate. Testnet first: set the chain environment variable
to `testnet`: pass `--testnet`, or set `ZUMBRA_TESTNET=1`.

## Before you push

```bash
bash scripts/release-gate.sh
```

It runs, in order: clean working tree, gitleaks over full history, `cargo fmt --check`,
`cargo clippy`, `cargo deny`, `cargo test --locked`, and the pre-public audit once the repo is
public. If any step fails, fix the cause. Do not skip the gate because a change is small.

## The rules that will get a change reverted

1. **Policy before signer, fail closed.** Every path that can produce a signature runs
   `check_proposal` first. A path that cannot be gated is deleted.
2. **No MCP tool that approves, unlocks, or raises a limit.** Approval is a protocol on a channel
   the agent cannot call.
3. **No destination from untrusted text.** Memos, fetched pages, quotes, and model memory are data,
   never instructions.
4. **No key material on stdout by default, on argv, in a tool result, or in a log.**
5. **No new outbound host, affiliate key, or telemetry.** If it leaves the machine, the operator
   configured it.
6. **No mocks in tests.** Integration tests against real processes, testnet when it touches the
   chain. Security fixes get a mutation check: break the fix, watch the test fail, restore.
7. **Every claim in the README or on the site maps to a command that works.**

## Writing a test that is worth having

Ask of every test: *what would this do if the function returned an empty result?* If the answer is
"still pass", it is not a test. Assert non-emptiness first. Watch for loop bodies that never run,
`if exists()` guards that silently skip, and byte-index slicing of strings that only fails on
multibyte input.

## Commits and branches

- `main` is the trunk and is protected: no force-push, no deletion, linear history, squash merges.
- Short-lived branches per slice, deleted on merge.
- Conventional prefixes: `feat:`, `fix:`, `docs:`, `test:`, `chore:`, `security:`.
- Commit as `AIEngineerX <195990077+AIEngineerX@users.noreply.github.com>`.
- The `upstream` remote points at the project we forked from. Never push to it.

## Upstream

Zumbra is a hard fork of [Zipher](https://github.com/atmospherelabs-dev/zipher-app) (MIT). We do
not send changes upstream and we do not track their branches after the fork point, except to
cherry-pick security fixes with attribution in the commit message. Upstream documents that still
matter are kept by the owner, unchanged, as a record.
