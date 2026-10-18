"""The relationships pack's facts: `became-acquainted {person, counterpart}` and `relationship-changed
{person, counterpart, from, to}`. A change of **my** level towards someone feeds L3 (step-17 §3.8.3)."""

from __future__ import annotations

from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId

from mineworld_cognition.memory.names import NameBook, one_line
from mineworld_cognition.memory.render import (
    LevelChange,
    Payload,
    Rendered,
    counterparts,
    decode,
    fallback,
    only_me,
)
from mineworld_cognition.memory.render.conversation import PersonRef

MAX_LEVEL_LENGTH = 40
CONCERNING_ME = 800
BETWEEN_OTHERS = 300
"""Base importance (pr-fg-b §5.4): a relationship of mine (either side), and one between others."""


def _importance(person: EntityId, other: EntityId, me: EntityId) -> int:
    return CONCERNING_ME if me in (person, other) else BETWEEN_OTHERS


class Acquainted(Payload):
    person: PersonRef
    counterpart: PersonRef


class Changed(Payload):
    person: PersonRef
    counterpart: PersonRef
    to: str


def became_acquainted(fact: PerceivedEvent, me: EntityId, names: NameBook) -> Rendered:
    payload = decode(Acquainted, fact)
    if payload is None:
        return fallback(fact, me, names)
    person, other = payload.person.entity, payload.counterpart.entity
    if person == me:
        gist = f"I got to know {names.person(other)}"
    elif other == me:
        gist = f"{names.person(person)} got to know me"
    else:
        gist = f"{names.person(person)} got to know {names.person(other)}"
    importance = _importance(person, other, me)
    return Rendered(gist, counterparts(fact, me), "notable", only_me(fact, me), importance)


def relationship_changed(fact: PerceivedEvent, me: EntityId, names: NameBook) -> Rendered:
    payload = decode(Changed, fact)
    if payload is None:
        return fallback(fact, me, names)
    person, other = payload.person.entity, payload.counterpart.entity
    level = one_line(payload.to)[:MAX_LEVEL_LENGTH]
    change = None
    if person == me:
        gist = f"I now feel {level} towards {names.person(other)}"
        change = LevelChange(other, level)
    elif other == me:
        gist = f"{names.person(person)} now feels {level} towards me"
    else:
        gist = f"{names.person(person)} now feels {level} towards {names.person(other)}"
    importance = _importance(person, other, me)
    return Rendered(
        gist, counterparts(fact, me), "notable", only_me(fact, me), importance, level=change
    )
