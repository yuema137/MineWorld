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
| **Hairstyle category** | A loose messy updo: gathered up and back into a soft twist at the crown, the silhouette noticeably wider and taller than the skull, a soft fringe sweeping across the forehead to her right, and loose strands in front of and below both ears and at the nape. | A close-fitting brown cap over the skull with a small knot on top. The silhouette is **narrower** than the skull at the temples. A hard band of card tips runs across the forehead just above the brows, and isolated dark spikes project from the crown and above her right ear. No strand frames the face. | **FAIL** |
| **Face shape** | Oval tending to heart: soft rounded jawline, small slightly pointed chin, cheekbones visible but not sharp. | A wider, squarer jaw with a heavier chin and flatter cheeks. The lower third of the face is broader than the reference's at the same eye width. | **FAIL** |
| **Expression** | A slight closed-mouth smile, asymmetric, lifting more on her right, with a faint dimple on that side. | A flat closed mouth with no lift at either corner. The candidate **cannot** smile: the CC0 mesh's 26 FACS blendshapes are not carried through the export. | **FAIL** |
| **Gaze** | Off camera, to the viewer's upper right — her own left. | Close to camera. The head and neck carry a 7°/9° yaw offset, which is a glance, not the reference's look away. | **FAIL** |
| **Skin tone** | Warm light, gently tanned, with visible subsurface warmth at the ear, the nose and the jaw edge. | Cooler and greyer, with a hard specular sheen across the forehead, nose and chin that reads as damp rather than as skin. | **FAIL** |
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

**Five hard-fail categories are wrong**: hairstyle category, face shape,
expression, gaze and skin tone. Under `CHARACTER_IDENTITY.md` §1 and
`VISUAL_FIDELITY.md` §5, any one of the first two alone rejects the candidate.

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
2. **The expression and the gaze.** Both are cheap and neither has been done.
   The blendshapes exist in the source and are dropped at export; re-exporting
   with them, then holding a small asymmetric smile, is a bounded change. The
   gaze is a rotation of the eye geometry about each eye's centre, which the
   export already touches when it pushes the iris proud of the cornea.
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
