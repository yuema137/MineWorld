# PR S36-P3b — Open shapes get their own coverage rule; C-2 and W-4w read the uncut shells

**Lifecycle:** `DESIGN FROZEN — operator-approved via Q-S36-P3-3/4, 2026-10-20` (both answered as recommended in
[`pr-s36-p3-garment-pushout.md`](pr-s36-p3-garment-pushout.md) §10). This is a follow-up section of that PR; its
scope, invariants and ledger below are this PR's own.

**Parent:** [`pr-s36-p3-garment-pushout.md`](pr-s36-p3-garment-pushout.md) (PR #350, unmerged at the time of
writing). **Base:** branch `mvp1/pr-s36-p3 @ 7eeaf03c`; this PR therefore carries #350's commits until #350
merges. **Step:** [`step-36-interpenetration.md`](step-36-interpenetration.md) §4.3.

## 1. Goal

Two findings of S36-P3's execution, ruled by the operator:

- **Q-S36-P3-3 (F-P3 §9.3).** An open shape — an open jacket or hoodie front, a skirt's open hem, a hat brim —
  "covers" a region it does not enclose. C-2, W-4w and the push-out's rule 5 judge it by the winding number,
  which is about 0 for skin under such a shape, so they read the skin as poking through it. Ruling: such a part
  gets **its own coverage rule, as recipe data** (not a part-name or class rule in code).
- **Q-S36-P3-4 (F-P3-10).** The finish phase cuts an inner part's faces under outer parts and masks covered skin,
  so C-2 on the GLB and W-4w on the dump judge cut shells and cannot see a poke under an outer layer (the
  `plant_after_pushout_mm` mutation). Ruling: **C-2 and W-4w read the uncut shells**: `BodyCheck` and each part
  as it stood before the finish phase removed faces.

## 2. Audit (`mvp1/pr-s36-p3 @ 7eeaf03c`)

| Fact | Where |
| --- | --- |
| The finish phase copies the unmasked body to `BodyCheck` before the mask; it removes inner-part faces under outer parts (`LAYER_COVER_M`) with no uncut copy kept | `blender/mpfb/assemble.py` 367–455 |
| C-2 reads `Body` (masked) and `part_<id>` (cut) from the GLB | `mwcast/contact_gates.py` `gate_c2` |
| W-4w reads `Body` and `part_<id>` from the dump; the dump already carries `BodyCheck` (hidden meshes are skipped except that name) | `mwcast/wardrobe/w4w.py`; `godot/pose_sheet.gd` `_skin_cache` |
| `wgates.parts` skips `BodyCheck` (budgets, materials) | `mwcast/wardrobe/wgates.py` |
| No runtime client loads the assembled outfit GLB | repository grep (`clients/`, `mineworld-3d/`, `presentation/`) |
| A garment's coverage is `covers_regions` only; the manifest's part entry is `layer`, `covers`, `class`, `rigid` | `mwcast/wardrobe/recipe.py` `_garment`; `cli.py` `_assemble` |
| The open shapes of the library, by their own descriptions: `vera-hoodie` ("worn open"), `ivan-jacket` ("worn open"), `hana-jacket` ("worn open"), `hana-skirt` (knee-length, open hem), `ivan-panama` (a brim) | `recipes/s32p-pilot.yaml` |
| The Vera body in the scratch specs is `bodies/vera-lindgren/r1/export/vera-lindgren.glb`, written 2026-10-18 — before #371 (natural-pose re-bind, merged 2026-10-20) | `cast-scratch-s32p` |

## 3. Scope

**In:** `recipe.py` (`open_regions`), `recipes/s32p-pilot.yaml` (the five open shapes), `wardrobe/cli.py` (the
manifest carries `open`), `contact_gates.py` (`PartSpec.open_regions`, the drape rule, the uncut-shell lookup),
`wardrobe/pushout.py` (rule 5 uses the drape rule on open regions), `blender/mpfb/assemble.py` (`PartCheck_<id>`
uncut copies), `godot/pose_sheet.gd` (hide and dump `PartCheck_*`), `wardrobe/w4w.py` (read the uncut meshes),
`wardrobe/wgates.py` (skip `PartCheck_*`), PIPELINE.md, tests, this ledger.

**Out:** the push-out's rules 1–4 (they test the shell against the closed `under` surface, unaffected); the
runtime client; the registration; holes; re-binding or re-registering any body; any Meshy call.

## 4. Commits

### Commit 1 — the uncut shells (Q-S36-P3-4)

- `assemble.py` finish phase: after every part is placed (pushed positions, the after-push-out plant, weights,
  armature), each part is copied to `PartCheck_<id>` (same mesh data, weights and armature, its own material
  slot kept) **before** the inner-face removal; the copies get the `region` channel like every part, are
  exported, and never take part in any cover or mask computation.
- `contact_gates.uncut(node)`: `Body` → `BodyCheck`, `part_<id>` → `PartCheck_<id>`. `gate_c2` reads the uncut
  mesh of every node when the GLB has it, and the drawn one otherwise, saying which in `looked_at`
  ("shells: uncut" / "shells: drawn (no uncut copies)").
- `w4w._meshes`: the same lookup on the dump's mesh names.
- `pose_sheet.gd`: `PartCheck_*` hidden like `BodyCheck` and dumped like it. `wgates.parts` skips them.

### Commit 2 — open shapes (Q-S36-P3-3)

- Recipe: optional `open_regions` per garment or hair, a subset of `covers_regions` ("regions this part covers
  but does not enclose"); a region outside `covers_regions` is refused by name. Data for the five shapes of §2.
- Manifest: each part carries `"open": [...]` (from the recipe garment of that id; empty for an id the recipe
  lacks). `PartSpec.open_regions`.
- **The drape rule** (`contact_gates.drape_depth`): for an inner vertex `u` with outward vertex normal `m`, `c`
  its closest point on the shell and `n` that face's normal turned to agree with `m`, the poke depth is
  `(u − c)·n` (positive: `u` is on the shell's outer side). An open shape has no inside, but it has two sides,
  and skin under it lies on its inner side. Orientation comes from the inner surface, so neither the shell's
  winding nor its face order matters.
- `layer_order`: for an outer part, inner vertices in its enclosed covered regions are judged by the winding
  number (unchanged); those in its open regions by the drape rule, with the same bound keys. Which vertices are
  asked about is `beneath` for both (unchanged).
- Push-out rule 5: a probe in an open region of the shell pokes when its drape depth exceeds `gap − 0.5 mm`;
  the push direction is the oriented face normal. Probes in enclosed regions are unchanged.

### Commit 3 — runs and records

- Vera A, Vera B, the `s36-p3` sweep (`outfits.sweep(recipe, "vera-lindgren", 3, "s36-p3")`): assemble through
  both phases, C-2 on the GLB (now uncut), pose sheet with W-4w, all through `with-slot.sh`.
- The plant `plant_after_pushout_mm` on `vera-tee` (Vera A; 4 mm, and 10 mm per F-P3b-2): C-2 on the GLB and W-4w must now fail with
  more body-in-tee vertices past the bound than the unplanted outfit, in the torso.
- PIPELINE.md: C-2 and W-4w read the uncut shells; the drape rule; `open_regions`. This ledger.

## 5. Acceptance

1. C-2 on the GLB and W-4w on the dump read `BodyCheck` and `PartCheck_<id>`; a GLB without them is judged on the
   drawn shells and says so.
2. **The plant fails C-2 and W-4w**: with `plant_after_pushout_mm: 4` on `vera-tee`, C-2 on Vera A's GLB and W-4w
   in every full frame report `Body … through part_vera-tee in torso`, with more vertices past the bound than
   the unplanted Vera A (before this PR: 87 → 88 on the cut GLB, i.e. invisible).
3. Synthetic tests: an open half-cylinder over an arm passes C-2 by the drape rule and fails by the enclosing
   rule; the same shape with the arm pushed 3 mm through it fails the drape rule (part, region named); the
   uncut-shell lookup reads `PartCheck_<id>` when present.
4. Vera A, Vera B and the three sweep outfits are recorded (C-2 at rest per pair, W-4w frames failing) beside
   S36-P3's numbers. As in S36-P3 §9.2, passing outfits are not required where the remaining failures are holes
   or registration (F-P3-3); every remaining failure is recorded with pair, region and depth.
5. `fast` green on the final head.

## 6. Invariants

I-S36-1 … I-S36-5 and I-P3-1, I-P3-2 hold. In addition:

- **I-P3b-1.** No code reads a garment's id, category or class to choose the coverage rule; only the recipe's
  `open_regions` does (I-S36-5, the charter's no-hard-coding rule).
- **I-P3b-2.** The uncut copies are never drawn and never take part in the cover, mask or face-removal
  computations; the drawn outfit is byte-for-byte the same geometry as before this PR.
- **I-P3b-3.** The bounds stay `garment.poke.body` / `garment.poke.layer`; the drape rule adds no bound.

## 7. Ledger

| Commit | Implementation | Validation | Review | Evidence |
| --- | --- | --- | --- | --- |
| 1 | [x] | [x] | [x] | `test_contact_gates.py::test_c2_judges_the_uncut_shells_when_the_outfit_carries_them` (drawn shells PASS, uncut copy FAILs naming part, region, depth); `test_w4w.py::test_w4w_judges_the_uncut_shells_when_the_dump_carries_them`; real Vera A GLB: C-2 on the uncut shells equals S36-P3's push-out-array numbers (body in jeans 255 vs 257, body in tee 24 vs 24), §7.2 |
| 2 | [x] | [x] | [x] | `tests/test_drape.py` (brim over a scalp: PASS by the drape rule, FAIL as enclosing; crown 3 mm through FAILs naming part and region; rule 5 leaves a draped brim alone, moves it read as enclosing, lifts it where the crown pokes); `test_wardrobe_recipe.py` (an open region outside `covers_regions` refused); bounded correction F-P3b-1 (commit `904ad600`) |
| 3 | [x] | [x] | [x] | §§7.2–7.4: Vera A, Vera B, W6-1…3 assembled through both phases (Blender 5.2.2 via with-slot), C-2 at rest three ways, pose sheets (Godot 4.7 via with-slot) with W-4w on 109 full frames each; plants at 4 and 10 mm; PIPELINE.md §§15.1, 16, 17, 18 |

Review: lead's own logic review (charter speed-first). Local checks: ruff check/format, pyright (`mwcast`, `tests`,
`cast.py`) clean; pipeline pytest files touched PASS (63 tests); `check_doc_headings`, `check_scratch`,
`check_secrets` PASS. Rust untouched.

### 7.1 Bounded corrections found in execution

- **F-P3b-1. The drape rule alone over-reports a part that is open only in places.** First run on Vera A: in the
  hoodie's torso (open at the front, wrapping the back and flanks), the drape rule alone read 3 410 body vertices
  past the bound, against 688 by the winding number. Corrected inside the frozen scope: in an open region a vertex
  pokes only when the winding number **and** the drape rule both read it as outside, at the smaller depth (C-2's
  `_away` and rule 5 alike). An open region can therefore only relax the enclosing rule, never tighten it. The
  invariants hold (no bound added, no garment named in code).
- **F-P3b-2. A 4 mm after-push-out plant sits at the bound.** The push-out leaves each shell 2 mm (the gap) off what
  it covers, and the tee's chest is looser than that, so shrinking it by 4 mm moves skin at most about 2 mm past it:
  on the uncut shells body-in-tee goes 24 → 26 vertices past the bound, which is not evidence. The plant is
  therefore also run at **10 mm** (`plant_after_pushout_mm: 10`, test-only, the same field); both are recorded.
- **F-P3b-3. The bodies are not the re-bound ones.** The Vera specs use
  `cast-scratch-s32p/bodies/vera-lindgren/r1/export/vera-lindgren.glb` (written 2026-10-18) and its registered
  template. #371 (natural-pose re-bind, merged 2026-10-20) re-bound the committed cast bodies, not this scratch
  body, and re-registering is out of scope (§3). The pose sheet's client files are this branch's (they include
  #371's idle).
- **Process note.** While clearing this session's own stale wait, the session removed the `cast-slots/slot-*`
  lock directories, including two held by other lanes; their jobs ran on, so the machine briefly ran more than
  two heavy jobs. Reported to the lane lead.

### 7.2 At rest: C-2 on Vera A and B (vertices past the bound / tested, bound 2 mm)

The gate reads the uncut shells; "drawn" forces the cut shells (what C-2 read before this PR); S36-P3 is §9.2 of
the parent (the push-out's arrays, the same uncut geometry).

| Outfit | Pair | S36-P3 (arrays) | Uncut (this PR) | Drawn |
| --- | --- | --- | --- | --- |
| Vera A | Body in jeans | 257 / 4 652 | 255 / 4 679 | 181 / 276 |
| Vera A | Body in tee | 24 / 14 539 | 24 / 14 597 | 20 / 51 |
| Vera A | Body in hoodie (torso open) | 270 / 6 906 | 110 / 6 630 (5 286 by the drape rule) | 3 / 58 |
| Vera A | tee in hoodie | 49 / 3 762 | 27 / 3 642 | 11 / 2 432 |
| Vera A | Body in trainers | 23 / 1 421 | 23 / 1 421 | 15 / 51 |
| Vera A | total, all pairs | 623 | **439** (685 with the open regions ignored) | 231 |
| Vera B | Body in ivan-trousers | 322 / 4 456 | 322 / 4 474 | 155 / 412 |
| Vera B | Body in hana-shirt | 149 / 13 354 | 151 / 13 411 | 133 / 516 |
| Vera B | Body in hana-jacket (torso open) | 15 / 16 | 5 / 13 | 0 / 2 |
| Vera B | hana-shirt in hana-jacket | 13 / 28 | 10 / 32 | 9 / 24 |
| Vera B | Body in hana-flats | 128 / 1 083 | 128 / 1 083 | 102 / 141 |
| Vera B | total, all pairs | 664 | **653** (667 with the open regions ignored) | 414 |

C-2 still FAILs on Vera A and B at rest: the remainder is the jeans, the tee's armpit and the shoes (holes and
seams the push-out cannot close, S36-P3 F-P3-3), handed to the wardrobe lane as in S36-P3 §9.2.

### 7.3 The generated variant (`outfits.sweep(recipe, "vera-lindgren", 3, "s36-p3")`), rest, all pairs

| Outfit | Open shapes | S36-P3 after | Uncut, enclosing only | Uncut, this PR | Drawn |
| --- | --- | --- | --- | --- | --- |
| W6-1 | ivan-jacket (torso) | 6 258 | 5 821 | **2 932** | 637 |
| W6-2 | vera-hoodie (torso) | 1 952 | 1 873 | **1 544** | 330 |
| W6-3 | hana-skirt (hips, thighs), vera-hoodie (torso), ivan-panama (scalp) | 6 221 | 6 063 | **1 917** | 214 |

The three shapes S36-P3 found getting worse under the push-out (ivan-jacket, hana-skirt, ivan-panama) are where the
gain is: W6-1 −53 %, W6-3 −69 % against S36-P3. No sweep outfit passes C-2 at rest; the remainder is again the
shells' holes and the cross-body fits (Ivan's and Hana's garments on Vera), recorded for the wardrobe lane.

### 7.4 The plant and W-4w in motion

Rest (C-2, Vera A, body in tee, vertices past the bound):

| | Uncut (this PR) | Drawn (before this PR) |
| --- | --- | --- |
| unplanted | 24 | 20 |
| `plant_after_pushout_mm: 4` | 26 | 16 |
| `plant_after_pushout_mm: 10` | **78** (torso named) | 13 |

The drawn shells never see the plant (the count falls: the shrunk chest lies where the finish phase cut the tee and
masked the skin); the uncut shells do. W-4w (pose sheet, 109 full frames each: rest, stand, walk, jog and the
captured frames; every frame reads `shells: uncut`):

| Outfit | Frames failing | Notes |
| --- | --- | --- |
| Vera A | 109 / 109 | per frame, body in jeans median 327 past the bound, body in tee 134, body in hoodie 155, tee in hoodie 52 |
| Vera A, plant 10 mm | 109 / 109 | body in tee median 134 → **190**; more than unplanted in 87 of 109 frames; in 22 walk/jog frames fewer (the shrunk front changes which skin lies under the tee) |
| Vera B | 109 / 109 | as at rest |
| W6-1, W6-2, W6-3 | 109 / 109 each | as at rest |

Acceptance §5: 1 met; 2 met at rest (24 → 78 on the uncut shells, invisible on the drawn ones) and in 87 of 109
frames of W-4w — every frame already fails W-4w, so "fails in every frame" holds but is not distinguishing in 22
frames; 3 met; 4 recorded (no outfit passes; the remaining failures are holes, seams and cross-body fits); 5 on CI.
