# PR S33-d — Lakeside's land, water and village exterior in 3D: terrain, the near lake, the shore, the houses, the woods

## DESIGN FROZEN — 2026-10-19 (operator approved in the primary session: every question answered with its recommendation; Q-S33h-1 OpenAI image cap US$8.00)

## Revision 2 — DESIGN FROZEN 2026-10-20 (operator, relayed by the coordinator; §14 as recommended)

```text
Design revision:        revision 2 (2026-10-20), DESIGN FROZEN 2026-10-20 (operator, relayed by the
                        coordinator: #386 merged, every §14 question as recommended).
                        Revision 1 (2026-10-18) stays FROZEN 2026-10-19 in every part this revision
                        does not name (the "Revision 2" block below lists what it changes)
Revision 2 author:      S33-d design reviser (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s33d-rev2, branch docs/s33d-rev2,
                        planning base origin/main @ e7c095d0 (S33-c, PR #381, merged)
Revision 2 cause:       operator ruling 2026-10-20 (the lane's Q4, relayed by the coordinator): Lakeside's
                        village is a data file read by generic builders, not GDScript constants
Design revision (r1):   revision 1 (2026-10-18), DRAFT
Author:                 S33 lane lead (Opus), worktree /Users/yuema137/mineworld-worktrees/design-s33de,
                        branch docs/pr-designs-s33d-j, planning base origin/main @ a1e05874. Read-only
                        references: 16c origin/mvp0/pr-16c @ 36325625, 12e origin/mvp0/pr-12e @ e5000ce7,
                        12d origin/mvp0/pr-12d-towns @ 841805ee
Approved by / evidence: the operator, 2026-10-19, in the primary session: every question answered with its recommendation; Q-S33h-1 OpenAI image cap US$8.00
                        §14's questions are open
Implementation base:    origin/main at C0; C0 records it and the merge state of S33-a, S33-b, S33-c, 12e
Start condition:        (1) S33-a merged (Lakeside's far fields); (2) S33-b merged (Lakeside's floors and
                        solids); (3) S33-c merged (the depiction seam and the Lakeside stub); (4) 12e merged
                        (disclosed walls, X-8); (5) §14 ruled; (6) this design frozen; (7) the lane lead's
                        dispatch. NONE holds at 2026-10-18.
                        At C0 (2026-10-21): (1) S33-a and (2) S33-b are OPEN, not merged, under the main
                        merge freeze; this PR is STACKED on them (#387 S33-a, #395 S33-b, #400 the stub
                        follow-up) by the coordinator's priority order (16f-c and 16f-d1 wait on C3), in
                        place of "merged". (3) S33-c merged (#381); (4) 12e merged (#254); (5), (6) the
                        operator's freeze of revision 2 (relayed); (7) the coordinator's dispatch
Execution contract:     §12
Implementer:            revision 2: Opus-class for C3 (the seam split), C7, C8, C9 (the art commits) and the
                        frame reviews; Haiku-class for C2, C4, C5, C6, C10, C11, C12 from this commit plan
                        (step-33 §12.1: "Opus (art judgement); mechanical parts to Haiku")
Lifecycle:              revision 1 DESIGN FROZEN 2026-10-19; revision 2 DESIGN FROZEN 2026-10-20
                        (§14 Q-S33d-3…Q-S33d-6 as recommended); implementation from 2026-10-21
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

> **Revision 2 (2026-10-20) — Lakeside's village is data read by generic builders.** A dated bounded revision,
> applied before S33-d starts. It answers the operator's ruling of 2026-10-20 (the lane's Q4): "Lakeside's
> village is a data file read by generic builders, not GDScript constants". It applies the operator's
> standing rule of 2026-10-18 (lane-lead charter, "No world-specific hard-coding"): "我们不能为了让 default
> 小世界跑起来，就 hardcode 一大堆东西，我们要把这些机制做的 robust 才行，不然换了一个世界就跑不了了" — mechanisms are
> generic and data-driven, and every acceptance criterion names a second world or a generated variant.
>
> **Why.** Revision 1 put Lakeside's heights, water line, quay, houses and woods into five GDScript builders
> under the world's own folder (`terrain.gd`, `houses.gd`, …). Each builder would have been Lakeside's alone.
> A second lakeside town would have needed five new scripts, and a moved cottage a code change. With the data
> in a file and the builders generic, a second town is a second file, and a moved cottage is one edited line.
>
> **What changes.**
>
> 1. **One data file per world, in the Presentation Pack:**
>    `presentation/mineworld-default/3D/depictions/lakeside.yaml` (format 1, YAML's JSON subset, `DEP-16`),
>    specified in §5.0. It holds the terrain grading, the water line and lake, the kerbs, revetment, quay and
>    railings, the trail, the house shells (massing, roofs, materials, façade openings), the woods (species
>    table, stands, density), the scattered boulders and ground cover, the light volumes, the places the
>    depiction binds, the spawn and the viewpoints. It replaces revision 1's `terrain.json` and every number
>    revision 1 would have written into a builder.
> 2. **A validator in the far field's style:** typed Rust schema in `mineworld_packages`, unknown keys
>    refused, one refusal code per kind of fault, run by `mineworld packs validate`, a fixture corpus with one
>    file per code, and a client reader that refuses by the same codes (§5.0.4; Q-S33d-4).
> 3. **Generic builders** in `clients/3d-spike/scripts/depictions/data/`: terrain from regions and height
>    profiles, water from a polygon, building shells from massing, woods from areas and a species table, edges
>    from lines, scatter from areas, light from boxes (§5.0.5). None names a world, a place key or a tag.
> 4. **Lakeside's depiction script shrinks to loading its data file:**
>    `clients/3d-spike/scripts/depictions/lakeside/depiction.gd` becomes `extends SliceDataDepiction` and a
>    header comment (AC-d-11).
> 5. **A generated variant proves the builders generic:** `scripts/depiction_variant.py` writes `lakeside-v7`
>    from Lakeside's file (seed 7), the RL-e0b far-field variant's pattern. It is drawn with no GDScript change
>    and is named in every acceptance criterion (§8; Q-S33d-6).
> 6. **Furniture never comes from this file** (`ARC-307`). The schema has no field for a free-standing piece.
>    The pump, the boat, the workbench, the well, the benches and the shore's three boulders are furnishings
>    in the world's `body:` data (S33-b revision 2 §5.3.1), drawn by `SliceFurnishings`. Revision 1 drew the
>    shore boulders, the pump, the well and the benches itself (§5.4, §5.5); that drawing is withdrawn.
> 7. **Paths follow S33-c as merged** (D-S33c-1): depictions live under `scripts/depictions/<key>/`, never
>    `scripts/worlds/<key>/`, because `client_rules.rs` check (c) refuses a literal path with a `worlds/`
>    component.
> 8. **Updated to match:** the reuse table (§4), the scope (§2.1), the invariants (§6: I-d-9…I-d-12), the
>    tests (§7), the acceptance criteria (§8: every one names `lakeside-v7`; AC-d-10…AC-d-12 new), the commit
>    plan (§9: C0…C13), the requirements on S33-e and S33-f (§10), the execution contract (§12), the risks
>    (§13: R-d-6…R-d-11) and the freeze questions (§14: Q-S33d-3…Q-S33d-6).
>
> **What stays frozen.** Every value of revision 1 — the grading of §5.2 (Q-S33d-1 ruled A), the water level,
> shader extension and reflections of §5.3, the lane, quay, railing, trail and reeds of §5.4, the shells of
> §5.5, the species, heights, placement and LOD of §5.6 (Q-S33d-2 ruled A), the light and viewpoints of §5.7 —
> is kept. Those values move from builder code into `lakeside.yaml`; none of them changes. The far-field
> checks, the geometry probe, X-1, X-8, the perf gates and the byte budget are kept. S33-c's `SliceDepiction`
> interface and its registry (ARC-144, ARC-146) are unchanged.
>
> **Records.** `ARC-177` (in the S33 lane's range ARC-170–179, assigned by the coordinator 2026-10-20): the per-world
> depiction data file of a 3D Presentation Pack (§5.8). `DEP-77`, `DEP-78`, `DEP-79` and `ARC-170` as in
> revision 1. Written by C1.

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

**Revision 2 replaces items 1–3 and amends items 5, 6, 8 and 13; items 4, 7, 9–12 stand.** The revised list:

- **R2-1.** `presentation/mineworld-default/3D/depictions/lakeside.yaml` (new): Lakeside's depiction data,
  §5.0's format, holding every value of §§5.2–5.7.
- **R2-2.** `clients/3d-spike/scripts/depictions/lakeside/depiction.gd`: the stub (S33-c §5.5) replaced by
  `extends SliceDataDepiction` and its header comment; the interface unchanged.
- **R2-3.** `clients/3d-spike/scripts/depictions/data/` (new), the generic builders of §5.0.5:
  `data_depiction.gd` (`SliceDataDepiction`), `depiction_file.gd` (`SliceDepictionFile`, the client reader),
  `terrain_builder.gd`, `water_builder.gd`, `edge_builder.gd`, `shell_builder.gd`, `woods_builder.gd`,
  `scatter_builder.gd`, `light_builder.gd`.
- **R2-4.** `clients/3d-spike/scripts/depictions/box_depiction.gd`: split so that registering volumes, entries
  and doors is one method and drawing grey walls another; `SliceDataDepiction` reuses the first. The stub's
  behaviour is unchanged for any world still drawn as a stub.
- **R2-5.** Rust: `packages/src/depiction_schema.rs`, `packages/src/depiction.rs` (new), `packages/src/lib.rs`,
  `packages/src/error.rs` (the codes); `tools/cli/src/packs.rs` (`packs validate` checks `depictions/` when a
  pack has it); `packages/tests/depiction.rs` and `packages/tests/fixtures/depiction/` (the corpus).
- **R2-6.** `clients/3d-spike/scripts/checks/depiction_check.gd` and `tools/cli/tests/client_depiction.rs`: the
  client reader run over the corpus and compared with the Rust validator (AC-d-10).
- **R2-7.** `scripts/depiction_variant.py` (new) and its committed output
  `packages/tests/fixtures/depiction/variants/lakeside-v7/lakeside.yaml`; `clients/3d-spike/scripts/slice/slice_main.gd`
  and the launcher `mineworld-slice` gain `--slice-depiction-file=<path>` / `--depiction-file=<path>`, the
  pattern of `--slice-far-field-file=` (RL-e0b).
- **R2-8.** `docs/MODULE_SPEC.md` §6.1 (the directory list) and a new §6.1.2; `clients/3d-spike/PRESENTATION.md`
  §§1, 6 (new): the file's specification for pack authors.
- **R2-9.** `docs/decisions/ARC-177.md`, with `python3 scripts/check_decision_ids.py --write-index`.

Revision 1's items 1–3 below are **withdrawn** (struck in effect; kept for the record). Items 5, 6, 8 and 13
are amended as noted inline.

1. *(withdrawn by R2-2)* `clients/3d-spike/scripts/worlds/lakeside/depiction.gd`: the stub (S33-c §5.5) replaced by the real
   depiction; the interface unchanged.
2. *(withdrawn by R2-3: the builders are generic, not Lakeside's)* `clients/3d-spike/scripts/worlds/lakeside/` new builders: `terrain.gd` (`LakesideTerrain`), `water.gd`
   (`LakesideWater`), `shore.gd` (`LakesideShore`: lane setts, kerb, railing, trail, boulders, reeds),
   `houses.gd` (`LakesideHouses`: cottage row and yard, bakery and boathouse exteriors, the square's well and lime
   tree), `woods.gd` (`LakesideWoods`: species MultiMeshes, ground cover, air walls), `light.gd` (GI volume and
   probes for Lakeside).
3. *(withdrawn by R2-1: the grid's control data is the `terrain` section of `lakeside.yaml`)* `clients/3d-spike/scripts/worlds/lakeside/terrain.json`: the authored height grid's control data (§5.2).
4. `clients/3d-spike/shaders/water.gdshader`: the extension of §5.3 (uniforms with defaults that leave Social
   Café's water byte-equal in output, I-d-2).
5. Tree species parameters in `scripts/props.gd` / `scripts/procgen.gd` (new species functions beside the
   existing `broadleaf` and `conifer`; existing functions unchanged). *(Revision 2: each new function is a
   species **form** the woods builder picks by name from the data file's species table — `conifer_open`,
   `cedar`, `shrub`, and the oak and lime leaf cards — never a Lakeside species by name.)*
6. `viewpoints()` of the Lakeside depiction: far-field VP-L-1…7 final values, the eight perf views of step §8.1
   minus the five that need S33-e/f/g (F-S33d-7; §5.7), the `farfield_shots` list. *(Revision 2: the values
   are the `viewpoints` section of `lakeside.yaml`; `SliceDataDepiction.viewpoints()` returns them.)*
7. `tools/cli/tests/client_far_field.rs`: Lakeside's 3D far-field case (R-S33d-c-1).
8. `clients/3d-spike/scripts/slice/slice_probe_world.gd`: `--geometry` (12e's disclosed-vs-drawn probe) runs on
   Lakeside's depiction (X-8). *(Revision 2: and on `lakeside-v7` through `--depiction-file=`.)*
9. `scripts/ci_layer.py`: `clients-probes` gains `slice --world=lakeside --geometry`.
10. `clients/3d-spike/shots/lakeside/{farfield,review}/` (committed shots, the V-gate evidence).
11. `presentation/mineworld-default/3D/far_field/lakeside.yaml` and its 2D twin: **only** `datum_m` and
    `datum_source`, if Q-S33d-1 moves the lane's height (§5.2); nothing else in either file.
12. `docs/decisions/` (#258 has merged): `DEP-77`, `DEP-78`, `DEP-79`, and `ARC-170` per Q-S33d-1;
    *(revision 2)* `ARC-177` (R2-9).
13. `clients/3d-spike/ASSETS.md`: one row per new CC0 set (DEP-8), only if C6's frame review adopts one (C8 in revision 2's plan).

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
| *(revision 2)* Furniture: the pump, the boat, the workbench, the bread racks, the long table and its chairs, the well, the benches, the shore's three boulders | the world's `body: furnishings` (S33-b revision 2 §5.3.1), drawn by `SliceFurnishings` (`ARC-307`); never this PR's data file |
| *(revision 2)* Migrating Social Café's depiction to a data file | not now: Social Café is the parity reference (I-d-2); a later design may migrate it once the format has met two worlds |
| *(revision 2)* A world present with **no** depiction script (data file alone, found by the registry) | 16f, which "changes only where depictions come from" (`SliceDepiction` header, R-16f-c-1). Until then a new world's script is the one line `extends SliceDataDepiction` |
| *(revision 2)* A real second world drawn from data (for example `bodies-yard`, which has no 3D far field yet) | a follow-up PR (Q-S33d-6, option B); this PR's second world is the generated variant `lakeside-v7` |
| *(revision 2)* The 2D client reading this file | not planned: the 2D look is the 2D pack's (S33-h) |

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

**Revision 2 audit (`origin/main @ e7c095d0`, S33-c merged as PR #381).**

| # | Fact | Where |
| --- | --- | --- |
| A-15 | The seam as merged: `SliceDepiction` (`build`, `place_volumes`, `depictions`, `doors`, `spawn`, `viewpoints`, `sit_points`, `far_field_key`, `hud_note`, static `tag()` and `entry()`); the registry opens `res://scripts/depictions/<key>/depiction.gd`, else `3d-<key>.pck`, else refuses `PRESENTATION_WORLD_MISSING` ("There is no fallback depiction") | `scripts/depictions/depiction.gd`; `scripts/depictions/registry.gd` |
| A-16 | `SliceBoxDepiction`: a stub built from data (`frame_origin()`, `street()`, `places()` with tags, label, footprint Rect2 mm, street-side doorway mm); "a world's stub is only its data table". Lakeside's stub is that table (cottages, boathouse, bakery, square, shore, pier, station, ticket-office, platform) | `scripts/depictions/box_depiction.gd:1–60`; `scripts/depictions/lakeside/depiction.gd` |
| A-17 | Depictions live at `scripts/depictions/<key>/`, not `scripts/worlds/<key>/`: `client_rules.rs` check (c) refuses a literal path with a `worlds/` component (D-S33c-1; `names_a_pack_path`) | pr-s33c §15; `tests/acceptance/tests/client_rules.rs:220–227` |
| A-18 | The far field's format pattern: YAML's JSON subset (`DEP-16`); typed serde schema with `deny_unknown_fields` (`packages/src/far_field_schema.rs`, 622 lines); checks and codes (`packages/src/far_field.rs`, `far_field_check.rs`); a corpus with one file per code (`packages/tests/fixtures/far_field/`); `mineworld packs validate` checks every `far_field/` file; the client reader `clients/shared/environment/far_field.gd` refuses by the same codes, compared by `tools/cli/tests/client_far_field.rs` | as listed |
| A-19 | The variant pattern: `scripts/far_field_variant.py <file> <degrees> <out-dir>` writes a rotated far field "the cheapest second world that proves it"; the slice draws it with `--slice-far-field-file=<path>` (launcher `--far-field-file=`) | `scripts/far_field_variant.py:1–13`; `scripts/slice/slice_main.gd:157–175` |
| A-20 | The 3D pack's folder is found as `PACK_DIR := "../../presentation/mineworld-default/3D"`; furnishings bind from `furniture/furnishings.yaml` (format 1) and are drawn by `SliceFurnishings`, which "names no place, no world and no kind" | `scripts/slice/slice_main.gd:110`; `scripts/slice/furnishings.gd:1–22` |
| A-21 | S33-b revision 2 (`ARC-307`): the pump, the boat, the workbench, the bread racks, the long table and five chairs, the well, the two benches, the three shore boulders, the pier bench and the halt's benches are **furnishings**; the walls, the oven wall, fitted counters, gated stair, the lime tree's trunk and the pavilion's posts stay **shell** solids | `pr-s33b-lakeside-bodies.md` §5.3.1 |

### 3.2 Findings

| ID | Finding | Resolution |
| --- | --- | --- |
| **F-S33d-1** | **Two metres is too high for the pier.** S33-a's datum puts the lane 2 m above full pool. Step §4.3 puts the pier deck ≈ 0.6 m above full pool. The pier's foot is on the lane, so a walker would drop 1.4 m in the deck's first metres. A ramp at the accessible slope of 1:12 (2010 ADA Standards §405.2) needs 16.8 m of run, most of a 20 m pier; stairs would break 1005.1 ("accessible routes serving fishing piers … shall comply with Chapter 4"). A deck at lane level instead stands 2 m over the water, which needs a 42 in guard (IBC §1015.3) and loses the fishing rail of 1005.2.1 | **Q-S33d-1**: lower the lane to 1.0 m above full pool (S33-a's note allows it: "S33-d's grading fixes it") and put the deck 0.70 m above full pool (§5.2). The foot then needs a 0.30 m ramp of 3.6 m at 1:12, inside the pier's first 4 m |
| **F-S33d-2** | **The water does not reach the pier from the shore place alone.** The shore's floor ends at its water line, scene z 13 (A-11), but the pier's floor starts at z 3. A drawn shore 10 m deep beside a pier that is over water from its foot needs the water line to turn north at the pier | The drawn water line follows the shore place's south edge (z 13) from x −15 to the shore floor's east edge (x 0.6), turns north along a short stone revetment at x ≈ 0.8 to z 4 at the pier's foot, and runs east at z 4 past the halt's buffer stop to x 30 (A-12; §5.4). East of the pier a quay wall stands on the water line at z 4, 1 m south of the lane's edge (§5.4). The server needs nothing new: the lane's south edge is already a wall except at its two doors (S33-b) |
| **F-S33d-3** | **The boathouse is on the land side.** World data put it north of the lane (A-11); no slip can reach the water without crossing the lane | Drawn as a boatbuilder's shed: a boat on a cradle inside (S33-b's solid), wide doors to the lane. Moored boats are S33-e's, at the pier. Recorded, not changed (I-S33-1) |
| **F-S33d-4** | **The real woods burned.** The 2003 Cedar Fire burned most of the Cuyamaca forest; the step's source says it is "regrowing" (step §3.2). A dense mature conifer ring would be a pre-2003 picture | **Q-S33d-2**: the ring as today's mixed regrowth (survivor pines and cedars, young pines, resprouted black oak, chaparral understorey), dense enough to close the land sectors; or pre-fire mature woods (reference 06's mood) |
| **F-S33d-5** | **Four needed materials are missing** (bark, forest floor, granite, shingle; A-4). The step allows at most four CC0 sets "only if a reviewed frame shows the shipped sets cannot read as the material" | C6's frame review (C8 in revision 2) decides each; each adopted set gets a DEP-8 row under `DEP-79` |
| **F-S33d-6** | **12e's probe exists only for Social Café** (`--world --geometry`, A-10) | C9 runs it on Lakeside's depiction; X-8 is planted there |
| **F-S33d-7** | Five of step §8.1's eight perf views need content from later PRs (`pier head looking north`: the pier, S33-e; `bakery interior`, `boathouse shed`: S33-f; `lakeside crowd`: S33-g; `halt platform`: S33-e) | This PR ships the three views it can draw (`lane wide`, `shore across the lake`, `square`); each later PR adds its own (§5.7). AC-S33-5 is closed by S33-j |

| **F-S33d-8** *(rev. 2)* | **Revision 1's builders were world-specific.** `LakesideTerrain`, `LakesideHouses`, `LakesideWoods` would each have held Lakeside's numbers in code. The charter's rule (2026-10-18) rejects a mechanism that only makes the default town work | §5.0: one data file per world, generic builders; the variant `lakeside-v7` proves it (AC-d-11) |
| **F-S33d-9** *(rev. 2)* | **Revision 1 drew furniture.** §5.4 drew S33-b's three shore boulders and §5.5 the pump, the well and the benches at their footprints. Since S33-b revision 2 these are furnishings (A-21), drawn from the world's data by `SliceFurnishings` | Withdrawn here. The data file can draw only shell and landscape (§5.0.3); I-d-4's 50 mm agreement for furnishings is `SliceFurnishings`'s (I-FD-3). M-d-3 is redefined (§9 C6) |
| **F-S33d-10** *(rev. 2)* | **The house shells' footprints are written twice:** in the world's `places/*.yaml` (the server's walls) and in the depiction file (the drawn walls). Deriving them from the disclosed `place-shape` at run time would remove the copy, but disclosure is only the observer's own place (`ARC-307`, Q-FD-3), so the neighbours' shells would be missing from the lane | Kept as two copies, as S33-c's stub does; the geometry probe (12e `--geometry`, AC-d-3) names any drift by place and face. Q-S33d-5 |

### 3.3 What does not exist

No Lakeside terrain, near water, building, tree or light; no Lakeside 3D far-field case; no Lakeside geometry probe.
*(Revision 2)* No `depictions/` folder in any Presentation Pack; no depiction schema, validator or corpus; no
data-driven builder for terrain, water, shells or woods (the spike's `town.gd` draws its lake and trees from
constants, A-1).

---

## 4. Reuse, both ways (`REUSE_POLICY.md`; the step's §4.9 table is the decision; C1 re-verifies its facts)

| Need | Options | Decision | C1 re-verifies |
| --- | --- | --- | --- |
| Terrain (`DEP-77`) | Terrain3D (TokisanGames, MIT, C++ GDExtension) · HTerrain (Zylann, GDScript, maintenance mode) · Godot built-ins (`ArrayMesh`, `HeightMapShape3D`) | **Built-ins** | Terrain3D's platform list and Godot 4.7 support; HTerrain's licence file and Godot 4.7 status. If either fact contradicts the step's reason, **stop** (a reversed reuse decision is the operator's) |
| Water (`DEP-78`) | ours (`water.gdshader`) · community Godot 4 lake and ocean shaders | **Ours, extended** | each named candidate's licence and Godot 4.7 status, recorded in `DEP-78` |
| Vegetation (`DEP-79`) | the spike's cards · Poly Haven tree models · CC0 low-poly packs (a second style) · paid packs (Q-S33-7(c): nothing bought) | **The spike's cards**; ≤ 2 Poly Haven hero trees only if C6's review (C8 in revision 2) needs them | the Poly Haven model licence (CC0) and each model's triangle count |
| Rocks | the spike's procedural `boulder` · Poly Haven rock models | **The procedural boulder**, tuned to granite (rounded, exfoliated); one Poly Haven rock set only under `DEP-79`'s four | — |
| Sett lane, kerb, railing | shipped `cobblestone_floor_08`, `stone_brick_wall_001`, `Props.railing` | **Reuse** | — |

No dependency is added to any lockfile. No GDExtension is added.

**Revision 2 rows** (the data file and the generic builders). Each "ours" below is recorded in `ARC-177` with
its reason, as `REUSE_POLICY.md` asks of a rejection; C1 re-verifies every licence and Godot 4.7 fact.

| Need | Options | Decision | Reason / C1 re-verifies |
| --- | --- | --- | --- |
| File syntax and parsing | YAML's JSON subset read by Godot's `JSON` and by `serde_json` (`DEP-16`, the far field's and furnishings' choice) · full YAML · Godot `.tres` resources | **JSON subset, reused** | one syntax for every pack data file; both readers already in the build. `.tres` cannot be read by the Rust validator |
| Validation | typed serde structs with `deny_unknown_fields` (the far field's pattern) · JSON Schema (`jsonschema` crate) | **Typed serde, reused** | the far field's style, named types (`CLAUDE.md` §4 item 7), no new crate. JSON Schema would add a lockfile entry and a second style |
| Polygon checks in Rust (closed, simple, ≤ 64 vertices, overlap with a footprint) | `geo` crate (MIT/Apache-2.0) · our own segment test | **Ours, ≈ 60 lines** | polygons are ≤ 64 vertices and the checks are two (simple; overlaps a rectangle); a new crate tree in the lockfile for two predicates is not worth it. C1 records the rejection; if C2's code passes 150 lines, adopt `geo` instead (a bounded discovery, recorded) |
| Polygon work in the client (inside test, triangulation, offset, union) | Godot `Geometry2D` (`is_point_in_polygon`, `triangulate_polygon`, `offset_polygon`, `merge_polygons`) · ours | **`Geometry2D`, built in** | — |
| Height profiles (distance → height along a bearing) | Godot `Curve` (baked, linear) · ours | **`Curve`, built in** | — |
| Scattering trees, rocks, ground cover | ProtonScatter (HungryProton, MIT, Godot 4 addon) · Spatial Gardener (MIT, Godot addon) · our Poisson-disc (Bridson's algorithm, ≈ 50 lines) | **Ours, Poisson-disc** | both addons are authored in the editor and store their result in a scene; ours must be driven by a data file the Rust validator also reads, seeded and identical on three OSes. C1 verifies each addon's licence, Godot 4.7 status and whether it can run from data at run time; if one can, **stop** (a reversed reuse decision is the operator's) |
| Building shells from massing | Godot CSG nodes (built in) · func_godot (MIT, GDScript; reads TrenchBroom `.map` brushes) · Cyclops Level Builder (MIT, editor only) · ours with `SurfaceTool` and the spike's `Build` helpers (`box`, `prism`, `slab`, A-2) | **Ours with `SurfaceTool` / `Build`** | Godot's documentation positions CSG for prototyping, with a run-time cost per operation; `.map` brushes are hand-modelled geometry, not massing data a validator can range-check; Cyclops works only in the editor. C1 re-verifies each fact |
| Terrain mesh and collision | unchanged: built-ins (`ArrayMesh`, `HeightMapShape3D`), `DEP-77` | **Built-ins** | revision 1's row; Terrain3D stays out (a GDExtension) |

---

## 5. Design

### 5.0 The depiction data file (revision 2; `ARC-177`)

#### 5.0.1 The idea, in three sentences

A world's 3D look is written as data in its Presentation Pack: where the ground rises, where the water lies,
what each house looks like, which trees grow where. Generic builders in the client read that data and build the
scene; they know how to raise a gable roof or plant a stand of pines, but not that Lakeside has a bakery. So a
second town by a lake is a second data file, and the builders do not change.

Example. Revision 1 would have written the bakery's roof in `houses.gd` as code, roughly
`roof(Vector3(-3, 5.8, -11), 6.0, 8.0, 40.0, "clay_roof_tiles_02")`. Revision 2 writes it in `lakeside.yaml` as
`"roof": {"form": "gable", "ridge": "x", "pitch_deg": 40, "material": "clay_roof_tiles_02"}`, and
`shell_builder.gd` draws whatever roof the file names.

#### 5.0.2 Where it lives and who reads it

- **Path:** `presentation/<style>/3D/depictions/<world-key>.yaml`; Lakeside's is
  `presentation/mineworld-default/3D/depictions/lakeside.yaml`. The folder is **optional** in a 3D pack (unlike
  `far_field/`, which is required): a world drawn by a bespoke script (Social Café) has no file.
- **Readers:** the 3D client (`SliceDepictionFile`, through `SliceDataDepiction`) and `mineworld packs validate`
  (`mineworld_packages::depiction`). **Nothing on the server reads it**: no System, no kernel, no world rule
  (I-d-11). It is presentation, like the far field: heights, roofs and trees never reach the simulation
  (step §4.6, F-S33-4).
- **Finding it:** `SliceDataDepiction` reads `<PACK_DIR>/depictions/<key>.yaml`, where `key` is the depiction's
  world key (A-15, A-20), unless `--slice-depiction-file=<path>` names another file (R2-7), exactly as
  `--slice-far-field-file=` does (A-19). The file's `scope.world` must equal its stem in both cases.
- **Specification for pack authors:** `docs/MODULE_SPEC.md` §6.1.2 (new) and `clients/3d-spike/PRESENTATION.md`
  §6 (new), written by C1 from §5.0.3 below.

#### 5.0.3 The format (format 1)

YAML's JSON subset (`DEP-16`). Every key below is required unless marked *optional*; any other key is refused
(`DEPICTION_MALFORMED`). Units are in the key's name: `_mm` integer millimetres, `_m` metres, `_deg` degrees.

**Plan coordinates** are the street frame's, in millimetres: x east, y north, `(0, 0)` at the street's south-west
corner (`CORE_CONCEPTS.md` §6.1; the same frame as S33-c's stub, A-16). The scene position of a plan point
`(x, y)` at height `h_m` is `frame.origin_m + (x / 1000, h_m, −y / 1000)`. Lakeside's origin is `[-15, 0, 3]`,
so the lane's centre `(15 000, 3 000)` is the scene's origin. **Heights** (`_m`) are metres above the street's
surface (`y = 0`, §5.1). Polygons are lists of `[x_mm, y_mm]`, 3 to 64 vertices, simple, closed implicitly.

**Citations.** Every block that carries a physical value has a `source` string: a citation, or
`"approximate: <who replaces it>"` (I-d-7, I-S33-7). A missing or empty `source` is `DEPICTION_UNCITED`.

| Section | What it holds | Fields |
| --- | --- | --- |
| `format` | the format | `1` |
| `scope` | the world | `{"world": "<world-key>"}`; equals the file's stem |
| `frame` | where plan (0, 0) stands | `{"origin_m": [x, y, z]}` |
| `street` | the street's binding (S33-c's stub table, moved) | `{"tags": [...], "footprint_mm": [x, y, w, h]}` |
| `places` | every other drawn place's binding | `[{"tags": [...], "label": "...", "footprint_mm": [x, y, w, h], "doorway_mm": [x, y]}]`: the entries, volumes and doors `SliceBoxDepiction` registers today (A-16); tags only, never a place key |
| `spawn` | the offline spawn | `{"at_mm": [x, y], "up_m": h, "yaw_deg": d}` |
| `terrain` | the ground | below |
| `water` | the near water | below |
| `edges` | kerbs, quays, revetments, rails, low walls and gates | below |
| `paths` *(optional)* | trails painted into the ground | `[{"name", "line_mm": [[x, y], ...], "width_m", "layer"}]`; `layer` names a terrain layer |
| `buildings` | shells | below |
| `trees` | species, stands and single trees | below |
| `scatter` *(optional)* | boulders, reeds, grass and wildflower clumps | below |
| `light` | GI volumes and reflection probes | `{"gi": [{"box_mm": [x, y, w, h], "height_m", "subdiv": 64 \| 128 \| 256}], "probes": [{"box_mm", "height_m", "centre_up_m", "update": "once"}]}` |
| `viewpoints` | the checks' and captures' views (A-15's `viewpoints()`) | `{"farfield": [{"id", "at_mm", "eye_m", "roles": [...]}], "farfield_shots": [{"name", "at_mm", "eye_m", "yaw_deg", "pitch_deg"}], "perf": [same], "perf_breakdown": [names], "probe_route": [[tags], ...]}` |

**`terrain`.**

```json
"terrain": {
  "extent_mm": [-85000, -97000, 200000, 200000], "cell_m": 1.0, "blend_m": 2.0, "seed": 33,
  "layers": [{"name": "setts", "material": "cobblestone_floor_08", "repeat_m": 1.0}, ...],
  "default_layer": "forest-floor",
  "regions": [
    {"name": "lane", "polygon_mm": [[0, 0], [30000, 0], [30000, 6000], [0, 6000]],
     "height": {"flat_m": 0.0, "crown_m": 0.04, "crown_along": "x"}, "layer": "setts",
     "source": "approximate: a sett lane's camber (§5.2)"},
    {"name": "shore-slope", "polygon_mm": [[0, -10000], [15600, -10000], [15600, 0], [0, 0]],
     "height": {"ramp": {"from_mm": [[0, 0], [15600, 0]], "from_m": 0.0,
                         "to_mm": [[0, -10000], [15600, -10000]], "to_m": -1.0}},
     "layer": "grass", "source": "§5.2, Q-S33d-1 (A): the shore falls 1.0 m to full pool over 10 m"}
  ],
  "profiles": [
    {"name": "north-rim", "bearing_deg": 0,
     "samples": [{"distance_m": 60, "height_m": 0.0}, {"distance_m": 80, "height_m": 0.0}],
     "source": "S33-a SOURCE.md, the 3DEP north-sector line (C4 copies the line number)"}
  ],
  "bed": {"depth_m": 3.0, "run_m": 30.0, "source": "approximate (§5.2): −1.0 at the water line to −4.0 at 30 m"}
}
```

- A `height` is exactly one of `{"flat_m"}` (with *optional* `crown_m`, `crown_along: "x" | "y"`), or
  `{"ramp": {"from_mm": line, "from_m", "to_mm": line, "to_m"}}` (height linear in the distance between the two
  lines, which must be parallel within 1°).
- Outside every region the ground follows the `profiles`: at a point, the two profiles whose bearings bracket it
  are each sampled at the point's distance from the frame origin (linear between samples, Godot `Curve`) and
  blended by angle. Regions blend into it over `blend_m`. Inside `water.polygon_mm` the ground is the lake bed.
- `layers` ≤ 4 (the splat limit, step §4.1); a region's or path's `layer` must name one.
- Revision 1's values (§5.2) are the values of Lakeside's regions; the yard, apron and square regions rise
  `+0.00 … +0.60` north, as §5.2 says. Nothing new is chosen.

**`water`.**

```json
"water": {
  "level_m": -1.0, "far_field_water": "lake", "extent_m": 160,
  "polygon_mm": [[-85000, -10000], [15800, -10000], [15800, -1000], [45000, -1000], ...],
  "shader": {"depth_fade_m": 1.5, "shore_foam": 0.25},
  "lilies": [{"polygon_mm": [...], "cover": 0.3, "seed": 5}],
  "source": "§5.3; F-S33d-2's water line (revetment at x ≈ 15 800, quay at y −1 000)"
}
```

- `polygon_mm` is the near water's outline; its land-side edges are the drawn water line (F-S33d-2), its outer
  edges lie at `extent_m` under the far-field annulus (A-7). C5 writes the outer vertices from §5.3's 160 m
  square.
- `far_field_water` names an entry of the same world's `far_field/<key>.yaml` water list: the colours are that
  entry's, never repeated here (`DEPICTION_REFERENCE` when the name is absent).
- `shader` sets §5.3's uniforms; an absent uniform keeps the shader's default (which leaves Social Café's water
  as it is, I-d-2).

**`edges`.** `[{"name", "kind", "line_mm": [[x, y], ...], "height_m", "width_m", "material", "open_at": [tags],
"source"}]`, with `kind` one of `kerb`, `quay` (*also* `drop_m`), `revetment` (*also* `drop_m`), `rail` (*also*
`posts_m`, the post spacing), `wall` (a low garden wall), `gate`. `open_at` lists tags of `places`: the builder
leaves a gap at each such place's doorway (`SliceBoxDepiction`'s `DOOR_W`), so a gap is never a coordinate typed
twice. Lakeside's are §5.4's: the south kerb (150 mm), the quay (z 4 → y −1 000, x 20 000…45 000, drop 1.0 m),
the revetment (x ≈ 15 800, y −10 000…−1 000), the 1.07 m railing (x 20 000…30 000 on y 0, IBC §1015.3), the
0.9 m rail west of the pier (x 0…15 600, open at `shore`), the yard's low wall and gate (§5.5).

**`buildings`.** One entry per shell:

```json
{"name": "bakery", "depicts": ["bakery"],
 "footprint_mm": [[12000, 6000], [18000, 6000], [18000, 14000], [12000, 14000]],
 "storeys": 2, "storey_m": 2.9, "plinth_m": 0.3,
 "walls": {"material": "sandstone_blocks_05"},
 "roof": {"form": "gable", "ridge": "x", "pitch_deg": 40, "overhang_m": 0.35, "material": "clay_roof_tiles_02"},
 "openings": [
   {"side": "s", "kind": "shop_window", "storey": 0, "offset_m": 0.6, "width_m": 2.4, "height_m": 1.6, "sill_m": 0.6, "state": "dark"},
   {"side": "s", "kind": "door", "storey": 0, "at_doorway": true, "width_m": 1.0, "height_m": 2.1, "state": "closed"}
 ],
 "chimney": {"at_mm": [12800, 13500], "size_m": [0.6, 0.6], "above_ridge_m": 0.8},
 "source": "§5.5; VISUAL_SLICE §3 (door 2.1 m, sill 0.9 m, storey 2.8–3.0 m)"}
```

- `depicts`: the tags of the place whose walls this shell draws, or `null` for a shell nobody enters (the four
  cottages, §5.5). A shell that depicts a place must have the footprint of that place in `places` within 50 mm;
  the probe compares it with the server's walls (AC-d-3, F-S33d-10).
- `roof.form`: `gable`, `hip`, `mono` (*also* `falls: "n" | "s" | "e" | "w"`) or `flat`.
- `openings[].kind`: `door`, `double_door`, `window`, `shop_window`; `state`: `closed`, `open` or `dark` (a dark
  interior card). `at_doorway: true` puts a door on the depicted place's `doorway_mm` (no coordinate twice). A
  door may be `open` only when its building depicts a place and the door is `at_doorway` (`DEPICTION_DOOR`):
  nothing pretends (I-d-5). Until S33-f every Lakeside door is `closed` and every window `dark` (§5.5).
- `walls.material`, `roof.material`: slugs of the client's texture library (A-4); *optional* `tint` `"#rrggbb"`.

**`trees`.**

```json
"trees": {
  "species": [
    {"name": "jeffrey-pine", "latin": "Pinus jeffreyi", "form": "conifer", "card": "needle_long",
     "height_m": {"min": 24, "max": 32}, "source": "Gymnosperm Database, Pinus jeffreyi: trees 24–61 m (survivors)"},
    {"name": "jeffrey-pine-young", "latin": "Pinus jeffreyi", "form": "conifer", "card": "needle_long",
     "height_m": {"min": 4, "max": 10}, "source": "§5.6: regrowth after the 2003 Cedar Fire (Q-S33d-2 A)"},
    {"name": "black-oak", "latin": "Quercus kelloggii", "form": "broadleaf", "card": "oak_leaf",
     "height_m": {"min": 6, "max": 14}, "source": "USDA FEIS, Quercus kelloggii: 9–25 m at maturity; resprouts"}
  ],
  "stands": [
    {"name": "ring", "area": {"ring": {"centre_mm": [15000, 3000], "inner_m": 30, "outer_m": 80}},
     "spacing_m": 3.5, "density": {"from": 0.0, "to": 1.0, "rise_m": [30, 60]},
     "mix": [{"species": "jeffrey-pine", "weight": 0.10, "prefer": "rise"},
             {"species": "black-oak", "weight": 0.25, "prefer": "low"}],
     "seed": 33}
  ],
  "singles": [{"species": "lime", "at_mm": [27000, 15000], "height_m": 12}],
  "keep_clear": {"places": true, "paths_m": 2.0},
  "lod_m": 40,
  "air_walls": [{"ring": {"centre_mm": [15000, 3000], "radius_m": 60}}, {"line_mm": [[0, -10000], [15600, -10000]]}],
  "source": "§5.6"
}
```

- `form` names a builder form of `Props` (`conifer`, `conifer_open`, `cedar`, `broadleaf`, `shrub`); `card` a
  `Procgen` card (`needle`, `needle_long`, `spray`, `oak_leaf`, `lime_leaf`, `leaf`, `shrub`). Both are checked by
  the client (`DEPICTION_REFERENCE`), since only the client knows its library.
- `area` is `{"ring": ...}` or `{"polygon_mm": ...}`. Placement is Poisson-disc with `spacing_m` and the stand's
  `seed`, thinned by `density` (rising linearly over `rise_m` from the ring's centre); `prefer` weights a species
  toward low ground near water or rises (§5.6). `keep_clear` keeps every trunk off every `places` footprint, off
  the streets and `paths_m` from every path.
- `singles` are planted trees (the square's lime, step §4.2). The lime's trunk is a shell solid of the world
  (A-21); the probe checks the drawn trunk within 50 mm of it (I-d-4).
- One `MultiMesh` per species and LOD band (`lod_m`), as §5.6.

**`scatter`.** `[{"name", "kind": "boulder" | "reeds" | "grass" | "wildflower", "area": {"polygon_mm"} |
{"along_mm": line, "width_m"}, "count", "size_m": {"min", "max"}, "seed", "range_m" (optional: visibility),
"source"}]`. `kind: "boulder"` is a granite boulder of `Props.boulder` (§5.4's up-to-twelve beyond the shore
floor); reeds ≤ 40 clumps (§5.4). **A scatter area may not overlap any `places` footprint by more than 50 mm**
(`DEPICTION_ON_FLOOR`), and the same holds for every stand and every `depicts: null` building. A thing drawn where
people walk must be a disclosed solid, and disclosed solids are the world's (furnishings or shell), never this
file's (I-d-10, `ARC-307`).

**What the file cannot say.** There is no field for a table, chair, bench, pump, well, boat, shelf or any other
free-standing piece, and `deny_unknown_fields` refuses one (`DEPICTION_MALFORMED`). Furniture is the world's
`body: furnishings`, drawn by `SliceFurnishings` from the pack's `furniture/furnishings.yaml` (`ARC-307`). There
is no field for a rule, a capacity, a door's passability or a height the server sees.

#### 5.0.4 The validator (Q-S33d-4)

`mineworld_packages::depiction` reads a file into the typed schema (`depiction_schema.rs`) and checks it in this
order; the first fault is the refusal, named with the field's path (for example
`DEPICTION_ON_FLOOR: scatter[1] "revetment-rocks" overlaps the place tagged shore by 1 240 mm`):

| Order | Code | Fault |
| --- | --- | --- |
| 1 | `DEPICTION_MALFORMED` | not the JSON subset; a missing required key; an unknown key; a wrong type or enum value |
| 2 | `DEPICTION_SCOPE_MISMATCH` | `scope.world` is not the file's stem |
| 3 | `DEPICTION_UNCITED` | a physical block without a non-empty `source` |
| 4 | `DEPICTION_OUT_OF_RANGE` | a value outside its range: heights −20…60 m; `cell_m` 0.5…4; extent ≤ 400 m; `storeys` 1…4; `storey_m` 2.4…4.0 (VISUAL_SLICE §3); `pitch_deg` 0…60; tree heights 0.3…100 m; `spacing_m` ≥ 1.0; `density` 0…1; counts and caps (≤ 4 layers, ≤ 16 species, ≤ 4 000 trees per stand, ≤ 64 buildings, ≤ 40 reeds per scatter, ≤ 4 GI volumes) |
| 5 | `DEPICTION_POLYGON` | a polygon with < 3 or > 64 vertices, or not simple; a ramp's lines not parallel within 1° |
| 6 | `DEPICTION_REFERENCE` | a name that names nothing in the file (a layer, a species, a tag in `open_at` or `depicts` that no `places` entry has) or in the same pack (`far_field_water` not in `far_field/<key>.yaml`) |
| 7 | `DEPICTION_DOOR` | an `open` door not `at_doorway` of a depicted place; `at_doorway` on a building that depicts nothing |
| 8 | `DEPICTION_ON_FLOOR` | a stand, scatter area or `depicts: null` building overlapping a `places` footprint by more than 50 mm; a building that depicts a place with a footprint more than 50 mm off that place's |

- `mineworld packs validate <pack>` checks every `depictions/*.yaml` of a 3D pack when the folder exists
  (`tools/cli/src/packs.rs`), after the far field.
- **The corpus:** `packages/tests/fixtures/depiction/`, one file per code and `valid-*.yaml` files (the far
  field's convention, A-18), plus the committed variant `variants/lakeside-v7/lakeside.yaml`.
  `packages/tests/depiction.rs` asserts each file's code by name.
- **The client reader** (`SliceDepictionFile`) parses with Godot's `JSON` and refuses with the same names for
  what it checks: `DEPICTION_MALFORMED` (types, required keys, unknown keys), `DEPICTION_SCOPE_MISMATCH`, and
  `DEPICTION_REFERENCE` for the names only the client knows (texture slugs, `form`, `card`). Geometry codes
  (4–8) are the Rust validator's alone; the client trusts a validated pack and draws what it is given.
  `tools/cli/tests/client_depiction.rs` runs `scripts/checks/depiction_check.gd` over the corpus and asserts that
  for every file both readers agree: the same code where the client checks that code, and the client accepts
  every file the Rust validator accepts.
- A refused file refuses the run by name, in RL-e0's line format (`[mineworld-slice] DEPICTION_…: { world: … }:
  …`), exit 3 in a scripted run. There is no fallback drawing (ARC-146).

#### 5.0.5 The generic builders (`clients/3d-spike/scripts/depictions/data/`)

| File | Class | Reads | Builds | Reuses |
| --- | --- | --- | --- | --- |
| `depiction_file.gd` | `SliceDepictionFile` | the file | the parsed, checked data; or a refusal by code | Godot `JSON` |
| `data_depiction.gd` | `SliceDataDepiction extends SliceBoxDepiction` | `SliceDepictionFile` | calls the builders below in order; registers volumes, entries and doors from `street`/`places` with the box depiction's registration (R2-4); returns `spawn`, `viewpoints` from the file | S33-c's seam unchanged |
| `terrain_builder.gd` | `DepictionTerrain` | `terrain`, `paths`, `water.polygon_mm` | the rasterised height grid (deterministic; a hash printed by `--layout`), one `ArrayMesh` with the ≤ 4-layer splat, one `HeightMapShape3D` on `LAYER_WORLD` | `Geometry2D`, `Curve`, `Mats.pbr` |
| `water_builder.gd` | `DepictionWater` | `water`, the far field's water entry | the triangulated surface at `level_m` out to `extent_m`, `water.gdshader` with the file's uniforms, the lily decal mask (`Procgen`), no probe (light's) | `Geometry2D.triangulate_polygon` |
| `edge_builder.gd` | `DepictionEdges` | `edges` | kerbs, quays and revetments (`Build.box` / `prism` along each segment), rails (`Props.railing`), walls and gates; gaps at `open_at` doorways | `Build`, `Props.railing` |
| `shell_builder.gd` | `DepictionShells` | `buildings` | walls with openings (plinth, reveals, eaves: VISUAL_SLICE §6.2), the roof forms, chimneys, dark interior cards and closed door leaves | `SurfaceTool`, `Build`, `Mats.pbr` |
| `woods_builder.gd` | `DepictionWoods` | `trees` | Poisson-disc placement, one `MultiMesh` per species and band, singles, air walls (`StaticBody3D` on `LAYER_WORLD`) | `Props` species forms, `Procgen` cards |
| `scatter_builder.gd` | `DepictionScatter` | `scatter` | boulders (`Props.boulder`, granite), reeds and ground-cover `MultiMesh`es | `Props`, `Procgen` |
| `light_builder.gd` | `DepictionLight` | `light` | `VoxelGI` volumes and `ReflectionProbe`s (the café pattern, A-6) | Godot built-ins |

**The builders' rules.** Every number in a builder is one of: a unit conversion, a cap the validator also
enforces, a cited physical default shared by every world (the capsule, A-14; VISUAL_SLICE §3's door and storey
ranges as defaults), or a schema default named in `PRESENTATION.md` §6. Anything else that varies between worlds
is a field of the file. No builder names a world, a place key, a tag or a species (I-d-9; check (d) still scans
them). The same seed gives the same scene on every OS: no unseeded noise, no `randf()` without a
`RandomNumberGenerator` seeded from the file.

#### 5.0.6 The generated variant (`scripts/depiction_variant.py`; Q-S33d-6)

`python3 scripts/depiction_variant.py <file> --seed <n> <out-dir>` writes `<out-dir>/<stem>.yaml`. It keeps
what the server's world fixes — `scope`, `frame`, `street`, `places`, and every building that `depicts` a place
(footprint and openings `at_doorway`) — and changes everything the presentation owns, deterministically from
`n`:

- the terrain profiles' heights scaled by a factor in 0.6…1.6, and the regions' rise north scaled the same way;
- the water polygon's outer vertices moved within 10 m (the land-side water line stays, because the shore's
  floor ends there);
- every roof form cycled (`gable` → `hip` → `mono` → `gable`), pitches re-drawn in 25…50°, wall and roof
  materials swapped among the library slugs Lakeside already uses, storeys ±1 within 1…3;
- the `depicts: null` cottages moved, resized within ±1 m and their count changed by ±1, re-drawn until clear of
  every footprint (`DEPICTION_ON_FLOOR` holds by construction);
- every stand's seed, spacing (±30 %), density rise and species weights re-drawn, every tree height range scaled
  by 0.7…1.3, and every scatter seed and count re-drawn.

`lakeside-v7` is its output for seed 7, committed under `packages/tests/fixtures/depiction/variants/` (C12
re-runs the script and compares with `cmp`). It is drawn by
`./mineworld-slice --world=lakeside --depiction-file=packages/tests/fixtures/depiction/variants/lakeside-v7/lakeside.yaml`.
A builder that kept a Lakeside number in code would draw `lakeside-v7` wrong — a pine still 24 m tall, a roof
still a gable — and the variant's checks (AC-d-1…AC-d-4, AC-d-11) would show it.

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

*(Revision 2: every height in this section is a value of `lakeside.yaml`'s `terrain` section, §5.0.3; the
"height grid" paragraph below describes that section, and `LakesideTerrain` is the generic `DepictionTerrain`.)*

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
  *(Revision 2: the three shore boulders are furnishings, kind `boulder`, drawn by `SliceFurnishings` from the
  world's data (A-21, F-S33d-9); this PR binds nothing for them. The up-to-twelve beyond the floor are a
  `scatter` entry of kind `boulder`, off every footprint.)*
- *(Revision 2)* Every line, height and gap in this section is a value of `lakeside.yaml`'s `edges`, `paths`
  and `scatter` sections (§5.0.3).
- **Reeds:** card clumps along the water line west of the pier, ≤ 40 clumps, one `MultiMesh`.

### 5.5 The village exterior

Every footprint is S33-b's (A-11); every solid S33-b names outdoors (the pump, the well, the lime trunk, the
square's benches) is drawn at its footprint within 50 mm (I-d-4). Façade fronts stand on the lane's north edge
(z −3).

*(Revision 2: of those solids, only the lime tree's trunk is this PR's to draw (a `trees.singles` entry; the
trunk is a shell solid, A-21). The pump, the well and the benches are furnishings drawn by `SliceFurnishings`
(F-S33d-9). Every row of the table below is a `buildings` entry of `lakeside.yaml` (§5.0.3): the cottages with
`depicts: null`, the boathouse and the bakery with their places' tags; the yard's wall and gate are `edges`.)*

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

*(Revision 2: the species table below is `lakeside.yaml`'s `trees.species`, each row with its `source`; the
placement, LOD, air walls and ground cover are `trees.stands`, `lod_m`, `air_walls` and `scatter`. The method
column names a builder **form**; no builder names a species.)*

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

*(Revision 2: the GI box, the probes, the far-field viewpoints, the far-field shots and the perf views are
`lakeside.yaml`'s `light` and `viewpoints` sections, in plan millimetres with `eye_m` above the street surface;
for example VP-L-4 is `{"id": "VP-L-4", "at_mm": [8000, -9600], "eye_m": 0.66, "roles": ["judged"]}`.)*

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
- *(Revision 2)* `ARC-177`: a 3D Presentation Pack may carry `depictions/<world-key>.yaml`
  (format 1, §5.0), read by the 3D client's generic builders and checked by `mineworld packs validate`; the
  server never reads it; it holds no furniture (`ARC-307`) and no rule. It records the reuse rejections of §4's
  revision 2 rows (`geo`, ProtonScatter, Spatial Gardener, CSG, func_godot, Cyclops) with their reasons, and
  the operator's ruling of 2026-10-20 with the charter rule it applies.

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-d-1 (I-S33-1) | No path under `kernel/`, `contracts/`, `systems/`, `server/`, `worlds/`, `clients/protocol/` changes | C13 diff scan (C10 in revision 1) |
| I-d-2 (I-S33-2) | Social Café is unchanged: its shots within `frame_diff`'s 8/255 of C0's capture (the shader's new uniforms default off) | AC-d-6 |
| I-d-3 (I-S33-4) | Every Lakeside viewpoint closes on a far field | AC-d-1 |
| I-d-4 (I-S33-9) | Every drawn wall and solid of a Lakeside place lies within 50 mm of a disclosed face; no drawn wall without a disclosed one | AC-d-3 (12e's `--geometry` on Lakeside) |
| I-d-5 (I-S33-5) | Nothing pretends: closed doors stay closed; no moving prop; no lit window without an interior behind it | review; S33-e's honesty scan |
| I-d-6 (I-S33-6) | One method: our builders, the shipped library, ≤ 4 new CC0 sets each with a DEP-8 row; no generated building | review; `ASSETS.md` |
| I-d-7 (I-S33-7) | Every physical value cites a source or is marked approximate with its replacer | review of §5's tables and `lakeside.yaml`'s `source` fields at C13; `DEPICTION_UNCITED` |
| I-d-8 (I-S33-3) | No place-key literal in any Lakeside builder (S33-c's check (d)) | `client_rules.rs` |
| I-d-9 *(rev. 2)* | **No world-specific constant in a builder.** Every value that differs between worlds is a field of the depiction file; the builders under `scripts/depictions/data/` name no world, place key, tag or species, and their numbers are only §5.0.5's kinds. Lakeside's script holds no number | check (d) over `scripts/depictions/data/`; review at C13; AC-d-11 (`lakeside-v7` drawn with no GDScript change) |
| I-d-10 *(rev. 2)* | **No furniture from the depiction file** (`ARC-307`): the schema has no field for a free-standing piece, and nothing the file draws stands on a place's floor except a shell that depicts that place | `DEPICTION_MALFORMED` and `DEPICTION_ON_FLOOR` (corpus files); AC-d-12 |
| I-d-11 *(rev. 2)* | **The server never reads the depiction file**: no crate under `kernel/`, `contracts/`, `systems/`, `server/` names `depictions/`; only `mineworld_packages` (validation) and the 3D client read it | C13 grep, recorded |
| I-d-12 *(rev. 2)* | **One validator's verdict.** The client and `mineworld packs validate` refuse by the same code names for the codes the client checks, and the client accepts every file the validator accepts | `client_depiction.rs` (AC-d-10) |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static scan | I-d-8 | `cargo test -p mineworld-acceptance --test client_rules` (CI `core`) |
| Rendered (Godot) | AC-d-1: Lakeside far field at VP-L-1…7 × 8 azimuths | `tools/cli/tests/client_far_field.rs`, `#[ignore]`, nightly `clients` on ubuntu-24.04, macos-26, windows-2025; RL-e0's per-PR Linux `farfield` job |
| Probe (Godot, real server) | AC-d-3 (`--world=lakeside --geometry`); AC-d-4 (`--world=lakeside --link` on the real depiction) | nightly `clients-probes`; local one at a time |
| Parity | AC-d-6 (Social Café shots) | local `frame_diff.gd` against C0's capture; nightly |
| Data | `terrain.json` loads and rasterises identically twice (a hash printed by `--layout`) | the probe |
| *(rev. 2)* Data | `lakeside.yaml`'s terrain rasterises identically twice, and `lakeside-v7`'s hash differs from it (the hashes printed by `--layout`) | the probe |
| *(rev. 2)* Format (Rust) | AC-d-10: every corpus file's code; `packs validate` on the 3D pack accepts `lakeside.yaml` | `cargo test -p mineworld-packages --test depiction` (CI `core`); `tools/cli/tests/packs.rs` |
| *(rev. 2)* Format (both readers) | AC-d-10: the client and Rust agree on the corpus (I-d-12) | `tools/cli/tests/client_depiction.rs`, `#[ignore]`, nightly `clients` |
| *(rev. 2)* Variant | `depiction_variant.py --seed 7` reproduces the committed `lakeside-v7` byte for byte | C12, `cmp`; `scripts/tests/` if a Python test layer exists at C0, else recorded |
| Gate 1 / 2 | NOT REQUIRED | — |

---

## 8. Acceptance criteria (fixed before measuring)

Revision 2 restates every criterion so that it names a second world or a generated variant (the charter's
rule). The second world is **`lakeside-v7`**, the generated variant of §5.0.6, drawn with
`--depiction-file=packages/tests/fixtures/depiction/variants/lakeside-v7/lakeside.yaml` against the same Lakeside
server. Where a criterion is about something the variant cannot change (Social Café's parity), the other world
named is Social Café, with Market Town, which draws Social Café's street (D-S33c-2).

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-d-1 | `client_far_field` Lakeside case: FF-BELOW ≥ 90 % and FF-ABOVE ≥ 25 % at VP-L-1…7 × 8 azimuths, S/H/E frames, on Linux, macOS and Windows; *(rev. 2)* **and the same thresholds for `lakeside-v7`** at the same viewpoints (its terrain and woods differ; its far field is Lakeside's) | AC-S33-1 |
| AC-d-2 | X-1: deleting the 3D water body fails by name (`FAR_FIELD_GAP` at validate, or `FF-BELOW` in S, SW at VP-L-3); *(rev. 2)* the same plant fails the same way when `lakeside-v7` is drawn | X-1 |
| AC-d-3 | `--world=lakeside --geometry` passes; X-8 (a drawn bakery wall with no disclosed solid) fails naming the place and the face; *(rev. 2)* `--geometry` **passes on `lakeside-v7`** (its shells that depict places keep their footprints), and X-8 planted as a `buildings` entry of `lakeside-v7` (a `depicts: ["bakery"]` wall 0.5 m off) fails naming the place and the face | X-8 |
| AC-d-4 | `--world=lakeside --link` on the real depiction: the probe route of S33-c passes, 0 refusals, the player stands on the terrain (ray hit within 0.05 m of the terrain's height at 20 sampled points); *(rev. 2)* **the same on `lakeside-v7`**, whose ground heights differ | §12.1 checkpoint |
| AC-d-5 | Counts for the three perf views recorded (draws, primitives, video memory) and each under step §8.1's advisory figures; timings INCONCLUSIVE locally; *(rev. 2)* `lakeside-v7`'s counts at the same views recorded beside them (not gated) | §8.1 |
| AC-d-6 | Social Café's `--shots` and `--farfield-shots` within 8/255 of C0's capture; *(rev. 2)* and `--world=market-town --link` prints the same `SHOWN` line as at C0 (the second world drawn by Social Café's depiction) | I-S33-2 |
| AC-d-7 | Lakeside's 3D bytes added by this PR ≤ 8 MB (half of the 16 MB gate, leaving S33-e/f the rest), measured as the summed size of *(rev. 2)* `scripts/depictions/lakeside/`, `scripts/depictions/data/`, `depictions/lakeside.yaml`, new shaders and any new CC0 sets; `lakeside.yaml` itself ≤ 64 KB, and `lakeside-v7`'s within 10 % of it | §8.2 |
| AC-d-8 | The operator's first `VIS-3D-LAKESIDE` preview: the committed `review/` shots (§9 C13) sent through the lane lead; *(rev. 2)* with the same five views of `lakeside-v7` beside them | ARC-24 |
| AC-d-9 | The diff touches only §2.1's paths (revision 2's list) | I-d-1 |
| AC-d-10 *(rev. 2)* | **The format is validated, both ways.** `mineworld packs validate presentation/mineworld-default/3D` accepts `lakeside.yaml`; `packages/tests/depiction.rs` refuses each corpus file by its code (all eight codes of §5.0.4) and accepts `valid-*.yaml` and `lakeside-v7`; `client_depiction.rs` shows the client agreeing on every corpus file (I-d-12) | ARC-177 |
| AC-d-11 *(rev. 2)* | **Built from data alone.** `lakeside-v7` is validated and drawn from the same commit with **no GDScript change**; its `--layout` terrain hash differs from Lakeside's; its frames show its own roofs, materials, cottage count and tree heights (the C12 frame review names three differences). `scripts/depictions/lakeside/depiction.gd` is ≤ 10 lines and contains no number. A mutation **M-d-5** that puts one Lakeside value back into a builder as a constant (the pines' 24 m minimum) is caught by `lakeside-v7`'s frame review and its recorded tree-height histogram | charter, 2026-10-18 |
| AC-d-12 *(rev. 2)* | **No furniture from the file** (`ARC-307`). A corpus file with a `furnishings` key is `DEPICTION_MALFORMED`; one with a scatter area on the shore's floor is `DEPICTION_ON_FLOOR`; `--world=lakeside --link`'s `FURNISHED` report draws every furnishing disclosed in the places on the probe route (S33-b §5.3.1's, for example the bakery's long table and chairs and the pier's bench) from world data with no `unbound` row caused by this PR, the same in `lakeside-v7` | I-d-10 |

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

**Revision 2 replaces revision 1's C0…C10 with C0…C13 below** (revision 1's plan stays readable in this file's Git
history). The work per area is the same; what moves is where each value is written (the
data file) and what each builder knows (nothing about Lakeside). Paths are under `clients/3d-spike/` unless
stated. Every builder commit also adds its section to `lakeside.yaml`, its corpus files, and runs on
`lakeside-v7` once C12 has written it (before C12, on the data alone).

Under the lane charter's SPEED FIRST rule the per-commit review and mutation items may be recorded as `N/A —
SPEED FIRST (charter 2026-10-18)` where the charter waives them; the mutations named here are then run at the
milestone. The deterministic validation items are not waived.

#### C0 — Base, prerequisites, ledger, captures (Markdown only)

- **Files:** this file (§15), `handoff-s33d.md` (new).
- Steps: record the base; confirm S33-a, S33-b (revision 2, furnishings), S33-c, 12e merged (any missing →
  material stop); re-read A-1…A-21 and record moved lines; capture Social Café's `--shots` and
  `--farfield-shots` and Market Town's `--link` `SHOWN` line into the scratchpad (sizes and base SHA in §15); read
  the Silvics chapters for §5.6's species and record each range.
- [ ] Implementation. [ ] Validation: doc scripts. [ ] Review: every prerequisite with evidence.
- **Commit message:** `docs(plan): S33-d — open the ledger at the base`

#### C1 — Records and spec text; the reuse facts re-verified; the grading

- **Files:** `docs/decisions/` (`DEP-77`, `DEP-78`, `DEP-79`, `ARC-170`, `ARC-177`; `INDEX.md` by
  `check_decision_ids.py --write-index`), `docs/MODULE_SPEC.md` §6.1 and §6.1.2 (new), `clients/3d-spike/PRESENTATION.md`
  §§1, 6 (new), the two `far_field/lakeside.yaml` files (`datum_m`, `datum_source` only, per Q-S33d-1), this file.
- Steps: §4's re-verifications (revision 1's rows and revision 2's), each fact with its URL and date; the records;
  §5.0.3 written into the two specs.
- Validation: doc scripts; `check_decision_ids.py`; `mineworld packs validate` on both packs; `cargo test -p
  mineworld-packages --test far_field`.
- [ ] Implementation. [ ] Validation. [ ] Review: no reuse fact reverses a decision (else stop); the spec text
  and §5.0.3 say the same.
- **Commit message:** `docs: ARC-177, DEP-77…79, ARC-170 — Lakeside's depiction as pack data (S33-d)`

#### C2 — The format and its validator (Haiku)

- **Files:** `packages/src/{depiction_schema.rs, depiction.rs, lib.rs, error.rs}`, `tools/cli/src/packs.rs`,
  `packages/tests/depiction.rs`, `packages/tests/fixtures/depiction/` (one file per code, `valid-*.yaml`), this
  file.
- Steps: §5.0.3's types (named types, `deny_unknown_fields`, no maps of untyped values); §5.0.4's checks in
  order; `packs validate` calls `depiction::check_pack` when `depictions/` exists.
- Validation: `cargo test -p mineworld-packages --test depiction`; `cargo test -p mineworld-cli --test packs`;
  `cargo clippy -p mineworld-packages -- -D warnings`. Mutation **M-d-6** (the `DEPICTION_ON_FLOOR` check's
  tolerance raised to 5 000 mm) → the `on-floor.yaml` corpus case fails by name.
- [ ] Implementation. [ ] Validation. [ ] Review: each code reachable by exactly one corpus file; no
  world name in the validator.
- **Commit message:** `feat(packages): the depiction file's schema and validator (S33-d)`

#### C3 — The client reader and the data depiction; Lakeside's script shrinks (Opus)

- **Files:** `scripts/depictions/data/{depiction_file.gd, data_depiction.gd}`, `scripts/depictions/box_depiction.gd`
  (R2-4's split), `scripts/depictions/lakeside/depiction.gd` (→ `extends SliceDataDepiction`),
  `scripts/slice/slice_main.gd` and `mineworld-slice` (`--slice-depiction-file=` / `--depiction-file=`),
  `presentation/mineworld-default/3D/depictions/lakeside.yaml` (first version: `frame`, `street`, `places`,
  `spawn`, `viewpoints` — the stub's table and viewpoints moved by value; minimal `terrain` and `water`
  sections drawing flat ground at y 0 and no water, so the file validates), `scripts/checks/depiction_check.gd`,
  `tools/cli/tests/client_depiction.rs`, this file.
- Steps: §5.0.2, §5.0.4's client part, §5.0.5's first two rows. Until C4…C9 land, `SliceDataDepiction` draws
  the stub's grey boxes from `places` (the box depiction's drawing), so the world stays walkable.
- Validation: S33-c's AC-c-3/AC-c-4 still pass (`--world=lakeside --link`: all link checks pass, the same places
  bound); `cargo test -p mineworld-cli --test client_depiction -- --ignored` (one run); check (d).
- [ ] Implementation. [ ] Validation. [ ] Review: the `SliceDepiction` interface unchanged; the script holds no
  number.
- **Commit message:** `feat(3d): depictions from pack data; Lakeside's depiction reads its file (S33-d)`

#### C4 — Terrain builder (Haiku)

- **Files:** `scripts/depictions/data/terrain_builder.gd`, `lakeside.yaml` (`terrain`, `paths`), corpus files
  for terrain faults, this file.
- Steps: §5.2's values as regions and profiles (the 3DEP samples copied from S33-a's `SOURCE.md` with their
  line); rasterising, the splat (≤ 4 layers: setts, grass, earth, forest floor — forest floor `leafy_grass`
  darkened until C8 decides), the `HeightMapShape3D`, the grid hash on `--layout`.
- Validation: the hash printed twice equal; AC-d-4's ray samples. Mutation **M-d-1** (the shore region's
  `to_m` set to 0.0 in the data) → AC-d-4's ray check fails at the shore samples by name.
- [ ] Implementation. [ ] Validation. [ ] Review: every height traces to §5.2's table or a `SOURCE.md` line.
- **Commit message:** `feat(3d): terrain from regions and height profiles; Lakeside's grading as data (S33-d)`

#### C5 — Water builder (Haiku)

- **Files:** `scripts/depictions/data/water_builder.gd`, `shaders/water.gdshader`, `lakeside.yaml` (`water`),
  this file.
- Steps: §5.3, the polygon of §5.0.3's `water`.
- Validation: AC-d-6 (Social Café's water unchanged); AC-d-1 run once (recorded, not gated). Mutation **M-d-2**
  (the shader's `depth_fade_m` default set to 1.5) → AC-d-6 fails on Social Café's water frames.
- [ ] Implementation. [ ] Validation. [ ] Review: no seam between the near surface and the far-field water in the
  S frame at VP-L-4 (recorded shot).
- **Commit message:** `feat(3d): water from a polygon — depth fade, shore band, lily pads (S33-d)`

#### C6 — Edges and scatter (Haiku)

- **Files:** `scripts/depictions/data/{edge_builder.gd, scatter_builder.gd}`, `lakeside.yaml` (`edges`,
  `scatter`), corpus files, this file.
- Steps: §5.4 and F-S33d-2 (kerb, revetment, quay, railings, the yard's wall and gate, the trail's paint), the
  boulders beyond the floor and the reeds. No shore boulder on the floor (F-S33d-9).
- Validation: AC-d-4. Mutation **M-d-3** *(redefined)*: one `scatter` boulder area moved onto the shore's floor →
  `packs validate` refuses `DEPICTION_ON_FLOOR` naming the entry and the tag `shore`.
- [ ] Implementation. [ ] Validation. [ ] Review: railing heights cite §5.4's clauses; every gap comes from
  `open_at`.
- **Commit message:** `feat(3d): edges and scatter from lines and areas; Lakeside's shore as data (S33-d)`

#### C7 — Shell builder (Opus)

- **Files:** `scripts/depictions/data/shell_builder.gd`, `lakeside.yaml` (`buildings`), corpus files, this file.
- Steps: §5.5 as `buildings` entries; the roof forms, openings, plinth, eaves, chimneys.
- Validation: AC-d-3 on the shells; frame review VR-1 (the lane from VP-L-2 looking north) and VR-2 (the yard).
  Mutation **X-8** (a `depicts: ["bakery"]` wall entry 0.5 m off) → `--geometry` red by name.
- [ ] Implementation. [ ] Validation. [ ] Review (Opus, frames): reads as references 01/06's idiom; scale against
  VISUAL_SLICE §3 (door 2.1 m, sill 0.9 m, storey 2.8–3.0 m); every roof form drawn once in a test frame (the
  variant uses all four).
- **Commit message:** `feat(3d): building shells from massing; Lakeside's village as data (S33-d)`

#### C8 — Woods builder (Opus)

- **Files:** `scripts/depictions/data/woods_builder.gd`, `scripts/props.gd`, `scripts/procgen.gd` (new forms
  and cards only), `lakeside.yaml` (`trees`), `ASSETS.md` and `assets/textures/<set>/` (only if a set is
  adopted), this file.
- Steps: §5.6; the per-material frame review (bark, forest floor, granite, shingle): adopt a CC0 set only where the
  frame shows the shipped material cannot read as it, ≤ 3 MB each, DEP-8 row.
- Validation: AC-d-1 in full; counts; Social Café's tree functions unchanged (AC-d-6). Mutation **M-d-4** (the
  `ring` stand removed from the data) → AC-d-1 FF-BELOW fails in N, NE, NW at VP-L-5 by name.
- [ ] Implementation. [ ] Validation. [ ] Review (Opus, frames): species read as named; no suggested path beyond
  the boundary.
- **Commit message:** `feat(3d): woods from areas and a species table; Lakeside's regrowth as data (S33-d)`

#### C9 — Light builder (Opus)

- **Files:** `scripts/depictions/data/light_builder.gd`, `lakeside.yaml` (`light`), this file.
- Steps: §5.7's GI and probes.
- Validation: counts; video memory recorded. Frame review VR-3 (the shore across the lake at the fixed sun).
- [ ] Implementation. [ ] Validation. [ ] Review: no light leak under the eaves; the water's probe reflects the
  village, not the café.
- **Commit message:** `feat(3d): GI volumes and probes from data (S33-d)`

#### C10 — Far-field case, viewpoints, perf views (Haiku)

- **Files:** `lakeside.yaml` (`viewpoints`, final values), `tools/cli/tests/client_far_field.rs`,
  `scripts/ci_layer.py`, `shots/lakeside/farfield/`, this file.
- Steps: §5.7's viewpoints and perf views; the Lakeside case in `client_far_field.rs`; SSR measured once (recorded).
- Validation: AC-d-1 locally once; AC-d-2 (X-1 planted in the 3D far-field file, reverted by `cmp`); AC-d-5.
- [ ] Implementation. [ ] Validation. [ ] Review: verdict lines unchanged in form.
- **Commit message:** `test(3d): Lakeside's far-field check at seven viewpoints and its perf views (S33-d)`

#### C11 — The geometry probe on Lakeside (Haiku)

- **Files:** `scripts/slice/slice_probe_world.gd`, `scripts/ci_layer.py`, this file.
- Steps: 12e's `--geometry` reads the current depiction's volumes; `clients-probes` gains the Lakeside run.
- Validation: AC-d-3; X-8 re-run red then green.
- [ ] Implementation. [ ] Validation. [ ] Review: the probe compares every drawn collider in a place's volume, not
  only walls.
- **Commit message:** `test(3d): drawn and disclosed walls agree on Lakeside (S33-d)`

#### C12 — The generated variant (Haiku)

- **Files:** `scripts/depiction_variant.py`, `packages/tests/fixtures/depiction/variants/lakeside-v7/lakeside.yaml`,
  `tools/cli/tests/client_far_field.rs` (the variant's case), `scripts/ci_layer.py` (`clients-probes` gains the
  variant's `--geometry` and `--link`), `shots/lakeside-v7/review/`, this file.
- Steps: §5.0.6; the variant validated, drawn, probed and shot from this commit with no GDScript change.
- Validation: AC-d-1…AC-d-4 and AC-d-11 on `lakeside-v7`; the script re-run → `cmp` equal. Mutation **M-d-5**
  (the pines' 24 m minimum written into `woods_builder.gd`) → the variant's recorded tree-height histogram and
  frame show it; reverted.
- [ ] Implementation. [ ] Validation. [ ] Review (frames): three named differences between the two worlds.
- **Commit message:** `test(3d): a generated Lakeside variant drawn from data alone (S33-d)`

#### C13 — Close-out and the operator's preview

- **Files:** this file (§15), `handoff-s33d.md`, `shots/lakeside/review/` (VR-1…VR-5: the lane, the yard, the
  shore across the lake, the square, the high view) beside `shots/lakeside-v7/review/`.
- Steps: AC-d-7, AC-d-9, AC-d-12; I-d-9 and I-d-11's scans recorded; push; `gh pr create` with the shots; CI on
  the exact head; repair to green; the preview to the lane lead for the operator (AC-d-8).
- [ ] Implementation. [ ] Validation: CI green on the exact head. [ ] Review: deviations recorded.
- **Commit message:** `docs(plan): S33-d close-out`
- **Then:** MERGE-READY (charter: the coordinator merges); READY FOR OPERATOR REVIEW of the V-gate shots.

---

## 10. Requirements this PR places on other PRs

- **R-S33e-d-1 (on S33-e).** The pier deck at y −0.30, its foot ramp 1:12 from the lane (Q-S33d-1 as ruled); the
  buffer stop just inside the quay line (z ≤ 4.0; S33-e puts it at z 3.5); VP-L-3 and VP-L-6 re-judged on S33-e's decks.
- **R-S33f-d-1 (on S33-f).** Interiors fit inside §5.5's shells; S33-f opens the bakery and boat-shed doors.
- **R-S33a-d-1 (on S33-a's file, through this PR).** `datum_m` 1 407 if Q-S33d-1 is ruled A (this PR edits it,
  §2.1 item 11).
- **R-S33e-d-2 (on S33-e; revision 2).** The pier's deck, rails and pavilion shell, the halt's building,
  platform, track and buffer stop are drawn from `lakeside.yaml` by generic builders, never as constants in a
  Lakeside script. Where format 1 cannot say something (a deck on posts over water, a track), S33-e adds an
  **optional** section or field to format 1 through its own design (schema, validator code, corpus file), and
  every existing file stays valid. Furniture on the pier and platform stays furnishings (`ARC-307`).
- **R-S33f-d-2 (on S33-f; revision 2).** Opening a door is changing its `state` to `open` in `lakeside.yaml`
  (allowed only `at_doorway` of a depicted place, `DEPICTION_DOOR`), in the same PR that draws the room;
  interiors' shell parts follow the same rule as R-S33e-d-2.

## 11. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-33-lakeside-3d.md` §4.1 | the water line at y −1.0 (Q-S33d-1); F-S33d-2's revetment and quay |
| `step-33-lakeside-3d.md` §4.3 | the deck 0.70 m above full pool (Q-S33d-1) |
| `step-33-lakeside-3d.md` §4.2 | the woods as today's regrowth (Q-S33d-2 as ruled) |
| `pr-s33a-setting-far-field.md` §5.4 | `datum_m` 1 407 (a dated note) |
| *(rev. 2)* `step-33-lakeside-3d.md` §4.1, §4.2, §12.1 | "our builder from a small authored height grid" → "generic builders from the world's depiction file (`ARC-177`)"; the S33-d row's scope gains the format, validator and variant |
| *(rev. 2)* `pr-s33e-pier-halt.md`, `pr-s33f-interiors-sit-points.md` | R-S33e-d-2, R-S33f-d-2: their Lakeside drawing goes through the data file; paths `scripts/depictions/lakeside/` (D-S33c-1) |
| *(rev. 2)* `pr-s33j-release-perf.md` | `3d-lakeside.pck` carries `depictions/lakeside.yaml` with the scripts |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S33-d — Lakeside's terrain, water and village exterior in 3D
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s33d-terrain-water-exterior.md
RELATED / BINDING DOCS:  step-33-lakeside-3d.md; pr-s33a-setting-far-field.md; pr-s33b-lakeside-bodies.md
                         (revision 2); pr-s33c-world-seam.md (§15 as merged); ../mvp1/pr-rl-e0-far-field-mandatory.md;
                         ../mvp1/pr-vf-furnishings-from-data.md; docs/decisions/ARC-307.md; docs/MODULE_SPEC.md
                         §6.1; docs/VISUAL_SLICE.md; docs/ART_DIRECTION.md; docs/REUSE_POLICY.md; CLAUDE.md;
                         the lane-lead charter (2026-10-18: no world-specific hard-coding; SPEED FIRST)
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s33d, its own, held by one session
BRANCH:                  mvp0/pr-s33d, created from origin/main
START CONDITION:         S33-a, S33-b (revision 2) and 12e merged (S33-c merged: PR #381); §14 ruled,
                         Q-S33d-3…6 included; revision 2 frozen by the operator; the lane lead's dispatch
IMPLEMENTATION BASE:     origin/main at C0
APPROVED SCOPE:          §2.1 revision 2's list (R2-1…R2-9 and items 4–13 as amended)
FROZEN INVARIANTS:       step-33 I-S33-1…I-S33-9 as they apply; §6 I-d-1…I-d-12; S33-c's interface and
                         registry unchanged; RL-e0 format 1 unchanged; ARC-307 (no furniture in the file)
SEQUENCE:                C0 … C13 (§9, revision 2)
ALLOWED COMMANDS:        cargo build -p mineworld-cli; cargo test -p mineworld-acceptance --test client_rules;
                         cargo test -p mineworld-packages --test far_field; cargo test -p mineworld-packages
                         --test depiction; cargo test -p mineworld-cli --test packs; cargo test -p mineworld-cli
                         --test client_far_field -- --ignored <one test>; cargo test -p mineworld-cli --test
                         client_depiction -- --ignored; cargo clippy -p mineworld-packages -- -D warnings;
                         ./mineworld-slice probes through cast-slots/with-slot.sh, one at a time; godot
                         --headless for frame_diff and depiction_check; mineworld packs validate; python3
                         scripts/* (depiction_variant.py included); git; gh (never merge)
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace; edits
                         under kernel/, contracts/, systems/, server/, worlds/, clients/protocol/; a GDExtension;
                         a paid or generated asset; a second art method; a world-specific constant, place key,
                         tag or species name in a builder; a furniture field in the depiction file; a new
                         crate in a lockfile without the bounded-discovery record of §4 (the `geo` row)
MATERIAL STOPS:          a prerequisite missing; a reuse fact reversing the step's decision; Social Café parity
                         unattainable; a drawn wall the server's floors cannot match without a world change; a
                         perf count that forecasts p95 > 16.7 ms with the levers applied; bytes over AC-d-7;
                         (rev. 2) a frozen Lakeside value that format 1 cannot express without a world-specific
                         branch in a builder; a change needed to the `SliceDepiction` interface or the registry;
                         the variant unbuildable without a GDScript change
PLATFORMS:               rendered checks on ubuntu-24.04, macos-26, windows-2025 (nightly); per-PR Linux farfield
VALIDATION BUDGET:       targeted local runs as listed; CI: the PR's runs. Gate 1/2: NOT REQUIRED
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s33d.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                         semantic commits, branch push, PR creation, CI repair: authorized
                         merge: by the coordinator (charter SPEED FIRST: sub-agents never merge), after the
                         operator's V-gate look at the preview shots (AC-d-8)
POST-MERGE SYNC OWNER:   primary: step-33, S33-a design note, S33-e/f/j notes (§11); implementation session:
                         this ledger
STOP CONDITION:          S33-d MERGE-READY with the preview sent
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-d-1 | Card trees read cheap beside reference 06 (R-S33-3) | the operator sees the woods in the C13 preview; levers: ≤ 2 Poly Haven hero trees, one bark set |
| R-d-2 | Water with the depth texture costs fill rate at low views | SSR off; the probe; the fade costs one depth read |
| R-d-3 | Q-S33d-1 ruled B (deck at lane level) | §5.2's option B is fully specified; S33-e follows it |
| R-d-4 | 12e's `--geometry` cannot see presentation colliders (terrain, rails) and flags them | the probe compares only colliders inside a place volume on the wall layer; terrain and air walls are excluded by group, recorded |
| R-d-5 | The far-field near rim from 3DEP sits higher than the drawn woods at 80 m, leaving a band | the woods' floor follows the same samples (§5.2); FF-BELOW catches a gap |
| R-d-6 *(rev. 2)* | **Premature abstraction** (`CLAUDE.md` §4 item 11): a format designed from one world grows fields nobody needs | format 1 holds only what Lakeside draws; every field is used by `lakeside.yaml`; the variant exercises every enum value (all four roof forms); new needs (S33-e's deck, track) are added as optional fields when a PR needs them (R-S33e-d-2) |
| R-d-7 *(rev. 2)* | **The format is too narrow** for a frozen Lakeside value, and a builder grows a special case | a material stop (§12); the fix is a field, reviewed, never a branch on a name |
| R-d-8 *(rev. 2)* | **Footprints drift** between the world's places and the depiction file (F-S33d-10) | AC-d-3's probe on both worlds; `DEPICTION_ON_FLOOR` checks the file against its own `places`; Q-S33d-5 |
| R-d-9 *(rev. 2)* | **The two readers disagree** on a file | I-d-12's corpus comparison; the client checks only the codes it can, and trusts a validated pack for geometry |
| R-d-10 *(rev. 2)* | **Building from data costs start-up time** (rasterising 201 × 201, Poisson-disc for thousands of trees, triangulating shells) | C4 and C8 record the build time on `--layout`; budget ≤ 500 ms for the whole depiction on the M5, INCONCLUSIVE locally; the lever is caching the grid by the file's hash under `user://` (recorded, not built, unless needed) |
| R-d-11 *(rev. 2)* | **The export file** (`3d-lakeside.pck`, S33-j) carries the scripts but not the pack's data file | §11's note to S33-j; a missing file is refused by name (`DEPICTION_MALFORMED: cannot read <path>`), never drawn as nothing |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S33d-1 [operator — the lane's height above the lake]** | S33-a set the lane 2 m above the lake, approximate. With a 2 m drop, a fishing pier needs either a 17 m ramp or a deck at lane level with a tall guard and no fishing rail (F-S33d-1). (A) Lower the lane to 1.0 m above full pool and the deck to 0.70 m, with a 3.6 m ramp at 1:12; (B) keep 2 m, deck at lane level, 42 in guard; (C) keep 2 m, deck 0.6 m, 17 m ramp? | **(A).** It keeps the pier accessible (ADA §405.2, §1005.1) and fishable over a 34 in rail (§1005.2.1), and a village a metre above a reservoir's full pool is realistic. Only `datum_m` changes in S33-a's files (1 408 → 1 407). *Chinese:* 湖边小路离湖面从 2 米降到 1 米，码头面离水 0.7 米，这样坡道短、栏杆能钓鱼——同意吗？ |
| **Q-S33d-2 [operator — what the woods look like]** | The real Cuyamaca forest largely burned in 2003 and is regrowing (F-S33d-4). Draw (A) today's regrowth — scattered survivor pines and cedars, young pines, black oaks, chaparral — still dense enough to close the view; or (B) mature pre-fire woods like reference 06? | **(A).** The realistic-defaults rule asks for the real place today, and the mix of tall survivors over young growth reads as a real mountain lake. The boundary still closes on trees at 60–80 m. *Chinese:* 树林按 2003 年大火后真实的再生林来画（老树稀疏、幼树和橡树多），还是按参考图画成成熟老林？建议前者。 |

**Revision 2's questions** (the operator's ruling of 2026-10-20 settles *whether* the village is data; these ask
only how):

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S33d-3 [operator — a new pack file type]** | The data file is a new kind of Presentation Pack file, so it is a public format. (A) Make it a pack file: `presentation/<style>/3D/depictions/<world-key>.yaml`, optional per pack, format 1, specified in `MODULE_SPEC.md` §6.1.2 and `PRESENTATION.md` §6, checked by `mineworld packs validate`, recorded as `ARC-177`; or (B) keep it client-private under `clients/3d-spike/` with no pack spec and no CLI check? | **(A).** A world's look belongs to the Presentation Pack (`ARC-307` point 4), and a pack file that `packs validate` checks is what makes a second world installable without touching the client. *Chinese:* 把湖边村的数据文件定为 Presentation Pack 的正式文件类型（`depictions/<世界>.yaml`，写进规范、`packs validate` 检查），而不是客户端私有文件——同意吗？ |
| **Q-S33d-4 [operator — how deep the client's check goes]** | The far field is validated twice in full (Rust and GDScript). For this file: (A) Rust validates in full; the client checks syntax, types, scope and the names only it knows (textures, tree forms) and refuses by the same codes; a corpus test proves they agree; or (B) a full mirror in GDScript too, as the far field has? | **(A).** The geometry checks (polygons, overlaps with floors) are the costly half, and a pack is validated before it is installed; one full validator is half the code and the corpus still proves the two readers agree. *Chinese:* 校验器 Rust 端做全量，客户端只查格式、类型和它自己才知道的名字，两边用同样的错误码、用样例集对齐，不在 GDScript 里再写一份全量——同意吗？ |
| **Q-S33d-5 [operator — house footprints written twice]** | The houses' footprints are in the world's place files (the server's walls) and again in the depiction file (the drawn walls). (A) Keep both copies and let the geometry probe name any drift (S33-c's stub already does this); or (B) derive the drawn walls from the server's disclosed place shapes at run time? | **(A).** The server discloses only the observer's own place (`ARC-307`, Q-FD-3), so under (B) the lane would show only the house you stand in; (A) draws every house and the probe catches every mismatch by place and face. *Chinese:* 房子的平面尺寸在世界数据和外观数据里各写一份，由几何探针检查两者一致（而不是运行时从服务器披露的形状推出来，因为服务器只披露你所在的地方）——同意吗？ |
| **Q-S33d-6 [operator — which second world]** | The charter asks every acceptance criterion to name a second world. (A) The generated variant `lakeside-v7` (`scripts/depiction_variant.py`, seed 7: different terrain, roofs, materials, cottages and woods on the same Lakeside server), in every AC now; a real second world later; or (B) also draw `bodies-yard` from data in this PR (it needs a new 3D far field and a one-line script, since a world without a script waits for 16f)? | **(A).** The variant follows RL-e0b's proven pattern, costs one script, and catches a builder that kept a Lakeside number; (B) adds a far field and a world outside S33's scope. Do (B) as a follow-up once 16f lets a world exist with data alone. *Chinese:* 第二个世界先用自动生成的湖边变体 `lakeside-v7`（地形、屋顶、材质、树林都不同）写进每条验收；真正的另一个世界（如 bodies-yard）等 16f 之后再做——同意吗？ |

### 14.1 Rulings

- 2026-10-19 (the freeze, header): Q-S33d-1 **(A)** and Q-S33d-2 **(A)**, as recommended.
- 2026-10-20 (revision 2's freeze, relayed by the coordinator; #386 merged): Q-S33d-3 **(A)**, Q-S33d-4
  **(A)**, Q-S33d-5 **(A)**, Q-S33d-6 **(A)**, as recommended.

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 base, ledger | (this commit) | [x] | [x] doc scripts | N/A — SPEED FIRST (charter 2026-10-18); prerequisites with evidence below |
| C1 records, spec text | (this commit) | [x] | [x] doc scripts; `packs validate` and `far_field` test in CI | N/A — SPEED FIRST; no reuse fact reverses a decision (evidence below) |
| C2 format, validator | (this commit) | [x] | CI: `depiction` test, `packs`, clippy (machine reserved); `rustfmt` local | N/A — SPEED FIRST; M-d-6 at the milestone |
| C3 client reader, data depiction | — | [ ] | [ ] | [ ] |
| C4 … C13 | — | [ ] | [ ] | [ ] |

### Evidence

**C0 (2026-10-21).**

- Worktree `/Users/yuema137/mineworld-worktrees/impl-s33d`, branch `mvp0/pr-s33d`. Base: `origin/main @ 5f9d8b0b`
  (2026-10-20 22:36 −0700) reached through #395's head `b6d60223` ("Merge origin/main into mvp0/pr-s33b"), merged
  into this branch as `9c1d6928`. The branch carries #387 (S33-a, head `9d590231`), #395 (S33-b, head
  `b6d60223`) and #400 (the stub follow-up, head `57e913fd`), all OPEN under the main merge freeze: **stacked**,
  not merged (the header's start condition, by the coordinator's priority order).
- S33-c: MERGED, #381, `e7c095d0`. 12e: MERGED, #254. Revision 2: #386 MERGED (`3508bfba`), frozen by the
  operator (relayed by the coordinator), §14 as recommended.
- Anchors re-read for C3: A-15 (`scripts/depictions/depiction.gd`, `registry.gd`: interface and lookup as
  stated), A-16 (`box_depiction.gd`; Lakeside's stub now uses S33-b's doorway points, #400). A-1…A-14 and
  A-17…A-21 are re-read by the commits that use them (C4…C9).
- **MACHINE RESERVED (coordinator, 2026-10-21):** no local cargo, Godot or Blender while the operator records.
  Social Café's `--shots`/`--farfield-shots` and Market Town's `--link` `SHOWN` capture (AC-d-6's baseline) are
  **INCONCLUSIVE — to be captured from `5f9d8b0b` when the machine is free**; every Rust and GDScript check
  runs in GitHub CI on the PR.
- Silvics ranges for §5.6: recorded by C1 with the reuse facts.

**C1 (2026-10-21).** Ids confirmed free on `origin/main @ 5f9d8b0b` (no `docs/decisions/` file and no mention
under `docs/` of ARC-170, ARC-177, DEP-77, DEP-78, DEP-79). Written: `docs/decisions/{ARC-170, ARC-177, DEP-77,
DEP-78, DEP-79}.md`, `INDEX.md` by `check_decision_ids.py --write-index` ("221 decision ids, all distinct;
index current"); `MODULE_SPEC.md` §6.1 (the directory row) and §6.1.2 (new); `PRESENTATION.md` §1 and §6
(new); the 3D `far_field/lakeside.yaml` `datum_m` 1 408 → 1 407 and its `datum_source` (ARC-170).

Reuse facts, read at source 2026-10-21 (WebFetch/WebSearch; URLs in the records):

| Fact | Result | Reverses the step? |
| --- | --- | --- |
| Terrain3D | MIT; C++ GDExtension; Windows, Linux, macOS; 1.0.2 lists Godot 4.4–4.6, 4.7 not listed | no (DEP-77 built-ins) |
| HTerrain | MIT; bug fixes only, "not in active development"; README targets Godot 4.6+ | no |
| Community water shaders | "Stylized Water for Godot 4.x" MIT (code); "Water shader 3D (Godot 4.3)" licence INCONCLUSIVE | no (DEP-78 ours) |
| Poly Haven models | CC0; tree polycounts 0.3 M … 17.4 M (API) | no (DEP-79 cards) |
| ProtonScatter | MIT, Godot 4; run-time scattering from data: **INCONCLUSIVE** (README silent) | not shown to; the stop rule ("if one can run from data at run time, stop") is not triggered by an unread fact — re-check at C8 |
| Spatial Gardener | MIT, Godot ≥ 4.2, an editor plugin; run time INCONCLUSIVE | as above |
| Godot CSG | "mainly intended for prototyping"; performance warning | no |
| func_godot | MIT; reads Quake/Valve `.map` (TrenchBroom) | no |
| Cyclops | MIT; "block in levels inside the Godot editor" | no |
| `geo` | MIT OR Apache-2.0, 0.33.1 | no (rejected; ours) |
| Silvics (AH 654) | **INCONCLUSIVE**: srs.fs.usda.gov TLS refused; mirror PDFs not text-readable. Secondary: Jeffrey pine 80–140 ft, black oak 40–80 ft (Virginia Tech fact sheets) | Gymnosperm DB / FEIS ranges stand |

Validation: doc scripts PASS (`check_decision_ids.py`, `check_doc_headings.py`). `mineworld packs validate` on
both packs and `cargo test -p mineworld-packages --test far_field`: **in CI** (machine reserved).

### Deviations and findings (implementation)

- **D-S33d-1 (C2, bounded; format detail).** §5.0.3 lists `terrain`, `water`, `edges`, `buildings`, `trees` and
  `light` as required. The implementation makes them (and `paths`, `scatter`) **optional**, absent meaning "not
  drawn": C3's first Lakeside file carries none of them (§9 C3 asks for "minimal terrain and water sections
  drawing flat ground at y 0 and no water" — absence says that without inventing values), and 16f-d1's café
  files carry only `models`, `light`, `viewpoints`, `spawn` and the bindings (16f §4.5). Within those sections,
  lists default to empty (`profiles`, `stands`, `singles`, `air_walls`, `lilies`, `openings`, `open_at`,
  `farfield_shots`, `perf`, `perf_breakdown`, `probe_route`) and `roof.overhang_m` to 0. No value of the frozen
  design changes; the format only accepts less. Recorded in `MODULE_SPEC.md` §6.1.2 and `PRESENTATION.md` §6.
- **D-S33d-2 (C1).** §2.1 item 11 names "the two `far_field/lakeside.yaml` files". The 2D file has no
  `datum_m` (a 3D-only section, MODULE_SPEC §6.1.1), so only the 3D file changes.
- **D-S33d-3 (C2).** The corpus has client-only faults (a texture slug, a tree form) that the Rust validator
  cannot see: stems containing `-client-` are valid for Rust and refused `DEPICTION_REFERENCE` by the client.
  I-d-12's "the client accepts every file the Rust validator accepts" holds for every other file.
- **D-S33d-4 (C2).** The checks live in `packages/src/depiction_check.rs` beside `depiction.rs` and
  `depiction_schema.rs` (the far field's three-file split), so no file passes ~800 lines.

### Mutations

(none yet; M-d-1 … M-d-6, X-1, X-8 as §9 lists them)

### Deviations and findings

F-S33d-1 … F-S33d-7 (§3.2), recorded at design time. F-S33d-8 … F-S33d-10, recorded by revision 2
(2026-10-20).

### Design review

2026-10-18: one independent Opus reviewer read the seven S33-d…j designs against the step, the frozen S33-a/b/c
designs, step-34, S29-a rev. 3 and the source. Verdict REQUEST CHANGES, no blockers. Applied here before the PR
opened: the cottages kept west and north of the yard, never on the boathouse (§5.5); the quay wall and the water
line together at z 4 from x 5 to x 30, the verge between them not walkable (§5.4, F-S33d-2); VP-L-1 at
(−10.5, eye, −4.5); the pump at x −12; "five" perf views wait for later PRs (F-S33d-7); the buffer stop inside
the quay line (R-S33e-d-1).

2026-10-20 (revision 2): drafted against `origin/main @ e7c095d0` after S33-c merged (PR #381); audited
A-15…A-21 from source. No independent reviewer under the charter's SPEED FIRST rule; the operator's freeze of
the revision is the review point (§14 Q-S33d-3…Q-S33d-6).
