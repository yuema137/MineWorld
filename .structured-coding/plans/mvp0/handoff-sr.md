# Handoff — S6 SR: save snapshot retention and compression (`F-SAVE-1`)

A continuation aid, never a design authority. The authority is
[`pr-s6-save-retention.md`](pr-s6-save-retention.md) (DESIGN FROZEN 2026-10-10), with the ledger,
evidence, deviations and findings in its §14.

```text
PROJECT / PR        MVP-0 · S6 persistence follow-up / PR SR — snapshot retention and compression
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/pr-s6-save-retention.md (§12 contract, §14 ledger)
RELATED / BINDING   step-06-persistence.md; DECISIONS ARC-25, ARC-27, ARC-49, ARC-55, DEP-2, DEP-5;
                    ARCHITECTURE.md (persistence); ENGINEERING_STANDARDS.md; REUSE_POLICY.md;
                    overall.md (F-SAVE-1; decision table); CLAUDE.md
BRANCH / WORKTREE   mvp0/pr-s6-save-retention in /Users/yuema137/mineworld-worktrees/impl-save-retention
                    (sole writer)
BASE                origin/main @ a5f5357 (#145 merged: the design frozen)
APPROVED SCOPE      §2.1: retention rule (D-SR-2/3/7), zstd snapshot codec (D-SR-4/5), SAVE_FORMAT 3,
                    verify_from, inspect line, the 30-day CI size assertion, zstd (DEP-43), ARC-81
FROZEN INVARIANTS   §2.3 I-SR-1 … I-SR-5; D-SR-1 … D-SR-9; anchor 64·interval, newest two kept;
                    format 2 refused; no retention configuration; macOS, Linux, Windows
ENDPOINT AUTHORITY  source: §12 (primary session's freeze rulings, 2026-10-10)
  implementation + local validation   authorized
  semantic commits, branch push       authorized
  PR creation / update, CI repair     authorized
  merge                               never by this session; operator only
TOOL DISCIPLINE     cargo, git, gh (no merge), python3 scripts/*, target/*/mineworld, sqlite3 read-only
                    on scratch saves, mkdir -p, sed -n; never python3 -c, sed -i, curl, heredoc writes
VALIDATION BUDGET   unrestricted locally, including 30- and 300-day release runs; CI: the PR's runs
STOP CONDITIONS     NORMAL: SR READY FOR OPERATOR REVIEW — DO NOT MERGE. MATERIAL: kernel/contracts/
                    systems/packages change; a dependency other than zstd or zstd failing a CI leg;
                    D-SR-3's constants or ASR-1/2 bounds changed after measurement; ASR-7 failing;
                    any part of SR-b
POST-MERGE SYNC     primary session: overall.md (F-SAVE-1, F-SAVE-2, decision table);
                    this session: the PR ledger and step-06 L-6 note
```

## Current checkpoint — READY FOR OPERATOR REVIEW — DO NOT MERGE

Context CLOSED / AWAITING OPERATOR ACTION. All commits C0 … C6 on PR #153; evidence in the design's
§14; exact-head CI and AC-8 (scratch/sr-ac8-scenario) reported on the PR. Merge only with the
operator's explicit authorization after the primary session's review.

### Earlier checkpoint — C5 in progress. Commits: d573ea2 C0 · 5e202e8 C1 · ded6e73 C2 · 054bf5b C3 · 93d50a8 C4. Draft PR
#153. Scratch branch `scratch/sr-c1-build` (three-platform build of zstd: green; delete at close).
Uncommitted: kill_and_resume anchor/retiring kill points + control oracle (green);
market_town ASR-2 assertion (running: /tmp/impl-sr/c5-market.log).

Scratch on this machine: /tmp/impl-sr (base binary, base-tree detached worktree at a5f5357 with an
uncommitted fixture-maker test — remove with `git worktree remove --force` at close), saves under
/tmp/impl-sr-target/sr-saves (base-30 kept for comparison; delete at close).

## Next actions

1. market_town green → ASR-2 mutation (bound 1 MiB) red → commit C5 tests.
2. Release build of head; 30- and 300-day saved runs; inspect; dbstat; replay 300-day; verify_from
   (ignored test) on 30-day all anchors and 300-day first/middle/last; SHA-256 vs base; wall times;
   M1 on the real binary (ASR-1 red); level-9 mutation (ASR-3/4 still pass, mixed-level resume);
   delete-a-fact mutation (ASR-11 + replay red).
3. Push; PR ready; CI all jobs; `scratch/sr-…-scenario` push for AC-8 (ASR-7).
4. C6 close-out; delete scratch saves and branches.
