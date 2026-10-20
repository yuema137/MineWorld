"""The walk pacer: one `walk-step` per wall second while my own walk is disclosed (step-27 §5.6).

It is the 2D client's rule (`clients/protocol/ADOPTION.md` §4.2), not an autopilot beyond a human's: once a
`walk-to` is accepted, the walk is this seat's own `walking` record in the next observations, and it moves
nobody by itself; the client sends one `walk-step` (no payload) per wall second while that record is
disclosed, and stops when it is gone. The pacer plans no route, checks no distance and splits nothing: the
server does all of that, and answers each step like a `move`.

Everything it depends on is injected — whether my walk is disclosed, whether the host paused the world, the
step itself, the wall clock and the sleep — so that the cadence is tested without a server
(`tests/test_pacer.py`). The `step` callable is the client's: it keeps one request in flight per seat
and maps the answer to a `Step`. It moved here from `clients/agent` so that every client that walks a seat
paces it the same way — the agent Player and the LM seat's `go` (`pr-le-leave-early.md` §0.4 item 3, LE-c
C2). It names no pack's vocabulary: the `walk-step` request is the injected `step`'s.
"""

from __future__ import annotations

import asyncio
from collections.abc import Awaitable, Callable
from dataclasses import dataclass
from typing import Literal

__all__ = ["INTERVAL", "Pacer", "Step", "Wait", "Walking"]

INTERVAL = 1.0
"""Wall seconds between two steps: the 2D client's cadence (ADOPTION.md §4.2, AC-S27-14)."""

POLL = 0.25
"""Wall seconds between two looks while the world is paused or the newest observation does not yet
reflect the last request. Looking sends nothing."""

UNSEEN_PATIENCE = 5.0
"""Wall seconds the pacer waits for an observation that reflects its last request before it stops: a
seat whose observations stopped arriving is not walked blind."""

type Walking = Literal["walking", "gone", "unseen"]
"""My own walk in the newest observation: disclosed, not disclosed, or not yet known because that
observation does not reflect the last request this handle sent (`acted_through`)."""


@dataclass(frozen=True, slots=True)
class Wait:
    """The server asked for patience (`rate_limited`, `retry_after_ms`): wait, then step again."""

    seconds: float


type Step = Literal["stepped", "stop"] | Wait
"""What one `walk-step` came to: accepted (keep pacing), an answer that ends the walk (`TooFarAway`,
`Busy`, any other rejection or refusal, a lost seat), or a courtesy wait."""

type Sleep = Callable[[float], Awaitable[None]]


class Pacer:
    """Paces one handle's walk. `start` after an accepted `walk-to`; `stop` on a `move`, a `stop`, a new
    `walk_to` or `leave`. At most one loop runs, and it sends at most one step per `interval`."""

    def __init__(
        self,
        *,
        walking: Callable[[], Walking],
        paused: Callable[[], bool],
        step: Callable[[], Awaitable[Step]],
        wall: Callable[[], float],
        sleep: Sleep,
        interval: float = INTERVAL,
    ) -> None:
        self._walking = walking
        self._paused = paused
        self._step = step
        self._wall = wall
        self._sleep = sleep
        self._interval = interval
        self._task: asyncio.Task[None] | None = None
        self._last_sent: float | None = None

    @property
    def running(self) -> bool:
        return self._task is not None and not self._task.done()

    async def start(self) -> None:
        """Starts pacing, ending a walk already paced. The interval holds across a restart: a new walk
        does not send its first step sooner than one interval after the last step of the previous one."""
        await self.stop()
        self._task = asyncio.create_task(self._run())

    async def stop(self) -> None:
        task, self._task = self._task, None
        if task is None or task.done():
            return
        task.cancel()
        await asyncio.gather(task, return_exceptions=True)

    async def wait(self) -> None:
        """Returns when the current loop has ended by itself (tests; `close`)."""
        task = self._task
        if task is not None:
            await asyncio.gather(task, return_exceptions=True)

    async def _run(self) -> None:
        unseen_since: float | None = None
        while True:
            seen = self._walking()
            if seen == "gone":
                return
            if seen == "unseen" or self._paused():
                now = self._wall()
                if seen == "unseen":
                    unseen_since = now if unseen_since is None else unseen_since
                    if now - unseen_since >= UNSEEN_PATIENCE:
                        return
                await self._sleep(POLL)
                continue
            unseen_since = None
            if self._last_sent is not None:
                due = self._last_sent + self._interval - self._wall()
                if due > 0:
                    await self._sleep(due)
                    continue
            self._last_sent = self._wall()
            answer = await self._step()
            if answer == "stop":
                return
            if isinstance(answer, Wait):
                await self._sleep(answer.seconds)
