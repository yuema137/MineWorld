# PR S36-CC10 — The arm–body check by winding number: CC-10's slab outline replaced by G-9w over the stand

**Lifecycle:** `DRAFT — 2026-10-20` (awaiting the operator's freeze). No implementation before a `DESIGN FROZEN`
header from the operator and a filled contract (§11).

**Parent:** [`step-36-interpenetration.md`](step-36-interpenetration.md) §2 objective O-3 ("no limb goes into the
trunk beyond tolerance, at rest and in every sampled frame"). **Operator directive** (relayed by the coordinator,
S36 follow-up, 2026-10-20): replace CC-10 with `mwcast/contact.py`'s libigl winding-number check; fail only on real
interpenetration beyond the tolerance file's bound.

**Depends on:** PR S26-c-pose merged (#371, `d6e2dfb9`: the 100STYLE idle on all 22 bodies, CC-10 record-only);
**S36-P2 merged** (the `MWCF` frame dump, `mwcast/frames.py`, `mwcast/gait.py` G-9w). P2 is not merged today
(§2.3); this PR builds on it and adds nothing that duplicates it.

**Audit base:** `origin/main @ d6e2dfb9` (2026-10-20, after #371), plus a read-only read of the local branch
`mvp1/pr-s36-p2 @ b6b10675` (based on `5a8c6f28`, before #371).

**Design session:** worktree `/Users/yuema137/mineworld-worktrees/design-s36-cc10`, branch
`docs/pr-design-s36-cc10`.

---

## 1. Goal

CC-10 asks: "does the lower forearm or the hand clear the body by at least 20 mm while the body stands?" It answers
with a slab outline. The body's vertices are cut into 2 cm horizontal slabs. In each slab, the outline is the
furthest vertex from the slab's centre in each of 48 directions. An arm vertex's clearance is its distance from the
centre minus the outline's radius in its direction.

That instrument cannot tell "touching" from "inside". Example. With the recorded idle, Grace Liu's right hand hangs
beside her hip, and her coat flares past the hand a little lower in the same slab. In the hand's direction the
outline's radius is the coat's flare, so the hand reads 175 mm "inside" the body, although no hand vertex is under
any surface. Every arm of all 22 bodies reads −41.4 to −175.0 mm this way (§2.1). So S26-c-pose made CC-10
record-only (its finding F-2) and named the right instrument: a true contact measure.

The right question is: "is any arm vertex under the body's surface, and how deep?" The generalized winding number
answers "under the surface?" (about 1 inside, about 0 outside, also at open holes), and the distance to the surface
answers "how deep?". S36-P1 built that answer (`contact.inside`, C-1 at rest). S36-P2 applies it per frame (G-9w),
but only to walk and jog frames and only against the torso between the Spine joint and the neck. A hanging hand at
hip height is in neither.

After this PR:

1. **G-9w covers the stand and the rest frame, and the arms are tested against the hips and thighs too.** The arms'
   target surface becomes the body core: torso, hips and thighs. The legs keep P2's torso test.
2. **CC-10 becomes the CI re-check of those recorded depths** against the tolerance file, like CC-9 for the feet.
   An arm resting against the hip passes (depth 0, or within the bound). A hand 8 mm inside the thigh fails, naming
   the body, the clip, the frames, the group and the bound key.
3. **The slab outline is deleted** from `stand_gates.gd`, with `ground.arm_clearance_mm` in every record.

## 2. Audit

### 2.1 On `origin/main @ d6e2dfb9`

| Fact | Where |
| --- | --- |
| CC-10's instrument: per Stand frame, body vertices (`BODY` = Hips, Spine, Chest, UpperChest, both UpperLeg) in 2 cm slabs (`SLAB_M`), a 48-bin polar outline (`BINS`) per slab; arm vertices = forearm beyond half its length from the elbow (`FOREARM_FROM` 0.5) and the hand up to `HAND_M` 0.20 m past the wrist, each within `LIMB_R` 0.08 m of its segment; clearance = distance to slab centroid − outline radius, in mm | `tools/character_pipeline/godot/stand_gates.gd` 38–49, 289–333 |
| `CLEAR_MM := 20.0` is declared but no code compares against it; `clearance_mm` is in `recorded_only` | `stand_gates.gd` 39, 413 |
| A fused pack's core (`pack_core`) and any `Pack` mesh are left out of the outline (an accessory is not the body) | `stand_gates.gd` 59–63, 71, 108–118; `pose_sheet.gd` 677; `mwcast/ground.py` 202–213 |
| The worst clearance per side is written to `ground.arm_clearance_mm` | `pose_sheet.gd` 690; `mwcast/ground.py` 37; PIPELINE.md 548 |
| CC-10 in CI checks only that a clearance number is present on each gated side; `ARM_CLEAR_MM = 20.0` is declared and unused | `scripts/check_cast.py` 224, 242–278 |
| PIPELINE §15.2 describes the clearance as "recorded, not gated, until the 22-body table sets their bounds" | `tools/character_pipeline/PIPELINE.md` 491, 534–541 |
| S26-c-pose F-2: "Clearance is not a usable bound for real mocap … A true contact measure (signed distance, S36's libigl checks) is the right instrument" | `.structured-coding/plans/mvp0/pr-s26c-natural-pose.md` 189–192 |
| PRESENTATION.md still says `check_cast.py` holds every gated arm to "CC-10 clearance" bounds | `clients/3d-spike/PRESENTATION.md` 96–98 |
| Recorded clearances, mm [left, right]: vera-lindgren (reference) −54.9/−41.4; alba-ferreira −62.7/−100.9; alice-moreau −67.0/−95.0; bob-achterberg −91.8/−103.6; bruno-lindqvist −51.9/−67.6; carol-mensah −60.8/−100.6; celia-okafor −53.2/−68.4; dario-russo −91.8/−97.8; dev-raman −67.8/−92.1; elin-berg −56.0/−68.8; erin-walsh −60.8/−67.5; felix-okafor −68.5/−100.5; finn-halvorsen −62.3/−71.5; grace-liu −123.5/−175.0; greta-novak −120.1/−145.7; hana-sato −62.6/−86.8; hugo-marchetti −75.8/−82.5; ines-duarte −70.1/−71.0; ivan-petrov −84.6/−86.0; nils-aaltonen −56.9/−77.5; otto-brandt −61.2/−76.9; wes-calloway −75.3/−98.4 | `clients/3d-spike/assets/characters/cast/index.json` 47–50; each `presentation/mineworld-default/3D/characters/cast/<id>/character.yaml` 61 |
| The 22 bodies `cast.py ground` enumerates: the index's reference body (vera-lindgren, grip side named) and the 21 cast manifests | `mwcast/ground.py` 52–61 |
| The Stand loop runs every frame of the clip (`Human._stand_clip().length × FPS`) and calls `gates.frame` on each | `pose_sheet.gd` 822–856 |
| C-1 (P1): at bind pose, limb query vertices inside the **trunk patch** (faces with all three vertices dominated by Spine, Chest or UpperChest and limb share ≤ 0.05) deeper than `self.limb_in_trunk` / `self.hand_in_trunk` fail | `mwcast/contact_gates.py` 203–210, 221–249 |
| C-1's query filter `limb_queries` keeps only vertices between the Spine joint and 5 cm below the Neck along Hips→Neck (`_trunk_range`, `NECK_MARGIN_M`) | `contact_gates.py` 63, 153–158, 161–197 |
| `contact.inside` returns the indices and depths of points with winding number > 0.5; the distance comes from `igl.point_mesh_squared_distance`, whose closest-face output is discarded | `mwcast/contact.py` 141–160 |
| Tolerance keys `self.limb_in_trunk` (5 mm, "an arm resting against the flank") and `self.hand_in_trunk` (5 mm, "the hand on the hip or the belly"), both `inherited` from G-9's `CLIP_MM` | `tools/character_pipeline/contact_tolerances.json` 28–39 |
| The tolerance loader refuses any key not in `KEYS` | `mwcast/tolerances.py` 24–38 |
| `pack_core` is also a bind-time input (rigid weights of the pack), unrelated to CC-10 | `blender/fit_bind.py` 42; `blender/weights_pack.py` 27 |
| libigl 2.6.3 is locked; its Python binding offers `signed_distance(…, SIGNED_DISTANCE_TYPE_PSEUDONORMAL)` | `uv.lock` 350–352; `igl/pyigl_core.pyi` (installed package) |

### 2.2 On `mvp1/pr-s36-p2 @ b6b10675` (read, not touched)

| Fact | Where |
| --- | --- |
| The `MWCF` v1 dump: labels (dominant bone, trunk share, limb share, mesh, region, under flag, layer), faces, and per frame the bone origins and either every vertex (`full`) or the ground set | `mwcast/frames.py`; `godot/pose_sheet.gd` `_dump_frame` |
| Stand frames are full only every 10th frame (`STAND_FULL_EVERY := 10`); every walk and jog frame is full | `pose_sheet.gd` 57, 895 |
| `gate_g9w(frames, tolerances, clips=("walk", "jog"))`: the stand and rest frames are not judged | `mwcast/gait.py` 418 |
| G-9w's target is `trunk_shell`: trunk-dominant faces with limb share ≤ `SHELL_LIMB_SHARE` 0.25, which closes the flank. The comment records that adding the Hips, the Neck or the shoulders "wraps the hem and the deltoid round a hanging arm (measured on the protagonist, wes-calloway and the planted arm)" | `mwcast/contact_gates.py` (branch) `trunk_shell`, `SHELL_LIMB_SHARE` |
| G-9w queries through `limb_queries` (the same Spine…Neck − 5 cm range) on the outermost layer only (`outermost`) | `mwcast/gait.py` 372–410 |
| The block records `limb_in_trunk_mm` {limb, hand}, where, and the frame count; `ground_findings.json` lets CC-9 excuse a recorded reading up to its recorded depth (+0.1 mm), and fails a stale one | `gait.py` 457–463; `tools/character_pipeline/ground_findings.json` |
| The branch is based on `5a8c6f28`; #371 changed `pose_sheet.gd`, `ground.py` and the records since, so P2 must merge main before it lands | `git merge-base` |

### 2.3 Findings

- **F-CC10-1. CC-10 is not a gate today, and its number cannot become one.** Nothing fails on it (`CLEAR_MM`,
  `ARM_CLEAR_MM` are unused). A star-shaped outline per slab overstates "inside" wherever any body part bulges
  beyond the arm in the same direction (a flared coat, the buttock behind a hand). A bound on it would fail all 44
  arms. So the number is retired, not re-bounded.
- **F-CC10-2. G-9w as P2 builds it does not see the stand's contact.** Its query range starts at the Spine joint,
  and its surface is the torso only. A relaxed hand hangs at hip and upper-thigh height, below that range and beside
  surfaces it does not include. Extending G-9w to the stand clip alone would pass every body without looking at the
  hand. Both the surface and the range must grow for the arms.
- **F-CC10-3. The open hem is the known hazard.** P2 measured that a trunk surface that includes the Hips wraps a
  flared hem round a hanging arm. Near a large open boundary the winding number can exceed 0.5 for a point that is
  outside. Two things answer this, both standard: (a) the body core adds the thighs, which closes most of the hem's
  opening; (b) a point counts as inside only when libigl's pseudonormal sign (the closest feature's angle-weighted
  normal, Bærentzen & Aanæs 2005) also says "inside". A real interpenetration is inside by both tests; a point beside
  an open hem fails the second.
- **F-CC10-4. No new instrument is needed.** C-1 and G-9w already hold the query filters (`limb_queries`), the
  surface builder (`trunk_patch`, `trunk_shell`), the inside test (`contact.inside`), the bounds (tolerance file), the
  per-frame verdict, the records and the findings file. This PR adds one surface (the core), one height range, one
  sign confirmation, and one tolerance key.
- **F-CC10-5. The pack exclusion was an outline workaround.** The outline left out a carried pack because the pack
  widened the outline. A winding number does not have that problem: a hand outside the pack is outside it, and a hand
  inside the pack is a clip the player sees. Q-CC10-3 asks whether the pack counts.
- **F-CC10-6. The grip arm was exempt from pose gates, not from contact.** CC-11's exemption exists because the
  protagonist's strap hand is posed by her grip, not by the clip. A hand inside the chest is a clip either way. P2's
  G-9w already tests both arms of the protagonist. Q-CC10-4 asks to keep that.

## 3. Scope

**In:** `mwcast/contact.py` (`inside` gains the closest face and an optional pseudonormal confirmation);
`mwcast/contact_gates.py` (`core_shell`, `arm_queries`, `arm_depths`); `tools/character_pipeline/contact_tolerances.json`
and `mwcast/tolerances.py` (one key); `mwcast/gait.py` (G-9w over rest, stand, walk and jog; arms against the core);
`godot/pose_sheet.gd` (`STAND_FULL_EVERY`, the plant, the clearance plumbing removed); `godot/stand_gates.gd` (the slab
outline removed); `mwcast/ground.py` (records; `_pack_core` removed); `scripts/check_cast.py` (CC-10 re-judges the
arms block); the recorded `ground` and `stand_gates` blocks of all 22 bodies; `ground_findings.json` if a body fails;
PIPELINE.md §§15, 15.2, 18; PRESENTATION.md 96–98; tests.

**Out:** C-1's behaviour at bind pose (unchanged, its tests stay green); the legs' torso test (P2's, unchanged);
CC-11 (ROM, forearm rotation; unchanged); the palm and reach recordings (unchanged, still record-only); any pose
change, any fix to a body that fails (a failing body is a recorded finding for the cast lane); `fit.yaml`'s
`pack_core` (still a bind input); P3's W-4w; any Meshy or paid call.

## 4. Definitions

- **Arm groups:** `upper_arm` (distal half, P1's `PROXIMAL_T`), `sleeve` (forearm) and `hand`, per side, chosen by
  `limb_queries`' dominance, seam-share and segment-radius filters, on the outermost layer (P2's `outermost`).
- **Body core:** the faces whose three vertices are each dominated by Spine, Chest or UpperChest with limb share ≤
  `SHELL_LIMB_SHARE` (P2's trunk shell), or by Hips, LeftUpperLeg or RightUpperLeg. On a layered outfit, the union of
  every part's such faces (P2's rule for the shell).
- **Core range:** along Hips→Neck, from 5 cm above the higher knee joint (`LowerLeg`) to 5 cm below the Neck
  (`NECK_MARGIN_M`, mirrored at the knee as `KNEE_MARGIN_M` = 0.05 m, where the thigh faces end). It replaces
  `_trunk_range`'s lower end for the arm groups only. It is a filter (which vertices to ask about), not a bound.
- **Inside the core:** winding number > 0.5 (`contact.INSIDE_THRESHOLD`) **and** pseudonormal sign negative
  (`igl.signed_distance`, `SIGNED_DISTANCE_TYPE_PSEUDONORMAL`). The depth is the distance to the closest point.
- **Bound per vertex:** by the region of the vertex's closest core face. A torso or Hips face: `self.hand_in_trunk`
  for the hand, `self.limb_in_trunk` otherwise (their rationales already name the flank and the hip). A thigh face:
  the new key `self.arm_in_thigh` (§5, commit 1).

## 5. Commits

### Commit 1 — the core surface, the sign confirmation, the key (Python only)

**Goal.** One function answers "how deep is each arm group inside the body core?", proven on generated bodies with
known answers, including an open flared hem.

**Scope.**

- `contact.py`: `inside(outer, points, confirm_sign: bool = False) -> Inside`. `Inside` gains `faces` (the closest
  face per inside point). With `confirm_sign`, a point whose pseudonormal sign is not negative is dropped. The default
  path is unchanged, so C-1, C-2 and G-9w's leg test keep their results. `contact.py` stays the only importer of
  `igl` (I-S36-1). If P3's `contact.closest` has landed, `inside` reuses it rather than calling libigl twice.
- `contact_gates.py`: `core_shell(skin, faces) -> Ints`; `arm_queries(skin, joints) -> dict[str, Ints]` (the arm
  groups of `limb_queries` with the core range); `arm_depths(skin, faces, joints, tolerances, keep) ->
  list[LimbDepth]` (each depth's `bound_key` chosen per vertex by §4's region rule; the group's verdict is its worst
  vertex against that vertex's bound).
- `contact_tolerances.json` and `tolerances.py` `KEYS`: `self.arm_in_thigh`:

  ```json
  "self.arm_in_thigh": {
    "bound_mm": 5,
    "rationale": "A relaxed arm hangs with the hand and forearm resting on the side of the thigh (the 100STYLE Neutral idle shows it on every body, S26-c-pose F-2). Skin, fat and cloth compress a few millimetres under that light contact, as at the flank; deeper is a visible clip.",
    "source": "step-26 §6.1 G-9 (pose_sheet.gd CLIP_MM = 5), the same basis as self.limb_in_trunk, extended from the trunk to the thigh by PR S36-CC10; no soft-tissue compression measurement was read",
    "status": "inherited"
  }
  ```

**Implementation.**

- [ ] `inside(…, confirm_sign)` and `Inside.faces` (pyright strict, no ignores).
- [ ] `core_shell`, `arm_queries`, `arm_depths`, `KNEE_MARGIN_M`.
- [ ] The key in the file and in `KEYS`; `test_tolerances.py` covers its absence.

**Validation (`tests/test_arm_core.py`, synthetic, deterministic).**

- **Generated bodies (the second body set).** A builder makes a skinned capsule body: torso, hips, two thighs, two
  arms hanging at a given gap, labelled like a real dump. Twelve bodies from seed `"s36-cc10"` vary trunk radius
  (120–180 mm), hip width, thigh radius (70–110 mm), arm abduction and the hand's gap to the thigh from +10 mm to
  −12 mm. Expected: every body whose deepest arm vertex is ≤ 5 mm inside passes; every deeper one fails with its
  planted depth ±0.5 mm, naming side, group and `self.arm_in_thigh` (or `self.hand_in_trunk` at the hip).
- **The open hem.** One generated body wears a flared open skirt weighted to the Hips, ending 60 mm below the hand
  with a 40 mm annulus between hem and thigh, and the hand 15 mm outside the skirt. With `confirm_sign`: PASS. A
  hand planted 8 mm through the skirt: FAIL 8 mm.
- **Mutation 1:** `core_shell` reduced to `trunk_shell` → the thigh plants pass (the test turns red).
- **Mutation 2:** `confirm_sign=False` in `arm_depths` → the open-hem body fails (the test turns red).

**Acceptance.** Every generated case ends as stated; both mutations turn the suite red; C-1's tests are green and
unchanged.

**Commit boundary.** Functions, key, tests. Nothing calls `arm_depths` yet.

### Commit 2 — G-9w over rest, stand, walk and jog; the arms against the core

**Goal.** One per-frame rule for the arms in every clip the probe plays.

**Scope.**

- `gait.py`: `gate_g9w(frames, tolerances, clips=("rest", "stand", "walk", "jog"))`. Per full frame: the leg groups
  against the trunk shell (P2's `limb_depths`, unchanged); the arm groups against the core (`arm_depths`). The verdict
  names clip, frames, group, depth and bound key: "G-9w Right hand, stand frames 120–141, 7.2 mm inside the core
  (bound 5 mm, self.arm_in_thigh)".
- The block per body: `arms: {<clip>: {"limb_mm", "hand_mm", "at", "frames"}}`, `gates["G-9w"]`,
  `tolerances_sha256`. It replaces P2's `limb_in_trunk_mm` / `limb_in_trunk_at` (the same numbers, now per clip).
- `pose_sheet.gd`: `STAND_FULL_EVERY` 10 → 3 (Q-CC10-2). About 126 full Stand frames, about 75 MB of scratch per
  body, deleted once read (P2's `ground.run`).
- `mwcast/pose_sheet.py`: `checks.torso_clip` per captured frame from the same function (P2's pattern).

**Implementation.**

- [ ] `gate_g9w` with the two readings, the per-clip block and the messages.
- [ ] The cadence; `ground.run` records the block.

**Validation.**

- Synthetic `MWCF` (P2's `tests/mwcf_fixture.py`): a stand frame with the hand 7 mm inside a thigh → FAIL naming
  `stand`, the frame and `self.arm_in_thigh`; 3 mm → PASS. A walk frame with the forearm 6 mm inside the flank →
  FAIL with `self.limb_in_trunk`.
- One real run through `with-slot.sh` on three bodies (vera-lindgren, wes-calloway, grace-liu: the protagonist, the
  hem case P2 measured, the worst CC-10 reading): the per-clip block, and each group's worst depth, recorded in §10.
- `--plant-arm-into-torso` (P2's plant) still fails every frame, depth ≥ 150 mm.

**Acceptance.** The three bodies are judged on every full frame of four clips; the torso plant still fails.

**Commit boundary.** G-9w per clip with the core. Records not yet rewritten.

### Commit 3 — CC-10 re-checks the arms; the slab outline is deleted

**Goal.** CI holds every body to its recorded arm depths and bounds, and no outline code is left.

**Scope.**

- `scripts/check_cast.py`: CC-10 becomes "every body's `ground.arms` block, for every clip, is within the tolerance
  file's bound for its key, and was measured under the current file (`tolerances_sha256`)". It reads the bounds from
  `contact_tolerances.json` with the standard library (CC-9's pattern), and excuses a reading only through
  `ground_findings.json` (P2's rule: up to the recorded depth + 0.1 mm; a finding that now passes is stale and fails).
  `ARM_CLEAR_MM` and the "no clearance measurement" check go. `--self-test` gains: a hand 7 mm over its bound
  (FAIL CC-10), a stale hash (FAIL CC-10), a passing block (PASS), a recorded finding exceeded by 0.2 mm (FAIL).
- `stand_gates.gd`: the clearance part of `frame()`, `_bin`, `CLEAR_MM`, `SLAB_M`, `BINS`, `FOREARM_FROM`, `LIMB_R`,
  `HAND_M`, the `pack_core` parameter and its vertex filter (108–118), and `clearance_mm` from `_keep`, `block` and
  `recorded_only`. The thigh grouping stays (the reach reads the thigh's front), and so does the `Pack` mesh skip
  (71), which only keeps the pack out of the palm and reach readings.
- `pose_sheet.gd`: `arm_clearance_mm` and the `pack_core` argument (line 677, 690) removed. `ground.py`: `RECORDED`
  loses `arm_clearance_mm`; `_pack_core` removed.

**Implementation.**

- [ ] CC-10 rewritten with its self-test cases.
- [ ] The outline and its plumbing removed from GDScript and Python.

**Validation.**

- `python3 scripts/check_cast.py --self-test` passes, and each new case fails for the stated reason.
- Mutation 3: CC-10's comparison `>` changed to `>=` with the bound + 10 mm → the 7 mm case passes (red self-test).
- `grep -n "clearance\|SLAB_M\|BINS" tools/character_pipeline/godot/stand_gates.gd` prints nothing.

**Acceptance.** CC-10 fails on depth over bound, on a stale hash and on an exceeded finding; nothing of the outline
remains.

**Commit boundary.** The CI rule and the deletion. The records are rewritten in commit 4, so this head's
`check_cast.py` run on the old records fails by design until then (recorded in the ledger, not pushed alone).

### Commit 4 — the generated variant, all 22 bodies, records and specification

**Goal.** Run on every body the client draws, and prove the gate on a pose no one tuned it on.

**Scope.** `pose_sheet.gd` `--plant-arm-adduct <deg>`: a `SkeletonModifier3D` after `Posture` and `GroundContact`
that rotates each `UpperArm` toward the body by `deg` about the chest's forward axis, on every frame. It is a test
plant; `ground.py` passes it through and never writes a planted run to a record.

**Runs (each a full-cast run through `with-slot.sh`; two runs in total):**

- **Unplanted:** all 22 bodies — vera-lindgren (reference, grip left), alba-ferreira, alice-moreau, bob-achterberg,
  bruno-lindqvist, carol-mensah, celia-okafor, dario-russo, dev-raman, elin-berg, erin-walsh, felix-okafor,
  finn-halvorsen, grace-liu, greta-novak, hana-sato, hugo-marchetti, ines-duarte, ivan-petrov, nils-aaltonen,
  otto-brandt, wes-calloway — rest, stand, walk and jog. Written with `cast.py ground --write`.
- **Generated variant:** the same 22 bodies with `--plant-arm-adduct 8`. An 8° adduction moves a hand at about
  0.6 m from the shoulder about 80 mm toward the midline, so every body must fail G-9w on both arms in the stand,
  naming the hand or sleeve and the core. The run is judged, not recorded.

**Records.**

- [ ] The `ground` and `stand_gates` blocks of the 21 manifests and the reference body's index entry rewritten
      (no `arm_clearance_mm`, `clearance_mm`; the `arms` block added).
- [ ] `ground_findings.json`: any body that fails unplanted, with clip, frames, group and depth, lane "cast lane /
      NPC lane". No number goes into code (I-CC10-2).
- [ ] PIPELINE.md §15 (checks list), §15.2 (the clearance bullet replaced by the arms rule; recording lines), §18
      (C-1/G-9w rows: the core, the range, the sign confirmation, `self.arm_in_thigh`); PRESENTATION.md 96–98.
- [ ] Step 36: §2 O-3 gains "and the arms against the hips and thighs, standing included"; §3.3 gains the key.
- [ ] This ledger, §10: per body and side, the worst depth per clip and its bound key, before (CC-10 mm) → after.

**Commit boundary.** Evidence and specification.

## 6. Acceptance criteria (PR level)

1. **All 22 bodies** listed in commit 4 are judged by G-9w on every full frame of rest, stand, walk and jog, both
   arms, with every margin printed. Each passes, or fails as a recorded finding in `ground_findings.json` with clip,
   frames, group and depth.
2. **Generated variant on real bodies:** with `--plant-arm-adduct 8` every one of the 22 bodies fails G-9w in the
   stand on both arms against the core.
3. **Second body set:** the twelve seeded generated bodies (`"s36-cc10"`) and the open-hem body end exactly as
   commit 1 states, and both mutations of commit 1 turn the suite red.
4. `--plant-arm-into-torso` still fails every frame (≥ 150 mm).
5. CC-10 in CI fails a block over its bound, a block measured under another tolerance file, and an exceeded or stale
   finding (self-test and mutation 3).
6. No slab, outline or clearance code remains in `stand_gates.gd`, `pose_sheet.gd`, `ground.py` or `check_cast.py`;
   no bound literal for the arm test anywhere but the tolerance file (I-S36-2).
7. C-1 at rest and the legs' G-9w give the same results as before this PR.
8. `fast` is green on the final head.

## 7. Invariants

I-S36-1 … I-S36-5 (step §5), and:

- **I-CC10-1.** One rule decides arm interpenetration in every clip: inside the core by winding number and
  pseudonormal sign, deeper than the bound of the closest face's region. No clip, body or side has its own rule.
- **I-CC10-2.** No per-body number in code or in a filter. A body that fails is recorded in `ground_findings.json`,
  never excused by a constant or a changed filter.
- **I-CC10-3.** The filters (`MIX_W`, `LIMB_R`, `PROXIMAL_T`, `HAND_REACH_M`, `NECK_MARGIN_M`, `KNEE_MARGIN_M`,
  `SHELL_LIMB_SHARE`) choose which vertices are asked about; they never decide a verdict.

## 8. Risks

| Risk | Effect | Mitigation |
| --- | --- | --- |
| A Meshy body's fused coat hem is weighted to the thighs, so the core wraps it | a hand beside the coat reads inside | the pseudonormal confirmation (F-CC10-3); commit 2 runs wes-calloway and grace-liu first and records each inside reading with its winding number and sign |
| The pseudonormal sign is wrong where faces are inverted | a real clip missed | P2 orients dump faces CCW (`b6b10675`); the torso plant and the adduction plant must still fail on all 22 bodies |
| Every 3rd Stand frame misses a peak between samples | a brief clip missed | the idle moves slowly (weight shifts over seconds); the worst depth's frame and its neighbours are listed, and Q-CC10-2 offers every frame |
| A real hand rests 6–10 mm into a soft Meshy thigh on several bodies | many findings | that is the gate working; the findings go to the cast lane with frames; the bound is not raised |
| P2 is delayed or changes G-9w's shape | this PR cannot start | it waits for P2 (Q-CC10-1); commit 1 is Python-only and can start on P2's merged functions alone |

## 9. Proposed edits to shared documents

- Step 36 §2 (O-3) and §3.3 (the key), commit 4.
- PIPELINE.md §§15, 15.2, 18 and PRESENTATION.md 96–98, commit 4.
- S26-c-pose F-2 stays as written; this PR's ledger links back to it.

## 10. Ledger

| Commit | Implementation | Validation | Review | Evidence |
| --- | --- | --- | --- | --- |
| 1 | [ ] | [ ] | [ ] | — |
| 2 | [ ] | [ ] | [ ] | — |
| 3 | [ ] | [ ] | [ ] | — |
| 4 | [ ] | [ ] | [ ] | — |

## 11. Questions for the operator

| Id | Question | Recommendation |
| --- | --- | --- |
| Q-CC10-1 | Build on S36-P2 (wait for it to merge on top of #371) instead of a standalone instrument? | Yes. P2 already dumps the frames and runs the winding-number test per frame; a standalone check would duplicate it |
| Q-CC10-2 | Judge every 3rd Stand frame (about 126 frames, about 75 MB scratch per body) rather than P2's every 10th, or every frame (about 230 MB)? | Every 3rd: the idle is slow, and the worst frame's neighbours are listed |
| Q-CC10-3 | Count a carried pack as part of the body for this test (a hand inside the pack fails), dropping the outline's pack exclusion? | Yes. The exclusion was a workaround for the outline; a hand in a pack is a visible clip |
| Q-CC10-4 | Test both of the protagonist's arms, including the strap hand that CC-11 exempts? | Yes. The exemption is for the pose, not for interpenetration; P2 already tests both |
| Q-CC10-5 | Add `self.arm_in_thigh` (5 mm, inherited from G-9) for an arm against the thigh, and use the existing trunk keys at the torso and hip? | Yes. The trunk keys' reasons already name the flank and the hip; the thigh needs its own key so it can be tuned alone |
| Q-CC10-6 | Count a point as inside only when the winding number and libigl's pseudonormal sign agree, for the arms against the core? | Yes. It removes the open-hem false reading P2 measured, and a real clip is inside by both |

## 12. Execution contract (filled at freeze)

```text
PROJECT / PR:          S36-CC10 — the arm–body check by winding number; CC-10's outline retired
PRIMARY DESIGN DOC:    .structured-coding/plans/mvp1/pr-s36-cc10-arm-clearance.md
RELATED / BINDING:     step-36-interpenetration.md; pr-s36-p1-contact-module.md; pr-s36-p2-gait-contact.md;
                       mvp0/pr-s26c-natural-pose.md (F-2); PIPELINE.md §§15, 15.2, 18
IMPLEMENTATION BASE:   origin/main at freeze, with #371 and S36-P2 merged
APPROVED SCOPE:        §3
FROZEN INVARIANTS:     I-S36-1 … I-S36-5; I-CC10-1 … I-CC10-3
SEQUENCE:              commits 1-4
VALIDATION BUDGET:     Godot only through cast-slots/with-slot.sh: one three-body run (commit 2) and two full-cast
                       runs (commit 4); no Meshy call; no money
LIVE DOCUMENTATION:    this file, §10
```
