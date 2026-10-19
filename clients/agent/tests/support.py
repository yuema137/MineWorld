"""The coroutine runner every test module of this member shares.

A coroutine runs on `asyncio.SelectorEventLoop` on every platform, as the SDK's tests do: the selector loop
connects with `socket.connect`, which the network guard (pytest-socket) patches, while Windows' default
proactor loop connects through `ConnectEx`, which the guard does not see (SDK ledger F-P5-4).
"""

from __future__ import annotations

import asyncio
from collections.abc import Coroutine
from typing import Any


def run[T](coroutine: Coroutine[Any, Any, T], *, timeout_s: float) -> T:
    """Runs `coroutine` to completion within `timeout_s` wall seconds on the selector loop."""
    return asyncio.run(
        asyncio.wait_for(coroutine, timeout_s), loop_factory=asyncio.SelectorEventLoop
    )


def run_default[T](coroutine: Coroutine[Any, Any, T], *, timeout_s: float) -> T:
    """Runs `coroutine` on the platform's default loop, for a test that starts a subprocess: Windows' selector
    loop cannot start one (S27-d F-S27d-6). Only the stdio files use it; the network guard still applies to the
    test process, and the adapter child is guarded by its own test (`test_mcp_guard.py`)."""
    return asyncio.run(asyncio.wait_for(coroutine, timeout_s))
