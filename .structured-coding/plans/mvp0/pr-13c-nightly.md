# PR 13c — Nightly stability, long parity and client checks in CI (S13, layer 4)

## DESIGN FROZEN 2026-10-10 (primary session)

```text
Design revision:        revision 1 (PR #158, first commit 29459bb) with §14.1's rulings applied and the
                        decision number corrected to ARC-83
Approved by / evidence: the coordinator's message relaying the operator's rulings (QC-1, QC-2) and the
                        primary session's rulings (QC-3 … QC-12, QC-8, decision ids), 2026-10-10 (§14.1)
Implementation base:    main at the start of implementation (exact commit recorded in N-C0's re-audit)
Execution contract:     §15 (filled at freeze)
Lifecycle:              FROZEN
```

Scope (§3), invariants (§4), acceptance and mutations (§7), the commit plan (§9) and the contract (§15)
are frozen. The ledger (§12), evidence, findings and bounded corrections stay writable. Implementation
starts in a fresh session (§15); this planning session does not implement.

*Superseded header:* `DRAFT — PR design, awaiting the primary session's review and the operator's answers
to the [OM] questions`.

- **Effort:** `mvp0`. **Step:** S13, [`step-14-ci.md`](step-14-ci.md). This document details the outline
  of step-14 §14 (and §7.3) to the commit. Where they differ, this document proposes the change and says
  why (§1.3); the step document is updated by the primary session, not by this one.
- **Parents and binding documents:** [`overall.md`](overall.md) §3 (S13), §5 "Still open" (QB-11);
  `step-14-ci.md` §§3.4–3.7, 4 (I-S13-1 … I-S13-9), 5.7, 13 (13b as merged: `ci_parity.py`, ARC-49), 15
  (13w as merged), 16 (13x: documentation-only changes skip the build);
  [`docs/ENGINEERING_STANDARDS.md`](../../../docs/ENGINEERING_STANDARDS.md) §§15–16;
  [`docs/DECISIONS.md`](../../../docs/DECISIONS.md) ARC-48 (and its notes of 2026-10-08/09), ARC-49,
  DEP-17, DEP-18, DEP-19, DEP-29, DEP-41; [`step-23-release.md`](step-23-release.md) §16.4 (one shared
  Godot download step); [`pr-s6-save-retention.md`](pr-s6-save-retention.md) ASR-1, ASR-11.
- **Written by:** the 13c design session. Worktree `/Users/yuema137/mineworld-worktrees/design-13c-ci`,
  branch `docs/13c-ci-design`, from `origin/main @ 98fe3e6` (2026-10-10).
- **Decision numbers:** `ARC-83` and `DEP-45`, assigned at freeze by the primary session (ARC-82 went to
  13c R-PK-1, doorway names, #156).

---

# 0. Summary for the reviewer

Layer 4 of `ENGINEERING_STANDARDS.md` §16 ("long-running stability … can run separately from the fastest
PR loop") does not exist yet: no workflow has a `schedule:` trigger, and ARC-48's `stability` and
`clients` rows are still "13c". This PR adds one scheduled workflow, `.github/workflows/nightly.yml`,
that runs once a night when `main` has moved, on macOS, Linux and Windows, and owns five things the
per-PR and per-push CI cannot afford or cannot see:

```text
parity-long   AC-8 at the long horizon: every world 1 000 days in memory and 300 days saved (every
              stored byte), on the four platforms of AC-8 (Darwin arm64, Linux x86_64 in the runtime
              container, Windows x86_64, Linux arm64), compared by 13b's ci_parity.py
baselines     the same long record checked against committed baselines (scripts/baselines.txt): an
              unintended behaviour change that is identical on every platform passes every check today
              (finding F-13c-3); tonight it turns the nightly red and names the world and the key
stability     the server killed and restarted ten times on one save while a client reconnects, then
              `mineworld replay` of every 300-day save, and S11-C's CA-13 test that needs a 300-day save;
              on all three OSes
clients       the 25 #[ignore]d Godot tests (2D client, interaction, settings) on all three OSes, and on
              Linux and macOS the 3D slice probes (verdict parsed: they exit 0 on failure) and the
              protocol module's live checks, all headless where Godot allows
repeat        flake detection: the unchanged default suite run twice more per OS on the night's commit;
              a test that fails in one sample and passes in another is named as a flake candidate
```

A red night opens (or comments on) one GitHub issue labelled `nightly`, and a green night closes it. That
needs `issues: write` on one job, with the automatic `GITHUB_TOKEN`; no secret is added anywhere. Every
runner is a standard GitHub-hosted runner, free on this public repository†. The estimated cost is about
**450 job-minutes per night** (§8), roughly a sixth more than the per-push CI already spends per day.

Nothing here becomes a required check. §11 gives the QB-11 recommendation the brief asks for: make
`test-macos` required now, `test-windows` after seven flake-free nights.

`†` marks a fact about GitHub's plans or limits taken from GitHub's published terms as this session
knows them. Billing cannot be read with the session's token (step-14 §2.2); the operator confirms each
† before relying on it.

---

# 1. Requirement and scope

## 1.1 Quoted requirements

`ENGINEERING_STANDARDS.md` §16:

> **Long-running stability tests** — Can run separately from the fastest PR loop. Examples: simulate 100
> days, simulate many agents, restart server repeatedly, replay event log.
> The principle does not change: at least some tests must exercise MineWorld as an actual running
> system.

`overall.md` §3, S13: "the four CI layers of `ENGINEERING_STANDARDS.md` §16 — fast structural checks,
core integration tests, scenario tests, long-running stability"; "CI runs the layers on the right
triggers".

`step-14-ci.md` §14.1–14.5 (the 13c outline), restated here as requirements:

- 13c owns the `schedule:` trigger; a `changed` gate skips a night when `main` has not moved; a
  `workflow_dispatch` input `force` overrides it.
- The nightly set: `stability`, `parity-long`, `clients`, a `benchmark` report.
- A red nightly reports and never gates a PR; it opens or updates one issue labelled `nightly`, "which
  needs `issues: write` on that job only (I-S13-7 is relaxed for one job; a question for the 13c
  design)".
- 13c never changes `fast`, `test`, or 13b's record format beyond adding the long profile, and never makes
  a nightly job a required check.
- All platforms: stability on Windows after 13w (now merged); a Windows leg for `clients` once the bash
  launchers have a Windows equivalent, "until then a recorded Linux-only reason"; a macOS leg for
  `clients` too.

The brief for this design (primary session, 2026-10-10) adds:

- long-run determinism, "e.g. 300-day towns and Lakeside digests vs recorded baselines";
- "AC-8 parity on all four platforms on main";
- Godot client suites (2D/3D) headless where possible;
- flake detection (repeat runs);
- how failures notify (issue or label, no secrets);
- runner-minute cost estimates; which jobs could later become required (the QB-11 recommendation);
- macOS, Linux and Windows coverage; no paid services.

Operator rules that bind this design:

- **All platforms** (operator, 2026-10-08; step-14 §13.0.1): designs and CI cover macOS, Linux and
  Windows.
- **Documentation-only changes do not run the build** (operator, 2026-10-09; step-14 §16). The nightly
  never runs on a pull request, so it cannot be triggered by a documentation PR. A night whose only new
  commits on `main` are documentation still runs (main moved), which is §16.4 point 5's rule "a push to
  `main` is always code". §5.2 explains why that is kept.
- **No paid services, no larger runners, no spending** (step-14 §13.12, §15.12 contracts).

## 1.2 Non-goals

- No change to `fast`'s or `test`'s commands, names or triggers, except the two bounded points of §4
  (I-13c-1): self-tests appended to `fast` (each well under 1 s, as 13b and 13x did), and `test`'s `if:`
  excluding `scratch/*-nightly` branches (as 13b excluded `-scenario`).
- No Rust, `Cargo.*`, test, `worlds/`, `clients/`, `presentation/` or `Dockerfile` edit (I-S13-1). A
  planted mutation on a scratch branch may touch them; it is never merged (I-13c-6).
- No re-tiering of the default suite (QS13-5 stands). The nightly runs **additional** programs; it moves
  nothing out of `test`.
- No required check. No repository setting (protection, required checks, rulesets, Actions policy). QB-11
  is a recommendation to the operator, not a change this PR makes.
- No "simulate many agents" scale test. §16 names it as an example; no world with many agents exists,
  and a scale test belongs to the performance lane when one does (recorded as a gap in ARC-83).
- No rendering or pixel acceptance in CI: `D-11` and `ARC-17` keep visual acceptance with the operator
  (step-14 §5.7). Windowed Godot tests run where a runner can open a window; they check behaviour, not
  frames.
- No Godot export or packaging job. That is S23's `export` job (DEP-41); 13c only provides the shared,
  pinned Godot download that S23 reuses (§3.5).
- No 1 000-day **saved** run before S6 save retention (#153) merges (§3.2, QC-6).

## 1.3 Where this design departs from the step-14 §14 outline

| # | Outline (§14) | This design | Why |
| --- | --- | --- | --- |
| D-1 | `on.schedule` added to `ci.yml`, beside its per-push jobs | **A separate workflow, `nightly.yml`.** | Every `ci.yml` job would otherwise need a `schedule` clause in its `if:`, and the required `fast`/`test` jobs would gain a third trigger they must not run on. A separate file has its own permissions (one `issues: write` job), concurrency (never cancelled) and triggers, and touches no required job. The layers stay shared: both workflows name layers of `ci_layer.py` and use the same composite actions (I-S13-9). |
| D-2 | 13b's `scenario`, `mac`, `linux-arm`, `windows`, `ac8`, `test-windows` re-run nightly "unchanged" | **Not re-run.** The nightly reads the conclusion of `main`'s own push run for the same commit and reports it. It re-runs only the default suite, as flake samples (`repeat`), and runs the long versions of parity (`parity-long`) | Those jobs already run on every `main` push (§2.1). Re-running identical work on the same commit duplicates evidence (test rules §9). The default suite is the exception, because a second sample of the same commit is exactly what flake detection needs |
| D-3 | `stability` (1): social-cafe 1 000 days `--save`, then `replay` | **Every world 300 days `--save`, then `replay`; 1 000 days in memory.** The 1 000-day saved run waits for save retention | A 1 000-day save is about 8 GB before S6 SR (§2.4); a macOS runner has about 14 GB of disk†. After SR, 300 days is ≤ 640 MiB (ASR-1), and the 1 000-day saved run becomes one line in the layer (QC-6) |
| D-4 | `stability` (2): market-town SIGKILLed at five points and resumed | **Dropped from 13c**, as already owned | The default suite already kills and resumes Market Town (`market_town.rs`), Lakeside (`milestone_e.rs`, PD-48), bodies-yard (`bodies_yard_restart.rs`) and the persistence program (`kill_and_resume.rs`) on every PR, on all three OSes since 13w. Five points instead of one adds no failure class. The server-level restart loop (D-5) is the new class |
| D-5 | `stability` (4): server restarted ten times "while a scripted client reconnects with its invite" | Kept, on all three OSes, with the Python SDK as the client | The SDK is the repository's own scripted client (`sdk/python`, S10 P3), already run against the real server in CI |
| D-6 | `clients` (2): `run.sh evidence` then `git diff --exit-code clients/protocol/evidence` | **Regenerate, then judge semantically**: `cargo test -p mineworld-cli --test ac13_semantic_parity` reads the regenerated evidence | The evidence holds server logs with ports and timings, so it is not byte-reproducible (§2.5). A byte diff would be red every night for no defect. The test that owns AC-13's evidence judges it |
| D-7 | `benchmark` over 30 nights | Over the last **14** nightly runs, from their small timing artifacts | Bounded download cost; 14 nights shows a trend. Information only |
| D-8 | "long-horizon parity of 300-day saves" | Plus **committed baselines** (`scripts/baselines.txt`) checked nightly | The brief's "digests vs recorded baselines" (F-13c-3) |

---

# 2. Audit (`main @ 98fe3e6`, 2026-10-10)

## 2.1 What runs where today

`.github/workflows/ci.yml` is the only workflow. Durations are measured from two completed `main` push
runs (38083255074 on `8c710d1`, 38076242331 on `d3bca96`) and the last three code PRs' runs.

| Job | Runs on | PR (non-draft) | push `main` | dispatch | `scratch/**` push | Required | Measured wall (min) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `changes` | ubuntu-24.04 | yes | yes | yes | yes | — | 0.2 |
| `fast` | ubuntu-24.04, container | yes (docs layer if docs-only) | yes | yes | yes | **yes** | 1.6–1.9 |
| `test` | ubuntu-24.04, container | yes | yes | — | yes, except `-image`, `-scenario` | **yes** | 19.4–24 |
| `test-windows` | windows-2025, native | yes | yes | yes | as `test` | no (QB-11) | 32–36 (43 cold, per the brief) |
| `test-macos` | macos-26, native | yes | yes | yes | as `test` | no (QB-11) | 18.5–22 |
| `platforms` ×2 | macos-26, windows-2025 | yes | yes | — | as `test` | no | 5–6 (mac), 7.5–11.5 (win) |
| `python` ×3 | ubuntu (container), windows-2025, macos-15 | yes | yes | — | as `test` | no | 3–4 (linux, mac), 10 (win) |
| `scenario` | ubuntu-24.04 | — | yes | yes | `-scenario` only | no ("blocks main's health") | 6.7–7.4 |
| `linux-arm` | ubuntu-24.04-arm | — | yes | yes | `-scenario` | no | 5.7–6.7 |
| `mac` | macos-26 | — | yes | yes | `-scenario` | no | 4.7–5 |
| `windows` | windows-2025 | — | yes | yes | `-scenario` | no | 7.3–7.6 |
| `ac8` | ubuntu-24.04 | — | yes | yes | `-scenario` | no | 0.2 |
| `schedule` | — | — | — | — | — | — | **no workflow has one** |

A completed `main` push run therefore costs about **139 job-minutes** (sum of the first run above), and a
code PR head about 110.

## 2.2 Findings

- **F-13c-1 — Layer 4 does not exist.** No `schedule:` trigger anywhere (`grep -rn schedule
  .github/` → nothing). ARC-48's table rows `stability` and `clients` ("nightly; workflow_dispatch (13c)")
  are unimplemented.
- **F-13c-2 — Most `main` pushes never run.** Of the last 60 `main` push runs, **31 were cancelled**
  before any job started, 17 succeeded, 11 failed, 1 was in progress (`gh run list --branch main --event
  push --limit 60`). A cancelled run has `jobs: []` (run 38066402664). The cause is `concurrency` with
  `cancel-in-progress: false` on `main`: GitHub keeps one running and one pending run per group, and a
  newer pending run replaces the older one. So only the head of each merge burst is verified on `main`.
  That is acceptable, because the head contains the intermediate commits, but it means "every main push
  runs AC-8" really means "the last commit of each burst". The nightly's gate keys on the commit, not on
  pushes (§3.1).
- **F-13c-3 — An identical behaviour change on every platform passes every check.** The towns' and
  Lakeside's 300-day digests are recorded in prose only: `docs/MVP_STATUS.md` (market-town `d5db8988…`,
  Lakeside `97dac8fc…`), step ledgers, and "not asserted by a test" by design (step-16 PD-48, EA-6). `ac8`
  compares platforms with each other on one commit, so a change that moves every platform's facts equally
  is AC-8-green. The default suite asserts properties of runs (activity, causation, byte-identical twins),
  not their exact history. A PR that unintentionally changes what a world does therefore merges with
  every check green. The only guard today is each lane's discipline to re-measure and re-record.
- **F-13c-4 — Godot tests exist but never run in CI.** 25 tests start Godot and are `#[ignore]`d so that
  `cargo test` reports them ignored, never passed, without Godot: `client_2d.rs` 8,
  `client_2d_interact.rs` 5, `client_2d_interact_stub.rs` 3, `client_settings.rs` 9 (`grep -c
  '^#\[ignore'`). They find Godot through `GODOT` or `PATH` (`tools/cli/tests/godot2d/mod.rs:35`) and fail
  loudly without it. Three need a real window: `two_d_display_settings_take_effect` (AC-SET-11),
  `three_d_switches_language_live_and_settings_never_reach_the_server` and
  `the_language_chosen_in_2d_is_in_the_3d_clients_first_frame` (`client_settings.rs:541, 582, 683`).
- **F-13c-5 — The 3D slice probes still exit 0 on failure.** `slice_probe.gd:160` calls
  `get_tree().quit(0)` after every mode (F-5 is still open for the 3D slice). Each mode prints a final
  line: `all drive checks pass` or `<n> DRIVE CHECKS FAILED` (`slice_probe.gd:1084`), and likewise
  `link`, `target`, `character`, `scale` (`slice_probe_world.gd:241, 424`). An early abort prints `FAIL: …`
  and returns **without** a summary line (`slice_probe_world.gd:128, 263, 270`). A verdict must therefore
  be parsed, and a missing summary is INCONCLUSIVE.
- **F-13c-6 — Other ignored tests.** `perceived.rs:434` (CA-13, "needs a 300-day save (minutes to make);
  run explicitly in S11-C's close") is a guard nobody runs since S11-C closed: the nightly runs it.
  `bodies_yard.rs:366` ("evidence for the AO-2 ruling (three seeds)") and `deltas.rs:41` ("CP-C1: a 60 s
  measurement whose numbers decide DEP-15") are measurements, not guards; the nightly does not run them
  (§3.4).
- **F-13c-7 — Infrastructure failures exist and look like failures.** `main` run 38027699190's
  `linux-arm` failed in the image build: "failed to fetch anonymous token … ghcr.io … i/o timeout". A
  nightly classifier must keep "the check ran and failed" apart from "the check never ran" (test rules
  §16: FAIL vs INCONCLUSIVE).
- **F-13c-8 — `workflow_dispatch` needs the workflow on the default branch.** GitHub dispatches only
  workflows present on `main`†. 13b could dispatch its branch because `ci.yml` existed on `main`;
  `nightly.yml` does not. Before merge, the nightly can only be exercised by a `push` trigger on scratch
  branches (§3.1, `scratch/**-nightly`), the pattern ARC-48 already uses for `-image` and `-scenario`.
- **F-13c-9 — `check_ci_pins.py` does not check action pins.** It checks the toolchain, the Dockerfile's
  digests and Debian release (`scripts/check_ci_pins.py:37–113`). Full-SHA pinning of `uses:` (I-S13-7) is
  a convention kept by review. A job that holds `issues: write` makes an unpinned action more costly;
  §3.6 proposes the check (QC-7).
- **F-13c-10 — QB-11's criterion is already met.** Every completed `main` push run since 13w merged
  (`0345922`, 2026-10-10T05:31Z) is green on `test-windows` and `test-macos` (17 successes, no failure;
  the earlier failures are all before 13w or are S10's `python` golden frame, since fixed). §11 uses this.

## 2.3 Instruments that exist and are reused

| Instrument | What it gives 13c | Evidence |
| --- | --- | --- |
| `scripts/ci_parity.py` (13b, ARC-49) | `record` (native or image), `compare` (G-1 … G-5, grouping, first differing line or chunk), `--self-test`; constants `SEED = 7`, `LONG_DAYS = 300`, `SAVE_DAYS = 30`, `CHUNK_ROWS = 1000`; worlds enumerated from `worlds/*/world.yaml`; `PACK_ROOTS` (ARC-77) | `scripts/ci_parity.py:1–72, 302–322, 344–372` |
| `scripts/ci_layer.py` | the only layer → command table; `fast`, `core`, `docs`, `parity`, `platforms`, `python`, `python-smoke`; `sys.executable` for Python on Windows | `scripts/ci_layer.py:47–157` |
| `.github/actions/layer` | the toolchain container, Cargo caches, prune | `action.yml` |
| `.github/actions/native` | native macOS/Windows runs of a layer, caches per OS and layer, prune | `action.yml` |
| `mineworld replay <world> --save DIR` | re-executes a save's whole history and checks it | `tools/cli/src/main.rs:8, 482` |
| `GET /status` | `instance` (the same world across restarts), `revision`, `faults`, `clients` | `server/PROTOCOL.md` §5.7 |
| `sdk/python` (`mineworld_sdk.session`) | `connect`, `join` with `resume` | `sdk/python/src/mineworld_sdk/session.py:187, 216` |
| `mineworld-test-support::process` (13w) | portable kill; Ctrl-Break on Windows | step-14 §15.3–15.4 |
| Godot 4.7.2-stable official builds | `Godot_v4.7.2-stable_linux.x86_64.zip`, `…_win64.exe.zip`, `…_macos.universal.zip`, `SHA512-SUMS.txt` | `gh api repos/godotengine/godot-builds/releases/tags/4.7.2-stable` (read 2026-10-10); the operator's Mac runs `4.7.2.stable.official.ed1daf0bf` |

## 2.4 Sizes and times that drive the budget

From step-14 E-13b-0 and MVP_STATUS, on the operator's Mac, release build:

```text
social-cafe   300 d memory 20.6 s     30 d save 2.2 s (249 MB)     300 d save ≈ 2.4 GB (§2.3)
market-town   300 d memory 28.3 s     30 d save 2.5 s (258 MB)     300 d save 37.6 s, 3.5 GB (PD-50)
bodies-yard   300 d memory 14.1 s     30 d save 2.8 s (246 MB)
lakeside      300 d save 22.6 s, 2.0 GB (MVP_STATUS, Milestone E)
after S6 SR   market-town 300 d save ≤ 640 MiB, snapshots ≤ 8 MiB (ASR-1); facts and journal unchanged (ASR-11)
```

Not measured: 1 000-day runs. If cost is linear in days, a 1 000-day in-memory run is about 70–95 s per
town on the laptop. N-C2 measures it before anything depends on the figure.

## 2.5 The protocol evidence is not byte-reproducible

`clients/protocol/run.sh evidence` writes server logs and transcripts into `clients/protocol/evidence/`
(`run.sh:36, 45–57`). The server listens on an OS-chosen port (`--listen 127.0.0.1:0`), and the logs carry
it. Every regeneration in history is its own commit ("evidence regenerated on the merged head",
`eeb3992`, `bb00db2`, `f43d73d`). The semantic owner of that evidence is
`tools/cli/tests/ac13_semantic_parity.rs:42`, which reads it. D-6 follows from this.

---

# 3. Design

## 3.1 The workflow: `.github/workflows/nightly.yml`

```text
on:
  schedule:            cron "17 10 * * *"     (10:17 UTC = 03:17 PDT: off the hour, outside the
                                               operator's working day; QC-9)
  workflow_dispatch:   inputs  force  boolean, default true   (run even if main has not moved)
                               groups string, default "all"   (comma list: parity-long, stability,
                                                               clients, repeat)
  push:                branches ["scratch/**-nightly"]        (pre-merge and mutation evidence, F-13c-8)

permissions:           contents: read                         (workflow default)
concurrency:           group nightly-${{ github.ref }}, cancel-in-progress: false
```

Jobs (no matrix, undecorated names, so that each is addressable by name, as ARC-48 requires of check
names):

```text
job                  runs-on            layer / script                                   timeout
gate                 ubuntu-24.04       ci_nightly.py gate (permissions: actions: read)    5
parity-long-linux    ubuntu-24.04       runtime image linux/amd64; ci_parity.py record      90
                                        --profile long --image
parity-long-arm      ubuntu-24.04-arm   runtime image linux/arm64; same                     90
parity-long-mac      macos-26           native layer parity-long                            90
parity-long-windows  windows-2025       native layer parity-long                            90
parity-long          ubuntu-24.04       ci_parity.py compare --profile long; then            15
                                        ci_parity.py baseline check (needs the four legs;
                                        if: always())
stability-linux      ubuntu-24.04       container layer stability                           90
stability-mac        macos-26           native layer stability                              90
stability-windows    windows-2025       native layer stability                              90
clients-linux        ubuntu-24.04       godot action; native layers clients, clients-probes  75
                                        (under xvfb-run, §3.4)
clients-mac          macos-26           godot action; native layers clients, clients-probes  75
clients-windows      windows-2025       core.symlinks; godot action; native layer clients   75
repeat-linux         ubuntu-24.04       container layer core-repeat                         120
repeat-mac           macos-26           native layer core-repeat                            120
repeat-windows       windows-2025       native layer core-repeat                            120
report               ubuntu-24.04       ci_nightly.py report (needs every job; if: always();  10
                                        permissions: contents read, actions read, issues write)
```

- **Every job but `gate` and `report`** `needs: gate` and runs only if `needs.gate.outputs.run == 'true'`
  and its group is in `needs.gate.outputs.groups`. A job skipped by the gate is reported as "skipped:
  <reason>", never as PASS.
- **`parity-long`** and **`report`** run `if: always()` (after the gate decided to run), so that a failed
  or cancelled leg is a red verdict, never a skipped one (I-13b-4's rule, carried).
- **Checkouts.** Every job that runs the default suite or a history-reading test (`repeat-*`) checks out
  with `fetch-depth: 0`, `filter: blob:none` (I-S13-2, ARC-35). `parity-long-mac` and
  `parity-long-windows` use 13b's sparse checkout (no `clients/`, `presentation/` beyond the manifests).
  `clients-*` need the whole tree. `persist-credentials: false` wherever the job does not read history
  on demand.
- **Pins.** Every `uses:` names a full commit SHA already pinned in `ci.yml` (checkout, cache,
  upload/download-artifact, setup-buildx, build-push). No new third-party action is added.

### 3.1.1 The gate (`scripts/ci_nightly.py gate`)

```text
event                       decision
schedule                    run iff HEAD ≠ the head_sha of the last nightly.yml run on main that
                            concluded success or failure (a run past the gate), excluding this run;
                            no such run → run
workflow_dispatch on main   force=true → run; force=false → as schedule
workflow_dispatch elsewhere run (a branch's own evidence)
push to scratch/**-nightly  run; groups from the branch name (below)
anything else, or any gh    run, with the reason printed (fail open towards running: a night that runs
api error                   needlessly costs minutes; a night skipped wrongly hides a defect)
```

- It reads runs with `gh api repos/{owner}/{repo}/actions/workflows/nightly.yml/runs?branch=main&…`,
  read-only, `GH_TOKEN: ${{ github.token }}`, `permissions: actions: read`.
- **Groups.** Dispatch: the `groups` input. Scratch push: the token before `-nightly` if it names a group
  (`scratch/13c-mn4-clients-nightly` → `clients`), otherwise `all`. An unknown group name is an error
  (exit 1, the run is red), never "none".
- Outputs: `run` (`true|false`), `groups`, `reason`, `last` (the previous nightly's run id and commit),
  `main-run` (the conclusion and id of `ci.yml`'s `push` run for this commit, if one completed).

### 3.1.2 Verdicts

Each job ends with an `if: always()` step that writes `verdict-<job>.txt` and uploads it as an artifact:

```text
job       <name>
commit    <sha>
verdict   PASS | FAIL | INCONCLUSIVE
detail    <one line per finding: the failing command, the parsed summary line, the flake candidate …>
```

- **PASS** only if the layer's step concluded `success` and every parsed verdict inside it is PASS.
- **FAIL** if the layer's step concluded `failure` and the failing command was reached (the layer prints
  "layer X FAILED at: <command>", `ci_layer.py`).
- **INCONCLUSIVE** if the step was cancelled or timed out, if a setup step failed before the layer ran
  (F-13c-7's image pull), or if a parsed verdict was missing (F-13c-5).
- `report` treats a job with **no verdict artifact** as INCONCLUSIVE. INCONCLUSIVE is red: a check that
  did not run is not a pass.

### 3.1.3 The report (`scripts/ci_nightly.py report`)

Inputs: `toJSON(needs)` (as an environment variable, never interpolated into a script), the downloaded
`verdict-*`, `timings-*` and `repeat-*` artifacts, the gate's outputs.

1. **Gate skipped the night:** the step summary says "skipped: main unmoved since run <id> on <sha>";
   the job succeeds; no issue is touched.
2. **Otherwise**, a step summary with:
   - one row per expected job: verdict, wall time, detail (the expected job list is a constant in the
     script, so a job missing from `needs` is a red row, not an absent one);
   - `main`'s own push run for this commit (F-13c-2): its conclusion and link, or "no completed push run
     for this commit";
   - **flake candidates** (§3.5);
   - **baseline drift** (§3.3), with the commits between the last green nightly and this one
     (`git log --oneline <last>..<sha>`);
   - **benchmark**: wall time per world-day for each world and leg, and the 300-day save sizes (ASR-1's
     quantity, reported; asserted only once SR merges and only if QC-6 says so), with the trend over the
     last 14 nightly runs from their `timings-*` artifacts (D-7).
3. **The issue** (only for runs on `main`, or on a scratch branch whose name ends `-issue-nightly`; §3.6):
   - red, no open issue with the label → `gh issue create --label nightly` titled "Nightly CI is red on
     main", body = the summary;
   - red, an open issue → `gh issue comment` with the new summary (one issue, never a second);
   - green, an open issue → comment "green on <sha>, run <id>" and `gh issue close`.
   The label is created if missing (`gh label create nightly`). A scratch run uses the label
   `nightly-scratch` instead, so evidence never touches the real issue.
4. **Exit status:** the `report` job fails iff the night is red, so the run is red in the Actions list and
   GitHub's own notification of a failed scheduled run reaches the account that last edited the cron†.
   No other notification channel is added (no Slack, no e-mail action, no secret).

## 3.2 `parity-long`: AC-8 at the long horizon

`ci_parity.py record --profile long` keeps the `default` profile's keys and adds two:

```text
default (13b, unchanged)   validate · summary-300 (memory, header kept) · line-300.* · summary-30s ·
                           line-30s.* · manifest · facts/journal/snapshots rows, digests, chunks (30 d save)
long adds                  summary-1000 <n> <sha>  · line-1000.*      memory, seed 7, 1 000 days
                           summary-300s <n> <sha>  · line-300s.*      save mode, seed 7, 300 days
                           manifest-300s · facts-300s/journal-300s/snapshots-300s rows, digests, chunks
```

- The `[source]` section gains `profile long`. `compare` requires every record to name the same profile
  (a new G-2 clause), so a default and a long record are never compared as equals. **The record format
  of the default profile is unchanged byte for byte** (I-13c-2): `record-format` stays `1`; `profile` is
  written only when it is not `default`, and an absent `profile` reads as `default`.
- **Why these horizons.** 1 000 days is past every horizon the default suite or `ac8` reaches (300), and
  exercises Market Town's weather replay across years (3 653 days of record, TW-d) and the calendar's
  floats (TW-a) for longer. 300 days saved gives every stored byte at the horizon E-RS0 fixed, which
  `ac8`'s 30-day saves cannot (F-13b-3).
- **Saves one at a time.** Each world's save is hashed and removed before the next world runs, as
  `record_world` already does (`ci_parity.py:308–327`). Peak disk is one 300-day save (≤ 3.5 GB today,
  ≤ 640 MiB after SR) plus the release build.
- **Timings are not in the record.** `record --timings F` writes a separate JSON file (per world and run:
  wall seconds, days, save bytes, rows). A record holds no wall-clock value by construction (13b), and
  stays byte-identical across runs of one commit on one platform.
- **The four legs** are exactly 13b's: the runtime image on `ubuntu-24.04` (linux/amd64) and on
  `ubuntu-24.04-arm`, natively on `macos-26` and on `windows-2025`. The image legs reuse 13b's GHA cache
  scopes `runtime` and `runtime-arm64`. The native legs run a new layer `parity-long` through
  `.github/actions/native`.
- **The comparison** is `ci_parity.py compare` over the four long records: G-1 … G-5 as merged, G-3
  requiring Darwin arm64, Linux x86_64 (container) and Windows x86_64, and Linux arm64 compared when
  present (it is always present in the nightly).

## 3.3 Baselines: `scripts/baselines.txt`

**Problem.** F-13c-3. **Claim.** For every world, the long record's history keys equal the committed
baseline unless a reviewed commit changed the baseline.

```text
# MineWorld long-run baselines: ci_parity.py's long profile, seed 7. Read by the nightly (ARC-83).
# Regenerate from any platform's long record (they are equal, ARC-49):
#   python3 scripts/ci_parity.py baseline write <record> > scripts/baselines.txt
[world lakeside]
summary-300    <n> <sha-256>      memory, 300 days, header kept (the key ac8 records on every main push)
summary-1000   <n> <sha-256>
summary-300s   <n> <sha-256>      save mode, 300 days: the deterministic() lines of the default suite
facts-300s     <rows> <sha-256>
journal-300s   <rows> <sha-256>
[world market-town]
…
```

- **Keys chosen, and why.** The summaries and the log (facts, journal) are what a world *did*. Snapshots
  and manifests are excluded: they are derived storage, and a storage-format change (S6 SR re-encodes
  snapshots; ASR-11 keeps facts and journal identical) must not read as a behaviour change. Snapshots stay
  covered by cross-platform parity. Chunks are excluded (location comes from `diff`, below).
- **`ci_parity.py baseline check <record> <baselines>`** exits 0 only if the baseline's world set equals
  the record's enumerated worlds (a world with no baseline is FAIL "no baseline for world X"; a baseline
  for a world that no longer exists is FAIL) and every key is equal. Otherwise it exits 1 naming each
  world and key with both values.
- **Locating a drift.** `ci_parity.py diff <old record> <new record>` prints, per differing world, the
  first differing summary line (both texts) and the first differing table chunk with its key range,
  regardless of platform. The report runs it against the last green nightly's Linux x86_64 long record
  (an artifact, 90-day retention†), so a red baseline arrives with the first changed day.
- **Baseline check runs once**, on the Linux x86_64 record, inside the `parity-long` job, after
  `compare`. Since `compare` proved every platform's record equal, checking one is checking all; if
  `compare` failed, the baseline check still runs and its verdict is reported separately (two findings,
  never one hiding the other).
- **Who updates it (QC-3, ruled yes).** A PR that intends to change a world's behaviour updates
  `scripts/baselines.txt` in the same PR, **with a reason line** for each changed world (a `reason <PR or
  step id>: <one line>` entry in that world's section, which `baseline check` ignores and `baseline write`
  keeps), as step-11 I-7 already requires prose digests to be re-baselined "here, and only here". It needs no Mac: any platform's long record serves, because parity
  holds. A PR author gets a record either locally (`ci_parity.py record --profile long --binary …`, about
  10 minutes on a laptop, N-C2 measures) or by pushing `scratch/<name>-parity-long-nightly`. If a PR
  forgets, the nightly turns red the next night, the issue names the world, the key and the commit range,
  and a follow-up commit records the new baseline with the reason. The nightly never blocks a PR, so this
  costs a red night, not a blocked merge.
- **Fail closed.** The file is read by CI, so it is code for `ci_changes.py` (not in the docs set): a PR
  that only edits baselines still runs `fast` and `test`.
- **Initial values.** Recorded at N-C2 from a long record on the implementation head, and cross-checked
  where a ledger already states the same quantity: market-town `summary-300s` against TW-d's
  `d5db8988…` and Lakeside `summary-300s` against Milestone E's `97dac8fc…` (both are the default suite's
  `deterministic()` lines of a saved 300-day run), and every `summary-300` against the same commit's
  `ac8` record. A mismatch is a finding recorded, never a silent overwrite (it means a ledger value went
  stale).

## 3.4 `clients`: Godot in CI

### 3.4.1 Getting Godot: `.github/actions/godot` and `scripts/ci_godot.py fetch`

- The official Godot 4.7.2-stable build for the runner's OS (Linux x86_64 zip, Windows `win64.exe` zip,
  macOS universal zip), downloaded from the `godotengine/godot-builds` release with the Python standard
  library, checked against **SHA-512 values pinned in `scripts/ci_godot.py`** (not against the release's
  own `SHA512-SUMS.txt`, which would only detect corruption, not a replaced asset), unpacked, cached with
  `actions/cache` keyed on the version and the pinned hash, and exported as `GODOT` (the variable
  `godot2d::godot()` and `mineworld-slice` read).
- On Windows it uses the archive's `_console.exe`, the variant that writes to a console (W-9).
- It runs `godot --version` and fails unless the output starts with `4.7.2.stable.official`.
- **Shared with S23** (step-23 §16.4: "13c and R-b share one pinned download step; one owner: whichever
  lands first"). If 13c lands first, the action is 13c's and S23's `export` job reuses it, adding the
  export templates (DEP-41). If S23 lands first, 13c reuses S23's and adds only what it lacks. Either way
  there is one pinned Godot version in the repository (QC-10).
- **Linux display.** `clients-linux` installs `xvfb` and `mesa-vulkan-drivers` from the Ubuntu archive
  (`apt-get install --no-install-recommends`, the runner's own mirror) and runs the `clients` layer under
  `xvfb-run -a`, so the three windowed tests (F-13c-4) get a display and a software Vulkan device. Headless
  tests pass `--headless` themselves and ignore the display.

### 3.4.2 Layers

```text
clients          all three OSes
  cargo build -p mineworld-cli
  cargo test -p mineworld-cli --test client_2d --test client_2d_interact
             --test client_2d_interact_stub --test client_settings -- --ignored --test-threads=1
  check_scratch.py left --target-dir target

clients-probes   Linux and macOS (bash launchers, W-9)
  python3 scripts/ci_godot.py slice --drive              ← ./mineworld-slice --drive, verdict parsed
  python3 scripts/ci_godot.py slice --world --link        ← verdict parsed
  python3 scripts/ci_godot.py slice --world --target      ← verdict parsed
  bash clients/protocol/run.sh evidence                   ← exits non-zero on failure (run.sh:13)
  bash clients/protocol/run.sh affordances | reconnect | perceived | deltas | admin
                                                          ← each exits non-zero on failure
  cargo test -p mineworld-cli --test ac13_semantic_parity ← judges the regenerated evidence (D-6)
```

- **The verdict parser** (`ci_godot.py slice …`) runs the launcher, streams its output, and decides:
  - PASS: exactly one line `all <mode> checks pass` and no line beginning `FAIL`;
  - FAIL: a line `<n> <MODE> CHECKS FAILED`, or any line beginning `FAIL` (after stripping indentation);
  - INCONCLUSIVE: neither summary line (an early return, a crash, a hang killed by the timeout), and the
    process exit status alone never decides PASS.
  It exits 0, 1 or 3 respectively and appends the decisive lines to the job's verdict detail. When S14
  makes the probe exit non-zero (F-5), the parser still applies: a non-zero exit with no summary is FAIL.
- **Windows.** `clients-windows` runs `clients` only. Before checkout it runs `git config --global
  core.symlinks true` (W-11), then checks that `clients/2d/mineworld` and `clients/3d-spike/mineworld` are
  directories, failing by name otherwise. `clients-probes` is not run on Windows, with the reason
  recorded in ARC-83: the launchers are bash scripts that use `kill`, `seq` and `sed`, and a portable
  launcher is S23 R-c's `mineworld-launch` (open, #148). When R-c merges, `clients-probes` moves onto it
  for all three OSes (a follow-up row in ARC-83, owner S13).
- **Windowed tests on a runner that cannot show a window (R-1).** Discovery at N-C6. If one of the three
  windowed tests fails on a leg for a display or GPU reason (not a behaviour reason), the pre-authorized
  fallback (QC-5) is: that leg's layer variant passes `--skip <test name>` for exactly those tests, each
  named in the layer table with its reason, and the nightly's coverage check (below) requires that every
  skipped test **passed on another leg the same night**. A behaviour failure is never skipped.
- **Coverage check.** `ci_godot.py coverage` lists the ignored Godot tests with `cargo test … -- --ignored
  --list` and fails if any test is run on no leg, so a skip list cannot grow into silence.
- **Import cache.** The 3D project's first `--import` processes about 195 MB of assets. The `.godot/`
  import directories of the three projects are cached, keyed on a hash of the projects' assets and
  `project.godot` files, and never committed. N-C6 measures cold and warm.

## 3.5 `repeat`: flake detection

- **Layer `core-repeat`:** `cargo test --workspace --no-run`, then `python3 scripts/ci_repeat.py --times 2
  --summary repeat-<os>.txt -- cargo test --workspace --no-fail-fast`, which runs the suite twice,
  streams the output, runs `check_scratch.py left` after each run, and writes per run: exit status, the
  failing test names (`test <name> ... FAILED` lines and `--test <target>` of `error: test failed`), and
  wall time. It runs **every** repetition even when one fails, and exits non-zero if **any** failed.
  Nothing is retried to green (I-S13-4): a repetition exists to sample, and every failure is reported.
- `--no-fail-fast` is used so that one failing binary does not hide another's result in a sample; it
  changes reporting, not what passes.
- **Three samples per OS per commit:** the `main` push run (when it completed; F-13c-2) and the two
  nightly repetitions.
- **Flake candidate:** a test that failed in at least one sample and passed in at least one other sample
  of the same commit on the same OS. The report lists each with its OS, the samples and the failure line.
  A test that failed in every sample is a **failure**, not a flake.
- **Why not more repetitions.** Two extra samples per night is about 30 job-minutes on Linux and 40 on
  Windows. Over a week that is 14 samples per OS per test, enough to see a flake with a 1-in-10 rate with
  probability above 75 % (1 − 0.9¹⁴). More would be cheap in money (free†) but not in macOS concurrency
  (R-12).
- **Infrastructure versus product.** A repetition that fails before any test ran (build, tool, network)
  is INCONCLUSIVE, named as such (F-13c-7), and never reported as a flake candidate.

## 3.6 Permissions, triggers and safety

- Workflow default `permissions: contents: read`. `gate`: `actions: read`. `report`: `contents: read`,
  `actions: read`, `issues: write`. **This is the one relaxation of I-S13-7**, proposed as QC-1 `[OM]`.
- **No secret.** The automatic `GITHUB_TOKEN` (`${{ github.token }}`) is passed as `GH_TOKEN` to `gh` in
  `gate` and `report` only. No repository or environment secret is created or read.
- **Who can run it.** `schedule` runs on `main` only. `workflow_dispatch` and pushes to `scratch/**`
  require write access. There is no `pull_request` or `pull_request_target` trigger, so code from a fork
  never runs with `issues: write`.
- **Script injection.** No `${{ }}` expression carrying a branch name, input or job output is interpolated
  into a `run:` script; each is passed through `env:` and quoted, as `ci.yml`'s `changes` job already does.
  N-C5's review checks every `run:`.
- **Action pins (F-13c-9, QC-7).** `check_ci_pins.py` gains one rule: every `uses:` in
  `.github/workflows/*.yml` and `.github/actions/*/action.yml` that is not a local `./` path must end in
  `@<40 hex>`. It runs in `fast` already (same command, wider check). Today every `uses:` satisfies it.
- **`ci.yml` and the new scratch route.** `test`, `test-windows`, `test-macos`, `platforms` and `python`
  gain `!endsWith(github.ref, '-nightly')` beside their `-image` and `-scenario` exclusions, so that a
  nightly scratch branch does not also spend a full per-push CI run. `fast` still runs on it (about 2
  minutes, as for `-scenario`). This is the same bounded change 13b made to `test` (D-13b-9).

## 3.7 What blocks and what reports (ARC-48's table, amended)

```text
job group          workflow     trigger                                      merge
stability-*        nightly.yml  schedule (main moved); dispatch; scratch     reports
clients-*          nightly.yml  same                                         reports
parity-long(-*)    nightly.yml  same                                         reports
repeat-*           nightly.yml  same                                         reports
report             nightly.yml  same                                         reports (opens the issue)
```

A red nightly never gates a PR. It **is** a statement about `main`: the issue stays open until a green
night closes it, and the primary session triages it (owner lane, or a recorded ruling). ARC-48's "blocks
main's health" stays with `scenario`/`ac8`; the nightly does not inherit it, because its failures include
flakes and long-horizon findings whose triage takes longer than "stop the line" allows (QC-11 asks
whether `parity-long` should block main's health like `ac8`).

---

# 4. Invariants

The step's I-S13-1 … I-S13-9 apply. I-S13-7 is amended for one job only (§3.6, QC-1). 13b's I-13b-2 …
I-13b-6 apply to the parity legs. 13c adds:

```text
I-13c-1  fast and test keep their names, jobs and commands. The only changes: self-test lines appended to
         fast (each < 1 s: ci_nightly.py, ci_godot.py, ci_repeat.py, ci_stability.py --self-test) and the
         wider check_ci_pins.py rule (same command); test's if: excludes scratch/*-nightly.
         ci_layer.py --list core is byte-identical to main's
I-13c-2  The default parity record is byte-identical to 13b's for the same commit and platform (no profile
         key, record-format 1); ci_parity.py --self-test's existing cases keep their verdicts
I-13c-3  Every nightly verdict is PASS only on positive evidence: a missing verdict file, a missing
         summary line, a cancelled or timed-out job, or a gate error is INCONCLUSIVE or FAIL, and red
I-13c-4  Nothing is retried to green. A repetition samples; every failing sample is reported
I-13c-5  No nightly job is a required check and none runs on pull_request; the report job alone holds
         issues: write; no secret exists
I-13c-6  Planted mutations live only on scratch/13c-* branches, are never merged or cherry-picked, and
         every scratch branch and scratch issue is deleted or closed after its run
I-13c-7  No test, Rust, Cargo.*, worlds/, clients/, presentation/ or Dockerfile file changes (I-S13-1); a
         Godot or scenario defect found by the nightly goes to its owner lane
I-13c-8  A world added under worlds/ enters parity-long, stability and the baseline check with no edit to
         any script; it needs a baselines.txt entry, and its absence is a named FAIL
```

---

# 5. Reuse analysis (`REUSE_POLICY.md`; operator directive: compare several, adopt before building)

## 5.1 Scheduling and the "main moved" gate

| Option | Fit | Verdict |
| --- | --- | --- |
| **`on.schedule` in GitHub Actions + a read-only `gh api` query for the last nightly's commit** | Same service, same runners, same composite actions; `gh` is preinstalled on every hosted runner† | **adopt** |
| `actions/cache` as a "last SHA" marker | Works, but caches are evicted (10 GB LRU†) and a missing marker silently means "run", which is the safe direction but hides why | decline: the runs list is the authority |
| Third-party "skip if unchanged" actions (`fkirc/skip-duplicate-actions`, `pr-mpt/actions-commit-hash-changed`) | Hold a token, add a dependency for ten lines of `gh api` | decline |
| Run every night regardless | Simplest; about 450 job-minutes on idle days for no new information | decline (but `force` exists) |

## 5.2 Reporting a red night

| Option | Fit | Verdict |
| --- | --- | --- |
| **One rolling issue with a label, by `gh` and `GITHUB_TOKEN`** | Visible on the repository, linkable from ledgers, no account or secret; closes itself on green | **adopt** (QC-1) |
| `JasonEtco/create-an-issue`, `actions/github-script` | Same effect through a third-party or JS action holding `issues: write` | decline: more code with the write token than `gh issue` |
| Step summary and GitHub's failure e-mail only | No write permission at all; but a summary is read only by someone who opens the run, and the e-mail goes to the cron's last editor† | **fallback** if QC-1 is "no" |
| Slack/Discord/e-mail webhooks | Need a secret | decline (brief: no secrets) |
| Commit a status file to the repository | Needs `contents: write` on `main`, which is protected | decline |

## 5.3 Godot in CI (DEP-45)

| Option | Fit | Licence | Verdict |
| --- | --- | --- | --- |
| **Official 4.7.2 builds, downloaded by our script, SHA-512 pinned in the repository, cached** | Exactly the binary the operator runs; one version source; works on all three OSes; S23 reuses it | MIT (Godot) | **adopt** (step-14 §5.7 already chose it for logic probes) |
| `chickensoft-games/setup-godot` (action) | Installs Godot on all three OSes, caches; but a third-party action with its own version resolution and .NET options we do not need; pinning by our hash would still be ours | MIT | decline: it would hold the download logic we must pin anyway |
| `barichello/godot-ci` (container) | Linux only; third party's cadence; bundles export templates | MIT | decline (§5.7) |
| Build Godot from source | Hours of CI, a toolchain to own | MIT | decline |
| Xvfb + Mesa lavapipe for the three windowed tests on Linux | Ubuntu archive packages; software Vulkan; no frames are judged, only behaviour | MIT/X11 | **adopt for the windowed tests**, with §3.4.2's fallback |

## 5.4 Flake detection

| Option | Fit | Verdict |
| --- | --- | --- |
| **Repeat the unchanged default suite on the night's commit and compare samples (`ci_repeat.py`)** | Tests exactly what PRs run; no runner change; flakes named per test | **adopt** |
| `cargo-nextest` with `--retries` and flaky reporting | Retries would turn a flaky failure green (I-S13-4); nextest also cannot list the two `harness = false` programs (step-14 §5.3) | decline |
| GitHub's re-run of failed jobs | Hides the first failure; manual | decline |
| A flaky-test service (BuildPulse, Trunk) | Paid or account-based; uploads results | decline (no paid services) |

## 5.5 Server restart loop

| Option | Fit | Verdict |
| --- | --- | --- |
| **A stdlib script driving the real binary, with the Python SDK as the client (`uv run --locked`)** | Same binary and protocol as players; SDK already in the lock and in CI | **adopt** |
| A new Rust integration test | It would be a test edit (I-S13-1) and join the default suite, which this layer exists to keep out of the PR loop | decline |
| The Godot client as the reconnecting client | Heavier, and the protocol checks (`run.sh reconnect`) already cover the Godot module's reconnect | decline |

---

# 6. Modularity and pluggability

| Piece | Seam | Swap without touching |
| --- | --- | --- |
| Scheduling | `nightly.yml`'s `on:` and the gate | the layers, the scripts |
| What a nightly group runs | `ci_layer.py`'s table (`parity-long`, `stability`, `clients`, `clients-probes`, `core-repeat`) | the workflow (I-S13-9) |
| Verdicts | `verdict-*.txt` written by `ci_nightly.py verdict` | the report's channel |
| Notification | `ci_nightly.py report`'s last step (issue) | everything else; the fallback is to drop that step |
| Godot version | one constant block in `ci_godot.py` (version + three SHA-512) | the tests, the launchers |
| Baselines | `scripts/baselines.txt` + `ci_parity.py baseline` | the record, the comparison |
| A new world | `worlds/<w>/world.yaml` + a baselines entry | every script (enumerated) |

Deliberately not pluggable: the PASS/FAIL/INCONCLUSIVE rule (I-13c-3) and "nothing retried to green"
(I-13c-4). Making them configurable would make the nightly negotiable.

---

# 7. Acceptance and adversarial criteria (decided before measuring)

## 7.1 Acceptance

```text
A-N1   A full nightly-shaped run (push of scratch/13c-full-nightly at the final head) completes with every
       job's verdict artifact present, and report's step summary lists the 14 jobs between gate and
       report with a verdict each, the
       main-run line, the benchmark table and the flake list (empty or not)
A-N2   parity-long PASS on that run: four long records, profile long, G-1 … G-5 hold; the step summary
       lists the number of worlds (4 at this base: bodies-yard, lakeside, market-town, social-cafe) and
       keys; each record's summary-300 equals the same commit's ac8 record (from a dispatch of ci.yml on
       the same head, or the main run if the head is main's)
A-N3   baseline check PASS against scripts/baselines.txt recorded at N-C2; the cross-checks of §3.3 are
       recorded (market-town summary-300s vs d5db8988…, lakeside vs 97dac8fc…: equal, or the stale
       ledger value named as a finding)
A-N4   stability PASS on Linux, macOS and Windows: ten kill/restart cycles each with the same instance and
       a revision ≥ the last one observed before the kill, a client seated and resumed after each restart,
       faults 0; replay PASS for every world's 300-day save; CA-13 (perceived.rs) PASS; check_scratch
       left clean
A-N5   clients: on all three OSes, the 25 ignored Godot tests run (25 passed, or skips exactly as QC-5's
       fallback names, each passed on another leg, and the coverage check PASS); on Linux and macOS the
       three slice probes PASS by parsed verdict, the protocol modes exit 0, and ac13_semantic_parity
       PASS on the regenerated evidence
A-N6   repeat: two samples per OS ran to completion; repeat-<os>.txt lists both; the report lists flake
       candidates or "none"
A-N7   gate: on main (after merge, primary session's check) a dispatch with force=false on an unmoved main
       skips every job with the reason and the previous run's id; force=true runs (MN-6 shows the same
       before merge with the gate's self-test and a scratch dispatch is impossible, F-13c-8)
A-N8   I-13c-1: ci_layer.py --list core is identical to main's; --list fast is main's plus the four
       self-test lines; the PR's fast and test pass on its final head
A-N9   I-13c-2: the default profile record of one commit made with the PR's ci_parity.py equals the one
       made with main's, byte for byte (local, laptop)
A-N10  scope: git diff --stat main...HEAD lists only .github/workflows/nightly.yml, .github/workflows/
       ci.yml (the -nightly exclusions only), .github/actions/godot/action.yml, scripts/ci_parity.py,
       scripts/ci_layer.py, scripts/ci_nightly.py, scripts/ci_godot.py, scripts/ci_repeat.py,
       scripts/ci_stability.py, scripts/check_ci_pins.py, scripts/baselines.txt, docs/DECISIONS.md,
       docs/MVP_STATUS.md, .structured-coding/standards.md (prose), this file
A-N11  cost: every job's wall time recorded from the runs; none exceeds its timeout; job-minutes per
       night and macOS/Windows shares recorded beside §8's estimate
A-N12  MN-1 … MN-10 each have their stated outcome
```

## 7.2 Mutations (scratch branches `scratch/13c-<name>[-<group>]-nightly`, never merged; outcomes fixed now)

```text
MN-1  CI    A PLANTED DIGEST CHANGE MAKES THE NIGHTLY RED. A scratch commit changes one recorded fact
            identically on every platform (for example one constant in a System Pack that market-town
            installs). parity-long group only. Expected: compare PASS (every platform agrees); baseline
            check FAIL naming market-town and the keys summary-300, summary-1000, summary-300s,
            facts-300s; report red; the ci_parity.py diff against the baseline commit's record names the
            first differing line. Precondition from the same run: the scratch record differs from the
            base record. Absorbed (no stored byte moved) → INCONCLUSIVE, re-planted once (budgeted)
MN-1b local A planted edit of one hex digit in scripts/baselines.txt → baseline check exit 1 naming that
            world and key; a world removed from the file → "no baseline for world X"
MN-2  CI    THE LONG HORIZON SEES WHAT AC-8 CANNOT. A scratch commit changes a recorded fact only under
            #[cfg(target_os = "windows")] and only after day 300. parity-long group, plus a ci.yml
            dispatch-equivalent check that the 300-day ac8 keys are unaffected (the same scratch record's
            summary-300 equals the base's). Expected: compare FAIL by G-5 on summary-1000 for the
            affected world(s), grouping "Windows/x86_64 ≠ {Darwin/arm64, Linux/x86_64, Linux/arm64}"; the
            300-day keys equal
MN-3  local A TAMPERED SAVE. ci_stability.py's replay check run against a copy of a 300-day save with one
            byte of one facts row flipped → `mineworld replay` refuses it and the verdict is FAIL, naming
            the world; the untampered copy → PASS
MN-4  CI    A 3D PROBE THAT FAILS BUT EXITS 0. A scratch commit makes the drive mode's door check fail
            (e.g. the expected place name altered in slice_probe.gd). clients group. Expected:
            clients-linux and clients-mac FAIL with "1 DRIVE CHECKS FAILED" in the detail, although the
            launcher exits 0
MN-4b local A probe killed before its summary (the parser's self-test with a truncated transcript, and
            once for real with a 5 s limit) → INCONCLUSIVE, exit 3, red
MN-5  CI    A 2D CLIENT DEFECT. A scratch commit breaks one client behaviour that a client_2d test checks.
            Expected: clients-linux, -mac and -windows FAIL naming that test
MN-6  local THE GATE. ci_nightly.py gate's self-test with recorded gh api answers: unmoved main + schedule →
            run=false with the previous run's id; moved → run=true; force=true → run=true; an api error →
            run=true with the error as reason; an unknown group name → exit 1
MN-7  CI    FLAKE DETECTION. A scratch commit adds a test that fails only in the second repetition (it
            reads the repetition number ci_repeat.py exports as MINEWORLD_CI_REPETITION — scratch only;
            no merged test reads a CI variable, I-S13-2). repeat group. Expected: each repeat-* job
            FAIL; the report lists the test as a flake candidate on every OS ("failed 1 of 2 nightly
            samples"), not as a failure
MN-8  CI    FAIL CLOSED. A scratch nightly.yml whose stability-mac job exits 1 before its layer.
            stability group. Expected: stability-mac INCONCLUSIVE ("failed before the layer ran"), report
            red, the run's conclusion failure, nothing reported skipped
MN-9  CI    THE ISSUE'S LIFECYCLE (label nightly-scratch). scratch/13c-mn9-issue-nightly with MN-8's
            failure → an issue labelled nightly-scratch is opened; a second push, still red → a comment on
            the same issue (one issue); a third push, the failure removed → the issue commented and
            closed. Afterwards no open issue carries either label
MN-10 local A PINNED HASH THAT DOES NOT MATCH. ci_godot.py fetch with one pinned SHA-512 altered → exit 1
            "SHA-512 mismatch for Godot_v4.7.2-stable_<os>…", nothing unpacked
```

Not attempted, and why: a planted flake inside the real suite's existing tests (it would edit tests;
MN-7's added scratch test proves the detector); a planted Godot rendering difference (frames are not
judged in CI, §1.2).

---

# 8. Cost (runner minutes)

Estimates scale §2.1's measurements and §2.4's laptop times (runners ≈ 1.5–2.5× the laptop). N-C6 replaces
them with measurements (A-N11).

```text
group        job                  est. min   OS minutes
gate         gate                    0.5     Linux
parity-long  parity-long-linux      25–35    Linux     (image build cached ≈ 2; 4 worlds × (1 000 d mem
             parity-long-arm        25–35    Linux       + 300 d mem + 300 d save + 30 d save + hashing)
             parity-long-mac        30–40    macOS
             parity-long-windows    35–45    Windows
             parity-long (compare)   1–2     Linux
stability    stability-linux        25–35    Linux     (release build; 4 × 300 d save + replay;
             stability-mac          25–35    macOS       10 restart cycles; CA-13 makes its own save)
             stability-windows      35–45    Windows
clients      clients-linux          25–35    Linux     (cold 3D import first night, cached later)
             clients-mac            25–35    macOS
             clients-windows        20–30    Windows   (clients layer only)
repeat       repeat-linux           30–40    Linux     (build + 2 × the suite's test time)
             repeat-mac             35–45    macOS
             repeat-windows         50–65    Windows
report       report                  1       Linux
---------------------------------------------------------------------------
per night                          ≈ 390–530 job-minutes  (Linux ≈ 160–215, macOS ≈ 115–155,
                                                           Windows ≈ 140–185); wall ≈ 60–75 min
per month (≈ 28 nights with a moved main)   ≈ 11 000–15 000 job-minutes
for comparison: ci.yml per completed main push ≈ 139; per code PR head ≈ 110
```

- **Money: $0.** Standard GitHub-hosted runners, Linux, Windows and macOS included, carry no per-minute
  charge on a public repository†. No larger runner, no paid service, no secret.
- **If the repository became private again:** Windows bills at 2× and macOS at 10× Linux†, so a night
  would be about 160 + 2 × 160 + 10 × 135 ≈ 1 800 Linux-equivalent minutes, ≈ 50 000 a month, far beyond
  any included quota† (on the order of $400/month at the published Linux rate†). The nightly would then
  have to shrink to Linux-only and weekly. Recorded as R-9; not a cost this design incurs.
- **Concurrency.** Four macOS jobs per night (`parity-long-mac`, `stability-mac`, `clients-mac`,
  `repeat-mac`), within the Free plan's 5 concurrent macOS jobs†. A `main` push during the night adds
  four more (`test-macos`, `platforms (macos-26)`, `mac`, `python (macos-15)`), so some queue (R-12):
  late, never wrong.
- **13c's own validation budget** (§10): at most 6 full nightly-shaped scratch runs (≈ 3 000 job-minutes)
  and at most 12 single-group scratch runs (≈ 1 500), plus each push's `fast` (2 min). About 4 500
  job-minutes in total, free†.

---

# 9. Commit plan

Each commit tracks implementation, deterministic validation and LLM review as separate items (CLAUDE.md
§3.1). Evidence is recorded in §12's ledger as it happens. Paths are as audited at `98fe3e6`; a moved base
is re-audited at N-C0.

### N-C0 — Design (this document), docs only

- **Goal.** The reviewable design (CLAUDE.md §2.2).
- **Scope.** This file. Non-goals: everything else.
- [x] Implementation: this file, written from the audit of §2.
- [ ] Validation: `python3 scripts/check_doc_headings.py`, `python3 scripts/check_decision_ids.py` pass
  (the docs layer); the PR's `changes` job classifies it documentation-only.
  - Local, 2026-10-10, on `98fe3e6` + this file: doc headings "193 numbered sections across 26
    documents, none duplicated"; decision ids "107 decision ids, all distinct". The PR run is pending.
- [ ] Review: every audited claim cites a file, line, run id or command; every [OM] item is marked; the
  freeze header is absent (DRAFT).
- **Commit boundary.** One file.

### N-C1 — Specs before code: ARC-83, DEP-45, ARC-48 note, standards prose

- **Goal.** The decisions exist before the code that implements them (CLAUDE.md §2.2).
- **Scope.** `docs/DECISIONS.md`: `ARC-83` (layer 4: `nightly.yml`, triggers, the gate, the five groups,
  the verdict rule I-13c-3, the report and the issue, `issues: write` on one job, the baselines policy
  and its update duty, the Windows `clients-probes` reason and its R-c follow-up, the "many agents" gap);
  `DEP-45` (official Godot builds in CI, SHA-512 pinned in the repository, cached; Xvfb and Mesa from the
  Ubuntu archive for windowed tests; the declined options of §5.3; shared with S23 per step-23 §16.4); an
  `ARC-48 note` (the table rows of §3.7; `test`'s `-nightly` exclusion; I-S13-7's one exception).
  `.structured-coding/standards.md`: prose naming the nightly layers (no declaration change).
- [ ] Implementation: the three records and the prose, with the ids the primary session confirms at
  freeze.
- [ ] Validation: `check_decision_ids.py` (ids distinct, count +2), `check_doc_headings.py`.
- [ ] Review: terminology fixed (Layer, System Pack, World Pack as defined); every rule in ARC-83 traceable
  to §3; nothing in ARC-83 stronger than what §3 implements.
- **Commit boundary.** Docs only.

### N-C2 — `ci_parity.py`: the long profile, baselines, `diff`; `scripts/baselines.txt`

- **Goal.** §3.2 and §3.3: the long record, the baseline check, and the initial baselines.
- **Scope.** `scripts/ci_parity.py`: `--profile long` (`LONG_PROFILE_DAYS = 1000`, `SAVE_LONG_DAYS =
  300`), the `profile` source key (written only when not default), G-2's same-profile clause, `--timings
  F`, `baseline check|write`, `diff`; self-test cases added. `scripts/baselines.txt` (new). Unchanged:
  the default profile's keys, format and verdicts (I-13c-2); `compare`'s G-1 … G-5 for default records.
- [ ] Implementation:
  - [ ] the long profile in `record_world`, reusing `summary`, `save_digests` and `remove`;
  - [ ] `profile` in `[source]`, read as `default` when absent; G-2 requires one profile;
  - [ ] `--timings` (JSON, outside the record);
  - [ ] `baseline write` (world set and keys of §3.3 from a long record) and `baseline check` (fail
    closed on world-set mismatch);
  - [ ] `diff A B` (first differing line and chunk per world, platform-agnostic);
  - [ ] self-test cases: profile mismatch; baseline equal; baseline key differs; world missing from the
    baseline; extra world in the baseline; diff locating a line and a chunk;
  - [ ] `scripts/baselines.txt` written from a long record of the implementation head on the laptop.
- [ ] Validation:
  - [ ] `ci_parity.py --self-test` passes, old and new cases;
  - [ ] A-N9: default-profile records of one commit by main's and the PR's script are byte-identical;
  - [ ] two long records of one head on the laptop are byte-identical (determinism precondition, as
    B13-5);
  - [ ] the 1 000-day timings measured (laptop) and recorded; §8 corrected if they differ by > 50 %;
  - [ ] MN-1b; A-N3's cross-checks recorded;
  - [ ] mutation of the comparator: invert the profile clause → the self-test fails naming it; restored.
- [ ] Review: no wall-clock value or host path enters a record; the default path's bytes unchanged; the
  baseline key set excludes snapshots and manifest for the stated reason; `write` and `check` cannot
  disagree about the key set (one constant).
- **Failure cases.** A world that fails to run → `Unmet`, named (as today). A save that cannot be removed
  → `Unmet`. A baseline file that does not parse → FAIL with the line number. Disk exhaustion on a leg →
  the run fails with the OS error (R-2).

### N-C3 — Layers and runners: `parity-long`, `stability`, `core-repeat`; `ci_stability.py`, `ci_repeat.py`

- **Goal.** §3.2's native leg, §3.5, and the stability programs of §0.
- **Scope.** `scripts/ci_layer.py`: layers `parity-long`, `stability`, `core-repeat` (commands only; no
  change to existing layers; `fast` gains the self-test lines of I-13c-1). `scripts/ci_stability.py`
  (new): `restarts` (ten cycles: start `mineworld server worlds/<w> --save D --listen 127.0.0.1:0`, read
  the join line, `GET /status`, join and resume a seat with the SDK, record `instance` and `revision`,
  kill with `Popen.kill`, restart, compare), `replay` (300-day save per world, `mineworld replay`, removal),
  `--self-test`. The Linux layer runs it in the toolchain container with `uv run --locked` (uv is in the
  image, DEP-26); the native layers likewise. `stability` also runs `cargo test -p mineworld-cli --test
  perceived -- --ignored` (CA-13, F-13c-6) and `check_scratch.py left`. `scripts/ci_repeat.py` (new):
  §3.5.
- [ ] Implementation: the three layers; `ci_stability.py` (stdlib + the SDK); `ci_repeat.py` (stdlib);
  their self-tests; the `fast` lines.
- [ ] Validation:
  - [ ] each self-test passes, and each fails when its decisive comparison is inverted (one mutation per
    script, reverted);
  - [ ] local (laptop): `ci_layer.py stability` PASS end to end; MN-3; `ci_layer.py core-repeat` with
    `--times 2` runs two samples and writes the summary;
  - [ ] `ci_layer.py --list core` identical to main's; `--list fast` = main's + four lines (A-N8).
- [ ] Review: the restart loop's oracle is the server's own `/status` and the save (`replay`), never the
  script's own bookkeeping; the client's resume uses the protocol's `resume`, not a reconnect-as-new; the
  kill is a kill (no graceful path is measured as a crash); every repetition runs even after a failure;
  infrastructure failures are not called flakes.
- **Failure cases.** The server prints no join line within its limit → FAIL naming the cycle. `instance`
  changes across a restart → FAIL (AC-6). `revision` goes backwards → FAIL. A client cannot resume →
  FAIL. Replay refuses → FAIL naming the world. A scratch save left → FAIL (`check_scratch`).

### N-C4 — Godot: `.github/actions/godot`, `ci_godot.py`, layers `clients` and `clients-probes`

- **Goal.** §3.4.
- **Scope.** `.github/actions/godot/action.yml` (new: cache, `ci_godot.py fetch`, `GODOT` exported, the
  version check; Windows `_console.exe`; Linux Xvfb and Mesa); `scripts/ci_godot.py` (new: `fetch`,
  `slice` with the verdict parser, `coverage`, `--self-test` over recorded probe transcripts);
  `scripts/ci_layer.py`: `clients`, `clients-probes`.
- [ ] Implementation: the action; the script with the three SHA-512 values read from the official
  release at implementation and recorded in the ledger with the date; the two layers.
- [ ] Validation:
  - [ ] self-test: PASS, FAIL (summary and bare `FAIL`), INCONCLUSIVE (no summary) transcripts → 0, 1, 3;
  - [ ] MN-10; MN-4b (local, laptop with Godot 4.7.2);
  - [ ] local (laptop): `ci_layer.py clients` and `clients-probes` PASS; the coverage check lists 25
    tests;
  - [ ] mutation: the parser made to accept an exit status of 0 as PASS → the INCONCLUSIVE self-test case
    fails; restored.
- [ ] Review: the parser never derives PASS from the exit status; the pinned hashes are the release's
  (cross-checked against `SHA512-SUMS.txt` once, at pinning); no `.godot/` directory is committed; the
  Windows symlink check fails by name.

### N-C5 — The workflow: `nightly.yml`, `ci_nightly.py`, `ci.yml`'s exclusions, the pin rule

- **Goal.** §3.1, §3.6.
- **Scope.** `.github/workflows/nightly.yml` (new); `scripts/ci_nightly.py` (new: `gate`, `verdict`,
  `report`, `--self-test` with recorded `gh api` answers); `.github/workflows/ci.yml` (`-nightly` in five
  jobs' scratch exclusions; nothing else); `scripts/check_ci_pins.py` (the action-SHA rule, QC-7).
- [ ] Implementation: the workflow as §3.1; the script; the exclusions; the pin rule.
- [ ] Validation:
  - [ ] `ci_nightly.py --self-test` (MN-6 and the report's classification table: PASS, FAIL,
    INCONCLUSIVE, missing verdict, skipped by gate, flake vs failure);
  - [ ] `check_ci_pins.py` passes on the tree and fails on a scratch edit replacing one SHA with `@v7`
    (local, reverted);
  - [ ] `actionlint` is not adopted; instead the first scratch push is the syntax check (GitHub rejects an
    invalid workflow with a named error);
  - [ ] `git diff main -- .github/workflows/ci.yml` shows only the `-nightly` additions.
- [ ] Review: every `run:` takes untrusted values only through `env:`; `issues: write` appears on `report`
  only; no `pull_request` trigger; every job but `gate` needs the gate; `parity-long` and `report` run
  `if: always()`; every job's name is undecorated.

### N-C6 — CI evidence: the first full run, the mutations, the measurements

- **Goal.** A-N1 … A-N6, A-N11, MN-1 … MN-9 in CI.
- **Scope.** No file change except the ledger (§12) and bounded fixes of defects found, each recorded.
- [ ] Implementation (runs, each on a `scratch/13c-*-nightly` branch pushed from the PR head or a scratch
  commit on it, deleted after its run): R1 full cold; R2 full warm; MN-1, MN-2 (parity-long); MN-4, MN-5
  (clients); MN-7 (repeat); MN-8, MN-9 ×3 (stability); repairs within the budget.
- [ ] Validation: each run classified PASS / FAIL / INCONCLUSIVE from its artifacts, with run id, head,
  wall time per job; every scratch branch deleted (`git ls-remote --heads origin 'scratch/13c-*'` →
  empty) and every `nightly-scratch` issue closed.
- [ ] Review: each mutation's precondition holds (the planted change reached the record or the test);
  QC-5's fallback, if used, names exactly the windowed tests that failed for a display reason, each with
  the leg where it passed.

### N-C7 — Close

- [ ] Implementation: `docs/MVP_STATUS.md` S13 row (layer 4 live; the nightly's workflow and issue
  label); the ledger; the handoff section; the PR description with the final head's runs.
- [ ] Validation: the PR's `fast`, `test` (and the non-required per-PR jobs) green on the exact final head;
  A-N8, A-N10.
- [ ] Review: every A-N and MN item has evidence or an audited N/A; deviations numbered D-13c-n.
- **After merge (primary session's check, not this PR's):** the first scheduled night on `main` runs, and
  A-N7's dispatch with `force=false` skips.

---

# 10. Test ownership (test rules §26)

```text
STATIC       check_ci_pins.py (now also action SHAs); fmt, clippy, deny — unchanged
UNIT         the self-tests: ci_parity.py (profile, baselines, diff), ci_nightly.py (gate, verdicts,
             report classification), ci_godot.py (verdict parser, coverage), ci_repeat.py (failure
             parsing, flake vs failure), ci_stability.py (oracle comparisons on recorded /status answers)
             — fail-closed classifiers, each shown to bite by one mutation
INTEGRATION  unchanged: the default suite in test, test-windows, test-macos (per PR)
REAL RUN     (Gate 2's role) the nightly itself: the real binary, the real server killed and restarted,
             the real Godot, real 300- and 1 000-day runs, on three OSes
CROSS-PLAT   parity-long: four platforms at 1 000 days and 300 days saved (ac8 stays per push)
GATE 1       NOT REQUIRED: no model anywhere in 13c
CI           the PR's own fast/test on its final head; the scratch nightly runs of N-C6 on the same head
```

Overlap, justified: `repeat` re-runs the default suite, which `test` owns. The claim it owns is different:
"the same commit gives the same verdict twice" (flakes), which one run cannot show.

---

# 11. QB-11: which checks should become required

**Evidence.** F-13c-10: since 13w merged, every completed `main` push run is green on `test-windows` and
`test-macos` (17 runs). Measured wall times: `test` 19–24 min; `test-macos` 18.5–22 min; `test-windows`
32–36 min warm (43 cold). A required check lengthens the merge loop only by the amount it outlasts `test`.

| Option | Merge loop | Coverage | Recommendation |
| --- | --- | --- | --- |
| (A) both required now | +10–20 min per PR head (Windows) | every PR proven on three OSes before merge | no: the Windows wait is paid on every PR, including ones a nightly would catch the next morning |
| **(B) `test-macos` required now; `test-windows` after seven consecutive nightly runs with no Windows flake candidate and no Windows failure, and once its warm wall time is within 5 min of `test`'s** | 0 now (macOS finishes before `test`); later +≤ 5 min | macOS per PR now; Windows per PR non-required (still runs and shows red) and nightly | **recommended** |
| (C) neither | 0 | per-PR runs visible but not enforced; nightly backstop | acceptable if the operator values merge speed over enforcement |

- The criterion "five consecutive green `main` pushes" (QB-11 as written) is met for both jobs; (B) adds
  the nightly's flake evidence for Windows, which is the slower and historically noisier platform, and a
  wall-time condition, which is the operator's stated concern ("test-windows ≈ 43 min on a PR").
- **Shortening `test-windows`** is a separate, measurable follow-up (for example `sccache` with the GHA
  backend, or a Defender exclusion for `target/` on the runner; step-14 §5.4 already lists `sccache` as
  "revisit if cached `test` stays above 15 min"). It is not 13c's scope; it is listed so (B)'s condition
  has a route.
- **Nightly jobs never become required** (I-13c-5): they do not run on PRs. A nightly check that proves
  stable and cheap can later move to PRs as a non-required job behind a fail-closed selector (for example
  `clients` for PRs touching `clients/`), which is QS13-5's selector question, not a settings change.
- `python` ×3 and `platforms` ×2 are outside QB-11; `ac8` stays non-required (QB-8).

---

# 12. Ledger (live; empty until implementation)

Implementation base: `main @ 3c8bbf3` (#158's merge, 2026-10-10). Re-audit (N-C0's rule): `git diff
--stat 98fe3e6..3c8bbf3` touches only `.structured-coding/plans/` (this file, the doorway-names design,
step-13 and step-14 prose), so every audited claim of §2 stands on the implementation base.

| Commit | Implementation | Validation | Review | Evidence |
| --- | --- | --- | --- | --- |
| N-C0 | [x] this file | [x] | [x] | PR #158 run 38089513604: `changes` classified it docs-only, `fast` (docs layer) passed; merged as `3c8bbf3`. Review: done by the primary session at freeze (§14.1) |
| N-C1 | [x] `docs/DECISIONS.md`: `ARC-48 note` (13c), `ARC-83`, `DEP-45`, placed after the 13x note; `.structured-coding/standards.md` one prose paragraph (no declaration change) | [x] | [x] | `check_decision_ids.py`: "109 decision ids, all distinct" (107 → 109); `check_doc_headings.py`: "193 numbered sections across 26 documents, none duplicated". Review: terms as defined (Layer, World Pack, System Pack); each ARC-83 point maps to §3.1–3.7; restart client wording follows `server/PROTOCOL.md` §4.2 (D-13c-1) |
| N-C2 | [x] `ci_parity.py`: `--profile long` (`LONG_PROFILE_DAYS`, `SAVE_LONG_DAYS`; `run_days`, `saved_run`, `record_world`), `profile` in `[source]` only when not default, G-2's profile clause, `--timings`, `baseline check\|write`, `diff`, 15 self-test cases added (29 in all); `scripts/baselines.txt` from the laptop's long record | [x] | [x] | See N-C2 evidence below |
| N-C3 | [x] `ci_layer.py`: layers `parity-long`, `stability`, `core-repeat` (and N-C4's two), `NIGHTLY_LAYERS` result files, four self-test lines in `fast`; `ci_stability.py` (`restarts`, `replay`, `replay-check`, self-test); `ci_repeat.py` (repeat, parse, `classify`, self-test) | [x] | [x] | See N-C3 evidence below |
| N-C4 | [x] `.github/actions/godot/action.yml`; `ci_godot.py` (`fetch`, `slice`, `coverage`, self-test); layers `clients`, `clients-probes` | [x] | [x] | See N-C4 evidence below |
| N-C5 | [x] `.github/workflows/nightly.yml`; `ci_nightly.py` (`gate`, `verdict`, `report`, self-test); `ci.yml`'s five `-nightly` exclusions; `check_ci_pins.py`'s action-SHA rule | [ ] first scratch push (syntax) pending | [x] | See N-C5 evidence below |

Commit mapping: N-C2 is `798ecf2`. N-C3 and N-C4's programs (`ci_stability.py`, `ci_repeat.py`,
`ci_godot.py`, the Godot action) are one commit, `a848d36`, inert until a layer names them; the layers
(`ci_layer.py`), `fast`'s four lines and N-C5 are the next commit, so that no commit names a script it
does not contain.

**N-C2 evidence** (laptop, Darwin arm64, release build of `c65a963`, the scripts as committed in N-C2):

- `ci_parity.py --self-test`: 29 cases, "passed" (the 14 earlier cases keep their verdicts, I-13c-2).
- A-N9 PASS: `record` (default profile) by `main`'s script (`git show origin/main:scripts/ci_parity.py`,
  run from `scripts/`) and by the PR's: both `e8fe59de4897…0c9adf`, `cmp` identical, 1 979 lines.
- Determinism PASS: two long records of one head, `cmp` identical, both `12427f0991…4a4616` (768 395
  bytes). Wall 366 s and 406 s.
- Laptop timings (`--timings`): 1 000 days in memory — bodies-yard 47.5 s, lakeside 29.9 s, market-town
  68.6 s, social-cafe 33.1 s (§2.4 guessed 70–95 s for a town); 300 days saved — 28.0 / 18.6 / 35.0 /
  29.6 s, saves 2.59 / 2.14 / 3.80 / 2.54 GB. §8's per-leg estimate (25–35 min) is replaced by the CI
  measurement in N-C6 (A-N11).
- MN-1b PASS: one hex digit of market-town `summary-300` changed in a copy → exit 1, "world market-town
  key summary-300: baseline …ef, record …ee"; bodies-yard's section removed → exit 1, "no baseline for
  world bodies-yard".
- Comparator mutation: G-2's profile clause weakened (`!= 1` → `> 2`) → "FAIL a default record among long
  ones … FAILED: 1 case(s)"; restored → passed.
- A-N3 cross-checks: F-13c-impl-1.

**N-C3 evidence:** `ci_stability.py --self-test` 13 cases passed; mutation `after.revision < seen` →
`< 0` → "FAIL a revision behind the kill fails"; restored. `ci_repeat.py --self-test` 9 cases passed;
mutation `len(failing) == len(usable)` → `>= 1` → "FAIL failed in one sample, passed in two"; restored.
Local `uv run --locked python scripts/ci_stability.py restarts --binary target/release/mineworld` (laptop,
32 s): ten cycles on market-town, one instance throughout, revision at each start = the last seen before
the kill (5, 9, 15 … 54), perceived cursor 135 → 197, faults 0; the killed save replayed ("54 revision(s)
re-executed from genesis, 205 fact(s) and 1 snapshot(s) reproduced byte for byte"); PASS. Local
`ci_stability.py replay` (laptop): every world 300 days saved and replayed ("… reproduced byte for byte"),
CA-13 `test result: ok. 1 passed` (14.9 s) on market-town's save, "replay PASS". MN-3 PASS: a lakeside
300-day save copied, one byte of fact 200000 flipped with `sqlite3` (`"at":14524206` → `"at":04524206`)
→ `replay-check` exit 1, "FAIL replay lakeside: exited 1: … replay diverged at revision r149075: fact
200000 differs from the logged fact 200000"; the untampered save → PASS. A-N8 static:
`--list` of `core`, `docs`, `parity`, `platforms`, `python`, `python-smoke` identical to `main`'s; `fast`
= `main`'s plus the four self-test lines.

**N-C4 evidence:** pinned SHA-512 values = the release's `SHA512-SUMS.txt` (`gh release download
4.7.2-stable -R godotengine/godot-builds`, 2026-10-10); `fetch` on the laptop downloaded the macOS
archive, its SHA-512 matched the pin (the once-only cross-check), `--version` "4.7.2.stable.official.
ed1daf0bf"; a second `fetch` used the unpacked copy (cache path). MN-10 PASS: Darwin pin's 64th hex digit
altered → exit 1, "SHA-512 mismatch for Godot_v4.7.2-stable_macos.universal.zip: pinned …, downloaded …;
nothing unpacked", the destination empty. Self-test 12 cases passed; mutation "no summary → PASS" →
"FAIL no summary line is INCONCLUSIVE" and "FAIL an empty transcript is INCONCLUSIVE"; restored.
`ci_godot.py coverage` (laptop): "25 ignored Godot tests …; skips none". Local `ci_layer.py clients`
(laptop, `GODOT` = the fetched build): client_2d 8 passed, client_2d_interact 5, client_2d_interact_stub
3, client_settings 9 (the three windowed tests among them); coverage; scratch clean; "layer clients
passed: 4 command(s) in 598.0 s". Local `ci_layer.py clients-probes`: drive, link and target "PASS (exit
0)" by their summary lines (64, 105, 51 s); `run.sh` evidence, affordances, reconnect, perceived, deltas,
admin each exit 0; `ac13_semantic_parity` passed on the regenerated evidence; "layer clients-probes
passed: 12 command(s) in 317.0 s" (the regenerated evidence was then restored with `git checkout`).
MN-4b PASS: `ci_godot.py slice --limit 5 --drive` → "probe drive: INCONCLUSIVE (exit None, 5 s) … killed
at the 5 s limit", exit 3, no Godot process left (the launcher's process group is killed).
Finding during the local run: `ci_repeat.py` wrote the suite's output without flushing, so a log showed
nothing until the run ended; fixed (flush per line) in the next commit.

**N-C5 evidence:** `ci_nightly.py --self-test` 37 cases passed (MN-6's six gate cases among them);
mutation of the gate's comparison (`==` → `!=`) → four MN-6 cases FAIL; restored. `check_ci_pins.py`
passes on the tree; with `ci.yml:42` changed to `actions/checkout@v7` → exit 1 ".github/workflows/
ci.yml:42: actions/checkout@v7 is not pinned to a full commit SHA"; restored. `git diff main --
.github/workflows/ci.yml`: the five `-nightly` clauses and one comment line only. Review: every `run:`
takes branch names, inputs and job outputs through `env:`; `issues: write` appears on `report` only; no
`pull_request` trigger; every job but `gate` needs it; `parity-long` and `report` run `if: always()`;
`compare`/`baseline` use `shell: bash` so `tee` keeps the exit status (pipefail).
| N-C6 | [ ] | [ ] | [ ] | — |
| N-C7 | [ ] | [ ] | [ ] | — |

### 12.1 Deviations (D-13c-n)

- **D-13c-1 — the restart loop's client re-joins; it does not `resume`.** Previous assumption (§3, N-C3):
  "join and resume a seat with the SDK … the client's resume uses the protocol's `resume`". Audit:
  `server/PROTOCOL.md` §4.2 — "After a server restart every hold and every secret is gone (none is
  persisted): a client joins again with its invite"; the SDK's `SeatSession.connect` takes no `resume`.
  Corrected: after each restart the client joins again with the invite and asks for the `perceived`
  stream from the cursor (`through`) it reached before the kill (§5.8), the protocol's own continuation
  across a restart; a `cursor_unavailable` refusal is FAIL. In-process resume within a hold stays
  covered by `run.sh reconnect` (`clients-probes`). Bounded: the oracle (`/status`, `replay`) is
  unchanged. Validation: N-C3's local run.

- **D-13c-2 — `baseline write RECORD [FILE]` rewrites the file in place.** §3.3 showed `baseline write
  <record> > scripts/baselines.txt`; a shell redirect truncates the file before the script can read the
  `reason` lines it must keep. The file argument defaults to `scripts/baselines.txt`. Self-test: "a
  reason line is kept by write and ignored by check".
- **D-13c-3 — one artifact per job, `nightly-<job>`.** §3.1.3 named `verdict-*`, `timings-*` and
  `repeat-*` artifacts. Every nightly job writes into `artifacts/nightly/` (git-ignored by `/artifacts/`)
  and uploads that directory as `nightly-<job>`: verdict, timings, layer results, probe verdicts, repeat
  summaries and, for the parity legs, the long record. The report downloads `nightly-*`, each into its
  own directory. A long record therefore defaults to `artifacts/nightly/parity-long-<os>-<arch>.txt`
  (the default profile's `ac8-<os>-<arch>.txt` is unchanged).
- **D-13c-4 — the nightly layers record how they ended.** For the verdict to tell "the layer never
  started" (INCONCLUSIVE) from "it failed at a command" (FAIL) without reading the job log,
  `ci_layer.py` writes `artifacts/nightly/layer-<layer>.txt` (`started`, then `passed …` or `failed at:
  <command> (exit n)`) for the layers in `NIGHTLY_LAYERS` only. Per-PR layers write nothing; `--list core`
  is unchanged (A-N8).
- **D-13c-5 — a background Xvfb, not `xvfb-run -a`.** The layer runs inside `.github/actions/native`,
  which a wrapper command would have to edit; `.github/actions/godot` starts `Xvfb :99` on Linux and
  exports `DISPLAY` instead. Same display, no edit outside A-N10's list.
- **D-13c-6 — the Godot tests run through `ci_repeat.py --times 1`.** Only so that the verdict names a
  failing test (MN-5): `ci_repeat.py` writes `clients-tests.txt` with the failing names; one run, nothing
  retried.
- **D-13c-7 — `clients-probes` is a plain `ci_layer.py clients-probes` step after the native action's
  `clients` layer**, run when the setup succeeded even if a Godot test failed (`if: success() ||
  steps.clients.outcome == 'failure'`); a second composite would restore and save a second cache.
- **D-13c-8 — CA-13 runs inside `ci_stability.py replay`, on the market-town 300-day save that program
  just made with the release binary** (`MINEWORLD_CA13_SAVE`, the test's own hook), before the save is
  removed. §3.4's layer line `cargo test … --test perceived -- --ignored` would have made a second
  300-day save with the debug binary. S11-C's close ran it the same way (step-12 §17, CA-13: 20 s).
- **D-13c-9 — the Godot cache key is `hashFiles('scripts/ci_godot.py')`**, not "the version and the
  pinned hash": YAML cannot read the pin, and the script holds it. Any script edit re-downloads once;
  `fetch` re-verifies the pin on every run anyway (its marker file).

### 12.2 Findings during implementation

- **F-13c-impl-1 — the market-town ledger digest is a memory run's.** §3.3 expected TW-d's
  `d5db8988…22ee` to equal the long record's `summary-300s` (saved). Local long record at `c65a963`:
  market-town `summary-300` (memory, header kept, 360 lines) = `d5db8988bb9d…a922ee`, `summary-300s` =
  `beb4ec5b…` (359 lines); `facts-300s` rows 375 619 = TW-d's "375 619 facts". Lakeside `summary-300s`
  (349 lines) = Milestone E's `97dac8fc5086…bc6fce3` and `facts-300s` rows 356 689 = its "356689 facts".
  Both ledger values are current; §3.3's pairing for market-town was the design's error, not a stale
  ledger. Every `summary-300` equals the same commit's default (ac8) record (local, all four worlds).

---

# 13. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| **R-1** | Windowed Godot tests cannot run on a hosted Linux or Windows runner (no GPU; software Vulkan too slow for AC-SET-11's 30 fps cap; no interactive desktop on Windows) | Discovery at N-C6 R1. QC-5's fallback: named skips on that leg only, each required to pass on another leg the same night (coverage check). A behaviour failure is never skipped |
| **R-2** | Runner disk: a 300-day save is 2–3.5 GB before SR; macOS runners have about 14 GB† | Saves are made and removed one at a time; disk is printed before and after each world (as `ci_layer.py` does for `core`); SR (#153) reduces it to ≤ 640 MiB. A disk failure is INCONCLUSIVE, named, and a material stop if it persists after SR |
| **R-3** | GitHub disables scheduled workflows on a public repository after 60 days without activity† | Activity is daily. MVP_STATUS's S13 row names the last nightly; a silent stop shows as a stale date |
| **R-4** | Scheduled runs start late or are dropped under load† | Off-hour cron; the gate keys on the commit, so a dropped night only delays coverage by a day |
| **R-5** | Baseline churn: every intended behaviour change must update `scripts/baselines.txt`, or the nightly goes red | QC-3: the PR updates it (no Mac needed); otherwise one red night with the commit range and the first changed line, fixed by a follow-up. Snapshots are excluded to keep storage changes out |
| **R-6** | Infrastructure failures read as product failures (F-13c-7) | INCONCLUSIVE when the layer never ran; per-job rows; never retried to green; a human triages |
| **R-7** | `issues: write` misused through script injection or a compromised action | Only `report` holds it; no PR trigger; values through `env:` only; every action pinned by SHA and checked (QC-7) |
| **R-8** | The nightly duplicates the per-push CI | D-2: only `repeat` re-runs the suite, by design; 13b's jobs are read, not re-run |
| **R-9** | The repository becomes private again | §8: about 50 000 billed-equivalent minutes a month; the nightly must then shrink to Linux-only and weekly. Any sign of billing is a stop |
| **R-10** | A long run exceeds its timeout on Windows | Timeouts set at about 2× the estimate; measured in N-C6; exceeding is a projection to the primary session, not a raised timeout |
| **R-11** | A world fails at 1 000 days (a fault, a panic, an overflow) | That is the nightly working. The finding goes to the world's or the pack's owner lane; the issue names it |
| **R-12** | macOS concurrency (5†): the nightly's four macOS jobs plus a `main` push's four | Queueing only; the cron is off the operator's working hours |
| **R-13** | S23 lands its own Godot download first, with a different shape | QC-10: one owner; 13c then reuses S23's step and adds only `GODOT` and the Xvfb setup |
| **R-14** | The SDK's API moves (S10) and the restart loop's client breaks | The loop uses `connect`/`join`/`resume` only, the protocol's stable surface; a break is a nightly FAIL routed to S10 |

---

# 14. Questions

`[OM]` marks an operator-material question: it changes a permission, a repository setting, or an operator
decision. The rest the primary session can decide.

| ID | Question | Recommendation |
| --- | --- | --- |
| **QC-1 [OM]** | Give the `report` job `issues: write` (the automatic `GITHUB_TOKEN`, no secret) so a red night opens or updates one issue labelled `nightly`, relaxing I-S13-7 for that job only? | **Yes.** It is the only notification that lands where the project already works (issues link from ledgers), needs no secret, and closes itself on green. Fallback if no: the step summary plus GitHub's failure e-mail to the cron's last editor. |
| **QC-2 [OM]** | QB-11: which checks become required? | **(B)** (§11): `test-macos` now; `test-windows` after seven flake-free nightly runs and once its warm time is within 5 min of `test`'s. The settings change is the operator's. |
| **QC-3** | Baselines (§3.3): a committed `scripts/baselines.txt` that a behaviour-changing PR updates, with the nightly catching a forgotten update? | **Yes.** It closes F-13c-3 at the cost of one line per behaviour change, which lanes already do in prose. Alternative (no committed file, compare with the previous night) flags every intended change and is self-referential; declined. |
| **QC-4** | `clients-probes` (bash launchers) on Linux and macOS only until S23 R-c's `mineworld-launch` merges, with the reason in ARC-83? | **Accept.** The 25 Rust-driven Godot tests run on Windows from the first night. |
| **QC-5** | Pre-authorize the named-skip fallback for the three windowed Godot tests on a leg that cannot show a window, with the coverage rule that each passes on another leg the same night? | **Accept.** Without it, a runner limitation would keep a leg red every night, which teaches people to ignore red. |
| **QC-6** | Sequencing with S6 SR (#153): 13c lands with 300-day saves now; the 1 000-day saved run and an ASR-1 assertion (≤ 640 MiB) are added after SR merges? | **Yes.** One layer line each, owned by S13, after SR. 13c does not wait for SR. |
| **QC-7** | Extend `check_ci_pins.py` (run by `fast`) to require full-SHA pins on every `uses:`? | **Yes.** Today's tree passes; it turns I-S13-7's convention into a check before a job with a write permission exists. |
| **QC-8 [OM]** | Authorize scratch branches `scratch/13c-*-nightly` (push to run the nightly before merge, F-13c-8; deleted after each run), the `nightly-scratch` label and issues for MN-9 (closed after), and the budget of §8 (≤ 6 full, ≤ 12 single-group scratch runs)? | **Authorize**, bounded as QS13-14 and QB-9 were. Without it nothing in §7.2 marked CI can be shown before merge. |
| **QC-9** | Cron time 10:17 UTC (03:17 PDT)? | **Accept**, or the operator names another quiet hour. |
| **QC-10** | One Godot download step shared with S23 (`.github/actions/godot`), owned by whichever lands first (step-23 §16.4)? | **Accept.** |
| **QC-11** | Should a red `parity-long` (a platform difference at the long horizon) "block main's health" like `ac8`, or only report? | **Report** for now. Revisit after a month of nights: if it never flakes, promote it to ARC-48's "blocks main's health". |
| **QC-12** | Run CA-13 (`perceived.rs`, ignored, needs a 300-day save) nightly in `stability`? | **Yes.** It is a guard nobody runs since S11-C closed. AO-2's evidence test and `deltas.rs`' measurement stay out (measurements, not guards). |

## 14.1 Rulings (2026-10-10)

Relayed by the coordinator. Each ruling replaces the recommendation above where they differ.

- **QC-1 [OM], operator: yes.** `issues: write` on the `report` job only, with the automatic
  `GITHUB_TOKEN`; it opens and closes one `nightly` issue. I-S13-7 is amended for that job only.
- **QC-2 [OM], operator: accepted.** `test-macos` becomes a required check now; the primary session is
  applying the branch-protection change (a settings change outside this PR). `test-windows` becomes
  required after seven clean nightly runs and once its warm wall time is within 5 min of `test`'s.
- **QC-8, primary: approved.** `scratch/13c-*-nightly` push and delete, `nightly-scratch` issues (closed
  after), and §8's run budget (≤ 6 full, ≤ 12 single-group scratch runs).
- **QC-3, primary: yes**, with a reason line in `scripts/baselines.txt` for each world a PR re-baselines
  (§3.3).
- **QC-4, primary: yes.** `clients-probes` on Linux and macOS until R-c's launcher.
- **QC-5, primary: yes.** Skip by name only when the test passed on another platform that night.
- **QC-6, primary: yes.** 13c lands before S6; the 1 000-day saved run and the 640 MiB (ASR-1) check are
  added after S6 merges.
- **QC-7, primary: yes.** Full-SHA pins required by `check_ci_pins.py`.
- **QC-9, primary:** 10:17 UTC.
- **QC-10, primary: yes.** One Godot download step shared with S23.
- **QC-11, primary:** `parity-long` reports only.
- **QC-12, primary: yes.** CA-13 runs nightly.
- **Decision ids, primary:** `ARC-83` (ARC-82 went to 13c R-PK-1, doorway names) and `DEP-45`.

---

# 15. Execution contract (filled and frozen 2026-10-10, primary session)

```text
PROJECT / PR        MineWorld mvp0 — S13 PR 13c, nightly stability, long parity and client checks
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/pr-13c-nightly.md (the live ledger: §12)
RELATED / BINDING   step-14-ci.md §§3.4–3.7, 4, 5.7, 13, 15, 16; overall.md §3 (S13), §5 "Still open";
                    docs/DECISIONS.md ARC-48 (+ notes), ARC-49, DEP-17, DEP-18, DEP-19, DEP-26, DEP-29,
                    DEP-41; step-23-release.md §16.4; pr-s6-save-retention.md ASR-1, ASR-11;
                    ENGINEERING_STANDARDS.md §§15–16; CLAUDE.md §§2–4
IMPLEMENTATION BASE main at the start of implementation (re-audit §2 if it moved); branch
                    mvp0/pr-13c-nightly; worktree /Users/yuema137/mineworld-worktrees/impl-13c-nightly
                    (sole writer)
PRECONDITION        met: QC-1, QC-2 (operator, 2026-10-10); QC-3 … QC-12 and QC-8 (primary session,
                    2026-10-10); DESIGN FROZEN 2026-10-10 (primary session); ids ARC-83, DEP-45
COMMANDS            cargo, git, gh (PR create/update; run list/view/download/cancel; issue list/view for
                    MN-9's checks; no merge, no settings), python3 scripts/*, uv, docker (local), godot
                    (local, the laptop's 4.7.2)
APPROVED SCOPE      §3; N-C1 … N-C7
FROZEN INVARIANTS   I-S13-1 … I-S13-9 (I-S13-7 amended per QC-1); I-13b-2 … I-13b-6; I-13c-1 … I-13c-8
VALIDATION BUDGET   local: unrestricted; CI: ≤ 6 full and ≤ 12 single-group scratch nightly runs (§8);
                    monetary: none; any sign of billing → stop
ENDPOINT AUTHORITY  implementation, local validation, semantic commits: authorized (D-12)
                    branch push, PR create/update, CI repair: authorized (D-12)
                    scratch/13c-*-nightly push + delete, nightly-scratch issues (closed after):
                      authorized (QC-8, primary session, 2026-10-10)
                    issues: write on the report job: authorized (QC-1, operator, 2026-10-10)
                    repository settings (required checks included), larger runners, paid services:
                      NOT authorized — operator only
                    merge: explicit operator authorization only
MATERIAL STOPS      any change to fast/test beyond I-13c-1; any test, Rust, Cargo, worlds/, clients/,
                    presentation/ or Dockerfile edit in the PR; a nightly finding in product code
                    (recorded with an owner lane — not fixed here); QC-5's fallback needed for a test
                    that fails for a behaviour reason; exceeding the budget; any settings change
NORMAL STOP         PR 13c READY FOR OPERATOR REVIEW — DO NOT MERGE
STOP CONDITION      A-N1 … A-N12 with evidence on the exact final head (A-N7 after merge, primary)
POST-MERGE SYNC     this session: §12, merge identity, evidence; the primary session: step-14 §14,
                    overall.md, MVP_STATUS rows of other lanes
MERGE AUTHORITY     never without explicit operator approval
```
