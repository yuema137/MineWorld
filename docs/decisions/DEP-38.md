## DEP-38 — `cargo-about` generates the Rust third-party notices of a bundle

**Date** 2026-10-10 · **Status** selected · **Approved by** the S23 freeze · **Relates to** `DEP-22`,
`ARC-80` · **Design** step-23 §3.1, §12

**Problem.** A bundle redistributes the server and launcher binaries, which link every crate of
`Cargo.lock`'s graph that they use. MIT and Apache-2.0 require the licence text and notices to travel
with binaries; a hand-kept file drifts from `Cargo.lock`.

**Choice.** `cargo-about` (Embark Studios, MIT OR Apache-2.0), version **0.9.2** (crates.io, read
2026-10-10), installed with `cargo install cargo-about --version 0.9.2 --locked --features cli` (0.9.2 builds its
binary only with the `cli` feature), run at packaging time
against `Cargo.lock` with the repository's configuration and template under `packaging/`. It writes
`LICENSES/THIRD_PARTY_RUST.html`. A crate whose licence the configuration does not accept fails the run,
so a new licence is a reviewed change, as `DEP-22`'s `deny.toml` already makes it for the graph.

**Options compared.** `cargo-license` (lists names and SPDX ids, no licence texts: insufficient);
`cargo-bundle-licenses` (less used; its maintenance was not verified, and nothing it adds was needed); a hand-written
file (drifts); `cargo-deny` (already checks licences, `DEP-22`, but emits no notices).

**Interface.** A packaging tool only: `scripts/package.py` calls it; no crate depends on it, and nothing
it produces is read by MineWorld.

**Limitation accepted.** It covers the Rust graph only. Godot's bundled third-party code is covered by
Godot's own `COPYRIGHT.txt`, shipped beside it.
