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

## Current checkpoint

C0 — implementation context opened; base release binary being built to /tmp/impl-sr/base-mineworld
for the ASR-8 fixture and the ASR-11 base hashes.

## Next actions

1. C1: DECISIONS ARC-81, DEP-43, ARC-25 note; ARCHITECTURE persistence; persistence README; step-06
   L-6; zstd in Cargo.toml; cargo deny; push for the three-platform build.
2. C2 … C6 per §10.
