"""The HTTP app and its gate (pr-s27e §§5.4, 5.5, C3), served in this process by uvicorn on loopback.

`security_settings` builds the `Host` and `Origin` rules; `BearerGate` runs the SDK's transport-security check first
(a bad `Host` 421, a foreign `Origin` 403, both before any bearer is read), then the bearer's form (401 no sooner than
500 ms after arrival), then capacity (503 with `Retry-After`), then the SDK app. Each bearer is answered by its own
`Player` (here a double). The real-binary checks of the same rules are `test_http_endpoint.py`."""

from __future__ import annotations

import asyncio
import socket
import time
from collections.abc import AsyncGenerator, Callable
from contextlib import asynccontextmanager
from dataclasses import dataclass
from typing import Any, cast

import httpx2
import uvicorn
from mcp import Client
from mcp.client.streamable_http import streamable_http_client
from mineworld_sdk.wire.frames import Invite
from starlette.datastructures import Headers
from support import run

from mineworld_agent.mcp_server import Adapter, bearer_of, http_app, security_settings, tools
from mineworld_agent.models import LeaveArgs, LeaveResult
from mineworld_agent.player import Player, PlayerConfig
from mineworld_agent.pool import Credentials

CONFIG = PlayerConfig(server="ws://127.0.0.1:7878/ws")
HANDLE = "0123456789abcdef0123456789abcdef"
ONE = "bearer-one-0001"
TWO = "bearer-two-0002"
THREE = "bearer-three-0003"
LIST = b'{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
MCP_HEADERS = {"Content-Type": "application/json", "Accept": "application/json, text/event-stream"}


class Recorder:
    """A `Player` double: it records the handles it was asked to leave; `playing` is what `active()` says."""

    def __init__(self, credential: Invite) -> None:
        self.credential = credential
        self.left: list[str] = []
        self.playing = 0

    async def leave(self, args: LeaveArgs) -> LeaveResult:
        self.left.append(args.handle)
        return LeaveResult()

    def active(self) -> int:
        return self.playing

    async def close(self) -> None:
        return None


@dataclass
class Served:
    port: int
    made: list[Recorder]

    @property
    def url(self) -> str:
        return f"http://127.0.0.1:{self.port}/mcp"

    def made_for(self, bearer: str) -> Recorder:
        return next(player for player in self.made if player.credential == Invite(bearer))


@asynccontextmanager
async def serving(*, limit: int = 4, origins: tuple[str, ...] = ()) -> AsyncGenerator[Served]:
    sock = socket.create_server(("127.0.0.1", 0))
    sock.setblocking(False)
    port = sock.getsockname()[1]
    made: list[Recorder] = []

    def make(_config: PlayerConfig, credential: Invite, _on_inbox: Callable[[str], None]) -> Player:
        player = Recorder(credential)
        made.append(player)
        return cast(Player, player)

    adapter = Adapter()
    credentials = Credentials(CONFIG, limit=limit, on_inbox=adapter.on_inbox, make=make)
    settings = security_settings(("127.0.0.1", port), [], list(origins))
    app = http_app(adapter, credentials, CONFIG, settings)
    server = uvicorn.Server(
        uvicorn.Config(app, log_config=None, access_log=False, log_level="warning")
    )
    task = asyncio.create_task(server.serve(sockets=[sock]))
    while not server.started:
        await asyncio.sleep(0.01)
    try:
        yield Served(port, made)
    finally:
        server.should_exit = True
        await task
        sock.close()


async def post(served: Served, headers: dict[str, str]) -> tuple[httpx2.Response, float]:
    """One raw `tools/list` POST; the response and the seconds from send to answer."""
    async with httpx2.AsyncClient() as http:
        start = time.monotonic()
        answer = await http.post(served.url, content=LIST, headers={**MCP_HEADERS, **headers})
        return answer, time.monotonic() - start


@asynccontextmanager
async def agent(served: Served, bearer: str) -> AsyncGenerator[Client]:
    async with httpx2.AsyncClient(headers={"Authorization": f"Bearer {bearer}"}) as http:
        async with Client(streamable_http_client(served.url, http_client=http)) as client:
            yield client


# ── security_settings (§5.5) ────────────────────────────────────────────────────────────────────────────────


def test_loopback_settings_name_the_three_loopback_hosts_with_the_port() -> None:
    settings = security_settings(("127.0.0.1", 4100), [], [])
    assert settings.enable_dns_rebinding_protection is True
    assert settings.allowed_hosts == ["127.0.0.1:4100", "localhost:4100", "[::1]:4100"]
    assert settings.allowed_origins == []


def test_remote_settings_add_the_bound_address_and_each_allowed_host() -> None:
    settings = security_settings(
        ("0.0.0.0", 4100), ["agents.example", "gw.example:443", "[fd00::1]"], ["https://ui.example"]
    )
    assert settings.allowed_hosts == [
        "127.0.0.1:4100",
        "localhost:4100",
        "[::1]:4100",
        "0.0.0.0:4100",
        "agents.example:4100",
        "gw.example:443",
        "[fd00::1]:4100",
    ]
    assert settings.allowed_origins == ["https://ui.example"]
    assert not any("*" in host for host in settings.allowed_hosts)


def test_an_ipv6_bind_is_bracketed() -> None:
    settings = security_settings(("::1", 4100), [], [])
    assert settings.allowed_hosts == ["127.0.0.1:4100", "localhost:4100", "[::1]:4100"]


def test_the_bearer_form() -> None:
    def of(*values: str) -> Invite | None:
        return bearer_of(Headers(raw=[(b"authorization", value.encode()) for value in values]))

    assert of("Bearer 12345678") == Invite("12345678")
    assert of("bearer 12345678") == Invite("12345678")
    assert of("Bearer " + "x" * 128) == Invite("x" * 128)
    for bad in ("Bearer 1234567", "Bearer " + "x" * 129, "Bearer 1234 5678", "Basic 12345678", ""):
        assert of(bad) is None, bad
    assert of("Bearer 12345678", "Bearer 12345678") is None
    assert of() is None


# ── The gate (§5.4) ─────────────────────────────────────────────────────────────────────────────────────────


def test_a_missing_or_malformed_bearer_is_401_after_half_a_second() -> None:
    async def scenario() -> list[tuple[int, float, str | None]]:
        found: list[tuple[int, float, str | None]] = []
        async with serving() as served:
            for headers in (
                {},
                {"Authorization": "Basic dXNlcjpwYXNz"},
                {"Authorization": "Bearer 1234567"},
                {"Authorization": "Bearer " + "x" * 129},
                {"Authorization": "Bearer 1234 5678"},
            ):
                answer, elapsed = await post(served, headers)
                found.append((answer.status_code, elapsed, answer.headers.get("www-authenticate")))
            assert served.made == []
        return found

    for status, elapsed, challenge in run(scenario(), timeout_s=60):
        assert status == 401
        assert challenge == "Bearer"
        assert 0.5 <= elapsed < 2.0, elapsed


def test_a_foreign_origin_is_403_before_the_bearer_is_read() -> None:
    async def scenario() -> tuple[int, float]:
        async with serving() as served:
            answer, elapsed = await post(served, {"Origin": "http://evil.example"})
        return answer.status_code, elapsed

    status, elapsed = run(scenario(), timeout_s=30)
    assert status == 403
    assert elapsed < 0.5, "a header refusal waited for the bearer's delay"


def test_a_rebinding_host_without_an_origin_is_421() -> None:
    """DNS rebinding: the page's own name in `Host`, and no `Origin` (a same-origin request sends none). The
    `Host` check alone stops it (F-S27e-4; mutation M-e-1 turns this red)."""

    async def scenario() -> list[int]:
        async with serving() as served:
            host = f"rebind.example:{served.port}"
            bare, _ = await post(served, {"Host": host, "Authorization": f"Bearer {ONE}"})
            with_origin, _ = await post(
                served,
                {"Host": host, "Origin": f"http://{host}", "Authorization": f"Bearer {ONE}"},
            )
            assert served.made == [], "a refused request created a Player"
        return [bare.status_code, with_origin.status_code]

    assert run(scenario(), timeout_s=30) == [421, 421]


def test_an_allowed_origin_passes_and_its_neighbour_does_not() -> None:
    async def scenario() -> list[int]:
        async with serving(origins=("http://127.0.0.1:5173",)) as served:
            bearer = {"Authorization": f"Bearer {ONE}"}
            good, _ = await post(served, {**bearer, "Origin": "http://127.0.0.1:5173"})
            bad, _ = await post(served, {**bearer, "Origin": "http://127.0.0.1:5174"})
        return [good.status_code, bad.status_code]

    assert run(scenario(), timeout_s=30) == [200, 403]


def test_a_full_pool_is_503_with_retry_after() -> None:
    async def scenario() -> list[tuple[int, str | None]]:
        found: list[tuple[int, str | None]] = []
        async with serving(limit=1) as served:
            first, _ = await post(served, {"Authorization": f"Bearer {ONE}"})
            served.made_for(ONE).playing = 1
            second, elapsed = await post(served, {"Authorization": f"Bearer {TWO}"})
            assert elapsed < 0.5
            served.made_for(ONE).playing = 0
            third, _ = await post(served, {"Authorization": f"Bearer {TWO}"})
            for answer in (first, second, third):
                found.append((answer.status_code, answer.headers.get("retry-after")))
        return found

    assert run(scenario(), timeout_s=30) == [(200, None), (503, "5"), (200, None)]


def test_each_request_gets_its_own_player() -> None:
    """Two agents through one app: each bearer's calls reach that bearer's `Player` only (handle binding, I-e-2)."""

    async def scenario() -> Served:
        async with serving() as served:
            async with agent(served, ONE) as one, agent(served, TWO) as two:
                for client in (one, two, two, one, two):
                    answer = await client.call_tool("leave", {"handle": HANDLE})
                    assert answer.is_error is not True
            return served

    served = run(scenario(), timeout_s=60)
    assert len(served.made_for(ONE).left) == 2
    assert len(served.made_for(TWO).left) == 3


def test_the_tool_list_over_http_is_the_published_one() -> None:
    """I-e-1: the same tools, schemas and descriptions as stdio (`tools()`), whatever their number."""

    async def scenario() -> list[dict[str, Any]]:
        async with serving() as served, agent(served, THREE) as client:
            listed = await client.list_tools()
        return [
            tool.model_dump(mode="json", by_alias=True, exclude_none=True) for tool in listed.tools
        ]

    published = [tool.model_dump(mode="json", by_alias=True, exclude_none=True) for tool in tools()]
    assert run(scenario(), timeout_s=60) == published
