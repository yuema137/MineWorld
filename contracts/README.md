# contracts/

The typed vocabulary every other layer speaks: **`mineworld-contracts`**, a Rust crate of pure
data plus validation and serialization. No storage, no dispatch, no scheduling, no persistence —
the kernel and the systems depend on this crate, and nothing here depends back.

What exists today:

```text
identity     EntityId, EntityKey, EntityType, PersonId / PlaceId / ItemId / OrganizationId,
             SystemId, ComponentTypeId, RelationTypeId, EventId, ActionId, ProcessId
entity       Entity, Tag, Tags, LifecycleState, Metadata
components   Component, ComponentDeclaration, ComponentRecord, ComponentSchemaVersion
relations    RelationTypeDeclaration, Relation, EntityTypeSet, RelationDirection, SelfEdges
actions      ActionTypeId, Action, ActionRecord, ActionIntent, ActionResult, Rejection,
             RejectionCode
events       EventTypeId, Event, EventRecord, EventEnvelope, Causation, Visibility, Provenance
spatial      Millimetres, Millidegrees, LocalPosition, Orientation, Location,
             PlaceRequirement, SpatialRequirement
observation  Observation, PerceivedEntity, PerceivedEvent, Affordance
time         WorldTime, SimDuration
errors       ContractError, IdentifierKind, RelationEnd, MAX_IDENTIFIER_LENGTH
```

Still to come, in this order: the `System` interface, then the world clock and scheduler.

Five rules govern anything added here, and the crate documentation states them in full: no domain
concept (no `Money`, no `Hunger`, no `Employment` — those are System Packs), order-deterministic
collections only, everything serializes, one documented payload-erasure boundary per family
(`ComponentRecord`, `ActionRecord`, `EventRecord`), and validation in constructors that rejects
rather than repairs.

Two more, which the spatial and observation contracts exist to keep: **no floating point** — a
position is `i32` millimetres and an angle is `i32` millidegrees, because positions reach the event
log and a replay must be exact — and **no renderer concept**, so no mesh, navmesh, collider, camera,
scene node, animation, skeleton or physics. A client maps a `Location` onto whatever its engine
uses; the simulation never learns what that was.

```sh
cargo test -p mineworld-contracts
cargo doc -p mineworld-contracts --open      # the crate docs are the reference
```

Some of the guarantees are checked by code that must *not* compile — a place used where a person
is required, a component without an owner, a recorded event edited after the fact — in
[`contracts/tests/compile_fail/`](tests/compile_fail/). One is checked by reading the crate's own
sources: `tests/spatial.rs` asserts that no floating-point type appears in any of them.

What belongs here and what must never leak in is specified in
[`../docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md) and
[`../docs/CORE_CONCEPTS.md`](../docs/CORE_CONCEPTS.md).
