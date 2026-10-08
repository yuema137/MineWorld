# Step 16 — Package composition: a world assembled from independently installable packs (S16, Milestone E)

**Role:** step document for a new step, proposed as **S16 — Package composition**. It delivers
Milestone E ("a real world assembled from independently installable packs",
[`docs/HUMAN_REVIEW_QUEUE.md`](../../../docs/HUMAN_REVIEW_QUEUE.md) milestone table), which no step in
[`overall.md`](overall.md) delivers today. It records the requirement and its candidate readings, an
audit of what exists against what the specifications describe, a spike, the design of the recommended
reading, a reuse comparison in both directions, a modularity statement, invariants, acceptance
criteria and the milestone test, a PR split, risks, the open questions, and a proposed amendment to
`overall.md`.
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) §1 (non-goals), §3, §7.
**Lifecycle:** `DRAFT — awaiting operator agreement`. A new step needs the operator's agreement on its
requirement before any PR is detailed. Nothing in this document authorizes implementation.
**Base audited:** `main @ 0fd0be3` (branch `plan/mE-packages`).

**Identifiers in this document are placeholders.** Decision records are `ARC-SE-a`, `ARC-SE-b`,
`DEP-SE-a`, `DEP-SE-b`; PRs are `E-a` … `E-e`; questions are `QSE-n`. The primary session assigns real
`ARC-`/`DEP-` numbers and PR numbers at freeze (`scripts/check_decision_ids.py`).

**Parallel-safety.** This document is the only file this planning session writes. It proposes edits to
`overall.md`, `docs/MVP_STATUS.md`, `docs/DECISIONS.md`, `docs/PACKAGE_FORMAT.md` and
`docs/MODULE_SPEC.md` as text (§13, and inside §§4–5); it applies none of them.

**Operator directive carried into this document (2026-10-08, relayed by the coordinator).** "Keep code
clean, modular and pluggable. Look for open-source solutions first, compare several implementations
instead of stopping at the first, and build our own only if nothing existing serves the goal." It is
answered by §5 (a reuse comparison of fifteen options, each with a verdict) and §6 (what
"independently installable" means exactly, and what can be added or removed without touching
anything else).

---

# 1. The requirement

## 1.1 Verbatim

`docs/HUMAN_REVIEW_QUEUE.md`, framework milestones:

```text
E   Package composition   a real world assembled from independently installable packs   ❌
```

The frozen top-level criterion (`CLAUDE.md` §1, `overall.md` §1) it builds on:

> The framework must demonstrate that materially different games can be constructed by composing the
> same core entities with different independently installable interaction systems, without modifying
> the kernel.

and the extension model it must make true (`CLAUDE.md` §1, `MODULE_SPEC.md` §1):

```text
install modules → compose world → configure → run        never   fork source → edit game code
```

## 1.2 What is already decided, and binds every reading

- **`ARC-33` — installing is static in MVP-0.** A System Pack is installed by a directory, two lines in
  `systems/installed`, and a rebuild. Installing without a rebuild is `ARC-8` (WASM, Tier 1), a
  non-goal (`overall.md` §1: "WASM plugin sandbox").
- **`ARC-33` also says, explicitly:** "So is Milestone E's publishing sense of 'a real world assembled
  from independently installable packs': `.mwpack`, a registry, and packs from outside this
  repository." Step-10 §2.2 says the same. **Any reading that installs a pack from outside this
  repository therefore revises one sentence of an accepted decision**, and that is operator-material
  (QSE-2).
- **Non-goals** that bound every reading: WASM plugin sandbox; Phase 2 world-creator GUI; Phase 3
  registry; Phase 4 public worlds.
- **`AC-1` is accepted** (QS-65, 2026-10-07) as `ARC-35` measures it. S16 must not reopen it, and must
  not change the two transformation merges' evidence or the `ac1_composability` test's meaning.

## 1.3 Three candidate readings

Two words carry the milestone: **"real world"** and **"independently installable"**.

### E-A — "Already true": Milestone E is S9's evidence, relabelled

- *Real world:* Market Town.
- *Independently installable:* `ARC-33` — six market packs installed by two lines each.
- *Work:* a milestone test that packages `ac1_composability`, `market_composition` and `market_town`.
- *Assessment.* Cheap and honest about MVP-0's boundary, but it delivers nothing Milestone C and `AC-1`
  have not already delivered, and the operator listed E as a separate milestone after C. Every pack is
  still compiled from this repository's `systems/` and authored by this project; no pack has an
  identity, a version, a licence or a dependency range; Entity, Controller and Presentation Packs do
  not exist as packs at all (§2). "Install modules → compose world" is true only for an author who can
  edit this repository. **Not recommended.**

### E-B — "Declared, versioned packs from more than one source, composed inside the static boundary"

- *Real world:* a **new** World Pack — not a copy of an existing one — that runs 300 headless days and
  can be hosted and joined, and whose composition comes from **more than one source**: packs bundled
  with the framework, and at least one pack authored and kept **outside this repository**.
- *Independently installable:* every pack has an identity (id, version, type, licence, provenance) and
  declares what it depends on, with version ranges; a pack is installed by a **declared mechanism**
  rather than by editing framework code; the build refuses, by name, a composition whose dependencies
  are absent or out of range; and a `mineworld packs` command lists, validates and resolves packs.
- *Inside ARC-33's boundary:* a System Pack still enters the build by two lines in `systems/installed`
  and a rebuild (proven by the spike in §3 to work for a pack outside the repository). Tier-0 packs —
  World, Entity, Presentation — need **no rebuild**: they are data (`PACKAGE_FORMAT.md` §6, "most packs
  are Tier 0, and that is the point").
- Two scope levels, chosen by the operator (QSE-1):
  - **E-B1 (content from outside, code from inside).** Only Tier-0 packs come from outside the
    repository. `ARC-33` stands unchanged. Proves pack identity, dependencies and composition, but no
    *interaction system* is installed from outside, which is the noun the frozen criterion uses.
  - **E-B2 (also a System Pack from outside).** As E-B1, plus one third-party-style System Pack that
    lives in its own repository, is authored only against the framework's published crates, and is
    installed into the reference build by `ARC-33`'s two lines with a pinned git source. Revises
    `ARC-33`'s one sentence (QSE-2): "outside this repository" moves into MVP-0; "without a rebuild"
    and "untrusted code" stay in `ARC-8`.
- **Recommended: E-B2.** It is the smallest reading that makes `install modules → compose world →
  configure → run` true for someone who never edits MineWorld's source, which is the extension model's
  whole claim, and it costs one framework precursor (§4.4) beyond E-B1.

### E-C — "The publishing sense"

- *Real world:* any world assembled from downloaded `.mwpack` archives.
- *Independently installable:* `mineworld install <id>` fetches from an index, verifies, and installs
  into a running or built runtime without a compiler.
- *Assessment.* This is `PACKAGE_FORMAT.md` §§5–6 in full: `.mwpack`, a registry, Tier-1 WASM. It
  collides with three non-goals (WASM sandbox, Phase 3 registry, and — for code — `ARC-8`). **Not
  recommended for MVP-0.** §12 records what E-B2 leaves ready for it, so that E-C is an extension of
  S16's surfaces rather than a replacement.

## 1.4 The recommended reading, stated as a claim

```text
A new world, <world>, is assembled from:
  bundled System Packs      presence, movement, conversation, naming, schedule, item, inventory, …
  a third-party System Pack  kept in its own repository, pinned by git revision, never edited here
  an Entity Pack             a catalogue of item kinds, shared, not copied into the world
  a Presentation Pack        mineworld-default-2d / -3d, declared and validated
  the World Pack itself      people, places, sections, and a dependency list with version ranges

every pack has id, version, type, licence and provenance;
the composition is resolved and refused by name when a dependency is absent or out of range;
`mineworld packs` lists, validates and resolves it;
installing the third-party System Pack is ARC-33's two lines and a rebuild; the Tier-0 packs need none;
no change to kernel/, contracts/, persistence/, server/, a controller or a client.
```

---

# 2. Audit — implemented versus specified (`main @ 0fd0be3`)

## 2.1 What a "pack" is today, per pack type

| Pack type (`MODULE_SPEC.md` §1, `PACKAGE_FORMAT.md` §1) | What exists in source | Identity / manifest | Installed how |
| --- | --- | --- | --- |
| **System Pack** | 14 crates under `systems/*` implementing `mineworld_sdk::SystemPack` (`sdk/rust/src/pack.rs`) and `PerceptionProvider`; the installed set `systems/installed` (`installed!` in `sdk/rust/src/installed.rs`). | `SystemId` (e.g. `inventory`) and `SystemVersion(u32)`, a contract/migration counter (`kernel/src/system.rs:61–85`). Cargo package `mineworld-<name>`, version `0.0.0` **for every crate** (`[workspace.package]`), licence `MIT` inherited. No pack-level semver, no licence of its own, no provenance. | `ARC-33`: directory + two lines + rebuild. `mineworld install` / `add-system` unimplemented (`MODULE_SPEC.md` §8). |
| **World Pack** | `worlds/{social-cafe, market-town, bodies-yard}`; `worldpack` reads `world.yaml` (`worldpack/src/format.rs` `WorldManifest`: `world{id,name}`, `systems`, `places`, `population`, `items`, `organizations`, `seats`). | `world.id` = directory name (`check_pack_id`). No version, licence, dependencies or `mineworld` range. `entity_packs`, `presentation_profile`, `cognition_profile`, `network_profile` are **refused by name** (`MODULE_SPEC.md` §4.1). | Already location-independent: `mineworld run|server|validate <path>` reads any directory. No rebuild. |
| **Entity Pack** | **None.** `EntityType` is a closed set in `contracts` (Person, Place, Item, Organization). Item kinds are authored per world in `items/*.yaml` (`ARC-36`); Market Town's twenty kinds exist only inside Market Town. | — | `entity_packs:` refused. |
| **Controller Pack** | `cognition/rule-controller` (`RuleController`, `PacedRuleController`), a crate compiled into `tools/cli`, which is the composition root (`tools/cli/Cargo.toml`). It depends on five System Packs by name (conversation, movement, group-activity, naming, schedule). `ARC-34` complete affordances let it use packs it was never compiled against. | Cargo crate only. | Not selectable: `run` always uses the paced controller, `server --agent` the reactive one. `cognition_profile` refused. |
| **Presentation Pack** | `presentation/mineworld-default/{2D,3D}/` with `manifest.yaml` (the **style** manifest of `ART_DIRECTION.md` §12, e.g. `id: mineworld-default-3d-realistic`), `ART_DIRECTION.md`, `references/`, `LICENSES/`. | The style manifest's `id`. No version, licence field or dependency. | **No program reads it.** The 3D client cites the references in comments (`clients/3d-spike/scripts/slice/palette.gd:20`, `cafe.gd:3`, `npc.gd:88`); colours are transcribed by hand. A world cannot name it (`presentation_profile` refused). |
| **Asset Pack** | **None as a pack.** Assets live inside `clients/3d-spike/assets/` (`models/`, `characters/`, `hdri/`), catalogued in `clients/3d-spike/ASSETS.md`, with licence text files beside some. No `asset.yaml`, no `semantic_bindings`. | — | Copied into the client project. `mineworld validate asset` unimplemented. |
| **`.mwpack`** | None. | — | — |
| **Tier 1 (WASM/WIT)** | None (non-goal). | — | — |

## 2.2 What the specifications describe that is not implemented

| # | Specified | Where | State |
| --- | --- | --- | --- |
| G-1 | A manifest per pack: `id`, `version` (semver), `type`, `mineworld.requires` (framework range), `dependencies` (`id@range`), `license.spdx` | `PACKAGE_FORMAT.md` §5; `MODULE_SPEC.md` §9 | Absent for every pack type. |
| G-2 | Semver required, not optional | `PACKAGE_FORMAT.md` §5 | Every crate is `0.0.0`; `SystemVersion` is a counter whose relation to a release version is undefined. |
| G-3 | Dependencies with version ranges; `requires` (capabilities); `provides` | `MODULE_SPEC.md` §9 | System-to-system `depends_on` exists, by `SystemId` only, unversioned, checked by the kernel registry at install. Pack-to-pack: Cargo path dependencies between sibling packs. World-to-pack: `systems:` by id, unversioned. `dependencies.yaml` in the §4 tree: absent. |
| G-4 | SPDX licence required ("a pack whose licence cannot be resolved cannot be redistributed") | `PACKAGE_FORMAT.md` §5; `DEP-8` (asset licences) | Workspace-wide `MIT`. Presentation and asset licences are prose files. |
| G-5 | Provenance | `ARC-9` (generated assets); `Metadata::source_pack` | `source_pack` is free text, the world id, "until pack loading becomes a contract" (`contracts/src/entity.rs:164`, `worldpack/src/format.rs:79`). The save manifest's `pack` is informative (`persistence/src/format.rs:31`). |
| G-6 | `entity_packs`, `presentation_profile`, `cognition_profile` in `world.yaml` | `MODULE_SPEC.md` §4 | Refused by name (§4.1), deliberately. |
| G-7 | `mineworld install`, `add-system`, `validate asset` | `MODULE_SPEC.md` §8; `PACKAGE_FORMAT.md` §4 | Unimplemented; §8 says so. |
| G-8 | Configuration schema and migration schema per pack | `MODULE_SPEC.md` §9; `CORE_CONCEPTS.md` §13 | `SystemDeclaration` omits both deliberately (`kernel/src/system.rs:99–101`). A save whose composition's `SystemVersion` differs is refused (presence v3 refuses pre-12a saves). "Configure" in the extension model is today done by sections (`ARC-31`). |
| G-9 | `.mwpack`, Tier 1 | `PACKAGE_FORMAT.md` §§5–6 | Non-goal for MVP-0. |

## 2.3 Findings the audit adds

- **F-E1 — the manifest name collides.** `PACKAGE_FORMAT.md` §5 names the package manifest
  `manifest.yaml`; `MODULE_SPEC.md` §6.1 and `ART_DIRECTION.md` §12 already use `manifest.yaml` for a
  Presentation Pack's **style** manifest, and `presentation/mineworld-default/{2D,3D}/manifest.yaml`
  exist with that meaning. Two specifications give one file name two schemas (`CLAUDE.md` §2.1(4)).
  QSE-4.
- **F-E2 — a World Pack's identity is already stated, in `world.yaml`.** `world.id` is the pack's id
  and must equal its directory. A second manifest file for World Packs would state the id twice
  (`MODULE_SPEC.md` §4.1 rule 1: "a key is stated once"). §4.2 puts the World Pack's package fields
  in `world.yaml`.
- **F-E3 — a code pack's identity is already stated, in `Cargo.toml`.** Name, semver version, SPDX
  licence expression, authors and repository are Cargo package fields, and Cargo enforces semver
  ranges between crates at build time. A `pack.yaml` beside `Cargo.toml` would state each twice.
  §4.1 makes Cargo the carrier for code packs.
- **F-E4 — the SDK is not yet a sufficient authoring surface.** An external System Pack needs
  `mineworld-sdk`, `mineworld-kernel`, `mineworld-contracts` and `mineworld-presence` (for
  `PerceptionProvider`, `Offer`), and `mineworld-authoring` if it owns a section. `presence` is itself
  a System Pack; the SDK cannot re-export it without depending on a pack (`ARC-33` point 1 forbids that).
  §4.4 names the **published surface** explicitly instead of hiding it.
- **F-E5 — `ac1_composability` check 3 compares Social Café's and Market Town's files.** A new file in
  both worlds whose content differs (e.g. a `pack.yaml` with each world's id) would be reported as a
  world delta. Keeping World Pack package fields in `world.yaml` (F-E2), with identical values in both
  worlds except the id check 3 already admits, avoids editing the accepted `AC-1` proof. Verified
  against `tests/acceptance/tests/ac1_composability.rs:1058–1090` (`compare_manifests`): inside
  `world:` every field but `id` and `name` must be equal in both worlds, and every other top-level key
  must be equal. Equal `version`, `license` and `mineworld` values, and no `requires` entry for a
  bundled pack (§4.2), therefore pass check 3 unchanged.
- **F-E6 — nothing reads a Presentation Pack.** Making a world *use* one means a server disclosure and
  a client that applies it — S11, S12 and S14's ground, being planned in parallel. QSE-9.

---

# 3. Spike — can a System Pack from outside the repository enter the build by ARC-33's two lines?

Run on this worktree at `0fd0be3`, 2026-10-08, every edit reverted afterwards (`git status` clean). A
scratch pack `mineworld-hello` (one `System` with no claims, `impl SystemPack`, `impl
PerceptionProvider`) was created under `/tmp/mw-e-spike/hello-pack`, **outside the repository**, with
its own `Cargo.toml` (no `workspace = true` keys, version `0.1.0`).

| # | Pack source | Pack's own dependencies on MineWorld | Root change | Result |
| --- | --- | --- | --- | --- |
| S-1 | `path = "/tmp/.../hello-pack"` | `path` into the checkout | none | **PASS.** `cargo metadata`: 25 workspace members, the pack **not** a member; `cargo check -p mineworld-installed-systems` compiled it into the installed set (26 s cold). Diff: the two `ARC-33` lines + 11 `Cargo.lock` lines (a path package outside the workspace is locked with no `source`). |
| S-2 | `path` | `git = "https://example.invalid/…"` (a URL that cannot resolve) | `[patch."https://example.invalid/…"]` → the four framework crates by path | **PASS** for `cargo metadata`/`check` with the pack as a path dependency: one `mineworld-kernel` in the graph, so the pack's `System` is the build's `System`. |
| S-3 | `git = "file:///tmp/.../hello-pack", rev = …` | `git = "https://example.invalid/…"` | same `[patch]` | **FAIL**: Cargo tried to fetch `example.invalid` ("failed to get `mineworld-contracts` as a dependency of package `mineworld-hello` … git"). A patched **git** source is fetched when the depending package itself comes from git. Since `github.com/yuema137/MineWorld` is **private** (`gh repo view`: `PRIVATE`), this route needs credentials everywhere it builds. |
| S-4 | `git = "file:///…", rev = …` | **version requirements** `mineworld-sdk = "0.0.0"` etc. (crates.io source) | `[patch.crates-io]` → the four framework crates by path | **PASS.** Resolved and checked with no fetch of any MineWorld source; `Cargo.lock` records the pack with `source = "git+file:///…?rev=…#…"` and the framework crates as path packages. `cargo search mineworld` finds no crate of that name on crates.io today. |

**Conclusions carried into the design.**

1. A pack outside the repository compiles into the build through exactly `ARC-33`'s two lines. No
   other file changes when the pack names the framework by path (S-1).
2. A pack that should build **anywhere** (not only beside one checkout) names the framework crates by
   **version requirement**, and the reference build maps those names to its own paths **once**, with a
   root `[patch.crates-io]` section (S-4). That section is a one-time framework precursor, not a
   per-pack edit. Patching a git URL (S-2/S-3) works only when the URL is fetchable, which a private
   repository makes fragile; rejected.
3. S-4 carries a supply-chain risk: if the `[patch]` were ever absent, Cargo would resolve
   `mineworld-sdk = "0.1"` from crates.io, where anybody could publish that name. The guard is
   mechanical (§4.4: a lock check that no `mineworld-*` package has a registry `source`; `cargo-deny`'s
   `sources` ban, §5), and reserving the names on crates.io is an operator option (QSE-16).
4. A pack's own repository may carry its own `[patch.crates-io]` pointing at MineWorld by git for
   standalone builds: Cargo applies `[patch]` only at the root of the workspace being built, so that
   section is ignored when the reference build consumes the pack. To be confirmed at E-c's design.

---

# 4. Design of the recommended reading (E-B2)

## 4.1 Package identity: one vocabulary, carried where the identity is already stated

Every pack has the same **package fields**, the MVP-0 subset of `PACKAGE_FORMAT.md` §5 and
`MODULE_SPEC.md` §9:

```text
id            the pack's id: a lowercase key. A code pack's is its Cargo package name; a data pack's is
              its directory name (as world.id is today)
version       semver (MAJOR.MINOR.PATCH)
type          system-pack | controller-pack | world-pack | entity-pack | presentation-pack
              (asset-pack reserved: refused by name in MVP-0, QSE-11)
mineworld     the framework versions the pack works with: a semver range
dependencies  other packs, each with a semver range
license       an SPDX licence expression
provenance    authors, and the repository the pack comes from
```

Not in the MVP-0 subset, and refused by name where a data pack states them: `provides` (derived, not
stated — a System Pack's declaration already says what it provides), `requires` *capabilities*
(`MODULE_SPEC.md` §10, meaningful only with Tier 1), configuration schema and migration schema (G-8).

**The fields are carried where the pack already states its identity**, so no fact is stated twice
(F-E2, F-E3; `MODULE_SPEC.md` §4.1 rule 1):

| Pack type | Carrier | `mineworld` range | `dependencies` |
| --- | --- | --- | --- |
| System Pack, Controller Pack (code) | its `Cargo.toml` `[package]`: `name`, `version`, `license`, `authors`, `repository` | its Cargo requirement on `mineworld-sdk` — enforced by Cargo at build time | its Cargo dependencies on other packs — enforced by Cargo; the kernel's `depends_on` still governs installation into a world |
| World Pack | its `world.yaml` (§4.2) | `mineworld:` | `requires:` |
| Entity Pack, Presentation Pack | a `pack.yaml` at the pack's root (file name QSE-4) | `mineworld:` | `dependencies:` |

**How a built binary knows its code packs' identities.** `mineworld_sdk::package!()` expands, inside the
pack's own crate, to a `Package` value read from Cargo's compile-time environment
(`CARGO_PKG_NAME`, `CARGO_PKG_VERSION`, `CARGO_PKG_LICENSE`, `CARGO_PKG_AUTHORS`,
`CARGO_PKG_REPOSITORY`, `CARGO_MANIFEST_DIR`). `SystemPack` gains a **required** associated constant
`PACKAGE`, written `mineworld_sdk::package!();` inside `impl SystemPack` — one line per pack, and a pack
without it does not compile (the safe direction: no pack is silently anonymous). The installed set's
`Capability` gains `package()`. The binary never runs Cargo and never reads a source tree (§5 row 13).

**The framework version.** `[workspace.package] version` moves from `0.0.0` to `0.1.0` (QSE-6), so the
framework crates and every bundled pack share one release version, and `mineworld:` ranges have
something to be checked against: the `mineworld-sdk` version the running binary was built with.

**`SystemVersion` and the release version stay two things.** `SystemVersion` is the contract counter
saves are checked against (`kernel/src/system.rs:61–66`); the semver version is what a dependency range
is checked against. Rule proposed for `MODULE_SPEC.md` §9 (QSE-7): raising a pack's `SystemVersion` —
which makes existing saves refuse — is a breaking release (before 1.0, a MINOR bump; from 1.0, a MAJOR
bump), so a range like `^0.1` never admits a pack whose saves are incompatible.

**Bundled and third-party.** A pack is **bundled** when it is compiled from this repository's workspace;
its version is the framework version, and its compatibility is therefore exactly the world's
`mineworld:` range. Every other pack is **third-party**. A world names every third-party pack it uses
in `requires:` with a range; it need not name bundled packs. This keeps the three existing worlds'
`world.yaml` free of a dependency list they do not need, and is what lets F-E5 hold.

## 4.2 World Pack: identity, compatibility and requirements in `world.yaml`

```yaml
world:
  id: lakeside                 # unchanged rule: the directory's name
  name: Lakeside
  version: 0.1.0               # new, required once E-a lands (existing worlds gain it)
  license: CC0-1.0             # new, required
mineworld: "^0.1"              # new, required: the framework versions this world is authored for
requires:                      # new, optional: every third-party pack this world uses, with a range
  acme-fishing: "^0.1"         #   a System Pack (third-party)
  modern-goods: "^0.1"         #   an Entity Pack: requiring it puts its item kinds in this world
  mineworld-default-3d: "^0.1" #   a Presentation Pack: the world is authored for it (§4.5)
systems: [presence, movement, …, fishing]   # unchanged: what is enabled, in installation order
```

Rules (proposed for `MODULE_SPEC.md` §4.1, QSE-8):

1. **`requires` states versions; `systems` states enabling and order.** They answer different
   questions (`MODULE_SPEC.md` §4.1 rule 2: `systems` order is observable), so neither replaces the
   other. A system enabled in `systems` whose pack is third-party and absent from `requires` is
   refused, naming the system, its pack and the missing requirement.
2. **Requiring an Entity Pack is using it; requiring a Presentation Pack is declaring it.** The frozen
   model's separate `entity_packs:` and `presentation_profile:` lists are therefore not implemented;
   they stay refused by name, and `MODULE_SPEC.md` §4's model is amended to show `requires` (QSE-8,
   operator-material because it changes the frozen World Pack model).
3. **Everything is refused by name, never ignored**: a framework outside `mineworld:`; a required pack
   that is absent (listing where it was looked for); present at a version outside the range (naming
   both); of the wrong type; a bundled pack listed in `requires` (it is versioned with the framework);
   a missing `version` or `license`; a licence outside the policy (§4.7).
4. Resolution is **one installed version per pack**, checked, never chosen: there is no version
   selection, so no solver (§5 row 10).

Where it is checked: `WorldPack::read` already resolves `systems` against the installed set
(`worldpack/src/read.rs:311–328`); requirement resolution is the step after it. The code lives in a new
framework crate `packages/` (`mineworld-packages`): the package types, `pack.yaml`'s format, semver
parsing and range checks, licence policy and pack roots. It depends on `semver` and `serde-saphyr`,
on no pack and not on `worldpack`; `worldpack` and `tools/cli` depend on it. One crate owns "what a
package is", and the World Pack loader stays the owner of the world format.

**Where data packs are found: pack roots.** A Tier-0 pack is installed by placing its directory in a
**pack root**: a directory named by `--packs DIR` (repeatable) on `validate`, `run`, `server`,
`biography`, `replay` and `packs`, or by the `MINEWORLD_PACKS` environment variable (a path list).
There is no implicit search: a pack found by accident is a world its author did not author (QSE-13).
`WorldPack::read(root)` keeps its signature and resolves with no pack roots; `WorldPack::read_with(root,
&PackRoots)` is added, so no existing caller changes (`DEP-12` option (e)'s cost is avoided). The CLI's
three call sites (`tools/cli/src/main.rs:252, 294, 425`) pass the roots.

## 4.3 Entity Pack (MVP-0 subset: shared item kinds)

```text
modern-goods/
  pack.yaml                 id: modern-goods, type: entity-pack, version, mineworld, license, provenance
  items/
    bread.yaml              the World Pack item-file format, unchanged: tags, note, sections
    coffee.yaml             (e.g. `item: { category: food }`, owned by the `item` System Pack)
```

- **Scope.** `MODULE_SPEC.md` §2 lists entity type definitions, component schemas, item types, tags
  and authoring templates. MVP-0 implements **item types** only: new *entity types* would extend the
  closed `EntityType` in `contracts`, which is a contract change outside S16 (I-E1); component schemas
  belong to System Packs in MVP-0 (`ARC-31` sections). Recorded as a limitation, not hidden.
- **Composition.** A world that requires an Entity Pack has that pack's item kinds as if its own
  `items:` listed them. Keys stay one namespace (`MODULE_SPEC.md` §4.1 rule 1): a key in both the
  world and a required Entity Pack, or in two required Entity Packs, is refused naming both sources.
- **Identity.** Item kinds are allocated, as today, after people and in key order, across the world's
  own and every required Entity Pack's kinds together. No existing world requires an Entity Pack, so
  no existing identity or genesis fact moves (I-E2).
- **Sections still belong to their owners.** An entity-pack file's `item:` section requires the
  world to enable `item`; otherwise the existing rule 6 refusal names the pack and the section.
- **Provenance.** `Metadata::source_pack` of an entity-pack item names the Entity Pack, not the world —
  the first use of that field as a pack identity (G-5). It is free text already; no contract change.

## 4.4 A System Pack from outside the repository

**Precursor (framework, once).**

1. **The published surface.** The crates a third-party System Pack may depend on are named, in
   `MODULE_SPEC.md` §3.2 (new): `mineworld-sdk`, `mineworld-kernel`, `mineworld-contracts`,
   `mineworld-authoring`, and `mineworld-presence` (F-E4), plus any bundled pack deliberately
   **published** for others to depend on (for the fishing pack, `mineworld-inventory`, whose `produce`
   is how a producing pack states `items-produced`, `ARC-38`). Publishing a bundled pack is one line in
   the root `[patch.crates-io]`, a reviewed act; installing a bundled pack still edits nothing outside
   `systems/` (`ARC-33` unchanged).
2. **Root `[patch.crates-io]`** maps each published crate name to its path (spike S-4). A third-party
   pack names the framework by version requirement (`mineworld-sdk = "0.1"`) and therefore builds
   against whichever MineWorld its consumer has.
3. **A lock guard** (`tests/acceptance`): no package named `mineworld-*` in `Cargo.lock` has a registry
   source, so a missing or broken patch fails a test instead of silently compiling a stranger's crate
   of that name (§3 conclusion 3). `cargo-deny`'s `sources` check states the same policy in CI (§5
   row 11, with S13).
4. **The SDK authoring guide** — how to write, test and publish a System Pack outside MineWorld — is a
   specification section (`MODULE_SPEC.md` §3.2), linked from `sdk/rust/README.md`.

**The third-party pack** (QSE-3, QSE-12). Recommended: a public repository
`github.com/yuema137/mineworld-pack-fishing` (MIT), package `acme-fishing`, system id `fishing` — the
`acme-` prefix marks a third party and keeps `mineworld-*` for the framework. It is authored only against
the published surface, is not a workspace member, and has no path into this repository. Sketch, detailed
at E-c's PR design:

```text
section     fishing   places   { catch: <item key>, minutes: 30..240 }: this place has water to fish
provides    fish                to a Person in a fishing place who can carry one more (complete
                                affordance, ARC-34, so the unchanged paced controller attempts it)
owns        Fishing             who is fishing, until when
process     a catch             ends after `minutes`; states inventory's items-produced for the fisher
                                through mineworld_inventory::produce (ARC-26, ARC-38)
depends on  presence, inventory
disabled    fish is Unavailable and offered nowhere; nothing else changes (AC-2)
```

**Installed into the reference build by `ARC-33`'s two lines**, with a pinned source:

```text
systems/installed/Cargo.toml   mineworld-fishing = { package = "acme-fishing", git = "https://github.com/yuema137/mineworld-pack-fishing", rev = "<sha>" }
systems/installed/src/lib.rs   Fishing => acme_fishing::FishingSystem,
Cargo.lock                     regenerated: one git-sourced package
```

**What `ARC-33` then says (ARC-SE-a, QSE-2).** Its sentence placing "packs from outside this repository"
outside MVP-0 is superseded: a System Pack whose **source** is outside the repository but which is
**compiled into this build** from a pinned revision is in MVP-0, installed by the same two lines. It is
trusted code, reviewed like any dependency before the reference build installs it (`MODULE_SPEC.md`
§3.1, last paragraph). Installing without a rebuild, and running code nobody reviewed, remain `ARC-8`.

## 4.5 Presentation Pack (declared and validated; consumed by the clients' steps)

- `presentation/mineworld-default/2D/` and `3D/` each gain a `pack.yaml` (`id: mineworld-default-2d` /
  `-3d`, `type: presentation-pack`, version, `mineworld`, `license`, provenance). The style manifest
  (`manifest.yaml`) is unchanged; its own `id` stays the style's id (F-E1, QSE-4).
- `packs validate` checks a presentation pack's package fields and that its style manifest parses with
  an `id` and a `dimension`.
- A world `requires` the presentation packs it is authored for; resolution checks them like any pack.
- **Not in S16 (QSE-9):** disclosing the world's presentation packs to a client (a `server/PROTOCOL.md`
  welcome-frame field — S11's), and a client applying one (S12, S14). S16 records the hook in its
  amendment so the clients' steps can take it; S16 changes no server or client file.

## 4.6 Controller Pack (identity only)

`cognition/rule-controller` carries `PACKAGE` like a System Pack, and `packs list` shows it with the
System Packs it is compiled against (its Cargo dependencies — the vocabulary it knows by name, `ARC-34`
point 6). Selecting a controller per world or per seat (`cognition_profile`) is S10/MVP-1's (QSE-10).

## 4.7 Licence policy

A pack's `license` must be an SPDX expression whose every identifier is on the project's redistribution
allow-list. MVP-0's list is `DEP-8`'s approved licences for assets plus the permissive code licences the
workspace already uses (`MIT`, `Apache-2.0`); anything else is refused naming the identifier. For code
packs the same policy is enforced over the whole dependency graph by `cargo-deny` in CI (§5 row 11).
Parsing an expression uses the `spdx` crate (§5 row 12); the allow-list is MineWorld's policy, in one
file.

## 4.8 `mineworld packs`

```text
mineworld packs list     [--packs DIR]...        every pack this build and these roots provide:
                                                 id · version · type · licence · bundled | third-party ·
                                                 source (path or git revision)
mineworld packs show     <id> [--packs DIR]...   one pack: package fields, dependencies, and for a System
                                                 Pack its system id and SystemVersion
mineworld packs validate <dir>                   a data pack directory: package fields, licence, and its
                                                 own content (an Entity Pack's item files; a Presentation
                                                 Pack's style manifest; a World Pack's whole read)
mineworld packs resolve  <world> [--packs DIR]... the world's composition: every pack it uses, the version
                                                 that satisfies each requirement — or the first refusal
```

`mineworld validate <world>` additionally prints the resolved composition. Every refusal is a message on
stderr naming what was wrong and a non-zero exit (`MODULE_SPEC.md` §8.1). The subcommand is a new module
`tools/cli/src/packs.rs`; `main.rs` gains one variant and one arm (§8.4, parallel-safety with S11).
`mineworld install` stays unimplemented (QSE-11): installing a code pack is two reviewed lines, and a
command that edits Rust sources is a code generator nobody asked for.

## 4.9 What S16 does not change

`kernel/`, `contracts/`, `persistence/` (no save-format change; QSE-14), `server/`, `clients/`,
`cognition/rule-controller`'s behaviour (`ARC-34`'s band and constants stay frozen), and every existing
world's facts. The only framework crates edited are `sdk/rust`, `worldpack`, `tools/cli`, the new
`packages/`, the root `Cargo.toml` (the version, the `packages` member, `[patch.crates-io]`), and one line
in each bundled pack.

---

# 5. Reuse analysis — both directions (`REUSE_POLICY.md` §§11–12, §17; operator directive 2026-10-08)

The question is not "which package manager?" but four separable ones: **(a)** how code packs are
fetched, versioned and linked; **(b)** how the build registers what it linked; **(c)** what a manifest
says and in what format; **(d)** how ranges, licences and sources are checked. Fifteen candidates, each
judged on fit with `ARC-33`'s static boundary, licence, maturity and cost.

| # | Candidate | Question | Fit with the static boundary (`ARC-33`) | Licence · maturity | Cost | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | **Cargo** — path and git dependencies with `rev`, workspaces, `[patch]`, `Cargo.lock`, `package.metadata` | (a) | Exact. It is already how every pack is linked; the spike (§3) shows an out-of-tree pack needs nothing else. Semver ranges between crates are enforced at build. | MIT/Apache-2.0 · the Rust toolchain | ~0: in use | **ADOPT** as the package manager of code packs. |
| 2 | Cargo **features**, one per pack, in the installed set | (b) | Partial. Toggles linking, but duplicates the installed list with a second switch, and feature unification makes "which packs are in this binary" a function of the whole graph. Enabling is already the world's job (`systems:`). | — · mature | low | **REJECT**: a second statement of the installed set. |
| 3 | A Cargo **registry** — crates.io, or a private one (e.g. `kellnr`, a sparse index) | (a) | Fits technically; is a registry, which is the Phase 3 non-goal, and crates.io publication commits public names and APIs before a stable contract exists (`ENGINEERING_STANDARDS.md` §29). | various · mature | medium; operational | **REJECT for MVP-0**; the path to E-C. `[patch.crates-io]` (row 1) makes the move to a registry a deletion, not a redesign. |
| 4 | **Bevy** plugins — `Plugin` trait + `App::add_plugins`; plugins are ordinary Cargo dependencies | (b) | Same shape as `SystemPack` + `installed!`: a trait implemented in the plugin's crate, registered by the composing binary. Bevy deprecated `bevy_dynamic_plugin` in 0.14 and removed it in 0.15 as unsound (no stable ABI, `TypeId` unstable across compilation units). | MIT/Apache-2.0 · mature | — | **REFERENCE**: independent confirmation of `DEP-12`'s choice and of rejecting dynamic loading. Nothing to adopt; adopting Bevy itself is framework lock-in (`REUSE_POLICY.md` §3). |
| 5 | **Nushell** plugins — separate executables (`nu_plugin_*`) speaking a serialized protocol over stdio, registered with `plugin add` | (a)(b) | Poor for System Packs: every `react`/`resolve` would cross a process boundary, `INV-7`'s write gate would have to be re-enforced across it, and determinism would depend on IPC. Good for things already out of process. | MIT · mature | high | **REJECT for System Packs; REFERENCE for out-of-process Controller Packs** (S10's Python cognition already plans a process boundary). |
| 6 | **Zed** extensions — WASM components, `extension.toml` manifest, a registry repository holding extensions as git submodules | (a)(b)(c) | This is `ARC-8`'s Tier 1 (WASM + WIT + manifest): outside MVP-0. The registry-as-a-git-repository is a cheap Phase 3 model. | Apache-2.0 (API crate) · production | high | **REFERENCE for E-C** (§12). REJECT for MVP-0. |
| 7 | **`inventory` / `linkme`** — linker-section registration (re-checked from `DEP-12`) | (b) | Unchanged by out-of-tree packs: an out-of-tree pack still needs its Cargo line *and* a `use pack as _;` line, so nothing is saved; it still forces type-erased section decoding. | MIT/Apache-2.0 · mature | medium | **REJECT, reaffirmed**: `DEP-12`'s reasons all still hold. |
| 8 | **`libloading` / `abi_stable`** — native dynamic loading | (b) | Contradicts the boundary: `unsafe`, unstable ABI, native code from strangers. Bevy's removal (row 4) is the field's own verdict. | MIT/Apache · mature | high | **REJECT, reaffirmed** (`DEP-12` (c), `ARC-8`). |
| 9 | Game-mod manifests — Fabric `fabric.mod.json` (id, semver version, `depends` with ranges, licence, authors, contact), Factorio `info.json` (name, version, engine version, dependencies with optional `?` / incompatible `!` markers) | (c) | Formats, not mechanisms: their **field vocabulary** is what `PACKAGE_FORMAT.md` §5 already chose (id, semver, engine range, ranged dependencies, licence). Factorio's optional/incompatible markers have no MVP-0 need. | various · mature | — | **ADAPT the vocabulary** (already done by §5); adopt no file format: a JSON mod file is bound to its loader. |
| 10 | Godot **`plugin.cfg`** (INI: name, description, author, version, script) | (c) | Engine-specific, no dependencies or licence. `PACKAGE_FORMAT.md`'s governing rule: "a MineWorld package is not a Godot mod". | MIT · mature | — | **REJECT.** |
| 11 | **`cargo-deny`** — licence, source and ban policy over a Cargo graph | (d) | Exact for code packs: licence allow-list over every crate, `sources` restricted to crates.io + allowed git hosts, bans on duplicates. Runs in CI, not in the product. | MIT/Apache-2.0 · Embark, widely used | low (a `deny.toml`) | **ADOPT as a CI tool** (`DEP-SE-b`, landed with S13's CI layers or by E-c if S13 is later). |
| 12 | **`spdx`** crate (the parser `cargo-deny` uses) vs a closed list of exact identifiers | (d) | Cargo licence fields are expressions (`MIT OR Apache-2.0`); a closed list of strings cannot read them correctly, and a hand-written expression parser is the wheel this crate is. | MIT/Apache-2.0 · mature | low | **ADOPT `spdx`** for Tier-0 `license` fields (`DEP-SE-a`, with row 13); the allow-list is MineWorld policy. Prototype first (E-b C0): confirm the dependency weight is proportionate. |
| 13 | **`semver`** crate (dtolnay; Cargo's own semantics) vs `node-semver` vs our own | (d) | `semver` gives Cargo's meaning of `^0.1`, so a data pack's range means what a code pack's Cargo range means. `node-semver` gives npm's, which differs for pre-1.0 carets. Our own would be a third dialect. | MIT/Apache-2.0 · ubiquitous | negligible | **ADOPT `semver`** (`DEP-SE-a`). REJECT `node-semver` (semantic mismatch) and our own. |
| 14 | Dependency **solvers** — `pubgrub`, `resolvo` | (d) | No selection problem exists: one installed version per pack, checked against ranges (§4.2 rule 4). | MPL-2.0 / BSD-3 · mature | medium | **REJECT**: dependency larger than the problem. Revisit with a registry (E-C). |
| 15 | **`cargo_metadata`** at runtime, git submodules / subtree for third-party packs, and building our own manifest | (a)(c) | `cargo_metadata` at runtime needs a source tree and Cargo on the user's machine: rejected for the product (compile-time `env!` gives the same facts, §4.1); acceptable in tests. Submodules put the third-party source inside this tree, defeating "outside", for nothing `git` + `rev` does not already pin. **Our own manifest (`pack.yaml`)** is needed only where no carrier exists — Tier-0 Entity and Presentation Packs — because no existing format describes engine-neutral content packs with MineWorld's types, and `PACKAGE_FORMAT.md` §5 already specifies its fields. | — | low | `cargo_metadata`: **REJECT at runtime**. Submodules: **REJECT**. Own manifest: **BUILD, narrowly** — two pack types, a vocabulary borrowed from rows 1 and 9. |

**Recommendation and its reason.** Cargo is the package manager for every pack that is code, including
third-party ones (row 1); MineWorld adds only what Cargo cannot know: the build's registration of what
it linked (`installed!`, unchanged), a manifest for packs that are data (row 15), and resolution of a
world's requirements against both — on `semver` (13) and `spdx` (12), with `cargo-deny` (11) holding the
code graph's licences and sources in CI. Every rejected option is rejected for a reason in
`REUSE_POLICY.md` §12's list: a second statement of one fact (2), the Phase 3 non-goal (3), architecture
mismatch (5, 6, 7, 8, 10), dependency larger than the problem (14), or defeating the requirement (15).
The two failure modes of §17 both checked: not reinventing (we build no fetcher, linker, resolver,
version or licence parser), not forcing (no registry, solver, process boundary or WASM runtime where the
static boundary has no use for one).

---

# 6. Modularity and pluggability — what "independently installable" means exactly

## 6.1 The definition

A pack is **independently installable** when all four hold:

1. **It is authored without editing anything it does not own.** Its source names only the published
   surface (§4.4) and the packs it declares as dependencies, with ranges.
2. **Installing it is declared, in one place per kind, and edits nothing else.** A code pack: the two
   `ARC-33` lines (three for an arrival resolver) and a rebuild. A data pack: a directory in a pack root,
   no rebuild.
3. **Using it is the world's choice, stated in the world.** `systems:` enables a System Pack;
   `requires:` names a third-party pack's range and puts an Entity or Presentation Pack in the
   composition. A world that does not name an installed pack is unaffected by it.
4. **Removing it is the reverse edit, and the result is either a world that still runs or a refusal
   that names it.** Never a world that silently runs differently.

## 6.2 What changes when you add or remove each thing

| Act | Files that change | Rebuild | Untouched |
| --- | --- | --- | --- |
| Install a bundled System Pack | `systems/<name>/` (new), 2 lines in `systems/installed/`, `Cargo.lock` | yes | everything else (`ARC-33`, unchanged) |
| **Install a third-party System Pack** | 2 lines in `systems/installed/` (git + `rev`), `Cargo.lock` | yes | no directory in this repository; root, `worldpack`, CLI, server, kernel, controllers |
| Upgrade a third-party System Pack | the `rev` on its line, `Cargo.lock` | yes | worlds whose range still admits it; a world whose range does not is refused naming both versions |
| Uninstall a System Pack | the same lines, removed | yes | worlds not enabling it run unchanged; a world enabling it is refused (`UnknownSystem`, listing what the build provides) |
| Enable / disable a System Pack in a world | one line of `systems:` (+ its sections; + a `requires` entry if third-party) | no | every other world; the pack's actions become `Unavailable` (`INV-10`, `AC-2`) |
| **Install an Entity or Presentation Pack** | a directory in a pack root | **no** | everything |
| Use an Entity Pack in a world | one `requires:` line | no | other worlds; identities of a world that does not use it |
| Add or move a World Pack | a directory anywhere | **no** | everything (already true today) |
| Publish a bundled pack for third parties | one line in the root `[patch.crates-io]` | yes | — (a deliberate, reviewed act, §4.4) |

## 6.3 What is still not pluggable in MVP-0, stated rather than implied

- **A new pack type, a new World Pack field or content kind, a new entity type** — framework or contract
  changes, by design (`MODULE_SPEC.md` §1; the closed `EntityType`).
- **Controllers** are composed in `tools/cli`; selecting one per world or seat is S10/MVP-1 (QSE-10).
- **Presentation** is declared and validated, not yet applied by a client (QSE-9).
- **Asset Packs** do not exist as packs; assets live in the 3D client (QSE-11).
- **Installing without a rebuild** of a code pack, and **untrusted code**, are `ARC-8` (Tier 1).
- **Per-world configuration of a System Pack** (configuration schema, G-8): "configure" is sections.

## 6.4 How the code stays modular

- One crate per responsibility: `packages/` owns what a package is; `worldpack` owns the world format;
  `sdk/rust` owns what a pack says about itself; `systems/installed` owns the list; the CLI only
  composes. No crate gains a second responsibility, and none learns a pack by name.
- Dependency direction stays one way: `tools/cli → worldpack → packages`, `worldpack → systems/installed
  → packs → sdk → kernel → contracts`; `packages` depends on no MineWorld crate except, if needed,
  `contracts` for `EntityKey`.
- Each new mechanism has a planted-violation test that bites (`ARC-23`): a pack without `package!()`
  does not compile; a framework crate from a registry fails the lock guard; a range that excludes the
  installed version, a missing pack, a disallowed licence, a key collision between a world and an Entity
  Pack — each refused, by name, in a test.

