"""The notice board and my subscriptions in the agent's view (pr-s31g §§5.3-5.4, C3).

The notices pack discloses to me alone my `subscriptions { topics }` and my `notice-board { notices }`: the
notices current at that instant on the topics I subscribe to or publish (`systems/notices/src/system.rs`). The
view shows the board as disclosed, whole and in its order, paged; it compares no time with `valid_until`
(M-g-3). Records are planted in the merged shapes (`board.rs`, `body.rs`: a price line's `item` is a bare id).
"""

from __future__ import annotations

from mineworld_sdk.wire.ids import EntityId
from planted import (
    INBOX,
    MOMENT,
    Raw,
    catalogue_names,
    me,
    person,
    plant,
    recorded,
    remove,
    rendered_of,
    shown,
)

from mineworld_agent.models import NoticeLine
from mineworld_agent.player import RESULT_MAX_BYTES, WRAPPER_BYTES
from mineworld_agent.view import Caps, delta_of, page_of, view_of

GRACE = "11"
"""Another person of the recorded café, as the publisher of planted notices."""


def text_notice(n: int, topic: str, words: str, issued_at: int, valid_until: int) -> Raw:
    return {
        "id": n,
        "topic": topic,
        "publisher": person(GRACE),
        "body": {"text": words},
        "issued_at": issued_at,
        "valid_until": valid_until,
    }


def prices_notice(n: int, lines: list[tuple[str, int, int]]) -> Raw:
    return {
        "id": n,
        "topic": "prices.ingredients",
        "publisher": person(GRACE),
        "body": {"prices": [{"item": i, "price": p, "basis": b} for i, p, b in lines]},
        "issued_at": 50,
        "valid_until": 3650,
    }


def board(value: Raw, notices: list[Raw], topics: list[str]) -> None:
    plant(value, me(value), "subscriptions", {"topics": topics})
    plant(value, me(value), "notice-board", {"notices": notices})


def test_the_board_and_my_subscriptions_are_shown_as_disclosed() -> None:
    value = recorded()
    names = catalogue_names(value)
    kinds = sorted(names)[:2]
    board(
        value,
        [
            text_notice(1, "news.daily", "Market opens at 8", 40, 7240),
            prices_notice(2, [(kinds[0], 300, 6), (kinds[1], 95, 2)]),
        ],
        ["news.daily", "prices.ingredients"],
    )
    view, refs = shown(value)
    assert view.me.subscriptions == ["news.daily", "prices.ingredients"]
    text, prices = view.notices
    assert (text.topic, text.publisher, text.issued_at, text.valid_until) == (
        "news.daily",
        refs.ref(EntityId(GRACE)),
        40,
        7240,
    )
    assert text.text is not None and text.text.text == "Market opens at 8"
    assert (text.text.kind, text.text.by) == ("speech", refs.ref(EntityId(GRACE)))
    assert text.prices == []
    assert prices.text is None
    assert [(p.item, p.price.minor, p.price.display, p.basis) for p in prices.prices] == [
        (refs.ref(EntityId(kinds[0])), 300, "3.00", 6),
        (refs.ref(EntityId(kinds[1])), 95, "0.95", 2),
    ]
    assert [p.name.text if p.name else None for p in prices.prices] == [
        names[kinds[0]],
        names[kinds[1]],
    ]


def test_the_board_is_shown_as_disclosed_even_a_notice_past_its_time() -> None:
    # The pack lists only current notices; the adapter never filters by `valid_until` (M-g-3).
    value = recorded()
    at = value["at"]
    board(value, [text_notice(1, "news.daily", "old news", at - 7200, at - 1)], ["news.daily"])
    view, _ = shown(value)
    assert [n.valid_until for n in view.notices] == [at - 1]


def test_twelve_notices_are_paged_in_board_order() -> None:
    value = recorded()
    board(
        value,
        [text_notice(n, "news.daily", f"notice {n}", 10, 3610 + n) for n in range(12)],
        ["news.daily"],
    )
    rendered, _ = rendered_of(value)
    first = view_of(rendered, MOMENT, INBOX, Caps())
    assert len(first.notices) == Caps().notices == 8
    seen = [n.valid_until for n in first.notices]
    pending = [p for p in first.more if p.list == "notices"]
    while pending:
        page = page_of(rendered, pending.pop(0), MOMENT, INBOX, Caps())
        seen += [n.valid_until for n in page.notices]
        pending += [p for p in page.more if p.list == "notices"]
    assert seen == [3610 + n for n in range(12)]


PLAYER_CAPS = Caps(max_bytes=RESULT_MAX_BYTES - WRAPPER_BYTES)
"""The cap a `Player` renders views under: the result's 8 KiB less the room an act's outcome takes."""


def largest_notice(price: int, basis: int) -> Raw:
    """A 32-line prices notice (the pack's most) whose 32 items each have a 64-byte name (item's most)."""
    value = recorded()
    kinds = [str(7000 + n) for n in range(32)]
    plant(
        value,
        value["self_location"]["place"]["entity"],
        "item-catalogue",
        {
            "kinds": [
                {"item": {"entity": k, "entity_type": "item"}, "category": "food", "name": "N" * 64}
                for k in kinds
            ]
        },
    )
    board(value, [prices_notice(1, [(k, price, basis) for k in kinds])], [])
    return value


def first_notice(value: Raw) -> NoticeLine:
    """The one notice, from the first view or, when the fitting loop gave the room to other lists, its page."""
    rendered, _ = rendered_of(value)
    first = view_of(rendered, MOMENT, INBOX, PLAYER_CAPS)
    if not first.notices:
        (more,) = [p for p in first.more if p.list == "notices"]
        first = page_of(rendered, more, MOMENT, INBOX, PLAYER_CAPS)
    (notice,) = first.notices
    return notice


def test_a_32_line_notice_with_64_byte_names_fits_a_page_with_its_names() -> None:
    notice = first_notice(largest_notice(450, 6))
    assert len(notice.prices) == 32
    assert all(p.name is not None and p.name.text == "N" * 64 for p in notice.prices)


def test_the_largest_notice_at_the_largest_prices_still_fits_without_its_names() -> None:
    # 32 lines at 10^12 minor units and basis 65 535 with 64-byte names do not fit one page; the view then
    # drops the lines' names (each item still answers `inspect`) rather than refuse the observation.
    notice = first_notice(largest_notice(1_000_000_000_000, 65_535))
    assert len(notice.prices) == 32
    assert all(p.name is None and p.price.minor == 1_000_000_000_000 for p in notice.prices)


def test_without_economy_a_notice_price_has_no_display() -> None:
    # W3's shape: notices installed, no currency disclosed (social-cafe has no economy).
    value = recorded()
    remove(value, me(value), "currency")
    kind = sorted(catalogue_names(value))[0]
    board(value, [prices_notice(1, [(kind, 300, 1)])], ["prices.ingredients"])
    view, _ = shown(value)
    (line,) = view.notices[0].prices
    assert (line.price.minor, line.price.display) == (300, None)


def test_a_new_notice_is_carried_by_a_delta() -> None:
    before = recorded()
    board(before, [], ["news.daily"])
    after = recorded()
    board(after, [text_notice(1, "news.daily", "hello", 10, 3610)], ["news.daily"])
    base, refs = rendered_of(before)
    newest, _ = rendered_of(after, refs=refs, number=2)
    delta = delta_of(base, newest, MOMENT, INBOX, Caps(max_bytes=1 << 20))
    assert delta is not None and delta.notices is not None
    assert [n.text.text for n in delta.notices if n.text] == ["hello"]
    same = delta_of(newest, newest, MOMENT, INBOX, Caps())
    assert same is not None and same.notices is None


def test_a_topic_that_is_no_topic_name_is_left_out() -> None:
    value = recorded()
    board(value, [text_notice(1, "Not A Topic", "x", 1, 2)], ["news.daily", "Bad Topic"])
    view, _ = shown(value)
    assert view.me.subscriptions == ["news.daily"]
    assert view.notices == []
