## DEP-17 — CI runs on GitHub Actions hosted Linux runners, inside the repository's own toolchain container

**Date** 2026-10-08 · **Status** selected; integrated in S13 PR 13a · **Approved by** the primary session
at 13a's design freeze (step-14 §9.0) under the operator's 2026-10-08 decision to make the repository
public · **Relates to** `ARC-48`, `DEP-18`, `ARC-30`, `ARC-35`, overall `D-12` · **Design**
`.structured-coding/plans/mvp0/step-14-ci.md` §§5.1–5.4, 6, 9

**Problem.** `ENGINEERING_STANDARDS.md` §15 says quality gates "must remain automatic", and §16 asks for
four CI layers. Until S13 the repository had no CI at all: every gate was a session running commands on
the operator's Mac. CI also has to be the other side of `AC-8`, which is native Linux x86_64.

**Options considered** (`REUSE_POLICY.md` §§11–12, §17, both directions).

```text
CI service
(a) GitHub Actions, hosted ubuntu-24.04           chosen
(b) GitLab CI through a push mirror               review on one platform, checks on another
(c) self-hosted Woodpecker or Forgejo Actions     a server MineWorld would have to operate
(d) CircleCI / Buildkite                          no fit advantage; a second account
(e) Actions on a self-hosted runner, the          arm64 macOS, so it cannot be AC-8's Linux x86_64
    operator's Mac                                side, and it would run PR code on the operator's
                                                  machine; kept only as a layer-4 fallback
(f) our own scripts and cron                      no PR integration

How a job gets the toolchain
(A) build the Dockerfile's `toolchain` stage, then `docker run` each layer      chosen
    with the checkout mounted
(B) `jobs.<id>.container:` on the official image, plus inline apt installs      the environment
    defined twice; actions/checkout silently falls back to a tarball with no .git when git is missing,
    which breaks AC-1's history scans
(C) the toolchain on the bare runner                                           the tests would not run
                                                                               in the container AC-8 names
(D) a prebuilt toolchain image on GHCR                                         needs packages: write,
                                                                               which the read-only workflow
                                                                               refuses

Test runner
plain `cargo test --workspace`      chosen: the project's canonical command, so CI and laptop agree
cargo-nextest                       declined for now (below)

Cache
actions/cache with an explicit key  chosen
Swatinem/rust-cache                 keys on the runner's rustc, not the container's, under (A)
sccache (GHA backend)               no gain on a link-heavy test build of ~117 binaries; revisit
```

**Choice.** GitHub Actions on hosted `ubuntu-24.04`. Each job builds the `toolchain` stage of the
repository's `Dockerfile` (`DEP-18`) with the BuildKit GitHub Actions cache, and runs one command inside
it: `python3 scripts/ci_layer.py <layer>`. The registry, git dependencies and compiled dependencies are
cached with `actions/cache` under a key of `Cargo.lock`, `rust-toolchain.toml` and the `Dockerfile`.
Third-party actions are pinned to full commit SHAs.

**Why not ourselves.** CI is commodity infrastructure (`REUSE_POLICY.md` §4). The repository, its PRs and
its review flow are already on GitHub (`D-12`). Checks reported there are the ones the operator sees
when reviewing.

**Why not the others.**
- (b), (d): no fit advantage, and a second platform or account to keep.
- (c): operational burden (`REUSE_POLICY.md` §2).
- (e): the wrong platform for AC-8, and an execution risk.
- (B): two definitions of the environment, and a checkout path that fails open.
- (C): would test outside the container that AC-8 names.
- (D): permission and storage cost. Public packages are free, so the storage reason lapses once the
  repository is public. The permission reason stands. Revisit if building the image per job is measured
  above about 2 minutes.
- **cargo-nextest.** It lists custom-harness targets by invoking them with libtest's
  `--list --format terse`. The project's two `harness = false` programs (`persistence/tests/kill_and_resume.rs`,
  `tests/acceptance/tests/arrival_resolvers_resume.rs`) do not implement that interface. Its retries would
  hide nondeterminism, which this project treats as a defect. Its gain is small, because six long tests
  dominate the suite. Revisit if `test` passes about 15 minutes cached, or needs sharding.

**Isolating interface.** `.github/workflows/ci.yml` only checks out, restores caches, builds the toolchain
image and calls `scripts/ci_layer.py`, the last three through the local composite action
`.github/actions/layer`. No check command appears in YAML (`ARC-48`, I-S13-9). Moving to another CI
service rewrites those two files. Changing what a layer runs edits one list in `scripts/ci_layer.py`, and
the same entry point runs locally.

**Accepted limitations.**
- **Cost.** As a private repository on GitHub Free, CI would draw on 2 000 included Linux minutes a month,
  while the project's PR rate needs several times that (step-14 §10.1). The repository became public on
  2026-10-08, so standard hosted runners are free. Runs are still kept purposeful.
- **Runner disk.** The default suite once wrote about 16 GB of scratch saves (step-14 F-3); since the
  test-hygiene PR (#77, `DEP-29`) each test removes its own, and `test` checks that nothing is left. The
  `test` layer prints free disk before and after. The public runner measured about 107 GB free before
  the tests, so no clean-up step is needed. A future shortfall is remedied in the workflow, never by changing tests in
  CI.
- **Building the image per job.** About 30–90 seconds per job. This is the price of one definition of the
  environment.
