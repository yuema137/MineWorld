# Step 02 / PR 02 — Action, Event, Observation and Spatial contracts

**Role:** combined step and PR document. S2 needs one PR.
**Effort:** `mvp0` · parent: [`overall.md`](overall.md)
**Lifecycle:** `DRAFT`
**Implementation base:** `main` after PR 01 merges (S1 must be merged first; this PR extends the
same crate)

Binding parents: [`overall.md`](overall.md) ·
[`docs/CORE_CONCEPTS.md`](../../../docs/CORE_CONCEPTS.md) ·
[`docs/ENGINEERING_RULES.md`](../../../docs/ENGINEERING_RULES.md) ·
[`docs/ENGINEERING_STANDARDS.md`](../../../docs/ENGINEERING_STANDARDS.md)

---

# 1. Goal

Define the four contracts through which anything outside the world talks to it, plus the spatial
vocabulary both reference clients need:

```text
ActionIntent    what a controller or client requests
ActionResult    what the world answers, including semantic rejections
Event           the immutable fact the world records
Observation     the filtered view a controller is given
Spatial         semantic place, optional continuous position, orientation, requirements
```

`ENGINEERING_RULES.md` §15 requires the spatial contract to exist **before** the 3D demo, and
§§8–9 require that both clients produce the same request for the same interaction. This PR is
where that becomes possible or is foreclosed.

## 1.1 Scope

```text
contracts/src/action.rs       ActionTypeId, Action trait, ActionRecord, ActionIntent,
                              ActionResult, Rejection
contracts/src/event.rs        EventTypeId, Event trait, EventRecord, EventEnvelope,
                              Causation, Visibility, Provenance
contracts/src/spatial.rs      Location, LocalPosition, Orientation, SpatialRequirement
contracts/src/observation.rs  Observation, PerceivedEntity, PerceivedEvent, Affordance
docs/CORE_CONCEPTS.md         §§10-15 gain the spatial vocabulary
contracts/README.md           updated inventory
```

## 1.2 Non-goals

```text
dispatch, validation, resolution — S3 consumes these contracts
the System trait and registry                     → S3
scheduler, process execution                      → S4
persistence of events                             → S5
any concrete action (move, talk, give)            → S6+
perception *systems* deciding what is visible     → S10
network encoding of these types                   → S11
```

This PR defines shapes and their invariants. Nothing here decides an outcome.

## 1.3 Frozen invariants

- `INV-2`: an `ActionIntent` is a request; only an `Event` is a fact. No type here lets an intent
  become state.
- `INV-10`: `ActionResult::Unavailable` is representable as a first-class answer, not an error.
- `INV-12`: no domain action, event, or place is named in this crate. `talk`, `give_item`, `Cafe`
  appear only in test fixtures, and only as opaque identifiers.
- `INV-13`: an `Observation` carries only what the world chose to expose. It has no field that
  could hold the whole world, and no way to reach an entity it does not name.
- `INV-15`: every `Event` states its cause. A causeless event is unrepresentable.
- `ENGINEERING_RULES.md` §12: no mesh, navmesh, camera, scene tree, animation, skeleton or
  physics concept. Spatial types are numbers and identifiers only.
- Determinism (`AC-12`): **no floating point anywhere in these contracts.** Positions and angles
  are fixed-point integers.
- One erasure boundary per family, mirroring `ComponentRecord` from S1: `ActionRecord` and
  `EventRecord`. Typed access goes through the `Action` and `Event` traits.

---

# 2. Design decisions

| ID | Decision | Rationale |
| --- | --- | --- |
| **DD-1** | Action identity splits in two: `ActionTypeId` is the slug a system declares (`talk`), `ActionId` (from S1) is the unique id of one submitted intent, used to correlate the result and the events it caused. | A result must be attributable to the exact request, and an event's `CausedBy` must name it (`INV-15`). |
| **DD-2** | An action's payload is typed through an `Action` trait carrying `ACTION_TYPE` and `OWNER: SystemId`, with `ActionRecord` as the single erasure boundary — the exact pattern S1 used for components. | Ownership at the type level is what lets S3 route an intent to exactly one system and reject an action no enabled system provides. |
| **DD-3** | `Rejection` is a closed kernel enum — `Busy`, `TooFarAway`, `PermissionDenied`, `NoSupportedInteraction`, `TargetUnavailable`, `Unavailable`, `PreconditionFailed` — plus `System { code: RejectionCode, detail }` where `RejectionCode` is a slug owned by the rejecting system. | `ENGINEERING_RULES.md` §8 names five reasons that clients must be able to show, so they are kernel vocabulary. But a system must be able to add its own reason without editing the kernel enum, or every new System Pack amplifies a change into the kernel (standards §8). |
| **DD-4** | `Location` is the semantic position: a `PlaceId`, plus an optional `LocalPosition` and optional `Orientation`. Place hierarchy stays in relations (S1), not in this type. | Semantic space is authoritative; continuous position is an optional refinement a world may not use at all (`ENGINEERING_RULES.md` §5). A headless or abstract world leaves both `None` and loses nothing. |
| **DD-5** | `LocalPosition` is fixed-point: `x`, `y`, `z` as `i32` millimetres relative to the place origin. `Orientation` is `yaw` and optional `pitch` as `i32` millidegrees, normalized on construction. | Floats in the event log would make replay platform-dependent and break `AC-12`. Millimetres and millidegrees are precise enough for embodied movement and exactly reproducible. A 2D client ignores `z` and `pitch`; a 3D client uses them. Neither type mentions an engine. |
| **DD-6** | `SpatialRequirement` is a declaration attached to an action type: `place` (`Any` / `SamePlaceAsActor` / `Specific`), optional `within_range: Millimetres`, `requires_line_of_access: bool`, `requires_target_available: bool`. | §7 of the rules: the requirement belongs to the System contract and no renderer evaluates it. Making it data rather than code means S3 can check the spatial part uniformly and a client can be *told* the requirement without implementing it. |
| **DD-7** | `SpatialRequirement::evaluate` is a pure function in this crate returning `Result<(), Rejection>`, given two `Location`s and a target-availability flag. Line of access is a declared requirement this PR does not evaluate: the evaluator returns `Ok` for it and records that the owning world geometry provider fills it in later. | The spatial arithmetic is deterministic and belongs where both the server and a system can call it once. Pretending to evaluate visibility without geometry would be a fake check; declaring the gap honestly is the alternative. |
| **DD-8** | `EventEnvelope` carries the ten fields of `CORE_CONCEPTS.md` §11. `Causation` is a closed enum: `Action(ActionId)`, `Process(ProcessId)`, `Event(EventId)`, `SystemTick { system: SystemId }`, `WorldGenesis`. | `AC-9` requires every mutation to trace to an action, a process, or a system event. `WorldGenesis` exists so initial state is explainable rather than causeless. |
| **DD-9** | `Visibility` is a closed enum: `Public`, `Place(PlaceId)`, `Participants`, `Entities(BTreeSet<EntityId>)`, `SystemInternal`. | Perception systems in S10 need a declared audience to filter on; an event with no stated audience would default to omniscience, violating `INV-13` by omission. |
| **DD-10** | `Observation` contains: observer, `WorldTime`, the observer's own `Location`, `PerceivedEntity` list, `PerceivedEvent` list, and `Affordance` list. It holds no component store, no entity registry, and no way to look up an entity it did not name. | `INV-13` structurally rather than by convention. |
| **DD-11** | `Affordance` — `action_type`, optional `target`, `available: bool`, `unavailable_reason: Option<Rejection>`, `requirement: SpatialRequirement` — is part of the contract. The server computes it; a client renders it. | This is what lets a 3D client show "press E to talk" and a 2D client grey out a menu entry **without either implementing a rule** (§§4, 8). Without it, clients would inevitably guess, which is the exact failure the rules forbid. |
| **DD-12** | Demos are validated by running Godot with a script that drives input and writes PNG frames, which are then inspected; Godot 4.7.2 is installed and confirmed to run headless with script output. | `ENGINEERING_RULES.md` §19 requires actually running the renderer. Recorded here because it is the reason no engine-visual concept needs to enter these contracts to make verification possible. |

---

# 3. Integration checkpoint

```text
cargo test -p mineworld-contracts
    ├── an intent for an action type no system declares is answerable as Unavailable
    ├── every Rejection variant round-trips, including a system-specific code
    ├── an event cannot be built without a Causation
    ├── Visibility::Entities is order-stable across serializations
    ├── spatial: same place / different place / within range / out of range /
    │            target unavailable each produce the documented Rejection or Ok
    ├── spatial: a Location with no LocalPosition still satisfies a SamePlace requirement
    ├── fixed-point: no f32/f64 appears in the crate (grep-asserted in a test)
    └── an Observation cannot be constructed containing an entity it does not perceive
```

**Adversarial criteria.** The checkpoint fails if: a float appears in any contract type; a client
could compute availability itself from the observation without server input; an event can be
constructed with no cause; the spatial types cannot express "standing 1.2 m from Alice, facing
her, inside the café" **and** "in the café, position irrelevant" with the same `Location` type.

## 3.1 The two gate questions (`ENGINEERING_RULES.md` §§11, 9)

Answered before implementation, re-checked at review:

1. *Can this support a Minecraft-like embodied 3D client without redesigning the kernel?*
   Yes: `LocalPosition` gives continuous millimetre coordinates, `Orientation` gives yaw and
   pitch for look direction, `SpatialRequirement.within_range` gives proximity interaction, and
   `Affordance` gives the "what can I do with the thing I am looking at" query an embodied client
   needs. None of it is tile-shaped.
2. *Can 2D and 3D use the capability without duplicating game logic?*
   Yes: both send `ActionIntent { action_type: talk, target: alice }`. The 2D client obtains the
   target from a click, the 3D client from a camera ray; neither evaluates distance, availability,
   permission, or system presence. `Affordance` carries the server's answer to both.

---

# 4. Test ownership

| Layer | Owns |
| --- | --- |
| Static | formatting, types, lints (`cargo fmt/check/clippy -D warnings`) |
| Unit | rejection round-trips; causation completeness; visibility ordering; fixed-point normalization of orientation; spatial evaluation across the documented case matrix; the no-float structural assertion |
| Real lifecycle | `NOT APPLICABLE` — no runtime yet. First applies in S5/S7 |
| Real model | `NOT REQUIRED` — no model before S10 |
| CI | none (D-9 local-only); canonical evidence is the four commands at the final HEAD |

---

# 5. Commit plan

Each commit: implement → validate → review → record → commit. `[ ]` until done with evidence.

## C1 — Action contracts

**Goal** the request half of the central pipeline, with rejections rich enough for a client to
render and no rule logic anywhere.
**Scope** `contracts/src/action.rs`, module wiring, tests. Depends on S1 merged.

- [ ] Implementation: `ActionTypeId` slug newtype; `Action` trait with `ACTION_TYPE`, `OWNER`, serde bounds; `ActionRecord` erasure boundary with typed conversion both ways.
- [ ] Implementation: `ActionIntent { action_id, actor, action_type, target: Option<EntityId>, payload: ActionRecord, issued_at: WorldTime, actor_location: Option<Location> }`.
- [ ] Implementation: `Rejection` per DD-3 with `RejectionCode` slug; `ActionResult { Accepted { events: Vec<EventId> }, Rejected(Rejection), Unavailable }`.
- [ ] Validation: wrong-type payload decode fails with a named error; every `Rejection` variant round-trips; an `Unavailable` result is constructible without any system present.
- [ ] Review: no validation logic in this module; no domain action named outside tests; `ActionRecord` is the only untyped surface.

**Acceptance** an `ActionIntent` for an undeclared action type is representable and answerable as
`Unavailable`; a system-specific rejection code survives round-trip; §6 commands clean.

## C2 — Event contracts

**Goal** the fact half: immutable, always caused, always with a declared audience.
**Scope** `contracts/src/event.rs`, wiring, tests. Depends on C1.

- [ ] Implementation: `EventTypeId`; `Event` trait (`EVENT_TYPE`, `OWNER`); `EventRecord` erasure.
- [ ] Implementation: `EventEnvelope` with the ten fields; `Causation` (DD-8); `Visibility` (DD-9); `Provenance { emitted_by: SystemId, controller_decision: Option<ActionId> }`.
- [ ] Implementation: constructors that make a causeless or audience-less event unrepresentable.
- [ ] Validation: `Causation` covers action, process, event, tick, genesis and round-trips; `Visibility::Entities` iterates deterministically; an envelope cannot be built without cause or visibility.
- [ ] Review: no event names a domain fact; no mutation helper exists; nothing lets an event be edited after construction.

**Acceptance** every event carries a cause and an audience by construction; §6 clean.

## C3 — Spatial contract

**Goal** the renderer-neutral spatial vocabulary that decides whether an embodied 3D client is
possible later.
**Scope** `contracts/src/spatial.rs`, wiring, tests. Depends on C1.

- [ ] Implementation: `Millimetres(i32)`, `Millidegrees(i32)`; `LocalPosition { x, y, z }`; `Orientation { yaw, pitch: Option<_> }` normalized to a canonical range on construction.
- [ ] Implementation: `Location { place: PlaceId, local: Option<LocalPosition>, facing: Option<Orientation> }`.
- [ ] Implementation: `SpatialRequirement` per DD-6, with `NONE` and `same_place()` constructors for the common cases.
- [ ] Implementation: `evaluate(requirement, actor: &Location, target: Option<&Location>, target_available: bool) -> Result<(), Rejection>`, integer distance only, documenting the line-of-access gap per DD-7.
- [ ] Validation: the case matrix — same place / different place → `TooFarAway` or pass per requirement; in range / out of range; absent `LocalPosition` with a range requirement (documented outcome, not a panic); unavailable target → `TargetUnavailable`; orientation normalization at ±360° boundaries; distance arithmetic cannot overflow `i32` at world scale.
- [ ] Validation: structural test asserting the crate source contains no `f32`/`f64`.
- [ ] Review: nothing engine-shaped; a 2D world ignoring `z` and a 3D world using it share one type; the §11 gate answer in §3.1 still holds against the written code.

**Acceptance** both "1.2 m from Alice facing her inside the café" and "in the café, position
irrelevant" are one type; the evaluator is pure, integer-only and total; §6 clean.

## C4 — Observation contract

**Goal** the view a controller gets, shaped so omniscience is impossible and affordances come
from the server.
**Scope** `contracts/src/observation.rs`, wiring, tests. Depends on C1–C3.

- [ ] Implementation: `PerceivedEntity { id, entity_type, location: Option<Location>, tags, components: Vec<ComponentRecord> }` — only what a perception system chose to include.
- [ ] Implementation: `PerceivedEvent` wrapping an `EventEnvelope` the observer is entitled to.
- [ ] Implementation: `Affordance` per DD-11.
- [ ] Implementation: `Observation { observer, at, self_location, entities, events, affordances }` with no store handle and no global accessor.
- [ ] Validation: an observation exposes exactly the entities it lists; affordances carry a server-computed reason; round-trip and ordering stability.
- [ ] Review: no field or method could return an entity not perceived; nothing here lets a client recompute availability.

**Acceptance** `INV-13` holds structurally; a client can render "press E to talk" purely from
`Affordance`; §6 clean.

## C5 — Specification synchronization

- [ ] `docs/CORE_CONCEPTS.md`: add the spatial vocabulary to §§6 and 15 and the affordance concept to §15, cross-referencing `ENGINEERING_RULES.md` §§5–9.
- [ ] `contracts/README.md`: updated inventory, still short and human-facing.
- [ ] This document: ledger, evidence, deviations.
- [ ] Review: no specification statement is now false; links resolve.

---

# 6. Verification commands

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt --all --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p mineworld-contracts
```

---

# 7. Ledger

## 7.1 Progress
```text
C1 not started   C2 not started   C3 not started   C4 not started   C5 not started
```

## 7.2 Evidence
```text
(none yet)
```

## 7.3 Findings, decisions, deviations
```text
(none yet)
```

---

# 8. Review and approval

Reviewed against the frozen specifications by the primary session acting as reviewer under the
operator's autonomous-execution authorization of 2026-09-25.

```text
CHECKED  INV-2, INV-10, INV-12, INV-13, INV-15 are expressed as structural properties
         of the types, not as future runtime checks                            → §1.3, §3
CHECKED  ENGINEERING_RULES §§4-9 land in the contract: the requirement is data, the
         evaluator is server-side, the affordance is server-computed            → DD-6, DD-7, DD-11
CHECKED  ENGINEERING_RULES §11 gate — embodied 3D is expressible                → §3.1
CHECKED  ENGINEERING_RULES §12 gate — no engine concept, no float               → §1.3, DD-5
CHECKED  AC-9 causality and AC-12 determinism have concrete mechanisms          → DD-5, DD-8
CHECKED  scope adds one crate's four modules; no unrelated module changes       → §1.1
CHECKED  S1 review criteria from overall.md are respected: this PR, not S1, is where
         spatial types arrive
OPEN     line of access is declared but not evaluated until geometry exists (DD-7),
         recorded as a known limitation rather than a silent pass
```

APPROVED for implementation. Base: `main` once PR 01 is merged.
