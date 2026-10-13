"""Renderers: one perceived fact → one L0 gist, by event type, with a generic default (D-P4-6; I-9).

A renderer is a pure function of `(fact, me, names)`. A plug-in exists for the event types whose
payloads memory reads; it decodes the payload with its own small model and, when that fails, returns
the generic rendering marked `fallback`, never an exception. Every other event type — including one
from a pack this code has never seen — is rendered from the envelope alone, so it is still ingested,
covered and citable.

A record's counterparts are the fact's participants other than me, whichever renderer runs: who took
part is the envelope's statement, and L3 builds on it.
"""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass

from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId
from pydantic import BaseModel, ConfigDict, ValidationError

from mineworld_cognition.memory.names import NameBook, one_line
from mineworld_cognition.memory.records import Salience

QUOTE_LIMIT = 100
"""Characters of an utterance kept in a gist; the fact itself is one citation away."""


@dataclass(frozen=True)
class LevelChange:
    """My relationship level towards a counterpart, as a perceived fact stated it."""

    counterpart: EntityId
    level: str


@dataclass(frozen=True)
class Rendered:
    gist: str
    counterparts: tuple[EntityId, ...]
    salience: Salience
    mine: bool
    level: LevelChange | None = None
    names: tuple[tuple[EntityId, str], ...] = ()
    """Names the fact disclosed, which ingestion teaches the `NameBook`."""
    fallback: bool = False
    """A plug-in could not decode the payload and the generic rendering stands in (counted)."""


Renderer = Callable[[PerceivedEvent, EntityId, NameBook], Rendered]


class Payload(BaseModel):
    """A plug-in's view of a pack's payload: strict on the fields it reads, blind to the rest."""

    model_config = ConfigDict(strict=True, frozen=True, extra="ignore")


def decode[P: Payload](model: type[P], fact: PerceivedEvent) -> P | None:
    try:
        return model.model_validate(fact.payload.payload)
    except ValidationError:
        return None


def counterparts(fact: PerceivedEvent, me: EntityId) -> tuple[EntityId, ...]:
    return tuple(entity for entity in dict.fromkeys(fact.participants) if entity != me)


def only_me(fact: PerceivedEvent, me: EntityId) -> bool:
    """`mine` where no plug-in knows the actor: I am the fact's only subject (R-P4-8; P6 refines)."""
    return fact.subjects == [me]


def quote(text: str) -> str:
    """`«…»` around world text that cannot close the quote: `«` and `»` inside become `"`."""
    clean = one_line(text).replace("«", '"').replace("»", '"')
    if len(clean) > QUOTE_LIMIT:
        clean = clean[: QUOTE_LIMIT - 1].rstrip() + "…"
    return f"«{clean}»"


def generic(fact: PerceivedEvent, me: EntityId, names: NameBook) -> Rendered:
    """`<event type> at <place>, with <people>`: the envelope only (I-9)."""
    who = counterparts(fact, me)
    gist = f"{fact.event_type} at {names.place(None if fact.place is None else fact.place.entity)}"
    together = names.people(who)
    if me in fact.participants:
        gist += f", with me and {together}" if together else ", with me"
    elif together:
        gist += f", with {together}"
    return Rendered(gist, who, "notable", only_me(fact, me))


def fallback(fact: PerceivedEvent, me: EntityId, names: NameBook) -> Rendered:
    rendered = generic(fact, me, names)
    return Rendered(rendered.gist, rendered.counterparts, "notable", rendered.mine, fallback=True)


def _registry() -> dict[str, Renderer]:
    from mineworld_cognition.memory.render import conversation, naming, presence, relationships

    return {
        "spoke": conversation.spoke,
        "conversation-started": conversation.conversation_started,
        "arrived": presence.arrived,
        "person-entered-place": presence.person_entered_place,
        "became-acquainted": relationships.became_acquainted,
        "relationship-changed": relationships.relationship_changed,
        "named": naming.named,
    }


RENDERERS: dict[str, Renderer] = _registry()
"""Event type → renderer. A pack's renderer is registered here; any other type renders generically."""


def render(fact: PerceivedEvent, me: EntityId, names: NameBook) -> Rendered:
    return RENDERERS.get(fact.event_type, generic)(fact, me, names)
