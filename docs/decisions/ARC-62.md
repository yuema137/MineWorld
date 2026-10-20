## ARC-62 — Extension catalogs: a pack-owned trait, implemented by other packs, listed in the installed set

**Date** 2026-10-08 · **Approved by** the operator (QPL-10: presence's `resolution:` line migrates to the
generic form, with no shim) and the primary session at PR IL-a's design freeze (step-18-interaction-list
§11) · **Implements** [`MODULE_SPEC.md`](MODULE_SPEC.md) §3.1 · **Relates to** `ARC-33`, `ARC-39`,
`DEP-12`, `ARC-61` · **Design** `.structured-coding/plans/mvp0/step-18-interaction-list.md` §11;
`step-18-physics-list.md` §4.7 (S17, PR IL-a)

**Problem.** `ARC-39` let other packs plug code into presence through `ArrivalResolver`, and spelled the
build's list as a hard-wired `resolution:` line of `installed!`, expanded into `Capability::resolvers()`
and registered by a call to `mineworld_presence::register_resolvers` that `worldpack::compose` makes by
name. The S17 Interaction List needs a second such catalog (`bodies`' interaction kinds, IL-i), and any
pack may need one later. A second hard-wired line would edit the SDK and the loader again for every
catalog — the change amplification `CLAUDE.md` §4 rule 5 forbids. Two real catalogs earn the
abstraction (rule 11).

**Choice.**

1. **One generic line per catalog.** `installed!`'s grammar is `perception: <path>;`, then zero or
   more lines

   ```text
   extension <trait path> => <register fn path>: [ <type>, … ];
   ```

   then the pack lines. Each listed type must implement the trait and `Default`; one that does not is
   refused by the compiler, at the list.
2. **What it expands to.** `Capability::register_extensions()` calls each line's register function once,
   with one value of each listed type, `Box<dyn Trait>`, lines in listed order and types in listed
   order. `Capability::extension_types()` returns each line's trait path and its types' Rust paths, for
   the installed set's own guard. The `resolution:` arm and `Capability::resolvers()` are deleted, with
   no shim (`CLAUDE.md` §4 rule 12).
3. **The loader calls one function.** `worldpack::compose` calls `Capability::register_extensions()`;
   neither the SDK nor the loader names a trait or a pack again. A third catalog is a line in
   `systems/installed` and nothing else.
4. **The catalog stays its owner's.** The register function, its storage and its rules belong to the
   pack that owns the trait. `ARC-39` item 5's rules carry over to every catalog: write-once and
   process-wide, registered before any world is assembled, a different list for one trait refused
   naming both; an implementation is pure (reads only what it is handed), keeps nothing, and is inert
   where its pack's state is absent, so a world that does not enable the implementing pack is
   unaffected.
5. **The installed set guards its lines.** A test refuses a type on an extension line that is not an
   installed pack, and a type listed twice on one line, naming each.
6. **Presence's catalog is the first line**, byte-identical in behaviour:
   `extension mineworld_presence::ArrivalResolver => mineworld_presence::register_resolvers:
   [mineworld_bodies::BodiesSystem,];`. Installing a pack that implements a catalog's trait is `ARC-33`'s
   two lines plus one entry on that catalog's line.

**Accepted limitations.** A catalog is per process, not per world (`ARC-39`'s limitation, unchanged):
per-world applicability comes from the world enabling the implementing pack, or from its configuration
(`ARC-61`). Disabling a pack at run time while its implementation is registered is not supported
(QB-17).

**Note, 2026-10-13 (S24, PR FG-a; `ARC-86`, QFGA-3) — a third line, selected by configuration, and the
loader registers earlier.** `relationships` owns the trait `DecayCurve` and its register function
`register_curves`; `systems/installed` carries
`extension mineworld_relationships::DecayCurve => mineworld_relationships::register_curves: [];`, empty
today. Unlike the first two lines, a curve is not applied wherever its pack's state is present: a world
**selects** it by name in `configure/relationships.yaml`, and must enable the curve's pack in `systems:`
(the section requires it, refused by name otherwise), so the curve's identity and version are part of the
save's composition. A name colliding with a bundled curve (`none`, `linear`, `half-life`, `power`) or
listed twice panics naming both, as item 4's rules. Because a curve name is checked while the section
decodes, `WorldPack::read_with` now calls `Capability::register_extensions()` before it decodes
configuration, as `compose` already did (idempotent for the same list).

**Note, 2026-10-19 (S34-b; `ARC-147` note of the same date) — a fifth line, owned by bodies.** `bodies`
owns the trait `Anchoring` (`anchoring_of`, `anchored(world, person)`, `fixed_points(world, place)`) and
its register function `register_anchorings`; `systems/installed` carries
`extension mineworld_bodies::Anchoring => mineworld_bodies::register_anchorings: [mineworld_posture::PostureSystem,];`.
Like the first two lines, an implementation applies wherever its pack's state is present, and a person is
anchored when any implementer says so. An implementer's `System::install` calls
`mineworld_bodies::require_anchoring` first. Bodies names no implementer; a later pack that pins a person
(a bed, a vehicle) adds an `impl` and an entry on this line.
