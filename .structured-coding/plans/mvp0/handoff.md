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
current HEAD     the C4 commit — the one that carries this handoff refresh
                 (C1 = e040818, C2 = 80fdf2c, C3 = 40e448e)
                 (session start was f44b7d1; `git log --oneline` is authoritative)
working tree     clean at each commit; the only untracked path is target/ (gitignored)
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
C4b World time value types  — added during C1, see the scope-reconciliation finding in
                              §8.3 of the design: §1.1 approves WorldTime/SimDuration but
                              §7's five commits never implement them
C5  Specification synchronization
```

## Current checkpoint

```text
C1  DONE  e040818  workspace, rust-toolchain.toml pinned to 1.97.1, clippy.toml denying
                     HashMap/HashSet, crate mineworld-contracts with src/lib.rs,
                     src/error.rs, src/ids.rs, tests/identity.rs, tests/compile_fail{.rs,/}
C2  DONE  80fdf2c  src/entity.rs (Tag, Tags, LifecycleState, Metadata, Entity),
                   IdentifierKind::Tag, ContractError::IllegalLifecycleTransition,
                   tests/entity.rs, third compile-fail case
C3  DONE  40e448e  src/component.rs (Component, ComponentDeclaration, ComponentRecord,
                   ComponentSchemaVersion), const-checked SystemId/ComponentTypeId
                   literals via from_static, three component errors, tests/component.rs,
                   two more compile-fail cases
C4  DONE           src/relation.rs (EntityTypeSet, RelationDirection, SelfEdges,
                   RelationTypeDeclaration, RelationEnd, Relation), three relation errors,
                   tests/relation.rs, sixth compile-fail case
C4b next
C4  not started
C5  not started

validation evidence: §8.2 of the design — all four §6 commands clean at C1…C4;
                     29 tests pass (9 identity + 6 entity + 6 component + 6 relation
                     + 1 trybuild harness over 6 cases + 1 doc-test);
                     five mutations confirmed the HashMap ban, the unchecked-construction
                     compile-fail case, the terminal Destroyed state, the ownership conflict
                     check and the canonical ordering of undirected edges
assumptions:         A-1 and A-2 resolved in C1 (§8.4); A-3 still carried to S4
background jobs:     none
```

## Exact next actions

1. C4b — `contracts/src/time.rs`: `WorldTime` and `SimDuration` as ordered value types with
   serde and no scheduling policy, which is all §1.1 approves (A-3 stays open for S4). See the
   scope-reconciliation finding in §8.3 for why this commit exists.
2. C5 — documentation: `docs/ARCHITECTURE.md` §13 amended for decision D-4,
   `contracts/README.md` rewritten for what now exists, final ledger entries.
3. Run §6's four commands at the final head, record evidence in §8.2, refresh this file, set the
   lifecycle to READY FOR OPERATOR REVIEW and mark the implementation context CLOSED.

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
