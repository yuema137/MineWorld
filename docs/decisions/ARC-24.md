## ARC-24 — A stable visual candidate is previewed early; acceptance stays strict

**Date** 2026-09-29 · **Refines** `ARC-17`, `ARC-20` · **Governs**
[`VISUAL_FIDELITY.md`](VISUAL_FIDELITY.md) §9.1

**The operator's direction:** as soon as the visual track has a relatively stable candidate, show
it — the operator's judgement is what makes a visual result acceptable, so the operator has to be
able to see it.

`VISUAL_FIDELITY.md` §9 forbade requesting review while the candidate was not recognisably the
reference. That was right for **acceptance** and wrong for **steering**: across two rounds every
candidate was withheld, correctly under the rule, and the operator saw nothing while effort went
into directions only they could have corrected.

**Decided.** Two reviews, never confused. A **preview** happens as soon as a candidate is stable —
launches cleanly, renders without broken geometry or missing textures, and the next change is
refinement rather than repair — and asks *is this the right direction, and what is most wrong?*
It can never end in `ACCEPTED`. An **acceptance review** still requires §9's answer to be yes, and
only the operator marks anything accepted (`ARC-11`).

**What does not relax.** The hard-fail categories of `ARC-17` must pass before a preview too; a
preview is not a route for a categorically wrong candidate. The banned comparative language stays
banned. And instability is not previewed: torn meshes, featureless faces and unloaded textures are
repair, which is the agent's to finish.
