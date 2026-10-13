"""AP-8: the suite reaches nothing but this machine (I-11).

The guard is pytest-socket, configured once for every test and every platform in `pyproject.toml`
(`--allow-hosts=127.0.0.1,::1`). This test is the evidence that it is on: an address that is never
routable (TEST-NET-1, RFC 5737) fails at once with the guard's own error, rather than after a timeout
or as an `OSError` from the network.
"""

from __future__ import annotations

import socket
import time

import pytest
import support
from pytest_socket import SocketConnectBlockedError
from websockets.asyncio.client import connect as websocket_connect


def test_a_connection_beyond_localhost_is_blocked_by_the_guard() -> None:
    began = time.monotonic()
    with pytest.raises(SocketConnectBlockedError):
        socket.create_connection(("192.0.2.1", 80), timeout=5)
    assert time.monotonic() - began < 1.0, "refused by the guard, not by a network timeout"


def test_an_asynchronous_connection_beyond_localhost_is_blocked_too() -> None:
    # AP3b-14 (F-P5-4): the SDK connects through asyncio and `websockets`, not a blocking socket. Under
    # `support.run` (the selector loop on every platform) the guard sees that connection on Windows too;
    # on the proactor loop it would go through `ConnectEx` and time out instead.
    async def attempt() -> None:
        await websocket_connect("ws://192.0.2.1:80/ws", open_timeout=5)

    began = time.monotonic()
    with pytest.raises(SocketConnectBlockedError):
        support.run(attempt(), timeout_s=5)
    assert time.monotonic() - began < 1.0, "refused by the guard, not by a network timeout"


def test_loopback_stays_reachable() -> None:
    with socket.create_server(("127.0.0.1", 0)) as server:
        with socket.create_connection(server.getsockname()[:2], timeout=5):
            pass
