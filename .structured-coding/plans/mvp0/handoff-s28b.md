# Handoff — PR S28-b (retention on seat views: attribution, FU-S28-1, C-4, the bound at the bound, Gate 2)

A continuation aid only. The authority is `pr-s28b-retention-on-seat-views.md` (design, contract §12, ledger §15).

```text
PROJECT / PR         MVP-0 · S28 / PR S28-b — retention on seat views
PRIMARY DESIGN DOC   .structured-coding/plans/mvp0/pr-s28b-retention-on-seat-views.md
RELATED / BINDING    step-28-shared-memory.md; pr-s28a-memory-home.md; pr-fg-b*, pr-fg-c; MODULE_SPEC §5.2;
                     DECISIONS ARC-87, ARC-109; CLAUDE.md; ENGINEERING_STANDARDS/RULES
BRANCH / WORKTREE    mvp0/pr-s28b · /Users/yuema137/mineworld-worktrees/impl-s28b (one writer: this session)
BASE                 origin/main @ b85f2804 (S28-a merged 9884c64c)
FROZEN               DESIGN FROZEN 2026-10-18 (QS28b-1, QS28b-2 yes, as recommended)
SCOPE                §2.1 items 1–5
INVARIANTS           I-b-1 … I-b-9; I-S28-*; I-a-*; FGb/FGb2/FGc; bound numbers unchanged
ENDPOINT AUTHORITY   implementation, commits, push, PR, CI repair: authorized (operator freeze 2026-10-18;
                     working rules §§14, 21). Merge: never by this session
VALIDATION           R-LOAD-1: one test file at a time; heavy files under cast-slots/with-slot.sh; CI is the gate
COORDINATION         #249 (P6c+FG-d) and #262 (home marker) touch host.py / home.py / schema.py: the one that
                     merges second adapts; parity golden is re-captured if #249 changes consolidate first
STOP                 S28-b READY FOR OPERATOR REVIEW — DO NOT MERGE
```

## Current checkpoint

C0 committed (base, re-anchor, frozen parity evidence). Next: C1 (docs), C2 (schema 5 + attribution).

## Next actions

1. C1 commit (MODULE_SPEC §5.2 rules 6, 9, `lm-summarise` row, Model calls; DECISIONS notes; FG-c pointer).
2. C2: drafts of `forgotten.py` and `test_retention_forgotten.py` are in the session scratchpad; apply, plumb
   `Forgetter`/`PassScope` through plan, bound, host; schema 5; re-pin the replay `DIGEST` after zero misses.
