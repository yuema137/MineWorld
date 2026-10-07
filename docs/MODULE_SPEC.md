# MineWorld — Module Specification

**Status:** frozen module model for v0.1 planning
**Audience:** coding agents. This document defines what can be published, installed, and
composed. It is the boundary that separates MineWorld from a fork-and-edit starter kit.

Vocabulary: [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md). Layering: [`ARCHITECTURE.md`](ARCHITECTURE.md).

---

# 1. Six pack types

```text
Entity Pack        what exists in the world
System Pack        what is allowed to happen between the things that exist
World Pack         a specific world: population, places, organizations, initial state, config
Controller Pack    who decides what a character does      (also: Cognition Pack)
Presentation Pack  how the world is rendered
Asset Pack         the models, images, textures, animation and audio it is rendered from
```

A seventh kind of pack is a design change, not a convenience. Adding one requires changing this
document — which is exactly how `Asset Pack` arrived, as `ARC-7` records. Packaging, formats and
extension tiers for all six are specified in [`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md).

Extension model:

```text
install modules  →  compose world  →  configure  →  run
```

Never:

```text
fork source  →  modify game code  →  modify agent  →  modify frontend
```

---

# 2. Entity Pack

Declares the kinds of things that can exist. An Entity Pack contains no gameplay rules: it
cannot make anything happen.

```text
Modern Life Pack        Space Colony Pack
Person                  Human
Vehicle                 Android
Food                    Spaceship
Phone                   Spacesuit
Building                OxygenTank
Furniture               Airlock
Money
Pet
```

Contents:

```text
entity type definitions
component schemas the pack introduces as data shapes
item types
tags and taxonomies
authoring templates
```

Constraint: an Entity Pack declares structure. Behavior requires a System Pack. Installing
`Vehicle` without a transport system yields an object that exists and does nothing — which is
correct, not broken.

---

# 3. System Pack

Declares what is *allowed to happen*. This is where composability lives.

```text
ConversationSystem   InventorySystem     EconomySystem
EmploymentSystem     VehicleSystem       CombatSystem
RomanceSystem        EducationSystem     PropertySystem
CraftingSystem       InteriorSystem      GroupActivitySystem
```

A System Pack declares, per [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §13:

```text
ID
Version
Dependencies

Owned Components        the only state it may write
Provided Actions        what becomes possible in a world that enables it
Emitted Events
Subscribed Events

Configuration Schema
Migration Schema
```

Composition changes the genre while `Person` stays untouched:

```yaml
# World A — walking and talking
systems: [movement, conversation]
```

```yaml
# World B — life simulation
systems: [movement, conversation, inventory, economy, employment, property]
```

```yaml
# World C — survival
systems: [movement, inventory, crafting, survival, combat]
```

Hard constraints on a System Pack:

1. It writes only the components it owns; cross-domain effects are emitted as events (INV-7).
2. It never assumes the presence of a system it does not declare as a dependency.
3. It never references renderer assets, transports, or model providers (INV-14).
4. Removing it removes its actions and its state contribution without requiring edits to
   `Person`, to other systems, or to the kernel.

---

# 4. World Pack

What a world author actually produces: semantic configuration and population, not code.

```text
lakewood/
├── world.yaml
├── people/
│   ├── alice.yaml
│   └── bob.yaml
├── places/
│   ├── cafe.yaml
│   ├── apartments.yaml
│   └── park.yaml
├── items/
├── organizations/
├── scenarios/
└── dependencies.yaml
```

```yaml
world:
  name: Lakewood
  id: lakewood
  era: 2026
  calendar: modern
  geography: pacific_northwest

entity_packs:
  - modern-life

systems:
  - conversation
  - relationships
  - inventory
  - economy
  - employment
  - property

population:
  - alice
  - bob
  - charlie

cognition_profile:
  default: local_agents

presentation_profile:
  default: lakewood_realistic

network_profile:
  default: private_server
```

Constraints:

1. A World Pack contains no secrets and no credentials. It declares required *capabilities*;
   the operator supplies endpoints and keys.
2. A World Pack contains no renderer-specific assets; it names a Presentation Pack.
3. A World Pack never redefines simulation rules. If a world needs a new rule, that is a
   System Pack.

## 4.1 The fields MVP-0's loader reads

The model above is the frozen one and is unchanged. This subsection states the **subset MVP-0
implements**, because the loader (`worldpack/`) exists and an author needs to know what it accepts.
Fields of §4's model that MVP-0 does not implement — `era`, `calendar`, `geography`, `entity_packs`,
`cognition_profile`, `presentation_profile`, `network_profile` — are **refused by name**, not ignored:
a silently accepted field is a world its author believes they authored.

```yaml
# world.yaml
world:
  id: social-cafe          # required. Must equal the pack directory's name.
  name: Social Café        # required. For a person; no system reads it.

systems:                   # the capabilities this world enables, in installation order
  - presence
  - movement
  - conversation

places:                    # each key names places/<key>.yaml
  - cafe
  - street

population:                # each key names people/<key>.yaml
  - alice
  - bob
  - visitor

seats:                     # the Persons a client may connect as, each one of `population`
  - visitor
```

```yaml
# people/<key>.yaml — the file's name is the key; the file does not repeat it
tags: [barista, staff]     # optional. An open vocabulary; reaches clients in an Observation.
note: Runs the counter.    # optional. Authoring provenance, never gameplay state.
location:                  # optional. Requires the `presence` system.
  place: cafe              # required: one of `places`
  position:                # optional: integer millimetres from the place's origin
    x: 1200
    y: 2400
    z: 0
  facing: 180000           # optional: integer millidegrees
```

```yaml
# places/<key>.yaml
tags: [cafe, public]       # optional
note: A small café.        # optional
passages:                  # optional. Requires the `movement` system.
  - to: street             # required: another of `places`
    here:                  # optional: the doorway in this place, integer millimetres
      x: 4600
      y: 2000
    there:                 # optional: the same doorway in `to`, integer millimetres
      x: 0
      y: 2000
```

A passage is a doorway joining two places, and it holds both ways: it is stated **once**, in either
of the two places' files, and the `movement` system records it on both. Stating the same pair twice
— in both files, or twice in one — is refused, because two statements could disagree about where the
doorway is. A passage to an undeclared place, or from a place to itself, is refused. Either position
may be omitted, for the reason `location.position` may: a world that models no positions says only
that the café opens onto the street. How far from a doorway a person may pass through it is the
`movement` system's rule, not the pack's (`DECISIONS.md` `ARC-26`).

Five rules govern this subset, and each one is a decision rather than an implementation detail:

1. **A key is stated once.** `population` and `places` name the keys; a content file never repeats
   its own key. Two copies of one fact in a pack are two chances for them to disagree.
2. **Order in a list is the pack's statement where it is observable, and nowhere else.** `systems`
   order is installation order and therefore reduction order, which reaches the event log. `places`
   and `population` order is *not* observable: entity identities are allocated in key order, so
   reordering either list changes nothing.
3. **`seats` is the world's, not the client's.** A client names a seat and the server resolves which
   entity it is, which is why no protocol frame ever names an entity. A seat must be one of
   `population`.
4. **Content that needs a capability names it, and the pack must enable it.** `location` is state the
   `presence` system owns; a pack that places its people without enabling `presence` is refused,
   rather than starting a world in which everybody is quietly nowhere. `passages` are state the
   `movement` system owns, and a pack that joins its places without enabling `movement` is refused
   the same way.
5. **There are no floats.** Positions are integer millimetres and orientations integer millidegrees,
   because these values reach the event log and floating-point arithmetic is not reproducible across
   platforms (`MVP.md` §9 `AC-12`).

Initial state is **not** written into the world by the loader. Each authored `passage` and each
authored `location` becomes a recorded event caused by `Causation::WorldGenesis` — passages first,
because they are facts about places that exist before anybody is in them — which the owning system
reduces. So a loaded world's state has a causal origin in its own log, and a replay rebuilds it
(`DECISIONS.md` `ARC-15`).

---

# 5. Controller Pack (Cognition Pack)

Declares who decides what a Person attempts.

```text
Human Controller
Rule Controller
Behavior Tree Controller
LM Controller
RL Controller
Script Controller
```

A Controller Pack may define:

```text
agent policy
memory policy
planning
prompting
model routing
cognition budgets
```

Mixed control within one world is normal:

```text
Alice   → human
Bob     → local Qwen
Charlie → cloud model
David   → deterministic NPC
Emma    → RL policy
```

Hard constraints:

1. A Controller Pack **cannot create new interactions**. With `CombatSystem` disabled, "I
   shoot Bob" returns `ActionUnavailable` (INV-10).
2. It consumes `Observation`s only; it never reads global world state (INV-13).
3. It emits `ActionIntent`s only; it never mutates state (INV-6).
4. Rebinding a Person's controller preserves that Person's biography, relationships,
   inventory, and employment (INV-1).
5. Provider-specific details stay behind the model-backend interface; cognition contracts stay
   provider-neutral.

---

# 6. Presentation Pack

Declares how a world looks and sounds.

```text
sprites
models
animations
architecture style
sounds
UI
lighting
materials
```

Examples for one semantic world: `Lakewood Pixel`, `Lakewood Godot 3D`, `Lakewood Anime`,
`Lakewood Realistic UE`.

Hard constraint: swapping a Presentation Pack changes nothing about the simulation. The same
`Person: Alice / Place: Cafe / Action: make_coffee` becomes a sprite animation, a skeletal
animation with a coffee-machine interaction, or the line `08:32 Alice starts making coffee.`
in a text client.

A Presentation Pack may describe how an interaction *looks*. It may never define whether the
interaction is *valid*: `Sit(actor, bench)` belongs to a System, and the pack only decides
whether that is a seat anchor with a sit animation or a sprite swapping to a seated frame
([`ART_DIRECTION.md`](ART_DIRECTION.md) §19).

## 6.1 Structure: the Presentation Style Pack

A Presentation Pack's internal structure is specified by
[`ART_DIRECTION.md`](ART_DIRECTION.md) §12, where it is called a **Presentation Style Pack**.
That is the same pack type given a concrete layout, not a sixth kind of module
([`DECISIONS.md`](DECISIONS.md) `ARC-1`):

```text
presentation/<style-id>/
  manifest.yaml            compact machine-readable summary
  references/              the images that actually define the style
  ART_DIRECTION.md         human-readable direction
  assets/asset_bindings.yaml
  materials/material_rules.yaml
  characters/character_style.yaml
  lighting/lighting_profile.yaml
  renderer/godot.yaml, renderer/unreal.yaml
  generation/optional_prompt_guidelines.md
```

Not every pack needs every directory. A style that covers more than one dimension is more than
one pack, grouped in a family directory, because one manifest cannot describe two dimensions
whose character proportions differ (`DECISIONS.md` `ARC-3`). The shipped default:

```text
presentation/mineworld-default/
├── README.md     what the family is, and what the packs must share
├── 2D/           manifest.yaml · ART_DIRECTION.md · references/
└── 3D/           manifest.yaml · ART_DIRECTION.md · references/
``` The reference images are the source of truth; the manifest
summarises them, and a generation prompt is an implementation aid rather than the contract
(§16).

## 6.2 Authoring a style

Creating a style does not mean writing that manifest by hand
([`ART_DIRECTION.md`](ART_DIRECTION.md) §23):

```text
New Presentation Style → upload 4–10 references → write two or three sentences
→ choose 2D / 3D → the system generates an initial manifest → the creator corrects it
```

This is why the manifest schema must stay small: every field has to be inferable from images,
chosen by the creator, or fixed by project policy. A field that is none of those does not belong
in it.

---

# 6.3 Asset Pack

Declares the material a presentation is made of: models, images, textures, animation, audio.

```text
mineworld-assets-modern-town/
  manifest.yaml
  previews/
  models/      cafe.glb · house_01.glb · grocery.glb
  materials/
  textures/
  LICENSES/
```

Separate from Presentation Pack because the same bench, tree and café model are reused across
styles and across worlds; binding them to one style would force a copy per style. A Presentation
Pack declares which asset families it is compatible with:

```yaml
compatible_asset_tags: [semi-realistic-modern, realistic-scale, medium-detail]
```

Canonical model format is glTF 2.0, `.glb` preferred; no engine-native scene format is ever a
published asset. Each asset carries an `asset.yaml` whose `semantic_bindings` attach it to world
meaning — entrances, interaction anchors, spaces — without the world knowing what a mesh is. Full
contract in [`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md) §3.

Hard constraint, the same one every presentation-side pack carries: an interaction anchor says
where a `Sit` is *rendered*, never whether sitting is *allowed*.

# 7. Composition

```text
Modern Life Entity Pack
+ Conversation System
+ Relationship System
+ Employment System
+ Local LM Controller Pack
+ Pixel Presentation Pack
= a game
```

Replace one line:

```text
+ Realistic 3D Presentation Pack
= the same world, rendered differently
```

Two semantically identical worlds, different presentation:

```text
Lakewood World Pack + Modern Life Systems + Local-LLM Cognition + Pixel Presentation
Lakewood World Pack + Modern Life Systems + Local-LLM Cognition + Photorealistic UE Presentation
```

---

# 8. Installation and composition workflow

Intended creator experience:

```bash
mineworld install modern-life
mineworld install employment
mineworld install restaurant
mineworld install relationships
mineworld install godot-realistic

mineworld create my-town
mineworld add-system university
mineworld run worlds/my-town --headless
```

After `add-system university`, the world gains `University`, `Student`, `Professor`,
`Course`, `Enroll`, `AttendClass`, `TakeExam`, `Graduate` without the world author
implementing a university simulation. That leverage is the reason the pack system exists.

The command surface above is the intended shape, not a frozen CLI contract; the CLI is
specified in the PR that implements it. `install` and `add-system` are not implemented in
MVP-0; §8.1 is what exists.

## 8.1 The `mineworld` command as implemented (MVP-0, S7)

One binary, `mineworld`, built from `tools/cli`. Its arguments are parsed by `clap`
([`DECISIONS.md`](DECISIONS.md) `DEP-11`). Every refusal is a message on stderr naming what was
wrong and a non-zero exit status, never a panic.

```text
mineworld server <world> [--listen ADDRESS] [--agent SEAT]... [--save DIR]
mineworld validate <world>
mineworld replay <world> --save DIR
mineworld run <world> --headless --seed N --days N [--save DIR]
mineworld inspect <save-directory> [--last N]
mineworld create <directory>
```

| Command | What it does |
| --- | --- |
| `server` | Hosts a World Pack for clients (`NETWORKING.md`). `--agent SEAT` drives that seat with the reactive rule controller in-process; `--save DIR` keeps the world in `DIR/world.sqlite`, created the first time and resumed afterwards. |
| `validate` | Reads and loads a World Pack and reports the world it describes: systems, places, people, seats, the identity each key received, the number of genesis facts. |
| `replay` | Re-executes a save's whole journal from genesis and checks every fact and snapshot byte for byte (`ARC-25`). |
| `run` | Runs a World Pack headless: no renderer, no network, no model. Every seat the pack offers is driven by a seeded paced rule controller (`ARC-27`). Described below. |
| `inspect` | Reports what a save holds, without resuming or writing it. Described below. |
| `create` | Writes a new, minimal World Pack into a directory that does not exist yet. Described below. |

**`run`.** `--headless` is required: it states the only mode `run` has in MVP-0, and leaves a
non-headless `run` possible later without changing what an existing invocation means. `--seed N`
is an unsigned 64-bit integer; it is the controllers' seed and nothing else in a world consumes
randomness. `--days N` (N ≥ 1) is the **age** the world is run to: until
`genesis instant + N × 86 400` simulated seconds. A day is the command's unit, not the kernel's
(`INV-12`). Without `--save` the world lives in memory. With `--save DIR`, a save that does not
exist is created and one that exists is resumed and run on to the same age, so re-running a killed
command completes the same world rather than appending a second run (`ARC-27`).

`run` prints, in this order: one line naming the world and its seats; one `day` line per completed
simulated day (`day D  revision R  facts F`; revision `-` without `--save`; F counts the facts
this invocation recorded); a summary of consults, requests by action type and outcome, facts by
event type, accepted `move` and `talk` per seat in each 30-day bucket, faults, and the history
fingerprint, which always covers the world's whole history; and last, on its own line beginning `wall`, the elapsed real time. Everything
before the `wall` line is a function of the pack, the seed, the age and the code. The fingerprint
(`FNV-1a 64` over every fact's stored encoding in `EventId` order, printed with the number of facts
it covers) is for a person comparing two runs by eye; it is not a proof of equality.

**`inspect`.** Reads `DIR/world.sqlite` and prints the manifest (pack, instance, composition), the
head revision and its instant, the journal counted by input kind and outcome, the facts counted by
event type and by cause kind, the result of the causation check, and the last N facts
(default 20) with their causes. The causation check (`MVP.md` §9 `AC-9`) requires every fact
caused by an action to name an `ActionId` some journaled request carried, every fact caused by an
event to name a fact with a smaller `EventId`, and world genesis to cause facts in revision 1
only; facts caused by a process or a system tick are counted, since the log alone cannot resolve
them. A failed check names the fact and exits non-zero.

**`create`.** The directory's final component becomes the world's id and must be a valid key (the
rule `EntityKey` enforces). The pack written has one place, two people who are both seats, and the
systems `presence`, `movement` and `conversation`, and it is read and loaded before `create`
reports success. An existing directory is refused and left untouched.

---

# 9. Dependencies, versions, and migration

Every pack declares:

```text
id
version
dependencies          other packs and version ranges
requires              capabilities, never secrets
provides              components, actions, events, entity types, assets
configuration schema
migration schema      how existing worlds move to a new version
```

Rules:

1. Version core contracts where long-term compatibility matters; a change to a core contract
   is a deliberate review, not an incidental edit
   ([`ENGINEERING_STANDARDS.md`](ENGINEERING_STANDARDS.md) §13).
2. A pack that changes an owned component's schema ships a migration for existing saves, since
   world history is durable ([`ARCHITECTURE.md`](ARCHITECTURE.md) §7).
3. Before MineWorld publishes stable public contracts, a wrong interface is fixed cleanly
   rather than wrapped in adapters (§29 of the standards).

---

# 10. Capabilities and sandboxing

```text
v0     trusted Rust systems, trusted Python extensions
later  WASM component plugins
```

A pack requests capabilities explicitly:

```text
world.read.person
world.read.place
world.emit.event
```

Network and filesystem access are never implicit. A pack that needs an outbound endpoint
declares it and the operator grants it.

---

# 11. What the community publishes

Not mods against a fork — composable packages:

```text
1990s American Small Town      Modern Tokyo
University Campus              Space Colony
Medieval Village               Research Station
Cruise Ship
```

One framework runs them all. If a published pack requires patching the kernel to work, the
pack model has failed and that is an architecture bug in MineWorld, not in the pack.
