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

**The subset MVP-0 implements: item kinds** (`DECISIONS.md` `ARC-71`). Of the contents above, MVP-0
reads **item types** only. New entity types would extend the closed `EntityType` of the contracts, and
component schemas belong to System Packs in MVP-0 (their sections, `ARC-31`); authoring templates are
not read. An MVP-0 Entity Pack is:

```text
modern-goods/
  pack.yaml          id, type: entity-pack, version, mineworld, license, authors (PACKAGE_FORMAT.md §5.0)
  items/
    bread.yaml       one item kind, keyed by the file's name, in the World Pack item-file format (§4.1):
    coffee.yaml      tags, note, and sections — e.g. `item: { category: food }`, owned by `item`
```

A world uses it by requiring it (`requires: { modern-goods: "^0.1" }`, §4.1); its kinds then exist in
that world as if the world had declared them. How a world requiring one is read, and every refusal,
is §4.1's "Requiring an Entity Pack"; `mineworld packs validate` of one is §8.1's.

The repository ships one: `entities/modern-goods`, shop goods (food, drink, everyday things) that
`worlds/lakeside` requires (`DECISIONS.md` `ARC-77`). It is found like any data pack, through a named
pack root (`--packs entities`), and is never "bundled": that word means a code pack compiled from this
workspace (`ARC-54`).

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

and then a rebuild. A pack whose source lives in **another repository** is installed by the same two
lines, its Cargo line naming a git source pinned to a full commit id ([`DECISIONS.md`](DECISIONS.md)
`ARC-66`; how to write one is §3.2):

```text
systems/installed/Cargo.toml           <package> = { git = "<url>", rev = "<40 hex>" }   (one line)
systems/installed/src/lib.rs           <Variant> => <crate>::<System>,                    (one line)
Cargo.lock                             regenerated by Cargo: one git package              (generated)
```

No other file is edited. In particular:
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

**A pack with a section of the World's Interaction List** (§4.2, `ARC-63`) implements
`mineworld_sdk::interactions::InteractionSection` instead and writes `mineworld_sdk::interactions!();`
inside its `impl SystemPack`: its section *is* its configuration, decoded, resolved and seeded by the
SDK. Its `declaration()` passes through `interactions::declare`, its `install()` calls
`interactions::install`, and its `react` calls `interactions::reduce` first; it looks up `permits`,
`parameters` and `consequence` where it decides. The installed set's `Capability::interaction_section()`
names its declared actions and facts for the tools.

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
built binary or a running server, or running a pack's code that nobody reviewed, is the WASM component
model of [`DECISIONS.md`](DECISIONS.md) `ARC-8` (Tier 1), outside MVP-0. A pack from another
repository is in MVP-0 only as `ARC-66` states it: compiled into this build from a pinned commit, after
review, like any dependency. `mineworld install` and `mineworld add-system` (§8) are not implemented;
the two lines are written by hand.

## 3.2 Writing a System Pack outside this repository

A System Pack does not have to live in this repository. It can be a crate in its own repository,
written against the framework's **published surface**, tested there, and installed into a build by
`ARC-33`'s two lines pinned to a commit ([`DECISIONS.md`](DECISIONS.md) `ARC-66`). The first one is
`acme-fishing` (<https://github.com/yuema137/mineworld-pack-fishing>, `DEP-23`).

**The published surface.** The crates a third-party pack may depend on, by version requirement:

```text
mineworld-sdk          SystemPack, package!(), owns_section!(), configures!()
mineworld-kernel       System, WorldView, WorldRead, Emission, ProcessKind, owned_component!
mineworld-contracts    identifiers, Action, Event, Rejection, Visibility, SpatialRequirement
mineworld-authoring    AuthoredSection, Seeding, Reference, ContentKind
mineworld-presence     Presence, Offer, PerceptionProvider — the seam every pack offers through
mineworld-inventory    produce, consume, transfer, can_take — published for packs that change holdings
```

A consuming build maps each name to its own source in one `[patch.crates-io]` table, in
`.cargo/config.toml` at its repository root (`ARC-66` point 3). Any other crate a pack needs comes from
a registry. Depending on a bundled pack outside this list is not possible without a reviewed line in
that table. A pack's dev-dependencies are its own business: Cargo never resolves a dependency's
dev-dependencies, so the pack's tests may use any framework crate (`mineworld-item`,
`mineworld-movement`, `mineworld-rule-controller`, …) through its own `[patch]`.

**The manifest.** A template:

```toml
[package]
name = "acme-fishing"              # the pack's id in `packs`; not mineworld-*, which is the framework's
version = "0.1.0"                  # semver; what a world's `requires:` range is checked against
edition = "2024"
rust-version = "1.97.1"
license = "MIT"                    # an SPDX expression; judged at resolution (ARC-55)
authors = ["…"]
repository = "https://github.com/…"

[dependencies]
mineworld-sdk = "0.1"              # every framework crate by version requirement, never by path or git
mineworld-kernel = "0.1"
mineworld-contracts = "0.1"
mineworld-presence = "0.1"
serde = { version = "1", features = ["derive"] }

[dev-dependencies]
mineworld-movement = "0.1"         # the pack's own tests may use more of the framework

# Standalone builds only: Cargo ignores a dependency's [patch], so a consuming build never reads this.
[patch.crates-io]
mineworld-sdk = { git = "https://github.com/yuema137/MineWorld", rev = "<40 hex>" }
# … one line per framework crate named above
```

No `path` into a MineWorld checkout and no `workspace = true`: the pack is not a member of the
framework's workspace, and the build's acceptance test refuses either (`ARC-66` point 5). For local
development against a sibling checkout, override the patch without editing the manifest:
`cargo test --config 'patch.crates-io.mineworld-sdk.path="../MineWorld/sdk/rust"' …`, one `--config` per
crate. Commit `Cargo.lock` and a `rust-toolchain.toml` matching the framework's, so the pack's own
builds are reproducible.

**The code.** Exactly what §3.1 asks of a bundled pack: `System`, `PerceptionProvider` and
`SystemPack`, whose first line is `const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();`.
`package!()` records the pack as **third-party** because it is compiled outside the framework workspace
(`ARC-54` point 2). Never name a `__private` module of a framework crate: it is not part of the surface.

**Tests.** In the pack's repository, against the framework at the pinned commit: the pack's actions
offered, accepted and refused by each `Rejection`, its facts, and the pack removed (`AC-2`): not
enabled, its actions `Unavailable` and offered nowhere. CI on Linux, macOS and Windows.

**Versioning.** The package version is semver: what a world's `requires:` names. A change to a fact's
meaning or a component's shape bumps the pack's `SystemVersion` (§9), as for any pack. A pack names the
framework by `"0.1"`; a framework release that breaks the published surface is a MINOR release before
1.0, and Cargo refuses a pack whose requirement the build's framework does not meet, naming the crate
and the requirement.

**Installing and upgrading.** The two lines of §3.1 with `rev` = the commit; Cargo regenerates the
lock. To upgrade, change the `rev` and run `cargo update -p <package>`. A world that enables the pack's
system must `require` the pack (`requires: { acme-fishing: "^0.1" }`), or it is refused naming the
system, the pack and `requires` (`ARC-54` point 4).

**Building offline.** After one `cargo fetch --locked`, `cargo build --offline --frozen` builds with no
network. For a machine that has never fetched, `cargo vendor --locked <directory outside the tree>`
elsewhere, copy it, and build with an empty `CARGO_HOME`, `--offline --frozen` and the `--config` the
vendor command prints. Never commit the vendor directory: inside the tree a pack counts as bundled
(`ARC-54`'s limitation).

**Removing.** Delete the two lines; the lock is regenerated. A world enabling the pack's system is then
refused `UnknownSystem`, listing what the build provides; other worlds run unchanged.

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
├── configure/            each enabled System Pack's world-level configuration, typed by it (ARC-61);
│   ├── classes.yaml      the world's entity classes, a framework key (ARC-64, §4.2)
│   └── packages.yaml     the world's licence policy, a framework key (ARC-55)
├── data/                 files a configuration names as attachments (ARC-61 note)
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

configure:                 # optional. Each key names configure/<key>.yaml (ARC-61): an enabled system's
  - <system id>            # id, or one of the framework keys `classes` (ARC-64) and `packages` (ARC-55)

decisions: decisions.yaml  # optional. The decision list (below): which decision model drives each seat.
                           # Read by hosts, never seeded into world state. Absent = every seat `rule`
```

**The decision list (`decisions:`, `decisions.yaml`).** Specified by [`ARC-84`](DECISIONS.md) and
[`ARC-85`](DECISIONS.md); implemented by the DL-a PR of step-24
(`.structured-coding/plans/mvp0/pr-dl-a-decision-list.md`; step-24 §4). The file is World Pack content
read by hosts (`run`, `server --town`, `mineworld decisions`, and through it the cognition process). It is
**never seeded**, journaled, compared at resume or stored in a save: which model decides a Person is not
a fact about the Person (INV-1). The operator may override it.

**The key.** `decisions:` is a path relative to the pack root, written with `/` on every platform. The
World Pack loader (`worldpack/`) owns only the key: it refuses an absolute path, a `\`, an empty, `.` or
`..` component and a drive (`:`), a file that does not exist, and a file whose real path, its links
followed, lies outside the pack. It never parses the file. The format belongs to `mineworld-decision`
(`cognition/decision`), and the host (`tools/cli`) hands that crate the build's installed models
(§5.1) and the world's roster: its seats in roster order, each seat's class (`ARC-64`), the defined
classes of persons, and its people.

```yaml
# decisions.yaml — every key optional
models:                            # the models this file uses, each once; a model not listed runs with defaults
  rule: {}                         # the paced rule controller, as today; no parameters
  motivated:                       # DL-b's model; not installed before DL-b merges
    parameters:                    # typed by the model; unknown keys and out-of-bound values refused there
      social_need_hours: 3
      curves: atus-a3-2025         # a named parameter set the model's pack ships
  lm:                              # an external (out-of-process) model; installed by step-17 P6
    fallback: motivated            # used when the model's process is absent or its budget is exhausted
default: rule                      # absent: rule
assign:                            # seat > class > default; one entry per seat and one per class
  - { class: staff,  model: motivated }        # an ARC-64 class of persons, from configure/classes.yaml
  - { seat: alice,   model: lm }
memory:                            # controller-owned memory policies (§5.2); decoded, not interpreted
  default: { policy: consolidate }
  assign:
    - { seat: alice, policy: lm-summarise, fallback: consolidate,
        parameters: { calls_per_sim_day: 4 } }
```

Rules:

1. **Typed and refused by name.** Every model id the file names (under `models:`, as `default:`, in an
   assignment, as a `fallback:`) must be a model this build installs (§5.1); a refusal lists the
   installed ids. Under `models:` each id appears once; its entry takes only `parameters` and
   `fallback`. `parameters` decode into that model's typed `Parameters`, with bounds and defaults; a
   model without parameters refuses any key. An unknown key anywhere, an unknown model, a seat that
   names no person, a class that is undefined, and an out-of-bound value are refused **at load**, with
   the file, line and column of the offending node, as `configure/` is refused (ARC-61). An external
   model takes no `parameters` until the PR that installs it defines how they are carried.
2. **Seats only, and one class.** An assignment is `{ seat: <key>, model: <id> }` or
   `{ class: <class>, model: <id> }`, never both, never neither. A `seat:` that names no person, or a
   person who is not a seat (`seats:`), is refused, because nothing would drive it. A `class:` must be a
   class of persons defined in `configure/classes.yaml`; an implicit class (`person`, `place`, `item`,
   `organization`) is refused with "use `default:`". Two entries for one seat, or two for one class, are
   refused at the second. A person is in at most one defined class (`ARC-64`), so no other ambiguity
   exists.
3. **Precedence.** A seat's model is its seat entry, else its class's entry, else `default:`, else
   `rule`. Parameters are per model, not per seat: one parameter block per model per file.
4. **Fallback chain.** `fallback: <id | none>` is accepted only on an external model, and an external
   model must name one; on an in-process model it is refused ("an in-process model is always present").
   An external model that is assigned but not listed under `models:` has no fallback and is refused at
   the assignment. Chains are followed at load: at most four links (four `fallback:` steps), no cycle,
   ending at an in-process model or at `none`, where the seat stays idle; a violation is refused at the
   `fallback:` node that starts it.
5. **Operator override.** `mineworld run … --decisions FILE`, `mineworld server … --decisions FILE` and
   `mineworld decisions … --decisions FILE` replace the pack's file; the file is decoded against the
   same world. `--seat-model SEAT=MODEL`, repeatable, then overrides one seat each: the seat must be a
   seat, the model installed, an external model listed under the file's `models:` with its fallback;
   its parameters are the file's block for that model, else defaults; one seat named twice is refused.
   A refusal names the flag.
6. **Absent means today.** With no `decisions:` key and no flag, every seat is `rule`, and every byte of
   every reference world's output — saves and `run`'s printed lines — is unchanged.
7. **Reproducibility without seeding.** When a decision list is in effect (a key or a flag), `run`
   appends `; decisions <digest> (<n> <model>, …)` to its header line, and `server --town` the same
   clause to its `town:` statement. The digest is FNV-1a 64 over the resolved list's canonical encoding
   (each seat's key, effective model, canonical parameters, chain, and, when `memory:` is present, its
   memory choice without source positions), for reading, not evidence, so two files that resolve alike
   print alike. With no list in effect nothing is printed, so every reference world's output is
   unchanged (QDLa-5). A save does not store the list, and resuming with a different list is allowed;
   the header of every invocation shows which list was in effect. Storing it in a save as
   non-authoritative host metadata is deferred until a resume needs it (step-24 QS24-1).
8. **`memory:` is passed through, not interpreted** (QFGb-1). Its shape is the controller memory
   selection of §5.2: `default:` and each `assign:` entry carry `policy`, optional `fallback` and
   optional `parameters`; an entry names a `seat` or a `class` as rule 2 says. The decoder refuses, at
   load and with line and column, an unknown key, a `policy` or `fallback` not spelled as a policy id
   (`^[a-z][a-z0-9-]{0,47}$`), a seat or class rule 2 refuses, a duplicate entry, and `parameters`
   that are not a mapping with string keys or that hold a tag or a non-finite number. It does not judge
   whether a policy exists or its parameters are valid: policy ids are Python entry points. Each seat's
   choice is resolved seat > class > default and emitted by `mineworld decisions <world> --json`
   (§8.1) as the `MemoryPolicyChoice` the cognition process validates when it starts; an unknown policy
   id therefore fails then, naming the file, line and column, not at `run`'s load.

A world does not name a provider, a model name, a URL or a key in this file. The decision list names
models, never backends; the backend binding stays in the operator's `cognition.toml`.

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

**What a place discloses about its doorways.** A person who perceives a place is told its `passages`
(`DECISIONS.md` `ARC-82`), built from the stored passages at that moment. Each entry carries the doorway's
`to` place and its `here` and `there` positions (each `null` where the world models none), and `to_tags`:
the tags of the place `to`, as they stand now, in tag order and always present (`[]` when that place has
none). Nothing else of the destination is disclosed: not its contents, occupants or activity. The stored
`Passages` has no `to_tags`; the field exists only in the disclosure. `component_type` is `passages` and
`schema_version` is `1`.

Passages are also what a **walk** follows (`DECISIONS.md` `ARC-75`). Besides `move`, the `movement`
system provides `walk-to { to: { place: <Location> } | { person: <PersonId> } }` — go to a point in this
place or in any place joined to it by a chain of passages, to a place without a position (enter it), or
to within 1 200 mm of a person in this place — and `walk-step`, no payload: the next stride of the
walker's own walk, at most 1 340 mm, or the crossing at a doorway. The walk is `movement`'s `walking`
component, disclosed to whoever perceives the walker as its destination and its next four waypoints;
it takes no calendar time, and its pace is the cadence at which its sender asks for steps (one a wall
second is 1.34 m/s). A destination no passage chain reaches, or one the route planner cannot reach, is
refused with the code `no-route`. The route inside a place is planned by whichever pack owns the place's
geometry, through `movement`'s `Wayfinder` catalog (`ARC-62`): with `bodies` enabled, round its solids
and loose objects (`DEP-34`); without it, straight. A pack never authors a route.

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
                              (`ARC-39` notes, `DEP-13`). The same shape and objects are what a walk
                              is routed round (`ARC-39` note 5, `DEP-34`)
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
   world already had. The items are the world's own kinds and every required Entity Pack's together,
   in one key order (`ARC-71`).
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
   - of a type a world cannot require — a World Pack or a Controller Pack;
   - bundled — "versioned with the framework; remove it from requires";
   - at a version the range does not admit — naming the id, the range and the version found;
   - a data pack whose own `mineworld:` range does not admit the running framework — naming the pack,
     its range and the framework's version (`ARC-54` note, F-Ed1);
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

**Requiring an Entity Pack** (`DECISIONS.md` `ARC-71`). A requirement may name an Entity Pack (§2): a
pack-root directory holding `pack.yaml` with `type: entity-pack` and an `items/` directory. Requiring it
is using it: its item kinds exist in the world as if the world's own `items:` listed them.

- **What the pack is.** Every `*.yaml` file in its `items/` is one item kind whose key is the file's
  name without `.yaml`; any other file there is not content and is ignored. There is no list to keep in
  step. Each file is in the item-file format above — `tags`, `note`, and sections.
- **When it is read.** After the world's own content has been read and checked for undeclared files
  (and before persons' locations, passages and sections are checked), each required Entity Pack, in id
  order, has its item files read **with the requiring world's enabled systems**: a section is decoded by
  its owner exactly as if the world had written it, and rules 1 and 6 and the section rules above apply
  unchanged — an `item:` section in a pack's file needs the world to enable `item`, and is refused naming
  the pack's file otherwise.
- **One namespace.** A key a required Entity Pack declares that the world also declares (in any of
  `places`, `population`, `items`, `organizations`), or that a second required Entity Pack declares, is
  refused naming the key and both sources. Nothing is overridden.
- **Identity.** Its kinds are allocated with the world's own, in key order (rule 2), so moving kinds out
  of a world into an Entity Pack it then requires changes no identity and no fact.
- **Provenance.** Each of its kinds carries the Entity Pack's id as `source_pack` and `items/<key>.yaml`
  as `source_path` (written with `/` on every operating system). A refusal names the file in the pack's
  own directory.
- **Self-contained.** A section in one of its files may name only kinds the same pack declares; a
  section naming any other key — a `body:` lying `at:` a place — is refused naming the pack, the file and
  the key.

Refused by name when the pack is identified (in every pack root, before any world uses it): an Entity
Pack holding `places/`, `people/` or `organizations/` ("an Entity Pack carries item kinds only in
MVP-0"); one whose `items/` is absent or holds no `.yaml` file ("declares nothing"). Refused when its
content is read: an item file whose name is not a valid key; an item file that does not parse (with its
line and column). A required Entity Pack that is absent, out of range, outside the licence policy, or
whose `mineworld:` range excludes the framework is refused like any other requirement. The two names
the frozen model used and MVP-0 replaced — `entity_packs:` in `world.yaml`, `dependencies` in
`pack.yaml` — stay refused. Line endings are not content: a file written with CRLF reads exactly as one
written with LF.

**Configuration: a world-level file a System Pack owns** (`DECISIONS.md` `ARC-61`). `configure:` lists
the enabled packs this world configures. Each key is a system id and names `configure/<key>.yaml`, which
the owning pack decodes straight into its own type — so a refusal carries its line and column and the
pack's own message — and seeds as its own genesis facts. It is read after requirements resolve. The
loader never learns what a configuration means. It refuses, by name and naming the file:

- a key that is no system of this build, or a system the world does not enable;
- a system that takes no configuration;
- a key listed twice; a listed file that is missing; a `.yaml` file in `configure/` that is not listed;
- a system the configuration requires that the world does not enable;
- an entity the configuration names that is not declared, or not of the type its owner needs;
- a configuration its owner refuses at genesis, or one that seeds another pack's fact or an event type
  its owner did not declare.

A world without `configure:` loads exactly as before the key existed. A save remembers its
configuration: resuming or replaying it against a World Pack whose configuration differs — changed,
added or removed — is refused, naming the system (`ConfigurationDrift`). Other content is not compared
at resume.

**Framework keys** (`ARC-61` note). Two keys of `configure:` name no System Pack; each is owned by a
framework crate, never resolved against the installed set, and never seeded on its own:

```text
classes    configure/classes.yaml, decoded by mineworld_authoring::EntityClasses (ARC-64, §4.2); read
           before any configuration is decoded; refused as ClassesInvalid at its line and column
packages   configure/packages.yaml, decoded by mineworld_packages::LicencePolicy (ARC-55 note); read right
           after the world's systems resolve and before its requirements do, and used in place of the
           default policy; refused as LicencePolicyInvalid at its line and column; not world state, not
           drift-checked
```

**Attachments** (`ARC-61` note). A configuration may name files under `data/`, written as relative
paths with `/` whose first component is `data` (`data/table.csv`), each checked as it decodes — no `..`,
no root, no drive, no `\` — so a path means the same on macOS, Linux and Windows. The loader reads every
named file and refuses one that is missing (`AttachmentMissing`), one whose real path after following
links lies outside the World Pack (`AttachmentOutside`), and one over 4 MiB (`AttachmentTooLarge`). The
owner receives the bytes when it seeds and states what it needs in its own fact, so a changed file is
drift. A file under `data/` that no configuration names is allowed.

Initial state is **not** written into the world by the loader. Each authored `passage`, each
authored `location`, each configuration and each section becomes a recorded event caused by
`Causation::WorldGenesis` — passages first, because they are facts about places that exist before
anybody is in them, then locations, then configuration in `configure:` order (so a section's reduction
may check a value against it), then sections: items', organizations', places', people's, each in key order, and within one
file in the order the world's `systems` lists their owners — which the owning system reduces. What a
person's or a place's section may refer to is seeded before it, and a world that declares no items or
organizations seeds exactly what it seeded before they existed (`ARC-36`). So a loaded world's state has a causal origin in its own log, and a replay rebuilds it
(`DECISIONS.md` `ARC-15`).

## 4.2 The World's Interaction List

A world says what its installed interactions may do — who may do what to whom, with which numbers, and
what a fact means for history — through one document shared by every pack (`DECISIONS.md` `ARC-63`,
`ARC-64`, `ARC-65`, `DEP-28`). The list is **not** a new carrier: it is `configure/classes.yaml` plus
each pack's section, `configure/<pack id>.yaml`, all carried by §4.1's configuration seam.

**Vocabulary.** These terms are defined here and nowhere else; none reuses a term of
[`CORE_CONCEPTS.md`](CORE_CONCEPTS.md).

| Term | Meaning |
| --- | --- |
| **entity class** | A name the list gives to a tag over one entity type: `noble` = a Person carrying the tag `noble`. Every entity also matches the **implicit class** named by its type (`person`, `place`, `item`, `organization`). |
| **section** | One pack's part of the list: `configure/<pack id>.yaml`. |
| **selector** | An entity class, an implicit class, or `*`, in one role of an entry. |
| **role** | A position in an interaction, declared by the owning pack per action, per fact and for its parameters: `actor`, `target`, `object`, `place`. |
| **rule** | `permit` or `forbid` for an action and role selectors. |
| **parameter block** | A pack's typed numbers, each bounded by the pack. |
| **consequence** | For one fact type and role selectors: its audience (narrowed within the owner's bounds), its biographical flag, and the pack's knobs. |
| **reference list** | A section compiled into a pack: always `default` (today's behaviour), and any named list the pack ships. |
| **region** | A place whose section entries override the world's for that place. |

**Classes** (`ARC-64`). `configure/classes.yaml`, listed as `classes` in `configure:`:

```yaml
- { class: noble,    of: person, tag: noble }      # priority is list order
- { class: heirloom, of: item,   tag: heirloom }
```

An entity's class is the first entry whose `of` is its type and whose tag it carries, otherwise its
implicit class. A class is defined once; an implicit name is never redefined; `of` is one of the four
types. Classes are fixed during play and are not seeded on their own: a section's resolved fact copies the
entries it can be affected by.

**A section** (`ARC-63`). Every key is optional and no other key is accepted:

```yaml
extends: default                 # a reference list of this pack; chains of at most four, no cycle
default: permit                  # permit | forbid: what an action no rule matches gets
rules:
  - { action: talk, actor: noble, target: commoner, effect: forbid }
parameters:
  - { gap: 600 }                                     # unscoped: the base
  - { actor: guard, gap: 1200 }                      # scoped: only the fields it names
consequences:
  - { fact: spoke, actor: servant, biography: off }  # audience: public | place | participants
regions:
  library:                                           # a declared place's key
    parameters: [ { gap: 60 } ]
```

**Precedence.** Levels, highest last: the pack's bounds (never overridden), its `default` list, `extends`,
the world's section, a region (for its place only). A higher level replaces a lower level's entry with
the same key (the action or fact plus its selectors), field by field for parameters and consequences.
Then the entry naming most roles wins (an implicit class counts as named). Then, between rules of equal
specificity, `forbid` wins. Two parameter or consequence entries of equal specificity that overlap and
disagree on a field are **refused at load**, naming both.

**A rule decided in a reaction.** A pack may declare a rule name for a yes/no decision it makes while
reducing another pack's fact rather than while validating an action — `relationships`' `acquaint`, asked
per directed pair before an acquaintance forms — and a rule about it is written, resolved and refused
like any other, but it names no action a client can send (`ARC-63` note of 2026-10-10).

**Parameter bounds, per-field roles, place-free lookups, required systems** (`ARC-63` note of
2026-10-13, S24 PR FG-a). Four additions to the shape, each generic:

- *A bound is a value, not only a range.* Each parameter field declares a bound that admits or refuses
  a value and describes itself for the refusal; an inclusive range is the common bound and keeps the
  message `'<field>' is <low> … <high>, not <value>`. A **named choice** is a bound over a typed name
  (`relationships`' `familiarity_decay`: one of the bundled curves or one its extension catalog lists),
  refused as `'<field>' is one of <names>, not <value>`.
- *A field may narrow the roles that scope it.* A pack declares, per parameter field, the subset of its
  parameter roles that may scope an entry setting that field; a field not declared may be scoped by
  every parameter role. An entry that sets a field and names a role outside that field's subset is
  refused at its line and column; so is a region entry that sets a field whose roles exclude `place`.
  (`relationships`' decay fields: `actor`, `target`; `known_at_most`: `actor`; `conversation`'s
  `forget_after_days`: `target`.)
- *A place-free lookup.* For exactly those fields, a pack looks parameters up with the place role
  unfilled: the base resolution with the given roles, or the compiled defaults, reading nothing, when
  the pack is unconfigured. A disclosure, which has no causing place, uses it.
- *A section may require systems.* A pack may state that its section, as written, requires other
  systems to be enabled (`relationships`: the pack implementing each catalog curve the section names);
  the loader refuses, by name, a section requiring a system the world does not enable, as for any
  configuration (§4.1). The loader registers the build's extension catalogs before it decodes
  configuration, so a catalog name is known at decode.

**What a section is refused for**, by name, naming the file:

- at its line and column, while its owner's type decodes it: an undeclared action, fact or role; a
  parameter outside its bound or unknown; a parameter field set under a role its owner does not let
  scope it, or in a region when its owner does not let a place scope it; an audience wider than the
  owner's default or narrower than its narrowest; `biography` on a fact whose owner does not allow it;
  a region rule for an action that is not regional; an `extends` naming no reference list, or a cyclic
  or over-long chain;
- with the list and the index of each entry, once the classes are read: a selector naming a class that
  is neither defined nor implicit (`ClassUndefined`); two ambiguous entries (`AmbiguousEntries`);
- a region naming no declared place, as any configuration naming an unknown entity (§4.1).

**What a list cannot do.** Grant: a `permit` is a filter on the pack's own checks, never a bypass. Name
an action, a fact or a role its owner does not declare. Exceed a bound. Widen an audience, or switch
biography where the owner does not allow it. Stop a fact from being recorded, change a payload or a
subject, or route another pack's fact. Configure a pack the world does not enable. Reach a client: a
client sees affordances (an offer refused `PermissionDenied`), requirements, tags and the facts it
perceives — never the list.

**At run time.** A configured section is one genesis fact `<pack>-interactions-configured`
(`SystemInternal`, no subjects), reduced by its pack into a component `<pack>-interactions` on every
Place. The pack looks up `permits`, `parameters` and `consequence` on the place it is deciding at (or,
for fields no place may scope, place-free); a pack
the world does not configure answers its compiled defaults without reading state, so an unconfigured
world is byte-identical. Editing a section or a class it references is refused at resume as drift.
`mineworld interactions <world> [--place KEY] [--json]` prints what precedence produced, and each
entity's class.

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

## 5.1 Decision models

A **decision model** is a Controller Pack's answer to "what does this seat attempt?". It is a pure
function of what the seat observes. A world chooses one per seat through `decisions:` (§4.1). Terms
follow [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md): a decision model is a Controller, not a Person's
property, and it never changes who a Person is (INV-1).

**Two kinds.**

- **In-process (Rust).** A model implements the `DecisionModel` trait of the crate `cognition/decision`
  (`mineworld-decision`). That crate depends on `mineworld-contracts`, on `mineworld-sdk` for the
  `parameters!` machinery only (so a model's bounds are checked by the same code as an Interaction List
  section's, QDLa-4), and on `serde`, `serde_json` and `serde-saphyr` to read `decisions.yaml`. The trait:

  ```rust
  pub struct Consult<'a> {                  // host context, never preferences
      pub seed: Seed,                       // the run's seed, the same for every seat
      pub pace: SimDuration,                // the answer window: the host's consult period, world seconds
      pub observation: &'a Observation<Value>,
  }
  pub trait DecisionModel: Default + Send + Sync + 'static {
      const ID: &'static str;               // the id decisions.yaml names; the identifier rule; not "none"
      const PACKAGE: Package;               // the model crate's package identity (ARC-53)
      type Parameters: ModelParameters;     // typed, bounded, with defaults; () for none
      fn decide(&self, parameters: &Self::Parameters, consult: &Consult<'_>) -> Option<ActionRequest>;
      fn step(&self, parameters: &Self::Parameters, consult: &Consult<'_>) -> Option<ActionRequest>;
  }
  ```

  `decide` is the consult; `step` is the next stride of a disclosed walk between consults. A host holds
  each seat's model as the object-safe `SeatModel` (`Configured<M>`: the model with its decoded
  parameters). `ModelParameters::decode` decodes a parameter block at the decoder's position, refusing an
  unknown key and an out-of-bound value there; a block made by `mineworld_sdk::parameters!` implements it
  as `mineworld_decision::bounded`, and `()` refuses every key. The signatures carry no clock, no network
  and no world handle. The receiver is `&self`, and a model crate may not use interior mutability: its own
  `clippy.toml` repeats the workspace's disallowed types and adds `Cell`, `RefCell`, `OnceCell`,
  `UnsafeCell`, `Mutex`, `RwLock`, `OnceLock`, `LazyLock` and the atomics. Randomness comes only from the
  provided `Seed` and the counter-based `Draw` (`Draw::new(seed, observation)` for the rule controller's
  stream, `Draw::salted(salt, seed, observation)` for a model's own); a model documents the draw indices
  it uses. A model that keeps state across consults is not conformant, and the template test below
  refuses it.
- **Out-of-process (any language).** A model is a Controller Pack running as a protocol client
  (step-17 §3.2): it joins a seat with the same frames as any client. Its `decisions.yaml` entry carries
  its `fallback` (and, once the PR that installs it defines them, model-specific parameters). While its
  process is absent, the seat's host runs the end of the fallback chain: `run` always (it hosts no
  out-of-process model in MVP-0, QS24-12, and its header says so: `; alice: lm → rule (run hosts no
  out-of-process model)`), and `server --town` as the seat's in-server controller until the external
  process takes the seat over through the existing take-over path (S11-B).

**Registration.** `cognition/installed` (`mineworld-installed-models`) lists every model this build
provides, one line each, through the `models!` macro of `mineworld-decision`:

```text
mineworld_decision::models! {
    Rule => mineworld_rule_controller::Rule,      // in-process: a DecisionModel type
    external "lm",                                // out-of-process: an id only (step-17 P6 adds it)
}
```

`decisions.yaml` model ids resolve through that list, as System ids resolve through `systems/installed`
(ARC-33). Each model crate is a Controller Pack with its own package identity (ARC-53); `mineworld packs
list` lists the in-process models' packages from this list. Installing a model is one line in
`cognition/installed/Cargo.toml` and one line in its `models!`, then a rebuild; no existing model,
System, contract or host changes. A guard test holds the list and the manifest's dependencies equal and
refuses an id listed twice, an id outside the identifier rule, and the reserved id `none`. A template
test in `tools/cli/tests` runs a reference world with every seat on each installed in-process model,
twice and across a kill-and-resume, and compares the saves byte for byte (AC-6), so a new model is held
to determinism with no test edited.

**What a model may attempt.** A model chooses only among affordances an installed System offered, after
the world's Interaction List has filtered them (ARC-63, ARC-34). Every request is revalidated at dispatch.
Removing a System removes options from every model at once. Decision parameters change preferences among
offered options; they cannot make an option available.

**Determinism, by host.**

| Host | Model kind | Guarantee |
| --- | --- | --- |
| `mineworld run` | in-process | Each model is a pure function of `(parameters, seed, pace, observation)`. The same pack, seed, decision list and code give the same history. Kill-and-resume equals an uninterrupted run (AC-6). |
| `mineworld run` | out-of-process | Not hosted in MVP-0. The seat uses its fallback, and the run header says so (step-24 QS24-12). |
| hosted server | any | Requests are journaled. Replay re-executes the journal and never calls a controller (ARC-25). Re-running an LM seat reproduces its decisions only under cassette replay (ARC-58). |

**Examples.** `rule` is today's paced rule controller, unchanged, and remains the default. `motivated` is
the no-LLM utility model (step-24 DL-b). `lm` is the LM Controller (step-17 P6), which is the LLM example.
A director is a Controller Pack seated on an Organization; it is valid only once the Organization-seat rule
is decided and recorded before the director PR (step-24 QS24-2), and `CORE_CONCEPTS.md` §14 is amended then.

## 5.2 Memory policies and host bounds

**Field reference.** This section states the shape and the rules. The field-level reference for the
operator's cognition configuration (`cognition.toml`) is generated from its Pydantic models into
[`reference/cognition-toml.md`](reference/cognition-toml.md); the parameters of each memory policy are
generated into `reference/memory-policies.md` once the policies exist (S25 PR 25b, `CLAUDE.md` §2.1). A
generated page is never edited by hand.

A **memory policy** decides which items of a seat's controller-owned memory are kept, dropped or merged
into a summary. It is part of a Controller Pack. It is selected per seat by `memory:` in `decisions.yaml`
(§4.1), never by `configure/`, because a controller file may not configure world state. System-owned
person state is not controller memory and is never governed by a memory policy; its forgetting rules are
configured in the owning System (`CORE_CONCEPTS.md` §5.3).

**Interface.** Inputs and outputs are strict, frozen Pydantic models (`mineworld_cognition.memory.retention`);
no `dict[str, Any]`. A policy is a class with a constant `id` (equal to its entry-point name, matching
`^[a-z][a-z0-9-]{0,47}$`), a `Parameters` model (strict, every field bounded and defaulted) and a pure
`plan(items, budget, params)`. It receives:

- a tuple of `MemoryItem`s, one per eligible item: an opaque `ItemId`; its level (0 an L0 record, 1 an
  episode, 2 a chapter, 3 a stable fact); its `container` (the unit it nests in: an L0 record's episode, an
  episode's week chapter if one exists, otherwise none, which makes it top-level); `first_at`, `last_at`
  and `day` in world time; its kind; its counterparts and place; whether the seat was the actor; its
  host-computed `importance`; its `Citation`; its names-only text; and its own size in the canonical dump;
- a `RetentionBudget`: the bytes the surviving items should fit in, `now` (world seconds at the start of
  the day whose first fact triggered the pass), and the model calls this pass may make (0 for every no-LLM
  policy).

It returns a sequence of decisions: `Keep(item)`; `Merge(items, into=1)`, L0 records of one episode into
that episode; `Merge(items, into=2)`, episodes of one chapter into that chapter; `Drop(item)` of a
top-level item. A policy never receives a store, a connection, a clock, a path or a gateway, and only the
host deletes rows. An assisted policy (FG-c) is a separate kind with a separate host branch; its prose per
`Merge` is at most 160 bytes.

**Importance and recency belong to the host.** `importance` (0 … 1 000) is computed by the host, never by
a policy or a model. Each L0 record's base importance is fixed at ingestion by its event type's renderer
(for example, a line addressed to me 700, another person's walk 30; an unknown event type gets the generic
renderer's value). An episode's base is the maximum of its records', a chapter's the maximum of its
episodes', both written when the unit closes so that they survive the removal of their members. At a pass
the host adds a reinforcement bonus of `min(200, 2 × the largest exchange count among the item's
counterparts)`, read from the stable facts at that moment. A stable fact's importance is
`min(1000, exchanges + 10 × times_met)`. Recency is event time, `now − last_at`, computed by the policy,
never time of last retrieval, because a store whose contents depended on retrieval history would no longer
derive from what the seat perceived (ARC-59, ARC-87).

**Host responsibilities.** These hold for every policy, including user policies and the hostile fixture,
and a policy cannot bypass them.

1. **When.** A pass is due when the newest ingested fact falls on a later simulated day than the last
   pass. The host (`absorb`) splits each perceived batch at every first fact of a new day, ingests up to
   and including that fact, and runs the pass, so the pass point is fixed by the perceived facts alone,
   never by batch boundaries; a jump over several silent days is one pass. A crash between the ingest and
   the pass is repaired by the next `absorb`, which runs the due pass first. Ingestion alone (`ingest`)
   never forgets.
2. **Eligibility.** Every L0 record of a closed episode, every closed episode, every chapter and every
   stable fact. The open episode and its record are never shown and never removed.
3. **Validation.** The plan is refused whole, by a typed reason, when a decision names an unknown item
   (`unknown-item`), names an item twice (`duplicate`), merges items that are not of the level below `into`
   or do not share one surviving container (`merge-mismatch`, which also covers a stable fact in a merge),
   drops a nested item (`drop-nested`), explicitly keeps an item whose container it removes
   (`keep-under-removed`), or is not a sequence of decisions at all (`malformed`). A policy that raises is
   refused too. An item no decision names is kept, unless its container is removed, which it then follows.
   A refused plan is never partly applied: the pass applies `consolidate` with its defaults, found through
   the same entry-point group as every policy, and records `fallback:<reason>`.
4. **Execution.** `Merge(into=1)` deletes the L0 records; their episode keeps its text and its citation,
   which already is the run of its members. `Merge(into=2)` deletes the episodes and any record still in
   them; the chapter's citation is stored as the runs of consecutive episodes it was written from.
   `Drop` deletes the unit and everything nested in it and adds its citation ranges to the forgotten set; a
   dropped stable fact adds nothing, because a stable fact cites its establishing facts and covers none.
5. **Coverage invariant.** At every pass, every perceived id up to the newest ingested is represented by
   exactly one top-level unit (a chapter's ranges, an episode of a week with no chapter, the open
   episode's record) or by the forgotten set; nested units lie inside their container; no L0, L1 or L2
   citation intersects the forgotten set (AC-FG-3). A stable fact's citation may point into the forgotten
   set: what established it still resolves in the save. The host checks the store-side half of this after
   every pass and rolls the pass back on a breach (a defect, never a fallback).
6. **Forgotten set.** Sorted, disjoint ranges of event ids over the seat's perceived sequence. Two ranges
   with no surviving unit between them merge. Beyond 256 ranges the oldest are removed and the watermark
   rises to their end, meaning "every perceived id at or below the watermark that no surviving unit
   represents is forgotten". The set is bounded by construction, and every forgotten id still resolves in
   the save (`mineworld perceived`).
7. **Hard bounds.** After executing a plan, while the canonical dump exceeds the host target of 786 432 B
   (768 KiB), the host applies, in order and until it is under: `consolidate` with its defaults; merging
   every remaining L0 record of a closed episode into its episode, oldest first; merging the episodes of
   weeks with a chapter into it, oldest first; dropping chapters, oldest first; dropping chapterless closed
   episodes, oldest first; dropping stable facts, fewest meetings first. Every such removal is counted as
   forced in the pass log. The post-pass dump is therefore at most the target, and the gap to the hard
   bound of 1 048 576 B (1 MiB) is room for one simulated day of ingestion before the next pass. If the
   store is still over after the last step, the pass raises `RetentionBoundUnreachable` and rolls back.
   Neither number is a parameter: no world or policy can loosen them.
8. **Context bound.** A retrieval for a model call is at most 6 000 bytes, per section, as P4 D-P4-9 defines
   it. This bound is not changed by any policy. Retrieval reads no row a policy may have removed.
9. **Pass log.** Each pass records its day, policy, outcome, the counts kept, merged, dropped and forced,
   the dump size before and after, and the number of forgotten ranges; the newest 32 rows only.

The cognition budget ledger is not memory and is not pruned by a pass: it prunes itself, by wall time,
when it records a charge, keeping two wall days, twice its longest window (ARC-57, `ARCHITECTURE.md` §9.1).

**Selection.** A world selects a policy per seat through `memory:` in `decisions.yaml` (§4.1). The Rust
reader resolves seat > class > default and emits, per seat, a typed JSON `MemoryPolicyChoice` (`policy`,
optional `fallback`, `parameters`, and the source span of the winning entry). The cognition process
resolves it when it starts: an unknown policy is refused with the installed ids; invalid parameters are
refused with the policy, the field and the source span (never the given value); a `fallback` on a policy
that is not assisted is refused. With no `memory:` the choice is `{ policy: consolidate }`.

**Bundled policies.** Each is an ordinary entry point of the group `mineworld.forgetting` in
`mineworld-cognition`, imports only the public names of `mineworld_cognition.memory.retention`, and none
is privileged. A user policy is a Python package that exposes one entry point of that group, loaded by
`importlib.metadata`. All arithmetic is integer.

| Policy | Uses a model | Rule | Parameters (bounded; default) |
| --- | --- | --- | --- |
| `consolidate` (default) | no | L0 records with `last_at < now − l0_hours · 3 600` merge into their episode. Episodes in a chapter with `last_at < now − l1_weeks · 604 800` merge into it. Chapters with `last_at < now − l2_years · 31 536 000` drop, except a chapter whose ranges hold a stable fact's first, last or level fact. Stable facts are kept. | `l0_hours` 0 … 720 (0); `l1_weeks` 1 … 104 (8); `l2_years` 1 … 50 (5) |
| `decay` | no | Retention `r` (per mille) of `now − last_at`: `half-life` halves per whole half-life, linear between; `hyperbolic` is `1000 · s / (s + Δ)`. An L0, L1 or L2 item survives while `importance · r / 1000 ≥ threshold`; a nested non-survivor merges into its container, a top-level one drops. Survivors over `store_bytes` get `consolidate`'s defaults. Stable facts are kept. | `curve` `half-life` \| `hyperbolic` (`half-life`); `half_life_hours` 1 … 87 600 (168); `scale_hours` 1 … 87 600 (72); `threshold` 0 … 1 000 (100) |
| `importance-recency-cap` | no | Score = `(w_importance · importance + w_recency · r) / 1000`, `r` by half-life. Greedy by score, top-down: top-level units, then episodes inside chosen chapters, then records inside chosen episodes, then stable facts, each level up to its `k` and all within `store_bytes`. Ties are broken by the sha256 of the seed and the item's first Event ID, so selection is a pure function. Unchosen nested items merge; unchosen top-level items and stable facts drop. | `w_importance` 0 … 1 000 (700); `w_recency` 0 … 1 000 (300); `half_life_hours` 1 … 87 600 (168); `k_l0` 0 … 100 000 (2 000); `k_l1` 0 … 100 000 (400); `k_l2` 0 … 100 000 (260); `k_l3` 0 … 10 000 (150); `seed` 0 … 2³² − 1 (0) |
| `keep-all` | no | Keeps everything; the host bound then applies. A test fixture only, for historical comparisons and the hostile-policy test. Not for use in a world. | none |
| `lm-summarise` | **yes** | Candidate groups are chosen exactly as `consolidate` would choose them. For each group, one summarisation call, with names but no ids, at most 6 000 bytes per request, returns prose of at most 160 bytes and an optional importance hint 1 … 10 that may move a group between `Merge` and `Drop`. The model may decide which items survive. | `calls_per_sim_day` 0 … 24 (default 4); the tier is bound in the operator's `cognition.toml` |

**Model calls.** Every call of an assisted policy goes through the `ModelGateway`. The budget gate runs
first (ARC-57), then the recorder (ARC-58), and the request is keyed by its provider-neutral form. In replay
mode a missing recording raises `CassetteMiss`, which the pass propagates; no policy may catch it and fall
back silently. In live mode, a refusal, a failure, an unbound tier, or `calls_left = 0` makes the host apply
the policy's declared `fallback` to the remaining groups. The seat's store records which policy produced each
decision. Rebuilding a store with its recordings reproduces it byte for byte. Rebuilding without them
reproduces the fallback's store and says so. An assisted policy names a no-model `fallback`, and it reaches a
model only through the host's summary gateway, which enforces the request bound and the pass's call allowance
before the `ModelGateway` is called; each pass records whether the fallback decided any group and why (ARC-87).
Forgotten Event IDs stay resolvable, and no summarisation request contains an Event ID or an entity id (AC-FG-2).

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

Not every pack needs every directory. What `assets/asset_bindings.yaml` and `renderer/godot.yaml`
contain for the 2D reference client is specified in
[`clients/2d/PRESENTATION.md`](../clients/2d/PRESENTATION.md) (`DECISIONS.md` `ARC-46`). A style that covers more than one dimension is more than
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
                         [--town [--seed N] [--pace SECONDS] [--decisions FILE]
                         [--seat-model SEAT=MODEL]...] [--hold SECONDS] [--time-scale N]
                         [--keyframe-every N] [--admin-token TOKEN] [--packs DIR]...
mineworld validate <world> [--packs DIR]...
mineworld replay <world> --save DIR [--packs DIR]...
mineworld run <world> --headless --seed N --days N [--save DIR] [--decisions FILE]
                      [--seat-model SEAT=MODEL]... [--packs DIR]...
mineworld decisions <world> [--decisions FILE] [--seat-model SEAT=MODEL]... [--json] [--packs DIR]...
mineworld inspect <save-directory> [--last N]
mineworld biography <world> --save DIR --person KEY [--json] [--packs DIR]...
mineworld perceived <world> --save DIR --person KEY [--since ID] [--json] [--packs DIR]...
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
`requires` line per requirement, after `seats`, only when the world states `requires:`; its `items` line
lists the world's item kinds composed with every required Entity Pack's (`ARC-71`).

| Command | What it does |
| --- | --- |
| `server` | Hosts a World Pack for clients (`NETWORKING.md`). In-server controllers run on the world thread (`DECISIONS.md` `ARC-42`): `--agent SEAT` drives that seat with the reactive rule controller, which answers only what it hears while bound (`F-13`); `--town` drives every other seat with its resolved decision model (§4.1; with no decision list, the paced rule controller), seeded by `--seed N` (default 0) and consulted every `--pace SECONDS` **wall** seconds (default 5), seat `k` at `genesis + (k + m·pace)·time_scale` in world time; the reactive controller is consulted once a wall second. The pack's `decisions:` file, `--decisions FILE` and `--seat-model SEAT=MODEL` choose the `--town` seats' models; `--decisions` or `--seat-model` without `--town` is refused ("decisions choose how in-server seats decide; add --town"), a seat named by both `--agent` and `--seat-model` is refused, `--agent` otherwise wins over the list for its seat, and a seat whose fallback chain ends at `none` is not driven in-server. With a decision list in effect, the statistics line's `town:` statement ends with the decisions clause of §4.1 rule 7. A paced seat whose person is walking is also asked for the next stride of its walk (`walk-step`) once a wall second, every `time_scale` world seconds, so a hosted walker covers 1.34 m per wall second at any time scale (`ARC-42` note of 2026-10-09; `ARC-75`). Cadence is wall time so that the time scale never makes a hosted Person walk or talk faster (QTW-13, amending `ARC-40`). A client joining a seat an in-server controller drives takes it over; when it leaves, or its hold ends, the seat returns to a controller built afresh (`server/PROTOCOL.md` §4.2, `ARC-40`). `--hold SECONDS` is how long a dropped connection's seat is held for its `resume`, in wall seconds (default 30; `0` holds none). `--time-scale N` is how many world seconds pass per wall second (an integer ≥ 1, default 1), reported as `time_scale` in `/status` and every welcome. `--keyframe-every N` (an integer ≥ 1, default 50) makes every Nth frame to a client a whole observation and the others `delta` frames (`server/PROTOCOL.md` §5.3, `DECISIONS.md` `DEP-15`); `1` sends every frame whole. On Ctrl-C the server prints `[world] ticks N, p50 A ms, p99 B ms, longest tick M ms` and one `[world] hosted <seat>: …` line per in-server controller with its consults and outcomes. `--save DIR` keeps the world in `DIR/world.sqlite`, created the first time and resumed afterwards. `--invite TOKEN` (or the environment variable `MINEWORLD_INVITE`; the flag wins) is the invite every client must present in its `join`; with neither, the server generates one and prints it once as `[mineworld] invite <token> — join with: <address> seat=<seat> invite=<token>`. There is no mode without an invite, loopback included (`server/PROTOCOL.md` §4.1, `DECISIONS.md` `DEP-14`). `--admin-token TOKEN` (or the environment variable `MINEWORLD_ADMIN_TOKEN`; the flag wins; `--help` names the variable and never a value) mounts the admin surface under `/admin` — sessions, seats, kick, release, and pausing and resuming the host clock (`server/PROTOCOL.md` §11, `DECISIONS.md` `ARC-44`) — and the server prints `[mineworld] admin surface: http://<address>/admin (bearer token as given)`; without it no `/admin` route exists and the server prints `[mineworld] no admin surface (no --admin-token)`. The admin token follows the invite's rules, is never printed, and must differ from the invite: an illegal token, or one equal to the invite, stops the server before it listens with a message that repeats neither. Launchers pass it through the environment, not the command line. On Ctrl-C — and on Windows also Ctrl-Break, closing the console window, or logging off — the server stops the same graceful way: it stops accepting connections, checkpoints the save and prints its statistics. |
| `validate` | Reads and loads a World Pack and reports the world it describes: systems, places, people, seats, the identity each key received, the number of genesis facts. When `world.yaml` names a decision list, it is decoded (§4.1) and one line follows the `requires` lines: `  decisions  <n> assignment(s), default <id>`; without the key nothing new is printed. |
| `replay` | Re-executes a save's whole journal from genesis and checks every fact and snapshot byte for byte (`ARC-25`). |
| `run` | Runs a World Pack headless: no renderer, no network, no model. Every seat the pack offers is driven by its resolved decision model (§4.1, §5.1) — with no decision list, the seeded paced rule controller (`ARC-27`). Described below. |
| `decisions` | Reads a World Pack and its decision list exactly as `run` does (the same decoder, `--decisions` and `--seat-model`, the same refusals), starts no world and writes nothing. Described below. |
| `inspect` | Reports what a save holds, without resuming or writing it. Described below. |
| `biography` | Prints a Person's objective biography, derived from a save's fact log without resuming or writing it ([`DECISIONS.md`](DECISIONS.md) `ARC-29`). Described below. |
| `perceived` | Prints the facts a Person perceived, judged from a save's fact log by the same audience rule the server's `perceived` stream uses, without resuming or writing the save ([`DECISIONS.md`](DECISIONS.md) `ARC-43`). Described below. |
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

Each seat `k` is consulted on the lattice `genesis + k + m·900` and, while its person is walking (has
movement's `Walking`), also at the step instants `genesis + k + n·30`: `RUN_STEP`, a notional stride
cadence, because `run` has no time scale (`ARC-27` note of 2026-10-09; `ARC-75`). At a step instant
the controller is asked only for the next stride of its walk. A pack with 30 or more seats is refused
naming `RUN_STEP`.

Each seat is consulted through its resolved decision model with the run's seed and the pace 900 as the
answer window (§5.1); a seat whose fallback chain ends at `none` is never consulted. `--decisions FILE`
and `--seat-model SEAT=MODEL` override the pack's decision list (§4.1 rule 5). The list is read afresh by
every invocation and never stored in the save, so a resumed run uses the list it is given.

`run` prints, in this order: one line naming the world and its seats (ending, only when a decision list
is in effect, with the decisions clause of §4.1 rule 7, and one `; <seat>: <assigned> → <runs> (run hosts
no out-of-process model)` clause per seat assigned an external model); one `day` line per completed
simulated day (`day D  revision R  facts F`; revision `-` without `--save`; F counts the facts
this invocation recorded); a summary of consults (lattice and step), requests by action type and outcome, facts by
event type, accepted strides (`move` — a `move` or a `walk-step`) and `talk` per seat in each 30-day bucket, faults, and the history
fingerprint, which always covers the world's whole history; and last, on its own line beginning `wall`, the elapsed real time. Everything
before the `wall` line is a function of the pack, the seed, the age and the code. The fingerprint
(`FNV-1a 64` over every fact's stored encoding in `EventId` order, printed with the number of facts
it covers) is for a person comparing two runs by eye; it is not a proof of equality.

**`decisions`.** Prints each seat's resolved decision model and memory choice (§4.1), in roster order.
Without `--json`, one line per seat for a person. With `--json`, one JSON document on stdout, the form the
cognition process reads (QFGb-1):

```json
{
  "format": 1,
  "world": "social-cafe",
  "digest": "00f3…",
  "seats": [
    { "seat": "alice",
      "model": { "id": "rule", "assigned": "lm", "chain": ["lm", "rule"] },
      "memory": { "policy": "lm-summarise", "fallback": "consolidate",
                  "parameters": { "calls_per_sim_day": 4 },
                  "source": { "file": "decisions.yaml", "line": 12, "column": 7 } } },
    { "seat": "bob", "model": { "id": "rule" }, "memory": null }
  ]
}
```

`format` is the document's version; a change to its shape bumps it, and a reader refuses a version it
does not know. Keys appear in the order shown. `model.id` is the in-process model that drives the seat in
`run` (`null` when its chain ends at `none`); `assigned` and `chain` appear only when the seat is assigned
an external model. `digest` is §4.1 rule 7's. `memory` is `null` when no `memory:` entry and no `memory:`
default applies (the cognition process then uses its default); otherwise it is the winning entry's
`policy`, `fallback` (`null` when absent), `parameters` (`{}` when absent; keys sorted) and `source`: the
file as written in `world.yaml`'s `decisions:` or as passed to `--decisions`, and the line and column of
the winning entry's mapping. The command is read-only.

**`inspect`.** Reads `DIR/world.sqlite` and prints the manifest (pack, instance, composition), the
head revision and its instant, the journal counted by input kind and outcome, the snapshots the save
retains with each one's stored size on one line (`DECISIONS.md` `ARC-81`), the facts counted by
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

**`perceived`.** Reads `DIR/world.sqlite`'s manifest and fact table, and the World Pack `<world>` for
the authoring keys, and nothing else: it opens the save, reads it and closes it, so no handle outlives
the command. It refuses, by name and with a non-zero exit, a missing save, a save whose manifest names
another pack, and a KEY the pack does not declare as a Person. It folds the whole fact log from the
first fact through the audience rule of the `presence` pack (`ARC-43`) and prints every fact the Person
learned of, oldest first, one per line:

```text
<event id> <at> <event type> place=<place key or -> caused_by=<cause>
```

`--since ID` prints only the facts with an `EventId` greater than `ID` (the earlier facts are still
folded, since where people were decides later judgements); it is the cursor of the server's
`perceived` stream (`server/PROTOCOL.md` §5.8). With `--json`, each fact is one line holding the
`PerceivedEvent` exactly as the server's `perceived` frame carries it, so a client's recorded stream and
this export compare line for line. A world without the `presence` pack perceives only `Public`,
`Participants` and `Entities` facts, and the `Place` facts that name the Person.

**`create`.** The directory's final component becomes the world's id and must be a valid key (the
rule `EntityKey` enforces). The pack written has one place, two people who are both seats, and the
systems `presence`, `movement` and `conversation`, and it is read and loaded before `create`
reports success. An existing directory is refused and left untouched.

**`packs`** (`DECISIONS.md` `ARC-53`). Package identities, never world state:
- `list` prints one line per pack — type, id, version, licence, authors (`—` for a world, which has
  none), then for a System Pack `system <id>` and for a data pack its directory — and last a count.
  It reads the build's code packs (every System Pack of the installed set in its order, then the
  Controller Packs of the in-process decision models `cognition/installed` lists, in its order, each
  package once) and then, for each `--packs DIR` in the order given, every
  immediate subdirectory of `DIR` holding `world.yaml` or `pack.yaml`, by name. A subdirectory holding
  neither is not a pack and is not listed; one holding both is refused; a `DIR` that is itself a pack
  is refused. Nothing is read from a directory that was not named.
- `show <id>` prints every package field of one pack, `repository` or `—`, a data pack's `mineworld`
  range, and for a System Pack its system id and `SystemVersion`. An id no source provides is refused,
  listing the ids that exist.
- `validate <directory>` checks one data pack: its package fields, all required, its licence against
  the licence policy, a `pack.yaml` pack's `mineworld:` range against the running framework (`ARC-54`
  note), then its content — a World Pack is read (its requirements resolved in the pack roots) and
  loaded as `validate` does; a Presentation Pack's style manifest must have an `id` and a `dimension`
  list; an Entity Pack's item files are each read against **this build's whole installed set** (each
  section decoded by its owner; whether a world enables that owner is checked when a world requires the
  pack), must be self-contained (§4.1), and are listed on an `items` line (`ARC-71`).
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
