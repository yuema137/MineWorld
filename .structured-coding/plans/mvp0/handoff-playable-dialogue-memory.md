# Handoff — playable dialogue and memory, PR-B (C2–C5)

Continuation aid only. The authority is the frozen design
[`pr-playable-dialogue-memory.md`](pr-playable-dialogue-memory.md) (DM-20261127-3) and its contract; this file never
overrides either. It replaced the PR-A handoff at the PR-A → PR-B boundary (PR-A's evidence is in design §10 PR-A).

## Identity

| Field | Value |
| --- | --- |
| PR | PR-B = commits C2–C5 (profile/store/contracts, cognition child, corrected memory, package + live gate) |
| Branch / worktree | `mvp0/pr-dialogue-b` in `/Users/yuema137/mineworld-worktrees/impl-dialogue-b` (one session) |
| Base | `origin/main` @ `b0b4690397c37a95a03aa7879fd45d9d6676f43a` (#549 merge; contains #544, #548, #549 and PR-A #547) |
| Sub-lane | C4 (correction scenario, isolation, MCP case, live-gate harness) by a delegated agent in `/Users/yuema137/mineworld-worktrees/impl-dialogue-b-c4`, branch `mvp0/pr-dialogue-b-c4` (same base, not pushed); its commits are cherry-picked here |
| Live model | only the §6 gate, once, on the final head (qwen3.5:4b on local Ollama; ≤ 8 calls) |
| Parallel lanes touching nearby files | launcher-solo #550 (`tools/launch/src/{config,hosting,lib}.rs`), nightlife N1 #552 (economy, controllers) |

## State

See design §10 PR-B for the ledger (implementation, validation, review per commit). Next action is recorded there.

## Scratch locations

- `CARGO_TARGET_DIR=/Users/yuema137/mineworld-worktrees/_targets/dialogue-b` (server binary for Python real-binary
  tests: `MINEWORLD_BIN=…/debug/mineworld`).
- `/Users/yuema137/mineworld-worktrees/_scratch/dialogue-b/` (manual manifest/preflight smoke; not evidence).
