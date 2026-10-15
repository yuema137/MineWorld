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

- C0, C1, C2, C3, C4, C4b, C5 committed (see the ledger's commit table). Docs only.
- Findings recorded during implementation: F-S30a-7 … F-S30a-11 (bounded, each decided in the spec), with
  requirements R-S30c-a-1, R-S30e-a-2, R-S30h-a-1, R-S30k-a-1, R-S31-a-2 and four primary edits in §11.
- After push: the PR number, the exact head SHA and its CI result are reported to the primary; the session then
  closes as AWAITING OPERATOR ACTION. Merge is not this session's.
- Post-merge (primary): ledger → MERGED; step-30 and `overall.md` S30-a merged; R-S26a-a-1 / R-P6a-a-1 notes
  under `ARC-91` / `ARC-74` by whichever merges second.

## No background jobs.
