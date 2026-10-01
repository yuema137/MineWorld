# Handoff — PR 06 implementation context: ACTIVE

**Active PR:** Step 04 / PR 06 — World clock, scheduler, and Process
**Effort:** `mvp0`
**Primary design doc (semantic authority):**
`.structured-coding/plans/mvp0/step-04-clock-scheduler-process.md` (combined step/PR document)
**Execution contract:** §10 of the primary design doc
**Binding parents:** `overall.md` §§2, 7; `docs/CORE_CONCEPTS.md` §§2, 10, 11, 13;
`docs/ARCHITECTURE.md` §5; `docs/DECISIONS.md` `DEP-6`, `ARC-15`, `ARC-23`;
`docs/ENGINEERING_RULES.md`; `docs/ENGINEERING_STANDARDS.md`; `CLAUDE.md`

PR 05d's handoff, which stood here until now, is superseded: that PR is merged (GitHub #7) and its
record lives in `pr-05d-clients-ac15.md`.

## Repository identity

```text
worktree         /Users/yuema137/mineworld-worktrees/s4-scheduler — this session's only; the
                 sibling worktrees vis-character and vis-environment belong to other agents
branch           mvp0/pr-06-scheduler
base             main @ 7cf8844 — 259 tests green there (32 s)
current HEAD     `git log --oneline` is authoritative — a commit cannot carry its own hash
remote           origin; push after every coherent step (the network has been intermittent)
```

## Environment

```text
PATH   export PATH="$HOME/.cargo/bin:$PATH"   — ~/.cargo/bin is not on the non-interactive PATH
```

## Scope, invariants, endpoints

See the primary design doc §§1.1, 1.3, 8 and 10. Endpoints: implement, commit, push and open the PR
are authorized; merge is not.

## Current checkpoint

```text
C0  re-audit, rename, contract     done
C1  clock and queue                next
C2  instants, server integration   pending
C3  processes and interruption     pending
C4  long run and documentation     pending
```

## Exact next actions

1. C1: `kernel/src/clock.rs`, `kernel/src/schedule.rs`; `World` owns both; `Deferral` captures its
   cause; dispatch refuses backwards time and outstanding work, then queues its deferrals.
2. Targeted tests, ledger, commit, push.

## Stop conditions

Material deviation per §10 of the primary doc; otherwise READY FOR OPERATOR REVIEW — DO NOT MERGE.
