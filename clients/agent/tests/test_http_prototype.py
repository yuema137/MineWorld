"""Prototype checks of the SDK's Streamable HTTP transport, kept as regression tests (pr-s27e §3.2, C1).

V-1 (P-e-1): a low-level `on_call_tool` handler served over stateless Streamable HTTP sees the request's headers
through `ctx.request.headers` (the Starlette `Request` the transport attaches), for a handshake-era client and a
2026-07-28 client. V-2 (P-e-2): `streamable_http_client` with a configured `httpx2` client sends the same
`Authorization` header on every request. V-3 (P-e-3): `subscriptions/listen` streams a cue over stateless HTTP with
SSE responses. V-4 (P-e-4): uvicorn serves a socket bound beforehand on port 0 — every test here is served that
way, so the port the client reaches is the socket's.

Every socket here is on loopback.
"""

from __future__ import annotations

import asyncio
import socket
from collections.abc import AsyncGenerator
from contextlib import asynccontextmanager
from typing import Any, Literal

import httpx2
import uvicorn
from mcp import Client, types
from mcp.client.streamable_http import streamable_http_client
from mcp.server.context import ServerRequestContext
from mcp.server.lowlevel import Server
from mcp.server.subscriptions import InMemorySubscriptionBus, ListenHandler
from mcp.server.transport_security import TransportSecuritySettings
from mcp.shared.subscriptions import ResourceUpdated
from support import run

URI = "mineworld://seat/00000000000000000000000000000000/inbox"


def _probe_server(seen: list[str | None], bus: InMemorySubscriptionBus) -> Server[Any]:
    async def list_tools(
        _ctx: ServerRequestContext[Any], _params: types.PaginatedRequestParams | None
    ) -> types.ListToolsResult:
        return types.ListToolsResult(
            tools=[types.Tool(name="probe", input_schema={"type": "object"})]
        )

    async def call_tool(
        ctx: ServerRequestContext[Any], _params: types.CallToolRequestParams
    ) -> types.CallToolResult:
        request = ctx.request
        headers = getattr(request, "headers", None)
        value = None if headers is None else headers.get("authorization")
        seen.append(value)
        return types.CallToolResult(
            content=[types.TextContent(type="text", text=str(len(value or "")))]
        )

    return Server(
        "probe",
        on_list_tools=list_tools,
        on_call_tool=call_tool,
        on_subscriptions_listen=ListenHandler(bus),
    )


def _bound() -> socket.socket:
    sock = socket.create_server(("127.0.0.1", 0))
    sock.setblocking(False)
    return sock


@asynccontextmanager
async def _serving(server: Server[Any]) -> AsyncGenerator[str]:
    sock = _bound()
    port = sock.getsockname()[1]
    settings = TransportSecuritySettings(
        enable_dns_rebinding_protection=True,
        allowed_hosts=[f"127.0.0.1:{port}"],
        allowed_origins=[],
    )
    app = server.streamable_http_app(stateless_http=True, transport_security=settings)
    config = uvicorn.Config(app, log_config=None, access_log=False, log_level="warning")
    http = uvicorn.Server(config)
    task = asyncio.create_task(http.serve(sockets=[sock]))
    while not http.started:
        await asyncio.sleep(0.01)
    try:
        yield f"http://127.0.0.1:{port}/mcp"
    finally:
        http.should_exit = True
        await task
        sock.close()


@asynccontextmanager
async def _client(url: str, bearer: str, mode: Literal["legacy", "auto"]) -> AsyncGenerator[Client]:
    async with httpx2.AsyncClient(headers={"Authorization": f"Bearer {bearer}"}) as http:
        async with Client(streamable_http_client(url, http_client=http), mode=mode) as client:
            yield client


def test_the_handler_sees_the_bearer_in_both_protocol_eras() -> None:
    """P-e-1 and P-e-2: two calls per era, and the handler saw the header on each."""

    async def scenario() -> tuple[list[str | None], list[str]]:
        seen: list[str | None] = []
        versions: list[str] = []
        async with _serving(_probe_server(seen, InMemorySubscriptionBus())) as url:
            modes: tuple[Literal["legacy", "auto"], ...] = ("legacy", "auto")
            for mode in modes:
                async with _client(url, "token-one-2345", mode) as client:
                    await client.call_tool("probe", {})
                    await client.call_tool("probe", {})
                    versions.append(str(client.session.protocol_version))
        return seen, versions

    seen, versions = run(scenario(), timeout_s=60)
    assert seen == ["Bearer token-one-2345"] * 4
    assert versions[0] != versions[1], versions


def test_a_listener_hears_a_cue_over_stateless_http() -> None:
    """P-e-3: one `ResourceUpdated` after a publish on the bus."""

    async def scenario() -> ResourceUpdated | None:
        bus = InMemorySubscriptionBus()
        async with _serving(_probe_server([], bus)) as url:
            async with _client(url, "token-one-2345", "auto") as client:
                async with client.listen(resource_subscriptions=[URI]) as subscription:
                    await bus.publish(ResourceUpdated(uri=URI))
                    event = await asyncio.wait_for(subscription.__anext__(), timeout=10)
        return event if isinstance(event, ResourceUpdated) else None

    event = run(scenario(), timeout_s=60)
    assert event is not None and event.uri == URI
