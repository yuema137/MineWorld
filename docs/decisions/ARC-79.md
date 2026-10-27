## ARC-79 — Release channels: GitHub Releases canonical, itch.io mirror, Zenodo archive; the tag equals the workspace version

**Date** 2026-10-10 · **Status** selected · **Approved by** the operator's rulings on S23 (QR-1, QR-9,
QR-12, QR-13; step-23 §15), the step frozen by the primary session on 2026-10-10; number allocated by the
primary session · **Relates to** `ARC-48`, `ARC-78`, `ARC-80`, `DEP-39`, `DEP-40`, `DEP-42` · **Design**
`.structured-coding/plans/mvp0/step-23-release.md` §§4, 5, 10

**Written by.** Step-23 §12 assigns this record to PR R-b (the CI release workflow). R-b has not started,
so the record is written on 2026-10-26 with the docs-only PR that records operator ruling QB-14, from the
rulings already made; R-b adds a note for anything its implementation decides.

**Problem.** A downloadable MineWorld bundle (`ARC-80`) needs a place players can find it, a place that is
the single source of truth for its bytes, and a lasting citable archive, without making a maintainer copy
files by hand or letting two stores disagree about what a version contains.

**Choice — three channels, one source of truth.**

1. **GitHub Releases is canonical.** The release workflow (`.github/workflows/release.yml`, R-b) builds every
   target on a `v*` tag and creates a **draft** release with every bundle, `SHA256SUMS`, the SBOM
   (`DEP-40`) and attestations (`DEP-42`). Only the operator publishes the draft. A workflow never publishes.
2. **itch.io is the player page, a mirror.** `.github/workflows/publish.yml` (R-d) runs only on
   `release: published`, downloads the published release's assets, verifies `SHA256SUMS` and each
   attestation, then pushes them with `butler` (`DEP-39`) to the channels `windows`, `mac`, `linux` and
   `linux-arm64`. It runs in the GitHub Environment `itch`, which holds `BUTLER_API_KEY` and requires the
   operator's approval. The page is free (price 0, optional donation), declares the AI-generated assets, and
   ticks only platforms whose smoke legs passed (QR-13).
3. **Zenodo is the archive.** The GitHub–Zenodo integration mints a DOI for each published release.
   Release candidates are not archived: they stay drafts or pre-releases and are tested from workflow
   artifacts (QR-9).

**Choice — versions.** The tag equals the workspace version in `Cargo.toml` (`v0.1.0`; candidates
`v0.1.0-rc.N`, marked pre-release on GitHub and pushed to hidden itch channels). The release workflow's
`verify-tag` job refuses any other tag.

**Choice — dry runs.** `workflow_dispatch` on any ref builds, smoke-tests and uploads workflow artifacts
only. It creates a release only with the input `draft: true`, and then only a draft named `test-<run>`.

**Options compared** (step-23 §4). GitHub Releases (adopted as canonical: beside the source, attestable,
no player account, no cost); itch.io via `butler` (adopted as the player page); Zenodo (adopted as the
archive); itch.io as the canonical store (declined: no provenance binding to the source, §4.4); a
self-hosted object store or GitHub Pages (declined: cost or size limits and an operational surface, §4.4);
Hugging Face (declined for downloads, revisit for a hosted server, §4.5); package managers and Linux
formats such as Flathub, AppImage, Homebrew, winget and Scoop (deferred, §4.6, QR-7).

**Limitation accepted.** Three channels are three pages to keep consistent. The mitigation is that only
GitHub receives bytes from a build; itch.io and Zenodo receive only what GitHub published (risk R-R8).

## ARC-79 note — 2026-10-26: packaged clients are published publicly (operator ruling QB-14)

The operator ruled QB-14 on 2026-10-26: "要不要公开发布打包好的客户端 当然要啊，要不然别人玩不了" ("Should we publish
the packaged clients publicly? Of course, otherwise nobody else can play"). The packaged clients for
**macOS, Windows and Linux** are therefore published publicly on the channels above: the bundles on GitHub
Releases, mirrored to itch.io, archived on Zenodo. The ruling decides that they are published, not when.
The first public tag still follows QR-12 (after Milestone F's hands-on acceptance and after the visuals
settle), and the operator gives the go-ahead for publishing after milestone acceptance.
