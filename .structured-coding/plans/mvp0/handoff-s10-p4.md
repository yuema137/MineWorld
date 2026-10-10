# Handoff — S10: PR P4 implementation (subjective memory, compression L0–L3, retrieval, AC-10)

A continuation aid, never a design authority. The authority is
[`pr-s10-p4-memory.md`](pr-s10-p4-memory.md) (DESIGN FROZEN 2026-10-10, revision 2), with its live
ledger in §14.

```text
PROJECT / PR        MVP-0 · S10 (step-17) / PR P4 — memory/ and compress/ in mineworld-cognition;
                    AC-10 (IC-4) over the real binary
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/pr-s10-p4-memory.md; ledger §14
RELATED / BINDING   step-17-cognition.md (§§3.8, 3.9, 3.16.3, 4.3, 4.4, 4.7, 5, 6, 10, 15);
                    pr-s10-p5-backends.md (gateway, recorder, R-P4-1); pr-s10-p5b-hosted-subscriptions.md;
                    pr-s10-p3-python-sdk.md; step-12-server.md §17; overall.md (S10; decision table);
                    docs/ARCHITECTURE.md §8; docs/CORE_CONCEPTS.md §5; docs/REUSE_POLICY.md; CLAUDE.md
BRANCH / WORKTREE   mvp0/pr-s10-p4-memory in /Users/yuema137/mineworld-worktrees/impl-s10-p4 (sole writer)
BASE                origin/main @ 573c205 (#139, which records ARC-59 / DEP-37 in overall.md)
HEAD / FINGERPRINT  see `git log -1` and `git status --short` (kept current at each milestone)
APPROVED SCOPE      design §2.1 as frozen; diff limited to AP4-13's list
FROZEN INVARIANTS   §2.3 (I-1 … I-13, P4-1 … P4-5); D-P4-1 … D-P4-13; QS10-11; QS10-17; QS10-18;
                    QS10-19; every platform (Linux, macOS, Windows)
ENDPOINT AUTHORITY  implementation + local validation, semantic commits, branch push, PR create/update,
                    CI repair to review readiness: authorized (primary session, 2026-10-10; working
                    rules §§14, 21). Merge: never by this session; only after the primary session's
                    review and the operator's explicit authorization (§12)
SEQUENCE            C0 … C8 (§10)
VALIDATION BUDGET   unit, static, local integration (incl. 100-day runs): unrestricted. Real model calls:
                    none. CI: the PR's runs; no manual dispatch
STOP CONDITIONS     READY FOR OPERATOR REVIEW — DO NOT MERGE; material stops per §12
CURRENT CHECKPOINT  C0 complete (freeze verified, base recorded, ledger §14 opened)
NEXT ACTIONS        C1: ARC-59 and DEP-37 in DECISIONS.md; G-2, G-6; real_binary marker; python-smoke
                    deselection; memory/fts.py and test_fts5_platform.py; push; draft PR; FTS5 per leg.
```
