# Step 14 — Deployment parity and layered CI (S13)

**Role:** step document for **S13 — Deployment parity and layered CI**, with the first PR (13a) detailed
to the commit in §9 so that it can be frozen and implemented quickly. Later PRs (13b, 13c) are at medium
scope (`CLAUDE.md` §3, "detail one step ahead").
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) §3 (S13), §4 (`AC-8`, layered CI), §7 (the carried
`fetch-depth: 0` requirement).
**Lifecycle:** `DRAFT — awaiting the primary session's review`. Nothing in this document authorizes
implementation. Decision records proposed here carry placeholder ids (`DEP-S13-a`, `ARC-S13-a`, …); the
primary session numbers them at freeze.
**Planning session:** worktree `/Users/yuema137/mineworld-worktrees/plan-s13-ci`, branch `plan/s13-ci`,
from `main @ 0fd0be3`. Written in parallel with the planning of S11, S12, S14 and Milestone E; this
session writes only this file (§12 lists the edits it proposes to parent documents).

**Operator directive carried into this design (2026-10-08, relayed by the primary session).** Keep code
clean, modular and pluggable; search for open-source solutions first; compare several implementations
and do not stop at the first; build our own only if nothing existing serves the goal. §5 is the
comparison for every infrastructure piece S13 introduces; §6 is what can be swapped without touching
anything else.

---

# 1. Requirement and acceptance, quoted

`docs/MVP.md` §9:

> **AC-8** — Local/cloud parity — The same World Pack runs on a laptop and inside a cloud Docker
> container with no semantic differences.

`overall.md` §3, S13:

> - **Output:** Dockerfile and container run; the four CI layers of `ENGINEERING_STANDARDS.md` §16 —
>   fast structural checks, core integration tests, scenario tests, long-running stability.
> - **Depends on:** S11. The repository now exists (`D-12`), so this step keeps the CI workflow files
>   and the container work only.
> - **Acceptance checkpoint:** the same World Pack runs on the laptop and in the container with no
>   semantic difference, demonstrated by identical seeded event sequences (`AC-8`); CI runs the layers
>   on the right triggers.

`docs/NETWORKING.md` §8:

> The same World Pack must run on a laptop and inside a cloud container with **no semantic
> differences**. This is an MVP acceptance criterion, not an aspiration. A behavior that appears only in
> one deployment is a defect in that deployment path.

`docs/ENGINEERING_STANDARDS.md` §15 (every pull request runs, at minimum, format checking,
compilation/type checking, linting, core integration tests, critical regression tests; for Rust
`cargo fmt --check`, `cargo check --workspace --all-targets`,
`cargo clippy --workspace --all-targets --all-features -- -D warnings`; "quality gates must remain
automatic") and §16 (the four layers):

> **Fast structural checks** — run on every change: format, lint, type/compile checks, schema
> validation.
> **Core integration tests** — run on every pull request: create world, start server, load systems,
> create entities, perform actions, observe events, persist, reload, continue simulation.
> **Scenario tests** — run realistic MineWorld scenarios: Social Café, Market Town.
> **Long-running stability tests** — can run separately from the fastest PR loop: simulate 100 days,
> simulate many agents, restart server repeatedly, replay event log.
> The principle does not change: at least some tests must exercise MineWorld as an actual running
> system.

Carried requirements:

- `overall.md` §7 (S9 closeout): **"S13 — CI must check out with `fetch-depth: 0`, or the AC-1 test
  fails (by design, never a skip)."** `tests/acceptance/tests/ac1_composability.rs:359–366` refuses a
  shallow clone by name and cites `fetch-depth: 0`; `precursor_vocabulary.rs:370–376` refuses a missing
  base the same way.
- `step-11-bodies.md` §12 / R-B7 / PB-12 / QP-16: cross-architecture float identity of the `bodies`
  pack (Rapier, `DEP-13`, `enhanced-determinism`) is evidenced only under Rosetta on the operator's
  Mac; **"a native x86_64 host (S13's container on such a host) remains the stronger check."**
- `overall.md` §3, "Cross-cutting": `tools/benchmark` is delivered "with S13". §8 places it.
- `overall.md` §5 `D-1`: the licence (MIT) is revisited "before publication in S13 if the patent-grant
  argument becomes material". QS13-2 touches publication; D-1 is raised there.

## 1.1 What S13 must make observable

```text
O-1  a container image built from a committed Dockerfile runs `mineworld` with a pinned toolchain
O-2  AC-8: the same World Pack, seed and age give identical event sequences on macOS arm64 (the laptop)
     and on Linux x86_64 inside that container — social-cafe, market-town and bodies-yard
O-3  CI runs the four §16 layers, each on its stated trigger, with full history (`fetch-depth: 0`)
O-4  a deliberately broken change turns CI red, at the layer that owns the failure
O-5  what blocks a merge and what only reports is written down, and true of the repository's settings
```

---

# 2. Audit (main @ 0fd0be3, 2026-10-08)

## 2.1 What exists

| Item | Finding | Evidence |
| --- | --- | --- |
| CI | **None.** No `.github/` directory, no workflow file; Actions has never run on this repository. | `ls .github` → no such directory; `gh api repos/yuema137/MineWorld/actions/workflows` → `total_count: 0`; `…/actions/runs` → `0` |
| Container | **None.** No `Dockerfile`, no `.dockerignore`, no compose file anywhere in the tree. | `find . -name 'Dockerfile*'` → nothing |
| Toolchain pin | `rust-toolchain.toml`: channel `1.97.1`, components `rustfmt`, `clippy`, profile `minimal`. Root `Cargo.toml`: `rust-version = "1.97.1"`, edition 2024. | `rust-toolchain.toml:1–8`, `Cargo.toml` `[workspace.package]` |
| Dev profile | `opt-level = 1`, debug assertions and overflow checks stated (`ARC-30`). Tests run in this profile. | `Cargo.toml` `[profile.dev]` |
| Declared checks | `.structured-coding/standards.md`: `cargo fmt --all --check`, `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `python3 scripts/check_decision_ids.py`, `python3 scripts/check_doc_headings.py`, `cargo test --workspace`; `ruff`, `pyright` disabled. "The gates that must actually block are CI's job." | `.structured-coding/standards.md` |
| Python | `scripts/check_decision_ids.py`, `scripts/check_doc_headings.py` — stdlib only. No Python package under `cognition/` (only `cognition/rule-controller`, a Rust crate). | `ls scripts cognition` |
| Workspace | 11 member globs incl. `systems/*`; `spike/server` is a separate Cargo project outside the workspace (`.gitignore`: `spike/server/target/`). | `Cargo.toml` `[workspace]` |
| `harness = false` programs | `persistence/tests/kill_and_resume.rs` (IC-1) and `tests/acceptance/tests/arrival_resolvers_resume.rs` (prints `[resolver-yard] … PASS`). Each re-executes itself as a child selected by an environment variable and SIGKILLs it. They run under `cargo test` as ordinary test targets. | `persistence/Cargo.toml:20–23`, `tests/acceptance/Cargo.toml:11–17` |
| Long runs in the default suite | `tools/cli/tests/run.rs` (three 300-day social-cafe runs in parallel), `market_town.rs` (a 300-day and four 30-day Market Town runs, one SIGKILLed), `bodies_yard.rs`, `bodies_yard_restart.rs`, `kernel/tests/long_run.rs`, `systems/bodies/tests/long_run.rs`. In the default suite by decision (step-10 QS-59: "the full 300 days, in the default suite"). | file headers |
| History-reading tests | `ac1_composability.rs` (`git log`, `git diff --name-only`, `git ls-tree`, `git show rev:path`, `rev-parse --is-shallow-repository`), `precursor_vocabulary.rs` and `seam_vocabulary.rs` (`git cat-file -e`, `merge-base --is-ancestor`, `git diff` text of a recorded range). All fail closed on a shallow clone; none skips. | `ac1_composability.rs:80–366, 469–494`; `precursor_vocabulary.rs:230, 352–394` |
| Tests that read the client tree | `tools/cli/tests/ac13_semantic_parity.rs:42` reads `clients/protocol/evidence`. Nothing reads `clients/3d-spike/` or `presentation/`. | `git grep "clients/\|presentation/" -- '*.rs'` |
| Run summary | `mineworld run <world> --headless --seed N --days D [--save DIR]` prints per-day lines, request/fact counts, activity buckets, `faults`, and `history N facts, fingerprint <FNV-1a 64>` over every fact's stored encoding, then `wall`. The fingerprint is "for reading; not evidence": tests compare stored bytes row by row (step-08 §10.1 Q9). | `tools/cli/src/run.rs:370–508` |
| Deterministic lines | Test helper `deterministic()` keeps every summary line but the `[mineworld] run …` header (it names where the world is kept) and `wall`. `Tables::read` reads `facts`, `journal`, `snapshots` row by row; `assert_same_history` compares them and the manifest's `pack` and `composition`, **not** the whole manifest. | `tools/cli/tests/headless/mod.rs:56–63, 159–244` |
| Save tables | `manifest(id, format, body)`, `journal(revision, at, action_id, entry)`, `facts(event_id, revision, fact)`, `snapshots(revision, snapshot)`. A save's creation records `WorldInstanceId::allocate()`, which mixes a process counter with the wall clock, so **a save file is never byte-identical across runs**; its fact, journal and snapshot rows are. | `persistence/src/sqlite.rs:46–66`; `tools/cli/src/run.rs:209–222`; `server/src/protocol.rs:393–400` |
| Pinned digests already in use | E-RS0 (step-11): sha-256 over every summary line but `wall` of the 300-day seed-7 in-memory runs: social-cafe `ad49c723…c64b` (365 330 facts), market-town `365b50e0…1d1d` (372 755 facts). 12d re-baselines both (step-11 §12); 12c changes bodies-yard's (QO-16). | `step-11-bodies.md:2456–2463, 4130, 4804` |
| Godot checks | `./mineworld-slice --drive` and `--world --link` run Godot `--headless` (physics, no pixels); `clients/protocol/run.sh evidence` regenerates the AC-13/AC-15 evidence headless against a real server. `slice_probe.gd:165` and `shots.gd:119` call `get_tree().quit(0)` unconditionally: **a failed drive still exits 0**. Godot 4.7.2 is installed on the operator's Mac (`/opt/homebrew/bin/godot`). | `mineworld-slice:60–92`; `clients/protocol/run.sh:1–40` |
| Hash crates | No SHA-2 implementation in `Cargo.lock` (only `sha1`, transitively, for the WebSocket handshake). | `grep '^name = "sha' Cargo.lock` |
| Docker on the operator's Mac | Docker Desktop client 29.7.2 (`darwin/arm64`, context `desktop-linux`) installed; **daemon not running** at audit time. No image built (planning only). | `docker version` |
| Spec text on deployment | `ARCHITECTURE.md` §11 (one binary; Local PC, Cloud VM, Docker, community server), §13 (local: native binary / Docker; cloud: "Docker first"); `NETWORKING.md` §7 (single VPS + Docker + persistent volume), §8 (parity); `MVP.md` §8 ("cloud: Docker deployment on a VPS"). | as cited |

## 2.2 The GitHub repository

Read-only `gh api` calls, 2026-10-08:

```text
repository        yuema137/MineWorld — private, owned by a User account, default branch main
size              501 774 KB reported by the API; locally the packed history is 464.8 MiB
                  (git count-objects -vH: size-pack 464.83 MiB, 6 743 objects); HEAD's tree
                  274.7 MB (git archive HEAD | wc -c), of which clients/ 195.0 MB and
                  presentation/ 72.1 MB; everything else under 4 MB. 458 commits on main.
                  Largest blobs: clients/3d-spike/assets/characters/meshy_d/meshy_d.glb 10.1 MB,
                  its base-colour PNG 6.1 MB, vitruvian.glb 4.6 MB, an HDRI 4.6 MB.
LFS               not used (has_lfs null; .gitattributes marks *.bin, *.glb binary only)
Actions           enabled; allowed_actions: all; sha_pinning_required: false
runners           0 self-hosted runners registered
branch protection GET …/branches/main/protection → HTTP 403 "Upgrade to GitHub Pro or make this
                  repository public to enable this feature."
rulesets          GET …/rulesets → the same 403
billing           not readable: the gh token's scopes are gist, read:org, repo; the billing
                  endpoints need `user`. The plan is inferred, not read: the 403s say the account
                  is on GitHub Free.
```

**Finding F-1 (material).** `main` is **not protected**, and on this plan it cannot be. `D-12` records
that "the operator is enabling branch protection on `main` … protection makes that a mechanism rather than
a promise". The API says the mechanism does not exist for a private repository on GitHub Free: neither
classic protection nor rulesets are available. Today "merge requires explicit operator authorization" is
a promise kept by every session, not a mechanism, and **a CI check cannot be made required**. §7 designs
around this honestly and QS13-1 puts the choice to the operator.

**Finding F-2.** The project has merged 61 PRs in about two weeks. Its Actions budget therefore matters
(§10.1).

## 2.3 The test suite's real shape

**E-S13-0** — planning measurement, 2026-10-08, this worktree at `0fd0be3`, `cargo test --workspace`,
dev profile (opt-level 1, `ARC-30`), the operator's Mac (Apple Silicon), cold `target/`. Log:
`/tmp/s13-plan/full-test.log` (not committed).

```text
result        exit 0; 582 tests passed, 0 failed, 0 ignored; 117 test binaries / doctest groups
wall          5 min 28.5 s total (921.6 s user, 106.4 s system, 312 % CPU)
compile       191 crates compiled; the test build finished in 1 min 39 s
tests         194.7 s summed over binaries (cargo test runs binaries one after another)
slowest       run.rs 66.7 s (three 300-day social-cafe runs in parallel)
              market_town.rs 45.8 s (300-day + four 30-day Market Town runs)
              bodies_yard_restart.rs 21.6 s
              run_restart.rs 12.0 s, social_composition.rs 9.4 s, milestone_b.rs 7.5 s
disk          target/ 20 GB after the run:
                target/debug   4.2 GB  (deps 2.0 GB, incremental 2.1 GB, build 43 MB)
                target/tmp    16 GB    (CARGO_TARGET_TMPDIR: 121 scratch saves the tests leave behind;
                                        run-300-a 2.4 GB, run-300-b 2.4 GB, market-300 2.5 GB, five
                                        market ablation saves ≈ 257 MB each)
```

**Finding F-3 (material for CI design).** The suite writes about **16 GB of scratch saves** and its build
is 4.2 GB. A hosted runner's free disk is not guaranteed to hold 20 GB. The first CI run must measure it
(`df -h` before and after the tests, §9 A-C3), and §10 R-2 carries the remedies.

**Finding F-4.** About 70 % of the suite's test time is the long runs in the default suite, which step-10
QS-59 placed there on purpose. S13 does not re-tier them (§3.4, QS13-5).

**Finding F-5.** The headless Godot probes exit 0 on failure (`get_tree().quit(0)`), so CI cannot take a
verdict from their exit code. §3.7 and QS13-9 cover this.

---

# 3. Design

## 3.1 The Dockerfile: one file, three stages

```text
Dockerfile (repository root)

stage toolchain   FROM rust:1.97.1-slim-<debian>@sha256:<digest>   ← pinned by tag AND digest
                  apt: git (the history-reading tests), python3 (scripts/check_*.py, scripts/ci_layer.py),
                       ca-certificates; nothing else
                  rustup component add rustfmt clippy               ← rust-toolchain.toml's components
                  ENV CARGO_TERM_COLOR=always
                  WORKDIR /work                                     ← the checkout is mounted here
                  No source is copied: this stage is the CI environment, and the checkout (with its .git)
                  is bind-mounted into it at run time.

stage build       FROM toolchain
                  COPY the Cargo workspace only (.dockerignore excludes .git, target, clients/,
                       presentation/, spike/, mineworld-3d, mineworld-slice, docs/, .structured-coding/)
                  RUN cargo build --release --locked -p mineworld-cli

stage runtime     FROM debian:<same debian>-slim@sha256:<digest>   ← same glibc as the build stage
                  COPY --from=build /work/target/release/mineworld /usr/local/bin/mineworld
                  COPY worlds/ /opt/mineworld/worlds/
                  a non-root user `mineworld`; VOLUME /var/lib/mineworld; EXPOSE 7878
                  WORKDIR /opt/mineworld
                  ENTRYPOINT ["mineworld"]
                  CMD ["server", "worlds/social-cafe", "--listen", "0.0.0.0:7878",
                       "--save", "/var/lib/mineworld/social-cafe"]
```

- **Toolchain pin.** `rust-toolchain.toml` stays the single source of `1.97.1`. The official image's
  rustup also reads that file inside a mounted checkout, so a mismatch would silently download another
  toolchain. `scripts/check_ci_pins.py` (§9 A-C2) therefore fails the fast layer unless three things
  agree: the Dockerfile's `rust:` tag, `rust-toolchain.toml`'s channel, and the root manifest's
  `rust-version`. It also requires every `FROM` to carry an `@sha256:` digest.
- **Debian release.** It is the release whose `rust:1.97.1-slim-*` tag exists at implementation time;
  `trixie` (Debian 13) is expected. The tag and both digests are read with `docker buildx imagetools
  inspect` at implementation and recorded in the ledger. This is a bounded discovery: if the expected
  variant does not exist, the nearest Debian variant that does exist is taken and the choice recorded.
- **Platform.** CI builds `linux/amd64`. That is the point: AC-8's other side is native x86_64, which
  Rosetta was not (step-11 R-B7). On the operator's Mac the same Dockerfile builds `linux/arm64`. That is
  a useful intermediate data point (other OS, same architecture) but not AC-8's evidence (§3.3).
- **Release profile in the runtime image.** Deployment ships `--release`. ARC-30 showed opt-levels 0 and 1
  give byte-identical output, and step-11 PC-f showed the same across optimization levels for the
  Rapier prototype. Comparing the release image with the laptop's dev build therefore tests two
  dimensions at once. The dev-profile parity test in the toolchain container (§3.3, P-2) isolates the
  platform dimension on its own.

## 3.2 What the container runs

| Use | Image / stage | Command |
| --- | --- | --- |
| Hosting a world (deployment, `NETWORKING.md` §7) | `runtime` | default `CMD`: `mineworld server worlds/social-cafe --listen 0.0.0.0:7878 --save /var/lib/mineworld/social-cafe`; a volume on `/var/lib/mineworld` keeps the world across restarts |
| Any other subcommand | `runtime` | `docker run … mineworld-runtime run worlds/market-town --headless --seed 7 --days 30`, `validate`, `replay`, `inspect`, `biography` — the same binary and arguments as on the laptop |
| CI layers 1–2 (and 13b's P-2) | `toolchain` | `docker run --rm -v "$PWD":/work … mineworld-toolchain python3 scripts/ci_layer.py <layer>` |
| AC-8 in the shipped image (13b) | `runtime` | three `mineworld run` invocations; summary digests compared with the committed references (§3.3, P-3) |

The container has **no container-only code path, flag or `cfg`** (I-S13-5). Authentication (invite token,
nickname) is S11's. When S11 adds server arguments, the runtime `CMD` gains them in S11's PR or in a
follow-up. That is a one-line edit, recorded in §11 as a coordination point, not a dependency.

## 3.3 AC-8: the parity design

**Claim.** For a fixed World Pack, seed and age, the event sequence a run produces on the laptop (macOS,
aarch64-apple-darwin) is identical to the one produced on Linux x86_64 inside the container.

**Why references and not a live two-machine comparison.** A live comparison needs a macOS runner on every
run, which GitHub bills at a 10× minute multiplier on a private repository (§10.1). Instead, a
**committed reference file** is recorded on the laptop. Each platform then checks itself against it:

```text
                          tools/cli/tests/parity/ac8.ref   (recorded on aarch64-apple-darwin, committed)
                                     ▲                 ▲
 laptop: cargo test (default suite)  │                 │  CI: the same test inside the toolchain
 → ac8_parity compares, macOS arm64 ─┘                 └─ container, linux x86_64 (P-2), and the
                                                          release runtime image (P-3)
 equal to the same bytes on both sides  ⇒  equal to each other
```

**The reference file** (`tools/cli/tests/parity/ac8.ref`, plain text, one `key value` per line; read by
the test with the standard library, no format dependency) records:

```text
recorded-on      aarch64-apple-darwin          (the target triple of the recording build)
recorded-at      <commit>                      (the executable head it was recorded on)
world social-cafe   seed 7  days 300  mode memory
  summary-lines  <n>      summary-sha256 <hex>   (sha-256 over the lines `deterministic()` keeps,
                                                  each terminated by "\n": every line but the
                                                  `[mineworld] run` header and `wall`)
world social-cafe   seed 7  days 30   mode save
  facts <rows> sha256 <hex>   journal <rows> sha256 <hex>   snapshots <rows> sha256 <hex>
  facts-chunk 0 <sha256 of rows 0..25 000>  facts-chunk 1 …       (locates a difference, ARC-23)
world market-town   seed 7  days 300 mode memory   … as above
world market-town   seed 7  days 30  mode save     … as above
world bodies-yard   seed 7  days 30  mode memory   … summary
world bodies-yard   seed 7  days 30  mode save     … tables and chunks
```

- **Bytes, not the FNV fingerprint.** Table digests are SHA-256 over each row's key, as 8 little-endian
  bytes, followed by its stored blob, in key order: the same rows `Tables::read` compares. The summary
  digest covers the FNV-1a line only as one line among many. It is the long runs' cheap cover, while the
  30-day saves carry the byte-level claim (I-S13-6, step-08 Q9). The manifest is excluded because it
  carries `WorldInstanceId::allocate()`, which mixes in the wall clock. As in `assert_same_history`, its
  `pack` and `composition` are compared as values instead.
- **Why these worlds and ages.** social-cafe and market-town at 300 days are E-RS0's runs (I-7), in memory
  (≈ 12–15 s each), so they cover the long horizon cheaply. Thirty-day saves give byte-level tables for all
  three worlds without three more 2.4 GB saves (F-3). bodies-yard at 30 days is the only world that
  installs `bodies` (step-11 F-O12), so it is the Rapier float path under test. QS13-6 asks whether the
  ages are right.
- **Locating (ARC-23).** On a mismatch the test names the world, the mode, the table and the first
  differing 25 000-row chunk. It also writes the observed file beside the reference
  (`CARGO_TARGET_TMPDIR/ac8.observed`); CI uploads it as an artifact, and on the laptop the observed file
  is diffed against the committed one.
- **Shown to see.** Before it is trusted, the instrument is shown to detect a difference. Seed 8 must
  differ from the seed-7 reference at a located chunk (a committed negative test). On a scratch branch,
  a deliberate platform dependence must turn CI red while the laptop stays green (P-M2, §8).
- **Self-reference guard.** A reference recorded *in the container* and checked *in the container* would
  prove nothing about AC-8 (test rules §25). The test therefore reports which triples it compared, and
  under `MINEWORLD_AC8_REQUIRE_CROSS=1`, which only CI sets, it fails unless its own target triple differs
  from `recorded-on`.
- **Re-recording.** `MINEWORLD_AC8_RECORD=1 cargo test -p mineworld-cli --test ac8_parity` rewrites the
  file. It refuses to record unless the build's target is `aarch64-apple-darwin`, the laptop the criterion
  names. A behaviour-changing PR re-records in the same commit that changes the digests, exactly where
  step-11's I-7 already requires "digests re-baselined here, and only here". CI then confirms the new
  reference holds on the other side. Forgetting to re-record is a red laptop gate, never a silent pass.
- **Hashing.** SHA-256 comes from the `sha2` crate as a dev-dependency of `mineworld-cli` only (§5.6,
  `DEP-S13-c`). No production crate gains a dependency.
- **The three parity checks:**

```text
P-1  laptop       cargo test (default suite) runs ac8_parity on macOS arm64 → equal to ac8.ref
P-2  CI, PR       the same test in the toolchain container on linux/amd64, with
                  MINEWORLD_AC8_REQUIRE_CROSS=1 → equal to ac8.ref, and a cross-platform comparison
P-3  CI, main     the release runtime image runs the three summary runs; the workflow hashes
     + nightly    `deterministic()`'s lines with `sha256sum` and compares them with ac8.ref's
                  summary-sha256 values (release profile, the shipped binary, the shipped worlds)
```

Optional local evidence, not a gate: the same Dockerfile built for `linux/arm64` on the operator's Mac
(Docker Desktop) runs P-3's commands, so a failure can be localized to OS versus architecture.

## 3.4 The four CI layers and their triggers

One workflow file, `.github/workflows/ci.yml`. Every job runs on `ubuntu-24.04` (x86_64), checks out with
**`fetch-depth: 0`**, builds the `toolchain` stage with the buildx GitHub Actions cache, and runs **one**
command inside it: `python3 scripts/ci_layer.py <layer>`. The commands of a layer live in that script
and nowhere else (§6).

| Layer (§16) | Job | Trigger | Runs | Cost per run (estimate, §10.1) | Merge |
| --- | --- | --- | --- | --- | --- |
| 1 Fast structural | `fast` | every `push` to any branch; every `pull_request` | `cargo fmt --all --check`; `scripts/check_doc_headings.py`; `scripts/check_decision_ids.py`; `scripts/check_ci_pins.py`; `cargo check --workspace --all-targets`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` | 4–8 min cached, 10–14 cold | **blocks** |
| 2 Core integration | `test` | `pull_request` (opened, synchronize, reopened, ready_for_review); `push` to `main` | `cargo test --workspace`: every test, including the two `harness = false` programs, the history-reading scans and (from 13b) `ac8_parity` (P-2) | 15–25 min cached, 25–35 cold | **blocks** |
| 3 Scenario | `scenario` (13b) | `push` to `main`; nightly `schedule`; `workflow_dispatch` | build `runtime`; P-3 for the three worlds; `mineworld validate` of every `worlds/*`; a server smoke test: start the image, a client connects and is seated, stop it, restart on the same volume, resumed at the same revision | 12–18 min | **blocks main's health** (a red `main` stops the line; it is not a per-PR gate) |
| 4 Long-running stability | `stability` (13c) | nightly `schedule` (only if `main` moved since the last run); `workflow_dispatch` | long runs and repeated restarts against the toolchain container (§8, 13c) | 30–60 min | **reports** |
| — Client probes | `clients` (13c) | nightly; `workflow_dispatch` | headless Godot 4.7.2: `./mineworld-slice --drive`, `clients/protocol/run.sh evidence` with `git diff --exit-code clients/protocol/evidence` | 10–15 min | **reports** (F-5, QS13-9) |

**Why layers 2 and 3 share the default suite.** The project's default suite already contains the Social
Café and Market Town scenarios (`run.rs`, `market_town.rs`, `milestone_b.rs`, `milestone_c.rs`, the
composition tests), by decision (step-10 QS-59). Splitting them into a separate job would mean
re-tiering existing tests. `cargo test` cannot exclude test targets without a hand-kept list, and a list
fails open: a new target lands in no layer. So in S13, layer 2 is the whole default suite, and the
`scenario` job adds what the default suite cannot give: the **shipped image** and the **server as a
deployed process**. Re-tiering is a later decision (QS13-5), and a selector would have to fail closed
(test rules §11).

**Concurrency and economy.**
- `concurrency: group: ${{ github.workflow }}-${{ github.ref }}`, `cancel-in-progress: true` for
  non-`main` refs. A new push cancels the stale run.
- `fast` on a branch push and `fast` on that branch's PR are the same work. The PR run is kept, and on
  `push` the job is skipped when an open PR exists for the branch. Mechanism: a `pull_request` trigger
  plus `push` restricted to `main` and to branches matching `wip/**`, `plan/**`, `docs/**`. QS13-2
  decides which.
- Draft PRs run `fast` only; `test` runs on `ready_for_review` and later pushes.

## 3.5 Caching

| What | Mechanism | Key | Size (est.) |
| --- | --- | --- | --- |
| The toolchain image | `docker/build-push-action` with `cache-from/to: type=gha` (buildx GitHub Actions cache backend) | the Dockerfile's content | ≈ 1.0–1.5 GB (rust slim + git + python3) |
| Cargo registry and git deps | `actions/cache` on a host directory mounted as `CARGO_HOME` (`.ci/cargo`) | `Cargo.lock` + `rust-toolchain.toml` + Dockerfile | ≈ 0.3–0.5 GB |
| Compiled dependencies | `actions/cache` on `target/`, after `cargo clean` of the workspace's own packages and with `target/tmp` removed before saving | as above + job name | ≈ 1.5–2.5 GB with the settings below |

- CI sets `CARGO_INCREMENTAL=0`, which removes the 2.1 GB `incremental/` directory, useless in CI. It also
  sets `CARGO_PROFILE_DEV_DEBUG=line-tables-only`, which shrinks `deps/` and keeps line numbers in panics.
  Neither changes codegen semantics: ARC-30 fixes `opt-level`, `debug-assertions` and `overflow-checks`,
  which stay as stated (QS13-7).
- The GitHub Actions cache holds 10 GB per repository by default and evicts least recently used entries.
  The three caches above fit, but `fast` and `test` caches are separate keys (check versus test
  profiles), so the plan is at most two target caches: one per job, restored by `main`'s key on a PR.
- **Checkout cost.** `fetch-depth: 0` with `filter: blob:none` (a partial clone) fetches every commit and
  tree but only the blobs of the checked-out revision. The repository is not shallow
  (`--is-shallow-repository` → `false`), so `ac1_composability` and the scans run. `git diff`/`git show`
  of old revisions fetch their blobs on demand over the checkout's persisted credentials. A
  `sparse-checkout` that omits `clients/3d-spike/` and `presentation/` avoids about 260 MB of blobs per
  job; `clients/protocol/` stays because `ac13_semantic_parity.rs:42` reads its evidence. Both are
  validated in A-C3: `ac1_composability` 13/13 and the scans green on such a checkout. If a scan's
  working-tree diff misbehaves under sparse checkout, sparse checkout is dropped and partial clone kept
  (a bounded fallback, recorded).

## 3.6 `fetch-depth: 0`

Every job that runs `cargo test` (`test`, `stability`) checks out with `fetch-depth: 0`. `fast` runs no
history-reading code, but uses the same checkout step anyway, so a test later moved into it cannot meet a
shallow clone (I-S13-2). The adversarial check: a scratch branch whose workflow uses `fetch-depth: 1` must
turn `test` red, with `ac1_composability` naming "shallow clone" and "fetch-depth: 0" (A13-M3).

## 3.7 What blocks a merge and what only reports

```text
blocks (required)          fast, test              on the exact PR head (merge candidate)
blocks main's health       scenario                a red main stops new merges until it is green or
                                                   the cause is recorded and the operator decides
reports                    stability, clients      a red run is triaged into an issue or a ledger
                                                   finding; it never gates a PR
reports (advisory)         the standards helper    unchanged: it reports and never blocks (CLAUDE.md §3.3)
```

**How "blocks" is enforced (F-1).** On this private GitHub Free repository, a check cannot be marked
required, because protection and rulesets return 403. Until QS13-1 is decided, "blocks" means:

1. Every execution contract's `READY FOR OPERATOR REVIEW` requires green `fast` and `test` on the
   exact final PR head. This replaces "the local full gate runs once on the final head" as the canonical
   full-suite evidence (test rules §9: one canonical full suite, the PR's).
2. The operator merges only with both green. This is the same discipline as today, now with a mechanical
   signal.
3. If QS13-1 makes protection available, a ruleset on `main` requires `fast` and `test` (and "require
   branches to be up to date", because the AC-1 scan reads merge structure). The ruleset is configured by
   the operator, not by a PR: the gh token used by sessions is read-only for administration by policy.

---

# 4. Invariants (proposed; frozen only by the primary session)

```text
I-S13-1  Operational only. No S13 PR edits kernel/, contracts/, systems/, worlds/, server/,
         persistence/, cognition/, clients/, presentation/, authoring/, sdk/, worldpack/, or
         tools/cli/src/. 13b adds a test and a dev-dependency under tools/cli/ only.
I-S13-2  No existing test is edited, re-tiered, #[ignore]d, or made to skip in CI. Every job that runs
         tests checks out with fetch-depth: 0; no test reads a CI variable to relax itself.
I-S13-3  One toolchain: rust-toolchain.toml is the source; the Dockerfile's rust tag and the manifest's
         rust-version agree with it, checked mechanically (check_ci_pins.py); every base image is pinned
         by digest.
I-S13-4  CI never weakens a verdict: no continue-on-error on a blocking job, no automatic retry of a
         failed test, no `|| true` after a check.
I-S13-5  The container runs the laptop's commands: no container-only flag, environment switch, cfg or
         code path in MineWorld.
I-S13-6  Parity is judged on bytes (sha-256 of stored rows and of deterministic summary lines), never on
         the FNV fingerprint alone (step-08 Q9), and never on a reference recorded on the platform
         under test.
I-S13-7  The workflow runs with `permissions: contents: read`; third-party actions are pinned to a full
         commit SHA; no secret is required by any layer.
I-S13-8  No world's facts change: the 300-day seed-7 digests on main at the time of the PR are
         unchanged by it (S13 changes no simulation code, and this says so measurably).
I-S13-9  The layer→command mapping exists once (scripts/ci_layer.py); the workflow names layers, not
         commands.
```

---

# 5. Reuse analysis (`REUSE_POLICY.md` §§1–2, 11–12, 17; operator directive 2026-10-08)

Each piece names at least two real candidates plus "build our own", judged on fit with MineWorld's model,
licence, maturity and maintenance, and cost. Prices and quotas are GitHub's and vendors' published terms
as this session knows them. Billing could not be read (§2.2), so every figure marked † must be confirmed
by the operator on the billing page before money is committed.

## 5.1 The CI service

| Option | Fit | Licence | Maturity / maintenance | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **GitHub Actions**, hosted runners | The repository, PRs and merge flow are already on GitHub (`D-12`); checks appear on the PR the operator reviews; `ubuntu-24.04` is native x86_64 Linux, which is AC-8's other side | proprietary service; workflow YAML is ours | very mature, the default for GitHub-hosted projects | private repo on Free: 2 000 Linux min/month included†, macOS at 10×†; public repo: standard runners free† | **adopt** |
| GitLab CI via a push mirror | Would split review (GitHub) from checks (GitLab); pull mirroring of a private repo is a paid tier† | proprietary SaaS / MIT core | mature | GitLab.com Free compute quota is smaller than GitHub's† | decline: two platforms for one flow |
| Self-hosted Woodpecker or Forgejo Actions | Full control, no minutes, but a server MineWorld has to operate; the reference x86_64 host would be ours to keep | Apache-2.0 / GPL-3.0+ (Forgejo), MIT-compatible use | healthy community projects | a VPS (money) plus operator time | decline for MVP-0: operational burden (`REUSE_POLICY.md` §2, "Operational burden") |
| CircleCI / Buildkite | Good runners; Buildkite needs our own agents | proprietary | mature | free tiers exist, credit-based† | decline: no fit advantage over Actions, and a second account |
| GitHub Actions with a **self-hosted runner** on the operator's Mac | No hosted minutes; but the Mac is arm64 macOS, so it cannot be AC-8's Linux x86_64 side, and a private repo's runner executes PR code on the operator's machine | as Actions | mature | free of hosted minutes; GitHub announced and then postponed a per-minute platform charge for self-hosted runners† | keep as a **fallback** for layer 4 only, if minutes run short (QS13-2) |
| Build our own (scripts + cron) | No PR integration | — | — | operator time | decline |

**Recommendation:** GitHub Actions on hosted `ubuntu-24.04` (`DEP-S13-a`). Keep the workflow thin (§6), so
that moving to another service means rewriting about 100 lines of YAML and nothing else.

## 5.2 How CI gets the toolchain

| Option | Fit | Licence | Maturity | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **(A) Build the Dockerfile's `toolchain` stage in the job; `docker run` each layer with the checkout mounted** | CI *is* the container AC-8 names; one definition of the environment; `git` present on the host for a partial clone | ours (Dockerfile) on the official image | standard Docker usage | +30–60 s per job with the GHA layer cache | **adopt** |
| (B) `jobs.<id>.container:` with the official `rust:1.97.1-slim` image, plus inline `apt-get install git python3` | Simpler caching (`Swatinem/rust-cache` runs inside); but the environment is defined twice (Dockerfile and workflow), and `actions/checkout` silently falls back to a tarball with **no `.git`** if git is missing at checkout time, which breaks AC-1 | MIT (actions) | mature | cheapest | decline: a second source of the environment, and a fail-open checkout path |
| (C) Toolchain on the bare runner (`dtolnay/rust-toolchain@1.97.1` or the preinstalled rustup) | Fast; but tests then run outside the container, so AC-8 needs a separate container job anyway | MIT | very mature | cheapest | decline for layers 2–3; acceptable for nothing S13 needs |
| (D) Push a prebuilt toolchain image to GHCR and use `container:` | Clean jobs; but a 1–1.5 GB private image exceeds the Free plan's 500 MB package storage† | — | mature | storage money† | decline while private |

## 5.3 The test runner

| Option | Fit | Licence | Maturity | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **plain `cargo test --workspace`** | Runs everything the project already treats as the gate, including the two `harness = false` programs, which re-execute themselves and SIGKILL a child; the canonical local command, so CI and laptop agree by construction | part of Cargo | — | none | **keep** |
| `cargo-nextest` | Process per test, JUnit output, partitioning and retries. But it lists custom-harness targets by invoking them with libtest's `--list --format terse`, which `kill_and_resume` and `arrival_resolvers_resume` do not implement (to be confirmed by running it in 13a, A-C3 review item). Retries would hide nondeterminism, which this project treats as a defect. Its parallelism gains are small: 70 % of time is six long tests | Apache-2.0 / MIT | mature, actively maintained | a binary to install and pin | **decline for now**; revisit if the suite passes ~15 min on CI or needs sharding across runners. Record the reason in `DEP-S13-a` |
| `cargo test` + `cargo2junit` or `--format json` (unstable) for reports | Test reports in the PR UI | MIT | small tools; `--format json` needs nightly | — | decline: the job log plus a step summary of slow binaries suffices |
| Build our own selector | Test rules §11 want selective PR CI that fails closed; there is no dependency map yet | — | — | — | defer: no selector until the full suite is a real tax (QS13-5) |

## 5.4 Caching

| Option | Fit | Licence | Maturity | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| `Swatinem/rust-cache` | The usual answer. It keys on the host `rustc -vV` and runs where the job's steps run, so under option (A) it would key on the runner's toolchain rather than the container's. It also prunes workspace artifacts and sets `CARGO_INCREMENTAL=0` | MIT | very mature | free (Actions cache) | **decline under (A)** for the key mismatch; it would be the choice under (B) |
| **`actions/cache` with an explicit key** on the mounted `CARGO_HOME` and `target/`, workspace packages cleaned and `target/tmp` removed before save | Keys on exactly what determines the build (Dockerfile, toolchain file, `Cargo.lock`); works with (A) | MIT | first-party, very mature | free within 10 GB | **adopt** |
| `sccache` with the GHA backend (`mozilla-actions/sccache-action`) | Caches per compilation unit; good when many small changes rebuild a few crates; adds a `RUSTC_WRAPPER` and a server; does not cache linking, and the test build is link-heavy (117 binaries) | Apache-2.0 | mature | free within the cache limit; slower cold starts | decline for now; revisit if cached `test` stays above 15 min |
| BuildKit cache mounts + `cargo-chef` (for the `build` stage) | Correct for the release image's dependency layer; `cargo-chef` (MIT/Apache-2.0) is the standard recipe | MIT / Apache-2.0 | mature | free | **adopt `cargo-chef` in 13b** if the `scenario` job's cold release build exceeds 8 min; otherwise plain layering |
| None | Simplest; every run is cold (≈ +10 min per job, measured in A-C3) | — | — | minutes | decline: a cold run roughly doubles minutes (§10.1) |

## 5.5 Base images

| Option | Fit | Licence | Maturity | Size / cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| Build: **`rust:1.97.1-slim-<debian>`** (Docker Official Image) | Exact toolchain version as a tag, rustup inside so `rust-toolchain.toml` components install, glibc, gcc for rusqlite's bundled C | Rust MIT/Apache-2.0; image packaging MIT | official, rebuilt per Rust release | ≈ 0.8–1 GB | **adopt** |
| Build: `rust:1.97.1-<debian>` (full) | Same, plus many unused packages | as above | official | ≈ 1.5 GB | decline: size |
| Build: musl static (`rust:alpine`, `cargo-zigbuild`) to a `scratch` runtime | A different target (`*-linux-musl`) from anything else in the project; musl's allocator is slow on allocation-heavy runs; a libm difference could not be ruled out without measuring Rapier on it | MIT / Apache-2.0 | mature | tiny runtime | decline: it would test a third target, not the deployed one |
| Runtime: **`debian:<same>-slim`** | Same glibc as the build stage; has a shell, so an operator can `docker exec` and run `mineworld inspect` against the volume | DFSG, free | official, security-maintained | ≈ 75 MB | **adopt for MVP-0** |
| Runtime: `gcr.io/distroless/cc-debian<n>` | glibc + libgcc, no shell, no package manager: a smaller attack surface; needs the distroless Debian generation matching the build stage's glibc | Apache-2.0 (tooling) | Google-maintained | ≈ 25–35 MB | **swap later** (a one-line `FROM`, §6) when a hosted deployment exists; QS13-10 |
| Runtime: `cgr.dev/chainguard/glibc-dynamic` (Wolfi) | Minimal, low-CVE; the free tier offers only `latest`, so a pinned digest may be garbage-collected, which conflicts with reproducible builds | Apache-2.0 | commercial vendor, well maintained | small; version tags are paid† | decline: reproducibility |
| Build our own base | — | — | — | — | decline |

## 5.6 Hashing for the parity test (13b)

| Option | Fit | Licence | Maturity | Verdict |
| --- | --- | --- | --- | --- |
| **`sha2` (RustCrypto)**, dev-dependency of `mineworld-cli` only | Pure Rust, no C, identical digests to `sha256sum`/`shasum -a 256`, so P-3's shell-side check and the Rust test agree, as do E-RS0's recorded values | MIT / Apache-2.0 | the de facto Rust SHA-2, widely depended on | **adopt** (`DEP-S13-c`) |
| `ring` | Has SHA-256, but brings C and assembly, which is more build surface for a test | Apache-2.0-style / ISC | mature | decline |
| `blake3` | Faster; but a different digest from every existing record and from the coreutils P-3 uses | CC0 / Apache-2.0 | mature | decline |
| Shell out to `shasum` / `sha256sum` from the test | Two different commands on macOS and Linux, absent on some systems: a platform difference inside a platform-parity test | — | — | decline |
| Compare FNV-64 (already printed) | Explicitly "not evidence" (step-08 Q9) | — | — | decline |
| Build our own SHA-256 | — | — | — | decline |

## 5.7 Godot in CI (13c)

| Option | Fit | Verdict |
| --- | --- | --- |
| Official Godot 4.7.2 Linux x86_64 binary run with `--headless`, downloaded once, checksum pinned, cached | The probes need physics, not pixels (`mineworld-slice:83–86`); matches `D-11`'s split: `--headless` for logic, windowed for pixels | **adopt** for logic probes |
| `barichello/godot-ci` and similar community images | Convenient, but pinned to a third party's update cadence and export templates we do not need | decline |
| Rendering in CI (Xvfb + Mesa llvmpipe/lavapipe) | Would produce frames, but software Vulkan differs from the Metal frames `D-11` and `ARC-17` accept; visual acceptance stays with the operator | decline in S13 |

---

# 6. Modularity and pluggability

Each piece below can be replaced without touching the others. The seams are named so that a reviewer
can check the claim.

| Piece | Seam | Swap without touching | Cost of the swap |
| --- | --- | --- | --- |
| CI service | `.github/workflows/ci.yml` does only: checkout, cache, build the toolchain stage, `docker run … python3 scripts/ci_layer.py <layer>`. No check command appears in YAML (I-S13-9) | the layers, the Dockerfile, the tests | rewrite one YAML file (~100 lines) for another service |
| A layer's contents | `scripts/ci_layer.py`: a table `layer → [commands]`, stdlib Python like `scripts/check_*.py`; the same entry point runs locally: `python3 scripts/ci_layer.py fast` | the workflow, the Dockerfile | edit one list; reviewed as policy |
| The test runner | `ci_layer.py`'s `core` entry is `cargo test --workspace` | everything else | one line to try `cargo nextest run` |
| Cache backend | the workflow's cache steps only | the image, the layers | replace `actions/cache` by `rust-cache` or `sccache` steps |
| Toolchain version | `rust-toolchain.toml`; the Dockerfile tag and `rust-version` follow, enforced by `check_ci_pins.py` | the workflow | a deliberate three-line change, checked |
| Runtime base image | the `runtime` stage's `FROM` | the build stage, CI layers 1–2 | one line (+ digest); P-3 re-proves parity |
| Deployment target | the `runtime` image's interface: `mineworld` + arguments, a volume at `/var/lib/mineworld`, port 7878 | the binary, the worlds | a compose file, a VPS unit or a Kubernetes manifest is a consumer of the image, not a change to it |
| Parity worlds | `ac8.ref`'s `world` records | the test's code | add a record and re-record |

What is deliberately **not** pluggable: the parity instrument's equality, which is bytes and SHA-256, and
the rule that a reference is recorded on the laptop (I-S13-6). Making those configurable would make AC-8
negotiable.

---

# 7. The PR split

Three PRs. The split follows integration checkpoints, not file counts: 13a makes CI real; 13b makes AC-8
true; 13c adds what runs at night. Each PR is independently reviewable and leaves `main` working.

## 7.1 PR 13a — the container and the per-PR CI (layers 1–2)

Full design in §9.

- **Output:** `Dockerfile` (toolchain, build, runtime), `.dockerignore`, `.github/workflows/ci.yml`
  (`fast`, `test`), `scripts/ci_layer.py`, `scripts/check_ci_pins.py`; `DEP-S13-a`, `DEP-S13-b`,
  `ARC-S13-a` in `docs/DECISIONS.md`; pointers in `.structured-coding/standards.md`, `README.md`,
  `docs/MVP_STATUS.md`.
- **Integration checkpoint:** on the PR itself, `fast` and `test` are green on the exact head, with
  `ac1_composability` 13/13 and both vocabulary scans green on a CI checkout. The runtime image is built
  locally (or by a one-off `workflow_dispatch` of the image build) and runs `mineworld validate` and a
  1-day `run` for each world.
- **Adversarial (decided before measuring):** four scratch branches, never merged. A formatting
  violation turns `fast` red. An assertion inverted in an existing test turns `test` red and names the
  test. `fetch-depth: 1` turns `test` red with "shallow clone" and "fetch-depth: 0" in the log. A
  Dockerfile tag `1.97.0` turns `fast` red through `check_ci_pins.py`.

## 7.2 PR 13b — AC-8 parity (layer 3)

- **Output:** `tools/cli/tests/ac8_parity.rs`; `tools/cli/tests/parity/ac8.ref` recorded on the laptop;
  `sha2` as a dev-dependency of `mineworld-cli` (`DEP-S13-c`); the `scenario` job (P-3, `validate` of
  every world, the server-in-container smoke with a volume restart); `ARC-S13-b` (how AC-8 is measured,
  §3.3); `MVP_STATUS` AC-8 row; `cargo-chef` in the `build` stage if §5.4's threshold is crossed.
- **Integration checkpoint:** P-1 green on the laptop, P-2 green in `test` on the PR (`linux/amd64`,
  cross-platform asserted), P-3 green from a `workflow_dispatch` of `scenario` on the PR branch. AC-8 is
  then demonstrated for social-cafe, market-town and bodies-yard.
- **Adversarial (decided before measuring):**
  - **P-M1:** seed 8 differs from seed 7's reference, and the test names a located chunk (committed
    negative test).
  - **P-M2:** a scratch commit makes the test's facts hashing feed one extra byte for one row only
    under `cfg(target_arch = "x86_64")`. The test, not MineWorld, is mutated, because I-S13-1 forbids
    MineWorld edits. The laptop stays green and CI turns red, naming world, table and the chunk holding
    that row. This proves that the CI side really compares, and that it locates.
  - **P-M3:** a reference recorded inside the container fails under `MINEWORLD_AC8_REQUIRE_CROSS=1`.
  - **P-M4:** re-recording is refused on a non-`aarch64-apple-darwin` build.
- **Depends on:** 13a. **Sequencing:** after S15's 12d if possible (12c changes bodies-yard's facts; 12d
  re-baselines both towns), otherwise whichever of 13b and 12c/12d lands second re-records (QS13-12).

## 7.3 PR 13c — long-running stability, client probes, benchmark timings (layer 4)

- **Output:** the `stability` job:
  - social-cafe 1 000 days with `--save`, then `mineworld replay` (byte-for-byte reconstruction);
  - market-town 300 days SIGKILLed at five points and resumed, compared with an uninterrupted run;
  - bodies-yard 300 days;
  - `mineworld server` killed and restarted ten times on one volume while a client reconnects.

  The `clients` job runs Godot headless probes, report-only. `tools/benchmark` is delivered as this
  job's timing report (wall time per world-day, recorded as an artifact and a step summary, with a
  nightly trend), not as a new crate (QS13-13).
- **Integration checkpoint:** one scheduled night and one `workflow_dispatch` run complete, with
  artifacts. Every check's verdict is read from content, not from an exit code.
- **Adversarial (decided before measuring):**
  - A replay against a save whose one fact row is altered (on a scratch copy) is refused by `mineworld
    replay`, and the job turns red.
  - A probe run made to fail, for example a missing scene, is reported as FAIL even though Godot exits 0.
    The job parses the probe's verdict line; if the probe prints none, the run is INCONCLUSIVE, not PASS.
- **Depends on:** 13a; independent of 13b. The client probes depend on S12/S14 giving the probes a
  failing exit status or a verdict line (QS13-9). Until then the job reports.

---

# 8. Test ownership across S13

```text
STATIC      check_ci_pins.py (toolchain and digest pins), check_doc_headings, check_decision_ids,
            fmt, clippy — layer 1
UNIT        none new: S13 adds no simulation logic (I-S13-1)
INTEGRATION the whole default suite, unchanged, on Linux x86_64 in the container — layer 2
REAL RUN    (Gate 2's role) the runtime image hosting a world on a volume and resuming it; P-3; the
            nightly long runs — layers 3–4
CROSS-PLAT  ac8_parity against the laptop-recorded reference — P-1 (laptop) and P-2 (CI)
GATE 1      NOT REQUIRED — no model anywhere in S13
CI          S13 is the CI; each PR's canonical evidence is its own workflow run on its exact head
```

---

# 9. PR 13a — the container and the per-PR CI (full design)

## 9.1 Identity, base, approved scope

```text
PR            13a — the container and the per-PR CI (S13, first of three)
base          main @ 0fd0be3, or the main the primary session names at freeze (re-audit if it moved)
branch        mvp0/pr-13a-ci (proposed), in a worktree held by the implementing session only
audit         §2 (files, API answers and measurements on 0fd0be3)
scope         §3.1, §3.2 (runtime image built, not deployed), §3.4 layers 1–2, §3.5, §3.6, §3.7;
              DEP-S13-a, DEP-S13-b, ARC-S13-a
non-goals     AC-8's parity test and references (13b); the scenario, stability and clients jobs
              (13b, 13c); configuring branch protection or rulesets (operator, QS13-1); any change
              to MineWorld code or tests (I-S13-1, I-S13-2); publishing an image to a registry
```

**Acceptance (all observable, decided before measuring):**

```text
A13-1  `fast` and `test` run on the PR and are green on its exact final head; `test`'s log shows
       582 or more tests passed, 0 failed, and `ac1_composability` 13/13, `precursor_vocabulary` and
       `seam_vocabulary` passing; `kill_and_resume` and `arrival_resolvers_resume` print PASS
A13-2  the CI checkout is not shallow (the job prints `git rev-parse --is-shallow-repository` → false)
       and is a partial clone (the job prints `git config remote.origin.partialclonefilter` → blob:none)
A13-3  the `test` job's log records `df -h` before and after the tests, the peak `target/` size and
       the job's wall time; the run does not fail for disk (F-3); if it does, R-2's remedy is applied
       and recorded, and A13-3 is re-measured
A13-4  the toolchain inside the container reports rustc 1.97.1, and rustfmt and clippy are present
       (printed by `ci_layer.py` before the first command)
A13-5  `docker build --target runtime` succeeds on linux/amd64 (a `workflow_dispatch` of an image job
       in 13a's PR, or locally), and the image runs `mineworld validate worlds/<w>` for social-cafe,
       market-town and bodies-yard, and `mineworld run worlds/social-cafe --headless --seed 7 --days 1`
       with `faults 0`; the image's default command starts a server that listens on 7878 and keeps
       its world on the volume (start, stop, start again: the second start reports resuming)
A13-6  a second `test` run with no change restores the caches; its wall time is recorded beside the
       cold run's, which gives §10.1 its measured figures
A13-7  no file under the I-S13-1 paths changes; no existing test file changes (git diff --stat)
A13-8  I-S13-8: the 300-day seed-7 social-cafe and market-town summary digests on the PR head equal
       main's (measured once locally, on the final head)
```

**Mutations (each on a scratch branch, never merged; each must turn CI red at the named job):**

```text
A13-M1  `cargo fmt` violation in one file             → `fast` red at `cargo fmt --all --check`
A13-M2  one assertion inverted in an existing test    → `test` red, naming that test; `fast` green
A13-M3  the workflow's checkout at fetch-depth: 1     → `test` red; ac1_composability's message
                                                        contains "shallow clone" and "fetch-depth: 0"
A13-M4  the Dockerfile's rust tag set to 1.97.0       → `fast` red at check_ci_pins.py, naming both
                                                        values
A13-M5  a `FROM` without `@sha256:`                    → `fast` red at check_ci_pins.py
```

## A-C0 — Design (this document) — docs only

- [x] Implementation: §§1–12 of this file, from the audit in §2 (draft; not frozen).
- [x] Validation: `python3 scripts/check_doc_headings.py` → 176 numbered sections across 25 documents,
  none duplicated; `python3 scripts/check_decision_ids.py` → 51 decision ids, all distinct (2026-10-08,
  on `plan/s13-ci`). E-S13-0 (§2.3) is the planning measurement; no other test was run, since this commit
  is documentation only.
- [x] Self-review by the drafting session: every §2 claim cites a file and line, a command, or an API
  answer; figures that could not be read are marked † and routed to QS13-1/-2. Operator-material
  questions are marked in §11.
- [ ] Review: by the primary session.

## A-C1 — Specs before code: DEP-S13-a, DEP-S13-b, ARC-S13-a

**Goal.** The CI service, the images and the layer policy exist as reviewable records before any workflow
relies on them (`CLAUDE.md` §2.2, `REUSE_POLICY.md` §11).

**Scope.**
- `docs/DECISIONS.md`:
  - **DEP-S13-a** — *CI runs on GitHub Actions hosted Linux runners, inside the repository's own
    toolchain container.* Covers §5.1–§5.4: candidates, verdicts, the isolating seam (`ci_layer.py`,
    §6), and the revisit triggers (the suite passes 15 min on CI; minutes run short; the repository
    goes public).
  - **DEP-S13-b** — *Container images: the official `rust` slim image to build, Debian slim to run, both
    pinned by digest.* Covers §5.5, including distroless as the planned hardening swap.
  - **ARC-S13-a** — *CI layers, triggers, and what blocks.* Covers §3.4, §3.6, §3.7: the four layers,
    `fetch-depth: 0`, "blocks" under F-1 (policy now, a ruleset when available), the canonical
    full-suite evidence being the PR's `test` run, and the economy rules.
  - Ids are placeholders. Before writing, grep `^## \(ARC\|DEP\)-` over every `origin/*` branch carrying
    `DECISIONS.md` and take the next free numbers, recording the mapping (as 11a did).
- `.structured-coding/standards.md`, "Current state of the checks": one paragraph saying CI runs the
  declared `cargo` and script checks through `scripts/ci_layer.py`, and which layer runs which. The JSON
  block is unchanged.

**Depends on:** freeze. **Non-goals:** no code.

- [ ] Implementation: the three records; the standards paragraph.
- [ ] Validation: `check_decision_ids` (all distinct); `check_doc_headings`; every cross-reference
  (ARC-30, ARC-35, D-12, QS-59, Q9, R-B7) checked by grep.
- [ ] Review: each DEP answers `REUSE_POLICY.md` §11's questions and gives a §12 reason per declined
  option. ARC-S13-a states F-1 truthfully: it claims no mechanism that the repository's settings do not
  have. No defined term is redefined.

**Commit boundary.** Documentation only.

## A-C2 — The Dockerfile, `.dockerignore`, and the pin check

**Goal.** O-1: a committed, pinned, multi-stage image definition, and a mechanical guard on its pins.

**Scope.**
- `Dockerfile`: §3.1's three stages. `ARG`-free pins, written literally, so the check can read them. The
  `toolchain` stage copies no source. The `build` stage uses `--locked`.
- `.dockerignore`: `.git`, `target`, `clients/`, `presentation/`, `spike/`, `docs/`, `.structured-coding/`,
  `.claude/`, `mineworld-3d`, `mineworld-slice`, `**/*.sqlite`. The build context then carries only the
  workspace and `worlds/`; verified by the build's "transferring context" size.
- `scripts/check_ci_pins.py`: stdlib Python, in the style of `check_decision_ids.py` (module docstring
  stating why it exists; exits non-zero naming each disagreement). It reads `rust-toolchain.toml`'s
  `channel` (the `tomllib` module is in the standard library), the root `Cargo.toml`'s `rust-version`,
  and every `FROM` line of the Dockerfile. It checks: the `rust:` tag's version equals the channel and the
  `rust-version`; every `FROM` has `@sha256:`; the build and runtime stages name the same Debian release.

**Depends on:** A-C1.

- [ ] Implementation: the three files; tag and digests read with `docker buildx imagetools inspect` and
  recorded in the ledger with the date.
- [ ] Validation:
  - `python3 scripts/check_ci_pins.py` → passes.
  - A13-M4 and A13-M5 run locally as working-tree edits (reverted): each fails, naming the values.
  - `docker build --platform linux/amd64 --target runtime .` on the operator's Mac if the Docker daemon is
    running. Otherwise this is deferred to A-C3's CI image build and recorded as NOT RUN locally, with
    the reason.
  - The image's `mineworld validate` for the three worlds; the 1-day run; server start, stop, start on a
    named volume (A13-5).
- [ ] Review: no `latest` tag; no `curl | sh`; non-root runtime user; nothing secret in a layer; the
  runtime stage contains only the binary and `worlds/`; the build is `--locked`, so a stale `Cargo.lock`
  fails rather than resolving.

**Commit boundary.** Container files and the pin check only.

## A-C3 — `scripts/ci_layer.py` and `.github/workflows/ci.yml` (layers 1–2)

**Goal.** O-3 and O-4 for the per-PR layers.

**Scope.**
- `scripts/ci_layer.py`: `LAYERS = {"fast": [...], "core": [...]}` holding exactly §3.4's commands.
  - It prints the toolchain (`rustc -V`, `cargo -V`, `cargo fmt --version`, `cargo clippy --version`)
    and the checkout's shallowness and partial-clone filter first (A13-2, A13-4).
  - It runs each command, streams output, and stops at the first failure with that command's exit status.
  - With `--list` it prints the commands without running them.
  - The `core` layer also prints `df -h` and `du -sh target target/tmp` before and after (A13-3).
  - Unknown layer: exit 2, naming the known ones.
- `.github/workflows/ci.yml`:
  - `on: push` (branches per QS13-2's answer; `main` always), `pull_request`, `workflow_dispatch`.
  - `permissions: contents: read`.
  - `concurrency` as §3.4.
  - Jobs `fast` and `test` (`test` skipped on draft PRs). Each:
    1. `actions/checkout` at a full SHA, `fetch-depth: 0`, `filter: blob:none`, sparse-checkout per §3.5;
    2. `docker/setup-buildx-action` and `docker/build-push-action` (`target: toolchain`, `load: true`,
       `cache-from/to: type=gha`), all pinned to full SHAs;
    3. `actions/cache` for `.ci/cargo` and `target`;
    4. `docker run --rm --user "$(id -u):$(id -g)" -v "$PWD":/work -e CARGO_HOME=/work/.ci/cargo
       -e CARGO_INCREMENTAL=0 -e CARGO_PROFILE_DEV_DEBUG=line-tables-only mineworld-toolchain python3
       scripts/ci_layer.py <layer>`;
    5. before the cache save: `rm -rf target/tmp` and `cargo clean` of the workspace's own packages,
       which keeps dependencies only.
  - `timeout-minutes`: `fast` 20, `test` 45, so a hung run cannot burn the month.
  - A `workflow_dispatch`-only job `image` builds `--target runtime` for `linux/amd64` and runs A13-5's
    commands. In 13a it is the image's evidence; 13b grows it into `scenario`.
- `.gitignore`: `/.ci/`.

**Depends on:** A-C2.

- [ ] Implementation: the script, the workflow, the ignore line.
- [ ] Validation (the PR's own runs are the evidence; each recorded with run id, head SHA, wall time):
  - `python3 scripts/ci_layer.py --list fast` and `core` locally: the commands equal §3.4 and
    `standards.md`'s declared checks (a review diff, recorded).
  - PR run 1 (cold): A13-1, A13-2, A13-3, A13-4.
  - PR run 2 (cached, a docs-only push): A13-6.
  - `workflow_dispatch` of `image`: A13-5.
  - A13-M1 … A13-M5 on scratch branches pushed only to run CI. Each branch is deleted after its run, and
    `git ls-remote --heads origin | grep -c scratch` → 0 is recorded.
  - nextest's harness question (§5.3): `cargo nextest list -p mineworld-persistence` run once locally if
    nextest is already installed. Otherwise recorded NOT RUN, and the §5.3 verdict rests on nextest's
    documented custom-harness requirements, cited.
- [ ] Review:
  - Every third-party action is pinned to a full SHA, with its version in a comment.
  - No `continue-on-error`, `|| true` or retry in a blocking job (I-S13-4).
  - The workflow contains no check command (I-S13-9).
  - Root-owned files cannot break the cache step (`--user`).
  - Partial clone plus sparse checkout leaves every history-reading test's verdict unchanged (A13-1),
    or the fallback in §3.5 was taken and recorded.

**Commit boundary.** Workflow, layer script and ignore line. Scratch mutations are never committed to
the PR branch.

## A-C4 — Close: status, ledger, handoff

- [ ] `docs/MVP_STATUS.md` S13 row: 13a merged state, the layer table's first two rows live.
- [ ] `README.md`: two lines, "CI" and "Run in Docker", linking to ARC-S13-a and the Dockerfile.
- [ ] Ledger: the measured cold and cached wall times and minutes per PR (§10.1, replacing estimates); disk
  peak; the mutation evidence; deviations.
- [ ] A13-7 (`git diff --stat main...HEAD` lists only §9's files) and A13-8 (two 300-day digests,
  locally, on the final head).
- [ ] Review: every A13 item has evidence on the exact final head; the four §7.1 adversarial results are
  recorded; nothing material arose, or it went to the operator.

## 9.2 Test ownership for 13a

```text
STATIC      check_ci_pins.py (A13-M4, A13-M5); fmt/clippy/doc checks now run by CI
UNIT        none (no Rust changed)
INTEGRATION the unchanged default suite, run by `test` (A13-1)
REAL RUN    the runtime image hosting a world on a volume (A13-5)
GATE 1      NOT REQUIRED
CI          this PR's own `fast` and `test` runs on its exact final head are the canonical evidence;
            no local full suite is run for 13a (test rules §9), except A13-8's two 300-day runs
```

## 9.3 Is any of this material?

Yes, and it is raised rather than assumed:

- **F-1.** `D-12`'s "protection makes that a mechanism" is false on this plan. ARC-S13-a records the
  truth. The fix costs money or changes publication (QS13-1, operator).
- **The Actions budget** (QS13-2, operator). 13a's own validation spends about 2 cold and 6 cached runs
  (≈ 150–200 min, §10.1).
- **The canonical evidence moves.** After 13a, "READY FOR OPERATOR REVIEW" cites CI rather than a local
  full gate. That changes every later execution contract's wording (ARC-S13-a), not its rigour.

Nothing in 13a changes a frozen invariant of another step, MineWorld code, or the behaviour of any world.

## 9.4 Proposed execution contract (for the primary session to fill and freeze)

```text
PROJECT / PR        MineWorld mvp0 — PR 13a, the container and the per-PR CI (S13)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-14-ci.md §9 (this section is the live ledger)
RELATED / BINDING   this file §§1–8, 10–11; overall.md §3 (S13), §5 (D-12), §7; docs/ENGINEERING_STANDARDS.md
                    §§15–16; docs/REUSE_POLICY.md §§11–12; docs/DECISIONS.md ARC-23, ARC-30, ARC-35;
                    .structured-coding/standards.md; CLAUDE.md §§2–4
IMPLEMENTATION BASE the main named at freeze; branch mvp0/pr-13a-ci; worktree
                    /Users/yuema137/mineworld-worktrees/s13-13a (proposed), held by the implementing
                    session only
APPROVED SCOPE      §9.1's scope; A-C1 … A-C4
FROZEN INVARIANTS   I-S13-1 … I-S13-9 (§4); 13a's non-goals (§9.1)
SEQUENCE            A-C1 → A-C2 → A-C3 → A-C4, each committed and pushed when coherent; the PR is opened
                    after A-C2 so that A-C3's workflow runs on it
VALIDATION BUDGET   ordinary local checks: unrestricted. Docker builds: local, bounded to the three
                    stages. CI: at most 12 runs of `test`-sized work in total for 13a (PR runs plus
                    the five mutations plus `image`), each with timeout-minutes; anything beyond →
                    stop with a minutes projection. Monetary: none; if the account's included minutes
                    are exhausted, stop (do not raise a spending limit). Local full suite: not run.
                    A13-8's two 300-day runs: once.
LIVE DOCUMENTATION  §9 of this file
HANDOFF             .structured-coding/plans/mvp0/handoff.md, re-initialized for 13a
ENDPOINT AUTHORITY  implementation + local validation: authorized (operator, 2026-09-25 autonomous
                      execution authorization, overall.md)
                    semantic commits: authorized (same source)
                    branch push: authorized (D-12)
                    PR creation / update: authorized (D-12)
                    scratch branches pushed to run CI, then deleted: proposed as authorized; source:
                      unresolved until the primary session records it at freeze
                    workflow_dispatch of the `image` job: same as above
                    CI repair to review readiness: authorized (D-12)
                    repository settings (protection, rulesets, Actions policy, spending limit): NOT
                      authorized: operator only
                    merge: explicit operator authorization only
POST-MERGE SYNC     the implementation session owns this section's ledger and the merge identity; the
                    planning session owns overall.md §7 and the step header
NORMAL STOP         PR 13a READY FOR OPERATOR REVIEW — DO NOT MERGE
STOP CONDITION      A13-1 … A13-8 with evidence on the exact final head; A13-M1 … M5 recorded; `fast`
                    and `test` green on that head
MERGE AUTHORITY     never without explicit operator approval
```

---

# 10. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| **R-1** | **Actions minutes on a private Free account** (§10.1): the projected per-PR load exceeds the included 2 000 min/month. | §3.4's economy rules; QS13-2 (operator) chooses between triggers, a public repository, Pro, or a spending limit; measured figures replace estimates in 13a A-C4. |
| **R-2** | **Runner disk.** The suite needs ≈ 20 GB (F-3), and a hosted runner's free disk may be less. | A13-3 measures it in the first run. Remedies in order, each recorded: (1) `CARGO_PROFILE_DEV_DEBUG=line-tables-only` and `CARGO_INCREMENTAL=0` (already planned); (2) remove preinstalled SDKs on the runner (a step deleting `/usr/share/dotnet`, `/usr/local/lib/android`, `/opt/ghc`, the well-known approach also packaged as `jlumbroso/free-disk-space`, MIT); (3) a larger runner (money, operator). Changing tests to write less is outside S13 (I-S13-2) and would be raised to their owners. |
| **R-3** | A real cross-platform difference shows up in P-2 or P-3, most plausibly in Rapier's floats on x86_64, or in iteration order somewhere. | That is AC-8 doing its job. The ledger records the located chunk; the defect belongs to the owning pack (S15 for `bodies`) and is a material finding against step-11's R-B7. S13 does not "fix" it by weakening the comparison (I-S13-6). |
| **R-4** | References churn: every behaviour-changing PR must re-record `ac8.ref` on the laptop. | It is the I-7 duty made mechanical. The record command is one line. QS13-12 sequences 13b after 12d to avoid three re-records in a week. |
| **R-5** | Partial clone or sparse checkout changes a history-reading test's verdict. | A13-1 checks it on the PR; §3.5's fallback drops sparse checkout first. |
| **R-6** | A pinned digest is withdrawn upstream (official images keep old digests; Chainguard's free tier does not, which is one reason it was declined). | The pinned digests are recorded in the ledger with their date; a missing digest fails the build loudly, and repinning is a reviewed change checked by `check_ci_pins.py`. |
| **R-7** | Supply chain: third-party actions. | Full-SHA pins (I-S13-7); `permissions: contents: read`; no secrets. Dependabot for actions is QS13-11. |
| **R-8** | Parallel planning: S11, S12, S14 and S15 edit `docs/DECISIONS.md`, `docs/MVP_STATUS.md` and `Cargo.lock`. | Placeholder ids; S13 appends and re-greps ids at freeze; `Cargo.lock` changes only in 13b (one dev-dependency) and is regenerated on conflict. |
| **R-9** | CI becomes a rubber stamp: green because a check was silently skipped. | I-S13-2, I-S13-4; `ci_layer.py` prints every command it runs; A13-1 counts tests; the mutations prove each job bites. |

## 10.1 The minutes and cost picture

**Status of the figures.** Billing is unreadable with the session's token (§2.2). The plan figures are
GitHub's published terms as this session knows them (†), and the per-run figures are estimates scaled
from E-S13-0 to a 2-vCPU x86_64 runner (the size GitHub gives private repositories†). A13-6 replaces
the estimates with measurements.

```text
account             private repository, User account, GitHub Free (inferred from the 403s)
included            2 000 Linux minutes / month†; macOS billed at 10×†; 500 MB artifact and package
                    storage†; Actions cache 10 GB per repository† (not counted against storage†)
beyond included     billed per minute only if the operator has set a spending limit above $0†;
                    otherwise jobs stop running until the month resets†
public repository   standard hosted runners free and unlimited†; branch protection and rulesets free†
GitHub Pro          about $4 / month†: 3 000 minutes, and protection and rulesets on private
                    repositories†

per run (estimate)  fast 4–8 min cached · test 15–25 min cached (25–35 cold) · scenario 12–18 ·
                    stability 30–60 · clients 10–15
project rate        61 PRs in about 14 days ≈ 4–5 PRs a day; ≈ 3 pushes per PR after it opens
monthly (estimate)  test:   4.5 PRs × 3 pushes × 20 min × 30 days   ≈ 8 100 min
                    fast:   that plus branch pushes                ≈ 2 000–3 000 min
                    scenario on main pushes (≈ 4.5/day × 15 min)   ≈ 2 000 min
                    nightly stability + clients (≈ 60 min × 30)    ≈ 1 800 min
                    total                                          ≈ 14 000 min / month
```

So on GitHub Free, private, the design as drawn costs **about seven times the included minutes**. The
choice is the operator's (QS13-1, QS13-2):

- **(a) Make the repository public.** Minutes and protection are free. This is a publication decision,
  and D-1's licence revisit applies.
- **(b) Stay private and spend.** Pro or a spending limit. At GitHub's published Linux rate† the overage
  of ≈ 12 000 min is on the order of tens of dollars a month. To be confirmed on the billing page.
- **(c) Stay private and free.** Cut the load to fit 2 000 minutes: `test` only on `ready_for_review` and
  on the last push before review (a `ci:full` label or `workflow_dispatch`); no `scenario` on every main
  push, nightly only; `stability` weekly; `fast` only on PRs. About 1 800 minutes. The cost is that the
  canonical evidence arrives later in a PR's life.
- **(d) A self-hosted runner** on the operator's Mac for layers 3–4. It is arm64 macOS, so it cannot
  replace the Linux x86_64 side of AC-8, and it runs PR code on the operator's machine.

---

# 11. Questions

`[OM]` marks an operator-material question: it costs money, changes repository settings or branch
protection, or changes an operator decision. The rest the primary session can decide.

| ID | Question | Recommendation |
| --- | --- | --- |
| **QS13-1 [OM]** | `main` cannot be protected on a private Free repository (F-1; `D-12` assumed it was). Public, Pro, or policy-only? | If publication is near: **public** (free minutes and protection; D-1's licence revisit first). Otherwise **Pro** (≈ $4/month†) so `fast` and `test` become required checks. Policy-only remains the floor and is what ARC-S13-a records until then. Whatever is chosen, `D-12`'s text should be corrected (§12). |
| **QS13-2 [OM]** | Minutes budget (§10.1): which of (a)–(d), and should a spending limit above $0 exist? | Decide together with QS13-1. If private and free, take (c) and measure for a month. No spending limit is raised by any session. |
| **QS13-3 [OM]** | A live macOS runner comparison for AC-8, in addition to the committed references? | **No.** It costs 10× minutes, and the laptop gate plus references give the same equality transitively (§3.3). Revisit if the laptop gate is ever skipped in practice. |
| **QS13-4** | Toolchain approach: (A) build the Dockerfile's stage and `docker run`, or (B) `container:` with inline setup? | **(A)** (§5.2): one definition of the environment, and no fail-open checkout without `.git`. |
| **QS13-5** | Re-tier the default suite into separate core and scenario jobs now? | **No.** Layer 2 is the whole default suite (§3.4). Revisit with a fail-closed selector when the suite passes ~15 min on CI. |
| **QS13-6** | AC-8's worlds and ages: towns 300 days in memory plus 30-day saves; bodies-yard 30 days. | **Accept.** It covers E-RS0's horizon and byte-level tables for all three worlds, at ≈ 1 min locally. |
| **QS13-7** | CI-only `CARGO_INCREMENTAL=0` and `CARGO_PROFILE_DEV_DEBUG=line-tables-only`. | **Accept.** No codegen semantics change; ARC-30's stated settings are untouched; disk and cache shrink. Recorded in ARC-S13-a. |
| **QS13-8** | `sha2` as a dev-dependency of `mineworld-cli` (`DEP-S13-c`). | **Accept** (§5.6). |
| **QS13-9** | Godot probes exit 0 on failure (F-5). Should S13 change the probes? | **No**: clients are S12/S14's (I-S13-1). 13c parses a verdict line and treats a missing one as INCONCLUSIVE. S12/S14 are asked to make probes exit non-zero on failure; until then `clients` reports only. |
| **QS13-10** | Runtime base: Debian slim now, distroless later? | **Accept** (§5.5). Swap when a hosted deployment exists. |
| **QS13-11** | Dependabot for GitHub Actions pins? | **Not yet.** SHA pins are updated deliberately; revisit when the repository is public (Dependabot PRs also cost CI minutes). |
| **QS13-12** | 13b relative to S15's 12c/12d (both change the digests 13b records). | Land **13b after 12d** if 12d is within days; otherwise land 13b, and 12c/12d each re-record `ac8.ref` in the commit that re-baselines (their PR designs gain one validation line). |
| **QS13-13** | `tools/benchmark` (overall §3, "with S13"): a crate or a report? | **A report** in 13c (wall time per world-day, nightly, as an artifact and a step summary). A crate waits for a second consumer (`ENGINEERING_STANDARDS.md` §28). |
| **QS13-14 [OM]** | Scratch branches pushed only to trigger CI for the mutation evidence (13a's A13-M1…M5): authorize pushing and deleting them? | **Authorize** at freeze, bounded to `scratch/13a-*`, deleted after each run and checked with `ls-remote`. Without it, the mutations are run locally and recorded as local evidence only (weaker for M3, the checkout depth, which only CI can show). |
| **QS13-15** | S11 dependency: the runtime image's `CMD` predates S11's invite token. | Not blocking. 13a ships the current server's arguments. S11's PR (or a one-line follow-up) adds the token argument, and the image's interface (§6) is unchanged. |

---

# 12. Parallelism, files touched, and proposed parent edits

## 12.1 Files each PR touches

```text
13a   Dockerfile · .dockerignore · .gitignore (/.ci/) · .github/workflows/ci.yml · scripts/ci_layer.py ·
      scripts/check_ci_pins.py · docs/DECISIONS.md (append DEP-S13-a, DEP-S13-b, ARC-S13-a) ·
      .structured-coding/standards.md (prose only) · README.md (two lines) · docs/MVP_STATUS.md (S13 row) ·
      this file
13b   tools/cli/Cargo.toml ([dev-dependencies] sha2) · Cargo.lock · tools/cli/tests/ac8_parity.rs ·
      tools/cli/tests/parity/ac8.ref · .github/workflows/ci.yml (scenario job) · scripts/ci_layer.py
      (`parity` layer) · Dockerfile (cargo-chef, only if §5.4's threshold) · docs/DECISIONS.md (DEP-S13-c,
      ARC-S13-b) · docs/MVP_STATUS.md (AC-8 row) · this file
13c   .github/workflows/ci.yml (stability, clients) · scripts/ci_layer.py (`stability` layer) ·
      possibly scripts/godot_probe.py (verdict parsing) · docs/MVP_STATUS.md · this file
```

## 12.2 Parallelism with S11, S12, S14, S15

| Other step | Its files (as planned) | Overlap with S13 | Consequence |
| --- | --- | --- | --- |
| S11 server | `server/`, `tools/cli/src/` (server arguments), `docs/NETWORKING.md` | none in 13a/13c; 13b none in `src/` | S11 may add an argument to the runtime `CMD` (QS13-15). S11's tests run in S13's CI automatically |
| S12 2D client | `clients/` | none; `clients/protocol/` stays in the sparse checkout | 13c's probes read S12's artefacts and depend on S12's exit status (QS13-9) |
| S14 3D client | `clients/3d-spike/`, launch scripts | none (excluded from CI checkout, §3.5) | as S12 |
| S15 12c–12e | `systems/bodies`, `systems/*`, `worlds/*`, tests | 13b's references record their worlds' digests | QS13-12 sequencing; `Cargo.lock` regenerated on conflict |
| Milestone E | unknown at this writing | likely `docs/` | ids are placeholders; MVP_STATUS rows are per-step |

13a can be implemented immediately and in parallel with all of them: it touches no Rust, no test, and no
file another step plans to edit, apart from appended records and one status row.

**Every other step gains CI.** Once 13a merges, S11/S12/S14/S15 PRs get `fast` and `test` on every push,
which changes their execution contracts' "canonical evidence" line (ARC-S13-a). The primary session should
tell those planning sessions at freeze.

## 12.3 Proposed edits to parent documents (applied by the primary session, not by this session)

- **`overall.md` §3, S13:** replace the "Output" line with "Dockerfile (toolchain, build, runtime);
  layers 1–2 per PR, 3 on main and nightly, 4 nightly (ARC-S13-a); AC-8 by committed laptop references
  checked on Linux x86_64 (ARC-S13-b). Design: `step-14-ci.md`, PRs 13a–13c."
- **`overall.md` §5, D-12:** append "**Corrected 2026-10-08 (S13 audit, F-1):** branch protection and
  rulesets are unavailable on a private repository under GitHub Free (API 403); merge-by-authorization is
  policy, not mechanism, until QS13-1 is decided."
- **`overall.md` §7:** add an S13 entry: "S13 planned (`step-14-ci.md`, DRAFT): 13a container + per-PR CI,
  13b AC-8 parity, 13c nightly stability; QS13-1/-2/-3/-14 with the operator." Mark the carried
  `fetch-depth: 0` item as "designed in step-14 §3.6, satisfied when 13a merges".
- **`overall.md` §3, "Cross-cutting":** `tools/benchmark with S13` → "with S13 (13c), as a nightly timing
  report" (QS13-13).
- **`docs/MVP_STATUS.md` S13 row:** "planned: `step-14-ci.md` (13a–13c); not frozen".
- **`step-11-bodies.md` R-B7:** "a native x86_64 host (S13)" → "S13's P-2 and P-3 (`step-14-ci.md` §3.3)",
  once 13b merges.
