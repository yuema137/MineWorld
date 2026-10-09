# MineWorld Package Ecosystem

**Status:** frozen package model
**Audience:** coding agents and pack authors.

This document defines what can be published for MineWorld, how it is packaged, and what an
extension is allowed to run. It settles three things before packaging decisions calcify:
**Asset Pack** as a pack type, the **`.mwpack`** format, and the **extension tiers**.

The governing rule, stated once:

> **A MineWorld package is not a Godot mod.** Godot is one renderer backend. The public
> extension format is engine-neutral, and an engine-specific artefact is something MineWorld
> *builds*, never something an author publishes.

Related: [`MODULE_SPEC.md`](MODULE_SPEC.md) (the pack types), [`ART_DIRECTION.md`](ART_DIRECTION.md)
(style), [`DECISIONS.md`](DECISIONS.md) (`ARC-7`, `ARC-8`, `DEP-7`).

---

# 1. Six pack types

```text
Asset Pack         models, images, textures, animation, audio        no code
World Pack         persons, places, organizations, initial state     no code
Presentation Pack  references, style manifest, lighting, bindings    normally no code
System Pack        components, actions, events, simulation rules     code optional
Controller Pack    rule policy, model config, decision logic         code optional
Entity Pack        what kinds of thing can exist                     no code
```

**Asset Pack is new, and it is deliberately separate from Presentation Pack.** The same tree,
bench, café and person model gets reused across many styles and many worlds; binding them to one
presentation would force a copy per style. A Presentation Pack says *how this world should look*;
an Asset Pack *is the material* it looks like.

This is the sixth type, and [`MODULE_SPEC.md`](MODULE_SPEC.md) §1 requires a sixth to be a
deliberate design change rather than a convenience. It earns it where a Presentation Style Pack
did not (`ARC-1`): a style pack described the *internal layout* of an existing type, while assets
are separately authored, separately licensed, separately versioned, and reused by packs that
never share a style.

## The particle analogy, completed

```text
Entity        →  the particles
System        →  the interactions
World Pack    →  the initial conditions
Presentation  →  the detector you observe it through
Asset Pack    →  what the detector is made of
```

---

# 2. Canonical asset formats

| Kind | Canonical format | Why |
| --- | --- | --- |
| 3D model | **glTF 2.0 — `.glb` preferred** | Khronos designed it as an API-neutral runtime delivery format carrying mesh, materials, textures, skins and animation; `.glb` packs it into one binary. Godot recommends it as its own import path, and Blender, Unreal and web renderers all read it. |
| 2D image | `.png`, `.webp` | transparency, lossless where it matters, universally readable |
| Audio | `.ogg`, `.wav` | unencumbered and engine-neutral |
| Texture | `.png`, `.ktx2` where compression matters | |

MineWorld does **not** accept `.tscn`, `.blend`, `.uasset` or any engine-native scene as a
published asset format. A renderer adapter imports the canonical form; it does not define it
(`DEP-7`).

## 2.1 2D and 3D are not the same problem

A 2D presentation is close to `image + animation + layout + audio`, and for the illustrated
direction MineWorld defaults to, a character can be a transparent PNG with layered scenery and
simple procedural or skeletal animation. Asset production, not code, is the hard part.

A 3D character is an order of magnitude more: mesh with UVs and weights, skeleton, materials for
skin and clothes and hair, an albedo/normal/roughness/metallic/AO texture set, an animation set,
collision, LODs. A building carries its own equivalent. So the hard question in 3D is never
*"is there a model?"* — it is:

> **do these models look like the same game?**

Sections 3 and 4 exist to answer that mechanically rather than by taste.

---

# 3. The asset contract

A model says what something *looks like*. MineWorld adds what it *means*.

```text
assets/modern_cafe/
  asset.yaml
  cafe.glb
  preview.png
  LICENSE
```

```yaml
id: modern-cafe-01
type: place_visual

format:
  model: cafe.glb

semantic_bindings:
  entrances: [front_door]
  interaction_anchors: [counter, table_01, table_02]
  spaces: [main_room, kitchen]

presentation:
  style_family: [mineworld-default-3d]

scale:
  unit: meter

license:
  spdx: CC0-1.0
```

`semantic_bindings` is the layer an ordinary asset store does not have: it is how a *model* is
attached to a *world* without the world knowing what a mesh is. An interaction anchor is where a
system's `Sit` or `Buy` is rendered as happening; it is **not** permission for the client to
decide that sitting is allowed ([`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §19).

---

# 4. Keeping a style coherent, in four layers

"Everything is semi-realistic" is not a specification. A Presentation Style Pack carries four
layers, of increasing machine-checkability:

| Layer | What it is | Who checks it |
| --- | --- | --- |
| 1 | **Reference images** | a human, at a glance — and they outrank the rest ([`ART_DIRECTION.md`](ART_DIRECTION.md) §16) |
| 2 | **Art direction prose** | a human or an LLM review |
| 3 | **Style manifest** | a tool, coarsely: realism, stylization, detail, lighting, character proportions |
| 4 | **Asset compatibility profile** | a tool, exactly |

Layer 4 is the one that makes contributed assets checkable:

```text
1 MineWorld unit        = 1 metre
character height        1.55 – 2.05 m
door height             ~2 m
materials               PBR required
texture resolution      1K – 2K recommended
LOD                     recommended
humanoid skeleton       MineWorld Humanoid Profile v1
```

so that a contribution can be answered by a command rather than an opinion:

```bash
mineworld validate asset my-house
✓ glTF valid          ✓ scale valid        ✓ materials PBR
✓ licence present     ✓ collision present  ✓ style declared
⚠ no LOD
```

---

# 5. `.mwpack`

One published artefact, for every pack type:

```text
japanese-town.mwpack        # a ZIP, deliberately

  manifest.yaml
  LICENSES/
  previews/
  assets/ | world/ | systems/ | presentation/
```

```yaml
id: mineworld.modern-cafe
version: 1.2.0
type: asset-pack

mineworld:
  requires: ">=0.2"

dependencies:
  - mineworld.modern-materials@^1

license:
  spdx: CC0-1.0
```

ZIP on purpose: cross-platform, openable by a human, trivial to attach to a GitHub release, and
bound to no engine. Semver and an SPDX licence identifier are required, not optional — a pack
whose licence cannot be resolved cannot be redistributed, and MineWorld redistributes.

**The package manifest's name is `pack.yaml`** ([`DECISIONS.md`](DECISIONS.md) `ARC-53`).
`manifest.yaml` is already a Presentation Pack's **style** manifest
([`ART_DIRECTION.md`](ART_DIRECTION.md) §12, [`MODULE_SPEC.md`](MODULE_SPEC.md) §6.1), and one file
name must not carry two schemas; the archive above carries `pack.yaml`.

## 5.0 Package identity in MVP-0

MVP-0 implements the identity part of the manifest, without an archive, for every pack, and states
it where the pack already states who it is (`ARC-53`):

```text
field       meaning                                         code pack          World Pack        pack.yaml
id          1–64 of a–z, 0–9, '-'; a letter first;          Cargo package      world.id (= its   id
            no '--', no trailing '-'                        name               directory)
version     semver                                          Cargo version      world.version     version
type        system-pack · controller-pack · world-pack ·    by its trait /     world-pack        type
            presentation-pack · entity-pack                 its crate
mineworld   framework range, semver                         (Cargo)            mineworld:        mineworld
license     SPDX expression                                 Cargo license      world.license     license
provenance  authors; repository where stated                Cargo authors,     (its directory)   authors,
                                                            repository                           repository
```

- A **code pack** records its Cargo fields at compile time with `package!()`
  (`SystemPack::PACKAGE`, required; [`MODULE_SPEC.md`](MODULE_SPEC.md) §3.1).
- A **World Pack**'s three fields are optional to the loader in MVP-0 and checked when present;
  `mineworld packs validate` requires them ([`MODULE_SPEC.md`](MODULE_SPEC.md) §4.1).
- A **`pack.yaml`** is read for a Presentation Pack (and an Entity Pack from S16's PR E-d). Its fields:

  ```yaml
  id: mineworld-default-3d            # required. Stated here; the directory's name is free
  type: presentation-pack             # required
  version: 0.1.0                      # required, semver
  mineworld: "^0.1"                   # required, the framework versions it works with
  license: MIT                        # required, an SPDX expression
  authors: [Yue Ma]                   # required, at least one
  repository: https://github.com/yuema137/MineWorld   # optional
  ```

  Any other field is refused by name, `dependencies` included: a world's requirements are resolved
  (below), a data pack's own dependencies on other data packs are not, in MVP-0. A `type` that
  `pack.yaml` does not carry is refused naming its carrier: `system-pack` and `controller-pack`
  (`Cargo.toml`), `world-pack` (`world.yaml`); `asset-pack` is refused in MVP-0.
- The framework's own version is **0.1.0**, shared by every framework crate and every bundled pack.
- **Bundled or third-party** ([`DECISIONS.md`](DECISIONS.md) `ARC-54`). A code pack is *bundled* when it
  was compiled from the framework's own workspace — decided at compile time by `package!()` from where
  the crate's manifest lies, and recorded as one boolean. Every other code pack is *third-party*.
- **Requirements** (`ARC-54`). A World Pack names in `requires:` every pack it uses that is not
  bundled, with a semver range; the build and the pack roots named by `--packs` and `MINEWORLD_PACKS`
  are searched, one installed version per pack is checked, and each failure is refused by name
  ([`MODULE_SPEC.md`](MODULE_SPEC.md) §4.1).
- **Licence policy** (`ARC-55`). A pack in a world's composition carries a licence the policy allows. The
  default allows `MIT`, `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `CC0-1.0` and
  `Unlicense`; an expression is judged by whether it can be satisfied with those alone. The policy is a
  typed value a world will be able to override through the generic configuration seam (S17).
  `CC-BY-4.0` is a candidate for asset packs once attribution is handled; it is not in the default.

## 5.1 Source pack versus runtime pack

```text
   author publishes              MineWorld builds
      .mwpack         ──────►      .pck  (Godot)
                                   future engine artefacts
```

Godot's own resource packs exist for exactly this and are documented for DLC, patches and mods —
but they are a *build output*. Authors publish `.mwpack`; a runtime may compile it into whatever
its engine loads. Conflating the two is how a project becomes a Godot project.

---

# 6. Extension tiers

What an extension may run, and what that costs in trust:

| Tier | What it is | Written in | Sandbox |
| --- | --- | --- | --- |
| **0** | Declarative content: assets, worlds, presentation configuration | YAML, JSON, media | nothing to sandbox |
| **1** | Simulation extension: a System or Controller with real behaviour | **WASM Component, interface described in WIT** | capability-based |
| **2** | Renderer extension: a shader, a water surface, a visual effect | Godot addon, GDScript, C#, GDExtension | engine-level, renderer-scoped |

**Most packs are Tier 0, and that is the point.** A Japanese hot-spring town — station, ryokan,
bathhouse, café, lake — is a World Pack plus Asset Packs plus presentation configuration, with
existing systems switched on. No Rust, no WASM, no code review. Code is needed only when a new
*law of the world* is added: farming means `Plant`, `Crop`, `Soil`, `Grow`, `Water`, `Harvest`,
and that is a System Pack.

## 6.1 Why Tier 1 is WASM and not the engine's language

Two reasons, and the second is the serious one.

**Engine neutrality.** A System expressed in GDScript binds world semantics to Godot, and
MineWorld's whole claim is that the world outlives its renderer.

**Safety.** A public MineWorld server downloads packs written by strangers. Arbitrary native,
Python or GDScript code in that position can read the filesystem, open sockets, and exfiltrate
API keys. That is not acceptable, and no amount of review policy fixes it. A WASM component gets
only the capabilities it is granted:

```text
granted:  world.read.entities · world.emit.events · world.query.time
absent:   filesystem · network · environment
```

The bet is current rather than speculative: the Component Model ships in real tools, WASI 0.2 is
stable, WIT generates typed bindings across languages, and plugin systems are specifically the
workload class for which server-side WASM is considered production-ready in 2026. Rust compiles
to it today; other languages follow as their toolchains mature.

Interfaces are declared in WIT, language-neutral by construction:

```wit
interface world {
    get-person: func(id: entity-id) -> person-state;
    emit-action: func(action: action-intent);
}

world mineworld-system {
    import world;
    export validate-action;
    export resolve-action;
}
```

## 6.2 What Tier 2 may never do

A renderer extension may decide how water looks. It may not decide whether Alice can afford a
boat. The moment a renderer addon answers a question about what is permitted, the architecture
has failed ([`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §8).

---

# 7. A DLC is a bundle

Nothing new is needed to express one:

```text
Lakewood Expansion
├── Lakewood North          World Pack
├── Lakeside Buildings      Asset Pack
├── Hiking                  System Pack
└── MineWorld Default 3D    Presentation Pack
```

Compared with Minecraft's resource pack / data pack / mod split, the boundaries here are drawn
one level finer, and the one that matters most is explicit: **System Packs are the rules of the
world**, separate from both the content and the look.

---

# 8. Status in MVP-0

Frozen as a design; implemented only as far as MVP-0 needs.

| Piece | MVP-0 |
| --- | --- |
| Asset Pack as a type, glTF canonical, asset contract | specified; the 3D spike is its first real test |
| `.mwpack`, `mineworld validate asset`, dependency resolution | after the vertical slice |
| Package identity (§5.0) | every pack: code packs by `Cargo.toml` and `package!()`, World Packs by `world.yaml`, Presentation Packs by `pack.yaml`; `mineworld packs list \| show \| validate`; framework 0.1.0 (`DECISIONS.md` `ARC-53`, S16 PR E-a) |
| Requirements and licence policy (§5.0) | a world's `requires:` resolved against the build and the pack roots (`--packs`, `MINEWORLD_PACKS`), one version per pack, each failure refused by name; bundled or third-party decided at compile time; the default licence policy; `mineworld packs resolve` (`ARC-54`, `ARC-55`, S16 PR E-b). A data pack's own `dependencies`, version selection and a registry: not in MVP-0 |
| A System Pack from another repository | installed by the same two lines, its Cargo line a git source pinned to a full commit id, locked by `Cargo.lock`; written against the published surface (`mineworld-sdk`, `-kernel`, `-contracts`, `-authoring`, `-presence`, `-inventory`), which the build maps once in `.cargo/config.toml`; `packs list` shows it `third-party`; guarded by `tests/acceptance/tests/package_sources.rs`; `cargo-deny` checks the graph's licences and sources in CI ([`MODULE_SPEC.md`](MODULE_SPEC.md) §3.2, `DECISIONS.md` `ARC-66`, `DEP-22`, `DEP-23`, S16 PR E-c). The first: `acme-fishing`. A registry, `.mwpack` and installing without a rebuild: not in MVP-0 |
| Tier 0 | what the sample worlds already are |
| World Pack fields | the subset [`MODULE_SPEC.md`](MODULE_SPEC.md) §4.1 specifies: identity, `systems`, `places`, `population`, `items`, `organizations`, `seats`, one content file per declared key (`places/`, `people/`, `items/`, `organizations/`; an authored Item is a kind, `DECISIONS.md` `ARC-36`), a person's `location`, and a place's `passages` (doorways between places, owned by the `movement` system, `DECISIONS.md` `ARC-26`), and **sections**: a top-level key of a person, place, item or organization file that a System Pack declares as its own, validates with its own type and turns into its own genesis facts (`ARC-31`; in MVP-0 `name`, owned by `naming`, `routine`, owned by `schedule`, `item`, owned by `item`, and `holdings`, owned by `inventory`, `ARC-37`; `economy`, owned by `economy`, and `job`, owned by `employment`, `ARC-38`); every other field of §4's model is refused by name |
| Tier 1 WASM/WIT | specified; MVP-0 ships trusted in-process Rust systems ([`ARCHITECTURE.md`](ARCHITECTURE.md) §12) |
| Tier 2 | available by virtue of Godot; unused |

The presentation spikes now carry a second meaning beyond looking good: binding one semantic
world to independently produced assets and styles is the smallest possible rehearsal of this
whole ecosystem.
