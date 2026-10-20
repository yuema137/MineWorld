# Handoff — PR S28-c (memory scopes)

A continuation aid only. The authority is `pr-s28c-memory-scopes.md` (design, contract §12, ledger §15).

```text
PROJECT / PR         MVP-0 · S28 / PR S28-c — memory scopes: interface, registry, person and world, the
                     stack in decisions.yaml, parameters, errors, the ceiling, the generated reference
PRIMARY DESIGN DOC   .structured-coding/plans/mvp0/pr-s28c-memory-scopes.md
BRANCH / WORKTREE    mvp0/pr-s28c · /Users/yuema137/mineworld-worktrees/impl-s28c (one writer: this session)
BASE                 stacked on mvp0/pr-s28b @ 2c13d9c9 (D-c-0); the lead rebases after S28-b merges
FROZEN               DESIGN FROZEN 2026-10-18 (every question as recommended)
VALIDATION           operator SPEED FIRST: quick local checks, one test file at a time; mutations and
                     independent review deferred to milestone
ENDPOINT             push mvp0/pr-s28c; the lead opens the PR and sequences CI. Never merge.
```

## Current checkpoint

C0 … C9 committed and pushed. `origin/main` (S28-b #340) is merged. The commit ledger is in §15 of the design.

## Next actions

1. The lead opens the PR and runs CI on its head. Watch the CLI tests `reference` and `decisions`, and the
   real-binary selection case.
2. `decisions.rs` `the_decisions_document_is_the_golden` fails on the base: a duplicate `configure:` key since
   TW-e. Fix it in whichever PR owns that test.
3. At the milestone: mutations M-c-1 … M-c-13, the independent review, the harness and Gate 2.
