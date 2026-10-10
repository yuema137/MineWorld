# Step 14 — Deployment parity and layered CI (S13)

**Role:** step document for **S13 — Deployment parity and layered CI**, with the first PR (13a) detailed
to the commit in §9 so that it can be frozen and implemented quickly. Later PRs (13b, 13c) are at medium
scope (`CLAUDE.md` §3, "detail one step ahead").
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) §3 (S13), §4 (`AC-8`, layered CI), §7 (the carried
`fetch-depth: 0` requirement).
**Lifecycle:** `STEP DESIGN FROZEN (2026-10-08)` — frozen at step level by the primary session under the operator decisions and coordination rulings in `overall.md` "Parallel build-out, 2026-10-08", which bind and override this document where they differ (decision numbers, protocol ownership, event perception, the shared module, digests). Superseded wording below: `DRAFT — awaiting the primary session's review`. Nothing in this document authorizes
implementation. Decision records proposed here carry placeholder ids (`DEP-S13-a`, `ARC-S13-a`, …); the
primary session numbers them at freeze.
**PR 13b: `DESIGN FROZEN (2026-10-08), primary session`.**
- §13 holds the design, the rulings (QB-1 … QB-10, operator and primary session) and the operator's
  all-platforms requirement with its Windows revisions (§13.0.1).
- The design was written by the 13b planning session: worktree
  `/Users/yuema137/mineworld-worktrees/plan-13b-ci`, branch `plan/s13-13b`, from `main @ 9cf8f8e`.
- Implementation runs under §13.12's contract, in a fresh session.
- §13 revises §3.3 and §7.2 as ruled.

**PR 13c:** outline only (§14).
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

> **Superseded in detail by §13 (2026-10-09), pending its freeze.** The outline below is kept as the
> step-level record. §13.0 lists every point where §13 departs from it and why: the comparison becomes
> live (a macOS runner against the Linux container) instead of a committed laptop reference, nothing is
> added to the required `test` check, `sha2` is not needed, and 13b no longer waits for 12d's digests.

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

> **Refined by §14's outline (2026-10-09).** §14 restates 13c against what 13a and 13b actually built
> and against the probes' current exit statuses.

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

## 9.0 Freeze-ready revision (2026-10-08, implementing session)

**Status:** `DESIGN FROZEN (2026-10-08), primary session`. Evidence: the primary session's freeze
message to the 13a session, 2026-10-08: "FREEZE: 13a is DESIGN FROZEN (2026-10-08), primary session. Your
§9.0 revision R-1 to R-6 is accepted as written", covering the full triggers, the `scratch/*-image` route,
no settings changes, job names `fast` and `test`, ARC-48 / DEP-17 / DEP-18 for 13a (ARC-49, DEP-19
reserved for 13b), `scratch/13a-*` branches deleted after use, and the contract amendments. Material
stops: any spending, any settings or protection change, any Rust or test edit, exceeding the 12-run
budget. Written by the 13a implementing session (worktree `/Users/yuema137/mineworld-worktrees/impl-s13a`, branch
`mvp0/pr-13a-ci`, from `main @ 47c81d1`). It applies the operator's 2026-10-08 decision to make the
repository public and the coordination rulings in `overall.md` "Parallel build-out, 2026-10-08" to §§3–9.
Where this subsection and older text in this file disagree, this subsection governs 13a.

### R-0 Base re-audit (main @ 47c81d1)

- The 29 commits in `0fd0be3..47c81d1` touch only `.structured-coding/plans/` (7 files, `git diff --stat`).
  §2's audit therefore holds unchanged: still no `.github/`, no `Dockerfile`; `rust-toolchain.toml` still
  `1.97.1` with `rustfmt` and `clippy`.
- GitHub API (GET, 2026-10-08): the repository is **still private**; `actions/workflows` → `0`;
  `actions/permissions` → enabled, `allowed_actions: all`, `sha_pinning_required: false`.
- The reserved numbers ARC-48, ARC-49 and DEP-17…19 appear in no `docs/DECISIONS.md` on any `origin/*`
  branch (grep over every remote branch).

### R-1 Triggers: the full design, not option (c)

The operator chose §10.1's option (a): public. The reduced trigger set of option (c) is therefore
dropped, and §3.4's table is implemented as drawn for layers 1–2:

| Job | Trigger in 13a's `ci.yml` |
| --- | --- |
| `fast` | `push` to **every** branch; `pull_request` (`opened`, `synchronize`, `reopened`, `ready_for_review`) |
| `test` | `pull_request` of a non-draft PR (same types); `push` to `main`; `push` to `scratch/**` (mutation evidence, R-4) |
| `image` | `workflow_dispatch`; `push` to a branch matching `scratch/*-image` (see the discovery below) |

- **The economy bullet of §3.4 is withdrawn.** It restricted `push` to `main`, `wip/**`, `plan/**` and
  `docs/**` to avoid running `fast` twice on a PR branch. That was a minutes measure, and minutes are free
  on a public repository. `fast` on every push gives a planning or docs branch its structural verdict
  before any PR exists. The cost is a second `fast` run on a PR branch's push, and it is accepted. The
  `concurrency` rule stays as §3.4 states it: per ref, cancel in progress except on `main`.
- **Draft PRs** still run `fast` only. Turning a PR ready (`ready_for_review`) starts `test` on the same
  head.
- **Discovery: `workflow_dispatch` cannot run before 13a merges.** GitHub offers a dispatch only for a
  workflow file present on the default branch, and `ci.yml` first reaches `main` when 13a merges. A-C3's
  "`workflow_dispatch` of `image`" for A13-5 is therefore impossible inside 13a. Bounded correction: the
  `image` job also runs on a push to a branch named `scratch/*-image`. 13a's A13-5 evidence comes from
  one such scratch branch, pushed and deleted (R-4). If that run is unavailable, the fallback is A-C2's
  local `docker build --platform linux/amd64 --target runtime`, recorded as local evidence. After merge,
  `workflow_dispatch` is the normal way to run `image`.
- **Until the flip happens** the repository is private on GitHub Free, and 13a's own runs are billed
  against the 2 000 included minutes. The contract's budget (§9.4: at most 12 runs of `test`-sized work,
  no spending) stays binding for 13a regardless of when the flip happens. Its expected use: PR cold run
  and cached run (2); A13-M1…M5 (5); the `image` scratch run (1); repairs and the final head (≤ 4).
  `fast`-only mutations (M1, M4, M5) cancel their run with `gh run cancel` once `fast` is red, so their
  parallel `test` job stops early.
- **Runner size.** Hosted `ubuntu-24.04` for a private repository has 2 vCPU. A public repository gets the
  larger standard runner (4 vCPU at GitHub's published terms†). 13a's A13-3 and A13-6 figures are measured
  before the flip, so the ledger labels them "private runner". They are an upper bound on the public
  figures, not a substitute for them. The first post-flip `test` run on `main` is recorded beside them by
  whichever session observes it. This is not a 13a acceptance item.

### R-2 Branch protection is a follow-up, not 13a's

- 13a configures **no** repository setting: no protection, no ruleset, no Actions policy, no spending
  limit (material stop, §9.4).
- **After the flip**, the primary session, with the operator, enables protection (or a ruleset) on
  `main` that requires the checks `fast` and `test`, plus "require branches to be up to date", because
  AC-1's scan reads merge structure. At that point `D-12`'s "protection makes it a mechanism" becomes
  true.
- What 13a must do so that follow-up works:
  - The two jobs' check names are exactly `fast` and `test`: no matrix, no decorated `name:`. A required
    check matches by name, so these two names are a stable interface and ARC-48 records them as such.
  - ARC-48 states the state of enforcement truthfully on the day it merges. If protection is not yet
    enabled, "blocks" means the §3.7 policy: green `fast` and `test` on the exact final head are required
    for `READY FOR OPERATOR REVIEW`, and the operator merges only on both green. It names the follow-up
    that turns this into a mechanism.
  - ARC-48 does not claim the mechanism exists before it does.
- **A known property for the follow-up to weigh, not a 13a defect.** A job skipped by its `if:` (for
  example `test` on a draft PR) reports "skipped", and GitHub counts a skipped required check as
  satisfied. A draft cannot be merged, and `ready_for_review` re-runs `test` on the same head, so the gap
  does not open in practice. ARC-48 records it.
- **Public-repository settings the follow-up owns,** also not 13a's:
  - the fork-PR approval policy ("require approval for first-time contributors" or stricter);
  - whether to set `sha_pinning_required: true`, which 13a's full-SHA pins already satisfy;
  - Dependabot for action pins, which QS13-11 deferred "until the repository is public".

  13a's workflow is already safe for fork PRs: it uses `pull_request`, never `pull_request_target`, has
  `permissions: contents: read`, and needs no secret (I-S13-7). The Actions cache is scoped per ref, so a
  fork PR cannot write `main`'s cache entries.

### R-3 Decision numbers (coordination ruling 6)

| Placeholder | Number | Title (as §A-C1 / §7.2) | PR |
| --- | --- | --- | --- |
| `DEP-S13-a` | **DEP-17** | CI runs on GitHub Actions hosted Linux runners, inside the repository's own toolchain container | 13a |
| `DEP-S13-b` | **DEP-18** | Container images: official `rust` slim to build, Debian slim to run, both pinned by digest | 13a |
| `DEP-S13-c` | **DEP-19** | `sha2` as a dev-dependency of `mineworld-cli` for the parity test | 13b |
| `ARC-S13-a` | **ARC-48** | CI layers, triggers, and what blocks | 13a |
| `ARC-S13-b` | **ARC-49** | How AC-8 is measured | 13b |

From here on, every placeholder in this file reads as its number. A-C1's "grep for the next free numbers"
step is replaced by a re-check that these five are still unused on every `origin/*` branch just before
A-C1 commits.

The public decision changes two records' content, not their numbers:

- **DEP-17** records §5.1's cost column as "public: free standard runners", and drops "the repository
  goes public" from its revisit triggers, since that has now happened.
- **DEP-18 / §5.2 option (D)**, a prebuilt toolchain image on GHCR, loses its "private storage" reason
  for declining, because public packages are free. It stays declined for 13a for two reasons. Pushing an
  image needs `packages: write`, which conflicts with I-S13-7's `contents: read` on PR jobs. And (A)
  costs about 30–60 s per job with the GHA layer cache. DEP-17 records (D) as a revisit when per-job image
  build time is measured above about 2 min.

### R-4 Scratch branches (QS13-14 accepted)

- The operator accepted QS13-14 (relayed by the primary session's 13a kickoff, 2026-10-08). The 13a
  session may push branches named `scratch/13a-*` only to trigger CI for A13-M1…M5 and the `image`
  evidence (`scratch/13a-image`), and must delete each after its run.
- `git ls-remote --heads origin 'scratch/*'` → empty is recorded at A-C4.
- Scratch commits are never merged and never cherry-picked onto `mvp0/pr-13a-ci`. Each run's id, head SHA,
  the job that turned red and the decisive log line go into the A-C3 ledger.
- M3 mutates the workflow's checkout on its scratch branch. Because the push trigger reads the workflow
  from the pushed commit, the mutated `ci.yml` is the one that runs. This is what makes M3 observable on a
  scratch branch at all.

### R-5 Other rulings that touch 13a

- **Ruling 10 (test hygiene).** About 16 GB of scratch saves (F-3) are fixed by a separate bounded PR
  after 12c. That PR is not 13a, and I-S13-2 still forbids 13a from editing tests. If A13-3 shows the disk
  does not hold, §10's R-2 remedies apply in order, and remedy (2), freeing the runner's preinstalled
  SDKs, is a workflow step inside 13a's scope.
- **Ruling 7 (PR numbering).** "13a" is a working name. The GitHub PR number is whatever GitHub assigns.
- **Ruling 8 (lanes).** 13a starts now. 13b waits for 12d.

### R-6 Contract amendments (§9.4, proposed with this revision)

```text
IMPLEMENTATION BASE  main @ 47c81d1; branch mvp0/pr-13a-ci; worktree
                     /Users/yuema137/mineworld-worktrees/impl-s13a (replaces the proposed s13-13a)
scratch branches     authorized, bounded to scratch/13a-*, deleted after each run;
                     source: operator acceptance of QS13-14, relayed in the primary session's 13a
                     kickoff, 2026-10-08
workflow_dispatch    N/A before merge (R-1 discovery); replaced by the scratch/13a-image push run
repository settings  NOT authorized (unchanged): protection, rulesets, Actions policy, spending limit;
                     source: kickoff "material stops", 2026-10-08
Rust code and tests  NOT authorized (unchanged, I-S13-1, I-S13-2; kickoff "material stops")
spending             none (unchanged; kickoff "material stops")
```

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

- [x] Implementation: `DEP-17`, `DEP-18`, `ARC-48` appended to `docs/DECISIONS.md` (numbers per §9.0 R-3,
  re-checked unused on every `origin/*` branch just before writing); the standards paragraph ("CI runs the
  same declared checks through one entry point"). The JSON block is unchanged.
- [x] Validation (2026-10-08, working tree on `e07b3ea`): `check_decision_ids` → 54 decision ids, all
  distinct (51 + 3); `check_doc_headings` → 176 sections across 25 documents, none duplicated. The
  cross-references ARC-30, ARC-35, D-12, QS-59 and R-B7 resolve by `git grep` (to `DECISIONS.md`,
  `overall.md`, `step-10-market.md` and `step-11-bodies.md`). Q9 is step-08's, cited as in §3.3.
- [x] Review:
  - Each record states its problem, its options, the choice, why not ourselves, why not the others (a
    `REUSE_POLICY.md` §12 reason for each), the isolating interface, and the accepted limitations.
  - ARC-48 states F-1 truthfully. Enforcement is policy now; protection is a named follow-up after the
    flip; and no claim is made of a mechanism the settings lack.
  - No defined term (`World Pack`, `System Pack`, …) is redefined.
  - Bounded additions, recorded here:
    - DEP-18 adds `STOPSIGNAL SIGINT`. The audit of `tools/cli/src/main.rs::serve` shows the server stops
      only on `ctrl_c()`. As PID 1 with no `SIGTERM` handler, `docker stop` would otherwise wait 10 s and
      `SIGKILL` it. No code change is involved.
    - The `nextest` decline cites nextest's documented custom-harness requirement, because nextest is
      not installed here (§A-C3).

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

- [x] Implementation: `Dockerfile`, `.dockerignore` and `scripts/check_ci_pins.py`. Pins were read with
  `docker buildx imagetools inspect`, which needs no daemon, on 2026-10-08 19:41 UTC:

  ```text
  rust:1.97.1-slim-trixie  index sha256:8e8cf8f7fd54a2d23d5a743b3a03f56e26b6c774276c33fa0595111704ebb15c
                           (linux/amd64 manifest sha256:fc0648ac…697c5a, created 2026-08-05,
                            base debian:trixie-slim; source rust-lang/docker-rust@40acf791)
  debian:trixie-slim       index sha256:a29215f6a35e51e22adffa17f89e9d2ef06214e64a2bad10d765c46aea49f11f
  ```

  Debian `trixie` is the expected variant (§3.1), so no fallback was needed. Both stages pin the
  multi-platform index digest, so the same `Dockerfile` builds `linux/amd64` in CI and `linux/arm64` on
  the Mac.
- [x] Validation (local pins and mutations; the image evidence is in CI, as recorded):
  - [x] `python3 scripts/check_ci_pins.py` → exit 0: "toolchain pins agree: rust 1.97.1
    (rust-toolchain.toml, Cargo.toml, Dockerfile), Debian trixie, every base image pinned by digest".
    Run locally with Python 3.14.7; the container runs trixie's Python 3.13, and `tomllib` exists in both.
  - [x] Local mutations, as working-tree edits, each reverted, with the restored file passing again:
    - **M4**, tag `1.97.0` → exit 1: "Dockerfile:13: rust image tag is 1.97.0, rust-toolchain.toml
      channel is 1.97.1".
    - **M5**, a runtime `FROM` with no digest → exit 1: "Dockerfile:29: debian:trixie-slim is not pinned by
      an @sha256: digest".
    - **Extra**, a runtime on `bookworm` → exit 1: "base images name different Debian releases:
      ['bookworm', 'trixie']".
    - **Finding, fixed before commit.** The first M5 run also printed a second, misleading line ("neither
      rust:… nor debian:…"), because the tag patterns required an `@`. The patterns now accept a
      missing digest, so only the true cause is printed.
  - N/A, local `docker build`: **NOT RUN**. The Docker Desktop daemon was not running at any point in
    this session: checked at the start and again before closing (socket `~/.docker/run/docker.sock`
    absent). Starting it is not this session's to do. As planned, A13-5 rests on CI instead: the `image`
    job's run from `scratch/13a-image` (§9.0 R-1), recorded in A-C3, which is **PASS**: `validate` ×3,
    the 1-day run with `faults 0`, and start, stop and start again on a named volume.
  - The optional local `linux/arm64` build (§3.3) was not run, for the same reason.
- [x] Review:
  - Both `FROM`s carry a tag and a digest, and neither tag is `latest`.
  - Nothing is piped to a shell (`curl | sh`): packages come from apt, and components from the image's
    own rustup.
  - The runtime runs as the non-root user `mineworld`.
  - No secret, `ARG` or build secret appears anywhere.
  - The runtime stage copies only `/usr/local/bin/mineworld` and `worlds/`.
  - The build is `--locked`.
  - `.dockerignore` keeps the context to 3.59 MB (the image run's "transferring context").

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

- [x] Implementation: `scripts/ci_layer.py`, `.github/workflows/ci.yml` and the `/.ci/` ignore line. Two
  bounded additions, each recorded as a deviation:
  - **D-13a-1. `.github/actions/layer/action.yml`, a local composite action.** `fast` and `test` need the
    same four steps: build the toolchain image, restore the caches, `docker run` the layer, prune. A
    matrix would also share them, but it decorates check names or needs per-entry `if:` tricks, and the
    names `fast` and `test` are an interface (§9.0 R-2). Copying the steps into both jobs would duplicate
    about 40 lines. The composite action names a layer and holds no command, so I-S13-9 holds. Impact:
    one more file under `.github/`. Validation: the PR's own runs.
  - **D-13a-2. `scripts/ci_image.py`, the `image` job's evidence.** A13-5's checks are written as a
    stdlib script that reads each verdict from the program's output, not as shell lines in YAML. This
    keeps the workflow free of commands (I-S13-9), and 13b grows the script into `scenario`. The
    checks: `validate` for the three worlds; the 1-day run, which must print `faults 0`; and the default
    command on a named volume. That last one must create a world and listen on 7878, with a TCP connect
    accepted. Then `docker stop` must stop it with exit 0, and a restart must resume.
  - **D-13a-3. Mounts.** The container mounts the checkout at its own host path, and `RUNNER_TEMP`
    read-only, instead of mounting at `/work` (§9 A-C3, step 4). The reason: actions/checkout v6+ keeps
    the fetch credential in a `RUNNER_TEMP` file, included from `.git/config` by an `includeIf gitdir:`
    on the host path (checkout README, "What's new"). The history scans fetch old blobs of the partial
    clone on demand, which needs that credential while the repository is private. At `/work` the
    include would not match, and the scans would fail on a private repository.
  - **Choices within scope:**
    - The `test` job skips `scratch/*-image` branches, so the `image` evidence run does not spend a
      `test`-sized run (budget, §9.0 R-1).
    - The `image` job's checkout is depth 1 with `persist-credentials: false`, because it reads no
      history and runs no test.
    - Action pins, resolved 2026-10-08 via `gh api repos/<r>/releases/latest` → `commits/<tag>`:
      - `actions/checkout` v7.0.1 `3d3c42e5…`;
      - `actions/cache` v6.1.0 `55cc8345…`;
      - `docker/setup-buildx-action` v4.4.1 `f87e5991…`;
      - `docker/build-push-action` v7.4.0 `c3c9e263…`.
- Local evidence before the first push (2026-10-08, working tree on `1a8c0b7` plus the A-C3 files):
  - [x] `ci_layer.py --list fast` and `--list core` print exactly §3.4's commands: the six `fast`
    commands, then `cargo test --workspace`. These equal `standards.md`'s declared checks, plus
    `check_ci_pins.py`. An unknown layer, or `--list` with no layer, exits 2 with the usage line naming
    `fast, core`.
  - [x] `PATH=~/.cargo/bin:$PATH python3 scripts/ci_layer.py fast` on the Mac → "layer fast passed: 6
    command(s) in 23.9 s". It printed rustc 1.97.1, rustfmt 1.9.0, clippy 0.1.97, not shallow, and no
    partial-clone filter (a full local clone). Log: `/tmp/s13a/fast-local.log`, not committed.
  - **Finding F-13a-1, fixed.** The first local run crashed with a Python traceback, because `rustc` was
    not on this shell's PATH. That is an environment difference, but a traceback is not a verdict. Now a
    missing tool in the environment report prints "(not found on PATH)", and a missing tool in a layer
    command fails the layer with exit 127, naming the tool.
- CI evidence log. Runs are counted against the 12-run budget. Runs that hold `test`-sized work are
  marked ●.
  - **Run 1 ●, `pull_request` 37834779943** (head `dc1686b`, merge ref `d6f66ca`; PR #63), on a 2-vCPU
    private runner. The repository became public while it ran.
    - `fast` **PASS** in 3 min 19 s (19:49:09 → 19:52:28 UTC), cold:
      - toolchain image built in ≈ 92 s (checkout done 19:49:12; cache lookup 19:50:44);
      - inside the container: rustc 1.97.1, rustfmt 1.9.0, clippy 0.1.97, git 2.47.3;
      - `--is-shallow-repository` → `false`, `partialclonefilter` → `blob:none` (A13-2 and A13-4 hold for
        `fast`);
      - `cargo check` 72.3 s, `clippy` 18.0 s; the layer itself took 91.1 s;
      - cache saved as `cargo-fast-c44d1b0f…`.
    - `test` **INCONCLUSIVE**: cancelled at its 45-minute `timeout-minutes`. The only annotation reads "The
      job has exceeded the maximum execution time of 45m0s".
      - The job's log is **missing**: `actions/jobs/<id>/logs` → `BlobNotFound`, and `gh run view --log`
        → "log not found". So the stage reached and the disk use are both unknown.
      - Diagnosis (test rules §18). Two causes fit:
        - (a) a runner whose disk filled: F-3, about 20 GB needed. A full disk also stops the runner
          writing its own log, which fits the missing log.
        - (b) a cold build plus the full suite simply exceeding 45 min on 2 vCPUs. E-S13-0 measured 921 s
          user CPU on the Mac.
      - Fix, bounded and within §10 R-2, recorded as **D-13a-4**:
        - R-2 remedy (2): the composite action gains a `free-disk` input, which `test` sets, and which
          deletes the runner's .NET, Android, GHC and CodeQL trees before the build;
        - `test`'s `timeout-minutes` goes from 45 to 75;
        - the `core` layer builds with `cargo test --workspace --no-run` before running
          `cargo test --workspace`. The tests run are the same; the log now separates build time from
          test time, which A13-6 needs.
  - Push run 37834772746 (`fast` only) **PASS**; not test-sized.
  - **Main moved: merged into the branch.** PR #63 became `dirty`, conflicting with main's #64–#75 in
    `README.md` and `docs/DECISIONS.md`. While dirty, GitHub created **no** `pull_request` run for the
    pushed head `4665954`. Main was merged at `e8caf0d`, the repository's usual practice. Both sides were
    kept:
    - `README.md`: main's new "Where to look" list, plus the CI and Docker lines;
    - `DECISIONS.md`: main's DEP-13 note, ARC-53 and DEP-21, then DEP-17, DEP-18 and ARC-48.

    `check_decision_ids` → 56 ids, all distinct. The PR diff still touches only §12.1's files plus D-13a-1
    and D-13a-2.
  - **Run 2 ●, `pull_request` 37843100106** (head `e8caf0d`), the first run on a **public** runner.
    - `fast` **PASS** in 1 min 31 s. Its cache was restored from `main`'s scope; `fast` was saved in run 1.
    - `test` **FAIL**, a genuine finding: `tools/cli/tests/packs.rs:161`,
      `the_data_packs_in_named_directories_are_listed_and_validate`, panicked with "packs list failed:
      [mineworld] …/presentation/mineworld-default: could not be read: No such file or directory".
      - The test arrived on main with S16 E-a (#70) after §2's audit. It reads `presentation/`, which
        §3.5's sparse checkout omitted.
      - §3.5's fallback is taken, recorded as **D-13a-5**: sparse checkout is dropped from both jobs, and
        the partial clone (`blob:none`, `fetch-depth: 0`) is kept. The checkout is now uniform; `fast` gets
        the same tree as `test`.
      - Everything else in the run was healthy:
        - cold `cargo test --workspace --no-run` took 282.5 s;
        - 32 binaries had passed before the failure, `ac1_composability` 13/13 among them;
        - free disk was 107 GB before the tests (`/dev/root` 145 G) and 95 GB after;
        - `target/` was 11 G at the failure.
    - **Disk is not a problem on the public runner.** The `free-disk` step freed 22 GB (from 86 G to
      108 G available) but was not needed. It is removed again, so D-13a-4's remedy (2) is withdrawn on
      the evidence. `test`'s `timeout-minutes` returns to the design's 45. Run 1's timeout belonged to
      the 2-vCPU private runner, which no longer applies.
  - **`image` ●, push run 37843147556** on `scratch/13a-image` (head `e8caf0d`): **PASS**, A13-5.
    - The `image` job took 3 min 44 s. The build context was 3.59 MB, so `.dockerignore` holds.
    - The release `cargo build --locked -p mineworld-cli` took 132.7 s, cold.
    - `scripts/ci_image.py mineworld:ci` printed PASS for each of:
      - `validate` for social-cafe, market-town and bodies-yard;
      - the 1-day social-cafe run, with `faults 0`;
      - the default command creating its world on the volume and listening on 7878, with a TCP connect
        accepted;
      - `docker stop` stopping the server with exit 0 (SIGINT reached it as PID 1);
      - a restart on the same volume resuming the world and listening again.

      It ended with "every expectation held". The branch was deleted after the run.
  - **A13-M1 ●, run 37843159994** on `scratch/13a-m1` (`6f73b91`: two spaces after `assert_eq!(` in
    `contracts/tests/action.rs`): `fast` **red** at `cargo fmt --all --check`, "Diff in
    …/contracts/tests/action.rs:73", "[ci] layer fast FAILED at: cargo fmt --all --check". The run was
    then cancelled, which stopped its parallel `test`.
  - **A13-M4 ●, run 37843175171** on `scratch/13a-m4` (`d6f805f`: tag `1.97.0`): `fast` **red** at
    `check_ci_pins.py`: "Dockerfile:13: rust image tag is 1.97.0, rust-toolchain.toml channel is 1.97.1".
    Cancelled the same way.
  - **A13-M5 ●, run 37843200610** on `scratch/13a-m5` (`b1b72fd`: runtime `FROM debian:trixie-slim`, no
    digest): `fast` **red** at `check_ci_pins.py`: "Dockerfile:29: debian:trixie-slim is not pinned by an
    @sha256: digest". Cancelled the same way.
  - The scratch branches for M1, M4, M5 and image were deleted after their runs. `git ls-remote --heads
    origin 'scratch/*'` → empty.
  - **Run 3 ●, `pull_request` 37844861446** (head `e498a94`, D-13a-5 applied): `fast` and `test` **PASS**,
    15 min 5 s for the run. It was the first green `test`, and it proves D-13a-5.
  - **A13-M3 ●, run 37846804875** on `scratch/13a-m3` (`12aa7e5`: the `test` job's checkout at
    `fetch-depth: 1`). `fast` was green. `test` was **red**:
    - the environment report printed `--is-shallow-repository: true`;
    - `check_1_the_change_set` FAILED at `tests/acceptance/tests/ac1_composability.rs:422` with "… is a
      shallow clone: check 1 reads the transformation merges from the full history, which a shallow clone
      does not have. Fetch it all (`git fetch --unshallow`, or `fetch-depth: 0` in CI) — the check fails
      rather than skips";
    - "[ci] layer core FAILED at: cargo test --workspace".
  - **A13-M2 ●, run 37846785240** on `scratch/13a-m2` (`61792da`: `assert_eq!` → `assert_ne!` at
    `contracts/tests/action.rs:76`). `fast` was **green**. `test` was **red**:
    `a_request_survives_erasure_and_is_readable_only_as_its_own_action_type ... FAILED`, panicked at
    `contracts/tests/action.rs:76:5`, then "[ci] layer core FAILED at: cargo test --workspace".
  - The M2 and M3 branches were deleted after their runs, so every `scratch/13a-*` branch is gone.
    `git ls-remote --heads origin 'scratch/*'` → empty (2026-10-08 ≈ 21:52 UTC).
  - **Resumed session, 2026-10-08.** The previous session was cut off by an API rate limit. On resume:
    - The one uncommitted file was A-C2's ledger update, which closes its validation and review with
      the CI image evidence. It was checked against the recorded runs, found accurate, and committed
      (`bc9f430`), not discarded.
    - `origin/main` was merged at `dd8926b`. It brought S11-A (#76), E-a (#70), 12c (#67) and
      test-hygiene (#77). `docs/DECISIONS.md` conflicted: DEP-17, DEP-18 and ARC-48 against main's new
      DEP-29. Both were kept, in that order. `check_decision_ids` → 59 ids, all distinct.
  - **QTH-4, wired (`9779b06`).** `ci_layer.py`'s `fast` layer gains `check_scratch.py scan`, the declared
    `scratch-scan` check, after `check_ci_pins.py`. `core` gains `check_scratch.py left --target-dir
    target`, which runs after `cargo test --workspace` passes. ARC-48, DEP-17's runner-disk limitation
    and the standards paragraph say so.
  - Push run 37847797628 (`9779b06`): `fast` **FAIL**, a genuine integration finding.
    - `check_scratch.py scan` reported five lines, all from other crates' tests in
      `.ci/cargo/registry/src/` (autocfg, httparse, pkg-config).
    - Cause: `.ci/` is CI's `CARGO_HOME` inside the checkout (D-13a-3), and the scan walked it.
    - Fix, recorded as **D-13a-6** (`5509426`): `check_scratch.py` skips `.ci`, as it already skips
      `target` and `.git`. That is a one-line edit to a script; no Rust and no test changes. `.ci/` is
      gitignored and holds no MineWorld test.
    - Checked locally: a probe `.rs` under `.ci/cargo/registry/src/x/tests/` that calls `temp_dir()`
      leaves the scan at "142 test sources, … (1 exempt)", exit 0.
    - The PR run on the same head (37847803235) was cancelled once its `fast` was red. Its `test` job
      had not started, so it ran no tests.
  - **Run 4 ●, `pull_request` 37848072922** (head `5509426`, merge ref `2c1c658`): `fast` and `test`
    **PASS**.
    - `fast`: 1 min 42 s for the job; the layer ran 7 commands in 22.0 s. The cache was restored
      (`cargo-fast-7ad3db3d…`). `check_scratch.py scan` → "142 test sources, none makes scratch outside
      mineworld-test-support (1 exempt)".
    - `test`: 11 min 59 s for the job (21:37:57 → 21:49:56 UTC); the layer ran 3 commands in 646.9 s.
      The cache was restored from `cargo-core-7ad3db3d…`, saved before main's merge changed
      `Cargo.lock`, so dependencies were partly rebuilt.
      - `cargo test --workspace --no-run` took 152.1 s, against 282.5 s cold in run 2.
      - `cargo test --workspace` took 494.8 s. Summed over 156 `test result:` lines: **689 passed, 0
        failed, 1 ignored**. `ac1_composability` passed 13/13. `precursor_vocabulary` and
        `seam_vocabulary` passed. `kill_and_resume` printed `[cafe] PASS` and `[clock] PASS`, and
        `arrival_resolvers_resume` printed `[resolver-yard] PASS`.
      - `check_scratch.py left --target-dir target` → "no scratch left under target/tmp or as
        /tmp/mineworld-*" (QTH-4, live).
      - Disk (`/dev/root`, 145 G): 83 G free before the tests, 80 G after, 83 G after pruning. `target/`
        was 789 M (restored) before, **3.9 G at its peak** after the tests (no `target/tmp` was left),
        and 340 M after the prune. That compares with 11 G at run 2's failure, before #77.
      - A13-2 and A13-4: rustc 1.97.1, rustfmt 1.9.0, clippy 0.1.97, git 2.47.3; `false` for shallow,
        and `blob:none`.
- [x] Validation (the PR's own runs are the evidence; each recorded with run id, head SHA, wall time).
  The final head's runs are recorded in A-C4. These were the planned items:
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
  - Outcome of those items:
    - `--list`: PASS. `fast` has 7 commands and `core` has 3, equal to `standards.md`'s declared checks
      plus `check_ci_pins.py` and the QTH-4 `left` check.
    - The PR's runs: run 4 and the final head's runs (A-C4).
    - `image`: from `scratch/13a-image` (R-1).
    - M1…M5: all red at the named job.
    - nextest: **NOT RUN**, because `cargo-nextest` is not installed (`~/.cargo/bin` has none). The §5.3
      decline rests on nextest's documented custom-harness requirement, as DEP-17 cites.
- [x] Review (on `5509426`):
  - Every third-party action is pinned to a full SHA, with its version in a comment: checkout, cache,
    setup-buildx and build-push (`grep uses:`).
  - No `continue-on-error`, `|| true` or retry appears anywhere in the workflow or the composite action.
  - The workflow contains no check command. Its only `run:` lines are the composite action's `docker run
    … ci_layer.py` and prune, and `image`'s `ci_image.py`.
  - Both `docker run`s pass `--user "$(id -u):$(id -g)"`.
  - The fallback in §3.5 was taken (D-13a-5): no sparse checkout. History-reading tests pass with the
    partial clone (`ac1_composability` 13/13), and M3 shows they fail closed when it is shallow.

**Commit boundary.** Workflow, layer script and ignore line. Scratch mutations are never committed to
the PR branch.

## A-C4 — Close: status, ledger, handoff

- [x] `docs/MVP_STATUS.md` S13 row: 13a is described as awaiting review, and the "merged" wording waits
  for the merge. The row now says the repository is public, and that blocking stays policy until
  protection requiring `fast` and `test` is enabled after 13a merges.
- [x] `README.md`: the "CI" and "Run in Docker" lines link to ARC-48 and the `Dockerfile`.
- [x] Ledger. These are measured figures; they replace §10.1's estimates for the public runner:
  - Cold `test` (run 2, empty `core` cache): `cargo test --no-run` took 282.5 s. No cold run went green
    end to end; run 2 failed on D-13a-5's finding.
  - Warm `test` (run 4, cache restored before a `Cargo.lock` change): 11 min 59 s for the job, with
    152.1 s of build and 494.8 s of tests.
  - `fast`: 1 min 31 s to 1 min 42 s when warm, and 3 min 19 s cold on the private runner (run 1).
  - A PR therefore costs about 14 job-minutes per pushed head: `fast` twice (push and PR), plus `test`
    once. That is free on a public repository.
  - Disk: `target/` peaks at 3.9 G and nothing is left in `target/tmp`; free space never fell below
    80 G. R-2 is closed: no clean-up step is needed.
  - Mutations: M1…M5, all red at the named job (A-C3).
  - Deviations: D-13a-0 … D-13a-6 (A-C3, §9.5).
  - Runs of `test`-sized work, counted ●: runs 1, 2, 3 and 4, `image`, M1…M5, and the final head's
    run make **11 of 12**. Run 37847803235 was cancelled before its `test` started and is not counted.
- [x] A13-7. `git diff --stat origin/main…HEAD` (after the `dd8926b` merge) lists only `.dockerignore`,
  `.github/actions/layer/action.yml`, `.github/workflows/ci.yml`, `.gitignore`, this file,
  `.structured-coding/standards.md`, `Dockerfile`, `README.md`, `docs/DECISIONS.md`, `docs/MVP_STATUS.md`
  and `scripts/{check_ci_pins,ci_image,ci_layer,check_scratch}.py`. No `.rs` file, no `Cargo.*` and no
  test file appears.
- [x] A13-8, locally, on `9779b06`'s Rust sources, which equal the final head's and main's (A13-7). Debug
  `target/debug/mineworld run worlds/<w> --headless --seed 7 --days 300`, sha-256 of every line but
  `wall`:
  - social-cafe: exit 0, 339 lines, faults 0, `ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b`,
    17.2 s;
  - market-town: exit 0, 355 lines, faults 0, `365b50e06638795912b12304b20b0f2fc33dbbc2ba1c20ac6648261195391d1d`,
    18.1 s.

  Both equal E-RS0 and step-16's records for main, so I-S13-8 holds.
- [x] Review:
  - A13-1 … A13-4 rest on run 4 and the final head's run. A13-5 rests on the `image` run, and A13-6 on
    the final head's run, recorded below. A13-7 and A13-8 are recorded above.
  - The §7.1 adversarial results are M1…M5.
  - The nothing-material check:
    - D-13a-6 is a one-line skip in a script, inside QTH-4's wiring.
    - No setting, protection or spending was touched.
    - No Rust or test file changed.
  - Final-head evidence (the ledger commit pushed after run 4; a docs-only change, so it is also A13-6's
    cached run): recorded in the PR description and in the session report, because a run on a commit
    cannot be written into that same commit.

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

## 9.5 Handoff for 13a (continuation aid, not a design authority)

**Deviation D-13a-0, from the contract's "HANDOFF handoff.md, re-initialized for 13a".** Six lanes run at
once (`overall.md` ruling 8), and `.structured-coding/plans/mvp0/handoff.md` currently holds S15's 12b
state. If every lane re-initialized that one file, they would overwrite each other's handoffs and
conflict at every merge. 13a's handoff therefore lives here, beside its ledger, and `handoff.md` is
untouched.

```text
PROJECT / PR       MineWorld mvp0 — S13 PR 13a, the container and the per-PR CI
PRIMARY DESIGN     this file §9 (§9.0 binds); contract §9.4 as amended by §9.0 R-6
BRANCH / WORKTREE  mvp0/pr-13a-ci · /Users/yuema137/mineworld-worktrees/impl-s13a (sole writer)
BASE               main @ 47c81d1; origin/main @ e98321a merged at dd8926b (2026-10-08)
STATE              READY FOR OPERATOR REVIEW (PR #63) — DO NOT MERGE; branch protection follows the merge
FROZEN             2026-10-08, primary session (§9.0 status line)
ENDPOINTS          commits, push, PR create/update, CI repair: authorized (D-12); scratch/13a-*
                   push+delete: authorized (QS13-14, §9.0 R-4); settings, spending, Rust/test edits:
                   NOT authorized; merge: operator only
BUDGET             ≤ 12 test-sized CI runs in total (count kept in A-C3's evidence)
STOP               PR READY FOR OPERATOR REVIEW — DO NOT MERGE
```

The current checkpoint and the next actions are the first unchecked item of A-C3 / A-C4.

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
13b′  as revised by §13 (pending freeze): scripts/ci_parity.py (new) · scripts/ci_image.py (world
      enumeration) · scripts/ci_layer.py (`fast` self-test) · .github/workflows/ci.yml (image → scenario;
      mac, linux-arm, ac8) · docs/DECISIONS.md (ARC-49, DEP-19, ARC-48 note) · docs/MVP_STATUS.md (AC-8
      row) · README.md (one line) · .structured-coding/standards.md (prose) · this file. No Cargo.*, no
      tools/cli/, no Dockerfile
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

---

# 13. PR 13b — AC-8: the same world on a Mac and in a Linux container (full design)

**Lifecycle:** `DESIGN FROZEN (2026-10-08), primary session` — once the all-platforms revisions of
§13.0.1 are in, which this commit makes. Evidence is the coordinator's message relaying the primary
session's ruling: "13b is DESIGN FROZEN 2026-10-08 (primary session) once these revisions are in: add
the freeze header in the same commit."
- Written by the 13b planning session: worktree `/Users/yuema137/mineworld-worktrees/plan-13b-ci`,
  branch `plan/s13-13b`, from `main @ 9cf8f8e`.
- Implementation starts in a fresh session, under §13.12's contract.
- Frozen means scope, invariants and acceptance. Progress, evidence and bounded corrections stay
  writable.

Placeholder ids in this section: run ids `<run-…>`, PR number `#<13b>`, commit SHAs `<sha>`, and the
decision numbers `ARC-49` and `DEP-19`, which ruling 6 reserved for S13 but whose titles §13.11 QB-3
proposes to change.

**Rulings (primary session, 2026-10-08, relayed by the coordinator):**

- **Accepted as recommended:**
  - QB-2: `macos-26`;
  - QB-3: drop `sha2`, and use `DEP-19` for the runner decision;
  - QB-4: the Linux arm64 leg;
  - QB-5: 13b proceeds before 12d;
  - QB-6, QB-7 and QB-10;
  - the deviations C-1 … C-4 of §13.0.
- **QB-8:** `ac8` is not a required check.
- **QB-9:** authorized, with the same bounds as 13a's QS13-14. Each `scratch/13b-*-scenario` branch is
  deleted after its run and checked with `ls-remote`. The run budget of §13.9 is frozen.
- **QB-1 `[OM]`: yes (operator, 2026-10-08, relayed by the coordinator).** "Use the macOS runner
  (macos-26). If billing later shows a charge, switch to the designed fallback." The fallback is (a) +
  (d) + the Windows leg: the laptop at acceptance, with the Linux and Windows legs continuously (R-B4).
  Taking it removes the `mac` job, and G-3's Darwin side is then met by the laptop record at P-L alone.
  Switching is a bounded, recorded change, not a redesign.

### 13.0.1 The all-platforms requirement (operator, 2026-10-08) and the revisions it makes

The operator's words, relayed by the coordinator as a new binding requirement:

> "我们要保证支持全平台，mac linux windows都可以"
> (we must support every platform: Mac, Linux and Windows)

What it changes in 13b and 13c, as directed:

1. **A Windows leg in AC-8's record and comparison.**
   - A job `windows` on `windows-2025` runs a native release build and records. The label is pinned,
     not `windows-latest`, for the reason QB-2 pins `macos-26`.
   - The comparator groups records by (OS, architecture). Passing requires Darwin arm64 (native), Linux
     x86_64 (container) and Windows x86_64 (native) to be present and all equal. The Linux arm64
     localizer must be equal too (§13.4.2, G-3 and G-5).
2. **A plant, PM-8:** a change compiled only for Windows turns `ac8` red (§13.7).
3. **The Windows hazards, audited.** §13.10.1 lists them, each with how it is handled and the lane that
   owns it.
4. **What `fast` and `test` cover.**
   - Today `fast` and `test` run on Linux only, and stay required as they are.
   - 13b adds a job `test-windows`: `ci_layer.py core`, natively on `windows-2025`. It is **not
     required.**
   - It is expected red at first, because eight integration-test files do not compile on Windows
     (W-T1).
   - It becomes required only once it is green on `main` (QB-11 `[OM]`: the settings change is the
     operator's).
   - Cost: free on a public repository (a standard runner†); expected wall time about 20–30 min warm and
     35–45 min cold (§13.9).
5. **Tests that cannot run on Windows today** are findings with owner lanes (§13.10.1). None is skipped,
   `#[cfg]`-ed out or `#[ignore]`d by 13b (I-S13-2).

Every one of these is reflected in §§13.3–13.14 below.

### 13.0.2 Cross-lane Windows items folded in (coordinator, 2026-10-08)

**Timing.** The coordinator's message ("fold them into your Windows revision before freezing") arrived
after the freeze commit `5193540` had been pushed and its `fast` and `test` had passed. The items are
folded in here.

**They change none of 13b's frozen scope, invariants or acceptance:**
- no new 13b job, file or acceptance item;
- each item is assigned to 13w, 13c, or a client lane.

The freeze therefore stands. The primary session should confirm that reading (QB-13).

**R-S13-W1 (from S11-C/D): a Windows and macOS CI matrix for the Rust suite, and a Windows graceful-stop
helper.**
- **Windows suite.** 13b's `test-windows` is that leg (§13.0.1 item 4). It reports until 13w makes it
  green.
- **macOS suite.** A `test-macos` job (`macos-26`, `ci_layer.py core` natively, non-required) belongs in
  **13w**, not 13b. 13b's frozen job set is unchanged. Nothing is expected to fail on macOS, because the
  suite is run on the operator's Mac at every gate, so 13w adds it at little risk. Cost: free on a public
  repository (standard runner)†, about 15–20 min wall time warm, in parallel. QB-13 asks whether the
  primary session prefers it in 13b instead.
- **The graceful-stop helper** (W-12, §13.10.1) lives in **`mineworld-test-support`**
  (`tests/support/src/lib.rs`), as one function, for example `interrupt(child)`:
  - on Unix it sends SIGINT with `libc::kill`, or through the `nix` crate behind the existing support
    crate. This replaces S11-B's `sh -c "kill -INT <pid>"`;
  - on Windows it starts the child with `CREATE_NEW_PROCESS_GROUP` and sends `CTRL_BREAK_EVENT` to that
    group with `GenerateConsoleCtrlEvent` (through `windows-sys`). Ctrl-C cannot be sent to a single
    process group; Ctrl-Break can;
  - **the server must then stop on Ctrl-Break as well as Ctrl-C.** `tokio::signal::windows::ctrl_break()`
    beside `ctrl_c()` in `tools/cli/src/main.rs`'s `serve` is a production-code change. It is S11's to
    make, because S11 owns the server, together with the helper's first users;
  - the alternative, a stop that needs no signal (S11's planned admin frame `shutdown`, or stdin EOF),
    stays open to S11. If S11 chooses it, the helper wraps that instead, and no `windows-sys` dependency
    is added. The dependency decision is a DEP record in whichever PR adds it.
  - **Ownership:**
    - the helper and its Windows path go to **13w** (test-support is a test crate; 13b may not edit
      tests, I-S13-2);
    - the server's Ctrl-Break handling, or the signal-free stop, goes to **S11**;
    - S11-C's CA-13, S11-D's SD-D13 check and `hosted_town`'s interrupt switch to the helper when it
      lands, in whichever of S11 and 13w lands second.
  - They **run on Windows** in `test-windows` (13b's job), and on macOS in 13w's `test-macos`, once 13w is
    merged. 13c adds nothing for them beyond running these jobs nightly.

**R-SET-10 (from the settings design): Windows symlinks.**
- **Audit.** `git ls-files -s` shows two tracked symlinks, both pointing to `../protocol/mineworld`:
  `clients/2d/mineworld` and `clients/3d-spike/mineworld`. Without `core.symlinks=true`, which needs
  Developer Mode or administrator rights, Git for Windows checks each out as a small text file holding the
  target path. Godot then cannot load the shared module.
- **What reads them.** No test in the default suite does: `client_rules.rs` reads `scripts/*.gd`, and
  `ac13_semantic_parity.rs` reads `clients/protocol/evidence`. The `#[ignore]`d `client_2d` tests, the
  launchers and Godot do. So **13b's Windows jobs are unaffected**, and B13-11 would show it if not.
- **CI.** **13c's** `clients` Windows leg (W-9) configures symlinks, with `git config --global
  core.symlinks true` before checkout. Hosted Windows runners run as administrator, so creating symlinks
  works†. 13c verifies it by checking that `clients/2d/mineworld` is a directory link after checkout. A
  text file there is a FAIL, not INCONCLUSIVE.
- **Players and contributors on Windows: three options.**
  - **(1) Packaged clients.** A Godot export per OS bundles the module, so no symlink exists in what a
    player runs. It is the right deployment answer for players. It needs an export pipeline and release
    artifacts. Publishing binaries is a publication decision, so it is **`[OM]`** (QB-14). The pipeline
    is S12's and S14's, and CI's export job is 13c's or later.
  - **(2) Contributors from source.** Developer Mode and `git clone -c core.symlinks=true`, documented in
    the README's Windows line (a docs change by 13w or the client lanes). Zero code.
  - **(3) Replacing the symlinks with committed copies kept equal by a `fast` check, or with a sync
    step.** Declined. Two copies in Git violate ruling 4's single owner of the shared module in spirit.
    A sync step that rewrites tracked files makes a dirty tree a normal state. Godot cannot reference
    outside `res://`, so a path setting cannot replace the link either.
  - **Recommendation: (2) now and (1) for players** (QB-14 `[OM]`). CI configures symlinks (13c). The
    symlinks stay.

### 13.0.3 One definition of the native-runner layers, shared with S16's E-c and E-d (coordinator, 2026-10-08)

**The overlap.** E-c's frozen design (`step-16-packages.md` §16.12 PD-p1, landed as Ec-C7b; PR #93,
pending merge; E-d reuses it as PD-q4 / Ed-C4b) adds a `platforms` layer to `scripts/ci_layer.py`. A
`platforms` job runs that layer natively on `macos-latest` and `windows-latest`, with `test`'s triggers.
The layer is a subset of the suite:
- `cargo build -p mineworld-cli`;
- the tests of `mineworld-packages`, `mineworld-worldpack` and `mineworld-installed-systems`;
- the CLI targets `packs`, `requirements` and `third_party` (and `entity_packs`).

13b's `mac`, `windows` and `test-windows` jobs also run natively on macOS and Windows. Two parallel
definitions of "run a layer on a native runner" would drift.

**One definition, whichever PR lands first.**

1. **The layer table stays the single place for commands** (I-S13-9). 13b adds one layer, `parity`:
   - `cargo build --release --locked -p mineworld-cli`;
   - `python3 scripts/ci_parity.py record --binary target/release/mineworld[.exe] --out ac8-<os>-<arch>.txt`,
     where the default output name comes from the platform.

   The native legs `mac` and `windows` run `ci_layer.py parity`. The Linux legs keep image mode, beside
   the container. `test-windows` runs `core`. E-c's `platforms` layer stays E-c's.
2. **One composite action for native runners: `.github/actions/native/action.yml`.** It is the
   counterpart of 13a's `.github/actions/layer` (which is container-only). Its input is `layer`. It:
   - lets rustup take `rust-toolchain.toml`'s channel;
   - restores `actions/cache` for `~/.cargo/registry`, `~/.cargo/git` and `target`, keyed on `runner.os`,
     the layer, `Cargo.lock`, `rust-toolchain.toml`;
   - runs `python ci_layer.py <layer>` with the runner's Python (`python3` on macOS, `python` on
     Windows; `ci_layer.py` uses `sys.executable` within, W-7);
   - prunes `target/` before the cache saves, with `ci_layer.py --prune-cache`.

   It names a layer and never a command.
   - **If 13b lands first,** 13b creates the action and the `parity` layer. E-c's Ec-C7b and E-d's
     Ed-C4b then add only their `platforms` layer and a job that calls this action.
   - **If E-c or E-d lands first** with its own native job, 13b's B-C3 lifts that job's steps into this
     action and switches every native job to it, with E-c's job included. The command lists stay
     unchanged.

   Either way there is one action and one layer table.
3. **Runner labels are one decision, DEP-19's.** 13b pins `macos-26` and `windows-2025` (QB-2; §13.0.1),
   not `-latest`, so that a label migration never silently changes a platform under AC-8. Recommend that
   E-c's `platforms` job use the same pinned labels. That is a one-line change at Ec-C7b, or a note that
   13b applies if it lands second.
4. **`platforms` and `test-windows` are different on purpose, until 13w.**
   - `platforms` is the subset that is green on Windows and macOS today. It runs on PRs, and S16 needs it
     to show its packages work everywhere.
   - `test-windows` is the whole suite, red at W-T1 until 13w.
   - When 13w makes `test-windows` (and its `test-macos`) green, `platforms` becomes a strict subset of
     them. 13w then proposes retiring the `platforms` job, keeping the layer only if a consumer remains.
     QB-15 decides.
   - Whether `platforms` blocks a merge is the ARC-48 question E-c raised to S13. Recommendation: it may
     be made required like any green-on-`main` job, under QB-11's rule (five consecutive green `main`
     pushes, then the operator's settings change).

**Effect on the frozen design.** No acceptance criterion or invariant changes. Two details of B-C3's scope
are added, and they are bounded:
- the `parity` layer;
- the `native` composite action (or the refactor into it).

B13-7's file list gains `.github/actions/native/action.yml`. B13-6 holds: `core` is unchanged, `fast`
gains only the self-test, and adding a layer does not touch either. Recorded here, and QB-15 confirms it.

## 13.0 What this design changes in the step design, and why

The step design (§3.3, §7.2) was written while the repository was private, when a macOS runner cost ten
times a Linux minute. On 2026-10-08 the repository became public (§9.0), and 13a measured what CI really
costs (§9 A-C4: about 14 job-minutes per PR head, `test` 12–19 min). Those two facts change the best design
for AC-8. Each change below is a question in §13.11 with a recommendation; none is assumed.

| # | Step design (§3.3, §7.2) | This design | Why | Question |
| --- | --- | --- | --- | --- |
| C-1 | A reference file `ac8.ref`, recorded on the laptop and committed; each platform checks itself against it | **A live comparison.** On every run, a macOS arm64 runner and the Linux x86_64 runtime container each write a parity record of the same commit, and a third job compares the two | A committed digest of three worlds turns every PR that changes any of their facts red until someone re-records it on a Mac. Every lane (S10, S11, S15 12d, S16, S17) would carry that duty, and a contributor without a Mac could not do it. A live comparison has no reference to keep: it asks only "do the two platforms agree on this commit?", which is exactly AC-8 | QB-1 `[OM]` |
| C-2 | `ac8_parity` runs inside the required `test` check (P-2), so `test` grows by about 2–3 min | **Nothing is added to `test`.** Parity runs in the non-required `scenario` group of jobs | The operator's constraint: no required check is renamed or made slower | QB-1 |
| C-3 | `sha2` as a dev-dependency of `mineworld-cli` (`DEP-19`), the test in `tools/cli/tests/` | **No Rust change, no new crate.** A stdlib Python script (`scripts/ci_parity.py`, in the style of `ci_image.py`) runs the binary, reads the save with `sqlite3` and hashes with `hashlib` | The same script then runs on the laptop, the macOS runner and beside the container, with one hashing implementation on every side (§5.6's objection to `shasum` versus `sha256sum` does not apply). I-S13-1 holds more strictly: 13b touches no `tools/cli/` file | QB-3 |
| C-4 | 13b waits for 12d, because 12d re-baselines the towns' digests (ruling 8, QS13-12) | **13b need not wait.** With no committed reference, a re-baseline in 12d or anywhere else needs no action from 13b. After 12d merges, the `scenario` run on `main` checks the towns with bodies automatically | Removes a dependency, and a re-record, from the critical path | QB-5 (primary session) |
| C-5 | The `image` job (13a) is evidence only, and a separate `scenario` job runs P-3 | `image` is **renamed `scenario`** and grows. It keeps `ci_image.py`'s checks (A13-5) and adds the Linux leg of the parity record | One job builds the runtime image once and uses it for both | — (bounded) |

What does **not** change: AC-8's claim (§3.3, "Claim"); I-S13-1 … I-S13-9 (§4); bytes, not the FNV
fingerprint (I-S13-6); the self-reference guard (now a check on two records' platforms, §13.4.2); and
§3.7's "scenario blocks main's health, not a PR".

## 13.1 Audit anchors (main @ 9cf8f8e, 2026-10-09)

| Anchor | Finding | Evidence |
| --- | --- | --- |
| Base | `main @ 9cf8f8e` (S12 13a, #82). Since 13a's merge (`2690c1b`, #63), main gained 12d-0's plan (#84) and S12 13a (#82). 12d-0's code (SD-Z5, bodies `VERSION` 3) is on main; 12d is not frozen | `git log --oneline -15`; `step-11-bodies.md` header, §20 |
| Repository | **public**; `main` protected, required checks `["fast","test"]`, `strict: false` (branches need not be up to date) | `gh api repos/yuema137/MineWorld` → `private: false`; `…/branches/main/protection` → `required_status_checks` |
| Workflow | `.github/workflows/ci.yml`: `fast` (every push, every PR), `test` (non-draft PR, push to `main`, push to `scratch/**` except `*-image`), `image` (`workflow_dispatch`, push to `scratch/*-image`; checkout depth 1, `persist-credentials: false`; builds `--target runtime` for `linux/amd64` with the GHA cache scope `runtime`; runs `python3 scripts/ci_image.py mineworld:ci`). `permissions: contents: read`; per-ref concurrency, cancelling except on `main`. No `schedule:` trigger | `ci.yml:8–83` |
| Composite action | `.github/actions/layer`: builds the `toolchain` stage, restores `.ci/cargo` and `target`, runs `ci_layer.py <layer>` in the container as the runner's uid, prunes. Linux only (it uses `docker`) | `action.yml` |
| Layers | `ci_layer.py`: `fast` = 7 commands, `core` = `cargo test --workspace --no-run`, `cargo test --workspace`, `check_scratch.py left`. The table is the only place commands live (I-S13-9) | `scripts/ci_layer.py:37–56` |
| Image evidence | `ci_image.py`: `validate` of a **hard-coded** world list (`WORLDS = ("social-cafe", "market-town", "bodies-yard")`), a 1-day run with `faults 0`, and the default command on a named volume (create, listen on 7878, TCP accepted, `docker stop` exit 0, restart resumes). Verdicts read from output | `scripts/ci_image.py:27, 96–117` |
| Dockerfile | `runtime` = Debian trixie slim + `/usr/local/bin/mineworld` + `/opt/mineworld/worlds/`; user `mineworld`; `WORKDIR /opt/mineworld`; no Python in the runtime image. `build` = `cargo build --release --locked -p mineworld-cli` (no dependency layer caching) | `Dockerfile:13–42` |
| Worlds | `worlds/` holds `bodies-yard`, `market-town`, `social-cafe`. Only bodies-yard installs `bodies` until 12d | `ls worlds`; step-11 F-O12 |
| Run summary | `mineworld run <w> --headless --seed N --days D [--save DIR]`. The first line, `[mineworld] run …`, names the world and seats and ends `in memory` or `created <save>/world.sqlite`; the last line is `wall <s>`. `history N facts, fingerprint …` is FNV-1a ("not evidence") | `tools/cli/src/run.rs:372–508`; E-13b-0 |
| Save layout | One file, `<save>/world.sqlite`. Tables `manifest(id, format, body)`, `journal(revision PK, at, action_id, entry)`, `facts(event_id PK, revision, fact)`, `snapshots(revision PK, snapshot)`. Blobs are `serde_json` (`persistence/src/format.rs:54`). The manifest body carries `instance` (allocated from the wall clock), `pack` and `composition` | `persistence/src/sqlite.rs:46–66, 125`; `format.rs:25–33` |
| Test-side comparison | `Tables::read` and `assert_same_history` compare facts, journal and snapshots by key and blob, and the manifest's `pack` and `composition` only | `tools/cli/tests/headless/mod.rs:159–244` |
| Platform-sensitive code | No transcendental float call (`sin`, `exp`, `powf`, …) in `kernel/`, `contracts/`, `systems/*/src`, `worldpack/`, `server/`, `persistence/`. `rapier3d =0.36.0` with `enhanced-determinism` (libm, not the platform's). `read_dir` is sorted where it is used (`worldpack/src/read.rs:466–476`, `packages/src/found.rs:70–76`) | `git grep` (see §13.2) |
| Precedent ZR-4 | 12d-0: an `x86_64-apple-darwin` build run under Rosetta (`arch -x86_64`), bodies-yard 30 days: summary sha `bd6a1002…80e6`, equal to arm64's. That is the architecture changed with the OS held fixed, under emulation. AC-8 needs the OS changed too, natively | `step-11-bodies.md` §20.4 ZR-4, E-Z7 |
| 13a's measured cost | `fast` ≈ 1.5–1.7 min warm; `test` 12 min (PR, warm) to 19 min (first `main` run after 13a, cold cache); `image` 3 min 44 s with a cold 132.7 s release build; ≈ 14 job-minutes per PR head. Disk is not a constraint on the public runner (`target/` peaks at 3.9 G, ≥ 80 G free) | §9 A-C3, A-C4; `gh run view 37856137552`, `37853658192` |
| Runner images | GitHub's `runner-images` README (read 2026-10-09): `macos-14` (arm64) is **deprecated**; `macos-15` and `macos-26` are arm64 standard labels (`macos-latest` = `macos-26`); `macos-15-intel` and `macos-26-intel` are x86_64; `ubuntu-24.04-arm` is arm64 Linux. The macOS 15 and 26 arm64 images (20260907) carry Rustup 1.29.0 and Python 3.14 | `gh api repos/actions/runner-images/contents/README.md`, `…/images/macos/macos-26-arm64-Readme.md` |
| The operator's laptop | Darwin 25.2 (macOS 26), Apple Silicon; Docker Desktop installed | session environment; §2.1 |

## 13.2 Planning measurement E-13b-0

2026-10-09, this worktree at `9cf8f8e`, the operator's Mac, `cargo build --release --locked -p
mineworld-cli` (38.3 s wall with warm dependencies; 232 s user CPU). Logs in `/tmp/s13b-plan/`, not
committed.

```text
run (release)                         wall     lines  sha-256 of every line but `wall`         facts
social-cafe  300 d seed 7, memory     20.6 s   339    ad49c723…c64b  = E-RS0, = A13-8 (dev)   365 330
market-town  300 d seed 7, memory     28.3 s   355    365b50e0…1d1d  = E-RS0, = A13-8 (dev)   372 755
bodies-yard   30 d seed 7, memory      1.5 s          bd6a1002…80e6  = 12d-0's ZI-3, = ZR-4    62 385
bodies-yard  300 d seed 7, memory     14.1 s          0bf87efc…7bbb                           624 482
social-cafe   30 d seed 7, --save      2.2 s          save 249 MB                              37 085
market-town   30 d seed 7, --save      2.5 s          save 258 MB
bodies-yard   30 d seed 7, --save      2.8 s          save 246 MB
```

Findings:

- **F-13b-1.** The release profile reproduces the dev-profile digests exactly (towns: E-RS0; bodies-yard:
  ZI-3). So comparing two release builds isolates the platform. The profile is not a second variable.
- **F-13b-2.** In memory, the `[mineworld] run` header is deterministic: it names the world and its seats
  and ends `in memory`. With `--save` it ends with the save's path, which differs between hosts. A record
  therefore keeps the header in memory mode and drops it in save mode. `wall` is always dropped.
- **F-13b-3.** A 30-day save is about 250 MB, mostly snapshots. Three of them fit any runner (≈ 750 MB).
  A 300-day save is about 2.4 GB (§2.3), so long-horizon table parity is nightly work (13c, §14).
- **F-13b-4.** The record's whole workload, three 300-day runs and three 30-day saves, takes about 70 s on
  the laptop in release. On a 3-vCPU M1 runner or a 4-vCPU x86_64 runner, expect roughly 1.5–2.5× that
  (an estimate; B13-10 measures it).

## 13.3 Where the Mac side runs: options compared

AC-8: "The same World Pack runs on a laptop and inside a cloud Docker container with no semantic
differences." The Linux side is settled: the runtime image on `ubuntu-24.04`, which is native x86_64
(§3.1). The question is the Mac side. Costs are GitHub's published terms as this session knows them (†):
billing cannot be read with the session's token (§2.2), so the operator confirms every † before relying
on it.

| Option | What it proves | Continuous? | Cost | Verdict |
| --- | --- | --- | --- | --- |
| **(a) The operator's laptop against a GitHub Linux run, with an evidence artifact.** CI's `scenario` uploads the Linux record; on the laptop, `ci_parity.py record` writes the Mac record of the same commit and `ci_parity.py compare` compares them | AC-8 literally: *the* laptop and the cloud container | **No.** A one-time comparison, made by a person or session. A platform difference introduced by a later PR goes unseen until somebody repeats it | $0 | **Adopt as the acceptance evidence** (P-L), with (c) |
| (b) Committed laptop reference (§3.3 as drawn), checked on Linux in `test` | Each side equals a recorded file, so they equal each other | Linux side yes; the Mac side only when someone runs the suite on a Mac | $0 | **Decline** (§13.0 C-1, C-2): re-record churn across every lane, Mac-only re-recording, and it slows a required check |
| **(c) A GitHub macOS arm64 runner (`macos-26`) and the Linux container, compared live in one workflow run** | A Mac (Apple Silicon, `aarch64-apple-darwin`, the laptop's macOS major) against the shipped container, on every run, with no stored reference | **Yes**, on every `main` push and on demand | **Free on a public repository**: standard hosted runners, macOS included, carry no per-minute charge†. A private repository would bill macOS at 10×† (§10.1). Free-plan concurrency is about 5 macOS jobs† | **Adopt** as the continuous check (`DEP-19`, QB-1, QB-3) |
| (c′) `macos-14` (the label named in the brief) | as (c) | yes | as (c) | **Decline the label**: deprecated in `runner-images`. `macos-26` matches the laptop's macOS 26; `macos-15` is the fallback label (QB-2) |
| (d) `ubuntu-24.04-arm` (Linux arm64) as a third leg | Localizes a difference: OS (macOS against both Linux legs) or architecture (x86_64 against both arm64 legs) without Rosetta. It also covers an arm64 VPS, a plausible deployment | yes | free on a public repository† | **Adopt as a localizer leg** (QB-4). It adds about 8 job-minutes per run and no wall time, since the legs run in parallel |
| (e) `macos-15-intel` or `macos-26-intel` (Darwin x86_64) as a fourth leg | The remaining corner of the OS × architecture square | yes | free if standard† | **Decline for now.** (c) and (d) already localize, and Intel Macs are not the laptop. Revisit if a difference appears that (c) and (d) cannot place |
| (f) Rosetta on the laptop (ZR-4's method) | Architecture only, under emulation, OS fixed | no | $0 | Keep as precedent and as a local diagnostic, not as evidence |
| (g) Docker Desktop `linux/arm64` on the laptop | OS only (Linux on the same Apple Silicon) | no | $0 | Optional local diagnostic. (d) gives the same corner in CI |
| (h) A self-hosted runner on the operator's Mac | the real laptop, continuously | yes | $0 in minutes†; the machine and its uptime | **Decline.** On a public repository, a self-hosted runner can execute code from fork PRs on the operator's machine; GitHub advises against it. It would also make CI depend on a laptop being awake |
| (i) Larger macOS runners (`macos-26-xlarge`) | as (c), faster | yes | **paid, even on public repositories**† | **Decline**: not needed at about 7 min per run. Any use would be `[OM]` |
| (j) Another CI service for macOS (Cirrus, CircleCI, Codemagic) | as (c) | yes | free tiers vary† | **Decline**: a second CI account and platform for one job (§5.1's reasoning) |
| **(k) `windows-2025` (Windows x86_64), native, added by §13.0.1** | The third OS the operator requires. It is native, because GitHub's Windows runners cannot run the Linux runtime container, and Windows is a native-binary platform for MineWorld anyway | yes | free on a public repository (standard runner)† | **Adopt** (required by the operator). The label is pinned, not `windows-latest` |

**As ruled (QB-1 yes; §13.0.1): (c) + (a) + (d) + (k).** The original recommendation, which follows, is
(c) + (a) + (d); the operator's all-platforms requirement adds (k), Windows.

**Recommendation: (c) + (a) + (d).** `macos-26` and the `linux/amd64` runtime container are compared on
every `main` push and on demand (the continuous check), with `ubuntu-24.04-arm` as the localizer. At 13b's
acceptance, the operator's laptop is compared once with the final head's Linux record (literal AC-8
evidence). **Cost: $0** under the published terms for public repositories†. In steady state that is about
24 job-minutes per `main` push, about 7 of them on macOS. 13b's own validation is at most 12 runs, about
290 job-minutes (§13.9). It is `[OM]` because it reverses QS13-3's "no macOS runner" and rests on a
billing fact the session cannot read (QB-1).

## 13.4 Design

### 13.4.1 The parity record

`scripts/ci_parity.py record` writes one plain-text file per platform, one `key value` per line, UTF-8,
sorted within each section. It uses the standard library only (`subprocess`, `sqlite3`, `hashlib`,
`json`, `platform`). It runs the binary in one of two ways:

- `--binary <path>`: native, on the laptop and the macOS runner;
- `--image <tag>`: `docker run --rm --platform <p> --user <uid>:<gid> -v <host-dir>:/var/lib/mineworld/ac8
  <tag> run worlds/<w> …` beside the container, in `scenario` and `linux-arm`. The script runs on the host,
  because the runtime image has no Python. The save is written to the bind-mounted host directory and read
  there.

```text
[platform]
os               Darwin | Linux | Windows           uname -s, run where the binary runs (for --image:
                                                    `docker run --entrypoint uname <tag> -sm`); on
                                                    Windows, platform.system() (there is no uname)
arch             arm64 | x86_64                     normalized: aarch64 → arm64, AMD64 → x86_64; on
                                                    Windows, platform.machine() plus
                                                    PROCESSOR_ARCHITEW6432 to detect emulation (an
                                                    emulated x86_64 on arm64 Windows is refused, as
                                                    Rosetta is)
translated       0 | 1 | absent                     macOS only: sysctl.proc_translated (Rosetta → 1)
container        <image id> <os>/<arch> | none      docker image inspect, for --image
rustc            <rustc -vV release line>           native: the build's toolchain; image: recorded by the
                                                    workflow from the toolchain stage (1.97.1 by check_ci_pins)
[source]
commit           <git rev-parse HEAD>               the commit whose binary and worlds were run
worlds           social-cafe market-town …          every directory under worlds/ with a pack manifest,
                                                    enumerated, never listed by hand (fails closed)
[world <w>]
validate         <sha-256 of `mineworld validate worlds/<w>` stdout>
summary-300      <n lines> <sha-256>                memory mode, seed 7, 300 days: every line but `wall`
line-300 <i>     <the line>                          the summary itself, verbatim, so that a difference is
                                                    shown, not only detected (≈ 350 lines per world)
summary-30s      <n lines> <sha-256>                save mode, seed 7, 30 days: every line but the header
                                                    and `wall` (F-13b-2)
manifest         <sha-256>                          the decoded manifest body with `instance` removed,
                                                    re-encoded with sorted keys; plus the `format` column
<table> rows     <n> <sha-256>                      for journal, facts and snapshots, in primary-key order
<table> chunk <k> <first key> <last key> <sha-256>  1 000 rows a chunk (locates, ARC-23)
```

- **Row encoding.** Each row is hashed as every column, in schema order. An integer is 8 bytes,
  little-endian, signed; NULL is one `0x00` byte, and a present value is preceded by `0x01`. A blob is
  preceded by its length as 8 bytes, little-endian. That covers more than `Tables::read`, which ignores
  `facts.revision`, `journal.at` and `journal.action_id`. Parity should cover every stored byte.
- **Why 300 days in memory and 30 days saved.** The 300-day runs are E-RS0's horizon for all three worlds
  (bodies-yard's 300 days take 14 s in release, so §3.3's 30-day bodies-yard summary is lengthened at no
  real cost). The 30-day saves give the byte-level claim for every table at about 250 MB each (F-13b-3).
  QB-6 asks whether the ages are right.
- **Worlds are enumerated.** A world added under `worlds/` (Lakeside, S16) enters the record with no edit.
  A world has no per-world settings: all use seed 7 and the two ages above. A world that fails to `run`
  fails the record, naming it.
- **`ci_image.py` gets the same enumeration** in place of its hard-coded `WORLDS` (a bounded improvement
  inside the job 13b grows). The fail-open list goes, for the reason test rules §11 gives.

### 13.4.2 The comparison and its guards

`scripts/ci_parity.py compare <record> <record> [<record> …]` exits 0 only if all of the following hold.
Otherwise it exits 1 and names the first failure of each kind.

```text
G-1  well-formed      every record parses; every section and key the format requires is present
G-2  same source      every record names the same commit, and the same world list, which equals the
                      worlds/ of that commit (re-enumerated by the compare job's own checkout)
G-3  cross-platform   records are grouped by (os, arch). The set must contain all three of: Darwin
                      arm64 (translated 0 or absent, container none); Linux x86_64 (container
                      present); Windows x86_64 (native, not emulated, container none). AC-8's pair
                      is the first two; the operator's all-platforms requirement adds the third
                      (§13.0.1). A Linux arm64 record, when present, is compared too. Records from
                      fewer platforms never satisfy G-3, however equal
G-4  same toolchain   every record's rustc release is the same and equals rust-toolchain.toml's channel
G-5  equal            for every world, every key in [world <w>] is equal across all records
```

On a G-5 failure it prints, for each world that differs:

- which (OS, architecture) groups agree with which. For example:
  - `Darwin/arm64 ≠ {Linux/x86_64, Linux/arm64, Windows/x86_64}` places the difference in macOS;
  - `Windows/x86_64 ≠ {…the rest}` places it in Windows;
  - `{Linux/x86_64, Windows/x86_64} ≠ {Darwin/arm64, Linux/arm64}` places it in the architecture
    (QB-4).
- the first differing summary line, as both texts, for summaries;
- for a table: its name, its row counts, and the first differing chunk with its key range.

The compare job also writes this to the step summary (`$GITHUB_STEP_SUMMARY`), uploads every record as an
artifact, and passes or fails on the comparison only.

`ci_parity.py --self-test` checks the comparator against synthetic records built in code: equal records,
a differing line, a differing chunk, a missing world, a same-platform pair, a Rosetta-translated "Mac", a
commit mismatch and a malformed file. Each must get its stated verdict. It runs in `fast` (well under 1 s;
§13.4.3). This is UNIT ownership of a fail-closed classifier (test rules §2). It is not a test of
MineWorld.

### 13.4.3 The workflow, and its boundary with the required checks

```text
job        runs-on            trigger (13b)                                    does
scenario   ubuntu-24.04       push to main; workflow_dispatch;                 build runtime (linux/amd64, GHA cache);
           (x86_64)           push to scratch/*-scenario                       ci_image.py (A13-5, worlds enumerated);
                                                                               ci_parity.py record --image → artifact
mac        macos-26           same                                             checkout (depth 1, sparse: no clients/,
           (arm64)                                                             presentation/); rustup takes 1.97.1 from
                                                                               rust-toolchain.toml; actions/cache for
                                                                               ~/.cargo/registry and target; cargo build
                                                                               --release --locked -p mineworld-cli;
                                                                               ci_parity.py record --binary → artifact
linux-arm  ubuntu-24.04-arm   same                                             build runtime (linux/arm64);
           (arm64)                                                             ci_parity.py record --image → artifact
windows    windows-2025       same                                             checkout (LF, by .gitattributes); rustup
           (x86_64)                                                            takes 1.97.1 (x86_64-pc-windows-msvc);
                                                                               actions/cache; cargo build --release
                                                                               --locked -p mineworld-cli; ci_parity.py
                                                                               record --binary target\release\
                                                                               mineworld.exe → artifact
ac8        ubuntu-24.04       same; needs: [scenario, mac, linux-arm,          download the four artifacts;
                              windows]; if: always()                           ci_parity.py compare; step summary

test-      windows-2025       push to main; workflow_dispatch (NOT pull_       python ci_layer.py core natively (the
windows    (x86_64)           request, NOT scratch, until QB-11)               same commands as Linux `test`), with
                                                                               fetch-depth: 0, filter: blob:none;
                                                                               reports; NOT required
```

- **`test-windows`, the Windows `test`-equivalent (§13.0.1 item 4).** It runs `core`'s commands
  unchanged, so it shows exactly what the Linux `test` would show on Windows. It reports and never blocks
  a merge or main's health.
  - Its expected state at 13b's merge is red: W-T1's eight files do not compile. 13b records the red run's
    finding list (B13-11) and does not hide it.
  - When the portability PR (13w, §13.10.1) makes it green on `main`, three changes follow. That PR adds
    the `pull_request` trigger. The operator may then make it required (QB-11 `[OM]`). ARC-48's table
    gains the row.
  - Until then it does not run on PRs. A known-red check on every PR would teach reviewers to ignore red.
- **`.gitattributes`** gains `* text=auto eol=lf`, so every text file checks out with LF on Windows too
  (W-3). The binary lines stay. The one committed CRLF file
  (`clients/3d-spike/assets/characters/LICENSE.quaternius-ual.txt`, the only `i/crlf` in `git ls-files
  --eol`) gets `-text`, so it is not renormalized.
- **`ci_layer.py` becomes portable where it measures, and where it starts Python** (W-7):
  - its `disk()` uses `shutil.disk_usage` and a Python directory walk instead of `df` and `du`, which do
    not exist on Windows;
  - a layer command that starts with `python3` runs with `sys.executable`, so the same table runs on a
    Windows runner, where `python3` is not on PATH;
  - a missing environment tool is reported as before;
  - the commands of each layer are unchanged.

- **The required checks are untouched.** `fast` and `test` keep their names and their jobs. `core` keeps
  its commands, byte-identical (B13-6). The one change to `fast` is `python3 scripts/ci_parity.py
  --self-test`, appended after `check_scratch.py scan`, which takes under a second. `test`'s `if:` gains
  `!endsWith(github.ref, '-scenario')`, as it already has for `-image`, so that a scenario scratch branch
  does not spend a `test` run. None of the six new jobs (`scenario`, `mac`, `linux-arm`, `windows`, `ac8`,
  `test-windows`) runs on `pull_request` in 13b, so none can appear as a check on a PR, slow one, or be
  mistaken for a required one.
- **Check names are an interface here too.** `scenario`, `mac`, `linux-arm`, `windows`, `ac8` and
  `test-windows` use no matrix and no decorated `name:`. A later decision to require one can then name
  it: `test-windows` under QB-11. `ac8` stays non-required (QB-8).
- **`ac8` fails closed.** It runs `if: always()`, so a leg that fails or is cancelled leaves `ac8` running
  and red ("no record from mac"). It never shows as skipped, and the run never concludes `success`.
- **`image` is renamed `scenario`.** Its `scratch/*-image` route becomes `scratch/*-scenario`. ARC-48's
  table row "image … evidence only" becomes "scenario … blocks main's health" (amended by a dated note,
  not rewritten).
- **The nightly boundary.** 13b adds **no `schedule:` trigger**. 13c owns `on.schedule` for the whole
  workflow (§14). There it adds `scenario`, `mac`, `linux-arm`, `windows`, `ac8` and `test-windows` to
  the nightly set beside
  `stability` and `clients`, and adds the long-horizon parity of 300-day saves (F-13b-3). 13b's triggers
  are `main` pushes, dispatch and scratch branches only.
- **`workflow_dispatch` on the PR branch.** `ci.yml` is on `main`, so `gh workflow run ci.yml --ref
  <13b branch>` runs the branch's own `ci.yml`. That is how 13b's PR shows its jobs green before merge
  (13a could not, §9.0 R-1). Dispatch is an endpoint the contract must authorize (§13.12). It runs
  `fast` on the branch too, because `fast` has no `if:`. That costs about 1.5 min and is accepted.
- **Pins and permissions.** `actions/upload-artifact` and `actions/download-artifact` join the existing
  pins at full commit SHAs, resolved at implementation the way §9 A-C3 resolved the others.
  `actions/cache` is reused for the macOS and Windows legs. `permissions: contents: read` is unchanged
  (artifacts need no extra permission). No secret is used.
- **Timeouts.** `scenario` 30 min, `mac` 30, `linux-arm` 30, `windows` 40, `ac8` 10, `test-windows` 60.
- **Release build cost (§5.4's `cargo-chef` threshold).** 13a measured a cold release build of 132.7 s,
  far under the 8-min threshold, so `cargo-chef` is not adopted. B13-10 re-measures on all four legs.

### 13.4.4 The laptop evidence (P-L)

At B-C4, on the final head, the implementing session runs this on the operator's Mac (this host):

```text
cargo build --release --locked -p mineworld-cli
python3 scripts/ci_parity.py record --binary target/release/mineworld --out /tmp/s13b/laptop.ac8
gh run download <final-head scenario run> -n ac8-linux-x86_64 -D /tmp/s13b/
python3 scripts/ci_parity.py compare /tmp/s13b/laptop.ac8 /tmp/s13b/ac8-linux-x86_64.txt
```

The ledger records both records' `[platform]` sections, the per-world summary and table digests, and the
compare verdict. The laptop's record is not committed, because a committed record is the churn C-1
removes. The run id of the Linux artifact and the commit identify the evidence.

### 13.4.5 Non-goals

- No Rust, test, `Cargo.*`, `worlds/` or `Dockerfile` change (I-S13-1, I-S13-2). If B13-10 crosses
  §5.4's threshold, `cargo-chef` is raised as a deviation, not added.
- No `schedule:` trigger, no stability runs, no Godot (13c).
- No seated-client smoke inside the container. S11's invite tokens make it a client-driving test. It
  belongs with 13c's restart-with-reconnect run (§14), and A13-5's TCP-level check stays.
- No change to repository settings: no required `ac8` check, no protection change (QB-8 is the
  operator's).
- No re-tiering of the default suite (QS13-5 stands).

## 13.5 Invariants added for 13b

The step's I-S13-1 … I-S13-9 apply unchanged. 13b adds:

```text
I-13b-1  No required check changes name, trigger or commands, except `fast`'s appended self-test (< 1 s)
         and `test`'s `if:` exclusion of `scratch/*-scenario`; `ci_layer.py --list core` is identical to
         main's
I-13b-2  AC-8's verdict is a comparison of records of one commit, made on Darwin arm64 (not translated),
         Linux x86_64 in a container and Windows x86_64 (native, not emulated) — the operator's three
         platforms (§13.0.1). No stored reference stands in for any side
I-13b-6  No test is skipped, #[ignore]d, cfg-gated or excluded to make a Windows job green; a test that
         cannot run on Windows is a finding with an owner lane (§13.10.1)
I-13b-3  The record covers every world under worlds/ and every stored column of every save table; no hand
         list, no sampling
I-13b-4  The ac8 job's verdict comes from ci_parity.py's comparison only: a missing, cancelled or
         malformed leg is FAIL, never skipped and never PASS
I-13b-5  Planted platform differences (§13.7) live only on scratch branches, are never merged or
         cherry-picked, and every scratch branch is deleted after its run
```

## 13.6 Acceptance (decided before measuring)

```text
B13-1   scenario on the final head (a workflow_dispatch of the PR branch): every ci_image.py expectation
        holds for every enumerated world, and an ac8-linux-x86_64 record is uploaded whose platform reads
        Linux / x86_64 / container <image id> linux/amd64
B13-2   mac on the same run: record platform Darwin / arm64 / translated 0 or absent / container none;
        rustc 1.97.1
B13-2w  windows on the same run: record platform Windows / x86_64 / not emulated / container none;
        rustc 1.97.1 (host x86_64-pc-windows-msvc); the checkout's worlds/ files are LF (W-3)
B13-3   ac8 on the same run: PASS, comparing all four records (G-1 … G-5); the step summary lists the
        number of worlds (3 at this base) and of compared keys; every world's summary-300 equals main's
        known digests where they exist (social-cafe ad49c723…c64b, market-town 365b50e0…1d1d,
        bodies-yard 300 d as on main), so I-S13-8 is shown by the same run
B13-4   P-L: the laptop's record of the final head equals the final head's ac8-linux-x86_64 and
        ac8-windows-x86_64 artifacts (compare exit 0, G-3 satisfied by laptop + container + Windows)
B13-5   determinism precondition: two laptop records of one head are byte-identical files (a record
        holds no wall-clock value by construction, §13.4.1), and the Linux x86_64 records of two CI runs
        on one head are byte-identical (R1 and R2, §13.9)
B13-6   I-13b-1: ci_layer.py --list core is byte-identical to main's; --list fast is main's plus the
        self-test line; the PR's fast and test pass on the final head with their names unchanged; test's
        wall time is recorded beside main's recent runs (information, not a criterion)
B13-7   scope: git diff --stat main…HEAD lists only scripts/ci_parity.py, scripts/ci_image.py,
        scripts/ci_layer.py, .github/workflows/ci.yml, .github/actions/native/action.yml (§13.0.3),
        .gitattributes, docs/DECISIONS.md,
        docs/MVP_STATUS.md, README.md (one line), .structured-coding/standards.md (prose), and this file —
        no .rs, Cargo.*, test, worlds/ or Dockerfile; after .gitattributes, `git ls-files --eol` shows no
        i/crlf file except the one marked -text, and `git status` is clean (no renormalization)
B13-8   PM-0 … PM-8 (§13.7) all have their stated outcome
B13-9   ci_parity.py --self-test passes in fast, and fails when its comparator is mutated (PM-7)
B13-10  cost recorded from the runs: per leg wall time, cold and warm release build, record time, and
        job-minutes per run, and none exceeds its timeout
B13-11  test-windows on the final head's dispatch run: its verdict and its complete finding list are
        recorded. Expected red at W-T1 (the compile errors of the eight files named in §13.10.1, every one
        named in the log). Any failure not in §13.10.1 becomes a new W-finding with an owner lane before
        READY. Green is not required for 13b; an unexplained or unrecorded failure is
```

## 13.7 Adversarial criteria (decided before measuring)

Every planted difference is a scratch commit on a branch `scratch/13b-<name>-scenario`. It is pushed only
to run CI, never merged, and deleted after its run (I-13b-5). The PR's own code is never edited to plant
anything. A planted difference may touch MineWorld code, because a scratch commit is evidence and not part
of the PR's diff (13a's M2 edited a test the same way). Outcomes are fixed now.

```text
PM-0  local   seed sensitivity: a laptop record with seed 8 (a scratch edit of the script's seed, never
              committed) compared with seed 7's → exit 1, naming every world, its first differing
              summary line and its first differing facts chunk. Shows the instrument sees a difference.

PM-1  CI      THE PLANTED PLATFORM DIFFERENCE. A scratch commit adds, under
              #[cfg(all(target_os = "linux", target_arch = "x86_64"))], a change to what one System Pack
              records for social-cafe and market-town. The site is chosen at implementation (bounded) and
              must change a recorded fact. Expected:
                - the mac, linux-arm and windows records are equal to each other and to main's;
                - ac8 is red with G-5, naming social-cafe and market-town, the first differing summary line
                  and facts chunk, and the grouping
                  "Linux/x86_64 ≠ {Darwin/arm64, Linux/arm64, Windows/x86_64}";
                - bodies-yard is reported equal unless the pack is installed there (located, not smeared).
              Precondition, checked from the same run: the Linux x86_64 record differs from main's.
              If the change is absorbed, so that no stored byte differs, the run is INCONCLUSIVE, not a
              pass, and a larger change at the same site is planted (one re-run, budgeted).

PM-2  CI      FINER THAN THE SUMMARY. As PM-1, but the planted change alters stored bytes and NOT a summary
              line, for example one field of one fact type that no summary line counts. Expected: ac8 red
              on a table (facts, or snapshots) of the affected world with its chunk and key range, while
              summary-300 and summary-30s are equal. Shows the table digests carry the byte-level claim
              that the summary cannot (I-S13-6). If no such site exists without moving a summary line, this
              is recorded as a finding and PM-2 is N/A with that reason (not silently dropped).

PM-3  CI      THE MAC SIDE REALLY COMPARES. The planted change of PM-1 under #[cfg(target_os = "macos")]
              instead. Expected: ac8 red, grouping
              "Darwin/arm64 ≠ {Linux/x86_64, Linux/arm64, Windows/x86_64}", and the Linux and Windows
              records equal to main's.

PM-4  CI      SELF-REFERENCE. A scratch ci.yml whose mac job runs on ubuntu-24.04 (the script then records a
              Linux/x86_64 "mac" leg). Every digest is equal, yet ac8 is red by G-3, "no Darwin arm64
              record". Shows equality alone never passes AC-8.

PM-5  CI      FAIL CLOSED. A scratch ci.yml whose mac job fails before recording (an `exit 1` step after
              checkout). Expected: ac8 runs (if: always()) and is red, "no record from mac"; the workflow
              run's conclusion is failure, never success; ac8 is not reported skipped.

PM-6  local   STALE PAIRING. Compare a record of the final head with one of its parent (the laptop records
              both) → exit 1 by G-2, naming both commits.

PM-7  local   THE SELF-TEST BITES. Invert G-3 in ci_parity.py (a working-tree edit, reverted) →
              `ci_parity.py --self-test` exits non-zero naming the same-platform case; restored → passes.

PM-8  CI      THE WINDOWS SIDE REALLY COMPARES (operator requirement, §13.0.1). The planted change of
              PM-1 under #[cfg(target_os = "windows")] instead, on scratch/13b-pm8-scenario. Expected:
                - ac8 red with G-5, grouping
                  "Windows/x86_64 ≠ {Darwin/arm64, Linux/x86_64, Linux/arm64}";
                - the Darwin and Linux records equal to main's.
              Because Windows and Linux share x86_64, this is the case that proves the grouping
              separates OS from architecture. The precondition and the INCONCLUSIVE rule are as PM-1.
```

Not attempted, and why: a planted difference in float maths inside `bodies`. A one-ulp perturbation is
usually absorbed by the pack's integer millimetre verification (step-11 PB-5), so it would test the pack's
rounding, not the instrument. PM-1 and PM-2 already prove the instrument sees and locates a platform-only
change. A real float divergence is R-B1's subject.

## 13.8 Commit plan

Each commit tracks implementation, validation and review as separate items, per CLAUDE.md §3.1.

### B-C0 — Design (this section), docs only

- [x] Implementation: §13 and §14 of this file, the §7.2 and §7.3 pointers, the header line.
- [x] Validation: `python3 scripts/check_doc_headings.py` and `python3 scripts/check_decision_ids.py`
  (results in the planning commit's message and the PR description).
- [x] Self-review: every §13.1 anchor cites a file and line, a command or an API answer; E-13b-0's figures
  were measured on this head; † marks every billing figure; `[OM]` marks every operator-material question.
- [ ] Review: by the primary session (and the operator for `[OM]` items), then the freeze.

### B-C1 — Specs before code: ARC-49, DEP-19, ARC-48's note

**Goal.** How AC-8 is measured and why a macOS runner is used are reviewable records before any workflow
relies on them (`CLAUDE.md` §2.2, `REUSE_POLICY.md` §11).

**Scope.** `docs/DECISIONS.md`:

- **ARC-49** — *How AC-8 is measured.* It records:
  - the claim, the two platforms, and the live comparison (I-13b-2);
  - the record format (§13.4.1) and the guards G-1 … G-5;
  - the localizer leg;
  - the triggers, and that `ac8` blocks main's health, not a PR;
  - the laptop evidence;
  - accepted limitations: the runner is not the operator's laptop, the long horizon is 13c's, and
    `main` is checked after merge, not before.
- **DEP-19** — *AC-8's non-Linux sides: GitHub-hosted macOS arm64 (`macos-26`) and Windows x86_64
  (`windows-2025`) runners.* It covers:
  - options §13.3 (a)–(k) and the cost basis†;
  - the operator's all-platforms requirement (§13.0.1), quoted;
  - the isolating seam: `ci_parity.py` runs anywhere, and a runner label is one line;
  - the revisit triggers: a billing change (QB-1's fallback), a label's deprecation, a self-hosted need.

  This replaces §5.6's `sha2` title for the reserved number (QB-3).
- **ARC-48, a dated note**: `image` → `scenario`; the new jobs and their triggers, including
  `test-windows` as "reports" until QB-11; `fast`'s self-test; and `test`'s `-scenario` exclusion. The
  decision itself is unchanged.
- `.structured-coding/standards.md`, prose only: the `fast` layer now includes the parity self-test.

- [x] Implementation: `ARC-49`, `DEP-19` and "ARC-48 note — the scenario group of jobs and the Windows
  suite" appended to `docs/DECISIONS.md`; the `fast` bullet of `.structured-coding/standards.md` names
  the self-test. ARC-49 and DEP-19 were re-checked unused on every `origin/*` branch first.
- [x] Validation (2026-10-09, working tree on `ec38570`): `check_decision_ids` → 73 ids, all distinct
  (main 71, +2; the note's heading is not an id, as ARC-53's note is not); `check_doc_headings` → 191
  sections across 26 documents, none duplicated. ARC-23, ARC-27, ARC-30, ARC-48, DEP-17, DEP-18 resolve to
  `## ` headings in `DECISIONS.md`; I-S13-6, ZR-4, E-RS0 resolve by `git grep` to step-11/12/14.
- [x] Review: each record has problem, options, choice, why not ourselves / the others, isolating
  interface and limitations. ARC-49 also cites ARC-27 for the excluded `instance` (an addition beyond the
  listed references, bounded). No defined term is redefined. The ARC-48 note lists only what changed.

**Commit boundary.** Documentation only.

### B-C2 — `scripts/ci_parity.py` (record, compare, self-test)

**Goal.** The instrument, runnable on the laptop before any CI exists for it.

**Scope.** A new stdlib script, in the house style of `ci_image.py` and `check_ci_pins.py`: a module
docstring stating why it exists, verdicts read from output, and non-zero exits that name the cause.

- Subcommands: `record (--binary P | --image T [--platform P]) --out F`, `compare F F [F …]`,
  `--self-test`. The usage line names them.
- World enumeration from `worlds/*/` with a pack manifest. The implementation reads which file marks a
  pack from `worldpack/src/read.rs` and does not guess it.
- In image mode, the save directory is bind-mounted, the container runs with `--user` set to the host's
  uid and gid, and the directory is removed afterwards. The script leaves no scratch behind, in keeping
  with QTH-4's rule.
- **Portable from the first line** (W-1, W-4, W-7, W-10):
  - paths are built with `pathlib`;
  - the binary's argument is always the POSIX-form `worlds/<w>`;
  - platform facts come from `platform` and `os.environ`, never a Unix-only command, on Windows;
  - every `sqlite3` connection is closed, and the child process is waited for, before a save directory
    is removed;
  - a removal failure is a named verdict, not ignored;
  - the record is written with `newline="\n"`, so a Windows record has the same bytes as the others;
  - timing uses `time.monotonic`, never `/usr/bin/time`.

**Depends on:** B-C1.

- [x] Implementation: `scripts/ci_parity.py` (stdlib only: `subprocess`, `sqlite3`, `hashlib`, `json`,
  `platform`, `tomllib`, `struct`). `record`, `compare`, `--self-test` as §13.4.1–§13.4.2. Bounded
  details, each a deviation from the letter of §13.4.1, none from its meaning:
  - **D-13b-1, key spelling.** Keys hold no space, so a line splits on its first space: `line-300.0007`,
    `line-30s.0007`, `table.facts.rows`, `table.facts.chunk.00003`, `manifest <format> <sha>`. Indices are
    zero-padded, so "sorted within each section" keeps line order. `[source]` gains `record-format 1`.
  - **D-13b-2, `line-30s` kept verbatim** beside `summary-30s` (≈ 50–75 lines a world), so a save-mode
    difference is shown as text too, not only detected.
  - **D-13b-3, the image's `rustc`.** The runtime image holds no `rustc`. Image mode reads the release
    from the Dockerfile's `FROM rust:<version>-` line, which built the binary and which `check_ci_pins.py`
    holds equal to `rust-toolchain.toml`; `target` is `<uname -m>-unknown-linux-gnu`. Native mode reads
    `rustc -vV`'s `release` and `host`. G-4 compares releases.
  - **D-13b-4, tables enumerated, not listed.** Every table of the save is hashed (from `sqlite_master`),
    ordered by its primary key (`PRAGMA table_info`), so a table added later enters the record with no
    edit (I-13b-3); `journal`, `facts`, `snapshots` are required (G-1). The row encoding is §13.4.1's for
    integers, blobs and NULL; a TEXT value is encoded as a length-prefixed UTF-8 blob and a REAL as an f64,
    though no column of today's schema has either type.
  - **D-13b-5, `.exe`.** `--binary target/release/mineworld` resolves to `mineworld.exe` on Windows, so
    the `parity` layer has one command on every OS.
  - The compare job's step summary is written by the script when `GITHUB_STEP_SUMMARY` is set (it relaxes
    nothing; the verdict is the exit status).
- [x] Validation, local, on the laptop (2026-10-09, release binary of `ee1ac11`, whose Rust equals main's;
  the host was heavily loaded by other lanes, load average 77–196, so wall times are upper bounds):
  - **Two findings while validating, fixed before commit.** (1) The self-test's "differing facts chunk"
    case failed: a table was located only when its `rows` digest differed, so a chunk-only difference
    would have been reported without its chunk. Now a table is reported when any of its chunks or its
    digest differs. (2) Duplicate platform labels were numbered `#1` and then unnumbered; fixed, and the
    difference lines use the numbered labels too.
  - `--self-test` → 15 cases ok, "passed", 0.2 s user (well under a second). **PM-7**: G-3's Darwin
    check inverted → exit 1, 3 cases FAIL ("equal records of the four platforms", "a same-platform
    pair", "a Rosetta-translated Mac"); restored → passed. **PASS.**
  - Records `laptop-a` (59.4 s: bodies-yard 31.9, market-town 15.2, social-cafe 12.2) and `laptop-b`
    (57.8 s) of `ee1ac11` → `cmp` **byte-identical** (B13-5, laptop half). 1 474 lines. No
    `ac8-parity-*` scratch left in the temporary directory. **PASS.**
  - `summary-300`: social-cafe `ad49c723…c64b` (338 lines), market-town `365b50e0…1d1d` (354), both
    E-RS0's; bodies-yard `0bf87efc…7bbb` (330), E-13b-0's. **PASS.**
  - `compare laptop-a laptop-a` → exit 1: "G-3 no Linux x86_64 record from the container", "G-3 no
    Windows x86_64 record (native, not emulated)", every world equal. The self-reference guard holds on
    real data. **PASS.**
  - **PM-0**: `SEED = 8` as a working-tree edit (reverted at once; the committed file says 7) →
    `compare laptop-a laptop-seed8` exit 1, G-5 for bodies-yard, market-town and social-cafe, each with
    its first differing `summary-300` line (line 0, the header naming the seed), `summary-30s` line 0
    (e.g. social-cafe "day 1 revision 966 facts 1368" against "… 969 … 1378"), and facts chunk 0, keys
    1 … 1000; journal and snapshots located the same way. **PASS.**
  - Records `a` and `b` were made before the two compare-side fixes and D-13b-5; the record-writing code
    did not change after them (the edits touch `compare` and `Native`'s Windows suffix only).
  - PM-6 needs a record of a second commit; it is run on this commit's head (below).
  - Optional Docker comparison (option (g)): **NOT RUN**, the Docker Desktop daemon is not running.
- Earlier wording of the plan, kept:
  - `--self-test` passes; PM-7 (inverted G-3 → fails; restored → passes);
  - two `record --binary target/release/mineworld` runs of one head → byte-identical files (B13-5, laptop
    half);
  - `compare` of a record with itself → exit 1 by G-3 (one platform), the self-reference guard on real data;
  - PM-0 (seed 8) and PM-6 (parent commit) → their stated verdicts;
  - the record's `summary-300` digests for the towns equal E-RS0's values, and bodies-yard's equal
    E-13b-0's (`0bf87efc…7bbb`, or main's at implementation if it moved);
  - optional, if Docker Desktop is running: `record --image` of a local `linux/arm64` build of the runtime
    stage → `compare` with the laptop record. G-3 fails (no x86_64), but G-5's equality is reported as
    information (option (g)).
- [ ] Review: no third-party import; every subprocess's failure is a named verdict, never a traceback; the
  row encoding matches §13.4.1 exactly (lengths and NULL markers); the manifest excludes `instance` and
  nothing else; no path or host name enters a hashed line; no `|| true`-style swallowing.

**Commit boundary.** One script.

### B-C3 — The workflow: `scenario`, `mac`, `linux-arm`, `windows`, `ac8`, `test-windows`; `fast`'s self-test; `ci_image.py`'s enumeration; `.gitattributes`

**Goal.** O-2 made continuous: AC-8 is checked on every `main` push and on demand, on all three operating
systems. The Windows state of the default suite becomes visible.

**Scope.**

- `.github/workflows/ci.yml` per §13.4.3:
  - `image` renamed to `scenario`, which records after `ci_image.py`;
  - new jobs `mac`, `linux-arm`, `windows`, `ac8` and `test-windows`;
  - `test`'s `if:` excludes `-scenario`;
  - upload and download artifact actions pinned to full SHAs.
- `.github/actions/native/action.yml`, the shared native-runner action, or the refactor of E-c's job into
  it if E-c landed first (§13.0.3). `mac`, `windows` and `test-windows` call it.
- `scripts/ci_layer.py`:
  - a new layer, `parity`: the release build, then `ci_parity.py record --binary` (§13.0.3);
  - `fast` gains `["python3", "scripts/ci_parity.py", "--self-test"]`, and `core` is unchanged;
  - `disk()` is made portable (`shutil.disk_usage`, a Python size walk);
  - a layer command whose first word is `python3` runs with `sys.executable`. Windows runners have no
    `python3` on PATH (W-7). The listed commands (`--list`) are unchanged, so I-13b-1 holds.
- `scripts/ci_image.py`: `WORLDS` becomes the same enumeration as the record's. Its expectations are
  otherwise unchanged.
- `.gitattributes`: `* text=auto eol=lf`, and `-text` for the one committed CRLF file (W-3). The two binary
  lines are kept.

**Depends on:** B-C2.

- [x] Implementation:
  - `.github/workflows/ci.yml`: `image` → `scenario` (`ci_image.py`, then `ci_parity.py record --image
    mineworld:ci --platform linux/amd64`, artifact `ac8-linux-x86_64`); `linux-arm` (`ubuntu-24.04-arm`,
    runtime image `linux/arm64`, GHA cache scope `runtime-arm64`); `mac` (`macos-26`) and `windows`
    (`windows-2025`), each a depth-1 checkout without `clients/` and `presentation/` (non-cone sparse
    checkout; nothing in a `src/` or build script reads them, and `validate` already passes in the runtime
    image, which holds only `worlds/`), then the `native` action with layer `parity`, then the artifact;
    `ac8` (`needs` the four, `if: always() && <trigger>`, downloads `ac8-*`, `ci_parity.py compare
    records/*.txt`, uploads the records); `test-windows` (`windows-2025`, `fetch-depth: 0`, `blob:none`,
    the `native` action with layer `core`). Triggers as §13.4.3; timeouts 30/30/30/40/10/60. `test`'s
    `if:` gains `!endsWith(github.ref, '-scenario')`. The header comment names ARC-49 and DEP-19.
    Artifact pins resolved 2026-10-09 (`gh api repos/<r>/releases/latest` → `commits/<tag>`):
    `actions/upload-artifact` v7.0.2 `cf430e03…`, `actions/download-artifact` v8.0.2 `9000827c…`.
  - `.github/actions/native/action.yml` (new; 13b lands first, §13.0.3: no `native` action on any
    `origin/*` branch at `ec38570`): `rustup toolchain install` (no argument: rust-toolchain.toml's
    channel, profile and components) and `rustc -vV`; `actions/cache` on `~/.cargo/registry`,
    `~/.cargo/git`, `target`, keyed `native-<os>-<arch>-<layer>-<hash of Cargo.lock,
    rust-toolchain.toml>`; `python`/`python3` by `RUNNER_OS`, then `ci_layer.py <layer>`;
    `ci_layer.py --prune-cache`. `CARGO_INCREMENTAL=0` and `CARGO_PROFILE_DEV_DEBUG=line-tables-only`,
    as in the container layer.
  - `scripts/ci_layer.py`: layer `parity` (`cargo build --release --locked -p mineworld-cli`;
    `python3 scripts/ci_parity.py record --binary target/release/mineworld`); `fast` gains
    `python3 scripts/ci_parity.py --self-test` after `check_scratch.py scan`; `disk()` uses
    `shutil.disk_usage` and an `os.walk` size; a command whose first word is `python3` runs with
    `sys.executable` (`--list` unchanged).
  - `scripts/ci_image.py`: `WORLDS` → `ci_parity.enumerate_worlds(ROOT)` (one enumeration for both
    scripts); a missing World Pack is a named FAIL.
  - `.gitattributes`: `* text=auto eol=lf`; `-text` for
    `clients/3d-spike/assets/characters/LICENSE.quaternius-ual.txt`; the two binary lines kept.
  - **D-13b-6, the YAML anchor.** A YAML anchor for the shared trigger was drafted and replaced by the
    expression written out in each job, to depend on no newer workflow-parser feature.
  - **D-13b-7, `ac8`'s message for a missing leg.** A missing record reads "G-3 no Darwin arm64 record
    (…)" (or Linux, Windows), naming the platform rather than the job ("no record from mac"). No job
    name enters the script, which keeps it runnable on the laptop. PM-5 is judged on that line.
- Local evidence before the first push of B-C3 (working tree on `25ed3a0`):
  - `ci_layer.py --list core` → identical to main's (`diff` empty); `--list fast` → main's plus one line,
    `python3 scripts/ci_parity.py --self-test`, after `check_scratch.py scan`; `--list parity` → the two
    commands; an unknown layer → exit 2, naming `fast, core, parity`. **PASS** (B13-6, local half).
  - `.gitattributes`: `git add --renormalize .` staged only files already edited by this commit;
    `git ls-files --eol` → `i/crlf` only for the `-text` file (`attr/-text`). **PASS** (B13-7, W-3).
  - Both YAML files parse (`ruby -ryaml`): jobs `fast test scenario linux-arm mac windows ac8
    test-windows`, with §13.4.3's runners and timeouts.
- CI evidence log (2026-10-09). Runs are counted against the 12-run cap (§13.9); a counted run is
  marked ●. The PR's own `fast` and `test` runs are not counted.
  - **PR #97** opened after B-C2 (https://github.com/yuema137/MineWorld/pull/97). `pull_request` run
    37908150200 on `6818376`: `fast` and `test` **PASS**.
  - **R1 ● dispatch 37908152878** on `6818376`, cold (https://github.com/yuema137/MineWorld/actions/runs/37908152878):
    - `scenario` **PASS**, 5 min 16 s: `[image] every expectation held` for the three enumerated worlds;
      record 122.2 s (bodies-yard 51.0, market-town 38.1, social-cafe 33.0). Platform `Linux / x86_64 /
      container sha256:c15a851c… linux/amd64`, target `x86_64-unknown-linux-gnu`, rustc 1.97.1. **B13-1
      PASS.**
    - `mac` **PASS**, 5 min 19 s: release build 164.2 s cold, record 122.7 s. Platform `Darwin / arm64 /
      translated 0 / container none`, `aarch64-apple-darwin`, rustc 1.97.1. **B13-2 PASS.**
    - `windows` **PASS**, 9 min 33 s: release build 261.0 s cold, record 263.6 s. Platform `Windows /
      x86_64 / emulated 0 / container none`, `x86_64-pc-windows-msvc`, rustc 1.97.1. Its world sections
      are byte-identical to the others', `validate`'s output included, so the checkout's `worlds/` reads
      the same bytes on Windows (W-3); no step lists line endings separately. **B13-2w PASS.**
    - `linux-arm` **PASS**, 6 min 20 s: `Linux / arm64 / container sha256:5096b862… linux/arm64`.
    - `ac8` **PASS**, 13 s: "4 records: Darwin/arm64, Linux/arm64, Linux/x86_64, Windows/x86_64; 3 worlds;
      1458 keys compared"; every world equal on 4 records; `summary-300` social-cafe `ad49c723…c64b`,
      market-town `365b50e0…1d1d` (main's, E-RS0), bodies-yard `0bf87efc…7bbb` (E-13b-0); "AC-8 PASS".
      **B13-3 PASS**, and I-S13-8 shown by the same run.
    - `fast` PASS (1 min 3 s; it ran the self-test); `test` skipped (dispatch), as designed.
    - `test-windows` **FAIL, expected** (3 min 13 s): `cargo test --workspace --no-run` exit 101,
      "could not compile `mineworld-acceptance` (test "configuration_seam")": E0433 `std::os::unix` at
      `tests/acceptance/tests/configuration_seam.rs:31`, E0599 `status.signal()` at `:331`. Cargo stops
      at the first failing target, so the log names one file. Free disk 218.6 G of 220 G. **B13-11: see
      the finding list below.**
    - Information: the four CI records' world sections are byte-identical to the laptop's record of
      `25ed3a0` (same Rust sources), before P-L proper.
  - **B13-11 finding list (test-windows, R1).** The log names one file, because cargo stops at the first
    failing target. The complete list comes from a source audit (`git grep -l 'os::unix' -- '*.rs'` at
    `6818376`). Every hit is W-5's `ExitStatusExt` / `signal() == Some(9)` pattern:
    - §13.10.1's eight W-T1 files, unchanged;
    - **W-T1b (new): `tests/acceptance/tests/configuration_seam.rs`**, which arrived with IL-a
      (`a83b103`, 2026-10-08) after §13.10.1's audit. Owner **13w**, with W-T1.
    - **W-6 is now on `main`.** S11-B merged `tools/cli/tests/support/mod.rs:163–164`
      (`Command::new("sh")`, `kill -INT`; `8ecee01`). It compiles on Windows and would fail at run time.
      Owners **S11** and **13w** (W-12's helper), as §13.10.1 says.
    - No failure outside W-5's class was observed: compilation stopped first. Further run-time findings
      appear only once 13w makes the suite compile.
  - **D-13b-8, B13-11's "every one named in the log".** It cannot hold while cargo fails fast, and
    `core`'s commands may not change (I-13b-1). The complete list rests on the source audit above, which
    is exact for a compile error of this kind: an import that does not exist on Windows.
  - **PM-6 (local) PASS.** Records of `25ed3a0` (`laptop-c`) and of its parent `ee1ac11` (`laptop-a`)
    give exit 1 with "G-2 the records name different commits: … 25ed3a05… , … ee1ac11c…".
  - **R2 ● dispatch 37975632318** on the same head `6818376`, warm
    (https://github.com/yuema137/MineWorld/actions/runs/37975632318): `scenario`, `mac`, `linux-arm`,
    `windows`, `fast` PASS; `ac8` **PASS**; `test-windows` FAIL, expected, this time stopping at
    `tools/cli/tests/bodies_yard_restart.rs` (a W-T1 file: cargo's fail-fast order varies). All four
    records are **byte-identical to R1's** (`cmp`). **B13-5, CI half: PASS.** Warm figures: `mac`
    release build 61.1 s, record 115.7 s, job 3 min 38 s; `windows` build 74.5 s, record 431.5 s, job
    9 min 36 s; `scenario` record 134.8 s, job 2 min 44 s; `linux-arm` job 2 min 19 s; `test-windows`
    `--no-run` 244.1 s to its compile error.
  - **Planted differences** (scratch commits on `relationships`, installed in both towns and not in
    bodies-yard; or on the scratch branch's `ci.yml`). Each branch was pushed once and deleted after its
    run. Each run's four records were compared with R1's (main's Rust) as the precondition.
    - **PM-1 ● run 37975701919** (`scratch/13b-pm1-scenario`, `f0b648d`): `became-acquainted` is not
      emitted under `cfg!(all(target_os = "linux", target_arch = "x86_64"))`. `ac8` red: "G-5 world
      market-town differs: Linux/x86_64 ≠ {Darwin/arm64, Linux/arm64, Windows/x86_64}" and the same for
      social-cafe. The first differing summary line is day 1's ("facts 1383" against "facts 1473";
      social-cafe 1284 against 1368), and facts are located at chunk 0 (37 872 rows against 38 004).
      "world bodies-yard: equal on 4 records". Precondition: only the Linux x86_64 record differs from R1's;
      the other three equal it. **PASS.**
    - **PM-3 ● run 37975724745** (`a28fb84`, `cfg!(target_os = "macos")`): "Darwin/arm64 ≠ {Linux/arm64,
      Linux/x86_64, Windows/x86_64}" for both towns; bodies-yard equal; only the Darwin record differs
      from R1's. **PASS.**
    - **PM-8 ● run 37975734631** (`7080c55`, `cfg!(target_os = "windows")`): "Windows/x86_64 ≠
      {Darwin/arm64, Linux/arm64, Linux/x86_64}" for both towns; bodies-yard equal; only the Windows
      record differs from R1's. Windows and Linux share x86_64, so the grouping separates OS from
      architecture. **PASS.**
    - **PM-2 ● run 37975831040** (`923c7e6`): under the Linux x86_64 cfg, `relationships`' facts list
      their two participants in reverse order. Stored bytes change; no count does. `ac8` red, "Linux/x86_64
      ≠ {…}" for both towns. Facts are located at chunk 0 with equal row counts (38 004 and 37 085), and
      journal and snapshots are equal. Every counting summary line is equal. **But one summary line
      differs:** `history N facts, fingerprint <FNV-1a 64>`, the last line before `faults`, which
      digests every stored fact. **Finding F-13b-5:** no planted change can alter stored fact bytes
      without moving that line. The summary therefore always carries a weak (64-bit, "not evidence")
      cover of the facts. Only the table digests locate the change and carry the byte-level claim
      (I-S13-6). **PM-2 is N/A as stated** (§13.7 allows this, "recorded as a finding"). The run still
      shows the table-level location, with journal and snapshots equal. A snapshot-only plant was not
      attempted. It would spend a run to show the same table-level location on another table.
    - **PM-4 ● run 37975787323** (`725150f`): the scratch `ci.yml` runs `mac` on `ubuntu-24.04` and
      uploads its Linux record under `mac`'s artifact name. Every leg is green. `ac8` red: "G-3 no
      Darwin arm64 record (native, not translated by Rosetta, no container)", with "4 records:
      Linux/x86_64#1, Linux/arm64, Linux/x86_64#2, Windows/x86_64", every world equal on 4 records.
      Equality alone never passes. **PASS.**
    - **PM-5 ● run 37975803573** (`a7870ef`): `mac` fails at an `exit 1` step after checkout. `ac8` ran
      (not skipped) and is red: "3 records: …; G-3 no Darwin arm64 record …". The run's conclusion is
      `failure`. The message names the platform, not the job (D-13b-7). **PASS.**
    - Not 13b's: the remote also holds `scratch/ec-platforms`, `scratch/ed-platforms` and
      `scratch/s10-p3-mutation`, which belong to other lanes and were left alone. After deletion, no
      `scratch/13b-*` branch remains, locally or remotely (`git ls-remote --heads origin 'scratch/*'`,
      2026-10-09).
  - **Run count: 8 of 12** (R1, R2, PM-1, PM-2, PM-3, PM-4, PM-5, PM-8). No repair run was needed. The
    final head's dispatch makes 9. No billing sign was seen.
  - **Main merged** at the final head: `a30755e` (#96, 13w's frozen design, this file only; a clean
    merge). 13w's §15 counts the same ninth file (its F-13w-1). It attributes the file to E-c (#93);
    `git log` gives `a83b103`, "IL-a IA-C7". Either way the owner is 13w.
- [x] Validation (CI; each run's id, head, legs' wall times and verdict in the ledger). Done: R1, R2,
  PM-1 … PM-5, PM-8, deletions recorded above. The planned items follow:
  - `ci_layer.py --list core` diff against main → empty; `--list fast` → main's plus one line (B13-6);
  - `.gitattributes`: after the edit, `git add --renormalize .` stages nothing but `.gitattributes`, and
    `git ls-files --eol` shows `i/crlf` only for the `-text` file (B13-7);
  - R1: `gh workflow run ci.yml --ref <branch>`, cold → B13-1, B13-2, B13-2w, B13-3, B13-11;
  - R2: the same head dispatched again, warm → B13-5's CI half (Linux records byte-identical) and B13-10's
    warm figures;
  - PM-1 … PM-5 and PM-8 on `scratch/13b-*-scenario` branches, each deleted after its run;
    `git ls-remote --heads origin 'scratch/*'` → empty, recorded.
- [x] Review (on the B-C3 diff): every `uses:` is pinned to a full SHA with its version (a `grep`
  for pins without one is empty); no `continue-on-error`, `|| true`, retry or `pull_request_target`;
  `ac8` is `if: always()` and its only verdict step is `compare` (the records upload is `if: always()`
  too, and judges nothing); no new job lists `pull_request`; no `name:` is decorated; `permissions:
  contents: read` unchanged; the workflow's `run:` lines call `ci_image.py` and `ci_parity.py` (scripts),
  and the action calls `ci_layer.py` (layers): no check command in YAML (I-S13-9). Planned items:
  - every `uses:` pinned to a full SHA with its version in a comment;
  - no `continue-on-error`, `|| true` or retry;
  - `ac8` is `if: always()` and judges by the script only;
  - no new job has a `pull_request` trigger;
  - check names undecorated;
  - `permissions: contents: read`;
  - the workflow names layers and scripts, never commands (I-S13-9).

**Commit boundary.** The workflow, the two script edits (`ci_layer.py`, `ci_image.py`) and
`.gitattributes`.

### B-C4 — Close: laptop evidence, status, ledger, handoff

- [x] Implementation (`7cec7fd` and the closing commit): the AC-8 evidence row and the S13 row of
  `docs/MVP_STATUS.md` (13a merged; 13b awaiting review; `test-windows` red at W-T1, nine files, 13w;
  W-6, S11 and 13w); README's CI line ("AC-8 parity on every `main` push", `ARC-49`); this ledger and
  §13.14's handoff. Planned items:
  - `docs/MVP_STATUS.md`:
    - the AC-8 row, stating what is demonstrated (three worlds, 300 days and 30-day saves; macOS arm64,
      the Linux x86_64 container and Windows x86_64, plus Linux arm64), with the run id;
    - a line for the Windows default suite: `test-windows`'s state and the open W-findings with their
      owner lanes;
  - `README.md`: the CI line gains "AC-8 parity on every `main` push";
  - this section's ledger.
- [x] Validation, before the closing commit:
  - **B13-7 PASS**: `git diff --stat origin/main...HEAD` (after merging `a30755e`) lists exactly
    `.gitattributes`, `.github/actions/native/action.yml`, `.github/workflows/ci.yml`, this file,
    `.structured-coding/standards.md`, `README.md`, `docs/DECISIONS.md`, `docs/MVP_STATUS.md`,
    `scripts/ci_image.py`, `scripts/ci_layer.py` and `scripts/ci_parity.py`. There is no `.rs`,
    `Cargo.*`, test, `worlds/` or `Dockerfile` change. `git ls-files --eol` shows `i/crlf` only for
    the `-text` file.
  - The doc checks: `check_doc_headings` and `check_decision_ids` pass.
  - **Recorded outside this file, because a commit cannot hold its own run.** The final head's PR
    checks (`fast`, `test`), its dispatch run (B13-1 … B13-3 and B13-11 on the exact final head), and
    B13-4 (P-L: the laptop's record of the final head compared with that run's Linux x86_64 and Windows
    records). These go in PR #97's description and the session report.
- [x] Review:
  - B13-1, B13-2, B13-2w, B13-3: R1 and R2, then the final head's run. B13-4: P-L on the final head.
    B13-5: laptop `a` = `b`, CI R1 = R2. B13-6: `--list` diffs and the PR's `fast`/`test`. B13-7:
    above. B13-8: PM-0, PM-1, PM-3 … PM-8 PASS, and PM-2 N/A with finding F-13b-5. B13-9: the
    self-test in `fast` (every run's `fast` green), and PM-7. B13-10: the timings in R1, R2 and the
    final run, every job under its timeout (the longest, `windows` at 9 min 36 s against 40). B13-11:
    the finding list.
  - Deviations D-13b-1 … D-13b-8 are recorded where they arose. Finding F-13b-5 is PM-2's.
  - **Size.** `scripts/ci_parity.py` is about 650 lines, past the ~500-line review trigger
    (`ENGINEERING_STANDARDS.md`). It is kept as one file: one instrument with three entry points,
    recorded and reviewed together. The self-test is about 70 lines of it, and splitting would put
    the comparator's guards and their self-test in different files.
  - **Nothing material.**
    - No platform difference was found (R-B1 did not occur).
    - The binary built and ran on Windows (R-B10 did not occur).
    - No billing sign was seen, and no settings were changed.
    - No Rust, test, `Cargo.*`, `worlds/` or `Dockerfile` change is in the PR. The scratch plants
      were never merged.
    - `fast` and `test` keep their names and their triggers. `core` is unchanged; `fast` gains only
      the self-test.
    - 9 of 12 runs, counting the final head's.
  - `test-windows` red at W-T1 is NOT A STOP (§13.12), and is recorded with owners.

**Commit boundary.** Docs and ledger. A run on a commit cannot be written into that commit, so the final
head's run ids go to the PR description and the session report (as in 13a).

## 13.9 Run budget

**Revised for the Windows legs (§13.0.1).** The run cap stays **12, frozen** (QB-9 ruling). PM-8 takes
one of the three repair slots, so repairs drop from ≤ 3 to ≤ 2. Job-minutes per run grow by the
`windows` leg (≈ 12) and, on dispatch runs only, `test-windows` (≈ 20–45). Scratch pushes do not
trigger `test-windows`.

| What | Runs | Job-minutes each (estimate; B13-10 measures) | macOS minutes |
| --- | --- | --- | --- |
| R1 dispatch, cold | 1 | scenario ≈ 8, mac ≈ 8, linux-arm ≈ 9, windows ≈ 15, ac8 ≈ 1, fast ≈ 2, test-windows ≈ 45 (cold; ≈ 10 if it stops at W-T1's compile errors) → ≈ 50–90 | 8 |
| R2 dispatch, same head, warm | 1 | ≈ 50–65 | 6 |
| PM-1 … PM-5, PM-8 (scratch pushes; `fast` runs too, `test-windows` does not) | 6 | ≈ 35–40 | 6–8 |
| Repairs (bounded) | ≤ 2 | ≈ 50–65 | 6 |
| Final head dispatch | 1 | ≈ 50–65 | 6 |
| Re-run of an absorbed PM-1 (§13.7) | ≤ 1 | ≈ 35 | 6 |
| **Cap** | **≤ 12 scenario-sized runs (frozen)** | **≈ 650 job-minutes at most** | **≈ 85** |

- **Windows minutes:** free on a public repository for standard runners†. On a private repository they
  would bill at 2×†, which does not apply here.
- **Expected `test-windows` wall time.** Linux `test` takes 12–19 min (§9 A-C4). Windows builds are
  typically slower: MSVC linking of about 150 test binaries, and Defender scanning of `target/`. Estimate
  about 20–30 min warm and 35–45 min cold once the suite compiles. Its timeout is 60 min. B13-10 records
  the real figures.

- Each PR push also runs `fast` and `test`, about 14 job-minutes (13a's measurement). These do not count
  against the cap. They are not limited beyond ordinary practice.
- **Monetary: none.** Every runner above is a standard hosted runner, free on a public repository†. No
  larger runner, no spending limit, no paid service. If any evidence shows billing, for example a usage
  warning or a job refused for minutes, stop and report (§13.12).
- **Steady state after merge:**
  - per `main` push, about 35 job-minutes for the parity legs (about 6 on macOS, 12 on Windows), plus
    `test-windows`, about 25;
  - at about 4.5 merges a day, about 270 job-minutes a day. All free†;
  - wall time about 15 min for parity, because the legs run in parallel, and about 30 for
    `test-windows`.
- **Exceeding the cap** is a stop with a projection, not a silent overrun.

## 13.10 Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| **R-B1** | **A real platform difference appears.** Candidates: Rapier's floats on x86_64 (ZR-4 says equal under Rosetta, but Rosetta is not native SSE codegen); a std or libm function whose result differs between glibc and Apple's libm (none in the simulation per §13.1, but a dependency might call one); a path or locale leak into a fact | That is AC-8 working. `ac8` locates the world, table and chunk; the ledger records it; the owning pack fixes it (S15 for `bodies`). It is a **material stop** for 13b: AC-8 cannot be claimed met, and nothing is weakened to pass (I-S13-6). The localizer grouping says whether it is OS or architecture |
| **R-B2** | **The macOS runner is not the laptop** (another M-series chip, another macOS 26 build) | The triple and the macOS major match; P-L compares the actual laptop at acceptance; any later doubt is settled by repeating P-L (one command). Accepted limitation in ARC-49 |
| **R-B3** | **Runner labels move.** `macos-26` gets deprecated, or `macos-latest` moves (not used) | A pinned label, one line. DEP-19's revisit trigger. `runner-images` marks deprecations months ahead (as for `macos-14`) |
| **R-B4** | **Billing assumption wrong** (†) | QB-1 `[OM]`: the operator confirms on the billing page before freeze. If macOS is billed, fall back to (a) + (d) only: the laptop at acceptance plus Linux arm64 and x86_64 continuously, and the Mac side by hand. That is weaker and recorded as such |
| **R-B5** | **macOS queueing** (about 5 concurrent macOS jobs on Free†) delays `ac8` | Not on PRs, so it never holds a merge. A queued `main` run is late, not wrong |
| **R-B6** | **`main` is checked after merge.** A PR that introduces a platform difference merges green, and `ac8` turns red on `main` | ARC-48's "blocks main's health": the line stops until fixed or ruled. QB-7 offers a PR-level dispatch for PRs that touch simulation code |
| **R-B7** | **Records diverge for a non-semantic reason** (a host path in a summary line, a wall-clock field in a table) | B13-5's same-platform determinism precondition fails first and names it. The record drops only the header (in save mode), `wall` and the manifest's `instance`, each justified in §13.4.1. Any further exclusion is a deviation to review, never a quiet filter |
| **R-B8** | **Disk or time on the macOS runner** (three 250 MB saves, a release build) | Well within a standard image†. Measured in R1 (B13-10). Saves are removed by the script |
| **R-B9** | **Scratch branches with planted bugs are public** for the length of a run | Named `scratch/13b-*`, deleted after each run, never merged; the ledger records their deletion |
| **R-B10** | **The `mineworld` binary has never run on Windows.** It may fail to build or run there, for example on a path, a file lock or the server's Ctrl-C handling | R1 is the discovery. A defect in MineWorld code is outside 13b (no Rust edits). It is a **material stop**, reported with the failing command, the log and a proposed owner lane. AC-8's Windows side is then not met, and nothing is relaxed to pass |

### 13.10.1 Windows hazards, audited (main @ 9cf8f8e, plus S11-B's open branch)

Each hazard was audited against source. Its handling is in 13b where 13b's scope allows. Otherwise it is a
finding with an owner lane. No test is skipped, `#[ignore]`d or `cfg`-gated to hide it (I-13b-6).

| ID | Hazard | Audit | Handling | Owner |
| --- | --- | --- | --- | --- |
| **W-1** | **Path separators** | Simulation output carries no host path. `validate` prints names, not paths (E-13b-0). `run` prints a path only in save mode's header, which the record drops (F-13b-2). `read_dir` is sorted where used (§13.1). Tests build paths with `Path::join` | The record passes POSIX-form `worlds/<w>`, which Windows accepts, and builds host paths with `pathlib`. Any path that reaches a hashed line is a G-5 difference, caught and located (R-B7), never filtered | 13b (the script); any finding goes to the code's owner |
| **W-2** | **Committed goldens and evidence compared as text** | About 30 test files `include_str!` or `read_to_string` fixtures, world files or evidence (`git grep`), for example `ac13_semantic_parity.rs` reading `clients/protocol/evidence` | Fixed at the source by W-3 (LF on every checkout). If `test-windows` still shows a newline-sensitive failure, it is a W-finding for 13w | 13b (W-3); 13w otherwise |
| **W-3** | **CRLF on checkout** (`core.autocrlf=true` is Git for Windows' default, on runners too†) | `.gitattributes` has only `*.bin binary` and `*.glb binary`. `git ls-files --eol`: one committed CRLF file, `clients/3d-spike/assets/characters/LICENSE.quaternius-ual.txt`. World packs are YAML: CRLF inside a block scalar would change a string, then a fact, then AC-8's Windows digest | `* text=auto eol=lf` plus `-text` for that one file (B-C3). It renormalizes nothing (B13-7). It also helps Windows contributors, not only CI | 13b |
| **W-4** | **File locking: SQLite saves and scratch removal** | Windows cannot delete an open file. `mineworld-test-support` removes scratch with `remove_dir_all`, ignoring the error at `tests/support/src/lib.rs:151` and reporting it at `:211`. A save still open by a live `SqliteBackend` (or the -wal/-shm files of WAL mode) at drop time would leave scratch behind on Windows only | In 13b's script: connections are closed and children waited for before removal, and a failed removal is a verdict (B-C2). In the suite: `check_scratch.py left`, already in `core`, catches leftovers on Windows. Each one is a W-finding naming the test | 13b (script); 13w (tests) |
| **W-5** | **SIGKILL semantics** (Windows has no signals; `Child::kill` is `TerminateProcess`) | **W-T1:** eight test files import `std::os::unix::process::ExitStatusExt` and assert `status.signal() == Some(9)`: `persistence/tests/kill_and_resume.rs`, `tests/acceptance/tests/arrival_resolvers_resume.rs`, `tools/cli/tests/bodies_yard_restart.rs`, `market_town.rs`, `milestone_b.rs`, `milestone_c.rs`, `restart.rs`, `run_restart.rs`. **They do not compile on Windows,** so `cargo test --workspace` and `fast`'s `cargo check --all-targets` fail there | A finding, not a skip. The fix is a portable "was killed" predicate in `mineworld-test-support`: on Unix, signal 9; on Windows, `TerminateProcess`'s exit code, with the child's own "finished" line absent, which each test already checks. That edits tests, which 13b may not (I-S13-2) | **13w** (proposed: a bounded "Windows portability" PR in the S13 lane, like ruling 10's test-hygiene PR, designed after 13b) |
| **W-6** | **SIGINT and the `sh -c kill` approach** | `tools/cli/src/main.rs:495` stops the server on `tokio::signal::ctrl_c()`, which works on Windows for a console Ctrl-C. S11-B's open branch (`origin/mvp0/pr-s11b-seats`, `tools/cli/tests/support/mod.rs:158–167`) stops a child with `Command::new("sh").args(["-c", "kill -INT <pid>"])`. Windows has no `sh` or `kill`, and Windows cannot deliver Ctrl-C to one child without sharing its console | A finding for S11. A portable graceful stop is needed, for example S11's planned admin frame `shutdown` or a stop on stdin EOF, used by the test instead of a signal. 13b changes nothing in S11's tree. The Docker image keeps `STOPSIGNAL SIGINT` (Linux only, unaffected) | **S11** (before S11-B merges, or as its follow-up) |
| **W-7** | **`python3`, `df`, `du` and `/usr/bin/time` in CI scripts** | `ci_layer.py`'s layers start commands with `python3`, absent from Windows PATH†. `disk()` calls `df` and `du`. No repository script or test uses `/usr/bin/time`: it appears only in planning measurements (E-13b-0, 12d-0). `check_scratch.py` uses `tempfile.gettempdir()`, which is portable | `ci_layer.py`: `sys.executable` for `python3`, and `shutil.disk_usage` plus a size walk (B-C3). `ci_parity.py` times with `time.monotonic`. Planning-only timing commands stay out of CI | 13b |
| **W-8** | **`rustup` targets and C toolchain** | `rust-toolchain.toml` pins channel 1.97.1 (profile minimal, rustfmt and clippy). On Windows, rustup resolves it to `x86_64-pc-windows-msvc`. `rusqlite`'s bundled SQLite needs MSVC's `cl.exe`, present on `windows-2025` images†. No target triple is hard-coded anywhere | The record writes `rustc -vV`'s host. G-4 compares releases, not hosts. A build failure is R-B10 | 13b (discovery) |
| **W-9** | **Godot 4.7.2 on Windows** (13c's client checks) | The probes and launchers are bash scripts (`mineworld-2d`, `mineworld-3d`, `mineworld-slice`, `clients/protocol/run.sh`). Godot's Windows build prints to a console only in its `_console.exe` variant | 13c decides whether `clients` gains a Windows leg (the official `win64_console` binary, checksum-pinned) or stays Linux-only with a recorded reason. Windows launchers for players (a `.ps1` or a cross-platform launcher) are the client lanes' | **13c** (CI); **S12 / S14** (launchers) |
| **W-11** | **Tracked symlinks** (R-SET-10) | `clients/2d/mineworld`, `clients/3d-spike/mineworld` → `../protocol/mineworld` (`git ls-files -s`, mode 120000). They need `core.symlinks` and Developer Mode or administrator rights. No default-suite test reads them | CI: 13c's `clients` Windows leg sets `core.symlinks true`, then checks that the links resolved. Players: packaged clients (QB-14 `[OM]`). Contributors: the documented clone flag (§13.0.2) | **13c** (CI); **S12 / S14** (packaging, docs) |
| **W-12** | **A portable graceful stop for tests** (R-S13-W1) | S11-B's `sh -c kill -INT` (W-6), S11-C's CA-13, S11-D's SD-D13 and `hosted_town`'s interrupt all need a graceful server stop in a test | `mineworld-test-support::interrupt`: SIGINT on Unix, `CTRL_BREAK_EVENT` to a new process group on Windows. The server must also handle Ctrl-Break, or S11's signal-free stop is used instead (§13.0.2) | **13w** (helper); **S11** (server side, and switching its tests) |
| **W-10** | **Locale, time zone and line endings in output** | Rust's `println!` writes `\n` on every platform. Nothing in the simulation reads the locale or the time zone (only `WorldInstanceId::allocate`'s wall clock, excluded by the manifest rule). Python on Windows decodes subprocess output with universal newlines | The script reads child output as bytes, decodes it as UTF-8 and splits on `\n`. An unexpected `\r` is a G-5 difference, never stripped silently | 13b |

**Lanes.**
- **13w** is a proposed new PR. Its owner is the S13 lane, and it is designed after 13b's merge. Its scope
  is:
  - W-T1, and every W-finding `test-windows` reports;
  - W-12's `interrupt` helper;
  - a non-required `test-macos` job (§13.0.2, QB-13).
- **Its acceptance:** `test-windows` is green on `main`. It adds the job's `pull_request` trigger. QB-11
  then asks the operator to make it required.

## 13.11 Questions

`[OM]` marks an operator-material question: it costs money, changes repository settings or protection, or
reverses an operator decision. The rest the primary session can decide.

**Status (2026-10-08, §13 rulings):**

- **Decided as recommended:** QB-2, QB-3, QB-4, QB-5, QB-6, QB-7 and QB-10.
- **QB-8:** decided, as recommended; `ac8` is not required.
- **QB-9:** authorized, with QS13-14's bounds.
- **QB-1: yes**, by the operator (2026-10-08): `macos-26`, with the designed fallback if billing ever
  shows a charge.
- **New, from the operator's all-platforms requirement (§13.0.1):** QB-11 and QB-12, below. QB-11 is
  `[OM]` and does not block the freeze: it is asked only after 13w makes `test-windows` green.

| ID | Question | Recommendation |
| --- | --- | --- |
| **QB-11 [OM]** | When `test-windows` is green on `main`, make it a required check, beside `fast` and `test`? | **Yes, once green on `main` for five consecutive pushes,** after 13w adds its `pull_request` trigger. Cost: free on a public repository (standard runner)†. The price is about 20–30 min of wall time on each PR, in parallel with `test`, so the merge loop lengthens by the difference (expected ≤ 15 min). Until then it reports. Settings are the operator's. |
| **QB-12** | Who owns making the default suite pass on Windows (W-T1 and any further W-findings)? | **A new bounded PR, 13w, in the S13 lane**, designed after 13b merges, with its own freeze. It edits tests (a portable "was killed" predicate in `mineworld-test-support`), which 13b may not. W-6 goes to **S11** (its `sh -c kill`). W-9's launchers go to **S12 and S14**. |
| **QB-13** | §13.0.2 folds R-S13-W1 and R-SET-10 in after the freeze commit, assigning everything to 13w, 13c, S11, S12 and S14. Does the freeze stand? And should `test-macos` join 13b instead of 13w? | **The freeze stands:** no 13b job, file or acceptance item changes. **`test-macos` goes in 13w** with the helper, so 13b's frozen job set and run cap stay as frozen. |
| **QB-14 [OM]** | Windows players and symlinks: publish packaged clients (Godot exports per OS) as release artifacts? | **Yes, as the player path,** owned by S12 and S14 with a CI export job in 13c or later. Until then, document the clone flag and Developer Mode. Keep the symlinks; do not commit copies (§13.0.2). It is `[OM]` because it publishes binaries. |
| **QB-15** | §13.0.3: one `native` composite action and one layer table, shared with E-c and E-d; 13b adds the `parity` layer; labels pinned (`macos-26`, `windows-2025`) for E-c too; `platforms` retired in favour of `test-windows` and `test-macos` once 13w makes them green. Accept? | **Accept.** One definition however the PRs are ordered. The pinned labels keep AC-8's platforms from moving silently. Retiring `platforms` removes a duplicate subset once the whole suite is green everywhere. |
| **QB-1 [OM] — DECIDED: yes (operator, 2026-10-08)** | Adopt the live comparison on a GitHub macOS runner (§13.3 (c)), reversing QS13-3's "no macOS runner"? Confirm on the billing page that standard macOS runners are free for this public repository†. | **Yes**, with (a), the laptop at acceptance, and (d), Linux arm64 as the localizer. QS13-3's "no" rested on the 10× private-repository price, which no longer applies. If the billing page shows otherwise, take (a) + (d) only (R-B4). |
| **QB-2** | Which macOS label: `macos-26`, `macos-15` or `macos-14`? | **`macos-26`**, pinned (not `macos-latest`). It matches the laptop's macOS 26. `macos-14` is deprecated; `macos-15` is the fallback if `macos-26` misbehaves (a bounded, recorded swap). |
| **QB-3** | Drop `sha2`, and use the reserved `DEP-19` for the macOS runner instead (C-3)? | **Yes.** No Rust change; one hashing implementation on all sides. If the primary session prefers to keep `DEP-19` unused, ARC-49 can carry the runner decision. |
| **QB-4** | Include the `linux-arm` localizer leg (adds about 8 free job-minutes per run, no wall time)? | **Yes.** It localizes a difference to OS or architecture without Rosetta, and it covers an arm64 VPS. Its record must also be equal (G-5), which is a stronger claim than AC-8's pair. |
| **QB-5** | Ruling 8 sequences "13b after 12d". With no committed reference, may 13b proceed before 12d? | **Yes** (C-4). 12d's re-baseline needs nothing from 13b, and the first `main` run after 12d checks the towns with bodies. If 12d lands first, nothing changes. |
| **QB-6** | Ages and modes: 300 days in memory (summary) and 30 days saved (tables), seed 7, every world. | **Accept.** It covers E-RS0's horizon and every stored byte at about 70 s per platform. Long-horizon table parity (300-day saves, 2.4 GB each) is 13c's nightly. One seed is enough for platform parity; seed sensitivity is shown by PM-0. |
| **QB-7** | Should `scenario`/`ac8` also run on PRs (non-required)? | **Not in 13b.** Instead, a PR author may dispatch them on the branch (`gh workflow run ci.yml --ref <branch>`), which ARC-49 recommends for PRs touching `systems/`, `kernel/`, `persistence/` or `worldpack/`. Revisit if R-B6 bites. |
| **QB-8 [OM]** | Make `ac8` a required check (on `main` or on PRs)? | **No.** A required check on PRs would put a 10-min macOS leg in the merge loop, which the operator's constraint forbids, and on `main` push it cannot block anything. ARC-48's "blocks main's health" policy stands. |
| **QB-9 [OM]** | Authorize scratch branches `scratch/13b-*-scenario` (pushed only to run CI for PM-1 … PM-5, then deleted) and `workflow_dispatch` of `ci.yml` on the 13b branch and those branches? | **Authorize**, bounded as 13a's QS13-14 was: deleted after each run and checked with `ls-remote`. Without it, PM-1 … PM-5 cannot be shown in CI, and AC-8's planted-difference proof would be local only. |
| **QB-10** | `ci_image.py`'s hard-coded `WORLDS` → enumeration (a change to 13a's script). | **Accept** as in scope: it is the job 13b grows, and a hand list fails open. |

## 13.12 Execution contract (filled and frozen 2026-10-08, primary session)

```text
PROJECT / PR        MineWorld mvp0 — S13 PR 13b, AC-8 parity (layer 3)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-14-ci.md §13 (the live ledger)
RELATED / BINDING   this file §§1–6, 9 (13a as built), 10; overall.md §3 (S13), "Parallel build-out,
                    2026-10-08"; docs/DECISIONS.md ARC-23, ARC-30, ARC-48, DEP-17, DEP-18;
                    docs/ENGINEERING_STANDARDS.md §§15–16; docs/REUSE_POLICY.md §§11–12; CLAUDE.md §§2–4
IMPLEMENTATION BASE main at freeze (re-audit §13.1 if it moved); branch mvp0/pr-13b-parity;
                    worktree /Users/yuema137/mineworld-worktrees/impl-13b-ci, held by the implementing
                    session only
PRECONDITION        met: QB-1 yes (operator, 2026-10-08); QB-2 … QB-10 decided (§13 rulings, primary
                    session, 2026-10-08); the all-platforms revisions of §13.0.1 in. QB-11 is asked only
                    after 13w and does not gate 13b. DESIGN FROZEN 2026-10-08, primary session
COMMANDS            as 13a's contract (§9.4, as amended by §9.0 R-6): cargo, git, gh (PR create/update,
                    run list/view/download/cancel, workflow run per QB-9; no merge, no settings),
                    python3 scripts/*, docker (local builds and runs of the three Dockerfile stages);
                    ordinary local checks unrestricted
APPROVED SCOPE      §13.4; B-C1 … B-C4
FROZEN INVARIANTS   I-S13-1 … I-S13-9; I-13b-1 … I-13b-6
SEQUENCE            B-C1 → B-C2 → B-C3 → B-C4, each committed and pushed when coherent; the PR opens
                    after B-C2, so B-C3's dispatch runs have a PR to report to
VALIDATION BUDGET   local: unrestricted (release builds, records, Docker Desktop if running);
                    CI: ≤ 12 scenario-sized runs (§13.9), FROZEN (QB-9 ruling); each job has
                    timeout-minutes; monetary: none — standard hosted runners only; any sign of
                    billing → stop
LIVE DOCUMENTATION  §13 of this file
HANDOFF             a §13 subsection beside the ledger (as 13a's D-13a-0), not handoff.md
ENDPOINT AUTHORITY  implementation, local validation, semantic commits: authorized (2026-09-25)
                    branch push, PR create/update, CI repair: authorized (D-12)
                    workflow_dispatch of ci.yml on mvp0/pr-13b-parity and scratch/13b-*-scenario:
                      authorized (QB-9, primary session, 2026-10-08)
                    scratch/13b-*-scenario push + delete: authorized (QB-9, bounds of QS13-14: each
                      deleted after its run; `git ls-remote --heads origin 'scratch/*'` → empty recorded)
                    repository settings (protection, required checks, rulesets, Actions policy,
                      spending limit): NOT authorized — operator only
                    larger runners, paid services: NOT authorized
                    merge: explicit operator authorization only
MATERIAL STOPS      R-B1 (a real platform difference, on any of the three OSes); R-B10 (the binary fails
                    to build or run on Windows); billing evidence (R-B4: switch to QB-1's fallback is
                    pre-authorized, recorded as a deviation); any change to fast/test beyond I-13b-1;
                    making any new job required (QB-8: ac8 is not; QB-11: test-windows only by the
                    operator, after 13w); skipping, ignoring or cfg-gating any test (I-13b-6); any
                    Rust, Cargo, test, worlds/ or Dockerfile edit; any settings change; exceeding the
                    budget
NOT A STOP          test-windows red at W-T1 or at another W-finding: recorded with an owner lane
                    (B13-11, §13.10.1)
NORMAL STOP         PR 13b READY FOR OPERATOR REVIEW — DO NOT MERGE
STOP CONDITION      B13-1 … B13-11 with evidence on the exact final head; PM-0 … PM-8 recorded
MERGE AUTHORITY     never without explicit operator approval
```

## 13.13 Test ownership for 13b

```text
STATIC      fast's existing checks; ci_parity.py --self-test (UNIT: the comparator's fail-closed verdicts)
UNIT        the self-test only; no Rust unit test (no Rust changes)
INTEGRATION the unchanged default suite in `test` (13a); nothing added there
REAL RUN    scenario: the shipped image hosting a world (A13-5) and running every world for the record
CROSS-PLAT  ac8: Darwin arm64 (macos-26) × Linux x86_64 (container) × Windows x86_64 (windows-2025) ×
            Linux arm64 (container), on every main push and on dispatch; P-L: the operator's laptop ×
            the final head's Linux and Windows records, once
WINDOWS     test-windows: the unchanged default suite on Windows; reports; its findings are owned by
            13w, S11, S12 and S14 (§13.10.1)
GATE 1      NOT REQUIRED (no model)
CI          the PR's fast/test on its final head, plus the dispatch run of scenario/mac/linux-arm/
            windows/ac8/test-windows on the same head
```

## 13.14 Handoff for 13b (continuation aid, not a design authority)

As 13a's D-13a-0, the handoff lives beside the ledger, not in `handoff.md` (§13.12 HANDOFF).

```text
PROJECT / PR       MineWorld mvp0 — S13 PR 13b, AC-8 parity (layer 3)
PRIMARY DESIGN     this file §13 (frozen 2026-10-08); contract §13.12
BRANCH / WORKTREE  mvp0/pr-13b-parity · /Users/yuema137/mineworld-worktrees/impl-13b-ci (sole writer)
BASE               main @ ec38570 (#88, 13b's freeze), 2026-10-09
RE-AUDIT (§13.1)   holds at ec38570: repository public; main protected, required ["fast","test"],
                   strict false; ci.yml has fast, test, image and no schedule; .github/actions holds
                   only `layer`; worlds/ = bodies-yard, market-town, social-cafe (each with world.yaml,
                   worldpack/src/read.rs MANIFEST); save tables as §13.1; no `native` action on any
                   origin/* branch (E-c's branch mvp0/pr-ec-third-party has touched no .github file),
                   so 13b defines `.github/actions/native` (§13.0.3, "if 13b lands first")
ENDPOINTS          §13.12: commits, push, PR create/update, CI repair, dispatch on this branch and
                   scratch/13b-*-scenario, scratch push+delete: authorized; settings, spending,
                   larger runners, Rust/Cargo/test/worlds/Dockerfile edits: NOT authorized; merge:
                   operator only
INVARIANTS         I-S13-1 … I-S13-9; I-13b-1 … I-13b-6
BUDGET             ≤ 12 scenario-sized CI runs (count kept in B-C3's evidence)
STOPS              §13.12 MATERIAL STOPS (R-B1, R-B10, billing, fast/test change, settings, budget)
STOP               PR READY FOR OPERATOR REVIEW — DO NOT MERGE
```

The current checkpoint and the next actions are the first unchecked item of B-C1 … B-C4.

**State at the closing commit (2026-10-09):**
- B-C1 … B-C4 are done; PR #97 is open.
- 8 of 12 runs are used; the final head's dispatch makes 9.
- Every scratch branch is deleted.
- The remaining steps run on the final head: the dispatch run and P-L, recorded in the PR
  description.
- Then READY FOR OPERATOR REVIEW — DO NOT MERGE. The implementation context is then CLOSED /
  AWAITING OPERATOR ACTION.
- **Main moved after the closing commit.** `f80bbb7` (#94, S19 TW-a) adds the `calendar` System Pack,
  installed in market-town, with solar positions in floats (`solar-positioning` with `libm`, DEP-30),
  and re-baselines market-town (TW-a's ledger: "market-town baseline 24a95d2a"). It is merged into
  this branch. The only conflict was `docs/DECISIONS.md`, where both sides appended; both were kept,
  13b's records first, and `check_decision_ids` gives 75 ids, all distinct. A re-run of the
  `os::unix` audit finds the same nine files and W-6, nothing new. The final head's dispatch therefore
  also checks the calendar's float path across the four platforms. The `365b50e0…1d1d` market-town
  digest cited above is main's before TW-a.
  - Run 10 ● (dispatch 37980642471, on `58f659c`): every leg passed and `ac8` PASSED, with 4 records and
    1 465 keys. market-town gained 7 keys from the calendar, and its `summary-300` is
    `24a95d2a…d270`, TW-a's baseline, equal on all four platforms. P-L on `58f659c` PASSED as well.
- **Main moved again.** `0744fee` (#104, S11-D) was merged cleanly. `docs/DECISIONS.md` merged
  without a conflict: 76 ids, all distinct.
  - The `os::unix` audit finds the same nine files and W-6, at `tools/cli/tests/support/mod.rs:187–188`.
  - It also finds `tools/cli/tests/admin.rs:557 #[cfg(unix)]`, which is S11-D's SD-D13 gate. 13w's
    QW-3 already owns its removal, so it is a known W-item and not a new finding. Its owners are S11
    and 13w.
  - Run 11 ● (dispatch 37982741312, on `0e07f42`): every leg passed and `ac8` PASSED, with 4 records
    and 1 465 keys. P-L on `0e07f42` PASSED. `test-windows` stopped at `bodies_yard_restart`. The PR
    run 37982719791 had `fast` and `test` green.
- **Main moved again, and this time it conflicted.** PR #97 became `CONFLICTING`, because #98
  (S10-P3, `7a0ec69`/`6b3434a`) changed the same CI files: a `python` matrix job on Linux, Windows and
  macOS; `uv` in the toolchain image; `python` and `python-smoke` layers; Python static checks appended
  to `fast`; and `COMMAND_ENVIRONMENT` in `ci_layer.py`. Main (`f867b25`) was merged, and the conflicts
  were resolved by keeping both sides:
  - `ci_layer.py`: one command runs with `resolved(command)` and main's environment. The usage lines
    list both sets of layers.
  - `standards.md`: the `fast` bullet names the self-test and the Python checks.
  - `DECISIONS.md`: 13b's records, then main's. 80 ids, all distinct.
  - `ci.yml`: main's `python` job, then 13b's `scenario` group.
  - **D-13b-9.** The `python` job's scratch exclusion gains `!endsWith(github.ref, '-scenario')`
    beside its `-image`, as `test`'s did. This PR renamed the `-image` route, so without it a
    `scratch/*-scenario` push would also run the Python matrix. It changes no trigger of a required
    check.
  - **§13.0.3 is not applied to the `python` job**, recorded here as a follow-up. That job installs
    `uv` and Python on the runner and runs `uv run --locked python scripts/ci_layer.py`, so it is not a
    plain layer-on-a-native-runner job. Moving it onto `.github/actions/native` changes S10's job and
    belongs to S10 or 13w, not to this merge.
  - Checks after the merge:
    - `ci_layer.py --list core` is identical to main's; `--list fast` is main's plus the self-test line
      (B13-6).
    - `check_ci_pins` passes, now including the copied uv image.
    - The self-test passes.
    - The `os::unix` audit finds the same nine files plus the known W-6 and SD-D13 items, and nothing
      new.
  - This is the last merge of main before review. Protection is `strict: false`, so later movements
    of main are recorded rather than chased, unless they conflict. Run 12 is the final head's dispatch,
    and the cap is then reached.
- **Run 12 ● (dispatch 38006525123, on `0df0be4`, the cap reached).** `scenario`, `mac`, `linux-arm`,
  `windows` PASS; `ac8` **PASS** (4 records, 1 465 keys; market-town `24a95d2a…d270`); `test-windows`
  red at W-T1 (`bodies_yard_restart`). P-L on `0df0be4` **PASS** (laptop + Linux x86_64 + Windows, exit
  0). PR run 38006501836: `fast` and `test` **PASS**. The `python` matrix of S10-P3 FAILED on all three
  platforms at `sdk/python/tests/test_golden_frames.py::test_every_golden_frame_has_a_model`
  ("golden frames with no Python model: ['clock.json']"). Main fails identically (push run
  38006324721 on `f867b25`): S11-D's new `server/tests/frames/clock.json` meets S10-P3's golden-frame
  check. **Not 13b's; owners S10 and S11.** `python` is not a required check.
- **MATERIAL STOP: the run cap (§13.12, "exceeding the budget").** After run 12:
  - main moved again (`aee8290`, #102/#103), and PR #97 now conflicts in `docs/DECISIONS.md` only
    (both sides append records);
  - the coordinator directs that 13b build on E-d's `.github/actions/native` and `platforms` layer once
    E-d (#101, open) merges. E-d's action has the same `layer` input; it differs in having no explicit
    `rustup toolchain install`, a cache key without the architecture, and a line-endings report step.
    The integration is to take E-d's file, drop 13b's, and keep `parity` beside `platforms` in
    `ci_layer.py`.

  Either change makes a new head, and the exact-head rule then needs one more dispatch: run 13, beyond
  the frozen cap of 12 (QB-9). **Proposed to the operator:** authorize one extra dispatch run (about
  30–60 job-minutes on standard runners, free on a public repository) after E-d merges. Then merge
  main, integrate E-d's action, dispatch once, repeat P-L, and mark READY. Until then the branch stays
  at `0df0be4`, whose evidence is complete.
- **Rulings after the stop (primary session, relayed by the coordinator, 2026-10-09):**
  - **Run 13 authorized.** The cap rises to 13 for one purpose: integrating E-d's action and main.
  - **The plan is accepted.** Once E-d (#101) merges:
    - merge main;
    - take E-d's `.github/actions/native` unchanged. Parity needs no explicit `rustup toolchain
      install` (the layer's `cargo build` installs the pinned toolchain first) and no architecture in
      the cache key (one architecture per OS here);
    - keep `parity` beside `platforms` in `ci_layer.py`, and keep 13b's portable `disk()`;
    - pin E-d's `platforms` job to `macos-26` / `windows-2025` (§13.0.3) and add `-scenario` to its
      scratch exclusion;
    - dispatch run 13, repeat P-L, and mark the PR READY.
  - **CRLF.** 13b's `.gitattributes`, with LF checkouts (W-3), stays. CRLF coverage comes from tests
    that write CRLF themselves (E-d's ED-13, IL-b's IB-10, TW-d's parser tests), not from Windows
    checkouts. E-d's "Report the checkout's line endings" step therefore shows 0 carriage returns
    **by design**, and that is recorded here.
- **E-d merged (`68176e7`, #101); main merged into this branch** (it also brought #105, #110 and #112,
  the last fixing S10-P3's `clock.json` frame). The conflicts were resolved as the plan said:
  - `.github/actions/native/action.yml` (add/add): **E-d's file taken unchanged**; 13b's version is
    dropped. Nothing was added to it. Parity needs no explicit toolchain step, because the layer's
    first command, `cargo build --release`, makes rustup install `rust-toolchain.toml`'s channel. It
    needs no architecture in the cache key: each OS here has one architecture, and the layer name
    separates `parity` from `core` and `platforms`.
  - `scripts/ci_layer.py`: `parity` beside `platforms` in the table, both usage lines kept. 13b's
    portable `disk()` (`shutil.disk_usage` and a walk) replaces E-d's `df`/`du` with its
    `FileNotFoundError` guard; it prints the same information on every OS.
  - `docs/DECISIONS.md`: 13b's records, then E-d's (ARC-71 …). 86 ids, all distinct.
  - `ci.yml`: E-d's `platforms` job is **pinned to `macos-26` / `windows-2025`** (§13.0.3, ruled), and
    its scratch exclusion gains `-scenario`. Its comment now says the line-endings report reads 0 by
    design (the CRLF ruling above).
- **Checks after the merge:**
  - `ci_layer.py --list core` and `--list platforms` are identical to main's; `--list fast` is main's
    plus the self-test.
  - `check_ci_pins`, `check_decision_ids`, `check_doc_headings` and the self-test pass.
  - Four jobs use the shared `native` action: `platforms`, `mac`, `windows`, `test-windows`.
  - The `os::unix` audit adds one hit: `worldpack/src/configure/tests.rs:655`, a `#[cfg(unix)]`
    symlink-escape test from IL-a (`73c478b`). It compiles on Windows and is gated out there, so it is
    a W-item for **13w** to review (whether a Windows equivalent is needed); 13b does not touch it.
- **E-c** will add `third_party` and `package_sources` to `platforms`. Whichever of E-c and 13b lands
  second reconciles the table (coordinator, 2026-10-09).
- **Post-merge synchronization:** this session's PR section only. `overall.md` and the step header
  belong to the planning session.

---

# 14. PR 13c — outline: nightly long runs and the Godot probes (medium scope)

**Lifecycle:** outline, not designed to the commit. It refines §7.3 against 13a and 13b as built and as
designed. It is detailed to the commit by its own planning session, after 13b's freeze.

## 14.1 What 13c owns

- **The `schedule:` trigger, for the whole workflow.** It is a single nightly cron, for example 03:17 UTC,
  off the hour, to avoid GitHub's top-of-hour congestion. A first job, `changed`, compares `main`'s SHA
  with the last successful nightly's (`gh api` over the workflow's runs, read-only, or the
  `actions/cache` marker pattern) and skips the night when `main` has not moved. A `workflow_dispatch`
  input `force` overrides that.
- **The nightly set:**
  - 13b's `scenario`, `mac`, `linux-arm`, `windows`, `ac8` and `test-windows`, unchanged;
  - `stability`;
  - `parity-long`;
  - `clients`.
- **What a red nightly does.** A red nightly reports and never gates a PR (ARC-48: `stability` and
  `clients` report; `scenario`/`ac8` keep "blocks main's health"). It opens or updates one issue labelled
  `nightly`, which needs `issues: write` on that job only (I-S13-7 is relaxed for one job; a question for
  the 13c design). The alternative is a step summary that a session reads. That choice is a 13c question.

## 14.2 Jobs

| Job | Runs | Verdict from |
| --- | --- | --- |
| `stability` (layer 4) | In the toolchain container, from `scripts/ci_layer.py stability`, a new layer. **(1)** social-cafe 1 000 days `--save`, then `mineworld replay`, byte-for-byte reconstruction (§16 "replay event log"). **(2)** market-town 300 days SIGKILLed at five points and resumed, compared with an uninterrupted run (tables equal). **(3)** bodies-yard 300 days `--save` with the bodies scan. **(4)** `mineworld server` killed and restarted ten times on one volume while a scripted client reconnects with its invite (S11's credentials), each restart resuming at the same revision. Scratch removed (QTH-4) | each program's own verdict lines; a missing verdict is INCONCLUSIVE, not PASS |
| `parity-long` | 13b's `ci_parity.py` with a `--long` profile: 300-day saves (2.4 GB each) for table parity at E-RS0's horizon, and 1 000-day summaries, on the same four legs (Darwin arm64, Linux x86_64 container, Windows x86_64, Linux arm64) plus `ac8`'s comparison | `ci_parity.py compare` |
| `clients` | Godot 4.7.2 Linux x86_64 headless, the official binary, downloaded once, pinned by its published SHA-512, cached (§5.7). **(1)** `cargo test -p mineworld-cli --test client_2d -- --ignored --test-threads=1`: the 2D client's eight `#[ignore]`d tests, which already exit non-zero on failure (`clients/2d/scripts/harness/drive.gd:464`). **(2)** `clients/protocol/run.sh evidence` then `git diff --exit-code clients/protocol/evidence`; the protocol checks exit non-zero on failure (`affordances_check.gd:220`, `reader_check.gd:40`). **(3)** `./mineworld-slice --drive`, whose probe still exits 0 unconditionally (`slice_probe.gd:160`; F-5 open for the 3D slice only), so its verdict line is parsed and a missing one is INCONCLUSIVE | exit status where the probe has one; a parsed verdict line otherwise |
| `benchmark` (report) | Wall time per world-day from `stability`'s and `parity-long`'s runs, as a step summary and an artifact; a trend over the last 30 nights from the artifacts (QS13-13: a report, not a crate) | information only |

## 14.3 Adversarial outline (to be fixed before measuring in 13c's design)

- One altered fact row in a scratch copy of a save → `replay` refuses it, and `stability` is red.
- A probe made to fail (a missing scene) → `clients` FAIL for the 2D and protocol checks. For the 3D slice
  it is INCONCLUSIVE or FAIL by the parsed line, never PASS.
- The `changed` gate: a dispatch with `force=false` on an unmoved `main` skips. With `force=true` it runs.
- A nightly with a planted platform difference (13b's PM-1 on a scratch branch, dispatched with the
  nightly inputs) → `parity-long` red, located.

## 14.4 Dependencies and boundary with 13b

- 13c depends on 13a (merged). It reuses 13b's `ci_parity.py` and jobs, so it follows 13b's merge.
  Otherwise only `stability` and `clients` can land first.
- **Boundary.** 13b never adds `schedule:`, `stability`, `parity-long` or `clients`. 13c never changes
  `fast`, `test`, or 13b's record format beyond adding the `--long` profile, and never makes a nightly job
  a required check.
- The 3D slice's exit status (F-5) belongs to S14. 13c asks S14 for `quit(1)` on failure, and until then
  parses the verdict line.
- Budget, estimated: `stability` 30–60 min, `parity-long` about 20–30 min per leg (2.4 GB saves; disk to be
  measured on the macOS runner†), `clients` 10–15 min. At most one night a day, skipped when `main` is
  unmoved. All on standard runners, free on a public repository†.

## 14.5 Questions to carry into 13c's design

- `issues: write` for the nightly reporter, or a step summary only?
- Whether `parity-long` includes the macOS leg every night, or weekly (macOS concurrency†).
- **All platforms (operator, 2026-10-08, §13.0.1):**
  - The `stability` runs on Windows depend on 13w, because the SIGKILL and resume programs must compile
    there first (W-5). Until then they are Linux-only, with that reason recorded, never silently.
  - Whether `clients` gains a Windows leg for Godot 4.7.2: the official `win64_console` binary,
    checksum-pinned, running the 2D client's `#[ignore]`d tests and the protocol checks. The bash
    launchers (`mineworld-slice`, `run.sh`) need a Windows equivalent from S12 and S14 first (W-9).
    Recommend: yes, once those exist; until then a recorded Linux-only reason.
  - A macOS leg for `clients` too, so that the 2D client is checked on the three OSes the operator named.
- Whether the 1 000-day social-cafe replay replaces or joins the default suite's 300-day runs (QS13-5
  stands: no re-tiering without a fail-closed selector).

---

# 15. PR 13w — the default suite on Windows and macOS (full design)

**Lifecycle:** `DESIGN FROZEN (2026-10-09), primary session`. Superseded: `PR design — ready for freeze
review`.
- Evidence: the coordinator's message relaying the primary session's rulings: "13w (§15) is DESIGN FROZEN
  2026-10-09 (primary session)".
- A fresh session implements it under §15.12, after 13b merges.

**Rulings (primary session, 2026-10-09). QW-1 … QW-5 accepted as recommended:**
- **QW-1:** relayed to S11-D, which adds `signal::windows::ctrl_break()` to SD-D13. 13w's one-line
  fallback (§15.4, W-C3) stays in the contract, for the case where S11-D merges without it.
- **QW-2:** our own `extern "system"` declaration, not `windows-sys`.
- **QW-3:** whichever of S11 and 13w lands second removes the `#[cfg(unix)]` gates.
- **QW-4:** E-c's `platforms` job is retired once `test-windows` and `test-macos` are green.
- **QW-5:** non-required `pull_request` triggers on both jobs.
- **Still open:** QB-11 `[OM]`, making the jobs required, stays with the operator.
- Written by the 13w planning session: worktree `/Users/yuema137/mineworld-worktrees/plan-13w`, branch
  `plan/s13-13w`, from `main @ ec38570`, after #88, #83 (S11-B) and #93 (E-c/E-d plan) merged.
- Placeholder ids: `#<13w>`, `<run-…>`, `<sha>`, decision note `DEP-29 note (13w)`.
- 13w was proposed in §13.10.1 and assigned by QB-12 and QB-13. It implements the operator's
  all-platforms requirement (§13.0.1) for the default suite.

## 15.1 Scope and material boundary

**Goal.** `cargo test --workspace` compiles and passes on Windows x86_64 and macOS arm64 in CI, so that
`test-windows` (13b) and a new `test-macos` are green on `main` and can become required (QB-11 `[OM]`).
No test may be skipped, `#[ignore]`d, `cfg`-gated away or excluded to get there (I-13b-6).

**Material boundary (coordinator, 2026-10-09).**
- 13w edits only test code and `mineworld-test-support`.
- One exception may be needed: the server's Ctrl-Break handling (§15.4). It is S11's code, and §15.4
  routes it to S11-D first.
- A failure on Windows or macOS whose cause is in production code (kernel, systems, persistence, server,
  CLI `src/`) is a **material stop**, reported with an owner lane. 13w never fixes it.

**Depends on:** 13b merged. 13b provides `test-windows`, the `native` composite action and
`ci_layer.py`'s portability (§13.0.3, §13.10.1 W-7). It must also land after S11-D's SD-D13 (§15.4), or
take the §15.4 fallback.

## 15.2 Audit (main @ ec38570)

| Anchor | Finding | Evidence |
| --- | --- | --- |
| Unix-only imports | **Nine** test files import `std::os::unix::process::ExitStatusExt`. 13b's audit counted eight; `tests/acceptance/tests/configuration_seam.rs:31`, added by E-c (#93), is the ninth (**F-13w-1**). The files: `persistence/tests/kill_and_resume.rs:39,533`, `tests/acceptance/tests/arrival_resolvers_resume.rs:38,235`, `tests/acceptance/tests/configuration_seam.rs:31,331`, and in `tools/cli/tests/`: `bodies_yard_restart.rs:23,64`, `market_town.rs:32,75`, `milestone_b.rs:38,86,333`, `milestone_c.rs:31,397`, `restart.rs:32,113`, `run_restart.rs:27,73` | `git grep -n "ExitStatusExt\|\.signal()"` |
| Kill patterns | Two shapes. (a) `child.kill()`, then `status.signal() == Some(9)` (`kill_and_resume`, `arrival_resolvers_resume`, `configuration_seam`). (b) `assert_eq!(status.signal(), Some(9), "killed by SIGKILL, not finished")` after the CLI test support's `Server::kill()` or a direct `child.kill()` (the rest). Every killing test also checks, separately, that the child had not finished (no final summary line, or a revision below the end) | the cited lines |
| Graceful stop | `tools/cli/tests/support/mod.rs:158–168`, `Server::interrupt`: `Command::new("sh").args(["-c", "kill -INT <pid>"])`, from S11-B's D-SB13. Its only caller on main is `tools/cli/tests/hosted_town.rs:79`, which asserts a clean exit and then reads the `[world] ticks` statistics line. S11-C's CA-13 and S11-D's SD-D13 check will be further callers (`step-12-server.md` §17.14, §18.14) | as cited |
| `mineworld-test-support` | `tests/support/src/lib.rs`: `#![forbid(unsafe_code)]`, standard library only (DEP-29); scratch directories only; its own tests in `tests/support/tests/scratch.rs` | `tests/support/Cargo.toml`, `lib.rs:17` |
| The server's stop | `tools/cli/src/serve.rs:214`: `serve_with_shutdown(…, async { let _ = tokio::signal::ctrl_c().await; println!("\n[mineworld] stopping"); })` | as cited |
| tokio on Windows | tokio 1.53.1 (`Cargo.lock`): `signal::ctrl_c()` on Windows registers **only `CTRL_C_EVENT`** (`tokio/src/signal/windows/sys.rs:47`, `ctrl_c()` → `SignalKind::CtrlC`). `CTRL_BREAK_EVENT` is a separate `signal::windows::ctrl_break()` | the registry source |
| SD-D13 (S11-D, frozen, in implementation) | "The shutdown future … completes on the first of `tokio::signal::ctrl_c()` (Ctrl-C on Unix; Ctrl-C **and Ctrl-Break** on Windows) and, under `#[cfg(windows)]`, `ctrl_close()` and `ctrl_shutdown()`" | `step-12-server.md` SD-D13 row |
| Windows console groups | A process started with `CREATE_NEW_PROCESS_GROUP` has Ctrl-C **disabled**, and only Ctrl-Break can be sent to its group with `GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, pid)`. Sending Ctrl-C to one child alone is not possible, and without a new group the event reaches the caller too (Win32 documentation) | Win32 `GenerateConsoleCtrlEvent`, `CREATE_NEW_PROCESS_GROUP` |
| `Child::kill` on Windows | `TerminateProcess(handle, 1)`: the killed child's exit code is 1, with no signal to read | Rust std `sys/pal/windows/process.rs` |
| Self-re-executing programs | `kill_and_resume` and `arrival_resolvers_resume` (`harness = false`) re-execute themselves through `std::env::current_exe`, which is portable | `persistence/Cargo.toml:23`, `tests/acceptance/Cargo.toml:17` |

**F-13w-2 (material for S11-D, raised now).** SD-D13's "Ctrl-C and Ctrl-Break on Windows" through
`tokio::signal::ctrl_c()` does not hold for tokio 1.53.1: Ctrl-Break needs `signal::windows::ctrl_break()`.
Unhandled, Windows' default handler ends the process with `STATUS_CONTROL_C_EXIT` (`0xC000013A`). Then
there is no `[mineworld] stopping`, no checkpoint and no statistics: an operator's Ctrl-Break and every
graceful-stop test on Windows would lose the graceful path. Routed to the primary session for S11-D
(§15.4, QW-1).

## 15.3 The portable "was killed" check

**In `mineworld-test-support`, a new module `process`, standard library only, no `unsafe` on any
platform:**

```text
pub struct Killed { status: ExitStatus, was_running: bool }

pub fn kill(child: &mut Child) -> Killed
    1. was_running = child.try_wait()? is None   (the child had not exited on its own)
    2. child.kill(); status = child.wait()
pub fn killed(&self) -> bool
    was_running && the status is a kill on this platform:
      unix     status.signal() == Some(9)               (std::os::unix::process::ExitStatusExt,
                                                         now inside the support crate, cfg(unix))
      windows  status.code() == Some(1)                  (TerminateProcess's code, as std uses it)
pub fn status(&self) -> ExitStatus
impl Debug for Killed (prints both, for assertion messages)
```

- **Strong typing.** Only `process::kill` makes a `Killed`. A test asserts `killed.killed()`, never a raw
  exit code.
- **Windows ambiguity.** An exit code of 1 alone could also be a child failing by itself. Two things keep
  that from passing:
  - `was_running` is checked immediately before the kill;
  - every killing test already proves separately that the child had not finished (§15.2, "Kill
    patterns").

  The residual race is a child exiting with code 1 by itself in the microseconds between `try_wait` and
  `TerminateProcess`. It is recorded as an accepted limitation in the DEP-29 note.
- **The nine files.** Each is changed only to:
  - drop the `ExitStatusExt` import;
  - route its kill through `process::kill` (or keep its own `child.kill()` site, but take the verdict
    from `Killed`);
  - assert `killed.killed()`, with the existing message.

  No other line changes, and every assertion keeps its meaning. The CLI test support's `Server::kill()`
  returns `Killed` instead of `ExitStatus`.

## 15.4 The `interrupt` helper, and the server's Ctrl-Break

**Helper, in `mineworld-test-support::process`:**

```text
pub struct Interruptible(Command)        constructed by interruptible(command): on windows it sets
                                         creation_flags(CREATE_NEW_PROCESS_GROUP) (std's
                                         os::windows::process::CommandExt, safe); on unix nothing
impl Interruptible { pub fn spawn(self) -> io::Result<InterruptibleChild> }
pub struct InterruptibleChild { child: Child }   (Deref to Child for stdout/stderr/id)
pub fn interrupt(child: &mut InterruptibleChild) -> io::Result<ExitStatus>
    unix     sh -c "kill -INT <pid>" (moved unchanged from tools/cli/tests/support/mod.rs:163),
             then wait
    windows  GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, pid) — pid is the new group's id — then wait;
             a zero return is io::Error::last_os_error(), never ignored
```

- **Types make the dangerous call impossible.** `interrupt` accepts only a child spawned through
  `interruptible`, so it never sends Ctrl-Break to a process that is not a group leader. On Windows such
  a call would reach every process on the console, the test runner included.
- **Why `sh -c kill` stays on Unix.** It is portable across Linux and macOS, needs no `libc` and no
  `unsafe`, and CI already proves it (13a's container, `hosted_town`). Moving it into the support crate
  makes it one definition.
- **Windows FFI: own declaration, not `windows-sys`.** Calling `GenerateConsoleCtrlEvent` is `unsafe`
  either way, because `windows-sys`' functions are `unsafe`. So the choice is between a new direct
  dependency and one `extern "system"` declaration from `kernel32`, which std already links:
  - **(a) Own declaration** (recommended, QW-2). The crate stays standard-library-only (DEP-29).
    `#![forbid(unsafe_code)]` becomes `#![deny(unsafe_code)]`, with one
    `#[allow(unsafe_code)]` function under `cfg(windows)`. Its safety comment says the call takes two
    integers and touches no memory.
  - **(b) `windows-sys` 0.61.2,** already in `Cargo.lock` transitively, as a `cfg(windows)` dependency.
    It is a real dependency (`REUSE_POLICY.md` §11) for one function, and still `unsafe`.
  - Either way, a DEP-29 note records it: why not the other, and the accepted limitations.
- **Callers.** The CLI test support's `Server::start` spawns through `interruptible`. `Server::interrupt`
  calls `process::interrupt` and keeps its signature. `hosted_town` is unchanged. CA-13 and SD-D13's
  check, when they land, call `Server::interrupt` and so become portable without edits. Their
  `#[cfg(unix)]` gates are removed by whichever of S11 and 13w lands second (QW-3).
- **The console requirement.** `GenerateConsoleCtrlEvent` needs the caller to be attached to a console.
  GitHub's Windows runners run steps under a console†. The helper turns a missing console into a named
  error ("no console attached; Ctrl-Break cannot be delivered"), so it can never pass silently. If the
  runner has no console, the bounded fallback is `AllocConsole` in the helper, applied once and recorded.
  Beyond that it is a material stop (R-W1).

**The server must stop on Ctrl-Break (F-13w-2).**
- **Recommended home: S11-D's SD-D13.** S11-D edits that very shutdown future and owns `serve.rs`. Adding
  `tokio::signal::windows::ctrl_break()` to its `cfg(windows)` select, beside `ctrl_close()` and
  `ctrl_shutdown()`, is one line inside its frozen intent ("Ctrl-C, Ctrl-Break and closing the console
  window"). It corrects the design's mistaken premise rather than widening it.
- The primary session relays F-13w-2 to the S11-D implementing session. **QW-1** records the answer.
- **Fallback,** only if S11-D merges without it: 13w makes that one-line edit in `tools/cli/src/serve.rs`,
  an authorized exception to the material boundary that the contract names (§15.9). It also adds a
  `#[cfg(windows)]` comment citing F-13w-2. Nothing else in `src/` is touched.
- **Evidence:** `hosted_town` on `test-windows` passes. That means `ended.success()`,
  `[mineworld] stopping`, and the `[world] ticks` line with its p99 bound.

## 15.5 `test-macos`

- A job `test-macos` on `macos-26`, through 13b's `.github/actions/native` with layer `core`: the same
  commands as Linux `test` (I-13b-1's list), natively. It has `fetch-depth: 0`, `filter: blob:none`, and
  `timeout-minutes` 60.
- **Triggers, set by 13w for both `test-macos` and `test-windows`:** non-draft `pull_request`, push to
  `main`, `workflow_dispatch`. 13b's "not on PRs until green" holds, because 13w turns them on in the
  same PR that makes them green, and shows them green on its own head first.
- **Not required.** Whether they become required is QB-11 `[OM]`: after five consecutive green `main`
  pushes, by the operator's settings change. ARC-48 gains a dated note with the two rows.
- **Cost and time:** both are standard runners, free on a public repository†. Measured in 13w's runs
  (B-W5). The estimates are about 15–20 min warm for macOS (the operator's Mac runs the suite in about
  5.5 min on more cores; E-S13-0) and about 20–30 min warm for Windows. Both run in parallel with Linux
  `test`.
- **E-c's `platforms` job (QB-15).** Once both jobs are green, `platforms` is a strict subset of them.
  13w removes the `platforms` job from `ci.yml` and keeps the layer only if another consumer names it.
  The removal is recorded in ARC-48's note. If the primary session prefers to keep `platforms`, it stays
  and nothing else changes (QW-4).

## 15.6 Acceptance (decided before measuring)

```text
A-W1  test-windows green on 13w's final head (a dispatch, then the PR run once its PR trigger is in):
      the same `test result:` count as Linux test on the same head (passed + ignored equal, 0 failed),
      kill_and_resume prints [cafe] PASS and [clock] PASS, arrival_resolvers_resume [resolver-yard] PASS,
      ac1_composability 13/13; check_scratch.py left clean
A-W2  test-macos green on the same head, with the same counts
A-W3  Linux `test` and `fast` green on the same head, counts unchanged from main's (no test lost)
A-W4  hosted_town passes on all three OSes: a clean exit after interrupt, `[mineworld] stopping`, the
      ticks line with p99 ≤ 50 ms reported (CP-B4's form)
A-W5  scope: git diff --stat lists only test files (the nine, tools/cli/tests/support/mod.rs),
      tests/support/ (src, tests, Cargo.toml), .github/workflows/ci.yml, docs/DECISIONS.md (DEP-29 and
      ARC-48 notes), docs/MVP_STATUS.md, this file — plus tools/cli/src/serve.rs ONLY under §15.4's
      fallback (one line); no `#[ignore]`, `cfg`-skip or excluded target added anywhere (grep diff)
A-W6  every assertion in the nine files keeps its message and its claim (review of the diff, line by line)
A-W7  test-windows and test-macos run on pull_request in this PR's own checks, are not required, and
      their wall times (cold and warm) are recorded
```

## 15.7 Mutations (decided before measuring)

Each runs on a scratch branch `scratch/13w-<name>`, which is deleted after its run. Exceptions: MW-1 and
MW-2 are local unit checks in `tests/support/tests/process.rs`, kept as tests.

```text
MW-1  unit, every OS: a child that exits 0 by itself, and one that exits 1 by itself, then "killed" via
      process::kill after they have exited → killed() is false (was_running false); a sleeping child
      killed → killed() is true. Committed as tests of the helper (they own the fail-closed verdict)
MW-2  unit, every OS: interrupt of a sleeping helper child that handles nothing → the child ends and
      interrupt returns its status (Unix: signal 2; Windows: 0xC000013A); a child spawned without
      interruptible cannot be passed to interrupt (a compile-fail doctest)
MW-3  CI, Windows: the scratch removes the server's Ctrl-Break handling (or, before S11-D lands, uses
      main's server) → hosted_town red on test-windows, naming the missing `[mineworld] stopping` and
      the exit status 0xC000013A; Linux and macOS green. Shows the Windows path really exercises the
      server's stop
MW-4  CI, Unix: the scratch makes process::interrupt send SIGKILL instead → hosted_town red on Linux
      and macOS (no ticks line); shows the helper's Unix path is what the test relies on
MW-5  CI, Windows: in one of the nine files, the kill is moved after the child's completion line (it
      finishes first) → that test red on test-windows AND Linux test, with the original message
      ("killed by SIGKILL, not finished"); shows the portable check keeps the claim on both
MW-6  CI, macOS: a planted #[cfg(target_os = "macos")] assert!(false) in one test → test-macos red,
      Linux test and test-windows green; shows test-macos bites
```

## 15.8 Commit plan

Each commit tracks implementation, validation and review separately.

### W-C0 — Design (this section), docs only

- [x] Implementation: §15.
- [x] Validation: `check_doc_headings`, `check_decision_ids` (the planning commit and the PR description).
- [x] Self-review:
  - every §15.2 anchor cites a file and line, or the tokio source;
  - F-13w-1 and F-13w-2 are new and stated;
  - the material boundary is explicit.
- [ ] Review: the primary session, then the freeze.

### W-C1 — `mineworld-test-support::process` (`kill`, `Killed`, `interruptible`, `interrupt`) and the DEP-29 note

- [x] Implementation:
  - the module, as §15.3 and §15.4 describe;
  - the `deny(unsafe_code)` change with one allowed function, per QW-2;
  - its tests (MW-1, MW-2);
  - `docs/DECISIONS.md`, a DEP-29 note.
  - Done: `tests/support/src/process.rs` (`kill`, `Killed { status, was_running }` with `killed()`,
    `status()` and a `Debug` that prints the verdict too; `interruptible`, `Interruptible::spawn`,
    `InterruptibleChild` with `Deref`/`DerefMut` to `Child`; `interrupt`; the private `send_interrupt`,
    `sh -c "kill -INT <pid>"` on Unix and the one `#[allow(unsafe_code)]` `GenerateConsoleCtrlEvent` call
    on Windows, a missing console (`ERROR_INVALID_HANDLE`) named). `lib.rs`: `forbid` → `deny`,
    `pub mod process`. `tests/support/tests/process.rs`: MW-1 and MW-2, the children being the test
    binary re-executed into `child_entry_point` with a role (no `sleep` on Windows). The compile-fail
    doctest (a plain `Child` passed to `interrupt`) beside a `no_run` doctest that compiles. DEP-29 note
    (2026-10-09).
  - **D-13w-1 (bounded).** `DerefMut` as well as `Deref` on `InterruptibleChild`: the CLI support takes
    the server's stdout and stderr (`Option::take`) and polls `try_wait`, both `&mut`. A swap of the
    inner `Child` for another is not reachable without a second spawn, so the guarantee holds in practice.
  - **D-13w-2 (bounded).** MW-2's per-platform expected status is read in the test file under
    `cfg(unix)` / `cfg(windows)`, so `ExitStatusExt` also appears in `tests/support/tests/process.rs`,
    not only in `src/process.rs` (W-C2's grep). Both sides assert; neither is skipped.
- [x] Validation:
  - local, on the Mac: `cargo test -p mineworld-test-support`, then `cargo clippy` with `-D warnings`;
  - Windows: compiled and run by this PR's `test-windows` dispatch.
  - Local (Mac, working tree on `cf18713`): `process.rs` 4 passed, `scratch.rs` 7 passed, doctests 2
    passed (the `no_run` compiles, the `compile_fail` fails to compile). **PASS.** `cargo clippy -p
    mineworld-test-support --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean.
  - First attempt: the sleeping child's marker was matched as a whole line and never seen, because
    libtest under `--nocapture` prints `test child_entry_point ... ` with no newline before the test's
    output; matched with `ends_with` instead.
  - **MW-1 mutation (local):** `killed()` without `was_running`, and code 1 read as a kill (Windows'
    ambiguity, emulated on Unix) → `a_child_that_ended_by_itself_is_never_reported_killed` red, "exit-1:
    … Killed { status: … (256), was_running: false, killed: true }"; restored → green. **PASS.**
  - Windows: see W-C4's runs.
- [x] Review:
  - no `unsafe` outside the one function;
  - every error is surfaced, none ignored;
  - the doc comments name the platforms' semantics and the residual race.
  - Done: the only `unsafe` is the call in `send_interrupt` (and its `extern` block), under
    `cfg(windows)` and `#[allow(unsafe_code)]`; the crate `deny`s it elsewhere. `kill` panics, naming the
    step, if `try_wait`, `kill` or `wait` fails; `interrupt` returns `io::Error` for a failed `sh`, a
    non-zero `kill`, or a zero `GenerateConsoleCtrlEvent`. The module doc carries the platform table and
    the residual race; DEP-29's note repeats both.

### W-C2 — The nine files and the CLI test support

- [ ] Implementation: the nine files and `tools/cli/tests/support/mod.rs` (§15.3, §15.4).
- [ ] Validation:
  - local full suite on the Mac (`cargo test --workspace`): the same counts as before;
  - `git grep ExitStatusExt` → only inside `tests/support/src/process.rs`.
- [ ] Review: A-W6, line by line.

### W-C3 — The server's Ctrl-Break (**only under §15.4's fallback**; otherwise N/A, with S11-D's merge commit cited)

- [ ] Implementation: one `cfg(windows)` `ctrl_break()` in `serve.rs`'s shutdown select.
- [ ] Validation: A-W4 on Windows (MW-3).
- [ ] Review: no other `src/` change.

### W-C4 — The workflow: `test-macos`, both jobs' PR triggers, and `platforms` per QW-4

- [ ] Implementation:
  - `.github/workflows/ci.yml`;
  - ARC-48's dated note.
- [ ] Validation:
  - dispatch runs, then the PR's own runs (A-W1, A-W2, A-W3, A-W7);
  - MW-3 … MW-6 on scratch branches, each deleted after its run, then `ls-remote` → empty.
- [ ] Review:
  - neither job is required;
  - no `continue-on-error`;
  - labels pinned (`macos-26`, `windows-2025`);
  - the native action is reused, with no parallel definition (§13.0.3).

### W-C5 — Close

- [ ] Implementation:
  - `docs/MVP_STATUS.md`: the suite runs on all three OSes, with the run ids;
  - §13.10.1's W-T1, W-4 and W-12 marked resolved, or carried with owners;
  - this ledger.
- [ ] Validation: A-W5 and the doc checks.
- [ ] Review: every A-W and MW item has evidence or N/A; deviations are numbered D-13w-n.

## 15.9 Run budget

- **Local:** unrestricted (the Mac).
- **CI:**
  - at most **14** dispatch or scratch runs that include `test-windows` or `test-macos`;
  - those are the cold and warm dispatch (2), MW-3 … MW-6 (4), repairs of findings from the first
    Windows run (≤ 6) and the final head (≤ 2);
  - each job runs under its `timeout-minutes`;
  - about 14 × (Windows 30 + macOS 20 + Linux 14) ≈ 900 job-minutes, all on standard runners, free on a
    public repository†.
- **Monetary:** none.
- **Exceeding the budget,** or a Windows finding in production code, is a stop with a projection.

## 15.10 Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| R-W1 | No console on the Windows runner, so `GenerateConsoleCtrlEvent` fails | A named error, never a pass; `AllocConsole` as the one bounded fallback; then a stop |
| R-W2 | `test-windows` reveals failures beyond W-T1 (file locks, W-4; CRLF, W-3; paths) | Test-code causes are fixed in 13w within the repair budget. Production-code causes are material stops with owner lanes. Each is recorded as a W-finding |
| R-W3 | S11-D merges with SD-D13 as written (no Ctrl-Break) | §15.4's fallback, pre-authorized in the contract and bounded to one line |
| R-W4 | Windows' exit code 1 is ambiguous | `was_running` plus each test's own "not finished" check; the residual race is recorded (§15.3) |
| R-W5 | Windows CI is slow (MSVC links about 150 test binaries; Defender scans `target/`) | Measured (A-W7). If it exceeds 60 min, stop and report; no test is sharded or dropped |
| R-W6 | Parallel lanes add new Unix-only tests (as E-c added the ninth file) | After 13w, `test-windows` on PRs catches it. Until then, W-C2 re-greps on its final head |

## 15.11 Questions

`[OM]` marks an operator-material question.

| ID | Question | Recommendation |
| --- | --- | --- |
| **QW-1** | Ctrl-Break in the server (F-13w-2): S11-D's SD-D13, or 13w? | **S11-D's SD-D13**: it owns that select and its intent already names Ctrl-Break. The primary session relays F-13w-2 now; 13w's fallback covers the case where S11-D merges first without it. |
| **QW-2** | Windows FFI: own `extern "system"` declaration (crate stays std-only; `forbid` → `deny(unsafe_code)` with one allowed function), or `windows-sys`? | **Own declaration.** Both need `unsafe`; the declaration adds no dependency for one function. Recorded in a DEP-29 note. |
| **QW-3** | Who removes CA-13's and SD-D13's `#[cfg(unix)]` gates? | **Whichever of S11 (C/D) and 13w lands second.** It is a deletion of one attribute per test, verified on `test-windows`. |
| **QW-4** | Retire E-c's `platforms` job once `test-windows` and `test-macos` are green (QB-15)? | **Yes, in 13w's W-C4**, if both are green on 13w's head; otherwise keep it. |
| **QW-5** | 13w adds the `pull_request` trigger to `test-windows` and `test-macos`, non-required. Accept? | **Accept.** The operator's QB-11 decides on requiring them after five consecutive green `main` pushes. |
| **QB-11 [OM]** (carried) | Make `test-windows` and `test-macos` required once green on `main`? | Unchanged from §13.11: yes, after five consecutive green pushes. Settings are the operator's. |

## 15.12 Proposed execution contract

```text
PROJECT / PR        MineWorld mvp0 — S13 PR 13w, the default suite on Windows and macOS
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-14-ci.md §15 (the live ledger)
RELATED / BINDING   §13 (13b as frozen, esp. §13.0.1–13.0.3, §13.10.1); step-12-server.md §17.14,
                    §18.14 (SD-D13); docs/DECISIONS.md DEP-29, ARC-48; ENGINEERING_STANDARDS.md §22;
                    CLAUDE.md §§2–4
IMPLEMENTATION BASE main after 13b merged (and, for QW-1, after S11-D if possible); branch
                    mvp0/pr-13w-windows; worktree /Users/yuema137/mineworld-worktrees/impl-13w (sole writer)
PRECONDITION        13b merged; QW-1 … QW-5 answered (primary session, 2026-10-09, as recommended);
                    DESIGN FROZEN 2026-10-09, primary session
COMMANDS            as 13a/13b's contracts: cargo, git, gh (PR create/update, run list/view/download/
                    cancel, workflow run on mvp0/pr-13w-windows and scratch/13w-*; no merge, no settings),
                    python3 scripts/*; ordinary local checks unrestricted
APPROVED SCOPE      §15.3–15.5; W-C1 … W-C5
MATERIAL BOUNDARY   test code and mineworld-test-support only; the one exception is §15.4's fallback
                    line in tools/cli/src/serve.rs, authorized only if S11-D has merged without
                    Ctrl-Break
FROZEN INVARIANTS   I-S13-1 … I-S13-9 as amended for 13w (test edits allowed by this contract, and only
                    to the files of A-W5); I-13b-1 … I-13b-6
VALIDATION BUDGET   §15.9; monetary none
ENDPOINT AUTHORITY  commits, push, PR create/update, CI repair: authorized (D-12)
                    scratch/13w-* push + delete, workflow_dispatch: authorized with QS13-14's bounds
                      (freeze, 2026-10-09)
                    repository settings (required checks included): NOT authorized — operator only
                    merge: explicit operator authorization only
MATERIAL STOPS      a production-code cause of any Windows/macOS failure; any skip/ignore/cfg-gate/
                    exclusion; an assertion whose claim would have to change; R-W1 beyond AllocConsole;
                    R-W5; exceeding the budget; any settings change
NORMAL STOP         PR 13w READY FOR OPERATOR REVIEW — DO NOT MERGE
STOP CONDITION      A-W1 … A-W7 with evidence on the exact final head; MW-1 … MW-6 recorded
MERGE AUTHORITY     never without explicit operator approval
```

## 15.13 Handoff for 13w (continuation aid, not a design authority)

```text
PROJECT / PR        MineWorld mvp0 — S13 PR 13w (§15.12's contract, frozen 2026-10-09)
PRIMARY DESIGN DOC  this file, §15 (the ledger: §15.8)
BRANCH / BASE       mvp0/pr-13w-windows from origin/main @ cf18713 (13b merged, #97; S11-D merged, #104)
WORKTREE            /Users/yuema137/mineworld-worktrees/impl-13w (sole writer; created by this session)
ENDPOINT AUTHORITY  commits, push, PR create/update, CI repair: authorized (§15.12, D-12)
                    scratch/13w-* push + delete, workflow_dispatch: authorized (§15.12, QS13-14's bounds)
                    settings, merge: NOT authorized (operator only)
POST-MERGE SYNC     this session: §15's ledger, merge identity, evidence, deviations; the primary
                    session: parent documents (overall, other steps)
```

**Re-audit at the base (2026-10-09, `git grep -n "std::os::unix\|cfg(unix)\|ExitStatusExt"` and
`.signal()`, `"sh"`, `kill -` on `cf18713`).** Every hit, and its handling:

| Site | Finding | Handling |
| --- | --- | --- |
| The nine files of §15.2 (line numbers moved: `configuration_seam.rs:31,358`) | `ExitStatusExt`, `signal() == Some(9)` | W-C2: `process::kill` / `Killed::killed()` |
| `tools/cli/tests/support/mod.rs:187` (W-6) | `sh -c kill -INT` | W-C2: `Server` spawns through `interruptible`, `Server::interrupt` calls `process::interrupt` |
| `tools/cli/tests/admin.rs:557` (SD-D13's check) | `#[cfg(unix)]` on `an_interrupt_stops_an_administered_server_gracefully` | W-C2: the gate removed (QW-3: 13w lands second) |
| `tools/cli/tests/server_command.rs:254` | `Server::kill()`'s status read with `success()` | W-C2: reads `Killed` (the type changed, §15.3) |
| `worldpack/src/configure/tests.rs:655` (IL-a/IL-b) | `#[cfg(unix)]` block: a file symlink out of the pack, refused as `AttachmentOutside` | W-C2: a Windows equivalent, see D-13w-3 |
| `tools/cli/src/serve.rs:267` | `#[cfg(windows)]` Ctrl-Break, close, shutdown in `stop_requested` | Production code, S11-D's (`9d83937`, merged by #104): already handles Ctrl-Break, so W-C3 is N/A |

**Current checkpoint.** W-C1 committed. Next: W-C2.
