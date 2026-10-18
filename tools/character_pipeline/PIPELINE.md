# The character pipeline — specification

Status: S26-b1, revision of 2026-10-14. Authority for the schemas, bounds, stage order, command line and provenance
of `tools/character_pipeline/`. Its decisions are recorded in `docs/DECISIONS.md` (`ARC-94`, `DEP-51`, `DEP-52`,
`DEP-55`). The step that owns the standard is `.structured-coding/plans/mvp0/step-26-cast.md`. Terms follow
`docs/CORE_CONCEPTS.md`.

## 1. Purpose and rules

The pipeline turns one generated mesh (a Meshy image-to-3D download) and one character record (`fit.yaml`) into a
game-ready character export, and checks the export against the per-character standard (`ARC-94`). It is tooling
under the rules of step-26 §5.1:

- It runs on an operator's machine, on demand. It writes **candidates**, never accepted assets.
- No server, client, kernel, System, contract, World Pack or cognition module imports it, and it is never a runtime
  dependency of MineWorld.
- Python 3.12 drives it. Blender 5.2.2 LTS runs its stages headless. Godot 4.7.2 runs the import check of §9.
- Every pipeline output goes to a scratch root outside the repository (§10). A scratch root inside the repository is
  refused.
- Per-character numbers live in that character's `fit.yaml`, never in code (I-B1-9). Blender scripts contain only
  method constants, each listed in §2.4 below, and parameters read from `params.json`.
- A gate bound is a literal in `mwcast/standard.py` with its source. No bound is computed from a candidate
  (`ARC-23` rule 2, I-B1-6). Every automatic gate prints where it looked before its verdict (`ARC-23` rule 1).

## 2. Stages

### 2.1 Stage table

| # | Stage | Implementation | D+ source | Notes |
| --- | --- | --- | --- | --- |
| 1 | Casting brief | not in this PR | — | S26-cast |
| 2 | Reference images | not in this PR | — | S26-b2 / S26-cast; `mineworld_gen` reused |
| 3 | Mask | `mwcast/mask.py` (tool venv) | `prep_inputs.py` | crop box from the alpha bounding box by default; `fit.mask.crop_box` overrides; session `birefnet-portrait`, `fg_ratio` 0.85, 1024 px, alpha threshold 16 |
| 4 | Meshy image-to-3D | `mwcast/meshy.py` | `meshy_i23d.py` | not called live in this PR; `--dry-run` only |
| 5 | Raw gate | `generation_log.yaml` schema | D+ §3 | agent judgement recorded per run; schema only |
| 6 | Game-ready mesh | `blender/gameready.py` | `gameready_mesh.py` | constants from `fit.gameready`; writes `gameready.json` |
| 7–8a | Fit armature, bind, head and pack-core fixes, smooth | `blender/fit_bind.py` | `rig_meshy.py` | armature from `fit.fit.rig_source`; hip front from `fit.fit.charmorph_front_y_at_hips`; `S`, `SOLE_M`, `SOLE_Z` read from `gameready.json` |
| 8b | Finger re-weight | `blender/weights_fingers.py` | `reweight_fingers.py` | |
| 8c | Pack side panels | `blender/weights_pack.py` | `reweight_pack.py` | only when `fit.accessories` contains `"backpack"` |
| 9 | Fleck repair | `blender/flecks.py` | `fix_flecks.py` | on by default for new characters (Q-B1-5); off for Vera (F-B1-4) |
| 10 | Export | `blender/export.py` | the export block of each D+ script | GLB (parity) and the stored-once format (§9); material renamed `fit.export.material` |
| 11 | Runtime tuning | not in this PR | — | S26-b2 |
| 12 | Gates and review frames | `mwcast/gates.py`, `blender/render_bones.py` | §§7–8 of D+; `render_rig.py` | static gates here; runtime gates G-7…G-13 in S26-b2 |

`rebake_head.py` is **dropped** (step-26 §5.2 stage 9), with D+ §8.4 step 7 as the reason.

### 2.2 Blender stage invocation

Every Blender stage is one subprocess with an argument list (never a shell string):

```text
<blender> --background --factory-startup [<input.blend>] --python-exit-code 1
          --python tools/character_pipeline/blender/<stage>.py -- <params.json>
```

`params.json` is written by the driver into the run directory and holds every value the script uses; nothing comes
from the environment. A script writes `<stage>.blend` (where the next stage needs it) and `<stage>.json` (every number
it decided, the same numbers D+ printed), and prints them too. `--factory-startup` keeps the user's preferences out of
the result (R-B1-3).

### 2.3 Port rule

Each Blender script keeps the D+ code statement for statement, in the same order. A typed constant becomes a read of
`params.json`; an environment variable becomes a params key; a print stays and is also written to the result JSON.
Nothing else changes. Every deviation is listed by its statement in the script's README and in the B7 port review.

### 2.4 Method constants

Method constants are the D+ values that describe how a stage works, not what a character looks like. They stay in
the Blender scripts, each with its D+ line in a comment:

- `fit_bind.py`: pick-to-joint slab half-widths `rx 0.03/0.04/0.02`, `rz 0.02`; the `xmin` filters `0.2/0.3`; the toe
  offset `0.012`; the foot selector `|x − xa| < 0.06, z < 0.06` (`rig_meshy.py`).
- `weights_fingers.py` and `weights_pack.py`: the algorithm constants of `reweight_fingers.py` and `reweight_pack.py`:
  the distance floor `1e-4` and `1e-12`, the keep-threshold `1e-4`, the smoothing `factor 0.5` and `limit 4`, the
  pack smoothing zone margins `0.02`, `0.13` and `0.02` (`reweight_pack.py:66-67`), and the `ARMISH` name suffixes.
- `flecks.py`: the straddle test (`6.0` times the median density and `12.0` texels, `fix_flecks.py:76`), the texel
  slack `1.5`, and the luminance weights `0.2126`, `0.7152`, `0.0722`.
- `fit_bind.py` also: the weight-sum floor `1e-4` (unweighted vertices), the slab minimum of 5 vertices, and the
  midpoint of the elbow and the wrist when no elbow pick exists.
- `fit_bind.py` reads the CharMorph front surface from the mesh named by `cm_front_mesh`: `cm_vitruvian` in D+'s
  `body_g5.glb`; `Body` in `vitruvian.glb`, which measured the same hip front within 1.0e-7 m (F-B1-B7-1).
- The driver's `GAMEREADY_HANDOFF_DECIMALS = 4` (`mwcast/pipeline.py`, F-B1-B10-3): `fit_bind` receives stage 6's
  `scale` and `sole_raw` rounded to 4 decimals, as D+ handed them over (`gameready_mesh.py:45` printed them with
  `:.4f`; `rig_meshy.py:26` typed `0.9563` and `-0.9516`). `gameready.json` keeps the full values. With the full
  values every joint moves by about 2.6e-5 m, the pack core gains one vertex (2,551 against D+'s 2,550), and the heat
  weights differ from `i23d/rig/meshy_char.glb` by up to 0.18 on 317 vertices; with the rounded values they differ by 0.
- `export.py`: the `EXPORT_SETTINGS` dict of `A-B1-8` (`:259-262`), verbatim.

Any other number in a Blender script is a parameter, and a review question (B7) checks it.

## 3. Command line

`python3 tools/character_pipeline/cast.py <subcommand>`

| Subcommand | Does | Needs |
| --- | --- | --- |
| `gate <asset.glb> --fit <fit.yaml> [--json]` | G-1…G-6, G-14 on one export; exit 1 on any FAIL | standard library |
| `check-characters [--root <repo>]` | for every `presentation/*/3D/characters/cast/*/`: validates `fit.yaml` and `generation_log.yaml`, and where `character.yaml` exists runs `gate` on its asset; prints one line per directory and the count; exit 1 on any failure. A directory with records only prints `records only (no character.yaml)`. A directory with no `fit.yaml` that holds a casting `brief.yaml` (stage 1, before any generation) prints `brief only (no fit.yaml yet)` and passes; one with neither fails | standard library |
| `run --fit <fit.yaml> --download <dir> [--scratch <dir>] [--from <stage>] [--to <stage>] [--run-id <id>]` | stages 6–10 (and 12's frames) into `<scratch>/<character>/<run-id>/`; writes `run_manifest.json` | Blender 5.2.2 LTS |
| `compare <candidate.glb> <accepted.glb> [--json]` | the parity comparison of §7; exit 1 when outside a bound | standard library |
| `compare-texture <candidate.png> <accepted.png> [--alpha-iou]` | texel comparison of §7, or with `--alpha-iou` the mask comparison | tool venv |
| `mask --fit <fit.yaml> --reference <png> --out <png>` | stage 3 | tool venv |
| `meshy --fit <fit.yaml> --input <png> --scratch <dir> --cap <credits> [--dry-run]` | stage 4; `--dry-run` prints the request without image bytes and the credit projection, and calls nothing | standard library; the key in the environment for a live run |
| `identifiers <file> …` | AI-identifier inspection (port of `inspect_ids.py`) | standard library |

**Scratch resolution:** `--scratch`, else the environment variable `MINEWORLD_CAST_SCRATCH`, else refusal naming both.
A scratch path inside the repository (the repository top level from `git rev-parse --show-toplevel` is a prefix of the
resolved path) is refused.

**Blender resolution:** `MINEWORLD_BLENDER`, else `blender` on `PATH`, else `/Applications/Blender.app/Contents/MacOS/Blender`
on macOS. The first line of `<blender> --version` must be exactly `Blender 5.2.2 LTS` (`BLENDER_PIN` in `standard.py`).
Otherwise the run is refused, unless `--allow-blender "<first line>"` is given. That flag is recorded in
`run_manifest.json`, and the run becomes a **new parity baseline**, never evidence of reproduction.

## 4. `fit.yaml`

`fit.yaml` is written in the JSON-compatible subset of YAML and parsed with `json`, like `ARC-46`'s files. Unknown keys
are refused naming the key path (e.g. `fit.yaml: weights.pack_panels.x_out: expected a number`). Every number is a JSON
number. `_why` strings are allowed where this section shows them.

### 4.1 Schema and defaults

The defaults are Vera's values. An **override** is any value that differs from its default. `check-characters` prints the
override count. More than three overrides is a finding (step-26 §5.3). Hand-supplied landmark picks and the per-character
`source` block are not overrides.

| Key | Type and range | Default (Vera) | D+ origin |
| --- | --- | --- | --- |
| `format` | `1` | 1 | — |
| `character` | slug, lower-case, `[a-z0-9-]+` | — (required) | — |
| `stature_m` | number, metres, > 0 | 1.767 | the accepted record (G-6, G-4); the realistic range is step-26 §7.2's, applied by S26-cast |
| `accessories` | list of `"backpack"` or empty | `["backpack"]` | `rig_meshy.py` pack-core path |
| `source.meshy_task` | UUID form | `01a1155c-dafe-70bc-9279-9cef9a35856b` | D+ §3 |
| `source.download_file` | file name | `model_glb.glb` | `meshy_i23d.py:83-94` |
| `source.download_sha256` | 64 hex | `3de801cdb7dcf34d21ee45758982319096bd62f3a111ab354c46fb2eeb586ef7` | the download's bytes |
| `source.input_sha256` | 64 hex | `fc749a474dcb5b7c1ee1b45e601b2bb0aa8c38e6016811d67c5813d0a68b05fd` | the input image's bytes |
| `source.reference.path`, `.sha256`, `.generator` | repository path, 64 hex, string | `presentation/mineworld-default/3D/references/04_character_closeup.png`; `5b0c8f0b818a9ca7555515e7374f2ec40770a09bc990f8c568b6dc913e41a3a1` | `references/PROVENANCE.md §1` |
| `mask.session` | `"birefnet-portrait"` | same | `prep_inputs.py` |
| `mask.crop_box` | `[x0, y0, x1, y1]` integers, or absent for automatic | `[230, 40, 860, 1086]` | `prep_inputs.py:42-47` |
| `mask.fg_ratio` | number, 0 < x ≤ 1 | 0.85 | `prep_inputs.py` |
| `mask.size_px` | integer | 1024 | `prep_inputs.py` |
| `mask.alpha_threshold` | integer 0..255 | 16 | `prep_inputs.py` |
| `gameready.target_triangles` | integer | 95000 | D+ §5.2 |
| `gameready.eye_z_raw`, `centre_band_z_raw`, `centre_band_half_raw` | numbers | 0.779, 0.05, 0.03 | `gameready_mesh.py` |
| `gameready.sole_z_m` | number | −0.0177 | `gameready_mesh.py` |
| `gameready.eye_from_sole_m` | number | 1.655 | `gameready_mesh.py` |
| `gameready.weld_m` | number ≥ 0 | 1e-06 | `gameready_mesh.py` |
| `gameready.bake.size_px`, `samples`, `margin_px` | integers | 2048, 1, 8 | `gameready_mesh.py` |
| `gameready.bake.extrusion_m`, `ray_m` | numbers | 0.015, 0.04 | `gameready_mesh.py` |
| `gameready.bake.device` | `"auto"`, `"metal"`, `"cuda"`, `"optix"`, `"hip"` or `"cpu"` | `"auto"` | §5.13 |
| `fit.rig_source` | repository path | `clients/3d-spike/assets/characters/vitruvian/vitruvian.glb` | `rig_meshy.py` |
| `fit.charmorph_body` | repository path to the CharMorph body export whose mesh `cm_vitruvian` gives the front surface at the hips and the head top (F-B1-B10-2) | `tools/character_pipeline/rig_source/body_g5.glb` (CC0; `rig_source/PROVENANCE.txt`) | D+ `rig_meshy.py:99-100` |
| `fit.charmorph_landmarks_z` | list of 7 numbers | `[-0.0177, 0.083, 0.51, 0.947, 1.401, 1.531, 1.637]` | `rig_meshy.py` |
| `fit.picks` | object: `frame` (`"dplus-v1"`), `view_px` (1024), `px_per_raw_unit` (512), and the 2D picks `r_shoulder` … `eye` as `[x, y]` integers | Vera's picks | D+ §8.2 |
| `weights.head_rigid.above_chin_m`, `half_width_m` | numbers | 0.01, 0.16 | `reweight` head block |
| `weights.pack_core.behind_spine1_m`, `above_hips_m`, `above_spine2_m`, `half_width_m` | numbers | 0.075, 0.02, 0.10, 0.12 | `rig_meshy.py` pack core |
| `weights.smooth.factor`, `repeat`, `limit` | number, integers | 0.5, 4, 4 | `rig_meshy.py` |
| `weights.fingers.hand_share_min`, `power`, `keep` | numbers | 0.95, 6, 2 | `reweight_fingers.py` |
| `weights.pack_panels.x_in`, `x_out`, `sleeve_share_max` | numbers | 0.12, 0.21, 0.5 | `reweight_pack.py` |
| `weights.pack_panels.arm_zero`, `smooth` | booleans | true, true | `reweight_pack.py` |
| `weights.pack_panels.smooth_repeat` | integer | 8 | `reweight_pack.py` |
| `flecks.enabled` | boolean | **true** (default for new characters, Q-B1-5); Vera sets `false` | D+ §8.4 step 8: the reviewed bake was committed without the repair (F-B1-4) |
| `flecks.z_min_m`, `dark`, `lighter` | numbers | 1.65, 0.30, 0.12 | `fix_flecks.py` |
| `export.material` | string | `"MW_Cast"` | step-26 §4.5 |

Vera's `fit.yaml` therefore has exactly one override, `flecks.enabled: false`, and `check-characters` reports
`["flecks.enabled"]` for her.

### 4.2 Per-character method constants are not overrides

The keys `character`, `stature_m`, `accessories`, every key under `source`, and the `fit.picks` object describe the
character, and they are not counted (`OVERRIDE_EXEMPT` in `records.py`). Changing a method constant in §2.4 is a
code change under `ARC-94`, not a `fit.yaml` edit.

## 5. `generation_log.yaml`

One entry per Meshy run:

| Key | Type | Meaning |
| --- | --- | --- |
| `run` | string | run label |
| `task_id` | string | the Meshy task id; `task_id_complete: false` where D+ kept only a prefix |
| `params` | object | `remesh` and `raw` as D+ §3 recorded them |
| `credits` | integer | credits the run consumed |
| `verdict` | `rejected` or `kept` | the raw gate verdict (D+ §3) |
| `reason` | string | D+ §3's sentence |
| `date` | ISO date | the day of the verdict |

Vera's log has three runs (`01a11555-…`, `01a11559-…`, `01a1155c-dafe-70bc-9279-9cef9a35856b`; the first two marked
`task_id_complete: false`), 30 credits each, verdicts `rejected`, `rejected`, `kept`, dated 2026-10-07. Totals: 3 runs,
90 credits, 1 kept.

## 6. Gates G-1 … G-6 and G-14

Each gate returns `GateResult(gate, verdict, looked_at, reasons)`, with `verdict ∈ {PASS, FAIL}`. A gate that cannot look
reports FAIL with the reason (for example G-4(b) without a run directory). No gate takes a bound as an argument.

| Gate | Check | Bound (`standard.py`) and source |
| --- | --- | --- |
| G-1 | triangle count of the one skinned mesh ≤ `MAX_TRIANGLES`; exactly one skinned mesh | `100_000` (D+ §5.2) |
| G-2 | exactly one material, named `MATERIAL`; it has a `baseColorTexture` and no normal, occlusion, emissive or metallic-roughness texture; one primitive | `"MW_Cast"` (step-26 §4.5, G-2) |
| G-3 | base-colour image `TEXTURE_PX × TEXTURE_PX`, 8-bit RGB or RGBA; when a Godot `.import` beside it exists, `mipmaps/generate=true`; the compression mode is printed, not judged (Q-S26-5 is S26-f's) | `2048` (D+ §5.2) |
| G-4 | (a) soles: min glTF `y` within `SOLE_TOL_M` of `SOLE_Z_M`; (b) sole-to-eye: the eye height (the `eye` pick through stage 6's transform, `gameready.json`) minus the sole, within `EYE_TOL_M` of `stature_m × EYE_RATIO` | `-0.0177`, `0.005`, `0.9366`, `0.010` (step-26 G-4) |
| G-5 | the 52 joint names and their parent relation equal the reference skeleton read from `vitruvian.glb`; one `JOINTS_0`/`WEIGHTS_0` set and no `_1`; each vertex's weight sum within `WEIGHT_SUM_TOL` of 1; 0 vertices below `UNWEIGHTED_SUM`. Rolls are checked from `dump_rig.json` when a run directory is given, within `ROLL_TOL_RAD` of the reference | `1e-3`, `1e-4`, `1e-4` |
| G-6 | (a) Winter: upper arm, forearm, thigh and shank on both sides, each `|joint→joint| / stature_m` within ±`WINTER_TOL` of Winter's ratio; (b) for the 18 trunk and limb joints (Hips, Spine, Spine1, Spine2, Neck, Head, and per side Arm, ForeArm, Hand, UpLeg, Leg, Foot) the joint lies inside the body: on a ray cast along the depth axis through it (the mesh vertices in a slab of ±`SLAB_HALF_M` at the joint's height whose x is within `DEPTH_RAY_HALF_M` of the joint's), the joint lies between the back-most and the front-most vertex, margin ≥ 0; a ray that meets no surface fails. Nothing is clipped by distance (R-B1-G6, 2026-10-14: the B3 rule clipped to 150 mm horizontally and read joints inside a fused body-and-pack shell as outside, F-B1-B3-1). Margins are printed per joint | `{upper_arm 0.186, forearm 0.146, thigh 0.245, shank 0.246}`, `0.12` (Winter, step-26 G-6); `0.010`; `0.03` (D+'s front-surface probe, `rig_meshy.py:57`) |
| G-14 | `scenes[0].extras.mineworld_provenance` has `source_type == "generated"`, `ai_generated` true, a non-empty `generator`, `meshy_task` in UUID form, `references` (each `{path, sha256}` with 64 hex, matching the file's SHA-256 when the path exists in the repository), `reference_generator`, `postprocess`, `pipeline` (`{version, fit_sha256, blender}`); no string in the provenance matches the key patterns of `scripts/check_secrets.py` | `ARC-9`, `DEP-8`, D+ §4 |

**Winter's figures.** Source: D. A. Winter, *Biomechanics and Motor Control of Human Movement*, 4th ed., Wiley 2009,
Fig. 4.1 (segment lengths as a fraction of height H). **Status in B1 (2026-10-14): INCONCLUSIVE.** The edition's figure
could not be read from the fetcher (the open pages checked returned 403 or nothing usable). The numbers are taken from
step-26 G-6 as the frozen gate, and they are not contradicted by anything read. The primary session re-checks against the
book before S26-b2 relies on G-6(a).

**Runtime gates.** G-7…G-13 (A_TPose, stride, sweep, headtrace, sit) belong to S26-b2 (step-26 §12). Until then a
character's runtime gates are recorded "by parity" with the operator-accepted asset when the parity comparison passes,
and never as PASS.

## 7. Parity comparison (`compare`, `compare-texture`)

Bounds are fixed before any run and are literals in `standard.py`:

| Quantity | Rule | Bound |
| --- | --- | --- |
| Triangles, vertex count, joint count | equal | — |
| Joint names | equal sets | — |
| Joint world position (from the inverse bind matrices), per joint | Euclidean distance | `PARITY_JOINT_M = 0.001` |
| Vertex position, by index | max absolute component difference | `PARITY_POSITION_M = 1e-05` |
| UV, by index | max absolute difference | `PARITY_UV = 1e-06` |
| Weights, by vertex index and joint name | max absolute difference over every (vertex, joint) | `PARITY_WEIGHT = 0.005` |
| Material name | reported, not compared | — |
| Provenance | reported, not compared | — |
| Base colour texels (`compare-texture`, same bake device only) | share of texels whose every channel differs by ≤ 2/255 | `PARITY_TEXEL_SHARE = 0.999` |

`compare` prints the counts it compared and the worst joint, vertex and weight with their indices, before its verdict.
If vertex counts agree but positions disagree at index 0, the verdict is FAIL with the message `vertex order differs`.
The comparison never re-matches vertices by search.

`compare-texture --alpha-iou` compares masks: alpha IoU at alpha > 127, and RGB mean absolute difference over the union
foreground (the stage 3 acceptance, AB1-13).

## 8. Provenance (stage 10)

Stage 10 writes this block into `scenes[0].extras`:

```json
"mineworld_provenance": {
  "source_type": "generated", "ai_generated": true,
  "generator": "Meshy image-to-3D, ai_model <from request.json>, pose_mode <from request.json>, paid plan",
  "meshy_task": "<from fit.source.meshy_task>",
  "references": [ { "path": "<fit.source.reference.path>", "sha256": "<…>" } ],
  "reference_generator": "<fit.source.reference.generator>",
  "input_sha256": "<fit.source.input_sha256>",
  "postprocess": "scaled to stature; welded; decimated to <n> triangles; base colour re-baked; fitted on the CharMorph 52-joint skeleton; heat weights with head, pack and finger fixes",
  "pipeline": { "version": "mwcast-1", "fit_sha256": "<sha256 of fit.yaml bytes>", "blender": "Blender 5.2.2 LTS" }
}
```

The Meshy model and pose mode come from the download's `request.json` (`params.ai_model`, `params.pose_mode`), read from
the scratch run directory. The export never copies `task.json`. Meshy's identifiers are kept: Terms §2.4 forbids
removing them, so the pipeline never strips a metadata chunk from the download.

## 9. The export format: stored once

`export.py` writes two outputs from the same `.blend`:

1. `<id>.glb`: embedded PNG, the D+ export settings, for the parity comparison with `meshy_d.glb`.
2. `<id>.gltf` with `<id>.bin` and `<id>_base_color.png`: Blender's `GLTF_SEPARATE`. The glTF exporter names a separate
   image after the basename of the image's file path, not after the datablock, so `export.py` packs the base-colour
   image (the packed bytes are written unchanged) and sets its path to `//<id>_base_color.png` before this export. The
   driver checks the result after stage 10 (`check_stored_once`): the `.gltf`'s only image URI is `<id>_base_color.png`
   and the file exists; otherwise the run stops.

**Decision (recorded in B10):** the decision rule is fixed here. Godot 4.7.2 headless, in a scratch project under
`S/godot-check/` (never in `clients/3d-spike`), imports the three files, `vitruvian_bonemap.tres`,
`godot/character.gltf.import` (renamed `<id>.gltf.import`) and `godot/inspect_import.gd`. The script prints the material's
albedo texture `resource_path`, the surface count, the bone count with the first three bone names, and the project files
with `base_color` in their name.

- **GLTF_SEPARATE is the stored-once format** if: the albedo path is `res://<id>_base_color.png`; one surface; 52 bones
  with `Hips` present; and no project file other than `<id>_base_color.png` and its `.import` holds the base colour.
- Otherwise the result is recorded and the question returns to the primary session. The fallback (GLB with
  `gltf/embedded_image_handling=3`) moves the texture's import policy out of a per-PNG `.import`, which touches Q-S26-5, so
  it is not chosen autonomously.

**§14 (stored-once outcome):** decided in B10 (run `r3`, 2026-10-14) and recorded in `rederive_report.json`.
**GLTF_SEPARATE is the stored-once format.** Godot 4.7.2 printed `surfaces=1`, `material=MW_Cast`,
`albedo=res://vera-lindgren_base_color.png`, `bones=52 first=["Hips", "LeftUpperLeg", "LeftLowerLeg"]`, and the only
project files holding the base colour were `res://vera-lindgren_base_color.png` and its `.import`. (The first run, `r1`,
before the export named the image's path, printed `albedo=res://base_color.png`.)

## 10. Scratch, and what is never committed

- Per run, the unmodified Meshy download, `request.json` and `task.json` live in the scratch root and are never committed
  or modified (I-B1-4, step-26 §5.1). Scratch roots are outside the repository (I-B1-5).
- Never committed: GLB, PNG, `.blend`, `.bin`, Meshy downloads, `request.json`, `task.json`, Blender or Godot outputs.
  The only committed records for a character are `fit.yaml`, `generation_log.yaml` and the re-derivation report.
- Committed binaries for the protagonist are the operator-accepted assets already in the repository
  (`clients/3d-spike/assets/characters/meshy_d/`). This PR changes none of them (I-B1-7).

## 11. Blender

- Pin: `Blender 5.2.2 LTS` (`BLENDER_PIN`). Checked at each run start (§3).
- Every stage runs with `--factory-startup`.
- The bake device is `auto`: Metal, then CUDA/OptiX/HIP, then CPU. The device actually used is recorded in
  `gameready.json` and in the run manifest. The texture parity bound (§7) applies within one device only.
- Scripts are GPL-2.0-or-later, in `blender/`, with `LICENSE` byte-identical to `clients/3d-spike/tools/blender/LICENSE`.
  The driver never imports them; it only runs them as subprocesses (I-B1-8).
- A change under `blender/` re-runs the local real-lifecycle Gate (B10 procedure) on macOS. This is a standing review
  question for every PR that edits `blender/`.

## 12. Platforms

The driver and the gates are standard-library Python 3.12 (`pathlib`, subprocess argument lists, binary reads for
GLB and PNG, UTF-8 reads for records). They are tested on Linux, macOS and Windows in CI. The Blender stages run
wherever Blender 5.2.2 runs. The local Gate of this PR runs on macOS. A Linux or Windows re-derivation is a new parity
baseline, recorded with its device and `--allow-blender`, and is not part of this PR's acceptance.

Mask and texel comparison use the tool venv (`tools/character_pipeline/requirements.txt`), which CI never installs.

## 13. Origin of the code

Every ported D+ script, with its SHA-256 at B1 (`shasum -a 256`, in `/Users/yuema137/mineworld-demos/i23d/`):

| D+ script | Lines | SHA-256 | Port |
| --- | --- | --- | --- |
| `meshy_i23d.py` | 101 | `c20a978009f9f9796aea72ea21b27638b12c4d2556cc44fd5369c037909d393f` | `mwcast/meshy.py` |
| `prep_inputs.py` | 64 | `dbe8df01745b027fdd276b86175e6d3141fb70060f53e29f9ca07a153f02a9c0` | `mwcast/mask.py` |
| `gameready_mesh.py` | 157 | `24ae6453d8a447762679b351532fca0492ed2739cfcd3a0df342963e1b4ccb18` | `blender/gameready.py` |
| `rig_meshy.py` | 264 | `fd1756f69ade4c1d30ecda1efb1af287b1efa0153db5f13022069fa2b84111e2` | `blender/fit_bind.py` |
| `reweight_fingers.py` | 75 | `fdd0e08923dc61d7d809d3fc50a5039bd2ffc8234fd7aa9118ad447cc0ceed2f` | `blender/weights_fingers.py` |
| `reweight_pack.py` | 91 | `6b3f3a03b6aea3dcbcec403c30e3661dd643d074993cb8e12ce8a55e30b1ad20` | `blender/weights_pack.py` |
| `fix_flecks.py` | 107 | `f49c82bb97701b6ed82143bbd77539b92df383970eec02056a13a318f9b9311c` | `blender/flecks.py` |
| `render_ortho.py` | 63 | `27b4b08410a64766dc5e113499b384af12ed7d9d61b38a8573917310f1be2352` | `blender/render_ortho.py` |
| `render_rig.py` | 74 | `6dc3bef3910ebdae458b506f904cf8226766b6abcc6d3f31f40ed53689106c29` | `blender/render_bones.py` |
| `dump_bones.py` | 25 | `4bf2076521b8bcef607ae444a51051d7bea1038d892a856b1354f91cf66f14f3` | `blender/dump_rig.py` |
| `inspect_ids.py` | 55 | `849bf6d306e622ccc27c9e71808317e9e2a83730dbc9f632b7888f31fcab1623` | `mwcast/identifiers.py` |
| `rebake_head.py` | 143 | `355aecffb6f2a9901490c9f57137d12d4d85f9b572532de6e2029c895e4dfa84` | **dropped** (§2.1) |

## 14. Fixed numbers in this PR

Vera's download (`out/meshy/apose_v3/model_glb.glb`) has SHA-256 `3de801cdb7dcf34d21ee45758982319096bd62f3a111ab354c46fb2eeb586ef7`,
which matches `fit.source.download_sha256`. Her `request.json` has SHA-256 `c002a274d7ea6251df5eaabd8daf5902b2585a343f3b509a0370af8893573bbf`
and her `task.json` `6583a7698d517d293214cba7315a1b15a938d4a048cb433da850dde57b2703b5`.
