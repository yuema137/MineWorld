## DEP-50 — The executable guide: `pulldown-cmark` and `snapbox` as dev-dependencies of `mineworld-cli`

**Date** 2026-10-14 · **Status** accepted (design, step-25 §4.2 option E2); implemented by PR S25c (dev-dependencies only, no runtime edge) · **Approved by** the operator's step-25 rulings of 2026-10-14 (QS25-1 to QS25-10; the design's §4.2 recommendation accepted under the standing instruction) · **Relates to** `ARC-90`, `DEP-29`, `DEP-49`, `CLAUDE.md` §4 rule 16, `REUSE_POLICY.md` §§1–4, 11–12 · **Design** `.structured-coding/plans/mvp0/step-25-creator-docs.md` §§4.2, 8, 16 · **Specification** [`GUIDE_FORMAT.md`](GUIDE_FORMAT.md)

**Problem.** Every page in `docs/guide/` must be executed against the real `mineworld` binary, in order, on Linux, macOS and Windows, so that a page which stops working is a red test. The runner needs a CommonMark parser that yields fenced blocks with their info strings and line numbers, and an output matcher with line elision and in-line wildcards.

**Options considered** (verified 2026-10-14 with `cargo info` and the repository metadata, step-25 §16).

| Option | Verdict |
| --- | --- |
| E1 `trycmd` 1.2.1 (MIT OR Apache-2.0) | Declined as the runner. Its per-case sandbox does not model a tutorial whose steps are cumulative; its file steps need a second copy of every file in an `.in/` directory. Its matching rules are adopted through `snapbox`, which is the library `trycmd` itself uses. |
| E2 `pulldown-cmark` 0.13.4 (MIT, not archived, last push 2026-10-12, MSRV 1.71.1) plus `snapbox` 1.2.2 (MIT OR Apache-2.0, not archived, last push 2026-10-04, MSRV 1.85) | **Adopted.** A test in `mineworld-cli` walks each page, writes file blocks, runs command blocks with the binary and matches output. Both are test-only and never enter a build. |
| E3 mdBook `test` | Declined: it runs Rust code blocks; the guide's blocks are YAML and shell. |
| E4 Python runner with `markdown-it-py` | Declined: the binary's tests already run on all three operating systems under `cargo test`; a Python runner would need a new job and a new dependency. |
| E5 Test the example world only | Declined: a page can describe what the world no longer does. |
| Hand-written Markdown scanner | Declined: a CommonMark fence rule (the fence length, indentation, and `~~~` fences) is exactly what a maintained parser already gets right, and a bespoke one would drift from what GitHub renders. |

**Choice.** `pulldown-cmark` (parsing) and `snapbox` (matching) as `[dev-dependencies]` of `mineworld-cli`, each with a workspace entry in the root `Cargo.toml` carrying this record's id. Neither crate is named outside `tools/cli/tests/`. Feature sets are chosen at C1 and recorded there.

**Consequences.** The guide runner adds two crates to the test graph of `mineworld-cli` only. `cargo deny check licenses` must still pass (both are on the existing allow-list: MIT, Apache-2.0). The runner's TOML reading of `features.toml` uses the `toml` crate already present in `Cargo.lock`, so it adds no new crate; the workspace entry is recorded when C1 adds it.

**Revisit** if `pulldown-cmark` or `snapbox` leaves maintenance, or if the guide's block grammar needs a construct CommonMark cannot express (then the grammar changes first, in `GUIDE_FORMAT.md`).
