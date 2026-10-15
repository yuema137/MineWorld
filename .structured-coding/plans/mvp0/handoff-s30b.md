# Handoff — S30-b (the authoring seam)

Continuation aid only. The authority is [`pr-s30b-authoring-seam.md`](pr-s30b-authoring-seam.md) (DESIGN FROZEN
2026-10-15), its §12 contract and §15 ledger in particular.

| Field | Value |
| --- | --- |
| PR | S30-b — the authoring seam: owned data directories, `SectionInContext`, `ResolvesSubjects`, `SystemPack::SHIPS` |
| Worktree | `/Users/yuema137/mineworld-worktrees/impl-s30b` (this session only) |
| Branch / base | `mvp0/pr-s30b`, from `origin/main @ 38323d4c` (#238, S30-a) |
| Primary design | `.structured-coding/plans/mvp0/pr-s30b-authoring-seam.md` |
| Binding parents | `step-30-person-attributes.md` §§3, 4.3, 8.5, 11–14; `pr-s30a-attribute-specs.md`; `docs/MODULE_SPEC.md` §§2, 3.1, 4.1; `docs/DECISIONS.md` `ARC-31`, `ARC-61`, `ARC-71`, `ARC-121`, `DEP-10`; `CLAUDE.md` |
| Approved scope | §2.1 items 1–6 of the design |
| Frozen invariants | I-S30-10; I-b-1 … I-b-9; §8 AC-b-1 … AC-b-9; no existing pack changes; `acme-fishing` builds |
| Endpoint authority | implementation, commits, push, PR create/update, CI repair: authorized (design §12; dispatch 2026-10-15). Merge: NOT authorized |
| Sequence | C0 … C7 (design §9) |
| Validation budget | targeted crate tests one at a time (R-LOAD-1, heavy runs through `cast-slots/with-slot.sh`); clippy on touched crates; one nightly dispatch in C7 |
| Stop condition | S30-b READY FOR OPERATOR REVIEW — DO NOT MERGE |

## State

- C0: base recorded, parent-doc edits applied, this handoff opened.
- Next: C1 (`authoring/src/data.rs`).

## No background jobs.
