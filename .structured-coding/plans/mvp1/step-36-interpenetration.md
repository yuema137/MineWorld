# Step 36 — No interpenetration: ground contact and clipping checks for people, garments and items

**Role:** step document (step planning phase). The step number **S36** is proposed; the primary session assigns it
in the overall plan. The document states the ruling, what the repository already checks, the shared mechanism, the
tolerances, the five PRs with their acceptance, risks and questions.

**Lifecycle:** `DESIGN FROZEN — 2026-10-19` (operator approved in the primary session: freeze with Q-S36-1 … Q-S36-9 answered as recommended). It authorizes no implementation. Each PR is
detailed to the commit in its own PR design and frozen in turn (`CLAUDE.md` §3.1). The first two are drafted beside
this document: [`pr-s36-p1-contact-module.md`](pr-s36-p1-contact-module.md) and
[`pr-s36-p2-gait-contact.md`](pr-s36-p2-gait-contact.md). P3–P5 are drafted (2026-10-19, DRAFT, awaiting freeze):
[`pr-s36-p3-garment-pushout.md`](pr-s36-p3-garment-pushout.md),
[`pr-s36-p4-foot-grounding.md`](pr-s36-p4-foot-grounding.md) and
[`pr-s36-p5-scene-layout.md`](pr-s36-p5-scene-layout.md); each lists the edits it proposes to §4 of this document.

**Design session:** worktree `/Users/yuema137/mineworld-worktrees/design-step36-clip`, branch
`docs/step36-interpenetration`, audit base `origin/main @ b843d546` (2026-10-19).

**Records:** `DEP-142` (libigl), `DEP-143` (trimesh + python-fcl), `DEP-144` (Godot built-in IK), `DEP-145`
(rejections). All four are drafts under `docs/decisions/` until this step is frozen.

---

## 0. Binding inputs

### 0.1 The ruling (operator, 2026-10-19)

> 以后，所有人物 物品等都需要检查地面和相互之间的穿模

In English: from now on, every person and every item is checked for ground contact and for interpenetration with
each other. The operator also approved the plan in `mineworld-demos/research/clipping-tools.md` (2026-10-19).

The lane-lead charter states it as a gate rule: every character, garment and item gate checks ground contact and
mutual interpenetration, at rest and across animation cycles, and fails beyond tolerance; every placed world checks
item–item and person–item overlap; existing open-source tools are adopted rather than written.

### 0.2 Standing rules that bind this step

- **No world-specific hard-coding** (operator, 2026-10-18). A check reads its inputs from data: the bodies come from
  the cast index and the manifests, the props from the model library, the places from `worlds/*`. A fix that only
  makes the café town pass is rejected. Every PR's acceptance names a second world or a generated variant.
- **Reuse before reinvention** (`CLAUDE.md` §4 rule 16, `docs/REUSE_POLICY.md`). Adopting and rejecting both need a
  record: `DEP-142` … `DEP-145`.
- **The client never decides where a person is** (`ENGINEERING_RULES.md`, `DEP-20`). Whether a person walks through
  a chair is a server question (Rapier against `Solid`s, `DEP-13`). The client only draws feet on the floor. So the
  runtime work (P4) is presentation only.
- **All platforms** (macOS, Linux, Windows). Every adopted wheel must exist on all three; this rules out Coal and
  CGAL (`DEP-145`).
- **Speed first** (charter, 2026-10-18). Each PR merges when implemented and `fast` is green; the full suites run at
  the milestone.

## 1. The requirement, as checkable outcomes

| Id | Outcome | Checked by |
| --- | --- | --- |
| O-1 | No body's lowest skinned point goes below the floor beyond tolerance, at rest, standing, and in every sampled frame of every locomotion cycle | G-4 (rest), G-4w (P2) |
| O-2 | A foot that the gait calls planted does not float above the floor beyond tolerance | G-4w (P2) |
| O-3 | No limb goes into the trunk beyond tolerance, at rest and in every sampled frame | C-1 (P1, rest), G-9w (P2, frames) |
| O-4 | No body surface pokes through the garment that covers it, and no inner garment pokes through an outer one, at rest and in every sampled frame | C-2 (P1, rest), W-4w (P3, frames) |
| O-5 | Every floor-standing item rests on the floor; no two parts of one item interpenetrate beyond tolerance | C-3, C-4 (P1) |
| O-6 | In the running client, feet meet the floor they stand on, also on uneven ground | P4 runtime check |
| O-7 | In every shipped place: every spawn and doorway point clears every `Solid` by `PERSON_RADIUS + GAP`; no two `Solid`s overlap; every drawn furniture mesh lies inside its `Solid` and rests on the floor | P5 (Rust test + trimesh check) |

## 2. Audit (`origin/main @ b843d546`, 2026-10-19)

### 2.1 What exists

| Concern | Today | Where |
| --- | --- | --- |
| Soles at rest | G-4(a): the lowest glTF `y` within `SOLE_TOL_M` (5 mm) of `SOLE_Z_M` (−0.0177 m). Rest pose only | `tools/character_pipeline/mwcast/gates.py` `gate_g4`; `mwcast/standard.py` |
| Torso clip | G-9 (simplified): 2 cm slabs, a 48-direction outline of trunk vertices, limb vertices more than 5 mm inside fail. Hand-written geometry | `tools/character_pipeline/godot/pose_sheet.gd` `_checks`, `CLIP_MM` |
| Garment poke-through | W-4: a vertex outside its covering garment's shell by more than 2 mm, by slab outlines | `pose_sheet.gd` `_poke_through`, `POKE_MM` |
| Seat contact | G-13: ±20 mm | `pose_sheet.gd` `_checks` |
| Skinned frames | `_skinned(h)` computes Σ w · pose · bind on the CPU, with poses read in `skeleton_updated` so modifier results count | `pose_sheet.gd` |
| Walk and jog sampling | the treadmill loop samples every frame of one `Walk` and one `Jog_Fwd` cycle at a fixed 30 fps (for stride only) | `pose_sheet.gd` `_run` |
| Binary vertex dump | `MWGT`: magic, count, float32 xyz | `godot/wardrobe/fit_check.gd` `_write_positions` |
| Python dependencies | the pipeline is stdlib only; CI's `python` layer runs `tools/character_pipeline/tests` from the root uv environment, which has no numpy | `scripts/ci_layer.py` (`PIPELINE`), `pyproject.toml`, `uv.lock` |
| Server solids | `Solid { min, max: Corner, height }`, integer mm, no id; `PlaceShape::checked` does not test solid–solid overlap | `systems/bodies/src/component.rs` |
| Load-time fit | people keep `PERSON_RADIUS` (250 mm) from solids (no `GAP`); doorways keep `R + GAP` (260 mm); objects are checked against solids | `systems/bodies/src/genesis.rs` `fits`, `doorways_fit`, `placed` |
| Seats and anchors | the server has none. Seat heights are client constants (`town.gd` `CAFE_SEAT_Y`) | `clients/3d-spike/scripts/town.gd` |
| Prop ↔ Solid link | none. Props are placed in GDScript (`cafe_interior.gd`); `put_solid` adds a box at 0.92× the mesh bounds; the YAML solids are a hand transcription | `slice/dressing.gd` `put_solid` |
| Client IK | none; the only `SkeletonModifier3D` is `Posture` | `clients/3d-spike/scripts/posture.gd` |

### 2.2 Work in flight that this step must build on: S26-c-ground

The NPC lane is fixing feet below the ground as a follow-up to PR #306, on the local branch `mvp0/pr-s26c-ground`
(commits `7f4eda66`, `321789d3`, `d7c08f3f`, 2026-10-19; not on `origin` at audit time). Read, not touched. It adds:

1. **A ground probe.** `pose_sheet.gd --ground-only --bodies <json>` builds every body the client draws, headless,
   and measures the lowest skinned point of the feet and shins (`_ground_set`, `_ground_low`, `_ground`) at rest,
   over the `Stand` loop and in every frame of one `Walk` and one `Jog_Fwd` cycle. Bounds: sink 5 mm, float 20 mm,
   as GDScript constants.
2. **A recorded block.** `mwcast/ground.py` and `cast.py ground --write` record a `ground` block in each body's
   `character.yaml`. `scripts/check_cast.py` CC-9 recomputes the verdict from the recorded numbers in CI. All 22
   bodies pass (worst sink −3.1 mm).
3. **Three runtime fixes.** The `Stand` clip's hips key is normalised by `motion_scale`; each scene is raised by its
   own rest sole; a `GroundContact` modifier (`clients/3d-spike/scripts/ground_contact.gd`) moves the `Hips` so the
   lowest support point stays on the floor.

So the lane has already built O-1 for its cast, with hand-written geometry and bounds in code. This step does not
duplicate it. P2 generalises the probe (per-foot, planted-foot float, data tolerances, arm–torso clearance) and
keeps CC-9's recorded block. P4 builds the foot IK on top of `GroundContact`.

### 2.3 Findings

- **F-1.** Every check today is a few hand-written geometric tests (slabs, outlines, a min-y). They misread
  layered garments (F-S32p-6), and a fused mesh needs special filters.
- **F-2.** Nothing compares a drawn furniture mesh with the `Solid` the server resolves against. A table drawn 30 cm
  wider than its `Solid` is exactly "people walk through furniture".
- **F-3.** Bounds live in code, in three places (GDScript constants, `standard.py`, `check_cast.py`). A tolerance
  change touches several files and can drift.
- **F-4.** No test checks that two `Solid`s do not overlap, and genesis checks people against solids without `GAP`.

## 3. The mechanism

### 3.1 One shape for every check

Every check has three parts, and only the middle one is new:

```text
pose source (exists)            contact probe (new, P1)             gate (exists or new)
Godot skinning, per frame  ──▶  mwcast/contact.py               ──▶  G-4w, G-9w, W-4w, C-1…C-4
or a GLB at rest                 winding number, distance,             verdict + margins, per region
                                 rigid overlap
```

1. **The pose comes from Godot.** None of the mesh libraries plays an animation. So Godot skins each sampled frame
   (as `_skinned` already does) and dumps the vertex positions. Python never re-implements skinning. The skinning
   that is checked is therefore the skinning the client draws.
2. **`mwcast/contact.py` is the only module that imports `igl`, `trimesh` or `fcl`.** It takes numpy arrays and
   returns typed results (vertex, depth in mm). This is the same isolation rule `DEP-13` applies to Rapier.
3. **A gate reads the results and the tolerances, and decides.** It prints every margin, as G-6 does.

### 3.2 Why a winding number

A Meshy body or a garment shell is often open (holes at the neck, the cuffs) and may self-intersect. "Is this point
inside?" by ray parity fails on such meshes: a ray through the cuff hole crosses the surface once, and an inside
point reads as outside. The generalized winding number (Jacobson et al., SIGGRAPH 2013) sums the solid angle the
surface subtends at the point. It is about 1 inside, about 0 outside, and degrades smoothly at holes. A threshold of
0.5 gives a robust answer.

Example: a hand pushed 8 mm into the side of a hoodie. The hoodie's trunk patch is open at the neck, the waist and
the armholes. Ray parity from the hand might exit through the armhole and say "outside". The winding number of the
patch at the hand is about 0.9, so the hand is inside. Its distance to the patch, 8 mm, is the depth.

### 3.3 Tolerances are data

One file holds every bound: `tools/character_pipeline/contact_tolerances.json` (P1). Each entry names a region, a
bound in millimetres, its rationale and the status of its source. The gates, the Godot probe's report and
`check_cast.py` all read this file. A presentation pack may supply its own file that **tightens** a bound, never
loosens it (Q-S36-4). The file's SHA-256 is recorded with every measurement, so a recorded block is re-judged
against the bounds it was measured with.

| Region key | Bound | Rationale | Source status |
| --- | --- | --- | --- |
| `floor.sink.foot` | 5 mm | The outsole and heel cushioning of an ordinary shoe compress by a few millimetres under body weight, and the eye accepts that much. It is G-4's existing bound, kept. | inherited (G-4, step-26); a footwear compression reference is read in P1 and the entry marked `cited` or `inconclusive` |
| `floor.sink.shin` | 5 mm | A trouser hem may brush the floor; same bound as the sole | inherited (S26-c-ground) |
| `floor.float.planted` | 10 mm | A planted foot must read as planted. In normal walking the swinging foot clears the floor by only about 13 mm at its lowest (minimum toe clearance, Winter 1992). A planted foot floating more than that looks like a swinging one, so the bound sits below it. | cited: D. A. Winter, "Foot trajectory in human gait", *Physical Therapy* 72(1), 1992 (P2 confirms the figure) |
| `floor.float.walk_frame` | 20 mm | In a walk one foot is always on the floor. The lowest point of each frame may float at most this far | inherited (S26-c-ground, CC-9) |
| `self.limb_in_trunk` | 5 mm | An arm resting against the flank compresses skin and fat by a few millimetres; deeper is a visible clip. G-9's bound, kept | inherited (G-9, step-26 §6.1) |
| `self.hand_in_trunk` | 5 mm | Same reason, for the hand on the hip or the belly | inherited (G-9) |
| `garment.poke.body` | 2 mm | About the thickness of the cloth itself: woven shirting is under 0.5 mm, denim under 1 mm, knitwear 1–3 mm (thickness measured per ASTM D1777) | inherited (W-4, step-32 §7.2); P1 records the figures read |
| `garment.poke.layer` | 2 mm | An inner layer poking through an outer one; the same cloth-thickness reason | new, same source as above |
| `item.rest.floor` | 3 mm | A floor-standing prop that floats or sinks by more shows a seam or loses its contact shadow at a 2 m viewing distance | new; plausibility, marked `inconclusive` until a viewing-distance test in P1 |
| `item.overlap.parts` | 2 mm | Rigid parts do not interpenetrate in reality; 2 mm covers tessellation error | new |
| `scene.mesh_over_solid` | 40 mm | A drawn mesh may overhang its `Solid` by at most this much. The server keeps a person's centre 250 mm (`PERSON_RADIUS`) from a solid; half a man's shoulder breadth is about 200 mm (biacromial breadth ≈ 400 mm, ANSUR II); 250 − 200 − 10 (`GAP`) ≈ 40 mm is the overhang a shoulder never meets | cited: ANSUR II (2012) biacromial breadth; P5 confirms the percentile used |

Each entry's `status` is one of `cited`, `inherited` or `inconclusive`, following PIPELINE.md §6's treatment of
Winter's figures. An `inconclusive` entry is still enforced; it records that its source was not read yet.

## 4. The PRs

| PR | Title | Layer | Size | Depends on |
| --- | --- | --- | --- | --- |
| **P1** | `mwcast/contact.py`: libigl + trimesh/python-fcl behind one module; tolerance data; static body, garment and item checks | asset tooling (Python, CI) | M | — |
| **P2** | G-4w: per-frame sole vs floor in walk and jog from Godot skinned-frame dumps; G-9w arm–torso clearance | asset tooling (Godot probe + Python, CI on recorded blocks) | M | P1; S26-c-ground merged |
| **P3** | Garment–body winding-number push-out in layer order; W-4w per frame | wardrobe tooling (Blender stage + Python) | M | P1, P2 (dump format) |
| **P4** | Runtime foot grounding: `TwoBoneIK3D` per leg on a floor ray, per-body hips offset; presentation only | 3D client | M | S26-c-ground merged; P2 (its gate keeps IK honest) |
| **P5** | Scene layout: Rust test (spawns, doorways, objects clear solids; solids do not overlap) and trimesh check (meshes inside their `Solid`, resting on the floor) | Rust test + Python + client dump | M | P1 |

P1 and P5's Rust half can run in parallel. P2 and P5's Python half follow P1. P3 and P4 follow P2.

### 4.1 P1 — the contact module and static checks

Detailed in [`pr-s36-p1-contact-module.md`](pr-s36-p1-contact-module.md). In short: add `numpy`, `libigl`, `trimesh`
and `python-fcl` to the root dev dependency group (CI installs them; no runtime does); write `mwcast/contact.py` and
the tolerance file; add four static gates — C-1 limb in trunk at rest, C-2 garment layer order and body poke at
rest, C-3 item rests on the floor, C-4 parts of one item do not overlap; and a test that fails if any file imports
`igl.copyleft` (GPL).

**Acceptance (step level).** The gates run over every body, outfit and prop they find by enumeration, never over a
list. A generated variant (synthetic meshes with known planted depths: a box raised 6 mm, a hand sunk 8 mm into a
trunk shell, an inner layer pushed 3 mm through an outer one) fails exactly where planted and passes at the
tolerance minus 1 mm.

### 4.2 P2 — G-4w and G-9w across the gait

Detailed in [`pr-s36-p2-gait-contact.md`](pr-s36-p2-gait-contact.md). In short: the Godot probe writes each sampled
frame's skinned vertices with per-vertex labels to one binary dump; Python decides per foot and per frame (sink, and
float while planted, with "planted" defined by G-8's drift rule) and per limb (winding number against the trunk
patch). It subsumes S26-c-ground's probe: the `ground` block in `character.yaml` keeps its name and gains per-foot
fields; CC-9 reads its bounds from the tolerance file. G-9's slab outline is retired.

**Acceptance (step level).** Every body in both shipped casts (the café town's and the lakeside's, read from the
manifests) passes, and two generated variants fail: a body lowered 6 mm, and a body with a planted foot raised
12 mm in the stance phase.

### 4.3 P3 — garment push-out in layer order

Replace the ray push-out in `blender/mpfb/assemble.py` (L-b; first added in `garment_clean.py` by `a2967fb3`) with a
winding-number push-out:
garment vertices with winding number > 0.5 against the layer under them move out along the distance gradient to
`+gap`. Layers are processed inner to outer (body → base layer → mid → outer), so each layer is pushed out of the
already-fixed one under it. libigl runs in the tooling venv on exported arrays, not inside Blender's bundled Python
(nothing is installed into Blender). W-4w replaces W-4's slab outlines in the pose sheet's outfit mode with the
P2 dump and C-2's pairwise test, per frame. (No vertex-group exclusion of cords: the push-out tests a garment only
against the surfaces under it, so a cord outside the tee does not move; F-P3-2.)

**Acceptance.** Vera's, Ivan's and Hana's outfits (route B, S32-p) pass W-4w in every sampled frame, and a
generated outfit from `outfits.sweep` with a different seed passes too. A planted outfit (the hoodie shrunk 3 %)
fails before push-out and passes after.

*Amended by the frozen PR design (Q-S36-P3-1) and the protagonist-only wardrobe ruling:* every Vera outfit and
the sweep pass C-2 at rest except vertices reported unreached; Vera A, Vera B and one sweep outfit pass W-4w in
every frame; Ivan and Hana are not run. The execution record (`pr-s36-p3-garment-pushout.md` §9) shows this is not
met by the current assets (open shells, holes): recorded findings, Q-S36-P3-3.

### 4.4 P4 — runtime foot grounding (presentation only)

Built on S26-c-ground's `GroundContact`, which already gives each body its per-body hips offset (the scene raised by
its own rest sole, the hips lifted out of the floor). P4 adds what a hips offset alone cannot do: put each foot on
uneven ground.

1. Per leg one `TwoBoneIK3D` (`LeftUpperLeg` → `LeftLowerLeg` → `LeftFoot`, the retargeted humanoid names), its target
   the floor hit under the ankle plus the body's sole offset, the pole the knee's forward direction.
2. The floor height under each foot comes from a ray against the client's disclosed colliders
   (`slice/client_bodies.gd` builds them) and the terrain.
3. Each foot's IK influence follows the gait's contact curve (the planted share G-8 measures); `GroundContact`
   lowers the pelvis to the lower foot target; `LimitAngularVelocityModifier3D` after them smooths pops.
4. IK runs only within a distance of the camera (data, not a literal), because it costs CPU per person.
5. Nothing it computes is sent to the server.

Wrapped in one script, so a fallback (the Godot 4.6/4.7 IK nodes are new; `DEP-144`) is one file.

**Acceptance.** On generated courses (seeds 1–3: kerbs, ramps, stairs, uneven patches) and the slice street's kerb
now, and on the lakeside's uneven terrain (S33-d) at the step's milestone run once S33-d merges (Q-S36-P4-1), the
client's own frame probe shows every planted foot of
every visible person within `floor.sink.foot` and `floor.float.planted` of the ground under it; on the café floor
nothing changes measurably (the flat case stays at P2's numbers).

### 4.5 P5 — scene layout checks

1. **Authoritative (Rust, `cargo test`).** For every world under `worlds/` (found by listing the directory: today
   `market-town`, `social-cafe`, `bodies-yard`, `lakeside`), every person's spawn, every doorway point and every
   placed object clears every `Solid` by `PERSON_RADIUS + GAP`, and no two `Solid`s of one place overlap. The test
   uses the `bodies` pack's own geometry (`Room::admits`, `Room::clear_of_solids`). The generated variant is
   `systems/bodies/tests/route.rs`'s seeded random scenes (`Scene::generate`). Whether genesis itself should refuse
   overlapping solids (an error code beside `bodies-in-solid`) is decided in P5's design.
2. **Presentation vs truth (Python, trimesh + python-fcl).** The client dumps each placed prop's mesh, transform and
   the `Solid` it blocks (headless slice run). Then: (a) every furniture mesh's floor footprint lies inside its
   `Solid` within `scene.mesh_over_solid`; (b) no two props interpenetrate beyond `item.overlap.parts`, except pairs
   declared resting (a cup on a table); (c) every floor-standing prop's lowest point is within `item.rest.floor` of
   the floor.

Today no id links a drawn prop to its `Solid` (§2.1). P5 needs one. PR vf-furniture-solids left the matching
question open (Q-VF-F1, recommended option (b): an optional `kind` / `yaw` on solids). P5's design resolves the
link together with Q-VF-F1.

**Acceptance.** Both towns and the bodies yard pass; the lakeside (no solids yet) passes vacuously and is listed as
such; a generated variant (a copy of a place with one solid moved to overlap another, and a prop drawn 60 mm wider)
fails by name.

### 4.6 Integration checkpoint of the step

At the milestone: `cast.py ground` over every body, `cast.py contact-items` over every prop, the Rust layout test
over every world, and the 3D slice on the lakeside with the foot probe. All pass, and each planted variant fails
with its name and depth printed.

## 5. Invariants (proposed)

- **I-S36-1.** Only `mwcast/contact.py` imports `igl`, `trimesh` or `fcl`. Nothing imports `igl.copyleft`. A test
  enforces both.
- **I-S36-2.** Every bound comes from the tolerance file; no gate, probe or check script holds a bound literal.
- **I-S36-3.** The checked skinning is Godot's. Python never re-skins a body.
- **I-S36-4.** A gate that cannot look (a library missing, a dump missing) reports FAIL with the reason, never PASS.
- **I-S36-5.** No check reads a world's, a body's or a prop's name to decide anything. Inputs are enumerated.
- **I-S36-6.** Runtime foot grounding is presentation only: it moves no authoritative state and sends nothing.

## 6. Risks

| Risk | Effect | Mitigation |
| --- | --- | --- |
| Sampling misses tunnelling between frames in a fast jog | a clip passes that clips between samples | sample every frame at the clip's fixed 30 fps; IPC Toolkit's CCD is recorded as a later option (`DEP-145`) |
| Accidental GPL import (`igl.copyleft`) | licence contamination | I-S36-1's test |
| Fused Meshy meshes blend limb and trunk at the seam | false clips at the shoulder | keep G-9's share and segment filters as *which vertices to ask about*; only the inside test changes |
| Soft-sole or cloth contact within 1–3 mm | false failures | per-region bounds with rationale; margins printed |
| libigl or python-fcl wheels lag a new Python | CI breaks on upgrade | versions pinned in `uv.lock`; Python stays at 3.12 (`requires-python >=3.12`) |
| Godot 4.6/4.7 IK nodes are new (forum reports of errors in `CCDIK3D`, `FABRIK3D`) | runtime bugs | use `TwoBoneIK3D` only; one wrapper script; P2's gate stays in CI, so IK corrects millimetres, not centimetres |
| CI time and download size grow | slower `python` layer | the wheels are binary; numpy is shared; measured in P1 |

## 7. Questions (Q-S36-n), each with a recommendation

**Ruled 2026-10-19 (operator, primary session):** every question below is answered as recommended.

| Id | Question | Recommendation |
| --- | --- | --- |
| Q-S36-1 | Is this step number 36, under `mvp1`? | Yes |
| Q-S36-2 | Use `DEP-142` … `DEP-145` (141 is taken by S26-c1's Git LFS record)? | Yes |
| Q-S36-3 | A planted foot may float at most 10 mm (below the ~13 mm toe clearance of real walking), rather than today's 20 mm? | Yes, 10 mm for planted feet; keep 20 mm for "some foot is down" in a walk frame |
| Q-S36-4 | Tolerances live in one pipeline file; a presentation pack may only tighten them? | Yes |
| Q-S36-5 | Add `numpy`, `libigl`, `trimesh`, `python-fcl` to the root **dev** dependency group, so CI installs them (tooling only, never a runtime dependency)? | Yes |
| Q-S36-6 | P2 waits for S26-c-ground to merge, then extends it in place (`cast.py ground` and the `ground` block keep their names)? | Yes |
| Q-S36-7 | Retire G-9's slab outline in P2 once the winding-number test agrees on the protagonist and the planted arm, with no fallback kept? | Yes |
| Q-S36-8 | Leave IPC Toolkit's between-frame check out for now? | Yes; revisit if a tunnelling clip is ever seen |
| Q-S36-9 | P5 needs a link from a drawn prop to its `Solid`; decide it in P5's design together with Q-VF-F1? | Yes |

## 8. Proposed edits to shared documents (for the primary session)

- The overall plan: add S36 with this document's PR table.
- `tools/character_pipeline/PIPELINE.md`: a new §18 "Contact checks" (written by P1), §15's checks list updated by
  P2, §17's W-4 row updated by P3.
- `/Users/yuema137/mineworld-demos/progress/`: the coordinator's page.

## 9. Sources

- Research note: `/Users/yuema137/mineworld-demos/research/clipping-tools.md` (2026-10-19), with its package-index
  facts and URLs.
- A. Jacobson, L. Kavan, O. Sorkine-Hornung, "Robust Inside-Outside Segmentation using Generalized Winding Numbers",
  *ACM Transactions on Graphics* 32(4), SIGGRAPH 2013.
- G. Barill, N. Dickson, R. Schmidt, D. Levin, A. Jacobson, "Fast Winding Numbers for Soups and Clouds",
  *ACM Transactions on Graphics* 37(4), SIGGRAPH 2018 (`igl.fast_winding_number`).
- D. A. Winter, "Foot trajectory in human gait: a precise and multifactorial motor control task", *Physical Therapy*
  72(1):45–53, 1992.
- ANSUR II anthropometric survey (US Army, 2012), biacromial breadth.
- ASTM D1777, standard test method for thickness of textile materials.
- Godot: <https://godotengine.org/article/inverse-kinematics-returns-to-godot-4-6/>.
- Repository: `DEP-13`, `DEP-20`, `DEP-34`; `tools/character_pipeline/PIPELINE.md` §§6, 15, 17;
  `.structured-coding/plans/mvp0/pr-s32p-wardrobe-pilot.md` §4.8; `.structured-coding/plans/mvp1/pr-vf-furniture-solids.md`.
