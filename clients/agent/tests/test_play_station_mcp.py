"""An agent finds the station by its roles over MCP, in five worlds (S29-g §8, AC-sg-1 … AC-sg-5).

Each test starts the real `mineworld server` on one world and plays one seat through the MCP adapter over
stdio. Places are found by exploring the exits' `roles`, never by a key, so the generated world W4 (other keys,
a separate landmark, tags in another order) passes by the same steps as the reference towns. The oracles are
the server's answers and `inspect`'s verbatim components.

Worlds: W1 `market-town`, W2 `social-cafe`, W3 `lakeside`, W4 `crossing` (generated here), W5 a copy of W1
with a planted ticket shop at the ticket office (X-5's agent half).
"""

from __future__ import annotations

import shutil
from collections.abc import Callable, Coroutine
from pathlib import Path
from typing import Any

import pytest
from mcp import ClientSession
from mcp_relay import Relay
from realserver import REPOSITORY, Server
from support import run_default
from test_play_mcp import RESULT_CAP, Transcript, adapter, call, walk

pytestmark = pytest.mark.real_server

type Start = Callable[..., Server]
SEARCH_LIMIT = 12
"""The most places one search may visit before it fails by name."""
FORBIDDEN = ("service", "departure", "timetable", "closed")


def roles_here(view: dict[str, Any]) -> list[str]:
    place: dict[str, Any] = view["me"].get("place") or {}
    roles: list[str] = place.get("roles", [])
    return list(roles)


def buys(view: dict[str, Any]) -> list[dict[str, Any]]:
    return [offer for offer in view["offers"] if offer["action_type"] == "buy"]


async def observe(session: ClientSession, transcript: Transcript, handle: str) -> dict[str, Any]:
    view: dict[str, Any] = (await call(session, transcript, "observe", {"handle": handle}))["view"]
    return view


async def components(
    session: ClientSession, transcript: Transcript, handle: str, ref: str
) -> dict[str, Any]:
    """`inspect(ref)` as `{component: payload}`."""
    found = await call(session, transcript, "inspect", {"handle": handle, "ref": ref})
    return {c["component"]: c["envelope"]["payload"] for c in found["components"]}


async def explore(
    session: ClientSession, transcript: Transcript, handle: str, role: str
) -> list[str]:
    """Walks breadth-first through exits, each place once, until an exit whose roles contain `role` appears;
    walks into it and returns that exit's roles."""
    seen: set[str] = set()
    frontier: list[str] = []
    for _ in range(SEARCH_LIMIT):
        view = await observe(session, transcript, handle)
        here = view["me"]["place"]["ref"]
        seen.add(here)
        for line in view["here"]["exits"]:
            leads = line["leads_to"]
            if role in leads.get("roles", []):
                await walk(session, transcript, handle, {"place": leads["ref"]})
                return list(leads["roles"])
            if leads["ref"] not in seen and leads["ref"] not in frontier:
                frontier.append(leads["ref"])
        if not frontier:
            break
        await walk(session, transcript, handle, {"place": frontier.pop(0)})
    pytest.fail(f"no exit with the role {role!r} within {SEARCH_LIMIT} places", pytrace=False)


async def into(
    session: ClientSession, transcript: Transcript, handle: str, roles: set[str]
) -> None:
    """Walks through the exit of the current place whose roles include all of `roles`."""
    view = await observe(session, transcript, handle)
    for line in view["here"]["exits"]:
        if roles <= set(line["leads_to"].get("roles", [])):
            await walk(session, transcript, handle, {"place": line["leads_to"]["ref"]})
            return
    pytest.fail(f"no exit with the roles {sorted(roles)} here", pytrace=False)


async def shop_rule(session: ClientSession, transcript: Transcript, handle: str) -> None:
    """AC-sg-2's hall arm: the `buy` offers are exactly what the place's `shop` lists, or none without one."""
    view = await observe(session, transcript, handle)
    shown = await components(session, transcript, handle, view["me"]["place"]["ref"])
    shop = shown.get("shop")
    if shop is None:
        assert buys(view) == []
        return
    listed = shop.get("listed", [])
    assert len(buys(view)) == len(listed), (buys(view), listed)


def nothing_pretends(transcript: Transcript) -> None:
    """AC-sg-3: no `departures` record, no service word outside world text, every answer within the cap."""
    for text in transcript.texts:
        lowered = text.lower()
        assert '"departures"' not in lowered
        for word in FORBIDDEN:
            assert word not in lowered.replace("world>>>", "").split("<<<world")[0], word
    assert transcript.largest() <= RESULT_CAP, transcript.largest()


async def session_for(
    server: Server, seat: str, tmp_path: Path
) -> tuple[Relay, str, Path, Path, str]:
    relay = Relay(server.url, server.http)
    relay_url = await relay.start()
    logs = tmp_path / "logs"
    logs.mkdir(exist_ok=True)
    return relay, relay_url, tmp_path / "state", logs, server.grants()[seat]


async def play_station(
    server: Server, seat: str, tmp_path: Path, hall_roles: list[str], separate_landmark: bool
) -> None:
    relay, relay_url, state, logs, grant = await session_for(server, seat, tmp_path)
    transcript = Transcript()
    try:
        async with adapter(relay_url, state, grant, logs / "a.log") as session:
            joined = await call(session, transcript, "join", {"seat": seat, "nickname": "Ada"})
            handle = str(joined["handle"])
            # AC-sg-1: the hall by its roles, then the ticket office and the platform from it.
            assert await explore(session, transcript, handle, "station") == hall_roles
            assert roles_here(await observe(session, transcript, handle)) == hall_roles
            await shop_rule(session, transcript, handle)
            await into(session, transcript, handle, {"ticket-office"})
            view = await observe(session, transcript, handle)
            assert roles_here(view) == ["station", "ticket-office"]
            # AC-sg-2: the window sells nothing.
            shown = await components(session, transcript, handle, view["me"]["place"]["ref"])
            assert "shop" not in shown
            assert buys(view) == []
            await into(session, transcript, handle, {"station"})
            await into(session, transcript, handle, {"stop", "platform"})
            assert roles_here(await observe(session, transcript, handle)) == [
                "station",
                "stop",
                "platform",
            ]
            if separate_landmark:
                assert "landmark" in await explore(session, transcript, handle, "landmark")
                assert roles_here(await observe(session, transcript, handle)) == ["landmark"]
            nothing_pretends(transcript)
    finally:
        await relay.close()


def write_crossing(root: Path) -> Path:
    """W4: a generated town whose keys and tag order are its own."""
    world = root / "crossing"
    (world / "places").mkdir(parents=True)
    (world / "people").mkdir()
    (world / "world.yaml").write_text(
        "world:\n  id: crossing\n  name: The Crossing\n  version: 0.1.0\n  license: MIT\n"
        'mineworld: "^0.1"\nsystems: [presence, movement, naming]\n'
        "places: [booking, clocktower, depot, green, siding]\npopulation: [rook]\nseats: [rook]\n"
        "settlement:\n  kind: town\n  station: depot\n  landmark: clocktower\n",
        encoding="utf-8",
    )
    places = {
        "green": "tags: [outdoor, public]\n",
        "depot": "tags: [public, station]\npassages:\n  - to: green\n",
        "booking": "tags: [public, ticket-office, station]\npassages:\n  - to: depot\n",
        "siding": "tags: [outdoor, platform, public, stop, station]\npassages:\n  - to: depot\n",
        "clocktower": "tags: [public, landmark]\npassages:\n  - to: green\n",
    }
    for key, text in places.items():
        (world / "places" / f"{key}.yaml").write_text(text, encoding="utf-8")
    (world / "people" / "rook.yaml").write_text(
        "tags: [traveller]\nname: Rook\nlocation:\n  place: green\n", encoding="utf-8"
    )
    return world


def ticketed_copy(root: Path) -> Path:
    """W5: Market Town with a ticket shop planted at its ticket office (X-5)."""
    world = root / "market-town"
    shutil.copytree(REPOSITORY / "worlds" / "market-town", world)
    (world / "items" / "day-ticket.yaml").write_text(
        "tags: [ticket]\nitem: { category: goods, name: Day ticket }\n", encoding="utf-8"
    )
    (world / "organizations" / "rail-co.yaml").write_text(
        "economy: { wallet: 100000, shop: { at: station-tickets, prices: { day-ticket: 450 } } }\n"
        "holdings: { day-ticket: 20 }\n",
        encoding="utf-8",
    )
    manifest = world / "world.yaml"
    text = manifest.read_text(encoding="utf-8")
    text = text.replace("\nitems:\n", "\nitems:\n  - day-ticket\n", 1)
    text = text.replace("\norganizations:\n", "\norganizations:\n  - rail-co\n", 1)
    manifest.write_text(text, encoding="utf-8")
    return world


def run(test: Coroutine[Any, Any, None]) -> None:
    run_default(test, timeout_s=600)


def test_market_town(mineworld: Start, tmp_path: Path) -> None:
    server = mineworld("market-town", "--grant", "visitor", "--time-scale", "10")
    run(play_station(server, "visitor", tmp_path, ["station", "landmark"], separate_landmark=False))


def test_social_cafe(mineworld: Start, tmp_path: Path) -> None:
    server = mineworld("social-cafe", "--grant", "visitor", "--time-scale", "10")
    run(play_station(server, "visitor", tmp_path, ["station", "landmark"], separate_landmark=False))


def test_lakeside(mineworld: Start, tmp_path: Path) -> None:
    server = mineworld(
        "lakeside",
        "--grant",
        "alba",
        "--time-scale",
        "10",
        "--packs",
        str(REPOSITORY / "entities"),
        "--packs",
        str(REPOSITORY / "presentation" / "mineworld-default"),
    )
    run(play_station(server, "alba", tmp_path, ["station"], separate_landmark=True))


def test_crossing(mineworld: Start, tmp_path: Path) -> None:
    world = write_crossing(tmp_path / "worlds")
    server = mineworld(str(world), "--grant", "rook", "--time-scale", "10")
    run(play_station(server, "rook", tmp_path, ["station"], separate_landmark=True))


async def play_ticketed(server: Server, tmp_path: Path) -> None:
    relay, relay_url, state, logs, grant = await session_for(server, "visitor", tmp_path)
    transcript = Transcript()
    try:
        async with adapter(relay_url, state, grant, logs / "a.log") as session:
            joined = await call(session, transcript, "join", {"seat": "visitor", "nickname": "Ada"})
            handle = str(joined["handle"])
            await explore(session, transcript, handle, "station")
            await into(session, transcript, handle, {"ticket-office"})
            view = await observe(session, transcript, handle)
            shown = await components(session, transcript, handle, view["me"]["place"]["ref"])
            assert "shop" in shown, shown
            offers = buys(view)
            assert len(offers) == 1 and offers[0]["price"]["minor"] == 450, offers
            done = await call(
                session,
                transcript,
                "act",
                {
                    "handle": handle,
                    "act": {"kind": "attempt", "view": view["view"], "offer": offers[0]["n"]},
                },
            )
            assert done["outcome"]["kind"] == "accepted", done["outcome"]
            after = await observe(session, transcript, handle)
            assert any(count["count"] >= 1 for count in after["me"]["holdings"])
    finally:
        await relay.close()


def test_a_planted_ticket_shop_is_shown_exactly(mineworld: Start, tmp_path: Path) -> None:
    world = ticketed_copy(tmp_path / "worlds")
    server = mineworld(str(world), "--grant", "visitor", "--time-scale", "10")
    run(play_ticketed(server, tmp_path))
