"""SCRATCH: test_a_handle_without_calls_gives_its_seat_back with frames, SDK calls and admin state traced."""
from __future__ import annotations

import asyncio
import json
import time
import urllib.request

from conftest import Start
from mineworld_sdk import session as sdk_session
from mineworld_sdk import resuming as sdk_resuming
from mineworld_sdk.wire.frames import Invite
from realserver import ADMIN_CANARY, INVITE, PATIENCE, Wire, market_town, seated
from support import run

from mineworld_agent import JoinArgs, Player, PlayerConfig


def _admin(server, path: str) -> str:
    request = urllib.request.Request(f"{server.http}{path}", headers={"Authorization": f"Bearer {ADMIN_CANARY}"})
    try:
        with urllib.request.urlopen(request, timeout=10) as answer:
            return answer.read().decode()
    except Exception as error:  # noqa: BLE001
        return f"admin {path} failed: {error!r}"


def test_release(mineworld: Start, monkeypatch) -> None:
    t0 = time.monotonic()
    marks: list[str] = []

    def mark(what: str) -> None:
        marks.append(f"{time.monotonic() - t0:7.3f} {what}")

    real_leave = sdk_session.SeatSession.leave
    real_close = sdk_session.SeatSession.close
    real_stop = sdk_resuming.ResumingSeat._stop

    async def leave(self):
        mark(f"SeatSession.leave ended={self._ended!r}")
        try:
            reason = await real_leave(self)
            mark(f"SeatSession.leave -> {reason!r}")
            return reason
        except BaseException as error:
            mark(f"SeatSession.leave raised {error!r}")
            raise

    async def close(self):
        mark(f"SeatSession.close ended={self._ended!r}")
        await real_close(self)

    async def stop(self, end, *, give_back):
        mark(f"ResumingSeat._stop give_back={give_back} end={self._end!r} session={self._session!r} candidate={self._candidate!r}")
        await real_stop(self, end, give_back=give_back)

    monkeypatch.setattr(sdk_session.SeatSession, "leave", leave)
    monkeypatch.setattr(sdk_session.SeatSession, "close", close)
    monkeypatch.setattr(sdk_resuming.ResumingSeat, "_stop", stop)

    server = mineworld("market-town", *market_town("--pace", "600"))
    wire = Wire(server)
    failed = False

    async def play() -> None:
        nonlocal failed
        config = PlayerConfig(server=server.url, idle_release_s=2)
        async with Player(config, Invite(INVITE), connector=wire) as player:
            joined = await player.join(JoinArgs(seat="wanderer", nickname="away"))
            mark(f"joined {joined.took_over}")
            async with asyncio.timeout(PATIENCE):
                while (c := server.status().clients) != 0:
                    await asyncio.sleep(0.2)
            mark("clients 0")
            seats = _admin(server, "/admin/seats")
            try:
                again = await seated(server, "wanderer")
            except Exception as e:
                failed = True
                mark(f"join refused {e!r}")
                mark(f"seats at clients 0: {seats}")
                mark(f"seats now: {_admin(server, '/admin/seats')}")
                mark(f"sessions now: {_admin(server, '/admin/sessions')}")
                raise
            await again.leave()

    try:
        run(play(), timeout_s=60)
    finally:
        if failed:
            for tap in wire.taps:
                print(f"\ntap opened {tap.opened - t0:.3f}")
                for ts, f in tap.sent:
                    print(f"  sent {ts - t0:.3f} {f.get('t')}")
                kinds = [json.loads(m).get("t") for m in tap.received]
                print("  received", len(kinds), kinds[-8:], [m[:300] for m in tap.received if '"t":"refused"' in m[:20] or '"t":"closing"' in m[:20]])
            print("\n".join(marks))
            print("server stderr:", "".join(server.stderr)[-3000:])
        else:
            print("\nPASS-TRACE", "\n".join(marks))
            print("PASS-STDERR", "".join(line for line in server.stderr if "[scratch]" in line)[-2000:])
