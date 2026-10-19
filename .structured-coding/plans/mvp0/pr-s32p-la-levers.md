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
