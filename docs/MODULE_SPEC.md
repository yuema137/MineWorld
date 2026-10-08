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

## 3.1 Installing a System Pack in MVP-0

MVP-0's System Packs are trusted, statically linked Rust crates under `systems/`
([`ARCHITECTURE.md`](ARCHITECTURE.md) §12). **Installing** a pack puts it into the build;
**enabling** it is a world's choice, made by naming it in a World Pack's `systems:` list (§4.1). A
world that does not enable an installed pack is not affected by it
([`DECISIONS.md`](DECISIONS.md) `ARC-33`).

**What a pack declares about itself.** Beside its `System` implementation, a pack implements
`mineworld_sdk::SystemPack` (crate `mineworld-sdk`, `sdk/rust/`). The trait requires `System` and
`Default` and carries what the build needs to know about the pack beyond `System`:

```text
BIOGRAPHICAL     the pack's event types that belong in a person's biography (ARC-29); default none
SECTION          the authored section the pack owns (ARC-31), if any; default none
decode_section   how that section is decoded from a content file; default: refused, naming the pack
```

A pack that owns a section implements `mineworld_authoring::AuthoredSection` and writes
`mineworld_sdk::owns_section!();` inside its `impl SystemPack`. The macro defines `SECTION` and
`decode_section` together from the `AuthoredSection` impl, so they cannot disagree. A pack that owns
no section writes neither.

```rust
impl SystemPack for ConversationSystem {}

impl SystemPack for NamingSystem {
    const BIOGRAPHICAL: &'static [EventTypeId] = BIOGRAPHICAL;
    mineworld_sdk::owns_section!();
}
```

A pack must also implement `mineworld_presence::PerceptionProvider`, because every installed pack
answers for its own actions in an observation (`systems/presence/src/interaction.rs`). The provider's
default methods offer nothing and disclose nothing, so a pack with no actions writes an empty `impl`.

**The installed set.** `systems/installed/` (crate `mineworld-installed-systems`) is the build's list
of System Packs, and the only one. Installing a pack is exactly:

```text
systems/<name>/                        the pack                                   (a new directory)
systems/installed/Cargo.toml           mineworld-<name> = { path = "../<name>" }  (one line)
systems/installed/src/lib.rs           <Variant> => mineworld_<name>::<System>,   (one line)
Cargo.lock                             regenerated by Cargo                       (generated)
```

and then a rebuild. No other file is edited. In particular:
- the root `Cargo.toml` lists `"systems/*"` as members, so a new directory is a member without an
  edit;
- the World Pack loader (`worldpack/`) reads the installed set through the generated `Capability`
  and names no pack except `presence` and `movement`, whose state the format's `location` and
  `passages` fields become (§4.1).

**A pack that resolves arrivals** ([`DECISIONS.md`](DECISIONS.md) `ARC-39`). Presence asks every
registered `mineworld_presence::ArrivalResolver` what an arrival actually achieves before it records
it. The installed set lists the build's resolvers on an optional line after `perception:`, each type
followed by a comma:

```text
mineworld_sdk::installed! {
    perception: mineworld_presence::PerceptionProvider;
    resolution: mineworld_presence::ArrivalResolver => [ mineworld_<name>::<System>, ];
    <Variant> => mineworld_<name>::<System>,
    …
}
```

The line expands to `Capability::resolvers()`, and `worldpack::compose` registers that list before it
installs anything. A listed type that does not implement the trait does not compile, and the installed
set's resolution test refuses a listed resolver that is not also an installed pack. Installing a
resolver pack is therefore three lines in `systems/installed/`, not two: its Cargo dependency, its
`installed!` line, and its entry on the `resolution:` line. With the list empty — `[]` — no world
records anything differently. A resolver pack has two duties beyond `System`:
- **inert where its state is absent:** its resolver returns the resolution it was handed unchanged
  when the world holds none of the pack's own state about the people and the place involved, so a world
  that does not enable the pack is not affected by it;
- **refuse to join an unregistered world:** its `System::install` calls
  `mineworld_presence::require_registered(&Self::ID)` first, which panics, naming the pack, when the
  host never registered the build's resolvers or registered a list without it.

**Dependencies between packs.** A pack that depends on another pack — to state its facts under
`ARC-26`, or to decode them as a subscriber under `ARC-28` — names it by path:
`mineworld-<other> = { path = "../<other>" }`. The root manifest's `[workspace.dependencies]` is not
edited for a new pack. A dependency outside this repository is still a reviewable decision recorded
in [`DECISIONS.md`](DECISIONS.md), as it is for every crate.

**What is refused, and how.**

| Mistake | Refused by |
| --- | --- |
| A listed pack that is not a `SystemPack`, not `Default`, or not a `PerceptionProvider` | the compiler, at the `installed!` invocation |
| A pack in `systems/installed/Cargo.toml` that the `installed!` list omits, or the reverse | the installed set's consistency test, naming the pack |
| Two listed packs with one system id | the installed set's consistency test, naming the id |
| Two packs claiming one section name, or a section shadowing a format field | `worldpack`'s section-namespace test |
| A world enabling a pack the build does not provide | the loader: `PackError::UnknownSystem`, listing the packs the build provides |
| A world enabling a pack without enabling its dependencies | the kernel's registry at installation, naming the missing dependency |

**What installing does not mean in MVP-0.** It always needs a rebuild. Installing a pack into a
built binary or a running server, or installing a pack that is not compiled from this repository's
build, is the WASM component model of [`DECISIONS.md`](DECISIONS.md) `ARC-8` (Tier 1), outside
MVP-0. `mineworld install` and `mineworld add-system` (§8) are not implemented; the two lines are
written by hand.

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

items:                     # optional. Each key names items/<key>.yaml (an item kind, ARC-36)
  - lantern

organizations:             # optional. Each key names organizations/<key>.yaml
  - chess-club

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
name: Alice Moreau         # a section owned by the `naming` system (below)
routine:                   # a section owned by the `schedule` system (below)
  - { from: "05:30", place: cafe, label: work }
  - { from: "18:00", place: apartments, label: home }
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
# body:                    # a section owned by the `bodies` system (below)
#   floor: { min: { x: 0, y: 0 }, max: { x: 8320, y: 10320 } }
#   solids:
#     - { min: { x: 3860, y: 6570 }, max: { x: 8320, y: 7170 }, height: 1100 }
```

```yaml
# items/<key>.yaml and organizations/<key>.yaml — the same two fields, and sections
tags: [light]              # optional
note: Hangs by the door.   # optional
```

An item file declares an **item kind**, not one object: in MVP-0 what anybody holds of an item is a
count of its kind, and a world has no item instances (`DECISIONS.md` `ARC-36`). An item or
organization file has no `location` and no `passages`; either key there is refused as unknown, at its
line.

A passage is a doorway joining two places, and it holds both ways: it is stated **once**, in either
of the two places' files, and the `movement` system records it on both. Stating the same pair twice
— in both files, or twice in one — is refused, because two statements could disagree about where the
doorway is. A passage to an undeclared place, or from a place to itself, is refused. Either position
may be omitted, for the reason `location.position` may: a world that models no positions says only
that the café opens onto the street. How far from a doorway a person may pass through it is the
`movement` system's rule, not the pack's (`DECISIONS.md` `ARC-26`).

**Sections: content a System Pack owns** (`DECISIONS.md` `ARC-31`). Besides the fields above, a person,
place, item or organization file may carry **sections** (`ARC-36` extends `ARC-31` to the last two). A section is a top-level key of a content file that a System
Pack declares as its own, and nothing else is a section. The pack that declares it:

- names it (one word, unique among every pack this build provides and distinct from the fields
  above);
- says which content files may carry it (people, places, items, organizations, or any of them);
- validates it with its own type — the loader decodes the section's YAML straight into that type,
  so an invalid section is refused with its line and column and the pack's own message;
- states, at genesis, the facts it becomes, in its own vocabulary.

The loader never learns what a section means. It checks only what is common to every section: that
its owner is enabled, that the file kind may carry it, and that every other entity it names by key
is declared and of the kind the owner requires. A seeded fact must be in the owner's own vocabulary,
or the pack is refused. MVP-0 has seven sections:

```text
name      naming     people   a display name: 1–64 bytes, no control characters, no surrounding
                              whitespace. Public: disclosed to whoever perceives the person
routine   schedule   people   2–24 segments { from: "HH:MM", place: <place key>, label: <slug> };
                              `from` strictly increasing; neighbouring segments differ, the last
                              against the first included, because the day wraps. An agenda the
                              person may follow, never a move (`ARC-32`)
item      item       items    { category: <slug> }: 1–32 bytes of a–z, 0–9 and '-', neither first nor
                              last '-'. Declares the file's Item a kind the market packs trade; an
                              item file without it is an inert entity. Disclosed to nobody (`ARC-37`)
holdings  inventory  people,  { <item key>: <count ≥ 1> }: what the person or organization holds at
                     organi-  genesis, each key one of `items`. A person's counts together are at most
                     zations  six (inventory's capacity); an organization's are unbounded. Disclosed to
                              the holder only (`ARC-37`)
economy   economy    people,  { wallet: <minor units ≥ 0> } on a person or organization; an organization
                     organi-  may add shop: { at: <place key>, prices: { <item key>: <price ≥ 1> } },
                     zations  which opens a shop in that place, operated by the organization. A pack
                              owns one section, so a shop is authored on its operator. Integer minor
                              units only. A wallet is disclosed to its holder; a shop's listing
                              (operator, prices, how many the operator holds) to whoever perceives its
                              place (`ARC-38`)
job       employment people   { employer: <organization key>, workplace: <place key>, from: "HH:MM",
                              until: "HH:MM", wage: <minor units per hour>, produces: { <item key>:
                              <count per full shift ≥ 1> } }; from < until (a shift lies within one
                              day); produces is optional. Work is attendance at the workplace during
                              the shift. Disclosed to the employee only (`ARC-38`)
body      bodies     places   { floor: { min: {x, y}, max: {x, y} }, solids: [ { min: {x, y}, max: {x,
                              y}, height } ] }: the place's walkable rectangle — its edge is the
                              place's walls — and up to 64 solid boxes standing on it, integer
                              millimetres in the place's frame. Each floor side ≥ 620 mm; each solid
                              non-empty, height 1–10 000; every coordinate within ±100 000. `solids` is
                              optional. Unknown keys refused. A place without it has no geometry, and
                              nobody in it is resolved. At genesis the pack refuses two people closer
                              than 595 mm, a centre outside the floor shrunk by 300 mm or within 300 mm
                              of a solid, and a floor that cannot hold the world's population. Disclosed
                              to whoever perceives the place (`ARC-39` note, `DEP-13`)
```

`location` and `passages` are fields of the format rather than sections. They predate the seam, and
moving them onto it would change `presence` and `movement` and the refusals authors already see;
that move is recorded as a later candidate in `ARC-31`, not made silently.

Six rules govern this subset, and each one is a decision rather than an implementation detail:

1. **A key is stated once.** `places`, `population`, `items` and `organizations` name the keys; a
   content file never repeats its own key. A key is stated once across all four lists, too: keys are
   one namespace, because a section names another entity by its key, so one key must never mean two
   entities. Two copies of one fact in a pack are two chances for them to disagree.
2. **Order in a list is the pack's statement where it is observable, and nowhere else.** `systems`
   order is installation order and therefore reduction order, which reaches the event log. The order
   of `places`, `population`, `items` and `organizations` is *not* observable: entity identities are
   allocated places, then people, then items, then organizations, each in key order, so reordering a
   list changes nothing. Items and organizations come last so that declaring them moves no identity a
   world already had.
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
6. **A section belongs to the pack that declares it.** A key that is neither a field above nor a
   section some pack of this build declares is refused as unknown. A section whose pack this build
   provides but the world does not enable is refused naming that pack, as rule 4 refuses an
   unowned `location`: a silently ignored section is a world its author believes they authored.

Initial state is **not** written into the world by the loader. Each authored `passage`, each
authored `location` and each section becomes a recorded event caused by `Causation::WorldGenesis`
— passages first, because they are facts about places that exist before anybody is in them, then
locations, then sections: items', organizations', places', people's, each in key order, and within one
file in the order the world's `systems` lists their owners — which the owning system reduces. What a
person's or a place's section may refer to is seeded before it, and a world that declares no items or
organizations seeds exactly what it seeded before they existed (`ARC-36`). So a loaded world's state has a causal origin in its own log, and a replay rebuilds it
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
   shoot Bob" returns `ActionUnavailable` (INV-10). A Controller Pack may attempt a complete
   affordance it was offered — submit, unchanged, the request an installed System Pack stated it
   would accept ([`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §15.2, [`DECISIONS.md`](DECISIONS.md)
   `ARC-34`); that is attempting an interaction the world offered, not creating one, and the server
   still validates it.
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
MVP-0; §8.1 is what exists. A System Pack is installed into an MVP-0 build by hand, as §3.1
describes.

## 8.1 The `mineworld` command as implemented (MVP-0, S7)

One binary, `mineworld`, built from `tools/cli`. Its arguments are parsed by `clap`
([`DECISIONS.md`](DECISIONS.md) `DEP-11`). Every refusal is a message on stderr naming what was
wrong and a non-zero exit status, never a panic.

```text
mineworld server <world> [--listen ADDRESS] [--invite TOKEN] [--agent SEAT]... [--save DIR]
mineworld validate <world>
mineworld replay <world> --save DIR
mineworld run <world> --headless --seed N --days N [--save DIR]
mineworld inspect <save-directory> [--last N]
mineworld biography <world> --save DIR --person KEY [--json]
mineworld create <directory>
```

| Command | What it does |
| --- | --- |
| `server` | Hosts a World Pack for clients (`NETWORKING.md`). `--agent SEAT` drives that seat with the reactive rule controller in-process; `--save DIR` keeps the world in `DIR/world.sqlite`, created the first time and resumed afterwards. `--invite TOKEN` (or the environment variable `MINEWORLD_INVITE`; the flag wins) is the invite every client must present in its `join`; with neither, the server generates one and prints it once as `[mineworld] invite <token> — join with: <address> seat=<seat> invite=<token>`. There is no mode without an invite, loopback included (`server/PROTOCOL.md` §4.1, `DECISIONS.md` `DEP-14`). |
| `validate` | Reads and loads a World Pack and reports the world it describes: systems, places, people, seats, the identity each key received, the number of genesis facts. |
| `replay` | Re-executes a save's whole journal from genesis and checks every fact and snapshot byte for byte (`ARC-25`). |
| `run` | Runs a World Pack headless: no renderer, no network, no model. Every seat the pack offers is driven by a seeded paced rule controller (`ARC-27`). Described below. |
| `inspect` | Reports what a save holds, without resuming or writing it. Described below. |
| `biography` | Prints a Person's objective biography, derived from a save's fact log without resuming or writing it ([`DECISIONS.md`](DECISIONS.md) `ARC-29`). Described below. |
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

**`biography`.** Reads `DIR/world.sqlite`'s manifest and fact table, and the World Pack `<world>`
for the authoring keys. It refuses, by name and with a non-zero exit, any of these:
- a missing save;
- a save whose manifest names another pack;
- a KEY the pack does not declare as a Person;
- a composition naming a system this build does not provide.

The biographical event types are the union of what each system in the save's composition declares
biographical (`ARC-29`). A fact is an entry for the Person when the Person is among its subjects or
participants and its type is in that set. Output is one line per entry, oldest first: the instant
(`t…`, day and time of day), the event id, the event type, the place's key, and the counterparts'
keys. With `--json`, each entry is one JSON object per line carrying the same values. Nothing is
stored; the biography is regenerated from the log on every invocation.

When the save's composition includes `naming`, the Person and every counterpart and place it names
are also shown by display name, as `key "Name"`. The names come from the save's own `named` facts,
read through the `naming` pack's published projection, so the command still names no event type of
its own (`ARC-31`). `--json` adds `name` (the Person's) and `counterpart_names` (aligned with
`counterparts`, `null` where unnamed) and changes no existing field.

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
