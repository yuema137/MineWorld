"""The presence pack's facts: `arrived {person, location}` and `person-entered-place {person, from,
place}`. Another person's comings and goings are `ambient` (F-P4-2); mine are notable."""

from __future__ import annotations

from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EntityIdField

from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.render import Payload, Rendered, counterparts, decode, fallback
from mineworld_cognition.memory.render.conversation import PersonRef


class PlaceRef(Payload):
    entity: EntityIdField


class Location(Payload):
    place: PlaceRef


class Arrived(Payload):
    person: PersonRef
    location: Location


class Entered(Payload):
    person: PersonRef
    place: PlaceRef


def arrived(fact: PerceivedEvent, me: EntityId, names: NameBook) -> Rendered:
    payload = decode(Arrived, fact)
    if payload is None:
        return fallback(fact, me, names)
    person = payload.person.entity
    if person == me:
        gist = f"I arrived at {names.place(payload.location.place.entity)}"
        return Rendered(gist, counterparts(fact, me), "notable", True)
    return Rendered(f"{names.person(person)} arrived", counterparts(fact, me), "ambient", False)


def person_entered_place(fact: PerceivedEvent, me: EntityId, names: NameBook) -> Rendered:
    payload = decode(Entered, fact)
    if payload is None:
        return fallback(fact, me, names)
    person = payload.person.entity
    if person == me:
        gist = f"I went to {names.place(payload.place.entity)}"
        return Rendered(gist, counterparts(fact, me), "notable", True)
    return Rendered(f"{names.person(person)} came in", counterparts(fact, me), "ambient", False)
