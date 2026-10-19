## ARC-17 — A declared reference is the source of truth, and vague comparison is prohibited

**Date** 2026-09-27 · **Governs** [`VISUAL_FIDELITY.md`](VISUAL_FIDELITY.md) · **Relates to**
`ARC-9`, `ARC-11`, [`ACCEPTANCE.md`](ACCEPTANCE.md)

A 3D character was brought to human review described as *"same wardrobe, recognisably not the same
person"*. The reference is a young woman in an open burgundy zip hoodie; the candidate was a man in
a red quilted puffer jacket, with black close-cropped hair against a brown voluminous updo, and
with the shirt graphic and backpack — the two strongest recognition cues on that character —
recorded as deferred. Nothing in the sentence was false, and it was still the wrong verdict.

**The root cause is a task definition, not a tuning gap.** The reference was treated as style
inspiration when it is the identity source of truth for one specific character. The implementation
also ran backwards: an available humanoid was chosen first and the reference was then approximated
around what that asset could provide.

**Decided.**

1. When a deliverable declares a reference, the task is to **recreate that specific visible thing
   as faithfully as practical**. Attractive and recognisably something else is a failure.
2. Requirements travel **reference → constraints → asset**, never asset → material edits →
   approximate match. An asset that cannot meet a constraint is evidence about the asset.
3. **Identity-defining features are never deferred** once a milestone is named for its reference.
   Deferring them is legitimate only while the milestone is a technical one.
4. **Hard-fail categories** the agent rejects itself, without involving the operator: perceived
   gender presentation, apparent age, body silhouette, hairstyle category, major hair colour,
   outer-garment category, a missing defining garment structure, grossly wrong face shape, missing
   identity-defining accessories, and the massing or silhouette of a referenced building.
5. **Vague comparative language is prohibited** in fidelity reporting — "broadly similar",
   "roughly matches", "approximately right", "same wardrobe". A verdict names the reference fact
   and the candidate fact side by side. This is the operative half of the decision: the banned
   phrasing is what allowed a categorical failure to read as polish.
6. Comparison is **pixels against pixels, at the reference's own framing**. This project has
   produced the same evidence defect three times — a face judged at 60 px, a character judged at
   street distance, and a review contact sheet at 620 px per tile read as "nothing changed" when
   two things had in fact changed.

**Deliberately general.** The failure was found on a character, but the reasoning that produced it
—"warm town, broadly similar" — applies just as easily to buildings, interiors, vegetation and
lighting, where each piece would run correctly and the whole would resemble nothing. The gate
therefore governs every referenced visual deliverable in both reference clients.

**What this does not change.** `ARC-11` still holds: the operator owns final visual acceptance and
the agent owns the architecture. `REUSE_POLICY.md` still holds: reuse remains preferred, and this
decision constrains only the direction the specification travels. A fidelity rejection resets a
visual candidate and never the engineering beneath it.
