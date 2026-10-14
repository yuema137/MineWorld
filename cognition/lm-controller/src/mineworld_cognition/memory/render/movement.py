"""The movement pack's facts: `walk-started {person, destination}` and `walk-ended {person, destination,
outcome}` (ARC-75). Another person's walks are background, `ambient`, as their comings and goings are;
mine are notable (pr-fg-b QFGb-6). Without this renderer the generic one filed every stride anyone took
as notable: 36 % of a seat's L0 bytes (F-FGb-5)."""

from __future__ import annotations

from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId

from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.render import Payload, Rendered, counterparts, decode, fallback
from mineworld_cognition.memory.render.conversation import PersonRef

MINE = 80
AMBIENT = 30
"""Base importance (pr-fg-b §5.4): a walk of mine, and another person's."""

ENDINGS = {
    "arrived": "arrived after a walk",
    "stalled": "got stuck on a walk",
    "no-route": "found no way on a walk",
    "replaced": "changed course",
    "stopped": "stopped walking",
}
"""How a walk ended (the pack's `Ended`), as words; an outcome this code does not know reads as
`stopped walking`, never as the pack's raw value."""


class Started(Payload):
    person: PersonRef


class Ended(Payload):
    person: PersonRef
    outcome: str


def _walk(
    fact: PerceivedEvent, me: EntityId, names: NameBook, person: EntityId, did: str
) -> Rendered:
    who = counterparts(fact, me)
    if person == me:
        return Rendered(f"I {did}", who, "notable", True, MINE)
    return Rendered(f"{names.person(person)} {did}", who, "ambient", False, AMBIENT)


def walk_started(fact: PerceivedEvent, me: EntityId, names: NameBook) -> Rendered:
    payload = decode(Started, fact)
    if payload is None:
        return fallback(fact, me, names)
    return _walk(fact, me, names, payload.person.entity, "set out walking")


def walk_ended(fact: PerceivedEvent, me: EntityId, names: NameBook) -> Rendered:
    payload = decode(Ended, fact)
    if payload is None:
        return fallback(fact, me, names)
    did = ENDINGS.get(payload.outcome, ENDINGS["stopped"])
    return _walk(fact, me, names, payload.person.entity, did)
