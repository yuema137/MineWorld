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

### E1 — nextest (P1)

- [ ] implementation
- [ ] deterministic validation
- [ ] LLM logic review

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

(none yet)
