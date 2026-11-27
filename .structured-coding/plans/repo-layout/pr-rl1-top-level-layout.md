# PR RL1 — Repository top-level layout: group by architectural layer

**`DESIGN FROZEN 2026-11-27 (operator ruling "仓库整理按推荐"; rulings in §0.3)`**

**Lifecycle:** FROZEN — awaiting execution. Scope, invariants, decisions and acceptance are frozen at
revision 3. Design base: `origin/main @ 303c0df360a04c2a4bd4abb6b2cd1596dc7d2f91`. **Execution timing:**
after PR #534 and the in-flight lanes (#540, #541 and any lane the primary names) merge; the primary
session schedules the quiet window (merge freeze on `main`) and dispatches a fresh implementation session
in its own worktree. The implementation base is `origin/main` at the start of that window, recorded in
§10. Superseded lifecycles: DRAFT — FREEZE-READY revision 2 (`bba8033f`, red-team PASS with two
non-blocking leftovers, applied in revision 3); revision 1 (`c9ea0a12`, red-team NOT-READY, findings
F-1 … F-10 traced in §11). Merge requires the operator's explicit authorization.
**Requirement (operator, 2026-11-27):** "现在这个repo顶层的文件和文件夹太多了，很杂乱，你需要好好organize"
("the repository's top level has too many files and folders and is messy; organize it properly").
**Effort:** `repo-layout`, one PR. This file is the step design and the PR design at once (the skill's
"expand the step in place" rule).
**Author:** design session, 2026-11-27, worktree `/Users/yuema137/mineworld-worktrees/design-repo-layout`,
branch `docs/pr-design-repo-layout`.
**Audit base:** `origin/main @ df785c98` (revision 2; revision 1 audited `b8394593`). Every count below was
measured at `df785c98` with `git grep` / `git ls-files` / `git ls-tree`. The implementation re-measures at
its own base and records the exact lists in §10.
**Placeholder ids:** Q-RL-n (freeze questions), I-RL-n (invariants), A-RL-n (acceptance), V-n
(validation), S-n (material stops), R-RL-n (risks), C-n (commits).

---

## 0. Decisions for the operator (plain language, with recommendations)

"Accept all recommendations" freezes the design as written (Option 1 below, every Q at its recommendation).

### 0.1 Intensity: how much to move

| Option | What moves | Root after | Extra cost over Option 1 |
| --- | --- | --- | --- |
| **1 (recommended)** | The seven kernel/host crates → `crates/`; launchers → `bin/`; `packaging/` → `scripts/packaging/`; `spike/` retired | 29 visible entries (from 39) | — |
| **2** | Option 1, **plus** `systems/`, `cognition/`, `entities/` → `packs/{systems,cognition,entities}/` and `sdk/` → `crates/sdk/{rust,python}/` | 26 visible entries | ≈ +273 code/config lines, +177 live-Markdown lines, +3 600 history lines (§2.1); changes the **public pack-installation recipe** (`MODULE_SPEC.md` §3.1 `systems/<name>/`, `ARC-33`), the `systems/*` workspace glob, `.cargo/config.toml`'s ARC-66 note ("every Cargo.toml outside systems/"), AC-1's scans that key on `systems/`, the uv workspace members, `ci_removability.py`'s `cognition/lm-controller` deletion, and the third-party path overrides in `MODULE_SPEC.md` §3.2. Each of those is a contract edit, not a move. |

**Recommendation: Option 1.** It removes the clutter that has no architectural meaning (seven loose crates,
three scripts, a dead spike, a 5-file template folder) and leaves the root reading as the extension model:
one folder per pack type (`systems/`, `cognition/`, `worlds/`, `entities/`, `presentation/`) beside the
kernel (`crates/`) and the clients. Option 2 buys three fewer entries at the price of changing the
documented "install a System Pack" recipe that the frozen top-level criterion ("independently installable
interaction systems") is demonstrated with. If wanted, Option 2 is a later, separate PR on top of Option 1.

### 0.2 Freeze questions

| # | Question | Options | Recommendation and why |
| --- | --- | --- | --- |
| **Q-RL-1** | Where do the root-level Rust crates of the kernel and host (`contracts`, `kernel`, `persistence`, `packages`, `worldpack`, `authoring`, `server`) go? | **A** `crates/<name>/` · **B** a layer folder `kernel/` (the `kernel` crate becomes `kernel/core/`) · **C** leave | **A.** `crates/` is the dominant Rust monorepo convention (Cargo, rust-analyzer, ripgrep, Bevy, uv, ruff) and introduces no new MineWorld term. B reuses the defined term `Kernel` for a folder that also holds `server`. Crate *names* do not change. |
| **Q-RL-2** | Do `worlds/`, `entities/`, `presentation/` move under one folder? | **A** keep · **B** `content/…` | **A.** Each is a pack type; they are user-facing paths (`mineworld run worlds/market-town`, `--packs entities`, the guide, the Docker runtime image, the bundle layout of `ARC-80`, where `--root` "stands for the repository root"). |
| **Q-RL-3** | Where do `mineworld-2d`, `mineworld-slice`, `mineworld-3d` go? | **A** `bin/` · **B** inside their client folders · **C** leave | **A.** One folder instead of three files; quick start stays one short command (`bin/mineworld-2d`); no bash file inside a Godot project. None is obsolete (§2.4). |
| **Q-RL-4** | `spike/`? | **A** keep the evidence that live documents measure against — `FINDINGS.md`, `README.md`, `evidence/`, `unreal/` (incl. `probe.py`), `client-3d/main.gd` — under `docs/history/renderer-spike/`; delete the rest (`server/`, `client-2d/`, `client-3d/` project files, `run.sh`), citing it by permalink · **B** move all of `spike/` there · **C** keep | **A.** The code is not built or run by anything (§2.3), but `docs/references/UNREAL_ADAPTER_SPIKE.md` links `spike/unreal/probe.py` and measures against `spike/client-3d/main.gd` (578 lines), and 27 live files cite `spike/FINDINGS.md`. Keeping exactly what is cited keeps every link resolvable; the rest stays in history. |
| **Q-RL-5** | `packaging/` (5 templates: `about.hbs`, `about.toml`, `BUNDLE.toml.in`, `PLAY.txt`, `release-notes.txt.in`; read by `scripts/package.py` and `scripts/ci_release.py`)? | **A** `scripts/bundle/` · **B** `scripts/packaging/` · **C** leave | **A.** Data next to its two readers. Named `bundle`, not `packaging`, so that the folder is not an importable namespace package shadowing the `packaging` PyPI module on `sys.path[0]` for scripts in `scripts/`. |
| **Q-RL-6** | Does `scripts/` merge into `tools/`? | **A** keep at root · **B** `tools/ci/` | **A.** Every script computes `ROOT = Path(__file__).parent.parent` (depth bugs), ~325 code lines and every workflow change, and the active `ci/econ-*` lanes edit `scripts/`. Revisit later. |
| **Q-RL-7** | Are path mentions in history rewritten (merged PR designs and handoffs under `.structured-coding/plans/`, decision records under `docs/decisions/`, released `CHANGELOG.md` entries)? | **A** rewrite nothing there except Markdown link *targets* in plans; decision records and released CHANGELOG entries are untouched; the new layout record carries the old→new path map · **B** rewrite every mention · **C** rewrite nothing | **A.** Plans quote commands and evidence true when written. `docs/DECISIONS.md` §§1–3 says a record changes only by an appended dated note, so decision records are not edited at all — their stale paths resolve through the path map, which `docs/DECISIONS.md` links. Released CHANGELOG entries are published release history. |
| **Q-RL-8** | **(behaviour, blocker F-1)** `mineworld-packages` decides whether a System Pack is *bundled* (`ARC-54` point 2) by asking whether the pack's `CARGO_MANIFEST_DIR` lies under "this crate's manifest directory without its last component" (`packages/src/declared.rs:19-21`, `compiled_under`). Moved to `crates/packages`, that root becomes `<repo>/crates/`, every `systems/*` and `cognition/*` pack becomes third-party, and `resolve.rs:161` refuses every shipped world (`ThirdPartyNotRequired`). | **A** strip **two** components: the workspace root is the manifest directory of `mineworld-packages` minus `crates/packages`, stated as a named constant (`FRAMEWORK_DEPTH = 2`) with a dated note appended to `ARC-54` · **B** derive the root from Cargo (`[env] MINEWORLD_WORKSPACE = { value = "", relative = true }` in `.cargo/config.toml`) · **C** keep `packages` at depth 1 (an exception to `crates/`) | **A.** Same semantics, same `const fn`, one named number, no new mechanism. B fails for a third-party pack that builds `mineworld-packages` from a git checkout without MineWorld's `.cargo/config.toml` (`env!` would not compile; `option_env!` would silently misclassify). C keeps an unexplained odd one out at the root. A git checkout of the framework (`~/.cargo/git/checkouts/<repo>/<rev>/crates/packages`) still yields `<rev>/` as root, so third-party packs stay third-party. |
| **Q-RL-9** | Source edits beyond path strings | **A** I-RL-1 permits exactly: path strings/constants/comments/links, Q-RL-8's root-depth change, launcher root computation (`$here/..`), the CI cache-key salt (V-11) · **B** stricter: split Q-RL-8 into its own PR first | **A.** Q-RL-8 must land atomically with the move (before it, depth 2 is wrong; after it, depth 1 is wrong); splitting would need a transitional "either depth" rule, which is compatibility debt. |
| **Q-RL-10** | In-flight branches and frozen-but-unimplemented designs (S30g–k, S31c–h, S34c3–e and the open PRs) name old paths | **A** integrate main by **merge** (not rebase), bottom-up for stacks (§6); a frozen design's file-level scope at an old path is mapped through the path map as a **bounded mechanical correction** recorded in that PR's ledger, not a material deviation · **B** treat each as a deviation needing operator review | **A.** Path renames change no scope, invariant, contract or ownership; requiring per-PR operator review would stall every lane for no information. |
| **Q-RL-11** | CI spend for validation | **A** one full CI run (`milestone` label) on the final head + one `release.yml` `workflow_dispatch` dry run on Linux, macOS and Windows at the same head (V-7) · **B** full CI only | **A.** The release path (`package.py`, `ci_release.py`) runs in no PR layer; it is the one place the launcher/`packaging` moves can break silently. The dry run is three bundle jobs, once. |
| **Q-RL-12** | Commit boundary | **A** C1 (move + rewrites + Q-RL-8 + record), C2 (spike retirement) — fixed · **B** three commits with the record separate | **A.** The decision record and the `ARCHITECTURE.md` §14 update describe the tree C1 creates; shipping them in C1 keeps every commit's documentation true. C2 is separate so Q-RL-4 can be reverted alone. |

### 0.3 Operator rulings (binding)

| Date | Source | Ruling | Effect |
| --- | --- | --- | --- |
| 2026-11-27 | operator, relayed by the primary session | "仓库整理按推荐" ("organize the repository as recommended") | **Option 1** (39 → 29 root entries) and **every freeze question at its recommendation**: Q-RL-1 A, Q-RL-2 A, Q-RL-3 A, Q-RL-4 A, Q-RL-5 A, Q-RL-6 A, Q-RL-7 A, **Q-RL-8 A** (two-component framework root, `ARC-54` note), Q-RL-9 A, Q-RL-10 A, Q-RL-11 A, Q-RL-12 A. Option 2 is not adopted. |
| 2026-11-27 | primary session (scheduling) | Execute after #534 and the in-flight lanes merge; the primary schedules the quiet window | §6 timing; implementation base recorded in §10 at the window's start |
| 2026-11-27 | red-team PASS at `bba8033f` (non-blocking leftovers) | Keep the Windows-path and `repo2` test cases at the new depth; list the Q-RL-8 tests in I-RL-1's allowed edits | Applied: A-RL-3, I-RL-1 |

**Stays at root, no question needed** (tool or convention requires it): `Cargo.toml`, `Cargo.lock`,
`rust-toolchain.toml` (rustup searches upward for this exact name), `clippy.toml`, `deny.toml` (documented
defaults; `cargo-deny`'s non-root search was not verified), `pyproject.toml` + `uv.lock` (uv workspace root;
pyright runs from it), `Dockerfile` + `.dockerignore` (CI builds `context: .`), `README.md`,
`README.zh-CN.md`, `LICENSE`, `NOTICE`, **`CHANGELOG.md`** (read at `ROOT` by `ci_release.py`; convention),
**`CITATION.cff`** (GitHub reads it only at the root; `ci_release.py` validates it), `AGENTS.md`,
`CLAUDE.md`, and `.cargo/`, `.claude/`, `.github/`, `.structured-coding/`, `.gitattributes`, `.gitignore`.

`.DS_Store` is untracked and ignored; it exists only in the operator's root checkout
(`find . -name .DS_Store -delete` there). Nothing in this PR.

**Deferred** (not root clutter; own PRs if wanted): `clients/3d-spike/` → `clients/3d/` (~1 500
references); `tools/cli` → `crates/cli` (~2 800 `tools/` references); retiring `mineworld-3d`.

---

## 1. Before / after

### 1.1 Before (`git ls-tree --name-only df785c98`: 39 visible + 7 dot entries)

```text
.cargo/ .claude/ .github/ .structured-coding/ .dockerignore .gitattributes .gitignore
AGENTS.md  CLAUDE.md  README.md  README.zh-CN.md  LICENSE  NOTICE  CHANGELOG.md  CITATION.cff
Cargo.toml  Cargo.lock  rust-toolchain.toml  clippy.toml  deny.toml  pyproject.toml  uv.lock  Dockerfile
mineworld-2d  mineworld-3d  mineworld-slice                                  launcher scripts
authoring/ contracts/ kernel/ persistence/ packages/ worldpack/ server/      kernel + host crates
sdk/  systems/  cognition/  worlds/  entities/  presentation/
clients/  tools/  scripts/  packaging/  tests/  docs/  spike/
```

### 1.2 After, Option 1 (29 visible entries)

```text
.cargo/ .claude/ .github/ .structured-coding/ .dockerignore .gitattributes .gitignore
AGENTS.md  CLAUDE.md  README.md  README.zh-CN.md  LICENSE  NOTICE  CHANGELOG.md  CITATION.cff
Cargo.toml  Cargo.lock  rust-toolchain.toml  clippy.toml  deny.toml  pyproject.toml  uv.lock  Dockerfile

bin/            mineworld-2d  mineworld-slice  mineworld-3d          developer launchers
crates/         contracts kernel persistence packages worldpack      the kernel and its host
                authoring (+attributes) server
sdk/            rust/ python/                                        pack author's boundary
systems/        System Packs                                         unchanged
cognition/      Controller Packs                                     unchanged
worlds/         World Packs                                          unchanged
entities/       Entity Packs                                         unchanged
presentation/   Presentation Packs                                   unchanged
clients/        2d 3d-spike agent protocol runtime shared            unchanged
tools/          cli launch weather-fetch + authoring pipelines       unchanged
scripts/        CI and repository checks (+ bundle/)                 + release templates
tests/          acceptance support fixtures                          unchanged
docs/           specifications, guide, decisions (+ history/renderer-spike/)
```

The order reads `ENGINEERING_STANDARDS.md` §6's direction: `crates/` ← `sdk/` ← `systems/`, `cognition/` ←
data packs; `clients/` observe over the protocol; `tools/`, `scripts/`, `tests/`, `docs/` sit outside the
runtime.

### 1.3 After, Option 2 (alternative, 26 visible entries)

As 1.2, with `sdk/`, `systems/`, `cognition/`, `entities/` replaced by `packs/{systems,cognition,entities}/`
and `crates/sdk/{rust,python}/`. Not designed further unless chosen.

### 1.4 Path map (Option 1; recorded in the layout decision record)

| Old | New |
| --- | --- |
| `contracts/`, `kernel/`, `persistence/`, `packages/`, `worldpack/`, `server/` | `crates/<same>/` |
| `authoring/` (incl. `authoring/attributes/`) | `crates/authoring/` |
| `mineworld-2d`, `mineworld-slice`, `mineworld-3d` (as **paths** only, §2.5) | `bin/<same>` |
| `packaging/` | `scripts/bundle/` |
| `spike/FINDINGS.md`, `spike/README.md`, `spike/evidence/`, `spike/unreal/`, `spike/client-3d/main.gd` | `docs/history/renderer-spike/<same relative path>` |
| `spike/server/`, `spike/client-2d/`, `spike/client-3d/{project.godot,main.tscn,.gitignore}`, `spike/run.sh` | deleted; cited by permalink at C2's parent |

---

## 2. Audit at `df785c98`

### 2.1 Reference counts

`code/config` = tracked non-Markdown files except `Cargo.lock`; `live md` = Markdown outside
`.structured-coding/plans/` and `docs/decisions/`; `history` = those two trees. Pattern: path + `/`, not
preceded by a path character, optional `../` prefixes. Segmented joins (§2.2 K-3) are counted separately.

| Path | Files | code/config | live md | history |
| --- | ---: | ---: | ---: | ---: |
| `authoring/` | 51 | 18 | 3 | 284 |
| `contracts/` | 40 | 32 | 44 | 567 |
| `kernel/` | 52 | 32 | 33 | 673 |
| `persistence/` | 20 | 8 | 21 | 388 |
| `packages/` | 117 | 23 | 3 | 187 |
| `worldpack/` | 33 | 21 | 18 | 597 |
| `server/` | 69 | 75 | 72 | 943 |
| `spike/` | 35 | 37 | 33 (27 files cite `FINDINGS.md`) | 44 |
| `packaging/` | 5 | 3 | 0 | 12 |
| launchers (by name, path contexts only after §2.5) | 3 | ~100 | ~136 | — |
| **Option 1 total** | | **~350 + 10 segmented** | **~360** | **~3 700 (not rewritten, Q-RL-7)** |
| Option 2 adds: `systems/` / `cognition/` / `sdk/` / `entities/` | | 143 / 75 / 40 / 15 | 84 / 47 / 14 / 32 | 2 029 / 955 / 511 / 133 |

Estimated rewrite, Option 1: **~720 lines** in code, CI, specifications and READMEs, plus link targets in
plans, across roughly 250–300 files. The exact list is §10's first entry.

### 2.2 Reference kinds (each is a rewrite rule *and* a self-test class of the tool, §4)

- **K-1 Plain repo-relative strings** — `"server/PROTOCOL.md"`, `kernel/src/lib.rs` in Markdown, YAML,
  TOML, workflows. Rewritten by path map.
- **K-2 `CARGO_MANIFEST_DIR` joins inside moved crates** — 28 sites (e.g. `worldpack/tests/social_cafe.rs`
  `concat!(env!("CARGO_MANIFEST_DIR"), "/../worlds/social-cafe")`, `persistence/tests/kill_and_resume.rs`,
  `packages/tests/{depiction,far_field,place_asset,shells}.rs`, `authoring/attributes/tests/{structure,
  fidelity}.rs`, `worldpack/tests/entity_packs.rs`). Each gains one `../`. Includes **depth-only** joins
  with no moved name (`packages/tests/depiction.rs:302` `join("..")` then `root.join("presentation/…")`):
  the tool flags every `CARGO_MANIFEST_DIR` site in a moved crate for review, not only those naming a
  moved path.
- **K-3 Python segmented joins** — `REPOSITORY / "server" / "tests" / "frames"` and `parents[3] / "server"`
  in `sdk/python/tests/{scripted,test_golden_frames,test_session_state,test_attributes}.py`,
  `clients/agent/tests/{test_play_mcp,test_view}.py`, `cognition/lm-controller/tests/
  test_controller_parts.py`; `ROOT / "mineworld-slice"` (`scripts/ci_godot.py:357`); `ROOT / "packaging"`
  (`scripts/package.py:57`, `scripts/ci_release.py:435`). 10 sites. Matched as a segment sequence whose
  first segment follows a root expression (`ROOT`, `REPOSITORY`, `parents[n]`, `repository()`), never a bare
  `"server"` (a CLI subcommand elsewhere).
- **K-4 Rust reach-ins from outside the moved crates** — `tools/cli/tests/client_depiction.rs:99`,
  `client_far_field.rs:200,372` (`repository("packages/tests/fixtures/…")`), `request_rate_2d.rs:42`
  (`/../../server/tests/traces/…`), `tests/acceptance/tests/{ac1_composability,client_text,
  configuration_vocabulary,seam_vocabulary}.rs` (current-file paths; *not* the history-pinned ones, §2.5).
- **K-5 GDScript** — clients stay at depth 2, only targets change: `clients/protocol/checks/
  {delta_check,attributes_check}.gd`, `clients/3d-spike/scripts/checks/depiction_check.gd`,
  `clients/shared/checks/far_field_check.gd`.
- **K-6 Prefix lists** (a stale prefix scans nothing): `scripts/check_secrets.py:47-52`,
  `scripts/ci_changes.py` (layer classification, `CLIENT_FILES`, `SCOPE_FILES`), `scripts/check_scratch.py`,
  `scripts/ci_removability.py` `REMOVED_3D`, `scripts/check_client_rules.py:97`, `.dockerignore:21-22`
  (Docker does not prefix `**/`; becomes `bin/mineworld-3d`, `bin/mineworld-slice`; `bin/mineworld-2d`
  was never excluded and stays included).
- **K-7 Manifests and config** — root `Cargo.toml` (`members`, `[workspace.dependencies]`);
  `.cargo/config.toml` `[patch.crates-io]` (read by `tests/acceptance/tests/package_sources.rs`);
  `.gitignore` `!persistence/tests/fixtures/…`; `.structured-coding/standards.md:18`; two trybuild `.stderr`
  files (`$WORKSPACE/authoring/src/…`). `Cargo.lock` has no path entries and stays byte-identical.
- **K-8 `./`-prefixed launcher mentions** — README, docs, guide pages 06 and 14, and **Presentation Pack
  text**: `presentation/mineworld-default/3D/i18n/{en,zh_Hans}.po` and `presentation/mineworld-greybox/
  3D/i18n/*.po` (4 lines of user-visible `./mineworld-slice --world`). An exception to "presentation
  untouched" (I-RL-5).
- **K-9 Semantic path computation** — `packages/src/declared.rs` `PACKAGES_DIR` / `compiled_under`
  (Q-RL-8). The only one found; the tool cannot fix it, it is a reviewed source edit.
- **K-10 External, user-facing** — `docs/MODULE_SPEC.md` §3.2 (line ~582): third-party pack authors
  override `patch.crates-io.<crate>.path="../MineWorld/<crate>"`; `kernel`, `contracts`, `authoring` move
  to `../MineWorld/crates/<crate>`. Changed in the same PR; listed in the PR body and the layout record as a
  user-visible change (R-RL-3).
- **K-11 Specification** — `docs/ARCHITECTURE.md` §14 "Repository structure" is the existing layout
  authority; C1 updates it (no second authority).

### 2.3 `spike/` evidence

Code: own `[workspace]` in `spike/server/Cargo.toml`, not a root member, not built or run by any CI job,
workflow or script (`git grep spike/ -- .github scripts` → only `.dockerignore`, `.gitignore`); last changed
2026-10-07 (`37f37aa9`); "a throwaway proof … nothing here is production code" (`spike/README.md`).
Live citations: 27 files cite `spike/FINDINGS.md` (code comments in `contracts/src/{action,ids,
observation}.rs`, `server/src/{protocol,parity}.rs`, `systems/*`, `clients/protocol/**`, `sdk/python/**`,
`tools/cli/**`); `docs/references/UNREAL_ADAPTER_SPIKE.md` links `spike/unreal/probe.py` (line 74) and
measures against `spike/client-3d/main.gd` (lines 165, 407, 810, 844, 1002); `docs/HUMANOID_PROFILE.md`
cites `spike/server/src/world.rs` (→ permalink). Q-RL-4 A keeps every linked or measured file.

### 2.4 Launchers are not obsolete

`mineworld-2d` and `mineworld-slice`: README quick start, guide 06 and 14, `ci_godot.py:357`,
`ci_changes.py`, `ci_removability.py:49`, `check_client_rules.py:97`. `mineworld-3d`: `--drive`, `--shots`,
`--portrait` are the character-review path (`docs/HUMAN_REVIEW_QUEUE.md`, `docs/HUMANOID_PROFILE.md`).

### 2.5 Preserve list — never rewritten (tool rule and negative self-tests)

- **P-1 History-pinned literals.** `tests/acceptance/tests/precursor_vocabulary.rs:133-189` `ALLOWED`
  entries (`path: "authoring/src/section.rs"`, `"worldpack/src/format.rs"`, …) are matched against paths in
  *historical* diffs `base..M^2`; "an entry that matches nothing fails the test" (lines 30-33). Rewriting
  them fails the test. Likewise `ac1_composability.rs` `allowed()` examples judged against historical merge
  diffs (lines 171-192, 543-563). Preserved; the tool skips these line ranges by marker comment added in C1
  (`// relayout: preserve — historical diff paths`).
- **P-2 Launcher names used as names.** `scripts/package.py:112,114` `Client(…, "mineworld-2d", …)` /
  `"mineworld-3d"` (the pack stem `runtime/clients/<name>.pck`); log prefixes `"[mineworld-2d]"`,
  `"[mineworld-slice]"` in `clients/**/*.gd` (9 sites); `tools/launch` binaries `mineworld-2d-launch` etc.;
  `.pck`/`.settings.json` stems. The launcher map entry applies **only** in path contexts: a leading `./`,
  `ROOT /`, `$here/`, `bin/`-eligible list entries in `CLIENT_FILES`, `SCOPE_FILES`, `REMOVED_3D`,
  `check_client_rules.py`'s launcher list, and `.dockerignore`.
- **P-3 Decision records and released CHANGELOG entries** (Q-RL-7 A), incl. `CHANGELOG.md:37`
  `packaging/PLAY.txt`.
- **P-4 Words, not paths** — `mineworld-packages`, `mineworld-server`, "packages", "server", "kernel" in
  prose; `systems/installed/…`; `clients/3d-spike/…`; `.../server/` inside other paths.

---

## 3. Invariants

- **I-RL-1 Move only, with named exceptions.** No behaviour change. Permitted source edits: path strings,
  path constants, comments and links naming a moved path; **Q-RL-8's root-depth change** (the one
  semantic edit, preserving `ARC-54` point 2's meaning) **and its tests** — the existing
  `packages/tests/identity.rs` `bundled_means_compiled_under_the_framework_workspace` cases moved to the
  new depth, plus A-RL-3's new cases and integration test; launchers' root computation (`$here/..`); the
  CI cache-key salt (V-11); the P-1 preserve marker comments. No crate renamed; no dependency, version or
  feature change.
- **I-RL-2 Same build graph.** `Cargo.lock` byte-identical. `cargo metadata --format-version 1` equal to
  base after normalising every `manifest_path` / path-dependency `path` through the path map.
- **I-RL-3 Same bundled/third-party classification.** Every pack bundled at base is bundled at head; a
  pack compiled outside the workspace is third-party at both.
- **I-RL-4 No silent skips.** Executed cargo tests, pytest cases and Godot check cases, and every
  scanner's scanned-file set, equal base's mapped through the path map.
- **I-RL-5 User-facing paths unchanged except** the launchers (`bin/…`, incl. the four `.po` lines, K-8)
  and the third-party `patch.crates-io` override paths (K-10). `worlds/`, `entities/`, `presentation/`
  layout, `clients/`, the bundle layout and every guide command other than the launcher are untouched.
- **I-RL-6 No old path in live surfaces.** `relayout.py --check` finds no old path outside P-1 … P-4 and
  the path map.
- **I-RL-7 Kernel ignorance and dependency direction unchanged** — placement only.

---

## 4. Rewrite tool and the reuse question (`CLAUDE.md` §4.16)

`scripts/relayout.py` (committed in C1; removal condition in §6): path map; rules K-1 … K-8 with
anchored, context-aware patterns; preserve list P-1 … P-4; depth adjustment for files that themselves
moved (K-2); `--list` (every site by kind), `--check` (exit 1 if a rewrite remains), `--exists` (V-10),
`--normalise-metadata` (V-1), `--links` (V-5), `--reverse` (rollback). Idempotent. Self-tests: at least one
positive and one negative line per K and P class, taken from the real sites in §2.

Reuse comparison (recorded in the layout decision record):
- **`fastmod`** (regex replace across a tree): no preserve ranges, no depth-aware `..` adjustment, no
  `--check`/`--exists`/metadata modes; would still need a wrapper holding all the rules.
- **`comby`** / **`ast-grep`**: structural matching needs a grammar per language; the sites span Rust,
  Python, GDScript, bash, TOML, YAML, Markdown, `.po` and Dockerignore — GDScript and `.po` have none.
- **`git grep` + hand edits**: what the tool does, but not repeatable by ~40 in-flight branches.
Our own ~300-line script is therefore justified; it is temporary.

---

## 5. Commit plan

### C1 — `chore(layout): crates/, bin/, scripts/bundle/; framework root depth; layout record`

- **Goal.** The §1.2 tree (minus the spike change), every live reference rewritten, Q-RL-8 applied, and the
  layout documented.
- **Scope.** `git mv` per §1.4; K-1 … K-8 sites; `packages/src/declared.rs` (Q-RL-8 A: `FRAMEWORK_DEPTH`
  constant, doc comment, `compiled_under` strips that many components; its unit tests); launchers' root
  computation; `.github/actions/layer/action.yml` cache-key salt; P-1 marker comments;
  `docs/ARCHITECTURE.md` §14 rewritten to the new tree with the layout rule; new decision record
  `docs/decisions/ARC-<next>.md` ("Repository layout": the rule "the root holds tool-required files and
  one folder per architectural layer or pack type; a new top-level entry is a reviewed decision", the path
  map, the Q answers, the §4 reuse comparison) and regenerated index; a dated note appended to `ARC-54`
  (root depth); `docs/DECISIONS.md` links the path map; `docs/MODULE_SPEC.md` §3.2 override paths;
  README quick start + a four-line folder map linking `ARCHITECTURE.md` §14; `CLAUDE.md` §1 "current
  repository state" sentence points to the record; `scripts/relayout.py`.
- **Non-goals.** No spike change (C2); no `clients/3d-spike` rename; nothing in P-1 … P-4.
- **Implementation.**
  - [ ] `relayout.py` with self-tests (§4); self-tests pass.
  - [ ] Capture base evidence on the implementation base (V-1 metadata, V-3/V-6 counts source run,
        V-4 scanner sets, V-10 literal list).
  - [ ] `git mv`; run the tool; review `--list` by kind; hand-fix unclassified sites; record each in §10.
  - [ ] Q-RL-8 edit + tests (A-RL-3).
  - [ ] Launchers; `git ls-files -s bin/` shows `100755` for all three.
  - [ ] Docs: ARCHITECTURE §14, ARC record + index, ARC-54 note, MODULE_SPEC §3.2, README, CLAUDE.md §1.
- **Deterministic validation.** [ ] V-1, V-2, V-4, V-5, V-10, V-12 locally; [ ] V-3, V-6, V-8, V-9, V-11
  in CI on the final head; [ ] V-7 release dry run (3 OS).
- **LLM logic review.** [ ] An independent reviewer reads the full diff minus pure renames
  (`git diff -M --diff-filter=M`) against I-RL-1 … I-RL-7, checks every non-path edit is on the I-RL-1
  list, re-derives Q-RL-8's root on a git-checkout path and a sibling-repository path, and samples ten
  rewritten sites per K class; findings and dispositions in §10.
- **Acceptance.** A-RL-1 … A-RL-9.
- **Failure handling.** Any S-n (§8) stops the commit.

### C2 — `chore(spike): retire the renderer spike; keep cited evidence under docs/history/`

- **Implementation.**
  - [ ] `git mv` the Q-RL-4 A keep-set to `docs/history/renderer-spike/`; `git rm` the rest.
  - [ ] `docs/history/renderer-spike/README.md`: what it proved, the commit where the code last existed,
        `git show <C2^>:spike/…` to recover it.
  - [ ] Rewrite the 27 `spike/FINDINGS.md` citations, `UNREAL_ADAPTER_SPIKE.md`'s links and measurements,
        `HUMANOID_PROFILE.md` (permalink); drop `spike/` lines from `.gitignore`, `.dockerignore`.
- **Deterministic validation.** [ ] `relayout.py --check` (spike map) exit 0; [ ] `--links` no new broken
  link; [ ] `scripts/check_doc_headings.py` passes over `docs/history/**`; [ ] `.gitignore`/`.dockerignore`
  diff is exactly the removed spike lines; [ ] `cargo metadata` unchanged by C2 (spike was never a member).
- **LLM logic review.** [ ] Reviewer confirms every deleted file is uncited by a live document (grep at
  C2) or cited by permalink, and that the keep-set matches Q-RL-4.
- **Acceptance.** A-RL-10.

Each commit is independently reviewable and revertible; nothing else rides along.

---

## 6. Timing, in-flight branches, rollback

**Timing.** After the stack #534 → #540 → #541 (`ci/econ-*`) merges or reaches a checkpoint, and before the
next implementation batch is dispatched. A short merge freeze on `main` from the final CI run to the merge;
any merge inside it means re-running the tool and V-1/V-2/V-10.

**Recipe for an in-flight branch** (in that branch's own worktree, by its owner; **merge, not rebase**, so
recorded head SHAs and earlier conflict resolutions survive and no force-push is needed):

```sh
git status --short                                   # must be clean before starting
git fetch origin
git -c merge.directoryRenames=true -c merge.renameLimit=0 merge origin/main
#   directoryRenames=true: files the branch ADDED under kernel/ land in crates/kernel/ (default stops to ask)
python3 scripts/relayout.py                          # rewrite path strings the branch introduced (idempotent)
python3 scripts/relayout.py --check                  # exit 0
git status --short                                   # review; stage tracked changes only
git add -u                                           # plus any new file by name; never `git add -A`
git commit -m "chore(layout): map this branch's paths after the crates/ move"
git status --ignored --short -- authoring contracts kernel persistence packages worldpack server packaging spike
#   remove leftover ignored/untracked folders at the old paths (caches, local *.sqlite, __pycache__)
cargo check --workspace                              # then the branch's own validation
```

**Stacks** are integrated bottom-up: main → #534; then #534's head → #540; then #540's head → #541 (each by
the same merge + tool steps). If a lane insists on rebasing, `git rebase --update-refs` on the top branch
only, with the owner's agreement.

**Frozen designs not yet implemented** (Q-RL-10 A): the implementer maps the design's old paths through the
path map and records "paths mapped per repo-layout ARC-<next>" in that PR's ledger.

**Tool retirement.** `scripts/relayout.py` is removed by a follow-up PR owned by the primary session when
no open PR and no `mineworld-worktrees/*` branch ahead of main predates the move, or 30 days after the
merge, whichever is later; the path map stays in the record permanently.

**Rollback.** `git revert -m 1 <merge>` restores the old tree (Q-RL-8 reverts with it, atomically); merged-in
branches re-run the tool with `--reverse`. C2 reverts alone.

---

## 7. Validation and acceptance

Local runs follow R-LOAD-1 (no local full-workspace test runs while other lanes are active): cheap checks
locally, the full suite in CI on the exact head. Local and remote results are recorded separately in §10.

- **V-1 Build graph (local).** `relayout.py --normalise-metadata` on base and head `cargo metadata`;
  diff empty. `git diff --quiet <base> HEAD -- Cargo.lock`. `cargo check --workspace --all-targets`,
  `cargo fmt --all --check`.
- **V-2 Old-path scan (local).** `relayout.py --check` exit 0.
- **V-3 Same tests executed (CI).** Baseline: a CI run of the same layers, same runner (cargo test or
  nextest, whichever main uses then), on a **named** main SHA taken when the merge freeze opens. Equal
  per-binary cargo test counts; equal count of `SKIPPED:` lines; equal pytest *collected* and *passed*
  counts for `sdk/python`, `cognition/lm-controller`, `clients/agent`, `tools/*` suites.
- **V-4 Scanners scan the same files (local).** `check_secrets.py`, `check_scratch.py`,
  `check_client_rules.py`, `ci_changes.py` self-tests, and `ci_changes.py` on a synthetic "every tracked
  file changed" diff: layer selections and scanned sets equal after mapping.
- **V-5 Links (local).** `relayout.py --links`: no relative Markdown link broken at head that resolved at
  base. Guide tests (`tools/cli/tests/guide*`) in CI.
- **V-6 Godot (CI).** `ci_gdparse.py`; `delta_check`, `attributes_check`, `far_field_check`,
  `depiction_check`: pass lines present **and** case counts equal to base (a check whose corpus is empty
  still reports its synthetic case, so a present pass line alone proves nothing).
- **V-7 Release dry run (CI, milestone item).** `release.yml` `workflow_dispatch` (its `--dry` path) at the
  exact head on Linux, macOS and Windows. Compare with a dry run at the baseline SHA: identical bundle file
  lists; identical `.pck` stems (`mineworld-2d`, `mineworld-3d`); identical digests for every non-binary
  file; rendered release notes identical; `ci_release.py` CITATION/CHANGELOG checks pass. Binary digests
  are excluded (they embed source paths and *will* change).
- **V-8 Removability (CI).** `ci_removability.py --target 3d --no-godot` and `--target lm`.
- **V-9 Docker (CI).** Image build succeeds; runtime image file list unchanged.
- **V-10 Path literals still resolve (local).** `relayout.py --exists`: every quoted repo-relative path
  literal in `.rs`, `.py`, `.gd`, `.sh`, launchers, workflows and YAML that **exists at base** maps to a path
  that **exists at head**. Catches early-return reads (`client_text.rs:412`, `attribute_consumers.rs:45`
  `let Ok(..) = read_dir(..) else { return; }`, `check_secrets.py:130` `is_file()`).
- **V-11 CI cache.** `.github/actions/layer/action.yml:43` key gains a layout salt (e.g. `layout-2`) once,
  so the first run after the merge does not restore a `target/` keyed to old paths forever (the key would
  otherwise never change, since `Cargo.lock` is identical). Recorded: first post-merge run's cache miss and
  save.
- **V-12 Repository shape (local).** `git ls-tree --name-only HEAD` equals §1.2's entry list;
  `git ls-files -s bin/` all `100755`.

Acceptance:
- **A-RL-1** V-12 holds.
- **A-RL-2** V-1 empty diffs; `Cargo.lock` identical.
- **A-RL-3** Q-RL-8: `packages` tests assert `compiled_under` is true for `<repo>/systems/presence` and
  `<repo>/cognition/decision` under `<repo>/crates/packages`, false for `<repo>2/systems/x` and for
  `/home/u/.cargo/git/checkouts/other/abc/systems/x`. Every existing case of
  `packages/tests/identity.rs::bundled_means_compiled_under_the_framework_workspace` is kept at the new
  depth, none dropped: `/a/repo2/systems/x` (sibling sharing the root's name as a prefix) and `/a/rep`
  stay false; the Windows case becomes `compiled_under(r"C:\w\repo\systems\x", r"C:\w\repo\crates\packages")`
  true, with a Windows `repo2` sibling (`r"C:\w\repo2\systems\x"`) false; the no-separator case and the
  `/elsewhere/acme` third-party `Package` stay; `/a/repo/crates/packages` itself is bundled. Then an
  integration test resolves `worlds/market-town`
  and `worlds/lakeside` with every enabled System Pack reported `bundled: true` (no `ThirdPartyNotRequired`).
- **A-RL-4** V-2 exit 0; V-10 no missing path.
- **A-RL-5** V-3 counts equal (cargo, `SKIPPED:`, pytest).
- **A-RL-6** V-4 equal sets; V-6 pass lines and case counts equal.
- **A-RL-7** V-5, V-8, V-9 pass.
- **A-RL-8** V-7 equal on all three OSes.
- **A-RL-9** V-11 recorded.
- **A-RL-10** C2's checks pass; no live link to a deleted spike file.

Classification `PASS` / `FAIL` / `INCONCLUSIVE` from evidence only.

---

## 8. Material stops (stop the dependent action; return to the operator with evidence)

- **S-1** Any source edit outside I-RL-1's list is needed.
- **S-2** A second semantic path computation like K-9 is found.
- **S-3** `Cargo.lock` changes, or V-1 metadata differs beyond path normalisation.
- **S-4** Any count in V-3/V-6 drops, or a scanner set in V-4 shrinks, and the cause is not a rewrite bug
  fixable within scope.
- **S-5** A P-1 history-pinned literal would have to change.
- **S-6** A user-facing path beyond I-RL-5's exceptions changes, or the bundle layout (V-7) differs.
- **S-7** CI spend beyond Q-RL-11's runs (e.g. repeated three-OS dry runs after failures: more than two
  attempts).

Bounded and autonomous (record in §10): rewrite-tool rule fixes, hand-fixes of individual sites,
additional preserve markers for history-pinned literals of the P-1 kind, doc wording.

---

## 9. Execution contract

```text
PROJECT / PR:           repo-layout PR RL1 — repository top-level layout (Option 1, ruled 2026-11-27)
PRIMARY DESIGN DOC:     .structured-coding/plans/repo-layout/pr-rl1-top-level-layout.md
RELATED / BINDING DOCS: CLAUDE.md §§2–4; docs/ARCHITECTURE.md §14; docs/ENGINEERING_STANDARDS.md §6;
                        docs/MODULE_SPEC.md §§3.1–3.2; docs/DECISIONS.md; decisions ARC-33, ARC-54,
                        ARC-66, ARC-80; docs/REUSE_POLICY.md; _reviews/repo-layout.md
IMPLEMENTATION BASE:    design base 303c0df3 (origin/main at freeze); executed on origin/main at the
                        start of the quiet window the primary schedules after #534 and the in-flight
                        lanes merge; that exact SHA recorded in §10 and the counts re-measured
APPROVED SCOPE:         §1.4 moves; §2.2 rewrites; Q-RL-8 A; §5 C1 docs; C2 spike retirement;
                        scripts/relayout.py. Nothing in §0 "Deferred".
FROZEN INVARIANTS:      I-RL-1 … I-RL-7; preserve list P-1 … P-4
APPROVED SEQUENCE:      base evidence → C1 (implement, validate, LLM review, commit) → C2 (same) →
                        CI milestone run + V-7 → READY FOR OPERATOR REVIEW
AUTONOMOUS BUDGET:      local: cargo check/fmt/clippy single-job, tool self-tests, scanners, link and
                        literal checks — unrestricted; no local full-workspace test runs (R-LOAD-1).
                        CI: one milestone full run + one 3-OS release dry run at the final head, plus
                        baseline equivalents at the named main SHA; at most two re-runs of a failed
                        job (S-7). No paid API calls.
REQUIRED LIVE DOCS:     this file (§10 ledger)
HANDOFF FILE:           .structured-coding/plans/repo-layout/handoff.md
WORKTREE:               /Users/yuema137/mineworld-worktrees/impl-repo-layout (one owner; never /tmp)
MERGE:                  operator authorization only
```

---

## 10. Evidence ledger (filled during implementation)

| Item | Local | Remote (CI, exact head) |
| --- | --- | --- |
| Base SHA / baseline SHA | — | — |
| Rewrite list by kind (counts) | — | — |
| `CARGO_MANIFEST_DIR` sites (list) | — | — |
| Hand-fixed sites | — | — |
| V-1 … V-12 | — | — |
| LLM review C1 / C2 (findings, dispositions) | — | — |

---

## 11. Risks and red-team trace

- **R-RL-1 Silent skips / wrong prefixes / early returns** → V-3, V-4, V-6 counts, V-10.
- **R-RL-2 Bundled-pack reclassification** (F-1) → Q-RL-8, I-RL-3, A-RL-3.
- **R-RL-3 User-visible path changes**: launchers (README, guide, `.po`, videos, progress page) and
  third-party `patch.crates-io` overrides (MODULE_SPEC §3.2). No root shim (no early compatibility debt,
  `CLAUDE.md` §4.12); listed in the PR body and the record; progress page updated on merge.
- **R-RL-4 Over-matching** → P-1 … P-4, negative self-tests, `--list` review.
- **R-RL-5 In-flight branches** (~40 worktrees, stacks) → merge recipe, bottom-up stacks, Q-RL-10.
- **R-RL-6 CI cost after merge** → V-11 salt.
- **R-RL-7 Windows** → longest moved path grows 96 → 103 characters (repository maximum stays 120); no case
  collisions; `100755` checked (V-12) because `core.fileMode=false` checkouts would not repair a lost bit.

| Red-team finding | Addressed in |
| --- | --- |
| F-1 bundled-pack root | Q-RL-8, Q-RL-9, I-RL-1, I-RL-3, A-RL-3, K-9 |
| F-2 stale/incomplete audit | new base `df785c98`; §1.1–1.2 (`CHANGELOG.md`, `CITATION.cff`); Q-RL-5 (5 files, two readers); K-2 depth-only, K-3, K-4, K-8 `.po`, K-10 MODULE_SPEC, K-11 ARCHITECTURE §14; §2.3 27 citations |
| F-3 history-pinned / name tokens | §2.5 P-1, P-2; §4 self-tests |
| F-4 spike links | Q-RL-4 A keep-set; §2.3; C2 |
| F-5 verification gaps | V-7 (3-OS dry run, non-binary digests), V-3 pytest, V-6 case counts, V-10, V-12 `git ls-tree` |
| F-6 cache / baseline | V-11; V-3 named baseline SHA |
| F-7 minor | `standards.md` in K-7; CHANGELOG in P-3/Q-RL-7; decision records untouched (Q-RL-7); `scripts/bundle/` (Q-RL-5); site list in §10; `.dockerignore` in K-6 |
| F-8 recipe | §6 merge-based, stacks, `git add -u`, stale-folder cleanup, Q-RL-10, retirement condition, `100755` |
| F-9 missing decisions | Q-RL-8 … Q-RL-12; CHANGELOG/CITATION at root; `.po` and MODULE_SPEC in I-RL-5 |
| F-10 process | §9 contract, §8 stops, per-commit validation + LLM review items in §5, fixed boundary (Q-RL-12), §4 reuse note |
