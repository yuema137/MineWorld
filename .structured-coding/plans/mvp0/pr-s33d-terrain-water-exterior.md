# PR S33-d — Lakeside's land, water and village exterior in 3D: terrain, the near lake, the shore, the houses, the woods

## PR DESIGN (DRAFT) — not frozen

```text
Design revision:        revision 1 (2026-10-18), DRAFT
Author:                 S33 lane lead (Opus), worktree /Users/yuema137/mineworld-worktrees/design-s33de,
                        branch docs/pr-designs-s33d-j, planning base origin/main @ a1e05874. Read-only
                        references: 16c origin/mvp0/pr-16c @ 36325625, 12e origin/mvp0/pr-12e @ e5000ce7,
                        12d origin/mvp0/pr-12d-towns @ 841805ee
Approved by / evidence: none yet. The step (APPROVED 2026-10-15) binds; Q-S33-1…11 are not re-asked;
                        §14's questions are open
Implementation base:    origin/main at C0; C0 records it and the merge state of S33-a, S33-b, S33-c, 12e
Start condition:        (1) S33-a merged (Lakeside's far fields); (2) S33-b merged (Lakeside's floors and
                        solids); (3) S33-c merged (the depiction seam and the Lakeside stub); (4) 12e merged
                        (disclosed walls, X-8); (5) §14 ruled; (6) this design frozen; (7) the lane lead's
                        dispatch. NONE holds at 2026-10-18
Execution contract:     §12
Implementer:            Opus-class for the art commits (C5, C6, C7) and the frame reviews; Haiku-class for the
                        mechanical commits (C2, C3, C4, C8, C9) from this commit plan (step-33 §12.1:
                        "Opus (art judgement); mechanical parts to Haiku")
Lifecycle:              PR DESIGN (DRAFT)
```

**Effort:** `mvp0`, release-blocking · **Step:** [`step-33-lakeside-3d.md`](step-33-lakeside-3d.md) (APPROVED
2026-10-15) §§1 (outcomes 1, 2, 7), 3.4, 4.0–4.2, 4.6, 4.7, 4.9, 8.1, 9.1, 9.2, 9.4 (X-1, X-8), 10, 11 (AC-S33-1,
AC-S33-5 in part), 12.1 (the S33-d row), 15 (`DEP-77`, `DEP-78`, `DEP-79`) · **Parent:** [`overall.md`](overall.md)
· **Consumes:** [`pr-s33a-setting-far-field.md`](pr-s33a-setting-far-field.md) (the far fields, `datum_m`,
`level_m`, `reach_m`), [`pr-s33b-lakeside-bodies.md`](pr-s33b-lakeside-bodies.md) (§5.3's floors and solids),
[`pr-s33c-world-seam.md`](pr-s33c-world-seam.md) (`SliceDepiction`, the Lakeside stub file this PR replaces),
12e (`SliceClientBodies`, `DisclosedGeometry`) · **Followed by:** S33-e (pier, halt), S33-f (interiors, sit
points), S33-g (people), S33-j (export) · **Binding parents:** `CLAUDE.md` §§1, 2, 4 (items 3, 9, 10, 14, 15,
16); `docs/ENGINEERING_RULES.md` §§2–6, 10–12; `docs/VISUAL_SLICE.md`; `docs/ART_DIRECTION.md` §4;
`docs/REUSE_POLICY.md`.

**Decision ids.** `DEP-77` (terrain), `DEP-78` (water), `DEP-79` (vegetation and at most four CC0 sets), all
step-assigned, written by C1. `ARC-170` is reserved to this PR for the grading decision of §5.2 (the lane's
height above the lake, Q-S33d-1), written by C1 only if Q-S33d-1 is ruled as recommended or with option B. No
other id.

No model is called by this design. No Meshy credit, no OpenAI call, nothing bought (Q-S33-7).

---

## 0. Binding inputs, exactly restated

- **Scope (step-33 §12.1, S33-d row).** "the height grid and terrain mesh; the near lake and shader extension;
  shore, lane, trail, boulders; cottages, bakery and boathouse exteriors; the woods (`MultiMesh` cards); GI volume
  and probes; far-field views; `farfield` shots; perf views". Depends on S33-a, S33-b, S33-c, **12e merged**.
- **Integration checkpoint.** "AC-S33-1 on three OSes; counts within §8; first `VIS-3D-LAKESIDE` preview to the
  operator (ARC-24)."
- **Adversarial.** "X-1 holds; X-8." X-1: "delete the lake's water sector from the 3D far field → `FAR_FIELD_GAP`
  at validate, or `FF-BELOW` in the south sectors at VP-L-3". X-8: "add a drawn wall in the bakery with no
  disclosed solid → the disclosed-vs-drawn wall check".
- **Terrain (step §4.1, `DEP-77`).** "One heightfield over the near field (≈ 200 × 200 m, 1 m cells), built by our
  builder from a small authored height grid … shaded with the slice's `leafy_grass`/`forest ground`/
  `cobblestone_floor_08` by a splat of at most four layers"; Godot `ArrayMesh` + `HeightMapShape3D`.
- **Water (step §4.1, `DEP-78`).** "A world-anchored surface from the shore line to `reach_m`, continuing into
  RL-e0's far-field water sector at the same level and colours. Shader: `water.gdshader` … extended with a depth
  fade at the shore and round the pier posts … and lily pads as a decal layer near the shore. No flow, no foam
  simulation." Reflections: "a reflection probe … SSR as a measured option … Planar reflection … is rejected".
- **Vegetation (step §4.2, `DEP-79`).** "Species follow the setting (L-1): Jeffrey pine and Coulter pine, incense
  cedar, black oak … The square's lime tree stays … the spike's procedural tree builder … One `MultiMesh` per
  species and LOD band … Dense planting from about 60 m to `reach_m` closes the land sectors … with air walls behind
  the first rank."
- **Heights (step §4.6, F-S33-4).** "Height is presentation … People stand on the terrain the client draws, found
  by a ray to `LAYER_WORLD`; the server never sees a height."
- **Perf (step §8.1).** Absolute gates p95 ≤ 16.7 ms, video memory ≤ 2 048 MB on the operator's M5, three runs;
  draw calls and primitives advisory. "A p95 above 16.7 ms in a new view is a material stop with the levers
  measured."
- **Size (step §8.2).** Lakeside's increment target ≤ 12 MB, gate ≤ 16 MB.
- **Operator rules (2026-10-18).** Every world has a far field (air walls fine, an empty horizon never); physical
  defaults follow real-world values with a cited source; everything works on macOS, Linux and Windows.
- **R-LOAD-1.** Locally: Godot runs one at a time through `cast-slots/with-slot.sh`; never
  `cargo test --workspace`; local timings INCONCLUSIVE.

---

## 1. Goal

A player who starts Lakeside in 3D stands in a real place: a cobbled lane a metre above a still mountain lake,
cottages, a bakery and a boatbuilder's shed along its north side, a shore of granite boulders and reeds falling to
the water, and woods of pine, cedar and oak closing the land on every side. Past the woods and across the water,
S33-a's far field (Cuyamaca Peak to the south-west) closes the horizon. Everything a body can collide with agrees
with the server's walls (S33-b). The halt and the pier are S33-e's; the interiors are S33-f's.

---

## 2. Scope

### 2.1 In scope

1. `clients/3d-spike/scripts/worlds/lakeside/depiction.gd`: the stub (S33-c §5.5) replaced by the real
   depiction; the interface unchanged.
2. `clients/3d-spike/scripts/worlds/lakeside/` new builders: `terrain.gd` (`LakesideTerrain`), `water.gd`
   (`LakesideWater`), `shore.gd` (`LakesideShore`: lane setts, kerb, railing, trail, boulders, reeds),
   `houses.gd` (`LakesideHouses`: cottage row and yard, bakery and boathouse exteriors, the square's well and lime
   tree), `woods.gd` (`LakesideWoods`: species MultiMeshes, ground cover, air walls), `light.gd` (GI volume and
   probes for Lakeside).
3. `clients/3d-spike/scripts/worlds/lakeside/terrain.json`: the authored height grid's control data (§5.2).
4. `clients/3d-spike/shaders/water.gdshader`: the extension of §5.3 (uniforms with defaults that leave Social
   Café's water byte-equal in output, I-d-2).
5. Tree species parameters in `scripts/props.gd` / `scripts/procgen.gd` (new species functions beside the
   existing `broadleaf` and `conifer`; existing functions unchanged).
6. `viewpoints()` of the Lakeside depiction: far-field VP-L-1…7 final values, the eight perf views of step §8.1
   minus the five that need S33-e/f/g (F-S33d-7; §5.7), the `farfield_shots` list.
7. `tools/cli/tests/client_far_field.rs`: Lakeside's 3D far-field case (R-S33d-c-1).
8. `clients/3d-spike/scripts/slice/slice_probe_world.gd`: `--geometry` (12e's disclosed-vs-drawn probe) runs on
   Lakeside's depiction (X-8).
9. `scripts/ci_layer.py`: `clients-probes` gains `slice --world=lakeside --geometry`.
10. `clients/3d-spike/shots/lakeside/{farfield,review}/` (committed shots, the V-gate evidence).
11. `presentation/mineworld-default/3D/far_field/lakeside.yaml` and its 2D twin: **only** `datum_m` and
    `datum_source`, if Q-S33d-1 moves the lane's height (§5.2); nothing else in either file.
12. `docs/DECISIONS.md` (or `docs/decisions/` if #258 has merged): `DEP-77`, `DEP-78`, `DEP-79`, and `ARC-170` per
    Q-S33d-1.
13. `clients/3d-spike/ASSETS.md`: one row per new CC0 set (DEP-8), only if C6's frame review adopts one.

### 2.2 Not in scope

| Item | Where |
| --- | --- |
| The pier, its rails, pavilion and boats; the halt building, platform, track and buffer stop; LM-1/LM-2 | S33-e |
| Interiors (bakery, boat shed, halt), sit points, the `--seat` probe | S33-f |
| People, decor figures, the fishing pose | S33-g |
| 2D façades and names | S33-h |
| `3d-lakeside.pck`, the size gate, the operator's perf session | S33-j |
| Time of day, weather | TW-e (the slice's fixed sun is kept) |
| Any server, protocol, kernel, contract, System or world-data change | never (I-S33-1) |

### 2.3 Dependencies

Required before C0: S33-a (far fields), S33-b (floors), S33-c (seam), 12e (disclosed geometry). Coordinated:
S33-e builds on this PR's terrain and water (R-S33e-d-1); S33-f's interiors sit inside this PR's exteriors
(R-S33f-d-1).

---

## 3. Audit (`origin/main @ a1e05874`)

### 3.1 Source anchors (paths under `clients/3d-spike/` unless stated)

| # | Fact | Where |
| --- | --- | --- |
| A-1 | The spike's lake: `const WATER_Y := -1.15`, `SHORE_Z := -28.0`; `_terrain`, `_lake`, `_mountains`, `_far_shore`, `_lake_trail` (a jetty and a rowing boat at :553–566) | `scripts/town.gd:13, 59, 61–196, 534–584` |
| A-2 | Tree builders: `Props.broadleaf(parent, pos, h := 7.5, seed_v)`, `Props.conifer(parent, pos, h := 9.0, seed_v)`, `flowerbed`, `grass_patch`, `boulder(parent, pos, r, seed_v)`, `railing(parent, from, to, h := 1.05)`; card textures from `Procgen.leaf_card`, `needle_card`, `flower_card`, `grass_card`, `treeline_card` (generated at load, zero texture bytes) | `scripts/props.gd:138, 191, 214, 227, 241, 438`; `scripts/procgen.gd:54–175` |
| A-3 | The water shader: 65 lines; uniforms `shallow`, `deep`, `wave_scale` 22.0, `wave_strength` 0.13, `sky_horizon`, `fresnel_cap` 0.6; one `fragment()` | `shaders/water.gdshader:16–36` |
| A-4 | The texture library: 22 Poly Haven 1k sets, among them `leafy_grass`, `cobblestone_floor_08`, `weathered_brown_planks`, `brown_planks_09`, `roof_slates_03`, `clay_roof_tiles_02`, `stone_brick_wall_001`, `sandstone_blocks_05`, `painted_plaster_wall`, `beige_wall_001`. **No** bark, forest-floor, granite or shingle set | `assets/textures/` |
| A-5 | `Mats.pbr(slug, metres, tint, rough)` is the one material entry point | `scripts/mats.gd:42` |
| A-6 | GI: `_gi_from_args`, one `VoxelGI` over the café (`SUBDIV_128`), two reflection probes | `scripts/slice/slice_main.gd:232, 389–450` |
| A-7 | RL-e0's view: `SliceFarFieldView`, `EYE0`, `PROXY_M` near/mid/far 1 480/1 520/1 560, `OUTER_M` 1 450, `SEA_RIBBON_M` 1 400 | `scripts/slice/far_field_view.gd:31–41` |
| A-8 | The far-field check: 1 440 × 240, HFOV 50°, 8 azimuths, `N_BELOW` 0.90, `N_ABOVE` 0.25; `VIEWPOINTS` and `SHOTS_DIR` are Social Café's (S33-c moves them into the depiction) | `scripts/slice/slice_far_field_check.gd:22–58` |
| A-9 | Merge and visibility levers: `SliceBatch` (24 m cells), `SliceBudget`, `SliceOccluders` | `scripts/slice/{batch,budget,occluders}.gd` |
| A-10 | 12e's client bodies: `SliceClientBodies` builds invisible colliders under `DisclosedGeometry` from `place-shape` (perimeter walls 0.20 m × 3.0 m, door gaps 2.0 m within 0.5 m of an edge); "Where they disagree, that is a content defect for the probe (`--world --geometry`) to name" | 12e `scripts/slice/client_bodies.gd:1–32` |
| A-11 | S33-b's floors and solids (lane frame), translated by S33-c §5.5 (`scene = (x/1000 − 15, 0, 3 − y/1000)`): lane x −15…15, z −3…3; cottages x −15…−9, z −3…−17 (pump at x −12, z −10); boathouse x −9…−3, z −3…−15; bakery x −3…3, z −3…−11; square x 3…15, z −3…−15 (well x 9, z −9; lime trunk x 12, z −12); shore x −15…0.6, z 3…13 (water line z 13); pier x 1…5, z 3…23 | `pr-s33b-lakeside-bodies.md` §5.3; `pr-s33c-world-seam.md` §5.5 |
| A-12 | S29-a's Lakeside halt (rev. 3, ruled 2026-10-18): `station` 8 × 5 m joined to the lane's east edge → scene x 15…23, z −2.5…2.5; `station-platform` 4 × 40 m north–south, track on its **east** edge → scene x 23…27, z −36…4, buffer stop at z 4; ticket bay 4.6 × 4.6 m | `pr-s29a-town-contract.md` §5 (Lakeside's halt table); this PR's arithmetic |
| A-13 | S33-a: `datum_m` 1 408 = full pool 1 406 m + 2 m "(approximate; S33-b's floors and S33-d's grading fix it)"; `level_m` 1 406; `reach_m` 80; `beyond: planting` | `pr-s33a-setting-far-field.md` §5.4 |
| A-14 | The player capsule r 0.30, h 1.72, floor limit 52°; steps are hidden wedge ramps; height never reported | `scripts/build.gd`; `scripts/player.gd` |

### 3.2 Findings

| ID | Finding | Resolution |
| --- | --- | --- |
| **F-S33d-1** | **Two metres is too high for the pier.** S33-a's datum puts the lane 2 m above full pool. Step §4.3 puts the pier deck ≈ 0.6 m above full pool. The pier's foot is on the lane, so a walker would drop 1.4 m in the deck's first metres. A ramp at the accessible slope of 1:12 (2010 ADA Standards §405.2) needs 16.8 m of run, most of a 20 m pier; stairs would break 1005.1 ("accessible routes serving fishing piers … shall comply with Chapter 4"). A deck at lane level instead stands 2 m over the water, which needs a 42 in guard (IBC §1015.3) and loses the fishing rail of 1005.2.1 | **Q-S33d-1**: lower the lane to 1.0 m above full pool (S33-a's note allows it: "S33-d's grading fixes it") and put the deck 0.70 m above full pool (§5.2). The foot then needs a 0.30 m ramp of 3.6 m at 1:12, inside the pier's first 4 m |
| **F-S33d-2** | **The water does not reach the pier from the shore place alone.** The shore's floor ends at its water line, scene z 13 (A-11), but the pier's floor starts at z 3. A drawn shore 10 m deep beside a pier that is over water from its foot needs the water line to turn north at the pier | The drawn water line follows the shore place's south edge (z 13) from x −15 to the shore floor's east edge (x 0.6), turns north along a short stone revetment at x ≈ 0.8 to z 4 at the pier's foot, and runs east at z 4 past the halt's buffer stop to x 30 (A-12; §5.4). East of the pier a quay wall stands on the water line at z 4, 1 m south of the lane's edge (§5.4). The server needs nothing new: the lane's south edge is already a wall except at its two doors (S33-b) |
| **F-S33d-3** | **The boathouse is on the land side.** World data put it north of the lane (A-11); no slip can reach the water without crossing the lane | Drawn as a boatbuilder's shed: a boat on a cradle inside (S33-b's solid), wide doors to the lane. Moored boats are S33-e's, at the pier. Recorded, not changed (I-S33-1) |
| **F-S33d-4** | **The real woods burned.** The 2003 Cedar Fire burned most of the Cuyamaca forest; the step's source says it is "regrowing" (step §3.2). A dense mature conifer ring would be a pre-2003 picture | **Q-S33d-2**: the ring as today's mixed regrowth (survivor pines and cedars, young pines, resprouted black oak, chaparral understorey), dense enough to close the land sectors; or pre-fire mature woods (reference 06's mood) |
| **F-S33d-5** | **Four needed materials are missing** (bark, forest floor, granite, shingle; A-4). The step allows at most four CC0 sets "only if a reviewed frame shows the shipped sets cannot read as the material" | C6's frame review decides each; each adopted set gets a DEP-8 row under `DEP-79` |
| **F-S33d-6** | **12e's probe exists only for Social Café** (`--world --geometry`, A-10) | C9 runs it on Lakeside's depiction; X-8 is planted there |
| **F-S33d-7** | Five of step §8.1's eight perf views need content from later PRs (`pier head looking north`: the pier, S33-e; `bakery interior`, `boathouse shed`: S33-f; `lakeside crowd`: S33-g; `halt platform`: S33-e) | This PR ships the three views it can draw (`lane wide`, `shore across the lake`, `square`); each later PR adds its own (§5.7). AC-S33-5 is closed by S33-j |

### 3.3 What does not exist

No Lakeside terrain, near water, building, tree or light; no Lakeside 3D far-field case; no Lakeside geometry probe.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`; the step's §4.9 table is the decision; C1 re-verifies its facts)

| Need | Options | Decision | C1 re-verifies |
| --- | --- | --- | --- |
| Terrain (`DEP-77`) | Terrain3D (TokisanGames, MIT, C++ GDExtension) · HTerrain (Zylann, GDScript, maintenance mode) · Godot built-ins (`ArrayMesh`, `HeightMapShape3D`) | **Built-ins** | Terrain3D's platform list and Godot 4.7 support; HTerrain's licence file and Godot 4.7 status. If either fact contradicts the step's reason, **stop** (a reversed reuse decision is the operator's) |
| Water (`DEP-78`) | ours (`water.gdshader`) · community Godot 4 lake and ocean shaders | **Ours, extended** | each named candidate's licence and Godot 4.7 status, recorded in `DEP-78` |
| Vegetation (`DEP-79`) | the spike's cards · Poly Haven tree models · CC0 low-poly packs (a second style) · paid packs (Q-S33-7(c): nothing bought) | **The spike's cards**; ≤ 2 Poly Haven hero trees only if C6's review needs them | the Poly Haven model licence (CC0) and each model's triangle count |
| Rocks | the spike's procedural `boulder` · Poly Haven rock models | **The procedural boulder**, tuned to granite (rounded, exfoliated); one Poly Haven rock set only under `DEP-79`'s four | — |
| Sett lane, kerb, railing | shipped `cobblestone_floor_08`, `stone_brick_wall_001`, `Props.railing` | **Reuse** | — |

No dependency is added to any lockfile. No GDExtension is added.

---

## 5. Design

### 5.1 The scene frame

Step §4.0's frame: metres, +x east, +z south, y up, the lane's centre at the origin. **y = 0 is the lane's
surface.** Places translate by S33-c §5.5 (A-11). True north is −z (S33-a §5.1).

### 5.2 Grading (Q-S33d-1; `ARC-170`)

Recommended (option A):

| Element | Height (y, m) | Source / reason |
| --- | --- | --- |
| Full pool (the water surface) | **−1.00** | the lane 1.0 m above the lake; S33-a's datum becomes 1 407 m (= 1 406 + 1.0); approximate, stated in `datum_source` |
| Lane | 0.00, crowned + 0.04 at the centre line, falling to the kerbs | a sett lane's camber; approximate |
| Pier deck (S33-e) | −0.30 (0.70 m above full pool) | below the 30 in (762 mm) drop at which IBC §1015.2 requires a guard, so 2010 ADA §1005.2.1's 34 in (865 mm) fishing rail applies |
| Pier foot ramp (S33-e) | 0.00 → −0.30 over 3.6 m | 1:12, the ADA §405.2 maximum running slope |
| Shore | 0.00 at its door, falling to −1.00 at the water line (z 13) over 10 m, mean slope 5.7° | packed earth and grass; under the capsule's 52° (A-14) by far |
| Cottage yard, boathouse apron, square | +0.00 … +0.60, rising gently north (the hillside behind the village) | the north shore rises from the lake (S33-a's 3DEP profile gives the real rise; C2 copies the first 60 m of the north-sector profile and scales nothing) |
| The woods' floor | follows S33-a's 3DEP near-rim samples from 60 m to `reach_m` | cited; nothing invented |
| Lake bed under the near water | −1.0 at the water line to −4.0 at 30 m from shore | approximate; only the depth fade (§5.3) sees it |

Option B keeps S33-a's 2 m and puts the deck at lane level (2.0 m above pool), with a 42 in guard
(IBC §1015.3) and no 34 in fishing rail. Option C keeps 2 m and puts the deck 0.6 m above pool with a
16.8 m ramp. C is rejected in the question as unworkable on a 20 m pier.

**The height grid** (`terrain.json`): a 201 × 201 grid at 1 m over x −100…100, z −100…100 is **not** stored.
Stored instead are: a list of named polygons, each with a height or a plane (lane, yard, square, apron, shore
slope, revetment), a list of the 3DEP profile samples copied from S33-a's `SOURCE.md` with their line, and the
blend radius (2 m). `LakesideTerrain` rasterises the grid at load (deterministic; no noise without a fixed seed;
the seed is written in the file). The file stays under 20 KB.

**Collision.** One `HeightMapShape3D` on `LAYER_WORLD` from the same grid. People stand on it by the ray of step
§4.6. Walls stay the server's (12e's `DisclosedGeometry`); the terrain decides only height.

### 5.3 The near lake (`DEP-78`)

- One `PlaneMesh` (subdivided 4 × 4) at y −1.00 from the drawn water line to a 160 m square around the origin, under
  RL-e0's far-field water sector, which continues it at `level_m` and the same `deep`/`shallow` colours (S33-a's
  file). The near plane's outer edge lies under the far-field annulus (A-7), so no seam shows at the horizon.
- `water.gdshader` gains, each behind a uniform whose default reproduces today's output:
  - `depth_fade_m` (default 0.0 = off): reads the depth texture; blends `shallow` → `deep` over that depth. Lakeside
    sets 1.5 m.
  - `shore_foam` (default 0.0): a thin lighter band where depth < 0.05 m; Lakeside sets 0.25. No foam simulation.
  - `lily_tex` and `lily_mask` (default none): lily pads drawn as a masked decal in the shallows west of the pier
    (reference 06). The mask is a 256 × 256 single-channel image generated at load by `Procgen`; zero bytes.
- **Reflections:** one `ReflectionProbe` over the near lake (box 60 × 20 × 40 m, update once), plus the sky
  radiance. SSR stays **off**; C8 measures it once on the `shore across the lake` view and records the cost.
  Planar reflection stays rejected (step §4.1).

### 5.4 Shore, lane and trail

- **Lane:** `cobblestone_floor_08` at 1.0 m repeat, a 150 mm granite kerb on the south side
  (`stone_brick_wall_001` tinted), gaps at the shore and pier doors.
- **Quay and railing:** the quay wall and the water line stand together at **z 4**, from the pier's east edge
  (x 5) to just past the track's end (x 30; the track's centre line is at x ≈ 28.6, S33-e §5.2). The wall falls
  1.0 m to the water. Between the lane's south edge (z 3) and the quay (z 4) lies a 1 m stone coping and grass
  verge; east of x 15 the verge runs between the waiting room's south wall (z 2.5) and the quay. The verge is
  **not walkable**: no place's floor covers it, so the server never puts anyone there. A timber railing 1.07 m
  high (42 in, IBC §1015.3's guard height) runs along the lane's south edge (z 3) from x 5 to x 15, where the
  server's wall is; it marks the edge a walker meets. West of the pier the lane's edge meets the shore's grass at
  grade; a low timber rail (0.9 m) marks it from x −15 to x 0.6, broken at the shore door (x −3).
- **Trail:** a packed-earth trail 1.5 m wide from the shore door west along the shore to x −15 and into the woods
  edge (reference 06), ending at the woods' air wall. Its surface is the terrain's splat layer; nothing extra.
- **Boulders:** S33-b's three shore boulders (solids, h 0.8) drawn at their footprints by `Props.boulder`, tuned to
  granite; up to twelve more beyond the shore floor (on the revetment and at the woods' edge), where nobody walks.
- **Reeds:** card clumps along the water line west of the pier, ≤ 40 clumps, one `MultiMesh`.

### 5.5 The village exterior

Every footprint is S33-b's (A-11); every solid S33-b names outdoors (the pump, the well, the lime trunk, the
square's benches) is drawn at its footprint within 50 mm (I-d-4). Façade fronts stand on the lane's north edge
(z −3).

| Building | Exterior (6 m frontages, Q-S33b-2 A) | Materials (shipped first) |
| --- | --- | --- |
| Cottages | the yard (6 × 14 m) behind a low wall and gate on the lane; four one-and-a-half-storey cottages round it, 5 × 6 m footprints outside every place's floor: two on the yard's west side (x −20…−15) and two on its north side (z −23…−17); **none east of the yard**, where the boathouse stands (x −9…−3); doors **closed** (Q-S33-5); the pump at the yard's centre | `painted_plaster_wall`, `roof_slates_03`, timber doors |
| Boathouse | a two-storey timber shed (6 m frontage, 12 m deep), wide double doors to the lane open on the shed (S33-f draws inside), rooms above with windows (no upstairs place; nothing enterable) | `weathered_brown_planks` board-and-batten, shingle roof (a DEP-79 candidate, else `roof_slates_03`) |
| Bakery | a two-storey stone house (6 × 8 m), a shop window onto the lane where the long table stands inside (S33-b's solid), a chimney over the oven wall | `sandstone_blocks_05`, `clay_roof_tiles_02` |
| Square | open, 12 × 12 m, setts; the well (1.2 m stone drum, h 0.9 m); the lime tree (a broadleaf, *Tilia*, 12 m; a planted village tree, step §4.2); the two benches under it (`painted_wooden_bench`, S33-f adds sit points) | shipped |

Façade detail follows the slice's structure rules (VISUAL_SLICE §6.2: plinth, openings with reveals, eaves).
Interiors are not drawn here: windows show a dark interior card until S33-f, and the bakery door is closed until
S33-f opens it (a door the server lets you through is drawn open by S33-f in the same PR that draws the room).

### 5.6 The woods (`DEP-79`; Q-S33d-2)

| Species | Real height (cited) | Drawn heights | Method |
| --- | --- | --- | --- |
| Jeffrey pine (*Pinus jeffreyi*) | "Trees 24–61 m tall, 60–250 cm dbh" ([Gymnosperm Database](https://www.conifers.org/pi/Pinus_jeffreyi.php)) | survivors 24–32 m; regrowth 4–10 m | `Props.conifer`, long needle cards |
| Coulter pine (*Pinus coulteri*) | "Trees to 24 m; trunk to 1 m diam." ([Gymnosperm Database](https://www.conifers.org/pi/Pinus_coulteri.php)) | 8–18 m | `Props.conifer`, open crown variant |
| Incense cedar (*Calocedrus decurrens*) | "18–46(–57) m tall and 90–150(–360) cm dbh" ([Gymnosperm Database](https://www.conifers.org/cu/Calocedrus_decurrens.php)) | survivors 18–26 m; young 3–8 m | new `Props.cedar`: a narrow columnar crown of flat-spray cards |
| California black oak (*Quercus kelloggii*) | "typically 30 to 80 feet (9–25 m) in height and 1 to 4.5 feet (0.3–1.4 m) DBH at maturity" ([USDA FEIS](https://www.fs.usda.gov/database/feis/plants/tree/quekel/all.html)) | 6–14 m (many resprouted after 2003) | `Props.broadleaf`, oak leaf card |
| Chaparral understorey | — | 1–2.5 m shrubs | `shrub` cards; one MultiMesh |

- C0 reads the Silvics of North America chapters (Jenkinson 1990 for Jeffrey pine; the others' chapters) and
  replaces a Gymnosperm Database range with Silvics' where they differ, recording both (I-S33-7).
- **Placement:** Poisson-disc, seed 33, density rising from 0 at 30 m to a closed stand at 60 m, to `reach_m`
  (80 m). No tree on a place's floor, on the lane or within 2 m of the trail. Species mix by distance and slope
  (oaks low and near the water, pines on the rises).
- **LOD:** two bands per species (≤ 40 m: full cards; > 40 m: the `treeline_card` impostor). One `MultiMesh` per
  species and band: ≤ 10 draws for the woods.
- **Air walls** (allowed, step §0.3): invisible `StaticBody3D` boxes on `LAYER_WORLD` behind the first rank at
  60 m, and along the lake at the shore floor's south edge extended. The server's floors already stop every walk;
  the air walls only keep the camera rig and a free-moving debug capsule inside.
- **Ground cover:** grass and wildflower cards (06's daisies and lupins; the Cuyamaca meadow flora is C0's to cite
  if a species is named) in clumps along the trail and the shore, visibility range 30 m.

### 5.7 Light and views

- **GI:** one `VoxelGI` over the village's north side (x −16…16, z −18…0, height 10 m, `SUBDIV_128`) and one
  reflection probe over the lane, the café pattern (A-6). The fixed sun and HDRI are the slice's (A-5 of the step).
- **Far-field viewpoints** (the step's VP-L-1…7, final values from A-11 and §5.2): VP-L-1 Elin's start in the yard
  (scene (−10.5, eye, −4.5): `cottages` (3500, 1500) → lane (4 500, 7 500) translated), VP-L-2 (0, 1.66, 0), VP-L-3 the pier head
  (3, −0.30 + 1.66, 22), VP-L-4 the shore's water line (−7, −1.0 + 1.66, 12.6), VP-L-5 the well (9, 1.66, −9),
  VP-L-6 the platform's south end (25, 1.66, 3.5), VP-L-7 (0, 40, 0). VP-L-3 and VP-L-6 stand on S33-e's decks;
  until S33-e lands they are judged at the same points over the water and the quay.
- **Perf views (this PR's three):** `lane wide` (−14, 1.66, 0, yaw 90°, looking east along the lane and the lake),
  `shore across the lake` (−7, 0.66, 12, yaw 210°: the peak's bearing), `square` (8, 1.66, −4, yaw 0°, looking
  north). Gates per step §8.1; counts recorded locally (R-LOAD-1); timings come from the operator (S33-j).

### 5.8 Records

- `DEP-77`, `DEP-78`, `DEP-79` as the step's §4.9, with C1's re-verified facts and dates.
- `ARC-170` (Q-S33d-1): Lakeside's lane stands 1.0 m above full pool, the pier deck 0.70 m above it; the far
  field's datum follows (1 407 m). Every height is presentation; the server stays planar (F-S33-4).

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-d-1 (I-S33-1) | No path under `kernel/`, `contracts/`, `systems/`, `server/`, `worlds/`, `clients/protocol/` changes | C10 diff scan |
| I-d-2 (I-S33-2) | Social Café is unchanged: its shots within `frame_diff`'s 8/255 of C0's capture (the shader's new uniforms default off) | AC-d-6 |
| I-d-3 (I-S33-4) | Every Lakeside viewpoint closes on a far field | AC-d-1 |
| I-d-4 (I-S33-9) | Every drawn wall and solid of a Lakeside place lies within 50 mm of a disclosed face; no drawn wall without a disclosed one | AC-d-3 (12e's `--geometry` on Lakeside) |
| I-d-5 (I-S33-5) | Nothing pretends: closed doors stay closed; no moving prop; no lit window without an interior behind it | review; S33-e's honesty scan |
| I-d-6 (I-S33-6) | One method: our builders, the shipped library, ≤ 4 new CC0 sets each with a DEP-8 row; no generated building | review; `ASSETS.md` |
| I-d-7 (I-S33-7) | Every physical value cites a source or is marked approximate with its replacer | review of §5's tables at C10 |
| I-d-8 (I-S33-3) | No place-key literal in any Lakeside builder (S33-c's check (d)) | `client_rules.rs` |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static scan | I-d-8 | `cargo test -p mineworld-acceptance --test client_rules` (CI `core`) |
| Rendered (Godot) | AC-d-1: Lakeside far field at VP-L-1…7 × 8 azimuths | `tools/cli/tests/client_far_field.rs`, `#[ignore]`, nightly `clients` on ubuntu-24.04, macos-26, windows-2025; RL-e0's per-PR Linux `farfield` job |
| Probe (Godot, real server) | AC-d-3 (`--world=lakeside --geometry`); AC-d-4 (`--world=lakeside --link` on the real depiction) | nightly `clients-probes`; local one at a time |
| Parity | AC-d-6 (Social Café shots) | local `frame_diff.gd` against C0's capture; nightly |
| Data | `terrain.json` loads and rasterises identically twice (a hash printed by `--layout`) | the probe |
| Gate 1 / 2 | NOT REQUIRED | — |

---

## 8. Acceptance criteria (fixed before measuring)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-d-1 | `client_far_field` Lakeside case: FF-BELOW ≥ 90 % and FF-ABOVE ≥ 25 % at VP-L-1…7 × 8 azimuths, S/H/E frames, on Linux, macOS and Windows | AC-S33-1 |
| AC-d-2 | X-1: deleting the 3D water body fails by name (`FAR_FIELD_GAP` at validate, or `FF-BELOW` in S, SW at VP-L-3) | X-1 |
| AC-d-3 | `--world=lakeside --geometry` passes; X-8 (a drawn bakery wall with no disclosed solid) fails naming the place and the face | X-8 |
| AC-d-4 | `--world=lakeside --link` on the real depiction: the probe route of S33-c passes, 0 refusals, the player stands on the terrain (ray hit within 0.05 m of the terrain's height at 20 sampled points) | §12.1 checkpoint |
| AC-d-5 | Counts for the three perf views recorded (draws, primitives, video memory) and each under step §8.1's advisory figures; timings INCONCLUSIVE locally | §8.1 |
| AC-d-6 | Social Café's `--shots` and `--farfield-shots` within 8/255 of C0's capture | I-S33-2 |
| AC-d-7 | Lakeside's 3D bytes added by this PR ≤ 8 MB (half of the 16 MB gate, leaving S33-e/f the rest), measured as the summed size of `scripts/worlds/lakeside/`, new shaders and any new CC0 sets | §8.2 |
| AC-d-8 | The operator's first `VIS-3D-LAKESIDE` preview: the committed `review/` shots (§9 C10) sent through the lane lead | ARC-24 |
| AC-d-9 | The diff touches only §2.1's paths | I-d-1 |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and its evidence; inapplicable → `N/A` with the audited reason.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s33d`. Steps 1–10 of
[`pr-s33a-setting-far-field.md`](pr-s33a-setting-far-field.md) §9.0 apply. Godot only through
`/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`, one run at a time. Checks: `cargo test -p
mineworld-acceptance --test client_rules`; `cargo test -p mineworld-cli --test client_far_field -- --ignored
<one test>`; `./mineworld-slice --world=lakeside …` probes; `frame_diff.gd`; doc scripts.

#### C0 — Base, prerequisites, ledger, captures (Markdown only)

- **Files:** this file (§15), `handoff-s33d.md` (new).
- Steps: record the base; confirm S33-a, S33-b, S33-c, 12e merged (any missing → material stop); re-read A-1…A-14
  and record moved lines; capture Social Café's `--shots` and `--farfield-shots` into the scratchpad (sizes and
  base SHA in §15); read the Silvics chapters for §5.6's species and record each range.
- [ ] Implementation. [ ] Validation: doc scripts. [ ] Review: every prerequisite with evidence.
- **Commit message:** `docs(plan): S33-d — open the ledger at the base`

#### C1 — Records; the reuse facts re-verified; the grading

- **Files:** `docs/DECISIONS.md` (or `docs/decisions/`), the two `far_field/lakeside.yaml` files (`datum_m`,
  `datum_source` only, if Q-S33d-1 moves it), this file.
- Steps: §4's re-verifications, each fact with its URL and date; `DEP-77…79`; `ARC-170` per Q-S33d-1.
- Validation: doc scripts; `mineworld packs validate` on both packs; `cargo test -p mineworld-packages --test
  far_field` (AC-FF-4 still holds: the two files' water bodies are unchanged).
- [ ] Implementation. [ ] Validation. [ ] Review: no reuse fact reverses the step's decision (else stop).
- **Commit message:** `docs: DEP-77…79 and ARC-170 — Lakeside's terrain, water and woods (S33-d)`

#### C2 — Terrain (Haiku)

- **Files:** `scripts/worlds/lakeside/{terrain.gd, terrain.json}`, `depiction.gd`, this file.
- Steps: §5.2: the polygons, the copied 3DEP samples, rasterising, the splat (≤ 4 layers: setts, grass, earth,
  forest floor — forest floor uses `leafy_grass` darkened until C6 decides), the `HeightMapShape3D`.
- Validation: the rasterised grid's hash printed twice equal; AC-d-4's ray samples. Mutation **M-d-1** (the shore
  polygon's water-line height set to 0.0) → AC-d-4's ray check fails at the shore samples by name.
- [ ] Implementation. [ ] Validation. [ ] Review: every height traces to §5.2's table or a `SOURCE.md` line.
- **Commit message:** `feat(3d): Lakeside's graded terrain from an authored grid and cited profile (S33-d)`

#### C3 — The near lake (Haiku)

- **Files:** `scripts/worlds/lakeside/water.gd`, `shaders/water.gdshader`, `depiction.gd`, this file.
- Steps: §5.3.
- Validation: AC-d-6 (Social Café's water unchanged); AC-d-1 run once on the partial depiction (recorded, not
  gated). Mutation **M-d-2** (the shader's `depth_fade_m` default set to 1.5) → AC-d-6 fails on Social Café's
  water frames.
- [ ] Implementation. [ ] Validation. [ ] Review: no seam between the near plane and the far-field water in the
  S frame at VP-L-4 (recorded shot).
- **Commit message:** `feat(3d): Lakeside's near lake — depth fade, shore band, lily pads (S33-d)`

#### C4 — Shore, lane, quay and trail (Haiku)

- **Files:** `scripts/worlds/lakeside/shore.gd`, `depiction.gd`, this file.
- Steps: §5.4; F-S33d-2's water line and revetment.
- Validation: AC-d-4. Mutation **M-d-3** (one shore boulder moved 0.3 m off S33-b's footprint) → `--geometry`
  names the boulder.
- [ ] Implementation. [ ] Validation. [ ] Review: railing heights cite §5.4's clauses.
- **Commit message:** `feat(3d): Lakeside's lane, shore, quay and trail (S33-d)`

#### C5 — The village exterior (Opus)

- **Files:** `scripts/worlds/lakeside/houses.gd`, `depiction.gd`, this file.
- Steps: §5.5.
- Validation: AC-d-3 on the houses; the frame review of `review/` shots VR-1 (the lane from VP-L-2 looking north)
  and VR-2 (the yard). Mutation **X-8** (a drawn bakery wall with no disclosed solid) → `--geometry` red by name.
- [ ] Implementation. [ ] Validation. [ ] Review (Opus, frames): reads as references 01/06's idiom; scale against
  VISUAL_SLICE §3 (door 2.1 m, sill 0.9 m, storey 2.8–3.0 m).
- **Commit message:** `feat(3d): Lakeside's cottages, bakery and boathouse exteriors, the square (S33-d)`

#### C6 — The woods (Opus)

- **Files:** `scripts/worlds/lakeside/woods.gd`, `scripts/props.gd`, `scripts/procgen.gd` (new species functions
  only), `ASSETS.md` (only if a set is adopted), `assets/textures/<set>/` (only if adopted), this file.
- Steps: §5.6; the per-material frame review (bark, forest floor, granite, shingle): adopt a CC0 set only where the
  frame shows the shipped material cannot read as it, ≤ 3 MB each, DEP-8 row.
- Validation: AC-d-1 in full (the land sectors now close on woods); counts; Social Café's tree functions unchanged
  (AC-d-6). Mutation **M-d-4** (the woods' ring removed) → AC-d-1 FF-BELOW fails in N, NE, NW at VP-L-5 by name.
- [ ] Implementation. [ ] Validation. [ ] Review (Opus, frames): species read as named; no suggested path beyond
  the boundary.
- **Commit message:** `feat(3d): Lakeside's woods — pine, cedar and oak cards, closed at the boundary (S33-d)`

#### C7 — Light (Opus)

- **Files:** `scripts/worlds/lakeside/light.gd`, `depiction.gd`, this file.
- Steps: §5.7's GI and probes.
- Validation: counts; video memory recorded. Frame review VR-3 (the shore across the lake at the fixed sun).
- [ ] Implementation. [ ] Validation. [ ] Review: no light leak under the eaves; the water's probe reflects the
  village, not the café.
- **Commit message:** `feat(3d): Lakeside's GI volume and reflection probes (S33-d)`

#### C8 — Far-field case, viewpoints, perf views (Haiku)

- **Files:** `depiction.gd` (`viewpoints()`), `tools/cli/tests/client_far_field.rs`, `scripts/ci_layer.py`,
  `shots/lakeside/farfield/`, this file.
- Steps: §5.7's viewpoints and perf views; the Lakeside case in `client_far_field.rs`; SSR measured once (recorded).
- Validation: AC-d-1 locally once; AC-d-2 (X-1 planted in the 3D file, reverted by `cmp`); AC-d-5.
- [ ] Implementation. [ ] Validation. [ ] Review: verdict lines unchanged in form.
- **Commit message:** `test(3d): Lakeside's far-field check at seven viewpoints and its perf views (S33-d)`

#### C9 — The geometry probe on Lakeside (Haiku)

- **Files:** `scripts/slice/slice_probe_world.gd`, `scripts/ci_layer.py`, this file.
- Steps: 12e's `--geometry` reads the current depiction's volumes; `clients-probes` gains the Lakeside run.
- Validation: AC-d-3; X-8 re-run red then green.
- [ ] Implementation. [ ] Validation. [ ] Review: the probe compares every drawn collider in a place's volume, not
  only walls.
- **Commit message:** `test(3d): drawn and disclosed walls agree on Lakeside (S33-d)`

#### C10 — Close-out and the operator's preview

- **Files:** this file (§15), `handoff-s33d.md`, `shots/lakeside/review/` (VR-1…VR-5: the lane, the yard, the
  shore across the lake, the square, the high view).
- Steps: AC-d-7, AC-d-9; push; `gh pr create` with the shots; CI on the exact head; repair to green; the preview
  to the lane lead for the operator (AC-d-8).
- [ ] Implementation. [ ] Validation: CI green on the exact head. [ ] Review: deviations recorded.
- **Commit message:** `docs(plan): S33-d close-out`
- **Then:** train-ready; READY FOR OPERATOR REVIEW of the V-gate shots.

---

## 10. Requirements this PR places on other PRs

- **R-S33e-d-1 (on S33-e).** The pier deck at y −0.30, its foot ramp 1:12 from the lane (Q-S33d-1 as ruled); the
  buffer stop just inside the quay line (z ≤ 4.0; S33-e puts it at z 3.5); VP-L-3 and VP-L-6 re-judged on S33-e's decks.
- **R-S33f-d-1 (on S33-f).** Interiors fit inside §5.5's shells; S33-f opens the bakery and boat-shed doors.
- **R-S33a-d-1 (on S33-a's file, through this PR).** `datum_m` 1 407 if Q-S33d-1 is ruled A (this PR edits it,
  §2.1 item 11).

## 11. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-33-lakeside-3d.md` §4.1 | the water line at y −1.0 (Q-S33d-1); F-S33d-2's revetment and quay |
| `step-33-lakeside-3d.md` §4.3 | the deck 0.70 m above full pool (Q-S33d-1) |
| `step-33-lakeside-3d.md` §4.2 | the woods as today's regrowth (Q-S33d-2 as ruled) |
| `pr-s33a-setting-far-field.md` §5.4 | `datum_m` 1 407 (a dated note) |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S33-d — Lakeside's terrain, water and village exterior in 3D
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s33d-terrain-water-exterior.md
RELATED / BINDING DOCS:  step-33-lakeside-3d.md; pr-s33a-setting-far-field.md; pr-s33b-lakeside-bodies.md;
                         pr-s33c-world-seam.md; ../mvp1/pr-rl-e0-far-field-mandatory.md; docs/VISUAL_SLICE.md;
                         docs/ART_DIRECTION.md; docs/REUSE_POLICY.md; CLAUDE.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s33d, its own, held by one session
BRANCH:                  mvp0/pr-s33d, created from origin/main
START CONDITION:         S33-a, S33-b, S33-c and 12e merged; §14 ruled; this design frozen; the lane lead's
                         dispatch (NONE met at 2026-10-18)
IMPLEMENTATION BASE:     origin/main at C0
APPROVED SCOPE:          §2.1 items 1–13
FROZEN INVARIANTS:       step-33 I-S33-1…I-S33-9 as they apply; §6 I-d-1…I-d-8; S33-c's interface unchanged;
                         RL-e0 format 1 unchanged
SEQUENCE:                C0 … C10 (§9)
ALLOWED COMMANDS:        cargo build -p mineworld-cli; cargo test -p mineworld-acceptance --test client_rules;
                         cargo test -p mineworld-packages --test far_field; cargo test -p mineworld-cli --test
                         client_far_field -- --ignored <one test>; ./mineworld-slice probes through
                         cast-slots/with-slot.sh, one at a time; godot --headless for frame_diff; mineworld
                         packs validate; python3 scripts/*; git; gh (never merge)
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace; edits
                         under kernel/, contracts/, systems/, server/, worlds/, clients/protocol/; a GDExtension;
                         a paid or generated asset; a second art method
MATERIAL STOPS:          a prerequisite missing; a reuse fact reversing the step's decision; Social Café parity
                         unattainable; a drawn wall the server's floors cannot match without a world change; a
                         perf count that forecasts p95 > 16.7 ms with the levers applied; bytes over AC-d-7
PLATFORMS:               rendered checks on ubuntu-24.04, macos-26, windows-2025 (nightly); per-PR Linux farfield
VALIDATION BUDGET:       targeted local runs as listed; CI: the PR's runs. Gate 1/2: NOT REQUIRED
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s33d.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                         semantic commits, branch push, PR creation, CI repair: authorized
                         merge: through a merge train, under the lane-lead charter's merge authority, after the
                         operator's V-gate look at the preview shots (AC-d-8)
POST-MERGE SYNC OWNER:   primary: step-33, S33-a design note; implementation session: this ledger
STOP CONDITION:          S33-d train-ready with the preview sent
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-d-1 | Card trees read cheap beside reference 06 (R-S33-3) | the operator sees the woods in the C10 preview; levers: ≤ 2 Poly Haven hero trees, one bark set |
| R-d-2 | Water with the depth texture costs fill rate at low views | SSR off; the probe; the fade costs one depth read |
| R-d-3 | Q-S33d-1 ruled B (deck at lane level) | §5.2's option B is fully specified; S33-e follows it |
| R-d-4 | 12e's `--geometry` cannot see presentation colliders (terrain, rails) and flags them | the probe compares only colliders inside a place volume on the wall layer; terrain and air walls are excluded by group, recorded |
| R-d-5 | The far-field near rim from 3DEP sits higher than the drawn woods at 80 m, leaving a band | the woods' floor follows the same samples (§5.2); FF-BELOW catches a gap |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S33d-1 [operator — the lane's height above the lake]** | S33-a set the lane 2 m above the lake, approximate. With a 2 m drop, a fishing pier needs either a 17 m ramp or a deck at lane level with a tall guard and no fishing rail (F-S33d-1). (A) Lower the lane to 1.0 m above full pool and the deck to 0.70 m, with a 3.6 m ramp at 1:12; (B) keep 2 m, deck at lane level, 42 in guard; (C) keep 2 m, deck 0.6 m, 17 m ramp? | **(A).** It keeps the pier accessible (ADA §405.2, §1005.1) and fishable over a 34 in rail (§1005.2.1), and a village a metre above a reservoir's full pool is realistic. Only `datum_m` changes in S33-a's files (1 408 → 1 407). *Chinese:* 湖边小路离湖面从 2 米降到 1 米，码头面离水 0.7 米，这样坡道短、栏杆能钓鱼——同意吗？ |
| **Q-S33d-2 [operator — what the woods look like]** | The real Cuyamaca forest largely burned in 2003 and is regrowing (F-S33d-4). Draw (A) today's regrowth — scattered survivor pines and cedars, young pines, black oaks, chaparral — still dense enough to close the view; or (B) mature pre-fire woods like reference 06? | **(A).** The realistic-defaults rule asks for the real place today, and the mix of tall survivors over young growth reads as a real mountain lake. The boundary still closes on trees at 60–80 m. *Chinese:* 树林按 2003 年大火后真实的再生林来画（老树稀疏、幼树和橡树多），还是按参考图画成成熟老林？建议前者。 |

### 14.1 Rulings

(none yet)

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 … C10 | — | [ ] | [ ] | [ ] |

### Evidence

(none yet)

### Mutations

(none yet; M-d-1 … M-d-4, X-1, X-8 as §9 lists them)

### Deviations and findings

F-S33d-1 … F-S33d-7 (§3.2), recorded at design time.

### Design review

2026-10-18: one independent Opus reviewer read the seven S33-d…j designs against the step, the frozen S33-a/b/c
designs, step-34, S29-a rev. 3 and the source. Verdict REQUEST CHANGES, no blockers. Applied here before the PR
opened: the cottages kept west and north of the yard, never on the boathouse (§5.5); the quay wall and the water
line together at z 4 from x 5 to x 30, the verge between them not walkable (§5.4, F-S33d-2); VP-L-1 at
(−10.5, eye, −4.5); the pump at x −12; "five" perf views wait for later PRs (F-S33d-7); the buffer stop inside
the quay line (R-S33e-d-1).
