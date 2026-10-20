# PR design — S32-p follow-up: levers L-a (chroma-key mannequin) and L-b (body push-out)

## Authority

- **Operator ruling, 2026-10-19** (relayed by the coordinator): "route B is REJECTED as built. Proceed with L-a
  (chroma-key clay figure) plus L-b (push garments out at assembly). Spend cap for L-a: 540 Meshy credits plus
  $1.5 OpenAI; ask before exceeding." The full-cast wardrobe (≈ 3,330 credits) is not approved; it is asked
  separately once L-a passes on Vera, Ivan and Hana. Recorded in `docs/decisions/ARC-138.md`.
- This is a bounded revision of the frozen S32-p design (`pr-s32p-wardrobe-pilot.md`). The scope, the invariants
  I-p-1…I-p-13, the gates and the decision rule are unchanged. The only changes are the two levers the operator
  named. The ruling is this revision's freeze; the coordinator relayed it naming L-a and L-b.

## Problem (from the pilot, ledger §16 L-9/L-10)

1. **Subtraction.** The mannequin in a garment reference was grey. Meshy bakes it into one fused mesh with the
   garment, and Blender must then remove it. Garments close to grey (Vera's jeans, Hana's sage jacket) were removed
   with it.
2. **Poke-through.** A garment is fitted to the body through the template, but nothing keeps it outside the body
   surface. Where the generated clay figure was slimmer than the Meshy body, the garment landed inside it. W-4
   failed in every frame.

## Design

**L-a — the chroma-key mannequin.**

- `recipes/s32p-la.yaml` adds `mannequin: {words, srgb, clay_tolerance_max}`. The magenta is `#E020E0`; no
  garment of the library is near it. It is parsed by `recipe.py` as `Mannequin`. A recipe without the key keeps
  the grey mannequin, so `s32p-pilot.yaml` and its prompts are byte-for-byte what they were.
- `cast.py wardrobe clay --subject K` (new; R-6) renders the wearer's Meshy body in the recipe's colour.
- `prompts/{garment,hair}.txt` take a `{mannequin}` placeholder, filled with the recipe's words.
- `garment_clean.py` reads `clay_tolerance_max`. Above 0.20 the mannequin is a chroma key: clay is found by colour
  alone (no 8 mm geometric condition), with a wider colour tolerance (≤ 0.35).
- The follow-up keeps its own ledger and image log in a new scratch, `cast-scratch-s32p-la`. Bodies, templates,
  regions and the MPFB2 install are linked from the pilot scratch. The recipe caps the spend at **540 credits** and
  **21 image calls** ($1.5 / $0.0684 per three-reference call). `cast.py wardrobe meshy` still admits every run
  against the recipe before the POST.
- `cast.py wardrobe balance` (new) reads the Meshy account balance (`GET /balance`, read-only). The ruling says
  to check it before the first paid call.

**L-b — body push-out at assembly.** In `assemble.py`, the body is now the innermost layer. A part vertex inside
the body, or less than 2 mm outside it, moves to 2 mm outside the nearest body surface point (search radius
100 mm). This runs before the existing layer push-out between parts. The pushed count per part is in
`assemble.json`. The Godot fit at load (`fit_at_load.gd`) does not yet apply L-b: W-9 compares the raw binding fit,
and the push-out is a reference-fit step that S32-c must carry into the client (requirement R-S32c-la-1).

## Validation

- Unit: `test_wardrobe_images.py`. The pilot recipe keeps its grey mannequin; the L-a recipe names magenta in
  every garment and hair prompt and carries the ruling's caps. The existing wardrobe test files stay green.
- Live, zero cost first. **L-b only:** the pilot's parts with the body push-out, six outfits, sheets in
  `cast-scratch-s32p-la/sheets-lb`.
- **L-a + L-b:** 17 new references, then 17 Meshy runs (≤ 510 credits; Vera first), then clean, bind, assemble
  and sheet. Sheets go in `sheets-la`.
- **The page:** `VIS-3D-WARDROBE-P` gains a before / L-b / L-a+L-b comparison per subject.

## Stops

- The Meshy balance is below what the next run needs: stop and report. Do not top up.
- The ledger would pass 540, or the image log 21: refused by the tool, then reported.
- Vera's L-a parts fail the look again: report with the page. No retries beyond the recipe's retry pool (30).

## Results (2026-10-19, live)

**Spend.** The account balance was 20 credits at the first check, so L-b ran first at no cost. After the operator's
top-up the balance read 1,020 before the first paid call. Following the operator's frugal staging, only Vera's six
garments were run, one Meshy task each with no retries:

| Garment | Measured credits | Ledger charge |
| --- | --- | --- |
| vera-tee | 30 | 30 |
| vera-hoodie | 30 | 30 |
| vera-jeans | 30 | 30 |
| vera-trainers | 30 | 30 |
| vera-backpack | 30 | 60 |
| vera-updo | 30 | 60 |

Total: 180 credits measured, 240 charged. Two balance deltas read 60 because another lane spent on the same account
at the same time; the larger value was charged, as the pilot rule says. Image calls: 18 of 21 (≈ $1.23). One of
them replaced the trainers reference, whose first image showed the whole outfit and failed the raw gate.

**Bounded fixes found on Vera (all zero-cost, all in this PR):**

- **Inside test skipped for a chroma key.** Under a garment no clay is visible, so the template is not fitted there.
  The inside test then cut Vera's close-fitting jeans away.
- **Placement by the key-coloured surface.** The crown and the horizontal centre come from the magenta surface
  only, so hair above the head and a backpack behind it no longer shift the figure. Without this, the updo sat
  over her face.
- **Template under the garment by harmonic diffusion.** Where the template has no clay hit, its displacement is
  interpolated from the hit vertices (400 rounds), so it follows the clay under the garment.
- **Stretched-triangle drop.** A fitted triangle whose edge grows past 3× its rest length and past 20 mm is dropped.
  This removed the spikes.
- **Fitted-cover body mask.** A body vertex is hidden when the ray along its outward normal meets a worn
  non-rigid, non-hair part within 50 mm.
- **Ray-based body push-out.** The push-out now casts an inward ray along the part's normal, which reaches a
  vertex sunk deep in the body; the nearest-point test did not.

**Vera's look.** L-a fixes the pilot's largest defects: the jeans are whole, no clay hands remain, the hair sits on
the head, and the spikes are gone. Visible defects remain in the posed frames:

- skin shows through the jeans front;
- light shards on the hoodie;
- a magenta fringe at the hairline;
- the hoodie sleeves are offset from the arms.

Against the operator's lowered bar ("visible holes and hair breakage must be fixed"), Vera does **not yet pass**.
Per the ruling, no credits were spent on Ivan or Hana. Both keep one outfit for now, which the operator called an
acceptable fallback.

The review page `VIS-3D-WARDROBE-P` has a new comparison section per subject and outfit: before | L-b only |
L-a + L-b.

**Zero-credit follow-up on Vera (coordinator's go, 2026-10-19).**

- **Fitted cover from both sides.** A body vertex is also hidden when a worn part lies within 15 mm inside it, so
  skin standing just outside a flat garment triangle no longer shows.
- **Push-out along the skin normal.** The body push-out moves a part vertex along the skin's normal, not the part's
  own noisy normal, so the shell no longer crumples.
- **Magenta hue test.** With a chroma key, a texel whose red and blue both exceed green by 0.15 counts as clay. This
  removes the key's fringe at the hairline.

**Result.** The jeans no longer show large skin patches; a few light flecks remain. The hair is whole on the head,
with small dark-red fragments at the hairline. The hoodie still has light shards, which are its zip and drawcords
smeared by the bake. The hoodie sleeves are still offset from the arms. No credits were spent.

## Follow-up 2: Vera's sleeves and colour (operator option 2, 2026-10-19; zero credits)

Ruling: spend about half a day, at zero credits, on Vera's sleeve offset (pose the template by the arm joints) and a
cleaner colour. No paid calls. #308 has merged, so this change ships as its own PR.

- **D-impl-17, the arms are posed by the shoulder joint (`garment_clean.py` `pose_arms`).** Per side, the shoulder
  pivot is the mean of the innermost 10 % of upper-arm template vertices. The template's arm tip is its hand vertex
  farthest from the pivot. The figure's arm tip is the farthest key-coloured point that lies:
  - beyond the shoulder by at least half the template arm's lateral reach;
  - no lower than the template's fingertips minus 25 cm;
  - within 35° of the template's arm direction.

  The rotation between the two directions is applied to the forearm and hand in full, and to the upper arm with a
  weight ramping from 0 to 1 over the first 40 % of the arm. A correction above 35° is logged and not applied. On
  Vera's hoodie the mannequin's arms were 17.5° more raised than her body's; the other garments needed 2–5°. The
  first two attempts picked a foot, then a thigh, as the arm tip, and the constraints above came from those
  failures.
- **Layer push-out along the inner surface's normal (`assemble.py`).** The body push-out already used the skin's
  normal; the layer push-out now uses the inner part's normal too. Pushing along the outer part's own noisy normals
  drew spikes at the hoodie's hem and opening.
- **Inner layers hidden under outer layers (`assemble.py`).** An inner part's face is removed when an outer, worn,
  non-rigid, non-hair part lies within 50 mm outside it or 15 mm inside it, the same rule as the body's cover. The
  "white shards" on the hoodie were the tee showing through it, not the bake; on Vera, 6,699 tee faces were removed.
- **Bake key fix (`garment_clean.py`).** Baked texels where red and blue both exceed green by 0.15 take the
  garment's median colour.
- **Kept UVs: tried and rejected.** Keeping Meshy's UVs and texture instead of re-baking garbled the tee's graphic
  after decimation, so the bake stays.

**Result (sheet run 10).**
- The sleeves sit on the arms.
- The tee's graphic is readable again.
- The hoodie no longer shows the tee through it; small light fragments remain at the hood edge.
- The jeans keep a few pale flecks.
- Small dark-red fragments remain at the hairline.

## Follow-up 3: Ivan and Hana (operator GO, 2026-10-19)

Ruling: GO on Ivan and Hana, with the wardrobe Meshy cap raised from 540 to 570. One run per garment; at most one
retry, and only for an unusable result. The balance is read before every paid call, and the run stops below 50
credits or at the cap.

**Spend.** 11 runs, measured 30 credits each:

| Garment | Measured | Charged |
| --- | --- | --- |
| ivan-polo | 30 | 30 |
| ivan-jacket | 30 | 30 |
| ivan-trousers | 30 | 30 |
| ivan-shoes | 30 | 30 |
| ivan-panama | 30 | 30 |
| ivan-fade | 30 | 30 |
| hana-shirt | 30 | 30 |
| hana-jacket | 30 | 30 |
| hana-skirt | 30 | 30 |
| hana-flats | 30 | 30 |
| hana-pixie | 30 | 30 |

No retries. The ledger stands at 570 of 570 charged; the measured total for L-a is 510 over 17 runs. The Meshy
balance went from 510 before the first run to 180 after the last. Hana's tights are a tight-fit shell and cost no
credits.

**D-impl-19 (budget.py projection).** The admission check refused the last run, `hana-pixie`, with "540 + projection
60 = 600". The 60 came from the backpack's and the updo's charges, which a shared account inflated (F-impl-5). The
projection now follows §4.6's wording: the largest *measured* `consumed_credits`, falling back to the charge only
when nothing was measured. The run then admitted at 540 + 30 = 570 = cap and charged 30. A new test covers it in
`test_wardrobe_budget.py`.

**Result (all six outfits, sheets-la).**
- **Vera A and B are the best of the three:** the sleeves sit on the arms, the graphic is readable, and only small
  flecks remain.
- **Ivan's outfits have clear holes and clipping.** His registration is the weak link: torso mean error 50 mm,
  and his rig's G-6 fails (pilot L-8). Better garments cannot fix that. The next lever for him is the rig and
  registration (L-e).
- **Hana's outfit A is mostly whole,** with pale patches on the jacket and tights. **Her outfit B** (Vera's tee,
  Ivan's jacket, Vera's jeans) shows more holes: these garments come from other people, so this is cross-body
  fitting.

Every frame keeps a W-4 flag.

## Follow-up 4: Ivan's rig and registration (operator ruling, 2026-10-19; zero credits)

**D-impl-20, `fit.spine_depth: "body-centre"`.** This is opt-in in `fit.yaml`; the default `hips-relative` is
#211's rule, unchanged for Vera and the fused cast.
- **Why.** #211 keeps CharMorph's spinal curve relative to the hips, because a hoodie and a pack hide the chest of
  a clothed Meshy body. On Ivan's bare, heavy, stooped base-layer body that curve put the neck and both shoulders
  outside the body (pilot G-6: Neck 7.6 mm, LeftArm 21.0 mm, RightArm 11.1 mm).
- **What.** With `body-centre`, the trunk joints, the neck, the head and the shoulders take the middle of the
  body's own depth: halfway between the front-most and back-most vertex of a 3 × 2 cm slab at the joint.
- **Where.** Ivan's `pilot/s32p/ivan-petrov/fit.yaml` sets it. His body was rebuilt as `bodies/ivan-petrov/r4` at
  no cost.
- **Result.** G-6(b) margins: Neck +79 mm, LeftArm +92 mm, RightArm +97 mm, Spine1 +138 mm, Spine2 +132 mm.
- **Spine still reads −143 mm.** The gate's depth ray at the waist meets only 7 vertices, all on the back surface
  (back z ≈ front z). The decimated waist under the tank's hem is too sparse there, so this is a sampling
  artifact of the gate, not a joint outside the body. On r2 the same ray met 6 vertices.
- **Outcome.** G-6 is recorded as **FAIL (Spine, sampling)**, with this evidence. The bound is not changed.
- **G-5** reads a roll off by 1.02e-4 rad against a 1e-4 bound; this was already the case on r2.

**D-impl-21, `register --fit-passes N`.** Further surface-fit passes start from the fitted surface, so they reach
a trunk beyond the first pass's 50 mm ray. The default, 1, is unchanged; Ivan and Hana were registered with 3.

| Region | Ivan, mean / p95 mm (r2, 1 pass → r4, 3 passes) | Hana, mean / p95 mm (1 → 3 passes) |
| --- | --- | --- |
| torso | 50.4 / 109.5 → 3.7 / 25.7 | 5.9 / 27.8 → 0.7 / 2.3 |
| face | 73.2 / 99.4 → 12.4 / 53.1 | 17.2 / 30.8 → 4.4 / 9.4 |
| hips | 25.1 / 82.8 → 11.7 / 72.6 | — |
| upper-arms | 24.0 / 79.6 → 4.9 / 32.7 | — |

Ivan's and Hana's outfits were then re-assembled from the existing garments; no paid call was made.

**Follow-up 4, final (body r7).**
- **Neck and head move with the chest.** They now shift with Spine2 as one piece, so their rest angles stay #211's.
  Placing them at their own depth middles (r4) tipped Ivan's head back in every walk frame. Placing the head where
  it was (r5) left the template's face 72 mm off.
- **G-6 on r7:** only the Spine sampling artifact above remains; the neck and both arms are inside the body. G-5
  is unchanged (a roll off by 1.02e-4 rad against a 1e-4 bound, as on r2).
- **Ivan's registration on r7** (mean / p95 mm):

  | Region | Mean / p95 |
  | --- | --- |
  | face | 3.3 / 8.3 |
  | scalp | 0.9 / 3.5 |
  | neck | 4.2 / 14.7 |
  | torso | 2.5 / 9.6 |
  | upper-arms | 4.7 / 29.9 |
  | hips | 11.7 / 72.6 |
  | thighs | 5.2 / 35.0 |

- **Ivan's six garments were re-cleaned and re-bound** against the new template, at no cost. Their earlier binding
  was made on the 50 mm-off template.
- **Inner-layer cover narrowed to 15 mm** (`assemble.py` `LAYER_COVER_M`). The 50 mm reach hid Ivan's polo behind
  the narrow opening of his open jacket.
- All six outfits were re-assembled and re-sheeted.
