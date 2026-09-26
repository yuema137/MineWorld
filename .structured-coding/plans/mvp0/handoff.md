# Handoff — active implementation context

**Active PR:** Step 01 / PR 01 — Entity and Component contracts
**Effort:** `mvp0`
**Primary design doc (semantic authority):**
`.structured-coding/plans/mvp0/step-01-entity-component-contracts.md`
**Binding parents:** `.structured-coding/plans/mvp0/overall.md`, `docs/CORE_CONCEPTS.md`,
`docs/ARCHITECTURE.md`, `docs/ENGINEERING_STANDARDS.md`, `CLAUDE.md`
**Execution contract:** §9 of the primary design doc

## Repository identity

```text
repository       /Users/yuema137/MineWorld
branch           mvp0/pr-01-entity-component-contracts
implementation   created from main @ 39abfb3
base
current HEAD     39abfb3
working tree     clean except this handoff (untracked at initialization)
remote           none (decision D-9: local-only until S13)
```

## Environment

```text
toolchain   rustup stable-aarch64-apple-darwin, rustc/cargo 1.97.1,
            rustfmt 1.9.0, clippy 0.1.97
PATH        ~/.cargo/bin is NOT on the default non-interactive PATH.
            Every command must start with: export PATH="$HOME/.cargo/bin:$PATH"
            A bare "command not found" is a PATH defect in the session, never a
            failed check.
```

## Approved scope

The `mineworld-contracts` crate and the workspace that holds it: identity, entity record,
component model, relation model, plus the `docs/ARCHITECTURE.md` §13 amendment for decision
D-4 and a rewritten `contracts/README.md`. Full statement in §1.1 of the design; non-goals in
§1.2.

## Frozen invariants

```text
no domain concept in mineworld-contracts (INV-12)
component ownership is type-level and inseparable from the component type
exactly one documented payload-erasure boundary: ComponentRecord
BTreeMap / BTreeSet / Vec only — HashMap / HashSet forbidden in this crate
no random or time-derived identity; serialization is byte-deterministic
every contract type is serde-serializable
no dependency on kernel/, systems/, clients/, cognition/
forbidden here: storage, registry, dispatch, scheduling, persistence, and any
ActionIntent / Event / Observation payload
```

## Endpoint authority

```text
implementation + local validation   authorized   (operator, 2026-09-25)
semantic commits                    authorized   (operator's commit sequence)
branch push                         NOT authorized (decision D-9)
PR creation / remote CI             N/A — no remote exists (decision D-9)
merge into main                     explicit operator authorization only
```

## Implementation sequence

```text
C1  Cargo workspace + identity module
C2  Entity record and lifecycle
C3  Component model
C4  Relation model
C5  Specification synchronization
```

## Current checkpoint

```text
C1  not started
C2  not started
C3  not started
C4  not started
C5  not started

validation evidence recorded so far: none
background jobs: none
```

## Exact next actions

1. Read the primary design doc in full, then `prompts/implementation-working-rules.md` and
   `prompts/test-ci-gate-rules.md` in the installed skill.
2. Inspect `docs/CORE_CONCEPTS.md` §§3–4, 9, 13 against the C1 plan before writing types.
3. Implement C1's checklist, run §6's commands, record evidence in §8.2, then commit.

## Stop conditions

```text
STOP and report when:
  a frozen invariant in §1.3 would have to change
  the single-writer precondition cannot be expressed as designed (risk R-1)
  a public specification statement is falsified
  the PR reaches READY FOR OPERATOR REVIEW

NORMAL STOP: PR 01 ready for operator review on
mvp0/pr-01-entity-component-contracts — DO NOT MERGE.
```
