# PR S36-P2 — G-4w and G-9w: sole vs floor and arm vs torso in every walk and jog frame

**Lifecycle:** `DRAFT — 2026-10-19`, awaiting the operator's freeze. No implementation before a `DESIGN FROZEN`
header and a filled contract (§9).

**Parent:** [`step-36-interpenetration.md`](step-36-interpenetration.md) §4.2. **Depends on:** S36-P1 merged
(`mwcast/contact.py`, the tolerance file); **S26-c-ground merged** (§2.2). **Audit base:** `origin/main @ b843d546`,
plus a read of the local branch `mvp0/pr-s26c-ground @ d7c08f3f` (2026-10-19).

---

## 1. Goal

Today's checks look at the feet in a handful of ways that do not generalise:

- G-4(a) looks at the rest pose only.
- S26-c-ground's probe looks at every frame, but only at the lowest point of the whole lower body, with bounds as
  GDScript constants, and it cannot tell a planted foot from a swinging one except in the jog's per-cycle minimum.
- G-9 looks at five captured frames with a slab outline.

This PR makes the walk and jog check per foot, per frame, with "planted" defined by the gait itself, and moves the
arm–torso check to the winding number. The rule is the same as P1's: Godot skins, Python decides, the bounds are data.

Example. A body whose left heel sinks 7 mm at heel strike in frame 3 of the walk, while the right foot is fine, fails
today only as "walk min −7 mm, LeftToes". After this PR it fails as "G-4w left foot, walk frame 3, planted, sink
7.0 mm (bound 5.0 mm, `floor.sink.foot`)". A body whose right foot hovers 12 mm above the floor while it should be
planted passes today (the other foot is down, so the frame's lowest point is on the floor). After this PR it fails
as "G-4w right foot, walk frames 18–24, planted, float 12.0 mm (bound 10.0 mm, `floor.float.planted`)".

## 2. Audit

### 2.1 On `origin/main`

| Fact | Where |
| --- | --- |
| The treadmill loop: `set_gait(clip speed × scale)`, wait one period, then sample every frame of one cycle (`period = round(length × 30)`), at the same point of the frame `_capture` reads | `tools/character_pipeline/godot/pose_sheet.gd` `_run`, P-2/P-3 block |
| CPU skinning of every visible skinned mesh with dominant bone, trunk and limb weight shares, mesh name, region index (UV2) and the "under" flag | `pose_sheet.gd` `_skinned` |
| Bone poses read in `skeleton_updated`, so `SkeletonModifier3D` results (Posture, later GroundContact) count | `pose_sheet.gd` `_on_skeleton_updated`, `_gpose` |
| G-9's slab test and its vertex filters (`_group`, `MIX_W`, `LIMB_R`, proximal cut-offs) | `pose_sheet.gd` `_checks` |
| W-4 uses `SLAB_M` too | `pose_sheet.gd` `_poke_through` |
| The flagged-frame red border is drawn in GDScript; `sheet.html` is written by Python from `report.json` | `pose_sheet.gd` `_capture`; `mwcast/pose_sheet.py` `write_sheet` |
| G-8 (planted foot): drift ≤ 25 % of body speed, contact ≥ 50 % | `.structured-coding/plans/mvp0/pr-s32p-wardrobe-pilot.md` §4.8 |
| Retargeted bone names: `LeftUpperLeg`, `LeftLowerLeg`, `LeftFoot`, `LeftToes` (Right likewise) | `pose_sheet.gd`, `human.gd`, `godot/character.gltf.import` |

### 2.2 On `mvp0/pr-s26c-ground` (read, not touched; it lands first)

| Fact | Where |
| --- | --- |
| `--ground-only --bodies <json>`: builds each body as the client does (`Human.build_character(manifest)`, or the reference body), headless, 30 fps | `pose_sheet.gd` `_run_ground_only` |
| `_ground_set`: vertices within `GROUND_SET_M` (0.30 m) of the lowest rest vertex, with bones, weights, side | `pose_sheet.gd` |
| `_ground_low`: the lowest skinned point overall and per side | `pose_sheet.gd` |
| `_ground`: rest; `Stand` loop (`STAND_LOOP × 30` frames); one `Walk` and one `Jog_Fwd` cycle at each body's measured clip speed; writes `min_mm`, `frame_low_max_mm`, `foot_contact_mm`, `hips_m`, `worst_sink_mm`, `worst_float_mm`, `pass` | `pose_sheet.gd` |
| Bounds `GROUND_SINK_MM` 5.0 and `GROUND_FLOAT_MM` 20.0 as GDScript constants; `SINK_MM`, `FLOAT_MM` again in Python; `GROUND_SINK_MM`, `GROUND_FLOAT_MM` again in `check_cast.py` | three files |
| `mwcast/ground.py`: `bodies()` enumerates the client index's reference bodies and every `presentation/mineworld-default/3D/characters/cast/*/character.yaml`; `run`, `table`, `write` (records the block) | new module |
| CC-9: `scripts/check_cast.py` `ground_failures` recomputes the verdict from the recorded numbers | CI's python layer |
| `GroundContact` (`clients/3d-spike/scripts/ground_contact.gd`): lifts the `Hips` so the lowest support point is on the floor; `pin` 1 when standing or walking, 0 at a jog | client |

**Consequence.** The probe already visits the right frames for every body. What it lacks is per-foot data, a planted
test, a full-body dump for the arm check, and one home for the bounds. So this PR extends the probe in place rather
than adding a second one (Q-S36-6).

## 3. Scope

**In:** `pose_sheet.gd` (frame dump; plants; slab torso test removed); `mwcast/frames.py` (new, dump reader);
`mwcast/gait.py` (new, G-4w and G-9w); `mwcast/ground.py` (decides in Python); `mwcast/pose_sheet.py` (merges G-9w
into `report.json`); `scripts/check_cast.py` (CC-9 reads the tolerance file and the new fields); the recorded
`ground` blocks of every body; PIPELINE.md §15 and §15.1; tests.

**Out:** `GroundContact` and every other client file (P4); W-4 (P3); G-13 (unchanged); new clips.

## 4. Commits

### Commit 1 — the frame dump

**Goal.** Godot writes what it skinned; Python reads it. One format for P2 and P3.

**Format `MWCF` v1** (little-endian, written with `FileAccess.store_*`):

```text
header   "MWCF" u32 version=1  u32 vertex_count  u32 face_count  u32 bone_count  u32 frame_count
bones    bone_count × (u16 length, UTF-8 name)                       — the skeleton's bone names
faces    face_count × 3 × u32                                         — global vertex indices, all meshes
labels   vertex_count × (u16 dominant bone, f32 trunk share, f32 limb share,
                         u16 mesh index, i16 region index, u8 under flag, u8 layer)
meshes   u16 count, count × (u16 length, UTF-8 name)
frames   frame_count × (u16 tag length, UTF-8 tag "walk:12", f32 floor_y, f32 speed_mps,
                        f32 fwd_x, f32 fwd_z, bone_count × 3 × f32 bone origins,
                        u8 full, full ? vertex_count × 3 × f32 : ground_count × 3 × f32)
ground   u32 ground_count, ground_count × u32 vertex index            — the `_ground_set` vertices
```

The faces and labels are written once. A frame carries either every vertex (`full`) or only the ground set.

**Size.** A cast body has up to 100 000 triangles (G-1), about 50 000 vertices: 0.6 MB per full frame. Full frames
are written for every walk and jog frame (about 70) and every 10th `Stand` frame; the other `Stand` frames carry the
ground set only. That is about 45 MB per body in scratch, deleted after the run; nothing is committed.

**Scope.** `pose_sheet.gd`: `_dump_open(path, h)`, `_dump_frame(h, tag, full)`, `_dump_close()`; flag
`--dump-frames <dir>`; called from `_ground`'s three loops and from `_capture`. `mwcast/frames.py`: `Frames` (numpy
arrays: `labels`, `faces`, `bone_names`, per-frame `tag`, `floor_y`, `speed`, `fwd`, `bones`, `positions`),
`read(path) -> Frames`, refusing a bad magic, version, truncated file or index out of range.

**Implementation.**

- [ ] Writer in GDScript, reusing `_skinned`'s per-vertex data and `_ground_set`'s indices (global index = running
      offset over meshes and surfaces, the same order `_skinned` visits).
- [ ] Faces from each surface's `Mesh.ARRAY_INDEX`, offset by the surface's first global vertex.
- [ ] Reader in Python with `numpy.frombuffer`; typed accessors; the floor is `floor_y` (the body's `global_position.y`).

**Validation.** A Python test writes a synthetic MWCF with the spec's layout (a writer helper in the test) and reads
it back; truncated, wrong-magic and bad-index files are refused by name. One real Godot run on the protagonist
(through `with-slot.sh`) writes a dump whose vertex count equals `_skinned`'s and whose rest frame's lowest foot
point equals S26-c-ground's `rest_mm` within 0.1 mm.

**Acceptance.** The reader round-trips the synthetic file exactly; the real dump agrees with the existing probe's
numbers within 0.1 mm on every frame it has in common.

**Commit boundary.** Format, writer, reader, tests. No verdict changes yet.

### Commit 2 — G-4w: per foot, per frame, planted

**Goal.** The decision moves to Python, per foot, with a planted test; the GDScript bound constants go.

**Definitions.**

- **Foot set** of a side: ground-set vertices whose dominant bone is that side's `Foot` or `Toes`. The **shin set**:
  dominant bone `LowerLeg`.
- **Foot height** in a frame: the lowest foot-set point minus `floor_y`.
- **Planted** (G-8's rule, on the treadmill): the body stays put while the floor would move backward at the gait
  speed `v`. A foot is planted in frame *k* when its `Foot` bone's horizontal velocity (central difference over the
  cyclic frames, × 30 fps) satisfies `|vel · fwd + v| ≤ 0.25 v` and `|vel × fwd| ≤ 0.25 v`. Standing, every frame is
  planted for both feet.
- **G-4w fails** when, for any foot and any frame: the foot height is below −`floor.sink.foot`; the shin set is
  below −`floor.sink.shin`; or the foot is planted and its height is above `floor.float.planted`. In a walk, also
  when a frame's lowest point is above `floor.float.walk_frame` (S26-c-ground's rule, kept). G-8's contact share
  (planted ≥ 50 % of the cycle per foot, walk) is reported, not judged here.

**Scope.** `mwcast/gait.py` (new): `planted(frames, side) -> NDArray[bool]`, `gate_g4w(frames, tolerances) ->
GateResult`; `GateId.G4W = "G-4w"` in `gates.py`. `mwcast/ground.py`: `run` passes `--dump-frames`; the block is
computed from the dumps. `pose_sheet.gd`: `_ground` still drives the frames but no longer judges (`GROUND_SINK_MM`,
`GROUND_FLOAT_MM` removed). `scripts/check_cast.py`: CC-9's bounds from `contact_tolerances.json`.

**The recorded block** keeps every field CC-9 reads today and adds:

```text
"ground": { …existing fields…,
  "feet": {"left":  {"walk": {"sink_mm", "planted_float_mm", "planted_share"}, "jog": {…}, "stand": {…}},
           "right": {…}},
  "shin_sink_mm": …,
  "gates": {"G-4w": "PASS|FAIL"},
  "tolerances_sha256": "…" }
```

CC-9 recomputes PASS/FAIL from these numbers and the tolerance file named by the hash. A block whose hash differs from
the current file fails with "measured under other bounds; re-run `cast.py ground --write`".

**Implementation.**

- [ ] `planted`, `gate_g4w` with every margin printed per foot and clip.
- [ ] `ground.py` builds the block from `gait.py`; `table()` gains per-foot columns.
- [ ] `check_cast.py` reads the bounds from the file (stdlib JSON) and checks the new fields; its `--self-test`
      covers a planted float of 12 mm (FAIL) and a stale hash (FAIL).

**Validation.**

- Synthetic dumps (test-built): a foot sinking 6 mm in one planted frame → FAIL naming side, clip and frame; 4 mm →
  PASS. A planted foot floating 12 mm for six frames → FAIL; the same lift in swing frames → PASS (it is swing).
- Mutation: replace the 0.25 drift share with 2.5 → the swing-frame case fails (everything reads as planted).

**Acceptance.** On synthetic dumps the gate fails exactly where planted. Every body in the cast passes or is
recorded as a finding with the frame and depth.

**Commit boundary.** G-4w, the block, CC-9.

### Commit 3 — G-9w: arm vs torso by winding number; the slab test retired

**Goal.** One arm–torso test, robust on fused and layered bodies, per sampled frame.

**Scope.** `mwcast/gait.py` `gate_g9w(frames, tolerances)`, reusing P1's trunk-patch and limb-query filters
(`contact_gates.py`) on each full frame; `mwcast/pose_sheet.py`: after Godot, read the capture dumps, compute G-9w
per captured frame, write it into `report.json`'s `checks.torso_clip`, recompute each frame's `pass`, then write the
sheet (the HTML border shows the verdict). `pose_sheet.gd`: the slab torso test in `_checks` removed with `CLIP_MM`,
`POLAR_BINS`, `LIMB_R`, `MIX_W` (the filters live in Python now); `SLAB_M` stays for W-4 until P3. The PNG's red
border is drawn for the checks still decided in GDScript (G-13, W-4); the HTML card is the authority for G-9w.

**Implementation.**

- [ ] `gate_g9w`: per full frame, trunk patch faces from the labels, limb queries from the filters, `contact.inside`;
      FAIL beyond `self.limb_in_trunk` / `self.hand_in_trunk`; worst depth per limb group per frame.
- [ ] In outfit mode, the trunk patch is the union of every part's trunk faces and a limb is queried only on its
      outermost covering layer (F-S32p-6's rule, kept).
- [ ] The ground probe also records `G-9w` for each body's walk and jog frames in the block's `gates`.

**Validation.**

- `--plant-arm-into-torso` (exists): every frame FAILs, depth ≥ 150 mm (the slab test read 210 mm).
- The protagonist (`meshy_d.glb`, `--grip`): every frame PASSes, as with the slab test (PIPELINE.md §15, validated
  2026-10-15).
- The CharMorph town body (`vitruvian.glb`, layered): recorded; the slab test misread it, so a change is expected and
  is reported with its reason.
- Synthetic: a hand query 8 mm inside an open trunk shell → FAIL 8 mm.

**Acceptance.** The plant fails, the protagonist passes, and the slab code is gone from `pose_sheet.gd`.

**Commit boundary.** G-9w and the retirement of the slab test.

### Commit 4 — generated variants

**Goal.** Prove the gates on bodies no one tuned them on.

**Scope.** `pose_sheet.gd`: `--plant-sole-offset <m>` and `--plant-stance-lift <m>`.

- `--plant-sole-offset <m>` makes the body read as sunk by `m`. Lowering the body itself would not work, because
  `GroundContact` lifts it back every frame. So the plant raises the **floor reference** the gate reads instead:
  `floor_y += m` in every dumped frame.
- `--plant-stance-lift <m>` adds a `SkeletonModifier3D` after `GroundContact` that raises the left `Foot` bone's
  pose by `m` in the frames the gait calls stance (read from the clip's foot speed).

`mwcast/ground.py` passes `--plant` through and never writes a planted run to a manifest.

**Validation (the acceptance of the PR).**

- Every body found by `ground.bodies()` — both shipped casts (café town and lakeside), and the reference body —
  passes G-4w and G-9w.
- The same run with `--plant-sole-offset 0.006` fails G-4w for every body (sink ≈ 6 mm); with `0.004`, passes.
- `--plant-stance-lift 0.012` fails G-4w on the left foot only, in planted frames only, for every body.

**Commit boundary.** Plants and the evidence run.

### Commit 5 — records and specification

- [ ] `cast.py ground --write` over every body (with `with-slot.sh`); the blocks committed.
- [ ] PIPELINE.md §15 (checks list: G-9w replaces G-9 simplified) and §15.1 (G-4w, the dump format, the planted rule,
      the tolerance keys).
- [ ] This ledger: evidence, run times, per-body table.

## 5. Acceptance criteria (PR level)

1. G-4w reports per foot, per clip, per frame, with the tolerance key named, for every enumerated body.
2. No bound literal remains in `pose_sheet.gd`, `ground.py` or `check_cast.py` for G-4w or G-9w (I-S36-2).
3. The plants of commit 4 fail for every body and pass below the bound; the planted-arm probe fails G-9w.
4. Both casts pass, or each failure is a recorded finding handed to the NPC lane with frame and depth.
5. CC-9 in CI fails a block measured under different bounds.
6. `fast` is green on the final head.

## 6. Risks

- **The runtime correction hides an asset defect.** `GroundContact` lifts the hips every frame, so G-4w measures the
  corrected body. That is right for "what the player sees", but it can mask a bad asset. The block also records
  `GroundContact.last_lift` per frame (max and mean); P4 decides a bound for it. Reported, not judged, here.
- **Dump size and time.** About 45 MB and one extra second per body per run; measured and recorded.
- **Planted test at a slow walk.** At low `v` the 25 % share is a small absolute speed; noise could flip frames. The
  clips run at their measured speeds (≥ 1 m/s), so this is not expected; if seen, record it.

## 7. Questions

Q-S36-3 (planted float bound), Q-S36-6 (extend S26-c-ground in place), Q-S36-7 (retire the slab test) in the step.

## 8. Ledger

| Commit | Implementation | Validation | Review | Evidence |
| --- | --- | --- | --- | --- |
| 1 | [ ] | [ ] | [ ] | — |
| 2 | [ ] | [ ] | [ ] | — |
| 3 | [ ] | [ ] | [ ] | — |
| 4 | [ ] | [ ] | [ ] | — |
| 5 | [ ] | N/A (records) | [ ] | — |

## 9. Execution contract (filled at freeze)

```text
PROJECT / PR:          S36-P2 — G-4w and G-9w across the gait
PRIMARY DESIGN DOC:    .structured-coding/plans/mvp1/pr-s36-p2-gait-contact.md
RELATED / BINDING:     step-36-interpenetration.md; pr-s36-p1-contact-module.md; PIPELINE.md §15; S26-c-ground
IMPLEMENTATION BASE:   origin/main at freeze, with S36-P1 and S26-c-ground merged
APPROVED SCOPE:        §3
FROZEN INVARIANTS:     I-S36-1 … I-S36-5 (step §5)
SEQUENCE:              commits 1-5
VALIDATION BUDGET:     Godot runs only through cast-slots/with-slot.sh; one full-cast run per commit at most; no money
LIVE DOCUMENTATION:    this file, §8
```
