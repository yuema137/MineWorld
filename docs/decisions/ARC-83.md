## ARC-83 — Layer 4: the nightly workflow, its verdicts and its baselines

**Date** 2026-10-10 · **Status** decided; live from S13 PR 13c · **Approved by** the operator (13c QC-1:
`issues: write` on the report job; QC-2) and the primary session (13c QC-3 … QC-12, design freeze
2026-10-10) · **Relates to** `ARC-48`, `ARC-49`, `DEP-17`, `DEP-19`, `DEP-26`, `DEP-45`,
[`ENGINEERING_STANDARDS.md`](ENGINEERING_STANDARDS.md) §16 · **Design**
`.structured-coding/plans/mvp0/pr-13c-nightly.md`

**Problem.** `ENGINEERING_STANDARDS.md` §16's fourth layer — "long-running stability tests … can run
separately from the fastest PR loop … simulate 100 days, … restart server repeatedly, replay event log" —
did not exist: no workflow had a schedule. Four further gaps had no owner. `AC-8` was compared at 300
days in memory and 30 days saved only. A behaviour change identical on every platform passed every
check, because the towns' long-run digests were recorded in prose only. The 25 `#[ignore]`d Godot tests
and the 3D slice probes never ran in CI. Nothing sampled a commit twice, so a flaky test was invisible
until it failed a PR.

**Options considered** (design §5). Scheduling: GitHub Actions `on.schedule` with a read-only `gh api`
query for the last nightly's commit (chosen); an `actions/cache` marker; a third-party "skip if
unchanged" action; running every night regardless. Reporting: one rolling issue written by `gh` with the
automatic token (chosen); a third-party issue action or `actions/github-script`; the step summary and
GitHub's failure e-mail only (the fallback); a webhook (needs a secret); a committed status file (needs
`contents: write` on a protected branch). Flake detection: repeating the unchanged default suite on the
night's commit (chosen); `cargo-nextest` retries (a retry turns a flaky failure green); GitHub's re-run
of failed jobs; a paid flaky-test service. The restart loop: a standard-library script driving the real
binary with the repository's own Python SDK as the client (chosen); a new Rust test (it would join the
default suite); the Godot client (heavier, and `run.sh reconnect` already covers it).

**Decision.**
1. **One workflow, `.github/workflows/nightly.yml`**, separate from `ci.yml`, so that no required job
   gains a trigger and the one write permission lives in its own file. Triggers: `schedule` at
   `17 10 * * *` (10:17 UTC, off the hour and outside the operator's working day); `workflow_dispatch`
   with inputs `force` (boolean, default true) and `groups` (default `all`); and pushes to
   `scratch/**-nightly`, the only way to exercise the workflow before it exists on `main`. Concurrency
   `nightly-<ref>`, never cancelled. The jobs name layers of `scripts/ci_layer.py`, never commands
   (`I-S13-9`), and reuse `ci.yml`'s composite actions and pinned actions.
2. **The gate** (`scripts/ci_nightly.py gate`, `actions: read`). A scheduled night runs iff `main`'s head
   differs from the `head_sha` of the last `nightly.yml` run on `main` that concluded `success` or
   `failure`; with no such run it runs. A dispatch on `main` with `force=false` decides the same way;
   `force=true`, a dispatch elsewhere and a scratch push always run. Any `gh api` error runs the night,
   with the error as the reason: a needless night costs minutes, a wrongly skipped one hides a defect.
   `groups` (`parity-long`, `stability`, `clients`, `repeat`) come from the dispatch input or, on a
   scratch push, from the token before `-nightly` in the branch name when it names a group; an unknown
   group is an error, never "none".
3. **Five groups.**
   - `parity-long`: `scripts/ci_parity.py record --profile long` on `AC-8`'s four legs (the runtime image
     on Linux x86_64 and Linux arm64, natively on `macos-26` and `windows-2025`). The long record keeps
     the default profile's keys and adds `summary-1000` (memory, seed 7, 1 000 days) and `summary-300s`
     with every stored byte of a 300-day save. `[source]` names `profile long`; `compare` refuses records
     of different profiles. The default record is unchanged byte for byte. Then the comparison, and the
     baseline check of point 5.
   - `stability`, on Linux (toolchain container), macOS and Windows: `scripts/ci_stability.py restarts`
     kills and restarts `mineworld server` ten times on one save while the SDK client joins with the
     invite and asks for the `perceived` stream from the cursor it reached (a restart drops every hold
     and resume secret, `server/PROTOCOL.md` §4.2). Each cycle's oracle is the server's own `GET /status`:
     the same `instance`, a `revision` not behind the last one seen before the kill, `faults` 0; then
     `mineworld replay` of the killed save. `scripts/ci_stability.py replay` saves every world 300 days and
     replays it. CA-13's ignored test (`perceived.rs`) runs here. Scratch must be left clean.
   - `clients`: the 25 `#[ignore]`d Godot tests (`client_2d`, `client_2d_interact`,
     `client_2d_interact_stub`, `client_settings`) on all three operating systems; on Linux and macOS
     also `clients-probes` — the 3D slice's `--drive`, `--world --link` and `--world --target`, whose
     verdict is parsed (they exit 0 on failure), the protocol module's live checks, and
     `ac13_semantic_parity` over the regenerated evidence, which is judged by that test, not by a byte
     diff (the evidence holds ports and timings). Windows runs `clients` only: the probe launchers are
     bash scripts, and `clients-probes` moves onto S23 R-c's portable launcher when it merges (follow-up
     owned by S13). A windowed test that cannot show a window on one leg may be skipped there by name
     only if it passes on another leg the same night, and a coverage check refuses a test run on no leg.
     The one exception (QC-5b, primary ruling 2026-10-10): a windowed test that fails on every hosted
     leg for a recorded display reason is named in `ci_godot.py` as not run on hosted runners, reported
     in each night's notes, and carried to the operator's milestone-F hand checks; the coverage check
     accepts exactly that list, and a third name turns it red. Two tests are on it today
     (`two_d_display_settings_take_effect`, `the_language_chosen_in_2d_is_in_the_3d_clients_first_frame`).
   - `repeat`: the unchanged default suite run twice more per OS (`scripts/ci_repeat.py`), every
     repetition run even after a failure. A test that failed in one sample and passed in another of the
     same commit and OS — the two repetitions and `main`'s own push run — is a flake candidate; a test
     that failed in every sample is a failure. A repetition that failed before any test ran is
     infrastructure, never a flake.
   - `report` (below).
4. **Verdicts are PASS only on positive evidence.** Each job writes one verdict — PASS, FAIL or
   INCONCLUSIVE — from its layer's own result file and parsed sub-verdicts. A failed command inside the
   layer is FAIL; a job cancelled or timed out, a setup step that failed before the layer ran, a missing
   summary line, or a missing verdict artifact is INCONCLUSIVE. INCONCLUSIVE is red. Nothing is retried to
   green.
5. **Committed baselines, `scripts/baselines.txt`.** For every world, the long record's `summary-300`,
   `summary-1000`, `summary-300s`, `facts-300s` and `journal-300s` must equal the committed values;
   snapshots and the manifest are excluded, because they are derived storage and a storage-format change
   must not read as a behaviour change (they stay covered by cross-platform parity). A world with no
   baseline, or a baseline for a world that no longer exists, is a named FAIL. **A PR that intends to
   change a world's behaviour updates the file in the same PR, with a `reason <PR or step id>: <one line>`
   entry for each world it re-baselines** (`ci_parity.py baseline write` keeps those lines). Any
   platform's long record serves, because parity holds. A forgotten update turns the next night red with
   the world, the key, the commit range and the first differing line. The file is code for
   `ci_changes.py`.
6. **The report** (`scripts/ci_nightly.py report`): a step summary with one row per expected job, `main`'s
   own push run for the commit, flake candidates, baseline drift, and a benchmark (wall time per
   world-day per leg, 300-day save sizes, a trend over the last 14 nights). On `main` a red night opens
   one issue labelled `nightly` or comments on the open one; a green night comments and closes it. A
   scratch run whose branch ends `-issue-nightly` uses the label `nightly-scratch` instead. The job fails
   iff the night is red, so the run is red in the Actions list.
7. **Permissions.** `contents: read` by default; `gate` adds `actions: read`; `report` alone adds
   `issues: write`. The automatic `GITHUB_TOKEN` only; no secret. No `pull_request` trigger, so no fork's
   code runs with a write token; every value from a branch name, an input or a job output reaches a
   script through `env:`, never `${{ }}` inside `run:`; every action is pinned by full SHA, and
   `check_ci_pins.py` now checks that.

**Why not ourselves / why not the others.** Scheduling, reporting and process control are GitHub's and
the standard library's; the scripts are the glue the existing CI already is. A third-party action would
hold a token for a few lines of `gh`. Retries are declined because they are the defect flake detection
exists to see.

**Isolating interface.** `nightly.yml`'s triggers and the gate; the layers of `ci_layer.py`; the verdict
files; the report's last step (the issue), which can be dropped without touching the rest; `baselines.txt`
with `ci_parity.py baseline`.

**Accepted limitations.**
- The 1 000-day **saved** run and the `ASR-1` 640 MiB save-size assertion wait for S6 save retention
  (13c QC-6); until then 300 days is the saved horizon (a 1 000-day save is about 8 GB).
- No "simulate many agents" scale test: no world with many agents exists; it belongs to a performance
  lane when one does.
- Windows has no `clients-probes` until a portable launcher exists (above).
- Rendering is not judged in CI (`ARC-17`); windowed Godot tests check behaviour, not frames.
- Hosted runners only, free on a public repository (GitHub's published terms; billing cannot be read by
  the session). If the repository became private, the night would cost about 1 800 Linux-equivalent
  minutes and must shrink to Linux-only and weekly.
- GitHub disables a schedule after 60 days without repository activity; `MVP_STATUS.md`'s S13 row names
  the nightly, so a silent stop shows as a stale date.
