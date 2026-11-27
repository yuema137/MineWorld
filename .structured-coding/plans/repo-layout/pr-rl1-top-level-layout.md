# PR RL1 — Repository top-level layout: group by architectural layer (move-only)

**Lifecycle:** DRAFT — FREEZE-READY. Not frozen. Implementation requires the operator's answers to §0
and a `DESIGN FROZEN` header written on the operator's approval. Nothing in this document authorizes a
file move.
**Requirement (operator, 2026-11-27):** "现在这个repo顶层的文件和文件夹太多了，很杂乱，你需要好好organize"
("the repository's top level has too many files and folders and is messy; organize it properly").
**Effort:** `repo-layout` (one PR; no step or overall document — the work is a single mechanical PR, so
this file is the step and the PR design at once, per the skill's "expand the step in place" rule).
**Author:** design session, 2026-11-27, worktree `/Users/yuema137/mineworld-worktrees/design-repo-layout`,
branch `docs/pr-design-repo-layout`. **Audit base:** `b8394593` (merge of PR #533, contained in
`origin/main`). Every count below was measured at that commit with `git grep` / `git ls-files`.
**Placeholder ids:** Q-RL-n (freeze questions), D-RL-n (decisions), I-RL-n (invariants), A-RL-n
(acceptance), R-RL-n (risks), C-n (commits).

---

## 0. Decisions for the operator (plain language, with recommendations)

Each question has a recommendation. Answering "accept all recommendations" freezes the design as written.

| # | Question | Options | Recommendation and why |
| --- | --- | --- | --- |
| **Q-RL-1** | Where do the seven root-level Rust crates of the kernel and host (`contracts`, `kernel`, `persistence`, `packages`, `worldpack`, `authoring`, `server`) go? | **A** `crates/<name>/` · **B** a layer folder `kernel/` holding them (the `kernel` crate becomes `kernel/core/`) · **C** leave them | **A.** `crates/` is the dominant Rust monorepo convention (Cargo itself, rust-analyzer, ripgrep, Bevy, uv, ruff all use it) and introduces no new MineWorld term. B would reuse the defined term `Kernel` for a folder that also holds `server` and `persistence`, and forces a `kernel/kernel` or a crate-folder rename. Crate *names* (`mineworld-kernel`, …) do not change. |
| **Q-RL-2** | Do the three content roots `worlds/`, `entities/`, `presentation/` move under one folder (e.g. `content/`)? | **A** keep at root · **B** `content/{worlds,entities,presentation}` | **A (keep).** Each is one pack type of `MODULE_SPEC.md` (World / Entity / Presentation Pack), the same as `systems/` (System Packs) and `cognition/` (Controller Packs), so the root then reads as the extension model. They are also user-facing paths: every documented command (`mineworld run worlds/market-town`, `--packs entities`), the executable guide, the Docker runtime image, and the downloadable bundle's layout (`ARC-80`: `--root` "stands for the repository root"; clients resolve `<root>/presentation/…`) name them. Moving them is a public-path change, not tidying. |
| **Q-RL-3** | Where do the three launcher scripts `mineworld-2d`, `mineworld-slice`, `mineworld-3d` go? | **A** `bin/` · **B** inside their clients (`clients/2d/mineworld-2d`, `clients/3d-spike/mineworld-{slice,3d}`) · **C** leave | **A.** Three root files become one folder; the README quick start stays one short command (`bin/mineworld-2d`); the conventional place for developer entry scripts. B gives better locality but puts bash files inside Godot project folders (exported by `all_resources` filters — risk to the bundle) and lengthens the quick start. None is obsolete: `mineworld-2d` and `mineworld-slice` are the playable entry points; `mineworld-3d` still runs `--drive/--shots/--portrait` for character review (`docs/HUMAN_REVIEW_QUEUE.md`). |
| **Q-RL-4** | What happens to `spike/` (the pre-kernel renderer-integration spike)? | **A** delete the code; move `FINDINGS.md`, `README.md`, `evidence/` and `unreal/` evidence to `docs/history/renderer-spike/` · **B** move whole to `docs/history/renderer-spike/` · **C** keep | **A.** Verified obsolete as code (§2.3): its own Cargo workspace, not a member, not built or run by any CI job, untouched since 2026-10-07 while `mineworld-contracts` kept changing, self-described as "throwaway … nothing here is production code". Its *findings* are still cited by live code comments (`spike/FINDINGS.md` F8.1, F8.2, F9), so the findings and evidence stay; the code stays in history at the move commit's parent. |
| **Q-RL-5** | `packaging/` (4 templates read only by `scripts/package.py`)? | **A** `scripts/packaging/` · **B** `tools/packaging/` · **C** leave | **A.** Data next to its only reader (`scripts/package.py` `PACKAGING = ROOT / "packaging"`, and the `cargo about -c packaging/about.toml` call). |
| **Q-RL-6** | Does `scripts/` (CI and repository checks) merge into `tools/` (e.g. `tools/ci/`)? | **A** keep `scripts/` at root · **B** move to `tools/ci/` | **A (keep), revisit later.** `scripts/` is a universally recognised root name; moving it changes `ROOT = Path(__file__).parent.parent` in every script (silent depth bugs), touches ~325 code/CI lines and every workflow, and collides with the active CI lanes (`ci/econ-*`, PR #534). One fewer root entry is not worth that now. |
| **Q-RL-7** | Are path mentions inside *history* (merged PR designs and handoffs under `.structured-coding/plans/`, decision-record prose under `docs/decisions/`) rewritten? | **A** rewrite Markdown link targets everywhere; leave code-span / prose mentions in history as written; the new decision record carries the old→new path map · **B** rewrite every mention everywhere · **C** rewrite nothing in history | **A.** Links must resolve (a reader with only the repository follows them), but plans and records quote commands and evidence that were true when written; rewriting ~5 000 history lines would falsify what was run and bloat the review. The path map (D-RL-6) resolves any old path in one lookup. |

Considered and **kept at root** (no question needed — tool or convention requires it): `Cargo.toml`,
`Cargo.lock` (Cargo workspace root), `rust-toolchain.toml` (rustup searches the directory upward for this
exact name), `clippy.toml`, `deny.toml` (each tool's documented default location; hiding them as
dotfiles is cosmetic and `cargo-deny`'s non-root search order was not verified), `pyproject.toml` +
`uv.lock` (the uv *workspace root* must hold both; `pyright` configuration lives in it and runs from the
root), `Dockerfile` + `.dockerignore` (CI builds with `context: .` and default file; moving them only
adds `-f`), `README.md`, `README.zh-CN.md`, `LICENSE`, `NOTICE` (GitHub and licence conventions),
`AGENTS.md`, `CLAUDE.md` (agent entry points read from the root), and the dot entries `.cargo/`,
`.claude/`, `.github/`, `.structured-coding/`, `.gitattributes`, `.gitignore`.

`.DS_Store` is **not tracked**: `.gitignore` already ignores it. It exists only in the operator's local
root checkout; `find . -name .DS_Store -delete` there removes it. Nothing in this PR.

Considered and **deferred** (out of scope — not root clutter): renaming `clients/3d-spike/` to
`clients/3d/` (it is the real 3D client, the name is historical; ~1 500 references, its own PR);
moving `tools/cli` (the `mineworld` binary) into `crates/` (~2 800 `tools/` references).

---

## 1. Before / after

### 1.1 Before (audit base, 37 visible entries + 7 tracked dot entries)

```text
.cargo/ .claude/ .github/ .structured-coding/ .dockerignore .gitattributes .gitignore
AGENTS.md  CLAUDE.md  README.md  README.zh-CN.md  LICENSE  NOTICE
Cargo.toml  Cargo.lock  rust-toolchain.toml  clippy.toml  deny.toml
pyproject.toml  uv.lock  Dockerfile
mineworld-2d  mineworld-3d  mineworld-slice                  launcher scripts
authoring/  contracts/  kernel/  persistence/  packages/  worldpack/  server/   Rust kernel + host crates
sdk/  systems/  cognition/                                   pack SDK, System Packs, Controller Packs
worlds/  entities/  presentation/                            World / Entity / Presentation Packs
clients/  tools/  scripts/  packaging/  tests/  docs/  spike/
```

### 1.2 After (recommended answers; 27 visible entries)

```text
.cargo/ .claude/ .github/ .structured-coding/ .dockerignore .gitattributes .gitignore
AGENTS.md  CLAUDE.md  README.md  README.zh-CN.md  LICENSE  NOTICE        project entry + legal
Cargo.toml  Cargo.lock  rust-toolchain.toml  clippy.toml  deny.toml     Rust workspace (tool-required)
pyproject.toml  uv.lock                                                 Python workspace (tool-required)
Dockerfile                                                              container (tool default)

bin/            mineworld-2d  mineworld-slice  mineworld-3d             developer launchers
crates/         contracts  kernel  persistence  packages  worldpack     the kernel and its host
                authoring (+ authoring/attributes)  server
sdk/            rust/  python/                                          the pack author's boundary
systems/        System Packs (+ installed/)                             unchanged
cognition/      Controller Packs                                        unchanged
worlds/         World Packs                                             unchanged
entities/       Entity Packs                                            unchanged
presentation/   Presentation Packs                                      unchanged
clients/        2d  3d-spike  agent  protocol  runtime  shared          unchanged
tools/          cli  launch  weather-fetch  + authoring pipelines       unchanged
scripts/        CI and repository checks (+ packaging/)                 + packaging templates
tests/          acceptance  support  fixtures                           unchanged
docs/           specifications, guide, decisions (+ history/renderer-spike/)
```

Reading order of the root now follows `ENGINEERING_STANDARDS.md` §6 and `CLAUDE.md` §4.4: `crates/`
(kernel contracts and kernel) ← `sdk/` ← `systems/`, `cognition/` ← `worlds/`, `entities/`,
`presentation/` (data packs) — with `clients/` observing over the protocol and `tools/`, `scripts/`,
`tests/`, `docs/` outside the runtime.

### 1.3 Path map (D-RL-6; becomes the table in the decision record)

| Old | New |
| --- | --- |
| `contracts/` | `crates/contracts/` |
| `kernel/` | `crates/kernel/` |
| `persistence/` | `crates/persistence/` |
| `packages/` | `crates/packages/` |
| `worldpack/` | `crates/worldpack/` |
| `authoring/` (incl. `authoring/attributes/`) | `crates/authoring/` |
| `server/` | `crates/server/` |
| `mineworld-2d`, `mineworld-slice`, `mineworld-3d` | `bin/mineworld-2d`, `bin/mineworld-slice`, `bin/mineworld-3d` |
| `packaging/` | `scripts/packaging/` |
| `spike/FINDINGS.md`, `spike/README.md`, `spike/evidence/`, `spike/unreal/{README.md,evidence/}` | `docs/history/renderer-spike/…` (same relative names) |
| `spike/server/`, `spike/client-2d/`, `spike/client-3d/`, `spike/run.sh`, `spike/unreal/probe.py` | deleted (last present at the move commit's parent) |

---

## 2. Audit (evidence for the decisions)

### 2.1 Reference counts per moved path (audit base)

`code/config` = every tracked non-Markdown file except `Cargo.lock`; `live md` = Markdown outside
`.structured-coding/plans/` and `docs/decisions/`; `history` = those two trees. Pattern: the path with a
trailing `/` not preceded by a path character (relative `../` prefixes included).

| Path | Files | code/config | live md | history |
| --- | ---: | ---: | ---: | ---: |
| `authoring/` | 51 | 18 | 3 | 284 |
| `contracts/` | 40 | 32 | 44 | 567 |
| `kernel/` | 52 | 32 | 33 | 673 |
| `persistence/` | 20 | 8 | 21 | 388 |
| `packages/` | 117 | 23 | 3 | 187 |
| `worldpack/` | 33 | 21 | 18 | 597 |
| `server/` | 69 | 75 | 72 | 943 |
| `spike/` | 35 | 37 | 33 | 44 |
| `packaging/` | 4 | 1 | 0 | 12 |
| launchers (all three, by name) | 3 | ~114 | ~136 | — |
| **Total** | | **~360** | **~360** | **~3 700** |

Rejected moves, for scale: `scripts/` 325 code + 79 live md + 1 529 history; `worlds/` 1 729 lines;
`presentation/` 736; `systems/` 2 213; `clients/` 2 963; `tools/` 2 871.

Estimated rewrite under the recommended answers: **~720 lines in code, CI, specs and READMEs, plus
history link targets only (expected low hundreds)**, across roughly 250–300 files. The exact list is
produced by the rewrite tool (C1) and recorded in §7 before commit.

### 2.2 Path dependencies that break silently (must be rewritten and proven, not assumed)

A move changes the depth of seven crates. Paths that are resolved at *run* time, not compile time, do
not fail the build:

1. **`CARGO_MANIFEST_DIR` + `..`** in the moving crates: 28 occurrences, e.g.
   `worldpack/tests/social_cafe.rs` `concat!(env!("CARGO_MANIFEST_DIR"), "/../worlds/social-cafe")`,
   `persistence/tests/kill_and_resume.rs`, `packages/tests/{depiction,far_field,place_asset,shells}.rs`,
   `authoring/attributes/tests/{structure,fidelity}.rs`, `worldpack/tests/entity_packs.rs`. Each gains
   one `../`.
2. **`include_str!` across crates:** `authoring/attributes/src/disclosed/tests.rs`,
   `server/src/seats/budget_tests.rs` (relative to the source file; internal to the crate, unchanged
   unless the target is outside it). `systems/weather/src/fixture.rs` points at `worlds/` — unchanged.
3. **GDScript paths from clients into moved crates** (clients stay at depth 2, so `res://../..` is still
   the repository root; only the *target* changes): `clients/protocol/checks/delta_check.gd`
   (`server/tests/frames/deltas`), `clients/protocol/checks/attributes_check.gd`
   (`authoring/attributes/tests/fixtures/composition.json`),
   `clients/3d-spike/scripts/checks/depiction_check.gd` and `clients/shared/checks/far_field_check.gd`
   (`packages/tests/fixtures/…`).
4. **Prefix allow/deny lists:** `scripts/check_secrets.py` lists `server/`, `kernel/`, `contracts/`,
   `worldpack/`, `persistence/` — a stale prefix silently scans nothing. `scripts/ci_changes.py`
   classifies changed paths into CI layers by prefix (stale → a layer silently skipped).
   `scripts/check_scratch.py` (`packages/tests/…`), `tests/acceptance/tests/{ac1_composability,
   client_text,configuration_vocabulary}.rs` name files under moved crates.
5. **Tests that skip when a path is absent** (`SKIPPED: 3D client absent …`, `ci_removability.py`
   D-e-3): a wrong path turns a check into a skip that still exits 0.
6. **`trybuild` expectations:** two `.stderr` files contain `$WORKSPACE/authoring/src/…`.
7. **Launchers** compute the repository root as `$(dirname "$0")`; in `bin/` it is `$here/..`.
   `scripts/ci_godot.py` runs `ROOT / "mineworld-slice"`; `scripts/ci_changes.py` `CLIENT_FILES`;
   `scripts/ci_removability.py` `REMOVED_3D`; `scripts/check_client_rules.py`; `.dockerignore`.
8. **Manifests:** root `Cargo.toml` `members` and `[workspace.dependencies]` paths;
   `.cargo/config.toml` `[patch.crates-io]` paths (read by `tests/acceptance/tests/package_sources.rs`);
   `.gitignore` `!persistence/tests/fixtures/…` negations (a stale one would let the committed SQLite
   fixtures be ignored — they are tracked, so the effect is on *new* fixtures only). Crate-to-crate
   path dependencies among the seven are all `workspace = true` or stay siblings inside `crates/`;
   `cognition/*`, `tools/*`, `systems/*` reach them through `[workspace.dependencies]`, so no other
   crate manifest changes. `Cargo.lock` records no path for path packages and must stay byte-identical.

Not affected (verified): the 20 tracked symlinks are all inside `clients/` (`clients/2d/mineworld ->
../protocol/mineworld`, …); `.github/workflows` name none of the seven crate folders (0 hits); the
bundle layout (`ARC-80`) contains no crate folder; the Godot projects do not move.

### 2.3 `spike/` obsolescence evidence

- `spike/server/Cargo.toml` declares its own `[workspace]` ("must not touch the root workspace") and
  depends on `../../contracts` by path; it is not a root member and no CI job, script or workflow builds
  or runs it (`git grep spike/ -- .github scripts` → only `.dockerignore`, `.gitignore`).
- Last change to `spike/`: 2026-10-07 (`37f37aa9`). `mineworld-contracts` changed many times since; the
  spike's build status is unknown and is not checked by anything.
- `spike/README.md`: "A throwaway proof … Nothing here is production code and nothing here is the
  kernel."
- Live references are to its *findings*: `tools/cli/src/perceive.rs`, `tools/cli/tests/
  ac13_semantic_parity.rs`, `tools/cli/tests/support/mod.rs` cite `spike/FINDINGS.md` F8.1/F8.2/F9;
  `docs/HUMANOID_PROFILE.md` cites `spike/server/src/world.rs` (rewritten to a permalink at the move
  commit's parent).

### 2.4 Launchers are not obsolete

`mineworld-2d` and `mineworld-slice` are the README quick start and are run by CI
(`scripts/ci_godot.py`, `ci_changes.py`, `ci_removability.py`). `mineworld-3d` refuses a plain walk
(Q-FF-3) but its `--drive`, `--shots`, `--portrait` modes are the character-review path in
`docs/HUMAN_REVIEW_QUEUE.md` and `docs/HUMANOID_PROFILE.md`. Whether to retire `mineworld-3d` is a
separate operator question, not part of tidying.

---

## 3. Invariants

- **I-RL-1 Move only.** No behaviour change: no source edit other than a path string, a path
  constant, or a comment/link naming a moved path. No crate renamed, no dependency added or removed, no
  version change.
- **I-RL-2 Same build graph.** `Cargo.lock` byte-identical; `cargo metadata --format-version 1` equal
  to the base after normalising each `manifest_path` / path-dependency `path` through the path map
  (package ids, names, versions, features, targets, dependency edges).
- **I-RL-3 No silent skips.** The set of executed tests, and every scanner's scanned-file set, equals the
  base's mapped through the path map (§5 V-3, V-4).
- **I-RL-4 No old path in live surfaces.** After C1, `git grep` for each old path (§1.3, left column,
  with the §2.1 pattern) returns nothing outside `.structured-coding/plans/**`, `docs/decisions/**`
  prose (Q-RL-7 A) and the path map itself.
- **I-RL-5 User-facing paths unchanged except the launchers.** `worlds/`, `entities/`,
  `presentation/`, `clients/`, `docs/guide/` commands and the bundle layout are untouched.
- **I-RL-6 Kernel ignorance and dependency direction unchanged.** The move is folder placement only;
  `ENGINEERING_STANDARDS.md` §6 direction is now visible in the tree but not altered.

---

## 4. Commit plan

### C1 — `chore(layout): move kernel and host crates to crates/, launchers to bin/, packaging under scripts/` (move + rewrite, one commit)

One commit so that every intermediate tree builds; `git mv` keeps rename detection (content changes
in moved files are path strings only, far below Git's 50 % similarity threshold).

- **Goal.** The §1.2 tree, with every reference rewritten.
- **Scope.** `git mv` per §1.3 (except `spike/`); root `Cargo.toml`; `.cargo/config.toml`;
  `.gitignore`, `.gitattributes`, `.dockerignore`, `Dockerfile` (only if it names a moved path — it does
  not at base); the 28 `CARGO_MANIFEST_DIR` sites; the GDScript sites of §2.2(3); `scripts/*.py` sites of
  §2.2(4, 7); the three launchers' root computation; the two `.stderr` files; `tests/acceptance`
  literal paths; every live Markdown file; Markdown link targets in history (Q-RL-7 A); the README quick
  start; `CLAUDE.md` §1 "Current repository state" line (points to the layout record); crate READMEs'
  relative links.
- **Rewrite tool.** `scripts/relayout.py` (committed in this PR; removed by a follow-up once every
  in-flight branch has rebased, §6): reads the path map, rewrites with anchored regexes (a path is
  matched only when preceded by start, whitespace, quote, backtick, `(`, `/` after `..`, or `=`), handles
  `../` depth changes for files that themselves moved, has `--check` (exit 1 if any rewrite remains) and
  `--list` (print every site). Idempotent. No `sed -i`.
- **Implementation steps.**
  - [ ] Write `scripts/relayout.py` with the path map and a self-test over representative lines from
        §2.2 (each category, plus negatives: `mineworld-packages`, the English word "packages",
        `systems/installed/…`, `clients/3d-spike/…`).
  - [ ] `git mv` the folders and launchers; commit nothing yet.
  - [ ] Run the tool; review `--list` output by category; hand-fix any site it cannot classify and
        record it in §7.
  - [ ] Launchers: `here=…/..` root computation; header comments' usage lines (`./mineworld-2d` →
        `bin/mineworld-2d`).
  - [ ] Add the decision record (C2 content may ride in this commit if the reviewer prefers one commit).
- **Acceptance.** A-RL-1 … A-RL-7 (§5).
- **Failure cases.** A site the tool rewrites wrongly (false positive on a word) → caught by the
  negative self-test and by review of `--list`; a site it misses → I-RL-4 grep and V-3/V-4; a moved
  fixture path that a test reads conditionally → V-3 skip-count equality.

### C2 — `docs(decisions): ARC-<next> repository layout and path map`

- [ ] New record `docs/decisions/ARC-<next>.md` (next free id from `docs/decisions/INDEX.md`, assigned
      at implementation): the layout rule ("the root holds tool-required files and one folder per
      architectural layer or pack type; a new top-level entry is a reviewed decision"), the §1.3 path map,
      the Q-RL answers. Regenerate the index with the existing script; `check_decision_ids.py` passes.
- [ ] `docs/ARCHITECTURE.md` gains a short "Repository layout" subsection linking the record (the
      specification, not a README, is the authority — `CLAUDE.md` §2.1 rule 1). Root `README.md` gets a
      four-line folder map linking it.

### C3 — `chore(spike): retire the renderer spike; keep its findings under docs/history/`

Separate commit so it can be reverted alone if the operator answers Q-RL-4 differently.

- [ ] `git mv` findings, README and evidence to `docs/history/renderer-spike/`; `git rm` the code.
- [ ] README there: one paragraph — what it proved, the commit at which the code last existed, how to
      check it out (`git show <parent>:spike/…`).
- [ ] Rewrite the live citations (§2.3) to the new path; `docs/HUMANOID_PROFILE.md`'s source citation
      to a permalink.
- [ ] Drop the `spike/` lines from `.gitignore` and `.dockerignore`.

Commit boundary: each commit is reviewable alone; nothing else (no 3d-spike rename, no `tools/cli`
move, no script refactor) rides along.

---

## 5. Validation (one full run; the move touches every layer)

Local validation is limited by R-LOAD-1 (no local full-workspace test runs while other lanes are
active): locally run the cheap checks; CI on the exact head is the gate for the full suite.

- **V-1 Build graph (local).** `cargo metadata --format-version 1` at base and head, both normalised by
  the tool's `--normalise-metadata` mode (path map applied, absolute prefix stripped); diff empty.
  `git diff --quiet <base> HEAD -- Cargo.lock`. `cargo check --workspace --all-targets` and
  `cargo fmt --all --check` (local, single job).
- **V-2 Old-path scan (local).** `python3 scripts/relayout.py --check` exits 0 (I-RL-4).
- **V-3 Same tests executed (CI).** Full CI once on the head with the `milestone` label (all layers,
  nightly included by dispatch). Compare per-binary test counts and the count of `SKIPPED:` lines with
  the latest green `main` run of the same layers: equal.
- **V-4 Scanners scan the same files (local).** `check_secrets.py`, `check_scratch.py`,
  `check_client_rules.py`, `ci_changes.py` self-tests, plus a `ci_changes.py` classification of the
  synthetic "every tracked file changed" diff at base and head: layer selections equal after mapping.
- **V-5 Guide and links.** The executable guide test (`tools/cli/tests/guide*`) passes in CI; a local
  link check over all Markdown (the tool's `--links` mode: every relative link target exists) reports no
  new broken link versus base.
- **V-6 Godot.** `scripts/ci_gdparse.py` and the protocol/shared checks (`delta_check`, `attributes_check`,
  `far_field_check`, `depiction_check`) run in CI and print their pass lines (missing line = FAIL).
- **V-7 Launchers and bundle.** `bin/mineworld-2d --drive` and `bin/mineworld-slice --drive` (local, one
  each, headless where supported); `scripts/package.py` on one platform in CI: bundle file list and
  `files_sha256` inputs equal to base except nothing (the bundle contains no moved path).
- **V-8 Removability.** `scripts/ci_removability.py --target 3d --no-godot` and `--target lm` pass in CI.
- **V-9 Docker.** The CI image build (`context: .`) succeeds; runtime image file list unchanged.

Acceptance:
- **A-RL-1** §1.2 tree exactly (`ls -A` at root lists only §1.2 entries).
- **A-RL-2** V-1 empty diffs; `Cargo.lock` identical.
- **A-RL-3** V-2 exit 0.
- **A-RL-4** V-3 counts equal, including `SKIPPED:` lines.
- **A-RL-5** V-4 equal sets.
- **A-RL-6** V-5, V-6, V-8, V-9 pass with their pass lines present.
- **A-RL-7** V-7 drives print their verdict lines; bundle list equal.

Results are classified `PASS` / `FAIL` / `INCONCLUSIVE` from evidence in §7, local and remote recorded
separately.

---

## 6. Timing, in-flight branches, rollback

**Timing.** Land when the active code lanes have merged or reached a checkpoint, so the rebase burden is
smallest: before dispatching the next batch of implementation PRs, after PR #534 (`ci/econ-p1-nextest`,
which edits `scripts/` and CI) merges or rebases. Design-doc-only PRs (#535–#537) are unaffected except
for link targets. Announce a short freeze window (no merges to `main` between the move PR's final CI
run and its merge), because any merge in between needs the tool re-run.

**Rebase recipe for an in-flight branch** (run in that branch's own worktree):

```sh
git fetch origin
git -c merge.directoryRenames=true -c merge.renameLimit=0 rebase origin/main
#   directoryRenames=true: a file the branch ADDED under kernel/ lands under crates/kernel/
#   (Git's default "conflict" stops and asks); renameLimit=0: no cap on rename detection.
python3 scripts/relayout.py            # rewrite path strings the branch itself introduced (idempotent)
python3 scripts/relayout.py --check    # must exit 0
git add -A && git commit -m "chore(layout): rewrite paths after the crates/ move"   # or fold into the rebase
cargo check --workspace                # then the branch's own validation
```

A conflict in a moved file is a normal content conflict at its new path. A branch that adds a new crate
under a moved folder's sibling (e.g. a new root-level crate) must decide its home per the layout rule
(D-RL-6 record) — normally `crates/` or `systems/`.

**Rollback.** The PR is pure renames + path strings, so `git revert -m 1 <merge>` restores the old tree;
branches that already rebased re-run the recipe against the reverted main (the tool has a `--reverse`
mode using the inverted map). C3 reverts independently.

---

## 7. Evidence ledger (filled during implementation)

| Item | Local | Remote (CI, exact head) |
| --- | --- | --- |
| V-1 … V-9 | — | — |

Exact rewrite count, hand-fixed sites, and deviations: —

---

## 8. Risks

- **R-RL-1 Silent skips / wrong-prefix scans** (§2.2 items 4–5). Mitigation: V-3 skip-count equality and
  V-4 scanned-set equality — these are the checks that would otherwise pass vacuously.
- **R-RL-2 Rebase pain for active lanes.** Mitigation: timing, the committed idempotent tool, the recipe,
  `merge.directoryRenames=true`.
- **R-RL-3 Muscle memory / external docs** (`./mineworld-2d` in videos, the progress page, chat history).
  Mitigation: the README quick start and `docs/MVP_STATUS.md` change in the same PR; the progress page
  (`mineworld-demos/progress/index.html`) is updated on merge. No compatibility shim at the root (no
  early compatibility debt, `CLAUDE.md` §4.12).
- **R-RL-4 Over-matching rewrite** (e.g. the word "packages", `mineworld-server`, `clients/…/server/`).
  Mitigation: anchored patterns, negative self-tests, human review of `--list`.
- **R-RL-5 Godot editor caches** (`.godot/` under clients, untracked) keep stale absolute paths on
  developer machines. Only clients' *targets* change, not their location; at worst a local re-import.
- **R-RL-6 History readers** meet old paths in prose (Q-RL-7 A). Mitigation: the path map in the decision
  record, linked from `CLAUDE.md` §1.
- **R-RL-7 `cargo-deny`/`clippy` config discovery** — unchanged (files stay at root).
