# Handoff — PR S28-a (the memory home: schema 4, shared fact records, seat views, positional dumps, GC, transparency harness)

A continuation aid only. The authority is `pr-s28a-memory-home.md` (design, contract §12, ledger §15).

```text
PROJECT / PR         MVP-0 · S28 / PR S28-a — the memory home (schema 4, seat views, positional dumps, GC,
                     transparency harness)
PRIMARY DESIGN DOC   .structured-coding/plans/mvp0/pr-s28a-memory-home.md
BRANCH / WORKTREE    mvp0/pr-s28a · /Users/yuema137/mineworld-worktrees/impl-s28a (one writer: this session)
BASE                 origin/main @ b37733b9; origin/main merged at 09ea595c
FROZEN               DESIGN FROZEN 2026-10-15, revision 1
ENDPOINT AUTHORITY   implementation, commits, push, PR, CI repair: authorized. Merge: never by this session
VALIDATION           R-LOAD-1: one test file at a time locally; heavy runs under cast-slots/with-slot.sh
BINARY               export PATH="$HOME/.cargo/bin:$PATH"; cargo build -p mineworld-cli (worktree target/)
STOP                 S28-a READY FOR OPERATOR REVIEW — DO NOT MERGE
```

## Current checkpoint

C0 … C9 committed (`817d202b`, `e2e5dab9`, `1cc4ce33`, `402426b5`, `7ec76cfd`, `419df400`, `e7b35625`,
`24c5ca07`, `d5b745b7`, merge `09ea595c`, C9). Every acceptance criterion PASS locally (§15 table). Open for the
operator: F-S28a-9 (I-a-6 (b) and subject-only `fact_who` rows; a proposed amendment). Recorded slips: D-a-3.

## Next actions

1. Push; open the PR; CI on the exact head (python × 3 OS carries the new tests, `HomeBusy` on Linux and Windows).
2. Repair any failure this change causes; record the CI result per job in §15; READY FOR OPERATOR REVIEW.
3. After merge (primary): §11's edits to `step-28-shared-memory.md` and `overall.md`; S28-b inherits the ported host.
