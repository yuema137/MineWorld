"""The notices pack's fact: `notice-published {notice: {id, topic, publisher, body, issued_at,
valid_until}}` (ARC-115; pr-s31b-notices §5.9).

A notice reaches its topic's subscribers and its publisher. The gist states when the notice stops being
valid, in recall's own convention (`Day <n> <HH:MM>`), so a reader can tell a stale notice from a current
one; the `(expired)` marker at recall is S28's to add (Q-S31b-3). Amounts are minor units with no currency:
the currency is economy's record, which this renderer does not read (SD-S31b-11).
"""

from __future__ import annotations

from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EntityIdField
from pydantic import model_validator

from mineworld_cognition.compress.summarize import clock, day_number
from mineworld_cognition.memory.names import NameBook, one_line
from mineworld_cognition.memory.render import (
    Payload,
    Rendered,
    counterparts,
    decode,
    fallback,
    quote,
)
from mineworld_cognition.memory.render.conversation import PersonRef

NOTICE = 400
"""Base importance of a notice: notable, and one I did not take part in, between the generic 300 and 500
(pr-s31b §5.9; a structural constant beside pr-fg-b §5.4's table, never fitted, ARC-23)."""

SHOWN_LINES = 6
"""Price lines a gist names before it says how many more there are."""

AN_ITEM = "an item"
"""What a price line's item is called when this seat knows no name for it. A rendered line never shows an
entity id (`names.py`, P4-5), so the design's `item <id>` is not used (pr-s31b D-S31b-4)."""


class PriceLine(Payload):
    item: EntityIdField
    price: int
    basis: int


class Body(Payload):
    """Externally tagged: exactly one of `prices` and `text`."""

    prices: list[PriceLine] | None = None
    text: str | None = None

    @model_validator(mode="after")
    def _one_kind(self) -> Body:
        if (self.prices is None) == (self.text is None):
            raise ValueError("a body is exactly one of prices and text")
        return self


class Notice(Payload):
    id: int
    topic: str
    publisher: PersonRef
    body: Body
    issued_at: int
    valid_until: int


class Published(Payload):
    notice: Notice


def _summary(body: Body, names: NameBook) -> str:
    if body.text is not None:
        return quote(body.text)
    lines = body.prices or []
    shown = [
        f"{one_line(names.name(line.item) or AN_ITEM)} {line.price}" for line in lines[:SHOWN_LINES]
    ]
    summary = ", ".join(shown)
    if len(lines) > SHOWN_LINES:
        summary += f" … and {len(lines) - SHOWN_LINES} more"
    return summary


def notice_published(fact: PerceivedEvent, me: EntityId, names: NameBook) -> Rendered:
    """`I posted <topic> (valid until Day n HH:MM): <summary>`, or `<publisher> posted …`."""
    payload = decode(Published, fact)
    if payload is None:
        return fallback(fact, me, names)
    notice = payload.notice
    publisher = notice.publisher.entity
    mine = publisher == me
    who = "I" if mine else names.person(publisher)
    until = f"Day {day_number(notice.valid_until)} {clock(notice.valid_until)}"
    gist = f"{who} posted {one_line(notice.topic)} (valid until {until}): {_summary(notice.body, names)}"
    return Rendered(gist, counterparts(fact, me), "notable", mine, NOTICE)
