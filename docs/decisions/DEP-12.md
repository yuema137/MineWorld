## DEP-12 — System Pack registration: a declared installed set, not linker-section registration or dynamic loading

**Date** 2026-10-07 · **Status** selected; no dependency added · **Relates to** `ARC-8`, `ARC-33`,
`DEP-10` · **Design** `.structured-coding/plans/mvp0/step-10-market.md` §2.2 (S9, PR 11a)

**Problem.** Make a statically linked Rust System Pack installable by declaring it in one place,
with no code elsewhere that has to learn it (`ARC-33`). The build must still be able to do what the
World Pack loader's catalog does with each pack:
- resolve it by id;
- install it;
- hand it out as a perception provider;
- list its biographical event types;
- decode its authored section **generically over the YAML stream**, so that a refusal keeps the line
  and column `DEP-10` chose `serde-saphyr` for.

**Options considered** (`REUSE_POLICY.md` §§11–12, §17 — both directions):

```text
(a) the status quo: a closed enum in worldpack, one arm per pack
(b) linker-section registration: `inventory` (dtolnay, MIT/Apache-2.0) or `linkme`'s distributed_slice
(c) dynamic loading: `libloading`, `abi_stable`
(d) a value registry with type-erased section decoding: `erased-serde`
(e) worldpack generic over a catalog type supplied by the binary
(f) a build script scanning systems/*/Cargo.toml to generate the list (needs the `toml` crate)
(g) a SystemPack trait each pack implements, and an installed-set crate whose only content is the
    list, expanded by a declarative macro into the closed enum (`ARC-33`)
```

**Choice: (g).** No dependency is added. The "implementation of our own" is a `macro_rules!` over the
code the catalog already held.

**Why not the others** (`REUSE_POLICY.md` §12's reasons):

- **(a)** is finding F-1 itself: about eleven edits in five files per pack.
- **(b) `inventory` / `linkme` — dependency larger than the problem, and an architecture mismatch.**
  - They solve distributed registration of *values*. MineWorld's remaining cost after (g) is not
    registration code; it is the one Cargo line no crate can remove. Both also need a `use pack as
    _;` line, or the linker drops a crate nothing references, so they would not save even the list
    line.
  - They would add a dependency with life-before-main (`inventory`) or per-platform linker support
    (`linkme`).
  - Registering values would force the section decoder from a generic function into a type-erased
    value, which is (d)'s risk.
  - This is the case of forcing an existing wheel where it does not fit (`REUSE_POLICY.md` §17), not
    of reinventing one.
- **(c) `libloading` / `abi_stable` — architecture mismatch and an unacceptable trust model.** Loading
  native libraries needs `unsafe` (every crate here is `forbid(unsafe_code)`), relies on an unstable
  Rust ABI, and runs downloaded native code with full privileges. `ARC-8` already chose the WASM
  component model for code that is not compiled into the build.
- **(d) `erased-serde` — missing required semantics, unverified.** Whether `serde-saphyr`'s line and
  column survive an erased round trip has not been shown, and they are `DEP-10`'s reason for the
  parser. It would add a dependency to put that at risk.
- **(e) a generic worldpack — inability to isolate it cleanly.** Every caller of `WorldPack::read` —
  the CLI, the server's and persistence's tests, worldpack's own tests — would change, and
  worldpack's tests would need a dev-dependency cycle to name a catalog.
- **(f) a build script — dependency larger than the problem.** Cargo still needs the dependency
  line, so the script would save one list line at the price of a TOML parser in the build.

**Isolating interface.** `mineworld-sdk`: the `SystemPack` trait, `SectionOwner`, and the macros
`owns_section!` and `installed!`. A pack names only `SystemPack`; the World Pack loader names only
the generated `Capability`. If registration moves to a startup registry, these two surfaces are what
change.

**Accepted limitations and the revisit trigger.**
- One Cargo line per pack remains, as it must for static linking (`ARC-33`).
- Revisit when the first build installs packs it does not compile from this repository, which is
  `ARC-8`'s Tier 1. A runtime registry is then needed anyway, and the comparison above is reopened
  rather than assumed.
