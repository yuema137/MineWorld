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
from pytest_socket import SocketConnectBlockedError


def test_a_connection_beyond_localhost_is_blocked_by_the_guard() -> None:
    began = time.monotonic()
    with pytest.raises(SocketConnectBlockedError):
        socket.create_connection(("192.0.2.1", 80), timeout=5)
    assert time.monotonic() - began < 1.0, "refused by the guard, not by a network timeout"


def test_loopback_stays_reachable() -> None:
    with socket.create_server(("127.0.0.1", 0)) as server:
        with socket.create_connection(server.getsockname()[:2], timeout=5):
            pass
