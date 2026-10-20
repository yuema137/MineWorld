# Handoff — PR S28-e (memory inspection, seat export / import)

A continuation aid only. The authority is `pr-s28e-memory-inspect.md` (design, contract §12, ledger §15).

```text
PROJECT / PR         MVP-0 · S28 / PR S28-e — mineworld-memory inspect, export, import
PRIMARY DESIGN DOC   .structured-coding/plans/mvp0/pr-s28e-memory-inspect.md
BRANCH / WORKTREE    mvp0/pr-s28e · /Users/yuema137/mineworld-worktrees/impl-s28e (one writer)
BASE                 mvp0/pr-s28b @ 2c13d9c9 (stacked, D-e-0)
FROZEN               DESIGN FROZEN 2026-10-18 (QS28e-1 … QS28e-4 as recommended)
VALIDATION           SPEED FIRST: ruff, ruff format, pyright, the commit's test files one at a time
ENDPOINT             push mvp0/pr-s28e; no PR (the coordinator sequences CI); never merge
C8                   N/A: S28-c is not on the base
```

## Current checkpoint

C0 … C7 and C9 committed and pushed. Implementation complete.

## Open items

1. `test_inspect_real.py` (AC-e-3, `real_binary`) has not run: it needs `cargo build -p mineworld-cli`, which the
   shared slots never let start. CI or the milestone run must run it.
2. The S28-a transparency set and S28-b's Gate 2 re-runs: milestone (SPEED FIRST).
3. QS28e-5 (F-e-6): operator question, recommended answer in §14.
4. C8 (the `scopes` view) passes to S28-c (R-S28e-c-1); if S28-c merges first, this branch adds it.
5. If S28-b changes before it merges, rebase this branch onto it.
