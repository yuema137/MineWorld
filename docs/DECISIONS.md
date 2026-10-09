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
