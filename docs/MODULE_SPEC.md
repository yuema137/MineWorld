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
PACKAGE          the pack's package identity (ARC-53): its Cargo name, version, licence, authors and
                 repository, recorded at compile time by mineworld_sdk::package!(); required
BIOGRAPHICAL     the pack's event types that belong in a person's biography (ARC-29); default none
SECTION          the authored section the pack owns (ARC-31), if any; default none
decode_section   how that section is decoded from a content file; default: refused, naming the pack
CONFIGURATION, CONFIGURATION_FACTS, decode_configuration
                 whether the pack takes a world-level configuration (ARC-61), defined together by
                 configures!(); default: not configurable (below)
```

`PACKAGE` has no default: a pack that does not state it does not compile, so no pack in a build is
anonymous. It is always the same line, the first of the `impl`, and it is read by `mineworld packs`
only — never by a system, so a pack's version never reaches a fact.

A pack that owns a section implements `mineworld_authoring::AuthoredSection` and writes
`mineworld_sdk::owns_section!();` inside its `impl SystemPack`. The macro defines `SECTION` and
`decode_section` together from the `AuthoredSection` impl, so they cannot disagree. A pack that owns
no section writes neither.

```rust
impl SystemPack for ConversationSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
}

impl SystemPack for NamingSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
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
it. The installed set lists the build's resolvers on presence's **extension line** (below), each type
followed by a comma:

```text
mineworld_sdk::installed! {
    perception: mineworld_presence::PerceptionProvider;
    extension mineworld_presence::ArrivalResolver => mineworld_presence::register_resolvers: [ mineworld_<name>::<System>, ];
    <Variant> => mineworld_<name>::<System>,
    …
}
```

`worldpack::compose` registers every extension line before it installs anything. A listed type that
does not implement the trait does not compile, and the installed set's resolution test refuses a listed
type that is not also an installed pack. Installing a resolver pack is therefore three lines in
`systems/installed/`, not two: its Cargo dependency, its `installed!` line, and its entry on presence's
extension line. With the list empty — `[]` — no world records anything differently. A resolver pack has
two duties beyond `System`:
- **inert where its state is absent:** its resolver returns the resolution it was handed unchanged
  when the world holds none of the pack's own state about the people and the place involved, so a world
  that does not enable the pack is not affected by it;
- **refuse to join an unregistered world:** its `System::install` calls
  `mineworld_presence::require_registered(&Self::ID)` first, which panics, naming the pack, when the
  host never registered the build's resolvers or registered a list without it.

**Extension catalogs** ([`DECISIONS.md`](DECISIONS.md) `ARC-62`). Arrival resolution is the first
instance of a general shape: a pack owns a trait, other packs implement it, and the build lists the
implementations. Each such catalog is one line of `installed!`, after `perception:` and before the pack
lines:

```text
extension <trait path> => <register fn path>: [ <type>, … ];
```

The trait and the register function are the owning pack's; each listed type implements the trait and
`Default`. The line expands into `Capability::register_extensions()`, which calls each line's register
function once with one value of each listed type, in the listed order, and `worldpack::compose` calls
it before it installs anything; and into `Capability::extension_types()`, which the installed set's
guard reads to refuse a listed type that is not an installed pack, or one listed twice on a line. The
SDK and the loader name no trait and no pack: a new catalog is one line in `systems/installed/`. The
owning pack's register function keeps `ARC-39` item 5's rules: write-once and process-wide, a different
list refused naming both. An implementation is pure and inert where its pack's state is absent.

**A configurable pack** ([`DECISIONS.md`](DECISIONS.md) `ARC-61`). A pack that accepts a world-level
configuration (§4.1, `configure:`) implements `mineworld_authoring::PackConfiguration` — its
`Configuration` type, the event types its configuration may seed (`FACTS`), the entity keys and systems
it requires, and `seed` — and writes `mineworld_sdk::configures!();` inside its `impl SystemPack`, which
defines three more items of the trait together:

```text
CONFIGURATION          Some(the pack's own id) when it is configurable; default None
CONFIGURATION_FACTS    the event types its configuration may seed (the drift check's filter); default none
decode_configuration   decodes configure/<id>.yaml into its own type; default: refused, "the '<id>'
                       system takes no configuration"
```

A pack states its configuration facts `Visibility::SystemInternal` with no subjects: a world's
configuration is nobody's perception and nobody's biography.

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
├── configure/            each enabled System Pack's world-level configuration, typed by it (ARC-61)
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

requires:                  # every pack this world uses that is not versioned with the framework
  modern-life: "^1.2"      #   an Entity Pack: requiring it puts its item kinds in this world
  lakewood-realistic: "^1" #   a Presentation Pack: the world is authored for it

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

network_profile:
  default: private_server
```

Constraints:

1. A World Pack contains no secrets and no credentials. It declares required *capabilities*;
   the operator supplies endpoints and keys.
2. A World Pack contains no renderer-specific assets; it requires a Presentation Pack.

**Amended 2026-10-08 (S16 QSE-8, accepted by the operator; `DECISIONS.md` `ARC-54`).** One
`requires:` map — pack id to semver range — replaces the frozen model's separate `entity_packs:` list
and `presentation_profile:`: requiring an Entity Pack is using it, and requiring a Presentation Pack is
declaring it. `requires:` names packs and versions only; which systems a world enables stays
`systems:`, and how a pack is configured is a separate seam.
3. A World Pack never redefines simulation rules. If a world needs a new rule, that is a
   System Pack.

## 4.1 The fields MVP-0's loader reads

The model above is the frozen one, amended only by QSE-8's `requires:`. This subsection states the
**subset MVP-0 implements**, because the loader (`worldpack/`) exists and an author needs to know what
it accepts. Fields of §4's model that MVP-0 does not implement — `era`, `calendar`, `geography`,
`cognition_profile`, `network_profile` — and the two `requires:` replaced, `entity_packs` and
`presentation_profile`, are **refused by name**, not ignored: a silently accepted field is a world its
author believes they authored.

```yaml
# world.yaml
world:
  id: social-cafe          # required. Must equal the pack directory's name.
  name: Social Café        # required. For a person; no system reads it.
  version: 0.1.0           # optional to the loader; semver. Required by `packs validate` (ARC-53)
  license: MIT             # optional to the loader; an SPDX expression. Required by `packs validate`

mineworld: "^0.1"          # optional to the loader; the framework versions this world is authored
                           # for. When present, a framework outside it is refused by name

requires:                  # optional; every pack this world uses that is not bundled, with a range
  mineworld-default-3d: "^0.1"   # found in a directory named by --packs or MINEWORLD_PACKS (ARC-54)

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

configure:                 # optional. Each key is a system id and names configure/<key>.yaml (ARC-61)
  - <system id>
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

```yaml
# items/<key>.yaml carrying `body:` — one loose object, not a kind (ARC-36 note)
tags: [toy]
# body:                    # a section owned by the `bodies` system (below)
#   shape: { ball: 110 }   # or { box: { x: 200, y: 200, z: 200 } }: half-extents, millimetres
#   at: { place: hall, x: 3000, y: 2000 }
```

An item file declares an **item kind**, not one object, **unless it carries `body:`**: in MVP-0 what
anybody holds of an item is a count of its kind, and a world has no item instances (`DECISIONS.md`
`ARC-36`). An item file with a `body:` section declares exactly one physical object lying on a place's
floor; it is never a declared kind (it carries no `item:` section), so nobody ever holds it (`ARC-36`
note). An item or
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
body      bodies     places,  Two forms, one per kind of file; mixing them, or neither, is refused,
                     items    naming the keys, and a form on the other kind of file is refused
                              (`bodies-section-kind`). Unknown keys refused.
                              Place form — { floor: { min: {x, y}, max: {x, y} }, solids: [ { min:
                              {x, y}, max: {x, y}, height } ] }: the place's walkable rectangle — its
                              edge is the place's walls — and up to 64 solid boxes standing on it,
                              integer millimetres in the place's frame. Each floor side ≥ 620 mm; each
                              solid non-empty, height 1–10 000; every coordinate within ±100 000.
                              `solids` is optional. A place without it has no geometry, and nobody in
                              it is resolved. At genesis the pack refuses two people closer than
                              595 mm, a centre outside the floor shrunk by 300 mm or within 300 mm of a
                              solid, and a floor that cannot hold the world's population.
                              Object form — { shape: { box: { x, y, z } } | { ball: <radius> }, at:
                              { place: <place key>, x, y } }: one loose object lying on that place's
                              floor (`ARC-36` note). A box's half-extents are 50–400 mm in x and y and
                              50–500 in z; a ball's radius 50–400. At genesis the pack refuses an
                              object in a place without `body:` (`bodies-unshaped`), on an item that is
                              also a declared kind (`bodies-held-kind`), a 33rd object in one place
                              (`bodies-objects-max`), a footprint leaving the floor
                              (`bodies-object-outside`), meeting a solid (`bodies-object-in-solid`),
                              overlapping another object (`bodies-object-overlap`) or a person's disc
                              (`bodies-object-on-person`), and a place whose capacity no longer holds
                              the population with its objects (`bodies-capacity`).
                              A place's shape, and a listing of the objects lying in it (each one's
                              shape and position), are disclosed to whoever perceives the place
                              (`ARC-39` notes, `DEP-13`)
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

**Package fields** (`DECISIONS.md` `ARC-53`). `world.version`, `world.license` and `mineworld:` state
the World Pack's package identity. They are optional to the loader in MVP-0, and checked whenever they
are present: a version that is not semver, or a licence that is not an SPDX expression, is refused at
its line and column, and a `mineworld:` range the running framework (0.1.0) does not satisfy is refused
naming the range and the framework version. `mineworld packs validate` (§8.1) requires all three. They
are never world state: no genesis fact, no `Metadata` and no save carries them.

**Requirements** (`DECISIONS.md` `ARC-54`, `ARC-55`). `requires:` maps a pack id to a semver range, with
Cargo's meaning. It names every pack the world uses that is **not bundled** — a bundled pack is a code
pack compiled from the framework's own workspace and is versioned with it, so the world's `mineworld:`
range already covers it. Packs are searched in this build and in the **pack roots**: the directories
named by `--packs DIR` on the command, in order, then those in `MINEWORLD_PACKS`; nothing else is
searched, not even the world's own directory. A pack root's immediate subdirectories holding
`pack.yaml` or `world.yaml` are its packs. Resolution keeps one installed version per pack and chooses
nothing. It runs after `systems` is resolved and before content is read, and the first failure is
refused by name, in this order:

1. any pack found twice under one id, naming both places;
2. for each requirement, in id order:
   - absent — naming the id and every place searched, or that no pack directory was given;
   - of a type a world cannot require — a World Pack or a Controller Pack — or an Entity Pack, which is
     read from S16's PR E-d;
   - bundled — "versioned with the framework; remove it from requires";
   - at a version the range does not admit — naming the id, the range and the version found;
3. a **third-party** system the world enables in `systems` that `requires:` does not name — naming the
   system, its pack and the missing requirement;
4. a licence outside the **licence policy** — the world's own when stated, every required pack's, every
   enabled system's pack's — naming the pack, its expression, the identifiers that failed and the
   allowed ones.

The **licence policy**'s default allows `MIT`, `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`,
`Zlib`, `CC0-1.0` and `Unlicense`. An expression is allowed when it can be satisfied with those
identifiers alone, with no `WITH` addition and no `+`: `MIT OR GPL-3.0-only` is allowed, `MIT AND
GPL-3.0-only` is not. The policy is a typed value, so a world will be able to narrow or extend it
through the generic configuration seam (S17); until then every world uses the default.

The resolved **composition** — the framework, each requirement with the version and the place that
satisfied it, each enabled system's pack — is printed by `mineworld packs resolve` (§8.1). It is never
world state: no fact and no save records a version or a pack root, and a world resumed from a save is
resolved again against the roots given then.

**Configuration: a world-level file a System Pack owns** (`DECISIONS.md` `ARC-61`). `configure:` lists
the enabled packs this world configures. Each key is a system id and names `configure/<key>.yaml`, which
the owning pack decodes straight into its own type — so a refusal carries its line and column and the
pack's own message — and seeds as its own genesis facts. It is read after requirements resolve. The
loader never learns what a configuration means. It refuses, by name and naming the file:

- a key that is no system of this build, or a system the world does not enable;
- a system that takes no configuration;
- a **reserved** key: `classes` (the Interaction List's entity classes) and `packages` (the licence-policy
  override), both reserved for a later build;
- a key listed twice; a listed file that is missing; a `.yaml` file in `configure/` that is not listed;
- a system the configuration requires that the world does not enable;
- an entity the configuration names that is not declared, or not of the type its owner needs;
- a configuration its owner refuses at genesis, or one that seeds another pack's fact or an event type
  its owner did not declare.

A world without `configure:` loads exactly as before the key existed. A save remembers its
configuration: resuming or replaying it against a World Pack whose configuration differs — changed,
added or removed — is refused, naming the system (`ConfigurationDrift`). Other content is not compared
at resume.

Initial state is **not** written into the world by the loader. Each authored `passage`, each
authored `location`, each configuration and each section becomes a recorded event caused by
`Causation::WorldGenesis` — passages first, because they are facts about places that exist before
anybody is in them, then locations, then configuration in `configure:` order (so a section's reduction
may check a value against it), then sections: items', organizations', places', people's, each in key order, and within one
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
```

Each dimension pack also carries a **`pack.yaml`**, its package identity (`DECISIONS.md` `ARC-53`,
[`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md) §5.0): `id` (`mineworld-default-2d`, `mineworld-default-3d`),
`type: presentation-pack`, `version`, `mineworld`, `license`, `authors`. The style manifest keeps its
own `id`, which names the style, not the pack. `mineworld packs validate` checks both files: the
package fields, and that the style manifest has an `id` and a `dimension` list.

The reference images are the source of truth; the manifest
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
                 [--packs DIR]...
mineworld validate <world> [--packs DIR]...
mineworld replay <world> --save DIR [--packs DIR]...
mineworld run <world> --headless --seed N --days N [--save DIR] [--packs DIR]...
mineworld inspect <save-directory> [--last N]
mineworld biography <world> --save DIR --person KEY [--json] [--packs DIR]...
mineworld create <directory>
mineworld packs list [--packs DIR]...
mineworld packs show <id> [--packs DIR]...
mineworld packs validate <directory> [--packs DIR]...
mineworld packs resolve <world> [--packs DIR]...
```

**Pack roots** (`DECISIONS.md` `ARC-54`). Every command that reads a world takes `--packs DIR`,
repeatable: the directories a world's `requires:` is resolved in (§4.1), in the order given, followed
by the entries of the `MINEWORLD_PACKS` environment variable (a path list, `:`-separated on Unix; empty
entries skipped). A root that does not exist or is not a directory is refused, naming it and whether it
came from `--packs` or `MINEWORLD_PACKS`. A world without `requires:` needs none. `validate` prints one
`requires` line per requirement, after `seats`, only when the world states `requires:`.

| Command | What it does |
| --- | --- |
| `server` | Hosts a World Pack for clients (`NETWORKING.md`). `--agent SEAT` drives that seat with the reactive rule controller in-process; `--save DIR` keeps the world in `DIR/world.sqlite`, created the first time and resumed afterwards. `--invite TOKEN` (or the environment variable `MINEWORLD_INVITE`; the flag wins) is the invite every client must present in its `join`; with neither, the server generates one and prints it once as `[mineworld] invite <token> — join with: <address> seat=<seat> invite=<token>`. There is no mode without an invite, loopback included (`server/PROTOCOL.md` §4.1, `DECISIONS.md` `DEP-14`). |
| `validate` | Reads and loads a World Pack and reports the world it describes: systems, places, people, seats, the identity each key received, the number of genesis facts. |
| `replay` | Re-executes a save's whole journal from genesis and checks every fact and snapshot byte for byte (`ARC-25`). |
| `run` | Runs a World Pack headless: no renderer, no network, no model. Every seat the pack offers is driven by a seeded paced rule controller (`ARC-27`). Described below. |
| `inspect` | Reports what a save holds, without resuming or writing it. Described below. |
| `biography` | Prints a Person's objective biography, derived from a save's fact log without resuming or writing it ([`DECISIONS.md`](DECISIONS.md) `ARC-29`). Described below. |
| `create` | Writes a new, minimal World Pack into a directory that does not exist yet. Described below. |
| `packs` | Prints package identities (`DECISIONS.md` `ARC-53`): `list`, `show` one, `validate` one data pack. Described below. |

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

**`packs`** (`DECISIONS.md` `ARC-53`). Package identities, never world state:
- `list` prints one line per pack — type, id, version, licence, authors (`—` for a world, which has
  none), then for a System Pack `system <id>` and for a data pack its directory — and last a count.
  It reads the build's code packs (every System Pack of the installed set in its order, then the
  controllers this binary composes) and then, for each `--packs DIR` in the order given, every
  immediate subdirectory of `DIR` holding `world.yaml` or `pack.yaml`, by name. A subdirectory holding
  neither is not a pack and is not listed; one holding both is refused; a `DIR` that is itself a pack
  is refused. Nothing is read from a directory that was not named.
- `show <id>` prints every package field of one pack, `repository` or `—`, a data pack's `mineworld`
  range, and for a System Pack its system id and `SystemVersion`. An id no source provides is refused,
  listing the ids that exist.
- `validate <directory>` checks one data pack: its package fields, all required, its licence against
  the licence policy, then its content — a World Pack is read (its requirements resolved in the pack
  roots) and loaded as `validate` does; a Presentation Pack's style manifest must have an `id` and a
  `dimension` list.
- `resolve <world>` prints the world's composition (`ARC-54`): the framework's version and the world's
  `mineworld:` range; each requirement with the version that satisfied it and where it was found;
  each enabled system with its pack, version and `bundled` or `third-party` — or the first refusal.

`list`, `show` and `resolve` read the pack roots of `--packs` and then `MINEWORLD_PACKS`. Every pack's
identity is validated: an id outside the rule, a version that is not semver, a licence that is not an
SPDX expression, a missing author, an unknown or missing field, a `type` the file does not carry, and
two packs with one id are each refused by name with a non-zero exit. `list` and `show` print licences;
they do not judge them against the policy.

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
4. **A pack's `SystemVersion` and its release version are two things** (`DECISIONS.md` `ARC-53`).
   `SystemVersion` is the contract counter a save is checked against; the semver version is what a
   range is checked against. Raising `SystemVersion` — which makes existing saves refuse — is a
   breaking release: a MINOR bump before 1.0, a MAJOR bump from 1.0.

**What MVP-0 implements of this list** (`ARC-53`): `id`, `version`, a type, a `mineworld` range for
data packs, an SPDX `license` and provenance (authors, repository), stated by each pack's own carrier
(`PACKAGE_FORMAT.md` §5.0), and printed by `mineworld packs` (§8.1). `dependencies` between code packs
are Cargo's; a world's requirements, `requires` capabilities, `provides` and migration schemas are not
implemented. The **configuration schema** is implemented by `ARC-61`: a pack's `PackConfiguration`
type is its schema, `configure/<id>.yaml` is a world's value for it, and a save is refused at resume
when that value has changed.

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
