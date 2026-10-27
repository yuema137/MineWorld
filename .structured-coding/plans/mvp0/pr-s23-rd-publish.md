# PR S23 R-d — Publishing: itch.io mirror, Zenodo archive, changelog, citation, download docs

## DESIGN FROZEN 2026-10-27 (operator approved in the primary session: every question answered as recommended) (not frozen)

**Ruled 2026-10-27 (operator, primary session):** every question below is answered as recommended.

```text
Design revision:        revision 1 (2026-10-26), DRAFT (not frozen)
Author:                 design sub-agent (Opus), worktree /Users/yuema137/mineworld-worktrees/design-s23-rb-rd,
                        branch docs/pr-design-s23-rb-rd, planning base origin/main @ 0b2faf2f
Approved by / evidence: nobody yet. Only the operator freezes this design (CLAUDE.md §3.1)
Implementation base:    origin/main after R-b (pr-s23-rb-release-workflow.md) merges
Start condition:        this design frozen; R-b merged; PR #497 (ARC-79, QB-14) merged; the operator actions
                        of §5 that the dry runs need (none) — the live checkpoint needs all of them
Execution contract:     §12, left unfilled on purpose
Implementer:            Opus-class for publish.yml (secret handling); Haiku-class for the docs commits
Lifecycle:              DRAFT (not frozen)
```

**Effort:** `mvp0` · **Step:** [`step-23-release.md`](step-23-release.md) §§4.2, 4.3, 5.1, 10, 11 (R-R4, R-R8),
13.5, 14 (A-R9, A-R10), 15 (QR-1, QR-5, QR-9, QR-11, QR-13) · **Operator ruling:** QB-14 (2026-10-26,
`overall.md` §5, `ARC-79` note — written by PR #497): the packaged clients for macOS, Windows and Linux are
published publicly; *when* stays QR-12's · **Decision records:** writes `DEP-39` (butler); an `ARC-79` note
· **Depends on:** R-b, [`pr-s23-rb-release-workflow.md`](pr-s23-rb-release-workflow.md) (its artifact and
release-asset names, `SHA256SUMS`, attestations)

Every statement about an external service carries the date it was read and its source. **verify** marks
what this session could not confirm from a primary source.

---

## 0. The problem in one paragraph

After R-b, a tag leaves a draft release on GitHub. Players still cannot find it on itch.io, nobody can cite
it, the repository has no changelog, and the README and the guide still say "a ready-made download is
coming with the first release". QB-14 settles that the bundles *will* be public. R-d adds the path from "the
operator presses Publish on GitHub" to "the same bytes are on itch.io and archived with a DOI", and writes
the human-facing pages that point players to the download.

Example of the end state. The operator publishes draft `v0.1.0`. `publish.yml` starts and waits for the
operator's approval in the `itch` Environment. After approval it downloads the four archives from the
published release, checks `SHA256SUMS` and each attestation, and pushes each unpacked bundle to its itch
channel with `--userversion 0.1.0`. Independently, Zenodo receives GitHub's release webhook and mints a DOI.

---

## 1. Audit (origin/main @ 0b2faf2f, read 2026-10-26)

| Fact | Evidence |
| --- | --- |
| No `publish.yml`, `CITATION.cff`, `.zenodo.json`, `CHANGELOG.md` | `ls .github/workflows`; `ls` at the root |
| One GitHub release exists: `promo-v2.2`, **pre-release**, the intro video, linked from the README's cover | `gh release list`; `README.md:2–5, 262` |
| README has no download section; "Quick start" builds from source with Rust, Godot and Git LFS | `README.md:176–208` |
| Guide chapter 01, Step 1.1: "A ready-made download for macOS, Linux and Windows is coming with the first release. Until then, build the command from the source code." Mirror `01-get-mineworld.zh-CN.md` | `docs/guide/01-get-mineworld.md:9–11` |
| Every English guide page is executed by `tools/cli/tests/guide.rs`; `sh`/`powershell` blocks run on CI per OS; a mirror must match its page's blocks byte for byte; `operator-only` blocks are pending in every run and run by the operator at milestones; `requires X` with `X = false` is pending; `http(s)` links are not checked (no network in tests) | `docs/GUIDE_FORMAT.md` §§2, 4.6, 7, 8.1, 8.3, 9 |
| `packaging/PLAY.txt` already tells players how to check `SHA256SUMS` on each OS and how to open unsigned apps | `packaging/PLAY.txt:16, 35` |
| The bundle ships AI-generated assets (`NOTICE`), so itch.io's AI disclosure applies (QR-13) | `NOTICE`; step §4.2 |
| `ARC-79` (channels) and its QB-14 note are written by PR #497, open | `gh pr view 497` |
| `release.yml` (R-b) creates drafts only, with assets `MineWorld-<version>-<target>.<zip|tar.gz>`, `MineWorld-<version>-sbom.spdx.json`, `SHA256SUMS`, and attestations for each | `pr-s23-rb-release-workflow.md` §§3.6–3.8 |

### 1.1 External facts (read 2026-10-26)

- **butler.** Latest release `v15.32.0` (2026-10-06) on `itchio/butler` GitHub releases, with assets
  `butler-linux-amd64.zip` (10 599 491 bytes) and others. GitHub reports each asset's digest:
  `butler-linux-amd64.zip` `sha256:5db7b053fc9b578b86ffaf3520a12b952eb95c9225171cc5720dfbc9fe4574b3`
  (`gh api repos/itchio/butler/releases/latest`, field `digest`). itch.io's install page recommends broth
  for CI (`https://broth.itch.zone/butler/<channel>/LATEST/archive/default`) and documents **no checksums
  or signatures** (itch.io/docs/butler/installing.html). broth also serves fixed versions,
  `/butler/linux-amd64/15.32.0/archive.zip` (broth.itch.zone listing). Neither source publishes a checksum
  file, so the pin is **ours**: R-d pins the GitHub asset by version and the SHA-256 above, and the first
  implementation commit re-downloads and confirms it (**verify** that the bytes equal the digest).
- **Pushing.** A channel name containing `win`/`windows`, `mac`/`osx`, `linux` sets the platform tag;
  `--userversion` sets our version; `--if-changed` skips a push identical to the channel's last build;
  "pushing to a new channel will immediately make the upload visible on your game's page", unless
  `--hidden`, which works **only when the channel is new** (an error on an existing channel); builds over
  30 GB uncompressed are refused (itch.io/docs/butler/pushing.html).
- **Zenodo.** `.zenodo.json`, when present, wins outright: "Zenodo will only use the `.zenodo.json`
  metadata and ignore the `CITATION.cff` entirely" (help.zenodo.org, "Describe software / citation file").
  The same page implies, without stating it, that `CITATION.cff` is used when there is no `.zenodo.json`,
  and says Zenodo "implements a subset of the CITATION.cff schema" (**verify** which fields).
  Whether the integration archives **pre-releases**, and whether it archives release **assets** or only the
  source archive, is not stated on the pages read (help.zenodo.org GitHub upload page; search): both stay
  **verify**. Drafts never trigger it, because GitHub sends no release event for a draft (GitHub community
  discussion 26954, via search).
- **GitHub events.** Publishing a draft fires `release` with action `published` (the same discussion); a
  release created or published with `GITHUB_TOKEN` starts no other workflow (GitHub docs, long-standing
  rule, **verify** wording). The operator publishes by hand, so `publish.yml` runs.

### 1.2 Findings (F-Rd-n)

- **F-Rd-1 — QB-14 changes nothing in R-d's mechanism**, only its certainty: the packaged clients are
  published (macOS, Windows, Linux), and nothing is published before the operator's go-ahead after milestone
  acceptance (`ARC-79` note, QR-12). So R-d's merge must not need a public release; its live checkpoint
  (step §13.5) moves after the go-ahead (Q-Rd-1).
- **F-Rd-2 — `release: published` fires for every published release, including `promo-v*`.** A future
  promo video release would start `publish.yml`. Its first job therefore accepts only
  `^v\d+\.\d+\.\d+(-rc\.\d+)?$` and ends green with a notice otherwise; and the `itch` Environment accepts
  deployments only from tags `v*` (§3.4), a second lock.
- **F-Rd-3 — Zenodo may archive promo releases too.** The integration has no tag filter. If it archives
  pre-releases (**verify**), every future promo video release mints a DOI, and QR-9's "no Zenodo for
  candidates" also forces candidates to stay drafts. Q-Rd-2.
- **F-Rd-4 — the guide cannot download in CI.** The guide test has no network, and a `sh` block that
  downloads would fail. The download branch is therefore `operator-only` (GUIDE_FORMAT §8.1.4), and the
  onboarding layer B (pr-25g) gives the CI coverage by unpacking `release.yml`'s artifact on a runner with no
  Rust and no Godot (R-b §4.3). Q-Rd-6.
- **F-Rd-5 — an unsigned command-line binary is also quarantined on macOS.** A player who downloads the
  bundle and runs `runtime/mineworld` from Terminal meets Gatekeeper too; the guide's macOS block clears the
  quarantine attribute on the unpacked folder (`xattr -dr com.apple.quarantine <folder>`), the same
  fallback `PLAY.txt` names (QR-3).

---

## 2. Scope

| File | What |
| --- | --- |
| `.github/workflows/publish.yml` (new) | §3 |
| `scripts/ci_release.py` (R-b's) | `publish-gate`, `verify-assets`, `itch-channels`, `citation`, and their self-tests; `check-secrets` (§3.5) |
| `CITATION.cff` (new) | §4.1 |
| `.zenodo.json` | only if Q-Rd-3 rules for it |
| `CHANGELOG.md` (new) | §4.2; specification standard, entries cite PRs (QR-11) |
| `README.md` | a short "Download and play" section (human, §4.3) |
| `docs/guide/01-get-mineworld.md` and `.zh-CN.md` | Step 1.1's download branch (§4.4) |
| `docs/decisions/DEP-39.md`; `ARC-79` note; `INDEX.md` regenerated | §6 |

Non-goals: signing and notarization (QR-3, QR-4: unsigned); package managers (§4.6, deferred); a Hugging
Face mirror (declined, §4.5); changing `release.yml`'s jobs (R-b's).

---

## 3. `publish.yml`

### 3.1 Triggers and jobs

```yaml
on:
  release:
    types: [published]
  workflow_dispatch:                 # verification only: never pushes, never enters the Environment
    inputs:
      tag: { description: "release tag or draft name to verify", required: true, type: string }
permissions:
  contents: read
concurrency:
  group: publish-${{ github.event.release.tag_name || inputs.tag }}
  cancel-in-progress: false
```

```text
gate      ubuntu-24.04   ci_release.py publish-gate: tag matches ^v\d+\.\d+\.\d+(-rc\.\d+)?$ and
                         `prerelease` is true exactly for -rc.N; otherwise a notice and success with
                         output publish=false (promo releases, F-Rd-2). Outputs: tag, version, rc
verify    ubuntu-24.04   needs gate (publish=true) — or dispatch. No secret, no Environment.
                         gh release download <tag> into a scratch folder; sha256sum -c SHA256SUMS;
                         every asset is listed in SHA256SUMS and every SHA256SUMS line has an asset;
                         gh attestation verify <archive> -R yuema137/MineWorld
                           --signer-workflow yuema137/MineWorld/.github/workflows/release.yml
                           (+ --source-ref refs/tags/<tag> on a release run)
                         for each archive and SHA256SUMS. Uploads nothing on dispatch.
                         On a release run: uploads the verified archives as artifact
                         publish-<tag> (1 day) for the push job
push      ubuntu-24.04   needs verify; release event only; environment: itch
                         (required reviewer: the operator; deployments only from tags v*).
                         Downloads publish-<tag>; re-checks every archive against SHA256SUMS (the
                         artifact passed through storage); installs butler (§3.3); unpacks each
                         archive; butler push <folder> <ITCH_TARGET>:<channel> --userversion <version>
                         [--hidden on a channel's first push for -rc]; butler status at the end
```

Why `verify` is a separate job without the secret. The download, the checksums and the attestations need
no key. Keeping them outside the Environment means a reviewer approves a deployment that has already been
proven to carry GitHub's published bytes, and the job that holds the key runs as few steps as possible. The
push job re-checks the hashes after the artifact hop, so no bytes enter itch.io that `SHA256SUMS` does not
name (A-R9).

### 3.2 Channels

| Target archive | Release channel | Candidate channel (`-rc.N`) |
| --- | --- | --- |
| `windows-x86_64` | `windows` | `windows-rc` |
| `macos-universal` | `mac` | `mac-rc` |
| `linux-x86_64` | `linux` | `linux-rc` |
| `linux-arm64` | `linux-arm64` | `linux-arm64-rc` |

- Candidate channels are created with `--hidden` on their first push, so a candidate never shows on the page
  (itch docs: `--hidden` only on a new channel). Later candidate pushes go to the same, still hidden,
  channels. The release channels are created visible, on the first real release.
- The itch app cannot tell `linux` from `linux-arm64` by architecture (step §4.2; **verify** how it
  chooses). The page's upload names say "Linux x86_64" and "Linux arm64"; `PLAY.txt` repeats it.
- `ci_release.py itch-channels <version>` returns the table above, so the mapping lives in one place and is
  self-tested (an unknown target fails; `-rc` adds the suffix).
- Example. Tag `v0.1.0-rc.2` → four pushes to `*-rc` channels with `--userversion 0.1.0-rc.2`; tag `v0.1.0`
  → four pushes to `windows`, `mac`, `linux`, `linux-arm64` with `--userversion 0.1.0`.

### 3.3 butler (`DEP-39`)

- Download `https://github.com/itchio/butler/releases/download/v15.32.0/butler-linux-amd64.zip`, refuse it
  unless its SHA-256 equals the pin (§1.1), unpack, `butler -V` must print `v15.32.0` (**verify** the
  output format), then `butler login` is not run: butler reads `BUTLER_API_KEY` from the environment
  (step §4.2, itch docs `login.html`).
- The key is passed only to the `butler push` and `butler status` steps' `env:`, never to the job, and never
  echoed. No third-party action wraps butler (DEP-39's comparison).

### 3.4 The Environment `itch` (A-R10)

Created by the operator (QR-5), not by the PR:

- secret `BUTLER_API_KEY`; variable `ITCH_TARGET` (for example `<itch user>/mineworld`, Q-Rd-5);
- protection: required reviewer = the operator; "prevent self-review" off (the operator is the only
  maintainer); deployment branches and tags = **selected tags `v*`** only;
- no other workflow names the Environment.

### 3.5 Static guard (A-R10, cheap)

`ci_release.py check-secrets`, run in `fast` via `ci_layer.py`: fails if any workflow other than
`publish.yml` mentions `secrets.` (except `secrets.GITHUB_TOKEN`) or `environment:`, if any workflow uses
`pull_request_target`, or if `publish.yml` has any trigger other than `release` and `workflow_dispatch`.
Self-test with planted workflow texts. Example: a PR that adds `environment: itch` to `ci.yml` turns `fast`
red, naming the file and the line.

---

## 4. Files for people and for citation

### 4.1 `CITATION.cff`

`cff-version: 1.2.0`, `title: MineWorld`, `message` (how to cite), `type: software`, `authors` (Q-Rd-4),
`license: MIT`, `repository-code: https://github.com/yuema137/MineWorld`, `abstract` (the README's one-line
idea), `keywords`, `version`, `date-released`.

- `version` must equal the workspace version. `ci_release.py citation` checks it and is called by R-b's
  `verify-tag` on a tag run, so a tag cannot ship with a stale citation. Example: tag `v0.2.0` with
  `CITATION.cff` still saying `0.1.0` → `verify-tag` red, `CITATION.cff: version 0.1.0 ≠ 0.2.0`.
- `date-released` is set by hand in the release PR that bumps the version (§7).
- GitHub renders it as "Cite this repository" (step §4.3).

### 4.2 `CHANGELOG.md` (QR-11)

At the root. A non-README Markdown file, so a specification: complete and exact. Sections per version in
"Keep a Changelog" form: `Added`, `Changed`, `Fixed`, `Known issues`; every entry cites its PR (`#NNN`).
The first content is `## [0.1.0] — unreleased`, written from the merged PRs since Milestone E by the
implementing session, with "Saves may not load across 0.x versions" under Known issues. R-b's `notes`
command reads this version's section; R-d adds a self-test case for a missing section. Heading levels follow
`check_doc_headings.py`'s rules (**verify** at C3 that the checker accepts `## [0.1.0] — date`).

### 4.3 README (human, short)

One section, "Download and play", placed before "Quick start" (the source path stays for contributors):

- Before the first public release: one line, "🚧 Downloads for macOS, Windows and Linux arrive with the
  first release; until then, use the Quick start below." (the operator's "README ship early" practice).
- From the first publication: three lines — the GitHub Releases link (`/releases/latest`), the itch.io page,
  and "unsigned: see PLAY.txt for the first open" — and a "Cite" line with the DOI badge once Zenodo has
  minted one. The flip is a one-line follow-up made by the session that publishes, not by this PR.

### 4.4 Guide chapter 01, Step 1.1 (English and the zh-CN mirror)

Step 1.1 becomes two branches with the same outcome ("`mineworld` on your `PATH`"):

- **Download (new, first).** Prose: open the Releases page, download `MineWorld-<version>-<your target>`,
  unpack it into the home folder. Then shell blocks, qualified `operator-only` (F-Rd-4), per OS:
  macOS `xattr -dr com.apple.quarantine "$HOME/MineWorld-<version>-macos-universal"` (F-Rd-5) and
  `export PATH=".../runtime:$PATH"`; Linux the `export`; Windows `$env:PATH = "...\runtime;$env:PATH"`.
  Optional: `shasum -a 256 -c SHA256SUMS` / `sha256sum -c` / `Get-FileHash`, as `PLAY.txt` says.
- **Build from source (unchanged).** The existing blocks, executed by CI as today.
- The version in the text is written as the literal current version and updated by the release PR (§7);
  `guide.rs` does not check `http(s)` links.
- Until the first publication the download branch says "from the first release on" and stays
  `operator-only`, so CI behaviour is unchanged. The mirror carries the same blocks byte for byte (§7 of
  GUIDE_FORMAT) and introduces no requirement (CLAUDE.md §2.1 rule 5).

---

## 5. Operator actions (QR-5), outside the PR

Each is free of charge. In order:

1. itch.io: create the project page **as a draft** (not public) — price 0, optional donation (QR-13), AI
   disclosure ticked and described (AI-generated assets, per `NOTICE`), platforms ticked only for legs
   whose smoke passed. **Money note:** price 0 costs nothing; enabling donations needs itch.io's payout and
   tax-information setup and its revenue share applies to donations (**verify** the current default share);
   no money is spent unless someone donates.
2. itch.io: create an API key; GitHub: create the Environment `itch` with the secret, the variable
   `ITCH_TARGET`, the operator as required reviewer and tags `v*` only (§3.4).
3. Zenodo: log in with GitHub and switch the repository on — **after** Q-Rd-2 is settled (F-Rd-3).
4. Optional, free: GitHub "immutable releases" for the repository, so a published release's assets cannot
   be replaced (**verify** availability and wording in the repository settings).

---

## 6. Decision records

- `DEP-39` — butler, as step §12 words it, with v15.32.0, the GitHub asset URL, and the SHA-256 pin of §1.1
  (our pin; upstream publishes none).
- `ARC-79` note — the gate on tag shape (F-Rd-2), the separate verification job, candidate channels with
  `--hidden`, and the Zenodo choice of Q-Rd-2/Q-Rd-3.

---

## 7. Commit plan

### C0 — revision marker and base (Markdown only)

- Implementation: [ ] base recorded; §1 re-verified, including the butler digest by download.
- Validation: [ ] `check_doc_headings.py`. LLM review: [ ] anchors current.

### C1 — `publish.yml` and its script commands

- Scope: §3; `ci_release.py publish-gate`, `verify-assets`, `itch-channels`, `check-secrets` with
  self-tests; `ci_layer.py fast` line.
- Implementation: [ ] workflow; [ ] script; [ ] fast line.
- Validation: [ ] `python3 scripts/ci_release.py --self-test` (cases: `v0.1.0` publish; `v0.1.0-rc.1` with
  `prerelease=true`; `v0.1.0-rc.1` with `prerelease=false` refused; `promo-v2.3` → publish=false;
  channel table; a secret in `ci.yml` refused; `pull_request_target` refused);
  [ ] `check_ci_pins.py`; [ ] **A-R9 (dispatch, no publish):** R-b's dry run with `draft: true` makes draft
  `test-<run>`; dispatch `publish.yml` with that name → `verify` PASS; then replace one archive in the
  draft with a modified copy (`gh release upload --clobber`) and dispatch again → `verify` red naming the
  file at `sha256sum -c`; draft deleted; run URLs in the ledger. (On a draft, `--source-ref` is skipped,
  since a draft has no tag.)
  [ ] **A-R10:** the static guard's planted cases; `gh api repos/yuema137/MineWorld/environments/itch` output
  showing the reviewer and the `v*` tag policy (after the operator creates it; until then INCONCLUSIVE with
  the owner named).
- LLM review: [ ] the key reaches only two steps; [ ] `push` cannot run on dispatch; [ ] nothing published.
- Failure cases: an asset missing from `SHA256SUMS` → FAIL; an attestation missing → FAIL (never skip);
  butler download mismatch → FAIL before the key is read.

### C2 — `CITATION.cff`, `CHANGELOG.md`, `ci_release.py citation`, `DEP-39`, `ARC-79` note

- Validation: [ ] self-test for `citation` (equal / unequal version); [ ] `cffconvert --validate` or the
  CFF JSON schema check (**verify** a stdlib-only route; else a pinned `cffconvert` run once locally,
  output recorded, not added to CI); [ ] `check_decision_ids.py --write-index`; [ ] `check_doc_headings.py`.
- LLM review: [ ] every CHANGELOG entry cites a PR that exists and is merged.

### C3 — README section and guide Step 1.1 (both languages)

- Validation: [ ] the guide test on the PR (`fast` and the guide test in `test`; CI only, R-LOAD-1):
  the new blocks reported as `operator-only` pending, every other block unchanged; mirror parity PASS.
- LLM review: [ ] the README stays short; the guide's every step reads "Do this → You should see this".

### C4 — gate

- [ ] `fast` green on the exact head; the A-R9 runs on the final `publish.yml`; READY FOR OPERATOR REVIEW.

### After merge, at the operator's go-ahead (not part of the PR's acceptance; Q-Rd-1)

- [ ] Operator actions 1–3 done. [ ] Tag `v0.1.0-rc.1`; R-b drafts; operator publishes as pre-release;
  approves `itch`; hidden `*-rc` channels receive four builds; the itch app installs and opens the bundle on
  one OS; Zenodo result recorded (a DOI, or none, per Q-Rd-2). [ ] Then `v0.1.0` the same way; README and
  guide flipped by a one-line follow-up.

---

## 8. Test ownership (test rules §26)

```text
UNIT OWNERSHIP     ci_release.py self-tests: tag gate, channel table, asset/SUMS matching, secret guard,
                   citation version.
GATE 1 (real LLM)  N/A (no model).
GATE 2 (real run)  publish.yml dispatch runs against R-b's test drafts (A-R9); the live rehearsal after the
                   go-ahead (§7, after merge).
STATIC             check_ci_pins.py; check-secrets; check_decision_ids.py; check_doc_headings.py; the guide test.
CI IMPACT          fast gains two self-test lines; the guide test sees new operator-only blocks only.
```

---

## 9. Acceptance mapping

| Criterion | How R-d meets it | Evidence |
| --- | --- | --- |
| A-R9 itch receives only GitHub-published bytes | `verify` (SUMS + attestations) and the re-check in `push` | dispatch runs: PASS, then red on a modified asset |
| A-R10 no PR-reachable workflow can read `BUTLER_API_KEY` | triggers; the Environment's reviewer and `v*` policy; `check-secrets` in `fast` | self-test; `gh api` of the Environment |
| §13.5 checkpoint | split: dry evidence in the PR, live rehearsal after the go-ahead (Q-Rd-1) | ledger |
| QB-14 | all three OS bundles have channels and are named on the page | channel table §3.2 |

---

## 10. Cost and money

- Runner minutes: Linux only (`gate`, `verify`, `push`); a few minutes per release. No macOS runner. The
  A-R9 dry runs need R-b's `draft: true` run, which counts against this lane's two in-flight code-PR runs.
- **Money: none.** itch.io hosting at price 0, Zenodo, GitHub Releases, butler and attestations are free
  (step §4; GitHub docs read 2026-10-26: attestations on Free for public repositories). Flags: donations
  bring itch.io's revenue share and tax setup (operator, §5.1); signing stays deferred because it costs money
  (step §8, QR-3, QR-4).

---

## 11. Operator questions (each with a recommended answer)

- **Q-Rd-1 — Checkpoint timing.** Step §13.5's live checkpoint publishes a candidate, which QB-14/QR-12 hold
  until the go-ahead. **Recommended:** merge R-d on dry evidence (A-R9, A-R10 by dispatch and static
  guard); run the live rehearsal (`v0.1.0-rc.1` to hidden channels) right after the go-ahead.
- **Q-Rd-2 — Zenodo and pre-releases (promo videos, candidates).** If Zenodo archives pre-releases, every
  promo video and candidate mints a DOI. **Recommended:** before switching Zenodo on, test on
  `sandbox.zenodo.org` with a scratch public repository (free); if pre-releases are archived, publish future
  promo videos from a separate media repository and keep candidates as drafts tested from artifacts (QR-9).
- **Q-Rd-3 — Metadata file.** `CITATION.cff` only, or also `.zenodo.json` (which makes Zenodo ignore the
  CFF)? **Recommended: `CITATION.cff` only** — one source; add `.zenodo.json` only if the sandbox record lacks
  a field we need.
- **Q-Rd-4 — Authors in `CITATION.cff`.** **Recommended:** "Yue Ma" as the sole author, ORCID if you have
  one (give it, or none).
- **Q-Rd-5 — itch.io page address.** The `ITCH_TARGET` (`<user>/<slug>`). **Recommended:** `<your itch
  user>/mineworld`; you create it as a draft page.
- **Q-Rd-6 — Guide download branch.** **Recommended:** `operator-only` blocks in Step 1.1, with CI coverage
  from the onboarding layer B (it unpacks `release.yml`'s artifact on a runner without Rust or Godot).
- **Q-Rd-7 — Candidate channels.** **Recommended:** separate `*-rc` channels created with `--hidden`, so a
  candidate never appears on the page and never replaces a release build.
- **Q-Rd-8 — Donations.** QR-13 ruled "price 0 with optional donation". **Recommended:** keep it, knowing it
  needs itch.io's payout/tax setup before money can arrive; no spending is involved.

---

## 12. Execution contract

Left unfilled on purpose; the operator freezes this design and the coordinator fills the contract before
any code.

```text
PROJECT / PR:              (unfilled)
PRIMARY DESIGN DOC:        (unfilled)
IMPLEMENTATION BASE:       (unfilled)
APPROVED SCOPE:            (unfilled)
FROZEN INVARIANTS:         (unfilled)
AUTONOMOUS ACTIONS:        (unfilled)
STOP CONDITIONS:           (unfilled)
VALIDATION BUDGET:         (unfilled)
MERGE AUTHORITY:           (unfilled)
```

---

## 13. Ledger

| Date | Commit / run | Event |
| --- | --- | --- |
| 2026-10-26 | — | revision 1 drafted from origin/main 0b2faf2f; QB-14 (PR #497) reflected |
