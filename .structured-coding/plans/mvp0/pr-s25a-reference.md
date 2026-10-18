# PR S25 25a — The generated configuration reference, Rust side

## DESIGN FROZEN (step design) · IMPLEMENTATION AUTHORIZED 2026-10-14

```text
Design revision:        step-25-creator-docs.md §10.3 "PR 25a" as frozen on main @ f2f0a71
                        (STEP DESIGN FROZEN 2026-10-14), detailed to the commit here (§6)
Approved by / evidence: the operator's rulings QS25-1/2/3/9/10 and the primary session's QS25-4…8
                        (step-25 §13.1); the primary session's dispatch of 25a to this Opus-class
                        implementer, 2026-10-14 ("implementation is authorized by the primary's
                        dispatch now")
Implementation base:    origin/main @ f2f0a71 (#187)
Execution contract:     §8
Lifecycle:              IN EXECUTION
```

**Effort:** `mvp0` · **Step:** [`step-25-creator-docs.md`](step-25-creator-docs.md) §§0, 2.2, 3, 4.1,
5, 10.3 (PR 25a), 11 (`ARC-90` reference part, `DEP-49`), 13.1, 15 (A-S25-1, A-S25-4), 16 ·
**Parent:** [`overall.md`](overall.md) · **Branch / worktree:** `mvp0/pr-s25a` in
`/Users/yuema137/mineworld-worktrees/impl-s25a` (sole writer) · **External repository:**
`yuema137/mineworld-pack-fishing` (the third-party `acme-fishing` pack, re-pinned by this PR, QS25-2).

Scope, invariants and acceptance are the step's (§10.3 PR 25a, AC-25a-1 … AC-25a-6; INV-S25-1,
INV-S25-2, INV-S25-5). This file adds the commit-level detail, the decisions a step plan could not
make (SD-25a-n), and the live ledger. It is the PR's one authority; the handoff is
[`handoff-s25a.md`](handoff-s25a.md).

---

## 1. Goal

Every file a creator writes is decoded by a typed Rust value. 25a derives a JSON Schema from each of
those types with `schemars` (`DEP-49`), makes the SDK refuse a pack whose section or configuration type
has none (QS25-2), asks every installed pack for its schemas through the installed set's macro (the
macro still names no pack), and renders the schemas into `docs/reference/` with a read-only subcommand,
`mineworld reference --out DIR | --check DIR`. CI regenerates the reference and fails on any byte of
difference, so the reference cannot drift (INV-S25-2). Nothing a world does changes (INV-S25-5).

## 2. Audit (origin/main @ f2f0a71)

The step's §2.2 table holds; the additions and corrections found at C0:

| # | Finding | Evidence | Consequence |
| --- | --- | --- | --- |
| A-1 | Section types: `naming::Name`, `schedule::AuthoredRoutine`, `item::AuthoredItem`, `inventory::AuthoredHoldings`, `economy::AuthoredEconomy`, `employment::AuthoredJob`, `bodies::Body`, `acme_fishing::AuthoredSpot`; configurations: `calendar::CalendarConfiguration`, `weather::WeatherConfiguration`; Interaction List sections: `conversation`, `relationships`, `group-activity` | `grep "type Authored =\|type Configuration =\|impl InteractionSection"` | Eight section types, two configurations, three Interaction List sections, plus test packs in `tests/acceptance`, `worldpack` and `authoring` tests |
| A-2 | Several types decode through `#[serde(try_from = "Raw…")]` | `employment`, `weather`, `bodies`, `acme-fishing`, `packages::policy` | `schemars_derive` 1.2.3 honours `try_from` (it uses the source type's schema in the deserialize contract, `schema_exprs.rs` `expr_for_container`); the doc comments go on the `Raw…` fields |
| A-3 | `schemars` takes a doc comment whole as `description`; a `#`-heading first line becomes `title` | `schemars-1.2.3/src/_private/rustdoc.rs` `get_title_and_description` | SD-25a-1 |
| A-4 | `serde_json` is built without `preserve_order`; enabling it through schemars would unify the feature workspace-wide and could change the bytes of any `serde_json::Map` serialization a world performs | `Cargo.lock` (no `indexmap` edge from `serde_json`); schemars feature `preserve_order = [serde_json/preserve_order]` | SD-25a-6: not enabled; properties are listed in name order |
| A-5 | `PackFile` and `StyleManifest` are private to `mineworld-packages` | `packages/src/manifest.rs:30–50` | A public `schema` function per file in `mineworld-packages`, not a visibility change |
| A-6 | The content files' framework fields have no `Deserialize`; their legal keys are the constants `PERSON_FIELDS`, `PLACE_FIELDS`, `ITEM_FIELDS`, `ORGANIZATION_FIELDS` | `worldpack/src/content.rs:28–48` | Manual `JsonSchema` impls over private mirror structs; a test holds each schema's `properties` equal to its constant (step C1 validation) |
| A-7 | Every `parameters!` field in the tree carries only doc comments | conversation, relationships, group-activity, the SDK and acceptance test packs | SD-25a-3: the macro's field grammar becomes `$(#[doc = …])*`, so it can read the description itself |
| A-8 | `acme-fishing` at `b40e71f` (tag `v0.1.0`) has one section, `AuthoredSpot` via `RawSpot`; its normal dependencies may be the published surface or registry crates | `ext-fishing/src/section.rs`; `tests/acceptance/tests/package_sources.rs` EC-4 | It adds `schemars = "1.2.3"` (a registry crate already in MineWorld's lock) and derives on `RawSpot`/`AuthoredSpot` |
| A-9 | The step's banner names `<git describe>` | step-25 §5.1 | A commit hash in every generated file would change the reference at every commit and make `--check` fail by construction: SD-25a-5 |

## 3. Decisions (bounded, within the frozen step design)

- **SD-25a-1 — The reference shows a doc comment's first paragraph.** The generator replaces every
  schema's `description` with its first paragraph (up to the first blank line), lines joined by a space.
  The first paragraph is written for an author; later paragraphs stay for developers and reach neither
  the Markdown nor the JSON Schema. A field whose first paragraph is about the code is rewritten
  (F-S25-4). A property with no description refuses generation, naming the schema and the field
  (AC-25a-3).
- **SD-25a-2 — Contract-typed fields name a schema type from `mineworld-authoring`.** `mineworld-contracts`
  stays free of schemars (step §4.1). `mineworld_authoring::schema` holds uninhabited marker types
  (`Key`, `Identifier`, `TagList`, `Length`, `EntityTypeName`) with hand-written `JsonSchema` impls that
  state the contract's own rule (`contracts/src/ids.rs` `check_identifier`: 1–64 bytes of `a-z0-9-_`, no
  separator at either end); a field uses `#[schemars(with = "…")]`.
- **SD-25a-3 — `parameters!` writes its own `JsonSchema` impls.** For `$Name` (every field with `default`)
  and `$Partial` (every field optional), each field with its type's schema, `default`, `minimum` and
  `maximum` (numeric bounds only; a `bool` field states none) and its doc comment as `description`. The
  impl goes through `$crate::__private::schemars`, so a pack using the macro needs no schemars
  dependency. `Fields` and `Parameters` gain the `JsonSchema` bound; both are made by the macro.
- **SD-25a-4 — Three more questions, answered by the macros a pack already writes.** `SystemPack` gains
  `section_schema`, `configuration_schema` and `interaction_declarations`, each defaulting to `None` and
  defined by `owns_section!`, `configures!` and `interactions!`; `installed!` gains the same three
  questions on `Capability`. An Interaction List section's schema is a hand-written `JsonSchema` impl on
  `Section<S>` built from the pack's declarations, so `configuration_schema` answers for every
  configuration alike. `Declarations` is plain data (actions, facts, parameter roles, reference list
  names, the parameter and knob schemas) in `mineworld_sdk::interactions`.
- **SD-25a-5 — The banner names the version, not the commit.** `<!-- GENERATED by mineworld reference
  from mineworld <version>. Do not edit … -->`. Deviation from step §5.1's `<git describe>` (A-9).
- **SD-25a-6 — Property order is name order** (A-4). Packs are in the installed set's order; within a
  type, fields are sorted by name. Deviation from step §5.3's "fields in declaration order".
- **SD-25a-7 — Examples come from the schema.** A type states its minimal example with
  `#[schemars(example = …)]`; the generator prints the JSON value as YAML (block style; a string is
  quoted when YAML would read it as anything else). An Interaction List section's example is generated
  from its declarations. Chosen over a per-type `const EXAMPLE: &str` because it adds no item to a public
  trait (QS25-2 allows exactly the bound).
- **SD-25a-8 — AC-25a-6 at the loader's decode seam.** Each printed example is decoded through the
  installed set's own `decode_section` / `decode_configuration` from a `serde-saphyr` stream — the
  loader's path — and each example with one unknown key added is refused with serde's `unknown field`,
  the refusal every page states for `deny_unknown_fields` types. Loading every example into one world
  would need a fixture naming every key every example references; that whole-world check is the
  guide's (25d), whose pages run the real binary.
- **SD-25a-9 — `--out` owns only what it generates.** It refuses a non-empty directory holding a file
  that is neither one it would write nor a generated file (a Markdown page starting with the banner, a
  `.schema.json` with the generated `$comment`); stale generated files are removed.

## 4. Test ownership

| Layer | Owns |
| --- | --- |
| Static (rustc) | A section or configuration type without `JsonSchema` is refused at the pack (`trybuild` case, C2) |
| Unit | Each manual `JsonSchema` impl lists exactly the keys its decoder accepts (C1); `parameters!`'s schema carries defaults and bounds (C2) |
| Integration (`tools/cli/tests/reference.rs`) | AC-25a-1 (`--check` on the checked-in tree), AC-25a-3, AC-25a-4, AC-25a-6, determinism, `--out` ownership |
| Mutation | AC-25a-2: a planted doc-comment change makes `--check` exit 1 naming the file |
| CI | `test` layer runs the reference test on Linux; `test-macos`, `test-windows` run it with `cargo test` |
| Gate 1 / Gate 2 | NOT REQUIRED: no LLM-facing semantics; no runtime lifecycle beyond the CLI integration test |

AC-25a-5 (worlds byte-identical) is owned by the existing parity check, `python3 scripts/ci_parity.py`.

## 5. Acceptance

AC-25a-1 … AC-25a-6 exactly as step-25 §10.3. Adversarial: A-S25-1 (a doc-comment change, a field
without a description, a pack without a schema: each red) and A-S25-4 (AC-8 baselines unchanged).

## 6. Commit plan (step §10.3's C0 … C5; C6 waits for DL-a)

- **C0 — Specs and decisions first.** This file and the handoff; `ARC-90` (reference part) and `DEP-49`
  in `docs/DECISIONS.md`; `MODULE_SPEC.md` §4.1, §4.2 and §8.1; `docs/README.md` row.
- **C1 — schemars on the framework's own files.** Workspace dependency; `mineworld_authoring::schema`;
  derives on `WorldManifest`, `WorldIdentity`, `AuthoredPassage`, `AuthoredLocation`, `AuthoredPosition`,
  `ClassDefinition`, `EntityClasses` (manual), `ConfigurationKey` (manual), `Attachment`; the content
  files' framework fields (manual); `mineworld-packages` identity types (manual), `LicencePolicy`,
  `PackFile`, `StyleManifest` and their public schema functions.
- **C2+C3 — The SDK bound and every pack's schemas (one green commit, as step C2 allows).** The bound on
  `AuthoredSection::Authored` and `PackConfiguration::Configuration`; `parameters!`; `Section<S>`'s
  schema; `Declarations`; the three `SystemPack` questions and `installed!`'s; derives and author-facing
  first paragraphs in every pack; test packs; `trybuild` case. `acme-fishing` updated, tagged and
  re-pinned.
- **C4 — `mineworld reference`.** `tools/cli/src/reference.rs` (+ rendering module), registration in
  `main.rs`, `tools/cli/tests/reference.rs`.
- **C5 — The checked-in reference and CI.** `docs/reference/`; `scripts/ci_layer.py` and
  `scripts/ci_changes.py`; AC-25a-2 mutation; AC-25a-5 parity.
- **C6 — after DL-a merges.** Not in this PR's head unless DL-a merges first (step §10.3). DL-a merged
  first (#191), so C6 is in this head (ledger §9).

## 7. Risks

- Parallel PRs (DL-a, FG-a, WE-a, TW-c, SET-b) add sections or configuration types: whichever lands
  later adds their schemas. origin/main is merged before C5 and before the final push.
- The third-party pack's tag must exist before the pin; pushing to `yuema137/mineworld-pack-fishing` was
  allowed to agents by the operator earlier (dispatch).

## 8. Execution contract

```text
PROJECT / PR              MVP-0 · S25 / PR 25a — the generated reference, Rust side
PRIMARY DESIGN DOC        .structured-coding/plans/mvp0/pr-s25a-reference.md (this file)
RELATED / BINDING DOCS    step-25-creator-docs.md (§§3, 4.1, 5, 10.3, 11, 13.1, 15, 16); CLAUDE.md §§2–4;
                          MODULE_SPEC §§3.1, 3.2, 4.1, 4.2, 8.1; DECISIONS ARC-31, ARC-33, ARC-61, ARC-63,
                          ARC-66, DEP-10, DEP-11, DEP-22
IMPLEMENTATION BASE       origin/main @ f2f0a71; merged with origin/main as it moves
APPROVED SCOPE            step §10.3 PR 25a (C0–C5); acme-fishing's section schema and its re-pin
FROZEN INVARIANTS         INV-S25-1, INV-S25-2, INV-S25-5; mineworld-contracts gains no dependency; the
                          installed! macro names no pack; no Deserialize behaviour changes; no world's
                          output changes
VALIDATION BUDGET         unit/integration unrestricted; one local `cargo test --workspace --no-fail-fast`
                          on the final executable head (dispatch); no model, no Gate
ENDPOINT AUTHORITY        implementation, commits, push, PR, CI repair: authorized (primary's dispatch,
                          2026-10-14); pushing a tag to mineworld-pack-fishing: authorized (dispatch);
                          merge: operator only — never by this session
STOP CONDITIONS           NORMAL: READY FOR OPERATOR REVIEW — DO NOT MERGE, with CI fast + test +
                          test-macos + test-windows + platforms + python green on the exact head.
                          MATERIAL: a public-contract change beyond §3, a new dependency beyond DEP-49,
                          a world whose output changes
POST-MERGE SYNC OWNER     the primary session (step and overall); this session: this file's ledger
```

---

## 9. Ledger (live)

### Commit ledger

- **C0** — [x] Implementation: this file, the handoff, `ARC-90` (reference part), `DEP-49`,
  `MODULE_SPEC.md` §4.1 (generated-reference paragraph), §4.2 (pointer to `interaction-list.md`), §8.1
  (`reference` row and paragraph), `docs/README.md` row · [x] Validation: `check_doc_headings.py` (223
  sections, none duplicated), `check_decision_ids.py` (124 ids, distinct); `cargo search schemars` →
  1.2.3, `cargo info` licence MIT (licence check `cargo deny` runs at C1, with the add) · [x] LLM review:
  `DEP-49` states both halves of the reuse question (adopt `schemars`; decline `clap-markdown` and our own
  derive); no field rule moved out of a spec — §4.1's examples and rules stay, the reference adds detail
- **C1** — [x] Implementation: `schemars` workspace entry (DEP-49); `authoring/src/schema.rs` (`Key`,
  `SystemName`, `TagName`, `TagList`, `Length`, `EntityTypeName`, `IDENTIFIER_PATTERN`), `JsonSchema` for
  `Attachment`, `ClassName`, `ClassDefinition` (derived), `EntityClasses`; `packages/src/schema.rs`
  (`PackId`, `PackType`, `Version`, `Compatibility`, `License`), `PackFile`, `StyleManifest`,
  `pack_file_schema`, `style_manifest_schema`; `worldpack/src/format.rs` derives on `WorldManifest`,
  `WorldIdentity`, `AuthoredPassage`, `AuthoredLocation`, `AuthoredPosition`, manual `ConfigurationKey`;
  `worldpack/src/configure.rs` `StatedPolicy` (the real decoder of `configure/packages.yaml`, not
  `LicencePolicy`, which is untouched: finding F-25a-1); `worldpack/src/schema.rs` (`world_manifest`,
  `content_file`, `classes`, `packages`, the four content-file mirrors) · [x] Validation:
  `each_content_files_schema_lists_exactly_the_fields_its_reader_accepts` (mirror keys = `content::fields`
  per kind, none required), `the_schemas_state_the_contracts_own_rules` (every listed entity type decodes
  by the contract, the identifier bound is the contract's constant); `cargo test -p mineworld-authoring -p
  mineworld-packages -p mineworld-worldpack` all ok; clippy `-D warnings` clean on the three; `cargo deny
  check`: licenses ok, bans ok, sources ok (advisories FAIL on `rustls` 0.23.45 via `ureq` in
  `tools/weather-fetch`, unrelated and present on the base) · [x] LLM review: the diff adds derives,
  attributes, doc wording and schema impls only; no `Deserialize` attribute or decoder changed
- **C2+C3** (one green commit, as step C2 allows) — [x] Implementation: the `JsonSchema` bound on
  `AuthoredSection::Authored` and `PackConfiguration::Configuration`; `Fields` and `Parameters` gain it
  (SD-25a-3); `parameters!` field grammar `$(#[doc = …])*` and its two `JsonSchema` impls through
  `sdk/rust/src/interactions/schema.rs` (`field`, `block`); `impl JsonSchema for Section<S>` and
  `Declarations` there; `SystemPack::{section_schema, configuration_schema, interaction_declarations}`
  defined by `owns_section!`, `configures!`, `interactions!`; the same three questions on `installed!`'s
  `Capability`; derives and author-facing first paragraphs in naming (`Name`), schedule (`TimeOfDay`,
  `AgendaLabel`, `Segments<EntityKey>` manual over the mirror `AuthoredSegment`), item (`Category`,
  `AuthoredItem`), inventory (`AuthoredHoldings`), economy (`AuthoredEconomy`, `AuthoredShop`),
  employment (`AuthoredJob` with `RawJob`), bodies (`Body` with `Written`, `Lies`, `Corner`, `Floor`,
  `Solid`, `HalfExtents`, `BodyShape` with `AuthoredShape`), calendar (`CalendarConfiguration` and its
  four field types), weather (`WeatherConfiguration` via `RawConfiguration`, `Source`,
  `RecordConfiguration`, `Rules`, `RawRules`, `Month`, `RawMonth` with ranges, `Quintiles`, `Fill`,
  `Station`); test packs (authoring, worldpack `load.rs` and `configure/tests.rs`, acceptance
  `configuration`); `trybuild` case `a_section_without_a_schema_does_not_compile`. `acme-fishing` 0.1.1
  (`yuema137/mineworld-pack-fishing` @ `2622137`, tag `v0.1.1`: `RawSpot`/`AuthoredSpot` schemas) and
  re-pinned in `systems/installed/Cargo.toml` and the lock (`src/lib.rs` unchanged: the type name did
  not change) · [x] Validation: before the fishing update `cargo check --workspace` failed exactly at
  `AuthoredSpot: JsonSchema` (the bound bites a third-party pack); after it, `cargo check --workspace
  --all-targets` and `cargo clippy --workspace --all-targets -D warnings` clean; tests of sdk,
  authoring, worldpack, the 13 affected packs and the installed set all pass; acceptance
  `configuration_seam` (4) and `package_sources` (11, the new pin) pass;
  `a_sections_schema_states_what_its_decoder_accepts` (the schema's example decodes through the real
  `Section<S>` decoder; defaults, bounds, description; region actions regional only; exactly the six
  keys); `the_sections_description_states_the_enforced_capacity` (inventory's text cites
  `PERSON_CAPACITY`); fishing's own `cargo test` green (5 + 2 + 2) · [x] LLM review: `installed!` still
  names no pack; the public-contract change visible to a pack author is the bound (QS25-2) plus three
  defaulted trait functions a pack never writes (its macros do); `parameters!` now admits only doc
  comments on fields (every use in the tree already did, A-7); no `Deserialize` attribute changed.
  Finding F-25a-2: a `try_from` container still compiles its own fields' schemas, so containers whose
  fields have no schema take `#[schemars(with = "Raw…")]` instead (employment, bodies, calendar,
  fishing). Finding F-25a-3: `acme-fishing` names `String` for `catch` (not `schema::Key`), because its
  standalone build pins MineWorld @ 6ca763d, which has no `schema` module; the fishing pack's own
  `[patch]` pin moves when 25a merges (follow-up, outside MineWorld)
- **C4** — [x] Implementation: `tools/cli/src/reference.rs` (`Target`, `reference`, `generate`, `write`
  with SD-25a-9's ownership rule, `check`, `listed`, `is_generated`) and `tools/cli/src/reference/`
  (`entries.rs`: what is documented, through `catalog::AVAILABLE` and the framework's schema functions,
  with `normalized` — first paragraphs, and refusal of an undescribed field or a missing example;
  `markdown.rs`: an entry's field table, the types it names, its example; `yaml.rs`: SD-25a-7's printer;
  `cli_page.rs`: clap's `Command` walked, `help` and `--help` left out; `pages.rs`: the seven pages, 22
  schema files, the index); `Reference { out, check }` in `main.rs` (an `ArgGroup` requires exactly
  one); `mineworld-cli` names `schemars` (DEP-49), `serde`, and `serde-saphyr` as a dev-dependency (all
  three already in the workspace and its lock); `StyleManifest` gains its example; author-facing first
  paragraphs for `AuthoredSegment`, calendar's `Epoch`, `UtcOffset`, `Latitude`, `Longitude`, weather's
  `Quintiles`, conversation's `remember` knob, and the SDK's section description ("keeps the pack's own
  defaults") · [x] Validation: `tools/cli/tests/reference.rs` (6 tests, 2.1 s): determinism (two `--out`
  byte-equal, no `\r`, `--check` clean), `--check` naming `differs`/`missing`/`extra` with exit code 1,
  `--out` ownership (a foreign `notes.txt` refuses and writes nothing; a stale banner page and a stale
  `$comment` schema are removed), AC-25a-4 (headings of `sections.md`, `configure.md`,
  `interaction-list.md` equal to `AVAILABLE`'s sections, configurations and Interaction List sections,
  `fishing` among them), AC-25a-6 (every section example decoded by `owning_section` +
  `decode_section`, every configuration by `decode_configuration`, classes by `EntityClasses`, all from
  a `serde-saphyr` stream; each predicted refusal holds with one unlisted key: 10 entries), the World
  Pack page's examples composed into a world that `mineworld validate` accepts (with `classes` and
  `packages` from `configure.md`) and the `pack.yaml` example into an Entity Pack `mineworld packs
  validate` accepts, each refusing one unlisted key with the predicted message; unit tests in the binary
  (4): AC-25a-3 `a_field_without_a_description_refuses_generation_by_name` (a nested type's undescribed
  field, named `Inner.depth`), first-paragraph rewriting (examples untouched), the YAML printer read back
  through `serde-saphyr`, two in-process generations equal; `cargo test` of sdk, calendar, weather,
  conversation, schedule, packages all ok; `cargo clippy --workspace --all-targets --all-features -D
  warnings` clean; `cargo fmt --check` clean · [x] LLM review: the command reads types only — no
  `WorldPack` is read or loaded (`entries.rs` imports `schema`, `catalog`, `Capability`); it writes only
  under `--out`, after listing it, and writes nothing when a foreign file is there; `--check` writes
  nothing; output is a function of the build (BTreeMap order, `serde_json` without `preserve_order`,
  the version, no time, no commit). Findings F-25a-4…F-25a-7 below
- **C4** findings:
  - F-25a-4 — an Interaction List section's decoder refuses an unlisted key with its own words
    (`'x' is not a key of a section (…)`, `sdk/rust/src/interactions/section.rs` `SectionKey`), not
    serde's `unknown field`. The page states the decoder's words; the test reads the predicted message
    off the page and holds the decoder to it, so a reworded decoder turns the test red. No decoder changed.
  - F-25a-5 — an enum variant written `{ box: … }` is an object of one property whose doc comment is
    the object's (`bodies::BodyShape`); inside a `oneOf`/`anyOf` such a property needs no description of
    its own. Everywhere else a property without one refuses generation.
  - F-25a-6 — every root schema must state an example (the generator refuses one without, naming it);
    only `StyleManifest` had none, and gained one.
  - F-25a-7 — an Interaction List section's full schema walk repeats the shared shape once per pack and
    once more per region (~500 lines for three packs). The page states the shared shape once, from the
    first section's own schema, then per pack only its declarations (actions, facts, roles, parameters,
    knobs, reference lists) and its example; the JSON Schema under `schema/` keeps the whole shape.
- **C5** — [x] Implementation: origin/main @ 31ba514 merged first (FG-b: Python and docs only, no
  section or configuration added); `docs/reference/` generated (7 pages, 22 schemas, 29 files);
  `the_checked_in_reference_is_what_this_build_generates` in `tools/cli/tests/reference.rs` (AC-25a-1);
  `scripts/ci_changes.py` classifies `docs/reference/` as code, with two self-test cases (found
  uncommitted from the first implementer, inspected and kept); `scripts/ci_layer.py` unchanged (deviation
  below) · [x] Validation: `tools/cli/tests/reference.rs` 7 passed; `ci_changes.py --self-test` 22 cases
  passed; `check_doc_headings.py` (223 sections, 38 documents, none duplicated), `check_decision_ids.py`
  (124 ids distinct); acceptance `precursor_vocabulary`, `configuration_vocabulary`, `seam_vocabulary`,
  `interaction_schema` pass with the new pages; `cargo deny check licenses sources bans`: bans ok,
  licenses ok, sources ok; mutations M1–M4 below; AC-25a-5 parity and the full workspace suite in the
  Evidence section · [x] LLM review: three entries read as a person who has not seen the code —
  `sections.md` `job` (each field's meaning and unit: minor units per hour, HH:MM quoted, an
  organization key), `configure.md` `configure/calendar.yaml` (epoch range, UTC offset quoted, degrees
  with sign conventions), `interaction-list.md` `configure/conversation.yaml` (the action `talk` and its
  roles, the facts with their audiences, `gap`/`range`/`remembered` with defaults, bounds and units): each
  says what to write and what it means, without a code name; the earlier developer-facing first
  paragraphs (`AuthoredSegment`, `Epoch`, `UtcOffset`, `Quintiles`, the `remember` knob) were rewritten at
  C4

- **Merge of origin/main @ f772a18** (`54ec37d`; DL-a, FG-a, 25b, SET-b, S26/S27 designs) — found
  half-done by the third session (MERGE_HEAD present, three files unmerged with their conflicts already
  resolved in the tree but not staged); inspected, kept and concluded. `parameters!` now takes FG-a's
  `ParameterBound` (`$bound:expr`), and its schema comes from `ParameterBound::schema` (a range's
  `minimum`/`maximum`; FG-a's `CurveCatalog` its `enum`); `relationships::CurveName` and DL-a's
  `DecisionsFile` gained schemas (the build failed without them, the bound biting as designed).
- **C6** (DL-a merged before this PR, so C6 is in this head per §6) — [x] Implementation:
  `ModelParameters` requires `JsonSchema` and answers `schema` (`()`: an empty mapping); `Catalog` gains
  `parameters_schema`, written by `models!`; `cognition/decision/src/schema.rs` `file_schema::<C>()`,
  hand-written beside the hand-written reader (`models` per installed model: in-process with its
  `parameters`, external with its required `fallback`; `default`; `assign`; `memory`); `decisions.md`
  and `schema/decisions.schema.json` (entries `decisions`, page and index row); `MODULE_SPEC.md` §§4.1,
  5.1, 8.1 · [x] Validation: `the_schema_states_what_the_reader_accepts` (mineworld-decision: the example
  read and checked by `DecisionList::parse` against a roster; the schema's keys equal the reader's
  `FILE_KEYS`, `ENTRY_KEYS` split by kind, and each raw struct's fields; a `parameters!` model's default
  and bounds; an unlisted key refused with `unknown field`);
  `the_decision_lists_example_is_read_and_an_unlisted_key_is_refused_as_predicted` (tools/cli: AC-25a-1,
  -4, -6 for `decisions.yaml` with the installed models); the generator refused `DecisionList.memory`
  until it had a description (AC-25a-3 biting on a hand-written schema) · [x] LLM review: the page names
  models (`rule`), never a backend, provider, URL or key (MODULE_SPEC §4.1); no decoder changed (the
  reader's two key lists became `pub(crate)` for the test only)
- **Bounded correction (after the merge) — the cognition half's files.** 25b put `cognition-toml.md`
  and `schema/cognition-toml.schema.json` into `docs/reference/`, which `--check` called `extra` (step-25
  §17's open row). Step §5.1 places both halves' pages in the one directory and the index is 25a's, so:
  `pages::COGNITION` names those two files; neither `--out` nor `--check` writes, removes or compares them
  (their drift test is 25b's pytest); the index links them. Test
  `the_cognition_halfs_files_are_left_to_it`; `MODULE_SPEC.md` §8.1 says so. A later 25b page
  (`memory-policies.md`) adds its name to `COGNITION`.
- **Bounded correction — a named choice is shown as its choices.** A field whose bound narrows its type
  (`familiarity_decay`: `$ref CurveName` + `enum`) printed only the type's link; `type_of` now prefers
  `enum` over `$ref` ("one of `none`, `linear`, `half-life`, `power`").
- **Bounded correction — the second merger's schema (after the origin/main merge).** `MotivatedParameters`
  (DL-b) did not compile under `ModelParameters: JsonSchema`. It and its nested `Base` and `Roles` derive
  `JsonSchema` (`schemars` added to `cognition/motivated`), with each documented bound as a schema range
  (`top_band` 0 … 500, `commit_minutes` 15 … 1 440, `social_need_hours` 1 … 72, `reply_floor` 0 … 1 000,
  each `base` option 0 … 1 000), `curves` as `enum: [atus-a3-2025]`, `roles` at most eight tags, `settings`
  skipped, and every key optional (the block keeps its defaults). `file_schema` carries the generator's
  definitions into `$defs`, so a model's nested types link to their own sections. FG-b2's `store_bound_mib`
  is added to `MemoryPolicy` and `MemoryAssignment` in the hand-written schema (20 … 1 024, the cognition
  host's bound; the Rust reader passes it through without judging it, so the schema states a range the
  reader does not itself refuse). `read_tests.rs` lists the two keys.
- Post-merge validation (R-LOAD-1: touched crates only): `cargo check --workspace --all-targets`,
  `cargo clippy --workspace --all-targets --all-features -D warnings` clean, `cargo fmt --check` clean,
  `cargo deny check licenses sources bans` ok; `cargo test` of mineworld-decision (18),
  mineworld-installed-models, mineworld-rule-controller, mineworld-relationships, mineworld-sdk all ok;
  `mineworld-cli --test reference` 9 passed, its binary's reference unit tests 4 passed; `mineworld
  reference --check docs/reference` → "matches this build (31 file(s))"; `check_doc_headings.py` (223
  sections, 40 documents), `check_decision_ids.py` (124 ids), `ci_changes.py --self-test` (22) pass.
  Stale objects of two crates in the shared target directory (linker errors in group-activity and
  weather rlibs from an interrupted build) were cleaned with `cargo clean -p`; environment, not code.

### Evidence

- **AC-25a-5 (INV-S25-5, A-S25-4) — PASS for every world's simulation; one expected `validate` line.**
  `ci_parity.py record` (default profile, release binaries, Darwin/arm64, load average ≈ 160–210 under
  R-LOAD-1, 15 min for the head and 59 min for the base) on the head a45b039 (C4 merged with origin/main;
  C5 adds only `docs/reference/`, the test and the classifier) and on the base origin/main @ 31ba514 (a
  separate detached worktree), then `ci_parity.py compare base head`: 2 375 keys compared; bodies-yard (664
  keys), market-town (598) and social-cafe (557) equal; lakeside equal in every run, fact and summary key
  and different only in `validate`. Cause, by running both binaries' `validate worlds/lakeside` from one
  tree: the one line `requires acme-fishing "^0.1" → 0.1.0` became `→ 0.1.1` — the third-party pack's
  version, which QS25-2's re-pin changes by construction. A package version is never world state
  (`ARC-53`); `scripts/baselines.txt` holds no `validate` key, so the nightly's baselines are unchanged.
  `baseline check` itself needs a long record (`profile default; baselines are read from a long
  record`); a long record was not made, under R-LOAD-1 item 4 — the default-profile comparison against the
  base is the evidence. Records: scratchpad `parity-base.txt`, `parity-record.txt` (not kept in the
  repository).
- **Workspace gate:** under the coordinator's ruling R-LOAD-1 (2026-10-14: no local `cargo test
  --workspace`), the local full run started on 393a777 was stopped after 50 test binaries had reported
  (no failure among them) and is not evidence. The gate is CI's `test`, `test-macos` and `test-windows` on
  the exact PR head, recorded at READY.
- Targeted local runs (all on the C4/C5 tree): `mineworld-cli` `--test reference` 7 passed, its binary's
  unit tests 11 passed; sdk, calendar, weather, conversation, schedule, packages tests ok; acceptance
  `precursor_vocabulary`, `configuration_vocabulary`, `seam_vocabulary`, `interaction_schema` ok; fmt and
  clippy `-D warnings` clean; `cargo deny check licenses sources bans` ok.

### Mutations

All planted on the C5 tree (merged with origin/main @ 31ba514), each reverted, the tree checked clean
again after (`mineworld reference --check docs/reference` → "matches this build (29 file(s))", exit 0).

| # | Mutation | Expected | Observed | Verdict |
| --- | --- | --- | --- | --- |
| M1 (AC-25a-2) | `economy` `wallet`'s doc comment gains " MUTATED." | `--check` exits 1 naming the page and the schema | exit 1: `differs  schema/section-economy.schema.json`, `differs  sections.md`; `the_checked_in_reference_is_what_this_build_generates` FAILED with the same two lines | killed |
| M2 (AC-25a-3, A-S25-1) | calendar's `epoch` field loses its doc comment (its type `Epoch` keeps one) | generation refused, naming the type and the field | exit 1: "`configure/calendar.yaml`: the field CalendarConfiguration.epoch has no description; write an author-facing doc comment on it" | killed |
| M3 (INV-S25-1) | a hand edit appended to `docs/reference/README.md` | `--check` exits 1 naming it | exit 1: `differs  README.md` | killed |
| M4 (step §5.4) | `ci_changes.py`'s generated-prefix rule disabled | the classifier's self-test fails on the two `docs/reference/` cases | `FAILED: 2 case(s)`: `docs/reference/sections.md` and `docs/reference/schema/world.schema.json` classified documentation | killed |
| M5 (A-S25-1, a pack without a schema) | owned by C2+C3's `trybuild` case `a_section_without_a_schema_does_not_compile` (the bound bites at compile time), and by the observed refusal of `acme-fishing` 0.1.0 at `AuthoredSpot: JsonSchema` before the re-pin | — | recorded at C2+C3 | killed |

### Deviations and findings

- SD-25a-5 and SD-25a-6 deviate from step §§5.1, 5.3 for the reasons A-9 and A-4 give.
- Session change (2026-10-14): the first implementer stopped after C2+C3 and was cancelled; a
  replacement session recovered the tree (HEAD 44d0cf7, one uncommitted file: `scripts/ci_changes.py`,
  the C5 classifier rule with its two self-test cases — inspected, correct, kept and committed at C5)
  and continued from C4. No other state was found.
- Deviation (C5, bounded): `scripts/ci_layer.py` is unchanged. Step §10.3 C5 says "add the `--check` test
  to the `test` layer"; the `core` layer that the `test`, `test-macos` and `test-windows` jobs run is
  `cargo test --workspace`, which already runs `tools/cli/tests/reference.rs` and its
  `the_checked_in_reference_is_what_this_build_generates` on all three platforms (AC-25a-1). A second,
  named command would run the same test twice (test rules §9). Evidence: `ci_layer.py` `LAYERS["core"]`,
  `.github/workflows/ci.yml` jobs `test`, `test-windows`, `test-macos` (`layer: core`).
- Merge of `origin/main` @ 5cbb469 (2026-10-17; branch head 589998e, merge base 2d07c6c). One textual
  conflict: `docs/DECISIONS.md`, where this branch's `ARC-90`/`DEP-49` and main's `ARC-72`/`ARC-73` were
  inserted at the same place (before `ARC-97`). Resolved by keeping all four records, ours first, with a
  `---` rule between them; `check_decision_ids.py`: 151 ids, all distinct. One semantic conflict found by
  the drift test: main (S10 P6b) added `external "lm"` to `cognition/installed`, so the generated
  `docs/reference/decisions.md` and `schema/decisions.schema.json` no longer matched the merged build
  (`the_checked_in_reference_is_what_this_build_generates` FAILED naming the two files). Regenerated from
  the merged binary with `mineworld reference --out` into a scratch directory and copied over (never
  hand-edited); `mineworld reference --check docs/reference` → "matches this build (31 file(s))". Main did
  not touch `Cargo.lock` or `scripts/baselines.txt` since the merge base, so no lock regeneration or
  baseline re-record was needed (`cargo metadata --locked` and `cargo fetch --locked` pass); main's sharded
  `.github/workflows/ci.yml` (#224) taken as-is (this PR does not edit it). Evidence on the merged tree:
  `mineworld-cli --test reference` 9 passed, the binary's reference unit tests 4 passed; mineworld-decision
  (18), installed-models, motivated tests ok; clippy `-D warnings` on mineworld-cli, decision, motivated
  clean; `cargo fmt --check`; `test_reference.py` (cognition-toml drift, 6) passed; doc headings, decision
  ids, `check_ci_pins`, `check_scratch scan`, `ci_changes`/`ci_shard` self-tests pass.
- Merges of `origin/main` @ 04d5145 (`2783bbb`: S10 P6a #236, S30-b #247) and @ e51ffc0 (`81c9315`: S10 P7
  #244, S27-b #241, S11e #243, #248, #216), 2026-10-17. Textual conflicts: none (`check_decision_ids.py`: 154
  ids, all distinct). Two semantic conflicts, found by CI on `2783bbb` (scratch run 39070189129: `fast`,
  `test`, `test-macos`, `test-windows` red; `python`, `platforms` green), both real defects, neither a flake:
  (1) S30-b's test-only pack `test-schema` (`tests/acceptance/tests/schema/mod.rs`) configures
  `SchemaSettings`, which the `PackConfiguration::Configuration: JsonSchema` bound refuses (E0277, the bound
  biting as designed); it derives `JsonSchema` and its field gains a description. (2) S10 P6a's `persona`
  section derives a schema but states no example, so every reference test failed with "`persona` (Persona)
  states no example" (F-25a-6's rule); `Persona` gains `#[schemars(example = …)]`, and it and its four
  field types gain author-facing first paragraphs with the bounds in numbers (the rustdoc links to the
  constants move to a second paragraph, which the reference does not print). Commit `2e7720a`. The reference
  was regenerated on GitHub CI, never locally (operator pause on heavy local jobs, 2026-10-17): scratch
  branch `scratch/s25-regen-25a` (`2e7720a` + a temporary workflow, `ci.yml`/`nightly.yml` removed on that
  branch only), run 39099469556: `cargo check --workspace --all-targets` ok; `mineworld reference --out
  docs/reference` → 32 files, `--check` → matches; `README.md`, `sections.md` changed and
  `schema/section-persona.schema.json` added; then `mineworld-cli --test reference` 9 passed and `clippy
  --workspace --all-targets --all-features -D warnings` clean on the regenerated tree. The artifact was
  downloaded with `gh run download` and copied over unchanged (`diff -rq` against the artifact: identical);
  the scratch branch was deleted.
- **Independent review (2026-10-18, REQUEST CHANGES), addressed in one head.** (1) Merge of `origin/main` @
  bcc689c (S26-a #206, the `appearance` System Pack; `03b7e03`): conflicts in `docs/DECISIONS.md` (ARC-90/DEP-49 and
  ARC-91/ARC-92 kept; 156 ids distinct), `systems/installed/Cargo.toml` (fishing v0.1.1 pin kept, appearance
  added), `tools/cli/Cargo.toml`, `worldpack/Cargo.toml` (both sides kept). Semantic conflict: `Appearance` and
  `LookKey` derived only `Deserialize`; both derive `JsonSchema` (`LookKey` states `^[a-z0-9-]+$`, 1 … 48 bytes,
  the decoder's own bound `LOOK_MAX_BYTES`), `Appearance` states its example, both have author-facing first
  paragraphs; `mineworld-appearance` names `schemars` (lock updated by `cargo metadata`, no build). (2) `entries.rs`
  no longer drops a capability whose `section()` or `configuration()` is `Some` without a schema: generation
  refuses, naming the pack and the file (today reachable only by an in-context section, F-25a-8). (3)
  `sdk/rust/tests/compile_fail/a_configuration_without_a_schema_does_not_compile.rs` beside the section case. (4)
  `authoring/src/schema.rs` uses `mineworld_contracts::MAX_IDENTIFIER_LENGTH` directly; the copy and its
  assertion are gone (`94932bd`). Generated on GitHub CI, never locally: scratch run 39117734295 (`cargo check
  --workspace --all-targets` ok; `reference --out` → 33 files, `sections.md`/`README.md` changed,
  `schema/section-appearance.schema.json` added, `--check` matches; `TRYBUILD=overwrite` wrote the new case's
  `.stderr`; then `mineworld-cli --test reference` 9, `--test appearance` 3, `mineworld-sdk --test compile_fail` (3
  cases ok), `mineworld-appearance` 4 passed; `cargo fmt --all --check` and `clippy -D warnings` clean). The
  artifact was copied unchanged (`diff -rq`: identical); the scratch branch was deleted.
- **F-25a-8 (for the primary) — S30-b's data half has no schema bound.** `SectionInContext::Authored` and
  `OwnsData::File` (`authoring/src/data.rs`) carry no `JsonSchema` bound, and `owns_section_in_context!` /
  `owns_data!` answer no schema question, so a pack's data directory files and its in-context section would
  not appear in the reference. No pack in the installed set uses either today (only `test-schema`), so the
  reference is complete for this build. Extending the bound is a public SDK contract change (QS25-2 named
  exactly two bounds) and is left to whoever lands the first installed data-owning pack.
- Deviation (C4, bounded): the CLI page is generated from the parser with `Command::build()`, so the
  parser's own `help` subcommands and `--help`/`--version` arguments are left out by name.
- Known limitation: a `Millimetres` type prints its full `i32` range as bounds; each field's description
  states the field's real bound (the decoders enforce it). Narrowing the schema to the decoder's bound
  per field is a follow-up for whoever touches those types.
