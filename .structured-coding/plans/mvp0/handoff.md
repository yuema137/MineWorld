# Handoff — PR 08 implementation context: ACTIVE

**Active PR:** Step 07 / PR 08 — First real systems: places and movement (S6)
**Effort:** `mvp0`
**Primary design doc (semantic authority):** `.structured-coding/plans/mvp0/step-07-movement.md`
(combined step/PR document, `DESIGN FROZEN` at `4d6e42e`, answers in §10.1, frozen-answer amendments
at the head of §4)
**Execution contract:** §11 of the primary design doc
**Binding parents:** `overall.md` §§2, 3 (S6), 7; `docs/CORE_CONCEPTS.md` §§2, 6, 10–13, 15;
`docs/ENGINEERING_RULES.md` §§4–12, 19; `docs/MVP.md` §9; `docs/NETWORKING.md` §4;
`docs/DECISIONS.md` `ARC-15`, `ARC-23`, `ARC-25`, `ARC-26`; `docs/ENGINEERING_STANDARDS.md`; `CLAUDE.md`

The previous content of this file described PR 07 (closed, merged as `41d4ab1`) and is replaced.

## Repository identity

```text
worktree         /Users/yuema137/mineworld-worktrees/s6-movement — this session's only; the sibling
                 worktrees vis-character and vis-environment belong to other agents
branch           mvp0/pr-08-movement
base             main @ a594164 — 311 tests green there (E-0)
current HEAD     `git log --oneline` is authoritative; pushed after every commit
```

## Scope, invariants, sequence

```text
scope        design §1.1 as answered in §10.1 (street in social-cafe, pack `passages`, reporting rule)
invariants   design §1.3 I-1 … I-8
sequence     C0 ✓ → C1 → C2 → C2b → C3 → C4 → C4b → C5 → C6 → C7
budget       unit/integration/static unrestricted; real model NOT REQUIRED; Godot run bounded by run.sh
```

## Endpoint authority (design §11)

```text
implementation, commits, push, PR creation   authorized (brief; overall §7; D-12)
merge                                        operator only — DO NOT MERGE
clients/protocol/run.sh                      allowed without prompt (§10.1 Q8)
```

## Current checkpoint and next actions

Checkpoint: C0, C1 (`507c041`), C2 (`47874ca`, gates E-2: 313 passed) done.
Next: C2b — `mineworld_presence::admit`, `arrival(world, ..)` checked, `react` refuses with
`FactRefusedByOwner`; then C3 → C4 → C4b → C5 → C6 → C7.
Note: earlier sessions stalled on connection problems; commit and push after each small step.

## Stop conditions

Normal: PR 08 READY FOR OPERATOR REVIEW, PR open, not merged. Material: any change to §1.3, to a public
contract beyond §1.1 as answered, to ownership, or to scope — stop and report with evidence.
