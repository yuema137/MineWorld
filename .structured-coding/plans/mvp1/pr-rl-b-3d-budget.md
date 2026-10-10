# PR RL-b — the 3D slice within the Default tier, and the San Diego skyline preview

**`DESIGN FROZEN 2026-10-09 (primary session; rulings in §0.1)`**

**Lifecycle:** FROZEN. Scope (§1), the measurement rules and thresholds (§3), the decisions (§4, §5), the
acceptance and mutation criteria (§6), the visual guards (§7) and the execution contract (§13) are frozen.
Progress, evidence, audit findings and bounded corrections stay writable (§11 checkboxes, §14 ledger). A fresh
session implements it in `impl-rl-b`. Superseded lifecycle: `PR design — ready for freeze review`.
**Step:** S22 Realism, [`step-22-realism.md`](step-22-realism.md) §9.2 row RL-b, ruled 2026-10-09 (§13.1 there).
**Runs as:** an MVP-0 visual-track PR, because the operator asked for it now (coordinator, 2026-10-09). It
changes only presentation (`clients/3d-spike/**`, the launchers, docs).
**Author:** the S22 planning session, 2026-10-09, worktree `/Users/yuema137/mineworld-worktrees/plan-s22`,
branch `plan/s22-realism`. Audit base: `origin/main @ f80bbb7` (code), with `origin/main @ a454e37` checked for
the merged 12e and TW-b/TW-d plans.
**Placeholder ids:** SD-RLb-n (decisions), M-n (measurement rules), A-n (acceptance), X-n (mutations),
V-n (visual guards), Q-RLb-n (questions), E-RLb-n (evidence), R-RLb-n (risks).

**One PR, two parts.** Part 1 is budget recovery (C1–C7). Part 2 is the skyline preview (C8), which is one
isolated commit, so that if the operator dislikes the preview it is reverted with one `git revert` while
part 1 stays. Choosing one PR rather than two keeps one worktree and one operator review for one visual
session. The cost is that part 2 waits for part 1's review.

---

## 0. Rulings this PR carries (2026-10-09)

- **QRL-11 (operator):** the ridge rescale may change accepted frames, as a preview the operator judges. If
  it is disliked, it is reverted.
- **QRL-2 (operator):** the terrain is real San Diego, with the skyline at about 1–2°. The preview uses
  real bearings and real elevation angles from named summits. RL-e replaces it with a 3DEP-derived profile.
- **QRL-1 (operator):** both scenes fail. `./mineworld-3d` is labelled as the movement and camera spike
  (C9).
- **QRL-6 (primary):** the tiers are accepted. This PR measures only on the operator's Mac. The Windows and
  Linux checks go on the operator's checklist and into S13's CI, and this PR's evidence says "measured on
  macOS".

## 0.1 Freeze rulings (primary session, 2026-10-09; binding)

| Question | Ruling |
| --- | --- |
| Q-RLb-1 | **One PR**, with the skyline preview as an isolated, separately revertible commit (C8). |
| Q-RLb-2 | **Primitives ≤ 3 M is a pass condition** beside 16.7 ms, 2 000 draw calls and 2 048 MB (M-6). |
| Q-RLb-3 | **Keep mipmaps.** Any visible change to distant paving goes into the operator's side-by-side at PR review (V-4); that is the default, **not a stop**. |
| Q-RLb-4 | **If M-6 can only be met by replacing VoxelGI with SSIL, stop and show the operator** (a material stop, §9). |
| Q-RLb-5 | **V = 80 km** as the preview's presentation default. |
| Q-RLb-6 | **Commit the tool-written `.import` files**, owned by `tools/slice_imports.py`. |
| Merge order | **RL-b merges first** (§10); 16c, 12e and 16d rebase onto it. |
| CI | The `python` jobs' failure (`clock.json` has no SDK model) is a main-wide gap with an SDK fix in flight. It is **not required** and does not block RL-b. The required checks are `fast` and `test`. |

## 1. Identity, base, scope

```text
PR            RL-b — the 3D slice within the Default tier; the skyline preview (S22, MVP-0 visual track)
branch        mvp0/pr-rl-b-3d-budget from origin/main at the start of execution
worktree      /Users/yuema137/mineworld-worktrees/impl-rl-b, held by the implementing session only
depends on    nothing unmerged. Independent of 12d, 16c, 12e and 16d (all frozen, all waiting on 12d)
merge         a merge commit, never a squash; operator authorization only
```

**Goal.** On the operator's Mac (Apple M5, macOS, Godot 4.7.2), the slice meets the **Default tier** of
step-22 §6.6. The thresholds are fixed now (M-6), and the baseline is measured before any change (C1):

```text
every perf view, gi=voxel (the accepted default), 1920x1080, vsync off:
  p95 frame time  <= 16.7 ms      (the median of three runs)
  draw calls      <= 2 000
  primitives      <= 3 000 000
  video memory    <= 2 048 MB     (at the end of the run)
```

The accepted visuals do not regress (V-1 … V-4), apart from the ruled skyline change (V-5).

**Scope.**

| Id | Item |
| --- | --- |
| SC-1 | A measurement instrument fixed before any change: the `--perf` protocol (§3), a category breakdown (`--perf --breakdown`), CPU and GPU time, three runs, and the baseline recorded on the base |
| SC-2 | Imported props: Poly Haven glTFs load as Godot-imported scenes, which gives mesh LODs, shadow meshes and imported textures. The GLTFDocument path stays as the fallback |
| SC-3 | Texture import ownership: committed `.import` files for the slice's textures (VRAM-compressed, high quality, mipmapped, normal maps as normal maps), written by a reviewed tool. Characters are untouched |
| SC-4 | Screen-size culling: a per-instance `visibility_range_end` computed from each instance's bounding radius, so it hides only below 1.5 px at 1080p (invisible by construction). It also removes those instances from the shadow passes |
| SC-5 | A shadow-caster policy: inside enclosed rooms, small objects cast no directional shadow. A pre-registered visual check decides whether it stays |
| SC-6 | Occlusion culling on, with occluders built from opaque massing only (upper storeys, party walls, rear walls, the café's solid walls); never glazing, never doors |
| SC-7 | Conditional levers, each taken only if still needed and only within V-1: VoxelGI 256 → 128 subdivisions; soft-shadow filter quality 3 → 2; SSIL quality |
| SC-8 | **The skyline preview** (QRL-11): the three sine ridges are replaced by summit-anchored bands at San Diego's real bearings and elevation angles. Aerial perspective uses Koschmieder's law, and the backdrop is camera-anchored, so it has no false parallax and stays inside the 1 600 m far plane |
| SC-9 | `./mineworld-3d` is labelled as the movement and camera spike (QRL-1) |

**Non-goals.** Each belongs elsewhere.
- Real terrain data and the terrain baker (RL-e).
- Wind, water and goods (RL-d, RL-f, RL-h).
- Quality tiers in settings (RL-i).
- Any Windows or Linux measurement (QRL-6).
- The character's files and assets.
- Anything 16c, 12e or 16d owns: the doorway binding, figures, colliders, objects, menus.
- The 2D client.
- Any server, kernel, system or world change.

**Change set** (every path this PR may touch):

```text
clients/3d-spike/scripts/slice/slice_perf.gd            new: the protocol and the breakdown (SC-1)
clients/3d-spike/scripts/slice/slice_probe.gd           `_perf` body replaced by a call into SlicePerf;
                                                        `perf_views` moves there (dispatch only)
clients/3d-spike/scripts/props.gd                       gltf(): imported scene first, GLTFDocument
                                                        fallback; a category meta on placed props (SC-2)
clients/3d-spike/scripts/slice/budget.gd                new: screen-size ranges, shadow policy (SC-4, SC-5)
clients/3d-spike/scripts/slice/occluders.gd             new: occluders from massing (SC-6)
clients/3d-spike/scripts/slice/terrace.gd               one line: append each unit's massing to a static
                                                        registry, beside `doors` (SC-6)
clients/3d-spike/scripts/slice/slice_world.gd           two call lines after SliceBatch.merge; the
                                                        backdrop call swapped (C8)
clients/3d-spike/scripts/slice/slice_main.gd            `_voxel_gi` subdiv constant only, if SC-7 takes it
clients/3d-spike/scripts/slice/skyline.gd               new: the preview backdrop (SC-8)
clients/3d-spike/scripts/slice/street.gd                `backdrop()` and `_ridge()` removed (C8)
clients/3d-spike/project.godot                          occlusion culling on; soft-shadow quality only if
                                                        SC-7 takes it
clients/3d-spike/tools/slice_imports.py                 new: writes the texture .import settings (SC-3)
clients/3d-spike/assets/textures/**/*.import            committed, tool-written (SC-3)
clients/3d-spike/assets/models/**/*.import              committed, tool-written (SC-2, SC-3)
clients/3d-spike/tools/frame_diff.gd                    an optional mask argument (V-5); default
                                                        behaviour unchanged
mineworld-slice                                         reimport when any .import or asset is newer than
                                                        the cache; --perf flags pass through
mineworld-3d, clients/3d-spike/README.md                the spike label (SC-9); the new perf flags
docs/HUMAN_REVIEW_QUEUE.md                              a dated VIS-3D-GODOT-2 line: the preview, the
                                                        checklist
.structured-coding/plans/mvp1/pr-rl-b-3d-budget.md      this file's ledger
```

**Paths with no diff:**
- `kernel/`, `contracts/`, `persistence/`, `server/`, `systems/**`, `cognition/`, `worlds/**`, `tools/cli/`;
- `clients/protocol/**`, `clients/2d/**`, `clients/shared/**`;
- the character: `human.gd`, `npc.gd`, `character_slot.gd`, `camera_rig.gd`, `posture.gd`, `player.gd`,
  `clients/3d-spike/assets/characters/**`, `tools/patch_imports.py`;
- 16c's, 12e's and 16d's new or owned files: `slice_link.gd`, `targeting.gd`, `intents.gd`,
  `controls_hud.gd`, `build.gd`, `streetscape.gd`, `cafe.gd`, `shop_interior.gd`, `cafe_interior.gd`,
  `slice_probe_world.gd`.

## 2. Audit anchors (re-verified in C1 against the base)

| Id | Finding | Evidence | Consequence |
| --- | --- | --- | --- |
| A-1 | Baseline cost on the M5 at 1600×900, gi=voxel: 14.7–19.5 ms mean, 3 409–4 420 draw calls, 8.1–10.4 M primitives, 3 468 objects, 2 436 MB video memory. With gi=none: 9.8–13.6 ms, 1 802 MB. So VoxelGI accounts for about 6 ms and about 635 MB | E-RL-1 and its gi=none run, step-22 §2.6 | Budget work must cover geometry and passes (draw calls are the same with GI off), textures (1.8 GB without GI) and GI |
| A-2 | **The perf mode is noisy and short.** It uses 20 samples per view with no warm-up and vsync not controlled. A repeat run showed 8.7 s and 5.6 s worst frames (pipeline compilation or a throttled window), and `--res=800x450` did not change the resolution | E-RLb-0 (a re-run of `--perf`, 2026-10-09) | M-1 … M-5 fix the protocol before any change |
| A-3 | **Every slice texture is imported Lossless with no mipmaps.** Of the generated `.import` files, 231 have `compress/mode=0` and 214 have `mipmaps/generate=false`. The `.import` files under `assets/textures` and `assets/models` are **untracked**, generated by the launcher's `--import` with Godot's defaults. The character's `.import` files are tracked and owned by `tools/patch_imports.py` (`docs/HUMANOID_PROFILE.md`) | `grep` over `assets/**/*.import`; `git ls-files` | SC-3 follows the character's precedent: settings owned by a tool, committed, reviewable as a diff |
| A-4 | **Props bypass the importer.** `Props.gltf` uses `GLTFDocument.append_from_file` at run time, "so nothing has to go through Godot's importer". The scene `.import` files that the launcher's `--import` already generates have `meshes/generate_lods=true` and `meshes/create_shadow_meshes=true`. Neither LODs nor shadow meshes reach the scene, and the textures arrive uncompressed | `props.gd:1–14, 60–74`; `croissant.gltf.import` | SC-2. The premise in the comment is stale, because `mineworld-slice` runs `--import` on first use and whenever a script is newer than the cache |
| A-5 | `SliceBatch` merges primitive meshes that carry a `material_override` into 24 m cells: 5 894 instances become 663 meshes and 79 068 triangles. It deliberately leaves Poly Haven props, labels and colliders alone | `batch.gd` | The remaining draw calls come from glTF props, characters, labels and the per-pass multiplication (depth prepass, colour, four shadow splits) |
| A-6 | Sun: four splits, 95 m, 2 048 atlas, soft-shadow quality 3. SSAO and SSIL on. VoxelGI SUBDIV_256 over the café. Two `UPDATE_ONCE` probes | `slice_main.gd:189–302`; `project.godot` | The levers of SC-5 and SC-7 |
| A-7 | Godot 4.7.2 has `BaseMaterial3D.disable_fog`, `RenderingServer.viewport_set_measure_render_time` and `viewport_get_measured_render_time_{cpu,gpu}`, the occlusion-culling setting, `visibility_parent`, and `ImporterMesh.generate_lods` | E-RLb-1 (a headless probe, 2026-10-09) | The instrument (SC-1), the skyline's own aerial perspective (SC-8) and occlusion culling (SC-6) use engine features, not new code |
| A-8 | The backdrop: three sine ridges **north** of the street (−z) at 320, 520 and 760 m, 62–214 m high, up to 15.7° high, in flat colour. The camera's far plane is `camera_rig.FAR = 1600` m, and `camera_rig.gd` must not change (V-2). Scene frame: +x east, −z north (`space.gd` `to_3d`; `street.gd` header) | `street.gd:147–200`; `camera_rig.gd:29` | SC-8: real bearings, camera-anchored proxies inside the far plane, and the aerial perspective in the material |
| A-9 | `slice_probe.gd` is 1 314 lines, past the review trigger | `wc -l` | New measurement code goes into a new file (`slice_perf.gd`) |

## 3. The measurement, fixed before any change (`ARC-23`: locate before counting)

| Id | Rule |
| --- | --- |
| M-1 | **Views.** These eight views are fixed now, each the pose of an existing shot or perf view:<br>• `street wide` (−20.0, 0.45, −5.20), yaw −75, pitch −1<br>• `cafe frontage` (0.30, 0.45, −5.55), −52, 3<br>• `interior` (2.75, 0.60, −10.60), −38, −3<br>• `doorway` (3.45, 0.45, −6.90), 0, 0<br>• `street east` (22.0, 0.45, −5.30), 96 (shot 17)<br>• `south side` (2.0, 0.45, −4.60), 165 (shot 16)<br>• `florist interior` (11.4, 0.45, −9.10), −38 (shot 21)<br>• `skyline east` (−20.0, 0.45, −5.20), yaw −90, pitch +4 (new: looks east along the street, above the eaves line) |
| M-2 | **Per view.** Place the player and settle 6 frames. Then **120 warm-up frames, discarded** (pipeline compilation). Then **300 measured frames.** Each frame records the wall frame interval and the viewport's measured CPU and GPU render times (`viewport_set_measure_render_time(true)`). Report mean, p50, p95 and max for all three, plus draw calls, primitives and objects (the `Performance` monitors). At the end, report video memory, texture memory and buffer memory |
| M-3 | **Environment.** Vsync off (`DisplayServer.window_set_vsync_mode(VSYNC_DISABLED)`) in perf mode only. The window is frontmost and unoccluded. Nothing else heavy runs; the run log records the machine (`sysctl machdep.cpu.brand_string`), the macOS version, the Godot build and the renderer |
| M-4 | **Resolution.** 1920×1080, the tier's resolution, set by the window size the launcher passes (`--res=1920x1080` is made to work, A-2). Also 1600×900, for continuity with E-RL-1 |
| M-5 | **Runs.** Three fresh processes per configuration. The statistic per view is the **median over the three runs of that view's p95 frame interval**. A run with any measured frame over 250 ms is `INCONCLUSIVE`; it is re-run, up to three times, and never averaged in |
| M-6 | **Pass, fixed now.** gi=voxel, 1920×1080: every view's statistic ≤ 16.7 ms; draw calls ≤ 2 000 in every view; primitives ≤ 3 000 000 in every view; video memory ≤ 2 048 MB. All four must hold. Missing any one after SC-2 … SC-7 is a **material stop** (§9) |
| M-7 | **Breakdown** (`--perf --breakdown`). For `street wide`, `cafe frontage` and `interior`, measure the base once with everything on. Then measure once with each category switched off: merged primitives, glTF props, foliage cards, characters, labels, backdrop, ground, VoxelGI, SSIL, SSAO, directional shadows. Report the deltas. Categories are told apart by a `mw_category` meta set where a node is made (`Props.place`, `SliceBatch`, the backdrop root, `Build.ground` by node name), never by guessing. The breakdown runs only on the base and on the final head. It locates costs and has no pass/fail role |
| M-8 | **Baseline.** C1 runs M-1 … M-5 and M-7 on the base **before any other commit**. The result goes into the ledger as E-RLb-2, and M-6's thresholds are not changed after it is seen |

## 4. Design decisions (SD-RLb-n)

The levers come in order: first those that change no pixel by construction, then those with a bounded
visual risk. Each is its own commit, re-measured (M-5) and frame-guarded (V-1). If a lever fails V-1, it is
reverted inside the PR and recorded. Any further lever is taken only while M-6 is unmet.

| Id | Decision | Alternatives | Why |
| --- | --- | --- | --- |
| SD-RLb-1 | **Imported props** (SC-2). `Props.gltf(slug)` first tries `load("res://assets/models/<slug>/<slug>.gltf")` as a `PackedScene` and instantiates it. If that resource does not exist (no import yet), it falls back to today's `GLTFDocument` path and prints one warning per slug. `retint`, `place`, `_exit_tree`'s release and the template cache keep working on either kind. Import options stay Godot's defaults for scenes (`generate_lods=true`, `create_shadow_meshes=true`), now written to committed `.import` files (SD-RLb-2). The launcher reimports when any `.import` file or asset is newer than the cache | keep GLTFDocument and generate LODs at run time with `ImporterMesh.generate_lods` (load-time cost; reimplements the importer); bake LOD meshes to new files (duplicates assets) | The importer already does this work, and the launcher already runs it (A-4). The fallback keeps the "runs from a clean checkout" property honest |
| SD-RLb-2 | **Texture import ownership** (SC-3). A new `tools/slice_imports.py`, with the same pattern as `patch_imports.py`, sets these keys in every `.import` under `assets/textures/**` and `assets/models/**/textures/**`: `compress/mode=2` (VRAM compressed), `compress/high_quality=true` (BPTC/BC7 on desktop), `mipmaps/generate=true`; and for `*_nor_gl_*` files `compress/normal_map=1` (RGTC/BC5). It never touches `assets/characters/**` or `assets/hdri/**`. The files are committed. Desktop BC formats are supported by Metal on Apple silicon and by D3D12/Vulkan on Windows and Linux, so every OS takes the same path | `[importer_defaults]` in `project.godot` (global: it would also change the character's textures, V-2); runtime `Image.compress` in `Mats.tex` (slow loads; the BPTC encoder at run time); ASTC (not the desktop default) | Owned, reviewable, scoped. Mipmaps also remove the distant-texture shimmer that Lossless without mips causes; V-1 decides whether that change is visible (A-3, R-RLb-2) |
| SD-RLb-3 | **Screen-size culling** (SC-4). A new `SliceBudget.apply(root)` runs after `SliceBatch.merge`. For every `GeometryInstance3D` that is not a merged cell, the ground, the backdrop or a character, it sets `visibility_range_end = r · (H/2) / (tan(fov_v/2) · p)`, where `r` is the instance's world bounding-sphere radius, `H = 1080`, `fov_v` is the camera rig's vertical field of view (read from `SliceCameraRig`, not copied), and `p = 1.5` px. It sets `visibility_range_end_margin = 0.1 · end` and fade off. An object is hidden only once its whole bounding sphere covers less than 1.5 px; a hidden instance also leaves the shadow passes | hand-tuned distances per category (taste, and wrong at another FOV); HLOD with `visibility_parent` (worth it only once there are coarse proxies, RL-e) | Invisible by construction at 1080p and below. At higher resolutions it is conservative by the resolution ratio, and RL-i passes the tier's `H` |
| SD-RLb-4 | **Shadow-caster policy** (SC-5). Inside an enclosed room volume (the slice's `Area3D` place volumes for the café and the florist), an instance whose bounding radius is ≤ 0.30 m gets `cast_shadow = OFF` for **directional** light. Larger furniture keeps casting. Applied by `SliceBudget`. **Kept only if V-1 passes on `interior`, `09_character_in_place`, `10_interior_counter`, `11`, `12`, `21` and `22`**; otherwise it is reverted and recorded | no interior shadows at all (the sun patch on the café floor is part of the accepted look, `slice_main.gd` header); per-object hand flags (touches the 16c/12e-owned build files) | Small interior objects are under a roof. Their only directional shadows fall inside the 4.8 m glazing patch, where cups and jars matter least. The pre-registered frames decide |
| SD-RLb-5 | **Occlusion culling** (SC-6). `project.godot` gets `rendering/occlusion_culling/use_occlusion_culling=true`. A new `SliceOccluders.build(root)` adds `OccluderInstance3D` nodes with `BoxOccluder3D` shapes for: each terrace unit's upper storeys (from the first-floor line up), set 0.15 m behind the façade face; each unit's rear and party walls; and the café's and florist's solid side and back walls. It never adds one over glazing, a door, a shopfront or anything below the shopfront head. Unit massing reaches it through a static `SliceTerrace.massing` registry, appended in `terrace.gd` beside the existing `doors` registry (one line) | an editor-baked occluder (the scene is built from script, so there is no edit-time bake); occluders from merged meshes (glazing cannot be told apart) | Godot's occlusion culling is CPU-rasterized (Embree) and removes instances from the colour and depth passes (not the shadow passes). Its main yield is the interior view, where today the whole street is drawn behind solid walls. C5 verifies it is active on macOS arm64 (A-7 shows the setting exists; the build's support is checked by an occluded-object count) |
| SD-RLb-6 | **Conditional levers** (SC-7), in this order, each only while M-6 is unmet: (a) VoxelGI `SUBDIV_256 → SUBDIV_128`, judged on the café views (V-1 at its limit, and V-3); (b) `soft_shadow_filter_quality 3 → 2`; (c) SSIL `ssil_quality` one step down (project setting). Each is reverted if V-1 fails | swap VoxelGI for SSIL-only in Default (changes the accepted interior; the operator's call, §9); lower the shadow distance (visible on the street) | Ordered by expected yield (A-1: VoxelGI is about 6 ms and 635 MB) against visual risk |
| SD-RLb-7 | **The skyline preview** (SC-8). §5 |
| SD-RLb-8 | **The spike label** (SC-9). `mineworld-3d`'s header and usage text and the 3D README line say it is "the movement and camera spike (accepted 2026-09), not the world, and not the visual standard; see `./mineworld-slice`". It also prints that sentence once at start. No scene change | remove the spike (it is the artefact behind an accepted decision; the operator ruled "label") | QRL-1 |

## 5. The skyline preview (SC-8, SD-RLb-7)

**What changes.** `SliceStreet.backdrop` and `_ridge` are removed. `SliceSkyline.build(parent, camera_rig)`
replaces them, called from the same line in `slice_world.gd`.

**Content: real summits, read as angles.** The town origin is Market Town's configured location, 32.7157 N,
−117.1611 W (`worlds/market-town/configure/calendar.yaml`), with the observer's eye about 20 m above sea level.
Elevation angle = `(h − h_eye − d²·(1 − k)/(2R)) / d`, with Earth radius R = 6 371 km and refraction
coefficient k = 0.13 (the standard surveying value; re-read at freeze). Summit heights and coordinates are
approximate (USGS GNIS / topographic maps, **to be re-read at freeze**). Bearings and distances below were
computed in this design by an equirectangular approximation:

| Summit (band) | Height | Bearing | Distance | Elevation angle |
| --- | --- | --- | --- | --- |
| Point Loma ridge (near, west) | ≈ 129 m | ≈ 249° | ≈ 7.9 km | ≈ 0.76° |
| Mount Soledad (near, north-north-west) | ≈ 251 m | ≈ 330° | ≈ 15.9 km | ≈ 0.77° |
| Cowles Mountain (mid, north-east) | ≈ 486 m | ≈ 49° | ≈ 16.3 km | ≈ 1.58° |
| Mount Helix (mid, east-north-east) | ≈ 418 m | ≈ 71° | ≈ 17.5 km | ≈ 1.23° |
| San Miguel Mountain (mid, east) | ≈ 790 m | ≈ 98° | ≈ 19.0 km | ≈ 2.2° |
| Otay Mountain (far, east-south-east) | ≈ 1 101 m | ≈ 114° | ≈ 32.6 km | ≈ 1.77° |
| Cuyamaca Peak (far, east-north-east) | ≈ 1 986 m | ≈ 64° | ≈ 57.9 km | ≈ 1.72° |
| Laguna Mountains, Monument Peak (far, east) | ≈ 1 881 m | ≈ 75° | ≈ 72 km | ≈ 1.20° |
| West, 255°–310° | sea horizon | — | — | 0° (no land) |

Between summits, each band's profile is a smooth interpolation of its summits' angles, with low-amplitude,
low-frequency relief: at most ±15 % of the band's local angle, and at least 8° of azimuth per undulation. A
band falls to its base at its sector edges. **This shape is a placeholder, and it says so in its code
comment.** RL-e replaces it with a profile computed from USGS 3DEP. The preview fixes size, direction and
atmosphere, which are what made the ridges read as stage flats (step-22 §2.2 causes 1, 4 and 5).

**Geometry.**
- Each band is a ribbon mesh drawn at a proxy distance D_p: near 900 m, mid 1 100 m, far 1 300 m, all inside
  the 1 600 m far plane (A-8). The ribbon's height at each azimuth is `D_p · tan(angle)`.
- The backdrop root copies the active camera's horizontal position every frame, so it shows no parallax, as
  true distance would. It does not rotate.
- No band casts shadows. Band order is far to near, with depth testing.

**Aerial perspective, in the material.**
- Each band's `StandardMaterial3D` has `disable_fog = true` (A-7), so the depth fog tuned for 70–1 500 m does
  not double-count.
- The band's colour is `lerp(horizon, terrain, C)` with `C = exp(−3.0 · d / V)`. This is Koschmieder's law
  at the WMO meteorological-optical-range threshold of 5 % contrast (WMO CIMO Guide, Part I ch. 9; re-read at
  freeze).
  - `d` is the band's real distance.
  - V = 80 km, a clear coastal afternoon. This is a presentation default the operator judges; TW-e will take
    it from the hour's weather.
  - `horizon` is the HDRI's horizon colour, sampled once at the band's bearing from the loaded panorama
    (rows ±1° about the horizon). So the band meets the sky without a seam.
  - `terrain` is a dry chaparral and coastal-sage albedo, sRGB about (0.38, 0.36, 0.27). That is a design
    default within DEP-8's albedo band 0.2–0.7.
- The material is unshaded, because at these distances the lit/unlit contrast is below the haze. A vertical
  gradient of −8 % luminance from crest to base suggests slope shading.
- The sun is unchanged (ARC-13).

**What stays.** The 900 m ground plane, the east bank and the west parapet stay. The fog settings stay. The
HDRI stays, because the moving sun is TW-e's.

**Its own checks.**
- `--perf`'s `skyline east` view (M-1).
- `--skyline-check`, a new probe mode. It raycasts the drawn crest at each whole degree of azimuth from the
  camera position, converts the hit back to an elevation angle, and compares it with the table's interpolated
  angle: PASS within ±0.1°. A deliberate 90° rotation of the bearings must FAIL it (X-5).

## 6. Acceptance and adversarial criteria (fixed before measuring)

| Id | Criterion | Evidence |
| --- | --- | --- |
| A-1 | **Default tier on the operator's Mac** (M-6), gi=voxel, 1920×1080, all eight views | E-RLb-final: three runs, with the table and the median-of-p95 per view |
| A-2 | The **breakdown** on the final head shows where each saved millisecond and draw call came from. Each lever's commit records its own before and after (M-5) | the ledger, per commit |
| A-3 | **Imported props are what is drawn.** With the cache present, `Props` reports 0 GLTFDocument fallbacks. With the imported resources removed in a scratch copy, it reports the fallback per slug and the scene still builds | `--measure` log line `props: N imported, 0 fallback`; the scratch run |
| A-4 | **Compressed textures are in use.** Texture memory falls, and a material probe reports BPTC/RGTC formats for one albedo, one ORM and one normal map, and RGBA8 for the untouched character textures | `tools/material_probe.gd` (existing), extended with a format column, or a new probe line |
| A-5 | **Screen-size culling hides only sub-pixel objects.** For every instance given a range end, its projected bounding-sphere diameter at that distance is ≤ 1.5 px at 1080p, asserted in code at build time and counted in the log | the build log |
| A-6 | **Occlusion culling is active.** In `interior`, the number of instances culled by occlusion is > 0. It is read from `Performance.get_monitor` objects-in-frame with and without the setting (breakdown) | M-7 |
| A-7 | **Nothing simulated changed.** `--drive`, `--threshold`, `--measure`, `--character`, `--world --link` and `--world --conversation` pass as on the base; no collider moved (colliders are separate nodes, and the batching and budget passes touch only `GeometryInstance3D`) | each mode's log |
| A-8 | **Skyline** (§5): `--skyline-check` passes; no band crest exceeds 2.3°; the west sector 255°–310° shows the sea horizon (no band) | the probe log |
| A-9 | **Load time** does not grow by more than 20 % (`build batched` total in the log), because imported scenes load faster than parsed glTF | the build timing lines |

**Mutations** (each must make its check FAIL, and be reverted):

| Id | Mutation | Must fail |
| --- | --- | --- |
| X-1 | `p = 6` px in SD-RLb-3 | A-5 (visible culling) and V-1 on `street wide` |
| X-2 | One occluder placed across the café glazing | V-1 on `03_cafe_exterior` and `11_interior_looking_out` |
| X-3 | `slice_imports.py` also run over `assets/characters/**` | the V-2 path check |
| X-4 | M-6's draw-call ceiling checked against a run with `SliceBatch.merge` skipped | A-1 (draw calls far above 2 000) — proves the instrument sees batching |
| X-5 | Skyline bearings rotated by 90° | A-8 |
| X-6 | `Props.gltf` forced to the fallback | A-3's imported count, and A-1 (LODs absent) |

## 7. The visual-regression guards

```text
V-1  STANDALONE FRAMES (16c §20.6 and 12e §22.6, same rule). --shots at 1600x900, twice on the base
     (the noise floor) and once on each lever's head; tools/frame_diff.gd per view. PASS iff each
     view's share of pixels differing by more than 8/255 is at most the base's own run-to-run share
     + 0.5 points. Over that: looked at side by side, one image at a time, cause named. An
     unexplained visible change is a material stop. Applied after every lever commit (C2-C7); C8 uses
     V-5 instead
V-2  THE CHARACTER. No diff to the character's files or assets (path check over the PR diff, §1);
     --character passes
V-3  CONNECTED FRAMES. --world --conversation's three frames, base and head: the accepted connected
     checklist holds (names on every person, the door toast, Alice in plain view at the counter, both
     caption lines); the only differences are those V-1 explained
V-4  MIPMAPS (SD-RLb-2). If V-1 flags a street view only in far paving or roof texture (frame_diff's
     boxes in the upper or far part of the frame), the side-by-side is shown to the operator as an
     expected change (less shimmer), not reverted autonomously. Any other location is V-1's rule
V-5  THE SKYLINE (ruled QRL-11). For every --shots view, a backdrop mask is rendered on the base and on
     the head (the view with the backdrop hidden versus shown; pixels that differ by more than 8/255).
     tools/frame_diff.gd with the union of the two masks as the ignore region: outside the mask,
     V-1's rule holds. Inside it, the change is the preview, judged by the operator (§8)
```

## 8. The operator's visual checklist

Run `./mineworld-slice` (standalone) and `./mineworld-slice --world`. Look for each of the following.

1. **Smoothness.** Walk the street end to end, enter the café and the florist, come back out. Look for: no
   hitch at doors, no pop-in of objects as you approach (screen-size culling should be invisible), no
   flicker on cobbles or roof tiles at distance (mipmaps should reduce it).
2. **The café inside.** Stand at the counter and at the back wall. Look for: the warm room and the sun patch
   on the floor are as accepted; small objects on shelves and tables look as before. If VoxelGI was reduced
   (SD-RLb-6a), check whether the room is flatter or blotchier.
3. **Through the glass.** From the street, look into the café and the florist. Look for: goods, tables and
   people visible through the glazing (nothing culled behind glass).
4. **Shadows.** Look at the tables on the terrace and the paving in the low sun. Look for: the long raking
   shadows of the accepted look are unchanged.
5. **The skyline (the preview).** Look east along the street, and west from the east end. Look for:
   - mountains low on the horizon (about the width of a finger at arm's length), layered from nearer
     grey-green to fainter blue-grey further away;
   - no seam where land meets sky;
   - an open sea horizon to the west;
   - nothing that moves wrongly as you walk.
   Ask whether it reads more real than before. If not, say which single thing is most wrong; the preview is
   reverted on request.
6. **The old spike.** Run `./mineworld-3d`. It now says at start that it is the movement and camera spike,
   not the world.
7. **Not checked by this PR (QRL-6).** The Windows and Linux tier checks are on the operator's later
   checklist and go into S13's CI. This PR's evidence is macOS only.

## 9. Material stops

- M-6 is unmet after SC-2 … SC-7. Report the breakdown, the remaining gap per view, and the options with
  their visual cost (for example, VoxelGI → SSIL in Default, or a lower shadow distance). The operator
  chooses.
- Any change outside §1's change set, or any diff in a "no diff" path.
- An unexplained V-1 change, or a V-2 or V-3 failure.
- A conflict with a merged 16c, 12e or 16d that needs more than a mechanical rebase (§10).

## 10. File overlaps and merge order

| File | 16c (frozen) | 12e (frozen) | 16d (frozen) | RL-b | Conflict size |
| --- | --- | --- | --- | --- | --- |
| `slice_world.gd` | `build(parent, connected)`; volumes | — | — | two call lines after `SliceBatch.merge`; the backdrop line | adjacent lines; mechanical |
| `slice_main.gd` | `connected` computed before build; the HUD wording | the F/G/R hook, a HUD line | 16d's menu wiring (per its plan) | `_voxel_gi` subdiv only, and only if SD-RLb-6a is taken | separate functions; mechanical |
| `terrace.gd` | two fields on `doors` entries (D-16c-4) | — | — | one `massing.append` line beside `doors` | adjacent lines; mechanical |
| `slice_probe.gd` | — (16c's probes are in `slice_probe_world.gd`) | — | — | `_perf` body becomes a call; `perf_views` moves | none expected |
| `props.gd` | — | — | — | `gltf()`, a meta in `place()` | none |
| `streetscape.gd` | `with_people` parameter | the bank tree prune | — | **not touched** (budget is a post-pass) | none |
| `project.godot` | — | — | — | occlusion culling (+ soft-shadow quality if SC-7b) | none expected |
| `mineworld-slice` | modes | `--geometry`, `--bodies`, `--rules`, pass-through | modes | the reimport condition, `--res` fix, perf flags | separate lines; mechanical |
| `tools/frame_diff.gd` | uses it | uses it | — | an optional mask argument; default unchanged | none (callers unaffected) |
| `assets/**/*.import` | — | V-2 path check covers `assets/**` (12e's own diff) | — | committed, tool-written | none (no other PR writes them). Their V-2 path checks are over their own diffs |
| `docs/HUMAN_REVIEW_QUEUE.md` | dated lines | dated lines (SD-E15) | dated lines | a dated line | append-only; mechanical |

**Merge order: RL-b first.** All three frozen PRs wait on 12d (16c: D-16c-1; 12e: 12d and 16c; 16d: per its
plan), and RL-b depends on nothing unmerged. Each of them then rebases onto a main that includes RL-b. Their
V-1 baselines are taken on that main, so RL-b's visual changes (the preview, possibly mipmaps) are never
counted against them. If any of them merges before RL-b, RL-b rebases and re-runs C1's baseline on the new
main, because M-6's thresholds are fixed and the baseline is evidence, not a threshold. The one-working-tree
rule applies: `impl-rl-b` is this PR's alone.

## 11. Commit plan

Each commit tracks implementation, deterministic validation and LLM logic review as separate items, each
with evidence.

### C0 — `docs(plan): RL-b design` (this file; frozen by the primary session)
- [ ] Implementation: the design, the execution contract.
- [ ] Validation: `check_doc_headings.py`, `check_decision_ids.py`.
- [ ] Review: freeze review by the primary session.

### C1 — `3d: a perf protocol fixed before measuring; the baseline`
- [x] Implementation: `slice_perf.gd` (M-1 … M-5, M-7); `slice_probe.gd` dispatch; `mineworld-slice`
  `--res` fix and `--perf --breakdown`; the `mw_category` meta in `props.gd` `place()`, the backdrop root and
  the merged cells. No behaviour change in any other mode. *Evidence:* `slice_perf.gd` (new);
  `slice_probe.gd` `_perf()` is one call; `mineworld-slice` loops three runs with up to three re-runs each
  and passes `--res=`; meta in `Props.gltf` (D-2), `street.gd` backdrop root, `batch.gd` `_category`
  (D-3).
- [x] Validation: the base measured, three runs at each resolution, plus the breakdown → E-RLb-2 in the
  ledger. V-1 on this commit: identical within noise. *Evidence:* E-RLb-2. V-1 noise floor: two `--shots`
  runs on this tree (`shots/slice/rlb/base1`, `base2`), `frame_diff.gd`: every view ≤ 0.037 % of pixels
  over 8/255 (09c_character_close 0.037 %, all street and interior views ≤ 0.009 %). The C1 diff changes
  no rendering state, so these frames are the base's and are V-1's reference for C2–C7.
- [x] Review: the instrument measures what it is named after (`ARC-23`). X-4 run once and recorded.
  *Evidence:* X-4 (`SliceBatch.merge` replaced by a no-op, one run at 1920x1080): draw calls 10 772 –
  21 654 per view against 3 409 – 4 420 merged; CPU render time 2.8–7.5 ms p95 against 1.1–2.0 ms. The
  draw-call ceiling sees batching; mutation reverted (`shots/slice/rlb/c1-x4.log`). Review findings:
  GPU timestamps absent on Metal (recorded, the wall interval is the judged statistic); the breakdown's
  base drifts in `cafe frontage` after the `gltf` row (recorded; no pass/fail role).

### C2 — `3d: props load as imported scenes (LODs, shadow meshes)`
- [x] Implementation: SD-RLb-1; `mineworld-slice` reimport condition; the scene `.import` files committed.
  *Evidence:* `Props.gltf` loads `<slug>.gltf` as a `PackedScene` when the import exists, else
  `_parse()` (the old GLTFDocument path) with one warning per slug; `Props.imported` / `fallback`
  counted and printed by `slice_world.gd` after the batch line. `mineworld-slice` reimports when the
  cache or a `.godot/mineworld-import.stamp` is missing, or any script, `.import`, glTF, bin, glb, jpg
  or png is newer than the stamp. The scene `.import` files were already tracked (D-1) with
  `generate_lods=true`, `create_shadow_meshes=true`; no diff to them. `frame_diff.gd` gains the
  optional `--mask=` (V-5, planned for C8, landed here with the tool's other option) and `--heat=`
  (a red-over-darkened locator image) — default output unchanged (re-run on C2's frames printed the
  identical lines).
- [x] Validation: A-3, X-6; M-5 re-measure; V-1; A-7's `--drive`, `--measure`. *Evidence (E-RLb-3):*
  A-3 `props  36 imported, 0 fallback`. X-6 (fallback forced): `props  0 imported, 36 fallback`, 36
  warnings, the scene builds, primitives back to 8.2–10.5 M (base values) — FAIL as required, reverted.
  M-5 at 1920x1080 (three conclusive runs, median p95 ms / max draws / max primitives):
  street wide 13.82 / 4155 / 2.11 M; cafe frontage 22.12 / 4426 / 3.70 M; interior 17.63 / 3912 /
  3.75 M; doorway 23.75 / 3428 / 3.17 M; street east 15.54 / 4215 / 2.34 M; south side 14.35 / 614 /
  0.17 M; florist interior 20.08 / 3649 / 3.53 M; skyline east 14.71 / 4047 / 2.08 M; video memory
  2 515.9 MB. M-6 still FAIL on all four; primitives fall 64–76 % per view. A-9: `build batched`
  7.4–8.6 s against 7.5–9.0 s on the base (no growth). `--drive`: all drive checks pass. `--measure`:
  prop extents identical to C1's run; the one out-of-range line (occupant stature 1.802 m vs
  1.70–1.80) is the same on C1 (pre-existing, character-owned, not this PR's).
  V-1 (base1 vs c2): 19 of 30 views over the 0.5-point bound (max 18_pavement_detail 5.23 %,
  12_back_wall 2.26 %). Located with `--heat` (`shots/slice/rlb/heat-c2/`): every changed pixel is on
  a Poly Haven prop surface (leaves, pot, crockery, cakes, chair seats); nothing else moves. Cause
  named: the props' textures now come through Godot's importer, whose committed settings for those
  textures are Lossless with no mipmaps (A-3), where the run-time parser generated its own textures;
  the surfaces alias differently (fine speckle). C3 sets mipmaps and VRAM compression on exactly
  these textures, so V-1 for prop surfaces is judged on C3's head against the base, not here.
- [x] Review: the fallback is exercised; `retint` on imported materials gives the same tint.
  *Evidence:* X-6 exercised the fallback end to end. `retint` duplicates whatever `BaseMaterial3D`
  the mesh carries and multiplies its albedo; imported materials are `StandardMaterial3D` like the
  parser's, and the heat maps show no prop changing hue (speckle only). `SliceMain._exit_tree` still
  frees the templates (instantiated nodes). `SliceProps.keep`'s suffix matching sees the same child
  names (the `--measure` prop table is identical).

### C3 — `3d: the slice's textures are VRAM-compressed and mipmapped, owned by a tool`
- [ ] Implementation: `tools/slice_imports.py`; texture `.import` files committed.
- [ ] Validation: A-4, X-3; M-5; V-1 and V-4.
- [ ] Review: no character path touched; the tool is idempotent (a second run gives no diff).

### C4 — `3d: screen-size culling`
- [ ] Implementation: `budget.gd` part 1; the call in `slice_world.gd`.
- [ ] Validation: A-5, X-1; M-5; V-1.
- [ ] Review: FOV read from the rig, not copied; characters, ground and backdrop excluded.

### C5 — `3d: occluders from opaque massing`
- [ ] Implementation: `occluders.gd`; the `terrace.gd` registry line; `project.godot`.
- [ ] Validation: A-6, X-2; M-5; V-1 (especially `03`, `11`, `13`, `22`).
- [ ] Review: no occluder intersects any glazing or door box (asserted at build time).

### C6 — `3d: small interior objects cast no sun shadow` (kept only if V-1 passes)
- [ ] Implementation: `budget.gd` part 2.
- [ ] Validation: M-5; V-1 on the pre-registered interior views (SD-RLb-4).
- [ ] Review: the policy keys on room volumes, not names.

### C7 — `3d: conditional levers` (only while M-6 is unmet; each sub-lever its own commit)
- [ ] Implementation: SD-RLb-6 (a), then (b), then (c), stopping as soon as M-6 holds.
- [ ] Validation: M-5; V-1 and V-3 for each sub-lever. Final: A-1 three runs → E-RLb-final; the breakdown on
  the head (A-2).
- [ ] Review: every lever not taken is recorded as "not needed", with the numbers.

### C8 — `3d: a San Diego skyline preview (QRL-11)` (isolated; revertible alone)
- [ ] Implementation: `skyline.gd`; `street.gd`'s backdrop removed; `slice_world.gd`'s line;
  `frame_diff.gd` mask; `--skyline-check`.
- [ ] Validation: A-8, X-5; V-5; M-5 (`skyline east` within budget); the summit table re-read at source and
  recorded.
- [ ] Review: no constant copied from the camera rig; the placeholder nature stated in code.

### C9 — `3d: label the promenade spike; docs`
- [ ] Implementation: SD-RLb-8; README; the `HUMAN_REVIEW_QUEUE.md` dated line with §8's checklist.
- [ ] Validation: `./mineworld-3d` prints the label; the doc checks.
- [ ] Review: no scene change in the spike.

### C10 — `docs(plan): RL-b evidence`
- [ ] The ledger, the final measurement and the review package (`ARC-20`): frames base/head per view, the
  skyline before/after, the perf tables.

## 12. Questions for the freeze (primary session; **[OM]** = the operator's)

| Id | Question | Recommendation |
| --- | --- | --- |
| Q-RLb-1 | One PR with the preview as an isolated commit, or two PRs? | **One PR** (§ header): one worktree, one review; the preview reverts alone |
| Q-RLb-2 | Is primitives ≤ 3 M a pass condition beside 16.7 ms, 2 000 draw calls and 2 GB, as step-22 §6.6 states? | **Yes**, as fixed; if it alone fails, that is a material stop and the primary session rules |
| Q-RLb-3 **[OM]** | If mipmaps visibly change distant paving (V-4), keep them? | **Keep**: less shimmer is the real-world look; shown side by side |
| Q-RLb-4 **[OM]** | If M-6 needs VoxelGI → SSIL in Default (a visible interior change), may the PR take it, or stop? | **Stop and show** (§9); the operator chooses |
| Q-RLb-5 | Visibility V = 80 km for the preview's haze? | **Yes** as a presentation default; TW-e will drive it from weather |
| Q-RLb-6 | Committing generated `.import` files for scene and texture assets (about 230 files) is a new practice outside the character's | **Yes**, owned by `tools/slice_imports.py` on the `patch_imports.py` precedent; reviewable diffs |

## 13. Proposed execution contract (fields; confirmed at freeze)

```text
PROJECT / PR        MVP-0 visual track · S22 / PR RL-b — the 3D slice within the Default tier; the
                    San Diego skyline preview
PRIMARY DESIGN DOC  .structured-coding/plans/mvp1/pr-rl-b-3d-budget.md (this file)
RELATED / BINDING   step-22-realism.md §§2.6, 6.5, 6.6, 9.2, 11.2, 13.1; step-15 §20.6 (16c V-guard);
                    step-11 §22.6 (12e V-guard); VISUAL_FIDELITY; HUMAN_REVIEW_QUEUE (VIS-3D-GODOT-1/-2);
                    ART_DIRECTION §§3–5, 18; DECISIONS ARC-13, ARC-20, ARC-23, ARC-24, DEP-4, DEP-8;
                    docs/HUMANOID_PROFILE.md (the character's import ownership)
PRECONDITION        none unmerged
IMPLEMENTATION BASE origin/main at the start; branch mvp0/pr-rl-b-3d-budget; worktree
                    /Users/yuema137/mineworld-worktrees/impl-rl-b (this session only; confirm no other
                    session holds it before editing)
APPROVED SCOPE      §1 SC-1 … SC-9, as answered by Q-RLb-1 … 6
FROZEN INVARIANTS   §1's change set and "no diff" paths; M-1 … M-8 and M-6's thresholds never changed
                    after a measurement; V-1 … V-5; no server/kernel/system/world change; DEP-8 for every
                    file (no new asset is added)
SEQUENCE            C0 (freeze) → C1 … C10, each committed and pushed when coherent
COMMANDS            ./mineworld-slice and ./mineworld-3d modes; godot --headless --path clients/3d-spike …
                    and clients/3d-spike/tools/*.gd; python3 clients/3d-spike/tools/slice_imports.py;
                    python3 scripts/check_doc_headings.py, check_decision_ids.py, check_scratch.py,
                    check_client_rules.py; cargo fmt / check / clippy -D warnings / test
                    ($HOME/.cargo/bin/cargo if needed; once on the final head); git and gh (no merge);
                    mkdir -p; sed -n. Never python3 -c, sed -i, awk, xargs, curl or heredoc writes. One
                    Godot window at a time; kill only this run's own server
VALIDATION BUDGET   perf runs ≤ ~15 min each (three runs × two resolutions × eight views), background when
                    > 2 min; total ≤ ~6 h of wall time including retries; real-model NOT REQUIRED
LIVE DOCUMENTATION  §11 checkboxes, §14 ledger
HANDOFF             §14's handoff block
ENDPOINT AUTHORITY
  implementation + local validation   after the freeze
  semantic commits, branch push       authorized by the freeze
  PR creation / update                authorized, READY FOR OPERATOR REVIEW
  CI repair                           authorized: `fast` and `test` green on the exact head
  merge                               explicit operator authorization only, merge commit
POST-MERGE SYNC     the planning session owns step-22 and overall; the implementing session owns this
                    file's ledger, deviations and merge identity
NORMAL STOP         PR RL-b READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       §9
```

## 14. Ledger (live)

```text
E-RLb-0  2026-10-09  planning: ./mineworld-slice --perf re-run on f80bbb7 (M5): gi=voxel worst frames
                     8 731.83 ms (cafe frontage) and 5 616.94 ms (doorway) in one run; --res=800x450
                     printed 1600x900 (A-2). gi=none: 9.79–13.56 ms mean, 1 801.5 MB (A-1)
E-RLb-1  2026-10-09  planning: headless API probe, Godot 4.7.2: disable_fog, measure_render_time
                     (cpu/gpu), occlusion-culling setting, visibility_parent, ImporterMesh.generate_lods
                     all present (A-7)
FREEZE   2026-10-09  DESIGN FROZEN by the primary session; rulings §0.1
E-RLb-2  2026-10-09  C1 baseline, recorded before any other commit (M-8). The C1 diff changes no
                     rendering state (a `mw_category` meta on nodes and the perf mode moved into
                     slice_perf.gd), so its frames are the base's. Machine: Apple M5 | macOS 26.2.0 |
                     Apple M5 (Apple9) Metal 4.0 | Godot 4.7.2-stable (official) | forward_plus.
                     Session dirs under clients/3d-spike/shots/slice/perf/ (ignored by Git);
                     logs copied to shots/slice/rlb/c1-*.log.

  1920x1080, gi=voxel, three conclusive runs, median of p95 (ms):
    view               median p95   p95 per run            draws   primitives
    street wide            16.23    15.13, 17.59, 16.23    4155     8 784 353
    cafe frontage          25.09    23.33, 26.56, 25.09    4420    10 460 745
    interior               20.31    20.15, 21.50, 20.31    3899     9 639 642
    doorway                27.26    26.76, 27.26, 44.88    3409     8 150 691
    street east            17.82    16.14, 17.82, 30.45    4215     8 522 496
    south side             15.22    15.22, 15.25, 14.37     614       372 639
    florist interior       19.74    36.29, 19.74, 17.90    3642     8 515 551
    skyline east           17.06    35.31, 17.06, 16.71    4047     8 732 462
    video memory 2 503.6 MB (texture 2 064 MB, buffer 108 MB)
    M-6  FAIL on all four: p95, draws, primitives, video memory

  1600x900, gi=voxel (continuity with E-RL-1), median of p95 (ms):
    street wide 13.49 | cafe frontage 18.87 | interior 15.69 | doorway 20.25 | street east 13.87 |
    south side 10.88 | florist interior 14.26 | skyline east 14.30 | video memory 2 435.7 MB

  M-7 breakdown (1920x1080, one run; d = category off minus all on):
    category    members                street wide           cafe frontage          interior
                                       d p95  d draws d prims  d p95 d draws d prims   d p95 d draws d prims
    merged      592 cells, 74 612 tris -1.21  -2250   -0.38M   -4.60 -2161  +0.14M    -1.48 -2158  -0.43M
    gltf        203 prop roots         -1.87  -1415   -8.03M   -1.55 -1548  -9.55M    -4.22 -1637  -9.47M
    foliage     70 cells, 4 454 tris   -0.33  -180     0.0M    (drift) +68   +0.58M   -0.02 -134    0.0M
    characters  1 (the occupant)       -0.30   0       0.0M    (drift)                +0.09  0       0
    labels      41                     -0.21  -34      0.0M    (drift)                +0.19 -6       0
    backdrop    1 root                 -0.21  -3       0.0M    (drift)                +0.04 -3       0
    ground      1 cell, 2 tris         -0.05  -15      0.0M    (drift)                +1.28 -20      0
    voxelgi     1                      -0.34   0       0       -7.95  (drift)         -3.46  0       0
    ssil        1 env                  -1.68   0       0       -2.73  (drift)         +0.18  0       0
    ssao        1 env                  -0.32   0       0       -1.54  (drift)        +20.16  0       0 (noise)
    shadows     1 sun                  -1.68  -2185   -4.28M   -3.50 -2495  -5.53M    -1.69 -2860  -6.62M
    occlusion   (off on the base)      not measured

  Readings (ARC-23: locate before counting):
  - Primitives are glTF props: switching them off removes 8.0-9.5 M of 8.8-10.5 M primitives. They
    carry no LODs (A-4), and each prop is drawn again in every shadow split.
  - Draw calls split about evenly between merged cells + props in the colour/depth passes and the
    sun's four shadow splits (-2 185 to -2 860 draws with shadows off).
  - VoxelGI is the largest single frame cost in the cafe frontage (-7.95 ms) and interior (-3.46 ms).
  - Instrument finding: in `cafe frontage` every row after `gltf` reads about +246 draws and +0.59 M
    primitives above the all-on base, so the base taken first was not the state the later rows ran in
    (drift after re-showing props). The deltas marked (drift) are against a shifted base and are not
    used. The breakdown has no pass/fail role (M-7); on the final head it is read with this caveat.
  - GPU time: Godot 4.7.2's Metal driver reports no GPU timestamps (the gpu columns read 0 in every
    run). The wall interval with vsync off is the frame cost M-5 and M-6 judge; CPU render time is
    recorded (1.1-2.0 ms p95 everywhere).
  - Run-to-run noise at 1920x1080 is large (florist interior 36.29 vs 19.74 ms; another session's
    headless Godot tests were running on the machine during these runs). The median of three is the
    statistic M-5 fixes, unchanged.

Bounded discoveries recorded at C1:
  D-1  A-3 is stale on one point: the scene and texture `.import` files under assets/models and
       assets/textures are already tracked (253 files, since #50). SC-3 is unchanged: the tool now
       owns settings in files Git already tracks.
  D-2  Most props are placed by `SliceProps.put` / `hang` (dressing.gd, not in the change set), which
       call `Props.gltf` directly. The `mw_category` meta is therefore set in `Props.gltf`, where every
       prop node is made, rather than in `Props.place`.
  D-3  `Build.ground`'s plane and the foliage cards are primitive meshes with a material override,
       so `SliceBatch` merges them. Their category is decided in batch.gd when the cell is made
       (a cell whose members were all `ground` planes is `ground`; an alpha-scissor card material is
       `foliage`). batch.gd is not in §1's change set, but C1 names "the merged cells", so this edit
       is the one C1 itself requires.
  D-4  `--res` did nothing because the project's stretch mode is `viewport`, which renders at
       `content_scale_size`; SlicePerf sets it (M-4). Verified: the run header reports the viewport
       texture's size, 1920x1080.
  D-5  The camera rig class is `CameraRig` (camera_rig.gd), not `SliceCameraRig`; SD-RLb-3 reads
       `CameraRig.FOV`.
```

**Handoff.** C1 implemented and measured (E-RLb-2); next: V-1 noise floor (two base `--shots`), then
commit C1 and start C2 (imported props). Worktree `impl-rl-b`, branch `mvp0/pr-rl-b-3d-budget`.
