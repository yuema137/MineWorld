## ARC-90 — Creator documentation: a generated reference that cannot drift (reference part)

**Date** 2026-10-14 · **Status** accepted (step-25 design); the reference part implemented by PR 25a; the guide part (`docs/guide/` executed by CI, the onboarding test) is added to this record by PR 25c · **Approved by** the operator's rulings QS25-1, QS25-2, QS25-6 of 2026-10-14 (step-25 §13.1) · **Relates to** `ARC-31`, `ARC-33`, `ARC-61`, `ARC-63`, `ARC-66`, `DEP-49`, `CLAUDE.md` §2.1 · **Design** step-25 §§4.1, 5; `pr-s25a-reference.md`

**Problem.** Every file a creator writes is decoded by a typed value whose owner validates it, but what each file may say was written down by hand in `MODULE_SPEC.md` §4.1 for the framework's own fields and nowhere at all for the sections, configurations and Interaction List sections the packs own. A hand-written field list drifts the first time a pack adds a field (risk R-3 of `overall.md`).

**Options considered.** R1 derive a JSON Schema from each decoding type and render it (chosen); R2 a MineWorld describe trait and derive macro (reinvents `schemars`); R3 scrape rustdoc JSON (nightly toolchain, sees types but not serde's behaviour); R4 a hand-written reference with a guard (an undocumented field is never caught); R5 schema-first code generation (inverts ownership: the pack's type would no longer be the owner's validation, contradicting `ARC-31` and `ARC-61`).

**Choice.**

1. **The reference is derived from the types that decode the files** and is never edited by hand. `docs/reference/` is written by `mineworld reference --out DIR`; `mineworld reference --check DIR` regenerates it in memory and exits 1 naming every file that differs, is missing or is extra. CI runs the check, and a change to `docs/reference/` alone still runs it.
2. **The SDK requires a schema** (QS25-2): `AuthoredSection::Authored` and `PackConfiguration::Configuration` are bound by `schemars::JsonSchema`, so a pack whose section or configuration has no schema does not compile. `parameters!` writes the schema of its block itself, with each field's default, bounds and doc comment.
3. **The installed set answers for every pack.** `installed!` gains `section_schema`, `configuration_schema` and `interaction_declarations`, defined by the macros a pack already writes (`owns_section!`, `configures!`, `interactions!`); the generator walks `Capability`'s list and names no pack.
4. **A field's description is the first paragraph of its doc comment**, written for an author, with its unit; a field with none fails generation. Later paragraphs are for developers and reach neither the Markdown nor the JSON Schemas.
5. **The command is read-only with respect to worlds**: it loads no World Pack, and its output is a function of the build alone — the installed set's order, fields in name order, no timestamp and no commit hash, `\n` line endings on every OS. It writes only into its `--out` directory and refuses one holding a file it did not generate.
6. **English only** (QS25-6): the reference is generated from English doc comments and has no `*.zh-CN.md` mirror.
7. **The reference describes this build.** A build with another installed set generates a different reference (`ARC-33`, `ARC-66`); the checked-in one describes the repository's own installed set.

**Consequences.** A specification states shape and rules; field-by-field detail is the generated reference's (`MODULE_SPEC.md` §§4.1, 4.2). `mineworld-contracts` gains no dependency: a contract-typed field names its schema through `mineworld_authoring::schema`. A third-party pack derives `JsonSchema` on its section and configuration types (`acme-fishing` 0.1.1 does).

**Revisit** when a public stable SDK contract is declared (the bound is then part of it), or when the reference must describe a build other than the repository's own.
