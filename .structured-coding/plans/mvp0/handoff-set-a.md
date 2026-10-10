# Handoff — S20 PR SET-a (the shared settings module, language, display, clock and menu)

A continuation aid, never a design authority. The authority is
[`step-20-client-settings.md`](step-20-client-settings.md) §12 (DESIGN FROZEN 2026-10-08), its rulings
(§1.5) and its execution contract (§12.9). Kept as its own file, because lanes run in parallel.

```text
PROJECT / PR        S20 PR SET-a — settings module, language, display, clock, menu, in both clients
DESIGN              step-20-client-settings.md §12; contract §12.9; ledger §12.10; findings §12.11;
                    acceptance §12.12; operator checklist §12.13
BRANCH / WORKTREE   mvp0/pr-set-a · /Users/yuema137/mineworld-worktrees/impl-set-a (this session only)
BASE                origin/main @ aee8290; origin/main merged at 72efd61
ENDPOINTS           implement, commit, push, PR create/update, CI repair: authorized (§12.9).
                    Merge: operator only — DO NOT MERGE.
STATE               CLOSED / AWAITING OPERATOR ACTION once the PR is READY FOR OPERATOR REVIEW with
                    fast and test green on its exact head (head and runs in the PR).
```

## What the operator needs to know

- Windows and Linux hand checks H-1 … H-10 are open (§12.13).
- Decision numbers: ARC-76, DEP-35, DEP-36, allocated by the primary session 2026-10-09 (F-10;
  the provisional ARC-72/DEP-32/DEP-33 belonged to S10 and were renumbered).
- F-15 and F-16 accepted by the primary session as bounded.
- F-15: the 3D toast's reason words follow 13b's wording (`busy` → "busy right now").
- F-16: the clock setting governs the HUD time line; 13b's schedule rows stay 24-hour.
- AC-SET-9's still byte-identity cannot discriminate (stills differ run to run with no settings at all).

## After merge

This session (if resumed): §12 lifecycle → MERGED with the merge commit. The primary session: step-20
§§1–11 and `overall.md`.

## Background processes

None.
