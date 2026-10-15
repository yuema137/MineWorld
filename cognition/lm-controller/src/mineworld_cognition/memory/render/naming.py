"""The naming pack's fact: `named {person, name}`. It also teaches the `NameBook` (F-P4-3)."""

from __future__ import annotations

from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId

from mineworld_cognition.memory.names import NameBook, one_line
from mineworld_cognition.memory.render import (
    Payload,
    Rendered,
    counterparts,
    decode,
    fallback,
    only_me,
)
from mineworld_cognition.memory.render.conversation import PersonRef

NAMED = 500
"""Base importance of learning a name, mine or anyone's (pr-fg-b §5.4)."""


class Named(Payload):
    person: PersonRef
    name: str


def named(fact: PerceivedEvent, me: EntityId, names: NameBook) -> Rendered:
    payload = decode(Named, fact)
    if payload is None:
        return fallback(fact, me, names)
    person, name = payload.person.entity, one_line(payload.name)
    gist = f"I am called {name}" if person == me else f"I learned the name {name}"
    return Rendered(
        gist,
        counterparts(fact, me),
        "notable",
        only_me(fact, me),
        NAMED,
        names=((person, name),),
    )
