## ARC-22 — The default cast stays inside the range one albedo can carry

**Date** 2026-09-27 · **Relates to** `ARC-11`, `ARC-9`, `DEP-8` · **Scope** the MineWorld default
World Pack only

The 3D crowd is built by tinting **one** photographic skin albedo per instance, because the CC0
Vitruvian set ships a single body texture and a single head. `clients/3d-spike/scripts/npc.gd`
already records the limit at the palette itself: *"pushed further they read as paint rather than
as people."* The palette nonetheless ran to `Color(0.63, 0.49, 0.39)`, well past that point.

**The technical fact, stated precisely because it is easy to restate wrongly.** Tinting an albedo
changes its colour and nothing else. Facial structure, hair geometry, and the way light behaves in
skin all stay whatever the source asset was. So a tint far from the source does not produce a
person of a different ethnicity — it produces the **same** person painted a different colour, and
it reads as exactly that. The defect is in the method, not in the range being attempted.

**Decided: the default cast's skin and hair palettes stay inside the range this one albedo carries
convincingly** — broadly fair European through East Asian — until there is a second texture set
and a second head to support anything wider. Applies to the 3D crowd, the 2D generated cast, and
generated textures under `ARC-9`.

**What this decision is not.** It is not a finding that a wider cast is expensive in itself, and a
later agent must not cite it that way. The cost is in **asset production** — a second albedo, a
second head mesh, matching grooms — and `ARC-19` already establishes that the default character's
asset cost is paid once by this project. When a second texture set exists, widening the palette is
a palette change and needs no new decision.

**It binds nothing but the default.** `ARC-11` governs: the default style is the operator's taste
and never a kernel assumption. Nothing in the contracts, the kernel, the loader or the renderer
bindings knows what a skin tone is; a World Pack ships its own Presentation Pack and its own cast
with no reference to this palette. **If a later change makes this palette hard to replace, that
change is the defect, not this record** — the same guardrail `ARC-14` carries for the 2D style.

**Do not widen the palette back on your own initiative.** Its narrowness is deliberate and
measured against a known asset limit. Widening it is an operator decision, and it should be
prompted by a second texture set existing, not by the palette looking short.
