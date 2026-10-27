# PR S23 R-b — The CI release workflow: tag, smoke, checksums, provenance, draft release

## DESIGN FROZEN 2026-10-27 (operator approved in the primary session: every question answered as recommended)

**Ruled 2026-10-27 (operator, primary session):** every question below is answered as recommended.

```text
Design revision:        revision 1 (2026-10-26), frozen 2026-10-27
Author:                 design sub-agent (Opus), worktree /Users/yuema137/mineworld-worktrees/design-s23-rb-rd,
                        branch docs/pr-design-s23-rb-rd, planning base origin/main @ 0b2faf2f
Builds on:              PR #498 (draft, branch release-dry-run/workflow, head 5cb3c313): "R-b part 1", the
                        dispatch dry run. This document specifies the remainder ("R-b part 2")
Approved by / evidence: the operator, 2026-10-27 (primary session; relayed by the coordinator)
Implementation base:    PR #498's head 469ffa8c plus origin/main 315a50a9 merged in (stacked on #498)
Start condition:        met: frozen; R-c C5 landed (#517); ARC-79 on main; #498 still open (stacked)
Execution contract:     §11, filled 2026-10-27
Implementer:            implementation session, worktree impl-s23-rb, branch mvp0/pr-s23-rb
Lifecycle:              FROZEN; implementation in progress
```

**Effort:** `mvp0` · **Step:** [`step-23-release.md`](step-23-release.md) §§3, 5, 7, 9, 10, 13.4, 14 (A-R2,
A-R7, A-R8), 15 (QR rulings), 16.2 · **Decision records:** writes `DEP-40` (SBOM) and `DEP-42` (provenance), an
`ARC-79` note (the record itself is written by PR #497), an `ARC-48` note · **Relates to:** `ARC-78`, `ARC-80`
(and its S23-RT amendment), `DEP-38`, `DEP-41`, `DEP-45`, `ARC-49` (AC-8), [`pr-s23-rt-shared-runtime.md`](pr-s23-rt-shared-runtime.md),
the onboarding "layer B" design (pr-25g, another agent) · **Next:** R-d, [`pr-s23-rd-publish.md`](pr-s23-rd-publish.md)

Every statement about an external service carries the date it was read and its source. A statement this
session could not confirm from a primary source is marked **verify**; the implementing session confirms it
before relying on it.

---

## 0. The problem in one paragraph

Step 23 promises that a version tag `v*` on `main` builds a bundle for every target, smoke-tests each bundle
on its own operating system, and leaves a **draft** GitHub release that only the operator publishes (§1.3,
§5). PR #498 builds and probes three of the four bundles on a manual dispatch, and publishes nothing. What is
still missing is everything that makes a run a *release*: the tag trigger and its guard (`verify-tag`), the
fourth target (`linux-arm64`), the older-macOS leg (`macos-15`, QR-2), the smoke steps that #498 does not run
(the AC-8 digest comparison, the launcher, the clean exit), `SHA256SUMS`, the SBOM, the attestations, the
draft release with its notes, and the reproducibility measurement. This PR adds them to the same
`.github/workflows/release.yml`, so one file owns the release build.

Example of the end state. The operator pushes `v0.1.0`. Forty to sixty minutes later the repository has a
draft release "MineWorld 0.1.0" with four archives, `SHA256SUMS`, one SBOM, and a provenance attestation for
each file. Nobody outside can see it until the operator presses "Publish".

---

## 1. Audit (origin/main @ 0b2faf2f and PR #498 head 5cb3c313, read 2026-10-26)

### 1.1 What exists

| Fact | Evidence |
| --- | --- |
| Workspace version `0.1.0`; no `v*` tag exists. One release exists, `promo-v2.2` (pre-release, intro video), created 2026-10-21 | `Cargo.toml:36`; `git tag` → `promo-v2.2`; `gh release list` |
| `main` has two workflows, `ci.yml` and `nightly.yml`; no `release.yml`, no `publish.yml` | `ls .github/workflows` |
| `ci.yml` runs `fast` and `test` on every push to `main` (`push: branches: [main, "scratch/**"]`); both are the required checks by name (`ARC-48`) | `.github/workflows/ci.yml:35–37, 85–86, 175–176`; `docs/decisions/ARC-48.md` |
| `scripts/package.py` has `build`, `export`, `assemble`, `budget`, `probe`, `--self-test`; no `smoke` subcommand | `scripts/package.py:8–19, 724–751` |
| `assemble` takes `--server FILE` (a server already built for the target) and honours `SOURCE_DATE_EPOCH`; archives have sorted entries, one mtime, normalized modes | `scripts/package.py:457–476, 418–419, 501–523` |
| `build` adds `--remap-path-prefix=<ROOT>=mineworld` and joins two triples with `lipo` on macOS | `scripts/package.py:210–227` |
| Linux targets are musl (`x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`), QR-6 (a) | `scripts/package.py:83–88` |
| The bundle ships three worlds: `social-cafe`, `market-town`, `bodies-yard` (not `lakeside`) | `scripts/package.py:56`; `ls worlds` → four |
| **The bundle holds no launcher.** `layout()` copies `PLAY.txt`, `LICENSE`, `NOTICE`, the server, worlds, packs and shared files; nothing from `tools/launch`. R-c's C5 ("launcher binaries, `.app` wrappers, `.desktop` files in `assemble`; `smoke` running §9.1 steps 1–6 through the launcher") is unchecked | `scripts/package.py:436–454`; `step-23-release.md` §19.4 C5 `[ ]` |
| `packaging/PLAY.txt` tells players to double-click "MineWorld 2D", "MineWorld 3D", "MineWorld 2D + 3D", which a bundle built today does not contain | `packaging/PLAY.txt:10–16` |
| `probe` checks the bundle's file digest against `BUNDLE.toml`, then starts the bundle's server and the shared runtime directly (not the launcher) for each client, headless, and requires exit 0 plus the verdict line; for 2D it also requires the Presentation Pack to load from the bundle (A-R1) | `scripts/package.py:591–636` |
| AC-8's instrument is `scripts/ci_parity.py`: `record --binary B` runs `validate`, 300 days in memory and 30 days saved (seed 7) for **every world under the checkout's `worlds/`**, with `--packs entities --packs presentation/mineworld-default`, from the current directory; `compare` requires G-3 (a Darwin arm64, a containerized Linux x86_64 and a Windows x86_64 record) | `scripts/ci_parity.py:66–76, 130–134, 364–404, 522–537` |
| AC-8 records are made only on a milestone dispatch or a `scratch/*-scenario` push, never on a push to `main` (ARC-154) — so "the AC-8 record of the same commit" (step §9.1 step 3) usually does not exist for a tagged commit | `.github/workflows/ci.yml:489–503, 633–644` |
| AC-8's macOS and Windows legs build `--release` (`ci_layer.py parity`), not the debug profile step §9.1 names | `scripts/ci_layer.py:103–106` |
| The pinned Godot exists for Linux x86_64, macOS universal and Windows x86_64 only; no Linux arm64 editor pin | `scripts/ci_godot.py:55–75` (`ASSETS`) |
| Runner labels in use: `ubuntu-24.04`, `ubuntu-24.04-arm`, `macos-26`, `windows-2025`; `macos-15` is named in step §5.2 (QR-2) | `.github/workflows/ci.yml:291, 419, 540, 576` |
| Every action is pinned by commit SHA, checked by `scripts/check_ci_pins.py` (`DEP-19`) | `ci.yml`; #498's diff |
| No `.github/release-notes.md`, `CHANGELOG.md`, `CITATION.cff`, `.zenodo.json` | `ls` |
| `ARC-79` (release channels) does not exist on `main`; PR #497 writes it (QB-14 note included). `DEP-39`, `DEP-40`, `DEP-42` do not exist | `ls docs/decisions`; `gh pr view 497` |

### 1.2 What PR #498 adds (head 5cb3c313) — R-b part 1

- `.github/workflows/release.yml`, name "Release dry run". Triggers: `workflow_dispatch`, and `push` to
  `release-dry-run/**` (a workflow is dispatchable only once it is on `main`, F-13c-8). `permissions:
  contents: read`. `concurrency: release-dry-run-<ref>`, cancel in progress.
- One job `bundle (<target>)`, a matrix of three: `linux-x86_64` on `ubuntu-24.04` (musl), `windows-x86_64`
  on `windows-2025`, `macos-universal` on `macos-26`. `timeout-minutes: 150`. Each leg: checkout with LFS,
  Rust triples, `musl-tools` on Linux, `./.github/actions/godot` (DEP-45), templates cache keyed on
  `hashFiles('scripts/ci_godot.py')`, `ci_godot.py templates` (DEP-41, new), `cargo install cargo-about
  --version 0.9.2` (DEP-38), `package.py assemble --target <T>`, unpack into `$RUNNER_TEMP/MineWorld 世界/`
  (§9.1 step 1), `package.py probe <bundle>`, a step summary, and `actions/upload-artifact` of the archive.
- **The artifact interface** (see §4.3): name `MineWorld-<target>`; content the one archive
  `MineWorld-<version>-<target>.<zip|tar.gz>`; `retention-days: 7`; `compression-level: 0`;
  `if-no-files-found: error`.
- `scripts/ci_godot.py templates`: downloads `Godot_v4.7.2-stable_export_templates.tpz`, refuses it unless
  its SHA-512 equals the pin, keeps only this OS's templates (`TEMPLATE_PREFIX`: `linux_`, `macos`,
  `windows_`) in Godot's per-user templates folder, marker file `PINNED-SHA512`.
- Not in #498, by its own statement: tag trigger, `verify-tag`, SBOM, attestation, draft release, secret,
  `linux-arm64`.

### 1.3 #498's dry-run run 40447534966 (read 2026-10-26, completed)

| Leg | Outcome | Evidence |
| --- | --- | --- |
| `linux-x86_64` | **FAIL** at "Unpack the archive and probe the bundle", exit code 2, with no output, 11 min after start. Assembly passed: archive 305.1 MB (hard 400, goal 250, "over goal"); 2D pack and 2D Presentation Pack 13.4 MB; `3d.pck` 240.7 MB (hard 250 — 9.3 MB of headroom) | job 122002263075 log |
| `macos-universal` | **FAIL** at the same step, exit code 1 (BSD `ls`), with no output, 17.5 min after start. Assembly passed: archive 348.8 MB (hard 400); 2D 13.4 MB; `3d.pck` 240.6 MB | job log of run 40447534966 |
| `windows-x86_64` | **FAIL** at the same step, exit code 2, with no output, 17.7 min after start. Assembly passed (so the Windows checkout and export worked, R-SET-10 included): archive 313.1 MB (hard 400); 2D 13.4 MB; `3d.pck` 240.7 MB | job log of run 40447534966 |

Cause of both failures (diagnosis from the logs, not yet confirmed by a fix): the step runs under
`bash -e -o pipefail`, and its first line is `archive=$(ls <glob>.zip <glob>.tar.gz 2>/dev/null | head -n 1)`.
One of the two globs always matches nothing (Linux makes `.tar.gz`, macOS and Windows `.zip`), so `ls`
exits non-zero (2 on GNU, 1 on BSD); `pipefail` makes that the pipeline's status, and `-e` stops the step
before its own error message can print. No probe ran on any leg yet. This is a workflow bug in #498, not a
product defect; #498's own session owns the fix (for example, choose the extension from the target instead
of listing both).

What the run already shows: on its own OS, each of the three targets assembles inside the budget, in 11 min
(Linux), 17.5 min (macOS) and 17.7 min (Windows), with cold template caches.

### 1.4 Findings (F-Rb-n)

- **F-Rb-1 — the launcher is not in the bundle (R-c C5 is outstanding).** Smoke steps 4–6 of §9.1 run
  through the launcher, and `PLAY.txt` names three entry points the bundle lacks. R-b cannot smoke-test what
  is not assembled. Q-Rb-2.
- **F-Rb-2 — #498 changes the job shape of step §5.2 (needs a ruling).** See §2. In short: §5.2 builds the
  server per OS, exports and assembles once on Linux, and smoke-tests on fresh runners; #498 builds,
  exports, assembles and probes all on the target's own runner. Q-Rb-1.
- **F-Rb-3 — the AC-8 comparison of §9.1 step 3 has no reference to compare against.** No AC-8 record is
  made for an ordinary `main` commit, `ci_parity.py compare` requires G-3's three platforms (a containerized
  Linux record among them), and `record` runs the checkout's four worlds with `--packs entities`, which the
  bundle does not ship. §3.5 specifies a reference made in the same run and a `match` mode. Q-Rb-3.
- **F-Rb-4 — step §9.1 says the reference is "debug-profile"; AC-8's native legs build `--release`**
  (`ci_layer.py:104`). The comparison bites either way, because the bundle's server differs in libc (musl),
  linking (universal, MSVC) and LTO settings from a glibc build. Bounded wording correction, recorded in
  §3.5.
- **F-Rb-5 — `.github/release-notes.md` would be a specification by the documentation law.** CLAUDE.md §2.1
  makes every `.md` that is not a `README.md` or under `docs/guide/` a specification held to the spec
  standard, and `check_doc_headings.py` reads Markdown. A fill-in template is not a specification. Q-Rb-4.
- **F-Rb-6 — Linux arm64 needs no Godot editor on arm64.** The bundle runs only the export *template*.
  The Linux x86_64 editor exports a `Linux arm64` preset from the templates archive, which #498 already
  keeps (`linux_` prefix covers both architectures). So the arm64 bundle can be assembled on
  `ubuntu-24.04` with a server built on `ubuntu-24.04-arm` (`assemble --server`), and smoke-tested on
  `ubuntu-24.04-arm`. #498's "waits for an arm64 Godot pin" is not needed (§3.3).
- **F-Rb-7 — `release: published` also fires for the existing `promo-v*` pre-releases.** Relevant to R-d,
  recorded there (F-Rd-2). R-b's `verify-tag` already refuses non-`v<version>` tags.
- **F-Rb-8 — the 3D pack ships `res://tools/*_probe.gd` and other `tools/` scripts.** `forbidden()` refuses
  only `tools/blender`, evidence folders, `.md` and `.env` (`package.py:134–146`), while ARC-80 and step §3.1
  say `tools/` is not shipped. Not GPL, so not an A-R3 failure as A-R3 is worded, but a mismatch between the
  record and the filter. Owner: R-a / the 3D lane; R-b only reports it (Q-Rb-7).
- **F-Rb-9 — `3d.pck` has 9.3 MB of headroom** under its 250 MB hard limit (§1.3). A release run will turn
  red when the cast grows; that is the budget doing its job (A-R4), and the 3D lane owns the fix.

---

## 2. Job shape: frozen step §5.2 against #498 (F-Rb-2)

| Aspect | Step §5.2 (frozen) | #498 | This design recommends |
| --- | --- | --- | --- |
| Server build | per-OS `server` matrix | inside each target's job (`assemble` calls `build`) | **#498's**, plus arm64 built natively (§3.3) |
| Godot export | once, on `ubuntu-24.04`, for every target | per target, on that target's OS | **#498's** for macOS, Windows, Linux x86_64; arm64 exported on `ubuntu-24.04` (F-Rb-6) |
| Templates download | once (1.28 GB), cached | per OS, cached per OS | #498's (a consequence of the row above) |
| Assembly | once, on Linux, per target | per target, on that target's OS | #498's |
| Smoke | separate jobs on fresh runners of each OS (+ `macos-15`, + arm64) | the same runner that built the bundle, with Rust, Godot and the checkout present | **step §5.2's**: separate smoke jobs that download the artifact on a fresh runner |
| Trigger | `push: tags: ['v*']` + `workflow_dispatch` | `workflow_dispatch` + `push: release-dry-run/**` | both: tags, dispatch, and #498's branch push for pre-merge evidence |

Why keep #498's per-OS assembly. It is already written and (once its probe-step bug is fixed) proven on
real runners; it exercises the same `package.py assemble` a developer runs on each OS (R-a's checkpoint ran
it on macOS); it avoids building the macOS `.app` signature from Linux; and the server must be built on
macOS and Windows anyway, so export-once saves Linux minutes, not macOS minutes. Its cost: three template
downloads instead of one (cached), and the Windows leg depends on Windows checking out the shared-module
symlinks (`R-SET-10`) — which #498's Windows leg settled: it assembled a bundle inside the budget (§1.3);
whether the exported clients then run is shown only once the probe runs.

Why still separate the smoke. A smoke test on the build runner can pass by accident: Godot is on `PATH`,
the checkout's `worlds/` and `presentation/` sit nearby, and Rust is installed. A player has none of that.
A fresh runner that only downloads the archive is the honest test (§9.1 "as a player would"), and it is the
same shape the onboarding layer B uses (§4.3). It also gives `macos-15` and `ubuntu-24.04-arm` their legs
without building there.

---

## 3. Design of R-b part 2

### 3.1 Triggers and permissions

```yaml
on:
  push:
    tags: ["v*"]
    branches: ["release-dry-run/**"]     # kept from #498: pre-merge evidence
  workflow_dispatch:
    inputs:
      draft:
        description: "Also create a draft release named test-<run id> (deleted by hand afterwards)"
        type: boolean
        default: false
permissions:
  contents: read                          # workflow default; only `draft` raises it
concurrency:
  group: release-${{ github.ref }}
  cancel-in-progress: ${{ !startsWith(github.ref, 'refs/tags/') }}   # never cancel a tag run
```

- `name:` changes from "Release dry run" to "Release". Job names are new and none is a required check, so
  `ci.yml`'s `fast`/`test` are untouched (ARC-48's "check names are an interface").
- No `pull_request`, no `pull_request_target`, no secret. The `draft` job alone has `contents: write`,
  `id-token: write`, `attestations: write`.

### 3.2 Jobs

```text
verify-tag      ubuntu-24.04   tag run: the tag equals "v" + workspace version, optionally "-rc.N";
                               the tagged commit is an ancestor of origin/main; `fast` and `test` are
                               green on that commit. Dispatch / branch push: computes the would-be tag
                               "v<version>" and runs the same version check, skips the ancestry and
                               check-run tests, and marks the run "dry". Outputs: tag, version, dry, epoch
bundle          matrix (#498)  needs verify-tag; macos-universal (macos-26), windows-x86_64 (windows-2025),
                               linux-x86_64 (ubuntu-24.04); SOURCE_DATE_EPOCH = epoch; package.py
                               assemble; per-archive .sha256 file; uploads MineWorld-<target> (§4.3).
                               The probe step of #498 stays as a fast in-place check
server-arm64    ubuntu-24.04-arm  cargo build --release --locked --target aarch64-unknown-linux-musl via
                               `package.py build --target linux-arm64`; uploads the server binary
bundle-arm64    ubuntu-24.04   needs server-arm64; package.py assemble --target linux-arm64
                               --server <downloaded>; uploads MineWorld-linux-arm64
reference       ubuntu-24.04   the AC-8 reference record of this commit (§3.5); uploads release-ac8-reference
smoke           matrix         needs bundle, bundle-arm64; fresh runner, no Rust, no Godot (§3.4):
                                 macos-26 (macos-universal), macos-15 (macos-universal),
                                 windows-2025 (windows-x86_64), ubuntu-24.04 (linux-x86_64),
                                 ubuntu-24.04-arm (linux-arm64)
                               uploads release-ac8-<leg> and the smoke log
parity          ubuntu-24.04   needs reference, smoke (always()); ci_parity.py match reference + every
                               leg's record; red if any is missing or differs (A-R8)
draft           ubuntu-24.04   needs every job above green; tag run, or dispatch with draft: true.
                               SHA256SUMS, SBOM, attestations, gh release create --draft (§3.6–§3.8)
```

`fail-fast: false` on both matrices, so one red leg still reports the others. `draft` uses
`needs:` on all of them without `always()`, so a red or skipped leg skips `draft` (A-R7).

### 3.3 `verify-tag` and Linux arm64

- **Logic lives in a script, not in YAML.** `scripts/ci_release.py verify-tag --tag T --commit C
  [--dry]` (standard library, as the other `scripts/*.py`). It reads the version with the same parser as
  `package.py workspace_version()` (imported, not copied), checks the tag against
  `^v(?P<v>\d+\.\d+\.\d+)(?P<rc>-rc\.\d+)?$` and `v == version`, checks ancestry with
  `git merge-base --is-ancestor C origin/main` (checkout with `fetch-depth: 0`), and reads the check runs of
  C with `gh api repos/{repo}/commits/C/check-runs` for names `fast` and `test`, requiring
  `conclusion == success` for both. It prints one line per check and exits 1 naming the first failure.
  `--self-test` covers: matching tag; `-rc.2`; `v9.9.9` against `0.1.0`; `v0.1`; `0.1.0` without `v`;
  `v0.1.0-beta`; a missing check; a failed check (fed from a recorded JSON fixture, never the network).
- Example: tag `v0.1.0-rc.1` on `main`'s head with `fast` and `test` green → PASS, `prerelease=true`.
  Tag `v9.9.9` → `verify-tag: v9.9.9 is not v0.1.0 (Cargo.toml [workspace.package] version)`, exit 1, and
  nothing downstream runs (A-R2).
- `epoch` output: the tagged commit's committer time (`git show -s --format=%ct C`), passed as
  `SOURCE_DATE_EPOCH` to every assembly (§5.3).
- **Linux arm64 (QR-2).** Built as F-Rb-6 says. `scripts/ci_godot.py` needs no arm64 editor pin. The
  `Linux arm64` export preset must exist in `clients/runtime/export_presets.cfg` and the two clients'
  presets (R-a / S23-RT wrote four targets, `pr-s23-rt-shared-runtime.md` §4.1; **verify** at C1). R-a's
  finding that the arm64 export may need `import_etc2_astc` is handled by `package.py`'s
  `etc2_astc_for` (D-Ra-6); the arm64 smoke leg is the evidence.

### 3.4 The smoke legs (step §9.1, one job per leg)

Each leg checks out the repository **without LFS and without building anything** (it needs only
`scripts/` and `worlds/`-free Python: `package.py`, `ci_parity.py`), uses the runner's own Python 3, and
downloads `MineWorld-<target>`. It installs neither Rust nor Godot. Steps, numbered as §9.1:

1. **Unpack as a player would** into `$RUNNER_TEMP/MineWorld 世界/` (a space and non-ASCII), with the
   OS's own tool: `tar -xzf` on Linux, `ditto -x -k` on macOS (what Finder uses; keeps the `.app`s'
   signatures and modes), Windows' `tar.exe` (bsdtar). Example of what this catches: a `.zip` whose mode bits
   lose the executable flag on the macOS `.app`'s inner binary.
2. **Checksums and file list.** The archive's SHA-256 equals the `.sha256` file the bundle job wrote, and
   `package.py probe`'s first check (every file against `BUNDLE.toml`'s `files_sha256`) passes.
3. **Server alone, AC-8.** `ci_parity.py record --binary <bundle>/runtime/mineworld[.exe] --root
   <bundle>/runtime --out release-ac8-<leg>.txt` (§3.5). The comparison happens in `parity`, so the leg
   only records.
4. **Launcher plus 2D, headless** — `package.py smoke <bundle> --mode 2d` (R-c C5): the launcher's
   `--smoke=2d` hosts `market-town` on a scratch save, runs the 2D client with `--drive`, exits with the
   client's code.
5. **Launcher plus 3D, headless** — `package.py smoke <bundle> --mode 3d`: the slice link probe; the verdict
   line `all link checks pass` is required (F-R6); missing is FAIL. Also `--mode both` (QR-10, `AC-15`).
6. **Clean exit** — `package.py smoke` checks the launcher log for the server's shutdown lines, that no
   process started by the run remains (by the PIDs the launcher logged), and that the per-user scratch
   folder is gone (`check_scratch.py`'s rule, `DEP-29`).
7. **Windowed frame (macOS legs, best effort)** — out of scope for part 2: recorded as INCONCLUSIVE with
   an owner, never a blocker (§9.1 already says so). Rationale: no frame recipe for the shared runtime exists
   on `main` yet.

Until R-c C5 lands, steps 4–6 cannot run; `package.py probe` (server + runtime, no launcher) stands in, and
the leg says so in its summary. A tag run with the stand-in is refused by `draft` (the `smoke` job exports
`launcher=absent`, and `draft` fails on it), so no release ships without the launcher being tested.

### 3.5 The AC-8 comparison (A-R8, F-Rb-3, F-Rb-4)

Two small additions to `scripts/ci_parity.py`, both inside AC-8's instrument so one implementation hashes
everything (ARC-49):

- **`record ... --root DIR`.** Worlds are enumerated under `DIR/worlds/` instead of the checkout's, every
  command runs with `DIR` as its working directory, and `PACK_ROOTS` keeps only the roots that exist under
  `DIR` (the bundle has `presentation/mineworld-default`, not `entities`). The record's `[source]` gains
  `root bundle` when given, so a bundle record can never be mistaken for a checkout record. Without
  `--root`, a record is byte-identical to today's (I-13c-2's rule, extended).
- **`match REFERENCE RECORD [RECORD ...]`.** Passes when G-1 holds (each file well-formed), G-2 holds (one
  commit, one profile, the same world list in every file), and G-5 holds (every key of every world equal to
  the reference). It does not apply G-3 (no platform set is required: the release legs are the platforms)
  or G-4 (the release build uses `rust-toolchain.toml` by construction). Output and failures in
  `compare`'s format, plus the step summary.
- **The reference.** Job `reference` on `ubuntu-24.04`: `cargo build --release --locked -p mineworld-cli`
  (native glibc, the profile AC-8's native legs use, `ci_layer.py:104`), then copy the bundle's three worlds
  and `presentation/mineworld-default` as tracked into a scratch folder laid out like `runtime/` (by the
  same `package.py` list, imported), and `record --binary target/release/mineworld --root <scratch>`. The
  reference is a different build path from every bundle (glibc vs musl, single-arch vs universal, the
  checkout's copy vs the archive's), so it is not self-referential (test rules §25).
- Example of what turns red: the musl allocator changing an iteration order in a world's simulation. The
  `linux-x86_64` record's `summary-300` differs from the reference; `parity` prints
  `G-5 world market-town differs: reference = macos-universal = … ≠ linux-x86_64` and the first differing
  line, and `draft` is skipped.
- **Cost.** One record is `validate` + 300 days + 30 days saved for three worlds. **Measure** in C3; if a
  leg exceeds 15 minutes, the operator may rule a shorter release profile (Q-Rb-3).

### 3.6 `SHA256SUMS`

Computed in `draft` over every release asset (four archives and the SBOM), in `sha256sum` format, sorted by
name; uploaded as an asset itself. Each `bundle` job's `.sha256` file is compared with the recomputed line
first: a mismatch between the build runner and the release job fails the job (the artifact changed in
transit).

### 3.7 SBOM (`DEP-40`) and attestations (`DEP-42`)

- **SBOM.** `anchore/sbom-action` v0.24.3 (`66cbf4bc1f1c0d2edc94016e65bc221b6bb0ad6c`, latest release,
  `gh api`, read 2026-10-26) on the checkout at the tag, `format: spdx-json`,
  `output-file: MineWorld-<version>-sbom.spdx.json`, `upload-artifact: false`. Its scope is the source
  tree's lock files (`Cargo.lock`, `uv.lock`); the release notes say it covers the Rust and Python graphs
  only, and that Godot's libraries are listed in `LICENSES/godot/COPYRIGHT.txt` (step §5.4).
- **Provenance.** `actions/attest` v4.2.2 (`1e69f48acb82d1966a394da916b4c1698aa569d6`, latest release,
  read 2026-10-26) with `subject-path` = the four archives and `SHA256SUMS`; a second call with
  `sbom-path` binds the SBOM to the four archives. Permissions `id-token: write`, `contents: read`,
  `attestations: write` (GitHub docs, "Using artifact attestations", read 2026-10-26: the examples use
  `actions/attest@v4` with these three permissions, and `sbom-path` on the same action).
- **Availability on Free.** GitHub's documentation says artifact attestations are available for public
  repositories on all current plans, Free included; private repositories need Enterprise Cloud (docs.github.com
  "Using artifact attestations to establish provenance for builds", via search, read 2026-10-26).
  Public repositories sign through the Sigstore Public Good Instance and are written to a public transparency
  log (docs.github.com "Artifact attestations", read 2026-10-26). MineWorld is public, so this holds; the
  first dispatch with `draft: true` is the evidence (`gh attestation verify`).
- **Verification text** for the release notes and `PLAY.txt`:
  `gh attestation verify <file> -R yuema137/MineWorld`, and for the SBOM
  `gh attestation verify <file> -R yuema137/MineWorld --predicate-type https://spdx.dev/Document/v2.3`
  (docs, read 2026-10-26).
- Attestations are made for dry runs with `draft: true` too (that is how they are tested). A dry run's
  attestations name a non-`v*` ref, so a verifier who checks `--source-ref refs/tags/v0.1.0` never accepts
  them. **Verify** the flag name `--source-ref` in the `gh` version on the runner.

### 3.8 The `draft` job and the release notes

- Tag run: `gh release create "$TAG" --draft --verify-tag --title "MineWorld $VERSION" --notes-file
  notes.md [--prerelease when -rc.N] <assets>`. `--verify-tag` refuses to create a tag that does not exist.
- Dispatch with `draft: true`: `gh release create "test-$RUN_ID" --draft --target "$SHA" --title "TEST
  $RUN_ID — not a release" ...`. A draft with a new tag name creates no git tag until it is published
  (**verify** on the first run: `git ls-remote --tags origin test-*` empty afterwards). The implementing
  session deletes it with `gh release delete test-<run> --yes` and records that in the ledger. Nothing is
  ever published by a workflow (ARC-79).
- Notes: rendered by `ci_release.py notes` from the template (Q-Rb-4 decides its path) with `{version}`,
  `{sha}`, `{run_url}`, the per-target lines `{target} — {size} MB`, and "What's new" / "Known issues" cut
  from `CHANGELOG.md`'s section for this version. Before R-d adds `CHANGELOG.md`, the two sections read
  "See the commit history" — the dry run must not fail for a file R-d owns. A tag run with no `CHANGELOG.md`
  section for its version fails (`notes: CHANGELOG.md has no section 0.1.0`), so the first real tag needs R-d.
- Template text: step §10.3's eight headings, unchanged. The "Known issues" section always carries "saves
  may not load across 0.x".

### 3.9 Reproducibility measurement (step §5.3)

Not a gate. In C4 the implementing session runs two dispatches on one commit and compares per-file SHA-256
of each unpacked bundle with a small `ci_release.py repro A B` (lists files that differ). The list (expected:
the `.pck` files' UIDs or order, the MSVC link timestamp in `.exe`s, `BUNDLE.toml`'s `built` if it ever reads
the clock) goes into §12's ledger as "known non-reproducible", each with its cause where found. A file that
differs for an unknown reason is a finding, not a failure.

### 3.10 ARC-48 note and decision records

- `ARC-48` note: `release.yml` exists; its jobs are not PR checks and need no names in branch protection; it
  holds no secret; only `draft` can write (contents, attestations), only on a tag or an explicit dispatch
  input; `publish.yml` (R-d) is the only workflow that can read a secret.
- `DEP-40` (SBOM) and `DEP-42` (provenance) as step §12 words them, with the versions and SHAs above.
- `ARC-79` note (the record is PR #497's): the dispatch dry run's push trigger on `release-dry-run/**`, and
  the artifact interface of §4.3.

---

## 4. Interfaces and invariants

### 4.1 Frozen invariants (proposed)

- I-Rb-1. No workflow publishes a release; `draft` creates drafts only (ARC-79).
- I-Rb-2. `release.yml` reads no secret and has no `pull_request*` trigger.
- I-Rb-3. A red or skipped `verify-tag`, `bundle*`, `smoke`, `reference` or `parity` job means no draft.
- I-Rb-4. Every downloaded tool is pinned by version and checksum; every action by commit SHA.
- I-Rb-5. Packaging rules stay in `scripts/package.py`; the workflow only installs tools and calls scripts.
- I-Rb-6. `ci.yml`'s jobs, names and triggers are unchanged.

### 4.2 Files

| File | Change |
| --- | --- |
| `.github/workflows/release.yml` | §3 (on top of #498) |
| `scripts/ci_release.py` (new) | `verify-tag`, `notes`, `sums`, `repro`, `--self-test` |
| `scripts/ci_parity.py` | `record --root`, `match` (§3.5) and their self-test cases |
| `scripts/ci_layer.py` | `fast` gains `ci_release.py --self-test` (one line) |
| release-notes template | path per Q-Rb-4 |
| `docs/decisions/DEP-40.md`, `DEP-42.md`; notes in `ARC-48.md`, `ARC-79.md`; `INDEX.md` regenerated | §3.10 |
| `packaging/PLAY.txt` | the "Verify (optional)" lines name `gh attestation verify` (text only) |

### 4.3 The artifact interface for consumers (onboarding layer B, pr-25g)

Layer B runs on a runner with no Rust and no Godot and consumes the bundle. This is a stable interface;
changing it is a change to this design.

- **Workflow:** `.github/workflows/release.yml`, workflow name "Release".
- **Artifact name:** `MineWorld-<target>`, `<target>` ∈ `macos-universal`, `windows-x86_64`,
  `linux-x86_64`, `linux-arm64`.
- **Content:** exactly one file, `MineWorld-<version>-<target>.zip` (macOS, Windows) or `.tar.gz` (Linux),
  plus `MineWorld-<version>-<target>.<ext>.sha256` (added by part 2). Unpacked, one top folder
  `MineWorld-<version>-<target>/` (ARC-80).
- **Retention:** 7 days for dispatch and branch runs (#498); 90 days for tag runs (the repository default,
  **verify** the repository's setting). Q-Rb-5.
- **Finding a run:** the latest successful run of `release.yml` on the wanted ref, for example
  `gh run list -w release.yml -b main -s success -L 1`, then `gh run download <id> -n MineWorld-<target>`.
- **Upload settings:** `compression-level: 0` (the archive is already compressed).

---

## 5. Commit plan

Each commit tracks implementation, deterministic validation and LLM review separately. `[x]` needs the
work and its evidence; nothing is checked in a draft.

### C0 — revision marker and implementation base (Markdown only)

- Implementation: [ ] record the base (origin/main after #498, or #498's head) and re-verify §1's anchors.
- Validation: [ ] `python3 scripts/check_doc_headings.py`.
- LLM review: [ ] anchors match the base.

### C1 — `verify-tag` and the triggers

- Goal: A-R2. Scope: `scripts/ci_release.py verify-tag` and `--self-test`; `release.yml` triggers,
  `concurrency`, job `verify-tag`, `bundle` `needs: verify-tag` and `SOURCE_DATE_EPOCH`; `ci_layer.py fast`.
  Non-goals: smoke, draft.
- Implementation: [ ] script; [ ] YAML; [ ] fast-layer line.
- Validation: [ ] `python3 scripts/ci_release.py --self-test` (cases of §3.3; counts and time recorded);
  [ ] `python3 scripts/check_ci_pins.py`; [ ] a branch-push dry run shows `verify-tag` PASS "dry";
  [ ] **A-R2:** scratch tag `v9.9.9` on the PR branch head → `verify-tag` red naming the version, no other
  job runs; tag deleted (`git push origin :refs/tags/v9.9.9`) and the deletion recorded. Needs the
  contract's authorization to push and delete a scratch tag (Q-Rb-6).
- LLM review: [ ] the regex refuses `v0.1.0-beta`, `v0.1`; [ ] a tag run cannot reach `bundle` on a red
  `verify-tag`.
- Failure cases: no `Cargo.toml` version → exit 1 naming it; `gh api` error → FAIL (never pass on an
  unknown); a tag on a commit not on `main` → FAIL.

### C2 — Linux arm64 and the fresh-runner smoke legs (no launcher yet)

- Goal: QR-2's two legs; §2's smoke separation. Scope: jobs `server-arm64`, `bundle-arm64`, `smoke` matrix
  with steps 1–2 and `package.py probe` as the stand-in for 4–6; `.sha256` per archive.
- Implementation: [ ] jobs; [ ] smoke-leg unpack per OS (§3.4 step 1).
- Validation: [ ] dispatch dry run: five smoke legs PASS or classified with evidence; arm64 probe output
  in the ledger; [ ] planted check: a `.sha256` file altered on a scratch branch → that smoke leg red.
- LLM review: [ ] smoke legs install no Rust and no Godot (read the job steps); [ ] `fail-fast: false`.

### C3 — AC-8 comparison

- Goal: A-R8. Scope: `ci_parity.py record --root`, `match`, self-test cases (a record differing in one
  `summary-300` line fails naming the world; a missing record fails; records of different commits fail);
  job `reference`; smoke step 3; job `parity`.
- Implementation: [ ] script; [ ] jobs.
- Validation: [ ] `python3 scripts/ci_parity.py --self-test`; [ ] a default `record` without `--root` is
  byte-identical to one made before the change on the same commit (captured before editing: test rules §25);
  [ ] dry run: `parity` PASS over five legs plus the reference; leg wall times recorded;
  [ ] **A-R8 mutation:** on a scratch branch, a step edits one line of `runtime/worlds/market-town/` in the
  unpacked bundle before step 3 → `parity` red naming `market-town` (and the digest check red too, each by
  name); run URL in the ledger.
- LLM review: [ ] the reference cannot come from a bundle; [ ] `match` cannot pass on zero records.

### C4 — SHA256SUMS, SBOM, attestations, draft, notes, reproducibility

- Goal: §13.4's checkpoint. Scope: job `draft`; `ci_release.py sums` and `notes`; the template; the
  `draft: true` input; `DEP-40`, `DEP-42`, notes in `ARC-48` and `ARC-79`; `PLAY.txt`'s verify lines.
- Implementation: [ ] job and scripts; [ ] records; [ ] `check_decision_ids.py --write-index`.
- Validation: [ ] dispatch with `draft: true` → draft `test-<run>` with five assets plus `SHA256SUMS`;
  `sha256sum -c SHA256SUMS` OK on the downloaded assets; `gh attestation verify` OK for each archive and for
  the SBOM predicate; no git tag `test-*` exists; draft deleted, recorded;
  [ ] **A-R7 mutation:** a scratch dispatch whose 2D drive is made to fail (a planted failing check in the
  drive on the scratch branch) → that smoke leg red, `draft` skipped, no release created;
  [ ] reproducibility: two dispatches on one commit, `ci_release.py repro` output in the ledger.
- LLM review: [ ] only `draft` has write permissions; [ ] no publish call anywhere (`grep -n "release edit\|--draft=false\|publish"`).

### C5 — launcher smoke (after R-c C5 lands)

- Goal: §9.1 steps 4–6 for real. Scope: smoke legs call `package.py smoke --mode 2d|3d|both`; `draft`
  refuses `launcher=absent`.
- Validation: [ ] dry run: every leg's three modes PASS; [ ] the stand-in path removed or kept only behind
  the `launcher=absent` refusal.
- LLM review: [ ] a tag run cannot draft without the launcher legs.

### C6 — gate

- [ ] Final head: one dispatch dry run green on all legs (exact head); `fast` green on the PR.
- [ ] Ledger, findings and the PR body current; READY FOR OPERATOR REVIEW.

---

## 6. Test ownership (test rules §26)

```text
UNIT OWNERSHIP     ci_release.py --self-test: tag/version rules, check-run reading (fixtures), notes
                   rendering, sums format. ci_parity.py --self-test: --root, match (fail and pass cases).
GATE 1 (real LLM)  N/A — no language model anywhere in the release (QR-8).
GATE 2 (real run)  the release workflow itself on GitHub runners: bundle, smoke (five legs), parity, draft.
                   This is the only place the musl / universal / MSVC servers and the exported runtime run
                   on their own OS from an archive.
STATIC             check_ci_pins.py (SHA pins, runner labels), check_decision_ids.py, check_doc_headings.py.
CI IMPACT          ci.yml unchanged; fast gains one self-test line. The PR's own CI is `fast` only (a
                   workflow + scripts change); its evidence is release.yml runs on release-dry-run/**.
```

---

## 7. Acceptance mapping

| Criterion | How R-b meets it | Evidence |
| --- | --- | --- |
| A-R2 tag ≠ version → no release | `verify-tag` (C1) | scratch tag run URL, red, no draft |
| A-R7 a failed smoke leg → no draft | `draft` needs every leg (C4) | planted-failure run URL, `draft` skipped |
| A-R8 release server = CI's world | `reference` + `match` (C3) | planted-world run URL, `parity` red by name |
| §13.4 checkpoint: four bundles, all smoke legs PASS, attestations verify, SHA256SUMS checks, `draft: true` makes only `test-<run>` | C4–C6 | dispatch run URL; `gh attestation verify` output; draft deleted |
| A-R4 budget (R-a's) stays red on overflow | unchanged `assemble` | — (R-a's evidence) |

---

## 8. Cost

- **Runner minutes.** Free on a public repository (ARC-48). The constraint is concurrency: macOS runners
  are capped at 5 machine-wide and this lane may have **at most 2 code-PR CI runs in flight**.
- **One full run** uses three macOS jobs: `bundle (macos-universal)`, `smoke (macos-26)`, `smoke (macos-15)`.
  The two smoke legs run at the same time, so the peak is 2 macOS runners. Estimated macOS minutes per run:
  bundle about 20 (#498's run 40447534966: 17.5 min to the end of assembly, before any probe ran), each
  smoke leg 10–20 (AC-8 record plus three launcher modes; **measure** in C3). Roughly 40–60 macOS minutes
  and 2 concurrent macOS runners.
- **Runs this PR needs:** C1 dry run + scratch tag (seconds; no macOS), C2 dry run, C3 dry run + planted
  run, C4 `draft: true` run + planted run + second reproducibility run, C6 final run: about **7 full runs**.
  Scheduled one at a time, so the lane never exceeds its two in-flight runs. The planted runs may cancel the
  macOS legs they do not need (`workflow_dispatch` inputs are not added for that; the scratch branch's copy
  of the matrix is trimmed instead and the trim recorded).
- **Storage.** Each run keeps about 1.3 GB of artifacts for 7 days (four archives of 300–400 MB). Within the
  free artifact storage of a public repository (**verify** the current limit).
- **Money:** none.

---

## 9. Risks

| Risk | Mitigation |
| --- | --- |
| The AC-8 record makes smoke legs slow on Windows | measure in C3; Q-Rb-3 offers a shorter release profile |
| `macos-15` cannot run the macOS universal runtime (minimum OS of Godot 4.7.2 templates) | the leg is the evidence; a FAIL is a finding with the Godot minimum cited, not a skip |
| The `draft: true` dry run leaves a draft or tag behind | ledger step deletes it; C4 checks `git ls-remote --tags` |
| A scratch tag is seen publicly | named `v9.9.9`, deleted within minutes; only if Q-Rb-6 allows |
| #498 changes before merging | stack on its head or wait for its merge (Q-Rb-1); read-only coordination, never its worktree |

---

## 10. Operator questions (each with a recommended answer)

- **Q-Rb-1 — Job shape.** Keep #498's per-OS build-and-assemble (deviates from §5.2's export-once-on-Linux)
  and add separate fresh-runner smoke jobs as §5.2 requires? **Recommended: yes.** It is written and runs
  the same command developers run; the macOS runner is needed for the server anyway; the fresh smoke legs
  keep the "as a player would" test.
- **Q-Rb-2 — R-c C5 (launcher in the bundle, `package.py smoke`).** Who lands it? **Recommended: a small
  separate PR, R-c C5, under R-c's frozen §19.4, before R-b's C5.** R-b C1–C4 do not wait for it; a tag run
  refuses to draft until it lands.
- **Q-Rb-3 — AC-8 in the release.** Compare each bundle's server against a reference built in the same run
  (`ci_parity.py record --root` + `match`), on the bundle's three worlds, with AC-8's default profile (300 +
  30 days)? **Recommended: yes**; if a leg takes over 15 minutes, allow a shorter profile then.
- **Q-Rb-4 — Release notes template path.** `.github/release-notes.md` would be a spec under CLAUDE.md
  §2.1. **Recommended: `packaging/release-notes.txt.in`** (plain text with Markdown syntax, rendered by
  `ci_release.py notes`), beside `PLAY.txt` and `BUNDLE.toml.in`.
- **Q-Rb-5 — Artifact retention.** **Recommended: 7 days for dry runs (as #498), 90 days for tag runs**;
  the onboarding layer B finds the latest successful run itself.
- **Q-Rb-6 — Scratch tag for A-R2.** May the implementing session push `v9.9.9` to the PR branch's head
  and delete it minutes later? **Recommended: yes**; the alternative (self-test only) does not prove the
  trigger and the job wiring.
- **Q-Rb-7 — F-Rb-8.** The 3D pack ships `res://tools/*` scripts though ARC-80 says `tools/` is not shipped.
  **Recommended: R-b only reports it; the 3D lane narrows the export filter in its own PR.**

---

## 11. Execution contract

Filled 2026-10-27 by the lane lead (the implementation session) on the coordinator's relay of the
operator's ruling of 2026-10-27 (the design frozen; Q-Rb-1 … Q-Rb-7 answered as recommended), with the
coordinator's narrowing of Q-Rb-6 recorded below.

```text
PROJECT / PR:              MineWorld mvp0, PR S23 R-b — the CI release workflow
PRIMARY DESIGN DOC:        .structured-coding/plans/mvp0/pr-s23-rb-release-workflow.md (this file)
RELATED / BINDING DOCS:    step-23-release.md §§3, 5, 9, 10, 14, 19 (R-c C5, F-RC-6, F-RC-7, D-RC-5);
                           docs/decisions ARC-48, ARC-79, ARC-80, DEP-38, DEP-41, DEP-45
IMPLEMENTATION BASE:       PR #498's head 469ffa8c (branch release-dry-run/workflow, not merged) plus
                           origin/main merged in (315a50a9: R-c C5 #517, 3D tools filter #512). Stacked
                           on #498, the design's allowed alternative; this PR merges after #498. If #498
                           merges first, origin/main is merged again.
APPROVED SCOPE:            §3 as frozen, with Q-Rb-1 … Q-Rb-7 answered as recommended, except Q-Rb-6 (below)
FROZEN INVARIANTS:         §4.1 (I-Rb-1 … I-Rb-6), and: nothing is published. `draft` creates drafts only;
                           on a dispatch only `test-<run id>` drafts, deleted after the evidence is taken.
                           The artifact interface of §4.3 is kept exactly (another PR, 25g layer B,
                           consumes it).
AUTONOMOUS ACTIONS:        implement, validate, commit, push, open the PR, dispatch release.yml on this
                           branch (or push it to a release-dry-run/** branch, the dry-run trigger, while
                           the workflow is not on main). Not: merge, arm auto-merge, push any tag (the
                           A-R2 scratch tag included), create a non-draft release.
Q-Rb-6 (narrowed):         the coordinator's instruction of 2026-10-27 replaces the recommended "yes": no
                           scratch tag is pushed by this session. A-R2 is proven by the script's
                           self-test and the CLI against a mismatched tag; the real scratch-tag run is
                           pending operator action.
STOP CONDITIONS:           any change to frozen scope, a public contract, an ownership boundary, or a
                           dependency beyond the design's pins.
VALIDATION BUDGET:         local: ci_release.py and ci_parity.py self-tests, check_ci_pins.py, actionlint
                           if available; cargo and Godot only through cast-slots/with-slot.sh. Platform
                           evidence: release.yml runs on this branch, at most ONE in flight, polled at
                           most every 5 minutes; only heads expected green are pushed.
MERGE AUTHORITY:           the coordinator only. This session never merges.
NORMAL STOP:               READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 12. Ledger

| Date | Commit / run | Event |
| --- | --- | --- |
| 2026-10-26 | — | revision 1 drafted from origin/main 0b2faf2f and #498 head 5cb3c313; #498 run 40447534966: all three legs assemble inside the budget, then FAIL at the probe step's first line (§1.3) |
