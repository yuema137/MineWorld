# Handoff — PR 01 implementation context: CLOSED / AWAITING OPERATOR ACTION

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
current HEAD     the C5 commit — the one that carries this handoff refresh
                 (C1 = e040818, C2 = 80fdf2c, C3 = 40e448e, C4 = 868cdc3, C4b = 90bb755)
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
C4  DONE  868cdc3  src/relation.rs (EntityTypeSet, RelationDirection, SelfEdges,
                   RelationTypeDeclaration, RelationEnd, Relation), three relation errors,
                   tests/relation.rs, sixth compile-fail case
C4b DONE  90bb755  src/time.rs (WorldTime, SimDuration), tests/time.rs
C4  not started
C5  DONE           docs/ARCHITECTURE.md §13 row + new §13.1 recording decision D-4,
                   contracts/README.md rewritten, design ledger closed

validation evidence: §8.2 of the design — all four §6 commands clean at C1…C4b;
                     32 tests pass (9 identity + 6 entity + 6 component + 6 relation
                     + 3 time + 1 trybuild harness over 6 cases + 1 doc-test);
                     five mutations confirmed the HashMap ban, the unchecked-construction
                     compile-fail case, the terminal Destroyed state, the ownership conflict
                     check and the canonical ordering of undirected edges
assumptions:         A-1 and A-2 resolved in C1 (§8.4); A-3 still carried to S4
background jobs:     none
```

## State

```text
PR 01 is READY FOR OPERATOR REVIEW on mvp0/pr-01-entity-component-contracts. DO NOT MERGE.

implementation   complete — six commits, §7 fully checked off
validation       §6's four commands clean at the final head; 32 tests pass
documentation    design ledger §§7-8 synchronized; ARCHITECTURE.md §13.1 records D-4;
                 contracts/README.md describes the real crate
not done         no push (no remote exists — decision D-9), no PR, no remote CI, no merge
```

## What the operator decides next

1. Review the branch. Two execution-time deviations change the shape of the PR and are argued in
   §8.3 of the design: the added **C4b** commit (because §1.1 approves `WorldTime`/`SimDuration`
   and the frozen §7 plan never implements them) and the **component payload as a type
   parameter** (because encoding it here would mean a new dependency and would pre-empt D-4).
2. Authorize or refuse the merge. Nothing in this branch has been merged.
3. On merge, three statements that describe `main` as containing no Rust code become stale and
   are updated in one commit — `README.md:35`, `CLAUDE.md:43`,
   `.structured-coding/standards.md:15` and its note about the cargo checks. Listed as
   follow-ups in §8.3.

## If this context is resumed instead

The next PR starts a fresh implementation context with its own filled contract (S2 —
`ActionIntent` / `Event` contracts). Do not continue that work here.

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
