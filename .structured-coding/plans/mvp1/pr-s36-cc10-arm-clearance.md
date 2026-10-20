# PR S36-CC10 — The arm–body check by winding number: CC-10's slab outline replaced by G-9w over the stand

**Lifecycle:** `DESIGN FROZEN 2026-10-20 (operator approved in the primary session: every question answered as recommended) — 2026-10-20, revision 2` (awaiting the operator's freeze). No implementation before a
`DESIGN FROZEN` header from the operator and a filled contract (§12).

**Parent:** [`step-36-interpenetration.md`](step-36-interpenetration.md) §2 objective O-3 ("no limb goes into the
trunk beyond tolerance, at rest and in every sampled frame"). **Operator directive** (relayed by the coordinator,
S36 follow-up, 2026-10-20): replace CC-10 with `mwcast/contact.py`'s libigl winding-number check; fail only on real
interpenetration beyond the tolerance file's bound. **Revision 2** (lead follow-up, 2026-10-20): the coordinator's
finding that C-1 likely misses an arm pushed through the flank, the same band defect S36-P2 fixed for G-9w, is folded
in as one shared trunk-band definition (F-CC10-7, commit 1).

**Depends on:** PR S26-c-pose merged (#371, `d6e2dfb9`: the 100STYLE idle on all 22 bodies, CC-10 record-only);
S36-P2 merged (#378, `22302866`: the `MWCF` frame dump, `mwcast/frames.py`, G-9w in `mwcast/gait.py`).

**Audit base:** `origin/main @ 16f19d3f` (2026-10-20, after #371 and #378).

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
but only to walk and jog frames, and only against the torso between the Spine joint and the neck. A hanging hand at
hip height is in neither.

A second defect sits under both. C-1 builds its torso surface from faces with at most 5 % limb weight. On a fused
body that band has holes at the flanks, where the arm's weights blend into the torso. An arm pushed through the flank
lands in the hole and reads 0 mm. P2 measured this for G-9w and fixed it there (faces with at most 25 % limb weight);
C-1 still has the old band.

After this PR:

1. **One trunk band.** C-1, G-9w and the new arm check build the torso surface with one function, P2's fixed band.
   An arm pushed through the flank fails C-1 at rest, as it fails G-9w in motion.
2. **G-9w judges the rest frame and the stand, and the arms are tested against the hips and thighs too.** The arms'
   target surface becomes the body core: torso, hips and thighs. The legs keep the torso test.
3. **CC-10 becomes the CI re-check of the rest and stand depths** against the tolerance file, like CC-9 for the
   feet. An arm resting against the hip passes (depth 0, or within the bound). A hand 8 mm inside the thigh fails,
   naming the body, the clip, the frames, the group and the bound key.
4. **Walk and jog arm readings stay recorded, not judged.** The operator is replacing the walking arm pose
   (Q-S36P2-2); this PR records the walk and jog depths against the new surface but does not gate them.
5. **The slab outline is deleted** from `stand_gates.gd`, with `ground.arm_clearance_mm` in every record.

## 2. Audit (`origin/main @ 16f19d3f`)

### 2.1 CC-10 and its records

| Fact | Where |
| --- | --- |
| CC-10's instrument: per Stand frame, body vertices (`BODY` = Hips, Spine, Chest, UpperChest, both UpperLeg) in 2 cm slabs (`SLAB_M`), a 48-bin polar outline (`BINS`) per slab; arm vertices = forearm beyond half its length from the elbow (`FOREARM_FROM` 0.5) and the hand up to `HAND_M` 0.20 m past the wrist, each within `LIMB_R` 0.08 m of its segment; clearance = distance to slab centroid − outline radius, in mm | `tools/character_pipeline/godot/stand_gates.gd` 38–49, 289–333 |
| `CLEAR_MM := 20.0` is declared but no code compares against it; `clearance_mm` is in `recorded_only` | `stand_gates.gd` 39, 413 |
| A fused pack's core (`pack_core`) and any `Pack` mesh are left out of the outline (an accessory is not the body) | `stand_gates.gd` 59–63, 71, 108–118; `godot/pose_sheet.gd` 703; `mwcast/ground.py` 365–376 |
| The worst clearance per side is written to `ground.arm_clearance_mm` | `pose_sheet.gd` 720; `mwcast/ground.py` 46; PIPELINE.md 629 |
| CC-10 in CI checks only that a clearance number is present on each gated side; `ARM_CLEAR_MM = 20.0` is declared and unused | `scripts/check_cast.py` 350, 368–404 |
| PIPELINE §15.2 describes the clearance as "recorded, not gated, until the 22-body table sets their bounds" | `tools/character_pipeline/PIPELINE.md` 572, 615–629 |
| S26-c-pose F-2: "Clearance is not a usable bound for real mocap … A true contact measure (signed distance, S36's libigl checks) is the right instrument" | `.structured-coding/plans/mvp0/pr-s26c-natural-pose.md` 189–192 |
| PRESENTATION.md still says `check_cast.py` holds every gated arm to "CC-10 clearance" bounds | `clients/3d-spike/PRESENTATION.md` 96–98 |
| Recorded clearances, mm [left, right] (unchanged by #378's re-record): vera-lindgren (reference) −54.9/−41.4; alba-ferreira −62.7/−100.9; alice-moreau −67.0/−95.0; bob-achterberg −91.8/−103.6; bruno-lindqvist −51.9/−67.6; carol-mensah −60.8/−100.6; celia-okafor −53.2/−68.4; dario-russo −91.8/−97.8; dev-raman −67.8/−92.1; elin-berg −56.0/−68.8; erin-walsh −60.8/−67.5; felix-okafor −68.5/−100.5; finn-halvorsen −62.3/−71.5; grace-liu −123.5/−175.0; greta-novak −120.1/−145.7; hana-sato −62.6/−86.8; hugo-marchetti −75.8/−82.5; ines-duarte −70.1/−71.0; ivan-petrov −84.6/−86.0; nils-aaltonen −56.9/−77.5; otto-brandt −61.2/−76.9; wes-calloway −75.3/−98.4 | `clients/3d-spike/assets/characters/cast/index.json` 118–121; each `presentation/mineworld-default/3D/characters/cast/<id>/character.yaml` 132 |
| The 22 bodies `cast.py ground` enumerates: the index's reference body (vera-lindgren, grip side named) and the 21 cast manifests | `mwcast/ground.py` 93 |
| The Stand loop runs every frame of the clip and calls `gates.frame` on each | `pose_sheet.gd` 853 (`_ground`) |
| `pack_core` is also a bind-time input (rigid weights of the pack), unrelated to CC-10 | `blender/fit_bind.py` 42; `blender/weights_pack.py` 27 |

### 2.2 C-1, G-9w and the trunk band after #378

| Fact | Where |
| --- | --- |
| `trunk_patch` (P1): faces whose three vertices are dominated by Spine, Chest or UpperChest with limb share ≤ `MIX_W` 0.05 | `mwcast/contact_gates.py` 203–210 |
| `trunk_shell` (P2): the same with limb share ≤ `SHELL_LIMB_SHARE` 0.25. Its comment: P1's band "is open at the flanks … (generalized winding number 0.18–0.44 at the protagonist's trunk axis, under the 0.5 threshold), and an arm forced through the flank is never 'inside' it" | `contact_gates.py` 213–228 |
| `limb_depths(…, shell=False)` picks `trunk_patch` by default and `trunk_shell` only when `shell=True`; C-1's `limb_in_trunk` calls it with the default, so **C-1 still uses the open band**; G-9w's `limb_frames` passes `shell=True` | `contact_gates.py` 275, 315–328; `mwcast/gait.py` 392–410 |
| P2's ledger item 9: the planted arm read 0 mm against P1's band; with Hips/Neck/shoulders added, "the hem and deltoid wrap round a hanging arm (protagonist standing 47 mm, wes 23–32 mm, both visibly clear)"; Spine/Chest/UpperChest at ≤ 0.25 caught the plant at 120–128 mm and kept clean walk frames at 0. "C-1 likely shares the flank weakness — handed to the S36 lane" | `.structured-coding/plans/mvp1/pr-s36-p2-gait-contact.md` 297–302 |
| C-1's own tests: the protagonist (`meshy_d.glb`) passes at rest; a generated open trunk box with pure Spine weights and a hand box sunk 8 mm fails, 4 mm passes. No test has trunk faces with blended limb weights, so none can see the flank hole | `tools/character_pipeline/tests/test_contact_gates.py` 22–70 |
| `limb_queries` keeps only vertices between the Spine joint and 5 cm below the Neck (`_trunk_range`, `NECK_MARGIN_M`) | `contact_gates.py` 63, 153–197 |
| `contact.inside` returns the indices and depths of points with winding number > 0.5; the closest face from `igl.point_mesh_squared_distance` is discarded | `mwcast/contact.py` 141–160 |
| `gate_g9w(…, clips=("walk", "jog"))`: rest and stand frames are not judged; Stand frames are full only every 10th (`STAND_FULL_EVERY := 10`) | `gait.py` 418; `pose_sheet.gd` 57, 891 |
| G-9w is recorded but **not judged** in CI: "not judged here while the standing and walking arm pose is being redesigned (S36-P2 operator question Q-S36P2-2)"; P2's run read 11 of 22 bodies at 5.4–35.1 mm in the walk/jog swing | `scripts/check_cast.py` 319–323; `pr-s36-p2-gait-contact.md` 303–307 |
| PIPELINE §15 states the shell, the flank defect and the hem wrap (40–50 mm on a hanging arm with the Hips added) | `PIPELINE.md` 416–430 |
| Tolerance keys `self.limb_in_trunk` (5 mm, "an arm resting against the flank") and `self.hand_in_trunk` (5 mm, "the hand on the hip or the belly"), both `inherited` from G-9; the loader refuses any key not in `KEYS` | `contact_tolerances.json` 28–39; `mwcast/tolerances.py` 24–38 |
| `ground_findings.json` lets CC-9 excuse a recorded reading up to its depth + 0.1 mm, and fails a stale one | `check_cast.py` 200, 216–334 |
| libigl 2.6.3 is locked; its Python binding offers `signed_distance(…, SIGNED_DISTANCE_TYPE_PSEUDONORMAL)` | `uv.lock` 350–352; `igl/pyigl_core.pyi` (installed package) |

### 2.3 Findings

- **F-CC10-1. CC-10 is not a gate today, and its number cannot become one.** Nothing fails on it (`CLEAR_MM`,
  `ARM_CLEAR_MM` are unused). A star-shaped outline per slab overstates "inside" wherever any body part bulges
  beyond the arm in the same direction (a flared coat, the buttock behind a hand). A bound on it would fail all 44
  arms. So the number is retired, not re-bounded.
- **F-CC10-2. G-9w does not see the stand's contact.** Its query range starts at the Spine joint, and its surface is
  the torso only. A relaxed hand hangs at hip and upper-thigh height, below that range and beside surfaces it does
  not include. Extending G-9w to the stand clip alone would pass every body without looking at the hand. Both the
  surface and the range must grow for the arms.
- **F-CC10-3. The open hem is the measured hazard.** P2 added the Hips to the torso and read the protagonist's
  hanging arm 47 mm and wes-calloway's 23–32 mm "inside", both visibly clear. Near a large open boundary the winding
  number can exceed 0.5 for a point outside. Two standard answers: (a) the body core adds the thighs, which closes most
  of the hem's opening; (b) a point counts as inside only when libigl's pseudonormal sign (the closest feature's
  angle-weighted normal, Bærentzen & Aanæs 2005) also says "inside". A real interpenetration is inside by both
  tests; a point beside an open hem fails the second. Commit 2 measures this on those two bodies first, and stops if
  it does not hold (§8).
- **F-CC10-4. No new instrument is needed.** C-1 and G-9w already hold the query filters, the surface builder, the
  inside test, the bounds, the per-frame verdict, the records and the findings file. This PR adds one surface (the
  core), one height range, one sign confirmation and one tolerance key, and removes one band.
- **F-CC10-5. The pack exclusion was an outline workaround.** The outline left out a carried pack because the pack
  widened the outline. A winding number does not have that problem: a hand outside the pack is outside it, and a hand
  inside the pack is a clip the player sees. Q-CC10-3 asks whether the pack counts.
- **F-CC10-6. The grip arm was exempt from pose gates, not from contact.** CC-11's exemption exists because the
  protagonist's strap hand is posed by her grip, not by the clip. A hand inside the chest is a clip either way. G-9w
  already tests both arms of the protagonist. Q-CC10-4 asks to keep that.
- **F-CC10-7. C-1 keeps the flank hole P2 fixed for G-9w.** C-1 calls `limb_depths` with `shell=False`, so it uses
  `trunk_patch` (≤ 0.05 limb share). On the protagonist that band reads winding number 0.18–0.44 at the trunk axis
  (P2's measurement): an arm through the flank is "outside". C-1 passes a bind-pose GLB whose arm cuts through the
  flank. Two definitions of "the torso" for one question is also a maintenance trap. The fix is one band
  (`trunk_shell`) for every caller and the `shell` switch removed (Q-CC10-7).

## 3. Scope

**In:** `mwcast/contact_gates.py` (one trunk band: `trunk_patch` and the `shell` switch removed, every caller on
`trunk_shell`; `core_shell`, `arm_queries`, `arm_depths`); `mwcast/contact.py` (`inside` gains the closest face and an
optional pseudonormal confirmation); `tools/character_pipeline/contact_tolerances.json` and `mwcast/tolerances.py`
(one key); `mwcast/gait.py` (G-9w over rest, stand, walk and jog; arms against the core); `godot/pose_sheet.gd`
(`STAND_FULL_EVERY`, the plant, the clearance plumbing removed); `godot/stand_gates.gd` (the slab outline removed);
`mwcast/ground.py` (records; `_pack_core` removed); `scripts/check_cast.py` (CC-10 judges the rest and stand arms;
walk and jog stay presence-only); the recorded `ground` and `stand_gates` blocks of all 22 bodies;
`ground_findings.json` if a body fails at rest or standing; PIPELINE.md §§15, 15.1, 15.2, 18; PRESENTATION.md
96–98; tests.

**Recorded, not judged (unchanged policy):** the arm readings in walk and jog frames. They are measured against the
new surface and written to the block, and CC-10 checks only that they are present, as CC-9 does today
(Q-S36P2-2: the walking arm pose is being replaced). Gating them is a later ruling, not this PR.

**Out:** the legs' torso test (unchanged); CC-11 (ROM, forearm rotation; unchanged); the palm and reach recordings
(unchanged, still record-only); any pose change, any fix to a body that fails (a failing body is a recorded finding
for the cast lane); `fit.yaml`'s `pack_core` (still a bind input); P3's W-4w; any Meshy or paid call.

## 4. Definitions

- **Trunk band** (one definition, every caller): the faces whose three vertices are each dominated by Spine, Chest or
  UpperChest with limb share ≤ `SHELL_LIMB_SHARE` 0.25 (P2's `trunk_shell`). C-1, G-9w's leg test and the core all
  use it.
- **Arm groups:** `upper_arm` (distal half, P1's `PROXIMAL_T`), `sleeve` (forearm) and `hand`, per side, chosen by
  `limb_queries`' dominance, seam-share and segment-radius filters, on the outermost layer (P2's `outermost`).
- **Body core:** the trunk band, plus the faces whose three vertices are each dominated by Hips, LeftUpperLeg or
  RightUpperLeg. On a layered outfit, the union of every part's such faces (P2's rule for the shell).
- **Core range:** along Hips→Neck, from 5 cm above the higher knee joint (`LowerLeg`) to 5 cm below the Neck
  (`NECK_MARGIN_M`, mirrored at the knee as `KNEE_MARGIN_M` = 0.05 m, where the thigh faces end). It replaces
  `_trunk_range`'s lower end for the arm groups only. It is a filter (which vertices to ask about), not a bound.
- **Inside the core:** winding number > 0.5 (`contact.INSIDE_THRESHOLD`) **and** pseudonormal sign negative
  (`igl.signed_distance`, `SIGNED_DISTANCE_TYPE_PSEUDONORMAL`). The depth is the distance to the closest point.
- **Bound per vertex:** by the region of the vertex's closest core face. A torso or Hips face: `self.hand_in_trunk`
  for the hand, `self.limb_in_trunk` otherwise (their rationales already name the flank and the hip). A thigh face:
  the new key `self.arm_in_thigh` (commit 2).

## 5. Commits

### Commit 1 — one trunk band for C-1 and G-9w; the flank plant

**Goal.** One definition of the torso surface. An arm through the flank fails C-1, as it fails G-9w.

**Scope.**

- `contact_gates.py`: `trunk_patch` and `limb_depths`' `shell` parameter removed; `limb_depths` always builds
  `trunk_shell`. `limb_in_trunk` (C-1) and `gait.limb_frames` (G-9w) both call it without a switch. The comment above
  `SHELL_LIMB_SHARE` names all callers.
- C-1's messages keep their shape ("trunk patch N of M faces" becomes "trunk band N of M faces").
- PIPELINE.md §18's C-1 row: the band's limb share 0.05 → 0.25, with F-CC10-7's reason.

**Implementation.**

- [ ] The band unified, the switch and `trunk_patch` removed (pyright strict, no ignores).
- [ ] The planted-flank test helper and the tests below.

**Validation (`tests/test_contact_gates.py`).**

- **Generated flank plant (synthetic).** A fused torso box whose flank faces carry 10–20 % upper-arm weight, as a
  Meshy body's do at the armpit seam, and a forearm box pushed 20 mm through that flank. Before this commit (the old
  band): C-1 PASS, 0 mm, because the flank faces are not in the band. After: C-1 FAIL, 20.0 mm ±0.5, naming
  `Left sleeve` and `self.limb_in_trunk`.
- **Real-body flank plant.** The protagonist's bind GLB (`meshy_d.glb`) with the left upper arm turned toward the
  body about the shoulder joint (each vertex moved by its own upper-arm-chain weight, linear-blend style), until the
  elbow lies 25 mm inside the flank. Before: C-1 reads ≤ 5 mm (PASS). After: C-1 FAIL ≥ 20 mm. The same body unposed
  still PASSes C-1 with its margins printed.
- The existing C-1 cases (pure-Spine box: 8 mm FAIL, 4 mm PASS) are unchanged and green.
- **Mutation 1:** restore `trunk_patch` for C-1 → both flank plants pass (the test turns red).
- `vitruvian.glb` (no pure-trunk face under the old band): the new band's result is recorded in §10.

**Acceptance.** Both flank plants fail C-1 after the commit and pass before it; the unposed protagonist passes;
G-9w's results on its own tests are unchanged (it already used this band).

**Commit boundary.** One band, C-1 corrected. Nothing else changes.

### Commit 2 — the core surface, the sign confirmation, the key (Python only)

**Goal.** One function answers "how deep is each arm group inside the body core?", proven on generated bodies with
known answers, including an open flared hem.

**Scope.**

- `contact.py`: `inside(outer, points, confirm_sign: bool = False) -> Inside`. `Inside` gains `faces` (the closest
  face per inside point). With `confirm_sign`, a point whose pseudonormal sign is not negative is dropped. The default
  path is unchanged, so C-1, C-2 and the legs' G-9w keep their results. `contact.py` stays the only importer of
  `igl` (I-S36-1). If P3's `contact.closest` has landed, `inside` reuses it rather than calling libigl twice.
- `contact_gates.py`: `core_shell(skin, faces) -> Ints` (the trunk band plus the Hips and thigh faces);
  `arm_queries(skin, joints) -> dict[str, Ints]` (the arm groups of `limb_queries` with the core range);
  `arm_depths(skin, faces, joints, tolerances, keep) -> list[LimbDepth]` (each vertex's bound key by §4's region rule;
  the group's verdict is its worst vertex against that vertex's bound).
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

- [ ] `inside(…, confirm_sign)` and `Inside.faces`.
- [ ] `core_shell`, `arm_queries`, `arm_depths`, `KNEE_MARGIN_M`.
- [ ] The key in the file and in `KEYS`; `test_tolerances.py` covers its absence.

**Validation (`tests/test_arm_core.py`, synthetic, deterministic).**

- **Generated bodies (the second body set).** A builder makes a skinned capsule body: torso, hips, two thighs, two
  arms hanging at a given gap, labelled like a real dump, with the flank seam weights of commit 1. Twelve bodies from
  seed `"s36-cc10"` vary trunk radius (120–180 mm), hip width, thigh radius (70–110 mm), arm abduction and the hand's
  gap to the thigh from +10 mm to −12 mm. Expected: every body whose deepest arm vertex is ≤ 5 mm inside passes; every
  deeper one fails with its planted depth ±0.5 mm, naming side, group and `self.arm_in_thigh` (or
  `self.hand_in_trunk` at the hip).
- **The open hem.** One generated body wears a flared open skirt weighted to the Hips, ending 60 mm below the hand
  with a 40 mm annulus between hem and thigh, and the hand 15 mm outside the skirt. With `confirm_sign`: PASS. A
  hand planted 8 mm through the skirt: FAIL 8 mm.
- **Mutation 2:** `core_shell` reduced to the trunk band → the thigh plants pass (the test turns red).
- **Mutation 3:** `confirm_sign=False` in `arm_depths` → the open-hem body fails (the test turns red).

**Acceptance.** Every generated case ends as stated; both mutations turn the suite red; C-1's tests stay green.

**Commit boundary.** Functions, key, tests. Nothing calls `arm_depths` yet.

### Commit 3 — G-9w over rest, stand, walk and jog; the arms against the core

**Goal.** One per-frame rule for the arms in every clip the probe plays; judged at rest and standing, recorded in
walk and jog.

**Scope.**

- `gait.py`: `gate_g9w(frames, tolerances, clips=("rest", "stand", "walk", "jog"), judged=("rest", "stand"))`. Per
  full frame: the leg groups against the trunk band (`limb_depths`, unchanged); the arm groups against the core
  (`arm_depths`). A failing reading in a judged clip fails the gate: "G-9w Right hand, stand frames 120–141, 7.2 mm
  inside the core (bound 5 mm, self.arm_in_thigh)". A reading in walk or jog is reported with the same message,
  prefixed "recorded:", and does not change the verdict.
- The block per body: `arms: {<clip>: {"limb_mm", "hand_mm", "at", "frames", "judged"}}`, `gates["G-9w"]`,
  `tolerances_sha256`. It replaces `limb_in_trunk_mm` / `limb_in_trunk_at` (the walk and jog numbers keep their
  meaning, now per clip).
- `pose_sheet.gd`: `STAND_FULL_EVERY` 10 → 3 (Q-CC10-2). About 126 full Stand frames, about 75 MB of scratch per
  body, deleted once read (`ground.run`).
- `mwcast/pose_sheet.py`: `checks.torso_clip` per captured frame from the same function (P2's pattern).

**Implementation.**

- [ ] `gate_g9w` with the two readings, the judged/recorded split, the per-clip block and the messages.
- [ ] The cadence; `ground.run` records the block.

**Validation.**

- Synthetic `MWCF` (`tests/mwcf_fixture.py`): a stand frame with the hand 7 mm inside a thigh → FAIL naming
  `stand`, the frame and `self.arm_in_thigh`; 3 mm → PASS. A walk frame with the forearm 6 mm inside the flank →
  verdict PASS, reported as "recorded:" with `self.limb_in_trunk`.
- **Stop rule, first real run** (through `with-slot.sh`, three bodies: vera-lindgren, wes-calloway, grace-liu — the
  two bodies on which P2 saw the hem wrap, and the worst CC-10 reading): every inside reading is listed with its
  winding number, its sign and the frame. If a visibly clear hanging arm still reads deeper than its bound, the
  implementation stops and returns to the operator with those frames (a material deviation; no filter is tuned to
  hide it, I-CC10-2).
- `--plant-arm-into-torso` still fails every frame, depth ≥ 120 mm (P2 read 120–128 mm).

**Acceptance.** The three bodies are judged on every full rest and stand frame and recorded in walk and jog; the
torso plant still fails; the stop rule's evidence is in §10.

**Commit boundary.** G-9w per clip with the core. Records not yet rewritten.

### Commit 4 — CC-10 re-checks the arms; the slab outline is deleted

**Goal.** CI holds every body to its recorded rest and stand arm depths, and no outline code is left.

**Scope.**

- `scripts/check_cast.py`: CC-10 becomes "every body's `ground.arms` block, for each judged clip (rest, stand), is
  within the tolerance file's bound for its key, and was measured under the current file (`tolerances_sha256`); the
  walk and jog entries are present". It reads the bounds from `contact_tolerances.json` with the standard library
  (CC-9's pattern), and excuses a reading only through `ground_findings.json` (up to the recorded depth + 0.1 mm; a
  finding that now passes is stale and fails). CC-9's presence check on `limb_in_trunk_mm` moves into CC-10's walk
  and jog presence check. `ARM_CLEAR_MM` and the "no clearance measurement" check go. `--self-test` gains: a
  standing hand 7 mm over its bound (FAIL CC-10), the same depth in walk (PASS), a stale hash (FAIL CC-10), a passing
  block (PASS), a recorded finding exceeded by 0.2 mm (FAIL).
- `stand_gates.gd`: the clearance part of `frame()`, `_bin`, `CLEAR_MM`, `SLAB_M`, `BINS`, `FOREARM_FROM`, `LIMB_R`,
  `HAND_M`, the `pack_core` parameter and its vertex filter (108–118), and `clearance_mm` from `_keep`, `block` and
  `recorded_only`. The thigh grouping stays (the reach reads the thigh's front), and so does the `Pack` mesh skip
  (71), which only keeps the pack out of the palm and reach readings.
- `pose_sheet.gd`: `arm_clearance_mm` and the `pack_core` argument (703, 720) removed. `ground.py`: `RECORDED` loses
  `arm_clearance_mm`; `_pack_core` removed.

**Implementation.**

- [ ] CC-10 rewritten with its self-test cases.
- [ ] The outline and its plumbing removed from GDScript and Python.

**Validation.**

- `python3 scripts/check_cast.py --self-test` passes, and each new case fails or passes for the stated reason.
- **Mutation 4:** CC-10's judged clips widened to walk → the walk case fails (red self-test); CC-10's comparison
  given the bound + 10 mm → the 7 mm standing case passes (red self-test).
- `grep -n "clearance\|SLAB_M\|BINS" tools/character_pipeline/godot/stand_gates.gd` prints nothing.

**Acceptance.** CC-10 fails on a rest or stand depth over its bound, on a stale hash and on an exceeded finding, and
does not fail on walk or jog depths; nothing of the outline remains.

**Commit boundary.** The CI rule and the deletion. The records are rewritten in commit 5, so this head's
`check_cast.py` run on the old records fails by design until then (recorded in the ledger, not pushed alone).

### Commit 5 — the generated variant, all 22 bodies, records and specification

**Goal.** Run on every body the client draws, and prove the gate on a pose no one tuned it on.

**Scope.** `pose_sheet.gd` `--plant-arm-adduct <deg>`: a `SkeletonModifier3D` after `Posture` and `GroundContact`
that rotates each `UpperArm` toward the body by `deg` about the chest's forward axis, on every frame. It is a test
plant; `ground.py` passes it through and never writes a planted run to a record.

**Runs (each a full-cast run through `with-slot.sh`; two runs in total):**

- **Unplanted:** all 22 bodies — vera-lindgren (reference, grip left), alba-ferreira, alice-moreau, bob-achterberg,
  bruno-lindqvist, carol-mensah, celia-okafor, dario-russo, dev-raman, elin-berg, erin-walsh, felix-okafor,
  finn-halvorsen, grace-liu, greta-novak, hana-sato, hugo-marchetti, ines-duarte, ivan-petrov, nils-aaltonen,
  otto-brandt, wes-calloway — rest, stand, walk and jog. Written with `cast.py ground --write`. C-1 at rest
  (`cast.py contact`) on each body's bind GLB with the unified band, recorded beside it.
- **Generated variant:** the same 22 bodies with `--plant-arm-adduct 8`. An 8° adduction moves a hand at about
  0.6 m from the shoulder about 80 mm toward the midline, so every body must fail G-9w on both arms in the stand,
  naming the hand or sleeve and the core. The run is judged, not recorded.

**Records.**

- [ ] The `ground` and `stand_gates` blocks of the 21 manifests and the reference body's index entry rewritten
      (no `arm_clearance_mm`, `clearance_mm`; the `arms` block added).
- [ ] `ground_findings.json`: any body that fails at rest or standing, with clip, frames, group and depth, lane
      "cast lane / NPC lane". Any body whose C-1 now fails with the unified band, likewise in this ledger. No number
      goes into code (I-CC10-2).
- [ ] PIPELINE.md §15 (the band, the core), §15.1 (the `arms` block; walk and jog recorded), §15.2 (the clearance
      bullet replaced by the arms rule; recording lines), §18 (C-1 row: the unified band; the core, the range, the
      sign confirmation, `self.arm_in_thigh`); PRESENTATION.md 96–98.
- [ ] Step 36: §2 O-3 gains "and the arms against the hips and thighs, standing included"; §3.3 gains the key.
- [ ] This ledger, §10: per body and side, the worst depth per clip and its bound key, before (CC-10 mm) → after;
      C-1 before → after.

**Commit boundary.** Evidence and specification.

## 6. Acceptance criteria (PR level)

1. **One trunk band.** C-1, G-9w and the core build the torso surface with one function. The synthetic and the
   real-body (`meshy_d.glb`) flank plants fail C-1 after this PR and passed it before; mutation 1 turns the suite
   red.
2. **All 22 bodies** listed in commit 5 are judged by G-9w on every full rest and stand frame, both arms, with every
   margin printed. Each passes, or fails as a recorded finding in `ground_findings.json` with clip, frames, group and
   depth. Their walk and jog arm readings are recorded, not judged.
3. **Generated variant on real bodies:** with `--plant-arm-adduct 8` every one of the 22 bodies fails G-9w in the
   stand on both arms against the core.
4. **Second body set:** the twelve seeded generated bodies (`"s36-cc10"`) and the open-hem body end exactly as
   commit 2 states, and mutations 2 and 3 turn the suite red.
5. `--plant-arm-into-torso` still fails every frame (≥ 120 mm).
6. CC-10 in CI fails a rest or stand block over its bound, a block measured under another tolerance file, and an
   exceeded or stale finding; it does not fail on walk or jog depths (self-test and mutation 4).
7. No slab, outline or clearance code remains in `stand_gates.gd`, `pose_sheet.gd`, `ground.py` or `check_cast.py`;
   no bound literal for the arm test anywhere but the tolerance file (I-S36-2).
8. The legs' G-9w gives the same results as before this PR.
9. `fast` is green on the final head.

## 7. Invariants

I-S36-1 … I-S36-5 (step §5), and:

- **I-CC10-1.** One rule decides arm interpenetration in every clip: inside the core by winding number and
  pseudonormal sign, deeper than the bound of the closest face's region. No clip, body or side has its own rule.
  Which clips CI judges is policy (rest and stand now), not a second rule.
- **I-CC10-2.** No per-body number in code or in a filter. A body that fails is recorded in `ground_findings.json`,
  never excused by a constant or a changed filter.
- **I-CC10-3.** The filters (`MIX_W`, `LIMB_R`, `PROXIMAL_T`, `HAND_REACH_M`, `NECK_MARGIN_M`, `KNEE_MARGIN_M`,
  `SHELL_LIMB_SHARE`) choose which vertices are asked about and which faces form a surface; they never decide a
  verdict.
- **I-CC10-4.** One trunk band. No gate builds its own torso surface.

## 8. Risks

| Risk | Effect | Mitigation |
| --- | --- | --- |
| The Hips faces wrap a flared hem round a hanging arm (P2 measured 47 mm and 23–32 mm on clear arms) | a hand beside the coat reads inside | the thighs close the opening and the pseudonormal sign must agree (F-CC10-3); commit 3's stop rule runs the two bodies first and returns to the operator if it does not hold |
| The pseudonormal sign is wrong where faces are inverted | a real clip missed | P2 dumps faces counter-clockwise (ledger item 8); the torso plant and the adduction plant must still fail on all 22 bodies |
| The wider band changes C-1 on a shipped body | a new C-1 failure at rest | that is the defect being fixed; it is recorded as a finding with depth, not hidden |
| Every 3rd Stand frame misses a peak between samples | a brief clip missed | the idle moves slowly (weight shifts over seconds); the worst depth's frame and its neighbours are listed, and Q-CC10-2 offers every frame |
| A real hand rests 6–10 mm into a soft Meshy thigh on several bodies | many findings | that is the gate working; the findings go to the cast lane with frames; the bound is not raised |

## 9. Proposed edits to shared documents

- Step 36 §2 (O-3) and §3.3 (the key), commit 5.
- PIPELINE.md §§15, 15.1, 15.2, 18 and PRESENTATION.md 96–98, commits 1 and 5.
- S26-c-pose F-2 and S36-P2 ledger item 9 stay as written; this PR's ledger links back to both.

## 10. Ledger

| Commit | Implementation | Validation | Review | Evidence |
| --- | --- | --- | --- | --- |
| 1 | [x] `trunk_patch` and `limb_depths(shell=)` removed; C-1 and G-9w both build `trunk_shell` (≤ 0.25); C-1 prints "trunk band N of M faces"; PIPELINE §15 / §18 C-1 row | [x] `pytest test_contact_gates.py test_gait.py` 20 passed. Synthetic flank (15 % `LeftArm` on the +x flank, forearm 20 mm through it): old band PASS 0.0 mm (mutation run) → FAIL "Left sleeve is 20.0 mm inside the trunk (4 vertices over 5 mm)", `self.limb_in_trunk`. Real-body flank (`meshy_d.glb`, left arm turned in about the shoulder, LBS by chain weight, until the elbow joint is ≥ 25 mm inside the band, reached at 46°): prototype read old band 0.0 mm sleeve / 0.0 mm upper arm at every angle 0–60° → new band ≈ 87–105 mm, FAIL. Unposed protagonist PASS, band 14018 of 95000 faces (old 6555), every group 0.0 mm, margin 5.0 mm. **Mutation 1** (band limb share back to `MIX_W`): both flank tests red (2 failed, 5 passed). `vitruvian.glb`: still "cannot look: no trunk-dominant face" under the new band (its weights put no face on Spine/Chest/UpperChest). G-9w tests (`test_gait.py`) unchanged and green. ruff, ruff format, pyright strict: 0 errors | [x] diff read: only C-1's band changes; G-9w already passed `shell=True`, so its results are identical; `MIX_W` still filters query vertices (I-CC10-3) | local runs above |
| 2 | [x] `contact.inside(…, confirm_sign)` (pseudonormal via `igl.signed_distance`, only for winding-inside points) and `Inside.faces`; `core_shell` (band ∪ Hips/UpperLeg vertices; a face is kept when all three vertices are in that set, so the trunk–Hips seam faces close the waist: bounded discovery, the design's "plus the faces" would leave the seam ring open), `arm_queries` (core range, `KNEE_MARGIN_M` 0.05), `arm_depths` (closest-face region: a face with ≥ 2 thigh-dominant vertices → `self.arm_in_thigh`); `LimbDepth.surface` ("trunk"/"core") in its messages; `self.arm_in_thigh` in the file and `KEYS` | [x] `test_arm_core.py` 16 passed: the twelve seeded bodies (`"s36-cc10"`, gaps +10…−12 mm, 4 at the hip) end as planted (≤ 5 mm PASS at the planted depth ±0.5; deeper FAIL at the planted depth, 4 vertices over, `self.arm_in_thigh` / `self.hand_in_trunk` at the hip); every unplanted group 0.0 mm. **Mutation 2** (core = trunk band only): 8 failed. **Mutation 3** (`confirm_sign=False`): the cloaked bodies fail (2 failed). `test_tolerances.py` covers the key's absence. C-1/C-2/G-9w/contact/items tests unchanged (63 passed; `test_contact_items` prop-library case fails identically on origin/main: `football` has no mount, unrelated). ruff, pyright strict: 0 errors | [x] **F-CC10-8 (bounded discovery, recorded):** the design's literal open hem (a hand 15 mm outside a flared skirt, hem 60 mm below) cannot read inside: a planar hole seen from above its plane only lowers the winding number (measured −0.19 beside the skirt, +0.09–0.12 below the hem, never > 0.5), so it cannot redden mutation 3. It is kept as a test (outside with and without the sign). The disagreement the sign answers is a distant sheet wrapping an arm while the closest surface faces it from outside: a cloak from the chest over both arms, open 60 mm below the hand. Without the sign every hand vertex reads inside (winding ≈ 1, FAIL 15+ mm); with it PASS; a cloaked hand 8 mm into the thigh FAILs 8.0 mm. No invariant, contract or scope change | local runs above |
| 3 | [x] `gate_g9w(frames, tol, clips=("rest","stand","walk","jog"), judged=("rest","stand"))`: per full frame the arm groups against the core (`arm_depths`), the leg groups against the band (`limb_depths`, unchanged); judged clips fail, walk/jog lines go to `looked_at` prefixed "recorded:"; block `{"arms": {clip: {limb_mm, hand_mm, keys, at, frames, judged}}}` replaces `limb_in_trunk_mm`/`_at`; `ground.RECORDED` gains `arms`; `pose_sheet.py` bound `arm_thigh_mm`; `STAND_FULL_EVERY` 10 → 3 | [x] Synthetic MWCF (fixture gains thighs and forearms): stand hand 7 mm in the thigh → FAIL "G-9w Left hand, stand frame 10, 7.0 mm inside the core (bound 5 mm, self.arm_in_thigh)"; 3 mm → PASS; walk forearm 6 mm in the flank → PASS with "recorded: G-9w Left sleeve, walk frame 5, 6.0 mm inside the core (bound 5 mm, self.limb_in_trunk)"; P2's walking hand 8 mm → recorded, PASS. `test_gait/test_pose_sheet/test_ground` 23 passed. **Stop-rule run** (`with-slot.sh`, `cast.py ground --only vera-lindgren wes-calloway grace-liu --keep-dumps`; 132 full stand frames each): G-4w PASS on all three; rest PASS on all three; **stand FAIL on all three**: worst limb / hand mm vera 49.5 / 14.5, grace 36.9 / 75.9, wes 103.4 / 63.0, on (nearly) every sampled stand frame. Every reading on stand:99: winding 0.52–0.97 and the pseudonormal sign negative (winding-only vertices: 1 per upper arm; the sign changes almost nothing here); closest faces Spine/Chest/UpperChest (upper arm, forearm at the flank) and UpperLeg (forearm and hand at the hip/thigh; wes's and grace's thigh-dominant faces reach the waist and the coat front). **Not a false reading:** horizontal slices of stand:99 (`slice_png.py`, the core's cut against the arm's cut) show the skinned arm's cross-section crossing the body outline (vera's right forearm at y 1.10 about half inside the flank; wes's arms at y 1.13 mostly inside the body outline); the hanging elbow axis lies at the body's own lateral extent (vera: elbow x −0.193, body to −0.192 at y 1.13). So the stop rule (a *visibly clear* arm reading deep) did not trigger: the readings are real interpenetration of the fused meshes in the 100STYLE idle, and proceed as findings (§8 risk row 5). `--plant-arm-into-torso` (pose sheet, protagonist): every frame FAIL, 120.8–130.1 mm on the standing/walking frames; the two seated frames read 18.7 mm (Left upper_arm against the core) and fail too | [x] diff read: legs unchanged (filtered from the same `limb_depths` call); one rule for every clip, the judged set is a parameter (I-CC10-1); no per-body number (I-CC10-2) | `cast-scratch-s36cc10/three.log`, `ps-plant/report.json` (local) |
| 4 | [x] CC-10 rewritten (`arm_failures`: rest and stand judged against each reading's recorded key's bound from `contact_tolerances.json`, stdlib; hash check; walk/jog presence; `arms.*` findings in `ground_findings.json` up to + 0.1 mm, stale fails; CC-9 leaves `arms.*` findings to CC-10); `ARM_CLEAR_MM` and the clearance presence check gone; `stand_failures` no longer takes the ground block. `stand_gates.gd`: clearance, `_bin`, `CLEAR_MM`, `SLAB_M`, `BINS`, `FOREARM_FROM`, `LIMB_R`, `HAND_M`, `BODY`, the `pack_core` parameter and filter, `clearance_mm` removed (forearm vertices no longer skinned; the `Pack` mesh skip and the thigh grouping stay). `pose_sheet.gd`: `arm_clearance_mm` and the `pack_core` argument removed; `ground.py`: `arm_clearance_mm` out of `RECORDED`, `_pack_core` removed | [x] `check_cast.py --self-test`: 33 cases as expected (standing hand 7 mm over → FAIL naming `self.arm_in_thigh`; same depth in walk → PASS; missing jog reading → FAIL; other hash → FAIL; finding at the depth → PASS, 0.2 mm under it → FAIL; stale finding → FAIL; CC-9 ignores an `arms.*` finding). **Mutation 4a** (`ARMS_JUDGED` + walk): self-test red. **Mutation 4b** (bound + 10 mm): self-test red. `grep -n "clearance\|SLAB_M\|BINS" stand_gates.gd`: nothing. `check_cast.py` on this head's old records fails CC-9/CC-10 "measured under other bounds" by design (the tolerance file gained a key; records rewritten in commit 5; this head is not pushed alone) | [x] diff read: no bound literal in `check_cast.py` (keys read from the file, I-S36-2) | local runs above |
| 5 | [ ] | [ ] | [ ] | — |

## 11. Questions for the operator

**Ruled 2026-10-20 (operator, primary session):** every question below is answered as recommended.

| Id | Question | Recommendation |
| --- | --- | --- |
| Q-CC10-1 | Build on S36-P2's dump and G-9w (merged, #378) instead of a standalone instrument? | Yes. P2 already dumps the frames and runs the winding-number test per frame; a standalone check would duplicate it |
| Q-CC10-2 | Judge every 3rd Stand frame (about 126 frames, about 75 MB scratch per body) rather than P2's every 10th, or every frame (about 230 MB)? | Every 3rd: the idle is slow, and the worst frame's neighbours are listed |
| Q-CC10-3 | Count a carried pack as part of the body for this test (a hand inside the pack fails), dropping the outline's pack exclusion? | Yes. The exclusion was a workaround for the outline; a hand in a pack is a visible clip |
| Q-CC10-4 | Test both of the protagonist's arms, including the strap hand that CC-11 exempts? | Yes. The exemption is for the pose, not for interpenetration; G-9w already tests both |
| Q-CC10-5 | Add `self.arm_in_thigh` (5 mm, inherited from G-9) for an arm against the thigh, and use the existing trunk keys at the torso and hip? | Yes. The trunk keys' reasons already name the flank and the hip; the thigh needs its own key so it can be tuned alone |
| Q-CC10-6 | Count a point as inside only when the winding number and libigl's pseudonormal sign agree, for the arms against the core? | Yes. It answers the open-hem false reading P2 measured, and a real clip is inside by both |
| Q-CC10-7 | Give C-1 the same trunk band as G-9w (limb share ≤ 0.25 instead of ≤ 0.05), as one shared definition, so an arm through the flank fails C-1 at rest? | Yes. P2 measured the 0.05 band open at the flanks (an arm through it read 0 mm); one torso definition for one question |

The walk and jog arm readings stay recorded, not judged, under the existing ruling Q-S36P2-2; this PR asks nothing
new about them.

## 12. Execution contract (filled at freeze)

```text
PROJECT / PR:          S36-CC10 — the arm–body check by winding number; one trunk band; CC-10's outline retired
PRIMARY DESIGN DOC:    .structured-coding/plans/mvp1/pr-s36-cc10-arm-clearance.md
RELATED / BINDING:     step-36-interpenetration.md; pr-s36-p1-contact-module.md; pr-s36-p2-gait-contact.md
                       (ledger items 8-10); mvp0/pr-s26c-natural-pose.md (F-2); PIPELINE.md §§15, 15.1, 15.2, 18
IMPLEMENTATION BASE:   origin/main at freeze (contains #371 and #378)
APPROVED SCOPE:        §3
FROZEN INVARIANTS:     I-S36-1 … I-S36-5; I-CC10-1 … I-CC10-4
SEQUENCE:              commits 1-5
VALIDATION BUDGET:     Godot only through cast-slots/with-slot.sh: one three-body run (commit 3) and two full-cast
                       runs (commit 5); no Meshy call; no money
LIVE DOCUMENTATION:    this file, §10
```
