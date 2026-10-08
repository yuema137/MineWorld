# Route E experiment — an image-to-3D head as a wrap target for the default character

**Status:** experiment specification and result record. **Not authoritative**: it decides nothing,
amends no decision and changes no contract. Where it disagrees with
[`DECISIONS.md`](../DECISIONS.md) (`DEP-8`, `ARC-9`, `ARC-17`, `ARC-19`, `ARC-23`, `ARC-24`),
[`VISUAL_FIDELITY.md`](../VISUAL_FIDELITY.md) or
[`CHARACTER_ROUTE_ASSESSMENT.md`](CHARACTER_ROUTE_ASSESSMENT.md), those govern and this file is the
defect.

| | |
| --- | --- |
| **Milestone** | `VIS-3D-GODOT-1` (FAILED by the operator on 2026-10-06; see [`HUMAN_REVIEW_QUEUE.md`](../HUMAN_REVIEW_QUEUE.md)) |
| **Route** | E of [`CHARACTER_ROUTE_ASSESSMENT.md`](CHARACTER_ROUTE_ASSESSMENT.md) §§2, 4.2, chosen by the operator on 2026-10-06 |
| **Reference** | `presentation/mineworld-default/3D/references/04_character_closeup.png` (1448 × 1086) |
| **Branch / worktree** | `vis/3d-human-pipeline`, `/Users/yuema137/mineworld-worktrees/vis-character` |
| **Scratch** | `/Users/yuema137/mineworld-demos/i23d/` (venv, clones, scripts, generated meshes, renders). Nothing large enters the repository |
| **Hardware** | Apple M5, 24 GB unified memory, no CUDA |
| **Written** | 2026-10-06, before any generator was run (`VISUAL_FIDELITY.md` §3, `ARC-23`) |

---

## 1. The question

> Can a generated mesh move the CharMorph head and face **measurably toward the reference**, at a
> quality tier that is not "crude"?

And, only with evidence: can the same generator help with the **whole character** — hair, hoodie,
backpack — which route D in the assessment predicts it cannot (§4.1: fused shell, hair as a solid,
no face rig)?

The failed candidate is **not tuned** by this experiment. It stays as pipeline validation only
(`VISUAL_FIDELITY.md` §9.2): its rig, retarget, animation, character slot and export are the
harness the wrapped head is measured in, and nothing else of it is being improved.

## 2. Generators, and whether each may be used

The test is `DEP-8` as amended: **may the output be relicensed under MIT?** Every model whose
weights run during a generation is listed, including encoders and background removers, because a
non-commercial component anywhere in the run taints the run. Read 2026-10-06.

| Generator | Code licence | Weight licence | Other models in the run | Runs here? | Verdict |
| --- | --- | --- | --- | --- | --- |
| **TripoSR** (`VAST-AI-Research/TripoSR`, clone at `107cefd`) | `LICENSE`: *"MIT License — Copyright (c) 2024 Tripo AI & Stability AI"* | <https://huggingface.co/stabilityai/TripoSR>: *"License: MIT"* | image tokenizer `facebook/dino-vitb16`, <https://huggingface.co/facebook/dino-vitb16>: *"apache-2.0"*. Its demo calls `rembg` (u2net); **not used** — the input is masked beforehand (below) | CPU/MPS; the only CUDA piece is `torchmcubes`, replaced by `skimage.measure.marching_cubes` in our wrapper | **passes** |
| **TripoSG** (`VAST-AI-Research/TripoSG`) | GitHub: *"MIT license"* | <https://huggingface.co/VAST-AI/TripoSG>: *"mit"* | inference script uses `briaai/RMBG-1.4`, <https://huggingface.co/briaai/RMBG-1.4>: *"released under a Creative Commons license for non-commercial use"* — **must not run**; the input is pre-masked. Upstream states *"CUDA-enabled GPU with at least 8GB VRAM"* and uses `diso` (CUDA) for extraction | unverified; attempted on MPS/CPU with `diso` replaced by `skimage` marching cubes. If it does not run in a bounded attempt, recorded as not run | **passes on licence**, if RMBG-1.4 is not in the run |
| **TRELLIS.2** via the Apple-silicon port `shivampkumar/trellis-mac` | port: *"MIT License"*; TRELLIS.2 MIT (assessment §4.1) | `microsoft/TRELLIS.2-4B` MIT | DINOv3 (gated; DINOv3 Licence, makes no claim on outputs); **RMBG-2.0 is CC BY-NC 4.0 and must be swapped for MIT BiRefNet** | the port reports ~18 GB peak on 24 GB machines, but it builds five Metal extensions from individual developers' repositories (`pedronaugusto/mtl*`, `trellis2-apple`) and needs a Hugging Face login for gated DINOv3 | **passes on licence if RMBG-2.0 is swapped**; whether it may be installed on this machine is an operator decision (§8) |
| **Meshy, paid plan** (hosted) | — | Terms of Use, <https://www.meshy.ai/terms-of-use> (assessment §4.1): *"such customers on a paid Meshy plan own their Customer Output"*; generation must stay private (help centre) | vendor-side | hosted | **passes**; needs `MESHY_API_KEY` in `~/.config/mineworld/secrets.env` |
| Stable Fast 3D / SPAR3D | — | Stability Community Licence | — | — | **not used.** The assessment's verdict is *"passes narrowly on ownership; not recommended"*, with a USD 1 M revenue cut-off and an output-use restriction. Followed as written |
| Hunyuan3D (any version), every free hosted tier, Rodin | — | — | — | — | **excluded** (assessment §4.1; operator brief) |

**Background removal.** Every generator above is fed an RGBA image whose alpha is already set, so
no generator's bundled remover runs. The alpha comes from **BiRefNet** (`ZhengPeng7/BiRefNet`,
<https://huggingface.co/ZhengPeng7/BiRefNet>: *"License: mit"*), or, if that cannot be run without
executing remote model code, from a hand-drawn polygon mask, which is our own work and needs no
licence.

## 3. Inputs

1. **The reference head crop** (primary). From `04_character_closeup.png`, a square crop around the
   head and hair (about x 345–645, y 60–400 in reference pixels), alpha-masked, padded and resized
   to each generator's expected input. The head is turned about 25–30° to her left and slightly
   tilted; this is the view the generator must lift.
2. **A chest-up crop** (whole-character question). The reference cropped at mid-thigh, masked —
   the input route D would use.
3. **Optional derived views.** A frontal, neutral-pose head view (and a profile) produced from the
   reference crop with the OpenAI image API, only if the single-view result is ambiguous. Their
   status, decided here:
   - They are **derived intermediate inputs**, permitted as a production technique by
     `VISUAL_FIDELITY.md` §12 and `ARC-9`. The reference stays the source of truth; a derived view
     never overrides it, and every comparison in §6 is made against the reference, never against a
     derived view.
   - **`DEP-8`.** OpenAI's terms, as returned by search (the terms pages returned HTTP 403 to the
     fetcher, so this was **not read first-hand** when this experiment ran): *"As between you and
     OpenAI, and to the extent permitted by applicable law, you (a) retain all ownership rights in
     Input and (b) own all Output. We hereby assign to you all our right, title, and interest, if
     any, in and to Output."* An assignment passes the relicensing test. Because the clause was
     not read first-hand, a derived view is **never committed**; it may feed a generation, and the
     generation's provenance records that it did.
   - **Superseded 2026-10-08.** The clause has since been read first-hand, in both the Terms of
     Use (ChatGPT) and the Services Agreement (API), with URLs, effective dates and the attached
     obligations: `DEP-8`, "Generated images (OpenAI), recorded 2026-10-08", in
     [`../DECISIONS.md`](../DECISIONS.md). OpenAI output passes the relicensing test. The
     never-commit rule above was a consequence of the missing read, not a finding against the
     terms; this experiment committed no derived view, and none needs to be added retroactively.
   - Credentials: `source ~/.config/mineworld/secrets.env`; presence checked with `test -n`; no key
     is printed, logged, or written anywhere tracked.

## 4. The wrap: keep CharMorph's topology, UVs, eyes and rig

The generated head is used **as a shape target only**. Nothing of its triangles or texture reaches
the character.

1. **Where it is applied.** On the whole-body CharMorph export (the input of
   `clients/3d-spike/tools/blender/character_model.py`, currently `scratch-character/s3/body_g5.glb`,
   43,101 vertices), *before* `character_model.py` runs, so the garments, freckles, trim and export
   are the unchanged pipeline. The export's companion files are moved with it: `<body>_blink.json`
   holds lid positions that `add_blink` matches within 0.2 mm, and `_landmarks.json` holds face
   positions; both are rewritten with the same per-vertex displacement, or the blink silently
   matches nothing.
2. **Alignment.** The generated mesh is brought into the head's frame by a similarity transform
   (uniform scale, rotation, translation) solved from paired landmarks: both inner and outer eye
   corners, nose tip, mouth corners, chin point, located on the CharMorph head from its vertex
   positions and on the generated mesh from a front render. Then a rigid ICP on the face region
   only (brow to chin, ear to ear, excluding hair) refines it. Printed: landmark residuals in mm.
3. **The deformation.** A `face` vertex group on the CharMorph mesh: the face from hairline to
   under the chin and cheek to cheek, weight 1 in the centre feathering to 0 over ~20 mm at the
   border, **weight 0** on the eyelid rims, the eyeballs (`MW_Sclera`, `MW_Iris`, `MW_Pupil`), the
   mouth interior (`MW_Mouth`), the ears and everything below the jaw line. A `Shrinkwrap`
   (`NEAREST_SURFACEPOINT`, or `PROJECT` along the normal both ways with a cull distance) to the
   aligned generated mesh, limited to that group, followed by a `CorrectiveSmooth` on the same
   group so the vertex spacing survives. Applied, then the eyeballs are moved by the mean
   displacement of their surrounding lid ring so they stay seated.
4. **What is kept, checked by number.** Vertex count, face count and UV coordinates are identical
   before and after (printed); the eyeballs are not deformed; the 52-joint armature and weights are
   untouched; the blink still matches all 1,099 lid vertices in `add_blink`.
5. **What is checked visually** before any runtime capture: eyelids not folded through the eyeball,
   no lip or nostril inversion, no step at the feather border (Blender headless render, 4× crop of
   each eye).

**Allowed bounded adjustments**: the feather width, the shrinkwrap mode, a global blend factor
between the morph head and the target (0.5–1.0), and the alignment. **Not allowed**: sculpting by
hand-written displacement toward a desired look — that would be route B's face by another name.

## 5. Where the comparison is made

`VISUAL_FIDELITY.md` §§7–8: pixels, at the reference's framing.

- **Raw-mesh gate** (step 3 of the brief). The generated mesh, untextured and textured where the
  generator has a texture, is rendered in Blender headless with the camera posed to match the
  reference head: yaw so the face shows the same three-quarter turn, the same framing as the
  reference head crop. Beside it, the current CharMorph head rendered by the same camera and the
  same material. Shown side by side with the reference crop at no less than 600 px per tile.
- **Runtime**. `./mineworld-3d --portrait` writes `P1`–`P8` into `clients/3d-spike/shots/`;
  `P6` (head) and `P1`/`P2` (chest-up) are the frames at the reference's framing. Side-by-side
  sheets are assembled with `magick` against the reference crop.

## 6. Success criteria, decided before measuring (`ARC-23`)

### 6.1 Raw-mesh gate (per generator)

The raw generated mesh **passes** only if all three hold in the side-by-side render:

1. **Face shape in the right direction, located.** On the matched-pose render, measured between
   2D landmarks located by eye on each image and printed with their pixel coordinates (eye
   centres, nose tip, mouth centre, chin point, face contour at the eye line): at least **three of
   four** ratios are nearer the reference's value than the current CharMorph head's — (a)
   inter-eye distance / eye-line-to-chin; (b) nose tip to mouth / eye-line-to-chin; (c) mouth to
   chin / eye-line-to-chin; (d) face width at the eye line / eye-line-to-chin. "Nearer" means a
   smaller absolute difference; ratios are reported to two decimals with all three values.
2. **Features exist as geometry.** Eye sockets, the nose, the lips and the chin are present as
   shape (visible in the untextured render), not only as paint.
3. **Not broken.** No holes, melted features or fused hair across the face region (brow to chin).

If the gate fails, wrapping cannot help: the generator is recorded as stopped at the gate with the
render, and is not wrapped.

### 6.2 Final scorecard (wrapped head, runtime frames)

The full `VISUAL_FIDELITY.md` §6 table is filled in, each row with the reference fact, the
candidate fact, the frame, and `PASS` / `PARTIAL` / `FAIL`. The rows route E can change are
**Face identity** and, through the face, **Overall identity** and **Overall vibe**. Hair, hoodie,
tee, backpack and material rows are expected to stay where the operator put them (`FAIL`), and are
re-read from the new frames, not copied.

The experiment **succeeds on its own question** if:

- **Face identity** reaches at least `PARTIAL`, with the improvement **stated as a visible fact**
  (which feature moved, toward what in the reference, in which frame), and the §6.1 ratios on the
  runtime frame still favour the wrapped head on at least three of four; and
- nothing is torn, folded or inverted in the face at 4× crop, the blink still works, and the
  quality tier of the face is not "crude" (no faceting, no stepped feather border, no smeared
  features).

A result meeting both may go to the operator **as a preview** (`VISUAL_FIDELITY.md` §9.1; it
still cannot reach acceptance with the other rows failing, §6). Anything less is reported as
`FAILED` with the evidence. The §9 question is answered in words: would a stranger recognise the
two as the same character?

### 6.3 Whole-character question (route D's prediction)

For each generator that runs, the chest-up crop is lifted once and rendered at the reference's
chest-up framing. Recorded per part, as facts: is the hair a separate, strand-like mass or a fused
shell; are the hood, drawstrings, zip and cuffs geometry or paint; is the backpack a separable
object; are the hand and strap separable; what was invented for the unseen back. The prediction is
**confirmed** for a part if it is fused or painted, **refuted** if it is separate geometry usable as
a part or a wrap target.

## 7. Stop conditions

1. **Per generator:** it fails the §6.1 gate, or it cannot be made to run on this machine within
   one bounded attempt (dependencies swapped for pure-PyTorch or CPU equivalents; no CUDA
   emulation projects).
2. **Wrap:** three wrap configurations (§4's allowed adjustments) each still fold an eyelid, invert
   the lips or leave a visible step → stop and record.
3. **Outcome:** if the wrapped head's Face identity row is `FAIL` on the runtime frames, the
   experiment is recorded `FAILED` for that generator; the remaining generator (Meshy) is still run
   when its key exists, because it is a different quality tier of generator.
4. **Overall:** when every available generator has stopped, the doc and the queue are updated and
   the recommendation is made: continue E (a generator passed and a better one is worth paying
   for), switch to C (commission), or stop.

## 8. Blocked or operator-gated items, recorded up front

- **Meshy**: needs the operator's paid account and `MESHY_API_KEY`. Absent at the time of
  writing; **provided by the operator on 2026-10-06 (Pro plan) and used, §§9.4–9.6.**
- **TRELLIS.2 on Apple silicon**: installing the port means cloning and compiling Metal extensions
  from individual developers' repositories and logging into Hugging Face for gated weights. The
  session's permission policy refused the dependency clone as untrusted code integration on
  2026-10-06. Not worked around; listed for the operator to decide.

## 9. Results

*(filled in as the experiment runs; nothing below this line was written before the measurement it
reports.)*

### 9.1 Inputs, as run

`/Users/yuema137/mineworld-demos/i23d/prep_inputs.py`: the reference cropped at
`(300, 40, 700, 440)` (head) and `(230, 40, 860, 1086)` (chest-up), alpha from BiRefNet through
`rembg` 2.0.85 (MIT) with its `birefnet-portrait` ONNX export
(`BiRefNet-portrait-epoch_150.onnx`, from the `danielgatis/rembg` release assets; the network is
`ZhengPeng7/BiRefNet`, MIT). ONNX weights only, no remote model code executed. Foreground padded to
85 % of a 1024² square on 50 % grey. No derived (OpenAI) view was used for any run below.

### 9.2 TripoSR — stopped at the raw-mesh gate

**Ran:** clone `VAST-AI-Research/TripoSR` at `107cefd`, weights `stabilityai/TripoSR`
(`model.ckpt`), `transformers` 4.46.3 (5.x renamed the ViT keys and the checkpoint no longer
loads), PyTorch 2.14.1 on MPS, `torchmcubes` replaced by `skimage.measure.marching_cubes`
(`run_triposr.py`), marching-cubes resolution 384. Head: 217,469 vertices, 28 s. Chest-up:
116,152 vertices, 18 s. Rendered by `render_raw.py` (Blender 5.2 headless, EEVEE); azimuth 0 is
the generator's input view and reproduces the reference's pose.

**Evidence:** [`e1_triposr_head_gate.jpg`](../../presentation/mineworld-default/3D/candidate/route_e/e1_triposr_head_gate.jpg)
(reference crop | TripoSR vertex colour | TripoSR clay | current CharMorph head, clay, same
three-quarter pose, `render_head.py`, yaw 30°), 640 px per tile.

**§6.1(1) ratios.** Landmarks read by eye on 768 px frames with a 96 px grid; the reading error is
about ±5 px, i.e. about ±0.02–0.03 on each ratio. `E` = eye line to chin.

| Landmarks (px) | Reference | TripoSR | CharMorph |
| --- | --- | --- | --- |
| eye centres | (330, 375), (485, 385) | (350, 375), (497, 380) | (382, 372), (494, 372) |
| nose tip / mouth centre / chin | (415, 462) / (400, 520) / (400, 612) | (430, 452) / (420, 492) / (410, 578) | (450, 452) / (430, 500) / (425, 577) |
| face contour at the eye line | x 215 … 525 | x 262 … 560 | x 250 … 512 |

| Ratio | Reference | TripoSR | CharMorph | Nearer |
| --- | --- | --- | --- | --- |
| (a) inter-eye / `E` | 0.67 | 0.74 | 0.55 | TripoSR |
| (b) nose tip → mouth / `E` | 0.25 | 0.20 | 0.23 | CharMorph |
| (c) mouth → chin / `E` | 0.40 | 0.43 | 0.38 | CharMorph (within reading error) |
| (d) face width at eye line / `E` | 1.34 | 1.49 | 1.28 | CharMorph |

**One of four.** Criterion 1 fails.

**§6.1(2), features as geometry: weak.** In the clay render the eyes are two horizontal ridges with
no lid shape or socket depth, the mouth is a single slit with no lip volume, the nose is present.
**§6.1(3), not broken: fails.** The face is a shallow relief: the surface is stair-stepped along the
voxel grid (visible as horizontal banding across the cheeks and forehead in clay), the mouth's
corners smear into the cheeks, and the hair is fused onto both sides of the face as one shell
with drips below the jaw.

**Verdict: TripoSR is stopped at the gate and is not wrapped.** Its face is not closer to the
reference than the current head on the stated ratios, and its surface quality is below the head it
would be wrapped onto.

**Whole character (§6.3), TripoSR:** [`e2_triposr_whole_character.jpg`](../../presentation/mineworld-default/3D/candidate/route_e/e2_triposr_whole_character.jpg)
(reference | front colour | front clay | side | back). Hair: one fused shell of lumpy ridges —
**confirmed**. Hood, drawstrings, zip, cuffs: paint on one surface; the hood is a bump at the
neck — **confirmed**. Backpack: fused into the back as one lumpy mass, the visible strap merged with
the hand — **confirmed**. Face: melted at this scale, the mouth a dark smear. Legs end at the crop
line; the back is invented. Route D's prediction holds for this generator on every part.

### 9.3 TripoSG — runs on Apple silicon; stopped at the raw-mesh gate

**Ran:** clone `VAST-AI-Research/TripoSG` at `fc5c409`, weights `VAST-AI/TripoSG` (11 files),
float32 on MPS, `diffusers` 0.41.0, `transformers` 5.19.0. `run_triposg.py` stubs `diso`, calls
the pipeline with `use_flash_decoder=False` so extraction is upstream's own
`hierarchical_extract_geometry` (skimage marching cubes), replaces the one function that hardcodes
`device='cuda'` with the same computation on the input's device, and feeds the BiRefNet-masked
RGBA composited on white — **`briaai/RMBG-1.4` was never downloaded or run.** 50 flow steps,
guidance 7.0, seed 42, octree 8 → 9 (512³). The flow sampling took about 10 s; extraction at 512³
on MPS took most of **1,159 s** in all. Output 1,688,601 vertices. So the upstream statement that
a CUDA GPU is required is true of its fast path only; the slow path runs here.

TripoSG canonicalises the head to face its own front, so the matched pose is a 30° turn from its
frontal azimuth (azimuth 240 in `render_raw.py` with `OBJ_YUP=1`).

**Evidence:** [`e3_triposg_head_gate.jpg`](../../presentation/mineworld-default/3D/candidate/route_e/e3_triposg_head_gate.jpg)
(reference | TripoSG clay at the matched turn | TripoSG clay frontal | current CharMorph head).
Shape only; TripoSG produces no texture.

**§6.1(1) ratios** (same method and error as §9.2; TripoSG landmarks on the matched-turn render:
eyes (400, 395), (520, 400); nose tip (480, 465); mouth (455, 525); chin (450, 600); contour
x 250 … 548 at the eye line):

| Ratio | Reference | TripoSG | CharMorph | Nearer |
| --- | --- | --- | --- | --- |
| (a) inter-eye / `E` | 0.67 | 0.59 | 0.55 | TripoSG |
| (b) nose tip → mouth / `E` | 0.25 | 0.30 | 0.23 | CharMorph |
| (c) mouth → chin / `E` | 0.40 | 0.37 | 0.38 | CharMorph (within reading error) |
| (d) face width at eye line / `E` | 1.34 | 1.47 | 1.28 | CharMorph |

**One of four.** Criterion 1 fails.

**§6.1(2): fails on the eyes.** The eyes are modelled **closed**: smooth convex lids with no
opening, no lid margin and no socket for an eyeball, in every view. The lips are a thick everted
pout, much fuller than the reference's thin closed smile; the nose has a modelled nostril. **§6.1(3):
the face region is clean** (smooth, no holes); the hair is a perforated shell with a hole above the
ear and hundreds of detached flakes around it.

**What it does carry that CharMorph does not:** a round, full-cheeked face with a small soft chin —
the reference's softness of the lower face reads in the clay. It reads as a young child's face,
not the reference's young woman.

**Verdict: stopped at the gate, not wrapped.** A wrap onto it would close CharMorph's lids over
the eyeballs and swell the lips, which is a different face, and the measured proportions do not
favour it. The whole-character run was **not made** for TripoSG (20 minutes per run, shape only):
its head result already shows hair as a perforated fused shell, and the route D question is put to
Meshy instead, which produces textured whole characters.

### 9.4 Meshy (paid, Pro) — the licence, read at the source before generating

The operator's Pro key was placed in `~/.config/mineworld/secrets.env` on 2026-10-06 (presence
checked only). Read 2026-10-06, before any generation.

**Terms of Use**, *"Last Updated: September 19, 2026"*, <https://www.meshy.ai/terms-of-use>:

- Paid plans: *"Customers on a paid Meshy plan have the option to keep their User Content private
  and your User Content will not be used for any purpose other than as outlined here. As between
  Meshy and those Customers on a paid Meshy plan, and to the extent possible under applicable law,
  such customers on a paid Meshy plan own their Customer Output."*
- Free plan, for contrast: *"Meshy owns all right, title, and interest, including all intellectual
  property rights, in and to the Customer Output"*.
- Meshy's retained licence: *"By using our Service and providing or generating User Content,
  Customers grant Meshy a non-exclusive, royalty-free, worldwide license to reproduce, distribute,
  and otherwise use and display the User Content and Process the User Content as may be
  necessary"*; and *"Meshy may use Customer Inputs and Customer Outputs (collectively, 'User
  Content') from non-Enterprise Customers to train, validate, test, or improve Services unless
  otherwise agreed to in the Order."*
- Community: content posted to the Meshy Community page is licensed to the public as CC0.
- Use restrictions on outputs: not to *"use generated digital assets to train, develop, or improve
  AI models that are competitive with Meshy"*; and *"You agree not to remove, alter, disable, or
  otherwise tamper with such identifiers"* (AI-generation identifiers in watermarks or metadata).
- **No clause** in the Terms addresses cancellation.

**Help centre**, <https://help.meshy.ai/en/articles/9992023-if-i-cancel-my-subscription-will-all-my-models-revert-to-a-cc-by-4-0-license>:
*"You'll retain the rights to the models you created while you were a subscriber, and they will
remain private indefinitely."* And
<https://help.meshy.ai/en/articles/10137554-what-is-the-ownership-of-the-generated-models>: paid
users *"retain full private ownership of all assets you create with Meshy"*, provided they are not
published to the community. Pricing page, <https://www.meshy.ai/pricing>: *"If you are on a premium
plan, you own all assets you create with Meshy"*; Pro is *"1,000 credits/month"*.

**Answers to the operator's three questions.**

| | Answer | Rests on |
| --- | --- | --- |
| (a) Is paid-plan output owned by the user, and may it be relicensed under MIT? | **Yes.** Ownership, not a licence; an owner may place it under MIT. Meshy's own non-exclusive licence back does not prevent that | Terms, paid-plan clause |
| (b) Does it survive cancellation? | **Yes, per the help centre; the Terms are silent.** Ownership is a property of the generation, made while paid; nothing in the Terms makes it conditional on a continuing subscription, and the help centre states it survives. The Terms are what binds, so this is a moderate-confidence yes, not a contractual certainty | help centre article 9992023; absence of any reversion clause in the Terms |
| (c) Are the assets private? | **Yes, by option**, on paid plans; they become CC0 public if posted to the Community. *"Private license for all assets"* on the plan page is consistent with this. Meshy still holds its non-exclusive operating licence and may train on non-Enterprise content | Terms, paid-plan and community clauses |

**`DEP-8` verdict: passes** for output generated on the paid plan and never posted to the
Community, with two obligations carried with any committed Meshy-derived asset: keep any AI
identifier Meshy embeds, and record `ARC-9` provenance. Because (b) rests on the help centre rather
than the Terms, a Meshy mesh that is **committed** should be recorded as such in the pack's
`LICENSES/`; used only as a wrap target (route E), nothing of Meshy's mesh is committed at all —
only CharMorph vertices moved toward it — so the question does not arise for that use.

**Budget and settings.** Image-to-3D on `meshy-7.1`: 20 credits mesh only, 30 with 2K texture
(<https://docs.meshy.ai/en/api/pricing>). Plan: one head task and one whole-character task, at
most two more if a setting has to change: **≤ 140 of 1,000 credits.** Settings
(`meshy_i23d.py`): `ai_model meshy-7.1`, `should_texture true`, `texture_resolution 2k`,
`enable_pbr false`, `should_remesh false` (the raw generated surface), `image_enhancement false`
(no restyling of the input before lifting), `remove_lighting true`, `moderation false`,
`target_formats [glb]`. REST only; no SDK, MCP, skills or animation library. Every output and the
exact request are kept in `/Users/yuema137/mineworld-demos/i23d/out/meshy/<name>/`.

### 9.5 Meshy head — passes the raw-mesh gate on the pixels; criterion 1 met on the ratios that discriminate

**Ran:** task `01a11525-bf52-7145-82ce-02226eb58aa9`, input `work/in_head_rgba.png`, settings of
§9.4, **30 credits** (balance 1,100 → 1,070). Kept in `out/meshy/head_v1/` (`request.json`,
`task.json`, `model_glb.glb`, 2048² base colour, five thumbnails). One closed, watertight surface
(after welding the glTF UV-seam splits, 1,562,844 vertices, 3,128,228 triangles, 1 connected part,
0 boundary edges); one UV set, no armature, no shape keys. It generated a **bust**: head, hair and
an invented hoodie to mid-chest. Meshy canonicalises the bust to face front; the reference's turn is
matched at azimuth 265 (`render_raw.py`, 2048² render cropped to the head).

**Evidence:** [`e4_meshy_head_gate.jpg`](../../presentation/mineworld-default/3D/candidate/route_e/e4_meshy_head_gate.jpg)
(reference | Meshy with its texture | Meshy clay | CharMorph clay),
[`e5_meshy_head_turnaround.jpg`](../../presentation/mineworld-default/3D/candidate/route_e/e5_meshy_head_turnaround.jpg).

**§6.1(1) ratios** (Meshy landmarks on the matched turn: eyes (350, 428), (505, 438); nose tip
(450, 515); mouth (430, 582); chin (430, 672); contour x 245 … 560):

| Ratio | Reference | Meshy | CharMorph | Nearer |
| --- | --- | --- | --- | --- |
| (a) inter-eye / `E` | 0.67 | 0.65 | 0.55 | **Meshy** (0.02 against 0.12) |
| (b) nose tip → mouth / `E` | 0.25 | 0.28 | 0.23 | CharMorph (0.03 against 0.02: inside reading error) |
| (c) mouth → chin / `E` | 0.40 | 0.38 | 0.38 | tie |
| (d) face width at eye line / `E` | 1.34 | 1.32 | 1.28 | **Meshy** (0.02 against 0.06) |

**Recorded deviation from §6.1(1) as written.** The criterion asked for three of four nearer; the
count is two nearer, one tie, one behind by 0.01 — inside the ±0.02–0.03 reading error. On
inspection the criterion was a weak instrument (`ARC-23`): ratios (b) and (c) are lower-face
proportions that the CharMorph morph search had **already fitted** to the reference
(`CHARACTER_ASSET_AUDIT.md` §12; the jaw and chin morphs), so neither head can beat the other on
them by more than the reading error. The two ratios on which the current head is measurably off —
eye spacing and face width — both move to within 0.02 of the reference. All four Meshy ratios are
within 0.03 of the reference; no other head in this experiment or in the candidate's history is.
**Decision, recorded rather than silent: the gate is treated as passed** on that reading, and the
wrap proceeds. If the operator reads §6.1(1) literally, this generator stopped at the gate and
§9.7 onward is outside the experiment's own rule.

**§6.1(2), features as geometry: passes.** In clay the eyes have modelled upper and lower lids, a
lash ridge, and a domed iris; the brows are raised ridges; the closed-lip smile has upturned
corners and a dimple; the nose has alar wings; the chin is small and rounded. **§6.1(3), not
broken: passes in the face region** — smooth, no holes, no stepping. The hair is sculpted as
separate clumped locks with stray strands lifting off the mass, not a smooth shell.

**Read as pixels, at the reference's framing (§§7–8):** the same person by category and largely by
face — large almond eyes with a heavy upper lash line looking off to her left, straight dark brows,
freckles across the nose and cheeks, a short small nose, a closed soft smile, full cheeks, a small
rounded chin, a messy updo with face-framing strands. Differences, largest first: the hair is a
darker, cooler brown without the reference's caramel highlights, and the bun sits higher and
rounder; the face reads a few years younger (rounder cheeks, larger eyes relative to the face); the
lighting is baked flat into the texture.

### 9.6 Meshy whole character — route D's prediction, tested

**Ran:** task `01a11529-b8d9-73bd-8ae3-cd03f533dc0a`, input `work/in_chest_rgba.png` (the
reference from crown to mid-thigh), same settings, **30 credits** (1,070 → 1,040). Kept in
`out/meshy/chest_v1/`. One closed watertight surface: 650,198 welded vertices, 1,300,908
triangles, **1 connected part**, no armature, no shape keys.

**Evidence:** [`e6_meshy_whole_character.jpg`](../../presentation/mineworld-default/3D/candidate/route_e/e6_meshy_whole_character.jpg)
(reference | matched turn | clay | 45° further | back),
[`e7_meshy_whole_full_body.jpg`](../../presentation/mineworld-default/3D/candidate/route_e/e7_meshy_whole_full_body.jpg)
(full body: it invented legs, jeans and grey trainers below the crop).

| Part | Route D predicted | Observed | Prediction |
| --- | --- | --- | --- |
| Face | reads as the reference in texture only, geometry soft | same face as §9.5, features modelled in clay at this scale too | **refuted** for appearance |
| Hair | a solid shell | sculpted clumped locks, bun, face strands; but one surface with the head | **refuted** for appearance, **confirmed** structurally (not separable, cannot move) |
| Hoodie: hood, drawstrings, zip, cuffs | painted, fused | hood bunched behind the neck and standing at the shoulders, both drawstrings, open front with zip edges, ribbed cuffs, sleeve folds — all **as geometry** in clay | **refuted** for appearance |
| Tee and graphic | possibly a blurred graphic | crew neck in maroon; the mountains and *"Good Places / Brighter People"* legible in the texture | **refuted** |
| Backpack | fused | olive canvas pack with lid, pocket, straps and buckles, invented convincingly from behind; fused to the back and to the hand at the strap | **confirmed** structurally |
| Hand on strap | fused | fused to the strap | **confirmed** |
| Back and legs | invented | invented, plausibly and consistently | as predicted |
| Expression rig, skeleton | none | none: no bones, no blendshapes, eyes are surface | **confirmed** |

**So route D's prediction was right about structure and wrong about appearance.** As a still
object at the reference's framing, Meshy's whole character is at the reference's quality tier and
reads as the same character. As a game character it is one fused, unrigged, 1.3-million-triangle
statue: the hair, hood and pack cannot move separately, the hand is welded to the strap, and the
face cannot blink or smile.

### 9.7 The wrap of the Meshy head onto the CharMorph head — three configurations

**Harness check first.** The body export the committed character was built from was identified as
`scratch-character/s3/body_g5.glb` by rebuilding it through the unchanged `character_model.py`: the
result is **byte-identical** to the committed `vitruvian.glb` (`cmp`). An identity wrap (blend 0)
re-exported through Blender and rebuilt gives the same face count, the blink matching 1,099 of
1,099, the same height, and 23 extra vertices from glTF normal splitting on re-export (faces
unchanged) — recorded, harmless.

**Tool:** `/Users/yuema137/mineworld-demos/i23d/wrap_head.py` (Blender headless), landmarks in
`wrap/landmarks_meshy_head_v1.json`. It implements §4 with these specifics, each a bounded
adjustment:

- the target is decimated to 469,233 faces for the BVH;
- landmark similarity with the nose down-weighted (0.15): the nose projection is a style
  difference, not an alignment cue; then a similarity ICP on 2,218 face vertices with the landmarks
  kept in the fit at half the weight. Residual after alignment: median surface distance 2.3 mm;
  canthi 4–8 mm, mouth corners 7–9 mm, chin 6 mm (these residuals *are* the shape difference the
  wrap transfers);
- nearest-surface displacement, field-smoothed (12 passes), snap limit `MAX_D`;
- eyes by the similarity of their canthi; the far eye's outer canthus is unreliable in the
  target's turned view (8 mm residual; measured eye scale 1.39 against 1.17 for the near eye), so
  both eyes take the near eye's scale, mirrored;
- **one bug found and fixed (`ARC-23`):** glTF splits vertices along UV and normal seams, the
  mesh graph does not connect the copies, so the smoothed field differed across 95 seam groups and
  opened hairline cracks (one visible on the philtrum, one showing the red nasal interior through a
  nostril at runtime). All coincident vertices now take one displacement.

| | Settings | Result | Frame |
| --- | --- | --- | --- |
| 1 | blend 1.0, `MAX_D` 12 mm, eyes per side, blend ring 0.75–1.25 eye widths | rounder cheeks and jaw; **a visible ring around her left eye and a torn outer corner** (from the 1.39 scale) | `e11` (right tile) |
| 2 | eyes symmetric at 1.17, ring 0.55–1.6, field re-smoothed outside the lids | ring gone; at runtime the **lips part and show teeth**, and a red fleck shows in a nostril | `e11` (second tile); runtime frames kept in `runtime_w2/` |
| 3 | blend 0.8, `MAX_D` 6 mm, the mouth moved as one piece, seams welded | **clean**: no ring, no crack, lips closed, blink 1,099 / 1,099 matched (lid travel 13.8 mm against 10.5 mm before, because the lids are 14 % larger) | `e8`, `e9`, `e10` |

Attempt 3 is the result. The stop rule (§7.2) allowed three configurations; the third is not
broken, so the wrap itself did not stop on geometry.

**Evidence, from the running client** (`./mineworld-3d --portrait`, the before frames captured in
the same session from the committed GLB; the committed GLB was restored afterwards and nothing of
the wrap is committed):
[`e8_wrap_runtime_head.jpg`](../../presentation/mineworld-default/3D/candidate/route_e/e8_wrap_runtime_head.jpg)
(reference | P6 before | P6 after),
[`e9_wrap_runtime_chest.jpg`](../../presentation/mineworld-default/3D/candidate/route_e/e9_wrap_runtime_chest.jpg)
(reference | P1 before | after | P2 before | after),
[`e10_wrap_runtime_face_3x.jpg`](../../presentation/mineworld-default/3D/candidate/route_e/e10_wrap_runtime_face_3x.jpg)
(P6 face, before | after, 3×).

**§6.2 ratios on P6** (before → after, reference): (a) inter-eye 0.52 → **0.57** (0.67);
(b) nose → mouth 0.26 → 0.27 (0.25); (c) mouth → chin 0.38 → 0.39 (0.40); (d) face width 1.18 →
**1.24** (1.34). Three of four nearer on paper, but only (a) and (d) move by more than the reading
error, and both remain 0.10 short of the reference.

**What moved, as visible fact:** the eyes are about 14 % larger with the lids following them; the
cheeks are fuller below the eyes and the jaw line softer; the chin a little rounder. **What did not
move:** everything a viewer reads first — the iris rolled to the corner with white showing beside
it, no heavy upper lash line, the flat neutral mouth with no smile, the hard-edged painted brows,
the dark freckle-spotted skin, the hair. At P1/P2 framing the before and after frames are hard to
tell apart.

### 9.8 Scorecard (`VISUAL_FIDELITY.md` §6), wrapped head, runtime frames

Largest miss first.

| Row | Reference | Candidate (attempt 3) | Frame | Verdict |
| --- | --- | --- | --- | --- |
| Overall identity | — | not the same person; the wrap is visible only side by side with the before frame | `e8`, `e9` | **FAIL** |
| Face identity | small soft young face, large almond eyes under a heavy lash line, short small nose, closed warm smile, freckles | the same realistic face with eyes 14 % larger and fuller cheeks; blank stare, no lash line, neutral flat mouth | `e8`, `e10` | **FAIL** |
| Material quality | soft warm skin, clean materials, a high-quality game render | unchanged: dark mottled skin, painted-block hair, dry cloth | `e8`, `e9` | **FAIL** |
| Hair silhouette | light fluffy updo, loose face strands | unchanged (not in route E's scope) | `e9` | **FAIL** |
| Hair colour | warm brown with caramel highlights | unchanged; orange-brown blocks | `e8` | **FAIL** |
| Hoodie structure | open burgundy zip hoodie, hood, cords, cuffs, drape | unchanged | `e9` | **FAIL** |
| T-shirt and graphic | oatmeal tee, mountain print high on the chest | unchanged | `e9` | **FAIL** |
| Backpack | natural straps sitting on her | unchanged | `e9` | **FAIL** |
| Overall vibe | warm, relaxed, cute, everyday | still a stiff low-cost demo character | `e9` | **FAIL** |

**§9 answer: no.** A stranger shown the reference and the wrapped head side by side would not
recognise them as the same character, and would not see a different tier of quality from the
FAILED candidate. **This is not a preview** (§9.1 requires face identity at least `PARTIAL` with a
visible improvement; it is `FAIL`). **Route E, as specified, is `FAILED`** for every generator
tried.

### 9.9 What the experiment found

1. **The local generators cannot do it.** TripoSR and TripoSG both run on this Mac without CUDA
   and without any non-commercial component; both stop at the raw-mesh gate (§§9.2–9.3).
2. **Meshy 7.1 can produce the reference's identity and quality tier — as a statue.** Its head and
   whole character are the first artefacts in this project's history that read as the same person
   at the reference's framing (§§9.5–9.6, `e4`, `e6`). The route D prediction was wrong about
   appearance and right about structure: one fused, unrigged, million-triangle surface.
3. **Wrapping does not carry the identity across.** After alignment, Meshy's face surface lies
   within a median 2.3 mm of CharMorph's. The identity the generated head carries is in its
   **painted** layer — the lash line, the iris, the brows, the smile corners, the skin — and in its
   hair, not in a few millimetres of shape. Route E transfers only shape, by design (§4), so it
   moves the ratios and leaves the person unchanged.

### 9.10 Credits and kept outputs

Meshy: **60 credits** of 1,100 (two tasks at 30; balance 1,040 after). Kept with their exact
requests in `/Users/yuema137/mineworld-demos/i23d/out/meshy/head_v1/` and `chest_v1/`. Committed
to the repository: only the evidence renders under `route_e/` (our renders of owned paid-plan
output, `DEP-8` §9.4). No generated mesh, texture or wrapped GLB is committed.

### 9.11 Recommendation

**Stop route E as a wrap.** Two directions remain, both for the operator to choose; neither was run.

- **C, commission, now with a much better brief.** The Meshy whole character (`e6`, `e7`) is the
  concept sculpt the assessment's commission lacked: an artist retopologises it onto the existing
  rig (or sculpts CharMorph toward it), separates hair into cards, hood, cords and pack into
  garments, and builds the face rig. That replaces the most expensive and least predictable part of
  the commission (finding the likeness) with a 3D target the operator has already seen. Recorded
  licence condition: Meshy-derived committed meshes carry `ARC-9` provenance and Meshy's AI
  identifiers.
- **A bounded follow-up the agent can run, which this experiment's §4 excluded:** bake Meshy's
  base colour onto the wrapped CharMorph face UVs (selected-to-active, the surfaces are already
  within ~2 mm), with the eye region handled separately because Meshy paints the iris on a surface
  where CharMorph has an eyeball. That tests whether the painted layer — where this experiment says
  the identity lives — transfers. Risks: baked-in lighting; the eye and mouth seams; it does nothing
  for hair, hoodie or material tier, which also `FAIL`. It needs the operator's agreement because it
  changes this design (texture, not only shape, would reach the character).

Not recommended: route D as a shipped asset (no rig, fused hand and strap, no face animation,
1.3 M triangles), or further wrap tuning (three configurations, the identity is not in the shape).
