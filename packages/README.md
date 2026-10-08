# mineworld-packages

What a MineWorld package is: every pack's **identity** — id, version, type, framework range, licence,
provenance — stated once, where the pack already says who it is.

- A System Pack writes one line, `const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();`,
  and its `Cargo.toml` says the rest.
- A World Pack states `version`, `license` and `mineworld` in its `world.yaml`.
- A Presentation Pack carries a `pack.yaml`.

`mineworld packs list` shows them all. The rules are in
[`docs/DECISIONS.md`](../docs/DECISIONS.md) `ARC-53` and
[`docs/PACKAGE_FORMAT.md`](../docs/PACKAGE_FORMAT.md) §5.0.
