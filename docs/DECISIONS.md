# MineWorld decision log

**Status:** durable, project-level decisions
**Audience:** coding agents and contributors.

This log holds decisions that outlive any single effort — dependencies, build-versus-adopt
calls, and architectural routes. It exists so that a future contributor does not replace or
duplicate something without knowing why it is there
([`REUSE_POLICY.md`](REUSE_POLICY.md) §§11–12).

Effort-scoped planning decisions (`D-1`, `D-2`, …) stay in that effort's plan under
`.structured-coding/plans/<effort>/`. Decisions here are referenced by id (`DEP-n`, `ARC-n`) and
are never silently reversed: superseding one means adding a new record that says so.

Record format is deliberately short: problem, options, choice, why not ourselves (or why not
them), the interface that isolates it, and the limitation we accepted.

---

## ARC-1 — A Presentation Style Pack is not a sixth pack type

**Date** 2026-09-25

[`ART_DIRECTION.md`](ART_DIRECTION.md) §12 introduces the "Presentation Style Pack" as a
first-class concept, while [`MODULE_SPEC.md`](MODULE_SPEC.md) §1 states that exactly five pack
types exist and that adding a sixth is a deliberate design change.

**Resolved as: the same type, now with a specified structure.** A Presentation Style Pack is the
internal layout of the Presentation Pack that `MODULE_SPEC.md` §6 already defines — manifest,
references, art direction, asset bindings, material rules, character style, lighting profile,
renderer configs. The name is used where the art-direction aspect is what matters.

**Why not a sixth type.** Nothing in §12 describes a different *kind* of module: it does not
add entities, rules, controllers, or world content, and it is installed and swapped exactly as a
Presentation Pack is. Adding a type to the core ontology to describe a directory layout would be
the ontology bloat the project explicitly guards against
([`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §1).

**Recorded because** a future contributor reading §12 alone would reasonably conclude the
ontology has six types. It has five.

---

## ARC-2 — The style manifest is generated from references, which constrains the schema

**Date** 2026-09-25

Style authoring is meant to be: upload 4–10 references, write two or three sentences, choose 2D or
3D, and let the system generate an initial manifest
([`ART_DIRECTION.md`](ART_DIRECTION.md) §23).

**Consequence, and the reason this is a decision rather than a feature note:** every manifest
field must be *inferable from images*, *chosen by the creator*, or *fixed by project policy*.
Auditing the §15 example against that rule already separates three kinds of field, and exposes
that `readability_priority` and `human_scale` were never style at all — gameplay readability
outranks decorative density for every MineWorld style, so they are project invariants a style
restates rather than choices a style makes.

**Applied as a rule:** a proposed manifest field that is none of the three is rejected. This is
what keeps §15's "do not overengineer an exhaustive universal art ontology" enforceable instead of
aspirational.

**Limitation:** the analysis reads style only. It may never infer simulation content from a
reference image — a café with outdoor seating says nothing about whether `Sit` exists in a world.

---

## ARC-3 — A style that spans dimensions is several packs, not one

**Date** 2026-09-25

The default style now has both 2D and 3D references. `ART_DIRECTION.md` §15 gives `dimension` as
a list, which suggests one pack could declare `[2d, 3d]`.

**Decided: one family directory, one pack per dimension.** `presentation/mineworld-default/`
contains `2D/` and `3D/`, each with its own manifest, art direction and references.

**Why, from the references rather than from taste.** The 3D references use realistic human
proportions; the 2D references are stylized. A single manifest would have to state one value for
`characters.proportions`, and whichever it stated would be false for half of its own reference
images. The same applies to `materials.physically_based` and `stylization`. A manifest that
contradicts its references is worse than no manifest, because the references are the source of
truth (§16) and the manifest is supposed to summarise them.

**What the family directory carries** is the part that genuinely is shared: the mood, and the
rule that both packs express identical semantics. `dimension` stays a list in the schema — a
style whose values really are identical across dimensions may still declare both.

**Consequence for the creator flow (§23):** choosing 2D or 3D selects which pack is being
authored. A creator who wants both answers the question twice, with their own references each
time, which is also the honest thing to ask of them.

---

## ARC-4 — A reference image is authoritative only within designated dimensions

**Date** 2026-09-25 · **Refines** `ART_DIRECTION.md` §16

`ART_DIRECTION.md` §16 makes reference images the strongest definition of a style, above prose and
above prompts. Taken literally, every property of every reference becomes normative — and the
first real conflict arrived immediately: `3D/references/04_character_closeup.png` renders a face
far closer to photorealism than §§3 and 7 allow, those sections having ruled out cinematic facial
rendering and called simplification desirable.

**Decided: each reference declares what it is a reference *for*.** `manifest.yaml` carries
`authoritative_for` (and optionally `not_authoritative_for`) per image. Within those dimensions the
image outranks prose. Outside them it carries no authority.

For `04_character_closeup.png`: authoritative for body proportions, casual modern clothing,
character-environment integration, close-encounter camera distance and ordinary-person identity;
**not** authoritative for facial fidelity, skin rendering or hair simulation.

**The default 3D character target stands unchanged:** realistic proportions with moderately
simplified face and materials, medium detail. No photoreal skin, no MetaHuman-level assets, no
facial scanning, no cinematic hair, no bespoke character pipeline.

**Why this is architectural, not aesthetic.** A photoreal face standard would put character
production beyond what community creators can afford, which contradicts the reason MineWorld has a
default style at all — that a creator should not need a professional 3D art team
(`ART_DIRECTION.md` §10). The cost of a style decision is part of the decision.

**Why scoping beats deleting the image.** The image is genuinely valuable for scale, clothing,
lived-in feel and camera framing. Removing it to resolve one conflicting dimension would discard
five useful ones. Scoping authority also makes the reference system more robust in general: a
future contributor adding a reference for lighting is not implicitly asserting a character
standard.

**Limitation:** `authoritative_for` is creator-stated, so a mis-scoped reference misleads exactly
as prose would. It is a small, honest schema addition (`ARC-2`), not a guarantee.

---

## ARC-5 — The repository is remote, and `main` is protected

**Date** 2026-09-26 · **Supersedes** the local-only half of effort decision `D-9`

`git@github.com:yuema137/MineWorld.git` exists. `main` and the in-flight branch are pushed, and
branch protection is being enabled, so from here every change reaches `main` through a pull
request.

**What changes.** Branch push and PR creation are authorized for implementation sessions. Remote
CI becomes possible before S13, which had assumed it would create the repository.

**What does not.** Merging remains the operator's decision. This is the one authority the
autonomous-execution grant never included, and protection now enforces it mechanically instead of
relying on an agent honouring a line in a contract — which is a strictly better place for it to
live.

**One command from the operator's setup was deliberately not run.** The given sequence ended
`git branch -M main`, which assumes a fresh repository whose current branch needs renaming. Here
`main` already existed and the checked-out branch was a feature branch, so that command would have
renamed the feature branch to `main` and clobbered the merged history. Recorded because the next
person setting up a mirror will meet the same instructions.

---

## ARC-6 — MineWorld ships as a runtime, tools and reference clients

**Date** 2026-09-26

MineWorld is not an application. It is an installable world runtime plus developer tools plus
reference clients — Godot, a Minecraft server and ROS are closer analogies than a game
executable. Infrastructure (server, kernel, systems, SDK) is the project; the clients are what
ship to demonstrate it, and a creator may replace every one of them.

**What this fixes about MVP-0.** The deliverable is a set of artefacts a person can run:
`mineworld server <world>`, `mineworld-2d`, `mineworld-3d`, a `create / validate / run / inspect`
CLI, and two sample World Packs. Not a video, and not one executable.

**What it rules out of MVP-0.** Launcher, world-editor GUI, MineWorld Studio. Those are the
product surface over the runtime, and building a shell before the runtime is proven means
building the shell of something that does not exist. Recorded so the temptation is answered once
rather than each time it recurs.

**The criterion it adds** is `AC-15` in [`MVP.md`](MVP.md) §9: a 2D client, a 3D client and an
agent on **one running server**, where something done in one window is carried forward by an NPC
met in the other. `AC-13` proves the clients ask the same question; `AC-15` proves they inhabit
the same world rather than two consistent copies of it. That difference is the whole claim of the
project, and it is also the fastest way to show a person what MineWorld is.

**Open, decided when the clients are built:** whether `mineworld-2d` and `mineworld-3d` are
exported native binaries or thin launchers over an installed Godot. Export templates are a
heavyweight dependency, and the criterion is that a person can run one command and play — not
which packaging achieves it.

---

## ARC-7 — Asset Pack is a sixth pack type

**Date** 2026-09-26 · **Extends** `MODULE_SPEC.md` §1 · **Relates to** `ARC-1`

`ARC-1` refused to make a Presentation Style Pack a sixth type. This one is accepted, and the
difference is worth stating because it is the test any future seventh must pass.

A Presentation Style Pack described the **internal layout of an existing type**: it added no new
kind of thing, installed identically, and named a directory structure. An Asset Pack is a
different kind of thing: separately authored, separately licensed, separately versioned, and
**reused by packs that share no style**. The same bench, tree and café model serve a
semi-realistic world and an illustrated one. Folding them into Presentation Pack would force one
copy of every model per style, which is the concrete cost that justifies the concept.

**Rule extracted for next time:** a new pack type is justified by a distinct *lifecycle* —
authored, licensed, versioned and reused independently — not by a distinct folder.

---

## ARC-8 — Extension tiers, and why simulation code is WASM

**Date** 2026-09-26 · **Specified in** [`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md) §6

```text
Tier 0  declarative content            YAML, JSON, media        nothing to sandbox
Tier 1  simulation extension           WASM Component + WIT     capability-based sandbox
Tier 2  renderer extension             Godot addon / GDScript   engine-level, renderer-scoped
```

**Most packs are Tier 0 and must stay that way.** A hot-spring town is a World Pack plus Asset
Packs plus configuration; wanting a new place, people and look must never require a compiler.
Code is for adding a law of the world — farming, with `Plant`, `Crop`, `Soil`, `Grow`, `Water`,
`Harvest` — and that is a System Pack.

**Why Tier 1 is not the engine's scripting language.** Engine neutrality is the stated reason and
the weaker one. The serious reason is safety: a public server downloads packs from strangers, and
arbitrary native, Python or GDScript code in that position can read the filesystem, open sockets
and exfiltrate credentials. Review policy does not fix that; a capability sandbox does. A system
gets `world.read.entities`, `world.emit.events`, `world.query.time` and no filesystem, network or
environment.

**Why the bet is safe now.** Verified rather than assumed: the Component Model ships in real
tooling, WASI 0.2 is stable with 0.3 adding native async, WIT generates typed bindings across
languages, and plugin systems are precisely the workload class for which server-side WASM is
considered production-ready in 2026. This also matches what `ARCHITECTURE.md` §12 already
planned, so it is a commitment to an existing direction rather than a new one.

**Tier 2 may decide how water looks. It may not decide whether Alice can afford a boat.**

---

## DEP-7 — glTF 2.0 as the canonical 3D interchange format

**Date** 2026-09-26

**Problem.** One published 3D format that carries mesh, materials, textures, skinning and
animation, that Godot imports natively, and that does not bind MineWorld to an engine.

**Options.** glTF 2.0 / `.glb`; engine-native scenes (`.tscn`, `.uasset`); `.fbx`; `.blend`;
USD.

**Choice: glTF 2.0, `.glb` preferred.** Khronos designed it precisely as an API-neutral runtime
delivery format, Godot recommends it as its import path, and Blender, Unreal and web renderers
read it. `.glb` keeps a pack to one binary per model.

**Why not the others.** An engine-native scene makes the published ecosystem a Godot ecosystem,
which contradicts the project's central claim. `.fbx` is proprietary and historically
inconsistent across exporters. `.blend` is an authoring format, not a delivery one. USD is
powerful and the right answer for a large studio pipeline; it is disproportionate here and far
less well supported on the runtime side we need.

**Isolation.** Renderer adapters import the canonical form. Whatever a runtime compiles it into —
a Godot `.pck`, say — is a build output and never something an author publishes.

**Limitation.** glTF has no concept of an interaction anchor or a semantic space, which is why
`asset.yaml` sits beside the model rather than inside it.

---

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

---

## ARC-9 — Generated assets are first-class candidates, never automatic content

**Date** 2026-09-26

MineWorld does not have to source its look from existing asset libraries, and for the default
style it largely cannot: the research behind `DEP-8` found no open-licensed semi-realistic
building kit, no CC0 clothed and rigged modern people, and nothing illustrated that matches the
2D references. Bending the art direction to fit what happens to exist would be the wrong
trade — the direction is the thing being protected.

So generation is accepted and encouraged where it reaches the intended style faster. The rule
that makes it safe is the pipeline, not the source:

```text
generate candidates  →  human selection  →  cleanup and normalization  →  packaged asset
```

**A generated output is a candidate. It becomes an asset when a person accepts it.** That
selection step is not ceremony: `DEP-8`'s coherence procedure — one scale, one material
authority, one lighting rig — is exactly what a pile of individually plausible generated objects
will otherwise fail.

### Where generation fits, by medium

2D takes it well: scenes, buildings, ground and wall textures, vegetation, background character
looks, UI and icons. 3D takes it unevenly: style references, PBR textures, props and building
concepts are productive; meshes usually need cleanup; **rigged characters and reusable animation
are where one-shot generation does not yet land**, which is why `DEP-8` records a character
pipeline rather than a shopping list.

### Provenance is the price

Anything entering the repository, a default pack, or anything publishable carries its origin:

```yaml
source_type: generated
generation:
  tool: <tool>
  model: <model and version>
  date: 2026-09-26
  prompt_summary: "semi-realistic modern outdoor bench…"
  human_curated: true
postprocess: { cleaned: true, retopology: false, texture_adjusted: true }
license: { redistribution_allowed: true }
```

The question is never *"was this AI-generated?"* It is **"do this asset's terms permit
commercial use, modification, redistribution, and bundling into an open-source project?"** —
which is the same question `DEP-8` asks of a downloaded asset, and the same ambiguity the Mixamo
entry above resolves conservatively.

### Two layers, deliberately different standards

| Layer | Examples | Standard |
| --- | --- | --- |
| **Official / shareable** | this repository, default packs, published DLC, a future registry | redistributable, modifiable, bundleable, provenance recorded |
| **Private / local** | a player's own world, a private server, an unpublished mod | the player's own business |

MineWorld should not police what someone generates for their own machine. It must be strict
about what it hands to someone else.

### Current practical constraint, stated so it is not mistaken for policy

No image or model generation tool is wired into the build agents, so today generated assets enter
through the operator, who produced the existing reference images that way. The pipeline above is
the standing policy; the operator is currently its generation step.

### Roadmap

```text
MVP-0   generated reference images and textures; placeholders elsewhere
MVP-1   provenance fields and asset validation in the pack format
MVP-2   reference-image-first authoring: upload 4-8 images, get a style manifest
        and candidate assets back as a pack draft
```

MVP-2 is the same flow `ART_DIRECTION.md` §23 already describes for style manifests, extended
from describing a style to producing the assets that embody it.

---

## DEP-9 — The frozen technology boundary

**Date** 2026-09-26 · **Consolidates** `DEP-2`, `DEP-3`, `DEP-4`, `DEP-5`, `DEP-6`, `DEP-7`,
`ARC-8`

One table, so the question stops being reopened per subsystem:

| Layer | Choice |
| --- | --- |
| Core runtime, world state, systems, scheduler, persistence, networking, contracts | **Rust** |
| Reference rendering, 2D and 3D | **Godot** (MIT) |
| 3D renderer default | **Godot Forward+** |
| GPU, physics, animation, camera, input, navigation, audio, platform export | **Godot's** — MineWorld writes none of it |
| Canonical 3D asset format | **glTF / GLB** (`DEP-7`) |
| Cognition and ML | **Python** or an external service where useful |
| Long-term system and plugin extension | **WASM Component + WIT** (`ARC-8`) |
| Persistence | **SQLite** locally, Postgres later (`DEP-2`) |
| Transport | **tokio + axum**, HTTP and WebSocket (`DEP-3`) |

### Why Godot rather than Unreal or Unity

Not because it renders best. Because the licence matches the project: MIT, forkable, patchable,
no vendor whose terms can change under an open-source framework, and 2D and 3D in **one** engine
— which matters more here than anywhere, since MineWorld's claim is that both clients render one
semantic world. Its Forward+ renderer already provides the GI, shadows, volumetric fog and
tonemapping a warm semi-realistic lakeside town needs; the target was never cinematic fidelity.

Unreal is stronger visually and stays welcome as an optional high-end renderer
(`renderers/unreal/`), but it is proprietary, carries a revenue royalty, and its build system
would absorb exactly the engineering attention this project cannot spare. Unity's licensing
controversy has eased, yet it offers no advantage over Godot for a project whose contributors
must be able to fork the engine.

**A Rust engine such as Bevy was considered and rejected**, despite the appeal of an all-Rust
stack. It would hand us renderer maturity, asset pipeline, animation tooling, character
controllers and editor workflow as *our* problems — precisely the work this decision exists to
avoid. Technical purity is not the objective; shortest path to an extensible framework is.

```text
Rust  → the world           Godot → the pixels
```

### What this buys

Engineering attention concentrates on the differentiating problem — **how an LM generates a
modular, verifiable, composable world that runs for a long time** — instead of on making a
street lamp three percent faster on the GPU.

---

## ARC-10 — Generation is development tooling, and its absence is not a design constraint

**Date** 2026-09-27 · **Extends** `ARC-9`

`ARC-9` accepted generated assets as candidates. This settles where the machinery lives and what
it may not become.

**Generation is content-production tooling.** It lives in `tools/`, it is never a runtime
dependency of the server or of a client, and no contract references it. A world that has been
built runs without it, exactly as a world runs without a language model (`VISION.md` §1.1). The
distinction is the same one: generation is how content comes to exist, never something the
runtime depends on.

**The interface is deliberately thin**, one implementation at a time:

```text
ImageGenerationProvider   generate(prompt, reference_images, params) -> image
3DGenerationProvider      later, when a second real need exists
```

`REUSE_POLICY.md` §6 forbids abstracting before a second implementation exists, so this stays one
interface with one implementation until a real second backend arrives. MineWorld orchestrates,
validates, normalizes and packages generated content; it does not implement diffusion runtimes,
upscalers or background removers.

**Reference images are the conditioning input, not the prompt.** `ART_DIRECTION.md` §16 already
makes references the source of truth over prose, and that has a technical consequence:
reference-guided workflows (image-to-image, IP-Adapter and their successors) are preferred over
text prompting, because a prompt cannot carry a style with the fidelity a committed image does.

**The rule this decision exists to state:** *the coding agent's own capabilities are not the
project's capabilities.* When generation is unavailable, the answer is to stand up a backend or
to record precisely what is missing — never to fall back to lower-fidelity procedural art and
call it the style. The 2D spike's procedural ceiling was a real finding about procedural
geometry; it was never a finding about what MineWorld's art should be.

**Status, 2026-09-27.** Audited this machine: no ComfyUI, no Stable Diffusion, no Ollama, no
image-generation API credentials, no torch or diffusers. The hardware is capable — Apple M5, 10
GPU cores, 24 GB unified memory, Metal 4, 569 GB free — so this is an installation gap rather
than a capability one, and an agent is standing a backend up. Model choice carries a licence
question of its own: SDXL, SD 3.5 and FLUX differ materially on commercial use and on
redistribution of outputs, and MineWorld commits what it ships, so a model whose outputs cannot
be redistributed is unusable here whatever its quality.

---

## ARC-11 — Default style is taste; style infrastructure is architecture

**Date** 2026-09-27

Two workstreams have been running under one name, and conflating them puts a subjective decision
inside an autonomous loop. They are separated here, with different authority over each.

| | Default style realization | Style infrastructure |
| --- | --- | --- |
| **What** | the official MineWorld 2D and 3D look — buildings, vegetation, characters, atmosphere, materials | Presentation and Asset Pack interfaces, style manifests, provenance, generation integration, renderer bindings, style switching, validation |
| **Nature** | taste | architecture |
| **Decides** | **the operator, finally and always** | the pi-agent, autonomously |
| **Blocks on review?** | that branch does, at integrated milestones | never |

**The default style exists because the operator personally likes it.** So an agent may build,
iterate, assemble and recommend, but it may not decide that something *is* the default look.
Before approval such work is a **default-style candidate**, a **review candidate**, or a
**proposed default presentation** — never a final accepted style, and the wording matters because
"good enough" silently becoming "accepted" is exactly the failure this record prevents.

**Style infrastructure never waits on that.** MineWorld must host anime, pixel, voxel, low-poly,
photorealistic, retro, hand-painted and minimal styles, and none of that work depends on which
style is default. Infrastructure tied only to the current default would fail the project's own
premise, and a pending taste decision must not stall it.

**Review granularity for taste:** whole scenes, whole character results, whole visual milestones.
Never one tree, one bench, one shirt, one shader tweak. A milestone that is ready is marked
`READY FOR HUMAN STYLE REVIEW`, its runnable artefact and launch command preserved, and then
**subjective polishing on that branch stops** — iterating further on a direction the operator may
reject is waste, and it also makes their eventual judgement harder by moving the target.

**What this changes in practice:** the visual tracks split. The generation pipeline, the asset
contract, provenance, the humanoid profile and pack loading are engineering and continue. Whether
the townspeople look right is the operator's, and that branch parks once a scene is reviewable.

---

## ARC-12 — Intent compliance is testable here, and the name is a launch question

**Date** 2026-09-27 · **Source** a reference audit of Microsoft's MineWorld (arXiv 2025), an
unrelated project that shares our name

Microsoft's MineWorld is a **video-generative world model**: given Minecraft frames and an action,
it generates the next frames — `p(x_{i+1} | x_{<i}, a_i)`. Ours is a world *runtime*. The two use
"world" to mean different things: theirs is visual state latent in model parameters, ours is
explicit structured fact. Their action vocabulary is Minecraft's and frozen; ours is composed at
run time by whichever System Packs are installed, so `Talk` exists because `ConversationSystem`
does and `Attack` may not exist at all. Nothing about their architecture transfers.

Three ideas do.

**1. An action is a first-class structured object, not prose.** They discretize Minecraft's input —
exclusive key groups, camera movement quantized into bins — into 11 tokens interleaved with visual
tokens. Utterly different mechanism, same conviction, and it is the one we already froze in
`ActionIntent`. Worth recording as independent corroboration rather than as something to copy.

**2. Controllability must be evaluated separately from plausibility.** Their sharpest idea: a
generated frame looking right does not mean it followed the action, so they run an inverse dynamics
model over the output to recover which action *actually* happened and compare it to the one
requested. They are explicit that visual quality alone is insufficient.

**This is the part MineWorld should adopt, and we can do it far more strictly than they can**,
because we hold structured state and need no second model to guess what happened:

```text
ActionIntent  GiveItem(Alice, Bob, Coffee)
expect        an ItemTransferred event, and owner(Coffee) == Bob
```

So `intent compliance`, `action validity`, `state transition correctness` and
`renderer consistency` become a class of integration evaluation we can assert directly. Recorded
here as a testing principle for `AC-13` and for every System Pack: **a system that accepts an
action must be shown to have caused the consequence it claims**, not merely to have returned
`Accepted`.

**3. Real-time is a first-class requirement, and that argues for our shape.** They treat latency as
a headline metric and optimize hard — 2 FPS to 5.91 FPS through diagonal decoding, on 32×A100 for
200k steps. The lesson is not the algorithm, which we will never need. It is that generating every
frame is ruinously expensive, and therefore:

> **LM-native does not mean an LM produces every frame.**

MineWorld's layering is the cheap one and should stay: language models make low-frequency semantic
decisions, the runtime holds authoritative state, and Godot renders at 60 FPS. `VISION.md` §1.1
already says authoring is LM-native while the runtime is not LM-dependent; this is the performance
argument for the same split.

**A generative renderer is a plausible future Presentation Pack, and is neither default nor MVP.**
Someone may one day want a video model as the presentation layer instead of a rasterizer. Our
renderer-independent contracts should carry that without change, which is a nice confirmation that
`INV-5` and `INV-14` are worth what they cost. Their project is imaginable as an exotic
*presentation backend*, never as our kernel.

**The name.** Microsoft's MineWorld has a 2025 paper, a GitHub repository and a Hugging Face
presence. That is a discoverability and package-naming collision, not a technical one, and it
blocks nothing today. **Re-evaluate the name before any public launch** — recorded so the decision
is deliberate rather than discovered in a search result.

---

## DEP-1 — Component storage: purpose-built, not an ECS

**Date** 2026-09-25 · **Supersedes** effort decision `D-5`, which is now this record

**Problem.** Store typed component state keyed by entity, where each component type is owned by
exactly one system, systems are installed and removed per world, all state is serializable, and
the whole store is reconstructible from an append-only event log.

**Options considered.** `bevy_ecs`, `hecs`, `shipyard`, `legion`, `specs`, `edict`; or a
purpose-built typed store.

**Choice: purpose-built**, one component table per owning system, keyed by `EntityId`.

**Why not an ECS.** Three mismatches, in order of severity.

1. *Identity.* Almost every Rust ECS uses generational handles that are not stable across a
   save/load cycle; `bevy_ecs` is the notable exception with globally unique ids. MineWorld's
   `EntityId` must be stable, serializable, and referenced by events that outlive the process
   ([`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §3, `AC-6`).
2. *What they optimize.* Archetype storage buys cache-friendly iteration over tens of thousands
   of entities per frame. MineWorld ticks semantically over hundreds of entities and spends its
   budget on causality, persistence and migration instead. We would pay the complexity and gain
   nothing measurable.
3. *Ownership.* `INV-7` requires that a system can only write components it owns. An ECS hands
   any system `&mut World`, so single-writer would degrade to a review convention — the exact
   guarantee the composability claim rests on
   ([`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §15).

Dynamic registration is a fourth, smaller mismatch: System Packs declare component types at load
time, which archetype ECSs support awkwardly, and serialization of dynamically registered types
generally needs a reflection layer we would then also own.

**Why not ourselves is weaker here.** This is `REUSE_POLICY.md` §7's third case: study the
implementations, build the smaller MineWorld-specific version. The store is a `BTreeMap` per
component type behind a narrow API, not a subsystem.

**Isolating interface.** The kernel's component-store API. If entity counts ever make iteration
the bottleneck, the internals can be replaced — including by an ECS — without touching systems,
because systems never see the storage.

**Accepted limitations.** We maintain this code. Iteration performance is far below an archetype
ECS, which is acceptable at MVP scale and is the thing to re-measure before it is not. No
built-in parallel scheduling; determinism (`AC-12`) currently matters more than parallelism.

**Licenses.** All candidates are MIT/Apache-2.0 and would have been compatible.

---

## DEP-2 — Persistence: `rusqlite` behind `PersistenceBackend`

**Date** 2026-09-25 · **Status** selected, integrated in S5

**Problem.** Durable local world state and an append-only event log, with a cloud backend later
([`ARCHITECTURE.md`](ARCHITECTURE.md) §8).

**Options.** `rusqlite`, `sqlx`, `diesel`, `sea-orm`, or a file format of our own.

**Choice: `rusqlite` with the `bundled` feature**, which compiles SQLite into the binary so a
local server has no system dependency — directly serving the local-hosting requirement in
[`NETWORKING.md`](NETWORKING.md) §6.

**Why not the others.** `sqlx` is async-first with compile-time query checking, which suits a
service talking to a remote database; our persistence is local and sits behind a blocking task,
so its async model buys nothing and costs build complexity. It is also a semver hazard alongside
`rusqlite`, since only one crate version may link `libsqlite3-sys`. An ORM is the wrong shape for
event sourcing, where rows are append-only facts rather than mapped objects. Writing our own file
format would be rebuilding commodity infrastructure (`REUSE_POLICY.md` §4) and losing crash
safety we would then have to prove.

**Isolating interface.** `PersistenceBackend`, with `SqliteBackend` as the first implementation
and Postgres later through the same trait. No SQL or SQLite type appears in the kernel or in any
system.

**Accepted limitations.** Synchronous API bridged with `spawn_blocking`. Single-writer concurrency
model, which matches an authoritative single-world server and would need revisiting only for
multi-world processes.

**Implementation note, 2026-09-30 (S5, PR 07).** Integrated as the crate `mineworld-persistence`
(`persistence/`), which holds `PersistenceBackend` and its one implementation `SqliteBackend`; the
kernel, every system and the server name no SQLite type. A save is **one file**, `world.sqlite`, whose
tables are the manifest, the journal, the facts and the snapshots (`ARC-25`), so that one transaction
commits a whole revision — a separate `manifest.json` would be a second copy of a fact outside that
transaction. WAL mode; `synchronous = FULL` for a hosted world, so a revision a client has been told
survives power loss, with `NORMAL` (durable against a process crash) available to headless bulk runs.
No `spawn_blocking` is needed: the server already runs its world on a dedicated blocking thread
(`server/src/host.rs`), which is where every persistence call happens.

---

## DEP-3 — Server and transport: `tokio` + `axum`

**Date** 2026-09-25 · **Status** selected, spike-proven, integrated in S11

**Problem.** HTTP control plane and WebSocket streams for observations and intents
([`NETWORKING.md`](NETWORKING.md) §3), identical for localhost, LAN and cloud.

**Options.** `axum`, `actix-web`, `warp`, `poem`, `hyper` directly, or raw `tokio-tungstenite`.

**Choice: `axum` on `tokio`**, with `axum`'s WebSocket support.

**Why.** It is a focused library over `hyper` rather than a framework demanding we adopt its
execution model (`REUSE_POLICY.md` §3), the stack is already the Rust default named in
[`ARCHITECTURE.md`](ARCHITECTURE.md) §13, and a spike on 2026-09-25 round-tripped an
`ActionIntent` and an `ActionResult` between a Godot client and an `axum` server before any
commitment was made (§15).

**Isolating interface.** A transport module owning the socket lifecycle; the kernel never sees an
HTTP or WebSocket type, and simulation semantics are transport-independent (`INV-14`).

**Accepted limitations.** JSON over WebSocket is not a compact encoding; acceptable for a life
simulation, and replaceable behind the same boundary. 64-bit ids must reach a client as decimal
strings — a defect found by that same spike (the S2 plan's DD-15 and risk R-9). That is no longer
the transport's obligation: the renderer-integration spike showed a protocol-level encoder cannot
reach inside a component or event payload, so `mineworld-contracts` now encodes the four opaque
identities as decimal strings whenever the format is human-readable and as a `u64` whenever it is
not (`contracts/src/ids.rs`, `spike/FINDINGS.md` F2). A transport carrying contract types in JSON
therefore inherits the correct encoding and must not re-implement it.

---

## DEP-4 — Presentation: Godot 4.7 for both reference clients

**Date** 2026-09-25 · **Status** selected, spike-proven

**Problem.** A 2D client and an embodied 3D client, neither of which may contain a world rule
([`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §§2, 8, 10).

**Choice: Godot 4.7.2 for both**, as [`ARCHITECTURE.md`](ARCHITECTURE.md) §13 already named.

**What we reuse rather than build**, all listed as commodity in `REUSE_POLICY.md` §4: the
renderer, physics and collision, the character controller (`CharacterBody3D`), camera, raycast
targeting, input handling, UI layer, and later navigation. MineWorld writes none of this.

**Why not something else.** Unreal is heavier for a low-fidelity embodied demo and is already
planned as a later adapter. A browser stack would have been viable and easier for me to automate,
but it would deviate from the frozen spec for a convenience reason, and the verification concern
it addressed turned out not to exist.

**Evidence before commitment.** Installed, then verified in three steps: headless script
execution; a windowed run rendering through Metal with `save_png` producing an inspectable image;
and an embodied spike that walked a collidable room, raycast-targeted an NPC by entity id, and
rendered a server-supplied affordance.

**Isolating interface.** The client speaks only the wire protocol. Engine types never enter a
contract (`ENGINEERING_RULES.md` §12), and the S14 adversarial criterion is that deleting the 3D
client changes no system and touches no kernel contract.

**Accepted limitations.** GDScript is a second language at the presentation boundary, which the
standards permit for renderer adapters. Godot's JSON parses numbers as doubles, handled at the
protocol boundary rather than in the contracts.

---

## DEP-5 — Serialization: `serde`

**Date** 2026-09-25 · **Status** selected, in use since S1

`serde` with derive for every contract type, `serde_json` for the wire and for tests. The Rust
ecosystem standard, zero operational burden, and format-agnostic so a binary encoding later needs
no contract change. `thiserror` accompanies it for typed errors, which
[`ENGINEERING_STANDARDS.md`](ENGINEERING_STANDARDS.md) §12 requires anyway. Both MIT/Apache-2.0.

**Note, 2026-09-30 (S5, PR 07).** `serde_json` becomes a runtime dependency of `mineworld-kernel`,
not only a dev-dependency. A world snapshot must encode every component row, and only the kernel still
holds those rows as their Rust types (`DEP-1`'s store erases the type behind each table); it encodes
each as a `ComponentRecord` whose payload is the component's JSON. This is the persistence encoding of
state the kernel holds, not a choice of payload format for a system: event and action payloads remain
bytes a system encodes and the kernel never interprets. JSON over values ordered by `BTreeMap` is
canonical — and `clippy.toml` bans `HashMap` across the workspace — so equal state encodes to equal
bytes, which is what replay verification compares (`ARC-25`).

---

## DEP-6 — Scheduler: purpose-built discrete-event queue

**Date** 2026-09-25 · **Status** decided; supersedes the provisional recommendation

**Problem.** Advance a world clock over simulated time, wake processes at scheduled moments,
deliver events to subscribed systems, skip idle time so hundreds of simulated days are cheap
(`AC-11`), and reproduce a run exactly from the same seed (`AC-12`) — while the same process
serves real network clients over tokio and can be saved and resumed mid-run (`AC-6`).

**Options considered.** `DesCartes`, `desim`, `simulacra`, `score`, or a purpose-built queue.

**Choice: purpose-built**, a `BinaryHeap` keyed by `(WorldTime, sequence)` inside the kernel.

**Why not the crates — one decisive reason, the same for most of them.** They model a *process*
as a coroutine: SimPy-style, where in-flight state lives on a suspended call stack. MineWorld
models a `Process` as serializable state owned by a system, with participants, progress and
interruptibility as data ([`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §10). That difference is not
cosmetic: a suspended coroutine cannot be written to SQLite and resumed in a new process, so
adopting one would forfeit "save, shutdown, restart, continue" for any process in flight —
a frozen MVP requirement, not a nicety.

Per crate, beyond that: `DesCartes` supplies its own deterministic async runtime as a
replacement for tokio, which is precisely the framework lock-in
[`REUSE_POLICY.md`](REUSE_POLICY.md) §3 warns against, and our server needs real tokio for real
sockets. `simulacra` models network message flow, a different domain. `desim` and `score` are
both process-as-coroutine.

**Why ours is small.** A binary heap, a monotonic sequence counter for tie-breaking, and a loop.
The hard part of this scheduler is not the data structure — it is the determinism rules, which
are ours to define either way.

**Isolating interface.** The scheduler is internal to the kernel; systems see scheduled wake-ups
and delivered events, never the queue.

**Accepted limitations.** Single-threaded advancement. Parallel system execution would need a
different design, and determinism currently matters more than throughput at MVP scale.

**Licenses.** All candidates are permissive; none was rejected for licensing.

**Implementation note, 2026-09-30 (S4, PR 06).** The queue is a `BTreeMap` keyed by
`(WorldTime, Sequence)` rather than a `BinaryHeap`: the same key and the same order, but a heap's
layout depends on its insertion history, so two equal queues could serialize to different bytes,
and a queue S5 saves and compares across runs needs a canonical form. The choice — purpose-built,
not a crate — and the reason for it are unchanged. Recorded here so that the code and this decision
do not disagree (`CLAUDE.md` §2.1 rule 4); the evidence is
`.structured-coding/plans/mvp0/step-04-clock-scheduler-process.md` §8 F-9.

---

## ARC-15 — A world's initial state is a recorded genesis fact

**Date** 2026-09-27 · **Implements** `MODULE_SPEC.md` §4 · **Relates to** `MVP.md` §9 `AC-9`,
`AC-12`

**Problem.** A World Pack states what is true of a world before anything happens: Alice is behind the
counter. That is component state, and in MineWorld a component is written **only** by the system that
owns it, **only** while it resolves an action or reacts to a fact. World assembly has no action to
resolve — so PR 05b, writing the server's first test world, found that placing four people meant
dispatching four intents, and `ActionIntent::new` needs an `ActionId` that at assembly time no
allocator has issued. Its test world worked around it by allocating from `9_000_000`, far above the
server's own allocator, so that assembly could not collide with a client's request.

**Options considered.** (1) A seeded allocator handed to the loader through configuration — the
server's allocator starts above whatever assembly spent. (2) An assembly-time allocator owned by the
pack loader. (3) A kernel path for authored state.

**Choice: (3), in the form that records events.** `World::genesis(at, facts)` records what a world
begins with and reduces it through the systems that own that state, using the same recorder and the
same reducer a request goes through. Two differences and no third: causation is
`Causation::WorldGenesis`, and there is no controller decision to name.

**Why, and the axis is the event log.** Options 1 and 2 both produce `Causation::Action(9_000_000)`
in a world's history — a request no controller made, no client sent and no allocator issued.
Replaying that log asks *what was action 9000000?* and the honest answer is *nothing; it was a number
chosen to avoid a collision*. That is inventing history, and moving the fabrication from a test into
the loader would tidy the workaround rather than answer it. Option 1 additionally leaks
world-assembly detail into the transport's configuration, so a loader and a server would have to
agree on a number to stay out of each other's way — a coupling with no owner.

A kernel path that wrote components directly would fail the other way: state with no cause in the
log, which breaks `AC-9` for the whole of a world's initial state and makes it unreconstructible from
its own history. `kernel/src/system.rs` already refuses exactly that for `System::install` — "a
system that wrote state while being installed would write facts no event explains" — and
`contracts/src/event.rs` already named the answer: `WorldGenesis` exists "so that a world's initial
state is *explained* rather than uncaused", and "a loaded World Pack's initial facts are caused by
this and by nothing else". The decision was therefore less a choice than a reading of two contracts
that had already made it.

**What it buys.**

```text
no ActionId is invented      Provenance::controller_decision stays None, which is what it means:
                             this fact came from no request. The server's allocator still starts at
                             1 and can never collide with assembly, because assembly allocates no
                             request identities at all.
state has a causal origin    "where did Alice's initial position come from" answers with an event
                             id, a genesis causation, and the system that reduced it — by the same
                             path a walked arrival takes, so state and log cannot disagree.
AC-12 holds through seeding  event ids come from the world's own counter in the order the loader
                             states the facts, and that order is the pack's (places in key order,
                             then people in key order).
```

**Scope, stated exactly.** Three public kernel items (`World::genesis`, `Emission::owner`, two
`KernelError` variants) and no domain knowledge: `genesis` cannot name a component, an action or a
system, and reads each fact's emitting system off `Event::OWNER` in the contract rather than from an
argument. `contracts/` is unchanged — `WorldGenesis` becoming reachable is that layer's own stated
intent.

**Accepted limitations.** Genesis is refused once a world has dispatched anything, so *the world
coming into existence* stays true of the log that claims it; a person who arrives while a world runs
arrives by acting. Reduction of genesis facts is not transactional, exactly as dispatch's is not: a
system that breaks its own contract while reducing leaves what it wrote, and a failed assembly is
discarded rather than repaired (`kernel/src/world.rs`).

---

## DEP-10 — World Pack YAML: `serde-saphyr`

**Date** 2026-09-27 · **Status** selected, integrated in PR 05c · **Extends** `DEP-5`

**Problem.** Read hand-authored YAML — a World Pack's `world.yaml`, `people/*.yaml`, `places/*.yaml` —
into validated Rust types, with errors an author can act on. The pack format is the surface a world
creator actually writes, so *where* an error is reported matters as much as *that* it is.

**Options considered.** `serde_yaml`, `serde_yml`, `serde_yaml_ng`, `noyalib`, `serde-saphyr`,
`yaml-rust2`/`saphyr` with a hand-written mapping layer, or TOML/JSON instead of YAML.

**Choice: `serde-saphyr` 1.3**, deserializer feature only.

**Why not the `serde_yaml` lineage.** `serde_yaml` was archived by its author in 2024 and says so in
its own README. `serde_yml` is a fork that has since self-deprecated. `serde_yaml_ng` is a
continuation of the same lineage and therefore of `unsafe-libyaml`, which is archived as well — a C
parser transliterated to `unsafe` Rust, under a crate nobody maintains, parsing files a world author
downloads from the internet. That is the wrong dependency to adopt in a workspace where every crate
carries `#![forbid(unsafe_code)]`.

**Why not `noyalib`.** Pure Rust, `forbid(unsafe_code)`, and a drop-in `compat-serde-yaml` feature —
attractive. But it is at `0.0.51`, pre-1.0 with no stability commitment, and it is published by the
author of `serde_yml`, the fork that deprecated itself. MineWorld needs the `Value` type and the
compatibility shim that are `noyalib`'s main argument for neither: the loader only ever calls
`from_str::<MyStruct>`.

**Why not a parser plus our own mapping.** `saphyr`/`yaml-rust2` is the right layer for a tool that
needs the YAML tree — an editor, a formatter. Writing a `serde` bridge over one to read four struct
shapes would be `REUSE_POLICY.md` §7's mistake in the other direction: rebuilding commodity
infrastructure that exists and is maintained.

**Why not TOML or JSON.** `MODULE_SPEC.md` §4 specifies a World Pack in YAML and MVP-0 does not
reopen a frozen format decision. On the merits YAML is also right for this content: it is what a
person hand-writes, it carries comments, and every one of this repository's pack files uses them to
explain itself.

**What `serde-saphyr` provides that decided it.** MIT OR Apache-2.0. Crate-level `deny(unsafe_code)`.
A stable major version, actively maintained, with `serde` as its only reason to exist. Errors that
carry line, column and an excerpt of the offending text — so `locatoin:` in `people/alice.yaml` is
reported as *line 2 column 1: unknown field `locatoin`, expected one of tags, note, location*, with
the source line printed under it. And a **duplicate-key policy that errors by default**, which is the
behaviour a configuration format needs, and the opposite of the usual default: a reader that took the
last of two identical keys would hand an author a world composed of a system they had deleted.

**Isolating interface.** One call site: `worldpack::read::parse`, twelve lines, which reads a file to
a string, deserializes it and turns any failure into `PackError::Malformed { path, kind, detail }`.
No `serde_saphyr` type appears in any signature, any struct field or any error variant of this
workspace, so replacing the parser is a change to one function.

**Accepted limitations.** The parser's error text names the input as `<input>` rather than the file,
so `PackError::Malformed` prints the path itself on the line above. Only the `deserialize` feature is
enabled; nothing in MineWorld writes YAML, and if a `mineworld create` command ever does (S7), that
is a feature flag and not a new decision. Anchors, aliases and `!include` are supported by the crate
and are deliberately not used by any pack — a World Pack an author can read is worth more than one
that avoids repetition.


---

## ARC-13 — The 3D lighting rig was changed on purpose, and it is one revert

**Date** 2026-09-27 · **Scope** `clients/3d-spike` · **Status under `ARC-11`** default-style
**candidate**, not an accepted default

This records a **reversal of direction**, because a reversal that lives only in an agent report
is not reviewable later, and `CLAUDE.md` §2.2 puts design decisions in documents rather than in
conversation.

**What happened.** The 3D spike's lighting rig was repeatedly protected: it was the best thing in
the spike and the standing instruction was to preserve it unchanged while the character and
buildings were worked on. Later review of the same build against
`presentation/mineworld-default/3D/references/` reversed that: the operator's complaint was
**画风** — the rendering idiom, not the geometry inventory — and the lighting was identified as the
single highest-leverage gap, above massing, weathering and asset detail.

**Why the later direction wins.** The plates are warm, saturated and lit by a low sun. The rig was
a bright overcast sky, a pale key and mild saturation, so every frame read grey regardless of what
the geometry did. No amount of bays, gables or dirt passes changes the register of a frame lit
like an overcast afternoon. The shot named `21_street_golden`, framed against
`05_main_street_golden_hour`, contained no golden hour at all — a filename promising what the
frame did not deliver, which is its own kind of evidence defect.

**What changed, and what deliberately did not.** The *structure* of the rig is untouched, because
the structure is the part that was good: one sky, one sun, one tonemap, one exposure, SSAO, glow,
and every asset judged inside it (`DEP-8`'s coherence procedure, step 3). Only values changed:

| | before | after |
| --- | --- | --- |
| sky HDRI | `kloofendal_48d_partly_cloudy_puresky_2k.hdr` | `qwantani_puresky_2k.hdr` |
| sun elevation | 26° | 17° |
| sun colour | `(1.0, 0.88, 0.71)` | `(1.0, 0.80, 0.56)` |
| sun energy | 2.35 | 2.9 |
| tonemap exposure | 1.18 | 1.06 |
| adjustment saturation | 1.06 | 1.16 |
| adjustment contrast | 1.04 | 1.07 |

All of it is in `scripts/main.gd` `_environment()` and `_sun()`, plus one filename. **Reverting is
those seven values and the sky file** — deliberately, so that a taste decision the operator has
not yet made stays cheap to undo.

**Authority.** Under `ARC-11` this is taste, so it is a *candidate*. No agent may record it as the
accepted default look; that remains the operator's, finally and always.

**Also fixed while in here, and not a taste decision:** the distant ridge's heightfield was wound
clockwise seen from above, so its generated normals pointed at the ground — lit from underneath,
culled from above, reading as pale angular shards floating over the lake. That was a defect, and
it is fixed regardless of what happens to the lighting.
---

## ARC-14 — The default 2D style is `town`

**Date** 2026-09-27

[`ARC-11`](#arc-11--default-style-is-taste-style-infrastructure-is-architecture) separates the
default style, which is taste and the operator's to decide, from the style infrastructure, which
is architecture and proceeds autonomously. This record is that decision taken: **the operator
compared four complete scenes and chose `town`.** It is no longer a candidate.

**What was compared.** Four art variants of the same walkable square, identical in layout,
projection, camera, crowd density and scene logic, differing only in which art each role
resolves to:

| Variant | Art |
| --- | --- |
| `town` | generated cast, plus the shared generated set — buildings, props, vegetation, ground |
| `full` | generated cast, plus shopfronts and planted props generated for the spike |
| `people` | generated cast over the procedural world |
| `procedural` | the earlier all-procedural build, as the baseline |

All four were rendered at one commit and at matched framing, including a still framed to
`presentation/mineworld-default/2D/references/02_cafe_street.png` so the comparison against the
plates is like for like. The sets are in `clients/2d-spike/screenshots/`, and
`clients/2d-spike/README.md` gives the launch commands.

**What distinguishes the choice.** `town` carries a warm overall key measurably close to the
reference plates — whole-frame mean luminance 0.513 against the plates' 0.517, with 4.0% of the
frame below 0.20 luminance against 4.4%. Beyond the numbers: cherry blossom against the greens,
a varied roofline of terracotta, slate and tile, and building silhouettes that differ from one
another rather than repeating one mass. It reads as a place with a history of being built in
rather than a row of one shop.

**Its known weakness, accepted.** The shared set has four distinct tree sprites, so repetition
shows at wide zoom where `full`'s nine procedural species do not. The operator chose `town` with
that visible.

**The other three are kept.** They are not dead alternatives — they are the demonstration that
the presentation layer is swappable, which is the substance of `ARC-11`'s split. `--variant=`
remains the interface, and a change that can only be made to `town` is a change made in the
wrong place.

**This fixes the default, not the style system.** MineWorld must host anime, pixel, voxel,
low-poly, photorealistic, retro, hand-painted and minimal styles
([`ARC-11`](#arc-11--default-style-is-taste-style-infrastructure-is-architecture)). A World Pack
ships its own Presentation Pack and selects its own style without touching this choice and
without modifying the default pack; nothing here privileges `town` in the loader, the contracts
or the renderer bindings. If a later change makes `town` hard to replace, that change is the
defect, not this record.

**Recorded because** "good enough" silently becoming "accepted" is the failure `ARC-11` exists to
prevent, and the converse also needs a record: once the operator has chosen, an agent should not
reopen the question as though it were still open.

**Note, 2026-10-08 — how this record reached `main`.** The text above is carried verbatim from
`vis/2d-generated-assets` @ `af5e236`, the branch it was written on, which is not merged (S12 PR 13a,
`.structured-coding/plans/mvp0/step-13-client-2d.md` §14). It fills the identifier gap `ARC-16`
recorded. Two things changed since, both operator decisions of 2026-10-08 (QS12-1, QS12-2): the
style is now drawn by the connected reference client `clients/2d/`, laid out from a world's disclosed
passages (market-town's street) rather than the spike's invented square; and the art moved into
the Presentation Pack `presentation/mineworld-default/2D/`, where the four variants are binding sets
(`ARC-46`). `clients/2d-spike/` and its screenshots stay on that branch as the visual reference.
Neither change reopens the choice of `town`.

---

## ARC-16 — A decision identifier names one decision, and a check enforces it

**Date** 2026-09-27 · **Source** two collisions found in one session · **Relates to**
`CLAUDE.md` §2.1(4), `ARC-12`

`docs/DECISIONS.md` is the authority that every specification cites *by identifier*. An identifier
naming two decisions makes each citation of it unresolvable, which is exactly the code/specification
contradiction `CLAUDE.md` §2.1(4) forbids leaving unrecorded. Two such collisions existed at once:

- **`ARC-10` was allocated twice on `main`** — "Generation is development tooling" (12:14) and
  "A world's initial state is a recorded genesis fact" (16:26), four hours apart, both on `main`,
  neither noticed. Two documents cited the second and four cited the first.
- **`ARC-13` was allocated twice across branches** — the 3D lighting reversal at 19:13 and the
  default 2D style at 19:28, on two branches from one `main`.

**The mechanism is what makes this worth a decision rather than a fix.** The two entries land in
different regions of a long file, so `git merge` reports no conflict and `main` simply gains a
duplicate. Reading `main` before allocating is not a defence: the 3D branch merged `main`
specifically to avoid this and collided anyway, because nothing serialises the allocation and the
other branch read the same `main` minutes later. A human noticing is not a control.

**Decided.**

1. **The earlier entry keeps the identifier**, by commit time, and the later one takes the next free
   number. Renumbering is not unilateral once others cite a decision — move every citation with it,
   in the same commit.
2. Applied here: the genesis decision became **`ARC-15`** (`MODULE_SPEC.md` §4 and the PR 05c plan
   moved with it); the generation-tooling `ARC-10` is unchanged, as the earlier of the two and the
   more widely cited. The 2D style decision became `ARC-14`.
3. **`scripts/check_decision_ids.py` fails on a duplicate** and names both entries with their line
   numbers. It was written against the real defect and verified both ways: it exits 1 on the tree
   before this commit and 0 after.

**Accepted limitation, stated plainly.** There is no CI in this repository yet, so this check is
declared in `.structured-coding/standards.md` and the standards helper *reports* — it does not
block, and `CLAUDE.md` §3.3 is explicit that a clean report is not proof a check ran. Until CI
exists this is a check that must be run, not a gate. That is weaker than the defect deserves and is
recorded as such rather than described as enforcement.

**What this does not do.** A checker catches a collision after it happens. Reserving identifiers up
front, or allocating them at merge, would prevent it. That is a process change rather than a script
and is left undecided.

**Identifier gap.** `ARC-13` and `ARC-14` are allocated on `vis/3d-human-pipeline` and
`vis/2d-generated-assets` and arrive on `main` when those branches land. The gap between `ARC-12`
and `ARC-15` here is that, not a deleted decision.

---

## ARC-17 — A declared reference is the source of truth, and vague comparison is prohibited

**Date** 2026-09-27 · **Governs** [`VISUAL_FIDELITY.md`](VISUAL_FIDELITY.md) · **Relates to**
`ARC-9`, `ARC-11`, [`ACCEPTANCE.md`](ACCEPTANCE.md)

A 3D character was brought to human review described as *"same wardrobe, recognisably not the same
person"*. The reference is a young woman in an open burgundy zip hoodie; the candidate was a man in
a red quilted puffer jacket, with black close-cropped hair against a brown voluminous updo, and
with the shirt graphic and backpack — the two strongest recognition cues on that character —
recorded as deferred. Nothing in the sentence was false, and it was still the wrong verdict.

**The root cause is a task definition, not a tuning gap.** The reference was treated as style
inspiration when it is the identity source of truth for one specific character. The implementation
also ran backwards: an available humanoid was chosen first and the reference was then approximated
around what that asset could provide.

**Decided.**

1. When a deliverable declares a reference, the task is to **recreate that specific visible thing
   as faithfully as practical**. Attractive and recognisably something else is a failure.
2. Requirements travel **reference → constraints → asset**, never asset → material edits →
   approximate match. An asset that cannot meet a constraint is evidence about the asset.
3. **Identity-defining features are never deferred** once a milestone is named for its reference.
   Deferring them is legitimate only while the milestone is a technical one.
4. **Hard-fail categories** the agent rejects itself, without involving the operator: perceived
   gender presentation, apparent age, body silhouette, hairstyle category, major hair colour,
   outer-garment category, a missing defining garment structure, grossly wrong face shape, missing
   identity-defining accessories, and the massing or silhouette of a referenced building.
5. **Vague comparative language is prohibited** in fidelity reporting — "broadly similar",
   "roughly matches", "approximately right", "same wardrobe". A verdict names the reference fact
   and the candidate fact side by side. This is the operative half of the decision: the banned
   phrasing is what allowed a categorical failure to read as polish.
6. Comparison is **pixels against pixels, at the reference's own framing**. This project has
   produced the same evidence defect three times — a face judged at 60 px, a character judged at
   street distance, and a review contact sheet at 620 px per tile read as "nothing changed" when
   two things had in fact changed.

**Deliberately general.** The failure was found on a character, but the reasoning that produced it
—"warm town, broadly similar" — applies just as easily to buildings, interiors, vegetation and
lighting, where each piece would run correctly and the whole would resemble nothing. The gate
therefore governs every referenced visual deliverable in both reference clients.

**What this does not change.** `ARC-11` still holds: the operator owns final visual acceptance and
the agent owns the architecture. `REUSE_POLICY.md` still holds: reuse remains preferred, and this
decision constrains only the direction the specification travels. A fidelity rejection resets a
visual candidate and never the engineering beneath it.

---

## ARC-18 — Godot stays the reference renderer; a second renderer is a spike, not a migration

**Date** 2026-09-27 · **Relates to** `ARC-17`, `DEP-9`, `ENGINEERING_RULES.md` §§11–12 ·
**Status** spike authorised, outcome open

`ARC-17` raises a fair question: can Godot reach the fidelity of the reference plates at all? The
answer is that nothing in those plates is out of reach for Forward+ — PBR, subsurface scattering,
baked `LightmapGI` with reflection probes, volumetric fog, HDR tonemapping are all present. The
real cost is **character and asset production**, not the renderer's ceiling. The current character
does not resemble the reference because it was built from the wrong mesh, hair, garment and
materials, which is `ARC-17`'s finding and not a rendering limit.

Unreal's advantage is therefore not a higher ceiling but a **shorter path** to it: a mature
ecosystem for high-fidelity humans, groom, cloth, cinematic lighting and retargeting.

**Decided — do not migrate.**

1. **Godot remains the official reference renderer.** It is MIT, so MineWorld can stay permissively
   licensed, forkable and modifiable end to end. Unreal grants source access under the Epic EULA
   and is not permissive open source. For a project that ships as open infrastructure, the fully
   open reference client is not negotiable.
2. **A second renderer is an additional Presentation Adapter**, never a replacement, and this is
   an architecture test as much as a visual one. Presentation independence is a claim this project
   makes; if adding a high-fidelity client requires changes to kernel or System contracts, the
   claim is false and we want to discover that now, while the kernel is small. If the interfaces
   are clean, the new client is a client.
3. **A time-boxed Unreal spike is authorised** to answer, with both slices in front of the
   operator: connect to the existing server, project the same Alice, build a reference-quality
   small-town slice with an enterable café and a reference-faithful character, and report the
   actual engineering cost of reaching a given visual target in each engine.
4. The outcome is **open**: Godot is good enough and continues alone; or Unreal becomes the default
   high-fidelity 3D client with Godot retained as the open reference; or both are kept, which would
   be the strongest demonstration of the framework claim.

**Cost, recorded because it becomes a licensing question if the spike succeeds.** Unreal Engine is
source-available proprietary software under one agreement, the Unreal Engine End User License
Agreement, which supersedes the former Publishing and Creators EULAs. Epic runs **two** payment
regimes with **two distinct $1M thresholds**, and they must not be merged. A MineWorld Unreal client
relies on Engine Code at runtime and is licensed to third-party end users, so it is a **Royalty
Product**: no seat fees at any company revenue, and **5% of worldwide gross revenue attributable to
the product**, excluding the first **$1,000,000 lifetime per product**, quarters under $10,000, the
first $5M on the Oculus Store, and Epic Games Store and Fab revenue outright. Ports share one $1M
allowance. The rate falls to **3.5%** for a "Launch Everywhere with Epic Release" — Epic Games Store
release before or simultaneous with other stores on each platform, plus content, feature and
marketing parity, for products released on or after 2025-01-01, reverting to 5% on disqualification.
The separate **$1,850 per seat per year** subscription applies only to uses that are *not* Royalty
Products, and only once the corporate group passes $1M gross revenue over the **trailing twelve
months**. This is a summary and the EULA governs.

It does not affect MineWorld's own licence: the server, kernel, contracts and Godot clients are
unaffected, and an Unreal adapter would be a separately licensed deliverable. The binding constraint
is not the royalty but §5(a): **no Engine Code may appear in a public MineWorld repository**, Engine
Code goes only to same-version Epic licensees, public distribution of Engine Tools must go through
Fab or a fork of Epic's GitHub `UnrealEngine` network, and a Product embeds Licensed Technology only
in object code as an inseparable part. MineWorld-authored **runtime** modules containing no Engine
Code may be MIT; **editor tooling is at risk** under the Engine Tools definition and belongs outside
the public repository.

**Corrected 2026-09-27** by the phase-one spike. The paragraph above previously read *"Unreal is free
below $1M USD trailing-twelve-month revenue"*, which described the **seat** exception — measured on
company revenue over twelve months — and applied it to the **royalty** exemption, which is lifetime
and per product. Epic names the confusion itself: *"There are two $1 million thresholds and they
depend on what you make and how much you make."* The seat regime, the 3.5% figure, the additional
exclusions, the Ports rule and the source-distribution constraint were all absent. Evidence, quoted
sources and the access limitations behind them are in
[`references/UNREAL_ADAPTER_SPIKE.md`](references/UNREAL_ADAPTER_SPIKE.md) §10, which also records
that Epic's own CDN still serves a 2022 EULA PDF that a search will return first and that contains
none of these terms.

**What is not in scope.** No kernel, contract, World Pack, persistence or networking change. Those
carry over untouched and their being untouched is the point. Only client-side work is new: the
protocol binding, `Entity` → actor projection, input → `ActionIntent`, cameras, and the scene.

**If the spike requires a contract change to proceed, stop and report it.** That result is more
valuable than the slice.

---

## ARC-19 — The default character is an identity reconstruction; the framework's bar stays low

**Date** 2026-09-27 · **Supersedes** part of `ARC-4` · **Relates to** `ARC-17`,
`ART_DIRECTION.md` §§3, 7, 10, 16

`ARC-4` scoped `3D/references/04_character_closeup.png` as **not** authoritative for facial
fidelity, skin rendering or hair simulation, and fixed the default 3D character at "realistic
proportions with moderately simplified face and materials, medium detail — no photoreal skin, no
MetaHuman-level assets, no bespoke character pipeline."

The operator has since rejected a candidate character **specifically on face, hair, freckles and
garment structure**, and has designated that image the identity source of truth for the default
character. `ARC-4`'s scoping and the current requirement cannot both stand, and leaving the
contradiction unrecorded is the defect `CLAUDE.md` §2.1(4) names.

**Decided: `ARC-4`'s facial-fidelity exclusion is superseded for the default character, and
`ARC-4`'s reasoning is preserved by separating two things it treated as one.**

`ARC-4`'s argument was that a photoreal face standard puts character production beyond what
community creators can afford, contradicting why a default style exists at all
(`ART_DIRECTION.md` §10). **That argument is correct and is not overturned.** It was applied to
the wrong object. It is an argument about **what the framework requires of every creator**, and it
was used to cap **what this project's own default character may be**. Those are separate:

| | Bar | Who pays it |
| --- | --- | --- |
| The MineWorld default character | as faithful to `04_character_closeup.png` as practical — face, hair, freckles, garment structure and accessories included | this project, once, and the result ships as an asset |
| What the framework *requires* | the humanoid profile only: scale, axes, root convention, skeleton, retarget compatibility, glTF expectations | every creator, and it stays cheap |

A creator shipping their own world needs a rig that satisfies the profile. They do not need our
face. **The humanoid profile standardises the runtime contract, never appearance** —

```text
same skeleton  ≠  same mesh  ≠  same face  ≠  same clothes
```

— so one animation library serves characters who look nothing alike. A high-fidelity default is
therefore an example of what the framework permits, not a threshold it imposes, and `ARC-11`
already says the default style is ours and never a kernel assumption.

**Consequences.**

1. `04_character_closeup.png`'s `not_authoritative_for` list drops `facial_detail`,
   `skin_rendering` and `hair_simulation` **for the default character**. Its `authoritative_for`
   dimensions are unchanged. `ARC-4`'s mechanism — a reference declares what it is a reference for
   — is untouched and remains correct; only this one image's scoping changes.
2. `ART_DIRECTION.md` §§3 and 7 need amending where they rule out cinematic facial rendering for
   the default character. A prose section and a decision disagreeing is the same defect as two
   decisions sharing an id.
3. The prohibition on a **bespoke from-scratch character pipeline** stands, on
   `REUSE_POLICY.md` grounds rather than fidelity grounds: evaluate mature tools — MetaHuman,
   Character Creator, Blender with MPFB, image-to-3D reconstruction, CC0 groom and animation
   libraries — against quality, automation, licence, redistribution, engine portability and
   runtime compatibility. Do not build character-reconstruction technology.
4. **`DEP-8` is not relaxed.** An asset must be free to *redistribute*, not merely free to use. An
   ecosystem asset usable only inside one engine is recorded as exactly that, and engine-specific
   Presentation Packs stay isolated from portable MineWorld assets.

**Cost accepted deliberately.** A higher default bar means the default character is expensive to
reproduce and a contributor cannot casually regenerate it. That is the trade `ARC-4` refused, and
it is accepted now because the artefact is shipped rather than re-derived, and because nothing
about it reaches the framework's own requirements.

---

## ARC-20 — Visual milestones are named, reviewed as packages, and never block other work

**Date** 2026-09-27 · **Relates to** `ARC-11`, `ARC-17`, `ARC-18`,
[`ACCEPTANCE.md`](ACCEPTANCE.md), [`HUMAN_REVIEW_QUEUE.md`](HUMAN_REVIEW_QUEUE.md)

With two 3D tracks running (`ARC-18`), "the 3D character" and "the 3D scene" stop being unique
descriptions, and a review round has already been lost to an ambiguous verdict (`ARC-17`).

**Decided.**

1. **Named milestones.** `VIS-2D-1` (playable 2D default scene with an enterable interior);
   `VIS-3D-GODOT-1` (reference-matched character in Godot); `VIS-3D-GODOT-2` (integrated Godot
   slice: character, street, enterable building, interior, lighting, movement, cameras);
   `VIS-3D-UE5-1` (Unreal slice of equivalent scope); `VIS-3D-AB-1` (the side-by-side).
2. **The A/B must compare like with like.** Same reference, same demo scope, both tracks pushed.
   A stale placeholder against a polished slice measures nothing, and `ARC-18` is only decidable
   on honest evidence. Godot is pushed properly before it is judged: the current gap traces to the
   wrong mesh, hair, garments, materials and lighting, not to a demonstrated rendering ceiling.
3. **A review package, not an engineering log.** Milestone id · what changed · the exact launch
   command · real runtime screenshots · the canonical reference · a side-by-side where applicable
   · known limitations · the specific subjective questions being asked. The operator must be able
   to launch, look, walk and judge quickly.
4. **Waiting for review never blocks development.** On reaching `READY FOR HUMAN VISUAL REVIEW`,
   an agent preserves the runnable candidate, saves the screenshots, records it in
   `HUMAN_REVIEW_QUEUE.md`, **stops subjective polishing on that branch**, and moves to
   independent work. Review is a branch-level checkpoint, never a global barrier, and neither 3D
   track waits on the other.
5. **The engine choice is not the agent's.** Whether Godot remains default, Unreal is promoted, or
   both are supported is a product decision the operator makes on `VIS-3D-AB-1`.

---

## ARC-21 — The Unreal spike answered its question; the operator has paused it there

**Date** 2026-09-27 · **Closes phase one of** `ARC-18` · **Evidence**
[`references/UNREAL_ADAPTER_SPIKE.md`](references/UNREAL_ADAPTER_SPIKE.md)

`ARC-18` authorised a spike to answer two things. Phase one answered both, and one of them came
back differently from how the decision assumed.

**1. The architecture claim holds.** No kernel, contract, System, World Pack, persistence or
networking change is required to add a non-Godot client. This was not concluded from reading: a
283-line Python client using only the standard library — no engine, no WebSocket library, no SDK,
no MineWorld code, RFC 6455 spoken by hand — joined `worlds/social-cafe`, was told
`talk → too_far_away`, submitted `arrive`, watched the server recompute the affordance to
`available`, had `talk` accepted, and collected nine distinct refusal codes. Reproduced
independently before this entry was written. A grep of the contracts, kernel, server and systems
for renderer vocabulary returns seven hits, all in doc comments, all saying the concept is
excluded; none in a type, field, variant or function name.

**A load-bearing property nobody had recorded.** `CORE_CONCEPTS.md` §6.1 defines yaw by naming its
two reference axes — "from +y toward +x" — rather than as a rotation sense about the vertical.
That makes it handedness-free, so the conversion into a left-handed, Z-up, centimetre engine is one
axis swap and one factor of ten with **zero sign flips**, cleaner than Godot's. A later edit
"simplifying" it to "counter-clockwise about +z" would remain true for Godot and silently become a
trap for every left-handed client. It is now written down as load-bearing.

**2. Unreal's advantage is much smaller than the decision assumed, and the reason is licensing.**
Epic's asset ecosystem was the whole of the claimed shortcut, and none of it can enter an MIT
repository — see the `DEP-8` amendment above, which generalises past Unreal entirely. Two further
findings narrow it: Unreal's runtime import does not support skeletal meshes or animation, so a
rigged glTF character must be cooked in rather than installed as a pack, which cuts against
`CLAUDE.md` §1's extension model; and measured against `ARC-17`'s hard-fail list, MetaHuman
addresses six of nine categories well and **three not at all — outer-garment category, defining
garment structure, identity-defining accessories — which are exactly the three
`VIS-3D-GODOT-1` failed on.** It solves the half that was not the problem. Environment cost is
unchanged by engine: box geometry dressed in CC0 materials is the same Blender work either way.

**Decided by the operator: do not install Unreal; return the effort to Godot.** The remaining prize
is a better out-of-the-box lighting start, worth roughly a week, against a client whose output can
only leave the building inside a cooked build. `ARC-18` stands unamended in principle — Godot
remains the reference renderer, a second renderer would be an additional Presentation Adapter, and
the outcome of that question is still open. `VIS-3D-UE5-1` and `VIS-3D-AB-1` are parked, not
cancelled, and the spike document is the record that restarting is cheap.

**`ARC-18`'s cost summary was wrong and is corrected in place.** "Free below $1M revenue" describes
the *seat-subscription* exception, measured on company revenue over twelve months; the *royalty*
exemption is lifetime and per product. A client depending on Engine Code at runtime is a Royalty
Product and needs no seats at any company revenue. Epic names the confusion itself.

**Method note, recorded because it nearly produced a wrong answer.** The spike's first verdict
rested on a clause it had quoted correctly and which was live and current — and which governed only
pre-5.6 MetaHuman. Epic runs two licence regimes in parallel and the older one is still published.
The question to ask of a clause is not only *is this current?* but *which path does this govern?*
Epic's CDN also still serves a 2022 EULA that a search returns first and that contains none of the
words "Seat", "Launch Everywhere", "3.5%" or "Fab".

---

## ARC-22 — The default cast stays inside the range one albedo can carry

**Date** 2026-09-27 · **Relates to** `ARC-11`, `ARC-9`, `DEP-8` · **Scope** the MineWorld default
World Pack only

The 3D crowd is built by tinting **one** photographic skin albedo per instance, because the CC0
Vitruvian set ships a single body texture and a single head. `clients/3d-spike/scripts/npc.gd`
already records the limit at the palette itself: *"pushed further they read as paint rather than
as people."* The palette nonetheless ran to `Color(0.63, 0.49, 0.39)`, well past that point.

**The technical fact, stated precisely because it is easy to restate wrongly.** Tinting an albedo
changes its colour and nothing else. Facial structure, hair geometry, and the way light behaves in
skin all stay whatever the source asset was. So a tint far from the source does not produce a
person of a different ethnicity — it produces the **same** person painted a different colour, and
it reads as exactly that. The defect is in the method, not in the range being attempted.

**Decided: the default cast's skin and hair palettes stay inside the range this one albedo carries
convincingly** — broadly fair European through East Asian — until there is a second texture set
and a second head to support anything wider. Applies to the 3D crowd, the 2D generated cast, and
generated textures under `ARC-9`.

**What this decision is not.** It is not a finding that a wider cast is expensive in itself, and a
later agent must not cite it that way. The cost is in **asset production** — a second albedo, a
second head mesh, matching grooms — and `ARC-19` already establishes that the default character's
asset cost is paid once by this project. When a second texture set exists, widening the palette is
a palette change and needs no new decision.

**It binds nothing but the default.** `ARC-11` governs: the default style is the operator's taste
and never a kernel assumption. Nothing in the contracts, the kernel, the loader or the renderer
bindings knows what a skin tone is; a World Pack ships its own Presentation Pack and its own cast
with no reference to this palette. **If a later change makes this palette hard to replace, that
change is the defect, not this record** — the same guardrail `ARC-14` carries for the 2D style.

**Do not widen the palette back on your own initiative.** Its narrowness is deliberate and
measured against a known asset limit. Widening it is an operator decision, and it should be
prompted by a second texture set existing, not by the palette looking short.

---

## ARC-23 — Locate before counting: an instrument must be shown to see what it measures

**Date** 2026-09-27 · **Relates to** [`VISUAL_FIDELITY.md`](VISUAL_FIDELITY.md) §8,
[`ACCEPTANCE.md`](ACCEPTANCE.md), `ARC-17` · **Applies to** every verification in this
repository, not only visual ones

This project has produced the same defect seven times, in code, in tests and in review. A number
was trusted because it was plausible, and it turned out to be measuring something adjacent to the
thing it was named after.

| | The instrument | What it actually measured |
| --- | --- | --- |
| 1 | A face-morph sweep taking `max abs(x)` across the head at a height | the neck under the chin, the ears beside the cheeks — **0.00% for every chin and cheek morph**, indistinguishable from the dead end that would have justified replacing the head |
| 2 | An inverted-face count, to explain holes in a garment | the armpit at `z 1.31–1.38`, a legitimate fold; the visible holes are the outer shoulder at `z 1.42–1.47`. Four optimisation sweeps ran against it |
| 3 | The same count, compared against the *previous* step | zero for a surface already inside out — the reassuring number was an artefact of the comparison |
| 4 | A review contact sheet at 620 px per tile | read as "nothing changed" when both the footwear and the roofline had changed |
| 5 | A third-person frame with a 60 px head | a face, judged at a scale where a face cannot be judged |
| 6 | A clamp bound computed as `PLAYER_SPEED * MAX_SIM_DT` | itself — raising the constant under test raised the bound, and the assertion passed while the player crossed 2.4 m in one frame |
| 7 | A per-frame step check as a multiple of the median | unusualness, not displacement — it failed on a *correctly* clamped hitch |

**The decisive observation is (1): a measurement that cannot see the thing it measures reports no
effect**, and "no effect" is exactly what a genuine dead end looks like. A broken instrument does
not announce itself; it returns a clean, confident, wrong number.

**Decided.**

1. **Locate before counting.** Before an aggregate is trusted, show it is reading the right
   region: print where the extrema are, which elements contributed, the bounding box of what was
   selected. A count with no location behind it is not evidence.
2. **A bound is never derived from the quantity under test.** Fixed literals, or a bound derived
   from the requirement rather than the implementation. Otherwise the test moves with the defect.
3. **Measure the property the claim names**, not a proxy that correlates with it. "No frame
   displaces the player more than this" is not "no frame is unusual".
4. **Evidence at the resolution of the claim** — `VISUAL_FIDELITY.md` §8, which is this rule for
   images specifically.
5. **A sub-item `PASS` is a claim and needs its own evidence.** An honest overall verdict does not
   license unverified per-item claims beneath it, and two were asserted here against frames that
   contradicted them.

**The tell worth teaching.** In (1) and (2) the defect surfaced only because the numbers refused
to behave — a sweep reporting exactly 0.00% everywhere, an optimisation bouncing 82–127 with no
trend. The agent that found both named the lesson better than the rule does: *"That is luck, not
method."* **Non-convergence and implausible uniformity are evidence about the instrument, not
noise to tune through.** When a number will not settle, stop optimising against it and go and look
at what it is reading.

**Why this is a decision and not a style note.** Six of the seven were caught, but every one was
caught late and two came within a step of a wrong architectural conclusion — replacing a character
head that did not need replacing, and shipping a movement clamp that did not clamp. The cost of a
broken instrument is not a wasted hour; it is a confident decision made on a clean number.

---

## ARC-24 — A stable visual candidate is previewed early; acceptance stays strict

**Date** 2026-09-29 · **Refines** `ARC-17`, `ARC-20` · **Governs**
[`VISUAL_FIDELITY.md`](VISUAL_FIDELITY.md) §9.1

**The operator's direction:** as soon as the visual track has a relatively stable candidate, show
it — the operator's judgement is what makes a visual result acceptable, so the operator has to be
able to see it.

`VISUAL_FIDELITY.md` §9 forbade requesting review while the candidate was not recognisably the
reference. That was right for **acceptance** and wrong for **steering**: across two rounds every
candidate was withheld, correctly under the rule, and the operator saw nothing while effort went
into directions only they could have corrected.

**Decided.** Two reviews, never confused. A **preview** happens as soon as a candidate is stable —
launches cleanly, renders without broken geometry or missing textures, and the next change is
refinement rather than repair — and asks *is this the right direction, and what is most wrong?*
It can never end in `ACCEPTED`. An **acceptance review** still requires §9's answer to be yes, and
only the operator marks anything accepted (`ARC-11`).

**What does not relax.** The hard-fail categories of `ARC-17` must pass before a preview too; a
preview is not a route for a categorically wrong candidate. The banned comparative language stays
banned. And instability is not previewed: torn meshes, featureless faces and unloaded textures are
repair, which is the agent's to finish.

---

## ARC-25 — A world's state is its journal re-executed; its history is the fact log

**Date** 2026-09-30 · **Implements** [`ARCHITECTURE.md`](ARCHITECTURE.md) §§7–8 · **Relates to**
`INV-11`, `INV-15`, [`MVP.md`](MVP.md) §9 `AC-6`, `AC-12`, `AC-15` §9.1, `DEP-1`, `DEP-2`, `ARC-15`,
`ARC-23` · **Design** `.structured-coding/plans/mvp0/step-06-persistence.md` (S5, PR 07)

**Problem.** A persisted world must be rebuilt after its process dies. `ARCHITECTURE.md` §7 described
that as `snapshot + events after it`. Read literally — re-apply the logged facts through the reducers —
it cannot be done with the System contract as merged in S3 and S4, for four independent reasons found
in source:

```text
resolve writes      System::resolve is handed a writable view; a write made there is caused by a
                    request and recorded in no fact
processes           start, end, suspend, reschedule and set-state are writes in any hook, with or
                    without a fact
time                a process wake and a deferred fact fire because the clock reached an instant;
                    nothing in the log says the world advanced
react emits         react, wake and interrupt both write state and return further emissions, so
                    re-applying a fact would emit its logged consequences a second time
```

**Options considered.** (1) Redesign the System contract for pure fact replay: `resolve`, `wake` and
`interrupt` read-only, process lifecycle and time expressed as facts, `react` split into an apply half
and an emit half. (2) Record the *inputs* that drive the deterministic pipeline, re-execute them, and
check the result against the fact log.

**Choice: (2).** A persisted world is three append-only things in one SQLite file, committed together
one revision at a time:

```text
facts       every EventEnvelope in EventId order      HISTORY: what happened (INV-11)
journal     every input that moved the world          what the world was asked, and when
snapshots   the whole of a world's state at some      a checkpoint of the function below,
            revisions                                 never an authority on its own
```

and the two questions have two answers:

```text
What happened?              the fact log. Append-only, never derived, never rewritten. Biographies and
                            projections derive from it (INV-4).
What is the world's state?  the kernel's one pipeline applied to the journal from genesis. A restart
                            loads the newest snapshot at or below the head and re-executes the journal
                            after it; every re-executed input must regenerate its logged answer and its
                            logged facts byte for byte, or the load is refused (ReplayDiverged).
                            Verification re-executes from genesis and requires every stored snapshot to
                            equal the state the history produces at its revision.
```

**Journaled inputs.** Genesis, always — revision 1, recording the assembled world before genesis, the
instant and the genesis facts (`ARC-15`), so genesis is re-run rather than trusted. Every
`dispatch(intent, at)`, whatever the answer — accepted, rejected, unavailable, or a system fault —
because a dispatch moves the clock even when refused and because journaling every request is what keeps
an `ActionId` unique across a restart. An `advance_to(until)` only when it fired at least one instant or
faulted: idle advances only move the clock, every later input carries its own instant, and the kernel's
checks pass identically without them.

**Revision.** A `WorldRevision` is the position of an input in the journal; genesis is 1. It is
committed before anyone is told it, and it is what a client is told a persisted world is at — the
referent of `MVP.md` §9.1's *same persisted state revision*.

**Why (2), and the axis is the pipeline.** Option 1 changes two merged contracts and introduces a
second way to reduce state — exactly the drift `kernel/src/dispatch.rs` was written to prevent, where
two recording paths would be two accounts of one world. Option 2 adds one table and keeps one pipeline.
It is sound only because the pipeline is deterministic, which is not assumed: S4 replays 300 simulated
days byte for byte from one seed. And it is the form recorded cognition needs anyway: an LM controller's
decision reaches a world only as an `ActionIntent`, so journaling intents is what makes an LM-driven run
replayable without the model (`ARCHITECTURE.md` §7).

**Why this is still event sourcing.** The fact log remains the one historical truth and the check on
every reconstruction; no state is accepted that would have produced a different history. Every state
change is still caused by an `ActionIntent`, a `Process` or an `Event` (`INV-15`) — the journal records
the first and the clock that drives the second. State that existed only in memory — a system breaking
`INV-7` through interior mutability, a global or a wall clock — makes re-execution differ, and the first
differing fact refuses the load.

**Versions are refused, never guessed.** A save format, system version, component schema or
composition (the installed systems in registration order, which is the reduction order) that differs
from the running code is refused by name. An event schema changed without a system version change is
still caught, as a byte divergence on replay. Migration is a later step, taken when a pack first needs
one (`MODULE_SPEC.md` §9 rule 2).

**Accepted limitations.** Reconstruction costs re-execution of the journal tail since the last
snapshot, not a load of facts. A restored world's clock is the instant of its last revision: idle
seconds after it are not persisted, and world time does not pass while no process hosts the world.
The log is kept whole; compaction and snapshot pruning are later work.

**Note 2026-10-10 (`ARC-81`).** Snapshot pruning is that later work, done for snapshots only:
`ARC-81` retains genesis, an anchor every 64 scheduled snapshots and the newest two, and stores each
snapshot as a zstd frame of its JSON encoding. Everything above still holds unchanged — the fact log
and the journal are kept whole, a restart restores the newest snapshot at or below the head, and
`verify` re-executes from genesis and compares every snapshot the save still holds. Compaction of the
log remains later work (SR-b), and dropping journal or facts before a checkpoint stays excluded.

**Note 2026-10-13 (`ARC-89`, SR-b, `SAVE_FORMAT` 4).** The log is now compressed, and still whole: the
journal and fact rows of each completed span move, byte for byte, into one sealed block of two zstd
frames, in the transaction of the anchor commit that seals them. Every guarantee above holds unchanged:
no fact or journal row is dropped, rewritten or re-encoded; a restart restores the newest snapshot and
re-executes the journal after it (which is always live, never sealed); replay and `verify` compare every
re-executed entry and fact with the logged bytes, decoded from a block where the row is sealed; and a
block's compressed bytes are a storage representation from which no comparison is argued. The
accepted limitation "the log is kept whole" now reads "kept whole, compressed in sealed blocks".

---

## ARC-26 — Movement decides, presence owns: an event type's owner is its vocabulary and its reducer

**Date** 2026-09-30 · **Implements** [`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §6,
[`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §§11, 13 · **Relates to** `INV-7`, `INV-9`, `INV-10`,
[`MVP.md`](MVP.md) §9 `AC-2`, `ARC-15`, `ARC-23`, `ARC-25` · **Design**
`.structured-coding/plans/mvp0/step-07-movement.md` (S6, PR 08)

**Problem.** S6 adds movement. `PresenceSystem` already owned where people are and provided `arrive`,
an action that put a person anywhere and checked only identity. Two questions followed. Where does
movement live, given that `ConversationSystem` depends on presence and the registry refuses to
disable a system another enabled system depends on? And how does a movement system change presence's
state, given that presence must not know movement exists and movement must read presence's state to
validate a move — a dependency in each direction is a Cargo cycle?

The audit also found an existing contradiction. `contracts/src/event.rs` documented `Event::OWNER` as
"the system that emits it", while `Dispatcher::record` (`kernel/src/dispatch.rs`) only ever checked
that the *running* system's own declaration lists the type — never that it is the type's owner. The
specification and the kernel already disagreed (`CLAUDE.md` §2.1(4)).

**Options considered.**

```text
(a) movement inside PresenceSystem      disabling movement disables presence; refused while conversation
                                        depends on it, so AC-2 for movement is unreachable
(b) MovementSystem owns location        presence and conversation would depend on movement; removing
                                        movement removes location and perception
(c) movement emits its own fact and     presence imports movement's event while movement imports
    presence reacts                     presence's component: a cycle, and presence names another pack
(d) movement keeps its own positions    two truths about one person's position
(e) movement decides; it states         movement → presence, one way; presence keeps the only state
    presence's own Arrived, built by    and never learns movement exists
    presence's checked constructor
```

**Choice: (e), and `arrive` is retired.**

1. **`Event::OWNER` is the event type's vocabulary owner**: the system that defines its schema and its
   public constructor, and **the only system that reduces it into owned state**. It is not merely the
   only system permitted to emit it. Provenance names the system that *stated* a fact; `OWNER` names
   whose vocabulary it is. They differ exactly when one system decided what another records.
2. **Another system may state a fact of that type only if it** declares the emission, **depends on the
   owner** (both refused at installation otherwise — `KernelError::EmittedEventOwnerNotADependency`),
   and builds the payload through the owner's public constructor.
3. **The owner still decides.** The owner's constructor checks the fact against the world and refuses
   one its state may not take; its reduction checks again, writes nothing, and fails with
   `KernelError::FactRefusedByOwner` if a fact built some other way reaches it. Movement decides
   whether a *move* is legal; presence decides whether its *state* may take the value.
4. **One movement path.** `arrive` is removed. A distance rule beside an unrestricted relocation is
   not a rule: `TooFarAway` would hold for the action and not for the world. Initial placement is a
   genesis fact (`ARC-15`), so nothing needs `arrive` to exist.

The resulting split:

```text
PresenceSystem   owns Presence and present-in; owns Arrived and PersonEnteredPlace; provides no action
MovementSystem   depends on presence; provides `move`; owns Passages (which places open onto which, and
                 where the doorway is) and PassageOpened; states presence's Arrived when a move is legal
```

`move` is decided entirely by `MovementSystem::validate` through `SpatialRequirement::evaluate`: a
stride of at most `MAX_STRIDE` (2 000 mm) inside a place; into another place only through a passage,
within a stride of the doorway on both sides; otherwise `TooFarAway`. `PersonEnteredPlace` is
presence's, emitted while it reduces an `Arrived` that changes a known place, because occupancy is
presence's state whoever caused the arrival.

**Why this does not weaken single ownership (`CLAUDE.md` §4 rule 1).** That rule is about mutable
state. `Presence` is still written by exactly one thing — presence's own reduction — and presence can
refuse the value. Exposing an owner's vocabulary to declared dependents is the smallest acyclic form of
"emit a fact and let the owner decide".

**Room for travel.** Travel between places that do not open onto each other is a `Process` that takes
simulated time (`ENGINEERING_RULES.md` §6). A future travel system depends on presence and states
`Arrived` when its process ends, under this same rule. `move` refuses exactly the requests such a
system would accept.

**The client reporting rule.** A per-request stride bound is correct only if clients report often
enough: a 3D client jogging at about 2.6 m/s that reports once a second would be refused for moving
legally. So clients **report before travelling `MAX_STRIDE` since their last accepted position**
(`server/PROTOCOL.md`, `clients/protocol/ADOPTION.md`).

**Accepted limitations.** The stride bounds one request, not requests per second: there is no speed
model, which needs time accounting finer than `WorldTime`'s one second. Walls inside a place are not
evaluated (line of access needs geometry no layer owns, `DD-7`). Passages are stated at genesis and are
always open; doors are a later system. `MAX_STRIDE` is a constant until world configuration exists
(S7). Saves made before this decision are refused by name (`SAVE_FORMAT` 2), not migrated.

**Note, 2026-10-06 (S7, step-08 §10.1 Q5).** S7 did not make `MAX_STRIDE` world configuration. No
pack yet needs another value, and a configuration mechanism built for one constant is the premature
abstraction `CLAUDE.md` §4 rule 11 forbids. It stays a published constant until the first pack that
needs a different stride, which introduces System Pack configuration (`MODULE_SPEC.md` §9) then.

**Note, 2026-10-09 (S15, PR 12n-1; step-11 §21, SD-N1, SD-N2) — walking is not the travel Process.**
`movement` gains a second scale of the same spatial model inside a place and across adjoining places:
`walk-to` (go to a destination) and `walk-step` (the next stride of my walk), with the walk held as
`movement`'s `Walking` component (`ARC-75`). It is not the "Room for travel" above: a walk takes no
calendar time and starts no `Process`; each stride is one request, at most `WALK_STRIDE` (1 340 mm),
checked by the same rule as `move` and stated through presence's `arrivals()`. Travel between places
that do not adjoin by a passage chain remains the later travel system's, a `Process` in calendar time.
`move` is unchanged and still one stride: a walker's own `move` ends their walk.

---

## ARC-27 — A headless run is a pace schedule over stateless seeded controllers

**Date** 2026-10-06 · **Implements** [`MVP.md`](MVP.md) §9 `AC-6`, `AC-11`, `AC-12` ·
[`MODULE_SPEC.md`](MODULE_SPEC.md) §8.1 `run` · **Relates to** `INV-1`, `INV-13`, `ARC-23`, `ARC-25`,
`DEP-6` · **Design** `.structured-coding/plans/mvp0/step-08-headless.md` (S7, PR 09)

**Problem.** `AC-11` asks that *a seeded rule-based configuration* run hundreds of simulated days with
no renderer and no model, and `AC-12` that the same seed reproduce the run. Three facts found in
source made the obvious reading empty. The real System Packs (presence, movement, conversation) are
time-inert: none starts a process or defers a fact. The only controller, `RuleController`, answers
whoever spoke to its Person and otherwise does nothing. And with no client connected nothing else
submits a request. A headless café would therefore advance an idle clock for three hundred days and
record its five genesis facts — a history perfectly stable and perfectly reproducible, which is the
clean number from an instrument that cannot see that `ARC-23` forbids trusting.

A second problem: the server's agent path cannot be reused. A hosted world's time follows the wall
clock, observations are delivered with `try_send` and dropped under load, and the agent is a task.
Which observation a controller decides on is a matter of timing. And the reactive controller keeps
*which line it has answered* in memory, so a restarted one answers its last line again (step-06
`F-13`) — a killed-and-resumed run would then differ from an uninterrupted one.

**Options considered.**

```text
(a) drive the world through the server, as --agent does     wall-paced and timing-dependent
(b) a scripted input file                                    reproducible, but a replay of a script,
                                                             not a rule-based configuration
(c) a stateful seeded controller whose memory is persisted   controller bookkeeping becomes world
                                                             state, against INV-1's separation
(d) a stateless seeded controller on a fixed pace schedule   chosen
```

**Choice: (d).**

1. **The driver.** `mineworld run` steps the world itself, synchronously, on one thread, calling only
   what the server calls: `advance_to`, the pack's perception, `ActionIntent::allocate`, `dispatch`.
   A run's `ActionId`s are allocated in order from 1, and a resumed run's from one past the
   journal's highest.
2. **The pace.** Seat *k*, in the pack's seat order, is consulted at `genesis + k + m·P` simulated
   seconds, `m = 0, 1, …` (P = 600 s; refused if the pack has P seats or more). No two seats are ever
   consulted at one instant, so the order of decisions inside an instant never arises.
3. **The controller is stateless.** `PacedRuleController::decide(&self, &Observation)`: its decision
   is a pure function of its seed, its pace and the observation, which the compiler enforces. Every
   choice is a SplitMix64 mix of `(seed, observer, instant)`. It answers the newest line a speaker
   said to it *since its previous consult* — a line heard at `h` is answered at `t` only if
   `t − P < h ≤ t` — and otherwise takes seeded initiative: greet, approach, wander, use a doorway.
   Because only consults dispatch and no two share an instant, each line lies in exactly one of its
   listener's windows: *answer once* holds with no record of having answered, and a controller built
   after a restart decides exactly as the one that died.
4. **Age, not duration.** `--days N` runs the world until it is N days old, so the same command run
   again after a crash completes the same world.

**What `AC-12` covers, and what it excludes.** Covered: the facts, the journal and the snapshots of a
run, byte for byte, as a function of the pack, the seed, the age, the pace and the code. Excluded by
name: the world instance identity in a save's manifest (allocated from the wall clock and the
process, because two runs are two worlds — `server/PROTOCOL.md` §5), and the elapsed wall time `run`
prints on its own line. Equality is always shown by comparing bytes; the printed fingerprint is for
reading, never evidence.

**What it does not cover.** A hosted world's `--agent` keeps the reactive controller and keeps `F-13`:
it has no fixed consult schedule, receives many observations per simulated second, and needs its
memory. Its remedy is a perception or cognition change (S10).

**Accepted limitations.** A line whose window passes while `talk` is unavailable is never answered —
the listener missed the moment. The controller's speech is formulaic, as a rule's is; interpretation
is cognition's. Places have no extent, so wandering is bounded only by the pull of nearby people.

**Note, 2026-10-07 (S8, `step-09-social.md` C3).** Three changes, none to the decision itself:

1. **P is 900 s**, not 600. S8 made `social-cafe` a town of twelve people with eleven seats, and one
   300-day debug run took 77 s at 600 s. The step-08 rule is that the pace rises before the days fall,
   so it rose: 51–55 s. A pack with P seats or more is still refused.
2. **A door is a seeded choice kept for six simulated hours**, a draw over `(seed, observer, instant ÷
   21 600)`. That is still a pure function of the observation. Before, *leave* took the first disclosed
   passage, and on a street of five doors everybody walked into the lowest-numbered place and never
   came back. In a place with more than one door, which in practice is a street, walking on is the
   likelier draw.
3. **A proposed stride never exceeds `MAX_STRIDE`.** The stride arithmetic divided by the *floored*
   square root, so a stride came out a fraction of a millimetre over 2 000 mm and was refused
   `TooFarAway`. It now divides by the root rounded up. S7's `TooFarAway` refusals, read at the time as
   crossings at a rounded 2 000 mm, were this defect, and the S7 test could not see it because it
   measured strides with the same floored root (`ARC-23`). With the fix, a 300-day run has no refusals
   at all.

**Note, 2026-10-09 (S15, PR 12n-2; `step-11-bodies.md` §21 SD-N15, SD-N16; `ARC-75`).** People walk
by asking movement for a walk (`walk-to`) and then for its strides (`walk-step`), one per request. The
run paces those strides; nothing about the decision itself changes:

1. **Step consults.** As well as the lattice `genesis + k + m·P`, seat `k` is consulted at `genesis + k
   + n·RUN_STEP` (`RUN_STEP` = 30 world seconds, a divisor of P) whenever its person has a `Walking`
   at that instant. The run reads that from world state through movement's `is_walking`, so a resumed
   run schedules exactly the step consults the dead one would have, which is this decision's restart
   property. A seat that is not walking is not consulted, and the clock is not advanced for it: a
   `Walking` is only ever created by a request, so a seat that was not walking before the instant is
   not walking at it. `RUN_STEP` is notional (step-19 §4.6: `run` has no time scale). A 20 m walk is
   15 strides, 7½ notional minutes, rather than ten lattice consults.
2. **At a step consult the controller is asked `step`**, a separate pure function that returns only
   `walk-step`, takes no draw and never answers. `decide` is still asked only on the lattice, once per
   pace, so "answer each line once" holds. A lattice instant is always a step instant. The one consult
   there is `decide`, and when `decide` returns nothing for a walker, `step`, so that a walker who
   rolls "stand" keeps walking.
3. **A pack with `RUN_STEP` (30) seats or more is refused**, naming `RUN_STEP` (it was P). Residues
   mod 30 keep every instant to one seat. The largest pack has eleven seats.

**Note, 2026-10-13 (S24, PR DL-a; `pr-dl-a-decision-list.md` SD-DLa-1, SD-DLa-2, I-DLa-3; `ARC-85`).**
"Stateless seeded controllers" is generalised from the paced rule controller to **every in-process
decision model**. The headless run now consults each seat through the model its decision list resolves
(`ARC-84`); a model is a `DecisionModel` (`cognition/decision`) whose `decide` and `step` take `&self`,
its typed parameters and a `Consult { seed, pace, observation }`, so what it attempts is a pure function
of `(parameters, seed, pace, observation)` and a model rebuilt after a restart decides exactly as the one
that died. The pace travels in `Consult` because it is the answer window, host context a model cannot
know otherwise. Model crates forbid interior mutability by their own `clippy.toml`, and a template test
runs every installed in-process model twice and across a kill-and-resume against the save's bytes.
`Draw` and `mix` moved verbatim from `paced.rs` to `mineworld-decision`, salt included; the rule
controller, `rule`, is the first model and the default, and with no decision list every seat is `rule`
and every byte as before. The lattice, `RUN_STEP` and everything above are unchanged.

---

## DEP-11 — The CLI's argument parsing: `clap`

**Date** 2026-10-06 · **Status** selected, integrated in PR 09 · **Design**
`.structured-coding/plans/mvp0/step-08-headless.md` §10.1 Q12

**Problem.** `mineworld` parses its own command line. Until S7 it had three subcommands and four
options, parsed by hand, and `tools/cli/src/main.rs` recorded when that would stop being right:
*"`clap` is the right answer the day `create` and `inspect` arrive with real option surfaces."*

**The trigger that was met.** S7 adds `run` (`--headless`, `--seed`, `--days`, `--save`), `inspect`
(`--last`) and `create`: six subcommands and ten options, with typed values (`u64`, a path, a socket
address), required and repeatable options, and help text per command. Hand parsing at that size is
re-implementing commodity infrastructure (`REUSE_POLICY.md`), and keeping it would have meant moving
the recorded trigger after reaching it.

**Options considered.** Hand parsing (status quo: grows with every option, and every refusal text
is ours to keep consistent); `pico-args` and `lexopt` (small, but still leave usage, help and
validation to us); `argh` (derive-based and small, but its conventions are Fuchsia's and its
ecosystem thin); **`clap` 4 with `derive`** (the ecosystem's standard, maintained, typed values,
generated help and errors that name the offending argument).

**Choice: `clap` 4 with the `derive` feature.** Dual MIT / Apache-2.0.

**Isolating interface.** `tools/cli/src/main.rs` alone: one derived `Cli` type and its subcommand
enum, turned into the command's own plain values before anything runs. No `clap` type appears in
any other module or crate, so replacing the parser is a change to one file.

**Accepted limitations.** Compile time and binary size grow by the parser's; the command's help
and error wording become `clap`'s format rather than hand-written prose.

---

## ARC-28 — Relationships are a `knows` edge with values its owner reduces from other systems' facts

**Date** 2026-10-07 · **Implements** [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §§4.4, 9, 13.1 ·
**Relates to** `INV-7`, `INV-13`, `DD-6`, `ARC-25`, `ARC-26`, [`MVP.md`](MVP.md) §9 `AC-2` ·
**Design** `.structured-coding/plans/mvp0/step-09-social.md` SD-5 … SD-9, §10.1 Q5/Q6, §4.2 (S8,
PR 10b)

**Problem.** S8 adds the first System Pack whose state changes *only* because of what other packs
did: people get to know each other because they spoke, accepted an invitation, or spent an hour
together. Four questions follow:
1. Where per-relation state lives, when the specification says "a component keyed by the triple"
   (`contracts/src/relation.rs`, `DD-6`) and the kernel keys components by `EntityId` alone.
2. Whether the pack must depend on the packs whose facts it reads.
3. How it reads their payloads.
4. Which changes are worth a fact of their own.

**Options considered.**

```text
state     (a) a kernel change: components keyed by a Relation          kernel learns a new key kind
          (b) a component on the `from` Person, keyed by the counterpart  chosen — same identity, no
              under the one relation type this pack declares               kernel change
dependency (a) depend on conversation and group-activity                 disabling either is refused
                                                                         while relationships is on
          (b) no system dependency; subscribe                             chosen
decoding  (a) a local struct shaped like the other pack's payload        drifts silently; bypasses the
                                                                         schema-version refusals
          (b) the owner crate's published event type, through            chosen
              EventRecord::payload_for
facts     (a) one per value change (~+43 000 per 300 days)               doubles the log, names nothing
          (b) one when a derived level crosses a boundary                 chosen (Q5)
```

**Choice.**
1. `RelationshipsSystem` declares a directed relation type `knows` (Person → Person, no self edges)
   and owns `Acquaintances` on the `from` Person: counterpart → `RelationshipValues { familiarity
   0..=1000, regard −1000..=1000, exchanges, activities_shared, first_met, last_contact }`. The edge
   and the entry are written together, in one reduction, by the one owner. That realizes `DD-6`'s
   "keyed by the triple" without a kernel change.
2. **No system dependency.** Subscribing is not emitting. `ARC-26` requires a dependency only to
   *state* another system's vocabulary. A world with relationships and no conversation installs; it
   simply hears no speech. `ARC-26` also calls the owner "the only system that reduces it into owned
   state". Read with `CORE_CONCEPTS.md` §13.1, that sentence is about the state the fact describes:
   only presence turns `arrived` into a `Presence`. It does not stop another system reacting to the
   fact by writing **its own** state, which is how `EconomySystem` answers `WageDue`. Relationships
   writes only `Acquaintances` and `knows`, never `ConversationHistory`. This sentence records the
   reading, so the two decisions do not appear to disagree (`CLAUDE.md` §2.1(4)).
3. **Decoding goes through the owner's published type**, a Cargo dependency on its vocabulary and never
   a registry dependency. A local mirror of another pack's payload is refused in review, because it
   would bypass `EventSchemaTooNew` / `EventSchemaOutdated`.
4. **Facts at level crossings only.** `became-acquainted` when an edge first forms (once per
   direction); `relationship-changed { from, to }` when the level (`Acquaintance`, `Friendly`,
   `Friend`, `Close`, from familiarity and regard) changes, up or down. Each is caused by the fact that
   formed or crossed it. Fine-grained values are reductions of logged facts, so a replay reproduces
   them (`ARC-25`).
5. **It provides no action, runs no process and has no wake.** Its state changes only by reducing
   `spoke`, `invitation-accepted`, `invitation-declined` and `group-activity-ended`. That is
   `CLAUDE.md` §4 rule 1 in its plainest form: the owner reacts, nobody else writes.
6. Values are disclosed to their holder only (`INV-13`): how Alice regards Bob is Alice's to know.

**Accepted limitations.** Values never decay, so a long-running world's social graph saturates: every
pair that keeps meeting reaches its top level within weeks and stops changing. That is a living-world
gap, recorded for a later step, not hidden by the tests (`step-09-social.md` QB-2). The constants are
published and not world configuration yet (the `ARC-26` note's rule). Only `knows` exists; romance and
typed relationships are later packs (`MVP.md` §4).

**Note, 2026-10-13 (S24, PR FG-a; `ARC-86`) — values may now decay, by configuration.** The limitation
"values never decay" is closed for a world that configures it: `configure/relationships.yaml` may name a
`DecayCurve` and a time constant and floor for familiarity and for regard, read lazily from the stored
`(value, last_contact)` (`ARC-86`), and a per-person cap `known_at_most` (default 150) evicts the lowest
current familiarity and states `acquaintance-forgotten`. Item 4's "`became-acquainted` … once per
direction" becomes **once per formation**: a counterpart evicted by the cap and met again forms a new
entry and is stated again. The defaults (`none`, 150) leave every reference world's facts unchanged, so a
world that configures nothing still saturates as described above. `relationships` is version 3.

---

## ARC-29 — A biography is a projection of the fact log, selected by what each pack declares biographical

**Date** 2026-10-07 · **Implements** [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §§5.2, 5.4 · **Relates to**
`INV-4`, `INV-11`, `ARC-25`, [`MODULE_SPEC.md`](MODULE_SPEC.md) §8.1 · **Design**
`.structured-coding/plans/mvp0/step-09-social.md` SD-12, §10.1 Q10, §4.2 (S8, PR 10b)

**Problem.** Milestone B asks that Alice and Bob "survive a restart with their history", and
`CORE_CONCEPTS.md` §5.2 says a Person's objective biography is derived from the event log and can
always be regenerated. Something must derive it. It must not become a second account of history, and
adding a pack must not mean writing biography code.

**Options considered.** (a) A stored biography component, maintained by reducers: a second truth that
can disagree with the log. (b) Per-pack narrator functions returning typed entries: more expressive,
and an abstraction with one shape so far (`CLAUDE.md` §4 rule 11). (c) **A generic projection over
fact envelopes**, with each pack declaring which of its own event types are biographical.

**Choice: (c).**
- An entry is `{ at, event id, event type, place, counterparts }`, read only from the envelope's
  kernel fields.
- A fact belongs to Person P's biography when P is among its subjects or participants **and** its
  type is in the composition's biographical set.
- Each System Pack exports `pub const BIOGRAPHICAL: &[EventTypeId]`, its own judgement over its own
  vocabulary, and the World Pack catalog aggregates them (`Capability::biographical`). A pack that
  declares none contributes none: presence, movement and conversation contribute nothing in S8.
- `mineworld biography <world> --save DIR --person KEY` reads a save's fact table and nothing else. It
  never resumes or writes the world, and every line carries its event id (§5.4).

**Why it is honest.** The biography is never stored, so it cannot drift. It is tested against the
owner packs' typed payloads rather than against the envelope rule it implements, so a biography that
invented or dropped an entry fails. It is regenerated identically from a restarted world's save.

**Accepted limitations.** L0 only: a long world's biography is long. Compression (L1–L3) is `AC-10`'s,
in S10. Generated prose is display, never state (§4.4). A pack that states a fact without naming its
people in the envelope is invisible to biographies; that is the pack's defect, not the projection's.

**Note, 2026-10-08 (S17, PR IL-b; `ARC-65` item 4, QIL-15).** The selection is now
`mineworld_sdk::interactions::biography::selected`. A fact whose owner has a configured section is
biographical when that section's consequence for the fact's roles says so (`biography: on | off`, only
where the owner allows); otherwise the compiled set above decides. `mineworld biography` therefore also
reads the save's genesis `*-interactions-configured` facts and each entity's type and tags (from the
World Pack it already reads for keys and names), and each composed capability's declared facts. It still
writes nothing and resumes nothing. With no configured section it is this decision exactly.

---

## ARC-30 — The development profile is optimized at level 1, with its debug checks stated explicitly

**Date** 2026-10-07 · **Relates to** [`MVP.md`](MVP.md) §9 `AC-11`, `AC-12`; `ARC-27`;
[`ENGINEERING_STANDARDS.md`](ENGINEERING_STANDARDS.md) §§15–16 · **Design**
`.structured-coding/plans/mvp0/step-09-social.md` §4.2.5 QB-3, §4.2.6 (S8, PR 10b)

**Problem.** MineWorld's strongest evidence is real worlds run for hundreds of simulated days:
`AC-11`/`AC-12` run `social-cafe` for 300 days three times and replay the save. In the default debug
profile (level 0), the default test loop took 272 s on `main @ 0592b3e`, 264 s of it in four
real-lifecycle test binaries, and 10b adds two systems to every run. A slow default loop gets run less,
and the long tests are the first a contributor is tempted to skip.

**Options considered.**

```text
(A) [profile.dev] opt-level = 1        every test stays in the default loop; runs get ~6× faster
(B) #[ignore] the 300-day test behind  keeps level 0, but the slowest and most important evidence
    a named gate command               becomes the easiest to skip
(C) neither                            a default loop of 6–7 minutes after 10b
```

**Choice: (A).** `[profile.dev]` sets `opt-level = 1` for the whole workspace and states
`debug-assertions = true` and `overflow-checks = true` explicitly. Those two are what make a debug build
a debug build. Stating them means a later change to the level cannot silently turn them off.

**Evidence** (this machine, `step-09-social.md` §9 E-B0, E-B8):
- `mineworld run worlds/social-cafe --headless --seed 7 --days 300` in memory: 50.3 s at level 0,
  8.4 s at level 1.
- Every printed line but the header and `wall` is identical (`diff` empty; 327 540 facts, fingerprint
  `fd0fe804108e9bf0` both). The level changes speed, not behaviour, as expected of integer-only
  simulation code (no float anywhere in the workspace).
- A clean `cargo build --workspace --all-targets`: 15.1 s at level 0, 39.4 s at level 1.

**Why it is safe.**
- The determinism claims (`AC-12`) are about the pipeline's semantics, which no optimization level may
  change in safe Rust without floating point.
- Overflow is still checked, and debug assertions still fire.
- Release builds are unaffected.
- No test left the default loop, and none changed.

**Accepted limitations.**
- A slower clean build (above).
- A debugger sees some values optimized out; a contributor who needs level 0 for one session sets
  `CARGO_PROFILE_DEV_OPT_LEVEL=0`.

---

## ARC-31 — A System Pack owns a section of an authored person or place file

**Date** 2026-10-07 · **Implements** [`MODULE_SPEC.md`](MODULE_SPEC.md) §4.1 (sections, rule 6),
[`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md) §8 · **Relates to** `INV-7`, `INV-13`, `ARC-15`, `ARC-26`,
`DEP-10`, [`MVP.md`](MVP.md) §9 `AC-2` · **Design** `.structured-coding/plans/mvp0/step-09-social.md`
§10.1 Q9, §4.3 (S8, PR 10c)

**Problem.** Authored content becomes state only as a genesis fact its owner reduces (`ARC-15`), and
until now the loader knew each such field by name: a person's `location` is presence's, a place's
`passages` are movement's. A third and a fourth arrived together — a person's routine (`schedule`)
and a person's name. Nobody owned a name, so none reached a client: the 3D slice showed only tags,
and Alice said "Earlier, person 4 said …". Adding each one as another loader field would make every
new System Pack an edit to the World Pack format and its loader, the change-amplification
`CLAUDE.md` §4 rule 5 forbids, and the pattern finding F-1 already names. The repeated concept is now
observed twice over (`CLAUDE.md` §4 rule 11).

**Options considered.**

```text
(a) one loader field per pack (`routine:`), as `location`    each new pack edits the format, the
                                                            reader and the loader
(b) free items per pack (a const, two functions) matched     lighter; the shape is unchecked by the
    by the catalog, as `biographical` is                    compiler
(c) a trait a System Pack implements, in a crate of its     chosen
    own below the packs
```

**Choice: (c).**

1. **The contract.** `mineworld-authoring` (`authoring/`) defines `AuthoredSection`, implemented by a
   System Pack:
   - `SECTION`: the key it owns, one word;
   - `CARRIED_BY`: the content files that may carry it;
   - `Authored`: the section's own type. Deserializing it *is* the owner's validation, so an
     invalid section cannot be constructed;
   - `references`: the other entities it names by key, each with the entity type it must be;
   - `seed`: its genesis facts, in its own vocabulary, built with its own codec.

   The crate depends on the kernel's contracts only. It lives below the packs because the kernel and
   contracts know nothing of authored files, and a pack cannot depend on the World Pack loader,
   which depends on every pack.
2. **The loader never learns what a section means.** It decodes a section straight from the YAML
   stream into the owner's type, so a refusal keeps the line and column `DEP-10` chose `serde-saphyr`
   for. It checks only what every section shares:
   - the owner is enabled (otherwise refused, naming the pack, as an unowned `location` is);
   - the file kind may carry it;
   - every reference is a declared key of the required type.

   An unknown key is still refused, now listing the sections this build knows beside the fields.
3. **A seeded fact must be the owner's own.** World genesis attributes a fact to its event type's
   owner and checks no dependency, so a section that seeded another pack's vocabulary would bypass
   `ARC-26`'s rule. The loader refuses it.
4. **Order.** Sections are seeded after passages and locations, whose event ids therefore do not
   move: places' before people's, each in key order, and within one file in composition order.
5. **`location` and `passages` stay fields.** Moving them would edit `presence` and `movement` and
   change refusals authors already see, so it was not the no-op the step required. They are the
   pre-seam special cases, and their move is a later candidate, made when those packs are next
   opened.
6. **Section names are one namespace.** Each pack chooses its word. No two packs may claim one, and
   none may shadow a field, which a structural test over the build's catalog enforces. This follows
   the precedent of action types, which are already one namespace across packs.

**The first two users.**

- **`name`, owned by `naming`.** It owns `DisplayName` (component `display-name`, payload
  `{ "name": … }`), seeded by a genesis `named` fact.
  - The pack is called `naming`, not `identity`, because identity is the kernel's word for
    `EntityId` (`CLAUDE.md` §2.1(3)).
  - Names are **public**: disclosed to whoever perceives the person, oneself included.
  - A controller reads a name only from its observation. When the person it means is not
    perceived, it says "someone else" — never an id.
  - Readers of a save (`mineworld biography`) use naming's published projection of its own facts.
- **`routine`, owned by `schedule`** (`ARC-32`).

**Accepted limitations.**
- The catalog is still a closed list compiled into the build (F-1). The seam removes the format
  edit, not the registration.
- Places carry no names yet.
- Names are not gated on acquaintance. That would couple naming to relationships' state, and it is
  a decision for the step that wants strangers to stay nameless.
- There is no rename action.

---

## ARC-32 — A schedule is an agenda that controllers follow, never a mover

**Date** 2026-10-07 · **Implements** [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §10 · **Relates to**
`INV-1`, `INV-3`, `INV-6`, `INV-12`, `INV-13`, `ARC-26`, `ARC-27`, `ARC-31` · **Design**
`.structured-coding/plans/mvp0/step-09-social.md` SD-13, §10.1 Q8, §4.3 (S8, PR 10c)

**Problem.** People in a living town keep a day: the café in the morning, work, the park, home. Someone
must own that day, and something must make it happen. If the owner also moved people, it would
override whoever controls them — a human included — and bypass movement's rules: a person teleported
to work never crossed a street.

**Options considered.** (a) The schedule states presence's `arrived` at each boundary: a mover,
rejected for the reasons above. (b) **The schedule owns an agenda, and controllers walk to it.**

**Choice: (b).**

1. `ScheduleSystem` owns, on each person:
   - `Routine`: the authored segments, each `{ from: time of day, place, label }`;
   - `Agenda`: the segment in force, with its start and end and the routine's process.
2. Each person's day is **one `Process` of kind `routine`**, started while schedule reduces the
   genesis `routine-assigned`. Its expected end is the next boundary. At each wake, schedule emits
   `agenda-changed` (`Causation::Process`) and reschedules the process. The process never ends.
3. **It moves no one.** It provides no action, depends on no system, and states no other pack's
   vocabulary, so the registry would refuse it if it tried. Following an agenda is a controller's
   choice:
   - `mineworld run`'s paced controller walks there through `move`, after being addressed and
     before taking any initiative;
   - a human may ignore it;
   - a person nobody drives keeps their place while their agenda changes.
4. **Time of day is world seconds since the epoch, modulo 86 400.** That is schedule's own
   convention, and the kernel does not know it (`INV-12`).
5. The agenda is disclosed to its holder only (`INV-13`). `agenda-changed` is biographical (`ARC-29`),
   and validation guarantees that every one is a real change.

**Accepted limitations.**
- A day is the same every day: no weekdays, no exceptions.
- Routes are found only in a star town.
- Hosted worlds have no paced controller, so their agendas change and nobody follows them unless a
  person chooses to.
- **Fixture dependency (step-09 §4.3.7).** `social-cafe`'s routines keep 00:00–05:00 free of
  boundaries. A world hosted from genesis or from a day-end save therefore commits nothing for five
  hours, and the restart tests' "revision unchanged" claims rely on that. Each of those tests checks
  the assumption first, and fails naming it.

---

## ARC-33 — A System Pack is installed by declaring it in the build's installed set

**Date** 2026-10-07 · **Implements** [`MODULE_SPEC.md`](MODULE_SPEC.md) §3.1 · **Relates to** `ARC-8`,
`ARC-23`, `ARC-26`, `ARC-29`, `ARC-31`, `DEP-10`, `DEP-12`, `ARC-35`, [`MVP.md`](MVP.md) §9 `AC-1`
· **Design** `.structured-coding/plans/mvp0/step-10-market.md` §2.2, SD-1 … SD-5 (S9, PR 11a)

**Problem.** The frozen top-level criterion asks for *independently installable* interaction
systems. Until this decision, installing one System Pack edited about eleven lines in five files,
three of them outside `systems/` (finding F-1, step-09 §8.2):

```text
Cargo.toml (root)          a `members` line and a `[workspace.dependencies]` line
worldpack/Cargo.toml       a dependency line
worldpack/src/catalog.rs   an enum variant, an AVAILABLE entry, and one arm in each of six matches
Cargo.lock                 regenerated
```

`ARC-31`'s accepted limitations already said so: "The catalog is still a closed list compiled into
the build. The seam removes the format edit, not the registration." Every one of those edits teaches
the World Pack loader that a pack exists, which is the change amplification `CLAUDE.md` §4 rule 5
forbids.

**What cannot be removed.** MVP-0's System Packs are trusted, statically linked Rust
(`ARCHITECTURE.md` §12, `PACKAGE_FORMAT.md` §8). A crate is in a statically linked Rust binary only
if some crate in the build declares it as a Cargo dependency, and no build script, macro or linker
trick links a crate nobody declares. So one declarative line naming the pack must exist in some
manifest. The decision is only *where* that line lives and what else must change with it.

**Choice.**

1. **A System Pack declares itself.** The SDK crate `mineworld-sdk` (`sdk/rust/`) defines
   `SystemPack: System + Default`. A pack implements it once, in its own crate, and says there
   everything the build needs to know about it beyond `System`:
   - `BIOGRAPHICAL`: which of its event types belong in a biography (`ARC-29`; default none);
   - `SECTION`: the authored section it owns, if any (`ARC-31`; default none);
   - `decode_section`: how that section is decoded. A section owner writes
     `mineworld_sdk::owns_section!();` inside its `impl`, which defines `SECTION` and
     `decode_section` together from its `AuthoredSection` impl, so the two cannot disagree. The
     default refuses, naming the pack: "the '<id>' system owns no section".

   The SDK depends on `authoring`, `contracts`, `kernel` and `serde`, and never on a pack, so every
   pack can implement it without a dependency cycle.
2. **The installed set is a crate whose only content is the list.** `systems/installed/` (crate
   `mineworld-installed-systems`) depends on the SDK, on `presence` for the perception trait, and on
   every installed pack. Its `lib.rs` is one invocation of `mineworld_sdk::installed!`, one line per
   pack. The macro expands to the closed enum `Capability`, the constant `AVAILABLE` in the listed
   order, and the methods `resolve`, `id`, `section`, `owning_section`, `decode_section`,
   `biographical`, `install`, `provider` and `Display` — the same closed enum and matches the
   catalog held, generated. Every code path therefore stays monomorphic, and a section is still
   decoded straight from the YAML stream with its line and column (`DEP-10`). A listed pack that is
   not a `PerceptionProvider`, not `Default`, or not a `SystemPack` does not compile into the set. A
   test holds the list and the crate's manifest equal, and no two listed packs may share an id.
3. **The root manifest stops registering.** Its `members` names `"systems/*"` instead of one line
   per pack. A new pack depends on a sibling pack by `path = "../<name>"`, so the root
   `[workspace.dependencies]` never learns it. Path dependencies between sibling packs add no
   external dependency, so the root's rule — a crate that needs a dependency not listed there is
   adding one, which is a reviewable decision — is unchanged.
4. **`worldpack` names only the packs its format fields belong to.** A person's `location` is
   `presence`'s and a place's `passages` are `movement`'s (`ARC-31` item 5), so `worldpack` keeps
   those two dependencies and gains the SDK and the installed set. It drops every other pack, and
   re-exports `Capability`, `AVAILABLE` and `SectionOwner`, so its public API does not change. A
   structural test holds its dependency allow-list, which names infrastructure and the two format
   owners and never another pack.

**What installing a System Pack means in MVP-0 — the static-linking boundary.** This is the whole of
it, and a reader must not take "independently installable" as more:

```text
systems/<name>/                        the pack                                   (a new directory)
systems/installed/Cargo.toml           mineworld-<name> = { path = "../<name>" }  (one line)
systems/installed/src/lib.rs           <Variant> => mineworld_<name>::<System>,   (one line)
Cargo.lock                             regenerated by Cargo                       (generated)
then                                   rebuild the binary
```

- No other file is edited: not the root manifest, not `worldpack`, not the CLI, not the server, not
  a controller, not the kernel.
- **Installing** puts a pack into the build. **Enabling** it is a world's choice: a World Pack's
  `systems:` list names it. A world that does not enable an installed pack is not affected by it.
- **Installing without a rebuild is not MVP-0.** Adding a pack to a binary that is already built or
  to a server that is already running, or installing a pack that is not compiled from this
  repository's build, is the WASM component model of `ARC-8` (Tier 1). That is outside MVP-0
  (`overall.md` §1 non-goals). So is Milestone E's publishing sense of "a real world assembled from
  independently installable packs": `.mwpack`, a registry, and packs from outside this repository.
  *(Superseded in part by `ARC-66`, 2026-10-08: a System Pack whose **source** is outside this
  repository but which is **compiled into this build** from a pinned commit is in MVP-0, installed by
  the same two lines. `.mwpack`, a registry, installing without a rebuild and unreviewed code remain
  outside it.)*
- `mineworld install` and `mineworld add-system` (`MODULE_SPEC.md` §8) remain unimplemented. The
  two lines are written by hand.

**Why `systems/installed` lives under `systems/`.** The installed set is the list of System Packs in
this build. Placing it beside the packs makes installing one an edit under `systems/` only, which is
what `ARC-35` measures. The operator approved this placement at S9's freeze (step-10 QS-3).

**Accepted limitations.**
- A rebuild is required to install or remove a pack, as above.
- Cargo still needs one dependency line per pack, and `Cargo.lock` changes with every install.
  `ARC-35` admits `Cargo.lock` only as a generated file under a rule that it gains path packages
  under `systems/` and nothing else.
- A declarative macro generates the catalog, which is harder to read than a hand-written enum. The
  macro is one file, documented method by method, and every existing loader, CLI, persistence and
  server test runs through its expansion.
- Revisit this decision together with `DEP-12` when the first build installs packs it does not
  compile from this repository: that is `ARC-8`'s Tier 1. The question each catalog method answers
  stays the same; the catalog becomes a registry populated at startup.

**Note, 2026-10-13 (S24, PR DL-a; `pr-dl-a-decision-list.md` SD-DLa-12; `ARC-85`).** The same pattern is
reused for decision models. `cognition/installed` (`mineworld-installed-models`) is the build's list of
models, written with `mineworld-decision`'s `models!` macro: one `Name => crate::Type` line per
in-process `DecisionModel`, and one `external "<id>"` line per out-of-process model, which has no Rust
type. The macro expands to a monomorphic catalog enum, as `installed!` does (`DEP-12`); `decisions.yaml`
model ids resolve through it, and `mineworld packs list` lists its in-process models' packages.
Installing a model is one `Cargo.toml` line and one `models!` line, then a rebuild. Its guard test holds
the list and the manifest's dependencies equal and refuses a duplicate id, as this crate's does. It lives
under `cognition/`, so installing a model is an edit there and nowhere else.

---

## DEP-12 — System Pack registration: a declared installed set, not linker-section registration or dynamic loading

**Date** 2026-10-07 · **Status** selected; no dependency added · **Relates to** `ARC-8`, `ARC-33`,
`DEP-10` · **Design** `.structured-coding/plans/mvp0/step-10-market.md` §2.2 (S9, PR 11a)

**Problem.** Make a statically linked Rust System Pack installable by declaring it in one place,
with no code elsewhere that has to learn it (`ARC-33`). The build must still be able to do what the
World Pack loader's catalog does with each pack:
- resolve it by id;
- install it;
- hand it out as a perception provider;
- list its biographical event types;
- decode its authored section **generically over the YAML stream**, so that a refusal keeps the line
  and column `DEP-10` chose `serde-saphyr` for.

**Options considered** (`REUSE_POLICY.md` §§11–12, §17 — both directions):

```text
(a) the status quo: a closed enum in worldpack, one arm per pack
(b) linker-section registration: `inventory` (dtolnay, MIT/Apache-2.0) or `linkme`'s distributed_slice
(c) dynamic loading: `libloading`, `abi_stable`
(d) a value registry with type-erased section decoding: `erased-serde`
(e) worldpack generic over a catalog type supplied by the binary
(f) a build script scanning systems/*/Cargo.toml to generate the list (needs the `toml` crate)
(g) a SystemPack trait each pack implements, and an installed-set crate whose only content is the
    list, expanded by a declarative macro into the closed enum (`ARC-33`)
```

**Choice: (g).** No dependency is added. The "implementation of our own" is a `macro_rules!` over the
code the catalog already held.

**Why not the others** (`REUSE_POLICY.md` §12's reasons):

- **(a)** is finding F-1 itself: about eleven edits in five files per pack.
- **(b) `inventory` / `linkme` — dependency larger than the problem, and an architecture mismatch.**
  - They solve distributed registration of *values*. MineWorld's remaining cost after (g) is not
    registration code; it is the one Cargo line no crate can remove. Both also need a `use pack as
    _;` line, or the linker drops a crate nothing references, so they would not save even the list
    line.
  - They would add a dependency with life-before-main (`inventory`) or per-platform linker support
    (`linkme`).
  - Registering values would force the section decoder from a generic function into a type-erased
    value, which is (d)'s risk.
  - This is the case of forcing an existing wheel where it does not fit (`REUSE_POLICY.md` §17), not
    of reinventing one.
- **(c) `libloading` / `abi_stable` — architecture mismatch and an unacceptable trust model.** Loading
  native libraries needs `unsafe` (every crate here is `forbid(unsafe_code)`), relies on an unstable
  Rust ABI, and runs downloaded native code with full privileges. `ARC-8` already chose the WASM
  component model for code that is not compiled into the build.
- **(d) `erased-serde` — missing required semantics, unverified.** Whether `serde-saphyr`'s line and
  column survive an erased round trip has not been shown, and they are `DEP-10`'s reason for the
  parser. It would add a dependency to put that at risk.
- **(e) a generic worldpack — inability to isolate it cleanly.** Every caller of `WorldPack::read` —
  the CLI, the server's and persistence's tests, worldpack's own tests — would change, and
  worldpack's tests would need a dev-dependency cycle to name a catalog.
- **(f) a build script — dependency larger than the problem.** Cargo still needs the dependency
  line, so the script would save one list line at the price of a TOML parser in the build.

**Isolating interface.** `mineworld-sdk`: the `SystemPack` trait, `SectionOwner`, and the macros
`owns_section!` and `installed!`. A pack names only `SystemPack`; the World Pack loader names only
the generated `Capability`. If registration moves to a startup registry, these two surfaces are what
change.

**Accepted limitations and the revisit trigger.**
- One Cargo line per pack remains, as it must for static linking (`ARC-33`).
- Revisit when the first build installs packs it does not compile from this repository, which is
  `ARC-8`'s Tier 1. A runtime registry is then needed anyway, and the comparison above is reopened
  rather than assumed.

---

## ARC-34 — Complete affordances: an offer may carry the request it would accept

**Date** 2026-10-07 · **Approved by** the operator at S9's design freeze (step-10 QS-4; refinements
QS-20 … QS-25 accepted by the primary session, step-10 §12.0) · **Implements**
[`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §15.2, [`MODULE_SPEC.md`](MODULE_SPEC.md) §5 · **Relates to**
`INV-10`, `INV-13`, `ARC-23`, `ARC-26`, `ARC-27`, `ARC-35`, [`MVP.md`](MVP.md) §9 `AC-1`, `AC-15`
· **Design** `.structured-coding/plans/mvp0/step-10-market.md` §2.3, SD-9 … SD-12, §4.3 (S9, PR 11c)

**Problem.** `PacedRuleController` submits only actions it was compiled against: every request it
builds is typed by a pack crate it depends on, and it reads affordances only to ask whether *an action
it already knows* is available. A System Pack installed after the controller was written therefore
appears in every observation as an affordance and is never attempted by any headless person, because an
`Affordance` names an action and carries no payload: to submit an offered action, a requester must
already know the action's payload shape. The same is true of a client. `AC-1` forbids teaching the
controller each new pack, and a world whose new actions nobody headless ever attempts would pass a path
check and prove nothing (`ARC-23`). `CORE_CONCEPTS.md` §15.2 already states the intent the gap blocks:
a controller "is told what is possible instead of guessing" — told *that* something is possible, not
yet *what exactly to send*.

**Options considered.**

```text
(a) the controller learns each new pack's actions         the edit AC-1 forbids; every future pack
                                                         repeats it
(b) a Controller Pack policy as world data ("at this      payloads authored in a World Pack are rules in
    place, attempt this request")                        content (MODULE_SPEC §4 constraint 3), and the
                                                         policy still has to know payloads
(c) a generic `interact` action the server resolves       the server would choose the action for the
                                                         person (INV-1, INV-6); one resolver must know
                                                         every pack's precedence — a God object
(d) payload schemas in affordances (JSON-Schema-like)     a second schema language for an enumerable
                                                         problem; heavy before any LM controller exists
(e) declare that headless people need not use new packs   the new pack would be untestable headless: an
                                                         instrument that cannot see (ARC-23)
(f) complete affordances: the offering system may attach  chosen
    the exact payload it would accept; any requester may
    submit it unchanged
```

**Choice: (f).**

1. **The contract.** `Affordance<P = Vec<u8>>` gains a last field `payload: Option<P>`, the complete
   request payload the offering system would accept, in the observation's payload encoding.
   `Observation<P>` holds `Affordance<P>`. The field is serialized **only when present**
   (`skip_serializing_if`), and a frame without it decodes, so every existing observation, transcript
   and frame is byte-identical. An affordance with a payload is a **complete affordance**.
   - `Affordance::available` and `Affordance::unavailable` are unchanged and carry no payload;
     `with_payload(P)` adds one; `payload()` reads it.
   - `Affordance::request(actor, encode) -> Option<ActionRequest<Q>>` turns a complete affordance
     into the request it names: labelled with the affordance's **own** action type, targeted at its
     target, its payload re-encoded by the caller's `encode` (an observation carries JSON values,
     dispatch takes bytes — the encoding is not decided in `contracts`). `None` without a payload.
     Whether the affordance is available is the caller's judgement, read from `is_available`.
   - The label is the affordance's own, so `request` needs a crate-private labelling constructor on
     `ActionRecord`. No trust is lost: `ActionRecord` already deserializes from any label, and
     dispatch decodes the payload with the owning system's type.
2. **Who makes an offer complete.** Only the owning System Pack, through presence's
   `Offer::complete(&action, requirement)`. The action type is read off the action value's own type,
   exactly as `Offer::new::<A>` reads it, so a payload of another action cannot be attached at all.
   Perception carries the payload into the affordance whatever the verdict — available or unavailable
   with its reason — and decides nothing about what it means. An offer of a disabled pack is dropped
   by the same route map as before (`INV-10`, `AC-2`), payload and all.
3. **The server still decides.** A complete affordance is an offer, not a permission: whatever is
   submitted is validated by the owning system at dispatch (`ARCHITECTURE.md` §9). A controller that
   submits one has not created an interaction; it has attempted one the world offered.
4. **The paced controller's offer band.** `PacedRuleController::decide` gains exactly one band, after
   the social initiative and before the walking roll: the available complete affordances of the
   observation, in observation order; none → the band takes no part; otherwise, if draw index
   `OFFER_DRAW = 14` is below `ATTEMPTS_OFFERED` (out of 100), the one chosen by draw index
   `OFFERED_CHOICE_DRAW = 15` is submitted through `Affordance::request`. It imports no pack type for
   it, and `decide(&self, …)` stays a pure function of seed, pace and observation (`ARC-27`).
   `ATTEMPTS_OFFERED` is fixed by step-10 C-C6's measurement on a scratch install of a synthetic pack
   offering a complete affordance to every person everywhere — the worst case — against a criterion
   stated before measuring: every seat still moves and talks in every 30-day bucket, and every seat's
   offered request is accepted in every bucket. It passed at **20**, which is frozen (step-10 §9.3
   E-C6, and the note below). The band's constants and position are then frozen for S9 (`ARC-35` item 6, I-9): if
   a later world behaves badly, the remedy is in what its packs offer, never in the controller.
5. **`RuleController` is unchanged.** The reactive controller (`--agent`) answers and takes no
   initiative; it never attempts a complete affordance (`AC-15`).
6. **Free-form actions stay known by name.** `talk`'s utterance, `move`'s position and `invite`'s
   kind cannot be enumerated by the offerer, so the paced controller keeps knowing those actions by
   name. They are the foundation vocabulary of a walking, talking world. A new pack whose actions are
   free-form is usable headless only through a language-model controller (S10), or by offering
   complete affordances for a bounded choice.
7. **Clients.** `server/PROTOCOL.md` §§5–6 and `clients/protocol/ADOPTION.md` document the field: a
   client may submit a complete affordance unchanged and decides nothing new. No client code changes
   in S9 (step-10 QS-20); the first client use is S12's.

**Why this is not an `AC-1` violation although it edits `contracts`, presence and the controller.** It
is a precursor (PR 11c) that lands before the market, names no market concept (`ARC-35` item 7), is
proven with a synthetic pack the controller crate has never been compiled against, and leaves every
existing world byte-identical, because no existing pack offers a complete affordance.

**Accepted limitations.**
- The offerer must be able to enumerate the choices it would accept. An action whose payload is
  free-form cannot be offered complete (point 6).
- Observations grow with the offers: one complete affordance per offered choice, per target. A pack
  that offers many choices to many people makes every consult larger and slower (step-10 R-S9-2); the
  remedy is that the pack offers less, never a change to the pace or the controller.
- The band's rate is one number for every pack: a world with many packs offering complete affordances
  divides the same rate among them, in observation order.

**Note, 2026-10-07 (S9, step-10 C-C6) — the measurement that fixed the rate.** A scratch install,
never merged, put a pack offering a complete `ring { low | high }` to every person everywhere, with no
spatial requirement, into Social Café, and ran it 300 days with seed 7. The criterion, stated before
measuring (step-10 QS-25): every seat moves and talks in every 30-day bucket, and every seat's `ring` is
accepted in every bucket. At `ATTEMPTS_OFFERED = 20` both held — every seat moved at least 1 346 and
talked at least 393 times per bucket, and rang at least 197 times per bucket (29 907 rings in all, no
fault). The value is therefore **20**, frozen with `OFFER_DRAW = 14`, `OFFERED_CHOICE_DRAW = 15` and
the band's position. Under that worst case, moves fell from 180 665 to 167 348 and talks from 67 752
to 57 741 over the 300 days — the share of consults the band takes — which is what a pack offering
something to everyone everywhere costs, and a reason for a pack to offer less, not for the controller
to change.

**Note, 2026-10-08 (S14, PR 16b) — the first client accessors.** Point 7's "no client code changes in
S9" stands as history. The shared GDScript module `clients/protocol/mineworld` now reads and submits
complete affordances, for both reference clients at once:
- `MineWorldObservation.affordances(action_type, target)`, `complete_affordances(…)`,
  `affordances_about(id)` and `is_complete(affordance)` list them in the server's order and never pick
  one;
- `MineWorldClient.submit_affordance(affordance)` submits one unchanged, whether or not it is
  available, and refuses only an affordance with no `payload`.

Nothing on the wire changed. The API is specified in `clients/protocol/ADOPTION.md` §2 and designed in
`.structured-coding/plans/mvp0/step-15-demo-3d.md` §18. No new record was needed: it is this decision's
point 3 and point 7 carried into code.

**Note, 2026-10-08 (S17, PR IL-b; QIB-3, QIL-12, QIL-13) — a pack-stated refusal on an offer.**
Presence's `Offer` gains one builder, `Offer::refused(Rejection)`: the owning pack says the action is
offered but refused for a reason only it can judge — `PermissionDenied` from its section (`ARC-63`
item 8). Perception reports a refusal **before** it evaluates the spatial requirement, as
`Affordance::unavailable(…, reason)`; the requirement is still shown and a complete offer's payload
still travels. An offer that is not refused behaves exactly as before, and no contract changes:
`Affordance` already carries any `Rejection`. The offer and the dispatch give the same answer in the
same order: a refused action is refused before its spatial check in both.

**Note, 2026-10-14 (S27-b, F-S27-2).** group-activity offers accept-invitation, decline-invitation,
join-group-activity and leave-group-activity complete, with the empty payload each accepts, so a
requester that does not know the pack can attempt them (step-27 §5.5). invite stays incomplete: its kind
is free text. Consequence recorded in pr-s27b §3.2 F-S27b-1: the paced rule controller's offered band now
draws among them.

---

## ARC-35 — How AC-1 is measured

**Date** 2026-10-07 · **Approved by** the operator at S9's design freeze (step-10 QS-2) · **Implements**
[`MVP.md`](MVP.md) §2, §9 `AC-1` · **Relates to** `ARC-23`, `ARC-33`, `DEP-12` · **Design**
`.structured-coding/plans/mvp0/step-10-market.md` §1.3 (I-1, I-2, I-9), §2.5, SD-6 (S9)

**Problem.** `AC-1` and the frozen top-level criterion say that Market Town is Social Café plus
installed systems and a configuration change, with no edit to the kernel, `Person`, a renderer or a
controller. `overall.md` §1 glosses that as a change that touches only `systems/` and `worlds/`. Two
things make the sentence measurable only if the measurement is decided first:
- Making packs installable (F-1, `ARC-33`) and letting a controller attempt an action it was never
  compiled against (F-3) are framework changes. They must land before the market, and outside the
  measured change.
- A measurement chosen after seeing the result is not a measurement (`ARC-23`).

**Choice.**

1. **The transformation is two named merges.** The market arrives in PRs 11d (item, inventory,
   item-transfer, and `worlds/market-town`) and 11e (economy, employment, and their content). Each
   merge commit `M` is read against its own first parent, so unrelated PRs merging in between do not
   enter the range.
2. **Check 1, the change set.** `git diff --name-only M^1 M` ⊆ allowed, where allowed is:
   - `systems/**`;
   - `worlds/**`;
   - `Cargo.lock`, under a rule: every `[[package]]` added between `M^1` and `M` has no `source`
     (a path package) and lives under `systems/`, and every `[[package]]` whose dependency list
     changed lives under `systems/`. No external dependency arrives with the market;
   - Markdown documentation: `**/*.md` under `docs/`, `systems/`, `worlds/` and
     `.structured-coding/plans/`, because a PR here always updates its ledger.
3. **Check 2, the structure** — from `cargo metadata` at HEAD, independent of history:
   - the direct dependents of each market pack are `systems/*` crates only;
   - no dependency path leads to a market pack from `kernel`, `contracts`, `persistence`, `server`,
     `authoring`, `sdk` or `rule-controller`;
   - no code file (`*.rs`, `Cargo.toml`) outside `systems/`, `worlds/` and `tests/acceptance/` names
     a market pack's crate. Crate names are matched, not action or event slugs: contract tests
     already use stub ids such as `inventory-stub` that name no pack.

   Check 2 catches what a path diff cannot: a kernel or controller taught the market *before* the
   transformation range.
4. **Check 3, the world delta.** Market Town is Social Café plus configuration:
   - `systems`: Social Café's list, in order, then the five market packs;
   - places, population and seats: identical keys;
   - every person and place file: Social Café's fields and sections unchanged, plus sections owned
     by market packs only;
   - `items/` and `organizations/`: present only in Market Town.
5. **Fail closed.** Check 1 needs git history. Without it the test fails, naming the missing
   history. It never skips.
6. **The precursors are bounded instead of measured** — by two frozen invariants, each checked:
   - **I-2: the precursors know no market.** PRs 11a, 11b and 11c add no market concept (item 7).
   - **I-9: the controller's offer band is decided before the market exists.** Its constants are
     fixed in 11c against a synthetic pack. If Market Town behaves badly, the remedy is in
     `systems/` or `worlds/`, never in the controller.
7. **I-2 is checked mechanically**, by `tests/acceptance/tests/precursor_vocabulary.rs` (crate
   `mineworld-acceptance`, the home `ARCHITECTURE.md` §14 gives acceptance tests):
   - **Vocabulary:** `item`, `inventory`, `money`, `price`, `wage`, `job`, `shift`, `shop`,
     `economy`, `employ`.
   - **What is scanned:** every line a precursor adds, and the path of every file it adds, in every
     file except Markdown. Code, comments, manifests, `Cargo.lock` and fixtures are all scanned.
     Documentation is not, because documentation must be able to discuss the market; this record
     does.
   - **How a match is found:** a line is split into words at every character that is not a letter
     or a digit, and at every lower-to-upper case boundary. A word matches when, lowercased, it
     begins with a vocabulary word, so `items`, `ShopFront`, `employer` and `wages` all match.
   - **Which lines are "the PR's added lines", deterministically.** The test holds one row per
     precursor: the PR, its recorded base commit, and its branch.
     - If the first-parent history of `HEAD` holds that branch's merge commit `M` (subject
       `Merge pull request #N from <owner>/<branch>`, the form every merge to the protected `main`
       takes, `ARC-5`), the range is `base..M^2`: exactly what the PR added, whatever merged later.
     - Otherwise the PR is not merged, and the range is from `base` to the working tree: tracked
       changes (`git diff <base>`) plus every untracked file Git does not ignore. A line is
       therefore scanned before it is committed.
     - A precursor PR adds its own row and its own allow-list entries; 11b and 11c extend the same
       test.
   - **Allow-list:** each entry names a file, a line substring and a reason. It admits only matches
     that are not a market concept: the scan's own vocabulary list, a word in another sense (Rust's
     `Iterator::Item`), or a pre-existing use carried into an added line. An entry with an empty
     reason fails the test, and so does an entry that matches nothing, so the list cannot go stale.
   - **Fail closed:** the test fails, naming the cause, when `git` cannot run, the directory is not a
     repository, a recorded base is missing (a shallow clone), or `HEAD` does not descend from it.
     It never skips.
   - **A precursor that turns out to need a market word is a material stop**, decided by the
     operator, never an allow-list entry added to pass.
   - A squash merge would leave no merge commit. The range would then fall back to `base..HEAD`,
     which fails once the market exists. The failure is loud, not silent.

**What this does not claim.** That S9 as a whole changed only `systems/` and `worlds/`: it did not,
and could not for any linked pack (`ARC-33`). The criterion is read as `MVP.md` §2 words it, and
`overall.md` §1's path gloss is made exact by items 1–7.

**Accepted limitations.**
- The measurement depends on two merge commits being identified by id, which exist only after they
  merge. The proof (PR 11f) records them.
- Documentation inside the range is admitted by path and extension, not by content.

**Note, 2026-10-07 (S9, PR 11b; step-10 QS-16, F-31).** Item 7's allow-list admits **words, not
lines**. This tightens the check and loosens nothing:

1. Each entry names a file, a line substring, the exact lowercase words it admits, and a reason. Every
   market word on an added line is found, not only the first. Each one is refused unless an entry for
   that precursor and that file, whose substring the line contains, admits that exact word. A refusal
   names the word.
2. Before this note, an entry admitted every match on the lines it covered. A line such as
   `item_price`, admitted for the defined term `item`, would then have hidden `price`. Now the same
   line is refused, naming `price`. A word that merely begins with an admitted one is refused as
   itself: `itemprice` is not `item`.
3. An entry may admit **any** word only for the scan's own file, which must name the vocabulary it
   looks for. The test fails, naming any other entry that admits any word.
4. The other rules of item 7 are unchanged:
   - the vocabulary;
   - what is scanned (every non-Markdown added line and added path);
   - how a precursor's range is found;
   - failing closed;
   - an entry with an empty reason fails, and so does an entry that admits nothing.

Item 7's first-parent detection of a merged precursor is unchanged as well. Step-10 QS-15 proposed
reading every merge reachable from `HEAD`, and the primary session declined it. PRs 11b and 11c are
merged one at a time instead. The second to merge rebases onto the new `main` and moves its own row's
base to that commit (step-10 §12.0).

**Note, 2026-10-07 (S9, PR 11e; step-10 QS-35, QS-52) — six market packs.** The operator added a
consumption System Pack to the transformation's second merge (step-10 QS-35), so the market is six
packs, not five: `item`, `inventory`, `item-transfer`, `economy`, `employment` and `consumption`
(`ARC-37`, `ARC-38`). Item 1's second merge (11e) therefore carries economy, employment and
consumption, and edits the merged `inventory` pack, which is under `systems/` and inside check 1's
allowed set (step-10 QS-42). Check 2's "market pack" and check 3's "the five market packs" read **the
six market packs**. No other item changes. The AC-1 test that reads the list is step-10 PR 11f's.

**Note, 2026-10-07 (S9, PR 11f; step-10 QS-54 … QS-59, F-58 … F-63) — how the proof measures.** The
AC-1 test is `tests/acceptance/tests/ac1_composability.rs`, one `#[test]` per check, so a failing
check never hides another. PR 11f is not part of the measured transformation (item 1): it adds tests
and documentation and changes no pack, no world configuration and no framework behaviour. It settles
how each item is read, and tightens where it can; no item is relaxed except by the operator's answer
to QS-54 below.

1. **The two merges, by subject and by id** (item 1, the accepted limitation; QS-55). The test records
   both merges: 11d is `70e532f383ff81464d00066ff85e1b7fdc2296b0`, GitHub #43 from
   `mvp0/pr-11d-owning-things`; 11e is `2dddda86a4347d54de7bd2303d5268a421870b5f`, #46 from
   `mvp0/pr-11e-work-money-shops`. Each must be found on `git log --first-parent --merges HEAD` by its
   subject, `Merge pull request #<n> from <owner>/<branch>`, exactly once. The match must equal the
   recorded id and have exactly two parents. The subject proves that the id is that PR's merge on
   `main`'s first-parent chain; the id pins it, so a later merge with a recycled branch name cannot
   take its place. A squash merge has one parent and is refused.
2. **Check 1's `Cargo.lock` rule, as risk R-S9-6 words it** (item 2; QS-56). Both versions of
   `Cargo.lock` are read with `git show`. Every `[[package]]` added, removed or changed in **any**
   field between `M^1` and `M` must have no `source` and must be a crate under `systems/` at `M`. This
   tightens item 2, which names only "added" and "dependency list changed": a version or checksum
   change of an external package inside the range, a `cargo update`, is refused too. Both merges
   pass it.
3. **Check 2's "dependency path" counts normal and build edges** (item 3, second bullet; QS-54,
   F-58). `persistence`'s test `kill_and_resume` dev-depends on `worldpack`, which links the installed
   set (`ARC-33`) and so every market pack. Read over every edge kind, the check would fail AC-1 for a
   test that loads a World Pack, though nothing in `persistence` knows the market. The operator
   decided (2026-10-07, step-10 §4.6.0):

   > "QS-54: accepted by the operator (2026-10-07). ARC-35 check 2's "no dependency path" rule counts
   > normal and build edges only. Dev edges stay bound by check 2's other two rules: every dependent
   > of a market pack lives under `systems/`, and no code outside `systems/`, `worlds/` and
   > `tests/acceptance/` names a market crate."

   So the path rule follows what a library or a binary links. A dev edge is still seen by the first
   bullet (any declared dependency on a market pack, of any kind, must come from `systems/`) and by
   the third (no code file outside the three directories names a market crate). The alternative of
   admitting every edge through `mineworld-installed-systems` was declined: it would let a kernel that
   linked `worldpack` in production pass.
4. **How check 2 reads the structure** (item 3; F-60). It runs `cargo metadata --no-deps
   --format-version 1 --offline` through the `cargo` that built the test. `--no-deps` is enough, and
   needs no network: only a workspace member can depend on a path crate, so every path to a market pack
   runs through members, whose declared dependencies the output lists with their kind and path. A
   `cargo` that cannot run, or output that does not parse, fails the check by name. The crate-name
   scan reads the working tree — tracked files and untracked files Git does not ignore — so an
   uncommitted edit is seen. A name matches as a word: `mineworld-item` does not match inside
   `mineworld-item-transfer` by accident.
5. **Check 3 compares configuration structurally** (item 4; QS-57, F-62). Both packs must first be
   read by `WorldPack::read`. Each YAML file is then parsed into a structural value and compared.
   `world.id` and `world.name` may differ. `systems` is Social Café's list, in order, followed by
   exactly the six market packs (as a set). `places`, `population` and `seats` are equal. `items` and
   `organizations` are present in Market Town only. In every place and person file, each of Social
   Café's keys has an equal value, and every key Market Town adds is a section whose owner, by the
   build's own `Capability::owning_section`, is one of the six market packs. `items/` and
   `organizations/` exist in Market Town only. `README.md` is not configuration. Every difference is
   named by file and key.
6. **World-level tests read the market by type slugs, never through a market crate** (QS-58, F-61).
   A test that runs the `mineworld` binary lives in `tools/cli`, and the third bullet of check 2
   forbids naming a market crate there. Those tests therefore decode market facts and components by
   their literal slugs into test-local mirrors: unknown fields refused, schema version 1 asserted.
   That is also the oracle independence the test rules ask for.
7. **CP-4 is committed at the full 300 days, in the default suite** (QS-59). The bounded-horizon
   economy (`ARC-38`, L-13) is a 300-day claim, so a shorter committed horizon would let content that
   drains after its end pass. Activity is checked before any comparison of runs (I-7). Money
   conservation and the capacity of six are checked against the state in the save's newest snapshot,
   as well as the replayed facts, because the facts alone conserve money by construction (F-63).
8. **Check 1 fails closed without full history** (item 5). A shallow clone fails, naming the missing
   history and the remedy, `fetch-depth: 0`; it never skips. A CI that runs the test must check out
   the full history (S13).

**What the proof does not claim.** Installation without a rebuild: installing a pack is a directory,
two lines in `systems/installed` and a rebuild (`ARC-33`); without a rebuild is `ARC-8`'s Tier 1,
outside MVP-0. The evidence is recorded in step-10 §9.6.

**Note, 2026-10-08 (S15, PR 12d; step-11 §19, QD-1 decided by the operator, SD-D9) — Social Café may
own items.** A loose object is an Item file carrying `body:` (`ARC-36` note), and it is Social Café's
content exactly as a place's `body:` is, so item 4's last bullet and item 5 of the 11f note are amended.
The operator's decision, as recorded in step-11 §19: Social Café may own `items/` (loose objects as Item
files with `body:`); Market Town carries them unchanged; the claim "Market Town = Social Café +
installed packs + configuration" stays word for word; market-owned item sections stay Market Town's
only. Check 3 therefore reads:

1. `items` may be present in Social Café. Market Town's `items` contains every key of Social Café's (as
   a set); `organizations` stays present in Market Town only.
2. Every item file Social Café has exists in Market Town, and is compared exactly as a place or person
   file is: each of Social Café's keys has an equal value, and every key Market Town adds is a section
   owned by one of the six market packs.
3. Every item file only Market Town has carries only the format's fields and sections a market pack
   owns, unchanged; so a `body:` in a Market Town-only item file is refused, by file and key.
4. `organizations/` exists in Market Town only, unchanged.

Every world before 12d has no `items` in Social Café, and the check reads it exactly as before. The
claim measured is unchanged: **Market Town is Social Café plus configuration.** (It landed after the
three S19 notes below, which it does not change: allow-listed generic packs, their configuration and
their data stay Market Town's only.)

**Note, 2026-10-09 (S19, PR TW-a; step-19 §16.9 TWa-F1) — generic packs after the market.** The
operator ruled on TW-a's material stop (2026-10-09, option (i)): Market Town may enable generic packs,
named on an explicit allow-list, after the six market packs, through configuration. **The claim is
unchanged: the market delta is measured exactly.** Check 3 (item 4, as note 11f item 5 reads it) now
reads:

1. `systems` is Social Café's list, in order, then exactly the six market packs (as a set), then only
   packs on the allow-list, each once. An allow-listed pack anywhere before the end of the six, and any
   pack after the six that is not on the list, is refused, naming the pack.
2. `configure` is absent in Social Café. In Market Town it is absent or lists only allow-listed packs
   that `systems` enables; `configure/` exists in Market Town only and holds only their files.
3. Every other part of check 3 is unchanged: an allow-listed pack adds no section to a person or place
   file, because only a market pack's sections are admitted there.

The allow-list is `GENERIC_PACKS` in `tests/acceptance/tests/ac1_composability.rs`. It is
`["calendar"]` (`ARC-67`, `DEP-30`). Weather and any later generic pack are added by their own PRs, each
with its own review. A pack is generic when it is neither Social Café's nor the market's: a calendar is
a world's date and sun, not a market. Checks 1 and 2 are untouched: the two merges, the lock rule and the
dependency structure still measure the market and nothing else. A test holds both refusals.

**Note, 2026-10-09 (S19, PR TW-b; step-19 §17.5 C4) — `weather` joins the allow-list.** `GENERIC_PACKS`
is now `["calendar", "weather"]` (`ARC-68`). Weather is a world's climate, not a market, so it is generic
by the rule above. Nothing else in check 3 changes. Order among the allow-listed packs is free; each
still follows the six and is listed once; and `configure/weather.yaml` is admitted as the configuration
of an allow-listed pack that `systems` enables. The unit test's example of a pack that is not
allow-listed is now `bodies` (it was `weather`).

**Note, 2026-10-09 (S19, PR TW-d; step-19 §18.9.1 QTWd-1) — an allow-listed pack's `data/` attachments.**
The primary session ruled (2026-10-09) that admitting an allow-listed generic pack's data is the same
allowance as admitting its configuration (TWa-R1), since an attachment is configuration (`ARC-61`'s
`data:` note). Check 3 therefore admits a `data/` directory **in Market Town only**, when every file in
it, at any depth, is either an attachment named by the `configure/` file of an allow-listed pack the
world enables, or a file named `NOTICE` in the same directory as such an attachment (its licence and
provenance record, `DEP-8`, `DEP-31`). Any other file under `data/` is refused, naming it; a `data/` in
Social Café is refused. **The claim is unchanged: the market delta is measured exactly.** No market pack
gains anything by this, and checks 1 and 2 are untouched. Two mutations guard it (step-19 M-TWd-A1: a
stray `data/other.txt` in Market Town fails naming the file; M-TWd-A2: a `data/` in Social Café fails).

---

## ARC-36 — An authored Item is a kind; items and organizations are content kinds of a World Pack

**Date** 2026-10-07 · **Approved by** the operator at S9's design freeze (step-10 QS-5, QS-6) ·
**Implements** [`MODULE_SPEC.md`](MODULE_SPEC.md) §4.1, [`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md) §8 ·
**Relates to** [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §§7–8, `ARC-15`, `ARC-31`, `ARC-35` · **Design**
`.structured-coding/plans/mvp0/step-10-market.md` §2.4, SD-7, SD-8, §4.2 (S9, PR 11b)

**Problem.** `CORE_CONCEPTS.md` defines four kinds of entity that a world is authored with: `Person`,
`Place`, `Item` and `Organization`. `MODULE_SPEC.md` §4's frozen World Pack layout already lists
`items/` and `organizations/`, and the contracts already have `EntityType::Item` and
`EntityType::Organization`. But the MVP-0 loader read only `places` and `population`, so no World Pack
could declare an Item or an Organization. A System Pack therefore could not own state on one the way
`naming` and `schedule` own state on a person (`ARC-31`).

There are three ways to give a world such entities:

```text
(a) a System Pack creates them at genesis      no system can create an entity: the view a system is
                                               handed has no create, and adding one is a kernel change
(b) no entities: kinds as slugs inside some    a kind would have no identity and no tags, and nothing
    pack's state, organizations as tags        for several packs to share; an Organization is a core
    on places                                  primitive (CORE_CONCEPTS §8), not a tag
(c) complete §4's frozen layout                chosen
```

**Choice: (c).**

1. **Two content kinds.** `world.yaml` gains `items:` and `organizations:`. Each key names a file,
   `items/<key>.yaml` or `organizations/<key>.yaml`, which may carry `tags`, `note` and sections. Both
   lists are optional, and a World Pack that declares neither is read exactly as before.
2. **An authored Item is a kind.** `CORE_CONCEPTS.md` §7 separates a type from an instance and allows
   "unique items, stacked items, or abstract resources". MVP-0 implements **stacked items only**. An
   Item entity declared in `items/lantern.yaml` *is* the kind `lantern`, and a quantity held of it is a
   count of that kind, kept by whichever System Pack owns holdings. This reads the defined term one
   of the two ways the ontology permits. It does not redefine it (`CLAUDE.md` §2.1(3)).
3. **Instances are out of MVP-0.** A unique instance, with its own owner, place or condition, would be
   an Item entity created while the world runs, by the pack that owns it. No system may create an
   entity (option (a)), so instances need a kernel decision of their own.
4. **Identities are allocated after people.** The order is places, then people, then items, then
   organizations, each in key order. Every identity a world had before it declared items or
   organizations stays where it was, and so does every event id of its genesis.
5. **Keys are one namespace across the four lists.** A key declared in two lists is refused, naming
   both. A section names another entity by key together with the kind it requires, so one key must
   never mean two entities.
6. **Item and organization files may carry sections.** This extends `ARC-31`'s person-or-place wording
   unchanged in every other respect:
   - the owner declares which kinds may carry its section;
   - the loader decodes the section with the owner's type;
   - a reference must name a declared key of the required kind, and that now includes items and
     organizations.
7. **Genesis order.** Passages and locations are seeded first, unchanged. Then sections are seeded:
   items', organizations', places', people's, each in key order, and within one file in composition
   order. What people's and places' sections are likely to refer to (a kind, an organization) is
   therefore seeded before them. The worlds that existed before this decision declare neither kind,
   so their genesis is unchanged.

**Accepted limitations.**

- There are no item instances (item 3).
- Items and organizations have no display names. `naming` carries people only, and a place has none
  either (`ARC-31`).
- No installed System Pack owns a section on an item or organization file yet. The first owners
  arrive with the packs that need them (step-10 PR 11d). Until then, the order and reference rules of
  items 5–7 are proven with a section owner that exists only in the loader's own tests.
- Organization membership, roles and accounts (`CORE_CONCEPTS.md` §8) are not authored fields. They
  are the state of whichever System Pack owns them, carried as its section.

**Note, 2026-10-08 (S15, PR 12c; step-11 QB-3, decided by the operator; §18.0, SD-O3) — an item file
with a `body:` section is one physical object.** Item 2 reads `Item` as a kind. The `bodies` System
Pack adds the second reading `CORE_CONCEPTS.md` §7 already allows, "unique items", for exactly one
case, and nothing else changes:

1. **An item file that carries the `bodies` pack's `body:` section declares one physical object**, not
   a kind: the Item entity *is* that object, lying in one place at one position, with one shape. It is
   created at genesis like every other authored Item (item 4); no entity is created while a world runs.
2. **It is never a declared kind, so it is never held.** It carries no `item:` section: `bodies`
   refuses, at genesis, an Item that is both a declared kind and an object (`bodies-held-kind`). What
   a person or an organization may hold is a declared kind (`ARC-37`), so the pack that owns holdings
   refuses every holding of an object by its own rule, at genesis and while the world runs. Lying in a
   place and being held are disjoint by construction.
3. **Instances in general stay out of MVP-0** (item 3). An object is authored, not created; a kind
   still has no instances; and how a unique object could one day be held — what carrying needs — is
   the first decision of the step that adds carrying, not this note's.

---

## ARC-37 — Owning and giving: kinds, holdings, give, and what a person can carry

**Date** 2026-10-07 · **Approved by** the primary session at 11d's design freeze (step-10 §4.4.0;
QS-27 operator-visible, QS-28 … QS-34, QS-36, QS-37 accepted) · **Implements**
[`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §§6.3, 7, 13.1, [`MODULE_SPEC.md`](MODULE_SPEC.md) §4.1 ·
**Relates to** `INV-7`, `INV-10`, `INV-13`, `ARC-23`, `ARC-26`, `ARC-28`, `ARC-31`, `ARC-34`, `ARC-35`,
`ARC-36`, [`MVP.md`](MVP.md) §9 `AC-1`, `AC-2` · **Design**
`.structured-coding/plans/mvp0/step-10-market.md` §2.6, SD-13, SD-16 … SD-21, §4.4 (S9, PR 11d)

**Problem.** Market Town begins with people who own things and give them to each other. Three
questions decide whether that can be done as installed System Packs without breaking single ownership
(`CLAUDE.md` §4 rule 1):
1. Who owns what an item kind *is*, and who owns how many of each kind somebody holds.
2. How a pack that decides a give — and later a purchase or a shift's production — changes holdings
   it does not own.
3. How a world whose people only give, and never use anything up, stays alive. Nothing in MVP-0 eats,
   drinks or sleeps (step-10 QS-10).

The R-S9-1 spike (step-10 §9 E-4) answered the third with a measurement. Social Café has one person
whom no seat names, Otto, and no controller consults him. He is given to and never gives, so he is an
**absorbing sink**. With no bound, he held 31 of the town's 33 items by day 30, and gives fell from
739 in days 1–15 to 312 in days 16–30.

**Options considered, for the sink.**

```text
(a) a person carries at most N items, all kinds    a pack rule in inventory; the sink becomes finite.
    together                                       Chosen
(b) consent: the taker must accept a pending give  a process and a second action; an undriven person
                                                   never accepts, so it also works, at about twice the
                                                   code and with offers nobody headless can answer
(c) give only to people a controller drives        impossible: a world does not know who is driven
                                                   (INV-1)
(d) content only                                   nothing in content stops a person receiving
(e) retune the paced controller                    forbidden: its offer band was frozen before the
                                                   market existed (ARC-34, ARC-35 item 6, I-9)
```

**Choice.**

1. **`item` owns what a kind is.** It owns `ItemKind { category }` on Item entities, from the `item:`
   section of an item file, `{ category: <slug> }`. A category is 1–32 bytes of `a–z`, `0–9` and `-`,
   neither beginning nor ending with `-`, its own type, refused at its line and column. It states the
   public genesis fact `item-kind-declared { item, category }` and alone reduces it. It provides no
   action, runs no process, depends on nothing, discloses nothing and has nothing biographical.
   `is_declared(world, item)` is the question other packs ask. An item file with no `item:` section is
   an inert entity that no pack trades (`ARC-36`).
2. **`inventory` owns holdings, and only it writes them.** It owns `Holdings` on Persons and
   Organizations: a list of `{ item, count }` sorted by item, with no zero entries. It is a list and not
   a map keyed by item because an `ItemId` serializes as `{ entity, type }`, which cannot be a JSON
   object key, and payloads, observations and snapshots are JSON (`DEP-5`). The `holdings:` section of
   a person or organization file, `{ <item key>: <count ≥ 1> }`, names Items. Its facts are `stocked {
   holder, item, count }` (genesis, visible to the holder) and `items-transferred { from, to, item,
   count }` (visible to its two participants). It depends on `item`. It discloses a holder's
   `Holdings` to that holder only (`INV-13`). Nothing it states is biographical: eighteen thousand
   gives in 300 days would bury a biography, and owning a coffee is not an event in a life.
3. **The owner decides, three times, through one function (`ARC-26`).** `admit_transfer(world, from,
   to, item, count)` is the whole of what `Holdings` refuses about a transfer:
   - the count is at least one;
   - `from` and `to` differ;
   - both are living Persons or Organizations;
   - `item` is a declared kind;
   - `from` holds at least `count`;
   - `to` can take `count` (item 5).

   It is asked by a deciding pack's `validate`, by the checked constructor `transfer(world, from, to,
   item, count)`, and again by inventory's own reduction. On refusal the reduction writes nothing and
   fails with `KernelError::FactRefusedByOwner`. A `stocked` fact is checked at reduction the same way
   (a living holder, a declared kind, a count of at least one, within capacity).
4. **Seeding is checked in two halves, because the source forces it** (step-10 F-37). Every genesis
   fact of a World Pack is computed before any is reduced, so a section is seeded against a world with
   no state yet, and inventory's seed cannot ask whether `item` has declared a kind. The seed checks
   what it can — each key names an Item, each count is at least one, a person's total is within
   capacity — and the reduction checks the declared kind. That reduction follows `item-kind-declared`
   in genesis order, because items' sections are seeded before people's and organizations' (`ARC-36`
   item 7).
5. **A person carries at most `PERSON_CAPACITY = 6` items, all kinds together; an organization is not
   bounded.** This is inventory's rule. A transfer that would take a person past it is refused
   `TargetUnavailable`, by the constructor and the reduction alike. A person's authored holdings past
   it are refused at genesis, naming the file. `can_take(world, holder, count)` answers the question
   for an offer. In the spike, with this bound, every seat gave in every 30-day bucket over 300 days
   (at least 118 times each), and Otto ended holding exactly six. The bound also limits observation
   size (step-10 R-S9-2): a person holds at most six kinds, so at most six `give` offers per person
   nearby.
6. **`item-transfer` provides `give { item, count }`, targeting a Person, and owns nothing.** The
   requirement is the same place, within 3 000 mm, and an available target. To an observer who holds
   something, and for each *other* living Person present, it offers **one complete affordance per kind
   held, count 1**, in item order (`ARC-34`). The target is available when it can take one more. It
   never offers a give to the observer itself: perception asks a provider about every person present,
   the observer included (step-10 F-40). `validate` reads the payload, requires a living Person actor
   and a different living Person target (`NoSupportedInteraction` otherwise), evaluates the
   requirement against presence's positions, then asks `admit_transfer`. `resolve` states inventory's
   `items-transferred` through `transfer` and nothing else. It depends on `inventory` and `presence`.
   Disabled, `give` is answered `Unavailable`, is offered nowhere, and holdings never change (`INV-10`,
   `AC-2`).

The resulting dependencies, one way:

```text
item ◄── inventory ◄── item-transfer ──► presence
```

**Accepted limitations.**
- **Nothing is consumed.** Gives conserve items, so 11d's flow stays alive under the bound. Purchases
  in step-10 PR 11e move items from shops to people, and with a bound of six per person buying would
  stop once everyone is full. The operator decided on 2026-10-07 (step-10 QS-35) that 11e adds a
  consumption System Pack, which removes items through inventory's checked constructor. Inventory
  stays the only writer of holdings.
- **Item kinds have no names, and items are never perceived.** An observation lists the observer's
  place and the people in it, so `ItemKind` is disclosed to no one. A client shown `give { item:
  { entity: 21, type: item } }` cannot name the item (step-10 F-41, recorded for S12).
- **No item instances** (`ARC-36` item 3).
- **Organizations are unbounded.** A shop's stock is content; a limit would be a later pack's rule.
- **`items-produced` does not exist yet.** It arrives in 11e with its first stater, `employment`
  (step-10 QS-28). Adding a fact before anything states it would design it ahead of its consumer
  (`CLAUDE.md` §4 rule 11).
- **The capacity is a published constant**, not world configuration, under the `ARC-26` note's rule.

**Note, 2026-10-08 (S15, PR 12d; step-11 SD-D10, QD-4, QD-12, QD-15, QD-16; step-13 R-PK-2, F-41) —
item kinds have names, disclosed as a catalogue.** The accepted limitation "Item kinds have no names"
is lifted; "items are never perceived" stands. Nothing else in this decision changes.

1. **A kind has a display name.** The `item:` section is `{ category, name }`. `name` is required: 1–64
   bytes, no control characters, no surrounding whitespace — the rule `naming` applies to a person's
   name, restated in `item`, which depends on no pack. Two kinds may share a name; a name is display,
   never identity. `item-kind-declared` and the `ItemKind` component carry it (schema 2 each); `item`
   is version 2.
2. **The catalogue is disclosed on a place.** To whoever perceives a place, `item` discloses one
   `item-catalogue` record on it: `[{ item, category, name }]` for every declared kind of the world, in
   `ItemId` order, built from `ItemKind` at disclosure and never stored (economy's listing is the
   precedent, `ARC-38`). A world without `item` discloses none.
3. **Items are still never perceived as entities.** An observation's entities are the observer's place
   and the people in it, as before; a kind appears only inside the catalogue. A client shown `give {
   item: { entity: 21, type: item } }` now finds entity 21's name there (F-41).
4. Loose objects (`ARC-36` note) are not declared kinds, so the catalogue never lists them.
   Organizations still have no names; `ARC-38`'s accepted limitation is lifted for kinds only.

---

## ARC-38 — Work, money, shops and consumption: who owns each, and how the loop is kept alive

**Date** 2026-10-07 · **Approved by** the primary session at 11e's design freeze (step-10 §4.5.0;
QS-39, QS-45 and QS-47 operator-material, accepted as designed; QS-40 … QS-44, QS-46, QS-48 … QS-53
accepted as recommended); the consumption pack itself by the operator (step-10 QS-35, 2026-10-07) ·
**Implements** [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §§8, 10, 13.1,
[`MODULE_SPEC.md`](MODULE_SPEC.md) §4.1, [`MVP.md`](MVP.md) §§3, 5 · **Relates to** `INV-7`, `INV-10`,
`INV-13`, `ARC-23`, `ARC-26`, `ARC-28`, `ARC-29`, `ARC-31`, `ARC-32`, `ARC-34`, `ARC-35`, `ARC-36`,
`ARC-37`, [`MVP.md`](MVP.md) §9 `AC-1`, `AC-2` · **Design**
`.structured-coding/plans/mvp0/step-10-market.md` §2.6, SD-13, SD-22 … SD-28, §4.5 (S9, PR 11e)

**Problem.** Market Town's people own and give things (`ARC-37`). The second half of the measured
transformation (`ARC-35` item 1) adds work, money, shops and the using-up of things, as installed
System Packs only. Five questions decide whether that keeps single ownership (`CLAUDE.md` §4 rule 1)
and a living market:
1. Who moves money, when a shift that one pack decides was worked must pay a person from an
   organization's funds.
2. How items come into existence and leave it, when only `inventory` writes holdings.
3. How work is noticed, when no controller knows that work exists.
4. How a shop is authored, when a System Pack owns at most one section.
5. Whether the money and item loops stay alive for the measured 300 days, and what is changed if
   they do not.

**Options considered.**

```text
closing the item loop  (a) shops buy goods back            declined by the operator (QS-35)
                       (b) a consumption pack              chosen by the operator (QS-35)
                       (c) relax CP-4's purchase criterion declined (QS-35)
production and use     (a) facts in the stating packs'     inventory would reduce another pack's
                           vocabularies, reduced by           vocabulary (against ARC-26 item 1)
                           inventory under ARC-28
                       (b) inventory's own facts,          chosen (QS-42)
                           stated through checked
                           constructors (ARC-26)
attendance             (a) polling presence at a fixed     wakes the whole town for one fact; worked
                           interval                         time only as fine as the interval
                       (b) presence at the shift's start,  chosen: exact, no polling; presence's
                           then presence's                 person-entered-place carries `from`
                           person-entered-place between      (step-10 F-49)
                           wakes (ARC-28: reacting into
                           employment's own state)
authoring a shop       (a) a `shop:` section on the place  a second section for economy; a pack owns
                                                           one (step-10 F-47) — a framework change
                                                           justified only by the market (I-2)
                       (b) on its operator organization,   chosen (QS-43); no place file changes
                           inside economy's one section
a drained market       (a) retune the paced controller     forbidden (ARC-34, ARC-35 item 6, I-9)
                       (b) size content against a          chosen
                           criterion stated before
                           measuring
```

**Choice.**

1. **`inventory` gains the two facts that create and remove items** (QS-41, QS-42). `items-produced {
   holder, item, count }` and `items-consumed { holder, item, count }` are inventory's vocabulary,
   reduced by inventory alone, visible to the holder. Their checked constructors `produce` and
   `consume` ask `admit_production` (a living holder, a count of at least one, a declared kind, and
   `can_take` — production into a person respects the capacity of six) and `admit_consumption` (a
   living holder, a count of at least one, a declared kind, at least `count` held). The reduction asks
   the same function again and, on refusal, writes nothing and fails `FactRefusedByOwner` (`ARC-26`,
   `ARC-37` item 3). The fact names say what happened to holdings, not why; the stater is the cause.
2. **`employment` owns jobs, the `employed-by` edge and the `shift` Process; work is attendance**
   (QS-8). Its section `job:` on a person file is `{ employer: <organization key>, workplace: <place
   key>, from: "HH:MM", until: "HH:MM", wage: <minor units per hour>, produces: { <item key>: <count
   per full shift ≥ 1> } }`. `from < until`: a shift lies within one day (QS-50). Times are schedule's
   `TimeOfDay`, a Cargo dependency on its type only, so the town has one time-of-day convention
   (`ARC-32`).
   - The genesis fact `hired { employee, job }` gives the person `Employment`, the `employed-by` edge
     (Person → Organization) and one `shift` Process, woken at each next `from` and `until`.
   - At `from` the wake states `shift-started { employee, present }`, where `present` is presence's
     answer: is the employee at the workplace.
   - Between wakes employment reacts to presence's `person-entered-place` for an employee on shift.
     Entering the workplace starts a present span; entering anywhere else *from* the workplace closes
     it into the worked seconds. That is employment writing its own state by reacting (`ARC-28`).
   - At `until` the wake states `shift-ended { employee, worked }`, then, when `worked > 0`,
     `wage-due { employee, employer, amount = wage × worked ÷ 3 600 }` and, per produced kind,
     inventory's `items-produced` for the employer with `count = per_shift × worked ÷ shift length`
     (integer floor, skipped at 0), through `produce`.
   - It depends on `inventory` (it states inventory's fact) and `presence` (it reads positions and hears
     arrivals). It discloses `Employment` to the employee only. `hired` is biographical (QS-49):
     shifts, wages, purchases and meals are thousands of facts that would bury a biography.
   - **It never reads or writes a `Wallet`.** It has no write token for one (`INV-7`), and its
     manifest does not name economy.
3. **`economy` owns `Wallet` and `Shop`, and is the only mover of money.**
   - Its one section, `economy:`, on person and organization files: `{ wallet: <minor units> }`, and on
     an organization optionally `shop: { at: <place key>, prices: { <item key>: <price ≥ 1> } }`
     (QS-43). The genesis facts `funded { holder, balance }` (visible to the holder) and `shop-opened {
     place, operator, prices }` (public) become `Wallet { balance: u64 }` on the holder and `Shop {
     operator, prices }` on the place. `u64` minor units: no floating point and no negative balance
     can be represented (I-6).
   - `money-transferred { from, to, amount }` is the one fact that moves money. Its reduction refuses
     an amount larger than the payer holds, a zero amount, a payer who is the payee, or a party that is
     not a living Person or Organization — `FactRefusedByOwner`, writing nothing.
   - **`buy { item }`** has no target. Its requirement is `at_place(shop)` with an available target,
     because a target-less offer can only say "at that place" (step-10 F-48). It is offered to a
     living person standing in a shop's place as **one complete affordance per priced kind**
     (`ARC-34`): available when the operator holds one (`admit_transfer`), the buyer can pay and the
     buyer can carry it; otherwise unavailable with `TargetUnavailable`, the one reason an offer can
     carry (QS-44). `validate` asks the same through `SpatialRequirement::evaluate`; a buyer who is not
     in a shop, or a kind the shop does not price, is `NoSupportedInteraction`. `resolve` states
     `money-transferred` (buyer → operator, visible in the shop's place, QS-45) and inventory's
     `items-transferred` (operator → buyer) through `transfer`.
   - **Wages.** Economy subscribes to employment's `wage-due`, decoding it through employment's
     published type — a Cargo dependency, **no system dependency** (`ARC-28`). It answers with
     `money-transferred` (employer → employee) caused by the `wage-due`, or, when the employer cannot
     pay, `wage-unpaid { employee, employer, amount }`. This is `CORE_CONCEPTS.md` §13.1's example,
     implemented literally.
   - It depends on `inventory` (it states `items-transferred`, and reads the operator's stock) and
     `presence`. It discloses a `Wallet` to its holder, and to everyone perceiving a shop's place the
     shop's **listing**: the operator, and per priced kind its price and how many the operator holds
     (QS-45). That count is read from inventory's state, which economy may read because it depends on
     inventory; it never writes it. Inventory itself still discloses holdings to the holder only. The
     listing is how a second client perceives a purchase (step-10 CP-7) without anybody's holdings being
     disclosed. Nothing economy states is biographical.
4. **`consumption` provides `eat { item }` and `drink { item }`, and owns nothing** (QS-35, QS-39).
   A held kind whose `ItemKind` category is `food` is eaten and one of category `drink` is drunk
   (published constants `EATEN` and `DRUNK`); goods are never consumed. There is no spatial
   requirement: a person eats what they carry, wherever they are (QS-40). It offers a living person one
   complete affordance per edible or drinkable kind held, without a target, in item order. `validate`:
   the payload, a living Person actor, no target, the action matching the kind's category
   (`NoSupportedInteraction` otherwise), then `admit_consumption(actor, item, 1)`. `resolve` states
   inventory's `items-consumed` through `consume` and nothing else. It depends on `inventory` (system
   and Cargo) and reads `item`'s `ItemKind` (Cargo). It closes `MVP.md` §5's `eat` as an interaction
   and adds `drink`, without which drinks would fill hands (step-10 F-50).
5. **Shops sell consumables only, and their stock is produced by the shift** (QS-48). Goods keep
   circulating by `give` (`ARC-37`); a bought good would occupy one of a person's six places for
   good. The item loop is then produce → buy → give → eat or drink.
6. **The loop is sized in content, against a criterion stated before measuring** (`ARC-23`, I-7, I-9).
   Endowments, prices, wages, production and opening stock are World Pack content. They are fixed by a
   300-day seed-7 run of Market Town read from its save, against conditions written down before the
   first run (step-10 §4.5.3 E-9 b): in every 30-day bucket a purchase, a wage paid to each job holder,
   an item produced, an item eaten or drunk, and a give; zero `wage-unpaid`; no wallet ever below the
   cheapest price in the town; money conserved; nobody holding more than six. A run that fails is
   recorded and the content or the packs are changed, never the controller.

The resulting dependencies, one way:

```text
item ◄── inventory ◄── item-transfer ──► presence
            ▲  ▲  ▲
            │  │  └──── consumption ┄┄► item           (┄┄ Cargo only: ItemKind)
            │  └─────── economy ──► presence
            │              ┆
            │              ┆ subscribes to wage-due: Cargo dependency on employment's type, no system
            │              ┆ dependency (ARC-28)
            └────────── employment ──► presence
```

**Accepted limitations.**
- **A bounded-horizon economy, not a closed one** (QS-47; living-world gap **L-13**). Two people hold
  jobs (MVP §3, QS-46). The other people have no income and live on an endowment sized for the
  measured 300 days; past that horizon they run out. Closing the loop — more jobs, or income without
  one — is a later step's work, not a retuning of this one. A market that balances itself over years
  is outside S9 (step-10 §1.2, R-S9-3).
- **No hunger, appetite or sleep.** Consumption is an interaction, not a need: people eat because they
  are offered food, at the paced controller's rate. A needs pack is later work (step-10 QS-10).
- **One reason for an unavailable buy.** Out of stock, cannot pay and cannot carry all read
  `TargetUnavailable`, in the offer and at dispatch, because an offer carries one pack-supplied verdict
  (step-10 F-48). Finer reasons wait for a contract that lets an offer carry one.
- **Shifts lie within one day**: `from < until`, no shift across midnight (QS-50).
- **No hiring, firing, promotion, business ownership or buying back** (step-10 §1.2, QS-35).
- **Item kinds and organizations still have no names** (step-10 F-41): a listing names kinds by id.
- **Prices, wages and endowments are content, not System Pack configuration** — they are values a
  world states, as `ARC-26`'s note says of constants that are not yet configuration.
- **A pack owns one section** (F-47): economy's shop is authored on its operator. The limit is worked
  within, not changed; lifting it would be framework work justified only by the market.

---

## ARC-39 — An arrival is resolved before it is recorded

**Date** 2026-10-07 · **Approved by** the operator (step-11 QB-1, resolve before recording; QB-15, a
catalog registered at start-up, with its three bounds) and the primary session at PR 12a's design
freeze (step-11 §16.0: QR-2 and QR-4 decided, QR-1, QR-3, QR-5 … QR-12 accepted as recommended) ·
**Implements** [`ARCHITECTURE.md`](ARCHITECTURE.md) §§7–8 · **Relates to** `INV-7`, `INV-15`,
[`MVP.md`](MVP.md) §9 `AC-2`, `AC-9`, `ARC-15`, `ARC-23`, `ARC-25`, `ARC-26`, `ARC-33` · **Design**
`.structured-coding/plans/mvp0/step-11-bodies.md` §§4.4, 16 (S15, PR 12a)

**Problem.** Step S15 (bodies and physical interaction) needs a person's arrival to be able to end
short of where it was asked to go, and to move other people standing in the way. Three constraints
bound the answer:
- **The log holds only true arrivals** (the operator's QB-1). An `arrived` that is recorded and then
  corrected by a later fact is refused as a design, even though each fact would be true in sequence.
- **Movement still alone decides whether a move is allowed, and names no resolver.** A mover that had
  to call the S15 pack would make that pack impossible to remove (`AC-2`) and every future mover would
  have to remember it.
- **No kernel change and no contract change.** Presence owns where people are (`ARC-26`); the question
  "what does this arrival actually achieve" is presence's to ask, not the kernel's.

Presence already has the only public way to build an `arrived`: its constructor `arrival(world,
person, location)`, which every stating system calls under `ARC-26`. An arrival is built **inside**
`World::dispatch`, where the only things in reach are the stating system's `&self` and a `WorldRead`
(`kernel/src/view.rs`: entities, components, relations, processes — no composition, no registry, no
enabled state). So whatever answers the question must reach presence's constructor without being
handed to it per call.

**Options considered.**

```text
(a) correct after recording: the resolving pack     transient facts in the log (QB-1, refused by the
    reacts to each arrived and states a second one   operator)
(b) movement calls the resolving pack               the pack is no longer removable; every future
                                                     mover must remember it
(c) a kernel geometry seam                          a kernel change; the likely route for line of
                                                     access later, not needed here
(d) F2: a kernel extension slot, World::provide /   per-world rather than per-process, but a kernel
    WorldRead::provided                             change (QB-15)
(e) F1: presence asks every registered              chosen
    ArrivalResolver before it records, through a
    write-once catalog registered at start-up
```

**Choice: (e).**

1. **The seam is presence's** (`systems/presence/src/resolve.rs`).
   - `Arriving { person, from: Option<Location>, to }`: one arrival as presence knows it before it
     records anything. `from` is the person's current `Presence`, read by presence, `None` at a first
     placement. Only presence builds one.
   - `Resolution { reached, displaced: Vec<(PersonId, Location)>, stopped_by: Option<EntityId> }`,
     with private fields. A resolver starts from what it is handed and changes it only through
     `Resolution::stopped_at` and `Resolution::displacing`; `Resolution::unchanged` is the arrival
     exactly as decided.
   - `ArrivalResolver: Send + Sync`, with `resolver_of() -> SystemId` and `resolve(world, arriving,
     so_far) -> Resolution`. Its contract: read only the `WorldRead`; keep nothing; read no clock, no
     random source and no process-wide state; return `so_far` unchanged when the world holds none of
     the pack's own state about the people and the place involved — which is what keeps a world
     without the pack byte-identical; and the pack's `System::install` calls `require_registered`
     first (item 5).
   - A resolver cannot emit a fact. Its pack's own consequences of an arrival happen in that pack's
     reactions to the facts presence records.
2. **Two constructors.**
   - **`arrivals(world, person, to) -> Result<Vec<Emission>, Rejection>`, for movers.** It asks
     `admit`; builds the `Arriving`; folds the registered resolvers in ascending `SystemId`, starting
     from `Resolution::unchanged` and **checking the result after each resolver** (item 3); and
     returns, in this order: `arrived { person, reached }`; one `arrived` for each displaced person, in
     the resolution's order; and `stopped-short { person, wanted: to, reached, by }` when `reached ≠
     to`. Every `arrived` is built exactly as `arrival` builds it.
   - **`arrival(world, person, to) -> Result<Emission, Rejection>`, for placement.** Its signature is
     unchanged. It runs the same fold and refuses with item 3's code when the result is not unchanged;
     otherwise it returns exactly the one emission it returned before this decision. An arrival is
     therefore recorded as resolved or refused — never recorded unresolved.
   - Resolvers are asked in ascending `SystemId` order, so the order is a function of the resolvers'
     names alone: not of the `installed!` list, not of a world's `systems:` list.
3. **Presence still decides** (`ARC-26`). After each resolver, a resolution is refused with
   `Rejection::System { code: "resolution-refused", detail: "<resolver id>: <rule>" }`, which the
   stating system turns into `KernelError::FactRefusedByOwner` as it does any refusal of presence's:
   - (a) `reached` is in `to`'s place;
   - (b) `reached` has `to`'s facing, and a local position exactly when `to` has one;
   - (c) when `from` is in `to`'s place and both have local positions, `reached` is no farther from
     `from` than `to` is, compared as squared millimetres in `i128`: a resolver may shorten or bend an
     arrival, never lengthen it, so the stating system's own bound still bounds it;
   - (d) `admit(person, reached)`;
   - (e) every displaced entry names a living `Person`, not the walker, listed once, whose current
     `Presence` is in `to`'s place, displaced to a location in that place that keeps rule (b)'s local
     position rule and passes `admit`;
   - (f) `stopped_by`, when present, names an entity of this world, and is present only when `reached
     ≠ to`.
   Checking after each resolver names the resolver at fault (`ARC-23`); a check of the final result
   alone could not.
4. **`stopped-short` is presence's fact, stated by the stating system.** `StoppedShort { person,
   wanted, reached, by }`, owner presence, schema 1. An arrival that ends short of where it was asked
   to go is a fact about where a person is, which is presence's domain, and it names nobody who moved
   them. Presence reduces it into nothing. **Presence's declaration does not list it**: presence never
   states one — not at genesis, where `arrival` refuses a changed placement, and not in its reaction
   — and a declaration must be true. Only a stating system declares it (movement, with this decision).
   Every fact of one arrival — the walker's `arrived`, each displaced person's, the `stopped-short` — is
   stated in one emission list and caused by the request (`AC-9`).
5. **The catalog: the one named process-wide value.** `ENGINEERING_RULES.md` §15 forbids hidden global
   state; this is a stated exception, not a hidden one.
   - It is a `static OnceLock<Vec<Box<dyn ArrivalResolver>>>` in presence, with exactly three entry
     points: `register_resolvers(list)`, `registered_resolvers() -> Option<Vec<SystemId>>`, and
     `require_registered(&SystemId)`. Nothing else reads or writes it.
   - **It is not world state.** It is the build's compiled-in list of resolver code, identical for
     every world in the process, set before any world runs and never changed — the same kind of thing
     as the `installed!` list itself. Per-world applicability needs no filter, because a resolver is
     inert where its own state is absent: a world that does not install a resolver's pack has no table
     for that pack's components, and a read of a missing table answers `None` rather than failing.
   - `register_resolvers` sorts the list by `resolver_of()`. A list naming one id twice panics, naming
     it. If the catalog is unset, it is set. If it is set to the same ids, the call does nothing. If it
     is set to different ids, the call panics, naming both lists: one build has one catalog, and a
     second, different one is a defect of the host, not a condition to recover from.
   - **Never registered means no resolver.** `arrivals` and `arrival` consult the registered list, or
     none when nothing was registered. A process that composes worlds by hand and never registers —
     the kernel's, persistence's, the server's and every pack's own tests — therefore records exactly
     what it recorded before this decision.
6. **How the catalog is written.** `installed!` gains an optional line after `perception:`,
   `resolution: <trait path> => [ <type>, … ];`, each type followed by a comma, and expands it into
   `Capability::resolvers()`: one `Default` value of each listed type. A listed type that does not
   implement the trait does not compile; a test of the installed set holds that every listed resolver
   belongs to an installed pack. `systems/installed` writes `resolution:
   mineworld_presence::ArrivalResolver => [];` — empty until the first resolver pack is listed.
   `worldpack::compose`, the one assembly path of every host (server, `run`, `create`, `biography`,
   `inspect`, and `load` and `assemble` through it), registers `Capability::resolvers()` before it
   installs anything.
7. **A host that never registers fails loudly** (QB-15's second bound). Three guards, none in the
   kernel:
   - `compose` always registers, so a host cannot compose a world without registering;
   - **a resolver's pack refuses to join a world while its resolver is not registered**: its `install`
     calls `require_registered`, which panics — naming the pack, `register_resolvers` and this
     decision — when the catalog is unset or does not list it. A host that installs a resolver pack and
     never registered dies at assembly, before any arrival;
   - the first resolver pack checks, in its reactions to the recorded arrivals, the invariant it
     exists to keep, so an arrival that escaped resolution fails the dispatch (S15's next PR).
   A host that never registers and installs no resolver pack runs exactly as before: there is nothing
   to resolve, so nothing runs silently without resolvers. A panic rather than an error value, because
   the kernel's `install` hook refuses only with a `KernelError`, and a variant for this would be a
   kernel change.
8. **Versions.** Presence's `VERSION` is 3: its constructors now answer through the resolvers and its
   vocabulary gained a fact. `arrived` keeps schema 1. Movement's `VERSION` stays 1. A save written
   before this decision is refused by name at presence's record — "system 'presence' is v3 here, but
   the save was written by the older v2" — before any snapshot is restored (`ARC-25`: versions are
   refused, never guessed). For the same world, seed and inputs, the only bytes that differ between a
   save written before and one written after are the composition records: presence's version, and
   movement's declared emissions gaining `stopped-short`. Every fact is byte-identical while no
   resolver is listed.

**Runtime `World::disable` of a resolver pack is not honoured.** A world that disables a resolver's
pack at runtime still asks that pack's resolver, and the pack's state still answers it, because a
`WorldRead` cannot see whether a system is enabled and the catalog is per process, not per world.
Disabling exists for `AC-2` tests; `AC-2` for a resolver pack is shown at world level — a world that
does not list the pack, which this decision handles. Honouring runtime disable needs a kernel read of
enabled state, which is not decided (step-11 QB-17) and is not part of this decision.

**Tests that compose worlds by hand.** They never register, so they run with no resolver (item 5). A
test that needs resolvers registers its own list first and does not compose through `worldpack` in the
same process, because `compose` would register the build's list and a different list panics (item 5).
Cargo runs each integration-test file as its own process, which is what keeps one rule true for hosts
and tests alike.

**Accepted limitations.**
- **One catalog per process.** Two worlds in one process share it; a resolver's inertness where its
  state is absent is what keeps that harmless.
- **Runtime disable is not honoured** (above).
- **The constructor can be bypassed.** `Arrived::new` is public, and presence's reduction re-asks only
  `admit`, not the resolvers: re-resolving at reduction would run the resolvers on a displaced person's
  arrival against a state the walker's arrival has already changed. The contract's way is the
  constructor (`ARC-26`), and S15's first resolver pack catches the consequence of a bypass in its
  reactions (item 7).
- **Genesis is not resolved against other people.** A world's genesis facts are built against the
  assembled world before any of them is applied (`ARC-15`), so at genesis every `arrival` sees nobody
  placed. Two authored people placed on top of each other are refused by the resolving pack's own world
  validation, not by `arrival`.
- **No length bound on an arrival from another place, or from nowhere,** beyond "the same place": rule
  (c) applies only when the person is already in the destination place. A resolving pack's own bounds
  apply to its own resolutions; there is no seam-level number without geometry.
- **A resolver cannot emit**; its pack's consequences happen in its reactions to the recorded facts,
  which its resolver must predict.

**Note, 2026-10-07 (S15, PR 12b; step-11 §17.0 QP-1, QP-7, QR-11) — the first resolver.** The `bodies`
System Pack (`DEP-13`) is the first registered resolver. This note records what it adds to this
decision; items 1–8 are unchanged except where item 7 is refined below.

1. **Its bounds** (QR-11 moved them here from the seam). Per arrival that bodies resolves within a place:
   - no other person is moved by more than 310 mm (`NUDGE_MAX` 300 mm plus the character controller's
     10 mm `GAP`);
   - at most two generations of nudges (`CHAIN_MAX`), and at most four people moved (`NUDGED_MAX`); an
     arrival that needs more is stopped at contact instead, and nobody else moves;
   - nobody is moved into another place (rule (a) and (e) already refuse it) or through fixed
     geometry;
   - after quantization to whole millimetres, no two people in one place are closer than 595 mm
     (`CLEARANCE`, 2 × 300 mm − 5 mm), nobody's centre is outside the floor shrunk by 295 mm, and
     nobody's centre is within 295 mm of a solid. The check is on integers, and it degrades the result —
     blocked, then the advance halved up to eight times, then stay — rather than trusting the character
     controller (step-11 F-P6).
2. **Item 7, third bullet, is realized as a guard on the starting state** (QP-1). The bullet promised a
   check "in its reactions to the recorded arrivals". That check cannot be built: a reduction lets every
   subscriber react to one fact before the next fact of the same emission list is reduced, and
   `arrivals` returns the walker's `arrived` before the displaced people's, so a reaction to the walker's
   arrival sees the walker overlapping the people it has not yet seen move (step-11 F-B1). Bodies
   therefore checks, at the start of every resolution that is not inert, the place's **current** state:
   every pair of positioned people at least 595 mm apart, everyone inside the floor and out of every
   solid. Every resolved arrival keeps that state and genesis checked it, so a violation means an arrival
   escaped resolution — a constructor bypass (the limitation above) or a host that registered nothing.
   The resolver then **panics**, naming the pair (or the person), the place, the distance and this
   decision: a resolver returns a `Resolution` and has no error path, and `require_registered` already
   chose a panic for host defects. The violation is caught at the next arrival into that place, not at
   the bypassing fact itself.
3. **An arrival from another place, or from nowhere, is placed** (QP-7). A resolver cannot refuse an
   arrival or end it in another place (rule (a)), so an arrival into a place that has geometry always
   ends in that place: at `to` if a person fits there; else at `to` with the people in the way nudged,
   under the bounds of point 1; else at the nearest free point of a 50 mm lattice, with nobody moved and
   `stopped-short` recorded. Bodies' genesis check guarantees such a point exists: a place with geometry
   must hold, on a 650 mm sub-lattice, at least `4 × (people in the world − 1) + 1` free points. Rule (c)
   does not bound such an arrival (the limitation above), so the point may be farther from the doorway
   than `to` is.

**Note 2, 2026-10-08 (S15, PR 12c; step-11 §18.0, SD-O8 … SD-O10, SD-O15, QO-4, QO-9, QO-18) —
loose objects, and a pack that is also a mover.** Items 1–8 and the first note are unchanged except
where this note refines item 7 for objects.

1. **The resolver predicts object pushes.** Loose objects are `bodies`' own state (an Item with a
   `body:` section, `ARC-36` note), so presence cannot record their moves and a resolver cannot emit
   them: they move in `bodies`' reaction to each recorded `arrived`. That reaction pushes every loose
   object lying on the floor whose footprint the person's disc overlaps, straight away from the
   person's centre, by a shape cast against the fixed geometry and the other objects — computed
   **against the objects as they lay before the request**, with people not as obstacles. Reduction is
   breadth-first: every reaction of one emission list sees the objects as they lay, and the people
   not yet where the list leaves them (step-11 F-O2, F-B1). Before answering, the resolver runs the
   same function on the same three inputs — the person's end point, the room, the objects as they lay
   — for the walker and for every person it displaces, in emission order, and accepts the result only
   if no object is pushed twice, no push is cut short (a jam), and in the final state no person
   overlaps an object and no two objects overlap. Otherwise it resolves the arrival again with the
   objects solid, and nothing is pushed. So what the reactions push is exactly what the resolver
   checked, and a walker never ends inside a jammed box.
2. **The guard covers objects.** The starting-state guard (the first note, point 2) also checks that
   every object lies within its floor, rests on the floor or on one solid's top, keeps clear of every
   other solid and object, and is overlapped by no person's disc. And because a push is computed from
   the arrival and the objects alone, item 7's third bullet can be built for objects at the reaction
   itself: a push that jams there means an arrival escaped resolution, and the reaction fails the
   dispatch with `bodies-object-jammed`.
3. **`bodies` is also a mover.** Its `shove` action states presence's `arrived` and `stopped-short`
   through `arrivals()`, exactly as `movement` does (`ARC-26`): the shoved person's arrival is resolved
   by the registered resolvers like any other — walls stop them, the people behind them are nudged
   within the first note's bounds, objects are pushed or block, and the head-on bias applies to it as
   to every arrival within a place. A pack may subscribe to a fact type it also states (step-11 F-R7).
4. **One crate dependency outside presence, used for one read.** `bodies` refuses, at genesis, an
   Item that is both an object and a declared kind (`ARC-36` note, point 2). It asks the `item` pack's
   read-only `mineworld_item::is_declared`, and nothing else from that crate; a structural test holds
   that. This is a crate dependency, not a system dependency: `bodies` declares a system dependency on
   presence alone, a world may install `bodies` without `item`, and in such a world the answer is "not
   declared".

**Note 3, 2026-10-08 (S15, PR 12d-0; step-11 §20, SD-Z2, SD-Z5, SD-Z6) — bodies' cost, and one result
changed by design.** Items 1–8 and the earlier notes are unchanged except as follows.

1. **A stride away from a person within the controller's offset is not stopped by them (SD-Z5,
   FU-12c-1).** A person whose centre lies within two radii and the gap (610 mm) of the mover's start
   and on the far side of it from the stride (d · (p − start) ≤ 0) is left out of the mover's contact
   sweep. Rapier's character controller otherwise sticks on them: a shove from 600 mm moved its target
   301 mm instead of half a metre (§18.11 DO-11). Verification still counts every person, so the result
   never overlaps anybody; a stride *toward* such a person is still stopped by them. This changes
   results, so `bodies` is version 3 and refuses a version-2 save by name (`ARC-25`).
2. **The nearest free entry point is searched outward (SD-Z6).** An entry placed at the nearest free
   point of the 50 mm lattice (item E3) finds it by visiting the lattice in rings about the asked
   point, stopping once no unvisited point can be nearer, instead of filtering the whole floor: the
   same point, under the same order (distance, then y, then x), at a fraction of the cost. It was half
   of a town's with-bodies CPU (step-11 E-Z3).
3. **What was tried and not adopted, recorded so it is not tried again unmeasured.** A scene holding
   only what a stride can reach (SD-Z1), an exact integer corridor (SD-Z3) and integer wall strides
   (SD-Z4) each change results, because Rapier's controller is neither local — its answer moves when
   colliders far from the stride are removed — nor exact at its own offset from a face (step-11 E-Z1,
   E-Z2, E-Z5); none bought measurable CPU. They were dropped.

**Note 4, 2026-10-08 (S17, PR IL-a; `ARC-62`, QPL-10) — the installed set's line has a new spelling.**
The `resolution:` line of `installed!` and `Capability::resolvers()` are replaced by `ARC-62`'s generic
extension line, `extension mineworld_presence::ArrivalResolver => mineworld_presence::register_resolvers:
[mineworld_bodies::BodiesSystem,];`, and `worldpack::compose` registers it through
`Capability::register_extensions()`. No rule of this decision changes: the catalog, its write-once
storage, `require_registered`, the order resolvers are asked in and every fact are as items 1–8 and the
notes above state. Where items above say "the `resolution:` line", read "presence's extension line".

**Note 5, 2026-10-09 (S15, PR 12n-1; step-11 §21, SD-N3, SD-N4, SD-N11) — bodies is also a
wayfinder.** Items 1–8 and the earlier notes are unchanged.

1. **`bodies` answers route queries.** It implements `movement`'s `Wayfinder` (`ARC-75`), listed on
   movement's own extension line of the installed set, and its `install` calls movement's
   `require_wayfinder` after presence's `require_registered`. (Movement's guard is not named
   `require_registered`: movement's sources name nothing of the resolver seam, `seam_vocabulary.rs`.)
   A route is a plan, never a permission: every stride of a walk
   is still a `walk-step` request checked by movement and resolved by this decision's resolvers. The
   planner reads `bodies`' own state (`PlaceShape`, `LooseObjects`) and nothing else, keeps nothing, and
   answers "not mine" for a place without a shape, so a world without `bodies` walks straight.
2. **The crate dependency `bodies → movement`.** Implementing movement's trait needs movement's crate;
   it is a crate dependency, not a system dependency (`bodies` still declares presence alone), like the
   `item` read of note 2, point 4.
3. **Two person-relative bounds are expressions of the radius** (step-11 §21.6, TD-D8): `NUDGE_MAX` is
   `PERSON_RADIUS` and `BIAS_BAND` is ⌊2 · `PERSON_RADIUS` / 3⌋. At the radius of 300 mm both are the
   values the first note and 12b fixed (300 mm, 200 mm), so no result changes and `bodies`' version does
   not; a different radius scales them with the body.

**Note 6, 2026-10-10 (S15, PR 12n-2; step-11 §21.15 M-3, the operator's ruling) — a walk out of the
margin.** The resolver may leave a person nearer a wall or a solid than the planner's margin
(R + GAP + PLAN_MARGIN), though never nearer than R + GAP. From such a start the planner first steps
to the nearest free point of the 50 mm lattice, the goal's own snap (SD-N5), when that point lies
within the margin's width (360 mm at R 300). The route then goes on from there, and the step is a
stride the resolver resolves like any other. A start with no free point that near keeps the core rule
(step-11 N-D7): the box holding it is shrunk to the start for the start's own edges. Before this, a person pinned between the café's front wall and the bench's corner had no route
out, and every walk they asked for was refused `no-route` (step-11 E-NW4). This changes routes, and so
results, in every world with bodies and walkers, so `bodies` is version 4 and refuses a version-3 save
by name (`ARC-25`).

**Note 7, 2026-10-08, landed 2026-10-12 (S15, PR 12d; step-11 §19, SD-D1, SD-D5, QD-5, QD-6) — the
towns install bodies, and a doorway inside a wall is refused.** Items 1–8 and the earlier notes are
unchanged except as follows. (Planned as note 4, then 5; IL-a's note 4 and 12n's notes 5 and 6 merged
first.)

1. **Both towns install `bodies`** (SD-D1): Social Café's `systems` ends with it, after `schedule`, and
   Market Town carries it at the same position, before its six market packs. Every place of both towns
   carries the same `body:` (a floor and solid boxes), and both carry the same four loose objects. A
   person in a town can no longer walk out through a wall: a stride is swept against the place's floor
   edge and its solids, and leaving is through a doorway. Walks (`ARC-75`) are routed round them by
   this pack's wayfinder (note 5).
2. **A doorway point must lie where a person fits** (SD-D5). At genesis, `bodies` checks every doorway
   point of a shaped place — the `here` of each passage out of it and the `there` of each passage into
   it, read from `movement`'s `Passages` — to lie inside the floor shrunk by R + GAP (260 mm with note
   8's radius) and at least that far from every solid. A point that does not is refused,
   `bodies-doorway`, naming the place, the passage's other place, the point and the distance.
   Otherwise every crossing would be silently shifted by entry placement (E3) instead of landing on the
   doorway.
3. **bodies reads `Passages` from `movement`'s crate** (QD-5), beside the Wayfinder seam of note 5,
   point 2, and as it reads `item`'s for `is_declared` (note 2): a crate dependency, no system
   dependency — a world without `movement` has no passages to check. `isolation.rs` asserts it names
   nothing else of `movement` but the Wayfinder seam and `Passages`, the latter once, in the genesis
   check.
4. **`bodies` is version 5** (QD-6; 12n-2's note 6 took version 4 first): a save made by version 4 of a
   world the refusal now rejects is refused by name (`ARC-25`), never re-executed into a genesis error.
   One bump covers this note and note 8.

**Note 8, 2026-10-08, landed 2026-10-12 (S15, PR 12d; the operator's ruling, relayed by the
coordinator; step-11 §19.13, §21.6) — a person is 50 cm across.** The operator asked that bodies follow
the real world: a 600 mm person is too wide (adult shoulders are 40–46 cm, chest depth 25–30 cm).
**`PERSON_RADIUS` is 250 mm**, changed inside 12d so that it shares 12d's one re-baseline. Every person
dimension keeps its formula and moves with it, none re-typed: `CLEARANCE = 2r − TOLERANCE` (495 mm), two
people stopped `2r + GAP` (510 mm) apart, a doorway point and an entry `r + GAP` (260 mm) from walls and
solids, a pushed object's footprint `r + GAP` from its pusher, the narrowest floor `2(r + GAP)`
(520 mm), the capsule's half-segment `(PERSON_HEIGHT − 2r) / 2`, and — by note 5, point 3 — `NUDGE_MAX =
r` (250 mm; a nudge is at most 260 mm) and `BIAS_BAND = ⌊2r/3⌋` (166 mm); the planner's margin
`r + GAP + PLAN_MARGIN` is 310 mm. What is not a person dimension does not move: `BIAS_TURN`,
`REST_CLEARANCE` (300 mm, how far a launched object rests from furniture), the reaches,
`SHOVE_DISTANCE`, `CAPACITY_GRID`, and the tolerances `GAP`, `TOLERANCE`, `SNAP`, `LATTICE`. At this
radius the crowd claim reads as the operator restated it (step-11 §21.15 M-1): nudge chains are bounded
and a stride may be blocked; a blocked walker re-plans. Results change, so the `bodies-yard` and
long-run bases are re-captured (step-11 §19.12), under note 7's version 5. The 3D client's capsules
(`player.gd`, 300 mm) must follow (12e/16c); IL-c is to make the radius world configuration.

---

## DEP-13 — Server physics: Rapier (`rapier3d`, `enhanced-determinism`) inside the `bodies` pack

**Date** 2026-10-07 · **Status** selected; dependency added in S15 PR 12b · **Approved by** the
operator (step-11 D-1 … D-5, QB-4, QB-12) and the primary session at PR 12b's design freeze (step-11
§17.0: QP-3 overruled, so the pin lives in the pack's own manifest) · **Relates to** `ARC-25`,
`ARC-30`, `ARC-33`, `ARC-39`, [`MVP.md`](MVP.md) §9 `AC-8`, `AC-12` · **Design**
`.structured-coding/plans/mvp0/step-11-bodies.md` §§3, 6, 9, 15.1, 17 (S15, PR 12b)

**Problem.** People must not pass through each other or through walls, and later must push, kick and
throw objects (the operator, 2026-10-07). The answer is decided on the server and must replay byte for
byte (`ARC-25`), on more than one machine (`AC-8`). Sweeping a body against walls and other bodies, and
later integrating a kicked object until it rests, is a physics engine's work.

**Options considered** (`REUSE_POLICY.md` §§11–12, §17 — both directions).

```text
(a) rapier3d (dimforge), Apache-2.0           chosen
(b) parry3d alone (Rapier's geometry crate)   no dynamics: kick and throw would need our own integrator
(c) our own circle-and-box code               a physics engine is commodity infrastructure
(d) Jolt through Rust bindings                bindings unmaintained since May 2024; cannot enable Jolt's
                                              cross-platform determinism
(e) Avian                                     requires Bevy's ECS and scheduler
```

**Choice: `rapier3d =0.36.0`**, with the feature `enhanced-determinism`, and without `simd8`,
`parallel` and `serde-serialize`. Declared **in `systems/bodies/Cargo.toml` only**, not in the root
`[workspace.dependencies]`: installing a pack touches only `systems/**` and `Cargo.lock` (`ARC-33`),
and the primary session chose that over the root's one-version convenience (step-11 §17.0, QP-3). No
other crate depends on it.

**Why not ourselves** (`REUSE_POLICY.md` §12). Kick and throw need integration, contacts, friction and
rest, which is a physics engine (`REUSE_POLICY.md` §4). Ours would also have to earn the cross-platform
determinism Rapier documents and the prototype measured (step-11 §§9.4, 9.8).

**Why not the others.** `parry3d` alone has no dynamics (missing required semantics). The Jolt bindings
are unmaintained and cannot enable Jolt's determinism (unmaintained project). Avian would put the
simulation inside Bevy's execution model (architecture mismatch, unacceptable lock-in).

**Isolating interface.** `systems/bodies/src/rapier.rs` is the only source file that names
`rapier3d`, and a structural test holds that. It takes integers in — a place's floor and solids, the
people standing in it, one sweep request — and gives integer millimetres out. No Rapier type appears
in a component, a fact, a contract, a public signature or another crate. Nothing of Rapier survives a
call: every resolution builds a fresh world for one place, in a canonical insertion order, and drops
it. Every float is converted from integers by one function and quantized back by one function,
`(metres × 1000).round()` to `i32`.

**Facts, re-verified 2026-10-07** (the crate's own `Cargo.toml` in the local registry, and crates.io,
the licence file, the determinism guide and the changelog):

```text
rapier3d     newest non-yanked on crates.io: 0.36.0, published 2026-09-25 (sebcrozet); 71 versions.
             The dimforge CHANGELOG's top heading is v0.36.1 (2026-10-04, Python bindings only),
             which is NOT on crates.io. Pin: =0.36.0.
licence      Apache-2.0 (crates.io field and the crate's Cargo.toml; LICENSE: "Apache License
             Version 2.0", "Copyright 2020 Sébastien Crozet"). Compatible with MineWorld's MIT.
MSRV         rust-version 1.86, edition 2024 (crate Cargo.toml and crates.io). Ours: 1.97.1.
features     default = [dim3, f32, std]
             enhanced-determinism = [simba/libm_force, parry3d/enhanced-determinism]
             parallel = [dep:rayon, std, parry3d/parallel]
             simd8 = [parry3d/simd8]  — the only SIMD feature in 0.36.0
parry3d      ^0.31.1 required; 0.31.1 newest (2026-09-18), Apache-2.0, no MSRV declared; 0.31.0 yanked
determinism  cross-platform needs `enhanced-determinism` and IEEE 754-2008 targets; it cannot be
             combined with `simd8`; inputs computed with functions beyond + − × ÷ must use nalgebra's
             ComplexField/RealField. MineWorld computes every direction and length in integers instead.
```

**Accepted limitations.**
- **Monthly breaking releases** (0.33 to 0.36 in four months). The pin is exact. An upgrade is a change
  of results, so it bumps `bodies`' `VERSION` and an old save is refused by name rather than diverging
  on replay; a test holds the pack's version and the locked Rapier version together.
- **A defect in 0.36.0, worked around** (step-11 F-P1). `PhysicsWorld::detect_collisions` on a fresh
  world leaves every dynamic body un-integrated by the next step. The cause is upstream behaviour
  (changelog v0.35.0: "`CollisionPipeline::step` now clears the rigid-bodies' modified flags"). The
  adapter re-marks each dynamic body after detecting collisions; a canary test asserts the defect as it
  is, so an upstream change turns it red and the workaround is then removed deliberately.
- **The character controller is not an interpenetration guarantee** (step-11 F-P6). The pack verifies
  the non-overlap invariant on integers after every resolution and degrades the result (`ARC-39` note).
- **Cross-architecture identity is evidenced under Rosetta only** (arm64 and x86_64 translated, on one
  machine, step-11 §9.8 and PR 12b's PB-12). A native x86_64 host (S13) is the stronger check.
- **No momentum between resolutions.** Everything is at rest between requests; a kicked object's whole
  motion is resolved at the instant of the kick (S15's later PR).
- **The licence tree is permissive**: every package Rapier brings, with these features, is Apache-2.0,
  MIT, Zlib, Unlicense or Unicode-3.0, each alone or as one of a permissive choice (recorded in step-11
  §17.10).

**Note, 2026-10-08 (S15, PR 12c; step-11 §18.0, SD-O14, SD-O16) — dynamics are now used.** `kick` and
`throw` simulate one object's flight at the instant of the request (step-11 QB-6): the first stepped
simulation in MineWorld. The isolating interface is unchanged — integers in, integers out, nothing
kept, every float converted by the same two functions — and what the flight adds is bounded:
- **One dynamic body per flight.** A flight scene holds the place's fixed geometry, the people as
  kinematic capsules in `EntityId` order, the other objects as fixed colliders in `ItemId` order, and
  the flying object last: dynamic, rotations locked (no rotation is ever persisted), with continuous
  collision detection so a fast object cannot pass a thin solid. Friction 0.5, restitution 0.1.
- **Bounded steps and a rest rule.** Sub-steps of 1/60 s, at most 180 for a kick and 240 for a throw;
  the object has come to rest once it has been slower than 50 mm/s for 10 consecutive sub-steps. Both
  are pure functions of the state.
- **F-P1's re-mark is exercised.** The adapter's `refresh` runs after the scene is built and before the
  first step; without it the flying object would never move (the canary above).
- **The engine's answer is never the last word.** The landing is quantized and verified on integers
  against the stored-state invariant of objects; if it fails, the nearest verified point of a 50 mm
  lattice is taken, and if none verifies, the object stays where it was.
- **Launch velocities are integers** (millimetres per second, computed with + − × ÷ only), converted
  into Rapier's metres by the adapter's one conversion.

**Note, 2026-10-08 (S15, PR 12d-0; step-11 §20, SD-Z2, SD-Z5) — what of Rapier a people scene now
uses.** A scene of people builds Rapier's broad phase and nothing else (`BroadPhaseBvh::update`, the
update `CollisionPipeline::step` makes, without the narrow phase): no query of a sweep reads contacts.
A `Pile` and a flight keep the full collision detection and F-P1's re-mark. A contact sweep may leave
named people out by a query predicate (SD-Z5); a sweep that leaves nobody out builds exactly the filter
it built before. Two properties of the character controller were measured and are now part of this
decision's record: it is not local (its end moves by up to 14 mm when colliders far from anything the
stride touches are removed from the scene), and it is not exact at its own offset from a face (a stride
starting at or gliding along it may stop up to 1.6 m short, or drift 1–2 mm in open floor) — step-11
E-Z1, E-Z2, E-Z5. A pruned scene or an integer replacement of the sweep therefore changes results.

---

## ARC-40 — Who controls a Person is host state: a binding change moves no revision

**Date** 2026-10-08 · **Status** accepted; implemented in S11 PR S11-B · **Approved by** the primary
session at S11-B's design freeze (step-12 §16, QS11B-1 … QS11B-6 as recommended) · **Relates to**
`ARC-25`, `ARC-27`, `ARC-42`, `INV-1`, [`NETWORKING.md`](NETWORKING.md) §§9–10 · **Design**
`.structured-coding/plans/mvp0/step-12-server.md` §§4.2–4.4, 16 · **Specification**
[`server/PROTOCOL.md`](../server/PROTOCOL.md) §4.2

**Problem.** `INV-1` says a Person exists independently of whatever decides its actions, and that
control may change at runtime without altering the Person; `AC-5` asks that a human take over an
existing NPC with biography, relationships, inventory and employment intact. Until S11-B a seat was not
exclusive: a human joining `alice` while `--agent alice` drove her made two controllers on one Person,
and nothing in the server knew who drove a seat.

**Choice.** Who drives a seat is **host state**, kept by the world thread's `SeatTable`
(`server/src/seats.rs`), its only writer: each seat is `free`, `hosted` (an in-server controller),
`connected` (one connection) or `held` (its connection dropped; nobody drives it until the hold ends or
the connection's resume secret returns). A binding change — a join, a takeover, a `leave`, a drop and its
hold, a resume, a hold's expiry, a release to the seat's default — is **never journaled, states no fact
and does not move the world's revision**. Seat bindings, sessions, holds, resume secrets and nicknames
appear in no save. A seat returns to its default with a controller built afresh from its configuration,
never one resumed from memory.

**Why this is how `AC-5` is measured.** If control changes and the journal does not, the change cannot
have altered the Person: everything the Person is — biography, `knows` edges, holdings, wallet, job — is
a projection of the fact log (`ARC-25`, `ARC-29`), and the log moved only by requests somebody submitted
through the one authority path. S11-B's acceptance reads the save's revision across every binding change
and requires every fact added while a human held the Person to name an `ActionId` that human's session
was answered with.

**Options considered.** (a) Bindings as host state on the world thread — chosen: the binding must live
where the world lives (step-12 A-9), and nothing about it is the world's business. (b) Bindings as a
component or a fact (`controlled-by`) — rejected: control would become world state, a replay would
re-bind controllers, and `INV-1` would be false by construction. (c) Bindings in the transport, per
session — rejected: two sessions could each believe they held one seat (the A-1 defect), and an
in-server controller is not a session. The pattern of hold-and-resume follows Colyseus's reconnection
(a grace window, a token refreshed on every connection, the stale connection superseded) as a reference,
not a dependency (step-12 §7.4).

**Limitations accepted (MVP-0).**
- **One level of trust.** Any holder of the server's invite may take any seat with `take_over: true`,
  including a dropped player's held seat. MVP-0 has one shared invite and no per-player identity
  (`NETWORKING.md` §9: "matchmaking, global accounts" are non-goals), so the server cannot tell the
  dropped player from a friend on the same invite; a finer policy — for example, only the operator may
  take a held seat — needs accounts. This is LAN-friends trust, stated rather than implied (QS11B-3).
- **A hold and a pace are both wall time.** `--hold` counts wall seconds (a hold is about a network).
  `--pace` counts wall seconds too — **amended 2026-10-08 by operator ruling QTW-13**
  ([`step-19-time-weather.md`](../.structured-coding/plans/mvp0/step-19-time-weather.md) §4.4), which
  reverses QS11B-4's "a pace is about a life": a hosted Person walks one stride per consult and walking is
  embodied, so the time scale must never make it walk or talk faster. An adapter states its cadence to
  the seam as `cadence × time_scale` world seconds (the reactive controller's one wall second likewise).
  With `--time-scale 60` a 30-second hold is half an hour of the world, and a paced seat is consulted
  every 5 wall seconds, 300 world seconds. Headless `run` is unaffected: it has no wall clock and keeps
  its 900-second world pace (`ARC-27`).
- **Nothing survives a restart.** Holds and resume secrets are host state and are not persisted; after a
  restart a player joins again and finds the seat free or hosted.
- **"Missed the moment."** A controller bound at an instant treats every line heard at or before that
  instant as not its to answer (`RuleController::since`, `F-13`), so a line said in the same world second
  a controller is bound is not answered — the limitation `ARC-27` already accepts for consults.

---

## ARC-45 — A 2D client draws one town from disclosed passages, and remembers it only as presentation

**Date** 2026-10-08 · **Approved by** the primary session at S12 PR 13a's design freeze · **Relates
to** `ARC-26`, `INV-5`, `INV-13`, [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §6.1 · **Design**
`.structured-coding/plans/mvp0/step-13-client-2d.md` §§4.3, 14

**Problem.** Every `Place` has its own frame, and a World Pack authors no geometry beyond positions
and passages. A 2D client must still draw one continuous town — a street with doors you walk through —
without inventing a fact the world did not state and without binding its drawing to one world's keys
or coordinates.

**Decided.**

1. **Frames are glued at disclosed passages, by translation only.** The place the client first
   stands in is the root of the drawing. A place `P` reached through a passage with `here = h` (in
   the place it was disclosed from, `Q`) and `there = t` (in `P`) is drawn with its origin at
   `origin(Q) + h − t`. Never rotated, never scaled: `+x` east and `+y` north everywhere.
2. **The layout is learned, per world instance, in memory.** A passage is learned when an
   observation discloses it; a place's tags when the observer stands in it. The cache holds no world
   truth and nothing is submitted from it except positions the player chose. It is dropped when the
   `instance` changes. A cache on disk across launches was considered and not chosen: it is state
   kept for a convenience that the world disclosing a doorway's destination (step-13 R-PK-1) provides
   properly.
3. **Appearance comes from world data.** A known place's look is chosen by its tags; a place not yet
   visited is drawn with a generic façade chosen by a stable hash of its id **string**. A person's
   look is chosen from tags and the same hash; a label is `display_name`, else the id.
4. **Dressing anchors only to disclosed doorways or to the extent they span.** Terraces, planters,
   lamps and trees are placed relative to a doorway, or to the rectangle the disclosed doorways of
   the root place span. Nothing is positioned by a world key or an absolute coordinate, so the same
   pack dresses any world.
5. **Decoration is not enforced.** Where no `place-shape` is disclosed, a place's footprint is drawn
   from the pack and never used to refuse or shorten a stride. Where one is disclosed (S15), the
   server's numbers are the only blocking geometry a client draws.
6. **Only the perceived is drawn.** Nobody outside the observer's place is drawn; other buildings
   are façades, honestly empty (`INV-13`).

**Limitation accepted.** Until a doorway names where it leads, a neighbouring building looks
generic until it has been entered once in the running instance.

---

## ARC-46 — A Presentation Pack binds roles to art; a client core names no asset

**Date** 2026-10-08 · **Approved by** the primary session at S12 PR 13a's design freeze · **Relates
to** `ARC-1`, `ARC-3`, `ARC-11`, `ARC-14`, [`MODULE_SPEC.md`](MODULE_SPEC.md) §6.1 · **Specified in**
[`clients/2d/PRESENTATION.md`](../clients/2d/PRESENTATION.md)

**Problem.** `MODULE_SPEC.md` §6.1 names `assets/asset_bindings.yaml` and `renderer/godot.yaml` and
specifies neither. The 2D spike resolved art through role maps compiled into its scene script, so its
four variants (`ARC-14`) were code, and the pack it drew from was its own project directory.

**Decided.**

1. **The core asks for roles, never for files.** `person:<n>`, `player`, `facade:<tag>`,
   `facade:unknown:<n>`, `ground:<tag>`, `interior:<tag>:<fitting>`, `prop:<name>`. The pack's
   `assets/asset_bindings.yaml` resolves a role to a file under the pack, per binding set; a role a
   set does not bind is drawn plainly (shapes and text), so the client runs and is testable with no
   pack at all (`--presentation=none`).
2. **`ARC-14`'s variants are binding sets** in one pack: `town` (default), `full`, `people`,
   `procedural`, selected by `--variant=`.
3. **`renderer/godot.yaml`** states the renderer's parameters: projection, pixels per metre, the
   sprite scale rule, the post-process and contour shaders, façade dressing and interior footprints
   per tag, all anchored as `ARC-45` point 4 requires.
4. **The pack is data, loaded at runtime from any directory**: images through `Image.load_from_file`
   and `Image.load_svg_from_buffer`, shaders from their source text. It is never imported into the
   client's Godot project and it contains no GDScript, so `--presentation=<dir>` takes any pack.
5. **Both files are written in YAML's JSON-compatible subset** (`DEP-16`): valid YAML for any YAML
   tool, read by Godot's built-in JSON parser. Their comments live in `PRESENTATION.md`.

**What it does not decide.** Whether an interaction is valid (`ART_DIRECTION.md` §19), or anything
about a world: a pack may depict any world, and a world may be depicted by any pack.

**Note 1, 2026-10-14 (S26-a; `ARC-92`) — the role list gains `look:<key>`.** A person's disclosed `look` (`ARC-91`) is asked for as `look:<key>` before `person:<n>`; for the observer, before `player`. A `look` is a disclosed component value, like the tag in `facade:<tag>`, so "the core never asks for a role by a world key" stands, and the list of role kinds gains one. Specified in `clients/2d/PRESENTATION.md` §3.1; read by the client from S26-e.

---

## ARC-47 — No rule in the client, made executable

**Date** 2026-10-08 · **Approved by** the primary session at S12 PR 13a's design freeze · **Relates
to** [`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §§4, 7–9; `INV-5`; `INV-9`; `ARC-23` · **Design**
step-13 §§4.4, 8, 14.2 (AC-W5 … AC-W8)

**Problem.** "The client implements no world rule" has so far been a review convention. A convention
is checked when someone remembers to; the 2D client is the place it is easiest to break, because a
renderer that already knows positions is one comparison away from deciding a distance.

**Decided.** The rule is a property tests hold, in two halves.

1. **Structure, scanned** (`scripts/check_client_rules.py`, on `clients/2d/**/*.gd`, comments
   excluded): only `scripts/intents.gd` submits; no other file names an action type that file
   composes; a function that submits reads no verdict (`may(`, `"available"`, `unavailable_reason`,
   `requirement(`); distances are computed only where they are request sizes, routes or drawing
   (`walker.gd`, `projection.gd`, `town.gd`, `scene/`); only `link.gd` opens a connection. Each rule is
   shown to report a planted violation by file and line before it is trusted.
2. **Behaviour, against a server that lies** (a test-only revision-1 stub): an offer marked
   unavailable is still submitted when the player chooses it; a refused stride ends the walk and the
   body is drawn where the next observation puts it; an observation that moves the player is followed,
   never argued with by a correcting `move`. Each is shown to fail under the planted mutation.

The client composes only the requests it knows how to ask for, decides nothing, never refuses to
submit what the player chose, and never retries a request whose answer it did not see (a retry
could do a thing twice; the server allocates identity, `INV-6`).

**Note 1, 2026-10-08 (S12 PR 13b; step-13 §15.4 D-b-1 … D-b-6, QS13b-5) — the interaction menu is the
observation's affordances.** No new principle; how this record applies to menus:

1. A menu is about one subject — a perceived person, or the observer itself — and lists exactly the
   affordances the newest observation offers against that subject (target-less ones for the
   observer), **in the server's order**, one entry per affordance: nothing sorted, grouped out of
   order, removed or invented. Several complete affordances sharing an action type and a target are
   several entries.
2. A **complete** affordance is submitted unchanged through the shared module's `submit_affordance`;
   a **composable** one (its type is in `intents.gd`'s `COMPOSED`) through that file's composer, with
   the player's input sent exactly as typed; any other is listed disabled, "not supported by this
   client", and never sent.
3. An entry the server marks unavailable is drawn greyed **and stays choosable**, with the server's
   reason in the client's wording and, where declared, the requirement's range, unevaluated. Choosing
   it submits it; the answer is the server's. No key submits "the first available" entry: selecting
   by the server's verdict is the client deciding.
4. The only entry a client adds is "walk to <person>", which is the offered `move` (listed only when
   the frame offers a target-less `move`), walked as ordinary strides the server answers one by one.
5. The scan grows with the menu: R2 admits `app.gd`'s `"invite"` (the join option) by explicit entry
   only; R3 extends to `menu.gd` and `hud/` (a function that calls `intents.` reads no verdict); R6
   refuses a user-visible English sentence outside the wording (`ARC-70`).

---

## ARC-70 — Client UI text is addressed by message keys; a Presentation Pack carries the wording as gettext files

**Date** 2026-10-08 · **Approved by** the primary session at S12 PR 13b's design freeze (QS13b-1) ·
**Relates to** `ARC-46`, `ARC-47`, `DD-13`, `DEP-16`, [`REUSE_POLICY.md`](REUSE_POLICY.md) · **Design**
step-13 §15.4 D-b-9; [`clients/2d/PRESENTATION.md`](../clients/2d/PRESENTATION.md) §7

**Problem.** The contract carries no display text (`DD-13`): an action type, a rejection code and a
component are identifiers, and a client words them. The 2D client's menus, results and panels add
dozens of user-visible strings. The operator requires every client UI string to be translatable,
with translations as standard files in the Presentation layer, so that a user adds a language
without code ("Framework, not demo" item 4, `en` default, `zh-Hans` next).

**Decided.**

1. Every user-visible string of a reference client is a **message key** looked up through the
   engine's translation server, with arguments substituted after the lookup. Keys are built from data
   where the thing worded is data: `action.<type>`, `action.<type>.done`, `reason.<code>` — so no
   client file holds an action type or a reason code to word it. Fixed UI text uses `ui.*`, panel
   titles `panel.*`, suggestion lists `suggest.*`, formats `format.*`.
2. A Presentation Pack carries the wording as **gettext `.po` files**, `i18n/<locale>.po`, loaded at
   runtime from the pack directory (never imported into the client project). `en` is the default and
   the one 13b ships.
3. A key the wording does not hold falls back to a readable form of the key's last part
   (`reason.too_far_away` → "too far away"); a client with no pack still says something true. A code
   the wording does not know is therefore shown as the code itself.
4. Wording holds no rule: no number, no condition, no filter. A suggestion list is a suggestion; what
   the player types is sent unchanged and the server decides whether it is valid.
5. The language switch, `zh-Hans` and the setting's persistence belong to the client-settings lane,
   which may change the file format behind the same keys.

**Options compared** (`REUSE_POLICY.md`, adopt → adapt → extend → build):

```text
Godot TranslationServer + gettext .po, loaded at runtime   adopt: the engine's own; .po is the standard
                                                           translators' tools edit; Godot 4.7 loads a .po
                                                           from an absolute path unimported (probed)
Godot CSV translations                                     reject for packs: need the editor's import
                                                           step, so a pack outside the project cannot be
                                                           loaded at runtime
our own JSON wording table                                 reject: non-standard; the operator asked for
                                                           standard files
third-party i18n addons                                    none needed over the engine's own
```

**Isolating interface.** One script per client reads the wording (`clients/2d/scripts/hud/words.gd`);
every other script asks it for a key's text.

**Note 2026-10-09 — extended to both clients by S20 SET-a** (design step-20 §3.6, ruling QSET-11,
QSET-16; this note stands in for the planned `ARC-SET-b`). The format and every key of point 1 are
unchanged. Catalogs are **layered per key**: the shared settings module's `locale/` (keys both clients
use, among them `action.*` and `reason.*`, moved verbatim from the 2D pack), then the Presentation Pack's
`i18n/` (the pack's own wording, and any override of a shared key it marks `#. override`), then the
user's `user://locale/`, the later layer winning. Languages are discovered from the `.po` files present
(a language is offered when some layer gives it a `language.self_name`), so a `fr.po` dropped into any
layer adds French with no code. The language is switched live. The fallback is English, then point 3's
readable form; a raw key is never shown. World content, stdout logs and evidence lines are never
translated. `words.gd` keeps its API and delegates loading and the fallback to
`clients/shared/settings/text.gd`. The translation comparison above is confirmed for both clients by
step-20 §4.1.

---

## DEP-16 — The 2D reference client: Godot built-ins and the shared protocol module, no addon

**Date** 2026-10-08 · **Status** selected · **Approved by** the primary session at S12 PR 13a's design
freeze · **Relates to** `DEP-4`, `DEP-9`, `ARC-46`, [`REUSE_POLICY.md`](REUSE_POLICY.md) · **Design**
step-13 §5 (the comparison), §14.4

**Problem.** The 2D client needs drawing, UI widgets, a connection, routing and pack-file reading.
Each has mature candidates; some would put MineWorld inside another framework's execution model.

**Choice, per piece** (step-13 §5 holds the full comparison and sources):

```text
drawing          Node2D + Sprite2D, y-sorted, Camera2D, canvas shaders     adopt (built-in)
                 TileMapLayer                                              only if ground drawing is too slow
                 Tiled + YATI, LDtk + importer                             reject: a second source of layout
widgets          Control nodes + Theme                                     adopt (built-in)
                 Dialogue Manager, Dialogic 2                              reject: a second source of conversation
connection       clients/protocol/mineworld over WebSocketPeer             adopt (ours, already shared with 3D)
                 Godot MultiplayerAPI, Nakama                              reject: another authority model
                 godot-rust (gdext)                                        reject for now: native per platform
routing          straight strides through disclosed doorways               adopt
                 NavigationServer2D from a disclosed place-shape           adopt when a place discloses solids
pack files       Godot's JSON parser over YAML's JSON-compatible subset    adopt
                 fimbul-works/godot-yaml, KoBeWi/Godot-YAML (GDExtension,  reject: native per platform
                 RapidYAML)
                 YAML.gd (pure GDScript)                                   reject: an unvetted parser for two
                                                                           small files we author
test harness     a scripted scene printing PASS/FAIL, driven by Rust tests adopt; GUT/gdUnit4 not needed yet
```

**Why not ourselves.** Nothing here is built that a built-in provides; the client's own code is
acquisition, drawing and the protocol module that already exists.

**Isolating interface.** Pack files are read in one script (`clients/2d/scripts/presentation.gd`); a
YAML reader, if a pack author ever needs full YAML, replaces that one call.

**Limitation accepted.** Pack files cannot carry comments; their documentation is a specification
file beside the client. Revisit when a pack author needs full YAML.

Sources checked 2026-10-08: [fimbul-works/godot-yaml](https://github.com/fimbul-works/godot-yaml) ·
[KoBeWi/Godot-YAML](https://github.com/KoBeWi/Godot-YAML) ·
[YAML.gd](https://godotengine.org/asset-library/asset/4120), plus step-13 §5's list.

---

## ARC-41 — Client protocol revision 2: specified whole, landed incrementally, and still JSON

**Date** 2026-10-08 · **Status** accepted; revision 2 begins on the wire in S11 PR S11-A · **Approved by**
the operator (QS11-1, QS11-15 as recommended, `overall.md` "Parallel build-out, 2026-10-08") and the
primary session at S11-A's design freeze · **Relates to** `DEP-3`, `ARC-25`, `ARC-34`, effort decision
`D-4` · **Design** `.structured-coding/plans/mvp0/step-12-server.md` §§5, 15 · **Specification**
[`server/PROTOCOL.md`](../server/PROTOCOL.md)

**Problem.** Step S11 changes the client protocol in five pull requests — authentication, seats and
hold, facts and deltas, admin, proof — while the 2D client (S12), the 3D client (S14) and the Python
cognition SDK (S10) are built against it at the same time. One revision number per pull request would
make three clients chase four revisions; one revision delivered at the end would block all three until
S11 finishes. Separately, effort decision `D-4` said Protobuf would be introduced "at the first real
cross-language boundary", and that boundary — the Godot client — has run on JSON since S5V.

**Choice, the revision rule.** Revision 2 is **specified whole** (step-12 §5, then `PROTOCOL.md`) and
**implemented incrementally**: `protocol` becomes `2` in the first pull request that changes the wire
(S11-A), and until the step completes a server may *omit* a frame or a field the specification marks as
arriving later, and never gives a frame or a field it sends a different meaning. `PROTOCOL.md` §10 is
the landing table. A client written against the whole specification is correct against every
intermediate `main`. Fields other steps asked for (the explicit takeover flag and time scale, S11-B;
`acted_through` and the cursor-resumable perceived stream, S11-C) are specified in `PROTOCOL.md` by the
pull request that lands them; until then a `join` carrying them is refused as malformed, because every
client frame denies unknown fields.

**Choice, the encoding.** **JSON text frames stay the wire encoding for MVP-0.** This supersedes
`D-4`'s *timing*, not its direction: the Rust types in `mineworld-contracts` remain the single source of
truth, and other languages mirror them — checked against golden frames the Rust tests keep in
`server/tests/frames/` — rather than through a parallel schema. Protobuf (`prost` with `godobuf`) was
declined for now because a mirror `.proto` of every contract type is the drift risk `R-3` names, the
GDScript generator has a single maintainer, and payloads owned by System Packs would be JSON inside
bytes anyway. MessagePack (`rmp-serde`) is the natural first step if a binary encoding is ever measured
to be needed, because it keeps `serde` as the one source; CBOR has no maintained GDScript decoder;
transport compression (`permessage-deflate`) cannot be negotiated by Godot's `WebSocketPeer`
(godot#103230). Bandwidth is answered by S11-C's measured deltas, not by the encoding.

**Disagreement recorded.** [`ARCHITECTURE.md`](ARCHITECTURE.md) §13.1 still states `D-4`'s timing. It is
stale; the edit belongs to S10's P3 (step-17 G-1), which introduces the second cross-language consumer,
and is recorded here so the contradiction is not silent (`CLAUDE.md` §2.1 rule 4).

**Limitation accepted.** Between S11's pull requests, two `main` commits that both say `protocol: 2`
differ in what they send. The landing table is the only place a client learns which; a client for a
deployed server built from an intermediate `main` reads that commit's `PROTOCOL.md`.

---

## DEP-14 — Join secrets: `getrandom` to make them, `subtle` to compare them

**Date** 2026-10-08 · **Status** selected; dependencies added in S11 PR S11-A · **Approved by** the
primary session at S11-A's design freeze (step-12 §7.2, DEP-S11-a) · **Relates to** `DEP-3`, `ARC-41`,
[`NETWORKING.md`](NETWORKING.md) §9 · **Design** `.structured-coding/plans/mvp0/step-12-server.md`
§§4.1, 7.2, 7.7, 15

**Problem.** MVP authentication is a server invite token plus a player nickname (`NETWORKING.md` §9).
The server must make an unguessable invite when its operator gives none (and, from S11-B, a resume
secret per seat binding), and must compare an offered invite with the real one without the comparison's
timing telling an attacker how many leading characters were right.

**Options considered** (`REUSE_POLICY.md` §§11–12, §17 — both directions).

```text
credential carriage
(a) invite + nickname in the protocol's join frame, checked by
    server/src/admission.rs                                    chosen: transport-independent, works for
                                                               a browser client, never in a URL
(b) tower-http bearer validation on the /ws upgrade           a browser cannot set the header; it
                                                               authenticates the transport, not the
                                                               protocol
(c) JWT / PASETO session tokens (jsonwebtoken)                accounts and expiry nobody has yet
(d) axum-login / tower-sessions                               cookie sessions and a user model
                                                               MineWorld does not have
constant-time comparison
(e) subtle 2.6 (ConstantTimeEq)                               chosen
(f) constant_time_eq 0.6                                      acceptable; subtle preferred for its
                                                               review history
(g) hash both sides (sha2) and compare                         an indirect argument for no gain
(h) our own fold-and-or loop                                   the classic way to be undone by an
                                                               optimizer
secret generation
(i) getrandom 0.3 (the OS random source)                      chosen: already compiled
(j) rand 0.9                                                  a full RNG API for 16 bytes
(k) uuid v4                                                   an identifier, not a secret format
(l) WorldInstanceId::allocate's clock ⊕ pid ⊕ ordinal         guessable
join rate limiting
(m) one guess per connection + a fixed 500 ms delay (ours)    chosen for MVP-0, a LAN or a tunnel
(n) governor / tower_governor (GCRA per IP)                   deferred: the adopt route for public
                                                               hosting
```

**Choice.** `subtle = "2.6"` and `getrandom = "0.3"`, both in the root `[workspace.dependencies]` and
used by `mineworld-server` only.

**Facts, verified 2026-10-08** (`cargo info`, `cargo tree`):

```text
subtle            2.6.1, BSD-3-Clause, dalek-cryptography/subtle; no dependencies; default features
                  std, i128. New to Cargo.lock.
getrandom         0.3.4, MIT OR Apache-2.0, rust-random/getrandom. Already in Cargo.lock as a normal
                  dependency of mineworld-server (rand_core ← rand ← tungstenite ← tokio-tungstenite
                  ← axum), so this adds an edge, not a package. 0.4.3 exists; 0.3 avoids a second copy.
constant_time_eq  0.6.1, CC0-1.0 OR MIT-0 OR Apache-2.0 — the recorded alternative.
```

**Why not ourselves.** Both are commodity primitives (`REUSE_POLICY.md` §4); an own constant-time loop is
exactly what an optimizer can turn back into an early exit. **Why ours for the rate limit.** One guess
per connection and a fixed delay are a few lines and suffice for a 128-bit invite on a LAN; `governor`
needs per-IP keying that is wrong behind a proxy, and is the adopt route the day public hosting is in
scope.

**Isolating interface.** `server/src/admission.rs` is the only file that names either crate. It exposes
`InviteToken` (generated or operator-given; `Debug` redacted; no `Serialize`), `OfferedInvite`,
`Nickname`, `Admission::admit` and `UNAUTHORIZED_DELAY`. No secret type implements `Serialize`, so none
can reach a frame, a fact or a save by accident.

**Limitations accepted.** `subtle`'s slice comparison returns early on unequal lengths: an invite's
length is not secret (a generated one is always 32 characters). Secrets cross a LAN in clear over
`ws://`; TLS comes from a gateway or a tunnel (`NETWORKING.md` §7, QS11-14). An operator-given invite on
the command line is visible to other local users through the process list; `MINEWORLD_INVITE` avoids
that.

---

## ARC-42 — In-server controllers run on the world thread behind a bounded synchronous seam

**Date** 2026-10-08 · **Status** accepted; implemented in S11 PR S11-B · **Approved by** the primary
session at S11-B's design freeze (step-12 §16; QS11-2) · **Relates to** `ARC-27`, `ARC-40`,
[`ARCHITECTURE.md`](ARCHITECTURE.md) §9 · **Design** `.structured-coding/plans/mvp0/step-12-server.md`
§§4.5, 7.5, 16

**Problem.** A hosted world had no walkers: `mineworld server` ran only `--agent` seats, as tokio tasks
that joined a seat like a client and decided on whichever observation the 10 Hz `try_send` stream
happened to deliver. The paced controller that makes a town live (`ARC-27`) must be consulted at
instants it can name — once per pace, on a lattice — which only the thread that owns the clock can
guarantee.

**Choice.** `server/src/hosted.rs` declares `HostedController`: `next_consult(after)` names the next world
instant it wants, `decide(&observation)` returns at most one `ActionRequest` from its own Person's
observation, and `answered(&answer)` tells it what became of that request. It is **synchronous and
bounded**: `decide` must return in microseconds and never block, wait or call out. On every tick the world
thread consults each hosted seat whose instant is due — in instant order, then seat order — computing
that seat's observation through the same perception call a session gets and passing any request through
the same `submit_at` a session's request takes (actor check, server-allocated `ActionId`, journal before
answer). Consults due while a person drives the seat are skipped, never queued. The server names no
controller crate: `tools/cli` adapts the reactive and the paced rule controllers onto the trait and
registers a factory per seat (`HostedWorld::hosting`), which the server calls at start and at every
release. **An asynchronous controller — a language model — does not use this seam: it connects as a
session**, over the protocol, so the world never waits for it (`ARCHITECTURE.md` §9).

**Options considered** (step-12 §7.5).

```text
(a) own consult lattice on the world thread, ARC-27's formula   chosen: exact world instants, no
                                                                 dropped frames, no new concept
(b) one tokio task per controller (the old agent.rs)             decides on whichever frame arrived;
                                                                 cannot name its instants. Kept only as
                                                                 the shape of a future asynchronous
                                                                 controller, which is a session
(c) the kernel's scheduler, a controller as a Process            a controller would become world state
                                                                 (INV-1; ARC-27 option (c))
(d) job schedulers (tokio-cron-scheduler, apalis)                wall-clock cron and queues: the wrong
                                                                 clock and the wrong concept
```

**Why not theirs.** Nothing external schedules on a world's own clock against a seat binding the world
thread owns; the lattice is the formula `mineworld run` already uses, a few lines here.

**Measured, not assumed.** `decide` runs on the world thread, so its cost is the world's. The server
counts its ticks in a fixed histogram and keeps the longest, and prints them on a graceful stop
(`[world] ticks N, p50 A ms, p99 B ms, longest tick M ms`); the composition root prints each hosted
seat's consults and outcomes beside it. S11-B's CP-B4 bounds the **p99** tick of a hosted
`market-town --town --save` at 50 ms, half the observation cadence, and reports the maximum beside it
(operator ruling on D-SB12, 2026-10-08: one off-CPU stall the server cannot control — descheduling,
or the commit's I/O wait — must not decide the bound; step-12 E-SB9 located every outlier there).

**Limitations accepted.** A hosted world is not reproducible from a seed — its instants follow the wall
clock (`ARC-27` excludes it from `AC-12`); its save still replays byte for byte, because the journal holds
the requests. The paced controller's rates are tuned to a 900-second pace; at the hosted default of 5 s
it is chattier than a person (QS11-4), which is cognition's to tune (S10), not the server's.

**Note, 2026-10-09 (S15, PR 12n-2; `step-11-bodies.md` §21 SD-N14; `ARC-75` item 7).** A hosted paced
seat steps its walks at wall cadence. `PacedSeat` keeps its lattice `genesis + (k + m·pace)·scale`
for `decide`. After a consult whose observation disclosed the seat's own `walking` record, or whose
request was `walk-to` or `walk-step`, it adds **step consults** every `EMBODIED_STEP × scale` world
seconds, offset `k·scale` (QTW-13's form), each answered by the controller's pure `step`. It stops at
the first step consult that returns nothing. `EMBODIED_STEP` is one wall second, a constant of this
adapter: one `walk-step` of 1 340 mm per wall second is 1.34 m/s on screen at any time scale
(Weidmann 1993). No pack and no controller sees the scale. Idle seats get no step consults. A
consult is the lattice consult when a lattice instant lies between the seat's previous consult and
this one, because the runtime builds the observation at `due.max(world.now())`. There it is `decide`,
and `step` when `decide` returns nothing for a walker. The server's seam is unchanged. The reactive
seat does not walk.

**Note, 2026-10-15 (F-tick-1; operator ruling amending D-SB12, `step-12-server.md` §16.11).** The
wall-clock p99 bound failed on shared CI runners while the server's own work was unchanged: F-12n-CI1's
57.3 and 62.8 ms, PR #206's 70.7 ms and PR #198's 189.6 ms, every one with a wall p50 of 0.3–0.6 ms and
each passing on re-run. E-SB9 had already located every slow tick off-CPU (wall ≫ CPU). The p99 of
1 200 wall ticks is the twelfth-longest, so twelve preemptions decide it. The measured statement is
therefore split by what each clock can see:

- The server also records the world thread's **own CPU time** per tick, in a second fixed histogram
  (`DEP-59`), and the statistics line becomes `[world] ticks N, p50 A ms, p99 B ms, longest tick M ms,
  cpu p50 C ms, cpu p99 D ms`. The first three duration columns are wall time, as before.
- CP-B4 (and CA-13, which reuses its bound) asserts **CPU p99 ≤ 50 ms**, the tail of the work a tick
  does, which runner load cannot move, and **wall p50 ≤ 50 ms**, the typical tick as a client feels it,
  which catches a tick that waits without computing (a sleep, a blocking call). The wall p99 and the
  maximum are printed beside them and do not gate.
- **Accepted gap.** An off-CPU wait in fewer than half the ticks (the journal's fsync per hosted request,
  `Durability::PowerLoss`, on a slow disk) is visible only in the advisory columns. E-SB9 measured a
  commit at ≤ 5.1 ms even under eight fsync-heavy neighbouring servers, so the bound gives up nothing
  measured; a durability regression is R-S11-5's to watch, not this bound's.
- If the thread's CPU clock cannot be read, the tick's CPU time is recorded as its wall time, an upper
  bound: a missing clock can only make the bound harder to meet.

---

## ARC-43 — Facts reach observers through one audience rule, judged when they are recorded

**Date** 2026-10-08 · **Status** accepted; implemented in S11 PR S11-C · **Approved by** the primary
session at S11-C's design freeze (QS11C-1 … QS11C-5, QS11C-7 … QS11C-9) and the operator (QS11C-6) ·
**Relates to** `INV-13`, `ARC-28`, `ARC-29`, `ARC-40`, `ARC-41`, `ARC-42`, coordination rulings 1, 2 and
4 of `.structured-coding/plans/mvp0/overall.md` "Parallel build-out, 2026-10-08", and "The World
Interaction List" QIL-8 · **Design** `.structured-coding/plans/mvp0/step-12-server.md` §§4.7, 17 ·
**Specification** [`server/PROTOCOL.md`](../server/PROTOCOL.md) §§5.2, 5.8; [`MODULE_SPEC.md`](MODULE_SPEC.md)
§8.1 (`mineworld perceived`)

**Problem.** A fact states who *could* have learned of it — its `Visibility` — and nothing yet decides
who *did*. Three consumers need that answer: a client's observation frame (`observation.events`), a
cognition process that must not miss a fact and must resume after a dropped socket (S10's R-S11-1 …
R-S11-3), and an offline reader building a Person's memory from a save (S10's IC-1). If each answered it
for itself, a resumed stream, a live one and an offline export would disagree, and nobody could say
which was right.

**Choice.**

1. **One rule, owned by the perception system.** `systems/presence/src/audience.rs`:
   `Whereabouts` (which place each person is in — a fold of presence's own `arrived`, seeded from its
   `Presence` components or from a world's first fact), `admits` and `perceived_by`. `SystemInternal`
   reaches nobody; `Public` everybody; `Participants` exactly the fact's participants; `Entities(S)`
   exactly `S`; `Place(p)` the fact's participants and subjects, and everybody whose whereabouts *after
   the fact is applied* is `p`. Presence owns it because where people are is presence's state and
   perception is presence's job (`INV-13`); the server names no pack (it asks an `EventPerception` seam
   the composition root fills).
2. **Judged at record time, fact by fact.** As each fact is recorded the fold advances past it and every
   connection's observer is asked about it then. A `Place` fact is heard by whoever was there when it
   happened, not by whoever is there at the next 100 ms sweep; a person's own arrival is heard by them; a
   line said just before somebody leaves reaches them.
3. **Delivered three ways from that one function.** `observation.events`, best effort and bounded,
   drops counted in `events_dropped`; the `perceived` stream, reliable, ordered before the observations
   it explains, cursor-resumable, backfilled for a persisted world from its save by `perceived_by`; and
   `mineworld perceived`, the same `perceived_by` over a save, offline. Live, resumed and offline agree
   by construction, and the acceptance tests check that they do.
4. **The audience narrows at emission, not here.** A World Interaction List rule that makes a fact
   quieter states a narrower `Visibility` when the fact is built (QIL-8); this function reads only the
   envelope and needs no change for it. A hearing range is a later refinement behind the same seam; for
   MVP-0 overhearing is place-level (the operator's ruling on QS11C-6): a player on market-town's street
   hears every line said anywhere on the street.

**Supersedes.** S10's plan (`step-17-cognition.md` §3.3.4) for `Observation.events` to stay empty, with
perceived facts only in their own frame, and its check IC-9 "transcripts byte-identical": coordination
ruling 2 delivers both from the one function, so IC-9 reduces to "the 300-day digests are unchanged" —
`mineworld run` calls `observe`, which this decision does not touch. S10's text is amended by its owner
at its next pull request (QS11C-1).

**Rejected.**
- *Judging at sweep time* against the world as it stands: answers where people are now, not where they
  were when the fact happened; the "said before they left" case fails.
- *A broadcast channel of every fact, filtered per connection*: the filter needs the whereabouts at
  record time, which only the world thread has; and a lagging receiver on a broadcast channel loses
  facts silently.
- *An ephemeral world keeping a re-foldable window of recent facts* so it can serve old cursors: a
  second log. An unsaved world serves the stream from the join on and answers older cursors
  `cursor_unavailable` (QS11C-2).
- *In-server controllers receiving facts* now: none reads them (the paced controller reads histories,
  the reactive one its conversation history), and a queue nobody drains is a cost with no consumer
  (QS11C-4). The seam is there when one does.

**Limitations accepted.** A resume of a persisted world folds its whole fact log off the world thread (a
300-day market town is about 373 000 facts); seeding the fold from a snapshot is the recorded
optimization, taken when measured necessary (QS11C-8). A payload that is not JSON reaches the wire as
`null` (`PROTOCOL.md` §5.2); `EventEnvelope::map_payload` is a proposed later contract change, not taken
here (QS11C-3).

---

## DEP-15 — Observation deltas: the typed `ObservationDelta`, chosen by measurement

**Date** 2026-10-09 · **Status** selected; shipped in S11 PR S11-C · **Approved by** the primary session
at S11-C's design freeze (SD-C10's rule and QS11C-5, frozen before measuring) · **Relates to** `ARC-41`,
`ARC-23`, `ARC-43`, [`REUSE_POLICY.md`](REUSE_POLICY.md) §15 · **Design**
`.structured-coding/plans/mvp0/step-12-server.md` §§4.8, 7.3, 17 (CP-C1, E-SC8) · **Specification**
[`server/PROTOCOL.md`](../server/PROTOCOL.md) §5.3

**Problem.** A hosted world sends every connected client a whole observation ten times a second. On a
hosted market town that is about 87 KB per client per second, almost all of it unchanged from the
frame before: the people in the room, their components, the list of what may be attempted.

**Options considered** (both directions of the reuse question).

```text
(a) the typed ObservationDelta (ours, server/src/protocol/delta.rs)   chosen, by the frozen rule
    keyed by contract identity: entities upserted / removed by EntityId,
    relations and affordances replaced whole when changed, events always
(b) RFC 6902 JSON Patch (the json-patch crate 4.2.0, MIT/Apache-2.0)  measured; 13 % smaller than (a)
    a generic diff of the observation's JSON                          on this world; no GDScript
                                                                      applier exists
(c) whole observations only                                           the conforming fallback
(d) transport compression, permessage-deflate                         a dead end: Godot's
                                                                      WebSocketPeer cannot negotiate
                                                                      it (godot#103230, ARC-41)
(e) a binary encoding (MessagePack)                                   a different question: ARC-41
                                                                      keeps JSON; bytes are answered
                                                                      by deltas, not by encoding
```

**Measurement** (step-12 E-SC8; `tools/cli/tests/deltas.rs`, run explicitly). A hosted
`worlds/market-town --town`, four sessions (visitor, wanderer, and Alice and Bob taken over from their
in-server controllers), 60 wall seconds, 601 whole frames per client; on the same frames, per client
per second:

```text
client     whole       typed            json-patch
visitor    81 993 B/s  1 141 B/s        993 B/s
wanderer   93 733 B/s  1 161 B/s      1 012 B/s
alice      90 487 B/s  1 155 B/s      1 007 B/s
bob        81 552 B/s  1 140 B/s        992 B/s
mean       86 941 B/s  1 149 B/s (1.3 %)  1 001 B/s (1.2 %)
```

Every one of the 2 400 consecutive pairs reconstructs exactly: `apply(previous, diff(previous, next))`
equals `next` with its entities in id order (CA-9).

**Choice.** The rule frozen before measuring (step-12 SD-C10) reads: *typed* if it is at most half the
whole bytes and json-patch is not within 10 % of it; *json-patch* if within 10 % of typed and at most
half the whole bytes; *whole observations only* otherwise. Typed is 1.3 % of whole, and json-patch is
not within 10 % of it (it is 12.9 % smaller), so the rule selects **typed**, and it ships: `delta`
frames between keyframes (`PROTOCOL.md` §5.3), the first frame, every `--keyframe-every`-th frame
(default 50) and the first after a resume or a backfill being whole.

**Disagreement recorded.** step-12 §4.8, written before the rule, said the typed delta is kept "only
if it beats both". On these numbers it beats whole observations by a factor of 75 and loses to
json-patch by 148 B/s per client; read that way, neither non-whole branch of the frozen rule applies.
The freeze is the binding text and its rule was applied as written; the operator is told of the
tension in S11-C's handoff. What the rule's outcome keeps: an applier a client writes in a few lines
against `PROTOCOL.md` §5.3's table, with no RFC 6902 implementation in GDScript to adopt or write, and
deltas keyed by contract identity rather than by JSON paths into a list. What it gives up: about
150 B/s per client against json-patch, on a stream already 75 times smaller.

**Isolating interface.** `server/src/protocol/delta.rs` (`ObservationDelta`, `diff`, `apply`,
`canonical`); the Godot module's `mineworld/delta.gd`. `json-patch` is a dev-dependency of
`mineworld-cli` only, for this measurement, and is in no shipped artefact.

**Limitations accepted.** Measured on one town and one cadence; a world whose observations churn
differently may weigh the encodings differently, and the measurement is re-runnable as written. A
`delta` whose `base` is not the frame a client holds cannot occur on one WebSocket; a client that sees
one resumes, which yields a whole observation.

---

## ARC-44 — The admin surface is HTTP behind a bearer token, mounted only when configured, and changes no world state

**Date** 2026-10-08 · **Status** accepted; implemented in S11 PR S11-D · **Approved by** the primary
session at S11-D's design freeze (step-12 §18, QS11D-1 … QS11D-8; QS11D-3 by the operator) · **Relates
to** `ARC-40`, `ARC-41`, `DEP-14`, `INV-9`, [`NETWORKING.md`](NETWORKING.md) §§3, 5, 9 · **Design**
`.structured-coding/plans/mvp0/step-12-server.md` §§4.9, 7.6, 18;
[`step-19-time-weather.md`](../.structured-coding/plans/mvp0/step-19-time-weather.md) §§7.2–7.4, 15.3 ·
**Specification** [`server/PROTOCOL.md`](../server/PROTOCOL.md) §§5.9, 11

**Problem.** An operator could see a hosted world only through `/status`: not who is connected, not who
drives which seat, and not how to remove a misbehaving player or pause the town. S19 needs a host-only
pause, and a `clock` announcement so clients re-anchor their time display. None of this may give a
client a new way to change the world, and none of it may make the invite a host credential.

**Choice.**
- **HTTP on the control plane, never a socket frame.** `server/src/admin.rs` serves `GET
  /admin/sessions`, `GET /admin/seats`, `POST /admin/sessions/{session}/kick`, `POST
  /admin/seats/{seat}/release`, `GET /admin/clock` and `POST /admin/clock` (`{ "paused": bool }`), each
  body typed and denying unknown fields. The client vocabulary stays `join`, `submit`, `leave` (`INV-9`):
  a socket frame `pause`, `clock`, `kick` or `release` is `unknown_frame`.
- **Mounted only when configured.** With `--admin-token` / `MINEWORLD_ADMIN_TOKEN` the routes exist;
  without it every `/admin` path is `404`. One bearer token, compared in constant time with `DEP-14`'s
  `subtle`; every failure answers `401` no sooner than 500 ms after the request, in that request's task.
  The token follows the invite's rules, is never printed, is in no save, frame or `/status`, and must
  differ from the invite.
- **No world state.** Kick, release, pause and resume are host state (`ARC-40`): never journaled, no fact,
  no revision. A kick or release returns the seat to its default with no hold, and the displaced
  connection is told `closing { kicked }`. **Pause is host pacing**: the host clock is a segment that
  stops where it stands and continues from there on resume (monotonic and jump-free); while paused the
  world is not advanced, hosted controllers are not consulted and every `submit` is refused `paused`
  before an `ActionId` is allocated; observers stay connected. This is S19's clarification of I-4: a
  route may change the pacing of the clock, never its value — **no route and no field names an
  instant**.
- **Telling clients.** `WorldSummary.paused`, and a `clock { at, time_scale, paused }` frame after
  `welcome` and on every change, carried by a `tokio::sync::watch` channel the world thread writes —
  newest wins, never dropped, never waited on.
- **The live scale change is TW-c's.** `POST /admin/clock` carrying `time_scale` answers `409
  time_scale_fixed` until TW-c lands it with the hosted-cadence rescheduling and the host journal it
  needs. S19's record of pause and scale as host commands (its placeholder ARC-69) remains TW-c's.

**Options considered.** (a) HTTP routes on the existing axum router — chosen: the control plane
`NETWORKING.md` §3 names, testable with any HTTP client, no new dependency. (b) Admin frames on the
WebSocket — rejected: it widens the client vocabulary and puts host commands one field away from player
commands. (c) A separate gRPC admin service (`tonic`) — rejected: a second protocol stack and listener for
six routes. (d) `axum-extra`'s `TypedHeader` for the bearer header — rejected: two crates to read one
header, which is about fifteen lines with `HeaderMap`.

**Limitations accepted (MVP-0).**
- **One admin token, one level of host trust.** Whoever holds it is the host (the operator, or a
  single-player launcher that generated one for its own client).
- **No ban list** (QS11D-3, operator): a kicked player may rejoin at once with the invite, because there
  is no per-player identity. To remove someone for good, rotate the invite (restart with a new
  `--invite`).
- **A pause is not persisted** until TW-c's host journal: a server restarted while paused runs. Holds are
  wall time and still end during a pause.
- **Unbounded parallel wrong-token requests** each wait 500 ms but are not counted; rate limiting is the
  adopt route for public hosting (`DEP-14`).

---

## ARC-53 — A pack's identity is stated once, where the pack already states who it is

**Date** 2026-10-08 · **Approved by** the primary session at PR E-a's design freeze (step-16 §14.0;
QSE-4, QSE-5, QSE-6 and FQ-1 … FQ-5 as ruled) · **Implements** [`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md)
§5 (MVP-0 subset), [`MODULE_SPEC.md`](MODULE_SPEC.md) §3.1, §4.1, §6.1, §8.1, §9 · **Relates to**
`ARC-3`, `ARC-33`, `ARC-35`, `DEP-12`, `DEP-21` · **Design**
`.structured-coding/plans/mvp0/step-16-packages.md` §4.1, §14 (S16, PR E-a)

**Problem.** Milestone E needs packs that can be told apart, versioned and checked: "independently
installable" means nothing for a pack that has no identity. Until this decision no pack had one. Every
crate was version `0.0.0` (`[workspace.package]`); a World Pack had an id and a name; a Presentation
Pack had only its style's id; nothing stated a licence per pack. `PACKAGE_FORMAT.md` §5 specified a
manifest, but three carriers already state who a pack is, and a second manifest beside each would
state every fact twice.

**Choice.**

1. **One vocabulary.** Every pack has these package fields, the MVP-0 subset of `PACKAGE_FORMAT.md`
   §5 and `MODULE_SPEC.md` §9:

   ```text
   id          the pack's id: 1–64 characters of a–z, 0–9 and '-', beginning with a letter, no '--',
               not ending in '-'. A code pack's id is its Cargo package name
   version     semver, MAJOR.MINOR.PATCH
   type        system-pack | controller-pack | world-pack | presentation-pack | entity-pack
               (asset-pack refused by name in MVP-0)
   mineworld   the framework versions the pack works with, a semver range (data packs)
   license     an SPDX licence expression
   provenance  authors, and the repository the pack comes from where stated
   ```

2. **Three carriers, so no fact is stated twice.**
   - **A code pack** (System Pack, Controller Pack) is identified by its `Cargo.toml` `[package]`.
     The crate `mineworld-packages` (`packages/`) defines `Package`, a record of `&'static str`, and
     the expression macro `package!()`, which expands to `Package::declared(env!("CARGO_PKG_NAME"),
     env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_LICENSE"), env!("CARGO_PKG_AUTHORS"),
     env!("CARGO_PKG_REPOSITORY"))`. `env!` is expanded where the macro is invoked, so each pack
     records its own crate's fields at compile time; the binary never runs Cargo and never reads a
     source tree. `mineworld-sdk` re-exports both. **`SystemPack::PACKAGE` is a required constant**:
     a System Pack writes, as the first line of its `impl SystemPack`,

     ```rust
     const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
     ```

     and a pack without it does not compile. The installed set's `Capability` gains `package()` and
     `version()`. The rule controller writes `pub const PACKAGE: mineworld_packages::Package =
     mineworld_packages::package!();` and depends on `mineworld-packages` directly, not on the SDK:
     the SDK is how a *System* Pack declares itself, a controller is not one, and its manifest names
     no kernel, transport or server directly (it reaches the kernel only through the packs whose
     vocabulary it uses).
   - **A World Pack** is identified by its `world.yaml`: `world.id` (unchanged: its directory's name),
     `world.version`, `world.license`, and the top-level `mineworld:` range.
   - **A Presentation Pack** is identified by a `pack.yaml` at its root, beside the unchanged style
     manifest (`manifest.yaml`, `ART_DIRECTION.md` §12, whose own `id` stays the style's id). Its `id`
     is stated in `pack.yaml` and its directory name is free: `ARC-3`'s family layout puts dimension
     packs in `2D/` and `3D/`, which are not ids. Entity Packs use the same file from S16's PR E-d.

3. **The framework has a release version: 0.1.0.** Every framework crate and every bundled pack share
   it (`[workspace.package] version`), with pre-1.0 semantics: a MINOR bump may break. A `mineworld:`
   range is checked against `mineworld_packages::FRAMEWORK_VERSION`, the version the running binary
   was built with.
4. **`SystemVersion` and the release version stay two things.** `SystemVersion` is the contract
   counter a save is checked against (`ARC-25`); the semver version is what a range is checked against.
   Raising a pack's `SystemVersion`, which makes existing saves refuse, is a breaking release: a MINOR
   bump before 1.0 and a MAJOR bump from 1.0, so a range like `^0.1` never admits a pack whose saves
   are incompatible.
5. **Validated once, at run time, by one crate.** `Package` is a plain record. `mineworld-packages`
   turns it, a `pack.yaml` and a world's fields into one validated `Identity` through the same types:
   `Version` and `Compatibility` (the `semver` crate) and `License` (the `spdx` crate, `DEP-21`). A code
   pack with an empty or unparseable licence compiles, and `mineworld packs list` refuses it, naming
   the pack and the text.
6. **Refused by name, never accepted unchecked.** A World Pack's three fields are optional to the
   loader in MVP-0, so existing test worlds and `mineworld create`'s template read unchanged, but when
   present each is typed and checked: a malformed version or licence is refused at its line and
   column, and a `mineworld:` range the framework does not satisfy is refused naming the range and the
   framework version. `mineworld packs validate` requires all three. A `pack.yaml` refuses an unknown
   field (`dependencies:` included, until requirements are resolved), a missing field, a malformed
   value, and a `type` it does not carry, naming the file where that type is identified instead.
7. **Identity never enters a fact or a save** (step-16 QSE-14). The `PACKAGE` constants are read by
   `mineworld packs` only. A save's composition is still checked by `SystemVersion` alone.
8. **`mineworld packs list | show | validate`** (`MODULE_SPEC.md` §8.1) prints identities: the build's
   code packs, and the data packs in each directory named by `--packs DIR`, which is given explicitly
   and never implied.

`ARC-33` point 1's list of the SDK's dependencies gains `mineworld-packages`, a leaf that depends on no
MineWorld crate; the SDK still never depends on a pack.

**Not in this decision.** A world's `requires:` and its resolution, pack roots on other commands, the
licence allow-list, bundled-versus-third-party classification, Entity Packs and packs from outside
this repository: later PRs of S16, each recorded when it lands.

**Accepted limitations.**
- A World Pack's fields are optional to the loader until requirements are resolved; only `packs
  validate` requires them.
- A World Pack has no authors field, so its provenance is its directory.
- The framework repository is not stated in the workspace's `[package]` fields, so a bundled code
  pack shows no repository.

---

## DEP-21 — Versions and licence expressions: `semver` and `spdx`

**Date** 2026-10-08 · **Status** selected; dependencies added in S16 PR E-a · **Approved by** the
operator's reuse table (step-16 §5 rows 12–13) and the primary session at PR E-a's freeze ·
**Relates to** `ARC-53`, `DEP-10` · **Design** `.structured-coding/plans/mvp0/step-16-packages.md` §5,
§14 (S16, PR E-a)

**Problem.** `ARC-53` checks semver versions, semver ranges and SPDX licence expressions, and a data
pack's range must mean what a code pack's Cargo range means.

**Options considered** (`REUSE_POLICY.md` §§11–12, §17 — both directions):

```text
versions   (a) `semver` (dtolnay; Cargo's own semantics)  (b) `node-semver` (npm's)  (c) our own parser
licences   (d) `spdx` (Embark; the parser cargo-deny uses)  (e) a closed list of exact identifier
           strings  (f) our own expression parser
```

**Choice: (a) and (d).**
- **(a)** gives Cargo's meaning of `^0.1`, so a range in a `pack.yaml` admits exactly the versions the
  same range admits in a `Cargo.toml`. **(b)** differs from Cargo for pre-1.0 carets, a semantic
  mismatch. **(c)** would be a third dialect of a solved problem.
- **(d)** parses expressions (`MIT OR Apache-2.0`) against the SPDX licence list. **(e)** cannot read
  an expression, which is what Cargo's `license` field holds. **(f)** is the wheel (d) already is.

**Isolating interface.** `mineworld-packages`' `Version`, `Compatibility` and `License`. No other
crate names `semver` or `spdx`.

**Weight, measured** (step-16 §14.8 E-Ea2). `semver` 1.0.28 (MIT OR Apache-2.0) has no dependency;
`spdx` 0.13.6 (Apache-2.0), with its default features (none), depends only on `smallvec`, already in
the build. `Cargo.lock` gains three packages: the two and `mineworld-packages`. No build script.

**Accepted limitations and the revisit trigger.** A registry (the publishing sense of Milestone E,
non-goal) brings version selection, which is a solver's problem and is not solved here; revisit then.

---

## ARC-61 — A System Pack may be configured per world, by a file its owner types

**Date** 2026-10-08 · **Approved by** the operator (`overall.md` "The World Interaction List": the
carrier stays `configure:`; QPL-2, QPL-12) and the primary session at PR IL-a's design freeze
(step-18-interaction-list §11; QIA-1 … QIA-6 accepted as recommended) · **Implements**
[`MODULE_SPEC.md`](MODULE_SPEC.md) §4.1, §9 (configuration schema) · **Relates to** `ARC-15`,
`ARC-25`, `ARC-26`, `ARC-31`, `ARC-33`, `ARC-62`, `DEP-10`, `INV-13` · **Design**
`.structured-coding/plans/mvp0/step-18-interaction-list.md` §4.3, §4.10, §11 (S17, PR IL-a)

**Problem.** A System Pack's numbers and policies (a range, a capacity, a step) are compiled
constants. A world that wants another value has nowhere to say so: `world.yaml` names packs but carries
nothing for them, and a section (`ARC-31`) belongs to one person, place, item or organization, not to
the world. The S17 Interaction List (`ARC-63` onward) needs each pack to accept a world-level document
it types and enforces itself. Three constraints bound the answer: the loader must not learn what a
configuration means (`ARC-31`'s rule, one level up); a world that configures nothing must load exactly
as before, fact for fact; and a save must never resume against a configuration other than the one it
was created with, because resume does not re-read content (`ARC-25`).

**Choice.**

1. **The contract is `mineworld_authoring::PackConfiguration`**, beside `AuthoredSection`:

   ```text
   trait PackConfiguration: SystemIdentity
     type Configuration: DeserializeOwned + Debug + Send + Sync + 'static   deserializing is validating
     const FACTS: &'static [EventTypeId]       the event types its configuration may seed
     fn references(&Configuration) -> Vec<Reference>    entity keys it names (default none)
     fn requires(&Configuration) -> Vec<SystemId>        systems that must also be enabled (default none)
     fn seed(&Seeding, &Configuration) -> Result<Vec<Emission>, Rejection>
   ```

   A configuration has no subject, so `seed` takes none. The loader holds a decoded configuration type
   erased, as `AuthoredConfiguration` (owner, references, requires, seed), produced by the
   `DecodeConfiguration<T>` `DeserializeSeed` — exactly as `AuthoredContent` and `Decode` hold a
   section.
2. **A pack says it is configurable in its `impl SystemPack`**, with `mineworld_sdk::configures!();`,
   which defines `CONFIGURATION` (`Some(<its own id>)`), `CONFIGURATION_FACTS` (its `FACTS`) and
   `decode_configuration` together from the `PackConfiguration` impl. The defaults are the safe
   direction: `None`, no facts, and a decode that refuses "the '<id>' system takes no configuration".
   The installed set's `Capability` aggregates them: `configuration`, `decode_configuration`,
   `configuration_facts`.
3. **The carrier is `world.yaml` `configure:`**, an optional list of keys. Each key is an enabled
   System Pack's id or a reserved key (item 4), and names `configure/<key>.yaml`, decoded straight from the YAML stream into the owner's type, so a
   refusal keeps its line and column (`DEP-10`). The list's order is the author's and is the seeding
   order. Absent means empty.
4. **Reserved keys.** `classes` (the Interaction List's entity classes, `ARC-64`, IL-b) and `packages`
   (the licence-policy override hook of S16 E-b, FQ-b2 — owned by the framework crate
   `mineworld-packages`, not by a System Pack) are reserved: listing either is refused, "reserved for
   <what>; not configurable in this build". Both are wired in IL-b, which lets `packages` through to its
   framework owner rather than resolving it against the installed set. A test holds that no installed
   pack's id is a reserved key.
5. **Refused by name**, each naming the key and the file, with line and column where a YAML value is
   involved: a key that is no system of this build; a system the world does not enable; a system that
   takes no configuration; a reserved key; a key listed twice; a listed file missing; a `.yaml` file in
   `configure/` that is not listed; a `requires` system not enabled; a reference to an undeclared
   entity or one of another type; a configuration the owner's `seed` refuses; a seeded fact in another
   pack's vocabulary or of an event type outside the owner's `FACTS`; and configuration drift (item 7).
   Malformed YAML is the loader's existing `Malformed`.
6. **Seeding order.** Genesis states passages, then locations, then **configuration, in `configure:`
   order**, then sections. A section's reduction may therefore check its value against the configured
   state (the step a value must be a multiple of, the range it must lie within); it is reduced after it.
   A world without `configure:` takes no new branch and seeds exactly what it seeded before this
   decision: its ids, facts and digests do not move.
7. **Drift is refused at resume** (QPL-12). Every host that resumes or verifies a save from a World Pack
   — `mineworld run` resuming, `mineworld server --save`, `mineworld replay` — first calls
   `WorldPack::check_configuration(saved genesis facts)`. It assembles the world, seeds the configuration
   as genesis would, and compares, in order, the event type, the record and the visibility of every
   genesis fact whose type is in the union of the enabled packs' `configuration_facts`, on both sides.
   The first difference is `PackError::ConfigurationDrift { system, saved, here }`. Configuration added
   and configuration removed are both drift. Nothing in `persistence/` or the kernel changes: the
   comparator reads the save's genesis facts through the existing backend.
8. **The configuration facts' audience is guidance, not a loader rule** (QIA-4). An owner states its
   configuration facts `Visibility::SystemInternal` with no subjects: a world's configuration is
   nobody's perception and nobody's biography (`INV-13`). The loader does not enforce it, because a
   loader rule over visibility would be the loader judging a pack's vocabulary.

**Not in this decision.** What any pack's configuration says: the Interaction List's schema, classes
and sections (`ARC-63` … `ARC-65`, IL-b onward). Wiring `configure/packages.yaml` into the licence
policy (IL-b). Drift in content other than configuration — a renamed person, a moved table — which is
not checked (QPL-12's scope).

**Accepted limitations.**
- Configuration is not a rule. `MODULE_SPEC.md` §4 constraint 3 holds: a configuration parameterizes and
  restricts what its owning pack implements; it never adds behaviour no installed pack has.
- Resume still does not re-read content, so content drift outside configuration is still accepted
  silently, as before this decision.
- A configured pack's declaration does not change, so configuring a pack does not change its
  `SystemVersion`; a pack whose configuration *schema* changes raises it, as for any owned type.

**Note, 2026-10-08 (S17, PR IL-b; QIA-1, QTW-7, SD-IB-1, SD-IB-3, SD-IB-5) — framework keys, the
seeding context and `data:` attachments.**
- **Item 4's reserved keys become framework keys.** `classes` is decoded by
  `mineworld_authoring::EntityClasses` (`ARC-64`) and `packages` by `mineworld_packages::LicencePolicy`
  (`ARC-55` note). A framework key is never resolved against the installed set and never seeded on its
  own; listing it is optional, an unlisted file in `configure/` is still refused, and a test holds that
  no installed pack's id is a framework key.
- **The seeding context.** `PackConfiguration::seed(&Seeding, &Configuration, &ConfigurationContext)`:
  the context hands a configuration the world's entity classes and the bytes of its own attachments,
  read-only. `Seeding`, which sections share, is unchanged. A configuration may also be checked against
  the context before anything is seeded (`PackConfiguration::check`, default accept); a refusal there is
  typed (`ConfigurationRefusal`: an undefined class, two ambiguous entries) and named by the loader with
  the file, the list and the index. No shim keeps the old signature.
- **`data:` attachments.** A configuration may name files under the World Pack's `data/` directory with
  `authoring::Attachment`: a relative path written with `/`, first component `data`, every component a
  plain name (no `..`, no root, no drive, no `\`), checked as it decodes, at its line and column — the
  same on every platform. `PackConfiguration::attachments` lists them (default none). The loader reads
  each one and refuses one that is missing, one that resolves outside the pack after its links are
  followed, and one over 4 MiB, each by name. `seed` receives the bytes; the owner decodes them and
  states what it needs in its own fact, so a changed file is drift with no new mechanism. Files under
  `data/` that nothing names are allowed.

---

## ARC-62 — Extension catalogs: a pack-owned trait, implemented by other packs, listed in the installed set

**Date** 2026-10-08 · **Approved by** the operator (QPL-10: presence's `resolution:` line migrates to the
generic form, with no shim) and the primary session at PR IL-a's design freeze (step-18-interaction-list
§11) · **Implements** [`MODULE_SPEC.md`](MODULE_SPEC.md) §3.1 · **Relates to** `ARC-33`, `ARC-39`,
`DEP-12`, `ARC-61` · **Design** `.structured-coding/plans/mvp0/step-18-interaction-list.md` §11;
`step-18-physics-list.md` §4.7 (S17, PR IL-a)

**Problem.** `ARC-39` let other packs plug code into presence through `ArrivalResolver`, and spelled the
build's list as a hard-wired `resolution:` line of `installed!`, expanded into `Capability::resolvers()`
and registered by a call to `mineworld_presence::register_resolvers` that `worldpack::compose` makes by
name. The S17 Interaction List needs a second such catalog (`bodies`' interaction kinds, IL-i), and any
pack may need one later. A second hard-wired line would edit the SDK and the loader again for every
catalog — the change amplification `CLAUDE.md` §4 rule 5 forbids. Two real catalogs earn the
abstraction (rule 11).

**Choice.**

1. **One generic line per catalog.** `installed!`'s grammar is `perception: <path>;`, then zero or
   more lines

   ```text
   extension <trait path> => <register fn path>: [ <type>, … ];
   ```

   then the pack lines. Each listed type must implement the trait and `Default`; one that does not is
   refused by the compiler, at the list.
2. **What it expands to.** `Capability::register_extensions()` calls each line's register function once,
   with one value of each listed type, `Box<dyn Trait>`, lines in listed order and types in listed
   order. `Capability::extension_types()` returns each line's trait path and its types' Rust paths, for
   the installed set's own guard. The `resolution:` arm and `Capability::resolvers()` are deleted, with
   no shim (`CLAUDE.md` §4 rule 12).
3. **The loader calls one function.** `worldpack::compose` calls `Capability::register_extensions()`;
   neither the SDK nor the loader names a trait or a pack again. A third catalog is a line in
   `systems/installed` and nothing else.
4. **The catalog stays its owner's.** The register function, its storage and its rules belong to the
   pack that owns the trait. `ARC-39` item 5's rules carry over to every catalog: write-once and
   process-wide, registered before any world is assembled, a different list for one trait refused
   naming both; an implementation is pure (reads only what it is handed), keeps nothing, and is inert
   where its pack's state is absent, so a world that does not enable the implementing pack is
   unaffected.
5. **The installed set guards its lines.** A test refuses a type on an extension line that is not an
   installed pack, and a type listed twice on one line, naming each.
6. **Presence's catalog is the first line**, byte-identical in behaviour:
   `extension mineworld_presence::ArrivalResolver => mineworld_presence::register_resolvers:
   [mineworld_bodies::BodiesSystem,];`. Installing a pack that implements a catalog's trait is `ARC-33`'s
   two lines plus one entry on that catalog's line.

**Accepted limitations.** A catalog is per process, not per world (`ARC-39`'s limitation, unchanged):
per-world applicability comes from the world enabling the implementing pack, or from its configuration
(`ARC-61`). Disabling a pack at run time while its implementation is registered is not supported
(QB-17).

**Note, 2026-10-13 (S24, PR FG-a; `ARC-86`, QFGA-3) — a third line, selected by configuration, and the
loader registers earlier.** `relationships` owns the trait `DecayCurve` and its register function
`register_curves`; `systems/installed` carries
`extension mineworld_relationships::DecayCurve => mineworld_relationships::register_curves: [];`, empty
today. Unlike the first two lines, a curve is not applied wherever its pack's state is present: a world
**selects** it by name in `configure/relationships.yaml`, and must enable the curve's pack in `systems:`
(the section requires it, refused by name otherwise), so the curve's identity and version are part of the
save's composition. A name colliding with a bundled curve (`none`, `linear`, `half-life`, `power`) or
listed twice panics naming both, as item 4's rules. Because a curve name is checked while the section
decodes, `WorldPack::read_with` now calls `Capability::register_extensions()` before it decodes
configuration, as `compose` already did (idempotent for the same list).

---

## DEP-17 — CI runs on GitHub Actions hosted Linux runners, inside the repository's own toolchain container

**Date** 2026-10-08 · **Status** selected; integrated in S13 PR 13a · **Approved by** the primary session
at 13a's design freeze (step-14 §9.0) under the operator's 2026-10-08 decision to make the repository
public · **Relates to** `ARC-48`, `DEP-18`, `ARC-30`, `ARC-35`, overall `D-12` · **Design**
`.structured-coding/plans/mvp0/step-14-ci.md` §§5.1–5.4, 6, 9

**Problem.** `ENGINEERING_STANDARDS.md` §15 says quality gates "must remain automatic", and §16 asks for
four CI layers. Until S13 the repository had no CI at all: every gate was a session running commands on
the operator's Mac. CI also has to be the other side of `AC-8`, which is native Linux x86_64.

**Options considered** (`REUSE_POLICY.md` §§11–12, §17, both directions).

```text
CI service
(a) GitHub Actions, hosted ubuntu-24.04           chosen
(b) GitLab CI through a push mirror               review on one platform, checks on another
(c) self-hosted Woodpecker or Forgejo Actions     a server MineWorld would have to operate
(d) CircleCI / Buildkite                          no fit advantage; a second account
(e) Actions on a self-hosted runner, the          arm64 macOS, so it cannot be AC-8's Linux x86_64
    operator's Mac                                side, and it would run PR code on the operator's
                                                  machine; kept only as a layer-4 fallback
(f) our own scripts and cron                      no PR integration

How a job gets the toolchain
(A) build the Dockerfile's `toolchain` stage, then `docker run` each layer      chosen
    with the checkout mounted
(B) `jobs.<id>.container:` on the official image, plus inline apt installs      the environment
    defined twice; actions/checkout silently falls back to a tarball with no .git when git is missing,
    which breaks AC-1's history scans
(C) the toolchain on the bare runner                                           the tests would not run
                                                                               in the container AC-8 names
(D) a prebuilt toolchain image on GHCR                                         needs packages: write,
                                                                               which the read-only workflow
                                                                               refuses

Test runner
plain `cargo test --workspace`      chosen: the project's canonical command, so CI and laptop agree
cargo-nextest                       declined for now (below)

Cache
actions/cache with an explicit key  chosen
Swatinem/rust-cache                 keys on the runner's rustc, not the container's, under (A)
sccache (GHA backend)               no gain on a link-heavy test build of ~117 binaries; revisit
```

**Choice.** GitHub Actions on hosted `ubuntu-24.04`. Each job builds the `toolchain` stage of the
repository's `Dockerfile` (`DEP-18`) with the BuildKit GitHub Actions cache, and runs one command inside
it: `python3 scripts/ci_layer.py <layer>`. The registry, git dependencies and compiled dependencies are
cached with `actions/cache` under a key of `Cargo.lock`, `rust-toolchain.toml` and the `Dockerfile`.
Third-party actions are pinned to full commit SHAs.

**Why not ourselves.** CI is commodity infrastructure (`REUSE_POLICY.md` §4). The repository, its PRs and
its review flow are already on GitHub (`D-12`). Checks reported there are the ones the operator sees
when reviewing.

**Why not the others.**
- (b), (d): no fit advantage, and a second platform or account to keep.
- (c): operational burden (`REUSE_POLICY.md` §2).
- (e): the wrong platform for AC-8, and an execution risk.
- (B): two definitions of the environment, and a checkout path that fails open.
- (C): would test outside the container that AC-8 names.
- (D): permission and storage cost. Public packages are free, so the storage reason lapses once the
  repository is public. The permission reason stands. Revisit if building the image per job is measured
  above about 2 minutes.
- **cargo-nextest.** It lists custom-harness targets by invoking them with libtest's
  `--list --format terse`. The project's two `harness = false` programs (`persistence/tests/kill_and_resume.rs`,
  `tests/acceptance/tests/arrival_resolvers_resume.rs`) do not implement that interface. Its retries would
  hide nondeterminism, which this project treats as a defect. Its gain is small, because six long tests
  dominate the suite. Revisit if `test` passes about 15 minutes cached, or needs sharding.

**Isolating interface.** `.github/workflows/ci.yml` only checks out, restores caches, builds the toolchain
image and calls `scripts/ci_layer.py`, the last three through the local composite action
`.github/actions/layer`. No check command appears in YAML (`ARC-48`, I-S13-9). Moving to another CI
service rewrites those two files. Changing what a layer runs edits one list in `scripts/ci_layer.py`, and
the same entry point runs locally.

**Accepted limitations.**
- **Cost.** As a private repository on GitHub Free, CI would draw on 2 000 included Linux minutes a month,
  while the project's PR rate needs several times that (step-14 §10.1). The repository became public on
  2026-10-08, so standard hosted runners are free. Runs are still kept purposeful.
- **Runner disk.** The default suite once wrote about 16 GB of scratch saves (step-14 F-3); since the
  test-hygiene PR (#77, `DEP-29`) each test removes its own, and `test` checks that nothing is left. The
  `test` layer prints free disk before and after. The public runner measured about 107 GB free before
  the tests, so no clean-up step is needed. A future shortfall is remedied in the workflow, never by changing tests in
  CI.
- **Building the image per job.** About 30–90 seconds per job. This is the price of one definition of the
  environment.

---

## DEP-18 — Container images: the official `rust` slim image to build, Debian slim to run, both pinned by digest

**Date** 2026-10-08 · **Status** selected; integrated in S13 PR 13a · **Approved by** the primary session
at 13a's design freeze (step-14 §9.0) · **Relates to** `DEP-17`, `ARC-48`,
[`ARCHITECTURE.md`](ARCHITECTURE.md) §§11, 13, [`NETWORKING.md`](NETWORKING.md) §§7–8,
[`MVP.md`](MVP.md) §9 `AC-8` · **Design** `.structured-coding/plans/mvp0/step-14-ci.md` §§3.1–3.2, 5.5

**Problem.** `NETWORKING.md` §7 deploys a world as a Docker container on a VPS, with a persistent volume.
`AC-8` requires the container to run the laptop's World Pack with no semantic difference. CI needs the
same environment for its layers (`DEP-17`). One `Dockerfile` serves all three.

**Options considered.**

```text
build base
(a) rust:1.97.1-slim-trixie (Docker Official Image)        chosen: the exact toolchain as a tag, rustup
                                                           inside, glibc, gcc for libsqlite3-sys
(b) rust:1.97.1-trixie (full)                              the same, with ~0.5 GB of unused packages
(c) musl static (rust:alpine, cargo-zigbuild) to scratch   a third target nobody else runs; musl's
                                                           allocator and libm untested against Rapier
runtime base
(d) debian:trixie-slim                                     chosen: the build stage's glibc; a shell, so
                                                           an operator can `docker exec … mineworld inspect`
(e) gcr.io/distroless/cc-debian13                          smaller, no shell; the planned hardening swap
                                                           once a hosted deployment exists (QS13-10)
(f) cgr.dev/chainguard/glibc-dynamic                       the free tier offers only `latest`, so a
                                                           pinned digest may be garbage-collected
(g) our own base image                                     nothing to gain
```

**Choice.** One `Dockerfile` at the repository root, with three stages.
- **`toolchain`** is `rust:1.97.1-slim-trixie` plus `git`, `python3`, `ca-certificates`, `rustfmt` and
  `clippy`. It copies no source; CI mounts the checkout into it.
- **`build`** copies the Cargo workspace and runs `cargo build --release --locked -p mineworld-cli`.
- **`runtime`** is `debian:trixie-slim` with the `mineworld` binary and `worlds/`.
  - It runs as the non-root user `mineworld`, keeps worlds on a volume at `/var/lib/mineworld`, and
    exposes 7878.
  - Its default command hosts `worlds/social-cafe` with a save on that volume.
  - It stops with `SIGINT`: the server's own stop signal (`tools/cli/src/main.rs`, `serve`), so `docker
    stop` shuts the world down the way an operator's Ctrl-C does.

Every `FROM` names its tag **and** its `@sha256:` digest. `scripts/check_ci_pins.py`, in CI's `fast`
layer, fails unless:
- the `rust:` tag equals `rust-toolchain.toml`'s channel and the root manifest's `rust-version`;
- every `FROM` carries a digest;
- the build and runtime stages name the same Debian release.

**Why not ourselves.** Base images are commodity infrastructure, maintained upstream with security
updates.

**Why not the others.**
- (b): size.
- (c): it would test a different target from the one deployed.
- (e): deferred, not declined. It is a one-line `FROM` swap.
- (f): reproducibility.

**Isolating interface.** The runtime image's interface is `mineworld` plus its ordinary arguments, a
volume at `/var/lib/mineworld`, and port 7878. MineWorld has no container-only flag, environment switch,
`cfg` or code path (I-S13-5). A compose file, a VPS unit or a Kubernetes manifest consumes the image
without changing it. The toolchain version has one source, `rust-toolchain.toml`; the image tag follows it
under `check_ci_pins.py`.

**Accepted limitations.**
- **Digests age.** A digest pins an image that stops receiving Debian security updates. Re-pinning is a
  deliberate, reviewed change. The pinned digests and their dates are in step-14's 13a ledger.
- **The runtime image's default command predates S11's invite token.** S11 adds the argument (QS13-15).
- **No published image.** 13a builds the image in CI and locally. A registry is a later decision.

---

## ARC-48 — CI layers, triggers, and what blocks a merge

**Date** 2026-10-08 · **Status** decided; layers 1–2 live from S13 PR 13a · **Approved by** the primary
session at 13a's design freeze (step-14 §9.0), under the operator's 2026-10-08 decision to make the
repository public · **Relates to** `DEP-17`, `DEP-18`, `ARC-30`, `ARC-35`, overall `D-12`,
[`ENGINEERING_STANDARDS.md`](ENGINEERING_STANDARDS.md) §§15–16 · **Design**
`.structured-coding/plans/mvp0/step-14-ci.md` §§3.4–3.7, 9.0

**Problem.** `ENGINEERING_STANDARDS.md` §16 names four CI layers: fast structural checks on every change;
core integration tests on every pull request; scenario tests; long-running stability tests that "can run
separately from the fastest PR loop". The project needs to know which layer runs when, what a red layer
stops, and what is mechanism rather than promise.

**Decision.** One workflow, `.github/workflows/ci.yml`. Its jobs name layers, never commands. The
layer → command table exists once, in `scripts/ci_layer.py`.

```text
job        layer (§16)            trigger                                         merge
fast       1 fast structural      push to every branch; pull_request               blocks
test       2 core integration     non-draft pull_request; push to main;           blocks
                                  push to scratch/**
image      (the runtime image)    workflow_dispatch; push to scratch/*-image      evidence only
scenario   3 scenario             push to main; nightly; workflow_dispatch (13b)  blocks main's health
stability  4 long-running         nightly; workflow_dispatch (13c)                reports
clients    Godot headless probes  nightly; workflow_dispatch (13c)                reports
```

- **`fast`** runs, in order:
  - `cargo fmt --all --check`;
  - `scripts/check_doc_headings.py`, `scripts/check_decision_ids.py`, `scripts/check_ci_pins.py` and
    `scripts/check_scratch.py scan`;
  - `cargo check --workspace --all-targets`;
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- **`test`** runs `cargo test --workspace`: the whole default suite. After it passes,
  `scripts/check_scratch.py left --target-dir target` fails the layer if any test left scratch behind
  (`ENGINEERING_STANDARDS.md` §22, `DEP-29`).
  - It builds first with `--no-run`, which runs no test and only separates build time from test time in
    the log.
  - That includes the two `harness = false` programs and the history-reading scans of `ARC-35`.
  - The default suite already holds the Social Café and Market Town scenarios and the long runs, by
    decision (step-10 QS-59).
  - `cargo test` cannot exclude a target without a hand-kept list, and such a list fails open. So layer 2
    is the whole suite until a fail-closed selector exists.
- **Full history.** Every job that runs a layer checks out with `fetch-depth: 0`, as a partial clone
  (`filter: blob:none`), of the whole tree. Tests read `presentation/` and `clients/protocol/`, so a
  sparse checkout that omitted them was tried and dropped (step-14 A-C3). A shallow clone makes
  `ac1_composability` fail by name, never skip.
- **Economy.**
  - `concurrency` is per ref, and cancels a stale run except on `main`.
  - A draft PR runs `fast` only.
  - Branch pushes run `fast` even when a PR also runs it, because minutes are free on a public
    repository.
  - `CARGO_INCREMENTAL=0` and `CARGO_PROFILE_DEV_DEBUG=line-tables-only` are CI-only settings. They change
    no codegen semantics: `ARC-30`'s `opt-level`, debug assertions and overflow checks are untouched.
- **Never weaker than local.**
  - A blocking job has no `continue-on-error`, no retry and no `|| true`.
  - No test reads a CI variable to relax itself.
  - The workflow runs with `permissions: contents: read`, uses `pull_request` and never
    `pull_request_target`, and needs no secret.
- **Check names are an interface.** The jobs are named exactly `fast` and `test`, with no matrix, because
  a required check matches by name.

**What "blocks" means, and what enforces it, as of this record.**
1. **Policy, in force now.** An execution contract's `READY FOR OPERATOR REVIEW` requires green `fast` and
   `test` on the exact final PR head. That run is the PR's one canonical full-suite evidence, replacing a
   local full gate. The operator merges only with both green.
2. **Mechanism, not yet in force.** While this record was being written, the repository was private on
   GitHub Free, where branch protection and rulesets are unavailable (step-14 F-1: the API answers 403).
   `D-12`'s "protection makes that a mechanism" was therefore not yet true.
   - The repository became public on 2026-10-08, which makes protection available.
   - Protection follows this record's PR (S13 13a) **after it merges**. Requiring checks that do not yet
     exist on `main` would block every merge. The primary session then enables protection, or a ruleset,
     on `main`.
   - It requires `fast` and `test` and "require branches to be up to date", because AC-1's scan reads
     merge structure.
   - That settings change is outside every PR, and is recorded where it is made.
3. **A known property of required checks.** A job skipped by its `if:` reports success to a required
   check, as `test` does on a draft PR. A draft cannot be merged, and `ready_for_review` re-runs `test` on
   the same head, so this opens no gap in practice.

**Why.** §16's layers become triggers without re-tiering any existing test. Each red layer has one
meaning. And no claim about enforcement is stronger than the repository's settings.

**Accepted limitations.**
- Layer 2 is the whole suite, about 70 % of it six long runs, so the PR loop's `test` is not fast.
- The canonical evidence arrives when CI finishes, not when the session stops typing.
- Until protection is enabled, a merge without green checks is prevented by discipline only.

**Note, 2026-10-08 (S10 PR P3, `pr-s10-p3-python-sdk.md` C5): the Python layers and the first jobs
outside the container.**
- **Layers.** `ci_layer.py` gains `python` (the Python static checks, `cargo build -p mineworld-cli`,
  `pytest` including the real-server tests, the scratch check) and `python-smoke` (the static checks and
  `pytest -m "not real_server"`, no Rust build). `core` is unchanged. `fast` gains the four static
  commands (`uv sync --locked`, `ruff check`, `ruff format --check`, `pyright`), because they cost about
  6 s against the primary session's 60 s allowance (QP3-3), measured on PR #98's first run.
- **Job.** One new job, `python`, a matrix over `ubuntu-24.04`, `windows-2025` and `macos-15`, each
  running the full `python` layer: on that first run Windows took 2 min 26 s longer than Ubuntu and
  macOS 8 s less, both under the 3-minute rule that would otherwise have reduced Windows to
  `python-smoke` and dropped macOS. `fast` and `test` keep their names and contents. `python` is not a
  required check until the primary session makes it one.
- **Departure from "inside the container".** The Linux leg runs in the toolchain container like every
  other layer, with uv copied into the image pinned by digest (DEP-26). Windows and macOS cannot run that
  container, so their legs install what they need on the runner, each pinned: the Rust toolchain from
  `rust-toolchain.toml` (`rustup toolchain install`), uv through `astral-sh/setup-uv` pinned by commit
  SHA at uv 0.12.5, and Python 3.12 (the `requires-python` floor) through uv. They still name a layer and
  never a command of it. This departure exists only because the operator requires every platform
  (2026-10-08) and a Linux container cannot show Windows or macOS behaviour.

---

## DEP-20 — Client collision: Godot's built-in Jolt Physics, never authoritative

**Date** 2026-10-08 · **Status** selected; in force from S14 PR 16a · **Approved by** the operator
(step-11 §1.2 D-2, Jolt as the direction) and the primary session at PR 16a's design freeze (step-15
§19.0) · **Relates to** `DEP-4`, `DEP-13`, [`NETWORKING.md`](NETWORKING.md) §4,
[`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §12 · **Design** `.structured-coding/plans/mvp0/step-11-bodies.md`
§§3.2, 15.2 (drafted there as "DEP-14", renumbered by `overall.md` "Parallel build-out, 2026-10-08",
ruling 6); `step-15-demo-3d.md` §§8.1, 19

**Problem.** The 3D client must stop the player at walls and, from S15 PR 12e, at people and objects,
predicting locally what the server decides (`NETWORKING.md` §4). It needs a collision engine for its own
body; nothing it simulates is authoritative.

**Options considered** (`REUSE_POLICY.md` §§11–12, §17 — both directions).

```text
(a) Jolt Physics, built into Godot 4.7.2, MIT          chosen
(b) Godot Physics (what ran before; the setting's      kept as the one-line fallback
    DEFAULT in this project)
(c) appsinacup/godot-rapier-physics (GDExtension)      symmetry with the server buys nothing when the
                                                       server always wins; a binary per platform
(d) godot-jolt/godot-jolt (the former extension)       maintenance mode; supports Godot 4.3-4.6 only
(e) our own collision                                  a physics engine is commodity infrastructure
```

**Choice: Jolt Physics, selected explicitly** — `[physics] 3d/physics_engine="Jolt Physics"` in
`clients/3d-spike/project.godot`, every other Jolt setting at its default. `CharacterBody3D.move_and_slide`
is the only physics the player uses, unchanged. Jolt has been built into Godot since 4.4 and is the
default for new projects since 4.6; existing projects keep Godot Physics unless they select it.

**Why not ourselves.** Collision is commodity infrastructure (`REUSE_POLICY.md` §4), and the engine
already ships two.

**Isolating interface.** The project setting is the only place that names Jolt. Every script uses the
generic `PhysicsServer3D` nodes and queries, so the fallback to Godot Physics is that one line
(step-15 §9.4). No physics concept reaches the wire protocol: colliders are built from disclosed shapes,
and the client's only rule is to adopt the server's answer (12e).

**Facts, measured 2026-10-08** (step-15 §19.7 E16a-1, E16a-2; Godot `4.7.2.stable.official.ed1daf0bf`,
Apple M5):

```text
which engine   the server singleton reports the abstract PhysicsServer3D under either engine; the
               running engine is told by a new space's solver iterations: 16 with Godot Physics
               (its own setting), 8 with Jolt (clients/3d-spike/tools/physics_engine.gd). With the
               setting DEFAULT this project ran Godot Physics
the slice      every accepted check passes unchanged on Jolt: --drive (the loop closes within
               0.10 m, jumps 0.488 m, 0.0000 m camera switches), --threshold, --character,
               --world --link (50 moves accepted, none refused), --world --conversation; the body
               settles within 2 mm of where it settled on Godot Physics
frame time     within run-to-run noise on both scenes (step-15 §19.7 E16a-2)
```

**Accepted limitations.** Godot states its physics, on either engine, is not deterministic; acceptable,
because nothing the client simulates is authoritative. Jolt's documented differences from Godot Physics
(position-only stabilization, kinematic bodies not reporting contacts with static or kinematic bodies
unless `generate_all_kinematic_contacts` is set) are re-checked by 12e, which first gives the player
bodies to meet.

**Revisit** if a client check fails on Jolt in a way the client cannot fix (step-15 QS14-12): fall back
to Godot Physics only with the operator, and amend this record.

---

## ARC-54 — A world's requirements are resolved against the build and the named pack roots

**Date** 2026-10-08 · **Approved by** the operator (S16 QSE-8, QSE-13) and the primary session at PR
E-b's design freeze (step-16 §15.0; FQ-b1, FQ-b3, FQ-b4) · **Implements**
[`MODULE_SPEC.md`](MODULE_SPEC.md) §4 (the model's `requires:`), §4.1, §8.1;
[`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md) §5.0 · **Relates to** `ARC-33`, `ARC-53`, `ARC-55`, `DEP-21`,
`ARC-61` · **Design** `.structured-coding/plans/mvp0/step-16-packages.md` §4.2, §15 (S16, PR E-b)

**Problem.** `ARC-53` gave every pack an identity, but a world could not yet say which packs it needs or
at which versions, and nothing refused a world whose packs were missing or wrong. "Install modules →
compose world" needs the composition to be stated by the world and checked before the world runs.

**Choice.**

1. **`requires:`**, a top-level map in `world.yaml`, pack id → semver range (Cargo's meaning,
   `DEP-21`). It replaces the frozen model's `entity_packs:` and `presentation_profile:` (QSE-8). It names
   packs and versions only: `systems:` still says which systems are enabled and in which order, and how a
   pack is configured belongs to the generic configuration seam (`ARC-61`, S17). Neither of the two reads
   the other.
2. **Only packs that are not bundled are named.** A code pack is *bundled* when it was compiled from the
   framework's own workspace; its version is the framework's, so the world's `mineworld:` range covers it.
   `package!()` decides this at compile time — the crate's manifest directory begins with the framework
   workspace's root, taken as `mineworld-packages`' own manifest directory without its last component —
   and records only a boolean, never a path. Every other code pack is third-party.
3. **Pack roots, explicit only.** Packs are searched in the build and in the directories named by
   `--packs DIR` (in order), then by the `MINEWORLD_PACKS` path list. Nothing else is searched — not the
   world's own directory, not the working directory. Only the composition root reads the environment;
   `mineworld_packages::PackRoots::new(cli, env)` is a pure function of both.
4. **Resolution is checked, never chosen.** One installed version per pack; no solver. A pure function,
   `mineworld_packages::resolve`, takes the world's stated facts, each enabled system's pack and every
   data pack found under the roots, and refuses the first failure by name, in a fixed order: a duplicate
   id; per requirement in id order — absent (naming every place searched), a type a world cannot require
   (a World or Controller Pack; an Entity Pack until S16's E-d), bundled, outside the range; an enabled
   third-party system that is not required; a licence outside the policy (`ARC-55`).
5. **Where.** `WorldPack::read_with(root, &PackRoots)` resolves after `systems` and before content.
   `WorldPack::read(root)` is `read_with` with no roots, so a world without `requires:` reads exactly as
   before and no existing caller changes. The loader that builds a world (`load.rs`) is not involved:
   nothing is seeded from a composition.
6. **Never world state.** No fact, `Metadata` or save carries a version, a root or a composition (S16
   QSE-14). A resume resolves again against the roots given then, and its composition is still checked by
   `SystemVersion` alone.
7. **What prints it.** `mineworld packs resolve <world>` prints the whole composition. `mineworld
   validate` prints one `requires` line per requirement, only when the world states `requires:`, so a
   world without one reports exactly what it did before (a departure from step-16 §9.3's "validate
   prints the composition", recorded at freeze, FQ-b4).
8. **Every command that reads a world takes `--packs`**: `validate`, `run`, `server`, `replay`,
   `biography`, and `packs list | show | validate | resolve`. `create` does not: it writes a world that
   requires nothing. Its template states `version: 0.1.0`, `license: MIT` (to be replaced by the author)
   and `mineworld: "^0.1"` (FQ-b1).

A World Pack's own `world.version` and `world.license` stay optional to the loader (`ARC-53` point 6);
`packs validate` and `packs resolve` require them.

**Accepted limitations.**
- A third-party code pack *vendored inside* the repository counts as bundled, because its manifest lies
  under the workspace root. Vendoring a third-party pack into the tree is not a supported installation in
  MVP-0.
- The rule that an enabled third-party system must be required is proven through the real binary only
  once a third-party code pack exists in the build (S16's E-c); in E-b it is held by the resolver's own
  tests.
- A data pack's own `dependencies` on other data packs are not resolved; `pack.yaml` refuses the field.

---

## ARC-55 — Which licences a pack may carry: a default policy, typed, overridable per world

**Date** 2026-10-08 · **Approved by** the primary session at PR E-b's design freeze (step-16 §15.0,
FQ-b2 as changed) · **Implements** [`MODULE_SPEC.md`](MODULE_SPEC.md) §4.1,
[`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md) §5.0 · **Relates to** `DEP-8`, `DEP-21`, `ARC-53`, `ARC-54`,
`ARC-61` · **Design** `.structured-coding/plans/mvp0/step-16-packages.md` §4.7, §15 (S16, PR E-b)

**Problem.** MineWorld redistributes what a world is composed of, and "a pack whose licence cannot be
resolved cannot be redistributed" (`PACKAGE_FORMAT.md` §5). `ARC-53` checks that a licence *is* an SPDX
expression; nothing yet judged whether it is one MineWorld may redistribute under MIT. MineWorld is a
framework, so the judgement is a default a world may change, not a rule fixed in code.

**Choice.**

1. **The default allow-list** is the common permissive set compatible with MIT redistribution: `MIT`,
   `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `CC0-1.0`, `Unlicense`.
2. **An expression is judged by `spdx`'s own evaluator** (`DEP-21`): allowed when it can be satisfied
   with listed identifiers alone, a requirement carrying a `WITH` addition or `+` counting as not
   listed. `MIT OR GPL-3.0-only` is allowed (one branch suffices); `MIT AND GPL-3.0-only` is not. A
   refusal names the pack, its expression, the identifiers that failed and the allowed ones.
3. **The policy is a typed value**, `mineworld_packages::LicencePolicy`, passed to the resolver rather
   than read from a constant; `LicencePolicy::default()` is the list above.
4. **It applies at resolution** (`ARC-54` point 4) — to the world's own licence when stated, every
   required pack, and every enabled system's pack — and in `mineworld packs validate`. `packs list` and
   `packs show` print licences and do not judge them. `cargo-deny` over the whole Cargo graph in CI
   remains S16 E-c's and S13's.
5. **Overridable per world, through no key of its own.** When the generic configuration seam lands
   (`ARC-61`, S17's PL-a), a world's `configure/packages.yaml` decodes into `LicencePolicy` and replaces
   the default for that world, narrowing or extending it. Until then every world uses the default. No
   `world.yaml` key is added for it.

**Options considered.** (a) A closed list hard-coded in the resolver — rejected: a framework must let a
world decide. (b) A dedicated `license_policy:` key in `world.yaml` — rejected at freeze: one generic
seam for configuration, not a bespoke key per concern. (c) A typed default overridable through the
generic seam — chosen.

**Candidates left out of the default, on purpose.** `CC-BY-4.0` requires attribution, which MineWorld
does not yet track; it is the first candidate for asset packs once it does. Copyleft licences
(`CC-BY-SA-*`, `GPL-*`) stay out, for `DEP-8`'s reason.

**Note, 2026-10-08 (S17, PR IL-b; QIA-1, QIB-7) — point 5 is wired.** `packages` is a framework key of
`configure:` (`ARC-61` note). When a world lists it, `configure/packages.yaml` is decoded straight into
`LicencePolicy` (`{ allowed: [...] }`, each identifier checked against the SPDX list; a refusal is
`LicencePolicyInvalid`, with its line and column) right after the world's systems resolve and before its
requirements do, and that policy replaces the default for the world: in resolution and in `mineworld
packs validate <world dir>`. `packs validate <pack dir>` keeps the default. The policy is not world
state: it is never seeded and never drift-checked, and is read again at every read, including every
resume.

---

## ARC-53 note — classification moved to E-b (2026-10-08)

Point 2's identity record gains one field in S16's PR E-b: `Package::bundled`, computed by `package!()`
at compile time (`ARC-54` point 2). Step-16 §14.1 had placed bundled-versus-third-party classification in
E-c; E-b needs it for two of its rules, and the primary session moved it at E-b's freeze (FQ-b3). No pack's
source changes: only the macro's expansion does.

---

## DEP-29 — Test scratch: a `std`-only helper of our own, not the `tempfile` crate

**Date** 2026-10-08 · **Status** selected; no dependency added · **Approved by** the primary session at
the freeze of PR test-hygiene (its QTH-3, which asked for this record) · **Relates to**
`ENGINEERING_STANDARDS.md` §22 "Test scratch" · **Design**
`.structured-coding/plans/mvp0/pr-test-hygiene.md` §§2–4

**Problem.** A passing `cargo test --workspace` left 135 entries, about 16 GB, under `target/tmp`:
helpers removed a test's directory *before* the test and never after. Every test must remove its own
scratch. The audit fixed what the mechanism must do:
- the scratch path's last component is exactly the name the test gives, because a World Pack's id is
  its directory's name and refusal messages name it;
- some tests need a path that does **not** exist yet (the CLI creates the save), others an existing
  empty directory;
- two tests of one process must never share a scratch (12c's "database is locked"), and two processes
  on one `target/` must not collide;
- saves an operator wants to keep (`BODIES_YARD_SAVES`) must be keepable, side by side in one
  directory.

**Options considered** (`REUSE_POLICY.md` §§11–12, §17 — both directions):

```text
(a) `tempfile` 3.27 (`TempDir`, removed on drop; MIT OR Apache-2.0; mature). Not in Cargo.lock: it
    would add tempfile, fastrand and rustix (+ linux-raw-sys on Linux)
(b) a std-only test-support crate of our own (`mineworld-test-support`, publish = false)
(c) a remove_dir_all at the end of every test
```

**Choice: (b).** No dependency is added.

**Why not the others** (`REUSE_POLICY.md` §12's reasons):

- **(a) `tempfile` — missing required semantics, so a wrapper of the whole helper's size anyway.**
  - Its names carry a random suffix. The leaf name must be exact, so the test's directory would be a
    child of the `TempDir`, which already is (b)'s layout.
  - Random containers scatter kept saves, so `BODIES_YARD_SAVES` could no longer point at one
    directory.
  - A same-name clash between two tests is hidden by random names rather than reported; (b) panics
    and names it, which turns a locked database into a clear failure.
  - Keeping a scratch on failure needs `std::thread::panicking()` in our own `Drop` anyway.
  - What it does well — secure, race-free creation in a shared world-writable directory — is not
    needed for a test's scratch under the build's own `target/`.
- **(c) cleanup in each test — the failure mode that caused the problem.** It does not run on an early
  `return` or a panic, nothing checks it, and about sixty sites would each have to remember it.

**Isolating interface.** `mineworld-test-support`: `Scratch` and the `scratch!` macro. Tests name
only those; if the helper is ever replaced (by `tempfile` or otherwise), only that crate changes.

**Accepted limitations and the revisit trigger.**
- A process killed with `SIGKILL` (or a test with an infinite loop that is killed) leaves its
  `mineworld-scratch-<pid>` directory; it is under `target/`, so `cargo clean` removes it, and
  `check_scratch.py left` reports it.
- Revisit if test scratch must live outside `target/` in a shared, world-writable directory, where
  `tempfile`'s secure creation matters.

---

## DEP-29 note — ending a test's child process on every platform (2026-10-09, S13 PR 13w)

**Approved by** the primary session at 13w's design freeze (step-14 §15, QW-2) · **Design**
`.structured-coding/plans/mvp0/step-14-ci.md` §15.3–15.4

The decision is unchanged: `mineworld-test-support` stays standard-library-only. It gains a module
`process` so that the default suite compiles and passes on Windows as on Unix (the operator's
all-platforms requirement, step-14 §13.0.1):

- **`kill(&mut Child) -> Killed`** and **`Killed::killed()`**: whether the kill ended the child. On Unix
  the status is signal 9 (`ExitStatusExt`, now used in this crate only). On Windows std's `Child::kill`
  is `TerminateProcess(handle, 1)`, so the status is exit code 1, and the child must have been running
  (`try_wait` is `None`) immediately before the kill. Tests assert the verdict, never a raw status.
- **`interruptible(Command)`, `InterruptibleChild` and `interrupt(&mut InterruptibleChild)`**: an
  operator's stop. On Unix, `SIGINT` through `sh -c "kill -INT <pid>"` (moved unchanged from the CLI's
  test support), then a wait. On Windows the child is spawned with `CREATE_NEW_PROCESS_GROUP` (std's
  safe `CommandExt::creation_flags`), and `GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, <pid>)` reaches
  that group and nothing else; Ctrl-C cannot be sent to one group. `interrupt` accepts only an
  `InterruptibleChild`, so a Ctrl-Break can never be broadcast to every process on the console (a
  compile-fail doctest pins it).

**Options considered for the one Windows call** (`REUSE_POLICY.md` §§11–12, both directions):

```text
(a) one `unsafe extern "system"` declaration of GenerateConsoleCtrlEvent from kernel32, which std
    already links on every Windows target
(b) `windows-sys` 0.61.2 (already in Cargo.lock, transitively) as a cfg(windows) dependency
(c) libc / nix on Unix in place of `sh -c kill`
```

**Choice: (a), and `sh -c kill` on Unix.**
- (b) is a real dependency (`REUSE_POLICY.md` §11) for one function, and its functions are `unsafe` to
  call as well; it buys no safety over (a).
- (c) adds a dependency and `unsafe` to replace a command that every Unix CI image and developer machine
  already has, and that 13a's container proves in CI.
- The crate's `#![forbid(unsafe_code)]` becomes `#![deny(unsafe_code)]`, with exactly one
  `#[allow(unsafe_code)]` function, `send_interrupt` under `cfg(windows)`. The call takes two integers by
  value and touches no memory.

**Accepted limitations.**
- **The residual race on Windows.** A child that exits with code 1 by itself in the instant between
  `try_wait` and `TerminateProcess` would read as killed. Every killing test also checks separately that
  the child had not finished (no final summary line, or a revision short of the end), so a false
  "killed" cannot also pass that check. On Unix the verdict is unambiguous.
- **A console is needed on Windows.** `GenerateConsoleCtrlEvent` works only for a caller attached to a
  console. Without one the call fails, and `interrupt` returns a named error ("no console attached;
  Ctrl-Break cannot be delivered"); it never passes silently.
- The server must handle Ctrl-Break itself: `tokio::signal::ctrl_c()` does not catch it on Windows
  (step-14 F-13w-2). `serve.rs`'s `stop_requested` does, since S11-D (step-12 SD-D13, QW-1).

**Revisit** if a test needs to stop a child that is not a console process, or the server gains a stop that
needs no signal (an admin `shutdown` frame); `interrupt` would then wrap that instead.

---

## ARC-66 — A System Pack from outside this repository is installed by the same two lines, pinned to a commit

**Date** 2026-10-08 · **Approved by** the operator (S16 QSE-2, QSE-3, QSE-12, QSE-16; FQ-c1, FQ-c2,
FQ-c5) and the primary session at PR E-c's design freeze (step-16 §16.0) · **Supersedes in part**
`ARC-33`'s sentence placing "packs from outside this repository" outside MVP-0 · **Implements**
[`MODULE_SPEC.md`](MODULE_SPEC.md) §3.1, §3.2; [`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md) §8 · **Relates
to** `ARC-8`, `ARC-26`, `ARC-33`, `ARC-35`, `ARC-38`, `ARC-53`, `ARC-54`, `ARC-55`, `DEP-12`, `DEP-22`,
`DEP-23` · **Design** `.structured-coding/plans/mvp0/step-16-packages.md` §3, §4.4, §16 (S16, PR E-c)

**Problem.** The frozen top-level criterion asks for *independently installable* interaction systems.
Until this decision every System Pack in the build was compiled from this repository, and `ARC-33`
placed a pack from anywhere else outside MVP-0 together with the WASM tier. A framework whose only packs
live in its own tree has not shown that somebody else can write one.

**Choice.**

1. **Same two lines, pinned.** A System Pack whose source lives in another repository is installed by
   `ARC-33`'s two lines in `systems/installed/`, the Cargo line naming a git source and a **full 40-hex
   commit id**, and the lock records the same commit:

   ```text
   systems/installed/Cargo.toml   <package> = { git = "<url>", rev = "<40 hex>" }   (one line)
   systems/installed/src/lib.rs   <Variant> => <crate>::<System>,                    (one line)
   Cargo.lock                     source = "git+<url>?rev=<sha>#<sha>"              (generated)
   ```

   No `branch`, no `tag`: a tag can be moved, a commit cannot. The dependency key is the pack's own
   package name, never a rename (`<package> = { package = "…" }`): the installed set's consistency
   guard compares the manifest's keys with the crates `core::any::type_name` reports, and that is the
   real crate name (spike SC-2 observed a rename fail it).
2. **The pack names the framework by version, and the build maps the names once.** A third-party pack
   depends on MineWorld's crates by version requirement (`mineworld-sdk = "0.1"`), so it builds against
   whichever MineWorld its consumer has. The reference build maps those names to its own paths in one
   `[patch.crates-io]` table — the **published surface** — and nothing else: `mineworld-sdk`,
   `mineworld-kernel`, `mineworld-contracts`, `mineworld-authoring`, `mineworld-presence`,
   `mineworld-inventory`. A pack may depend on these and on registry crates; a pack's dev-dependencies
   are not part of the surface (Cargo never resolves a dependency's dev-dependencies). Publishing one
   more crate to third parties is one line in that table, a reviewed act.
3. **Where the table lives: `.cargo/config.toml` at the repository root**, a supported Cargo location
   for `[patch]` (FQ-c2 (a)). Paths in it are relative and written with `/`; Cargo resolves them against
   the directory that holds `.cargo/` on every operating system. Finding F-Ec1 decided the location:
   `ARC-35`'s check 2, bullet 3 scans every `*.rs` and every `Cargo.toml` — the root one explicitly —
   outside `systems/`, `worlds/` and `tests/acceptance/` for the market's crate names, and
   `mineworld-inventory` is one of them (a producing pack states `items-produced` only through
   `mineworld_inventory::produce`, `ARC-26`, `ARC-38`). The table in the root `Cargo.toml` would fail
   `AC-1`. In `.cargo/config.toml` it is build configuration, which the scan does not read — and this
   record does not lean on that silently: the published-surface table names one market crate outside
   `systems/`, which is consistent with `ARC-35`'s claim, because installing the market still edited only
   `systems/`, and publishing a crate to third parties is a separate, reviewed act after it. The
   acceptance test `package_sources.rs` reads the table, so it is guarded rather than invisible.
   `ac1_composability.rs` and `ARC-35` are not edited.
4. **Third-party is decided by where the pack was compiled** (`ARC-54` point 2). A git checkout lies
   under `CARGO_HOME`, outside the framework workspace, so `package!()` records it third-party; a world
   that enables its system must `require` the pack, in range, and its licence is judged by `ARC-55`.
   `mineworld packs list` says `bundled` or `third-party` for every code pack. The binary does not print
   the revision: Cargo exposes no variable for a dependency's source, and reading `Cargo.lock` at run
   time is `cargo_metadata` at run time, rejected in step-16 §5 row 15. The revision is stated once, on
   the pack's line in `systems/installed/Cargo.toml`, and `package_sources.rs` proves `Cargo.lock` holds
   the same one.
5. **Guarded by one acceptance test file**, `tests/acceptance/tests/package_sources.rs`, which names no
   pack:
   - over `Cargo.lock`: every package without a source is a workspace member, every registry package
     has a checksum, every git package's source is `git+<url>?rev=<40 hex>#<the same 40 hex>`; no
     `mineworld-*` package comes from a registry; each git package's revision equals the one its line
     in `systems/installed/Cargo.toml` states; only the installed set depends on a git package;
   - over `cargo metadata --locked --offline`: the git packages are exactly the installed capabilities
     whose `bundled()` is false; each is not a workspace member, its manifest is not under the workspace
     root, every normal dependency is in the published-surface table or a registry crate, no dependency
     has a `path`, and its manifest says no `workspace = true`;
   - over the tracked files: no `*.rs` and no `Cargo.toml` other than `systems/installed/Cargo.toml`
     spells a git package's crate name; `systems/installed/src/lib.rs` may name it once.
6. **Reproducible offline after one fetch** (§16 PD-26). `cargo fetch --locked`, then `--offline
   --frozen`; or `cargo vendor --locked <directory outside the tree>` and an empty `CARGO_HOME` with the
   generated `--config`. The vendor directory is never committed. A committed test does not vendor; the
   lock guard proves the precondition (every non-workspace package content-addressed).
7. **Trusted, reviewed code.** A pinned pack is compiled into the binary and runs in process with the
   kernel's authority, like every MVP-0 pack (`ARCHITECTURE.md` §12). It is reviewed like any
   dependency before the reference build installs it, and recorded as one (`DEP-23` is the first).
   Installing without a rebuild, and running code nobody reviewed, remain `ARC-8`'s Tier 1.
8. **Upkeep** (FQ-c7). A framework PR that breaks a pinned pack updates the pack in its repository and the
   `rev` here, in the same review. A breaking change to a published crate is a MINOR release before 1.0
   (`ARC-53`'s rule, extended to the published surface).

**Options considered** (`REUSE_POLICY.md` §§2, 11–12, 17 — both directions; step-16 §16.4):

```text
M1 a Cargo git dependency with `rev`, locked by Cargo.lock          ADOPT
M2 vendoring into this repository (copy, in-tree `cargo vendor`, subtree)
                                                                      REJECT: inside the tree it is
                                                                      bundled (ARC-54's limitation) and,
                                                                      under systems/, a workspace member
M3 a git submodule + a path dependency                               REJECT: bundled; the lock loses
                                                                      the pin; four places, not two
M4 `cargo vendor` outside the tree                                   ADOPT as the offline route, never
                                                                      committed
M5 a registry (crates.io, a private one, a static index)             REJECT for MVP-0: the Phase 3
                                                                      non-goal; publishes names before a
                                                                      stable contract. The patch table
                                                                      makes the move a deletion
M6 a hermetic build system (Nix crane/naersk, Bazel rules_rust)      REJECT: a second build description,
                                                                      framework lock-in
M7 our own fetcher (tarball + sha256 into vendor/)                   REJECT: rebuilds M1 + M4 badly
```

Not reinventing: Cargo fetches, pins, locks, vendors and enforces the framework requirement (a pack
requiring `mineworld-sdk = "0.2"` is refused by Cargo). MineWorld adds only the published-surface table
and a guard over the lock. Not forcing: no registry, second build system or fetch script where a git
`rev` already does the job.

**How it fails, closed.**
- The table removed (spike SC-5): `cargo metadata` refuses — "no matching package named
  `mineworld-contracts` found, location searched: crates.io index". A crate of that name published by a
  stranger would change that, which is why the lock guard refuses any `mineworld-*` package with a
  registry source and `cargo-deny`'s `sources` check allows only crates.io and the pack's repository
  (`DEP-22`).
- Cargo run from outside the repository (`--manifest-path <repo>/Cargo.toml` from another directory)
  does not read `.cargo/config.toml` and refuses the same way (spike SC-6). Run Cargo in the repository.
- The pack's repository or commit gone: the fetch fails, naming the source; an existing vendor
  directory still builds it.

**Accepted limitations.**
- `.cargo/config.toml` is read from the working directory upward, so the published surface depends on
  where Cargo runs (above).
- The CI cache key (`Cargo.lock`, toolchain, Dockerfile) does not include `.cargo/config.toml`; a change
  to the table that changes resolution changes `Cargo.lock` too.
- A third-party pack takes no world configuration in this first instance (FQ-c9); the seam
  (`ARC-61`) is the same for it, and its third-party reach is S17's to demonstrate.
- Revisit together with `ARC-33` and `DEP-12` when a registry or `ARC-8`'s Tier 1 arrives.

---

## DEP-23 — The first third-party System Pack: `acme-fishing`, pinned by git revision

**Date** 2026-10-08 · **Status** selected; installed in S16 PR E-c · **Approved by** the operator
(QSE-3, QSE-12, QSE-16; FQ-c1, FQ-c4, FQ-c5) and the primary session at E-c's freeze (step-16 §16.0) ·
**Relates to** `ARC-66`, `ARC-26`, `ARC-34`, `ARC-38`, `ARC-55` · **Design**
`.structured-coding/plans/mvp0/step-16-packages.md` §16.2 PD-27 (S16, PR E-c)

**What.** `acme-fishing` 0.1.0, a System Pack with system id `fishing`: a place's `fishing:` section
(`catch: <item key>`, `minutes: 30..=240`) makes it a fishing spot; a living person at a spot who can
carry one more is offered `fish`, a complete affordance with no target (`ARC-34`); a `catch` process ends
after the spot's minutes and, if the angler is still there and can carry one more, states inventory's
`items-produced` through `mineworld_inventory::produce` (`ARC-26`, `ARC-38`). It depends on `presence` and
`inventory`, discloses nothing, takes no configuration. Its law is step-16 §16.2's table.

**Where.** <https://github.com/yuema137/mineworld-pack-fishing>, branch `main`, a separate public
repository. The build installs it by `ARC-66`'s two lines; the revision is stated once, on its line in
`systems/installed/Cargo.toml` (the tag `v0.1.0` names the same commit, informatively).

**Why this and not a bundled pack.** It is the evidence `ARC-66` exists for: a pack written only
against the published surface, in a repository that is not this one, with its own tests and CI on Linux,
macOS and Windows, compiled into this build without a path into its source.

**Licence.** MIT (its `Cargo.toml` and `LICENSE`), on `ARC-55`'s default allow-list. Its normal
dependencies outside the published surface are `serde` and `serde_json`, both already in this build's
lock: installing it adds no registry crate to the graph.

**Who maintains it.** The MineWorld maintainers (the repository's owner is this project's operator). A
framework PR that breaks it updates it in its repository and the `rev` here, in the same review
(`ARC-66` point 8).

**Upgrade.** Commit to its repository; change the `rev` on its line; `cargo update -p acme-fishing`
regenerates the lock; a world whose `requires:` range no longer admits the new version is refused naming
both.

**Removal.** Delete its two lines; the lock is regenerated. A world enabling `fishing` is then refused
`UnknownSystem`, listing the systems the build provides; every other world is unaffected.

---

## DEP-22 — `cargo-deny` checks the code graph's licences, sources and bans in CI

**Date** 2026-10-08 · **Status** selected; integrated in S16 PR E-c (Ec-C7) · **Approved by** the
operator's reuse table (step-16 §5 row 11) and the primary session at E-c's freeze (FQ-c6) ·
**Relates to** `ARC-48`, `ARC-55`, `ARC-66`, `DEP-17`, `DEP-18`, `DEP-21` · **Design**
`.structured-coding/plans/mvp0/step-16-packages.md` §5 row 11, §16.2 PD-29 (S16, PR E-c)

**Problem.** `ARC-55` judges the licence a *pack* declares, at resolution. Nothing judged the licences
of every crate compiled into the binary, and nothing stated which sources a crate may come from — the
policy that keeps a crate published under a framework name by a stranger out of the build (`ARC-66`,
"How it fails, closed").

**Options considered** (`REUSE_POLICY.md` §§11–12, §17 — both directions):

```text
(a) cargo-deny (EmbarkStudios; MIT OR Apache-2.0): licences, sources, bans, advisories over a Cargo graph
(b) cargo-about / cargo-license: licence listing only, no source or ban policy
(c) a script of our own over `cargo metadata`
```

**Choice: (a)**, run by CI only — it is not a dependency of any crate. (b) lists licences but enforces
neither sources nor bans, so a second tool would still be needed; (c) is the wheel (a) is.

**Policy (`deny.toml`).**
- `[sources]`: crates.io, and git only from `https://github.com/yuema137/mineworld-pack-fishing`
  (`DEP-23`); anything else is refused.
- `[licenses]`: the allow-list the code graph actually needs — `ARC-55`'s eight, plus only what
  `cargo deny list` shows, each addition named in `deny.toml` with the crates that need it.
- `[bans]`: no wildcard version requirements.
- Advisories are **not** checked: the answer depends on a database fetched at run time, and `ARC-48`
  allows nothing to fail non-deterministically.

**Where it runs.** Installed in the CI image's `toolchain` stage at a pinned version with `--locked`,
and run by the `fast` layer (`scripts/ci_layer.py`): `cargo deny check licenses sources bans`.

**Weight, measured** at integration: recorded in step-16 §16.8 (install time, image size).

**Revisit** when an advisory database can be pinned and read offline, or when the graph needs a licence
outside the list (a reviewed addition here first).

**Note, 2026-10-09 (operator ruling on S19 TW-d's TWd-F4) — one crate-scoped exception.** `deny.toml`
gains `[[licenses.exceptions]] crate = "webpki-roots", allow = ["CDLA-Permissive-2.0"]`. `webpki-roots`
is Mozilla's root-certificate list as data; it is reached only through `ureq`'s `rustls` feature in
`mineworld-weather-fetch`'s off-by-default `fetch` feature (`DEP-31`), a developer tool that no world
build compiles. The exception names that crate only: CDLA-Permissive-2.0 is not added to the allow-list,
and any other crate carrying it is still refused. Every rustls route to a root store carries a CDLA root
list (`webpki-roots` or `webpki-root-certs`), and `native-tls` would need OpenSSL headers in the CI
image (step-19 §18.11 TWd-F4); the alternative, dropping `fetch`, was not chosen.

---

## ARC-49 — How AC-8 is measured

**Date** 2026-10-09 · **Status** decided; live from S13 PR 13b · **Approved by** the primary session at
13b's design freeze (step-14 §13, 2026-10-08), the operator for QB-1 and for the all-platforms
requirement · **Relates to** `ARC-23`, `ARC-30`, `ARC-48`, `DEP-17`, `DEP-18`, `DEP-19`,
[`MVP.md`](MVP.md) §9 `AC-8`, [`NETWORKING.md`](NETWORKING.md) §8 · **Design**
`.structured-coding/plans/mvp0/step-14-ci.md` §13

**Problem.** `AC-8`: "The same World Pack runs on a laptop and inside a cloud Docker container with no
semantic differences." The operator added on 2026-10-08: "we must support every platform: Mac, Linux and
Windows." The project needs an instrument that says, for one commit, whether those platforms produce the
same world, that locates a difference when they do not, and that cannot pass by comparing a platform with
itself.

**Options considered.**

```text
(a) a reference file recorded on the laptop and committed; each platform    every PR that changes a
    checks itself against it inside the required `test` check (step-14      world's facts turns red until
    §3.3 as first drawn)                                                     someone re-records on a Mac;
                                                                             slows a required check
(b) a live comparison: on one workflow run, each platform writes a          chosen
    parity record of the same commit, and a separate job compares them
(c) compare the FNV-1a fingerprint `run` prints                             "for reading; not evidence"
                                                                             (step-08 Q9)
(d) a Rust test with `sha2` (the reserved DEP-19 of the step design)        a dev-dependency and a Rust
                                                                             change for what stdlib Python
                                                                             does on every side
```

**Decision.**
1. **The record.** `scripts/ci_parity.py record` runs the `mineworld` binary (natively with `--binary`,
   or in the runtime image with `--image`) and writes one plain-text record, `key value` per line, `\n`
   line endings on every platform. It holds:
   - `[platform]`: OS, architecture, Rosetta translation (macOS), emulation (Windows), the container
     image and its platform when there is one, and `rustc`'s release;
   - `[source]`: the commit and the worlds, enumerated from every `worlds/*/world.yaml`, never listed by
     hand;
   - per world: the digest of `validate`'s output; `summary-300`, seed 7, 300 days in memory, every line
     but `wall`, digested and also kept verbatim; `summary-30s`, seed 7, 30 days with `--save`, every
     line but the header (it names the host's save path) and `wall`; and the save's tables.
   - Tables: `manifest` (the body decoded, `instance` removed, re-encoded with sorted keys, plus the
     `format` column), and `journal`, `facts` and `snapshots`, each as a row count and a SHA-256 over
     every column of every row in primary-key order, with a digest per 1 000-row chunk to locate a
     difference (`ARC-23`). A row encodes each column: NULL as `0x00`; otherwise `0x01`, then an
     integer as 8 bytes little-endian or a blob as its 8-byte length and its bytes.
   - Nothing else is excluded. `instance` is excluded because it is allocated from the wall clock
     (`ARC-27`); any further exclusion is a reviewed change to this record.
2. **The comparison.** `scripts/ci_parity.py compare` exits 0 only if:
   - **G-1** every record is well-formed;
   - **G-2** every record names the same commit and the same worlds, equal to `worlds/` of that commit;
   - **G-3** the records include Darwin arm64 (not translated, no container), Linux x86_64 in a
     container, and Windows x86_64 (native, not emulated). Records from fewer platforms never pass,
     however equal;
   - **G-4** every record's `rustc` release equals `rust-toolchain.toml`'s channel;
   - **G-5** every world's every key is equal across all records, a Linux arm64 record included.

   On a G-5 failure it names the world, groups the platforms that agree (which places the difference in
   an OS or an architecture), and prints the first differing summary line or the first differing table
   chunk with its key range. `--self-test` checks these verdicts against synthetic records and runs in
   `fast`.
3. **Where it runs.** Four jobs record the same commit: `scenario` (the runtime image, `linux/amd64`, on
   `ubuntu-24.04`), `linux-arm` (the runtime image, `linux/arm64`, on `ubuntu-24.04-arm`), `mac`
   (native, `macos-26`) and `windows` (native, `windows-2025`) (`DEP-19`). `ac8` downloads the four
   records and compares them. It runs `if: always()`, so a missing leg is a red `ac8`, never a skip.
4. **When.** On every push to `main`, on `workflow_dispatch`, and on `scratch/*-scenario` branches
   (planted-difference evidence). Not on pull requests. `ac8` is not a required check; like `scenario`
   it blocks main's health (`ARC-48`). A PR touching `systems/`, `kernel/`, `persistence/` or
   `worldpack/` should dispatch the workflow on its branch before review.
5. **The laptop.** At 13b's acceptance the operator's Mac records the final head and is compared with
   that head's Linux and Windows records (step-14 §13.4.4). That is `AC-8` literally; it is repeated by
   one command whenever doubt arises.

**Why not ourselves / why not the others.** Hashing, SQLite reading and process control are the
standard library's; nothing is built that a mature library provides. (a) moves a duty onto every lane and
onto whoever owns a Mac; (c) is not evidence; (d) adds a dependency for no gain and two hashing
implementations.

**Isolating interface.** The record format and `ci_parity.py`'s two subcommands. A runner label is one
line of the workflow (`DEP-19`). A world joins the record by existing under `worlds/`.

**Accepted limitations.**
- The macOS runner is not the operator's laptop: same OS major and target triple, another machine. The
  laptop comparison at acceptance covers the literal claim.
- `main` is checked after a merge, not before: a PR that introduces a platform difference merges green and
  `ac8` turns red on `main`.
- The long horizon (300-day saves, 1 000-day runs) is S13 PR 13c's nightly work.

---

## DEP-19 — AC-8's non-Linux sides: GitHub-hosted macOS arm64 (`macos-26`) and Windows x86_64 (`windows-2025`) runners

**Date** 2026-10-09 · **Status** selected; integrated in S13 PR 13b · **Approved by** the operator
(QB-1, 2026-10-08: "Use the macOS runner (macos-26). If billing later shows a charge, switch to the
designed fallback"; and "we must support every platform: Mac, Linux and Windows") and the primary session
at 13b's freeze · **Relates to** `ARC-49`, `DEP-17`, `ARC-48` · **Design**
`.structured-coding/plans/mvp0/step-14-ci.md` §§13.0.1, 13.3

Ruling 6 reserved `DEP-19` for `sha2`; 13b needs no `sha2` (`ARC-49` option (d)), and QB-3 gave the
number to this decision.

**Problem.** `AC-8`'s Linux side is the runtime container on hosted `ubuntu-24.04` (`DEP-17`). Its Mac
side, and the Windows side the operator requires, need machines that run on every check.

**Options considered** (step-14 §13.3).

```text
(a) the operator's laptop against a CI Linux record, once         chosen for acceptance, not continuous
(b) a committed laptop reference                                  declined (ARC-49 option (a))
(c) GitHub-hosted macos-26 (arm64)                                chosen: continuous
(c') macos-14                                                     deprecated in runner-images
(d) ubuntu-24.04-arm (Linux arm64) as a localizer                 chosen: places a difference in OS or
                                                                  architecture without Rosetta
(e) macos-26-intel (Darwin x86_64)                                not needed while (c) and (d) localize
(f) Rosetta on the laptop                                         emulation; a diagnostic only
(g) Docker Desktop linux/arm64 on the laptop                      a diagnostic only; (d) covers it
(h) a self-hosted runner on the operator's Mac                    runs fork-PR code on the operator's
                                                                  machine; depends on a laptop being awake
(i) larger macOS runners                                          paid, even on a public repository
(j) another CI service for macOS                                  a second platform for one job
(k) GitHub-hosted windows-2025 (Windows x86_64), native           chosen: the operator's third platform;
                                                                  hosted Windows cannot run the Linux image
```

**Choice.** (c) + (a) + (d) + (k). Labels are pinned (`macos-26`, `windows-2025`, `ubuntu-24.04-arm`),
never `-latest`, so a label migration never silently changes a platform under `AC-8`. The native legs
build with rustup taking `rust-toolchain.toml`'s channel, through the shared composite action
`.github/actions/native`, which names a layer of `scripts/ci_layer.py` and never a command.

**Cost basis.** Standard GitHub-hosted runners, macOS and Windows included, carry no per-minute charge on
a public repository (GitHub's published terms; the session cannot read billing, so this is the operator's
to confirm). No larger runner and no paid service is used.

**Isolating interface.** `ci_parity.py` runs anywhere Python 3 and the binary do. A runner is one line;
the native action is one file.

**Revisit triggers.**
- Billing shows a charge: switch to the designed fallback, (a) + (d) + (k), recorded as a deviation.
- A label is deprecated in `runner-images`: move to the next pinned label.
- A need for a machine GitHub does not host.

**Accepted limitations.** Hosted runners are not the operator's laptop (`ARC-49`). About five macOS jobs
run concurrently on the Free plan, so a busy day delays `ac8`; it never holds a merge.

---

## ARC-48 note — the scenario group of jobs and the Windows suite (2026-10-09, S13 PR 13b)

The decision is unchanged; its table gains these rows, and two details of the required checks change.

```text
job           layer / role                  trigger                                         merge
scenario      3 scenario: the runtime        push to main; workflow_dispatch;               blocks main's
              image's evidence (was          push to scratch/*-scenario                     health
              `image`) and the Linux x86_64
              parity record (ARC-49)
mac           parity record, macos-26        same                                           blocks main's health
linux-arm     parity record, Linux arm64     same                                           blocks main's health
windows       parity record, windows-2025    same                                           blocks main's health
ac8           AC-8's comparison (ARC-49)     same; if: always()                             blocks main's health
test-windows  the `core` layer natively on   push to main; workflow_dispatch                reports
              windows-2025
```

- `image` is renamed `scenario`; its scratch route `scratch/*-image` becomes `scratch/*-scenario`.
- `fast` gains one command, `python3 scripts/ci_parity.py --self-test` (under a second). `test` is not
  run on `scratch/*-scenario` branches, as it was not on `-image` ones. `core`'s commands are unchanged.
- `test-windows` reports only. It is red until the Windows-portability PR (13w) makes the suite compile
  and pass there; that PR adds its `pull_request` trigger, and only then may the operator make it a
  required check (QB-11).
- None of the new jobs runs on `pull_request`, and none is required (QB-8).

---

## ARC-48 note — the default suite on Windows and macOS (2026-10-09, S13 PR 13w)

The decision is unchanged; its table gains or changes these rows (step-14 §15.5, QW-4, QW-5).

```text
job           layer / role                  trigger                                         merge
test-windows  the `core` layer natively on  non-draft pull_request; push to main;          reports
              windows-2025                  push to scratch/** (not -image, -scenario);
                                            workflow_dispatch
test-macos    the `core` layer natively on  the same                                       reports
              macos-26
```

- **`test-windows` gains its `pull_request` trigger**, in the PR that makes it green (13w), as 13b's
  note said it would. `test-macos` is new, with the same triggers. Both carry `test`'s triggers plus
  dispatch, so a mutation pushed to a `scratch/` branch is judged on all three operating systems.
- **Neither is a required check.** Making them required is the operator's settings change (QB-11): the
  recommendation is after five consecutive green `main` pushes.
- **E-c/E-d's `platforms` job stays** (QB-15, QW-4's fallback). Retiring it was conditional on its layer
  being a strict subset of `core`. E-c (#99) merged first and added its offline vendor check (PD-p3,
  EC-3 (b): `ci_layer.py --offline-check`), which `core` does not run, so the condition fails and the job
  is kept unchanged. The rest of the layer duplicates part of `core` on the same runners; trimming it to
  what `core` lacks is a later, separate change.
- Wall times are recorded in step-14 §15.8 (W-C4); both jobs have a 60-minute timeout and run in
  parallel with `test`.

---

## ARC-48 note — documentation-only changes do not run the build (2026-10-09, S13 PR 13x)

The decision is unchanged in what each layer runs; its triggers change (step-14 §16, under the
operator's requirement of 2026-10-09 that a documentation-only update must not trigger CI).

```text
job        layer / role                         trigger                                  merge
changes    classify the diff: code or docs      every run                                (feeds fast)
           (scripts/ci_changes.py)
fast       code: 1 fast structural              push to main and scratch/**;             blocks
           docs only: the `docs` layer          pull_request; workflow_dispatch
test, python, platforms, test-windows,          as before, and only when changes says    as before
test-macos, scenario, linux-arm, mac,           code; otherwise skipped
windows, ac8
```

- **The docs set is fixed by audit, not by guess** (step-14 §16.3): `docs/**`,
  `.structured-coding/plans/**`, the root `README*.md`, `CLAUDE.md`. Nothing else is documentation,
  including subdirectory READMEs, some of which tests read (`cognition/lm-controller/README.md`,
  `server/PROTOCOL.md`). A diff is docs-only when every changed path, renames split into deletion and
  addition, is in the set; an empty diff, an unknown base (a new branch, a force-push) and any `git`
  failure are code. A push to `main` and a dispatch are always code, so `main` stays fully verified.
- **The `docs` layer** (`ci_layer.py docs`) is `check_doc_headings.py`, `check_decision_ids.py` and
  `ci_changes.py --self-test`, the only CI commands that read the docs set, run on the runner's Python
  in seconds. `fast` also runs the classifier's self-test.
- **Required checks still report.** No workflow-level `paths-ignore` (a workflow that never starts leaves
  `fast` and `test` pending). Skipped jobs use a job-level `if:` on `needs.changes.outputs.code`, which a
  required check reads as passing. `fast` runs `if: always()` and fails first if `changes` did not
  succeed, so a broken classifier cannot let a PR merge with everything skipped.
- **One run per PR commit.** `push` now triggers on `main` and `scratch/**` only; feature branches are
  run by `pull_request`. This supersedes the "Economy" bullet that branch pushes run `fast` even when a
  PR also runs it: a branch with no PR is not run until a (draft) PR is opened.
- **Reuse.** `dorny/paths-filter` was considered and rejected, to avoid a third-party action holding a
  repository-read token for a 30-line check that `git diff` already answers.
- **The standards agree.** `ENGINEERING_STANDARDS.md` §15 ("every pull request should run" format,
  compile, lint and core tests) and §16 ("run on every change") now say that a documentation-only change
  as defined here runs the documentation checks only: no other check reads its files, so their verdict
  is the base's, and `main` runs every layer on every push.

---

## ARC-48 note — layer 4, the nightly workflow (2026-10-10, S13 PR 13c)

The decision is unchanged in what `ci.yml`'s layers run; its table gains the rows of `ARC-83`, all in a
second workflow, `.github/workflows/nightly.yml`.

```text
job group          workflow     trigger                                              merge
gate               nightly.yml  schedule 10:17 UTC; workflow_dispatch; push to        (decides the night)
                                scratch/**-nightly
parity-long(-*)    nightly.yml  the same, when the gate runs the night               reports
stability-*        nightly.yml  the same                                             reports
clients-*          nightly.yml  the same                                             reports
repeat-*           nightly.yml  the same                                             reports
report             nightly.yml  the same; if: always()                               reports; opens and
                                                                                     closes the `nightly`
                                                                                     issue
```

- **Nothing in `nightly.yml` is a required check, and nothing in it runs on `pull_request`.** A red night
  reports; it never gates a PR. `parity-long` reports only and does not inherit `ac8`'s "blocks main's
  health" (13c QC-11); that is revisited after a month of nights.
- **`ci.yml`'s scratch route.** `test`, `test-windows`, `test-macos`, `platforms` and `python` are not
  run on a `scratch/*-nightly` branch, beside their `-image` and `-scenario` exclusions; `fast` still is.
  Nothing else in `ci.yml` changes.
- **`fast` gains four self-tests** (`ci_nightly.py`, `ci_godot.py`, `ci_repeat.py`, `ci_stability.py`
  `--self-test`, each under a second), and `check_ci_pins.py` gains one rule: every `uses:` that is not
  a local `./` path ends in `@<40 hex>`, a full commit SHA (13c QC-7). `core`'s commands are unchanged.
- **`I-S13-7` is amended for one job.** The workflow default stays `permissions: contents: read`. `gate`
  adds `actions: read`; `report` adds `actions: read` and `issues: write` (operator, 13c QC-1,
  2026-10-10). Both use the automatic `GITHUB_TOKEN`; no secret exists.

---

## ARC-83 — Layer 4: the nightly workflow, its verdicts and its baselines

**Date** 2026-10-10 · **Status** decided; live from S13 PR 13c · **Approved by** the operator (13c QC-1:
`issues: write` on the report job; QC-2) and the primary session (13c QC-3 … QC-12, design freeze
2026-10-10) · **Relates to** `ARC-48`, `ARC-49`, `DEP-17`, `DEP-19`, `DEP-26`, `DEP-45`,
[`ENGINEERING_STANDARDS.md`](ENGINEERING_STANDARDS.md) §16 · **Design**
`.structured-coding/plans/mvp0/pr-13c-nightly.md`

**Problem.** `ENGINEERING_STANDARDS.md` §16's fourth layer — "long-running stability tests … can run
separately from the fastest PR loop … simulate 100 days, … restart server repeatedly, replay event log" —
did not exist: no workflow had a schedule. Four further gaps had no owner. `AC-8` was compared at 300
days in memory and 30 days saved only. A behaviour change identical on every platform passed every
check, because the towns' long-run digests were recorded in prose only. The 25 `#[ignore]`d Godot tests
and the 3D slice probes never ran in CI. Nothing sampled a commit twice, so a flaky test was invisible
until it failed a PR.

**Options considered** (design §5). Scheduling: GitHub Actions `on.schedule` with a read-only `gh api`
query for the last nightly's commit (chosen); an `actions/cache` marker; a third-party "skip if
unchanged" action; running every night regardless. Reporting: one rolling issue written by `gh` with the
automatic token (chosen); a third-party issue action or `actions/github-script`; the step summary and
GitHub's failure e-mail only (the fallback); a webhook (needs a secret); a committed status file (needs
`contents: write` on a protected branch). Flake detection: repeating the unchanged default suite on the
night's commit (chosen); `cargo-nextest` retries (a retry turns a flaky failure green); GitHub's re-run
of failed jobs; a paid flaky-test service. The restart loop: a standard-library script driving the real
binary with the repository's own Python SDK as the client (chosen); a new Rust test (it would join the
default suite); the Godot client (heavier, and `run.sh reconnect` already covers it).

**Decision.**
1. **One workflow, `.github/workflows/nightly.yml`**, separate from `ci.yml`, so that no required job
   gains a trigger and the one write permission lives in its own file. Triggers: `schedule` at
   `17 10 * * *` (10:17 UTC, off the hour and outside the operator's working day); `workflow_dispatch`
   with inputs `force` (boolean, default true) and `groups` (default `all`); and pushes to
   `scratch/**-nightly`, the only way to exercise the workflow before it exists on `main`. Concurrency
   `nightly-<ref>`, never cancelled. The jobs name layers of `scripts/ci_layer.py`, never commands
   (`I-S13-9`), and reuse `ci.yml`'s composite actions and pinned actions.
2. **The gate** (`scripts/ci_nightly.py gate`, `actions: read`). A scheduled night runs iff `main`'s head
   differs from the `head_sha` of the last `nightly.yml` run on `main` that concluded `success` or
   `failure`; with no such run it runs. A dispatch on `main` with `force=false` decides the same way;
   `force=true`, a dispatch elsewhere and a scratch push always run. Any `gh api` error runs the night,
   with the error as the reason: a needless night costs minutes, a wrongly skipped one hides a defect.
   `groups` (`parity-long`, `stability`, `clients`, `repeat`) come from the dispatch input or, on a
   scratch push, from the token before `-nightly` in the branch name when it names a group; an unknown
   group is an error, never "none".
3. **Five groups.**
   - `parity-long`: `scripts/ci_parity.py record --profile long` on `AC-8`'s four legs (the runtime image
     on Linux x86_64 and Linux arm64, natively on `macos-26` and `windows-2025`). The long record keeps
     the default profile's keys and adds `summary-1000` (memory, seed 7, 1 000 days) and `summary-300s`
     with every stored byte of a 300-day save. `[source]` names `profile long`; `compare` refuses records
     of different profiles. The default record is unchanged byte for byte. Then the comparison, and the
     baseline check of point 5.
   - `stability`, on Linux (toolchain container), macOS and Windows: `scripts/ci_stability.py restarts`
     kills and restarts `mineworld server` ten times on one save while the SDK client joins with the
     invite and asks for the `perceived` stream from the cursor it reached (a restart drops every hold
     and resume secret, `server/PROTOCOL.md` §4.2). Each cycle's oracle is the server's own `GET /status`:
     the same `instance`, a `revision` not behind the last one seen before the kill, `faults` 0; then
     `mineworld replay` of the killed save. `scripts/ci_stability.py replay` saves every world 300 days and
     replays it. CA-13's ignored test (`perceived.rs`) runs here. Scratch must be left clean.
   - `clients`: the 25 `#[ignore]`d Godot tests (`client_2d`, `client_2d_interact`,
     `client_2d_interact_stub`, `client_settings`) on all three operating systems; on Linux and macOS
     also `clients-probes` — the 3D slice's `--drive`, `--world --link` and `--world --target`, whose
     verdict is parsed (they exit 0 on failure), the protocol module's live checks, and
     `ac13_semantic_parity` over the regenerated evidence, which is judged by that test, not by a byte
     diff (the evidence holds ports and timings). Windows runs `clients` only: the probe launchers are
     bash scripts, and `clients-probes` moves onto S23 R-c's portable launcher when it merges (follow-up
     owned by S13). A windowed test that cannot show a window on one leg may be skipped there by name
     only if it passes on another leg the same night, and a coverage check refuses a test run on no leg.
     The one exception (QC-5b, primary ruling 2026-10-10): a windowed test that fails on every hosted
     leg for a recorded display reason is named in `ci_godot.py` as not run on hosted runners, reported
     in each night's notes, and carried to the operator's milestone-F hand checks; the coverage check
     accepts exactly that list, and a third name turns it red. Two tests are on it today
     (`two_d_display_settings_take_effect`, `the_language_chosen_in_2d_is_in_the_3d_clients_first_frame`).
   - `repeat`: the unchanged default suite run twice more per OS (`scripts/ci_repeat.py`), every
     repetition run even after a failure. A test that failed in one sample and passed in another of the
     same commit and OS — the two repetitions and `main`'s own push run — is a flake candidate; a test
     that failed in every sample is a failure. A repetition that failed before any test ran is
     infrastructure, never a flake.
   - `report` (below).
4. **Verdicts are PASS only on positive evidence.** Each job writes one verdict — PASS, FAIL or
   INCONCLUSIVE — from its layer's own result file and parsed sub-verdicts. A failed command inside the
   layer is FAIL; a job cancelled or timed out, a setup step that failed before the layer ran, a missing
   summary line, or a missing verdict artifact is INCONCLUSIVE. INCONCLUSIVE is red. Nothing is retried to
   green.
5. **Committed baselines, `scripts/baselines.txt`.** For every world, the long record's `summary-300`,
   `summary-1000`, `summary-300s`, `facts-300s` and `journal-300s` must equal the committed values;
   snapshots and the manifest are excluded, because they are derived storage and a storage-format change
   must not read as a behaviour change (they stay covered by cross-platform parity). A world with no
   baseline, or a baseline for a world that no longer exists, is a named FAIL. **A PR that intends to
   change a world's behaviour updates the file in the same PR, with a `reason <PR or step id>: <one line>`
   entry for each world it re-baselines** (`ci_parity.py baseline write` keeps those lines). Any
   platform's long record serves, because parity holds. A forgotten update turns the next night red with
   the world, the key, the commit range and the first differing line. The file is code for
   `ci_changes.py`.
6. **The report** (`scripts/ci_nightly.py report`): a step summary with one row per expected job, `main`'s
   own push run for the commit, flake candidates, baseline drift, and a benchmark (wall time per
   world-day per leg, 300-day save sizes, a trend over the last 14 nights). On `main` a red night opens
   one issue labelled `nightly` or comments on the open one; a green night comments and closes it. A
   scratch run whose branch ends `-issue-nightly` uses the label `nightly-scratch` instead. The job fails
   iff the night is red, so the run is red in the Actions list.
7. **Permissions.** `contents: read` by default; `gate` adds `actions: read`; `report` alone adds
   `issues: write`. The automatic `GITHUB_TOKEN` only; no secret. No `pull_request` trigger, so no fork's
   code runs with a write token; every value from a branch name, an input or a job output reaches a
   script through `env:`, never `${{ }}` inside `run:`; every action is pinned by full SHA, and
   `check_ci_pins.py` now checks that.

**Why not ourselves / why not the others.** Scheduling, reporting and process control are GitHub's and
the standard library's; the scripts are the glue the existing CI already is. A third-party action would
hold a token for a few lines of `gh`. Retries are declined because they are the defect flake detection
exists to see.

**Isolating interface.** `nightly.yml`'s triggers and the gate; the layers of `ci_layer.py`; the verdict
files; the report's last step (the issue), which can be dropped without touching the rest; `baselines.txt`
with `ci_parity.py baseline`.

**Accepted limitations.**
- The 1 000-day **saved** run and the `ASR-1` 640 MiB save-size assertion wait for S6 save retention
  (13c QC-6); until then 300 days is the saved horizon (a 1 000-day save is about 8 GB).
- No "simulate many agents" scale test: no world with many agents exists; it belongs to a performance
  lane when one does.
- Windows has no `clients-probes` until a portable launcher exists (above).
- Rendering is not judged in CI (`ARC-17`); windowed Godot tests check behaviour, not frames.
- Hosted runners only, free on a public repository (GitHub's published terms; billing cannot be read by
  the session). If the repository became private, the night would cost about 1 800 Linux-equivalent
  minutes and must shrink to Linux-only and weekly.
- GitHub disables a schedule after 60 days without repository activity; `MVP_STATUS.md`'s S13 row names
  the nightly, so a silent stop shows as a stale date.

---

## DEP-45 — Godot in CI: the official 4.7.2 builds, SHA-512 pinned in the repository; Xvfb and Mesa for windowed tests

**Date** 2026-10-10 · **Status** selected; integrated in S13 PR 13c · **Approved by** the primary session
(13c QC-5, QC-10, design freeze 2026-10-10) · **Relates to** `ARC-83`, `ARC-17`, step-23 §16.4 · **Design**
`.structured-coding/plans/mvp0/pr-13c-nightly.md` §§3.4, 5.3

**Problem.** The nightly's `clients` group needs Godot 4.7.2 — the version the operator runs
(`4.7.2.stable.official.ed1daf0bf`) — on Linux, macOS and Windows runners, fetched reproducibly and
without trusting whatever a release page serves on the night.

**Options considered.**

```text
(a) the official 4.7.2-stable builds from godotengine/godot-builds, downloaded   chosen
    by our script, checked against SHA-512 values pinned in the repository,
    unpacked and cached (MIT)
(b) chickensoft-games/setup-godot (third-party action, MIT)                      declined: holds the
                                                                                 download logic we must
                                                                                 pin anyway, with .NET
                                                                                 options we do not need
(c) barichello/godot-ci container (MIT)                                          declined: Linux only, a
                                                                                 third party's cadence
(d) Godot built from source                                                      declined: hours of CI
(e) Xvfb and Mesa (lavapipe) from the Ubuntu archive for the windowed tests      chosen for Linux
```

**Choice.** (a) and (e). `scripts/ci_godot.py fetch` downloads `Godot_v4.7.2-stable_linux.x86_64.zip`,
`…_win64.exe.zip` or `…_macos.universal.zip` with the standard library, refuses any archive whose SHA-512
differs from the value pinned in the script (cross-checked once against the release's
`SHA512-SUMS.txt` when pinned; the release's own file is not trusted at run time, because it would only
detect corruption, not a replaced asset), unpacks it, and checks that `--version` starts with
`4.7.2.stable.official`. On Windows it uses the archive's `_console.exe`. `.github/actions/godot` caches
the unpacked build keyed on the version and the pinned hash, exports `GODOT`, and puts a `godot` command
on `PATH` for the bash launchers; on Linux it installs `xvfb` and `mesa-vulkan-drivers` from the
runner's Ubuntu mirror and starts a virtual display. One pinned Godot version exists in the repository:
S23's `export` job reuses this action and adds its export templates (step-23 §16.4: one download step,
owned by whichever lands first).

**Why not ourselves / why not the others.** The binary is Godot's own build; only the hundred lines that
fetch and verify it are ours, because any adopted action would still need our pin to be trustworthy.

**Isolating interface.** One constant block in `scripts/ci_godot.py` (version, three SHA-512 values) and
the composite action. The tests and launchers read only `GODOT` or `godot` on `PATH`.

**Revisit triggers.** A Godot upgrade (change the block, re-pin from `SHA512-SUMS.txt`); S23 needing
export templates; a hosted runner gaining a GPU.

**Accepted limitations.** Windowed tests on Linux run on a software Vulkan device under a virtual
display; they judge behaviour, never frames (`ARC-17`).

---

## ARC-71 — An Entity Pack in MVP-0 is a directory of item kinds a world requires

**Date** 2026-10-08 · **Approved by** the primary session at PR E-d's design freeze (step-16 §17.0;
FQ-d1 … FQ-d6) and the operator's every-platform requirement (§17.12) · **Implements**
[`MODULE_SPEC.md`](MODULE_SPEC.md) §2, §4.1, §8.1; [`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md) §5.0, §8 ·
**Relates to** `ARC-31`, `ARC-36`, `ARC-53`, `ARC-54`, `ARC-55`, `DEP-10` · **Design**
`.structured-coding/plans/mvp0/step-16-packages.md` §4.3, §17 (S16, PR E-d)

**Problem.** `MODULE_SPEC.md` §2 defines an Entity Pack — "what exists in the world" — and `ARC-53`
gave it an identity, but nothing read one: `pack.yaml` refused `type: entity-pack` and `requires:`
refused to resolve one. A kind of thing (bread, coffee) could exist only inside the one world that
declared it, so two worlds sharing goods had to copy the files. "Install modules → compose world" needs
a kind of thing to be installable on its own, with no rebuild.

**Choice.**

1. **Item kinds and nothing else, in MVP-0.** An Entity Pack is a directory holding `pack.yaml`
   (`type: entity-pack`, `ARC-53`'s fields, no new field) and `items/<key>.yaml`, each in the World Pack
   item-file format (`tags`, `note`, sections, `ARC-36`), read by the same code. A `places/`, `people/`
   or `organizations/` directory in it is refused by name ("an Entity Pack carries item kinds only in
   MVP-0"); one with no item file is refused ("declares nothing").
2. **The directory is the declaration** (FQ-d1). Every `*.yaml` in `items/` is one kind, keyed by the
   file's stem (`Path::file_stem` of an entry whose `Path::extension` is `yaml`), which must be an
   `EntityKey` or the file is refused naming it. Any other file in `items/` is not content and is left
   alone. `pack.yaml` has no `items:` list, so the key is stated once, by the file name.
3. **Who reads what.** `mineworld-packages` identifies an Entity Pack and checks its directory layout
   (file names only; it stays a leaf that knows no `EntityKey`); `mineworld-worldpack` reads its content
   in `worldpack/src/entities.rs`, because the item-file format and section decoding are its.
4. **Composition: read order step 6b.** After the world's own content is read and checked for
   undeclared files, each requirement that resolved to an Entity Pack, in id order, has its `items/`
   read **with the requiring world's enabled systems**, so a section is decoded by its owner exactly as
   if the world had written it, and the section rules (owner enabled, file may carry it, references
   declared and typed — `ARC-31`) apply unchanged. The kinds merge into the world's item kinds.
5. **One key namespace, refused by name** (`MODULE_SPEC.md` §4.1 rule 1). A key an Entity Pack declares
   that the world also declares — in `places`, `population`, `items` or `organizations` — or that another
   required Entity Pack declares, is refused naming the key and both sources. Never "last one wins":
   Minecraft data packs' override-by-load-order is exactly the silent shadowing refused here.
6. **Identity in key order across every source** (FQ-d2). Item kinds — the world's and every required
   pack's together — are allocated after people and before organizations, in key order, by the loop
   that already exists. A world that requires no Entity Pack allocates exactly what it did; moving kinds
   out of a world into an Entity Pack it then requires changes no id and no fact.
7. **Provenance names the pack.** A kind read from an Entity Pack carries `Metadata { source_pack: <the
   pack's id>, source_path: "items/<key>.yaml" }`, `source_path` written with `/` on every operating
   system so a save does not depend on the machine. Every refusal that names a content file names the
   file in the Entity Pack's directory. `Metadata` is free text already: no contract change.
8. **Self-contained** (FQ-d3). A section in an Entity Pack's item file may name only keys the same pack
   declares; any other key is refused naming the pack, the file and the key. In MVP-0 the only item
   section with references is `bodies`' object form (`at: { place }`), which describes one loose object,
   not a kind (`ARC-36` note), so it is refused in an Entity Pack, as intended.
9. **What the CLI shows** (FQ-d5). `mineworld packs validate <entity pack>` checks the identity, the
   licence policy and the framework range (the `ARC-54` note below), then reads every item file against
   **the build's whole installed set** — each section decoded by its owner; whether the owner is enabled
   is a world's question, answered at resolution — and rule 8, and prints the kinds. `packs list` lists
   `entity-pack` lines. `validate <world>` prints the composed item kinds (the world's and its packs') on
   its existing `items` line.
10. **Every platform.** CRLF line endings in `pack.yaml`, `items/*.yaml` and `world.yaml` read exactly
    as LF does, relying on `serde-saphyr` (`DEP-10`); `items/` is read into a map by key, so no file
    system's listing order can change an id; paths are built with `Path` operations, never by splitting
    text on `/`.

**Options considered.** An `items:` list in `pack.yaml` mirroring `world.yaml`'s (FQ-d1) — rejected:
the key would be stated twice. Pack kinds allocated after organizations (FQ-d2) — rejected: one order,
and moving kinds into a pack would then move ids. Namespaced keys (`goods:bread`, as Minecraft and
Factorio prototypes do) — rejected for MVP-0: `EntityKey` is a contract type and every reference would
change. A generic schema-driven content loader — rejected: a second statement of what each owner
already decodes with its own type.

**Accepted limitations.**
- Item kinds only: new entity types would extend the closed `EntityType` in `contracts`, and component
  schemas belong to System Packs in MVP-0 (`ARC-31` sections). Authoring templates are not read.
- No namespaced keys: two packs that both declare `tea` cannot be required by one world.
- No pack-to-pack dependency: `pack.yaml` still refuses `dependencies`, so an Entity Pack cannot build
  on another.
- A stray `.yaml` in `items/` becomes a kind; `packs validate` lists every kind so an author sees it.
- A world that adopts an Entity Pack shifts the ids of its later item kinds and organizations, as
  editing its own `items:` does today. Saves do not record which packs they used (QSE-14).

**Note, 2026-10-12 (S15, PR 12d; step-11 §19.13 TD-D12 item 3, the operator's ruling) — kinds carry a
display name, and `modern-goods` is 0.2.0.** `item`'s section now requires `name` (step-11 SD-D10,
R-PK-2; `ARC-37` note), so an Entity Pack's kinds state one like any world's. `modern-goods`' twelve
kinds gain theirs (the key in plain English, as Market Town's: Apple, Bread, …, Umbrella), and the
facts a world requiring the pack declares change, so its version moves **0.1.0 → 0.2.0**; a world
requires it as `"^0.2"`. Nothing else in the rules above changes. A third-party Entity Pack written
before this note is refused at load until each kind has a `name`, naming the file and line.

---

## ARC-54 note — a required data pack's own framework range is checked (F-Ed1, 2026-10-08)

Point 4's rule 2 gains a check `ARC-54` left out: **a required data pack whose `mineworld:` range does
not admit the running framework is refused**, naming the pack, its range and the framework's version —
after the version check, as the last check of that requirement. `mineworld packs validate` applies the
same check to every `pack.yaml` pack. Before S16's PR E-d a Presentation Pack stating `mineworld: "^9"`
passed both (finding F-Ed1, spike SC-7), although step-16 §4.2's refusals list "a framework outside
`mineworld:`" and `ARC-53` says a package fact is never ignored. A bounded correction of E-b's
merged behaviour, landed with the PR that first reads data packs' content; the two shipped Presentation
Packs state `^0.1` and are unaffected. A World Pack's own range is checked as before, by the reader.

---

## ARC-63 — The World's Interaction List: one section shape for every pack, typed and enforced by its owner

**Date** 2026-10-08 · **Approved by** the operator (`overall.md` "The World Interaction List": QIL-2
overruled, QIL-4 … QIL-6, QIL-9, QIL-11 … QIL-16) and the primary session at PR IL-b's design freeze
(step-18-interaction-list §12; QIB-1 … QIB-14 accepted as recommended) · **Implements**
[`MODULE_SPEC.md`](MODULE_SPEC.md) §4.2 · **Relates to** `ARC-25`, `ARC-31`, `ARC-33`, `ARC-34`,
`ARC-61`, `ARC-64`, `ARC-65`, `DEP-28`, `INV-13` · **Design**
`.structured-coding/plans/mvp0/step-18-interaction-list.md` §4.4 … §4.6, §4.10, §12 (S17, PR IL-b)

**Problem.** A world author wants to say, without code, that nobles and commoners do not talk, that a
conversation lasts longer in this world, that an heirloom cannot be given. Every such rule today is a
compiled constant or a hard-coded branch inside one System Pack (step-18 §2.2). `ARC-61` gave every pack
a world-level file it types; it did not say what that file looks like. If each pack invented its own
shape, an author would learn twelve dialects, a tool could print none of them, and the first rule that
needed a class of person would be written twelve times.

**Choice.**

1. **One shape for every pack.** A pack's part of the list is its **section**: the file
   `configure/<pack id>.yaml`, carried by `ARC-61`'s seam unchanged. Every section has the same six
   optional keys and refuses any other (`deny_unknown_fields` throughout):

   ```yaml
   extends: default            # one of this pack's reference lists (item 4)
   default: permit             # permit | forbid: what an action no rule matches gets; permit unless said
   rules:                      # { action, <role>: <selector>…, effect: permit | forbid }
     - { action: talk, actor: noble, target: commoner, effect: forbid }
   parameters:                 # { <role>: <selector>…, <field>: <value>… }; unscoped = the base
     - { gap: 600 }
     - { actor: guard, gap: 1200 }
   consequences:               # { fact, <role>: <selector>…, audience?, biography?, <knob>… }
     - { fact: spoke, actor: servant, biography: off }
   regions:                    # by place key: rules, parameters and consequences for that place only
     library: { parameters: [ { gap: 60 } ] }
   ```

   A **selector** is an entity class (`ARC-64`), a type's implicit class, or `*`. A **role** is a position
   the owning pack declares per action and per fact: `actor`, `target`, `object`, `place`.
2. **The SDK supplies the shape; the pack supplies the meaning.** A pack implements
   `mineworld_sdk::interactions::InteractionSection` and writes `mineworld_sdk::interactions!();` inside
   its `impl SystemPack`:

   ```text
   type Parameters      made by parameters!: each field's type, bound (L0) and default, with an
                        all-optional partial twin for scoped entries
   type Knobs           pack-specific consequence fields (`()` for none)
   ACTIONS              each action: its roles, and whether a region may scope it
   FACTS                each fact type: which envelope position fills each role, its default audience,
                        the narrowest audience a list may choose, whether biography is configurable
   PARAMETER_ROLES      the roles a parameter entry may scope by
   CONFIGURED, COMPONENT   the names of its configured fact and per-place component
   reference_lists()    the compiled lists; always `default` (today's behaviour)
   encode, decode       the configured fact's payload, with the pack's own codec
   ```

   `interactions!()` makes the section the pack's configuration (`ARC-61`'s `CONFIGURATION`,
   `CONFIGURATION_FACTS`, `decode_configuration`), so a pack with a section has no other configuration,
   and states `INTERACTIONS` for the installed set's `Capability::interaction_section()`, which the tools
   read. The pack's `declaration()` and `install()` call `interactions::declare` and
   `interactions::install`, and its `react` calls `interactions::reduce` first.
3. **What decoding refuses, at its line and column** (the section's own type, `DEP-10`): an action or fact
   the pack does not declare; a role the pack does not declare for that action, fact or parameter
   block; a parameter outside its bound or unknown; a widened audience or one below the owner's
   narrowest (`ARC-65`); `biography` on a fact whose owner does not allow it; a rule in a region for an
   action that is not regional; an `extends` naming no reference list of the pack, or one whose chain
   is cyclic or longer than four. After decoding, once the world's classes are read, the loader refuses
   by name, naming the file, the list and the index of each entry: a selector naming a class that is
   neither declared nor implicit (`ClassUndefined`), and two **ambiguous** entries (`AmbiguousEntries`,
   item 5). A region naming no declared place is `ARC-61`'s unknown-entity refusal.
4. **Levels, highest last** (`L0` is never overridden):

   ```text
   L0  the pack's bounds         code; a value outside them is refused at load
   L1  the pack's `default`      compiled; today's behaviour
   L2  extends                   a named reference list of this pack (chains of at most four, no cycle)
   L3  the world's section       configure/<pack>.yaml
   L4  a region                  that section's regions.<place> entries, for that place only
   ```

   A higher level's entry replaces a lower level's entry with the same key — the action or fact plus its
   selectors — field by field for parameters and consequences; entries with different selectors
   coexist.
5. **Then specificity, then forbid.** Among the entries that apply to a request, the one naming the most
   roles wins (an implicit class counts as named). Among rules of equal specificity, `forbid` overrides
   `permit` (Cedar's rule, `DEP-28`). Two parameter entries or two consequence entries of equal
   specificity that **overlap** and give one field different values are refused at load: so ambiguity
   never reaches run time, and a lookup's answer does not depend on the order an author wrote entries
   in. Two entries overlap when, for every role, their selectors are equal, or one is `*`, or one is an
   implicit class whose type the other's class selects.
6. **Storage.** A configured section is resolved at genesis, by a pure function of the files, into one
   genesis fact `<pack>-interactions-configured`, `Visibility::SystemInternal`, no subjects, holding
   every level merged, each region, and the classes the section references. The pack reduces it into a
   component `<pack>-interactions` on every Place, holding the base and that place's region. The drift
   check (`ARC-61` item 7) compares that fact, so an edited section, an edited referenced class or a
   changed attachment is refused at resume.
7. **Lookups are pure and total.** `permits` (`Err(PermissionDenied)` when forbidden), `parameters`
   and `consequence` read the component on the place the pack names (a request's actor's place, a
   reaction's fact's place; without a place, the base, which every copy holds). With no component —
   the world configures nothing for that pack — each returns the compiled default and reads nothing
   else: `Ok`, the default parameters, the owner's default audience and compiled biographical flag. So
   a world that configures nothing produces byte-identical facts, pack by pack.
8. **Enforcement stays in the owning pack.** In `validate`, `permits` after the payload, actor and
   target exist and before the spatial requirement; in its offers through the same call, as
   `Offer::refused(PermissionDenied)` (`ARC-34` note); `consequence` at emission; `parameters` where it
   decides. The kernel sees nothing new.
9. **A list cannot grant.** It names only declared actions and facts; `permit` is a filter AND-ed with
   the pack's own validation; parameters stay within bounds; audience only narrows and biography changes
   only where the owner allows (`ARC-65`); a section for a pack the world does not enable is refused
   (`ARC-61`).
10. **`mineworld interactions <world> [--place KEY] [--json]`** prints each configured section resolved
   — base, then each region — each entity's class, and "default (compiled)" for a pack with a section
   that the world does not configure. It reads the World Pack only and writes nothing. Its JSON has
   sorted keys and is stable.

**Options considered.** (a) Each pack's own file format — rejected: twelve dialects, no tool. (b) A
general policy engine — rejected (`DEP-28`). (c) **One SDK shape, the meaning per pack** — chosen.

**Accepted limitations.**
- No attribute conditions (`when regard > 50`, QIL-11): data only, no expressions (QPL-1).
- A reference list is written in Rust by its pack, not in YAML: neither the SDK nor a pack depends on a
  YAML parser, and the loader's parser is not handed to them.
- A pack with a section has no other configuration file; a pack that needs a `data:` attachment
  (`ARC-61` note) and a section at once is a later amendment, when one exists.
- Converting a pack raises its `SystemVersion` (its declaration grows), so older saves are refused by
  name (`ARC-25`); no fact digest moves. A pack's `default` is pinned to its version by one test per
  pack.
- In IL-b only `conversation` and `group-activity` have sections, and they declare parameters only.
  Rules and consequences of real packs arrive with IL-e … IL-g.

**Note, 2026-10-10 (S17, PR IL-e; QIE-1, QIE-2, SD-IE-2, SD-IE-8) — the first rules of real packs, and a
rule decided in a reaction.** `conversation` declares the action `talk`; `group-activity` declares
`invite`, `accept-invitation` and `join-group-activity` (never `decline-invitation` or
`leave-group-activity`: a list can stop people coming together, not trap anyone in an invitation or an
activity; `accept-invitation` is governed because accepting joins the inviter's activity, so a
`join-group-activity` rule alone would leak). Each pack asks `permits` in `validate` after the payload is
readable, the actor and target exist and are Persons and the actor is located, and before any condition
that depends on the pack's own state (an open invitation, `Busy`) and before the spatial requirement; its
offers ask the same `permits` and answer `Offer::refused(PermissionDenied)` (item 8).

Item 1 says a rule is "an action and role selectors". A pack may also declare a **rule name for a yes/no
decision it makes in a reaction**: `relationships` declares `acquaint` (roles `actor`, the person who
would come to know, and `target`, the counterpart; not regional), asked per directed change while it
reduces another pack's fact, before any state is written; forbidden, that direction forms no
acquaintance, no `knows` edge and no fact. The name is declared through the same `ActionDecl` (the SDK
keys rules by `ActionTypeId`); it **names no action type a client can send**, and a test pins that no
installed pack provides an action type `acquaint`. No SDK type is added.

**Note, 2026-10-13 (S24, PR FG-a; QFGA-2, D-FGA-8) — four additions to the shape.** Item 2's "every
field with a bound" is generalised and three seams are added, each generic and none naming a pack
(`MODULE_SPEC.md` §4.2):

1. **A bound is a value.** `parameters!` declares each field as `name: Type = default, bound;`, where the
   bound implements `ParameterBound<Type>` (`admits`, `describe`). `RangeInclusive` implements it with
   today's message byte for byte, so every existing section decodes and refuses unchanged; a pack may
   supply a named-choice bound (`relationships`' curve catalog).
2. **Per-field roles.** `InteractionSection::FIELD_ROLES` lists, per field, the subset of
   `PARAMETER_ROLES` that may scope an entry setting it (unlisted: all of them). The decoder refuses, at
   the entry's line and column, an entry that sets such a field under another role, and a region entry
   that sets a field whose roles exclude `place`.
3. **A place-free lookup**, `parameters_unplaced`: the base resolution with the place role unfilled, or
   the compiled defaults reading nothing when unconfigured. It is correct for exactly the fields item 2
   makes unplaceable.
4. **A section's required systems.** `InteractionSection::requires(section)` (default none) is what the
   held section answers to the loader's existing requires check (`ARC-61`), so a section can require
   the packs whose code it selects.

New fields grow a pack's configured fact, so the pack's version is raised (`relationships` 3,
`conversation` 4), as item "Converting a pack raises its `SystemVersion`" already states.

---

## ARC-64 — Entity classes are named tag selectors, fixed during play

**Date** 2026-10-08 · **Approved by** the operator (`overall.md` "The World Interaction List": QIL-3,
QIL-6, QIL-7; QIL-2 overruled, so classes live in `configure/classes.yaml`) and the primary session at PR
IL-b's freeze (QIB-8) · **Implements** [`MODULE_SPEC.md`](MODULE_SPEC.md) §4.2 · **Relates to**
`ARC-36`, `ARC-61`, `ARC-63`, `A-1` · **Design** `.structured-coding/plans/mvp0/step-18-interaction-list.md`
§4.2, §12 SD-IB-4

**Problem.** A rule about "nobles" needs to know who is a noble, in every pack, without a new owner of
state and without changing a world that never mentions nobles.

**Choice.**

1. **A class names a tag over one entity type.** `configure/classes.yaml` is a list of
   `{ class, of, tag }`: `class` has the system-id grammar (`authoring::ClassName`); `of` is `person`,
   `place`, `item` or `organization`; `tag` is a `Tag`. It is decoded by
   `mineworld_authoring::EntityClasses`.
2. **One class per entity, by priority.** An entity's class is the first entry whose `of` is its type
   and whose tag it carries; otherwise its type's **implicit class**, named by the type. Every entity
   also matches the selector of its type's implicit class. The four implicit names are reserved.
3. **Classes are not state.** Tags are a taxonomy fixed after genesis (`A-1`), so a class needs no fact,
   no component and no owner, and does not change while a world runs. `classes.yaml` is not seeded on
   its own: each section's resolved fact copies the class entries it references, and the entries of the
   same type listed before one of them (which could shadow it). So editing a referenced class is drift
   (`ARC-61` item 7), and editing an entry no section can see is not.
4. **`classes` is a framework key of `configure:`**, not a System Pack id: listed to be read, read
   before any section is decoded, refused as `ClassesInvalid` (its line and column) when a class is
   defined twice, an implicit name is reused, or `of` is not one of the four types. An unlisted
   `configure/classes.yaml` is refused as any undeclared file is.

**Options considered.** (a) A `classes` pack owning a mutable `Class` component and facts —
rejected for MVP-0 (QIL-7): an owner, facts and a migration for what tags already say. (b) **Named tag
selectors** — chosen: byte-identical when unused, already disclosed to clients, already immutable.

**Accepted limitations.** One class per entity and no inheritance (QIL-6); no promotion during play
(QIL-7); a tag no class names has no effect, and `mineworld interactions` prints each entity's class so
an author sees which applies (R-IL-7).

---

## ARC-65 — Consequence routing: a list narrows a fact's audience and switches its biography, within the owner's bounds

**Date** 2026-10-08 · **Approved by** the operator (`overall.md` "The World Interaction List": QIL-8,
QIL-10, QIL-15) and the primary session at PR IL-b's freeze (QIB-11, QIB-12) · **Implements**
[`MODULE_SPEC.md`](MODULE_SPEC.md) §4.2 · **Relates to** `INV-4`, `INV-11`, `INV-13`, `ARC-29`, `ARC-63`
· **Design** `.structured-coding/plans/mvp0/step-18-interaction-list.md` §4.7, §12 SD-IB-11, SD-IB-13

**Problem.** "Servants' lines are not history; nobody overhears in this world." A world must be able to
say what a fact means for history and perception without ever stopping a fact from being recorded,
leaking private state, or telling a mind what to forget.

**Choice.**

1. **What a list may govern, per fact type and role selectors:** its **audience**, narrowing along
   `Public ⊇ Place ⊇ Participants`, never below the owner's declared narrowest and never wider than its
   default (a widening is refused at load); its **biography** flag, `on` or `off`, only where the owner's
   `FactDecl` allows; and the pack's own typed knobs. `Entities(…)` and `SystemInternal` are never
   produced by a list: a list cannot name entities.
2. **What it may never do:** widen an audience; stop a fact from being recorded (`INV-11`); change a
   payload, an owner or a fact's subjects; route another pack's fact. A fact stated through another pack's
   constructor takes its consequence from its owner's section.
3. **"Enters history", layer by layer:**

   ```text
   fact log     always: every fact is recorded in every world (INV-11)
   biography    the ARC-29 projection asks the save's configured sections (item 4)
   in-world     the owning pack's own state (conversation's Remembered), through that pack's knobs
   memory       cognition's, from perceived facts only: the list reaches it only through audience
   ```

   No list writes into a mind; a world cannot tell cognition what to forget (QIL-10).
4. **`ARC-29` amended.** The biography projection selects through
   `mineworld_sdk::interactions::biography::selected(fact, person, compiled, configured)`. `Configured`
   is assembled from the save's genesis `*-interactions-configured` facts, each composed capability's
   `FactDecl`s, and each entity's type and tags. With nothing configured it is `ARC-29` exactly.
5. **The audience is chosen by the owner at emission**, through `consequence`, and carried in the
   envelope's `Visibility`; perception reads only the envelope, so it applies a list's narrowing with no
   change and no knowledge of lists.

**Accepted limitations.** In IL-b no installed pack declares a configurable fact: the routing is proven
with a test-only pack, and through the binary when IL-e converts conversation (QIB-11). Perceiving a
narrowed fact as a bystander is proven by S11-C's audience function and `mineworld perceived` when they
exist (QIB-12); IL-b proves the envelope's `Visibility`.

**Note, 2026-10-10 (S17, PR IL-e; QIE-3, QIE-5, QIE-6, QIE-10 … QIE-12) — the first configurable facts of
installed packs.** `conversation` declares `spoke` (owner default `Place`, narrowest `Participants`) and
`conversation-started` (`Participants`), both with configurable biography, compiled off, and the knob
`remember` (item 3's "in-world" layer: whether the listener's `ConversationHistory` keeps the line; it
never stops the fact being recorded, perceived or reduced by another pack). `group-activity` declares its
seven facts — the three invitation facts `Participants`, compiled non-biographical; `group-activity-started`
and `-ended` by `place` only, since their members list has no fixed positions; `joined-` and
`left-group-activity` by `actor` and `place` — each narrowable to `Participants`, biography configurable.
`relationships` declares `became-acquainted` and `relationship-changed` (already `Participants`;
biography configurable, compiled on). Each owner writes its default audience once, in its `FactDecl`, and
passes it as `owner_default` at emission.

The accepted limitations above are closed: the biography projection through the binary (QIB-11) and a
narrowed `spoke` not perceived by a bystander through `audience::admits` and `mineworld perceived`
(QIB-12) are proven by IL-e's tests. **Biography is chosen per fact, not per reader** (QIE-6, accepted for
MVP-0): `selected` fills the roles from the envelope, so its answer is the same for every person the fact
names — `{ fact: spoke, actor: servant, biography: off }` removes a servant's line from the servant's and
the listener's biography alike. Per-reader selection would change the SDK and this record. No line is
biographical by default in MVP-0 (QIE-12): a world that wants lines in biographies writes
`{ fact: spoke, biography: on }` first.

---

## DEP-28 — The Interaction List: our own narrow schema, on Cedar's semantics; no policy engine

**Date** 2026-10-08 · **Approved by** the operator (`overall.md` "The World Interaction List", decision
numbers) and the primary session at PR IL-b's freeze · **Relates to** `REUSE_POLICY.md` §§2, 11–12, 17,
`ARC-63`, `QPL-1` · **Design** `.structured-coding/plans/mvp0/step-18-interaction-list.md` §3

**Problem.** The list decides three things, deterministically, on the hottest paths (`validate`,
offers, emission): is a (role classes, action, place) tuple allowed; which value of a typed parameter
applies; which audience and biographical flag a fact gets.

**Options considered** (step-18 §3.2, primary sources):

```text
flecs relationships        storage and queries, not rules; an ECS is DEP-1's rejection    reference
RimWorld / Factorio        data names code; ordered composition; untyped, game-bound      reference
Cedar                      exactly the rule semantics (scope triple, forbid overrides     adopt the semantics,
                           permit, schema validation); nothing for parameters or          not the engine
                           consequences; conditions are an expression language; default
                           deny; an entity set per request; "skip on error"
OPA / Rego                 general language; time, randomness and network builtins        reject
Casbin                     scripted matchers (rhai), async runtime, untyped model files   reject
interaction matrices       the presentation authors expect                                reference
our own SDK schema         all three columns, integers, sorted data, pure lookups         build, narrowly
```

**Choice: build, narrowly, with Cedar's semantics.** Our rule grammar is Cedar's scope triple without
`when`; our tie rule is forbid-overrides-permit; our sections are validated by their owner's type before
anything runs. Unlike Cedar, a world's default is `permit` (today's behaviour) and an erroring entry is
refused at load, never skipped. No dependency is added; YAML is `serde-saphyr` (`DEP-10`), as before.

**Revisit trigger.** Attribute conditions becoming a requirement (QIL-11): Cedar's engine is then the
candidate, not a language of our own.

---

## ARC-56 — Cognition is a client: a Python process joins a seat over the public protocol

**Date** 2026-10-08 · **Status** accepted; implemented first by S10 PR P3 (`mineworld-sdk`) · **Approved
by** the primary session at P3's design freeze (step-17 S10 freeze; P3 §11.1) · **Relates to** `ARC-41`,
`ARC-34`, `INV-1`, `INV-9`, `INV-13`, effort decision `D-4` · **Design**
`.structured-coding/plans/mvp0/step-17-cognition.md` §§3.2, 4.6;
`.structured-coding/plans/mvp0/pr-s10-p3-python-sdk.md` · **Placeholder in the step design**
`ARC-S10-a` (S10's range `ARC-56 … ARC-60`, `overall.md` ruling 6)

**Problem.** Cognition is Python (`ENGINEERING_STANDARDS.md` §3) and the world is Rust, and the server
must never block on a model (`ARCHITECTURE.md` §9). Something carries what a seat perceives out to the
cognition process and its requests back in.

**Options considered** (step-17 §4.6, with licences and maturity):

```text
(a) an LM controller in Rust, inside the server   a model stall shares the world's process; Rust's model
                                                  clients are thinner; against §3's Python rule
(b) Python embedded in the server (PyO3)          a cognition crash or GIL stall is a world crash or
                                                  stall; the server crate would depend on a controller
(c) a Python sidecar behind a Rust driver          a second protocol (gRPC or stdio JSON-RPC) beside the
    (gRPC or stdio JSON-RPC)                       client one; one controller split over two processes
(d) a Python process that is a client of the      chosen
    public protocol, one WebSocket per seat
```

**Choice: (d).** A model-driven Person is a client. Its process joins a seat with the invite and a
nickname, receives exactly what that seat is entitled to, and sends the same three frames a 2D or 3D
client sends (`join`, `submit`, `leave`). The Python side is `sdk/python` (`mineworld-sdk`): typed
mirrors of the revision-2 frames, held to `server/tests/frames/`, and one seat session.

- **`INV-9` and `INV-1` by construction.** The server cannot tell a model from a person, and has no path
  a model could use that a player lacks.
- **One boundary.** The JSON client protocol is already proven from Godot; Python is its next consumer,
  checked by the same golden frames (`ARC-41`). gRPC is declined: a second protocol beside the client
  one, with `.proto` mirrors of every contract, for an encoding gain the protocol's rates do not need.
- **The server never waits.** A model call happens in another process.

**Consequences.** A seat's cognition has no more privilege than a player. Whatever cognition needs from
the world (reliable perceived events, a resumable cursor, seat exclusivity) is a protocol feature every
client gets (step-17 §11, S11-B, S11-C). A Python SDK names no System Pack's vocabulary
(`mineworld-sdk` builds requests generically from what an observation offers); pack-specific request
builders live with the controller that uses them (P6).

**Revisit** if a measured latency or bandwidth need cannot be met by the JSON protocol — the same
trigger as `ARC-41`'s encoding choice.

---

## DEP-24 — Pydantic for the Python wire models

**Date** 2026-10-08 · **Status** adopted by S10 PR P3 (`pydantic>=2.13,<3`; locked 2.14.0) · **Approved
by** the primary session at P3's freeze (QP3-7) · **Licence** MIT · **Relates to** `ARC-56`, `ARC-41`
· **Design** step-17 §4.2, §4.7 ("Wire models"); P3 D-P3-3, D-P3-7 · **Placeholder** `DEP-S10-b`

**Problem.** The Python SDK holds every revision-2 frame as a typed value, decodes what the server
sends strictly (unknown fields refused, ids only as decimal strings, integers never floats, closed
enumerations, the contract's cross-field rules), and re-encodes to exactly what the Rust types write.
Later S10 PRs need JSON Schema generation for structured model output from the same library.

**Options considered** (`REUSE_POLICY.md` §§11–12):

```text
(a) Pydantic 2 (MIT; the ecosystem standard; pyright understands its models)
(b) msgspec (BSD-3-Clause; faster)
(c) dataclasses plus our own validation
```

**Choice: (a).** Strict models with discriminated unions; validation is local and deterministic.
- **(b) declined:** speed is not a constraint at 10 Hz per seat, and it would be a second schema library
  once P6 needs Pydantic's JSON Schema for structured output.
- **(c) declined:** re-implementing strict decoding, discriminated unions and schema generation is
  exactly the wheel `REUSE_POLICY.md` §12 forbids rebuilding.

**Isolating interface.** `mineworld_sdk.wire`: callers name the SDK's model types and `codec.encode` /
`codec.decode`; how a model validates is not part of the SDK's interface.

**Revisit** if decoding cost is measured to matter, or Pydantic 3 changes the model API.

---

## DEP-25 — `websockets` for the Python client's WebSocket

**Date** 2026-10-08 · **Status** adopted by S10 PR P3 (`websockets>=17,<18`; locked 17.2) · **Approved
by** the primary session at P3's freeze · **Licence** BSD-3-Clause · **Relates to** `ARC-56` · **Design**
step-17 §4.6, §4.7 ("WebSocket client"); P3 R-P3-6 · **Placeholder** `DEP-S10-c`

**Problem.** One asyncio WebSocket connection per seat, text frames only, to a local or remote server.

**Options considered:**

```text
(a) websockets (BSD-3-Clause; asyncio-native; maintained; requires Python >= 3.10)
(b) aiohttp (a whole HTTP client and server framework)
(c) httpx-ws (not verified)
(d) our own client over asyncio streams (RFC 6455 framing, masking, close handshake)
```

**Choice: (a)**, its `websockets.asyncio.client` implementation only, pinned to one major version
because the library has replaced its asyncio API before (R-P3-6).
- **(b) declined:** a framework for one socket.
- **(c) declined:** maturity not verified.
- **(d) declined:** commodity infrastructure (`REUSE_POLICY.md` §12).

**Isolating interface.** `mineworld_sdk.session.SeatSession` is the only module that imports
`websockets`; nothing else in the SDK, and no caller, names a `websockets` type.

**Revisit** at the next major version, or if a transport other than WebSocket is added.

---

## DEP-26 — The Python toolchain: uv, ruff, pyright, pytest and pytest-socket

**Date** 2026-10-08 · **Status** adopted by S10 PR P3 · **Approved by** the primary session at P3's
freeze (QP3-3, QP3-4, QP3-6) and the operator's platform requirement of 2026-10-08 · **Relates to**
`DEP-17`, `DEP-18`, `ARC-48` · **Design** step-17 §4.7; P3 D-P3-4, D-P3-10, D-P3-11, R-P3-1, R-P3-2,
R-P3-9, R-P3-10 · **Placeholder** `DEP-S10-e`

**Problem.** Python enters a Rust repository. Its environments must be reproducible from one lock on
Linux, macOS and Windows; its code must be linted and type-checked strictly from the first module; its
suite must run with no network but localhost (`I-11`); and CI must run all of it without slowing the
required Rust checks.

**Choices, each with its alternatives:**

| Piece | Adopted (licence; version locked at P3) | Declined, and why |
| --- | --- | --- |
| Environments, locking, running | **uv** (MIT OR Apache-2.0, verified on PyPI 2026-10-08; uv 0.12) with a **virtual workspace root** (`pyproject.toml` with only `[tool.uv.workspace]`) and one universal `uv.lock`; its build backend `uv_build` for the SDK package | pip + venv + pip-tools: two tools and no workspace; Poetry: heavier, slower, no universal lock across members. A lock per package: two resolutions of shared dependencies once P6 adds a member (QP3-4) |
| Lint and format | **ruff** (MIT; 0.16) | flake8 + isort + black: three tools doing what one does |
| Types | **pyright** (MIT; 1.1.414) from PyPI with its **`nodejs` extra**, which brings Node.js as a locked wheel (`nodejs-wheel-binaries`, MIT) for every platform | mypy: weaker inference on Pydantic and asyncio code, and the step's design names pyright; basedpyright: equivalent, kept as R-P3-1's fallback; Node.js in the CI image: a second toolchain to pin |
| Tests | **pytest** (MIT; 9.1) | unittest: no fixtures or markers to run the real server once per test and deselect it by name |
| Network guard | **pytest-socket** (MIT; 0.8.1), `--allow-hosts=127.0.0.1,::1` on every platform | our own socket patching in `conftest.py`: re-implementing a small, purpose-built library (QP3-6) |
| CI, non-Linux legs | **`astral-sh/setup-uv`**, pinned by commit SHA in `ci.yml` (recorded with the job) | installing uv by a shell script fetched at run time: not pinned |

**Verified at P3 (C1), not assumed:**
- uv accepts a virtual root, and `uv sync --locked` builds the workspace from it.
- `uv.lock` holds `win_amd64`, `macosx_*_arm64` and `manylinux*_x86_64` wheels for every compiled
  dependency (`pydantic-core`, `websockets`, `nodejs-wheel-binaries`).
- The pyright wrapper prefers a `node` already on `PATH` and, unless told otherwise, asks PyPI whether it
  is the newest release on every run. CI therefore runs it with `PYRIGHT_PYTHON_GLOBAL_NODE=0` (the locked
  Node wheel, not whatever Node the runner has) and `PYRIGHT_PYTHON_IGNORE_WARNINGS=1` (no version query).
- pytest-socket blocks `connect` to any host but the allowed ones; with `--allow-hosts` given it ignores
  `--disable-socket`, so asyncio's self-pipe works on every platform with the same flags.

**Isolating interface.** The commands live in `.structured-coding/standards.md` and in
`scripts/ci_layer.py`; every one runs `uv run --locked`, so the lock is the whole resolution.

**Revisit** if uv's licence changes, if pyright's wrapper stops shipping its JavaScript, or when a
second workspace member needs a different Python floor.

---

## ARC-67 — Two time domains: `WorldTime` is calendar time; embodied time is not a world quantity

**Date** 2026-10-08 · **Approved by** the operator (requirements of 2026-10-08, rulings QTW-1, QTW-2,
QTW-5, QTW-9, QTW-13) and the primary session at PR TW-a's design freeze · **Implements**
[`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §17 · **Relates to** `ARC-26`, `ARC-27`, `ARC-28`, `ARC-61`,
`ARC-68` (calendar and weather packs, TW-b), `ARC-69` (pause and scale, TW-c), `DEP-6`, `DEP-30` ·
**Design** `.structured-coding/plans/mvp0/step-19-time-weather.md` §4, §15.1 (S19)

**Problem.** The operator asked for a town whose day passes faster than a real day — a real 24 hours
mapped onto 4, 2 or 1 real hours (6×, 12×, 24×; 12× by default), changeable live, and pausable — and
fixed what the speed may and may not touch: it changes the rate of weather and of world events (the
calendar, day and night, the sun, schedules, shifts, wages), and it must **not** change how fast a
character walks, animates, falls or talks. The kernel already has one clock, `WorldTime`, and states
that "how fast a hosted world's seconds pass in real time is the host's decision" (`kernel/src/clock.rs`).
The question is how the two kinds of time relate without the kernel learning what a day is.

**Choice.**

1. **Calendar time is `WorldTime`**, unchanged: the world's one clock, whole simulated seconds from the
   epoch. Every fact, Process wake-up and `Observation.at` is stamped in it, and the calendar, the sun,
   weather, routines, shifts, wages and every calendar-driven Process are functions of it.
2. **Embodied time is not a world quantity.** It is real (wall-clock) time as a body experiences it:
   how fast a player walks, how long a line takes to type, how long an animation or a physics impulse
   plays. No fact, component, Process or rule carries it. It exists only as the cadence at which
   embodied inputs arrive at the server: a human's client sends requests at human speed, the host
   consults a hosted controller every *cadence* wall seconds, and a client animates at frame rate. A
   per-request rule (a stride, an impulse, a spoken line) is resolved at the instant its request
   arrives, so the scale cannot reach it.
3. **Time scale and paused are host pacing.** The time scale is world seconds per wall second while a
   hosted world runs (an integer ≥ 1); paused is the host state in which world seconds do not pass. The
   host turns wall time into the instant it advances the world to; the event queue fires what is due on
   the way. Neither is world state: no System Pack, controller decision or fact reads either
   (`INV-TW-2`). Their changes are host records in a host journal, never facts (`ARC-69`, TW-c).
4. **Consequence: hosted consult cadence is in wall seconds** (QTW-13, ruled by the operator; it reverses
   QS11B-4 for consult cadence). Headless `run` has no scale and no wall clock: it is unchanged, and its
   output for a world without the time packs is byte-identical.

**Alternatives rejected** (step-19 §4.2).
- *Two kernel clocks with a ratio:* the kernel would learn a day, every Process would have to say which
  clock it uses, and persistence would reconcile two timelines — kernel ignorance and change
  amplification both fail.
- *Each pack multiplies its durations by the scale:* packs would read host state; a rule that depends on
  the host is a determinism and ownership defect, and headless and hosted runs would diverge.
- *Embodied actions take world durations divided by the scale:* movement is per request and not
  modelled in world time (`ARC-26`); inventing durations only to cancel the scale is the coupling the
  operator asked to avoid.

**Accepted limitations.** World-time constants that a human answers in practice — conversation's
300 s gap, group-activity's 1 800 s invitation lifetime — become short in wall time at 12–24×. They stay
calendar time by this decision's rule; making them configurable content is QTW-15 (IL-b). FX-24
(step-19 §9.2) measures whether agendas remain feasible at 24×; its remedies are content, never a
controller that reads the scale.

---

## DEP-30 — Solar position: `solar-positioning` with `libm`, behind a private seam; civil dates built

**Date** 2026-10-08 · **Status** selected; one dependency added to `systems/calendar` only · **Approved
by** the primary session at PR TW-a's design freeze (SD-TW-a-1, SD-TW-a-6, SD-TW-a-7; §16.8 (a)) ·
**Relates to** `ARC-67`, `REUSE_POLICY.md` §§11–12 · **Design**
`.structured-coding/plans/mvp0/step-19-time-weather.md` §3.1, §5.4, §16 (S19, PR TW-a)

**Problem.** The `calendar` System Pack must say, for a configured latitude and longitude, where the
sun is at any instant and when the day's light events fall (astronomical and civil dawn, sunrise, solar
noon, sunset, civil and astronomical dusk), including polar days and nights. It must also turn an
instant into a civil date and weekday. Every result enters facts, so it must be the same integers on
every platform.

**Options considered** (read 2026-10-08; the full table is step-19 §3.1).

```text
solar position
  (a) solar-positioning 0.7.0 — MIT; NREL SPA and Grena3; azimuth, zenith, rise, set, transit, every
      twilight; numeric Julian-date API with no chrono and no heap; `libm` feature; pre-1.0
  (b) NOAA Solar Calculator equations (Meeus), ~100 lines of our own
  (c) NREL SPA C reference code — not freely redistributable
  (d) sunrise 3.0 — rise and set only, no elevation or azimuth; chrono
  (e) spa 0.5 — not NREL's SPA despite its name; stale
  (f) sun 0.3, astro 2.0 — low accuracy or abandoned
civil dates
  (g) chrono or time — mature, but heavier than the two functions the pack needs
  (h) H. Hinnant's public-domain integer algorithms days_from_civil / civil_from_days, ~25 lines
```

**Choice: (a) and (h).**
- `solar-positioning = { version = "=0.7.0", default-features = false, features = ["libm"] }`, an exact
  pin, in `systems/calendar/Cargo.toml` only. With `libm` and without `std` the crate's transcendental
  functions are the pure-Rust `libm`'s, not the platform's, so the same inputs give the same bits on
  every host. The lock gains `solar-positioning` alone (`libm` 0.2.16 is already locked).
- Civil dates are (h), in `systems/calendar/src/civil.rs`, citing their source.

**Why not the others.** (b) is the fallback, kept ready: it would be ours to maintain and test for no gain
in fit. (c) fails the licence rule. (d) cannot light a scene. (e) and (f) are less accurate, stale or
dead. (g) would add a dependency for two integer functions and a weekday; formatting a date is a
client's job.

**Isolating interface.** `systems/calendar/src/sun.rs` is the only file that names the crate:
`sun_at(…) -> SunSample` and `day_events(…) -> DayEvents`, both returning integers (millidegrees,
whole seconds; round half away from zero). If the pre-1.0 API churns or the crate is withdrawn, the same
two functions are re-implemented from (b) without touching anything else.

**Determinism and its guard.** Every output is quantized to integers before it reaches a fact, a
Process state or a disclosure. A test pins three exact `SunSample` integers (San Diego at 12:00 and
18:00 local on 2026-10-08, 78° N at winter-solstice noon). A change to any of them — a dependency bump,
a platform difference at a rounding boundary — fails the test and is a material stop, never a re-golden.

**Accepted limitations.** The library is pre-1.0 and its README warns the API may change: the exact pin
holds it. Delta T comes from the library's own estimate at the middle of the day's month, so it is fixed
per month, not observed. A day is searched for its first rise and first set of each horizon; at polar
latitudes on the few days with two crossings of one horizon, the second is not an event that day and
the next day's opening phase corrects the light.

---

## ARC-76 — Client settings are a shared, presentation-only client module

**Date** 2026-10-09 · **Approved by** the primary session at S20 PR SET-a's design freeze (2026-10-08;
`ARC-SET-a` of step-20 §11.1; number allocated by the primary session, 2026-10-09) · **Relates to** `ARC-46`, `ARC-47`, `ARC-70`, `DEP-35`, `DEP-36`,
[`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §§2–3, 10–12 · **Design**
`.structured-coding/plans/mvp0/step-20-client-settings.md` §3, §12; specification
[`clients/shared/SETTINGS.md`](../clients/shared/SETTINGS.md)

**Problem.** The operator requires an in-game settings menu in both reference clients: language (`en`
by default, `zh-Hans`), display (window mode, resolution, VSync, frame-rate cap) and a 12-hour or
24-hour clock, persisted per user, never reaching the server ("Framework, not demo" item 4). Two clients
that each built their own would drift; settings placed in the protocol module or on the server would
mix presentation with the world.

**Decided.**

1. Client settings are **one Godot module**, `clients/shared/settings/`, in a sibling project
   `clients/shared/` (the protocol module's pattern, `ADOPTION.md` §1). Each client takes it by symlink
   as `res://mineworld_settings`. No autoload, no required project setting, no import step.
2. The module is **presentation-only and has no network code**. A setting never becomes an
   `ActionIntent`, a frame or a request, and a client's frames are the same whatever its settings.
3. **One per-user `ConfigFile`**, `user://settings.cfg`, shared by both clients through the custom user
   directory `MineWorld`. It holds settings only: never an invite, an admin token, a seat, an address, a
   nickname or a world. Every scripted and capture mode runs on defaults (`--settings=none`).
4. **Host commands are not settings.** Pause and day length (S19 `/admin/clock`) live in a separate
   module, `clients/shared/host/` (SET-c), which contributes a tab to the menu and never reads or writes
   the settings file; the admin token is held in memory only.
5. The module has **no platform branch** beyond one declared case (a Wayland tooltip); per-platform
   behaviour is Godot's.

**Rejected.** Settings in the protocol module (it is the wire, owned by S11); settings as a server- or
world-side concept (they are one person's presentation); an autoload (a module a client adopts must not
change the project's global composition); a copy per client (two copies drift).

---

## DEP-35 — Client settings: Godot's `ConfigFile`, `DisplayServer` and `TranslationServer`; settings addons declined

**Date** 2026-10-09 · **Status** selected; no dependency added · **Approved by** the primary session at
S20 PR SET-a's design freeze (`DEP-SET-a` of step-20 §11.1; number allocated by the primary session, 2026-10-09) ·
**Relates to** `ARC-76`, `ARC-70`, [`REUSE_POLICY.md`](REUSE_POLICY.md) · **Design** step-20 §4.1–§4.3,
§4.5

**Options compared** (step-20 §4 holds the full tables and sources):

```text
persistence  Godot ConfigFile in user://                 adopt: typed values round-trip, readable,
                                                         per user; our own atomic write and validation
             ProjectSettings override.cfg                reject: per checkout, read once at start
             JSON through FileAccess                     reject: numbers come back as floats
             a Resource saved as .tres                   reject: a user-writable .tres can carry scripts
             OS-native preferences                       reject: native code, three back ends
display      DisplayServer + Engine.max_fps +            adopt: everything except changing the
             Viewport.scaling_3d_scale                   monitor's mode, which Godot does not expose
             project settings (display/window/*)         reject: read at start only
             native code to change the monitor's mode    reject (QSET-3): risky, per OS
menus        Maaack's Godot Menus / Game Template (MIT)  decline adoption, borrow the pattern
                                                         (PlayerConfig: a ConfigFile plus static apply
                                                         helpers): its scene and plugin flow does not
                                                         fit two code-built clients, and it has no
                                                         borderless/exclusive split, frame cap, render
                                                         scale or locale setting
             GGS — Godot Game Settings (MIT)             decline: editor-centric, saves resources, no
                                                         localisation
             our own small module                        build: about six short scripts over three
                                                         engine APIs, typed, with no editor step
text         ARC-70's TranslationServer + gettext .po    confirmed for both clients (CSV needs the
                                                         editor's importer; P-3)
```

**Credit.** The `ConfigFile` + static-apply pattern is Maaack's (`Maaack/Godot-Menus-Template`, MIT).

**Isolating interface.** `MineWorldSettingsStore` (`store.gd`) is the only code that reads or writes
the settings file; `MineWorldDisplay` (`display.gd`) is the only code that applies display settings.

---

## DEP-36 — Noto Sans SC Regular bundled as the clients' CJK fallback font

**Date** 2026-10-09 · **Status** selected; one font file added under the DEP-8 font exception ·
**Approved by** the operator (QSET-1, 2026-10-08) · **Relates to** `DEP-8` (its font exception and table
row), `ARC-55`, `ARC-76` · **Design** step-20 §3.7, §4.4

**Problem.** Godot's default font has no CJK glyphs. Without a bundled font, Chinese text renders only
through whatever font the operating system supplies — PingFang on macOS, Microsoft YaHei on Windows,
nothing at all (tofu boxes) on a minimal Linux — so the same build looks different per machine and a
capture cannot be compared.

**Choice.** `clients/shared/settings/fonts/NotoSansSC-Regular.otf`, the unmodified upstream file
(`notofonts/noto-cjk`, `Sans/SubsetOTF/SC/NotoSansSC-Regular.otf`, commit
`165c01b46ea533872e002e0785ff17e44f6d97d8`, 8,331,336 bytes, SHA-256
`faa6c9df652116dde789d351359f3d7e5d2285a2b2a1f04a2d7244df706d5ea9`), beside its `OFL.txt`, named in
`NOTICE`. It is loaded at runtime and appended to the **fallbacks** of the default theme font, so Latin
text keeps its current font glyph for glyph and only glyphs the default font lacks come from Noto Sans
SC, before the system is asked. A check proves every character of every shipped catalog is in the
bundled font with system fallback off (`clients/shared/checks/glyph_check.gd`).

**Options compared** (step-20 §4.4): the variable Noto Sans SC (twice the size for weights the UI does
not use); a GB 2312 subset (a build step, rare characters become tofu, and a Modified Version needs a
new decision); Source Han Sans SC (identical glyphs, no advantage); WenQuanYi Micro Hei through its
Apache-2.0 branch (the fallback had OFL been refused); Droid Sans Fallback (dated); the OS's own fonts
(rejected as the mechanism: different on every machine).

**Accepted cost.** 8.3 MB in Git history, once.

---

## ARC-75 — A walk is movement's state; its route is the geometry owner's answer; its strides are embodied requests

**Date** 2026-10-09 · **Approved by** the operator ("add pathfinding; 12d waits for it", 2026-10-08;
realistic defaults; the two time domains, QTW-13) and the primary session at PR 12n's design freeze
(step-11 §21.0: revision 1 with R-12n-1, QN-1 … QN-13, binding note N-1) · **Implements**
[`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §§6, 8–9, [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §17 ·
**Relates to** `ARC-25`, `ARC-26`, `ARC-27`, `ARC-34`, `ARC-39`, `ARC-62`, `ARC-67`, `DEP-34` ·
**Design** `.structured-coding/plans/mvp0/step-11-bodies.md` §21 (S15, PRs 12n-1 and 12n-2)

**Problem.** `move` is one stride toward a point and sees no walls (`ARC-26`). With walls and furniture
in the towns (`bodies`, `ARC-39`), a controller that strides straight at its target stops short at the
first table, and a café door behind an A-board becomes unreachable (step-11 §19, TD-D7). Somebody has to
go round things. Three rules bound where that lives: no client or controller may compute a world rule
(`ENGINEERING_RULES.md` §8; a client only reports intent, and 2D, 3D and a Python controller must share
the capability without duplicating it, §9); controllers are stateless (`ARC-27`); and embodied pace is
the wall-clock cadence of requests, never a world duration and never a pack reading the time scale
(`ARC-67`).

**Options considered** (step-11 §21.3).

```text
(a) a movement Process that strides once per      wakes in calendar time, so the time scale multiplies
    simulated second                              the walking speed (≈ 16 m/s at 12×); correcting it
                                                  in the pack would read the scale — both rejected by
                                                  ARC-67 (R-12n-1)
(b) a planning library every controller and       every client and controller embeds a planner (GDScript
    client embeds                                 2D and 3D, Python): duplicated logic; a stateless
                                                  controller re-plans every consult
(c) a server plan plus a client preview planner   two planners that can disagree
(d) the walk owned by bodies, or a third pack     walking would vanish without bodies, or the stride and
                                                  passage rule would be duplicated
(e) the route on the server, owned by movement    chosen
    and planned through a catalog the geometry
    owner answers; each stride an embodied request
```

**Choice: (e).**

1. **Two actions of `movement`.** `walk-to { to: Destination }` asks to go somewhere; `Destination` is
   `place(Location)` — a point in the walker's place or in a place reachable through a chain of
   passages, or a place without a position, meaning "enter it" — or `person(PersonId)` in the walker's
   own place. `Destination` is `#[non_exhaustive]` and serialized with one explicit tag per arm, so a
   later arm (a region, a remote place the travel system reaches, an object) is an addition, not a
   breaking change. `walk-step` (no payload) asks for the next stride of one's own walk. Both are offered
   once, against nobody, without a request, like `move` (`ARC-34`): a client or controller composes them.
2. **The walk is world state.** `walk-to`'s resolution plans the first leg and records it as the
   walker's `Walking` component — destination, the places still to enter, the current leg's waypoints,
   the last stride asked, stall and re-plan counts — and states `walk-started { person, destination }`.
   No stride, no `Process`, no world duration: a walk nobody steps waits, and takes no calendar time.
   `Walking` is persisted, restored and replayed like every component, so a resumed world continues the
   walk where it was.
3. **Each stride is a request.** `walk-step` takes at most `WALK_STRIDE` (1 340 mm) toward the next
   waypoint and never past it, or, at a doorway, the crossing into the next place, after which the next
   leg is planned. The stride is checked by the same rule as `move` and stated through presence's
   `arrivals()`, so the registered resolvers resolve it like any other arrival (`ARC-39`). The walk ends
   with `walk-ended { person, destination, outcome }`, outcome `arrived`, `stalled` (three consecutive
   steps with under 50 mm of progress, or more than eight re-plans), `no-route` (a later leg or a
   re-plan found no way), `replaced` (a new `walk-to`) or `stopped` (the walker's own `move` — direct
   control always wins). Arrival is recognised when presence records the walker at the leg's end (or,
   for a person, within `PERSON_APPROACH`, 1 200 mm, of them), in the same dispatch.
4. **The route is the geometry owner's answer.** `movement` owns a catalog (`ARC-62`):
   `Wayfinder { wayfinder_of, route(world, RouteAsk) -> Option<RouteAnswer> }`, `RouteAsk { person,
   place, from, to, avoid }` built only by movement, `RouteAnswer` either waypoints (non-empty, integer
   positions ending at the possibly adjusted goal) or `unreachable`. Its rules are `ARC-62` item 4's:
   write-once and process-wide, pure, keeps nothing, inert (`None`) where its pack holds no state for
   the place. Asked in ascending `SystemId`; the first answer wins; with none, the leg is the straight
   segment. `bodies` is the build's wayfinder (`DEP-34`, `ARC-39` note 5). Movement never names it.
5. **Refusals.** `walk-to` is refused `malformed-payload` for a payload it cannot read;
   `NoSupportedInteraction` for an actor that is not a person; `PreconditionFailed` for an actor without
   a presence or a destination presence refuses; `TooFarAway` only for a `person` destination outside
   the walker's place; and **`Rejection::System { code: "no-route" }`** when the destination place has no
   passage chain from the walker's or the first leg's wayfinder answers `unreachable`. The code string
   `no-route` is part of movement's public vocabulary and is never renamed. `walk-step` without a walk
   is `PreconditionFailed`.
6. **Re-planning** happens at a step when the last stride did not end where it was asked (stopped
   short, nudged or shoved), with the person who stopped the walker given as `avoid`; or when a `person`
   destination moved more than 500 mm from where the leg was planned to. A walk re-plans at most eight
   times. A shove or a nudge does not end a walk.
7. **Cadence belongs to the sender.** One `walk-step` per wall second (`EMBODIED_STEP`) is 1.34 m/s
   — Weidmann (1993), the mean free walking speed of pedestrians; Bohannon (1997), 1.27–1.46 m/s
   comfortable gait for adults aged 20–59. A player's client sends it on a timer while its own `walking`
   record is disclosed; a hosted controller is stepped by the host in wall seconds (`ARC-67` item 4); a
   headless `run` steps at a notional cadence. No System Pack reads the cadence, a wall clock or the time
   scale: a walk at 6×, 12× or 24× covers the same ground per wall second.
8. **Disclosure.** To whoever perceives a walking person, movement discloses `walking { destination,
   next }` — the next four waypoints at most. Clients draw intent from the server's own plan; nobody
   plans a route outside the server.

**Accepted limitations.**
- **A sender may step faster than a person walks**, exactly as `move` may be sent back to back
  (`ARC-26`, L-1): a bound per request, not per second.
- **Geometry changed after a plan is noticed only through its effect**: movement cannot see another
  pack's state, so an object kicked across a planned leg is pushed or blocks the stride like any
  object a `move` meets, and a blocked stride re-plans. (Step-11 §21.15 records this against SD-N9's
  wording.)
- **People are not planned round**, except the one who just stopped the walker: passing other people is
  the resolver's nudging and head-on bias (`ARC-39` notes).
- **No overhead geometry, stairs or floors**; no travel between places not joined by passages.

---

## DEP-34 — Route search: `pathfinding`'s A*, over a visibility graph `bodies` builds on integers

**Date** 2026-10-09 · **Status** selected; one dependency added to `systems/bodies` only · **Approved by**
the primary session at PR 12n's design freeze (step-11 §21.0, QN-4) · **Relates to** `ARC-25`,
`ARC-39`, `ARC-55`, `ARC-75`, `DEP-13`, `REUSE_POLICY.md` §§11–12, §17 · **Design**
`.structured-coding/plans/mvp0/step-11-bodies.md` §21.4 (S15, PR 12n-1)

**Problem.** `bodies` must answer "how does a person get from here to there in this place" round the
place's solids and loose objects, with an answer that becomes positions in the log: exact, and the
same on every machine (`ARC-25`, `AC-8`), with no C++ toolchain (Windows) and no engine bound in. A
place holds at most 64 solids and 32 objects, all axis-aligned boxes or discs.

**Options considered** (step-11 §21.4; crates.io and the projects' pages, 2026-10-09; both directions of
`REUSE_POLICY.md`).

```text
search
  (a) pathfinding 4.16.0 (evenfurther), MIT / Apache-2.0 — A*, Dijkstra, BFS, fringe, IDA*, Yen;
      generic over node and integer cost; since 2016, ≈ 3 M downloads; MSRV 1.88       chosen
  (b) our own A*                                       a mature, small, generic search exists
graph
  (c) pathfinding's Grid at 50 or 100 mm               110 k–440 k cells on the street; zig-zag paths
                                                       needing string-pulling against the same boxes;
                                                       a second discretization. Kept as the fallback
                                                       if the graph's cost fails (step-11 NV-10)
  (d) landmass 0.9.2 — a navigation system with        its own agent loop, velocities and frame delta;
      steering and avoidance                           f32 (glam), no determinism statement
  (e) oxidized_navigation 0.12.0                       a Bevy plugin: bound to Bevy's ECS and schedule
  (f) recastnavigation-rs 0.1.0 (Recast/Detour)        C++ on every platform; MPL-2.0 binding; floats
  (g) polyanya 0.17.1 — any-angle paths on a mesh      f32 triangulation (glam, spade), no determinism
                                                       statement; the reference for a later, larger
                                                       geometry
  (h) a visibility graph over the grown boxes, ours    chosen
```

**Choice: (a) for the search and (h) for the graph.** `pathfinding = "=4.16.0"`, an exact pin, in
`systems/bodies/Cargo.toml` only; `systems/bodies/src/route.rs` is the only file that names it.
The graph: every solid's footprint and every loose object's footprint (a ball as its bounding square),
and the person who stopped the walker when there is one, each grown on every side by `PERSON_RADIUS +
GAP + PLAN_MARGIN` (300 + 10 + 50 mm); the floor shrunk by the same. Nodes are the start, the goal and
every grown corner strictly inside the shrunk floor and outside every other grown box, in authored
order. An edge exists when its segment meets no grown box's open interior, decided with integer cross
products; its cost is ⌈√(dx² + dy²)⌉ mm and the heuristic ⌊√(dx² + dy²)⌋, admissible and consistent.
Edges are evaluated lazily as A* expands a node, successors in node order. A goal inside a grown box or
outside the shrunk floor is moved to the nearest free point of the 50 mm lattice (distance, then y,
then x — the entry order of `ARC-39` note 3); none, an enclosed goal, or more than `WAYPOINTS_MAX` (64)
waypoints is `unreachable`.

**Why the graph is ours** (rejecting a mature wheel needs a reason, `CLAUDE.md` §4 rule 16). Every
navmesh candidate is float geometry whose cross-machine bit-identity is unproven, and a route's
waypoints become positions in the log, so they must be exact (`CORE_CONCEPTS.md` §6.2). Rapier is
admitted only behind quantization and a verify-then-degrade check (`DEP-13`); a planner's output has no
such check to fall back on. For axis-aligned boxes the shortest path bends only at grown corners — a
graph of at most 386 nodes — so the case our geometry is made of is the one case where a visibility
graph is both optimal and small.

**Determinism.** Pure integer code in and out. `pathfinding`'s A* keeps its parents in an `FxIndexMap`
(a fixed hasher, insertion order) and orders its heap by estimated cost, then by cost, so equal inputs
give equal paths on every machine; `bodies`' successor order is fixed. An upgrade may change
tie-breaking and so results: it bumps `bodies`' `VERSION`, as a Rapier upgrade does.

**Licences** (`ARC-55`): `pathfinding` is Apache-2.0 or MIT, and so are the crates it brings
(`deprecate-until`, `indexmap`, `integer-sqrt`, `num-traits`, `rustc-hash`, `thiserror`), each recorded in
step-11 §21.15 as `Cargo.lock` admits them.

**Accepted limitations.** Grown boxes are squares, so a route keeps at least the margin from a corner
and more than it needs diagonally. A route is planned per request from current state, so its cost is
bounded by measurement (step-11 NV-10: a plan's maximum ≤ 5 ms, p99 ≤ 1 ms), with the grid as the named
fallback. Overhead geometry and slopes are out of scope (`body:` has none).

---

## ARC-68 — Calendar and weather are System Packs; weather is a seeded WGEN-lite generator built from the published algorithm

**Date** 2026-10-09 · **Approved by** the primary session at PR TW-b's design freeze (step-19 §17.9.1;
QTWb-1 … 7 ruled as recommended) · **Implements** [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §17 ·
**Relates to** `ARC-26`, `ARC-28`, `ARC-33`, `ARC-35`, `ARC-61`, `ARC-67`, `DEP-30`, `DEP-31` (weather
data and the fetch tool, TW-d), `REUSE_POLICY.md` §§11–12 · **Design**
`.structured-coding/plans/mvp0/step-19-time-weather.md` §3.3, §5, §6, §15.1, §16, §17 (S19, PRs TW-a and
TW-b)

**Problem.** A world that wants a date, a sun and weather must have them without the kernel learning what
a day, a sunrise or rain is (kernel ignorance), without any client computing them (clients only render,
`INV-TW-6`), and without the time scale or pause reaching a rule (`ARC-67`, `INV-TW-2`). The weather must
be the same for every observer, reproducible from the world's facts, realistic by default, and
replaceable by a world's author.

**Choice.**

1. **Both are System Packs.** `calendar` (`systems/calendar`, PR TW-a) owns the civil date and the sun.
   `weather` (`systems/weather`, PR TW-b) owns the weather: condition, cloud, temperature, precipitation
   and wind, hour by hour, one climate per world. Each is installed by one line in `systems/installed`
   (`ARC-33`) and enabled by a world's `systems:` list.
2. **`weather` depends on `calendar`, never the reverse** (`INV-TW-10`). It declares the dependency, so a
   world that enables `weather` without `calendar` is refused at assembly by the kernel's existing
   dependency check. It reacts to calendar's Public `day-began` and reads the date and the day's light
   events from that fact's payload; it never recomputes a date or the sun. Removing `weather` leaves
   calendar's facts byte-identical.
3. **Both are configured through `ARC-61`'s seam**: `configure/calendar.yaml` and
   `configure/weather.yaml`, decoded by the owner's own types so a refusal names the file, its line and
   column, and the key. Each configuration becomes one SystemInternal fact with no subjects
   (`calendar-configured`, `weather-configured`). A pack that is enabled but not configured states
   nothing.
4. **Each keeps one world-level Process** (`calendar`, `climate`) whose state is the fold of the pack's
   own facts, so a snapshot and a replay agree and a resumed world continues the same weather chain.
5. **Both emit Public facts at changes** — `day-began`, `daylight-changed`, `weather-changed` — so other
   packs (an umbrella seller, a rainy-day routine) may react without either pack knowing them
   (`ARC-26`, `ARC-28`). The per-day weather record, `weather-day`, is SystemInternal.
6. **Both disclose on the observer's place** through presence's existing `PerceptionProvider::discloses`
   (`calendar-day`, `calendar-light`; `weather-today`, `weather-now`). An observer in no place gets
   none.
7. **Integers only.** Weather is fixed-point integers end to end (0.1 °C, 0.1 mm, 0.1 m/s, per-mille
   probabilities, oktas); no `f32` or `f64` appears in the weather pack's source, which a test scans
   for (`INV-TW-5`). Its randomness is counter-based SplitMix64 keyed by the configured seed, the world
   day and a fixed draw index, so a fixed configuration gives the same weather on every platform.
8. **Kernel, contracts and presence are unchanged** (`INV-TW-3`).

**Alternatives rejected.** A `discloses_at` on presence (a perception contract change for a need the
existing `discloses` meets); client-side astronomy or weather (every client would compute a world rule,
and two clients could disagree); a kernel clock that knows days (`ARC-67`).

**Reuse: the weather generator is built from the published algorithm** (`REUSE_POLICY.md`; step-19
§3.3, read 2026-10-08).

```text
(a) WGEN — Richardson 1981, Water Resources Research 17:182–190; Richardson & Wright 1984, USDA-ARS
    ARS-8. A published algorithm: a first-order two-state Markov chain for wet and dry days with monthly
    P(W|D) and P(W|W), gamma wet-day amounts, AR(1) temperature conditioned on wet or dry. A clean-room
    implementation has no licence issue.
(b) LARS-WG — academic, non-commercial licence only
(c) ClimGen (WSU) — Weibull amounts; not pursued
(d) plain monthly rule tables — trivially authored, but independent days with no rain spells
```

**Selected: (a), simplified to "WGEN-lite"** and written by us in `systems/weather`: the Markov chain
with monthly per-mille probabilities; wet-day amounts from a per-month table of five quintile amounts
instead of a gamma sampler (no floating point); temperature as the monthly mean plus an integer AR(1)
anomaly with bounded integer noise. (b) fails the licence rule. (c) offers no gain in fit for this use.
(d) is kept as the degenerate case of the same file: a table with `p_wet_after_dry == p_wet_after_wet`
*is* an independent table, so there is one schema, not two. No crate is adopted: no maintained Rust
weather generator exists, and the generator is about two hundred lines of integer arithmetic over a
mixer the tree already uses (the paced controller's SplitMix64 finalizer, restated in the pack and
pinned to the reference outputs). The station record, its CSV form and the fetch tool's HTTP client are
`DEP-31`'s (TW-d), not this decision's.

**Defaults are content with sources.** The pack's climate is `configure/weather.yaml`. Market Town's
table is derived from NOAA's U.S. Climate Normals 1991–2020 for San Diego Lindbergh Field (USW00023188)
by formulas written in the file's header; values the normals do not give are labelled provisional until
TW-d fits them from the station record. The daily-to-hourly constants are pack constants with cited
sources: the diurnal temperature shape of Parton & Logan (1981), the American Meteorological Society's
drizzle and heavy-rain intensity thresholds, and the WMO okta scale (step-19 QTWb-6).

**Accepted limitations.** One climate per world (regional weather is QTW-12). Wet-hour counts by daily
amount are a stated design default, not a measured climatology, until the hourly layer (TW-g). Wind has
no day-to-day noise in this version.

---

## DEP-31 — Weather data: NOAA GHCN-Daily, reshaped offline by `tools/weather-fetch`; `ureq` confined to that tool behind an off-by-default feature

**Date** 2026-10-09 · **Status** selected; one optional dependency (`ureq`) added to `tools/weather-fetch`
only · **Approved by** the primary session at PR TW-d's design freeze (step-19 §18.9.1; QTWd-3 … 8 ruled
as recommended) · **Relates to** `ARC-61` (and its `data:` attachment note), `ARC-68`, `DEP-8`,
`REUSE_POLICY.md` §§11–12, step-19 `INV-TW-7` · **Design**
`.structured-coding/plans/mvp0/step-19-time-weather.md` §3.2, §6.3–§6.5, §18 (S19, PR TW-d)

**Problem.** Market Town's weather should be San Diego's real weather, replayed day by day, with the
rules of `ARC-68` filling only what the record lacks. The data must be redistributable inside the
repository, carry its provenance, be produced by a program anyone can re-run, and reach a world without
any crate that `run`, the server or a client builds ever opening a network connection (`INV-TW-7`).

**Data source** (read 2026-10-08, re-read 2026-10-09; the full table is step-19 §3.2).

```text
(a) NOAA GHCN-Daily, station USW00023188 (San Diego International Airport, Lindbergh Field) —
    CC0-1.0 through NOAA's Open Data Dissemination ("There are no restrictions on the use of the
    data", https://registry.opendata.aws/noaa-ghcn/); a US federal work. NOAA requests attribution,
    forbids stating or implying endorsement, and asks that modified data not be presented as original
    NOAA data. Daily TMAX, TMIN, PRCP, AWND, WDF2 and weather-type flags since 1939.
(b) NOAA GHCNh (hourly) — CC0; ISD's successor, with visibility and present weather (fog)
(c) NOAA ISD / ISD-Lite — CC0; superseded (no updates after about 2025-08-24), no fog field
(d) Meteostat — CC BY 4.0; repackages NOAA for this station and adds an attribution duty
(e) Open-Meteo historical — data CC BY 4.0, but the free API is non-commercial only
(f) ERA5 (Copernicus) — CC BY 4.0 since 2025-07-02; a ~31 km grid that smooths the coastal marine
    layer; needs an account and NetCDF/GRIB extraction
```

**Selected: (a)** as the default daily record, years 2015–2024 (ten whole years, two of them leap
years; QTW-5). **(b)** is the optional hourly layer of a later PR (TW-g). (c) is rejected for new work,
(d) as a redundant intermediary, (e) because its access route fails `DEP-8`'s commercial-use rule, and
(f) is kept only as the fallback for a world placed where no station exists (QTW-10). The generator that
fills gaps stays `ARC-68`'s (WGEN-lite, built from the published algorithm; LARS-WG rejected there for
its non-commercial licence); this decision does not restate it.

**Input format: the station's `.dly` file**, `https://www.ncei.noaa.gov/pub/data/ghcn/daily/all/<station>.dly`
— plain fixed-width text whose columns NOAA's `readme.txt` §III documents (one line per station, month
and element; 31 values of eight columns each; missing is −9999). The by-station CSV is a long format
served gzipped, so it would add a gzip dependency for no gain (QTWd-3).

**The committed form is our own CSV, labelled modified.** `worlds/<world>/data/weather/<name>.csv`, the
`weather` pack's input format (`systems/weather/README.md`, step-19 SD-TW-d-1): one row per day,
`date,tmax_dc,tmin_dc,prcp_tenth_mm,awnd_dms,wdf2_deg,fog,thunder`, integers in GHCN-Daily's own units, an
empty cell for a missing or quality-flagged value, `fog = WT01 ∨ WT02 ∨ WT21`, `thunder = WT03`. A `NOTICE`
beside it records the source URL, station, retrieval date, the tool's version and exact command, the
`.dly` input's byte count, the CSV's byte and line counts, the attribution, NOAA's two requested
citations (Menne et al. 2012, *J. Atmos. Oceanic Technol.* 29:897–910, doi:10.1175/JTECH-D-11-00103.1;
Menne et al. 2012, GHCN-Daily Version 3, doi:10.7289/V5D21VHZ), and the statement that the data is
modified (reshaped and gap-reported) from NOAA GHCN-Daily and not endorsed by NOAA. No digest is recorded
(no SHA-256 crate is in the tree, and adding one for this alone is not worth a dependency, QTWd-7):
reproducibility is owned by the tool's determinism test and by `.gitattributes`' `eol=lf` for World Pack
data, which makes every platform check out the same bytes. The pack decodes LF, CRLF and a leading BOM
identically, so a user's own file in either form works too.

**The tool: `tools/weather-fetch`** (crate `mineworld-weather-fetch`), a developer tool that `run`, CI and
the server never execute. `reshape` turns a `.dly` file into the CSV and NOTICE and prints a report
(missing values by year, gap runs, weather-type population by year, quality-flagged values dropped);
`fit` estimates a `configure/weather.yaml` rules block from a CSV in integers only; both are byte-
reproducible for the same input and arguments. It uses the `weather` pack's own CSV and rules types, so
the format has one authority. `fetch` downloads the `.dly` file verbatim and exists only with the cargo
feature `fetch`, **off by default**.

**HTTP client** (`REUSE_POLICY.md`: a commodity, adopted, not built).

```text
(a) ureq 3.4.2 — MIT OR Apache-2.0; blocking, small; rustls with the ring provider and webpki-roots
    (features: default off, `rustls`)
(b) reqwest — async (tokio) for one GET; far larger tree
(c) no HTTP client: the developer downloads the file by hand and runs `reshape`
```

**Selected: (a)**, pinned exactly (`=3.4.2`) in `tools/weather-fetch/Cargo.toml` as an optional dependency
enabled only by the tool's `fetch` feature. No other crate depends on the tool, so no runtime crate can
reach `ureq` (`INV-TW-7`, checked by `cargo tree` and by a test over `Cargo.lock`). The ring provider
needs only a C compiler, which the CI image has; `cargo clippy --all-features` compiles it there. The
feature adds 22 entries to `Cargo.lock` (twelve crates and ten Windows target shims), all under permissive
licences (MIT, Apache-2.0, ISC, BSD-3-Clause; `ring` is
Apache-2.0 AND ISC; `webpki-roots`, Mozilla's root certificates as data, is CDLA-Permissive-2.0); none
is in any default build, and none ships with a world. (b) is
the wrong size for one download. (c) remains the fallback when the network is unavailable: `reshape`
takes any `.dly` file, and the NOTICE records who obtained it.

**Accepted limitations.** WT flags are sparsely reported for some stations and years; the tool reports
their population and the pack then takes fog only from what the record says (TW-g's hourly layer adds
GHCNh). `overcast_morning_permille` cannot be estimated from GHCN-Daily and is carried over from the base
rules by `fit`. The record series adds about 66 KB to every world snapshot (QTWd-2, accepted; retention is
the persistence lane's F-SAVE-1). Windows is argued from `eol=lf`, the CRLF/BOM-tolerant decoder and its
test, not run in CI.

**Note, 2026-10-09 (operator ruling on TWd-F4) — `webpki-roots`' licence.** After this decision was
frozen, `DEP-22`'s `deny.toml` (with `all-features = true`) refused `webpki-roots` (CDLA-Permissive-2.0),
which the `fetch` feature's `ureq` + `rustls` brings. The operator chose a single crate-scoped exception
in `deny.toml` over dropping `fetch` (`DEP-22`'s note of the same date). The tree, the pin and the
confinement above are unchanged.

---

## ARC-57 — Cognition budgets: cost ceilings on wall time, decided before the recorder and the backend

**Date** 2026-10-09 · **Status** accepted; implemented by S10 PR P5a (`mineworld-cognition`,
`budget.py`, `gateway.py`) · **Approved by** the primary session's ruling QS10-18 (2026-10-08) and the
P5a freeze (QP5-4, 2026-10-09) · **Relates to** `ARC-56`, `ARC-58`, `INV-4`, step-17 `I-13` · **Design**
`.structured-coding/plans/mvp0/step-17-cognition.md` §3.10, §15.6;
`.structured-coding/plans/mvp0/pr-s10-p5-backends.md` D-P5-8, §5.3, AP5-7 · **Placeholder**
`ARC-S10-d`

**Problem.** A model call costs wall time on a GPU, or money on a user's hosted key. S11-B's
`time_scale` lets one wall second be N simulated seconds, so a ceiling keyed on simulated time
(`ARCHITECTURE.md` §9.1 as first written: calls per simulated hour, tokens per simulated day) costs N
times as much per wall hour at scale N. A ceiling must also stop a call *before* it is made, or it bounds
nothing.

**Options considered:**

```text
(a) ceilings on simulated time (the first §9.1 text)   cost grows with time_scale; rejected by QS10-18
(b) ceilings on wall time, checked after the call      a refused call has already been paid for
(c) ceilings on wall time, checked before the call,    chosen
    with the actual usage charged after it
(d) one shared ceiling per process, no per-seat one    one talkative seat starves every other seat
```

**Choice: (c).** Cost ceilings are keyed on **wall** time and read only an injected clock (UTC epoch
seconds); no budget code reads simulated time. The **context** bound stays on simulated time, because
memory is about a life (`I-13`); it belongs to memory and context assembly (P4, P6), not here.

- **Per seat** (`EntityKey`), rolling windows: `calls_per_wall_hour` (default 20, 3 600 s) and
  `tokens_per_wall_day` (default 30 000, 86 400 s) — `ARCHITECTURE.md` §9.1's numbers re-keyed.
- **Per process:** `max_in_flight` (default 2, FIFO) and `call_timeout_s` (default 20).
- **Order** (`ModelGateway.complete`): key-material check → budget pre-check → in-flight slot → timeout
  around (recorder → backend) → charge. A refused call reaches neither the recorder nor the backend.
- **Pre-check** reserves `ceil(utf8_bytes(messages) / 4) + max_output_tokens`, erring toward refusal; the
  **post-charge** uses the backend's reported usage, or the estimate flagged `estimated`. A transport
  failure charges the call and no tokens; a timeout charges the call.
- **Ledgers:** an in-memory one for tests and a SQLite one (standard-library `sqlite3`) at a path the
  operator configures, never inside a world save (`INV-4`), so a restart neither resets nor
  double-counts a window.
- **Outcomes** are typed, never exceptions: `Completed | Refused | Failed`. A controller falls back on
  `Refused` and `Failed` (P6).

**Consequences.** A world's `time_scale` changes nothing about what cognition may spend per hour of the
operator's day. A crash-looping cognition process cannot spend again after each restart. Prices are
never modelled: tokens are reported, never priced (step-17 §3.10.4).

**Revisit** if a provider bills on a unit that tokens do not bound (for example per-request fees with
no token count), or if per-seat ceilings prove too coarse for many seats sharing one GPU.

**Dated note, 2026-10-14 (S10 P6b, C4): two additive interface changes.** `ModelGateway.complete` gains an
optional keyword `timeout_s`; the call's timeout is the smaller of `call_timeout_s` and `timeout_s`, so a
consult deadline cut by the caller is charged like a timeout (one call, no tokens), where cancelling from
outside would skip the charge. `BackendFailure` gains `retry_after_s` (an integer, 0 to 3 600, or absent),
set by both HTTP adapters from a delta-seconds `Retry-After` on any non-2xx response; an HTTP date is
ignored. The gateway still never retries (`ARC-73`). Failures are never recorded, so cassette format and
keys are unchanged. Ruled by QP6-6 (2026-10-14).

---

## ARC-58 — Recorded model outputs: an interface-level cassette, keyed on the provider-neutral request

**Date** 2026-10-09 · **Status** accepted; implemented by S10 PR P5a (`record.py`,
`backend/canonical.py`) · **Approved by** the primary session at the P5a freeze (2026-10-09), under the
operator's rulings QS10-14 (recorded outputs, tests with no network) and QS10-19 (local models only)
· **Relates to** `ARC-56`, `ARC-57`, `AC-4`, `CLAUDE.md` §4 rule 10 · **Design** step-17 §3.11, §4.5;
`pr-s10-p5-backends.md` D-P5-5, D-P5-6, D-P5-7, AP5-1 … AP5-5, AP5-8 · **Placeholder** `ARC-S10-e`

**Problem.** Core tests never need a live model (`CLAUDE.md` §4 rule 10), yet the cognition path must be
exercised with real model outputs, deterministically, on every platform. `AC-4` requires that swapping
the backend needs no World Pack edit, so a recording must replay under any backend.

**Options considered** (P5a §4.3, with licences and maturity):

```text
vcrpy / pytest-recording (MIT)   HTTP cassettes: provider wire formats, URLs and headers to scrub; a
                                  cassette recorded against Ollama does not replay under llama.cpp; no
                                  httpx2 support found
respx (BSD-3-Clause)              mocks httpx, not httpx2; HTTP-level, so provider-specific
LiteLLM caching (MIT)             the key contains the provider-prefixed model name
inline-snapshot (MIT)             reference only: its review-the-diff workflow is the model for re-recording
our own interface-level cassette  chosen
```

**Choice.** A cassette records at **our** interface, above HTTP:

- **The key** is `sha256` over canonical JSON of `{"key_scheme": 1, "request": CompletionRequest}`:
  UTF-8, sorted keys, `(",", ":")` separators, no ASCII escaping, no Unicode normalization, and **no
  floats** (a float anywhere, including inside an output schema, is refused). The request holds no
  provider, model, URL, tier, binding or key, so one cassette replays under every backend.
- **The format** is JSON Lines, UTF-8, LF: a header `{"cassette": "mineworld-cognition", "format": 1,
  "key_scheme": 1}`, then entries `{key, request, completion, meta: {binding, model, recorded_at,
  latency_ms}}`. Never a URL, a header, a key, `key_env`, or a transport failure.
- **Replay is strict and ordered.** Entries are grouped by key in file order; the n-th call with a key
  returns the n-th entry; one more is a miss (`exhausted`), an unknown key is a miss (`absent`, with the
  nearest recorded request and the first differing JSON path). A miss is an exception, never a fallback,
  and never reaches a backend. An entry whose `key` does not match its `request` is refused at load.
- **Record is atomic**: it writes `<name>.jsonl.partial`, flushes each line, and replaces the cassette on
  clean close; a crash leaves the old cassette and a visible `.partial`, which blocks the next record.
- **Modes**, process-wide: `live`, `record`, `replay`, `scripted`. In `replay` and `scripted` no backend
  object is constructed and the HTTP adapter is never imported, which is stronger than a network guard
  that allows loopback, where a user's local model listens.

**Consequences.** Redaction holds by construction rather than by scrubbing. A prompt change invalidates
the cassettes that recorded it, and the miss names the first differing path; re-recording from a real
model is operator work (QS10-19).

**Revisit** if a provider feature the plain request cannot express (tool calls, streaming deltas) must be
recorded, which would be a `key_scheme` or `format` bump, never a silent change.

---

## DEP-27 — The model HTTP client: `httpx2`, and no provider SDK

**Date** 2026-10-09 · **Status** adopted by S10 PR P5a (`httpx2>=2.13,<3`; locked 2.13.1) · **Approved
by** the primary session (QP5-1, 2026-10-09) · **Licence** BSD-3-Clause · **Supersedes** step-17 §4.1's
placeholder `DEP-S10-a` (the `openai` SDK) · **Relates to** `ARC-58`, `DEP-24`, `DEP-32` · **Design**
`pr-s10-p5-backends.md` §4.1, §4.2, D-P5-4, D-P5-10, AP5-6

**Problem.** Every backend MineWorld targets — Ollama, llama.cpp's server, LM Studio, vLLM, and the hosted
OpenAI-compatible APIs a user configures (OpenAI, xAI, DeepSeek, Zhipu GLM, Mistral, Kimi, DashScope,
Gemini, OpenRouter, Groq) — speaks the OpenAI-compatible `POST /chat/completions` schema. Something must
send that request with a cancellable timeout, and must refuse every source of configuration but the
operator's file.

**Options considered** (facts read 2026-10-09):

| Candidate | Licence; maturity | Verdict |
| --- | --- | --- |
| `openai` SDK (`AsyncOpenAI(base_url=…)`) | Apache-2.0; 3.27.0 | **Rejected.** With an argument omitted its constructor reads `OPENAI_API_KEY`, `OPENAI_BASE_URL`, `OPENAI_ORG_ID`, `OPENAI_PROJECT_ID`, `OPENAI_ADMIN_KEY`, `OPENAI_WEBHOOK_SECRET` and `OPENAI_CUSTOM_HEADERS`, and defaults to `api.openai.com`: a hosted key in a user's shell could redirect local traffic, or its key, to a hosted API (QS10-19). Its retries hide calls from the budget ledger. Its value-add, typed parsing, is ours anyway (local validation) |
| `ollama-python` | MIT; 0.6.3 | Reference only: native API only; pins the old `httpx` line |
| LiteLLM | MIT core, proprietary `enterprise/` | Rejected: the provider lives in the model string, and the tree is large |
| `httpx` | BSD-3-Clause; 0.28.1, no release since 2024-12 | Fallback only: unmaintained; `openai` 3.x and the Anthropic SDK moved to `httpx2` |
| **`httpx2`** | **BSD-3-Clause; 2.13.1 (2026-09-23)**, stewarded by Pydantic Services (whose `pydantic` is `DEP-24`) | **Adopted** |
| `aiohttp` | Apache-2.0 AND MIT; 3.14.4 | Rejected: nine runtime dependencies and a server framework, for one POST per decision |
| standard library (`urllib.request` in a thread) | PSF | Rejected: blocking I/O in a thread cannot be cancelled when the timeout fires |

**Choice.** One adapter of our own, `backend/openai_compatible.py`, about 200 lines over
`httpx2.AsyncClient`: request mapping, response parsing, error mapping, **no retries** (one call is one
ledger entry). It is constructed only from a typed `BackendConfig`; `base_url` has no default; it reads
no environment variable except the one `key_env` names. It is the only module that imports `httpx2`.

**Verified at P5a (C1), not assumed:**

- `httpx2` 2.13.1 exposes `AsyncClient`, `Timeout` (per-phase), `MockTransport` (sync and async
  handlers, `httpx2/_transports/mock.py`) and the `httpx` exception hierarchy (`ConnectError`,
  `TimeoutException`, …).
- Its dependencies (`anyio`, `httpcore2`, `h11`, `idna`, `truststore`; `typing-extensions` below 3.13)
  are pure-Python `py3-none-any` wheels, so the universal lock serves Linux, macOS and Windows alike.
  `httpx2-jsfetch` is locked but installed only on `emscripten`.
- **Finding:** `AsyncClient(trust_env=True)`, the default, reads proxy variables, `.netrc` and
  certificate-file variables from the environment (`httpx2/_client.py`). The adapter therefore passes
  `trust_env=False`: a user who needs an HTTP proxy for a hosted API is not served by this version, which
  is recorded as a limitation, not worked around with an implicit read.

**Revisit** if the native Anthropic adapter needs the `anthropic` SDK (P5b compares it), if a needed
provider feature (tool calls, batch, realtime) cannot be expressed in the plain schema, or if `httpx2`
stops being maintained — the switch to `httpx` or `aiohttp` is one file.

---

## DEP-32 — Reading a `.env`-style key file: `python-dotenv`'s `dotenv_values`, never `load_dotenv`

**Date** 2026-10-09 · **Status** adopted by S10 PR P5a (`python-dotenv>=1.2,<2`; locked 1.2.4) ·
**Approved by** the operator's requirement of 2026-10-09 (API keys through a `.env`-style file) and the
primary session at the P5a freeze (QP5-6, QP5-10 … QP5-12) · **Licence** BSD-3-Clause · **Relates to**
`DEP-27`, `ARCHITECTURE.md` §9.2 · **Design** `pr-s10-p5-backends.md` §4.2b, D-P5-9, D-P5-14, AP5-9,
AP5-13

**Problem.** A user's hosted API needs a key. The configuration names only the **variable** (`key_env`);
its value comes from the process environment, or from a `.env`-style file the operator names. That
file's format (quoting, `export` prefixes, comments, multiline values, CRLF) is easy to get subtly wrong.

**Options considered:**

| Candidate | Licence; maturity | Verdict |
| --- | --- | --- |
| **`python-dotenv`** | **BSD-3-Clause; 1.2.4; no runtime dependencies** | **Adopted**, one call: `dotenv_values(path, interpolate=False)`, which returns a mapping and does not touch `os.environ` |
| our own parser | ~40 lines | Rejected: the quoting and escaping corner cases would be ours to find (`REUSE_POLICY.md` §12) |
| `pydantic-settings` | MIT | Rejected: it binds every settings field to environment variables; our file is TOML and only key values come from the environment |
| `environs` | MIT | Rejected: wraps `python-dotenv` and mutates `os.environ` by default |
| process environment only | — | Kept as one of two sources, not the only one: the operator asked for `.env`-style files |

**Choice.** `secrets.py` is the only importer of `dotenv` and the only reader of a key. The process
environment wins over the file (`python-dotenv`'s own `override=False` default). Never `load_dotenv`, so
a file's values never become ambient credentials for a child process or for another library's implicit
read; never interpolation; never a default file location. A key file inside a directory holding a
`world.yaml` is refused (credentials never travel with a world), and on macOS and Linux a file readable
by group or others is refused, as `ssh` refuses a key. A key value lives in a `Secret` whose `repr` is
redacted, and is never logged, recorded, put in an exception, or written to the ledger.

**Revisit** if `python-dotenv` gains runtime dependencies or changes `dotenv_values`' contract, or if
an OS keychain becomes a required source (a separate decision).

---

## DEP-33 — Claude by API key: a native Anthropic Messages adapter over `httpx2`; the SDKs declined

**Date** 2026-10-09 · **Status** adopted by S10 PR P5b (no new dependency: `httpx2` is `DEP-27`'s) ·
**Approved by** the primary session (QP5b-4, 2026-10-09) and the operator's ruling QP5b-1 (Claude is
supported by API key only) · **Relates to** `DEP-27`, `DEP-32`, `ARC-58`, `ARC-60` · **Design**
`pr-s10-p5b-hosted-subscriptions.md` §4.3, §5.1, AB-1, AB-2

**Problem.** The operator asked for Claude among the hosted models. Anthropic's OpenAI-compatibility
layer, reachable through `DEP-27`'s adapter, ignores `response_format` and `seed` and refuses a
temperature below 1 on recent models (`platform.claude.com/docs/en/api/openai-sdk`, read 2026-10-09:
"not considered a long-term or production-ready solution"). Schema-constrained output needs the native
Messages API (`output_config.format = {"type": "json_schema", …}`).

**Options considered** (facts read 2026-10-09):

| Candidate | Licence; maturity | Verdict |
| --- | --- | --- |
| `anthropic` SDK | MIT; 1.13.0; depends on `httpx2`, `anyio`, `docstring-parser`, `jiter`, `pydantic`, `sniffio`, `typing-extensions` | **Rejected.** Verified in its source (`src/anthropic/_client.py`, `main`, read 2026-10-09): with an argument omitted, both `Anthropic.__init__` and `AsyncAnthropic.__init__` run `os.environ.get("ANTHROPIC_API_KEY")`, `os.environ.get("ANTHROPIC_AUTH_TOKEN")` and `os.environ.get("ANTHROPIC_BASE_URL")`; `max_retries` defaults to `DEFAULT_MAX_RETRIES = 2` (`_constants.py`), so one decision could be three calls the budget ledger never sees. Both contradict `DEP-27`'s rules (no environment read but `key_env`; no retries) |
| `claude-agent-sdk` | MIT; 0.2.165; bundles the Claude Code CLI | **Rejected.** It would ship Claude Code inside MineWorld (Anthropic's Commercial Terms apply), and its documentation forbids the subscription login it would be used for |
| Anthropic's OpenAI-compatibility layer through `DEP-27`'s adapter | — | **Rejected for Claude**, and refused by `config.load`: `kind = "openai-compatible"` with a `base_url` on `api.anthropic.com` is a `ConfigError` naming `kind = "anthropic"` |
| **Our adapter over `httpx2`** | — | **Adopted**: `backend/anthropic_messages.py`, one POST per call |

**Choice.** `AnthropicMessagesBackend` (`kind = "anthropic"`): `POST {base_url}/v1/messages` with
`x-api-key` (only when a key was resolved) and `anthropic-version: 2023-06-01`, never `Authorization`;
`system` messages hoisted; `max_tokens` from the request; `temperature` sent only when configured
(`"omit"` is the default for this kind); no `seed` (the API has none); no retry; `trust_env=False`. The
request's schema is **lowered** for Anthropic's structured outputs (keywords it refuses removed,
`additionalProperties: false` on every object) by a pure function that returns a copy: the cassette key
is computed on the unlowered request (`ARC-58`), so a cassette recorded through Claude replays under any
backend, and local validation (P6) still enforces every removed bound. `anthropic_messages.py` and
`openai_compatible.py` are the only importers of `httpx2`.

**Revisit** if Anthropic's API needs a feature the plain Messages request cannot express (tool use,
batches, streaming), if the OpenAI-compatibility layer gains `response_format`, or if the SDK stops
reading the environment implicitly and retrying by default.

---

## ARC-60 — Subscriptions: the user's own CLI as a backend, opt-in, and never Claude's

**Date** 2026-10-09 · **Status** accepted; the bridge core implemented by S10 PR P5b; the Codex preset
**conditional** on the re-read of OpenAI's Terms of Use (P5b C1 → C4) · **Approved by** the operator's
rulings QP5b-1 (drop the Claude subscription route) and QP5b-2 (Codex as an opt-in for the user's own
local use), and the primary session (QP5b-3), 2026-10-09 · **Relates to** `DEP-27`, `DEP-32`, `DEP-33`,
`ARC-57`, `ARC-58` · **Design** `pr-s10-p5b-hosted-subscriptions.md` §4, §5.2 … §5.4, I-B1 … I-B5, AB-3
… AB-9

**Problem.** The operator asked for subscriptions "authorized by Codex's or Claude Code's own login"
beside API keys. A subscription is the user's account with a vendor, and the vendor's terms decide what
a third-party program may do with it.

**The evidence** (verbatim; dates and URLs in the design's §4):

- Anthropic, Claude Code "Legal and compliance" (`code.claude.com/docs/en/legal-and-compliance`, read
  2026-10-09): "Anthropic does not permit third-party developers to offer Claude.ai login into their own
  applications, or to route requests through Free, Pro, or Max plan credentials on behalf of their
  users. Moreover, developers may not collect, store, or intermediate Claude.ai credentials or session
  tokens". Enforcement "may do so without prior notice". Developers "should use API key authentication
  through Claude Console or a supported cloud provider".
- OpenAI, Codex authentication (`learn.chatgpt.com/docs/auth`, read 2026-10-09): "API keys are still the
  recommended default for automation." "Treat `~/.codex/auth.json` like a password." OpenAI's Terms of
  Use answered HTTP 403 to every fetch (planning, freeze, and P5b C1 on 2026-10-09); the clause quoted
  from search excerpts ("Automatically or programmatically extract data or Output") is not yet re-read
  from the live page.

**Choice.**

1. **No Claude subscription route** (QP5b-1). No `claude-code-subscription` kind exists; `config.load`
   reports it as unknown. Claude is supported by the user's own API key (`DEP-33`).
2. **A generic bridge**, `backend/cli_bridge.py` (`CliBridgeBackend`), runs the user's own installed and
   logged-in model CLI for one tool-less answer, under these invariants:
   - **I-B1** MineWorld never reads, writes, copies or forwards a CLI's credentials; the CLI
     authenticates itself;
   - **I-B2** the child's environment is an allowlist (system paths, home and temporary directories,
     locale, XDG directories, the preset's home variable): no `*_API_KEY`, no `*_TOKEN`, nothing from an
     `env_file`;
   - **I-B3** no model-facing text in argv: the prompt goes through stdin, the schema through a
     temporary file; argv holds only literals and validated configuration values (the Windows `.cmd`
     argument-injection class, CVE-2024-24576);
   - **I-B4** the call is bounded and its timeout kills the whole process tree (POSIX: a new session and
     `killpg`; Windows: a new process group and `taskkill /T /F`);
   - **I-B5** a subscription backend exists only when its preset was ruled in **and** the user set
     `acknowledge_terms = "<kind>"`; `replay` and `scripted` modes never construct it.
   The child runs in an empty temporary directory, so no project configuration is picked up. A bridged
   call passes the same gateway: the budget pre-check before the spawn, the same cassettes, the CLI's
   reported usage or the estimate.
3. **Codex through the user's ChatGPT plan** (QP5b-2): an opt-in for the user's own local use only, off
   by default, never in a shipped example, gated by `acknowledge_terms`, and documented after every
   API-key route. **Conditional:** its preset is built only after OpenAI's Terms of Use are re-read from
   the live page and recorded in the design's §4.2. If the re-read forbids the use, the route is dropped
   as Claude's was.

**Limitation accepted.** The user's own global CLI configuration (hooks, MCP servers) runs during a
bridged call; MineWorld does not and cannot disable it. A CLI's flags may change between versions; a
preset is one file.

**Revisit** if a vendor changes its terms or enforces against such use (remove the preset: one file and
one registry line), or if a vendor publishes an explicit permission or prohibition for third-party
programs running its CLI with a subscription.

---

## ARC-81 — Save retention: snapshots pruned by a pure rule and stored compressed; the log kept whole

**Date** 2026-10-10 · **Amends** `ARC-25`'s accepted limitation ("compaction and snapshot pruning are
later work"), for snapshots only · **Relates to** `INV-11`, `ARC-23`, `ARC-25`, `ARC-27`, `ARC-49`,
`DEP-2`, `DEP-5`, `DEP-43`, [`MVP.md`](MVP.md) §9 `AC-8`, `AC-12` · **Approved by** the operator's
rulings on QSR-1, QSR-3, QSR-4 and the primary session's on QSR-2, QSR-5, QSR-6 (2026-10-10) ·
**Design** `.structured-coding/plans/mvp0/pr-s6-save-retention.md` (finding `F-SAVE-1`)

**Problem.** A save wrote a full world snapshot every 64 revisions and never dropped one. Measured at
`bb62edf`: a 30-day Market Town save is 357 MiB and a 300-day one 3.53 GiB, of which 80–84 % is
snapshots of ≈ 0.7 MB each. Snapshot size had plateaued; the growth was purely their count.
Snapshots are checkpoints, never an authority (`ARC-25`), so almost all of them carry no information
the journal cannot regenerate.

**Options considered** (design §5; sizes for the 300-day save).

```text
5.1  keep genesis + an anchor every 4 096 revisions + the newest 2     ≈ 636 MiB   adopted
5.2  zstd-compressed snapshot rows                                    ≈ 809 MiB   adopted
5.1 + 5.2                                                             ≈ 587 MiB   the choice
5.3  delta snapshots (zstd --patch-from a base)                       chained restore, one damaged
                                                                      delta damages every later one
5.4  SQLite VACUUM / auto_vacuum                                      no effect alone; nothing deleted
5.5  page compression: ZIPVFS, sqlite-zstd                            proprietary; LGPL-3.0, layout
                                                                      changed outside our transaction
5.6  content-addressed dedup of snapshot sections                     a second storage model for ≈ 3 MiB
5.7  drop journal and facts before a checkpoint                       contradicts ARC-25 and INV-11
5.8  compress the log in sealed blocks (SR-b)                         ≈ 40 MiB; changes where the
                                                                      authority's bytes live: its own PR
```

**Choice.**

1. **Only snapshot rows are ever deleted** (D-SR-1). No row of `facts` or `journal` is deleted,
   updated or re-encoded; their bytes are exactly what a format-2 build wrote for the same run.
2. **Anchors** (D-SR-2). Scheduled snapshots stay every `interval` revisions (default 64). An anchor is
   a scheduled snapshot at a multiple of `64 · interval` (default 4 096). Genesis (revision 1) is
   always kept.
3. **The retention rule** (D-SR-3). When the scheduled snapshot at revision *n* is committed, the
   retained set is `K(n) = {1} ∪ {a ≤ n : a mod (64·interval) = 0} ∪ {n, n − interval}`; every stored
   snapshot *s < n* outside `K(n)` is deleted in the same transaction. An off-lattice checkpoint (a
   clean shutdown's) is kept until the next scheduled snapshot and then falls under the rule. The rule
   reads *n*, the interval and the set of stored revisions, nothing else, so a killed-and-resumed
   headless run holds the same rows as an uninterrupted one (`ARC-27`).
4. **The codec** (D-SR-4, D-SR-5). A stored snapshot is a zstd frame — level 3, checksum on, content
   size in the header, single-threaded, no dictionary — of the exact bytes `serde_json` gives the
   `WorldSnapshot`. It lives in `persistence/src/format.rs` (`encode_snapshot`, `decode_snapshot`);
   the backend stays opaque (`DEP-2`). Journal entries and facts are not compressed.
5. **Comparison is of decoded bytes** (D-SR-6). `verify` decompresses each stored snapshot and compares
   its JSON bytes with the state the history produces. `verify_from(backend, composed, anchor)`
   restores a retained snapshot and re-executes to the head with the same checks.
6. **Where pruning runs** (D-SR-7). `PersistentWorld` tracks the stored snapshot revisions, computes
   the revisions to retire and hands them to the backend in `RevisionRow::retire`; the SQLite backend
   deletes them inside the revision's transaction, after inserting the new snapshot. A failed delete
   fails the whole revision.
7. **Format** (D-SR-8). `SAVE_FORMAT` is 3. A format-2 save is refused by name
   (`SaveFormatOutdated { saved: 2, supported: 3 }`); no migration (QSR-3: no release has shipped).
8. **Visible** (D-SR-9). `mineworld inspect` prints the retained snapshot revisions and their stored
   sizes (`ARC-23`).

**Why it does not weaken `ARC-25`.** A restart still restores the newest snapshot at or below the head,
which the rule never deletes (`n` and `n − interval` are always kept). `verify` still re-executes every
revision from genesis and compares every fact and every snapshot the save holds. Equality of state is
argued only from decoded JSON, so no codec setting can change a replay verdict. Free pages left by a
delete are reused by the next insert; `VACUUM` is never run on a user's save.

**Accepted limitations.** The log alone still grows ≈ 1.9 MiB per simulated day in Market Town; SR-b
(log compression, design §6.9) is its remedy, approved as a separate PR (QSR-1). Retention is a constant,
not a player or World Pack setting (QSR-4). A hosted world may hold one extra off-lattice checkpoint
between scheduled snapshots, which `AC-12` does not cover (`ARC-27`).

**Revisit** if a feature needs denser history (a "rewind to any hour" would revisit 5.3 / 5.6 and the
anchor spacing), if `AC-8` parity ever differs in the `snapshots` table (see `DEP-43`), or when SR-b is
designed.

**Note, 2026-10-10 (S15, PR 12n-2; step-11 §21.15 M-6, the operator's ruling) — ASR-2 for walking
towns.** Since people walk (`ARC-75`), a walk's strides are requests and facts of their own. Market Town
records about 2.2 × the facts, and its 30-day save is 112.3 MiB, against 59.2 MiB before. ASR-2's bound
becomes ≤ 128 MiB for walking towns; the retention rule and the snapshots it keeps are unchanged. The
300-day size is printed by `tools/cli/tests/market_town.rs` as a measurement, not a bound. SR-b (log
compression) is the expected remedy for the size, as above.

**Note, 2026-10-12 (S15, PR 12d; step-11 §19.13 TD-D12 item 2, the operator's ruling) — ASR-2 for towns
with bodies.** With `bodies` installed in the towns (`ARC-39` note 7), every walk also meets walls,
furniture and people: stopped-short, displaced arrivals and shoves join the log, and Market Town's
30-day save measured 170 811 392 B (162.9 MiB), about 45 % over the walking town's 112.3 MiB. ASR-2's
bound becomes **≤ 192 MiB for towns with bodies** (the 30-day market-town save); the retention rule and
the snapshots it keeps are unchanged. SR-b's log compression is still expected to bring the size back
down.

**Note, 2026-10-13 (SR-b, `ARC-89`) — rule 1 amended.** Rule 1 now reads: no row of `facts` or `journal`
is updated or re-encoded, and none is deleted except when it is **moved unchanged into a sealed block**,
in the transaction of the anchor commit that seals its span, after the block has been decoded and
compared with the rows it replaces. This is the only amendment: rules 2 – 8 (anchors, the retention rule,
the snapshot codec, comparison of decoded bytes, where pruning runs, visibility) stand, except that
`SAVE_FORMAT` is now 4 (rule 7) and a format-3 save is refused by name like a format-2 one. The snapshot
table no longer references the journal, since a retained genesis or anchor snapshot outlives its sealed
journal row. ASR-2's 30-day bound is replaced by `ARC-89`'s tighter ones (≤ 48 MiB at 30 days).

---

## DEP-43 — Snapshot compression: `zstd` (bundled libzstd), level 3

**Date** 2026-10-10 · **Status** selected; one dependency added to `mineworld-persistence` only ·
**Approved by** the primary session (QSR-6) and the operator's QSR-1 ruling, 2026-10-10 · **Relates
to** `ARC-25`, `ARC-49`, `ARC-55`, `ARC-81`, `DEP-2`, `DEP-22`, `REUSE_POLICY.md` §§1–4, 11–12 ·
**Design** `.structured-coding/plans/mvp0/pr-s6-save-retention.md` §5

**Problem.** A plateaued Market Town snapshot is ≈ 700 KB of JSON, written every 64 revisions on the
world thread. It must shrink by an order of magnitude, cheaply, and the stored bytes must be the same
on macOS, Linux and Windows, because `AC-8` hashes every stored byte of a 30-day save on all three
(`ARC-49`).

**Options considered** (one day-30 snapshot of 703 730 B, measured on the design laptop).

```text
zstd 3 (zstd crate, bundled libzstd)     52 045 B  13.5×   ≈ 1.2 ms; decompress ≈ 1 GB/s   chosen
zstd 9                                   41 550 B  16.9×   ≈ 5× slower than level 3
zstd 19                                  33 021 B  21.3×   seconds per 50 snapshots
deflate 6 (miniz_oxide / flate2)         45 672 B  15.4×   ≈ 16 ms: ≈ 13× slower
lz4 (lz4_flex, pure Rust)                96 989 B   7.3×   fastest, 1.9× larger
ruzstd (pure Rust zstd)                  its own documentation: the encoder does not yet reach the
                                         reference library's speed or ratio        the fallback
sqlite-zstd                              LGPL-3.0 (outside ARC-55); asynchronous chunking
ZIPVFS                                   proprietary licence
```

**Choice.** `zstd = { version = "0.13", default-features = false }` in the workspace, used only by
`persistence/src/format.rs` (the codec behind `encode_snapshot` / `decode_snapshot`, `ARC-81`). Default
features off: no legacy-format decoding, no dictionary builder, no multithreading. libzstd is compiled
from the source bundled in `zstd-sys` with the same `cc` path `libsqlite3-sys` already uses (MSVC on
Windows, clang on macOS, gcc in the Linux container): no new toolchain. `zstd-sys`'s `pkg-config`
feature is never enabled, and no MineWorld build or CI job sets `ZSTD_SYS_USE_PKG_CONFIG` (the
environment variable that would link a system libzstd instead); the exact versions are pinned by
`Cargo.lock`.

**Determinism.** One compression level, no dictionary, single-threaded, checksum and content size in
the frame header: the stored bytes are a pure function of the JSON bytes and the pinned libzstd. That is
checked rather than assumed: `AC-8` compares the `snapshots` table's stored bytes across the three
platforms. A future zstd upgrade may change the stored bytes of new snapshots; it changes no decoded
state and no replay verdict (`ARC-81`: comparison is of decoded JSON), and `AC-12` compares two runs of
one build.

**Licences** (`ARC-55`, `DEP-22`), as `Cargo.lock` admits them: `zstd` 0.13.3 is MIT; `zstd-safe`
7.3.0 and `zstd-sys` 2.1.1+zstd.1.5.7 declare BSD-3-Clause, the licence the bundled libzstd 1.5.7 is
taken under (upstream is dual BSD / GPL-2.0). `zstd-sys` builds with `cc`'s `parallel` feature, which
adds build-time crates only: `jobserver` 0.1.35 and `getrandom` 0.4.3 (MIT or Apache-2.0) and `r-efi`
6.0.0 (MIT or Apache-2.0 or LGPL-2.1-or-later, taken under MIT; compiled only for UEFI targets).
`cargo deny check licenses sources bans` admits the graph.

**Re-evaluation trigger.** A platform build of `zstd-sys` fails, or `AC-8` parity differs in the
`snapshots` table: then `ruzstd` (MIT, pure Rust) is the named fallback, or `ci_parity` hashes decoded
snapshot JSON (a reviewed change to `ARC-49`). Either is a decision returned to the operator.

---

## ARC-77 — Milestone E's composition: Lakeside, the repository's own data packs, and named roots everywhere

**Date** 2026-10-10 · **Approved by** the primary session at PR E-e's design freeze (step-16 §18.0;
FQ-e1 … FQ-e8) · **Implements** [`MODULE_SPEC.md`](MODULE_SPEC.md) §2;
[`ARCHITECTURE.md`](ARCHITECTURE.md) §14; [`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md) §8 · **Relates to**
`ARC-33`, `ARC-34`, `ARC-53`, `ARC-54`, `ARC-55`, `ARC-66`, `ARC-71`, `ARC-49` · **Design**
`.structured-coding/plans/mvp0/step-16-packages.md` §1.4, §8.2, §9.6, §18 (S16, PR E-e)

**Problem.** E-a … E-d made every mechanism of Milestone E — pack identity, requirements, a System Pack
from another repository, Entity Packs — but no world used them together, so "a real world assembled from
independently installable packs" was still a claim about parts. Composing one raises three questions
the mechanisms leave open: where the repository's own data packs live, whether they are "bundled", and
how every program that reads a world finds packs that are, by `ARC-54`, never found implicitly.

**Choice.**

1. **Lakeside is Milestone E's composition.** `worlds/lakeside` is assembled from bundled System Packs
   (presence, movement, conversation, group-activity, relationships, naming, schedule, item, inventory,
   item-transfer, consumption), the third-party `fishing` System Pack (`ARC-66`), the Entity Pack
   `modern-goods` and the Presentation Packs `mineworld-default-2d` and `-3d`. Its `requires:` names the
   third-party pack, the Entity Pack and both Presentation Packs, each with a range. Its acceptance is
   step-16 §8.2 (M-1 … M-10), held by `tools/cli/tests/milestone_e.rs` through the real binary and server.
2. **The repository's own data packs have one home per type.** Entity Packs live under `entities/`
   (`entities/modern-goods`), beside `presentation/` for Presentation Packs. Each is found only through
   a named pack root (`--packs entities`, `--packs presentation/mineworld-default`, or
   `MINEWORLD_PACKS`): living in this repository changes nothing about how a data pack is found.
3. **"Bundled" keeps `ARC-54`'s meaning.** A bundled pack is a code pack compiled from this workspace,
   versioned with the framework, and may not be listed in `requires:`. A data pack is never bundled,
   wherever its files live: a world that uses `modern-goods` requires it with a range, exactly as it
   would a data pack from anywhere else. "The repository's own" is description, not a category.
4. **Every program that reads a world names its roots.** The milestone test passes the roots to every
   command and removes `MINEWORLD_PACKS` from every child it spawns; the tests that enumerate `worlds/`
   pass `--packs entities` too; AC-8's instrument (`scripts/ci_parity.py`, `ARC-49`) and the image check
   (`scripts/ci_image.py`) pass `--packs entities --packs presentation/mineworld-default` for every
   world, and the runtime image carries `entities/` and the two Presentation Packs' `pack.yaml` and
   `manifest.yaml` (not their art). A world that requires nothing is unaffected by roots it does not use.
5. **The catch is the world's; the goods are the pack's.** `fish`, what Lakeside's water yields, is the
   world's own item kind; `modern-goods` holds shared shop goods and must not declare it (`ARC-71`
   rule 5 would refuse the collision).
6. **No product code.** Milestone E's world and test change no crate's source or manifest: every
   behaviour they rely on merged with E-a … E-d. The rule controller is not taught the new pack
   (`ARC-34`): seats fish because `fish` is offered complete and their routines take them to the water.

**Options considered.** Copying Market Town's item files into `modern-goods` — rejected: a second
statement of each kind, and Market Town keeps its own. Calling the repository's data packs "bundled" and
letting worlds use them unrequired — rejected: it reuses a defined term for another concept and brings
back an implicit search. Dropping Lakeside's Presentation Pack requirements so the image needs only
`entities/` (FQ-e2) — rejected: the requirement is how a world declares what it is authored for
(`ARC-54`), and the image cost is four small files. Skipping worlds that need roots in AC-8 — rejected:
it would exempt exactly the composed world from the parity claim.

**Accepted limitations.**
- No economy or employment in Lakeside, so `modern-goods`' kinds enter only as people's starting
  holdings; fish is the only renewable food, and the milestone test measures that it closes the loop.
- The reference clients do not draw Lakeside: a Presentation Pack is declared and validated, not yet
  applied (step-16 QSE-9).

**Note, 2026-10-12 (S15, PR 12d; step-11 §19.13 TD-D12 item 3, the operator's ruling) — Lakeside after
item names.** Lakeside's own `fish` kind gains its display name (`Fish`), and its `requires` moves to
`modern-goods: "^0.2"` (`ARC-71` note of the same date). Lakeside's facts change by the names alone;
the composition, its packs and the milestone's claims are unchanged.
- A saved 300-day run of a world this size is gigabytes (F-SAVE-1); the persistence lane owns retention.

**Revisit** when a registry or `.mwpack` (E-C) gives data packs a place outside this repository, or when
the clients apply Presentation Packs.

---

## ARC-59 — A seat's memory is a derivation of what it perceived, compressed with its Event IDs kept

**Date** 2026-10-10 · **Status** accepted; implemented by S10 PR P4 (`mineworld-cognition`, `memory/`
and `compress/`) · **Approved by** the primary session's P4 rulings QP4-1 … QP4-10 and the operator's
QP4-11 (2026-10-10), under QS10-1 (`AC-10` is an MVP-0 gate), QS10-11, QS10-17 and QS10-19 ·
**Relates to** `ARC-56`, `ARC-57`, `ARC-58`, `ARC-43` (the `perceived` stream), `INV-4`, `INV-13`,
`AC-10`, `DEP-37` · **Design** `.structured-coding/plans/mvp0/step-17-cognition.md` §3.8, §3.9;
`.structured-coding/plans/mvp0/pr-s10-p4-memory.md` D-P4-2 … D-P4-10, AP4-10 · **Placeholder**
`ARC-S10-c`

**Problem.** A language-model controller must remember a life of 100 simulated days and more, yet a
model's context is bounded, and `AC-10` requires that "character history does not require feeding all
historical events to a model; biography compression stays bounded while original event provenance is
retained". Memory must also respect `INV-13` (a controller never learns world truth it did not
perceive) and `INV-4` (the save is the world's truth, and only the world's).

**Options considered** (step-17 §4.3, §4.4; P4 §4):

```text
(a) an agent-memory framework (Letta, mem0, LangGraph memory,   none keeps Event-ID provenance by
    Graphiti, generative-agents reflection)                      construction; most put a model or an
                                                                 embedding service on the ingest path
(b) model-written summaries as the index                         provenance and coverage would depend on
                                                                 a model; AC-10 would need one
(c) a deterministic derivation of the perceived facts, with      chosen
    structural compression; a model may only rewrite text
(d) memory kept inside the world's save (save/cognition_cache/)   couples a controller's state to the
                                                                 world's truth (INV-4)
```

**Choice: (c).**

- **Input** is the seat's `perceived` stream only (`mineworld perceived` offline, the same audience
  function live), never `observation.events`, a save, the fact log or another seat's memory.
- **Levels.** L0: one record per perceived fact, rendered by a per-event-type renderer, with a generic
  renderer for every type no plug-in knows, so a new System Pack needs no cognition code to be
  remembered. L1: episodes, maximal runs of records at one place, with gaps of at most 1 800 s of world
  time, within one day (`at // 86 400`). L2: chapters, the closed episodes of one simulated week. L3:
  stable facts per counterpart. Every level carries the Event IDs it stands for, as ranges over the
  seat's perceived sequence; coverage is exact (every perceived id is in the open tail or in exactly one
  closed episode).
- **Bounds by construction.** The memory section a decision's context holds is at most 6 000 bytes,
  with per-section line and byte budgets and a 160-byte line, whatever the store holds. The store grows;
  the context does not. No forgetting in MVP-0 (QP4-11).
- **A model never writes provenance.** `StructuralSummarizer` writes every summary with no model, and is
  the only summarizer ingestion calls. An optional, off-by-default `embellish` pass may replace a
  summary's text with prose through the `summarize` tier of the gateway (`ARC-57`, `ARC-58`); it never
  writes, adds or drops a citation.
- **The store is the operator's, outside the save.** One SQLite file per seat at a path the caller
  gives, keyed by world instance, seat and self entity, refused by name on a mismatch.
- **Determinism is defined on a canonical dump** (every table in key order, JSON Lines, no float, no
  wall-clock value), not on SQLite file bytes, which differ across SQLite versions and platforms.

**Consequences.** `AC-10` holds with no model reachable. Deleting a store loses nothing: re-ingesting
the perceived history rebuilds it byte for byte (as a dump). Retrieval is lexical and structural only;
embeddings, importance scores and beliefs come later behind the retrieval seam (QS10-10).

**Revisit** if a world's perceived history makes the 6 000-byte section too small for a useful decision
context (a retune is a new record), if forgetting is added (MVP-1), or if embedding retrieval is adopted.

---

## DEP-37 — The memory store: standard-library `sqlite3`, with FTS5 as a match filter and integer ranking

**Date** 2026-10-10 · **Status** adopted by S10 PR P4 (no new dependency: CPython's `sqlite3` and the
SQLite it bundles) · **Approved by** the primary session (QP4-1, QP4-4, QP4-8, 2026-10-10) ·
**Licence** SQLite: public domain; `sqlite3`: the PSF licence · **Supersedes** step-17's placeholder
`DEP-S10-d` and the memory half of `DEP-S10-f` · **Relates to** `ARC-59`, `ARC-57` (`SqliteLedger`
uses the same module) · **Design** `pr-s10-p4-memory.md` §4, D-P4-3, D-P4-8, D-P4-11, D-P4-12, AP4-12

**Problem.** One seat's memory is about 54 000 records per 100 simulated days, written by one process,
read back in order and by range, and searched lexically for the words of a trigger. Records, roll-ups
and the cursor must commit together (the cursor is never ahead of what is durable). Bytes must be
reproducible, and everything must work with no model and no embedding service, on Linux, macOS and
Windows, with no native build step.

**Options considered** (facts checked 2026-10-10):

| Candidate | Licence; maturity | Verdict |
| --- | --- | --- |
| **stdlib `sqlite3`** | PSF; SQLite public domain; ships with CPython | **Adopted**: transactions, ordered reads, zero new dependencies |
| plain JSON Lines files per level | n/a | Rejected as the store: we would rebuild indexes, transactions and lexical search; kept as the **canonical dump** format |
| LanceDB | Apache-2.0; 0.30.x | Rejected for P4, reference only: a columnar vector database with native wheels, for a few small ordered tables; re-evaluate with embedding retrieval |
| DuckDB | MIT; mature | Rejected: analytical rather than per-row transactional; its full-text search is an extension that may be downloaded at run time, which a network-guarded test must never do |
| SQLAlchemy or another ORM | MIT | Rejected: an ORM for a handful of tables |
| LangGraph `SqliteStore` / checkpointers | MIT | Rejected: brings the LangGraph runtime and its key-value shape (`REUSE_POLICY.md` §3) |
| **SQLite FTS5** (same database) | public domain | **Adopted as a candidate filter only**: it decides which records match; its `bm25()` score is a float and is never used |
| sqlite-vec | MIT or Apache-2.0; pre-1.0 | Rejected for P4, reference only: needs embeddings and a loadable extension, which some CPython builds disable; re-evaluate with embedding retrieval, at 1.0 or later |
| rank-bm25 | Apache-2.0 | Rejected: float scores, and an in-memory index of the whole history |
| tantivy-py | MIT | Rejected: a second storage engine beside SQLite, for a match FTS5 already provides |
| Chroma and other vector stores | Apache-2.0 | Rejected: embeddings (QS10-10) |
| `LIKE` over notable records | n/a | **Fallback** only, behind the same query function, if a platform's CPython lacks FTS5 |

**Choice.** `MemoryStore` (`memory/store.py`) is a concrete class over `sqlite3`: one file per seat,
`journal_mode` left at `DELETE` (one file to copy or delete), every ingest call one `BEGIN IMMEDIATE`
transaction, ids stored as strings with a 20-character zero-padded ordering key (a u64 id can overflow
SQLite's signed `INTEGER`). `":memory:"` is the in-memory store for tests; there is no second
implementation. FTS5 (`tokenize='unicode61 remove_diacritics 2'`) indexes the gists of notable
records. A trigger's words become quoted FTS5 strings, never query syntax, and matching episodes are
ranked by the number of distinct terms matched (an integer), then recency, then the first Event ID.

**Why not ourselves.** Storage, transactions and lexical indexing are commodity; SQLite provides all
three in the interpreter we already require. What stays ours is the derivation and the ranking rule,
which no candidate provides with Event-ID provenance.

**Limitation accepted.** `unicode61` treats a run of CJK characters as one token, so CJK triggers match
only whole runs (QP4-8); a trigram or ICU tokenizer is a follow-up behind the retrieval seam. FTS5 must
be compiled into the interpreter; P4 checks it on every CI platform (`test_fts5_platform.py`).

**Revisit** if a platform's CPython ships without FTS5, if embedding retrieval is adopted (sqlite-vec or
LanceDB, re-evaluated then), or if CJK worlds become a demo target.

## ARC-82 — A doorway discloses the tags of the place it leads to: a door sign, never its contents

**Date** 2026-10-10 · **Approved by** the operator (Q1 of PR 13c's design, `.structured-coding/plans/mvp0/pr-13c-doorway-names.md` §11.1) · **Implements** [`MODULE_SPEC.md`](MODULE_SPEC.md), the movement system's disclosure · **Relates to** `ARC-26`, `ARC-75`, `ARC-62`, `ARC-45`

**Problem.** A person perceives only the place it stands in, and its doorways (`ARC-26`); the other places
are not perceived (`A-20`). A client therefore cannot tell which doorway leads to the café, cannot choose a
façade before entering, and labels every neighbour alike. The 2D client named a doorway by the last tag in
sorted order, which for most places is `public`.

**Choice.** Each entry of the `passages` disclosure gains one field, `to_tags`: the destination place's tag
set, in `Tag` order, always present (`[]` when the destination has none). It is built from current state
when the place is disclosed (`discloses`), by a disclosure-only type; the stored `Passage` and `Passages`
are unchanged, so no fact, persisted row or genesis changes. `component_type` stays `passages` and
`schema_version` stays `1`.

**The line.** The field carries tags and nothing else. It never carries the destination's identity beyond
`to`, its contents, occupants or activity. The audience of the disclosure does not widen: a person still
perceives no place as an entity except the one they stand in (pinned by the movement disclosure tests).

**Alternatives.** (a) Learn tags by entering the place (the fallback in step-13 QS12-7): keeps the street's
doorways generic, which the client needs to name them before entry. (b) Disclose the destination's full
record: widens the audience to what is inside. (c) Store `to_tags` in `Passage`: changes persisted state and
the genesis facts for a value that can be read from the destination at any time.

**Why.** Tags are authored world data, already shown to anyone who enters, and a door sign is what a
doorway shows from outside. Consumers that decode `passages` ignore unknown fields (the paced controller,
the agenda, the 3D slice, the protocol demo), so no controller decision changes; the 300-day digests of
`social-cafe` and `market-town` are the evidence (PR 13c).

**Consequences.** The 2D client names a doorway by the first tag, in order, that the shared wording names
(`place.<tag>`), else the destination's display name, else `outside`. A doorway without modelled positions is
not glued and its destination's tags are not learned by the 2D client; no drawn label is added on the street
(a follow-up finding).

**Revisit** if a place's tags must be hidden from people outside it, or if a door sign needs more than tags.

---

## DEP-44 — The resuming seat's reconnect loop is our own, over the adopted `websockets` transport

**Date** 2026-10-10 · **Status** adopted by S10 PR P3b (no new dependency; `uv.lock` unchanged) ·
**Approved by** the primary session at P3b's freeze (Q-P3b-1, 2026-10-10) · **Relates to** `DEP-25`,
`ARC-56` · **Design** `pr-s10-p3b-perceived.md` §4.3 (D-P3b-5, D-P3b-9, D-P3b-10), §4.5 · **Placeholder**
`DEP-P3b-a`

**Problem.** A Python seat that must not miss a fact rejoins after a dropped socket, a `lagged` refusal
or a server restart (`server/PROTOCOL.md` §§4.2, 5.5, 5.8). Which outcomes are retried, whether a rejoin
carries `resume`, which cursor it presents, and when a changed world instance ends the seat are protocol
decisions; the delays between attempts are the small remainder.

**Options considered** (facts read 2026-10-10 from the installed or cached distributions):

| Candidate | Licence (from its own metadata); version | Verdict |
| --- | --- | --- |
| `websockets`' reconnecting iterator, `async for connection in connect(url)` | BSD-3-Clause (`License-Expression`, 17.2, already `DEP-25`) | **Declined** for the protocol loop. `websockets/asyncio/client.py` `connect.__aiter__`: it backs off only when opening a connection fails (`process_exception` decides which failures); a connection that ends after it was yielded is reopened at once and resets the backoff, and nothing in it knows `join`, `resume`, the hold, the cursor or a world instance — the whole problem. Its default delays (`websockets/client.py` `backoff`) come from module globals read from `WEBSOCKETS_BACKOFF_*` environment variables at import (`D-P3-9`: the SDK reads no environment), the first `random.random() * 5` s from the unseeded global generator, then 3.1 s × 1.618 up to 90 s — past the server's default 30 s hold. A `reconnect_delays` callable can replace that sequence, but the sleep is `asyncio.sleep`, not injectable for a fake clock, and each retry is logged through the library's logger |
| `tenacity` | Apache-2.0 (`License: Apache 2.0`, 9.1.4) | **Declined**: a dependency for a capped exponential sequence of a few lines; the decision of *what* to retry stays ours either way, and it would put its own sleep and clock in the path the scripted tests replace |
| `backoff` | MIT (`License: MIT`, 2.2.1) | **Declined**, for the same reasons |
| **Our loop**: `mineworld_sdk/resuming.py` (the seat) and `mineworld_sdk/reconnect.py` (the table and the delays) | — | **Adopted** |

**Choice.** `ResumingSeat` owns the loop (`pr-s10-p3b-perceived.md` §4.5's table is the whole retry
policy). Its delays: the first attempt after a loss is immediate, then `first_delay × factor^k` capped
at `ceiling`, each scaled by a factor drawn from `random.Random(policy.seed)` (per seat), until
`give_up_after` wall seconds from the loss. The sleep and the clock are injectable (default
`asyncio.sleep` and the running loop's `time()`); no environment variable is read. The transport stays
`websockets` (`DEP-25`) with its keepalive; `session.py` remains the only importer of `websockets` in the
library.

**Revisit** when a second protocol client in Python needs the same loop: it then moves, unchanged, into a
shared module, not into a library.

**Dated note, 2026-10-14 (S10 P6b, C1): retry and breaker libraries declined.** For the LM controller's
layered retries (`ARC-73`) and its per-seat circuit breaker, the following are declined, and our own code
is built instead (`pr-s10-p6-lm-controller.md` §4). Licences were read on 2026-10-14 from each project's
PyPI metadata; none was installed or added, and `uv.lock` is unchanged.

| Candidate | Licence (PyPI metadata, 2026-10-14); latest version | Verdict |
| --- | --- | --- |
| `tenacity` | Apache-2.0; 9.2.1 | **Declined.** It owns the loop and the sleep, while the controller needs per-class budgets under one deadline and attempt cap, seeded jitter, an injected clock, and a record per attempt |
| `backoff` | MIT; 2.2.1 | **Declined**, for the same reasons |
| `stamina` | MIT; 26.1.0 (wraps `tenacity`) | **Declined**, for the same reasons as `tenacity` |
| `pybreaker` | BSD (classifier "BSD License", no licence field); 1.4.1 | **Declined.** It counts HTTP-call failures; the controller's breaker counts consult outcomes, excludes budget refusals, and runs its cooldown in simulated time |
| `aiobreaker` | BSD; 1.2.0 | **Declined**, for the same reason as `pybreaker` |
| `purgatory` | MIT; 3.0.1 | **Declined**, for the same reason as `pybreaker` |

The licence of `pybreaker` and `aiobreaker` is taken from their PyPI metadata as written; their licence
files were not read in this session.

---

## DEP-46 — Prior-art audit, October 2026: no shared-world or AI-NPC framework is adopted

**Date** 2026-10-11 · **Status** adopted (a decision not to depend; no dependency added) · **Approved by** the
primary session on the operator's instruction of 2026-10-11 ("learn from existing frameworks; do not add
dependencies on immature tools") · **Relates to** `REUSE_POLICY.md` §§8, 12, 17, 19; `ARC-25`, `ARC-58`, INV-9 ·
**Evidence** `docs/references/PRIOR_ART_2026-10.md` and the four detailed audits it links

**Problem.** A persistent shared world with composable behaviour, multiplayer and AI characters has been
built in part by several projects. `REUSE_POLICY.md` §12 requires a record when MineWorld keeps its own
implementation instead of adopting one.

**Options considered** (facts as read 2026-10-11; details and sources in the audits):

| Candidate | Licence | Verdict and the MineWorld rule it would break |
| --- | --- | --- |
| Croquet Microverse | Apache-2.0 | Reference only: authority runs on every client (INV-9); beta, no release |
| Decentraland SDK7 | Apache-2.0 | Reference only: no single-writer rule, per-client sync, publishing tied to its platform |
| Vircadia (native core / vircadia-world) | Apache-2.0 | Reject / reference only: deprecated lineage; pre-1.0 successor with state in Postgres rows and no event log (`ARC-25`) |
| Concordia | Apache-2.0 | Reference only now: a model referee resolves actions (VISION: models removable); no replay (`ARC-58`). Candidate for an optional controller adapter after Milestone D, under its own design |
| AI Town, Stanford Generative Agents | MIT; Apache-2.0 (unverified) | Reference only: the framework owns state; OpenAI-only or unseeded |
| Inworld, Convai | proprietary, hosted | Reject as controllers: hosted, not offline, no replay |

**Choice.** MineWorld keeps its own semantic layer (contracts, kernel, Systems, packs, persistence,
server, perception, seats, cognition contracts) and continues to reuse commodity infrastructure through
its existing `DEP-` records. Design ideas PA-1 … PA-11 are proposals, each entering through its own PR
design.

**Revisit** when any audited project reaches a stable release with a server-authoritative mode and an
event log, or when an optional controller adapter (Concordia) is designed after Milestone D.

---

## ARC-84 — The decision list is host-read content, not world state

**Date** 2026-10-12 · **Status** accepted (design, step-24); implemented by DL-a · **Approved by** the operator's step-24 rulings of 2026-10-12 (QS24-1, accepted as recommended) · **Relates to** `INV-1`, `ARC-61`, `ARC-64`, `ARC-33`, `ARC-53`, `AC-4`, `AC-5` · **Design** `.structured-coding/plans/mvp1/step-24-decision-and-memory.md` §4.1, §4.2

**Problem.** A world must choose which decision model drives each seat (rule, motivated, LM, director), with typed parameters, and may mix models per person. The choice could be seeded into world state as a `configure/` framework key, or kept outside the world. Seeding makes control a fact about a Person, which `INV-1` forbids, and it would trip the configure drift check at resume (`ARC-61`).

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) A World Pack file, `decisions.yaml`, read by hosts and never seeded; the operator may override it | **Chosen** |
| (b) A `configure/` framework key, seeded, with the drift check | Declined: makes control a world fact, and a controller change would alter world state at resume |
| (c) Operator-only configuration, with no file in the World Pack | Declined: the author's default casting of who decides how would be lost, and a world could not be run as authored |

**Choice.** The decision list is `decisions.yaml`, named by `world.yaml`'s `decisions:` key (`MODULE_SPEC.md` §4.1). It names models, never providers, model names, URLs or keys; the backend binding stays in the operator's `cognition.toml`. Hosts (`run`, `server --town`, the cognition process) read it; `--decisions FILE` and `--seat-model SEAT=MODEL` override it. Absent, every seat is `rule`, and every byte is as today. `run` prints the digest of the resolved list in its header. A save does not store the list; storing it as non-authoritative host metadata is deferred until a resume needs it (QS24-1).

**Consequences.** A controller choice never changes a Person or a world fact. The decision list adds one loader and no new pack type. Its assignments are validated against the same world with line and column, as `configure/` is.

**Revisit** when a resume needs the resolved list recorded in the save, or when a world needs a decision list that must travel with the save as authored content.

**Note, 2026-10-13 (S24, PR DL-a; `pr-dl-a-decision-list.md` SD-DLa-9, SD-DLa-14, SD-DLa-15; QDLa-5, QFGb-1).** Two refinements, implemented by DL-a, neither changing the choice. (1) `run` prints the digest of the resolved list **only when a decision list is in effect** (a `decisions:` key, `--decisions` or `--seat-model`), as `; decisions <digest> (<counts>)` at the end of its header line; with none, the header is byte for byte what it was, because the parity record keeps that line and an always-printed clause would move every reference world's baseline. (2) `decisions.yaml`'s `memory:` section is decoded structurally by the Rust reader (unknown keys, entry shape, seats and classes, policy-id spelling, `parameters` as a string-keyed mapping, each refused with line and column), resolved per seat seat > class > default, and passed through uninterpreted as a typed `MemoryPolicyChoice` by `mineworld decisions <world> --json` (format 1, `MODULE_SPEC.md` §8.1). The cognition process validates the policy id and its parameters when it starts (FG-b's `resolve`), so an unknown policy id fails then, naming the file, line and column it was written at, not at `run`'s load. One YAML parser reads the file; the cognition process reads JSON.

---

## ARC-85 — Behaviour extension points share one pattern: interface, carrier, selection, parameters, host

**Date** 2026-10-12 · **Status** accepted (design, step-24); implemented by DL-a (decision models), FG-a and FG-b (forgetting), WE-a (world evolution) · **Approved by** the operator's step-24 rulings of 2026-10-12 (§1.2 rulings 2 and 3) · **Relates to** `ARC-62`, `ARC-64`, `ARC-61`, `ARC-34`, `ARC-53`, `CLAUDE.md` §4 rules 2, 7, 11 · **Design** step-24 §§0, 1.2, 3

**Problem.** Three behaviour extension points need to be pluggable: person decisions, forgetting, and world evolution. Each needs a typed interface, a selection mechanism, validated parameters, and bounds that a plug-in cannot exceed. Left ad hoc, each would invent a loader, a registry and a pack type, and bounds would live in plug-in code.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) One new pack type per extension point | Declined: `MODULE_SPEC.md` §1 already has the carriers, and a new pack type is an operator question |
| (b) One pattern over the existing carriers: five parts, the interface, the carrier pack type, selection by content, typed parameters, and a host that enforces the bounds | **Chosen** |
| (c) A plug-in registry with untyped metadata and no host around the plug-in | Declined: it violates `CLAUDE.md` §4 rule 7 and lets a plug-in exceed a budget |

**Choice.** Each extension point has five parts: (1) a typed interface (Rust trait or Python Protocol, named types only); (2) a carrier that is an existing pack type (System Pack or Controller Pack); (3) selection by content, per world, per class (`ARC-64`), and per person, with precedence person > class > default, refused by name when unknown, and ambiguity refused at load; (4) parameters typed by the implementation, each with a bound and a default; (5) a host, which is code around the plug-in that enforces the hard bounds and invariants whatever the plug-in returns. Each point ships bundled implementations that are ordinary packs, none privileged, and two examples: one without a language model and one with one. No layer may require a model. An LLM never mutates world state.

**Consequences.** Adding a fourth extension point is a design question, not a new mechanism. A world's behaviour is selected by its `systems:`, `configure/` and `decisions.yaml`, and a user's rule is installed like any other pack.

**Revisit** when a proposed extension point cannot be expressed by the carriers above, or when a host bound is found to be enforceable only by a plug-in.

---

## ARC-86 — System-owned forgetting is lazy, pure and in the owning System; no model touches it

**Date** 2026-10-12 · **Status** accepted (design, step-24); implemented by FG-a · **Approved by** the operator's step-24 rulings of 2026-10-12 (§1.2 ruling 2(6); QS24-7 accepted as recommended) · **Relates to** `INV-7`, `INV-11`, `INV-12`, `ARC-25`, `ARC-62`, `ARC-28`, `ARC-65`, `CORE_CONCEPTS.md` §5.3 · **Design** step-24 §§3.2, 8.2

**Problem.** System-owned person state grows: relationship familiarity and regard, the acquaintance list, and the conversation's remembered lines. Forgetting it could be a timer-driven Process that writes a fact per day, a controller policy that forgets world state, or a pure rule inside the owning System.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) A Process that decays values on a timer and writes a fact per step | Declined: decay would add facts, journal rows and save growth every day, and it would break "no per-tick fact" |
| (b) A controller forgets world state for the world | Declined: a controller may not own a Component (`INV-7`); it can stop reading it, not forget it |
| (c) A lazy, pure rule in the owning System: each value is computed from `(value, last_contact, now)` where it is read, and stored state changes only inside reductions that already happen | **Chosen** |

**Choice.** Decay is a `DecayCurve` catalog entry (`ARC-62`) named in the owning System's `configure/<pack>.yaml`. The owning System clamps every curve result to `[floor, value]`, so forgetting only lowers a value toward its floor. The cap (`known_at_most`, default 150) evicts the lowest current familiarity inside the reduction that adds the acquaintance, emits `acquaintance-forgotten`, and unrelates `knows`. No model is involved. A level change caused by decay is observed lazily, and a biographical fact is emitted only by a reduction that touches the pair. Defaults change no reference world's bytes.

**Consequences.** Decay adds zero facts, zero journal rows and zero save growth, and replay reproduces it exactly. `last_contact` is already stored (`RelationshipValues::last_contact`), so no component encoding changes. The configuration payload grows, which bumps the System's version and refuses older saves by name (PA-3), and the new fact `acquaintance-forgotten` is declared. Read-time decay uses the operator-approved read-only clock `WorldRead::at()` (2026-10-12), a kernel addition available only during disclosure.

**Revisit** when a biographical decay must be visible without a reduction touching the pair, or when a System needs per-person state beyond `(value, last_contact)`.

**Note, 2026-10-13 (S24, PR FG-a; F-FGA-8, QFGA-4, QFGA-1) — two precisions.** (1) "Defaults change no reference world's bytes" means every fact and every run summary: a save's genesis journal row records the composition (each installed system's declaration and version), so raising `relationships` to 3 and `conversation` to 4 and declaring `acquaintance-forgotten` moves that one row, and `scripts/baselines.txt`'s `journal-300s` is re-recorded with a reason line while every later journal row, every fact and every summary stay byte-identical (`pr-fg-a-relationship-forgetting.md` FA-1). (2) `WorldRead::at()` is the world-time instant of whatever read holds it — a reaction's event instant, a validation's dispatch instant, `World::read()`'s `now()` — not something available only to disclosure; disclosure is its first user. It is never a wall clock (`INV-12`).

---

## ARC-87 — Controller-memory retention: host-computed importance, event-time recency, a bounded forgotten set, and host-enforced bounds

**Date** 2026-10-12 · **Status** accepted (design, step-24); implemented by FG-b (retention host, bundled policies) and FG-c (`lm-summarise`) · **Approved by** the operator's step-24 rulings of 2026-10-12 (§1.2 ruling 1; QS24-4, QS24-6, QS24-10, QS24-11 accepted as recommended, with QS24-6 as ruled) · **Relates to** `ARC-59`, `ARC-25`, `ARC-57`, `ARC-58`, `INV-4`, `INV-13`, `DEP-37`, P4's `D-P4-9`, `AC-10`, `DECISIONS.md` `QP4-11` (superseded) · **Design** step-24 §§1.2, 8.3, 10

**Problem.** The per-seat store grows without bound: 19.5 MB for one seat at 100 days (`pr-s10-p4-memory.md` R-P4-4). P4 deferred forgetting to MVP-1 (QP4-11). The operator reversed that on 2026-10-12, because without forgetting storage, memory and LLM tokens all explode. Retention must be bounded in every mode, replayable, and must not make the store depend on retrieval history, which `ARC-59` forbids.

**Options considered.** Importance: (a) rated by a model on a 1–10 scale, as in Generative Agents; (b) computed by the host from a structural table. Recency: (a) time since an item was last retrieved; (b) time since its event. Forgotten items: (a) their citations are deleted; (b) their citations join a forgotten set of ranges. Bounds: (a) enforced by each policy; (b) enforced by the host after every policy, including a hostile one.

**Choice.** (b) in all four places. The host computes `importance` (a per-kind table, the seat's current relationship level with each counterpart, and reinforcement by later facts) for every item, so every policy and every rebuild sees the same value. Recency is `now − last_at`. Dropped items' citations join the forgotten set, a sorted list of Event ID ranges, adjacent ranges merged, and beyond 256 ranges the oldest collapse into a watermark. The host enforces: a store of at most 1 MiB per seat, applying `consolidate` and then dropping the oldest L0 and L1 rows if a plan still exceeds it, and recording `bound-enforced`; at most 6 000 bytes of memory per model call (`D-P4-9`); and `calls_per_sim_day` (default 4, at most 24) inside P5's budget gate. An LLM-assisted policy may decide which items survive, provided that forgotten Event IDs stay resolvable and that a fallback to a no-LLM policy is recorded. `keep-all` exists only as a test fixture behind the host bound. Forgetting never deletes truth.

**Consequences.** A rebuild from a null cursor reproduces a no-LLM store byte for byte; an assisted store reproduces with its recordings, and without them the difference is visible. The P4 note that forgetting is MVP-1 is superseded. The P7 Milestone D test gains a 100-day store assertion.

**Revisit** when a retention policy needs retrieval statistics, or when AC-FG-1 shows that the 1 MiB bound is wrong for the reference worlds (the bound may be tightened, never loosened without a ruling).

**Note, 2026-10-14 (PR FG-b2; `pr-fg-b2-size-trigger.md` §1.1; operator rulings).** Two rulings revise the bound, neither changing the choice that the host, not a policy, enforces it. (1) "改成'快满了就提前整理'": a pass also runs early, after the fact that takes the store near its bound (a size pass), so the bound holds in a world whose perception exceeds a day's headroom — PR 12d's busier social-cafe reached 1 496 077 B before a daily pass. The trigger is measured exactly after every fact on the store's content, a function of the facts so far, so retention stays replayable and batch-independent. (2) "一个人物1mb太少了吧，我觉得起码20mb", then "默认 20 MB，世界可以调大": the per-seat bound is **20 MiB by default and a world may raise it**, per seat or class (`store_bound_mib` in `decisions.yaml` `memory:`), never below 20 MiB; the post-pass target is 75 % of it. This reverses, in the upward direction only, step-24 §1.2 ruling 2 (4) and FG-b's "not a parameter": the bound is now a world's choice above a fixed floor, still never a policy's. The 6 000-byte per-call memory ceiling (`D-P4-9`) is unchanged. "1 MiB" above reads "the seat's bound" from this date.

**Note, 2026-10-15 (PR S28-a; `pr-s28a-memory-home.md` §5.7; QS28-4, `ARC-99`, `ARC-109`).** The bound now measures the **seat view**: the seat's `SeatView.dump()` in schema 4's positional encoding (`ARC-101`), less its `meta` lines, with every record the seat holds counted in full however many seats of its home hold it. Nothing a home-mate perceives, keeps or forgets changes a seat's size (I-S28-3). FG-b2's numbers are unchanged: 20 MiB by default, raise-only, the target 75 % of the bound, the size trigger at the bound less a sixteenth.

---

## ARC-88 — World evolution: a deterministic `happenings` System Pack, and LLM directors that only propose

**Date** 2026-10-12 · **Status** accepted (design, step-24); implemented by WE-a (`happenings`); the director is implemented by WE-b (MVP-1) · **Approved by** the operator's step-24 rulings of 2026-10-12 (§1.2 ruling 3; QS24-2 ruled yes, narrowly, for the director) · **Relates to** `ARC-68` (weather), `ARC-34`, `ARC-63`, `ARC-62`, `DEP-46`, `PA-10`, `INV-6`, `INV-10`, `CLAUDE.md` §4 rule 2 · **Design** step-24 §§3.1, 9

**Problem.** A world needs changes beyond individuals: market days, festivals, prices, shop stock, news and weather-driven happenings. Such changes could be written into controllers, or an LLM could be given authority to change the world directly, as a game master would. Neither fits: the first hides world change in controller code, and the second makes the world invalid without a model.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) An LLM controller mutates world state, or resolves actions as a referee | Declined: an LLM never mutates world state (`INV-6`, ruling 3); the world must stay valid with the model removed |
| (b) A deterministic System Pack, `happenings`, generates scheduled and seeded happenings; an LLM director is a Controller Pack that submits `propose-happening` intents, which `happenings` validates and may refuse | **Chosen** |
| (c) A director drives a designated Person as a puppet | Kept as the alternative to the Organization seat; not chosen, because a puppet Person would appear to act in biography |

**Choice.** `happenings` (WE-a, MVP-0) owns `Happenings` on places and emits `happening-announced`, `happening-began` and `happening-ended`. Its configuration is a plain `ARC-61` configure file, `configure/happenings.yaml`, not an Interaction List section, because `ARC-63` forbids a second file beside a section. Happening kinds are content in that file. Generators are the `ARC-62` code catalog `HappeningGenerator`. Calendar entries are pure functions of the date; random entries are seeded integer draws from the content `seed:` in that file, since a world has no world seed; and its own `limits` (per day, per place, per actor per day) are caps it enforces. Other packs react to happenings only through their own opt-in configuration (for example, economy price modifiers). The director (WE-b, MVP-1) is a Controller Pack seated on an Organization, which is valid only for the director (QS24-2). It has no referee authority: it proposes, `happenings` validates the kind, place, window, limits and text, and it may refuse with a typed reason, and a refused proposal changes nothing. With the director removed, its seat runs its `fallback` (`none` by default), and `happenings` keeps its own schedule, so the world stays valid and still evolves.

**Consequences.** `CORE_CONCEPTS.md` §14 ("a Controller decides what a Person attempts") is amended by WE-b, not by this record, and WE-b's PR design states the exact wording. No `worldpack` change: `happenings` is an ordinary enabled system and its file is an ordinary configure entry.

**Revisit** when WE-b's Organization-seat rule is recorded (the amendment of `CORE_CONCEPTS.md` §14), or when a director must act on world state beyond proposals, which would require a new decision.

---

## DEP-47 — The utility core is our own; big-brain, bonsai-bt and dogoap are declined

**Date** 2026-10-12 · **Status** accepted (design, step-24); implemented by DL-b (no dependency added) · **Approved by** the operator's step-24 rulings of 2026-10-12 (the design's §6.2 and §7 recommendation, accepted under the standing instruction) · **Relates to** `REUSE_POLICY.md` §12 (a record when a candidate is rejected), `CLAUDE.md` §4 rule 16, `ARC-25`, `ARC-23`, `DEP-46` · **Design** step-24 §§6.2, 6.3, 7

**Problem.** The `motivated` decision model (`MODULE_SPEC.md` §5.1) needs a utility core: options scored by multiplying response curves over observed inputs, with a seeded weighted draw among the best options. Mature Rust crates exist for utility AI, behaviour trees and GOAP. `REUSE_POLICY.md` requires a record both when MineWorld adopts a substantial dependency and when it rejects one.

**Options considered.**

| Candidate | Verdict and reason |
| --- | --- |
| `big-brain` (utility AI) | **Declined.** It is built on Bevy's ECS, so adopting it forces Bevy's execution model on world state, which `CLAUDE.md` §4 rule 16 forbids; it has had 28 breaking releases |
| `bonsai-bt` (behaviour trees) | **Declined.** Standalone, but last active in early 2024, and it solves a behaviour-tree case MineWorld has not chosen: a tree encodes today's bands with more structure, not motivation |
| `dogoap` (GOAP) | **Declined.** Its core is standalone GOAP, which is not chosen for now (no System publishes effect declarations); revisit with DL-d |
| **Our own utility core**: integer response curves as data, following Dave Mark's published method, over our own types | **Chosen** |

**Choice.** Build the core ourselves. It is about 300 lines of integer arithmetic over our types. Scores are integers in 0 … 1 000, with a product of curves rescaled by compensation; there is no float anywhere. Its state is none: each consult is a pure function of `(seed, parameters, observation)`. Adoption would cost more than it saves, because the framework's execution model and its determinism would be someone else's.

**Consequences.** The core lives in `cognition/motivated`. A later GOAP model (DL-d) reopens this record only if it is needed; the candidates above are not re-evaluated from memory.

**Revisit** when Systems publish effect declarations for multi-step goals (DL-d), or when a candidate crate's core is shown to be independent of its engine.

**Note, 2026-10-14 (DL-b, the core as built; `.structured-coding/plans/mvp0/pr-dl-b-motivated.md` §§5, 15).**
- **Where it lives.** `cognition/motivated` (`mineworld-motivated`) holds the core and adds no external
  dependency.
- **Scoring.** Every option is offered only from what the observation discloses. Its score is its base weight
  multiplied, in the listed order, by its considerations. Each consideration is a piecewise-linear integer
  curve (≤ 24 points) over one input, adjusted by Mark's compensation factor `1 000 − 1 000 / k`. All
  arithmetic is floor division in `u64`, and the crate denies `clippy::float_arithmetic`.
- **Choice.** A seeded draw, weighted by score, chooses among the options within `top_band` ‰ of the best,
  using `Draw::salted` with the model's own salt. A door is re-drawn per commitment window.
- **Shared code.** The observation readers and request builders are shared with `rule` through
  `mineworld_rule_controller::{read, act}` (SD-DLb-1).
- **Measured result (social-cafe, 30 days, seed 7, every seat `motivated`).**
  - AC-DLb-3: night speech 0 ‰ (PASS).
  - AC-DLb-4: TV against ATUS 513 ‰ against the limit of ≤ 250, and `rule`'s TV is 382 (**FAIL**).
  - AC-DLb-5: night at home 972 ‰ (PASS).
  - AC-DLb-6: agenda kept 930 ‰ against `rule`'s 937 (PASS).
- **Why AC-DLb-4 fails.** The diagnosis (F-DLb-5) is a limitation of the frozen defaults, not a defect. Away
  from home `stay`'s base equals `greet`'s, considerations only lower a score, and the top band is 10 %. So a
  greeting is drawable only at 16:00–20:59 or at home in the evening, and the run speaks 97 lines in 30 days,
  against `rule`'s ≈ 10 159.
- **Status.** `motivated` therefore ships as an optional, non-default model. `rule` stays the default
  (QS24-3), and the revision of model or criterion is the operator's (R-DLb-6).

---

## DEP-48 — Daily-rhythm calibration: BLS ATUS A-3 public-domain tables, committed as derived integer tables through an offline fit tool

**Date** 2026-10-12 · **Status** accepted (design, step-24); implemented by DL-b · **Approved by** the operator's step-24 rulings of 2026-10-12 (QS24-9 accepted as recommended) · **Relates to** `ARC-68` (the `weather-fetch fit` pattern), `DEP-31`, `ARC-23`, `R-S24-9` · **Design** step-24 §§6.6, 7, 15

**Problem.** `motivated` needs daily rhythms calibrated from real data, so that its sociability and home pull follow a measured hour-of-day curve rather than arbitrary constants. The candidate sources have different licences. The American Time Use Survey's published tables are public domain (BLS), with credit requested. IPUMS ATUS-X extracts require registration, and redistribution needs approval. MTUS requires registration, and its redistribution terms are unstated. HETUS microdata is research-only.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) Use IPUMS, MTUS or HETUS microdata | Declined: redistribution is not permitted, and repository licensing would be a risk |
| (b) Commit BLS ATUS A-3 (percent of the population engaging in selected activities by time of day) as derived integer tables with a credit line; an offline tool `routine-fit` produces the curve set and optional routine sections; the run never reads the source | **Chosen** |
| (c) Hand-set the curves | Declined: the arbitrary constants are the problem this record solves, and `ARC-23` requires numbers fixed before measurement from a named source |

**Choice.** Option (b). Only the aggregate public-domain tables are committed, as derived integer tables, and their credit line is in the repository. Microdata never enters the repository. The `routine-fit` tool runs offline, and its output is committed content. HETUS aggregate tables on Eurobase (CC BY 4.0 for content, acknowledgement required) are reserved for a later European world.

**Consequences.** The acceptance of rhythm (AC-DL-4) is measured against the fitted curve, with its tolerances written in the PR design before the run. A future calibration from a different source is a new record.

**Revisit** when a European world needs calibration from HETUS aggregates, or when the BLS terms of use change.

**Note, 2026-10-14 (DL-b, the calibration as built; `.structured-coding/plans/mvp0/pr-dl-b-motivated.md` §7,
§15).**
- **Edition and rows.** The edition is `atus-a3-2025`: BLS ATUS Table A-3, 2025 annual averages (QDLb-3). The
  six rows are Sleeping, Eating and drinking, Working, Socializing and communicating, Leisure and sports, and
  Purchasing goods and services.
- **The committed file.** `cognition/motivated/data/atus-a3-2025.tsv` holds the rows as per-mille integers
  (percent × 10, so the conversion is exact), with a `zero_markers` line and a per-row `checksum` line. The
  credit and public-domain statement are in `cognition/motivated/data/NOTICE`. No microdata is committed.
- **Derivation.** It is in the crate and pure, and there is no offline `routine-fit` tool: the routine
  generator is deferred (QDLb-1). `curves::from_a3` maps each row to a 24-point curve with
  `y(h) = ⌊v(h) × 1 000 / max v⌋`, and `restless` = `1 000 − rest`. The table is compiled in and derived when
  parameters are decoded, so a run never opens a file.
- **Transcription.** The values were transcribed from the HTML table, read through a summarising fetch. The
  reviewer's three-cell check against the PDF is still open (§15, C2).
- **The acceptance oracle.** `A(h)` is the socializing row normalised to sum 1 000, read from the TSV. It is
  not the model's curve output. The measured results are in `DEP-47`'s note of the same date.

---

## ARC-89 — Sealed log blocks: journal and fact rows of completed spans move unchanged into compressed blocks, and history is read as a stream

**Date** 2026-10-12 · **Status** accepted (design, step-24 FG-d; operator QSR-1 of 2026-10-10 and QS24-5 of 2026-10-12, building SR-b in MVP-0); implemented by FG-d (SR-b), `SAVE_FORMAT` 4 · **Approved by** the operator (QSR-1, QS24-5); the concrete design is `.structured-coding/plans/mvp0/pr-sr-b-log-compression.md` (design PR #178; frozen 2026-10-12) · **Relates to** `ARC-25`, `ARC-27`, `ARC-49`, `ARC-81` (amended, rule 1), `DEP-2`, `DEP-43`, `ARC-59`, `AC-9`, `AC-12`, `QS24-4` (save growth bound) · **Design** `pr-sr-b-log-compression.md` §§6.1, 6.2 (D-SRB-1 … D-SRB-11)

**Problem.** After `ARC-81`'s snapshot pruning, a Market Town save is almost entirely its event log. The log grows by about 3.75 MiB per simulated day on walking and 5.59 MiB per day with bodies, against a bound of 0.25 to 0.5 MiB per day (QS24-4). Facts and journal entries compress 18 to 40 times when compressed together in spans, but only 2 to 2.5 times one row at a time. Readers of history (`perceived`, `inspect`, `biography`) also load the whole log, peaking at 1.3 to 1.5 GiB on a 300-day save (`F-S24-1`). `ARC-25` requires that the log stay history, never derived and never rewritten, and that replay compare bytes.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) Delete or rewrite old rows, or keep only the newest span | Declined: destroys history; violates `ARC-25` and `AC-9`, whose causes must resolve |
| (b) Compress each row alone | Declined: 2 to 2.5 times, far short of the bound |
| (c) Seal completed spans into blocks, moving rows unchanged; readers stream one block at a time | **Chosen** |
| (d) Archive old segments to separate files (FG-e) | Deferred to MVP-1 (FG-e), after streaming readers exist; it builds on this record |

**Choice.** Let `A = 64 · interval` be `ARC-81`'s anchor interval (4 096 revisions by default). At each anchor commit `a` of that lattice, with `a ≥ 2A`, the live rows of the revisions in (`sealed_through`, `a − A`] move, byte for byte, into one sealed block in the same transaction as the anchor. A block holds two zstd frames (journal and facts) over a length-prefixed framing of the rows, at one fixed level for blocks (level 9), as `DEP-43` requires of every stored kind. Rows keep their key columns and bytes; a row is either live or inside exactly one block, never both. The backend stores opaque blocks and never decodes a fact (`DEP-2`). Replay compares re-executed entries and facts with the logged bytes, decoded from a block. Every reader of history streams blocks. The save format becomes `SAVE_FORMAT` 4; an older save is refused by name, with no migration (QSR-3).

**Consequences.** `ARC-81` rule 1 ("`facts` and `journal` are only ever inserted into") is amended to "inserted into, and moved unchanged into a sealed block". No other row is deleted. Stored bytes remain a function of pack, seed, age, pace and code (`ARC-27`), so `ARC-49`'s parity hashes continue to hold on all three platforms. The per-day save bound is checked nightly (`AC-SAVE-1`), not by refusing facts.

**Revisit** if a block-level read cannot meet the streaming bound, if a sealed span must be rewritten for a correction (which this record forbids), or when FG-e's archive segments need a manifest format, which a successor record decides.

**Note 2026-10-13 (implementation, SR-b; design frozen 2026-10-12, rulings QSRB-1 … QSRB-8).** The record's `a − interval` in the first draft meant `a − A`, as stated above. The decisions the implementation holds to, in full in the design's §6.1:

```text
D-SRB-1   rows move, bytes do not change: a journal or fact row is live or in exactly one block, with the
          same key columns and bytes; ARC-81 rule 1 is amended to say so, and no other deletion of a log
          row exists
D-SRB-2   the sealing rule, a pure function sealing(n, interval, sealed_through) beside ARC-81's retired:
          when n mod A = 0 and n ≥ 2A, seal (sealed_through, n − A]; blocks are (kA, (k+1)A], sealed one
          span late, so resume never reads a block and every block ends at an anchor (QSRB-1)
D-SRB-3   one table: sealed(first_revision PK, last_revision, first_event_id, last_event_id,
          max_action_id, journal BLOB, facts BLOB); spans contiguous, the live tail after the newest
D-SRB-4   the framing (persistence/src/block.rs), little-endian: journal rows (revision, at, action id,
          length, entry) in revision order; fact rows (event id, revision, length, fact) in EventId order;
          decoding checks contiguity, spans, ordering and trailing bytes, and names the block's span
D-SRB-5   each frame is zstd level 9 (QSRB-2), checksum and content size on, single-threaded, no
          dictionary: block bytes are a function of the row bytes on every platform, which AC-8 checks
D-SRB-6   the backend's history is two cursors, revisions_after and facts_after, each inside one read
          transaction, holding one decoded block at a time and paging the live tail; journal_after is
          removed; facts_of, last_facts, head and highest_action_id read both storages
D-SRB-7   replay, verify, verify_from and resume iterate the revision cursor and compare the
          re-executed entry and facts with the cursor's bytes
D-SRB-8   every production whole-log reader streams: the server's backfill under its existing seam,
          perceived, inspect, biography, run's fingerprint and genesis instant (F-S24-1, persistence and
          CLI side; the server's seam itself is routed to the server lane, QSRB-6)
D-SRB-9   SAVE_FORMAT 4; snapshots no longer reference the journal; a format-3 save is refused by name,
          no migration (QSRB-3)
D-SRB-10  mineworld inspect prints one `log` line: blocks, their span, stored bytes and row bytes, the
          live tail
D-SRB-11  a seal is verified before the rows go: the block's frames are decoded and compared with the
          source rows and the deleted counts with the block's, or the commit fails (WorldAheadOfSave)
```

**Ways of building it compared** (design §5): blocks of `A` revisions in a `sealed` table on the anchor lattice (chosen); per-row zstd (2 – 2.5×, misses QS24-4 by 5×); per-row zstd with a trained dictionary (9 – 11×, plus a stored, versioned dictionary; a dictionary buys 2 % at block scale); blocks as files beside `world.sqlite` (two-store atomicity, deferred to FG-e); SQLite page compression (rejected in `ARC-55`); a columnar container (no column structure in opaque rows); a fixed 4 096 independent of the interval (a second lattice and a test-only knob, QSRB-1).

**Bounds** (operator QSRB-4): 30-day Market Town save ≤ 48 MiB; 300-day ≤ 96 MiB with bodies and ≤ 80 MiB walking; growth ≤ 0.25 MiB per simulated day (CI, every PR), and the nightly `AC-SAVE-1` at 0.5 MiB per day until stable; peak memory of a history reader on a 300-day save ≤ 256 MiB; a seal ≤ 100 ms per block.

**Revisit triggers added:** a block-level `AC-8` difference on any leg; FG-e moving blocks out of the file; a world whose span exceeds 64 MiB of raw rows.

---

## ARC-96 — Agent players are clients: `mineworld-agent`, a Python client of the public protocol, with MCP confined to one module of it

**Date** 2026-10-14 · **Status** accepted (step-27 design, operator QS27-1 … QS27-10 of 2026-10-14); written by S27-c C0, which builds the transport-free core (`mineworld_agent.Player`); the MCP module is S27-d's · **Approved by** the operator (step-27 §14.1, every recommendation accepted as written) · **Relates to** `ARC-34`, `ARC-56`, `ARC-67`, `ARC-75`, `DEP-24`, `DEP-26`, `DEP-44`; `DEP-56` (the `mcp` dependency, S27-d), `ARC-97` (seat grants and the per-seat budget, S27-a) · **Design** `.structured-coding/plans/mvp0/step-27-agent-players.md` §§3, 4.1–4.4, 5, 6; `pr-s27c-player-library.md` · **Placeholder** `ARC-S27-a`

**Problem.** Humans and agents are both to be first-class players of a MineWorld world (the operator, 2026-10-14). An **agent player** is a Person whose Controller is an agent MineWorld does not run — Claude Code, Codex, a script — reaching its seat through a client process it does not own (step-27 F-S27-5). The question is where that agent's adapter lives, in which language, and what it may know, such that the server cannot tell an agent from a person (`ARC-56`), omniscience stays impossible (`INV-13`), no MCP concept enters a world, kernel, server or cognition contract, and removing the adapter leaves a valid world.

**Options considered** (step-27 §4.2; external facts read 2026-10-14, step-27 §17):

| Option | Verdict |
| --- | --- |
| **P** — a Python client under `clients/agent/` (`mineworld-agent`), on `mineworld-sdk`'s `ResumingSeat` (`DEP-44`), with the official `mcp` Python SDK confined to one module of it | **Chosen.** It reuses the proven resuming client (resume, `lagged` recovery, exactly-once perceived facts behind a caller-owned cursor, at-most-once submits) and Pydantic (`DEP-24`), from which the tool schemas also come; the Python CI legs already run real-server tests on three platforms. Cost accepted: the release bundle ships no Python (QS27-8) |
| R-in — Rust `rmcp` inside the server process | **Rejected.** The server would speak a second client vocabulary beside `join`/`submit`/`leave`, which `server/PROTOCOL.md` §2 forbids by construction; MCP types would enter `server/`; an adapter fault would share the world's process |
| R-beside — `rmcp` in a separate Rust client binary | **Declined for now.** The resuming client would be written a second time in Rust; revisit if the operator's live check (S27-g) shows that installing `uv` is the main obstacle for players of the bundle (QS27-8) |
| T — the TypeScript MCP SDK | **Declined.** TypeScript is for web clients and admin tooling only (`ENGINEERING_STANDARDS.md` §3), and the resuming client would be written a third time |
| H — no MCP library, our own JSON-RPC | **Rejected.** Rebuilding a maintained official SDK of a moving protocol needs a reason (`REUSE_POLICY.md` §12), and there is none |

**Choice.**

1. The adapter is a **client** in `server/PROTOCOL.md`'s sense, not a Controller Pack: it decides nothing. The Controller is the external agent; the adapter renders that Person's own `Observation` for it and turns its choices into `ActionRequest`s, one seat per connection, through `ResumingSeat`. It lives under `clients/`, is installed into no world, and no World Pack, decision list or cognition configuration names it.
2. **MCP is a skin over a typed library.** `mineworld_agent.Player` (`seats`, `join`, `observe`, `inspect`, `events`, `wait`, `say`, `walk_to`, `move`, `stop`, `attempt`, `clock`, `leave`) takes and returns Pydantic models (strict, frozen, `extra="forbid"`), the same models the MCP module publishes as each tool's `inputSchema` and `outputSchema`. Only that one module imports `mcp`; no other package of the repository depends on it or names an MCP concept.
3. **No REST agent API.** A non-MCP agent uses the `Player` library or the public WebSocket protocol, both first-class; a REST surface would be a second schema of the same operations, and a stateful seat cannot be held by stateless calls without re-inventing MCP's handles.
4. **Names the adapter knows**, like every client (`ARC-34`): `talk`'s utterance payload and the payloads of `move`, `walk-to` and `walk-step` (free-form, so never offered complete). Every other action is reached through `attempt` on a complete affordance, unchanged. The adapter evaluates no requirement and adds no rule; its only local refusals are what the newest observation does not offer (exactly `offers.request`'s check), a stale offer, an unknown reference and an unknown handle, each with its own code. It paces a walk as the 2D client does, one `walk-step` per wall second while the seat's own `walking` is disclosed (`ARC-67`, `ARC-75`), and plans no route.
5. **Credentials are launch configuration.** The invite (or, after S27-a, a seat grant) is given to the adapter when it starts; it is never a tool argument, a result, a log line or an error, and neither is the welcome's resume secret, which only the optional state directory holds.
6. **In-world text is typed untrusted.** Every string written inside the world — an utterance, a display name, an item name — reaches the agent only as `WorldText { text, by, kind }`, and in the text rendering it is quoted between `<<<world` and `world>>>` with every `<<<` and `>>>` inside it neutralised.

**Consequences.** The server is unchanged by this record (S27-a's grants and budget are `ARC-97`'s, and apply to every connection alike). `mineworld-sdk` gains one additive read-only property, `ResumingSeat.clock` (S27-c, QS27c-1), and otherwise is reused as it is. Removing `clients/agent/` changes no World Pack, System, contract, test outside the adapter's own, or recorded digest. P6's `Situation` and the adapter's `View` solve one presentation problem for two readers; when both have shipped, a review decides whether to lift the shared re-location rule into `mineworld-sdk` (step-27 R-S27-6).

**Revisit** if a named agent can speak neither MCP nor the library nor the public protocol (then a further thin module over `Player`, never a server change), or under QS27-8's trigger for a single-binary adapter.

## DEP-50 — The executable guide: `pulldown-cmark` and `snapbox` as dev-dependencies of `mineworld-cli`

**Date** 2026-10-14 · **Status** accepted (design, step-25 §4.2 option E2); implemented by PR S25c (dev-dependencies only, no runtime edge) · **Approved by** the operator's step-25 rulings of 2026-10-14 (QS25-1 to QS25-10; the design's §4.2 recommendation accepted under the standing instruction) · **Relates to** `ARC-90`, `DEP-29`, `DEP-49`, `CLAUDE.md` §4 rule 16, `REUSE_POLICY.md` §§1–4, 11–12 · **Design** `.structured-coding/plans/mvp0/step-25-creator-docs.md` §§4.2, 8, 16 · **Specification** [`GUIDE_FORMAT.md`](GUIDE_FORMAT.md)

**Problem.** Every page in `docs/guide/` must be executed against the real `mineworld` binary, in order, on Linux, macOS and Windows, so that a page which stops working is a red test. The runner needs a CommonMark parser that yields fenced blocks with their info strings and line numbers, and an output matcher with line elision and in-line wildcards.

**Options considered** (verified 2026-10-14 with `cargo info` and the repository metadata, step-25 §16).

| Option | Verdict |
| --- | --- |
| E1 `trycmd` 1.2.1 (MIT OR Apache-2.0) | Declined as the runner. Its per-case sandbox does not model a tutorial whose steps are cumulative; its file steps need a second copy of every file in an `.in/` directory. Its matching rules are adopted through `snapbox`, which is the library `trycmd` itself uses. |
| E2 `pulldown-cmark` 0.13.4 (MIT, not archived, last push 2026-10-12, MSRV 1.71.1) plus `snapbox` 1.2.2 (MIT OR Apache-2.0, not archived, last push 2026-10-04, MSRV 1.85) | **Adopted.** A test in `mineworld-cli` walks each page, writes file blocks, runs command blocks with the binary and matches output. Both are test-only and never enter a build. |
| E3 mdBook `test` | Declined: it runs Rust code blocks; the guide's blocks are YAML and shell. |
| E4 Python runner with `markdown-it-py` | Declined: the binary's tests already run on all three operating systems under `cargo test`; a Python runner would need a new job and a new dependency. |
| E5 Test the example world only | Declined: a page can describe what the world no longer does. |
| Hand-written Markdown scanner | Declined: a CommonMark fence rule (the fence length, indentation, and `~~~` fences) is exactly what a maintained parser already gets right, and a bespoke one would drift from what GitHub renders. |

**Choice.** `pulldown-cmark` (parsing) and `snapbox` (matching) as `[dev-dependencies]` of `mineworld-cli`, each with a workspace entry in the root `Cargo.toml` carrying this record's id. Neither crate is named outside `tools/cli/tests/`. Feature sets are chosen at C1 and recorded there.

**Consequences.** The guide runner adds two crates to the test graph of `mineworld-cli` only. `cargo deny check licenses` must still pass (both are on the existing allow-list: MIT, Apache-2.0). The runner's TOML reading of `features.toml` uses the `toml` crate already present in `Cargo.lock`, so it adds no new crate; the workspace entry is recorded when C1 adds it.

**Revisit** if `pulldown-cmark` or `snapbox` leaves maintenance, or if the guide's block grammar needs a construct CommonMark cannot express (then the grammar changes first, in `GUIDE_FORMAT.md`).

---

## ARC-90 — Creator documentation: a generated reference that cannot drift (reference part)

**Date** 2026-10-14 · **Status** accepted (step-25 design); the reference part implemented by PR 25a; the guide part (`docs/guide/` executed by CI, the onboarding test) is added to this record by PR 25c · **Approved by** the operator's rulings QS25-1, QS25-2, QS25-6 of 2026-10-14 (step-25 §13.1) · **Relates to** `ARC-31`, `ARC-33`, `ARC-61`, `ARC-63`, `ARC-66`, `DEP-49`, `CLAUDE.md` §2.1 · **Design** step-25 §§4.1, 5; `pr-s25a-reference.md`

**Problem.** Every file a creator writes is decoded by a typed value whose owner validates it, but what each file may say was written down by hand in `MODULE_SPEC.md` §4.1 for the framework's own fields and nowhere at all for the sections, configurations and Interaction List sections the packs own. A hand-written field list drifts the first time a pack adds a field (risk R-3 of `overall.md`).

**Options considered.** R1 derive a JSON Schema from each decoding type and render it (chosen); R2 a MineWorld describe trait and derive macro (reinvents `schemars`); R3 scrape rustdoc JSON (nightly toolchain, sees types but not serde's behaviour); R4 a hand-written reference with a guard (an undocumented field is never caught); R5 schema-first code generation (inverts ownership: the pack's type would no longer be the owner's validation, contradicting `ARC-31` and `ARC-61`).

**Choice.**

1. **The reference is derived from the types that decode the files** and is never edited by hand. `docs/reference/` is written by `mineworld reference --out DIR`; `mineworld reference --check DIR` regenerates it in memory and exits 1 naming every file that differs, is missing or is extra. CI runs the check, and a change to `docs/reference/` alone still runs it.
2. **The SDK requires a schema** (QS25-2): `AuthoredSection::Authored` and `PackConfiguration::Configuration` are bound by `schemars::JsonSchema`, so a pack whose section or configuration has no schema does not compile. `parameters!` writes the schema of its block itself, with each field's default, bounds and doc comment.
3. **The installed set answers for every pack.** `installed!` gains `section_schema`, `configuration_schema` and `interaction_declarations`, defined by the macros a pack already writes (`owns_section!`, `configures!`, `interactions!`); the generator walks `Capability`'s list and names no pack.
4. **A field's description is the first paragraph of its doc comment**, written for an author, with its unit; a field with none fails generation. Later paragraphs are for developers and reach neither the Markdown nor the JSON Schemas.
5. **The command is read-only with respect to worlds**: it loads no World Pack, and its output is a function of the build alone — the installed set's order, fields in name order, no timestamp and no commit hash, `\n` line endings on every OS. It writes only into its `--out` directory and refuses one holding a file it did not generate.
6. **English only** (QS25-6): the reference is generated from English doc comments and has no `*.zh-CN.md` mirror.
7. **The reference describes this build.** A build with another installed set generates a different reference (`ARC-33`, `ARC-66`); the checked-in one describes the repository's own installed set.

**Consequences.** A specification states shape and rules; field-by-field detail is the generated reference's (`MODULE_SPEC.md` §§4.1, 4.2). `mineworld-contracts` gains no dependency: a contract-typed field names its schema through `mineworld_authoring::schema`. A third-party pack derives `JsonSchema` on its section and configuration types (`acme-fishing` 0.1.1 does).

**Revisit** when a public stable SDK contract is declared (the bound is then part of it), or when the reference must describe a build other than the repository's own.

---

## DEP-49 — JSON Schema derivation: `schemars` 1.x; `clap-markdown` declined

**Date** 2026-10-14 · **Status** accepted; implemented by PR 25a · **Approved by** the operator's ruling QS25-2 and the step-25 freeze of 2026-10-14 · **Relates to** `ARC-90`, `DEP-10`, `DEP-11`, `DEP-22`, `REUSE_POLICY.md` §§1–3, 11–12 · **Design** step-25 §§4.1, 16; `pr-s25a-reference.md` §§2–3

**Problem.** `ARC-90` derives a JSON Schema from every type that decodes a World Pack's files, including serde's attributes (`deny_unknown_fields`, `default`, `rename`, `transparent`, `try_from`), and renders the `mineworld` command's own surface.

**Options considered.**

| Option | Verdict and reason |
| --- | --- |
| **`schemars` 1.2.3** (MIT; `GREsau/schemars`, not archived, pushed 2026-10-12; MSRV 1.74) | **Chosen.** Reads serde's attributes and doc comments; manual `impl JsonSchema` for custom decoders; `#[schemars(with, example)]`. Adds `schemars_derive`, `serde_derive_internals`, `dyn-clone` and `ref-cast`, all on the workspace's existing `syn` 3 |
| Our own derive (R2) | Declined: reinvention of a maintained crate |
| rustdoc JSON (R3) | Declined: nightly; does not see serde's behaviour |
| Schema-first with `typify` (R5) | Declined: inverts the decoding type's ownership |
| `clap-markdown` 0.1.5 for the CLI page | Declined: a 0.1 crate for about 80 lines over `clap`'s `Command` model, which the build already depends on (`DEP-11`) |

**Choice.** `schemars = { version = "1.2.3", default-features = false, features = ["derive", "std"] }` in `[workspace.dependencies]`, named by `mineworld-authoring`, `mineworld-sdk`, `mineworld-packages`, `mineworld-worldpack`, the System Packs that own a section or configuration, and `mineworld-cli`. The `preserve_order` feature is **not** enabled: it would turn on `serde_json/preserve_order` for the whole workspace and could change bytes a world serializes (`INV-S25-5`); the reference lists a type's fields in name order instead. `mineworld-contracts` never names the crate. The CLI page is rendered by walking `clap`'s `Command`.

**Consequences.** `cargo deny check licenses` passes with the added crates (MIT, or MIT OR Apache-2.0). A third-party pack depends on `schemars` from the registry at the version the build's lock holds.

**Revisit** when `schemars` 2 is released, or if its maintenance stops.

---

## ARC-91 — Appearance is a perceivable trait of a Person, owned by the `appearance` System Pack

**Date** 2026-10-14 · **Status** accepted (step-26 freeze, operator Q-S26-11); implemented by S26-a · **Approved by** the operator (Q-S26-11, Q-S26-4, Q-S26-9) · **Relates to** `ARC-31` (sections), `ARC-33` (installed set), `ARC-46`, `ARC-92`, [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §4.2 (Traits), [`MODULE_SPEC.md`](MODULE_SPEC.md) §4.1 · **Design** `.structured-coding/plans/mvp0/step-26-cast.md` §§4.1–4.2; `pr-s26a-appearance.md`

**Problem.** Every person must be drawn as themselves in both reference clients, and in any third client. A client can read only a person's entity id, tags, disclosed components and display name. None of them identifies an appearance: an id is a creation-order counter that renumbers when a place is added; a display name is absent without `naming`, can be shared, and would make a face change when a name changes or, if names are ever gated by acquaintance (`ARC-31`), when a stranger is introduced; a tag is semantic content that Systems and controllers read. Nobody owned an appearance, so each client invented one from a hash, and nine sprites came to serve twelve people.
**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) `hash(entity id)` modulo a pool | Declined: distinct only by luck; cannot say "Alice looks like this" |
| (b) The entity id listed in a Presentation Pack | Declined: ids are creation-order counters; an unreviewable list of numbers |
| (c) The display name | Declined: absent without `naming`; may collide; makes appearance depend on what the observer knows |
| (d) A presentation tag (`look:alice`) | Declined: pollutes a semantic vocabulary every consumer reads, with no owner |
| (e) A disclosed `look` component owned by a System Pack | **Chosen** |

**Choice.**

1. **`appearance` is a System Pack** on the `naming` pattern: it owns a person file's `appearance:` section (`{ look: <key> }`, unknown keys refused), states a public `appeared` fact at genesis, reduces it into the `look` component (`{ "look": "<key>" }`, schema 1) and discloses that to every observer who perceives the person. It depends on nothing and declares no action or process.
2. **A look is an opaque key**: 1–48 bytes of `a`–`z`, `0`–`9` and `-`. It names an appearance; it is not a mesh, a file, a texture, a colour, a height or any other renderer concept. A Presentation Pack resolves it (`ARC-92`).
3. **No descriptors.** Apparent age, stature, build, skin and hair are not world facts: no System consumes them. They live in the casting brief, in the Presentation Pack. A System that later needs stature makes it a `bodies` section.
4. **Appearance never depends on what the observer knows.** Disclosure is gated by perception only.
5. **Single ownership.** A later clothing, ageing or disguise System changes a look by having this pack state `appeared` again; no other pack writes `look`.

**Consequences.** A world without the pack, or a person without the section, has no look, and every client draws them with its fallback. The default worlds' keys are name slugs for review readability. A controller sees `look` as it sees any component; the memory renderer stores `appeared` generically and never shows the key as a name.

**Revisit** if names become acquaintance-gated (the default keys then become opaque codes in one data migration per world, R-S26-9), or when a System needs an appearance descriptor.

---

## ARC-92 — The cast binding lives in the Presentation Pack; one resolution order; the fallback is the extras pool

**Date** 2026-10-14 · **Status** accepted (step-26 freeze, operator Q-S26-8, Q-S26-9); specified by S26-a; read by the 3D client from S26-c2 and by the 2D client from S26-e · **Approved by** the operator (Q-S26-8, Q-S26-9, Q-S26-11) · **Relates to** `ARC-3`, `ARC-9`, `ARC-46` (amended, Note 1), `ARC-91`, `DEP-16`, [`MODULE_SPEC.md`](MODULE_SPEC.md) §6.1 · **Specified in** [`clients/3d-spike/PRESENTATION.md`](../clients/3d-spike/PRESENTATION.md), [`clients/2d/PRESENTATION.md`](../clients/2d/PRESENTATION.md) §3.1 · **Design** `.structured-coding/plans/mvp0/step-26-cast.md` §§4.3–4.4, 8

**Problem.** `ARC-91` gives every person a disclosed look key. Something must turn a key into a character in 3D and a sprite set in 2D, without the world learning anything about art, and with a fallback for a person whose look a pack does not know that is never a cruder kind of body.

**Choice.**

1. **3D: `characters/cast.yaml`** in a 3D Presentation Pack, in YAML's JSON-compatible subset: `format` (1), `looks` (look key → character id), `extras` (character ids: the fallback and decorative pool), `default` (the character used when no world is connected). A character id names `characters/cast/<id>/` in the pack; its meshes stay resources of the client that imports them (the 3D client cannot load a skinned, retargeted model from a runtime pack today).
2. **2D: a `look:<key>` role** in the `people` binding set. It is a disclosed component value, the same class of input as the tag in `facade:<tag>`, never a world key.
3. **One resolution order in both clients.** A person whose look the pack binds is drawn as that character or sprite. A person with no look, or one the pack does not bind, is drawn as `extras[posmod(hash(id string), len(extras))]` in 3D and as `person:<n>` by the same hash in 2D. A pack with no extras (3D) or no `person:<n>` (2D) uses its `default` (3D) or plain drawing (2D). With no pack, plain drawing. The observer is resolved the same way, so a player is drawn as the Person they occupy; the 2D `player` role is only the fallback for an observer with no bound look.
4. **The fallback is never a cruder model class.** Every `extras` entry is a character made by the same pipeline and held to the same standard as a named person (step-26 §6, which S26-b1 records as `ARC-94`). In the default pack, the café street's extras are the lakeside cast (Q-S26-8, E-2), so no person of the present world appears twice.
5. **A pack decides nothing.** A binding changes how a person looks, never which requests a client sends, and a look is never shown as text.

**Consequences.** The default 3D pack ships an empty but valid binding until its characters exist; `scripts/check_cast.py` checks its structure. Adding a world's cast is an edit to one pack file, never to a client or a world.

**Revisit** when 3D Presentation Packs become loadable at runtime (the binding then also names the meshes), or when a world that is neither the café town nor the lake needs its own extras (E-1).

## ARC-72 — The LM controller is built from typed parts; model text is a proposal; a request is built only through `mineworld_sdk.offers`

**Date** 2026-10-14 · **Status** accepted for S10 P6b (`cognition/lm-controller/src/mineworld_cognition/controller/`)
· **Approved by** the primary session's freeze of `pr-s10-p6-lm-controller.md` (2026-10-14; operator
QP6-1, QP6-3, QP6-7; QP6-12 on the number) · **Relates to** `ARC-56`, `ARC-58`, `ARC-59`, `ARC-34`,
`ARC-85`, `INV-10`, step-17 I-10, I-15 · **Design** `pr-s10-p6-lm-controller.md` §§5.1–5.4, §5.9 ·
**Prior art** `docs/references/PRIOR_ART_2026-10.md` PA-10, PA-11 (reference only)

**Problem.** A language model asked "what does this character do next" returns free text. Treated as a
command, that text can name people who are not there, invent offers, forge memories, or move a person. A
controller that is one prompt and one parse has no place to put the rules, the tests or the failure
handling.

**Options considered:**

```text
(a) one prompt, one parse; the reply is sent as the action            open to every failure above
(b) an agent framework (Concordia, LangGraph, AI Town) as the loop   the framework owns the execution and
                                                                      the memory model; reference only (DEP-46)
(c) small typed parts (persona, situation, memory view, choice
    prompt, parser, validator, intent builder); the reply is a         chosen
    proposal the owning contracts check
```

**Choice: (c).** The controller is a Python Controller Pack. Its parts are frozen Pydantic models or pure
functions, with no clock, randomness, environment or other seat's store read. A model's answer becomes a
`talk` utterance or the unchanged payload of a complete affordance the server offered, and nothing else.
The controller builds an `ActionRequest` only through `mineworld_sdk.offers`, which refuses any affordance
not offered by the newest observation. It never builds a movement request (`ARC-32`, step-17 I-15). The
prompt holds no entity id and no provider concept; the request's canonical JSON is the cassette key
(`ARC-58`).

**Consequences.** A model that "decides" anything else has no representable proposal (`INV-10`). Every
answer is validated before it becomes an intent. A new social choice needs a `SocialChoice` value and a
validation row, not a change to the kernel or the server. Nothing in the world reads the controller.

**Revisit** when a second model-driven controller needs the same parts, which then move into a shared
module, or when the offer protocol (`ARC-34`) gains a movement affordance that the controller must decide.

---

## ARC-73 — Failure containment: layered retry above a retry-free gateway, a per-consult deadline and cap, a per-seat breaker, and fallback by handing the seat back

**Date** 2026-10-14 · **Status** accepted for S10 P6b · **Approved by** the primary session's freeze
(2026-10-14; operator QP6-7 on the thresholds, QP6-6 on the two P5a changes, QP6-8 and QP6-9 on the
fallback and the hold) · **Relates to** `ARC-57`, `ARC-58`, `ARC-56`, `ARC-72`, `DEP-44`, `ARC-33`,
step-24 §8.6 · **Design** `pr-s10-p6-lm-controller.md` §§5.5–5.8, §5.13, §4

**Problem.** A model fails in three different ways: the transport fails (rate limits, server errors, timeouts),
the content is unusable (invalid JSON, a schema mismatch, an unknown person), or the world refuses a valid
proposal (an offer that moved). Each deserves its own response. The world must never wait for a model,
and every model call must be charged to a budget.

**Options considered:**

```text
(a) one retry loop in the gateway                       breaks ARC-57: the gateway would make calls the ledger
                                                        has not budgeted for the retry's class
(b) one retry policy for all failures                   content and transport need different budgets
(c) three retry layers in the controller host, above a
    gateway that never retries; each layer has its own
    pluggable policy; one deadline, one call cap, one
    per-seat breaker; fallback by handing the seat back  chosen
(d) a retry or breaker library                          DEP-44: they own the loop and the clock; declined
```

**Choice: (c).** Layers: transport (`exponential-backoff`, honouring `Retry-After`), content (`fixed`, two
re-asks with the validation error), semantic (`fixed`, one re-plan from the newest observation). Every model
call is one gateway call and one ledger charge. A consult ends in exactly one terminal outcome. A
per-consult deadline (20 wall s), a cap of five calls, and the P5 budgets bound it. A `failure-rate` breaker
per seat (window 20, minimum 10, more than 20 %, cooldown one simulated hour) stops a failing model. When
the controller cannot decide, the seat is handed back with `leave` (only where the welcome said
`took_over: "hosted"`), and the server's fallback model drives it. Where no in-server fallback exists, the
seat stands for that consult and sends nothing (QP6-8). The hand-back is host state and writes no fact.
Counters go to the controller's own log.

**Consequences.** The world's facts stay valid under any model behaviour, because the world validates every
request at dispatch (step-17 §3.6). A failing model costs at most five calls per consult and one handback
per breaker trip. The fallback share is an acceptance criterion (AP6-4, AP6-14), not an assumption.

**Revisit** when a second model seat needs a different breaker or retry shape, which becomes a registered
policy rather than a code change, or when an idle handback (QP6-3's follow-up) is decided.

---

## ARC-97 — Seat grants and a per-seat request budget: narrower credentials and a fair bound, both host state

**Date** 2026-10-14 · **Status** accepted (design frozen 2026-10-14); implemented by S27-a · **Approved by** the operator's step-27 rulings QS27-1 … QS27-10 and the primary session's QS27a-1, QS27a-2 (2026-10-14) · **Relates to** `ARC-40`, `ARC-41`, `ARC-44`, `ARC-67`, `ARC-75`, `DEP-14`, `DEP-57`, `INV-S27-1`, `INV-S27-4`, `INV-S27-5`, `INV-S27-13`, [`NETWORKING.md`](NETWORKING.md) §9, [`server/PROTOCOL.md`](../server/PROTOCOL.md) §§4.1, 4.2, 5.5, 8, 10, 11 · **Design** `.structured-coding/plans/mvp0/step-27-agent-players.md` §§6.4, 7.2; `.structured-coding/plans/mvp0/pr-s27a-seat-grants-budget.md`

**Problem.** Revision 2 has one level of player trust: whoever holds the server's invite may take any seat, a connected or held one included (`ARC-40`'s and `ARC-44`'s recorded limitation). A host who wants to let a friend, or an agent player, onto one seat must hand over the right to take every seat. And nothing bounds how many requests a connection may send: a client that floods `submit`s makes the world allocate an `ActionId`, journal and answer each one, and a client that is not the reference client is under no cadence at all.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) Change the invite's meaning (per-seat invites, or an invite with a seat list) | Declined: every client and launcher reads the invite as "any seat"; changing it is a protocol change for every existing user |
| (b) Accounts or durable identity | Declined: out of MVP scope (`NETWORKING.md` §9); bans and identity are a later design |
| (c) **Seat grants**: host-generated credentials, each naming seats and an optional take-over right, presented in the existing `join.invite` field | **Chosen** |
| (d) A grant token given on the command line | Declined: process lists show argv; a grant is generated, like a generated invite |
| (e) Bound requests per connection | Declined: a reconnect or a takeover would refill it; the seat is what is being protected |
| (f) **A per-seat token bucket** kept by the seat table, refusing `rate_limited` before an `ActionId` is allocated | **Chosen** |
| (g) A per-action embodied cadence ("`walk-step`: at most 1 per wall second") declared by a System Pack | Deferred: a design change to `ARC-67`/`ARC-75`, ruled a separate design after S27-a (QS27-7) |

**Choice.**

1. **Grants.** `mineworld server WORLD --grant SEATS[:take-over]` (repeatable; `SEATS` one seat or several joined by `+`) generates one token per grant from the operating system's random source (`DEP-14`) and prints it once, on its own startup line. A grant token is never given, never persisted, and appears in no frame, fact, save, `/status` or admin answer. A `join` presents it in `invite`; check 2 admits the invite or any grant in constant time over every credential (a wrong one is one answer, `unauthorized`); a new check 4a refuses a seat the grant does not name with `not_granted`; check 6 refuses `take_over: true` on a connected or held seat with `not_granted` unless the grant carries take-over. Resuming one's own hold is always allowed; a hosted seat is joined without a flag. The invite, the admin token and every grant are distinct, or the server refuses to start. `GET /admin/sessions` reports which credential admitted a session — `{ kind: "invite" }` or `{ kind: "grant", seats, take_over }` — never a token and never a Controller kind.
2. **The budget.** Each seat has a bucket of `R` requests per wall second sustained and `B` burst, keyed by the seat (a reconnect, resume, takeover, kick or release does not refill it), spent by every `submit` a connection sends after the `paused` check and before allocation, and refused `rate_limited` with `retry_after_ms` (the wait rounded up to a whole millisecond, ≥ 1). It is the same for every connection and not applied to in-server controllers, which the host paces at its consult cadence. `--request-budget R/B` sets it; `0` means none.
3. **The numbers, from evidence (C0, rule fixed before measuring: `R = max(10, 2 × measured p99)`, `B = 2R`).** The real 2D client against the real `market-town` (`tools/cli/tests/request_rate_2d.rs`, Godot 4.7.2, macOS, load average ≈ 215–235): walking 40 strides along the street, 43 answers over 49 wall s, per-second p50 1 / p99 3 / max 3; walking with the keys out of a home, p50 1 / p99 2 / max 2. Analytic bound: one request in flight, a `move` every 0.5 m at 1.6 m/s — 3.2 a second. The 3D client has no request-timestamped harness: **INCONCLUSIVE** (ruled QS27a-2); its documented cadence is a `move` every 0.5 m (≈ 6.2 a second at its 3.10 m/s jog) with no more than one per 0.08 s (12.5 a second, turning in place). **`R = 10`, `B = 20`.** The walking trace is committed as `server/tests/traces/walker-2d.json` and replayed by the server's tests, which must see no `rate_limited`.
4. **Landing within revision 2.** `not_granted` reaches only a join that presented a grant (a credential that did not exist before), and `rate_limited` only a seat that spent the budget, which the reference clients' cadence does not; `retry_after_ms` appears on `rate_limited` only. So `protocol` stays 2, and every client in the repository — the server, the Python SDK, the 2D client's final-refusal list and the shared catalog — learns both codes in the same pull request. An older SDK fails closed on them (a malformed frame ends the session), never silently. The next revision must make a client treat an unknown refusal code as a generic refusal (`PROTOCOL.md` §10).

**Consequences.** `ARC-40`'s and `ARC-44`'s "one level of trust" limitation is amended by adding a narrower credential, not by changing the invite. Grants and buckets are host state (`ARC-40`): none is journaled, states a fact, moves a revision or appears in a save. Nothing distinguishes an agent from a human (`INV-S27-4`, `INV-S27-13`): a grant binds whoever presents it. `R = 10` still lets a client step a walk ten times a second; bounding embodied speed is QS27-7's separate design, built on this bucket. The 3D client's turning-in-place ceiling (12.5 a second, no request in flight held) exceeds `R` sustained; the burst absorbs about eight seconds of continuous turning, and a refused report is shown, not retried.

**Revisit** when the 3D client gains a request-timestamped harness (re-measure; QS27a-2), when QS27-7's per-action cadence is designed, or when public hosting needs per-address join limits (`DEP-14` option (n)).

---

## DEP-57 — The per-seat request bucket: `governor`, driven from the seat table's injected `Instant`

**Date** 2026-10-14 · **Status** accepted; dependency added in S27-a C5 · **Approved by** the step-27 frozen rule (§8, `DEP-S27-b`): adopt `governor` if C0's prototype shows its clock can be driven from the seat table's injected `Instant`, otherwise our own bucket · **Relates to** `ARC-97`, `DEP-14` (option (n)), `REUSE_POLICY.md` §§11–12, 15 · **Design** `pr-s27a-seat-grants-budget.md` §§4, 5.3, 9 C0

**Problem.** `ARC-97`'s per-seat budget needs a token bucket (GCRA or equivalent) whose time is the seat table's injected wall `Instant`, so every transition is testable with literal instants and no sleep (the seat table's existing model for holds), and whose state is keyed by seat inside the table, which iterates deterministically.

**Options considered** (both directions).

| Option | Verdict |
| --- | --- |
| (a) **`governor` 0.10.4 (MIT), one direct limiter per seat** (`RateLimiter::direct_with_clock`), with a clock the seat table sets to its injected `now` before each check | **Chosen** — the prototype passed (below) |
| (b) `governor`'s keyed limiter (`hashmap_with_clock`) | Declined: its keys live in a `HashMap` the seat table cannot iterate deterministically, duplicating the table's own key set |
| (c) Our own integer bucket (~40 lines in `seats.rs`) | Declined since (a) works: GCRA's arithmetic, its rounding and its burst semantics are what a mature crate has already had reviewed; "writing it ourselves feels cleaner" is not a reason |
| (d) `tower_governor` / `tower::limit` | Declined: they limit HTTP requests in the transport, where no per-seat key exists, not WebSocket frames per seat |

**Prototype (C0, 2026-10-14), outside the repository build** (a scratch crate with `governor = { version = "=0.10.4", default-features = false, features = ["std"] }`). Facts verified from the crate source and a test with no sleep:

```text
governor::clock::Reference is implemented for std::time::Instant   src/clock/with_std.rs:23 (feature "std")
Reference's bounds   Sized + Add<Nanos> + Eq + Ord + Copy + Send + Sync + Debug   src/clock.rs:61-62
Clock's bounds       none beyond `type Instant: Reference; fn now(&self)` (and Clone for the limiter);
                     a clock over Rc<Cell<Instant>> — neither Send nor Sync — compiles and works, so a
                     clock owned by the world thread is fine
test                 Quota::per_second(10).allow_burst(20): 20 checks pass at t0; the 21st fails with
                     NotUntil::wait_time_from(t0) == 100 ms exactly; at t0 + 100 ms − 1 ns still
                     refused; at t0 + 100 ms one passes and only one                    PASS
licence              crate licence field "MIT"; repository boinkor-net/governor, MIT License
```

New crates in `Cargo.lock` (normal edges, `default-features = false, features = ["std"]`): `governor`, `futures-timer`, `lock_api`, `nonzero_ext` (Apache-2.0), `parking_lot`, `parking_lot_core`, `portable-atomic`, `spinning_top`, `web-time` — every one MIT, Apache-2.0, or a dual of them, all in `deny.toml`'s list. (C5 note: adding the dependency to the workspace also put `redox_syscall` (MIT) in `Cargo.lock`, a Redox-only target edge of `parking_lot_core` that no supported platform compiles; the prototype's `cargo tree` for the host target did not show it.) The rest (`futures-util`, `hashbrown` 0.16, `smallvec`, `cfg-if`, `libc`, …) are already compiled.

**Choice.** (a). `governor` is named only in `server/src/seats.rs` (the bucket, behind the seat table's own `spend`), so its types do not cross the crate (`REUSE_POLICY.md`: a thin MineWorld-owned interface).

**Revisit** if `governor`'s clock model stops accepting an injected `Instant`, or if per-address join limiting for public hosting (`DEP-14` option (n)) is adopted, which would reuse this dependency.

---

## DEP-59 — The world thread's CPU time per tick: `cpu-time`, not our own `unsafe` over `libc` and `windows-sys`

**Date** 2026-10-15 · **Status** accepted; dependency added in the F-tick-1 fix · **Approved by** the operator, 2026-10-15 ("服务器加测'线程真正干活的时间'" — the server also measures the time the thread actually works), amending D-SB12 · **Relates to** `ARC-42` (note of 2026-10-15), `DEP-29` note, `REUSE_POLICY.md` §§11–12 · **Design** `.structured-coding/plans/mvp0/step-12-server.md` §16.11 (D-SB12, F-tick-1 note)

**Problem.** CP-B4's tick bound needs the CPU time the world thread spends inside one tick, on macOS, Linux and Windows. The standard library has no thread CPU clock. Every platform has one behind a C call: `clock_gettime(CLOCK_THREAD_CPUTIME_ID)` on Unix, `GetThreadTimes(GetCurrentThread(), …)` on Windows, both `unsafe` to call from Rust.

**Options considered** (both directions).

| Option | Code we own | Verdict |
| --- | --- | --- |
| (a) **`cpu-time` 1.0.0** (MIT OR Apache-2.0, tailhook/cpu-time): `ThreadTime::try_now()` / `try_elapsed()`, `clock_gettime` over `libc` on Unix, `GetThreadTimes` over `winapi` 0.3 on Windows | One call site, no `unsafe` | **Chosen** |
| (b) `libc` (Unix) and `windows-sys` 0.61 (Windows) as direct `cfg` dependencies — both already in `Cargo.lock` transitively | Two `cfg`-gated `unsafe` blocks (~30 lines), the `FILETIME` arithmetic, a `windows-sys` feature set (`Win32_System_Threading`, `Win32_Foundation`), and a Windows branch no developer machine here compiles | Declined: the same `unsafe` calls as (a), written and kept by us |
| (c) `libc` on Unix and one `extern "system"` declaration of `GetThreadTimes` from kernel32 (the `DEP-29` note's form) | As (b), less one dependency edge, plus a hand-written FFI signature with pointer arguments | Declined: `DEP-29`'s call took two integers and touched no memory; `GetThreadTimes` writes four structures through pointers, which is exactly what a reviewed binding is for |
| (d) Process CPU time read from outside (`ps`, `/proc`, `GetProcessTimes` in the test harness) | A per-platform parser in the tests | Declined: the whole process, not the world thread; one total, not a per-tick tail |
| (e) `sysinfo`, `nix` | — | Declined: `sysinfo` is a whole-system inspector far beyond one clock; `nix` is Unix-only |

**Facts verified at source** (crate sources in the cargo registry, 2026-10-15): `cpu-time` 1.0.0 is 346 lines (`lib.rs`, `clock_gettime.rs`, `windows.rs`). Its `ThreadTime` is a `Duration` with a `PhantomData<Rc<()>>`, so it is neither `Send` nor `Sync` and cannot be read on one thread and compared on another. `try_now` returns `io::Result` (the panicking `now` is not used). Licence field `MIT/Apache-2.0`, with `LICENSE-MIT` and `LICENSE-APACHE` in the package. New crates in `Cargo.lock`: `cpu-time`, and on Windows targets only `winapi` 0.3.9, `winapi-i686-pc-windows-gnu` 0.4.0 and `winapi-x86_64-pc-windows-gnu` 0.4.0, each `MIT/Apache-2.0`. `cargo deny check licenses sources bans` passes with them. `libc` is already compiled on Unix.

**Choice.** (a). It is the least code we own that is correct on all three platforms, and it has no `unsafe` of ours. `cpu-time` is named only in `server/src/runtime/world.rs` (`TickStart`, `TickTimes`), so its types do not cross the module. If the clock cannot be read, a tick's CPU time is recorded as its wall time, an upper bound.

**Accepted limitations.**
- **Maintenance.** `cpu-time` 1.0.0 and `winapi` 0.3 are not actively developed. The calls they wrap are stable OS interfaces, and the crate is small enough to vendor or replace with (b) in an afternoon if either stops building.
- **Windows resolution.** Windows charges thread time at the scheduler's clock interval (commonly 15.6 ms), not per instruction. A sub-millisecond tick therefore reads as 0 or one interval, and a tick has to burn at least four intervals (about 63 ms) to be sure to read over 50 ms there. The bound keeps its meaning — steady-state ticks measured ≤ 0.6 ms of CPU (step-12 E-SB9) — but the planted CPU mutation is evidence on macOS, not Windows.

**Revisit** if `cpu-time` or `winapi` stops building on a supported target, or if the server comes to need more than one OS clock (then (b) behind a MineWorld module).

---

## ARC-105 — Towns declare `settlement:`; every town has a station group and a landmark, carried by role tags on plain Places

**Date** 2026-10-15 · **Status** accepted (step plan approved 2026-10-15); implemented by S29-a · **Approved by** the operator, directly in the primary session, 2026-10-15 (step-29 Q-S29-2, Q-S29-3, Q-S29-7, Q-S29-9) · **Relates to** `ARC-53`, `ARC-82`, `ARC-26`, `ARC-37`, `ARC-38`, `ARC-107`, [`MODULE_SPEC.md`](MODULE_SPEC.md) §4.1, [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §6 · **Design** `.structured-coding/plans/mvp0/step-29-station-landmark.md` §§3, 4

**Problem.** The operator requires a train station in every town, and a signature landmark for each (2026-10-15). No world has a station. The framework also has no way to say whether a world is a town, so it cannot refuse a town that lacks either.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) Infer "town" from a tag on some place | Declined: a world that forgot the tag would silently stop being a town |
| (b) Every world is a town, with no opt-out | Declined: test yards are not settlements, and a station there would be fake content |
| (c) A `settlement:` key that the loader requires | Declined: every inline fixture world would change, with no safety beyond (d)'s gate |
| (d) **A `settlement:` key, required by `mineworld validate` / `packs validate` and validated by the loader whenever present** (the `ARC-53` precedent) | **Chosen** |
| (e) A new Entity type or System for stations | Declined: Places, tags and passages already express it. Behaviour arrives with S21's `transport` |

**Choice.**

1. `world.yaml` gains `settlement: { kind: town, station: <place>, landmark: <place> }` or `settlement: { kind: none }`. The World Pack loader owns it; no System or kernel does.
2. **Role tags** have fixed meanings: `station`, `stop`, `platform`, `ticket-office` and `landmark`. A station is the group of `station`-tagged Places joined by passages to the declared hall. It contains a `stop` + `platform` Place, which is S21's stop, and a `ticket-office` Place. The origin of a stop is its platform anchor.
3. Ten named refusals cover the contract: `SETTLEMENT_UNDECLARED`, `SETTLEMENT_MALFORMED`, `TOWN_STATION_UNKNOWN`, `TOWN_LANDMARK_UNKNOWN`, `TOWN_STATION_UNTAGGED`, `TOWN_LANDMARK_UNTAGGED`, `TOWN_STATION_NO_STOP`, `TOWN_STATION_NO_TICKET_OFFICE`, `TOWN_STATION_SCATTERED`, `TOWN_STATION_UNREACHABLE` and `TOWN_PLATFORM_ANCHOR`. The last two are checked when `movement` and `bodies`, respectively, are enabled.
4. Reference worlds:
   - `market-town`, `social-cafe` and `lakeside` are towns.
   - The station is the landmark in the first two.
   - Lakeside has a small halt, and `pier` is its landmark.
   - `bodies-yard` declares `none`.
5. Adding the station's places, an organization and routines shifts allocated ids. This takes **one recorded digest re-baseline**, in S29-a's C5, after 12d (Q-S29-7).

**Consequences.** There is no kernel, contract or System change. A role tag grants no behaviour. S21's `transport` attaches `StopInfo` to the existing stop, and `fares` attaches its ticket shop to the `ticket-office` Place. `atlas` may seed `contains` from the station group.

**Revisit** when regions (S21) allow more than one station per world, or when a settlement kind finer than `town` is needed.

---

## ARC-106 — The town's landmark station: the Santa Fe Depot, at real size, at the street's end, built with the town's own method

**Date** 2026-10-15 · **Status** accepted (step plan approved 2026-10-15); implemented by S29-d, S29-e and S29-f · **Approved by** the operator, directly in the primary session, 2026-10-15 (step-29 Q-S29-1, Q-S29-4, Q-S29-5, Q-S29-8, Q-S29-10) · **Relates to** `DEP-8`, `ARC-102`, `ARC-105`, `ARC-107`, [`VISUAL_SLICE.md`](VISUAL_SLICE.md) §§3, 4, 11, [`ART_DIRECTION.md`](ART_DIRECTION.md) §4 · **Design** step-29 §§6, 8

**Problem.** The operator asked for a station that is the town's signature building, modelled on San Diego's Santa Fe Depot. It must sit in the far field's view, at street level, and be reachable on foot from spawn. The real depot is 790 m from the slice's origin, and the walkable area has a radius of 60 m. A building that can be seen but never reached is forbidden (`VISUAL_SLICE.md` §4).

**Options considered.** (P-1) **At the street's west end, at real size, with the distance compressed (chosen).** (P-2) At the real 790 m, with a ≈ 700 m walkable approach: declined, because it is region scale, needs ≈ 700 m of frontage, and breaks the perf budget. (P-3) A silhouette at 790 m plus a separate near station: declined, because it draws one Place group twice and the far one is unreachable.

**Choice.**

1. **Placement.**
   - The forecourt replaces the west parapet.
   - The arch stands on the street's axis, with the east façade at about x = −62 m. This mirrors the real Broadway axis ending at the depot.
   - The compression from 790 m is recorded in the far field's `source`.
2. **Size.** The hall is at real size, 51.8 × 16.8 m, with a tile wainscot 2.59 m high (Wikipedia; Gaslamp Foundation). The tower height is measured from **HABS CA-1965**'s public-domain photographs, which may be committed to the 3D pack's `references/`. CC BY-SA photographs are linked, never committed.
3. **Method and style.**
   - One method: the slice's procedural builder and its shipped CC0 materials. The glazed tile of the domes and wainscot is our own shader.
   - The depot is the **one sanctioned Mission Revival exception** beside the lakeside-referenced street, rendered with the same materials, palette, light and precision.
4. **Budget.**
   - ≤ 6 MB of new data in `3d.pck`, with a target of 0 new textures.
   - At most +0.8 ms p95 in each existing view, and the absolute 16.7 ms and 2 048 MB gates in the new station views.
   - A reduced hall is the reserved lever, used only at a material stop.

**Consequences.** `VISUAL_SLICE.md` §4 is amended: the station becomes a required third enterable building, and the forecourt becomes the west termination. RL-e0's far-field shots are re-shot as an intended change. A rendered landmark check (LM-1, LM-2) runs on all three OSes. The 2D client binds `facade:station` to a sprite from the existing image pipeline.

**Revisit** if the hall view cannot meet 16.7 ms with the built-in levers, or when S21 regions make a real-distance station reachable by travel.

---

## ARC-107 — The far field's `filled` slot: a reachable building recorded where it stands

**Date** 2026-10-15 · **Status** accepted (step plan approved 2026-10-15); implemented by S29-c · **Approved by** the operator, directly in the primary session, 2026-10-15 (step-29 Q-S29-1); dated note on `ARC-102` · **Relates to** `ARC-102`, `ARC-105`, `ARC-106`, [`MODULE_SPEC.md`](MODULE_SPEC.md) §6.1.1 · **Design** step-29 §6.7; `.structured-coding/plans/mvp1/pr-rl-e0-far-field-mandatory.md` §5.1.6

**Problem.** RL-e0's slots are `unplaced` or `reserved`. A reserved slot must lie beyond `boundary.reach_m`. The station now stands inside the boundary (`ARC-106`), and a town's far field must agree with its declared station and landmark.

**Choice.**

1. Format 1 gains a slot state, `{"state": "filled", "place", "bearing_deg", "distance_m", "footprint_m", "source"}`. The change is additive.
2. A filled slot is the one far-field entry allowed inside the boundary. Massing keeps clear of its footprint.
3. In a town's 3D far field, both slots are `filled`, and their places equal `settlement.station` and `settlement.landmark`. Otherwise the file is refused `FAR_FIELD_SLOT_TOWN { pack, world, slot }`, at validate and at client load. Under `kind: none`, `filled` is refused (`FAR_FIELD_SLOT`).
4. Social Café's 3D `reach_m` grows from 60 to 100 m.

**Consequences.** The Rust and GDScript validators change together, and RL-e0's agreement corpus gains one file per new refusal. Lakeside's 3D slots carry design content until 16f draws Lakeside in 3D.

---

## ARC-108 — Station fixtures show only disclosed state

**Date** 2026-10-15 · **Status** accepted (step plan approved 2026-10-15); implemented by S29-e, S29-f and S29-g · **Approved by** the operator, directly in the primary session, 2026-10-15 (step-29 approval) · **Relates to** `ARC-67`, `ARC-105`, `INV-S27-2`, step-21 §§5.3, 5.7 · **Design** step-29 §§5, 9.4

**Problem.** Before trains exist (S21), a station could easily pretend: an invented timetable, an open ticket window, people waiting for a train that cannot come.

**Choice.**

1. **One pure rule, from disclosed components.** It is written once in `clients/shared/station/station_state.gd` and used by both clients; the agent view renders the same Observation.
   - **The departures board** lists lines only when a `departures` record is disclosed. Otherwise it reads "No service yet".
   - **The ticket window** is open only when its place discloses a `shop` listing a `ticket`-tagged kind.
   - **The kiosk** is open only when the hall discloses a `shop`.
   - **The clock** shows the disclosed world time, or nothing.
2. **No fakes.** No routine label waits for a train before `transport` exists, and no train is drawn before S21's carriage.

**Consequences.** When S21 lands, the board and window come alive with no client change. S21 must disclose `departures` to observers in `station`-tagged places, because a board in the hall cannot perceive the platform's `StopInfo` (a non-preclusion item for RT-c).

---

## DEP-60 — Reserved: one new CC0 texture for the station, only if a reviewed frame needs it

**Date** 2026-10-15 · **Status** reserved; withdrawn at S29-d's merge if unused · **Approved by** the operator, directly in the primary session, 2026-10-15 (step-29 Q-S29-10: ≤ 6 MB, target 0 new textures) · **Relates to** `DEP-8`, `ARC-106` · **Design** step-29 §6.4

**Problem.** The depot is built from the shipped Poly Haven CC0 set. A reviewed frame might show that a material cannot be read from that set.

**Choice.** At most two Poly Haven CC0 1k textures, each ≤ 3 MB and each checked under `DEP-8`. Each is added only on that evidence, and S29-d records it here, replacing this entry's status. If none is added, this entry is marked withdrawn and the number is not reused.

---

## DEP-56 — The official MCP Python SDK (`mcp`), for the stdio adapter only, registered on its low-level `Server`

**Date** 2026-10-15 · **Status** accepted (design frozen 2026-10-15); dependency added in S27-d C1 · **Approved by** the operator's step-27 rulings of 2026-10-14 (§4.2, the MCP adapter) and the S27-d rulings QS27d-1 and QS27d-2 of 2026-10-15 · **Relates to** `ARC-96` (the adapter is a client; MCP confined to one module), `ARC-97`, `DEP-24`, `DEP-44`, `REUSE_POLICY.md` §§11–12, 15, `CLAUDE.md` §4 rule 16 · **Design** `.structured-coding/plans/mvp0/pr-s27d-mcp-stdio.md` §§3.2, 4.1, 4.2

**Problem.** An MCP-speaking agent must reach a `Player` (`ARC-96`) over stdio. The MCP wire protocol (JSON-RPC, lifecycle, capability negotiation, the 2026-07-28 revision and its predecessors, subscriptions) is a mature protocol with an official SDK; re-implementing it would duplicate that work without a reason. The SDK must not, however, own world state, seats, validation or caps, and its high-level decorator API would publish each tool's input schema nested under its parameter name, which would break the rule that each tool's `inputSchema` and `outputSchema` are the `Player`'s own models.

**Options considered** (both directions).

| Option | Verdict |
| --- | --- |
| (a) **`mcp` (Python, official, MIT), confined to `clients/agent/src/mineworld_agent/mcp_server.py`**, on the low-level `Server` with `Tool(input_schema=Args.model_json_schema(), output_schema=Result.model_json_schema())` | **Chosen.** The SDK owns the transport and JSON-RPC; the schemas are the models' exactly (one source); dispatch validates arguments through the models; the subscription handler is the SDK's `ListenHandler`. Cost: about 40 lines of wiring that the high-level server would have provided |
| (b) `MCPServer` with one decorated function per tool | Declined: the published `inputSchema` is nested under the parameter name (`$defs` reference), so it is not `ObserveArgs`'s schema (F-S27d-1) |
| (c) `MCPServer` with flattened parameters | Declined: thirteen hand-written signatures mirroring the models, a second schema source |
| (d) `rmcp` (Rust) inside the server | Declined: the server would speak a second client vocabulary beside `join`/`submit`/`leave`, which `server/PROTOCOL.md` §2 forbids; MCP types would enter `server/` |
| (e) `rmcp` 3.5.1 in a separate Rust client binary beside the adapter | Declined for now (step-27 §4.2): the resuming client would be written a second time in Rust. Revisit on QS27-8's trigger (installing `uv` as the main obstacle for players of the bundle) |
| (f) The TypeScript SDK 1.32.1 | Declined: the language rule (`CLAUDE.md` §5 and `ENGINEERING_STANDARDS.md` §3) puts client code in Python |
| (g) Our own JSON-RPC and MCP lifecycle | Rejected: a protocol implementation is commodity infrastructure (`REUSE_POLICY.md`), and the revision churn (`X-2`, `X-3`) would be ours to follow |

**Version rule** (step-27 §4.2): the newest release at least 14 days old when the dependency is added. On 2026-10-15 the newest release at least 14 days old is **2.2.0**: `mcp` 2.3.0 was uploaded 2026-10-02, 13 days earlier. The requirement is `mcp>=2.3,<3` from 2026-10-16 on, and `mcp>=2.2,<2.3` before it; C1 records the date, the requirement written and the locked version.

**Sources** (verified 2026-10-15): PyPI JSON for `mcp` (newest 2.3.0, 2026-10-02T22:06:52Z; 2.2.0, 2026-09-07; 2.0.0, 2026-07-28; none yanked; licence MIT; `requires_python >=3.10`); the MCP specification's versioning page (current revision 2026-07-28); the SDK's release notes (`https://github.com/modelcontextprotocol/python-sdk/releases`); the SDK's documentation for tools, structured output, error handling, the low-level server, subscriptions and testing (`https://py.sdk.modelcontextprotocol.io/`).

**Consequences.** `mcp` is a dependency of the `mineworld-agent` member only. The repository-wide scan (`clients/agent/tests/test_mcp_confined.py`) finds `mcp` nowhere else: no other `pyproject.toml` names it and no other Python file imports it. The packages `mcp` pulls in are listed with their licences in the design's ledger (§15); a GPL-family licence is a material stop. Removing `clients/agent` from the workspace removes this dependency with it (AC-d-10).

**Revisit** if the SDK stops accepting an `outputSchema` shape for error results this adapter relies on (`ToolError` as an `isError` result without `structuredContent`, `QS27d-1`); if subscriptions over stdio change; or when S27-e (Streamable HTTP) reuses `build_server` (`R-S27e-d-1`).
---

## ARC-98 — Shared controller memory is a cognition-side home of perceived envelopes, and perception is the only access right

**Date** 2026-10-14 · **Status** accepted (step-28 design frozen 2026-10-14); implemented by S28-a · **Approved by** the operator's step-28 rulings QS28-1 … QS28-10 (2026-10-14) · **Relates to** `ARC-25`, `ARC-27`, `ARC-43`, `ARC-57`, `ARC-58`, `ARC-59`, `ARC-86`, `ARC-87`, `ARC-89`, `ARC-99`, `ARC-100`, `ARC-101`, `DEP-37`, `DEP-58`, `INV-4`, `INV-11`, `INV-13`, [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §5.3, [`ARCHITECTURE.md`](ARCHITECTURE.md) §9.3 · **Design** `.structured-coding/plans/mvp0/step-28-shared-memory.md` §§4, 6; `.structured-coding/plans/mvp0/pr-s28a-memory-home.md`

**Problem.** Today each seat stores its own copy of every fact it perceived. In the 12-person `social-cafe` at seed 7 over 30 days, 86.7 % of the 84 639 distinct facts are perceived by more than one person, and the seats hold 449 726 copies, 5.3 per fact (step-28 §2, F-S28-1 and F-S28-2). Shared events must be stored once, and a person's access to them must be a right, loaded when needed (operator, 2026-10-14). The world log cannot be the shared store: a controller is a client that sees only its perceived stream (INV-6, INV-13), and the log is world truth, not a subjective layer (INV-4).

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) One store per seat, as today | Declined: 5.3 copies per fact; the 20 MiB bound would hold about 45 days of undigested detail |
| (b) Shared records in the kernel or in the world log, read by controllers | Declined: the kernel would learn what memory and a holder are (INV-12, `CLAUDE.md` §4.2), and a controller would read truth it must not read (INV-13) |
| (c) **One cognition-side home per process: fact records keyed by `EventId` and verified by digest, and one access row per person** | **Chosen** |
| (d) Content-addressed records keyed by a hash of the content | Declined as the key: the world already gives each fact a stable, unique id (INV-11), and a hash would add a second identity. The method is adopted (`DEP-58`) |
| (e) Gists deduplicated by content hash | Declined: a gist is the person's own rendering and depends on that person and their name book at ingestion (I-S28-2); the saving is under 4 % of world bytes |
| (f) Per-seat files joined by `ATTACH` in one connection | Declined: atomic commit across attached files holds only in rollback-journal mode, a restriction to defend indefinitely. One file per home gives one transaction |
| (g) A reference count stored on each record | Declined: a stored count can drift from its rows. The anti-join over the access rows is exact by construction |

**Choice.** Shared controller memory is a cognition-side home of perceived envelopes, and perception is the only access right. Fact records are stored once per home, keyed by `EventId` and verified by digest. Access rows are per person. GC is the anti-join in the releasing transaction. There is no kernel, server or protocol change in MVP-0.

The concrete rules that follow, stated in step-28 §§4 and 6: the home is one SQLite database per world instance and cognition process, and it is the only writer of its tables. A fact record exists exactly while at least one access row cites its key (I-S28-1, I-S28-4). Insert-or-verify raises `ConflictingFact` on a digest difference. Each ingest call and each retention pass is one `BEGIN IMMEDIATE` transaction. Forgetting removes an access row or demotes it into a private summary; it never redirects to a peer's copy.

**Why not ourselves, and why not them.** Storage and transactions are commodity, so SQLite is reused (`DEP-37`, `DEP-58`). The access rule, the garbage-collection rule and the determinism contract are MineWorld's own, because no framework's execution model may own them (`REUSE_POLICY.md`).

**Interface that isolates it.** `mineworld_cognition.memory.home` (`MemoryHome`, `SeatView`). The fact and access tables are written only by the home.

**Limitation accepted.** Sharing is per process in MVP-0. Seats in different processes have different homes, so duplication is bounded by the number of processes, not of persons (step-28 §6.6). Cross-process sharing is MVP-1, through the server as the cold tier (QS28-2, path (i)). A cognition-side shared service is declined as the default because it cannot verify perception.

**Revisit** when MVP-1 opens cross-process sharing (S28-h), or when a world needs more than one process per instance.

---

## ARC-99 — A person's size is the seat view, counting shared records in full

**Date** 2026-10-14 · **Status** accepted (step-28 design frozen 2026-10-14); implemented by S28-a and S28-b · **Approved by** the operator's ruling QS28-4 (2026-10-14) and the 20 MiB default (operator, 2026-10-14) · **Relates to** `ARC-59`, `ARC-86`, `ARC-87`, `ARC-98`, `ARC-100`, `INV-13`, `pr-fg-b2-size-trigger.md`, [`ARCHITECTURE.md`](ARCHITECTURE.md) §9.3, [`MODULE_SPEC.md`](MODULE_SPEC.md) §5.2 · **Design** `.structured-coding/plans/mvp0/step-28-shared-memory.md` §§3 (I-S28-2, I-S28-3, I-S28-6), 7

**Problem.** Once a fact record is shared, the bound must say what it measures. A fair share (bytes divided by holders) couples one person's memory to everyone else's perception: Alice's size would fall when Bob perceives a scene and rise when he forgets it, which could trigger Alice's size pass and change what Alice remembers because of Bob. That breaks I-S28-3, makes Alice's memory a function of the town rather than of her perception (`ARC-59`), and makes a shared Alice differ from a standalone Alice (I-S28-2).

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) Fair share of each shared record (bytes ÷ holders) | Declined: the coupling above |
| (b) Count only the private rows | Declined: the records a person holds would be uncharged, and the 20 MiB promise would not bound what the person remembers |
| (c) **The seat view, with every held record counted in full** | **Chosen** |

**Choice.** A person's size is the seat view, counting shared records in full. Sharing is transparent (I-S28-2, I-S28-3) and is a deployment setting, not a world rule. The bound applies to that size. Physical bytes are at most the sum of the seat views, and that sum is at most the number of seats times the bound (I-S28-6). The physical share of a record is reported by `inspect bytes` as a diagnostic and is never charged.

**Consequences.** The 20 MiB promise keeps its literal meaning: each person may remember up to 20 MiB, measured as if alone. At the positional encoding of `ARC-101` that is about 106 days of undigested detail, against about 45 at today's encoding (step-28 §8.3). Sharing still saves the world's disk: measured on social-cafe at seed 7 over 30 days, the world's physical bytes are 59 % of the logical sum at today's encoding and 23 % with `ARC-101` (step-28 §§7.1, 8.2).

**Limitation accepted.** The saving is measured only on 12-person worlds. No World Pack on `main` has more than 12 people (F-S28-11), so the saving at 100 people is unmeasured until the S28-g fixture exists.

**Revisit** when a world with more than 12 people is measured (AS28-17), or when a world-level budget is proposed, which would need its own decision.

---

## ARC-100 — Memory scopes: one typed interface, entry-point registration, host-written tables, `SeatView`-only reads, no access grants

**Date** 2026-10-14 · **Status** accepted (step-28 design frozen 2026-10-14); implemented by S28-c and S28-d · **Approved by** the operator's rulings QS28-3, QS28-6, QS28-7 and QS28-10 (2026-10-14) · **Relates to** `ARC-85`, `ARC-86`, `ARC-98`, `ARC-99`, `FGb-3`, `FGb-4`, P4 invariants I-3 and I-10, `INV-4`, `INV-13`, [`MODULE_SPEC.md`](MODULE_SPEC.md) §5.3 · **Design** `.structured-coding/plans/mvp0/step-28-shared-memory.md` §§5, 12, 15 (AS28-2, AS28-14, AS28-15)

**Problem.** Operators ask for person, town, place and item, and user-defined memory (for example `organization`). Each is a different way of organising what a person perceived, and none is a different source of knowledge. A "town memory" that a person could read without perceiving it would be INV-13's omniscience by another name. A user-defined scope must also be able to extend memory without editing the kernel or cognition internals (R-11).

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) Each scope a bespoke module with its own write path and access rule | Declined: each would be a second access rule |
| (b) A scope may grant access beyond perception, for example an organization's archive readable by all members | Declined (QS28-7): knowledge reaches a member as a world fact that a System decides and perception delivers |
| (c) **A scope is indices, private rows and a recall section over `SeatView`, behind one typed interface and an entry point** | **Chosen** |
| (d) Scopes as application tenancy, with model-decided writes (mem0) | Declined (`DEP-58`) |
| (e) Model-managed paging between memory tiers (Letta) | Declined (`DEP-58`): not a deterministic derivation (`ARC-59`) |

**Choice.** Memory scopes: one typed interface, entry-point registration, host-written tables, `SeatView`-only reads, no access grants. Entity-anchored memory is fed from perception, not the log.

The interface is the `MemoryScope` protocol of `mineworld_cognition.memory.scopes`, with `index` (pure), `recall` (reads only through the seat view) and `account` (the scope's share of the seat view). Registration is through the group `mineworld.memory_scopes`. A scope receives no connection, path, clock or gateway, and only the home writes (AS28-14). The log-derived place history is a test and tool oracle only (QS28-6).

**Limitation accepted.** A scope can organise only what perception delivered. Information that reaches people outside perception does so as a world fact, decided by a System and kept by the log (step-28 §5.1). An anchored scope therefore shows what a person saw happen at a place, not the place's objective history.

**Revisit** when a scope needs a write that the home cannot express, or when a scope package must read something other than a seat view.

---

## ARC-101 — Canonical schema-4 encoding: positional rows, on which the bound and determinism are defined

**Date** 2026-10-14 · **Status** accepted (step-28 design frozen 2026-10-14); implemented by S28-a. The schema number 4 follows the 2026-10-15 dated note of step-28 (F-S28a-1), because FG-b2 shipped schema 3 after the step was written · **Approved by** the operator's ruling QS28-5 (2026-10-14: "换") · **Relates to** `ARC-25`, `ARC-27`, `ARC-86`, `ARC-87`, `ARC-98`, `ARC-99`, `DEP-37`, `DEP-58` · **Design** `.structured-coding/plans/mvp0/step-28-shared-memory.md` §§8.1, 8.2, 10, 15 (AS28-12); the concrete grammar is fixed by `.structured-coding/plans/mvp0/pr-s28a-memory-home.md` §5.4

**Problem.** The v2 canonical form writes sorted-key JSON objects with a table name on every row. About 41 % of each `l0` line is syntax (step-28 F-S28-9), and the memory bound and the determinism contract are both defined on those bytes. At the v2 encoding, 20 MiB holds about 45 days of undigested detail per person.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) Keep sorted-key JSON objects (the v2 form, or a canonical-objects layout) | Declined: about 3.3 MB per world-day against 1.29 MB, and the bound holds about 45 days |
| (b) **Positional rows: `[table, c1, c2, …]` in schema order, ids unpadded, a 128-bit digest in the record** | **Chosen** (QS28-5) |
| (c) A binary serialization library's format | Declined: the library would own the determinism contract (`REUSE_POLICY.md`), and the canonical form is ours to define |
| (d) The full 256-bit digest in each record | Declined for the record: the 128-bit digest is kept, and the full digest remains the re-delivery check inside the transaction, where it is not dumped (≈ 90 KB per world-day, +7 %, if dumped) |

**Choice.** Canonical schema-4 encoding (QS28-5, ruled 2026-10-14): positional rows, which the bound and determinism are defined on. Each row is `[table, c1, c2, …]` in schema order. The encoding is defined once in `memory.home`, and it is used for the bound and for determinism.

**Consequences.** On social-cafe at seed 7 over 30 days, the world-day total falls to 1 291 514 B against 5 588 338 B at v2 (−77 %), and 20 MiB holds about 106 days of full detail instead of about 45 (step-28 §§8.2, 8.3). The dump is a determinism artefact and is harder to read by hand; `mineworld-memory inspect` is the readable surface.

**Limitation accepted.** The record keeps a 128-bit digest, not the full 256-bit digest. The full digest is checked in every transaction, so a difference is still caught when a seat is delivered, but it is not in the dump.

**Revisit** when a schema change is needed, or when a tool outside MineWorld must read the dump.

**Note, 2026-10-15 (PR S28-a; `pr-s28a-memory-home.md` §5.4 and ledger D-a-5).** The concrete grammar, as implemented in `mineworld_cognition.memory.home` (`encoding.py`, `schema.py`):

- A line is the compact JSON array `[table, c1, …, cn]` (separators `,` and `:`, no spaces, UTF-8 not escaped), then `\n`.
- Cells go through their column's codec. The `EventKey` columns (`event_key`, `first_key`, `last_key`, `level_key`, `source_key`, `trigger`) and `event_id` are written as the unpadded decimal number. A digest is written as its first 32 hex characters. `prose_key` is a cassette key and stays text. Every other value is written as stored (`INTEGER` → number, `TEXT` → string, `NULL` → `null`); no table holds a float.
- `meta` lines are `["meta", key, value]`. The values of `cursor`, `newest` and `forgotten_below` are numbers, `null` while unset; every other value is a string (`schema` is `"4"`).
- **Seat view** (`SeatView.dump()`): `meta` (`world_instance`, `schema`, `seat`, `self_entity`, and the seat's own rows, in key order), `names`, `l0` (`[l0, event_id, at, kind, place, salience, mine, gist, digest, importance]` in key order), `l0_who` (`[l0_who, event_id, entity]` for the held facts' participants other than the seat, in envelope order), then `episodes`, `episode_who`, `chapters`, `chapter_ranges`, `stable`, `forgotten`, `retention_passes`, each without its `seat` column, rows in key order.
- **Home** (`MemoryHome.dump()`): the `home_meta` lines; one `[fact, event_id, at, kind, place, digest]` line per record in key order (the id is written once, because the key's number is the id); one `[fact_who, event_id, role₀, entity₀, role₁, entity₁, …]` line per record that has `fact_who` rows, in ordinal order; then, for each seat in key order, `[seat, key, self_entity]`, followed by that seat's rows in the seat view's table order, with `[access, event_id, gist, salience, mine, importance]` lines where the view has `l0` and no identity rows. The internal seat number never appears.
---

## DEP-58 — Reuse verdicts for shared memory: build on SQLite and P4/FG-b, adopt the methods of content addressing and tiered memory, decline Letta, Zep, Graphiti and mem0 as runtime dependencies

**Date** 2026-10-14 · **Status** accepted (considered 2026-10-14, step-28 §16); no new dependency is added by this record. Each licence marked **verify** in step-28 §16 is confirmed by the PR that first relies on it · **Approved by** the step-28 frozen design · **Relates to** `DEP-37`, `ARC-98`, `ARC-100`, `ARC-101`, [`REUSE_POLICY.md`](REUSE_POLICY.md) §§11–12, 15 · **Design** `.structured-coding/plans/mvp0/step-28-shared-memory.md` §16

**Problem.** Shared memory needs storage, content addressing, tiered loading and pluggable scopes. `CLAUDE.md` §4 (rule 16) requires each candidate to be checked in both directions: whether a mature wheel is being reinvented, and whether an existing wheel is being forced where it does not fit.

**Options considered** (verdicts from step-28 §16).

| Candidate | Verdict |
| --- | --- |
| SQLite (`sqlite3`, `DEP-37`) | **Keep.** One file per home, FTS5, incremental vacuum |
| Content-addressed stores (Git objects, IPFS / `multiformats`, Perkeep, restic, borg) | **Adopt the method, decline the code.** Objects already have a stable world id (`EventId`); a hash key would add a second identity |
| Ref-counted blob stores (`diskcache`, LMDB-backed stores) | **Decline.** A stored refcount can drift from its references; the anti-join over access rows is exact |
| Generative Agents memory stream (Park et al., 2023) | **Reference only**; each agent's stream is its own |
| MemGPT / Letta tiered memory | **Adopt the method, decline the dependency.** Hot and cold tiers and load-when-needed are the method. Shared blocks are agent-writable text, a knowledge channel outside the world, and model-managed paging is not deterministic |
| Zep / Graphiti | **Decline.** Model-extracted triples are not a deterministic derivation, it needs a graph server, and its sharing is by tenant, not by perception |
| mem0 | **Decline.** Its scopes are application tenancy and its writes are model-decided. Scopes as first-class configuration are adopted (`ARC-100`) |
| Forcing an existing wheel | Not done. No framework's execution model owns our transactions, access rule or determinism |

**Choice.** Build on SQLite and P4/FG-b; adopt the methods of content addressing and tiered memory; decline Letta, Zep, Graphiti and mem0 as runtime dependencies.

**Why not them.** Each declined candidate either makes model output the source of stored knowledge, or gives access by tenant rather than by perception. Both conflict with INV-13 and `ARC-59`.

**Revisit** when the MVP-1 cold tier (S28-h) or an embedding-based scope is proposed (step-28 §18). Either needs a `DEP` record of its own.

---

## ARC-120 — Person attributes are declared data with one owner each; `attributes` owns plain traits; an owner's attribute is a view, never a copy

**Date** 2026-10-15 · **Status** accepted (step plan approved 2026-10-15); specified by S30-a, implemented by S30-c (the `attributes` pack and the model crate) and S30-f (`employment.occupation`) · **Approved by** the operator, directly in the primary session, 2026-10-15 (step-30 approval; Q-S30-4, Q-S30-6, Q-S30-8; Q-S30a-2) · **Relates to** `ARC-26`, `ARC-31`, `ARC-61`, `ARC-74`, `ARC-114`, `ARC-115`, `ARC-121` … `ARC-126`, [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §4.5, [`MODULE_SPEC.md`](MODULE_SPEC.md) §§3.1, 4.1 · **Design** `.structured-coding/plans/mvp0/step-30-person-attributes.md` §4

**Problem.** The operator requires that a Person can carry very many attributes — age, build and ancestry, and many more — that a world chooses by data, with most of them available out of the box and custom ones added freely (2026-10-15). Nothing in the world holds a person's attributes today: casting keeps appearance in Presentation Pack briefs, `persona` keeps prose, and cognition has nothing typed to read. Some of the attributes a reader needs are already modelled by a System: wealth by `economy`, occupation by `employment`, notice subscriptions by S31's `notices`.

**Options considered** (step-30 §4.1).

| Option | Verdict |
| --- | --- |
| (a) One `attributes` pack owns every attribute | Declined: it would copy wealth from `economy` and occupation from `employment`, breaking single ownership (`CLAUDE.md` §4 rule 1), and S31's System would have to ask another pack to change its own routing table |
| (b) Each System Pack declares what it needs, with no shared schema | Declined: every reader (cognition, MCP, both clients, casting, the sampler) would need a per-pack adapter, and hundreds of plain traits would have no owner |
| (c) **One declared schema (the catalogue) and one owner per attribute: a default `attributes` pack owns plain traits; specialised packs own theirs** | **Chosen** |

**Choice.**

1. **An attribute** is a declared, typed property of a Person, in one of three classes of state: identity, trait, or a view of mutable state another System owns ([`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §4.5). It is declared as data and named `<namespace>.<slug>`, the namespace being the declaring pack's id.
2. **Every attribute has exactly one owner**, named in its declaration (`owner:`, default `attributes`). Only the owner writes it (INV-7).
3. **`attributes` owns plain traits** — attributes no installed System models — and stores a Person's values in one component, `traits` (schema 1): a sorted list of `(qualified id, value)` pairs, each checked against the catalogue. It provides no action; it reacts to `attribute-change-proposed` from a pack that declared the attribute in `proposes:` and that depends on it (`ARC-26`), and states `trait-changed { person, attribute, from, to, because }` or refuses by name.
4. **A specialised owner keeps its own state.** Its attribute is a projection of its own component, computed when read or disclosed, and nobody else stores it: a view, never a copy. The owner implements `AttributeOwner` (`ARC-124`); because the projection is code, a view over another pack's state cannot be made with data alone.
5. **An owner seeds its own attributes** from the values the resolution hands it (`ARC-123`), in its own vocabulary.
6. **No owner, no attribute.** An attribute whose owner the world does not enable is refused at load (`OwnerNotInstalled`).
7. **The kernel learns nothing** (INV-12): the catalogue, values, handles and sampler live in `mineworld-attributes-model` (above `mineworld-authoring`, below the packs); no type, word or id of it enters `kernel/` or `contracts/`.
8. **Not attributes:** a display name (`naming`'s identity, never sampler input); what someone said about themselves (memory, INV-4); the look key (`appearance`'s, `ARC-125`).

**Consequences.**

- The owners named so far: `economy` (`economy.wealth-band`, S31-f, `ARC-114`); `employment` (`employment.occupation`, a projection of `Employment { job }` to the job's label, added by **S30-f as the one named exception** to step-30 §9.4's "no existing System is changed by S30", Q-S30a-2, ruled 2026-10-15; nothing else in `employment` changes); `notices` (`notices.subscriptions`, S31, `ARC-115`); `persona` (`persona.summary`, `persona.traits`, `persona.speech`, `text`, `private`; P6a merges as frozen and the overlap with the typed psychology group is reviewed after S30-f, Q-S30-4).
- Because rows 32 and 37 of step-30 §6.1 are owner views, the built-in catalogue declares 93 `core` attributes, not 94 (`ARC-122`).
- Sensitive attributes (ancestry, religion, income and any a world adds, such as sexual orientation or disability) are `private` by default, never in another observer's view or the agent description; a world may narrow them, never widen them (Q-S30-8).
- Owner packs depend on the model crate, not on the `attributes` System Pack (step-30 §4.3). The data directories that hold declarations are `attributes`' (`ARC-121`), so **how a world that enables an owner but not `attributes` reads the owner's shipped declarations is S30-c's decision**, recorded in its PR design and as a dated note here (S30-a F-S30a-10).
- Change amplification holds: a new plain trait is a data file; a new view over another pack's state is that pack's projection; nothing else changes.
- *Ids cited before their records exist.* When `ARC-120` … `DEP-68` were written (2026-10-15, S30-a), `ARC-74` (P6a), `ARC-91` and `ARC-92` (S26-a, #206), `ARC-95` (S26), and `ARC-114` and `ARC-115` (S31) were assigned but not yet on `main`; each is written by its own PR. Whichever of that PR and S30-a merges second checks the citations here against the merged text, and records a dated note where they differ.

**Revisit** when a second plain-trait store is proposed, or when an owner needs a view that data cannot declare and its owner cannot project.

---

## ARC-121 — The authoring seam: data directories, decoding in the owner's context, owner resolution before seeding, and shipped data

**Date** 2026-10-15 · **Status** accepted (step plan approved 2026-10-15; S30-b design frozen 2026-10-15); specified by S30-a, implemented by S30-b · **Approved by** the operator, directly in the primary session, 2026-10-15 (step-30 approval; S30-b freeze "同意冻结并实施 S30-a/b"; Q-S30b-1 for item 4) · **Relates to** `ARC-31`, `ARC-36`, `ARC-61`, `ARC-71`, `DEP-10`, `ARC-120`, [`MODULE_SPEC.md`](MODULE_SPEC.md) §§2, 3.1, 4.1 · **Design** `.structured-coding/plans/mvp0/step-30-person-attributes.md` §4.3; `.structured-coding/plans/mvp0/pr-s30b-authoring-seam.md` §5

**Problem.** `ARC-31`'s sections decode statically: a section's type cannot see world-level data, so a schema declared in data cannot be checked at the author's line, and no section sees the whole set of subjects at once, which stratified sampling needs. The loader reads data only from World Packs and Entity Packs, while some System Packs must contribute declarations of their own (step-30 §9.6 N-2, step-31 §7.4).

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) A world-level section in `world.yaml` | Declined: `world.yaml` carries no sections, and its `population:` key already means the people files (step-30 F-S30-8); world settings are a pack's `configure/<id>.yaml` (`ARC-61`) |
| (b) An optional associated `type Context` on `AuthoredSection` | Declined in form, kept in meaning: on the pinned Rust an associated-type default is unstable, and a required one breaks every existing section, including the external `acme-fishing` (S30-b F-S30b-1) |
| (c) **Four additive capabilities beside `ARC-31`'s: owned data directories; a sibling trait decoding a section in its owner's context; a resolution across subjects before seeding; shipped data** | **Chosen** |
| (item 4 alternative) a companion Entity Pack per System Pack | Declined (Q-S30b-1): it splits one capability across two packages, and a world that enables the System but forgets the companion fails far from the cause |

**Choice.**

1. **A pack may own data directories** in World Packs and Entity Packs. The loader collects their `.yaml` files in one deterministic order — shipped files in `systems:` order, then required Entity Packs in id order, then the world; within each, directories in the owner's order and files by name — and the owner decodes each with line and column and builds one context from all of them and its world configuration. An unknown directory is refused, listing those the build knows; an installed owner's directory in a world that does not enable it is refused naming the owner; dot directories and the `decisions:` file's directory are not content.
2. **A section may be decoded in its owner's context**, so a value is checked against a schema declared in data at its own line and column. Its Rust form is a sibling trait (`SectionInContext`); a pack owns one section either way.
3. **An owner may resolve across subjects once, before any seeding**: it receives each subject's key, kind, tags and decoded section — never a name, note or location — and returns at most one seed per subject, seeded where its section's facts would have been; seeding order is unchanged (`ARC-31` item 4), and a seeded fact outside the owner's vocabulary is refused.
4. **A System Pack may ship data files for an owner's directory**, compiled in (`SystemPack::SHIPS`, `ships_data!`), collected first whenever the shipping pack is enabled, each refusal naming the pack and the crate-relative path (Q-S30b-1, ruled 2026-10-15).
5. **Entity Packs** may carry data directories instead of, or beside, `items/`; the "item kinds only" sentence of [`MODULE_SPEC.md`](MODULE_SPEC.md) §2 is amended (2026-10-15).

**Consequences.** The seam names no domain; its first user is `attributes` (S30-c), and an Entity Pack's taxonomies are a plausible second, which is why it belongs in `mineworld-authoring`. Every reference world reads, loads and runs byte-identically (I-S30-10). The seam's contexts are not facts, so S30-c adds a resume-time check that a world's data has not drifted from its save's (S30-b R-S30c-b-2). Root files of a data pack (a backend's `backend.yaml`) and tooling directories inside one are not covered by item 1; S30-e and S30-h fix how they are carried, as dated notes here (S30-a F-S30a-9). A divergence S30-b finds between this text and the code is corrected in one of them, never left (S30-b R-S30a-b-1).

*Note 2026-10-16 (S30-b, implemented; R-S30a-b-1).* Item 4 is refined as built: a shipped file is collected only when the world enables its directory's owner as well as the shipping pack; one shipped into a directory whose installed owner is not enabled is neither collected nor refused, and one shipped into a directory no installed pack owns is refused (`UnknownDirectory`). The `SystemPack` item behind item 3 is `resolve_subjects`, not `resolve`, which every `System` already has ([`MODULE_SPEC.md`](MODULE_SPEC.md) §3.1, corrected 2026-10-16). Design: `pr-s30b-authoring-seam.md` F-b-1, D-b-6.

**Revisit** when a second owner of data directories appears, or when Tier-1 packs (`ARC-8`) need data contributions without a rebuild.

---

## ARC-122 — The attribute model: kinds, typed values and handles, visibility, mutability, rendering, the built-in catalogue

**Date** 2026-10-15 · **Status** accepted (step plan approved 2026-10-15); specified by S30-a, implemented by S30-c and S30-e (the catalogue), S30-k (localisation) · **Approved by** the operator, directly in the primary session, 2026-10-15 (step-30 approval; RR-S30-3; Q-S30a-1; Q-S30-8) · **Relates to** `ARC-120`, `ARC-121`, `ARC-123`, `ARC-125`, `ARC-126`, `DEP-10`, [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §4.5, [`MODULE_SPEC.md`](MODULE_SPEC.md) §4.1 "Attributes" · **Design** `.structured-coding/plans/mvp0/step-30-person-attributes.md` §§5, 6.1–6.3, 7

**Problem.** Hundreds of attributes, many added by worlds, must reach Systems, cognition, agent players and both clients through one typed path — not `map<string, any>` (`CLAUDE.md` §4 rule 7) — with each value validated where it is written and with visibility that keeps omniscience impossible (INV-13).

**Options considered.**

| Option | Verdict |
| --- | --- |
| Free-form key/value metadata on a Person | Declined: untyped at the boundary; nothing checks a value; every reader parses strings |
| One Rust type per attribute | Declined: a custom attribute would need code, against the operator's "no hard-coding" |
| **A frozen, data-declared catalogue; a closed value enum checked against its declaration; typed handles obtained at install** | **Chosen** |

**Choice.**

1. **Declarations** (YAML, `format: 1`): id (`^[a-z][a-z0-9_.-]{0,63}$`), `group`, `facets`, `kind`, `visibility`, `mutability`, `owner`, `label`, `phrase`, `words`, `perceived_as`, `default`, `since`, `prompt_rank`. The catalogue is decoded once at load into a frozen `Catalogue`.
2. **Kinds:** `category`, `level` (ordered), `quantity` (an integer at a declared `scale` in a unit from a closed list: `m`, `kg`, `year`, `day`, `hour`, `count`, `ratio`; no currency, no float), `flag`, `set` (`of:` a list or `vocabulary:`; `max`), `text` (≤ 280 bytes, never readable by a System), `instant`. `category` also accepts `vocabulary:` (S30-a F-S30a-7, for `economy.wealth-band`). A value is the closed enum `AttributeValue`; saves hold slugs, never indices. `unknown` is not a value: absent means not stated.
3. **Typed reads.** A System declares what it reads (`reads_attributes`) and obtains a `Handle<K>` once at install, refused by name (`Missing`, `WrongKind { declared, requested }`, `MissingUnderBackend`); no `Handle<Text>` exists; a structural test refuses a catalogue lookup outside install code.
4. **Visibility:** `perceivable` (another observer is told `perceived_as`, else the value), `learnable` (only conversation reveals it, into the learner's memory), `private` (the holder only), `hidden` (nobody; Systems only). Disclosure is by perception only; a world may narrow a built-in's visibility, never widen it.
5. **Mutability:** `fixed`, `evolving` (through its owner, citing a cause, INV-15), `derived` (`derive:` from `age`, `band`, `projection`; never stored or stated). Age advances with no event, from `birth` and the observation's instant. No drift rule engine ships.
6. **Rendering:** `label`, a `phrase` with the one placeholder `{value}`, `words`; deterministic, bounded, English by default; locale label packs (`labels/<locale>.yaml`, zh-Hans first) in S30-k.
7. **Namespaces, short slugs and overrides:** `core` is the built-in catalogue's; a world's and an owner's declarations use their own id; a foreign namespace is refused; a unique short slug is accepted; an ambiguous one is refused with the candidates; a world changes another's declaration only through `overrides:` (presentation, added facets, `default`, narrower visibility), never its kind, values, unit, scale, owner or mutability.
8. **The built-in catalogue** `entities/person-attributes` (CC0-1.0) is **curated to at most 100 attributes, each read by something the player notices** (RR-S30-3), in six groups — appearance, background, psychology, capability, behaviour, lifestyle. It declares **93 `core` attributes**: step-30 §6.1's 94 rows, plus `core.natal-sex` (`category` female, male; `private`, `fixed`; read by casting as the profile parent of stature, build and facial hair, Q-S30a-1, ruled 2026-10-15), minus `employment.occupation` and `economy.wealth-band`, which their owners declare (`ARC-120`). A 101st built-in, or one without a reader line, fails the catalogue check. Versioning is semantic; an id is never reused.

**Consequences.** A custom attribute (`freckles`) is one YAML file and reaches every surface with no code (step-30 §7.3, AC-S30-3). What still needs code: a new view over another pack's state, a new unit, a System that acts on an attribute. Cognition renders "who I am" and "who I see" from the same phrases under byte budgets; MCP gains `PersonLine.description` and `me.attributes`; both clients show one inspect panel (S30-f; step-30 §§9.2–9.3). An attribute learned in conversation is a memory, never world state (S30-i).

**Revisit** when a world needs a unit or a kind the closed lists lack, or when the catalogue reaches its cap.

---

## ARC-123 — Population profiles and the deterministic stratified sampler; the population seed is not the run seed; the lock file

**Date** 2026-10-15 · **Status** accepted (step plan approved 2026-10-15); specified by S30-a, implemented by S30-d (sampler, lock), S30-e (cited profiles), S30-g (reference worlds, the S26 switch), S30-j (runtime sampling) · **Approved by** the operator, directly in the primary session, 2026-10-15 (step-30 approval; Q-S30-2, Q-S30-5) · **Relates to** `ARC-61`, `ARC-95`, `ARC-120`, `ARC-121`, `ARC-122`, `ARC-126`, `DEP-67`, `DEP-68`, [`MODULE_SPEC.md`](MODULE_SPEC.md) §4.1 "Population profiles", "The lock file" · **Design** `.structured-coding/plans/mvp0/step-30-person-attributes.md` §§8, 10, 11

**Problem.** Worlds must choose their own distributions by data — sampling that respects dependencies (stature given sex and age) and impossibilities — while the biography always wins, nothing is inferred from names, and the result is reproducible on three platforms. The interim casting sampler is Python with floats and is not shared with the world's genesis, so casting and the world could drift.

**Options considered.** Profile in the 3D Presentation Pack (today) — declined, Q-S30-2: a population is world content that cognition, agents and casting read. Independent per-attribute draws — declined: no conditionals, poor small-population fidelity. **A per-world profile over a strict DAG, sampled by one integer, stratified engine with two entry points** — chosen. (The sampler's implementation choice is `DEP-67`.)

**Choice.**

1. **A profile** is a `populations/` file of a data pack: `covers`, cited `sources` (status `read`, `verify` or `authored`; `source: authored` is reserved for conventions), `distributions` (`prior`, `given`, `adjust` likelihood ratios, `mask` hard compatibility, `normal`/`uniform`/`table` numerics with `clip_sd`, `stratify`), and `roles` (defaults by tag or by resolved attribute). Every rule is refused by name at load (`DependencyCycle`, `MissingCondition`, `BadDistribution`, `BadAdjustment`, `BadMask`, `EmptySupport`, `UncitedDistribution`, `MissingSupport`, `NotCovered`, `UnknownAttribute`).
2. **Semantics:** `weight(v) = prior(v | pa) × Π adjust × mask`, normalised to integer parts per million by largest remainder; only a mask zeroes a value; numerics through a committed standard-normal quantile table with integer interpolation — no float, no `libm`.
3. **Algorithm:** stated and locked values fixed first; per attribute in topological order, groups by resolved parents, largest-remainder allocation with seeded tie-breaks, assignment by a counter-based mix of (seed, attribute, person key), a bounded matching for `allowed:`/`between:` restrictions (`Unsatisfiable` when none exists, never a silent re-draw); roles after sampling. The input holds the person's key, tags, authored values and lock entries — never a display name, note or persona.
4. **The population seed is world content**, in `configure/attributes.yaml`, never the run's `--seed`; `salt` exists for S26's distinctness re-draw and changes sampled values only.
5. **The lock file** pins values with origins; **precedence** is the person's file, the lock, roles, the profile's sample, the declaration's default. A casting ruling writes the lock, so a ruled brief and the world's attributes cannot diverge.
6. **Origins** (`authored`, `ruled`, `locked`, `role`, `sampled`, `default`) are tool output, never saved. Genesis gains one private `traits-stated` fact per person with a plain trait; the sampler runs only at genesis.
7. **One implementation, two entry points:** genesis (the seam's resolution, `ARC-121` item 3) and `mineworld attributes resolve --json`, which is the resolved cast S26 consumes (its C-1 … C-7 contract, step-26 §7.4b.1; step-30 §10).

**Consequences.** For 1 000 covered people a synthetic resolution matches every marginal within 0.5 pp and every conditional within 2 pp for groups of at least 50 (AC-S30-5). Profiles move to `entities/populations-us` (S30-e). A release refuses a reference world whose profile marks a **data** figure `verify`, but not an `authored` convention (Q-S30-5). The interim Python sampler is removed when S30-g switches casting to `resolve --json`, with no adapter kept (`CLAUDE.md` §4 rule 12). Adopting attributes in a reference world moves its digests once, under one recorded re-baseline (S30-g); a world without `attributes` is byte-identical.

**Revisit** when MVP-1's region worlds need joint fitting to control totals (PopulationSim, `DEP-67`), or when runtime arrivals (S30-j) show a drift the tolerance does not cover.

---

## ARC-124 — The attribute owner interface, with S31's subscriptions and wealth band as its first users

**Date** 2026-10-15 · **Status** accepted (step plan approved 2026-10-15); specified by S30-a, implemented by S30-c (`AttributeOwner`), used by S31 (`notices`, `economy`) and S30-f (`employment`) · **Approved by** the operator, directly in the primary session, 2026-10-15 (step-30 approval; the coordinator input of 2026-10-15, step-30 §1.2 item 3; Q-S30-6; Q-S30a-2) · **Relates to** `ARC-26`, `ARC-114`, `ARC-115`, `ARC-120`, `ARC-121`, `ARC-122`, [`MODULE_SPEC.md`](MODULE_SPEC.md) §3.1 "Owns" · **Design** `.structured-coding/plans/mvp0/step-30-person-attributes.md` §§4.2, 9.6; `.structured-coding/plans/mvp0/step-31-wealth-prices.md` §§7.4, 8.5

**Problem.** Each NPC needs an attribute saying which public-information topics it subscribes to, mutable and private, read by a System rather than by cognition (operator, 2026-10-15); initial wealth is to be sampled through the same path (Q-S30-6). Both are state another System owns, so `attributes` cannot hold them.

**Options considered.** Copy the value into `traits` and synchronise — declined: two writers. Let `attributes` call into the owner — declined: a dependency from the general pack to every owner. **An interface in the model crate that the owner implements** — chosen.

**Choice.** An owner implements `AttributeOwner` from `mineworld-attributes-model` and depends on that crate, not on the `attributes` System Pack. It:

1. declares its attributes in its own namespace with `owner:` its id, shipped with the pack (`ships_data!`, `ARC-121` item 4), and any vocabulary they draw from, as a `vocabularies/` file or from its world configuration;
2. projects each owned attribute from its own component when read or disclosed (a view, never a copy);
3. receives at genesis each person's resolved value, by the precedence of `ARC-123` (file, lock, roles, profile, default), as a typed value, and states its own facts;
4. changes the attribute only through its own actions and facts — `attributes` is not involved;
5. discloses through the model crate's filter and renderer.

**Consequences.** The interface S31 consumes is step-30 §9.6 N-1 … N-6: `notices.subscriptions` (`set` over `notices.topics`, ≤ 32, `private`, `evolving`), its role default (`{ when: { tags: [cook] }, add: { notices.subscriptions: [prices.ingredients] } }`), and `subscribe`/`unsubscribe` as `notices`' own facts. `economy.wealth-band` is a `derived`, `private` `category` over `economy.wealth-bands` (the band ids of `configure/economy.yaml`), projected from the person's wealth (step-31 §7.4); an authored wallet wins, else `economy` funds the resolved band's opening amount. A role default keyed on occupation reads `employment.occupation` once S30-f has made `employment` its owner. If S31 freezes before S30-c merges, it codes against this text and adopts the crate at S30-c's merge; a conflicting change is a material stop for both.

**Revisit** when an owner needs to react to a change of another owner's attribute, or when a projection needs state its owner does not hold.

---

## ARC-125 — Apparent descriptors are perceivable attributes; `look` stays opaque (amends `ARC-91` item 3)

**Date** 2026-10-15 · **Status** accepted (Q-S30-1); specified by S30-a, implemented by S30-c and S30-e (the appearance group), S30-f (consumers) · **Approved by** the operator, directly in the primary session, 2026-10-15 (step-30 Q-S30-1) · **Relates to** `ARC-91` (S26-a, PR #206: **not on `main` when this record was written**), `ARC-92`, `ARC-120`, `ARC-122`, `ARC-126`, [`MVP.md`](MVP.md) AC-16 · **Design** `.structured-coding/plans/mvp0/step-30-person-attributes.md` §4.4

**Problem.** `ARC-91` item 3 (S26-a) holds that apparent age, stature, build, skin and hair are not world facts — "no descriptors" — because no System consumes them, with a revisit clause "when a System needs an appearance descriptor". AC-16 is a stronger consumer: an agent's observation must be at least as information-rich as what a human sees on screen, within the same perception limits, and today an agent learns only a look key where a human sees a face.

**Options considered.** Keep item 3 and let agents infer from the look key — declined: a look key is opaque by design. Encode descriptors in the look key — declined: `ARC-91`'s opacity. **Declare descriptors as `perceivable` attributes, keep `look` the only key a renderer binds** — chosen.

**Choice.** `ARC-91` item 3 is amended to read: *apparent descriptors (stature, build, apparent age band, skin tone, hair, facial features, glasses, visible marks, clothing style, posture, voice) are `perceivable` attributes of the appearance group, owned by `attributes`, disclosed by perception only; the look key stays opaque and is the only thing a renderer binds; a descriptor never selects art, and the look key never encodes a descriptor.* `ARC-91`'s principles otherwise stand: a look is opaque, appearance never depends on what the observer knows, and disclosure is gated by perception only. The appearance group and quantity attributes are always MineWorld's, under every backend (`ARC-126`).

**Consequences.** S26 casting reads the appearance subset by facet (`appearance`) from `resolve --json`, with no hard-coded attribute list (step-26 §7.4b.1 C-1 … C-7). A human and an agent are told the same thing: a human sees a face and reads "tall, grey hair"; an agent reads "tall, grey hair". **On `ARC-91`'s merge** (S26-a, #206), whichever of S26-a and S30-a merges second adds under `ARC-91` item 3 the dated note "amended by `ARC-125` (2026-10-15)" (S30-a R-S26a-a-1).

**Revisit** when a renderer or a System needs a descriptor to select art, which this record forbids and which would need a new decision.

---

## ARC-126 — Person-generation backends and masks are data; the MineWorld backend is the default; a backend runs no code

**Date** 2026-10-15 · **Status** accepted (Re-ruled 2026-10-15, operator, final: RR-S30-5); specified by S30-a, implemented by S30-e (the backend and mask loader, the default backend) and S30-h (the optional backend) · **Approved by** the operator, directly in the primary session, 2026-10-15 (RR-S30-5, final; RR-S30-3 for the bound of 100; Q-S30-9 for `log-linear`) · **Relates to** `ARC-120`, `ARC-122`, `ARC-123`, `ARC-125`, `DEP-66`, `DEP-67`, [`MODULE_SPEC.md`](MODULE_SPEC.md) §§2, 4.1 "Backends and masks" · **Design** `.structured-coding/plans/mvp0/step-30-person-attributes.md` §§6.4–6.5, 16.2

**Problem.** The operator, final (2026-10-15): "我觉得我们应该是支持MatrAIx为后端生成人物，并给出demo，但是不要把它们作为强依赖，而是一个可选项" — support an external persona schema as a backend for generating people, with a demo, never as a hard dependency. A world must be able to choose its catalogue and its way of generating people, and narrow a large catalogue to what its play uses.

**Options considered** (step-30 §§6.4, 16.2).

| Option | Verdict |
| --- | --- |
| No external schema; learn from its approach only (RR-S30-1) | Superseded by RR-S30-4 and RR-S30-5 |
| Import it as the built-in catalogue, masked to 100 (RR-S30-4) | Refined by RR-S30-5: kept, but optional, not the default |
| A backend that runs its own code (a plug-in sampler) | Declined: determinism on three platforms, kernel ignorance and review all depend on one engine |
| **Pluggable backends and masks as data, executed by MineWorld's engine; MineWorld's curated backend is the default** | **Chosen** (RR-S30-5) |

**Choice.**

1. **A backend** is a data pack supplying a catalogue and the means to generate people over it, chosen in `configure/attributes.yaml` (`backend:`, `mask:`). Its contract, `backend.yaml`, states `id`, `version`, `license`, `attribution`, `catalogues`, `masks`, `sampling`, `provenance` and `mapping`, each validated (`BackendMalformed`, `UnknownMask`, `MaskNamesUnknownAttribute`, `UnsupportedSampling`, `ProvenanceMismatch`, `MissingUnderBackend`).
2. **No backend runs code.** Sampling modes are a closed list in the engine: `profile` (`ARC-123`) and an optional `log-linear` factor model executed in fixed point (Q-S30-9: offered by the optional backend only, used by its demo, never by a reference world, prototyped first and dropped with a report if determinism, load time or a ≤ 2 pp fidelity check fails).
3. **The default backend is `mineworld`** (`entities/person-attributes`, `ARC-122`) with the `profile` mode and masks `core` and `appearance`; every reference world and test uses it.
4. **Appearance and quantities are always MineWorld's:** every backend's catalogue is composed with the `core` appearance group and `core` quantities, which a backend may not redeclare (`ARC-125`).
5. **A mask** is a named, versioned selection (`include`/`exclude` by id, group or facet, each with a `why` naming its reader; `MaskEntryUnjustified`); `all` includes everything. What a mask does not enable does not exist in the world (`MaskedOut` for a file stating it; a handle fails at install). Owner packs' attributes are never masked. A world writes its own mask with no code.
6. **A mapping** gives, per built-in a reader declares, a `derived` view from the backend's values, checked for coverage; an unmapped built-in outside the appearance group is absent under that backend.

**Consequences.** The only optional backend specified is recorded in `DEP-66`. Switching `backend:` and `mask:` changes the world's catalogue and nothing else (AC-S30-15). How the seam carries `backend.yaml`, a root file, is S30-e's (S30-a F-S30a-9).

**Revisit** when a backend needs a sampling mode the engine lacks, which is a model-crate change reviewed as such.

---

## DEP-66 — The MatrAIx persona schema: adopted as an optional backend pack (licence check, pinned commit and digest, update procedure, attribution, isolation); earlier texts kept

**Date** 2026-10-15 · **Status** accepted (Re-ruled 2026-10-15, operator, final: RR-S30-5); implemented by S30-h; no dependency is added before S30-h, and none outside its pack ever · **Approved by** the operator, directly in the primary session, 2026-10-15 (RR-S30-1 … RR-S30-5, the last final; Q-S30-8 for the overlay's privacy; Q-S30-9 for `log-linear`) · **Relates to** `ARC-122`, `ARC-123`, `ARC-126`, `DEP-67`, [`REUSE_POLICY.md`](REUSE_POLICY.md) §§11–12, [`MODULE_SPEC.md`](MODULE_SPEC.md) §2 · **Design** `.structured-coding/plans/mvp0/step-30-person-attributes.md` §§3.2, 6.6–6.7, 16.2

**Problem.** The operator pointed to MatrAIx (Li, Hao et al., *MatrAIx: Simulating the World with 8.3 Billion Persona Agents*, arXiv 2608.04205v1, 2026) as the model for very many person attributes. Its schema has 1,290 categorical dimensions in five groups, natural-language phrases and six locale label packs, but no body or appearance dimension and no numeric kind, and some dimensions and its coreset carry licences narrower than the repository's.

**Options considered — the rulings in order, every earlier text kept** (step-30 §16.2).

| Ruling | Text | Status |
| --- | --- | --- |
| Step approval (Q-S30-7) | "taxonomy adapted, optional import in MVP-1"; about 200 built-ins | superseded by RR-S30-3 (≤ 100) and RR-S30-2 (everything before the release) |
| RR-S30-1 | "我们不要对matrixl有依赖，我只是让你学习它的方式" — inspiration only, no dependency | superseded by RR-S30-4 and RR-S30-5; its "no hard dependency" survives in RR-S30-5 |
| RR-S30-3 | ≤ 100 attributes, only what affects play | in force for the built-in catalogue and as the default mask's bound |
| RR-S30-4 | import as-is, behind a modular mask of 100, `all` available | in force as refined by RR-S30-5 |
| **RR-S30-5 (final)** | an optional backend with a demo, never a hard dependency | **in force** |

**Choice.** An optional Entity Pack, `entities/attributes-matraix`, not installed by default.

1. **Licence check.** The repository `MatrAIx-ai/MatrAIx-Persona-8B` is MIT. Excluded **on licence grounds only**: the 50 dimensions whose `source_origin.license` is `citation-only` (28) or `research-use-reference` (22), listed in step-30 §6.6, until their owners' terms are confirmed (re-admission is a reviewed pack change citing the confirmation); and the Persona-1M coreset (`matraix-research-only`, derived from real people's records), never imported. 1,240 dimensions remain. Redundancy is the mask's job, never a reason to drop one.
2. **Pinned.** `persona/schema/dimensions.json` (schema 1.0) at upstream commit `3633d8dab149a9482a71b024418a49ae828cc941`, SHA-256 `cbf4db9d887dec057431aff409e1d28d83f345e7a31e29a4275a607212ecfca5`, imported verbatim: ids, categories as groups, value lists, phrases and the six label packs, nothing renamed.
3. **Overlay** inside the pack, keyed by their ids and never editing their file: visibility (default `learnable`; `private` for ethnicity, religion, sexual orientation, disability, citizenship, veteran status, income and every `health_*` dimension, Q-S30-8), mutability (`derived` `age_bracket` from `core.birth`), owner (`economy` for `socioeconomic_band`), facets, and level order.
4. **Masks** `matraix-core-100` (default; ≤ 100 chosen by gameplay, each with its `why`) and `all`; **mapping** `mappings/core.yaml` to the built-ins readers declare (for example `core.chronotype ← lstyle_sleep_schedule`, `economy.wealth-band ← socioeconomic_band`, S30-a F-S30a-2), every target value existing and every source value covered.
5. **Update procedure.** The pack's offline import tool (Python standard library) verifies the digest, re-applies the exclusions and regenerates the pack; CI re-runs it over the pinned inputs and requires byte equality; an upgrade is a reviewed PR with a diff report; a removed dimension that a mask, mapping or profile names fails by name; a removal is a major version.
6. **Attribution.** `pack.yaml` (`license: MIT`, `attribution:`), `LICENSE` (upstream text), `NOTICE` (commit, digest, paper), and the line *"Optional person-generation backend built on the MatrAIx persona schema (Li, Hao et al., 2026, arXiv 2608.04205; MIT), imported unchanged."* in the README's backends row, the demo's guide chapter, and a `CITATION` entry.
7. **Isolation.** Nothing outside the pack names it, except `worlds/matraix-demo/`, its guide page, the README's attribution line, the plans and this file: a CI grep refuses any other mention (X-S30-18), and a CI job builds and tests the workspace with the pack removed (X-S30-19). Their sampler code is not used (`DEP-67`).

**Why not ourselves.** A broad, documented, MIT persona schema with phrases and translations exists; re-authoring 1,240 dimensions would reinvent it. **Why not as the core.** It has no appearance or numeric kind, it is oriented to software-user personas, and a hard dependency would make every world depend on an external release (RR-S30-5).

**Consequences.** The workspace, every reference world and every test run without the pack; a world uses it only by requiring it and naming `backend:` for it. `worlds/matraix-demo` (S30-h) shows its attributes in casting briefs, "who I am" and "who I see", the agent description and both inspect panels, including a zh-Hans panel from its label pack, and a CI-executed guide section walks through switching backend and mask and back. Under it, the appearance group and quantities still come from `core` (`ARC-125`, `ARC-126`).

**Revisit** when a pinned upstream release changes the schema, when an excluded dimension's licence is confirmed, or when the authors' terms for joint promotion are agreed (the operator's).

---

## DEP-67 — The population sampler is our own; pgmpy, pomegranate, PopulationSim and MatrAIx's sampler code declined; MatrAIx's DAG as an optional `log-linear` mode

**Date** 2026-10-15 · **Status** accepted (step plan approved 2026-10-15; Q-S30-9 ruled 2026-10-15); implemented by S30-d, the `log-linear` mode by S30-h · **Approved by** the operator, directly in the primary session, 2026-10-15 (step-30 approval; Q-S30-9) · **Relates to** `ARC-123`, `ARC-126`, `DEP-66`, [`REUSE_POLICY.md`](REUSE_POLICY.md) §§11–12 · **Design** `.structured-coding/plans/mvp0/step-30-person-attributes.md` §§8.6, 16.3

**Problem.** Genesis needs a dependency-aware, stratified sampler that is integer, deterministic on Linux, macOS and Windows, shared by the world's genesis and the casting CLI, and that never re-draws authored or ruled values. `CLAUDE.md` §4 rule 16 asks both halves of the reuse question.

**Options considered** (step-30 §8.6).

| Candidate | Verdict |
| --- | --- |
| `pgmpy` (MIT, Python) | Declined: Python while genesis is Rust; float sampling; a dependency larger than the problem (≈ 400 lines of sampler) |
| `pomegranate` (MIT, Python) | Declined, for the same reasons |
| PopulationSim (ActivitySim, BSD-3) | Declined for MVP-0: it solves household synthesis against zone control totals (IPF), in Python with pandas; recorded as the reference if MVP-1's region worlds need joint fitting |
| MatrAIx's sampler code | Not used: float Python. Its formula — prior × likelihood ratios × hard masks — is the one `ARC-123` uses |
| **Our own, in `mineworld-attributes-model`** | **Chosen:** integer, deterministic, one implementation with two entry points |

**Choice.** The sampler is MineWorld's, in the model crate (S30-d). **MatrAIx's `full_dag.json`** (1,308 nodes, 6,999 directed proposal edges with a valid topological order, 524 masks, 54 full CPTs) **is offered as an optional `log-linear` mode of its backend only** (Q-S30-9, ruled 2026-10-15): the import tool converts each term to fixed-point log weights, the engine sums them and exponentiates through a committed integer table, cycles are refused at import, and their energy-scoring repair is not reproduced. Its priors are global, so it is conditioned on adults and is never a place's distribution; it is used by `worlds/matraix-demo`, never by a reference world. S30-h prototypes it first and drops it, reporting to the operator, if determinism, load time (a 24.8 MB file) or the ≤ 2 pp fidelity check against their own sampler on the masked set fails.

**Why not them.** Each library is float and Python, outside genesis's language and determinism; the interim Python casting sampler is removed with no adapter (`CLAUDE.md` §4 rule 12). **Not forced:** no library's execution model owns our genesis.

**Consequences.** No sampling dependency is added. The sampler is about 400 lines of Rust that MineWorld maintains, tested by golden draws on three platforms (AC-S30-7) and by planted mutations (an `f64` draw, a re-drawn ruled value, a read display name; step-30 X-S30-6, X-S30-13, X-S30-14). The interim casting sampler on `mvp0/s26-cast-director` is removed by S30-g.

**Revisit** when MVP-1 region worlds need joint fitting (PopulationSim), or if a deterministic Rust library solving the same problem appears.

---

## DEP-68 — Population data sources: ACS and NHANES (public domain), the Monk Skin Tone scale (CC BY 4.0)

**Date** 2026-10-15 · **Status** accepted (step plan approved 2026-10-15); used by S30-e's cited profiles · **Approved by** the operator, directly in the primary session, 2026-10-15 (step-30 approval; Q-S30-5 for `verify` marks) · **Relates to** `ARC-95`, `ARC-123`, [`REUSE_POLICY.md`](REUSE_POLICY.md) §§11–12 · **Design** `.structured-coding/plans/mvp0/step-30-person-attributes.md` §§6.2, 8.2

**Problem.** Realistic defaults must follow real-world values with citations, and a profile must keep data apart from authored casting conventions (step-30 F-S30-5).

**Options considered.** Authored figures only — declined: unrealistic, and indistinguishable from data. Commercial demographic datasets — declined: licences incompatible with a CC0 pack. **Public-domain U.S. federal statistics, plus an openly licensed skin-tone scale, each cited per figure** — chosen.

**Choice.**

| Source | Licence | Used for |
| --- | --- | --- |
| American Community Survey (U.S. Census Bureau), e.g. 2024 1-year tables B01001, B03002 for San Diego County | public domain (U.S. federal work) | sex, age-band and ancestry marginals |
| NHANES 2015–2018 anthropometric reference data (Fryar et al. 2021, Vital Health Stat 3(46)) | public domain (U.S. federal work) | stature and body-mass by sex and age; build |
| Monk Skin Tone scale (Ellis Monk; released by Google, 2022) | CC BY 4.0, attribution in the pack | the ten-level `core.skin-tone` domain |

Each figure in a profile names its source and is marked `read` or `verify`; casting conventions are sources of status `authored` and are never presented as data. A release refuses a reference world whose data figures are still `verify` (Q-S30-5).

**Consequences.** The default domain ranges follow real values: stature 0.50–2.50 m and mass 2–300 kg as domains only, the realistic distribution being the profile's. A county figure not yet read (for example build, step-30 §8.2) stays `verify` until it is.

**Revisit** when a world needs a population outside the United States, which brings its own sources and a record of its own.

---

## ARC-110 — The default suite runs as weighted shards on every OS, behind gates that keep the check names

**Date** 2026-10-15 · **Status** accepted for review (PR #224, "ci: shard test-windows under the job cap"); in force at its merge · **Approved by** the operator through the primary session, 2026-10-15: shard the Windows test step and keep check names stable; scope extended the same day to `test` and `test-macos` and to the Windows Python subprocess bound, and on 2026-10-16 (primary ruling, after the move to GitHub Pro) to at most two macOS jobs per run; merge is the operator's · **Relates to** `ARC-48` and its notes of 2026-10-09 (13b, 13w), `ARC-83`, [`ENGINEERING_STANDARDS.md`](ENGINEERING_STANDARDS.md) §§15–16, [`REUSE_POLICY.md`](REUSE_POLICY.md) · **Design** this entry; `scripts/ci_shard.py`

**Problem.** The `core` layer outgrew one 60-minute job on all three operating systems, and every run that hit the cap was cancelled with every test it had run green.

- **Windows.** PR #200's run 38787218062 and PR #206's run 38782285755 were cancelled after 78 and 80 of the suite's 250 test binaries.
- **Linux and macOS.** With 12d (#171), the suite took 48 and 40 min on main (run 38787218062). On #171's run 38786685064 it was cancelled at 60 min on both, inside `town_bodies`: two 300-day town runs, ~390–440 s on its green run 38664714380.
- **12d is slower on every binary.** Measured per binary as wall time between cargo's `Running` lines, the binaries #171 shares with main take 1.3× (Linux), 1.6× (macOS) and 1.7× (Windows) as long.

Raising the cap again (as `#182` did for `test`, 45 → 60) only moves the wall, and makes every code PR wait over an hour.

**Choice.**

1. **Shards.** Each test job is a matrix of shards running `scripts/ci_layer.py core --shard K/N`, through `.github/actions/layer` (Linux, container) or `.github/actions/native`, each with a `shard` input:
   - `test (1/3)…(3/3)` on ubuntu-24.04;
   - `test-windows (1/4)…(4/4)` on windows-2025, the slowest runner;
   - macOS in two jobs on macos-26, as item 3 below sets out.

   Every Linux and Windows shard's cap is 45 min. Every PR still runs the whole suite.
2. **The partition** is `scripts/ci_shard.py`'s, and `ci_layer.py` stays the only place that names a layer's commands.
   - **Units.** A unit is a test target name across the workspace. `cargo test --workspace --test NAME` runs every package's target of that name from its own directory, using cargo's own filter, not a re-implemented runner. Two more units cover the rest: `<lib+bins>` (`--lib --bins`) and `<doc>` (`--doc`, which cargo cannot mix with other filters).
   - **Names.** They come from `cargo metadata` at run time, so a new test is in exactly one shard.
   - **Weights.** They are Windows seconds from the first complete sharded run (2855 s on main 4457ad47), plus `town_bodies` at 480 s, its share of `market_town` on #171. One table serves every OS: only proportions matter, and they agree across the three. An unknown name weighs 1 s.
   - **Assignment.** Longest-processing-time first, with fixed tie-breaks, hence deterministic.
   - **Per shard.** Each shard builds (`--no-run`) and runs only its own targets, then `check_scratch.py left`, as `core` does. Linking a third of the test binaries instead of all of them cut the Windows build from 773 s to 181–363 s.
   - **Self-test.** `fast` runs `ci_shard.py --self-test`: completeness, no duplicates, determinism, the LPT bound, and that the commands run every unit once.
3. **macOS: two jobs per run, because macOS concurrency is the binding constraint.** GitHub-hosted macOS runs at most 5 jobs at once on Free, Pro and Team alike, while Linux and Windows get more slots on Pro. A run with five macOS jobs (three `test-macos` shards, `platforms (macos-26)`, `python (macos-15)`) held every macOS slot, so the next run's macOS stage waited for it. The macOS jobs are now:
   - **`test-macos (a: lock, core 1+2/3)`** runs the `platforms-lock` layer, then shards 1 and 2 of 3 together (`--shard 1,2/3`). `platforms-lock` is `cargo fetch --locked` and EC-3 (b)'s `--offline-check`, run first so that nothing can rewrite `Cargo.lock` before it is checked. These two commands are what the `platforms` layer checks on macOS beyond `core`: every test command of `platforms` is a subset of `cargo test --workspace` on the same runner, and `cargo fetch --locked` fails on a stale lock as `--locked` on each command did.
   - **`test-macos (b: core 3/3, python)`** runs shard 3 of 3, then the `python` layer, with uv's Python 3.12 (`setup-uv`). The Python workspace's macOS run moves from macos-15 to macos-26, the version every other macOS job uses.

   `scripts/ci_layer.py` takes several layers in order, stopping at the first failure. A shard spec may name several shards of one partition (`K,L/N`), whose units run as one. On head 3045db8b, before 12d: macOS `core` was 1806 s of tests plus 130–240 s of build per shard, `python` 21 min (`lm-controller` pytest 823 s, a cold `cargo build` 294 s), and `platforms` 8.3 min. That puts (a) at about 27 min and (b) at about 31 min. One unsharded macOS test job would be about 39 min before 12d and over 45 min with `town_bodies`, so the suite still needs two jobs. Their cap is 55 min, not 45. 12d's effect on macOS was measured at 1.1× (run 38664714380) and 1.6× (run 38786685064) of main's suite, and at 1.6× both jobs come to about 46–49 min. When either passes 40 min, re-weigh or give macOS a third job.
4. **The check names do not change.** `test`, `test-macos` and `test-windows` become gate jobs on ubuntu-24.04.
   - Each needs its shards and is green only when every shard is (`needs.<shards>.result == success`).
   - Each has its shards' triggers plus `always()`, so a failed or cancelled shard is a red gate, and a documentation-only change still skips both gate and shards.
   - The required checks stay `fast`, `test`, `test-macos`. The shard checks are new and not required. `platforms (macos-26)` and `python (macos-15, python)` disappear (their work is in `test-macos (a)` and `(b)`); neither was required.
5. **The Windows bound on one Python-driven binary run** (`cognition/lm-controller/tests/binary.py`) goes from 600 to 1200 s; Linux and macOS keep 600 s.
   - On run 38786685064 the 100-day social-cafe run behind the AC-10 and retention fixtures passed 600 s on windows-2025 and was killed (10 errors, `TimeoutExpired`).
   - On Linux and macOS, every binary run finished inside 600 s. Their one failure there, `test_keep_all_is_held_by_the_host_on_the_real_history`, is an assertion of #171's, not a timeout.
6. **Not adopted: cargo-nextest.**
   - Its `--partition hash:`/`count:` splits by test count, not weight, so the ~490 s `market_town` and `town_bodies` can share a shard.
   - It runs each test in its own process, which changes the semantics of tests that share a binary's state.
   - It does not run doctests.
   - It is a new tool to pin on three platforms.

   The weighted split over cargo's own filters is about 200 lines of standard-library Python. Revisit if per-test retries or JUnit output are wanted.
7. **Re-weigh** when any shard's test step passes 35 min: take new per-binary times from a run's log and replace `WEIGHTS`.

**Consequences.**

- **Runners.** A run uses seven Linux and Windows test runners and two macOS ones, instead of three test jobs plus separate macOS `platforms` and `python` jobs. Each job builds the workspace's libraries itself. The wall time to a verdict falls from >60 min to about 30 min.
- **macOS.** Two macOS jobs per run (down from five) let two or more PR runs share the five macOS slots.
- **The nightly's flake sampler** (`ci_nightly.py`, `REPEAT_OS`) reads `main`'s `test`, `test-macos` and `test-windows` jobs. A green gate stays a passing sample, but a red gate has no test transcript, so it counts as an inconclusive sample rather than a failed one.
- **Nightly repeat jobs.** The `repeat-*` jobs run unsharded and are unchanged. At these suite times `repeat-windows` (two samples, 120 min) is likely to need the same treatment.

---

## ARC-74 — The `persona` System Pack: a person's authored character, which shapes speech and is disclosed to the person alone

**Date** 2026-10-14 · **Status** accepted (design, S10 P6 revision 2, §5.14); implemented by P6a · **Approved by** the operator (QP6-1, QP6-2, 2026-10-14) and the primary session of S10 P6 · **Relates to** `ARC-28` (point 6, self-only disclosure), `ARC-31` (a System Pack owns a section of an authored file), `ARC-33` (the installed set), `ARC-72` (the LM controller's composition), `CORE_CONCEPTS.md` §4.2 (personality is a trait) · **Design** `pr-s10-p6-lm-controller.md` §§5.14, 6 (I-P6-1), 8 (AP6a-1 … AP6a-3)

**Problem.** A person has a name (`naming`) and a routine (`schedule`), but nothing in the world says what they are like. A controller that speaks for a person has no source of character to speak from. `note` is authoring provenance, "never gameplay state" (`MODULE_SPEC.md` §4.1), and no System reads it, so it cannot carry one. Putting the character in the operator's cognition configuration would make it a property of one operator's machine, not of the world.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) A System Pack that owns a `persona:` section of the person's file, and a `Persona` component disclosed to its holder only | **Chosen** |
| (b) The operator's cognition configuration (`cognition.toml`) | Declined: a person's character is world content. It would be lost when the world moved to another operator, and a world without a controller would have no character at all |
| (c) `note:` | Declined: `note` is authoring provenance and is never read as state. Making it state would mean a System reading prose it does not own |

**Choice.** Option (a). The character is world content, so it is authored in a person's file and owned by a System, as `CORE_CONCEPTS.md` §4.2 places personality among traits. The pack has no action, no process and no reaction. Its `Persona` component is disclosed through `PerceptionProvider::discloses` to its holder and to nobody else, as `ARC-28` point 6 discloses relationships. It shapes speech only (`R-S10-9`): it grants no ability, moves nobody, and a rule controller ignores it.

**Consequences.** Any world that enables `persona` gets a genesis fact per authored person, private to that person (`Visibility::Entities({holder})`), so its save bytes change; the baseline rules of `scripts/baselines.txt` apply to such a world. A World Pack author may write a character in any person's file without changing a rule, and a world without any `persona:` section is unchanged. A controller that wants to use the character reads it through the holder's own observation; no other observer sees it.


---

## ARC-94 — The per-character standard: literal gates with sources, a scorecard, and a failed-generation policy

**Date** 2026-10-14 · **Status** accepted (design, step-26) · **Approved by** the operator's step-26 rulings of 2026-10-14 · **Relates to** `ARC-9`, `DEP-8`, `DEP-51`, `DEP-52`, `DEP-55`, `ARC-23` · **Design** step-26 §§6, 15; `pr-s26b1-pipeline.md` §§5.6–5.8

**Problem.** Every protagonist-quality character is generated from a third-party mesh, and its acceptance has so far rested on one person's judgement recorded in a long note. A second character would be judged by a different eye against a different standard, and the bounds would drift toward whatever the candidate already is.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) Per-character judgement only | Declined: no reproducible bar; the next character is judged against the last one's mood |
| (b) Gates without a scorecard | Declined: a pass on the gates does not show the characters are of the same quality tier |
| (c) Literal gates with a source per bound, a scorecard row per character, quality-tier parity with the protagonist as the control, and a failed-generation policy | **Chosen** |

**Choice.** Option (c). The standard is step-26 §6 in substance: gates G-1…G-14 with each bound a fixed literal from the protagonist's accepted record or a named real-world reference (`ARC-23` rule 2); each gate prints what it looked at before its verdict (`ARC-23` rule 1); each batch has two review pages (the static scorecard and the visual review; the runtime page is parked for the large milestone, Q-S26-6); a failed generation is recorded in `generation_log.yaml` with its reason and is never silently replaced. The gates are code in `tools/character_pipeline/mwcast/gates.py`, with their bounds in `standard.py`.

**Consequences.** A bound changes only by amending this record. A gate that cannot look (for example G-4(b) without a run directory) reports FAIL with the reason, not PASS. Runtime gates G-7…G-13 are built in S26-b2; until then they are recorded "by parity" against the accepted asset, never as PASS.

**Revisit** when the first batch of new characters is judged, or if a bound must move for a reason outside the protagonist's record.

## DEP-51 — Meshy image-to-3D as the cast's mesh source, at REST only and with the model pinned per batch

**Date** 2026-10-14 · **Status** accepted (design, step-26) · **Approved by** the operator's step-26 rulings of 2026-10-14 · **Relates to** `DEP-8`, `ARC-9`, `ARC-94` · **Design** step-26 §§4.5, 5.2; `pr-s26b1-pipeline.md` §§2.3, 5.11–5.12

**Problem.** The cast needs a generated mesh per character at a cost and quality the protagonist's route already showed. The source's terms bind the output.

**Options considered.** Meshy image-to-3D over REST, with the model and parameters pinned per batch (chosen), against other image-to-3D services and against hand modelling. The protagonist's route (D+) already used Meshy and recorded its credits and verdicts.

**Choice.** Meshy image-to-3D over its REST API only. `ai_model` is pinned per batch (`meshy-7.1` for Vera), `pose_mode` is `a-pose`, no remesh, a 2k texture, `remove_lighting` on. Ownership is as `DEP-8` records under the paid plan; B1 re-read only the Terms' §2.4 and their date (last updated 2026-09-19, read on 2026-10-14), not the ownership clause. The Terms are re-read at each batch's start (R-S26-11). Downloads are kept unmodified in scratch and never committed. A model change is a new pilot, not a silent swap.

**Identifiers, verified 2026-10-14.** Section 2.4 of the Terms reads: "The Service may include machine-readable metadata, digital watermarks, or other identifiers in Customer Output to identify that the Customer Output was generated by AI." and "You agree not to remove, alter, disable, or otherwise tamper with such identifiers." This matches the D+ record's quote (`docs/references/CHARACTER_ROUTE_D_PLUS.md` §4) in substance. Identifiers are kept and never stripped; a committed Meshy-derived asset carries `ARC-9` provenance.

**Consequences.** Every Meshy call is made by the operator's batch under the cap in `DEP-51`'s ledger; no test or CI job calls Meshy (the client is tested with an injected transport). The Terms date and the §2.4 quote are re-checked at every batch start.

**Revisit** when the Terms change in substance, or when a model change is proposed.

## DEP-52 — Blender, headless and pinned, as the mesh runtime; no Blender in CI

**Date** 2026-10-14 · **Status** accepted (design, step-26) · **Approved by** the operator's step-26 rulings and Q-B1-4 (2026-10-14) · **Relates to** `DEP-8`, `ARC-9`, `ARC-94`, `DEP-55` · **Design** step-26 §§5.1–5.2; `pr-s26b1-pipeline.md` §§5.10, 5.13

**Problem.** Mesh decimation, bone fitting, weight painting, texture baking and export need a DCC tool that runs headless and is scriptable.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) Blender, pinned to `Blender 5.2.2 LTS`, run headless with `--factory-startup`, scripts in `tools/character_pipeline/blender/` under GPL-2.0-or-later | **Chosen** |
| (b) A pinned Blender in CI | Declined: the Meshy download and the accepted intermediates are outside the repository, so CI would have nothing real to run on; the bake is device-dependent (Cycles on Metal against a CPU runner cannot be compared texel for texel); a several-hundred-megabyte download per OS per run for a stage that changes only with `blender/`; CI owns the stdlib part (`§5.10`) |
| (c) A Python mesh library (trimesh, pygltflib) for the stages | Declined: the D+ stages use Blender's bake, weight-painting operators and glTF export; a port to a library would reimplement them and the parity claim would then be about a different algorithm |

**Choice.** Option (a). The licence split is explicit: the driver is MIT and standard-library; the Blender scripts are GPL-2.0-or-later with their own `LICENSE` (byte-identical copy of `clients/3d-spike/tools/blender/LICENSE`), and they are never imported, only run as subprocesses. Blender's licence page (read 2026-10-14) states that "What you create with Blender is your sole property", so the outputs are ours. The pinned version is checked at run start; a different version needs `--allow-blender` and is recorded as a new parity baseline. The Blender stages are proven by the local real-lifecycle Gate on macOS (B10), re-run whenever `blender/` changes; CI is not asked to run Blender.

**Consequences.** Q-B1-4 is recorded here: no Blender in CI, for the four reasons of `pr-s26b1-pipeline.md` §5.10. Blender's bake device is recorded in every run manifest.

**Revisit** when a CI runner can hold the inputs without the operator's download, or when a second device needs a parity baseline.

## DEP-55 — Rigging: the fitted CharMorph armature is kept; the other rig sources are declined with reasons

**Date** 2026-10-14 · **Status** accepted (design, step-26) · **Approved by** the operator's step-26 rulings and Q-B1-2 (2026-10-14) · **Relates to** `DEP-8`, `DEP-51`, `DEP-52`, `ARC-94` · **Design** step-26 §5.4; `pr-s26b1-pipeline.md` §§2.2, 14.1

**Problem.** The protagonist's route rigged the mesh on a 52-joint CharMorph-derived armature (`vitruvian.glb`). The cast needs a rig source per character, and four others were considered.

**Options considered and verified facts (2026-10-14).**

| Option | Fact checked | Verdict |
| --- | --- | --- |
| Our fitted CharMorph armature (`vitruvian.glb`) with one recorded number (`charmorph_front_y_at_hips`) | D+ reproduced exactly (A-B1-9) | **Chosen** (`Q-B1-2`, option a) |
| Meshy rigging API as the rig | The rigging page (docs.meshy.ai/en/api/rigging) states no bone count, no finger bones, and no per-task price; it says rigging "currently only works well with standard humanoid (bipedal) assets" | **Declined as the rig.** Evaluated in S26-b2 as a joint source only (`step-26` §5.4 R) |
| Mixamo | Not fetched in B1; declined on the reason recorded in step-26 §5.4 | Declined (step-26 §5.4); re-verified before any use |
| AccuRIG (Reallusion) | Product page and CLI/SDK availability could not be read on 2026-10-14 (HTTP 404 and 403 from the fetcher); no command-line interface is verified | Declined as the rig; the decline does not depend on the CLI question. Re-checked before S26-b2 |
| UniRig (`VAST-AI-Research/UniRig`) | MIT licence; README requires a CUDA GPU with at least 8 GB VRAM and Python 3.11 with PyTorch >= 2.3.1 | Declined for the cast's CI-free local path and for the hardware floor; evaluated in S26-b2 only if its weight quality is shown |
| RigNet (`zhan-xu/RigNet`) | GPL-3.0 (`LICENSE-GPLv3`) | Declined: a GPL-3.0 dependency in the mesh path would bind the pipeline's licence; not used |

**Choice.** Our fitted armature is kept. **Amended 2026-10-14 (primary ruling F-B1-B10-2, option (b) of `Q-B1-2`):** the one recorded number is retired, because the CharMorph front surface is read from the committed CC0 body export `tools/character_pipeline/rig_source/body_g5.glb` (6.4 MB; SHA-256 `75b93806…`, provenance in `rig_source/PROVENANCE.txt`). The armature still comes from `vitruvian.glb`. S26-b2's weight-transfer prototype may still ask for the body's own weights under `DEP-53`.

**Consequences.** The rig owns a 52-joint skeleton whose bone names G-5 checks; Meshy's rig is not a joint source in this PR.

**Revisit** when S26-b2 measures a weight-transfer need, or when a rig source is shown to beat the fitted armature on the gates.

---

## ARC-149 — The nightly's Windows scratch on the local disk, the client projects imported through a cache, SET-b's connected 3D tests under QC-5, and the 3D client warmed before its tests

**Date** 2026-10-16 · **Status** proposed in the PR "ci: fix the red nightly (closes #170)"; pending the
operator's merge review · **Relates to** `ARC-83`, `DEP-45`, issue #170,
`.structured-coding/plans/mvp0/pr-13c-nightly.md` §3.4 (QC-5, QC-5b, the import cache) and §12.2
(F-13c-impl-2, F-13c-impl-5), `.structured-coding/plans/mvp0/pr-set-b-controls.md` §7.2

**Problem.** The nightly was red on every night since SET-b (#192) merged, for four causes, each read from
the logs of nightly 38919486641 on `38323d4c`, and two more this PR's own dispatches found:
1. `parity-long-windows` and `stability-windows` were cancelled at their 90-minute limit, and the
   `parity-long` comparison then failed for the missing Windows record. Windows saved runs were the cost:
   `stability-windows` saved bodies-yard's 300 days in 3 280.6 s (Linux 285.2 s) and replayed it in 77.5 s
   (Linux 70.2 s); `parity-long-windows` spent 3 550.2 s on bodies-yard alone. In-memory runs were within
   a factor of two of Linux. A measurement on `windows-2025` (run 39004172480, 30 days of bodies-yard,
   seed 7) placed it: in memory 14.8 s; saved under `TEMP` (`C:\Users\RUNNER~1\AppData\Local\Temp`, the
   OS disk) 263.4 s; saved under `RUNNER_TEMP` (`D:\a\_temp`, the runner's local disk) 38.8 s, and 38.6 s
   in the workspace on `D:`. Defender's real-time protection is off on that image
   (`RealTimeProtectionEnabled False`), and an exclusion changed nothing (208.8 s on `C:`, 37.6 s on
   `D:`). The scripts make their scratch with Python's `tempfile`, which follows `TEMP`, so every saved
   run of both legs wrote to `C:`. The Linux legs and the tests' own scratch (`CARGO_TARGET_TMPDIR`, under
   the workspace) were never affected.
2. `client_settings::two_d_the_settings_file_holds_only_settings_and_a_bad_one_is_kept` failed on every
   OS: its expected key list predates SET-b's `[input]` section. `store.gd` writes every owned section a
   file lacks (since SET-a), so a first Apply in the 2D client writes `mouse_sensitivity` and `invert_y`
   too, as `clients/shared/SETTINGS.md` §§3–4 and `AC-SETb-6` allow. **The test was wrong, not the code.**
   The Godot tests run only in the nightly, and SET-b merged before a night ran them.
3. `client_settings::three_d_controls_apply_live_and_persist` and
   `client_settings::three_d_settings_never_reach_the_server`, SET-b's two connected 3D slice tests, hung
   on Linux (llvmpipe under Xvfb) and on Windows (the Microsoft Basic Render Driver, D3D12 WARP): the
   slice built, seated and perceived, but did not render the probe's 12 warm-up frames within the test's
   240 s, so the probe never printed its header. That is F-13c-impl-5, the reason
   `three_d_switches_language_live_and_settings_never_reach_the_server` is already skipped there by QC-5.
   On the laptop's GPU all three of SET-b's tests pass (104.8 s, at this PR's head). On the macOS runner
   `three_d_settings_never_reach_the_server` passed that night.
   The macOS runner's own failure is a cold first launch. In this PR's first dispatched nightly
   (39007080793) the first connected 3D test there, `three_d_controls_apply_live_and_persist`, built the
   slice's scene in 58.1 s and hung before it was seated; the next two (`three_d_settings_never_reach_the_server`,
   `three_d_switches_language_live_and_settings_never_reach_the_server`) passed, and every later launch
   that night built the scene in 3.3–3.7 s. On `63069777` and `171ea061` the first connected 3D test was
   `three_d_switches_language_live_and_settings_never_reach_the_server`, and it was the one that hung
   (the triage of 2026-10-12 on #170). The shader and pipeline caches are cold at the first launch on a
   runner; that is the macOS part of F-13c-impl-5.
4. `clients-mac` was cancelled at its 75-minute limit with no verdict. `the_settings_folder_is_shared`,
   the first test to import the 3D client, took 45 minutes of it (9.5 minutes on Linux): the 3D client's
   imported resources are 584 MB, and `pr-13c-nightly.md` §3.4's import cache was specified but never
   built. The Godot tests then started at minute 65.

5. `client_2d::reconnects_to_a_restarted_world_and_to_a_replaced_one` failed on Windows on `63069777`
   and again on this PR's clients dispatch 39012891928 ("it saw the drop"), and passed on the other
   Windows nights. The client's own transcript in the failure shows the behaviour right: one
   `STATE reconnecting the connection closed`, a new `STATE seated`, `[PASS] reconnected 1 disconnect(s)`.
   The test counted the lines it had read only after `world.kill()` returned, and the client had seen the
   connection close, and its line had been read, while the kill was being reaped, so the slice after the
   count skipped that line. **The test was wrong, not the client.**

6. `repeat-windows` was near its 120-minute limit: two unsharded samples of the default suite took
   114.9 min on `38323d4c` and 107.2 min on this PR's dispatch 39007080793 (ARC-110's note expected it;
   the per-PR `test-windows` job had already outgrown its cap and been sharded).

The older findings of #170 are not live: `stability-linux` passed on `3a9b9eff` and `38323d4c`, and the
`baseline` step passed on both (only `compare` failed, for cause 1).

**Options considered.** For cause 1: sharding the two Windows legs by world (the comparison would have to
merge partial records, and bodies-yard alone would still take about an hour); running worlds concurrently
on the runner's four cores (timings become contention, and the cost is untouched); raising the limits
(3 550 s for one world of four leaves no limit with headroom); **moving the scratch to `D:`** (chosen: it
removes the cost, a 6.8-fold difference measured on the same runner, and changes no script). For cause 3:
a QC-5b ruling (not needed: the tests pass on the macOS leg, which is QC-5's condition); raising the
240 s deadline (the slice renders fewer than 12 frames in 240 s on a software renderer; a deadline it can
meet would not be a deadline); **QC-5, as for the existing name** (chosen). For cause 4: raising the limit
alone (every night would pay 45 minutes); **the specified import cache, plus a limit that holds a cold
night** (chosen). For the macOS cold launch: raising the tests' 240 s deadline (it would hold every
launch to the slowest one, and edits the tests); ordering the tests (libtest orders by name, and the
first would still pay); **one unjudged launch of the slice before the tests** (chosen: the tests and their
verdicts are unchanged, and it is not a retry, because nothing it does is judged).

**Decision.**
1. `parity-long-windows` and `stability-windows` point `TEMP` and `TMP` at `RUNNER_TEMP` before their
   layer runs (one step in each job, `nightly.yml`). No script names a disk.
2. The test's expected keys gain `invert_y` and `mouse_sensitivity`. The assertion stays an exact
   equality; nothing else in the test changes.
3. `scripts/ci_godot.py` `SKIPS` names SET-b's two connected 3D tests on Linux and Windows with the
   existing software-renderer reason, beside `three_d_switches_language_live_and_settings_never_reach_the_server`.
   The macOS leg runs all three every night, so a failure there turns the night red; `QC5B_RULED` and
   `HOSTED_UNRUNNABLE` are unchanged, and the coverage check still refuses a test skipped everywhere.
4. `.github/actions/godot` imports the four client projects (`clients/shared`, `clients/2d`,
   `clients/3d-spike`, `clients/protocol`) before any test, through `actions/cache/restore` and
   `actions/cache/save` (the pinned `actions/cache` commit). The key is the OS, the Godot build's
   `--version` and `hashFiles('clients/**')`; `restore-keys` start from the newest entry of the same OS and
   build, so Godot re-imports only what changed. The entry is saved right after the import, because
   `actions/cache`'s post step saves only when the whole job passes. A failed import is not cached and is
   left to the tests, which assert it by name.
5. `clients-mac`'s limit is 120 minutes (was 75), so a night whose cache is cold (45 minutes of import)
   still reaches a verdict. The other limits are unchanged.
6. The reconnect test counts the lines before it kills the world. Its assertions are unchanged.
7. `repeat-windows` becomes `repeat-windows-1` and `repeat-windows-2`: `ci_layer.py core-repeat --shard
   K/2` runs ARC-110's partition (`scripts/ci_shard.py`, the same weights), building shard K's targets and
   sampling them twice through `ci_repeat.py` (`--no-fail-fast`; the doctests, in whichever shard holds
   them, write `repeat-doc.txt`). The report joins an OS's summaries by sample name
   (`ci_nightly.py` `merged_samples`), so the flake verdict is still one per OS, and each shard is a
   verdict row of its own. Linux and macOS keep one job (80.8 and 64.6 min of 120).
8. `ci_godot.py tests` first launches the 3D slice once in its standalone `measure` mode
   (`godot --always-on-top --path clients/3d-spike res://scenes/slice.tscn -- --slice-measure
   --settings=none`, limit 600 s), where a connected 3D test is about to run (the macOS leg; Linux and
   Windows skip them all). Its exit and time go to `artifacts/nightly/notes.txt` as a note, never a
   verdict. On dispatch 39012891928 that launch took 406 s, and all three connected 3D tests then passed
   on the macOS leg.

**Consequences.** The evidence of the PR's dispatched nightly is recorded in the PR. Each Windows leg's
saved runs now measure the simulation and SQLite, not the runner's OS disk; a regression there is
visible in the benchmark table again. The import cache costs about 0.3 GB (compressed) of the repository's
Actions cache per OS per changed `clients/` tree. The repository's caches were at 10.6 GB of GitHub's
10 GB on 2026-10-17, so entries are evicted within hours (the Cargo and import entries of dispatch
39007080793 were gone by 39012891928, the macOS import entry survived); the 120-minute limit, not the
cache, is what guarantees `clients-mac` a verdict, and the cache shortens the nights it survives. `ci.yml`'s Windows jobs are
not changed here (#224 owns them); whether their scratch also lands on `C:` is a follow-up for that PR.
F-13c-impl-5 (the connected 3D slice on hosted runners) stays open with the 3D lane.

**Revisit** when the runner image changes its disks, or when F-13c-impl-5 is resolved and the QC-5 names
can run on every leg.

---

## ARC-109 — The memory home replaces the per-seat store; the seat view is memory's one storage API; schema 4

**Date** 2026-10-15 · **Status** accepted (PR design frozen 2026-10-15); implemented by S28-a · **Approved by** the operator's freeze of `pr-s28a-memory-home.md` (2026-10-15) and its rulings QS28a-1 … QS28a-4 (§14.1) · **Relates to** `ARC-59`, `ARC-87`, `ARC-98`, `ARC-99`, `ARC-101`, `DEP-37`, `DEP-58`, `DEP-61` · **Design** `.structured-coding/plans/mvp0/pr-s28a-memory-home.md` §§1, 2, 5, 6

**Problem.** P4's `MemoryStore` is one SQLite file per seat. Every seat that perceived a fact stores its envelope again, so a home of twelve persons stores most facts five to twelve times (step-28 §2). `ARC-98` rules that shared memory is a cognition-side home of perceived envelopes, with perception as the only access right; it leaves the storage API, the schema number and what replaces the store to the implementing PR.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) Keep `MemoryStore` beside the home, as a compatibility layer, until every consumer moves | Declined: a second storage path before a stable contract exists (`CLAUDE.md` §4 item 12), and QS28-8 rules that old stores are refused, not read |
| (b) One file per seat with shared records in an `ATTACH`ed file | Declined (step-28 §6.3): an atomic commit across attached files holds only in rollback-journal mode, and a seat file is no longer self-contained once its records are shared |
| (c) **One home per cognition process and world instance; every consumer reads and writes through a `SeatView`** | **Chosen** |

**Choice.** `memory.home.MemoryHome` (one SQLite file, schema **4**) replaces `MemoryStore`, which is deleted with no alias. A home stores each perceived fact's perspective-free envelope once (`fact`, `fact_who` with the role of each entity, QS28a-3), each seat's holding of it with that seat's rendering (`access`), and each seat's private tables with a leading `seat` column. `SeatView` is the only storage API of memory: ingestion, names, the roll-up, retrieval and the retention host take a `SeatView`, and its canonical dump is the standalone seat view (`ARC-101`). The schema is 4, because FG-b2 shipped schema 3 (F-S28a-1). A store of schema 2 or 3 is refused by name (`UnsupportedStoreSchema`) and never modified; memory is rebuilt from the perceived stream (QS28-8). Records are collected exactly, in the releasing transaction, by an anti-join over the access rows, with no stored count (I-S28-4). I-S28-6 is replaced by two statements (QS28a-1): sharing never costs disk, and each one-seat home's dump exceeds its seat view's by at most a fixed `K` bytes per held record.

**Consequences.** The kernel, contracts, server and protocol are unchanged (I-S28-10). A seat's bytes, recall and retention decisions are identical whether it shares its home or not (I-S28-2, I-S28-3), which the transparency harness checks. Until S28-f wires `sharing`, the LM controller opens one home per seat at the seat's existing path (the `none` arm).

**Revisit** when S28-c introduces the `MemoryScope` interface (the `world` scope's functions become a scope), or when S28-f shares one home among a process's seats.

---

## DEP-61 — The home's single-writer lock is SQLite's own exclusive locking; `filelock` is declined

**Date** 2026-10-15 · **Status** accepted (PR design frozen 2026-10-15); implemented by S28-a; no dependency is added · **Approved by** the operator's freeze of `pr-s28a-memory-home.md` (2026-10-15); QS28a-4 ruled by the primary · **Relates to** `DEP-37`, `DEP-58`, `ARC-109`, [`REUSE_POLICY.md`](REUSE_POLICY.md) · **Design** `.structured-coding/plans/mvp0/pr-s28a-memory-home.md` §4, §5.5

**Problem.** A home has one writer: a second process writing the same home would interleave its transactions with the first's. The second process must be refused at open, by name (`HomeBusy`), on Linux, macOS and Windows, and a process killed while holding the home must leave nothing that refuses the next open.

**Options considered.**

| Candidate | Verdict |
| --- | --- |
| **SQLite's own locking: `PRAGMA locking_mode = EXCLUSIVE`, then `BEGIN EXCLUSIVE; COMMIT` at open, `timeout = 0`** | **Chosen.** Cross-platform through SQLite's VFS (POSIX advisory locks, Windows `LockFileEx`), no dependency, no side file; the operating system releases the lock when the holder dies |
| `filelock` (Unlicense; mature) | Declined: a second lock beside SQLite's own, which can disagree with it, and a dependency for one call |
| A hand-written lock file (`fcntl` / `msvcrt`) | Declined: two code paths, and a file that goes stale after a SIGKILL |
| A side file recording the holder's pid (`HomeBusy(path, holder_pid)`) | Declined (QS28a-4): it can go stale and disagree with SQLite's lock; `HomeBusy(path)` names the file, and the platform's tools find the process |

**Choice.** SQLite's exclusive locking. `HomeBusy(path)` is raised when the lock cannot be taken at once. `auto_vacuum` is set only on an empty database, because on a database with pages the pragma rewrites the header even when the value is unchanged, and a refused file must be left byte-identical (S28-a ledger, F-S28a-10).

**Consequences.** Network file systems, whose locking SQLite documents as unreliable, are out of scope. The lock is tested across processes on all three platforms in CI (`test_home_open.py`), including a holder killed with `os._exit`.

**Revisit** if the `HomeBusy` test fails on a platform; the recorded alternative is `filelock`, by decision.
