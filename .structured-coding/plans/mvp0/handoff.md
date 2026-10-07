# Handoff — PR 10c implementation context: ACTIVE

**Active PR:** Step 09 / PR 10c — the content seam, names and routines (S8, third of three)
**Effort:** `mvp0`
**Primary design doc (semantic authority):** `.structured-coding/plans/mvp0/step-09-social.md` §4.3
(`DESIGN FROZEN` at `805bf4d`, freeze record §4.3.7; ledger §4.3.3, evidence §9 `E-C*`)
**Execution contract:** §11.2 (confirmed)
**Related / binding:** overall.md §§2, 3 (S8), 7; this file §§1.3, 10.1; MODULE_SPEC §4.1;
PACKAGE_FORMAT §8; DECISIONS ARC-23, ARC-25 … ARC-30, DEP-10

## Repository identity

```text
worktree         /Users/yuema137/mineworld-worktrees/s8-social — this session's only
branch           mvp0/pr-10c-social (pushed)
base             main @ 266daf7
```

## Scope, invariants, authority (summary; §11.2 governs)

- Scope: the generic content seam (`authoring/`), `systems/naming`, `systems/schedule`, the paced
  controller following its agenda, names in replies, biography and the Godot module.
- Invariants: §1.3 I-1 … I-9; I-1 means no edit to kernel/, contracts/, persistence/, server/,
  presence, movement, conversation, group-activity, relationships.
- Binding freeze condition (§4.3.7): every test relying on the 00:00–05:00 quiet window asserts
  first that no routine boundary falls in it, failing with "fixture assumption violated: …"; the
  routine files' comments state the dependency.
- Authority: implementation, commits, push, PR creation, run.sh — authorized. Merge: operator only.
- Sequence: C1 → C9. Budget: about one hour of validation wall time.

## Checkpoint

C1–C5 committed and pushed (923b438 specs, fc2fdd3 authoring+naming, 9f1d1c0 worldpack seam,
5aa7c69 names in replies/biography/Godot, 3ddafbf schedule). Full workspace 405 passed at C5.
Bounded deviation D-C1 (AC-2 naming oracle) recorded in §4.3.3 C4.

## Next actions

C6: register schedule (catalog), twelve routines in social-cafe with the quiet-window comment,
refusals for routines, literal 29 → 53, run.sh evidence. Then C7 (controller follows agenda; frozen
pre-C7 binary from the C6 commit for parity), C8 (routines.rs, I-4 clause, quiet-window assertions
in milestone_b/restart per §4.3.7, AC-2 schedule, biography set), C9 docs + gates + PR.

## Stop conditions

- Normal: PR 10c READY FOR OPERATOR REVIEW — DO NOT MERGE.
- Material: §11.2.
