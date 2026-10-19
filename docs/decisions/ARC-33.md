## ARC-33 — A System Pack is installed by declaring it in the build's installed set

**Date** 2026-10-07 · **Implements** [`MODULE_SPEC.md`](MODULE_SPEC.md) §3.1 · **Relates to** `ARC-8`,
`ARC-23`, `ARC-26`, `ARC-29`, `ARC-31`, `DEP-10`, `DEP-12`, `ARC-35`, [`MVP.md`](MVP.md) §9 `AC-1`
· **Design** `.structured-coding/plans/mvp0/step-10-market.md` §2.2, SD-1 … SD-5 (S9, PR 11a)

**Problem.** The frozen top-level criterion asks for *independently installable* interaction
systems. Until this decision, installing one System Pack edited about eleven lines in five files,
three of them outside `systems/` (finding F-1, step-09 §8.2):

```text
Cargo.toml (root)          a `members` line and a `[workspace.dependencies]` line
worldpack/Cargo.toml       a dependency line
worldpack/src/catalog.rs   an enum variant, an AVAILABLE entry, and one arm in each of six matches
Cargo.lock                 regenerated
```

`ARC-31`'s accepted limitations already said so: "The catalog is still a closed list compiled into
the build. The seam removes the format edit, not the registration." Every one of those edits teaches
the World Pack loader that a pack exists, which is the change amplification `CLAUDE.md` §4 rule 5
forbids.

**What cannot be removed.** MVP-0's System Packs are trusted, statically linked Rust
(`ARCHITECTURE.md` §12, `PACKAGE_FORMAT.md` §8). A crate is in a statically linked Rust binary only
if some crate in the build declares it as a Cargo dependency, and no build script, macro or linker
trick links a crate nobody declares. So one declarative line naming the pack must exist in some
manifest. The decision is only *where* that line lives and what else must change with it.

**Choice.**

1. **A System Pack declares itself.** The SDK crate `mineworld-sdk` (`sdk/rust/`) defines
   `SystemPack: System + Default`. A pack implements it once, in its own crate, and says there
   everything the build needs to know about it beyond `System`:
   - `BIOGRAPHICAL`: which of its event types belong in a biography (`ARC-29`; default none);
   - `SECTION`: the authored section it owns, if any (`ARC-31`; default none);
   - `decode_section`: how that section is decoded. A section owner writes
     `mineworld_sdk::owns_section!();` inside its `impl`, which defines `SECTION` and
     `decode_section` together from its `AuthoredSection` impl, so the two cannot disagree. The
     default refuses, naming the pack: "the '<id>' system owns no section".

   The SDK depends on `authoring`, `contracts`, `kernel` and `serde`, and never on a pack, so every
   pack can implement it without a dependency cycle.
2. **The installed set is a crate whose only content is the list.** `systems/installed/` (crate
   `mineworld-installed-systems`) depends on the SDK, on `presence` for the perception trait, and on
   every installed pack. Its `lib.rs` is one invocation of `mineworld_sdk::installed!`, one line per
   pack. The macro expands to the closed enum `Capability`, the constant `AVAILABLE` in the listed
   order, and the methods `resolve`, `id`, `section`, `owning_section`, `decode_section`,
   `biographical`, `install`, `provider` and `Display` — the same closed enum and matches the
   catalog held, generated. Every code path therefore stays monomorphic, and a section is still
   decoded straight from the YAML stream with its line and column (`DEP-10`). A listed pack that is
   not a `PerceptionProvider`, not `Default`, or not a `SystemPack` does not compile into the set. A
   test holds the list and the crate's manifest equal, and no two listed packs may share an id.
3. **The root manifest stops registering.** Its `members` names `"systems/*"` instead of one line
   per pack. A new pack depends on a sibling pack by `path = "../<name>"`, so the root
   `[workspace.dependencies]` never learns it. Path dependencies between sibling packs add no
   external dependency, so the root's rule — a crate that needs a dependency not listed there is
   adding one, which is a reviewable decision — is unchanged.
4. **`worldpack` names only the packs its format fields belong to.** A person's `location` is
   `presence`'s and a place's `passages` are `movement`'s (`ARC-31` item 5), so `worldpack` keeps
   those two dependencies and gains the SDK and the installed set. It drops every other pack, and
   re-exports `Capability`, `AVAILABLE` and `SectionOwner`, so its public API does not change. A
   structural test holds its dependency allow-list, which names infrastructure and the two format
   owners and never another pack.

**What installing a System Pack means in MVP-0 — the static-linking boundary.** This is the whole of
it, and a reader must not take "independently installable" as more:

```text
systems/<name>/                        the pack                                   (a new directory)
systems/installed/Cargo.toml           mineworld-<name> = { path = "../<name>" }  (one line)
systems/installed/src/lib.rs           <Variant> => mineworld_<name>::<System>,   (one line)
Cargo.lock                             regenerated by Cargo                       (generated)
then                                   rebuild the binary
```

- No other file is edited: not the root manifest, not `worldpack`, not the CLI, not the server, not
  a controller, not the kernel.
- **Installing** puts a pack into the build. **Enabling** it is a world's choice: a World Pack's
  `systems:` list names it. A world that does not enable an installed pack is not affected by it.
- **Installing without a rebuild is not MVP-0.** Adding a pack to a binary that is already built or
  to a server that is already running, or installing a pack that is not compiled from this
  repository's build, is the WASM component model of `ARC-8` (Tier 1). That is outside MVP-0
  (`overall.md` §1 non-goals). So is Milestone E's publishing sense of "a real world assembled from
  independently installable packs": `.mwpack`, a registry, and packs from outside this repository.
  *(Superseded in part by `ARC-66`, 2026-10-08: a System Pack whose **source** is outside this
  repository but which is **compiled into this build** from a pinned commit is in MVP-0, installed by
  the same two lines. `.mwpack`, a registry, installing without a rebuild and unreviewed code remain
  outside it.)*
- `mineworld install` and `mineworld add-system` (`MODULE_SPEC.md` §8) remain unimplemented. The
  two lines are written by hand.

**Why `systems/installed` lives under `systems/`.** The installed set is the list of System Packs in
this build. Placing it beside the packs makes installing one an edit under `systems/` only, which is
what `ARC-35` measures. The operator approved this placement at S9's freeze (step-10 QS-3).

**Accepted limitations.**
- A rebuild is required to install or remove a pack, as above.
- Cargo still needs one dependency line per pack, and `Cargo.lock` changes with every install.
  `ARC-35` admits `Cargo.lock` only as a generated file under a rule that it gains path packages
  under `systems/` and nothing else.
- A declarative macro generates the catalog, which is harder to read than a hand-written enum. The
  macro is one file, documented method by method, and every existing loader, CLI, persistence and
  server test runs through its expansion.
- Revisit this decision together with `DEP-12` when the first build installs packs it does not
  compile from this repository: that is `ARC-8`'s Tier 1. The question each catalog method answers
  stays the same; the catalog becomes a registry populated at startup.

**Note, 2026-10-13 (S24, PR DL-a; `pr-dl-a-decision-list.md` SD-DLa-12; `ARC-85`).** The same pattern is
reused for decision models. `cognition/installed` (`mineworld-installed-models`) is the build's list of
models, written with `mineworld-decision`'s `models!` macro: one `Name => crate::Type` line per
in-process `DecisionModel`, and one `external "<id>"` line per out-of-process model, which has no Rust
type. The macro expands to a monomorphic catalog enum, as `installed!` does (`DEP-12`); `decisions.yaml`
model ids resolve through it, and `mineworld packs list` lists its in-process models' packages.
Installing a model is one `Cargo.toml` line and one `models!` line, then a rebuild. Its guard test holds
the list and the manifest's dependencies equal and refuses a duplicate id, as this crate's does. It lives
under `cognition/`, so installing a model is an edit there and nowhere else.
