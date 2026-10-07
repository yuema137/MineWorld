# PR 01a — the `VIS-3D-GODOT-2` environment slice

## DESIGN FROZEN

```text
Design revision       vis-3d-godot-2/pr-01a, sections 1-6
Approved by           operator brief of 2026-09-27 delivered to this session, which states the
                      scope, the two declarable states, the review package, the concurrency
                      boundary with vis/3d-human-pipeline, and the standing instruction to
                      "commit and push after each meaningful completed step"
Implementation base   main @ 5b99840, branch vis/3d-godot-2-environment (own worktree)
Execution contract    pr-01a-contract.md
Lifecycle             IN EXECUTION
```

**Combined step/PR document.** Steps 02–09 of [`overall.md`](overall.md) are one PR executed as a
sequence of semantic commits. Step 01 (`docs/VISUAL_SLICE.md`) is the first commit of it.

---

## 1. Goal and scope

Build the slice specified in [`docs/VISUAL_SLICE.md`](../../../docs/VISUAL_SLICE.md) inside the
existing `clients/3d-spike` Godot project, as a new scene with its own launcher, and reach
`VIS-3D-GODOT-2 ENVIRONMENT: TECHNICALLY READY` with recorded evidence.

**In scope:** the slice scene and its construction code; a material and lighting pass; the café
interior; the protocol binding and semantic `Place` identity; the character slot; the capture and
measurement harness; asset acquisition with `DEP-8` records; the review package.

**Out of scope:** everything in [`overall.md`](overall.md) §3.

## 2. Invariants

1. **No file owned by `vis/3d-human-pipeline` is edited.** Named in `overall.md` §3.
2. **The existing promenade scene keeps working.** `./mineworld-3d`, `--drive` and `--shots`
   behave as they do on `main`.
3. **One world unit is one metre**, and no node carries a non-uniform import scale
   (`DEP-8`, coherence procedure step 1).
4. **No world rule is evaluated in the client** (`ENGINEERING_RULES.md` §§4, 7–9).
5. **Identities stay strings** (`clients/protocol/ADOPTION.md` §3.1).
6. **No asset enters the repository without a recorded relicensing answer** (`DEP-8` as amended).
7. **No fidelity claim without a pixel comparison at the reference's own framing** (`ARC-17` §6),
   and none of the phrases prohibited by `VISUAL_FIDELITY.md` §6.
8. **`ACCEPTED` is never written by this branch** (`ARC-11`).

## 3. Source audit

Read in this session before any edit:

| Path | Finding |
| --- | --- |
| `clients/3d-spike/project.godot` | Forward+, 1600×900, 2048 directional shadows, SSAA; no SDFGI, no lightmapper settings, no reflection probes |
| `clients/3d-spike/scripts/main.gd` | the `ARC-13` rig: one HDRI sky at 0.80, one sun at 17°/2.9 energy, a cool fill, AgX at 1.06, SSAO on, **SSIL off**, glow, depth fog, saturation 1.16 |
| `clients/3d-spike/scripts/mats.gd` | world-triplanar PBR from Poly Haven `diff`/`nor_gl`/`arm` triples; `pbr()`, `paint()`, `glass()`, `clear_glass()`, `emissive()`, `card()`; 11 named materials |
| `clients/3d-spike/scripts/build.gd` | `box` `slab` `cyl` `prism` `sphere` `ground` `card` `label` `blocker` `box_blocker`; collision is opt-in per call |
| `clients/3d-spike/scripts/interior.gd` | an enterable shell already exists: pieced front wall, cafe and shop fit-outs, local omni lights at energy 6.5. **Ceiling is a blank plane; the back wall is bare; there is no display case, no glassware, no hand-scale object.** This is the largest single gap |
| `clients/3d-spike/scripts/player.gd`, `camera_rig.gd` | one controller, three cameras, boom with one collision ray; mode switch preserves state |
| `clients/3d-spike/scripts/shots.gd` | 24 scripted viewpoints and a headless `--drive`; capture writes `res://shots` |
| `clients/3d-spike/screenshots/21_street_golden.jpg`, `17_cafe_counter.jpg` | the measured baseline. See §4 |
| `clients/protocol/mineworld/` + `ADOPTION.md` | `MineWorldClient`, `MineWorldObservation`, `MineWorldSpace`; copy or symlink into the project; four rules a client must not break |
| `mineworld-3d` | the launcher; rebuilds the class cache when a script is newer |

### 3.1 The measured baseline, reference fact against candidate fact

Largest miss first, from `21_street_golden.jpg` against `05_main_street_golden_hour.png` and
`17_cafe_counter.jpg` against the interior visible in `03_cafe_frontage.png`:

```text
INTERIOR CONTENT  reference: a back wall of full-height timber shelving carrying jars, bottles,
  ceramics and plants; a glass pastry case with two loaded shelves; a framed chalkboard menu of
  handwritten lines; six pendants at three heights; potted plants on the counter.
  candidate: a bare brick wall, one green box counter, one grey box, a 12-character label
  reading TODAY, three cone pendants, and no object smaller than a chair.

OPENINGS          reference: every window has a reveal, a sill and a head, and the shopfront has
  pilasters, mullions, a transom, a moulded cornice and a dentil course.
  candidate: flat rectangles cut in a tiled brick plane, a flat green fascia with no cornice and
  no lettering at all.

GROUND            reference: pale grey-buff setts roughly 0.30 x 0.20 m in stretcher courses with
  a granite kerb, per-unit tonal variation and joint shadow.
  candidate: one flat orange plane with a 1.8 m concrete tile and no kerb read.

CEILING           reference: exposed warm timber planking running front to back.
  candidate: an untextured pale grey plane.

VEGETATION        reference: street trees with layered foliage, window boxes, hanging baskets,
  trailing ivy, planters at three sizes.
  candidate: four card-blob trees and no frontage planting.

BACKDROP          reference: a forested ridge and a church spire in warm haze.
  candidate: pale angular polygonal shards.
```

## 4. Acceptance

The nine objective gates of the operator brief, each with evidence:

```text
G1  the slice launches reliably, from a clean checkout, with no missing resource
G2  the cafe is physically enterable on foot
G3  the interior is fully navigable; VISUAL_SLICE.md §6.3 passes
G4  collision is correct: no pass-through, no fall-through, no escape but the doorway
G5  real materials: PBR maps, correct texel density, no untextured architectural surface
G6  the lighting pipeline is active, and the GI choice is recorded with measurements
G7  reflections and environment lighting work indoors and out
G8  scale is objectively checked against VISUAL_SLICE.md §3
G9  the three camera modes work indoors and out and preserve state across a switch
G10 performance is reasonable, measured
G11 asset provenance and licences recorded, with the DEP-8 relicensing question answered
G12 MineWorld integration works: a real intent round-trip against the real server
G13 the character slot is stable
```

## 5. Commit plan

Each commit: implementation, deterministic validation, LLM logic review — separately checkable.
Each is pushed on completion.

### C1 — the slice specification
- [x] Implementation: `docs/VISUAL_SLICE.md`, engine-neutral; `overall.md`; this document; contract.
- [x] Validation: grep the document for engine vocabulary. Evidence in §7.
- [x] Review: every operator-named scope element appears as a requirement; nothing Godot-specific.

### C2 — scene skeleton, launcher, capture and measurement harness
- [ ] Implementation: `scripts/slice/*.gd`, `scenes/slice.tscn`, root launcher, capture views,
      a scale probe that prints measured dimensions, a luminance probe.
- [ ] Validation: launch from a clean checkout; walk the street end to end under `--drive`.
- [ ] Review: no shared file edited beyond the launcher; the promenade scene still builds.

### C3 — café exterior architecture
- [ ] Implementation: profiled shopfront, signage, masonry with real openings, awning, brackets.
- [ ] Validation: measured against `VISUAL_SLICE.md` §3.2; captured and compared to
      `03_cafe_frontage.png` at that reference's framing.
- [ ] Review: massing and silhouette checked against `VISUAL_FIDELITY.md` §5 hard-fails.

### C4 — street, secondary façades, ground, furniture, planting
- [ ] Implementation: `VISUAL_SLICE.md` §4 in full.
- [ ] Validation: no repeated façade unit; frontage length and widths measured.
- [ ] Review: the out-of-scope list in §4 has not been crossed.

### C5 — material authority and the lighting pipeline
- [ ] Implementation: material re-authority pass, texel density, sky, exposure, probes, GI.
- [ ] Validation: the GI comparison, with frames and numbers for each candidate technique.
- [ ] Review: no technique used merely because it exists; the decision is recorded.

### C6 — the café interior
- [ ] Implementation: `VISUAL_SLICE.md` §6 in full, with collision.
- [ ] Validation: the walk-in loop, clearances, three object sizes, counted.
- [ ] Review: nothing in the room is a billboard or a projected image.

### C7 — threshold verification
- [ ] Implementation: the four-sample measurement path and its report.
- [ ] Validation: `VISUAL_SLICE.md` §7.1, as numbers.
- [ ] Review: thresholds stated before the run, not chosen to fit the result.

### C8 — runtime integration and the character slot
- [ ] Implementation: protocol module adopted, `Place` identity, intent path, character slot.
- [ ] Validation: an intent round-trip against the running server.
- [ ] Review: `ADOPTION.md` §3's four rules, checked one by one.

### C9 — the review package
- [ ] Implementation: the §12 views, `ASSETS`/licence records, limitations, queue entry.
- [ ] Validation: every view captured from the running client at its normal settings.
- [ ] Review: it is a review package and not an engineering log (`ARC-20` §3).

## 6. Test ownership

| Failure class | Owner |
| --- | --- |
| a script that does not parse, a missing resource, a broken path | launch of the slice; the class cache build |
| collision, navigability, camera state across a switch, walking speed | the scripted drive run |
| scale | the scale probe, printed and recorded |
| exposure across the threshold, interior legibility, clipped exterior | the luminance probe, printed and recorded |
| visual fidelity to a reference | pixel comparison at the reference's framing; never a test |
| taste | **the operator.** Not testable and not the agent's (`ARC-11`) |

Rust `cargo` layers are **N/A** for this PR until C8 touches a Rust path; C8 runs the server and
therefore uses the existing workspace build.

## 7. Evidence log

### C1 — 2026-09-27

```text
$ grep -niE 'godot|unreal|forward\+|lumen|nanite|sdfgi|lightmapgi|ssao|ssil|reflectionprobe|
    volumetric|shader|tonemap|node|gdscript|blueprint|\.tscn|\.uasset' docs/VISUAL_SLICE.md
```

Result recorded with the commit. Occurrences of `Godot` and `Unreal` are permitted only where the
document names the two **tracks** (`VIS-3D-GODOT-2`, `VIS-3D-UE5-1`) rather than a technique; any
other hit is a defect in the document.

### Recovery session, 2026-09-30 — first run, repair, preview

Taken over from a stalled session at `fa021cd`. `origin/main` was merged first (`b651208`). Every
item below is a commit on the branch, carrying its own evidence.

```text
assets      fetch_slice_assets.sh did not parse (stale duplicate list); fixed, made to fail loudly
            45 assets checked one by one: check_slice_provenance.py 0 failed, every atlas by eye
            EXCLUDED on licence: CashRegister_01 (Bank of Canada note), hanging_picture_frame_02
            (painting signed "Celine F", not a credited author)
first run   launcher passed --slice--drive, so no scripted mode had ever run
drive       door blocked by a pot; glass walkable; Place volumes floored at foot height;
            instrument defects: timed loop legs, distance-proxy wall test, wedged start
measure     constants only -> added rays against built colliders + occupant mesh; found an
            unglazed bay beside the door, a 0.3 m sliver, non-colliding jambs
render      boughs/bicycle frames at world origin (look_at_from_position with local args);
            foliage cards uncut (JPG base colour has no alpha); every window, door and shop
            window buried inside solid walls (wall_with_holes); window lettering faced inward;
            three views started inside furniture (capture now warns)
exit        510 leaked instances from the Props template cache; freed from SliceMain
```

Final evidence, all on the same build: `--drive` all pass; `--measure` all pass, with the
frontage open only at the door; `--threshold` four PASS (worst step ×1.38, 0.00 % clipped
looking out); `--perf` 10.9–17.6 ms; plain launch clean. The preview package is in
`docs/HUMAN_REVIEW_QUEUE.md`.

## 7a. C8 design — the slice as a MineWorld presentation (written before implementing)

**Scope:** inside C8 as frozen in §5. No contract, kernel, system, server or World Pack change.

```text
module     clients/3d-spike/mineworld -> ../protocol/mineworld (symlink, ADOPTION.md §1). No copy,
           no private transport, no second axis conversion: MineWorldSpace only.
adapter    scripts/slice/slice_link.gd -- one Node. The ONLY slice file that names an action type.
           const MOVE_ACTION := "arrive"   (presence pack on main @ a594164). S6 retires `arrive`
           for `move`; that change is this one constant plus the payload builder beside it.
launch     ./mineworld-slice --server=host:port [--seat=visitor]. Without --server the slice runs
           offline and says so on the HUD; nothing else changes.
world      worlds/social-cafe, served by `mineworld server` exactly as clients/protocol/run.sh does.
```

**Spatial binding — one constant, stated as a discrepancy.** The pack's café is a frame in
millimetres with no authored geometry; its people stand at x 1.2–4.6 m, y 0.2–4.4 m, with the
visitor "at the door" at (4.6, 0.2). The slice's room is 8.32 m × 10.32 m with its door 1.61 m
from the west wall. Since a metre stays a metre, no single origin puts the pack's door on the
slice's door while keeping the pack's people inside the room. The binding chosen is: **pack origin
= the room's inner front-west corner, pack +x = east, pack +y = into the room**, so that every
authored person stands inside the room. Recorded as a discrepancy between World Pack and
presentation; resolving it means authoring the pack's café from the slice's geometry, or the
reverse, which is a World Pack decision outside this PR.

**Behaviour.**

```text
welcomed   HUD: seat, observer id (string), world instance
observed   reconcile: every perceived Person with a location in the café gets a stand-in figure at
           the mapped position, keyed by EntityId string (set_meta), removed when no longer listed.
           The observer's own entity is the local player and is not drawn twice.
move       the player inside the café volume: submit MOVE_ACTION with the mapped location and yaw
           when the body has moved > 0.30 m or turned > 20 deg since the last submission, at most
           every 0.4 s. Outside the café, nothing is submitted: social-cafe models no street place.
           Prediction is local and the server is the authority (ADOPTION.md §4).
talk       E: submit `talk` to the perceived person nearest the camera's forward ray (targeting is
           presentation). Submitted whatever the affordance says; the HUD shows may()/the
           unavailable reason as reported, never a client-side verdict (§3.3).
answers    resolved/refused shown on the HUD by code; never branch on detail.
```

**Validation (C8):** `--slice-link` probe, headless, against a real `mineworld server
worlds/social-cafe`. Welcome. Walk through the door on foot. At least one `arrive` resolved by the
server. The next observation's `self_location` equals the submitted position, millimetre for
millimetre. Alice and Bob reconciled as figures inside the room. One `talk` resolved or rejected
by the server with its code printed. Transcript saved under `shots/slice/` (scratch, not
committed).

## 7b. C8 after S6 — `move`, the street, and the doorway (2026-10-06)

**Trigger.** S6 merged to `main` (`6f61582`, `ef53484`): `arrive` is retired, `move` is the one
movement action (`DECISIONS.md` `ARC-26`), and `worlds/social-cafe` gained a `street` place joined to
the café by one passage. Directed by the primary session. Bounded: no contract, kernel, system,
server or World Pack change; the slice's adapter (`slice_link.gd`), its probe and its launcher only.

```text
action     MOVE_ACTION = "move", payload { "to": Location } (server/PROTOCOL.md §6.2)
reporting  every 0.50 m of travel or 20 deg of turn, at most every 0.08 s -- a quarter of the
           MAX_STRIDE (2 m) rule; at the 3.10 m/s jog, every ~0.16 s against the rule's ~0.65 s
height     never reported: a jump straight up sends nothing; a running jump sends ordinary strides
crossing   when the body enters the other place's slice volume, report it there at once; the
           movement system decides whether the door may be crossed
authority  the body is put where the first observation says, and again after any refused move
           (reconcile, never argue or retry)
places     the observer's place is learned from its tag; the other place from --places, which the
           launcher reads from `mineworld validate`'s identity table
```

**Spatial binding — re-decided, because the server now rules on the doorway.** `places/cafe.yaml`
puts the doorway at `here` (5.0, 0.2) m in the café frame and `there` (0.0, 3.0) m in the street
frame. The world frame is fixed (+x east, +y north, `CORE_CONCEPTS.md` §6.1), so a binding may only
translate. Each place is now bound so that the pack's doorway lands on the slice's door (0.2 m each
side of the façade): a crossing is accepted only within `MAX_STRIDE` of the doorway on both sides,
so it is the one point the binding must get right. §7a's binding (origin at the room's inner
front-west corner) put the pack's doorway 3.36 m east of the slice's door, and every crossing
would have been refused `too_far_away`.

**Does the slice's geometry match the pack? No, and no translation can make it.** The pack's café
has its door **east** of everybody in it (door x 5.0; people x 1.2–4.6); the slice's door is at
the café's **west** end, as `03` draws it. With the doorway aligned, `visitor` (4.6, 0.2) stands
0.4 m inside the door and `wanderer` is inside the room, but `alice` (1.2, 2.4) and `bob`
(1.4, 0.6) are drawn 0.3–0.5 m west of the café's west wall. A mirror would fit both and is
forbidden by the fixed frame. Resolving it is a World Pack decision outside this PR, offered to the
operator: re-author `social-cafe`'s café from the slice (doorway at x ≈ 1.6 m, or people moved east
of it), or move the slice's door to the pack's — which contradicts `03`.

**Architectural hole, surfaced rather than patched.** `MovementSystem` discloses no `Passages`
(`ARC-26`), and an observation lists only the observer's own place. So a client inside the café is
never told the identity of the place its door opens onto, and cannot name it in `move.to`. The
slice uses the identity table the pack's own `mineworld validate` prints, passed as `--places=`;
the CLI's test names that table "what an author checks before writing a client that refers to
them". That is a stopgap. The proposed fix is in `systems/movement`, not here: disclose the
observer's place's `Passages` (destination identity and doorway) in its observation, so a client
learns where a door leads from the world.

**Evidence** — `./mineworld-slice --world --link`, headless, against `mineworld server
worlds/social-cafe`, the real controller walking and jogging, never placed:

```text
seated as observer 5 in place 1 (cafe); body placed where the world says (4600, 200)
out      through the café door onto the pavement -> server place street
jog      6 m east and 6 m back at 3.10 m/s; one standing jump, one running jump
in       back through the door                     -> server place cafe
places   [cafe -> street, street -> cafe]
move     50 reports, 50 accepted, 0 refused (no too_far_away)
facts    48 moves stated one fact (Arrived); the two crossings stated two (Arrived and
         PersonEnteredPlace), counted from the server's answers, which name facts by EventId only
position last report (4986, 1138) = the server's view (4986, 1138)
talk     to alice (3) -> rejected too_far_away: she is drawn 5.2 m away, beyond the wall (above)
```

## 8. Deviations and discoveries

1. **Shared files edited.** `scripts/props.gd` (bough and bicycle orientation, a bug, which also
   corrects the promenade's trees) and `clients/3d-spike/.gitignore` (admit
   `shots/slice/preview/` only). Neither is owned by `vis/3d-human-pipeline`. The promenade's
   `./mineworld-3d --drive` still completes with its checks.
2. **Preview before `TECHNICALLY READY`**, per the operator's direction and `ARC-24`. C8
   (server integration) is not done; the preview says so.
3. **The constants-only scale table** was an `ARC-23` instrument defect. It is kept, labelled
   *declared*, beside the new *measured* section.
