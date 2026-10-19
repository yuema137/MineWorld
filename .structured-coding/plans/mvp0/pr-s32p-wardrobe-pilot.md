# PR S32-p — The wardrobe pilot: route B on Vera, Ivan and Hana

## DESIGN FROZEN — 2026-10-15 (operator approved the freeze in the primary session: "同意冻结并实施")

```text
Design revision:        revision 1 (2026-10-15), FROZEN 2026-10-15
Author:                 S32-p PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s32p, branch docs/pr-design-s32p,
                        planning base origin/main @ 5d901e85 (#226, step-32 APPROVED, merged)
Approved by / evidence: §15.1 ruled 2026-10-15: Q-S32p-2…6 by the operator (answered directly in the
                        primary session), Q-S32p-1 and Q-S32p-7 by the primary; freeze "同意冻结并实施";
                        relayed by the coordinator
Implementation base:    origin/main at C0, with the tooling base of §14 on it; C0 records the commit
Start condition:        §14 (honest state on 5d901e85: NOT startable today)
Execution contract:     §12 (filled, endpoint authority included)
Implementer:            Opus-class (step-32 §16, the S32-p row: research and vision judgement)
Lifecycle:              DESIGN FROZEN 2026-10-15
```

**Effort:** `mvp0` · **Step:** [`step-32-wardrobe.md`](step-32-wardrobe.md) (`APPROVED 2026-10-15`; rulings §20.1)
§§1, 4.2, 5.1–5.3, 6.1–6.6, 7, 14, 15, 16 (the S32-p row), 17, 18, 19 · **Amends the work of:**
[`step-26-cast.md`](step-26-cast.md) (the 2026-10-15 S32 amendment in its header; §§5.2, 5.5, 6.1–6.5, 10)
· **Parent:** [`overall.md`](overall.md) · **Consumes:** PR #211 (`mvp0/pr-s26b1` @ `121662c9`,
`tools/character_pipeline/`), the pose-sheet tool (`mvp0/s26-pose-sheet` @ `73c5bc2c`), the v2 casting inputs
(`mvp0/s26-cast-director` @ `d81baeb3`, which contains the `mineworld_gen` provenance fix `bd7bc92f`), the
prototype `/Users/yuema137/mineworld-worktrees/s32-proto/` (P-1…P-6) · **Followed by:** S32-c (3D modular
characters), S32-d (modular batches), S32-e (2D layers), which all need this pilot's route verdict · **Binding
parents:** `CLAUDE.md` §§1, 2, 3, 4 (items 2, 3, 8, 9, 13, 14, 16); `docs/REUSE_POLICY.md`;
`docs/VISUAL_FIDELITY.md` §§5, 8, 9; `docs/HUMANOID_PROFILE.md` §§1–4; `docs/DECISIONS.md` `ARC-9`, `ARC-19`,
`ARC-23`, `ARC-91`, `ARC-94`, `DEP-8`, `DEP-51`, `DEP-52`, `DEP-53`, `DEP-55`.

**Decision ids.** From step-32's reserved range only (§19.1, which names S32-p as the writer):
`ARC-138` (the modular character and route B, with the pilot's verdict), `ARC-140` (the runtime garment format
part; the runtime part is S32-c's), `ARC-141` (`ARC-94` amended for modular characters), `DEP-72` (MPFB2 as a
hidden tool), `DEP-73` (garment generation and fitting tools), `DEP-74` (rejected bases and generators). No other
id is needed. `ARC-135…137`, `ARC-139`, `ARC-142`, `DEP-75` and `DEP-76` belong to other S32 PRs.

**Money.** This design spends nothing. Its implementation spends Meshy credits (hard cap **1,200**, §4.6) and
OpenAI image calls (≤ 70, ruled in Q-S32p-6, §4.7), and only after the start condition of §14 holds.

---

## 0. Binding inputs, restated

**Operator rulings, 2026-10-15 (step-32 §20.1, Q-S32-1 and Q-S32-5; relayed for this pilot).**

- **Route B, "v2 + Meshy 分件生成，可换装".** Meshy generates each person's **base-layer body** from a v2-style
  reference image. Meshy also generates **each garment**, in the v2 style (Vera's realistic style, real human
  proportions, no cartoon features). Hair is a separate Meshy-generated asset (step-32 §§5.2, 6.4).
- **MPFB2 is only a hidden fitting tool.** No MPFB2 stock asset — body, face, skin, hair or garment — ever reaches a
  player, or the operator's review as the look.
- **Template v2 only** ("v2很好，v3全都不行").
- **Pilot cap: ≤ 1,200 Meshy credits**, inside S26's existing 3,000 approval. The full budget (≈ 8,000) is asked
  separately, after the pilot, with measured costs.
- **OpenAI image calls use model `gpt-image-2.5-sunburst`, named explicitly** (the tool's default is
  `gpt-image-2.5-flare`, `openai_images.py:34`). The `mineworld_gen` used is the one with the provenance fix
  (`bd7bc92f`).
- **Deliverable per person:** the multi-pose sheet (P-1, P-2, P-3, P-4 seated, P-6, P-8); the **same person in two
  different outfits**; everything shown **beside the fused Vera**.
- **Failure rule:** if the pilot does not reach the v2/Vera look, route B is **not adopted**. That is reported, with
  the diagnosis and the levers of step-32 R-S32-1, instead of shipping something close.
- **Order:** Vera first, as the parity control, in her own garments (hoodie, tee, jeans, backpack). Then Ivan and
  Hana, from their v2 identity references in `cast-out/<key>/`; the operator's casting pick may change which
  candidate.
- **Keys:** never read, printed or copied. Every keyed command runs as
  `source ~/.config/mineworld/secrets.env && <cmd>` (step-26 §5.5).
- **Machine load:** heavy Blender and Godot jobs run through
  `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`.

**Step-32, the S32-p row (§16), verbatim in substance.** Scope: "Garment pipeline g1–g7 for a pilot library of 12
garments + 4 hair styles, and the runtime format converter. Bodies: route B for Vera, Ivan and Hana. Cap ≤ 1,200.
Runners: W-1…W-12 headless; fit at load in a GDScript prototype; the pose sheet with `--outfit`. Measured: credits,
bytes, per-change rebuild time, frame time on a 16-person crowd." Depends on: "S26-b1 (#211) merged; Q-S32-1 ruled;
the pilot's v2 base-layer references approved at a casting page; no S32-a dependency." Checkpoint: "the pilot review
page `VIS-3D-WARDROBE-P`". Adversarial: "X-S32-5 a planted poke-through (the shirt shell shrunk 4 mm on the chest)
fails W-4 with location and turns the sheet red; X-S32-11 a garment with an empty hidden set on a tight top fails
W-7." Step-32 §15.3 also folds S26-b2 (the automation spike) into this PR; §3.6 F-S32p-9 and Q-S32p-7 settle how
much of it.

**The decision rule, fixed now (step-32 §5.2, `ARC-23` rule 2).** Route B is adopted only if **every subject**
passes W-1…W-12 and G-1…G-14 per assembled outfit, meets the S26 scorecard bar (step-26 §6.2), and the operator
judges it at the **v2/Vera look** beside the fused Vera, **in both outfits**. Every bound comes from Vera's accepted
record or a cited reference, never from a candidate.

---

## 1. Goal

Produce, for Vera, Ivan and Hana, a **modular character** by route B — a Meshy base-layer body, a hidden `hm08`
fitting template registered to it, and separately generated Meshy garments and hair, all on the shared 52-joint
`mixamorig` skeleton — show each person in **two different outfits** with the pose sheet beside the fused Vera,
pass or honestly fail every gate, measure what the full build will cost, and put it in front of the operator on the
review page `VIS-3D-WARDROBE-P`, inside 1,200 Meshy credits.

The pilot answers one question the step could not answer on paper: **does route B reach Vera's look while wearing
separate garments?** Everything else in S32's 3D half (S32-c, S32-d, S32-e) waits on that answer.

---

## 2. Scope

### 2.1 In scope

1. **The route-B body stages** in `tools/character_pipeline/` (tooling, `ARC-9`): the base-layer reference image,
   the Meshy base body, the existing stages 3 and 6–10 with the body/template export split, and the registration
   of `hm08` onto the body (§4.3 stages R-1…R-5).
2. **The garment pipeline g1–g7** (step-32 §6.6) as stages R-6…R-11, including the tight-fit class (§4.3 R-8b,
   Q-S32p-5), the hidden sets, and the converter from `.mhclo` to the runtime garment format (§4.4).
3. **The GDScript fit-at-load prototype** and the outfit assembly in a scratch Godot project (§4.3 R-12). The
   client is not touched.
4. **The gate runners:** G-1…G-14 per assembled outfit (step-32 §7.1, `ARC-141`), W-1…W-12 (§7.2), and the
   no-stock scan V-1 (§4.8), each with a planted mutation.
5. **The pose sheet's `--outfit` mode** (step-32 §7.3) and the pilot review page `VIS-3D-WARDROBE-P`.
6. **The live pilot:** Vera (parity control), then Ivan and Hana; at most 1,200 Meshy credits (§4.6).
7. **Measurements:** credits per accepted body and per accepted garment, retention, bytes per body and per
   garment, per-change rebuild time, and frame time on a 16-person crowd (§4.11).
8. **Records:** `ARC-138`, `ARC-140` (format), `ARC-141`, `DEP-72`, `DEP-73`, `DEP-74` in `docs/DECISIONS.md`;
   `VIS-3D-WARDROBE-P` in `docs/HUMAN_REVIEW_QUEUE.md`; `tools/character_pipeline/PIPELINE.md` §§16–17.

### 2.2 Not in scope

- Anything under `clients/`, `server/`, `systems/`, `kernel/`, `contracts/`, `worldpack/`, `persistence/`,
  `cognition/`, `sdk/` or `worlds/`. The wardrobe System Pack is S32-a1/a2; the client's modular characters are
  S32-c; the 2D layers are S32-e.
- The full garment library (≈ 70 garments, ≈ 20 hair styles) and the other 18 cast bodies (S32-d).
- Garment colour variants beyond what W-12 needs (one alternative palette colour per garment).
- Committing any generated binary. Pipeline outputs stay in scratch (`PIPELINE.md` §10 of #211: "Never committed:
  GLB, PNG, `.blend`, `.bin`, Meshy downloads …"; Q-S26-10: accepted characters only). The pilot commits records
  only (§4.9). Accepted assets enter the repository in S32-c/S32-d after the verdict.
- The Meshy Rigging API (step-32 §4.2: "not used (S26 `DEP-55` stands)").
- Route A (the parametric MPFB2 body as the look). It needs a fresh operator ruling (step-32 §5.2).
- The full 2D question, the release size gate and the release provenance scan X-S32-20 (S32-e, S32-h). V-1 is the
  pilot's own scan of its deliverable.

### 2.3 Dependencies

| Dependency | State on `5d901e85` | Needed for |
| --- | --- | --- |
| PR #211 (`tools/character_pipeline/`, S26-b1) merged | **OPEN** | every commit after C0 (step-32 §16 start condition) |
| The pose-sheet tool (`mvp0/s26-pose-sheet` @ `73c5bc2c`, one commit on top of #211) on `main` | **no PR** | C4 onwards |
| The v2 casting inputs and the fixed `mineworld_gen` (`mvp0/s26-cast-director` @ `d81baeb3`, stacked on #211) on `main` | **no PR** | C1 onwards (template v2, `anchor_fullbody.png`, briefs, `bd7bc92f`) |
| Ivan's and Hana's casting picks | **pending** (`mineworld-demos/casting/index.html`; agent ranking: candidate 2 for both) | C6 only |
| The base-layer references approved at a pre-spend page (step-32 §16) | not started | C5 (Vera), C6 (Ivan, Hana) |
| S32-a1/a2, S30, S31 | not needed | — ("no S32-a dependency") |

---

## 3. Audit

All paths under `tools/character_pipeline/` are read on `origin/mvp0/pr-s26b1 @ 121662c9` unless another branch is
named.

### 3.1 The pipeline (#211)

| # | Fact | Evidence |
| --- | --- | --- |
| A-p-1 | The stage runner has eight Blender stages, `gameready` → `fit_bind` → `weights_fingers` → `weights_pack` → `flecks` → `export` → `render_bones` → `dump_rig`; each is one subprocess with a `params/<tag>.json` | `mwcast/pipeline.py:26-52`, `:68-89` (`plan`), `:335` (`run`) |
| A-p-2 | `weights_pack` runs only when `fit.accessories` contains `"backpack"`; `flecks` only when enabled | `mwcast/pipeline.py:71-74` |
| A-p-3 | Every Blender stage runs `--background --factory-startup` against the pin `Blender 5.2.2 LTS` | `mwcast/blender.py:71-73`; `mwcast/standard.py:11`; `PIPELINE.md` §11 |
| A-p-4 | Stage 7 fits **our** armature from 2D picks on a frontal ortho render (`fit.picks`, hand-supplied in B1) against the CharMorph body `rig_source/body_g5.glb` | `blender/fit_bind.py:1-50`; `PIPELINE.md` §4.1 (`fit.picks`) |
| A-p-5 | Stage 10 names the armature `mixamo_vitruvian`, the mesh `Body`, the material `MW_Cast`, and writes a GLB plus a separate-texture glTF | `blender/export.py:27-58` |
| A-p-6 | glTF export settings: GLB, Y-up, skins on, morphs and animations off, extras on | `blender/common.py:17-28` |
| A-p-7 | **The Meshy client counts a nominal 30 credits per run** in a per-scratch ledger (`credits.json`) and refuses a POST when `spent + 30 > cap`; the measured `consumed_credits` is returned but **not** written to the ledger; the balance is read before and after | `mwcast/meshy.py:34`, `:118-125`, `:254-262`, `:271-272`, `:296-297` |
| A-p-8 | The Meshy defaults: `ai_model meshy-7.1`, texture on, 2k, no remesh, `remove_lighting`, `pose_mode a-pose`, multi-view thumbnails | `mwcast/meshy.py:40-52` |
| A-p-9 | `cast.py meshy` requires a `fit.yaml` whose `source.input_sha256` equals the input image; the cap is a required CLI argument; the ledger is `<scratch>/credits.json` | `mwcast/cli.py:49-55`, `:133-178` (`:143`, `:150`) |
| A-p-10 | Gates G-1 (exactly one skinned mesh, ≤ 100,000), G-2 (exactly one material `MW_Cast`, one primitive), G-3 (2048²), G-4, G-5, G-6, G-14 are static Python over one GLB | `mwcast/gates.py:70-127`, `:135-295`, `:312-394`; `mwcast/standard.py:20-31` |
| A-p-11 | `fit.yaml` defaults are Vera's; `gameready.target_triangles` defaults to 95,000; more than three overrides is a finding | `mwcast/records.py:513-520`; `PIPELINE.md` §4.1–4.2 |
| A-p-12 | `mwcast/` is standard-library Python checked by ruff and pyright and tested by `pytest tools/character_pipeline/tests` in CI's `python` layer on all three platforms; the tool venv (`requirements.txt`: rembg, onnxruntime, numpy, pillow) is never installed by CI | `scripts/ci_layer.py` on `mvp0/pr-s26b1` lines 194-224; `requirements.txt`; `PIPELINE.md` §12 |
| A-p-13 | Never committed: GLB, PNG, `.blend`, `.bin`, Meshy downloads, Blender or Godot outputs; committed per character: `fit.yaml`, `generation_log.yaml`, the re-derivation report | `PIPELINE.md` §10 |
| A-p-14 | The Blender scripts are GPL-2.0-or-later under `blender/LICENSE`; the MIT driver never imports them | `blender/README.md`; `PIPELINE.md` §11 |

### 3.2 The pose-sheet tool (`mvp0/s26-pose-sheet @ 73c5bc2c`)

| # | Fact | Evidence |
| --- | --- | --- |
| A-p-15 | `pose-sheet <export> --out <dir>` stages a scratch Godot project, copies the client's `human.gd`, `posture.gd`, Vera (`meshy_d.glb`), the Quaternius clips and bone maps, the café chair and table and the HDRI; renders P-1 (front, three-quarter, rear), P-8, lineup with Vera, P-6 two-shot with Vera, P-2 contact and passing, P-3 contact, P-4 seated (three-quarter, side) | `PIPELINE.md` §15; `mwcast/pose_sheet.py:27-33`, `:128-166`, `:247` |
| A-p-16 | The subject is built by `Human.build(... Body.REFERENCE)`; only a surface whose material is named exactly `MW_Cast` gets the cast surface response | `godot/pose_sheet.gd:176-197` |
| A-p-17 | **`Human.build` hides any mesh named `Hoodie` or `Pack`** when the hoodie or pack colour is transparent | `clients/3d-spike/scripts/human.gd:326-332` (on `main`) |
| A-p-18 | The torso-clip check (G-9, simplified) and the seat-contact check (G-13, ±20 mm) run on CPU-skinned vertices; **a layered character is flagged by the torso check** (the CharMorph town body's tee sleeve under its hoodie sleeve reads "inside") | `PIPELINE.md` §15 "Checks", "Validated" |
| A-p-19 | P-5 and P-7 are not rendered; the window must stay drawable; the driver stops Godot after 600 s; the frames are byte-identical across reruns except P-4 by ≤ 24 pixels at 1/255 | `PIPELINE.md` §15 |

### 3.3 The v2 casting inputs (`mvp0/s26-cast-director @ d81baeb3`)

| # | Fact | Evidence |
| --- | --- | --- |
| A-p-20 | Template **v2**: `version: 2`; identity anchor `style/anchor.png` (sha256 `5b0c8f0b…a3a1`), full-body style anchor `style/anchor_fullbody.png` (sha256 `c4b936fe…3ef32`); the full-body preamble and the `person_full` paragraph; "Changing any field below is a new template version" | `presentation/mineworld-default/3D/characters/style/template.yaml:1-31`, `:57-75`, `:105-115` |
| A-p-21 | Ivan: male, 70, 1.67 m, heavy, MST 5, white close fade, stubble; jacket stone-beige linen sport jacket, burgundy polo, dark brown trousers, brown leather walking shoes, straw panama | `cast/ivan-petrov/brief.yaml:4-27` |
| A-p-22 | Hana: female, 30, 1.68 m, heavy, MST 4, black pixie cut; sage cropped jacket, pale blue collared shirt, knee-length pleated charcoal skirt **over dark opaque tights**, black ballet flats | `cast/hana-sato/brief.yaml:4-26` |
| A-p-23 | Candidates: `cast-out/{ivan-petrov,hana-sato}/candidate_{1,2,3}.png`, each with an `ARC-9` sidecar naming `gpt-image-2.5-sunburst`, `images.edit`, 1024×1536, high; measured ≈ $0.0563 per call; agent ranking candidate 2 for both; the operator's pick is pending on `mineworld-demos/casting/index.html` | `cast-out/ivan-petrov/generation_log.md`, `candidates.md`; `cast-out/hana-sato/generation_log.md`; casting page lines 67-68 |
| A-p-24 | Ivan's candidates read "more photographic than Vera's" in the agent check — a style-parity concern the base-layer image inherits | `cast-out/ivan-petrov/candidates.md` "Style/tier" |
| A-p-25 | `mineworld_gen` CLI: `--prompt`, repeatable `--reference`, `--out`, `--name`, `--count`, `--size`, `--quality`, `--model` (default `None` → `MINEWORLD_IMAGE_MODEL` → `DEFAULT_MODEL = "gpt-image-2.5-flare"`) | `tools/asset_generation/mineworld_gen/cli.py:24-46`; `openai_images.py:31-35`, `:61-62` |
| A-p-26 | **The provenance fix is not on `main`.** `bd7bc92f` reads each reference's record before the paid call and keeps an absolute path for a reference outside the repository; `origin/main..d81baeb3` shows it changing `openai_images.py`, `provenance.py`, `tests/test_provenance.py` | `git show --stat bd7bc92f`; `provenance.py:38-49` on `d81baeb3` |
| A-p-27 | Vera's fused record: Meshy task `01a1155c-…`, three runs at 30 credits, one kept; `fit.yaml` `accessories: ["backpack"]`, stature 1.767 m | `cast/vera-lindgren/fit.yaml:1-16`, `generation_log.yaml:1-34` |

### 3.4 The prototype (`/Users/yuema137/mineworld-worktrees/s32-proto/`, never committed)

| # | Fact | Evidence |
| --- | --- | --- |
| A-p-28 | MPFB2 2.0.18 installed into an isolated `BLENDER_USER_RESOURCES` (`s32-proto/blender-user`); the build runs **without** `--factory-startup` (the extension must load) through `with-slot.sh` | `scripts/run.sh:3-5` |
| A-p-29 | `HumanService.create_human(macro_detail_dict=…)`, `add_builtin_rig(basemesh, "mixamo")`, `add_mhclo_asset(…, material_type="GAMEENGINE")`; the `hm08` basemesh has **19,158 vertices**, 13,380 after the helper mask | `scripts/build.py:13-33`, `:48-54`; `out/womanA.log` lines 5-7 |
| A-p-30 | Delete groups are `Delete.<asset>` vertex groups with MASK modifiers; only socks and shoes carried them | `out/womanA.log` lines 10-12 |
| A-p-31 | One Skeleton3D with 52 bones; each part a `MeshInstance3D` with its own `Skin` of 52 binds; a garment reparented under the skeleton with `skeleton = NodePath("..")` after clearing `owner`; `find_bone("mixamorig_LeftArm")` (underscore) | `godot/swap.gd:17-55` |

### 3.5 External facts used (read at source by the step session, 2026-10-15; step-32 §4.7)

- MPFB2 core assets CC0; code GPL-3.0-or-later; the CC-BY packs fail `DEP-8` (step-32 §4.1).
- Meshy paid plan: the customer owns Customer Output (step-32 §4.2); Auto Split does not keep textures.
- Godot named binds resolve per `Skin`; the importer renames `mixamorig:` to `mixamorig_` (P-6).
- Robust Skin Weights Transfer: MIT, libigl Python, CPU (step-32 §4.2; S26 `DEP-53`). Its licence and libigl's
  Python binding licence are re-read at C3 before use (step-26 §5.4 W says **verify**).

### 3.6 Findings

- **F-S32p-1 — The credit ledger is nominal.** The ledger adds 30 per run whatever Meshy charged (A-p-7). A hard
  cap on a nominal count is not a hard cap. C1 makes the ledger record the measured charge (§4.6).
- **F-S32p-2 — `cast.py meshy` is tied to `fit.yaml`.** A garment has no `fit.yaml` (A-p-9). C1 adds a second entry
  that checks the input against the pilot's candidate record instead, with the same ledger and cap.
- **F-S32p-3 — MPFB2 stages cannot run with `--factory-startup`** (A-p-3, A-p-28). They run in their own Blender
  invocation with an isolated `BLENDER_USER_RESOURCES` holding only MPFB2 2.0.18, whose zip SHA-256 is recorded.
  This is a documented exception to `PIPELINE.md` §11, not a relaxation of it.
- **F-S32p-4 — Name-based hiding in `human.gd`** (A-p-17). Parts are named `part_<garment-id>`, never `Hoodie` or
  `Pack`.
- **F-S32p-5 — Material names.** The pose sheet matches `MW_Cast` exactly (A-p-16). Every part's one material is
  named `MW_Cast_<part-id>`, the body's `MW_Cast`; the pose sheet's outfit mode matches the `MW_Cast` prefix.
- **F-S32p-6 — The torso check misreads layers** (A-p-18). In outfit mode the check takes the trunk outline from
  the union of all parts' trunk vertices and the limb vertices only from visible surfaces (outermost sleeve, or
  unhidden skin); layer order is W-4's and W-8's business.
- **F-S32p-7 — Skin-tight garments do not separate.** A garment within ≈ 2 mm of skin (Hana's tights; the sides of
  Ivan's close fade) cannot be cut from a Meshy mannequin by an inside test, because nothing of it lies outside the
  test's tolerance. §4.3 R-8b, Q-S32p-5.
- **F-S32p-8 — Base-layer bodies are barefoot**, so every outfit needs footwear; Vera's outfit therefore includes
  her trainers beside the four garments the operator named.
- **F-S32p-9 — Step-32 is internally inconsistent about S26-b2's spike.** §15.3 folds "the landmark, weights and
  Meshy-rig comparisons" into S32-p; §4.2 says the Meshy rig is "not used". This design follows §4.2 for the rig,
  evaluates RSWT (as §4.2 says), and keeps landmark picks hand-supplied from the ortho render (as #211 does) with
  automation deferred to S32-d (Q-S32p-7).
- **F-S32p-10 — The garment mannequin.** Step-32 §6.6 g2 says "the neutral template mannequin". A rendered `hm08`
  mannequin is an MPFB2 stock body in every garment reference image, and those images are shown to the operator.
  This design renders the **wearer's own Meshy base-layer body as grey clay** instead (§4.3 R-6): no MPFB2 asset in
  any image, and the subtraction uses a surface whose shape we already hold. The garment is still bound to the shared
  template (R-9), so it still fits every body. This follows the ruling ("no MPFB2 stock asset ever reaches … the
  operator's review"); it is recorded as a bounded refinement of g2 (§11).
- **F-S32p-11 — Template v2 has no base-layer paragraph**, and editing `template.yaml` makes a new version
  (A-p-20). The base-layer and garment prompts are separate files that quote the v2 preamble from `template.yaml`
  at run time and refuse any file whose `version` is not 2.

---

## 4. Design

### 4.1 Overview

```text
identity reference (approved v2 image)                       Vera's fused character (control, unchanged)
        │                                                                     │
R-1 base-layer reference (OpenAI, sunburst, template v2)  ──pre-spend page──► │
R-2 Meshy base body (≤ 3 runs)                                                │
R-3 body stages 3, 6–10 (#211; 60 k; body + correspondence export)            │
R-4 register hm08 → fitting template T_P (+ weights from the body)            │
R-5 regions on T_P                                                            │
        │                                                                     │
R-6 clay render of the wearer's body ─► R-7 garment reference (OpenAI)        │
R-8 Meshy garment (≤ 3 runs) ─► R-8a subtraction, R-8b tight-fit class        │
R-9 cleanup: shell, retopo, UV, bake, tint mask                               │
R-10 bind: to neutral hm08 → MakeClothes .mhclo → binding.bin                 │
R-11 hidden sets (hides.bin), weights rule                                    │
        │                                                                     │
R-12 outfit assembly: Blender reference fit  +  Godot fit at load (prototype) │
R-13 gates G-1…G-14 per outfit, W-1…W-12, V-1                                 │
R-14 pose sheets (--outfit) for outfit A and B ───────────────────────────────┤
R-15 scorecard, measurements, review page VIS-3D-WARDROBE-P ◄─────────────────┘
```

### 4.2 Module layout (all tooling; nothing outside `tools/character_pipeline/`, `docs/` and the plan files)

```text
tools/character_pipeline/
  PIPELINE.md                         §16 "Route B and the garment pipeline", §17 "Wardrobe gates" (new)
  recipes/s32p-pilot.yaml             subjects, identity references, garments, outfits, budget (§4.5, §4.6)
  wardrobe/
    prompts/base_layer.txt            the base-layer paragraph (quotes the v2 preamble at run time, F-S32p-11)
    prompts/garment.txt               the garment-reference paragraph
    template/hm08_regions.json        the 11-region partition of the 19,158 hm08 vertices, generated by
                                      blender/mpfb/regions.py (our rule; CC0-derived data), with its SHA-256
  mwcast/wardrobe/                    standard-library Python (CI: ruff, pyright, pytest)
    __init__.py
    recipe.py                         parse and validate recipes/s32p-pilot.yaml; unknown keys refused at the line
    budget.py                         the credit plan, phase ceilings, retry pool, measured reconcile (§4.6)
    images.py                         build prompts; call mineworld_gen with --model pinned; check each sidecar
    garment_format.py                 read/write garment.yaml, binding.bin, hides.bin, corr.bin (§4.4)
    wgates.py                         W-1, W-2, W-3, W-12 (colour read), G-1/G-2/G-3/G-14 per outfit, V-1
    outfits.py                        outfit composition; the seeded W-6 sweep; slot exclusivity for the pilot
    cli.py                            `cast.py wardrobe <sub>` (§4.3 table, last column)
  blender/mpfb/                       GPL-3.0-or-later (they call MPFB2's GPL-3 API); LICENSE; never run in CI
    LICENSE                           GPL-3.0 text
    install.py                        install MPFB2 2.0.18 from its zip into the isolated user resources
    regions.py                        R-5's partition rule → hm08_regions.json
    register.py                       R-4
    clay.py                           R-6
    garment_clean.py                  R-8a, R-8b, R-9
    garment_bind.py                   R-10 (MakeClothes headless) and R-11
    assemble.py                       R-12's Blender reference fit and the assembled-outfit GLB
    fixtures.py                       MPFB2 fixture bodies (W-5 extremes) and stock CC0 garments, test-only
  godot/wardrobe/
    fit_at_load.gd                    the prototype (§4.3 R-12)
    outfit_gates.gd                   G-7…G-13 and W-4…W-11 runner, headless where it can be
  godot/pose_sheet.gd                 + `--outfit <manifest>` (§4.3 R-14)
  mwcast/pose_sheet.py                + `--outfit` staging
  tests/test_wardrobe_*.py            unit tests (§6)
```

### 4.3 The stages

Every Blender stage runs through `with-slot.sh`, pinned to `Blender 5.2.2 LTS`; the MPFB stages (`blender/mpfb/`)
run without `--factory-startup` under `BLENDER_USER_RESOURCES=<scratch>/blender-user` (F-S32p-3). Every Godot run
goes through `with-slot.sh`. Every output goes to the pilot scratch root `MINEWORLD_CAST_SCRATCH=<scratch>`
(outside the repository, refused inside it by #211's resolver). Every keyed command is
`source ~/.config/mineworld/secrets.env && …`.

| # | Stage | Input → output | Tool and method | `cast.py` entry |
| --- | --- | --- | --- | --- |
| **R-1** | **Base-layer reference image** | the approved identity reference (Vera: her identity anchor `style/anchor.png`, with `anchor_fullbody.png` as her full-body view and the style; Ivan/Hana: `cast-out/<key>/candidate_<pick>.png`) → 3 candidates 1024×1536 | `mineworld_gen` `images.edit`, **`--model gpt-image-2.5-sunburst`**, quality high, opaque; references in order: identity reference FIRST (identity), `anchor_fullbody.png` (style). Prompt: the v2 full-body preamble verbatim (read from `template.yaml`, version 2 required) + `prompts/base_layer.txt`: the same person, same face, skin, age, build and stature; a plain fitted mid-grey sleeveless athletic tank top and fitted mid-grey athletic shorts ending at mid-thigh; barefoot; all hair tucked under a smooth close-fitting cap in the person's skin tone; A-pose with clear gaps; no accessories. The sidecar's `model` must read `gpt-image-2.5-sunburst` or the candidate is refused (I-p-2). The agent's raw check against the identity reference is written to `candidates.md` | `wardrobe image base-layer --subject <key>` |
| — | **Pre-spend page** (step-26 §6.4 gate 1; step-32 §16) | per subject: the identity reference beside the 3 base-layer candidates, the agent's ranking and misses | static HTML in `mineworld-demos/wardrobe-pilot/prespend.html`; the operator picks one or asks a redo; the pick is recorded in the recipe with the SHA-256 (§4.9) | — |
| **R-2** | **Meshy base body** | the picked base-layer image → download in `<scratch>/meshy/<subject>/body-<n>/` | #211's stage 3 mask (BiRefNet) then `meshy.run_live` with the defaults (A-p-8); the pilot ledger and cap (§4.6); retry budget: 2 retries per body. Raw gate (step-26 stage 5) on the multi-view thumbnails: hard-fail categories against the identity reference; the skin cap reads as a smooth head; base layer intact; identity | `wardrobe meshy body --subject <key>` |
| **R-3** | **Body cleanup and rig** | the kept download → `<subject>.glb` (body only), `<subject>_corr.bin` | #211's stages 6–10 unchanged, with `fit.yaml` for the route-B body: `gameready.target_triangles: 60000` (W-2's body budget; one override), `accessories: []` (the backpack is a garment), `flecks.enabled: true`; picks hand-supplied by the agent on stage 7's ortho render (`render_ortho.py`) and drawn over the body for G-6 (Q-S32p-7). Export as #211 does; the correspondence file is written by R-4 | `run` (existing) |
| **R-4** | **Registration of `hm08` to the body** | `<subject>.glb` → `<subject>_template.bin` (19,158 × 3 f32, the fitting template T_P), `<subject>_template_weights.bin`, `<subject>_corr.bin` | `blender/mpfb/register.py`: (1) `create_human` with macros from the brief (sex, age, weight, height; the stature calibrated by bisection on the `height` macro until the template's sole-to-crown equals the body's within 5 mm, R-S32-6); (2) `add_builtin_rig("mixamo")` — its 52 names equal ours (P-2); (3) pose that rig onto the body's fitted 52 joints (joint heads and rolls from `dump_rig.json`) and apply linear-blend skinning, so the limbs match; (4) non-rigid surface fit: Shrinkwrap (nearest surface point, normal agreement ≤ 60°) then Laplacian smoothing (Corrective Smooth), face, hands and feet weighted to follow the joint chain; (5) T_P's skin weights transferred **from the body** (Data Transfer, nearest face interpolated; RSWT evaluated on Vera, §4.10 D-p-4); (6) per body vertex, the closest T_P triangle and barycentrics (`corr.bin`). Printed: mean and 95th-percentile distance T_P → body per region | `wardrobe register --subject <key>` |
| **R-5** | **Regions** | `hm08` → `wardrobe/template/hm08_regions.json` (once, committed) | `blender/mpfb/regions.py`: each template vertex takes the region of its dominant `mixamo` bone (Head → scalp or face by the brow plane and the ear line; Neck → neck; Spine, Spine1, Spine2 → torso; Arm → upper-arms; ForeArm → forearms; Hand and fingers → hands; Hips → hips; UpLeg → thighs; Leg → shins; Foot, ToeBase → feet). The rule is the data's source; the file's SHA-256 is recorded | `wardrobe regions` |
| **R-6** | **Clay mannequin** (F-S32p-10) | the wearer's `<subject>.glb` → `clay_<subject>.png` 1024×1536 | `blender/mpfb/clay.py` (no MPFB call; it lives beside the others for the scratch environment): the body in a uniform mid-grey matte material, the skin cap included, at the template v2 camera (85 mm equivalent, navel height, about 6 m, front view, the v2 backdrop colour `#C8C4BE`) | `wardrobe clay --subject <key>` |
| **R-7** | **Garment reference image** (g1–g2) | garment brief (category, slots, fabric, colours, from the subject's brief and identity reference) → 2 candidates | `mineworld_gen` `images.edit`, sunburst, references in order: `clay_<wearer>.png` (shape and pose), the identity reference (the garment's look: Vera's hoodie, tee graphic, backpack), `anchor_fullbody.png` (style). Prompt: v2 preamble + `prompts/garment.txt`: "the grey clay figure wearing only <the garment> exactly as in the second image; the figure stays plain grey clay; nothing else worn"; hair styles use a head-and-shoulders crop of the clay figure. Agent raw gate (Q-S32p-3) | `wardrobe image garment --garment <id>` |
| **R-8** | **Meshy garment** (g3) | the picked garment image → download | stage 3 mask, then `meshy.run_live` (defaults); retry budget 2 per asset; raw gate: the garment's defining structures present (hood, cords, graphic, straps, pleats, band), no garment fused into the clay beyond the inside test's reach | `wardrobe meshy garment --garment <id>` |
| **R-8a** | **Subtraction** | download → `garment_raw.blend` | `blender/mpfb/garment_clean.py`: register the wearer's T_P onto the generated clay figure exactly as R-4 step 4 does (the clay is the wearer's own shape, so the fit converges from the posed start); delete every vertex within 1.5 mm inside or on the registered surface, and every connected component whose vertices are ≥ 95 % within 4 mm of it; print the kept and removed counts per region | `wardrobe clean --garment <id>` |
| **R-8b** | **Tight-fit class** (F-S32p-7; Q-S32p-5) | the wearer's body + the garment reference → shell | for `fit: tight` garments only (pilot: Hana's tights; Ivan's hair sides are kept by R-8a's 1.5 mm tolerance only if they survive it, else the hair's top volume is generated and the sides are a tight-fit shell): a shell offset 0.8 mm outward from the wearer's **own Meshy body** over the garment's declared regions, textured by projecting the picked garment reference image in the v2 front view and the agent-checked back view, baked to 1024². No Meshy run; no MPFB2 asset | same entry, `fit: tight` |
| **R-9** | **Cleanup** (g4) | → `garment.glb` in the wearer's rest pose | single-sided shell with 1.2 mm thickness (Solidify, inner faces only where the shell opens); decimate to the slot budget (W-2); UV unwrap (Smart UV Project, 1 % margin) and a bake of albedo from the Meshy texture at 1024² (2048² for outer and top when W-3 needs it), a normal map from the high-poly, ORM with roughness from albedo luminance and AO; the tint mask from albedo segmentation (k-means in Lab, 3 clusters → R primary, G secondary, B trim); one material `MW_Cast_<id>` | `wardrobe clean` (continued) |
| **R-10** | **Binding** (g5) | `garment.glb` → `<id>.mhclo` → `binding.bin` | `blender/mpfb/garment_bind.py`: (1) move the garment from the wearer's T_P space to the **neutral** `hm08` (default macros, A-pose rest): bind it to T_P with Surface Deform, morph T_P to the neutral template through their shared topology, apply; (2) MakeClothes headless on the neutral base mesh → `.mhclo` (vertex triples, weights, offsets, scale references, `delete_verts` when given); (3) convert to `binding.bin` (§4.4). The neutral-space `garment.glb` is the shipped rest shape | `wardrobe bind --garment <id>` |
| **R-11** | **Hidden set and weights rule** | → `hides.bin`; `garment.yaml` | ray-cast each neutral-template vertex outward along its normal against the garment shell in A-pose and in the W-gate pose set (stand, idle, walk and jog phases, sit), skinned with the template weights; a vertex is hidden only if covered with ≥ 3 mm clearance in **every** pose (conservative, step-32 §6.3). `weights: inherit` (from the template triangle) or `weights: rigid <bone>` (the backpack: `Spine2`, so G-12 holds) | `wardrobe bind` (continued) |
| **R-12** | **Outfit assembly** | body + T_P + parts → (a) Blender reference fit, (b) Godot fit at load | (a) `blender/mpfb/assemble.py`: fit each part to T_P by the `.mhclo` formula (§4.4), layer push-out in slot order, weights by the part's rule, remove the body triangles whose three vertices map to hidden template vertices, export `outfit_<subject>_<A|B>.glb` (one armature, one skinned mesh per part, `part_<id>` names, `MW_Cast_<id>` materials), with provenance per part. (b) `godot/wardrobe/fit_at_load.gd`: the same formula in GDScript on `garment.glb` (imported as a static mesh) + `binding.bin` + T_P; weights from `template_weights.bin`; normals and tangents by `SurfaceTool`; **each part reuses the body `MeshInstance3D`'s `Skin` resource**, so no bind name is resolved and the importer's `mixamorig:` → `mixamorig_` rename cannot bite; any direct bone lookup uses the imported names (`mixamorig_Hips`, or the bone map's profile names after retarget); the body index buffer is rebuilt without hidden triangles; per-change times are printed | `wardrobe assemble --subject <key> --outfit <A|B>` |
| **R-13** | **Gates** | the assembled outfits, every part | §4.8 | `wardrobe gate --subject <key> --outfit <id>`; `wardrobe sweep` |
| **R-14** | **Pose sheets** | per subject, outfits A and B | `pose-sheet <body.glb> --outfit <manifest.yaml> --out …` (the manifest lists T_P, parts and order); the subject is assembled by `fit_at_load.gd` in the scratch project; the frames P-1 (front, three-quarter, rear), P-2 (contact, passing), P-3, P-4 (three-quarter, side), P-6 (two-shot with the fused Vera), P-8, the lineup with the fused Vera; plus one **outfit pair frame** (A, B and the fused Vera side by side, front, same light) | `pose-sheet --outfit` |
| **R-15** | **Scorecard and review page** | frames, gates, measurements | the S26 scorecard (step-26 §6.2) per subject per outfit beside Vera; the review page (§4.9) | `wardrobe page` |

### 4.4 The runtime garment format as the pilot writes it (`ARC-140`, format part)

Directory per garment in scratch: `garments/<id>/` with `garment.yaml`, `garment.glb` (neutral `hm08` rest shape,
one surface, no skin), `binding.bin`, `hides.bin`, textures. Per subject: `<subject>_template.bin`,
`<subject>_template_weights.bin`, `<subject>_corr.bin`.

```yaml
# garments/vera-hoodie/garment.yaml
format: 1
id: vera-hoodie
category: hoodie                # step-32 §11.3 vocabulary
slots: [outer]
layer: 4                        # push-out order inside the slot stack (§4.5's order)
covers_regions: [torso, upper-arms, forearms]
fit: shell                      # shell | tight (R-8b)
weights: inherit                # inherit | rigid <bone>
mesh: garment.glb
binding: binding.bin
hides: hides.bin
material: { name: MW_Cast_vera-hoodie, albedo: albedo.png, normal: normal.png, orm: orm.png, tint_mask: tint.png,
            variants: { default: null, alt: slate } }   # W-12
tris: 9400
provenance: { source_type: generated, meshy_task: <uuid>, reference: { path: …, sha256: … }, image_model:
              gpt-image-2.5-sunburst, template_version: 2, pipeline: { version: …, blender: Blender 5.2.2 LTS,
              mpfb: 2.0.18 } }
```

**`binding.bin`** (little-endian): magic `MWGB`, `u32` version 1, `u32` vertex count `n`, three scale references
(`u32 a, u32 b, f32 rest_length` for x, y, z), then `n` records of `u32 v0, v1, v2; f32 w0, w1, w2; f32 ox, oy, oz`.
The fitted position of garment vertex `k` on template `T` is
`Σ_i w_i · T[v_i] + (ox · |T[a_x] − T[b_x]| / rest_x, oy · …, oz · …)`, the `.mhclo` formula (step-32 §6.2).
**`hides.bin`**: magic `MWGH`, `u32` count, sorted `u32` template vertex ids. **`corr.bin`**: magic `MWGC`, `u32`
body vertex count, per vertex `u32` template triangle id + `f32 b0, b1`. **`template.bin`**: magic `MWGT`, `u32`
19,158, positions. **`template_weights.bin`**: magic `MWGW`, per vertex 4 × (`u8` joint index into the body's
skin, `f32` weight). The readers and writers are `mwcast/wardrobe/garment_format.py` (stdlib `struct`) and
`fit_at_load.gd`; W-1 reads the binding.

**Layer push-out.** In slot order (inner first), for each outer-garment vertex, the inner layers' fitted surfaces are
ray-cast along the template normal; a vertex less than 2 mm outside the outermost inner hit is moved to 2 mm outside
it. The same rule runs in Blender (R-12 a) and Godot (R-12 b), so W-9 compares like with like.

### 4.5 The pilot library and outfits

Slot order (inner → outer, for push-out and W-8): `socks`, `legs`, `top`, `outer`, `feet`, `head`, `bag`.

| Garment id | Wearer (generated on) | Category | Slot | Class | Source |
| --- | --- | --- | --- | --- | --- |
| `vera-tee` | Vera | tee (her graphic) | top | shell | Meshy |
| `vera-hoodie` | Vera | hoodie (hood, cords, cuffs) | outer | shell | Meshy |
| `vera-jeans` | Vera | jeans | legs | shell | Meshy |
| `vera-trainers` | Vera | shoes-trainer | feet | shell | Meshy (F-S32p-8) |
| `vera-backpack` | Vera | backpack | bag | shell, `rigid Spine2` | Meshy |
| `ivan-polo` | Ivan | polo | top | shell | Meshy |
| `ivan-jacket` | Ivan | jacket-casual (linen) | outer | shell | Meshy |
| `ivan-trousers` | Ivan | trousers | legs | shell | Meshy |
| `ivan-shoes` | Ivan | shoes (walking) | feet | shell | Meshy |
| `ivan-panama` | Ivan | hat-panama | head | shell | Meshy |
| `hana-shirt` | Hana | shirt-collared | top | shell | Meshy |
| `hana-jacket` | Hana | jacket-cropped | outer | shell | Meshy |
| `hana-skirt` | Hana | skirt-knee (pleated) | legs | shell | Meshy |
| `hana-flats` | Hana | shoes-flat | feet | shell | Meshy |
| `hana-tights` | Hana | tights | socks (covers hips, thighs, shins, feet) | **tight** | R-8b, no Meshy (Q-S32p-5) |
| hair `vera-updo` | Vera | messy updo with face strands | — | shell | Meshy |
| hair `ivan-fade` | Ivan | close fade | — | shell (+ tight sides if needed) | Meshy |
| hair `hana-pixie` | Hana | pixie cut | — | shell | Meshy |

That is 14 garments + 1 tight-fit garment and 3 hair styles. Step-32's "12 garments + 4 hair styles" was an
estimate before the outfits were named; the deliverable needs these 18 assets and no more. A fourth hair style is
not needed for any outfit and is dropped (bounded; §16 Deviations).

| Subject | Outfit A (starting, from the brief) | Outfit B (different; all from the shared library, so it proves cross-body fitting) |
| --- | --- | --- |
| Vera | `vera-tee`, `vera-hoodie`, `vera-jeans`, `vera-trainers`, `vera-backpack`; hair `vera-updo` | `hana-shirt`, `hana-jacket`, `ivan-trousers`, `hana-flats`; hair `vera-updo` |
| Ivan | `ivan-polo`, `ivan-jacket`, `ivan-trousers`, `ivan-shoes`, `ivan-panama`; hair `ivan-fade` (under the hat) | `ivan-polo`, `vera-hoodie`, `vera-jeans`, `vera-trainers`; hair `ivan-fade` (no hat) |
| Hana | `hana-shirt`, `hana-jacket`, `hana-skirt`, `hana-tights`, `hana-flats`; hair `hana-pixie` | `vera-tee`, `ivan-jacket`, `vera-jeans`, `ivan-shoes`; hair `hana-pixie` |

Outfit B changes the top, the outer layer, the legs (Hana: skirt → jeans) and the feet for everyone, so the
silhouettes differ. A blocked B-garment may be replaced by another library garment of the same slot (recorded); a
blocked A-garment blocks that subject.

### 4.6 The Meshy credit budget and the hard stop

Basis: 30 credits per image-to-3D run (A-p-7; Vera's three runs, A-p-27). The projection for the next run is
`max(30, the largest consumed_credits measured so far in this pilot)`.

| Phase | Runs | Nominal credits | Ceiling (hard) |
| --- | --- | --- | --- |
| V-body: Vera's body | 1 + ≤ 2 retries | 30–90 | 90 |
| V-garments: Vera's 5 garments + 1 hair | 6 + ≤ 4 retries (≤ 2 per asset) | 180–300 | **phase V total 390** |
| IH-body: Ivan, Hana | 2 + ≤ 4 retries | 60–180 | — |
| IH-garments: Ivan 5 + 1 hair, Hana 4 + 1 hair | 11 + retries from the pool | 330–… | — |
| **Pilot total** | 20 first runs = **600**; retry pool **600** | | **1,200** |

Rules, enforced by `mwcast/wardrobe/budget.py` and the generalised ledger (C1):

1. **One ledger** for the pilot: `<scratch>/credits.json`. The cap (1,200) and the phase ceilings live in
   `recipes/s32p-pilot.yaml`; `cast.py wardrobe meshy` reads them and has **no `--cap` argument**, so the cap cannot
   be raised on the command line.
2. **Before every POST:** refuse when `spent + projection > 1,200`, or when the run would exceed its phase ceiling or
   its asset's retry budget. Nothing is sent; the refusal names the rule (I-p-3).
3. **After every task:** the ledger replaces the run's provisional 30 with the measured `consumed_credits`, and the
   balance delta (`balance_before − balance_after`) is recorded beside it; a disagreement between the two is a
   finding and the larger is charged.
4. **Phase gate IH:** before Ivan's first run, `1,200 − spent ≥ 450`: the IH first runs (2 bodies + 11 assets = 390) plus one
   retry for each body (60). Otherwise stop and report: a subject is never started that the budget cannot finish.
5. **Reports:** at 600 credits (50 % of the cap) and at the end, each with the measured cost per kept body and
   per kept garment, and the S26 running total against 3,000 (the pilot's spend plus any earlier S26-step spend
   the primary records; Vera's 90 of 2026-10-07 predate the S26 approval and are listed separately).
6. **No other Meshy endpoint** is called (no rig, no remesh, no Auto Split, no retexture).

### 4.7 OpenAI image calls

| Use | Calls (first + replacements) |
| --- | --- |
| R-1 base-layer: 3 subjects × 3 candidates, ≤ 2 replacements each | 9 + ≤ 6 |
| R-7 garment and hair references: 17 Meshy assets × 2 candidates, plus the tight-fit garment's front and back views; ≤ 1 replacement per asset | 36 + ≤ 18 |
| **Cap (Q-S32p-6, ruled 2026-10-15)** | **≤ 70 calls** ≈ $3.9 at the measured $0.0563 per 1024×1536 high call (A-p-23) |

Every call: `mineworld_gen` from the tree that contains `bd7bc92f`, `--model gpt-image-2.5-sunburst` passed
explicitly, `--size 1024x1536 --quality high`, the sidecar checked after the call (I-p-2). The call count and the
estimated cost are logged per subject in `<scratch>/images.json`.

### 4.8 Gates

**Per assembled outfit — the restated S26 gates (`ARC-141`, step-32 §7.1).**

| Gate | Pilot form | Instrument |
| --- | --- | --- |
| G-1 | body + all worn parts ≤ 100,000 triangles | `wgates.py` over the assembled GLB |
| G-2 | exactly one material per part (`MW_Cast` for the body, `MW_Cast_<id>` per part); parts ≤ 8; no unbound surface | `wgates.py`; the Godot import log (no `human.gd: no material` warning) |
| G-3 | body 2048²; each part 1024² or 2048², as its `garment.yaml` declares; sRGB; mipmaps | `wgates.py` |
| G-4, G-5, G-6 | unchanged, on the body (#211 gates); G-5 also on every part: the part's skin is the body's skin (same 52 joints), ≤ 4 influences, sums within 1e-4 | #211 `gates.py`; `wgates.py` for parts |
| G-7 | the assembled outfit imports and builds with no error and no unbound-surface warning; `A_TPose` clean (arms level within 5°, legs straight, feet flat) | `outfit_gates.gd` |
| G-8 | `Idle`, `Walk`, `Jog_Fwd`, `Sit`, stand clips play; clip speeds measured on this body (`clients/3d-spike/tools/measure_stride.gd`'s method); planted-foot drift ≤ 25 % of body speed, contact ≥ 50 % | `outfit_gates.gd` |
| G-9 | stand, idle, walk and jog sweeps (8 phases each) and sit: no triangle stretched beyond 3× its rest edge outside the shoulders' and hips' known ranges; the torso check as F-S32p-6 restates it, 5 mm | `outfit_gates.gd`; pose sheet |
| G-10 | head yaw, pitch, roll each ≤ 6° peak-to-peak in steady walk and jog | `outfit_gates.gd` |
| G-11 | finger bones ≥ 10 vertices weighted > 0.5 (thumb ≥ 5); the curl closes the hand by ≥ 40 % | #211 Blender weights; `outfit_gates.gd` |
| G-12 | the backpack part: no upper-arm or forearm weight share; no slit in the jog sweep at 2×; sleeve and side panel never interpenetrate in the walk sweep | `wgates.py` (weights); `outfit_gates.gd` |
| G-13 | `Sit` at the café chair: thighs, hips and hem without tear; seat contact ±20 mm | pose sheet P-4 |
| G-14 | provenance on the body and on every part and hair asset (Meshy task id, references by path and SHA-256, image model, template version, pipeline, MPFB2 version for the binding), no key-shaped string | `wgates.py` (extends #211's G-14 reader) |

**Wardrobe gates W-1…W-12** — exactly step-32 §7.2's requirements, bounds and sources; the pilot's instruments:

| Gate | Instrument in the pilot |
| --- | --- |
| W-1 Binding | `garment_format.py` reader: every triple a valid template triangle, weights sum 1 ± 1e-4, scale references valid |
| W-2 Part budget | `wgates.py`: body ≤ 60 k; hair ≤ 12 k; top or outer ≤ 10 k; legs ≤ 8 k; feet ≤ 4 k; socks/tights ≤ 1.5 k; accessory ≤ 4 k |
| W-3 Texel density | `wgates.py`: UV area / surface area per part within ± 15 % of the **fused Vera's** body density (computed once from `meshy_d.glb` and written into `recipes/s32p-pilot.yaml` as a literal with its derivation) |
| W-4 Poke-through | `outfit_gates.gd`: per garment × shape (W-5) × pose (stand, idle, walk, jog phases, sit), ray tests from skinned positions: no visible body vertex outside the garment shell by > 2 mm within its covered regions; no inner-layer vertex outside an outer layer by > 2 mm; each failure printed with vertex, region, pose, shape |
| W-5 Shape sweep | the three pilot bodies + five **fixture** bodies built by `blender/mpfb/fixtures.py` (shortest woman 1.55 m, tallest man 1.92 m, heaviest and lightest build of the San Diego profile, oldest stooped), stature-calibrated; fixtures are hidden test bodies, never drawn on any sheet |
| W-6 Outfit sweep | per subject, outfit A and three seeded random library outfits (`outfits.py`, seed `s32p-w6`, one garment per slot, slot-compatible) through G-7…G-13 and W-4 |
| W-7 Mask coverage | ID-buffer render (body flat colour, each garment another) at four views per sweep frame: no hidden-body triangle visible, no gap between a hem and visible skin |
| W-8 Layering | every `over`/`under` pair in §4.5's slot order renders outer over inner through the walk sweep |
| W-9 Fit at load | `fit_at_load.gd`'s fitted vertices vs `assemble.py`'s reference fit: ≤ 0.5 mm per vertex, every garment on every pilot body; every part's skin is the body's |
| W-10 Swap | wear and take off each garment in sequence: the body index buffer equals a fresh build's, byte for byte |
| W-11 Hair under headwear | every hair × `ivan-panama`: W-4 applied to hair (the `under_headwear: hidden` variant for the pilot) |
| W-12 Variant | each garment's `default` and `alt` variant rendered under the neutral review light: ΔE₀₀ ≤ 5 to the palette value in the tint-mask region |

**V-1 No-stock scan (the ruling; the pilot's form of AC-S32-13).** Every mesh and texture drawn in any deliverable
frame (each part, hair and body of each assembled outfit) carries provenance whose `meshy_task` is a pilot task or
whose `fit: tight` source names a pilot body; any provenance naming an MPFB2/MakeHuman stock asset, or a part with
no provenance, fails. The fixtures and the template carry `role: fixture` / `role: fitting-template, drawn: false`
and are refused by V-1 if they appear in a drawn part.

### 4.9 The review page `VIS-3D-WARDROBE-P` and the records

- **Where:** `/Users/yuema137/mineworld-demos/wardrobe-pilot/index.html` (private, outside the repository, like
  `mineworld-demos/casting/`), in Chinese with English terms (the operator's language), images as JPEG.
- **Per subject:** the identity reference; the picked base-layer image; the full pose sheet for **outfit A** and for
  **outfit B** (P-1 front, three-quarter, rear; P-2 contact and passing; P-3; P-4 three-quarter and side; P-6
  two-shot with the fused Vera; P-8; the lineup with the fused Vera); the outfit pair frame (A | B | fused Vera); the
  scorecard per outfit; the gate table (every G and W gate per outfit with PASS/FAIL and where it looked); red
  frames shown red, never hidden.
- **Header:** the decision rule (§0), the credits spent and remaining, the measured costs, the full-budget
  re-estimate for S26 + S32 with measured numbers, and the question "route B 达到 v2/Vera 的观感了吗？" with
  accept / reject per subject and overall.
- **If the pilot fails** (any subject below the bar after its budget): the page leads with **"route B 未达到 v2/Vera
  观感，不采用"**, the diagnosis per miss, the R-S32-1 levers (a)–(d) each with a cost estimate, and no further
  spend proposed without the operator.
- **Records committed** (no binaries, A-p-13): `recipes/s32p-pilot.yaml` with picks and SHA-256s; per subject
  `tools/character_pipeline/pilot/s32p/<subject>/{fit.yaml, generation_log.yaml}`; per garment
  `pilot/s32p/garments/<id>/{garment.yaml, generation_log.yaml}`; `pilot/s32p/report.json` (gates, measurements,
  credits); `docs/HUMAN_REVIEW_QUEUE.md` gains the `VIS-3D-WARDROBE-P` row (🟡 READY FOR HUMAN VISUAL REVIEW, with
  the page path); the decision records of §2.1 item 8.

### 4.10 Design decisions

- **D-p-1 — The wearer's clay, not an MPFB2 mannequin** (F-S32p-10).
- **D-p-2 — Skin reuse in the fit-at-load prototype** (R-12 b): parts share the body's `Skin`, which removes name
  resolution and the `mixamorig_` pitfall from the runtime path; S32-c may keep or change it, with W-9 as the check.
- **D-p-3 — Bodies at 60 k.** Route-B bodies are decimated to W-2's 60 k, not D+'s 95 k, so G-1 holds for every
  outfit by construction.
- **D-p-4 — Weights.** T_P's weights come from the body by Data Transfer; RSWT is evaluated once on Vera (outfit A,
  W-4 and G-9 failure counts in the walk and jog sweeps) and adopted for the pilot only if it lowers them; the result
  is recorded in `DEP-73`.
- **D-p-5 — Vera's control is never rebuilt.** The fused Vera (`clients/3d-spike/assets/characters/meshy_d/`) is
  read, never written; every sheet carries it.
- **D-p-6 — Garments are generated once, on their first wearer**, and fitted to everyone else through the template:
  that is the claim W-5, W-6 and outfit B test.

### 4.11 Measurements (for the full-budget question)

| Quantity | How |
| --- | --- |
| Credits per kept body, per kept garment; retention per class | the ledger and `generation_log.yaml` |
| OpenAI calls and estimated cost per subject and per garment | `images.json` |
| Bytes: per body (texture as imported, mesh, template 0.23 MB), per garment (mesh, maps) — raw and with UASTC | a Godot headless export of a scratch pack with the pilot assets (`--export-pack`), sizes per file |
| Per-change rebuild time (fit at load + mask rebuild) | `fit_at_load.gd` timings, median of 20 changes per subject, on the operator's M5 |
| Frame time, 16-person crowd | a scratch scene: the three pilot subjects × outfits A/B/W-6 outfits = 16 people at 3–15 m in the café street light, RL-b's protocol (1920×1080, three fresh runs, median p95, `tools/perf_compare.py`'s verdict per R-2); one run at a time with the load average recorded (R-LOAD-1); indicative only (the client integration is S32-c's) |

---

## 5. Invariants

| Id | Invariant | Source | Checked by |
| --- | --- | --- | --- |
| I-p-1 | Template **v2** only: every prompt quotes `template.yaml` at `version: 2`; any other version is refused before a call | ruling; F-S32p-11 | `test_wardrobe_images.py` |
| I-p-2 | Every OpenAI call passes `--model gpt-image-2.5-sunburst`; a sidecar naming another model refuses the candidate; calls come from a `mineworld_gen` containing `bd7bc92f` | ruling | `test_wardrobe_images.py`; C1 records `git merge-base --is-ancestor bd7bc92f HEAD` |
| I-p-3 | **≤ 1,200 Meshy credits**, refused **before** the POST; measured charges reconciled; no CLI cap override; inside S26's 3,000 | ruling; F-S32p-1 | `test_wardrobe_budget.py`; `test_meshy.py` additions |
| I-p-4 | **No MPFB2 stock asset** (body, face, skin, hair, garment) appears in any image sent to a model, in any deliverable frame, or on any page as the look; the template is hidden data | ruling; F-S32p-10 | V-1; the page builder refuses a frame whose manifest fails V-1 |
| I-p-5 | The 52-joint skeleton is unchanged (names, parents, rolls) on every body; every part's skin is the body's | `HUMANOID_PROFILE.md` §1; G-5 | G-5, W-9 |
| I-p-6 | Every gate bound is a literal from Vera's accepted record or a cited reference, written in the recipe or `standard.py` with its source; never computed from a candidate | `ARC-23` rule 2 | review question of each gate commit; `test_wardrobe_gates.py` asserts no bound is an argument |
| I-p-7 | Keys are never read, printed, logged, copied or committed; keyed commands run as `source ~/.config/mineworld/secrets.env && <cmd>`; no output contains a key-shaped string | step-26 §5.5 | #211's secrets test and G-14 scan; V-1's provenance scan |
| I-p-8 | Every Blender and Godot run goes through `with-slot.sh` | operator | the commands in §8; the handoff's job log |
| I-p-9 | No file outside `tools/character_pipeline/`, `docs/DECISIONS.md`, `docs/HUMAN_REVIEW_QUEUE.md` and `.structured-coding/plans/mvp0/` changes | §2.2 | C8's `git diff --name-only origin/main...HEAD` |
| I-p-10 | No MPFB2 code is vendored, imported by the MIT driver, or run in CI; `blender/mpfb/` is GPL-3.0-or-later and runs only inside Blender on the operator's machine | step-32 §4.6; `DEP-72` | `test_wardrobe_confined.py` (no `mwcast` module imports `bpy`/`bl_ext`; no CI command names `blender/mpfb`) |
| I-p-11 | No generated binary is committed | `PIPELINE.md` §10; Q-S26-10 | C8's `git diff --stat` (no GLB, PNG, BIN, blend) |
| I-p-12 | The fused Vera and every client asset are read-only | D-p-5 | C8's diff |
| I-p-13 | A failed pilot is reported as failed; nothing "close" is shipped, labelled accepted or committed as an asset | ruling | the page builder's header rule; review of C7 |

---

## 6. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | ruff, ruff format, pyright over `mwcast/`, `tests/`, `cast.py` (CI `fast`, `python`) | CI |
| Unit (stdlib, CI on Linux, macOS, Windows) | `test_wardrobe_recipe.py` (unknown keys, missing picks, slot conflicts refused at the line); `test_wardrobe_budget.py` (phase ceilings, pool, the IH gate, reconcile with measured and balance delta, refusal before POST through #211's fake `Transport`); `test_wardrobe_images.py` (prompt assembly from a v2 fixture, v3 refused, `--model` always passed, a sidecar with another model refused); `test_wardrobe_format.py` (binding/hides/corr/template round-trips; W-1 on hand-built bindings; the `.mhclo` formula against a hand-computed fixture of three vertices); `test_wardrobe_gates.py` (G-1/G-2/G-3/G-14 per outfit and W-2/W-3/W-12 on small hand-built GLB fixtures from #211's `gltf_fixture.py`; V-1 red on a fixture-provenance part); `test_wardrobe_outfits.py` (the seeded W-6 sweep is reproducible and slot-compatible); `test_wardrobe_confined.py` (I-p-10) | `tools/character_pipeline/tests/` |
| Real lifecycle, local (Gate 2, macOS, operator's machine) | Blender + MPFB2 stages, Meshy runs, Godot fit at load, sweeps, pose sheets, perf — the pilot itself; each run's claim, command and verdict recorded in §16 before launch (test rules §15) | `<scratch>`; evidence in §16 and `pilot/s32p/report.json` |
| Gate 1 (real LLM) | NOT REQUIRED: no language model is involved; the image model is a content generator whose outputs are judged by the raw gates and the operator | — |
| CI | the PR's runs on the exact head (docs, fast, python on three platforms) | GitHub Actions |

Mutations (each planted, the named check turns red, reverted from a byte copy, green again; never committed):
**M-p-1** X-S32-5 — `vera-tee` shell shrunk 4 mm on the chest → W-4 red with location, the pose-sheet frame red.
**M-p-2** X-S32-11 — `hana-shirt`'s `hides.bin` emptied → W-7 red. **M-p-3** a binding weight perturbed by 1e-3 →
W-1 red. **M-p-4** a part over its W-2 budget → W-2 red. **M-p-5** a part's provenance without `meshy_task` →
G-14 and V-1 red. **M-p-6** an MPFB2 stock T-shirt (`s32-proto` fixture) swapped into outfit A → V-1 red and the page
builder refuses the sheet. **M-p-7** the ledger set to 1,190 → the next run is refused, no POST (fake transport
records zero requests). **M-p-8** a garment fitted in Godot with one record's `v1` off by one → W-9 red. **M-p-9**
`template.yaml` with `version: 3` → refused before any call. **M-p-10** a part named `Hoodie` → the import check
reports the hidden part (F-S32p-4).

---

## 7. Acceptance criteria (fixed before measuring)

| Id | Criterion | Step ref |
| --- | --- | --- |
| AC-p-1 | **The deliverable exists per subject:** for Vera, Ivan and Hana, two pose sheets (outfits A and B) with P-1 (three views), P-2 (two), P-3, P-4 (two), P-6 with the fused Vera, P-8, the lineup with the fused Vera, and the A/B/fused-Vera pair frame, on `VIS-3D-WARDROBE-P` | ruling; step-32 §16 checkpoint |
| AC-p-2 | **Gates:** every assembled outfit (6 deliverable + 9 W-6 sweep outfits) has a recorded PASS/FAIL for G-1…G-14 and W-1…W-12 with where each looked; route B is reported as passing the gates only if all pass | step-32 §§5.2, 7 |
| AC-p-3 | **Scorecard beside Vera:** each subject × outfit has the step-26 §6.2 scorecard, judged beside the fused Vera at the same framing and light (§6.3), with the largest miss first | step-26 §§6.2–6.3 |
| AC-p-4 | **No MPFB2 stock asset** in any drawn part or model input (V-1 PASS on every deliverable outfit; M-p-6 red) | ruling; AC-S32-13 |
| AC-p-5 | **Credits:** the ledger total ≤ 1,200 with every run's measured charge; the 50 % report made; refusal before POST proven (M-p-7) | ruling; I-p-3 |
| AC-p-6 | **Image model:** every candidate sidecar names `gpt-image-2.5-sunburst`; the call count ≤ the ruled OpenAI cap | ruling |
| AC-p-7 | **X-S32-5** (M-p-1) red with location and a red sheet frame; **X-S32-11** (M-p-2) red | step-32 §16 adversarial |
| AC-p-8 | **Cross-body fitting:** outfit B for each subject passes W-4, W-5, W-9 (the garments were generated on other people) | step-32 §6.2, D-p-6 |
| AC-p-9 | **Measurements** of §4.11 recorded with method and machine; the full-budget re-estimate written from them | step-32 §16 "Measured" |
| AC-p-10 | **Verdict honesty:** the page states the decision rule; if any subject misses, the page leads with "not adopted" and the R-S32-1 levers with costs; no asset is labelled accepted | ruling; I-p-13 |
| AC-p-11 | **Records:** `ARC-138`, `ARC-140` (format), `ARC-141`, `DEP-72`, `DEP-73`, `DEP-74` written, the route verdict field `pending VIS-3D-WARDROBE-P`; `check_decision_ids.py` green | step-32 §19.1 |
| AC-p-12 | **Confinement:** I-p-9, I-p-10, I-p-11 hold on the final head | §5 |

---

## 8. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason.

### 8.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s32p` (its own worktree, branch `mvp0/pr-s32p`).
Scratch: `export MINEWORLD_CAST_SCRATCH=/Users/yuema137/mineworld-worktrees/cast-scratch-s32p`. Slot wrapper:
`S=/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`.

1. Re-read this commit's section, the design sections it cites, and every file you will edit, in full.
2. Make the steps in order; nothing that belongs to a later commit.
3. Before any live run (Blender, Godot, Meshy, OpenAI), write its Gate specification in §16 (claim, command,
   budget, expected PASS evidence); after it, classify PASS / FAIL / INCONCLUSIVE from the artifacts, not the exit
   code (test rules §§15–16).
4. Unit tests one file at a time: `uv run --locked pytest tools/character_pipeline/tests/<file>`; static:
   `uv run --locked ruff check tools/character_pipeline`, `uv run --locked ruff format --check
   tools/character_pipeline`, `uv run --locked pyright tools/character_pipeline/mwcast tools/character_pipeline/tests
   tools/character_pipeline/cast.py`; docs: `python3 scripts/ci_layer.py docs`. **R-LOAD-1** (§9).
5. Mutations as listed: plant, run, record red, revert from a byte copy (`cmp` clean), green again.
6. Review: answer the commit's review questions in §16.
7. `git status --short`, `git diff --stat`: every path in the commit's **Files**; no binary (I-p-11).
8. Update §16 and the handoff in the same commit. Stage by explicit path; never `git add -A`.
9. A material stop (§12) is reported, not improvised around.

#### C0 — Ledger, handoff, base, `DEP-72`, `DEP-74`

- **Files:** this file (§16), `.structured-coding/plans/mvp0/handoff-s32p.md` (new), `docs/DECISIONS.md`.
- [ ] Implementation:
  1. Record the base commit; verify #211, the pose-sheet commit `73c5bc2c` and `bd7bc92f` + template v2 are
     ancestors of `HEAD` (`git merge-base --is-ancestor`); any missing → material stop (§14).
  2. Re-read A-p-1…A-p-31 on the base; record moved lines (bounded) or changed behaviour (stop if a stage, gate or
     the Meshy client changed meaning).
  3. Write `DEP-72` (MPFB2/MakeHuman as a hidden tool: `hm08` template, `mixamo` rig, `.mhclo`, MakeClothes; GPL
     code never shipped; CC0 core only; stock assets as test fixtures only; the isolated-extension exception
     F-S32p-3) and `DEP-74` (step-32 §4.5's rejected list with reasons) at the end of `docs/DECISIONS.md`.
  4. Handoff with the contract's fields and the sequence C0…C8.
- [ ] Validation: `python3 scripts/ci_layer.py docs`.
- [ ] Review: `DEP-72` says "hidden fitting tool" and "never reaches a player" as the ruling does; no id outside
  step-32 §19.1.
- **Commit:** `docs(plan): S32-p — open the ledger; DEP-72 MPFB2 as a hidden tool, DEP-74 rejected bases`

#### C1 — Budget, ledger reconcile, image calls, recipe (no spend)

- **Files:** `mwcast/meshy.py`, `mwcast/cli.py` (the `wardrobe` dispatch only), `mwcast/wardrobe/{__init__,recipe,
  budget,images,cli}.py`, `recipes/s32p-pilot.yaml`, `wardrobe/prompts/{base_layer,garment}.txt`,
  `tests/test_meshy.py`, `tests/test_wardrobe_{recipe,budget,images}.py`, this file, handoff.
- [ ] Implementation:
  1. `meshy.py`: `run_live` gains an optional `charge: Callable[[Ledger, str, int | None, dict], Ledger]` hook
     called after the task ends; the default keeps #211's behaviour exactly (S26-b1 parity). `check_cap` gains a
     `projection` argument defaulting to `CREDITS_PER_RUN`.
  2. `budget.py`: the rules of §4.6 (ceilings, retry budgets, pool, IH gate, reconcile measured vs balance delta,
     reports at 600) as pure functions over the ledger and the recipe.
  3. `images.py`: assemble prompts (v2 preamble from `template.yaml`, version 2 required) and run `python -m
     mineworld_gen … --model gpt-image-2.5-sunburst` as a subprocess; check the sidecar's `model`; log to
     `images.json`; refuse when the OpenAI cap is reached.
  4. `recipe.py` and `recipes/s32p-pilot.yaml`: subjects, identity references (path + SHA-256; Ivan/Hana `pick:
     null` until ruled, which refuses their stages), garments and outfits (§4.5), budget (§4.6), OpenAI cap, the W-3
     literal (filled in C3).
  5. `cast.py wardrobe meshy body|garment`, `wardrobe image base-layer|garment` (both refuse on a missing pick or a
     missing approval record).
- [ ] Validation: the four test files; M-p-7 and M-p-9; #211's `test_meshy.py` unchanged results (parity).
- [ ] Review: can any path send a POST without passing `budget.admit`? Is the default `run_live` byte-for-byte
  #211's behaviour? Does any code path call `mineworld_gen` without `--model`?
- **Commit:** `feat(cast): S32-p — the pilot's credit budget with measured charges, pinned image calls, recipe`

#### C2 — MPFB2 environment, regions, registration (no spend)

- **Files:** `blender/mpfb/{LICENSE,install,regions,register}.py`, `wardrobe/template/hm08_regions.json`,
  `mwcast/wardrobe/cli.py` (`regions`, `register`), `tests/test_wardrobe_confined.py`, `PIPELINE.md` §16 (body
  half), this file, handoff.
- [ ] Implementation:
  1. `install.py`: install `mpfb-2.0.18.zip` (SHA-256 checked against the published value P-1 recorded) into
     `<scratch>/blender-user`; record the version.
  2. `regions.py` → `hm08_regions.json` (R-5) with its rule in the file header.
  3. `register.py` (R-4 steps 1–6), with every printed number named.
- [ ] Validation (Gate 2, zero credits): register onto **the fused Vera's kept download** re-derived by #211 (clothed,
  but it exercises the pose, LBS and surface fit) and onto the prototype's `womanA`/`manB` bodies; PASS = per-region
  mean distance ≤ 3 mm and 95th percentile ≤ 8 mm on the MPFB bodies (identity-topology case), the stature within
  5 mm, and a printed region table for Vera; `test_wardrobe_confined.py`.
- [ ] Review: does any `mwcast` module import `bpy` or `bl_ext`? Is the macro calibration bound from a cited stature
  (S26 §7.2), not from the candidate?
- **Commit:** `feat(cast): S32-p — hm08 regions and registration onto a body (MPFB2 as a hidden tool)`

#### C3 — Garment stages, format, fit at load (no spend; stock fixtures only)

- **Files:** `blender/mpfb/{clay,garment_clean,garment_bind,fixtures}.py`, `mwcast/wardrobe/garment_format.py`,
  `godot/wardrobe/fit_at_load.gd`, `tests/test_wardrobe_format.py`, `PIPELINE.md` §16 (garment half),
  `recipes/s32p-pilot.yaml` (the W-3 literal), this file, handoff.
- [ ] Implementation: R-6, R-8a, R-8b, R-9, R-10, R-11 and R-12 (b), §4.4's files. W-3's literal: Vera's fused body
  texel density read once from `meshy_d.glb` and written with its derivation. RSWT: read its licence and libigl's
  binding licence; record; install into the tool venv only if both pass `DEP-8`-compatible use as a tool.
- [ ] Validation (Gate 2, zero credits): run R-9…R-12 on two **CC0 stock fixtures** from the prototype (T-shirt,
  trousers) on `womanA`, `manB` and the five W-5 fixture bodies; PASS = W-1, W-9 (≤ 0.5 mm), W-10 on all; the unit
  file; M-p-3, M-p-8. The fixtures carry `role: fixture` (they never reach a sheet).
- [ ] Review: is the `.mhclo` formula the same in Python, Blender and GDScript (one hand-computed fixture shared)?
  Does any part get a `Skin` other than the body's?
- **Commit:** `feat(cast): S32-p — garment cleanup, binding to hm08, the runtime garment format, fit at load`

#### C4 — Gates, outfit assembly, pose sheet `--outfit`, V-1 (no spend)

- **Files:** `blender/mpfb/assemble.py`, `godot/wardrobe/outfit_gates.gd`, `godot/pose_sheet.gd`,
  `mwcast/pose_sheet.py`, `mwcast/wardrobe/{wgates,outfits}.py`, `tests/test_wardrobe_{gates,outfits}.py`,
  `PIPELINE.md` §17, this file, handoff.
- [ ] Implementation: R-12 (a), R-13, R-14; F-S32p-4, F-S32p-5, F-S32p-6 applied; the A/B/Vera pair frame.
- [ ] Validation (Gate 2, zero credits): a fixture outfit on `womanA` through every gate and a pose sheet; the fused
  Vera's pose sheet without `--outfit` re-rendered and compared with #211-era frames (byte-identical except P-4
  within A-p-19's tolerance: the outfit mode must not change the single-mesh path); M-p-1 (on the fixture tee),
  M-p-2, M-p-4, M-p-5, M-p-6, M-p-10. Expected: V-1 FAILS on the fixture outfit (stock) — that is the check working.
- [ ] Review: is every bound a literal with its source (I-p-6)? Does the torso check still flag
  `--plant-arm-into-torso` in outfit mode?
- **Commit:** `feat(cast): S32-p — outfit assembly, G/W gates per outfit, pose sheet --outfit, no-stock scan`

#### C5 — Vera, the parity control (spend: phase V ≤ 390 credits)

- **Files:** `recipes/s32p-pilot.yaml` (Vera's pick), `pilot/s32p/vera-lindgren/*`, `pilot/s32p/garments/vera-*/*`,
  `pilot/s32p/report.json`, this file, handoff. No binary.
- [ ] Implementation: R-1 for Vera → pre-spend page → (operator pick recorded) → R-2, R-3, R-4 → R-6…R-11 for her
  five garments and hair → outfit A → R-13, R-14 → scorecard beside the fused Vera; D-p-4's RSWT evaluation.
- [ ] Validation: every gate on outfit A; the ledger ≤ 390; the 50 % report if crossed.
- [ ] Review: **the Vera checkpoint.** Does modular Vera in her own clothes read at the fused Vera's tier, face and
  hair included, in every frame? If not after the budgets: stop, write the failure diagnosis and the levers to the
  page, and go to C7 (no Ivan/Hana spend; Q-S32p-4).
- **Commit:** `feat(cast): S32-p — Vera by route B in her own garments (records; assets in scratch)`

#### C6 — Ivan and Hana; every outfit B (spend: the rest of the 1,200)

- **Files:** as C5 for `ivan-petrov`, `hana-sato` and their garments.
- [ ] Implementation: picks recorded (§14); the IH phase gate (§4.6 rule 4); R-1 → pre-spend page → R-2…R-11 →
  outfits A for both; outfits B for all three; the W-6 sweep (9 outfits).
- [ ] Validation: every gate per outfit; the ledger ≤ 1,200; AC-p-8.
- [ ] Review: does any subject's base-layer body lose identity beside its reference (R-S32-4)? Is any garment
  reused under a name that hides a blocked one?
- **Commit:** `feat(cast): S32-p — Ivan and Hana by route B; outfits B for all three`

#### C7 — Measurements, review page, records

- **Files:** `pilot/s32p/report.json`, `docs/DECISIONS.md` (`ARC-138`, `ARC-140` format part, `ARC-141`,
  `DEP-73`), `docs/HUMAN_REVIEW_QUEUE.md` (`VIS-3D-WARDROBE-P`), `PIPELINE.md` (measured numbers), this file,
  handoff. The page itself is outside the repository.
- [ ] Implementation: §4.11's measurements; `wardrobe page` → `mineworld-demos/wardrobe-pilot/index.html`; the
  records with the verdict field `pending VIS-3D-WARDROBE-P` and the measured costs; the full-budget re-estimate.
- [ ] Validation: the page opens; every frame present (AC-p-1); V-1 PASS on the six deliverable outfits;
  `python3 scripts/ci_layer.py docs`.
- [ ] Review: does the page state the decision rule and the honest outcome (I-p-13)? Are the costs measured, not
  estimated?
- **Commit:** `docs(cast): S32-p — measurements, VIS-3D-WARDROBE-P, ARC-138/140/141, DEP-73`

#### C8 — Close-out

- **Files:** this file, handoff.
- [ ] Implementation: AC-p-12 (`git diff --name-only origin/main...HEAD` within I-p-9; no binary); push
  `mvp0/pr-s32p`; `gh pr create` (body ends with the attribution line); CI on the exact head; repair to green.
- [ ] Validation: CI green on Linux, macOS, Windows on the exact head.
- [ ] Review: deviations recorded; the PR body names the page and the credits.
- **Then:** READY FOR OPERATOR REVIEW — DO NOT MERGE. The operator's verdict on `VIS-3D-WARDROBE-P` sets the route
  field of `ARC-138` in a follow-up commit on this PR before any merge.

---

## 9. R-LOAD-1 and machine load

- Locally only the touched unit-test files, one at a time, plus ruff, ruff format, pyright and the docs layer. Never
  `cargo test`, never a whole-workspace or whole-member Python run; CI on the exact head is the gate.
- Every Blender and Godot job through `with-slot.sh` (default 2 slots machine-wide). One pose sheet per slot; the
  W-sweeps split per subject; the perf run alone, with `uptime` recorded before and after.
- Meshy polling (15 s) and OpenAI calls hold no slot. At most two Meshy tasks in flight.
- The pose sheet needs a drawable window (A-p-19): it runs `--always-on-top`; a stalled run is INCONCLUSIVE, not
  FAIL, and is re-run once.

---

## 10. Requirements this PR places on other PRs

- **R-S32c-p-1 (S32-c).** The client's fit at load reproduces `fit_at_load.gd`'s results within W-9's 0.5 mm, with
  the format of §4.4; parts named `part_<id>` and materials `MW_Cast_<id>` or a client binding that replaces the
  name-based hiding (F-S32p-4).
- **R-S32d-p-1 (S32-d).** Landmark automation (MediaPipe vs geometry) is evaluated at S32-d's start on the pilot's
  three bodies before the 18 others (Q-S32p-7); the measured retention and costs of §4.11 replace step-32's
  estimates in its budget question.
- **R-S32h-p-1 (S32-h).** X-S32-20 adopts V-1's provenance roles (`fixture`, `fitting-template, drawn: false`).
- **R-S26cast-p-1 (S26-cast).** The base-layer image becomes a per-person casting output only if route B is adopted
  (step-32 §15.1).

---

## 11. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-32-wardrobe.md` §6.6 g2 | the garment mannequin is the wearer's own Meshy body in grey clay (F-S32p-10, D-p-1) |
| `step-32-wardrobe.md` §6.6, §7.2 | the tight-fit class for skin-tight garments (F-S32p-7), as ruled in Q-S32p-5 |
| `step-32-wardrobe.md` §16 S32-p row | the library is 15 garments + 3 hair styles, sized by the outfits (§4.5) |
| `step-32-wardrobe.md` §15.3 | S26-b2's spike: Meshy rig not evaluated; landmarks automated in S32-d (F-S32p-9, Q-S32p-7) |
| `overall.md` | S32-p frozen / ready / verdict |
| `mineworld-demos/progress/index.html` | the pilot's lane and verdict (primary, after each ruling) |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S32-p — the wardrobe pilot (route B) on Vera, Ivan and Hana
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s32p-wardrobe-pilot.md
RELATED / BINDING DOCS:  step-32-wardrobe.md (§§0, 4.2, 5–7, 14–19, 20.1); step-26-cast.md (header amendment
                         2026-10-15, §§5.2, 5.5, 6.1–6.5, 10); tools/character_pipeline/PIPELINE.md (#211 and
                         the pose-sheet commit); docs/HUMANOID_PROFILE.md; docs/VISUAL_FIDELITY.md §§5, 8, 9;
                         docs/DECISIONS.md ARC-9, ARC-19, ARC-23, ARC-94, DEP-8, DEP-51…55;
                         .structured-coding/standards.md; CLAUDE.md; docs/ENGINEERING_STANDARDS.md;
                         docs/ENGINEERING_RULES.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s32p, its own, held by one session
BRANCH:                  mvp0/pr-s32p, created from origin/main
START CONDITION:         §14: this design frozen; #211, the pose-sheet commit and the v2 casting tooling
                         (with bd7bc92f) on main; the primary session's dispatch. Spend per subject only after
                         that subject's base-layer pick is recorded; Ivan/Hana also need their casting picks
IMPLEMENTATION BASE:     origin/main at C0; C0 records it and the ancestry checks
APPROVED SCOPE:          §2.1 items 1–8
FROZEN INVARIANTS:       §5 I-p-1 … I-p-13; step-32 §5.2's decision rule; step-26 §5.5; the 52-joint profile
SEQUENCE:                C0 … C8 (§8)
ALLOWED COMMANDS:        git (never merge); gh (never merge); uv run --locked {ruff, pyright, pytest <one file>};
                         python3 scripts/ci_layer.py docs; python3 tools/character_pipeline/cast.py *;
                         with-slot.sh <blender|godot|python3 cast.py pose-sheet|wardrobe sweep>;
                         source ~/.config/mineworld/secrets.env && python3 tools/character_pipeline/cast.py
                         wardrobe {meshy,image} …; mkdir -p; sed -n; grep; shasum; uptime;
                         WebFetch for licence reads (RSWT, libigl) only
NEVER:                   reading, printing or copying a key or secrets.env; a Meshy call outside `cast.py wardrobe
                         meshy`; an OpenAI call without --model gpt-image-2.5-sunburst; any Meshy endpoint but
                         image-to-3D and balance; raising the cap; cargo test; a whole-workspace or whole-member
                         test run (R-LOAD-1); committing a GLB, PNG, BIN or .blend; edits outside I-p-9's paths;
                         writing the fused Vera or any client asset; git add -A; python3 -c; sed -i; heredoc
                         writes; curl
MATERIAL STOPS:          a missing tooling ancestor (C0); a needed change to #211's stage semantics or gate
                         bounds; the ledger would pass 1,200 or a phase ceiling; Vera fails the checkpoint after
                         her budgets (→ C7 with the failure page, no Ivan/Hana spend); a subject's pick missing
                         when its spend is due; a licence that fails DEP-8 for a tool or asset; any MPFB2 stock
                         asset needed in a drawn part; a step-32 frozen invariant contradicted
PLATFORMS:               unit tests and static checks on Linux, macOS, Windows (CI python layer); the live pilot
                         on macOS (the operator's M5), like #211's Gate
VALIDATION BUDGET:       unit files unrestricted, one at a time; Gate 2 runs bounded per §16's specifications
                         (Blender stage ≤ 15 min, a pose sheet ≤ 10 min, a W-sweep per subject ≤ 30 min);
                         total wall time for the live pilot ≈ 2 working days, much of it Meshy wait.
                         MONEY: Meshy ≤ 1,200 credits (ruled, inside S26's 3,000); OpenAI ≤ the cap ruled in
                         Q-S32p-6 (ruled: ≤ 70 calls ≈ $3.9). Gate 1: NOT REQUIRED
LIVE DOCUMENTATION:      this document (§16)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s32p.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                                       (operator, 2026-10-15: "同意冻结并实施"; starts on the primary's dispatch)
                         spend: Meshy ≤ 1,200 credits   (operator ruling Q-S32-1, 2026-10-15)
                                OpenAI ≤ 70 calls       (operator ruling Q-S32p-6, 2026-10-15)
                         semantic commits: authorized   (working rules §14)
                         branch push: authorized        (working rules §21)
                         PR creation / update: authorized (working rules §21)
                         CI repair to review readiness: authorized (working rules §21)
                         endpoint: the PR READY FOR OPERATOR REVIEW plus the review page
                                   VIS-3D-WARDROBE-P delivered (the coordinator's brief, 2026-10-15)
                         merge: NOT authorized — only with the operator's explicit authorization after the
                         primary's review (working rules §22). No spend beyond the cap under any reading
POST-MERGE SYNC OWNER:   primary: step-32, step-26, overall.md, the progress page; implementation session: this
                         ledger, the merge identity, the evidence, the deviations
STOP CONDITION:          S32-p READY FOR OPERATOR REVIEW with VIS-3D-WARDROBE-P delivered — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-p-1 | **Modular Vera does not reach the fused Vera's look** (face from a new generation; hair as a separate shell) — R-S32-1, R-S32-4 | Vera first, with a hard checkpoint (C5); the levers of R-S32-1 costed on the failure page; never shipped close |
| R-p-2 | The skin cap confuses Meshy (a helmet, a seam, a lost hairline) | raw gate on the head; the base-layer prompt names a smooth skin-tone cap; a retry with "hair in a tight low bun under a thin skin-tone cap" as the one documented parameter change (step-26 §6.5) |
| R-p-3 | OpenAI refuses the base-layer image (minimal clothing) | athletic tank top and mid-thigh shorts, not underwear; a refusal is logged, the replacement uses a fitted short-sleeve tee |
| R-p-4 | Subtraction leaves clay fused to a garment, or eats the garment (low retention) — R-S32-3 | 1.5 mm inside test + connected-component rule; the retry budget; retention measured |
| R-p-5 | Registration error shows as gaps or poke-through — R-S32-2 | joint-driven pre-pose before the surface fit; W-4/W-5/W-7 by location |
| R-p-6 | Cross-sex/cross-build fitting fails (Ivan in Vera's jeans; Hana in Ivan's jacket) | that is what outfit B is for; a blocked B-garment is replaced by another of its slot (recorded), the failure is evidence for S32-d |
| R-p-7 | Ivan's candidate style is more photographic than Vera's (A-p-24) and the base-layer image inherits it | the scorecard's quality-tier row beside Vera; a redo of the base-layer image (OpenAI cost only) before any Meshy spend |
| R-p-8 | MPFB2 headless API differs from the prototype's use | C2 runs on the pinned 2.0.18 exactly as P-1…P-3 did; a needed MPFB2 change is a stop |
| R-p-9 | The pose sheet stalls (window not drawable) | `--always-on-top`; INCONCLUSIVE and one re-run (A-p-19) |
| R-p-10 | The casting picks change after a base-layer image or a Meshy body exists | the recipe records pick + SHA-256; a changed pick invalidates that subject's downstream records; regenerating the base-layer image costs OpenAI only, a new body costs from the retry pool or stops |
| R-p-11 | The tooling branches land late or diverge (§2.3) | C0's ancestry check; Q-S32p-1 |

---

## 14. Start condition (honest state on `origin/main @ 5d901e85`)

**The pilot cannot start today.** On `5d901e85`: #211 is OPEN; the pose-sheet tool and the v2 casting tooling (with
`bd7bc92f`) are on branches with no PR; Ivan's and Hana's casting picks are pending.

**Does the pilot wait for the casting picks of Ivan and Hana? Recommendation: partly.**

- It does **not** wait for the casting of the other 18 people, and it does **not** wait for Ivan's and Hana's picks
  to start: C0…C4 spend nothing, and C5 (Vera) needs no pick, because her identity is fixed (the fused Vera and
  `anchor_fullbody.png`).
- Ivan's and Hana's **base-layer images (OpenAI only)** wait for their picks, and their **Meshy spend** waits for
  their base-layer approval on the pre-spend page (step-32 §16; step-26 §6.4 gate 1: a pre-spend decision, asked as
  soon as it is ready). With the Vera-first order and the stop on a Vera failure, those picks are needed only at C6,
  roughly a day of work after the start.
- If the picks are not ruled when C6 is due, the session stops there (a material stop), reports, and holds; it does
  not substitute the agent's ranking for the operator's pick.

**Start condition, as recommended:** this design frozen; #211, the pose-sheet commit and the v2 casting tooling
(with `bd7bc92f`) on `main` (Q-S32p-1); the primary session's dispatch. Per-subject spend gates as above.

---

## 15. Operator questions (each with a recommendation; questions the rulings settled are not re-asked)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S32p-1 [primary — the tooling base]** | The pilot builds on #211, the pose-sheet commit `73c5bc2c` and the v2 casting tooling on `mvp0/s26-cast-director @ d81baeb3` (template v2, the anchors, the briefs, `bd7bc92f`). Only #211 has a PR. Land the other two by their own PRs before S32-p starts, or let S32-p's branch merge them in C0 (a stacked PR whose diff includes them)? | **Their own PRs first.** The pilot's review diff then shows only the pilot; the cast director and the pose sheet get reviewed as what they are. If that would delay the start by more than a day, the stacked fallback is acceptable, with the base recorded in C0 |
| **Q-S32p-2 [operator — when the pilot starts]** | Start Vera's track as soon as the design is frozen and the tooling is on `main`, with Ivan's and Hana's base-layer images following their casting picks (§14)? | **Yes.** It costs no extra spend, and the Vera checkpoint decides whether Ivan's and Hana's credits are spent at all |
| **Q-S32p-3 [operator — garment references]** | Approve each of the 17 garment and hair reference images on a pre-spend page before its Meshy run, or let the agent's raw gate (against the identity reference) decide, with every reference shown on the final page? | **The agent's raw gate.** The base-layer image is the identity decision and stays operator-gated; a garment reference is a copy of a garment the operator already approved in the identity image. Seventeen extra round trips would stall the pilot for ≈ 30 credits each |
| **Q-S32p-4 [operator — the Vera checkpoint]** | When modular Vera passes every gate and the agent's scorecard beside the fused Vera, continue straight to Ivan and Hana (your look verdict comes on the final page), or stop for your look verdict on Vera first? | **Continue; stop only on a Vera failure.** You see Vera with the others on one page, and a failing Vera stops all further spend automatically |
| **Q-S32p-5 [operator — skin-tight garments]** | Tights (and a close fade's sides) cannot be cut out of a Meshy mannequin (F-S32p-7). Make them a **tight-fit class**: a 0.8 mm shell offset from the wearer's own Meshy body, textured from the v2 garment reference image — no Meshy run, no MPFB2 asset? | **Yes.** Every pixel still comes from the v2 image model and every surface from the person's Meshy body; it saves ≈ 30–90 credits per skin-tight garment, and a Meshy run would be cut away by the subtraction anyway. Alternative: one Meshy run of tights on bare clay legs, expected to fail R-8a |
| **Q-S32p-6 [operator — OpenAI cap]** | The pilot needs ≤ 70 image calls with `gpt-image-2.5-sunburst` (≈ $3.9 at the measured $0.0563 per 1024×1536 high call; §4.7). Approve that cap? | **Approve ≤ 70 calls.** It is the base-layer images (≤ 15) and the garment and hair references (≤ 54); a cap reached stops the image stage and is reported |
| **Q-S32p-7 [primary — S26-b2's spike]** | Step-32 §15.3 folds S26-b2's landmark, weights and Meshy-rig comparisons into S32-p; §4.2 says the Meshy rig is "not used". Here: no Meshy-rig evaluation (no credits), RSWT evaluated on Vera (D-p-4), landmarks hand-supplied on the ortho render as #211 does, automation evaluated at S32-d's start (R-S32d-p-1). Accept? | **Yes.** Three bodies do not need automated landmarks; eighteen do, and S32-d is where that pays. No credit goes to a rig the step already declined |


### 15.1 Rulings, 2026-10-15 (relayed by the coordinator; binding)

| Question | Ruling | By |
| --- | --- | --- |
| Q-S32p-1 | **Own PRs first** for the pose-sheet and cast-director branches; the primary opens them. Applied: §2.3, §14, C0 step 1 | Ruled 2026-10-15 (primary) |
| Q-S32p-2 | **Yes:** Vera starts once the design is frozen and the tooling is on `main`; Ivan and Hana follow their casting picks. Applied: §14, C5, C6 | Ruled 2026-10-15 (operator answered directly in the primary session) |
| Q-S32p-3 | **The agent's raw gate** decides garment references; every reference is shown on the final page. Applied: R-7, §4.9 | Ruled 2026-10-15 (operator answered directly in the primary session) |
| Q-S32p-4 | **Continue after Vera; stop only on a Vera failure.** Applied: C5 review, §12 material stops | Ruled 2026-10-15 (operator answered directly in the primary session) |
| Q-S32p-5 | **The tight-fit class is approved.** Applied: R-8b, §4.5, step-32 §6.6 | Ruled 2026-10-15 (operator answered directly in the primary session) |
| Q-S32p-6 | **≤ 70 image calls approved.** Applied: §4.7, §12 | Ruled 2026-10-15 (operator answered directly in the primary session) |
| Q-S32p-7 | **Yes:** no Meshy-rig evaluation; RSWT evaluated on Vera; landmarks hand-supplied; automation at S32-d's start. Applied: F-S32p-9, R-3, R-S32d-p-1, step-32 §15.3 | Ruled 2026-10-15 (primary) |

Freeze: "同意冻结并实施" (operator, 2026-10-15, in the primary session). No question remains open. Parent edits of §11
to `step-32-wardrobe.md` and `overall.md` applied with the freeze.

---

## 16. Ledger (live during implementation)

### Execution record (implementation session, opened 2026-10-18)

- **Worktree / branch:** `/Users/yuema137/mineworld-worktrees/impl-s32p`, branch `mvp0/pr-s32p` (§12), held by this
  session only (no other session on it: the worktree did not exist before; `git worktree add` created it).
- **Implementation base (C0 step 1):** `origin/main @ b0a9d7382825489335cb5e87504f31199534e9ee` (#233,
  `mvp0/s26-cast-director`, merged). Ancestry, each `git merge-base --is-ancestor <c> HEAD` → 0:
  #211 head `121662c9` ✓; pose-sheet `73c5bc2c` ✓ (#232); provenance fix `bd7bc92f` ✓; cast director `d81baeb3` ✓
  (template v2: `template.yaml` `version: 2`, SHA-256 `9894cbcd…a8aa8`; `anchor.png` `5b0c8f0b…a3a1`;
  `anchor_fullbody.png` `c4b936fe…3ef32`, both equal to A-p-20). Start condition §14 holds.
- **Casting picks (§2.3, §14):** ruled. `presentation/mineworld-default/3D/characters/cast/PICKS.yaml` (operator,
  2026-10-15, "人物可以全部按推荐，我已经看过了"): `ivan-petrov: 2`, `hana-sato: 2`. Identity references
  `cast-out/ivan-petrov/candidate_2.png` SHA-256 `f54d8a71…bfd771`, `cast-out/hana-sato/candidate_2.png`
  `beadbea4…b139` (outside the repository; recorded in the recipe, C1).
- **A-p-1…A-p-31 re-read on the base (C0 step 2):** unchanged in meaning. Spot-checked lines: `meshy.py:34,
  118-125, 254-262, 271-272, 296-297` (A-p-7) exact; `pipeline.py:27-52` stage list (A-p-1) exact;
  `human.gd:326-332` name hiding (A-p-17) exact; `pose_sheet.gd:176-197` `MW_Cast` match (A-p-16) exact. No stage,
  gate or Meshy-client meaning changed.
- **Endpoint authority, current session (working rules, authority order):**
  1. The coordinator's dispatch of 2026-10-18 (relaying the operator): "Scope now: code only. The operator has
     paused heavy local jobs" — implement the commits that need no Blender, Godot or Meshy spend; prepare the live
     pilot's commands "marked waits for the operator to lift the PAUSE".
  2. **Superseded the same day** by the coordinator's message relaying the operator: "你现在可以跑本机重任务了"
     (the PAUSE file removed). Blender and Godot may run, always through `with-slot.sh`; the live pilot runs per
     §4.6, Vera first, inside the 1,200-credit hard cap; "stop before spending past the cap. If the measured
     per-character cost implies the pilot can't finish within 1,200, stop and report the numbers". This replaces
     line 1; §12's endpoint authority is otherwise unchanged (merge not authorized).

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 ledger, handoff, base, `DEP-72`, `DEP-74` | (this commit) | [x] base and ancestry recorded above; `DEP-72`, `DEP-74` appended to `docs/DECISIONS.md`; handoff `handoff-s32p.md` | [x] `python3 scripts/check_decision_ids.py` → "168 decision ids, all distinct"; `python3 scripts/ci_layer.py docs` (below) | [x] `DEP-72` says "hidden fitting tool" and "never drawn for a player … or shown to the operator as the look" (the ruling's words); ids used: `DEP-72`, `DEP-74` only, both in step-32 §19.1's S32-p list |
| C1 … C8 | — | [ ] | [ ] | [ ] |

### Gate specifications and evidence

(none yet; each live run's claim, command, budget and expected evidence is written here before it runs)

### Credits and image calls

| Phase | Runs | Measured credits | Balance delta | Kept |
| --- | --- | --- | --- | --- |
| — | — | — | — | — |

### Mutations

(none yet; M-p-1 … M-p-10 as §6 lists them)

### Deviations and findings

- F-S32p-1 … F-S32p-11 (§3.6), recorded at design time.
- **Bounded, at design time:** the pilot library is 15 garments + 3 hair styles (step-32 §16 said 12 + 4, an
  estimate); the fourth hair style serves no outfit (§4.5).
