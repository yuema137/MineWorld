"""A deterministic backend for tests: a function answers each request (D-P5-5, `scripted` mode)."""

from __future__ import annotations

import asyncio
from collections.abc import Callable

from mineworld_cognition.backend.model import BackendFailure, Completion, CompletionRequest

Script = Callable[[CompletionRequest], Completion | BackendFailure]


class ScriptedBackend:
    """Answers with `script(request)`. `delay_s` holds each answer back, so a test can make a call outlast
    the gateway's timeout; `calls` counts the requests that reached it."""

    def __init__(self, script: Script, *, delay_s: float = 0) -> None:
        self._script = script
        self._delay_s = delay_s
        self.calls = 0
        self.closed = False

    async def complete(self, request: CompletionRequest) -> Completion | BackendFailure:
        self.calls += 1
        if self._delay_s:
            await asyncio.sleep(self._delay_s)
        return self._script(request)

    async def aclose(self) -> None:
        self.closed = True

    def __repr__(self) -> str:
        return f"ScriptedBackend(calls={self.calls})"
