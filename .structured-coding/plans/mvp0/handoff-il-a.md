# Handoff — PR IL-a (S17: the configuration seam and the extension catalogs)

A continuation aid only. The authority is `step-18-interaction-list.md` §11 (design, contract §11.10,
ledger §11.11, deviations §11.12).

```text
PROJECT / PR         MVP-0 · S17 / PR IL-a — configuration seam (ARC-61) and extension catalogs (ARC-62)
PRIMARY DESIGN DOC   .structured-coding/plans/mvp0/step-18-interaction-list.md §11
RELATED / BINDING    overall.md "The World Interaction List"; step-18 §§4.3, 4.10, 5; step-16 §15 (E-b);
                     DECISIONS ARC-31, ARC-33, ARC-39, ARC-53; MODULE_SPEC §§3.1, 4, 4.1, 9; CLAUDE.md
BRANCH / WORKTREE    mvp0/pr-il-a-seam · /Users/yuema137/mineworld-worktrees/impl-il-a (one writer)
BASE                 main @ 0d35d6b (re-checked 2026-10-08: origin/main unchanged; E-b, 12d and
                     test-hygiene not merged)
FROZEN               DESIGN FROZEN (2026-10-08), primary session (f6489fd)
APPROVED SCOPE       §11.1 change set + the primary session's amendment (tests/acceptance/Cargo.toml
                     two dev-deps; Cargo.lock exactly two lines in mineworld-acceptance's list)
FROZEN INVARIANTS    §11.10: no diff under kernel/ contracts/ persistence/ server/ clients/ cognition/
                     worlds/; root Cargo.toml unchanged; IA-1 digests equal E-IA-0; ac1_composability,
                     precursor_vocabulary, seam_vocabulary unedited and passing; no resolution: shim
ENDPOINT AUTHORITY   implementation + local validation: authorized (primary session's freeze brief)
                     semantic commits, branch push: authorized (same)
                     PR creation/update, marked READY FOR OPERATOR REVIEW: authorized (same)
                     scratch canary branch: authorized, never pushed, deleted after (QIA-5, same)
                     merge: operator only
SEQUENCE             IA-C1 … IA-C8
VALIDATION BUDGET    §11.10 (300-day town runs ≤ 5 in all; one full gate on the final head)
STOP                 PR IL-a READY FOR OPERATOR REVIEW — DO NOT MERGE
```

## Current checkpoint

- IA-C1 … IA-C8 committed and pushed; origin/main e98321a (#77) merged (c6ca232). IA-1 … IA-10 PASS
  (§11.11 E-IA-0 … E-IA-8); IA-11, the full gate on the final code head, is E-IA-9.
- The canary branch is deleted and was never pushed. Town 300-day run budget used up (5 of 5).
- The session was cut off by an API rate limit during IA-C7 and resumed; the uncommitted IA-C7 files
  were kept and finished (D-15).

## Next actions

- Operator review of the PR (READY FOR OPERATOR REVIEW). Do not merge without explicit authorization.
- Done: E-b (#78), #81 and 16a (#79) merged in (be30002, fe40daf) with §11.5's lines placed as
  designed. IA-1 (2 of 4 town runs), IA-9 and the full gate were re-run, all PASS (E-IA-10).
- IL-b: let the framework-owned `packages` key through to `mineworld-packages`; give `classes` meaning.
