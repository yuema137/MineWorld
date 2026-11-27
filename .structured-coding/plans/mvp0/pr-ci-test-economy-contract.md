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

## 2. Ledger

Newest entries last within each PR. `[ ]` / `[x]` per item; evidence is a run URL, a SHA or a command output.

### P0 prerequisite — failure visibility (#523)

Status at the 2026-11-23 backlog audit: public #523 remains OPEN at
`ce69e9e5210c1ca11ed8637b943cc72aea38d5a0`; local correction is
`2adf50427e920dad2284c7e484ae310146bee529` in `/tmp/mineworld-recover-ci-p0`.
This is evidence of a local candidate, not delivered/merged P0.

- [x] Implementation: shard test/doctest `--no-fail-fast`, layer continuation after test failures,
  first nonzero result retained, non-test/build failures stop, and Docker Markdown summary.
  The local correction fixes only the stale module docstring.
- [x] Bounded deterministic validation: real dependency-free Cargo proof observes two failing
  integration binaries and a failing doctest, reaches a later command, returns 101, stops a
  non-test exit 7, clears stale summary on success, and retains exactly one repeat flag.
  Committed AST excluding the module docstring and byte-identical executable suffix establish
  parity from the reviewed public payload to `2adf5042`; no new executable test run is claimed.
- [x] Independent logic review: functional PASS at `ce69e9e5`; the required docstring correction
  is present in `2adf5042` with parity PASS. Local coordinator packet:
  `/Users/yuema137/MineWorld/.structured-coding/plans/mvp0/ci-p0-recovery-evidence/`
  (`proof.log`, `red-team-review.md`, `docstring-parity.txt`). These artifacts are local and
  are not introduced by this docs-only PR; publication remains coordinator-owned.
- [ ] Publish/integrate the correction while preserving main's intervening gdparse self-test,
  obtain exact final-head remote evidence and complete operator merge review.

### E1 — nextest (P1)

Status: implementation and bounded local evidence complete; independent exact-head review PASS.
E1 is not published/merged or fully accepted. Local candidate:
`1562cdcd7147f463ffcb7ed6c6bd30c1a6dfdfa0`, clean `/tmp/mineworld-recover-ci-p1`,
branch `recover/ci-p1-nextest`. The source revision recorded by the evidence fingerprint is
`b191491e72be691a2e97926f0e102781da16f113`; the final candidate adds evidence documentation.
Independent review explicitly targets `1562cdcd` and verifies recorded source fingerprints.
This ledger corrects the source contract's stale review-PENDING / next-send-for-review prose;
those earlier statements do not describe the reviewed candidate's current state.

- [x] Implementation: pinned cargo-nextest and DEP-163; one archive producer per OS;
  weighted archive consumers with revision/root/OS/architecture/toolchain/checksum provenance;
  strict JUnit diagnostics and separate artifacts; workflow producer/consumer/full-gate wiring;
  Cargo-owned doctests/custom harnesses; recursive repeat-wrapper continuation. P0 docstring
  correction and main's gdparse self-test are retained. No shipping Rust/assertion/seed/world edits.
- [x] Bounded deterministic validation: six focused orchestration tests; shard, repeat (11),
  gdparse (7) self-tests; pin/static/workflow graph checks. Real pinned-nextest fixture builds
  once, deletes original outputs, consumes two weighted partitions, matches four nonignored
  direct-list identities, accesses the compiled auxiliary executable and reports failures while
  continuing custom harnesses/doctests. Wrapped repeat executes both samples and fails overall.
  Actual MineWorld archive validation passes 16/16 selected manifest/structure/scratch tests,
  matches direct nextest identities and passes isolated scratch cleanup. This is a bounded
  archive check, not the full workspace producer/consumer suite.
- [x] Independent LLM logic review: PASS for local code/evidence readiness at exact candidate
  `1562cdcd7147f463ffcb7ed6c6bd30c1a6dfdfa0`, report
  `/tmp/mineworld-ci-p1-red-team-review.md`. Reviewer inspected inherited P1/P0 delta against
  main `2b4d36fa`, recovery delta from `d0872869`, and independently reran six focused tests
  plus shard self-test. This review is not publication/merge authorization or full acceptance.
- [ ] Hosted/integration acceptance: Linux container and Windows executions, real GitHub
  artifact transfer/rerun/expiration behavior, full workspace and wall-clock timing suites,
  and required final-head full milestone CI remain NOT RUN. Local macOS and static graph
  evidence do not certify these. No overall E1 delivery or merge completion is recorded.

Evidence packet at exact local P1 candidate:
`.structured-coding/plans/mvp0/ci-p1-recovery-evidence/` in `/tmp/mineworld-recover-ci-p1`.
`source-fingerprint.json` maps source revision and file hashes; `static.log`, `workflow.log`,
`fixture.log`, shard XML, repeat summaries, `mineworld.log`, `mineworld-junit.xml` and
`real-check.py` record commands and observations. This packet belongs to the separate P1
candidate and is not copied or made public by #526. The independent review's substantive
findings and limitations are recorded below so its local path is not the only review record.

Independent review disposition: archive manifests are cleared before builds and written only
on success; consumers reject provenance mismatch. Failed/skipped producers cannot produce a
green full gate. Missing/malformed/empty JUnit cannot turn zero exit into success, and parsed
results cannot clear a nonzero status. Custom harness/doctest and repeat fallbacks retain
failure visibility. The archive fixture exercises auxiliary binaries and removal/extraction,
not just a simulated command plan. No blocking new implementation debt found. Archive
provenance and JUnit diagnostics have separate ownership; partition planning and execution
remain in their existing modules. No inheritance or simulation coupling is introduced.
Inherited large command-table debt remains. Diagnostic retention scales with failures and
is not a hard fixed memory bound. Exact roots/tool versions deliberately trade portability
for explicit refusal; custom-harness inventory fails on changes and requires future audit.

### E1 implementation record — P1 recovery contract (2026-11-22)

Merged verbatim from the P1 branch (headings demoted one level) when origin/main `7718d952` was merged
into the reconstructed P1 branch on 2026-11-27. "Supersedes multi-PR execution above" applied to the
2026-11-22 P1 recovery session only; the E1-E5 split in section 1 remains the lane plan.

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

#### Bounded implementation decisions and audit

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

#### Recovery ledger

- [x] Implementation: continuation, archive producer/consumers, provenance, strict results and pins.
- [x] Deterministic validation: script/static checks and bounded real fixture/MineWorld runs.
- [ ] LLM logic review: local contract/debt audit; independent red-team remains primary-owned.

Current checkpoint: implementation and bounded deterministic validation PASS. No owned heavy job running.
Final source revision `b191491e72be691a2e97926f0e102781da16f113` includes P0 doc-only correction.
Next: send exact-head evidence for independent red-team. Stop for a material contract departure or missing capability
that cannot be resolved within the recorded budget. Read this ledger and actual git/process state on resume.

#### Real fixture Gate specification

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

#### MineWorld Gate specification

Archive and execute `mineworld-packages` targets `manifest`, `structure` and `mineworld-test-support`
target `scratch` using the shipping nextest ci profile, original source/target paths and toolchain 1.97.1.
Compare archived JUnit identity set to direct nextest list; require all selected tests and final scratch
cleanup to pass. This exercises actual source fixtures, compile-time paths and scratch guard behavior,
without a whole-workspace test run. Command driver `/tmp/ci-p1-real-check.py` is retained in evidence.
One slot, expected <3 minutes; each subprocess bounded at 600 seconds, no paid calls.


#### Evidence and debt closeout

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

### E2 — shared baselines (P3)

- [ ] implementation
- [ ] deterministic validation
- [ ] LLM logic review

### E3 — long tier (P2, Q-5)

- [ ] implementation
- [ ] deterministic validation
- [ ] LLM logic review

### E4 — impact job (P4)

- [ ] implementation
- [ ] deterministic validation
- [ ] LLM logic review

### E5 — required check (operator)

- [ ] the operator adds `impact` to the required checks of `main`

## 3. Deviations and discoveries

### 3.1 Evidence reconciliation (2026-11-23)

The public contract at `da9a5d2835d5743978c1a19b7326d62468a69172` had every E1 item
unchecked. The local P1 candidate and independent review now support the three separate
completed local items above. Hosted/full-milestone acceptance remains a separate unchecked
item. E2/P3, E3/P2 and E4/P4 remain unchecked; P5 remains open and excluded from this lane.
E5 remains operator-owned; branch protection has not been changed by this closeout.
Section 1 preserves Q-1 through Q-5, I-TH-1 and the frozen scope verbatim; this is a live
evidence update, not a new design approval or a replacement P1 implementation contract.

Preserved failed attempts: the P1 packet retains initial fixture setup failures (missing
Cargo.lock, then a doc-only shard while the fixture expected JUnit on every shard) and their
bounded fixture corrections. Initial MineWorld tests passed 16/16 but global scratch scan
failed on coordinator-owned retained #530 evidence. It was not deleted; the final check
uses isolated TMPDIR/TMP/TEMP and unchanged production scratch checker. These failed attempts
are not relabeled PASS. Full-workspace and timing claims remain untested.

Deployment limits: archives require identical revision, absolute checkout, OS/architecture,
Rust and nextest identities and default target path because tests compile paths with env!.
Cargo-only fallbacks compile on their owning shards. Two-day retention limits hosted rerun
reuse; expiration is not validated locally. Timing scheduling reserves nextest slots but
does not bound subprocess threads or unrelated host load.

### 3.2 Public CI and remaining coordination

The 2026-11-23 backlog audit records #523 run `40586369072` at `ce69e9e5` and docs-only
#526 run `40587050689` at `da9a5d28` as successful, each with 6 SUCCESS / 11 SKIPPED.
Deferred aggregate gates and skipped matrices are not evidence of suite execution. Those
old-head results do not validate a revised #523 or #526 head, or the local P1 candidate.
Fresh final-head remote evidence remains pending after any authorized publication.

The audited primary design's lifecycle line said both DESIGN FROZEN and "Not frozen"
despite the preserved operator approval record. The coordinator reports a local correction
preserving DESIGN FROZEN 2026-10-27 and distinguishing the docs-only design PR from the
implementation ledger; that parent-doc correction awaits the coordinator's docs commit.
#526 closeout does not edit that file or redefine approval.

This closeout is authorized for local docs edits, cheap docs checks, a semantic local
commit and a prepared PR body only. Independent review by the coordinator/review_sol is
required before publication; it is separate from the completed P1 implementation review.
No public push, PR body edit, merge, workflow dispatch or branch-protection change is
performed. Coordinator owns concrete publication/merge approval and subsequent integration.

### 3.3 Docs-only closeout validation (2026-11-23)

PASS: `python3 scripts/check_doc_headings.py` reports 254 numbered sections across 337
`docs/` documents with no duplicates. A separate contract check covers this planning file:
unique headings, resolved relative Markdown link, byte-identical frozen pre-ledger contract,
unchanged E2-E5 blocks, six supported local completion items and twelve pending items.
`git diff --check` PASS after removing an extra EOF blank line. All thirteen P1 source
fingerprints also match the exact reviewed local candidate. No Cargo/Godot or remote run
was launched for this docs reconciliation. Gate 1 and runtime Gate 2 are N/A to this diff.

Closeout author logic/debt audit: one canonical contract remains; implementation, bounded
validation and independent P1 review have separate evidence; remote acceptance stays open.
Only this planning file changes, with no production/module boundaries, functions, inheritance
or coupling affected. No new maintainability debt found. Independent review of this docs
closeout remains PENDING with the coordinator/review_sol before publication.

### 3.4 E1 reconstruction and publication (2026-11-27)

The local candidate `1562cdcd` and its `/tmp` checkout were lost (~2026-11-26). Branch
`recovered/ci-p1` reconstructs it by deterministic replay of the producer's recorded edit
commands; the published branch is `ci/econ-p1-nextest`. Commit SHAs differ (recovery-time
dates; `Recovered-From:` trailers name the originals).

- PASS (recovery): all 13 files in the reviewed `source-fingerprint.json` match SHA-256
  byte-for-byte; per-file numstat matches the reviewer-captured numstat.
- PASS (re-run during replay and again after merging main `7718d952`): six orchestration
  tests, shard/repeat/gdparse self-tests, pin consistency, doc headings, decision IDs,
  compileall, YAML parse, workflow graph check.
- Evidence packet: `ci-p1-recovery-evidence/` holds 8 of the 15 original files. LOST:
  `fixture.log`, `fixture-shard-{1,2}.xml`, `mineworld-attempt1.log`, `mineworld-junit.xml`,
  `repeat.txt`, `repeat-doc.txt`. The E1 items that cite them remain historical claims backed
  by the independent red-team PASS at `1562cdcd`, not by files in this tree.
- NOT RUN after reconstruction: the real pinned-nextest fixture and the MineWorld archive
  check (the pinned 0.9.148 binary lived in `/tmp` and is gone). Hosted acceptance (the last
  E1 item) stays open and is supplied by the PR's exact-head CI.
- Merge of `7718d952`: the only conflict was this file (main #526 vs the P1 recovery
  ledger). Resolution keeps main's text verbatim and inserts the P1 record as the
  "E1 implementation record" subsection.
