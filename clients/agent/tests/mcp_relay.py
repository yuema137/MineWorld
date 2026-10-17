"""A WebSocket relay between the MCP adapter (a child process) and the real server (S27-d §9, C5; R-d-7).

The adapter connects to the relay's `/ws`; for each such connection the relay opens one connection to the real
server's `/ws` and forwards text frames both ways unchanged. It records every frame the adapter sent (`sent`), the
server's welcome frames (`welcomes`, with their resume secrets, which AC-d-4 reads), and every frame the adapter
received. `abort()` cuts every transport at once, with no close frame, as `realserver.Tap` does;
`cut_on_next_submit` cuts them just after a submit, withholding its answer, and refuses reconnections for a while
(step 8's `answer_lost` and `not_connected` arms). A `GET /status`
is answered with the real server's `/status`: the adapter derives its status URL from `--server` (D-8), so the
adapter connects to the relay alone (AC-d-7).
"""

from __future__ import annotations

import asyncio
import json
import time
import urllib.request
from typing import Any

from mineworld_sdk.wire.ids import JsonValue
from websockets.asyncio.client import ClientConnection
from websockets.asyncio.client import connect as websocket_connect
from websockets.asyncio.server import Server as WebSocketServer
from websockets.asyncio.server import ServerConnection, serve
from websockets.datastructures import Headers
from websockets.http11 import Request, Response

MAX_FRAME = 2**24


class Relay:
    """One adapter's path to one real server, observed."""

    def __init__(self, upstream_ws: str, upstream_http: str) -> None:
        self.upstream_ws = upstream_ws
        self.upstream_http = upstream_http
        self.sent: list[dict[str, JsonValue]] = []
        self.timed_sent: list[tuple[float, dict[str, JsonValue]]] = []
        """Each frame the adapter sent, with the monotonic time the relay received it (AC-d-5's pacing)."""
        self.received: list[str] = []
        self.welcomes: list[dict[str, JsonValue]] = []
        self._transports: list[Any] = []
        self._server: WebSocketServer | None = None
        self.port = 0
        self._cut_for: float | None = None
        self._refuse_until = 0.0
        self.cut_submit: dict[str, JsonValue] | None = None
        """The submit the last `cut_on_next_submit` let through to the server, its answer withheld."""
        self.refused = 0
        """How many adapter connections the relay refused while a cut lasted."""

    async def start(self) -> str:
        """Listens on loopback and returns the adapter's `--server` URL."""
        self._server = await serve(
            self._pair,
            "127.0.0.1",
            0,
            process_request=self._status,
            compression=None,
            max_size=MAX_FRAME,
        )
        self.port = self._server.sockets[0].getsockname()[1]
        return f"ws://127.0.0.1:{self.port}/ws"

    async def close(self) -> None:
        self.abort()
        if self._server is not None:
            self._server.close()
            await self._server.wait_closed()

    def abort(self) -> None:
        """Cuts every transport now: no close frame, so the server sees a dropped connection."""
        for transport in self._transports:
            transport.abort()

    def cut_on_next_submit(self, *, lasting_s: float) -> None:
        """Arms a cut: the adapter's next `submit` reaches the server, then every transport is aborted before any
        server frame after it is forwarded, so its answer is lost; for `lasting_s` wall seconds after that, the
        relay refuses the adapter's reconnections (HTTP 503), so the seat stays between connections."""
        self._cut_for = lasting_s

    def frames_sent(self, kind: str) -> list[dict[str, JsonValue]]:
        return [frame for frame in self.sent if frame.get("t") == kind]

    async def _status(self, _connection: ServerConnection, request: Request) -> Response | None:
        if request.path == "/ws" and time.monotonic() < self._refuse_until:
            self.refused += 1
            return Response(503, "Service Unavailable", Headers(), b"the relay is cut\n")
        if request.path != "/status":
            return None
        body = await asyncio.to_thread(_fetch, f"{self.upstream_http}/status")
        headers = Headers([("Content-Type", "application/json")])
        return Response(200, "OK", headers, body)

    async def _pair(self, adapter: ServerConnection) -> None:
        upstream = await websocket_connect(self.upstream_ws, compression=None, max_size=MAX_FRAME)
        self._transports += [adapter.transport, upstream.transport]
        withheld = (
            asyncio.Event()
        )  # set when this pair's cut began: no server frame is forwarded after it
        try:
            pumps = [
                asyncio.ensure_future(self._adapter_to_server(adapter, upstream, withheld)),
                asyncio.ensure_future(self._server_to_adapter(upstream, adapter, withheld)),
            ]
            _, pending = await asyncio.wait(pumps, return_when=asyncio.FIRST_COMPLETED)
            for task in pending:
                task.cancel()
            await asyncio.gather(*pending, return_exceptions=True)
        finally:
            await _close_quietly(upstream)
            await _close_quietly(adapter)

    async def _adapter_to_server(
        self, adapter: ServerConnection, upstream: ClientConnection, withheld: asyncio.Event
    ) -> None:
        async for message in adapter:
            cut = False
            if isinstance(message, str):
                parsed: JsonValue = json.loads(message)
                if isinstance(parsed, dict):
                    self.sent.append(parsed)
                    self.timed_sent.append((time.monotonic(), parsed))
                    if parsed.get("t") == "submit" and self._cut_for is not None:
                        withheld.set()  # before the send, so not even a fast answer gets through
                        self.cut_submit = parsed
                        cut = True
            await upstream.send(message)
            if cut and self._cut_for is not None:
                self._refuse_until = time.monotonic() + self._cut_for
                self._cut_for = None
                self.abort()
                return

    async def _server_to_adapter(
        self, upstream: ClientConnection, adapter: ServerConnection, withheld: asyncio.Event
    ) -> None:
        async for message in upstream:
            if withheld.is_set():
                continue
            if isinstance(message, str):
                self.received.append(message)
                if '"t":"welcome"' in message.replace(" ", ""):
                    parsed: JsonValue = json.loads(message)
                    if isinstance(parsed, dict):
                        self.welcomes.append(parsed)
            await adapter.send(message)


def _fetch(url: str) -> bytes:
    with urllib.request.urlopen(url, timeout=10) as answer:
        body: bytes = answer.read()
    return body


async def _close_quietly(connection: ServerConnection | ClientConnection) -> None:
    try:
        await connection.close()
    except Exception:
        return
