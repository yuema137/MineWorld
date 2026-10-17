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
| Sequence | C0 … C7 (design §9) — all committed |
| Validation budget | targeted crate tests one at a time (R-LOAD-1); clippy on touched crates; one nightly dispatch |
| Stop condition | S30-b READY FOR OPERATOR REVIEW — DO NOT MERGE |

## State

- C0 `0a11d021`, C1 `9797ecd7`, C2 `d8bdc4a0`, C3 `ee0d2ea4`, C4 `6e6d0a59`, C5 `b436c03b`, C6 `08e80e5f`, C7 the PR head.
- Mutations M-b-1 … M-b-7: each red, restored (`cmp`), green — ledger §15.
- After push: PR "S30-b: the authoring seam for owned data"; CI on the exact head; one `nightly.yml` dispatch for AC-b-5's
  `baseline check`. Their results are reported to the primary and recorded in the PR, not in a commit.
- Post-merge (primary): ledger → MERGED; step-30 (§4.3 item 3's `resolve_subjects`, §8.5 item 5 `Subject`) and `overall.md`.
- Open for later PRs: R-S30c-b-1, R-S30c-b-2, R-S31-b-1 (design §10); F-b-1 (F-S30a-9/10 stay open, not foreclosed),
  F-b-2 (sizes), F-b-3 (`validate_entity_pack` has no context).

## Background jobs

None locally. CI and the nightly run on GitHub.
