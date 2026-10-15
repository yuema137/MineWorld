"""What an act came to, as the agent reads it: one table, from every SDK outcome (pr-s27c §5.6).

```text
Answered(Accepted)            → accepted { events }   the facts it caused that this seat perceived
Answered(Rejected(r))         → rejected { reason, system_code? }   the server's reason, unchanged
Answered("unavailable")       → unavailable
RefusedRequest(code)          → refused { code }   paused, rate_limited, dispatch_failed, world_stopped,
                                                    actor_not_observer
AnswerLost                    → refused { answer_lost }   may or may not have been applied: look again
NotConnected                  → refused { not_connected }   nothing was sent
SeatLost(r)                   → seat_lost { reason: r }
NotOffered (local)            → refused { not_offered }   exactly the SDK's offers check
```

A refusal code the library can never meet through its own requests (`malformed_frame`, `not_joined`, …)
is a defect of this library, not a world answer, and raises `LibraryDefect`.
"""

from __future__ import annotations

from typing import Final, cast

from mineworld_sdk.reconnect import AnswerLost, NotConnected
from mineworld_sdk.reconnect import SeatLost as SdkSeatLost
from mineworld_sdk.session import Answered, RefusedRequest
from mineworld_sdk.session import Outcome as SdkOutcome
from mineworld_sdk.wire.contract import Accepted as WireAccepted

from mineworld_agent.models import (
    Accepted,
    EventLine,
    Outcome,
    RefusalReason,
    Refused,
    Rejected,
    SeatLost,
    Unavailable,
)
from mineworld_agent.view import reason_line

__all__ = ["SERVER_REFUSALS", "LibraryDefect", "outcome_of", "unanswered"]

SERVER_REFUSALS: Final = frozenset(
    {"paused", "rate_limited", "dispatch_failed", "world_stopped", "actor_not_observer"}
)
"""The server's refusals a submit through this library can meet (`PROTOCOL.md` §5.5; `rate_limited`
lands with S27-a and is mapped by its code from the start, F-S27c-6)."""


class LibraryDefect(RuntimeError):
    """The server refused a frame this library built, for a reason only a malformed frame earns."""


def outcome_of(answer: SdkOutcome, caused: list[EventLine]) -> Outcome:
    """The agent's outcome of an answered or refused submit. `caused` are the lines of the facts an
    accepted request caused, already taken from the inbox (shown once)."""
    if isinstance(answer, RefusedRequest):
        if answer.code not in SERVER_REFUSALS:
            raise LibraryDefect(f"the server refused a request this library built: {answer.code}")
        return Refused(code=cast(RefusalReason, answer.code))
    return _answered(answer, caused)


def _answered(answer: Answered, caused: list[EventLine]) -> Outcome:
    result = answer.result
    if isinstance(result, WireAccepted):
        return Accepted(events=caused)
    if result == "unavailable":
        return Unavailable()
    line = reason_line(result.rejected)
    return Rejected(reason=line.reason, system_code=line.system_code)


def unanswered(error: AnswerLost | NotConnected | SdkSeatLost) -> Outcome:
    """The outcome of a submit that got no answer."""
    match error:
        case AnswerLost():
            return Refused(code="answer_lost")
        case NotConnected():
            return Refused(code="not_connected")
        case SdkSeatLost():
            return SeatLost(reason=error.reason)
