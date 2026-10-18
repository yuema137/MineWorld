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

**The subset MVP-0 implements: item kinds and owned data directories** (`DECISIONS.md` `ARC-71`, amended
2026-10-15 by `ARC-121`). Of the contents above, MVP-0 reads **item types** and the **data directories an
installed System Pack owns** (`ARC-121`; implemented by S30-b). New entity types would extend the closed
`EntityType` of the contracts, and component schemas belong to System Packs in MVP-0 (their sections,
`ARC-31`); authoring templates are not read.

*Amended 2026-10-15 (S30-a, `ARC-121`).* The sentence read, until then: ~~"Of the contents above, MVP-0
reads **item types** only."~~ A data directory is not a component schema: its files are data that the
owning System Pack decodes and validates, and the Entity Pack still makes nothing happen.

An MVP-0 Entity Pack that declares item kinds is:

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

**Data directories** (`DECISIONS.md` `ARC-121`; implemented by S30-b). An Entity Pack may also carry, beside
or instead of `items/`, a directory that an **installed** System Pack owns (§3.1, "A pack that owns data"). The
loader collects its `.yaml` files and hands them to the owner, which decodes and validates them with line and
column; the loader never learns what they mean. Two rules hold for every Entity Pack:

- **Only an installed owner's directory.** A directory that is neither `items/`, nor a directory some installed
  System Pack owns, nor one whose name begins with `.` (ignored, as non-`.yaml` files are) is refused as
  `UnknownDirectory`, listing every directory this build knows. A directory whose owner is installed but not
  enabled by the requiring world is refused naming the owner (`DataNeedsASystem`), as a section of a disabled
  pack is (§4.1 rule 6).
- **Flat.** A data directory holds files only; a subdirectory inside it is refused as `UnknownDirectory`.

The data directories the `attributes` System Pack owns, and the format of each, are §4.1's "Attributes"
(`ARC-120` … `ARC-126`):

| Directory | Holds | Format |
| --- | --- | --- |
| `attributes/` | attribute declarations and overrides | §4.1 "Attribute declarations", "Overrides" |
| `populations/` | population profiles | §4.1 "Population profiles" |
| `vocabularies/` | value vocabularies a `set` or `category` attribute draws from | §4.1 "Vocabularies" |
| `masks/` | masks: named selections over a backend's catalogue | §4.1 "Backends and masks" |
| `mappings/` | a backend's views of built-in attributes | §4.1 "Backends and masks" |
| `labels/` | per-locale labels, phrases and words (S30-k) | §4.1 "Rendering" |

A **backend pack** also carries `backend.yaml` at its root, the backend contract (`ARC-126`; §4.1 "Backends
and masks").

Three Entity Packs of this shape are specified (none is shipped before its PR):

```text
entities/person-attributes/     the built-in catalogue: backend `mineworld`, namespace `core` (S30-e)
  pack.yaml                     id, type: entity-pack, version, mineworld, license: CC0-1.0, authors
  backend.yaml                  the backend contract (ARC-126)
  attributes/<group>.yaml       the declarations, one file per group: appearance, background,
                                psychology, capability, behaviour, lifestyle (at most 100 in all)
  masks/core.yaml               every built-in
  masks/appearance.yaml         the appearance group only, for casting-only use
entities/populations-us/        cited population profiles (S30-e; Q-S30-2)
  pack.yaml
  populations/san-diego-county.yaml
entities/attributes-matraix/    an optional backend pack, not installed by default (S30-h)
```

- **`entities/person-attributes`** is the default backend, and every reference world and test uses it. Its
  catalogue is curated: at most 100 attributes, each read by something the player notices, each with the
  reader that makes it noticeable (`ARC-122`, `ARC-126`). It declares 93 `core` attributes: step-30 §6.1's 94
  rows, plus `core.natal-sex` (step-30 Q-S30a-1), minus the two rows that are other packs' views —
  `employment.occupation` and `economy.wealth-band` — which their owners declare in their own namespaces.
- **`entities/populations-us`** holds profiles a world names in its configuration. A population is world
  content that the world, cognition, agents and casting read, so profiles live in a data pack the world
  requires, never in a Presentation Pack (`ARC-123`).
- **`entities/attributes-matraix`** is an optional person-generation backend. *Optional person-generation
  backend built on the MatrAIx persona schema (Li, Hao et al., 2026, arXiv 2608.04205; MIT), imported
  unchanged.* It is the only pack that names that schema, the workspace builds and passes every test without
  it, and its contents — the verbatim import, the overlay, the licence exclusions, its masks and mapping — are
  recorded in `DEP-66`.

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
DATA, decode_data_file, data_context
                 the data directories the pack owns and how their files become its context (ARC-121),
                 defined together by owns_data!(); default: owns none (below; implemented by S30-b)
decode_section_in
                 how the pack's section is decoded in its context (ARC-121), defined by
                 owns_section_in_context!(); default: decode_section, the context ignored (S30-b)
RESOLVES, resolve_subjects
                 whether the pack resolves across subjects before seeding (ARC-121), defined by
                 resolves!(); default: does not (S30-b)
SHIPS            data files the pack ships, compiled in, for an owner's directories (ARC-121 item 4),
                 declared by ships_data!(); default: none (S30-b)
```

*Corrected 2026-10-16 (S30-b, R-S30a-b-1; `pr-s30b-authoring-seam.md` D-b-6).* The block named the item
`RESOLVES, resolve`. The item is `resolve_subjects`: every pack is a `System`, whose `resolve` resolves an action,
and one name for both made `Self::resolve` ambiguous in a pack's own code. The trait a resolving pack implements,
`mineworld_authoring::ResolvesSubjects`, keeps its `resolve`. `owns_section_in_context!()` needs `owns_data!()`
beside it; written without it, the pack's section is refused at read, naming the system.

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

**A pack that owns data** ([`DECISIONS.md`](DECISIONS.md) `ARC-121`; specified by S30-a, implemented by S30-b,
`.structured-coding/plans/mvp0/pr-s30b-authoring-seam.md` §5). The authoring seam of `ARC-31` and `ARC-61`
gains four general capabilities. Each is additive: a pack that uses none of them, and a world that carries no
data directory, read, load and run exactly as before.

1. **A pack may own data directories** in World Packs and Entity Packs (`owns_data!()`). A directory name is one
   word of 1–32 bytes of `a`–`z`, `0`–`9` and `-`, unique among every pack this build provides and distinct
   from the format's own directories. The loader collects the `.yaml` files of the owner's directories in one
   deterministic order — files the enabled packs **ship** (item 4), in the world's `systems:` order; then each
   required Entity Pack, in id order; then the world itself; within each source, directories in the owner's
   declared order and files by name, byte order — and the owner decodes each file, a refusal keeping its line
   and column. The owner then builds one **context** from all its files and its own world configuration
   (`ARC-61`), once, checking what spans files (duplicates, namespaces). An owner with no files still gets a
   context, built from none. The loader never learns what a file means. Refused, naming the file: a directory
   no installed pack owns (`UnknownDirectory`, listing every directory this build knows, sorted); an installed
   owner's directory in a world that does not enable the owner (`DataNeedsASystem`); a subdirectory inside a
   data directory (`UnknownDirectory`); a file that does not decode (`Malformed`, with line and column); a
   context the owner refuses (`DataRefused`, naming the system and the file). Directories whose name begins
   with `.` are not content and are ignored; so is the directory holding the `decisions:` file.
2. **A section may be decoded in its owner's context** (`owns_section_in_context!()`). The owner's decoded data
   and its world configuration are available while its section is decoded, so a value is checked against a
   schema declared in data, at the value's own line and column. The Rust form is a sibling trait of
   `AuthoredSection`; a pack owns one section either way (`ARC-31`), and its seeding obligations are
   unchanged.
3. **An owner may resolve across subjects before seeding** (`resolves!()`). Once, after every section is
   decoded and every reference checked and before any genesis fact is seeded, the owner receives every file of
   the kinds its section is carried by — each subject's key, kind and tags, and its decoded section if the file
   carries one; never a display name, a `note` or a location — and returns at most one seed per subject. Each
   seed then becomes that subject's genesis facts, in the owner's own vocabulary, exactly where the owner's
   section facts would have been seeded: seeding order is unchanged (`ARC-31` item 4). Refused, naming the
   system: a resolution the owner refuses (`ResolveRefused`, naming the subject when there is one); a seed for
   a key that is not one of the subjects; a seeded fact outside the owner's vocabulary, as for a section.
4. **A pack may ship data files for an owner's directory** (`ships_data!("<directory>/<file>.yaml", …)`;
   step-30 Q-S30b-1, ruled 2026-10-15). The files are compiled into the shipping pack from its own crate
   directory and travel with its code and version. They are collected first (item 1) whenever the shipping
   pack is enabled, and every refusal about one names the shipping pack and the file's crate-relative path.

   *Clarified 2026-10-16 (S30-b, R-S30a-b-1; `pr-s30b-authoring-seam.md` F-b-1).* A shipped file is collected
   into its owner's context only when the world enables the owner too: a file shipped into a directory whose
   installed owner the world does not enable is not collected and is not refused, so a world that enables the
   shipping pack without the owner reads as before (how such a world reads the shipped declarations is S30-c's
   decision, `ARC-120`). A file shipped into a directory that **no installed** pack owns is a defect of the build
   and is refused (`UnknownDirectory`, naming the shipping pack and the path). A pack root's files (as opposed to
   its directories) are neither refused nor collected by item 1.

A data directory is a **general** capability of the seam: it names no domain, and the loader treats every
owner's directories alike.

**Attributes a pack reads, proposes or owns** ([`DECISIONS.md`](DECISIONS.md) `ARC-120`, `ARC-122`, `ARC-124`;
specified by S30-a, implemented by S30-c; the vocabulary is [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §4.5). A
pack that uses a Person's attributes declares it, by qualified id (§4.1 "Attributes"); it never looks an
attribute up by a string at run time. The exact Rust form is S30-c's; what is declared, and what is refused, is
this:

- **Reads.** A pack lists the attributes it reads (`reads_attributes: [<qualified id>, …]`). For each, it
  obtains a typed handle once, when it is installed, and reads only through that handle afterwards. Refused at
  install, by name: an attribute the world's catalogue does not hold (`Missing`), one of another kind than the
  handle's (`WrongKind { declared, requested }`), and a built-in that the world's backend neither provides nor
  maps (`MissingUnderBackend { attribute, backend }`), unless the read is declared optional. A handle to a
  `text` attribute cannot be obtained at all: no System reads prose.
- **Proposes.** A pack that changes a plain trait (one `attributes` owns) lists the attributes it may propose
  changes to (`proposes: [<qualified id>, …]`), declares `attributes`' vocabulary and depends on `attributes`
  (`ARC-26`), and emits `attribute-change-proposed { person, attribute, to, because }`. `attributes` accepts
  only an `evolving` attribute, a value in its domain and a proposer that listed the attribute, stating
  `trait-changed { person, attribute, from, to, because }`, or refuses by name.
- **Owns.** A pack that already models some state of a Person and exposes it as attributes is an **owner**
  (`AttributeOwner`, in `mineworld-attributes-model`, `ARC-124`). It depends on that model crate, not on the
  `attributes` System Pack. It:
  1. declares each attribute it owns in its own namespace, with `owner:` its own id, in an `attributes/` file
     it ships (item 4 above);
  2. projects each owned attribute from its own component when the attribute is read or disclosed — a view,
     never a copy, so nothing else stores it;
  3. receives, at genesis, each person's resolved initial value of each owned attribute (authored, locked,
     from a role, sampled or defaulted, §4.1 "Population profiles") as a typed value, and states it in its own
     vocabulary;
  4. discloses its attributes within its own component, through the model crate's visibility filter and
     renderer, so no owner writes its own visibility rules.

  The owners named so far: `economy` (`economy.wealth-band`, a `derived` projection of the person's wealth;
  S31, `ARC-114`); `employment` (`employment.occupation`, a projection of its own `Employment { job }` to the
  job's label; added by S30-f, the one named exception to step-30 §9.4's "no existing System is changed by
  S30", step-30 Q-S30a-2, ruled 2026-10-15); `notices` (`notices.subscriptions`; S31, `ARC-115`); and
  `persona` (`persona.summary`, `persona.traits`, `persona.speech`, `text` and `private`; P6a, `ARC-74`,
  reviewed after S30-f, step-30 Q-S30-4).
- **No owner, no attribute.** An attribute whose owner the world does not enable is refused at load, naming
  the attribute and the owner (`OwnerNotInstalled`).

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

**The field-level reference is generated** ([`DECISIONS.md`](DECISIONS.md) `ARC-90`). This subsection
and §4.2 state the shape of each file and the rules every file shares. Every field of every file — its
type, whether it is required, its default, its bounds and what it means — is in
[`docs/reference/`](reference/), which `mineworld reference` derives from the types that decode the files
(`world.yaml` and the content files' own fields, every section a pack owns, every `configure/` file, the
Interaction List sections, the decision list, a data pack's `pack.yaml`, and the command line). It is never edited by hand,
and CI fails when it differs from what the build generates. Where this specification and the generated
reference disagree about a field, the decoding type is the fact and the specification is corrected.

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
   selection of §5.2: `default:` and each `assign:` entry carry `policy`, optional `fallback`,
   optional `parameters` and optional `store_bound_mib` (the seat's memory store bound in MiB, §5.2 host
   rule 7); an entry names a `seat` or a `class` as rule 2 says. The decoder refuses, at
   load and with line and column, an unknown key, a `policy` or `fallback` not spelled as a policy id
   (`^[a-z][a-z0-9-]{0,47}$`), a seat or class rule 2 refuses, a duplicate entry, `parameters`
   that are not a mapping with string keys or that hold a tag or a non-finite number, and a
   `store_bound_mib` that is not an unsigned integer. It does not judge whether a policy exists or its
   parameters are valid, nor whether the bound is in range: policy ids are Python entry points, and the
   bound's floor and ceiling are the cognition host's. The bound travels with the winning entry; it is
   not merged across seat, class and default. Each seat's
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
appearance: { look: alice-moreau }   # a section owned by the `appearance` system (below)
routine:                   # a section owned by the `schedule` system (below)
  - { from: "05:30", place: cafe, label: work }
  - { from: "18:00", place: apartments, label: home }
persona:                   # a section owned by the `persona` system (below)
  summary: Runs the counter. Warm with regulars, dry with strangers.
attributes:                # a section owned by the `attributes` system (below, "Attributes"; S30-c)
  hair-colour: auburn      #   a category, by short slug (unique among the world's catalogues)
  core.stature: 1.660      #   a quantity, in its declared unit and scale (m, 3 decimals)
  subscriptions: [news.daily]           #   a set over a vocabulary (owned by `notices`)
  core.mass: { between: [55.0, 70.0] }  #   an allowed: restriction on a quantity; the sampler draws inside it
  chronotype: { allowed: [early-bird, morning-leaning] }   # the same on a level
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
or the pack is refused. MVP-0 has nine sections:

```text
name      naming     people   a display name: 1–64 bytes, no control characters, no surrounding
                              whitespace. Public: disclosed to whoever perceives the person
persona   persona    people   { summary: <1–280 bytes, required, non-empty>, traits: [ <≤ 32 bytes
                              each> ] (at most 8), speech: <≤ 160 bytes> }, traits and speech
                              optional; unknown keys refused, with line and column. Disclosed to the
                              person themself only. It shapes how the person speaks and grants
                              nothing: no action, no process and no rule reads it (`ARC-74`)
appearance appearance people  { look: <key> }: 1–48 bytes of a–z, 0–9 and '-'; unknown keys refused.
                              An opaque key naming the person's appearance, which a Presentation Pack
                              resolves (`ARC-92`); never a mesh, file, colour or height. Public:
                              disclosed to whoever perceives the person (`ARC-91`)
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

An eighth section, `attributes` (owned by the `attributes` System Pack, carried by people), is specified below
("Attributes") before it is implemented (S30-c); until then a person file carrying it is refused as an unknown
key (rule 6).

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
MVP-0"); one whose `items/` is absent or holds no `.yaml` file ("declares nothing").

*Amended 2026-10-15 (S30-a, `ARC-121`; implemented by S30-b).* An Entity Pack may carry data directories
(§2, "Data directories"), so a pack need not have `items/`: it "declares nothing" only when it has no
`items/*.yaml` **and** no `.yaml` file in any other directory whose name does not begin with `.`. The three
forbidden directories stay forbidden. Which other directories are known is decided where the installed packs
are known — by the loader when a world requires the pack, and by `mineworld packs validate` against the
build's installed packs — never by the package check. A required Entity Pack's data files are collected with
the world's, in the order of §3.1 "A pack that owns data", item 1. Refused when its
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

**Attributes** ([`DECISIONS.md`](DECISIONS.md) `ARC-120` … `ARC-126`, `DEP-66` … `DEP-68`; specified by S30-a;
implemented by S30-b … S30-k; the concept is [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §4.5). A Person's
attributes are written as data in five places, all owned by the `attributes` System Pack unless a row says
otherwise:

| Where | What | Owner |
| --- | --- | --- |
| `attributes/*.yaml` of an Entity Pack, a World Pack, or shipped by a System Pack (§3.1 item 4) | declarations and overrides | `attributes` (the directory); each declaration names its attribute's owner |
| `vocabularies/*.yaml` (same sources) | value vocabularies | `attributes` (the directory); each vocabulary's namespace is its declaring pack's |
| `populations/*.yaml`, `masks/*.yaml`, `mappings/*.yaml`, `labels/*.yaml`, and a backend pack's `backend.yaml` | profiles, masks, a backend's mappings, locale labels, the backend contract | `attributes` |
| `configure/attributes.yaml` of a World Pack, and the lock file it names | the world's choice of backend, mask, profile and population seed; pinned values | `attributes` (its world configuration, `ARC-61`) |
| a person file's `attributes:` section | the values this person's biography states | `attributes` (the section) |

None of them is a key of `world.yaml`. `world.yaml`'s `population:` already means the list of people files, so
no attribute setting is ever a key named `population`.

*Attribute declarations* (`ARC-122`). One file holds any number of declarations:

```yaml
# entities/person-attributes/attributes/appearance.yaml (excerpt)
format: 1
attributes:
  stature:
    group: appearance
    facets: [appearance, body]           # open tags consumers select by; S26 casting reads `appearance`
    kind: { quantity: { unit: m, scale: 3, min: 0.500, max: 2.500 } }   # stored as integer millimetres
    visibility: perceivable
    mutability: fixed
    label: Stature
    phrase: "{value} m tall"             # `{value}` is the only placeholder a phrase may name
    perceived_as: { bands: [ { below: 1.55, words: short }, { below: 1.85, words: of average height },
                             { words: tall } ] }   # what another observer is told; the holder sees the value
    since: 1.0.0
  hair-colour:
    group: appearance
    facets: [appearance]
    kind: { category: [black, dark-brown, brown, light-brown, blond, auburn, red, grey, silver-grey, white] }
    visibility: perceivable
    mutability: evolving
    label: Hair colour
    phrase: "{value} hair"
    words: { dark-brown: dark brown, light-brown: light brown, silver-grey: silver-grey }
    since: 1.0.0
  chronotype:
    group: behaviour
    kind: { level: [early-bird, morning-leaning, neutral, evening-leaning, night-owl] }
    visibility: learnable
    mutability: evolving
    label: Chronotype
    phrase: "{value}"
    since: 1.0.0
```

| Field | Required | Meaning |
| --- | --- | --- |
| the key | yes | the slug, `^[a-z][a-z0-9_.-]{0,63}$` (`_` admitted so that an imported schema's ids are kept verbatim); qualified by the declaring pack's namespace (below) |
| `group` | yes | one of the six built-in groups — `appearance`, `background`, `psychology`, `capability`, `behaviour`, `lifestyle` — or a group the declaring pack declares |
| `facets` | no | open tags for selection by consumers (`appearance`, `body`, `prompt`, `casting` …) |
| `kind` | yes | the value kind (below) |
| `visibility` | yes | `perceivable`, `learnable`, `private` or `hidden` ([`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §4.5) |
| `mutability` | yes | `fixed`, `evolving` or `derived` (§4.5 there); a `derived` attribute also declares `derive:` (below) |
| `owner` | no | the owning System Pack's id; default `attributes`. An owner pack's declarations use its own namespace and name itself |
| `label` | yes | the attribute's name in prose |
| `phrase`, `words` | no | the rendering (below) |
| `perceived_as` | no | what an observer other than the holder is told, when it is coarser than the value |
| `default` | no | the value when nothing else states one (precedence below). Absent: the attribute stays absent |
| `since` | built-ins | the catalogue version that introduced the attribute |
| `prompt_rank` | no | the order in which a byte-budgeted rendering keeps attributes (cognition, S30-f) |

| Kind | Declared as | Stored as |
| --- | --- | --- |
| `category` | a closed list of value slugs, or `vocabulary: <qualified id>` | the value's slug |
| `level` | an ordered list of value slugs; order is meaningful (comparison, bands) | the value's slug |
| `quantity` | `unit` (one of `m`, `kg`, `year`, `day`, `hour`, `count`, `ratio`), `scale` (decimal places), `min`, `max` | an integer at that scale (`m` at scale 3 is millimetres); no float is stored or compared |
| `flag` | — | `true` or `false` |
| `set` | `of:` a closed list, or `vocabulary: <qualified id>`; `max` (members) | a sorted set of slugs |
| `text` | `max_bytes`, at most 280 | bounded UTF-8 with no control characters; never readable by a System |
| `instant` | — | a world instant in seconds, possibly before the epoch |

- **Units are a closed list.** Currency is not one: money is `economy`'s. A new unit is a change to the model
  crate, reviewed as such.
- **Absent is not a value.** No domain contains `unknown`. An attribute nothing states is absent: rendering
  omits it, and a System's handle reads `None`.
- **Derived attributes** declare `derive:` from a closed list: `age { from: <instant attribute> }` (whole years,
  floor), `band { from: <quantity or level attribute>, bands: [...] }`, and `projection` (the owner's code,
  §3.1 "Owns"). A derived value is computed at disclosure and on a read, from stored state and the instant; it
  is never stored and never stated as a fact.
- **Versioning.** A catalogue is a versioned data pack ([`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md)), required by
  range (`requires: { person-attributes: "^1" }`). A minor version may add an attribute, append a value to a
  `category` or `set` list, or add words, phrases or facets. Removing or renaming an attribute, removing or
  reordering values, or changing a kind, a unit or a scale is a major version. An id is never reused for another
  meaning. Saves hold slugs, never positions, so appending a value moves no fact.

*Namespaces and short slugs* (`ARC-122`). Every attribute's qualified id is `<namespace>.<slug>`, and the
namespace is the declaring pack's id: `core` is reserved for `entities/person-attributes`; a World Pack's own
declarations use the world's id; an owner System Pack's use its pack id (`notices.subscriptions`,
`economy.wealth-band`). A declaration in any other namespace is refused (`ForeignNamespace`). An author may
write the **short slug** in a person file, a profile, a lock entry or a mask wherever it is unique among the
world's catalogues; an ambiguous short slug is refused, listing the qualified candidates (`AmbiguousSlug`). The
qualified id is always accepted.

*Overrides* (`ARC-122`). A world changes a declaration it did not author only through an explicit entry:

```yaml
# worlds/harbour/attributes/overrides.yaml
format: 1
overrides:
  core.hair-colour:
    words: { auburn: copper }          # allowed: presentation
    visibility: private                # allowed: narrowing only
```

| May override | May not override (refused, `OverrideNotAllowed`, naming the field) |
| --- | --- |
| `label`, `phrase`, `words`, the words of `perceived_as`, `facets` (add only), `default`, `visibility` narrowed along `perceivable` → `learnable` → `private` → `hidden` | `kind`, the values and their order, a `vocabulary`, `unit`, `scale`, `owner`, `mutability`, a widened `visibility` |

Declaring an id that already exists, instead of overriding it, is refused (`DuplicateAttribute`). A world that
wants another value set declares its own attribute (`harbour.ancestry`) and its profile uses that one.

*Vocabularies* (`ARC-122`, `ARC-124`). A `category` or `set` attribute may take its values from a vocabulary: a
closed set of value ids named `<namespace>.<name>`, declared by its owner either as a `vocabularies/` file —

```yaml
# vocabularies/topics.yaml, shipped by the `notices` System Pack (step-31 §8.5)
format: 1
vocabularies:
  topics: [prices.ingredients, prices.fuel, station.departures, news.daily, weather.warnings]
```

— or from its own world configuration, through its owner interface (`economy.wealth-bands`, the band ids of
`configure/economy.yaml`, step-31 §7.4). A value outside the vocabulary is refused where it is written
(`UnknownTopic`, naming the vocabulary and the nearest ids).

*Rendering* (`ARC-122`). `label` names the attribute; `phrase` is a template with exactly one placeholder,
`{value}`; `words` gives per-value words, by default the slug with hyphens turned into spaces. A `quantity`
renders at its scale with its unit, a `set` as a list joined with "and", a `perceived_as` band as its words.
Rendering is deterministic and bounded, and English by default; locale label packs (`labels/<locale>.yaml`,
keyed by attribute and value, S30-k) replace label, phrase and words, falling back to English.

*The person file's section* (`ARC-122`, `ARC-123`). `attributes:` in `people/<key>.yaml` maps attribute ids
(qualified, or short where unique) to either a **value** — written in the kind's own form: a slug, a number in
the declared unit, `true`/`false`, a list of slugs, a string, or an instant — or a **restriction** the sampler
must respect: `{ allowed: [<value>, …] }` for a `category`, `level` or `set`, `{ between: [<min>, <max>] }` for
a `quantity`. Each entry is decoded against its declaration in the owner's context (§3.1 seam item 2) and
refused at its own line and column: an unknown id (`UnknownAttribute`, with the nearest declared ids), a value
of the wrong kind (`WrongKind`, printing the domain), a value outside a closed list or vocabulary
(`OutOfDomain`, or `UnknownTopic` for a vocabulary), a quantity outside `min`/`max` (`OutOfRange`), an
attribute the world's mask disables (`MaskedOut { attribute, mask }`), a `derived` attribute (`OutOfDomain`:
nothing states a derived value). A value written here is authored: the sampler never re-draws it.

*The world's configuration* (`ARC-123`, `ARC-126`). `configure/attributes.yaml`, listed under `configure:` like
any pack's (`ARC-61`):

```yaml
format: 1
backend: mineworld                 # default; or the id of any installed backend pack
mask: core                         # optional; a mask the backend ships, or one of the world's own masks/
profile: san-diego-county          # resolved through the world's required packs; absent: no sampling
seed: mineworld-s26-cast-v1        # the population seed: world content, never the run's --seed
salt: 0                            # optional; the distinctness re-draw of S26 casting, once written here
lock: attributes.lock.yaml         # optional; the lock file, relative to the pack root
```

- **The population seed is not the run seed.** The same world gives the same people under every
  `mineworld run --seed`, so casting and the world agree. Several worlds may name one profile.
- **No profile, no sampling.** A world whose configuration names no profile takes every value from its people
  files, its lock and declaration defaults; tools report `no profile`.
- **`salt`** changes `sampled` values only. A world resolved by a tool with a non-zero salt is not the world's
  own population until that salt is written here.
- A profile the world's required packs do not provide is refused naming the profile (`UnknownProfile`).

*Population profiles* (`ARC-123`, `DEP-68`). A profile is a `populations/` file, versioned with its pack:

```yaml
# entities/populations-us/populations/san-diego-county.yaml (excerpt)
format: 1
profile: san-diego-county
version: 3
covers: adults                         # who the profile describes
sources:
  acs-2024-1y:
    cite: "American Community Survey 2024 1-year, B01001, B03002, San Diego County (05000US06073)"
    url: https://data.census.gov/table/ACSDT1Y2024.B03002?g=050XX00US06073
    read: 2026-10-15
    status: read                       # read | verify | authored
  nhanes-2015-2018:
    cite: "Fryar et al. 2021, Vital Health Stat 3(46), Tables 9, 11, 14, 15"
    status: read
  mst-convention:
    cite: "MineWorld casting convention (operator-gated, Q-S26-7, ARC-95)"
    status: authored                   # an authored convention, never presented as data
distributions:
  core.natal-sex:
    prior: { female: 0.4986, male: 0.5014 }
    source: acs-2024-1y
    stratify: true
  core.age-band:                       # the band is sampled; birth follows
    prior: { 20s: 0.1864, 30s: 0.2050, 40s: 0.1716, 50s: 0.1506, 60s: 0.1417, 70s: 0.0973, 80-plus: 0.0474 }
    source: acs-2024-1y
    stratify: true
  core.birth:
    given: [core.age-band]
    uniform_age_within_band: true      # birth = epoch − a uniform age inside the band, in whole days
  core.stature:
    given: [core.natal-sex, core.age-band]
    normal: { by: { female.20s: { mean: 1.626, p5: 1.510, p95: 1.728 }, … } }
    clip_sd: 2
    source: nhanes-2015-2018
  core.build:
    given: [core.natal-sex]
    prior: { female: { slim: 0.144, average: 0.167, solid: 0.262, heavy: 0.427 }, male: { … } }
    source: nhanes-2015-2018
    verify: "county figure not read; national NHANES over-represents `heavy` here"
  core.hair-colour:
    given: [core.ancestry, core.age-band]
    prior: { … }
    adjust:                            # likelihood ratios r(v; parents)
      - { when: { core.age-band: [60s] }, ratios: { grey: 4.0, silver-grey: 2.0 } }
    source: mst-convention
  core.facial-hair:
    given: [core.natal-sex]
    prior: { none: 0.55, stubble: 0.15, short-beard: 0.20, moustache: 0.10 }
    mask:                              # hard compatibility m(v; parents)
      - { when: { core.natal-sex: [female] }, only: [none] }
    source: authored                   # the reserved source: an authored convention, needing no entry
roles:                                 # defaults by authored role: tags, or any resolved attribute
  - { when: { tags: [cook] }, add: { notices.subscriptions: [prices.ingredients] } }
```

- **Semantics.** For an attribute X with parents Pa (its `given`), a person's weight for value v is
  `prior(v | pa) × Π (adjust ratios matching pa) × mask(v; pa)`. Weights are normalised to integer parts per
  million, rounded by largest remainder so that each row sums to exactly 1 000 000. Only a mask zeroes a value;
  a rare valid combination stays eligible. Numeric distributions (`normal` by stratum, `uniform`, `table`
  quantiles) use a committed table of standard-normal quantiles in micro-units and integer interpolation — no
  `ln`, `exp` or platform `libm` — so the result is identical on Linux, macOS and Windows.
- **The algorithm.** Attributes are resolved in topological order of `given`, ties broken by qualified id:
  1. values the person's file states and the lock pins are fixed first and never drawn;
  2. for each attribute, the covered people are grouped by their resolved parent values; within a group of
     n people, each value's target is n × weight; the fixed members already holding a value are subtracted
     (floor 0); the remaining unfixed members are allocated by **largest remainder**, remainder ties broken by
     the counter-based mix of (seed, attribute, group, value); the allocated values are assigned to the unfixed
     members in the order of mix(seed, attribute, person key), respecting each member's `allowed:` or
     `between:` restriction by a bounded matching — when no matching exists the resolution is refused
     (`Unsatisfiable { person, attribute }`), never silently re-drawn;
  3. with `stratify: false`, each unfixed member draws independently with mix(seed, attribute, person key);
  4. a numeric attribute draws one quantile per member — stratified `(i + ½) / n` points permuted by mix when
     `stratify`, else one draw — mapped through the stratum's distribution and clipped at `clip_sd`;
  5. `roles:` apply after sampling: each adds to a `set` or fixes a value, only for members whose tags or
     resolved attributes match.
- **Nothing reads a name.** The sampler's input for a person is their key (an opaque hash input), tags,
  authored values and lock entries (the seam's subject, §3.1 item 3): never a display name, a `note` or a
  `persona`. Renaming every person changes no resolved value.
- **Origins.** Each resolved value has an origin — `authored`, `ruled`, `locked`, `role`, `sampled { stratum,
  draw }` or `default` — reported by the tools and never saved (§3 "Metadata" of `CORE_CONCEPTS.md`).
- **Guarantee.** Within each parent group, every value's realised count differs from its target by less than
  one person; for 1 000 covered people a synthetic resolution matches every declared marginal within 0.5 pp
  and every conditional within 2 pp for groups of at least 50.
- **`verify` marks** are reported by the tools and on S26's casting page and do not fail a load. A release
  (S23) refuses a reference world whose profile marks a **data** figure `verify`; a source of status
  `authored` is a convention, always shown as authored, and never refused for it (step-30 Q-S30-5).
- **`source: authored`** is reserved: it marks a distribution as an authored convention without a `sources:`
  entry (as a world's own small profile does), and it is reported exactly as a source of status `authored`.
- **Data and conventions stay apart.** Provider limits ("no loose long hair for this route") are S26's
  declarations on the provider, never profile content.

*The lock file* (`ARC-123`). Machine-written, human-reviewed, it pins values per person with their origin:

```yaml
# attributes.lock.yaml
format: 1
profile: { id: san-diego-county, version: 3 }
seed: mineworld-s26-cast-v1
people:
  alice:
    core.stature:   { value: 1.660, origin: ruled, at: 2026-10-15, by: "casting gate B0 (Q-S26-7)" }
    core.skin-tone: { value: 4,     origin: ruled, at: 2026-10-15 }
```

- **Precedence**, for every attribute of every person: the person's own `attributes:` section, then the lock,
  then `roles:`, then the profile's sample, then the declaration's `default`; otherwise absent.
- **A casting ruling binds the world through the lock.** When the operator rules a person at a casting gate, the
  casting tool writes the ruled values here, so the world's attributes and the look's brief never diverge, even
  when a later profile version or seed changes every unruled draw. The tool writes the file; people never edit
  positions by hand.
- Each entry is checked as a person-file value is (`UnknownAttribute`, `WrongKind`, `OutOfDomain`,
  `OutOfRange`, `MaskedOut`), and a person key the world does not declare is refused naming it.

*Backends and masks* (`ARC-126`). A **backend** is a data pack that supplies a world's catalogue and the means of
generating people over it; the world names it in `backend:`. Its contract is `backend.yaml` at the pack root,
every field validated and every failure named:

| Field | Meaning | Refusal |
| --- | --- | --- |
| `id`, `version`, `license`, `attribution` | identity and credit, shown by the tools and in a resolution's provenance | `BackendMalformed { pack, field }` |
| `catalogues` | the attribute files it supplies, in the declaration format above | as for any declaration (`UnknownAttribute`, `DuplicateAttribute` …) |
| `masks` | the masks it ships, and its default mask | `UnknownMask { backend, mask }`; `MaskNamesUnknownAttribute { mask, id }` |
| `sampling` | the modes it supports, from the engine's closed list: `profile` (§ "Population profiles") and, optionally, `log-linear` (a backend-supplied factor model executed by MineWorld's engine in fixed point) | `UnsupportedSampling { backend, mode }` |
| `provenance` | upstream source, version, commit and file digests | `ProvenanceMismatch { file, expected, found }` (CI re-verifies) |
| `mapping` | how the built-in ids that Systems and consumers read are provided under this backend: a `derived` view per built-in, value by value, in `mappings/` | `MissingUnderBackend { attribute, backend }`, at the reader's install |

- **No backend runs code.** A backend is data, executed by the model crate's engine; a new sampling mode is a
  model-crate change, reviewed as such.
- **The default backend** is `mineworld` (`entities/person-attributes`), with the `profile` mode and the masks
  `core` (every built-in) and `appearance` (the appearance group). Every reference world and test uses it.
- **Appearance and quantities are always MineWorld's.** Every backend's catalogue is composed with the `core`
  appearance group and `core` quantity attributes, because a body in a client must come from one place
  (`ARC-125`); a backend may not redeclare them.
- **A mapping is checked:** every target value exists and every source value is covered. Under a backend, a
  reader's handle to a mapped built-in reads the mapped view; a built-in neither mapped nor in the appearance
  group is absent.

A **mask** is a named, versioned selection over a backend's catalogue:

```yaml
# masks/<id>.yaml
format: 1
mask: core-lite
version: 1
include:
  - { id: core.chronotype, why: "DM: when they rise and go home" }
  - { group: psychology, why: "DM, CV: the five traits and the core value" }
exclude:
  - { id: core.honesty, why: "this world's LM characters never shade the truth" }
```

- `include` and `exclude` take ids, groups or facets, and every entry carries a `why` naming the reader it
  serves; an entry without one is refused (`MaskEntryUnjustified`). The mask `all` includes everything.
- **What the mask does not enable does not exist in the world:** it is absent from the frozen catalogue, the
  sampler skips it, no observation, prompt, agent view or panel carries it, a person file stating it is refused
  (`MaskedOut`), and a System's handle to it fails at install.
- Owner packs' own attributes are never masked: they belong to Systems the world already chose.
- A world writes its own mask with no code; a missing or malformed mask is refused by name.

*Disclosure* (`ARC-122`). `attributes` discloses one `traits` component per perceived person, filtered for the
observer: the holder receives every non-hidden value; another observer receives the `perceivable` ones,
through `perceived_as`. Each entry is `{ attribute: <qualified id>, value: <typed by kind>, text: <rendered
phrase> }`, schema 1, in catalogue order; `text` is world text. An owner discloses its own attributes in its own
component by the same rule, through the model crate's filter and renderer.

*Persistence* (`ARC-120`, `ARC-123`):

| Fact | Owner | When | Payload | Audience |
| --- | --- | --- | --- | --- |
| `traits-stated` | `attributes` | genesis, one per person with at least one plain trait | the sorted `(qualified id, value)` pairs; no origin | the holder only (it holds private values); others learn by disclosure, per observer, at observation time |
| `trait-changed` | `attributes` | an accepted proposal | `{ attribute, from, to, because }` | the holder, and the observers present if the attribute is `perceivable` |
| `attribute-change-proposed` | `attributes` (its vocabulary) | emitted by a pack that declared the attribute in `proposes:` | `{ person, attribute, to, because }` | none: an internal request |
| an owner's own facts | that owner | its own rules | its own vocabulary | its own rules |

- **Derived attributes write nothing.** Origins and sources are not saved: the save holds world truth, and
  provenance is a tool's output. Replay reduces `traits-stated` and `trait-changed` exactly; the sampler runs
  only at genesis, so a resumed world never re-samples.
- **A world without `attributes`** and without owner-declared attributes is byte-identical to the same world
  before attributes existed. Adopting attributes in a reference world moves its digests once, under one
  recorded re-baseline whose reason names the moved columns (S30-g).
- **No migration layer.** Resuming a save made under a composition without `attributes` under one that adds it
  is a composition change, refused by name as any added pack is.

*Refusals*, in one place. Each is raised at load (or at a reader's install, where marked) and names what the
*Names* column lists; the source is the step-30 section that states the rule.

| Refusal | When | Names | Step-30 |
| --- | --- | --- | --- |
| `BadDeclaration` | a declaration breaks a field rule: an id outside the slug pattern, an unknown group, kind or unit, a `text` over 280 bytes, a phrase naming a placeholder other than `{value}`, a `derived` attribute without `derive:` | the file, line, attribute and field | §§5.1, 5.2, 5.5, 5.6 (the rules, given one name) |
| `UnknownAttribute` | a person file, profile, lock, mask, role or override names an id no catalogue declares | the id and the nearest declared ids | §§5.3, 8.2 |
| `DuplicateAttribute` | two declarations of one qualified id, or a redeclaration instead of an override | the id and both files | §§6.4, 7.2 |
| `ForeignNamespace` | a declaration in a namespace other than its pack's | the id and the pack | §7.1 |
| `AmbiguousSlug` | a short slug two catalogues declare | the slug and the qualified candidates | §7.1 ("refused by name and the message lists the qualified candidates") |
| `WrongKind` | a value of another kind than its declaration's; at install, a handle of another kind | the attribute, the value and the domain; at install `{ declared, requested }` | §5.3 |
| `OutOfDomain` | a value outside a closed list, or a stated value for a `derived` attribute | the attribute, the value and the domain | §5.3 ("an out-of-domain value") |
| `OutOfRange` | a quantity outside `min`/`max` | the attribute, the value and the bounds | §5.3 ("a value outside `min`/`max`") |
| `UnknownTopic` | a value outside a `vocabulary:` | the vocabulary, the value and the nearest ids | §9.6 N-1 ("unknown topics refused by name at load") |
| `OverrideNotAllowed` | an override of a field the table forbids, or a widened visibility | the attribute and the field | §7.2 ("refused by name") |
| `OwnerNotInstalled` | an attribute whose owner the world does not enable | the attribute and the owner | §4.2 item 5 ("refused at load by name") |
| `UnknownProfile` | `profile:` names no profile the world's required packs provide | the profile | §8.1 ("resolved through the world's required packs") |
| `NotCovered` | a person outside the profile's `covers:` | the profile and the person | §8.2 ("people outside it are refused by name") |
| `DependencyCycle` | the profile's `given` lists form a cycle | the profile and the cycle `[a, b, …, a]` | §8.2 |
| `MissingCondition` | a `prior` misses a parent combination that can occur and states no `otherwise:` | the attribute and the parents | §8.2 |
| `BadDistribution` | a probability below 0 or with more than 6 decimals, or a row not summing to 1 ± 1e-6 | the attribute, the row and its sum | §8.2 |
| `BadAdjustment` | an `adjust` ratio that is not > 0 | the attribute and the entry | §8.2 |
| `BadMask` | a profile `mask` naming a value outside the attribute's domain | the attribute and the value | §8.2 |
| `EmptySupport` | a profile mask leaving no value for a parent combination that can occur | the attribute and the parents | §8.2 |
| `UncitedDistribution` | a distribution with no `source`, or naming one the file does not declare (the reserved source `authored` needs no entry) | the attribute | §8.2 |
| `MissingSupport` | a `quantity` distribution that does not declare the support it can return | the attribute | §8.2 |
| `Unsatisfiable` | no assignment respects the people's `allowed:` / `between:` restrictions | the person and the attribute | §8.5 |
| `BackendMalformed` | a `backend.yaml` field missing or invalid | the pack and the field | §6.4 |
| `UnknownMask` | a mask the backend and the world do not provide | the backend and the mask | §6.4 |
| `MaskNamesUnknownAttribute` | a mask entry naming an id the backend does not declare | the mask and the id | §6.4 |
| `MaskEntryUnjustified` | a mask entry without `why` | the mask and the entry | §6.5 |
| `MaskedOut` | a person file or lock states an attribute the mask disables | the attribute and the mask | §6.5 |
| `UnsupportedSampling` | a mode the backend does not support | the backend and the mode | §6.4 |
| `ProvenanceMismatch` | a backend file whose digest differs from its provenance | the file, the expected and the found digest | §6.4 |
| `MissingUnderBackend` | at a reader's install, a required built-in the backend neither provides nor maps | the attribute and the backend | §6.6 |
| `Missing` | at a reader's install, an attribute the world's catalogue does not hold | the attribute | §5.3 |

The commands that report all of this — `mineworld attributes catalogue | show | check | resolve | lock |
backends` — are added to §8.1 by the PRs that implement them (S30-c, S30-d, S30-e).

## 4.2 The World's Interaction List

A world says what its installed interactions may do — who may do what to whom, with which numbers, and
what a fact means for history — through one document shared by every pack (`DECISIONS.md` `ARC-63`,
`ARC-64`, `ARC-65`, `DEP-28`). The list is **not** a new carrier: it is `configure/classes.yaml` plus
each pack's section, `configure/<pack id>.yaml`, all carried by §4.1's configuration seam.

This subsection states the list's shape and its rules. Which actions, roles, facts, parameters (with
their defaults and bounds) and knobs each installed pack declares is generated from the packs' own
declarations into [`docs/reference/interaction-list.md`](reference/interaction-list.md) (`ARC-90`).

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
  as `mineworld_decision::bounded`, and `()` refuses every key. `ModelParameters` requires
  `schemars::JsonSchema` (a `parameters!` block has it), and the catalog `models!` writes answers each
  in-process model's parameter schema, from which `docs/reference/decisions.md` is generated (`ARC-90`).
  The signatures carry no clock, no network
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

- **LM Controller (S10 P6b; `ARC-72`, `ARC-73`).** The `lm` model is the one out-of-process model this build
  installs (`external "lm"`). The LM Controller is a Python Controller Pack and a protocol client (`ARC-56`):
  it joins each seat that `mineworld decisions --json` assigns to `lm`, and it is the only process that
  holds such a seat while it runs. Its social choices (what to say, to whom, which offered affordance to
  attempt) come from a language model through the P5 gateway; a model's text is a proposal that the
  controller validates, and a request is built only through `mineworld_sdk.offers`. It never chooses
  movement. When it cannot decide (a failed consult, an exhausted retry layer, a breached bound, an open
  breaker), it gives the seat back with `leave`, and the server's fallback model for that seat drives it
  (`took_over: "hosted"`). The hand-back is host state: no fact is written and no world state changes.
  Retry layers (transport, content, semantic), the per-seat circuit breaker, the consult deadline and the
  call cap are configured in `cognition.toml`, never in a decision list. Counters go to the controller's
  own decision log, never to world facts. Where no in-server fallback exists, the seat stands for that
  consult and sends nothing.

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

**The bundled no-LLM model, `motivated`** (`cognition/motivated`, DL-b, `DECISIONS.md` `DEP-47`, `DEP-48`).
- **What it is.** A pure, stateless, integer utility model. It scores every option the observation offers by
  integer response curves, combines them through Mark's compensation factor, and chooses by a seeded draw
  among the top band.
- **Its daily rhythm** comes from the curve set `atus-a3-2025`. That set is derived from BLS American Time Use
  Survey Table A-3 (2025), which is public domain. Source: U.S. Bureau of Labor Statistics, American Time Use
  Survey. The credit is in `cognition/motivated/data/NOTICE`.
- **Parameters.** `curves`, `top_band`, `commit_minutes`, `social_need_hours`, `reply_floor`, `base`, `roles`
  and per-curve `curve` overrides (≤ 24 points each).
- **Status.** It is installed but is never a default. A world selects it through a decision list, such as
  the opt-in `cognition/motivated/examples/social-cafe-all-motivated.yaml` for Social Café. Its rhythm acceptance AC-DLb-4 is recorded as FAIL
  (F-DLb-5, `pr-dl-b-motivated.md` §15).

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
  host-computed `importance`; its `Citation`; its names-only text; and its own size in the seat view, the
  bytes it contributes to the canonical dump of that seat view with any shared record it cites counted in
  full (`ARC-99`);
- a `RetentionBudget`: the bytes the surviving items should fit in, `now` (world seconds at the start of
  the day whose first fact triggered the pass, or, for a size pass, the world time of the fact that took
  the store over its trigger; rule 1), and the model calls this pass may make (0 for every no-LLM policy).

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
   never forgets. A pass also runs **early, when the store nears its bound** (a size pass): `absorb`
   stops each ingest call after the fact that takes the store's content (its canonical dump without
   `meta`, measured exactly after every fact) over the trigger of rule 7, and runs a pass there before
   ingesting the next fact. The measure after a fact is a function of the facts up to it, so size passes
   too fall on the same facts however the batches were cut. At a day's first fact only the daily pass
   runs, so a fact triggers at most one pass; a size pass does not move the day of the last daily pass,
   and it never calls a model: an assisted policy's size passes are decided by its fallback alone and
   recorded `fallback:size`. A
   stop whose pass did not commit is repaired like a due daily pass: the next `absorb` runs it first.
2. **Eligibility.** Every L0 record of a closed episode, every closed episode, every chapter and every
   stable fact. The open episode and its records are never shown and never removed.
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
   episode's records) or by the forgotten set; nested units lie inside their container; no L0, L1 or L2
   citation intersects the forgotten set (AC-FG-3). A stable fact's citation may point into the forgotten
   set: what established it still resolves in the save. The host checks the store-side half of this after
   every pass and rolls the pass back on a breach (a defect, never a fallback).
6. **Forgotten set.** Sorted, disjoint ranges of event ids over the seat's perceived sequence. Two ranges
   with no surviving unit between them merge. Beyond 256 ranges the oldest are removed and the watermark
   rises to their end, meaning "every perceived id at or below the watermark that no surviving unit
   represents is forgotten". The set is bounded by construction, and every forgotten id still resolves in
   the save (`mineworld perceived`).
7. **Hard bounds.** Each person's size has a **bound**: 20 MiB (20 971 520 B) by default, which a world may
   raise per seat or class with `store_bound_mib` in its `memory:` entry, up to 1 024 MiB, and never set
   lower (operator rulings of 2026-10-14). From it the host derives a **target**, 75 % of the bound
   (15 728 640 B at the default), and a **trigger**, the bound less a sixteenth (19 660 800 B). After
   executing a plan, while the canonical dump exceeds the target less 512 B kept for the pass's own log
   row, the host applies, in order and until it is under: `consolidate` with its defaults; merging every
   remaining L0 record of a closed episode into its episode, oldest first; merging the episodes of weeks
   with a chapter into it, oldest first; dropping chapters, oldest first; dropping chapterless closed
   episodes, oldest first; dropping stable facts, fewest meetings first. Every such removal is counted as
   forced in the pass log. The post-pass dump is therefore at most the target. Between passes the size
   trigger (rule 1) holds every state `absorb` commits under the bound: the content after every fact is at
   most the trigger, except after the one fact that crossed it, whose pass follows at once; the sixteenth
   between trigger and bound holds that fact's growth and `meta`. A single fact that grows the store by more
   than that margin is committed over the bound until its pass, which the pass log's `bytes_before` shows.
   If the store is still over the target after the last step — the open episode alone, which no pass
   removes, being too large — the pass raises `RetentionBoundUnreachable` and rolls back, and the next
   `absorb` retries it before ingesting anything, so the store does not grow past that point. No policy
   sees or changes the bound, the target or the trigger. A person's size is the canonical dump of their seat
   view, with every shared record they hold counted in full (`ARC-99`, `ARCHITECTURE.md` §9.3); each
   "canonical dump" in this rule means that dump, and the physical share of a shared record is never charged.
8. **Context bound.** A retrieval for a model call is at most 6 000 bytes, per section, as P4 D-P4-9 defines
   it. This bound is not changed by any policy, nor by the store's bound. Retrieval reads no row a policy
   may have removed.
9. **Pass log.** Each pass records the event key of its triggering fact (the newest ingested, the log's
   key), its day, its cause (`day` or `size`), its policy, outcome, the counts kept, merged, dropped and
   forced, the dump size before and after, and the number of forgotten ranges; the newest 32 rows only.

The cognition budget ledger is not memory and is not pruned by a pass: it prunes itself, by wall time,
when it records a charge, keeping two wall days, twice its longest window (ARC-57, `ARCHITECTURE.md` §9.1).

**Selection.** A world selects a policy per seat through `memory:` in `decisions.yaml` (§4.1). The Rust
reader resolves seat > class > default and emits, per seat, a typed JSON `MemoryPolicyChoice` (`policy`,
optional `fallback`, `parameters`, optional `store_bound_mib`, and the source span of the winning entry).
The cognition process resolves it when it starts: an unknown policy is refused with the installed ids;
invalid parameters are refused with the policy, the field and the source span (never the given value); a
`fallback` on a policy that is not assisted is refused; a `store_bound_mib` outside 20 … 1 024 is refused
as `StoreBoundOutOfRange`, naming the value, the floor, the ceiling and the source span. The bound is the
host's, not a policy parameter: it is a field of the choice, and no policy sees it. With no `memory:` the
choice is `{ policy: consolidate }` with the 20 MiB bound.

**Bundled policies.** Each is an ordinary entry point of the group `mineworld.forgetting` in
`mineworld-cognition`, imports only the public names of `mineworld_cognition.memory.retention`, and none
is privileged. A user policy is a Python package that exposes one entry point of that group, loaded by
`importlib.metadata`. All arithmetic is integer.

| Policy | Uses a model | Rule | Parameters (bounded; default) |
| --- | --- | --- | --- |
| `consolidate` (default) | no | L0 records with `last_at < now − l0_hours · 3 600` merge into their episode. Episodes in a chapter with `last_at < now − l1_weeks · 604 800` merge into it. Chapters with `last_at < now − l2_years · 31 536 000` drop, except a chapter whose ranges hold a stable fact's first, last or level fact. Stable facts are kept. **Keepsakes:** per counterpart, the first `keepsakes` exchange records — a record of an event type in `EXCHANGE_KINDS` (`spoke`) whose only counterpart is that person, a line said to them or by them to the seat — stay records; their episode does not merge into its chapter, and a chapter holding one is not dropped. `keepsakes = 0` is the rule by age alone. The bound and its backstop are unchanged and still win. | `l0_hours` 0 … 720 (0); `l1_weeks` 1 … 104 (8); `l2_years` 1 … 50 (5); `keepsakes` 0 … 64 (4) |
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

## 5.3 Memory scopes

**Status:** design (step 28, `ARC-98` … `ARC-101`). Not yet implemented; S28-c implements the interface,
registry, selection and parameters, and S28-d the anchored scope. **Field reference:** the parameters of each
scope are generated into `reference/memory-scopes.md` (S25 PR 25b) and the `[memory]` table of `cognition.toml`
into `reference/cognition-toml.md`; a generated page is never edited by hand.

A **memory scope** is a plug-in that organises and recalls what a person perceived. It adds indices, private
rows and one recall section over a **seat view** (`ARCHITECTURE.md` §9.3). It is part of a Controller Pack or
of a user's own package, and it is selected per person in `decisions.yaml` `memory:` (§4.1), like a memory
policy (§5.2). A scope never grants access. Access is the **memory home**'s, and it comes only from perception:
a person can read a fact record only if their access row holds its key (`ARC-100`, `ARC-98`).

### Scopes

| Scope id | Owner module | Role |
| --- | --- | --- |
| `person` | `mineworld_cognition.memory.scopes.person` | Required. The person's private layer: access rows, episodes, chapters, stable facts, names, the forgotten set, the pass log, summaries. Its recall is the four sections L3, L2, L1 and L0 of P4, unchanged in content and order. A seat with no memory runs without a home; `person` is the base of every stack that has one. |
| `world` | `mineworld_cognition.memory.scopes.world` | Required. The shared fact records (`fact`, `fact_who`), insert-or-verify and garbage collection. Not removable: envelope fields must be stored somewhere. Its placement parameter `sharing` (`none` \| `home`) is an operator setting in `cognition.toml`, because it is invisible to the world. |
| `place`, `item` | `mineworld_cognition.memory.scopes.anchored` | Optional. Entity-anchored memory: one implementation configured per anchor kind. Rows derive purely from the envelope of a shared fact record (`place` gives role `at`, `subjects` give `subject`, `participants` give `participant`), and the recall section `About <name>:` is off unless a world sets `section_bytes`. Fed from perception, never from the world log. |
| user-defined, for example `organization` | the user's package | Any package exposing one entry point of the group `mineworld.memory_scopes`. An anchor scope is implemented as `place` is, outside the cognition package, to prove that a scope can be added without editing the kernel or cognition (`ARCHITECTURE.md` §9.3). |

Inner notes, reflections and any other model-written private text belong to the `person` scope's private layer.
No scope may index them as shared, and they are never written into a fact record. A belief reaches another
person only by being said or written in the world, which makes it a world fact that a System decides and
perception delivers (`ARC-100`).

Information that should reach someone who did not witness it reaches them as a world fact. A System decides
that someone was told, briefed, read a notice or heard a rumour, and perception then delivers that fact. A scope
organises what was perceived; it never widens who perceived it.

### Interface

Scopes are strict typed objects of `mineworld_cognition.memory.scopes`, checked by pyright in strict mode. The
shape follows the policy interface of §5.2 (`ARC-85`: interface, carrier, selection, parameters, host):

```python
class MemoryScope(Protocol):
    id: ClassVar[str]                                 # equal to the entry-point name; ^[a-z][a-z0-9-]{0,47}$
    Parameters: ClassVar[type[ScopeParameters]]       # strict, frozen; every field bounded and defaulted
    tables: ClassVar[tuple[TableSpec, ...]]           # named `<id>_*`; keyed by event_key and/or seat

    def index(self, fact: FactRecord, reading: Reading, params: Parameters) -> tuple[ScopeRow, ...]: ...
    def recall(self, view: SeatView, query: RecallQuery, budget: SectionBudget,
               params: Parameters) -> tuple[SectionLine, ...]: ...
    def account(self, view: SeatView) -> ScopeBytes: ...
```

- `index` is pure. It returns the rows to add when a seat first holds a fact. It receives the fact record and the
  seat's `Reading` (its gist, salience, `mine` flag and anchors), and never a store, connection, clock, path or
  gateway.
- `recall` reads only through `SeatView`, whose every method is filtered by the seat's access index. Every returned
  line carries a citation of held keys, and the host checks it.
- `account` reports the scope's share of the seat view, by table, for the size rule.
- **The host writes.** The memory home creates and drops the scope's tables, inserts the rows `index` returns,
  deletes them by `event_key` when a fact record is collected, and by `seat` when a seat leaves the home. A scope
  never deletes and never executes SQL writes (`ARC-100`).
- **The host checks every line.** A citation outside the seat's held keys is `AccessDenied`: a defect, named with
  the scope, the seat and the key, and never shown to a model. A section over its budget is
  `ScopeSectionOverBudget`.

### Registry and selection

- **Registry.** `ScopeRegistry` reads the entry points of the group `mineworld.memory_scopes` with
  `importlib.metadata`. It applies the same load checks as the policy registry of §5.2, and refuses duplicate ids,
  unknown ids and invalid parameters with the named errors below. The bundled `person`, `world`, `place` and `item`
  are ordinary entry points, and none is privileged.
- **Selection.** The memory stack is chosen per person: seat > class > default, in `decisions.yaml` `memory:` beside
  `policy` and `bound_mib`:

  ```yaml
  memory:
    default:
      policy: consolidate
      bound_mib: 20
      scopes:
        - id: person
          parameters: { sections: { L0: { lines: 48, bytes: 2000 } } }
        - id: place
          parameters: { section_lines: 4, section_bytes: 0, anchors_max: 256 }
  ```

  The Rust reader passes `scopes` through as typed JSON, `MemoryScopeChoice { id, parameters, span }`, as it passes
  `MemoryPolicyChoice` today. With no `scopes` key the stack is `[person]`. `person` is required and must be listed
  first; ids are unique.
- **Placement.** `cognition.toml` `[memory]` holds `sharing` (`none` \| `home`; `service` is refused before MVP-1 with
  `SharingModeUnavailable`), `home` (a path template, default `store/<instance>/memory.sqlite`), `cache_records`
  (0 … 1 000 000; default 4 096), `page_cache_kib` (256 … 1 048 576; default 8 192) and `verify` (`off` \| `pass` \|
  `every-commit`; default `off`, for debugging: it recomputes the invariants and sizes and raises on a mismatch). A
  stack changes what a person recalls, so it is the world author's; placement provably does not change it
  (`ARC-98`, `ARCHITECTURE.md` §9.3), so it is the operator's.

### Parameters

Every parameter model is a strict Pydantic model with bounded, defaulted fields. The defaults reproduce today's
context byte for byte.

| Scope | Parameter | Range | Default |
| --- | --- | --- | --- |
| `person` | `sections.<L>.lines` / `.bytes` | L3 0 … 16 / ≤ 1 600; L2 0 … 4 / ≤ 800; L1 0 … 8 / ≤ 1 600; L0 0 … 48 / ≤ 2 000 | P4's budgets. The sum over every scope's sections must be ≤ 6 000 B (D-P4-9) |
| `person` | `l0_window_s` | 3 600 … 86 400 | 86 400 (D-P4-11) |
| `place`, `item` | `roles` | a subset of `at`, `subject`, `participant` | `place`: `[at]`; `item`: `[subject, participant]` |
| `place`, `item` | `granularity` | `event` \| `episode` | `event`. In `episode` mode the index is per seat, `(seat, episode, entity)` rows, and recall cites the episode |
| `place`, `item` | `anchors_max` | 1 … 10 000 | 256 |
| `place`, `item` | `section_lines` / `section_bytes` | 0 … 16 / 0 … 1 600 | 4 / 0 (indexed, not recalled) |

A scope's sections count against the 6 000 B ceiling together with every other scope's sections; a stack whose
sections sum above it is refused with `TotalSectionsOverCeiling`.

### Named errors

Each error is raised by the module it names, and its message begins with that module's tag. No error echoes a
parameter's value.

| Module | Errors |
| --- | --- |
| `memory.home` | `NotAMemoryHome`; `HomeIdentityMismatch(field, stored, given)`; `HomeBusy(path, holder_pid)`, an OS lock on all three platforms; `SeatNotInHome(seat)`; `UnsupportedStoreSchema(found, wanted)`, which refuses schemas 2 and 3 and names the rebuild instruction |
| `memory.scopes` | `ScopeLoadError`; `DuplicateScope`; `UnknownScope(id, installed)`, naming the installed ids; `ScopeParameterError(scope, field, span)`; `PersonScopeRequired`; `ScopeSectionOverBudget(scope, section, bytes, limit)`; `TotalSectionsOverCeiling(total)`; `AccessDenied(scope, seat, event)`, raised by the host about a scope's line that cited a key the seat does not hold |
| `memory.scopes.world` | `ConflictingFact(event)`, when a second holder's digest differs; `FactRecordMissing(seat, event)`; `HolderInvariantBroken(keys)`, when `verify` finds an orphan or a dangling key |
| `memory.retention` | `RetentionBoundUnreachable` (`BoundBelowDefault(seat, given_mib)` for a bound under 20 MiB, naming the source span) |
| `memory.config` | `SharingModeUnavailable(mode)` |

### Inspection and degradation

- **Inspection.** `mineworld-memory inspect HOME` (a console script of `mineworld-cognition`) and
  `MemoryHome.inspect()` share one implementation and one typed `InspectReport`. The command opens the home
  read-only, never writes, and runs while the cognition process is stopped. It answers four questions: what a person
  can access and why (`--seat S access [--event N]`); what was forgotten, by which policy and pass (`forgotten`);
  where the bytes are, by scope and table, and which scope overran (`bytes`); and every recalled line traced back to
  its facts (`recall --at T [--trace]`). `scopes` shows the installed and selected scopes, with the source span of
  each parameter.
- **Removing a scope from a stack.** Its section is gone, its private rows for that seat are dropped at the next open
  and recorded in the pass log as `scope-removed:<id>`, and shared index rows are dropped when no remaining seat
  selects the scope. Memory and the world are unaffected.
- **Uninstalling a scope a stack still names.** The home refuses to open with `UnknownScope`, naming the installed ids.
  The home is never opened with a scope it cannot run, so no table is silently stale.
- **Reinstalling a scope.** The home rebuilds the scope's rows from the held fact records and access rows, in key
  order. `index` is pure, so the rebuilt rows are byte-identical to those of an uninterrupted run.
- **No home.** A controller without `cognition.toml` memory, or a non-cognition controller, runs as it does today. The
  world never reads memory.

### Testing each scope in isolation

Each scope has, without a home or a binary:

- a property test of `index`: the same fact and reading give the same rows, and no row cites another key;
- a recall test against a `SeatView` fake that holds a fixed key set and raises on any other key;
- a budget test: every line is ≤ 160 B and every section is within its bytes.

The memory home's own tests use a hostile fixture scope that cites unheld keys, returns oversized sections and
raises. Every scope is covered by the isolation and provider scans of `ENGINEERING_STANDARDS.md` (no provider concept
in memory, no connection, path or clock in a scope's signature).

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
[`clients/2d/PRESENTATION.md`](../clients/2d/PRESENTATION.md) (`DECISIONS.md` `ARC-46`). What `characters/cast.yaml` contains for the 3D reference client — which character draws a person with a given disclosed `look`, and the extras that draw everyone else — is specified in [`clients/3d-spike/PRESENTATION.md`](../clients/3d-spike/PRESENTATION.md) (`DECISIONS.md` `ARC-92`). A style that covers more than one dimension is more than
one pack, grouped in a family directory, because one manifest cannot describe two dimensions
whose character proportions differ (`DECISIONS.md` `ARC-3`). The shipped default:

```text
presentation/mineworld-default/
├── README.md     what the family is, and what the packs must share
├── 2D/           manifest.yaml · ART_DIRECTION.md · references/
└── 3D/           manifest.yaml · ART_DIRECTION.md · references/ · characters/cast.yaml
```

Each dimension pack also carries a **`pack.yaml`**, its package identity (`DECISIONS.md` `ARC-53`,
[`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md) §5.0): `id` (`mineworld-default-2d`, `mineworld-default-3d`),
`type: presentation-pack`, `version`, `mineworld`, `license`, `authors`. The style manifest keeps its
own `id`, which names the style, not the pack. `mineworld packs validate` checks both files: the
package fields, and that the style manifest has an `id` and a `dimension` list.

The reference images are the source of truth; the manifest
summarises them, and a generation prompt is an implementation aid rather than the contract
(§16).

A 3D pack that casts characters may also carry `characters/casting.yaml`. Per cast it holds:

- the worlds the cast serves;
- the style template;
- the character source and its parameters;
- the distinctness rule;
- substitutions and prompt phrasing;
- the quality-tier control and band;
- the licence policy.

The rules are in `.structured-coding/plans/mvp0/step-26-cast.md` §7.4b.2 (`ARC-134`, which S26-src2 writes into `DECISIONS.md`).

The population profile and its seed are not in it. They are the world's own data, read through the
`attributes` pack (step 30).

It is production data read only by `tools/character_pipeline/`. No client, server or System reads it, and no
runtime file names which character source made a character (`ARC-130`).

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
mineworld reference (--out DIR | --check DIR)
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
| `server` | Hosts a World Pack for clients (`NETWORKING.md`). In-server controllers run on the world thread (`DECISIONS.md` `ARC-42`): `--agent SEAT` drives that seat with the reactive rule controller, which answers only what it hears while bound (`F-13`); `--town` drives every other seat with its resolved decision model (§4.1; with no decision list, the paced rule controller), seeded by `--seed N` (default 0) and consulted every `--pace SECONDS` **wall** seconds (default 5), seat `k` at `genesis + (k + m·pace)·time_scale` in world time; the reactive controller is consulted once a wall second. The pack's `decisions:` file, `--decisions FILE` and `--seat-model SEAT=MODEL` choose the `--town` seats' models; `--decisions` or `--seat-model` without `--town` is refused ("decisions choose how in-server seats decide; add --town"), a seat named by both `--agent` and `--seat-model` is refused, `--agent` otherwise wins over the list for its seat, and a seat whose fallback chain ends at `none` is not driven in-server. With a decision list in effect, the statistics line's `town:` statement ends with the decisions clause of §4.1 rule 7. A paced seat whose person is walking is also asked for the next stride of its walk (`walk-step`) once a wall second, every `time_scale` world seconds, so a hosted walker covers 1.34 m per wall second at any time scale (`ARC-42` note of 2026-10-09; `ARC-75`). Cadence is wall time so that the time scale never makes a hosted Person walk or talk faster (QTW-13, amending `ARC-40`). A client joining a seat an in-server controller drives takes it over; when it leaves, or its hold ends, the seat returns to a controller built afresh (`server/PROTOCOL.md` §4.2, `ARC-40`). `--hold SECONDS` is how long a dropped connection's seat is held for its `resume`, in wall seconds (default 30; `0` holds none). `--time-scale N` is how many world seconds pass per wall second (an integer ≥ 1, default 1), reported as `time_scale` in `/status` and every welcome. `--keyframe-every N` (an integer ≥ 1, default 50) makes every Nth frame to a client a whole observation and the others `delta` frames (`server/PROTOCOL.md` §5.3, `DECISIONS.md` `DEP-15`); `1` sends every frame whole. On Ctrl-C the server prints `[world] ticks N, p50 A ms, p99 B ms, longest tick M ms, cpu p50 C ms, cpu p99 D ms` — the world thread's tick in wall time, then in that thread's own CPU time (`ARC-42` note of 2026-10-15, `DEP-59`) — and one `[world] hosted <seat>: …` line per in-server controller with its consults and outcomes. `--save DIR` keeps the world in `DIR/world.sqlite`, created the first time and resumed afterwards. `--invite TOKEN` (or the environment variable `MINEWORLD_INVITE`; the flag wins) is the invite every client must present in its `join`; with neither, the server generates one and prints it once as `[mineworld] invite <token> — join with: <address> seat=<seat> invite=<token>`. There is no mode without an invite, loopback included (`server/PROTOCOL.md` §4.1, `DECISIONS.md` `DEP-14`). `--admin-token TOKEN` (or the environment variable `MINEWORLD_ADMIN_TOKEN`; the flag wins; `--help` names the variable and never a value) mounts the admin surface under `/admin` — sessions, seats, kick, release, and pausing and resuming the host clock (`server/PROTOCOL.md` §11, `DECISIONS.md` `ARC-44`) — and the server prints `[mineworld] admin surface: http://<address>/admin (bearer token as given)`; without it no `/admin` route exists and the server prints `[mineworld] no admin surface (no --admin-token)`. The admin token follows the invite's rules, is never printed, and must differ from the invite: an illegal token, or one equal to the invite, stops the server before it listens with a message that repeats neither. Launchers pass it through the environment, not the command line. On Ctrl-C — and on Windows also Ctrl-Break, closing the console window, or logging off — the server stops the same graceful way: it stops accepting connections, checkpoints the save and prints its statistics. |
| `validate` | Reads and loads a World Pack and reports the world it describes: systems, places, people, seats, the identity each key received, the number of genesis facts. When `world.yaml` names a decision list, it is decoded (§4.1) and one line follows the `requires` lines: `  decisions  <n> assignment(s), default <id>`; without the key nothing new is printed. |
| `replay` | Re-executes a save's whole journal from genesis and checks every fact and snapshot byte for byte (`ARC-25`). |
| `run` | Runs a World Pack headless: no renderer, no network, no model. Every seat the pack offers is driven by its resolved decision model (§4.1, §5.1) — with no decision list, the seeded paced rule controller (`ARC-27`). Described below. |
| `decisions` | Reads a World Pack and its decision list exactly as `run` does (the same decoder, `--decisions` and `--seat-model`, the same refusals), starts no world and writes nothing. Described below. |
| `inspect` | Reports what a save holds, without resuming or writing it. Described below. |
| `biography` | Prints a Person's objective biography, derived from a save's fact log without resuming or writing it ([`DECISIONS.md`](DECISIONS.md) `ARC-29`). Described below. |
| `perceived` | Prints the facts a Person perceived, judged from a save's fact log by the same audience rule the server's `perceived` stream uses, without resuming or writing the save ([`DECISIONS.md`](DECISIONS.md) `ARC-43`). Described below. |
| `create` | Writes a new, minimal World Pack into a directory that does not exist yet. Described below. |
| `packs` | Prints package identities (`DECISIONS.md` `ARC-53`): `list`, `show` one, `validate` one data pack. Described below. |
| `reference` | Writes the generated configuration reference (`DECISIONS.md` `ARC-90`), or checks a directory against it. Described below. |

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

**`reference`.** Exactly one of `--out DIR` and `--check DIR`. It loads no World Pack and starts no
world; what it generates is a function of the build alone: the installed set's packs in its order, each
type's fields in name order, no timestamp and no commit hash, `\n` line endings on every OS. The pages
are `README.md` (the index), `world-pack.md` (`world.yaml` and the content files' own fields),
`sections.md` (each section an installed pack owns), `configure.md` (each `configure/` file other than an
Interaction List section, the framework keys `classes` and `packages` included), `interaction-list.md`
(the section shape and each pack's declarations), `decisions.md` (`decisions.yaml`, with each installed
decision model and the parameters it takes, from the installed models' catalog), `pack-manifest.md` (a data pack's `pack.yaml` and a
style manifest) and `cli.md` (every visible subcommand and argument, walked from the parser), plus the
JSON Schemas under `schema/`. Every Markdown page begins with a generated banner. A field's description
is the first paragraph of its doc comment; generation is refused, naming the type and the field, when a
field has none. `--out DIR` writes the files, creating `DIR` if needed, removes generated files it no
longer writes, and refuses — writing nothing — when `DIR` holds a file it did not generate.
`--check DIR` writes nothing; it exits 0 when `DIR` holds exactly what this build generates, and
otherwise exits 1 naming every file that differs, is missing, or is extra. The same directory holds the
cognition half's files, `cognition-toml.md` and `schema/cognition-toml.schema.json`, generated from the
cognition package's models and drift-checked by that package's own test (§5.2): neither `--out` nor
`--check` writes, removes or compares them, and the index links them.

**`decisions`.** Prints each seat's resolved decision model and memory choice (§4.1), in roster order.
Without `--json`, one line per seat for a person. With `--json`, one JSON document on stdout, the form the
cognition process reads (QFGb-1):

```json
{
  "format": 2,
  "world": "social-cafe",
  "digest": "00f3…",
  "seats": [
    { "seat": "alice",
      "model": { "id": "rule", "assigned": "lm", "chain": ["lm", "rule"] },
      "memory": { "policy": "lm-summarise", "fallback": "consolidate",
                  "parameters": { "calls_per_sim_day": 4 }, "store_bound_mib": 40,
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
`policy`, `fallback` (`null` when absent), `parameters` (`{}` when absent; keys sorted),
`store_bound_mib` (`null` when absent: the host's 20 MiB; §5.2 host rule 7) and `source`: the
file as written in `world.yaml`'s `decisions:` or as passed to `--decisions`, and the line and column of
the winning entry's mapping. `format` 2 added `store_bound_mib`. The command is read-only.

**`inspect`.** Reads `DIR/world.sqlite` and prints the manifest (pack, instance, composition), the
head revision and its instant, the journal counted by input kind and outcome, the snapshots the save
retains with each one's stored size on one line (`DECISIONS.md` `ARC-81`), the log's storage on one
`log` line — how many blocks are sealed, the revisions they cover, their stored bytes and the bytes of
the rows they hold, and the revisions of the live tail (`DECISIONS.md` `ARC-89`), the facts counted by
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
