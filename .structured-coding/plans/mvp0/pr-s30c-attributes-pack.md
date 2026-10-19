# PR S30-c — The `attributes` System Pack and the attribute model crate

## DESIGN FROZEN — 2026-10-18 (operator approved in the primary session: freeze with the recommended answers)

```text
Design revision:        revision 1 (2026-10-17), FROZEN 2026-10-18
Author:                 S30-c / S30-d / S30-e PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s30cde, branch docs/pr-designs-s30cde,
                        planning base origin/main @ 38323d4c (#238, S30-a merged)
Approved by / evidence: §15 ruled 2026-10-18 by the operator in the primary session: Q-S30c-1 and Q-S30c-2
                        as recommended; the S30-b review follow-ups added as §2.1 item 7
Implementation base:    main (origin/main at C0); C0 records the commit and that S30-b (#247) has merged
Start condition:        S30-b merged (met: #247, merge 04d5145e; its branch mvp0/pr-s30b @ 40af5a11 was read
                        for this design), this design frozen (met: 2026-10-18), §15's questions ruled (met:
                        2026-10-18), the primary's dispatch
Execution contract:     §13
Implementer:            Opus-class (§13); the step's "Opus design → Haiku implementation" is not taken,
                        because C1 extends the seam every world loads through (R-c-1)
Lifecycle:              DESIGN FROZEN 2026-10-18
```

**Effort:** `mvp0` · **Step:** [`step-30-person-attributes.md`](step-30-person-attributes.md) (APPROVED 2026-10-15;
amended; notes of #234 and #238) §§0, 3 (A-1 … A-19, F-S30-1 … F-S30-11), 4 (`ARC-120`, SD-S30-1 … 3), 5 (`ARC-122`),
7, 9.1, 9.4, 9.6, 11, 12 (I-S30-1 … I-S30-5, I-S30-8, I-S30-10), 13.1 (the S30-c row), 13.4, 13.5 · **Specified by:**
[`pr-s30a-attribute-specs.md`](pr-s30a-attribute-specs.md) (MERGED, #238): `docs/CORE_CONCEPTS.md` §4.5,
`docs/MODULE_SPEC.md` §§2, 3.1, 4.1 "Attributes", `docs/DECISIONS.md` `ARC-120` … `ARC-126` · **Builds on:**
[`pr-s30b-authoring-seam.md`](pr-s30b-authoring-seam.md) (FROZEN; implemented on `mvp0/pr-s30b`, PR #247) ·
**Parent:** [`overall.md`](overall.md) · **Followed by:** S30-d (profiles, the sampler), S30-e (the built-in
catalogue, backends and masks), S30-f (consumers) · **Binding parents:** `CLAUDE.md` §§2, 4 (items 1, 2, 3, 5, 7, 8,
9, 10, 11, 12, 13); `docs/CORE_CONCEPTS.md` §§3.1, 4.5, INV-4, INV-7, INV-12, INV-13, INV-15; `docs/DECISIONS.md`
`ARC-25`, `ARC-26`, `ARC-28`, `ARC-31`, `ARC-61`, `ARC-120` … `ARC-125`, `DEP-10`.

**Decision ids.** None new. This PR implements `ARC-120` (ownership), `ARC-122` (the model) and `ARC-124` (the owner
interface), and writes dated notes under `ARC-120` (R-S30c-a-1, §15 Q-S30c-2), `ARC-121` (seam item 5, §15
Q-S30c-1) and `ARC-122` (the forms §5 fixes). `ARC-127` … `ARC-129` and `DEP-69` stay reserved and unused.

No model is called. No reference world's bytes move.

---

## 0. Binding inputs, exactly restated

- **Scope (step-30 §13.1, S30-c row).** "`authoring/attributes/` (catalogue, kinds, values, handles,
  `AttributeOwner`, renderer, filter); `systems/attributes/` (section, `traits`, facts, proposals, disclosure,
  derived age); `mineworld attributes catalogue/show/check`; a minimal test catalogue."
- **Integration checkpoint (same row).** "`mineworld server` on a fixture world: Alice's own observation lists her
  attributes, Bob's lists Alice's perceivable ones only; a save/restart keeps them; a proposal changes an
  `evolving` trait with a cited cause."
- **Adversarial criteria (same row).** X-S30-1 … X-S30-4, X-S30-7, X-S30-8, X-S30-10, X-S30-11 (step-30 §13.5).
- **Requirements placed on this PR.**
  - *R-S30c-a-1* (`pr-s30a-attribute-specs.md` §10; F-S30a-10): "Decide how a world that enables an owner pack but not
    `attributes` reads the owner's shipped declarations, and record it as a dated note under `ARC-120`."
  - *R-S30c-b-1* (`pr-s30b-authoring-seam.md` §10): "`attributes` implements `OwnsData` (`attributes/`,
    `populations/`, `masks/` …), `SectionInContext` (`attributes:`), and `ResolvesSubjects` (the sampler at genesis,
    S30-d); `mineworld attributes catalogue` reads `WorldPack::context(Capability::Attributes)`."
  - *R-S30c-b-2* (same): "A resume-time check that a world's data (its catalogue's identity) has not drifted from the
    save's, in the manner of `check_configuration` (`ARC-61` item 7), since this seam's contexts are not facts."
- **Invariants that bind (step-30 §12).** I-S30-1 one owner; I-S30-2 derived never stored; I-S30-3 private/hidden never
  in another seat's observation; I-S30-4 every value checked; I-S30-5 handles only, never `text`; I-S30-8 kernel and
  contracts ignorant; I-S30-10 a world without the pack byte-identical.
- **Operator rulings that bind** (step-30 §§13.2, 16.1, 16.2; S30-a §14.1): sensitive attributes are `private` by
  default (Q-S30-8); appearance descriptors are `perceivable` (Q-S30-1, `ARC-125`); `natal-sex` is a built-in
  (Q-S30a-1, S30-e's data); `employment.occupation` comes in S30-f (Q-S30a-2); the wealth band is `economy`'s (Q-S30-6,
  S31); populations live in a world-required data pack (Q-S30-2, S30-d/e); at most 100 built-ins (RR-S30-3, S30-e).
- **R-LOAD-1.** Locally only the targeted test files of a commit and `cargo clippy` on touched crates; never
  `cargo test --workspace`; CI on the exact head is the gate.

---

## 1. Goal

Make a Person's attributes real, end to end, for data the world declares — with no profile and no sampler yet:

1. **A model crate** (`mineworld-attributes-model`, `authoring/attributes/`) that every user of attributes depends on:
   declarations decoded and validated with line and column; one frozen `Catalogue` per world; the closed value enum;
   typed handles; the renderer; the visibility filter; derived `age` and `band`; the `AttributeOwner` interface; the
   pack's component and fact types, so that owners and readers depend on this crate and never on the pack.
2. **The `attributes` System Pack** (`systems/attributes/`): owns the `attributes/` and `vocabularies/` directories,
   the person file's `attributes:` section, the `traits` component and the four facts; resolves each person's values
   (authored, else the declaration's `default`) and hands owner-declared values to their owners; checks proposals;
   discloses one observer-filtered `traits` component.
3. **The running world knows its catalogue** through one genesis fact, `attributes-composed`, which a resumed world is
   checked against (R-S30c-b-2), by one general addition to the seam (§5.11, Q-S30c-1).
4. **Three commands:** `mineworld attributes catalogue | show | check`.

Every reference world stays byte-identical; none enables the pack (I-S30-10, X-S30-9).

---

## 2. Scope

### 2.1 In scope

1. **The seam, item 5 (general; §5.11).** `authoring/src/data.rs`: `OwnsData::CONTEXT_FACTS` and
   `OwnsData::seed_context` (provided, default none); `sdk/rust/src/{pack.rs, installed.rs}`: the `SystemPack` items
   and `installed!` arms (`context_facts`, `seed_context`), defined by `owns_data!()`; `worldpack/src/load.rs`: the
   context facts seeded once per enabled data owner, after configuration, before sections; `worldpack/src/configure.rs`:
   `check_configuration` also compares context facts. Names no domain.
2. **The model crate** `authoring/attributes/` (new member in the root `Cargo.toml`): §§5.2–5.10.
3. **The System Pack** `systems/attributes/` (new; `systems/*` is already a member glob): §§5.5–5.10; installed by one
   line in `systems/installed/Cargo.toml` and one in `systems/installed/src/lib.rs` (`ARC-33`).
4. **The CLI** `tools/cli/src/attributes.rs` (new) and one `Subcommand::Attributes` variant in `tools/cli/src/main.rs`:
   §5.12.
5. **Fixtures and tests:** a minimal test catalogue and fixture world `tools/cli/tests/fixtures/attributes/`; the
   test-only packs `test-early-bird` (a reader that proposes) and `test-badge` (an owner) under
   `tests/acceptance/tests/attributes/`; the structural scans (§5.13).
6. **Specification sync (dated notes; `CLAUDE.md` §2.1 rule 4):** `docs/MODULE_SPEC.md` §§3.1, 4.1, 8.1;
   `docs/DECISIONS.md` notes under `ARC-120`, `ARC-121`, `ARC-122` (§5.14).
7. **The S30-b review follow-ups (operator ruling, 2026-10-18), in C1:** (a) rename `ResolvesSubjects::resolve` to
   `resolve_subjects` (`authoring/src/data.rs` and every caller), matching the SDK's `resolve_subjects` (S30-b D-b-6);
   (b) add the R3b ordering test named by the S30-b review.

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| `populations/`, profiles, the sampler, the lock, roles, `configure/attributes.yaml`, `resolve`, `lock` | S30-d |
| The built-in catalogue (93 `core` rows), backends, masks, `backend.yaml`, `attributes backends` | S30-e |
| Cognition, the SDK reader for decision models, MCP, both clients' panels; `employment.occupation` | S30-f |
| Any reference world adopting attributes; re-baselines; S26's switch | S30-g |
| `economy.wealth-band`, `notices.subscriptions` | S31 |
| `mappings/`, `labels/` | S30-h, S30-k |
| Any change under `kernel/`, `contracts/`, `server/`, `clients/`, `cognition/`, `worlds/`, `systems/<existing>/src` | never |

### 2.3 Dependencies

S30-a merged (yes, #238). **S30-b merged** (#247; OPEN when this was written; merged since, 04d5145e): every type of §3.1's A-c-1 … A-c-6
exists only on `mvp0/pr-s30b` today. Not required: S26-a (#206), P6a (#236), S31.

---

## 3. Audit (`origin/main @ 38323d4c`; S30-b at `origin/mvp0/pr-s30b @ 40af5a11`; 2026-10-17)

### 3.1 Anchors

| # | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-c-1 | `OwnsData { DIRECTORIES; type File; type Context; decode_file; context(files, configuration: Option<&dyn Any>) }` | `mvp0/pr-s30b:authoring/src/data.rs:163-195` | `attributes` decodes `attributes/` and `vocabularies/` files and builds the world's `Catalogue` as its context |
| A-c-2 | `SectionInContext { SECTION; CARRIED_BY; type Authored; decode(context, d); references; seed }`; `seed` is not called for a resolving owner | `…/data.rs:295-334` | the `attributes:` section is decoded against the catalogue at the value's line and column (X-S30-1 … 3) |
| A-c-3 | `ResolvesSubjects { type Seed; resolve(context, configuration, subjects); seed_resolved }`; `Subject { key, kind, tags, authored }` — no name, note or location | `…/data.rs:420-483` | the resolution in S30-c applies "authored, else default" and is where S30-d adds the sampler |
| A-c-4 | `ShippedData::new(path, text)` — compiled-in text with its path; `Capability::shipped()`; files shipped into a directory whose owner the world does not enable are **not collected** (S30-b F-b-1 (a)) | `…/data.rs:608-680`; `mvp0/pr-s30b:sdk/rust/src/installed.rs:337`; S30-b ledger F-b-1 | the base of R-S30c-a-1 (§5.10) |
| A-c-5 | Genesis order: passages, locations, `configure::seed` (`load.rs:380`), then sections and resolutions per file in `systems:` rank (`in_composition_order`, `load.rs:412`); `own_vocabulary` refuses another pack's fact (`load.rs:87`) | `mvp0/pr-s30b:worldpack/src/load.rs:80-130, 360-440` | the context facts (§5.11) take one fixed position: after configuration, before sections |
| A-c-6 | `WorldPack::context(&self, owner: Capability) -> Option<&Arc<dyn OwnerContext>>`; `OwnerContext::as_any()` answers the owner's own `Context` | `mvp0/pr-s30b:worldpack/src/read.rs:351`; `…/data.rs:149-160` | the CLI downcasts to `Catalogue` |
| A-c-7 | `check_configuration(saved_genesis)` assembles the world and `compare`s, in order, the facts whose event type is a configuration fact of an enabled capability (`configuration_facts()`); callers `tools/cli/src/run.rs:330`, `serve.rs:421`, `main.rs:609` | `worldpack/src/configure.rs:478-552` | R-S30c-b-2 is met by widening the type set compared, not by a new call site |
| A-c-8 | `PackConfiguration::seed` receives `ConfigurationContext` (classes, attachments), not the owner's data context | `worldpack/src/configure.rs:409-447` | a configuration fact cannot carry the catalogue (F-S30c-1) |
| A-c-9 | "A system holds no fields, because its mutable state is the components it owns and those live in the world"; `install` and `provider` construct a pack with `Default` | `mvp0/pr-s30b:sdk/rust/src/installed.rs:416-452`; `worldpack/src/load.rs:196-206` | the catalogue reaches the running pack only through world state (F-S30c-1) |
| A-c-10 | World-level configured state lives in a Process: `weather-configured` → `ClimateProcess`; a 66 KB series → `RecordProcess`, "encoded once" | `systems/weather/src/process.rs:1-40`; `system.rs:229-245` | the catalogue's runtime home is a Process (§5.5) |
| A-c-11 | `ProcessKind { PROCESS_TYPE; type Owner: SystemIdentity }`; `Process::state_for::<P>()` checks only `PROCESS_TYPE` | `kernel/src/process.rs:70-75, 162-172` | readers outside the pack read the catalogue Process through a model-crate kind generic over its owner (§5.5) |
| A-c-12 | `owned_component!` needs the owner *type* and implements `OwnedBy<Owner>`; `Component` is a plain trait of `contracts` | `kernel/src/access.rs:262-279`; `contracts/src/component.rs:91-98` | `Traits` is defined in the model crate with `Component` implemented by hand; the pack implements `OwnedBy<AttributesSystem>` (orphan-legal) |
| A-c-13 | `PerceptionProvider::discloses(world: &WorldRead, observer, subject) -> Vec<ComponentRecord<Value>>`, called once per perceived entity including the observer; self-only disclosure compares the two (`economy`) | `systems/presence/src/interaction.rs:97-130`; `systems/economy/src/system.rs:240-262` | the visibility filter is "observer == subject → holder view, else other view" |
| A-c-14 | `WorldRead::at()` is the read's world instant | `kernel/src/view.rs:88-96` | derived `age` needs no calendar (step-30 §4.5) |
| A-c-15 | `WorldTime(i64)`, seconds, negative before the epoch | `contracts/src/time.rs:29-42` | `instant` values are `i64` seconds |
| A-c-16 | `SystemDeclaration::depending_on([SystemId])` names a dependency by id only; subscribing needs no dependency (`ARC-28` item 2) | `kernel/src/system.rs:141`; `docs/DECISIONS.md` `ARC-28` | readers and proposers depend on `attributes` by id; owners subscribe to `attributes-handed` without depending |
| A-c-17 | `Visibility::{Public, Place, Participants, Entities(BTreeSet), SystemInternal}` | `contracts/src/event.rs:296-310` | `traits-stated` is `Entities({holder})`; `attributes-composed`, `attributes-handed`, `attribute-change-proposed` are `SystemInternal` |
| A-c-18 | Decimals cross the YAML boundary as `f64`; calendar rounds `degrees × 10⁶` to integers at the boundary | `systems/calendar/src/configuration.rs:120-130` | a quantity is decoded once to an exact integer at its scale (§5.4), never compared as a float |
| A-c-19 | The workspace lists `authoring` explicitly and globs `systems/*` | `Cargo.toml:2-25` | the model crate is one new `members` line; the pack needs none |
| A-c-20 | The installed set is one `installed!` list and its manifest | `systems/installed/src/lib.rs:37-59`; `systems/installed/Cargo.toml` | installing `attributes` is two lines |
| A-c-21 | S30-b's directory-rule tests pin the admitted names: `"configure, data, items, organizations, people, places"` and `"items"` | `mvp0/pr-s30b:worldpack/tests/data_directories.rs:69-71, 111` | installing a data owner changes those two expected strings — an expected, reviewed edit in C4 |
| A-c-22 | The CLI is one `clap` `Subcommand` enum; each command is a module | `tools/cli/src/main.rs:99-300`; `tools/cli/src/packs.rs` | `attributes.rs` with `catalogue`, `show`, `check` |
| A-c-23 | CI's `core` layer runs `cargo test --workspace` on Linux (container), Windows and macOS | `scripts/ci_layer.py:70-75`; `.github/workflows/ci.yml:130-170` | every test here runs on three platforms without a CI edit |

### 3.2 Findings

- **F-S30c-1. The running pack cannot see the catalogue.** The catalogue is built at load as `attributes`' data context
  (A-c-1), but a System is constructed with `Default` and holds no fields (A-c-9); a configuration fact is seeded
  without the data context (A-c-8); a resolution seeds per subject only (A-c-3). Disclosure, rendering, derived values
  and proposal checks all need the declarations at run time. The tree's idiom for world-level data at run time is a
  genesis fact folded into a Process (A-c-10). **Q-S30c-1:** a general seam item 5 lets a data owner state
  world-level facts from its context (§5.11); `attributes` states one, `attributes-composed`, holding the frozen
  catalogue; the same item makes R-S30c-b-2's drift check generic.
- **F-S30c-2. "At install" cannot be literal for reads.** `MODULE_SPEC` §3.1 says a reader's handle is refused "at
  install"; install receives no world data (A-c-9). The check moves to **load**, where the catalogue exists, which is
  earlier and stricter. A pack states what it reads and proposes as **data it ships** (`uses:`, §5.9); `attributes`'
  context refuses `Missing`, `WrongKind` (and, from S30-e, `MissingUnderBackend`) naming the pack. Dated note in
  `MODULE_SPEC` §3.1 (C8).
- **F-S30c-3. Owners without `attributes` (R-S30c-a-1).** Shipped files into a disabled owner's directory are not
  collected (A-c-4). Two designs are possible; §5.10 and **Q-S30c-2** recommend: an owner's attributes exist only in a
  world that enables `attributes`; the owner's own state and actions work either way.
- **F-S30c-4. The pack's vocabulary must live in the model crate.** Owners decode `attributes-handed` and readers read
  `traits` without depending on the pack (`ARC-124`); the types are therefore the model crate's, with `Component` and
  `Event` implemented by hand against the id `attributes` (A-c-12), and a test holds that id equal to
  `AttributesSystem::ID`.
- **F-S30c-5. A quantity's text and value.** `MODULE_SPEC` §4.1 says a quantity "renders at its scale with its unit",
  while its example phrase is `"{value} m tall"`. Decided: `{value}` renders the number at exactly its declared scale
  (`1.660`); the unit is written by the phrase. Dated note in §4.1 (C8).
- **F-S30c-6. A flag and an empty set render as nothing unless worded.** A phrase has exactly one `{value}`. A `flag`'s
  default words are `true: <label in lower case>`, `false: ""`; an empty `set` and an empty word render no text
  (`text` absent in the disclosed entry). Dated note in §4.1 (C8).
- **F-S30c-7. `owner:` and `OwnerNotInstalled`.** With shipped declarations (Q-S30b-1), an owner declares its own
  attributes; a world or Entity Pack naming another owner could never be projected by it. Decided: `owner:` may be
  `attributes` or the shipping System Pack itself; a data pack or world naming a pack that is not enabled is
  `OwnerNotInstalled`, one naming an enabled pack is `BadDeclaration { field: owner }` ("only <pack> declares the
  attributes it owns").
- **F-S30c-8. Derived age without a calendar.** `age { from }` is whole years at the read's instant (A-c-14) by the
  mean Gregorian year, `floor((at − birth) × 400 / (146 097 × 86 400))`, integer only. A birthday-exact age needs a
  calendar and is not MVP-0 (step-30 §4.5 "Kernel time suffices").

### 3.3 What does not exist

No attribute type, catalogue, component, fact, command or pack. No reader or owner of an attribute. No world with an
`attributes/` directory. `persona` (P6a) and `appearance` (S26-a) are not on `main`.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`; `CLAUDE.md` §4 rule 16)

- **Adopted, already in the tree:** `serde` with `DeserializeSeed` (decoding against a declaration, the same
  mechanism as `Decode<S>`), `serde-saphyr` (`DEP-10`, line and column), `serde_json` (payloads), `clap` (the CLI).
- **Considered and declined:** a JSON-Schema validator (`jsonschema`) for declarations — it validates shape, not this
  model's cross-file rules (namespaces, overrides, vocabularies), reports JSON pointers instead of YAML positions, and
  would be a second schema language beside the Rust types; a fluent-template engine (`handlebars`, `tera`) for
  phrases — a phrase has exactly one placeholder, which is a `str::replacen`, and a template language would admit
  logic in data. No dependency is added; no `DEP` record is needed.
- **Not forced:** the attribute model sits on MineWorld's own seam (`ARC-121`), not on a library's execution model.

---

## 5. Design

### 5.1 Module layout

```text
authoring/attributes/                 mineworld-attributes-model (new workspace member)
  Cargo.toml                          deps: mineworld-contracts, mineworld-kernel, mineworld-authoring, serde,
                                      serde_json, serde-saphyr, thiserror
  src/lib.rs                          module map, re-exports, ATTRIBUTES: SystemId = "attributes"
  src/id.rs                           Namespace, Slug, QualifiedId (parse, short-slug resolution)
  src/kind.rs                         Kind (closed), Unit (closed), Domain, ValueSlug, the marker kinds and
                                      SystemReadable (Category, Level, Quantity, Flag, Set, Instant; not Text)
  src/value.rs                        AttributeValue, ScaledInt, decimal decoding (§5.4)
  src/declaration.rs                  Declaration (decoded), Visibility, Mutability, Derive, PerceivedAs, the file
                                      formats (attributes:, overrides:, uses:, vocabularies:) with line/column
  src/catalogue.rs                    Catalogue (frozen), its build from files (namespaces, duplicates, overrides,
                                      vocabularies, uses), AttributeError (every refusal of §5.3)
  src/section.rs                      the person section's DeserializeSeed (values and restrictions)
  src/traits.rs                       Traits (component), Handle<K>, reads
  src/derive.rs                       age, band; projection hand-off
  src/render.rs                       phrases and words; the holder and observer views; the disclosed payload
  src/owner.rs                        AttributeOwner; disclose_owned
  src/facts.rs                        AttributesComposed, TraitsStated, TraitChanged, AttributeChangeProposed,
                                      AttributesHanded; CatalogueProcess<O>
  tests/                              structure.rs (X-S30-10, AC-S30-12), compile_fail (X-S30-11), golden render
systems/attributes/                   mineworld-attributes (new System Pack)
  src/{lib.rs, system.rs, data.rs, section.rs, resolve.rs, reduce.rs, disclose.rs}
  tests/{pack.rs}
tools/cli/src/attributes.rs           mineworld attributes catalogue | show | check
tools/cli/tests/{attributes.rs, fixtures/attributes/…}
tests/acceptance/tests/{attributes_seam.rs, attributes/{mod.rs, early_bird.rs, badge.rs}}
authoring/src/data.rs, sdk/rust/src/{pack.rs, installed.rs}, worldpack/src/{load.rs, configure.rs}   seam item 5
```

### 5.2 Identifiers, kinds and values (`ARC-122` items 1–2)

- **`QualifiedId`** = `Namespace` `.` `Slug`. A namespace is a pack id (`PACKAGE_FORMAT.md` §5.0: no `.`); the slug
  matches `^[a-z][a-z0-9_.-]{0,63}$` and may contain `.` (`skill.cooking`), so a qualified id splits at its **first**
  `.`. `core` is reserved for `entities/person-attributes` (S30-e).
- **`Kind`** (closed): `Category(Domain)`, `Level(Domain)`, `Quantity { unit, scale: u8 (0..=6), min: i64, max: i64 }`,
  `Flag`, `Set { domain, max: u16 }`, `Text { max_bytes: u16 (≤ 280) }`, `Instant`. `Domain` is a closed list of
  `ValueSlug`s or a `VocabularyRef` resolved at catalogue build. `Unit` (closed): `m`, `kg`, `year`, `day`, `hour`,
  `count`, `ratio`.
- **`AttributeValue`** (closed): `Category(ValueSlug)`, `Level { slug, rank: u16 }`, `Quantity(i64)` (at the
  declaration's scale), `Flag(bool)`, `Set(BTreeSet<ValueSlug>)`, `Text(BoundedText)`, `Instant(WorldTime)`. A level
  carries its rank so a reader compares without the catalogue; the rank cannot drift inside a composition because
  reordering values is a major version (`ARC-122` versioning) and a resumed world is checked for drift (§5.11).
- **Serialized form** (facts, component, disclosure, CLI): externally tagged by kind — `{"level":"early-bird"}`,
  `{"quantity":"1.660"}` (the exact decimal string at the scale), `{"flag":true}`, `{"set":["a","b"]}`,
  `{"text":"…"}`, `{"instant":-1072915200}`, `{"category":"auburn"}`. No float appears anywhere.

### 5.3 Declarations and the catalogue

**Files of `attributes/`** (`format: 1`; any of three top-level maps):

```yaml
format: 1
attributes:            # declarations in the declaring pack's namespace (MODULE_SPEC §4.1 "Attribute declarations")
  chronotype: { group: behaviour, kind: { level: [early-bird, …] }, visibility: learnable, mutability: evolving,
                label: Chronotype, phrase: "{value}" }
overrides:             # MODULE_SPEC §4.1 "Overrides"
  core.hair-colour: { words: { auburn: copper }, visibility: private }
uses:                  # §5.9: only in a file a System Pack ships
  reads: { core.chronotype: level }
  proposes: [core.hair-colour]
```

**Files of `vocabularies/`**: `format: 1`, `vocabularies: { <name>: [<value id>, …] }`; a vocabulary is
`<declaring pack>.<name>`.

**The build** (`Catalogue::build(files: Vec<(DataOrigin, AttributesFile)>) -> Result<Catalogue, AttributeError>`), in
collection order (S30-b I-b-7: shipped, Entity Packs by id, world):

1. Each file is decoded by `decode_file` with `#[serde(deny_unknown_fields)]`; field rules (§5.2, the slug pattern, a
   phrase with exactly one `{value}` and no other `{`, `derived` needs `derive:`, `text` ≤ 280) are checked inside the
   field's own deserializer so a refusal keeps its line and column (`BadDeclaration`).
2. The **namespace** of a file is its origin's pack id (`DataOrigin.pack`; a World Pack's is its world id). A
   declaration whose id carries a namespace other than its file's is `ForeignNamespace`. `core` is accepted only from
   the pack whose id is `person-attributes` (S30-e names it; S30-c's tests use a fixture namespace).
3. **Duplicates** across files are `DuplicateAttribute` (both files named). **Owners** per F-S30c-7.
4. **Vocabularies** are collected first; a `vocabulary:` naming none is `UnknownTopic` naming the nearest (§5.3's
   nearest = the three smallest Levenshtein distances, ties by id; one small function in `catalogue.rs`).
5. **Overrides** apply in collection order to declarations from other files, by the table of `MODULE_SPEC` §4.1
   (`OverrideNotAllowed`, naming the field); an override of an unknown id is `UnknownAttribute` with the nearest.
6. **`uses:`** (§5.9) are checked last, against the frozen result.
7. The result is **frozen**: `Catalogue { attributes: BTreeMap<QualifiedId, Declaration>, vocabularies, short:
   BTreeMap<Slug, Vec<QualifiedId>>, uses: BTreeMap<SystemId, Uses> }`, with `Catalogue::resolve(id_or_short) ->
   Result<&Declaration, AttributeError>` refusing `UnknownAttribute { nearest }` and `AmbiguousSlug { candidates }`.

Every refusal is one variant of `AttributeError` named as in `MODULE_SPEC` §4.1's table, carrying the file and, for a
decode, the line and column; `attributes`' `context` maps it to S30-b's `DataRefusal { file, reason }`.

### 5.4 The person section and values (`ARC-122`, MODULE_SPEC §4.1 "The person file's section")

`attributes:` in `people/<key>.yaml` maps an id (qualified, or short where unique) to a **value** or a
**restriction** (`{ allowed: [...] }` for `category`, `level`, `set`; `{ between: [min, max] }` for `quantity`). S30-c
decodes and stores restrictions (`Authored { values, restrictions }`); only S30-d's sampler reads them, and with no
profile a restriction on an attribute with no value is reported by `attributes check` as "restricted, never drawn".

- Decoding is a `DeserializeSeed` per entry (`section.rs`): the key resolves through the catalogue (`UnknownAttribute`
  with the nearest; `AmbiguousSlug`), then the value decodes by the declaration's kind, refusing `WrongKind`
  (printing the domain), `OutOfDomain`, `UnknownTopic`, `OutOfRange` and a stated `derived` value (`OutOfDomain`) at
  the value's own line and column (X-S30-1 … X-S30-3).
- **Quantities** decode from a YAML number or a decimal string. A number reaches the crate as `f64` (A-c-18); it is
  converted once, exactly, through Rust's shortest round-trip `Display` (`format!("{x}")`, which for every literal
  of at most 15 significant digits returns the literal), then parsed as a decimal by integer arithmetic: more
  decimals than the scale is `OutOfRange` ("1.6605 has more than 3 decimals"); the result is `i64` at the scale. No
  float is stored, compared or used in arithmetic.
- An `instant` is an integer of seconds (a YAML integer, possibly negative).
- `Authored` is `Debug + Send + Sync` and holds `BTreeMap<QualifiedId, AttributeValue>` and the restrictions.

### 5.5 The running catalogue: `attributes-composed` and its Process (Q-S30c-1)

- At genesis `attributes` states **one** `attributes-composed` fact (`SystemInternal`, no subjects) through seam item 5
  (§5.11): `{ schema: 1, catalogue: <the frozen catalogue's run-time projection> }` — per attribute: id, group,
  facets, kind and domain, visibility, mutability, derive, owner, label, phrase, words, `perceived_as`, `prompt_rank`;
  the vocabularies; and per reading or proposing pack its `uses:`. Canonical JSON (sorted maps), so equal catalogues
  give equal bytes.
- Its reduction starts one open-ended, never-woken Process `attributes-catalogue` whose state is the fact's catalogue,
  encoded once (weather's `RecordProcess` precedent, A-c-10). A second `attributes-composed` is refused
  (`attributes-composed-twice`, as weather's `weather-configured-twice`).
- **Reading it.** `CatalogueProcess<O: SystemIdentity>` is the model crate's `ProcessKind` (type
  `attributes-catalogue`); `attributes` starts it as `CatalogueProcess<AttributesSystem>`; every other reader reads it
  as `CatalogueProcess<AttributesId>` (a marker whose `ID` is `ATTRIBUTES`), which `state_for` accepts because it
  checks only the process type (A-c-11). `RunningCatalogue::in_world(&WorldRead) -> Option<RunningCatalogue>` decodes
  it; `None` when `attributes` is not enabled.
- **Cost.** C7 measures the decode per `discloses` call on the fixture sized like the built-in catalogue (93
  declarations; §5.13) and records it. If the median exceeds **0.5 ms** on the Linux CI runner, the pre-approved
  bounded fix is a process-wide memo (at most 4 entries) keyed by `(process id, state length, FNV-1a 64 of the state
  bytes)` — a pure cache whose hit and miss give identical results — recorded as a deviation; a larger cost is a
  material stop (R-c-3).

### 5.6 Facts and reductions (`MODULE_SPEC` §4.1 "Persistence")

| Fact | Payload | Visibility | When | Reduced into |
| --- | --- | --- | --- | --- |
| `attributes-composed` | §5.5 | `SystemInternal` | genesis, once, after configuration (§5.11) | the `attributes-catalogue` Process |
| `traits-stated` | `{ person, values: [(QualifiedId, AttributeValue)] }`, sorted, plain traits only | `Entities({person})` | genesis, one per person with ≥ 1 plain trait | `Traits` on the person |
| `attributes-handed` | `{ person, owner, values }`, sorted | `SystemInternal` | genesis, one per (person, owner) with ≥ 1 owned value, after the person's `traits-stated` | nothing in `attributes`: the owner subscribes (§5.10) |
| `attribute-change-proposed` | `{ person, attribute, to, because: EventId }` | `SystemInternal` | emitted by a proposer that declared `uses.proposes` and depends on `attributes` (`ARC-26`) | checked → `trait-changed` or a named refusal |
| `trait-changed` | `{ person, attribute, from: Option, to, because }` | `Entities({person})`; `Place(p)` when the attribute is `perceivable` and presence places the person in `p` (read through presence's published location, the crate `attributes` already depends on for `PerceptionProvider`) | an accepted proposal | `Traits` on the person |

- `Traits` (component `traits`, schema 1, owner `attributes`) is `BTreeMap<QualifiedId, AttributeValue>` — plain,
  stored traits only. **No derived value and no owner-declared value is ever in it or in any fact** (I-S30-2, I-S30-1).
- **Proposal check** (`ARC-120` item 3, `MODULE_SPEC` §3.1 "Proposes"): the proposer (the emission's `emitted_by`) has
  the attribute in its `uses.proposes`; the attribute is `attributes`-owned and `evolving`; `to` is in its domain;
  `because` is an existing event. Each failure is a `Rejection` with a reason code (`not-a-proposer`, `not-evolving`,
  `out-of-domain`, `unknown-cause`), so the refusal is in the log as every refusal is. Accepted: `trait-changed` with
  `from` the stored value. INV-15 holds by construction: `because` is required.
- Replay (`ARC-25`) reduces these facts exactly; nothing is sampled after genesis.

### 5.7 Resolution in S30-c (`ResolvesSubjects`)

For every Person subject (A-c-3), by `MODULE_SPEC` §4.1's precedence with the parts S30-c has: **the person's own
`attributes:` value, else the declaration's `default`**; otherwise absent. (Lock, roles and the profile are S30-d's,
inserted between the two.) The seed per person is `{ plain: BTreeMap, owned: BTreeMap<SystemId, BTreeMap> }`;
`seed_resolved` states `traits-stated` (if `plain` is not empty) then one `attributes-handed` per owner. A person
with neither states nothing. The resolution reads only the subject's key, tags and section (the type admits nothing
else, S30-b I-b-8).

### 5.8 Disclosure and rendering (`ARC-122` items 4, 6; `MODULE_SPEC` §4.1 "Disclosure", "Rendering")

- `discloses(world, observer, subject)` returns one `ComponentRecord` of type `traits` when the subject has any
  disclosable value, with payload `{ schema: 1, entries: [ { attribute, kind, value, text? } ] }` in catalogue order:
  - **holder** (`observer == subject`): every stored plain trait and every derived plain attribute whose sources exist,
    except `hidden`; values exact.
  - **another observer**: only `perceivable` ones; a value with `perceived_as` is disclosed as its band's **words
    only** — `value` absent, `text` the words — so "tall" never carries "1.87".
  - `kind` is the kind's name, which S30-f's typed reader checks against.
- **Derived** (`derive.rs`): `age { from }` per F-S30c-8 at `world.at()`; `band { from, below: [t₁ < t₂ < …] }` maps a
  quantity (compared at its scale) or a level (by rank) to the `level` value of the first threshold it is below, else
  the last. Computed per read; never stored.
- **Rendering** (`render.rs`): `phrase.replacen("{value}", words, 1)`; words from `words`, default the slug with `-` as
  spaces; a level and category render their word; a quantity per F-S30c-5; a set as `a, b and c` (no serial comma,
  members in domain order); a flag and empty set per F-S30c-6; `perceived_as` bands render their words. Pure and
  bounded: no entry's `text` exceeds 160 bytes (a longer one is cut at a UTF-8 boundary with `…`, and C7 shows none in
  the fixtures).
- One function serves the pack and every owner (`render::disclose(catalogue, values, view)`), so no owner writes
  visibility rules (`ARC-124` item 5).

### 5.9 Reads and proposals: declared as shipped data, refused at load (F-S30c-2)

- A pack that reads attributes ships an `attributes/` file whose `uses.reads` maps each id to its kind name (and
  `optional: true` where the read may be absent under a backend, S30-e); one that proposes lists ids in
  `uses.proposes`. `uses:` in a World Pack or Entity Pack file is `BadDeclaration { field: uses }` ("only a System Pack
  declares what it reads").
- At load, the catalogue build refuses, naming the pack: an id it does not hold (`Missing`), a kind other than the one
  declared (`WrongKind { declared, requested }`), a `text` read (`WrongKind`, "no System reads prose"), and a proposal
  for an attribute that is not `attributes`-owned and `evolving` (`BadDeclaration { field: proposes }`).
- In code, a reader holds **constants**: `const CHRONOTYPE: Handle<Level> = Handle::declared("core.chronotype");`
  (`const fn`, validating the id's shape at compile time) and reads `Traits::get(&CHRONOTYPE) -> Option<LevelValue>`
  or, for a derived attribute, `traits::read(&world, person, &AGE)` through the running catalogue. `Handle<Text>` does
  not exist: `Handle<K: SystemReadable>`, and `Text` implements no `SystemReadable` (X-S30-11, a `trybuild` case).
- Two structural tests in the model crate (`tests/structure.rs`, reading the workspace's `.rs` files): every
  `Handle::declared(` call sits in a `const` item (X-S30-10's "never by slug at run time"); and the `uses.reads` of
  each shipped file equals the set of `Handle::declared` ids and kinds in its pack's `src/`, so data and code cannot
  disagree.
- A reader or proposer `depending_on([ATTRIBUTES])` (A-c-16), so a world enabling it without `attributes` is refused
  at install by the kernel, naming `attributes`.

### 5.10 Owners (`ARC-124`; R-S30c-a-1, Q-S30c-2)

```rust
pub trait AttributeOwner: SystemIdentity {
    /// One owned attribute's value for `person`, projected from this pack's own state; None when absent.
    fn project(world: &WorldRead<'_>, person: EntityId, attribute: &QualifiedId) -> Option<AttributeValue>;
}
/// The owner's disclosure of its own attributes, through the one filter and renderer (§5.8).
pub fn disclose_owned<O: AttributeOwner>(world: &WorldRead<'_>, observer: EntityId, subject: EntityId)
    -> Option<Value>;
```

- An owner declares its attributes in a file it ships (`ships_data!("attributes/<file>.yaml")`, `owner:` itself), and
  receives each person's resolved values by **subscribing** to `attributes-handed` (no dependency, `ARC-28` item 2;
  decoded with the model crate's type), reducing them into its own state or stating its own facts — its choice.
- **Decision (Q-S30c-2): an owner's attributes exist only where `attributes` is enabled.** Without it nothing collects
  the owner's shipped declarations (A-c-4), no `attributes-composed` exists, `disclose_owned` returns `None`, and the
  owner's state, section, configuration and actions work exactly as they do today. A reader of an owner's attribute
  depends on `attributes` (§5.9). Recorded as a dated note under `ARC-120` and step-30 §4.3's sentence "A world may
  therefore have owned attributes without the `attributes` pack" is marked superseded by that note (the primary's edit,
  §12).
- The proof in S30-c is the test-only owner `test-badge` (§5.13); the first real owners are S30-f (`employment`) and
  S31.

### 5.11 Seam item 5: a data owner's world-level facts (general; Q-S30c-1; R-S30c-b-2)

- **`OwnsData`** gains two provided items: `const CONTEXT_FACTS: &'static [EventTypeId] = &[];` and
  `fn seed_context(seeding: &Seeding<'_, '_>, context: &Self::Context) -> Result<Vec<Emission>, Rejection> { Ok(vec![]) }`.
- **`SystemPack`** gains `CONTEXT_FACTS` and `seed_context_erased(seeding, context: &dyn OwnerContext)`, defined by
  `owns_data!()` (no new macro); `installed!` gains `context_facts()` and `seed_context(...)`.
- **The loader** seeds, after `configure::seed` and before the sections loop (A-c-5), for each enabled data owner in
  `systems:` order, its context facts; each must be the owner's vocabulary (`own_vocabulary`, A-c-5) **and** one of
  its `CONTEXT_FACTS` (refused as `ConfigurationStatedUndeclaredFact`'s sibling `ContextStatedUndeclaredFact`), and
  must have no subjects.
- **The drift check**: `check_configuration` adds every enabled capability's `context_facts()` to the compared type
  set (A-c-7), so a resumed world whose catalogue differs from its save's is refused `ConfigurationDrift`, naming
  `attributes` and both facts. No call site changes.
- A world with no data owner, and every data owner that states nothing (S30-b's `test-schema`), seed exactly as before
  (I-S30-10). The item names no domain (S30-b I-b-3's scan extended to it).

### 5.12 The commands (`MODULE_SPEC` §8.1)

| Command | Prints | Exit |
| --- | --- | --- |
| `mineworld attributes catalogue <world> [--json] [--facet F] [--packs DIR]` | the declarations in effect, in catalogue order: id, group, kind and domain (unit, scale), visibility, mutability, owner, label, phrase, words, origin file; per namespace a count | 0; a load refusal as every command prints it |
| `mineworld attributes show <world> [--person KEY] [--json]` | per person: every resolved value with its origin (`authored` or `default` in S30-c), and the holder's and another observer's rendered lines at the epoch | 0 |
| `mineworld attributes check <world>` | everything `catalogue` validates, plus: restrictions never drawn (§5.4), per-namespace counts, and the `uses:` of every enabled pack | 0 when clean; 1 with the refusals |

The module reads `WorldPack::context(Capability::Attributes)` and downcasts (A-c-6). Nothing it prints reaches a fact.

### 5.13 Fixtures and test packs

- **The minimal test catalogue** (`tools/cli/tests/fixtures/attributes/attributes/test.yaml`, namespace the fixture
  world's id `attributes-town`): one attribute of each kind — `stature` (quantity m, scale 3, 0.500–2.500, perceivable,
  `perceived_as` short/average/tall), `hair-colour` (category, perceivable, evolving), `chronotype` (level, learnable,
  evolving), `glasses` (flag, perceivable), `languages` (set over `attributes-town.languages`, learnable),
  `motto` (text 80, private), `birth` (instant, private, fixed), `age-years` (derived `age`, private), `age-band`
  (derived `band`, perceivable), `secret` (level, hidden), `religion` (category, private) — plus `filler.N`, 82
  generated declarations in a second file so the catalogue has 93 like the built-in (§5.5's cost; R-S30-3's size).
- **The fixture world** `tools/cli/tests/fixtures/attributes/`: `social-cafe`'s two places, three people (Alice with
  authored values for every kind; Bob with two; Carol with none), `systems: [presence, movement, conversation, naming,
  attributes]`.
- **`test-early-bird`** (`tests/acceptance/tests/attributes/early_bird.rs`): reads `attributes-town.chronotype`
  through a const handle; provides `open-shop`, refused unless the actor is `early-bird`; proposes
  `attributes-town.hair-colour` on its own `dye` action with `because` the action's event.
- **`test-badge`** (`…/badge.rs`): owns `test-badge.badge` (category, perceivable) in a shipped file, subscribes to
  `attributes-handed`, reduces into its own `Badges` component, projects from it and discloses through
  `disclose_owned`.

### 5.14 Specification sync (C8)

- `MODULE_SPEC` §3.1: seam item 5 (with a dated note); "Reads"/"Proposes" — declared as shipped `uses:`, refused **at
  load** (F-S30c-2), handles are constants; "Owns" — the handoff is `attributes-handed`, and owners' attributes exist
  only with `attributes` (Q-S30c-2). The "implemented by S30-c" marks become "implemented".
- `MODULE_SPEC` §4.1: the facts table gains `attributes-composed` and `attributes-handed`; `uses:` in the declaration
  format; F-S30c-5, F-S30c-6, F-S30c-7 as dated notes; the refusal table gains `ContextStatedUndeclaredFact`.
- `MODULE_SPEC` §8.1: the three commands.
- `DECISIONS.md`: dated notes under `ARC-120` (Q-S30c-2), `ARC-121` (item 5, Q-S30c-1), `ARC-122` (§§5.2, 5.4, 5.9
  forms).

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-c-1 (I-S30-10, X-S30-9) | Every reference world reads, loads and runs byte-identically | `worldpack/tests/social_cafe.rs` unchanged; the `decisions` golden (`tools/cli/tests/decisions.rs`) unchanged; one nightly `baseline check` on the final head (C9) |
| I-c-2 (I-S30-8, AC-S30-12) | `kernel/` and `contracts/` contain none of `AttributeId`, `AttributeValue`, `Catalogue`, `Handle<`, `AttributeOwner`, `traits-stated`, `trait-changed`, `attribute-change-proposed`, `attributes-composed`, `attributes-handed`, `attributes-catalogue`, the component type `traits` as a string literal | `authoring/attributes/tests/structure.rs` |
| I-c-3 (I-S30-1, X-S30-7) | Only `attributes` writes `Traits`; an owner's value is never in `Traits` or any `attributes` fact other than `attributes-handed` | the kernel's `OwnedBy`; `systems/attributes/tests/pack.rs` (an owner-declared value lands in `attributes-handed` only); X-S30-7's plant |
| I-c-4 (I-S30-2, X-S30-8) | No derived value is stored or stated | the save scan of C7; X-S30-8's plant |
| I-c-5 (I-S30-3, X-S30-4) | A `private` or `hidden` value never reaches another observer; `hidden` never reaches the holder | C7's scan over every observation of a 30-day run of the fixture world (every seat × every perceived person) |
| I-c-6 (I-S30-4) | Every value in the world passed its declaration at load or its acceptance as a proposal | X-S30-1 … X-S30-3; the proposal refusals |
| I-c-7 (I-S30-5, X-S30-10, X-S30-11) | Reads only through constant handles; never `text` | `structure.rs`; the `trybuild` case |
| I-c-8 | The seam's item 5 names no domain and changes nothing for a world without a data owner that states | S30-b's I-b-3 scan extended to the new items; `data_seam.rs` unchanged and green |
| I-c-9 | The resolution sees no name, note or location | S30-b's `Subject` type; review |
| I-c-10 | No float is stored, compared or computed | review; a `clippy::float_arithmetic` deny on the model crate |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | fmt; clippy `-D warnings` on `mineworld-authoring`, `mineworld-sdk`, `mineworld-worldpack`, `mineworld-attributes-model`, `mineworld-attributes`, `mineworld-cli`, `mineworld-acceptance`; `#![deny(clippy::float_arithmetic)]` in the model crate | per commit; CI `fast` |
| Unit (model crate) | id parsing and short slugs; every kind's decoding and refusal with line/column; exact decimal decoding; catalogue build (namespaces, duplicates, owners, vocabularies, overrides, uses); derived `age`/`band` against hand-computed instants; rendering goldens hand-written in the test (never produced by the renderer, test rules §25); the filter's two views | `authoring/attributes/src/*/tests.rs` |
| Unit (seam) | item 5's defaults; the erased `seed_context`; an undeclared context fact refused; context facts compared | `authoring/src/data/tests.rs`, `sdk/rust/tests/data.rs`, `worldpack/src/configure/tests.rs` |
| Crate (pack) | the reductions; the proposal check's four refusals; owner values only in `attributes-handed` | `systems/attributes/tests/pack.rs` |
| Structural | AC-S30-12; X-S30-10; uses = handles | `authoring/attributes/tests/structure.rs` |
| Compile-fail | X-S30-11 | `authoring/attributes/tests/compile_fail/` (`trybuild`, `DEP` already in the workspace) |
| Acceptance (seam-driven) | `test-early-bird` and `test-badge` with the real `attributes`: the read refuses `open-shop` for a night owl; `dye` → `trait-changed` citing its cause; the owner's projected value disclosed through the filter | `tests/acceptance/tests/attributes_seam.rs` |
| Integration (real binary) | the checkpoint: `mineworld server` on the fixture world; Alice's and Bob's observations; SIGKILL and restart; the catalogue drift refusal on resume; the three commands; X-S30-4 30-day scan; size and cost measurement | `tools/cli/tests/attributes.rs` |
| Regression | I-c-1 | existing tests unchanged; one nightly dispatch (C9) |
| Gate 1, Gate 2 | NOT REQUIRED: no model, no rendering client, no lifecycle timing | — |

---

## 8. Acceptance criteria (fixed before measuring)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-c-1 | **Checkpoint.** `mineworld server` on the fixture world: Alice's own observation's `traits` lists every non-hidden value of hers with exact values (`stature` `"1.660"`, `birth`, `age-years`, `motto`, `religion` …) and no `secret`; Bob's observation of Alice lists exactly her `perceivable` ones (`stature` as `"of average height"` with no value, `hair-colour`, `glasses`, `age-band`) | S30-c row |
| AC-c-2 | **Restart.** After SIGKILL and a restart on the same save, both observations are byte-identical to before | S30-c row |
| AC-c-3 | **Proposal.** `dye` by `test-early-bird` states `trait-changed { from: brown, to: auburn, because: <the dye fact> }`; a proposal for `chronotype` by a pack that did not list it, for a `fixed` attribute, outside the domain, or citing no event is refused with its reason code | S30-c row; `ARC-120` item 3 |
| AC-c-4 | **Drift.** Resuming the save after the fixture's `hair-colour` gains a value is refused `ConfigurationDrift` naming `attributes` | R-S30c-b-2 |
| AC-c-5 | X-S30-1, X-S30-2, X-S30-3: refused at the value's own line and column with the domain, the bounds and the nearest `stature` | step-30 §13.5 |
| AC-c-6 | X-S30-4: a 30-day run's every observation scanned, no private or hidden value of another person; the planted mutation turns it red naming the attribute and the seat | step-30 §13.5 |
| AC-c-7 | X-S30-7: `test-badge` writing `Traits` does not compile (`OwnedBy`); a world file declaring `owner: test-badge` is `BadDeclaration`/`OwnerNotInstalled` by F-S30c-7 | step-30 §13.5 |
| AC-c-8 | X-S30-8: no `age-years` or `age-band` in any fact of the save; the plant (stating `age-years` in `traits-stated`) turns the save scan red | step-30 §13.5 |
| AC-c-9 | X-S30-10, X-S30-11: the structural test refuses a planted `Handle::declared` inside a function body; `Handle<Text>` fails to compile | step-30 §13.5 |
| AC-c-10 | **Owner.** `test-badge`'s badge reaches Bob's observation of Alice inside `test-badge`'s own component, through `disclose_owned`; in a world without `attributes` the same pack runs and discloses no attribute | `ARC-124`; Q-S30c-2 |
| AC-c-11 | **Commands.** `catalogue --json` lists 93 declarations of the fixture; `show --person alice` prints each value with its origin; `check` exits 0 on the fixture and 1 with the refusal on each of AC-c-5's planted files | `MODULE_SPEC` §8.1 |
| AC-c-12 | I-c-1: the reference worlds' tests and the `decisions` golden pass unchanged; the nightly `baseline check` on the final head passes with `scripts/baselines.txt` unchanged | X-S30-9 |
| AC-c-13 | I-c-2: the scan is clean | AC-S30-12 |
| AC-c-14 | **Size and cost recorded.** For the 93-declaration fixture: the bytes of Alice's own and Bob's view of her `traits`, and the median and p95 of one `discloses` call over 1 000 calls, in §16 | R-S30-3; §5.5 |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]` needs the
work and recorded evidence; `N/A` needs an audited reason.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s30c`. Never `cd` elsewhere.

1. Re-read this commit's section and every file before editing it.
2. Make the numbered steps in order; nothing of a later commit.
3. Run the commit's validation one `--test`/`--lib` at a time; record command, count, wall time and PASS / FAIL /
   INCONCLUSIVE. `cargo clippy -p <touched crates> --all-targets -- -D warnings`; `cargo fmt --all --check`.
   **R-LOAD-1:** never `cargo test --workspace` and never a whole member's suite.
4. Mutations: copy the file, plant, run the named test red (record its name), restore from the copy, `cmp`, run green.
5. Answer the commit's review questions in §16 in one or two sentences each.
6. `git status --short`: every changed path is in the commit's **Files**; otherwise revert or stop and record why.
7. Update §16 and the handoff in the same commit; stage by path; commit with the message and the session's attribution
   line. No heredoc, no `sed -i`, no `python3 -c`.

#### C0 — Base and ledger

- **Files:** this file (§16), `.structured-coding/plans/mvp0/handoff-s30c.md` (new).
- Steps: record the base; confirm S30-b merged (`authoring/src/data.rs` with `OwnsData`, `ships_data!`); re-read A-c-1 …
  A-c-23 on the base and record moved lines; confirm §15's rulings; record whether S26-a, P6a and S31's notices have
  merged (none is needed).
- [ ] Implementation. [ ] Validation: `python3 scripts/check_doc_headings.py`, `check_decision_ids.py`.
  [ ] Review: the base satisfies §2.3; every ruling of §15 is applied to §§5, 8 before C1.
- **Commit message:** `docs(plan): S30-c — open the ledger at the base`

#### C1 — The seam, item 5: a data owner's world-level facts, compared at resume

- **Files:** `authoring/src/data.rs`, `authoring/src/data/tests.rs`, `sdk/rust/src/{pack.rs, installed.rs}`,
  `sdk/rust/tests/data.rs`, `worldpack/src/{load.rs, configure.rs, error.rs}`, `worldpack/src/configure/tests.rs`,
  `worldpack/src/data/tests.rs`.
- Steps: §5.11 exactly — the two `OwnsData` items; `SystemPack::CONTEXT_FACTS` and `seed_context_erased` inside
  `owns_data!()`; the `installed!` arms; the loader's seeding position and both checks; `PackError::
  ContextStatedUndeclaredFact { system, event_type, path }`; `check_configuration`'s widened type set. Also §2.1
  item 7: the `resolve_subjects` rename and the R3b ordering test.
- Tests: a probe owner stating one context fact appears after configuration facts and before the first section fact;
  an undeclared or foreign context fact is refused by name; a context fact with a subject is refused; two genesis
  facts lists differing only in a context fact are `ConfigurationDrift` naming the owner; `test-schema` (states none)
  and `data_seam.rs` unchanged.
- Validation: `cargo test -p mineworld-authoring --lib`; `-p mineworld-sdk --lib`, `--test data`; `-p
  mineworld-worldpack --lib`, `--test social_cafe`, `--test data_directories`; `-p mineworld-acceptance --test
  data_seam`; `-p mineworld-cli --test decisions`. Mutation **M-c-1** (context facts seeded after sections) → the
  position test red; **M-c-2** (`check_configuration` ignores context facts) → the drift test red.
- [ ] Implementation. [ ] Validation. [ ] Review: no attribute word (I-c-8 scan); every new item is provided; a world
  without a data owner runs the same code path plus one empty loop.
- **Commit message:** `feat(seam): a data owner states world-level facts from its context, compared at resume (ARC-121, S30-c)`

#### C2 — The model crate: ids, kinds, values, declarations, the catalogue

- **Files:** `Cargo.toml` (members), `Cargo.lock`, `authoring/attributes/{Cargo.toml, README.md}`,
  `authoring/attributes/src/{lib.rs, id.rs, kind.rs, value.rs, declaration.rs, catalogue.rs}` and their `tests.rs`.
- Steps: §§5.2, 5.3 (except `uses:` checks, C5), 5.4's value decoding; `AttributeError` with every refusal of
  `MODULE_SPEC` §4.1 this PR raises (`BadDeclaration`, `UnknownAttribute`, `DuplicateAttribute`, `ForeignNamespace`,
  `AmbiguousSlug`, `WrongKind`, `OutOfDomain`, `OutOfRange`, `UnknownTopic`, `OverrideNotAllowed`,
  `OwnerNotInstalled`, `Missing`); `#![deny(clippy::float_arithmetic)]`.
- Tests: one per refusal, each asserting the file, line and column from YAML text decoded through `serde-saphyr`; the
  decimal decoder on `1.66`, `1.660`, `"1.660"`, `0.5`, `2.5`, `1.005` (scale 3 → 1660, 1660, 1660, 500, 2500,
  1005) and `1.6605` refused; overrides by the table, both directions; a qualified id with a `.` in its slug.
- Validation: `cargo test -p mineworld-attributes-model --lib`; clippy. Mutation **M-c-3** (the decimal decoder
  multiplies by `10^scale` as `f64` and truncates with `as i64`) → the decoder test on `1.005` at scale 3 red (the
  float path gives 1004, because `1.005 × 1000` is `1004.9999999999999` in binary floating point).
- [ ] Implementation. [ ] Validation. [ ] Review: every error names what `MODULE_SPEC` §4.1's *Names* column lists; no
  `HashMap`; the crate depends on no pack.
- **Commit message:** `feat(attributes-model): declarations, kinds, values and the frozen catalogue (ARC-122, S30-c)`

#### C3 — The model crate: the section, traits, handles, derived values, rendering, owners, facts

- **Files:** `authoring/attributes/src/{section.rs, traits.rs, derive.rs, render.rs, owner.rs, facts.rs}` and tests;
  `authoring/attributes/tests/{compile_fail.rs, compile_fail/handle_text.rs, compile_fail/handle_text.stderr}`.
- Steps: §5.4's `DeserializeSeed`; `Traits` with `Component` by hand (`traits`, schema 1, owner `ATTRIBUTES`);
  `Handle<K>`, `SystemReadable`, `Handle::declared` (`const fn`); `derive.rs` per F-S30c-8 and §5.8; `render.rs` per
  §5.8 and F-S30c-5/6; `AttributeOwner`, `disclose_owned`; the five fact types and `CatalogueProcess<O>`,
  `AttributesId`, `RunningCatalogue::in_world`.
- Tests: `age` at hand-computed instants (birth `−1 072 915 200`, at `0` → 34; one second before the 35th mean year →
  34; at it → 35); `band` on a quantity and a level; render goldens written by hand in the test; the two views;
  `disclose_owned` with a stub owner.
- Validation: `cargo test -p mineworld-attributes-model --lib`; `--test compile_fail`. Mutation **M-c-4** (the
  observer view returns `learnable` values) → the filter test red.
- [ ] Implementation. [ ] Validation. [ ] Review: `Text` has no `SystemReadable`; no derived value can be put into
  `Traits` through any public function; every payload is canonical (sorted maps).
- **Commit message:** `feat(attributes-model): typed reads, derived values, rendering, the owner interface and the pack's vocabulary (ARC-122, ARC-124, S30-c)`

#### C4 — The `attributes` System Pack, installed

- **Files:** `systems/attributes/{Cargo.toml, README.md, src/*.rs, tests/pack.rs}`, `systems/installed/{Cargo.toml,
  src/lib.rs}`, `Cargo.lock`, `worldpack/tests/data_directories.rs` (the two expected lists, A-c-21).
- Steps: `AttributesSystem` (`ID = ATTRIBUTES`, test-asserted); `OwnsData` (`attributes`, `vocabularies`; `context`
  = `Catalogue::build`; `CONTEXT_FACTS = [attributes-composed]`; `seed_context`); `SectionInContext` (`attributes:`,
  carried by `Person`); `ResolvesSubjects` (§5.7); `System` (declaration owning `Traits`, emitting its facts,
  subscribing to `attributes-composed`, `traits-stated`, `attribute-change-proposed`; install; react per §5.6);
  `PerceptionProvider` per §5.8; `impl OwnedBy<AttributesSystem> for Traits`; the `SystemPack` impl with
  `owns_data!()`, `owns_section_in_context!()`, `resolves!()`; the installed-set lines.
- Tests (`pack.rs`, a kernel `World` composed directly, as `systems/*/tests` do): genesis order (composed, then per
  person stated, then handed); reductions; the proposal check's four refusals and the acceptance; replay equality.
- Validation: `cargo test -p mineworld-attributes --test pack`; `cargo build -p mineworld-installed-systems`; `cargo
  test -p mineworld-installed-systems`; `-p mineworld-worldpack --test data_directories`, `--test social_cafe`; `-p
  mineworld-cli --test decisions`. Mutation **M-c-5** (the reducer accepts a proposal for a `fixed` attribute) →
  `a_fixed_attribute_is_never_changed` red.
- [ ] Implementation. [ ] Validation. [ ] Review: the pack holds no field; every fact is in its declaration; the
  directory list in the two edited tests is exactly what `DataOwnerDecl` declares.
- **Commit message:** `feat(attributes): the attributes System Pack — section, traits, facts, proposals, disclosure (ARC-120, S30-c)`

#### C5 — `uses:`: reads and proposals declared by the packs that make them

- **Files:** `authoring/attributes/src/catalogue.rs` (uses checks), `authoring/attributes/tests/structure.rs`,
  `tests/acceptance/tests/{attributes_seam.rs, attributes/{mod.rs, early_bird.rs, badge.rs}}`,
  `tests/acceptance/attributes/{early-bird.yaml, badge.yaml}` (the shipped files), `tests/acceptance/Cargo.toml`.
- Steps: §5.9's checks and refusals; the two structural tests and AC-S30-12's scan (I-c-2); the two test packs (§5.13);
  `attributes_seam.rs` driving `attributes` (real) with the two packs exactly as the loader calls the seam (S30-b's
  `data_seam.rs` pattern), then `World::genesis`, then actions.
- Tests: AC-c-3, AC-c-7 (data half), AC-c-10; `Missing` and `WrongKind` naming the pack.
- Validation: `cargo test -p mineworld-acceptance --test attributes_seam`; `-p mineworld-attributes-model --test
  structure`. Mutations **M-c-6** (X-S30-10: a `Handle::declared` call in `open-shop`'s validate body) → `structure`
  red naming the file and line; **M-c-7** (X-S30-7: `test-badge` declares `owner: attributes` for its badge and
  states it through `attributes`) → the owner test red.
- [ ] Implementation. [ ] Validation. [ ] Review: the test packs name no installed pack's vocabulary; `uses:` is refused
  in a world file.
- **Commit message:** `feat(attributes): packs declare what they read and propose, refused at load (S30-c)`

#### C6 — `mineworld attributes catalogue | show | check`

- **Files:** `tools/cli/src/{attributes.rs, main.rs}`, `tools/cli/Cargo.toml` (`mineworld-attributes-model`),
  `tools/cli/tests/attributes.rs` (the command half), `tools/cli/tests/fixtures/attributes/**`.
- Steps: §5.12; the fixture world and catalogue of §5.13 (the 82 fillers in `attributes/filler.yaml`, written by hand
  with a fixed pattern `filler-NN` level `[low, mid, high]`, learnable).
- Validation: `cargo test -p mineworld-cli --test attributes` (the command tests only, filtered by name prefix
  `command_`). AC-c-11.
- [ ] Implementation. [ ] Validation. [ ] Review: nothing printed reaches a fact; the JSON is canonical.
- **Commit message:** `feat(cli): mineworld attributes catalogue, show and check (S30-c)`

#### C7 — The checkpoint through the real server

- **Files:** `tools/cli/tests/attributes.rs` (the server half).
- Steps: AC-c-1, AC-c-2 (the `restart.rs` pattern: start, join Alice and Bob through `tools/cli/tests/support`, read
  the observation frames, SIGKILL, restart, compare), AC-c-4 (edit the copied fixture, resume, read the refusal),
  AC-c-6 (`mineworld run` 30 days on a scratch copy; every recorded observation of every seat scanned — the scanner
  decodes `traits` entries and refuses any `private`/`hidden` id of another person, naming it), AC-c-8 (the save's
  facts scanned for derived ids), AC-c-14 (sizes from the frames; the cost from a timed loop in the same test, printed
  and recorded, not asserted beyond the §5.5 stop).
- Validation: `cargo test -p mineworld-cli --test attributes`. Mutation **M-c-8** (X-S30-4: the observer view returns
  every value) → AC-c-6's scan red naming `attributes-town.religion` and Bob's seat; **M-c-9** (X-S30-8) → AC-c-8 red.
- [ ] Implementation. [ ] Validation. [ ] Review: the scan covers every seat × perceived person × observation; the
  test leaves no scratch (`check_scratch.py`).
- **Commit message:** `test(cli): attributes through the real server — two views, restart, drift and a 30-day scan (S30-c)`

#### C8 — Specification sync

- **Files:** `docs/MODULE_SPEC.md`, `docs/DECISIONS.md`.
- Steps: §5.14, each change a dated note citing this PR and the finding or ruling; old text kept visible.
- Validation: `python3 scripts/ci_layer.py docs`.
- [ ] Implementation. [ ] Validation. [ ] Review: every finding of §3.2 that changed specified text has its note; no
  specification text and code disagree (`CLAUDE.md` §2.1 rule 4).
- **Commit message:** `docs(spec): attributes as implemented — seam item 5, uses, the composed catalogue (S30-c)`

#### C9 — Close-out

- **Files:** this file (§16), the handoff.
- Steps: I-c-2 and I-c-8 scans; `git diff --stat origin/main...HEAD -- systems/ ':!systems/attributes'
  ':!systems/installed'` empty; push `mvp0/pr-s30c`; `gh pr create`; CI on the exact head (three OSes) green; one
  `nightly.yml` dispatch on the final head for AC-c-12.
- [ ] Implementation. [ ] Validation: CI and the nightly verdict. [ ] Review: deviations recorded.
- **Commit message:** `docs(plan): S30-c close-out`
- **Then:** READY FOR OPERATOR REVIEW — DO NOT MERGE.

---

## 10. R-LOAD-1

Several agents share the machine. Locally: only the commit's targeted test files, one at a time, and `cargo clippy` on
the touched crates. Never `cargo test --workspace`, never a whole member's suite, never the nightly locally. The PR's
CI on the exact head (Linux, Windows, macOS) is the gate; one `nightly.yml` dispatch on the final head is the only
long run.

---

## 11. Requirements this PR places on other PRs

- **R-S30d-c-1 (S30-d).** The sampler is inserted in §5.7's precedence between the person's value and the default; its
  inputs are S30-b's `Subject`s and `Authored.restrictions`; `attributes-composed` gains the population's identity, so
  a resumed world whose profile, seed, salt or lock changed is refused by the same drift check.
- **R-S30e-c-1 (S30-e).** `core` is accepted only from `person-attributes`; the `uses.reads` `optional` flag and
  `MissingUnderBackend` are S30-e's; re-measure AC-c-14 on the real catalogue.
- **R-S30f-c-1 (S30-f).** Readers of the disclosed payload use `kind` and the externally tagged value (§5.2); the
  Python renderer mirrors `render.rs` by goldens produced from hand-written expectations; `employment` becomes an
  owner through `AttributeOwner` and `attributes-handed`.
- **R-S31-c-1 (S31).** `notices` and `economy` ship their declarations with `owner:` themselves, receive initial values
  through `attributes-handed`, disclose with `disclose_owned`, and exist as attributes only where `attributes` is
  enabled (Q-S30c-2).

## 12. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-30-person-attributes.md` §4.3 | "A world may therefore have owned attributes without the `attributes` pack" → superseded by `ARC-120`'s note (Q-S30c-2) |
| `step-30-person-attributes.md` §§5.3, 9.4 | "obtains `Handle<Level>` once, at install" → declared as shipped `uses:`, refused at load; handles are constants (F-S30c-2) |
| `step-30-person-attributes.md` §11.1 | the facts table gains `attributes-composed` and `attributes-handed` (Q-S30c-1) |
| `overall.md` | S30-c frozen / merged |

---

## 13. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S30-c — the attributes System Pack and the attribute model crate
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s30c-attributes-pack.md
RELATED / BINDING DOCS:  step-30-person-attributes.md (§§3, 4, 5, 7, 9, 11, 12, 13); pr-s30a-attribute-specs.md;
                         pr-s30b-authoring-seam.md; docs/CORE_CONCEPTS.md §4.5; docs/MODULE_SPEC.md §§2, 3.1, 4.1,
                         8.1; docs/DECISIONS.md ARC-25, ARC-26, ARC-28, ARC-31, ARC-61, ARC-120…ARC-125, DEP-10;
                         CLAUDE.md; docs/ENGINEERING_STANDARDS.md; .structured-coding/standards.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s30c, its own, held by one session
BRANCH:                  mvp0/pr-s30c, created from origin/main
START CONDITION:         S30-b merged (met: #247, 04d5145e) AND this design frozen AND §15 ruled (both met:
                         operator, 2026-10-18), then the primary's dispatch
IMPLEMENTATION BASE:     main (origin/main at C0); C0 records it
APPROVED SCOPE:          §2.1 items 1–7, as §15's rulings (2026-10-18) shape them
FROZEN INVARIANTS:       step-30 I-S30-1 … I-S30-5, I-S30-8, I-S30-10; §6 I-c-1 … I-c-10; §8 as written
SEQUENCE:                C0 … C9 (§9)
IMPLEMENTER:             Opus-class; reviewer the primary session
ALLOWED COMMANDS:        cargo build/test/clippy/fmt -p <crate> (never --workspace); target/*/mineworld *; git;
                         gh (pr create/view; workflow run nightly.yml once in C9; run download; never merge);
                         python3 scripts/check_doc_headings.py, check_decision_ids.py, check_scratch.py,
                         ci_layer.py docs, ci_parity.py baseline check; cp; cmp; mkdir -p; sed -n; grep
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace (R-LOAD-1);
                         any edit under kernel/, contracts/, server/, clients/, cognition/, worlds/ or an existing
                         systems/<pack>/src; any reference-world edit; any model call; a new external dependency
MATERIAL STOPS:          S30-b not merged; a reference world's bytes moving; a needed required item on an existing
                         trait; a needed kernel or contracts change; §5.5's cost above the memo's reach; a
                         specification rule that the implementation cannot meet without changing it beyond §5.14
PLATFORMS:               Linux, macOS, Windows (CI core on all three)
VALIDATION BUDGET:       crate, acceptance and CLI tests unrestricted one file at a time; one nightly dispatch (C9).
                         Gate 1 and Gate 2: NOT REQUIRED
LIVE DOCUMENTATION:      this document (§16)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s30c.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                                     (task brief 2026-10-17: "endpoint is a PR ready for
                                                      review, with no merge")
                         semantic commits: authorized                (working rules §14)
                         branch push: authorized                     (working rules §21)
                         PR creation / update: authorized            (working rules §21)
                         CI repair to review readiness: authorized   (working rules §21)
                         endpoint: the PR READY FOR OPERATOR REVIEW
                         merge: NOT authorized — only with the operator's explicit authorization (working
                         rules §22)
POST-MERGE SYNC OWNER:   primary: step-30, overall.md; implementation session: this ledger and the merge identity
STOP CONDITION:          S30-c READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 14. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-c-1 | Seam item 5 touches the loader every world uses | additive and provided; a world without a stating data owner runs today's path; I-c-1 by the decisions golden, `social_cafe.rs` and the nightly; Opus implementer |
| R-c-2 | `attributes-composed` grows with the catalogue (≈ 30 KB at 93; ≈ 0.5–1 MB under MatrAIx `all`, S30-h) | one fact, once, at genesis; its Process is encoded once (weather-record precedent); S30-h measures its own case |
| R-c-3 | Decoding the catalogue per disclosure is slow | measured in C7 (AC-c-14); the memo is pre-approved; beyond it, a material stop |
| R-c-4 | Data `uses:` and code handles diverge | the structural test holds them equal per pack (§5.9) |
| R-c-5 | A level's stored rank outlives a reordered domain | reorder is a major version; a resumed world is refused on drift (§5.11) |
| R-c-6 | S31 codes against `ARC-124` before this merges | §11 R-S31-c-1 states the realised interface; a conflict is a material stop for both (step-30 §13.3) |

---

## 15. Operator questions (each with a recommendation; limited to what the rulings do not settle)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S30c-1 [the seam, `ARC-121`; the persistence table]** | The running world must know its catalogue (visibility, words, derived rules, proposal checks), but a System holds no fields, a configuration fact is seeded without the data context, and a resolution seeds per subject only (F-S30c-1). How does the catalogue reach run time, and how is a resumed world checked against it (R-S30c-b-2)? | **Add a general seam item 5: a data owner may state world-level facts from its context, once at genesis after configuration, declared as `CONTEXT_FACTS` and compared at resume like configuration facts** (§5.11). `attributes` states one `attributes-composed` fact holding the frozen catalogue, folded into a Process (weather-record's precedent). It amends `ARC-121` (dated note) and adds two facts to `MODULE_SPEC` §4.1's table. Declined: copying the declarations into every person's `traits` (a copy of the catalogue per person, and an override would move every fact); a process-wide registry (tests and a host run several worlds in one process); handing data to systems at install (a kernel and SDK change, and a snapshot would no longer be self-sufficient). **Ruled 2026-10-18 (operator): as recommended — seam item 5, `CONTEXT_FACTS`, one `attributes-composed` fact.** |
| **Q-S30c-2 [R-S30c-a-1, `ARC-120`]** | In a world that enables an owner pack (`employment`, `notices`) but not `attributes`, do the owner's attributes exist? | **No: an owner's attributes exist only where `attributes` is enabled.** The owner's state, section and actions work unchanged; nothing discloses them as attributes; a reader of them depends on `attributes`. One path, one catalogue, overrides and masks always apply. Step-30 §4.3's "a world may therefore have owned attributes without the `attributes` pack" is superseded by a dated note. The alternative — each owner parses its own compiled-in declarations when `attributes` is absent — gives two catalogues, two renderings, and overrides that apply only sometimes. **Ruled 2026-10-18 (operator): no, as recommended — an owner's attributes exist only where `attributes` is enabled.** |

---

## 16. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 … C9 | — | [ ] | [ ] | [ ] |

### Evidence

(none yet)

### Mutations

(none yet; M-c-1 … M-c-9 as §9 lists them)

### Deviations and findings

F-S30c-1 … F-S30c-8 (§3.2), recorded at design time.
