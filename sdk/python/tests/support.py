"""The coroutine runner every SDK test module shares.

By default a coroutine runs on `asyncio.SelectorEventLoop` on every platform: the selector loop connects
with `socket.connect`, which the network guard (pytest-socket) patches, while Windows' default proactor
loop connects through `ConnectEx`, which the guard does not see (ledger F-P5-4; #142 moved the suite to
the selector loop). `loop="default"` runs on the platform's own default loop instead — the proactor on
Windows, the loop a cognition process with a subscription bridge must run (F-P5b-1) — for the test that
proves the SDK works there (design §4.6, AP3b-9). Its connections then go only to the loopback address
its own server printed.
"""

from __future__ import annotations

import asyncio
from collections.abc import Coroutine
from typing import Any, Literal


def run[T](
    coroutine: Coroutine[Any, Any, T],
    *,
    timeout_s: float,
    loop: Literal["selector", "default"] = "selector",
) -> T:
    """Runs `coroutine` to completion within `timeout_s` wall seconds on the chosen event loop."""
    bounded = asyncio.wait_for(coroutine, timeout_s)
    if loop == "default":
        return asyncio.run(bounded)
    return asyncio.run(bounded, loop_factory=asyncio.SelectorEventLoop)
