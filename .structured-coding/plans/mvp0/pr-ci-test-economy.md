# PR design — CI test economy: a full suite that sees every failure, runs once, and is not the only gate

**Lifecycle: DRAFT (2026-10-27). Not frozen. Docs only; no code changes in this PR.**
Requested by the operator through the coordinator on 2026-10-27: "is running the full milestone suite
every round necessary, or wasteful? Shard the work and check for over-testing."

- **Effort:** `mvp0`. **Related:** [`pr-ci-milestone-runs.md`](pr-ci-milestone-runs.md) (ARC-154, `fast`
  only per PR), [`step-14-ci.md`](step-14-ci.md), [`pr-test-hygiene.md`](pr-test-hygiene.md),
  `docs/decisions/ARC-110.md` (the weighted shards), `docs/ENGINEERING_STANDARDS.md` §§15–18,
  `CLAUDE.md` §4 item 8.
- **Out of scope:** the Python job (61 min). Another lane is splitting it
  (`/Users/yuema137/mineworld-worktrees/python-test-split`); this note only leaves room for it.
- **Branch / worktree:** `docs/ci-test-economy`, `/Users/yuema137/mineworld-worktrees/plan-ci-economy`,
  from `origin/main` at `8a3412ab`.

---

## 0. The answer in five lines

1. **The full suite is not the waste. A full suite that stops early is.** Since the ARC-110 sharding,
   each shard runs `cargo test` without `--no-fail-fast`. The first red test binary in a shard stops
   every later binary in that shard. In the last four milestone runs, Linux ran only **23–62 %** of its
   planned test binaries (§1.1). Each round showed a few failures and hid the rest. That alone explains
   much of "three red rounds in a row".
2. **About 40 % of the Linux test time is five 300-day runs.** Two pairs of them run the *same command*
   (same world, seed and horizon) in different test binaries (§1.3). The 300-day horizon belongs to the
   nightly `stability` tier, which already produces and replays every world's 300-day save.
3. **Many tests re-run the same 30-day baseline.** Social Café at seed 7 for 30 days is recomputed by at
   least six test binaries; Market Town's by at least four. Nothing is shared today: no test file uses
   `OnceLock` or a shared fixture (§1.4).
4. **Add a per-PR impact job between `fast` and the full suite.** It runs, on Linux only, the tests that
   the changed paths can affect. Recommended stack: adopt **cargo-nextest** (filtersets with `rdeps()`,
   archives, JUnit), and extend our own `ci_changes.py` with path rules and a recorded
   test → pack map (§2).
5. **Re-run only what failed, and run the full suite once on the final head** (§3). Expected effect:
   a full milestone run drops from about 910 to about 550 runner-minutes, and a red milestone cycle
   from about 2 400 to about 1 200 runner-minutes and from three rounds to one or two (§4).

---

## 1. Over-testing audit

### 1.1 Method and evidence

- **Source.** The `core` shard logs (`gh run view --job … --log`) of the Linux test jobs of four
  milestone dispatches: `40454706809` (2026-10-27), `40403074977` and `40397285400` (2026-10-26),
  `39679913374` (2026-10-21). CI writes no JUnit today, so per-test time is reconstructed from log
  timestamps: a test's time is the span from its binary's `Running` line to its `test … ok` line. Inside
  a binary, tests run in parallel, so this is an upper bound for short tests and exact for a binary's
  longest test.
- **The parser** is a throwaway script, kept out of the repository. P1 (§5) replaces it with nextest's
  JUnit output, which records each test's own time.
- **Operator figures.** `town_bodies` and `social_cafe`-with-bodies never ran on Linux in any of the
  four runs (see 1.2). For them this note uses the operator's figures: `town_bodies` market-town
  854 s and social-cafe 487 s. These are the two 300-day tests in `tools/cli/tests/town_bodies.rs`.

**Finding F-1 — the shards stop at the first red binary.** `scripts/ci_shard.py` `commands()` emits
`cargo test --workspace --test a --test b …` without `--no-fail-fast`. `cargo test` runs binaries one
after another and stops after the first binary that fails. The unsharded layer had the flag
(`scripts/ci_layer.py` line 121: "one red test binary must not hide another's result"); the shard
commands lost it. Measured: "Executable" lines (built) against "Running" lines (run) per Linux job.

| Milestone run | Binaries built | Binaries run | Share run | Shards with exactly one failed binary |
| --- | ---: | ---: | ---: | ---: |
| 40454706809 | 315 | 196 | 62 % | 4 of 6 |
| 40403074977 | 304 | 71 | 23 % | 6 of 6 |
| 40397285400 | 300 | 143 | 48 % | 4 of 6 |
| 39679913374 | 294 | 128 | 44 % | 4 of 6 |

Example: in `40454706809`, shard 3/6 failed after 6 min on `far_field::two_d_and_three_d_water_agree`
(0.01 s). It then skipped 48 binaries, among them `market_composition`, `milestone_e` and
`run_restart`. Every red shard held exactly one failed binary, which is what fail-fast produces, not
what the code's health is. Windows and macOS use the same commands, so the same holds there.

**Consequence.** A milestone round shows the first failure per shard. The fix round then uncovers the
next. This is a serial discovery loop. It costs a full round (~80 min wall, ~800 runner-minutes) per
layer of hidden failures. The fix is one argument; it is the first PR in §5 and does not wait for the
rest of this design.

### 1.2 Where the time goes (Linux, seconds)

Totals per binary (max of the four runs) against the ARC-110 weight table (Windows seconds,
2026-10-21). Linux core holds about 6 700 s of test time in all.

| Binary | Linux s | ARC-110 weight | Tests | Note |
| --- | ---: | ---: | ---: | --- |
| `market_town` | 961–1 059 | 1 168 | 1 | the floor of any shard |
| `town_bodies` | 854 + 487 (operator) | 1 123 | 7 | never ran on Linux in 4 runs (F-1) |
| `run` | 656–687 | 853 | 4 | |
| `market_settlement` | 415–577 | 763 | 3 | |
| `market_composition` | 346–603 | 433 | 1 | |
| `guide` | 299–324 | 402 | 40 | |
| `social_interactions` | 265 | 409 | 3 | |
| `milestone_e` | 177–285 | 327 | 7 | |
| `forgetting_runs` | 181–196 | 250 | 1 | |
| `run_restart` | 125–188 | 292 | 3 | |
| `third_party` | 186 | 222 | 3 | |
| `price_survey` | 154–169 | — (unweighted) | 6 | missing from the weight table |
| `interaction_runs` | 87–131 | 199 | 3 | |
| `social_composition` | 127 | 263 | 5 | |
| lib unit tests (45 binaries) | 1.2 | — | 439 | negligible |

`price_survey` (≈170 s) and `price_reporter` (≈92 s) are not in `WEIGHTS`, so the planner counts them
as 1 s each. The table needs a re-weigh; P1's JUnit timings make that automatic (§5).

### 1.3 The 300-day tier: same command, run twice, on three OSes

Each pair below runs the identical command — same world directory, seed 7, 300 days, with `--save` —
in two different test binaries. The run is deterministic; `run` and `market_town` themselves prove it
byte for byte. So the second run of each pair recomputes a save the suite already has.

| World, seed, horizon | Run by | What each asserts on the save |
| --- | --- | --- |
| `worlds/market-town`, 7, 300 d | `market_town::market_town_lives_three_hundred_days_…` (961 s) | CP-4 liveness per 30-day bucket, money against the newest snapshot, ASRB-2 save growth |
| same | `town_bodies::market_town_lives_with_bodies_for_300_days_…` (854 s) | TD-6/7: shoves stay rare, ≤ 10 % strides stopped, the overlap scan finds nobody inside a wall or another body |
| `worlds/social-cafe`, 7, 300 d | `run::three_hundred_days_with_the_real_systems_…` (656 s; three runs: two saved, one in memory) | the same seed is the same world |
| same | `town_bodies::social_cafe_lives_with_bodies_for_300_days_…` (487 s) | TD-6/7 as above |
| `worlds/lakeside`, 7, 300 d | `milestone_e::lakeside_lives_three_hundred_days_…` (222–285 s) | M-5: every seat lives in every 30-day bucket |

Together these hold about **3 180 s, roughly 45 % of Linux core time**, before the 30-day legs inside
the same tests are counted. On Windows the same work takes about 1.3 times longer.

Why they are in the milestone gate today: **QS-59** (`step-10-market.md` SD-34) put CP-4's full
300-day horizon in the default suite. The recorded reason was cost: "the market run costs ~34 s of a
~3-minute gate". It now costs about 1 000 s — thirty times more — since walking and bodies (12d)
landed. The reason no longer holds at the price it was given.

The nightly already has the right home for this tier. Its `stability` layer builds the release binary
and replays "every world's 300-day save" (`scripts/ci_layer.py` line 159). Cross-platform agreement of
long runs is the nightly `parity-long` group. So running the 300-day horizon in the milestone gate on
three OSes repeats what the nightly proves every night on `main`.

### 1.4 Shared 30-day baselines, computed many times

The common pattern is "run the unconfigured world, run a variant, compare". The unconfigured run is
the same in every binary. Nothing shares it: `git grep OnceLock tools/cli/tests` finds nothing.

| Baseline | Recomputed by (at least) |
| --- | --- |
| Social Café, seed 7, 30 d, unconfigured, `--save` | `social_composition` (the full run every "without X" compares to), `social_interactions` (the unconfigured twin), `interaction_runs` (unconfigured), `forgetting_runs` (unconfigured), `run_restart` (control A), `motivated` (the rule-only part) |
| Market Town, seed 7, 30 d | `market_town` (control C and twin D), `market_composition` (the full composition), `market_settlement` (the rename baseline), `third_party` (the pre-append world) |
| Market Town + price-survey "W2", seed 7, 30 d | `price_survey` runs W2 three times in one binary: lines 250, 305 and 413 of `price_survey.rs` |

A single 30-day run costs roughly 40–100 s on a contended runner (from the binaries above whose only
work is one or two such runs). Removing about ten duplicates saves roughly **600–1 000 s per OS**.
This is an estimate; P3 measures it.

**Compute once, assert many.** One fixture in `tests/support`:
`shared_run(world_dir, seed, days, overlay) -> SavePath`. Its key is a hash of the world directory's
files, the overlay, the arguments and the `mineworld` binary. The first caller runs it; every later
caller in the same machine reads the save. A file lock (`std::fs::File::lock`, stable since Rust 1.89;
the workspace pins 1.97.1) serialises the first run, so no new dependency is needed. Saves are
read-only to callers; a test that mutates a save copies it first. The shard planner places binaries that
share a key in the same shard. Determinism tests that compare *two independent processes* keep their own
second run: sharing that would let them pass without proving anything (ARC-23).

### 1.5 Duplicate coverage in the kill-and-resume tests

"SIGKILL a run, run it again, compare byte for byte with an uninterrupted run" is the `run_restart`
pattern. Seven binaries repeat it, each on its own world:

`run_restart` (3 kill points), `market_town` (1), `market_settlement` (**20** kill points on a 10-day
run, 415–577 s), `bodies_yard_restart` (3), `milestone_e` (1), `price_reporter` (1),
`decision_models` (resume at day 5).

Each world adds systems that write state, so one kill test per world is real coverage. Twenty kill
points in one world at every milestone is not: it is a sampling test, and sampling belongs to the
nightly `repeat` group. Recommendation: five seeded kill points in the gate, the twenty in the nightly.

### 1.6 Unit-only tests

The standards (§§17–18) discourage tests that only check a getter, a constructor, an enum's variants
or a library. A name scan of all 1 748 `#[test]` functions found none of that shape; the names state
behaviour (for example `a_kick_returns_the_seat_to_its_default_with_no_hold`). The 439 lib unit tests
take **1.2 s** in all. There is no CI time to gain here, and no unit test is proposed for removal. A
reviewer may still drop individual trivial tests found later; that is hygiene, not economy.

### 1.7 The audit table

Times are Linux seconds (range over the four runs, or the operator's figure). "Gate" means the
milestone gate; "nightly" means the existing nightly groups on `main`.

| Test | Time | What it proves | Overlap | Recommendation |
| --- | ---: | --- | --- | --- |
| `market_town::…three_hundred_days…` | 961–1 059 | CP-4 300-day market liveness; ASRB-2 save growth; 30-day twin, kill, seed 8 | the 300-day run = `town_bodies` market-town's; the 30-day legs overlap `run_restart`, `market_settlement` | **Split.** 300-day leg → nightly `stability`, computed once and shared with `town_bodies`. 30-day legs stay in the gate. Needs Q-2. |
| `town_bodies::market_town_lives_with_bodies_for_300_days…` | 854 | TD-6/7 on Market Town | same command as above | **Merge** onto the shared 300-day save; **move to nightly**. Keep a 30-day scan in the gate. |
| `run::three_hundred_days…` | 656–687 | 300-day Social Café determinism (2 saves + in memory) | `town_bodies` social-cafe; `run_restart`'s 30-day control | **Move 300-day leg to nightly**; the gate keeps a 30-day twin pair. |
| `town_bodies::social_cafe_lives_with_bodies_for_300_days…` | 487 | TD-6/7 on Social Café | same command as `run`'s | **Merge** onto `run`'s save; **move to nightly**; 30-day scan in the gate. |
| `milestone_e::lakeside_lives_three_hundred_days…` | 222–285 | M-5 Lakeside liveness, determinism | pattern of `market_town` | **Split** as `market_town`. |
| `market_settlement::…killed_anywhere…` | 415–577 | every sale whole across 20 kill points | kill pattern ×7 binaries | **Keep 5 seeded kills in gate; 20 in nightly `repeat`.** |
| `market_settlement::…renames_its_coin…` | 125–167 | currency is data | Market Town baseline | **Keep**; baseline via `shared_run`. |
| `market_composition::each_market_pack_removed…` | 346–603 | AC-2 removability per market pack | Market Town baseline | **Keep**; baseline via `shared_run`. |
| `guide::the_guide_in_this_repository_runs_green` | 299–324 | the human guide works as written (QS25-1: on all three OSes) | none | **Keep** on all OSes. |
| `social_interactions` (3) | 87–265 | IE-2, IE-4…IE-7 sections change the world as configured | Social Café baseline | **Keep**; baseline via `shared_run`. |
| `interaction_runs` (3) | 59–131 | IB-2/3/8 sections | Social Café baseline | **Keep**; baseline via `shared_run`. |
| `social_composition` (5) | 51–127 each | AC-2 for the social packs | Social Café baseline, recomputed per test | **Merge** the baseline (`shared_run`); keep the five assertions. |
| `forgetting_runs` | 181–196 | FA-6 through the binary, resume, replay | Social Café baseline | **Keep**; baseline via `shared_run`. |
| `price_survey` (6) | 98–169 | AC-c-6…9 | W2 run three times in one binary | **Merge** W2 into one run per binary (`OnceLock`); add to `WEIGHTS`. |
| `price_reporter` (5) | 39–92 | AC-d-4, 8–10 | kill at day 5 = kill pattern | **Keep**; add to `WEIGHTS`. |
| `run_restart` (3) | 66–188 | the canonical kill-and-resume proof | the reference others copy | **Keep.** |
| `bodies_yard_restart` | 82–144 | bodies-yard deterministic across SIGKILL | kill pattern | **Keep** (its own world). |
| `third_party::…thirty_days…fishes…` | 186 | EC-7: a third-party System Pack works | Market Town base | **Keep.** |
| `hosted_town` | 120 (fixed wall time) | the hosted town fits its tick budget | none | **Keep** (a timing claim, not a horizon). |
| `activity_length` | 101–105 | AC-FAD-1…3 in 3 worlds × 2 seeds | none | **Keep.** |
| `decision_models`, `motivated` | 81–136 | AC-6 for every installed model | kill/resume pattern | **Keep.** |
| lib unit tests (439) | 1.2 total | algorithms, parsing, invariants | — | **Keep**; no trivial tests found. |

Net effect on Linux core test time: roughly 6 700 s → 3 300–3 700 s (the 300-day tier −2 800 s,
shared baselines −600 to −1 000 s, kills −300 s). The shard floor falls from `market_town`'s 1 059 s to
about 600 s (`market_composition`).

---

## 2. Impact-based selection per PR

### 2.1 The gap it closes

Today a code PR runs `fast` (format, compile, lint, script self-tests) and merges. Tests run only at a
milestone. Three milestone rounds in a row came back red, mostly from regressions and stale
expectations that merged on `fast`. A stale expectation is a test file edited, or a test whose world
changed, that nobody ran. Both are cheap to catch before merge if CI runs *only the tests the change
can affect*.

### 2.2 What the merged PRs actually touch

The 108 PRs merged to `main` since 2026-10-20, classified by changed path (tests and `.md` files
excluded from "source"):

| Class of change | PRs | Share | What the impact job would run |
| --- | ---: | ---: | --- |
| test files and docs only | 28 | 26 % | the changed test binaries |
| clients only (`clients/3d-spike`, `clients/shared`, `mineworld-slice`, `presentation`) | ~30 | 28 % | that client's Godot tests |
| System Packs, cognition crates, worlds | ~15 | 14 % | the tests whose worlds use those packs |
| hub crates (`kernel`, `contracts`, `persistence`, `server`, `tools/cli/src`, `worldpack`, `packages`, `Cargo.lock`) | 18 | 17 % | the gate tier of `core` on Linux |
| scripts, CI, tools only | ~17 | 15 % | the scripts' self-tests (already in `fast`) |

Crate-level reverse dependencies alone are too coarse for MineWorld. Almost every slow test lives in
`tools/cli/tests` and drives the `mineworld` binary, which links every System Pack through
`systems/installed`. So `rdeps(any system crate)` selects nearly the whole CLI suite. The useful unit of
selection for those tests is the **world and its packs**, not the crate.

### 2.3 Options compared (REUSE_POLICY: adopt → adapt → extend → build)

| Option | What it gives | Fit for MineWorld | Verdict |
| --- | --- | --- | --- |
| **cargo-nextest** filtersets (`rdeps()`, `deps()`, `package()`, `binary()`, `test()`), `cargo nextest archive`, `--partition`, JUnit, `--no-fail-fast`, retries | Selects tests by crate graph in one expression; builds once and runs the archive on N shards; per-test JUnit timing; each test in its own process. Apache-2.0/MIT, mature, Linux/macOS/Windows. | Fits the shards and the re-runs (§3). Its crate graph is still coarse for `tools/cli` (above), so it needs the pack map below. Does not take over world state, ownership or CI policy. | **Adopt.** Record a `DEP-` decision. |
| **`ci_changes.py` path rules + `cargo metadata`** (ours, exists) | Already classifies a change set (docs / code), fails closed. Mapping a path to its package is the longest manifest-dir prefix in `cargo metadata`. | Small extension of code we own; it is where the "docs only" rule already lives. | **Extend** — it produces the nextest filterset. |
| **Recorded test → pack map** | The CLI, run under an opt-in environment variable, appends the world and the System Pack ids it resolved to a per-test file. The milestone run uploads the map as an artifact; the impact job reads the latest one. | Data-driven, no hand-kept table, survives a new world or pack (charter: no world-specific hard-coding). Unknown test or missing map → run the test (fail closed). | **Build** (small). Pack-level coverage, the same idea as `cargo-difftests` at a grain that works with subprocesses. |
| **guppy `determinator`** | Changed packages between two commits, including `Cargo.lock` and feature changes, with path rules. | Accurate, but our rule for `Cargo.lock` is "run the gate tier", so its extra precision buys little. A second Rust tool to install on CI. | **Decline for now**; revisit if lockfile-only PRs become frequent. |
| **`cargo-difftests`** | Per-test LLVM coverage; re-runs tests whose covered files changed. | Needs instrumented builds; our tests spend their time in a spawned `mineworld` process, where profile collection per test is fragile; experimental; Windows support unproven. | **Decline.** |
| **Bazel / Buck2 test impact analysis** | Exact, cached, remote-executed test selection. | Replaces Cargo as the build system and forces MineWorld into that framework's model. Far beyond the problem. | **Decline** (REUSE_POLICY: do not force a framework). |
| **GitHub `paths:` filters / `dorny/paths-filter`** | Job-level on/off by glob. | Too coarse for test selection; our `changes` job already does this work. | Use only the existing `changes` job. |

### 2.4 Recommended design

A new job, `impact`, on every non-draft code PR, Linux only, after `changes`. It runs one command:
`cargo nextest run --workspace -E '<filterset>' --no-fail-fast`, sharded 1–3 ways by expected time.

The filterset is the union of these rules, applied to the PR's change set:

1. **Changed test file** → that test binary (`binary(=name)`). Catches stale expectations directly.
2. **Changed crate source** (not a hub) → `rdeps(crate)`, **minus** the CLI integration binaries, **plus**
   the CLI tests whose recorded pack set contains a changed System Pack or cognition crate.
3. **Changed world directory** (`worlds/<w>/…`) → the tests whose recorded map names `<w>`.
4. **Hub path** (`kernel`, `contracts`, `persistence`, `server`, `tools/cli/src`, `worldpack`,
   `packages`, `tests/support`, `Cargo.lock`, `rust-toolchain.toml`) → the whole **gate tier** of `core`
   (core minus the long tier of §1.3), on 3 Linux shards.
5. **Client paths** (`clients/**`, `mineworld-slice/**`, `presentation/**`) → that client's Godot test
   group (`scripts/ci_godot.py` groups). The Python paths go to the Python job being split elsewhere.
6. **CI paths** (`.github/**`, `scripts/ci_*`) → the scripts' self-tests (already in `fast`).
7. **Fail closed:** a path no rule names, a missing or stale pack map, or a change set that cannot be
   computed → rule 4.

Runtime cap: 25 min wall. A selection whose expected time (from the latest JUnit timings) exceeds the
cap is sharded, never trimmed.

**Invariants.**
- **I-1.** Selection never removes a test that rule 1 or rule 7 selects.
- **I-2.** The `impact` job always reports, as the deferred gates do today (ARC-154 I-1): green means
  "the selected tests passed", and its summary lists the selection and why.
- **I-3.** The milestone gate stays the authority. `impact` narrows what reaches it; it does not
  replace it.

---

## 3. Milestone re-runs

### 3.1 Rules

1. **Same head, flaky or infrastructure failure** → `gh run rerun <run-id> --failed`. It re-runs only
   the failed jobs and the gates that depend on them. It tests the same commit, so it can only answer
   "was that a flake?".
2. **New head after a fix** (the usual case) → dispatch the milestone run with a new input
   `rerun_of: <previous run id>`. The run selects:
   - every test that failed or did not run in the previous run, read from its JUnit artifacts, on each
     OS where it failed or did not run; and
   - the impact set (§2.4) of `git diff <previous head>..<new head>`, on Linux.
   With nextest this is one filterset per OS (`test(=a) | test(=b) | …`). "Did not run" matters: before
   F-1 is fixed, a skipped binary is unknown, not green.
3. **A full run is required:**
   - **once on the final head** before a milestone handoff, all OSes, all layers, including the long
     tier on Linux;
   - when a round's selection would exceed half of the full suite's expected time (then the full run is
     cheaper to reason about);
   - after any change to the CI definition itself (`.github/**`, `scripts/ci_*`, the shard plan).

### 3.2 Why not "re-run the whole suite every round"

Each full round costs about 800 runner-minutes and 80 min wall (§4), mostly re-proving tests that
already passed on a head whose diff cannot reach them. With F-1 fixed, round 1 shows every failure.
After that, only the failures and the fix's reach are unknown.

---

## 4. Expected effect

Runner-minutes are summed job durations. The repository is public, so minutes are not billed; the real
constraints are wall time and runner concurrency (macOS is capped at 5 runners and every full run takes
4 of them).

**Measured today.** Run `40454706809` (red): Linux 297, macOS 157, Windows 349 → **803 runner-minutes,
80 min wall**. Run `40397285400` (red): 638. A green full run, with every shard complete, is estimated
at **~910** (Linux core 6 × 25, Windows 8 × 30, macOS 4 × 32, plus Python, far field, scenario, AC-8
legs). Wall time is bounded by `farfield` (79 min) and Python (61 min), not by the core shards.

| Change | Full milestone run | Red milestone cycle | Per merged PR |
| --- | --- | --- | --- |
| Today | ~910 runner-min; 80 min wall | 3 red rounds + final ≈ 2 400–3 200 runner-min; ≥ 4 h of CI wall | 3 min (`fast`) |
| P0: `--no-fail-fast` in shards | +~10 % per red round (the hidden binaries now run) | round 1 shows every failure: rounds 3 → ~2 | — |
| P1: nextest archive (build once per OS) | −~115 runner-min (Linux 5 × 6, Windows 7 × 9, macOS 3 × 8 min of repeated builds) | same per round | — |
| P2: long tier → nightly (Linux on the final head only) | −~2 800 s Linux, −~3 600 s Windows, −~2 800 s macOS ≈ −155 runner-min; Linux 6 → 3 shards | same per round | — |
| P3: shared baselines, 5 kill points | −~600–1 000 s per OS ≈ −45 runner-min | same per round | — |
| **P0–P3 together** | **~550 runner-min**; core shards ~20–25 min wall; overall wall still bounded by far field and Python | round 1 full (~550) + one targeted round (~60–120) + final full (~550) ≈ **1 200** | — |
| P4: `impact` job per PR | — | most regressions caught before merge, so round 1 is more often green: **~550** for a green cycle | avg ~20 Linux runner-min; wall 3–10 min for 70 % of PRs, ≤ 25 min for hub PRs |
| P5: `rerun_of` dispatch | — | a fix round costs ~60–120 instead of ~550–910 | — |

The per-PR average is from §2.2's mix: 26 % × 5 + 28 % × 12 + 14 % × 25 + 17 % × 70 ≈ 20
runner-minutes, Linux only. At the recent rate of ~108 PRs a week that is ~2 200 Linux runner-minutes a
week, with no macOS or Windows runners. One avoided red milestone round saves ~550–800 runner-minutes and
one to two hours of wall time, including 4 of the 5 macOS runners for ~30 min.

**Time-to-green of a milestone** (CI wall only, excluding fix time): today ≥ 4 × 80 min. With P0–P5:
one 40–80 min full round, at most one 15–25 min targeted round, and the final full round on the same
head as the first when it is green.

---

## 5. Proposed PR sequence (each frozen separately)

| PR | Content | Size | Depends on |
| --- | --- | --- | --- |
| **P0** | `--no-fail-fast` in `ci_shard.commands()`; self-test asserts it | 1 line + test | nothing; recommended now |
| **P1** | adopt cargo-nextest in CI: archive per OS, shards from the archive, JUnit artifacts, weights refreshed from JUnit; `DEP-` record | medium | P0 |
| **P2** | long tier: the five 300-day legs split out, computed once per world, run by nightly `stability` and by the final-head full run; ARC note revising QS-59 | medium | Q-2 ruling |
| **P3** | `tests/support::shared_run`, the shared 30-day baselines, `price_survey` W2 once, 5 kill points in the gate | medium | P1 (timings to measure the saving) |
| **P4** | `impact` job: `ci_changes.py` path rules → nextest filterset; recorded test → pack map | medium | P1; Q-1 ruling |
| **P5** | milestone `rerun_of` input reading the previous run's JUnit | small | P1 |

Each PR keeps I-TH-1 of `pr-test-hygiene.md` where it moves tests: **no assertion, expected value,
seed or world changes**; a test only moves tier or reads a shared save.

---

## 6. Operator questions (with recommendations)

- **Q-1. Per-PR `impact` job.** The speed-first rule (ARC-154) runs no tests per PR. Add `impact`
  (Linux only, ≤ 25 min, tests the change can reach), and make the auto-merger wait for it?
  **Recommend: yes, required.** It costs 3–10 min on most PRs and is the only place a stale expectation
  is caught before it reaches a milestone.
- **Q-2. Revise QS-59.** Move the 300-day horizons (`market_town`, `run`, `town_bodies` ×2,
  `milestone_e`) from the milestone gate to the nightly `stability` tier, plus once on Linux on the
  final head of each milestone; the 30-day legs stay in the gate on all OSes.
  **Recommend: yes.** QS-59's reason was "~34 s of a ~3-minute gate"; it now costs ~3 200 s per OS.
- **Q-3. Adopt cargo-nextest** for CI test execution (DEP record). **Recommend: yes.**
- **Q-4. Ship P0 now** as a bounded CI fix, ahead of this design's freeze. It changes no test and no
  policy; it restores the completeness ARC-110 intended. **Recommend: yes.**
- **Q-5. Kill-point sampling.** `market_settlement` runs 5 seeded kill points in the gate and 20 in the
  nightly `repeat` group. **Recommend: yes.**
