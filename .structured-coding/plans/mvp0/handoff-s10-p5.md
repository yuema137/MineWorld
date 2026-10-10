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
BASE                origin/main @ 2c6d34c (#111)
APPROVED SCOPE      design §2.1 (+ the amendment's providers.py); diff limited to AP5-11's list
FROZEN INVARIANTS   §2.3 (I-1, I-2, I-7, I-10, I-11, I-16, INV-14, P5-1, P5-2, P5-3); D-P5-1 … D-P5-15;
                    QS10-18; QS10-19; QP5-1 … QP5-12 as ruled; every platform
ENDPOINT AUTHORITY  implementation + local validation, commits, push, PR create/update, CI repair:
                    authorized (primary session freeze, design §11); merge: operator only
SEQUENCE            C0 … C8 (C7's operator step NOT RUN at review is allowed)
VALIDATION BUDGET   unit/static/local integration unrestricted; no model run by agents, no key, no
                    hosted API, no key file read; CI: the PR's runs
STOP CONDITIONS     normal: READY FOR OPERATOR REVIEW — DO NOT MERGE; material: any Rust or sdk/python
                    change; any dependency beyond DEP-27/DEP-32/P3's toolchain; any provider SDK; any
                    hosted-API use; a real key in a test or example; a change to AP5-S's thresholds
CURRENT CHECKPOINT  C0 done (ledger §13.1). Next: C1
NEXT ACTIONS        C1: workspace member, lock, DECISIONS ARC-57/ARC-58/DEP-27/DEP-32, ARCHITECTURE §9.1/§9.2,
                    .gitignore, .gitattributes, ci_layer.py, standards.md; verify httpx2 facts
```
