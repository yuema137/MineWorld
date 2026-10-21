"""An MCP agent shops, sells and reads notices against the real server, on three generated worlds (pr-s31g §8,
AC-g-1 … AC-g-6; step-31 AC-S31-8, AC-S31-6's agent half, X-9 through the agent).

The worlds are scratch copies the test builds, as `test_mcp_same_answers.py` builds its own (A-18):

- **W1 `market-g`**: Market Town with Grace in the café 1 m from the visitor, holding two flowers she asks 4.50
  for; Felix a keeper of the corner store; `notices` enabled with `news.daily` (text, the visitor publishes) and
  `prices.ingredients` (prices, Felix publishes); Grace subscribes to both.
- **W2 `market-g-crown`**: W1 whose currency is the crown (`cr`, exponent 0, after): the same code reads every
  amount differently.
- **W3 `cafe-notices`**: Social Café with only `notices` added: notices with no economy, so no amount has a
  `display`.

The server runs without `--town` (A-19): nobody moves unless a client asks. Every oracle is the server's: a plain
SDK client's own observation and answers, and the facts the adapter's outcome carries. The file runs on the
platform's default loop (F-S27d-6): the adapter is a subprocess. A missing binary fails the test; it never skips.
"""

from __future__ import annotations

import shutil
from collections.abc import Iterator
from pathlib import Path
from typing import Any, cast

import pytest
from mcp import ClientSession
from mcp_relay import Relay
from mineworld_sdk import SeatSession, offers
from mineworld_sdk.wire.contract import Observation, PerceivedEntity
from mineworld_sdk.wire.ids import ActionTypeId, JsonValue
from realserver import REPOSITORY, Server, seated
from support import run_default
from test_play_mcp import Transcript, adapter, call

pytestmark = pytest.mark.real_server

NOTICES = """\
topics:
  news.daily:
    body: text
    publishers: [{news}]
    valid_for: {{ min: "1h", max: "7d" }}
    per_day: 4
  prices.ingredients:
    body: prices
    publishers: [{prices}]
    valid_for: {{ min: "1h", max: "48h" }}
    per_day: 4
"""
CROWN = 'currency:\n  name: crown\n  symbol: "cr"\n  exponent: 0\n  symbol_at: after\n'
RESULT_CAP = 8192


def _once(text: str, old: str, new: str) -> str:
    assert text.count(old) == 1, f"exactly one {old!r}"
    return text.replace(old, new, 1)


def _edit(path: Path, old: str, new: str) -> None:
    path.write_text(
        _once(path.read_text(encoding="utf-8").replace("\r\n", "\n"), old, new), "utf-8"
    )


def _append(path: Path, text: str) -> None:
    path.write_text(path.read_text(encoding="utf-8") + text, encoding="utf-8")


def _with_notices(world: Path, after: str, configuration: str) -> None:
    """`notices` enabled after the system `after`, and configured: the Rust test's recipe (`notices.rs`)."""
    _edit(world / "world.yaml", f"  - {after}\n", f"  - {after}\n  - notices\n")
    _edit(world / "world.yaml", "\nconfigure:\n", "\nconfigure:\n  - notices\n")
    (world / "configure").mkdir(exist_ok=True)
    (world / "configure" / "notices.yaml").write_text(configuration, encoding="utf-8")


def market_g(root: Path, *, crowns: bool = False) -> Path:
    """W1 (or W2 with `crowns`), built from `worlds/market-town`."""
    world = root / ("market-g-crown" if crowns else "market-g")
    shutil.copytree(REPOSITORY / "worlds" / "market-town", world)
    grace = world / "people" / "grace.yaml"
    _edit(
        grace,
        "location:\n  place: workplace\n  position:\n    x: 3000\n    y: -3000\n",
        "location:\n  place: cafe\n  position:\n    x: 2610\n    y: 600\n",
    )
    _edit(
        grace,
        "holdings: { candle: 1, soap: 1, umbrella: 1 }",
        "holdings: { candle: 1, flowers: 2 }",
    )
    _edit(
        grace, "economy: { wallet: 200000 }", "economy: { wallet: 200000, asks: { flowers: 450 } }"
    )
    _append(grace, "notices: { subscribe: [news.daily, prices.ingredients] }\n")
    _edit(
        world / "organizations" / "corner-store.yaml",
        "    prices: { apple: 55, bread: 130, milk: 45, juice: 40 }\n",
        "    prices: { apple: 55, bread: 130, milk: 45, juice: 40 }\n    keepers: [felix]\n",
    )
    _with_notices(world, "posture", NOTICES.format(news="visitor", prices="felix"))
    if crowns:
        economy = world / "configure" / "economy.yaml"
        text = economy.read_text(encoding="utf-8").replace("\r\n", "\n")
        start, end = text.index("currency:\n"), text.index("symbol_at: before\n")
        economy.write_text(text[:start] + CROWN + text[end + len("symbol_at: before\n") :], "utf-8")
    return world


def cafe_notices(root: Path) -> Path:
    """W3, built from `worlds/social-cafe`: notices and nothing of economy."""
    world = root / "cafe-notices"
    shutil.copytree(REPOSITORY / "worlds" / "social-cafe", world)
    _with_notices(world, "appearance", NOTICES.format(news="visitor", prices="wanderer"))
    return world


@pytest.fixture
def scratch(tmp_path: Path) -> Iterator[list[Server]]:
    started: list[Server] = []
    try:
        yield started
    finally:
        for server in started:
            server.stop()


def _serve(started: list[Server], world: Path) -> Server:
    server = Server(str(world), "--grant", "visitor")
    started.append(server)
    return server


# ── Reading results ───────────────────────────────────────────────────────────────────────────────


async def _view(session: ClientSession, transcript: Transcript, handle: str) -> dict[str, Any]:
    return cast(
        dict[str, Any], (await call(session, transcript, "observe", {"handle": handle}))["view"]
    )


async def _every_offer(
    session: ClientSession, transcript: Transcript, handle: str, view: dict[str, Any]
) -> list[dict[str, Any]]:
    """Every offer of `view`, over its pages."""
    found = list(view["offers"])
    pages = [page for page in view.get("more", []) if page["list"] == "offers"]
    while pages:
        page = (await call(session, transcript, "observe", {"handle": handle, "page": pages[0]}))[
            "view"
        ]
        found += page["offers"]
        pages = [p for p in page.get("more", []) if p["list"] == "offers"]
    return found


def _person(view: dict[str, Any], name: str) -> dict[str, Any]:
    return next(
        p for p in view["here"]["people"] if p["name"] and p["name"]["text"].startswith(name)
    )


async def _act(
    session: ClientSession, transcript: Transcript, handle: str, act: dict[str, Any]
) -> dict[str, Any]:
    return await call(session, transcript, "act", {"handle": handle, "act": act})


def _readings(answer: dict[str, Any], event_type: str) -> list[dict[str, Any]]:
    events = answer["outcome"].get("events", [])
    return [e["reading"] for e in events if e["event_type"] == event_type and e.get("reading")]


def _rejected(answer: dict[str, Any], code: str) -> None:
    outcome = answer["outcome"]
    assert outcome["kind"] == "rejected" and outcome.get("system_code") == code, outcome


def _display(minor: int, crowns: bool) -> str:
    return str(minor) if crowns else f"{minor // 100}.{minor % 100:02d}"


def _component(entity: PerceivedEntity, kind: str) -> JsonValue:
    return next((c.payload for c in entity.components if c.component_type == kind), None)


def _me(observation: Observation) -> PerceivedEntity:
    return next(e for e in observation.entities if e.id == observation.observer)


def _place(observation: Observation) -> PerceivedEntity:
    assert observation.self_location is not None
    here = observation.self_location.place.entity
    return next(e for e in observation.entities if e.id == here)


async def _plain_request(session: SeatSession, action: str, payload: object) -> Any:  # noqa: ANN401
    """A plain client's request through the SDK's own offer matching; the server's answer. `payload` is the
    pack's JSON, written as a literal by the test."""
    newest = session.newest
    assert newest is not None
    body = cast(JsonValue, payload)
    request = offers.request(newest.observation, ActionTypeId(action), target=None, payload=body)
    return getattr(await session.submit(request), "result", None)


# ── W1 and W2 ─────────────────────────────────────────────────────────────────────────────────────


@pytest.mark.parametrize("crowns", [False, True], ids=["dollars", "crowns"])
def test_an_agent_shops_sells_and_reads_notices(
    crowns: bool, scratch: list[Server], tmp_path: Path
) -> None:
    server = _serve(scratch, market_g(tmp_path, crowns=crowns))
    run_default(_market(server, tmp_path, crowns), timeout_s=300)


async def _market(server: Server, tmp_path: Path, crowns: bool) -> None:
    relay = Relay(server.url, server.http)
    relay_url = await relay.start()
    transcript = Transcript()
    grace: SeatSession = await seated(server, "grace")
    felix: SeatSession = await seated(server, "felix")
    try:
        async with adapter(
            relay_url, tmp_path / "state", server.grants()["visitor"], tmp_path / "market.log"
        ) as session:
            joined = await call(
                session, transcript, "join", {"seat": "visitor", "nickname": "Vera"}
            )
            handle = str(joined["handle"])
            view = await _view(session, transcript, handle)
            await _prices_and_display(session, transcript, handle, view, crowns)
            await _buys_from_both(session, transcript, handle, grace, crowns)
            await _seller_kinds(session, transcript, handle, felix)
            await _notices(session, transcript, handle, felix, crowns)
    finally:
        await grace.close()
        await felix.close()
        await relay.close()
    # AC-g-6: the surface, and every result under the cap.
    assert transcript.largest() <= RESULT_CAP, transcript.largest()


async def _prices_and_display(
    session: ClientSession, transcript: Transcript, handle: str, view: dict[str, Any], crowns: bool
) -> None:
    """AC-g-1: the currency once, every buy priced from its payload with its item's name, Grace's ask."""
    currency = view["currency"]
    wanted = ("crown", "cr", 0, "after") if crowns else ("US dollar", "$", 2, "before")
    shown = (
        currency["name"]["text"],
        currency["symbol"]["text"],
        currency["exponent"],
        currency["symbol_at"],
    )
    assert shown == wanted, currency
    buys = [
        o
        for o in await _every_offer(session, transcript, handle, view)
        if o["action_type"] == "buy"
    ]
    assert len(buys) >= 7, buys  # the café's six kinds and Grace's flowers
    for buy in buys:
        assert buy["price"]["display"] == _display(buy["price"]["minor"], crowns), buy
        assert buy["item_name"]["text"], buy
    grace = _person(view, "Grace")
    (ask,) = grace["asks"]
    assert (ask["name"]["text"], ask["price"]["minor"], ask["available"]) == ("Flowers", 450, True)
    assert ask["price"]["display"] == _display(450, crowns)
    hers = [b for b in buys if b["target"] == grace["ref"]]
    assert [(b["price"]["minor"], b["item_name"]["text"]) for b in hers] == [(450, "Flowers")]


async def _buys_from_both(
    session: ClientSession, transcript: Transcript, handle: str, grace: SeatSession, crowns: bool
) -> None:
    """AC-g-2 and AC-g-3: buy the café's coffee and Grace's flowers; nothing private of hers shows; a price she
    moves is never paid."""
    view = await _view(session, transcript, handle)
    every = await _every_offer(session, transcript, handle, view)
    grace_ref = _person(view, "Grace")["ref"]
    coffee = next(
        o for o in every if o["action_type"] == "buy" and o["item_name"]["text"] == "Coffee"
    )
    flowers = next(o for o in every if o["action_type"] == "buy" and o["target"] == grace_ref)
    wallet = view["me"]["wallet"]["minor"]
    for offer in (coffee, flowers):
        bought = await _act(
            session,
            transcript,
            handle,
            {"kind": "attempt", "view": view["view"], "offer": offer["n"]},
        )
        assert bought["outcome"]["kind"] == "accepted", bought["outcome"]
        (sale,) = _readings(bought, "sold")
        assert sale["price"]["minor"] == offer["price"]["minor"], sale
        assert sale["price"]["display"] == _display(offer["price"]["minor"], crowns)
    after = await _view(session, transcript, handle)
    assert (
        after["me"]["wallet"]["minor"]
        == wallet - coffee["price"]["minor"] - flowers["price"]["minor"]
    )
    held = {c["name"]["text"]: c["count"] for c in after["me"]["holdings"] if c.get("name")}
    assert held.get("Coffee") == 1 and held.get("Flowers") == 1, held

    inspected = await call(session, transcript, "inspect", {"handle": handle, "ref": grace_ref})
    components = {c["component"] for c in inspected["components"]}
    assert not components & {
        "wallet",
        "wealth",
        "holdings",
        "subscriptions",
        "survey-book",
        "proposals",
    }, components
    (asks,) = [c for c in inspected["components"] if c["component"] == "asks"]
    (line,) = asks["envelope"]["payload"]["asks"]
    assert set(line) == {"item", "price", "available"}, line

    # AC-g-3: Grace (a plain client) re-prices; the agent's view shows it; the offer it read before is stale.
    flower_id = _component(_me(cast(Any, grace.newest).observation), "asks")
    assert isinstance(flower_id, dict)
    item = cast(list[dict[str, Any]], flower_id["asks"])[0]["item"]
    repriced = await _plain_request(grace, "set-price", {"item": item, "price": 500})
    assert type(repriced).__name__ == "Accepted", repriced
    await call(
        session,
        transcript,
        "wait",
        {"handle": handle, "until": {"kind": "view_changed"}, "timeout_s": 20},
    )
    stale = await _act(
        session,
        transcript,
        handle,
        {"kind": "attempt", "view": view["view"], "offer": flowers["n"]},
    )
    assert stale["outcome"] == {"kind": "refused", "code": "stale_offer"}, stale["outcome"]
    fresh = stale["view"]
    assert _person(fresh, "Grace")["asks"][0]["price"]["minor"] == 500
    assert fresh["me"]["wallet"]["minor"] == after["me"]["wallet"]["minor"]


async def _seller_kinds(
    session: ClientSession, transcript: Transcript, handle: str, felix: SeatSession
) -> None:
    """AC-g-4: my own asks through the six kinds' seller three; a keeper's shop ask through a plain client."""
    view = await _view(session, transcript, handle)
    refs = {c["name"]["text"]: c["item"] for c in view["me"]["holdings"] if c.get("name")}
    apple, scarf = refs["Apple"], refs["Scarf"]
    listed = await _act(
        session, transcript, handle, {"kind": "list_for_sale", "item": apple, "price": 120}
    )
    assert listed["outcome"]["kind"] == "accepted", listed["outcome"]
    (reading,) = _readings(listed, "listed")
    assert (reading["kind"], reading["price"]["minor"], reading["was"]) == ("ask", 120, None)
    mine = (await _view(session, transcript, handle))["me"]["asks"]
    assert [(a["item"], a["price"]["minor"]) for a in mine] == [(apple, 120)]
    moved = await _act(
        session, transcript, handle, {"kind": "set_price", "item": apple, "price": 150}
    )
    (reading,) = _readings(moved, "price-set")
    assert (reading["price"]["minor"], reading["was"]["minor"]) == (150, 120)
    _rejected(
        await _act(session, transcript, handle, {"kind": "set_price", "item": apple, "price": 150}),
        "price-unchanged",
    )
    _rejected(
        await _act(
            session, transcript, handle, {"kind": "list_for_sale", "item": apple, "price": 99}
        ),
        "already-listed",
    )
    _rejected(
        await _act(
            session, transcript, handle, {"kind": "list_for_sale", "item": scarf, "price": 0}
        ),
        "price-out-of-range",
    )
    gone = await _act(session, transcript, handle, {"kind": "withdraw", "item": apple})
    assert gone["outcome"]["kind"] == "accepted", gone["outcome"]
    (reading,) = _readings(gone, "withdrawn")
    assert reading["price"] is None

    # A keeper standing in the store re-prices its bread: the server accepts it from Felix (Q-S31-8).
    store = _component(_place(cast(Any, felix.newest).observation), "item-catalogue")
    assert isinstance(store, dict)
    bread = next(
        k["item"] for k in cast(list[dict[str, Any]], store["kinds"]) if k["name"] == "Bread"
    )
    kept = await _plain_request(felix, "set-price", {"item": bread, "price": 300, "for": "shop"})
    assert type(kept).__name__ == "Accepted", kept


async def _notices(
    session: ClientSession, transcript: Transcript, handle: str, felix: SeatSession, crowns: bool
) -> None:
    """AC-g-5: publish, subscribe, receive a publisher's prices notice with names and display, unsubscribe."""
    published = await _act(
        session,
        transcript,
        handle,
        {
            "kind": "publish_notice",
            "topic": "news.daily",
            "valid_for_s": 7200,
            "body": {"kind": "text", "text": "Market opens at 8"},
        },
    )
    assert published["outcome"]["kind"] == "accepted", published["outcome"]
    view = await _view(session, transcript, handle)
    (notice,) = view["notices"]
    assert (notice["topic"], notice["publisher"], notice["text"]["text"]) == (
        "news.daily",
        view["me"]["ref"],
        "Market opens at 8",
    )
    assert notice["valid_until"] == notice["issued_at"] + 7200
    subscribed = await _act(
        session, transcript, handle, {"kind": "subscribe", "topic": "prices.ingredients"}
    )
    assert _readings(subscribed, "subscribed") == [{"kind": "topic", "topic": "prices.ingredients"}]
    assert "prices.ingredients" in (await _view(session, transcript, handle))["me"]["subscriptions"]

    catalogue = _component(_place(cast(Any, felix.newest).observation), "item-catalogue")
    assert isinstance(catalogue, dict)
    kinds = {k["name"]: k["item"]["entity"] for k in cast(list[dict[str, Any]], catalogue["kinds"])}
    lines = sorted([(kinds["Apple"], 60), (kinds["Bread"], 140)], key=lambda line: int(line[0]))
    body = {"prices": [{"item": item, "price": price, "basis": 3} for item, price in lines]}
    sent = await _plain_request(
        felix, "publish-notice", {"topic": "prices.ingredients", "body": body, "valid_for": 3600}
    )
    assert type(sent).__name__ == "Accepted", sent
    woke = await call(
        session,
        transcript,
        "wait",
        {"handle": handle, "until": {"kind": "any_event"}, "timeout_s": 20},
    )
    readings = [e["reading"] for e in woke["events"] if e["event_type"] == "notice-published"]
    (reading,) = readings
    assert [
        (p["name"]["text"], p["price"]["minor"], p["price"]["display"]) for p in reading["prices"]
    ] == sorted(
        [("Apple", 60, _display(60, crowns)), ("Bread", 140, _display(140, crowns))],
        key=lambda line: int(kinds[line[0]]),
    )
    board = (await _view(session, transcript, handle))["notices"]
    assert {n["topic"] for n in board} == {"news.daily", "prices.ingredients"}
    left = await _act(
        session, transcript, handle, {"kind": "unsubscribe", "topic": "prices.ingredients"}
    )
    assert left["outcome"]["kind"] == "accepted", left["outcome"]
    assert "prices.ingredients" not in (await _view(session, transcript, handle))["me"].get(
        "subscriptions", []
    )
    _rejected(
        await _act(session, transcript, handle, {"kind": "subscribe", "topic": "no.such"}),
        "unknown-topic",
    )


# ── W3 ────────────────────────────────────────────────────────────────────────────────────────────


def test_notices_without_economy_show_no_display(scratch: list[Server], tmp_path: Path) -> None:
    server = _serve(scratch, cafe_notices(tmp_path))
    run_default(_cafe(server, tmp_path), timeout_s=180)


async def _cafe(server: Server, tmp_path: Path) -> None:
    relay = Relay(server.url, server.http)
    relay_url = await relay.start()
    transcript = Transcript()
    wanderer: SeatSession = await seated(server, "wanderer")
    try:
        async with adapter(
            relay_url, tmp_path / "state", server.grants()["visitor"], tmp_path / "cafe.log"
        ) as session:
            joined = await call(
                session, transcript, "join", {"seat": "visitor", "nickname": "Vera"}
            )
            handle = str(joined["handle"])
            view = await _view(session, transcript, handle)
            assert "currency" not in view and view["me"].get("wallet") is None
            subscribed = await _act(
                session, transcript, handle, {"kind": "subscribe", "topic": "prices.ingredients"}
            )
            assert subscribed["outcome"]["kind"] == "accepted", subscribed["outcome"]
            # The wanderer publishes a prices line on a loose object of the café: an Item, with no kind name.
            loose = _component(_place(cast(Any, wanderer.newest).observation), "loose-objects")
            assert isinstance(loose, list) and loose
            thing = cast(dict[str, Any], loose[0])["object"]["entity"]
            body = {"prices": [{"item": thing, "price": 300, "basis": 1}]}
            sent = await _plain_request(
                wanderer,
                "publish-notice",
                {"topic": "prices.ingredients", "body": body, "valid_for": 3600},
            )
            assert type(sent).__name__ == "Accepted", sent
            woke = await call(
                session,
                transcript,
                "wait",
                {"handle": handle, "until": {"kind": "any_event"}, "timeout_s": 20},
            )
            (reading,) = [
                e["reading"] for e in woke["events"] if e["event_type"] == "notice-published"
            ]
            (line,) = reading["prices"]
            assert (line["price"]["minor"], line["price"]["display"]) == (300, None)
            published = await _act(
                session,
                transcript,
                handle,
                {
                    "kind": "publish_notice",
                    "topic": "news.daily",
                    "valid_for_s": 3600,
                    "body": {"kind": "text", "text": "Open late tonight"},
                },
            )
            assert published["outcome"]["kind"] == "accepted", published["outcome"]
            board = (await _view(session, transcript, handle))["notices"]
            assert {n["topic"] for n in board} == {"news.daily", "prices.ingredients"}
    finally:
        await wanderer.close()
        await relay.close()
    assert transcript.largest() <= RESULT_CAP, transcript.largest()
