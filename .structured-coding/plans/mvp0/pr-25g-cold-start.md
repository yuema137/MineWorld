# PR 25g — The onboarding cold-start check, layers A and B

**Lifecycle:** `DESIGN FROZEN 2026-10-27 (operator approved in the primary session: every question answered as recommended) (not frozen)`. Written 2026-10-26 by a design session for the release lane. Nothing here
authorizes implementation, a workflow run that costs money, a tag, a release or a merge. Only the operator
freezes this design.
**Author / worktree:** `/Users/yuema137/mineworld-worktrees/design-25g-cold-start`, branch
`docs/pr-design-25g-cold-start`, from `origin/main @ 0b2faf2f`.
**Effort:** `mvp0` · **Parent:** [`step-25-creator-docs.md`](step-25-creator-docs.md) (S25; this is a new PR
after 25f) · **Siblings:** [`step-14-ci.md`](step-14-ci.md) (CI architecture), [`step-23-release.md`](step-23-release.md)
(R-b's release workflow, R-d's publishing), `pr-s23-rb-release-workflow.md` (R-b, drafted in parallel; not read
here because it does not exist on `main` yet).
**Binding parents:** `CLAUDE.md` §§2, 3, 4 (items 8, 9, 10, 16); `docs/GUIDE_FORMAT.md`; `docs/decisions/ARC-48.md`
(CI layers, "check names are an interface"), `ARC-83` (the nightly), `ARC-154` (speed first), `DEP-19` (pinned
runner labels); step-25 INV-S25-3 (the guide executes) and INV-S25-6 (no paid model in CI, ever).
**Background for humans:** the progress page, section 新用户上手检验
(`/Users/yuema137/mineworld-demos/progress/index.html#onboard`).

Every external fact below carries "read 2026-10-26" and its source (§13). A fact not yet confirmed from a primary
source is marked **verify**, and the commit that first relies on it confirms it.

---

## 0. The answer in brief

1. **What is missing today.** The guide test already runs every guide page on Linux, macOS and Windows. But it
   runs on a CI machine that already has Rust, Git LFS and a warm build. Chapter 01's `git clone` reads the local
   checkout, not GitHub. And every `mineworld` command runs the test's own debug binary, not the one the reader
   built (`tools/cli/tests/guide/command.rs:14`, `:51`, `:145-167`). So no test shows that a stranger with a
   clean computer can follow the guide from the first line.
2. **Layer A (from source).** A new workflow, `.github/workflows/cold-start.yml`. On each OS it takes a fresh
   hosted runner, removes the tools the image preinstalls (Rust, Git LFS), checks that they are gone, and then
   lets the guide runner follow the pages: install the prerequisites with chapter 01's own commands, really clone
   `github.com/yuema137/MineWorld`, build with the reader's command, and run chapters 01–06 and 14 (and the pages
   14 continues from) with the **reader's** `target/release/mineworld`. It records the time of every chapter.
3. **Layer B (from the download).** In the same workflow, a job takes the bundle that the release workflow
   uploaded, unpacks it on a runner where Rust and Godot are absent (asserted), and runs chapters 02–06 with the
   bundled `runtime/mineworld`.
4. **Reuse.** No new Markdown parser. The existing runner (`tools/cli/tests/guide/`) gets a **cold-start mode**:
   the binary is a parameter instead of a compile-time constant, shell blocks run with the reader's real
   environment, and each page's time is recorded. The runner is built once per OS in a helper job and copied, as
   one executable file, onto the clean runner. The clean runner never compiles test code.
5. **Operator-only blocks.** No paid model ever (INV-S25-6). Chapter 06's server is run headless and stopped;
   06's 2D window is replaced on Linux and macOS by the 2D client's own `--drive=walk` check against the reader's
   server. Chapter 14's agent is replaced by the recorded agent session, replayed through the real MCP adapter
   against the reader's server (`clients/agent/tests/test_agent_session.py`). Each substitute is listed in the
   report as a substitute, never as the page's block passing.
6. **Architecture verdict: this is not "only adding jobs".** It adds a workflow with its own triggers, a
   runner-mutating "make clean" step, a new mode of the guide runner (a `GUIDE_FORMAT.md` change), a new
   directive, and an artifact contract with `release.yml`. It needs an ARC decision record and an `ARC-48` note
   (§3).
7. **Never a required check.** Triggers: a weekly schedule, `workflow_dispatch`, and a dispatch before every tag
   (§7). No cache in layer A's clean jobs. Minutes are free on this public repository; the real costs are macOS
   runner slots (cap 5) and about 0.63 GiB of Git LFS bandwidth per run (§10).
8. **Pass bar (proposed):** all three OS green in A and in B, and zero-to-Corner-Shop (clean runner → end of
   chapter 05) within **60 minutes of machine time** on each OS.

---

## 1. The requirement

### 1.1 The operator's brief, as relayed by the release lane lead (2026-10-26)

- **Layer A.** A CI job on macOS (`macos-26`), Linux (`ubuntu-24.04`) and Windows (`windows-2025`). It starts from
  a clean runner, installs the documented prerequisites exactly as chapter 01 says, really clones
  `yuema137/MineWorld` from GitHub with LFS (`git lfs pull`), builds with the reader's own commands, and follows
  `docs/guide` 01–06 and 14 exactly as written, using the binary the reader built. It records the time per chapter.
  Proposed pass bar: all three green; zero to Corner Shop within a stated machine-time budget (the page suggests
  ≤ 60 min).
- Hosted images preinstall Rust, rustup, git-lfs and more. The design must say how the runner becomes "clean" for
  the prerequisites, and what is not achievable.
- Operator-only blocks: no paid model in CI ever. A headless substitute, or "not covered".
- **Layer B.** A job takes a packaged bundle from a release-workflow artifact and runs it on a runner with no Rust
  and no Godot (both asserted absent), following 02–06 with the bundled `mineworld`. The artifact interface is a
  dependency on R-b.
- Decide whether this only adds jobs or touches CI architecture. Triggers: weekly + dispatch + before each tag.
  Never a required PR check. No cache in layer A. Cost per run. Per-chapter timings in the job summary.
- Prefer extending the existing guide runner over a new parser.

### 1.2 Standing rules this PR must satisfy

- **INV-S25-3 (the guide executes)** stays true and unchanged in the default mode: the PR changes no verdict of
  `the_guide_in_this_repository_runs_green` (`tools/cli/tests/guide.rs:786`) except the reviewed edit of
  `expected_run.toml` for any new non-running block (§5.4).
- **INV-S25-6 (no paid model in CI).** Nothing in this PR starts a language model.
- **ARC-48: no test reads a CI variable to relax itself.** The cold-start test is a separate, ignored test that
  fails (never skips) when its configuration is missing. Reading the configuration selects what to run; it never
  relaxes a check.
- **I-S13-7:** `permissions: contents: read` (plus `actions: read` where an artifact of another run is read);
  every third-party action pinned to a full SHA (`scripts/check_ci_pins.py:44-58`).
- **I-S13-9 spirit:** the workflow names subcommands of one script, `scripts/ci_cold_start.py`; no check command is
  written inline in YAML.
- **Every platform** (`overall.md` §5): macOS, Linux and Windows, with pinned labels (`DEP-19`).

---

## 2. Audit — what exists today (`origin/main @ 0b2faf2f`)

### 2.1 The guide runner

| What | Where | What it means for 25g |
| --- | --- | --- |
| `run(label, guide, repo)` takes the guide folder and the repository as paths | `tools/cli/tests/guide/run_guide.rs:20` | An external checkout already works: pass the clone's `docs/guide` and the clone's root. |
| Pages in run order: root pages by file name, then `compare/` | `run_guide.rs:66-105` | B needs a page filter (02–06). A runs everything 14 depends on. |
| One "reader's home" per run, a scratch folder | `run_guide.rs:120-127` | Cold-start mode uses the runner's real home instead (rustup installs there; the clone lands in `$HOME/MineWorld`). |
| `continues` chains pages; a page waits for the page it continues | `run_guide.rs:151-170` | Chapter 14 continues 12, 12 continues 08, 08 continues 07, 07 continues 05 (`grep continues docs/guide/*.md`). So "01–06 and 14" means running 01–08, 12 and 14 at least. |
| `BINARY = env!("CARGO_BIN_EXE_mineworld")`, a compile-time constant | `command.rs:14` | Every console command and every feature probe runs this binary (`command.rs:51`, `run_guide.rs:281-293`). Cold-start mode must make it a run-time parameter. |
| Console commands run the binary directly, no shell, no `PATH` lookup | `command.rs:48-74` | The reader's `PATH` line in chapter 01 is never used by the runner. Cold-start mode passes the reader's binary by path and checks that the path is the one chapter 01 names. |
| Shell blocks: `HOME` = scratch home; `CARGO_HOME`/`RUSTUP_HOME` = the test process's; `CARGO_TARGET_DIR` = `guide-build/`; `url.<repo>.insteadOf` the GitHub address; `safe.directory=*`; `GIT_LFS_SKIP_SMUDGE=1` | `command.rs:136-167` | Each of these is exactly what makes today's run "warm". Cold-start mode sets none of them. |
| Shell blocks: `sh -e FILE` or `powershell -NoProfile … -File FILE` | `command.rs:112-133` | A new process per block. After an installer changes `PATH` in a profile or the registry, the next block does not see it. Cold-start mode starts each block as "a new terminal" (§5.2). |
| `operator-only` → `Pending("operator-only")`, never run | `page_run.rs:93`; parser `page.rs:226` | Unchanged. Substitutes run outside the page (§6). |
| `Summary` counts, pending list, `checks` | `report.rs:16-30` | Gains per-page wall time. |
| The real guide's committed run: `not_run` list and `minimum_passed = 164` | `tools/cli/tests/guide/expected_run.toml`; read at `guide.rs:822` | Cold-start mode gets its own committed expectation (§5.4). |
| Scratch folders under `env!("CARGO_TARGET_TMPDIR")` | `tests/support/src/lib.rs:43-50`, created with `create_dir_all` (`:161-162`) | A test executable copied to another runner still works if that path can be created; cold-start mode passes its own scratch root (§5.3). |
| `GUIDE_FORMAT.md` §4.6 rule 4–5: the clone reads the repository under test; the build goes to `guide-build/` | `docs/GUIDE_FORMAT.md:186-195` | 25g adds §4.7 "cold-start mode", which states the opposite for that mode. |

### 2.2 The pages in scope

- **Chapter 01** (`docs/guide/01-get-mineworld.md`). Step 1.1 says "You need Git and Rust" with links only
  (`:12-13`). It gives **no install command** for either. It does **not** mention Git LFS or a C compiler. Its
  shell blocks clone and build (`:16-30`), then put `target/release` on `PATH` (`:38-46`).
- **README quick start** (`README.md:176-191`) asks for Rust, Godot 4.7 and Git LFS, with per-OS LFS commands
  (`brew install git-lfs`, `sudo apt install git-lfs`, `winget install GitHub.GitLFS`), then `git lfs install`,
  clone, `git lfs pull`.
- **Chapters 02–05** are file blocks and console blocks only. 05 ends with `equals corner-shop
  docs/guide/examples/corner-shop` (`05-one-shop.md:382`).
- **Chapter 06**: 6.1 is a console block. 6.2 (`mineworld server corner-shop --town`) and 6.3 (`./mineworld-2d
  --server … --invite … --seat ben`, needs Godot 4.7 on `PATH`) are `operator-only` (`06-play-it.md:35`, `:66`).
- **Chapter 12** copies `$HOME/MineWorld/entities/modern-goods` from the clone (`12-packs.md:229`, `:238`).
- **Chapter 14**: 14.1 is a console block; 14.2–14.10 are `operator-only` (18 entries in `expected_run.toml`).
  14.3 needs `uv` and runs `uv sync` in `$HOME/MineWorld` (`14-play-as-an-agent.md:59-83`).

### 2.3 What already exists for the operator-only parts

- **06.3:** `./mineworld-2d --drive[=scenario]` runs the real 2D client headless and "exits non-zero on failure"
  (`mineworld-2d:19-20`). The `walk` scenario derives every waypoint "from the disclosed passages"; "no scenario
  names a world's coordinates" (`clients/2d/scripts/harness/drive.gd:20-31`). So it is not tied to Market Town.
  Whether `walk` passes on Corner Shop seated as `ben` is **verify** (C5).
- **14:** `clients/agent/tests/agent_session.py` replays the recorded session `docs/guide/examples/agent-session.json`
  against "a fresh real server", through the adapter "as a child process over stdio, reached through the official
  MCP client" (`agent_session.py:1-12`). `realserver.py:60-63` takes the binary from `MINEWORLD_BIN`. A missing
  binary fails, never skips (`clients/agent/tests/conftest.py:24-32`). This is a deterministic, model-free agent.
  It plays the committed `docs/guide/examples/corner-shop`, not the reader's chapter-12 folder (a stated gap,
  §6.3).

### 2.4 CI today

- `ci.yml` runs on `push` to `main` and `scratch/**`, on `pull_request`, and on dispatch (`ci.yml:31-44`). A push to
  any other branch prefix does not start it.
- `nightly.yml` runs daily at 10:17 UTC and on dispatch; no job is required (`nightly.yml:1-35`).
- Godot is installed by the local action `./.github/actions/godot` (pinned version and checksum, `scripts/ci_godot.py`).
- Native legs use `./.github/actions/native`, which restores the Cargo cache (`.github/actions/native/action.yml`).
  Layer A's clean jobs must not use it.

### 2.5 The release dry run (PR #498, draft, branch `release-dry-run/workflow`, head `5cb3c313`)

Read with `gh pr diff 498` on 2026-10-26. Not touched.

- File `.github/workflows/release.yml`, `name: Release dry run`, triggers `workflow_dispatch` and `push` to
  `release-dry-run/**`; `permissions: contents: read`.
- One job `bundle`, matrix `linux-x86_64` (`ubuntu-24.04`), `windows-x86_64` (`windows-2025`), `macos-universal`
  (`macos-26`). It checks out with `lfs: true`, builds, exports, runs `package.py assemble` and `package.py probe`.
- **Artifact:** `name: MineWorld-${{ matrix.target }}`, path = the one archive
  `MineWorld-<version>-<target>.zip|.tar.gz`, `retention-days: 7`, `compression-level: 0`.
- The bundle layout (`scripts/package.py:436-455`): `runtime/mineworld[.exe]`, `runtime/worlds/{social-cafe,
  market-town,bodies-yard}`, the 2D and 3D presentation parts, `runtime/BUNDLE.toml` with `commit`, `target`,
  `server` (`package.py:422-434`). **No `entities/` folder** and no guide.
- Run `40447534966` (event `push`, head `5cb3c313`): at the time read (2026-10-27 04:0x UTC) it was
  `in_progress`, with `bundle (linux-x86_64)` already `failure` and **no artifact uploaded yet**
  (`gh api repos/yuema137/MineWorld/actions/runs/40447534966/artifacts` → empty). Layer B therefore designs
  against the declared names, not an observed artifact; C6 re-reads them.

### 2.6 What the hosted images preinstall (read 2026-10-26, runner-images READMEs)

| Image | Rust | Git LFS | Godot | uv | Python | C toolchain |
| --- | --- | --- | --- | --- | --- | --- |
| `ubuntu-24.04` (20261023.334.3) | Rust 1.99.0, rustup 1.29.1 | 3.8.0 | no | no | 3.12.3 | gcc 12–14, clang 16–18 |
| `macos-26` arm64 (20261013.0424.1) | Rust 1.99.0, rustup 1.29.0 | 3.8.0 | no | no | 3.15.0 | Xcode 26.x, Command Line Tools |
| `windows-2025` (20261021.299.1) | Rust 1.99.1, rustup 1.29.1 | 3.8.0 (and Git for Windows' own copy, **verify**) | no | no | 3.12.10 | Visual Studio Enterprise 2022 with the C++ workload |

`winget` on `windows-2025`: preinstalled according to the archived `Cyberboss/install-winget` notice; **verify** in C5
from the image README or a run.

---

## 3. Architecture verdict

**This touches CI architecture.** It is not "only adding jobs", for six reasons:

1. **A new workflow file** `.github/workflows/cold-start.yml` with its own triggers (schedule, dispatch,
   `workflow_run`, a `cold-start/**` push for pre-merge evidence). `ARC-48`'s table of jobs and triggers gains a
   row.
2. **A runner-mutating step.** Today every job adds tools. These jobs **remove** preinstalled tools and assert
   their absence. That is a new kind of CI step, with limits that must be written down (§4.3).
3. **A new mode of the guide runner** with different rules for the clone, the build, `HOME` and the binary. That
   changes `docs/GUIDE_FORMAT.md` (a spec) and adds one directive, `cold-start` (§5.1).
4. **A test executable built in one job and run in another**, on a machine with no Rust. No job does that today.
5. **A cross-workflow artifact contract** with `release.yml` (name pattern, retention, `BUNDLE.toml` fields,
   trigger). Two workflows owned by two PRs now share an interface (§8).
6. **A pre-tag gate proposal**: R-b's `verify-tag` may require a green cold-start run on the tagged commit
   (Q-25g-6). That would make this workflow's job names an interface too.

**Check names.** The jobs are named `cold-start-a (<os>)`, `cold-start-b (<target>)` and `cold-start-harness (<os>)`.
None is a required check, so none is an `ARC-48` interface today. If Q-25g-6 is ruled yes, the summary job
`cold-start` (no matrix) becomes the one name R-b reads, and it is recorded as an interface then.

**Decision record.** One new record, **`ARC-362`** ("The onboarding cold-start check: a clean
runner, the guide in cold-start mode, the bundle check"), with options C1–C4 of §4.1 and P1–P2 of §4.2. The number
was assigned by the coordinator on 2026-10-26 (the next unused ARC id after ARC-361). Plus a dated note on `ARC-48` naming the new workflow and stating that it blocks
nothing.

---

## 4. Options compared

### 4.1 How the clean runner runs the guide

| Option | How | Verdict |
| --- | --- | --- |
| **C1** A new runner in Python that reads the pages | A second parser of the same format | **Declined**: two parsers drift; the brief prefers reuse; `GUIDE_FORMAT.md` would have two implementations. |
| **C2** Build the guide test on the clean runner after the reader installs Rust | `cargo test` in the clone, after chapter 01 | **Declined**: the guide's first page *is* the install, so the runner must exist before Rust does. Also, building the test rebuilds `mineworld` in the test profile and could replace or shadow the reader's binary. |
| **C3** **Build the guide test executable in a helper job; copy it to the clean runner** | `cargo test --no-run -p mineworld-cli --test guide` on a normal runner of the same OS; upload the one executable plus `docs/guide` and the script; the clean job downloads and runs it with `--ignored` | **Chosen**. One parser. The clean runner never compiles test code. The executable is self-contained apart from the system C runtime the image keeps (§4.3). |
| **C4** Run the guide in a container image | A clean image per OS | **Declined**: no macOS or Windows containers on hosted runners; Linux only would miss the operator's three platforms. |

### 4.2 Where the prerequisite install commands come from

| Option | How | Verdict |
| --- | --- | --- |
| **P1** **Chapter 01 gains a step "Install Git, Git LFS and Rust"** with per-OS `sh` and `powershell` blocks, marked `<!-- guide: cold-start -->`. In the default guide test they are listed as not run (`cold-start`); in cold-start mode they run. | The page says exactly what is executed. A real reader gets real commands, which chapter 01 lacks today (F-25g-1). | **Recommended** (Q-25g-1). |
| P2 | `ci_cold_start.py install` runs the vendors' commands (rustup.rs etc.) | It tests a script, not the guide. A reader who follows chapter 01 still has no command. | Fallback if the operator does not want chapter 01 changed before R-d. |

Why not let the default guide test run the install blocks too? Because it would run `curl … | sh` and
`rustup-init` on every CI leg, every time, on machines that already have Rust. The directive keeps them to the
clean runner.

### 4.3 How the runner becomes "clean"

`ci_cold_start.py clean` removes, then `ci_cold_start.py assert-clean` proves absence. Absence is proved by
behaviour, not by a file list: each command must be **not found** (`shutil.which` returns nothing, and running it
fails to start).

| Tool | Linux (`ubuntu-24.04`) | macOS (`macos-26`) | Windows (`windows-2025`) |
| --- | --- | --- | --- |
| Rust | `rustup self uninstall -y` (removes `~/.cargo`, `~/.rustup`); delete any leftover `cargo`, `rustc`, `rustup` on `PATH` | same | same, from PowerShell; also remove `%USERPROFILE%\.cargo\bin` from the User `PATH` in the registry |
| Git LFS | `sudo apt-get remove -y git-lfs`; `git config --system --unset-all filter.lfs.*` and `--global` | `brew uninstall git-lfs`; unset the same config | rename every `git-lfs.exe` found under `C:\Program Files\Git` and on `PATH` (Git for Windows ships its own copy, **verify**); unset the config |
| Godot (B, and A before 06) | assert only: not on the image | assert only | assert only |
| uv (A before 14) | assert only | assert only | assert only |

Asserted absent after `clean` (A and B): `cargo`, `rustc`, `rustup`, `git lfs version` fails, `godot`, `uv`;
the folders `~/.cargo` and `~/.rustup`; the variables `CARGO_HOME`, `RUSTUP_HOME`, `GODOT`, `MINEWORLD_BIN`,
`MINEWORLD_PACKS` unset or pointing nowhere. Layer B adds: no `MineWorld` clone anywhere under `$HOME` and the
workspace.

**Not achievable, and therefore not claimed** (stated in the job summary of every run):

- **The C toolchain stays.** Visual Studio 2022 with the C++ workload (Windows), Xcode and its Command Line Tools
  (macOS), gcc/clang (Linux). Removing them is impractical (tens of GB, system packages). The `mineworld-cli`
  build needs a C compiler: `libsqlite3-sys` and `zstd-sys` are in its dependency tree (`cargo tree -p
  mineworld-cli -e normal,build`). A real reader may lack it. rustup's Windows page says rustup-init "will offer to
  automatically install the prerequisites" (Visual Studio Community) (read 2026-10-26). F-25g-2 asks chapter 01 to
  name this.
- **Git, Python, Homebrew, PowerShell, the system's archive tools** stay. Git is a stated prerequisite but cannot
  be removed safely (the runner itself uses it). Python is the test tooling's, not the reader's.
- **The network is GitHub's datacentre network**, much faster than a home connection. Times are machine times,
  not a reader's times.
- **The image is not a fresh OS.** Other preinstalled tools (Docker, Node, Java, …) remain. They are not
  MineWorld prerequisites, so they cannot hide a missing step — unless one of them provides a C linker or a
  `git-lfs`, which `assert-clean` checks by command name.

---

## 5. The runner's cold-start mode

### 5.1 `docs/GUIDE_FORMAT.md` changes (spec first, C0)

- **§4.7 Cold-start mode (new).** A run in this mode is configured by a file (§5.3). It differs from §4.6 in
  exactly these ways:
  1. **The reader's home is the real home** of the runner's user. Shell blocks see the process's own `HOME`
     (Windows: `USERPROFILE`, `HOMEDRIVE`, `HOMEPATH`). `CARGO_HOME`, `RUSTUP_HOME` and `CARGO_TARGET_DIR` are not
     set.
  2. **The clone is real.** No `insteadOf`, no `safe.directory`, no `GIT_LFS_SKIP_SMUDGE`. `git clone
     https://github.com/yuema137/MineWorld` reads GitHub.
  3. **The build is real.** `cargo build --release -p mineworld-cli` writes the clone's own `target/`.
  4. **Every block starts as a new terminal.** On macOS and Linux a shell block runs as `sh -l -e FILE` (a login
     shell reads `~/.profile`, where rustup's installer adds `~/.cargo/bin`; **verify** for `dash` and macOS
     `/bin/sh` in C1). On Windows, before each block, `PATH` is rebuilt from the Machine and User `PATH` in the
     registry, as a newly opened PowerShell window would have it.
  5. **The binary.** Console commands and feature probes run the file the configuration names. After the page
     named as the build page (`01-get-mineworld`), the runner checks that this file exists and is the one the
     page's text names (`$HOME/MineWorld/target/release/mineworld[.exe]`). A missing file fails page 01 with
     "the reader's build made no binary at <path>".
  6. **`cold-start` blocks run**; in the default mode they are not run and are listed as `cold-start`.
  7. **Pages** can be limited to a list (layer B: `02` … `06`).
- **§6 Directives:** `<!-- guide: cold-start -->` — the next block runs only in cold-start mode. A `cold-start`
  directive on a `console` block or a file block fails the page (only shell blocks install software).
- **§11 Reporting:** each page's wall time, and in cold-start mode a Markdown table for the job summary.
- **§4.6** is unchanged for the default mode.

### 5.2 Code changes (all under `tools/cli/tests/guide/`, test code only)

| Change | Where | Default-mode effect |
| --- | --- | --- |
| `Environment` gains `binary: &Path` and `mode: Mode { Repository, ColdStart }` | `page_run.rs:14-18` | `binary = command::BINARY`, `mode = Repository`: identical behaviour. |
| `command::run` takes the binary as an argument | `command.rs:48-74`; callers `page_run.rs:130`, `run_guide.rs:286`, `:308` | none |
| `command::shell` sets the §4.6 environment only in `Repository` mode; `ColdStart` applies §5.1 rules 1–4 | `command.rs:99-190` | none |
| `Directive::ColdStart`; parser accepts `cold-start` on shell blocks only | `page.rs:225-240`, `:344-371` | blocks so marked are `Pending("cold-start")` |
| `run_guide::run` takes a `RunOptions { mode, binary, home, pages, scratch_root }` | `run_guide.rs:20-44`, `:113-176` | `RunOptions::repository()` keeps today's values |
| `Summary.timings: Vec<(String, Duration)>` per page; `render_markdown()` | `report.rs` | `render()` unchanged; timings printed as one extra line |
| New module `cold_start.rs` and one `#[test] #[ignore]` entry `cold_start` in `guide.rs` | new file; `guide.rs` gains about 15 lines (it is already 912 lines, so the logic lives in the module) | ignored: `cargo test` never runs it |

### 5.3 The `cold_start` test entry

- Reads `MINEWORLD_COLD_START` = path of a TOML file. **Missing variable or file → the test panics** ("cold_start
  is run by .github/workflows/cold-start.yml with MINEWORLD_COLD_START set; see GUIDE_FORMAT.md §4.7"). It never
  passes by doing nothing.
- The file, written by `ci_cold_start.py` (one place for every value):

  ```toml
  layer = "a"                          # or "b"
  guide_before_clone = "/…/harness/docs/guide"   # A: pages read until the clone exists
  clone = "/Users/runner/MineWorld"    # A: where chapter 01 clones; B: absent
  repo = "/Users/runner/MineWorld"     # A: the clone; B: the harness copy (for `equals` and reference links)
  binary = "/Users/runner/MineWorld/target/release/mineworld"   # B: "<bundle>/runtime/mineworld"
  pages = []                           # B: ["02-your-first-world", …, "06-play-it"]
  expected_commit = "<40 hex>"         # the commit the harness was built from
  scratch_root = "/…/runner-temp/cold-start"
  report = "/…/runner-temp/cold-start/report"   # report.md and report.json written here
  mutation = ""                        # §9: a committed mutation's name, dispatch only
  ```

- **Layer A order.** Run page 01 from `guide_before_clone`. When page 01 is done: (1) the clone exists; (2)
  `git -C clone rev-parse HEAD` equals `expected_commit`, else the run is **INCONCLUSIVE** ("main moved between
  the harness build and the clone; re-run"), never PASS; (3) the clone's `docs/guide` equals `guide_before_clone`
  byte for byte; (4) the binary exists. Then the remaining pages run from the clone's `docs/guide`, continuing
  in the same home and the same scratch folders.
- **Layer B:** `pages` only; `BUNDLE.toml`'s `commit` must equal `expected_commit` (else INCONCLUSIVE).
- **Expectation:** `tools/cli/tests/guide/expected_cold_start.toml` with tables `[a]` and `[b]`, each a `not_run`
  list and a `minimum_passed`, compared exactly as `expected_run()` does (`guide.rs:822-845`). The cold-start
  `not_run` lists the remaining `operator-only` blocks; the substitutes (§6) are reported separately.
- **Exit:** the test fails on any `Fail`; INCONCLUSIVE fails the job too (red, with the reason), because a run that
  did not prove anything is not a pass (test rules §18).

### 5.4 Effect on the default guide test

`expected_run.toml` gains one `not_run` entry per new `cold-start` block of chapter 01 (about six: three tools ×
two shells; exact count in C3) and nothing else. `minimum_passed` stays 164. This is the only intended change to
the default run's report, reviewed in the same commit.

---

## 6. Operator-only blocks: substitutes and gaps

Every substitute runs **after** the guide run, in the same home and folders, with the reader's binary. The report
lists it under "Substitutes (not the page's block)" with its own verdict and time. A substitute never turns a
page's `operator-only` block into a pass.

### 6.1 Chapter 06

| Block | Substitute | OS |
| --- | --- | --- |
| 6.2 `mineworld server corner-shop --town` | `ci_cold_start.py substitute serve`: start that exact command line in the reader's worlds folder, match its first lines against the page's expected lines with the same `[..]` rules (read from the page, not copied), capture the invite, keep it running for 6.3, then stop it (Ctrl+C equivalent; **verify** the clean shutdown lines). | all three |
| 6.3 `./mineworld-2d --server … --invite … --seat ben` | Install Godot 4.7.2 with the pinned download of `scripts/ci_godot.py fetch` (the same pin CI uses), then run `./mineworld-2d --server 127.0.0.1:<port> --invite <invite> --seat ben --drive=walk` in the clone. PASS needs the line `drive complete: PASS`. | Linux, macOS |
| 6.3 on Windows | **Not covered.** `mineworld-2d` is a bash script and chapter 06 sends Windows readers to `clients/2d/README.md` (`06-play-it.md:72-73`). | — |

Godot is a documented prerequisite of 06.3 (`06-play-it.md:60-61`) but chapter 06 gives no install command. The
substitute uses CI's pin, which is **not** "exactly as written" (F-25g-3). It is installed only after chapters
01–05 have run, so it cannot mask a gap in them.

### 6.2 Chapter 14

| Block | Substitute |
| --- | --- |
| 14.2 server with a grant | `substitute serve` with 14.2's command line (`--packs packs --town --agent ana --grant ben --save saves/agent`), matched against the page's lines. |
| 14.3 install the adapter | Install `uv` by the vendor's documented command (**verify** at C5 and pin a version), then run 14.3's own `sh`/`powershell` block text (`cd $HOME/MineWorld; uv sync; uv run mineworld-agent --help`). |
| 14.4–14.7 an agent plays | `MINEWORLD_BIN=<reader's binary> uv run --locked pytest clients/agent/tests/test_agent_session.py` in the clone: the recorded session through the real MCP adapter over stdio, against a real server. No model. |
| 14.8 Python program | **Not covered** (needs a grant from a running server and a hand-made file; low value over 14.4–14.7). |
| 14.9 mixed play with the 2D client | **Not covered.** |
| 14.10 restart with a new grant | Covered by `substitute serve` started twice: the two grants must differ. |
| Claude Code / Codex / any live model | **Never** (INV-S25-6). |

Gap: the replayed session plays the committed `docs/guide/examples/corner-shop` (`agent_session.py:31`), not the
reader's chapter-12 folder with `--packs packs`. The report says so.

### 6.3 Layer B

Layer B runs chapters 02–06.1. 06.2 runs as `substitute serve` with the bundled binary. 06.3 is **not covered** until
R-d writes chapter 06's download branch (how a bundle reader opens the 2D client and joins); then B runs the
bundle's own client headless the same way `package.py probe` does (`package.py:591-626`). Chapters 07–14 are out of
B's scope; 12 could not run anyway, because the bundle has no `entities/` folder (F-25g-4).

---

## 7. The workflow

### 7.1 Triggers

```yaml
on:
  schedule:
    - cron: "41 13 * * 0"        # Sundays 13:41 UTC: off the hour, away from the nightly's 10:17
  workflow_dispatch:
    inputs:
      layers:   { type: choice, options: [all, a, b], default: all }
      release-run-id: { type: string, default: "" }   # B: the release run whose artifacts to test
      mutation: { type: string, default: "" }         # §9 adversarial runs only
      budget-minutes: { type: string, default: "60" }
  workflow_run:
    workflows: ["<R-b's release workflow name>"]      # an interface with R-b (§8)
    types: [completed]
  push:
    branches: ["cold-start/**"]   # pre-merge evidence only; ci.yml does not run on this prefix (ci.yml:35-36)
```

- **Weekly** (both layers; B takes the newest successful release-workflow run on `main`, or reports B
  INCONCLUSIVE "no bundle in the last N days").
- **Dispatch** (any layer).
- **`workflow_run`** on the release workflow's `completed` with `conclusion == success`: B only. GitHub runs it from
  the default branch's workflow file (docs, read 2026-10-26), so it is live only after merge.
- **Before each tag:** a dispatch with `layers: all` on the candidate commit, recorded in the release checklist;
  Q-25g-6 asks whether R-b's `verify-tag` requires it.
- **Pre-merge:** a push to `cold-start/<name>`. The clone then reads `main`, which is not the branch under test, so
  §5.3's commit check would be INCONCLUSIVE. In this mode only, `ci_cold_start.py` checks out the branch's commit in
  the clone after the clone succeeds, and the summary is headed **PRE-MERGE (clone moved to <sha>); not acceptance
  evidence**. Acceptance evidence comes only from runs on `main`.
- Scheduled workflows in a public repository are disabled after 60 days without activity (docs, read 2026-10-26);
  not a concern while the repository is active, recorded as a risk.
- Never `pull_request`, never `pull_request_target`. No job is a required check.

### 7.2 Jobs

```text
cold-start-harness (linux|macos|windows)   normal runner; checkout; Rust from rust-toolchain.toml;
                                           cargo test --no-run --locked -p mineworld-cli --test guide
                                           --message-format=json → the executable's path;
                                           upload artifact cold-start-harness-<os>: the executable,
                                           docs/guide/, scripts/ci_cold_start.py, the commit id
cold-start-a (linux|macos|windows)         needs harness(<os>); NO checkout of the repository; NO cache;
                                           download harness → ci_cold_start.py clean → assert-clean →
                                           run-a (writes the TOML, runs `<exe> --ignored --exact cold_start
                                           --nocapture`) → substitute serve/06/14 → summary → budget
cold-start-b (linux-x86_64|windows-x86_64|macos-universal)
                                           needs harness(<os>); NO checkout; download the bundle artifact of
                                           the release run (actions/download-artifact with run-id and
                                           github-token; permission actions: read) → clean → assert-clean
                                           (adds: no clone) → unpack as a player would (into
                                           "MineWorld 世界/", as package.py's probe does) → run-b → summary
cold-start                                 ubuntu; needs all; collects the per-OS reports into one summary
                                           table; red if any leg is red or INCONCLUSIVE
```

- `defaults.run.shell: bash` except where PowerShell is the reader's shell. Every value from an input reaches a script
  through `env:`, never an expression inside `run:` (the nightly's rule, `nightly.yml:12-13`).
- `timeout-minutes`: harness 60, A 120, B 45.
- `concurrency: cold-start-${{ github.ref }}`, `cancel-in-progress: false`.
- **No cache** in `cold-start-a` and `cold-start-b`: no `actions/cache`, no `setup-*` action, no `./.github/actions/native`.
  `ci_cold_start.py --self-test` reads `cold-start.yml` and fails if one appears there (§9 AD-8).
- The harness job may **restore** the existing `native-<os>-core` Cargo cache read-only (Q-25g-8); it is test tooling,
  not the reader's path, and nothing from it reaches the clean jobs except the one executable.

### 7.3 Reporting

Each A and B leg writes `report.md` into `$GITHUB_STEP_SUMMARY` and uploads `report.json` and the full log as artifact
`cold-start-<layer>-<os>` (retention 30 days):

```text
### cold-start A — macOS (macos-26), commit 0b2faf2f, image 20261013.0424.1
| phase                              | time     | verdict |
| clean + assert-clean               | 0:41     | PASS    |
| 01 Get MineWorld (install, clone, LFS, build) | 18:12 | PASS |
| 02 Your first world                | 0:02     | PASS    |
| …                                  |          |         |
| zero to Corner Shop (clean → 05)   | 19:30    | PASS (budget 60:00) |
| substitute 06.2 serve              | 0:08     | PASS    |
| substitute 06.3 2D --drive=walk    | 2:10     | PASS    |
| substitute 14.4–14.7 recorded MCP session | 1:05 | PASS |
NOT COVERED: 06.3 on Windows; 14.8; 14.9; any live model (INV-S25-6)
NOT CLEAN (kept by design): C toolchain (Xcode CLT), git, python3, Homebrew
```

Chapter 01's row is split into its blocks (install, clone, LFS pull, build), because 01 holds almost all the time.

---

## 8. The artifact interface Layer B needs (dependency on R-b)

Stated as **I-25g-ART**; R-b's design (`pr-s23-rb-release-workflow.md`) is asked to adopt it or say what differs.

1. **Name:** one artifact per target, named `MineWorld-<target>`, holding exactly one archive
   `MineWorld-<version>-<target>.zip` (macOS, Windows) or `.tar.gz` (Linux). This is what PR #498 already uploads
   (its `release.yml`, step "Keep the archive", head `5cb3c313`).
2. **Targets B consumes:** `linux-x86_64` on `ubuntu-24.04`, `windows-x86_64` on `windows-2025`, `macos-universal`
   on `macos-26`. `linux-arm64` is out of B's scope until it exists.
3. **Retention:** at least 7 days (PR #498: 7). B by `workflow_run` runs minutes later; the weekly B needs a run from
   the last 7 days, or reports INCONCLUSIVE.
4. **`runtime/BUNDLE.toml`** keeps the fields `commit` and `server` (`package.py:422-434`). B reads the commit to
   build its harness at the same commit and the server path to find the binary.
5. **Finding the run:** (a) `workflow_run` on the release workflow's **name**, so that name is an interface and must
   not change silently; (b) dispatch input `release-run-id`; (c) weekly: the newest successful run of that workflow on
   `main` (`gh run list --workflow <file> --branch main --status success --limit 1`).
6. **Success only:** B runs only when the release run concluded `success`. A missing artifact for one of the three
   targets in a successful run is a FAIL of that leg, not a skip.
7. **No secret** is needed to read the artifacts (`actions: read`, `github.token`).

Recommended trigger: **`workflow_run` plus dispatch with `release-run-id`**; not `workflow_call` from `release.yml`,
because B must stay out of the release workflow's critical path and its permissions.

---

## 9. Adversarial criteria (decided before measuring)

Each is run once by dispatch (or on a `cold-start/**` branch) and its run URL recorded in §14. There are two kinds
of mutation, both named and committed. **Page mutations** (AD-1, AD-2) are data in
`tools/cli/tests/guide/cold_start_mutations.toml` (page, exact old text, new text), applied in memory by the runner
to the page text it reads, never to files on disk. **Runner mutations** (AD-3 … AD-6) are a fixed list inside
`ci_cold_start.py` that skip one removal or write one wrong path, so the check that should catch them is shown to
bite.

| Id | Planted defect | Expected |
| --- | --- | --- |
| AD-1 | `mutation=wrong-build`: chapter 01's build line becomes `cargo build --release -p mineworld-clii` | A red on all three OS, naming `01-get-mineworld`, step 1.1, the block's output |
| AD-2 | `mutation=wrong-expected`: one expected line of 03 changed | A red, naming page 03 and the step |
| AD-3 | `mutation=keep-rust` (the `clean` step skips the Rust removal) | A and B red at `assert-clean`, naming `cargo` and its path |
| AD-4 | `mutation=keep-lfs` | A red at `assert-clean` naming `git-lfs` |
| AD-5 | `mutation=godot-on-path` (B: a dummy `godot` on `PATH`) | B red at `assert-clean` |
| AD-6 | `mutation=wrong-binary`: `ci_cold_start.py config` writes a binary path that does not exist (a script-side mutation, not a page edit) | A red after page 01: "the reader's build made no binary at …" |
| AD-7 | `budget-minutes: 1` | A red at `budget`, with the measured time |
| AD-8 | a branch whose `cold-start.yml` adds `actions/cache` to `cold-start-a` | `ci_cold_start.py --self-test` (run by `fast`'s scripts self-tests) red |
| AD-9 | harness built at a commit other than the clone's (pre-merge run without the PRE-MERGE checkout) | INCONCLUSIVE, red, never PASS |
| AD-10 | B with a bundle whose `BUNDLE.toml` commit differs from the harness | INCONCLUSIVE, red |

A mutation name the runner does not know fails the run. `mutation` is accepted only from `workflow_dispatch`; a
scheduled run with a non-empty mutation is refused.

---

## 10. Cost

- **Money:** none. Standard hosted runners carry no per-minute charge on a public repository (`DEP-19` cost basis;
  the repository is `PUBLIC`, `gh repo view`, 2026-10-26). No model is called.
- **Git LFS bandwidth:** the LFS objects total about 214.5 MB (42 files, `git lfs ls-files -s`, 2026-10-26). Layer A
  pulls them three times: about 0.63 GiB per run, about 2.7 GiB per month weekly. GitHub Free includes 10 GiB of LFS
  bandwidth per month, and the release workflow's `lfs: true` checkouts draw from the same quota (GitHub billing docs,
  read 2026-10-26). If the budget is $0, LFS is blocked for the rest of the month when exceeded — that would break
  every LFS checkout, not only this one. Q-25g-2.
- **macOS runner slots (cap 5, shared by the whole repository; GitHub limits page, read 2026-10-26):** at most two
  macOS jobs of this workflow at a time (harness, then A and B in parallel). The schedule avoids the nightly.
- **Machine time per run (estimates; measured in C6):**

  | Job | Linux | macOS | Windows |
  | --- | --- | --- | --- |
  | harness (debug test build; warm cache 5–10 min, cold 20–30) | 10 | 15 | 20 |
  | A (clean 1, rustup 2, clone + LFS 2, release build 10–20, guide 1, substitutes 5–10) | 25 | 30 | 40 |
  | B (download, clean, unpack, 02–06) | 5 | 5 | 8 |

  About 160 runner-minutes per weekly run, of which about 50 on macOS. Wall time about 60–70 minutes.

---

## 11. Test ownership (test rules §26)

```text
UNIT / RUNNER TESTS (cargo test, every CI leg, default mode)
  - cold-start mode's environment: a ColdStart shell block sees the real HOME and no CARGO_TARGET_DIR,
    insteadOf or GIT_LFS_SKIP_SMUDGE (a block that prints them, against a mock repo)
  - the binary parameter: a console block runs the named file; a missing file fails naming the path
  - `cold-start` directive: pending in default mode, listed as `cold-start`; refused on console/file blocks
  - page filter; per-page timings present for every page run
  - the cold_start entry panics without MINEWORLD_COLD_START (run as an explicit test of the loader,
    not of the ignored entry)
  - mutations: an unknown name fails; a known one changes only the named page text, in memory
SCRIPT SELF-TESTS (ci_cold_start.py --self-test, in fast)
  - assert-clean's detection on a synthetic PATH holding a fake `cargo`; budget arithmetic;
    the workflow lint (no cache, no setup-* in the clean jobs); report rendering
GATE (the workflow itself; the only owner of these failure classes)
  - a stranger's path really works from a clean runner: install, real clone, real build, the guide
  - the bundle runs the guide with no Rust and no Godot
  - per-OS timing and the budget
STATIC
  - ruff, pyright for the script (as for scripts/ today); clippy for the runner; check_ci_pins.py for pins
CI IMPACT
  - fast: the script self-test joins the scripts' self-tests; the guide's runner tests run where the core layer
    runs today (deferred to milestones under ARC-154). No required check changes.
```

No test asserts a getter or an enum. Expected values come from the pages and committed files, never from the
runner's output (test rules §25).

---

## 12. Commit plan

Each commit tracks implementation, deterministic validation and LLM logic review separately. Nothing is checked:
this is a draft.

### C0 — Specifications and decision (Markdown only)

**Goal.** The rules exist before the code. **Scope.** `docs/GUIDE_FORMAT.md` §4.7, §6 (`cold-start`), §11;
`docs/decisions/ARC-362.md` and its INDEX line; a dated note in `ARC-48.md`; this design's ledger.
**Non-goals.** No change to §4.6's default rules.

- [ ] Implementation: the three spec edits and the record (options C1–C4, P1–P2, the not-achievable list of §4.3).
- [ ] Validation: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py --write-index`.
- [ ] LLM review: §4.7 and the default §4.6 cannot both apply to one run; every rule of §5.1 has a test in C1/C2.

### C1 — Runner: binary parameter, modes, timings, directive

**Goal.** The runner can run against an external binary and checkout with real-world rules. **Scope.**
`command.rs`, `page.rs`, `page_run.rs`, `run_guide.rs`, `report.rs`, new tests in `guide.rs` (or a new test module);
§5.2's table. **Depends on** C0. **Must stay unchanged:** every verdict of the default run.

- [ ] Implementation: §5.2 rows 1–6.
- [ ] Validation: `cargo test -p mineworld-cli --test guide` (local, through `with-slot.sh`); the default run's
      `Summary.render()` before and after is identical apart from the timings line (captured at the base SHA and
      committed as evidence, test rules §25); the new unit tests of §11; `cargo clippy -p mineworld-cli --all-targets
      -- -D warnings`.
- [ ] LLM review: no default-mode path reads the new fields; Windows `PATH` rebuild cannot leak the test process's
      `PATH` into a cold-start block.

### C2 — The `cold_start` entry and its expectation

**Goal.** One ignored test that a configuration file drives. **Scope.** `tools/cli/tests/guide/cold_start.rs`,
the entry in `guide.rs`, `expected_cold_start.toml`, `cold_start_mutations.toml`. **Depends on** C1.

- [ ] Implementation: §5.3 (configuration, order, commit and equality checks, INCONCLUSIVE, report files, mutations).
- [ ] Validation: a local cold-start run of layer B shape against a mock repo and the debug binary passed as an
      external path; the loader test without the variable panics; AD-6 locally (a wrong binary path fails page 01).
- [ ] LLM review: INCONCLUSIVE can never print PASS; the mutation code cannot write to disk.

### C3 — Chapter 01's prerequisite step (only if Q-25g-1 = P1)

**Goal.** A reader gets the install commands, and A runs them. **Scope.** `docs/guide/01-get-mineworld.md` and its
`.zh-CN.md` mirror (new step 1.1 "Install Git, Git LFS and Rust", per-OS blocks under `cold-start`, one sentence on
the C toolchain, the LFS lines per Q-25g-2); `expected_run.toml` `not_run` entries. **Depends on** C1.
**Coordination:** R-d writes the download branch of the same page; this commit touches only the from-source part.

- [ ] Implementation: the blocks (rustup: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y`
      on macOS and Linux, the rustup.rs command plus `-s -- -y`; Windows: download `https://win.rustup.rs/x86_64` and
      run it with `-y`, read 2026-10-26; Git LFS: README's per-OS commands, then `git lfs install`; after the clone
      `git lfs pull`).
- [ ] Validation: the default guide test green with the new `not_run` entries; mirror parity.
- [ ] LLM review: the blocks are the commands a person types, in the order the page says, and the page tells the
      reader to open a new terminal after installing.

### C4 — `scripts/ci_cold_start.py`

**Goal.** One script holds every command the workflow runs. **Scope.** Subcommands `clean`, `assert-clean`,
`config` (writes §5.3's TOML), `run-a`, `run-b`, `substitute serve|06|14`, `summary`, `budget`, `--self-test`;
stdlib only; registered with the scripts' self-tests in `fast` (`scripts/ci_layer.py`).

- [ ] Implementation: §4.3, §6, §7.3.
- [ ] Validation: `python3 scripts/ci_cold_start.py --self-test`; ruff; pyright.
- [ ] LLM review: `clean` touches only the listed tools; nothing deletes outside `~/.cargo`, `~/.rustup` and the named
      binaries; every subprocess is an argument list, not a shell string.

### C5 — `.github/workflows/cold-start.yml`

**Goal.** The jobs of §7. **Scope.** The workflow file only; pins by full SHA (the versions already used in
`ci.yml`/`nightly.yml`); the `ARC-48` note from C0 completed with the job names.

- [ ] Implementation: §7.1–7.3, §8.
- [ ] Validation: `python3 scripts/check_ci_pins.py`; the self-test's workflow lint; one push to
      `cold-start/25g-first` (PRE-MERGE) per layer.
- [ ] LLM review: no input reaches `run:` by expression; `actions: read` only where an artifact is downloaded; no cache
      in A or B.

### C6 — Evidence (no code)

- [ ] Verify items: `sh -l` with rustup's profile edit on dash and macOS `/bin/sh`; Git for Windows' bundled
      `git-lfs.exe`; `winget` on `windows-2025`; `walk` on Corner Shop as `ben`; the server's shutdown lines; the
      release workflow's final name and artifact names (R-b).
- [ ] AD-1 … AD-10, each with its run URL.
- [ ] One green PRE-MERGE run of A and B on all three OS; per-chapter times recorded in §14; the budget met or a
      material stop with the numbers.
- [ ] After merge (operator): one dispatch on `main`; that run is the acceptance evidence.

---

## 13. Acceptance

- **AC-25g-1** On `main`, one `workflow_dispatch` run with `layers: all`: `cold-start-a` green on `ubuntu-24.04`,
  `macos-26` and `windows-2025`; each leg's summary shows the clone's commit equal to the harness commit, the
  `assert-clean` list, and a time per chapter.
- **AC-25g-2** Zero to Corner Shop (start of `clean` → end of page 05) ≤ 60 minutes of machine time on each OS in that
  run (or the operator's ruled value, Q-25g-7).
- **AC-25g-3** `cold-start-b` green on the three targets of a successful release-workflow run, with `assert-clean`
  showing no `cargo`, `rustc`, `rustup` and no `godot`, and the binary used being `runtime/mineworld[.exe]` of the
  bundle.
- **AC-25g-4** Every adversarial row of §9 turned its run red (or INCONCLUSIVE-red) as stated, with URLs.
- **AC-25g-5** The default guide test's report is unchanged except the reviewed `not_run` entries and the timings
  line.
- **AC-25g-6** No job of `cold-start.yml` is a required check; `ci.yml`'s job names and triggers are unchanged
  (`git diff` of `ci.yml` is empty).

---

## 14. Evidence ledger

Empty until implementation is authorized.

---

## 15. Findings for other owners

- **F-25g-1 (S25 guide / R-d):** chapter 01 names Git and Rust with links only; it has no install command
  (`01-get-mineworld.md:12-13`). A newcomer needs one per OS. Proposed fix: C3.
- **F-25g-2 (S25 guide):** chapter 01 does not say that building needs a C compiler (Visual Studio C++ tools on
  Windows, Xcode Command Line Tools on macOS, `build-essential` on Linux). `mineworld-cli` needs one
  (`libsqlite3-sys`, `zstd-sys`). The hosted runners hide this gap; layer A cannot detect it (§4.3).
- **F-25g-3 (S25 guide):** chapters 06 and 14 name Godot and uv with links only; no install command.
- **F-25g-4 (R-b / R-d):** the bundle has no `entities/` folder, but chapter 12 copies `MineWorld/entities/modern-goods`
  from the clone. A download-only reader cannot follow chapter 12 (or 14, which continues it). R-d's download branch
  must say where the pack comes from, or the bundle ships it.
- **F-25g-5 (S25 guide):** README's quick start installs Git LFS and runs `git lfs pull`; chapter 01 does neither.
  The two front doors disagree on prerequisites (Q-25g-2).
- **F-25g-6 (R-b):** at read time the dry run `40447534966` had `bundle (linux-x86_64)` failed and no artifact;
  B's first evidence needs a green run.

---

## 16. Execution contract (unfilled)

```text
PROJECT / PR:              S25 PR 25g — onboarding cold-start check, layers A and B
PRIMARY DESIGN DOC:        .structured-coding/plans/mvp0/pr-25g-cold-start.md
RELATED / BINDING DOCS:    step-25-creator-docs.md; step-14-ci.md; step-23-release.md; docs/GUIDE_FORMAT.md;
                           docs/decisions/ARC-48.md, ARC-83.md, ARC-154.md, DEP-19.md; pr-s23-rb-release-workflow.md
IMPLEMENTATION BASE:       <unfilled: origin/main at dispatch>
APPROVED SCOPE:            <unfilled>
FROZEN INVARIANTS:         <unfilled; proposed: §1.2; default guide run unchanged except §5.4; no model; no cache in
                           the clean jobs; never a required check; no PAT or secret>
APPROVED IMPLEMENTATION SEQUENCE: <unfilled; proposed C0–C6>
AUTONOMOUS VALIDATION BUDGET: <unfilled; proposed: local runner tests via with-slot.sh; at most 2 code-PR CI runs in
                           flight; cold-start/** pushes: at most 4 full runs (≈ 4 × 50 macOS minutes) plus the AD
                           runs; LFS ≤ 3 GiB in total>
REQUIRED LIVE DOCUMENTATION: this file
CONTEXT HANDOFF / MEMORY FILE: .structured-coding/plans/mvp0/handoff-25g.md
ENDPOINT AUTHORITY:        commit: <unresolved> · push: <unresolved> · PR: <unresolved> · merge: operator only
```

---

## 17. Operator questions (each with a recommendation)

**Ruled 2026-10-27 (operator, primary session):** every question below is answered as recommended.

- **Q-25g-1 Prerequisite commands.** Add per-OS install commands to chapter 01 (run only by this check), or let a CI
  script install them? **Recommend:** add them to chapter 01 (P1). 建议：写进第 01 章，只在冷启动检查里执行。
- **Q-25g-2 Git LFS in chapter 01.** Chapter 01 does not mention LFS; README does. Should chapter 01 install LFS and
  run `git lfs pull` (≈ 0.63 GiB LFS traffic per run, free quota 10 GiB/month)? **Recommend:** yes, so both front
  doors agree. 建议：加，和 README 一致；每次约 0.63 GiB。
- **Q-25g-3 The C toolchain.** It cannot be removed from hosted runners. Accept that, and add one sentence to chapter
  01 naming it? **Recommend:** yes. 建议：接受，第 01 章加一句说明需要 C 编译器。
- **Q-25g-4 Chapter 06.3.** Run the 2D client's `--drive=walk` against the reader's server on Linux and macOS, with
  CI's pinned Godot; Windows not covered? **Recommend:** yes. 建议：Linux/macOS 用 `--drive=walk` 代替，Windows 不覆盖。
- **Q-25g-5 Chapter 14.** Replace the agent with the recorded session through the real MCP adapter (no model); 14.8
  and 14.9 not covered? **Recommend:** yes. 建议：用录好的会话走真实 MCP，不调模型。
- **Q-25g-6 Before a tag.** Should R-b's `verify-tag` refuse a tag without a green cold-start run on that commit?
  **Recommend:** yes, after the first three green weekly runs; until then a manual dispatch in the release checklist.
  建议：先手动跑，连续三周全绿后再让打 tag 必须过它。
- **Q-25g-7 Budget.** Zero to Corner Shop ≤ 60 minutes per OS (harness build not counted)? **Recommend:** 60. 建议：60 分钟。
- **Q-25g-8 Cache for the helper job.** May the harness job (not the clean jobs) restore the existing Cargo cache?
  **Recommend:** yes; it only builds the test runner. 建议：可以，只用于构建测试工具，干净机不用缓存。
- **Q-25g-9 A red run.** Job summary only, or also open an issue as the nightly does? **Recommend:** summary only now;
  an issue after the check has been green for a month. 建议：先只看 summary，稳定一个月后再自动开 issue。
- **Q-25g-10 Implementer.** **Recommend:** an Opus-class implementer (CI architecture, three OS, cross-workflow
  contract). 建议：用 Opus 实现。

---

## 18. Sources (read 2026-10-26)

- actions/runner-images: `images/ubuntu/Ubuntu2404-Readme.md` (image 20261023.334.3),
  `images/macos/macos-26-arm64-Readme.md` (20261013.0424.1), `images/windows/Windows2025-Readme.md`
  (20261021.299.1) — preinstalled Rust, Git LFS 3.8.0, Python, C toolchains; no Godot, no uv.
- rustup.rs — Unix command `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`; Windows
  `https://win.rustup.rs/x86_64`.
- rust-lang.github.io/rustup/installation/windows-msvc.html — rustup-init offers to install Visual Studio Community
  when Visual Studio is missing.
- git-lfs.com — macOS `brew install git-lfs`; Linux via PackageCloud; Windows installer.
- docs.github.com, Actions limits — Free plan: 20 concurrent jobs, 5 concurrent macOS jobs, 6 h per job.
- docs.github.com, events that trigger workflows — `workflow_run` needs the file on the default branch, types
  `completed`/`requested`/`in_progress`, at most three levels of chaining; `schedule` runs on the default branch, may be
  delayed, disabled after 60 days without activity in public repositories.
- docs.github.com, Git LFS billing — 10 GiB bandwidth and 10 GiB storage included for Free; downloads count against the
  owner; a $0 budget blocks LFS for the rest of the month.
- Cyberboss/install-winget (archived) — windows-2025 images preinstall winget (**verify**).
