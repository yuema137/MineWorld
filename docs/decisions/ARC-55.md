## ARC-55 — Which licences a pack may carry: a default policy, typed, overridable per world

**Date** 2026-10-08 · **Approved by** the primary session at PR E-b's design freeze (step-16 §15.0,
FQ-b2 as changed) · **Implements** [`MODULE_SPEC.md`](MODULE_SPEC.md) §4.1,
[`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md) §5.0 · **Relates to** `DEP-8`, `DEP-21`, `ARC-53`, `ARC-54`,
`ARC-61` · **Design** `.structured-coding/plans/mvp0/step-16-packages.md` §4.7, §15 (S16, PR E-b)

**Problem.** MineWorld redistributes what a world is composed of, and "a pack whose licence cannot be
resolved cannot be redistributed" (`PACKAGE_FORMAT.md` §5). `ARC-53` checks that a licence *is* an SPDX
expression; nothing yet judged whether it is one MineWorld may redistribute under MIT. MineWorld is a
framework, so the judgement is a default a world may change, not a rule fixed in code.

**Choice.**

1. **The default allow-list** is the common permissive set compatible with MIT redistribution: `MIT`,
   `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `CC0-1.0`, `Unlicense`.
2. **An expression is judged by `spdx`'s own evaluator** (`DEP-21`): allowed when it can be satisfied
   with listed identifiers alone, a requirement carrying a `WITH` addition or `+` counting as not
   listed. `MIT OR GPL-3.0-only` is allowed (one branch suffices); `MIT AND GPL-3.0-only` is not. A
   refusal names the pack, its expression, the identifiers that failed and the allowed ones.
3. **The policy is a typed value**, `mineworld_packages::LicencePolicy`, passed to the resolver rather
   than read from a constant; `LicencePolicy::default()` is the list above.
4. **It applies at resolution** (`ARC-54` point 4) — to the world's own licence when stated, every
   required pack, and every enabled system's pack — and in `mineworld packs validate`. `packs list` and
   `packs show` print licences and do not judge them. `cargo-deny` over the whole Cargo graph in CI
   remains S16 E-c's and S13's.
5. **Overridable per world, through no key of its own.** When the generic configuration seam lands
   (`ARC-61`, S17's PL-a), a world's `configure/packages.yaml` decodes into `LicencePolicy` and replaces
   the default for that world, narrowing or extending it. Until then every world uses the default. No
   `world.yaml` key is added for it.

**Options considered.** (a) A closed list hard-coded in the resolver — rejected: a framework must let a
world decide. (b) A dedicated `license_policy:` key in `world.yaml` — rejected at freeze: one generic
seam for configuration, not a bespoke key per concern. (c) A typed default overridable through the
generic seam — chosen.

**Candidates left out of the default, on purpose.** `CC-BY-4.0` requires attribution, which MineWorld
does not yet track; it is the first candidate for asset packs once it does. Copyleft licences
(`CC-BY-SA-*`, `GPL-*`) stay out, for `DEP-8`'s reason.

**Note, 2026-10-08 (S17, PR IL-b; QIA-1, QIB-7) — point 5 is wired.** `packages` is a framework key of
`configure:` (`ARC-61` note). When a world lists it, `configure/packages.yaml` is decoded straight into
`LicencePolicy` (`{ allowed: [...] }`, each identifier checked against the SPDX list; a refusal is
`LicencePolicyInvalid`, with its line and column) right after the world's systems resolve and before its
requirements do, and that policy replaces the default for the world: in resolution and in `mineworld
packs validate <world dir>`. `packs validate <pack dir>` keeps the default. The policy is not world
state: it is never seeded and never drift-checked, and is read again at every read, including every
resume.
