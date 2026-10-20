## DEP-73 — Garments at Vera's tier: Meshy image-to-3D of the garment on the wearer's clay figure, OpenAI references through `mineworld_gen`, Blender subtraction and MakeClothes binding

**Date** 2026-10-19 · **Status** accepted for the pilot; adoption beyond it waits on `ARC-138` · **Approved by** the operator's ruling Q-S32-1 (2026-10-15; Meshy ≤ 1,200 credits for the pilot) and Q-S32p-6 (≤ 70 image calls) · **Relates to** `DEP-51`, `DEP-53`, `DEP-55`, `DEP-72`, `ARC-138` · **Design** `step-32-wardrobe.md` §§4.2, 6.6; `pr-s32p-wardrobe-pilot.md` §§4.3 (R-6…R-11), 4.6, 4.7, 4.10 (D-p-4)

**Problem.** Garments must look like Vera's own clothes (the same generator and style) and still be separate, fittable meshes.

**Choice.** A garment reference image is made with `gpt-image-2.5-sunburst` (template v2) from the wearer's own Meshy body rendered as grey clay (F-S32p-10); Meshy turns it into one fused mesh (paid plan: the customer owns the output); Blender removes the clay and keeps the garment, decimates it to the slot budget, bakes its colour, and MakeClothes binds it to the `hm08` template (`DEP-72`). Skin-tight garments are a shell offset 0.8 mm from the wearer's body (Q-S32p-5). Part-aware generators remain declined (licence or CUDA, step-32 §4.2).

**Pilot findings (2026-10-19).** Measured cost: 30 credits per garment run (the balance moved by 60 on three runs while another consumer shared the account; the larger was charged, finding F-impl-5). The subtraction as designed (an inside test against the template fitted to the download) cannot tell clay from a garment, because the fit snaps onto the garment; the pilot replaced it with a colour test on the clay faces (D-impl-14), which works on strongly coloured garments (tee, hoodie, polo, shirt) and fails on garments close to the clay's grey (mid-grey jeans, sage jacket). RSWT (D-p-4) was **not evaluated**: the pilot stopped at the look failure before the weights could be judged; T_P's weights come from the body by Data Transfer.

**Revisit** with the levers on `VIS-3D-WARDROBE-P` (a coloured or chroma-keyed mannequin; Meshy multi-image input; a part-aware generator with acceptable terms).
