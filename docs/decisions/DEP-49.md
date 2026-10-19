## DEP-49 — JSON Schema derivation: `schemars` 1.x; `clap-markdown` declined

**Date** 2026-10-14 · **Status** accepted; implemented by PR 25a · **Approved by** the operator's ruling QS25-2 and the step-25 freeze of 2026-10-14 · **Relates to** `ARC-90`, `DEP-10`, `DEP-11`, `DEP-22`, `REUSE_POLICY.md` §§1–3, 11–12 · **Design** step-25 §§4.1, 16; `pr-s25a-reference.md` §§2–3

**Problem.** `ARC-90` derives a JSON Schema from every type that decodes a World Pack's files, including serde's attributes (`deny_unknown_fields`, `default`, `rename`, `transparent`, `try_from`), and renders the `mineworld` command's own surface.

**Options considered.**

| Option | Verdict and reason |
| --- | --- |
| **`schemars` 1.2.3** (MIT; `GREsau/schemars`, not archived, pushed 2026-10-12; MSRV 1.74) | **Chosen.** Reads serde's attributes and doc comments; manual `impl JsonSchema` for custom decoders; `#[schemars(with, example)]`. Adds `schemars_derive`, `serde_derive_internals`, `dyn-clone` and `ref-cast`, all on the workspace's existing `syn` 3 |
| Our own derive (R2) | Declined: reinvention of a maintained crate |
| rustdoc JSON (R3) | Declined: nightly; does not see serde's behaviour |
| Schema-first with `typify` (R5) | Declined: inverts the decoding type's ownership |
| `clap-markdown` 0.1.5 for the CLI page | Declined: a 0.1 crate for about 80 lines over `clap`'s `Command` model, which the build already depends on (`DEP-11`) |

**Choice.** `schemars = { version = "1.2.3", default-features = false, features = ["derive", "std"] }` in `[workspace.dependencies]`, named by `mineworld-authoring`, `mineworld-sdk`, `mineworld-packages`, `mineworld-worldpack`, the System Packs that own a section or configuration, and `mineworld-cli`. The `preserve_order` feature is **not** enabled: it would turn on `serde_json/preserve_order` for the whole workspace and could change bytes a world serializes (`INV-S25-5`); the reference lists a type's fields in name order instead. `mineworld-contracts` never names the crate. The CLI page is rendered by walking `clap`'s `Command`.

**Consequences.** `cargo deny check licenses` passes with the added crates (MIT, or MIT OR Apache-2.0). A third-party pack depends on `schemars` from the registry at the version the build's lock holds.

**Revisit** when `schemars` 2 is released, or if its maintenance stops.
