"""The market in the agent's view (pr-s31g §§5.2-5.4, C2): the currency, amounts with `display`, my wallet and
wealth, my asks, the asks of the people I perceive, and the price of every `buy` offer.

Built on a real post-S31-a observation with economy's records planted in the shapes C0 pinned (`planted.py`).
"""

from __future__ import annotations

import copy
import json

import pytest
from mineworld_sdk.wire.contract import Observation
from mineworld_sdk.wire.ids import EntityId
from planted import (
    INBOX,
    MOMENT,
    Raw,
    catalogue_names,
    entity,
    item,
    me,
    payload,
    place,
    plant,
    recorded,
    rendered_of,
    shown,
)

from mineworld_agent.lexicon import Lexicon, display_of
from mineworld_agent.refs import RefTable
from mineworld_agent.view import Caps, delta_of, page_of, render, view_of


@pytest.mark.parametrize(
    ("minor", "exponent", "wanted"),
    [
        (450, 2, "4.50"),
        (5, 2, "0.05"),
        (450, 0, "450"),
        (450, 3, "0.450"),
        (123_456, 4, "12.3456"),
        (0, 2, "0.00"),
        (450, None, None),
    ],
)
def test_display_is_minor_units_in_major_units_by_the_disclosed_exponent(
    minor: int, exponent: int | None, wanted: str | None
) -> None:
    assert display_of(minor, exponent) == wanted


def test_the_currency_wallet_and_wealth_are_shown_as_disclosed() -> None:
    value = recorded()
    view, _ = shown(value)
    assert view.currency is not None
    told = payload(value, me(value), "currency")
    assert (view.currency.name.text, view.currency.symbol.text) == (told["name"], told["symbol"])
    assert (view.currency.name.kind, view.currency.symbol.kind) == ("name", "label")
    assert (view.currency.exponent, view.currency.symbol_at) == (2, "before")
    balance = payload(value, me(value), "wallet")["balance"]
    assert view.me.wallet is not None and view.me.wallet.minor == balance == 20000
    assert view.me.wallet.display == "200.00"
    worth = payload(value, me(value), "wealth")
    assert view.me.wealth is not None
    assert (view.me.wealth.amount.minor, view.me.wealth.unvalued) == (
        worth["amount"],
        worth["unvalued"],
    )


def test_every_buy_shows_its_own_payloads_price_and_its_items_name() -> None:
    value = recorded()
    view, refs = shown(value)
    names = catalogue_names(value)
    sent = [a for a in value["affordances"] if a["action_type"] == "buy"]
    buys = [o for o in view.offers if o.action_type == "buy"]
    assert len(buys) == len(sent) >= 5
    for line, raw_offer in zip(buys, sent, strict=True):
        price, kind = raw_offer["payload"]["price"], raw_offer["payload"]["item"]["entity"]
        assert line.price is not None and line.price.minor == price
        assert line.price.display == f"{price // 100}.{price % 100:02d}"
        assert line.item_name is not None and line.item_name.text == names[kind]
        assert line.about == [refs.ref(EntityId(kind))]
    others = [o for o in view.offers if o.action_type != "buy"]
    assert others and all(o.price is None and o.item_name is None for o in others)


def test_a_cafe_buy_reads_110_minor_units_as_1_10() -> None:
    view, _ = shown(recorded())
    cake = next(o for o in view.offers if o.price is not None and o.price.minor == 110)
    assert cake.price is not None and cake.price.display == "1.10"
    assert cake.item_name is not None and cake.item_name.text == "Cake"


def person_seller(value: Raw, asks: list[tuple[str, int, bool]]) -> str:
    """Plants an `asks` view on the first other person, and one `buy` per ask aimed at that person, as economy
    discloses and offers them (`component.rs` `AsksView`; `listing.rs` person buys)."""
    seller = next(
        e["id"] for e in value["entities"] if e["entity_type"] == "person" and e["id"] != me(value)
    )
    plant(
        value,
        seller,
        "asks",
        {"asks": [{"item": item(i), "price": p, "available": a} for i, p, a in asks]},
    )
    template = next(a for a in value["affordances"] if a["action_type"] == "buy")
    for i, p, _ in asks:
        buy = copy.deepcopy(template)
        buy["target"] = seller
        buy["payload"] = {"item": item(i), "price": p}
        value["affordances"].append(buy)
    return seller


def test_a_person_sellers_asks_and_buys_carry_her_prices_not_the_shops() -> None:
    value = recorded()
    names = catalogue_names(value)
    shop_kind = payload(value, place(value), "shop")["listed"][0]
    kind, shop_price = shop_kind["item"]["entity"], shop_kind["price"]
    seller = person_seller(value, [(kind, 450, True)])
    view, refs = shown(value)
    line = next(p for p in view.here.people if p.ref == refs.ref(EntityId(seller)))
    assert line.asks_total == 1 and len(line.asks) == 1
    (ask,) = line.asks
    assert (ask.item, ask.price.minor, ask.price.display, ask.available) == (
        refs.ref(EntityId(kind)),
        450,
        "4.50",
        True,
    )
    assert ask.name is not None and ask.name.text == names[kind]
    rendered, refs = rendered_of(value)  # every offer, past the first page
    hers = next(
        o
        for o in rendered.offers
        if o.action_type == "buy" and o.target == refs.ref(EntityId(seller))
    )
    assert hers.price is not None and hers.price.minor == 450 != shop_price


def test_an_item_payload_that_is_no_buy_carries_no_price() -> None:
    # Before S31-g a `give` of a kind the shop lists showed the shop's price (F-S31g-4).
    value = recorded()
    kind = payload(value, place(value), "shop")["listed"][0]["item"]["entity"]
    template = next(a for a in value["affordances"] if a["action_type"] == "buy")
    give = copy.deepcopy(template)
    give["action_type"] = "give"
    give["payload"] = {"item": item(kind), "count": 1}
    value["affordances"].append(give)
    rendered, _ = rendered_of(value)  # every offer, past the first page
    line = next(o for o in rendered.offers if o.action_type == "give")
    assert (line.price, line.item_name, line.reference) == (None, None, None)


def test_a_world_with_exponent_0_displays_whole_units() -> None:
    value = recorded()
    plant(
        value,
        me(value),
        "currency",
        {"name": "crown", "symbol": "cr", "exponent": 0, "symbol_at": "after", "references": []},
    )
    view, _ = shown(value)
    assert view.currency is not None
    assert (view.currency.name.text, view.currency.exponent, view.currency.symbol_at) == (
        "crown",
        0,
        "after",
    )
    assert view.me.wallet is not None and view.me.wallet.display == "20000"
    for line in view.offers:
        if line.price is not None:
            assert line.price.display == str(line.price.minor)


def test_without_a_currency_no_amount_is_displayed() -> None:
    value = recorded()
    entity(value, me(value))["components"] = [
        c for c in entity(value, me(value))["components"] if c["component_type"] != "currency"
    ]
    view, _ = shown(value)
    assert view.currency is None
    assert view.me.wallet is not None and view.me.wallet.display is None
    assert all(o.price is None or o.price.display is None for o in view.offers)


def test_reference_values_reach_holdings_asks_and_buys() -> None:
    value = recorded()
    held = payload(value, me(value), "holdings")["held"][0]["item"]["entity"]
    plant(
        value,
        me(value),
        "currency",
        {
            "name": "US dollar",
            "symbol": "$",
            "exponent": 2,
            "symbol_at": "before",
            "references": [{"item": item(held), "value": 75}],
        },
    )
    view, refs = shown(value)
    count = next(c for c in view.me.holdings if c.item == refs.ref(EntityId(held)))
    assert count.reference is not None and (count.reference.minor, count.reference.display) == (
        75,
        "0.75",
    )
    priced = [
        o for o in view.offers if o.about == [refs.ref(EntityId(held))] and o.price is not None
    ]
    assert priced and all(o.reference is not None and o.reference.minor == 75 for o in priced)


def test_one_persons_asks_show_eight_and_say_how_many() -> None:
    value = recorded()
    seller = person_seller(value, [(str(5000 + n), 100 + n, n % 2 == 0) for n in range(30)])
    view, refs = shown(value)
    line = next(p for p in view.here.people if p.ref == refs.ref(EntityId(seller)))
    assert line.asks_total == 30
    assert [a.price.minor for a in line.asks] == [100 + n for n in range(8)]


def test_my_own_asks_are_paged() -> None:
    value = recorded()
    plant(
        value,
        me(value),
        "asks",
        {
            "asks": [
                {"item": item(str(6000 + n)), "price": 10 + n, "available": True} for n in range(40)
            ]
        },
    )
    rendered, _ = rendered_of(value)
    first = view_of(rendered, MOMENT, INBOX, Caps())
    assert 1 <= len(first.me.asks) <= Caps().asks
    seen = [a.price.minor for a in first.me.asks]
    pending = [p for p in first.more if p.list == "asks"]
    assert pending, first.more
    while pending:
        page = page_of(rendered, pending.pop(0), MOMENT, INBOX, Caps())
        seen += [a.price.minor for a in page.me.asks]
        pending += [p for p in page.more if p.list == "asks"]
    assert seen == [10 + n for n in range(40)]


def test_a_change_to_my_asks_answers_a_whole_view_and_a_currency_change_is_carried() -> None:
    before = recorded()
    plant(
        before, me(before), "asks", {"asks": [{"item": item("22"), "price": 10, "available": True}]}
    )
    after = copy.deepcopy(before)
    plant(
        after, me(after), "asks", {"asks": [{"item": item("22"), "price": 12, "available": True}]}
    )
    lexicon, refs = Lexicon(), RefTable()
    base = render(
        Observation.model_validate_json(json.dumps(before)), refs, 1, True, lexicon=lexicon
    )
    newest = render(
        Observation.model_validate_json(json.dumps(after)), refs, 2, True, lexicon=lexicon
    )
    assert delta_of(base, newest, MOMENT, INBOX, Caps()) is None
    plain = recorded()
    base = render(
        Observation.model_validate_json(json.dumps(plain)), refs, 4, True, lexicon=lexicon
    )
    renamed = copy.deepcopy(plain)
    plant(
        renamed,
        me(renamed),
        "currency",
        {"name": "crown", "symbol": "cr", "exponent": 0, "symbol_at": "after", "references": []},
    )
    third = render(
        Observation.model_validate_json(json.dumps(renamed)), refs, 3, True, lexicon=lexicon
    )
    delta = delta_of(base, third, MOMENT, INBOX, Caps(max_bytes=1 << 20))  # every offer re-priced
    assert delta is not None and delta.currency is not None and delta.currency.exponent == 0


def test_the_lexicon_keeps_item_names_learned_in_another_place() -> None:
    value = recorded()
    held = payload(value, me(value), "holdings")["held"][0]["item"]["entity"]
    name = catalogue_names(value)[held]
    lexicon = Lexicon()
    rendered_of(value, lexicon)
    away = copy.deepcopy(value)
    here = entity(away, place(away))
    here["components"] = [c for c in here["components"] if c["component_type"] != "item-catalogue"]
    without, refs = rendered_of(away)
    kept, kept_refs = rendered_of(away, lexicon)
    assert next(c for c in without.me.holdings if c.item == refs.ref(EntityId(held))).name is None
    named = next(c for c in kept.me.holdings if c.item == kept_refs.ref(EntityId(held))).name
    assert named is not None and named.text == name


def test_the_symbol_never_enters_display() -> None:
    value = recorded()
    view, _ = shown(value)
    text = view.model_dump_json()
    assert view.currency is not None
    symbol = view.currency.symbol.text
    displays = [o.price.display for o in view.offers if o.price is not None and o.price.display]
    assert displays and all(symbol not in d for d in displays)
    assert text.count(json.dumps(symbol)) == 1, "the symbol appears once, in View.currency"
