# Handoff — S10 P6a: the `persona` System Pack

A continuation aid, never a design authority. The authority is
[`pr-s10-p6-lm-controller.md`](pr-s10-p6-lm-controller.md) (DESIGN FROZEN 2026-10-14, revision 2),
its execution contract §12.1, and its live ledger §15 (P6a rows).

```text
PROJECT / PR        MVP-0 · S10 (step-17) / PR P6a — the `persona` System Pack (ARC-74); P6b is separate
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/pr-s10-p6-lm-controller.md (§§2.1 items 1–3, 5.14, 6, 8
                    AP6a-*, 9.0, 9 P6a, 14.1, 15)
RELATED / BINDING   step-17-cognition.md §3.8.5, QS10-7; docs/MODULE_SPEC.md §4.1; docs/DECISIONS.md
                    ARC-28, ARC-31, ARC-33; systems/naming (the pattern); scripts/baselines.txt and
                    .structured-coding/standards.md; CLAUDE.md; docs/ENGINEERING_STANDARDS.md;
                    docs/ENGINEERING_RULES.md
BRANCH / WORKTREE   mvp0/pr-s10-p6a in /Users/yuema137/mineworld-worktrees/impl-s10-p6a (sole writer)
BASE                origin/main @ adf5d2c (Merge PR #184, docs/p6-design)
HEAD / FINGERPRINT  see `git log -1` and `git status --short` (kept current at each milestone)
APPROVED SCOPE      §12.1 APPROVED SCOPE: systems/persona; one systems/installed line (Cargo.toml and
                    lib.rs); social-cafe world.yaml plus nine people files; docs/MODULE_SPEC.md §4.1;
                    docs/DECISIONS.md ARC-74; scripts/baselines.txt only under QP6-2
FROZEN INVARIANTS   I-P6-1 as it applies to P6a (no change under kernel/, contracts/, server/,
                    persistence/, PROTOCOL.md, or any System Pack other than systems/persona);
                    persona discloses to its holder only; persona grants nothing (R-S10-9);
                    no new external dependency (I-P6-9)
RULINGS (verbatim, §14.1)
                    QP6-1 [primary]: "Split. P6a (the `persona` pack) now; P6b (the controller) after DL-a
                    and FG-b merge" (operator).
                    QP6-2 [operator]: "Yes, narrowly. P6a may re-baseline, with reason lines, **only** the
                    columns that move because people now perceive their own persona" (operator).
ENDPOINT AUTHORITY  implementation + local validation: authorized (primary, 2026-10-14)
                    semantic commits: authorized (primary; working rules §14)
                    branch push: authorized (primary; working rules §21)
                    PR creation / update: authorized (primary; working rules §21)
                    CI repair to review readiness: authorized (primary; working rules §21)
                    merge: never by this implementation session; operator only (working rules §22)
SEQUENCE            A0 (this commit: ledger and handoff), A1 (docs), A2 (systems/persona), A3 (install +
                    social-cafe personas; baseline check)
VALIDATION BUDGET   unit and integration tests unrestricted locally; one long parity record in A3
                    (≤ 20 min); CI: the PR's runs. Gate 1: not required (no LLM-facing change)
STOP CONDITIONS     any moved baseline column other than a social-cafe column that moves because people
                    perceive their own persona (A3 step 7); any change to an existing System's code; any
                    change outside APPROVED SCOPE; a new external dependency
CURRENT CHECKPOINT  A0 committed (7dcb1ac); A1 committed (9fd8315: MODULE_SPEC §4.1 persona row; ARC-74).
                    A2 committed (4561074): systems/persona built, tests green, M-P6a-1 seen red and
                    reverted. A3 committed locally with baselines pending (main re-record); STOPPED at
                    A3 step 6: baseline check FAIL, 20 of 20 columns moved, main drift (F-P6a-2), NOT
                    PUSHED. Findings: F-P6a-1 (struct-level bounds lose line/column; per-value bounds
                    keep them); F-P6a-2 (main drift); F-P6a-3 (ci_parity.py needs Python 3.11+);
                    D-P6a-1 … D-P6a-4 in §15.
NEXT ACTIONS        WAIT (coordinator ruling): main's baselines are re-recorded in a separate PR after #171
                    (12d) merges. Then, on resume: merge origin/main into mvp0/pr-s10-p6a; re-record the
                    long profile with /opt/homebrew/bin/python3.11 (F-P6a-3); baseline check. Only
                    social-cafe columns that move because people perceive their persona may change, with
                    reason lines citing QP6-2 (F-P6a-2); any other moved column is a material stop. Then
                    run the full workspace tests, fmt, clippy; push; open the PR; CI on the exact head.
                    Never merge.
POST-MERGE SYNC     primary session: step-17, overall.md. This session: the P6a rows of the §15 ledger.
READY STATE         P6a STOPPED at A3 (baseline gate open: main drift, F-P6a-2). NOT PUSHED. NOT READY.
                    DO NOT MERGE.
