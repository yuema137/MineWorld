"""The conversation pack's facts: `spoke {speaker, listener, utterance}` and `conversation-started
{speaker, listener}`."""

from __future__ import annotations

from typing import Literal

from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EntityIdField

from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.render import (
    Payload,
    Rendered,
    counterparts,
    decode,
    fallback,
    quote,
)


class PersonRef(Payload):
    entity: EntityIdField
    entity_type: Literal["person"]


class Spoke(Payload):
    speaker: PersonRef
    listener: PersonRef
    utterance: str


class Started(Payload):
    speaker: PersonRef
    listener: PersonRef


def _between(
    speaker: EntityId, listener: EntityId, me: EntityId, names: NameBook, verb: str
) -> str:
    """`I <verb> Bob`, `Bob <verb> me`, or `Bob <verb> Carol`."""
    if speaker == me:
        return f"I {verb} {names.person(listener)}"
    if listener == me:
        return f"{names.person(speaker)} {verb} me"
    return f"{names.person(speaker)} {verb} {names.person(listener)}"


def spoke(fact: PerceivedEvent, me: EntityId, names: NameBook) -> Rendered:
    payload = decode(Spoke, fact)
    if payload is None:
        return fallback(fact, me, names)
    speaker, listener = payload.speaker.entity, payload.listener.entity
    gist = f"{_between(speaker, listener, me, names, 'said to')}: {quote(payload.utterance)}"
    return Rendered(gist, counterparts(fact, me), "notable", speaker == me)


def conversation_started(fact: PerceivedEvent, me: EntityId, names: NameBook) -> Rendered:
    payload = decode(Started, fact)
    if payload is None:
        return fallback(fact, me, names)
    speaker, listener = payload.speaker.entity, payload.listener.entity
    gist = _between(speaker, listener, me, names, "started talking with")
    return Rendered(gist, counterparts(fact, me), "notable", speaker == me)
