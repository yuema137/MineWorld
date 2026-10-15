# Handoff — S30-a (person-attribute specifications)

Continuation aid only. The authority is [`pr-s30a-attribute-specs.md`](pr-s30a-attribute-specs.md) (DESIGN
FROZEN 2026-10-15), its §15 ledger in particular.

| Field | Value |
| --- | --- |
| PR | S30-a — person-attribute specifications (docs only) |
| Worktree | `/Users/yuema137/mineworld-worktrees/impl-s30a` (this session only) |
| Branch / base | `mvp0/pr-s30a`, from `origin/main @ 8c5d3598` (#234) |
| Primary design | `.structured-coding/plans/mvp0/pr-s30a-attribute-specs.md` |
| Binding parents | `step-30-person-attributes.md`; `pr-s30b-authoring-seam.md` §5; `step-31-wealth-prices.md` §§7.4, 8.5; `CLAUDE.md` §2 |
| Endpoint | PR READY FOR OPERATOR REVIEW — DO NOT MERGE |
| Validation | `python3 scripts/ci_layer.py docs` (the doc checks); CI on the exact head |

## State

- C0 committed: base, ids, merge states (#206 OPEN, P6a unpushed), rulings, deviations D-a-1, D-a-2.
- Next: C1 (`docs/CORE_CONCEPTS.md` §4.5), C2–C3 (`docs/MODULE_SPEC.md`), C4 (`docs/DECISIONS.md`), C4b (step-30
  reconciliation), C5 (`overall.md`, review table, push, PR).

## No background jobs.
