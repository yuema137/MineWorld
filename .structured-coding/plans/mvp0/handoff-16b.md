# Handoff — S14: PR 16b implementation (the shared module)

A continuation aid, never a design authority. The authority is
[`step-15-demo-3d.md`](step-15-demo-3d.md) §18 (DESIGN FROZEN 2026-10-08, primary session), with
evidence in §18.9 (`E-B<n>`) and deviations in §18.10. Per-PR file (QSB-3): `handoff.md` belongs to
other lanes.

```text
PROJECT / PR        MVP-0 · Step 15 (S14) / PR 16b — shared module: affordance readers, raw components,
                    revision
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-15-demo-3d.md §18
RELATED / BINDING   §18.11's list (overall.md parallel build-out rulings 4, 6, 7, 9, 10; step-15 §§10,
                    12, 13; step-13 §7; ARC-31, ARC-34; PROTOCOL.md §§5–6; ADOPTION.md; CORE_CONCEPTS
                    §15.2)
BRANCH / WORKTREE   mvp0/pr-16b-shared-module in /Users/yuema137/mineworld-worktrees/impl-shared-module
                    (sole writer)
BASE                main @ 47c81d1; design commits 5682784, eb17191 (freeze header)
APPROVED SCOPE      §18.1's paths; B-C1 … B-C4; SB-1 … SB-7; QSB-1 … QSB-7 as recommended
FROZEN INVARIANTS   §18.11 (no edit outside §18.1; demo.gd, space.gd, clients/3d-spike/**, committed
                    evidence, PROTOCOL.md, Rust, worlds untouched; request-{2d,3d}.json byte-identical;
                    no rule / action literal / engine type in mineworld/; submit_affordance never reads
                    `available`; scratch data removed)
ENDPOINT AUTHORITY  implementation, commits, push, PR (READY FOR OPERATOR REVIEW): authorized by the
                    primary session's freeze message 2026-10-08. CI: N/A. Merge: operator only; lands
                    before S11-A
VALIDATION BUDGET   §18.11 (~1 hour; run.sh evidence ≤ 3 times)
CHECKPOINT          see the live state below
STOP CONDITIONS     NORMAL: READY FOR OPERATOR REVIEW — DO NOT MERGE. MATERIAL: §18.11
```

## Live state

- Checkpoint: **READY FOR OPERATOR REVIEW** (2026-10-08). B-C0 … B-C4 done. Final executable head
  `5c10e06`; the PR head is the closing ledger commit (Markdown only). Context CLOSED / AWAITING
  OPERATOR ACTION.
- Evidence: step-15 §18.9 E-B0 … E-B9; deviations D-1 (typed references in `affordances_about`), D-2
  (integers restored by `submit_affordance`) in §18.10.
- Next (operator / primary session): review and merge before S11-A; post-merge sync as §18's header
  lists. No background jobs running; scratch saves removed.
