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

- C0 design `26c451d`; freeze header `610b50f`.
- Current: C1 (specs before code).

## Next actions

1. C1: DECISIONS (ARC-14 port, ARC-45…47, DEP-16), MVP §7.1, ART_DIRECTION §20, MODULE_SPEC §6.1 pointer,
   clients/2d/PRESENTATION.md, ADOPTION §6.
2. C2 … C7 per §14.6.

## Background processes

None.
