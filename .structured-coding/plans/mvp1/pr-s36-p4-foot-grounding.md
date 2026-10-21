# PR S36-P4 — Runtime foot grounding: one `TwoBoneIK3D` per leg on top of `GroundContact` (presentation only)

**Lifecycle:** `DESIGN FROZEN — 2026-10-19 (operator approved in the primary session: every question answered as recommended)`. No implementation before a `DESIGN FROZEN` header from the operator and a filled
contract (§10).

**Parent:** [`step-36-interpenetration.md`](step-36-interpenetration.md) §4.4. **Depends on:** S26-c-ground merged
(#319: `GroundContact`); S36-P2 merged (G-4w keeps the assets honest, so IK corrects millimetres, not centimetres).
**Record:** `DEP-144` (Godot's built-in IK; drafted with the step). **Audit base:** `origin/main @ 5a8c6f28`.

**Design session:** worktree `/Users/yuema137/mineworld-worktrees/design-s36-p345`, branch
`docs/pr-design-s36-p345`.

---

## 1. Goal

`GroundContact` (#319) keeps a body's lowest sole on the floor by moving the `Hips` up or down. It assumes one flat
floor: the floor node's local `y = 0`. That is enough on the café floor. It is not enough where the two feet stand on
different heights.

Example. Bob stands at the kerb with his left foot on the pavement (0.14 m up, `SliceStreet.KERB_H`) and his right
foot on the road. `GroundContact` can only move the hips. With the hips lowered to the road, the left foot is
140 mm inside the kerb; with them raised to the pavement, the right foot floats 140 mm. After this PR the hips go to
the lower foot's ground, and the left leg bends so that foot stands on the kerb.

This PR adds what a hips offset alone cannot do, and nothing else:

1. Each foot finds the ground under it (a ray against the scene's own floor colliders).
2. `GroundContact` judges each foot against its own ground, so the hips settle on the foot that is lower.
3. One `TwoBoneIK3D` per leg moves the other planted foot onto its ground, and turns the sole to the slope.
4. It all happens in the client, after the animation. Nothing is sent to the server (I-S36-6).

## 2. Audit (`origin/main @ 5a8c6f28`)

| Fact | Where |
| --- | --- |
| `GroundContact extends SkeletonModifier3D`: support points per body (cached by scene path), skinned on the CPU every frame (`lowest()`); `_process_modification` lifts the `Hips` by `−low` when sunk, or by `−low · pin` when floating; `last_lift` recorded | `clients/3d-spike/scripts/ground_contact.gd` |
| The floor is `floor_node`'s local `y = 0` (the `Human`); there is no ray and no per-foot height | `ground_contact.gd` `lowest`, `_process_modification` |
| Wiring: `Human._stand_on_soles` adds `GroundContact` as a child of the skeleton after `Posture`, sets `floor_node = self`, raises the scene by `sole_y` | `clients/3d-spike/scripts/human.gd` 420–433 |
| `pin = 1 − smoothstep(walk, jog, blend)` in `set_gait`; `sit()` sets `_ground.active = false` | `human.gd` 775–850 |
| Bone names after retargeting: `LeftUpperLeg`, `LeftLowerLeg`, `LeftFoot`, `LeftToes` (and Right) | `ground_contact.gd` `BONES`; `human.gd` |
| The project runs Godot 4.7 (`config/features = "4.7"`) | `clients/3d-spike/project.godot` line 11 |
| No IK node is used anywhere in the client yet | grep `TwoBoneIK3D`, `IKModifier3D`: no hits |
| Physics layers: `Build.LAYER_WORLD` 1 (the scene's floors and walls; `Build.ground` adds a 0.4 m box collider on it), `LAYER_BODIES` 2, `LAYER_OBJECTS` 4, `LAYER_DISCLOSED` 8 (the server's walls and solids, `client_bodies.gd`) | `clients/3d-spike/scripts/build.gd` 6–18, 103–118 |
| Uneven ground in shipped scenes today: the street's kerb `KERB_H` 0.14 m (the pavement at `WALK_Y`), door sills (`SliceLayout.in_point`'s `floor_h`). The lakeside's terrain (S33-d, frozen 2026-10-19) is **not on `main`**; `town.gd`'s spike ground is flat at y = −0.02 | `slice/street.gd` 34–38; `slice/slice_layout.gd`; `town.gd` `_terrain` |
| Perceived people (`figures.gd`) stand at the place origin's height; the player is a `CharacterBody3D` that rests on `LAYER_WORLD` colliders | `slice/figures.gd` 151; `player.gd` 37, 107 |
| The camera's vertical field of view is 70° | `camera_rig.gd` `FOV` |
| P2's probe builds every body the client draws (`Human.build_character(manifest)`), headless, from `mwcast/ground.py bodies()` (the client index's reference body and every cast `character.yaml`) | `pr-s36-p2-gait-contact.md` §2.2; `tools/character_pipeline/godot/pose_sheet.gd` `--ground-only` |
| `DEP-144`: use `TwoBoneIK3D` only (not `CCDIK3D`/`FABRIK3D`, which have forum error reports), wrapped in one script, with `LimitAngularVelocityModifier3D` after it | `docs/decisions/DEP-144.md` |

### 2.1 Findings

- **F-P4-1. The step's uneven ground is not on `main`.** S33-d is frozen, not merged. So the lakeside cannot be this
  PR's acceptance world. The acceptance uses a generated course of realistic steps and slopes, every shipped body,
  and the street's kerb; the lakeside joins the step's milestone run once S33-d lands (Q-S36-P4-1).
- **F-P4-2. P2's gate must not see the IK.** P2's probe builds bodies "as the client does". If the IK is on there,
  it would pull a floating planted foot to the floor and hide an asset defect (P2's own risk). So the pose-sheet
  probe builds bodies with foot grounding off; this PR's own probe measures it on.
- **F-P4-3. The node API must be checked, not assumed.** The 4.6 announcement documents `TwoBoneIK3D`; this design
  does not rely on remembered property names. Commit 1 lists the node's real property list in the pinned Godot and
  records it.

## 3. Scope

**In:** `clients/3d-spike/scripts/foot_grounding.gd` (new); `ground_contact.gd` (per-foot ground); `human.gd` (wiring,
gait, sit); `clients/3d-spike/assets/characters/foot_grounding.json` (new, tunables); `clients/3d-spike/tools/feet_course.gd`
(new, headless probe); `tools/character_pipeline/mwcast/feet.py` and `cast.py feet` (the judge); the pose-sheet probe's
"grounding off" switch; `scripts/ci_layer.py` (`clients-probes`); `DEP-144` status; `clients/3d-spike/PRESENTATION.md`;
tests.

**Out:** the server and every message (I-S36-6); root heights of perceived people (S33-d's terrain work); seated feet
(the seat is the contact while seated; `sit()` stands grounding down as it stands `GroundContact` down); hands
(reaching, carrying).

## 4. Commits

### Commit 1 — the API spike and the per-foot ground

**Goal.** Know the real IK API; give `GroundContact` one ground height per foot.

**Scope.**

- A throwaway headless script (not committed) instantiates `TwoBoneIK3D` and `LimitAngularVelocityModifier3D` in the
  pinned Godot 4.7 and prints `get_property_list()`. The ledger records the property names used below (root, middle
  and end bone; target; pole; influence) and the Godot build. If either class is missing: stop, record, and fall back
  per §7 (bounded: the fallback is inside the one script).
- `ground_contact.gd`: two new fields `ground_left`, `ground_right` (metres, in the floor node's frame, default 0.0).
  `lowest()` becomes `lowest_by_side() -> {left, right}`, each support point already knows its side (`_prepare`
  keeps `is_left`). The lift is `−min(d_left, d_right)` when that is a sink, else `−min(...) · pin`, with
  `d_s = low_s − ground_s`. With both grounds at 0 this is today's rule exactly.
- `last_lift` keeps its meaning; `last_heights` (per side, after the lift) is added for the probe.

**Implementation.**

- [ ] API spike, recorded in §9.
- [ ] `ground_contact.gd` per-side lowest and grounds.

**Validation.** A GDScript test (the client's existing test runner, `scripts/ci_godot.py tests`) builds the reference
body on a flat floor: with both grounds 0, `last_lift` equals the old rule's within 0.1 mm over a walk cycle. With
`ground_left = 0.14`, `ground_right = 0`, the hips settle on the right foot (its `d` is 0 ± 0.5 mm).

**Acceptance.** The flat case is unchanged; the per-side case settles on the lower foot.

**Commit boundary.** No IK yet.

### Commit 2 — `FootGrounding`: rays, IK, slope

**Goal.** Each planted foot stands on its own ground.

**Scope.** `foot_grounding.gd`, class `FootGrounding extends Node`, one per `Human`, created in `_stand_on_soles`
after `GroundContact`. It owns:

1. **Rays** (in `_physics_process`, where the space state may be queried): per foot, one ray straight down from
   `ray_up_m` above the foot's lowest support point to `ray_down_m` below it, against `Build.LAYER_WORLD` only (the
   drawn floors; never `LAYER_DISCLOSED`, whose solids are not floors, and never `LAYER_BODIES`). The hit height and
   normal are cached. No hit: that foot's ground is the floor node's `y = 0` (today's behaviour).
2. **Grounds to `GroundContact`:** `ground_left`, `ground_right` set from the cached hits every frame.
3. **Two `TwoBoneIK3D` nodes** (skeleton children, after `GroundContact`): `LeftUpperLeg → LeftLowerLeg → LeftFoot`
   and the right side. Target: the `Foot` bone's current position raised by `ground_s − low_s` (so the lowest support
   point lands on the ground). Pole: a marker in front of the knee along the body's forward direction.
4. **Slope:** the `Foot` bone turned so the sole's normal meets the ground normal, at most `max_foot_tilt_deg`.
5. **Influence per foot** follows "planted": the foot bone's world horizontal speed is at most
   `planted_drift_share × body speed` (G-8's 25 % drift rule, the same rule P2 uses on the treadmill, here in the
   world frame); standing, both feet are planted. Influence moves toward 1 or 0 at `influence_per_s`, so a foot never
   snaps.
6. **`LimitAngularVelocityModifier3D`** after both IK nodes, on the leg bones, at `max_leg_deg_per_s`.
7. **Distance:** beyond `ik_distance_m` from the active camera, the IK nodes go inactive (influence ramps to 0);
   `GroundContact` keeps working at every distance.
8. **Height difference too large:** when `|ground_left − ground_right|` exceeds `max_step_m`, the higher foot is not
   lifted (a leg cannot step that high in place); the ground is recorded as unreached for the probe.

`human.gd`: `set_gait` passes the speed; `sit()` and `stand()` switch it off and on with `GroundContact`; a static
`Human.foot_grounding := true` lets the pose-sheet probe build bodies without it (F-P4-2).

**The data file** `clients/3d-spike/assets/characters/foot_grounding.json` (`format: 1`), every value with its reason:

| Key | Value | Reason |
| --- | --- | --- |
| `ray_up_m` | 0.30 | Above the highest step the IK handles, so a foot inside a step still finds its top |
| `ray_down_m` | 0.30 | The same reach below |
| `max_step_m` | 0.20 | A stair riser is at most 178 mm (7 in, International Building Code §1011.5.2); a kerb is 100–150 mm. Above this, standing with one foot up is not a pose a body holds |
| `max_foot_tilt_deg` | 20 | A ramp is at most 1:12 (4.8°, ADA Standards §405.2); hillside paths run steeper. Above 20° the ankle's dorsiflexion range (about 20°, AAOS normal range of motion) is used up |
| `planted_drift_share` | 0.25 | G-8's planted-foot rule (`pr-s32p-wardrobe-pilot.md` §4.8), the same as P2's |
| `influence_per_s` | 8.0 | A foot's influence goes 0 → 1 in 125 ms, under a walk's stance phase (about 0.6 s at 1.4 m/s) |
| `max_leg_deg_per_s` | 720 | Twice the fastest knee swing in normal walking (about 350°/s); it stops pops, not motion. Status `inconclusive` until the commit-4 run |
| `ik_distance_m` | 20 | At 70° vertical field of view and 1080 rows, one pixel spans 26 mm at 20 m, so the planted-float bound (10 mm) is under half a pixel there. Status `inconclusive` until commit 4 measures the per-person cost |

The bounds the probe judges with are not here: they are `floor.sink.foot` and `floor.float.planted` in
`tools/character_pipeline/contact_tolerances.json` (I-S36-2).

**Implementation.**

- [ ] `foot_grounding.gd` (rays, IK nodes, slope, influence, limit, distance, step) and the data file reader, which
      refuses an unknown key or a missing one by name.
- [ ] `human.gd` wiring; the probe switch.

**Validation.** The client's tests: a body on a flat floor with grounding on and off → per-foot heights equal within
0.5 mm over a walk cycle (the flat case does nothing). A body straddling a 0.14 m step → the upper foot's lowest
point within 1 mm of the step top after 0.5 s.

**Acceptance.** Flat: no measurable change. Step: both feet on their ground.

**Commit boundary.** The client draws grounded feet.

### Commit 3 — the course probe and the judge

**Goal.** Measure every body on ground no one tuned it on, and judge in Python with the tolerance file.

**Scope.**

- `clients/3d-spike/tools/feet_course.gd` (headless: `godot --headless --path clients/3d-spike -s
  tools/feet_course.gd -- --bodies <json> --seed <n> --out <dir> [--no-ik]`). It builds a **course** from the seed:
  a flat stretch; a kerb up and down (height drawn from 0.10–0.15 m); a ramp (slope drawn from 2°–10°); a flight of
  three stairs (riser drawn from 0.15–0.178 m); and a patch of seeded unevenness (a height grid at 0.25 m spacing,
  ±30 mm). Every surface is a `LAYER_WORLD` collider built with `Build`, so the rays meet what a scene would have.
- For every body in `--bodies` (P2's list from `mwcast/ground.py bodies()`): it walks the course at its measured walk
  speed (root height at the ground under its centre, as a terrain-following figure stands), then stands still
  straddling each kerb and stair edge for 2 s. Each frame (read in `skeleton_updated`, after every modifier): per
  foot, the lowest support point's height minus the ground under that point (a second ray at that point), planted
  or not, IK influence, and `GroundContact.last_lift`. Written to `<out>/feet.json`.
- `mwcast/feet.py` and `cast.py feet <feet.json>` (stdlib JSON; bounds from the tolerance file): per body, per foot,
  per course segment, per frame — **FAIL** when a planted foot sinks below −`floor.sink.foot` or floats above
  `floor.float.planted`; frames flagged `unreached` (commit 2, rule 8) are listed, not judged. Every margin printed.

**Implementation.**

- [ ] `feet_course.gd` and its seeded course.
- [ ] `feet.py`, `cast.py feet`.

**Validation.**

- Python: synthetic `feet.json` → a planted foot at −6 mm FAILs naming body, foot, segment and frame; at −4 mm
  PASSes; a swing foot 40 mm up PASSes (not planted).
- Mutation: `--no-ik` on the kerb straddle → FAIL by about the kerb height for every body (the probe sees the
  defect it exists for).

**Acceptance.** The judge fails exactly where planted; `--no-ik` fails on the kerb and stairs.

**Commit boundary.** The probe and its judge.

### Commit 4 — every body, three courses, the street, and the records

**Runs (through `with-slot.sh`).**

- Every body from `ground.bodies()` (both shipped casts and the reference body) on courses with seeds 1, 2 and 3
  (generated variants), with IK.
- The same bodies on the flat stretch with and without IK: equal within 0.5 mm per foot per frame, and P2's G-4w
  numbers unchanged (the pose-sheet probe runs with grounding off).
- **A second, shipped scene:** the slice street (`social-cafe`'s and `market-town`'s). `feet_course.gd --scene
  street` builds the slice's street scene instead of a course, finds the kerb edges by rays across the pavement line
  (no coordinate literal), stands every body across three of them and walks one body along and across the kerb,
  measured with the same per-foot rows.
- Cost: milliseconds per frame per grounded body, measured with 20 bodies in view; `ik_distance_m` and
  `max_leg_deg_per_s` marked `cited` (measured) or kept `inconclusive` in the data file's notes.

**Records.**

- [ ] `scripts/ci_layer.py` `clients-probes` (nightly, already a Godot layer): `feet_course.gd --seed 1` and `cast.py
      feet`. Per-PR CI is unchanged (charter: run only what is necessary).
- [ ] `DEP-144` status → accepted, with the API facts of commit 1.
- [ ] `clients/3d-spike/PRESENTATION.md`: the grounding chain (animation → `Posture` → `GroundContact` → IK → limit),
      what is presentation and what is not.
- [ ] This ledger, §9: per body and course, worst sink and float per foot.

## 5. Acceptance criteria (PR level)

1. On generated courses (seeds 1, 2, 3: kerbs, ramps, stairs, uneven patches), every planted foot of every enumerated
   body stays within `floor.sink.foot` and `floor.float.planted` of the ground under it, except frames the probe
   lists as `unreached` (a step above `max_step_m`).
2. On the flat floor, IK on and IK off agree within 0.5 mm per foot per frame (nothing changes measurably), and
   P2's G-4w records do not move.
3. On the slice street's kerb (a shipped scene of `social-cafe` and `market-town`), every body standing across it,
   and one body walking over it, meets criterion 1.
4. `--no-ik` fails criterion 1 on the kerb and stairs (the mutation).
5. `foot_grounding.gd` sends nothing and writes no authoritative state: it references no `SliceLink`, no intent and
   no observation writer (a static check in the client's tests, I-S36-6).
6. `fast` is green on the final head.

## 6. Invariants

I-S36-2, I-S36-5, I-S36-6 (step §5), and:

- **I-P4-1.** The IK changes only how a body is drawn, after the animation; the root's position is never changed by
  it.
- **I-P4-2.** Every tunable is in `foot_grounding.json`; every bound the probe judges with is in the tolerance file.

## 7. Risks

| Risk | Effect | Mitigation |
| --- | --- | --- |
| `TwoBoneIK3D` misbehaves on our skeleton (new in 4.6) | knee flips, pops | the pole marker; the angular limit; the fallback inside `foot_grounding.gd` is an analytic two-bone solve (law of cosines, about 30 lines) behind the same interface; `DEP-144`'s revisit clause |
| Rays from many bodies cost frame time | slower frames in a crowd | rays only inside `ik_distance_m`; measured in commit 4 |
| A foot ray hits a prop's top (a crate) instead of the floor | a foot lifted onto a crate | `LAYER_WORLD` holds the drawn floors and walls; prop colliders from `put_solid` are blockers on the same layer, so a body stands on a crate only where its feet are over it, which is what a person would do; recorded if seen |
| The IK hides an asset defect | a bad foot passes | P2's G-4w judges the assets with grounding off (F-P4-2) |

## 8. Proposed edits to shared documents

- Step 36 §4.4 acceptance: "On the lakeside's uneven terrain (S33-d)" → generated courses and the street's kerb now;
  the lakeside joins the step's milestone run when S33-d merges (Q-S36-P4-1).
- `DEP-144` (commit 4).

## 9. Ledger

Implementation branch `mvp1/pr-s36-p4`, worktree `impl-s36-p4`, rebased on `origin/main` after S36-P2 (#378) merged.
All runs below are local, pinned Godot 4.7.2-stable official (`ed1daf0bf`), macOS, through `cast-slots/with-slot.sh`;
a nightly `clients` dispatch of the evidence head (run 39423233017) hit the job's 75-minute limit inside
`clients-probes` and left no log, so the evidence is local (INCONCLUSIVE on CI, see F-P4-impl-9).

| Commit | Implementation | Validation | Review | Evidence |
| --- | --- | --- | --- | --- |
| 1 | [x] | [x] | [x] | API spike (below); `feet_course.gd --self-test`: `flat-rule` PASS (lowest point after the per-side lift 0.001 mm), `settle-lower` PASS (0.000 mm) |
| 2 | [x] | [x] | [x] | self-test `flat-on-off` PASS (0.002 mm outside closed planted floats), `straddle-step` PASS (upper foot −0.05 mm, lower 0.00 mm); cost 0.07–0.27 ms per frame per grounded body (20 bodies, headless, loaded machine) |
| 3 | [x] | [x] | [x] | `tests/test_feet.py` 9 PASS (−6 mm FAILs naming body, foot, segment, frame; −4 mm and a 40 mm swing PASS; runs of 3; tolerance file; AC-5 static check; tunables) ; mutation `--no-ik`: seed 1 10 983 failing foot-frames against 160 with IK |
| 4 | [x] | [x] | [x] | the runs and records below; `clients-probes` entry; `DEP-144` accepted; PRESENTATION.md §2.2 |

### 9.1 API spike (commit 1)

`TwoBoneIK3D` and `LimitAngularVelocityModifier3D` exist in 4.7.2. Names used: `setting_count`;
`set_root_bone_name` / `set_middle_bone_name` / `set_end_bone_name(index, name)`; `set_target_node` /
`set_pole_node(index, NodePath)`; `SkeletonModifier3D.influence` / `active`; on the limit `max_angular_velocity`
(radians/s), `chain_count`, `set_root_bone_name` / `set_end_bone_name(index, name)`, `reset()`. Recorded in `DEP-144`.
No fallback solver was needed.

### 9.2 Runs (commit 4): every body of `ground.bodies()` (22), worst planted sink/float per foot (mm), failing foot-frames

Cells: `L sink/float, R sink/float; failing foot-frames`. Jog frames are excluded from the worst values (none is planted).

| body | seed1 | seed2 | seed3 | street | seed1-noik |
| --- | --- | --- | --- | --- | --- |
| alba-ferreira | L -4/27, R -7/7; 13 | L -0/26, R -5/9; 4 | L -0/26, R -1/7; 11 | L -0/0, R 0/54; 177 | L -164/116, R -164/32; 398 |
| alice-moreau | L -1/10, R -5/13; 3 | L -306/15, R -0/10; 5 | L -313/16, R -0/14; 9 | L -82/0, R 0/55; 354 | L -160/26, R -164/20; 592 |
| bob-achterberg | L -0/13, R -2/9; 1 | L -306/19, R -2/9; 6 | L -313/24, R -4/14; 26 | L -38/0, R -38/0; 354 | L -164/164, R -146/26; 383 |
| bruno-lindqvist | L -147/25, R -1/27; 22 | L -2/25, R -0/27; 13 | L -143/25, R -0/27; 27 | L -40/0, R -38/0; 354 | L -164/118, R -159/28; 390 |
| carol-mensah | L -2/13, R -2/10; 3 | L -4/8, R -2/8; 0 | L -1/12, R -2/12; 5 | L -53/0, R -38/48; 354 | L -164/23, R -22/20; 603 |
| celia-okafor | L -1/16, R -8/11; 5 | L -1/16, R -3/11; 3 | L -2/16, R -4/13; 8 | L -95/0, R 0/54; 354 | L -162/164, R -161/23; 598 |
| dario-russo | L -0/9, R -4/29; 3 | L -0/12, R -5/29; 10 | L -1/14, R -1/29; 21 | L -40/0, R -39/0; 354 | L -164/19, R -145/22; 337 |
| dev-raman | L -126/23, R -2/25; 27 | L -120/23, R -7/25; 31 | L -123/23, R -2/25; 44 | L -148/0, R -38/0; 354 | L -163/116, R -164/25; 642 |
| elin-berg | L -2/55, R -1/19; 11 | L -2/54, R -1/19; 11 | L -5/54, R -1/19; 16 | L -84/0, R 0/58; 354 | L -164/170, R -156/165; 637 |
| erin-walsh | L -2/18, R -2/11; 10 | L -7/12, R -2/15; 12 | L -14/14, R -2/18; 41 | L -84/0, R 0/53; 354 | L -163/169, R -18/169; 596 |
| felix-okafor | L -321/6, R -8/11; 5 | L -304/12, R -8/10; 7 | L -311/17, R -3/16; 25 | L -40/0, R -37/0; 354 | L -161/18, R -152/31; 365 |
| finn-halvorsen | L -2/9, R -9/7; 2 | L -6/16, R -7/16; 12 | L -2/18, R -3/13; 9 | L -37/0, R -40/0; 354 | L -164/17, R -164/171; 359 |
| grace-liu | L -4/43, R -3/6; 3 | L -5/43, R -2/11; 8 | L -4/43, R -3/11; 7 | L -83/0, R 0/56; 354 | L -164/43, R -164/20; 604 |
| greta-novak | L -146/6, R -83/10; 3 | L -137/9, R -102/14; 8 | L -141/12, R -91/12; 12 | L -85/0, R 0/54; 354 | L -161/164, R -164/164; 594 |
| hana-sato | L -4/11, R -4/8; 5 | L -3/10, R -0/16; 10 | L -0/11, R -1/16; 13 | L -40/0, R 0/55; 354 | L -164/21, R -32/164; 598 |
| hugo-marchetti | L -0/8, R -2/9; 0 | L -3/8, R -4/10; 1 | L -3/23, R -2/23; 12 | L -38/0, R -36/0; 354 | L -164/23, R -164/22; 371 |
| ines-duarte | L -4/16, R -162/11; 3 | L -5/16, R -309/9; 3 | L -4/16, R -157/9; 2 | L -85/0, R 0/56; 354 | L -164/164, R -162/22; 600 |
| ivan-petrov | L -327/10, R -0/9; 3 | L -310/20, R -7/16; 16 | L -317/14, R -0/16; 18 | L -40/0, R -38/0; 354 | L -164/19, R -164/19; 584 |
| nils-aaltonen | L -152/23, R -1/14; 18 | L -312/23, R -2/13; 23 | L -319/23, R -1/17; 31 | L -174/0, R -37/0; 354 | L -164/20, R -152/23; 363 |
| otto-brandt | L -0/27, R -1/30; 12 | L -4/27, R -1/30; 11 | L -0/27, R -5/30; 27 | L -40/0, R -39/0; 354 | L -162/122, R -146/171; 398 |
| vera-lindgren | L -1/103, R -2/4; 3 | L -7/121, R -5/11; 6 | L -1/111, R -2/14; 15 | L -74/0, R -176/60; 478 | L -160/23, R -26/23; 600 |
| wes-calloway | L -0/3, R -313/10; 5 | L -0/7, R -296/23; 6 | L -0/10, R -303/15; 12 | L -40/0, R -40/0; 354 | L -164/20, R -164/26; 371 |
| **all** | 160 of 28757 frames | 206 of 28757 frames | 391 of 28757 frames | 7735 of 4243 frames | 10983 of 28757 frames |

**AC-1 (courses): FAIL, small residual.** With the grounding, seeds 1/2/3 fail 160/206/391 planted foot-frames of
28 757 frames each (0.3–0.7 % of foot-frames); without it, 10 983 (seed 1). What is left: (a) the first frames of a
touchdown on a stair or kerb edge with the flat walk clip (floats 20–30 mm, 1–4 frames); (b) a few straddles where
a body's staggered stance puts a foot across the next stair's riser (−126 to −327 mm, a placement the probe cannot
avoid for that stance); (c) the uneven patch, floats up to 24 mm in single frames. Recorded, not hidden:
`clients-probes` runs the course `--record-only` until Q-S36-P4-3 is answered.

**AC-2 (flat): PASS.** Self-test: equal within 0.002 mm, except planted floats the IK closes. On the course's flat
stretches (1 s from their ends, 1 246 foot-frames, every body), IK on against IK off: worst 8.6 mm outside closed
floats (otto-brandt, one lift-off frame), floats closed up to 15.7 mm. P2's G-4w is untouched: the pose-sheet probe
sets `Human.foot_grounding = false`.

**AC-3 (the slice street): FAIL, a scene defect.** 7 735 failing foot-frames with the grounding, 7 907 without. The
street's kerb is drawn by a kerbstone mesh (top 0.18 m, 40 mm proud of the pavement) that has **no collider**, and
walked on through a **hidden 21° wedge** on `LAYER_WORLD` (`street.gd` `_pavement`, "KerbRamp_*"), which reaches 0.21 m
at its ridge. The feet's rays meet the wedge, the probe's judge meets the drawn kerbstone; the two disagree by tens of
millimetres. `cafe.gd` and `profile.gd` door steps use the same hidden-wedge pattern. Fixing it changes scene
colliders and the player's collision mask, outside this PR's scope: Q-S36-P4-4.

**AC-4 (mutation): PASS.** `--no-ik` fails every kerb and stair segment (seed 1: 10 983 failing foot-frames; kerb
straddles −97 to −115 mm).

**AC-5: PASS** (`test_foot_grounding_sends_nothing_and_writes_no_authoritative_state`). **AC-6:** `fast` on the final head.

**Hand-off item 1, bob-achterberg's 10.5 mm touchdown/lift-off float (S36-P2 finding 4): fixed.** The IK now also
lowers a planted foot that floats (by at most `max_planted_lower_m`, 13 mm). Bob's flat walk: planted float 13.3 mm
(left) without the grounding, 0.3 mm with it.

**Hand-off item 2, the jog foot-slip (S36-P2 finding 3): measured, not fixable here.** On every body the jog's
contact point drifts at no less than 0.48 of the gait speed (seed 1, `jog_flat`, least drift share), so no jog frame
is planted and the IK never engages. A foot sliding at twice the gait speed is a clip-speed or playback-rate error
(`human.gd set_gait` and the jog clip's measured `jog_mps`), not a contact error: holding the foot still would need
foot locking (a horizontal pin), which this design does not include. Q-S36-P4-5.

### 9.3 Bounded corrections and findings (implementation)

- **F-P4-impl-1. The lift rule.** §4 commit 1 wrote "−min(d_left, d_right)", which on a kerb lifts the hips onto the
  pavement and floats the road foot, the opposite of §1 and of commit 1's own check. Implemented: the lowest point is
  held against the **lower** ground (`GroundContact.lift_for`); with both grounds 0 it is the old rule exactly.
- **F-P4-impl-2. Signed correction.** The IK raises a planted foot onto higher ground **and** lowers a floating one
  (bounded by `max_planted_lower_m`, added to `foot_grounding.json` with Winter 1992's minimum toe clearance), as
  §4 commit 2 item 3 states ("raised by ground − low"); a foot that is not planted is only kept out of the ground.
- **F-P4-impl-3. Planted rule.** S36-P2 found the `Foot` bone misreads real clips (bounded correction 2 there); both
  `FootGrounding` and the probe use P2's reading: the contact point's 3D drift, once per drawn frame (the physics
  tick runs twice per frame), and the judge keeps a planted run of at least 3 frames.
- **F-P4-impl-4. Rays.** One ray per foot cannot see a kerb under the toe. Each foot casts nine (its lowest support
  point and its extremes in eight directions) from where the points were **drawn** (a ray cast from the body's
  current frame lands a frame's walk ahead of the foot); hits steeper than `max_foot_tilt_deg` are a riser, not
  ground. A support point's ground is the highest hit plane within half the foot's own width.
- **F-P4-impl-5. FootTilt.** `TwoBoneIK3D` turns the `Foot` with the shin; a small modifier after it restores the
  animated orientation turned to the ground, so a zero correction leaves the pose unchanged.
- **F-P4-impl-6. The angular limit holds the clip's own switch** (Stand → Walk, 17.8 mm on its first frame);
  `FootGrounding` calls `reset()` on every gait change.
- **F-P4-impl-7. A standing body's residual.** The shin's skin moves with the knee as the IK bends it (about 10 mm
  under the kerb on a straddle); for a body standing still, what is left under the ground after the IK is carried
  into the next frame (at most 50 mm), so it converges in a frame or two.
- **F-P4-impl-8. `influence_per_s` 8 → 30.** At 8/s the first frames of every stance were under-corrected (planted
  floats of 12 mm on stairs at influence 0.8); a contact begins within a frame, and the angular limit is what stops
  a pop. The status stays `cited`; `max_leg_deg_per_s` and `ik_distance_m` stay `inconclusive` (the cost was measured
  only headless on a loaded machine: 0.07–0.27 ms per frame per body).
- **F-P4-impl-9. CI time.** The evidence dispatch of `clients` (run 39423233017) ran out of the 75-minute job limit
  inside `clients-probes` and left no log. Locally the whole set (self-test, three seeds, the street, two `--no-ik`
  runs, 22 bodies each) takes under three minutes. The nightly keeps three feet commands; if the limit recurs, the
  layer needs its time budget checked (the coordinator's call).
- **F-P4-impl-10. The straddle placement** comes from each body's own support points (the widest gap across the edge
  within 45° of side by side, the lower foot 3 cm clear of the edge); no per-body number.
- **F-P4-impl-11. GDScript tests.** The client has no GDScript test runner (`ci_godot.py tests` runs Rust tests that
  launch the client); commits 1–2's checks are `feet_course.gd --self-test`, gated in `clients-probes`.

## 10. Questions for the operator

**Answered 2026-10-20 (operator):** Q-S36-P4-3, Q-S36-P4-4 and Q-S36-P4-5 as recommended. The follow-up is
[`pr-s36-p4b-colliders-jog.md`](pr-s36-p4b-colliders-jog.md): the course stays record-only; drawn steps collide and
the walk aids moved to their own layer; the jog's contact-point reading showed its slip is the clip's timing, not
its speed constant (its Q-S36-P4b-1).

| Id | Question | Recommendation |
| --- | --- | --- |
| Q-S36-P4-1 | The lakeside's terrain (S33-d) is not on `main`. Judge P4 on generated courses (kerbs, ramps, stairs, uneven patches) and the street's kerb now, and add the lakeside at the milestone, rather than wait for S33-d? | Yes, do not wait |
| Q-S36-P4-2 | Run the IK only within 20 m of the camera (a 10 mm error is under half a pixel there), with the distance in a data file and measured in commit 4? | Yes |
| Q-S36-P4-3 | The courses leave 0.3–0.7 % of planted foot-frames out of bounds (stair and kerb touchdowns with the flat walk clip, staggered straddles, single uneven frames). Keep the course run record-only in the nightly and gate it after a follow-up (stair/kerb step clips, or a placement that steps the body), or gate now and accept a red nightly? | Record-only now; a follow-up PR gates it |
| Q-S36-P4-4 | The street kerb (and the café and profile door steps) are walked on through hidden wedges on `LAYER_WORLD` while the drawn kerbstone has no collider, so feet stand on invisible geometry. Move the wedges to a new `Build` layer the player collides with but feet, camera and targeting rays do not, and give the drawn kerb and steps colliders? | Yes, as a small follow-up PR (touches `street.gd`, `cafe.gd`, `profile.gd`, `build.gd`, `player.gd`) |
| Q-S36-P4-5 | The jog's contact point slides at about twice the gait speed on every body (S36-P2 finding 3, confirmed here: least drift share 0.48). Fix the jog's speed calibration (`jog_mps` per body, `set_gait`'s rate) at its owner, rather than pin feet with IK foot locking? | Yes: re-measure the jog stride on the contact point; foot locking only if a calibrated clip still slides |
