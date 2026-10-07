# Handoff — PR 11a implementation context: ACTIVE

**Active PR:** Step 10 / PR 11a — installable System Packs (S9, first of six)
**Effort:** `mvp0`
**Primary design doc (semantic authority):** `.structured-coding/plans/mvp0/step-10-market.md` §4.1
(`DESIGN FROZEN (2026-10-07)`, freeze record in its header; ledger §4.1 checkboxes, evidence §9 `E-A*`)
**Execution contract:** step-10 §11, as confirmed by the freeze record and the kickoff below.
**Related / binding:** overall.md §§1, 2, 3 (S9), 7; MVP §§1–2, 9; MODULE_SPEC §§3, 4.1, 8;
PACKAGE_FORMAT §§6, 8; ARCHITECTURE §§12, 14; REUSE_POLICY §§11–12, 17; DECISIONS ARC-8, ARC-26,
ARC-28, ARC-31, DEP-10; step-09-social.md §10.1.

The previous PR's (10c) handoff is closed and replaced by this one; 10c merged as recorded in
step-09-social.md.

## Repository identity

```text
worktree         /Users/yuema137/mineworld-worktrees/s9-11a — this session's only writer
branch           mvp0/pr-11a-installable
base             main @ b53e19d (docs-only on top of b9e5937, the design's audit base)
current HEAD     see `git log -1`; this file is refreshed at each semantic milestone
```

## Approved scope

§1.1 PR 11a; SD-1 … SD-6; F-10's fixture; plus the two binding freeze conditions:
1. I-2 checked mechanically (a vocabulary scan over this PR's added lines, with a reasoned
   allow-list, shown to fail on a planted violation, fail-closed);
2. ARC-33 states the static-linking boundary (directory + two lines in `systems/installed` +
   rebuild; no-rebuild installation is ARC-8, outside MVP-0).

## Frozen invariants (bind 11a)

I-2 (precursors name no market concept), I-4 (social-cafe unchanged; E-0 sha
`ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b`), I-5 (controllers unchanged),
I-8 (`kernel/` untouched; `contracts/` untouched in 11a).

## Endpoint authority

```text
implementation + local validation   authorized — source: the operator-relayed kickoff of 2026-10-07
                                    ("implement C1–C5")
semantic commits, branch push       authorized — same source ("commit and push to
                                    mvp0/pr-11a-installable")
PR creation / update                authorized — same source ("open the PR against main and mark it
                                    READY FOR OPERATOR REVIEW")
scratch branch for the canary (C5)  authorized, never merged, never pushed (or deleted if pushed) —
                                    same source
CI repair                           N/A — no CI workflow (S13)
merge                               operator only; never inherited, never widened
```

## Sequence and budget

C1 → C2 → C3 → C4 → C5, each committed and pushed when coherent. Unit/integration/static
unrestricted; real-model NOT REQUIRED; real runs: the 300-day social-cafe comparison (~13 s) and the
canary install; about one hour in total.

## Checkpoint

- E-0 reproduced on b53e19d before any edit: 339 lines, sha-256 (all but `wall`) `ad49c723…c64b`,
  faults 0, fingerprint 59339a9c281829c9, wall 12.5 s. Transcript `/tmp/s9-11a/base-run.txt`
  (scratch; the hash is the evidence).
- C1 in progress.

## Next actions

1. C1: DEP-12, ARC-33, ARC-35 in DECISIONS; MODULE_SPEC §3.1; systems/README "Adding a pack".
2. C2 … C5 as §4.1.

## Stop conditions

- Normal: PR 11a READY FOR OPERATOR REVIEW — DO NOT MERGE.
- Material: step-10 §11 MATERIAL STOP; a precursor needing a market word (freeze condition 1).

## Post-merge synchronization

The planning session owns step/overall updates (including overall §1's AC-1 gloss citing ARC-35,
QS-2); this session owns §4.1 and §9 `E-A*`.
