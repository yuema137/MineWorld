## DEP-42 — Provenance: GitHub artifact attestations through `actions/attest`

**Date** 2026-10-27 · **Status** selected · **Approved by** the operator's freeze of PR S23 R-b
(2026-10-27, every question answered as recommended) · **Relates to** `ARC-79`, `DEP-40`, `DEP-19`
· **Design** step-23 §5.4, §12; `pr-s23-rb-release-workflow.md` §3.7

**Problem.** Public binaries from one maintainer need a checkable origin: a player or a reviewer must be
able to tell that an archive was built by this repository's workflow from a named commit.

**Choice.** `actions/attest` v4.2.2, pinned by commit SHA `1e69f48acb82d1966a394da916b4c1698aa569d6` (read
with `gh api` 2026-10-26 and re-read 2026-10-27). The `draft` job of `.github/workflows/release.yml` — the
only job with `id-token: write` and `attestations: write` — attests the four archives and `SHA256SUMS`
(build provenance), then binds the SBOM (`DEP-40`) to each archive with `sbom-path`. The job verifies
every attestation with `gh attestation verify <file> -R yuema137/MineWorld` (and, for the SBOM,
`--predicate-type https://spdx.dev/Document/v2.3`) before it creates the draft. A public repository signs
through the Sigstore Public Good Instance and the attestations are written to its public transparency log
(GitHub docs, "Artifact attestations" and "Using artifact attestations to establish provenance for
builds", read 2026-10-26: available for public repositories on every current plan).

**Options compared.** `slsa-framework/slsa-github-generator` (SLSA build level 3 through reusable
workflows; heavier, verified with `slsa-verifier`, a second tool for players to learn); no provenance
(declined: the archives are unsigned for the pre-alpha, QR-3/QR-4, so the attestation is the only
statement of origin).

**Limitation accepted.** A dispatch dry run with `draft: true` also attests its archives (that is how the
step is tested). Those attestations name a non-tag ref, so a verifier who checks the source ref of a
release tag never accepts them.
