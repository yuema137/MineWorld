# Handoff — PR 10a implementation context: ACTIVE

**Active PR:** Step 09 / PR 10a — the town (S8, first of three)
**Effort:** `mvp0`
**Primary design doc (semantic authority):** `.structured-coding/plans/mvp0/step-09-social.md`
(step document with PR 10a's full design in §4.1; `DESIGN FROZEN` at `700f0e0`, answers in §10.1)
**Execution contract:** §11 of the primary design doc
**Binding parents:** `overall.md` §§2, 3 (S8), 7; `docs/MVP.md` §§3, 6, 9; `docs/CORE_CONCEPTS.md` §6.1;
`docs/DECISIONS.md` `ARC-23`, `ARC-26`, `ARC-27`; step-08 §§1.3, 10.1; `CLAUDE.md`

The previous content of this file described PR 09 (merged as `4f4cb1d`) and is replaced.

## Repository identity

```text
worktree         /Users/yuema137/mineworld-worktrees/s8-social — this session's only; vis-character and
                 vis-environment belong to other agents (vis-environment is read for the café layout,
                 never written)
branch           mvp0/pr-10-social (pushed; tracks origin)
base             main @ f4301c1 — 350 tests per step-08 E-final
current HEAD     `git log --oneline` is authoritative; pushed after every commit
```

## Scope, invariants, sequence

```text
scope        design §1.1 "PR 10a" as answered in §10.1: café re-authored to the slice's layout, the MVP
             town (6 places, 12 people, 11 seats), seeded door choice in PacedRuleController, existing
             test literals under I-5, demo.gd room drawing, Godot evidence re-recorded
invariants   design §1.3 I-1 … I-9 (I-5: every edited test keeps its claim; listed in §9)
sequence     C0 → C1 → C2 → C3 → C4 → C5
budget       unit/integration/static unrestricted; real model NOT REQUIRED; long runs and kill tests a
             few minutes in total; pace rises before days fall (Q4)
```

## Endpoint authority (design §11)

```text
implementation, commits, push, PR creation   authorized for 10a (coordinator's freeze message; brief)
clients/protocol/run.sh                      authorized (§10.1 Q11)
10b / 10c                                     NOT authorized until the coordinator says 10a is merged
merge                                        operator only — DO NOT MERGE
```

## Current checkpoint and next actions

Checkpoint: C1 (7eb4462), C2 (aacfa16), C3 (d61fb29) committed and pushed. C4's work was folded into
C1 and C2 (bounded deviation: ac13 replays recorded frames). Evidence is in design §9, E-1 to E-3.
Discoveries so far:
- the pre-existing `toward` overshoot defect, fixed in C3;
- pace raised to 900 s under Q4;
- two procedural slips (`sed -i`, a no-op `awk`), recorded in E-1 and E-3 and reported.
Next: C5. Update `worlds/social-cafe/README.md` and `docs/MVP_STATUS.md` (lines 19 and 35 are stale:
"two places", "four people, three seats"). Then run the full gates once on the final executable head,
open the PR with `gh pr create` without merging, and write the closeout §12.

## Stop conditions

Normal: PR 10a READY FOR OPERATOR REVIEW, PR open, not merged. Material: any change to §1.3, to a
public contract beyond §1.1, to ownership, or to scope; any existing test whose claim cannot be kept
under I-5 — stop and report with evidence.
