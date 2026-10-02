# Reference assessment — production routes for the default 3D character

**Status:** informational research spike. **This document is not authoritative.** It decides
nothing, changes no contract, and amends no decision. It records what was found about alternative
ways to produce the MineWorld default character so that the operator can decide, and so that a
later contributor does not repeat the search. Where this document and
[`DECISIONS.md`](../DECISIONS.md) (`DEP-8`, `ARC-9`, `ARC-17`, `ARC-19`, `ARC-21`, `ARC-24`),
[`VISUAL_FIDELITY.md`](../VISUAL_FIDELITY.md), [`HUMANOID_PROFILE.md`](../HUMANOID_PROFILE.md),
[`REUSE_POLICY.md`](../REUSE_POLICY.md) or
[`CHARACTER_IDENTITY.md`](../../presentation/mineworld-default/3D/CHARACTER_IDENTITY.md)
disagree, those govern and this file is the defect.

| | |
| --- | --- |
| **Question** | Is there a materially better route than CharMorph + Blender-modelled garments and hair + generated textures to a character that reads as the person in `04_character_closeup.png`, at its stylized-realistic quality, within `DEP-8` (relicensable under MIT), glTF → Godot 4.7, `SkeletonProfileHumanoid`, and `ARC-19` §3 (no in-house reconstruction technology)? |
| **Date** | 2026-10-01 |
| **Evidence read** | the reference at full resolution; the current candidate's frames in the `vis-character` worktree (`presentation/mineworld-default/3D/candidate/`, branch `vis/3d-human-pipeline`, frames dated 2026-09-30, branch at `c8a65f6` when read); its `FIDELITY_REVIEW.md`; `clients/3d-spike/tools/hair.py` and `garments.py` headers; vendor terms and model licences fetched 2026-10-01 |
| **Licence method** | every licence verdict below quotes the clause it rests on, with the URL it was fetched from on 2026-10-01. Two vendors (Tripo, pixiv/VRoid) returned HTTP 403 to the fetcher; for those the quote is from the vendor's own page **as returned by search, not read first-hand**, and the row says so. Quotes were extracted by a fetch tool and should be re-read at the URL before any decision rests on one word of them |
| **What was not done** | nothing was generated. See §6 for why no free tier was tried |

---

## 1. Recommendation

**No alternative route is materially better as a whole, and the reason the current candidate fails
is not its base.** CharMorph/Vitruvian is a CC0, rigged, profile-compatible body whose proportions
the morph search has already brought within measurement of the reference
(`FIDELITY_REVIEW.md` §6); every route that could replace it either fails `DEP-8` outright
(Character Creator, VRoid, Hunyuan3D, Rodin, every free tier of a hosted generator) or produces a
single fused, unrigged, statue-like mesh that would trade the parts that now pass — garment
structure, freckles, tee graphic, grip, rig — for a face that resembles the reference only in paint.
What fails is **how the three recognition-carrying parts are authored**: the hair is ribbons swept
procedurally over a fitted ellipsoid, the hoodie is the body surface pushed out along its normals
(which is exactly why it reads padded — it has no drape and no folds), and the face is a realistic
parametric head being pushed by morphs toward a *stylized* face that no realistic-proportion morph
set contains. Those are artist tasks — sculpting, grooming, cloth draping — being performed by
code, and no amount of parameter tuning in code will reach the reference's quality. **The
recommended route is therefore the current route with those three parts made by the method
the industry uses for them, on the same rig:** (1) a **commissioned work-for-hire artist** to
sculpt the head toward the reference on the existing CharMorph topology, groom the updo as hair
cards, and drape the hoodie — the only option that plausibly reaches the reference's quality,
estimated at **USD 3,000–8,000 and 3–6 weeks** with a CC0 dedication in the contract; or, if
money is not to be spent, (2) the same three jobs done by an agent in Blender with the right tools
rather than procedural code — **hair curves with Blender's CC0 Essentials node groups converted to
cards and textured with CC0 strand alphas, and a cloth-simulated hoodie** — which will close the
"padded" and "cap of stripes" defects but will **not** close the face, because sculpting a
stylized face toward a reference is not something a headless agent does well. Image-to-3D is
worth one bounded experiment, not a route: generate the **head and the backpack only**, from a
paid Meshy account or a locally run TRELLIS.2 with its non-commercial background remover swapped
out, and use the head as a **shape target** that the CharMorph head is wrapped onto in Blender,
keeping its topology, UVs, rig and eyes. That is the one place where a generator's strength
(matching a picture's shape) meets the current route's weakest part, and it is cheap to test once
a GPU or an account exists. **Confidence: moderate.** The licence verdicts are high-confidence;
the fidelity predictions for routes nobody has run on this reference are informed judgement, and
§7 says where they could be wrong.

---

## 2. Comparison

Fidelity is predicted against `CHARACTER_IDENTITY.md` and stated as the specific categories each
route is expected to pass or fail, because `ARC-17` §5 bans the comparative alternative.

| Route | Expected fidelity against the reference | `DEP-8` (relicensable under MIT?) | Effort and time | Automatable / regenerable | Main risk |
| --- | --- | --- | --- | --- | --- |
| **A. Current route, procedural** (as it stands) | body silhouette, gender, garment category, freckles, tee, grip pass; **hairstyle category, face shape, skin fail**; hoodie reads padded | **Yes** — CC0 data, own output (`CHARACTER_ASSET_AUDIT.md` §§3, 10) | already spent; further tuning has diminishing returns | fully scripted, regenerable headless | more rounds of tuning code toward an artist's result |
| **B. Current rig + artist methods by agent** (hair curves → cards, cloth-sim hoodie, material pass) | likely closes hood loft, padded hoodie, hair silhouette and strand separation; **face shape and the stylized look still fail** | **Yes** — Blender Essentials node groups are CC0, OpenGameArt strand alphas CC0, sim output is ours | 1–3 agent-weeks | partly: a groom and a sim are reproducible from a committed `.blend`, not from a short script | groom quality limited by agent dexterity; still no sculpt |
| **C. Commissioned artist on the current rig** (head sculpt, groom, garments, textures) | the only route that plausibly passes **face shape, hairstyle, skin and the stylized-realistic quality together** | **Yes, if the contract says so** — work-for-hire or assignment plus a CC0 dedication; inputs restricted to CC0 | USD 3,000–8,000; 3–6 weeks plus review rounds | no; the `.blend` and source textures are committed and edits are manual | contract wording; artist quality varies; operator review rounds |
| **D. Image-to-3D, whole character** (Meshy / Tripo paid, TRELLIS.2 local) | face reads as the reference **in texture only**, geometry soft; hair a solid shell; hood, cords, backpack fused into one mesh; back and legs invented; **no expression rig** | **Meshy paid: yes** ("own their Customer Output"); **TRELLIS.2 local: yes**, if run with MIT BiRefNet not RMBG-2.0; Tripo paid: probably (not read first-hand); **all free tiers: no** (CC BY or vendor-owned); Rodin: unclear; Hunyuan3D: no | hours to generate; 1–2 weeks to clean, remesh, rig, retarget | regenerable in principle, but outputs are not deterministic and a vendor may change models | discards the parts that pass; result reads as a scanned statue up close |
| **E. Image-to-3D, parts only, as wrap targets** (head shape, backpack) | can move the head's shape toward the reference while keeping CharMorph topology, eyes, rig; does not supply hair or skin quality | as D for the generator used | 3–7 agent-days once a GPU or account exists | the wrap is a Blender modifier stack, reproducible | the generated head's shape may be no closer than the morphs |
| **F. Character Creator (Reallusion)** | high quality ceiling for realistic humans; stylized-realistic needs paid content | **No.** "The unique topology and character rigs of CC Avatar … are the property of Reallusion" | licence plus content purchases | n/a | excluded by licence |
| **G. MPFB2 / MakeHuman** | same class as CharMorph — a realistic parametric body; its CC0 hair and clothes are older, lower-detail assets | **Yes** for core and the CC0 packs; some packs are CC-BY | 1–2 weeks to switch | scripted | a sideways move: same ceiling, new integration cost |
| **H. VRoid / VRM** | anime base head and MToon shading; the face cannot be pushed to this reference without replacing the head | **No.** pixiv base models and presets are "not CC0"; pixiv "retains the copyrights" (not read first-hand) | days | partly | excluded by licence and by style |

---

## 3. What the gap actually is

Read from `side_by_side_head.jpg`, `side_by_side_chest.jpg` and `full_body_sheet.jpg` at their own
framing, reference fact beside candidate fact. These are this spike's reads, recorded to explain
the recommendation; the authoritative per-category verdict is the candidate's own
`FIDELITY_REVIEW.md`.

| | Reference | Candidate | What produces the difference |
| --- | --- | --- | --- |
| Face style | stylized-realistic: large almond eyes with a heavy upper lash line, short small nose, full soft cheeks, a small rounded chin, planes blended with no hard edges | realistic adult proportions: smaller eyes with exposed sclera, a long nose bridge, hollow cheeks under the zygomatic, a flat-planed jaw | CharMorph's morphs are calibrated to real anatomy; the reference's eye-to-face ratio is outside what a realistic morph set spans. This is a **style** gap, not a proportion gap, which is why the morph search closed the ratios and the face still fails |
| Skin | warm, matte, soft subsurface glow at the nose and ear | grey-brown with a wet specular sheen | material settings; cheap, and correctly deferred by the candidate until the face is settled |
| Hair | a tall, wide, soft mass of clumped wavy locks with highlights; individual strands separate and catch light | a thin cap of flat dark stripes following the skull; a few ribbons beside the ears; visible gaps | `hair.py` sweeps ribbons from the hairline to one gather point across a fitted ellipsoid. There is no clumping, no layering of under- and over-coat, no strand texture variation per card |
| Hoodie | jersey drape: the sleeve falls in soft folds, the front panels hang, the hood bunches in irregular folds | a smooth inflated shell, sleeves of uniform diameter, the hood a level roll | `garments.py` copies the body surface and pushes it out along its normal, so the garment is a dilated body; nothing in that method can produce a fold |
| Pose and life | weight on one leg, head turned, a hand on the strap, a soft smile | upright, arms held off the body, flat feet, a neutral mouth | a separate agent is on pose and idle animation; out of scope here, but it is half of what "stiff" means |

**So "stiff, not like the reference" decomposes into three authoring problems and one animation
problem.** None of them is "the base body is wrong", and only one of them — the face — is a
reconstruction problem at all.

---

## 4. Per route

### 4.1 Image-to-3D and character reconstruction generators

#### What these tools produce, independent of vendor

All current single-image generators — hosted (Meshy, Tripo, Rodin, Hunyuan3D cloud) and open
(TRELLIS.2, TripoSG, Step1X-3D, Hunyuan3D 2.1, Stable Fast 3D, SPAR3D) — produce **one closed
surface with a baked colour texture**. For this reference that has six consequences, and they hold
whichever vendor is chosen:

1. **Hair, hood, drawstrings, strap and hand are fused into the body.** The hair becomes a solid
   sculpted shell; the reference's loose strands, which `CHARACTER_IDENTITY.md` §4 says are "a large
   part of why the head reads as a person rather than a mannequin", are either lost or become
   blobs. A hosted auto-rigger then skins the hood and the hand-on-strap to the torso.
2. **The reference is cropped at mid-thigh and three-quarter on.** Legs, feet and the back are
   invented. The practical workflow is to first generate a full-body, front-facing, A-pose image
   of the same character with an image model (an `ARC-9` generated candidate), then lift that —
   which adds a second place for the identity to drift.
3. **Lighting is baked into the albedo.** The reference's warm low sun ends up in the texture,
   which `DEP-8`'s coherence procedure (one material authority, one lighting rig) then has to
   remove.
4. **Topology is generator output** — dense, triangulated, not edge-flowed for deformation. A
   retopology pass (Blender's QuadriFlow or a manual pass) is needed before skinning looks right at
   the shoulders and elbows.
5. **No face rig.** The eyes, brows and mouth are paint on a surface. The 26 FACS blendshapes the
   Vitruvian head ships with (`CHARACTER_ASSET_AUDIT.md` §6; the current export does not yet carry
   them, `FIDELITY_REVIEW.md` §4), the off-camera gaze and the smile become impossible to animate.
6. **Quality on faces is uneven and is texture-led.** Two independent comparisons agree on the
   pattern. Vitalify's portrait test (2026-07-10,
   <https://www.vitalify.asia/en/blog/generative-ai/ai-image-to-3d-generators-comparison>): Meshy 6
   *"Simplified facial features"*, *"slight cartoon-like appearance"*; Hunyuan3D *"Less expressive
   face"*, *"softer facial details"*; TRELLIS.2 *"Numerous holes, broken hair strands, and mesh
   artifacts"*; Rodin 2.5 *"loss of clothing and hair information"*. Ideate's character test
   (2025-04-12, <https://ideate.xyz/blogs/posts/ai-3d-model-comparison-trellis-tripo-meshy-rodin-hunyuan>):
   the hosted services *"delivered superior facial textures via post-processing"* while Meshy
   *"incorrectly added a beard to the female character"*. Both are third-party blog tests, not
   controlled benchmarks; they are cited for the pattern, not for a ranking.

**Prediction for this reference.** A whole-character generation would probably pass gender
presentation, apparent age, hoodie colour and category, tee colour and possibly a blurred tee
graphic, and jeans. It would probably fail hairstyle (a shell, not a messy updo with loose
strands), drawstrings and ribbed cuffs as structure, the hand-on-strap as a separable pose, and
face shape at chest-up framing, where the soft geometry under painted features reads as a statue.
It would trade categories the current candidate already passes for a face that matches only in
paint. **That is why route D is not recommended**, and why route E uses a generator for shape
only.

**Rigging is not the obstacle.** Meshy's and Tripo's auto-riggers return humanoid skeletons with
Mixamo-convention names (Meshy docs <https://docs.meshy.ai/en/webapp/guides/3d-model/rigging>;
Tripo developer docs <https://developers.tripo3d.ai/en/docs/animations-rig>), and UniRig is MIT
(`gh api repos/VAST-AI-Research/UniRig` → `MIT`). As `CHARACTER_ASSET_AUDIT.md` §3 already
established, a `mixamorig:` name is a convention, not authorship; any such skeleton maps to
`SkeletonProfileHumanoid` through a generated `BoneMap` the way the CharMorph rig does
(`HUMANOID_PROFILE.md` §1). The obstacle is what is being rigged.

#### Licence, per tool

**Meshy (hosted).** Terms of Use, last updated 2026-09-19, <https://www.meshy.ai/terms-of-use>:

> Free plan: *"Meshy owns all right, title, and interest, including all intellectual property
> rights, in and to the Customer Output"* and *"makes such rights available to free plan customers
> under the Creative Commons Attribution 4.0 International License"*, requiring *"appropriate
> credit to Meshy."*
>
> Paid plan: *"such customers on a paid Meshy plan own their Customer Output."* Customers *"grant
> Meshy a non-exclusive, royalty-free, worldwide license"* over User Content, which the Terms
> define to include Customer Output. *"The Service may generate similar or identical 3D models for
> different users who provide similar 2D inputs."*

Verdict: **free tier fails** — the output is Meshy's and only licensed CC BY 4.0, which we may
redistribute but not relicense under MIT. **Paid tier passes**: owned output may be placed under
MIT; Meshy's own non-exclusive licence does not prevent that. Requires an operator account and
money, so it was not tried. Help-centre wording adds a condition — ownership *"provided you do not
publish them publicly to the Meshy Community"*
(<https://help.meshy.ai/en/articles/10137554-what-is-the-ownership-of-the-generated-models>) — so
the generation must stay private.

**Tripo (hosted).** Primary terms at <https://www.tripo3d.ai/terms> returned **HTTP 403** and were
**not read first-hand.** Tripo's own help page, as returned by search: *"Tripo retains all rights to
Inputs and Outputs generated by Free Users, including all Intellectual Property rights"* (Terms
5.2.1); Paid Users *"hold all rights to their Inputs and Outputs, including commercial use,
modification, distribution, and licensing"* (Terms 5.2.2); free-plan models are public under CC BY
4.0. Verdict: **free fails; paid probably passes, unconfirmed** — the clause must be read at the
URL before any paid generation is committed.

**Rodin / Hyper3D (hosted).** Terms <https://hyper3d.ai/legal/terms>: Hyper3D *"will not limit your
use of such Output, subject to any restrictions set forth in these Agreements"*. The Copyright
License <https://hyper3d.ai/legal/copyright-license> grants *"a worldwide, non-sublicensable,
non-exclusive license … to sell, redistribute, modify, publish, or sublicense the purchased
content"*, and prohibits use *"to train, improve, enhance, or otherwise develop competing AI
models"*. Verdict: **unclear, treat as fail.** The output is licensed, not owned, under a grant
that calls itself both non-sublicensable and sublicensable; an MIT licence is a sublicense. That is
precisely the ambiguity the `DEP-8` amendment resolves conservatively.

**TRELLIS.2 (Microsoft, open weights, run locally).** Model card
<https://huggingface.co/microsoft/TRELLIS.2-4B>: *"This model is released under the MIT License."*
Code: `gh api repos/microsoft/TRELLIS.2` → `MIT`. Two dependencies matter, read from the shipped
`pipeline.json` (<https://huggingface.co/microsoft/TRELLIS.2-4B/raw/main/pipeline.json>):

- `rembg_model: BiRefNet` with `model_name: "briaai/RMBG-2.0"`. RMBG-2.0
  (<https://huggingface.co/briaai/RMBG-2.0>): *"The model is released under a CC BY-NC 4.0 license
  for non-commercial use."* **The default pipeline therefore runs a non-commercial model.** The
  wrapper class loads any segmentation checkpoint by name; the upstream
  `ZhengPeng7/BiRefNet` is MIT (`gh api repos/ZhengPeng7/BiRefNet` → `MIT`), or the input can be
  supplied pre-masked so background removal never runs.
- `image_cond_model: DinoV3FeatureExtractor`, `facebook/dinov3-vitl16-pretrain-lvd1689m`, under the
  DINOv3 License (<https://ai.meta.com/resources/models-and-libraries/dinov3-license/>): a
  *"non-exclusive, worldwide, non-transferable and royalty-free limited license"*; it makes no
  claim on outputs and restricts trade-controlled and military end uses. Access is gated behind a
  Hugging Face account.

Verdict: **passes**, provided RMBG-2.0 is not in the run and the generation is recorded as `ARC-9`
provenance. **Hardware:** *"An NVIDIA GPU with at least 24GB of memory is necessary"* (repository
README). This machine is a Mac and cannot run it; a rented GPU costs money and an account.

**TripoSG** (`gh api` → `MIT`) and **Step1X-3D** (`gh api` → `Apache-2.0`): permissive open
weights; TripoSG produces shape only. Pass on code and weight licences; not evaluated further
because they share every structural limit above.

**Hunyuan3D 2.1 (Tencent, open weights).** Licence
<https://huggingface.co/tencent/Hunyuan3D-2.1/blob/main/LICENSE>: *"'Territory' shall mean the
worldwide territory, excluding the territory of the European Union, United Kingdom and South
Korea"*, and *"You must not use, reproduce, modify, distribute, or display the Tencent Hunyuan 3D
2.1 Works, Output or results of the Tencent Hunyuan 3D 2.1 Works outside the Territory."* Verdict:
**fails.** A public MIT repository distributes its contents into the EU and UK; the licence
forbids distributing the Output there, so no Hunyuan3D output can be committed, regardless of
*"Tencent claims no rights in Outputs You generate."*

**Stable Fast 3D / SPAR3D (Stability AI).** Community License
<https://stability.ai/community-license-agreement>: *"You own any outputs generated from the Models
or Derivative Works to the extent permitted by applicable law"*; licences terminate above *"USD
$1,000,000 in annual revenue"*; outputs may not be used *"to create or improve any foundational
generative AI model"*. Verdict: **passes narrowly** on ownership; not recommended: it shares every
structural limit above, its human quality was not evaluated, and the use restriction is one more
clause to carry.

**StdGEN** (`gh api repos/hyz317/StdGEN` → `Apache-2.0`) is the one generator that decomposes a
character into body, clothing and hair (<https://stdgen.github.io/>). It is trained for and
evaluated on **anime** characters (*"state-of-the-art performance in 3D anime character
generation"*, <https://arxiv.org/abs/2411.05738>). Not a fit for this reference; recorded because
decomposition is exactly what the fused generators lack, and a realistic successor would change
this assessment.

**A note on ownership claims for generated output.** The U.S. Copyright Office's January 2025
report holds that prompts to generally available systems *"do not provide sufficient human control
to make users of an AI system the authors of the output"* (summarised at
<https://newsroom.loc.gov/news/copyright-office-releases-part-2-of-artificial-intelligence-report/s/f3959c36-d616-498d-b8f9-67641fd18bab>).
So a vendor's "you own the output" is a contractual promise not to assert rights, over material that
may carry no copyright at all. For `DEP-8` that is acceptable — nothing obstructs placing it in an
MIT repository — but it means the MIT notice protects only the human contribution (the cleanup,
retopology, rig and texture work), and `ARC-9`'s provenance record is what keeps that honest.

### 4.2 Image-to-3D for parts, as shape targets (route E)

The one place a generator's strength meets the current route's weakness is **the shape of the
head.** The proposal, which is a Blender modifier stack and not reconstruction technology:

1. Crop the reference's head; generate a head-and-shoulders mesh with a `DEP-8`-passing generator
   (paid Meshy, or TRELLIS.2 with BiRefNet).
2. Align it to the CharMorph head; **Shrinkwrap** (project, with a mask excluding eyes, mouth
   interior and ears) the CharMorph head toward it, then smooth corrective. Topology, UVs, eye
   sockets, the 26 FACS blendshapes and the neck seam are kept, because the vertices only move.
3. Re-run the face-ratio measurements of `FIDELITY_REVIEW.md` §6 against it, under `ARC-23`'s
   locate-before-counting rule.

The same applies to the **backpack**, which is a rigid prop and suffers none of the fusion
problems: a generated olive canvas pack is a reasonable candidate as a separate object.

Do not use it for hair or garments: a generated hair shell is the opposite of what the hair needs.

**Risk:** the generated head may carry the generator's softening and be no closer to the reference
than the morphs; the wrap may fold in the eyelids. Three to seven agent-days, which is why it is an
experiment with a stop rule — keep it only if the per-row comparison improves.

### 4.3 Character Creator / Reallusion (route F)

Reallusion Software EULA, version dated 2026-05-25, Section 5 "Character Creator Base Model
License", <https://www.reallusion.com/Content/EULA/AP/EULA_AP.htm>:

> *"The unique topology and character rigs of CC Avatar that generated from Character Creator's
> Base Model are the property of Reallusion."* Prohibited: *"Use in any character generation
> system"*; *"Use for AI training, machine learning, or synthetic data generation purpose"*;
> *"Selling the 3D model created from Character Creator's Base Model as a content asset in any
> third-party marketplace, regardless of file format."*

Reallusion Content EULA, 2025-08-01, <https://www.reallusion.com/Content/EULA/EULA.htm>, §2.1(B)
(ii): *"You may NOT import, upload, reproduce, make available, publish, transmit, distribute, or
sublicense the Content to any third party."*

**Verdict: fails `DEP-8`.** Every Character Creator character carries Reallusion-owned topology
and rig; committing it under MIT would sublicense property Reallusion expressly keeps. This is the
same structure as MetaHuman in `UNREAL_ADAPTER_SPIKE.md` §9: free to embed in a shipped game, not
free to relicense. AccuRIG, Reallusion's free auto-rigger, runs under the same Content EULA and is
excluded for the same reason. Fidelity is not assessed further; it does not matter.

### 4.4 MPFB2 / MakeHuman (route G)

Licence: *"All core assets are shared under Creative Commons, CC0"*; *"The source code of MPFB is
shared under GPL"* (<https://static.makehumancommunity.org/about/license.html>). The asset-pack
index (<https://static.makehumancommunity.org/assets/assetpacks.html>) lists most packs CC0 and
some CC-BY (Hair 02, Hair 03, Shirts 02/03, Pants 02/03 and others); **CC-BY packs fail the
relicensing test** and must be filtered per asset, which `DEP-8` already requires.

**Quality ceiling.** MPFB2's body is the same kind of thing as CharMorph's: a realistic parametric
human driven by anatomical sliders. It cannot produce the reference's stylized face any more than
CharMorph can, for the same reason (§3). Its distinguishing assets are clothes and hair:

- `makehuman_system_assets` (CC0): hair `afro01, bob01, bob02, braid01, long01, ponytail01,
  short01–04`; clothing suits and shoes. **No updo, no hoodie.**
- `hair01` (CC0 per row, <https://static.makehumancommunity.org/assets/assetpacks/hair01.html>):
  26 community styles including `rehmanpolanski_hair_bun_brown` and
  `elvs_reverse_french_braid_bun`, each listed `CC0`. These are the only CC0 updo meshes found in
  this search. They are proxy-style meshes built for MakeHuman's real-time preview; expect a
  hairstyle *category* match and a quality below the reference's, and inspect before relying on
  them.

**Verdict:** passes `DEP-8` for the CC0 subset; **not a better route** — switching bodies costs a
new rig mapping, a new texture set and a new export pipeline to reach the same ceiling. The `hair01`
buns are worth downloading as a **reference or a starting groom shape** for route B or C.

### 4.5 VRoid / VRM (route H)

pixiv's VRoid Studio guidelines (<https://vroid.com/en/studio/guidelines>) returned **HTTP 403** and
were **not read first-hand.** As returned by search from that page: *"all content provided by pixiv,
including the base models when creating a new avatar, is not CC0"*; *"pixiv retains the copyrights
and other rights to the data pixiv provides and licenses said content so that you can use it for
as wide a range of purposes as possible"*; textures and hair meshes created from scratch belong to
the creator.

**Verdict: fails `DEP-8`.** Every VRoid export contains pixiv's base body and head mesh, which pixiv
licenses but retains; an MIT licence over it is a sublicense pixiv has not granted. Writing it as
"broad commercial use is allowed" would repeat exactly the redistribution-versus-relicensing error
the `DEP-8` amendment records.

**Style:** independently disqualifying. VRoid's head is an anime construction — large eyes drawn
into the texture, minimal nose geometry — rendered with MToon toon shading. It can be pushed toward
semi-realism with custom textures, but the face structure is the opposite direction from the
reference's soft-planed, sculpted face. The VRM humanoid bone set *is* `SkeletonProfileHumanoid`
(`HUMANOID_PROFILE.md` §1), so the rig would have been free — the only thing VRoid gets right.

### 4.6 Commissioned or hand-modelled character (route C)

**Scope to commission, on the existing rig:** head sculpt from the CharMorph head topology (keeping
UVs and blendshapes) toward the reference; skin albedo, roughness and freckles at 2K; a hair-card
groom of the messy updo with loose strands; the hoodie with draped folds, hood, cords, ribbing and
zip; jeans and backpack polish. Body, rig, tee graphic and pipeline stay ours.

**Cost and time.** Industry pricing guides put a game-ready stylized character at USD 1,000–5,000
and a professional game-ready character at USD 3,000–10,000, with hair grooming, facial work and
clothing named as the cost drivers (<https://www.chicmicstudios.in/blogs/how-much-does-3d-character-design-cost/>,
<https://vsquad.art/blog/how-much-does-a-3d-character-cost-vsquad>). Those are vendor marketing
pages, not quotes. Because the body, rig and pipeline already exist, the commissioned scope is
smaller than a full character: **USD 3,000–8,000, 3–6 weeks**, plus one to three review rounds.
This is an estimate, and the honest error bar is a factor of two either way.

**Licence.** Passes `DEP-8` **only through the contract**, which must state: work made for hire or
full assignment of copyright to the operator, a waiver or CC0 dedication so it can ship under MIT,
**no third-party inputs** other than this repository's CC0 assets (no Character Creator bases,
Marvelous Designer preset garments, purchased alphas or kitbash packs), no generative-AI inputs
without disclosure, and delivery of the source `.blend` and textures. Most freelance character work
is sold under a licence, not an assignment, so this clause is where the risk sits. A contract is
an operator decision; this document does not draft one.

**Fidelity.** The only route in this table where the face's stylization is in the hands of someone
who sculpts. It is also the route the reference implies: it is a render in the manner of a
modern AAA game character, and that look is produced by sculpting, grooming and draping by hand.

**Regeneration.** None — manual edits from then on. `ARC-19` already accepts this cost explicitly:
*"the artefact is shipped rather than re-derived"*.

### 4.7 Higher-quality CC0 hair and clothing sources

The search confirms `DEP-8`'s finding for characters in general: **no CC0 source ships a messy updo
or a hoodie at this quality.** What exists are CC0 *tools and materials* that let the parts be made
by the right method:

| Source | Licence, quoted | Use |
| --- | --- | --- |
| Blender Essentials hair node groups, bundled since 3.5 (`datafiles/assets/nodes/procedural_hair_node_assets.blend`) | *"We will only accept CC-0 licensed assets"* — Blender asset-bundle guidelines, <https://developer.blender.org/docs/features/asset_system/asset_bundles/guidelines/> | grooming an updo with hair curves: guides, interpolation, clump, frizz, curl; then curve → card ribbons with geometry nodes. The groom output is our own work |
| OpenGameArt "Hair Alphas For Days" by OwlishMedia, 85 strand alphas at 2048² | listed **CC0**, <https://opengameart.org/content/hair-alphas-for-days> | the strand atlas for the cards; record per asset in `LICENSES/` |
| MakeHuman `hair01` buns | **CC0** per row (§4.4) | a category-correct reference shape or starting point |
| GarmentCode (ETH), parametric sewing patterns | MIT, <https://github.com/maria-korosteleva/GarmentCode> | a hoodie pattern to drape with Blender cloth; Blender's own cloth sim is enough on its own for one garment |
| Blender Studio Human Base Meshes | CC0 (already approved in `DEP-8`) | a sculpt base for a stylized head if route C prefers it to CharMorph's |
| Blender Studio character library (Sprite Fright, Charge, Spring) | *"CC-BY"* with credit *"© Blender Foundation"*, <https://studio.blender.org/characters/> | **fails relicensing** — CC-BY content cannot be placed under MIT. Study only |
| Meshy gallery "pre-made Hair assets" ("Brown Bun Updo with Face-Framing Strands") | the tag page says *"released under a CC0 … license"*, <https://www.meshy.ai/tags/hair>, which conflicts with Meshy's Terms (free output Meshy-owned, CC BY) | needs a per-asset check and an account to download; a generated solid shell in any case. Not recommended |

**The structural point, for routes B and C alike:** the hoodie must be **draped**, not offset. A
cloth simulation of a simple pattern against the posed body, frozen and skinned with weight
transfer, produces the folds, the hanging panels and the bunched hood that §3 lists as missing. The
current `garments.py` comment explains why offsetting was chosen — exact fit and free skin weights —
and those are real benefits, but they are the reason it reads padded. Weight transfer from the body
recovers most of the benefit.

---

## 5. Ranked recommendation

1. **Fix the three authoring methods on the existing rig** (route B), now, at no cost: groom the
   updo from hair curves with the CC0 node groups and bake to cards with CC0 alphas; drape the
   hoodie with cloth simulation; do the skin material pass. Expected to close the padded hoodie,
   the hood loft and the hair silhouette. Not expected to close the face.
2. **In parallel, put the face to the operator as a decision** between a commission (route C;
   money, highest confidence) and the wrap experiment (route E; needs a paid Meshy account or a
   rented NVIDIA GPU; lower confidence).
3. **Do not switch bases.** Character Creator, VRoid and Hunyuan3D are excluded by licence; MPFB2
   is a sideways move; a whole-character generation discards what passes.

---

## 6. What was tried

The brief asked for a cheap legal test of any free tier that needs no operator account. **None was
possible, and nothing was generated:**

- **Every hosted free tier** (Meshy, Tripo, Rodin) requires an account to generate or download, and
  its output would in any case be vendor-owned or CC BY, so it could not have been committed.
- **TRELLIS.2 locally** needs an NVIDIA GPU with 24 GB; this machine is a Mac.
- **TRELLIS.2's Hugging Face Space** (<https://huggingface.co/spaces/microsoft/TRELLIS.2>) could in
  principle be used anonymously. The available browser automation timed out on every call this
  session, and this spike's permissions excluded scripted HTTP clients, so the Space was not
  reached. Note for whoever tries it: the Space runs the default pipeline, which includes the
  non-commercial RMBG-2.0 unless the input already has an alpha channel. Treat a Space result as
  evidence about fidelity only, never as a committable asset.

**This is the largest uncertainty in this document.** Route E's value and route D's predicted
failures are judgement from how these models behave on other portraits, not a run on this one.

---

## 7. Where this assessment could be wrong

- **A generator may do better on this reference than predicted.** Stylized-realistic renders are
  close to the training distribution of game-asset generators, and Tripo and Meshy both offer
  part-segmented and quad-remeshed modes not tested here. One paid run on the face crop would settle
  route E in an afternoon.
- **An agent may groom better than expected.** Route B's ceiling is set by the dexterity of
  scripted grooming, which has not been tried; the current `hair.py` is not a groom.
- **The commission estimate** is from marketing pages, not quotes; a single real quote would replace
  it.
- **The Tripo and VRoid quotes** were not read first-hand (HTTP 403). The VRoid verdict does not
  depend on the exact wording — any licence that is "not CC0" with rights "retained" fails the
  relicensing test — but the Tripo paid-tier verdict does.
- **"Stiff"** is partly animation. If the pose and idle work lands well, the operator's verdict on
  the next frames may weigh the face and hair differently.

---

## 8. Side findings

- **`manifest.yaml` contradicts `ARC-19`.** `presentation/mineworld-default/3D/manifest.yaml` still
  lists `04_character_closeup.png` as `not_authoritative_for: [facial_fidelity, skin_rendering,
  hair_simulation, character_asset_budget]`, and still comments that *"MineWorld Default 3D does
  not target photorealistic human rendering"* with `facial_detail: medium`. `ARC-19` consequence 1
  says that list *"drops `facial_detail`, `skin_rendering` and `hair_simulation` for the default
  character"*. That is a recorded decision not applied to the machine-readable file — the defect
  `CLAUDE.md` §2.1(4) names. Not fixed here; this spike edits only this document.
- **TRELLIS.2's default pipeline carries a non-commercial model** (RMBG-2.0, CC BY-NC 4.0) under an
  MIT-labelled repository. Any future use must swap or bypass it and record that it did.
- **Generator licences that restrict training on outputs** (Hunyuan3D, Stability, Rodin) bind the
  party that generated the output. Recorded because MineWorld's purpose includes LM-driven
  characters, the same concern `DEP-8` raises about MetaHuman §6(e).
