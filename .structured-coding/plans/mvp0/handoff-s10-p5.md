# Handoff — S10: PR P5a implementation (model backends, recorder, budgets, API keys)

A continuation aid, never a design authority. The authority is
[`pr-s10-p5-backends.md`](pr-s10-p5-backends.md) (DESIGN FROZEN 2026-10-09, with the provider-preset
amendment), with its live ledger in §13. A file of its own because several lanes run in parallel.

```text
PROJECT / PR        MVP-0 · S10 (step-17) / PR P5a — model backends, recorder and cassettes, budgets,
                    API keys (cognition/lm-controller, mineworld-cognition)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/pr-s10-p5-backends.md; ledger §13
RELATED / BINDING   step-17-cognition.md §§3.5, 3.7, 3.10, 3.11, 4.1, 4.5, 5, 15; pr-s10-p3-python-sdk.md;
                    overall.md S10 and §5; docs/ARCHITECTURE.md §9; docs/MODULE_SPEC.md §5;
                    docs/ENGINEERING_STANDARDS.md §§22–24; docs/REUSE_POLICY.md; CLAUDE.md
BRANCH / WORKTREE   mvp0/pr-s10-p5-backends in /Users/yuema137/mineworld-worktrees/impl-s10-p5 (sole writer)
BASE                origin/main @ 2c6d34c (#111); origin/main merged at cf18713 (#97, #114), no conflict
APPROVED SCOPE      design §2.1 (+ the amendment's providers.py); diff limited to AP5-11's list
FROZEN INVARIANTS   §2.3; D-P5-1 … D-P5-15; QS10-18; QS10-19; QP5-1 … QP5-12; every platform
ENDPOINT AUTHORITY  implementation + local validation, commits, push, PR create/update, CI repair:
                    authorized (primary session freeze, design §11); merge: operator only
STOP CONDITIONS     READY FOR OPERATOR REVIEW — DO NOT MERGE
CURRENT CHECKPOINT  READY FOR OPERATOR REVIEW — DO NOT MERGE. C0 … C8 complete (§13.1 … §13.10).
                    C7's operator step (the spike) NOT RUN. Context CLOSED / AWAITING OPERATOR ACTION.
                    Final heads and the PR's CI runs: in the PR body (a commit cannot carry its own run).
                    Scratch branch scratch/s10-p5-platforms is evidence only (F-P5-4); delete after review.
NEXT ACTIONS        operator review and the merge decision. The operator runs the spike (README command)
                    before P6 freezes; an agent then commits the report and cassette. After merge: record
                    the merge identity in §13; the S10 planning session updates step-17 §15 and overall.md,
                    including F-P5-4's note on the SDK suite's Windows guard.
```
