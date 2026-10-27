"""An MCP agent walks Lakeside from the pier to the halt and back, like a person (S33-i §5.1, AC-i-1 … AC-i-3).

The real `mineworld server` hosts Lakeside; one seat is played through the MCP adapter over stdio, through the
recording relay, exactly as `test_play_station_mcp.py` plays a station. The script is deterministic and calls no
model. It holds no Lakeside place key (I-i-2): every place it walks to is chosen from what it observed, by the
role tags (`PlaceLine.roles`, S29-g) and the exits of its own place. The oracles are the server's answers: the
views it sends and `inspect`'s verbatim components.

The same script is the agent half of the 3D-observer pair (`tools/cli/tests/agent_with_3d_observer.rs`), which
runs this file as a program against a server it hosts:

    python clients/agent/tests/test_play_lakeside.py --server ws://… --http http://… --seat nils

with the seat's grant in `MINEWORLD_GRANT`. It prints one `LAKESIDE <json>` line per step and `LAKESIDE PASS` at
the end, and exits non-zero on any failed step.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import tempfile
from collections.abc import Callable
from pathlib import Path
from typing import Any

import pytest
from mcp import ClientSession
from mcp_relay import Relay
from realserver import REPOSITORY, Server, lakeside
from support import run_default
from test_play_mcp import RESULT_CAP, Transcript, adapter, call, walk
from test_play_station_mcp import components, observe, roles_here

from mineworld_agent import ROLE_TAGS

pytestmark = pytest.mark.real_server

SEAT = "nils"
"""The seat the script plays: Lakeside's fisherman, who begins on the pier (step-33 §7)."""
SEARCH_LIMIT = 8
"""The most places the search for the station may pass through before it fails by name."""

type Say = Callable[[str, dict[str, Any]], None]
"""Reports one step of the script (a no-op under pytest; a `LAKESIDE` line as a program)."""


def quiet(_step: str, _facts: dict[str, Any]) -> None:
    return None


def exits_with(view: dict[str, Any], role: str) -> list[dict[str, Any]]:
    """The exits of this view whose destination carries `role` among its role tags."""
    return [line for line in view["here"]["exits"] if role in line["leads_to"].get("roles", [])]


def no_service(view: dict[str, Any]) -> None:
    """AC-i-1's window step, and M-i-1's oracle: a ticket window in a world without an economy offers nothing.

    The seat's own acts (`move`, `walk-to`, `put-down`, `eat`, `drink` of what it carries) are offered in every
    place and have no target; they are not the window's (F-S33i-5). A service is any other offer: one with a
    target here, or any `buy`. Each is named in the failure (S29-g: "the adapter must not invent a service
    state")."""
    offered = [
        (offer["n"], offer["action_type"], offer.get("target"))
        for offer in view["offers"]
        if offer.get("target") is not None or offer["action_type"] == "buy"
    ]
    assert offered == [], (
        f"the ticket window offers {offered}; a world without an economy offers no service"
    )


async def walk_into(
    session: ClientSession, transcript: Transcript, handle: str, line: dict[str, Any]
) -> dict[str, Any]:
    """Walks through one exit of the current place, waited to its end; returns the view there."""
    target = str(line["leads_to"]["ref"])
    await walk(session, transcript, handle, {"place": target})
    view = await observe(session, transcript, handle)
    assert view["me"]["place"]["ref"] == target, (view["me"]["place"], target)
    return view


async def to_the_station(
    session: ClientSession, transcript: Transcript, handle: str, route: list[str], say: Say
) -> dict[str, Any]:
    """From here, through exits, until an exit's destination is the station; walks in and returns the view.
    Each place is entered once; a place with an exit to the station is left through it at once."""
    for _ in range(SEARCH_LIMIT):
        view = await observe(session, transcript, handle)
        found = exits_with(view, "station")
        if found:
            view = await walk_into(session, transcript, handle, found[0])
            route.append(view["me"]["place"]["ref"])
            say("station", {"place": route[-1], "roles": roles_here(view)})
            return view
        unseen = [line for line in view["here"]["exits"] if line["leads_to"]["ref"] not in route]
        assert unseen, f"no way on from {route[-1]}: every exit leads back"
        view = await walk_into(session, transcript, handle, unseen[0])
        route.append(view["me"]["place"]["ref"])
        say("walked", {"place": route[-1], "roles": roles_here(view)})
    pytest.fail(f"no exit to a station within {SEARCH_LIMIT} places", pytrace=False)


async def play_lakeside(session: ClientSession, transcript: Transcript, say: Say) -> list[str]:
    """§5.1's script on one MCP session; returns the places walked through, there and back, by reference."""
    joined = await call(session, transcript, "join", {"seat": SEAT, "nickname": "Nils's agent"})
    handle = str(joined["handle"])
    # On the pier: the landmark, and (AC-i-2, the adversarial check) no exit straight to the station.
    view = await observe(session, transcript, handle)
    start = str(view["me"]["place"]["ref"])
    assert "landmark" in roles_here(view), roles_here(view)
    assert exits_with(view, "station") == [], "the halt is not next to the landmark"
    route = [start]
    say("landmark", {"place": start, "roles": roles_here(view)})
    # To the station, found from disclosure only.
    view = await to_the_station(session, transcript, handle, route, say)
    assert "station" in roles_here(view), roles_here(view)
    # The ticket window: no economy, so no service (AC-i-1).
    windows = exits_with(view, "ticket-office")
    assert windows, "the station shows no exit to its ticket office"
    view = await walk_into(session, transcript, handle, windows[0])
    route.append(view["me"]["place"]["ref"])
    assert "ticket-office" in roles_here(view), roles_here(view)
    no_service(view)
    shown = await components(session, transcript, handle, route[-1])
    assert "shop" not in shown, shown["shop"]
    say("window", {"place": route[-1], "roles": roles_here(view), "offers": len(view["offers"])})
    # And back the way it came, one exit at a time.
    for back in reversed(route[:-1]):
        exits = [line for line in view["here"]["exits"] if line["leads_to"]["ref"] == back]
        assert exits, f"no exit back to {back}"
        view = await walk_into(session, transcript, handle, exits[0])
        say("back", {"place": back, "roles": roles_here(view)})
    assert view["me"]["place"]["ref"] == start
    assert "landmark" in roles_here(view)
    await call(session, transcript, "leave", {"handle": handle})
    assert transcript.largest() <= RESULT_CAP, transcript.largest()
    return route


async def run_once(
    upstream_ws: str, upstream_http: str, grant: str, scratch: Path, say: Say
) -> list[str]:
    """One adapter process, through the recording relay, plays the script once."""
    relay = Relay(upstream_ws, upstream_http)
    relay_url = await relay.start()
    logs = scratch / "logs"
    logs.mkdir(parents=True, exist_ok=True)
    transcript = Transcript()
    try:
        async with adapter(relay_url, scratch / "state", grant, logs / "nils.log") as session:
            return await play_lakeside(session, transcript, say)
    finally:
        await relay.close()


def test_an_agent_walks_from_the_pier_to_the_halt_and_back(
    mineworld: Callable[..., Server], tmp_path: Path
) -> None:
    """AC-i-1 and AC-i-2 on the real server: landmark, station, an empty window, back on the pier."""
    server = mineworld("lakeside", *lakeside("--grant", SEAT, "--time-scale", "10"))
    route = run_default(
        run_once(server.url, server.http, server.grants()[SEAT], tmp_path, quiet), timeout_s=600
    )
    # pier, lane, station, the ticket office: four distinct places, the last three found by their roles.
    assert len(route) == 4 and len(set(route)) == 4, route
    # The server agrees: nobody holds the seat any more, and the world kept running.
    assert server.status() is not None


def test_a_planted_offer_fails_the_window_step() -> None:
    """AC-i-3 (M-i-1): an offer planted in the ticket window's observation fails the step, naming the offer."""
    own = [
        {"n": 1, "action_type": "walk-to", "target": None},
        {"n": 2, "action_type": "eat", "target": None},
    ]
    no_service({"offers": own})
    planted = {"offers": [*own, {"n": 3, "action_type": "buy", "target": "P9"}]}
    with pytest.raises(AssertionError, match=r"offers \[\(3, 'buy', 'P9'\)\]"):
        no_service(planted)
    untargeted_buy = {"offers": [*own, {"n": 3, "action_type": "buy", "target": None}]}
    with pytest.raises(AssertionError, match=r"offers \[\(3, 'buy', None\)\]"):
        no_service(untargeted_buy)


def test_the_script_holds_no_place_key() -> None:
    """I-i-2: no Lakeside place key is a string literal of this file. A key that is also a role tag
    (`station`) is the framework's vocabulary (ARC-105), read from `roles`, and is exempt."""
    source = Path(__file__).read_text(encoding="utf-8")
    keys = sorted(
        path.stem for path in (REPOSITORY / "worlds" / "lakeside" / "places").glob("*.yaml")
    )
    assert keys, "Lakeside's places are found"
    held = [
        key
        for key in keys
        if key not in ROLE_TAGS and (f'"{key}"' in source or f"'{key}'" in source)
    ]
    assert held == [], held


def main(arguments: list[str]) -> int:
    """The agent half of the 3D-observer pair: the script against a running server, reported line by line."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--server", required=True, help="the server's WebSocket URL")
    parser.add_argument("--http", required=True, help="the server's HTTP base URL")
    options = parser.parse_args(arguments)
    grant = os.environ.get("MINEWORLD_GRANT", "")
    if not grant:
        print("LAKESIDE FAIL: no grant in MINEWORLD_GRANT", flush=True)
        return 2

    def say(step: str, facts: dict[str, Any]) -> None:
        print(f"LAKESIDE {json.dumps({'step': step, **facts}, sort_keys=True)}", flush=True)

    # A file the adapter process still holds on Windows must not turn a passed walk into a crash on cleanup.
    with tempfile.TemporaryDirectory(
        prefix="lakeside-agent-", ignore_cleanup_errors=True
    ) as scratch:
        try:
            run_default(
                run_once(options.server, options.http, grant, Path(scratch), say), timeout_s=600
            )
        except (AssertionError, pytest.fail.Exception) as failure:
            print(f"LAKESIDE FAIL: {failure}", flush=True)
            return 1
    print("LAKESIDE PASS", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
