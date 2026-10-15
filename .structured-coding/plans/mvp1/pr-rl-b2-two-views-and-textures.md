# PR RL-b2 — the two remaining views within 16.7 ms, and 512 px textures for small and distant props

**`DESIGN FROZEN 2026-10-14 (primary session; operator Q2, Q3, Q5)`**

**Lifecycle:** FROZEN. Scope (§1), the measurement rules (§3), the trial set and the decision rule (§4.2,
§4.3), the classification and floors (§5), acceptance and mutations (§6), the visual guards (§7) and the
execution contract (§13) are frozen. Progress, evidence, audit findings and bounded corrections stay
writable (§11 checkboxes, §14 ledger). Rulings in §0.1. **Implementation starts only on the primary
session's dispatch**, in a fresh session (a Haiku implementer), following §11 step by step. Superseded
lifecycle: `DRAFT — PR design, awaiting operator freeze review`.
**Step:** S22 Realism, [`step-22-realism.md`](step-22-realism.md) §9.2 row RL-b (follow-up). Parent PR:
[`pr-rl-b-3d-budget.md`](pr-rl-b-3d-budget.md) (RL-b, MERGED 2026-10-14 as `171ea06`).
**Runs as:** an MVP-0 visual-track PR, like RL-b. It changes only presentation (`clients/3d-spike/**`, the
`mineworld-slice` launcher, docs).
**Author:** design session, 2026-10-14, worktree `/Users/yuema137/mineworld-worktrees/design-rl-b2`,
branch `docs/rl-b2-design`. Audit base: `origin/main @ 171ea06` (RL-b's merge).
**Placeholder ids:** SD-B2-n (decisions), T-n (perf trials), A-n (acceptance), X-n (mutations), V-n (visual
guards, RL-b's numbering kept), Q-B2-n (questions), E-B2-n (evidence), R-B2-n (risks).

**Why one PR for two parts.** Both parts are small, both change the 3D slice's look in bounded ways the
operator judges from stills, and both need one operator measurement session on the same head (the texture
pass changes texture bandwidth, so the perf verdict must be taken on a tree that includes it). One PR means
one worktree, one final measurement and one review.

---

## 0. Rulings and facts this PR carries

| Source | Content | Effect here |
| --- | --- | --- |
| RL-b E-RLb-operator, 2026-10-14 (operator-run) | Apple M5, Godot 4.7.2, Metal, 1920×1080, gi=voxel, three conclusive runs (one discarded: a 4.4 s frame). Median p95 ms: street wide 10.85, **cafe frontage 17.11**, interior 13.61, **doorway 18.59**, street east 11.12, south side 10.18, florist interior 14.63, skyline east 10.69. Video memory 1 608.1 MB (pass). Draws 614–4 215, primitives up to 3.67 M (advisory, R-2). CPU p95 ≤ 3 ms in every view (GPU-bound; Metal reports no GPU timestamps) | The gap to close: cafe frontage −0.41 ms, doorway −1.89 ms, plus a margin (SD-B2-4). The frame is GPU-bound, so levers that only cut CPU-side draw submission are expected to yield little |
| Operator ruling at RL-b's merge, 2026-10-14 | "merge now, open a small PR to optimise the two remaining views; **the 16.7 ms target stays**" | A-1 is RL-b's M-6 as amended by R-2, unchanged |
| RL-b R-2 (2026-10-09) | p95 ≤ 16.7 ms and video memory ≤ 2 048 MB binding; draw calls ≤ 2 000 and primitives ≤ 3 M advisory | Same here. The instrument's printed verdict is brought in line with R-2 (SD-B2-1) |
| RL-b R-3 | V-gates re-baselined on RL-b's final look | V-1's base is `main @ 171ea06`'s frames |
| **MS-Ra-2 ruling (operator, 2026-10-10)** | "512 px textures for small and distant props; near walls, characters and large surfaces keep 1k; before/after stills go to the operator. 3D-lane work on top of RL-b, a separate small PR after RL-b merges, not part of R-a" (step-23 §18.7 on `mvp0/pr-s23-ra @ 09c28a1`) | Part (b). R-a's §7.1 limits stand unchanged: `3d.pck` hard limit 250 MB compressed (goal 150 MB) |
| R-a §18.7 measurement (2026-10-10, RL-b's textures) | `3d.pck` raw 354 128 840 B, **`gzip -6` 259 509 293 B**; every one of the 213 slice textures is 1 398 180 B as BC7 with mipmaps; BC7 hardly compresses further | Each 1k → 512 px texture saves about 1 048 635 B (¾ of 1 398 180) in the pack and in video memory |
| step-22 §6.4 | Hand-held goods need **≈ 1 024–2 048 px/m** texel density at a shopper's distance | A floor for downsizing hand-held goods (SD-B2-6) |
| DEP-8 coherence procedure item 4 (`docs/DECISIONS.md`) | Texel density target ≈ 512 px/m for a walkable street | The floor for every other downsized prop (SD-B2-6) |

## 0.1 Freeze rulings (2026-10-14; binding)

| Question | Ruled by | Ruling |
| --- | --- | --- |
| Q-B2-1 | primary | **Accepted as recommended:** one trial session for every candidate lever (§8.1), then one final session (§8.2) |
| Q-B2-2 | **operator** | **If no low-risk lever closes the doorway gap, STOP and show** the measured numbers and the side-by-side **before** taking two cascades (T-1). T-1 is still measured in session 1; it is never committed without a fresh operator ruling (§4.3 step 4, §9) |
| Q-B2-3 | **operator** | **Accepted:** 21 models / 79 textures to 512 px (§5.1), under the density floors (SD-B2-6); a slug under its floor stays 1k |
| Q-B2-4 | primary | **Accepted as recommended:** the four promenade-only models stay at 1k |
| Q-B2-5 | **operator** | **Accepted:** ≈ 177 MB (estimate) is the target result; ≤ 250 MB gzipped is the gate (A-3); the 150 MB goal is not pursued here |
| Q-B2-6 | primary | **Accepted as recommended:** floors 1 024 px/m (hand-held goods) and 512 px/m (other downsized props) |
| Q-B2-7 | primary | **Accepted as recommended:** `cells48` stays in the trial set |

Consequence for §4.3: "eligible" in step 1 means passing V-1 outright, or a located change the operator has
accepted. T-1 (`shadow2`) and any trial containing it are never chosen autonomously; a trial whose V-1 change
is located but not yet accepted is not chosen autonomously either — that is the step-4 stop.

## 1. Identity, base, scope

```text
PR            RL-b2 — cafe frontage and doorway within 16.7 ms; 512 px textures for small and distant props
branch        mvp1/pr-rl-b2 from origin/main at the start of execution
worktree      /Users/yuema137/mineworld-worktrees/impl-rl-b2, held by the implementing session only
depends on    RL-b (merged, 171ea06). Nothing unmerged
merge         a merge commit, never a squash; operator authorization only
```

**Goal.** On the operator's Mac (Apple M5, macOS, Godot 4.7.2, Metal), at gi=voxel and 1920×1080, every one
of RL-b's eight perf views has a median-of-three p95 ≤ 16.7 ms and video memory stays ≤ 2 048 MB, without
an unexplained or unaccepted change to the accepted frames (V-1 … V-4). And the S23 3D pack, exported as R-a
exports it, is ≤ 250 MB with `gzip -6`, by 512 px textures on small and distant props only.

**Scope.**

| Id | Item |
| --- | --- |
| SC-1 | **Trials, measured in one operator session.** A `--trial=<names>` option applies a candidate lever at load, in any mode (`--perf`, `--shots`, play), without a code commit per candidate; `--trials=<list>` runs the full M-5 protocol once per trial and prints one comparison table. The breakdown (M-7) gains the `doorway` view and an `omni shadows` category |
| SC-2 | **The instrument's verdict follows R-2.** One judge (`tools/perf_compare.py`) prints PASS/FAIL per view for p95 and for video memory, and reports draw calls and primitives as advisory |
| SC-3 | **The lever(s) chosen by SD-B2-4 from the trial table**, committed as plain code/settings (no trial switch in the shipped path) |
| SC-4 | **512 px textures for small and distant props** (MS-Ra-2): `tools/slice_imports.py` gains a reviewed per-slug size table and owns `process/size_limit`; the affected `.import` files are rewritten and committed |
| SC-5 | **A texel-density check** (`--texel-density`) that measures, in the built slice, each downsized prop's texel density at 512 px and fails if it is under its floor (SD-B2-6) |
| SC-6 | **Before/after stills** for the operator: the 27 `--shots` views, plus two new goods detail crops and one new close goods pose (SD-B2-7) |
| SC-7 | **The S23 pack measured**, base and head, by R-a's method in a scratch copy (SD-B2-8) |

**Non-goals.**
- Any lever RL-b tried and rejected, without new evidence (§4.1): small interior objects casting no sun
  shadow (C6, zero yield), SSIL quality 2 → 1 (C7c, V-1 FAIL on 15 views), VoxelGI → SSIL in Default (O-2,
  ruled a stop by Q-RLb-4), a further VoxelGI reduction below 128.
- Per-view occluders tuned to the perf poses (§4.1, SD-B2-3).
- Environment textures (walls, paving, roofs, floors), the character, the HDRI: they keep 1k (MS-Ra-2).
- Textures of models the slice does not use (promenade only): Q-B2-4.
- The committed export preset, `package.py`, any bundle work: R-a owns them.
- Windows and Linux measurement (QRL-6). Quality tiers (RL-i). The 2D client. Any server, kernel, system,
  world or contract change.

**Change set** (every path this PR may touch):

```text
clients/3d-spike/scripts/slice/slice_trial.gd          new: the trial levers (SC-1)
clients/3d-spike/scripts/slice/slice_main.gd           one call to SliceTrial.apply after the build and
                                                       GI setup; the chosen lever's constants (SC-3)
clients/3d-spike/scripts/slice/slice_perf.gd           BREAKDOWN_VIEWS += "doorway"; "omni_shadows"
                                                       category; the trial names in the run record; the
                                                       M-6 line delegates to perf_compare (SC-1, SC-2)
clients/3d-spike/scripts/slice/batch.gd                `merge(root, cell := CELL)` parameter (T-4), and
                                                       CELL itself only if T-4 is chosen (SC-3)
clients/3d-spike/scripts/slice/slice_world.gd          passes the trial cell size to SliceBatch.merge
clients/3d-spike/scripts/slice/texel_density.gd        new: the texel-density measure (SC-5)
clients/3d-spike/scripts/props.gd                      one line in gltf(): an `mw_slug` meta beside
                                                       `mw_category`, so SC-5 knows each prop's slug
clients/3d-spike/scripts/slice/slice_probe.gd          dispatch only: `--slice-texel-density`; two crop
                                                       entries and one view in the shots table (SC-6)
clients/3d-spike/tools/perf_compare.py                 new: the R-2 judge and the trial table (SC-2)
clients/3d-spike/tools/slice_imports.py                the size table; owns process/size_limit (SC-4)
clients/3d-spike/assets/models/<small slugs>/textures/*.import   tool-written (SC-4)
clients/3d-spike/project.godot                         only if the chosen lever is a project setting
mineworld-slice                                        --trials loop; --texel-density; calls perf_compare
clients/3d-spike/README.md                             the new flags, one paragraph
docs/HUMAN_REVIEW_QUEUE.md                             a dated VIS-3D-GODOT-2 line: the stills, checklist
.structured-coding/plans/mvp1/pr-rl-b2-two-views-and-textures.md   this file's ledger
```

**Paths with no diff:** `kernel/`, `contracts/`, `persistence/`, `server/`, `systems/**`, `cognition/`,
`worlds/**`, `tools/cli/`, `clients/protocol/**`, `clients/2d/**`, `clients/shared/**`; the character
(`human.gd`, `npc.gd`, `character_slot.gd`, `camera_rig.gd`, `posture.gd`, `player.gd`,
`assets/characters/**`, `tools/patch_imports.py`); `assets/textures/**/*.import` and `assets/hdri/**`; the
build files frozen PRs own (`streetscape.gd`, `cafe.gd`, `shop_interior.gd`, `cafe_interior.gd`, `build.gd`,
`terrace.gd`, `dressing.gd`); `occluders.gd` and `budget.gd` (unless §4.1's occluder evidence changes, which
is a material deviation).

## 2. Audit anchors (`origin/main @ 171ea06`, 2026-10-14)

| Id | Finding | Evidence | Consequence |
| --- | --- | --- | --- |
| B-1 | The sun: `SHADOW_PARALLEL_4_SPLITS`, max distance 95 m, splits 0.055 / 0.16 / 0.42 (absolute 5.2 / 15.2 / 39.9 m), blend splits on, atlas 2 048, soft-shadow filter quality 2 (R-1) | `slice_main.gd:262–281`; `project.godot` | T-1, T-2. Split fractions are relative to max distance, so shortening the distance without rescaling them would change every near cascade |
| B-2 | Sun shadows cost −1.49 to −2.51 ms and −2 183 to −2 871 draws when switched off (RL-b's final breakdown, on C5 with VoxelGI 256 and filter 3). Not measured at the doorway: M-7 covers street wide, cafe frontage and interior only | `pr-rl-b-3d-budget.md` E-RLb-final | The breakdown is extended to the doorway before any lever is chosen (`ARC-23`: locate before counting) |
| B-3 | **Deltas are not additive.** VoxelGI 128 alone gave doorway 18.59 ms (C7a on C5); filter 3 → 2 alone gave −2.06 ms there (C7b on C5); both together measure 18.59 ms (operator, 2026-10-14) | E-RLb-8, E-RLb-operator | Trials measure combinations (T-5), and the final verdict is a fresh measurement of the committed head, never a sum |
| B-4 | Five shadow-casting omni lights: two café pendants (`hanging_industrial_lamp`, `cafe_interior.gd:584, 587`, `p[6] = true`) and three florist pendants (`shop_interior.gd:337–342`). Never measured | grep | The breakdown gains `omni_shadows` (diagnostic). Switching them off is a visible change and is not a candidate without that evidence (§4.1) |
| B-5 | Occlusion culling is on, with 17 massing boxes; occluders cull colour and depth passes, not shadow passes. Occlusion off adds 98–459 objects in the three breakdown views | `occluders.gd`; E-RLb-6 | The doorway looks north into the café, whose walls already occlude. T-row `occlusion` at the doorway tells whether anything is left to cull (SD-B2-3) |
| B-6 | `SliceBatch` merges by (material instance, shadow setting, 24 m cell); materials are cached by parameters (`Mats._mat_cache`), so the cell count is materials × cells: 592 cells, about 2 100 draws | `batch.gd`; `mats.gd:16–130`; E-RLb-2 | T-4 (48 m cells) cuts draws with no pixel change by construction, but on a GPU-bound frame the yield is expected small, and coarser cells cull worse |
| B-7 | Mesh LOD: imported props carry importer LODs since RL-b C2; the viewport's LOD threshold is Godot's default (1 px; `rendering/mesh_lod/lod_change/threshold_pixels` not set) | `project.godot`; `props.gd` | T-3: 2 px threshold, set at run time through `Viewport.mesh_lod_threshold` |
| B-8 | The launcher passes unknown arguments through to the probe (`*) args+=("$a")`), and `--perf` already loops three runs with re-runs | `mineworld-slice:92–105, 157–181` | `--trial=` needs no launcher case; `--trials=` needs a loop around the existing one |
| B-9 | `SlicePerf._aggregate` prints an M-6 line judging draws and primitives as pass conditions; after R-2 that line prints FAIL for a run that passes R-2 | `slice_perf.gd` `_aggregate` | SC-2: the operator must read one honest verdict |
| B-10 | 213 slice texture `.import` files, all 1k, owned by `slice_imports.py` (four keys). `process/size_limit=0` is present in each (Godot's default), not owned by the tool. 147 are prop textures under `assets/models/*/textures/`, 66 environment textures under `assets/textures/` | `slice_imports.py`; `find … -name '*.import'` | SC-4 adds `process/size_limit` to the owned keys: 512 for the table's slugs, 0 elsewhere |
| B-11 | Props by use in the slice (grep of the slug strings in `scripts/slice/*.gd`): 36 slugs used; `celandine_01`, `grass_medium_01`, `street_lamp_02`, `wooden_bookshelf_worn` are not used by the slice (promenade spike only) | grep | §5's table classifies the 36; the four are Q-B2-4 |
| B-12 | First person is the default camera; vertical FOV 70° (`CameraRig.FOV`) | `camera_rig.gd:27, 54` | Screen pixels per metre at distance d and 1 080 lines: 540 / (tan 35° · d) ≈ 771 / d |
| B-13 | No export preset exists on `main` for the 3D client (R-a's C4 3D half is waiting on this PR); R-a measured in a scratch copy with the preset of step-23 §18.3/§18.7 | `ls clients/3d-spike`; step-23 §18.3, §18.7 on `mvp0/pr-s23-ra` | SD-B2-8 measures the same way, on base and head, without committing a preset |
| B-14 | Godot 4.7.2 export templates are installed on the operator's Mac (`~/Library/Application Support/Godot/export_templates/4.7.2.stable`) | `ls` | The pack export can run on this machine |

## 3. Measurement (inherits RL-b §3; only what is new here)

- **M-1 … M-5 unchanged.** Eight fixed views, 120 warm-up and 300 measured frames, vsync off, 1920×1080,
  three fresh processes, a run with any measured frame over 250 ms is INCONCLUSIVE and re-run, never
  averaged in. The statistic is the median over three conclusive runs of each view's p95.
- **M-6 as amended by R-2.** Binding: p95 ≤ 16.7 ms in every view; video memory ≤ 2 048 MB. Advisory
  (reported, not judged): draw calls, primitives.
- **Who measures.** Timing (p95) is **operator-run only**: RL-b's agent runs were repeatedly disturbed (a
  locked or idle session throttles the window to about 1 fps; E-RLb-ruled, E-RLb-final). The implementing
  session may run `--perf` for draw calls, primitives, objects and video memory, which do not depend on the
  window's pacing (E-RLb-final-2 found them identical in a disturbed run), and must label any timing from
  its own runs `NOT USED`.
- **M-9 (new): trials.** A trial is a named lever (or `+`-joined set) applied at load by `SliceTrial`. A
  trial session runs the full M-5 protocol per trial, in the order given, in fresh processes; `none` is the
  head as committed and is always first. Each run's JSON records its trial name. `perf_compare.py` prints
  one row per view and one column per trial (median p95, Δ against `none`), then the R-2 verdict per trial.
- **M-10 (new): the pack.** `3d.pck` exported in a scratch copy with R-a's Linux preset (SD-B2-8), measured
  raw and as `gzip -6 -c 3d.pck | wc -c`, on the base and on the head, same preset, same machine.

## 4. Part (a) — the two views

### 4.1 Candidates, what RL-b already learned, and what is not repeated

| Lever | RL-b evidence | Here |
| --- | --- | --- |
| VoxelGI 256 → 128 (C7a) | taken under R-1 | in the base; no further reduction (a new visible change; the florist already shifted 16–28 %) |
| Soft-shadow filter 3 → 2 (C7b) | taken under R-1 | in the base |
| SSIL quality 2 → 1 (C7c) | V-1 FAIL on 15 views (0.53–3.4 %), up to −0.9 ms | **not repeated** |
| Small interior objects cast no sun shadow (C6) | zero yield: identical draws and primitives; Godot 4.7.2's `shadow_caster_mask` did not lower draw calls | **not repeated** |
| VoxelGI → SSIL in Default (O-2) | ruled a stop (Q-RLb-4) | **not repeated** |
| Fewer sun splits; shorter shadow distance (O-5) | proposed, never measured | T-1, T-2 |
| Coarser merged cells (O-5) | proposed, never measured | T-4 |
| Per-view occluders at the doorway | never proposed in RL-b | **Not a candidate as stated.** Occluders come from opaque massing (SD-RLb-5), never from a benchmark pose: an occluder placed because the perf view looks there tunes the instrument, not the scene, and does nothing for a player standing a metre away. The doorway's `occlusion` breakdown row decides whether any opaque massing is missing from the set; only if it shows a material number of culled-but-drawn objects behind solid walls does a massing occluder become a candidate, and adding it touches `occluders.gd`, outside this design's change set, so it comes back to the operator (SD-B2-3) |
| Mesh LOD threshold | never measured | T-3 |

### 4.2 The trials (SC-1)

| Trial | What it applies | Pixel change expected | Expected yield (stated as a hypothesis, measured, never assumed) |
| --- | --- | --- | --- |
| `none` | nothing | — | the head's reference |
| T-1 `shadow2` | `SHADOW_PARALLEL_2_SPLITS`, `split_1 = 0.16` (the old second boundary, 15.2 m), max distance 95 m unchanged | yes: the near cascade covers 0–15.2 m instead of 0–5.2 m, so near shadows are softer; 15.2–95 m in one cascade | about half the sun's shadow-pass draws (two passes instead of four); the shadow pass was −1.5 to −2.5 ms in total |
| T-2 `shadow60` | max distance 95 → 60 m, with the split fractions rescaled by 95/60 (0.0871, 0.2533, 0.6650) so that the first three cascades keep their absolute ranges (5.2 / 15.2 / 39.9 m) | small: only the last cascade changes (39.9–60 m instead of 39.9–95 m, at a higher resolution); objects beyond 60 m stop receiving sun shadow, with Godot's fade | the last cascade's casters beyond 60 m; at the doorway and the cafe frontage little is beyond 60 m, so the yield may be small there |
| T-3 `lod2` | `Viewport.mesh_lod_threshold = 2.0` (Godot's default is 1.0) | small: the screen-space error a mesh LOD may introduce grows from 1 px to 2 px | fewer primitives in every pass, including the shadow passes; props were 1.99–3.67 M primitives |
| T-4 `cells48` | `SliceBatch.merge(root, 48.0)` instead of 24 m | none by construction (same triangles, same materials); only frustum culling of merged cells changes | fewer draw calls (about 2 100 come from merged cells); on a GPU-bound frame possibly nothing, possibly worse (more vertices outside the frustum drawn) |
| T-5 `shadow60+lod2+cells48` | the three low-risk levers together | as the three | B-3: deltas do not add; this measures the combination SD-B2-4 is most likely to choose |

T-1 is measured even though its visual change is the largest, so that if no low-risk lever closes the gap
the operator rules on a number, not on an estimate (Q-B2-2).

### 4.3 SD-B2-4 — the decision rule, fixed before the trials are measured

From the trial table (operator session 1) and the trials' V-1 results (agent):

1. **Eligible** trials pass V-1 under RL-b's rule (base = `none`'s frames, noise floor from two `none`
   runs, ≤ base share + 0.5 points per view), or fail it only where the change is located and named and the
   operator accepts it at review (as RL-b's V-4).
2. **Sufficient** trials have, in the trial session, median p95 ≤ **16.2 ms** (16.7 ms minus a 0.5 ms margin
   for session-to-session drift) in **cafe frontage and doorway**, and ≤ 16.7 ms in the other six views.
3. Choose, among eligible and sufficient trials, the one with the **fewest levers**, ties broken by the
   smallest V-1 share; T-4 is chosen only if it is itself needed (a no-pixel-change lever that yields
   nothing is dead complexity, as C6 was).
4. **If none is eligible and sufficient: material stop** (§9). The table, the V-1 heat maps and the
   side-by-sides go to the operator, with the options (T-1's visual cost, Q-B2-2) and their numbers. Nothing
   further is added autonomously.

The chosen lever is then committed as plain code (SC-3), and the final measurement (operator session 2) is
taken on the committed head. The trial's numbers are evidence for the choice; the verdict is A-1.

### 4.4 Other design decisions (part a)

| Id | Decision | Alternatives | Why |
| --- | --- | --- | --- |
| SD-B2-1 | **`--trial=` applies levers at load, in any mode; `--trials=` loops.** `SliceTrial.apply(slice, names)` runs once after `SliceWorld.build` and `_gi()`; it knows exactly the trial names of §4.2 and refuses an unknown name (push_error and exit non-zero, so a typo cannot measure the base under a lever's name). T-4 is the one build-time lever: `slice_world.gd` passes `SliceTrial.cell_size()` (24 unless `cells48` is named) into `SliceBatch.merge`. The run JSON records `trial`. The shipped path (no `--trial`) is byte-for-byte today's behaviour | one branch per lever and one operator session per branch (four to five sessions of ~25 min each); editing constants between runs by hand (unreviewable, error-prone) | One operator session measures all candidates; the agent runs the same trials through `--shots` for V-1. The instrument stays for later RL PRs (each of which spends from this budget, step-22 §6.6) |
| SD-B2-2 | **One judge: `tools/perf_compare.py`** (stdlib only) reads a session directory or several, computes the median-of-conclusive-runs p95 per view and judges R-2 (p95 per view, video memory) and reports draws and primitives as advisory. `SlicePerf._aggregate` keeps its median table but its M-6 line is replaced by "verdict: tools/perf_compare.py" and the launcher calls the tool after each session | fix the GDScript line only (then the trial table would need a second judge); judge in both (two verdicts that can disagree) | One implementation of the verdict, testable without a GPU on planted JSON (X-5) |
| SD-B2-3 | **The breakdown gains `doorway` and `omni_shadows`.** `BREAKDOWN_VIEWS = [street wide, cafe frontage, interior, doorway]`; `omni_shadows` switches `shadow_enabled` off on every `OmniLight3D` that has it (B-4). Diagnostic only, as M-7: no pass/fail role. The doorway's `occlusion` row is the evidence §4.1 requires before any occluder is proposed | leave the breakdown as is (the doorway's cost would stay unlocated) | `ARC-23` |

## 5. Part (b) — 512 px textures for small and distant props (MS-Ra-2)

### 5.1 The classification (provisional; confirmed by the texel-density check, SD-B2-6)

Classes follow the ruling's words. **Small**: an object held, carried or placed on a table or shelf.
**Distant**: an object the player cannot approach closer than about 1.5 m from the eye (ceiling and wall
fittings above head height, façade lamps). **Kept at 1k**: furniture and fittings a player stands or sits at
(chairs, tables, benches, shelving units, planters, chalkboards), street lamps (tall: their 1k is already
spread over 4–5 m), foliage props, and every environment surface.

| Class | Slugs (textures) | Floor at 512 px |
| --- | --- | --- |
| Small, hand-held goods | `croissant` (3), `carrot_cake` (3), `food_apple_01` (3), `tea_set_01` (3), `jug_01` (3), `wooden_bowl_01` (3), `wine_bottles_01` (12), `antique_ceramic_vase_01` (3), `ceramic_vase_02` (3), `pot_enamel_01` (3), `brass_pot_01` (3), `wicker_basket_01` (3) | ≥ 1 024 px/m (step-22 §6.4) |
| Small, other | `wooden_crate_01` (3), `planter_pot_clay` (3), `calathea_orbifolia_01` (3), `water_manhole_cover` (3) | ≥ 512 px/m (DEP-8) |
| Distant | `hanging_industrial_lamp` (5), `modern_ceiling_lamp_01` (3), `industrial_pipe_lamp` (4), `wall_clock` (4), `fancy_picture_frame_01` (6) | ≥ 512 px/m (DEP-8) |
| **Kept 1k (props)** | `bar_chair_round_01`, `dining_chair_02`, `painted_wooden_chair_01`, `round_wooden_table_02`, `outdoor_table_chair_set_01`, `painted_wooden_bench`, `planter_box_01`, `Shelf_01`, `steel_frame_shelves_02`, `wooden_display_shelves_01`, `standing_chalkboard_01`, `street_lamp_01`, `shrub_02`, `shrub_03`, `potted_plant_02` (56 textures) | — |
| Kept 1k (not props) | all 66 environment textures under `assets/textures/`, the character, the HDRI | — |
| Not used by the slice | `celandine_01`, `grass_medium_01`, `street_lamp_02`, `wooden_bookshelf_worn` (12) | Q-B2-4 (recommended: unchanged) |

Downsized: **21 slugs, 79 textures.** A slug in the table whose measured 512 px density falls under its floor
is moved to "kept 1k" and recorded; it is never kept small by lowering the floor.

**Expected size (an estimate, to be measured, SD-B2-8):** 79 × (1 398 180 − 349 545) ≈ 82.8 MB less in the
pack and in texture memory. `3d.pck` with `gzip -6`: ≈ 259.5 − 82.8 ≈ **177 MB**, under the 250 MB hard
limit and above the 150 MB goal (Q-B2-5). Video memory ≈ 1 608 − 83 ≈ 1 525 MB.

### 5.2 Decisions (part b)

| Id | Decision | Alternatives | Why |
| --- | --- | --- | --- |
| SD-B2-5 | **`slice_imports.py` owns `process/size_limit`**: a module-level table `SMALL_512 = {slug: class}` (the 21 slugs of §5.1 with their class); textures under `assets/models/<slug>/textures/` get `process/size_limit=512` when the slug is in the table, else `0`. The tool refuses a table entry that is not a directory under `assets/models/` (so an environment set or a character cannot be listed) and prints owned / changed / downsized counts. Idempotent; `--check` as today | `mipmaps/limit` (keeps the 1k file in the pack, saves nothing on disk); resizing the source JPGs (changes provenance-tracked assets, `check_slice_provenance.py`); Basis Universal (a new codec path and transcoding at load, a larger decision); BC1 for opaque albedo (a different visual change on every texture, not the ruled one) | The ruling names size, and `process/size_limit` is the importer's own key for it: the source assets and their provenance stay untouched, the decision is a reviewable diff of 79 `.import` files plus a table |
| SD-B2-6 | **A texel-density check in the built slice** (`./mineworld-slice --texel-density`, headless). `texel_density.gd` walks every node with `mw_category = "gltf"`, and for each mesh surface computes per triangle `sqrt(uv_area · W · H / world_area)` with the surface's albedo texture size (W, H) as loaded and the world area from the global transform (so placement scale counts); it reports per slug the area-weighted median px/m, at the loaded size and at 512, and FAILs any downsized slug under its class floor | judge from the asset's nominal size (ignores UV packing and placement scale); judge only from stills (no number for review) | It turns the ruling's "small" into a measured, falsifiable property and ties it to the two density rules the project already has (step-22 §6.4, DEP-8) |
| SD-B2-7 | **Stills for the operator.** `--shots` (27 views, 1600×900) on the base and on the texture commit; two new crops in `CROPS`: `10b_counter_goods_detail` (from `10_interior_counter`) and `21b_florist_shelf_detail` (from `21_florist_interior`), rectangles chosen from the frames at implementation and recorded; one new pose `26_counter_goods_close` (first person, the eye about 0.6 m from the café counter's goods, the closest a shopper stands). Pairs go to the `review/rl-b2-frames` branch, not this PR's diff | crops only (no worst case); a free-flying camera (not a player's view) | The ruling asks for before/after; the close pose is the honest worst case for hand-held goods |
| SD-B2-8 | **The pack is measured as R-a measured it**, in a scratch copy outside the repository (`/Users/yuema137/mineworld-worktrees/scratch-rl-b2/`): a copy of `clients/` with an uncommitted `clients/3d-spike/export_presets.cfg` holding one preset, `name="Linux"`, `platform="Linux"`, `export_filter="all_resources"`, `exclude_filter="shots/*, screenshots/*, tools/blender/*, tools/texture_art/*, *.md"`; then `godot --headless --path <copy>/clients/3d-spike --import` and `godot --headless --path <copy>/clients/3d-spike --export-pack "Linux" <scratch>/3d.pck`; size `stat -f %z` and `gzip -6 -c 3d.pck \| wc -c`; the pack's file list for the top-20 report. Taken on `main @ 171ea06` first: it must reproduce R-a's 259.5 MB within 1 MB (or the difference is explained) before the head's number counts | commit a preset (R-a's file, its C4); estimate from `.ctex` sizes (does not see what the exporter ships) | The number is comparable with R-a's, and the base re-measure is the instrument check (`ARC-23`) |

## 6. Acceptance and adversarial criteria (fixed before measuring)

| Id | Criterion | Evidence | Owner |
| --- | --- | --- | --- |
| A-1 | **Every view's median-of-three p95 ≤ 16.7 ms, and video memory ≤ 2 048 MB**, on the final PR head (both parts in), gi=voxel, 1920×1080 | operator session 2; `perf_compare.py` verdict pasted into the ledger | operator run |
| A-2 | The chosen lever was chosen by SD-B2-4 from a trial table measured on the texture head, and its V-1 result is recorded | operator session 1's table; V-1 per trial | operator run + agent |
| A-3 | **`3d.pck` ≤ 250 MB with `gzip -6`** on the head; the base re-measure reproduces R-a's 259.5 MB within 1 MB | SD-B2-8 logs | agent |
| A-4 | **Every downsized slug meets its floor** at 512 px; no slug outside the table changed size | `--texel-density` log: `N downsized, 0 under floor`; `slice_imports.py --check` exit 0; `git diff --name-only` lists only `.import` files of the 21 slugs | agent |
| A-5 | **The loaded textures are what was ruled**: the format probe prints `croissant_diff_1k.jpg` as BPTC 512×512 with 10 mip levels and `cobblestone_floor_08_diff_1k.jpg` as BPTC 1024×1024 with 11 | `--perf` header lines (the probe gains a size column) | agent |
| A-6 | **Texture memory falls** by about 83 MB (79 textures) | `--perf` memory line, base vs head (memory does not depend on pacing) | agent |
| A-7 | **Nothing simulated changed**: `--drive`, `--threshold`, `--measure`, `--character`, `--world --link`, `--world --conversation` pass as on the base | each mode's log | agent |
| A-8 | **The shipped path ignores the trial machinery**: with no `--trial`, frames equal the base's within V-1's noise floor (C1), and `SliceTrial` is not consulted beyond one name check | V-1 c1 vs base | agent |

**Mutations** (each must make its check FAIL, then be reverted):

| Id | Mutation | Must fail |
| --- | --- | --- |
| X-1 | `cobblestone_floor_08` added to `SMALL_512` | `slice_imports.py` refuses it (exit non-zero, nothing written) |
| X-2 | `round_wooden_table_02` (a near, large prop) added to `SMALL_512` as "small, other" | A-4: `--texel-density` FAILs it under 512 px/m, or, if its density is above the floor, the result is recorded and the mutation is replaced by a hand-held floor case (`outdoor_table_chair_set_01` as hand-held) — the check must be shown to bite on a real slug |
| X-3 | `--trial=shadowsoff` (a test-only name: sun shadows disabled) | the trial is seen by the instrument: draw calls fall by ≥ 2 000 in street wide (deterministic, agent-run) — then the name is removed |
| X-4 | `--trial=shadow6O` (a typo) | the run refuses the unknown name and writes no record |
| X-5 | a planted session directory with one view at p95 16.8 ms in all three runs | `perf_compare.py` prints FAIL for that view and an overall FAIL; with draws 4 000 it still prints PASS when every p95 ≤ 16.7 (advisory) |
| X-6 | `process/size_limit` set in the `.import` files but the import cache not rebuilt | A-5's probe still reads 1024 for the croissant → the launcher's stamp rule must reimport; if it does not, that is a defect fixed in this PR |

## 7. Visual guards (RL-b §7, re-applied)

```text
V-1  STANDALONE FRAMES. --shots at 1600x900, twice on the base (noise floor), once per lever head and per
     trial (--shots --trial=<t>). frame_diff.gd per view. PASS iff each view's share of pixels over 8/255
     is at most the base's own run-to-run share + 0.5 points. Over that: located with --heat, looked at
     side by side, cause named; an unexplained visible change is a material stop
V-2  THE CHARACTER. No diff to the character's files or assets; --character passes
V-3  CONNECTED FRAMES. --world --conversation's three frames: names on every person, the door toast,
     Alice in plain view at the counter, both caption lines
V-4  EXPECTED, EXPLAINED CHANGE (this PR's two cases). (i) The texture commit: every changed pixel must lie
     on a downsized prop's surface (heat map); then it is the ruled change, shown to the operator as
     before/after, not reverted autonomously. A changed pixel anywhere else is V-1's rule. (ii) A trial
     whose change is located (for example T-1's near penumbrae) is shown side by side and is eligible only
     with the operator's acceptance (SD-B2-4 step 1)
V-5  N/A: the skyline preview was reverted under R-5; the backdrop is the old ridges and no lever here
     touches it (skyline east stays in M-1 as a perf view)
```

## 8. The operator's runs and checklist

**Before each run:** the Mac on power, the screen unlocked and awake, no other Godot window, the slice
window frontmost and untouched for the whole run (a locked or idle session throttles the window and makes
the run INCONCLUSIVE). Run from the repository root of the PR's worktree.

### 8.1 Session 1 — trials (on the C2 head, before C3; about 25–35 minutes)

```sh
caffeinate -d ./mineworld-slice --perf --trials=none,shadow2,shadow60,lod2,cells48,shadow60+lod2+cells48
```

| Step | You run / do | What you should see |
| --- | --- | --- |
| 1 | the command above | `perf session: …/shots/slice/perf/<time>-voxel-1920x1080-none`; a 1920×1080 window opens; on the first run after this PR's texture commit, an import pass first (several minutes, once) |
| 2 | nothing; leave the window frontmost | the camera jumps through eight views, about 7 s each; the terminal prints a table per run (`view mean p50 p95 max …`) and `run verdict conclusive` |
| 3 | — | if a run prints `INCONCLUSIVE`, the launcher re-runs it by itself (up to three times); that is expected occasionally |
| 4 | — | after three conclusive runs, the next trial starts with its own `perf session:` line (`…-shadow2`, and so on); six trials in all |
| 5 | — | at the end, a table from `tools/perf_compare.py`: one row per view, one column per trial, median p95 and Δ vs `none`, then a verdict line per trial (`R-2 PASS` / `FAIL: cafe frontage, doorway`) and video memory |
| 6 | paste the whole terminal output (or `shots/slice/perf/*/compare.txt`) to the session | the agent records it as E-B2-trials and applies SD-B2-4 |

### 8.2 Session 2 — the final measurement (on the C3 head; about 5–8 minutes)

```sh
caffeinate -d ./mineworld-slice --perf
```

| Step | You run / do | What you should see |
| --- | --- | --- |
| 1 | the command above | one `perf session:` line, three runs as in session 1 |
| 2 | — | the texture probe lines at the top: `croissant … BPTC_RGBA 512x512, 10 mip level(s)`, `cobblestone … 1024x1024, 11` |
| 3 | — | the closing verdict: `R-2 PASS` with every view's median p95 ≤ 16.7 ms and video memory about 1 520 MB; draws and primitives listed as `advisory` |
| 4 | paste the output | recorded as E-B2-final; A-1 decided |

### 8.3 Visual checklist (with the stills on `review/rl-b2-frames`, and in play)

1. **Goods up close.** `26_counter_goods_close`, `10b_counter_goods_detail`, `21b_florist_shelf_detail`,
   before and after: croissants, cake, apples, cups, jugs, bottles, vases, baskets. Look for: no visible
   blur or blockiness at a shopper's distance. Name any one item that reads worse.
2. **Lamps, clock, picture frame** in `05_interior_wide`, `12_back_wall`, `10_interior_counter`: the same.
3. **Walls, paving, roofs, chairs, tables**: unchanged (they kept 1k).
4. **The chosen perf lever** (whatever SD-B2-4 chose): its side-by-side, if V-1 located any change.
5. **Play**: `./mineworld-slice`; walk the street, enter the café, stand at the counter. Look for no popping
   of objects or shadows as you move (T-2 / T-3 would show here first, where frames cannot).

## 9. Material stops

- SD-B2-4 finds no eligible and sufficient trial (§4.3 step 4).
- A-1 fails on the final head (the chosen trial's margin did not hold): report the numbers; no further
  lever autonomously.
- A-3 fails (pack > 250 MB after the ruled downsizing): report the top-20 files and the options (Q-B2-5).
- Any change outside §1's change set or in a "no diff" path; an unexplained V-1 change; a V-2 or V-3 failure.
- The doorway's `occlusion` row showing missing massing (§4.1): reported with the evidence, not built here.

## 10. File overlaps and merge order

| File | Other PRs (16c, 12e, 16d frozen and waiting on 12d; R-a draft) | This PR | Conflict size |
| --- | --- | --- | --- |
| `slice_main.gd` | 16c HUD/`connected`; 12e hook; 16d menus | one `SliceTrial.apply` call; the chosen lever's constants in `_sun()` | separate functions; mechanical |
| `slice_world.gd` | 16c `build(parent, connected)` | the `merge` call's argument | adjacent; mechanical |
| `slice_probe.gd` | — | dispatch line, two crops, one view | none expected |
| `batch.gd` | — | one parameter | none |
| `project.godot` | R-a: `import_etc2_astc` (D-Ra-3) | only if the chosen lever is a setting | separate keys |
| `mineworld-slice` | 16c/12e/16d modes | `--trials` loop, `--texel-density`, the compare call | separate lines; mechanical |
| `assets/models/*/textures/*.import` | R-a reads them (pack size) | 79 files rewritten | none (no other PR writes them) |
| `docs/HUMAN_REVIEW_QUEUE.md` | dated lines | a dated line | append-only |

**Order:** RL-b2 depends on nothing unmerged. R-a's C4 3D half and C6 wait for it (step-23 §18.7). If 16c,
12e or 16d merge first, RL-b2 rebases and re-takes V-1's base and both operator sessions on the new main
(thresholds are fixed; baselines are evidence).

## 11. Commit plan

Each commit tracks implementation, deterministic validation and LLM logic review separately, with evidence.

**How to read this plan (for the implementing session).** Do the steps in order. Every step names the
file, the function and the exact behaviour; where a detail is left open ("chosen at implementation"), choose
it, record the choice in §14's ledger, and continue. After each step marked **(record)**, append the command,
its key output lines and PASS / FAIL / INCONCLUSIVE to §14 before moving on. Never mark `[x]` without that
evidence. Paths are relative to the repository root of `impl-rl-b2`; `P` = `clients/3d-spike`. Shell rules
(§13 COMMANDS): no `python3 -c`, no `sed -i`, no heredocs, no `awk`/`xargs`/`curl`; edit files with the
editor tool; one Godot window at a time. **You never run perf for timing.** Any `--perf` you run is for draw
calls, primitives, objects and memory only; label its milliseconds `NOT USED`.

**Frame-diff recipe** (used by several steps). `--shots` always writes its PNGs to `P/shots/slice/` (top
level, overwritten each run). After each `--shots` run, copy the frames out:
`mkdir -p P/shots/slice/rlb2/<name>` then `cp P/shots/slice/*.png P/shots/slice/rlb2/<name>/`. Compare two
folders with
`godot --headless --path P --script res://tools/frame_diff.gd -- <dirA> <dirB> --heat=<dirHeat>` (paths as
`res://shots/slice/rlb2/<name>`). The V-1 rule: each view's "% of pixels over 8/255" must be ≤ that view's
share in `base1` vs `base2` + 0.5 points.

### C0 — `docs(plan): RL-b2 design` (this file)
- [x] Implementation: this design and §13's contract. *Evidence:* commits `d5448be` (draft) and the freeze
  commit on `docs/rl-b2-design`, PR #183.
- [x] Validation: `python3 scripts/check_doc_headings.py` (223 sections, none duplicated),
  `python3 scripts/check_decision_ids.py` (122 ids, distinct).
- [x] Review: freeze rulings §0.1 (primary session; operator Q2, Q3, Q5), 2026-10-14.

### C1 — `3d: perf trials, the doorway in the breakdown, one R-2 judge`
**Goal:** measure every candidate in one operator session and read one honest verdict (SC-1, SC-2).
**Commit boundary:** instrument only; with no `--trial` argument, nothing renders differently.

Steps:
- [x] **1.0 Session start.** Evidence: E-B2-1 (branch and worktree confirmed; base b489543, 10 commits behind origin/main, docs/CI only). `git status` clean; `git log -1` is `origin/main`; `git worktree list` shows
  `impl-rl-b2` held by no other session. Read this file in full and RL-b's §3 and §7.
- [x] **1.1 Base frames (record).** Evidence: E-B2-1. On the unmodified tree: `./mineworld-slice --shots` → copy to
  `rlb2/base1`; run again → copy to `rlb2/base2`. frame_diff `base1` vs `base2`: record each view's share
  (the noise floor). The first run imports (minutes, once).
- [x] **1.2 New file `P/scripts/slice/slice_trial.gd`**, `class_name SliceTrial extends RefCounted`:
  - `const NAMES := ["none", "shadow2", "shadow60", "lod2", "cells48", "shadowsoff"]` (`shadowsoff` is X-3's
    test-only name; it is removed in step 1.11).
  - `static func requested() -> PackedStringArray`: reads the user argument `--trial=<a+b+…>`; returns its
    `+`-separated names, or `["none"]` if absent. Any name not in `NAMES` → `push_error("unknown trial: …")`
    and `requested()` returns `["__invalid__"]`.
  - `static func cell_size() -> float`: `48.0` if `"cells48"` is requested, else `SliceBatch.CELL`.
  - `static func apply(slice: Node) -> bool`: returns `false` (caller quits with exit code 2, writes no
    record) if `"__invalid__"`. Otherwise, for each name: `shadow2` → the `DirectionalLight3D` named `Sun`:
    `directional_shadow_mode = SHADOW_PARALLEL_2_SPLITS`, `directional_shadow_split_1 = 0.16`;
    `shadow60` → `Sun.directional_shadow_max_distance = 60.0`, splits `0.0871`, `0.2533`, `0.6650`
    (= 0.055, 0.16, 0.42 × 95/60); `lod2` → `slice.get_viewport().mesh_lod_threshold = 2.0`;
    `cells48` → nothing here (build-time, step 1.3); `shadowsoff` → `Sun.shadow_enabled = false`;
    `none` → nothing. Print one line `trial  <names joined by +>` either way.
  - `static func label() -> String`: the requested names joined by `+` (for the run record).
- [x] **1.3 Cell size.** `P/scripts/slice/batch.gd`: `static func merge(root: Node3D, cell: float = CELL)`,
  and in `_collect` use the passed `cell` instead of `CELL` (thread it as a parameter). `slice_world.gd`:
  the existing `SliceBatch.merge(...)` call passes `SliceTrial.cell_size()` as the second argument.
- [x] **1.4 Apply call.** `P/scripts/slice/slice_main.gd` `_ready()`: directly after `_gi()` (line ~63),
  `if not SliceTrial.apply(self): get_tree().quit(2); return`.
- [x] **1.5 `slice_perf.gd`.** (a) `BREAKDOWN_VIEWS` += `"doorway"`. (b) `CATEGORIES` += `"omni_shadows"`;
  in `_members`: every `OmniLight3D` under `slice` with `shadow_enabled`; in `_switch`: set their
  `shadow_enabled = on`. (c) The run record (`rec` in `_protocol` and in `_breakdown`) gains
  `"trial": SliceTrial.label()`, and the header line prints it. (d) In `_aggregate`, delete the M-6 verdict
  block (from `var applies` to the end of the function) and print instead
  `verdict: python3 clients/3d-spike/tools/perf_compare.py <session dir>`; keep the median table. (e) The
  format probe line gains the texture's size: `%dx%d` from `t.get_width()`, `t.get_height()`.
- [x] **1.6 New file `P/tools/perf_compare.py`** (stdlib only: `json`, `pathlib`, `sys`, `statistics`).
  `python3 perf_compare.py <session dir> [<session dir> …]`: for each directory, read every `*.json` whose
  `protocol` is `"RL-b M-1..M-5"` and whose `inconclusive` is false; the trial name is the records' `trial`
  (default `"none"`). Per view (the order of the first record's `views`): median of the runs' `wall.p95`
  (`statistics.median`), max draws, max primitives; video memory = max `memory.video_mb`. Print one table:
  rows = views, columns = trials (`median p95` and, for trials after the first, `Δ` vs the first). Then per
  trial: `R-2 PASS` iff ≥ 3 conclusive runs, every median p95 ≤ 16.7 and video ≤ 2048.0, else
  `R-2 FAIL: <failing views / video>` or `R-2 INCONCLUSIVE: n of 3 runs`; then
  `advisory: draws max N (bound 2000), primitives max N (bound 3000000)`. Write the same text to
  `<last dir>/compare.txt`. Exit 0 (it reports; it does not block). Constants at top: `P95_MS = 16.7`,
  `VIDEO_MB = 2048.0`, cite RL-b R-2 in a comment.
- [x] **1.7 Launcher `mineworld-slice`.** (a) New case `--trials=*` → `trials="${a#--trials=}"`, `perf`
  must also be set (error otherwise). (b) In the `--perf` block: if `trials` is empty, behave as today and
  after the three runs call `python3 "$proj/tools/perf_compare.py" "$session"`. If `trials` is set, loop
  `for t in ${trials//,/ }`: session dir suffixed `-$t`, the existing three-run loop with `--trial=$t`
  added to the Godot arguments; collect the session dirs; after the loop call `perf_compare.py` with all of
  them. `--trial=` alone needs no case (it passes through, B-8).
- [x] **1.8 A-8 (record).** `./mineworld-slice --shots` → `rlb2/c1`; frame_diff `base1` vs `c1`: every
  view within the noise floor + 0.5 points. Also `./mineworld-slice --drive`: all checks pass.
- [x] **1.9 Trials' frames (record).** For each `t` in `shadow2`, `shadow60`, `lod2`, `cells48`,
  `shadow60+lod2+cells48`: `./mineworld-slice --shots --trial=$t` → `rlb2/trial-$t`; frame_diff `c1` vs it
  with `--heat=res://shots/slice/rlb2/heat-$t`. Record per trial: max share, views over the bound, and for
  each view over the bound the location (look at the heat image) and the named cause. These results feed
  SD-B2-4 step 1.
- [x] **1.10 Counts (record).** `./mineworld-slice --perf --trial=$t` once per trial (one run is enough;
  stop it after run 1 with Ctrl-C if you like): draws / primitives / objects per view, video memory. Label
  timings `NOT USED`. `./mineworld-slice --perf --breakdown`: record the doorway rows (draws / primitives /
  objects for `occlusion` and `omni_shadows`; timings `NOT USED`). If doorway `occlusion` off adds more
  than 100 objects, raise §9's occluder item to the primary session (do not build occluders).
- [x] **1.11 Mutations (record).** X-3: `./mineworld-slice --perf --trial=shadowsoff` (one run): street wide
  draws fall by ≥ 2 000 vs `none`. Then delete `"shadowsoff"` from `NAMES` and its branch in `apply`.
  X-4: `./mineworld-slice --shots --trial=shadow6O` exits non-zero with `unknown trial`, no PNG newer than
  the command start. X-5: make `/tmp/rlb2-x5/` with three planted JSON files (copy a real record from step
  1.10, set `inconclusive` false, set `cafe frontage` `wall.p95` to 16.8 in all three, `draw_calls` 4000
  everywhere, `trial` "none"): `perf_compare.py /tmp/rlb2-x5` prints `R-2 FAIL: cafe frontage`; set that
  p95 to 16.0 in all three: prints `R-2 PASS` with draws listed as advisory. Delete `/tmp/rlb2-x5`.
- [x] **1.12 Review.** Re-read the diff against §4.2 (split values, threshold location), check that with no
  `--trial` the code path only prints `trial  none`, and that `perf_compare.py` judges only p95 and video.
- [x] **1.13 Commit** (`ad6debc`; pushed with C2, E-B2-5) `3d: perf trials, the doorway in the breakdown, one R-2 judge` (files: the five
  scripts above, `perf_compare.py`, `mineworld-slice`, this file's ledger). Push.

### C2 — `3d: 512 px textures for small and distant props (MS-Ra-2)`
**Goal:** part (b). **Commit boundary:** textures, the density check and the review stills only.

Steps:
- [x] **2.1 Pack size on the base (record).** Evidence: E-B2-5. Before touching any texture, SD-B2-8 on `origin/main`'s
  tree (`git worktree add --detach /Users/yuema137/mineworld-worktrees/scratch-rl-b2/base origin/main`, or
  copy `clients/` from it into `scratch-rl-b2/base-clients/`). Write the scratch
  `clients/3d-spike/export_presets.cfg` with the editor tool: one `[preset.0]` with `name="Linux"`,
  `platform="Linux"`, `runnable=true`, `export_filter="all_resources"`,
  `exclude_filter="shots/*, screenshots/*, tools/blender/*, tools/texture_art/*, *.md"`,
  `export_path="../../3d.pck"`, and a `[preset.0.options]` section (an empty one is accepted; Godot fills
  defaults). Then `godot --headless --path <scratch>/clients/3d-spike --import`, then
  `godot --headless --path <scratch>/clients/3d-spike --export-pack "Linux" <scratch>/3d.pck`. Record
  `stat -f %z 3d.pck` and `gzip -6 -c 3d.pck | wc -c`. **It must be 259.5 MB ± 1 MB** gzipped; if not,
  find and record the reason (preset difference) before going on.
- [x] **2.2 `P/tools/slice_imports.py`.** Evidence: E-B2-5. (a) Add `SMALL_512: dict[str, str]` with exactly the 21 slugs of
  §5.1 mapped to `"hand-held"`, `"small"` or `"distant"`. (b) In `main`, before walking, check every key
  is an existing directory `assets/models/<slug>`; if not, print `slice_imports: <slug> is not a prop under
  assets/models; refusing` and return 2 before writing anything. (c) `wanted(path)` adds
  `"process/size_limit": "512"` when the path is under `assets/models/<slug>/textures/` for a slug in
  `SMALL_512`, else `"0"`. (d) The summary line adds `, N at 512 px`. Update the docstring (cite MS-Ra-2 and
  this file).
- [x] **2.3 Run the tool (record).** Evidence: E-B2-5 (79, then 29 after the floor moves). `python3 P/tools/slice_imports.py`: `213 texture imports owned, 79
  changed, 79 at 512 px`. Run again: `0 changed`. `python3 P/tools/slice_imports.py --check`: exit 0.
  `git diff --name-only`: only `.import` files under the 21 slugs' `textures/` folders.
- [x] **2.4 Texel-density check.** Evidence: E-B2-5. New file `P/scripts/slice/texel_density.gd`,
  `class_name SliceTexelDensity extends RefCounted`, `static func run(world: Node3D) -> int` (returns the
  number of failures). For every node with meta `mw_category == "gltf"` (the prop root; its slug is the
  first `MeshInstance3D`'s scene file name or, simpler, record the slug as a second meta `mw_slug` in
  `Props.gltf` — `props.gd` is then in the change set for that one line, recorded as a bounded addition):
  for each `MeshInstance3D` below it and each surface, take the albedo texture of the active material
  (`BaseMaterial3D.albedo_texture`); skip surfaces without one. For each triangle: world area from the
  global-transformed vertices, UV area from `ARRAY_TEX_UV`; density = `sqrt(uv_area * W * H / world_area)`
  with W, H the texture's loaded width and height. Per slug: area-weighted median density. Floors from
  `SliceImports`-equivalent constants in this file: hand-held 1024, small 512, distant 512 (copy the class
  table as a `const` with a comment that it must equal `slice_imports.py`'s `SMALL_512`; X-2 checks the
  pairing). Print per slug `slug  class  loaded WxH  median px/m  floor  ok/UNDER`, then
  `texel density: N downsized, M under floor`. Skip degenerate triangles (area < 1e-8).
- [x] **2.5 Dispatch.** Evidence: E-B2-5. `slice_probe.gd`: add `"texel-density"` to `MODES`; when it is the mode, after the
  world is built call `SliceTexelDensity.run(slice.world)`, print, and quit with exit code 1 if failures >
  0, else 0. Launcher: `--texel-density` is covered by the existing generic mode list only if added to the
  case at line ~97: add it there, and mark it headless like `--drive`.
- [x] **2.6 Reimport and A-5 (record).** Evidence: E-B2-5. A-4, A-5 PASS; A-6 INCONCLUSIVE locally
  (the windowed run stalled before its memory line), parked to operator session 2. `./mineworld-slice --texel-density` (triggers the reimport by the
  stamp rule; a few minutes). Every downsized slug `ok`, `0 under floor`. If a slug is `UNDER`: move it
  out of `SMALL_512`, re-run 2.3 and 2.6, and record the move (allowed by Q-B2-3; not a stop). If headless
  reports texture sizes of 0, run the same mode without headless (record which). Then one
  `./mineworld-slice --perf` run (stop after run 1): the probe lines show
  `croissant_diff_1k.jpg BPTC_RGBA 512x512, 10 mip level(s)` and `cobblestone … 1024x1024, 11` (A-5);
  record video and texture memory vs step 1.10's `none` (A-6, ≈ −83 MB).
- [x] **2.7 Mutations (record).** Evidence: E-B2-5. X-1: add `"cobblestone_floor_08": "small"` to `SMALL_512`, run the tool:
  it refuses, exit 2, `git status` shows no new change; revert. X-2: add `round_wooden_table_02` as
  `"hand-held"` in both tables, run 2.3 and 2.6: `--texel-density` reports it `UNDER` (exit 1); revert both
  and re-run 2.3 so its files return to `0` (check `git diff` shows none of its files). X-6: touch is not
  needed — record that step 2.6's first run reimported (the launcher printed its import line) and that
  A-5 read 512.
- [x] **2.8 Stills.** Evidence: E-B2-5. `slice_probe.gd`: add to `CROPS` `"10_interior_counter": ["10b_counter_goods_detail",
  Rect2i(...), 2]` and `"21_florist_interior": ["21b_florist_shelf_detail", Rect2i(...), 2]`, rectangles
  chosen by looking at `rlb2/base1/10_interior_counter.png` and `21_florist_interior.png` so that each
  frames the goods (record the rectangles). Add a view `["26_counter_goods_close", Vector3(x, 0.60, z),
  yaw, pitch, FP]` with the eye ≈ 0.6 m from the café counter's croissants/cake (find their position in
  `cafe_interior.gd`; record the pose). Note: the base frames lack these three; take them on the base by
  checking out only `slice_probe.gd` into a scratch copy, or capture the base once more with the new probe
  file and the old `.import` files (`git stash` is forbidden; use `git worktree add --detach` of the C1
  commit plus a copy of the new `slice_probe.gd`). Record how.
- [x] **2.9 V-1 / V-4(i) (record).** Evidence: E-B2-5. `./mineworld-slice --shots` → `rlb2/c2`; frame_diff `c1` vs `c2` with
  `--heat`. Every view over the bound: confirm from the heat image that the changed pixels lie only on
  downsized props; name them. Any change elsewhere → §9 stop.
- [x] **2.10 Review package.** Evidence: E-B2-5 (`review/rl-b2-frames` @ `1d74a02`). Make side-by-side pairs for `10`, `10b`, `21`, `21b`, `26`, `05`, `12`
  (before | after, one PNG each, with the existing tooling or by placing both files in the same folder),
  and push the folders `rlb2/base1`, `rlb2/c2` and the pairs to the branch `review/rl-b2-frames` (images
  only; not part of this PR's diff; create it as an orphan branch in a separate scratch worktree).
- [x] **2.11 Pack size on the head (record).** Evidence: E-B2-6. A-3 PASS (240.3 MB gzipped). Repeat 2.1 on this commit's tree. A-3: gzipped ≤ 250 MB.
  Record raw, gzipped, and Δ vs 2.1. If over 250 MB → §9.
- [x] **2.12 A-7 (record).** Evidence: E-B2-5. `--drive`, `--threshold`, `--measure`, `--character`, `--world --link`,
  `--world --conversation` each as on the base.
- [x] **2.13 Review.** Evidence: E-B2-5. `git diff --name-only origin/main`: no character, HDRI, `assets/textures/` or
  promenade-only model file; floors cite step-22 §6.4 and DEP-8 in code comments.
- [x] **2.14 Commit** (`a5b9558`) `3d: 512 px textures for small and distant props (MS-Ra-2)`. Push. Then **stop and
  ask the primary session to schedule operator session 1 (§8.1)**; give the exact command. Under the
  coordinator's batching rule the session is parked (table below), not waited for.

### C2-E — operator session 1 (§8.1), ledger only
- [ ] **E.1** Record the operator's pasted output (the `perf_compare.py` table) as E-B2-trials in §14.
- [ ] **E.2** Apply §4.3 with §0.1's consequence, mechanically:
  1. Drop every trial containing `shadow2` from automatic choice.
  2. Drop every trial whose step-1.9 V-1 has a view over the bound (located-but-unaccepted is not
     eligible).
  3. Of the rest, keep those with median p95 ≤ 16.2 ms in cafe frontage **and** doorway and ≤ 16.7 ms in
     the other six views.
  4. If any remain: choose the one with the fewest levers; tie → smallest max V-1 share; `cells48` only if
     the trial without it does not qualify. Record the choice and why.
  5. If none remain: **STOP (Q-B2-2, §9).** Report to the primary session the table, the V-1 shares and
     heat images of every trial, and `shadow2`'s side-by-side. Do not commit any lever.
- [ ] **E.3 Commit** `docs(plan): RL-b2 trial measurements (operator)`. Push.

### C3 — `3d: <the chosen lever> (cafe frontage and doorway within 16.7 ms)`
Only after E.2 chose a trial.
- [ ] **3.1** Make the chosen lever permanent with exactly the trial's values: `shadow60` → in
  `slice_main.gd` `_sun()`, `directional_shadow_max_distance = 60.0` and splits `0.0871 / 0.2533 / 0.6650`;
  `lod2` → `project.godot` `[rendering]` `mesh_lod/lod_change/threshold_pixels=2.0`; `cells48` →
  `batch.gd` `const CELL := 48.0`. Each with a comment citing E-B2-trials and this file. Trial names stay.
- [ ] **3.2 (record)** `./mineworld-slice --shots` → `rlb2/c3`; frame_diff `c2` vs `c3` must equal the
  chosen trial's step-1.9 result within the noise floor. One `--perf` run: draws / primitives equal the
  trial's. `--drive` passes.
- [ ] **3.3 Commit** `3d: <lever> (cafe frontage and doorway within 16.7 ms)`. Push.

### C4 — `docs: RL-b2 review package` and operator session 2 (§8.2)
- [x] **4.1** (E-B2-6) `P/README.md`: one paragraph for `--trial`, `--trials`, `--texel-density` and
  `perf_compare.py`. `docs/HUMAN_REVIEW_QUEUE.md`: a dated `VIS-3D-GODOT-2` line for RL-b2 with §8.3's
  checklist and the `review/rl-b2-frames` link.
- [x] **4.2** (E-B2-6; taken before C2-E/C3 because both operator sessions are parked) Commit
  `docs: RL-b2 review package`, push, open the PR (draft) with the ledger summary.
  Ask the primary session to schedule operator session 2 (§8.2).
- [ ] **4.3** Record the operator's output as E-B2-final. A-1 PASS → mark the PR ready (`gh pr ready`),
  lifecycle READY FOR OPERATOR REVIEW — DO NOT MERGE. A-1 FAIL → §9 stop with the numbers.
- [ ] **4.4** `fast` and `test` CI green on the exact final head; `cargo fmt --check`, `cargo clippy
  --workspace --all-targets -D warnings`, `cargo test --workspace` once on the final head (no Rust change
  expected; record).

### Test ownership (`test-ci-gate-rules` §26)

```text
DETERMINISTIC (agent)   the judge (X-5); trial name handling (X-4); the import table (X-1, idempotence);
                        the texel floors (A-4, X-2); loaded size and format (A-5, X-6); pack size (A-3);
                        draws/primitives/memory per trial; V-1 per trial and per commit; A-7 modes
REAL LIFECYCLE (operator)  p95 timing, sessions 1 and 2 (A-1, A-2); the visual checklist (§8.3)
STATIC                  cargo fmt/check/clippy/test once on the final head (no Rust change expected);
                        doc checks
CI IMPACT               no Rust, no Python package; `fast` covers the doc checks; `test` unchanged
REAL MODEL              NOT REQUIRED
```

## 12. Questions for the operator (answered 2026-10-14; rulings in §0.1)

| Id | Question | Recommendation |
| --- | --- | --- |
| Q-B2-1 | Take the trials approach (one ~30 min measurement session for all candidates, then one ~6 min final session) rather than one session per lever? | **Yes.** Two sessions in total instead of five or six, and the choice is made on measured numbers |
| Q-B2-2 | If no low-risk lever closes the doorway's 1.9 ms, may the PR take T-1 (two sun cascades; softer near shadows) after you see its side-by-side, as you did for R-1 — or stop? | **Stop and show** (§9), with T-1's measured number and side-by-side, so you rule on evidence. T-1 is measured in session 1 either way |
| Q-B2-3 | Confirm §5.1's classification: 21 slugs / 79 textures to 512 px (goods, small objects, ceiling/wall fittings); chairs, tables, benches, shelving, planters, chalkboards, street lamp, foliage props and every wall/floor/roof stay 1k | **Confirm**, with the texel-density floors as the guard (a slug under its floor stays 1k automatically) |
| Q-B2-4 | The four models the slice does not use (`celandine_01`, `grass_medium_01`, `street_lamp_02`, `wooden_bookshelf_worn`; 12 textures, ~12.6 MB): leave them at 1k? | **Leave them.** They serve the promenade spike only; whether they ship at all is R-a's export-filter question |
| Q-B2-5 | Accept ≈ 177 MB (estimate) as the result, under the 250 MB hard limit but above the 150 MB goal? Reaching 150 MB would need another ruled change (BC1 for opaque albedo maps, Basis Universal, or roofs at 512) | **Accept**; the goal is not a gate (§7.1), and each further option is a visible change on large surfaces |
| Q-B2-6 | Hand-held goods floor 1 024 px/m (step-22 §6.4), other downsized props 512 px/m (DEP-8): use these as the gate? | **Yes**; both are already project rules |
| Q-B2-7 | `cells48` (no pixel change, probably little yield on a GPU-bound frame): keep it in the trials? | **Yes**: it costs ~4 minutes of session 1 and settles the O-5 batching question RL-b left open |

## 13. Execution contract (filled at the freeze, 2026-10-14)

```text
PROJECT / PR        MVP-0 visual track · S22 / PR RL-b2 — cafe frontage and doorway within 16.7 ms; 512 px
                    textures for small and distant props (MS-Ra-2)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp1/pr-rl-b2-two-views-and-textures.md (this file)
RELATED / BINDING   pr-rl-b-3d-budget.md (§§3, 7, 0.2 R-1…R-6, ledger E-RLb-operator); step-22-realism.md
                    §§6.4, 6.6, 8 (INV-RL-9, INV-RL-10), 9.2; step-23-release.md §7.1 and §18.7 (on
                    mvp0/pr-s23-ra); docs/DECISIONS.md DEP-8, ARC-23; VISUAL_FIDELITY; HUMAN_REVIEW_QUEUE
                    (VIS-3D-GODOT-2); docs/HUMANOID_PROFILE.md (character import ownership)
START               only on the primary session's dispatch (source: primary, 2026-10-14). The implementer
                    is a fresh session (Haiku) that follows §11 step by step and records every (record)
                    step in §14 before the next
IMPLEMENTATION BASE origin/main at the start (>= 171ea06); branch mvp1/pr-rl-b2
WORKTREE            /Users/yuema137/mineworld-worktrees/impl-rl-b2, created by the implementer with
                    `git worktree add <path> -b mvp1/pr-rl-b2 origin/main`; this session only; confirm no
                    other session holds it before editing
APPROVED SCOPE      §1 SC-1 … SC-7, as ruled in §0.1 (Q-B2-1 … 7)
FROZEN INVARIANTS   §1's change set and "no diff" paths; M-1 … M-5 and R-2's thresholds (16.7 ms, 2 048 MB)
                    never changed; SD-B2-4's rule with §0.1's consequence, fixed before session 1; T-1
                    (shadow2) never committed without a fresh operator ruling (Q-B2-2); V-1 … V-4; the
                    floors of SD-B2-6 (1 024 / 512 px/m); the 21-slug table of §5.1 may only shrink (a slug
                    under its floor moves to 1k), never grow; no lever of §4.1's "not repeated" rows; no
                    server/kernel/system/world change; no new asset (DEP-8); timing verdicts only from
                    operator runs
SEQUENCE            C0 (frozen) → C1 → C2 → [stop: operator session 1, §8.1] → C2-E → C3 →
                    C4 with [stop: operator session 2, §8.2]. The two stops are hand-backs to the primary
                    session, which schedules the operator
COMMANDS            ./mineworld-slice modes (--shots, --drive, --threshold, --measure, --character, --world
                    --link/--conversation, --perf for counts only, --texel-density, --trial=); godot
                    --headless --path clients/3d-spike …; the scratch export of SD-B2-8 (outside the repo);
                    python3 clients/3d-spike/tools/{slice_imports,perf_compare}.py; python3 scripts/check_*.py;
                    cargo fmt/check/clippy -D warnings/test once on the final head; gzip, stat, wc; git and
                    gh (no merge). Never python3 -c, sed -i, awk, xargs, curl or heredoc writes. One Godot
                    window at a time
VALIDATION BUDGET   agent: ≤ ~4 h wall time including imports (a full BC7 reimport ≈ 11 min; the scratch
                    import twice); operator: session 1 ≈ 35 min, session 2 ≈ 8 min; real model NOT REQUIRED
LIVE DOCUMENTATION  §11 checkboxes, §14 ledger
HANDOFF             §14's handoff block
ENDPOINT AUTHORITY  (source for each line: the freeze of 2026-10-14 by the primary session on the operator's
                    rulings, applying the working rules' defaults; the start gate is the primary's dispatch)
  implementation + local validation   authorized from the primary's dispatch
  semantic commits                    authorized; no approval before each commit (working rules §14)
  branch push                         authorized
  PR creation / update                authorized (draft until A-1 passes; then `gh pr ready`)
  CI repair                           authorized: `fast` and `test` green on the exact head
  merge                               explicit operator authorization only, merge commit; no source
                                      value changes this line
POST-MERGE SYNC     the planning session owns step-22 and overall; the implementing session owns this
                    file's ledger and merge identity; R-a is told the pack number (it resumes C4/C6)
NORMAL STOP         PR RL-b2 READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       §9
```

## 14. Risks and ledger

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-B2-1 | No low-risk trial closes the doorway (its cost looks per-pixel: VoxelGI, SSIL, SSAO inside the café, B-3) | T-1 measured anyway; §9 stop with numbers; Q-B2-2 |
| R-B2-2 | Session-to-session drift on the operator's Mac (17.11 vs C7a's 17.54 for the same code family) | SD-B2-4's 0.5 ms margin; the verdict is session 2, not the trial |
| R-B2-3 | A lever passes V-1 but pops in motion (T-2's shadow edge at 60 m, T-3's LOD switch) | §8.3 item 5, walked by the operator |
| R-B2-4 | 512 px goods look soft at nose distance | the close pose; the 1 024 px/m floor; per-item feedback moves a slug back to 1k |
| R-B2-5 | The scratch preset differs from R-a's, so 259.5 MB is not reproduced | the base re-measure must match within 1 MB or be explained before the head counts |
| R-B2-6 | A frozen 3D PR (16c, 12e, 16d) merges first | mechanical rebase (§10); V-1 base and both sessions re-taken |

```text
E-B2-0   2026-10-14  design audit on origin/main @ 171ea06 (§2); R-a's §18.7 read on mvp0/pr-s23-ra @ 09c28a1;
                     RL-b's closeout recorded in pr-rl-b-3d-budget.md (E-RLb-operator, RULING, MERGED)
DRAFT    2026-10-14  this design, for the operator's freeze review (§12); PR #183
FREEZE   2026-10-14  DESIGN FROZEN by the primary session on operator rulings Q-B2-2, Q-B2-3, Q-B2-5 and
                     primary rulings Q-B2-1, 4, 6, 7 (§0.1); contract filled (§13); §11 written out step by
                     step for the implementer; §8.1/§8.2 corrected to run on the C2 and C3 heads
```

```text
E-B2-1   2026-10-14  step 1.0 (branch mvp1/pr-rl-b2 from origin/main b489543; worktree impl-rl-b2; no other session held it).
                     step 1.1: `./mineworld-slice --shots` twice on the unmodified tree (run 1 included a ~50 min
                     first import; `DONE 0` both). frame_diff base1 vs base2, over 8/255 share (noise floor), the
                     views with a nonzero share: 09b_character_detail 1.559 % (character animation, not touched by
                     this PR), 08_third_front 0.234 %, 09c 0.225 %, 03r 0.126 %, 03s 0.116 %, 04c 0.130 %, 06 0.092 %,
                     07 0.118 %, 09 0.146 %, 22 0.050 %, others <= 0.02 %. Frames in shots/slice/rlb2/base1 and base2
                     (uncommitted scratch). Verdict: PASS (baseline captured)
E-B2-2   2026-10-14  steps 1.2-1.7 written: scripts/slice/slice_trial.gd (new; NAMES incl. the test-only shadowsoff),
                     batch.gd merge(root, cell) threaded through _collect, slice_world.gd passes SliceTrial.cell_size(),
                     slice_main.gd applies SliceTrial after _gi() (quit 2 on an unknown name), slice_perf.gd (doorway
                     view; omni_shadows category; trial in the record and header; M-6 verdict block and its constants
                     removed; texture probe prints WxH), tools/perf_compare.py (new, stdlib; the R-2 judge),
                     mineworld-slice (--trials= with a per-trial loop, requires --perf, one perf_compare call).
                     step 1.8: `./mineworld-slice --shots` -> rlb2/c1, `trial  none` printed, no script error.
                     frame_diff base1 vs c1: no view above 0.35 % over 8/255; the largest shares are 09b 0.346 % (base
                     noise 1.559 %), 22 0.258 %, 21 0.199 %, 03 0.119 % (base 0.000 %, under the 0.5-point bound).
                     `./mineworld-slice --drive`: "all drive checks pass", DONE 0. Verdict: PASS (A-8 by V-1 rule)
E-B2-3   2026-10-14  step 1.9 (V-1 per trial, bound per view = that view's base1-vs-base2 share + 0.5 points; all
                     five runs exit 0, `trial  <names>` printed, no script error). Frames rlb2/trial-<t>, heat rlb2/heat-<t>.
                     shadow2  (T-1): max 37.27 % (01_street_wide); 22 frames over bound (of the 30 PNGs compared). Located: whole-frame
                              shadow change, the near cascade now covers 0-15.2 m (heat not opened for every view).
                              Not eligible without operator acceptance (Q-B2-2); not chosen autonomously.
                     shadow60 (T-2): max 3.19 % (01_street_wide); 14 views over. Located (heat 01_street_wide): changes
                              on facade and pavement shadow edges and penumbra dots at mid/far range (the last cascade
                              now ends at 60 m). Eligible only with operator acceptance (V-4(ii)).
                     lod2     (T-3): max 1.81 % (09b, character detail, under its 2.06 bound); over bound: 05b 0.63 %,
                              12_back_wall 0.54 % (heat 12): the LOD of small goods, chairs, the pendant lamp and the
                              cabinet bread changes (the 2 px threshold). Eligible only with operator acceptance.
                     cells48  (T-4): max 6.33 % (16_south_side); 8 views over. Located (heat 16): one foliage canopy
                              above the Lakeside Deli frontage, not the merged building geometry. The design expected
                              zero pixel change; this is a finding. Named cause (hypothesis, not yet tested): the merged
                              alpha-blended canopy triangles are drawn in a different order when the cell changes.
                              T-4 is chosen only if it is needed (SD-B2-4 rule 3).
                     shadow60+lod2+cells48 (T-5): max 6.66 % (01r); 25 views over; the union of the three causes above.
                     Next (done in E-B2-4): step 1.10 counts; direct runs, timings labelled NOT USED.
```

```text
E-B2-4   2026-10-14  steps 1.10-1.12 (C1, agent).
                     1.10 counts: `--perf --trial=<t>` run 1 for none, shadow2, shadow60, lod2, cells48,
                     shadow60+lod2+cells48 under shots/slice/perf/rlb2-counts/<t>/run1.json; each has
                     `inconclusive: false` and its `trial`. Timings NOT USED. A first breakdown run was cut
                     short at street wide (its log is overwritten). A full `./mineworld-slice --perf --breakdown`
                     then ran to EXIT 0 (session shots/slice/perf/20261014-184734-voxel-1920x1080). Doorway
                     rows (draws, objs off): occlusion +4 / +4 (below the 100-object trigger, so no occluder is
                     raised, §4.1); shadows -2342 draws; omni_shadows -971 draws. Timings are NOT USED (several
                     rows INCONCLUSIVE from the disturbed run).
                     1.11 X-3: `--perf --trial=shadowsoff`, run 1: street wide draws 3814 (none) -> 1631, a drop
                     of 2183 >= 2000. PASS. The test-only name is then removed from NAMES and from apply.
                     X-4: `./mineworld-slice --shots --trial=shadow6O`: EXIT 2, `ERROR: unknown trial: shadow6O`,
                     0 PNGs newer than the start. PASS. X-5 (no GPU, planted 3-run session, deleted): FAIL names
                     cafe frontage (and doorway, 17.64 ms in the recorded none run); with those at 16.0 and draws
                     4000 it is R-2 PASS (draws advisory). PASS. The planted PASS fixture is synthetic.
                     1.12 review: no --trial prints `trial  none` only (the c1 shots run); perf_compare judges
                     p95 and video only, with draws and primitives advisory. PASS.
```

```text
E-B2-5   2026-10-14  C2 (agent; takeover session, Opus, after the Haiku implementers were stopped).
                     Takeover audit: HEAD ad6debc (C1, unpushed); 84 dirty files were C2 in progress (the
                     tool table, 79 tool-written .import files with process/size_limit=512, texel_density.gd,
                     the props.gd mw_slug line, the probe and launcher dispatch); all kept. origin/main
                     (12 commits, plan docs only) merged first, no conflict.
                     2.1 base pack (SD-B2-8): scratch worktree scratch-rl-b2/base at 9aa185b (C1's base) with
                     the uncommitted Linux preset of SD-B2-8; `--import` EXIT 0, `--export-pack "Linux"` EXIT 0.
                     raw 354 174 684 B, gzip -6 259 551 222 B; R-a 259 509 293 B: +41 929 B, within 1 MB. PASS.
                     2.3 `slice_imports.py`: `213 owned, 0 differ, 79 at 512 px` (the table as written);
                     `--check` EXIT 0; the .import diff touches only the 21 slugs' textures/ folders.
                     2.6 `./mineworld-slice --texel-density` (reimported first; ~1:44 incl. import). Defect found
                     and fixed (bounded): texel_density.gd did not parse (an untyped Variant comparison), so the
                     mode ran nothing and exited 0; now typed, and a table slug not found in the slice counts as
                     a failure (fail closed). Result at 512 px, area-weighted median px/m vs floor:
                       ok     croissant 2048.2, carrot_cake 1149.5, food_apple_01 2397.3 (hand-held, 1024);
                              planter_pot_clay 840.0, calathea_orbifolia_01 1042.7 (small, 512);
                              industrial_pipe_lamp 1189.7, wall_clock 875.2, fancy_picture_frame_01 831.6 (distant, 512)
                       UNDER  tea_set_01 669.7, jug_01 878.7, wooden_bowl_01 862.4, wine_bottles_01 786.5,
                              antique_ceramic_vase_01 534.2, ceramic_vase_02 666.5, pot_enamel_01 821.0,
                              brass_pot_01 521.1, wicker_basket_01 466.1 (hand-held, 1024); wooden_crate_01 223.6,
                              water_manhole_cover 342.1 (small, 512); hanging_industrial_lamp 392.0,
                              modern_ceiling_lamp_01 393.4 (distant, 512)
                     The 13 UNDER slugs moved to "kept 1k" in both tables (Q-B2-3: the table only shrinks; not
                     a stop). Several measure under their floor even at 1k (density doubles at 1k: wicker
                     basket 932, crate 447), so the 1 024 px/m goods floor is not met by these assets at any
                     size here; recorded, not acted on. Re-run: tool `50 changed, 29 at 512 px`, then
                     `0 changed`, `--check` EXIT 0; `--texel-density` `8 downsized, 0 under floor`, EXIT 0.
                     A-4 PASS (8 slugs, 29 textures). The size estimate of §5.1 becomes 29 x ~1.05 MB = ~30 MB.
                     A-5: one windowed `--perf` run 1 (direct godot, rlb2-counts/c2/run1.log), probe lines
                     `croissant_diff_1k.jpg BPTC_RGBA 512x512, 10 mip level(s)` and
                     `cobblestone_floor_08_diff_1k.jpg BPTC_RGBA 1024x1024, 11 mip level(s)`. PASS. Counts equal
                     `none`'s in the four views it finished (street wide 3814 draws / 1 991 868 primitives,
                     cafe frontage 4215 / 3 563 368, interior 3814 / 3 667 079, doorway 3424 / 3 158 635).
                     Timings NOT USED (a 150 s frame; load average 13-22 from other sessions' cargo builds).
                     The window then stalled for 30 min (no progress, 15 % CPU) and was killed before the memory
                     line: A-6 INCONCLUSIVE locally, parked to operator session 2 (expected ~1 578 MB, −30 MB).
                     2.7 X-1: `cobblestone_floor_08` added: `... is not a prop under assets/models; refusing`,
                     EXIT 2, `git status` unchanged. PASS. X-2: `round_wooden_table_02` as hand-held in both
                     tables: tool `3 changed`, `--texel-density` `UNDER` at 234.9 px/m (also under 512), EXIT 1.
                     PASS. Reverted; tool `3 changed` back to 0, no round_wooden_table_02 file in the diff;
                     `--texel-density` `8 downsized, 0 under floor`. X-6: the first 2.6 run printed `building the
                     Godot resource cache...` (the stamp rule reimported) and A-5 read 512. PASS.
                     2.8 crops `10b_counter_goods_detail` Rect2i(220, 400, 960, 220) x2 (the pastry case, the
                     back bar and the apples), `21b_florist_shelf_detail` Rect2i(0, 420, 1040, 380) x2 (the
                     left shelf's vases and the centre table's baskets and pots); view `26_counter_goods_close`
                     at (6.65, 0.60, -14.70), yaw 0, pitch -50, FP: the pastry case is at world x 6.65,
                     z -15.07 (café origin (6.0, 0.14, -8.10), counter cx0 + 0.95), its shelves at y 1.49 / 1.76;
                     the eye is ~0.55-0.6 m from the front croissants. Body on the floor (no WARNING).
                     Base frames for all 33: scratch-rl-b2/base (9aa185b, the old .import files) with the new
                     slice_probe.gd and texel_density.gd copied in: `--shots` EXIT 0 -> rlb2/base-new. (9aa185b
                     is C1's parent; C1 renders as the base, A-8.)
                     2.9 `--shots` on the head -> rlb2/c2. frame_diff c1 vs c2: largest 09b 0.397 % (noise 1.559),
                     09c 0.367 %, 03s 0.162 %, 03r 0.124 %, 03 0.123 %; every view within its bound. base-new vs
                     c2: 26_counter_goods_close 0.287 % (heat: only the two carrot cakes, a downsized prop),
                     10b 0.013 %, 21b 0.000 %, 05 0.011 %, 12 0.025 %. V-1 PASS, V-4(i) PASS (every changed
                     pixel on a downsized prop).
                     2.10 `review/rl-b2-frames` (orphan, scratch-rl-b2/frames) @ 1d74a02, pushed: base/ (33),
                     c2/ (33), heat/, pairs/ (05, 10, 10b, 12, 21, 21b, 26 as -1-before / -2-after), README.
                     2.12 A-7: --drive `all drive checks pass`; --threshold four PASS verdicts; --measure all
                     ok; --character `all character checks pass`; --world --link `all link checks pass`;
                     --world --conversation EXIT 0 (too-far refusal at the door, Alice answers at the counter,
                     caption 2 lines). The headless `keyboard_get_label_from_physical` errors are the display
                     server's and predate this PR. PASS.
                     2.13 `git diff --name-only origin/main`: no character, HDRI, assets/textures or
                     promenade-only file; floors cite step-22 §6.4 and DEP-8 in both files.
```

```text
E-B2-6   2026-10-14  2.11 head pack (SD-B2-8): the same scratch worktree checked out at a5b9558 (C2), same
                     preset; `--import` EXIT 0 (the 29 changed textures), `--export-pack "Linux"` EXIT 0.
                     raw 323 771 700 B (−30 402 984 vs base, = 29 x ~1 048 k), gzip -6 240 317 698 B
                     (−19 233 524 vs base). A-3: 240.3 MB <= 250 MB. PASS. The design's ~177 MB estimate assumed
                     79 textures; 29 remain after the floors, and BC7 gzips a little, so the gzipped saving is
                     smaller than the raw one. Load average 17-40 during the import (other sessions' cargo
                     builds); sizes do not depend on load.
                     C4 4.1: README paragraph (--trial, --trials, --texel-density, perf_compare.py);
                     HUMAN_REVIEW_QUEUE dated VIS-3D-GODOT-2 RL-b2 entry, the checklist reflecting the 8 slugs
                     actually downsized. 4.2: the PR is opened as a draft now, because C2-E, C3, 4.3 need the
                     two parked operator sessions. Local cargo checks not run (no Rust change; R-LOAD-1: CI on
                     the exact head is the gate).
```

**Parked operator checks (batched to milestone).** Coordinator ruling: the two operator perf sessions are
batched into the milestone. The tables below are parked, not run. Each row begins with an explicit `cd`.

Operator session 1 (§8.1, the C2 head, after C2 is committed; about 25-35 min):

| Step | Command or action | Expected result |
| --- | --- | --- |
| 1 | `cd /Users/yuema137/mineworld-worktrees/impl-rl-b2 && caffeinate -d ./mineworld-slice --perf --trials=none,shadow2,shadow60,lod2,cells48,shadow60+lod2+cells48` | a `perf session:` line ending `...-voxel-1920x1080-none`; a 1920x1080 window opens; on the first run after the texture commit an import pass runs first (several minutes, once) |
| 2 | `cd /Users/yuema137/mineworld-worktrees/impl-rl-b2 && ls clients/3d-spike/shots/slice/perf` (leave the window frontmost and untouched) | the camera moves through eight views, about 7 s each; each run prints a table and `run verdict conclusive`; an INCONCLUSIVE run is re-run by the launcher, up to three times |
| 3 | `cd /Users/yuema137/mineworld-worktrees/impl-rl-b2 && ls clients/3d-spike/shots/slice/perf` | six trial sessions (`...-none`, `...-shadow2`, `...-shadow60`, `...-lod2`, `...-cells48`, `...-shadow60+lod2+cells48`), then a closing table from tools/perf_compare.py: per view the median p95 and Δ against none; per trial `R-2 PASS` or `R-2 FAIL: <views>`; video memory; advisory draws and primitives |
| 4 | `cd /Users/yuema137/mineworld-worktrees/impl-rl-b2 && cat clients/3d-spike/shots/slice/perf/*/compare.txt` | the operator pastes the whole terminal output here; recorded as E-B2-trials, SD-B2-4 applied by the agent. `shadow2` is never chosen autonomously (Q-B2-2) |

Operator session 2 (§8.2, the C3 head, after E.2 chose a lever and C3 is committed; about 5-8 min):

| Step | Command or action | Expected result |
| --- | --- | --- |
| 1 | `cd /Users/yuema137/mineworld-worktrees/impl-rl-b2 && caffeinate -d ./mineworld-slice --perf` | one `perf session:` line; three conclusive runs |
| 2 | `cd /Users/yuema137/mineworld-worktrees/impl-rl-b2 && ls clients/3d-spike/shots/slice/perf` | the probe lines at the top of the run output: `croissant_diff_1k.jpg BPTC_RGBA 512x512, 10 mip level(s)` and `cobblestone_floor_08_diff_1k.jpg BPTC_RGBA 1024x1024, 11` |
| 3 | `cd /Users/yuema137/mineworld-worktrees/impl-rl-b2 && cat clients/3d-spike/shots/slice/perf/*/compare.txt` | `R-2 PASS` with every view's median p95 at or below 16.7 ms and video memory about 1 578 MB (1 608 minus ~30 MB for the 29 textures at 512 px, E-B2-5; this also decides A-6); draws and primitives listed as advisory |
| 4 | `cd /Users/yuema137/mineworld-worktrees/impl-rl-b2 && cat clients/3d-spike/shots/slice/perf/*/compare.txt` (the operator pastes the output) | recorded as E-B2-final; A-1 decided |

Session 2 can only run after session 1's numbers are back and the agent has committed C3 (the chosen
lever); if session 1 finds no eligible and sufficient trial, session 2 is not run and §9's stop applies.

Visual checklist (§8.3, with the stills on `review/rl-b2-frames`, any time; about 10 min):

| Step | Command or action | Expected result |
| --- | --- | --- |
| 1 | `cd /Users/yuema137/mineworld-worktrees && git -C impl-rl-b2 fetch origin review/rl-b2-frames && git -C impl-rl-b2 worktree add --detach /Users/yuema137/mineworld-worktrees/view-rl-b2-frames origin/review/rl-b2-frames && open /Users/yuema137/mineworld-worktrees/view-rl-b2-frames/pairs` | a Finder window with 14 PNGs: seven views, each as `-1-before` and `-2-after` |
| 2 | look at `26_counter_goods_close`, `10b_counter_goods_detail`, `21b_florist_shelf_detail`, before and after | croissants, carrot cakes, apples, the florist's clay pots and calathea: no visible blur or blockiness at a shopper's distance. Name any item that reads worse (it goes back to 1k) |
| 3 | look at `05_interior_wide`, `12_back_wall`, `10_interior_counter`, before and after | the pipe lamp, the wall clock and the picture frame read the same |
| 4 | `open /Users/yuema137/mineworld-worktrees/view-rl-b2-frames/heat` | red pixels only on the downsized props (in `26`, only the two carrot cakes); walls, paving, roofs, furniture unchanged |
| 5 | `cd /Users/yuema137/mineworld-worktrees/impl-rl-b2 && ./mineworld-slice` ; walk the street, enter the café, stand at the counter | no popping of objects or shadows as you move; goods at the counter look as before |
| 6 | `cd /Users/yuema137/mineworld-worktrees && git -C impl-rl-b2 worktree remove /Users/yuema137/mineworld-worktrees/view-rl-b2-frames` | the viewing worktree is gone |

**Handoff.** Superseded: see E-B2-6. Next action: wait for the batched operator session 1 (table above),
then C2-E (E.1, E.2 by the mechanical rule), C3, session 2, 4.3, 4.4.

Note on order (bounded, recorded at freeze): the trials' frames (step 1.9) are taken on C1, before the
texture commit, while operator session 1 runs on C2. The texture change touches only downsized props, and
each trial's V-1 is measured against its own `none`, so the trial's located change is unaffected; step 3.2
re-checks the committed lever on the textured tree.

**Handoff (historical, as written at the freeze).** Not started. Waiting for the primary session's dispatch. Next action of the implementer: §11
step 1.0 (create `impl-rl-b2` on `mvp1/pr-rl-b2` from `origin/main`, read this file in full), then steps
1.1 onward.
