# Handoff — S27-a (seat grants and a per-seat request budget)

Continuation aid only; the authority is the PR design's ledger (§15).

```text
PROJECT / PR:          MineWorld mvp0, S27-a — seat grants and a per-seat request budget
PRIMARY DESIGN DOC:    .structured-coding/plans/mvp0/pr-s27a-seat-grants-budget.md (DESIGN FROZEN 2026-10-14)
BINDING DOCS:          step-27-agent-players.md (§§2.2, 3, 6.3–6.5, 7.2, 8, 11, 12, 13.1, 16);
                       server/PROTOCOL.md; docs/NETWORKING.md §§2, 9; docs/DECISIONS.md ARC-40, ARC-41,
                       ARC-44, ARC-67, ARC-75, DEP-14; docs/REUSE_POLICY.md; clients/protocol/ADOPTION.md
WORKTREE / BRANCH:     /Users/yuema137/mineworld-worktrees/impl-s27a · mvp0/pr-s27a
IMPLEMENTATION BASE:   origin/main @ 708bc5679de1d7121f7b40f65a6fae2d3bbcd375
CARGO_TARGET_DIR:      /tmp/impl-s27a-target (prototype: /tmp/impl-s27a-proto-target)
APPROVED SCOPE:        design §2.1 items 1–15
FROZEN INVARIANTS:     design §0, I-a-1 … I-a-10, AA-1 … AA-12; R = 10, B = 20 (fixed at C0)
ENDPOINT AUTHORITY:    implementation + local validation, semantic commits, branch push, PR create/update,
                       CI repair: authorized (design §12, primary session at freeze); merge: never
                       (operator only). Local validation narrowed by coordinator ruling R-LOAD-1
                       (2026-10-14): touched crates only + fmt + clippy; workspace gate = CI.
SEQUENCE:              C0 … C6 (design §9)
VALIDATION BUDGET:     design §12; Godot measurement ≤ 15 min (used ≈ 5 min)
STOP CONDITION:        S27-a READY FOR OPERATOR REVIEW — DO NOT MERGE
```

## Current checkpoint

C0 committed (measurement, trace, ARC-97, DEP-57). C1's spec edits are in the working tree
(`server/PROTOCOL.md`, `docs/NETWORKING.md`, `clients/protocol/ADOPTION.md`, `server/README.md`).

## Next actions

1. Commit C1 after the doc checks.
2. C2: the two codes in `server/src/protocol.rs`, goldens, SDK, `link.gd`, catalogs.
3. C3 … C6 as the design says. No background jobs are running.
