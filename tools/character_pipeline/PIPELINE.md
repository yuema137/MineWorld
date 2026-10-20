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
| `check-characters [--root <repo>]` | for every `presentation/*/3D/characters/cast/*/`: validates `fit.yaml` and `generation_log.yaml`, and where `character.yaml` exists runs `gate` on its asset; prints one line per directory and the count; exit 1 on any failure. A directory with records only prints `records only (no character.yaml)`. A directory with no `fit.yaml` that holds a casting `brief.yaml` (stage 1, before any generation) passes, printing `brief only (no fit.yaml yet)`, only when the brief is valid (`mwcast/brief.py`: `person`, `from`, `casting`, `sources`; `person` equal to the directory; `casting.world`, `profile`, `template_version` (an integer), `seed`, `attempt`; every `sources` key a top-level key) and no generation record (`generation_log.yaml`, `character.yaml`, `rederive_report.json`) is present; a directory with neither `fit.yaml` nor `brief.yaml` fails | standard library |
| `run --fit <fit.yaml> --download <dir> [--scratch <dir>] [--from <stage>] [--to <stage>] [--run-id <id>]` | stages 6–10 (and 12's frames) into `<scratch>/<character>/<run-id>/`; writes `run_manifest.json` | Blender 5.2.2 LTS |
| `compare <candidate.glb> <accepted.glb> [--json]` | the parity comparison of §7; exit 1 when outside a bound | standard library |
| `compare-texture <candidate.png> <accepted.png> [--alpha-iou]` | texel comparison of §7, or with `--alpha-iou` the mask comparison | tool venv |
| `mask --fit <fit.yaml> --reference <png> --out <png>` | stage 3 | tool venv |
| `meshy --fit <fit.yaml> --input <png> --scratch <dir> --cap <credits> [--dry-run]` | stage 4; `--dry-run` prints the request without image bytes and the credit projection, and calls nothing | standard library; the key in the environment for a live run |
| `identifiers <file> …` | AI-identifier inspection (port of `inspect_ids.py`) | standard library |
| `pose-sheet <asset.glb or .gltf> --out <dir> [--character-yaml <path>] [--label <name>] [--grip] [--project <dir>] [--plant-seat-gap <m>]` | the step-26 §6.4 pose sheet (§15): frames, `report.json`, `sheet.html`; exit 1 when any frame is flagged | Godot 4.7, windowed |
| `contact <asset.glb> [--outfit <outfit_manifest.json>] [--tolerances <pack file>]` | C-1 on a body at rest; with `--outfit`, C-2 on an assembled outfit (§18); exit 1 on any FAIL | the root uv environment (numpy, libigl, trimesh, python-fcl) |
| `contact-items [--root <model library>] [--tolerances <pack file>]` | C-3 and C-4 over every prop of the library (§18); exit 1 on a failure that is not a recorded finding | the root uv environment |

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
| `fit.depth` | `"dplus"` or `"midspan"` (optional) | `"dplus"` | S26-c, F-S26c-2 (§17) |
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

Exception (S36-P1): the contact checks (§18) need numpy, libigl, trimesh and python-fcl. They are pinned in the root
dev dependency group, which CI installs (`uv sync --locked`), and in `requirements.txt` for the tool venv. Every other
subcommand stays standard-library: `cli.py` imports the contact gates only inside `contact` and `contact-items`.

## 13. Origin of the code

Every ported D+ script, with its SHA-256 at B1 (`shasum -a 256`, in the operator's local D+ workspace, outside the repository):

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

## 15. The pose sheet (`pose-sheet`)

Step-26 §6.4 (operator, 2026-10-15) asks for every finished character in several poses, seated on a chair included,
rendered by a scripted, deterministic probe, with clipping flagged red and never hidden. `pose-sheet` is that probe. It
is tooling under §1: it reads one export and writes outside the repository.

**Command.** One per character, holding one machine slot for both Godot runs (R-LOAD-1):

```text
with-slot.sh python3 tools/character_pipeline/cast.py pose-sheet <export.glb|export.gltf> --out <dir> \
    [--character-yaml <character.yaml>] [--label <name>]
```

`--grip` holds the protagonist's strap grip (solved for her body only, `human.gd` `GRIP_*_D`); a cast character is
rendered with `Posture.natural_stance()`. `--plant-seat-gap <m>` raises the seated body by `m` metres after it settles
(acceptance X-26); `pose_sheet.gd` also takes `--plant-arm-into-torso` (the right arm forced through the trunk) and
`--debug`. Godot is `MINEWORLD_GODOT`, else `godot` on `PATH`.

**What runs.** `mwcast/pose_sheet.py` builds a scratch Godot project (default `<out>/.project`), never touching
`clients/3d-spike`: it copies the client's `human.gd`, `posture.gd`, the hair shader, the protagonist (`meshy_d`), the
Quaternius clip library and its bone map, the CharMorph bone map and textures, the café's `dining_chair_02` and
`round_wooden_table_02` (`cafe_interior.gd` `_loose_seating`), and the slice's HDRI; it copies the export to
`cast/subject.<ext>` with the protagonist's import settings (the same bone map and rest fixer), the skeleton path taken
from the export's armature node. It runs `godot --headless --import` (log: `<out>/import.log`), then
`godot --fixed-fps 30 --disable-vsync --script res://pose_sheet.gd` windowed (a headless Godot renders nothing). The
frames render off-screen in a 1000 × 1250 `SubViewport` (MSAA 4×, no TAA), so the window's size and the display do not
change them.

**The body.** The subject is built by `Human.build` on the `REFERENCE` slot (scale 1: an export is at its own stature),
so the clips, the `Stand` loop, the `Sit` clip and the retarget are the client's. A `MW_Cast` surface gets the same
surface response `human.gd` gives `MW_Meshy`. Blinking is held open and the reference body's head low-pass is set to 0
(its converged value at a steady gait), because `posture.gd` times that filter with the wall clock.

**The light.** The slice's: the `qwantani_puresky_2k` sky at 0.80, AgX at exposure 1.06, SSAO, the warm sun at
elevation −19.3°, azimuth −48°, energy 4.4, and the cool fill (`slice_main.gd`). The subject faces 25° round from the
sun's azimuth. The floor is a neutral grey plane.

**Frames** (`frames/<id>.png`):

| Id | Step-26 row | Pose and framing |
| --- | --- | --- |
| `P-1_front`, `P-1_three-quarter`, `P-1_rear` | P-1 | the `Stand` loop 1 s in; full body from 0°, 35°, 180°, fov 30°, distance from the skinned stature so nothing crops |
| `P-8_face` | P-8 | the same stand; the camera 2.5 m from the eyes, 8° off the front, fov 13.7° |
| `P-1_lineup-with-vera` | P-1, §6.3 | the subject and the protagonist side by side, front, same light (the quality-tier control) |
| `P-6_two-shot-with-vera` | P-6 | facing the protagonist at 1.1 m, three-quarter on the subject; idle (no gesture clip exists yet) |
| `P-2_walk-contact`, `P-2_walk-passing` | P-2 | `Walk` at its clip speed on a treadmill; side; the frames of largest and smallest foot separation in one cycle, taken one cycle later |
| `P-3_jog-contact` | P-3 | `Jog_Fwd`, likewise, the contact frame |
| `P-4_seated-three-quarter`, `P-4_seated-side` | P-4, G-13 | on the café chair at its table, hands on the table; three-quarter and side |

**The seated pose.** The `Sit` clip's trunk reclines, its feet hang, and its hands lie by the thighs, so the sheet
layers a solved `Posture` (rest-relative, `posture.gd`'s convention) after it: the pelvis turned so the trunk leans 10°
forward of vertical; each leg so its ankle rests on the floor below the knee, the thigh about level; each arm so the wrist
lies on the table top 5 cm in from its near edge. The solves are grid searches with coordinate descent, so they are
deterministic. The table's pedestal would take the shins at the café's 0.66 m, so the chair stands until the base is
0.58 m ahead of the seat's middle (0.81 m for this table). The seat top (0.454 m) and the table top (0.746 m) are
measured from the assets' vertices. The body is then settled on its own seat surface: from the protagonist's
hips-over-seat height, three corrections move the hips until the lowest buttock-or-thigh vertex over the seat touches it,
the pose re-solved each time.

**Checks** (per frame, on the subject only, from CPU-skinned vertices: Σ w · pose · bind, with the poses read in
`Skeleton3D.skeleton_updated` so the SkeletonModifier3D results are included):

- **Torso clip (G-9, simplified).** The trunk volume is a stack of 2 cm slabs along the Hips→Neck axis, from the Spine
  joint to 5 cm below the Neck; in each slab, the outline of the pure trunk vertices (dominant bone Spine, Chest or
  UpperChest, limb weight share ≤ 0.05; a fused pack included) as the outermost radius from the slab's centroid in 48
  directions. A hand, forearm (sleeve), distal-half upper-arm, distal thigh or lower-leg vertex (dominant bone of that
  limb, trunk share ≤ 0.05, and within 0.08–0.12 m of the limb's bone segment, so it lies on the limb) that is inside
  the outline by more than **5 mm** fails. The share and segment filters exist because a fused generated mesh blends
  trunk and arm where they meet: without them the protagonist's hanging arm read 60 mm "inside" her hoodie.
- **Seat contact (G-13), P-4 only.** After settling, the lowest buttock-or-thigh vertex over the seat panel (from 10 cm
  behind the Hips joint forward, so a pack against the chair's back is not counted) must be within **±20 mm** of the
  seat top, and the body's settled hips-over-seat height within 20 mm of the protagonist's 0.250 m
  (`protagonist_rule_error_mm`: how far it would float or sink where one shared rule seats every body).
- `hands_above_table_mm` is reported, not judged.

A failing frame is still written, with a 14 px red border, and listed first-class in the sheet. `report.json` holds
every frame's checks, the Godot version and the GPU, and notes (the clip periods and chosen frames, the furniture's
measured heights, the solves' residuals, every plant).

**Determinism (X-27).** Every pose is reached by counting frames under `--fixed-fps`; nothing reads the wall clock.
Two runs at the same commit on the same machine (Apple M5, Metal, Godot 4.7.2) wrote byte-identical `report.json` and
byte-identical P-1, P-2, P-3, P-6 and P-8 frames. The two P-4 frames differed in 13 and 24 pixels by 1/255 each, in
small patches; the poses and the checks were identical (waiting 45 frames after the chair and the table appear, for
their pipelines to compile, did not remove it). Another GPU or driver renders different bytes.

**The window must stay drawable.** The sheet waits for each drawn frame; on macOS a Godot window that is hidden,
minimised or on another desktop stops drawing, and two concurrent runs once stalled that way. The render runs
`--always-on-top`, and the driver stops Godot after 600 s (import: 900 s) with that explanation.

**Validated (2026-10-15).** The protagonist (`meshy_d.glb`, `--grip`) and the pipeline's re-derivation of her
(`vera-lindgren.gltf`, run `r3`): 11 frames, none flagged. The CharMorph town body (`vitruvian.glb`) renders, and is
flagged: its garments are separate layered meshes (a tee sleeve under a hoodie sleeve is "inside" the outer layer), so
the torso check does not apply to it, and its settled hips sit 55 mm under the protagonist's. Cast exports are one fused
mesh (G-1), which is what the check is for. Plants: `--plant-arm-into-torso` flagged every frame (210 mm);
`--plant-seat-gap 0.05` flagged both P-4 frames (+50.0 mm).

**Known limits.** P-5 (bench or stool) and P-7 (reaching over the counter) are not rendered. The two-shot has no
gesture. The pose sheet never judges identity or quality; the scorecard (step-26 §6.2) does, beside the protagonist.
The tool does not check the pack against the chair's back.

**Finding for the client (not changed here).** `human.gd` `sit(seat_y)` places the `Hips` joint itself at `seat_y`. In
its own `Sit` pose that leaves the protagonist's lowest buttock vertex 127 mm under the seat surface; in the sheet's
upright table pose the body rests on the seat with the `Hips` joint 0.250 m above it.

## 16. Route B and the garment pipeline (S32-p)

Status: S32-p, 2026-10-18. Authority: `.structured-coding/plans/mvp0/pr-s32p-wardrobe-pilot.md` (frozen design, live
ledger §16), `step-32-wardrobe.md` §§5.2, 6. Records: `DEP-72` (MPFB2 as a hidden fitting tool), `DEP-74`.

**What a modular character is.** A body (one skinned mesh on the 52 joints, from a Meshy base-layer download through
stages 6–10), a hidden fitting template `T_P` (the 19,158 `hm08` vertices registered to that body), and parts (garments
and hair), each one neutral-rest mesh plus a binding that places it on any `T_P` (MakeHuman's `.mhclo` formula).

**Files** (all in scratch; nothing generated is committed, §10):

| File | Writer | Content |
| --- | --- | --- |
| `recipes/s32p-pilot.yaml` (committed) | the session | subjects, identity references and picks with SHA-256, the library (15 garments + 3 hair styles), outfits A and B, the budget, the fixtures; parsed by `mwcast/wardrobe/recipe.py`, unknown keys refused by key path |
| `wardrobe/template/hm08_regions.json` (committed) | `cast.py wardrobe regions` (R-5) | the 11-region partition of `hm08` and its rule |
| `<subject>_template.bin`, `_template_weights.bin`, `_corr.bin`, `register.json` | `wardrobe register` (R-4) | `T_P` in the body mesh's frame; its skin weights (joint table in the body's glTF skin order, so an index is a bind index); per body vertex the closest template triangle; per-region distances |
| `garment.glb`, `binding.bin`, `hides.bin` | `wardrobe bind` (R-10, R-11) | the neutral-rest mesh; MakeClothes' binding at full precision; the hidden set |
| `outfit_<subject>_<A|B>.glb`, `outfit_manifest.json` | `wardrobe assemble` (R-12 a) | the assembled outfit: `Body` (`MW_Cast`, hidden triangles removed, regions in UV2) and `part_<id>` (`MW_Cast_<id>`) |
| `credits.json`, `images.json` | `wardrobe meshy`, `wardrobe image` | the measured-charge ledger (§4.6) and the image-call log (§4.7) |

The binary layouts are `mwcast/wardrobe/garment_format.py`'s docstring. Coordinates are the body mesh's frame: glTF
axes, +Y up, facing +Z, which are MakeHuman's axes at 1/10 scale, so a `.mhclo` binding applies unchanged.

**Commands** (`python3.12 tools/character_pipeline/cast.py wardrobe <sub>`; Blender and Godot only through
`MINEWORLD_CAST_SLOT=<with-slot.sh>`; MPFB2 stages with `BLENDER_USER_RESOURCES=<scratch>/blender-user`):

| Subcommand | Stage | Needs |
| --- | --- | --- |
| `check`, `budget` | recipe validation; the ledger against §4.6 | standard library |
| `image base-layer --subject K` / `image garment --garment G --clay <png>` [`--dry-run`] | R-1 / R-7: `mineworld_gen --model gpt-image-2.5-sunburst`, template v2's preamble verbatim, ≤ 70 calls | `MINEWORLD_GEN_PYTHON` (the tool venv), the OpenAI key |
| `mask body|garment` | stage 3 on the recorded pick | tool venv (rembg) |
| `meshy body|garment` [`--dry-run`] | R-2 / R-8 under the pilot ledger: admitted before the POST (cap, ceilings, retries, pool, IH gate), reconciled after it with the measured charge | the Meshy key; the recorded pick |
| `install-mpfb --zip` | MPFB2 2.0.18 into the isolated user resources (zip SHA-256 checked) | Blender 5.2.2 |
| `regions`, `register --subject K --body <glb>` | R-5, R-4 | Blender + MPFB2 |
| `bind --id G --glb <garment> --template <bin> --covers r,…` | R-10 (MakeClothes) and R-11 | Blender + MPFB2 |
| `fixtures`, `fixture-check` | W-5's fixture bodies and stock CC0 fixture garments; W-1 and the formula oracle against MPFB2's own fits | Blender + MPFB2; `MINEWORLD_MPFB_ASSETS` |
| `assemble --spec <json>` | R-12 (a): reference fit, layer push-out, weights, body mask, outfit GLB | Blender |
| `w9 --template-dir … --garment id=dir` | W-9: `fit_at_load.gd` in Godot against the Python fit | Godot 4.7 |
| `preview --item path=r,g,b,a --out` | flat-colour review renders (judges nothing) | Blender |

**Exception to §11 (`DEP-72` item 4).** The `blender/mpfb/` scripts are GPL-3.0-or-later (they call MPFB2's API) and
run without `--factory-startup`, because the extension must load; nothing else changes.

## 17. Wardrobe gates (S32-p)

Bounds are literals with their sources (I-p-6): `mwcast/wardrobe/wgates.py` (G-1/G-2/G-3/G-14 per outfit, W-2, W-3,
V-1), `garment_format.py` (W-1), `godot/pose_sheet.gd` (W-4's `POKE_MM`), `cli.py` (W-9's 0.5 mm). The pose sheet's
outfit mode (`pose-sheet <outfit.glb> --outfit <manifest> --pair <outfit B>`) matches `MW_Cast_*` for the surface
response, tests limbs only on their outermost layer (F-S32p-6), adds W-4 to every frame, and renders the pair frame
(fused Vera | A | B). The live record of what each gate found is the ledger, §16 of the design.

## 17. The fused cast (S26-c)

S26-c builds every named Person the way Vera was built (route D+): one Meshy image-to-3D run from the operator's v2
casting pick (`cast/PICKS.yaml`), then stages 6–10 and the static gates. Two bounded corrections came from the first
fused cast member (Alice, 2026-10-19). Both leave Vera's re-derivation unchanged.

- **F-S26c-1, G-6(b) by triangles.** The depth ray through a joint now also intersects the mesh's triangles
  (`gates._line_hits`), and the span is taken over those hits and the old vertex window together. Cause: a decimated
  fused body has large flat triangles on the back, so at Alice's Hips the ±10 mm × ±30 mm vertex window held only the
  apron's front vertices and read the joint 90.7 mm "outside"; the line crosses the back triangle 140 mm behind the
  front. The bound is unchanged (margin ≥ 0). A joint that is really outside still fails (test
  `test_g6_a_joint_behind_the_back_triangle_stays_outside`).
- **F-S26c-2, `fit.depth: "midspan"`.** D+ places a limb joint at the mean depth of its slab's vertices. A fused
  apron or skirt puts many vertices in front of one thigh, so Alice's left hip joint landed 55 mm in front of the
  right one. With `midspan`, a slab's depth is the middle of its depth span, and a centre joint (Hips … Head) left
  outside its section is moved 5 mm inside it, with every joint above it moved by the same amount so the head keeps
  its rest pitch (moving the Neck alone tipped Alice's head back in the pose sheet). Under `midspan` the centre
  points (Hips … Head) also keep CharMorph's fraction of the section's depth (front 0, back 1) instead of D+'s fixed
  offset behind the front surface, and both sections are read by the depth ray (Blender `ray_cast` from the front
  and the back), as G-6(b) reads them: a belly (Otto) or an apron moves the front surface, not the spine, and an
  open collar beside the centre line is not the neck's front (Hana). `dplus` stays the default, so
  Vera and every existing record are unchanged; `write_fit` sets `midspan` for the fused cast, and it is counted as
  one override.

The paid runs go through `cast.py wardrobe meshy body` under `recipes/s26c-cast.yaml`, a recipe with
`"route": "fused"`: its subjects are the 21 picks, it has no garments or outfits, and its own ledger is capped at
1,800 credits (S26's 3,000 less the S32-p pilot's 1,200). The base-layer slot of a fused subject holds its clothed
casting pick.


## 18. Contact checks (S36-P1)

Design: `.structured-coding/plans/mvp1/step-36-interpenetration.md` §§3–4.1 and `pr-s36-p1-contact-module.md`.
Records: `DEP-142` (libigl), `DEP-143` (trimesh + python-fcl), `DEP-145` (what was rejected).

**Module boundary (I-S36-1).** `mwcast/contact.py` is the only module that imports `igl`, `trimesh` or `fcl`. It
takes numpy arrays in glTF metres and returns typed results with depths in millimetres: `inside` / `outside` (the
generalized winding number against a surface, threshold 0.5, and the point-to-surface distance), `nearest`,
`lowest`, `clearance`, `scene_parts` (a glTF scene's meshes with world transforms) and `overlaps` (trimesh's
`CollisionManager` over python-fcl, the deepest contact per pair). `igl.copyleft` (GPL) is never imported;
`tests/test_contact_licence.py` scans every Python file in the repository for it and for a mesh-library import
outside `contact.py`, and checks `sys.modules` after the module ran. A missing library is `ContactUnavailable`, and
the gate reports FAIL "cannot look" (I-S36-4).

**Tolerances (I-S36-2).** `tools/character_pipeline/contact_tolerances.json` holds every bound: per region key a
`bound_mm`, a `rationale`, a `source` and a `status` (`cited`, `inherited` or `inconclusive`; an inconclusive bound is
still enforced). `mwcast/tolerances.py` refuses an unknown or missing key, a missing field, a non-positive bound and
an unknown status, by name. A pack may pass its own file with `--tolerances`; it may lower a bound and never raise
one (Q-S36-4). Every gate prints the SHA-256 of the tolerances it judged with.

**The gates** (`mwcast/contact_gates.py`):

| Gate | Input | Question | Bound |
| --- | --- | --- | --- |
| C-1 | a fused body GLB, bind pose | Does a hand, forearm, distal upper arm, distal thigh or shin vertex lie inside the trunk patch? The patch is the faces whose three vertices are dominated by Spine, Chest or UpperChest with a limb share ≤ 0.05; the query vertices are chosen by G-9's filters (`limb_queries`: dominant group, trunk share ≤ 0.05, proximal cut-off, segment radius, Spine … Neck − 5 cm), ported from `pose_sheet.gd` so P2 uses the same function on dumped frames. Bones are read by their Godot humanoid names (`humanoid` maps `mixamorig:` names). | `self.limb_in_trunk`, `self.hand_in_trunk` |
| C-2 | an assembled outfit GLB and its `outfit_manifest.json` | In layer order, for each part that covers regions (hair never does): the drawn body's vertices in a covered region, flagged by the assembly as under an outer layer (TEXCOORD_1 `v`), and each lower layer's flagged vertices in a covered region (a part vertex takes its nearest body vertex's region), must lie inside the part or outside it by at most the bound. The body's hidden set is already removed by the mask. A check that tests no vertex fails. | `garment.poke.body`, `garment.poke.layer` |
| C-3 | every `<slug>/<slug>.gltf` under the model library | A `floor` prop's lowest vertex lies within the bound of y = 0. Other mounts are listed, not judged (their placement is the place's check, step-36 P5). | `item.rest.floor` |
| C-4 | the same props | No two mesh nodes of one prop interpenetrate beyond the bound. A pair declared `resting` may touch and still fails beyond the bound. | `item.overlap.parts` |

**The mount file.** `clients/3d-spike/assets/models/contact.json`: `{"format": 1, "items": {<slug>: {"mount":
"floor|surface|wall|ceiling", "resting": [[node, node], …], "findings": {"C-3": mm, "C-4": [[node, node, mm], …],
"lane": "…"}}}}`. The slugs are the directory listing (I-S36-5): a directory without an entry fails C-3 "no mount
declared", and an entry without a directory fails too. A node whose mesh has several primitives is named
`<node>#<k>`. A recorded finding is a prop that fails today and is owed a fix by the named lane: the gate still
reports FAIL with the reason prefixed `known finding:`, and `contact-items` exits 0 only while every failure is a
recorded finding no deeper than recorded (+0.1 mm). A new failure, a deeper one, or a fixed finding left in the
file fails the command. CI's `python` layer runs `uv run --locked python tools/character_pipeline/cast.py
contact-items`.

**What the first run found** (2026-10-19; the ledger of `pr-s36-p1-contact-module.md` §8.3–8.4 has the numbers):
the protagonist (`meshy_d.glb`) passes C-1 with 5.0 mm margin on every arm group; `vitruvian.glb` cannot be judged by
C-1 or C-2 (no pure-trunk face; no region channel). Twelve of the forty props fail C-3 or C-4 and are recorded as
findings for the asset lane.
