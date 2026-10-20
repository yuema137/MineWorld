# PR S36-P3 — Garment push-out by winding number, in layer order; W-4w in every sampled frame

**Lifecycle:** `DRAFT — 2026-10-19`. No implementation before a `DESIGN FROZEN` header from the operator and a filled
contract (§10).

**Parent:** [`step-36-interpenetration.md`](step-36-interpenetration.md) §4.3. **Depends on:** S36-P1 merged (#318:
`mwcast/contact.py`, the tolerance file, C-2); **S36-P2 merged** (the `MWCF` frame dump, `mwcast/frames.py`).
**Audit base:** `origin/main @ 5a8c6f28` (2026-10-19), after #318, #319, #320, #323.

**Design session:** worktree `/Users/yuema137/mineworld-worktrees/design-s36-p345`, branch
`docs/pr-design-s36-p345`.

---

## 1. Goal

An outfit is several shells stacked on a body: a tee, a hoodie over it, jeans, shoes. Today two things keep them in
order, and both are hand-written ray tests:

- **At assembly** (`blender/mpfb/assemble.py`), a part vertex that lies inside the body, or less than 2 mm outside
  it, is moved to 2 mm outside. The test casts one ray inward along the vertex's own normal. Then each outer part
  is pushed out of the parts under it the same way.
- **In the pose sheet** (`godot/pose_sheet.gd` `_poke_through`, W-4), each frame's vertices are sorted into 2 cm
  slabs, and a vertex further out than the garment's slab outline by more than 2 mm "pokes through".

A single ray asks the wrong question on an open or folded shell. Example: a hoodie vertex sits 3 mm inside the tee at
the armpit. Its normal points sideways, the ray from 100 mm out meets the arm first, the "gap" reads as −45 mm, which
is beyond `SINK_MAX_M` (40 mm), and the vertex is left inside the tee. The winding number of the tee at that vertex is
about 0.9, so the vertex is inside, and the closest point on the tee is 3 mm away. This PR uses that answer.

After this PR:

1. The push-out runs in Python on arrays, through `mwcast/contact.py` (libigl), layer by layer, inner to outer. Only
   the outer layer moves; the body and every layer already fixed stay put.
2. W-4's slab outline is retired. **W-4w** runs C-2's layer-order test (P1) on every full frame of P2's dump, so the
   rest pose and every sampled walk and jog frame are judged by one rule and one bound.

## 2. Audit (`origin/main @ 5a8c6f28`)

| Fact | Where |
| --- | --- |
| The push-out lives in **`assemble.py`**, not in `garment_clean.py` as the step says (§4.3 cites commit `a2967fb3`, which added the first version there; commits `6f36729d` and `9dbbd64f` moved it into the assembly as L-b). Constants `PUSH_M` 2 mm, `REACH_M` 50 mm, `BODY_REACH_M` 100 mm, `SINK_MAX_M` 40 mm | `tools/character_pipeline/blender/mpfb/assemble.py` lines 37–40, 208–257 |
| Order inside one part: fit by the `.mhclo` formula → drop stretched triangles (faces only; vertices kept) → optional plant `plant_shrink_chest_mm` → body push-out → layer push-out against every placed inner part → weights | `assemble.py` 152–285 |
| After all parts: `BodyCheck` (the unmasked body, W-4's reference), the fitted-cover body mask, inner faces under outer parts removed, the `region` UV channel with the under flag (`v` 1.5 / 0.5) | `assemble.py` 300–420 |
| The same fit formula exists in Python | `mwcast/wardrobe/garment_format.py` `fit` |
| `assemble` is one Blender subprocess launched by `mwcast/wardrobe/cli.py` `_assemble` through `stages.run_stage`; a part's layer is `part_layer(recipe, class)` (hair: the head slot − 0.5) | `cli.py` 645–698; `stages.py` |
| Blender's bundled numpy is used inside the stages; libigl is not installed into Blender and must not be (step §4.3) | `assemble.py`, `garment_clean.py` imports |
| C-2 (P1): for each covering part in layer order, the flagged-under body vertices and lower-layer vertices in its covered regions must lie inside it (winding number > 0.5) or outside by at most `garment.poke.body` / `garment.poke.layer` (2 mm each) | `mwcast/contact_gates.py` `gate_c2`, `_layer_order` |
| `contact.py` offers `inside`, `outside`, `nearest`; distances come from `igl.point_mesh_squared_distance`, which also returns the closest point and face, unused today | `mwcast/contact.py` 141–190 |
| W-4 in the pose sheet: `POKE_MM` 2.0, `SLAB_M` 0.02, `POKE_REGIONS`, `_poke_through(h, sk_data)` per frame in outfit mode (`--outfit <manifest>`) | `godot/pose_sheet.gd` 25, 82–112, 1196–1340 |
| P2's `MWCF` dump carries per vertex the dominant bone, mesh index, region index, under flag and layer, and full frames for every walk and jog frame | `pr-s36-p2-gait-contact.md` §4 commit 1 |
| The outfits: Vera A and B, Ivan A and B, Hana A and B, plus the W-6 sweep (`outfits.sweep`, seed `s32p-w6`); all assets live in scratch (`/Users/yuema137/mineworld-worktrees/cast-scratch-s32p`), none are committed | `pr-s32p-wardrobe-pilot.md` §§4.8, 16; `mwcast/wardrobe/outfits.py` |
| The pilot's state after L-a/L-b (#317): Vera A and B are the best; **Ivan has clear holes and clipping because his registration fails** (torso mean error 50 mm; G-6 fails, pilot L-8); Hana B is cross-body fitted and shows holes. **Every frame still carries a W-4 flag** | `pr-s32p-la-levers.md` "Follow-up 3" |
| Garments come from one fused Meshy mesh each; they carry **no vertex groups** for cords or straps | `garment_clean.py` R-8a/R-9 (one joined mesh, groups rebuilt only for skin weights) |

### 2.1 Findings

- **F-P3-1. The step names the wrong file.** The push-out to replace is `assemble.py`'s L-b body and layer push-out.
  `garment_clean.py` keeps its own R-8a subtraction tolerance (`INSIDE_M`), which is a different job (deciding
  which download vertices are clay) and is not touched. Bounded correction, recorded.
- **F-P3-2. "Thin features excluded by vertex group" has no input.** No garment has vertex groups for cords. It is
  also not needed: the push-out tests a garment against the surfaces *under* it, never against itself, so a cord
  lying outside the tee has winding number about 0 and does not move. A cord sunk into the tee moves out like any
  other vertex. Dropped with this reason (step §4.3 amended in §8).
- **F-P3-3. A push-out cannot repair holes or a failed registration.** Ivan's clipping comes from his registration
  (50 mm torso error), and holes are missing faces. Pushing vertices outward fixes order, not coverage. So W-4w
  cannot be required to pass on every outfit (Q-S36-P3-1).
- **F-P3-4. Moving outer vertices fixes only half of C-2.** C-2 asks whether an inner vertex lies inside the outer
  shell. A sharp inner vertex can poke out between three outer vertices that are each outside the inner surface. So
  each pass also takes C-2's own pokes and moves the three vertices of the outer face nearest each poke (§4,
  commit 2).

## 3. Scope

**In:** `mwcast/contact.py` (one new function, `closest`); `mwcast/wardrobe/pushout.py` (new); `blender/mpfb/assemble.py`
(split into two phases; the ray push-outs removed); `mwcast/wardrobe/cli.py` (`_assemble` runs fit → push-out →
finish); `mwcast/contact_gates.py` (C-2's core takes arrays, so W-4w reuses it); `mwcast/gait.py` or a new
`mwcast/wardrobe/w4w.py` (W-4w on the dump); `godot/pose_sheet.gd` (W-4 and `SLAB_M` removed; the outfit mode dumps
frames); `mwcast/pose_sheet.py` (W-4w merged into `report.json`); PIPELINE.md §§16–18; tests.

**Out:** `garment_clean.py` (F-P3-1); the registration (Ivan, L-e); holes and texture flecks; the runtime client
(`fit_at_load.gd` keeps loading the assembled positions; R-S32c-la-1 carries the push-out to S32-c as positions,
which this PR changes, not the loader); any Meshy call (zero credits).

## 4. Commits

### Commit 1 — `contact.closest` and the push-out core

**Goal.** One function that moves an outer shell out of the surfaces under it, tested on shapes with known answers.

**Scope.**

- `mwcast/contact.py`: `closest(outer: TriMesh, points) -> Closest` with `distance_mm`, `points` (n, 3), `faces`
  (n) from `igl.point_mesh_squared_distance`, and `winding` from `igl.fast_winding_number`. `contact.py` stays the
  only importer of `igl` (I-S36-1).
- `mwcast/wardrobe/pushout.py` (new, numpy only): `push_out(shell: TriMesh, under: TriMesh, gap_m: float,
  reach_m: float, passes: int) -> PushReport`.

**The rule, per pass.**

1. For every shell vertex `v`, take `w` = winding number of `under` at `v`, `c` = closest point on `under`, `d` =
   distance.
2. A vertex **needs moving** when `w > 0.5` (inside) or `d < gap` (outside but too close). A vertex with `d > reach`
   and `w > 0.5` is **not moved** and is reported: a vertex 40 mm or more inside is another limb or a failed
   registration, not this layer's skin (L-b's `SINK_MAX_M` rule, kept as `reach`).
3. The outward direction `n`: `(c − v)/|c − v|` when inside, `(v − c)/|v − c|` when outside. When `|c − v|` is under
   0.01 mm, take the normal of face `faces[i]` and keep the sign whose side has the lower winding number (sampled
   at `c ± 0.5 mm · n`).
4. The target is `c + n · gap`. The displacement field (zero for vertices that do not move) is smoothed over the
   shell's edge graph for 3 rounds (each moved vertex and its 1-ring take the mean of their neighbours'
   displacements, never shrinking a needed displacement), so the shell does not crumple. L-b's notes record that
   pushing along a noisy normal drew spikes; smoothing the field is the fix.
5. **The other half of C-2 (F-P3-4).** For every vertex `u` of `under` flagged under and in the shell's covered
   regions that lies outside the shell by more than `gap − 0.5 mm`, the three vertices of the shell face closest
   to `u` move outward along that face's normal by the poke depth plus `gap`.

Repeat until no vertex needs moving, at most `passes` (3). `PushReport` counts moved, unreached and still-failing
vertices per pass, and the largest displacement in mm.

**Parameters, not bounds.** `gap` is 2 mm (ARC-138's L-b ruling: about the cloth's thickness) and `reach` is 40 mm
(L-b's `SINK_MAX_M`). They say where a vertex goes, not what passes, so they are parameters of the push-out with
their sources in the docstring. What passes is C-2's `garment.poke.body` / `garment.poke.layer` from the tolerance
file (I-S36-2 holds).

**Implementation.**

- [ ] `contact.closest` and its `Closest` result type (pyright strict, no ignores).
- [ ] `pushout.push_out` with the five rules above and `PushReport`.

**Validation (synthetic, `tests/test_pushout.py`).**

- Two concentric open cylinders (radius 100 mm inner, 98 mm outer, both open at both ends, as a sleeve is): after
  push-out every outer vertex is ≥ 101.5 mm from the axis; C-2 on the pair PASSes. Before, C-2 FAILs with 2 mm + the
  gap.
- One outer vertex dragged 50 mm inside: reported as unreached, not moved.
- A sharp inner cone tip poking 3 mm through a coarse outer quad, with all four outer vertices outside the inner
  surface: rule 5 moves the face, C-2 PASSes.
- Mutation: rule 5 disabled → the cone case fails C-2. Mutation: `w > 0.5` replaced by `w > 1.5` → the cylinder
  case fails.

**Acceptance.** Each planted case ends exactly as stated; every reported count matches the construction.

**Commit boundary.** `closest`, `pushout.py`, tests. Nothing calls it yet.

### Commit 2 — the assembly in two phases

**Goal.** Blender fits and finishes; Python pushes in between. Nothing is installed into Blender.

**Scope.**

- `assemble.py` gains `"phase": "fit" | "finish"`.
  - **fit:** the existing steps up to and including the plant, for every part in layer order (fit, stretched-face
    drop, `plant_shrink_chest_mm`). It writes, per part and for the body, `<id>_fitted.npy` (vertices, Blender world
    frame, float64) and `<id>_faces.npy` (int32), with Blender's bundled numpy. The body's arrays are the unmasked
    body (the future `BodyCheck`). It also writes each body vertex's region (the `region` channel's rule) so the
    push-out knows covered regions.
  - **finish:** repeats the deterministic fit steps (same inputs, same order), then replaces each part's vertex
    positions with `<id>_pushed.npy`, refusing a count mismatch by name, and continues with weights, the body mask,
    the inner-face removal, the under flags and the export, unchanged.
  - The ray body push-out and layer push-out are deleted, with `PUSH_M`, `REACH_M`, `BODY_REACH_M`, `SINK_MAX_M`.
- `cli.py` `_assemble`: `run_stage(phase=fit)` → `pushout.run(out_dir, parts in layer order)` → `run_stage(phase=finish)`.
  `pushout.run` builds the `under` surface of each part as the body plus every lower non-hair part already pushed,
  restricted to faces in the part's covered regions widened by one ring (so a sleeve's edge still sees the arm),
  and writes `<id>_pushed.npy` and `pushout_report.json`. Hair is pushed out of the body only (it covers no region).
  Parts of class `hair` are never an `under` surface for others except the head slot (headwear over hair), which
  keeps `part_layer`'s order.
- A new optional spec field `plant_after_pushout_mm` (test only, never a deliverable): applied in the finish phase,
  after the pushed positions are loaded, by the same chest-shrink rule as `plant_shrink_chest_mm`. It exists so that
  a mutation can still turn W-4w red once the push-out repairs the old plant.
- The command runs under the root uv environment (it needs numpy and libigl): `uv run --locked python
  tools/character_pipeline/cast.py wardrobe assemble …`. Without them it fails with `ContactUnavailable`'s message
  (I-S36-4), never silently skips the push-out.

**Implementation.**

- [ ] `assemble.py` phases; arrays written and read; the ray code removed.
- [ ] `pushout.run` and `pushout_report.json` (per part: moved, unreached, still failing, worst displacement).
- [ ] `_assemble` orchestration and the plant field.

**Validation.**

- Python test: a fake `out_dir` with two nested parts written as `.npy` → `pushout.run` writes pushed arrays whose
  C-2 PASSes, and a report with the expected counts.
- One real assembly (through `with-slot.sh`) of Vera A: the GLB exports; C-2 at rest PASSes with every margin
  printed; `pushout_report.json` lists every part. The before/after C-2 numbers go into §9.
- The old plant `plant_shrink_chest_mm: 4` on `vera-tee`: before this PR W-4 red; after it, the push-out repairs it
  (C-2 PASS). `plant_after_pushout_mm: 4` → C-2 FAIL naming `part_vera-tee` and the torso.

**Acceptance.** Vera A assembles through both phases with no Blender-side push-out code left; C-2 at rest PASSes; the
after-push-out plant fails C-2.

**Commit boundary.** The assembly runs the new push-out end to end.

### Commit 3 — W-4w: C-2 on every sampled frame; W-4 retired

**Goal.** One layer-order rule at rest and in motion.

**Scope.**

- `contact_gates.py`: `_layer_order` takes arrays (positions, faces, regions, under flags per layer) instead of GLB
  readers, so it serves both C-2 (rest, from the GLB) and W-4w (per frame, from the dump). C-2's behaviour and
  messages are unchanged; its tests stay green.
- `mwcast/wardrobe/w4w.py` (new): `gate_w4w(frames: Frames, manifest: OutfitManifest, tolerances) -> GateResult`.
  For every full frame of the dump, the layers come from the labels' mesh and layer fields, the regions from the
  region field (part vertices take their nearest body vertex's region, as in C-2), the under flags from the labels.
  The verdict names part, region, clip and frame: "vera-tee pokes 3.4 mm through vera-hoodie in torso, walk frame
  12 (garment.poke.layer 2 mm)". `GateId.W4W = "W-4w"`.
- `pose_sheet.gd`: the outfit mode writes the `MWCF` dump (P2's writer, `--dump-frames`) for every captured frame
  and every walk and jog frame; `_poke_through`, `POKE_MM`, `POKE_REGIONS` and `SLAB_M` are removed. W-4's slot in
  `report.json` (`checks.poke_through`) is filled by Python.
- `mwcast/pose_sheet.py`: after Godot, read the dump, compute W-4w per frame, write it into `report.json`, recompute
  each frame's `pass`, then write `sheet.html` (P2's pattern for G-9w).

**Implementation.**

- [ ] `_layer_order` on arrays; C-2's tests unchanged and green.
- [ ] `gate_w4w` and its per-frame report.
- [ ] Pose-sheet outfit mode: dump on, slab code removed; `report.json` merge.

**Validation.**

- Synthetic `MWCF` (P2's test writer): two frames of nested shells, the inner pushed 3 mm through the outer in
  frame 2 only → W-4w FAIL naming frame 2; the same at 1 mm → PASS.
- Vera A sheet run (through `with-slot.sh`): W-4w per frame recorded; the old W-4 flag counts (every frame flagged,
  levers ledger) beside the new counts in §9.
- Mutation: `plant_after_pushout_mm: 4` → W-4w FAIL in every frame on the torso.

**Acceptance.** No slab or `POKE_MM` code remains in `pose_sheet.gd`; W-4w fails the plant in every frame and names
part, region, clip and frame.

**Commit boundary.** W-4w and the retirement of W-4.

### Commit 4 — the outfits, a second set, and the records

**Goal.** Run it on every outfit the pilot has, and on outfits no one tuned it on.

**Runs (all through `with-slot.sh`, zero credits):**

- The six deliverable outfits (Vera, Ivan and Hana, A and B): assemble, C-2, sheet, W-4w.
- **The generated variant:** `outfits.sweep(recipe, "vera-lindgren", count=3, seed="s36-p3")` — a seed no earlier run
  used, so three outfits drawn from the pilot library that no one has looked at. Same runs.
- The planted hoodie: `vera-hoodie` shrunk 3 % about its centroid (`plant_scale_pct: -3`, a new test-only field
  beside the other plants) → C-2 FAIL with the old ray push-out disabled and with no push-out; PASS after this PR's
  push-out.

**Records.**

- [ ] PIPELINE.md §16 (assembly: the two phases, the push-out rule and its parameters), §17 (W-4 row → W-4w, the dump,
      the bound keys), §18 (C-2's array form).
- [ ] This ledger, §9: a table per outfit — moved/unreached per part, C-2 at rest, W-4w frames failing (before → after).
- [ ] `ARC-138`: a dated note that L-b's push-out is now the winding-number push-out of this PR (method changed, ruling
      unchanged). No new decision id.

**Commit boundary.** Evidence and specification.

## 5. Acceptance criteria (PR level)

1. The ray push-outs are gone from `assemble.py`; the push-out runs through `contact.py` only (I-S36-1), inner layer
   to outer, moving only the outer layer.
2. **Rest pose (deterministic):** every assembled outfit — the six deliverables and the three `s36-p3` sweep outfits —
   passes C-2 at rest after the push-out, except vertices the push-out reports as unreached (beyond `reach`), which
   C-2 then reports by name. The planted 3 % hoodie fails before and passes after.
3. **Motion:** W-4w is recorded for every full frame of every outfit above. **Vera A, Vera B and at least one `s36-p3`
   sweep outfit pass W-4w in every sampled frame.** Ivan's and Hana's outfits pass or are recorded findings with
   frame, region and depth, handed to the wardrobe lane (Q-S36-P3-1).
4. The after-push-out plant fails C-2 and W-4w in every frame.
5. No bound literal for W-4w in `pose_sheet.gd`, `pose_sheet.py` or `w4w.py` (I-S36-2).
6. `fast` is green on the final head.

## 6. Invariants

I-S36-1 … I-S36-5 (step §5), and:

- **I-P3-1.** The push-out never moves the body or a layer already fixed. Order is `part_layer`'s, inner first.
- **I-P3-2.** Nothing is installed into Blender's Python; arrays cross the boundary as `.npy` files.

## 7. Risks

| Risk | Effect | Mitigation |
| --- | --- | --- |
| Smoothing pulls a needed vertex back inside | a pass leaves pokes | smoothing never shrinks a needed displacement; up to 3 passes; the report counts what still fails |
| A widened covered region lets a sleeve see the trunk | a sleeve pushed off the arm by the chest | the `under` surface is the covered regions plus one ring only; C-2 per region reports where |
| Two Blender launches double the assembly time | slower runs | measured in §9; both phases are a few seconds per part today |
| Pushed positions drift from the runtime fit | the client draws a different shell | the client loads the assembled positions (S32-c's R-S32c-la-1); W-9 still compares Godot's load with Python's |

## 8. Proposed edits to shared documents

- Step 36 §4.3: "Replace the ray push-out in `blender/mpfb/garment_clean.py`" → "in `blender/mpfb/assemble.py` (L-b)";
  drop "thin features excluded by vertex group" with F-P3-2's reason; acceptance per §5 here (Q-S36-P3-1).
- PIPELINE.md §§16–18 (commit 4).

## 9. Ledger

| Commit | Implementation | Validation | Review | Evidence |
| --- | --- | --- | --- | --- |
| 1 | [ ] | [ ] | [ ] | — |
| 2 | [ ] | [ ] | [ ] | — |
| 3 | [ ] | [ ] | [ ] | — |
| 4 | [ ] | [ ] | [ ] | — |

## 10. Questions for the operator

| Id | Question | Recommendation |
| --- | --- | --- |
| Q-S36-P3-1 | The step asked that Vera's, Ivan's and Hana's outfits pass W-4w in every frame. A push-out fixes layer order, not holes or Ivan's failed registration (F-P3-3). Accept "every outfit passes C-2 at rest; Vera A, Vera B and one new sweep outfit pass W-4w in every frame; Ivan and Hana are recorded findings"? | Yes. Ivan's fix is his registration (lever L-e), outside this step |
| Q-S36-P3-2 | Run the push-out between two Blender phases (Blender fits, Python pushes, Blender finishes), rather than installing libigl into Blender's Python? | Yes, as the step already says: nothing goes into Blender |

## 11. Execution contract (filled at freeze)

```text
PROJECT / PR:          S36-P3 — garment push-out by winding number; W-4w per frame
PRIMARY DESIGN DOC:    .structured-coding/plans/mvp1/pr-s36-p3-garment-pushout.md
RELATED / BINDING:     step-36-interpenetration.md; pr-s36-p1-contact-module.md; pr-s36-p2-gait-contact.md;
                       pr-s32p-wardrobe-pilot.md; pr-s32p-la-levers.md; PIPELINE.md §§16–18; ARC-138
IMPLEMENTATION BASE:   origin/main at freeze, with S36-P1 and S36-P2 merged
APPROVED SCOPE:        §3
FROZEN INVARIANTS:     I-S36-1 … I-S36-5; I-P3-1, I-P3-2
SEQUENCE:              commits 1-4
VALIDATION BUDGET:     Blender and Godot only through cast-slots/with-slot.sh; zero Meshy credits; no money
LIVE DOCUMENTATION:    this file, §9
```
