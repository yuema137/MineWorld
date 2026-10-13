"""Cognition budgets: cost ceilings keyed on wall time, decided before any call (ARC-57; D-P5-8).

Per seat, two rolling windows: calls per wall hour and tokens per wall day. Per process: a limit on calls
in flight, and a timeout per call (applied by the gateway). Every window reads only `Clock.now()`, the
wall clock in UTC epoch seconds: nothing here knows simulated time (P5-3). The context bound, which is on
simulated time, is memory's and context assembly's (P4, P6), not this module's.
"""

from __future__ import annotations

import asyncio
import sqlite3
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Annotated, Literal, Protocol

from mineworld_sdk.wire.ids import EntityKey
from pydantic import BaseModel, ConfigDict, Field

from mineworld_cognition.backend.model import CompletionRequest, estimate_tokens

HOUR_S = 3600
DAY_S = 86400


class Clock(Protocol):
    def now(self) -> int:
        """Wall time, UTC epoch seconds."""
        ...


class SystemClock:
    def now(self) -> int:
        return int(time.time())


class BudgetPolicy(BaseModel):
    """The ceilings (QP5-4's defaults: `ARCHITECTURE.md` §9.1's numbers, re-keyed on wall time)."""

    model_config = ConfigDict(strict=True, frozen=True, extra="forbid")

    calls_per_wall_hour: Annotated[int, Field(ge=1)] = 20
    tokens_per_wall_day: Annotated[int, Field(ge=1)] = 30000
    max_in_flight: Annotated[int, Field(ge=1)] = 2
    call_timeout_s: Annotated[int, Field(ge=1)] = 20


Limit = Literal["calls_per_wall_hour", "tokens_per_wall_day"]


@dataclass(frozen=True)
class BudgetRefusal:
    """Why a call was not made: which ceiling, what was already spent in its window, and the ceiling."""

    limit: Limit
    spent: int
    asked: int
    ceiling: int


@dataclass(frozen=True)
class Window:
    calls: int
    tokens: int


class Ledger(Protocol):
    """What each seat spent, and when (wall seconds)."""

    def window(self, seat: EntityKey, now: int, seconds: int) -> Window:
        """The sums of every charge made after `now - seconds`."""
        ...

    def charge(
        self, seat: EntityKey, at: int, *, calls: int, tokens: int, estimated: bool
    ) -> None: ...

    def close(self) -> None: ...


@dataclass(frozen=True)
class _Charge:
    seat: EntityKey
    at: int
    calls: int
    tokens: int
    estimated: bool


class MemoryLedger:
    """For tests: forgets everything when the process ends."""

    def __init__(self) -> None:
        self._charges: list[_Charge] = []

    def window(self, seat: EntityKey, now: int, seconds: int) -> Window:
        inside = [c for c in self._charges if c.seat == seat and c.at > now - seconds]
        return Window(sum(c.calls for c in inside), sum(c.tokens for c in inside))

    def charge(self, seat: EntityKey, at: int, *, calls: int, tokens: int, estimated: bool) -> None:
        self._charges.append(_Charge(seat, at, calls, tokens, estimated))

    def close(self) -> None:
        return None


class SqliteLedger:
    """Durable across restarts (R-P5-5), at a path the operator configures — never in a world save."""

    def __init__(self, path: Path) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        self._connection = sqlite3.connect(path)
        self._connection.execute(
            "CREATE TABLE IF NOT EXISTS budget_ledger ("
            "seat TEXT NOT NULL, at INTEGER NOT NULL, calls INTEGER NOT NULL, "
            "tokens INTEGER NOT NULL, estimated INTEGER NOT NULL)"
        )
        self._connection.commit()

    def window(self, seat: EntityKey, now: int, seconds: int) -> Window:
        row: tuple[int, int] = self._connection.execute(
            "SELECT COALESCE(SUM(calls), 0), COALESCE(SUM(tokens), 0) FROM budget_ledger "
            "WHERE seat = ? AND at > ?",
            (seat, now - seconds),
        ).fetchone()
        return Window(row[0], row[1])

    def charge(self, seat: EntityKey, at: int, *, calls: int, tokens: int, estimated: bool) -> None:
        self._connection.execute(
            "INSERT INTO budget_ledger (seat, at, calls, tokens, estimated) VALUES (?, ?, ?, ?, ?)",
            (seat, at, calls, tokens, int(estimated)),
        )
        self._connection.commit()

    def close(self) -> None:
        """Closes the connection, which Windows requires before the file is moved or deleted."""
        self._connection.close()


def reservation(request: CompletionRequest) -> int:
    """The tokens a call may cost at most: its estimated input plus every output token it may use."""
    asked = "".join(message.text for message in request.messages)
    return estimate_tokens(asked) + request.sampling.max_output_tokens


class Reservations:
    """Calls admitted but not yet charged, so two concurrent calls cannot both pass a pre-check that
    only one of them fits."""

    def __init__(self) -> None:
        self._held: dict[EntityKey, Window] = {}

    def held(self, seat: EntityKey) -> Window:
        return self._held.get(seat, Window(0, 0))

    def hold(self, seat: EntityKey, tokens: int) -> None:
        current = self.held(seat)
        self._held[seat] = Window(current.calls + 1, current.tokens + tokens)

    def release(self, seat: EntityKey, tokens: int) -> None:
        current = self.held(seat)
        self._held[seat] = Window(current.calls - 1, current.tokens - tokens)


def precheck(
    policy: BudgetPolicy,
    ledger: Ledger,
    reservations: Reservations,
    seat: EntityKey,
    now: int,
    tokens: int,
) -> BudgetRefusal | None:
    """Refuses when one more call, or `tokens` more tokens, would exceed a ceiling."""
    held = reservations.held(seat)
    hour = ledger.window(seat, now, HOUR_S)
    spent_calls = hour.calls + held.calls
    if spent_calls + 1 > policy.calls_per_wall_hour:
        return BudgetRefusal("calls_per_wall_hour", spent_calls, 1, policy.calls_per_wall_hour)
    day = ledger.window(seat, now, DAY_S)
    spent_tokens = day.tokens + held.tokens
    if spent_tokens + tokens > policy.tokens_per_wall_day:
        return BudgetRefusal(
            "tokens_per_wall_day", spent_tokens, tokens, policy.tokens_per_wall_day
        )
    return None


class InFlightLimiter:
    """At most `limit` calls in flight in this process, admitted in arrival order (asyncio's semaphore
    wakes its waiters first in, first out)."""

    def __init__(self, limit: int) -> None:
        self._semaphore = asyncio.Semaphore(limit)

    async def __aenter__(self) -> None:
        await self._semaphore.acquire()

    async def __aexit__(self, *_: object) -> None:
        self._semaphore.release()
