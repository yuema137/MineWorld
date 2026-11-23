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
