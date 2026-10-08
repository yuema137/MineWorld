# PR test-hygiene — every test removes its own scratch data

**Role:** PR design and live ledger for one bounded PR, required by `overall.md` "Parallel build-out,
2026-10-08", coordination ruling 10: *"The full suite leaves about 16 GB of scratch saves in `target/`
(S13 audit). Every test must remove its own scratch data. One bounded PR fixes this after 12c merges."*
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) ruling 10; related:
[`step-14-ci.md`](step-14-ci.md) F-3 and risk R-2 (runner disk).
**Lifecycle:** `DESIGN FROZEN (2026-10-08), primary session.` Source: the primary session's freeze
message of 2026-10-08 ("FREEZE: the test-hygiene PR is DESIGN FROZEN … The design is accepted as
written"). Rulings: QTH-1, QTH-2, QTH-4, QTH-5 accepted as recommended; **QTH-3 overruled** — add
**DEP-29** to `docs/DECISIONS.md` (`tempfile` declined for a `std`-based helper, citing §3), per
`CLAUDE.md` §4 rule 16. Coordination: merge `origin/main` before the final gate and convert S16 E-a's
(#70) three raw `CARGO_TARGET_TMPDIR` uses. Material stops: any assertion change, any production-code
change, any digest change.
**PR number:** assigned at freeze (ruling 7). Working name: *test-hygiene*.

---

## 1. Identity, base, approved scope (proposed)

```text
PR            test-hygiene — every test removes its own scratch data
base          main @ 889d217 (12c merged)
branch        mvp0/pr-test-hygiene, worktree /Users/yuema137/mineworld-worktrees/impl-test-hygiene,
              held by the implementing session only
audit         §2 (measured on 889d217, 2026-10-08)
scope         where each test's scratch lives and when it is removed; one shared test-support crate;
              one check script; one rule in the specifications
```

**The invariant this PR is judged by (I-TH-1): no test's claim changes.** Only where a test's scratch
lives and when it is removed change. No `assert*!` line, no expected value, no seed, no day count, no
run argument other than a scratch path, and no production source file is edited.

## 2. Audit (measured, not estimated)

### 2.1 The measurement

```text
command   TMPDIR=/tmp/th-tmp/ CARGO_TARGET_DIR=/tmp/th-target cargo test --workspace --no-fail-fast
tree      889d217, cold target, the operator's Mac (Apple Silicon), 2026-10-08
result    exit 0; 636 passed, 0 failed, 1 ignored; 143 "test result" lines, all ok
build     test profile built in 1 min 52 s
disk      /tmp/th-target          21 GB
          /tmp/th-target/debug    4.5 GB  (build artefacts; out of scope)
          /tmp/th-target/tmp      16 GB   (CARGO_TARGET_TMPDIR) — 135 entries left behind
          /tmp/th-tmp             0 B     (std::env::temp_dir()) — 0 entries left behind
          /tmp (hard-coded)       no test names a literal "/tmp" path (grep, §2.3)
repo      `git status --short` clean after the run: no test writes into the source tree
log       /tmp/th-logs/suite.log, /tmp/th-logs/leftover.txt (not committed)
```

S13's figure (16 GB, 121 saves on an earlier main) is reproduced: 16 GB, now 135 entries.

### 2.2 What is left, by size

The 43 entries above 1 MiB hold 16.0 GiB; the other 92 hold 5.6 MiB together.

| Entry under `target/tmp` | Size | Written by |
| --- | ---: | --- |
| `market-300` | 2.5 GiB | `tools/cli/tests/market_town.rs:115` via `headless::fresh` |
| `run-300-a`, `run-300-b` | 2.4 GiB each | `tools/cli/tests/run.rs:36` via `fresh` |
| `bodies-yard-restart-{a,b,killed-5,killed-15,killed-25}` | ≈ 257 MiB each | `bodies_yard_restart.rs:92,123,130` |
| `bodies-yard-30` | 257 MiB | `bodies_yard.rs:399` (`scanned`) |
| `market-30-{control,twin,killed,seed-8}` | ≈ 257 MiB each | `market_town.rs:117–120` |
| `market-without-{economy,employment,item-transfer,consumption}-save` | ≈ 256 MiB each | `market_composition.rs:189` |
| `inspect-run` | 249 MiB | `inspect.rs:18` |
| `run-restart-{control,killed-5,killed-15,killed-25}`, `run-continue{,-control}` | ≈ 240 MiB each | `run_restart.rs:104,117,171,194` |
| `milestone-b-{control,killed}` | ≈ 240 MiB each | `milestone_b.rs:201,229` |
| `biography-30`, `routines-30`, `run-seed-{7,8}` | ≈ 240 MiB each | `biography.rs:50`, `routines.rs:64`, `run.rs:131` |
| `composition-*` (8 saves) | 208–241 MiB each | `social_composition.rs:154–389` |
| `bodies-yard-without-bodies-save` | 232 MiB | `bodies_yard.rs:436` |
| `content-kinds-{inert,resume}-saves` | ≈ 165 MiB each | `tools/cli/tests/content_kinds.rs:144,174` |
| `milestone-c`, `inspect-forged`, `biography-refusals` | 12, 12, 4.6 MiB | `milestone_c.rs:251`, `inspect.rs:64`, `biography.rs:206` |
| 27 `bodies-yard-*refused*`, `bodies-yard-without-bodies`, 6 `market-without-<pack>` | ≤ 224 KiB each | pack copies: `bodies/mod.rs:copy_of`, `market_composition.rs:102`, `content_kinds.rs:31` |
| `create-*`, `biography-another-town`, `inspect-nothing`, `content-kinds-*`, `composition-without-*` | ≤ 96 KiB each | `fresh` in `create.rs`, `biography.rs`, `inspect.rs`, `content_kinds.rs`, `social_composition.rs:44` |
| ≈ 45 worldpack fixtures (`passage-*`, `named-*`, `section-*`, `routine-*`, …) | ≤ 20 KiB each | `worldpack/tests/refusals.rs:30` (`Fixture::new`), `worldpack/tests/content_kinds.rs:20` |
| `unnamed` | 80 KiB | `worldpack/tests/social_cafe.rs:342` |
| `cli-malformed` | 12 KiB | `tools/cli/tests/commands.rs:76` |
| `ac1-one-commit`, `ac1-two-commits`, `ac1-shallow-clone` | ≤ 128 KiB each | `tests/acceptance/tests/ac1_composability.rs:431,473` |

**Peak, not only residue.** Cargo runs test binaries one after another, and tests inside one binary
in parallel. With every scratch removed when its test ends, the peak is the largest set of saves alive
at once inside one binary: `run.rs` (two 300-day saves, ≈ 4.8 GiB) or `market_town.rs` (≈ 3.5 GiB).
This PR therefore turns R-2's ≈ 20 GB into ≈ 4.5 GB build + ≈ 5 GB peak scratch; it does not shrink
a 300-day save (that would change what the test writes, I-TH-1). The peak is measured in C5 (A-7).

### 2.3 Every place a test makes scratch (grep over `*.rs`, `target/` excluded)

Two populations, and only the first leaks.

**(L) Under `CARGO_TARGET_TMPDIR`, removed first and never afterwards — the whole 16 GB:**

- `tools/cli/tests/headless/mod.rs:17` `fresh(name) -> PathBuf` — a path that does **not** exist yet
  (`run --save` or `create` makes it). Callers: `biography.rs` (4), `bodies_yard.rs` (5),
  `bodies_yard_restart.rs` (3), `content_kinds.rs` (3), `create.rs` (3), `inspect.rs` (3),
  `market_composition.rs` (2), `market_town.rs` (5), `milestone_b.rs` (2), `milestone_c.rs` (1),
  `routines.rs` (1), `run.rs` (4), `run_restart.rs` (4), `social_composition.rs` (8).
- `tools/cli/tests/bodies/mod.rs:109` `copy_of(directory)` — removes `directory`, copies the pack to
  `directory/bodies-yard`.
- `tools/cli/tests/commands.rs:76`, `worldpack/tests/content_kinds.rs:20`,
  `worldpack/tests/refusals.rs:30`, `worldpack/tests/social_cafe.rs:342`,
  `tests/acceptance/tests/ac1_composability.rs:431,473` — inline `env!("CARGO_TARGET_TMPDIR")`.

**(T) Under `std::env::temp_dir()`, named `mineworld-<area>-<pid>-<name>`, removed on drop — 0 bytes
left on a passing run:**

- `tools/cli/tests/support/mod.rs:96` `SaveDir` (used by `ac15_one_alice.rs`, `restart.rs`; S11-A adds
  two more uses on `mvp0/pr-s11a-handshake`).
- `persistence/tests/support/mod.rs:380` `Scratch` (14 uses in `save.rs`).
- Five copies of one `Scratch` newtype: `systems/{schedule,group-activity,employment,movement,
  inventory}/tests/persisted.rs`.
- The two `harness = false` programs: `persistence/tests/kill_and_resume.rs:592` and
  `tests/acceptance/tests/arrival_resolvers_resume.rs:290`, each `scratch(name) -> PathBuf`, removed
  explicitly at the end of a passing scenario (so kept when it fails, by accident of control flow).

No `src/` unit test writes scratch; `server/tests` writes none (its worlds are in memory).

### 2.4 Constraints the audit found (each binds the design)

- **C-a: a scratch's basename is load-bearing.** A pack's id must equal its directory name
  (`worldpack/tests/refusals.rs:20–22`, `bodies/mod.rs:118`, `market_composition.rs:102`), and some
  refusal messages name the directory. The helper must hand out a path whose **last component is
  exactly the name the test gives** — no random suffix on the leaf.
- **C-b: two shapes.** `fresh` hands out a path that must **not** exist (`biography.rs:217` relies on
  "holds no save"; `create.rs:40` on a missing root); `Fixture::new`, `persistence::Scratch` and the
  `harness = false` programs need an **existing empty** directory.
- **C-c: parallel tests in one process.** 12c's "storage: database is locked" (`step-11-bodies.md`
  E-PO11 notes) was two tests of one binary saving under one name. Names are unique today only by
  convention.
- **C-d: parallel processes.** An operator may run an evidence test while the gate runs on the same
  `target/`. Today `fresh`'s names carry no pid, so two processes collide.
- **C-e: `BODIES_YARD_SAVES`** (`bodies_yard.rs:362–368`): the AO-2 evidence test scans
  `<directory>/bodies-yard-30-seed-<n>` instead of running. Those saves came from a previous run's
  residue in `target/tmp`. Once residue is gone, a documented way to keep them is needed. A save named
  by `BODIES_YARD_SAVES` belongs to the operator and is never removed. It is the only such variable
  (grep `std::env::var` over every `tests/` directory: the others are the two `harness = false`
  programs' role/scenario/directory variables, which children read and never own, and
  `systems/bodies`' `SECOND_PROCESS` re-spawn, which writes no files).
- **C-f: `harness = false` children** receive their directory through an environment variable and are
  `SIGKILL`ed; only the parent may own (and remove) a scratch.
- **C-g: a temporary dropped too early.** `copy_of(&fresh(..))` (`bodies_yard.rs:33,55`) and
  `fresh(name).join(..)` (`content_kinds.rs:31`, `market_composition.rs:102`) would drop a guard at the
  end of the statement. With an absent path that is a **silent leak** (the test then writes into a path
  nobody owns), so the after-run check (§4.3) is the net, and every such expression is rewritten to
  bind its guard.
- **C-h: lanes in flight.** S11-A (`tools/cli/tests/support/mod.rs`, `server/tests`) adds two
  `SaveDir::new` uses; S16 E-a adds three raw `CARGO_TARGET_TMPDIR` uses (`packages/tests`,
  `tools/cli/tests/packs.rs`, `worldpack/tests/package_fields.rs`). Keeping `SaveDir::new(name)` and
  `.path() -> &str` source-compatible makes S11-A's rebase trivial; E-a's three lines are flagged by the
  scan (§4.3) and converted by whichever of the two PRs lands second (one line each).

## 3. Options compared (operator's standing rule: several real candidates)

| | (1) `tempfile` `TempDir` | (2) shared `std` helper crate | (3) cleanup in each test | (4) keep-on-failure via env var |
| --- | --- | --- | --- | --- |
| What it is | `tempfile` 3.27.0, MIT OR Apache-2.0, mature; **not in `Cargo.lock`**: adds `tempfile`, `fastrand`, `rustix` (+ `linux-raw-sys` on Linux); `getrandom`, `once_cell`, `libc`, `errno`, `windows-sys` are already locked | one `publish = false` crate, `std` only, ≈ 120 lines with its tests | a `remove_dir_all` at the end of every test | a policy, not a mechanism: combinable with (1) or (2) |
| Removed on drop | yes | yes | only on the happy path; skipped by early `return` and by panic | — |
| C-a exact leaf name | no: `Builder::prefix` adds a random suffix; the leaf must be a child of the `TempDir` anyway | yes, by construction | yes | — |
| C-b absent / empty | empty only; absent = a child path | both, as two constructors | both | — |
| C-c / C-d parallel | random names: safe, but a same-name clash is hidden, not reported | pid in the container (C-d) + a process-wide registry that **panics naming the duplicate** (C-c becomes a loud failure, not a locked database) | by convention only, as today | — |
| C-e kept saves in one directory | no: random container per scratch | yes: one container per process, leaves side by side | n/a | needs (2)'s layout |
| Keep on failure | needs a wrapper (`TempDir::keep` + `std::thread::panicking`) | in the helper's `Drop` | yes, by accident | — |
| Cost | a new dependency + DEP record (number from the primary session, ruling 6) + a wrapper anyway | ≈ 120 lines we own; no dependency | ≈ 60 edits, unenforceable, regresses silently | — |

**Recommendation: (2) with (4)'s policy.** Every reason to adopt `tempfile` (secure random creation,
cross-platform removal) is irrelevant to a test's scratch under `target/`, and each of C-a, C-c's
loud failure and C-e still needs a wrapper of the same size as the whole helper. This is the
"building our own" direction of `REUSE_POLICY.md`, so the comparison above is the record; whether it
also needs a `DECISIONS.md` entry is QTH-3. (3) is rejected: it is the status quo's failure mode.

**(4)'s policy, concretely (QTH-1):** by default a scratch is removed when its guard drops, **whether
the test passed or failed** — a failing 300-day test re-run while debugging must not leave 2.4 GB per
attempt, which is R-2 again. `MINEWORLD_KEEP_SCRATCH=failed` keeps the scratch of a test that panicked;
`MINEWORLD_KEEP_SCRATCH=all` keeps every scratch. A kept scratch prints one line to stderr,
`[scratch] kept <path>`, so the evidence is findable. Any other value is refused with a panic naming
the accepted values (a typo must not silently delete evidence).

## 4. Design

### 4.1 `mineworld-test-support` (new crate, `tests/support/`)

```text
tests/support/Cargo.toml   package mineworld-test-support, publish = false, no dependencies
tests/support/README.md    short human orientation (CLAUDE.md §2.1)
tests/support/src/lib.rs   Scratch, the scratch! macro, Keep, and the helper's own tests
```

- `scratch!(name)` → `Scratch::absent(env!("CARGO_TARGET_TMPDIR"), name)`; `scratch!(empty name)` →
  `Scratch::empty(..)`. The macro expands `env!` in the **calling** test crate, which is where Cargo
  defines `CARGO_TARGET_TMPDIR`; a library cannot read it for its callers.
- Layout: `<CARGO_TARGET_TMPDIR>/mineworld-scratch-<pid>/<name>`. The leaf is exactly `name` (C-a);
  the pid separates processes (C-d); one container per process keeps kept saves side by side (C-e).
- A process-wide `Mutex<BTreeSet<String>>` registry of live names: creating a name already alive in
  this process **panics** — "scratch '<name>' is already in use by another test in this process"
  (C-c). Registering, creating, removing, and removing the container when the registry empties all
  happen under that one lock, so no create/remove race exists.
- `absent`: removes any stale `<container>/<name>` (a recycled pid) and does not create it (C-b).
  `empty`: the same, then creates it.
- `Scratch: Deref<Target = Path>` and `AsRef<Path>`, `path() -> &Path`; `#[must_use]`.
- `Drop`: reads `Keep` (`MINEWORLD_KEEP_SCRATCH`, §3), checks `std::thread::panicking()`, then
  removes `<container>/<name>` or prints `[scratch] kept …`; unregisters; removes the container if
  this was the last live name and nothing was kept.
- `#![forbid(unsafe_code)]`. No `set_var` anywhere: the helper's tests pass `Keep` explicitly through a
  `#[doc(hidden)]` constructor, so no test mutates the process environment (edition 2024 makes
  `set_var` `unsafe`).

### 4.2 Call sites (each edit limited to its scratch lines, C-h)

- `tools/cli/tests/headless/mod.rs` `fresh(name)` returns `Scratch` (via `scratch!`); its body is the
  only change. `support::SaveDir` keeps its name, `new(name)` and `path() -> &str`, now wrapping
  `Scratch` (S11-A's two new uses compile unchanged). `bodies::copy_of` takes the guard's path and no
  longer removes anything itself.
- Callers change only where the type demands: `Some(&save)` passed as `Option<&Path>` becomes
  `Some(save.path())` (≈ 26 lines over 11 files, counted by grep), or the support functions take
  `Option<&impl AsRef<Path>>` — the implementer picks whichever edits fewer lines and records it; C-g
  expressions bind their guard. A `let` keeps its name, so the lines that use the binding stay as they
  are.
- `worldpack/tests/{refusals,content_kinds,social_cafe}.rs`, `tools/cli/tests/commands.rs`,
  `tests/acceptance/tests/ac1_composability.rs`: the inline `env!("CARGO_TARGET_TMPDIR")` lines become
  `scratch!`/`scratch!(empty …)`; the guard is held for the test's duration (in `Fixture`, as a field).
- The `harness = false` programs: `scratch(name)` returns `Scratch` (empty) owned by the parent only
  (C-f); the explicit `remove_dir_all` lines at the end of a scenario are deleted (the guard does it);
  a failing scenario panics in `main`, so `Drop` sees `panicking()` and (4)'s policy applies.
- (T) population, QTH-2: the persistence `Scratch`, the five `systems/*/tests/persisted.rs` newtypes
  and `SaveDir` already clean up; migrating them gives one convention, one location (`target/tmp`,
  removed by `cargo clean`, rather than the system temporary directory), and a scan with no exceptions.
  Recommended: migrate.
- `BODIES_YARD_SAVES` (C-e): unchanged code path; when it is set, the test opens the operator's saves
  and creates no scratch. Its doc comment gains one sentence: produce the saves with
  `MINEWORLD_KEEP_SCRATCH=all` and point the variable at the printed `mineworld-scratch-<pid>`
  directory.

### 4.3 The check: `scripts/check_scratch.py`

Two subcommands, standard library only, report-and-exit-code like the other `scripts/check_*.py`:

- `scan` (static): no `*.rs` under a `tests/` directory other than `tests/support/` contains
  `CARGO_TARGET_TMPDIR`, `temp_dir()` or a literal `"/tmp`. Prints `file:line` per hit; exit 1 on any.
  Declared in `.structured-coding/standards.md` `checks.tools` as `scratch-scan` (repository scope).
- `left --target-dir <dir> [--tmp-dir <dir>]` (after a run): lists every entry under `<dir>/tmp` and
  every `mineworld-*` entry under the temporary directory, with sizes; exit 1 if any. CI wiring is a
  one-line call after the test layer in `scripts/ci_layer.py`, which lives on the in-flight
  `mvp0/pr-13a-ci`; it is added by whichever of 13a and this PR merges second (QTH-4). Until then it
  is run by hand and its output is recorded here.

### 4.4 Specification

`docs/ENGINEERING_STANDARDS.md` §22 (Headless Testing) gains a short subsection *Test scratch*: a test
that writes files does so through `mineworld-test-support`'s `Scratch`; the scratch is removed when the
test ends, pass or fail; `MINEWORLD_KEEP_SCRATCH` keeps it; names are unique within a process and the
helper refuses a duplicate. `standards.md` `review.conventions` gains the one-line form. (Written
before code, `CLAUDE.md` §2.2.)

## 5. Acceptance (all observable; bounds fixed before the measurement in §2 was read)

```text
A-1  I-TH-1, claims unchanged: `git diff main -U0 -- '*.rs' | grep -E '^[-+].*assert'` prints nothing;
     every changed hunk in an existing test file is classified in §8 as scratch-only; the full suite
     passes with 636 + N passed (N = the helper's own tests), 0 failed, 1 ignored — the same 636
     names as §2.1 (diff of the sorted "test … ok" lists).
A-2  Residue bound: after a passing `cargo test --workspace` into a fresh CARGO_TARGET_DIR with a fresh
     TMPDIR, `check_scratch.py left` reports 0 entries under target/tmp and 0 `mineworld-*` entries in
     TMPDIR, and `du -sk` of both is ≤ 1 024 KiB.
A-3  Static: `check_scratch.py scan` reports 0 hits on the PR head.
A-4  Mutations (§6) each turn the named check red, and the unmutated head is green.
A-5  Parallel safety: two `cargo test -p mineworld-cli --test inspect` processes started together on
     one target dir both pass; the helper's duplicate-name test shows the in-process clash panics with
     the name.
A-6  Keep policy: with MINEWORLD_KEEP_SCRATCH=all, `cargo test -p mineworld-cli --test bodies_yard --
     --ignored` leaves `mineworld-scratch-<pid>/bodies-yard-30-seed-{7,8,9}` and prints their paths;
     re-running with BODIES_YARD_SAVES=<that directory> passes without running a world (no new
     scratch) and removes nothing it did not create. Default policy on a deliberately failing test
     (the helper's own test) removes; `failed` keeps.
A-7  Peak recorded (evidence for R-2, no bound): the largest `du -sk target/tmp` sampled every 2 s during
     the A-2 run.
A-8  No production change: `git diff --stat main` names only tests/**, */tests/**, tests/support/**,
     Cargo.toml / */Cargo.toml [dev-dependencies] and the workspace member line, Cargo.lock (exactly one
     new path package, no registry package), scripts/check_scratch.py, Markdown, standards.md. Replay
     digests are untouched by construction; the 300-day seed-7 deterministic lines in run.rs's output
     are compared to §2.1's log as a spot check.
A-9  Gates: cargo fmt --check, cargo check/clippy --workspace --all-targets --all-features -D warnings,
     cargo test --workspace, check_decision_ids.py, check_doc_headings.py, check_scratch.py scan.
```

## 6. Mutations (scratch commits in the worktree, never pushed; each reverted, evidence in §9)

- **M-1 (the required one).** In `tools/cli/tests/inspect.rs`, `std::mem::forget` the `inspect-run`
  guard. Run `cargo test -p mineworld-cli --test inspect` into a fresh target dir, then
  `check_scratch.py left` → FAIL naming `mineworld-scratch-<pid>/inspect-run`.
- **M-2.** Revert `worldpack/tests/social_cafe.rs:342` to the raw `env!("CARGO_TARGET_TMPDIR")` line →
  `check_scratch.py scan` FAILs naming that file and line; `left` also FAILs after the worldpack run.
- **M-3.** Give two tests in one binary the same scratch name → the second panics with the duplicate
  message (instead of "database is locked").
- **M-4.** Rewrite one C-g site back to `copy_of(&fresh(..))` → that test fails loudly (its pack copy
  is gone before it is read); and `fresh(..).join(..)` → `left` FAILs (the silent case is caught).

## 7. Commit plan

Each commit tracks implementation, validation and review separately; evidence goes to §9 as `E-TH<n>`.

### C0 — Design (this document) — docs only

- [x] Implementation: §§1–8, from the audit in §2.
- [x] Validation: `check_doc_headings.py` → 176 numbered sections across 25 documents, none
  duplicated; `check_decision_ids.py` → 51 ids, all distinct (§9 E-TH0).
- [x] Review: every claim in §2 cites a file and line or the measurement; freeze questions in §10.

### C1 — Specs and the helper: `tests/support`, `check_scratch.py`, ENGINEERING_STANDARDS §22

**Goal.** The rule and the mechanism exist, tested, before any test moves. **Scope.** §4.1, §4.3,
§4.4; root `Cargo.toml` member `tests/support` and `[workspace.dependencies] mineworld-test-support`.
**Non-goals.** No existing test edited.

- [x] Implementation: `tests/support/{Cargo.toml, README.md, src/lib.rs, tests/scratch.rs}`; root
  `Cargo.toml` member and workspace dependency; `scripts/check_scratch.py`; `ENGINEERING_STANDARDS.md`
  §22 "Test scratch"; `DECISIONS.md` **DEP-29** (added per the freeze's QTH-3 ruling);
  `standards.md` convention line and `scratch-scan` check.
  - Bounded detail: a scratch name must be one plain path component (asserted). The one site that
    used a two-component name (`social_cafe.rs`'s `unnamed/social-cafe`) takes `unnamed` as its
    scratch and joins `social-cafe` (C3), so removal always covers the whole tree.
  - Bounded detail: `Keep` is read when the guard is **made**, so an invalid value panics in the test,
    never inside `Drop` during unwinding (which would abort).
- [x] Validation (E-TH1): `cargo test -p mineworld-test-support` → 6 passed (removed on drop for both
  shapes, leaf name and location; failing test × `Never`/`Failed`/`All`; passing test × the three,
  including the container's removal; duplicate live name refused with its name, reusable after drop;
  non-single-component names refused; the variable's values). `cargo clippy -p
  mineworld-test-support --all-targets -- -D warnings` clean. `/tmp/th-dev/tmp` empty after it.
  `check_scratch.py scan` on the pre-migration tree → exit 1 with exactly the 16 lines of §2.3's (L)
  and (T) populations, nothing else.
- [x] Review: `#![forbid(unsafe_code)]`, no `set_var`; registry lock held across registration,
  creation, removal and the container's removal; the lock is released before any panic; leaf exact.

### C2 — `tools/cli/tests` on the helper

**Scope.** `headless/mod.rs` `fresh`, `support/mod.rs` `SaveDir`, `bodies/mod.rs` `copy_of`, `commands.rs`,
and the callers §4.2 names; `tools/cli/Cargo.toml` dev-dependency. **Depends on** C1.

- [x] Implementation: as §4.2. `fresh` returns `Scratch` (re-exported by `headless`); `SaveDir` keeps
  `new(name)` / `path() -> &str` and wraps a `Scratch`; `commands.rs` uses `scratch!`.
  - **Bounded deviation (C-g resolved by `Scratch::within`, commit `5c5a087`).** Deviation: the helper
    gains `within(child)` and `AsRef<OsStr>`. Reason: four helpers return a pack copy *inside* a
    scratch (`content_kinds::with_things`, `market_composition::without`,
    `social_composition::without_owning` — a fourth C-g site the design's audit missed, found by
    reading every function that calls `fresh` — and `bodies::copy_of` / `without_bodies`); binding a
    guard inside them would remove the copy on return. They now return the `Scratch` itself,
    `fresh(name).within("<pack>")`, so the guard lives as long as the caller's binding and the leaf is
    still the pack's id. `AsRef<OsStr>` lets a `&Scratch` go to `WorldPack::read(impl Into<PathBuf>)`
    unchanged. Impact: no call-site line of any test changed for it; `copy_of`/`without_bodies` take
    the guard by value (`copy_of(fresh(..))`, three lines). Validation: the helper's own
    `a_pack_copy_is_named_as_its_pack_and_its_whole_scratch_goes_with_it`.
  - `Some(&save)` where `Option<&Path>` is expected needed **no** edit: the compiler coerces
    `&Scratch` through `Deref` there. Only `market_town.rs`'s explicit `(PathBuf, …)` tuple type and
    `bodies_yard.rs::scanned`'s `match` (C-e: the guard is held in a deferred `let scratch;` so the
    operator's `BODIES_YARD_SAVES` path is never owned) changed.
- [x] Validation (E-TH2): `cargo test -p mineworld-cli -p mineworld-test-support` → 53 passed,
  0 failed, 1 ignored (the AO-2 evidence test), exit 0; `/tmp/th-dev/tmp` empty afterwards (it held
  only this run). Name-for-name comparison with §2.1 is A-1, on the full run.
- [x] Review: hunks classified in §8; `git diff -U0 -- '*.rs'` (helper excluded) has no added or
  removed `assert` line; the only removed `.expect(` lines are the old helpers' own
  `create_dir_all(..).expect("a scratch directory")`.

### C3 — `worldpack`, `tests/acceptance` (incl. `arrival_resolvers_resume`), `persistence`
(incl. `kill_and_resume`), `systems/*/tests/persisted.rs`

**Scope.** §4.2's remaining sites; QTH-2 decides whether the (T) population is included (if not, this
commit is only `worldpack` + `ac1_composability` and the two `harness = false` programs).

- [x] Implementation: as §4.2, with QTH-2 accepted (the (T) population included).
  - `worldpack`: `Fixture.root` is a `Scratch`; `write_pack` returns one; `social_cafe.rs` uses
    `scratch!("unnamed").within("social-cafe")`.
  - `ac1_composability.rs`: `scratch_repository` returns `scratch!(empty name)`; the clone is
    `scratch!("ac1-shallow-clone")`.
  - The two `harness = false` programs: `scratch(name)` returns `scratch!(empty "kill-<name>")` /
    `"resolver-kill-<name>"`, constructed in the parent only; their end-of-scenario
    `remove_dir_all` lines are deleted (the guard removes the save at the same point).
  - `persistence/tests/support` `Scratch` and the five `systems/*/tests/persisted.rs` newtypes keep
    their names and constructors and wrap the helper's `Scratch`; their `Drop` impls are deleted.
  - Unused `PathBuf` imports removed where the change left them unused.
  - Not done (C-e's doc sentence): `bodies_yard.rs`'s doc comment is unchanged; `scanned` gained a
    two-line comment instead, and the keep workflow is in `ENGINEERING_STANDARDS.md` §22. Bounded.
- [x] Validation (E-TH3): `cargo test -p` test-support, worldpack, acceptance, persistence, schedule,
  group-activity, employment, movement, inventory → 162 passed, 0 failed, exit 0; `[resolver-yard]
  PASS`, `[cafe] PASS`, `[clock] PASS` printed by the two `harness = false` programs;
  `/tmp/th-dev/tmp` empty afterwards. `check_scratch.py scan` → "133 test sources, none makes scratch
  outside mineworld-test-support". `cargo clippy --workspace --all-targets --all-features -D
  warnings` clean; `cargo fmt --check` clean.
- [x] Review: children read `MINEWORLD_KILL_TEST_DIR` / `MINEWORLD_RESOLVER_KILL_DIR` and construct
  no `Scratch` (C-f); `BODIES_YARD_SAVES` is joined, never owned (C-e).

### C4 — Close: full gate, mutations, A-2…A-8 evidence, ledger

- [ ] Implementation: §9 filled; status line; handoff note for S13 (R-2's new figures, QTH-4).
- [ ] Validation: A-1 … A-9, M-1 … M-4, each PASS/FAIL/INCONCLUSIVE from evidence.
- [ ] Review: the whole diff against I-TH-1 and A-8.

A planned commit may split into several coherent commits; the mapping is recorded here.

## 8. Hunk classification (filled during C2–C3)

Every hunk in an existing test file is listed with one of: `scratch-construct`, `scratch-pass-path`,
`scratch-bind-guard`, `scratch-remove-deleted`, `import`. Any other kind is a material stop.

## 9. Evidence

- **E-TH-audit** — §2.1's run (log `/tmp/th-logs/suite.log`): 636 passed, 0 failed, 1 ignored;
  `target/tmp` 16 GB in 135 entries; TMPDIR 0 entries.
- **E-TH0** — C0's doc checks, as recorded under C0.

## 10. Freeze questions

- **QTH-1** Keep policy: default remove even on failure, `MINEWORLD_KEEP_SCRATCH=failed|all` to keep.
  *Recommend accept.* Alternative: keep on failure by default (more evidence, but a failing long test
  re-run locally re-creates the disk problem).
- **QTH-2** Migrate the already-clean (T) population (persistence, five system packs, `SaveDir`, the
  harness programs) too. *Recommend yes:* one convention, `scan` without exceptions, scratch removed by
  `cargo clean` if a process is killed. Cost: ≈ 7 small struct deletions in files no other lane edits.
- **QTH-3** Record the build-our-own choice over `tempfile` in `docs/DECISIONS.md`? *Recommended no;*
  **overruled at freeze:** DEP-29 is added in C1 (`CLAUDE.md` §4 rule 16).
- **QTH-4** CI wiring of `check_scratch.py left`: in this PR if 13a has merged first, otherwise in 13b.
  *Recommend accept.* `scan` goes into `standards.md` now either way.
- **QTH-5** New workspace crate `tests/support` (`mineworld-test-support`, dev-dependency only). *Recommend
  accept;* the alternative (a `#[path]`-included module per crate) duplicates code across crates.

## 11. Execution contract (proposed; fields follow `implementation-working-rules.md`)

```text
PROJECT / PR:          MineWorld mvp0 — PR test-hygiene (number assigned at freeze)
PRIMARY DESIGN DOC:    .structured-coding/plans/mvp0/pr-test-hygiene.md
RELATED / BINDING:     CLAUDE.md; overall.md ruling 10; step-14-ci.md F-3, R-2;
                       docs/ENGINEERING_STANDARDS.md; docs/REUSE_POLICY.md; .structured-coding/standards.md
IMPLEMENTATION BASE:   main @ 889d217 (rebase onto the current main before the PR if it moved; re-run
                       §2.3's grep after a rebase)
APPROVED SCOPE:        §1 and §4; the files of §7; nothing else
FROZEN INVARIANTS:     I-TH-1 (no claim changes: no assertion, expected value, seed, day count or
                       non-path run argument edited); no production source change (A-8); no digest
                       change; a save named by BODIES_YARD_SAVES is never removed; harness children
                       never own a scratch
APPROVED SEQUENCE:     C0 → (freeze) → C1 → C2 → C3 → C4
VALIDATION BUDGET:     unrestricted local cargo runs; the full suite at most 3 times (A-2 run, the
                       final gate, one spare), each in the background with a fresh CARGO_TARGET_DIR
                       under /tmp/th-*; no network, no model calls
LIVE DOCUMENTATION:    this file (§§7–9)
HANDOFF FILE:          N/A unless the session is compacted; then a §12 "Handoff" section here
ENDPOINT AUTHORITY:    implementation + local validation: authorized after freeze (source: primary
                       session's task message, 2026-10-08, Phase 2)
                       semantic commits: authorized (same source)
                       branch push: authorized (same source)
                       PR creation, marked READY FOR OPERATOR REVIEW: authorized (same source)
                       CI repair to review readiness: authorized (same source)
                       merge: explicit operator authorization only — never by this session
MATERIAL STOPS:        any change to a test's assertion; any production-code change beyond test
                       support; any digest change (source: same task message); plus any change to
                       §4's layout or §3's policy after freeze
POST-MERGE SYNC OWNER: this session owns this file's evidence and status; the primary session owns
                       overall.md and step-14-ci.md (R-2's figures are reported to it)
NORMAL STOP:           PR test-hygiene READY FOR OPERATOR REVIEW — DO NOT MERGE
MERGE AUTHORITY:       NEVER merge without explicit operator approval
```
