"""AC-d-9 (AC-S27-4): the adapter's connection is indistinguishable from a plain client's to the server's own accounts.

While the adapter holds `visitor`, `/status.clients` counts it as one client; a plain client adds one more. The admin
surface lists every seated connection (`GET /admin/sessions`, with the canary admin token from the test): the
adapter's session and a plain client's session have the same set of fields."""

from __future__ import annotations

import asyncio
import json
import urllib.request
from pathlib import Path
from typing import Any, cast

from mcp_relay import Relay
from realserver import ADMIN_CANARY, Server, seated
from support import run_default
from test_play_mcp import Transcript, adapter, call


def admin_sessions(server: Server) -> list[dict[str, Any]]:
    """`GET /admin/sessions` with the canary admin token, as the operator's tooling would."""
    request = urllib.request.Request(
        f"{server.http}/admin/sessions", headers={"Authorization": f"Bearer {ADMIN_CANARY}"}
    )
    with urllib.request.urlopen(request, timeout=10) as answer:
        body: Any = json.loads(answer.read())
    assert isinstance(body, dict), body
    return cast(dict[str, list[dict[str, Any]]], body)["sessions"]


def session_named(sessions: list[dict[str, Any]], nickname: str) -> dict[str, Any]:
    """The one session whose nickname is `nickname` (the adapter's and the plain client's are distinct)."""
    matches = [entry for entry in sessions if entry.get("nickname") == nickname]
    assert len(matches) == 1, (nickname, sessions)
    return matches[0]


def test_the_adapter_is_one_client_with_the_same_session_fields_as_a_plain_client(
    town: Server, tmp_path: Path
) -> None:
    run_default(_admin_view(town, tmp_path), timeout_s=120)


async def _admin_view(server: Server, tmp_path: Path) -> None:
    relay = Relay(server.url, server.http)
    relay_url = await relay.start()
    transcript = Transcript()
    plain = None
    try:
        async with adapter(
            relay_url, tmp_path / "state", server.grants()["visitor"], tmp_path / "admin.log"
        ) as session:
            await call(session, transcript, "join", {"seat": "visitor", "nickname": "Ada"})
            assert (await asyncio.to_thread(server.status)).clients == 1
            plain = await seated(server, "bob")
            assert (await asyncio.to_thread(server.status)).clients == 2
            sessions = await asyncio.to_thread(admin_sessions, server)
            adapter_session = session_named(sessions, "Ada")
            plain_session = session_named(sessions, "bob-sdk")
            assert set(adapter_session) == set(plain_session), sorted(
                set(adapter_session) ^ set(plain_session)
            )
            assert {type(value).__name__ for value in adapter_session.values()} <= {
                type(value).__name__ for value in plain_session.values()
            } | {"NoneType"}
    finally:
        if plain is not None:
            await plain.close()
        await relay.close()
