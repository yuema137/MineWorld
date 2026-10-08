# Handoff — S12 PR 13a (a world you can walk)

A continuation aid, never a design authority. The authority is
[`step-13-client-2d.md`](step-13-client-2d.md) §14 (frozen 2026-10-08).

Kept as its own file rather than a block in `handoff.md` (bounded deviation, §14.10 D-1): six lanes
run in parallel and every one would append to the same file.

```text
PROJECT / PR        S12 PR 13a — a world you can walk
DESIGN              step-13-client-2d.md §14; contract §14.9
BRANCH / WORKTREE   mvp0/pr-s12-13a-walkable-2d · /Users/yuema137/mineworld-worktrees/impl-s12-13a
BASE                main @ 47c81d1; main merged in at d6bd41d (S11-A), 3b0d1e1 (test hygiene #77)
SCOPE               §14.0–§14.7; frozen invariants §14.9
ENDPOINTS           implement, commit, push, PR (READY FOR OPERATOR REVIEW, preview) — primary session's
                    freeze message 2026-10-08 and D-12; merge: operator only
STOPS               any edit to clients/protocol/mineworld/**; any server/kernel/contract/pack change
```

## Checkpoint

- C0 `26c451d`; freeze `610b50f`; C1 `318c0b9`; C2 `4f6f6b2`; C3–C5 client `74b7c2c`; C5/C6 tests and
  reconnect `bc935c4`; revision 2 `a00e87f`; F-6 `f4e0a75`; F-7 `42884a3`; merge of #77 `3b0d1e1`.
- D-12 test worlds on port 0 `20061a3`; painter's order check `2426c5e`; preview stills `51afae6`;
  C7 ledger with the final gates (this commit).

- Operator test failed (F-9 room grows, F-10 no way out): red drive `b6624ad`; fix `9e433cb`;
  main (CI) merged `13e37b4`; D-14 `f83d7c3`; F-11 and retaken stills `2add1f3`.
- Server gap reported, not patched: no place extent; movement bounds only the stride (12d walls or
  movement's place extent).

## Next actions

1. The PR is open, READY FOR OPERATOR REVIEW (preview). Await the operator; do not merge.
2. After merge: §14's lifecycle and evidence (this session's sync duty, §14.9).

## Background processes

Only processes this session started (by PID), never by name. The host restarted once mid-run
(2026-10-08); its leftover scratch is F-8. Cargo is at `$HOME/.cargo/bin` (not on the restarted
host's PATH).
