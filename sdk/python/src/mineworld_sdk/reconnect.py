"""The reconnect decision table (design §4.5) and its backoff, apart from the seat that applies them.

`resuming.ResumingSeat` asks two questions here: what does the end of a connection mean (`end_outcome`),
and what does a failed join mean (`join_outcome`). Each answer is "rejoin" (`Retry`), "the seat is lost"
(`SeatLost` with its reason), or — for an error this table does not know — the error itself, which
reaches the caller unchanged (D-P3b-5). The delays between attempts are ours, seeded, and read no
environment variable (D-P3b-9, DEP-44).
"""

from __future__ import annotations

import random
from collections.abc import Iterator
from dataclasses import dataclass
from typing import Literal

from mineworld_sdk.errors import JoinRefused, MineWorldError, ProtocolMismatch, SessionClosed
from mineworld_sdk.perceived import LocalLag
from mineworld_sdk.session import TRANSPORT_FAILURES

__all__ = [
    "AnswerLost",
    "NotConnected",
    "ReconnectPolicy",
    "Retry",
    "SeatLossReason",
    "SeatLost",
    "delays",
    "end_outcome",
    "join_outcome",
]

SeatLossReason = Literal[
    "taken_over",
    "superseded",
    "kicked",
    "world_stopped",
    "unauthorized",
    "protocol_mismatch",
    "seat_occupied",
    "unknown_seat",
    "seat_not_in_world",
    "invalid_nickname",
    "cursor_unavailable",
    "world_changed",
    "protocol_violation",
    "gave_up",
]
"""Why a seat ended for good (rows 3 and 8 to 12)."""

_FINAL_CLOSINGS: dict[str, SeatLossReason] = {
    "taken_over": "taken_over",
    "superseded": "superseded",
    "kicked": "kicked",
    "world_stopped": "world_stopped",
    "unauthorized": "unauthorized",
    "protocol_mismatch": "protocol_mismatch",
}
_RETRIED_CLOSINGS = (None, "lagged", "server_stopping")
_FINAL_REFUSALS: dict[str, SeatLossReason] = {
    **_FINAL_CLOSINGS,
    "cursor_unavailable": "cursor_unavailable",
    "seat_occupied": "seat_occupied",
    "unknown_seat": "unknown_seat",
    "seat_not_in_world": "seat_not_in_world",
    "invalid_nickname": "invalid_nickname",
}


@dataclass(frozen=True, slots=True)
class ReconnectPolicy:
    """How a seat retries. The first attempt after a loss is immediate; then `first_delay * factor**k`,
    capped at `ceiling`, each scaled by a factor drawn from `[1 - jitter, 1]` by `random.Random(seed)`,
    until `give_up_after` wall seconds have passed since the loss (D-P3b-9, Q-P3b-7)."""

    first_delay: float = 0.1
    factor: float = 2.0
    ceiling: float = 5.0
    jitter: float = 0.5
    give_up_after: float = 120.0
    seed: int = 0
    perceived_buffer: int = 4096
    """Facts accepted but not yet taken by the consumer before a local `lagged` (D-P3b-4)."""


class SeatLost(MineWorldError):
    """The seat ended for good. `reason` says why; `cause` is the error behind it, if any."""

    def __init__(self, reason: SeatLossReason, cause: MineWorldError | None = None) -> None:
        detail = "" if cause is None else f" ({type(cause).__name__}: {cause})"
        super().__init__(f"the seat is lost: {reason}{detail}")
        self.reason: SeatLossReason = reason
        self.cause = cause


class AnswerLost(MineWorldError):
    """The connection carrying a submit ended before its answer. It is not re-sent: the caller reads
    what happened from later observations (`acted_through`) and facts (`Causation`)."""

    def __init__(self) -> None:
        super().__init__("the connection ended before the request was answered; it is not re-sent")


class NotConnected(MineWorldError):
    """A submit while the seat is between connections. Nothing was sent."""

    def __init__(self) -> None:
        super().__init__("the seat is reconnecting; nothing was sent")


class Retry(Exception):
    """One attempt failed in a way the table retries (rows 4 to 7). `resume_refused` is row 7: retry
    at once, without `resume`."""

    def __init__(self, *, resume_refused: bool = False) -> None:
        super().__init__()
        self.resume_refused = resume_refused


def end_outcome(ended: MineWorldError) -> SeatLost | None:
    """What the end of a welcomed connection means: `None` to rejoin (rows 4 to 6), or the loss (rows 3
    and 11). A violation of any kind is row 11, with the violation as the cause."""
    if isinstance(ended, LocalLag):
        return None
    if isinstance(ended, SessionClosed):
        if ended.reason in _RETRIED_CLOSINGS:
            return None
        reason = _FINAL_CLOSINGS.get(ended.reason or "")
        if reason is not None:
            return SeatLost(reason)
    return SeatLost("protocol_violation", ended)


def join_outcome(failure: BaseException) -> BaseException:
    """What one failed join means (rows 6 to 9 and 11): a `Retry`, a `SeatLost`, or `failure` itself
    when this table does not know it."""
    if isinstance(failure, JoinRefused):
        if failure.code == "invalid_resume":
            return Retry(resume_refused=True)
        if failure.code == "server_stopping":
            return Retry()
        reason = _FINAL_REFUSALS.get(failure.code)
        return SeatLost(reason or "protocol_violation", failure)
    if isinstance(failure, ProtocolMismatch):
        return SeatLost("protocol_mismatch", failure)
    if isinstance(failure, TRANSPORT_FAILURES):
        return Retry()
    if isinstance(failure, MineWorldError):
        return SeatLost("protocol_violation", failure)
    return failure


def delays(policy: ReconnectPolicy, rng: random.Random) -> Iterator[float]:
    """The waits before each attempt after one loss: 0, then the capped, jittered exponential."""
    yield 0.0
    delay = policy.first_delay
    while True:
        yield min(delay, policy.ceiling) * rng.uniform(1 - policy.jitter, 1)
        delay = min(delay * policy.factor, policy.ceiling)
