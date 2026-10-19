## DEP-8 — Asset sourcing and licence rules

**Date** 2026-09-26 · **Applies to** every pack MineWorld publishes or bundles

MineWorld is MIT and redistributes what it ships, so "free to use" is not the test —
**"free to redistribute"** is. Several well-known sources fail that test while looking fine.

**Amended 2026-09-27 — relicensing, not distribution, is the binding test.** The Unreal spike
went looking for a distribution clause to quote and found a broader rule that makes the
clause-by-clause search unnecessary. Epic's grant is *"non-exclusive, non-transferable,
**non-sublicensable**"* and for *"private"* use (Unreal Engine EULA §2), and the Content EULA §6
declares any inconsistent sublicense *"null and void"*. **An MIT `LICENSE` file covering a
vendor-authored asset is a sublicense that vendor never granted.**

That holds even where distribution is expressly *allowed*: the Unreal templates may be shipped in
source form to any third party under §5(b), and they still may not be relicensed. So the rule for
this repository is simpler than reading each agreement — **nothing Epic authored enters it,
whatever its distribution permission says**: not MetaHuman on either of its two licence paths, not
Quixel Megascans on any tier including Personal, not Fab Standard License content, not the Third
Person mannequin. The one door that stays open is a Fab §2(c) listing a third-party publisher has
marked open source, which is the same per-asset check this decision already prescribes for
Sketchfab.

The generalisation is what matters: **ask whether we may relicense, not whether we may
redistribute.** A permission to distribute is not a permission to place under MIT, and a project
whose whole licence story is "MIT end to end" needs the second one. Details and the verbatim
clauses are in [`references/UNREAL_ADAPTER_SPIKE.md`](references/UNREAL_ADAPTER_SPIKE.md) §9.

**A second, independent problem** if an Unreal client is ever adopted: Unreal Engine EULA §6(e)
bars using *"MetaHuman digital characters and animation curves … to build or enhance any database
or training or testing any artificial intelligence"*. For a project whose purpose is LM-driven
characters with recorded cognition fixtures, that constrains use even where redistribution is not
at issue, and it needs its own decision at that point rather than being discovered later.

**Amended 2026-10-08 — the font exception (operator ruling QSET-1, S20).** DEP-8's binding test is
whether MineWorld may relicense an asset under MIT. Fonts under the SIL Open Font License 1.1
cannot pass it, because OFL-1.1 requires the font to stay under OFL. They are the only exception to
the test, under every one of these conditions:

1. **Fonts only.** The exception covers font software (`.otf`, `.ttf`, `.otc`, `.woff2`) licensed
   under `OFL-1.1` and nothing else: no texture, model, sound, code or data file, and no other
   licence.
2. **Bundled, never sold on its own.** The font ships only inside MineWorld, as a file the clients
   load, and is never offered, sold or distributed as a product on its own (OFL-1.1 §1).
3. **Unmodified and named as upstream.** The file is the upstream release, byte for byte. Its
   source URL, version and SHA-256 are recorded beside it. A modified or subset font is not covered
   without a new decision, which would also have to respect any Reserved Font Name (OFL-1.1 §3).
4. **Its licence travels with it.** `OFL.txt`, with the upstream copyright notice, sits in the same
   folder (OFL-1.1 §2). `NOTICE` names the font, its copyright holder and its licence, and states
   that MineWorld's MIT licence does not cover it.
5. **Never merged into MIT material.** The font is never embedded into, or concatenated with, a
   file under MIT. Loading it at runtime is use, not merging.
6. **Listed.** Every font admitted under this exception is a row of DEP-8's table. At present
   there is one: **Noto Sans SC Regular** (`notofonts/noto-cjk`,
   `Sans/SubsetOTF/SC/NotoSansSC-Regular.otf`, 8,331,336 bytes, OFL-1.1, copyright Adobe with
   Reserved Font Name "Source"), the CJK fallback font of both reference clients' UI (S20 SET-a).

ARC-55's default pack allow-list is unchanged. A **pack** that carries an OFL font still needs a
world-level policy that allows `OFL-1.1` (ARC-55 point 5). This exception covers fonts bundled with
MineWorld's own clients.

### Approved, licences confirmed from primary sources

| Source | Licence | For |
| --- | --- | --- |
| [Poly Haven](https://polyhaven.com) | CC0 1.0, redistribution explicit | models, textures, HDRIs; ships glTF at real metric scale, so Godot needs no conversion |
| [ambientCG](https://ambientcg.com) | CC0 1.0, raw files may ship in a game | the PBR material library, and the main lever for visual coherence |
| [MakeHuman / MPFB2](https://static.makehumancommunity.org) | core assets CC0 | character meshes. **The code is AGPL/GPL and the meshes are CC0** — we ship meshes only. Community-repository clothes and hair carry per-asset licences and need individual checking |
| Blender Studio Human Base Meshes | CC0 | retarget skeleton and scale yardstick |
| [CMU Motion Capture](http://mocap.cs.cmu.edu) | free for any use including commercial | animation. One restriction: the data may not be *resold* as data; embedding it is use, not resale |
| [Quality Godot First Person Controller v2](https://github.com/ColormaticStudios/quality-godot-first-person-2) | MIT | the controller. v1 is archived — take v2 |
| [Sky3D](https://github.com/TokisanGames/Sky3D) | MIT | sky and daylight. Credit is required only if the bundled star map ships |
| [Kenney](https://kenney.nl) | CC0 | blockout and placeholder only; the style is deliberately not ours |
| [NOAA GHCN-Daily](https://www.ncei.noaa.gov/pub/data/ghcn/daily/), station USW00023188 | CC0-1.0 / US public domain ("no restrictions on the use of the data", [NODD](https://registry.opendata.aws/noaa-ghcn/)) | Market Town's daily weather record (`DEP-31`). Attribution and both requested citations in the `NOTICE` beside the file; no implied endorsement; modified data so labelled |
| [Noto Sans SC Regular](https://github.com/notofonts/noto-cjk) (`notofonts/noto-cjk`, `Sans/SubsetOTF/SC/NotoSansSC-Regular.otf`, commit `165c01b`, SHA-256 `faa6c9df…706d5ea9`) | **OFL-1.1**, admitted only under the font exception above | the CJK fallback font of both reference clients' UI (`clients/shared/settings/fonts/`, S20 SET-a, `DEP-36`) |

### Excluded, with the reason

- **Mixamo** — **MineWorld project policy, not a statement about Adobe's terms.** Adobe's FAQ says Mixamo characters and animations may be used royalty-free in personal, commercial and non-profit projects including games. What it does *not* state in terms we have read at first hand is whether committing the raw FBX into a public repository counts as permitted use or as redistributing the asset itself. Because MineWorld redistributes everything it commits, we resolve that ambiguity conservatively: **no Mixamo file is committed.** Usable as local scaffolding. If someone later finds Adobe language that settles it either way, this entry should be updated with the citation rather than left as caution hardened into fact.
- **SMPL, SMPL-X, AMASS, 100STYLE** — non-commercial academic. This is the trap sitting directly beside CMU, which is fine: CMU itself is free, its research reprocessings are not.
- **Synty, Unity and Unreal marketplace packs, CGTrader and TurboSquid "free", Renderpeople and AXYZ samples** — per-seat or explicitly no-redistribution.
- **Vecteezy, Freepik** — require attribution *and* forbid redistributing the file. Not open.
- **MB-Lab** — licence of generated characters unconfirmed from a primary source. Marked unconfirmed rather than assumed; prefer MPFB2.
- **CC-BY-SA and GPL assets** — copyleft incompatible with MIT redistribution, whatever their quality.
- **The Tencent Hunyuan family under Tencent's community licences: code, weights and outputs.**
  This covers Hunyuan3D, HunyuanWorld 1.0, HunyuanWorld-Mirror, HY-World 2.0 (including the
  WorldStereo 2.0 weights) and HunyuanImage. Operator decision, 2026-10-08. The licences fail the
  relicensing test on three counts:
  - they exclude the EU, the UK and South Korea, including use of Output there (§5(c));
  - every downstream licence must carry Tencent's use restrictions as enforceable terms, which
    MIT cannot (§5(a));
  - outputs may not be used to improve other AI models (§5(b)).

  The operator also chose **no private look-development use**, so that no output or derivative can
  leak into the repository. Re-evaluate only if Tencent relicenses a component under a permissive
  licence. Evidence, read at source on 2026-10-08:
  [`docs/references/HY_WORLD_2_COMPARISON.md`](references/HY_WORLD_2_COMPARISON.md).

**Generated meshes, recorded 2026-10-06 (route E experiment, read at source).** Meshy's paid
plan passes the relicensing test: *"such customers on a paid Meshy plan own their Customer
Output"* (Terms of Use, last updated 2026-09-19, <https://www.meshy.ai/terms-of-use>); ownership
survives cancellation per the help centre (*"You'll retain the rights to the models you created
while you were a subscriber, and they will remain private indefinitely"*,
<https://help.meshy.ai/en/articles/9992023-if-i-cancel-my-subscription-will-all-my-models-revert-to-a-cc-by-4-0-license>;
the Terms themselves are silent on cancellation); output is private by option and becomes CC0 if
posted to the Meshy Community. A committed Meshy-derived asset keeps Meshy's AI identifiers (the
Terms forbid removing them) and carries `ARC-9` provenance. The free plan fails. TripoSR and
TripoSG (MIT code and weights) pass, provided their bundled background removers (`rembg`/u2net
defaults, and `briaai/RMBG-1.4`, non-commercial) are not run; BiRefNet (MIT) is the substitute.
Details and quotes: [`references/CHARACTER_ROUTE_E_EXPERIMENT.md`](references/CHARACTER_ROUTE_E_EXPERIMENT.md) §§2, 9.4.

**Generated images (OpenAI), recorded 2026-10-08 (pre-publication audit, read at source).**
OpenAI image output passes the relicensing test under both agreements that can apply. Read
first-hand on 2026-10-08 in a browser (an automated fetcher receives HTTP 403 from these pages,
which is why the route E experiment could only quote a search result):

- **ChatGPT** (consumer) — Terms of Use, effective 2026-01-01,
  <https://openai.com/policies/terms-of-use/>: *"Ownership of content. As between you and OpenAI,
  and to the extent permitted by applicable law, you (a) retain your ownership rights in Input
  and (b) own the Output. We hereby assign to you all our right, title, and interest, if any, in
  and to Output."*
- **The API (`gpt-image-1`, the `tools/asset_generation` models) and ChatGPT Business /
  Enterprise** — OpenAI Services Agreement, effective 2026-01-01 (page updated 2025-12-01),
  <https://openai.com/policies/services-agreement/>, §4.1: *"As between Customer and OpenAI, to
  the extent permitted by applicable law, Customer: (a) retains all ownership rights in Input;
  and (b) owns all Output. OpenAI hereby assigns to Customer all OpenAI's right, title, and
  interest, if any, in and to Output."* The Agreement *"only applies to use of OpenAI's APIs,
  ChatGPT Enterprise, ChatGPT Business, ChatGPT for Clinicians, and other services for customers
  who are businesses and developers"*.

Both versions were in force from 2026-01-01 through the read date, so they are the terms that
governed every OpenAI image in this repository (generated 2026-09-26 onwards). The ChatGPT
account tier is not recorded in the images' metadata; the conclusion does not depend on it,
because both agreements assign Output to the user. As the owner, MineWorld distributes these
images under the repository's licence. Neither agreement restricts redistribution or publication
of Output. What they do require, and how MineWorld meets it:

- *"Represent that Output was human-generated when it was not"* is prohibited (Terms of Use,
  "What you cannot do"). Every OpenAI-derived asset is recorded as AI-generated in its
  provenance record (`ARC-9`), and the top-level `NOTICE` says so.
- *"Use Output to develop models that compete with OpenAI"* (Terms of Use) and *"use Output to
  develop artificial intelligence models that compete with OpenAI's products and services"*
  (Services Agreement §3.3(e)) are prohibited. The restriction binds the account holder;
  MineWorld trains no model on these images.
- The Sharing & Publication Policy (updated 2022-11-14,
  <https://openai.com/policies/sharing-publication-policy/>; binding under the Terms of Use's
  "What you can do" and an "OpenAI Policy" under the Services Agreement) asks that shared content be attributed to the publisher and that one
  *"Indicate that the content is AI-generated in a way no user could reasonably miss or
  misunderstand."* The provenance records and `NOTICE` do both.
- *"Output may not be unique and other users may receive similar output"* (Terms of Use;
  Services Agreement §4.4). Ownership is of our Output only; MineWorld claims nothing in anyone
  else's similar image.
- No clause in either agreement, nor in the Usage Policies (effective 2025-10-29) or the Service
  Terms (updated 2026-09-29), requires an identifier to be kept. The ChatGPT images carry a
  signed C2PA manifest (`claim_generator` "OpenAI Media Service API", software agent "ChatGPT" /
  "gpt-image", source type `trainedAlgorithmicMedia`); MineWorld keeps it in every committed
  original anyway, matching the Meshy rule. Re-encoded derivatives (JPEG crops, the 768 px texture
  sources) lose it, and their provenance is carried by the written record instead.

The records this entry backs: `presentation/mineworld-default/{2D,3D}/references/PROVENANCE.md`
(the ten ChatGPT reference images and their crops), `clients/3d-spike/ASSETS.md` (the two
`gpt-image-1` texture sources) and the `.provenance.yaml` sidecars of the generated 2D candidates.

Per-asset sources — Sketchfab's CC0 filter, OpenGameArt, BlenderKit's free tier — mix licences
within one site, so they are usable only with a per-asset check recorded in that pack's
`LICENSES/`.

### Two gaps that are not a shopping problem

**No semi-realistic modular building kit exists under an open licence** — every CC0 kit is
low-poly or PSX. The matching route is box geometry at correct proportions dressed with CC0 brick,
plaster, painted wood and roofing. Recorded so no future contributor repeats the search and
concludes they searched badly.

~~**No CC0 library ships clothed, rigged, realistically proportioned modern people** either; that
is a pipeline to build, not a product to find.~~ **Corrected 2026-09-27 — this was false.** One
does: the **CharMorph "Vitruvian"** character, distributed inside
[`ibrews/VitruvianGodot`](https://github.com/ibrews/VitruvianGodot). It is clothed, rigged,
realistically proportioned, and CC0. It is in the tree at
`clients/3d-spike/assets/characters/vitruvian/` and the capsule mannequin it replaced is gone.

The correction carries a licence trap worth stating here, because the surface reading is alarming
and wrong. Upstream ships six Mixamo-derived animation clips, and the character's skeleton is
named `mixamorig:*` throughout — which looks like Mixamo output and would fall under the exclusion
below. It is not. `config.yaml` inside CharMorph-Vitruvian declares its own armature preset
(`mixamo: {title: "Mixamo (Game-Ready)", obj_name: mixamo_vitruvian, weights: weights/Mixamo.npz}`)
under `license: CC0`; the rig is built by CharMorph's own operator with no Mixamo file involved;
and the retarget reconciles the pose mismatch in the other direction, never writing the rest pose.
**The `mixamorig:` prefix is a naming convention for clip compatibility, not authorship.** The
animations and the source FBX are excluded and are not in the tree; the character is not.

Full evidence, file by file, in [`CHARACTER_ASSET_AUDIT.md`](CHARACTER_ASSET_AUDIT.md). One gap is
stated there rather than smoothed over: the CharMorph data repositories carry no `LICENSE` file,
so CC0 rests on a machine-readable field in the shipped data, a documented relicensing permission,
and a CC0 upstream (*Antonia Polygon*).

### The coherence procedure, in order

The failure mode is not bad assets. It is good assets that disagree, along four axes, and the
order matters:

1. **Scale, mechanically.** One unit = one metre, a permanent 1.7 m reference in the scene,
   re-export anything that disagrees rather than scaling it in the node tree — a non-uniform
   import scale quietly breaks physics, shadow bias and LOD distances.
2. **Take material authority away from the asset authors.** Re-point every albedo at the
   project's own material set; roughness ~0.3–0.8 for painted wood, brick and stucco; albedo
   luminance ~0.2–0.7 sRGB. This is what removes the "this came from somewhere else" feeling.
3. **Build the lighting rig before importing anything and never judge an asset outside it** — one
   HDRI or one sky config, one sun, one tonemap, one exposure. Then a wrong asset is rejected on
   sight instead of patched later.
4. **Texel density target** (~512 px/m for a walkable street); downsample higher tiers to it.

And keep sourcing narrow on purpose: two libraries used consistently read as one town, while
eight individually more beautiful ones do not. That restates
[`ART_DIRECTION.md`](ART_DIRECTION.md) §18 as a procedure.
