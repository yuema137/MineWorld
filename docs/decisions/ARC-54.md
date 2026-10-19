## ARC-54 — A world's requirements are resolved against the build and the named pack roots

**Date** 2026-10-08 · **Approved by** the operator (S16 QSE-8, QSE-13) and the primary session at PR
E-b's design freeze (step-16 §15.0; FQ-b1, FQ-b3, FQ-b4) · **Implements**
[`MODULE_SPEC.md`](MODULE_SPEC.md) §4 (the model's `requires:`), §4.1, §8.1;
[`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md) §5.0 · **Relates to** `ARC-33`, `ARC-53`, `ARC-55`, `DEP-21`,
`ARC-61` · **Design** `.structured-coding/plans/mvp0/step-16-packages.md` §4.2, §15 (S16, PR E-b)

**Problem.** `ARC-53` gave every pack an identity, but a world could not yet say which packs it needs or
at which versions, and nothing refused a world whose packs were missing or wrong. "Install modules →
compose world" needs the composition to be stated by the world and checked before the world runs.

**Choice.**

1. **`requires:`**, a top-level map in `world.yaml`, pack id → semver range (Cargo's meaning,
   `DEP-21`). It replaces the frozen model's `entity_packs:` and `presentation_profile:` (QSE-8). It names
   packs and versions only: `systems:` still says which systems are enabled and in which order, and how a
   pack is configured belongs to the generic configuration seam (`ARC-61`, S17). Neither of the two reads
   the other.
2. **Only packs that are not bundled are named.** A code pack is *bundled* when it was compiled from the
   framework's own workspace; its version is the framework's, so the world's `mineworld:` range covers it.
   `package!()` decides this at compile time — the crate's manifest directory begins with the framework
   workspace's root, taken as `mineworld-packages`' own manifest directory without its last component —
   and records only a boolean, never a path. Every other code pack is third-party.
3. **Pack roots, explicit only.** Packs are searched in the build and in the directories named by
   `--packs DIR` (in order), then by the `MINEWORLD_PACKS` path list. Nothing else is searched — not the
   world's own directory, not the working directory. Only the composition root reads the environment;
   `mineworld_packages::PackRoots::new(cli, env)` is a pure function of both.
4. **Resolution is checked, never chosen.** One installed version per pack; no solver. A pure function,
   `mineworld_packages::resolve`, takes the world's stated facts, each enabled system's pack and every
   data pack found under the roots, and refuses the first failure by name, in a fixed order: a duplicate
   id; per requirement in id order — absent (naming every place searched), a type a world cannot require
   (a World or Controller Pack; an Entity Pack until S16's E-d), bundled, outside the range; an enabled
   third-party system that is not required; a licence outside the policy (`ARC-55`).
5. **Where.** `WorldPack::read_with(root, &PackRoots)` resolves after `systems` and before content.
   `WorldPack::read(root)` is `read_with` with no roots, so a world without `requires:` reads exactly as
   before and no existing caller changes. The loader that builds a world (`load.rs`) is not involved:
   nothing is seeded from a composition.
6. **Never world state.** No fact, `Metadata` or save carries a version, a root or a composition (S16
   QSE-14). A resume resolves again against the roots given then, and its composition is still checked by
   `SystemVersion` alone.
7. **What prints it.** `mineworld packs resolve <world>` prints the whole composition. `mineworld
   validate` prints one `requires` line per requirement, only when the world states `requires:`, so a
   world without one reports exactly what it did before (a departure from step-16 §9.3's "validate
   prints the composition", recorded at freeze, FQ-b4).
8. **Every command that reads a world takes `--packs`**: `validate`, `run`, `server`, `replay`,
   `biography`, and `packs list | show | validate | resolve`. `create` does not: it writes a world that
   requires nothing. Its template states `version: 0.1.0`, `license: MIT` (to be replaced by the author)
   and `mineworld: "^0.1"` (FQ-b1).

A World Pack's own `world.version` and `world.license` stay optional to the loader (`ARC-53` point 6);
`packs validate` and `packs resolve` require them.

**Accepted limitations.**
- A third-party code pack *vendored inside* the repository counts as bundled, because its manifest lies
  under the workspace root. Vendoring a third-party pack into the tree is not a supported installation in
  MVP-0.
- The rule that an enabled third-party system must be required is proven through the real binary only
  once a third-party code pack exists in the build (S16's E-c); in E-b it is held by the resolver's own
  tests.
- A data pack's own `dependencies` on other data packs are not resolved; `pack.yaml` refuses the field.

---

## ARC-54 note — a required data pack's own framework range is checked (F-Ed1, 2026-10-08)

Point 4's rule 2 gains a check `ARC-54` left out: **a required data pack whose `mineworld:` range does
not admit the running framework is refused**, naming the pack, its range and the framework's version —
after the version check, as the last check of that requirement. `mineworld packs validate` applies the
same check to every `pack.yaml` pack. Before S16's PR E-d a Presentation Pack stating `mineworld: "^9"`
passed both (finding F-Ed1, spike SC-7), although step-16 §4.2's refusals list "a framework outside
`mineworld:`" and `ARC-53` says a package fact is never ignored. A bounded correction of E-b's
merged behaviour, landed with the PR that first reads data packs' content; the two shipped Presentation
Packs state `^0.1` and are unaffected. A World Pack's own range is checked as before, by the reader.
