# Handoff — SR-b (step-24 FG-d): sealed-block compression of the event log

Continuation aid only. The authority is the PR design and its ledger
([`pr-sr-b-log-compression.md`](pr-sr-b-log-compression.md) §14). A line here that the design does not
support is reported, not obeyed.

```text
PROJECT / PR:          MineWorld mvp0, step-24 FG-d = SR-b — sealed-block compression of the world event
                       log (SAVE_FORMAT 4, streaming readers, format 3 refused). PR number: assigned on open
PRIMARY DESIGN DOC:    .structured-coding/plans/mvp0/pr-sr-b-log-compression.md (DESIGN FROZEN 2026-10-12,
                       revision 2)
RELATED / BINDING:     pr-s6-save-retention.md (ARC-81); mvp1/step-24-decision-and-memory.md (§2.5 F-S24-1,
                       §8.1, §10, §11 FG-d); step-06-persistence.md; docs/DECISIONS.md ARC-25, ARC-27,
                       ARC-49, ARC-55, ARC-81, ARC-89, DEP-2, DEP-5, DEP-43; docs/ARCHITECTURE.md;
                       docs/ENGINEERING_STANDARDS.md; docs/REUSE_POLICY.md; overall.md; CLAUDE.md
WORKTREE / BRANCH:     /Users/yuema137/mineworld-worktrees/impl-sr-b, mvp0/pr-sr-b (held by this session only)
IMPLEMENTATION BASE:   origin/main @ 6306977 (Merge pull request #177, docs/dl-design), fetched 2026-10-13
CURRENT HEAD:          see `git log -1`; the ledger (§14.2) names each commit
WORKING-TREE PRINT:    `git status --short` empty at each commit boundary
APPROVED SCOPE:        design §2.1, as frozen
FROZEN INVARIANTS:     §2.3 I-SRB-1 … I-SRB-6; D-SRB-1 … D-SRB-11; anchor lattice (QSRB-1); BLOCK_LEVEL 9
                       (QSRB-2); format 3 refused (QSRB-3); bounds 30-day ≤ 48 MiB, 300-day ≤ 96 MiB bodies /
                       ≤ 80 MiB walking, growth ≤ 0.25 MiB/day (nightly 0.5), reader peak ≤ 256 MiB, seal ≤
                       100 ms per block (QSRB-4); baselines re-recorded once with a reason line (QSRB-5); no
                       change to the server's PerceivedHistory seam (QSRB-6); ARC-89 (QSRB-8)
ENDPOINT AUTHORITY:    implementation + local validation: authorized (operator's go 2026-10-13, relayed by
                         the coordinator; freeze rulings 2026-10-12)
                       semantic commits: authorized (working rules §14 default, after the go)
                       branch push: authorized (working rules §21 default, after the go)
                       PR creation / update: authorized (working rules §21 default, after the go)
                       CI repair to review readiness: authorized (working rules §21 default)
                       merge: NEVER by this session (freeze rulings "Do not merge"; working rules §22)
COORDINATOR ADDITIONS: full local `cargo test --workspace --no-fail-fast` before any push that changes
                       tests; a `-scenario` scratch branch for AC-8 parity; CI fast + test + test-macos +
                       test-windows + platforms + python green on the exact head; a milestone pause → commit
                       locally and stop (coordinator brief, 2026-10-13)
SEQUENCE:              C0 … C7 (design §10)
VALIDATION BUDGET:     unit, static, local integration incl. 30/100/300-day release runs and one 12d scratch
                       merge: unrestricted; CI: the PR's runs, no manual dispatch
STOP CONDITION:        READY FOR OPERATOR REVIEW — DO NOT MERGE
ENV:                   export PATH="$HOME/.cargo/bin:$PATH"; export CARGO_TARGET_DIR=/tmp/impl-sr-b-target
PARALLEL LANES:        12d (#171, unmerged; QSRB-7: second to merge re-runs market_town.rs and re-records
                       baselines), DL-a, FG-a, FG-b, WE-a
```

## Current checkpoint

C0 — freeze verified, base recorded, handoff initialized. The format-3 fixture
(`persistence/tests/fixtures/format-3/world.sqlite`) was written by the base build before any code edit
(ledger §14.2).

## Exact next actions

1. C1: ARC-89 completed (D-SRB list, level, revisit triggers), notes under ARC-25 and ARC-81,
   ARCHITECTURE persistence section, MODULE_SPEC §8.1 `log` line, persistence README, step-06 L-6.
2. C2: `persistence/src/block.rs` + `sealing` in `world.rs`.
