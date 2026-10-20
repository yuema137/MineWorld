# PR S36-P4b — Drawn steps collide, walk aids on their own layer, and the jog's speed measured on its contact point

**Lifecycle:** `DESIGN FROZEN — operator-approved via Q-S36-P4-3/4/5, 2026-10-20` (the three questions of
[`pr-s36-p4-foot-grounding.md`](pr-s36-p4-foot-grounding.md) §10, answered as recommended). This is the follow-up
PR those answers name; its scope is exactly theirs.

**Parent:** [`pr-s36-p4-foot-grounding.md`](pr-s36-p4-foot-grounding.md) (PR #407, branch `mvp1/pr-s36-p4`,
unmerged behind a merge freeze) and [`step-36-interpenetration.md`](step-36-interpenetration.md) §4.4.
**Branch:** `mvp1/pr-s36-p4b`, cut from `origin/mvp1/pr-s36-p4 @ 95521f1a`, worktree
`/Users/yuema137/mineworld-worktrees/impl-s36-p4b`. Until #407 merges, this PR's diff against `main` contains #407's
four commits; its own commits are the ones below.

---

## 1. The three answers

| Id | Answer (operator, 2026-10-20) | What this PR does |
| --- | --- | --- |
| Q-S36-P4-3 | The generated course runs stay record-only in the nightly until a follow-up gates them | Confirmed in #407: `scripts/ci_layer.py` `clients-probes` runs `cast.py feet --run … --seed 1 --record-only`. Nothing changes for the course. |
| Q-S36-P4-4 | Move the hidden kerb and door-step wedges to a walk-aid layer the foot rays ignore, and give the drawn kerbstones and door steps real colliders, generically | Commit 1 |
| Q-S36-P4-5 | Fix the jog foot-slip at its owner: re-measure `jog_mps` (and so `set_gait`'s rate) from the jog clip's own stride on the contact point, not by IK foot locking | Commit 2 |

## 2. Audit (`origin/mvp1/pr-s36-p4 @ 95521f1a`)

| Fact | Where |
| --- | --- |
| Physics layers: `LAYER_WORLD` 1, `LAYER_BODIES` 2, `LAYER_OBJECTS` 4, `LAYER_DISCLOSED` 8 | `clients/3d-spike/scripts/build.gd` 6–18 |
| The street's kerb: a drawn kerbstone box (top 0.18 m, 0.28 m deep, **no collider**) beside a drawn pavement (top `KERB_H` 0.14 m, collider), and a hidden 21° box wedge `KerbRamp_*` on `LAYER_WORLD` whose ridge stands at 0.21 m; its plane passes under the kerbstone's road edge by 54 mm | `slice/street.gd` `_pavement` 72–105 |
| The street's gutter course (top 0.038 m) and zebra bars (top 0.016 m) are drawn with no collider: a foot beside the kerb stands 38 mm inside the drawn gutter (the ledger's −37…−40 mm street sinks) | `slice/street.gd` `_carriageway`, `_crossing` |
| The café's door step (`FLOOR_Y` 0.15 m) and threshold are drawn with no collider; a hidden 15° wedge on `LAYER_WORLD` | `slice/cafe.gd` 250–266 |
| A profile's open door: step and threshold (0.12 m) drawn, hidden 13° wedge on `LAYER_WORLD`; a closed door's step drawn with no collider and no wedge | `slice/profile.gd` 370–411 |
| Who masks `LAYER_WORLD`: the player's body (`Player.COLLIDES_WITH`) and its press cast (`PRESS_CAST_MASK`); the camera boom (`camera_rig.gd` 214); targeting (`targeting.gd` `RAY_MASK`); `FootGrounding`'s rays; the probes (`slice_probe*.gd`, `feet_course.gd`) | grep `LAYER_WORLD` |
| `--slice-geometry` compares every `LAYER_WORLD` collider in the person's height band against the server's disclosed solids, and passes tilted boxes no steeper than the controller climbs (ramps) as floor | `slice/slice_probe_geometry.gd` `_scene_shapes`, `_ramp`, `_collider_row`, `_sweep` |
| `jog_mps` (and `walk_mps`) per cast body in its `character.yaml` `clips`, measured by `tools/measure_stride.gd`: the foot's peak-to-peak travel against the hips is one stride, two strides a cycle, so speed = 2 · p2p / length. That holds when each foot is planted half the cycle; a jog's foot is planted about a third of it, so the reading is not the speed the planted foot moves at | `tools/measure_stride_runner.gd`; `human.gd` 50–65 |
| `set_gait` pins the blend to the jog point above the walk/jog midpoint and plays it at `rate = speed / jog_mps`: the rate is right exactly when `jog_mps` is | `human.gd` `set_gait` |
| #407's ledger: the jog's contact point drifts at no less than 0.48 of the gait speed on every body, so no jog frame is planted and the IK never engages | `pr-s36-p4-foot-grounding.md` §9.2 |
| CC-9 judges each manifest's recorded `ground` block, including the jog's planted float when the jog has planted frames | `scripts/check_cast.py` 284–305 |

## 3. Scope

**In:** `build.gd` (`LAYER_WALK_AID`, `Build.step`), `slice/street.gd`, `slice/cafe.gd`, `slice/profile.gd`,
`player.gd` (mask), `slice/slice_probe_geometry.gd` (a step is floor), `tools/measure_stride_runner.gd` (the
contact-point reading), the 22 cast manifests' `clips.jog_mps` (and their re-recorded `ground` blocks when the jog's
numbers move), `human.gd` `JOG_CLIP_MPS`, `scripts/ci_layer.py` (the street run), `PRESENTATION.md`, this ledger.

**Out:** the server and every message (I-S36-6); the generated courses' gate (Q-S36-P4-3); foot locking; stair or
kerb step clips; the walk's calibration (it is planted today; re-measured and recorded, changed only if its own
reading disagrees by more than the planted rule allows — then it is a finding for the operator, not changed here).

## 4. Commits

### Commit 1 — drawn steps collide; walk aids on their own layer

1. `Build.LAYER_WALK_AID := 16`: hidden slopes that make a step a walk. The player's body collides with them
   (`Player.COLLIDES_WITH`, and the press cast that bounds a stride, as it did when they were on `LAYER_WORLD`);
   nothing else masks them — not the feet's rays, not the camera boom, not targeting, not the probes' scene rays.
2. `Build.step(parent, centre, size, mat, approach, low_y, yaw)`: draws a step box **and** gives it a box collider on
   `LAYER_WORLD`, in the group `Build.STEP_GROUP`; with `approach` ±1 (the local ±z face the step is walked onto
   from) it adds the walk aid computed from the box: a slope of `Build.WALK_AID_DEG` (21°, the street's, inside the
   controller's 52° floor limit) from `low_y` in front of that face up to the face's top edge, its top face ending
   at that edge (no lip above the step), on `LAYER_WALK_AID`. No coordinate literal per scene: the helper takes the
   geometry the scene already draws.
3. The street's kerbstones, the café's door step and threshold, and both profile door variants' steps and thresholds
   are drawn with `Build.step`; the three hand-placed wedges are deleted. The gutter courses and the zebra bars
   (drawn walking surfaces of the same class, a bounded discovery) get plain `LAYER_WORLD` colliders.
4. `--slice-geometry`: a body in `Build.STEP_GROUP` is floor (walked onto), passed like a ramp in both directions.

**Validation.** `feet_course.gd --scene street` (every body) — the AC-3 run. The slice's own walk probes that cross
the kerb and the café door run in `clients-probes` (`slice --drive`, `--world --geometry`, `--world --bodies`); one
nightly `clients` dispatch of the final head (§6).

### Commit 2 — the jog's speed measured on its contact point

1. `tools/measure_stride_runner.gd` reads each clip's ground speed from the **contact point**: over one cycle
   (sampled at 120 points), per foot, the lowest support point (`GroundContact`'s, the vertex P2's and P4's planted
   rule reads), its velocity against the body; the stance frames are those in which that point is within
   `floor.float.planted` of its lowest height over the cycle; the clip's speed is the median backward speed of the
   stance frames, both feet. It prints the old stride reading beside it and the planted share the new speed gives
   under G-8's rule. Measured with `Human.foot_grounding = false` (the asset, not the IK; F-P4-2).
2. Every cast manifest's `clips.jog_mps` and `measured_by` are rewritten from that run; `JOG_CLIP_MPS` likewise for
   the town and reference bodies. `set_gait` is unchanged: its rate is `speed / jog_mps`, now right by construction.
3. The manifests' `ground` blocks are re-recorded with `cast.py ground --write` if the jog's planted share moves
   (CC-9 then judges the jog's planted float from the tolerance file).

**Validation.** The seed-1 course before and after (`jog_flat` segment): least drift share per body and foot, and
the planted jog frames the IK now engages on, judged against `floor.sink.foot` / `floor.float.planted`.

### Commit 3 — runs, the street gated, records

- Runs (local, `with-slot.sh`): every body on the street (AC-3), seed 1 (the jog before/after, AC-2), and the
  `--self-test`.
- `clients-probes`: the street run loses `--record-only` once it passes (it is the street-kerb acceptance); the
  course stays record-only (Q-S36-P4-3).
- `PRESENTATION.md`: the walk-aid layer. This ledger, §6.

## 5. Acceptance

1. **AC-3 of #407 for every body.** On the slice street (the shipped scene of `social-cafe` and `market-town`), all
   22 bodies of `ground.bodies()` — alba-ferreira, alice-moreau, bob-achterberg, bruno-lindqvist, carol-mensah,
   celia-okafor, dario-russo, dev-raman, elin-berg, erin-walsh, felix-okafor, finn-halvorsen, grace-liu,
   greta-novak, hana-sato, hugo-marchetti, ines-duarte, ivan-petrov, nils-aaltonen, otto-brandt, vera-lindgren,
   wes-calloway — standing across three kerb stations, and the first walking over and along the kerb: every planted
   foot within `floor.sink.foot` and `floor.float.planted` of the drawn surface under it (`cast.py feet` PASS).
2. **The jog slip.** On the generated course (seed 1; seeds 2 and 3 as the generated variants), every body's
   `jog_flat` least drift share falls below `planted_drift_share` (0.25) — jog stance frames are planted and the IK
   engages — shown against the 0.48 floor before. The course itself stays record-only (Q-S36-P4-3).
3. No foot ray, camera boom or targeting ray meets a walk aid (they mask `LAYER_WORLD`; the aids are on
   `LAYER_WALK_AID` only); the player still walks up the kerb and through the café's and the profile's doors
   (`slice --drive`, `--world --geometry`, `--world --bodies` green).
4. `fast` green on the final head.

## 6. Invariants

I-S36-2, I-S36-5, I-S36-6, I-P4-1, I-P4-2, and:

- **I-P4b-1. Presentation only.** Nothing is sent to the server; no message, observation or intent changes. The
  colliders and walk aids are the client's scene; the player's movement stays prediction the server decides.
- **I-P4b-2. Generic.** Every collider and walk aid is derived where the geometry is built; no scene holds a
  wedge's coordinates.
- **I-P4b-3. Measured, cited.** `jog_mps` is a measurement with its `measured_by`; nothing is tuned by hand.

## 7. Ledger

| Commit | Implementation | Validation | Review | Evidence |
| --- | --- | --- | --- | --- |
| 1 | [ ] | [ ] | [ ] | |
| 2 | [ ] | [ ] | [ ] | |
| 3 | [ ] | [ ] | [ ] | |
