## Findings and operator rulings (appended from `CASTING_NOTES.md`, hand-written)

Every finding of the casting director (2026-10-15) was ruled by the operator on 2026-10-15 in the primary
session.

- **F-CAST-1 — Vera had no full-body reference. RULED: generate one.** `../style/anchor_fullbody.png`
  (one OpenAI call, `gpt-image-2.5-sunburst` through `mineworld_gen`, conditioned on the identity anchor;
  sidecar `anchor_fullbody.provenance.yaml`) is template v2's full-body style anchor. The chest-up
  `../style/anchor.png` stays the identity anchor. Known misses of the anchor: arms about 15-20 degrees from
  the body rather than 35 (still clear of the torso); canvas sneakers rather than the 3D body's low-tops.
- **F-CAST-2 — Lakeside declares no setting. RULED: San Diego County profile**, as chosen
  (`../populations/lakeside.yaml`).
- **F-CAST-3 — Seven Lakeside person files state no pronoun. RULED:** bruno, dario, finn and hugo are men;
  celia, elin and greta are women; recorded in `casting_inputs.json` as "operator ruling 2026-10-15".
  **Follow-up for the S26-cast PR:** state this in `worlds/lakeside/people/{bruno,celia,dario,elin,finn,greta,hugo}.yaml`
  and re-baseline the digests that move. Not done here, so no digest moves on this branch.
- **F-CAST-4 — Build. RULED: use San Diego County (CHIS) figures if readable, else national with verify.**
  The CHIS county figures could not be read at source on 2026-10-15 (AskCHIS is interactive; the county PDF
  timed out; the CDPH page failed TLS verification). `build` keeps NHANES 2015-2018 national figures,
  marked **verify** in the profile's `sources.county_build`, which records the unread figures as leads.
- **F-CAST-5 — Statures outside §7.2's literal windows. RULED:** G-4 checks a brief's stature against the
  profile's band for the same sex and age (mean +/- 2 SD); dated bounded-correction note under `ARC-95` in
  step-26 §§6.1, 7.2.
- **F-CAST-6 — Vera's 1.767 m is above the 95th percentile** of US women in their twenties. Informational.
- **F-CAST-7 — Prompts. RULED: prompts describe only visible features** — skin tone (MST level in words),
  hair colour, texture and style, face shape and features. No census category appears in any prompt; the
  sampled ancestry stratum stays in the briefs and in this file, for the distribution only.
- **F-CAST-8 — Template v3 calibration. REJECTED by the operator 2026-10-15; v2 retained.** A calibration
  (22 image calls; record in the operator's local `cast-out/calibration/` scratch, outside the repository, not committed)
  tried a template v3 that changed three things:
  - a stylised-render style paragraph;
  - a style-sheet anchor of four non-Vera people in place of Vera;
  - calibrated visible-feature wording, with face features re-weighted by stratum.

  The operator, verbatim: "v2很好，v3全都不行" (v2 is good; none of v3 is acceptable), then "不行，完全回到v2"
  (no — go fully back to v2). Template v2, the Vera full-body anchor, profile v2, the casting inputs and
  every brief and prompt are therefore exactly as at `9ddeb571`. Nothing of v3 is kept.

## Modularity (operator principle, 2026-10-15)

"我们这些都要做成模块化的，而不是hard code" and "年龄体重族裔也不一定是所有特征，其实还可以有很多很多": the tool is generic.
`../casting.yaml` names the seed, template, inputs, output directory, the worlds with their profiles, the
distinctness categories and the table columns. The profile's `attributes:` define every trait (kind,
dependencies, stratification, distribution, prompt labels); the template phrases them. The tool contains
no attribute name, no population and no cast. A user's world supplies its own `casting.yaml`, profile and
template; the generator side (Meshy or imported models) is out of this tool's scope.
