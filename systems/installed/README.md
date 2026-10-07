# systems/installed — the build's System Packs

The list of every System Pack this build provides, one line per pack. It is the only such list:
the World Pack loader, the CLI and the server reach every pack through it.

Installing a pack is two lines here, then a rebuild:

```text
Cargo.toml    mineworld-<name> = { path = "../<name>" }
src/lib.rs    <Variant> => mineworld_<name>::<System>,
```

A world then enables the pack by naming it in its `systems:` list.
`cargo test -p mineworld-installed-systems` checks that the two lists agree and that no two packs
share an id. The rules are [`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §3.1 and
[`DECISIONS.md` `ARC-33`](../../docs/DECISIONS.md).
