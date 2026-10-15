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
- **F-CAST-8 — Template v3 calibration (2026-10-15). OPEN: the operator rules on the style anchor.** Phase 1
  under v2 drew near-photographic faces, average bodies for "solid" and "heavy", 80-year-olds as about 75,
  skin lighter than the brief, and Vera's freckles, updo strands and eyes on other people. Template v3 and
  profile v3 fix the words (calibrated skin, hair-texture, eye, nose, lip, face-shape, body and age phrases;
  a closing CHECK line; per-candidate `candidate_variants`). The anchor question is the operator's: v3
  attaches `../style/anchor_stylesheet.png` (four non-Vera people generated in Vera's style from
  `anchor_fullbody.png`) instead of Vera herself, because it was the only anchor of the three tested that
  kept the finish stylised for the elderly and removed the Vera leakage. Rejecting it means reverting
  `anchors.full_body` and the preamble's first paragraph to v2's, and accepting more photographic elderly
  faces.
  **Sampled values re-drawn by v3's compatibility masks** (same seed, attempt 0 in both worlds; no other
  attribute moved): eyes, nose and lips are now weighted by stratum (an `nh_asian` brief drew deep-set eyes
  and a wide-bridged nose under v2 and read as Mediterranean), so they re-drew for alba-ferreira (nose,
  lips), bob-achterberg (nose, lips), bruno-lindqvist (lips), carol-mensah (lips), celia-okafor (lips),
  dario-russo (lips), elin-berg (eyes, nose, lips), erin-walsh (eyes, nose, lips), felix-okafor (lips),
  grace-liu (nose, lips), greta-novak (eyes, nose, lips), hana-sato (nose, lips), hugo-marchetti (eyes),
  ines-duarte (eyes, nose), ivan-petrov (lips), nils-aaltonen (eyes, nose, lips), otto-brandt (eyes); a
  man's hair style is drawn given his hair texture, so felix-okafor's "wavy" + "buzz cut" became "short side
  part"; the `nh_black` skin range starts at MST 6 (MST 5's swatch is a light golden tan), so carol-mensah's
  skin moved 5 → 6. grace-liu's shoulder bag became a cross-body bag worn at the back hip
  (`casting_inputs.json`, director's field).

## Modularity (operator principle, 2026-10-15)

"我们这些都要做成模块化的，而不是hard code" and "年龄体重族裔也不一定是所有特征，其实还可以有很多很多": the tool is generic.
`../casting.yaml` names the seed, template, inputs, output directory, the worlds with their profiles, the
distinctness categories and the table columns. The profile's `attributes:` define every trait (kind,
dependencies, stratification, distribution, prompt labels); the template phrases them. The tool contains
no attribute name, no population and no cast. A user's world supplies its own `casting.yaml`, profile and
template; the generator side (Meshy or imported models) is out of this tool's scope.
