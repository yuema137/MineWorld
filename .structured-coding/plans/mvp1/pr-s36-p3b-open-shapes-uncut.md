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
- The plant `plant_after_pushout_mm: 4` on `vera-tee` (Vera A): C-2 on the GLB and W-4w must now fail with
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
| 1 | [ ] | [ ] | [ ] | |
| 2 | [ ] | [ ] | [ ] | |
| 3 | [ ] | [ ] | [ ] | |
