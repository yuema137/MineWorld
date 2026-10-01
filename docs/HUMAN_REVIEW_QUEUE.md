# Human review queue

**Only large integrated milestones belong here.** Not individual PRs, structs, components, events,
dependency choices or refactors — the pi-agent owns routine engineering review. A milestone earns
a place here when a coherent capability can be *experienced* as a whole.

Engineering progress is tracked separately in [`MVP_STATUS.md`](MVP_STATUS.md). A milestone parked
here does not block anything else.

**Updated:** 2026-09-30

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
| **VIS-3D-GODOT-1** | Reference-matched character in Godot | 👀 **preview 2026-09-30 — not an acceptance request** (`ARC-24`); one §5 hard-fail category is open and is the first question; see below |
| **VIS-3D-GODOT-2** | Integrated Godot slice: character, street, enterable building, interior, lighting, cameras | ❌ |
| **VIS-3D-UE5-1** | Unreal slice of equivalent scope | ⏸ **parked** — spike phase one done (`ARC-21`), operator paused the install |
| **VIS-3D-AB-1** | Godot vs Unreal side-by-side, same reference, same scope | ⏸ parked with `VIS-3D-UE5-1` |

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
