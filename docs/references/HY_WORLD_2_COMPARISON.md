# Reference assessment — HY-World 2.0 (Tencent Hunyuan): capability comparison and reuse

**Status:** informational reference audit under [`REUSE_POLICY.md`](../REUSE_POLICY.md) §19. **This
document is not authoritative.** It decides nothing, changes no contract and amends no decision.
Where it disagrees with [`DECISIONS.md`](../DECISIONS.md) (`DEP-8`, `ARC-9`, `ARC-10`, `DEP-9`),
[`ENGINEERING_RULES.md`](../ENGINEERING_RULES.md), [`ARCHITECTURE.md`](../ARCHITECTURE.md) or
[`VISION.md`](../VISION.md), those govern and this file is the defect. Proposed changes to them are
listed in §9 as questions, not applied.

| | |
| --- | --- |
| **Question** (operator, 2026-10-08) | *"我觉得工具设计和优化上我们可以参考：https://github.com/Tencent-Hunyuan/HY-World-2.0 但是我们需要弄清楚我们的哪些功能是他们的框架并不满足的，然后他们有哪些开源实现我们可以用"* — use HY-World 2.0 as a reference for tool design and optimisation; establish which MineWorld capabilities it does not provide, and which of its open-source implementations MineWorld could use. |
| **Date** | 2026-10-08 |
| **Evidence read** | the HY-World 2.0 repository through the GitHub API and web: `README.md`, `License.txt`, `DOCUMENTATION.md`, `hyworld2/worldgen/README.md`, `hyworld2/panogen/README.md`, `.gitmodules`, `third_party/navmesh/setup.py`, the file tree, commit history and issues #1, #2, #4, #15, #16, #31 with comments; the Hugging Face model pages `tencent/HY-World-2.0` and `hanshanxue/WorldStereo`; the arXiv abstract 2604.14268; the licence files of HunyuanWorld-1.0, HunyuanWorld-Mirror and HunyuanImage-3.0; licence metadata and READMEs of every alternative named in §6. All read 2026-10-08. |
| **Licence method** | every verdict quotes the clause it rests on and names the URL. "First-hand" means the text was read at that URL on 2026-10-08, either by `gh api` (raw file contents) or by a fetch agent reading the page. "Not first-hand" means the claim comes from a summary, a third party, or a metadata tag only, and the row says so. Nothing here is legal advice; it is the `DEP-8` test applied to quoted text. |
| **What was not done** | nothing was cloned, installed, run or generated, and no weights were downloaded. The project page `https://3d-models.hunyuan.tencent.com/world/` renders by JavaScript and yielded no content. The paper's runtime sections (§8.1.5, §8.2.2 of the arXiv HTML) were not read first-hand, so no generation-time figure from the paper is relied on. |

---

## 1. Summary

**HY-World 2.0 is an offline 3D environment generator and reconstructor, not a world runtime.**
Its README describes it as *"a multi-modal world model framework for **world generation** and
**world reconstruction**"* that accepts *"text, single-view images, multi-view images, and videos"*
and produces *"3D world representations (meshes / Gaussian Splattings)"*
(<https://github.com/Tencent-Hunyuan/HY-World-2.0>, first-hand). Its use of "world model" means a
model that outputs static 3D scenes — Gaussian splats, point clouds and a TSDF mesh — for import into
Blender, Unity, Unreal or Isaac Sim. It is not a simulation, has no entities, rules, time, events,
networking or persistence of state, and the interactive character-and-collision viewer shown in its
materials (*WorldLens*, arXiv abstract <https://arxiv.org/abs/2604.14268>, first-hand) is **not in
the open-source release**.

The answers to the operator's two questions, in brief:

1. **What it does not satisfy for MineWorld: essentially everything MineWorld exists to build.**
   Every row of §4's comparison except two is "they don't" or "not comparable". HY-World 2.0 touches
   MineWorld only at the edge of one layer — producing environment art for a 3D Presentation Pack —
   and even there it produces the wrong kind of artefact for MineWorld's rule that the server is the
   single source of layout (`overall.md`, "One world, two views").
2. **What MineWorld can use from it: no code and no weights.** Every Tencent-authored component —
   code, weights and the 80B panorama model's base — is under a Tencent community licence that
   excludes the EU, the UK and South Korea and forbids distributing or displaying *Output* there
   (`License.txt` §5(c)). That fails `DEP-8` exactly as Hunyuan3D 2.1 already failed it
   ([`CHARACTER_ROUTE_ASSESSMENT.md`](CHARACTER_ROUTE_ASSESSMENT.md) §4.1). Independently, none of
   it runs on the operator's Apple M5 (CUDA 12.8, FlashAttention, custom CUDA kernels, a 4–8 GPU
   recommendation). **What is reusable is the permissively licensed ecosystem it is built on or
   points to** — gsplat, Recast, Brush, the MIT Godot splat addon, Spark, SPZ, MapAnything's Apache
   weights, Depth Anything 3's Apache sizes — and a set of **tool-design ideas** (§7) that transfer to
   `tools/asset_generation` without copying a line.

**Recommendation:** classify HY-World 2.0 as `REFERENCE ONLY` for design ideas and `REJECT` for every
artefact; record it beside Hunyuan3D in `DEP-8`'s excluded list (§9, Q-HY-1); adopt the §7 ideas into
the asset-generation tool where a step already needs them; keep Gaussian splats out of MVP-0 and
evaluate them, if at all, as a Presentation Pack backdrop spike with Brush and the MIT Godot addon.

---

## 2. What HY-World 2.0 is

### 2.1 Category

| Kind | Does HY-World 2.0 do it? | Evidence |
| --- | --- | --- |
| World **generator** (text/image → 3D scene) | **Yes**, its main capability | README, first-hand |
| World **reconstructor** (photos/video → 3D scene) | **Yes**, WorldMirror 2.0 | README, `DOCUMENTATION.md`, first-hand |
| Interactive **video world model** (action-conditioned frame prediction) | **No.** The README contrasts itself with *"Genie 3, Cosmos, and HY-World 1.5 (WorldPlay+WorldCompass)"*, which *"generate pixel-level videos"*. Video diffusion is used only inside the offline expansion stage | README, first-hand |
| **Engine** or runtime | **No** in the open release. Interaction (*"first-person navigation and third-person character mode … with physics-based collision"*) is offered through the hosted product <https://3d.hunyuan.tencent.com/sceneTo3D>; the paper's WorldLens renderer is not in the repository, and issue #31 (2026-07-01) asks which renderer the web demo uses, unanswered | README, arXiv abstract, issue #31, first-hand |

The stated uses are *"robotics simulation, game development, and environment mapping"* (arXiv HTML
Fig. 1 caption, first-hand via fetch agent).

### 2.2 Components and pipeline

World Generation (text or one image → navigable 3D scene), from `hyworld2/worldgen/README.md`
(first-hand):

| Stage | Component | What it does | Base models and libraries |
| --- | --- | --- | --- |
| 0 | **HY-Pano 2.0** (`hyworld2/panogen`) | text/image → 360° equirectangular panorama, default 1952 × 960 | backend 1: HunyuanImage-3 (*"~80B"*); backend 2: Qwen-Image-Edit-2509 + a *"~425M"* LoRA. Optional prompt enhancement through the DeepSeek API (`PE/deepseek.py`) |
| 1 | **WorldNav** (`traj_generate.py`, `src/navi_utils.py`) | *"VLM-guided camera trajectory planning with obstacle-aware navigation"*: finds targets with a VLM, segments with SAM3, builds a Recast navmesh over the panorama's proxy mesh, plans surround, exploration, reconstruction and aerial camera routes | Qwen3-VL-8B served by a **separate vLLM server**; SAM3; recastnavigation (git submodule) through Tencent's own pybind11 binding |
| 2 | Trajectory rendering (`traj_render.py`) | renders the panorama's point cloud along each route, VLM-captions each route | as stage 1 |
| 3 | **WorldStereo 2.0** (`video_gen.py`, `models/`) | camera-controlled video diffusion that "expands" the scene along each route; a *"PanoramaMemoryBank"* keeps routes mutually consistent; *"DMD-accelerated four-step inference (default, recommended)"* | *"extending WanTransformer3DModel"* (Wan 2.1); ~17B |
| 4 | GS data preparation (`gen_gs_data.py`) | frames, aligned depth, normals, cameras, sky split | WorldMirror 2.0, MoGe |
| 5 | **World composition** (`world_gs_trainer.py`, `gs/`) | trains the final Gaussian splat with depth/normal/LPIPS regularisation; *"Exports to `.ply`, `.spz` (compressed), and mesh (TSDF fusion) formats"* | `third_party/gsplat_maskgaussian`, *"our modified version of gsplat"* with MaskGaussian pruning |
| — | Viewer (`show_gs.py`) | *"Interactive browser-based 3DGS viewer (viser + nerfview)"* | viser, nerfview |

World Reconstruction: **WorldMirror 2.0** (~1.2B), a feed-forward model predicting depth, normals,
camera parameters, point cloud and 3DGS attributes in one pass, with optional camera/depth priors.
Outputs `gaussians.ply`, `points.ply`, depth and normal maps, `camera_params.json`, optional COLMAP
`sparse/0/` (`DOCUMENTATION.md`, first-hand).

Every scene is one directory that each stage reads from and adds to (`panorama.png`,
`meta_info.json`, `objects.json`, `navmesh/`, `render_results/view{N}/traj{M}/…`, `gs_data/`), so a
stage can be re-run without repeating earlier ones (worldgen README "Data Layout", first-hand). §7
returns to this.

### 2.3 Inputs and outputs, against MineWorld's formats

| | HY-World 2.0 | MineWorld equivalent |
| --- | --- | --- |
| Input | a prompt, one image, several images, or a video | a World Pack (YAML: people, places, items, organizations, enabled systems) and a Presentation Pack (assets and bindings) |
| Primary output | a Gaussian splat (`.ply`, `.spz`), a point cloud, a TSDF mesh (`.ply`) | a running, persisted, replayable world; for art, glTF/GLB (`DEP-7`) |
| Semantic content | `objects.json` (VLM-detected objects, an intermediate) and a navmesh used to plan camera routes | `Place`, `PlaceShape`, doorways, `Item`, `Person` — authoritative, owned by systems |
| Lighting | baked into the splat's colours | one lighting rig judged in-engine (`DEP-8` coherence procedure step 3, `ARC-13`) |

### 2.4 Hardware

| Fact | Source |
| --- | --- |
| *"We recommend **CUDA 12.8** and **Python 3.11+**"*; install *"one FlashAttention backend"* (FA3 *"Recommended for Hopper GPUs"* or FA2); the custom gsplat fork and the navmesh extension *"must be compiled from source"*; `cupy`, `pytorch3d`, `fused-ssim` from git | README, worldgen README, `requirements*.txt`, first-hand |
| World generation: *"≥4 GPUs recommended (tested with 8× H20)"*; the example VLM server runs `--tensor-parallel-size 8` *"on a separate GPU group or machine"*; stages 2–5 run under `torchrun --nproc_per_node 8` | worldgen README, first-hand |
| HY-Pano 2.0 full backend: 32 safetensors shards, *"169 GB"*; a user on a 95 GiB H20 ran out of memory at VAE decode, at *"11.88s/it"* for 50 steps | HF tree (fetch agent, first-hand); issue #15 opening post, first-hand |
| Commenter `ewrfcas` (affiliation not verified): HY-Pano-2 *"基于HYImage3，推荐8卡运行。否则推荐使用HY-Pano-2-Qwen"* (based on HunyuanImage-3, 8 GPUs recommended, otherwise use the Qwen variant); and for WorldStereo *"我们目前8卡推理每张GPU需要30G+"* (8-GPU inference needs 30 GB+ per GPU) | issues #15 and #16 comments, first-hand via `gh api` |
| Commenter `luoguohui99`: Qwen panorama 43 GB; full pipeline peak *"160G显存"* in total, about 40 GB per card on *"4张-48G显卡"*, *"跑一个场景大概要半小时"* (about half an hour per scene) | issues #1 and #15 comments, first-hand; a user report, not a project claim |
| A user cites the paper as *"10 minutes to generate a scene on a single Nvidia H20 GPU"* | issue #2 comment, first-hand; the paper text itself **not** read first-hand |
| No page read mentions macOS, MPS or Apple Silicon | all of the above |

**Apple M5, 24 GB unified memory, no CUDA (`ARC-10` audit): it cannot run.** The world-generation
pipeline needs CUDA kernels (FlashAttention, the gsplat fork, cupy) and roughly 30–40 GB per GPU on
several GPUs; the lighter panorama path alone was reported at 43 GB. WorldMirror 2.0 (1.2B) is the
only component small enough to be plausible on 24 GB, and whether it runs on MPS with its
gsplat-dependent heads disabled is **unverified** — the licence (§3) makes the experiment pointless
for anything that would be committed. Running it at all means rented multi-GPU CUDA machines.

### 2.5 Maturity

| Fact (2026-10-08) | Source |
| --- | --- |
| Repository created 2026-04-10; 2,703 stars, 228 forks; 0 releases; licence reported as `NOASSERTION` | `gh api repos/Tencent-Hunyuan/HY-World-2.0`, first-hand |
| Releases: technical report and WorldMirror 2.0 on 2026-04-16; HY-Pano 2.0 on 2026-05-11; world-generation inference and WorldStereo 2.0 weights on 2026-05-18; *"[July, 2026]: Update HY World 2.1! Try our product"* — 2.1 is hosted only | README "News", first-hand |
| Last code change 2026-05-18 (*"simplify installation"*); every commit since is README or citation (last: 2026-08-12 *"Fix citation"*) | commit list, first-hand |
| `DOCUMENTATION.md` §"World Generation": *"Coming soon."*; no training code (issue #32 asks), no datasets, no WorldLens | fetch agent and issues, first-hand |
| Open issues include holes in results with no completion (#5), point cloud to mesh (#8), panorama resolution limits (#29, #30), installation failures of the gsplat fork (#24, closed) and a Stage 1 segmentation fault (#28, closed) | issues, first-hand |
| Hugging Face `tencent/HY-World-2.0`: 3,603 downloads last month, last updated about five months before reading | fetch agent, first-hand |

**Reading:** a strong research release from a well-funded lab, active as a paper artefact and
inactive as software since late May 2026; the product line has moved to a hosted 2.1. That pattern —
open release as a snapshot, development continuing behind a hosted product — is the maintenance
profile `REUSE_POLICY.md` §2 ("Maturity") warns about for anything one would depend on.

---

## 3. Licence, per component, under `DEP-8`

### 3.1 The licence

`License.txt`, *"TENCENT HY-WORLD 2.0 COMMUNITY LICENSE AGREEMENT"*, *"Release Date: April 15,
2026"* (<https://github.com/Tencent-Hunyuan/HY-World-2.0/blob/main/License.txt>, first-hand via
`gh api`). It defines *"Tencent HY-WORLD 2.0"* as *"the 3D generation models and their software and
algorithms, including trained model weights, … inference-enabling code, training-enabling code …
made publicly available by Us at [https://github.com/Tencent-Hunyuan/HY-World-2.0]"* (§1(j)). The
clauses that decide MineWorld's question, verbatim:

| Topic | Clause |
| --- | --- |
| **Territory** | Header: *"THIS LICENSE AGREEMENT DOES NOT APPLY IN THE EUROPEAN UNION, UNITED KINGDOM AND SOUTH KOREA AND IS EXPRESSLY LIMITED TO THE TERRITORY"*. §1(l): *"'Territory' shall mean the worldwide territory, excluding the territory of the European Union, United Kingdom and South Korea."* §2: the grant is *"for the Territory only"*. |
| **Output outside the Territory** | §5(c): *"You must not use, reproduce, modify, distribute, or display the Tencent HY-WORLD 2.0 Works, Output or results of the Tencent HY-WORLD 2.0 Works outside the Territory. Any such use outside the Territory is unlicensed and unauthorized under this Agreement."* |
| **User threshold** | §4: if *"the monthly active users of all products or services made available by or for Licensee is greater than 1 million monthly active users in the preceding calendar month, You must request a license from Tencent, which Tencent may grant to You in its sole discretion"*. Measured *"on the Tencent HY-WORLD 2.0 version release date"*. |
| **Acceptable use** | §5(a): use must *"adhere to the Acceptable Use Policy"*, and *"You must include the use restrictions referenced in these Sections 5(a) and 5(b) as an enforceable provision in any agreement (e.g., license agreement, terms of use, etc.) governing the use and/or distribution"*. Exhibit A: *"Tencent reserves the right to update this Acceptable Use Policy from time to time. Last modified: December 30, 2025"*; 20 items, including *"For military purposes"* (19), *"In a manner that violates or disrespects the social ethics and moral standards of other countries or regions"* (15), and placing machine-generated content in public *"without expressly and conspicuously identifying that the information and/or content is machine generated"* (12). |
| **Training on outputs** | §5(b): *"You must not use the Tencent HY-WORLD 2.0 Works or any Output or results of the Tencent HY-WORLD 2.0 Works to improve any other AI model (other than Tencent HY-WORLD 2.0 or Model Derivatives thereof)."* |
| **Attribution and notice** | §3(a) copy of the Agreement to every recipient; §3(b) modified files marked; §3(c) *encouraged*, not required: mark products *"Powered by Tencent HY"*; §3(d) *required* for non-hosted distribution: a *"Notice"* text file stating *"Tencent HY-WORLD 2.0 is licensed under the Tencent HY-WORLD 2.0 Community License Agreement, Copyright © 2026 Tencent. All Rights Reserved. …"*; §3(e) a service built on it must state *"that Tencent is not affiliated with, associated with, sponsoring, or endorsing"* it. |
| **Ownership of outputs** | §6(d): *"Tencent claims no rights in Outputs You generate. You and Your users are solely responsible for Outputs and their subsequent uses."* §1(g): *"Outputs by themselves are not deemed Model Derivatives."* |
| **Relicensing derivatives** | §3, last paragraph: *"You may add Your own copyright statement to Your modifications and, except as set forth in this Section and in Section 5, may provide additional or different license terms and conditions … provided Your use, reproduction, modification, distribution, performance and display of the work otherwise complies with the terms and conditions of this Agreement (including as regards the Territory)."* |
| **Law** | §9: Hong Kong law, Hong Kong courts. §8(b): Tencent may terminate on breach; on termination *"You must promptly delete and cease use"*. |

### 3.2 Applying `DEP-8`

`DEP-8`, as amended 2026-09-27, asks **whether MineWorld may place the thing under MIT in a public
repository used worldwide**, not merely whether it may use it.

- **Outputs fail.** §6(d) disclaims ownership, but §5(c) forbids distributing or displaying Output
  outside the Territory. A public GitHub repository distributes its contents into the EU, the UK and
  South Korea, and MineWorld cannot geofence a `git clone`. This is the same reading, on the same
  clause, that excluded Hunyuan3D 2.1 ([`CHARACTER_ROUTE_ASSESSMENT.md`](CHARACTER_ROUTE_ASSESSMENT.md)
  §4.1; [`CHARACTER_ROUTE_E_EXPERIMENT.md`](CHARACTER_ROUTE_E_EXPERIMENT.md) §2, the
  "Hunyuan3D (any version) … excluded" row). An MIT `LICENSE` carries no territorial restriction, so
  MIT-licensing a HY-World output would grant what Tencent's terms withhold.
- **Code and weights fail, on three independent grounds.** (1) the same territory limit; (2) §5(a)
  requires every downstream agreement to carry the use restrictions *"as an enforceable provision"*,
  which MIT, having no field-of-use restrictions, cannot do; (3) §3's permission to relicense
  modifications is expressly *"except as set forth in this Section and in Section 5"* and conditional
  on Territory compliance — a permission to add terms, not to remove Tencent's.
- **The private layer of `ARC-9`** (*"a player's own world … the player's own business"*) is outside
  this test. A user inside the Territory may use HY-World 2.0 for their own world. MineWorld simply
  cannot ship, bundle, or commit its outputs or code, and its official tooling should not wire it in
  (§8, R-1).

### 3.3 Per component

| Component | Where published | Licence, as read | `DEP-8` (relicensable under MIT, public worldwide repository?) |
| --- | --- | --- | --- |
| Repository code: `worldrecon`, `worldgen`, `panogen`, the navmesh binding | GitHub `Tencent-Hunyuan/HY-World-2.0` | HY-WORLD 2.0 Community License (`License.txt`), first-hand | **Fails** (§3.2) |
| WorldMirror 2.0 weights | HF `tencent/HY-World-2.0/HY-WorldMirror-2.0` | HF tag `tencent-hy-world-2.0-community`; same `License.txt` in the HF repository (fetch agent, first-hand) | **Fails** |
| HY-Pano 2.0 weights (HunyuanImage-3 based, 169 GB) and the Qwen LoRA (`pytorch_lora_weights.safetensors`, 850 MB) | HF `tencent/HY-World-2.0/HY-Pano-2.0` | same tag (fetch agent, first-hand) | **Fails** |
| HunyuanImage-3.0, the base of HY-Pano backend 1 | GitHub `Tencent-Hunyuan/HunyuanImage-3.0` | *"TENCENT HUNYUAN COMMUNITY LICENSE AGREEMENT"*, *"'Territory' shall mean the worldwide territory, excluding the territory of the European Union, United Kingdom and South Korea"* (`gh api …/license`, first-hand) | **Fails** |
| WorldStereo 2.0 weights (68 GB) | HF **personal account** `hanshanxue/WorldStereo`, linked from the HY-World README's model zoo | HF metadata tag **`mit`**; README 24 bytes, no `LICENSE` file in the tree (fetch agent, first-hand). The HY-World licence covers what Tencent made available *"at [the HY-World-2.0 GitHub URL]"*, and the README is where these weights are made available | **Ambiguous — treat as fails.** An MIT tag without a licence text, on a personal mirror of weights released as a component of a Tencent-licensed system, is the kind of conflict `DEP-8`'s Mixamo entry resolves conservatively. It is also unusable on its own: inference needs Tencent-licensed code, Wan 2.1 base weights (issue #16), and ~8 × 30 GB |
| WorldStereo 1.0 code (predecessor) | GitHub `FuchengSu/WorldStereo` | `Apache-2.0` (`gh api`, first-hand) | Passes as code; not a usable tool here (same hardware class, no complete pipeline of its own) |
| `third_party/gsplat_maskgaussian` | vendored in the HY-World repository | a Tencent modification of gsplat (upstream Apache-2.0) combined with MaskGaussian (upstream `kaikai23/MaskGaussian`, licence `Other`, not read); sits under a repository-wide Tencent licence | **Do not take the fork.** Take upstream gsplat instead (§6.4) |
| `recastnavigation` submodule | upstream `recastnavigation/recastnavigation` | `Zlib` (`gh api`, first-hand) | **Passes** (upstream, not Tencent's binding) |
| Runtime models it calls | Qwen3-VL (code `Apache-2.0`, `gh api` first-hand), Qwen-Image-Edit-2509 (code repository `QwenLM/Qwen-Image` `Apache-2.0`, first-hand; the weight licence was **not** read first-hand), SAM3 (*"SAM License"*: derivatives *"may only"* be distributed *"under the terms of this Agreement"*, first-hand), DeepSeek API (terms not read) | Not relevant unless MineWorld built its own pipeline; listed so no one assumes them clean by association |
| Datasets | none released | — | — |
| Training code | not released (issue #32) | — | — |
| Hosted product (`3d.hunyuan.tencent.com`, HY World 2.1) | Tencent | terms of service **not read** | Unknown; not pursued |

**Predecessors, same family.** HunyuanWorld-1.0 (`LICENSE`, *"TENCENT HUNYUANWORLD-1.0 COMMUNITY
LICENSE AGREEMENT"*, same territory header) and HunyuanWorld-Mirror (`License.txt`, *"TENCENT
HUNYUANWORLD-MIRROR COMMUNITY LICENSE AGREEMENT"*, same header) were read first-hand via `gh api` and
fail for the same reason. HY-WorldPlay (HY-World 1.5, the real-time video world model) was not opened.

---

## 4. Capability comparison

Verdicts: **they cover it** · **partial** · **they don't** · **not comparable** (the two answer
different questions, so "covering" is not meaningful).

| MineWorld capability | What HY-World 2.0 offers | What MineWorld does | Verdict |
| --- | --- | --- | --- |
| **Persistent, server-authoritative simulation** | A static 3D scene that persists as a file. Nothing in it changes over time; there is no server, no owner of state, no rule. Its README's *"Build a world, keep it forever"* means the asset is kept, not that a world runs | A Rust server owns every fact; systems change state only through intents and events; the world runs for simulated days with no client attached (`ARCHITECTURE.md` §§3–5, `MVP.md` §7.4) | **They don't** |
| **Event sourcing and deterministic replay** | Generation accepts a seed (`--seed 42 --reproduce` for HY-Pano), which seeds one diffusion run. No event log, no replay, no digest. Multi-GPU diffusion is not bit-reproducible across hardware | The journal re-executed is the world (`ARC-25`); replay digests are tested byte for byte | **They don't** |
| **Installable System Packs and AC-1 composability** | A fixed five-stage pipeline in one repository; stages are replaceable only by editing scripts | Packs declared in an installed set (`ARC-33`, `DEP-12`); `ac1_composability.rs` proves `market-town` = `social-cafe` + six packs + configuration (`ARC-35`) | **They don't** |
| **The World Interaction List** (step 18: which entities may interact and with what consequence — talk, biography entry, trade, physical contact — as data plus pluggable packs) | Nothing in the release. The hosted product adds character collision against a collider mesh, which is one hard-coded physical interaction, not a configurable list | Interaction kinds, consequences and per-world configuration as content over installed packs; `bodies` uses Rapier with `enhanced-determinism` (`DEP-13`) | **They don't** |
| **Multiplayer protocol** | None | Server-authoritative WebSocket protocol, invites, seats, observation frames, affordances (`NETWORKING.md`, S11) | **They don't** |
| **2D and 3D clients sharing one world** | 3D only. The panorama is an intermediate image, not a 2D view of a world | One server, one World Pack; both Godot clients render the same disclosure; a parity test is planned (`overall.md`, "One world, two views") | **They don't** |
| **NPC social life, economy, memory** | None. `objects.json` lists detected objects for camera planning; there are no persons | Conversation, relationships (`ARC-28`), biography (`ARC-29`), items, holdings, money, shops, work (`ARC-36`–`ARC-38`) | **They don't** |
| **LM-optional design** | The *tool* is model-dependent end to end: an image model, a VLM behind a vLLM server, SAM3, a video model, optionally the DeepSeek API. Its *output* is model-free once made | The runtime runs with every model removed (`VISION.md` §2.4); generation is development tooling, never a runtime dependency (`ARC-10`) | **Not comparable.** HY-World is generation tooling; under `ARC-10` model-dependence is acceptable for tooling and forbidden for the runtime. Its outputs would satisfy LM-optional; its licence does not satisfy `DEP-8` |
| **Presentation Packs and art styles** | Styled 3D environments from a prompt or a reference image (*"diverse styles: realistic, cartoon, game, and more"*). No pack concept, no binding from semantic entities to visuals, no style manifest; lighting is baked in | Presentation Packs bind disclosed entities to assets; style is a pack (`ARC-1`–`ARC-4`, `ARC-11`); one lighting rig, one material authority (`DEP-8` coherence procedure) | **Partial**, on the narrow question of producing environment art for one 3D style; nothing on the pack, binding or 2D side |
| **Content authoring** | Strong at one thing: turning a picture into an explorable 3D scene. It authors appearance and geometry only | Authoring is semantic first (World Pack YAML, CLI `create`/`validate`) and visual second (Presentation Pack, `tools/asset_generation` candidates under `ARC-9`) | **Partial.** It covers the visual-environment half for 3D, in a form MineWorld cannot use as layout (§5, item 7) |

---

## 5. What HY-World 2.0 does not satisfy for MineWorld

The operator's first question, as a list. Items 1–6 are absences; items 7–10 are places where what
it *does* provide conflicts with MineWorld's rules, which matters more because those are the ones a
reader could mistake for coverage.

1. **No simulation.** No entities, components, time, processes, actions, events or systems. The
   output never changes after export.
2. **No authority or persistence of state.** "Persistent" in its README means a kept file. There is
   no event log, no replay, no save/resume.
3. **No composition.** One pipeline, one repository, no pack or plugin boundary. Nothing comparable
   to AC-1.
4. **No interaction semantics.** No interaction list, no affordances, no spatial requirements, no
   answer such as `TooFarAway` or `NoSupportedInteraction`. Collision exists only in the hosted
   product.
5. **No people, society or economy.** No persons, relationships, memory, biography, items or money.
6. **No networking and no 2D.** Single-user 3D asset output only; no protocol, no multiplayer, no
   second presentation of the same world.
7. **It invents layout, and MineWorld's layout must come from the server.** A generated splat world
   decides where walls, doors and paths are. MineWorld's rule is the reverse: *"Every place, doorway,
   wall/solid (`PlaceShape`) … a client shows comes from what the server discloses. A client may add
   decoration … It may not add or omit anything the world has"* (`overall.md`, "One world, two
   views"). A HY-World scene used as walkable space would be precisely the 3D-only geometry the
   parity test exists to forbid, and it has no 2D counterpart.
8. **Its navmesh is the wrong owner.** WorldNav builds a Recast navmesh to plan cameras. In
   MineWorld a navmesh is a client concept that never decides where a person can walk
   (`ENGINEERING_RULES.md` §12; `step-15-demo-3d.md` rejected `NavigationAgent3D` for that reason).
9. **Baked lighting defeats the coherence procedure.** Splat colours carry the generated scene's
   light. `DEP-8`'s coherence procedure requires one lighting rig and one material authority, and
   `ARC-13` fixes the rig; a splat can be neither relit nor re-pointed at the project's materials.
10. **Licence and hardware.** Nothing from it can be committed (§3), and none of it runs on the
    operator's machine (§2.4).

---

## 6. Reusable open-source pieces

The operator's second question. Each candidate is a job MineWorld might plausibly want done, the
HY-World component that does it, and at least two alternatives. **Placement** is always outside the
authoritative simulation: a tool under `tools/`, an asset pipeline step, or a Presentation Pack
builder (`ARC-10`). Classifications follow `REUSE_POLICY.md` §19: `REUSE` (adopt) · `ADAPT` ·
`REFERENCE ONLY` · `REJECT`.

### 6.1 Summary table

| Job | HY-World component | Verdict on it | Best alternative found | Verdict on the alternative |
| --- | --- | --- | --- | --- |
| Panorama sky / distant backdrop for a 3D style | HY-Pano 2.0 | **REJECT** (licence, 43–169 GB) | Poly Haven HDRIs (CC0, already approved in `DEP-8`); the existing OpenAI image path (`tools/asset_generation`, output owned) | **REUSE** what is already approved; no new dependency |
| Text/image → whole explorable 3D environment | WorldNav + WorldStereo 2.0 + composition | **REJECT** (licence, hardware, and §5 item 7) | none passes `DEP-8` and runs locally: Lyra 2 (weights research-only), WorldGen (needs FLUX.1-dev, gated), Marble (hosted, terms not read) | **REJECT** the job itself for MVP-0; it conflicts with server-owned layout |
| Feed-forward reconstruction (photos → cameras, depth, points) | WorldMirror 2.0 | **REJECT** (licence) | MapAnything with `facebook/map-anything-apache` weights; Depth Anything 3 Small/Base/Metric-Large/Mono-Large (Apache-2.0) | **REFERENCE ONLY** — no current MineWorld need |
| Gaussian-splat training | `gsplat_maskgaussian` fork | **REJECT** the fork | Brush (Apache-2.0, Rust, wgpu, runs on macOS); upstream gsplat (Apache-2.0, CUDA) | **REFERENCE ONLY** now; Brush is the candidate if a splat spike is approved (Q-HY-3) |
| Splat rendering inside the Godot client | none open (WorldLens unreleased; `show_gs.py` is a viser web viewer) | — | `ReconWorldLab/godot-gaussian-splatting` (MIT, Godot 4 addon); Spark (MIT, three.js); SuperSplat (MIT, editor) | **REFERENCE ONLY** now; the Godot addon is the `ADAPT` candidate for a spike |
| Compressed splat format | `.spz` export | uses Niantic's SPZ | `nianticlabs/spz` (MIT) | **REFERENCE ONLY**; follows the renderer decision |
| Navmesh generation | Tencent's pybind11 Recast binding | **REJECT** (licence) | upstream recastnavigation (Zlib); Godot's built-in navigation baking (already in the client, built on Recast) | **REFERENCE ONLY**; never authoritative (§5 item 8) |
| Collider mesh from a splat (TSDF fusion) | `gs/extract_mesh.py` | **REJECT** (licence) | Open3D TSDF (MIT); PlayCanvas `splat-transform` voxel collision (MIT, ported in the Godot addon) | **REJECT** the job: colliders come from server geometry (`DEP-13`) |
| VLM-guided automatic camera routes through a scene | WorldNav | **REJECT** the code; **REFERENCE ONLY** the idea (§7, T-4) | a scripted route over disclosed places; Qwen3-VL (Apache-2.0) only if a judge is wanted | see §7 |

### 6.2 Panorama backdrops (HY-Pano 2.0)

- **What it would do for MineWorld:** an equirectangular sky or far-horizon backdrop for a 3D
  Presentation Pack (Godot `PanoramaSkyMaterial`), outside the walkable area.
- **Where it would plug in:** `tools/asset_generation` as a second `ImageGenerationProvider`
  backend, writing candidates with `ARC-9` provenance.
- **Licence:** fails (§3.3: both the HY-Pano weights and HunyuanImage-3).
- **Hardware:** the full backend is 169 GB of weights and ran out of memory on a 95 GiB GPU; the
  Qwen variant was reported at 43 GB. Not on the M5.
- **Alternatives:** Poly Haven HDRIs (CC0, approved, real captured light — also the right input for
  the lighting rig); the OpenAI image path already in the repository, which passes `DEP-8`
  (`DEP-8`, "Generated images (OpenAI)") and is reference-conditioned (`ARC-10`); Sky3D (MIT,
  approved) for a procedural sky.
- **Recommendation: REJECT.** The approved sources already cover the job.

### 6.3 Whole generated environments (WorldNav + WorldStereo 2.0 + composition)

- **What it would do:** produce an explorable environment from one concept image — the thing the
  project is famous for.
- **Where it could plug in, at most:** a non-walkable backdrop in a Presentation Pack (a distant
  shoreline, a skyline). Never the walkable town: that geometry is the server's (§5 item 7), and it
  would have no 2D counterpart.
- **Licence:** fails. **Hardware:** 4–8 CUDA GPUs, ~30–40 GB each.
- **Alternatives, all checked 2026-10-08 via `gh api`:**
  - NVIDIA **Lyra 2** — code `Apache-2.0`, but *"Lyra 2.0 models are released under the NVIDIA
    Internal Scientific Research and Development Model License"* (`Lyra-2/README.md`, first-hand);
    *"Runtime on 1× H100 80GB: ~9 min per 80 frames"*. Fails on weights.
  - **WorldGen** (`ZiYang-xie/WorldGen`) — code `Apache-2.0`, low-VRAM mode around 10 GB, but
    *"You should also accept the license of the gated model (FLUX.1-dev)"* and it credits Apple's
    ml-sharp, whose model licence is *"exclusively for Research Purposes"*
    (`apple-aiml-research/ml-sharp/LICENSE_MODEL`, first-hand). FLUX.1-dev's output terms were not
    read first-hand. Unclear at best; treat as fails.
  - **Matrix-3D** (`SkyworkAI/Matrix-3D`) — code `MIT`, a 12 GB low-VRAM mode; its base models'
    licences were not checked. Not evaluated further because the job itself is rejected.
  - World Labs **Marble** — hosted, cited by the paper as the closed-source comparison; terms not
    read.
- **Recommendation: REJECT** for MVP-0, on architecture first and licence second. Revisit only as
  backdrop art, behind Q-HY-3.

### 6.4 Gaussian splats as a Presentation Pack asset kind

If MineWorld ever wants splats — photographed or generated backdrops, a captured real place as a
style — the reusable stack is entirely outside HY-World:

| Piece | Candidate | Licence (first-hand, `gh api`) | Fit |
| --- | --- | --- | --- |
| Training from photos | **Brush** (`ArthurBrussee/brush`) | `Apache-2.0` | Rust, wgpu; *"works on … macOS/windows/linux … and in a browser"*; trains from COLMAP or Nerfstudio data; also a `.ply` viewer. **Runs on the M5** and matches `DEP-9`'s Rust half |
| | upstream **gsplat** (`nerfstudio-project/gsplat`) | `Apache-2.0` | CUDA only; the library HY-World forked |
| | OpenSplat (`WebODM/OpenSplat`) | `AGPL-3.0` | tool-only use would not infect outputs, but `DEP-8` lists copyleft as incompatible for anything shipped; prefer Brush |
| | Inria `gaussian-splatting` | research licence (*"to allow the research community to use, test and evaluate"*) | **REJECT** |
| Rendering in Godot | `ReconWorldLab/godot-gaussian-splatting` | `MIT` | Godot 4 addon; Compute backend (Forward+) and, since 3.2.0-beta, a Raster backend for Mobile/Compatibility; its collision port comes from PlayCanvas `splat-transform` (MIT) |
| | `haztro/godot-gaussian-splatting` | `MIT` | smaller, older (last push 2026-03-13) |
| Rendering on the web | Spark (`sparkjsdev/spark`) | `MIT` | for a future web client only |
| Editing and cleanup | SuperSplat (`playcanvas/supersplat`) | `MIT` | crop, clean, export |
| Format | SPZ (`nianticlabs/spz`) | `MIT` | the compressed format HY-World exports; size ratios not measured here |

- **Where it would plug in:** a Presentation Pack asset kind (an `ARC-7` Asset Pack entry), loaded
  by the 3D client only, as decoration over disclosed geometry. Contracts unchanged.
- **Costs that decide it, not the tooling:** splats bake lighting (§5 item 9), have no 2D
  counterpart, and need `DEP-7`'s "glTF canonical" rule extended or excepted (glTF's
  `KHR_gaussian_splatting` extension was not checked here).
- **Recommendation: REFERENCE ONLY** for MVP-0. If the operator approves a spike (Q-HY-3): Brush for
  training (`REUSE`), the ReconWorldLab addon for rendering (`ADAPT`), SuperSplat for cleanup
  (`REUSE`), with a `DEP-` record per `REUSE_POLICY.md` §11.

### 6.5 Reconstruction, depth and camera estimation (WorldMirror 2.0)

- **What it would do:** recover cameras, depth and points from photos — useful for photo-based
  environment capture, or for depth-conditioned image generation over a blockout (§7, T-5).
- **Licence:** fails. **Hardware:** 1.2B; possibly feasible on 24 GB, unverified on MPS.
- **Alternatives:** MapAnything (code `Apache-2.0`; README: *"For Apache 2.0 license model, use
  'facebook/map-anything-apache'"*, first-hand); Depth Anything 3 (README table: DA3-Small, DA3-Base,
  DA3Metric-Large, DA3Mono-Large are *"Apache 2.0"*, the Giant and Large multi-view models *"CC BY-NC
  4.0"*, first-hand); VGGT (*"VGGT License"*: derivatives may be distributed *"only … under the terms
  of this Agreement"*, first-hand — not relicensable); DUSt3R (*"CC BY-NC-SA 4.0"*, first-hand —
  reject); COLMAP for classical reconstruction (licence not re-read here).
- **Recommendation: REFERENCE ONLY.** No step needs it. If T-5 is adopted, depth comes from
  MineWorld's own disclosed geometry rendered in Godot, which is exact and needs no model.

### 6.6 Navmesh and collision (WorldNav's Recast binding, TSDF mesh export)

- **What it would do:** walkable-area extraction and collider meshes for generated scenes.
- **MineWorld's position:** walkability is decided by the server (movement, `PlaceShape`, `bodies`
  with Rapier, `DEP-13`); the 3D client draws what it is told (`ENGINEERING_RULES.md` §12). Godot
  already bundles Recast-based navigation baking should a client ever need local steering.
- **One legitimate offline use:** a validation tool that bakes upstream Recast (Zlib) over a 3D
  Presentation Pack scene and checks that every disclosed doorway is reachable through the art — a
  complement to S14 16a's geometry probe. That uses Recast, not HY-World.
- **Recommendation: REJECT** the HY-World code; **REFERENCE ONLY** upstream Recast for that check.

---

## 7. Ideas for tool design and optimisation

Separate from code reuse: what MineWorld's tools can learn from how HY-World 2.0 is built. Ideas are
not covered by the licence's grant of rights over code and weights; this document reuses no code or
text. Each idea is mapped to the MineWorld step or tool that would own it; none is authorised by this
document.

| ID | HY-World pattern (evidence) | What MineWorld's tool would do | Owner |
| --- | --- | --- | --- |
| **T-1** | **One scene directory, accumulated stage by stage.** Every stage reads and adds to `<scene_dir>/`, so any stage can be re-run alone (worldgen README "Data Layout") | `tools/asset_generation` writes each candidate set as a job directory — `request.yaml`, `references/`, `candidates/`, `normalized/`, `contact_sheet.png`, `provenance/` — so normalisation or a contact sheet can be re-run without re-generating (and re-paying) | `tools/asset_generation`; the `ARC-9` roadmap's MVP-1 pack validation |
| **T-2** | **Content-addressed resume.** Stages skip work whose outputs exist | Key each generation by a hash of (provider, model id, prompt, reference image hashes, parameters, seed); a re-run with an unchanged key is a cache hit. This makes recipe iteration cheap and makes the provenance sidecar the cache index | `tools/asset_generation/batch.py` |
| **T-3** | **Draft and final tiers.** WorldStereo's DMD *"four-step inference"* is the default; the full model is the slow path | Two named tiers per recipe: `draft` (cheap model, low resolution, many candidates) for selection, `final` (better model) only for the candidates a person picked under `ARC-9`. Already half-present as `gpt-image-2.5-flare` versus `-sunburst` | `tools/asset_generation` recipes |
| **T-4** | **Automatic camera routes through a scene for evaluation.** WorldNav plans surround, exploration and aerial routes and captions them | A headless Godot tool that walks a scripted route through every disclosed place and doorway of a world and captures fixed frames. That gives `ARC-17` reviews and visual milestones (`ARC-20`, `ARC-24`) the same views every time, and a regression baseline per Presentation Pack. Routes come from the server's disclosure, not from a VLM; a VLM judge is optional and never a gate | S14 (16e/16f) and the visual-review tooling |
| **T-5** | **Geometry as a prior.** WorldMirror accepts camera and depth priors; WorldStereo conditions on point-cloud renders through a ControlNet | Condition generated art on MineWorld's own geometry: render the disclosed `PlaceShape` blockout to depth/normal/segmentation in Godot, and pass it with the style references when generating facade textures or concept frames. Art then fits the layout by construction, which is the "One world, two views" rule applied to generation. Needs a reference-plus-structure capable backend; the current OpenAI path accepts reference images only, so this is a second-provider question under `ARC-10` | S14 16f (Presentation Pack builds over disclosure); `ARC-10` second provider |
| **T-6** | **A memory bank for consistency.** *"PanoramaMemoryBank — Retrieval-based memory that maintains cross-trajectory consistency"* | A per-pack reference bank: every accepted asset becomes a candidate reference for the next generation in that pack, chosen by similarity, so a set stays coherent as it grows. This is `ART_DIRECTION.md`'s references-as-truth rule made incremental | `tools/asset_generation`; `ART_DIRECTION.md` §23 flow |
| **T-7** | **Seeded and reproducible flags** (`--seed 42 --reproduce`) | Record seed, model id and every parameter in the `ARC-9` sidecar; record "not reproducible" explicitly for hosted models that ignore seeds. Reproducibility of *tooling* is best-effort and declared; determinism of the *runtime* stays absolute and never depends on it | `mineworld_gen/provenance.py` |
| **T-8** | **Output filtering as a named stage** (edge, sky and confidence masks; `split_sky`) | Make normalisation steps named, ordered and individually skippable (`trim`, `matte`, `scale-to-metre`, `texel-density`, `palette-check`), mirroring `DEP-8`'s coherence procedure 1–4 as code | `mineworld_gen/normalize.py` |
| **T-9** | **Prompt enhancement as a separate, swappable stage** (`PE/`, system prompts by task) | Keep prompt templates as reviewable data in `recipes/` (already true) and, if an LM expands prompts, record the expanded prompt in provenance rather than only the short one | `tools/asset_generation` |
| **T-10** | **Benchmarks per stage**, each with a named metric (camera error, reconstruction F1, image quality) | Every tool stage gets a declared check rather than one end-to-end eye test: provenance completeness, licence field present, scale within tolerance, texel density, palette distance from the style references. Matches `ARC-17`'s prohibition on vague comparison | MVP-1 pack validation (`ARC-9` roadmap) |
| **T-11** | **Viewer bundled with the tool** (`show_gs.py`, the Gradio app) | A contact sheet already exists; add a one-command in-engine preview that loads candidates into the default lighting rig, since `DEP-8` step 3 says an asset is never judged outside it | `tools/asset_generation`, 3D client |

Two things **not** to copy, both visible in HY-World's issues and both mapping to MineWorld rules:
installation that requires compiling CUDA forks from source (#24, #28; `REUSE_POLICY.md` §9 on forks),
and a tool that needs a separately operated model server to run at all (`REUSE_POLICY.md` §2,
"Operational burden").

---

## 8. Risks

| ID | Risk | Kind | Mitigation |
| --- | --- | --- | --- |
| **R-1** | A contributor generates with HY-World (or any Tencent community-licensed model) on their own machine and commits the result. The licence breach is invisible in the file | licence | Add the Tencent community-licensed family to `DEP-8`'s excluded list by name (Q-HY-1); `ARC-9` provenance names the tool and model, so review can catch it; never wire such a model into `tools/` |
| **R-2** | The AUP can change unilaterally (*"Tencent reserves the right to update this Acceptable Use Policy"*), and §8(b) termination requires deleting everything. Even private-layer use carries terms that can move | licence | Do not depend on it, privately or officially |
| **R-3** | §5(b) bars using outputs *"to improve any other AI model"*. MineWorld's cognition step records fixtures; a contaminated asset in a training or evaluation corpus would breach it | licence | Same as R-1; MineWorld trains no model today |
| **R-4** | The WorldStereo weights' MIT tag invites a "this part is MIT" shortcut | licence | §3.3 records the ambiguity and the conservative reading; reopen only on a licence text from the authors (Q-HY-5) |
| **R-5** | Generated scenes used as walkable space would put layout in the client and break 2D/3D parity | architecture | §5 item 7; any splat or generated scene is backdrop-only, outside disclosed places |
| **R-6** | Diffusion output is not bit-reproducible across GPUs, drivers or library versions even with a seed | determinism | Generation is tooling and its output is committed as a file; the runtime never generates, so replay digests are unaffected. Record seeds anyway (T-7) |
| **R-7** | Hardware: CUDA-only, multi-GPU, 160 GB peak reported; renting machines adds cost, accounts and secrets handling | hardware | Not pursued. Mac-native tools (Brush) are preferred for any splat work |
| **R-8** | Model dependence and abandonment: a VLM server, SAM3, a video model, an optional DeepSeek API; code inactive since 2026-05-18; the product moved to a hosted 2.1 | maintenance | Reference only; nothing to maintain on MineWorld's side |
| **R-9** | Baked lighting in splats conflicts with the lighting rig and material authority | coherence | Backdrop-only, judged inside the rig (`DEP-8` step 3), behind Q-HY-3 |

---

## 9. Recommendation and questions

**Recommendation.**

1. **HY-World 2.0: `REFERENCE ONLY` for design ideas, `REJECT` for every artefact** — code, weights
   and outputs — on `DEP-8` (territory, §5(c)) and on hardware.
2. **Adopt the tool ideas where a step already needs them:** T-1, T-2, T-7 and T-8 in
   `tools/asset_generation` (small, no dependency); T-4 with S14's visual work; T-5 as the design
   brief for a second image provider under `ARC-10`, when one is chosen.
3. **No splats in MVP-0.** If wanted later, a bounded spike with Brush and the MIT Godot addon, as
   backdrop decoration only, with its own `DEP-` record.

**Questions.** "Operator-material" means it changes a decision record, scope, or what MineWorld may
depend on, and so needs the operator; the others the primary session can settle.

| ID | Question | Operator-material? | Recommendation |
| --- | --- | --- | --- |
| **Q-HY-1** | Amend `DEP-8`'s "Excluded" list with one line naming the Tencent community-licensed family — Hunyuan3D (any version), HunyuanWorld 1.0, HunyuanWorld-Mirror, HY-World 2.0 (including WorldStereo 2.0 weights), HunyuanImage — citing the territory clause. Today the Hunyuan3D exclusion lives only in two reference documents (`CHARACTER_ROUTE_ASSESSMENT.md` §4.1, `CHARACTER_ROUTE_E_EXPERIMENT.md` §2), not in `DEP-8` itself | **Yes** (amends a decision) | Yes |
| **Q-HY-2** | May the operator use HY-World 2.0 privately for look-development (inside the Territory, `ARC-9` private layer), on the understanding that nothing it produces — not even a cleaned or traced derivative — is committed? | **Yes** | No. The contamination risk (R-1, R-3) outweighs the value, and there is no hardware for it |
| **Q-HY-3** | Should Gaussian splats become a Presentation Pack asset kind at all (backdrops, captured places)? It would need an `ART_DIRECTION.md` statement, a `DEP-7` exception or extension, and a `DEP-` record for Brush and the Godot addon | **Yes** (scope and dependencies) | Not in MVP-0; reconsider at MVP-1 with a one-week spike |
| **Q-HY-4** | Take T-1, T-2, T-7 and T-8 into the next `tools/asset_generation` change, and T-4 into S14's visual-review work | No | Yes |
| **Q-HY-5** | Ask the WorldStereo authors to clarify the MIT tag on `hanshanxue/WorldStereo`? | No | No; the weights are unusable here regardless of the answer (§3.3) |
| **Q-HY-6** | Should `REUSE_POLICY.md` §19's "Audited so far" list name this document? | No | Yes, in the same PR that settles Q-HY-1 |

---

## 10. Sources

All read 2026-10-08. "API" means the raw file was read through `gh api`; "fetch" means a web-fetch
agent read the page; both are first-hand. Anything not first-hand is marked where it is used.

| Source | How |
| --- | --- |
| <https://github.com/Tencent-Hunyuan/HY-World-2.0> — repository metadata, `README.md`, file tree, `.gitmodules`, commits, issues #1, #2, #4, #15, #16, #31 and comments | API |
| <https://github.com/Tencent-Hunyuan/HY-World-2.0/blob/main/License.txt> | API and fetch |
| `DOCUMENTATION.md`, `hyworld2/worldgen/README.md`, `hyworld2/panogen/README.md`, `hyworld2/worldgen/third_party/navmesh/setup.py`, `hyworld2/panogen/PE/deepseek.py`, `requirements.txt`, `requirements_git.txt` in the same repository | API and fetch |
| <https://huggingface.co/tencent/HY-World-2.0>, its `/tree/main` and `/tree/main/HY-Pano-2.0` | fetch |
| <https://huggingface.co/hanshanxue/WorldStereo> and `/tree/main` | fetch |
| <https://arxiv.org/abs/2604.14268> (abstract, first-hand); <https://arxiv.org/html/2604.14268v1> (first ~39K characters first-hand, the remainder only as a summary and not relied on) | fetch |
| <https://github.com/Tencent-Hunyuan/HunyuanWorld-1.0> `LICENSE`; <https://github.com/Tencent-Hunyuan/HunyuanWorld-Mirror> `License.txt`; <https://github.com/Tencent-Hunyuan/HunyuanImage-3.0> `LICENSE` | API |
| <https://github.com/Tencent-Hunyuan/HY-WorldPlay> README (HY-World 1.5) | fetch |
| Licence metadata, and where quoted the README or licence text, of: `nerfstudio-project/gsplat`, `recastnavigation/recastnavigation`, `FuchengSu/WorldStereo`, `Wan-Video/Wan2.1`, `facebookresearch/vggt`, `facebookresearch/map-anything`, `naver/dust3r`, `graphdeco-inria/gaussian-splatting`, `ArthurBrussee/brush`, `WebODM/OpenSplat`, `sparkjsdev/spark`, `nv-tlabs/lyra` (`Lyra-2/README.md`), `SkyworkAI/Matrix-3D`, `ZiYang-xie/WorldGen`, `ByteDance-Seed/Depth-Anything-3`, `apple-aiml-research/ml-sharp` (`LICENSE_MODEL`), `microsoft/TRELLIS.2`, `facebookresearch/sam3`, `QwenLM/Qwen-Image`, `QwenLM/Qwen3-VL`, `haztro/godot-gaussian-splatting`, `ReconWorldLab/godot-gaussian-splatting`, `nianticlabs/spz`, `playcanvas/supersplat` | API |
| Not read: the project page `https://3d-models.hunyuan.tencent.com/world/` (JavaScript only), the hosted product's terms, HunyuanWorld-Voyager, FLUX.1-dev's licence, Qwen-Image-Edit-2509's weight licence, MaskGaussian's licence, COLMAP's licence | — |
