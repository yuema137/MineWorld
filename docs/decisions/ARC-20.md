## ARC-20 — Visual milestones are named, reviewed as packages, and never block other work

**Date** 2026-09-27 · **Relates to** `ARC-11`, `ARC-17`, `ARC-18`,
[`ACCEPTANCE.md`](ACCEPTANCE.md), [`HUMAN_REVIEW_QUEUE.md`](HUMAN_REVIEW_QUEUE.md)

With two 3D tracks running (`ARC-18`), "the 3D character" and "the 3D scene" stop being unique
descriptions, and a review round has already been lost to an ambiguous verdict (`ARC-17`).

**Decided.**

1. **Named milestones.** `VIS-2D-1` (playable 2D default scene with an enterable interior);
   `VIS-3D-GODOT-1` (reference-matched character in Godot); `VIS-3D-GODOT-2` (integrated Godot
   slice: character, street, enterable building, interior, lighting, movement, cameras);
   `VIS-3D-UE5-1` (Unreal slice of equivalent scope); `VIS-3D-AB-1` (the side-by-side).
2. **The A/B must compare like with like.** Same reference, same demo scope, both tracks pushed.
   A stale placeholder against a polished slice measures nothing, and `ARC-18` is only decidable
   on honest evidence. Godot is pushed properly before it is judged: the current gap traces to the
   wrong mesh, hair, garments, materials and lighting, not to a demonstrated rendering ceiling.
3. **A review package, not an engineering log.** Milestone id · what changed · the exact launch
   command · real runtime screenshots · the canonical reference · a side-by-side where applicable
   · known limitations · the specific subjective questions being asked. The operator must be able
   to launch, look, walk and judge quickly.
4. **Waiting for review never blocks development.** On reaching `READY FOR HUMAN VISUAL REVIEW`,
   an agent preserves the runnable candidate, saves the screenshots, records it in
   `HUMAN_REVIEW_QUEUE.md`, **stops subjective polishing on that branch**, and moves to
   independent work. Review is a branch-level checkpoint, never a global barrier, and neither 3D
   track waits on the other.
5. **The engine choice is not the agent's.** Whether Godot remains default, Unreal is promoted, or
   both are supported is a product decision the operator makes on `VIS-3D-AB-1`.
