# Handoff — PR 07 implementation context: ACTIVE

**Active PR:** Step 06 / PR 07 — Persistence and event sourcing (S5)
**Effort:** `mvp0`
**Primary design doc (semantic authority):** `.structured-coding/plans/mvp0/step-06-persistence.md`
(combined step/PR document, `DESIGN FROZEN` 2026-09-30, answers in §10.1)
**Execution contract:** §11 of the primary design doc
**Binding parents:** `overall.md` §§2, 3 (S5), 7; `docs/CORE_CONCEPTS.md` §§2, 3, 11, 13;
`docs/ARCHITECTURE.md` §§7, 8, 14; `docs/MVP.md` §9, §9.1; `docs/NETWORKING.md` §10;
`docs/DECISIONS.md` `DEP-1`, `DEP-2`, `DEP-5`, `DEP-6`, `ARC-15`, `ARC-23`, `ARC-25` (from C1);
`docs/ENGINEERING_RULES.md`; `docs/ENGINEERING_STANDARDS.md`; `CLAUDE.md`

PR 06's handoff, which stood here, is superseded: PR 06 is merged (`1241cab`) and its record lives in
`step-04-clock-scheduler-process.md`.

## Repository identity

```text
worktree         /Users/yuema137/mineworld-worktrees/s5-persistence — this session's only; the
                 sibling worktrees vis-character and vis-environment belong to other agents
branch           mvp0/pr-07-persistence
base             main @ 5f02332 — 294 tests green there
current HEAD     `git log --oneline` is authoritative
remote           origin; push after every coherent step (crates.io and the network are intermittent)
```

## Scope, invariants, endpoint authority

Approved scope: design §1.1 as answered in §10.1. Frozen invariants: §1.3 I-1 … I-9. Endpoint
authority: §11 — implementation, commits, push and PR creation authorized; CI N/A; **merge only with
explicit operator authorization**.

## Sequence and current checkpoint

```text
C0  design                                         done (2dc8a7f), frozen (910c1f4)
C1  spec amendments (ARC-25, DEP-2/5, ARCH, PROTO)  in progress
C2  kernel WorldSnapshot / restore
C3  mineworld-persistence
C4  worldpack split + process-kill checkpoint (IC-1)
C5  server + CLI --save, revision, restart test (IC-2), AC-15 line four
C6  docs, ledger close, terminal gates, PR
```

## Validation budget and stop conditions

Unit/integration unrestricted; real-model NOT REQUIRED; process-kill and restart tests well under a
minute in total. Normal stop: PR 07 READY FOR OPERATOR REVIEW — DO NOT MERGE. Material stop: any
change to §1.3, an existing public contract beyond §1.1, ownership or scope.

## Exact next action

Finish C1, commit, push; then C2 starting from `kernel/src/components.rs` (`ComponentRows`).
