## ARC-66 — A System Pack from outside this repository is installed by the same two lines, pinned to a commit

**Date** 2026-10-08 · **Approved by** the operator (S16 QSE-2, QSE-3, QSE-12, QSE-16; FQ-c1, FQ-c2,
FQ-c5) and the primary session at PR E-c's design freeze (step-16 §16.0) · **Supersedes in part**
`ARC-33`'s sentence placing "packs from outside this repository" outside MVP-0 · **Implements**
[`MODULE_SPEC.md`](MODULE_SPEC.md) §3.1, §3.2; [`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md) §8 · **Relates
to** `ARC-8`, `ARC-26`, `ARC-33`, `ARC-35`, `ARC-38`, `ARC-53`, `ARC-54`, `ARC-55`, `DEP-12`, `DEP-22`,
`DEP-23` · **Design** `.structured-coding/plans/mvp0/step-16-packages.md` §3, §4.4, §16 (S16, PR E-c)

**Problem.** The frozen top-level criterion asks for *independently installable* interaction systems.
Until this decision every System Pack in the build was compiled from this repository, and `ARC-33`
placed a pack from anywhere else outside MVP-0 together with the WASM tier. A framework whose only packs
live in its own tree has not shown that somebody else can write one.

**Choice.**

1. **Same two lines, pinned.** A System Pack whose source lives in another repository is installed by
   `ARC-33`'s two lines in `systems/installed/`, the Cargo line naming a git source and a **full 40-hex
   commit id**, and the lock records the same commit:

   ```text
   systems/installed/Cargo.toml   <package> = { git = "<url>", rev = "<40 hex>" }   (one line)
   systems/installed/src/lib.rs   <Variant> => <crate>::<System>,                    (one line)
   Cargo.lock                     source = "git+<url>?rev=<sha>#<sha>"              (generated)
   ```

   No `branch`, no `tag`: a tag can be moved, a commit cannot. The dependency key is the pack's own
   package name, never a rename (`<package> = { package = "…" }`): the installed set's consistency
   guard compares the manifest's keys with the crates `core::any::type_name` reports, and that is the
   real crate name (spike SC-2 observed a rename fail it).
2. **The pack names the framework by version, and the build maps the names once.** A third-party pack
   depends on MineWorld's crates by version requirement (`mineworld-sdk = "0.1"`), so it builds against
   whichever MineWorld its consumer has. The reference build maps those names to its own paths in one
   `[patch.crates-io]` table — the **published surface** — and nothing else: `mineworld-sdk`,
   `mineworld-kernel`, `mineworld-contracts`, `mineworld-authoring`, `mineworld-presence`,
   `mineworld-inventory`. A pack may depend on these and on registry crates; a pack's dev-dependencies
   are not part of the surface (Cargo never resolves a dependency's dev-dependencies). Publishing one
   more crate to third parties is one line in that table, a reviewed act.
3. **Where the table lives: `.cargo/config.toml` at the repository root**, a supported Cargo location
   for `[patch]` (FQ-c2 (a)). Paths in it are relative and written with `/`; Cargo resolves them against
   the directory that holds `.cargo/` on every operating system. Finding F-Ec1 decided the location:
   `ARC-35`'s check 2, bullet 3 scans every `*.rs` and every `Cargo.toml` — the root one explicitly —
   outside `systems/`, `worlds/` and `tests/acceptance/` for the market's crate names, and
   `mineworld-inventory` is one of them (a producing pack states `items-produced` only through
   `mineworld_inventory::produce`, `ARC-26`, `ARC-38`). The table in the root `Cargo.toml` would fail
   `AC-1`. In `.cargo/config.toml` it is build configuration, which the scan does not read — and this
   record does not lean on that silently: the published-surface table names one market crate outside
   `systems/`, which is consistent with `ARC-35`'s claim, because installing the market still edited only
   `systems/`, and publishing a crate to third parties is a separate, reviewed act after it. The
   acceptance test `package_sources.rs` reads the table, so it is guarded rather than invisible.
   `ac1_composability.rs` and `ARC-35` are not edited.
4. **Third-party is decided by where the pack was compiled** (`ARC-54` point 2). A git checkout lies
   under `CARGO_HOME`, outside the framework workspace, so `package!()` records it third-party; a world
   that enables its system must `require` the pack, in range, and its licence is judged by `ARC-55`.
   `mineworld packs list` says `bundled` or `third-party` for every code pack. The binary does not print
   the revision: Cargo exposes no variable for a dependency's source, and reading `Cargo.lock` at run
   time is `cargo_metadata` at run time, rejected in step-16 §5 row 15. The revision is stated once, on
   the pack's line in `systems/installed/Cargo.toml`, and `package_sources.rs` proves `Cargo.lock` holds
   the same one.
5. **Guarded by one acceptance test file**, `tests/acceptance/tests/package_sources.rs`, which names no
   pack:
   - over `Cargo.lock`: every package without a source is a workspace member, every registry package
     has a checksum, every git package's source is `git+<url>?rev=<40 hex>#<the same 40 hex>`; no
     `mineworld-*` package comes from a registry; each git package's revision equals the one its line
     in `systems/installed/Cargo.toml` states; only the installed set depends on a git package;
   - over `cargo metadata --locked --offline`: the git packages are exactly the installed capabilities
     whose `bundled()` is false; each is not a workspace member, its manifest is not under the workspace
     root, every normal dependency is in the published-surface table or a registry crate, no dependency
     has a `path`, and its manifest says no `workspace = true`;
   - over the tracked files: no `*.rs` and no `Cargo.toml` other than `systems/installed/Cargo.toml`
     spells a git package's crate name; `systems/installed/src/lib.rs` may name it once.
6. **Reproducible offline after one fetch** (§16 PD-26). `cargo fetch --locked`, then `--offline
   --frozen`; or `cargo vendor --locked <directory outside the tree>` and an empty `CARGO_HOME` with the
   generated `--config`. The vendor directory is never committed. A committed test does not vendor; the
   lock guard proves the precondition (every non-workspace package content-addressed).
7. **Trusted, reviewed code.** A pinned pack is compiled into the binary and runs in process with the
   kernel's authority, like every MVP-0 pack (`ARCHITECTURE.md` §12). It is reviewed like any
   dependency before the reference build installs it, and recorded as one (`DEP-23` is the first).
   Installing without a rebuild, and running code nobody reviewed, remain `ARC-8`'s Tier 1.
8. **Upkeep** (FQ-c7). A framework PR that breaks a pinned pack updates the pack in its repository and the
   `rev` here, in the same review. A breaking change to a published crate is a MINOR release before 1.0
   (`ARC-53`'s rule, extended to the published surface).

**Options considered** (`REUSE_POLICY.md` §§2, 11–12, 17 — both directions; step-16 §16.4):

```text
M1 a Cargo git dependency with `rev`, locked by Cargo.lock          ADOPT
M2 vendoring into this repository (copy, in-tree `cargo vendor`, subtree)
                                                                      REJECT: inside the tree it is
                                                                      bundled (ARC-54's limitation) and,
                                                                      under systems/, a workspace member
M3 a git submodule + a path dependency                               REJECT: bundled; the lock loses
                                                                      the pin; four places, not two
M4 `cargo vendor` outside the tree                                   ADOPT as the offline route, never
                                                                      committed
M5 a registry (crates.io, a private one, a static index)             REJECT for MVP-0: the Phase 3
                                                                      non-goal; publishes names before a
                                                                      stable contract. The patch table
                                                                      makes the move a deletion
M6 a hermetic build system (Nix crane/naersk, Bazel rules_rust)      REJECT: a second build description,
                                                                      framework lock-in
M7 our own fetcher (tarball + sha256 into vendor/)                   REJECT: rebuilds M1 + M4 badly
```

Not reinventing: Cargo fetches, pins, locks, vendors and enforces the framework requirement (a pack
requiring `mineworld-sdk = "0.2"` is refused by Cargo). MineWorld adds only the published-surface table
and a guard over the lock. Not forcing: no registry, second build system or fetch script where a git
`rev` already does the job.

**How it fails, closed.**
- The table removed (spike SC-5): `cargo metadata` refuses — "no matching package named
  `mineworld-contracts` found, location searched: crates.io index". A crate of that name published by a
  stranger would change that, which is why the lock guard refuses any `mineworld-*` package with a
  registry source and `cargo-deny`'s `sources` check allows only crates.io and the pack's repository
  (`DEP-22`).
- Cargo run from outside the repository (`--manifest-path <repo>/Cargo.toml` from another directory)
  does not read `.cargo/config.toml` and refuses the same way (spike SC-6). Run Cargo in the repository.
- The pack's repository or commit gone: the fetch fails, naming the source; an existing vendor
  directory still builds it.

**Accepted limitations.**
- `.cargo/config.toml` is read from the working directory upward, so the published surface depends on
  where Cargo runs (above).
- The CI cache key (`Cargo.lock`, toolchain, Dockerfile) does not include `.cargo/config.toml`; a change
  to the table that changes resolution changes `Cargo.lock` too.
- A third-party pack takes no world configuration in this first instance (FQ-c9); the seam
  (`ARC-61`) is the same for it, and its third-party reach is S17's to demonstrate.
- Revisit together with `ARC-33` and `DEP-12` when a registry or `ARC-8`'s Tier 1 arrives.
