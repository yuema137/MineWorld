# Handoff — PR FE-a (food-energy engine)

Continuation aid only; the authority is `pr-fe-a-food-energy-engine.md` (ledger §9.1 and the C0–C4 items).

```text
PROJECT / PR:          MineWorld mvp0 — Step FE, PR FE-a: food-energy engine
PRIMARY DESIGN DOC:    .structured-coding/plans/mvp0/pr-fe-a-food-energy-engine.md
RELATED / BINDING:     pr-fe-a-contract.md, step-fe-food-energy.md, CLAUDE.md, docs/MODULE_SPEC.md, ARC-370
BRANCH / WORKTREE:     mvp0/pr-fe-a-food-energy @ /Users/yuema137/mineworld-worktrees/impl-fe-a
BASE:                  origin/main @ d252338e
APPROVED SCOPE:        contract APPROVED SCOPE (C0–C4); no worlds/* edits
FROZEN INVARIANTS:     contract FROZEN INVARIANTS
ENDPOINT AUTHORITY:    implement/commit/push/PR/CI repair authorized (operator 2026-11-27, Q-FE-5); merge: operator only
SEQUENCE:              C0 docs → C1 SDK+error seams → C2 valid-genesis-first → C3 needs v2 → C4 views → review → PR/CI
BUDGET:                R-LOAD-1: one test target at a time via cast-slots/with-slot.sh; own CARGO_TARGET_DIR; no full workspace
CURRENT CHECKPOINT:    see the design's C0–C4 checkboxes (authoritative)
STOP CONDITION:        READY FOR OPERATOR REVIEW — DO NOT MERGE
```

**State 2026-11-27:** C0–C4 + review fixes committed; PR #555 open; READY FOR OPERATOR REVIEW pending CI on the
final head (see PR). Next action for any resumed session: verify CI on the PR head; do not merge.
