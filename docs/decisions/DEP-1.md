## DEP-1 — Component storage: purpose-built, not an ECS

**Date** 2026-09-25 · **Supersedes** effort decision `D-5`, which is now this record

**Problem.** Store typed component state keyed by entity, where each component type is owned by
exactly one system, systems are installed and removed per world, all state is serializable, and
the whole store is reconstructible from an append-only event log.

**Options considered.** `bevy_ecs`, `hecs`, `shipyard`, `legion`, `specs`, `edict`; or a
purpose-built typed store.

**Choice: purpose-built**, one component table per owning system, keyed by `EntityId`.

**Why not an ECS.** Three mismatches, in order of severity.

1. *Identity.* Almost every Rust ECS uses generational handles that are not stable across a
   save/load cycle; `bevy_ecs` is the notable exception with globally unique ids. MineWorld's
   `EntityId` must be stable, serializable, and referenced by events that outlive the process
   ([`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §3, `AC-6`).
2. *What they optimize.* Archetype storage buys cache-friendly iteration over tens of thousands
   of entities per frame. MineWorld ticks semantically over hundreds of entities and spends its
   budget on causality, persistence and migration instead. We would pay the complexity and gain
   nothing measurable.
3. *Ownership.* `INV-7` requires that a system can only write components it owns. An ECS hands
   any system `&mut World`, so single-writer would degrade to a review convention — the exact
   guarantee the composability claim rests on
   ([`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §15).

Dynamic registration is a fourth, smaller mismatch: System Packs declare component types at load
time, which archetype ECSs support awkwardly, and serialization of dynamically registered types
generally needs a reflection layer we would then also own.

**Why not ourselves is weaker here.** This is `REUSE_POLICY.md` §7's third case: study the
implementations, build the smaller MineWorld-specific version. The store is a `BTreeMap` per
component type behind a narrow API, not a subsystem.

**Isolating interface.** The kernel's component-store API. If entity counts ever make iteration
the bottleneck, the internals can be replaced — including by an ECS — without touching systems,
because systems never see the storage.

**Accepted limitations.** We maintain this code. Iteration performance is far below an archetype
ECS, which is acceptable at MVP scale and is the thing to re-measure before it is not. No
built-in parallel scheduling; determinism (`AC-12`) currently matters more than parallelism.

**Licenses.** All candidates are MIT/Apache-2.0 and would have been compatible.
