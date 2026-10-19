## DEP-43 — Snapshot compression: `zstd` (bundled libzstd), level 3

**Date** 2026-10-10 · **Status** selected; one dependency added to `mineworld-persistence` only ·
**Approved by** the primary session (QSR-6) and the operator's QSR-1 ruling, 2026-10-10 · **Relates
to** `ARC-25`, `ARC-49`, `ARC-55`, `ARC-81`, `DEP-2`, `DEP-22`, `REUSE_POLICY.md` §§1–4, 11–12 ·
**Design** `.structured-coding/plans/mvp0/pr-s6-save-retention.md` §5

**Problem.** A plateaued Market Town snapshot is ≈ 700 KB of JSON, written every 64 revisions on the
world thread. It must shrink by an order of magnitude, cheaply, and the stored bytes must be the same
on macOS, Linux and Windows, because `AC-8` hashes every stored byte of a 30-day save on all three
(`ARC-49`).

**Options considered** (one day-30 snapshot of 703 730 B, measured on the design laptop).

```text
zstd 3 (zstd crate, bundled libzstd)     52 045 B  13.5×   ≈ 1.2 ms; decompress ≈ 1 GB/s   chosen
zstd 9                                   41 550 B  16.9×   ≈ 5× slower than level 3
zstd 19                                  33 021 B  21.3×   seconds per 50 snapshots
deflate 6 (miniz_oxide / flate2)         45 672 B  15.4×   ≈ 16 ms: ≈ 13× slower
lz4 (lz4_flex, pure Rust)                96 989 B   7.3×   fastest, 1.9× larger
ruzstd (pure Rust zstd)                  its own documentation: the encoder does not yet reach the
                                         reference library's speed or ratio        the fallback
sqlite-zstd                              LGPL-3.0 (outside ARC-55); asynchronous chunking
ZIPVFS                                   proprietary licence
```

**Choice.** `zstd = { version = "0.13", default-features = false }` in the workspace, used only by
`persistence/src/format.rs` (the codec behind `encode_snapshot` / `decode_snapshot`, `ARC-81`). Default
features off: no legacy-format decoding, no dictionary builder, no multithreading. libzstd is compiled
from the source bundled in `zstd-sys` with the same `cc` path `libsqlite3-sys` already uses (MSVC on
Windows, clang on macOS, gcc in the Linux container): no new toolchain. `zstd-sys`'s `pkg-config`
feature is never enabled, and no MineWorld build or CI job sets `ZSTD_SYS_USE_PKG_CONFIG` (the
environment variable that would link a system libzstd instead); the exact versions are pinned by
`Cargo.lock`.

**Determinism.** One compression level, no dictionary, single-threaded, checksum and content size in
the frame header: the stored bytes are a pure function of the JSON bytes and the pinned libzstd. That is
checked rather than assumed: `AC-8` compares the `snapshots` table's stored bytes across the three
platforms. A future zstd upgrade may change the stored bytes of new snapshots; it changes no decoded
state and no replay verdict (`ARC-81`: comparison is of decoded JSON), and `AC-12` compares two runs of
one build.

**Licences** (`ARC-55`, `DEP-22`), as `Cargo.lock` admits them: `zstd` 0.13.3 is MIT; `zstd-safe`
7.3.0 and `zstd-sys` 2.1.1+zstd.1.5.7 declare BSD-3-Clause, the licence the bundled libzstd 1.5.7 is
taken under (upstream is dual BSD / GPL-2.0). `zstd-sys` builds with `cc`'s `parallel` feature, which
adds build-time crates only: `jobserver` 0.1.35 and `getrandom` 0.4.3 (MIT or Apache-2.0) and `r-efi`
6.0.0 (MIT or Apache-2.0 or LGPL-2.1-or-later, taken under MIT; compiled only for UEFI targets).
`cargo deny check licenses sources bans` admits the graph.

**Re-evaluation trigger.** A platform build of `zstd-sys` fails, or `AC-8` parity differs in the
`snapshots` table: then `ruzstd` (MIT, pure Rust) is the named fallback, or `ci_parity` hashes decoded
snapshot JSON (a reviewed change to `ARC-49`). Either is a decision returned to the operator.
