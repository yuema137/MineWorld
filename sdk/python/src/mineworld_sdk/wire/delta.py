"""`ObservationDelta`: one observation as the change from the previous one (`server/PROTOCOL.md` §5.3).

Mirrors `server/src/protocol/delta.rs`. A session applies each `delta` frame to the observation it
holds and keeps the whole result, so a caller only ever sees whole observations (`DEP-15`). The table
of §5.3, field by field:

    at                 always present; replaces
    self_location      present only if it changed; replaces, `null` included
    entities.upsert    each replaces the entity with its id, or is added; then sorted by id
    entities.remove    ids no longer perceived; one the held observation does not list is an error
    relations          present only if changed; replaces the whole list
    affordances        present only if changed; replaces the whole list, in its order
    events             always present; replaces (a since-the-last-frame stream)

Ids stay strings and are ordered as the numbers they are — by length, then by digits — without ever
becoming an `int` a double could merge (D-P3-3).
"""

from __future__ import annotations

from pydantic import Field, SerializerFunctionWrapHandler, model_serializer

from mineworld_sdk.wire.contract import (
    I64,
    Affordance,
    Location,
    Observation,
    PerceivedEntity,
    PerceivedEvent,
    Relation,
    WireModel,
)
from mineworld_sdk.wire.ids import EntityIdField, JsonValue


class EntityChanges(WireModel):
    """The entity half of a delta: upserted entities and removed ids, each omitted when empty."""

    upsert: list[PerceivedEntity] = Field(default=[], exclude_if=lambda value: value == [])
    remove: list[EntityIdField] = Field(default=[], exclude_if=lambda value: value == [])


class ObservationDelta(WireModel):
    """What changed between two observations of one observer (`PROTOCOL.md` §5.3).

    `self_location` distinguishes "absent" (unchanged) from "present and `null`" (the observer now has
    no location): the field is present exactly when it is in `model_fields_set`."""

    at: I64
    self_location: Location | None = None
    entities: EntityChanges = Field(
        default=EntityChanges(), exclude_if=lambda value: value == EntityChanges()
    )
    relations: list[Relation] | None = Field(default=None, exclude_if=lambda value: value is None)
    affordances: list[Affordance] | None = Field(
        default=None, exclude_if=lambda value: value is None
    )
    events: list[PerceivedEvent]

    @model_serializer(mode="wrap")
    def _omit_an_unchanged_location(
        self, handler: SerializerFunctionWrapHandler
    ) -> dict[str, JsonValue]:
        written: dict[str, JsonValue] = handler(self)
        if "self_location" not in self.model_fields_set:
            written.pop("self_location", None)
        return written

    @property
    def location_changed(self) -> bool:
        """Whether the delta carries `self_location` (it may carry it as `None`)."""
        return "self_location" in self.model_fields_set


class DeltaMismatch(ValueError):
    """A delta that cannot apply to the observation held: it removes an entity that is not there."""


def apply_delta(held: Observation, delta: ObservationDelta) -> Observation:
    """The observation `delta` describes, from the one held. Raises `DeltaMismatch` when it removes an
    entity `held` does not list — which cannot happen on one connection (§5.3)."""
    entities = {entity.id: entity for entity in held.entities}
    for removed in delta.entities.remove:
        if removed not in entities:
            raise DeltaMismatch(f"the delta removes entity {removed}, which is not held")
        del entities[removed]
    for entity in delta.entities.upsert:
        entities[entity.id] = entity
    ordered = sorted(entities.values(), key=lambda entity: (len(entity.id), entity.id))
    return Observation(
        observer=held.observer,
        at=delta.at,
        self_location=delta.self_location if delta.location_changed else held.self_location,
        entities=ordered,
        relations=delta.relations if delta.relations is not None else held.relations,
        events=delta.events,
        affordances=delta.affordances if delta.affordances is not None else held.affordances,
    )
