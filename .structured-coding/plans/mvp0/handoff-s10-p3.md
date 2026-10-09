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
                    (resolved the same day: pushes and gh calls worked throughout)
CURRENT CHECKPOINT  READY FOR OPERATOR REVIEW — DO NOT MERGE. C0–C7 complete (C6 live: #83 merged during
                    P3). Ledger §12.1–§12.1h, evidence E-P3-0…8, mutations M-1…M-13, deviations
                    DV-P3-1…5. Context CLOSED / AWAITING OPERATOR ACTION.
                    Final executable head = final PR head = the commit carrying this line; its CI run
                    and the like-for-like `test` comparison with main's run 37977732034 (a30755e) are
                    recorded in the PR body, which is updated after the run.
                    Scratch branch scratch/s10-p3-mutation (5c8f195) is evidence only; delete after
                    review if wanted.
REVIEW, 2026-10-09  primary session approved #98 (its own mutation of offers.attempt's "not among
                    this observation's affordances" check was caught). #98 then conflicted with main
                    (TW-a, S19 plan): origin/main merged at 7e87ceb; the one conflict was
                    docs/DECISIONS.md (ARC-56/DEP-24..26 beside TW-a's ARC-67…), both kept; no golden
                    frame, PROTOCOL.md, CI or Python file changed on main's side. Re-checked on the
                    merge: cargo fmt, clippy -D warnings, binary rebuilt; ruff, format, pyright strict,
                    pytest 32 passed (real server on main's new binary); doc, pin and scratch checks
NEXT ACTIONS        operator review and the merge decision. If main moves before merge: merge
                    origin/main; if a golden frame changed, update sdk/python (R-S11-9); re-run the
                    python layer. After merge: record the merge identity in §12; the S10 planning
                    session updates step-17 §15 and overall.md (DEP id shortage and the 13w overlap
                    are in §12.1h).
```
