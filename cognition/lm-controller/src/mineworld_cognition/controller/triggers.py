"""Trigger detection and the one-in-flight pending slot (pr-s10-p6 §5.4, C11).

T1 (spoken to): a `spoke` fact whose listener is me and whose speaker is not me. T3 (a remembered person entered my
place): an `arrived` or `person-entered-place` fact for a person with a stable L3 fact, in the place I am in, at
most once per counterpart per cooldown. `detect` reads a batch the store has already absorbed (R-P6-FG-1, I-P6-6);
it reads no clock, the caller passes the world time. `Pending` holds one consult in flight and at most one trigger
that waits: a newer trigger replaces an older one (the seat is dirty; step-17 §3.4.5).

Two kinds come from `pr-le-leave-early.md` (LE-c). `complaint` (§0.5): a `late-for-shift` fact in which I am the
employee or the manager, rendered as one line in my words. `upcoming` (§0.4 item 2): not a fact but the time; the
runner asks `upcoming_due` at each observation, once when a punctual next part is within `horizon_minutes`, then
every `reask_minutes` until I set out, arrive, or the part begins. Neither decides anything: the model does.
"""

from __future__ import annotations

import asyncio
from collections.abc import Sequence
from dataclasses import dataclass
from typing import NamedTuple

from mineworld_sdk.wire.contract import PerceivedEvent, PlaceId
from mineworld_sdk.wire.ids import EntityId, JsonValue

from mineworld_cognition.controller.prompt import Trigger
from mineworld_cognition.controller.situation import NextPart, Situation
from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.names import SOMEONE, NameBook

SPOKE = "spoke"
LATE_FOR_SHIFT = "late-for-shift"
ORDINALS = ("first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth")
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
        elif fact.event_type == LATE_FOR_SHIFT:
            line = complaint_line(body, me, NameBook.load(store))
            if line is not None:
                triggers.append(Trigger(event=fact.id, kind="complaint", speaker=None, words=line))
    return triggers


def complaint_line(body: dict[str, JsonValue], me: EntityId, names: NameBook) -> str | None:
    """employment's `late-for-shift` (§0.6) in my words, when I am its employee or its manager; else `None`.

    *"Alice was 12 minutes late for the shift at the Café — the third time in 7 days."* Every part the fact
    does not carry is left out; nothing is inferred."""
    employee, manager = person_of(body, "employee"), person_of(body, "manager")
    if employee is None or me not in (employee, manager):
        return None
    mine = me == employee
    who = "I" if mine else names.person(employee)
    employer = person_of(body, "employer")
    named = None if employer is None else names.person(employer)
    shift = ("my shift" if mine else "the shift") + (
        "" if named is None or named == SOMEONE else f" at {named}"
    )
    late_by, count = _int(body.get("late_by")), _int(body.get("count"))
    if body.get("absent") is True:
        line = f"{who} did not come to {shift}"
    elif late_by is not None:
        line = f"{who} was {-(-late_by // 60)} minutes late for {shift}"
    else:
        line = f"{who} was late for {shift}"
    if count is not None and count >= 1:
        nth = ORDINALS[count - 1] if count <= len(ORDINALS) else f"{count}th"
        window = _int(body.get("window_days"))
        line += f" — the {nth} time" + ("" if window is None else f" in {window} days")
    level = body.get("level")
    tail = {"warning": " It is a warning.", "final": " It is a final warning."}
    return line + "." + (tail.get(level, "") if isinstance(level, str) else "")


@dataclass(frozen=True, slots=True)
class Asked:
    """The last `upcoming` ask: about the part beginning at `from_at`, at world second `at`."""

    from_at: int
    at: int


class Felt(NamedTuple):
    """How I feel at one observation (S35 N-e): my need words in the pack's order, and whether I sleep."""

    pairs: tuple[tuple[str, str], ...]
    asleep: bool


def felt_of(situation: Situation) -> Felt:
    """What `needs` and `asleep` disclose to me now, as the situation holds it."""
    pairs = tuple((feeling.need, feeling.word) for feeling in situation.feelings)
    return Felt(pairs=pairs, asleep=situation.asleep_since is not None)


def feeling_due(now: Felt, last: Felt | None) -> str | None:
    """The line a `feeling` trigger carries, or `None` (S35 N-e, `pr-s35-ne-presentation.md` §4.7). Pure.

    The first observation only sets the baseline. While I sleep nothing is asked. On waking I am told so once.
    Otherwise I am told each need word that changed since the previous observation, and nothing else."""
    if last is None or now.asleep:
        return None
    if last.asleep:
        return "I notice that I am awake again."
    changed = [f"{need}: {word}" for need, word in now.pairs if (need, word) not in last.pairs]
    if not changed:
        return None
    return "I notice how I feel: " + "; ".join(changed) + "."


def upcoming_due(
    next_part: NextPart | None,
    here: EntityId | None,
    walking: bool,
    now_at: int,
    horizon_s: int,
    reask_s: int,
    last: Asked | None,
) -> bool:
    """Whether to ask the seat about its punctual next part now (§0.4 item 2). Pure."""
    if next_part is None or not next_part.punctual or walking or here == next_part.place:
        return False
    if not next_part.from_at - horizon_s <= now_at < next_part.from_at:
        return False
    return last is None or last.from_at != next_part.from_at or now_at - last.at >= reask_s


def _int(value: JsonValue) -> int | None:
    return value if isinstance(value, int) and not isinstance(value, bool) else None


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


class Waiting(NamedTuple):
    """A trigger in the pending slot, with the world time of the fact it was detected on (`heard_at`) and the
    runner's clock when it was offered (`received_at`)."""

    trigger: Trigger
    heard_at: int
    received_at: float


class Pending:
    """One slot for the trigger that waits while a consult runs. `offer` replaces a waiting trigger with the
    newer one; `take` waits for one and removes it."""

    def __init__(self) -> None:
        self._waiting: Waiting | None = None
        self._ready = asyncio.Event()

    def offer(self, trigger: Trigger, heard_at: int, received_at: float) -> None:
        self._waiting = Waiting(trigger, heard_at, received_at)
        self._ready.set()

    async def take(self) -> Waiting:
        while True:
            await self._ready.wait()
            self._ready.clear()
            if self._waiting is not None:
                waiting, self._waiting = self._waiting, None
                return waiting
