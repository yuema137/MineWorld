"""Trigger detection and the one-in-flight pending slot (pr-s10-p6 §5.4, C11).

T1 (spoken to): a `spoke` fact whose listener is me and whose speaker is not me. T3 (a remembered person entered my
place): an `arrived` or `person-entered-place` fact for a person with a stable L3 fact, in the place I am in, at
most once per counterpart per cooldown. `detect` reads a batch the store has already absorbed (R-P6-FG-1, I-P6-6);
it reads no clock, the caller passes the world time. `Pending` holds one consult in flight and at most one trigger
that waits: a newer trigger replaces an older one (the seat is dirty; step-17 §3.4.5).
"""

from __future__ import annotations

import asyncio
from collections.abc import Sequence

from mineworld_sdk.wire.contract import PerceivedEvent, PlaceId
from mineworld_sdk.wire.ids import EntityId, JsonValue

from mineworld_cognition.controller.prompt import Trigger
from mineworld_cognition.memory.home import SeatView

SPOKE = "spoke"
ARRIVAL_EVENTS = frozenset({"arrived", "person-entered-place"})


def detect(
    facts: Sequence[PerceivedEvent],
    me: EntityId,
    store: SeatView,
    here: PlaceId | None,
    now_at: int,
    greeted: dict[EntityId, int],
    greet_cooldown_s: int,
) -> list[Trigger]:
    """The triggers of one absorbed batch, in fact order. `greeted` maps a person to the world second of their
    last T3 trigger; the caller records the triggers it gets back."""
    known = _stable_people(store)
    triggers: list[Trigger] = []
    for fact in facts:
        body = body_of(fact)
        if fact.event_type == SPOKE:
            listener = person_of(body, "listener")
            speaker = person_of(body, "speaker")
            if listener == me and speaker is not None and speaker != me:
                words = body.get("utterance")
                triggers.append(
                    Trigger(
                        event=fact.id,
                        kind="spoken_to",
                        speaker=speaker,
                        words=words if isinstance(words, str) else "",
                    )
                )
        elif fact.event_type in ARRIVAL_EVENTS and here is not None and fact.place == here:
            person = person_of(body, "person")
            if person is None or person == me or person not in known:
                continue
            last = greeted.get(person)
            if last is not None and now_at - last < greet_cooldown_s:
                continue
            triggers.append(Trigger(event=fact.id, kind="arrived", speaker=person, words=""))
    return triggers


def body_of(fact: PerceivedEvent) -> dict[str, JsonValue]:
    """The owning pack's payload object of a fact; an empty object when the payload is not one."""
    body = fact.payload.payload
    return body if isinstance(body, dict) else {}


def person_of(body: dict[str, JsonValue], key: str) -> EntityId | None:
    """A person reference of a payload: `{"entity": "<id>", ...}`, or `None` when absent or malformed."""
    reference = body.get(key)
    if not isinstance(reference, dict):
        return None
    entity = reference.get("entity")
    return EntityId(entity) if isinstance(entity, str) else None


def _stable_people(store: SeatView) -> set[EntityId]:
    rows = store.connection.execute(
        "SELECT entity FROM stable WHERE seat = ?", (store.seat_id,)
    ).fetchall()
    return {EntityId(row[0]) for row in rows}


class Pending:
    """One slot for the trigger that waits while a consult runs. `offer` replaces a waiting trigger with the
    newer one; `take` waits for one and removes it."""

    def __init__(self) -> None:
        self._waiting: tuple[Trigger, float] | None = None
        self._ready = asyncio.Event()

    def offer(self, trigger: Trigger, received_at: float) -> None:
        self._waiting = (trigger, received_at)
        self._ready.set()

    async def take(self) -> tuple[Trigger, float]:
        while True:
            await self._ready.wait()
            self._ready.clear()
            if self._waiting is not None:
                waiting, self._waiting = self._waiting, None
                return waiting
