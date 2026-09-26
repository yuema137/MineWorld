# Step 02 / PR 02 — Action, Event, Observation and Spatial contracts

**Role:** combined step and PR document. S2 needs one PR.
**Effort:** `mvp0` · parent: [`overall.md`](overall.md)
**Lifecycle:** `FROZEN — IN IMPLEMENTATION`
**Implementation base:** `main` @ `c8f2934` (PR 01 merged at `e85c889`; this PR extends the same
crate)
**Implementation branch:** `mvp0/pr-02-action-event-spatial`

## DESIGN FROZEN

```text
Design revision:      §§1-5 of this document as approved in §8 (2026-09-25)
Approved by:          operator, 2026-09-25 — autonomous-execution authorization for PR 02
Implementation base:  main @ c8f2934
Execution contract:   the operator's PR 02 execution kickoff, recorded in §9
Lifecycle:            FROZEN — scope (§1.1), non-goals (§1.2), invariants (§1.3), design
                      decisions (§2), acceptance (§3) and test ownership (§4) are frozen;
                      §7's ledger stays live
```

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
| **DD-13** | `Affordance` carries `action_type` and `target`, not display text. The client maps an action type to a label ("talk" → "Talk") and reads the target's name from a component in the same `Observation`. | Label text and localization are presentation concerns, so putting them in the contract would pull presentation into the kernel. But the *name* of a target is world data and must come from a component, or the client would invent it. Established by the 3D spike below, which needed exactly `entity_id → name → action label` to render `[E] Talk to Alice`. |
| **DD-14** | `PerceivedEntity` must carry enough for a client to bind a rendered body to a world entity and back: the `EntityId` is the join key, and the client attaches it to its own node. The contract says nothing about how. | The spike attached `entity_id` as node metadata and recovered it from a raycast collider. That pattern works for any engine and needs no contract support beyond the id already being in the observation — confirming no engine concept has to enter the contract (`ENGINEERING_RULES.md` §12). |
| **DD-15** | Contract types keep integer ids. The **JSON wire encoding is S11's responsibility** and must render 64-bit ids as decimal strings. Recorded here so S11 inherits it as a requirement rather than rediscovering it. | A networking spike found that Godot's `JSON.parse_string` returns every number as a double: the server sent `"events":[9001]` and the client read `9001.0`. Ids above 2^53 would corrupt silently. The wrong fix is to make the contract serialize ids as strings — that would distort persistence and any binary encoding to suit one client's parser. The right fix is a wire-level representation at the protocol boundary, which is exactly what a presentation-independent architecture is for. |
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

- [x] Implementation: `ActionTypeId` slug newtype; `Action` trait with `ACTION_TYPE`, `OWNER`, serde bounds; `ActionRecord` erasure boundary with typed conversion both ways.
      → `contracts/src/action.rs`. `ActionTypeId` mirrors `SystemId`: `Cow<'static, str>`, `new`
      validating, `const from_static` checked while the declaring crate compiles. `Action` declares
      `ACTION_TYPE` and `OWNER` and nothing else — see the schema-version finding in §7.3.
      `ActionRecord<P = Vec<u8>>` follows `ComponentRecord<P = Vec<u8>>` exactly, minus the entity
      (a request is not state attached to one) and minus the schema version.
- [x] Implementation: `ActionIntent { action_id, actor, action_type, target: Option<EntityId>, payload: ActionRecord, issued_at: WorldTime, actor_location: Option<Location> }`.
      → `actor_location` lands in C3 with the `Location` type it needs; deviation D-1 in §7.3. Every
      other field is in C1. The envelope's `action_type` is derived from the payload record in `new`
      and re-checked on deserialization, so the two can never disagree (§7.3, decision K-1).
- [x] Implementation: `Rejection` per DD-3 with `RejectionCode` slug; `ActionResult { Accepted { events: Vec<EventId> }, Rejected(Rejection), Unavailable }`.
      → both as designed. `Rejection::Unavailable` and `ActionResult::Unavailable` are documented as
      two different statements and pinned as distinguishable on the wire (§7.3, decision K-2).
- [x] Validation: wrong-type payload decode fails with a named error; every `Rejection` variant round-trips; an `Unavailable` result is constructible without any system present.
      → `contracts/tests/action.rs`, 5 tests; three mutations confirmed the guards are load-bearing.
      Evidence in §7.2.
- [x] Review: no validation logic in this module; no domain action named outside tests; `ActionRecord` is the only untyped surface.
      → `grep -niE 'mesh|navmesh|camera|scene|animation|skeleton|physics|collider|viewport'` and
      `grep -n 'f32|f64'` over `contracts/src/*.rs`: no hits. `talk`, `give_item` and `shoot` occur
      in `action.rs` only inside prose that names them as *System Pack* vocabulary the kernel must
      not know — the same way `entity.rs` already names `cafe` as an example tag. No type, field,
      constant or identifier in `src/` is named after a domain concept. The only rule-like code in
      the module is identifier well-formedness and the envelope/payload agreement check; there is no
      permission, distance, availability or system-presence logic. `ActionRecord` is the module's
      only generic payload.

**Acceptance** an `ActionIntent` for an undeclared action type is representable and answerable as
`Unavailable`; a system-specific rejection code survives round-trip; §6 commands clean.

## C2 — Event contracts

**Goal** the fact half: immutable, always caused, always with a declared audience.
**Scope** `contracts/src/event.rs`, wiring, tests. Depends on C1.

- [x] Implementation: `EventTypeId`; `Event` trait (`EVENT_TYPE`, `OWNER`); `EventRecord` erasure.
      → `contracts/src/event.rs`, following the `Action` and `Component` pattern exactly. No schema
      version, as frozen; the consequence is open item O-1 in §7.3.
- [x] Implementation: `EventEnvelope` with the ten fields; `Causation` (DD-8); `Visibility` (DD-9); `Provenance { emitted_by: SystemId, controller_decision: Option<ActionId> }`.
      → all ten fields of `CORE_CONCEPTS.md` §11 are present. §11's `Location` is `place:
      Option<PlaceId>`, not a `Location`; decision K-3 in §7.3 argues why. `Subjects` and
      `Participants` are `Vec<EntityId>`, not sets, because for a two-ended fact the order is part
      of the fact; `Visibility::Entities` is the `BTreeSet` DD-9 specifies.
- [x] Implementation: constructors that make a causeless or audience-less event unrepresentable.
      → `EventEnvelope::new` takes identity, time, payload, `Causation`, `Visibility` and
      `Provenance` positionally; the four optional facts are consuming `about` / `with_participants`
      / `at_place` builders. Neither field is an `Option`, so a record missing either fails to
      deserialize as well — the guarantee holds for a log read from disk, not only one built in
      memory.
- [x] Validation: `Causation` covers action, process, event, tick, genesis and round-trips; `Visibility::Entities` iterates deterministically; an envelope cannot be built without cause or visibility.
      → `contracts/tests/event.rs`, 6 tests, including the same `BTreeSet` filled in ascending and
      descending order serializing byte-identically. Two mutations in §7.2.
- [x] Review: no event names a domain fact; no mutation helper exists; nothing lets an event be edited after construction.
      → `grep -n '&mut self\|pub [a-z_]*:' contracts/src/event.rs`: no hits, so no accessor hands
      out a mutable reference and no field is public. `ItemTransferred` and `ConversationStarted`
      appear in `event.rs` only in the sentence that names them as System Pack vocabulary the kernel
      must not know. The assignment path is pinned shut by a new compile-fail case,
      `tests/compile_fail/event_cannot_be_edited_after_construction.rs`, whose `.stderr` records
      that both `caused_by` and `visibility` are refused as private fields — the same mechanism S1
      used for an entity's lifecycle.

**Acceptance** every event carries a cause and an audience by construction; §6 clean.

## C3 — Spatial contract

**Goal** the renderer-neutral spatial vocabulary that decides whether an embodied 3D client is
possible later.
**Scope** `contracts/src/spatial.rs`, wiring, tests. Depends on C1.

- [x] Implementation: `Millimetres(i32)`, `Millidegrees(i32)`; `LocalPosition { x, y, z }`; `Orientation { yaw, pitch: Option<_> }` normalized to a canonical range on construction.
      → `contracts/src/spatial.rs`. Yaw is canonicalized into `[0, 360_000)` by `rem_euclid`, in a
      `const fn`, so `Orientation::facing` is usable in a constant. Pitch is **rejected** outside
      ±90 000 rather than clamped; decision K-4 in §7.3 separates the two cases.
- [x] Implementation: `Location { place: PlaceId, local: Option<LocalPosition>, facing: Option<Orientation> }`.
      → with `in_place` plus `with_local` / `with_facing`, the S1 builder pattern. Also added
      `ActionIntent.actor_location` and `from_location` here, closing deviation D-1.
- [x] Implementation: `SpatialRequirement` per DD-6, with `NONE` and `same_place()` constructors for the common cases.
      → plus `at_place`, `within` (fallible — a negative range is refused where it is declared),
      `requiring_line_of_access` and `requiring_target_available`. `PlaceRequirement` is the
      `Any` / `SamePlaceAsActor` / `Specific` enum DD-6 names.
- [x] Implementation: `evaluate(requirement, actor: &Location, target: Option<&Location>, target_available: bool) -> Result<(), Rejection>`, integer distance only, documenting the line-of-access gap per DD-7.
      → a method on `SpatialRequirement`. Squared distances are compared in `i128`; there is no
      square root, so nothing rounds. The documented precedence is availability → place → range →
      line of access, and the doc comment argues each. The line-of-access gap is stated on the
      method and in the code at the point where a geometry provider will eventually answer it.
- [x] Validation: the case matrix — same place / different place → `TooFarAway` or pass per requirement; in range / out of range; absent `LocalPosition` with a range requirement (documented outcome, not a panic); unavailable target → `TargetUnavailable`; orientation normalization at ±360° boundaries; distance arithmetic cannot overflow `i32` at world scale.
      → `contracts/tests/spatial.rs`, 8 tests. Distances use a 3-4-5 triangle scaled by 400, so the
      2 000 mm boundary and the 2 001 mm miss are checkable on paper. The overflow case evaluates
      between `i32::MIN` and `i32::MAX` on all three axes. Five mutations in §7.2, including one
      that survived and what it exposed.
- [x] Validation: structural test asserting the crate source contains no `f32`/`f64`.
      → `no_floating_point_appears_anywhere_in_the_contract_crate` walks `contracts/src/`, strips
      line comments and fails on either token in code, and asserts it actually scanned something so
      that a broken path cannot pass silently. Mutation M6 (adding an `as_metres() -> f64`
      convenience, the most likely way a float would really arrive) turned it red.
- [x] Review: nothing engine-shaped; a 2D world ignoring `z` and a 3D world using it share one type; the §11 gate answer in §3.1 still holds against the written code.
      → `grep -niE 'mesh|navmesh|camera|scene|animation|skeleton|physics|collider|viewport|transform|godot|unreal|shader|texture|sprite'` over `contracts/src/`: four hits, all in `spatial.rs`
      prose that names those concepts as the ones that must *not* be here. No code hit, and no
      `sqrt`, `powi` or `as f…` anywhere. One `Location` type carries both the embodied and the
      purely semantic case, asserted by
      `one_location_type_describes_both_an_embodied_and_a_purely_semantic_position`. §3.1 re-checked
      against the written code in §7.4.

**Acceptance** both "1.2 m from Alice facing her inside the café" and "in the café, position
irrelevant" are one type; the evaluator is pure, integer-only and total; §6 clean.

## C4 — Observation contract

**Goal** the view a controller gets, shaped so omniscience is impossible and affordances come
from the server.
**Scope** `contracts/src/observation.rs`, wiring, tests. Depends on C1–C3.

- [x] Implementation: `PerceivedEntity { id, entity_type, location: Option<Location>, tags, components: Vec<ComponentRecord> }` — only what a perception system chose to include.
      → `contracts/src/observation.rs`, generic over the payload encoding like every other record
      holder in the crate. Documented as *not* a copy of the entity: two observers of one entity can
      be shown different component lists, and a controller cannot tell a withheld component from an
      absent one.
- [x] Implementation: `PerceivedEvent` wrapping an `EventEnvelope` the observer is entitled to.
      → a transparent newtype. The wrapper is not ceremony: handing raw log entries to a controller
      is the omniscience `INV-13` forbids, and the distinct type means code that holds one knows the
      perception decision already happened.
- [x] Implementation: `Affordance` per DD-11.
      → built by `available` or `unavailable`, never field by field, so "available, because too far
      away" is unrepresentable in code and refused on deserialization. Carries the action type, the
      target, the reason and the unevaluated `SpatialRequirement` — and no display text (`DD-13`).
- [x] Implementation: `Observation { observer, at, self_location, entities, events, affordances }` with no store handle and no global accessor.
      → plus `entity(id)`, which searches the list it was given and nothing else, because a client
      must be able to resolve an affordance's target to something it can name (`DD-13`).
- [x] Validation: an observation exposes exactly the entities it lists; affordances carry a server-computed reason; round-trip and ordering stability.
      → `contracts/tests/observation.rs`, 4 tests. The scene is the 3D spike's: an observer, a person
      within a three-metre reach, a person outside it, and a third person the world contains and this
      observer was not shown. One test reconstructs `[E] Talk to Alice` from the observation alone
      and then shows the server's own evaluator reaching the same two answers from the same
      locations — the evidence that there is one implementation of the rule rather than one per
      client. Two mutations in §7.2.
- [x] Review: no field or method could return an entity not perceived; nothing here lets a client recompute availability.
      → every public method of `Observation` was listed and inspected: six return an own field, three
      return a slice of an own field, and `entity` searches that slice. There is no handle, no
      registry, no query and no method taking a store. A client *can* call
      `SpatialRequirement::evaluate` — it is public because the server and the systems need it — but
      it has nothing authoritative to call it on: an observation gives it only the positions the
      world chose to expose, and the answer it would compute has no standing. The authoritative
      answer is the one the server put in the `Affordance`, which is `ENGINEERING_RULES.md` §8's
      division exactly. Recorded as observation OBS-2 in §7.3.

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
C1 DONE   C2 DONE   C3 DONE   C4 DONE   C5 not started
```

## 7.2 Evidence
```text
ENVIRONMENT  rustc/cargo 1.97.1 (pinned by rust-toolchain.toml), rustfmt 1.9.0,
             clippy 0.1.97, macOS aarch64. PATH must carry ~/.cargo/bin.

--- C1 (working tree at the C1 commit) -------------------------------------------
cargo fmt --all --check                                        PASS  (clean)
cargo check --workspace --all-targets                          PASS  (0 warnings)
cargo clippy --workspace --all-targets --all-features
                                    -- -D warnings             PASS  (0 warnings)
cargo test -p mineworld-contracts                              PASS
  36 integration tests + 2 doc-tests, 0 failed, wall time 0.51s for the whole
  binary set (the trybuild harness dominates it; every other file reports 0.00s)
  action 5 · component 6 · entity 6 · identity 9 · relation 6 · time 3 ·
  compile_fail harness 1 (6 cases) · doc-tests 2
  baseline before C1 was 32 integration tests + 1 doc-test

--- C2 (working tree at the C2 commit) -------------------------------------------
cargo fmt --all --check                                        PASS  (clean)
cargo check --workspace --all-targets                          PASS  (0 warnings)
cargo clippy --workspace --all-targets --all-features
                                    -- -D warnings             PASS  (0 warnings)
cargo test -p mineworld-contracts                              PASS
  42 integration tests + 3 doc-tests, 0 failed, 0.59s dominated by the trybuild
  harness; every other test binary reports 0.00s
  action 5 · component 6 · entity 6 · event 6 · identity 9 · relation 6 · time 3 ·
  compile_fail harness 1 (now 7 cases) · doc-tests 3

--- C3 (working tree at the C3 commit) -------------------------------------------
cargo fmt --all --check                                        PASS  (clean)
cargo check --workspace --all-targets                          PASS  (0 warnings)
cargo clippy --workspace --all-targets --all-features
                                    -- -D warnings             PASS  (0 warnings)
cargo test -p mineworld-contracts                              PASS
  50 integration tests + 3 doc-tests, 0 failed, 0.64s dominated by the trybuild
  harness; every other test binary reports 0.00s
  action 5 · component 6 · entity 6 · event 6 · identity 9 · relation 6 ·
  spatial 8 · time 3 · compile_fail harness 1 (7 cases) · doc-tests 3

--- C4 (working tree at the C4 commit) -------------------------------------------
cargo fmt --all --check                                        PASS  (clean)
cargo check --workspace --all-targets                          PASS  (0 warnings)
cargo clippy --workspace --all-targets --all-features
                                    -- -D warnings             PASS  (0 warnings)
cargo test -p mineworld-contracts                              PASS
  54 integration tests + 3 doc-tests, 0 failed, 0.72s dominated by the trybuild
  harness; every other test binary reports 0.00s
  action 5 · component 6 · entity 6 · event 6 · identity 9 · observation 4 ·
  relation 6 · spatial 8 · time 3 · compile_fail harness 1 (8 cases) · doc-tests 3

MUTATIONS  (purpose: prove the new guards are load-bearing, not decorative)
  M1  ActionRecord::payload_for stops comparing the action type
      expected: a_request_survives_erasure_and_is_readable_only_as_its_own_action_type RED
      observed: that test FAILED, the other four passed            → behaviour-changing
  M2  ActionIntent's deserialization stops comparing envelope to payload
      expected: an_intents_envelope_cannot_disagree_with_its_payload RED
      observed: that test FAILED, the other four passed            → behaviour-changing
  M3  ActionIntent::new writes a stale default action type instead of reading the record
      expected: the shape, INV-10 and agreement tests RED
      observed: 3 of 5 FAILED                                      → behaviour-changing
  M4  EventEnvelope's deserialization stops comparing envelope to payload
      expected: an_envelope_cannot_disagree_with_its_payload RED
      observed: that test FAILED, the other five passed            → behaviour-changing
  M5  a missing caused_by quietly defaults to WorldGenesis instead of failing
      (the realistic defect: someone makes the field lenient)
      expected: a_fact_with_no_cause_or_no_audience_cannot_be_read_back RED
      observed: that test FAILED, the other five passed            → behaviour-changing
  NOT MUTATED  the order-stability of Visibility::Entities. The guarantee is the
      BTreeSet in the type, and replacing it with a Vec makes the test file stop
      compiling rather than fail, which would be an inconclusive mutation, not a
      counterfactual. The test instead pins the serialized order and pins that two
      opposite insertion orders produce identical bytes.
  M6  a convenience `Millimetres::as_metres(self) -> f64` is added — the most likely
      way a float would actually arrive in this crate
      expected: no_floating_point_appears_anywhere_in_the_contract_crate RED
      observed: that test FAILED, the other seven passed           → behaviour-changing
  M7  yaw is stored as given instead of canonicalized with rem_euclid
      expected: a_heading_is_canonicalized_and_an_impossible_pitch_is_refused RED
      observed: that test FAILED                                   → behaviour-changing
  M8  evaluate answers place and distance before target availability
      expected: the_evaluator_answers_every_documented_spatial_case RED
      observed: that test FAILED                                   → behaviour-changing
  M9  the unmodelled-position branch stops comparing places, so a millimetre range
      reaches across the world
      expected: a_range_requirement_degenerates_to_the_same_place... RED
      observed FIRST RUN: ALL EIGHT TESTS PASSED — MUTATION SURVIVED
      diagnosis: the degeneracy test used same_place().within(2m), and
      PlaceRequirement::SamePlaceAsActor already rejects a different place before the
      range logic is reached. The branch under test was never the deciding factor, so
      the test asserted the right answer for the wrong reason. The case that isolates
      the branch is a *bare* range — SpatialRequirement::NONE.within(2m), which is how
      a system declares "near the target, wherever that is" — with no place clause to
      fall back on.
      fix: added that case, both directions, plus the positions-present counterpart so
      the test also pins that a real distance is still a distance.
      observed AFTER the fix: that test FAILED under the same mutation, the other seven
      passed                                                        → behaviour-changing
  M10 squared distances are computed in i64 instead of i128
      expected: distance_arithmetic_survives_the_extremes_of_the_representable_range RED
      observed: that test FAILED                                   → behaviour-changing
  M11 the affordance availability/reason agreement check is dropped
      expected: an_affordance_cannot_disagree_with_itself_about_availability RED
      observed: that test FAILED, the other three passed           → behaviour-changing
  M12 Observation::entity ignores the identity it was asked about and returns the first
      entity in the list — the shape a "helpful" lookup defect would actually take
      expected: the INV-13 test and the prompt-rendering test RED
      observed: both FAILED                                        → behaviour-changing
  one mutation (M9) survived and was resolved by strengthening the test, as recorded
  above; no mutation survives now. Source restored and re-verified green after each.
```

## 7.3 Findings, decisions, deviations
```text
DEVIATION D-1 (bounded) — ActionIntent.actor_location arrives in C3, not C1
REASON   the frozen commit order puts action.rs (C1) before spatial.rs (C3), but the
         field's type is Location, which C3 defines. C3 in turn needs Rejection from
         C1 for SpatialRequirement::evaluate, so the two modules depend on each other
         and no commit order gives both commits a compiling tree with the field in C1.
EVIDENCE §5 C1 lists actor_location; §5 C3 lists Location and declares "Depends on
         C1"; a Rust module cycle inside one crate is legal, a commit cycle is not.
DECISION keep the frozen commit order and let the one field that cannot exist yet
         arrive with its type in C3, together with its accessor, its builder method
         and its test. Rejected alternative: reorder C3 before C1, which would have
         moved Rejection out of action.rs or evaluate out of C3 — a larger change to
         the frozen design for no gain.
IMPACT   the contract at the final HEAD is exactly the one §5 specifies; only the
         commit that introduces one field differs. C3's diff touches action.rs.
VALIDATION  C1's and C3's §6 runs are both clean; the field is tested in C3.

DECISION K-1 — an intent's envelope and its payload cannot name different action types
QUESTION §5's ActionIntent carries action_type *and* payload: ActionRecord, and the
         record carries the action type too. Two places to say one thing.
EVIDENCE dispatch (S3) routes by the envelope while the owning system decodes the
         payload, so a disagreement would make one of them act on a request nobody
         made. ENGINEERING_STANDARDS §7 requires models in which invalid states are
         hard to represent; crate rule 5 in lib.rs requires deserialization to run the
         same check as construction.
DECISION keep both fields as the design specifies, and remove the disagreement
         instead: `new` derives the envelope's type from the record, and
         deserialization goes through a private fields struct that refuses a
         mismatched pair with ContractError::ActionIntentPayloadMismatch. The field
         list and the stored shape are unchanged; only the ability to lie is gone.
VALIDATION  an_intents_envelope_cannot_disagree_with_its_payload; mutations M2 and M3.

DECISION K-2 — Rejection::Unavailable and ActionResult::Unavailable are different facts
QUESTION DD-3 lists Unavailable among Rejection's variants and §5's ActionResult has
         an Unavailable variant of its own. As written, one world fact would have two
         representations.
EVIDENCE ENGINEERING_RULES §8 lists Unavailable among the answers a client renders, so
         it is kernel rejection vocabulary; INV-10 is a separate and stronger statement
         — "there is no shoot() in this universe" — and CORE_CONCEPTS §12 gives it its
         own name, ActionUnavailable.
DECISION implement both, as frozen, with documented and non-overlapping meanings:
         ActionResult::Unavailable = no enabled system provides this action type, so
         the answer is the same for every actor at every moment; Rejection::Unavailable
         = the action exists, the owning system considered this attempt and refused it
         without classifying the reason further. Pinned as distinguishable on the wire
         ("unavailable" vs {"rejected":"unavailable"}), because a client that showed
         the first as the second would tell a player to try again in a world where the
         capability is simply absent.
VALIDATION  an_intent_for_an_action_no_system_provides_is_representable_and_answered_unavailable

FINDING F-1 — the identifier rule had to become crate-visible
SOURCE   §1.1 places ActionTypeId in action.rs and EventTypeId in event.rs, while S1
         put every other declaration name in ids.rs, where `check_identifier` is a
         private const fn.
DECISION follow §1.1 for placement and widen `check_identifier` to `pub(crate)` rather
         than write a second const checker. ids.rs's own documentation says the rule is
         checked in one place so that a literal in code and a name in an authored file
         cannot be judged by two drifting implementations; a copy would have broken
         exactly that. ids.rs now records where the two new declaration names live.
IMPACT   no public API change in S1's surface; one visibility change.

DECISION K-4 — a yaw is canonicalized, a pitch is rejected
QUESTION crate rule 5 in lib.rs says validation rejects rather than repairs and that
         nothing is silently normalized. DD-5 says an orientation is "normalized on
         construction". Both cannot be true of the same field.
EVIDENCE 450 000 and 90 000 millidegrees of yaw are the *same direction*; storing them
         differently would put two representations of one fact in the event log and make
         two identical worlds compare unequal, which is the determinism AC-12 wants. A
         pitch of 100° is not another way of writing a legal value — past straight up
         there is no steeper direction — so clamping it to 90° would turn an impossible
         value into a plausible one and hide the mistake.
DECISION canonicalize the periodic quantity, reject the bounded one. Both rules are
         honoured, and the difference between them is documented on the type:
         canonicalization maps an equivalence class to one representative, repair
         invents a value the caller did not mean.
VALIDATION  a_heading_is_canonicalized_and_an_impossible_pitch_is_refused, including
         the deserialization path; mutation M7.

DECISION K-5 — a range requirement degenerates to the same place, not to pass or fail
QUESTION §5 C3 requires a "documented outcome, not a panic" when a range requirement
         meets a Location with no LocalPosition. Three outcomes were available: pass,
         fail, or something else.
EVIDENCE DD-4 promises that a world leaving the continuous refinement out "loses
         nothing". Passing unconditionally would let a three-metre range be satisfied
         from another town, so a system's declaration would silently mean nothing.
         Failing unconditionally would make any action declared by a 3D-capable system
         unusable in a purely semantic 2D world, breaking DD-4's promise and
         ENGINEERING_RULES §9's requirement that both clients exercise the same actions.
DECISION the range degenerates to *same place*: being in one place is the finest
         proximity such a world can express, so that is what the requirement means
         there. A world that models positions on both sides gets an exact integer
         distance; a world that models neither gets a place comparison; a world that
         models one side gets the place comparison, because a distance needs both ends.
VALIDATION  a_range_requirement_degenerates_to_the_same_place_when_no_position_is_modelled,
         and mutation M9, whose survival exposed that the first version of this test was
         asserting the right answer for the wrong reason.

DECISION K-6 — no public distance function
QUESTION `evaluate` needs a distance comparison. Should `LocalPosition` expose the
         distance itself?
EVIDENCE two positions at opposite ends of the `i32` millimetre range are about
         4 295 km apart, which does not fit in an `i32` of millimetres — so a
         `distance_to(&self) -> Millimetres` would have an unrepresentable result at the
         edges, and an exact distance needs a square root this crate cannot take without
         a float.
DECISION keep the comparison private (`LocalPosition::within`) and expose no distance.
         Squared distances are compared in `i128`, which is exact and cannot overflow at
         any representable input. A system that later needs a distance gets one designed
         for its actual use, rather than an overflowing convenience added in advance
         (ENGINEERING_STANDARDS §11).
VALIDATION  distance_arithmetic_survives_the_extremes_of_the_representable_range;
         mutation M10.

OPEN O-1 — an event payload has no schema version, and the event log is permanent
SOURCE   §5 C2 specifies the Event trait as (EVENT_TYPE, OWNER). ComponentRecord
         carries a ComponentSchemaVersion precisely because stored state outlives the
         code that wrote it, and payload_for refuses a record from a newer schema
         instead of guessing. The event log is the single source of truth for history
         (INV-11) and is replayed, so an event payload written years earlier is exactly
         the same problem.
DECISION implement C2 as frozen and record the gap here rather than adding a constant
         to a frozen public trait on this session's own authority. S5 (persistence) is
         where the migration question becomes concrete, and ENGINEERING_STANDARDS §12
         ("no early compatibility debt") permits changing the interface cleanly then.
IMPACT   none inside this PR. S5 must decide payload versioning for events before any
         event is persisted, or the first schema change to a system's event payload
         will be undetectable on replay.

DECISION K-3 — an event's location is a PlaceId, not a Location
QUESTION CORE_CONCEPTS §11 lists `Location` among an event's ten fields, and this PR
         introduces a `Location` type. Should the envelope hold that type?
EVIDENCE ENGINEERING_RULES §5 and INV-5: the authoritative record is semantic. A
         `Location` is a place plus an optional millimetre position plus an optional
         orientation — and an orientation is a property an entity has, not a property a
         fact has. Every event in every world would carry a `facing` field that can
         never mean anything.
DECISION `place: Option<PlaceId>`, named `place` rather than `location` so that a
         reader is not invited to expect the `Location` type. A system whose events
         genuinely carry continuous geometry — a movement system recording a new
         position — puts it in its own typed payload, where it is that system's
         contract rather than the kernel's. This also removes C2's dependency on C3.
GATE     re-checked against §3.1 question 1: an embodied 3D client still gets
         continuous position where position lives, on the entity, through
         `PerceivedEntity.location` and `ActionIntent.actor_location`. Nothing about
         this choice makes a 3D client harder.

OPEN O-2 — nothing here maps an action type to its owning system as data
SOURCE   DD-2's rationale is that ownership at the type level is what lets S3 route an
         intent to one system. A registry cannot store types, which is why S1 has
         ComponentDeclaration beside Component. §1.1 does not list an ActionDeclaration
         and §1.2 assigns dispatch to S3.
DECISION do not add one. A::OWNER is readable wherever the type is known, and inventing
         the registry's value type before the registry exists would be the premature
         abstraction ENGINEERING_STANDARDS §11 forbids. Recorded so S3 adds it
         deliberately rather than rediscovering the need.

OBSERVATION OBS-2 — a client can call the evaluator, and that is not a hole
`SpatialRequirement::evaluate` is public, because the server and the systems that declare
requirements both need it, and a client linked against this crate could call it too. That
does not let a client evaluate a world rule in the sense ENGINEERING_RULES §8 forbids: an
observation gives a client only the positions the world chose to expose, and an answer a
client computes has no standing anywhere — the authoritative answer is the one the server
put in the Affordance, and the only thing a client can do with a computed one is mislead
its own player. Making the evaluator private would not change that and would break its
actual purpose, which is that there is exactly one implementation of the check.

OBSERVATION OBS-1 — ActionResult carries no ActionId
DD-1's rationale says a result must be attributable to the exact request, and §5's
ActionResult has no field for it. Implemented as frozen, with the reasoning documented
on the type: correlation belongs to whatever paired the request with its answer — the
dispatcher in process, the protocol on a wire (S11) — and a field here would make every
in-process answer restate what its caller already holds while still not preventing a
mismatched pair. S3 and S11 own the pairing.


FINDING (spike, 2026-09-25) — embodied 3D is mechanically feasible on this host
SOURCE  scratchpad/spike3d: Godot 4.7.2, CharacterBody3D + CollisionShape3D,
        Camera3D child, RayCast3D with a 3.0 m reach, StaticBody3D NPC carrying
        entity_id as node metadata, CanvasLayer HUD.
EVIDENCE  scripted walk from z=+4.0 to z=-2.0 with gravity and collision; at
        z=-2.0 the ray hit the NPC collider, recovered entity_id 4201, and the
        HUD rendered "[E] Talk to Alice"; three PNGs captured and visually
        inspected. First run failed to target: the player stopped 3.55 m away
        with a 3.0 m reach, which is itself useful evidence that interaction
        reach is a real spatial parameter and belongs in SpatialRequirement
        rather than in client code.
IMPLICATION  S14 is wiring, not research. Two contract refinements follow,
        recorded as DD-13 and DD-14.

FINDING (spike, 2026-09-25) — Godot/Rust transport works; JSON numbers do not
SOURCE  scratchpad/spikenet: axum 0.8 WebSocket server on 127.0.0.1:7878 and a
        Godot WebSocketPeer client exchanging an ActionIntent and an
        ActionResult as JSON.
EVIDENCE  server logged recv {"action_type":"talk","target":4201} and sent
        {"result":"accepted","events":[9001]}; the client logged recv of the
        same text but parsed events=[9001.0].
WHY IT MATTERS  Godot parses every JSON number as a double. EntityId and
        EventId above 2^53 would round silently — a corruption that unit tests
        on the Rust side would never catch, because the Rust side is correct.
DECISION  DD-15: contracts keep integer ids; S11 owns a wire encoding that
        renders 64-bit ids as decimal strings, with a round-trip test through
        an actual Godot client rather than through a Rust-only test.
NOT A CHANGE TO S1  PR 01 is unaffected and was not interrupted: the defect is
        at the protocol boundary, not in the contract representation.
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

---

# 9. Execution contract

Filled from `.claude/skills/structured-coding/prompts/implementation-working-rules.md`. The
operator's PR 02 execution kickoff of 2026-09-25 is the source for every endpoint below.

```text
PROJECT / PR        Step 02 / PR 02 — Action, Event, Observation and Spatial contracts
PRIMARY DESIGN DOC  this document
RELATED / BINDING   overall.md · docs/CORE_CONCEPTS.md · docs/ENGINEERING_RULES.md §§4-12, §15 ·
                    docs/ENGINEERING_STANDARDS.md · CLAUDE.md ·
                    step-01-entity-component-contracts.md (merged precedent)
IMPLEMENTATION BASE main @ c8f2934, branch mvp0/pr-02-action-event-spatial
APPROVED SCOPE      §1.1 only. Everything in §1.2 belongs to a later step.
FROZEN INVARIANTS   §1.3, restated by the operator: no floating point anywhere; no domain
                    concept; no engine concept; BTreeMap/BTreeSet/Vec only; an Event cannot be
                    constructed without a Causation and a Visibility; an Observation exposes only
                    the entities it lists; ActionRecord and EventRecord are the only erasure
                    boundaries, mirroring ComponentRecord<P = Vec<u8>>
SEQUENCE            §5: C1 action · C2 event · C3 spatial · C4 observation · C5 specifications
VALIDATION BUDGET   ordinary static/unit/contract runs unrestricted. No real-LLM and no
                    real-lifecycle layer exists at S2 (§4), so no Gate applies and none was run.
REQUIRED LIVE DOC   this document, §7
HANDOFF FILE        .structured-coding/plans/mvp0/handoff.md
TEST OWNERSHIP      §4

ENDPOINT AUTHORITY
  implementation + local validation   authorized      source: operator kickoff, 2026-09-25
  semantic commits                    authorized      source: operator kickoff — "Semantic
                                                      commits on your branch: AUTHORIZED"
  branch push                         NOT authorized  source: operator kickoff — "git push: NOT
                                                      AUTHORIZED, there is no remote and none
                                                      may be added"; decision D-9 in overall.md
  PR creation / remote CI             N/A             source: no remote exists (D-9)
  merge into main                     explicit operator authorization only — "Merging to main:
                                                      NOT yours. Stop at ready-for-review."

POST-MERGE SYNCHRONIZATION OWNER
  this implementation session owns this PR document, its evidence and its deviations;
  the planning session owns the step-level and overall.md updates after merge
  (overall.md §7 convention, unchanged from PR 01)

NORMAL STOP CONDITION
  PR 02 READY FOR OPERATOR REVIEW on mvp0/pr-02-action-event-spatial — DO NOT MERGE

STOP AND REPORT INSTEAD OF IMPROVISING WHEN
  a frozen invariant in §1.3 would have to change
  a gate question in §3.1 cannot be answered yes against the written code
  a public specification statement is falsified
```
