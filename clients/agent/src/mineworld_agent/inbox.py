"""A handle's inbox: the facts its seat perceived, bounded, as an agent reads them (step-27 §5.4).

The reliable `perceived` stream is drained into it continuously, so neither the SDK's buffer nor the
server's queue ever fills (INV-S27-6). The inbox holds at most `bound` facts (512 by default); when a new
fact arrives at the bound, the oldest is dropped and counted (`dropped`). Each fact gets an `EventRef`,
`F1`, `F2`, … in arrival order, and its subjects and place get the handle's own entity references.

Payloads read by name, as a client knows them (pr-s27c C-8): conversation's `spoke` (`speaker`, `listener`,
`utterance`) and movement's `walk-ended` (`person`); and, as `EventLine.reading` (pr-s31g §5.5), economy's
`sold`, `listed`, `price-set` and `withdrawn`, and the notices pack's `notice-published`, `subscribed` and
`unsubscribed`; and economy's `price-proposed`, `proposal-accepted`, `proposal-declined` and
`proposal-withdrawn` (pr-s31h §5.14). A payload that does not read gives no reading; the fact is still listed. Every other fact shows
its type and its participants; `detail` adds its payload as an envelope, capped at 1 KiB.
"""

from __future__ import annotations

import asyncio
from collections import deque
from collections.abc import Callable, Iterable
from dataclasses import dataclass
from functools import partial

from mineworld_sdk.wire.contract import ActionCause, Observation, PerceivedEvent
from mineworld_sdk.wire.ids import ActionId, EntityId, JsonValue

from mineworld_agent import disclosed
from mineworld_agent.lexicon import Lexicon
from mineworld_agent.models import (
    AnyEvent,
    AskReading,
    EventLine,
    EventRef,
    NoticeReading,
    OfferAvailable,
    ProposalReading,
    Reading,
    SaleReading,
    SpokenTo,
    TopicReading,
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
SOLD, LISTED, PRICE_SET, WITHDRAWN = "sold", "listed", "price-set", "withdrawn"
NOTICE_PUBLISHED, SUBSCRIBED, UNSUBSCRIBED = "notice-published", "subscribed", "unsubscribed"
PRICE_PROPOSED = "price-proposed"
PROPOSAL_FACTS = (PRICE_PROPOSED, "proposal-accepted", "proposal-declined", "proposal-withdrawn")
"""economy's four haggling facts (pr-s31h §5.7), each heard only by its buyer and those who may answer it."""


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


def _minor(value: JsonValue) -> int | None:
    return value if isinstance(value, int) and not isinstance(value, bool) and value >= 0 else None


def reading(event: PerceivedEvent, refs: RefTable, lexicon: Lexicon) -> Reading | None:
    """The fact read by name, for the economy and notices facts an agent trades and reads by; `None` for any
    other fact and for a payload that does not read (pr-s31g §5.5). The values are the payload's."""
    kind = event.event_type
    field = partial(_field, event)
    if kind == SOLD:
        seller, buyer = disclosed.any_entity(field("seller")), disclosed.any_entity(field("buyer"))
        item, where = disclosed.any_entity(field("item")), disclosed.any_entity(field("place"))
        count, price = _minor(field("count")), _minor(field("price"))
        if seller is None or buyer is None or item is None or count is None or price is None:
            return None
        return SaleReading(
            seller=refs.ref(seller),
            buyer=refs.ref(buyer),
            item=refs.ref(item),
            name=lexicon.name(item),
            count=count,
            price=lexicon.amount(price),
            place=None if where is None else refs.ref(where),
        )
    if kind in (LISTED, PRICE_SET, WITHDRAWN):
        seller, item = disclosed.any_entity(field("seller")), disclosed.any_entity(field("item"))
        now = _minor(field("price" if kind == LISTED else "to"))
        was = _minor(field("from")) if kind == PRICE_SET else None
        if seller is None or item is None or (kind != WITHDRAWN and now is None):
            return None
        if kind == PRICE_SET and was is None:
            return None
        return AskReading(
            seller=refs.ref(seller),
            item=refs.ref(item),
            name=lexicon.name(item),
            price=None if kind == WITHDRAWN or now is None else lexicon.amount(now),
            was=None if was is None else lexicon.amount(was),
        )
    if kind == NOTICE_PUBLISHED:
        notice = disclosed.notice_line(refs, field("notice"), lexicon)
        return None if notice is None else NoticeReading.model_validate(notice.model_dump())
    if kind in (SUBSCRIBED, UNSUBSCRIBED):
        topic = disclosed.topic(field("topic"))
        return None if topic is None else TopicReading(topic=topic)
    if kind in PROPOSAL_FACTS:
        return _proposal_reading(event, refs, lexicon)
    return None


def _proposal_reading(
    event: PerceivedEvent, refs: RefTable, lexicon: Lexicon
) -> ProposalReading | None:
    """economy's four proposal facts: `{ proposal, seller, buyer, item, price }`, and for `price-proposed` also
    `place?` and `valid_until` (pr-s31h §5.7)."""
    field = partial(_field, event)
    number, price = _minor(field("proposal")), _minor(field("price"))
    seller, buyer = disclosed.any_entity(field("seller")), disclosed.any_entity(field("buyer"))
    item, where = disclosed.any_entity(field("item")), disclosed.any_entity(field("place"))
    until = field("valid_until")
    if number is None or number < 1 or price is None:
        return None
    if seller is None or buyer is None or item is None:
        return None
    proposed = event.event_type == PRICE_PROPOSED
    if proposed and (not isinstance(until, int) or isinstance(until, bool)):
        return None
    return ProposalReading(
        proposal=number,
        seller=refs.ref(seller),
        buyer=refs.ref(buyer),
        item=refs.ref(item),
        name=lexicon.name(item),
        price=lexicon.amount(price),
        place=None if where is None else refs.ref(where),
        valid_until=until if proposed and isinstance(until, int) else None,
    )


class Inbox:
    """The bounded inbox of one handle."""

    def __init__(
        self,
        refs: RefTable,
        me: EntityId,
        bound: int = INBOX_BOUND,
        lexicon: Lexicon | None = None,
    ) -> None:
        if not 1 <= bound <= INBOX_BOUND:
            raise ValueError(f"an inbox holds 1 to {INBOX_BOUND} facts, not {bound}")
        self._refs = refs
        self._lexicon = Lexicon() if lexicon is None else lexicon
        """The handle's own lexicon: names and the currency for readings, as the views learned them."""
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
    def newest(self) -> EventRef | None:
        """The reference of the newest fact still held (S27-d's `inbox` status); `None` when none is held."""
        return f"F{self._entries[-1].number}" if self._entries else None

    @property
    def arrived(self) -> int:
        """Facts ever added: the number of the newest."""
        return self._arrived

    @property
    def read_through(self) -> int:
        """The number just before the oldest fact not yet read or shown (`arrived` when every fact
        has been): where `wait` starts looking, so a fact that arrived between two calls is not missed."""
        unread = next((e.number for e in self._entries if not e.read and not e.shown), None)
        return self._arrived if unread is None else unread - 1

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
            reading=reading(event, self._refs, self._lexicon),
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
        fresh = [e.event for e in self._entries if e.number > after and not e.shown]
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
