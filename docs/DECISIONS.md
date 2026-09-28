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

**Cost, recorded because it becomes a licensing question if the spike succeeds.** Unreal is free
below $1M USD trailing-twelve-month revenue. For a product whose runtime depends on Unreal, the
standard model is a 5% royalty on lifetime gross revenue above the first $1M per product, reducible
under Epic's "Launch Everywhere with Epic" terms. This is a summary and the EULA governs. It does
not affect MineWorld's own licence: the server, kernel, contracts and Godot clients are unaffected,
and an Unreal adapter would be a separately licensed deliverable.

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
