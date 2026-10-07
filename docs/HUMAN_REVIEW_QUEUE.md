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
| **VIS-3D-GODOT-1** | Reference-matched character in Godot | ❌ **failed fidelity gate 2026-09-27** — rebuilding |
| **VIS-3D-GODOT-2** | Integrated Godot slice: character, street, enterable building, interior, lighting, cameras | 👁 **environment PREVIEW 2026-09-30, updated 2026-10-06** — not an acceptance request; see below |
| **VIS-3D-UE5-1** | Unreal slice of equivalent scope | ⏸ **parked** — spike phase one done (`ARC-21`), operator paused the install |
| **VIS-3D-AB-1** | Godot vs Unreal side-by-side, same reference, same scope | ⏸ parked with `VIS-3D-UE5-1` |

`VIS-3D-GODOT-1` failed on categorical identity mismatch, not polish: the reference is a young
woman in an open burgundy zip hoodie and the candidate was a man in a red quilted puffer jacket
(`ARC-17`). The rig, retarget, animation, cadence, footwear and ground-contact work underneath it
is unaffected and is kept.

### `VIS-3D-GODOT-2` environment — PREVIEW, 2026-09-30

**This is a preview under `ARC-24`, not an acceptance request.** It asks *is this the right
direction, and what is most wrong?* It cannot end in `ACCEPTED`. The character slot holds the
current **technical stand-in** (`npc.gd` reference build over the Vitruvian rig), not the
`04_character_closeup.png` character, which is `VIS-3D-GODOT-1`'s track.

**Launch.** Branch `vis/3d-godot-2-environment`, from the repository root, with Godot 4.7 on
`PATH`:

```sh
./mineworld-slice          # walk it: W/S/A/D, mouse look, Shift jog, F5 cycles the three cameras
```

You start on the pavement west of the café, facing it. The door is at the café's left end.

**Frames** — `clients/3d-spike/shots/slice/preview/`, captured from the running client at
1600×900 in its normal lighting:

| | Frame |
| --- | --- |
| street wide | `01_street_wide.jpg`; at `05`'s framing, `01r_street_at_05_framing.jpg` |
| approach | `02_cafe_approach.jpg` |
| café exterior | `03_cafe_exterior.jpg`; at `03`'s framing, `03r_cafe_at_03_framing.jpg` |
| doorway | `04_doorway_from_pavement.jpg`, `04b_doorway_from_inside.jpg` |
| interior wide | `05_interior_wide.jpg` |
| interior with the stand-in | `06_interior_with_standin.jpg` (rear camera), `06b_standin_facing.jpg` |
| also | `07_counter.jpg`, `08_interior_looking_out.jpg` |
| **side by side** | `sbs_03_cafe_frontage.jpg`, `sbs_05_main_street.jpg`, `sbs_03_interior_through_glass.jpg` |

**Measured, not eyeballed.** These come from `--drive`, `--measure` and `--threshold`, read from the built
colliders and the stand-in's own mesh:

```text
stand-in stature 1.750 m (mesh) · eye 1.66 m · walk 1.45 m/s · capsule r 0.30 m
door clear width 0.980 m · clear head 2.220 m · step 0.15 m · café floor-to-ceiling 3.300 m
counter 1.06 m · pavement 4.50 m · carriageway 7.20 m · kerb 0.14 m · street 68 m · sun 19.3°
café frontage: open only at the door (x 3.00–3.95) at 0.30, 1.00 and 1.60 m — ray scan
walk-in: through the door on foot, place street.main → cafe.main → street.main; closed
   loop inside (13.3 m, closes within 0.10 m); stopped by back wall, counter (0.30 m short
   = capsule radius) and glazing; three cameras indoors, body moved 0.0000 m on each switch
threshold (mean linear luma): pavement 0.239 → doorway 0.330 → 2 m in 0.341 → deep 0.354;
   worst step ×1.38 (limit ×3); looking out through the glass 0.00 % clipped
frame cost 10.9–17.6 ms at 1600×900 (Apple M5) · build ~6 s · clean launch, clean headless exit
```

**Known misses, largest first, as facts.**

1. **The interior reads brighter and emptier than `03`'s.** Through `03`'s glass the room is a
   dim amber space, darker than the street. Shelving of jars and bottles runs floor to ceiling,
   six pendants glow, there is a chalkboard menu, and a barista stands at a loaded pastry case.
   Through the candidate's glass it is a pale, evenly lit room: plaster walls and pale ceiling
   boards fill most of the view, shelving is only on the back wall about 10 m away, and there is
   no barista. Measured, the inside frames (mean 0.33–0.35) are brighter than the pavement
   frame (0.24).
2. **Paving.** `03` and `05` have grey-buff setts with dark joints, raked by the sun. The
   candidate has warm beige rectangular paving whose joints do not read at `03`'s framing.
3. **Light on the frontage.** In `03` a low sun rakes across the shopfront, with hard shadows
   and a saturated blue sky. At the same framing the candidate's frontage is in soft, even
   light with no cast-shadow pattern, under a pale grey-blue sky.
4. **Planting density.** `03` has two hanging baskets of trailing flowers, ivy over the pier, a
   flowering planter box and potted plants on the tables. The candidate, in the same frame, has
   one hanging basket, one climbing strand and one potted plant.
5. **Window lettering.** "Better Coffee Brighter Days" is on the glass, facing the street, but
   it is white script over a bright interior and barely legible. In `03` it is crisp.
6. **A-board scale.** `standing_chalkboard_01` measures 1.51 m tall against `VISUAL_SLICE.md`
   §3.4's 0.90–1.15 m. `DEP-8` forbids rescaling it in the node tree, so it needs a different
   asset or an authored board.
7. **Interior rug** is a flat, untextured colour. In the rear-camera interior frame
   (`06_…`) a pendant fills the upper left.
7a. **Street doors read as blank dark slabs.** The flats' door in `01_street_wide.jpg` and Maple
   & Co.'s in the third-person frames are flat dark leaves. Their raised panels stand about 5 mm
   proud and do not read in shade. In `05`, the doors are visibly panelled and glazed.
8. **Not yet connected to the server** (`VISUAL_SLICE.md` §9). Place identity is a client-side
   volume reporting `cafe.main` / `street.main`, and no intent round-trip exists yet.

**Fixed on the way to a runnable slice** (it had never run): the launcher's scripted modes,
which never ran because of a flag typo; the fetch script, which did not parse; a pot blocking
the door; glass you could walk through; an unglazed bay beside the door; every upper window and
shop window buried inside solid walls; tree boughs and bicycle frames drawn at the world
origin as a floating fan of sticks; foliage cards with no alpha cut; review views that started
inside furniture; and leaks on exit. After the preview went out: the flower shop's gable roof
had its two tiled planes rising in a V (a rotation-sign error), and the flats' stone plinth ran
across their doorway. Both are fixed, and `01r_street_at_05_framing.jpg` and
`01_street_wide.jpg` are re-captured. The commits on the branch carry each one.

**Assets.** 45 Poly Haven CC0 assets, checked one by one (`clients/3d-spike/ASSETS.md`).
**Two are excluded on licence:** `CashRegister_01`, whose atlas reproduces a Bank of Canada
banknote, and `hanging_picture_frame_02`, whose painting is signed by an uncredited artist.

**Questions for the operator.**

1. Is the street's direction right — warm sandstone and brick, green painted shopfronts,
   generated street trees — and what is most wrong with it?
2. Should the interior go darker and denser, like `03`, with a dim amber room, glowing pendants
   and the street brighter than inside? That raises the exposure step at the door, which is
   ×1.38 now.
3. Should the paving go to grey setts, as in both references?
4. Of interior density, paving, light on the frontage and planting, which should come first?

#### Preview update, 2026-10-01 — playability: camera key, jump, doors

From the operator's first walk. **Still a preview.** Same launch command.

| | What changed | Frames (`clients/3d-spike/shots/slice/preview/`) |
| --- | --- | --- |
| camera | **V** cycles the camera (F5 kept: on a Mac it is a media key without Fn). A controls line is always on screen, and a centred toast names the new mode ("Third person — rear"), fading after about 2 s. Both scenes share one HUD | `controls_hud_toast.jpg`, `controls_hud_after_fade.jpg` |
| jump | **Space** jumps 0.45 m by design (0.49 m measured at 60 Hz): a person, not Minecraft's 1.25 m. From the floor only, so no mid-air second jump; client-local rendered movement, not a server action. No jump clip exists in the rig's set, so the body holds its idle/walk blend in the air | `jump_sequence.jpg` |
| doors | every street door now has a light architrave, a recessed leaf with real panels or glazing, a brass handle on the front, and a step. House doors are coloured to stand out. The café's door is the one that opens: it stands open onto the lit room | `doors_from_the_pavement.jpg` (all 12), `door_detail_shop.jpg`, `door_detail_house.jpg`, `door_cafe_enterable.jpg` |

**One question this raises.** The operator's rule, "buildings must be real and enterable", suggests making
a second shop enterable. `VISUAL_SLICE.md` §4 lists *a second enterable building* as explicitly out
of scope, because the Godot and Unreal slices must stay comparable. Should §4 change? Until it
does, the other doors read as doors and do not open.

#### Preview update, 2026-10-06 — light, setts, planting, a second shop you can enter

**Still a preview, not an acceptance request** (`ARC-24`). It asks: *is this the right direction,
and what is most wrong now?* The operator judged the café and street "looks good" on 2026-10-01;
this round works the misses listed above.

**Launch.** Branch `vis/3d-godot-2-environment`, from the repository root, with Godot 4.7 on `PATH`:

```sh
./mineworld-slice            # walk it: W/S/A/D, mouse, Shift jog, Space jump, V camera
./mineworld-slice --world    # the same, connected to worlds/social-cafe (needs cargo)
```

You start on the pavement west of the café. **The second shop you can enter is The Flower Room**,
the next building east of the café, with the maroon front and the open door at its left end.

**Scope change, recorded.** `VISUAL_SLICE.md` §4 now **requires** one second enterable building
(new §4.1), on the operator's request that shops be enterable and the standing rule *a smaller
world with complete interiors over a larger fake town*. The amendment is in the shared target, so a
resumed Unreal track builds the same second interior and A/B stays like-for-like.

**Frames** — `clients/3d-spike/shots/slice/preview/`, from the running client at 1600×900:

| | Frame |
| --- | --- |
| **side by side, reference / before / after** | `sbs_03_cafe_frontage.jpg` (at `03`'s framing), `sbs_05_main_street.jpg` (at `05`'s framing) |
| through the glass, reference / after | `sbs_03_interior_through_glass.jpg` |
| the second building | `10_second_building_door.jpg` (from the pavement, the café beside it), `10b_second_building_interior.jpg`, `10c_second_building_looking_out.jpg` |
| re-captured | `01`–`08` as before |

**What changed, as facts against the references.**

| | Reference | Before (2026-10-01) | Now |
| --- | --- | --- | --- |
| sun on the frontage | `03`: low sun raking across the shopfront, hard shadows | even soft light, no cast-shadow pattern | the beam meets the café façade at 39°; the cornice, lamp brackets, hanging sign and baskets cast hard shadows onto the fascia and masonry |
| the room through the glass | `03`: dim amber, darker than the street, dark timber shelving to the ceiling | pale, evenly lit plaster; brighter than the pavement (0.34 against 0.24) | dark timber shelving floor to ceiling on the west wall, the wall `03`'s framing sees; browner plaster, darker ceiling and floor; at the door the room is darker than the street (0.116 against 0.175) |
| paving | `03`/`05`: grey-buff setts, dark joints; `05`'s road is setts too | warm beige oblong slabs, joints invisible at `03`'s framing; asphalt road | grey granite setts with soil joints, ~0.18 m courses, on both pavements and the carriageway |
| planting | `03`: two hanging baskets, ivy on the pier, a flowering planter box | one basket, one climber, one potted plant | two baskets (a large one on the right pier above the slate board), ivy on both piers, window boxes under all three first-floor windows, flowering planter boxes |
| window lettering | `03`: crisp cream upright serif, centred | thin white script, barely legible | cream serif, centred, opaque; all four words read at 1600×900 |
| rug | — | one flat colour | a woven flat-weave pattern, generated in-engine |
| roofs | — | — | **a defect found and fixed:** every pitched roof had its ridge tile turned front-to-back, a dark bar over Maple & Co. at `05`'s framing |

**Measured** (`--threshold`, `--drive`, `--world --link`):

```text
café threshold    pavement 0.175 -> doorway 0.116 -> 2 m in 0.109 -> deep 0.195; worst step x1.79 (limit x3)
florist threshold outside 0.188 -> doorway 0.274 -> 2 m in 0.332 -> deep 0.373; worst step x1.46
                  both: looking out through the glass 0.00 % clipped
florist walk      in through the door on foot, a loop round the room closing within 0.12 m, out again
connected         café -> street -> café through the door on foot, with S6's `move`:
                  50 moves, 50 accepted, 0 refused, through walking, jogging and two jumps
```

**Known misses, largest first, as facts.**

1. **Through `03`'s glass the pendants glow and a barista stands at a loaded pastry case by the
   window.** Through the candidate's, the pendants read as grey metal shades with no visible bulb
   at this distance, nobody stands behind the counter, and the pastry case is at the back, about
   10 m from the glass. A pale haze band lies across the lower half of the pane.
2. **Bistro furniture.** `03`: black metal frames with dark, weathered slats. Candidate: bright
   honey-orange slats, the most saturated object in the frame.
3. **Sky.** `03` and `05`: saturated blue with white cumulus. Candidate: pale grey-blue, cloudless.
4. **Fascia contrast.** `03`: cream lettering on a deep green board, high contrast. Candidate: in
   full sun, the board washes to a light sage and "The Daily Bean" loses contrast.
5. **Setts size.** `03`'s setts are larger and squarer, about 0.25–0.30 m, with tight light joints.
   The candidate's are about 0.18 m, rounder, with dark soil joints, a cobbled rather than paved
   look.
6. **Shadow on the paving.** `03` has a strong dappled tree shadow across the pavement in front of
   the café. The candidate's pavement there is in unbroken sun.
7. **`05`'s massing is not this street's.** `05` has a corner café with a chamfered angle and an
   awning on a 3–5-storey brick street. The candidate has a straight terrace of 2–3-storey
   limestone and render. That was the accepted direction; it is stated so it is not mistaken for
   something this round changed.
8. **A-board scale** is unchanged: 1.51 m against §3.4's 0.90–1.15 m.
9. **The florist is brighter than the street** (0.37 deep inside against 0.19 outside). That is by
   design, a florist's cool daylight, but it is the opposite of the café's relationship.
10. **Connected only:** the World Pack's café is the mirror of the slice's, with its door east of
    its people. With the doorway aligned, `alice` and `bob` are drawn just west of the café wall.
    Recorded for a World Pack decision (`pr-01a-slice.md` §7b).

**Questions for the operator.**

1. Does The Flower Room read as a shop you can walk into from the pavement, and is its interior
   complete enough to keep?
2. Is the room through the café glass now dark and warm enough? Should the next round put a
   barista and a pastry case at the window, as `03` does?
3. Of the misses above, which comes first: the glass (1), the furniture colour (2), or the sky (3)?

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
