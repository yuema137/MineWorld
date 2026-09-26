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
time         WorldTime, SimDuration
errors       ContractError, IdentifierKind, RelationEnd, MAX_IDENTIFIER_LENGTH
```

Still to come, in this order: `ActionIntent` and `Event` payloads, then the `System` interface.

Five rules govern anything added here, and the crate documentation states them in full: no domain
concept (no `Money`, no `Hunger`, no `Employment` — those are System Packs), order-deterministic
collections only, everything serializes, one documented payload-erasure boundary
(`ComponentRecord`), and validation in constructors that rejects rather than repairs.

```sh
cargo test -p mineworld-contracts
cargo doc -p mineworld-contracts --open      # the crate docs are the reference
```

Some of the guarantees are checked by code that must *not* compile — a place used where a person
is required, a component without an owner — in
[`contracts/tests/compile_fail/`](tests/compile_fail/).

What belongs here and what must never leak in is specified in
[`../docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md) and
[`../docs/CORE_CONCEPTS.md`](../docs/CORE_CONCEPTS.md).
