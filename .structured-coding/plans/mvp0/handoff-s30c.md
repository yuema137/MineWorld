# Handoff — S30-c (the `attributes` System Pack and `mineworld-attributes-model`)

Continuation aid only. The authority is [`pr-s30c-attributes-pack.md`](pr-s30c-attributes-pack.md) (DESIGN FROZEN
2026-10-18), its §13 contract and §16 ledger in particular.

| Field | Value |
| --- | --- |
| PR | S30-c — the `attributes` System Pack and the attribute model crate |
| Worktree | `/Users/yuema137/mineworld-worktrees/impl-s30c` (this session only) |
| Branch / base | `mvp0/pr-s30c-attributes-pack` (the dispatch's name; the design's `mvp0/pr-s30c` is the same branch), from `origin/main @ b85f2804` (#260) |
| Primary design | `.structured-coding/plans/mvp0/pr-s30c-attributes-pack.md` |
| Binding parents | `step-30-person-attributes.md` §§3, 4, 5, 7, 9, 11–13; `pr-s30a-attribute-specs.md`; `pr-s30b-authoring-seam.md`; `docs/CORE_CONCEPTS.md` §4.5; `docs/MODULE_SPEC.md` §§2, 3.1, 4.1, 8.1; `docs/DECISIONS.md` `ARC-25`, `ARC-26`, `ARC-28`, `ARC-31`, `ARC-61`, `ARC-120` … `ARC-125`, `DEP-10`; `CLAUDE.md` |
| Approved scope | design §2.1 items 1–7 as §15's rulings shape them |
| Frozen invariants | I-S30-1 … I-S30-5, I-S30-8, I-S30-10; design §6 I-c-1 … I-c-10; §8 AC-c-1 … AC-c-14 |
| Endpoint authority | implementation, local validation, commits, push, PR create/update, CI repair: authorized (design §13; dispatch 2026-10-18 "When S30-c is ready for review, report"). Merge: NOT authorized (working rules §22) |
| Sequence | C0 … C9 (design §9) |
| Validation budget | targeted crate tests one file at a time (R-LOAD-1); heavy local jobs only through `cast-slots/with-slot.sh`; clippy on touched crates; one nightly dispatch in C9; validate locally before every push |
| Stop condition | S30-c READY FOR OPERATOR REVIEW — DO NOT MERGE; then S30-d, then S30-e, each its own stacked PR |

## State

- C0: base recorded (design §16).
- Next: C1 — seam item 5, the `resolve_subjects` rename, the R3b ordering test.

## Background jobs

None.
