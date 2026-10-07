# Handoff — PR 11b implementation context: ACTIVE

**Active PR:** Step 10 / PR 11b — items and organizations as World Pack content kinds (S9, second of
six; a precursor)
**Effort:** `mvp0`
**Primary design doc (semantic authority):** `.structured-coding/plans/mvp0/step-10-market.md` §4.2
(4.2.1–4.2.4), evidence §9.2 (`E-B*`); the freeze record §12.0 binds and overrides other text.
**Related / binding:** step-10 §§1.3, 2.4, 2.5, 8.4, 10 (QS-5, QS-6, QS-15 … QS-19 as answered), 12,
13; overall.md §§1, 7; MODULE_SPEC §§4, 4.1; PACKAGE_FORMAT §8; CORE_CONCEPTS §§7, 8; DECISIONS
ARC-15, ARC-31, ARC-33, ARC-35, ARC-36, DEP-10.
**Execution contract:** step-10 §13, confirmed by §12.0 and the primary session's kickoff.

## Repository identity

```text
worktree         /Users/yuema137/mineworld-worktrees/s9-11b — this session's only
branch           mvp0/pr-11b-content-kinds
base             main @ da31613 (da316138…, the frozen planning merge)
parallel         PR 11c in /Users/yuema137/mineworld-worktrees/s9-11c — never touched from here
```

## Approved scope and invariants

- Scope: §4.2 B-C1 … B-C6. Nothing in contracts/, kernel/, persistence/, server/, clients/,
  cognition/, systems/.
- I-2: only `item`/`items` admitted (§4.2.2); any other market word is a material stop.
- I-4: the 300-day seed-7 social-cafe sha stays
  ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b.
- I-8: no contracts/ or kernel/ diff.
- §12.0: merged one at a time; if 11c merges first, rebase onto the new main (never merge main in),
  move the 11b scan row's base to it, re-run sha, planted violations and full gate; force-push only
  then. Merge with a merge commit (say so in the PR body).

## Endpoint authority

```text
implementation + local validation   authorized — primary session's kickoff (2026-10-07)
semantic commits, branch push       authorized — kickoff
PR creation / update                authorized — kickoff (open against main, READY FOR OPERATOR REVIEW)
rebase onto new main + force-push   authorized only after 11c merges first — §12.0, kickoff
CI repair                           N/A — no CI workflow (S13)
merge                               operator only; never inherited
```

## Sequence and budget

B-C1 → B-C2 → B-C3 → B-C4 → B-C5 → B-C6, each committed and pushed. Unit/integration unrestricted;
300-day run (~15 s) and 10-day CLI runs; one full workspace gate on the final head in background.

## Checkpoint

B-C1 in progress (docs).

## Next actions

B-C1 doc checks, commit, push; then B-C2 (scan word-level admission + 11b row).

## Stop conditions

- Normal: PR 11b READY FOR OPERATOR REVIEW — DO NOT MERGE.
- Material: step-10 §13 MATERIAL STOP.
