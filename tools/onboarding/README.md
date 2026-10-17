# tools/onboarding/

The onboarding test: give a fresh agent only the guide and the reference, ask it for a world, and see whether it
gets there. The rules are in [`PROTOCOL.md`](PROTOCOL.md).

```sh
uv run python tools/onboarding/run.py --instructions ON-1 --agent-command claude-allowlist --budget 5 \
    --binary target/debug/mineworld --out /tmp/onboarding-round-1
```

It runs on the operator's machine only, under their own Claude Code login, never in CI. The agent may run only
`mineworld` and touch only the sandbox; every other tool call is denied before it runs. Prove that first:

```sh
uv run python tools/onboarding/escape_test.py --binary target/debug/mineworld --out /tmp/onboarding-escape
```

Its own tests need no model:

```sh
cargo build -p mineworld-cli
uv run --locked pytest tools/onboarding
```
