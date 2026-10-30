## DEP-39 — `butler` pushes the release bundles to itch.io, pinned by version and SHA-256

**Date** 2026-10-27 · **Status** selected · **Approved by** the operator's ruling QR-1 and QR-5 (step-23 §15, 2026-10-10)
and the S23 R-d design freeze of 2026-10-27 · **Relates to** `ARC-79`, `ARC-80`, `DEP-40`, `DEP-42` · **Design**
`.structured-coding/plans/mvp0/pr-s23-rd-publish.md` §§3.1, 3.3, 6; step-23 §§4.2, 12

**Problem.** The itch.io mirror (`ARC-79`) needs a command-line uploader that runs on a Linux runner and reads its
key from the environment. Its install page recommends a download service for CI and publishes no checksum or
signature, so the bytes we run must be pinned by us.

**Choice.** `butler` from `itchio/butler`, version **v15.32.0** (GitHub release, published 2026-10-06, read
2026-10-27 with `gh api repos/itchio/butler/releases/tags/v15.32.0`). The asset is
`butler-linux-amd64.zip`, 10 599 491 bytes, with GitHub's digest
`sha256:5db7b053fc9b578b86ffaf3520a12b952eb95c9225171cc5720dfbc9fe4574b3`. The release URL is
`https://github.com/itchio/butler/releases/download/v15.32.0/butler-linux-amd64.zip`. The implementing session
downloaded that asset on 2026-10-27 and confirmed the same SHA-256 with `shasum -a 256`. `publish.yml` refuses the
archive unless its SHA-256 equals the pin in the workflow, before the API key is read. The pin is ours: upstream
publishes no checksum file.

**Options compared.** The broth service (`broth.itch.zone`, recommended by itch.io for CI) serves a `LATEST` build
that changes without a review, so it cannot be pinned by us. Building butler from source adds a Go toolchain to the
publish job for no gain. A third-party GitHub Action that wraps butler adds an unreviewed dependency that would hold
the API key (`ARC-79`, A-R10); rejected.

**Interface.** A publishing tool only: `publish.yml`'s `push` job calls it with `butler push <folder>
<ITCH_TARGET>:<channel> --userversion <version>` and `butler status`. No MineWorld code depends on it. The key
`BUTLER_API_KEY` reaches only those two steps' `env:`.

**Limitation accepted.** The pin protects against a changed download, not against a compromised upstream release
whose digest GitHub also reports. The attestation-free upstream is accepted because the pin is reviewed in this
record and every change to it is a reviewed diff.
