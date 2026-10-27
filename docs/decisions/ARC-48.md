## ARC-48 — CI layers, triggers, and what blocks a merge

**Date** 2026-10-08 · **Status** decided; layers 1–2 live from S13 PR 13a · **Approved by** the primary
session at 13a's design freeze (step-14 §9.0), under the operator's 2026-10-08 decision to make the
repository public · **Relates to** `DEP-17`, `DEP-18`, `ARC-30`, `ARC-35`, overall `D-12`,
[`ENGINEERING_STANDARDS.md`](ENGINEERING_STANDARDS.md) §§15–16 · **Design**
`.structured-coding/plans/mvp0/step-14-ci.md` §§3.4–3.7, 9.0

**Problem.** `ENGINEERING_STANDARDS.md` §16 names four CI layers: fast structural checks on every change;
core integration tests on every pull request; scenario tests; long-running stability tests that "can run
separately from the fastest PR loop". The project needs to know which layer runs when, what a red layer
stops, and what is mechanism rather than promise.

**Decision.** One workflow, `.github/workflows/ci.yml`. Its jobs name layers, never commands. The
layer → command table exists once, in `scripts/ci_layer.py`.

```text
job        layer (§16)            trigger                                         merge
fast       1 fast structural      push to every branch; pull_request               blocks
test       2 core integration     non-draft pull_request; push to main;           blocks
                                  push to scratch/**
image      (the runtime image)    workflow_dispatch; push to scratch/*-image      evidence only
scenario   3 scenario             push to main; nightly; workflow_dispatch (13b)  blocks main's health
stability  4 long-running         nightly; workflow_dispatch (13c)                reports
clients    Godot headless probes  nightly; workflow_dispatch (13c)                reports
```

- **`fast`** runs, in order:
  - `cargo fmt --all --check`;
  - `scripts/check_doc_headings.py`, `scripts/check_decision_ids.py`, `scripts/check_ci_pins.py` and
    `scripts/check_scratch.py scan`;
  - `cargo check --workspace --all-targets`;
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- **`test`** runs `cargo test --workspace`: the whole default suite. After it passes,
  `scripts/check_scratch.py left --target-dir target` fails the layer if any test left scratch behind
  (`ENGINEERING_STANDARDS.md` §22, `DEP-29`).
  - It builds first with `--no-run`, which runs no test and only separates build time from test time in
    the log.
  - That includes the two `harness = false` programs and the history-reading scans of `ARC-35`.
  - The default suite already holds the Social Café and Market Town scenarios and the long runs, by
    decision (step-10 QS-59).
  - `cargo test` cannot exclude a target without a hand-kept list, and such a list fails open. So layer 2
    is the whole suite until a fail-closed selector exists.
- **Full history.** Every job that runs a layer checks out with `fetch-depth: 0`, as a partial clone
  (`filter: blob:none`), of the whole tree. Tests read `presentation/` and `clients/protocol/`, so a
  sparse checkout that omitted them was tried and dropped (step-14 A-C3). A shallow clone makes
  `ac1_composability` fail by name, never skip.
- **Economy.**
  - `concurrency` is per ref, and cancels a stale run except on `main`.
  - A draft PR runs `fast` only.
  - Branch pushes run `fast` even when a PR also runs it, because minutes are free on a public
    repository.
  - `CARGO_INCREMENTAL=0` and `CARGO_PROFILE_DEV_DEBUG=line-tables-only` are CI-only settings. They change
    no codegen semantics: `ARC-30`'s `opt-level`, debug assertions and overflow checks are untouched.
- **Never weaker than local.**
  - A blocking job has no `continue-on-error`, no retry and no `|| true`.
  - No test reads a CI variable to relax itself.
  - The workflow runs with `permissions: contents: read`, uses `pull_request` and never
    `pull_request_target`, and needs no secret.
- **Check names are an interface.** The jobs are named exactly `fast` and `test`, with no matrix, because
  a required check matches by name.

**What "blocks" means, and what enforces it, as of this record.**
1. **Policy, in force now.** An execution contract's `READY FOR OPERATOR REVIEW` requires green `fast` and
   `test` on the exact final PR head. That run is the PR's one canonical full-suite evidence, replacing a
   local full gate. The operator merges only with both green.
2. **Mechanism, not yet in force.** While this record was being written, the repository was private on
   GitHub Free, where branch protection and rulesets are unavailable (step-14 F-1: the API answers 403).
   `D-12`'s "protection makes that a mechanism" was therefore not yet true.
   - The repository became public on 2026-10-08, which makes protection available.
   - Protection follows this record's PR (S13 13a) **after it merges**. Requiring checks that do not yet
     exist on `main` would block every merge. The primary session then enables protection, or a ruleset,
     on `main`.
   - It requires `fast` and `test` and "require branches to be up to date", because AC-1's scan reads
     merge structure.
   - That settings change is outside every PR, and is recorded where it is made.
3. **A known property of required checks.** A job skipped by its `if:` reports success to a required
   check, as `test` does on a draft PR. A draft cannot be merged, and `ready_for_review` re-runs `test` on
   the same head, so this opens no gap in practice.

**Why.** §16's layers become triggers without re-tiering any existing test. Each red layer has one
meaning. And no claim about enforcement is stronger than the repository's settings.

**Accepted limitations.**
- Layer 2 is the whole suite, about 70 % of it six long runs, so the PR loop's `test` is not fast.
- The canonical evidence arrives when CI finishes, not when the session stops typing.
- Until protection is enabled, a merge without green checks is prevented by discipline only.

**Note, 2026-10-08 (S10 PR P3, `pr-s10-p3-python-sdk.md` C5): the Python layers and the first jobs
outside the container.**
- **Layers.** `ci_layer.py` gains `python` (the Python static checks, `cargo build -p mineworld-cli`,
  `pytest` including the real-server tests, the scratch check) and `python-smoke` (the static checks and
  `pytest -m "not real_server"`, no Rust build). `core` is unchanged. `fast` gains the four static
  commands (`uv sync --locked`, `ruff check`, `ruff format --check`, `pyright`), because they cost about
  6 s against the primary session's 60 s allowance (QP3-3), measured on PR #98's first run.
- **Job.** One new job, `python`, a matrix over `ubuntu-24.04`, `windows-2025` and `macos-15`, each
  running the full `python` layer: on that first run Windows took 2 min 26 s longer than Ubuntu and
  macOS 8 s less, both under the 3-minute rule that would otherwise have reduced Windows to
  `python-smoke` and dropped macOS. `fast` and `test` keep their names and contents. `python` is not a
  required check until the primary session makes it one.
- **Departure from "inside the container".** The Linux leg runs in the toolchain container like every
  other layer, with uv copied into the image pinned by digest (DEP-26). Windows and macOS cannot run that
  container, so their legs install what they need on the runner, each pinned: the Rust toolchain from
  `rust-toolchain.toml` (`rustup toolchain install`), uv through `astral-sh/setup-uv` pinned by commit
  SHA at uv 0.12.5, and Python 3.12 (the `requires-python` floor) through uv. They still name a layer and
  never a command of it. This departure exists only because the operator requires every platform
  (2026-10-08) and a Linux container cannot show Windows or macOS behaviour.

---

## ARC-48 note — the scenario group of jobs and the Windows suite (2026-10-09, S13 PR 13b)

The decision is unchanged; its table gains these rows, and two details of the required checks change.

```text
job           layer / role                  trigger                                         merge
scenario      3 scenario: the runtime        push to main; workflow_dispatch;               blocks main's
              image's evidence (was          push to scratch/*-scenario                     health
              `image`) and the Linux x86_64
              parity record (ARC-49)
mac           parity record, macos-26        same                                           blocks main's health
linux-arm     parity record, Linux arm64     same                                           blocks main's health
windows       parity record, windows-2025    same                                           blocks main's health
ac8           AC-8's comparison (ARC-49)     same; if: always()                             blocks main's health
test-windows  the `core` layer natively on   push to main; workflow_dispatch                reports
              windows-2025
```

- `image` is renamed `scenario`; its scratch route `scratch/*-image` becomes `scratch/*-scenario`.
- `fast` gains one command, `python3 scripts/ci_parity.py --self-test` (under a second). `test` is not
  run on `scratch/*-scenario` branches, as it was not on `-image` ones. `core`'s commands are unchanged.
- `test-windows` reports only. It is red until the Windows-portability PR (13w) makes the suite compile
  and pass there; that PR adds its `pull_request` trigger, and only then may the operator make it a
  required check (QB-11).
- None of the new jobs runs on `pull_request`, and none is required (QB-8).

---

## ARC-48 note — the default suite on Windows and macOS (2026-10-09, S13 PR 13w)

The decision is unchanged; its table gains or changes these rows (step-14 §15.5, QW-4, QW-5).

```text
job           layer / role                  trigger                                         merge
test-windows  the `core` layer natively on  non-draft pull_request; push to main;          reports
              windows-2025                  push to scratch/** (not -image, -scenario);
                                            workflow_dispatch
test-macos    the `core` layer natively on  the same                                       reports
              macos-26
```

- **`test-windows` gains its `pull_request` trigger**, in the PR that makes it green (13w), as 13b's
  note said it would. `test-macos` is new, with the same triggers. Both carry `test`'s triggers plus
  dispatch, so a mutation pushed to a `scratch/` branch is judged on all three operating systems.
- **Neither is a required check.** Making them required is the operator's settings change (QB-11): the
  recommendation is after five consecutive green `main` pushes.
- **E-c/E-d's `platforms` job stays** (QB-15, QW-4's fallback). Retiring it was conditional on its layer
  being a strict subset of `core`. E-c (#99) merged first and added its offline vendor check (PD-p3,
  EC-3 (b): `ci_layer.py --offline-check`), which `core` does not run, so the condition fails and the job
  is kept unchanged. The rest of the layer duplicates part of `core` on the same runners; trimming it to
  what `core` lacks is a later, separate change.
- Wall times are recorded in step-14 §15.8 (W-C4); both jobs have a 60-minute timeout and run in
  parallel with `test`.

---

## ARC-48 note — documentation-only changes do not run the build (2026-10-09, S13 PR 13x)

The decision is unchanged in what each layer runs; its triggers change (step-14 §16, under the
operator's requirement of 2026-10-09 that a documentation-only update must not trigger CI).

```text
job        layer / role                         trigger                                  merge
changes    classify the diff: code or docs      every run                                (feeds fast)
           (scripts/ci_changes.py)
fast       code: 1 fast structural              push to main and scratch/**;             blocks
           docs only: the `docs` layer          pull_request; workflow_dispatch
test, python, platforms, test-windows,          as before, and only when changes says    as before
test-macos, scenario, linux-arm, mac,           code; otherwise skipped
windows, ac8
```

- **The docs set is fixed by audit, not by guess** (step-14 §16.3): `docs/**`,
  `.structured-coding/plans/**`, the root `README*.md`, `CLAUDE.md`. Nothing else is documentation,
  including subdirectory READMEs, some of which tests read (`cognition/lm-controller/README.md`,
  `server/PROTOCOL.md`). A diff is docs-only when every changed path, renames split into deletion and
  addition, is in the set; an empty diff, an unknown base (a new branch, a force-push) and any `git`
  failure are code. A push to `main` and a dispatch are always code, so `main` stays fully verified.
- **The `docs` layer** (`ci_layer.py docs`) is `check_doc_headings.py`, `check_decision_ids.py` and
  `ci_changes.py --self-test`, the only CI commands that read the docs set, run on the runner's Python
  in seconds. `fast` also runs the classifier's self-test.
- **Required checks still report.** No workflow-level `paths-ignore` (a workflow that never starts leaves
  `fast` and `test` pending). Skipped jobs use a job-level `if:` on `needs.changes.outputs.code`, which a
  required check reads as passing. `fast` runs `if: always()` and fails first if `changes` did not
  succeed, so a broken classifier cannot let a PR merge with everything skipped.
- **One run per PR commit.** `push` now triggers on `main` and `scratch/**` only; feature branches are
  run by `pull_request`. This supersedes the "Economy" bullet that branch pushes run `fast` even when a
  PR also runs it: a branch with no PR is not run until a (draft) PR is opened.
- **Reuse.** `dorny/paths-filter` was considered and rejected, to avoid a third-party action holding a
  repository-read token for a 30-line check that `git diff` already answers.
- **The standards agree.** `ENGINEERING_STANDARDS.md` §15 ("every pull request should run" format,
  compile, lint and core tests) and §16 ("run on every change") now say that a documentation-only change
  as defined here runs the documentation checks only: no other check reads its files, so their verdict
  is the base's, and `main` runs every layer on every push.

---

## ARC-48 note — layer 4, the nightly workflow (2026-10-10, S13 PR 13c)

The decision is unchanged in what `ci.yml`'s layers run; its table gains the rows of `ARC-83`, all in a
second workflow, `.github/workflows/nightly.yml`.

```text
job group          workflow     trigger                                              merge
gate               nightly.yml  schedule 10:17 UTC; workflow_dispatch; push to        (decides the night)
                                scratch/**-nightly
parity-long(-*)    nightly.yml  the same, when the gate runs the night               reports
stability-*        nightly.yml  the same                                             reports
clients-*          nightly.yml  the same                                             reports
repeat-*           nightly.yml  the same                                             reports
report             nightly.yml  the same; if: always()                               reports; opens and
                                                                                     closes the `nightly`
                                                                                     issue
```

- **Nothing in `nightly.yml` is a required check, and nothing in it runs on `pull_request`.** A red night
  reports; it never gates a PR. `parity-long` reports only and does not inherit `ac8`'s "blocks main's
  health" (13c QC-11); that is revisited after a month of nights.
- **`ci.yml`'s scratch route.** `test`, `test-windows`, `test-macos`, `platforms` and `python` are not
  run on a `scratch/*-nightly` branch, beside their `-image` and `-scenario` exclusions; `fast` still is.
  Nothing else in `ci.yml` changes.
- **`fast` gains four self-tests** (`ci_nightly.py`, `ci_godot.py`, `ci_repeat.py`, `ci_stability.py`
  `--self-test`, each under a second), and `check_ci_pins.py` gains one rule: every `uses:` that is not
  a local `./` path ends in `@<40 hex>`, a full commit SHA (13c QC-7). `core`'s commands are unchanged.
- **`I-S13-7` is amended for one job.** The workflow default stays `permissions: contents: read`. `gate`
  adds `actions: read`; `report` adds `actions: read` and `issues: write` (operator, 13c QC-1,
  2026-10-10). Both use the automatic `GITHUB_TOKEN`; no secret exists.

---

## ARC-48 note — tests at milestones, not per PR or per merge (2026-10-18, ARC-154)

Amended by [`ARC-154`](ARC-154.md) (operator policy of 2026-10-18, speed first).

- Pull requests and pushes to `main` run `fast` only.
- The required gates pass on a pull request with "deferred to milestone", and do not run on a push to `main`.
- The full matrix runs on a dispatch with `milestone: true` or on a PR labelled `milestone`. That includes the AC-8 legs and `ac8`, which no longer run on every push to `main`.
- Scratch branches and the documentation-only rule of the 13x note are unchanged.

---

## ARC-48 note — the release workflow (2026-10-27, S23 R-b)

Design: `.structured-coding/plans/mvp0/pr-s23-rb-release-workflow.md` §3.10.

- `.github/workflows/release.yml` (workflow "Release") exists beside `ci.yml` and `nightly.yml`. Its
  triggers are a `v*` tag push, `workflow_dispatch`, and a push to `release-dry-run/**`; it has no
  `pull_request` or `pull_request_target` trigger. Its jobs (`verify-tag`, `bundle`, `server-arm64`,
  `bundle-arm64`, `reference`, `smoke`, `parity`, `draft`) are not PR checks and need no name in branch protection;
  `ci.yml`'s `fast` and `test`, their names and their triggers are unchanged.
- It reads no secret. The workflow default is `permissions: contents: read`; `verify-tag` adds
  `checks: read` (it reads the tagged commit's `fast` and `test` check runs); only `draft` can write
  (`contents: write`, `id-token: write`, `attestations: write`), and only on a tag run or a dispatch with
  the input `draft: true`. It creates draft releases only; nothing in it publishes (`ARC-79`).
- `publish.yml` (R-d) is the only workflow that may read a secret.
- `fast` gains `ci_release.py --self-test`.
