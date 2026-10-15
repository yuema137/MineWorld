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
- **C6 — after DL-a merges.** Not in this PR's head unless DL-a merges first (step §10.3).

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
- **C4** — [ ] Implementation · [ ] Validation · [ ] LLM review
- **C5** — [ ] Implementation · [ ] Validation · [ ] LLM review

### Evidence

(filled per commit)

### Mutations

(filled at C4/C5)

### Deviations and findings

- SD-25a-5 and SD-25a-6 deviate from step §§5.1, 5.3 for the reasons A-9 and A-4 give.
