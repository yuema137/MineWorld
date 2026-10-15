# PR S30-b — The authoring seam: data directories, decoding in the owner's context, and owner resolution before seeding

## DESIGN FROZEN — 2026-10-15 (operator approved the freeze in the primary session: "同意冻结并实施 S30-a/b")

```text
Design revision:        revision 2 (2026-10-15: §14's ruling applied), FROZEN 2026-10-15
Author:                 S29-a / S30-a / S30-b PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s29a-s30ab, branch docs/pr-designs-s29a-s30ab,
                        planning base origin/main @ e6030a36
Approved by / evidence: the operator, directly in the primary session, 2026-10-15: "同意冻结并实施 S30-a/b";
                        Q-S30b-1 ruled as recommended (§14.1), relayed by the coordinator
Implementation base:    origin/main at C0; C0 records the commit and that S30-a has merged
Start condition:        S30-a merged (step-30 §13.3: "S30-b … after S30-a"; NOT merged at e6030a36; S30-a was
                        frozen 2026-10-15), this design frozen (2026-10-15), the primary's dispatch
Execution contract:     §12
Implementer:            Opus-class (step-30 §13.1, the S30-b row; R-S30-4: "the seam touches the loader that
                        every world uses … Opus implementer"); reviewer the primary session
Lifecycle:              DESIGN FROZEN 2026-10-15
```

**Effort:** `mvp0` · **Step:** [`step-30-person-attributes.md`](step-30-person-attributes.md) (APPROVED 2026-10-15;
amended) §§3.1 (A-1, A-2), 3.3 (F-S30-2, F-S30-8), 4.3 (`ARC-121`), 8.5 item 5, 11.2 (X-S30-9), 12 (I-S30-10),
13.1 (the S30-b row), 13.5 (X-S30-9), 14 (R-S30-4) · **Specified by:** [`pr-s30a-attribute-specs.md`](pr-s30a-attribute-specs.md)
§5.3 (`MODULE_SPEC.md` §3's seam text, `ARC-121`) · **Parent:** [`overall.md`](overall.md) · **Followed by:** S30-c
(the `attributes` pack, the seam's first real user) · **Binding parents:** `CLAUDE.md` §§2, 4 (items 1, 2, 4, 5, 7,
8, 11, 12, 13); `docs/MODULE_SPEC.md` §§2, 3.1, 4.1; `docs/DECISIONS.md` `ARC-15`, `ARC-26`, `ARC-31`, `ARC-33`,
`ARC-36`, `ARC-54`, `ARC-61`, `ARC-71`, `DEP-10`, `ARC-121` (written by S30-a).

**Decision ids.** None new. `ARC-121` is S30-a's record; this PR implements it and, if its implementation finds a
correction, adds a dated note under it (§10 R-S30a-b-1). `ARC-127` … `ARC-129` and `DEP-69` are not used.

No model is called; no world's bytes move.

---

## 0. Binding inputs, exactly restated

- **Scope (step-30 §13.1, S30-b row).** "`authoring/`: data directories, context-seeded decode, owner `resolve`;
  `worldpack/` loader wiring; no pack uses it except a test fixture pack."
- **Integration checkpoint (same row).** "a fixture pack owning a data directory and a configuration loads; a
  person-file value is refused at its own line against a data-declared schema."
- **Adversarial criteria (same row).** "X-S30-9 every reference world byte-identical; an unknown data directory
  refused naming known ones; a seeded fact from another vocabulary still refused (`ARC-31` item 3)."
- **The seam (step-30 §4.3, `ARC-121`).** "1. A System Pack may own a data directory in World Packs and Entity
  Packs. … The loader collects the files, in composition order, and hands them to the owner to decode with line
  and column. It never learns what they mean. … an unknown directory is refused and the error lists the
  directories this build knows. 2. No world-level section is needed … `configure/attributes.yaml` … 3. A section
  may be decoded in its owner's context … decoded from the owner's data directories and its configuration … 4. An
  owner may resolve across files before seeding … called once, before any per-subject `seed` … Seeding order is
  unchanged (`ARC-31` item 4). The seam is general. It does not mention attributes."
- **Invariant I-S30-10.** "A world without the `attributes` pack and owned attributes is byte-identical to before
  S30" — for this PR: **every** world, since no installed pack uses the seam.
- **R-LOAD-1.** Locally only the targeted test files of a commit and `cargo clippy` on touched crates; never
  `cargo test --workspace`; CI on the exact head is the gate.

---

## 1. Goal

Give System Packs four general, additive capabilities in the authoring seam, so that S30-c's `attributes` pack can
be written as an ordinary pack without the loader learning a word of it:

1. **Own a data directory** (`attributes/`, `populations/` …) in World Packs and Entity Packs: the loader collects
   the files in composition order, the owner decodes each with line and column, then builds one **context** from
   all of them and its world configuration. An unknown directory is refused, naming the ones the build knows.
2. **Decode a section in that context**, so a value in a person file is checked against a schema declared in
   data, at the value's own line and column.
3. **Resolve across subjects once, before seeding**: the owner sees every file of the kinds its section is
   carried by — the key, the tags and its decoded section if any, never a name or a note — and returns one seed
   per subject; seeding order is unchanged.
4. **Ship data files** from a System Pack for an owner's directory (Q-S30b-1, ruled 2026-10-15: compiled in through `SystemPack::SHIPS`), so that S31's notices and
   `economy` can declare their attributes (step-30 §9.6 N-2, step-31 §7.4).

No existing section, configuration, world or pack changes; every reference world is byte-identical (X-S30-9).

---

## 2. Scope

### 2.1 In scope

1. **`authoring/src/data.rs`** (new): `DataDirectory`, `DataOrigin`, `DataRefusal`, `OwnerContext`, the traits
   `OwnsData`, `SectionInContext`, `ResolvesSubjects`, the erased holders and `DecodeIn`, `Subject`,
   `ResolvedSeeds`, `ResolveRefusal`. `authoring/src/content.rs` and `configuration.rs`: `as_any()` on the two
   erased traits (implemented only inside this crate). `authoring/src/lib.rs`: re-exports.
2. **`sdk/rust/src/pack.rs`, `section.rs`, `installed.rs`, `lib.rs`**: `SystemPack`'s new provided items (each with
   a default that keeps every existing pack unchanged), the macros `owns_data!`, `owns_section_in_context!`,
   `resolves!`, `ships_data!`, and the `installed!` dispatch arms.
3. **`packages/src/manifest.rs`**: an Entity Pack "declares something" when it has an item kind **or** a `.yaml`
   file in some other non-dot directory; the three forbidden directories stay forbidden.
4. **`worldpack/`**: `src/data.rs` (new: collect, decode, build contexts; the unknown-directory rule); `src/read.rs`
   (read steps 4d and 10b, fields, accessors, module doc); `src/content.rs` (decode in context); `src/load.rs`
   (seeding from resolutions, in the unchanged order, with the vocabulary check); `src/entities.rs` (an Entity
   Pack's data directories; `validate_entity_pack` decodes them); `src/error.rs` (four variants); in-crate tests.
5. **`tests/acceptance/`**: a test-only System Pack `test-schema` and `tests/data_seam.rs` (the integration
   checkpoint, driven exactly as the loader drives the seam, as `configuration_seam.rs` does for `ARC-61`).
6. **`docs/MODULE_SPEC.md` §3.1**: the "implemented by S30-b" marks S30-a wrote become "implemented"; any bounded
   correction found here, as a dated note (R-S30a-b-1).

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| Any attribute word, type, file or pack; `systems/attributes`, `authoring/attributes` | S30-c |
| Profiles, the sampler, `configure/attributes.yaml`'s type | S30-d |
| Installing a pack that uses the seam; any world adopting one | S30-c, S30-g |
| `mineworld attributes …` commands | S30-c, S30-d |
| A resume-time drift check over data directories | S30-c (R-S30c-b-2) |
| Any change under `kernel/`, `contracts/`, `systems/*/src` (other than none), `server/`, `clients/`, `cognition/`, `worlds/` | never |

### 2.3 Dependencies

S30-a merged (the specification and `ARC-121`). Nothing else. Not required: S26-a, 12d, S29-a (independent files;
if S29-a's `settlement.rs` lands first, `read.rs`'s step list gains both 9b and 10b, and whichever merges second
rebases).

---

## 3. Audit (`origin/main @ e6030a36`, 2026-10-15)

### 3.1 Anchors

| # | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-1 | `AuthoredSection { SECTION, CARRIED_BY, type Authored: DeserializeOwned, references, seed }`; `Decode<S>` is a `DeserializeSeed` producing `Arc<dyn AuthoredContent>`; the erased `AuthoredContent` answers `section`, `owner`, `references`, `seed` | `authoring/src/section.rs:172-203`; `authoring/src/content.rs:18-96` | Static decode: a section's type cannot see world data (F-S30-2) |
| A-2 | Seven `AuthoredSection` impls in `systems/` (economy, bodies, schedule, employment, item, naming, inventory), one in `tests/acceptance/tests/configuration/mod.rs`, and the pinned external `acme-fishing` (`systems/installed/Cargo.toml:31`, git rev `b40e71ff`) | `grep -rln "AuthoredSection for" systems tests` | **No required item may be added to `AuthoredSection` or `SystemPack`** (F-S30b-1) |
| A-3 | The toolchain is Rust 1.97.1; associated-type defaults (`type Context = ()`) are unstable on stable Rust | `rust-toolchain.toml`; rust-lang issue 29661 (`associated_type_defaults`) | The step's "optional `type Context`" is realised as a **sibling trait**, not as a defaulted associated type (F-S30b-1) |
| A-4 | `PackConfiguration` / `AuthoredConfiguration` / `DecodeConfiguration` / `ConfigurationContext`; `AuthoredConfiguration` is implemented only by `Held<P>` inside `authoring` | `authoring/src/configuration.rs:20-222` | Adding `fn as_any(&self) -> &dyn Any` to the erased trait breaks no implementor outside the crate; an owner reaches its own typed configuration by downcasting |
| A-5 | `SystemPack: System + Default` with provided items: `PACKAGE` (required), `BIOGRAPHICAL`, `SECTION`, `decode_section`, `CONFIGURATION`, `CONFIGURATION_FACTS`, `INTERACTIONS`, `decode_configuration`; `owns_section!` and `configures!` define pairs of them | `sdk/rust/src/pack.rs:25-180` | New items are **provided** with safe defaults, defined only through new macros, as the existing ones are |
| A-6 | `installed!` expands the build's set into `Capability` with one dispatching method per item (`resolve`, `id`, `section`, `owning_section`, `decode_section`, `configuration` …) | `sdk/rust/src/installed.rs:140-200`; `systems/installed/src/lib.rs:34-58` | The new items get dispatch arms there; installing a pack still edits nothing in `worldpack/` (`ARC-33`) |
| A-7 | `read_with`'s order: manifest … systems (`read.rs:133`), decisions, licence policy, requirements (composition), extensions, **configuration** (`read.rs:148`, step 4c), keys, **content** (`read.rs:150-180`), undeclared files, Entity Pack items (`read.rs:192`, step 6b), seats, locations, passages, `check_sections` (`read.rs:220`), `configure::check_references` | `worldpack/src/read.rs:8-35, 115-224` | Data directories are read after configuration and before content (**step 4d**), so every section is decoded with its owner's context; resolution runs after `check_sections` (**step 10b**), when every section is decoded and every reference checked |
| A-8 | Content is decoded through `ContentFile`'s `visit_map`; a section key resolves to its owner by `Capability::owning_section` and is decoded by `owner.decode_section(&mut map)` | `worldpack/src/content.rs:172-205` | One call changes to `owner.decode_section_in(&mut map, contexts.get(&owner))`; for every installed pack it falls back to `decode_section` |
| A-9 | Genesis order: passages, locations, configuration in `configure:` order, then sections — items', organizations', places', people's, each in key order; within a file in the world's `systems:` order (`in_composition_order`) | `worldpack/src/load.rs:31-35, 300-380` | A resolving owner seeds at exactly the positions its sections would: for every subject of its carried kinds, in that order. A world with no resolving owner seeds exactly as today |
| A-10 | `seeded()` refuses a fact of another pack's vocabulary (`PackError::SectionStatedAnotherPacksFact`), tested by `Trespasser` | `load.rs:63-100, 451-540` | Resolved seeds go through the same check (adversarial criterion 3) |
| A-11 | Entity Packs: `packages::check_entity_layout` refuses `places/`, `people/`, `organizations/` and requires ≥ 1 `items/*.yaml`; `worldpack::entities` reads only `items/`; other directories are ignored | `packages/src/manifest.rs:26, 110-145`; `worldpack/src/entities.rs:1-120, 143-206` | An Entity Pack of data only (`entities/person-attributes`, S30-e) is refused today as `EntityPackDeclaresNothing`: the rule is widened (C3) |
| A-12 | A World Pack's directories: the loader reads `people/`, `places/`, `items/`, `organizations/`, `configure/` and the `data/` attachments; it ignores any other directory | `read.rs:565-637`; `configure.rs` (`DIRECTORY`, `ATTACHMENT_DIRECTORY`); `worlds/*` hold only those plus `README.md` | The unknown-directory refusal is **new** behaviour (F-S30b-3); `decisions:` may name a file in a subdirectory (`format.rs:95-128`), whose first component must be admitted |
| A-13 | `serde-saphyr` is called from `read.rs`, `configure.rs` and `entities.rs:88` (the DEP-10 comment at `read.rs:434` names only two) | `grep -n serde_saphyr worldpack/src/*.rs` | `data.rs` becomes a fourth caller; the comment is corrected to list all four |
| A-14 | The configuration seam's end-to-end test drives a test-only pack (`test-tuning`) through the same calls the loader makes, in `tests/acceptance`, because the loader's `Capability` is closed; "the loader's order itself is held by worldpack's in-crate test" | `tests/acceptance/tests/configuration_seam.rs:1-25`; `tests/acceptance/Cargo.toml` dev-dependencies note (step-10 QS-24) | The same split here: `tests/acceptance/tests/data_seam.rs` drives `test-schema`; worldpack's in-crate tests hold collection order, refusals and seeding order with in-crate probes (the `WorldPack::in_memory` / F-22 precedent, `read.rs:347-380`) |
| A-15 | `Capability` is `Ord` (it is a `BTreeSet` element in `resolve_systems`, `read.rs:508-524`) | `read.rs:510` | Contexts and resolutions are `BTreeMap<Capability, …>`: deterministic |
| A-16 | No mechanism lets a System Pack crate contribute data: a pack's crate directory is not a pack root the loader reads | `worldpack/src/read.rs`; `packages/src/roots.rs` | F-S30b-2, Q-S30b-1 |

### 3.2 Findings

- **F-S30b-1. The context is a sibling trait.** A defaulted associated type is unstable (A-3), and a required one
  breaks nine implementors, one of them external at a pinned revision (A-2). So `SectionInContext` is a separate
  trait in `authoring`, with its own `owns_section_in_context!` macro that defines the same `SystemPack::SECTION`
  and both decode methods. A pack implements either `AuthoredSection` or `SectionInContext`, never both (one
  section per pack, `ARC-31`); the macro makes the second impossible to state twice. The step's intent (§4.3
  item 3) is unchanged; `pr-s30a-attribute-specs.md` F-S30a-4 already words the spec as semantics.
- **F-S30b-2. A System Pack cannot contribute data today** (A-16), yet step-30 §9.6 N-2 and step-31 §7.4 have
  S31's notices pack and `economy` declare attributes "in its pack's `attributes/`". **Q-S30b-1.**
- **F-S30b-3. Refusing unknown directories is new.** No reference world has one (A-12), but a user's World Pack
  may hold `notes/` or `.git/`. The rule this PR writes: directories whose name begins with `.` are not content and
  are ignored (as files that are not `.yaml` are ignored today); every other directory must be a format directory,
  the first component of `decisions:`, or a data directory of an **installed** pack; an installed pack's
  directory in a world that does not enable the pack is refused naming the pack (as a section of a disabled pack
  is, `ContentNeedsASystem`). This is the step's rule with its two necessary admissions stated, not a widening.
- **F-S30b-4. Resolution must see subjects without a section.** Sampling fills attributes a person's file does not
  state, including for people whose file has no `attributes:` at all. So `resolve` receives every file of the
  carried kinds, the section optional, and the owner seeds every subject it returns a seed for — including those
  with no section. Without a resolving owner, nothing changes.
- **F-S30b-5. The subject type is the privacy boundary.** Step-30 §8.5 item 5: "The sampler's input type holds the
  person's key, their tags, their authored attributes and their lock entries. It holds no display name, no `note`
  and no `persona`." `Subject` has exactly `key`, `kind`, `tags` and `authored: Option<&A>`; the loader never
  passes `AuthoredPerson`. X-S30-6's structural half is therefore a type property; its behavioural half is S30-d's
  rename test.
- **F-S30b-6. The `DEP-10` comment is stale** (A-13); corrected in C4.

### 3.3 What does not exist

No data directory, owner context, contextual decode, resolution or shipped data anywhere. No test-only pack owns
data. `MODULE_SPEC.md` §3.1's seam text exists only after S30-a.

---

## 4. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-b-1 (I-S30-10, X-S30-9) | Every reference world reads, loads and runs byte-identically: genesis, every fact, every save | the nightly long record's `baseline check` on the final head; `tools/cli/tests/decisions.rs`' `social-cafe-seed-7-30-days` golden unchanged; `worldpack/tests/social_cafe.rs` unchanged |
| I-b-2 | No existing `impl AuthoredSection`, `impl PackConfiguration` or `impl SystemPack` changes, and `acme-fishing` at its pinned revision builds | the diff (`git diff --stat origin/main...HEAD -- systems/`: only `systems/installed` is allowed, and only if a dispatch needs it — it should not); CI build |
| I-b-3 | The seam names no attribute word: `git grep -n -i "attribute\|catalogue\|traits\|population\|profile" -- authoring/src/data.rs worldpack/src/data.rs sdk/rust/src` is empty (the Rust keyword `trait` is not S30 vocabulary and is not searched) | C7 scan |
| I-b-4 | The loader never learns what data means: it lists, reads and hands over files, then holds the owner's context and resolution type-erased | review; the types (`Arc<dyn OwnerContext>`, `Arc<dyn ResolvedSeeds>`) |
| I-b-5 | Seeding order is unchanged (`ARC-31` item 4): a resolving owner's facts appear exactly where its section's facts would | worldpack in-crate test `resolved_seeds_take_the_sections_places` |
| I-b-6 | A resolved seed of another pack's vocabulary is refused by name, as a section's is | worldpack in-crate test with a resolving `Trespasser` |
| I-b-7 | Data files are collected in one deterministic order: shipped (per Q-S30b-1) in `systems:` order, then required Entity Packs in id order, then the world; within each, directory then file name, byte order | worldpack in-crate test `data_is_collected_in_composition_order` |
| I-b-8 | A subject handed to `resolve` carries no name, note or location | the `Subject` type (F-S30b-5); review |
| I-b-9 | Every refusal names the file (and, for a decode, the line and column) and the owning system | the tests of §7 |

---

## 5. Design

### 5.1 Module layout

```text
authoring/src/data.rs        DataDirectory, DataOrigin, DataRefusal, OwnerContext,
                             OwnsData, SectionInContext, ResolvesSubjects, Subject, ResolvedSeeds,
                             ResolveRefusal, DecodeIn, the erased holders
authoring/src/content.rs     AuthoredContent::as_any
authoring/src/configuration.rs  AuthoredConfiguration::as_any
sdk/rust/src/pack.rs         SystemPack: DATA, decode_data_file, data_context, decode_section_in, RESOLVES,
                             resolve, SHIPS (Q-S30b-1); macros owns_data!, owns_section_in_context!,
                             resolves!, ships_data!
sdk/rust/src/installed.rs    Capability: data, owning_data, decode_data_file, data_context, decode_section_in,
                             resolves, resolve, shipped
packages/src/manifest.rs     check_entity_layout: data-only Entity Packs declare something
worldpack/src/data.rs        the Owner seam inside the loader (a pub(crate) trait implemented for Capability and
                             for in-crate probes), collect, decode, contexts, the directory rule
worldpack/src/{read,content,load,entities,error}.rs   wiring
tests/acceptance/tests/data_seam.rs, tests/acceptance/tests/schema/mod.rs   test-schema, end to end
```

### 5.2 Data directories (`OwnsData`)

```rust
/// A directory of a World or Entity Pack that a System Pack owns: one word, 1..=32 bytes of a-z, 0-9, -,
/// checked at compile time for a constant (as SectionName).
pub struct DataDirectory(&'static str);

/// Where one data file came from, for every refusal about it.
pub struct DataOrigin { pub pack: String, pub source: DataSource, pub directory: DataDirectory, pub path: PathBuf }
pub enum DataSource { SystemPack, EntityPack, World }

/// A pack that owns data directories.
pub trait OwnsData: SystemIdentity {
    const DIRECTORIES: &'static [DataDirectory];
    /// One decoded file. Decoding is validating; a refusal keeps the file's line and column.
    type File: core::fmt::Debug + Send + Sync + 'static;
    /// What every file of the world's composition, and the owner's configuration, become.
    type Context: core::fmt::Debug + Send + Sync + 'static;
    fn decode_file<'de, D: Deserializer<'de>>(directory: DataDirectory, d: D) -> Result<Self::File, D::Error>;
    /// Cross-file checks (duplicates, namespaces …) and the context, once.
    fn context(files: Vec<(DataOrigin, Self::File)>, configuration: Option<&dyn Any>)
        -> Result<Self::Context, DataRefusal>;
}
pub struct DataRefusal { pub file: Option<PathBuf>, pub reason: String }   // the owner's words, naming the file

pub trait OwnerContext: core::fmt::Debug + Send + Sync { fn owner(&self) -> SystemId; fn as_any(&self) -> &dyn Any; }
```

`configuration` is the owner's own decoded configuration (`AuthoredConfiguration::as_any`), `None` when the world
lists none. The owner downcasts to its `PackConfiguration::Configuration`; a failed downcast is a programming
error of the owner, refused as `DataRefusal` naming no file.

### 5.3 Decoding a section in its owner's context (`SectionInContext`)

```rust
pub trait SectionInContext: OwnsData {
    const SECTION: SectionName;
    const CARRIED_BY: &'static [ContentKind];
    type Authored: core::fmt::Debug + Send + Sync + 'static;
    fn decode<'de, D: Deserializer<'de>>(context: &Self::Context, d: D) -> Result<Self::Authored, D::Error>;
    fn references(authored: &Self::Authored) -> Vec<Reference<'_>> { Vec::new() }
    fn seed(seeding: &Seeding<'_, '_>, subject: EntityId, context: &Self::Context, authored: &Self::Authored)
        -> Result<Vec<Emission>, Rejection>;
}
/// A DeserializeSeed carrying the context; produces the same Arc<dyn AuthoredContent> the loader holds today.
pub struct DecodeIn<S: SectionInContext> { context: Arc<S::Context> }
```

The erased holder keeps an `Arc` of the context, so `AuthoredContent::seed(seeding, subject)` — whose signature does
not change — can call `S::seed` with it. `SectionOwner::of_in_context::<S>()` gives the same `{ name, carried_by }`
the build lists today.

### 5.4 Resolving across subjects (`ResolvesSubjects`)

```rust
pub struct Subject<'a, A> { pub key: &'a EntityKey, pub kind: ContentKind, pub tags: &'a Tags, pub authored: Option<&'a A> }
pub trait ResolvesSubjects: SectionInContext {
    type Seed: core::fmt::Debug + Send + Sync + 'static;
    /// Once, after every section is decoded and every reference checked, before any seeding.
    fn resolve(context: &Self::Context, configuration: Option<&dyn Any>, subjects: &[Subject<'_, Self::Authored>])
        -> Result<BTreeMap<EntityKey, Self::Seed>, ResolveRefusal>;
    /// The genesis facts one subject's seed becomes, in this pack's own vocabulary.
    fn seed_resolved(seeding: &Seeding<'_, '_>, subject: EntityId, context: &Self::Context, seed: &Self::Seed)
        -> Result<Vec<Emission>, Rejection>;
}
pub struct ResolveRefusal { pub subject: Option<EntityKey>, pub reason: String }
pub trait ResolvedSeeds: core::fmt::Debug + Send + Sync {
    fn owner(&self) -> SystemId;
    /// None: no seed for this subject (it states nothing).
    fn seed(&self, seeding: &Seeding<'_, '_>, key: &EntityKey, subject: EntityId) -> Option<Result<Vec<Emission>, Rejection>>;
}
```

- `subjects` is every file of `CARRIED_BY`'s kinds, in `WorldPack::sectioned_files` order (items, organizations,
  places, people; key order), each with its decoded section if the file carries it.
- A resolving owner's section is **never** seeded by `SectionInContext::seed`; the resolution seeds every subject
  it names, at the position in §5.6.
- A key in the result that is not one of `subjects` is refused (`ResolveRefused`, naming the key).

### 5.5 What `SystemPack` gains (all provided; no existing pack changes)

| Item | Default | Defined by |
| --- | --- | --- |
| `const DATA: Option<DataOwnerDecl>` (`{ directories }`) | `None` | `owns_data!()` |
| `fn decode_data_file(directory, d) -> Result<Box<dyn Any + Send + Sync>, D::Error>` | refuses: "the '<id>' system owns no data directory" | `owns_data!()` |
| `fn data_context(files, configuration: Option<&dyn AuthoredConfiguration>) -> Result<Arc<dyn OwnerContext>, DataRefusal>` | refuses (never called when `DATA` is `None`) | `owns_data!()` |
| `fn decode_section_in(map, context: Option<&Arc<dyn OwnerContext>>) -> Result<Arc<dyn AuthoredContent>, A::Error>` | `Self::decode_section(map)` (the context is ignored) | `owns_section_in_context!()` (with `SECTION`, `decode_section` refusing "decoded only in its owner's context") |
| `const RESOLVES: bool` | `false` | `resolves!()` |
| `fn resolve(context, configuration, subjects: &[ErasedSubject<'_>]) -> Result<Option<Arc<dyn ResolvedSeeds>>, ResolveRefusal>` | `Ok(None)` | `resolves!()` |
| `const SHIPS: &'static [ShippedData]` (Q-S30b-1) | `&[]` | `ships_data!("attributes/x.yaml", …)` (`include_str!` from the crate's directory, with the directory and file name kept for refusals) |

`ErasedSubject` carries `Option<&Arc<dyn AuthoredContent>>`; the macro downcasts it with `AuthoredContent::as_any`
to the pack's own `Authored`.

### 5.6 The loader

**Read step 4d (after configuration, before content):**

1. **The directory rule** for the World Pack root and for every required Entity Pack's directory (F-S30b-3):
   ignore names beginning with `.`; admit the format directories (`people`, `places`, `items`, `organizations`,
   `configure`, `data` for a World Pack; `items` for an Entity Pack), the first component of `decisions:` when it
   has more than one, and every installed capability's `DATA` directories. Refuse any other as
   `PackError::UnknownDirectory { path, known }` (`known` lists every admitted name for that pack, sorted). Refuse an
   installed pack's directory in a world that does not enable it as `PackError::DataNeedsASystem { directory,
   system, path }`.
2. **Collect** per enabled data owner, in I-b-7's order: shipped files (Q-S30b-1) of every enabled pack that ships
   into this owner's directories, in `systems:` order; then each required Entity Pack in id order; then the world;
   within each source, directories in the owner's `DIRECTORIES` order and files by name (only `.yaml`; a
   subdirectory inside a data directory is refused as `UnknownDirectory`).
3. **Decode** each file with `serde_saphyr::with_deserializer_from_str(text, |d| owner.decode_data_file(dir, d))`; a
   failure is `PackError::Malformed { path, kind: "data", detail }` (line and column kept).
4. **Build the context**: `owner.data_context(files, configuration_of(owner))`; a refusal is
   `PackError::DataRefused { system, file, reason }`. An owner with no files still gets its context (from zero
   files), so its section can always be decoded.

**Content (unchanged step 5–6):** `ContentFile::new(kind, &systems, &contexts)`; the section arm calls
`owner.decode_section_in(&mut map, contexts.get(&owner))`.

**Read step 10b (after `check_sections`, before `configure::check_references`):** for each enabled capability with
`RESOLVES`, in `systems:` order, build the subjects (§5.4) and call `owner.resolve(...)`; store the result; a
refusal is `PackError::ResolveRefused { system, subject, reason }`.

**Genesis (load.rs, sections pass):** for each `(kind, key, sections)` in `sectioned_files()` order, the owners in
the world's `systems:` order (`in_composition_order`): an owner with a resolution seeds `resolved.seed(seeding, key,
id)` (if `Some`), whether or not this file carries its section; any other owner seeds its decoded section as today.
Every emission passes `seeded()`'s vocabulary check (A-10), the refusal naming the owner and the subject.

**`WorldPack`** gains `contexts: BTreeMap<Capability, Arc<dyn OwnerContext>>` and `resolved: BTreeMap<Capability,
Arc<dyn ResolvedSeeds>>`, `pub fn context(&self, owner: Capability) -> Option<&Arc<dyn OwnerContext>>` (for S30-c's
CLI), and `in_memory` sets both empty.

**Inside the loader**, collection, decoding and the directory rule are written against a `pub(crate) trait
DataOwnerOps` (`id`, `directories`, `decode_file`, `context`, `ships`) implemented for `Capability` and, in tests,
for probes — so the order, the refusals and the seeding positions are tested in-crate without installing a fixture
pack (A-14).

### 5.7 Entity Packs

- `packages::check_entity_layout`: unchanged refusals for `places/`, `people/`, `organizations/`; "declares nothing"
  now means "no `items/*.yaml` **and** no `.yaml` file in any other non-dot directory". The doc comment says why: the
  loader decides which directories are known (`ARC-121`), and this crate does not know the installed packs.
- `worldpack::entities`: the directory rule (§5.6 step 1) for each required Entity Pack; its data files collected in
  step 4d, not in step 6b.
- `validate_entity_pack(dir)` (`entities.rs:208`): also applies the directory rule with every **installed** pack's
  directories admitted, and decodes each data file with its installed owner's `decode_data_file` (file-level only;
  no context, since no world composes it). Returns the item keys as today.

---

## 6. Reuse

The seam extends MineWorld's own authoring contract (`ARC-31`, `ARC-61`); no library offers "a typed, owner-decoded
data directory inside a game content pack". `serde`'s `DeserializeSeed` is the standard mechanism for decoding with
context and is what `Decode<S>` already uses; `serde-saphyr` (`DEP-10`) keeps positions. No dependency is added.

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | `cargo fmt --check`; `cargo clippy -p mineworld-authoring -p mineworld-sdk -p mineworld-packages -p mineworld-worldpack -p mineworld-acceptance --all-targets -- -D warnings` | per commit; CI `fast`, `core` |
| Unit (`authoring`) | `DataDirectory`'s rule; `DecodeIn` hands the context to `decode` and the erased value answers owner, section, references; `seed` reaches `S::seed` with the context; `ResolvedSeeds` returns `None` for a key it was not given | `authoring/src/data.rs` tests (serde value deserializers, as `configuration.rs`' tests) |
| Unit (`sdk`) | a `Silent` pack owns no data, decodes in context as it decodes, resolves nothing, ships nothing; each macro defines its items together; `installed!` dispatches them (a local expansion, as `systems/installed/tests/installed.rs:130` does) | `sdk/rust/src/pack.rs` tests; `sdk/rust/tests/data.rs` (new) |
| Unit (`packages`) | a data-only Entity Pack declares something; one with an empty data directory does not; the forbidden three still refuse | `packages/src/manifest.rs` tests |
| Crate (`worldpack`, in-crate) | I-b-5 … I-b-7, I-b-9 with probes: collection order; the directory rule (unknown, dot, `decisions:` component, disabled owner); a malformed data file refused at its line; a context refusal named; the section decoded in context; resolution's subjects (with and without sections, no name); seeding positions; the vocabulary check on resolved seeds | `worldpack/src/data.rs` tests, `worldpack/src/load.rs` tests |
| Crate (`worldpack`, integration) | an unknown directory in a real World Pack directory refused, listing the known ones; a reference-world read unchanged | `worldpack/tests/data_directories.rs` (new); `worldpack/tests/social_cafe.rs` (unchanged) |
| Acceptance | the S30-b checkpoint: `test-schema` owns `schemas/` and a configuration; a person-file value outside a data-declared range refused at its own line and column; a valid world's resolution fills absent values from the configuration and seeds every person, with a person without the section included | `tests/acceptance/tests/data_seam.rs` |
| Regression (X-S30-9) | every reference world byte-identical | nightly `baseline check` on the final head (one dispatch, C7); the `decisions` golden (`tools/cli/tests/decisions.rs`) in the PR's CI |
| Gate 1, Gate 2 | NOT REQUIRED: no model, no rendering, no lifecycle timing | — |

---

## 8. Acceptance criteria (fixed before measuring)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-b-1 | **Checkpoint.** In `data_seam.rs`, a world composed with `test-schema` (data in `schemas/`, a `configure/test-schema.yaml`) reads through the same calls the loader makes; a person file whose `schema:` value lies outside the range a `schemas/*.yaml` file declares is refused with the file, **line and column** of the value, and the owner's message naming the range | S30-b row |
| AC-b-2 | **Resolution.** In the same world, three people (one with no `schema:` section) are resolved once; each is seeded with its value (authored, or the configuration's default); the facts appear in the people's key order, after configuration and every other section of earlier files | §4.3 item 4 |
| AC-b-3 | **Unknown directory.** A World Pack with `extras/` is refused `UnknownDirectory`, naming `extras` and listing `configure, data, items, organizations, people, places` (and any installed pack's directories; none in this build); `.git/` is ignored; a `decisions: cognition/decisions.yaml` world's `cognition/` is admitted | S30-b row |
| AC-b-4 | **Vocabulary.** A resolving probe that seeds presence's `arrived` is refused `SectionStatedAnotherPacksFact` naming the probe and the subject | S30-b row, `ARC-31` item 3 |
| AC-b-5 | **X-S30-9.** The nightly long record of the final head passes `ci_parity.py baseline check` with no world's section changed in `scripts/baselines.txt`; the `decisions` golden passes unchanged; `worldpack/tests/social_cafe.rs` passes unchanged | X-S30-9, I-S30-10 |
| AC-b-6 | **No existing pack changes.** `git diff --stat origin/main...HEAD -- systems/ tests/acceptance/tests/configuration` is empty; CI builds `acme-fishing` at its pinned revision | I-b-2 |
| AC-b-7 | **Data-only Entity Pack.** A scratch Entity Pack with `pack.yaml` and `schemas/a.yaml` only passes `check_entity_layout`; required by a world that enables an owner of `schemas`, its file is collected before the world's own | §4.3 item 1 |
| AC-b-8 | **Shipped data**. A probe owner with one shipped file and one world file receives them in that order, the shipped file's origin naming the shipping pack and its crate-relative path | Q-S30b-1 |
| AC-b-9 | I-b-3's scan is empty | I-b-3 |

---

## 9. Commit plan

Each commit tracks implementation, validation and review separately; `[x]` needs the work and its evidence; `N/A`
needs an audited reason.

### 9.0 The procedure for every commit

Work in `/Users/yuema137/mineworld-worktrees/impl-s30-b`; never `cd` elsewhere. Re-read the commit's section and
every file before editing it. Run the commit's targeted tests one `--test`/`--lib` at a time; `cargo clippy` on the
touched crates with `-D warnings`; `cargo fmt --all --check`. **R-LOAD-1:** never `cargo test --workspace` or a whole
member's suite. Mutations: copy, plant, run the named test red, restore, `cmp`, run green; record each. Update §15
and the handoff in the same commit; stage by path; commit with the message and the attribution line. No heredoc,
`sed -i` or `python3 -c`.

#### C0 — Base and ledger

- **Files:** this file (§15), `.structured-coding/plans/mvp0/handoff-s30b.md` (new).
- Steps: record the base; confirm S30-a merged (`ARC-121` in `docs/DECISIONS.md`, `MODULE_SPEC` §3's seam text);
  re-read A-1 … A-16 and record moved lines; record whether S29-a has merged (`read.rs` step 9b); Q-S30b-1 is ruled (§14.1; confirm nothing has superseded it).
- [ ] Implementation. [ ] Validation: doc scripts. [ ] Review: the seam text in `MODULE_SPEC` §3 matches §5 here;
  any difference is recorded and decided before C1.
- **Commit message:** `docs(plan): S30-b — open the ledger at the base`

#### C1 — `authoring`: the seam's contracts

- **Files:** `authoring/src/{data.rs, content.rs, configuration.rs, section.rs, lib.rs}`.
- Steps: §5.2–5.4 types and traits, the erased holders, `DecodeIn`; `as_any` on both erased traits; unit tests per
  §7.
- Validation: `cargo test -p mineworld-authoring --lib`; clippy. Mutation **M-b-1** (`DecodeIn` passes a fresh
  default context instead of the given one) → `decode_in_context_sees_the_context` red.
- [ ] Implementation. [ ] Validation. [ ] Review: no attribute word; `Subject` has no name, note or location; every
  type is `Debug + Send + Sync`.
- **Commit message:** `feat(authoring): data directories, decoding in the owner's context, and resolution before seeding (ARC-121, S30-b)`

#### C2 — `sdk`: what a System Pack declares

- **Files:** `sdk/rust/src/{pack.rs, section.rs, installed.rs, lib.rs}`, `sdk/rust/tests/data.rs` (new).
- Steps: §5.5's items and macros; the `installed!` arms; `SectionOwner::of_in_context`; tests per §7.
- Validation: `cargo test -p mineworld-sdk --lib`; `--test data`; `cargo build -p mineworld-installed-systems` (every
  installed pack, `acme-fishing` included, compiles unchanged). Mutation **M-b-2** (`decode_section_in`'s default
  returns an error) → `a_silent_pack_decodes_in_context_as_it_decodes` red.
- [ ] Implementation. [ ] Validation. [ ] Review: every new item is provided; no existing macro's expansion changes.
- **Commit message:** `feat(sdk): System Packs may own data, decode in context, resolve and ship data (S30-b)`

#### C3 — `packages`: a data-only Entity Pack

- **Files:** `packages/src/manifest.rs`.
- Steps: §5.7's first bullet and its tests.
- Validation: `cargo test -p mineworld-packages --lib`; `cargo test -p mineworld-cli --test entity_packs`
  (unchanged claims).
- [ ] Implementation. [ ] Validation. [ ] Review: the three forbidden directories still refuse first.
- **Commit message:** `feat(packages): an Entity Pack may carry data directories only (S30-b)`

#### C4 — `worldpack`: collect, decode, context; the directory rule

- **Files:** `worldpack/src/{data.rs, read.rs, content.rs, entities.rs, error.rs, lib.rs}`,
  `worldpack/tests/data_directories.rs` (new).
- Steps: §5.6 step 4d with `DataOwnerOps`; `ContentFile` in context; `WorldPack.contexts`, `context()`; the four
  error variants; `read.rs`' module doc (steps 4d, 10b) and the `DEP-10` comment (F-S30b-6); in-crate tests for the
  order, the directory rule and the refusals; the integration test file of AC-b-3.
- Validation: `cargo test -p mineworld-worldpack --lib`; `--test data_directories`; `--test social_cafe`; `--test
  refusals`; `--test entity_packs`. Mutations **M-b-3** (files collected in `read_dir` order) →
  `data_is_collected_in_composition_order` red (two files named so the filesystem orders them otherwise is not
  portable; the test instead asserts the sorted order against a probe that records it, and the mutation removes
  the sort); **M-b-4** (dot directories not ignored) → `a_dot_directory_is_not_content` red.
- [ ] Implementation. [ ] Validation. [ ] Review: no `HashMap`; the loader holds contexts erased; no data is read
  for an owner a world does not enable.
- **Commit message:** `feat(worldpack): owners' data directories, decoded in composition order, and unknown directories refused (S30-b)`

#### C5 — `worldpack`: resolution and seeding

- **Files:** `worldpack/src/{read.rs, load.rs}`.
- Steps: step 10b; the genesis pass of §5.6; in-crate tests: `resolved_seeds_take_the_sections_places` (a probe
  resolving owner listed between two section owners in `systems:`; its facts appear at each subject's position in
  `sectioned_files` order and between the two owners' facts within a file), `a_subject_without_a_section_is_resolved`,
  `a_resolution_naming_an_unknown_subject_is_refused`, the resolving `Trespasser` (AC-b-4).
- Validation: `cargo test -p mineworld-worldpack --lib`; `--test social_cafe`; `cargo test -p mineworld-cli --test
  decisions` (the 30-day golden, unchanged). Mutation **M-b-5** (resolved seeds emitted after all sections) →
  `resolved_seeds_take_the_sections_places` red; **M-b-6** (the vocabulary check skipped for resolved seeds) →
  AC-b-4's test red.
- [ ] Implementation. [ ] Validation. [ ] Review: a world with no resolving owner runs exactly today's code path.
- **Commit message:** `feat(worldpack): owners resolve every subject once, seeded where their sections were (S30-b)`

#### C6 — The checkpoint: `test-schema`, end to end

- **Files:** `tests/acceptance/tests/schema/mod.rs` (new), `tests/acceptance/tests/data_seam.rs` (new),
  `tests/acceptance/Cargo.toml` (only if a dev-dependency is missing; `serde-saphyr` is already one,
  `tests/acceptance/Cargo.toml:41`).
- Steps: `test-schema` (`OwnsData` for `schemas/`: files declaring `fields: { <name>: { min, max } }`;
  `PackConfiguration` `{ default: <n> }`; `SectionInContext` `schema: { <name>: <n> }` decoded against the declared
  ranges, refusing out of range with the range; `ResolvesSubjects` filling absent fields with the default and
  seeding `schema-stated { subject, values }`); `data_seam.rs`: AC-b-1, AC-b-2, AC-b-7, AC-b-8 (as ruled), driven
  through `serde_saphyr::with_deserializer_from_str` and the `SystemPack` methods, as the loader calls them.
- Validation: `cargo test -p mineworld-acceptance --test data_seam`. Mutation **M-b-7** (`decode` ignores the context
  and accepts any integer) → AC-b-1's test red.
- [ ] Implementation. [ ] Validation. [ ] Review: `test-schema` names no installed pack's vocabulary; the test calls
  only what the loader calls.
- **Commit message:** `test(acceptance): a test-only pack owns data, is checked at its line, and resolves (S30-b)`

#### C7 — Close-out

- **Files:** this file (§15), the handoff, `docs/MODULE_SPEC.md` (the "implemented" marks; a dated correction if any,
  R-S30a-b-1), `docs/DECISIONS.md` (a dated note under `ARC-121` only if C1–C6 found a correction).
- Steps: I-b-3's scan; AC-b-6's diff; push `mvp0/pr-s30b`; `gh pr create`; CI on the exact head (three OSes) green;
  one `nightly.yml` dispatch on the final head (the PR's CI does not run the long profile; test rules §10 item 3)
  and AC-b-5's verdict.
- [ ] Implementation. [ ] Validation: CI and the nightly verdict. [ ] Review: deviations recorded.
- **Commit message:** `docs(plan): S30-b close-out`
- **Then:** READY FOR OPERATOR REVIEW — DO NOT MERGE.

---

## 10. Requirements this PR places on other PRs

- **R-S30a-b-1 (S30-a's text).** If C1–C6 change a sentence of `MODULE_SPEC` §3's seam text, C7 corrects it with a
  dated note, never silently.
- **R-S30c-b-1 (S30-c).** `attributes` implements `OwnsData` (`attributes/`, `populations/`, `masks/` …),
  `SectionInContext` (`attributes:`), and `ResolvesSubjects` (the sampler at genesis, S30-d); `mineworld attributes
  catalogue` reads `WorldPack::context(Capability::Attributes)`.
- **R-S30c-b-2 (S30-c).** A resume-time check that a world's data (its catalogue's identity) has not drifted from the
  save's, in the manner of `check_configuration` (`ARC-61` item 7), since this seam's contexts are not facts.
- **R-S31-b-1 (S31).** The notices pack ships `attributes/subscriptions.yaml` and
  `vocabularies/topics.yaml` with `ships_data!`; `economy` ships `attributes/wealth.yaml`.

## 11. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-30-person-attributes.md` §4.3 | item 3 worded as semantics (F-S30b-1); item 1's admissions (dot directories, `decisions:`'s directory, disabled owner, F-S30b-3); a new item 5 for shipped data per Q-S30b-1. **Applied in PR #234, 2026-10-15** |
| `step-30-person-attributes.md` §8.5 item 5 | "The sampler's input type" is the seam's `Subject` (F-S30b-5) |
| `overall.md` | S30-b frozen / merged |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S30-b — the authoring seam (data directories, decoding in context,
                         resolution before seeding, shipped data)
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s30b-authoring-seam.md
RELATED / BINDING DOCS:  step-30-person-attributes.md (§§3, 4.3, 8.5, 11.2, 12, 13, 14); pr-s30a-attribute-specs.md;
                         docs/MODULE_SPEC.md §§2, 3.1, 4.1; docs/DECISIONS.md ARC-31, ARC-33, ARC-36, ARC-61, ARC-71,
                         ARC-121, DEP-10; CLAUDE.md; docs/ENGINEERING_STANDARDS.md; .structured-coding/standards.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s30-b, its own, held by one session
BRANCH:                  mvp0/pr-s30b, created from origin/main
START CONDITION:         S30-a merged AND this design frozen, then the primary's dispatch (step-30 §13.3)
IMPLEMENTATION BASE:     origin/main at C0; C0 records it
APPROVED SCOPE:          §2.1 items 1–6, as §14's ruling shapes item 2 (SHIPS) and §5.6
FROZEN INVARIANTS:       step-30 I-S30-10; §4 I-b-1 … I-b-9; §8 as written
SEQUENCE:                C0 … C7 (§9)
IMPLEMENTER:             Opus-class (step-30 §13.1; R-S30-4); reviewer: the primary session
ALLOWED COMMANDS:        cargo build/test/clippy/fmt -p <crate> (never --workspace); git; gh (pr create/view;
                         workflow run nightly.yml once in C7; run download; never merge); python3
                         scripts/check_doc_headings.py, check_decision_ids.py, ci_parity.py baseline check;
                         cp; cmp; mkdir -p; sed -n; grep
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace (R-LOAD-1);
                         editing any impl of AuthoredSection, PackConfiguration or SystemPack in systems/ or
                         tests/acceptance/tests/configuration; any world edit; any model call
MATERIAL STOPS:          S30-a not merged; a needed required item on AuthoredSection or SystemPack; any reference
                         world's bytes moving (X-S30-9); acme-fishing failing to build; a seam change that needs an
                         attribute word; a new external dependency
PLATFORMS:               Linux, macOS, Windows (CI)
VALIDATION BUDGET:       crate and acceptance tests unrestricted one at a time; one nightly dispatch (C7). Gate 1 and
                         Gate 2: NOT REQUIRED
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s30b.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                                                     (task brief 2026-10-15: "endpoint: PR ready
                                                                      for operator review; no merge")
                         semantic commits: authorized                (working rules §14)
                         branch push: authorized                     (working rules §21)
                         PR creation / update: authorized            (working rules §21)
                         CI repair to review readiness: authorized   (working rules §21)
                         endpoint: the PR READY FOR OPERATOR REVIEW
                         merge: NOT authorized — only with the operator's explicit authorization (working rules
                         §22)
POST-MERGE SYNC OWNER:   primary: step-30, overall.md; implementation session: this ledger and the merge identity
STOP CONDITION:          S30-b READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-b-1 | The seam touches the loader every world uses (step-30 R-S30-4) | additive; a world with no data owner runs today's path; X-S30-9 by the nightly record; Opus implementer |
| R-b-2 | The unknown-directory rule refuses a user's world that keeps notes in a folder | the refusal lists the admitted names and says that dot directories are ignored; MODULE_SPEC §4.1 states the rule; the release notes (S23) mention it |
| R-b-3 | Type erasure hides an owner's downcast mistake until run time | every downcast failure is a named refusal, and `test-schema` exercises every erased path in CI |
| R-b-4 | S29-a and this PR both edit `read.rs`' step list and `read_with` | separate steps (9b, 4d/10b); whichever merges second rebases; both designs name it |
| R-b-5 | Shipped data (Q-S30b-1) makes a System Pack's crate a source of world content | it is compiled in, versioned with the pack and named in every refusal by pack and path; a world sees it only by enabling the pack |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S30b-1 [operator — the seam's shape, `ARC-121`]** | Step-30 §9.6 N-2 and step-31 §7.4 have S31's notices pack and `economy` declare their attributes "in its pack's `attributes/`", but a System Pack is a Rust crate and the loader reads data only from World Packs and Entity Packs (F-S30b-2). How does a System Pack contribute declarations to an owner's directory? | **Ship them compiled in: `SystemPack::SHIPS` via `ships_data!("attributes/subscriptions.yaml", …)`**, collected first (in `systems:` order) whenever the shipping pack is enabled, each file named in refusals by pack and crate-relative path. A pack's declarations then travel with its code and its version, and a world that enables `notices` cannot forget them. The alternative — a companion Entity Pack per System Pack (`entities/notices-attributes`) that every world must also require — needs no seam change but splits one capability across two packages, and a world that enables `notices` without requiring its companion fails only at a reader's install (`MissingUnderBackend`-like), far from the cause |

### 14.1 Rulings, 2026-10-15 (relayed by the coordinator; binding)

| Question | Ruling | By |
| --- | --- | --- |
| Q-S30b-1 | **Declarations are compiled in through `SystemPack::SHIPS` / `ships_data!`**, collected first in `systems:` order whenever the shipping pack is enabled. Applied: §1 item 4, §5.5, §5.6 step 2, AC-b-8, R-S31-b-1, step-30 §4.3's dated note (item 5) | Ruled 2026-10-15 (operator answered directly in the primary session) |
| Freeze | "同意冻结并实施 S30-a/b" | Operator, directly in the primary session, 2026-10-15 |

No question remains open.

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 … C7 | — | [ ] | [ ] | [ ] |

### Evidence

(none yet)

### Mutations

(none yet; M-b-1 … M-b-7 as §9 lists them)

### Deviations and findings

F-S30b-1 … F-S30b-6 (§3.2), recorded at design time.
