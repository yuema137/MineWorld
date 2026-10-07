# Human review queue

**Only large integrated milestones belong here.** Not individual PRs, structs, components, events,
dependency choices or refactors — the pi-agent owns routine engineering review. A milestone earns
a place here when a coherent capability can be *experienced* as a whole.

Engineering progress is tracked separately in [`MVP_STATUS.md`](MVP_STATUS.md). A milestone parked
here does not block anything else.

**Updated:** 2026-10-01

---

## Framework milestones

| | Milestone | Demonstrates | State |
| --- | --- | --- | --- |
| **A** | Runnable world runtime | load Social Café → server → two clients + an agent → cause a change → all observe it | ✅ **complete 2026-09-27** — `AC-15` holds, 259 tests · restart/persistence is S5 and moves to **B** |
| **B** | Persistent people and social life | Alice and Bob persist, know each other, share an activity, and survive a restart with their history | ❌ |
| **C** | Objects and everyday economy | Market Town: work → earn → buy → inventory changes → another client sees it → persists | ❌ |
| **D** | LM-native persistent characters | speak to Alice in 2D, meet her in 3D, and she reacts consistently with what happened | ❌ |
| **E** | Package composition | a real world assembled from independently installable packs | ❌ |

**Milestone A, and the one thing in it worth watching.** `AC-15` — *there is only one Alice* —
holds, proved against the real binary hosting the real pack. The evidence names identity rather
than appearance, which is the whole point: same world instance, same Alice `EntityId` found by
tag and never by a literal, one monotonic event sequence `[6, 7, 10, 11]` across all three
participants. The sentence that carries it is Alice repeating to the second window what the first
window said, naming the other speaker by identity — **two synchronised copies of Alice could not
produce that sentence.**

The counterfactual is a committed, passing test rather than a caution: two servers, and the test
*asserts* that both clients perceive a barista with the **same** `EntityId` at the same authored
position, because two loads of one pack resolve the same keys (`AC-12` working). An appearance
test passes there and is wrong.

```sh
clients/protocol/run.sh          # windowed, scripted — what a person watches
clients/protocol/run.sh play     # windowed, driven by the keyboard
```

Recorded as **not** done and not approximated: `MVP.md` §9.1's fourth line, *same persisted state
revision*. The world is in memory; persistence is S5's and belongs to Milestone B.

## Default-style milestones — taste, and the operator decides

These are candidates until the operator says otherwise (`ARC-11`). An agent may build and
recommend; it may not declare something the default look.

Named per `ARC-20`, because with two 3D tracks running (`ARC-18`) "the 3D character" no longer
identifies one thing.

| | Milestone | State |
| --- | --- | --- |
| **VIS-2D-1** | Playable 2D default scene with an enterable interior | 🚧 candidate in progress |
| **VIS-3D-GODOT-1** | Reference-matched character in Godot | 👀 **preview 3, 2026-10-06 — not an acceptance request** (`ARC-24`): the updo groomed from strands and cut into cards, the hoodie draped by cloth simulation with a real hood, the eyes level, the skin matte; the face itself deferred by the operator; see below |
| **VIS-3D-GODOT-2** | Integrated Godot slice: character, street, enterable building, interior, lighting, cameras | ❌ |
| **VIS-3D-UE5-1** | Unreal slice of equivalent scope | ⏸ **parked** — spike phase one done (`ARC-21`), operator paused the install |
| **VIS-3D-AB-1** | Godot vs Unreal side-by-side, same reference, same scope | ⏸ parked with `VIS-3D-UE5-1` |

### `VIS-3D-GODOT-1` — preview 3, 2026-10-06: groomed hair, a draped hoodie with a hood

**A preview, not an acceptance request** (`ARC-24`). It answers the operator's verdict on preview 1,
*"too stiff, not like the reference"*, along the route
[`references/CHARACTER_ROUTE_ASSESSMENT.md`](references/CHARACTER_ROUTE_ASSESSMENT.md) §5(1)
recommends: the CharMorph body and rig are kept, and the two parts that failed are made by the
method the industry uses for them. The **face is deferred** by the operator's choice; only the gaze
and the skin material were touched.

**Launch**

```sh
./mineworld-3d              # walk: WASD, mouse to look, Shift jogs, F5 cycles the camera
./mineworld-3d --portrait   # P1–P8 review frames (P8 = rear chest-up) into clients/3d-spike/shots/
godot --path clients/3d-spike --always-on-top -- --bodycheck   # the operator's views + townspeople
godot --path clients/3d-spike --always-on-top --resolution 1200x1500 -- --motion   # frame sequences
```

**What changed — methods, not tuning**

1. **Hair** (`tools/hair_groom.py`, `tools/hair_atlas.py`). Guide strands authored as data — the
   sweep up from the hairline, a twisted knot at the crown, the fringe and the named loose strands —
   grown, clumped, curled and frizzed by Blender's bundled CC0 Essentials hair node groups, each grown
   strand then cut into one card (5,400 cards). The atlas is fine strands with transparent gaps, our
   own procedural work. The procedural ribbons are gone.
2. **Hoodie** (`tools/drape.py`, `tools/garments.py`). The shell starts with 30 mm of slack and is
   dropped by Blender's cloth solver onto the body, held only at the collar and the wrists: it lies on
   the shoulders, the sleeves fall and fold, the front panels hang. The **hood** is a pouch of cloth
   sewn along the draped collar and dropped onto the body and the rucksack. The front is open to the
   collar. Seam-weld, inverted-face and `report_boundaries` checks still run (3 open loops, as
   intended: front-hem-neck, two cuffs).
3. **Tee neckline.** The jagged edge was three things, each located before it was fixed: a strip of
   hoodie crossing the upper chest, zip tape folded in off the collar, and the skin trim's serrated
   edge. Now the hoodie is open to the collar, the crew neck is a separate maroon ribbed band over a
   trimmed edge, and the skin's cut sits under it.
4. **Eyes.** Gaze pitch was −3°, which is *up*; the lids were lowered and hooded. Lids relaxed, iris
   7° down and 15° to her left, pupil shrunk, iris matte warm brown. Values in
   `CHARACTER_ASSET_AUDIT.md` §12.
5. **Skin.** No glossy roughness map; roughness 0.66, specular 0.30, stronger subsurface with warm
   transmittance, her tint warmed.

**Frames**, all from the running client, in
[`presentation/mineworld-default/3D/candidate/`](../presentation/mineworld-default/3D/candidate/):
`side_by_side_chest.jpg` (reference | P1 | P2), `side_by_side_head.jpg` (reference | P6 | P2),
`side_by_side_rear.jpg` (reference | P8, the hood from behind), `full_body_sheet.jpg` (front |
three-quarter | rear | walking), `operator_views.jpg`, `townspeople_whole.jpg`, `motion_idle.gif`
(12.6 s standing loop at 2×), `motion_walk_stop.gif`, and `p1`–`p8` individually.

**Reference fact | candidate fact**, each with the frame that shows it

| Category | Reference | Candidate now | Frame | |
| --- | --- | --- | --- | --- |
| Hairstyle category (§5) | loose messy updo: gathered up and back into a twist at the crown, silhouette taller and wider than the skull | gathered up and back into a knot on the crown; the silhouette is taller than the skull, and narrower at the temples than the reference's | `p6`, `p1`, `p8` | PASS, weakly — see miss 1 |
| Loose strands | fringe sweeping across the forehead to her right; wavy strands before and below both ears; nape wisps | a fringe from the crown across the forehead to her right temple; wavy locks before both ears to the jaw; short wisps at the nape | `p6`, `p2` | PASS |
| Major hair colour | warm mid-brown with caramel; crown 0.42/0.29/0.21 sRGB | warm mid-brown, crown darker (miss 4) | `p6` | PASS |
| Outer-garment category (§5) | open burgundy cotton zip hoodie | open zip hoodie lying on the body, sleeves folding, panels hanging, open to the collar, zip tape on both edges, ribbed cuffs | `p1`, `p2`, `p4` | PASS |
| Defining garment structure: hood (§5) | bunched in soft folds behind and around the neck | a cloth hood lying in folds over the rucksack's lid from behind; at the sides of the neck its ends show as small bunches, mostly behind the straps | `p8`, `side_by_side_rear` | PASS from behind; weak from the front — miss 2 |
| Drawstrings | cream cords down the chest, her right one clear against the tee | two cream cords over the tee inside the zip edges | `p1` | PASS |
| Tee neckline | ribbed crew in maroon-brown | a maroon ribbed band, edge clean | `p1`, `p6` | PASS |
| Gaze | off-camera to her left, level | iris level, touching both lids, no white below, toward her left | `p1`, `p2` (4–5× crops in the session) | PASS |
| Skin | warm, matte, soft glow; lit cheek 0.59/0.38/0.30 sRGB | matte, no sheen on the T-zone; lit cheek 0.51/0.33/0.28 | `p6` | PASS, warmer light in the reference accounts for part of the gap |
| Gender, age, silhouette, accessories | — | unchanged from preview 2 | `p3`, `p4` | PASS |

**Known misses, largest first**

1. **Hair, from the front, reads as a helmet with a band across the crown.** The swept mass is one
   continuous surface of cards; the reference's front is soft, separate locks with highlights. The
   knot reads small from the front.
2. **Hood from the front.** The reference's hood stands proud of both shoulders around the neck; ours
   shows only small bunches beside the straps.
3. **Face** — deferred by the operator: no smile, realistic rather than stylised proportions.
4. **Hair colour** reads darker and redder at the crown than the reference in this light.
5. **Jeans** read light grey in the rear frame's sun; the **rucksack** reads khaki and untextured;
   the **tee graphic** sits mid-chest where the reference's sits high.
6. **Hoodie back**: a soft fold runs down the middle of the seat.
7. **Strand atlas** is our own procedural one; the CC0 OwlishMedia alphas need
   `clients/3d-spike/tools/fetch_hair_alphas.sh` run with a download grant.

**Questions**

1. Does the hair now read as the reference's *category* — a messy updo — and is the helmet-like
   front the thing to fix next, or the knot's size?
2. Does the hoodie read as cotton jersey, not a padded jacket?
3. Is the hood enough as it is from behind, and how much should it stand up at the sides?
4. May `fetch_hair_alphas.sh` be run (138.5 MB, CC0, into scratch)?

### `VIS-3D-GODOT-1` — preview 2, 2026-10-01: the body repaired, and she stands like a person

**A preview, not an acceptance request** (`ARC-24`). It answers the operator's two verdicts on
preview 1, *"the character renders incomplete — the body is broken / missing parts"* and *"still
looks too stiff"*. Hairstyle category still fails §5; the hair, the hoodie method and the face are
the next work and are not in this preview (see the end of this entry).

**Launch**

```sh
./mineworld-3d              # walk: WASD, mouse to look, Shift jogs, F5 cycles the camera
./mineworld-3d --portrait   # the reference-framing frames, incl. P6b_blink, into clients/3d-spike/shots/
godot --path clients/3d-spike --always-on-top -- --bodycheck   # the operator's views + townspeople
godot --path clients/3d-spike --always-on-top --resolution 1200x1500 -- --motion   # frame sequences
```

**1. The missing body parts — located and fixed.** The player character was whole from every
camera, standing and walking. **The townspeople were not:** the seated café customer rendered as
a front panel of jersey, a floating head and two floating hands, with no shoulders, arms or back.
The skin had been trimmed for the *hoodie* (torso, shoulders, both arms to the cuff), the tee had
been cut down to the band the reference character's open front shows, and only she wears the
hoodie. Now one predicate cuts the tee and decides the trim. The tee is whole again (torso,
shoulders, short sleeves, crew neck), and the skin under the hoodie's long sleeves stays as
everyone else's forearms. Evidence: `candidate/townspeople_whole.jpg` (seated and standing, front
and back) and `candidate/operator_views.jpg` (rear and front cameras, standing and walking).

**2. Stiffness — the standing pose is authored, through the `AnimationTree`.** The library's
`Idle` is an action-game stance: feet wide, arms held clear, head back. It is replaced by a
`Stand` clip of our own at the same blend point, the way `Sit` is done:

| | Reference | Candidate now | Frame |
| --- | --- | --- | --- |
| Arms | her left hand on her left strap at chest height, elbow down at her side; the other arm relaxed | the same grip, knuckle 0 mm from the strap at every sampled time; the free arm hangs at her side with the elbow soft, the hand 100–108 mm out from the hip joint | `p1`, `p3` |
| Weight | on one leg, body slightly turned | on her right leg: pelvis dropped 5° to the free side, the spine countering, the free knee soft and the foot a little forward; both soles on the floor (measured) | `p3`, `p4` |
| Head | turned to her left, level, looking off-frame | turned 28–39° to her left over the loop, level | `p1`, `p2`, `p6` |
| Life | — | breathes (a 4.2 s cycle in the chest and shoulders), the gaze drifts over 12.6 s, and she **blinks** every 2.4–5.5 s, sometimes twice | `motion_idle.gif`, `p6b_blink` |

The townspeople use the same standing clip, without the grip.

**Motion:** `candidate/motion_idle.gif` (one 12.6 s standing loop at the reference's
three-quarter chest-up framing, played at 2× speed) and `candidate/motion_walk_stop.gif` (walking
towards the front camera, stopping, standing).

**Known misses, largest first**

1. **Hair** — still the procedural cap; it fails its category (§5).
2. **Face.** The skin carries a specular sheen; the mouth is neutral where the reference smiles;
   the pupils are oversized. The head is now level, so she no longer reads as looking up.
3. **Hoodie** — reads padded; it is a body-surface offset with no drape.
4. **Weight shift** — present and measured, but at full-body distance it is subtle. Lightly
   built, as asked; it can go further.
5. **Blink** — at its 30 ms closed peak a speck of iris shows at the nearer eye
   (`p6b_blink`).

**Questions**

1. Does she still read as stiff, standing and walking (the two GIFs)? If so, what reads stiff
   first?
2. Is the head turn right, at 28–39° to her left, or should she look further away from the
   viewer, as the reference does?
3. Is the weight shift too subtle?

**Next, in this order** (direction from `CHARACTER_ROUTE_ASSESSMENT.md`, keeping the CharMorph
body and rig): the hair groomed from strands and converted to cards; the hoodie as a draped
cloth-simulated garment with a real hood; then a skin material pass. The face beyond tilt, eyes
and sheen is the operator's decision.

### `VIS-3D-GODOT-1` — preview, 2026-09-30

**This is a preview, not an acceptance request** (`VISUAL_FIDELITY.md` §9.1, `ARC-24`). It asks
*is this the right direction, and what is most wrong?* It cannot end in `ACCEPTED`.

**One thing must be said before the frames.** §9.1 requires the §5 hard-fail categories to pass
before a preview, and **hairstyle category does not** (table below). It is shown anyway because
the operator directed that a stable candidate be shown quickly and that hair is exactly what the
operator wants to steer; whether that direction overrides §9.1 here is the operator's call, and it
is question 1. Nothing else in §5 fails; the hood is at risk.

**Launch**

```sh
./mineworld-3d              # walk around her: WASD, mouse to look, Shift jogs, F5 cycles the camera
./mineworld-3d --portrait   # re-capture the seven frames into clients/3d-spike/shots/
```

**What changed since the last state.** The garments were torn — shoulders opened into flaps, a
spiked sleeve and cuffs, a serrated hem, a split thigh and a broken neckline. That is repaired,
and the cause was neither of the two the handover named: glTF splits a vertex at every UV seam,
and the offset pulled each seam's two sides apart by up to 65 mm. The repair, the measurements
and the three further defects found on the way are in commit `9320563`. The eyes, which rendered
silver, are fixed in `3535a50`. The portrait frames are now deterministic — the real mouse used to
turn her between shots, and the "rear" frame once showed her front.

**Frames** — all from one `--portrait` run, in the client's ordinary daylight, in
[`presentation/mineworld-default/3D/candidate/`](../presentation/mineworld-default/3D/candidate/):
`side_by_side_chest.jpg` (reference | three-quarter | front), `side_by_side_head.jpg`,
`full_body_sheet.jpg` (front | three-quarter | rear | walking), and `p1`–`p7` individually.

**§5 hard-fail categories, each with the frame that shows it**

| Category | Reference | Candidate | Frame | |
| --- | --- | --- | --- | --- |
| Hairstyle category | a loose messy updo, gathered into a voluminous twist at the crown, wider and taller than the skull, loose strands at the ears | hair lies close to the skull, swept back from a short fringe; no gathered mass shows from the front or three-quarter, and from behind it reads as a short crop | `p6`, `p5` | **FAIL** |
| Missing defining garment structure | hood bunched behind the neck, standing proud of the shoulders | the hood roll exists as geometry but reads as a thick collar at chest-up, and nothing reads as a hood above the pack from behind | `p2`, `p5` | **AT RISK** |
| Outer-garment category | open burgundy cotton zip hoodie | open zip hoodie: two panels apart, zip tape, ribbed cuffs and hem, intact | `p1`, `p2` | PASS |
| Perceived gender presentation | female | female: bust, waist taper, hip line | `p3`, `p4` | PASS |
| Body silhouette | slim, sloped shoulders, waist taper | the same | `p3` | PASS |
| Apparent age | early-to-mid twenties | young adult | `p6` | PASS, weakly |
| Major hair colour | warm mid-brown | warm brown, redder than the reference | `p6` | PASS |
| Grossly wrong face shape | soft oval tending to heart | jaw/cheek and chin/cheek ratios within the ±10 % the plate allows (`FIDELITY_REVIEW.md` §6); the surface reads flatter-planed | `p6` | PASS, weakly |
| Missing identity accessories | backpack, hand on the left strap | backpack, both straps, left hand on the strap | `p2` | PASS |

**Known misses, largest first, as facts**

1. **Hair** — row one above.
2. **Face and expression.** Reference: a slight closed-mouth smile, warm brown eyes looking off to
   her left under relaxed lids, matte warm skin. Candidate: the mouth reads neutral; the head
   idles tilted up, so the gaze reads upward rather than sideways; the pupils are oversized and
   leave a thin brown rim; the skin carries a specular sheen across the forehead, nose and chin.
3. **Hoodie weight.** The fleece stands 40 mm off the body, and the sleeves and shoulders read as
   padded, not as cotton jersey hanging on the arm — the direction of §5's own worked example
   (zip hoodie → puffer), though no quilting is present.
4. **Hood** — row two above.
5. **Jeans.** At the front of the crotch the denim follows an 11 mm mound in the body and bridges
   the crease below it, which reads as a rounded bulge at full-body distance (`p3`). In the rear
   frame's low sun they read light grey, not mid-blue (`p5`).
6. **Backpack** reads khaki rather than grey-green, untextured (`p5`).
7. **Drawstrings** exist and are hidden behind the straps in every front frame.
8. **Footwear** — dark low shoes read as bare feet at full-body distance. The reference crops
   above the knee, so this is not a reference fact.
9. **Hoodie hem** keeps a fine serration visible at 2× in `p5`.

**Questions for the operator**

1. **Hair.** It fails its category. Is this preview the right moment to steer it, and which way:
   rebuild the procedural card updo with a gathered mass standing well above and behind the
   crown; or source a redistributable CC0 updo (the search so far is in
   `CHARACTER_ASSET_AUDIT.md` §13); or something else?
2. **Hoodie.** Does it read to you as a hoodie or as a padded jacket? A thinner fleece (15–20 mm
   instead of 40) would read more like cotton, at the risk of the tee clipping through again.
3. **The face.** Which is most wrong: the skin's sheen, the eyes, the upward tilt, or the mouth?
4. **Direction.** CharMorph body and face, with garments modelled from the body surface: keep
   going on this base, or replace the head (`ARC-19` §3 permits it)?

### History

`VIS-3D-GODOT-1` first failed on categorical identity mismatch, not polish: the reference is a
young woman in an open burgundy zip hoodie and the candidate was a man in a red quilted puffer
jacket (`ARC-17`).

**It has since been rebuilt and it fails again, and the agent is reporting that rather than
submitting it.** The wardrobe now matches the contract fact by fact — open burgundy zip hoodie
with a hood, cream drawstrings, ribbed cuffs and hem, two panels with zip tape, the cream tee with
its legible mountain-and-slogan print, worn mid-blue denim, the backpack with her hand on the
strap — and the *person* still does not. Five hard-fail categories are wrong: hairstyle category,
face shape, expression, gaze and skin tone. A viewer shown the two side by side would say *same
clothes, different person*, so `VISUAL_FIDELITY.md` §9 forbids requesting review.

The per-category verdict, the evidence and what would close each failure are in
[`presentation/mineworld-default/3D/FIDELITY_REVIEW.md`](../presentation/mineworld-default/3D/FIDELITY_REVIEW.md);
the runtime frames are in `presentation/mineworld-default/3D/candidate/`. Reproduce them with
`./mineworld-3d --portrait`.

**One decision is the operator's and blocks the largest remaining failure.** The CC0 Vitruvian
head is one head, and CharMorph's morphs reshape the body far more than the skull. Reaching this
face means either searching the face morph sliders, or accepting the base mesh cannot get there
and replacing the head — which `ARC-19` §3 explicitly permits, while forbidding only that we build
character-reconstruction technology ourselves. More effort on the current head should wait on
that.

The rig, retarget, animation, cadence, footwear, ground-contact, garment-modelling, hair-modelling
and texture work underneath it is unaffected and is kept.

### What a review package contains (`ARC-20`)

Milestone id · what changed · **the exact launch command** · real runtime screenshots · the
canonical reference · a side-by-side where applicable · known limitations · the specific
subjective questions being asked. The operator must be able to launch, look, walk and judge
quickly. An engineering log is not a review artefact.

### Reaching review does not stop work

On `READY FOR HUMAN VISUAL REVIEW`: preserve the runnable candidate, save the screenshots, record
it here, **stop subjective polishing on that branch**, and move to independent work. Review is a
branch-level checkpoint, never a global barrier. Neither 3D track waits on the other, and none of
the framework milestones wait on any of them.

## Style infrastructure — architecture, and it never waits here

Tracked in [`MVP_STATUS.md`](MVP_STATUS.md), listed only so the split is visible: Presentation and
Asset Pack interfaces, style manifest schema, provenance, generation-pipeline integration,
renderer bindings, style switching, validation and composition. None of it blocks on a taste
decision, and none of it may be tied to whichever style happens to be default.

---

## Open questions for the operator, not blocking anything

- **The project's name.** Microsoft published an unrelated "MineWorld" in 2025 — a video-generative
  world model — with a paper, a repository and a Hugging Face presence. Purely a discoverability
  and package-naming collision (`ARC-12`), and it blocks no engineering. Worth a deliberate
  decision before public launch rather than discovering it in a search result.

## Accepted, not to be re-litigated

- **The 2D and 3D presentation direction**, as of 2026-09-27. Movement, camera work and the
  3D lighting rig are accepted; the visual track now improves fidelity toward the references
  rather than re-deciding the direction.
- **Procedural SVG is retired as the 2D art strategy**, keeping its layout, placement, projection,
  camera and occlusion logic. Visuals get replaced; architecture does not.
- **Primitive humanoids are below baseline** and are being replaced by a reusable rigged pipeline.
- **The default character is an identity reconstruction of `04_character_closeup.png`**
  (`ARC-19`), which supersedes `ARC-4`'s facial-fidelity exclusion for that one image. `ARC-4`'s
  reasoning survives, applied to the right object: the bar is high for **our** default character,
  which we pay once and ship as an asset, and stays low for **what the framework requires**, which
  is the humanoid profile and nothing about appearance.
