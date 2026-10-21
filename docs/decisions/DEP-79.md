## DEP-79 — Lakeside's woods: the spike's procedural card trees; at most four CC0 material sets

**Date** 2026-10-21 · **Status** accepted (step-33 §4.2, §4.9, approved 2026-10-15; Q-S33d-2 ruled (A)
2026-10-19; re-verified by S33-d C1) · **Relates to** `DEP-8` (the asset rows), `ARC-177`,
[`REUSE_POLICY.md`](../REUSE_POLICY.md), [`clients/3d-spike/ASSETS.md`](../../clients/3d-spike/ASSETS.md) ·
**Design** `.structured-coding/plans/mvp0/pr-s33d-terrain-water-exterior.md` §5.6

**Problem.** The woods round Lakeside — Jeffrey and Coulter pine, incense cedar, California black oak and
chaparral, drawn as today's regrowth after the 2003 Cedar Fire (Q-S33d-2 (A)) — must close the land sectors
from about 60 m to the far field's `reach_m` at a cost of at most ten draws.

**Options, re-verified 2026-10-21.**

| Candidate | Fact (source) | Verdict |
| --- | --- | --- |
| the spike's procedural cards (`Props.conifer`, `Props.broadleaf`, `Procgen` cards) | zero texture bytes; one `MultiMesh` per species and LOD band | **adopted**; new species forms and cards are added beside the existing ones, existing functions unchanged |
| Poly Haven tree models | CC0 ("Poly Haven's HDRIs, textures, and 3D models are licensed under CC0", polyhaven.com/license); polycounts from the Poly Haven API: `pine_tree_01` 17 427 094, `fir_tree_01` 7 853 731, `island_tree_01` 3 729 692, `fir_sapling` 433 021, `pine_sapling_small` 398 144 | at most two hero trees, and only if S33-d's frame review asks; each needs decimation before use. Not for the stands |
| CC0 low-poly packs | a second style beside the slice's | rejected |
| paid packs | Q-S33-7(c): nothing bought | rejected |

**Material sets.** At most four CC0 sets (bark, forest floor, granite, shingle), each adopted only where a
reviewed frame shows the shipped library cannot read as the material, ≤ 3 MB each, each with a `DEP-8` row in
`ASSETS.md`.

**Heights.** The drawn heights cite the Gymnosperm Database and USDA FEIS (S33-d §5.6). The Silvics of North
America chapters (Burns & Honkala 1990, Agriculture Handbook 654: Jenkinson, Jeffrey pine, vol. 1 pp. 359–369;
McDonald, California black oak, vol. 2 pp. 661–671) could not be read at source on 2026-10-21
(srs.fs.usda.gov refused TLS; the mirrors' PDFs were not text-readable): INCONCLUSIVE, the Gymnosperm Database
and FEIS ranges stand. A secondary check agrees: Jeffrey pine "80 to 140 feet tall" (≈ 24–43 m) and California
black oak "40 to 80 feet tall" (≈ 12–24 m), Virginia Tech dendrology fact sheets 228 and 242.
