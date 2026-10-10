# Handoff — S10: PR P4 implementation (subjective memory, compression L0–L3, retrieval, AC-10)

A continuation aid, never a design authority. The authority is
[`pr-s10-p4-memory.md`](pr-s10-p4-memory.md) (DESIGN FROZEN 2026-10-10, revision 2), with its live
ledger in §14.

```text
PROJECT / PR        MVP-0 · S10 (step-17) / PR P4 — memory/ and compress/ in mineworld-cognition;
                    AC-10 (IC-4) over the real binary. GitHub PR #143
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/pr-s10-p4-memory.md; ledger §14
RELATED / BINDING   step-17-cognition.md (§§3.8, 3.9, 3.16.3, 4.3, 4.4, 4.7, 5, 6, 10, 15);
                    pr-s10-p5-backends.md (gateway, recorder, R-P4-1); pr-s10-p5b-hosted-subscriptions.md;
                    pr-s10-p3-python-sdk.md; step-12-server.md §17; overall.md (S10; decision table);
                    docs/ARCHITECTURE.md §8; docs/CORE_CONCEPTS.md §5; docs/REUSE_POLICY.md; CLAUDE.md
BRANCH / WORKTREE   mvp0/pr-s10-p4-memory in /Users/yuema137/mineworld-worktrees/impl-s10-p4 (sole writer;
                    the replacement session since 2026-10-10, after the first stopped before C0's commit)
BASE                origin/main @ 573c205 (#139)
HEAD / FINGERPRINT  see `git log -1` and `git status --short` (kept current at each milestone)
APPROVED SCOPE      design §2.1 as frozen; diff limited to AP4-13's list
FROZEN INVARIANTS   §2.3 (I-1 … I-13, P4-1 … P4-5); D-P4-1 … D-P4-13; QS10-11; QS10-17; QS10-18;
                    QS10-19; every platform (Linux, macOS, Windows)
ENDPOINT AUTHORITY  implementation + local validation, semantic commits, branch push, PR create/update,
                    CI repair to review readiness: authorized (primary session, 2026-10-10; working
                    rules §§14, 21). Merge: never by this session; only after the primary session's
                    review and the operator's explicit authorization (§12)
SEQUENCE            C0 … C8 (§10); C2–C4 landed as one commit (§14.1)
VALIDATION BUDGET   unit, static, local integration (incl. 100-day runs): unrestricted. Real model calls:
                    none. CI: the PR's runs; no manual dispatch, no scratch branch
STOP CONDITIONS     READY FOR OPERATOR REVIEW — DO NOT MERGE; material stops per §12
CURRENT CHECKPOINT  C0–C8 committed; CI run 38037220947 on b724861 green on every job; FTS5 present on
                    all three legs. MATERIAL STOP (§14.5): test_ac10.py took 198.8 s on windows-2025
                    (> the 120 s stop line of §12); Linux 71.6 s, macOS 66.7 s.
NEXT ACTIONS        Wait for the primary session / operator ruling on §14.5 (a)/(b)/(c). After it:
                    apply only what is ruled, re-run CI on the new head, record §14.4, close C1/C7/C8
                    validation boxes, hand off READY FOR OPERATOR REVIEW. Never merge.
```
