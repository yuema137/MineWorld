"""The inbox reads sales, ask changes, notices and subscription changes by name (pr-s31g §5.5, C4).

Facts are built in the contract's JSON (`PerceivedEvent`), each payload exactly as the merged pack writes it
(C0's table): economy's `sold { seller, buyer, item, count, price, place }` with bare-string sellers and buyers,
typed items and places; `listed`, `price-set { seller, item, from, to }`, `withdrawn`; the notices pack's
`notice-published { notice }` and `subscribed` / `unsubscribed { person, topic }`. Expected values are the
literals written here.
"""

from __future__ import annotations

import json

from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, JsonValue

from mineworld_agent.inbox import Inbox
from mineworld_agent.lexicon import Currency, Lexicon
from mineworld_agent.models import (
    AskReading,
    CurrencyLine,
    EventLine,
    NoticeReading,
    SaleReading,
    TopicReading,
    WorldText,
)
from mineworld_agent.refs import RefTable

ME, GRACE, CAFE, FLOWERS = "10", "11", "2", "40"


def item(entity: str) -> JsonValue:
    return {"entity": entity, "entity_type": "item"}


def fact(n: int, kind: str, payload: object, owner: str = "economy") -> PerceivedEvent:
    """One perceived fact; `payload` is the pack's JSON, written as a literal by the test."""
    value: dict[str, object] = {
        "id": str(100 + n),
        "at": n,
        "event_type": kind,
        "subjects": [GRACE, ME],
        "participants": [GRACE, ME],
        "place": {"entity": CAFE, "entity_type": "place"},
        "caused_by": "world_genesis",
        "payload": {"event_type": kind, "schema_version": 1, "payload": payload},
        "visibility": "public",
        "provenance": {"emitted_by": owner, "controller_decision": None},
    }
    return PerceivedEvent.model_validate_json(json.dumps(value))


def dollars() -> Lexicon:
    lexicon = Lexicon()
    line = CurrencyLine(
        name=WorldText(text="US dollar", kind="name"),
        symbol=WorldText(text="$", kind="label"),
        exponent=2,
        symbol_at="before",
    )
    lexicon.learn(Currency(line=line, references={}), {EntityId(FLOWERS): "Flowers"})
    return lexicon


def read(
    events: list[PerceivedEvent], lexicon: Lexicon | None = None
) -> tuple[list[EventLine], RefTable]:
    refs = RefTable()
    inbox = Inbox(refs, EntityId(ME), lexicon=dollars() if lexicon is None else lexicon)
    inbox.add(events)
    lines, _ = inbox.events(None, 20)
    return lines, refs


def test_a_sale_reads_as_who_sold_what_to_whom_at_what_price() -> None:
    sold = {
        "seller": GRACE,
        "buyer": ME,
        "item": item(FLOWERS),
        "count": 1,
        "price": 450,
        "place": {"entity": CAFE, "entity_type": "place"},
    }
    (line,), refs = read([fact(1, "sold", sold)])
    assert isinstance(line.reading, SaleReading)
    reading = line.reading
    assert (reading.seller, reading.buyer, reading.item, reading.place) == (
        refs.ref(EntityId(GRACE)),
        refs.ref(EntityId(ME)),
        refs.ref(EntityId(FLOWERS)),
        refs.ref(EntityId(CAFE)),
    )
    assert (reading.count, reading.price.minor, reading.price.display) == (1, 450, "4.50")
    assert reading.name is not None and reading.name.text == "Flowers"
    assert line.words is None


def test_listed_price_set_and_withdrawn_read_as_ask_changes() -> None:
    lines, refs = read(
        [
            fact(1, "listed", {"seller": GRACE, "item": item(FLOWERS), "price": 120}),
            fact(2, "price-set", {"seller": GRACE, "item": item(FLOWERS), "from": 120, "to": 150}),
            fact(3, "withdrawn", {"seller": GRACE, "item": item(FLOWERS)}),
        ]
    )
    readings = [line.reading for line in lines]
    assert all(isinstance(r, AskReading) for r in readings)
    shown = [
        (
            line.event_type,
            r.seller,
            None if r.price is None else r.price.minor,
            None if r.was is None else r.was.minor,
        )
        for line, r in zip(lines, readings, strict=True)
        if isinstance(r, AskReading)
    ]
    grace = refs.ref(EntityId(GRACE))
    assert shown == [
        ("listed", grace, 120, None),
        ("price-set", grace, 150, 120),
        ("withdrawn", grace, None, None),
    ]


def test_a_published_notice_reads_as_its_board_line() -> None:
    notice = {
        "id": 7,
        "topic": "prices.ingredients",
        "publisher": {"entity": GRACE, "entity_type": "person"},
        "body": {"prices": [{"item": FLOWERS, "price": 300, "basis": 6}]},
        "issued_at": 40,
        "valid_until": 7240,
    }
    (line,), refs = read([fact(1, "notice-published", {"notice": notice}, owner="notices")])
    reading = line.reading
    assert isinstance(reading, NoticeReading)
    assert (reading.topic, reading.publisher, reading.issued_at, reading.valid_until) == (
        "prices.ingredients",
        refs.ref(EntityId(GRACE)),
        40,
        7240,
    )
    (price,) = reading.prices
    assert (price.price.minor, price.price.display, price.basis) == (300, "3.00", 6)
    assert price.name is not None and price.name.text == "Flowers"


def test_a_text_notice_is_the_publishers_speech() -> None:
    notice = {
        "id": 8,
        "topic": "news.daily",
        "publisher": {"entity": GRACE, "entity_type": "person"},
        "body": {"text": "Market opens at 8"},
        "issued_at": 40,
        "valid_until": 7240,
    }
    (line,), refs = read([fact(1, "notice-published", {"notice": notice}, owner="notices")])
    reading = line.reading
    assert isinstance(reading, NoticeReading) and reading.text is not None
    assert (reading.text.text, reading.text.kind, reading.text.by) == (
        "Market opens at 8",
        "speech",
        refs.ref(EntityId(GRACE)),
    )


def test_subscribed_and_unsubscribed_read_as_the_topic() -> None:
    person = {"entity": ME, "entity_type": "person"}
    lines, _ = read(
        [
            fact(1, "subscribed", {"person": person, "topic": "news.daily"}, owner="notices"),
            fact(2, "unsubscribed", {"person": person, "topic": "news.daily"}, owner="notices"),
        ]
    )
    assert [(line.event_type, line.reading) for line in lines] == [
        ("subscribed", TopicReading(topic="news.daily")),
        ("unsubscribed", TopicReading(topic="news.daily")),
    ]


def test_a_payload_that_does_not_read_gives_no_reading_and_the_fact_is_still_listed() -> None:
    lines, _ = read(
        [
            fact(1, "sold", {"seller": GRACE, "item": item(FLOWERS)}),
            fact(2, "price-set", {"seller": GRACE, "item": item(FLOWERS), "to": 150}),
            fact(3, "subscribed", {"topic": "Not A Topic"}, owner="notices"),
            fact(4, "tick", {}),
        ]
    )
    assert [(line.event_type, line.reading) for line in lines] == [
        ("sold", None),
        ("price-set", None),
        ("subscribed", None),
        ("tick", None),
    ]


def test_without_a_currency_a_sale_has_no_display() -> None:
    sold = {
        "seller": GRACE,
        "buyer": ME,
        "item": item(FLOWERS),
        "count": 2,
        "price": 3,
        "place": None,
    }
    (line,), _ = read([fact(1, "sold", sold)], Lexicon())
    assert isinstance(line.reading, SaleReading)
    assert (line.reading.price.minor, line.reading.price.display, line.reading.name) == (
        3,
        None,
        None,
    )
    assert line.reading.place is None
