# Human review queue

**Only large integrated milestones belong here.** Not individual PRs, structs, components, events,
dependency choices or refactors — the pi-agent owns routine engineering review. A milestone earns
a place here when a coherent capability can be *experienced* as a whole.

Engineering progress is tracked separately in [`MVP_STATUS.md`](MVP_STATUS.md). A milestone parked
here does not block anything else.

**Updated:** 2026-10-07 (Milestones B and C accepted by the operator)

---

## Framework milestones

| | Milestone | Demonstrates | State |
| --- | --- | --- | --- |
| **A** | Runnable world runtime | load Social Café → server → two clients + an agent → cause a change → all observe it | ✅ **complete 2026-09-27** — `AC-15` holds, 259 tests · restart/persistence is S5 and moves to **B** |
| **B** | Persistent people and social life | Alice and Bob persist, know each other, share an activity, and survive a restart with their history | ✅ **accepted by the operator 2026-10-07** — PR 10b, `tools/cli/tests/milestone_b.rs`, run by the operator on `main` and passing |
| **C** | Objects and everyday economy | Market Town: work → earn → buy → inventory changes → another client sees it → persists | ✅ **accepted by the operator 2026-10-07** — PR 11f, `tools/cli/tests/milestone_c.rs` (with `ac1_composability`, `market_town` and a 30-day `run`), run by the operator on `main` and passing |
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

**Milestone B, and how to see it for yourself.** Twelve people live a month in the town, headless.
They talk, invite each other for coffee or a walk, and come to know each other. Alice's life is then
read back out of the log:

```sh
mineworld run worlds/social-cafe --headless --seed 7 --days 30 --save /tmp/cafe
mineworld biography worlds/social-cafe --save /tmp/cafe --person alice
mineworld biography worlds/social-cafe --save /tmp/cafe --person bob
mineworld server worlds/social-cafe --save /tmp/cafe     # then join the `alice` seat with a client
```

What to look at:
- On day 1, Alice and Bob each `became-acquainted` with the other. On day 1 they share a
  `group-activity` (event `#98`). By day 3 a `relationship-changed` shows them growing closer.
- Every line of the biography carries the event id it came from.
- Kill the `run` with Ctrl-C or SIGKILL partway and run the same command again: the world finishes
  identically, and so do both biographies.
- Alice's own observation in the server shows her `acquaintances`: how she regards Bob, and nobody
  else's view.

The test performs all of this with real SIGKILLs of both `run` and `server`.

Watch for one known gap: relationships never decay. After about two months every pair is as close as
it gets, and no more relationship changes happen. The 300-day test prints that per month.

**Milestone C, and how to see it for yourself.** Market Town is Social Café plus six installed System
Packs and configuration. Alice works the café counter in the mornings and is paid by the hour; the
café restocks from her shift; everyone buys, eats, drinks and gives. Two days, saved, then hosted:

```sh
mineworld run worlds/market-town --headless --seed 7 --days 2 --save /tmp/market
mineworld inspect /tmp/market --last 0                         # every cause resolves
mineworld server worlds/market-town --save /tmp/market         # then join `alice` and `bob`
```

What to look at:
- In the save, on day 1: Alice is `hired`; her shift starts at 05:30 while she is still walking
  over, she arrives at the café at 07:15, and at 14:00 `shift-ended`, `wage-due` and the
  `money-transferred` that pays her follow.
- In the server, walk Alice and Bob into the café. Alice is offered a `buy` per thing on the shelf,
  each a complete request; submit one as it is offered. Her own `wallet` falls by the price, and her
  `holdings` gain one.
- Bob, in the café, sees the shop's listing: the stock of what Alice bought is one lower. He is never
  shown Alice's wallet or what she carries.
- Kill the server and start the same command again: the same world, the same revision, the same
  wallet, holdings and shelf.

The test (`tools/cli/tests/milestone_c.rs`) does all of this with real sockets and a real SIGKILL.
Its sibling tests prove the rest of S9: `tests/acceptance/tests/ac1_composability.rs` (AC-1 as
`ARC-35` measures it, within `ARC-33`'s static-linking boundary — installing a pack is a directory,
two lines in `systems/installed` and a rebuild), `tools/cli/tests/market_town.rs` (the market lives
300 days) and `tools/cli/tests/market_composition.rs` (each market pack removable).

Watch for the known limits: people without a job live on an endowment sized for 300 days (L-13);
items and organizations have no names, so a listing shows ids (F-41); every unavailable buy says
`TargetUnavailable`, whatever the reason (F-48).

## Default-style milestones — taste, and the operator decides

These are candidates until the operator says otherwise (`ARC-11`). An agent may build and
recommend; it may not declare something the default look.

Named per `ARC-20`, because with two 3D tracks running (`ARC-18`) "the 3D character" no longer
identifies one thing.

| | Milestone | State |
| --- | --- | --- |
| **VIS-2D-1** | Playable 2D default scene with an enterable interior | 🚧 candidate in progress |
| **VIS-3D-GODOT-1** | Reference-matched character in Godot | ✅ **route D+ accepted by the operator as the interim standard, 2026-10-07**; the clipping and head-sway defects flagged after it are fixed (see below). Not a final acceptance: refinement continues later. Before it: ❌ FAILED by the operator, 2026-10-06 (the CharMorph candidate, kept as pipeline validation only, `VISUAL_FIDELITY.md` §9.2) |
| **VIS-3D-GODOT-2** | Integrated Godot slice: character, street, enterable building, interior, lighting, cameras | ✅ **ACCEPTED by the operator, 2026-10-07, after playing it**: the operator ran the combined session on `main` (standalone `./mineworld-slice` and connected `--world`) and reported every checklist point as passing — the character, hand and pack without clipping, a steady head when running, the doorway, the terrace, real names, the too-far refusal, the counter conversation and Alice in view. Earlier the same day it was accepted on screenshots. Before that, 🟡 READY FOR HUMAN VISUAL REVIEW (2026-10-06): the reference character in the slice, connected to the MVP town, talking to Alice at the counter; your three findings of 2026-10-06 fixed (terrace clipping, labels, dialogue); people now shown by the names the world discloses (S8 PR 10c, merged); the barista visible from the counter; **since 2026-10-07 with the route D+ character as the player**, one combined test session for slice and character; see below |
| **VIS-3D-UE5-1** | Unreal slice of equivalent scope | ⏸ **parked** — spike phase one done (`ARC-21`), operator paused the install |
| **VIS-3D-AB-1** | Godot vs Unreal side-by-side, same reference, same scope | ⏸ parked with `VIS-3D-UE5-1` |

### `VIS-3D-GODOT-1` — route D+, accepted as the interim standard 2026-10-07 (prepared as a preview the same day)

**Operator verdict, 2026-10-07**, given on eight screenshots: reference against game, street front
and rear, doorway, counter, hand detail, full body and rear, walking.

> 跟上一版本相比好多了，我们可以就先维持这个标准。以后再精雕细琢

*Gloss:* "Much better than the previous version. We can hold this as the standard for now, and
refine it later."

So route D+ is **accepted as the interim default character** (`ARC-11`: only the operator
accepts). After accepting, the operator added:

> 人的手和背包好像还是有点穿模

*Gloss:* "The person's hand and the backpack still seem to clip a little."

**Fixed on `vis/3d-human-pipeline` at `330f1fd`, by the character's session** (record §§8.7–8.9,
summarised at the end of this entry). Standing, the hand grips the strap above the hoodie's edge,
and it lets go while she moves. The pack's side panels follow the spine. The torso twists less
while she moves, and her head is steadied in the walk and the jog.

The other known misses below are **deferred refinements, not blockers**: the hair's tone and
locks, the painted eyes with no blink or expression, the hand, the pack's colour, and the tee's
first word. They are what "refine later" refers to, and this acceptance neither closes nor
relaxes them.

The text below is the preview as it was prepared, kept as the record of what was judged.

**As prepared: a preview (`VISUAL_FIDELITY.md` §9.1), not an acceptance request.** It was held for
the coordinator, who showed it to the operator with the other items. It asked: *is this the
right direction, and what is most wrong?*

The candidate is one Meshy generation of the reference, made game-ready (95,000 triangles, one
material) and rigged on our existing skeleton, in its own body slot: only the player uses it,
the townspeople keep the CharMorph body. Record:
[`references/CHARACTER_ROUTE_D_PLUS.md`](references/CHARACTER_ROUTE_D_PLUS.md) §8.

**Launch.**

```sh
./mineworld-3d             # walk around; the player is the candidate (third-person camera)
./mineworld-3d --portrait  # the review frames, written to clients/3d-spike/shots/
```

**Frames**, in `presentation/mineworld-default/3D/candidate/route_d/`:
`d3_side_by_side.jpg` (the reference chest-up beside the runtime three-quarter and front
portraits), `d4_full_and_rear.jpg`, `d5_walk.jpg`, `d6_grip_armpit_hairline_zoom.jpg`,
`d7_idle_loop.jpg`.

**Known misses, largest first.**

1. The hair is a darker, cooler brown with no caramel highlights, and its locks read as heavier
   sculpted ropes than the reference's soft wisps.
2. The eyes are painted: the look to her left comes from turning the head, and the eyes cannot
   move. No blink and no expressions either; the face has no lids or mouth that move.
3. She holds the strap only while standing. When she walks or jogs she lets go and both arms
   swing; the reference pose is a standing pose. Standing, the fist sits on the strap's lower
   padded end, a little higher on the chest than the reference's hand; at 3× the fingertips show
   dark creases and the thumb stands out.
4. The backpack is charcoal grey where the reference's is olive.
5. The tee's slogan has a garbled first word; the tee is greyer-white than oatmeal.
6. Light flecks along the hair's locks in the head close-up at 2×; not visible at the
   reference's framing.
7. While moving, the head faces the direction of travel and holds steady (within 4.2° in the
   jog); the look to her left is a standing pose only.

(Fixed before preparation, at the coordinator's review: the gripping hand had read as an open,
splayed hand with one finger pointing up, and she had looked straight at the camera, chin up.
Record §8.6.)

**Accepted by the operator as the interim standard, 2026-10-07** (*"跟上一版本相比好多了，我们可以就先维持这个
标准。以后再精雕细琢"*). Two defects the operator flagged afterwards are fixed (record §§8.7–8.9):
the hand and the pack clipped (*"人的手和背包好像还是有点穿模"*), and the head bobbed and swayed when
she ran (*"人物跑步的时候不要摇头晃脑的"*). Evidence: `d8_head_trace.png` (before, after, and the town
body as control), `d9_sweep_pack.jpg` (rear three-quarter walk and rear jog, eight phases each),
`d10_sweep_front.jpg` (front three-quarter walk and jog, eight phases each).

**Scorecard** (§6, full table in the record §8.5): overall identity `PASS`; face identity, hair
silhouette, hair colour, hoodie, tee, backpack, material quality and vibe `PARTIAL`; no hard-fail
category fails. No tear in any pose or in motion.

**Questions for the operator.**

1. Is this the right person at the right quality tier — enough to continue on this candidate
   rather than return to route C?
2. Which miss matters most: the hair colour and texture, the painted eyes, or the hand?
3. Is a character without blink or expression acceptable for now, or is that a blocker?

### `VIS-3D-GODOT-1` — FAILED, 2026-10-06 (operator verdict on the preview 3 state)

The operator compared the candidate with `04_character_closeup.png` side by side and called it
**far off, not a polish gap**: the reference's surface features were copied onto a low-fidelity
character, and it is neither the same person nor the same quality tier. The scorecard below is in
the operator's words (`VISUAL_FIDELITY.md` §6).

| Row | Reference | Candidate | Verdict |
| --- | --- | --- | --- |
| Face identity | small, soft, young face; large bright eyes; delicate, balanced features; a natural warm expression; light freckles | a wider, harder face; the eyes are blank, the expression wooden; the nose, mouth and chin are wrong | FAIL |
| Hair silhouette | a light, fluffy, tied-up updo with natural loose strands at the face | it takes "a high updo" literally: the mass is coarse, the locks messy, the volume is clumped rather than soft, and the face strands are unnatural | FAIL |
| Hoodie structure | a clear open burgundy zip hoodie: hood, zip, cords, cuffs | a red shell; the hoodie structure is weak and the cut is wrong | FAIL |
| T-shirt and graphic | an oatmeal tee with a natural mountain print | the print is present but reads as a crude texture | FAIL |
| Backpack | natural straps, sitting naturally on her | the straps do not relate naturally to the clothes | FAIL |
| Material quality | soft skin, warm light, clean materials, like a high-quality game promo image | rough skin texture, dry materials, plain lighting, hair like painted blocks: a low-quality real-time character | FAIL |
| Overall identity | — | not the same character | FAIL |
| Overall vibe | warm, relaxed, cute, everyday | a low-cost demo character: stiff and rough | FAIL |

**Consequence.**

- Work on this candidate as the default character stops.
- The parts it proved stay as pipeline validation: the character slot, the import, the rig, the
  retarget and the animation.
- `CHARACTER_ROUTE_ASSESSMENT.md` §1 already found that its route B (an agent doing artist methods
  in Blender) would not close the face. Preview 3 ran that route, and the operator's verdict shows
  it does not reach the quality tier either.

**What remains is a route decision for the operator.** Either route C (a commissioned artist on
the existing rig) or route E (an image-to-3D head as a wrap target, which needs a paid generator
account or a CUDA GPU) has to be chosen.

**Route E, run 2026-10-06/07: `FAILED`. Not a preview.** Full record and frames in
[`references/CHARACTER_ROUTE_E_EXPERIMENT.md`](references/CHARACTER_ROUTE_E_EXPERIMENT.md) §9;
evidence in `presentation/mineworld-default/3D/candidate/route_e/` (`e1`–`e11`).

- **TripoSR** and **TripoSG** (both MIT, both run on this Mac without CUDA and without any
  non-commercial background remover) stopped at the raw-mesh gate: relief-quality or closed-eyed
  faces, one of four proportions nearer the reference.
- **Meshy 7.1 (Pro, 60 credits)** passed the gate. Its head and its whole character read as the
  same person at the reference's quality tier (`e4`, `e6`) — but as one fused, unrigged,
  million-triangle statue: hair, hood and pack cannot move, the hand is welded to the strap, no face
  rig. Route D's prediction was wrong about appearance and right about structure.
- **The wrap** of Meshy's head onto the CharMorph head (three configurations; the last one clean:
  topology, UVs, eyes, rig and blink kept) moved eye size and face width toward the reference by
  measurement, and **did not change who she is** (`e8`–`e10`). Face identity `FAIL`; every other row
  unchanged and `FAIL`. The identity in the generated head lives in its painted layer and its hair,
  not in a few millimetres of shape.

**Operator decision now:** (1) route C with the Meshy whole character as the artist's likeness
target; or (2) one more bounded agent test outside route E's own design: bake Meshy's face colour
onto the wrapped CharMorph face (the surfaces are within ~2 mm) to test whether the painted
identity transfers. See the experiment doc §9.11.

### `VIS-3D-GODOT-1` — working state after preview 3, 2026-10-06 (superseded by the FAILED verdict above)

The primary session's four items on preview 3, worked at working resolution (620 × 900 per tile)
against the reference at its own framing. The frames in `candidate/` are regenerated from this
state (`side_by_side_chest.jpg`, `side_by_side_head.jpg`, `side_by_side_rear.jpg`, `p1`–`p8`).

| Item | Reference | Candidate now | Frame | |
| --- | --- | --- | --- | --- |
| 1. Hood structure (§5 hard fail) | hood stands up at both shoulders, bunched behind and around the neck | a rolled rim round the neck, standing up at both sides of it; from behind, a flap over the rucksack lid with a centre point and three large folds. Modelled and laid on the body, hoodie and bag by ray cast (`garments.build_hood`); the bag hangs 45 mm lower | `side_by_side_rear`, `p5`, `p1` | PASS: reads as a hood from behind |
| 2. Hair, front | soft, loose volume; a visible messy bun; caramel lights | strands from eight CC0 OwlishMedia maps; the front and side lift is about doubled; the knot is a quarter larger; back-facing cards are lit correctly (no dark stiff strands); crown 0.37/0.28/0.23 sRGB against 0.42/0.29/0.21 | `side_by_side_head` | improved, still a weak PASS: the swept front is still one continuous mass, smoother than the reference's separate locks |
| 2. Hair, rear | — (not shown by the reference) | card edges show from behind, and the low sun turns the hair orange, as it does the hoodie | `p8` | open |
| 3. Rucksack | grey-green canvas, straps, a lid | seams round the lid and pocket and down both sides; two compression straps with buckles; a greyer canvas, but still olive in the rear frame's low sun | `p8`, `p5` | PASS on structure; the colour depends on the light |
| 4. Tee | cream/oatmeal 0.75/0.62/0.54 on the upper chest; the print high, mountains as wide as the text | oatmeal albedo `#CBB9A0` (was `#D9D2C4`, which rendered 0.70/0.71/0.72); the print is cropped to the range, in the reference's proportions, with its top 82 px under the crew band | `side_by_side_chest` | PASS |

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

### `VIS-3D-GODOT-2` — ACCEPTED by the operator after the interactive test (2026-10-07)

**Verdict, 2026-10-07.** The operator judged the slice from its screenshots, shown together with
route D+'s (street front and rear, doorway, counter, walking), in the verdict quoted under
`VIS-3D-GODOT-1` above: 跟上一版本相比好多了，我们可以就先维持这个标准。以后再精雕细琢 ("much
better than the previous version; hold this as the standard for now, refine later"). **The
operator has not yet run the build.** So this is acceptance on screenshots only, and the
interactive check — the combined session below, walked by the operator — is still pending. Nothing
here claims more than that.

The character's hand-and-pack clipping, reported after the acceptance, is fixed on
`vis/3d-human-pipeline` at `330f1fd`, and that fix is merged here. The street, doorway and counter
frames are re-captured on it (the `v2` frames, below). The slice's original claim of "no clipping"
(further below) was about the character against the scene — jambs, counter, case, bench. It never
covered the hand against the pack.

#### `v2` frames, on the fix (`330f1fd` merged), 2026-10-07

At 1600×900, in `clients/3d-spike/shots/slice/review/` (copies of the first six in
`mineworld-demos/character-preview/`):

| Frame | What it shows, looked at |
| --- | --- |
| `v2_street_walk.jpg` | Walking toward the camera: she lets go of the strap, both arms swing, the head is level; nothing of the hand passes into the pack or the hoodie |
| `v2_street_jog.jpg` | Jogging toward the camera: the arms swing wide and clear of the body, the head upright |
| `v2_rear_jog.jpg` | Jogging away: the pack sits on her back; at 3× its side panels lie on the hoodie with no hoodie showing through them, and both arms pass outside it |
| `v2_doorway.jpg` | On the café threshold: shoulders, pack and hood inside both jambs |
| `v2_counter.jpg` | Beside the counter: she and Alice Moreau in one frame, both caption lines up |
| `v2_hand_standing.jpg` | Standing, ×4: the hand at chest height on the strap's line, fingers curled; nothing passes into the hoodie. The strap itself runs just outboard under the hoodie's edge, so the hand reads as raised to the strap rather than wrapped round it — a refinement, not clipping |
| `v2_run_into_townsperson.jpg` | **The known gap.** Jogging west into the standing townsperson at (−8.5, −5.95), she passes **through** her: the body centres came within 0.10 m and the bodies interpenetrate. Nothing makes bodies collide yet — the street's figures are drawn, not simulated, and the player's capsule meets only world geometry. **That is the physics step's to address, and it is not fixed here** |

The entry as it was submitted follows.

#### Submitted: READY FOR HUMAN VISUAL REVIEW, 2026-10-06; with the route D+ character, 2026-10-07

**This is the acceptance request** for the integrated slice: the street, the café you walk into, The
Flower Room, the light, movement, the three cameras, the connection to the world, and **the
reference character in the slot**. Since 2026-10-07 that character is **route D+**, the new default
character (`vis/3d-human-pipeline` at `28710b1`, merged, not copied; ledger
`docs/references/CHARACTER_ROUTE_D_PLUS.md`, entry *route D+ preview, PREPARED* above). The entry
asks *is this acceptable as the integrated Godot slice?* Only you can answer that (`ARC-11`).
Accepting it does not accept the character itself: that is still `VIS-3D-GODOT-1`'s question, above.

#### One session: the slice and the character together

Branch `vis/3d-godot-2-environment`, from the repository root, Godot 4.7 on `PATH`.

1. **`./mineworld-slice`** (standalone). Press **V** to cycle the cameras: first person shows no
   body and no hair; third person rear and front show her. Look at her **scale** against the doors
   and the people on the street, and at the **hand on the rucksack strap**. Walk into The Daily Bean
   and **through the door** in third person: the pack and hood should stay clear of the jambs and
   the chairs. Then look at the **seated woman on the café terrace**: she sits on a chair, not
   inside the table.
2. **`./mineworld-slice --world`** (connected; needs cargo). Every person is labelled by their
   name. Walk out onto the pavement and back in: no snapping back. From the door, face Alice
   Moreau and press **E**: *can't talk to Alice Moreau: too far away*. Walk to the counter. She
   stands in plain view behind it, with the espresso machine on the back bar. Press **E** again:
   two subtitle lines, *You: …* and *Alice Moreau: …*, readable for several seconds.
3. **The character itself** (`VIS-3D-GODOT-1`). Judge her face, hair and clothes against the
   reference with the route D+ entry's own frames and checklist, above. The slice frames below show
   her at street and café distance only.

Frames of her in the slice, in `clients/3d-spike/shots/slice/review/`: `char_street_rear.jpg`,
`char_street_front.jpg`, `char_cafe_rear.jpg`, `char_cafe_first_person.jpg` (no body, as
intended), `char_walking_pavement.jpg`, `04c_doorway_with_character.jpg` with
`char_doorway_detail.jpg`, `07_third_person_rear.jpg`, `08_third_person_front.jpg`,
`char_counter_talking_to_alice.jpg` (beside the counter, both caption lines up) and
`char_strap_hand_detail.jpg`.

What I saw in them, as facts. No clipping against the door jambs, the counter, the pastry case or
the window bench. Her stature reads correctly against the 2.1 m doors and the townspeople. In first
person nothing of her is drawn: `Player` hides the whole body there, so the head and hair cannot
reach the near plane. Feet are planted mid-stride. **The hand on the strap is a closed fist at chest
height, in line with the strap.** But the strap runs under the hoodie's open edge, so at street
distance it reads as a fist held near the strap rather than a hand clearly gripping it
(`char_strap_hand_detail.jpg`, ×4). That is the character's track to judge.

Checks on this build: `--character` all pass (the slot reports body `meshy_d.glb`; idle and walk
animate); `--drive` all pass; `--world --link` all pass (50 moves, 50 accepted, 0 refused; door
rejected too far away; counter accepted, reply heard; 60 s street watch in 60.0 s of wall time);
`--world --conversation` passes, with no id on screen.

One `--world --link` run stalled in its 60 s street watch for more than 17 minutes before being
stopped. It did not recur: two 10 s runs ran in real time on both the route D+ body and the town
body (`--town-body`), and a full 60 s run took 60.0 s of wall time. That run fell in the window when
other sessions also stalled, but the cause was not found. The watch now prints simulated against
wall seconds, so a recurrence will show.

The world it connects to is the MVP town merged from `main` (`9ab5e62`, through S8 PR 10c), whose
café is authored to this slice's layout (10a) and whose people carry names (10c): Alice Moreau
stands behind the counter you see.

#### Your findings of 2026-10-06, and what changed

| You saw | Cause, located | Fix | Evidence |
| --- | --- | --- | --- |
| Standalone: a person outside the shop clips through a table | The seated townsperson on the café terrace (`SliceStreetscape._people`, figure `c`) was placed at the middle terrace table set's own coordinate (6.65, −6.30), which is the **table's centre**: her torso came up through the table top, with no chair under her. Identified by `--pick` on the frame as `outdoor_table_chair_set_01_table` and her `Body`/`Hoodie` meshes at the same spot | She sits on the set's east chair, facing the table. The seat and facing are read from the placed set's chair and table meshes, not typed beside it (`_chair_seat`) | `review/fix1_terrace_before.jpg` (before, ×3 crop), `fix1_terrace_after.jpg` (after, same viewpoint, ×3), `fix1_terrace_from_pavement.jpg` (the pavement east of the café, looking west along the frontage), `fix1_terrace_across_table.jpg` (from the pavement across her table); `sbs_03_cafe_frontage.jpg` re-captured |
| Character names are not shown, only things like `regular` | **The world has no names yet**: `worlds/social-cafe/people/*.yaml` author tags, not names, so the client had only tags to show | Every figure label, HUD line and conversation line now comes from one function, `SliceLink.display_label`. **Since S8 PR 10c (merged at `9ab5e62`) it shows the name the world discloses**: the `naming` pack's `display-name`, read through the protocol module's own `MineWorldObservation.display_name` (Alice Moreau, Bob Achterberg, Wes Calloway). Where the world discloses no name, it falls back to the role: the first tag that names a role, capitalised (`Barista`; `staff`/`public` are skipped). You are `You`. It never shows an entity id: ids, tokens and raw results go to the log only | `fix3_conversation_from_door.jpg`, `fix3_conversation_at_counter.jpg` |
| The dialogue doesn't look like natural language | The heard line was printed as `<speaker id> (<tags>) said: <utterance JSON-encoded>`, which quoted and backslash-escaped the words | A heard line is now a conversation line, `Alice Moreau: words`, with the words as plain text. What you say is shown the same way (`You: Hello! A coffee, please.`) once the world accepts it. Lines appear as subtitles at the bottom centre, wrapped, the last three stacked, each held 6–14 s depending on length | `fix3_conversation_at_counter.jpg`, `fix3_conversation_caption_detail.jpg` |

**What this does not fix: the wording of Alice's reply.** "I remember you. You said "Hello! A coffee,
please.". You are the first person to speak to me here." (and, once others have spoken, "Earlier,
<name> said …") is the fixed template of the town's **rule controller**. The client now shows it
cleanly, but cannot make it natural. Since S8 PR 10c the template names people rather than
numbering them. Natural dialogue needs a language-model controller, which is Milestone D.

#### Follow-up of 2026-10-07

| Item | Cause, located | Fix | Evidence |
| --- | --- | --- | --- |
| At the counter the barista was mostly hidden | The espresso machine and grinder stood on the counter at 6.0 m along, their tops at 1.96 m, above a customer's eye (≈1.89 m). They sat squarely between the customer side of the counter and where the world stands Alice (`alice.yaml`: 6.0 m along, 8.0 m in). Her spot is **not** inside the counter: in the scene it lies between the counter's back face and the back worktop. So only the client's props needed to move | The machine and grinder moved to the back worktop behind her, facing the room. Nothing taller than the till stands on the counter in front of her. Her world anchor is unchanged | `fix3_conversation_at_counter.jpg`, `fix4_barista_visible_detail.jpg`; `05`, `06`, `09`, `09b` re-captured |
| One run put her behind the pastry case, and the talk from the door went to different people in different runs | **Not the world.** Three runs printed every person and the starting body at identical positions, yet the door talk went to Alice, to Bob, and to nobody. The capture runner turns the body by setting its yaw and sends no mouse motion, but the player accepted every mouse motion while scripted. The real cursor crossing the capture window therefore turned the view between the runner's turn and its read. That changed whom it faced and the path it walked to the counter | The slice's runner now ignores mouse look (`Player.ignore_mouse_look`; only the slice runner sets it). Three runs since: Alice targeted each time, and the body stops at the counter at the same point to the micrometre. This also explains the earlier run-to-run drift of first-person `--shots` framings | `--world --conversation` log (positions printed per run) |

**Launch.** Branch `vis/3d-godot-2-environment`, from the repository root, Godot 4.7 on `PATH`:

```sh
./mineworld-slice            # walk it
./mineworld-slice --world    # the same, connected to worlds/social-cafe (needs cargo)
```

Controls: **W A S D** move · mouse look · **Shift** jog · **Space** jump · **V** camera (first
person → third person rear → third person front) · **E** talk (connected only) · **Esc** release
the mouse.

#### Test checklist — run this, you should see that

1. **`./mineworld-slice`.** A window opens on the pavement west of the café, in low warm sun, a
   controls line along the bottom. You are in first person, so no body is drawn.
2. **Press V once.** A toast says *Third person — rear*. She stands ahead of the camera: bun,
   open burgundy hoodie with its hood at the neck, cream tee, grey-green rucksack, one hand on its strap. Wait a few
   seconds: she breathes and her gaze drifts; she is not a statue.
3. **Hold W, then Shift+W, then Space.** She walks, then jogs, feet planted without sliding; Space
   is a person's hop (about 0.5 m), not a block-climb. Press V again for *Third person — front*
   and walk toward the camera.
4. **Walk east to The Daily Bean** (green front, door at its left end) **and through the open door
   on foot.** No fade, no loading. The room is warm and a little darker than the street. Cycle V
   through all three cameras inside: none goes through a wall, none loses her.
5. **Walk round the tables and back out of the door** onto the pavement.
6. **Walk on east to The Flower Room** (maroon front, next door, the open door at its left end) and
   go in. A florist's interior, lighter and cooler than the café.
7. **Quit, then `./mineworld-slice --world`.** You start just inside the café door, where the
   world puts the visitor. Figures stand in the room, labelled with the names the world discloses:
   `Alice Moreau` behind the counter, `Bob Achterberg` at it, and `Wes Calloway`, the other
   visitor. Walk out onto the pavement and back in: no snapping back, no refusals on screen (each
   crossing is the server's decision).
8. **Talk, from the door and then from the counter.** Just inside the door, face Alice and press
   **E**: a toast says `can't talk to Alice Moreau: too far away`. She is about 8 m away, and the
   world, not the client, says no. Walk up to the counter in front of her: she is in plain view,
   with the espresso machine on the back bar beside her. Press **E** again. Two subtitle lines
   appear at the bottom of the screen:
   `You: Hello! A coffee, please.`, then
   `Alice Moreau: I remember you. You said "Hello! A coffee, please.". You are the first person to speak to me here.`

The objective checks behind this, if you want them: `./mineworld-slice --drive` (walk-in, loop,
walls, cameras, jumps, The Flower Room), `--measure` (scale), `--threshold` (light at the door),
`--character` (she animates; every camera mode), `--world --link` (the connected round trip),
`--world --conversation` (windowed: talk from the door and at the counter, capturing the HUD you
see). Each ends with *all … checks pass*, or for `--conversation` *conversation on screen, no ids*.

#### Frames — `clients/3d-spike/shots/slice/review/`

From the running client, 1600×900, its normal lighting, the character in the slot:

| `VISUAL_SLICE.md` §12 | Frame |
| --- | --- |
| 1 street wide | `01_street_wide.jpg` |
| 2 approach to the café | `02_cafe_approach.jpg` |
| 3 café exterior | `03_cafe_exterior.jpg` |
| 4 doorway transition | `04_doorway_from_pavement.jpg`, `04b_doorway_from_inside.jpg`, `04c_doorway_with_character.jpg` |
| 5 interior wide | `05_interior_wide.jpg` |
| 6 interior with the character | `06_interior_with_character.jpg` |
| 7 third person rear | `07_third_person_rear.jpg` |
| 8 third person front | `08_third_person_front.jpg` |
| 9 character in environment, closer | `09_character_close.jpg` (1.9 m, the counter behind her); `09b_character_in_cafe.jpg` |
| 10 The Flower Room | `10_flower_room_door.jpg`, `10b_flower_room_interior.jpg` |
| **side by side** | `sbs_03_cafe_frontage.jpg` (`03` / runtime at `03`'s framing, her on the pavement where `03` has its walker), `sbs_05_main_street.jpg` (`05` / runtime at `05`'s framing) |
| motion and cameras | `walk_cycle_strip.jpg` (eight frames of the walk, fixed camera), `camera_modes_standing_walking.jpg` (rear, front, first person; standing and walking; street then café) |
| your 2026-10-06 findings | `fix1_terrace_*.jpg` (the clipping, before and after), `fix3_conversation_*.jpg` (the talk with the world's names, as on your screen), `fix4_barista_visible_detail.jpg` (Alice from the counter) |

Views 1–5 and 10 are first person, at the framings §12 names, so the body is not drawn in them;
`04c`, `06`–`09` and the side-by-side of `03` have her in frame. The runtime half of
`sbs_03_cafe_frontage.jpg` and `09_character_close.jpg` are taken by a fixed camera in the running
scene, because the player's boom cannot reach those framings.

#### Measured on this build

```text
stature       (preview-3 body, superseded) 1.748 m sole to crown; the route D+ body's
              measurements are its own entry's (VIS-3D-GODOT-1, route D+)
animates      route D+ body, 2026-10-07: idle bones move in two consecutive 4.5 s windows
              (largest 13.5 mm, the strap hand's fingers); walk: a toe travels 0.80 m in 3 s
drive         walk-in, 13.3 m loop closing within 0.10 m, back wall / counter / glazing stop the
              body, three cameras indoors with 0.0000 m body movement on each switch, jumps 0.49 m,
              The Flower Room loop within 0.12 m -- all pass
threshold     café 0.184 -> 0.136 -> 0.125 -> 0.195, worst step x1.57 (limit x3), 0.00 % clipped
              looking out; Flower Room worst step x1.42 -- all pass
connected     the MVP town; places learned from the world (café 2, street 5). café -> street ->
              café on foot: 50 moves, 50 accepted, 0 refused; last position = the server's.
              talk from the door, 8.15 m: rejected too_far_away. On foot to the counter (19 moves,
              all accepted), 1.99 m: accepted, and Alice's reply heard
              (re-run on the final build, 2026-10-07, main 9ab5e62 merged: 50 moves,
              50 accepted, 0 refused; door rejected too_far_away; counter accepted, reply heard)
conversation  windowed, the player's HUD: from the door "can't talk to Alice Moreau: too far
              away"; at the counter (1.88 m) the reply drawn 0.3 s after E, both lines on
              screen, no entity id in the caption; three runs identical in who is targeted and
              where the body stops
drive         re-run on the final build: all drive checks pass
launch        plain launch and every mode exit 0; no script or resource errors
```

#### Known limitations, as facts

1. **Nobody walks the street in the connected slice.** `--world` hosts the town with
   `mineworld server`, whose controllers only answer; the town's walking controller runs only in
   `mineworld run`, which hosts no clients. Connected, you see the café's people and one
   `passerby` standing on the pavement west of the café, still for the 90 s measured. The town's
   other people are in places the slice does not draw and are never shown.
2. **If people did walk, three of the town's doorways open onto blank walls in the slice.** Each
   place's doorway, as the world discloses it, lands in this scene at: café — on the café door;
   workplace — 0.30 m from a house door on the south side; apartments — the north frontage 3.40 m
   west of Maple & Co.'s door; store — 3.62 m east of The Flower Room's door; park — the south
   frontage 3.29 m east of Lakeside Deli's door. A person going home, shopping or to the park
   would vanish into a wall there. The Flower Room is not the world's store; the world has no
   florist.
3. **Figures move in steps**: a perceived person is redrawn where each observation puts them,
   with no walking animation between.
4. **(Preview-3 body, superseded by route D+ on 2026-10-07.)** That body was 1.748 m, not the
   1.750 m asked for. The route D+ body's stature is recorded in its own entry.
5. **The idle is subtle by design**: on the route D+ body the largest idle movement is the strap
   hand's fingers (13.5 mm); the hips do not sway (0.0 mm).
6. **She is never seated in the slice**, so sitting with the hood and rucksack is untested here.
   The one seated figure on the street is a townsperson without either.
7. **First person draws no body**, on purpose (`player.gd`).
8. **On exit, Godot prints `7 RIDs of type "Texture" were leaked`.** No resource or object is named,
   it is present with GI off, and it has no effect while running. Not attributed.
9. **In `09_character_close.jpg` a pendant lamp hangs just above her bun** in the frame.
10. **The environment misses of the 2026-10-06 preview stand**, unchanged by this step: through
   `03`'s glass a barista and glowing pendants, ours a counter at the back (empty offline; with
   `--world`, Alice's figure stands behind it); honey-orange
   bistro slats against `03`'s dark weathered ones; a pale, cloudless sky; the fascia washes to
   sage in full sun; setts smaller and rounder than `03`'s; no tree shadow on the café's pavement;
   `05`'s corner massing is not this street's; the A-board is 1.51 m tall against 0.90–1.15 m;
   The Flower Room is brighter inside than the street.
11. **The character's own open items** are `VIS-3D-GODOT-1`'s (above): the face, deferred by you;
   card edges in the hair from behind.
12. **Alice's reply is a rule controller's fixed template**, shown verbatim, so it reads as a
   template. Natural dialogue is Milestone D (a language-model controller). The "Earlier, <name>
   said …" form was not seen on screen here: in a freshly hosted town nobody else has spoken to her.
13. **From the door, Alice's label overlaps Bob's**, and Bob stands in front of her, because the
   two stand in line from there. This follows from where the world puts them. Labels are drawn
   with depth, so a prop can cover one.
14. **A name label floats about 0.3 m above the head**: it is placed for a 1.72 m figure, and Alice's
   bun is taller.

#### Questions for you

1. Is this acceptable as the integrated Godot slice — street, café, second shop, light, movement,
   cameras, connection, with her in it?
2. Does she belong in it: does she read at the same scale and in the same light as the street and
   the room (`07`, `08`, `09`, `sbs_03`)?
3. If not yet: which single thing is most wrong?

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
| bicycle | `05`: a spoked bicycle against the kerb | — | **a defect found in review and fixed:** the slice's one bicycle, beside The Flower Room, had solid black disc wheels that hid its spokes and read as a toy. Now a tyre ring, a steel rim and twelve spokes, checked at 3× in `10_second_building_door.jpg` |

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
