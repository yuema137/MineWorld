## ARC-45 — A 2D client draws one town from disclosed passages, and remembers it only as presentation

**Date** 2026-10-08 · **Approved by** the primary session at S12 PR 13a's design freeze · **Relates
to** `ARC-26`, `INV-5`, `INV-13`, [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §6.1 · **Design**
`.structured-coding/plans/mvp0/step-13-client-2d.md` §§4.3, 14

**Problem.** Every `Place` has its own frame, and a World Pack authors no geometry beyond positions
and passages. A 2D client must still draw one continuous town — a street with doors you walk through —
without inventing a fact the world did not state and without binding its drawing to one world's keys
or coordinates.

**Decided.**

1. **Frames are glued at disclosed passages, by translation only.** The place the client first
   stands in is the root of the drawing. A place `P` reached through a passage with `here = h` (in
   the place it was disclosed from, `Q`) and `there = t` (in `P`) is drawn with its origin at
   `origin(Q) + h − t`. Never rotated, never scaled: `+x` east and `+y` north everywhere.
2. **The layout is learned, per world instance, in memory.** A passage is learned when an
   observation discloses it; a place's tags when the observer stands in it. The cache holds no world
   truth and nothing is submitted from it except positions the player chose. It is dropped when the
   `instance` changes. A cache on disk across launches was considered and not chosen: it is state
   kept for a convenience that the world disclosing a doorway's destination (step-13 R-PK-1) provides
   properly.
3. **Appearance comes from world data.** A known place's look is chosen by its tags; a place not yet
   visited is drawn with a generic façade chosen by a stable hash of its id **string**. A person's
   look is chosen from tags and the same hash; a label is `display_name`, else the id.
4. **Dressing anchors only to disclosed doorways or to the extent they span.** Terraces, planters,
   lamps and trees are placed relative to a doorway, or to the rectangle the disclosed doorways of
   the root place span. Nothing is positioned by a world key or an absolute coordinate, so the same
   pack dresses any world.
5. **Decoration is not enforced.** Where no `place-shape` is disclosed, a place's footprint is drawn
   from the pack and never used to refuse or shorten a stride. Where one is disclosed (S15), the
   server's numbers are the only blocking geometry a client draws.
6. **Only the perceived is drawn.** Nobody outside the observer's place is drawn; other buildings
   are façades, honestly empty (`INV-13`).

**Limitation accepted.** Until a doorway names where it leads, a neighbouring building looks
generic until it has been entered once in the running instance.
