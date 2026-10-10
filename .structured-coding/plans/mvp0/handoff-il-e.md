# Handoff — PR IL-e (S17: the World Interaction List for the social packs)

A continuation aid only. The authority is `pr-il-e-social.md` (design §§1–11, contract §12, ledger §14,
deviations §15).

```text
PROJECT / PR         MVP-0 · S17 / PR IL-e — sections of the World's Interaction List for conversation,
                     group-activity and relationships
PRIMARY DESIGN DOC   .structured-coding/plans/mvp0/pr-il-e-social.md
RELATED / BINDING    step-18-interaction-list.md §§2.2, 4, 5, 6.2, 12, 12.15; overall.md "The World
                     Interaction List"; step-12-server.md QS11C-6; DECISIONS ARC-5, 23, 25, 29, 34, 43, 61,
                     63, 64, 65, DEP-28; MODULE_SPEC §4.2; CLAUDE.md §§2–4
BRANCH / WORKTREE    mvp0/pr-il-e · /Users/yuema137/mineworld-worktrees/impl-il-e (one writer)
BASE                 origin/main @ c9832d3 (#147, the freeze merge)
FROZEN               DESIGN FROZEN 2026-10-10 (primary session; operator accepted QIE-1 … QIE-12)
APPROVED SCOPE       §1, §4, §5 as answered by §13; IE-C1 … IE-C7
FROZEN INVARIANTS    IL-I1 … IL-I11; §11's no-diff list (kernel, contracts, persistence, server, clients,
                     worlds, sdk, authoring, worldpack/src, tools/cli/src, cognition, presence, other
                     packs, root Cargo.toml/Cargo.lock, tests/acceptance); IE-1 equal to E-IE-0; every
                     compiled default equals today's constant; a list cannot grant
ENDPOINT AUTHORITY   implementation + local validation: authorized (freeze §12; primary session's kickoff)
                     semantic commits, branch push: authorized (same)
                     PR creation/update, marked READY FOR OPERATOR REVIEW: authorized (same)
                     CI repair to review readiness: authorized (same)
                     merge: operator only, with a merge commit
SEQUENCE             E-IE-0 before IE-C2; IE-C1 → IE-C7 (C2/C3/C4 any order)
VALIDATION BUDGET    four 300-day town runs (2 used for E-IE-0); 30-day runs uncounted; bodies-yard
                     30-day twice; one full workspace gate on the final head; real model not required
STOP                 PR IL-e READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP        an edit in the no-diff list; a digest change with default content not explained by
                     main moving; a schema change for a social pack; a ruling other than the freeze's
```

## Current checkpoint

- E-IE-0 captured on c9832d3 (see the ledger). IE-C1 (specs) in progress.

## Next actions

- IE-C2 conversation, IE-C3 group-activity, IE-C4 relationships; then IE-C5 worldpack tests, IE-C6 CLI
  tests, IE-C7 close.
