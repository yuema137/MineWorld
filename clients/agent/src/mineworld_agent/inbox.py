"""A handle's inbox: the facts its seat perceived, bounded, as an agent reads them (step-27 §5.4).

The reliable `perceived` stream is drained into it continuously, so neither the SDK's buffer nor the
server's queue ever fills (INV-S27-6). The inbox holds at most `bound` facts (512 by default); when a new
fact arrives at the bound, the oldest is dropped and counted (`dropped`). Each fact gets an `EventRef`,
`F1`, `F2`, … in arrival order, and its subjects and place get the handle's own entity references.

Two payloads are read by name, as a client knows them (pr-s27c C-8): conversation's `spoke`
(`speaker`, `listener`, `utterance`) and movement's `walk-ended` (`person`). Every other fact shows its
type and its participants; `detail` adds its payload as an envelope, capped at 1 KiB.
"""

from __future__ import annotations

import asyncio
from collections import deque
from collections.abc import Callable, Iterable
from dataclasses import dataclass

from mineworld_sdk.wire.contract import ActionCause, Observation, PerceivedEvent
from mineworld_sdk.wire.ids import ActionId, EntityId, JsonValue

from mineworld_agent.models import (
    AnyEvent,
    EventLine,
    OfferAvailable,
    SpokenTo,
    ViewChanged,
    WaitFor,
    WalkEnded,
    WorldText,
)
from mineworld_agent.refs import RefTable, UnknownRef
from mineworld_agent.view import envelope

type Fits = Callable[[list[EventLine], str | None], bool]
"""Whether a result built from these lines (and this `next`) fits the byte cap."""

INBOX_BOUND = 512
DETAIL_BYTES = 1024
SPOKE = "spoke"
WALK_ENDED = "walk-ended"


@dataclass(slots=True)
class _Entry:
    number: int
    event: PerceivedEvent
    read: bool = False
    shown: bool = False
    """Shown in an act's `accepted` outcome: never shown again."""


def _entity(value: JsonValue) -> EntityId | None:
    """A person in a pack payload, as `{ entity, entity_type }` or a bare id string."""
    if isinstance(value, dict) and isinstance(entity := value.get("entity"), str):
        return EntityId(entity)
    if isinstance(value, str):
        return EntityId(value)
    return None


def _field(event: PerceivedEvent, name: str) -> JsonValue:
    payload = event.payload.payload
    return payload.get(name) if isinstance(payload, dict) else None


class Inbox:
    """The bounded inbox of one handle."""

    def __init__(self, refs: RefTable, me: EntityId, bound: int = INBOX_BOUND) -> None:
        if not 1 <= bound <= INBOX_BOUND:
            raise ValueError(f"an inbox holds 1 to {INBOX_BOUND} facts, not {bound}")
        self._refs = refs
        self._me = me
        self._bound = bound
        self._entries: deque[_Entry] = deque()
        self._arrived = 0
        self._dropped = 0
        self.changed = asyncio.Event()
        """Set whenever a fact arrives; a waiter clears it before it waits."""

    # ── Filling ───────────────────────────────────────────────────────────────────────────────────

    def add(self, events: Iterable[PerceivedEvent]) -> None:
        for event in events:
            self._arrived += 1
            self._entries.append(_Entry(self._arrived, event))
            for subject in event.subjects:
                self._refs.ref(subject)
            if event.place is not None:
                self._refs.ref(event.place.entity)
            while len(self._entries) > self._bound:
                self._entries.popleft()
                self._dropped += 1
        self.changed.set()

    # ── Counting ──────────────────────────────────────────────────────────────────────────────────

    @property
    def unread(self) -> int:
        return sum(1 for entry in self._entries if not entry.read and not entry.shown)

    @property
    def dropped(self) -> int:
        return self._dropped

    @property
    def arrived(self) -> int:
        """Facts ever added: the number of the newest."""
        return self._arrived

    # ── Reading ───────────────────────────────────────────────────────────────────────────────────

    def line(self, entry: _Entry, detail: bool = False) -> EventLine:
        event = entry.event
        words = None
        if event.event_type == SPOKE and isinstance(said := _field(event, "utterance"), str):
            speaker = _entity(_field(event, "speaker"))
            words = WorldText(
                text=said, by=None if speaker is None else self._refs.ref(speaker), kind="speech"
            )
        return EventLine(
            ref=f"F{entry.number}",
            at=event.at,
            event_type=event.event_type,
            subjects=[self._refs.ref(subject) for subject in event.subjects],
            place=None if event.place is None else self._refs.ref(event.place.entity),
            words=words,
            detail=envelope(event.payload.payload, DETAIL_BYTES) if detail else None,
        )

    def events(
        self,
        after: str | None,
        limit: int,
        detail: bool = False,
        fits: Fits | None = None,
    ) -> tuple[list[EventLine], str | None]:
        """Up to `limit` facts after `after` (from the oldest held, when `None`), skipping those an act
        already showed; they are marked read. `next` is the last one returned when more remain. When
        `fits` refuses a page (the byte cap), it is shortened by one fact at a time; what is cut stays
        unread for the next page — never cut inside a fact."""
        start = 0 if after is None else _number(after)
        chosen = [e for e in self._entries if e.number > start and not e.shown][:limit]
        while True:
            lines = [self.line(e, detail) for e in chosen]
            remaining = any(
                e.number > (chosen[-1].number if chosen else start) and not e.shown
                for e in self._entries
            )
            following = f"F{chosen[-1].number}" if chosen and remaining else None
            if fits is None or len(chosen) <= 1 or fits(lines, following):
                break
            chosen = chosen[:-1]
        for entry in chosen:
            entry.read = True
        return lines, following

    def since(self, number: int, limit: int, fits: Fits | None = None) -> list[EventLine]:
        """Up to `limit` facts that arrived after fact `number`, marked read (for `wait`); shortened
        like `events` when `fits` refuses."""
        chosen = [e for e in self._entries if e.number > number and not e.shown][:limit]
        while True:
            lines = [self.line(e) for e in chosen]
            if fits is None or len(chosen) <= 1 or fits(lines, None):
                break
            chosen = chosen[:-1]
        for entry in chosen:
            entry.read = True
        return lines

    def caused_by(self, action: ActionId) -> list[EventLine]:
        """The facts an accepted request caused that this seat perceived, shown once: marked so that
        `events` never shows them again."""
        chosen = [
            e
            for e in self._entries
            if isinstance(e.event.caused_by, ActionCause) and e.event.caused_by.action == action
        ]
        for entry in chosen:
            entry.shown = True
        return [self.line(e) for e in chosen]

    # ── What `wait` waits for ─────────────────────────────────────────────────────────────────────

    def matches(self, until: WaitFor, after: int) -> bool:
        """Whether a fact that arrived after fact `after` is what `until` waits for (the fact-shaped
        conditions; `view_changed` and `offer_available` are the observation's, `offered`)."""
        fresh = [e.event for e in self._entries if e.number > after]
        match until:
            case AnyEvent():
                return bool(fresh)
            case SpokenTo():
                return any(
                    e.event_type == SPOKE and _entity(_field(e, "listener")) == self._me
                    for e in fresh
                )
            case WalkEnded():
                return any(
                    e.event_type == WALK_ENDED and _entity(_field(e, "person")) == self._me
                    for e in fresh
                )
            case ViewChanged() | OfferAvailable():
                return False


def offered(until: OfferAvailable, observation: Observation, refs: RefTable) -> bool:
    """Whether `observation` offers `until.action_type` (on `until.target`, when named) as available."""
    try:
        target = None if until.target is None else refs.resolve(until.target)
    except UnknownRef:
        return False
    return any(
        a.action_type == until.action_type
        and a.available
        and (until.target is None or a.target == target)
        for a in observation.affordances
    )


def _number(ref: str) -> int:
    return int(ref[1:])
