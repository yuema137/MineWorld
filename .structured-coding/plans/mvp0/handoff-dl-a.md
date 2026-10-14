# Handoff — PR DL-a (S24: the decision list)

A continuation aid only. The authority is `pr-dl-a-decision-list.md` (design §§1–12, contract §13, rulings
§14.1, ledger §15).

```text
PROJECT / PR         MVP-0 · S24 / PR DL-a — the decision list: decisions.yaml, the DecisionModel trait,
                     cognition/installed, host wiring, the `mineworld decisions --json` subcommand
PRIMARY DESIGN DOC   .structured-coding/plans/mvp0/pr-dl-a-decision-list.md
RELATED / BINDING    mvp1/step-24-decision-and-memory.md §§3, 4, 5, 10.2, 11.2, 12, 13, 14.1; overall.md
                     2026-10-12; pr-fg-b-retention-host.md §6 (memory: shape, MemoryPolicyChoice); DECISIONS
                     ARC-23, 25, 27, 33, 53, 61, 64, 84, 85, DEP-10, DEP-12; MODULE_SPEC §§4.1, 5, 8.1;
                     ENGINEERING_STANDARDS; ENGINEERING_RULES; CLAUDE.md
BRANCH / WORKTREE    mvp0/pr-dl-a · /Users/yuema137/mineworld-worktrees/impl-dl-a (one writer: this session)
BASE                 origin/main @ 6306977 (#177, DL-a/DL-b freeze merge); DOC-24 (#175) on it
FROZEN               DESIGN FROZEN 2026-10-12 (primary session, operator standing instruction); operator's
                     go 2026-10-13
APPROVED SCOPE       §3.1 (including 5a and the decisions subcommand)
FROZEN INVARIANTS    I-DLa-1 … I-DLa-7; SD-DLa-1 … SD-DLa-15; SD-DLa-15's JSON shape (format 1)
ENDPOINT AUTHORITY   implementation + local validation, semantic commits, branch push, PR creation/update,
                     CI repair: authorized (§13, primary session 2026-10-12; operator's go 2026-10-13).
                     Merge: never by this session
VALIDATION BUDGET    unit/integration unrestricted; one long parity record at C7 (≤ 20 min); full local
                     `cargo test --workspace --no-fail-fast` and the Python suites before any push that
                     changes tests (operator's kickoff instruction, 2026-10-13); CI: the PR's runs
STOP                 PR DL-a READY FOR OPERATOR REVIEW — DO NOT MERGE; material stops per §13
```

## Lifecycle: IMPLEMENTING

## Goldens captured on the base (before any code change, test rules §25)

- `Draw` goldens: 32 tuples plus 4 `mix` values from `paced.rs`'s private `Draw` at `6306977`, via a
  temporary `#[cfg(test)]` module inside `paced.rs` (removed, never committed). Frozen in
  `cognition/decision/src/draw_tests.rs` (C2).
- `run worlds/social-cafe --headless --seed 7 --days 30` output minus the `wall` line, from the base binary:
  `tools/cli/tests/fixtures/decisions/social-cafe-seed-7-30-days.txt` (C5).
- `mineworld packs list` and `validate` output of the base binary, kept in scratch for AC-DLa-9 comparison.

## Next actions

- C1 specifications, then C2 … C7 in order (design §10).
