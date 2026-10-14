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
                       scripts/ci_parity.py (system python3 lacks tomllib)
SCRATCH:               /tmp/impl-we-a-scratch (mineworld-base binary, base.txt record)
APPROVED SCOPE:        §2.1 items 1–9; C4 economy price modifiers; C5 Market Town opt-in (after 12d #171 merges)
FROZEN INVARIANTS:     INV-WE-1 … INV-WE-8; no kernel/contracts/persistence/worldpack/server/client change; no LM;
                       no new third-party dependency
ENDPOINT AUTHORITY:    implementation, commits, push, PR create/update, CI repair: authorized (operator go
                       2026-10-13, relayed by the primary session); merge: explicit operator authorization only
SEQUENCE:              C0 → C1 → C2 → C3 → C4 → C5 → C6 → PR → one canonical CI
BUDGET:                unit/integration/static unrestricted; ci_parity default record ≤ 4 runs (1 used: base);
                       long profile once (E-4); local full workspace test before any push that changes tests
STOP CONDITIONS:       C5 before 12d merged; frozen invariant falsified; U-2 false; QWE ruling contradicted;
                       operator milestone pause → commit locally and stop
```

## Current checkpoint

C0 done (base recorded). Next: C1.

## Next actions

1. C1: crate `systems/happenings` (configuration, kinds, entries, genesis fact, almanac start), install lines,
   T1 + T12 (a)(c), E-1.
