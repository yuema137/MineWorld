## ARC-53 — A pack's identity is stated once, where the pack already states who it is

**Date** 2026-10-08 · **Approved by** the primary session at PR E-a's design freeze (step-16 §14.0;
QSE-4, QSE-5, QSE-6 and FQ-1 … FQ-5 as ruled) · **Implements** [`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md)
§5 (MVP-0 subset), [`MODULE_SPEC.md`](MODULE_SPEC.md) §3.1, §4.1, §6.1, §8.1, §9 · **Relates to**
`ARC-3`, `ARC-33`, `ARC-35`, `DEP-12`, `DEP-21` · **Design**
`.structured-coding/plans/mvp0/step-16-packages.md` §4.1, §14 (S16, PR E-a)

**Problem.** Milestone E needs packs that can be told apart, versioned and checked: "independently
installable" means nothing for a pack that has no identity. Until this decision no pack had one. Every
crate was version `0.0.0` (`[workspace.package]`); a World Pack had an id and a name; a Presentation
Pack had only its style's id; nothing stated a licence per pack. `PACKAGE_FORMAT.md` §5 specified a
manifest, but three carriers already state who a pack is, and a second manifest beside each would
state every fact twice.

**Choice.**

1. **One vocabulary.** Every pack has these package fields, the MVP-0 subset of `PACKAGE_FORMAT.md`
   §5 and `MODULE_SPEC.md` §9:

   ```text
   id          the pack's id: 1–64 characters of a–z, 0–9 and '-', beginning with a letter, no '--',
               not ending in '-'. A code pack's id is its Cargo package name
   version     semver, MAJOR.MINOR.PATCH
   type        system-pack | controller-pack | world-pack | presentation-pack | entity-pack
               (asset-pack refused by name in MVP-0)
   mineworld   the framework versions the pack works with, a semver range (data packs)
   license     an SPDX licence expression
   provenance  authors, and the repository the pack comes from where stated
   ```

2. **Three carriers, so no fact is stated twice.**
   - **A code pack** (System Pack, Controller Pack) is identified by its `Cargo.toml` `[package]`.
     The crate `mineworld-packages` (`packages/`) defines `Package`, a record of `&'static str`, and
     the expression macro `package!()`, which expands to `Package::declared(env!("CARGO_PKG_NAME"),
     env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_LICENSE"), env!("CARGO_PKG_AUTHORS"),
     env!("CARGO_PKG_REPOSITORY"))`. `env!` is expanded where the macro is invoked, so each pack
     records its own crate's fields at compile time; the binary never runs Cargo and never reads a
     source tree. `mineworld-sdk` re-exports both. **`SystemPack::PACKAGE` is a required constant**:
     a System Pack writes, as the first line of its `impl SystemPack`,

     ```rust
     const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
     ```

     and a pack without it does not compile. The installed set's `Capability` gains `package()` and
     `version()`. The rule controller writes `pub const PACKAGE: mineworld_packages::Package =
     mineworld_packages::package!();` and depends on `mineworld-packages` directly, not on the SDK:
     the SDK is how a *System* Pack declares itself, a controller is not one, and its manifest names
     no kernel, transport or server directly (it reaches the kernel only through the packs whose
     vocabulary it uses).
   - **A World Pack** is identified by its `world.yaml`: `world.id` (unchanged: its directory's name),
     `world.version`, `world.license`, and the top-level `mineworld:` range.
   - **A Presentation Pack** is identified by a `pack.yaml` at its root, beside the unchanged style
     manifest (`manifest.yaml`, `ART_DIRECTION.md` §12, whose own `id` stays the style's id). Its `id`
     is stated in `pack.yaml` and its directory name is free: `ARC-3`'s family layout puts dimension
     packs in `2D/` and `3D/`, which are not ids. Entity Packs use the same file from S16's PR E-d.

3. **The framework has a release version: 0.1.0.** Every framework crate and every bundled pack share
   it (`[workspace.package] version`), with pre-1.0 semantics: a MINOR bump may break. A `mineworld:`
   range is checked against `mineworld_packages::FRAMEWORK_VERSION`, the version the running binary
   was built with.
4. **`SystemVersion` and the release version stay two things.** `SystemVersion` is the contract
   counter a save is checked against (`ARC-25`); the semver version is what a range is checked against.
   Raising a pack's `SystemVersion`, which makes existing saves refuse, is a breaking release: a MINOR
   bump before 1.0 and a MAJOR bump from 1.0, so a range like `^0.1` never admits a pack whose saves
   are incompatible.
5. **Validated once, at run time, by one crate.** `Package` is a plain record. `mineworld-packages`
   turns it, a `pack.yaml` and a world's fields into one validated `Identity` through the same types:
   `Version` and `Compatibility` (the `semver` crate) and `License` (the `spdx` crate, `DEP-21`). A code
   pack with an empty or unparseable licence compiles, and `mineworld packs list` refuses it, naming
   the pack and the text.
6. **Refused by name, never accepted unchecked.** A World Pack's three fields are optional to the
   loader in MVP-0, so existing test worlds and `mineworld create`'s template read unchanged, but when
   present each is typed and checked: a malformed version or licence is refused at its line and
   column, and a `mineworld:` range the framework does not satisfy is refused naming the range and the
   framework version. `mineworld packs validate` requires all three. A `pack.yaml` refuses an unknown
   field (`dependencies:` included, until requirements are resolved), a missing field, a malformed
   value, and a `type` it does not carry, naming the file where that type is identified instead.
7. **Identity never enters a fact or a save** (step-16 QSE-14). The `PACKAGE` constants are read by
   `mineworld packs` only. A save's composition is still checked by `SystemVersion` alone.
8. **`mineworld packs list | show | validate`** (`MODULE_SPEC.md` §8.1) prints identities: the build's
   code packs, and the data packs in each directory named by `--packs DIR`, which is given explicitly
   and never implied.

`ARC-33` point 1's list of the SDK's dependencies gains `mineworld-packages`, a leaf that depends on no
MineWorld crate; the SDK still never depends on a pack.

**Not in this decision.** A world's `requires:` and its resolution, pack roots on other commands, the
licence allow-list, bundled-versus-third-party classification, Entity Packs and packs from outside
this repository: later PRs of S16, each recorded when it lands.

**Accepted limitations.**
- A World Pack's fields are optional to the loader until requirements are resolved; only `packs
  validate` requires them.
- A World Pack has no authors field, so its provenance is its directory.
- The framework repository is not stated in the workspace's `[package]` fields, so a bundled code
  pack shows no repository.

---

## ARC-53 note — classification moved to E-b (2026-10-08)

Point 2's identity record gains one field in S16's PR E-b: `Package::bundled`, computed by `package!()`
at compile time (`ARC-54` point 2). Step-16 §14.1 had placed bundled-versus-third-party classification in
E-c; E-b needs it for two of its rules, and the primary session moved it at E-b's freeze (FQ-b3). No pack's
source changes: only the macro's expansion does.
