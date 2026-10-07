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

Starting C1 (specs: ARC-31, ARC-32, PACKAGE_FORMAT §8, MODULE_SPEC §4.1/§8.1).

## Next actions

C1 → commit/push → C2 (authoring + naming).

## Stop conditions

- Normal: PR 10c READY FOR OPERATOR REVIEW — DO NOT MERGE.
- Material: §11.2.
