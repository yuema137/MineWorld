# PR S26-b1 — Route D+ in the repository: the protagonist's character pipeline, ported and parameterized

## DESIGN FROZEN 2026-10-14 (primary session; operator Q-B1-3; rulings §14.1)

```text
Design revision:        revision 2 (2026-10-14), FROZEN: revision 1 (PR #197, commit ad23a1a) with §14.1's
                        rulings applied and the audit anchors re-checked on origin/main after #195
Author:                 S26-b1 PR-design agent, worktree /Users/yuema137/mineworld-worktrees/design-s26b1,
                        branch docs/s26b1-design, planning base origin/main @ 3ada60a
Approved by / evidence: the operator answered Q-B1-3 directly in the primary session on 2026-10-14
                        ("150 美元上限"); the primary session ruled Q-B1-1, -2, -4, -5, accepted the
                        check-characters rename, and froze this design itself (§14.1)
Implementation base:    origin/main at B0 (B0 records the commit)
Start condition:        this design frozen; nothing else (step-26 §12.1)
Execution contract:     §12, confirmed at freeze (primary, 2026-10-14)
Lifecycle:              DRAFT. Not frozen. No implementation, no Meshy call and no OpenAI call is authorized
                        by this document
```

**Effort:** `mvp0` · **Step:** [`step-26-cast.md`](step-26-cast.md) (STEP DESIGN FROZEN 2026-10-14): §5
(the pipeline and its 12 stages), §6 (the standard, gates G-1…G-14), §10.1–§10.2 (budget), §12 (the S26-b1
row), §12.1 (start condition, implementer), §12.2 (AC-S26-2, -3, -9), §15 (`ARC-94`, `DEP-51`, `DEP-52`,
`DEP-55`), §16.1 (rulings) · **Parent:** [`overall.md`](overall.md) · **Source record:**
[`docs/references/CHARACTER_ROUTE_D_PLUS.md`](../../../docs/references/CHARACTER_ROUTE_D_PLUS.md) (route D+, the
protagonist) · **Pattern for this document:** [`pr-s10-p6-lm-controller.md`](pr-s10-p6-lm-controller.md) §9.0
and §12.1 · **Followed by:** S26-b2 (automation spike and pilot B0), which starts after this PR merges.

**Ids.** Step-level ids this PR writes: `ARC-94`, `DEP-51`, `DEP-52`, `DEP-55` (assigned by the primary
session at the step freeze, step-26 §16.1). Local ids: `A-B1-n` (audit), `F-B1-n` (findings), `D-B1-n` (design
decisions), `I-B1-n` (invariants), `AB1-n` (acceptance), `XB1-n` (mutations beyond the step's X-5…X-9),
`R-B1-n` (risks), `Q-B1-n` (operator questions).

---

## 0. Binding inputs, verbatim or exactly restated

| Source | Ruling |
| --- | --- |
| step-26 §12, S26-b1 row, scope | `tools/character_pipeline/` (§5.1): the eight scripts ported, constants into `fit.yaml`, **landmarks still hand-supplied**; stages 3, 6–12; static gates G-1…G-6, G-14 as a CI check on committed characters; secret handling (§5.5); `PIPELINE.md` |
| step-26 §12, S26-b1 row, integration checkpoint | **the pipeline re-derives the protagonist** from her kept download (`out/meshy/apose_v3`) and her `fit.yaml`: G-1…G-14 pass, joints within 1 mm of `meshy_d.glb`'s, weights equal after rounding |
| step-26 §12, S26-b1 row, adversarial | X-5 a 100,001-triangle mesh fails G-1; X-6 a second material fails G-2; X-7 a renamed bone fails G-5; X-8 a key-shaped string in a sidecar fails the scan; X-9 a bound computed from the candidate is rejected in review (`ARC-23` rule 2) |
| step-26 §12.1 | S26-b1 starts when its PR design is frozen; nothing else. **Its PR states the OpenAI dollar cap for the primary to confirm (Q-S26-3).** Haiku for the port; Opus review |
| step-26 §10.2 / Q-S26-3 | S26-b1 reads the current per-image price and states a dollar cap in its PR for the primary session to confirm; **no image is generated before it is confirmed** |
| step-26 §5.1 | Production tooling under the same rules as `tools/asset_generation`: no server, client, kernel, System or contract imports it; never a runtime dependency; runs on an operator's machine, on demand; writes **candidates**. Python 3.12 driver; Blender headless (≥ the version D+ used, **verify** the exact version in S26-b1); Godot 4.7.2 headless for the runtime gates |
| step-26 §5.1 | Per run, in a scratch root given by `MINEWORLD_CAST_SCRATCH`: the unmodified Meshy download, `request.json`, `task.json` — never committed; nothing is ever stripped from them |
| step-26 §5.2 | Stages 3 and 6–12 are deterministic functions of the Meshy download and `fit.yaml` (same triangle count, joints within 1 mm, same weight groups — AC-S26-3). Stages 2 and 4 are samples; never re-run to "reproduce" |
| step-26 §5.2 stage 9 | `fix_flecks.py` automatic and logged; `rebake_head.py` dropped |
| step-26 §5.2 stage 10 | texture **stored once** (external PNG referenced by the asset) — **verify** in S26-b1 that Godot's importer keeps the external reference |
| step-26 §5.3 | per-character knowledge goes into `fit.yaml`, never into code; more than three overrides is a finding |
| step-26 §5.5 | keys from the environment only (`MESHY_API_KEY`, `OPENAI_API_KEY`); never read, printed, logged, copied or committed; no runtime directory names the secrets path; both checked by a CI scan (AC-S26-9) |
| step-26 §6 | a gate's bound is a fixed literal from the protagonist's accepted record or a stated real-world reference, never from the candidate (`ARC-23` rule 2); every automatic gate prints where it looked before its verdict (`ARC-23` rule 1) |
| step-26 §15 | S26-b1 writes `ARC-94` (the per-character standard, review protocol, failed-generation policy), `DEP-51` (Meshy image-to-3D at cast scale), `DEP-52` (Blender headless), `DEP-55` (rigging: our fitted armature kept; the others declined with reasons) |
| operator, 2026-10-14 (step-26 §1) | agents may call Meshy and OpenAI by running `source ~/.config/mineworld/secrets.env` inside the calling command; never read, print, log, copy or commit a key; the path never becomes a hard runtime dependency |
| coordinator brief for this design, 2026-10-14 | spending in S26-b1 is the operator's decision: state the expected credits; re-deriving Vera should reuse the recorded Meshy output rather than buy a generation — verify that this is possible |

## 1. Goal, in one paragraph

Route D+ made one character — Vera, the protagonist — from one Meshy generation, through eight scripts that
live outside the repository with that one generation's numbers typed into them (step-26 A-12). This PR moves
that process into `tools/character_pipeline/` as versioned, parameterized tooling: one MIT Python driver
(`mwcast`), the Blender stages as GPL scripts ported one for one, every per-character number in that
character's `fit.yaml`, the static gates G-1…G-6 and G-14 as code with literal bounds and a CI check, and the
secret rules as a CI scan. It proves the port by **re-deriving Vera** from her kept Meshy download and her
`fit.yaml` and comparing the result with the committed, operator-accepted `meshy_d.glb`: same triangles, joints
within 1 mm, weights equal within 0.005. It spends **no Meshy credit and no OpenAI money**, commits **no new
binary asset**, and changes nothing a player sees.

## 2. Scope

### 2.1 In scope

1. `tools/character_pipeline/`: `README.md` (human), `PIPELINE.md` (specification), `cast.py` (launcher),
   `mwcast/` (driver and gates, MIT, standard library only except the mask and texture modules), `blender/`
   (the ported Blender stages, GPL-2.0-or-later, with `LICENSE`), `godot/` (the import check), `templates/`,
   `tests/`, `requirements.txt`, `ruff.toml`.
2. The D+ scripts ported (A-B1-2's table): `meshy_i23d.py`, `prep_inputs.py`, `gameready_mesh.py`,
   `rig_meshy.py`, `reweight_fingers.py`, `reweight_pack.py`, `fix_flecks.py`, and the support scripts the
   stages need (`inspect_ids.py`, `render_ortho.py`, `render_rig.py`, `dump_bones.py`). `rebake_head.py` is
   **dropped** (step-26 §5.2 stage 9), with the reason in `PIPELINE.md`.
3. Static gates G-1…G-6 and G-14 (§5.6) and `cast.py check-characters` over committed characters, run in CI on
   Linux, macOS and Windows.
4. A repository-wide secrets scan, `scripts/check_secrets.py` (AC-S26-9, X-8), in the `fast` layer.
5. Vera's recorded inputs: `presentation/mineworld-default/3D/characters/cast/vera-lindgren/fit.yaml` and
   `generation_log.yaml` (records only: no GLB, no PNG), plus the re-derivation report
   `rederive_report.json`.
6. The stored-once export format, decided on evidence (§5.9).
7. Decision records `ARC-94`, `DEP-51`, `DEP-52`, `DEP-55` in `docs/DECISIONS.md`.
8. CI wiring in `scripts/ci_layer.py` and the root `pyproject.toml` (`[tool.pyright]`).
9. A dated pointer in `docs/references/CHARACTER_ROUTE_D_PLUS.md` saying where the scripts now live.

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| Landmark detection, weight transfer, Meshy rig as a joint source | S26-b2 (step-26 §5.4 L, W, R) |
| Runtime gate runners G-7…G-13 automated (`A_TPose`, stride, sweep, headtrace, sit) | S26-b2 (step-26 §12) |
| Any Meshy generation, any OpenAI image | S26-b2 onward, under the caps (§5.12) |
| Moving Vera onto a `character.yaml`, removing `Human.Body`, replacing `meshy_d.glb` with the stored-once export | S26-c1 (step-26 §4.5, AC-S26-6). §10 states what this PR hands it |
| `cast.yaml`, the `appearance` System Pack | S26-a |
| Casting briefs, `brief.yaml`, the distinctness table | S26-cast |
| Release texture policy (UASTC) | S26-f (Q-S26-5) |
| `HUMANOID_PROFILE.md` §§5–6 rewrite, `ASSETS.md` rows, `HUMAN_REVIEW_QUEUE.md` milestones | S26-c1 / S26-d (step-26 §15 "Document edits") |
| Any edit under `clients/`, `server/`, `systems/`, `kernel/`, `contracts/`, `worldpack/`, `persistence/`, `cognition/`, `sdk/`, `worlds/` | none — forbidden (I-B1-1) |

### 2.3 Dependencies

None on other open PRs (step-26 §12.1). Machine prerequisites for the local Gate (B10), all present on the
operator's Mac and audited 2026-10-14 (A-B1-9…A-B1-12): Blender 5.2.2 LTS, Godot 4.7.2, the D+ scratch
directory `/Users/yuema137/mineworld-demos/i23d/` with Vera's download and every intermediate, the CharMorph
scratch file `/Users/yuema137/mineworld-worktrees/scratch-character/s3/body_g5.glb`, the cached BiRefNet weights
`~/.rembg/models/birefnet-portrait/birefnet-portrait.onnx`, and the D+ venv
`/Users/yuema137/mineworld-demos/i23d/venv` (rembg 2.0.85, onnxruntime 1.30.0, numpy 2.5.3, Pillow 12.3.0).
CI needs none of them (D-B1-1).

## 3. Audit (`origin/main @ 3ada60a`, and the out-of-repository D+ scratch, 2026-10-14)

### 3.1 The D+ scripts, read in full

| # | Fact | Evidence |
| --- | --- | --- |
| A-B1-1 | **The committed protagonist is exactly the D+ chain's last output.** `sha256(meshy_d.glb)` = `sha256(i23d/rig7/meshy_char.glb)` = `4bc1280f…8d95`; `sha256(meshy_d_base_color.png)` = `sha256(i23d/gr/base_color.png)` = `8117f2b7…20c1` | `shasum -a 256` on both trees |
| A-B1-2 | **The chain, recovered from the D+ session's recorded commands** (transcript `…/subagents/agent-a7d7101349e761284.jsonl`), and each output's mtime matching its script's last edit: (1) `gameready_mesh.py -- out/meshy/apose_v3/model_glb.glb gr 95000`, no env overrides → `gr/low.blend` 01:03:36 (script 01:03:24); (2) `rig_meshy.py` on `gr/low.blend` with `-- …/scratch-character/s3/body_g5.glb rig/meshy_char.glb`, default picks → `rig/` 01:25:50 (script 01:25:45); (3) `reweight_fingers.py` on `rig/meshy_char.blend` → `rig4/` 07:42:10 (script 07:42:02); (4) `ARMZERO=1 SMOOTH=1 reweight_pack.py` on `rig4/meshy_char.blend` → `rig7/` 16:10:45 (script 16:10:40). `rig5` (no flags) and `rig6` (`ARMZERO=1`) are superseded experiments. **`fix_flecks.py` and `rebake_head.py` are not in the chain**: the committed texture is the stage-1 bake (D+ §8.4 step 8) | transcript; `stat`; D+ §§8.4, 8.6, 8.7, 8.9 |
| A-B1-3 | **Vera's Meshy download is kept and complete**: `out/meshy/apose_v3/` holds `model_glb.glb` (32,005,860 B, `sha256 3de801cd…6ef7`), `tex0_base_color.png`, `request.json`, `task.json` (task `01a1155c-dafe-70bc-9279-9cef9a35856b`, `consumed_credits` 30), four thumbnails. The input was `work/in_chest_rgba.png` (`sha256 fc749a47…05fd`). **Re-deriving Vera needs no new generation: 0 credits** | `ls`; `request.json`; `task.json` |
| A-B1-4 | `task.json` carries **signed asset URLs** (`model_url`, `thumbnail_url`, … with `Signature=`, `Key-Pair-Id=`); not a key, but nothing that a committed file should copy. The pipeline never copies `task.json` or its URLs into the repository; provenance carries the task id only | `task.json` |
| A-B1-5 | `meshy_i23d.py`: stdlib `urllib`; `API = https://api.meshy.ai/openapi/v1` (:18); key from `MESHY_API_KEY` only, never printed (:22-25); `HTTPError` body truncated to 2,000 chars (:33); defaults `ai_model meshy-7.1, should_texture, enable_pbr false, texture_resolution 2k, should_remesh false, image_enhancement false, remove_lighting true, moderation false, target_formats [glb], multi_view_thumbnails true` (:47-58); Vera's run added `pose_mode a-pose` (`request.json`); balance printed before and after (:46, :95); polls every 15 s (:76); `request.json` written without image bytes (:60-62) | source |
| A-B1-6 | `prep_inputs.py`: reference path **hard-coded into a deleted worktree** (`/Users/yuema137/mineworld-worktrees/vis-character/…/04_character_closeup.png`, :18-19; that worktree no longer exists); session `birefnet-portrait` (:42); hand-typed crop box `chest (230, 40, 860, 1086)` (:47); `square_pad(fg_ratio 0.85)` to 1024² with alpha threshold 16 (:22-31). The repository's reference has `sha256 5b0c8f0b…3a1`, unchanged since its first commit `8978cee` (`references/PROVENANCE.md` §2), so it is the same image D+ cropped | source; `PROVENANCE.md` |
| A-B1-7 | `gameready_mesh.py`: constants typed for this generation — `EYE_Z_MESHY = 0.779` (:23, read off a render), `SOLE_Z = -0.0177` (:24), `EYE_FROM_SOLE = 1.655` (:25), hip band `|z − 0.05| < 0.03` in the raw model's units (:39); scale `s = 1.655 / (0.779 − sole)` (:37); weld `remove_doubles 1e-6` (:54); Decimate collapse `ratio = TARGET / tris` (:65); EMIT bake, Cycles, GPU `METAL` with CPU fallback, 1 sample, `cage_extrusion` `EXTRUDE` 0.015, `max_ray_distance` `RAYDIST` 0.04, margin 8, 2048² (:119-147); material name `MW_Meshy` (:110); prints every number | source |
| A-B1-8 | `rig_meshy.py`: `S, SOLE_M, SOLE_Z = 0.9563, -0.9516, -0.0177`, `CY = -0.1214` typed from stage 1's print (:24-25; `CY` is never read); `px(u, v)` maps a pick on the 1024 view, `1 px = 1/512` raw unit, **without the centring offset `cx`** that stage 1 applied (:28-31 against `gameready_mesh.py:42`); the 16 picks (:36-47); the CharMorph armature and the **`cm_vitruvian` mesh** from the CharMorph body export (:67-73); centre line from a height map over 7 landmarks `[-0.0177, 0.083, 0.51, 0.947, 1.401, 1.531, 1.637]` (:85-91); spine above the hips kept at CharMorph's curve relative to the hips (:114-118, D+ fix 1); arm, leg and foot joints from slab means (:120-163); tails follow child heads (:165-181); heat weights (:191); head rigid above `chin + 0.01` within `|x| < 0.16` (:215); pack core rigid to `Spine2`: `y > Spine1.y + 0.075`, `z > Hips.z + 0.02`, `z < Spine2.z + 0.10`, `|x| < 0.12` (:217-222); unweighted vertices to the nearest bone head (:224-230); smooth all groups `factor 0.5`, `repeat WSMOOTH=4`, limit 4, normalize (:237-242); scene property `mineworld_provenance` (:248-255); glTF export settings (:259-262) | source |
| A-B1-9 | **The rig source is not in the repository.** `rig_meshy.py` read `scratch-character/s3/body_g5.glb` (CharMorph's CC0 body export, 6,360,660 B, mesh `cm_vitruvian`, 52-joint skin, no images). Of the `cm_vitruvian` mesh it uses **one number**: the front surface at the `Hips` head height, `front_y(cm_co, 1.0048)` — every other centre-line joint's `y` is overwritten (:114-117). Measured from the glTF buffers (numpy, glTF→Blender axes `y_bl = −z_gltf`): **−0.045356** over 13 vertices. Its armature equals the committed `clients/3d-spike/assets/characters/vitruvian/vitruvian.glb`'s: same 52 joint names, identical translations, rotations equal to ~3e-8 | numpy over both GLBs |
| A-B1-10 | `reweight_fingers.py`: per hand, vertices with hand share `> 0.95` re-weighted by inverse distance to the hand's bone segments, `POWER` 6, two nearest kept, normalized (:20, :42-62); prints dominant-bone counts | source |
| A-B1-11 | `reweight_pack.py`: `X_IN 0.12`, `X_OUT 0.21` (:20); region `y > Spine1.y + 0.075`, `z > Hips.z + 0.02`, `z < Spine2.z + 0.10`, `X_IN ≤ |x| < X_OUT` (:30-31); sleeve vertices (arm share `> 0.5`) skipped (:39); blend toward `Spine2` by `f = clip((X_OUT − |x|)/(X_OUT − X_IN))` (:42-44); `ARMZERO` moves `Arm`/`ForeArm` shares to `Spine2` (:45-50); `SMOOTH` over `y > Spine1.y + 0.02`, `z > Hips.z`, `z < Spine2.z + 0.13`, `|x| < X_OUT + 0.02`, 8 passes, limit 4, normalize, with the vertex paint mask (:61-81) | source |
| A-B1-12 | `fix_flecks.py`: `Z_MIN 1.65`, `DARK 0.30`, `LIGHTER 0.12` (:21-22); **also rewrites UVs** of triangles that straddle an island (:65-83; 0 of 95,000 flagged on Vera, D+ §8.4 step 5) | source |
| A-B1-13 | Installed tools: Blender **5.2.2 LTS** (build 2026-09-15; `/Applications/Blender.app`, Homebrew wrapper `/opt/homebrew/bin/blender`); the committed `meshy_d.glb`'s `asset.generator` is `Khronos glTF Blender I/O v5.2.40`, i.e. this Blender. Godot **4.7.2.stable.official.ed1daf0bf** (`/opt/homebrew/bin/godot`). The D+ `.blend` files are zstd-compressed (`28 b5 2f fd`) | `--version`; GLB `asset` |
| A-B1-14 | `meshy_d.glb` structure: one mesh `Body`, one primitive, `POSITION/NORMAL/TEXCOORD_0` float, `JOINTS_0` `UNSIGNED_BYTE`, `WEIGHTS_0` float, 65,176 glTF vertices, **95,000 triangles**, one material `MW_Meshy`, one embedded PNG `base_color`, 52 joints, weight sums 0.9999998…1.0000001, 0 unweighted, sole at glTF `y = −0.017694`, top 1.80158; `scenes[0].extras.mineworld_provenance` with `source_type`, `ai_generated`, `generator`, `meshy_task`, `input`, `postprocess` — **no reference SHA-256 and no pipeline version** | parsed with stdlib `json`/`struct` |
| A-B1-15 | **Vera against G-6's Winter ratios, H = 1.767 m:** upper arm 0.1664 H (Winter 0.186, −10.5 %), forearm 0.1610 H (0.146, +10.3 %), thigh 0.2558/0.2543 H (0.245, +4.4 %/+3.8 %), shank 0.2175/0.2173 H (0.246, **−11.6 %**). Inside ±12 % on every segment, by 0.4 % on the shank | numpy, joint world positions from the inverse bind matrices |
| A-B1-16 | `meshy_d.glb.import`: `meshes/generate_lods=true`, `create_shadow_meshes=true`, `gltf/embedded_image_handling=1` (extract — the reason A-7's PNG exists), the bone map `vitruvian_bonemap.tres` and the six retarget options. `meshy_d_base_color.png.import`: `compress/mode=0`, `mipmaps/generate=true`, `process/size_limit=0` | files |
| A-B1-17 | `human.gd:35` `const MESHY_MATERIAL := "MW_Meshy"`; `_meshy` (:243) re-surfaces only a surface of that name (:342). An export named `MW_Cast` (step-26 G-2) would lose Vera's roughness 0.9 / specular 0.25 until S26-c1 rebinds it | source |

### 3.2 The repository's tooling and CI

| # | Fact | Evidence |
| --- | --- | --- |
| A-B1-20 | `clients/3d-spike/tools/blender/` is the Blender-script precedent: scripts import `bpy`, so they are **GPL-2.0-or-later** with a `LICENSE` copy and an SPDX header; "nothing outside this directory imports these scripts"; the assets they produce are not covered by the scripts' licence | `clients/3d-spike/tools/blender/README.md`, `LICENSE`, `character_model.py:1-2` |
| A-B1-21 | `character_model.py`'s glTF export: `export_format GLB, use_selection, export_apply False, export_yup, export_materials EXPORT, export_normals, export_tangents False, export_texcoords, export_skins, export_animations False, export_morph True` (:237-242). D+ used the same minus morphs (`export_morph=False`) plus `export_extras=True` (A-B1-8) | source |
| A-B1-22 | `tools/asset_generation` is the tooling precedent: not imported by anything shipped; venv outside the repository (`~/.cache/mineworld-gen/venv`); credentials `OPENAI_API_KEY` → `MINEWORLD_SECRETS_FILE` → the documented default; `PRICE_PER_MTOK = {"image_input": 8.0, "text_input": 5.0, "image_output": 30.0}` (`openai_images.py:39-41`), dated 2026-09-27 | README; source |
| A-B1-23 | CI's layers live in `scripts/ci_layer.py`: `fast` (:48, Linux container; includes `check_scratch.py scan` :53 and `PYTHON_STATIC`, :200); `python` (:201, run on ubuntu-24.04, windows-2025, macos-15 by the `python` job of `.github/workflows/ci.yml:210-254`); `python-smoke` (:213). `PYTHON_MEMBERS = ["sdk/python", "cognition/lm-controller"]` (:184). Repository scripts in Python layers are run by `sys.executable`, because Windows has no `python3` | source |
| A-B1-24 | Root `pyproject.toml` is a uv workspace root; `[tool.pyright] include = ["sdk/python", "cognition/lm-controller"]`, `typeCheckingMode = "strict"`, `pythonVersion = "3.12"` (:13-17). Adding a pyright path changes no `uv.lock` | source |
| A-B1-25 | `scripts/ci_changes.py` classifies every path outside `docs/`, `.structured-coding/plans/`, `CLAUDE.md` and root `README*.md` as code, so this PR runs the full PR CI | `ci_changes.py:31-41` |
| A-B1-26 | No key-shaped string (`sk-…`, `msy_…`) is in the tree today; the literal secrets path appears only in plans, docs, `.gitignore`, `tools/asset_generation` and `cognition/` (its config and tests name `secrets.env` as a file name). No runtime directory names `.config/mineworld/secrets.env` | `git grep` |
| A-B1-27 | `.gitattributes` makes every text file LF and marks `*.glb` binary | file |
| A-B1-28 | The 3D reference `04_character_closeup.png` is ChatGPT `gpt-image` output with a C2PA manifest (`references/PROVENANCE.md` §1) — the OpenAI model field of Vera's provenance | file |

### 3.3 What does not exist

`tools/character_pipeline/`; any `characters/cast/` directory in the pack; any static gate or GLB reader in the
repository's Python; any CI check over characters; any committed copy of CharMorph's `cm_vitruvian` body.

### 3.4 Findings carried into the design

- **F-B1-1** The step's integration check names "G-1…G-14 pass", while the same row scopes the runtime runners
  G-7…G-13 to S26-b2. This design runs G-1…G-6 and G-14 on the re-derived Vera, runs G-7's import half, and
  establishes G-8…G-13 by **parity with the accepted asset** (identical triangles, joints within 1 mm, weights
  within 0.005 ⇒ the same deformation under the same clips). **Ruled (Q-B1-1, primary):** as described;
  G-8…G-13 are recorded "by parity", never as PASS.
- **F-B1-2** Two eye lines in D+: `gameready_mesh.py` scales by `EYE_Z_MESHY = 0.779`, `rig_meshy.py` reads
  the eye pick `(512, 113)` = 0.77930 raw units, 0.3 mm apart in the result. Vera's `fit.yaml` records both,
  as D+ used them; S26-b2's landmark stage unifies them.
- **F-B1-3** `px()` omits stage 6's centring offset `cx` (A-B1-8). The faithful port keeps the D+ mapping under
  `picks.frame: "dplus-v1"` and prints `cx`; S26-b2 replaces hand picks altogether.
- **F-B1-4** `fix_flecks.py` is not in Vera's chain (A-B1-2), yet step-26 §5.2 makes stage 9 automatic. The
  port implements it, on by default; Vera's `fit.yaml` sets `flecks.enabled: false` with D+ §8.4 step 8 as the
  reason (one override). **Ruled (Q-B1-5, primary):** yes.
- **F-B1-5** The rig source (A-B1-9) is reproducible from the repository plus one recorded number. **Ruled
  (Q-B1-2, primary):** use the `vitruvian.glb` armature plus that number; `body_g5.glb` is not committed.
- **F-B1-6** G-2 requires `MW_Cast`; the accepted asset's material is `MW_Meshy` and `human.gd` binds that name
  (A-B1-17). The pipeline exports `MW_Cast`; the parity comparison excludes the material name; S26-c1 rebinds
  (§10).
- **F-B1-7** G-6's Winter band holds for Vera by 0.4 % on the shank (A-B1-15). Not a defect of this PR; the
  pilot (S26-b2) reports how close its characters come.
- **F-B1-8** G-14 as specified (reference SHA-256, pipeline version) fails on the committed `meshy_d.glb`, whose
  provenance predates it (A-B1-14). Because this PR ships no character, CI's committed-character check covers
  only the records under `characters/cast/`; the real `meshy_d.glb` is a frozen-evidence unit test that pins
  exactly which gates it passes and which fields G-14 reports missing (§7).

## 4. Reuse, in both directions (`REUSE_POLICY.md`; `CLAUDE.md` §4 rule 16)

| Need | Adopt / build | Why |
| --- | --- | --- |
| Mesh work: import glTF, weld, decimate, bake, armature, heat weights, export glTF | **Blender, headless, adopted** (`DEP-52`) | the D+ tool; a mature mesh library with a glTF I/O the rest of the repository already uses; GPL governs only the scripts that import `bpy` (A-B1-20) |
| Reading a GLB for the gates | **build our own, ~150 lines of stdlib** (`struct`, `json`) | the gates read a fixed subset (accessors, skin, materials, images, scene extras) of one exporter's output; `pygltflib` or `trimesh` would add a dependency to every CI platform for 150 lines, and `trimesh` brings numpy into the uv lock. Recorded in `DEP-52`'s consequences |
| Masking | **`rembg` + BiRefNet, kept** (MIT; DEP-8's 2026-10-06 note at `docs/DECISIONS.md:374-375`) | the D+ stage; tool-only, in a venv outside the repository |
| Meshy REST | **our own stdlib client, kept** | the D+ client; Meshy's SDKs add nothing a 100-line client lacks; the MCP and animation library are excluded by D+ §2 |
| Rig | **our fitted CharMorph armature, kept** (`DEP-55`) | step-26 §5.4 R; Meshy rig, Mixamo, AccuRIG, UniRig/RigNet declined as the rig, each re-verified in B1 |
| PNG pixel comparison (Gate only) | Pillow + numpy from the tool venv | only the local Gate compares textures; CI reads PNG headers with stdlib |
| A pinned Blender in CI | **declined** (D-B1-1) | §5.10 |

*Are we forcing a wheel?* No framework owns the pipeline's execution: the driver is ours, Blender is called as a
subprocess per stage with a JSON parameter file, and every stage's output is a file we can inspect.

## 5. Design

### 5.1 Layout

```text
tools/character_pipeline/
  README.md                  human orientation: what it is, the one command, where the spec is
  PIPELINE.md                the specification (stages, files, schemas, gates, provenance, secrets, platforms)
  cast.py                    launcher: puts this directory on sys.path, calls mwcast.cli.main()
  ruff.toml                  line-length 110, target py311 (Blender's Python), same rule set as the members
  requirements.txt           the mask and texture-compare venv: rembg[cpu]==2.0.85, onnxruntime==1.30.0,
                             numpy==2.5.3, pillow==12.3.0 (the versions D+ ran, A-B1 §2.3)
  mwcast/                    MIT; standard library only, except mask.py and texture.py
    __init__.py              PIPELINE_VERSION = "mwcast-1"
    cli.py                   argument parsing; subcommands (§5.3)
    glb.py                   GLB reader (§5.5)
    records.py               fit.yaml, generation_log.yaml, character.yaml (draft) loaders and validators
    standard.py              every gate bound as a literal, each with its source (§5.6)
    gates.py                 G-1…G-6, G-14 (§5.6)
    compare.py               the parity comparison (§5.7)
    pipeline.py              stage planner and runner: Blender subprocesses, params files, run manifest
    blender.py               finding Blender, the version pin, building a command line
    meshy.py                 Meshy image-to-3D client (stage 4), injected transport, credit ledger and cap
    identifiers.py           AI-identifier inspection of downloads (port of inspect_ids.py)
    mask.py                  stage 3 (imports rembg lazily; run from the tool venv)
    texture.py               texel comparison for the Gate (imports numpy and PIL lazily)
    scratch.py               scratch root resolution; refuses a path inside the repository
  blender/                   GPL-2.0-or-later; LICENSE (byte-identical copy of clients/3d-spike/tools/blender/LICENSE)
    README.md                the licence boundary, as clients/3d-spike/tools/blender/README.md says it
    common.py                params loading, result JSON, export settings, provenance
    gameready.py             stage 6   (port of gameready_mesh.py)
    fit_bind.py              stages 7–8a (port of rig_meshy.py)
    weights_fingers.py       stage 8b  (port of reweight_fingers.py)
    weights_pack.py          stage 8c  (port of reweight_pack.py)
    flecks.py                stage 9   (port of fix_flecks.py)
    export.py                stage 10  (the export the D+ scripts each repeated, once)
    render_ortho.py          the picking render (port of render_ortho.py)
    render_bones.py          stage 12's bone-overlay frames (port of render_rig.py)
    dump_rig.py              armature dump for G-5's roll check (port of dump_bones.py)
  godot/
    inspect_import.gd        prints what Godot made of an imported character (§5.9)
    character.gltf.import    the import template (A-B1-16's settings)
  templates/
    MESHY_CAST_BATCH.txt     the per-batch LICENSES entry, patterned on MESHY_ROUTE_D_CHARACTER.txt
  tests/
    conftest.py              sys.path; a socket guard that fails any network connection
    gltf_fixture.py          builds small synthetic GLBs in memory
    test_glb.py  test_records.py  test_gates.py  test_compare.py  test_meshy.py  test_pipeline.py
```

```text
presentation/mineworld-default/3D/characters/cast/vera-lindgren/
  fit.yaml                   her per-character parameters, every value from the D+ scripts (§5.4)
  generation_log.yaml        her three Meshy runs, with verdicts and reasons (D+ §3)
  rederive_report.json       the re-derivation's numbers (B10)
```

No GLB, PNG or `.blend` is committed by this PR (I-B1-4).

### 5.2 Stage map (step-26 §5.2 → the port)

| # | Stage | Port | D+ source | Notes |
| --- | --- | --- | --- | --- |
| 1 | Casting brief | — | — | S26-cast |
| 2 | Reference images | — | — | S26-b2/S26-cast, `mineworld_gen` reused |
| 3 | Mask | `mwcast/mask.py` | `prep_inputs.py` | crop box from the alpha bounding box by default; `fit.mask.crop_box` overrides (Vera: D+'s box); session, `fg_ratio` 0.85, 1024², alpha threshold 16 kept |
| 4 | Meshy image-to-3D | `mwcast/meshy.py` | `meshy_i23d.py` | not called live in this PR |
| 5 | Raw gate | `generation_log.yaml` schema | D+ §3 | agent judgement recorded per run; schema only here |
| 6 | Game-ready mesh | `blender/gameready.py` | `gameready_mesh.py` | constants from `fit.gameready`; writes `gameready.json` (scale, sole, `cx`, `cy`, counts) |
| 7–8a | Fit armature, bind, head and pack-core fixes, smooth | `blender/fit_bind.py` | `rig_meshy.py` | armature from `vitruvian.glb`; CharMorph's hip front from `fit.fit.charmorph_front_y_at_hips` (A-B1-9); `S`, `SOLE_M`, `SOLE_Z` read from `gameready.json`, not typed; pack core only when `fit.accessories` names a pack |
| 8b | Finger re-weight | `blender/weights_fingers.py` | `reweight_fingers.py` | |
| 8c | Pack side panels | `blender/weights_pack.py` | `reweight_pack.py` | only with a pack; `arm_zero`, `smooth` from `fit.weights.pack_panels` |
| 9 | Fleck repair | `blender/flecks.py` | `fix_flecks.py` | on by default; Vera off (F-B1-4) |
| 10 | Export | `blender/export.py` | the export block of each D+ script | GLB (parity) and the stored-once format (§5.9); material renamed `MW_Cast`; provenance (§5.8) |
| 11 | Runtime tuning | — | — | S26-b2 |
| 12 | Gates and review frames | `mwcast/gates.py`, `blender/render_bones.py` | §§7–8 of D+, `render_rig.py` | static gates here; runtime gates S26-b2 |

Every Blender stage is one subprocess:

```text
<blender> --background --factory-startup [<input.blend>] --python-exit-code 1
          --python tools/character_pipeline/blender/<stage>.py -- <params.json>
```

`params.json` is written by the driver into the run directory and contains every value the script uses
(nothing from the environment); the script writes `<stage>.blend` (where the next stage needs it) and
`<stage>.json` (every number it decided, the same numbers D+ printed), and prints them too. A script reads
nothing else. `--factory-startup` keeps the user's preferences out of the result (R-B1-3).

### 5.3 The command line (`python3 tools/character_pipeline/cast.py …`)

| Subcommand | Does | Needs |
| --- | --- | --- |
| `gate <asset.glb> --fit <fit.yaml> [--json]` | G-1…G-6, G-14 on one export; exit 1 on any FAIL | stdlib |
| `check-characters [--root <repo>]` | for every `presentation/*/3D/characters/cast/*/`: validates `fit.yaml`, `generation_log.yaml`, and, where `character.yaml` exists, runs `gate` on its asset; prints one line per directory and the count; exit 1 on any failure. A directory with records only prints `records only (no character.yaml)` | stdlib |
| `run --fit <fit.yaml> --download <dir> [--scratch <dir>] [--from <stage>] [--to <stage>] [--run-id <id>]` | stages 6–10 (and 12's frames) into `<scratch>/<character>/<run-id>/`; writes `run_manifest.json` (fit SHA-256, download SHA-256, Blender version, bake device, stage outputs with SHA-256) | Blender 5.2.2 |
| `compare <candidate.glb> <accepted.glb> [--json]` | the parity comparison (§5.7); exit 1 when outside a bound | stdlib |
| `compare-texture <candidate.png> <accepted.png>` | texel comparison (§5.7) | tool venv |
| `mask --fit <fit.yaml> --reference <png> --out <png>` | stage 3 | tool venv |
| `meshy --fit <fit.yaml> --input <png> --scratch <dir> --cap <credits> [--dry-run]` | stage 4; `--dry-run` prints the request (no image bytes) and the credit projection and calls nothing | stdlib; the key in the environment for a live run |
| `identifiers <file> …` | AI-identifier inspection (port of `inspect_ids.py`) | stdlib |

Scratch resolution (`mwcast/scratch.py`): `--scratch`, else `MINEWORLD_CAST_SCRATCH`, else refuse with a message
naming both. A scratch path inside the repository (`git rev-parse --show-toplevel` is a prefix of its resolved
path) is refused (I-B1-5).

Blender resolution (`mwcast/blender.py`): `MINEWORLD_BLENDER`, else `blender` on `PATH`, else
`/Applications/Blender.app/Contents/MacOS/Blender` on macOS. The first line of `<blender> --version` must be
exactly `Blender 5.2.2 LTS` (`BLENDER_PIN` in `standard.py`); otherwise refuse, unless
`--allow-blender "<first line>"` is given, which is recorded in `run_manifest.json` and makes the run a **new
parity baseline**, never evidence of reproduction.

### 5.4 `fit.yaml` (written in YAML's JSON-compatible subset, parsed with `json`, like `ARC-46`'s files)

Unknown keys are refused naming the key path; every number is a JSON number; `records.py` validates types and
ranges. `PIPELINE.md` holds the schema and the **defaults**, which are Vera's D+ values; an **override** is a
value that differs from the default, and `check-characters` prints the override count (step-26 §5.3: more than three
is a finding). Hand-supplied picks and the per-character source block are not overrides.

Vera's file (B9), with every value's D+ origin written in a `"_why"` string beside it where the schema allows:

```json
{
  "format": 1,
  "character": "vera-lindgren",
  "stature_m": 1.767,
  "accessories": ["backpack"],
  "source": {
    "meshy_task": "01a1155c-dafe-70bc-9279-9cef9a35856b",
    "download_file": "model_glb.glb",
    "download_sha256": "3de801cdb7dcf34d21ee45758982319096bd62f3a111ab354c46fb2eeb586ef7",
    "input_sha256": "fc749a474dcb5b7c1ee1b45e601b2bb0aa8c38e6016811d67c5813d0a68b05fd",
    "reference": { "path": "presentation/mineworld-default/3D/references/04_character_closeup.png",
                   "sha256": "5b0c8f0b818a9ca7555515e7374f2ec40770a09bc990f8c568b6dc913e41a3a1",
                   "generator": "ChatGPT gpt-image (C2PA, references/PROVENANCE.md §1)" }
  },
  "mask": { "session": "birefnet-portrait", "crop_box": [230, 40, 860, 1086], "fg_ratio": 0.85,
            "size_px": 1024, "alpha_threshold": 16,
            "_why": "prep_inputs.py:42-47; the reference is chest-up, so the box is crown to mid-thigh" },
  "gameready": { "target_triangles": 95000, "eye_z_raw": 0.779, "centre_band_z_raw": 0.05,
                 "centre_band_half_raw": 0.03, "sole_z_m": -0.0177, "eye_from_sole_m": 1.655,
                 "weld_m": 1e-06,
                 "bake": { "size_px": 2048, "samples": 1, "margin_px": 8, "extrusion_m": 0.015,
                           "ray_m": 0.04, "device": "auto" } },
  "fit": { "rig_source": "clients/3d-spike/assets/characters/vitruvian/vitruvian.glb",
           "charmorph_front_y_at_hips": -0.045356,
           "charmorph_landmarks_z": [-0.0177, 0.083, 0.51, 0.947, 1.401, 1.531, 1.637],
           "picks": { "frame": "dplus-v1", "view_px": 1024, "px_per_raw_unit": 512,
                      "r_shoulder": [418, 228], "l_shoulder": [606, 228],
                      "r_elbow": [325, 355], "l_elbow": [699, 355],
                      "r_wrist": [248, 480], "l_wrist": [776, 480],
                      "r_handtip": [200, 530], "l_handtip": [828, 530],
                      "r_hip": [467, 490], "l_hip": [557, 490],
                      "r_knee": [470, 730], "l_knee": [553, 730],
                      "r_ankle": [458, 935], "l_ankle": [565, 935],
                      "chin": [512, 155], "eye": [512, 113] } },
  "weights": { "head_rigid": { "above_chin_m": 0.01, "half_width_m": 0.16 },
               "pack_core": { "behind_spine1_m": 0.075, "above_hips_m": 0.02, "above_spine2_m": 0.10,
                              "half_width_m": 0.12 },
               "smooth": { "factor": 0.5, "repeat": 4, "limit": 4 },
               "fingers": { "hand_share_min": 0.95, "power": 6, "keep": 2 },
               "pack_panels": { "x_in": 0.12, "x_out": 0.21, "sleeve_share_max": 0.5, "arm_zero": true,
                                "smooth": true, "smooth_repeat": 8 } },
  "flecks": { "enabled": false, "z_min_m": 1.65, "dark": 0.30, "lighter": 0.12,
              "_why": "not in the accepted chain: D+ §8.4 step 8 committed the reviewed bake" },
  "export": { "material": "MW_Cast" }
}
```

`charmorph_front_y_at_hips` is measured once in B10 in Blender from `body_g5.glb` and must equal A-B1-9's
numpy value within 1e-5; the measured value is what the file keeps. The pick coordinates, the slab half-widths
and the `xmin` filters of `rig_meshy.py` that are **method** (`rx 0.03/0.04/0.02`, `rz 0.02`, `xmin 0.2/0.3`,
the toe offset 0.012, the foot selector `|x − xa| < 0.06, z < 0.06`) stay code constants in `fit_bind.py`,
each with its D+ line in a comment, and are listed in `PIPELINE.md`.

`generation_log.yaml` (Vera): one entry per run — `run`, `task_id` (D+ §3: `01a11555-…`, `01a11559-…`,
`01a1155c-dafe-70bc-9279-9cef9a35856b`; the first two are recorded as the prefixes D+ kept, marked
`"task_id_complete": false`), `params` (remesh/raw), `credits` 30 each, `verdict` (`rejected` / `rejected` /
`kept`), `reason` (D+ §3's sentence), `date` 2026-10-07. Totals: 3 runs, 90 credits, 1 kept.

### 5.5 `mwcast/glb.py` — the GLB reader

```python
class GlbError(Exception): ...

@dataclass(frozen=True)
class Glb:
    path: Path
    document: Mapping[str, object]     # the glTF JSON chunk
    binary: bytes                      # the BIN chunk

def read_glb(path: Path) -> Glb                                 # magic, version 2, chunk types checked
def accessor(glb: Glb, index: int) -> list[tuple[float, ...]]   # FLOAT, UNSIGNED_BYTE/SHORT/INT, normalized
def triangle_count(glb: Glb) -> int                             # sum over TRIANGLES primitives of indices/3
def material_names(glb: Glb) -> list[str]
def skin_joint_names(glb: Glb) -> list[str]
def joint_parents(glb: Glb) -> dict[str, str | None]            # from node children, names only
def joint_world_positions(glb: Glb) -> dict[str, tuple[float, float, float]]   # inverse of each IBM
def vertex_weights(glb: Glb) -> list[dict[str, float]]          # JOINTS_0/WEIGHTS_0, by joint name
def positions(glb: Glb) -> list[tuple[float, float, float]]
def scene_extras(glb: Glb) -> Mapping[str, object]
def embedded_image(glb: Glb, index: int) -> bytes
def png_size(data: bytes) -> tuple[int, int, int, int]         # width, height, bit depth, colour type
```

Positions are glTF's (Y up); gates convert explicitly where a bound is stated in Blender's Z-up frame. A
4×4 inverse is a small Gauss–Jordan function in `glb.py`, tested against a hand-computed matrix.

### 5.6 Gates in this PR (`gates.py`, `standard.py`)

`GateResult(gate: GateId, verdict: Verdict, looked_at: tuple[str, ...], reasons: tuple[str, ...])`,
`Verdict = PASS | FAIL`. Each gate first fills `looked_at` (what it read: counts, extrema, the region), then
decides. Every bound is a module-level literal in `standard.py` with a comment naming its source; no gate
takes a bound as an argument and no bound is computed from the asset (X-9, reviewed in B3).

| Gate | Check in this PR | Bound (`standard.py`) and source |
| --- | --- | --- |
| G-1 | triangle count of the one skinned mesh `≤ MAX_TRIANGLES`; exactly one mesh with a skin | `100_000` (D+ §5.2) |
| G-2 | exactly one material, named `MATERIAL`; it has a `baseColorTexture` and no normal, occlusion, emissive or metallic-roughness texture; one primitive | `"MW_Cast"` (step-26 §4.5, G-2) |
| G-3 | the base-colour image is `TEXTURE_PX × TEXTURE_PX`, 8-bit RGB or RGBA (sRGB is glTF's base-colour space); when a Godot `.import` for the PNG exists beside it: `mipmaps/generate=true`; the compression mode is printed, not judged (Q-S26-5 is S26-f's) | `2048` (D+ §5.2) |
| G-4 | (a) soles: min of glTF `y` within `SOLE_TOL_M` of `SOLE_Z_M`; (b) sole-to-eye: the eye height from `fit.yaml` (the `eye` pick through stage 6's transform, `gameready.json`) minus the sole, within `EYE_TOL_M` of `stature_m × EYE_RATIO`. The realistic stature range of step-26 §7.2 needs a brief and is S26-cast's | `-0.0177`, `0.005`, `0.9366`, `0.010` (step-26 G-4) |
| G-5 | the 52 joint names and their parent relation equal the reference skeleton read from `vitruvian.glb`; one `JOINTS_0`/`WEIGHTS_0` set and no `_1`; every vertex's weight sum within `WEIGHT_SUM_TOL` of 1; 0 vertices below `UNWEIGHTED_SUM`. Rolls: checked from `dump_rig.py`'s JSON when a run directory is given (equal to the reference's rolls within `ROLL_TOL_RAD`); the bone map's Godot validation is S26-b2's | `1e-3`, `1e-4`, `1e-4` |
| G-6 | (a) Winter: upper arm, forearm, thigh, shank of both sides, each `|joint→joint| / stature_m` within ±`WINTER_TOL` of Winter's ratio; (b) inside the body: for the 18 trunk and limb joints (Hips, Spine, Spine1, Spine2, Neck, Head, and per side Arm, ForeArm, Hand, UpLeg, Leg, Foot), the joint's horizontal position lies inside the axis-aligned extent of the mesh vertices in a slab of ±`SLAB_HALF_M` at the joint's height within `NEIGHBOUR_M` horizontally; prints each joint's margins. (c) The 30 mm geometric cross-check needs a detector to cross-check and is S26-b2's (step-26 §5.4 L) | `{upper_arm 0.186, forearm 0.146, thigh 0.245, shank 0.246}`, `0.12` (Winter, step-26 G-6); `0.010`, `0.15` |
| G-14 | `scenes[0].extras.mineworld_provenance` has: `source_type == "generated"`, `ai_generated is true`, non-empty `generator`, `meshy_task` (UUID form), `references` (list of `{path, sha256}`; each `sha256` is 64 hex; when the path exists in the repository its SHA-256 matches), `reference_generator`, `postprocess`, `pipeline` (`{version, fit_sha256, blender}`); no string anywhere in the provenance matches the key patterns of `scripts/check_secrets.py`; the per-batch `LICENSES/` entry and the `ASSETS.md` row are checked by `check-characters` once a `character.yaml` names them | `ARC-9`, `DEP-8`, D+ §4 |

Winter's table must be verified in B1 against a citable edition (step-26 G-6 "**verify**"): D. A. Winter,
*Biomechanics and Motor Control of Human Movement*, 4th ed., Wiley 2009, Fig. 4.1 (segment lengths as a
fraction of height H). If the edition's figure differs from step-26's numbers, that is a **material stop**
(the step's frozen gate), reported with the page.

### 5.7 The parity comparison (`compare.py`, `texture.py`) — the integration check's instrument

Bounds fixed before any run (`ARC-23`), literals in `standard.py`:

| Quantity | Rule | Bound |
| --- | --- | --- |
| Triangles, glTF vertex count, joint count | equal | — |
| Joint names | equal sets | — |
| Joint world position (from the inverse bind matrices), per joint | Euclidean distance | `PARITY_JOINT_M = 0.001` (step-26: "within 1 mm") |
| Vertex position, by index | max abs component difference | `PARITY_POSITION_M = 1e-05` |
| UV, by index | max abs difference | `PARITY_UV = 1e-06` |
| Weights, by vertex index and joint name | max abs difference over every (vertex, joint) | `PARITY_WEIGHT = 0.005` ("equal after rounding" to two decimals, read as half a rounding step, so a value on a rounding boundary does not flip the verdict) |
| Material name | reported, not compared (F-B1-6) | — |
| Provenance | reported, not compared | — |
| Base colour texels (`compare-texture`, same bake device only) | share of texels whose every channel differs by ≤ 2/255 | `PARITY_TEXEL_SHARE = 0.999` |

`compare` prints, before its verdict, the counts it compared and the worst joint, vertex and weight with their
indices. If vertex order differs (counts equal but positions disagree at index 0), the verdict is FAIL with the
message "vertex order differs" — the comparison never re-matches vertices by search, because a re-ordered
mesh is a reproduction failure to diagnose, not to hide.

### 5.8 Provenance written by stage 10

```json
"mineworld_provenance": {
  "source_type": "generated", "ai_generated": true,
  "generator": "Meshy image-to-3D, ai_model meshy-7.1, pose_mode a-pose, paid plan",
  "meshy_task": "<from fit.source.meshy_task>",
  "references": [ { "path": "<fit.source.reference.path>", "sha256": "<…>" } ],
  "reference_generator": "<fit.source.reference.generator>",
  "input_sha256": "<fit.source.input_sha256>",
  "postprocess": "scaled to stature; welded; decimated to <n> triangles; base colour re-baked; fitted on the CharMorph 52-joint skeleton; heat weights with head, pack and finger fixes",
  "pipeline": { "version": "mwcast-1", "fit_sha256": "<sha256 of fit.yaml bytes>", "blender": "Blender 5.2.2 LTS" }
}
```

The `generator` string's Meshy model comes from the download's `request.json` (`params.ai_model`,
`params.pose_mode`), read by the driver from the scratch run directory; the export never copies `task.json`.

### 5.9 Stored once: the export format (step-26 §5.2 stage 10, "**verify**")

`export.py` writes two outputs from the same `.blend`:

1. `<id>.glb` — embedded PNG, the exact D+ export settings (A-B1-8 :259-262), for the parity comparison with
   `meshy_d.glb`;
2. `<id>.gltf` + `<id>.bin` + `<id>_base_color.png` — Blender's `GLTF_SEPARATE`, the image datablock named so
   the PNG file is `<id>_base_color.png`.

B10 decides with Godot 4.7.2 headless, in a scratch Godot project (never in `clients/3d-spike`): copy the three
files, `vitruvian_bonemap.tres`, `godot/character.gltf.import` (renamed `<id>.gltf.import`), and
`godot/inspect_import.gd`; run `godot --headless --path <proj> --import`, then
`godot --headless --path <proj> --script res://inspect_import.gd`. The script prints: the material's albedo
texture `resource_path`, the surface count, the skeleton's bone count and the first three bone names, and the
list of project files with "base_color" in the name. **Decision rule, fixed now:**

- **GLTF_SEPARATE is the stored-once format** if: the albedo path is `res://<id>_base_color.png`; one surface;
  52 bones renamed by the bone map (`Hips` present); and no project file other than `<id>_base_color.png` and
  its `.import` holds the base colour.
- Otherwise the result is recorded and the question goes to the primary session with the evidence, with the
  fallback named: GLB with `gltf/embedded_image_handling=3` (embedded, uncompressed; the bytes once, inside the
  GLB). That fallback moves the texture's import policy out of a per-PNG `.import`, which touches Q-S26-5, so
  it is not chosen autonomously (material stop).

### 5.10 CI: three operating systems, no Blender (D-B1-1)

**Decided: CI runs no Blender and no Godot for this PR** (ruled, Q-B1-4, primary: the local Gate is re-run
whenever the Blender scripts change). CI runs, on Linux, macOS and Windows (the existing
`python` job), the standard-library part: the unit tests (`pytest tools/character_pipeline/tests`), ruff and
pyright over the driver, `cast.py check-characters` over the committed records; and, in `fast` (Linux), the secrets
scan.

Why not a pinned Blender in CI:

1. **The inputs cannot be there.** Re-deriving needs the Meshy download (32 MB) and, for new characters, every
   future download; `DEP-8`'s Meshy clause and step-26 §5.1 keep them unmodified **outside** the repository.
   A Blender job in CI would have nothing real to run on; a synthetic mesh would test that Blender works,
   which the test rules forbid as a unit's whole claim.
2. **The bake is a GPU-device-dependent render.** Cycles on Metal (the operator's Mac, where every accepted
   character is made) and Cycles on a CI runner's CPU need not agree texel for texel; a CI parity run would
   compare against a different device and prove nothing about the accepted asset.
3. **Cost.** Blender 5.2.2 is a several-hundred-megabyte download per OS per run, for a stage that changes only
   when `blender/` changes.
4. **What CI can own, it owns:** every gate on committed files (static gates read GLB, PNG and `.import`
   bytes), every record's schema, every pure function of the driver (stage planning, command lines, parameter
   files, the credit cap, the scratch refusal, the parity arithmetic), and the secrets scan.

The Blender stages' correctness is owned by the **local real-lifecycle Gate** (B10), on the pinned Blender
(`BLENDER_PIN`), on the operator's Mac, re-run whenever `blender/` changes (PIPELINE.md says so, and the
review question of every later PR that edits `blender/` asks for it). Recorded in `DEP-52` with the option
"Blender in CI" and its verdict.

### 5.11 Secrets (step-26 §5.5)

- `meshy.py` reads `MESHY_API_KEY` from `os.environ` once, at the request, never from a file; refuses with
  "MESHY_API_KEY is not set" when absent. No log line, exception, `request.json`, `task.json`, run manifest or
  test output contains the header or the key; an HTTP error body is truncated to 2,000 characters and every
  occurrence of the key's value in it is replaced by `<redacted>` before it is stored or printed.
- The pipeline never names `~/.config/mineworld/secrets.env` in code. `PIPELINE.md` documents the operator
  convention (`source ~/.config/mineworld/secrets.env` inside the calling command) as one way to export the
  variables.
- `scripts/check_secrets.py` (stdlib; `scan` and `--self-test`):
  1. no tracked text file matches a key pattern: `sk-[A-Za-z0-9_-]{20,}`, `sk-proj-[A-Za-z0-9_-]{20,}`,
     `msy_[A-Za-z0-9]{20,}`, `Bearer [A-Za-z0-9._-]{20,}`, `Key-Pair-Id=[A-Z0-9]{8,}` (a signed URL, A-B1-4);
  2. no tracked file under `clients/ server/ systems/ kernel/ contracts/ worldpack/ persistence/ cognition/ sdk/`
     contains `.config/mineworld/secrets.env`;
  3. it lists files with `git ls-files -z`, skips files Git marks binary by the `.gitattributes` `binary`
     attribute or that contain a NUL byte in their first 8,000 bytes, and prints the count scanned;
  4. an allowlist is a literal dict in the script, each entry with a reason; it starts **empty**.

### 5.12 Spending

| | This PR | Basis |
| --- | --- | --- |
| Meshy credits | **0** | Vera's download is kept (A-B1-3); the Meshy client is tested with an injected transport only; no `meshy` subcommand runs without `--dry-run` |
| OpenAI images | **0** | no stage 2 in this PR |
| OpenAI cap for the step (Q-S26-3) | **$150** for the whole of S26 — **ruled by the operator, Q-B1-3** ("150 美元上限"): reported at 50 % and 100 %, hard stop at the cap | below |

**Price read 2026-10-14** (OpenAI pricing page, `https://developers.openai.com/api/docs/pricing`, Standard tier,
per 1M tokens): `gpt-image-2.5-sunburst` and `gpt-image-2.5-flare` — image input $8.00, text input $5.00, image
output $30.00; cached input does not apply to `/v1/images/edits`. The image guide
(`https://developers.openai.com/api/docs/guides/image-generation`) gives no fixed 2.5 figure for a 1024×1536
high image (its calculator is interactive); it gives GPT Image 2 at **$0.165** per high 1024×1536 image and
6,240 output tokens for that size on earlier models. **Estimate per image** (`mineworld_gen` at 1024×1536,
high, up to four references): output ≤ 6,240 tokens × $30/M = $0.187; image input ≈ 6,000 tokens × $8/M ≈ $0.05;
text ≈ $0.002 → **≈ $0.24 per image, an upper estimate**. Step-26 §10.2's ceiling: 30 people × 20 images = 600
images → **≈ $144**. **Cap $150, ruled by the operator (Q-B1-3)**, reported at 50 % and 100 % like the Meshy
cap, with a hard stop at the cap; the pilot (S26-b2) records each image's `cost_usd_estimate` (already computed
by `mineworld_gen` from usage) and re-estimates before S26-cast's batches. This PR generates nothing.

### 5.13 Platforms

The driver and the gates are standard-library Python 3.12 with `pathlib`, `subprocess` argument lists (never a
shell string), binary reads for GLB/PNG and UTF-8 reads for records; they run and are tested on Linux, macOS and
Windows. The Blender stages run wherever Blender 5.2.2 runs (all three); the `"auto"` bake device tries Metal,
then CUDA/OptiX/HIP, then CPU, and records the device used, because the texture parity bound applies only within
one device (§5.7). The local Gate in this PR runs on macOS only (the operator's machine); a Linux or Windows
re-derivation is a new parity baseline (`--allow-blender`/device recorded), not part of this PR's acceptance.

## 6. Invariants this PR must hold

| | Invariant |
| --- | --- |
| I-B1-1 | No file under `clients/ server/ systems/ kernel/ contracts/ worldpack/ persistence/ cognition/ sdk/ worlds/` changes. Nothing outside `tools/character_pipeline/` imports `mwcast` |
| I-B1-2 | No Meshy credit is spent and no OpenAI request is made by any agent in this PR |
| I-B1-3 | No key is read, printed, logged, copied or committed; nothing under the runtime directories names the secrets path (§5.11) |
| I-B1-4 | No GLB, PNG, `.blend`, Meshy download, `request.json` or `task.json` is committed |
| I-B1-5 | Every pipeline output goes to a scratch root outside the repository; a scratch root inside it is refused |
| I-B1-6 | Every gate bound is a literal with a source; none is computed from the candidate (`ARC-23` rule 2) |
| I-B1-7 | `meshy_d.glb`, `meshy_d_base_color.png` and their `.import` files are unchanged |
| I-B1-8 | Blender scripts are GPL-2.0-or-later and live only in `tools/character_pipeline/blender/`; the driver never imports them, only runs them |
| I-B1-9 | Per-character numbers live in `fit.yaml`; the Blender scripts contain no number typed for one character |

## 7. Test ownership (test rules §26)

| Layer | Owns |
| --- | --- |
| **Static** (ruff, pyright strict over `mwcast/`, `tests/`, `cast.py`; ruff only over `blender/`, which imports `bpy`) | syntax, imports, types; not duplicated in units |
| **Unit** (pytest, CI, three OSes) | GLB decoding (accessor types, normalized integers, IBM inverse) against synthetic GLBs with hand-computed expectations; each gate's PASS and FAIL on synthetic GLBs (X-5, X-6, X-7, plus a joint outside the body for G-6b and a missing provenance field for G-14); **frozen real evidence**: `meshy_d.glb` passes G-1 (95,000), G-5, G-6 (with `stature_m` 1.767) and fails G-2 with "material MW_Meshy" and G-14 naming exactly `references`, `reference_generator`, `pipeline` as missing — pinned from this design's audit (A-B1-14, A-B1-15), not from the implementation; records: Vera's `fit.yaml` and `generation_log.yaml` load, an unknown key and a wrong type are refused with the key path; parity arithmetic (a joint moved 1.1 mm fails, 0.9 mm passes; a weight moved 0.006 fails; reordered vertices fail with the message); the Meshy client with a fake transport (request body has no `Authorization`, `request.json` has no image bytes, the key never appears in any stored file or captured output, a 401 body echoing the key is redacted, the cap refuses the run that would exceed it, `--dry-run` calls nothing); the scratch refusal; Blender command lines and params files built for each stage; the secrets scan's self-test (X-8) |
| **Gate 1** (real LLM) | **NOT REQUIRED**: no LLM-facing change |
| **Gate 2** (real lifecycle, local, the operator's Mac) | B10: real Blender 5.2.2 runs stages 6–10 on Vera's real download; stage-by-stage parity with the D+ intermediates; the final parity with `meshy_d.glb`; stage 3 re-run against the recorded Meshy input; the Godot import of the stored-once format; static gates on the re-derived export. Owned here because only real Blender on the real download can show reproduction |
| **CI** | the PR's automatic run, once on the final head: `fast` (Linux, includes the secrets scan and the pipeline's static checks), `python` on three OSes (pytest and `check-characters`), and the unchanged rest |

Selective CI: this PR touches `scripts/ci_layer.py` and the root `pyproject.toml`, so the full PR CI runs (test
rules §11: CI workflow and central configuration fall back to full) — the one canonical run.

## 8. Acceptance criteria (fixed before anything is measured, `ARC-23`)

| | Criterion | Evidence |
| --- | --- | --- |
| AB1-1 | `pytest tools/character_pipeline/tests` passes on Linux, macOS and Windows in the PR's CI, with the counts recorded | CI `python` job |
| AB1-2 | X-5: a synthetic mesh of 100,001 triangles fails G-1 and the message names 100001; 100,000 passes | unit |
| AB1-3 | X-6: a second material fails G-2 naming both materials | unit |
| AB1-4 | X-7: one renamed bone (`mixamorig:LeftForeArm` → `mixamorig:LeftForearm`) fails G-5 naming the missing and the unexpected name | unit |
| AB1-5 | X-8: `check_secrets.py --self-test` plants a `msy_`-shaped string and an `sk-`-shaped string in an in-memory sidecar and both are found; `check_secrets.py scan` over the tree reports 0 findings and the number of files scanned | `fast` |
| AB1-6 | X-9: review finds no gate bound computed from the candidate; `standard.py` holds every bound as a literal with a source comment | B3 review, recorded |
| AB1-7 | **Integration (B10):** stage 6 on Vera's download prints 95,000 triangles, 47,142 vertices, 2 parts (44,143 and 2,999), 0 non-manifold edges (D+ §8.1's numbers) | `gameready.json` |
| AB1-8 | **Integration (B10):** `compare <re-derived>.glb meshy_d.glb` passes every row of §5.7 (same 95,000 triangles and 65,176 vertices; every joint within 1 mm; positions ≤ 1e-5 m; UVs ≤ 1e-6; weights ≤ 0.005) | `rederive_report.json` |
| AB1-9 | **Integration (B10):** on the same device as D+ (Metal), `compare-texture` of the re-derived base colour against `meshy_d_base_color.png` reaches ≥ 99.9 % of texels within 2/255 | `rederive_report.json` |
| AB1-10 | **Integration (B10):** `gate` on the re-derived export: G-1, G-2, G-3, G-4, G-5 (rolls included), G-6 (a, b) and G-14 PASS | `rederive_report.json` |
| AB1-11 | **Integration (B10):** the stored-once decision of §5.9 recorded with Godot's printed evidence | ledger, `PIPELINE.md` |
| AB1-12 | **Counterfactual (B10, XB1-1):** with `l_shoulder` moved by +5 px in a scratch copy of `fit.yaml`, `compare` FAILS naming `mixamorig:LeftArm` (drift ≈ 9 mm) — the comparison is load-bearing on real data | ledger |
| AB1-13 | Stage 3 re-run (`mask`) on the repository's reference with Vera's `fit.mask` reproduces `work/in_chest_rgba.png`: alpha IoU (alpha > 127) ≥ 0.999 and RGB mean absolute difference over the foreground ≤ 1/255. If the BiRefNet weights are missing, NOT RUN with the reason (not a pass) | ledger |
| AB1-14 | `check-characters` in CI prints `vera-lindgren: records only (no character.yaml)`, 0 overrides beyond `flecks.enabled`, and exits 0 | CI log |
| AB1-15 | I-B1-1, I-B1-4, I-B1-7: `git diff --stat origin/main...HEAD` lists no path under the forbidden directories, no binary file, and no change to `meshy_d*` | close-out |
| AB1-16 | `docs/DECISIONS.md` gains `ARC-94`, `DEP-51`, `DEP-52`, `DEP-55`; `check_decision_ids.py` passes | `fast` |

**G-7…G-13 for the re-derived Vera (F-B1-1; ruled, Q-B1-1, primary):** AB1-11 covers G-7's import half; G-8…G-13 are
established by AB1-8 (same mesh and skin as the asset the operator accepted on 2026-10-07 with every runtime
check D+ §§8.4–8.9 records). This is written into `rederive_report.json` as `"G-8…G-13": "by parity (AB1-8)"`,
not as PASS.

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason. Targeted validation
per commit; the PR's CI is the one full run (test rules §§8–9). Every commit is written for a small-model
implementer: numbered steps, exact paths, exact commands. A step that says "read" is part of the work, and what
was read is recorded in §15 before the commit.

### 9.0 The procedure for every commit

Run every command from the worktree root `/Users/yuema137/mineworld-worktrees/impl-s26b1`. Never `cd`
elsewhere. Paths below are relative to it unless absolute.

1. Re-read this commit's section and the design sections it cites. Re-read every file you will edit, in full,
   before editing it.
2. Make the numbered implementation steps, in order. Do not do work that belongs to a later commit.
3. Run the commit's validation commands. Record for each: the command, the test count, the wall time, and
   PASS / FAIL / INCONCLUSIVE (a test not run is not a pass).
4. For each mutation the commit lists: plant it, run the named test, record that it turned red (with the failing
   test's name), revert it, run the test again green. Never commit a planted mutation.
5. Static checks for any commit touching `tools/character_pipeline/` or `scripts/`:
   - `uv run --locked ruff check tools/character_pipeline scripts/check_secrets.py`
   - `uv run --locked ruff format --check tools/character_pipeline scripts/check_secrets.py`
   - `uv run --locked pyright tools/character_pipeline/mwcast tools/character_pipeline/tests tools/character_pipeline/cast.py`
   - `uv run --locked pytest tools/character_pipeline/tests` (one path only)
   - Markdown: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
6. Review: answer each review question of the commit in §15 in one or two sentences.
7. `git status --short` and `git diff --stat`: every changed path must be in the commit's **Files** list. If
   another path changed, revert it or stop and record why.
8. Update §15 (ledger row, evidence, findings) and the handoff file in the same commit.
9. Stage by explicit path (`git add <path> …`, never `git add -A`), then `git commit -m "<message>"` with the
   message given, plus the attribution line your session specifies. Separate commands; no heredoc, no `sed -i`,
   no `python3 -c`.
10. Stop and report (a material stop, §12) instead of improvising if a step cannot be done as written.

Blender and Godot commands (B7 smoke, B10 only) are run exactly as written, with output redirected to a log
**inside the scratch root** (`> <scratch>/<name>.log 2>&1`), and the log read in full before a verdict.

#### B0 — Freeze and contract (Markdown only)

- **Goal.** Start against the frozen design. **Files:** this file (§15);
  `.structured-coding/plans/mvp0/handoff-s26b1.md` (new).
- Steps:
  1. `git log --oneline -1` and record the base commit in §15 and the handoff.
  2. Confirm the header reads `DESIGN FROZEN` and §12 is filled.
  3. Create `handoff-s26b1.md` with the fields the working rules list (project/PR, primary design, binding docs,
     branch, base, HEAD, working-tree fingerprint, approved scope, frozen invariants, endpoint authority with
     sources, sequence B0–B11, budget, checkpoint, next actions, stop conditions).
  4. Record in the handoff the scratch root you will use for B7 and B10:
     `/Users/yuema137/mineworld-demos/cast-scratch/s26b1` (outside the repository; `mkdir -p` it).
- [ ] Implementation: steps 1–4.
- [ ] Validation: both doc checks.
- [ ] Review: no authority line widened; I-B1-2 (no spend) copied verbatim into the handoff.
- **Commit message:** `docs(plan): S26-b1 — open the ledger and handoff`.

#### B1 — Specification and decision records, before code

- **Goal.** The pipeline is specified and its dependencies recorded first (`CLAUDE.md` §2.2). **Files:**
  `tools/character_pipeline/README.md`, `tools/character_pipeline/PIPELINE.md`, `docs/DECISIONS.md`.
- Steps:
  1. Read `docs/DECISIONS.md` `DEP-8` (:267 onward, including the 2026-10-06 generated-meshes note and
     :374-375), `ARC-9` (:485), `ARC-23` (:1595), `ARC-24` (:1648), and `DEP-48` (:7930 on origin/main after #195; was :7860) as the record format.
  2. **Verify, with WebFetch, and record each answer with URL and date in §15:** (a) Meshy Terms of Use
     (`https://www.meshy.ai/terms-of-use`): the "last updated" date and §2.4's identifier clause, compared with
     D+ §4's quote; (b) Meshy rigging API (`https://docs.meshy.ai/en/api/rigging` or the current rigging page):
     the skeleton's bone count, whether it has finger bones, its credit price; (c) AccuRIG: whether a CLI or SDK
     exists (Reallusion's product page); (d) RigNet's licence (its GitHub repository) and UniRig's hardware
     requirement (its GitHub README); (e) Blender's licence page (`https://www.blender.org/about/license/`) on
     the status of outputs; (f) Winter's segment-length figure (§5.6). If (a)'s clause changed in substance, or
     (f) contradicts step-26's numbers, it is a **material stop**.
  3. Write `PIPELINE.md` as the specification (a coding agent's document, complete and precise): §1 purpose and
     the rules of step-26 §5.1; §2 the stage table of §5.2 with inputs and outputs per stage; §3 the command line
     of §5.3; §4 `fit.yaml`'s schema with every key, type, range, default (Vera's values) and D+ origin, and the
     override rule; §5 `generation_log.yaml`'s schema; §6 the gates of §5.6 with each bound and source, and the
     sentence that runtime gates G-7…G-13 come with S26-b2; §7 the parity comparison of §5.7; §8 provenance
     (§5.8) and the per-batch licence template; §9 secrets (§5.11) and the operator convention; §10 scratch and
     what is never committed (I-B1-4, I-B1-5, A-B1-4); §11 Blender: the pin, `--factory-startup`, the bake
     device, and the rule that a change under `blender/` re-runs the local Gate; §12 platforms; §13 the origin of
     the code: a table of every ported D+ script with its path in `/Users/yuema137/mineworld-demos/i23d/`, its
     SHA-256 (compute with `shasum -a 256` and paste), its line count, and its port; `rebake_head.py` dropped
     with D+ §8.4 step 7 as the reason; §14 the stored-once format (left "decided in B10" until then).
  4. Write `README.md` (human, short): one paragraph on what it is, the `check-characters` and `run` commands, a link to
     `PIPELINE.md`, the licence split (MIT driver, GPL Blender scripts).
  5. Append to `docs/DECISIONS.md`, in `DEP-48`'s format (Date 2026-10-14 · Status accepted (design, step-26) ·
     Approved by the operator's step-26 rulings of 2026-10-14 · Relates to · Design):
     - **`ARC-94`** — The per-character standard: step-26 §6 verbatim in substance (G-1…G-14 with bounds and
       sources; the scorecard rows; quality-tier parity with Vera as the control; the two review pages per batch,
       the runtime page parked for the big milestone (Q-S26-6); the failed-generation policy of §6.5). Options:
       per-character judgement only / gates without a scorecard / both (chosen). Consequences: the gates are code
       in `tools/character_pipeline/mwcast/gates.py`; a bound changes only by amending this record.
     - **`DEP-51`** — Meshy image-to-3D as the cast's mesh source: REST only, `ai_model` pinned per batch
       (`meshy-7.1` for Vera), `pose_mode a-pose`, no remesh, 2k texture, `remove_lighting`; ownership under the
       paid plan as `DEP-8` records; the Terms re-read on 2026-10-14 (step 2a) and at every batch's start
       (R-S26-11); downloads kept unmodified in scratch, never committed; a model change is a new pilot.
     - **`DEP-52`** — Blender, headless, as the mesh runtime: pinned `Blender 5.2.2 LTS`; scripts GPL-2.0-or-later
       in their own directory, outputs ours (step 2e); a tool, never distributed. Options include **"a pinned
       Blender in CI" — declined** with §5.10's four reasons; and "a stdlib GLB reader instead of pygltflib/trimesh
       for the gates — built" with §4's reason.
     - **`DEP-55`** — Rigging: our fitted CharMorph armature kept; Meshy rigging (step 2b's facts), Mixamo,
       AccuRIG (step 2c), UniRig/RigNet (step 2d) declined as the rig, each with step-26 §5.4's reason and the
       verified fact; Meshy rig evaluated in S26-b2 as a joint source only.
- [ ] Implementation: steps 1–5.
- [ ] Validation: both doc checks.
- [ ] Review: every term (`Presentation Pack`, `Person`, `System Pack`) used exactly; no rule only in the README;
  every **verify** of step-26 §§5.1, 5.4 that this PR owns is answered or named as S26-b2's.
- **Commit message:** `docs: the character pipeline's specification; ARC-94, DEP-51, DEP-52, DEP-55 (S26-b1)`.

#### B2 — `mwcast`: the GLB reader and the records

- **Goal.** Read what the gates read, and load what a character records. **Files:**
  `tools/character_pipeline/cast.py`, `tools/character_pipeline/ruff.toml`,
  `tools/character_pipeline/mwcast/{__init__.py, cli.py, glb.py, records.py, scratch.py}`,
  `tools/character_pipeline/tests/{conftest.py, gltf_fixture.py, test_glb.py, test_records.py}`.
- Steps:
  1. Read `clients/3d-spike/tools/check_slice_provenance.py` and `scripts/check_scratch.py` for the repository's
     stdlib-script style (docstring first, `from __future__ import annotations`, typed functions).
  2. `cast.py`: insert its own directory at `sys.path[0]`, import `mwcast.cli`, `raise SystemExit(main())`.
  3. `ruff.toml`: `line-length = 110`, `target-version = "py311"`, `[lint] select = ["E", "F", "I", "B", "UP"]`.
  4. `glb.py`: §5.5's functions. `GlbError` messages name the file and what was wrong (magic, version, chunk,
     accessor index).
  5. `records.py`: `load_record(path) -> object` (UTF-8, `json.loads`; a `JSONDecodeError` re-raised as
     `RecordError` with file, line and column); `FitParams` and `GenerationLog` as frozen dataclasses built by
     `parse_fit(obj, path)` and `parse_generation_log(obj, path)`; unknown keys and wrong types refused with the
     key path (e.g. `fit.yaml: weights.pack_panels.x_out: expected a number`); `"_why"` strings allowed where
     §5.4 shows them; `overrides(fit) -> list[str]` against the defaults table (a module-level dict that **is**
     PIPELINE.md §4's defaults).
  6. `scratch.py`: `resolve_scratch(arg: str | None, environ: Mapping[str, str], repo_root: Path) -> Path`, the
     §5.3 rule.
  7. `cli.py`: `main(argv: Sequence[str] | None = None) -> int` with subparsers for §5.3's subcommands; only
     those implemented so far are wired; others print "not yet implemented" and return 2 until their commit.
  8. `tests/conftest.py`: insert `tools/character_pipeline` into `sys.path`; an autouse fixture that replaces
     `socket.socket.connect` with a function raising `RuntimeError("network access in a unit test")`.
  9. `tests/gltf_fixture.py`: `build_glb(*, triangles: int, materials: list[str], joints: list[tuple[str, str |
     None]], joint_positions: dict[str, tuple[float, float, float]], weights: … , extras: dict[str, object],
     image_png: bytes | None) -> bytes` writing a valid GLB (positions, indices, JOINTS_0 as UNSIGNED_BYTE,
     WEIGHTS_0 as FLOAT, identity-rotation IBMs at the given positions).
  10. `test_glb.py`: a fixture GLB decodes to the given triangle count, joint names, parents and positions; an
      IBM with rotation and translation inverts to a hand-computed world position; a normalized UNSIGNED_SHORT
      accessor decodes to `value / 65535`; a truncated file raises `GlbError` naming the file; the committed
      `clients/3d-spike/assets/characters/meshy_d/meshy_d.glb` decodes to 95,000 triangles, 65,176 vertices,
      52 joints, material `MW_Meshy` (A-B1-14 — frozen evidence).
  11. `test_records.py`: a minimal valid `fit.yaml` parses; an unknown key, a string where a number belongs and a
      bad JSON line each raise `RecordError` with the key path or line; `overrides` of the defaults is `[]`; of a
      file with `flecks.enabled: false` is `["flecks.enabled"]`; `resolve_scratch` refuses a path inside a
      temporary git repository and refuses no argument with no environment variable.
- [ ] Implementation: steps 1–11.
- [ ] Validation: §9.0 step 5; mutation: make `triangle_count` count `indices` instead of `indices/3` →
  `test_glb` red → revert.
- [ ] Review: no third-party import in `glb.py`/`records.py`; every public function typed; no `Any` crossing the
  module boundary except the JSON document (`Mapping[str, object]`).
- **Commit message:** `feat(character-pipeline): GLB reader and character records (S26-b1)`.

#### B3 — Static gates G-1…G-6, G-14

- **Goal.** The standard as code with literal bounds. **Files:** `tools/character_pipeline/mwcast/{standard.py,
  gates.py, cli.py}`, `tools/character_pipeline/tests/test_gates.py`.
- Steps:
  1. `standard.py`: every literal of §5.6 and §5.7, each with a one-line source comment, plus `BLENDER_PIN =
     "Blender 5.2.2 LTS"` and `REFERENCE_SKELETON = "clients/3d-spike/assets/characters/vitruvian/vitruvian.glb"`.
  2. `gates.py`: `GateId`, `Verdict` (enums), `GateResult` (frozen dataclass), one function per gate as §5.6,
     and `run_static_gates(glb, fit, repo_root, run_dir: Path | None) -> list[GateResult]`. G-4(b) computes the eye
     height from `fit.fit.picks.eye` through the transform in `run_dir/gameready.json`; without a run directory it
     reports FAIL with the reason "needs a run directory" (a gate that cannot look does not pass). G-5's roll check reads `run_dir/dump_rig.json` when given, else reports
     `rolls: not checked (no run directory)` in `looked_at` and judges names and parents only.
  3. `cli.py`: wire `gate` (prints each result: gate, `looked at …`, verdict, reasons; `--json` prints a list)
     and `check-characters` (§5.3).
  4. `test_gates.py`: per gate, one PASS and one FAIL on fixture GLBs: X-5 (100,001 vs 100,000 triangles), X-6
     (two materials), X-7 (`mixamorig:LeftForeArm` renamed), G-3 (a 1024² PNG fails), G-4 (soles 6 mm low fail),
     G-6a (a shank 15 % short fails), G-6b (a `Hips` joint 0.3 m behind the body fails and prints the joint), G-14
     (missing `meshy_task`; a provenance string `msy_` + 24 letters fails). **Frozen evidence:** `meshy_d.glb`
     with Vera's stature 1.767: G-1 PASS reporting 95000; G-2 FAIL whose reason contains `MW_Meshy`; G-5 PASS
     (names and parents); G-6a PASS; G-14 FAIL whose reasons name exactly `references`, `reference_generator`,
     `pipeline` (A-B1-14, F-B1-8). A test that every function in `gates.py` taking a `GateResult`-producing role
     has no parameter named like a bound (`max_*`, `tol*`, `bound*`) — the structural half of X-9.
- [ ] Implementation: steps 1–4.
- [ ] Validation: §9.0 step 5; mutations: (a) `MAX_TRIANGLES = 100_001` → the X-5 test red; (b) G-5 compares
  names only (parents ignored) → a fixture with a re-parented bone stays green? It must turn red: add that
  fixture case; (c) G-14 skips the key scan → its test red. Revert each.
- [ ] Review: X-9 — read every gate: no bound from the asset; each prints where it looked before the verdict
  (`ARC-23` rule 1).
- **Commit message:** `feat(character-pipeline): static gates G-1…G-6 and G-14 with literal bounds (S26-b1)`.

#### B4 — The secrets scan and CI wiring

- **Goal.** AC-S26-9's scan, and the pipeline's checks in CI on three OSes. **Files:** `scripts/check_secrets.py`
  (new), `scripts/ci_layer.py`, `pyproject.toml`.
- Steps:
  1. Read `scripts/ci_layer.py` in full and `.github/workflows/ci.yml:205-254`.
  2. Write `scripts/check_secrets.py` (§5.11): `scan` and `--self-test`; docstring stating the rule and its
     source (step-26 §5.5, AC-S26-9).
  3. `ci_layer.py`: (a) in `"fast"`, after `["python3", "scripts/check_scratch.py", "scan"]` (:53), add
     `["python3", "scripts/check_secrets.py", "--self-test"]` and `["python3", "scripts/check_secrets.py",
     "scan"]`; (b) after `PYTHON_STATIC`'s definition (:185-190), add `PIPELINE = "tools/character_pipeline"` and
     extend `PYTHON_STATIC` with `["uv", "run", "--locked", "ruff", "check", PIPELINE]`, `["uv", "run",
     "--locked", "ruff", "format", "--check", PIPELINE]`, `["uv", "run", "--locked", "pyright",
     f"{PIPELINE}/mwcast", f"{PIPELINE}/tests", f"{PIPELINE}/cast.py"]`, with a comment: the pipeline is not a
     workspace member (no dependencies, no lock entry), it is checked by path; (c) in `LAYERS["python"]` and
     `LAYERS["python-smoke"]`, add `["uv", "run", "--locked", "pytest", f"{PIPELINE}/tests"]` and
     `[sys.executable, f"{PIPELINE}/cast.py", "check-characters"]`.
  4. `pyproject.toml`: add to `[tool.pyright]` an `executionEnvironments` entry
     `{ root = "tools/character_pipeline", extraPaths = ["tools/character_pipeline"] }` so `import mwcast`
     resolves in the tests; leave `include` unchanged. Run `uv sync --locked` to prove the lock is unaffected.
  5. `python3 scripts/ci_layer.py --list fast` and `--list python`: the new commands are listed in place.
- [ ] Implementation: steps 1–4.
- [ ] Validation: step 5; `python3 scripts/check_secrets.py --self-test`; `python3 scripts/check_secrets.py
  scan` (0 findings, the count printed); `uv sync --locked` exit 0; §9.0 step 5; mutation: remove the `msy_`
  pattern → self-test red → revert.
- [ ] Review: no existing command changed or reordered; the scan's allowlist is empty; no key pattern matches
  this design document (it shows prefixes only, never a full key-shaped string).
- **Commit message:** `ci: secrets scan and the character pipeline's checks on three platforms (S26-b1)`.

#### B5 — Stage 4: the Meshy client and the identifier inspection

- **Goal.** `meshy_i23d.py` ported with a credit cap, redaction and an injected transport; `inspect_ids.py`
  ported. No live call. **Files:** `tools/character_pipeline/mwcast/{meshy.py, identifiers.py, cli.py}`,
  `tools/character_pipeline/tests/test_meshy.py`.
- Steps:
  1. Read `/Users/yuema137/mineworld-demos/i23d/meshy_i23d.py` and `inspect_ids.py` in full.
  2. `meshy.py`: `Transport` protocol `request(method: str, url: str, headers: Mapping[str, str], body: bytes |
     None, timeout_s: float) -> tuple[int, bytes]`; `UrllibTransport` (the real one); `DEFAULT_PARAMS` = A-B1-5's
     values plus `pose_mode "a-pose"`; `CREDITS_PER_RUN = 30` (D+ §3; the measured `consumed_credits` is recorded
     per run); `MeshyRun.create(...)` writes `request.json` (endpoint, the input's absolute path and SHA-256,
     params, date; never bytes or headers), polls every 15 s, writes `task.json`, downloads the files exactly as
     `meshy_i23d.py:83-94`, records balance before and after; a credit ledger `<scratch>/credits.json` (runs,
     credits spent) and a cap: refuse before the POST when `spent + CREDITS_PER_RUN > cap`; an HTTP error body
     truncated to 2,000 chars and the key's value replaced by `<redacted>`.
  3. `identifiers.py`: `inspect(path) -> list[Finding]` for glTF `asset`/`extras`/`extensions`, PNG text/iTXt/
     zTXt/caBX/eXIf chunks, JPEG APP segments with XMP or JUMBF, and any occurrence of `meshy` (case-insensitive).
  4. `cli.py`: wire `meshy` (with `--dry-run`) and `identifiers`.
  5. `test_meshy.py` with a fake transport: the POST body's JSON has the params and an `image_url` data URI, the
     headers carry the key but no stored file does (scan every file the run wrote for the fake key string); a 401
     whose body echoes the key stores and prints `<redacted>`; a cap of 29 refuses before any request; a cap of 60
     after one run of 30 allows one more and refuses the third; `--dry-run` makes zero transport calls;
     `MESHY_API_KEY` absent → the refusal message; `identifiers` on the committed `meshy_d.glb` finds the
     `mineworld_provenance` extras and no `meshy` occurrence outside them (D+ §4's "nothing found").
- [ ] Implementation: steps 1–5.
- [ ] Validation: §9.0 step 5; mutation: remove the redaction → the 401 test red → revert.
- [ ] Review: no code path reads a file for the key; `UrllibTransport` is the only network code and no test
  constructs it.
- **Commit message:** `feat(character-pipeline): Meshy client with credit cap and redaction; identifier inspection (S26-b1)`.

#### B6 — Stage 3: the mask

- **Goal.** `prep_inputs.py` ported, parameterized, crop box automatic by default. **Files:**
  `tools/character_pipeline/mwcast/{mask.py, texture.py, cli.py}`, `tools/character_pipeline/requirements.txt`,
  `tools/character_pipeline/tests/test_pipeline.py` (new; the crop rule only).
- Steps:
  1. Read `/Users/yuema137/mineworld-demos/i23d/prep_inputs.py` in full.
  2. `mask.py`: `alpha_bbox(alpha_rows: Sequence[bytes], threshold: int) -> tuple[int, int, int, int]` (pure, no
     numpy); `run_mask(fit, reference, out)` importing `rembg`, `PIL`, `numpy` inside the function: crop to
     `fit.mask.crop_box` when given, else to the alpha bounding box of a first full-image pass; remove; square-pad
     with `fg_ratio`; resize to `size_px` with LANCZOS; save RGBA PNG; print the box and alpha coverage as D+ did.
  3. `texture.py`: `compare_texels(a: Path, b: Path) -> TexelReport` (numpy/PIL inside), §5.7's rule; and
     `compare_masks(a: Path, b: Path) -> MaskReport` (alpha IoU at alpha > 127, RGB mean absolute difference over
     the union foreground), exposed as `compare-texture --alpha-iou` (AB1-13).
  4. `requirements.txt`: §5.1's pins, with a header comment "tooling only; never installed by CI or any runtime".
  5. `cli.py`: wire `mask` and `compare-texture`.
  6. `test_pipeline.py`: `alpha_bbox` on a hand-made 6×5 alpha grid returns the hand-computed box; a fully
     transparent grid raises `ValueError`.
- [ ] Implementation: steps 1–6.
- [ ] Validation: §9.0 step 5 (CI has no rembg: importing `mwcast.mask` must not import rembg at module level —
  the tests prove it by importing the module).
- [ ] Review: no third-party import at module level in `mask.py` or `texture.py`.
- **Commit message:** `feat(character-pipeline): stage 3 mask with an automatic crop; texel comparison (S26-b1)`.

#### B7 — The Blender stages, ported

- **Goal.** Stages 6–10 and the support renders as parameterized GPL scripts, numerically identical to D+.
  **Files:** `tools/character_pipeline/blender/{LICENSE, README.md, common.py, gameready.py, fit_bind.py,
  weights_fingers.py, weights_pack.py, flecks.py, export.py, render_ortho.py, render_bones.py, dump_rig.py}`.
- Steps:
  1. Read each D+ source in full: `gameready_mesh.py`, `rig_meshy.py`, `reweight_fingers.py`, `reweight_pack.py`,
     `fix_flecks.py`, `render_ortho.py`, `render_rig.py`, `dump_bones.py` (all in
     `/Users/yuema137/mineworld-demos/i23d/`), and `clients/3d-spike/tools/blender/README.md`.
  2. `LICENSE`: copy `clients/3d-spike/tools/blender/LICENSE` byte for byte (`cp`), then `cmp` the two.
  3. Every script starts with `#!/usr/bin/env python3`, `# SPDX-License-Identifier: GPL-2.0-or-later`, a docstring
     naming its D+ source and its stage, and imports `common` from its own directory as `character_model.py` does.
  4. `common.py`: `params() -> dict` (the JSON after `--`), `write_result(name, data)` (writes `<run_dir>/<name>.json`
     and prints each item), `EXPORT_SETTINGS` (A-B1-8 :259-262, verbatim), `set_provenance(dict)`.
  5. **Port rule for every stage script:** the D+ code is kept statement for statement, in the same order; each
     typed constant becomes `p["…"]` from `params.json` (the keys of §5.4), each environment variable becomes a
     params key, each print stays and is also written to the result JSON; nothing else changes. Specifically:
     - `gameready.py`: inputs `download_glb`, `out_dir`, `fit.gameready`; output `gameready.blend` and
       `gameready.json` with `scale`, `sole_raw`, `cx`, `cy`, `triangles`, `vertices`, `parts` (sorted, top six),
       `non_manifold_edges`, `bake_device` (the device actually used). Material name stays `MW_Meshy` here (the
       export renames it).
     - `fit_bind.py`: inputs `gameready.blend` (opened by Blender), `gameready.json`, `fit.fit`, `fit.weights`,
       `fit.accessories`; replaces `S, SOLE_M, SOLE_Z` with `gameready.json`'s `scale`, `sole_raw` and
       `fit.gameready.sole_z_m`; imports the armature from `fit.fit.rig_source` and removes every imported mesh;
       for the `Hips` centre point uses `fc = fit.fit.charmorph_front_y_at_hips` instead of `front_y(cm_co, …)`,
       and keeps the `front_y(co, z)` existence check on the new body for every centre joint; applies the pack
       core only when `fit.accessories` contains `"backpack"`; output `fit_bind.blend`, `fit_bind.json` (every
       printed joint, the shoulder width ratio, the heat-weight group count and unweighted count, each fix's
       vertex count).
     - `weights_fingers.py`, `weights_pack.py`, `flecks.py`: inputs the previous `.blend`, their `fit` block;
       outputs `<stage>.blend` and `<stage>.json`; `weights_pack.py` and `flecks.py` are skipped by the driver
       when not applicable (no pack; `flecks.enabled` false), and the skip is written to the run manifest.
     - `export.py`: opens the last `.blend`; renames the material to `fit.export.material`; names the image
       datablock `<id>_base_color`; sets `mineworld_provenance` (§5.8, built by the driver into `params.json`);
       exports `<id>.glb` with `EXPORT_SETTINGS` and `GLTF_SEPARATE` `<id>.gltf` with the same settings; writes
       `export.json` with both paths and SHA-256 of each file.
     - `render_ortho.py`, `render_bones.py`, `dump_rig.py`: ports of their D+ sources with arguments from
       `params.json`; `dump_rig.py` writes `dump_rig.json` (`{bone: {head, tail, roll, parent}}`) for G-5, and, when
       `params.json` has `"front_y": {"glb": …, "mesh": "cm_vitruvian", "z": …, "half_x": 0.03, "band": 0.01}`,
       also the minimum Blender-`y` of that mesh's vertices with `|x| < half_x` and `|z − z| < band`, with the
       vertex count (B10 step 1's measurement, `rig_meshy.py:60-62`'s `front_y`).
  6. `README.md`: the licence boundary (`clients/3d-spike/tools/blender/README.md`'s wording, adapted).
  7. Smoke (real Blender, not a parity claim): `blender --background --factory-startup --python-exit-code 1
     --python tools/character_pipeline/blender/dump_rig.py -- <scratch>/smoke/params.json` with a params file
     naming `vitruvian.glb` (write the params file with the Write tool); expect 52 bones and exit 0.
- [ ] Implementation: steps 1–6.
- [ ] Validation: step 7, recorded; `uv run --locked ruff check tools/character_pipeline/blender`; `cmp` of the
  LICENSE; a diff review of each port against its source (record per script: "statements unchanged except the
  listed parameter reads").
- [ ] Review: I-B1-9 — search the scripts for any number that is per character (every literal is either a
  method constant listed in PIPELINE.md or a params read); I-B1-8 — nothing imports these scripts.
- **Commit message:** `feat(character-pipeline): the D+ Blender stages, ported and parameterized (GPL-2.0-or-later) (S26-b1)`.

#### B8 — The driver: run and compare

- **Goal.** One command runs stages 6–10 and 12's frames; one compares. **Files:**
  `tools/character_pipeline/mwcast/{pipeline.py, blender.py, compare.py, cli.py}`,
  `tools/character_pipeline/tests/{test_pipeline.py, test_compare.py}`.
- Steps:
  1. `blender.py`: `find_blender(environ, platform) -> Path`; `blender_version(path, runner) -> str` (first line of
     `--version`); `check_pin(version, allow: str | None) -> None`; `command(blender, script, params, in_blend:
     Path | None) -> list[str]` (§5.2's argument list).
  2. `pipeline.py`: `STAGES = ("gameready", "fit_bind", "weights_fingers", "weights_pack", "flecks", "export",
     "render_bones", "dump_rig")`; `plan(fit, from_stage, to_stage) -> list[str]` (drops `weights_pack` without a
     pack, `flecks` when disabled); `write_params(stage, fit, run_dir, prior_results) -> Path`; `run(...)`: for each
     stage, build the command, run it with `subprocess.run(..., check=False)`, write stdout/stderr to
     `<run_dir>/<stage>.log`, stop on a non-zero exit naming the stage and the log; then `run_manifest.json`
     (§5.3). The download's SHA-256 is checked against `fit.source.download_sha256` before stage 6; a mismatch
     stops the run.
  3. `compare.py`: §5.7 over two `Glb`s, returning a `ParityReport` with one row per quantity, its worst offender
     and its verdict; `cli.py` wires `run` and `compare`.
  4. `test_compare.py`: identical fixtures pass; one joint moved 1.1 mm fails naming it, 0.9 mm passes; one weight
     moved by 0.006 fails, 0.004 passes; a swapped vertex pair fails with "vertex order differs"; different
     triangle counts fail first.
  5. `test_pipeline.py` (extend): `plan` for Vera's fit is `gameready, fit_bind, weights_fingers, weights_pack,
     export, render_bones, dump_rig`; for a fit without a pack and with flecks on it drops `weights_pack` and keeps
     `flecks`; `command` builds the exact list of §5.2 (with and without an input `.blend`); `check_pin` refuses
     `Blender 5.2.1` and accepts it with `allow`; a download whose SHA-256 differs stops before any subprocess
     (a fake runner records zero calls).
- [ ] Implementation: steps 1–5.
- [ ] Validation: §9.0 step 5; mutation: `PARITY_JOINT_M = 0.002` → the 1.1 mm test red → revert.
- [ ] Review: `subprocess` only with argument lists; no stage reads an environment variable; the manifest records
  every input's hash.
- **Commit message:** `feat(character-pipeline): stage runner and parity comparison (S26-b1)`.

#### B9 — Vera's recorded inputs, the import template, the Godot check

- **Goal.** The protagonist's parameters as data. **Files:**
  `presentation/mineworld-default/3D/characters/cast/vera-lindgren/{fit.yaml, generation_log.yaml}`,
  `tools/character_pipeline/godot/{character.gltf.import, inspect_import.gd}`,
  `tools/character_pipeline/templates/MESHY_CAST_BATCH.txt`, `tools/character_pipeline/tests/test_records.py`.
- Steps:
  1. Write `fit.yaml` exactly as §5.4, with `charmorph_front_y_at_hips` as **−0.045356** for now (B10 replaces it
     with the Blender measurement if they differ by ≤ 1e-5; more is a stop).
  2. Write `generation_log.yaml` as §5.4 describes, from D+ §3 and `apose_v3/task.json` (`consumed_credits`).
  3. `character.gltf.import`: `[remap] importer="scene" type="PackedScene"` and `[params]` copied from
     `clients/3d-spike/assets/characters/meshy_d/meshy_d.glb.import` (every line from `nodes/root_type` to
     `gltf/texture_map_mode`), with the bone map path unchanged; no `uid`, no `path`, no `[deps]`.
  4. `inspect_import.gd`: extends `SceneTree`; `_init()` loads the scene named on the command line after `--`,
     instantiates it, prints `surfaces=`, `material=`, `albedo=<resource_path>`, `bones=` and the first three bone
     names, then lists `res://` files containing `base_color` and quits.
  5. `MESHY_CAST_BATCH.txt`: `MESHY_ROUTE_D_CHARACTER.txt`'s structure with placeholders (`<batch>`, `<character
     ids>`, `<task ids>`, `<terms date>`), and the identifier paragraph kept verbatim in substance.
  6. `test_records.py` (extend): the committed Vera `fit.yaml` and `generation_log.yaml` parse; `overrides` is
     `["flecks.enabled"]`; the log totals are 3 runs, 90 credits, 1 kept; `check-characters` on the repository prints
     the `records only` line and returns 0.
- [ ] Implementation: steps 1–6.
- [ ] Validation: §9.0 step 5; `python3 tools/character_pipeline/cast.py check-characters`.
- [ ] Review: every number in Vera's `fit.yaml` traced to its D+ line (record the trace table in §15); no URL from
  `task.json` copied.
- **Commit message:** `feat(cast): Vera's recorded pipeline inputs; Godot import template and check (S26-b1)`.

#### B10 — Gate 2: re-derive Vera

- **Goal.** AB1-7…AB1-13: the port reproduces the accepted protagonist from her recorded inputs, spending nothing.
  **Files:** `presentation/mineworld-default/3D/characters/cast/vera-lindgren/{fit.yaml (only the measured
  constant, if it moved), rederive_report.json}`, `tools/character_pipeline/PIPELINE.md` (§14 decided), this file.
- **Gate specification (record in §15 before running):** claim — stages 6–10 reproduce `meshy_d.glb` within §5.7;
  owner Gate 2; HEAD = B9's commit; inputs `/Users/yuema137/mineworld-demos/i23d/out/meshy/apose_v3/` (read only),
  scratch `S=/Users/yuema137/mineworld-demos/cast-scratch/s26b1`; budget ≤ 30 min wall, 0 credits, 0 dollars;
  PASS = AB1-7…AB1-11 all hold; FAIL = any bound missed with the stage localized; INCONCLUSIVE = a stage did not
  run (Blender error, missing input).
- Steps:
  1. **Measure the rig constant.** Write `S/measure/params.json` naming
     `/Users/yuema137/mineworld-worktrees/scratch-character/s3/body_g5.glb` and run `dump_rig.py` with a
     `front_y_at` request for z 1.0048 (`dump_rig.py` supports `"front_y": {"mesh": "cm_vitruvian", "z": …,
     "half_x": 0.03, "band": 0.01}` for this purpose); also dump the armature of `vitruvian.glb` and of
     `body_g5.glb` and compare bone heads, tails and rolls (≤ 1e-5 m, ≤ 1e-5 rad), and compare with
     `/Users/yuema137/mineworld-demos/i23d/rig/cm_bones.json`. Record all three. If the front value differs from
     −0.045356 by > 1e-5, write the measured value into `fit.yaml`; if the armatures differ beyond the bound, stop
     (F-B1-5's premise false).
  2. **Run:** `python3 tools/character_pipeline/cast.py run --fit
     presentation/mineworld-default/3D/characters/cast/vera-lindgren/fit.yaml --download
     /Users/yuema137/mineworld-demos/i23d/out/meshy/apose_v3 --scratch S --run-id r1 > S/r1.log 2>&1`. Read the log.
  3. **Stage-by-stage parity** (localizes any drift; record each): stage 6 counts against AB1-7; `fit_bind`'s
     printed joints against the D+ prints recorded in D+ §8.2 and, by `compare`, `S/vera-lindgren/r1/` exported
     from `fit_bind.blend` against `/Users/yuema137/mineworld-demos/i23d/rig/meshy_char.glb` (run `export.py` on
     that intermediate with a params file pointing at it); likewise `weights_fingers` against
     `i23d/rig4/meshy_char.glb`; the final `<id>.glb` against both `i23d/rig7/meshy_char.glb` and
     `clients/3d-spike/assets/characters/meshy_d/meshy_d.glb` (identical files, A-B1-1).
  4. **Texture:** with the tool venv (`/Users/yuema137/mineworld-demos/i23d/venv/bin/python`), `compare-texture`
     of `S/vera-lindgren/r1/export/vera-lindgren_base_color.png` against
     `clients/3d-spike/assets/characters/meshy_d/meshy_d_base_color.png` (AB1-9).
  5. **Gates:** `python3 tools/character_pipeline/cast.py gate S/vera-lindgren/r1/export/vera-lindgren.glb --fit
     <fit> --run-dir S/vera-lindgren/r1` (AB1-10).
  6. **Stored once:** the §5.9 procedure in `S/godot-check/`; record the printed lines and the decision; write
     `PIPELINE.md` §14.
  7. **Mask (AB1-13):** with the tool venv, `cast.py mask --fit <fit> --reference
     presentation/mineworld-default/3D/references/04_character_closeup.png --out S/mask/in_chest_rgba.png`; compare
     with `/Users/yuema137/mineworld-demos/i23d/work/in_chest_rgba.png` (alpha IoU and RGB difference, computed by
     `compare-texture --alpha-iou`, which B6's `texture.py` provides).
  8. **Counterfactual (AB1-12, XB1-1):** copy `fit.yaml` to `S/x1/fit.yaml` with `l_shoulder` `[611, 228]`; `run
     --from fit_bind` reusing r1's `gameready.blend`; `compare` must FAIL naming `mixamorig:LeftArm`.
  9. **Determinism (AC-S26-3):** `run` again as `r2` from stage 6; `compare r2 r1` passes with every difference 0
     (record the maxima).
  10. Write `rederive_report.json`: date, HEAD, Blender version, bake device, input hashes, every comparison row
      (§5.7) with value and bound, each gate's verdict, the G-7…G-13 note of §8, the stored-once decision, the mask
      result, the counterfactual, the determinism maxima.
- [ ] Implementation: steps 1, 6 (PIPELINE.md §14), 10.
- [ ] Validation: steps 2–9, each PASS / FAIL / INCONCLUSIVE with evidence in §15.
- [ ] Review: was any bound changed after a run? (none may be); is every claim in the report backed by a log line?
- **Gate FAIL policy:** do not touch a bound. Localize with step 3 to the first stage that diverges; read that
  stage's port against its D+ source; a port defect is fixed in `blender/` (a new commit "fix(character-pipeline):
  …") and the Gate re-run; a non-reproducible Blender behaviour (e.g. the bake device) is recorded and raised to
  the primary session with the numbers (material stop if AB1-8 cannot hold).
- **Commit message:** `test(cast): re-derive Vera from her recorded inputs — Gate 2 evidence (S26-b1)`.

#### B11 — Close-out

- **Goal.** Documents reconciled; the PR opened. **Files:** `docs/references/CHARACTER_ROUTE_D_PLUS.md` (a dated
  note at the end of the header table or §8.9: "2026-10-14: the scripts are versioned and parameterized in
  `tools/character_pipeline/` (S26-b1); this record remains the history"), this file (§15), the handoff.
- Steps:
  1. Add the note; nothing else in the D+ record changes.
  2. AB1-15: `git diff --stat origin/main...HEAD`; `git diff --name-only origin/main...HEAD` checked against I-B1-1
     and I-B1-4 (no `.glb`, `.png`, `.blend`, `.bin`).
  3. Update §15 and the handoff; commit.
  4. `git push -u origin mvp0/pr-s26b1`; `gh pr create --title "feat(character-pipeline): route D+ in the
     repository (S26-b1)" --body …` (what changed, the Gate evidence summary, spending 0/0, the ruled OpenAI cap
     of $150 (Q-B1-3), any finding raised during implementation). Wait for CI on the exact head; repair routine failures; READY FOR
     OPERATOR REVIEW — DO NOT MERGE.
- [ ] Implementation: steps 1–4.
- [ ] Validation: both doc checks; CI on the final head.
- [ ] Review: no claim in the PR body without evidence in §15.
- **Commit message:** `docs(plan): S26-b1 close-out; route D+ record points to the pipeline`.

## 10. Requirements this PR places on other PRs

| PR | Requirement |
| --- | --- |
| S26-b2 | Uses `cast.py run`, the gates and `fit.yaml`; adds landmark detection (replacing `picks.frame: "dplus-v1"`, F-B1-2, F-B1-3), the runtime runners G-7…G-13, G-6's 30 mm cross-check; spends only under the confirmed caps; re-runs B10's Gate when it edits `blender/` |
| S26-c1 | Brings Vera into `characters/cast/vera-lindgren/` with a `character.yaml` whose asset is the **re-derived stored-once export** (r1 of B10, or a re-run), replacing `meshy_d.glb` and its duplicated PNG (A-7), proven unchanged by AC-S26-6's frame diff; rebinds `human.gd`'s surface response from `MW_Meshy` to `MW_Cast` (A-B1-17); Vera's G-14 then passes in CI |
| S26-a | No dependency. S26-a's design (merged as a design, #195) adds `characters/cast.yaml` and `scripts/check_cast.py`, which checks that binding file's structure. This PR's per-character checker is `cast.py check-characters`, renamed from `check-cast` in revision 2 so that the two are not confused. It reads `characters/cast/<id>/` and never `cast.yaml`. S26-a's `default: "vera-lindgren"` names the directory that this PR starts with records only |
| S26-f | Sets the cast texture import policy in the per-PNG `.import` (the stored-once format, §5.9) |

## 11. Edits to parent documents (owner: the primary session, after freeze or merge)

- `step-26-cast.md` §12, S26-b1 row: record F-B1-1's reading of the integration check (ruled, Q-B1-1).
- `step-26-cast.md` §10.2: the OpenAI cap of $150 for the whole step (ruled by the operator, Q-B1-3).
- `overall.md`: S26 progress after merge.

## 12. Execution contract (to be confirmed at freeze)

```text
PROJECT / PR:            MineWorld mvp0, S26-b1 — route D+ in the repository (the character pipeline)
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s26b1-pipeline.md
RELATED / BINDING DOCS:  step-26-cast.md (§§5, 6, 10, 12, 12.1, 12.2, 15, 16.1); docs/references/
                         CHARACTER_ROUTE_D_PLUS.md; docs/DECISIONS.md DEP-8, ARC-9, ARC-17, ARC-19, ARC-23, ARC-24;
                         docs/HUMANOID_PROFILE.md §§1–4; tools/asset_generation/README.md (the tooling precedent);
                         clients/3d-spike/tools/blender/README.md (the GPL boundary); CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md; docs/REUSE_POLICY.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s26b1, its own, held by one session
BRANCH:                  mvp0/pr-s26b1, created from origin/main
START CONDITION:         this design frozen; nothing else (step-26 §12.1)
IMPLEMENTER:             Haiku (step-26 §12.1); Opus review of B3, B7 and B10 before the PR is marked ready
IMPLEMENTATION BASE:     origin/main at B0; B0 records the commit
APPROVED SCOPE:          §2.1 items 1–9
FROZEN INVARIANTS:       §6 I-B1-1 … I-B1-9; §8's bounds (§5.6, §5.7) unchanged after any run
SEQUENCE:                B0 … B11 (§9)
ALLOWED COMMANDS:        git; gh (never merge); uv run --locked *; uv sync --locked; python3 scripts/*;
                         python3 tools/character_pipeline/cast.py *; the tool venv's python
                         (/Users/yuema137/mineworld-demos/i23d/venv/bin/python) for mask and compare-texture;
                         blender (5.2.2 only) and godot (4.7.2) in B7 step 7 and B10 only; shasum; cmp; cp; ls;
                         mkdir -p; sed -n; WebFetch in B1 step 2 only
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; sourcing the secrets file or reading
                         any key or .env file; `cast.py meshy` without --dry-run; any OpenAI request; writing into
                         /Users/yuema137/mineworld-demos/i23d/ or scratch-character/ (read only); committing any
                         binary; edits outside APPROVED SCOPE; changing a bound after a Gate run
MATERIAL STOPS:          B1 step 2: Meshy's §2.4 changed in substance, or Winter's figure contradicts step-26's
                         numbers; B10: the armatures of vitruvian.glb and body_g5.glb differ beyond 1e-5; AB1-8 cannot
                         hold after a localized port review (a Blender non-determinism); the stored-once rule of §5.9
                         fails (the fallback touches Q-S26-5); any need to spend a credit or a dollar; any edit under
                         a forbidden directory (I-B1-1)
PLATFORMS:               CI: Linux, macOS, Windows (python job) and Linux (fast). Local Gate: macOS (the operator's
                         Mac, Metal)
VALIDATION BUDGET:       unit tests unrestricted locally; B7 smoke ≤ 2 min; B10 Gate ≤ 30 min wall in total (two
                         full runs, one partial, the Godot check, the mask); 0 Meshy credits; $0 OpenAI; CI: the PR's
                         runs. Gate 1: NOT REQUIRED (no LLM-facing change)
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s26b1.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once frozen        (source: to be the
                                                                                   primary's freeze relay)
                         semantic commits: authorized                     (working rules §14)
                         branch push: authorized                          (working rules §21)
                         PR creation / update: authorized                 (working rules §21)
                         CI repair to review readiness: authorized        (working rules §21)
                         merge: never by the implementation session; only with the operator's explicit
                         authorization (working rules §22; coordinator brief 2026-10-14)
POST-MERGE SYNC OWNER:   primary session: step-26, overall.md; implementation session: this ledger
STOP CONDITION:          S26-b1 READY FOR OPERATOR REVIEW — DO NOT MERGE
```

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-B1-1 | Decimation or heat weights are not bit-reproducible across Blender processes | B10 step 9 measures it (r1 vs r2); the D+ intermediates localize the first diverging stage; the bound is not loosened |
| R-B1-2 | The bake differs across devices | the texel bound applies on one device only (§5.7); the device is recorded; non-Metal runs are new baselines |
| R-B1-3 | `--factory-startup` changes a default the D+ runs relied on | B10 step 3 compares stage by stage; if stage 6 or 7 diverges, the first experiment is a run without the flag, recorded |
| R-B1-4 | The port drifts from D+ by "tidying" | B7's port rule (statement for statement), its diff review, and B10's stage parity |
| R-B1-5 | A Haiku implementer cannot debug a Blender divergence | B10's FAIL policy stops with the localized stage; Opus review is in the contract |
| R-B1-6 | The CI scan flags a legitimate string later (a test fixture) | fixtures build key-shaped strings at run time from parts; the allowlist needs a reason per entry |
| R-B1-7 | pyright strict over a non-member path does not resolve `mwcast` | B4 step 4's `executionEnvironments`; verified by the pyright command before commit |

## 14. Operator questions (each with a recommendation; none re-asks a step-26 ruling)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-B1-1 [primary — reading of a frozen checkpoint]** | The S26-b1 row's integration check says "G-1…G-14 pass", while the runtime runners G-7…G-13 are S26-b2's scope. Accept: static G-1…G-6 and G-14 run on the re-derived Vera, G-7's import half runs (§5.9), and G-8…G-13 are established by parity with the operator-accepted asset (AB1-8), recorded as "by parity", not as PASS? | **Yes.** With identical triangles, joints within 1 mm and weights within 0.005, the runtime gates measure the same deformation the operator accepted on 2026-10-07; running them belongs with the runners S26-b2 builds |
| **Q-B1-2 [operator — repository content]** | The rig source: (a) the committed `vitruvian.glb` armature plus one recorded CharMorph number (`charmorph_front_y_at_hips`), or (b) commit CharMorph's CC0 body export `body_g5.glb` (6.4 MB) under `tools/character_pipeline/rig_source/`? | **(a) now.** It reproduces D+ exactly (A-B1-9) and adds no binary. S26-b2's weight-transfer prototype needs the CharMorph body's own weights; if it is adopted (`DEP-53`), that PR asks for (b) with its measured need |
| **Q-B1-3 [primary — spend, Q-S26-3]** | Confirm the OpenAI cap for the whole of S26: **$150** (≈ $0.24 per 1024×1536 high image with up to four references, upper estimate from the price read 2026-10-14; 600 images at step-26 §10.2's ceiling ≈ $144), reported at 50 % and 100 %, hard stop? | **Confirm $150.** This PR generates nothing; the pilot replaces the estimate with measured `cost_usd_estimate` values before S26-cast |
| **Q-B1-4 [primary — CI]** | No Blender in CI (§5.10): the Blender stages are proven by the local Gate on the pinned Blender, re-run whenever `blender/` changes; CI owns the stdlib gates, records, driver logic and the scan on three OSes | **Yes**, for §5.10's four reasons; recorded in `DEP-52` |
| **Q-B1-5 [primary — scope detail]** | The stage-9 fleck repair: on by default for new characters (step-26 §5.2), off for Vera because her accepted texture never had it (F-B1-4)? | **Yes.** Re-deriving her must reproduce the accepted asset; the pilot shows whether the repair helps a new character |

### 14.1 Rulings, 2026-10-14 (relayed by the coordinator; binding)

| Question | Ruling | By |
| --- | --- | --- |
| Q-B1-1 | **Yes.** Static gates and G-7's import half run here; G-8…G-13 are recorded "by parity" with the operator-accepted asset, never as PASS | primary session |
| Q-B1-2 | **Yes.** The `vitruvian.glb` armature plus the one recorded number; `body_g5.glb` is not committed | primary session |
| Q-B1-3 | **$150 OpenAI cap for the whole of S26**, reported at 50 % and 100 %, hard stop at the cap. Answered directly in the primary session: "150 美元上限" | operator |
| Q-B1-4 | **Yes.** No Blender in CI; the local Gate is re-run whenever the Blender scripts change | primary session |
| Q-B1-5 | **Yes.** Fleck repair on by default, off for Vera | primary session |

**Audit re-check after these rulings** (origin/main after #195 (S26-a design), DL-a, FG-a, 25b, SET-b, merged
into this branch): `scripts/ci_layer.py` (:48, :53, :184-213), `pyproject.toml` `[tool.pyright]` (:13-17),
`.github/workflows/ci.yml` (`python` job :210), `human.gd` (:35, :243, :342), `docs/DECISIONS.md` `DEP-8` (:267),
`ARC-9` (:485), `ARC-23` (:1595), `ARC-24` (:1648) and the rembg note (:374) are unchanged; `DEP-48` moved from
:7860 to :7930 (B1 step 1 updated). No decision heading `ARC-94`, `DEP-51`, `DEP-52` or `DEP-55` exists yet.
Nothing under `presentation/mineworld-default/3D/`, `clients/3d-spike/assets/characters/`, `clients/3d-spike/tools/`
or `tools/asset_generation/` changed.

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| B0 | this commit (base `f772a18`) | [x] steps 1–4; handoff created; scratch `/Users/yuema137/mineworld-demos/cast-scratch/s26b1` created | [x] doc checks (see Evidence) | [x] no authority widened; I-B1-2 copied into handoff |
| B1 | this commit | [x] steps 1–5: `PIPELINE.md`, `README.md`, DECISIONS `ARC-94`, `DEP-51`, `DEP-52`, `DEP-55`; step 2 verifications recorded below | [x] doc checks (see Evidence) | [x] terms exact; no rule only in the README; the Winter and AccuRIG verifications are INCONCLUSIVE, named for S26-b2 |
| B2 | this commit | [x] steps 1–11: `cast.py`, `ruff.toml`, `mwcast/{__init__,cli,glb,records,scratch}.py`, tests `{conftest,gltf_fixture,test_glb,test_records}.py` | [x] pytest 19 passed (2.0 s); ruff check and format clean; pyright strict 0 errors; mutation (drop `// 3` in `triangle_count`) red on 3 tests, reverted green | [x] no third-party import in `glb.py`/`records.py`; all public functions typed; `Mapping[str, object]` only at the JSON document |
| B3 | this commit | [x] steps 1–4: `standard.py`, `gates.py`, `cli.py` (`gate`, `check-characters`), `test_gates.py`; 40 pytest passed; ruff, format, pyright strict clean | [x] mutations (a) MAX_TRIANGLES 100_001 → X-5 red; (b) G-5 parent comparison off → re-parented test red; (c) G-14 key scan off → key-shaped test red; all reverted green. Frozen evidence pinned (G-1 95000, G-2 `MW_Meshy`, G-5 PASS, G-6a PASS, G-14 exactly `references`, `reference_generator`, `pipeline`) | [ ] Opus review pending (contract requires Opus review of B3 before ready); see finding F-B1-B3-1 |
| B4 | this commit | [x] steps 1–4: `scripts/check_secrets.py` (scan, self-test); `scripts/ci_layer.py` (fast: two secrets commands; pipeline ruff/pyright in `PYTHON_STATIC`; pytest and `check-characters` in `python` and `python-smoke`); `pyproject.toml` `executionEnvironments`; `uv sync --locked` exit 0 | [x] `--self-test` PASS; `scan` 0 findings (1,903 tracked text files at the time); pipeline pytest 41 passed; ruff and pyright clean; mutation (remove `msy_` pattern) → self-test FAIL, reverted PASS | [x] no existing command changed or reordered; allowlist empty; scanner prints no matched text |
| B5 | this commit | [x] steps 1–5: `mwcast/meshy.py` (Transport seam, UrllibTransport, DEFAULT_PARAMS with `pose_mode a-pose`, CREDITS_PER_RUN 30, ledger, cap, redaction, run_live, plan), `mwcast/identifiers.py`, `cli.py` (`meshy --dry-run` and live, `identifiers`), `tests/test_meshy.py` (7 tests) | [x] 49 pytest passed; ruff, format, pyright clean; mutation (remove redaction) → 401 test red, reverted green; no live call made, no key read | [x] the key is read only from `os.environ` at the live request; `UrllibTransport` is constructed only in `cli.py`, never in a test |
| B6 | this commit | [x] steps 1–6: `mwcast/mask.py` (`alpha_bbox`, `run_mask`), `mwcast/texture.py` (`compare_texels`, `compare_masks`), `requirements.txt` (the four pins), `cli.py` (`mask`, `compare-texture [--alpha-iou]`), `tests/test_pipeline.py` (3 tests); texel bounds in `standard.py` | [x] 52 pytest passed; ruff and format clean; pyright 0 errors; imports of `rembg`, `PIL`, `numpy` are inside functions (the tests import the modules with none installed) | [x] no third-party import at module level; deviation F-B1-B6-1 recorded below |
| B7 | this commit | [x] steps 1–6: `blender/{LICENSE (cmp-identical), README.md, common.py, gameready.py, fit_bind.py, weights_fingers.py, weights_pack.py, flecks.py, export.py, render_ortho.py, render_bones.py, dump_rig.py}`; ruff check and format clean under the `blender/*` rule relaxation (F-B1-B7-2) | [x] step 7 smoke: real `blender 5.2.2` on `vitruvian.glb` via `dump_rig.py`, exit 0, 52 bone lines (log read in full, `smoke.log`); I-B1-9 literal scan recorded in §2.4; `cmp` of LICENSE | [x] per-script port diff recorded in the deviations list; no script imports another except `common` |
| B8 | — | [ ] | [ ] | [ ] |
| B9 | — | [ ] | [ ] | [ ] |
| B10 | — | [ ] | [ ] | [ ] |
| B11 | — | [ ] | [ ] | [ ] |

### Evidence

- **E-B1-audit** (design session, 2026-10-14): §3, including A-B1-1's hashes, A-B1-2's recovered chain, A-B1-9's
  measured −0.045356, A-B1-15's ratios, and §5.12's price read.

### Assumptions to verify in implementation

| Id | Assumption | Verified in | If false |
| --- | --- | --- | --- |
| A-V-1 | Blender's import of `vitruvian.glb` gives the same armature (heads, tails, rolls) as its import of `body_g5.glb` | B10 step 1 | stop: Q-B1-2's ruling rests on this, so the question returns to the primary session with the evidence |
| A-V-2 | The D+ scripts as they are on disk produced the committed asset (mtimes, A-B1-2) | B10 step 3 | localize; if an intermediate does not reproduce from its own script, record the evidence and stop |
| A-V-3 | Godot imports a `.gltf` with an external PNG without copying the pixels | B10 step 6 | §5.9's fallback goes to the primary session |
| A-V-4 | pytest at `tools/character_pipeline/tests` runs under the workspace's environment with no `addopts` | B2 | **verified in B2:** the root `pyproject.toml` has no `[tool.pytest]` section, and `uv run --locked pytest tools/character_pipeline/tests` runs (19 passed) with no `pytest.ini` |

### Mutations

(none run yet)

### Deviations and findings

(planning findings are §3.4)

- **B2 mutation (§9.0 step 4):** planted `triangle_count` without `// 3` in `mwcast/glb.py`; `test_glb.py`
  reported 3 FAILED (`test_fixture_decodes_to_the_counts_it_was_built_with`,
  `test_triangle_count_counts_every_material_primitive`, `test_committed_protagonist_mesh_decodes_to_the_frozen_counts`);
  reverted; 19 passed.
- **B2 finding F-B1-B2-1 (defaults):** PIPELINE §4.1 sets `flecks.enabled` default `true` (Q-B1-5: fleck repair on by
  default for new characters), so Vera's `false` is her single override, as B9's test expects. The design text
  "defaults are Vera's values" is read with this one exception, recorded here.
- **B2 finding F-B1-B2-2 (exempt keys):** the override rule exempts `character`, `stature_m`, `accessories`,
  `source.*` and `fit.picks` (PIPELINE §4.2 now lists them), which the design's §5.4 names as not overrides.
- **B3 finding F-B1-B3-1 (material, for the primary session; not decided autonomously):** the static gates run on
  the accepted protagonist mesh `meshy_d.glb` with Vera's stature 1.767 (read-only scratch run, 2026-10-14, load
  average ~200): G-1 PASS (95000); G-2 FAIL (`MW_Meshy`, expected, the export renames it); G-3 PASS (2048², 8-bit RGB);
  G-4 FAIL only because (b) needs the run directory (not yet run); G-5 PASS (52 joints, 0 mismatched parents, 0
  unweighted vertices); **G-6 FAIL on (b): `mixamorig:Spine` lies outside the body by 63.5 mm and `mixamorig:Spine2` by
  46.9 mm** (the other 16 body joints inside, margins 35–110 mm); G-6(a) PASS, but the arm and shank ratios sit at
  10.5–11.7 % against the 12 % relative bound; G-14 FAIL naming exactly `references`, `reference_generator`,
  `pipeline`. Consequence: AB1-10 (G-6 (a, b) PASS on the re-derived export) cannot hold if the accepted asset itself
  fails G-6(b) under the frozen bounds. The bounds are not changed (I-B1-6, §12 "changing a bound after a Gate run").
  **B4 notes:** `check_secrets.py` has no shebang (ruff EXE001 would fail it; the file is run by `python3` and I could
  not set its mode under the allowed-command rule). Its pattern list is kept equal to `standard.KEY_PATTERNS` by
  `tools/character_pipeline/tests/test_secrets_rule.py`.
  **B5 findings:** (i) `identifiers` on the accepted mesh reports `scenes[0].extras.mineworld_provenance.generator`
  (a `meshy` occurrence inside the provenance, as designed), `materials[0].name` as kind `meshy in a name` (the
  material `MW_Meshy`, a name, not a metadata identifier; reported separately, not hidden), and `images[0]` as a PNG
  `eXIf` chunk. D+ §4 said no machine-readable identifier was found; the presence of an `eXIf` chunk is structural
  and its content is not judged here: **to be confirmed by the primary session** before the B11 close-out.
  (ii) The `meshy` client's `DEFAULT_PARAMS` add `pose_mode: a-pose` (design §9 B5), not in D+'s `meshy_i23d.py`.
  **B6 deviation F-B1-B6-1 (pyright scope):** `mwcast/mask.py` and `mwcast/texture.py` run from the tool venv,
  where `numpy`, `PIL` and `rembg` are not in the root lock, so strict pyright cannot resolve them. Both files carry
  a file-level `# pyright: basic, reportMissingImports=false, reportMissingModuleSource=false` header; their images
  are typed `Any`. Ruff still checks them, and every other module and every test is pyright strict. Recorded here
  as a deviation from design §9.0 step 5's "pyright strict over mwcast/" for these two files only.
  **B6 procedural deviation:** before writing `mask.py` I ran `uv run --locked python -c "import numpy"` to see
  whether numpy was in the lock. That is a `-c` run, which the design's NEVER list forbids. It wrote no file and
  printed `ModuleNotFoundError: No module named 'numpy'`. Reported here, not repeated.
  **B7 finding F-B1-B7-1 (bounded, resolved and recorded):** `vitruvian.glb` has no mesh named `cm_vitruvian`
  (its mesh is `Body`). D+'s `rig_meshy.py` reads the CharMorph front surface from `cm_vitruvian` in the uncommitted
  `body_g5.glb`. Measured with `dump_rig.py`'s `front_y` (Blender 5.2.2, load average 85–170):
  `body_g5` `cm_vitruvian` at z 1.0048 → min y **−0.04535631090** (13 vertices); `vitruvian.glb` `Body` at the same
  height → **−0.04535621032** (13 vertices); difference **1.0e-7 m**, inside the design's 1e-5 bound (A-V-1). So
  `fit_bind.py` takes the mesh name as `cm_front_mesh` (driver: `Body` for `vitruvian.glb`). The measured value
  (−0.0453563) replaces −0.045356 in `fit.yaml` at B9 (≤ 1e-5 difference, B10 step 1). B10's parity run is what tests
  the rest of the front surface (non-hip centre points) against D+; a larger difference there is a material stop.
  **B7 finding F-B1-B7-2 (lint scope):** ruff's style rules E501, E731, E741, B905 and I001 are relaxed for
  `blender/*` only (`tools/character_pipeline/ruff.toml`), so each D+ statement keeps its shape for the port review.
  F (pyflakes) still applies.
  **Open: the primary session must decide before B10 step 5 is recorded as PASS**, with these numbers; B10's FAIL
  policy applies (localize, no bound change, material stop if AB1-8 cannot hold).
- **B2 record scope:** `records.py` types every section of `fit.yaml` as a frozen dataclass; the schema is the
  PIPELINE §4.1 table. `generation_log.yaml` run params are `raw` or `remesh` with a target polycount (D+ §3).
- **B0 evidence (2026-10-14):** `check_doc_headings.py` PASS (223 sections, 32 documents); `check_decision_ids.py`
  PASS (122 ids, all distinct). Blender `--version` = `Blender 5.2.2 LTS`; Godot `4.7.2.stable.official`.
  Scratch root created, empty.
- **B1 step 2 verifications (WebFetch, 2026-10-14):**
  - (a) Meshy Terms `https://www.meshy.ai/terms-of-use`: "Last updated: September 19, 2026". §2.4 reads (verbatim
    fragments): "The Service may include machine-readable metadata, digital watermarks, or other identifiers in Customer
    Output to identify that the Customer Output was generated by AI." and "You agree not to remove, alter, disable, or
    otherwise tamper with such identifiers." §3.3 names "Customer Output, including 3D models". Matches D+ §4's quote in
    substance: **no material change; no stop**.
  - (b) Meshy rigging `https://docs.meshy.ai/en/api/rigging`: bone count, finger bones and per-task price **not stated**
    on the page; only sample `consumed_credits: 5` in a sample object. Recorded; the Meshy rig is a joint source in
    S26-b2 only.
  - (c) AccuRIG: `reallusion.com/AccuRIG/` returned HTTP 404; `reallusion.com/accurig/` returned HTTP 404.
    **INCONCLUSIVE**: CLI/SDK not verified. Declined as the rig regardless (`DEP-55`); re-check before S26-b2.
  - (d) UniRig `github.com/VAST-AI-Research/UniRig`: MIT licence; README "CUDA-enabled GPU with at least 8GB VRAM" for
    generation; Python 3.11, PyTorch >= 2.3.1. RigNet `github.com/zhan-xu/RigNet`: **GPL-3.0** (`LICENSE-GPLv3`), a
    finding against the design's assumption that it is usable; declined (`DEP-55`).
  - (e) Blender licence `blender.org/about/license/`: "What you create with Blender is your sole property." No page date
    shown. Outputs are ours.
  - (f) Winter, 4th ed., Fig. 4.1: **INCONCLUSIVE**. The edition's figure could not be read (a ScienceDirect topic
    page returned HTTP 403). Nothing read contradicts step-26's `{upper_arm 0.186, forearm 0.146, thigh 0.245,
    shank 0.246}`; the primary re-checks against the book before S26-b2 uses G-6(a). Not a material stop.
- **B1 hashes (`shasum -a 256`, D+ scratch, 2026-10-14):** recorded in PIPELINE §13 (12 scripts). Vera's
  `model_glb.glb` = `3de801cdb7dcf34d21ee45758982319096bd62f3a111ab354c46fb2eeb586ef7` matches `fit.source.download_sha256`;
  `request.json` = `c002a274d7ea6251df5eaabd8daf5902b2585a343f3b509a0370af8893573bbf`;
  `task.json` = `6583a7698d517d293214cba7315a1b15a938d4a048cb433da850dde57b2703b5` (PIPELINE §14).
- **B1 doc checks:** `check_doc_headings.py` and `check_decision_ids.py` run after the edits (results recorded in the
  B1 commit message review line).
