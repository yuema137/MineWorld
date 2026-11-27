"""The absolute consult deadline (pr-playable-dialogue-memory §4 items 4 to 6, C1).

`consult_deadline_s` is "wall seconds from a trigger's receipt to the submit" (`docs/reference/cognition-toml.md`).
A trigger's receipt R, the runner's clock when `Pending.offer` took it, fixes its `Expiry` E = R +
`consult_deadline_s` once. Every later stage spends that one budget: the pending wait, the wait for a first
observation, the consult's preparation, the gateway's admission and backend call, every retry delay, the submit
lock and the dispatch. No stage starts a fresh one. No backend call and no submit begins at or after E; the submit
is checked again once the submit lock is held, immediately before the send (`send_locked`).

A submit sent before E may be answered after E. That answer is uncertain, and it is never sent again.

The clock is the runner's monotonic wall clock. A bounded wait uses the event loop's timer for what is left of E
when the wait begins; in production both are `time.monotonic`, and a caller re-checks the clock after the wait.
"""

from __future__ import annotations

import asyncio
import math
from collections.abc import Awaitable, Callable
from dataclasses import dataclass

type Clock = Callable[[], float]


@dataclass(frozen=True)
class Expiry:
    """The instant, on the runner's clock, at and after which a trigger's consult begins nothing new."""

    at: float

    @staticmethod
    def after(received_at: float, deadline_s: float) -> Expiry:
        return Expiry(received_at + deadline_s)

    def remaining(self, now: float) -> float:
        return self.at - now

    def passed(self, now: float) -> bool:
        return now >= self.at


NEVER = Expiry(math.inf)
"""No deadline: the walk pacer's submits, which belong to no trigger."""


@dataclass(frozen=True)
class Expired:
    """A step that was not begun because its trigger's expiry had passed."""


EXPIRED = Expired()


async def before[T](expiry: Expiry, clock: Clock, wait: Callable[[], Awaitable[T]]) -> T | Expired:
    """`wait()`, begun only before `expiry` and bounded by what is left of it; `EXPIRED` otherwise. A result
    does not say the expiry is still ahead: the caller checks the clock again."""
    left = expiry.remaining(clock())
    if left <= 0:
        return EXPIRED
    try:
        async with asyncio.timeout(None if math.isinf(left) else left):
            return await wait()
    except TimeoutError:
        return EXPIRED


async def send_locked[R, T](
    lock: asyncio.Lock, expiry: Expiry, clock: Clock, send: Callable[[R], Awaitable[T]], request: R
) -> T | Expired:
    """`send(request)` under `lock`, begun only before `expiry`: the lock is waited for within what is left, and the
    expiry is checked again once the lock is held, immediately before the send."""
    acquired = await before(expiry, clock, lock.acquire)
    if isinstance(acquired, Expired):
        return EXPIRED
    try:
        if expiry.passed(clock()):
            return EXPIRED
        return await send(request)
    finally:
        lock.release()
