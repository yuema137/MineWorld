# `VIS-3D-GODOT-1` — fidelity review of the current candidate

**Date** 2026-09-27 · **Reference** `references/04_character_closeup.png`
· **Contract** [`CHARACTER_IDENTITY.md`](CHARACTER_IDENTITY.md)
· **Method** [`../../../docs/VISUAL_FIDELITY.md`](../../../docs/VISUAL_FIDELITY.md)
· **Evidence** [`candidate/`](candidate/), captured from the running client

> **Verdict: FAIL. This candidate is not submitted for human review.**
>
> `VISUAL_FIDELITY.md` §9 asks whether a person who had never seen this project,
> shown the reference and the candidate side by side, would reasonably recognise
> them as the same character. Shown `candidate/side_by_side_head.jpg`, they would
> not. They would say: same clothes, different person.

This document exists because reporting that as near-success is the failure
`ARC-17` was written about. It is not a request for review and it is not a
milestone; it is the record of where the candidate actually is.

---

## 1. How to reproduce the evidence

```sh
./mineworld-3d --portrait     # writes clients/3d-spike/shots/P*.png
./mineworld-3d                # walk around her
```

`--portrait` places the character on the promenade, in the client's ordinary
daylight rig, and captures seven frames from a camera at the reference's own
framing: chest-up front and three-quarter, full front, three-quarter and rear,
a head close view, and one mid-stride. The frames in `candidate/` are those
files, scaled and JPEG-compressed, and nothing else. `Camera3D.fov` is vertical,
so the portrait crop changes the aspect and not the framing.

## 2. Per category, reference fact beside candidate fact

Largest miss first. Comparative adjectives are not permitted here
(`CHARACTER_IDENTITY.md` §1).

| Category | Reference | Candidate | |
| --- | --- | --- | --- |
| **Hairstyle category** | A loose messy updo: gathered up and back into a soft twist at the crown, the silhouette noticeably **wider and taller than the skull**, a soft fringe sweeping across the forehead to her right, and loose strands in front of and below both ears and at the nape. | Hair swept up from the hairline and gathered at the crown, reading as strands rather than as a shell, with a short fringe and a few loose wisps beside each ear. The silhouette still follows the skull instead of standing away from it: it is roughly the width of her head at the temples where the reference's is wider, and the crown mass is thin. | **FAIL** |
| **Face shape** | Oval tending to heart: soft rounded jawline, small slightly pointed chin, cheekbones visible but not sharp. | A wider, squarer jaw with a heavier chin and flatter cheeks. The lower third of the face is broader than the reference's at the same eye width. **Unchanged, and the largest remaining miss.** | **FAIL** |
| **Expression** | A slight closed-mouth smile, asymmetric, lifting more on her right, with a faint dimple on that side. | A slight asymmetric lift is now **baked into the mesh** — 4.0 mm at her right corner, 2.5 mm at her left, with a falloff over 22 mm. It is present and it is the right asymmetry, but at 4 mm it reads as a neutral mouth that is not quite flat rather than as the reference's smile. No dimple. | **PARTIAL** |
| **Gaze** | Off camera, to the viewer's upper right — her own left. | The eye geometry is rotated 9° toward her left and 3° up about each eye's own centre, on top of the head and neck's 7°/9° yaw. The direction is right and it is measured (the export prints the iris direction before and after). At 9° it is a glance rather than the reference's clear look away, because more than that showed too much sclera and read as startled. | **PARTIAL** |
| **Eye opening** | Upper lid covering the top of the iris; a relaxed, slightly narrowed eye. | The lid sits high, so white shows above and below the iris and the eye reads wide. This is the CC0 mesh's lid shape, not a material setting. | **FAIL** |
| **Skin tone** | Warm light, gently tanned, with visible subsurface warmth at the ear, the nose and the jaw edge. | Cooler and greyer, with a specular sheen across the forehead, nose and chin that reads as damp rather than as skin. | **FAIL** |
| **Brows** | Dark brown, thick, naturally arched, groomed but not drawn. | Present and arched, in a material 42 % darker than the hair. CharMorph ships the brows as a particle system, which does not survive a glTF export, so before this the face had none at all. They are thinner than the reference's. | **PASS, weakly** |
| **Backpack colour** | Grey-green olive canvas; the strap is grey-green upper webbing with a dark navy-black lower section and a visible adjuster. | Straps in the right place and gripped, but the webbing is a flat neutral grey-green, lighter than the reference's olive, and it is untextured. | **FAIL** |
| **Hood** | Bunched in soft folds behind and around the neck, **standing proud of the shoulders**, reaching about ear height beside the neck. | Present as a roll behind the neck, but sitting level with the shoulder line rather than above it; at chest-up framing it reads as a thick collar. | **FAIL** |
| **Drawstrings** | Cream flat cords hanging from the hood down the chest, her right one clearly visible against the tee. | Both cords exist and hang down the chest; in this view they are occluded by the backpack strap and the raised forearm. Visible in `candidate/p2_portrait_tq.jpg`. | **PARTIAL** |
| **Freckles** | Present and prominent, scattered densely across the bridge of the nose and both upper cheeks, thinning toward the temples. | The same distribution, placed from the geometry rather than painted onto the UV sheet, and readable at portrait framing. | **PASS** |
| **Outer-garment category** | Open burgundy cotton zip hoodie: two front panels apart with the tee between them, lighter zip tape down each panel edge, ribbed cuffs, ribbed hem, no collar. | The same garment and the same category: panels apart, tape down both edges, ribbed cuffs at the wrists, ribbed hem at the hip. | **PASS** |
| **Hoodie colour** | Burgundy / brick red, desaturated and warm. | The same. | **PASS** |
| **Tee and graphic** | Cream/oatmeal crew tee; a dark brown three-peak mountain range with snow as negative space, above "Good Places" / "Brighter People" in a rounded humanist face. | The same, legible at portrait framing, both lines correct. The print sits slightly smaller and lower relative to the collar than the reference's. | **PASS** |
| **Jeans** | Mid-blue worn denim, relaxed straight, mid-rise. | The same cut and tone. | **PASS** |
| **Backpack presence and grip** | Padded straps over both shoulders; her left hand grips the left strap at chest height, fingers curled over the webbing. | The same, and the bag rides the spine because it is skinned to it rather than parented to the node. | **PASS** |
| **Perceived gender presentation** | Female, unambiguous at chest-up framing. | Female on the body — shoulder 0.392 m, waist 0.247 m, hip 0.346 m, measured off the morphed mesh. At head framing the squarer jaw works against it. | **PASS, weakly** |
| **Apparent age** | Early-to-mid twenties. | Adult, not adolescent. The blunter jaw pushes it younger and less specific than the reference. | **PASS, weakly** |
| **Body silhouette** | Slim, athletic, narrow-to-average sloped shoulders, visible waist taper. | The same, from CharMorph's Ultra Feminine morph: shoulder 0.419 → 0.392 m, waist 0.264 → 0.247 m, hip 0.344 → 0.346 m. | **PASS** |

**Three hard-fail categories are still wrong**: hairstyle category, face shape
and skin tone; the eye opening compounds the third. Under
`CHARACTER_IDENTITY.md` §1 and `VISUAL_FIDELITY.md` §5, either of the first two
alone rejects the candidate.

Expression and gaze moved from FAIL to PARTIAL in this pass: both are now real,
baked into the mesh, and in the right direction, and both are smaller than the
reference's. Brows moved from absent to present.

## 3. What the wardrobe proves, and what it does not

The clothing is the same clothing. Every garment structure the contract names as
a checkable fact is present as geometry: the hood, the cords, the ribbing, the
two panels, the zip tape, the graphic, the straps, the grip. That is a real
result and it is what the rebuilt pipeline bought.

It is also not the thing being judged. A viewer matching two images matches the
**person** first, and the person does not match. Recording "same wardrobe" as
progress toward identity is exactly the sentence `ARC-17` bans.

## 4. What would close each failure

Ordered by how much identity each one buys, not by effort.

1. **The face.** The CC0 Vitruvian head is one head; the morph system reshapes
   the body far more than the skull. Closing this means either driving
   CharMorph's face morph sliders toward the reference's proportions — which is
   a search over sliders that nothing here has attempted yet — or accepting that
   the base mesh cannot reach this face and replacing the head. `ARC-19` §3
   permits evaluating other tools for exactly this and forbids only building
   character-reconstruction technology ourselves. **This is the decision that
   should be taken before more effort goes into the current head.**
2. **The expression and the gaze, further.** Both are now baked and both are
   under-done. The smile is a 4 mm corner lift on a mesh that has no other
   facial deformation available, and carrying the 26 FACS blendshapes through
   the export instead would give a real zygomatic pull and an eye-crinkle. The
   gaze is capped at 9° because the lid shape shows too much sclera beyond
   that, which is the same limit as the eye-opening row: both want the head
   decision in (1) first.
3. **The hair.** The card atlas fix made strands render as strands, and the
   remaining gap is silhouette: the mass has to be wider than the skull at the
   temples and carry loose strands that separate from it. Card-hair authored
   procedurally from one gather point may not reach that; a groom converted to
   cards, or hand-modelled cards, is the honest alternative, and the licence
   search for a redistributable CC0 updo is in
   [`../../../docs/CHARACTER_ASSET_AUDIT.md`](../../../docs/CHARACTER_ASSET_AUDIT.md).
4. **Skin response.** The specular sheen and the cool cast are material
   settings over a CC0 albedo, not geometry — roughness, subsurface strength and
   the albedo tint. Cheap, and worth doing only after the face is settled,
   because grading a face that is the wrong face is what §9 warns against.
5. **Hood loft and strap texture.** Both are small, bounded, and neither changes
   the verdict.

## 5. What is not in question

The engineering under the appearance is unaffected by this verdict and is kept
(`VISUAL_FIDELITY.md` §11): the humanoid profile, the BoneMap, the retarget, the
distance-driven cadence, the ground-contact measurement, the garment and hair
modelling tools, the texture pipeline and the import-flag fix. Their numbers are
in [`../../../docs/HUMANOID_PROFILE.md`](../../../docs/HUMANOID_PROFILE.md).

---

## 6. The face morph search, as measurement

The operator directed that the face morphs be tried before the head is replaced,
and that the search be a measurement rather than a series of impressions. It
was, and it settles the question: **the head does not need replacing for the
jaw, the chin or the eye.**

Two scratchpad tools drive it (they import CharMorph, so they stay out of the
repository — `CHARACTER_ASSET_AUDIT.md` §12): one sweeps each morph over its
range and reports what each contract row moves through, the other applies a
candidate stack and reports the resulting ratios against the reference.

### The rows, and where the reference sits

The reference plate is a three-quarter view, so absolute widths foreshorten.
**Ratios of two horizontal widths do not**, so the comparison is made on those,
read off the plate in pixels at 3× (`cheek 458 px, jaw 375, chin 180`) and good
to about ±10%.

| Row | Reference | Base (Ultra Feminine) | After the morph stack |
| --- | --- | --- | --- |
| jaw width / cheek width | **0.819** | 0.948 | **0.849** (3.7% miss) |
| chin width / cheek width | **0.393** | 0.397 | **0.374** (4.7% miss) |
| visible sclera | relaxed lid, iris top covered | 105 points | **59 points (−44%)** |

Both misses are inside the ±10% the pixel reads carry.

### What each morph can actually reach

Single-morph spans, measured on the Ultra Feminine base, against the row its
family is aimed at. This is the list that would justify replacing the head if it
came back empty — it does not.

| Morph | Row | Span | Verdict |
| --- | --- | --- | --- |
| `Jaw_Ramus_Extrusion` | jaw width | 5.51% (0.1286 … 0.1359 m) | usable |
| `Jaw_Width` | jaw width | 4.92% | usable |
| `Jaw_Mandible` | jaw width | 3.11% | usable |
| `Jaw_Mandible_GonialAngle` | jaw width | 2.86% | usable |
| `Jaw_Definition` | jaw width | 1.27% | marginal |
| `Jaw_Ramus_LocY` | jaw width | 0.43% | **dead end** |
| `Face_Maxilla`, `Head_TemporalLines` | jaw width | 0.00% | **dead end** |
| `Face_Zygomatic_Bone` | cheek width | 11.22% (0.1312 … 0.1468 m) | the strongest single control on the taper |
| `Cheeks_BoneDefinition`, `Cheeks_UpperCheek_Bone`, `Cheeks_CheeksBonePositionZ`, `Face_Puffy`, `Cheeks_BuccalFat` | cheek width | 0.00–0.79% | **dead ends for width** (they move flesh in depth, which is why two are still used, for softness) |
| `Chin_Width` | chin width | 19.19% (0.0498 … 0.0604 m) | usable |
| `Chin_SecondaryWidth` | chin width | 12.53% | usable |
| `Chin_Height`, `Chin_Portrusion`, `Chin_ChinCleft` | chin width | 0.00–0.01% | **dead ends for width** |

**No single morph closes the jaw taper; four stacked ones plus the cheekbone
do.** Narrowing the jaw and widening the zygomatic each contribute about half.
At their full extremes the ratio lands at 0.813 against the reference's 0.819 —
a 0.7% miss — but the face then reads gaunt, which fails the contract's own
"soft, rounded" wording. The shipped values sit at roughly 0.9 of the extremes
with `Cheeks_BuccalFat` and `Face_Puffy` putting flesh back, trading a 0.7% miss
for 3.7% and a face that is not bony.

### The eye was not skull geometry

The operator's hypothesis was right. `Eyes_UpperLidOpenness` and
`Eyes_LowerLidOpenness` exist as morphs, and the base pose simply sits near the
open end of their range. Four morphs move the visible-sclera count
monotonically: `Eyes_Eyelid_Hooded` (105 → 84 at +1), `Eyes_UpperLidOpenness`
(105 → 84 at −1), `Eyes_EyeBagsSize` (105 → 92 at +1), `Eyes_LowerLidOpenness`
(105 → 90 at −1). Stacked at moderate values they reach **−44%**, which is the
relaxed lid the reference shows.

**One measurement had to be thrown away and replaced**, which is worth recording
because it nearly produced a wrong conclusion. The first eye metric was the
vertical extent of the exposed eyeball, and it came back *non-monotonic across a
single morph's own range* — `Eyes_Eyelid_Hooded` gave 0.0118 m at +0.5 and
0.0230 m at +1.0. An extent decided by one stray vertex at the top or bottom is
not a measurement. The count of exposed vertices is monotonic and is what the
numbers above use.

Likewise, three morphs reported a 100%+ span — `Eyes_Size`,
`Face_EyeSocket_Protrusion`, `Age_Baby` — by moving the region out of the fixed
measurement band entirely, so the band found nothing and returned zero. Those
are **measurement failures, not results**, and none of them is cited as a
finding above.

And the first run of the whole sweep reported `0.00%` for *every* chin and
*every* cheek morph, which looked exactly like the dead end that would have
justified replacing the head. It was not: the band took `max |x|` over the whole
head at a height, which measures the neck under the chin and the ears beside the
cheeks. A measurement that cannot see the thing it measures reports no effect,
and it is indistinguishable from the thing not moving.

### What this does not settle

The **face shape row in §2 is still FAIL**, because the ratios are not the whole
of a face: the reference's jaw is soft and rounded where the candidate's is
still flat-planed, and that is surface form rather than proportion. The morph
search says the *proportions* are reachable and the head should not be replaced
on their account. Whether the remaining difference is reachable is a separate
question, and the cheap test for it has now been run.
