"""Hand-built perceived facts, in the server's wire form, for memory's unit tests.

Each helper writes the JSON the server would send (`mineworld perceived --json`) and validates it with the
SDK's `PerceivedEvent`; nothing here calls the code under test. Entity ids are ten digits long so that no
id can be mistaken for a count, a day or a time in a rendered line.
"""

from __future__ import annotations

from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EntityKey, JsonValue, WorldInstanceId

from mineworld_cognition.memory.records import StoreIdentity

ALICE = EntityId("7000000001")
BOB = EntityId("7000000002")
CAROL = EntityId("7000000003")
DEV = EntityId("7000000004")
CAFE = EntityId("5000000001")
PARK = EntityId("5000000002")
INSTANCE = WorldInstanceId("0000000018dd1ad807f6bbd134630000")
OTHER_INSTANCE = WorldInstanceId("ffffffffffffffffffffffffffffffff")

HOUR = 3600
DAY = 86400


def alice_identity() -> StoreIdentity:
    return StoreIdentity(world_instance=INSTANCE, seat=EntityKey("alice"), self_entity=ALICE)


def _ref(entity: EntityId, entity_type: str = "person") -> JsonValue:
    return {"entity": entity, "entity_type": entity_type}


def fact(
    event: int | str,
    at: int,
    event_type: str,
    payload: JsonValue,
    *,
    subjects: list[EntityId],
    participants: list[EntityId],
    place: EntityId | None,
    decision: str | None = None,
) -> PerceivedEvent:
    """Any perceived fact. `event` may be an int for brevity; it is written as a decimal string."""
    wire: dict[str, JsonValue] = {
        "id": str(event),
        "at": at,
        "event_type": event_type,
        "subjects": list[JsonValue](subjects),
        "participants": list[JsonValue](participants),
        "place": None if place is None else _ref(place, "place"),
        "caused_by": "world_genesis" if decision is None else {"action": decision},
        "payload": {"event_type": event_type, "schema_version": 1, "payload": payload},
        "visibility": "public" if place is None else {"place": _ref(place, "place")},
        "provenance": {"emitted_by": "test", "controller_decision": decision},
    }
    return PerceivedEvent.model_validate(wire)


def spoke(
    event: int | str,
    at: int,
    speaker: EntityId,
    listener: EntityId,
    utterance: str,
    place: EntityId,
) -> PerceivedEvent:
    return fact(
        event,
        at,
        "spoke",
        {"speaker": _ref(speaker), "listener": _ref(listener), "utterance": utterance},
        subjects=[listener],
        participants=[speaker, listener],
        place=place,
        decision="1",
    )


def arrived(event: int | str, at: int, person: EntityId, place: EntityId) -> PerceivedEvent:
    location: JsonValue = {
        "facing": {"pitch": None, "yaw": 0},
        "local": {"x": 0, "y": 0, "z": 0},
        "place": _ref(place, "place"),
    }
    return fact(
        event,
        at,
        "arrived",
        {"person": _ref(person), "location": location},
        subjects=[person],
        participants=[person],
        place=place,
    )


def entered(
    event: int | str, at: int, person: EntityId, came_from: EntityId, place: EntityId
) -> PerceivedEvent:
    return fact(
        event,
        at,
        "person-entered-place",
        {"person": _ref(person), "from": _ref(came_from, "place"), "place": _ref(place, "place")},
        subjects=[person],
        participants=[person],
        place=place,
    )


def named(event: int | str, person: EntityId, name: str) -> PerceivedEvent:
    return fact(
        event,
        0,
        "named",
        {"person": _ref(person), "name": name},
        subjects=[person],
        participants=[person],
        place=None,
    )


def relationship_changed(
    event: int | str,
    at: int,
    person: EntityId,
    counterpart: EntityId,
    before: str,
    after: str,
    place: EntityId,
) -> PerceivedEvent:
    return fact(
        event,
        at,
        "relationship-changed",
        {"person": _ref(person), "counterpart": _ref(counterpart), "from": before, "to": after},
        subjects=[person],
        participants=[person, counterpart],
        place=place,
    )


def became_acquainted(
    event: int | str, at: int, person: EntityId, counterpart: EntityId, place: EntityId
) -> PerceivedEvent:
    return fact(
        event,
        at,
        "became-acquainted",
        {"person": _ref(person), "counterpart": _ref(counterpart)},
        subjects=[person],
        participants=[person, counterpart],
        place=place,
    )


def names_of_everyone() -> list[PerceivedEvent]:
    """The four `named` facts a world's genesis would deliver, ids 1 to 4."""
    return [
        named(1, ALICE, "Alice Moreau"),
        named(2, BOB, "Bob Achterberg"),
        named(3, CAROL, "Carol Mensah"),
        named(4, DEV, "Dev Raman"),
    ]
