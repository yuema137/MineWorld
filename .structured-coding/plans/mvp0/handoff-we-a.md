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

## Current checkpoint (2026-10-14)

C0 … C4 committed locally, not pushed. STOPPED: E-2 met the stop condition "U-2 false" in its literal
text. A declaration change also moves Market Town's genesis journal entry and its snapshots'
composition, by exactly economy's added declaration (150 bytes). Facts, summaries and every other journal
entry are byte-identical (F-WE-a-9, §11 C4). The ruling is proposed there. 12d (#171) is still open,
so C5 waits as well.

## Next actions (after the operator's ruling and 12d's merge)

1. Record the F-WE-a-9 ruling in §16.1 and U-2's corrected wording.
2. Rebase onto origin/main with 12d. Re-run economy tests. C5: Market Town configuration, plus
   `happenings` in `tests/acceptance/tests/ac1_composability.rs` `GENERIC_PACKS` (found in C1). Then E-3,
   E-4 (long profile, baseline write, with a reason line covering C4's journal move), E-5.
3. C6: README, `systems/README.md` line, ARC-88 dated note (reconcile "owns Happenings on places" with
   F-WE-a-6), MODULE_SPEC §3.1 catalog list.
4. Run the full local `cargo test --workspace --no-fail-fast`, push, open the PR, and get one canonical
   CI run. Then READY FOR OPERATOR REVIEW. Do not merge.
