# Handoff — PR S28-a (the memory home: schema 4, shared fact records, seat views, positional dumps, GC, transparency harness)

A continuation aid only. The authority is `pr-s28a-memory-home.md` (design, contract §12, ledger §15).

```text
PROJECT / PR         MVP-0 · S28 / PR S28-a — the memory home (schema 4, seat views, positional dumps, GC,
                     transparency harness)
PRIMARY DESIGN DOC   .structured-coding/plans/mvp0/pr-s28a-memory-home.md
RELATED / BINDING    step-28-shared-memory.md (§§3, 4, 6, 7, 8, 9, 10, 11, 14.1, 15); pr-s10-p4-memory.md;
                     pr-fg-b-retention-host.md; pr-fg-b2-size-trigger.md; pr-fg-c-lm-summarise.md;
                     docs/CORE_CONCEPTS.md §5.3; docs/ARCHITECTURE.md §9.3; docs/DECISIONS.md ARC-59, ARC-87,
                     ARC-98 … ARC-101, DEP-37, DEP-58; .structured-coding/standards.md; CLAUDE.md;
                     ENGINEERING_STANDARDS; ENGINEERING_RULES
BRANCH / WORKTREE    mvp0/pr-s28a · /Users/yuema137/mineworld-worktrees/impl-s28a (one writer: this session;
                     the design's §9.0/§12 name `impl-s28-a`, the dispatch named `impl-s28a`: the dispatch's path)
BASE                 origin/main @ b37733b9 (#227, S28-0 merged; FG-b2 #213 merged; P6b #207 merged)
FROZEN               DESIGN FROZEN 2026-10-15, revision 1 (operator approval in the primary session)
APPROVED SCOPE       §2.1 items 1–8, as §14's rulings shape them; plus R-P6b-a-1 (P6b merged first, so this PR
                     adapts controller/ to the home, §15 D-a-1)
FROZEN INVARIANTS    I-S28-1 … I-S28-10 (I-S28-6 replaced by I-a-6 (a)/(b)); §6 I-a-1 … I-a-11; FGb-1 … FGb-7 and
                     FG-b2's bound numbers unchanged; §8 as written
ENDPOINT AUTHORITY   implementation + local validation, semantic commits, push, PR creation/update, CI repair:
                     authorized. Merge: never by this session
SEQUENCE             C0 … C9 (§9)
VALIDATION           R-LOAD-1: only the commit's test files, one at a time, plus ruff, ruff format, pyright;
                     heavy runs (50 kills, 12 seats × 2 days) one at a time under
                     /Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh, load average recorded
BINARY               export PATH="$HOME/.cargo/bin:$PATH"; cargo build -p mineworld-cli (worktree target/)
STOP                 S28-a READY FOR OPERATOR REVIEW — DO NOT MERGE
```

## Current checkpoint

C0 `817d202b`, C1 `e2e5dab9`, C2 (the home: schema, encoding, open/lock/refusal, `scopes.world`, `ARC-109`,
`DEP-61`). Findings open for the operator: F-S28a-9 (I-a-6 (b) and subject-only `fact_who` rows). Mutation tool:
`/tmp/s28a-scratch/mut.sh FILE OLD NEW` (exact snippet; restore from a byte copy, `cmp`).

## Next actions

1. C3: `memory/home/view.py` (`SeatView`, meter, hold/release/GC), `home.py` (`seat`, `view_of`, `remove_seat`,
   verify), `tests/{test_home_store.py, test_home_gc.py}`; M-a-2, M-a-3, M-a-4.
2. C4 … C9 as §9 lists them; controller/ joins the port (D-a-1).
