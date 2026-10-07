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
     fetcher, so this is **not read first-hand**): *"As between you and OpenAI, and to the extent
     permitted by applicable law, you (a) retain all ownership rights in Input and (b) own all
     Output. We hereby assign to you all our right, title, and interest, if any, in and to Output."*
     An assignment passes the relicensing test. Because the clause was not read first-hand, a
     derived view is **never committed**; it may feed a generation, and the generation's
     provenance records that it did.
   - Credentials: `source ~/.config/mineworld/secrets.env`; presence checked with `test -n`; no key
     is printed, logged, or written anywhere tracked.

## 4. The wrap: keep CharMorph's topology, UVs, eyes and rig

The generated head is used **as a shape target only**. Nothing of its triangles or texture reaches
the character.

1. **Where it is applied.** On the whole-body CharMorph export (the input of
   `clients/3d-spike/tools/character_model.py`, currently `scratch-character/s3/body_g5.glb`,
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
  writing; reported as "blocked on operator: Meshy key" if still absent at that step.
- **TRELLIS.2 on Apple silicon**: installing the port means cloning and compiling Metal extensions
  from individual developers' repositories and logging into Hugging Face for gated weights. The
  session's permission policy refused the dependency clone as untrusted code integration on
  2026-10-06. Not worked around; listed for the operator to decide.

## 9. Results

*(filled in as the experiment runs; nothing below this line was written before the measurement it
reports.)*
