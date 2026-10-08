# Handoff — S12 PR 13a (a world you can walk)

A continuation aid, never a design authority. The authority is
[`step-13-client-2d.md`](step-13-client-2d.md) §14 (frozen 2026-10-08).

Kept as its own file rather than a block in `handoff.md` (bounded deviation, §14.10 D-1): six lanes
run in parallel and every one would append to the same file.

```text
PROJECT / PR        S12 PR 13a — a world you can walk
DESIGN              step-13-client-2d.md §14; contract §14.9
BRANCH / WORKTREE   mvp0/pr-s12-13a-walkable-2d · /Users/yuema137/mineworld-worktrees/impl-s12-13a
BASE                main @ 47c81d1
SCOPE               §14.0–§14.7; frozen invariants §14.9
ENDPOINTS           implement, commit, push, PR (READY FOR OPERATOR REVIEW, preview) — primary session's
                    freeze message 2026-10-08 and D-12; merge: operator only
STOPS               any edit to clients/protocol/mineworld/**; any server/kernel/contract/pack change
```

## Checkpoint

- C0 design `26c451d`; freeze header `610b50f`; C1 `318c0b9`; C2 `4f6f6b2`; merge of main (16b)
  `38dfe93`; client code C3–C5 `74b7c2c`.
- Current: C5 tests (`tools/cli/tests/client_2d.rs`, `godot2d/`) and C6 reconnect, in the working tree;
  then C7 (stills, preview, final gates, PR).

## Next actions

1. Commit tests + reconnect once `cargo test -p mineworld-cli --test client_2d -- --ignored
   --test-threads=1` is green; record mutations (W1, W3, W5, W6, W7 done; W4, W10 pending).
2. C7: capture runs (town, none, full), stills into `clients/2d/shots/preview/`, final gates, PR.

## Background processes

Only processes this session started (by PID), never by name. Godot user dir may hold
`mutation_w3.txt` only during the W3 mutation; it was removed.
