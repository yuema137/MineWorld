# sdk/rust — `mineworld-sdk`

What a Rust System Pack says about itself so the build never has to be told: the `SystemPack`
trait, and the `installed!` macro the build's list of packs is written in.

A pack implements it beside its `System`:

```rust
impl SystemPack for ConversationSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
}
```

The `PACKAGE` line is required and always the same: it records the pack's identity — name, version,
licence, authors — from its own `Cargo.toml` ([`docs/DECISIONS.md`](../../docs/DECISIONS.md) `ARC-53`).
`mineworld packs list` shows it.

and is installed with two lines in [`../../systems/installed/`](../../systems/installed/), then a
rebuild:

```text
Cargo.toml    mineworld-<name> = { path = "../<name>" }
src/lib.rs    <Variant> => mineworld_<name>::<System>,
```

The rules — what a pack declares, what is refused, and why installing always means a rebuild in
MVP-0 — are [`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §3.1 and
[`docs/DECISIONS.md`](../../docs/DECISIONS.md) `ARC-33`.

```sh
cargo test -p mineworld-sdk
cargo doc -p mineworld-sdk --open
```
