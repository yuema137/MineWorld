# PR S34-f — sitting and standing in the 3D client

## Lifecycle

```text
Design revision:        revision 2 (2026-10-26), DRAFT — R2: recorded mocap replaces code-built clips (§15)
Approved by / evidence: none yet — awaiting the operator's freeze (questions in §14)
Audit base:             origin/main @ 0b2faf2f (after #462 S33-f, #368 S34-c1, #363 S34-b, #301 S34-a)
Start condition:        S34-e merged (the shared walk-then-sit chain in clients/shared/sitting/, and the
                        2D half of AC-S34-8); #460 (16f-d2) merged or this branch rebased on it (§2.3)
Execution contract:     §12
Lifecycle:              DRAFT — not frozen. Only the operator freezes this design.
```

**Effort:** `mvp0`, release-blocking · **Step:** [`step-34-posture.md`](step-34-posture.md) §§8.2, 8.3, 9.1,
12–14 (the S34-f row; AC-S34-8 3D half, AC-S34-9, AC-S34-12; X-S34-6, X-S34-7; I-S34-4, -5, -8, -9;
Q-S34-12, Q-S34-14) · **Previous PRs:** [`pr-s34a-posture-pack.md`](pr-s34a-posture-pack.md) (#301),
[`pr-s34b-bodies-anchoring.md`](pr-s34b-bodies-anchoring.md) (#363),
[`pr-s34c-world-sit-points.md`](pr-s34c-world-sit-points.md) (#368, S34-c1) · **Sibling:**
`pr-s34e-2d-seated.md` (S34-e, designed in parallel; not on main at this audit) · **Consumes:**
[`pr-s33f-interiors-sit-points.md`](pr-s33f-interiors-sit-points.md) (#462: the shared decision, the depiction
file, `Human.stand()`, `SeatContact`, the `--seat` probe) · **Records:** a dated note on `ARC-147`; **a `DEP-`
record is needed** for adopting the recorded sitting clips (Q-S34f-2). This design does not write it; the
coordinator assigns the id and the record lands with C1.

**Working name:** S34-f. **Branch:** `mvp0/pr-s34f`. One worktree, one session.

**In one sentence.** S33-f already draws a seated person in 3D, but only by placing them on the seat in one
frame, only for other people, and only in Lakeside; this PR makes sitting down and standing up visible
motions recorded from real people (motion capture, retargeted onto the standard humanoid and checked
against anatomical joint limits), lets the player sit with E and stand by moving, draws the player's own body seated, gives Social
Café and Market Town their seat depictions, and turns the `--seat` probe into a gate over the points the
server discloses, in every world that has them.

---

## 0. Binding inputs

- `CLAUDE.md` §4 rules 3, 14, 15, 16. The client reports intent. The server decides who sits where.
  Height, pelvis and animation are presentation.
- Step 34 (frozen 2026-10-18): §8.1 SD-S34-8 (one shared decision), §8.2 (depictions bound by
  `{place tag, key}`; drawn-vs-disclosed check), §8.3 (the S34-f list), §9.1 (one path), I-S34-4
  (clients decide nothing), I-S34-8 (no height in the simulation).
- Q-S34c-2 (ruled 2026-10-19): `seat_offset_m` is signed. A chair's point stands behind the chair; the
  pelvis is drawn forward onto the seat.
- Q-S34-12 (ruled): clip adoption only after reading each licence at source; nothing is bought. Decided at
  this freeze (§4.2, Q-S34f-2). The step's "procedural transitions now" is superseded by the standing rule
  below, as the coordinator directed on 2026-10-26.
- Tools first, no hand-tuning (operator, 2026-10-19), as applied to natural pose by S26-c-pose (`DEP-146`):
  motion comes from real recordings, retargeted onto the standard humanoid, and checked by a joint-limit
  gate. Code may fit a clip to a seat; it never authors motion (I-34f-7).
- Q-S34-14 (ruled): Opus frame review, Haiku implementation.
- Lane-lead charter: no world-specific hard-coding; no interpenetration at rest or across animation;
  tools first, no hand-tuned per-asset numbers; every acceptance criterion names a second world or a
  generated variant.

## 1. What S34-f must still deliver (gap analysis)

The step's S34-f row predates S33-f. S33-f (#462) landed part of it. This table separates done from open.

| Step §8.3 item | State on main | Left for S34-f |
| --- | --- | --- |
| `human.gd` `stand()` | **Done**: `stand()` returns the tree to `move` and re-enables `Posture`, ground and feet (`human.gd:738-750`) | Nothing |
| Pelvis calibration | **Done, better than planned**: `sit_on(surface_y)` measures each body's own seat vertex (`SeatContact.hips_over_seat`, `seat_contact.gd:79-85`) instead of the fixed 0.250 m | Nothing |
| Sit-down / stand-up transitions, `xfade_time > 0` | **Open**: `mode.xfade_time = 0.0` (`human.gd:512`); a person snaps between poses. The seated pose itself is a code-built one-key clip (`_sit_clip`, `human.gd:544-571`), which the standing rule no longer allows | C1–C3: recorded clips, the state machine, the joint-limit gate |
| `figures.gd` reads `posture-state` | **Half**: `SliceSeating` reads it through the shared rule and calls `figures.seat()` (`seating.gd:24-42`), which **places** the figure on the seat in one frame (`figures.gd:69-80`) | C4: walk to `at`, turn, sit down; stand up, then walk |
| Player E / move-to-stand | **Open**: E is "ask words of the person faced" (`slice_link.gd:947`); nothing submits `sit` or `stand` | C5 |
| The player's own body seated | **Open**: `seating.observe` only walks `figures.nodes`; the observer is not among them (`slice_link.gd:372-395`) | C5 |
| Depictions bound by key | **Done for Lakeside only**: `sit_points/lakeside.yaml`. Social Café and Market Town disclose 21 points each (S34-c1) but have **no depiction file**, so `pose_of` returns `{}` and every seated person there is drawn standing (`seating.gd:47-50`) | C6 |
| Drawn-vs-disclosed check | **Open**: the probe only fails when no drawn seat lies under the pelvis (`slice_probe_seat.gd:138-142`); a disclosed point with no depiction is never reported | C7 |
| `--seat` probe on server-disclosed points | **Half**: the probe iterates the depiction file's points, and uses the disclosed `at` where the world discloses one (`slice_probe_seat.gd:127-131`); it runs only for Lakeside in CI (`scripts/ci_layer.py`, S33-f line) | C7 |
| AC-S34-8 (3D half of AC-13) | **Open** | C8 |
| `VIS-POSTURE` | **Open** | C9 |

## 2. Source audit (`origin/main @ 0b2faf2f`)

### 2.1 The 3D client

| # | Fact | Where |
| --- | --- | --- |
| A-1 | The tree is `Locomotion` → `Rate` → `Mode` (an `AnimationNodeTransition` with inputs `move`, `sit`), `xfade_time = 0.0` | `clients/3d-spike/scripts/human.gd:497-528` |
| A-2 | `Sit` is a one-key looping clip built in code; Quaternius' `Sitting_Enter/Idle/Exit` are in the paid tier | `human.gd:537-571` |
| A-3 | `sit(seat_y)` puts the `Hips` joint at `seat_y` after two frames (`_settle_seat`, `_place_on_seat`); `sit_on(surface_y)` then lifts it by the measured hips-over-seat | `human.gd:701-735`, `796-825` |
| A-4 | `SeatContact.lowest` CPU-skins the seat vertices (hips and upper legs) of the current pose; one rule for every body | `clients/3d-spike/scripts/seat_contact.gd:23-74` |
| A-5 | `SliceFigures.seat(id, at, yaw, height)` sets position and yaw and calls `sit_on` at once; `unseat` calls `stand()`; a seated figure is skipped by `_process` | `slice/figures.gd:66-90`, `152-159` |
| A-6 | `SliceSeating.pose_of` takes `at` and `facing` from the disclosed `sit-points` value when present, else the depiction's fallback; pelvis = `at` moved by `seat_offset_m` against facing | `slice/seating.gd:47-67` |
| A-7 | The shared decision has a posture branch: where the place discloses `sit-points`, a person is seated iff their `posture-state` names a point | `clients/shared/sitting/sit_rule.gd:95-113` |
| A-8 | The depiction file is `presentation/mineworld-default/3D/sit_points/<world>.yaml`; only `lakeside.yaml` exists | `presentation/mineworld-default/3D/sit_points/` |
| A-9 | `--seat` probe (`SliceProbeSeat`): walks the world's probe route; for each depiction point bound to the place, rays down for the drawn seat, seats every cast body, extra, default and the town figure, and fails beyond 20 mm. Windowed (skinning needs it) | `slice/slice_probe_seat.gd:1-199`; `mineworld-slice` (`--seat`) |
| A-10 | CI runs `--seat` for Lakeside only | `scripts/ci_layer.py` (the S33-f line under the clients layer) |
| A-11 | Key handling: E talk, F kick, G throw, R shove; `aim()` knows people and loose objects only | `slice/slice_link.gd:856-873`, `943-950` |
| A-12 | Every request is built in `SliceIntents`; `submit_offered` sends a complete affordance unchanged | `slice/intents.gd:1-70` |
| A-13 | `affordances(action, target)` with `target == null` returns offers directed at nobody; `sit` and `stand` have `target: None` and payloads `{point}` and `{}` | `clients/protocol/mineworld/observation.gd:200-226`; `systems/posture/src/action.rs:1-60` |
| A-14 | The player's body is an `NPC` PUPPET with a `Human` body; hidden in first person | `scripts/player.gd:159-177`; `npc.gd:83, 124-160` |
| A-15 | Furnishings are drawn from the world's `body: furnishings` by key and kind (`ARC-307`); café chairs and stools have keys such as `table.east.chair-w`, `stool.1` | `slice/furnishings.gd:149-349`; `worlds/social-cafe/places/cafe.yaml:50-64` |
| A-16 | The café's sit points stand behind their chairs and stools (Q-S34c-2): e.g. `table-1.1` at (3 648, 3 060) for the chair at (4 200, 3 060), facing 81 406 | `worlds/social-cafe/places/cafe.yaml:129-138` |
| A-17 | The street's benches are drawn by the slice's code, not from furnishings; the park has no drawn bench | `worlds/social-cafe/places/street.yaml:106-126`; `park.yaml:36-46` |
| A-18 | AC-13's harness compares frames the real Godot client submitted (`clients/protocol/evidence/request-<flavour>.json`) through `mineworld_server::semantic_core` | `tools/cli/tests/ac13_semantic_parity.rs:1-60, 176-230` |
| A-19 | The pose sheet renders P-4 seated with the client's own `human.gd` and `seat_contact.gd`; G-13 is decided there | `tools/character_pipeline/godot/pose_sheet.gd:21-35, 438-530`; `mwcast/pose_sheet.py:36` |
| A-20 | S36's contact module exists: libigl (`DEP-142`) and trimesh / python-fcl (`DEP-143`) behind `mwcast/contact.py`; tolerances in `contact_tolerances.json` (`item.overlap.parts`, `floor.sink.foot`, …) | `tools/character_pipeline/mwcast/contact.py:1-40`; `contact_tolerances.json` |
| A-21 | The station halls of Social Café and Market Town draw 34 bench rows each from furnishings, but no sit point is authored there | `worlds/social-cafe/places/station.yaml:25-40`; `grep sit-points worlds/` |
| A-22 | Lakeside does not enable `posture` yet (S34-c2 pending); its probe runs on the fallback branch | `worlds/lakeside/world.yaml` |
| A-23 | **The recorded-clip pipeline exists** (S26-c-pose, `DEP-146`): `clients/3d-spike/tools/blender/stand_from_bvh.py` imports a BVH with Blender's own importer, folds the extra chest joint in world space, renames bones to the `mixamorig_*` names, loops the take, removes root drift, and exports a GLB with one action. The GLB imports through `vitruvian_bonemap.tres` + `SkeletonProfileHumanoid` + the rest fixer, the same path as every body. Output: `assets/characters/stand_lib.glb`; attribution in `LICENSE.100style.txt` and `ASSETS.md` | `clients/3d-spike/tools/blender/stand_from_bvh.py:1-40`; `docs/decisions/DEP-146.md`; `clients/3d-spike/ASSETS.md:236-243` |
| A-24 | **The joint-limit gate exists for the arms only**: `stand_gates.gd` measures shoulder, elbow, forearm and wrist by swing–twist about the profile axes, against AAOS normal ranges (Greene & Heckman 1994), on every frame of the `Stand` loop, on all 22 bodies; `check_cast.py` re-checks the recorded numbers in CI. Hips, knees, ankles and spine are not measured | `tools/character_pipeline/godot/stand_gates.gd:1-37, 185-330`; `PIPELINE.md` §15.2 |
| A-25 | Foot planting on uneven floors uses Godot's built-in `TwoBoneIK3D` + `LimitAngularVelocityModifier3D` (`DEP-144`), wrapped in `foot_grounding.gd`. It is the adopted tool for keeping feet on the floor when a clip's pelvis is moved | `docs/decisions/DEP-144.md`; `clients/3d-spike/scripts/foot_grounding.gd` |
| A-26 | Earlier licence reading: `DEP-146` lists CMU as "licence wording ambiguous, and unverifiable here", Mixamo as "no redistribution (`DEP-8`)", HDM05 CC BY-SA, Bandai Namco and LaFAN1 non-commercial, SMPL/AMASS non-commercial. `DEP-8`'s test is whether a file may be relicensed, not only redistributed | `docs/decisions/DEP-146.md`; `clients/3d-spike/ASSETS.md:62-63, 298-306` |

### 2.2 Motion sources for sit-down, stand-up and the seated idle (licences read 2026-10-26)

| Source | Licence, as read | Sitting clips it actually has | Fit |
| --- | --- | --- | --- |
| **CMU Graphics Lab Motion Capture Database** | Notice, as mirrored: "free for use in research projects. You may include this data in commercially-sold products, but you may not resell this data directly, even in converted form"; older wording "free for use in research and commercial projects worldwide"; an acknowledgement line is asked for. Not a formal licence. The live page `mocap.cs.cmu.edu` failed TLS from this machine, so the text was read through mirrors (KIT, 4TU, Papers with Code) | Subject 114 (one performer, 120 Hz): **114_04 Getting up from chair, 114_05 Sitting in chair, 114_06 Getting up from chair**, 114_16 Sit down and stretch. High stool: 13_01–13_03 (sit on a high stool and stand), 14_27, 14_28 (high-stool sit-to-stand), 15_10. Trial names read on the KIT database's CMU import (motions #1131, #1015, #1017, #1019) | **Recommended.** The only redistributable set found with real chair and stool sit-down, stand-up and seated takes. Needs the operator's `DEP-8` ruling (the file cannot be resold, so it keeps its own notice and is not relicensed MIT) |
| 100STYLE (CC BY 4.0, adopted in `DEP-146`) | "This work is licensed under a Creative Commons Attribution 4.0 International License" (project page) | None. Every style has walks, runs, sidesteps, an idle and transitions (`BR, BW, FR, FW, ID, SR, SW, TR1–TR3`); no sitting | Keep for standing; no help here |
| ACCAD Open Motion Project | "licensed under a Creative Commons Attribution 3.0 Unported License" (ACCAD's mocap data page) | None listed: titles are dance, martial arts, running and general takes; no sit or chair take named | No |
| Quaternius UAL 1 | CC0 ("free to use in personal, educational and commercial projects"); the free Standard tier is 60–70 % of the pack | `Sitting_Enter`, `Sitting_Idle`, `Sitting_Exit` exist only in the paid tier (`human.gd:537`); the free tier's one idle was already rejected as an action-game idle (`DEP-146`) | Clean licence, but needs a purchase; game-authored, not recorded |
| Quaternius UAL 2 | CC0 (itch.io and OpenGameArt listings) | No sitting clip named in its listings | No |
| SFU Motion Capture Database | "free for research purposes. The data cannot be used for commercial products or resale" | None found | Excluded on licence |
| Mixamo | Use in projects allowed; raw character and animation files may not be distributed (Adobe FAQ text, as reposted on Adobe's forum; helpx returned HTTP 403) | Has sitting clips | Excluded (`DEP-8`, `ASSETS.md`) |
| Bandai Namco, LaFAN1, HDM05, SMPL/AMASS, COUCH | Non-commercial or ShareAlike (`DEP-146`; COUCH's licence page: non-commercial research) | COUCH has chair interactions | Excluded |

Sources: <https://www.ianxmason.com/100style/>, <https://accad.osu.edu/research/motion-lab/mocap-system-and-data>,
<https://motion-database.humanoids.kit.edu/details/motions/1131/>, <https://mocap.cs.sfu.ca/>,
<https://quaternius.com/packs/universalanimationlibrary.html>, <https://opengameart.org/content/universal-animation-library-2>,
<https://community.adobe.com/questions-696/mixamo-faq-licensing-royalties-ownership-eula-and-tos-589400>,
<https://data.4tu.nl/datasets/0448aab2-3332-449f-a8e2-d208cb58c7df>, <https://virtualhumans.mpi-inf.mpg.de/couch/license.html>.

### 2.3 Open PRs touching the same files

| PR | Touches | Effect on S34-f |
| --- | --- | --- |
| #460 (16f-d2, conflicting) | `slice/figures.gd` (one line in `_leave`), `slice_main.gd`, `slice_probe_world.gd`, both depiction scripts, `ci_layer.py` | Textual overlap only. S34-f starts after #460 merges, or rebases on it (start condition) |
| #496 | `slice/slice_layout.gd` | None expected; S34-f does not edit the layout |

## 3. Dependency on S34-e, stated precisely

S34-e owns three things S34-f consumes. S34-f does not write them.

1. **The shared walk-then-sit chain**, in `clients/shared/sitting/` (step §8.4). Given an observation and a
   sit point key, it answers the next intent: the complete `sit {point}` offer when it is available or
   refused for any reason but distance; a `walk-to` the point's `at` when the offer says `TooFarAway`; and,
   on the walk's end, the same `sit` offer again. Its file and function names are S34-e's; S34-f calls it
   by whatever name S34-e freezes.
2. **The 2D half of AC-S34-8**: the 2D client's recorded `sit` frame in `clients/protocol/evidence/` and the
   test case that compares it with the 3D frame.
3. **The rule-constant scan entry** (step §8.1) that no posture constant appears in `clients/3d-spike`.

If S34-e's design moves any of these into S34-f, that is a scope change for both designs and goes to the
operator. The posture branch of the shared decision itself already exists (A-7); neither PR rewrites it.

## 4. Scope and reuse

### 4.1 Scope

**In:**

1. Recorded clips `SitDown`, `SitIdle`, `StandUp` (a chair set and a high-stool set) from CMU mocap,
   converted by the existing BVH route and committed with their notice (C1).
2. `human.gd`: a state machine that plays them, and the fit of a clip to a seat (C2).
3. The joint-limit gate extended to hips, knees, ankles and spine, run over every frame of the new clips
   on every body, plus the ground and seat-contact gates (C3).
4. `slice/figures.gd`: a seated person walks to `at`, turns, sits down; on standing, stands up, then walks (C4).
5. The player: E on a sit point, movement or E to stand, the player's own body drawn seated, the seated
   eye height (C5).
6. `sit_points/social-cafe.yaml` and `sit_points/market-town.yaml`, generated from world data by a small
   committed tool (C6).
7. The `--seat` probe as the drawn-vs-disclosed gate over disclosed points, with the no-clipping check
   over the played clips, in every world with a depiction file, plus a generated variant (C7).
8. The 3D half of AC-S34-8 (C8).
9. `VIS-POSTURE`, `docs/HUMANOID_PROFILE.md`, `clients/3d-spike/PRESENTATION.md` §10, the `ARC-147` note (C9).

### 4.2 Reuse, both halves (`REUSE_POLICY.md`; `CLAUDE.md` §4 rule 16)

| Concern | Candidates | Choice |
| --- | --- | --- |
| Sitting motion | CMU mocap (subject 114 chair, subjects 13/14/15 stool); Quaternius UAL 1 paid clips; Mixamo; code-built keyframes | **Adopt CMU recordings** (§2.2). Mixamo excluded on licence; Quaternius needs spend; code-built motion is the hand-tuning the standing rule forbids |
| BVH → engine | Blender's BVH importer + `stand_from_bvh.py`'s steps (A-23); Expy Kit (GPL-3 add-on); Godot `RetargetModifier3D` | **Reuse the existing route.** Generalize `stand_from_bvh.py` into one script that cuts named frame ranges into named actions. No new dependency. CMU ships ASF/AMC and C3D; the BVH input is a published conversion of the same takes, named with its SHA-256 in the `DEP-` record (C1 picks and records it) |
| Retarget onto every body | `vitruvian_bonemap.tres` + `SkeletonProfileHumanoid` + rest fixer (in use) | **Reuse**, unchanged |
| Joint limits | `stand_gates.gd` ROM gate (arms, AAOS); published ROM tables | **Extend the existing gate** with hip, knee, ankle and spine rows from AAOS (Greene & Heckman 1994) and CDC Soucie et al. 2011, each with its citation. Facts, not licensed data |
| Feet on the floor when the pelvis is moved | `DEP-144` `TwoBoneIK3D` foot grounding (in use) | **Reuse** to keep the recorded feet planted when the clip is fitted to a different seat height |
| Blending states | Godot `AnimationNodeStateMachine` (engine) | **Adopt**; no hand-written blend timer |
| Interpenetration across the clip | S36's `mwcast/contact.py` (libigl `DEP-142`, trimesh / python-fcl `DEP-143`); `SeatContact`'s CPU skinning in Godot | **Reuse** the tolerance file and S36's rules; sample the clip in Godot with `SeatContact`'s skinning (no re-skinning outside Godot, as clipping-tools §1.3 advises) |

Forcing a wheel where it does not fit: a full-body IK solver that "makes the body sit" would author motion,
not fit it. Rejected for that reason (I-34f-7).

**Out:**

- The shared chain, the 2D drawing, the rule-constant scan (S34-e).
- Sit points in the station halls (A-21). That is world data (S34-c's kind of work), Q-S34f-6.
- Lakeside's `posture` and its server sit points (S34-c2).
- NPC frequency and the agent view (S34-d).
- Pulling a tucked chair out when a person sits behind it (Q-S34f-1 recommends a follow-up, S34-f2).
- Any server, contract or System change. If one seems needed, stop and escalate.

## 5. Design

### 5.1 Recorded clips (C1) — SD-34f-1

- **Takes.** Chair set: `SitDown` and `SitIdle` from 114_05 (Sitting in chair), `StandUp` from 114_04 or
  114_06 (Getting up from chair). Stool set: the same three from 13_01–13_03 or 14_27/14_28. C1 picks the
  take and the frame range of each by viewing them, records the choice and the reason in the ledger, and
  picks again only if C3's gates refuse the first choice. A take is never edited to pass a gate.
- **Conversion.** `stand_from_bvh.py` becomes one script, `clip_from_bvh.py`, that applies the same steps
  (A-23) to named frame ranges and writes named actions; `stand_lib.glb` is re-made by it byte-equivalent,
  as its regression. Output: `assets/characters/sit_lib.glb` (chair and stool actions), imported through
  `vitruvian_bonemap.tres` like every body. A CMU skeleton maps onto `mixamorig_*` names with its own
  name table; extra joints fold away in world space, as the fourth chest joint does today.
- **Root motion.** The take's horizontal root path is kept as data on the action (a curve from the start
  to the end position). It is not played as translation; §5.2 uses it to shape the slide.
- **Notice.** `assets/characters/LICENSE.cmu-mocap.txt` carries CMU's notice and acknowledgement, the takes
  used and the changes made; `ASSETS.md` gains the entry. The `DEP-` record (id from the coordinator)
  states the `DEP-8` ruling (Q-S34f-2).
- `_sit_clip()` (the code-built seated pose) is deleted, not kept as a fallback (as S26-c-pose deleted
  `stance.gd`).

### 5.2 Playing and fitting the clips in `human.gd` (C2) — SD-34f-2

- Replace the `Mode` transition with Godot's own `AnimationNodeStateMachine`. States: `move` (the existing
  `Rate` → `Locomotion` chain), `sit_down`, `sit`, `stand_up`. `sit_down → sit` and `stand_up → move`
  advance at the end of the clip; the other two are travelled on request. Cross-fades use the engine's
  own `xfade_time`.
- **Fit, never author** (I-34f-7). Three adjustments are allowed, each computed from the seat and the
  body, none typed by hand:
  1. **Height.** The pelvis is offset vertically so that the body's own seat vertex (`SeatContact`) lands on
     the depiction's `seat_height_m` at the end of `sit_down` and through `SitIdle`; over the clip the
     offset blends from 0 (standing) to that value, in step with the take's own pelvis descent.
  2. **Depth.** The root slides from `at` to the pelvis point along the take's recorded root path, scaled
     to the distance the depiction asks for.
  3. **Feet.** `DEP-144`'s foot grounding keeps the recorded planted feet on the floor after (1) and (2).
- The chair or stool set is chosen per seat from the depiction (`seat_height_m` above or below the two
  takes' measured seat heights' midpoint). No world or kind name decides it.
- `sit_on(surface_y, animate := true)` and `stand(animate := true)`. `animate := false` jumps to `SitIdle`
  with the fit applied; the probe and the pose sheet use it for the at-rest checks.
- The hips-over-seat value is measured once per body in `SitIdle`'s first frame and cached on the `Human`.
- `is_settling()` reports a running transition, so callers do not start another.

### 5.3 The joint-limit, ground and contact gates on the clips (C3) — SD-34f-3

- `stand_gates.gd` gains rows for hip flexion, extension, abduction, adduction and rotation; knee flexion;
  ankle dorsiflexion and plantarflexion; and spine flexion, each from AAOS (Greene & Heckman 1994) or CDC
  Soucie et al. 2011 where stricter, with the citation beside the number, as the arm rows have.
- The pose sheet's probe runs the gates on **every frame** of `SitDown`, `SitIdle` and `StandUp`, on all 22
  bodies and both sets, with the fit applied at a 0.45 m seat. `check_cast.py` re-checks the recorded
  numbers in CI, as for `Stand`.
- The same run applies the ground gate (§15.1, `floor.sink.foot`) to planted feet and G-13 (±20 mm) to
  `SitIdle`. A miss fails the build and names body, clip, frame, joint and range.

### 5.4 Other people (`figures.gd`, C4) — SD-34f-4

```text
posture-state appears (Seated {point})
    the figure walks to the disclosed `at` (its goal already: the server moved them there)
    → turns to the point's facing
    → sit_down (recorded `SitDown`, fitted per §5.2): the root follows the take's own path from `at`
      to the pelvis point
seated  (`SitIdle`, held; `_process` skips the figure, as today)
posture-state disappears
    stand_up (recorded `StandUp`, fitted): the root follows the take's path from the pelvis point to `at`
    → locomotion resumes toward the newest observed position
```

- **A figure first seen seated** is placed seated with no transition (it was seated before we looked).
- **A seat where the straight slide would pass through the drawn chair** is placed in one frame, as main
  does today, until S34-f2. The rule is data-derived and names no world: a negative `seat_offset_m`
  (the point behind a chair, Q-S34c-2) means the slide crosses the chair's back. The probe reports
  each such point as `cut` (never as a pass of the transition check). Q-S34f-1.
- A person who stands up and walks away at once (the server's auto-stand, SD-S34-6) plays `stand_up`
  before walking; the figure catches up by `CATCH_UP`'s existing rule. No new constant.

### 5.5 The player (C5) — SD-34f-5

- **Aiming at a seat.** `SliceSeating` adds one pickable body per depicted point in a drawn place: a box
  on the targeting layer at the pelvis point, the seat's width along the facing's normal from the drawn
  seat, carrying meta `sit_point = <key>`. `aim()` gains the kind `"sit_point"`. Nothing here evaluates
  distance or occupancy.
- **E** acts on what the reticle meets (Q-S34f-3): a person → talk (unchanged); a sit point → the shared
  chain of §3 with that key; while the player is disclosed seated → the complete `stand {}` offer.
- **Moving while seated** (Q-S34f-5). The first movement key press while disclosed seated submits the
  `stand {}` offer once. The body does not move until the observation shows the player standing. The
  client never predicts standing.
- **Far seat** (Q-S34f-4). The shared chain sends `walk-to` and then `sit`; the player's body follows the
  server's walk exactly as it does for any `walk-to`. 2D and 3D then send the same two intents.
- **The player's own body.** The observer's `posture-state` is read from the same observation; when Seated,
  the player's `Human` plays the same `sit_down` to the same pelvis point (`SliceSeating.pose_of`), and the
  correction rule (`ARC-50`) is suspended while seated, because the server's `at` and the drawn pelvis
  differ on purpose. On standing, the body returns to `at` and correction resumes.
- **Camera.** First person: the eye follows the `Head` bone, so the seated eye height is whatever the body
  gives. Third person: unchanged rig. Both are presentation only.
- Refusals arrive as today (`_on_refused`) and are said in the HUD in the player's language: "that seat is
  taken" for `target_unavailable`, and so on. New catalog keys only; no rule.

### 5.6 Depictions for Social Café and Market Town (C6) — SD-34f-6

- A committed tool, `tools/presentation/sit_point_depictions.py`, writes a world's depiction file from
  world data. For each disclosed sit point it finds the furnishing whose solids are nearest along the
  point's facing line; `prop` is that furnishing's key; `seat_offset_m` is the signed distance from `at`
  to the furnishing's `at` along facing (A-16: `table-1.1` → about −0.55 m); `seat_height_m` is the
  kind's seat height from the furnishings catalogue when present, else 0.45 m (`VISUAL_SLICE.md` §3.4;
  ADA 2010 §903.5, as `lakeside.yaml` cites). A point with no furnishing on its line (the street's
  code-drawn benches) gets `prop: ""` and the bench default 0.45 m.
- The tool reads only the world's files. It names no world, place or key. Social Café and Market Town
  share place files today, so their outputs are equal; they are two files because a depiction file is
  scoped to one world (`sit_rule.gd:59`).
- Every value is then checked against what is drawn by the `--seat` probe (§5.7). A value the probe
  refuses is fixed in the world data or the catalogue, never patched in the output by hand.
- Lakeside's file stays as S33-f wrote it; S34-c2 removes its `fallback` blocks.

### 5.7 The gate: `--seat` over disclosed points (C7) — SD-34f-7

In a place that discloses `sit-points`, the probe's list is the **disclosed** points, not the file's:

1. A disclosed point with no depiction fails, naming world, place and key.
2. The pelvis point must lie over the drawn seat with margin: rays down at the pelvis and at ±100 mm
   along facing must all meet the same drawn surface within 20 mm of height. A depiction 200 mm off
   (X-S34-6) puts one ray off the seat or onto the table and fails, naming the key.
3. Every body (cast looks, extras, default, the town figure) seated there: contact within ±20 mm of the
   drawn seat (unchanged, G-13). X-S34-7 (+50 mm) fails naming point and body.
4. **The no-clipping gate over the played clips.** For each point not marked `cut`, every body plays the
   recorded `SitDown`, `SitIdle` and `StandUp` with the fit of §5.2 applied for that seat. At 30 samples per
   second, every skinned vertex (A-4's skinning) is tested against the drawn furnishings' triangles; only
   the seat surface under the pelvis and the floor under planted feet may be touched. Depth beyond
   `item.overlap.parts` in `contact_tolerances.json` fails; feet below the floor by more than
   `floor.sink.foot` fail. Bounds come from the tolerance file; none is set in code. A failure names world,
   key, body, clip, frame and the furnishing met.
5. A place the slice does not draw is reported `SKIP <place> not drawn`, never passed.

In a place that discloses none (Lakeside before S34-c2), the probe keeps S33-f's behaviour over the
fallback points. The verdict line is unchanged, so the CI layer reads it as before.

### 5.8 AC-S34-8, the 3D half (C8)

A real run of the 3D client against a real server, in Social Café: walk to `window-stools.1`, look at it,
press E. The submitted frame is committed as `clients/protocol/evidence/request-3d-sit.json` with its
transcript, the same way AC-13's talk frames were (A-18). S34-e's test case then compares it with the 2D
frame through `semantic_core`. The expected values come from the real clients, never from this PR's code.

## 6. Invariants

| Id | Invariant |
| --- | --- |
| I-34f-1 | **Clients decide nothing** (I-S34-4). No distance, occupancy or stillness test in `clients/3d-spike` for a world that discloses sit points. E sends; the server answers |
| I-34f-2 | **No seated pose without a disclosed Seated** (I-S33-5) in posture worlds, the player's own body included |
| I-34f-3 | **No interpenetration.** Seated: contact within ±20 mm. Transitions: no trunk or thigh in drawn furniture beyond the tolerance file's bound. A seat that cannot pass is `cut`, never drawn through furniture |
| I-34f-4 | **Generic.** No world, place, key or furnishing name in code or in the tool. Every number comes from world data, the furnishings catalogue, the depiction file or the tolerance file |
| I-34f-5 | **Seated contact holds at rest.** With the recorded `SitIdle`, G-13 (±20 mm) still holds on every body; the pose sheet's P-4 is re-rendered from the recording and its new numbers are recorded, not tuned |
| I-34f-6 | **No server change** (I-S34-8). Nothing in `systems/`, `contracts/`, `kernel/`, `server/` changes |
| I-34f-7 | **Fit, never author.** Every frame of sitting, seated and standing comes from a recorded take. Code may only (a) offset the pelvis vertically to the seat height, (b) scale the take's own root path to the seat's depth, and (c) keep planted feet on the floor with `DEP-144`'s IK. No bone rotation is keyed, searched or solved in code. `_sit_clip()` is deleted |
| I-34f-8 | **Anatomically possible.** Every frame of every sitting clip, on every body, after the fit, lies inside the cited joint ranges (§5.3) |
| I-34f-9 | **Licence travels with the clip.** CMU's notice and acknowledgement, the takes and the changes ship beside `sit_lib.glb`; the clip is not relicensed beyond what the operator rules in the `DEP-` record |

## 7. Acceptance criteria (decided before measuring, `ARC-23`)

| Id | Criterion | Worlds |
| --- | --- | --- |
| AC-34f-1 | Another person who sits is seen walking to the point, turning and sitting down, at the recorded take's pace; on standing, standing up, then walking. A person first seen seated is drawn seated at once | Social Café (café table and street bench) **and** Lakeside (pier bench, fallback branch) |
| AC-34f-2 | The player looks at a free sit point within reach and presses E: the server records `sat-down`; the player's body (third person) is drawn seated; the first-person eye drops to the head bone. A movement key stands the player up; nothing moves before the observation says so | Social Café **and** Market Town |
| AC-34f-3 | E on a taken point shows the server's refusal and sends nothing else. E on a far point walks there and sits (the shared chain) | Social Café; Market Town |
| AC-34f-4 (AC-S34-9) | `--seat` passes: every disclosed point has a depiction; its pelvis is over the drawn seat with margin; every cast body and extra is within ±20 mm; every body passes the no-clipping gate over the played clips at every non-`cut` point | Social Café, Market Town, Lakeside, **and a generated variant**: a seeded scratch copy of Market Town whose benches and sit points are moved by a generator, with its depiction file written by C6's tool |
| AC-34f-5 (AC-S34-8) | The 3D frame for `window-stools.1` and S34-e's 2D frame have the same `semantic_core` | Social Café (the evidence world); the test is world-agnostic |
| AC-34f-6 | **Joint limits.** The extended ROM gate passes on every frame of `SitDown`, `SitIdle` and `StandUp` (chair and stool sets), on all 22 bodies, with the fit applied; the ground gate and G-13 pass on the same run; `check_cast.py` re-checks the recorded numbers | The pose sheet's cast, **and a generated variant**: the same run at seat heights drawn by a seeded generator across 0.42–0.47 m (`VISUAL_SLICE.md` §3.4) and depths across the depiction files' range |
| AC-34f-8 | **Provenance.** `sit_lib.glb` is rebuilt from the recorded BVH files (SHA-256 in the `DEP-` record) by `clip_from_bvh.py`, and the rebuild is byte-equivalent; the same script rebuilds `stand_lib.glb` byte-equivalent; no `_sit_clip` or other keyed pose remains in `human.gd` | Repository-wide (`git grep`) |
| AC-34f-7 (AC-S34-12) | `VIS-POSTURE`: the operator sits at a café table in 3D, watches a person sit and stand on a Social Café street bench, sees a seated person on the pier in Lakeside, tries a taken seat and reads the refusal, and stands by walking away | Social Café, Lakeside (the 2D step of AC-S34-12 is S34-e's) |

## 8. Planted mutations

| Id | Mutation | Must turn red |
| --- | --- | --- |
| X-S34-6 | move one Social Café depiction's `seat_offset_m` by 0.20 m | AC-34f-4 item 2, naming the key |
| X-S34-7 | raise one depiction's `seat_height_m` by 0.05 m | AC-34f-4 item 3, naming the point and the body |
| X-34f-1 | delete one point from `market-town.yaml` | AC-34f-4 item 1 |
| X-34f-2 | let `sit_down` slide straight through the chair for a negative offset (drop the `cut` rule) | AC-34f-4 item 4, naming the key and the depth |
| X-34f-3 | draw the player seated on the E press, before the observation | AC-34f-2's scripted run (`SEATED` logged before `sat-down` is observed) |
| X-34f-5 | in a test copy of `SitDown`, bend one knee 20° past full extension on one frame | AC-34f-6, naming body, clip, frame, joint and the AAOS range |
| X-34f-6 | double the fit's vertical pelvis offset | AC-34f-6's G-13 row and AC-34f-4 item 3 |
| X-34f-7 | skip the foot grounding during the fit | AC-34f-6's ground row (feet through the floor) |
| X-34f-4 | a distance test in `slice_link.gd` that refuses E beyond 0.75 m | S34-e's rule-constant scan entry |

## 9. Test ownership

| Failure class | Owner | Runs where |
| --- | --- | --- |
| Joint limits, ground and G-13 on every clip frame, every body | the pose-sheet probe with the extended `stand_gates.gd`; `check_cast.py` in CI | Godot (pose sheet) + Python |
| Clip provenance and rebuild | `clip_from_bvh.py` rebuild compared byte-wise; `git grep` for keyed poses | Blender (local, `with-slot.sh`) |
| Seat contact, depiction coverage, pelvis over seat, no clipping over the played clips | `--seat` probe (`slice_probe_seat.gd`), CI clients layer | Godot, windowed, Linux CI per `ci_layer.py`; locally through `with-slot.sh` |
| The 3D request equals the 2D one | `ac13_semantic_parity.rs` (S34-e's case, S34-f's evidence) | cargo |
| Player and figure behaviour (AC-34f-1…3) | a scripted connected run (`mineworld-slice --world=<w> --sit`), printing `SAT`, `STOOD`, `REFUSED <code>`, `CUT <key>` lines, compared by a `tools/cli/tests/` test like `agent_with_3d_observer.rs` | Godot + server, `#[ignore]`d locally, run by the clients layer |
| The tool's output | its own `pytest` over world fixtures plus the generated variant | Python |
| No server change | `git diff --stat` in review | — |
| Real-LLM Gate | **NOT REQUIRED**: no model is involved | — |

Static ownership: `gdlint`/Godot import errors and `ruff`/`pyright` for the tool; no unit test repeats them.

## 10. Commit plan

Each commit tracks Implementation, Validation and Review separately.

### C0 — Design (this file), docs only
- [ ] Implementation: this file.
- [ ] Validation: `python3 -I scripts/check_doc_headings.py`.
- [ ] Review: scope equals step §8.3 minus S33-f's delivered part (§1); every question in §14 has a recommendation.

### C1 — Recorded sitting clips (needs the `DEP-` ruling, Q-S34f-2)
- **Goal:** the motion comes from real people. **Files:** `clients/3d-spike/tools/blender/clip_from_bvh.py` (generalized from `stand_from_bvh.py`, which it replaces), `assets/characters/sit_lib.glb` and its `.import`, `assets/characters/LICENSE.cmu-mocap.txt`, `clients/3d-spike/ASSETS.md`, the `DEP-` record with the coordinator's id.
- [ ] Implementation: SD-34f-1. Download the chosen takes (no purchase), record each file's SHA-256 and source URL; pick and record frame ranges; build `sit_lib.glb`.
- [ ] Validation: `clip_from_bvh.py` rebuilds `stand_lib.glb` byte-equivalent and `sit_lib.glb` reproducibly (AC-34f-8); Godot imports `sit_lib.glb` through `vitruvian_bonemap.tres` with no unmapped bone.
- [ ] Review: no rotation is written by the script except the world-space fold and the rename; the notice ships with the clip (I-34f-9).

### C2 — Playing and fitting the clips in `human.gd`
- **Files:** `clients/3d-spike/scripts/human.gd`, `foot_grounding.gd` only if a call site is needed.
- [ ] Implementation: SD-34f-2: the state machine; the three fits; the set choice from `seat_height_m`; cached hips-over-seat; `animate` flags; `is_settling()`; delete `_sit_clip()`.
- [ ] Validation: the pose sheet's P-4 re-rendered from `SitIdle`; G-13 passes on all 22 bodies (I-34f-5); `--seat` on Lakeside passes; X-34f-6 and X-34f-7 red with C3's run.
- [ ] Review: I-34f-7 — the diff keys, searches or solves no bone rotation; every number traces to the seat, the body or the take.

### C3 — The joint-limit gate on the clips
- **Files:** `tools/character_pipeline/godot/stand_gates.gd` (or a sibling `clip_gates.gd` sharing its swing–twist code), `pose_sheet.gd` (run the gates over the three clips), `scripts/check_cast.py`, `PIPELINE.md` §15.
- [ ] Implementation: SD-34f-3: hip, knee, ankle, spine rows with citations; every frame, all 22 bodies, both sets, the fit applied; the ground gate and G-13 on the same run; the seeded seat-height and depth sweep (AC-34f-6's variant).
- [ ] Validation: AC-34f-6 passes; X-34f-5 red naming the joint and range, reverted; recorded numbers re-checked by `check_cast.py --self-test`.
- [ ] Review: each new bound has its source beside it; no bound was widened after a measurement.

### C4 — Other people sit and stand
- **Files:** `slice/figures.gd`, `slice/seating.gd`.
- [ ] Implementation: SD-34f-4, including the first-seen and `cut` rules.
- [ ] Validation: the scripted run on Social Café and on Lakeside logs walk, turn, `sit_down`, `stand_up` per AC-34f-1.
- [ ] Review: `_process` still skips seated figures; `_leave` unseats; no position reaches the server.

### C5 — The player
- **Files:** `slice/slice_link.gd`, `slice/seating.gd`, `slice/intents.gd` (only if the chain needs a send helper), `player.gd`, `correction.gd` (suspension while seated), the HUD catalog files.
- [ ] Implementation: SD-34f-5.
- [ ] Validation: AC-34f-2 and -3 on Social Café and Market Town; X-34f-3 red then reverted.
- [ ] Review: I-34f-1 — grep the diff for any distance or occupancy test; refusals come only from the server.

### C6 — Depiction files from world data
- **Files:** `tools/presentation/sit_point_depictions.py` and its test; `presentation/mineworld-default/3D/sit_points/social-cafe.yaml`, `market-town.yaml`.
- [ ] Implementation: SD-34f-6; sources block as `lakeside.yaml` has.
- [ ] Validation: tool tests on both worlds and on the generated variant; X-34f-1 red with C7.
- [ ] Review: the tool reads no name; each output value traces to a world field or a cited default.

### C7 — The seat gate and the no-clipping gate over the played clips
- **Files:** `slice/slice_probe_seat.gd`, `mineworld-slice` (help text), `scripts/ci_layer.py` (Social Café and Market Town `--seat` lines), `contact_tolerances.json` only if an existing region does not fit (then a new region with rationale and source).
- [ ] Implementation: SD-34f-7 items 1–5.
- [ ] Validation: the probe passes on three worlds and the variant; X-S34-6, X-S34-7, X-34f-2 each red, reverted, outputs recorded.
- [ ] Review: no bound set in code; `SKIP` and `cut` never counted as passes.

### C8 — AC-S34-8 evidence
- **Files:** `clients/protocol/evidence/request-3d-sit.json` and its transcript.
- [ ] Implementation: §5.8. If S34-e's test case is not on main, this commit waits; it is not rewritten here.
- [ ] Validation: `cargo test -p mineworld-cli --test ac13_semantic_parity` through `with-slot.sh`.
- [ ] Review: the frame came from a real run, not from an editor.

### C9 — Docs and VIS-POSTURE
- **Files:** `docs/ACCEPTANCE.md` (a `VIS-POSTURE` row), `docs/HUMANOID_PROFILE.md` (the sitting clips, their fit, the cached hips-over-seat), `clients/3d-spike/PRESENTATION.md` §10 (disclosed points first; the tool), `clients/3d-spike/README.md` (one line: how to sit), `docs/decisions/ARC-147.md` (dated note), `python3 -I scripts/check_decision_ids.py --write-index`.
- [ ] Implementation: as listed; a short operator test list for `VIS-POSTURE`.
- [ ] Validation: doc scripts PASS.
- [ ] Review: no spec text contradicts step §8; the README stays short.

## 11. Decisions proposed

- **SD-34f-1…7** as in §5.
- **Q-S34-12 answered (recommendation):** adopt CMU mocap takes for sit-down, seated idle and stand-up
  (chair: subject 114; stool: subjects 13/14), converted by the existing BVH route and retargeted through
  the standard humanoid import, gated by the extended joint-limit gate and the no-clipping gate. Mixamo
  is rejected (raw files may not be redistributed); Quaternius' sitting clips need a paid tier; 100STYLE and
  ACCAD have no sitting take; SFU, COUCH and the rest are non-commercial.
- **A `DEP-` record is needed** before C1 lands. It records the adoption, the rejections above, the takes
  and their SHA-256, and the operator's `DEP-8` ruling on CMU's notice. This design does not write it.

## 12. Execution contract

```text
Repository / worktree:  yuema137/MineWorld · /Users/yuema137/mineworld-worktrees/s34f-3d-sit-stand
Branch / base:          mvp0/pr-s34f from origin/main after S34-e and #460 merge and this design freezes
Implementer:            Haiku, from this commit plan (Q-S34-14); Opus picks the takes in C1, reviews C2's
                        frames and C3's and C7's gates
Start of C1:            only after the operator's DEP-8 ruling and the DEP- record (Q-S34f-2)
Authorized (on freeze): implement, validate locally through cast-slots/with-slot.sh, commit, push, open the PR
Validation budget:      the probes and tests named per commit (speed-first charter); full suites at milestone
Stop condition:         MERGE-READY: PR number and head SHA reported to the coordinator
Merge boundary:         the coordinator merges; never this session
Escalate:               any server or contract change; any change to S34-e's shared files; a bound in code
Post-merge sync owner:  this session owns this file's ledger; the primary session owns step-34 and overall
```

## 13. Risks

| Risk | Mitigation |
| --- | --- |
| #460 and S34-f both edit `figures.gd` and the probe family | Start after #460, or rebase; the overlap is small (§2.3) |
| S34-e slips | C1–C4, C6, C7 do not need it; C5's far-seat path and C8 wait |
| Café chairs are all `cut` in S34-f, so the café shows snaps at tables | Recorded per point by the probe; S34-f2 (Q-S34f-1) |
| The operator does not accept CMU's notice under `DEP-8` | C1 stops; nothing else is lost. The fallback is the Quaternius UAL 1 paid tier (CC0), which needs a spend approval with a cap |
| CMU's live terms page could not be read here (TLS failure); the text came from mirrors | The `DEP-` record quotes the page as read in a browser by the implementer or the operator before C1 lands |
| The 2001–2003 takes are noisy: foot slide, hand jitter, no fingers | Foot grounding (`DEP-144`) pins planted feet; the gates decide; another take is picked if one fails. No take is edited to pass. Fingers keep the bodies' rest curl, as with `Stand` |
| One performer's chair height differs from ours | The fit (I-34f-7) moves the pelvis and scales the root path only; AC-34f-6's sweep proves it over the seat range |
| The recorded hands push on armrests our chairs do not have | Seen in C1's viewing; such a take is not picked |
| The no-clipping gate is slow (CPU skinning per frame, per body, per point) | Every body at one point per group and per set; one body per build class at the rest; the full sweep runs at the milestone |
| Correction suspended while seated hides a real server move | Suspension ends on any change of the disclosed point or on standing; a moved seated person is a server bug S34-b's tests own |
| The player's own seated body clips the camera in first person | The body is hidden in first person (A-14); only the eye moves |

## 14. Operator questions (each with a recommendation)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S34f-1** | A chair's sit point stands behind the chair (Q-S34c-2). Sliding the body onto the seat would pass through the chair's back. Split the "pull the chair out, sit, tuck in" motion into a follow-up S34-f2, and draw those seats with today's one-frame placement until then? | **Yes, split.** S34-f animates benches and stools with open fronts; chair seats are `cut` and reported per point. S34-f2 adds the chair motion with its own contact gate |
| **Q-S34f-2** | Q-S34-12: where does the sitting motion come from? CMU's notice lets the data go into products but forbids reselling it, even converted; under `DEP-8` it therefore keeps its own notice and is not relicensed MIT. Accept that? | **Adopt CMU mocap**: chair takes 114_04/114_05/114_06, stool takes 13_01–13_03 or 14_27/14_28, converted by the existing `stand_from_bvh.py` route (generalized), retargeted through `vitruvian_bonemap.tres` + `SkeletonProfileHumanoid`, gated by the extended AAOS/CDC joint-limit gate and the no-clipping gate. Code only fits a clip to the seat. Ship the clip with CMU's notice, not relicensed. A `DEP-` record is needed (not written here). Rejected: Mixamo (no redistribution), Quaternius sitting clips (paid), 100STYLE and ACCAD (no sitting take), SFU/COUCH/LaFAN1/AMASS (non-commercial) |
| **Q-S34f-3** | E is "talk" today. Make E act on what the reticle meets: a person → talk, a seat → sit, while seated → stand? | **Yes.** One key, as Minecraft-like play expects; no new binding to learn |
| **Q-S34f-4** | E on a seat that is too far: walk there and sit (the shared chain), or only say "too far"? | **Walk there and sit**, through S34-e's shared chain, so 2D and 3D send the same two intents (step §8.4) |
| **Q-S34f-5** | A movement key while seated: send `stand` and wait for the world (one round trip), or move at once and let the server auto-stand? | **Send `stand` and wait.** The client never predicts posture; the delay is one observation |
| **Q-S34f-6** | The station halls draw 34 benches each in Social Café and Market Town, but have no sit points. Add them here? | **No: a small data PR, S34-c3,** under S34-c's frozen rules. S34-f's gate covers whatever points the world discloses |
| **Q-S34f-7** | Social Café and Market Town depiction values: generate them from world data with a committed tool, or author them by hand? | **Generate** (tools first, no hand-tuning). The probe then checks every value against what is drawn |

## 15. Evidence ledger

**2026-10-26, revision 2 (design, before freeze).** The coordinator rejected revision 1's Q-S34f-2 ("build
the animations in code"): it breaks the standing rule "tools first, no hand-tuning", which S26-c-pose
(`DEP-146`) already applied to natural pose. Revision 2:

- surveys redistributable sitting motion at source (§2.2) and recommends CMU takes;
- reuses the repository's recorded-clip route (A-23) and joint-limit gate (A-24), and extends the gate to
  the lower body (§5.3);
- adds I-34f-7 (fit, never author), I-34f-8 (joint limits), I-34f-9 (licence), AC-34f-6 (joint limits on
  every frame), AC-34f-8 (provenance), X-34f-5…7, and the no-clipping gate over the played clips (§5.7
  item 4);
- re-orders the commit plan (C1 clips, C2 fit, C3 gates, then the client work);
- states that a `DEP-` record is needed and does not write it.

Implementation evidence: none yet.
