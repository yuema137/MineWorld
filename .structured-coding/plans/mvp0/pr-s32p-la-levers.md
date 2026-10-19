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
