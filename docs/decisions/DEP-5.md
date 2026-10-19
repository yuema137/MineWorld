## DEP-5 — Serialization: `serde`

**Date** 2026-09-25 · **Status** selected, in use since S1

`serde` with derive for every contract type, `serde_json` for the wire and for tests. The Rust
ecosystem standard, zero operational burden, and format-agnostic so a binary encoding later needs
no contract change. `thiserror` accompanies it for typed errors, which
[`ENGINEERING_STANDARDS.md`](ENGINEERING_STANDARDS.md) §12 requires anyway. Both MIT/Apache-2.0.

**Note, 2026-09-30 (S5, PR 07).** `serde_json` becomes a runtime dependency of `mineworld-kernel`,
not only a dev-dependency. A world snapshot must encode every component row, and only the kernel still
holds those rows as their Rust types (`DEP-1`'s store erases the type behind each table); it encodes
each as a `ComponentRecord` whose payload is the component's JSON. This is the persistence encoding of
state the kernel holds, not a choice of payload format for a system: event and action payloads remain
bytes a system encodes and the kernel never interprets. JSON over values ordered by `BTreeMap` is
canonical — and `clippy.toml` bans `HashMap` across the workspace — so equal state encodes to equal
bytes, which is what replay verification compares (`ARC-25`).
