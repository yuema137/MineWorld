# Handoff — S10: PR P3 implementation (the Python protocol SDK)

A continuation aid, never a design authority. The authority is
[`pr-s10-p3-python-sdk.md`](pr-s10-p3-python-sdk.md) (DESIGN FROZEN 2026-10-08), with its live ledger in
§12. A file of its own rather than `handoff.md`, because several lanes run in parallel.

```text
PROJECT / PR        MVP-0 · S10 (step-17) / PR P3 — the Python protocol SDK (sdk/python, mineworld-sdk)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/pr-s10-p3-python-sdk.md; ledger §12
RELATED / BINDING   step-17-cognition.md §§3.2, 3.4, 4, 5, 15; overall.md S10 and rulings 1–8;
                    server/PROTOCOL.md rev 2; server/tests/frames/*; docs/ARCHITECTURE.md §13;
                    docs/MODULE_SPEC.md §5; docs/ENGINEERING_STANDARDS.md; docs/REUSE_POLICY.md; CLAUDE.md
BRANCH / WORKTREE   mvp0/pr-s10-p3-sdk in /Users/yuema137/mineworld-worktrees/impl-s10-p3 (sole writer)
BASE                origin/main @ 827daf9 (#86 merged: the P3 freeze). #83 (S11-B) is NOT in the base
                    (open, head 39d02ea): C2 models S11-A's frames; C6 applies only if #83 merges
                    during P3 (D-P3-5)
APPROVED SCOPE      design §2.1; diff limited to AP-9's list
FROZEN INVARIANTS   §2.3 (I-1, I-2, I-7, I-10, I-11, INV-9, INV-13); D-P3-1 … D-P3-11; §11.1 rulings
                    (QP3-3's CI condition: fast/test neither renamed nor slowed; C5 matrix rule)
ENDPOINT AUTHORITY  implementation + local validation, commits, push, PR create/update, CI repair:
                    authorized (primary session freeze, design §10); merge: operator only
SEQUENCE            C0 … C7 (C5 under QP3-3's condition; C6 conditional)
VALIDATION BUDGET   unit/static/local integration unrestricted; no LLM, no model, no key, no hosted API;
                    CI: C5's measuring run and one scratch mutation branch
STOP CONDITIONS     normal: READY FOR OPERATOR REVIEW — DO NOT MERGE; material: any Rust server or
                    protocol change, any dependency beyond DEP-S10-b/c/e, any hosted-API use
NOTE                2026-10-08: the coordinator reports the gh token is invalid while the operator
                    re-authenticates; a failed push or gh call is INCONCLUSIVE and retried later
CURRENT CHECKPOINT  C0 (freeze and contract recorded; handoff initialized)
NEXT ACTIONS        C1: workspace root, sdk/python skeleton, uv lock, standards.md, DECISIONS, ARCH §13.1
```
