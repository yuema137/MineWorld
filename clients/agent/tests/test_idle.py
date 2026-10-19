"""The world never waits (INV-S27-6), and an idle handle gives its seat back (step-27 §5.9, QS27-3).

AC-c-7, as ruled (QS27c-2, "换成新标准"): a handle left idle for 30 wall seconds at `--time-scale 12` in the
café — the world clock advanced by 360 ± 24 world seconds; the server's `events_dropped` stayed 0 and no
`lagged` closing reached the Player's socket; the next `observe` shows the newest observation (its `at`
within 2 world seconds of `/status.at`); the inbox dropped nothing it did not count. The oracles are
`/status` and the frames on the Player's own socket.

Idle release: with `idle_release_s=2`, a handle with no call gives its seat back by itself — a plain client
then finds the seat `hosted` again — and the next call answers `seat_lost { idle_released }`, once.
"""

from __future__ import annotations

import asyncio
import time
from pathlib import Path

import pytest
from conftest import Start
from mineworld_sdk.wire.frames import Invite, WorldSummary
from realserver import INVITE, PATIENCE, Server, Wire, market_town, seated
from support import run

from mineworld_agent import (
    JoinArgs,
    ObserveArgs,
    Player,
    PlayerConfig,
    PlayerRefusal,
    SeatLostRefusal,
    View,
)

pytestmark = pytest.mark.real_server

IDLE_S = 30
SCALE = 12


async def _status(server: Server) -> WorldSummary:
    """`/status`, read off the event loop. `Server.status` is a blocking HTTP call: on the loop it stops
    the Player reading its socket for as long as the server takes to answer, and the next `observe` then
    shows the observation from before the call — the oracle's own stall, not the Player's (pr-s27c D-c-23)."""
    return await asyncio.to_thread(server.status)


def test_an_idle_handle_holds_up_nothing(mineworld: Start, tmp_path: Path) -> None:
    server = mineworld(
        "market-town",
        *market_town("--save", str(tmp_path / "save"), "--time-scale", str(SCALE)),
    )
    wire = Wire(server)

    async def play() -> tuple[int, int, View, int]:
        async with Player(PlayerConfig(server=server.url), Invite(INVITE), connector=wire) as p:
            joined = await p.join(JoinArgs(seat="visitor", nickname="idle"))
            before = (await _status(server)).at
            began = time.monotonic()
            await asyncio.sleep(IDLE_S)  # the agent thinks: no call at all for 30 wall seconds
            status = await _status(server)
            advanced = status.at - before
            elapsed = time.monotonic() - began
            seen = (await p.observe(ObserveArgs(handle=joined.handle))).view
            assert isinstance(seen, View)
            print(f"AC-c-7: {elapsed:.1f} wall s idle, {advanced} world s, view at {seen.at}")
            return advanced, status.events_dropped, seen, (await _status(server)).at

    advanced, events_dropped, seen, now = run(play(), timeout_s=80)
    assert abs(advanced - IDLE_S * SCALE) <= 24, advanced
    assert events_dropped == 0
    assert not any('"lagged"' in frame for tap in wire.taps for frame in tap.received)
    assert len(wire.taps) == 1, "the connection was never closed for falling behind"
    assert now - seen.at <= 2, (now, seen.at)
    assert seen.inbox.dropped == 0 and seen.inbox.unread > 0


def test_a_handle_without_calls_gives_its_seat_back(mineworld: Start) -> None:
    server = mineworld("market-town", *market_town("--cadence", "600"))

    async def play() -> None:
        config = PlayerConfig(server=server.url, idle_release_s=2)
        async with Player(config, Invite(INVITE)) as player:
            joined = await player.join(JoinArgs(seat="wanderer", nickname="away"))
            assert joined.took_over == "hosted"
            async with asyncio.timeout(PATIENCE):
                while server.status().clients != 0:
                    await asyncio.sleep(0.2)
            again = await seated(server, "wanderer")
            assert again.welcome.took_over == "hosted", "the seat went back to its controller"
            await again.leave()
            with pytest.raises(SeatLostRefusal) as lost:
                await player.observe(ObserveArgs(handle=joined.handle))
            assert lost.value.reason == "idle_released"
            with pytest.raises(PlayerRefusal) as gone:
                await player.observe(ObserveArgs(handle=joined.handle))
            assert gone.value.code == "unknown_handle"

    run(play(), timeout_s=60)
