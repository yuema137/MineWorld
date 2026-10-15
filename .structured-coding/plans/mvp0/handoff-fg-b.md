# Handoff — step-24: PR FG-b implementation (the retention host, pluggable `ForgettingPolicy`)

A continuation aid, never a design authority. The authority is
[`pr-fg-b-retention-host.md`](pr-fg-b-retention-host.md) (DESIGN FROZEN 2026-10-12, revision 2), with its
live ledger in §15.

```text
PROJECT / PR        MVP-0 · step-24 / PR FG-b — the retention host (memory/retention/), the bundled no-LLM
                    policies (forgetting/), schema v2, the movement renderer, ledger pruning
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/pr-fg-b-retention-host.md; ledger §15
RELATED / BINDING   mvp1/step-24-decision-and-memory.md (§§1.2, 2.3–2.5, 3, 7, 8.3, 8.4, 8.6, 10, 11, 14.1);
                    pr-s10-p4-memory.md; pr-s10-p5-backends.md; pr-s10-p3b-perceived.md;
                    pr-fg-c-lm-summarise.md (R-FGc-FG-1); overall.md; docs/DECISIONS.md ARC-59, ARC-84,
                    ARC-87, DEP-37; docs/ENGINEERING_STANDARDS.md; docs/REUSE_POLICY.md; CLAUDE.md
BRANCH / WORKTREE   mvp0/pr-fg-b in /Users/yuema137/mineworld-worktrees/impl-fg-b (sole writer)
BASE                origin/main @ 6306977 (#177), DOC-24 (#175) merged: ARC-87 present in docs/DECISIONS.md
HEAD / FINGERPRINT  see `git log -1` and `git status --short` (kept current at each milestone)
APPROVED SCOPE      design §2.1 as frozen; diff limited to AFGb-13's list
FROZEN INVARIANTS   §2.3 (I-1, I-2, I-3, I-5, I-10 … I-13, P4-2, FGb-1 … FGb-7); §5 as ruled in §14.1;
                    STORE_BOUND 1 048 576 B, TARGET 786 432 B, the 6 000-byte section ceiling; every platform
ENDPOINT AUTHORITY  implementation + local validation, semantic commits, branch push, PR create/update, CI
                    repair to review readiness: authorized (freeze of 2026-10-12, §12; the operator's go of
                    2026-10-13 relayed in this session's brief). Merge: never by this session; explicit
                    operator authorization only
SEQUENCE            C0 … C8 (§10)
VALIDATION BUDGET   unit, static, local integration (incl. one 300-day real run): unrestricted. Model calls:
                    none. CI: the PR's runs; no manual dispatch. The brief also asks for the sdk/python and
                    cognition suites and any affected Rust tests to run locally before pushing
STOP CONDITIONS     READY FOR OPERATOR REVIEW — DO NOT MERGE; material stops per §12; a milestone pause
                    announced by the operator: commit locally and stop
CURRENT CHECKPOINT  C0–C7 committed (C7 = 62b71cd, Gate 2 PASS); C8 in progress: README paragraph,
                    local 300-day real run (scratch /Users/yuema137/mineworld-worktrees/fgb-scratch,
                    outside the repository: save301/, alice.jsonl, run300.py), push, PR, CI
NEXT ACTIONS        Record the 300-day sizes in §15.2; commit C8; push mvp0/pr-fg-b; open the PR;
                    watch CI (fast, test, test-macos, test-windows, platforms, python ×3) on the exact
                    head; record per-leg times of test_retention_real.py / test_ac10.py; stop at READY
                    FOR OPERATOR REVIEW. Resumed once on 2026-10-14 after a session limit (coordinator)
```
