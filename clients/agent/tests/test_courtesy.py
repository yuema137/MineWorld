"""R-S27c-a-1 (S27-a merged first): a handle refused `rate_limited { retry_after_ms }` sends nothing more
until that many wall milliseconds have passed, on every path (an act, the pacer), and one request is in
flight at a time. A scripted seat answers; a virtual wall clock measures the wait."""

from __future__ import annotations

import asyncio
from dataclasses import dataclass, field
from typing import cast

from mineworld_sdk.perceived import CursorCell
from mineworld_sdk.resuming import ResumingSeat
from mineworld_sdk.session import Answered, RefusedRequest
from mineworld_sdk.session import Outcome as SdkOutcome
from mineworld_sdk.wire.contract import ActionRecord, ActionRequest
from mineworld_sdk.wire.ids import ActionId, ActionTypeId, EntityId, EntityKey
from support import run

from mineworld_agent.handle import Handle
from mineworld_agent.inbox import Inbox
from mineworld_agent.outcomes import outcome_of
from mineworld_agent.refs import RefTable

ME = EntityId("17")
STEP = ActionTypeId("walk-step")
REQUEST = ActionRequest(
    actor=ME, action_type=STEP, target=None, payload=ActionRecord(action_type=STEP, payload={})
)


@dataclass
class Clock:
    now: float = 0.0

    def wall(self) -> float:
        return self.now

    async def sleep(self, seconds: float) -> None:
        self.now += seconds
        await asyncio.sleep(0)


@dataclass
class Seat:
    """A seat whose server answers from a script, noting the wall time of every submit."""

    clock: Clock
    answers: list[SdkOutcome]
    sent: list[float] = field(default_factory=list[float])
    connection: int = 1

    async def submit(self, request: ActionRequest) -> SdkOutcome:
        del request
        self.sent.append(self.clock.now)
        return self.answers.pop(0)


def handle(seat: Seat, clock: Clock) -> Handle:
    refs = RefTable()
    return Handle(
        seat=cast(ResumingSeat, seat),
        key=EntityKey("visitor"),
        refs=refs,
        inbox=Inbox(refs, ME),
        cursor=CursorCell(),
        wall=clock.wall,
        sleep=clock.sleep,
        idle_release_s=0,
        states=None,
    )


def test_the_next_submit_waits_what_the_server_asked() -> None:
    clock = Clock()
    accepted = Answered(ActionId("9"), "unavailable")
    seat = Seat(clock, [RefusedRequest("rate_limited", None, 2500), accepted, accepted])

    async def go() -> list[str]:
        h = handle(seat, clock)
        kinds: list[str] = []
        for _ in range(3):
            _, answer = await h.send(REQUEST)
            kinds.append(outcome_of(answer, []).kind)
        return kinds

    kinds = run(go(), timeout_s=5)
    assert kinds == ["refused", "unavailable", "unavailable"]
    assert seat.sent[1] - seat.sent[0] >= 2.5, seat.sent
    assert seat.sent[2] == seat.sent[1], "after the wait, nothing more is held back"


def test_a_rate_limited_outcome_says_so() -> None:
    answer = outcome_of(RefusedRequest("rate_limited", None, 100), [])
    assert answer.model_dump() == {"kind": "refused", "code": "rate_limited"}
