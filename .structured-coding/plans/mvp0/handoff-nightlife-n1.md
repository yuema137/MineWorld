# Handoff — PR N1 (nightlife shop hours)

Continuation aid only; the authority is `pr-nightlife.md` (§5 N1 boxes + §11 ledger) and
`pr-nightlife-contract.md` (Contract N1).

```text
PROJECT / PR:     MineWorld operator-playable MVP / PR N1 — optional economy shop hours and Market Town nights
PRIMARY DESIGN:   .structured-coding/plans/mvp0/pr-nightlife.md (§§0–5, 7–9, 11)
BINDING DOCS:     CLAUDE.md; docs/{VISION,CORE_CONCEPTS,ARCHITECTURE,MODULE_SPEC,ENGINEERING_RULES,
                  ENGINEERING_STANDARDS,REUSE_POLICY}.md; operator-playable-mvp-20261123.md; step-35; ARC-28/61/112/118
BRANCH / TREE:    mvp0/pr-nightlife-n1 @ /Users/yuema137/mineworld-worktrees/impl-nightlife-n1 (one session)
BASE:             origin/main 8c971f0f
SCOPE:            N1-C1 … N1-C3; packaged launcher check INCONCLUSIVE — prerequisite P1
INVARIANTS:       N1-I1 … N1-I8; no business hours outside economy; absent hours = today; AC-S35-8 never relaxed
ENDPOINTS:        implement/commit/push/PR/CI repair authorized (operator freeze 2026-11-27 "同意三个都按推荐冻结");
                  merge: explicit operator authorization only
SEQUENCE:         C1 → C2 → C3 → READY FOR OPERATOR REVIEW
BUDGET:           owning-crate tests only locally (R-LOAD-1); heavy runs ≤ 60 min via cast-slots/with-slot.sh;
                  report before re-recording any world golden (primary dispatch); $0
STOP:             N1 READY FOR OPERATOR REVIEW — DO NOT MERGE
```

Checkpoint and next action: see the last row of `pr-nightlife.md` §11.
