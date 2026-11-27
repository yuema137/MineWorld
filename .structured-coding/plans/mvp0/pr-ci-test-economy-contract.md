# Execution contract and ledger — CI test economy

**Design:** [`pr-ci-test-economy.md`](pr-ci-test-economy.md) (DESIGN FROZEN 2026-10-27, #520). This file is the
execution contract and the live ledger for that design. It adds no requirement. Where the two disagree, the design
governs, and the disagreement is recorded here as a deviation.

## 1. Contract

```text
PROJECT / PR:
  CI test economy, P1-P4 of pr-ci-test-economy.md §5 (P0 is ci/no-fail-fast, #523, another agent's).

PRIMARY DESIGN DOC:
  .structured-coding/plans/mvp0/pr-ci-test-economy.md

RELATED / BINDING DOCS:
  pr-ci-milestone-runs.md (ARC-154), step-14-ci.md, pr-test-hygiene.md (I-TH-1), docs/decisions/ARC-110.md,
  docs/ENGINEERING_STANDARDS.md §§15-18, 22, docs/REUSE_POLICY.md, CLAUDE.md §4.

OPERATOR RULINGS (2026-10-27, relayed by the primary session):
  Q-1  a required `impact` job on PRs, Linux only, <= 25 min wall.
  Q-2  QS-59 amended: the 300-day horizons move to the nightly, plus one Linux run on the final head.
  Q-3  adopt cargo-nextest, with a DEP record.
  Q-4  P0 ships now (another agent: ci/no-fail-fast).
  Q-5  market_settlement: 5 seeded kill points in the gate, 20 in the nightly.

APPROVED SCOPE:
  P1 nextest in CI (DEP record); P3 shared 30-day baselines and de-duplicated identical runs; P2 the long tier
  (300-day legs, the 20 kill points) to the nightly plus the final-head rule; P4 the `impact` job. P5 (`rerun_of`)
  is not in this lane's brief and stays open.

FROZEN INVARIANTS:
  - I-TH-1: no assertion, expected value, seed or world changes. A test only moves tier or reads a shared save.
  - A determinism test that compares two independent processes keeps its own second process (ARC-23).
  - Impact I-1: selection never drops a test that rule 1 (changed test file) or rule 7 (fail closed) selects.
  - Impact I-2: `impact` always reports and lists its selection and why.
  - Impact I-3: the milestone gate stays the authority.
  - No world-specific hard-coding: path rules and the pack map are data, recorded at run time.
  - All three OSes keep running the gate tier; the long tier runs on Linux (nightly, final head).

SPLIT INTO PRs (each its own branch and worktree):
  E1  ci/econ-nextest        P1: cargo-nextest runs `core` and its shards, JUnit artifacts, DEP record.
  E2  test/econ-shared-runs  P3: tests/support shared_run, the shared baselines, price_survey W2 once.
  E3  test/econ-long-tier    P2 + Q-5: the long tier, the nightly `long` layer, the final-head rule, ARC note.
  E4  ci/econ-impact         P4: the `impact` job, ci_changes.py rules, the recorded test -> pack map, ARC note.
  E5  operator action        branch protection: `impact` becomes a required check (repo settings).

VALIDATION BUDGET:
  - local heavy jobs only through cast-slots/with-slot.sh, at most 2 at once for this lane;
  - at most 2 code-PR CI runs in flight for this lane;
  - a full (milestone) CI run only where the design requires one: a change to the CI definition (E1, E3, E4).
```

## 2. P1 recovery contract (2026-11-22; supersedes multi-PR execution above)

Owner: fresh P1 session only, isolated shared clone `/tmp/mineworld-recover-ci-p1`, branch
`recover/ci-p1-nextest`. Source `631045e4161d59de89a5958cb751973ff427d8b2`; prerequisite
P0 `ce69e9e5210c1ca11ed8637b943cc72aea38d5a0`, OPEN #523, functional independent review PASS at ce69e9e5. P0 doc-only correction
`2adf50427e920dad2284c7e484ae310146bee529` integrated with executable parity PASS.
Main `2b4d36fac345b9325a91fd4313d40e3633e728b0` deliberately merged before implementation;
its ci_gdparse self-test is preserved. Old checkout is read-only and clean; snapshot
`/tmp/ci-p1-recovery-snapshot/source.tar`, with status, diff and untracked inventory beside it.
Shared worktree registration failed with Operation not permitted; authorized shared clone used.

Authority: coordinator dispatch and kickoff-ci-p1-recovery.md from coordinator checkout; approved
Q-3 at original operator JSONL lines 26848/26860 (2026-10-27T21:23:54.135Z).
Only P1 implementation, bounded local validation and semantic commits authorized. No public writes,
workflow dispatch, merge, paid APIs or subagents. Coordinator owns shared docs/dashboard and independent
red-team review. Stop at local evidence readiness; full milestone CI is NOT RUN, a remote obligation.
P2/P3/P4/P5, Python splitting, guide work and test semantics remain excluded. I-TH-1 holds unchanged.

Implementation sequence: repair continuation/results; implement archive producer and consumers with
workflow wiring; targeted real fixture and MineWorld validation; evidence/debt packet to primary.
One heavy slot initially (maximum two), via cast-slots/with-slot.sh, respecting PAUSE/PRIORITY_ONLY;
real runs approximately ten minutes each, one hour total; no full workspace test campaign.

### Bounded implementation decisions and audit

- Preserved P1 ci_shard.is_test_run misses ci_repeat's Python wrapper. Restore continuation through the
  wrapper (a P1 regression, not a P0 bug), retaining nonzero final status and diagnostic summary. Build failures still stop.
- Adopted archive requirement is unfinished in preserved DEP-163; this recovery completes it.
  One producer per OS emits nextest archive for regular lib/bin/integration tests; existing weighted
  units select consumers by nextest filtersets. Doctests and custom harnesses retain Cargo fallback
  on their owning shard (their compilation is not represented by nextest archives).
- Tests embed env!(CARGO_MANIFEST_DIR) and env!(CARGO_BIN_EXE_mineworld). Remapping runtime variables
  cannot fix compiled constants without prohibited Rust edits. Require identical absolute workspace
  root, OS/architecture, toolchain and source SHA, and extract into the original target location.
  Hosted producer/consumer use identical labels/checkouts and Linux toolchain container. Mismatch
  fails explicitly. Archive includes non-test binaries through nextest's own machinery.
- Authoritative semantics: https://nexte.st/docs/ci-features/archiving/ and pinned 0.9.148 installed
  help (`/tmp/ci-p1-nextest-{run,archive}-help.txt`). Archive consumers cannot carry Cargo target flags;
  selection uses binary filtersets. --extract-to plus --extract-overwrite preserves target paths.
- Unit/static ownership: partition completeness, custom-harness coverage, wrapper continuation,
  archive provenance and command plans, pins/config/workflow structure, strict JUnit aggregation.
  Real lifecycle ownership: build once then consume twice, failures/ignored cases, binary paths,
  custom harness and doctest fallback, real MineWorld small targets. Gate 1 N/A: no LM boundary.

### Recovery ledger

- [x] Implementation: continuation, archive producer/consumers, provenance, strict results and pins.
- [x] Deterministic validation: script/static checks and bounded real fixture/MineWorld runs.
- [ ] LLM logic review: local contract/debt audit; independent red-team remains primary-owned.

Current checkpoint: implementation and bounded deterministic validation PASS. No owned heavy job running.
Final source revision `b191491e72be691a2e97926f0e102781da16f113` includes P0 doc-only correction.
Next: send exact-head evidence for independent red-team. Stop for a material contract departure or missing capability
that cannot be resolved within the recorded budget. Read this ledger and actual git/process state on resume.

### Real fixture Gate specification

Claim: the implemented producer builds an archive once; two weighted consumers extract it after
all original target outputs are removed, preserve the direct-run identity set (ignored tests stay
ignored), reach an auxiliary executable through compile-time paths, and both fail visibly. Cargo
custom harness and doctests continue after nextest failures. Owner: real subprocess/archive lifecycle.
Command: `PATH=/tmp/ci-p1-tools:$PATH RUSTUP_TOOLCHAIN=1.97.1 .../with-slot.sh
.../MineWorld/.venv/bin/python scripts/test_ci_execution.py --integration`.
Budget: dependency-free fixture, one slot, expected <1 minute, bounded by 10-minute command timeout.
Evidence: ci-p1-recovery-evidence/fixture.log and source-fingerprint.json. Full suite not run.
Static baseline: ci_shard self-test and six focused orchestration regression tests PASS.
System Python lacks tomllib; Homebrew Python has broken expat linkage. Existing uv Python 3.14.5
(`/Users/yuema137/MineWorld/.venv/bin/python`) works; no interpreter files changed.

### MineWorld Gate specification

Archive and execute `mineworld-packages` targets `manifest`, `structure` and `mineworld-test-support`
target `scratch` using the shipping nextest ci profile, original source/target paths and toolchain 1.97.1.
Compare archived JUnit identity set to direct nextest list; require all selected tests and final scratch
cleanup to pass. This exercises actual source fixtures, compile-time paths and scratch guard behavior,
without a whole-workspace test run. Command driver `/tmp/ci-p1-real-check.py` is retained in evidence.
One slot, expected <3 minutes; each subprocess bounded at 600 seconds, no paid calls.


### Evidence and debt closeout

- PASS: six focused orchestration/provenance/result tests; shard, repeat (11), gdparse (7) self-tests;
  pin consistency, compileall, decision IDs and documentation headings. New modules pass Ruff lint/format.
  Workflow YAML parses and three producer/consumer/gate graph checks pass; `workflow.log` records these.
- PASS: `fixture.log` from the real pinned nextest subprocess fixture. One producer archive; original
  target removed; two weighted consumers return 100, four nonignored identities equal direct list,
  auxiliary binary via compiled env! path succeeds, custom harness and failing doctest run after failure.
  Real wrapped `core-repeat` executes both alpha and doctest twice, `ran yes` for each sample, fails overall.
  Saved shard XML and repeat summaries preserve results. Final fixture uses actual custom manifest audit.
- PASS: `mineworld.log`, 16/16 actual MineWorld tests, archive identity set equals direct nextest list,
  and scratch cleanup passes. `real-check.py` records the bounded command driver. TMPDIR, TMP and TEMP
  point to `artifacts/real-temp` and are inherited by Cargo/nextest tests and the unchanged checker.
- Initial MineWorld attempt: tests passed 16/16; global scratch scan FAILED on pre-existing retained
  #530 full-link evidence `mineworld-full-link-honaky1m` (863.2 MiB). Coordinator confirmed ownership.
  This is unrelated retained evidence, not a P1 leak. It was not deleted. Production checker/exclusions
  remain unchanged. `mineworld-attempt1.log` preserves the original outcome.
- Initial fixture attempts FAILED for fixture setup: missing generated Cargo.lock, then a doc-only shard
  while the fixture expected JUnit on every shard. Generate the lock and balance alpha/beta explicitly;
  shipping assertions/worlds/seeds untouched. Earlier attempt logs retained, not represented as PASS.
- Timing configuration audit: retained `threads-required = "num-test-threads"` reserves all nextest
  slots for each timing test; timing group max-threads=1 also serializes those tests. Upstream authority:
  https://nexte.st/docs/configuration/threads-required/. This controls nextest scheduling only; it does
  not constrain a test's child process threads or unrelated host load. No heavy timing test claim here.
- PASS: no shipping Rust/assertion/seed/world changes, gdparse self-test retained. Streamed logs now retain
  only diagnostic lines; full console output remains visible, JUnit supplies structured nextest failures.
- NOT RUN: full workspace/milestone CI, hosted Linux container and Windows executions, hosted artifact
  transfer/rerun behavior, wall-clock timing suites. Those remain final milestone/independent-review
  obligations. Local macOS evidence does not certify them. No public write, dispatch, paid call or subagent.
- Deployment constraint (explicit, not hidden debt): archives require matching revision, OS/architecture,
  toolchain, nextest and absolute checkout; nondefault target directories are refused. Same hosted runner
  label/root is required because tests compile env! paths. Cargo-only fallbacks compile on owning shards.
- Logic self-audit complete; independent exact-head review still PENDING and owned by primary. No merge
  readiness claim before that review. Original clean checkout and snapshot remain preserved.
