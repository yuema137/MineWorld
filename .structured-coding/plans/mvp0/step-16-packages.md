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
  worlds except the id check 3 already admits, avoids editing the accepted `AC-1` proof. To be
  verified against `tests/acceptance/tests/ac1_composability.rs` `compare_manifests` at E-a's design.
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

