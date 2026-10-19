# Handoff — PR WE-a (the `happenings` System Pack)

Continuation aid only; the authority is `pr-we-a-happenings.md` (§13 contract, §15 ledger).

```text
PROJECT / PR:          MineWorld step-24 PR WE-a — the `happenings` System Pack
PRIMARY DESIGN DOC:    .structured-coding/plans/mvp0/pr-we-a-happenings.md (DESIGN FROZEN 2026-10-12, revision 2)
BINDING DOCS:          mvp1/step-24-decision-and-memory.md §§3, 9, 10, 11, 14.1; overall.md; DECISIONS.md ARC-25,
                       ARC-28, ARC-33, ARC-61 … ARC-65, ARC-68, ARC-88; CORE_CONCEPTS; MODULE_SPEC §§3.1, 4.1;
                       ENGINEERING_RULES; ENGINEERING_STANDARDS; .structured-coding/standards.md
BRANCH / WORKTREE:     mvp0/pr-we-a at /Users/yuema137/mineworld-worktrees/impl-we-a (this session only)
BASE:                  origin/main @ 6306977 (contains DOC-24 #175, merge baa0d33)
ENV:                   PATH="$HOME/.cargo/bin:$PATH"; CARGO_TARGET_DIR=/tmp/impl-we-a-target; python3.12 for
                       scripts/ci_parity.py (system python3 lacks tomllib); rustc on PATH for record
SCRATCH:               /tmp/impl-we-a-scratch (binaries base/c1/c3/c4; records base/c1/c3/c4.txt; e2/ saves)
APPROVED SCOPE:        §2.1 items 1–9; C4 economy price modifiers; C5 Market Town opt-in (after 12d #171 merges)
FROZEN INVARIANTS:     INV-WE-1 … INV-WE-8; no kernel/contracts/persistence/worldpack/server/client change; no LM;
                       no new third-party dependency
ENDPOINT AUTHORITY:    implementation, commits, push, PR create/update, CI repair: authorized (operator go
                       2026-10-13, relayed by the primary session); merge: explicit operator authorization only
SEQUENCE:              C0 → C1 → C2 → C3 → C4 → C5 → C6 → PR → one canonical CI
BUDGET:                default ci_parity records 4 of 4 USED; long profile once (E-4, C5); local full
                       workspace test before any push that changes tests (not yet run)
STOP CONDITIONS:       C5 before 12d merged; frozen invariant falsified; U-2 false; QWE ruling contradicted
```

## Current checkpoint (2026-10-15)

C0 … C6 done. 12d (#171, unmerged, `4ff4856`) and main `5cbb469` were merged in on the operator's
instruction (maximum parallelism), then C5 + C6 were committed as one commit and pushed. PR #189 is
marked ready, with base `main`; it shows 12d's commits until 12d merges. Evidence is in the design's §11
C5/C6 and §15. Scratch: `/tmp/impl-we-a-scratch/c5/` (run30, run300, save300, replay300, long.txt).
Lifecycle: READY FOR OPERATOR REVIEW — DO NOT MERGE. CI on the final head is tracked on the PR.

## Requirements left for other PRs

- **S31-a** must not repeat the AC-1 change (Q-S31a-1 (a): `configurable` in `ac1_composability.rs`
  and the `ARC-35` note of 2026-10-15), nor the `market_composition.rs` `without()` change (F-S31a-7).
  It extends WE-a's `EconomyConfiguration` and Market Town's `configure/economy.yaml` (R-WEa-a-1).
- **12d** merging first changes nothing here. If 12d changes again before merging, WE-a re-merges it
  and re-checks Market Town's baseline section.
- **F-WE-a-10** (presentation): a 2D/3D HUD reader for `happenings-here` / `happenings-announced`.
