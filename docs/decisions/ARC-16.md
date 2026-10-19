## ARC-16 — A decision identifier names one decision, and a check enforces it

**Date** 2026-09-27 · **Source** two collisions found in one session · **Relates to**
`CLAUDE.md` §2.1(4), `ARC-12`

`docs/DECISIONS.md` is the authority that every specification cites *by identifier*. An identifier
naming two decisions makes each citation of it unresolvable, which is exactly the code/specification
contradiction `CLAUDE.md` §2.1(4) forbids leaving unrecorded. Two such collisions existed at once:

- **`ARC-10` was allocated twice on `main`** — "Generation is development tooling" (12:14) and
  "A world's initial state is a recorded genesis fact" (16:26), four hours apart, both on `main`,
  neither noticed. Two documents cited the second and four cited the first.
- **`ARC-13` was allocated twice across branches** — the 3D lighting reversal at 19:13 and the
  default 2D style at 19:28, on two branches from one `main`.

**The mechanism is what makes this worth a decision rather than a fix.** The two entries land in
different regions of a long file, so `git merge` reports no conflict and `main` simply gains a
duplicate. Reading `main` before allocating is not a defence: the 3D branch merged `main`
specifically to avoid this and collided anyway, because nothing serialises the allocation and the
other branch read the same `main` minutes later. A human noticing is not a control.

**Decided.**

1. **The earlier entry keeps the identifier**, by commit time, and the later one takes the next free
   number. Renumbering is not unilateral once others cite a decision — move every citation with it,
   in the same commit.
2. Applied here: the genesis decision became **`ARC-15`** (`MODULE_SPEC.md` §4 and the PR 05c plan
   moved with it); the generation-tooling `ARC-10` is unchanged, as the earlier of the two and the
   more widely cited. The 2D style decision became `ARC-14`.
3. **`scripts/check_decision_ids.py` fails on a duplicate** and names both entries with their line
   numbers. It was written against the real defect and verified both ways: it exits 1 on the tree
   before this commit and 0 after.

**Accepted limitation, stated plainly.** There is no CI in this repository yet, so this check is
declared in `.structured-coding/standards.md` and the standards helper *reports* — it does not
block, and `CLAUDE.md` §3.3 is explicit that a clean report is not proof a check ran. Until CI
exists this is a check that must be run, not a gate. That is weaker than the defect deserves and is
recorded as such rather than described as enforcement.

**What this does not do.** A checker catches a collision after it happens. Reserving identifiers up
front, or allocating them at merge, would prevent it. That is a process change rather than a script
and is left undecided.

**Identifier gap.** `ARC-13` and `ARC-14` are allocated on `vis/3d-human-pipeline` and
`vis/2d-generated-assets` and arrive on `main` when those branches land. The gap between `ARC-12`
and `ARC-15` here is that, not a deleted decision.
