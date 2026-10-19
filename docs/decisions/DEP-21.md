## DEP-21 — Versions and licence expressions: `semver` and `spdx`

**Date** 2026-10-08 · **Status** selected; dependencies added in S16 PR E-a · **Approved by** the
operator's reuse table (step-16 §5 rows 12–13) and the primary session at PR E-a's freeze ·
**Relates to** `ARC-53`, `DEP-10` · **Design** `.structured-coding/plans/mvp0/step-16-packages.md` §5,
§14 (S16, PR E-a)

**Problem.** `ARC-53` checks semver versions, semver ranges and SPDX licence expressions, and a data
pack's range must mean what a code pack's Cargo range means.

**Options considered** (`REUSE_POLICY.md` §§11–12, §17 — both directions):

```text
versions   (a) `semver` (dtolnay; Cargo's own semantics)  (b) `node-semver` (npm's)  (c) our own parser
licences   (d) `spdx` (Embark; the parser cargo-deny uses)  (e) a closed list of exact identifier
           strings  (f) our own expression parser
```

**Choice: (a) and (d).**
- **(a)** gives Cargo's meaning of `^0.1`, so a range in a `pack.yaml` admits exactly the versions the
  same range admits in a `Cargo.toml`. **(b)** differs from Cargo for pre-1.0 carets, a semantic
  mismatch. **(c)** would be a third dialect of a solved problem.
- **(d)** parses expressions (`MIT OR Apache-2.0`) against the SPDX licence list. **(e)** cannot read
  an expression, which is what Cargo's `license` field holds. **(f)** is the wheel (d) already is.

**Isolating interface.** `mineworld-packages`' `Version`, `Compatibility` and `License`. No other
crate names `semver` or `spdx`.

**Weight, measured** (step-16 §14.8 E-Ea2). `semver` 1.0.28 (MIT OR Apache-2.0) has no dependency;
`spdx` 0.13.6 (Apache-2.0), with its default features (none), depends only on `smallvec`, already in
the build. `Cargo.lock` gains three packages: the two and `mineworld-packages`. No build script.

**Accepted limitations and the revisit trigger.** A registry (the publishing sense of Milestone E,
non-goal) brings version selection, which is a solver's problem and is not solved here; revisit then.
