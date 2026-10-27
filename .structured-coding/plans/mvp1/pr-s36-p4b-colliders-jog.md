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

1. `Build.LAYER_WALK_AID := 64` (16 at design time; `main`'s `LAYER_CAMERA`, #428, took bit 16 before this
   merged, then 16f-b's `LAYER_ART` took 16 and moved `LAYER_CAMERA` to 32 — bounded corrections at each merge of
   `main`): hidden slopes that make a step a walk. The player's body collides with them
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

**Deviation (2026-10-20, F-P4b-2).** Items 2–3 were not done: the contact reading shows the clip's timing is the
defect, and rewriting `jog_mps` with it would make the gait worse. Stopped at the material deviation; Q-S36-P4b-1.

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

All runs local, pinned Godot 4.7.2-stable official (`ed1daf0bf`), macOS, through `cast-slots/with-slot.sh`; evidence
under `artifacts/p4b/` while the session ran (deleted after; numbers copied here).

| Commit | Implementation | Validation | Review | Evidence |
| --- | --- | --- | --- | --- |
| 1 | [x] | [x] | [x] | street run below (7 735 → 6 failing foot-frames); every edited script loads in Godot (a headless `load()` + `can_instantiate()` of the eight files: all ok). nightly `clients` dispatch of `1b34fe71` (run 39508188632): `slice --drive` **PASS** on Linux and macOS (the player walks the street and into the café over the new steps), `--world --link` PASS; the layer then stopped at `--world --target` ("no person perceived on the street"), which fails the same way on another lane's run of the same day (39504825584), so `--world --geometry`, `--world --bodies` and the feet commands did not run: **INCONCLUSIVE**. The `clients` layer's failures (`client_2d`, `client_settings`) and `farfield-windows` also fail on `main`'s nightly 39445579609: not this PR |
| 2 | [x] | [x] | [x] | the stride reading reproduces every manifest's recorded `walk_mps`/`jog_mps` within 0.002 m/s (e.g. hana-sato 1.070/2.687 against 1.068/2.685); the contact reading table below. `jog_mps` **not** rewritten: stopped at a material deviation (F-P4b-2, Q-S36-P4b-1) |
| 3 | [x] | [x] | [x] | `clients-probes` comment; the street run stays `--record-only` (F-P4b-3); this ledger |

### 7.1 The street (AC-1): `cast.py feet --run … --scene street`, the 22 bodies of `ground.bodies()`

| | failing planted foot-frames (of 4 243 frames) | where |
| --- | --- | --- |
| #407 head (`95521f1a`) | 7 735 | every body, every straddle: the drawn kerbstone had no collider (feet stood on the hidden wedge) and the drawn gutter course (top 38 mm) had none either (the −37…−40 mm sinks) |
| commit 1 without the pole change | 17 | dev-raman, nils-aaltonen (5 frames each, one station), ines-duarte (1 frame), the walker |
| commit 1 | **6** | the walker only (vera-lindgren, the listing's first body): a 3-frame touchdown sink (−18.8 … −24.6 mm) stepping from the pavement onto the kerbstone, and three single walk-cycle frames 0.3–0.4 mm over `floor.float.planted` walking along the kerbstone |

All 22 bodies standing across the kerb at three stations: **PASS** (0 failing foot-frames; worst planted sink −1.9 mm,
worst planted float 3.5 mm, any frame). One body walking over and along it: **FAIL**, 6 foot-frames — the flat-walk touchdown class
that Q-S36-P4-3 keeps record-only on the courses (§9.2 of #407, residual (a)).

### 7.2 The jog (Q-S36-P4-5): `measure_stride.gd -- --character <id>`, the 21 cast manifests

Contact reading / stride reading (m/s); planted share of foot-frames at the stride reading → at the contact reading.

| body | walk stride | walk contact | jog stride (= `jog_mps`) | jog contact | jog planted share |
| --- | --- | --- | --- | --- | --- |
| alba-ferreira | 1.042 | 1.163 | 2.638 | 5.965 | 0.00 → 0.15 |
| alice-moreau | 1.018 | 1.159 | 2.576 | 5.976 | 0.00 → 0.15 |
| bob-achterberg | 1.103 | 1.221 | 2.760 | 6.272 | 0.00 → 0.14 |
| bruno-lindqvist | 1.043 | 1.302 | 2.632 | 5.988 | 0.00 → 0.15 |
| carol-mensah | 0.985 | 1.073 | 2.481 | 5.673 | 0.00 → 0.14 |
| celia-okafor | 1.029 | 1.182 | 2.575 | 5.892 | 0.00 → 0.15 |
| dario-russo | 1.089 | 1.143 | 2.741 | 6.186 | 0.00 → 0.16 |
| dev-raman | 1.042 | 1.062 | 2.652 | 5.856 | 0.00 → 0.12 |
| elin-berg | 0.886 | 0.921 | 2.229 | 5.035 | 0.00 → 0.14 |
| erin-walsh | 0.987 | 1.110 | 2.548 | 5.921 | 0.00 → 0.13 |
| felix-okafor | 1.098 | 1.229 | 2.773 | 6.387 | 0.00 → 0.15 |
| finn-halvorsen | 1.071 | 0.999 | 2.698 | 6.108 | 0.00 → 0.13 |
| grace-liu | 0.967 | 1.060 | 2.437 | 5.577 | 0.00 → 0.15 |
| greta-novak | 0.955 | 1.170 | 2.416 | 5.473 | 0.00 → 0.16 |
| hana-sato | 1.070 | 1.233 | 2.687 | 6.226 | 0.00 → 0.14 |
| hugo-marchetti | 1.000 | 1.230 | 2.491 | 5.681 | 0.00 → 0.14 |
| ines-duarte | 0.995 | 1.065 | 2.482 | 5.678 | 0.00 → 0.14 |
| ivan-petrov | 1.017 | 1.249 | 2.565 | 5.862 | 0.00 → 0.15 |
| nils-aaltonen | 1.130 | 1.392 | 2.856 | 6.539 | 0.00 → 0.15 |
| otto-brandt | 1.047 | 1.167 | 2.655 | 6.045 | 0.00 → 0.14 |
| wes-calloway | 1.164 | 1.009 | 2.953 | 6.584 | 0.00 → 0.12 |

vera-lindgren (the reference body, `JOG_CLIP_MPS`) and the town body were not measured: the tool builds the town body
(`vitruvian.glb`), which is not imported in this checkout. Hana-sato's left foot over one jog cycle (120 samples):
down for samples 2–22 (17 % of the cycle), moving back at a steady 5.9–6.6 m/s against the body.

### 7.3 Findings

- **F-P4b-1. More drawn surfaces had no collider.** The gutter courses (top 38 mm) and the zebra bars (16 mm) were
  drawn walking surfaces without colliders: the cause of #407's −37…−40 mm street sinks. Given colliders (bounded:
  the same class as the kerbstone). The closed profile door's step had neither collider nor wedge; it gets both,
  with walk aids on its sides as well (a door step stands free on three sides), so it is no new obstacle along the
  pavement. The kerbstone stands 40 mm proud of the pavement, which its old comment called flush; the drawn
  geometry is kept and the comment corrected.
- **F-P4b-2. The jog clip cannot be calibrated by a speed (material).** Measured on its contact point, every body's
  jog wants 5.0–6.6 m/s at rate 1, 2.25–2.30 × the stride reading. The clip's foot is down for about 14 % of its
  cycle per foot (a natural jog: 35–40 %), and sweeps about 0.96 m back in that time. No playback rate gives both a
  planted foot and a plausible gait: with `jog_mps` at the contact reading, `set_gait`'s walk/jog switch moves to
  about 3.6 m/s, so the player's 3.1 m/s jog plays the **walk** clip at 2.9 × its rate; forcing the jog clip there
  plays it at half rate (64 steps per minute, slow motion). The defect is in the clip's timing, not in its speed
  constant. Stopped before rewriting `jog_mps`; Q-S36-P4b-1.
- **F-P4b-3. The street stays record-only.** Every straddle passes, but the walker's 6 touchdown foot-frames would
  turn the nightly red; they are the course's residual class, which Q-S36-P4-3 keeps record-only.
- **F-P4b-4. The knee pole (bounded correction).** The 5-frame −47…−136 mm sinks of a raised standing foot came from
  `FootGrounding`'s pole following the animated bend of a near-straight leg (noise below about 1 cm): the knee
  flipped and `LimitAngularVelocityModifier3D` then held the leg for several frames. The pole now leans 5 cm
  forward of the bend (`POLE_FORWARD_M`); a bent knee keeps its animated plane.

## 8. Questions for the operator

| Id | Question | Recommendation |
| --- | --- | --- |
| Q-S36-P4b-1 | The jog's foot-slip is the clip's timing (stance 14 % of the cycle), not its speed constant (F-P4b-2). Fix it in the clip: a measured stance-phase time warp in the animation tooling that stretches each foot's contact to the duty its stride implies (about 39 % here) and compresses the swing, keeping the cycle length — so the contact reading equals the stride reading and `jog_mps` stays as measured; or replace the clip with a recorded jog; or, last, IK foot locking for the jog? | The time warp, as a follow-up PR on the animation tooling (generic: driven by the contact reading this PR adds, no per-body number); a recorded jog if the warped clip reads wrong in review |
| Q-S36-P4b-2 | The street walker's 6 touchdown foot-frames (F-P4b-3): keep the street run record-only with the courses until the step/kerb-clip follow-up of Q-S36-P4-3, or gate the straddles alone now (a segment filter in `cast.py feet`)? | Gate the straddles alone in the follow-up that gates the courses; record-only until then |

**Ruled 2026-10-26 (operator, "all pending questions as recommended", relayed by the coordinator).** Q-S36-P4b-1:
the jog's stance-phase time warp, as recommended, is a follow-up PR on the animation tooling. Q-S36-P4b-2: the
street run stays record-only until that follow-up. Both are **known issues deferred to after the milestone**.
