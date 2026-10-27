## DEP-40 — SBOM: `anchore/sbom-action` (syft), SPDX JSON, attested with `actions/attest`

**Date** 2026-10-27 · **Status** selected · **Approved by** the operator's freeze of PR S23 R-b
(2026-10-27, every question answered as recommended) · **Relates to** `ARC-79`, `DEP-42`, `DEP-38`,
`DEP-19` · **Design** step-23 §5.4, §12; `pr-s23-rb-release-workflow.md` §3.7

**Problem.** A downloadable bundle built by one maintainer should say what it was built from, in a form a
reviewer's tools can read, at a cost of one workflow step.

**Choice.** `anchore/sbom-action` v0.24.3, pinned by commit SHA `66cbf4bc1f1c0d2edc94016e65bc221b6bb0ad6c`
(the tag `v0.24.3`'s commit, read with `gh api` 2026-10-26 and re-read 2026-10-27), run by
`.github/workflows/release.yml`'s `draft` job on the checkout at the released commit with
`format: spdx-json`, writing `MineWorld-<version>-sbom.spdx.json`. Its scope is the source tree's lock files
(`Cargo.lock`, `uv.lock`). The SBOM is a release asset, listed in `SHA256SUMS`, and bound to each of the
four archives by `actions/attest` with `sbom-path` (`DEP-42`). The release notes say that it covers the Rust
and Python graphs only.

**Options compared.** `cargo-cyclonedx` (the Rust graph only, a second format to explain); `cargo-auditable`
(embeds the dependency list in each binary — complementary and cheap, worth a later look, not a release
asset); no SBOM (acceptable for a pre-alpha, but the cost is one step).

**Limitation accepted.** Godot's bundled libraries are not in any lock file the action reads; they are
listed in each archive's `LICENSES/godot/COPYRIGHT.txt`. The action downloads syft at the version its own
pinned commit names; that version moves only when this record's pin moves.
