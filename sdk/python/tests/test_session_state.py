"""AP-4 and the session's error paths, with the server's frames given directly.

A `Script` stands in for the socket: it holds the frames the server would send and records what the
session sent. What these tests own is the session's own logic — pairing by token, newest-wins, and how
each error ends it. That the real server sends these frames, in this order, is AP-5's (real binary).
"""

from __future__ import annotations

import asyncio
import json
from collections.abc import Coroutine
from typing import Any

import pytest

from mineworld_sdk import offers
from mineworld_sdk.errors import MineWorldError
from mineworld_sdk.session import (
    Answered,
    ForeignObserver,
    JoinRefused,
    Outcome,
    ProtocolMismatch,
    ProtocolViolation,
    RefusedRequest,
    SeatSession,
    SessionClosed,
)
from mineworld_sdk.wire.contract import (
    ActionRecord,
    ActionRequest,
    Affordance,
    Observation,
    SpatialRequirement,
)
from mineworld_sdk.wire.frames import Invite
from mineworld_sdk.wire.ids import ActionTypeId, EntityId, EntityKey, JsonValue

ME = EntityId("101")
WORLD: JsonValue = {
    "protocol": 2,
    "instance": "1a2b3c4d5e6f70819293a4b5c6d7e8f9",
    "at": 0,
    "entities": 3,
    "systems": [],
    "seats": ["visitor"],
    "clients": 1,
    "observations_dropped": 0,
    "events_dropped": 0,
    "faults": 0,
    "revision": None,
}


def welcome(protocol: int = 2) -> str:
    return json.dumps(
        {
            "t": "welcome",
            "protocol": protocol,
            "seat": "visitor",
            "observer": ME,
            "nickname": "tester",
            "session": "1",
            "resume": None,
            "hold_seconds": 0,
            "took_over": "none",
            "world": WORLD,
        }
    )


def observation(seq: int, observer: str = ME) -> str:
    return json.dumps(
        {
            "t": "observation",
            "seq": seq,
            "revision": None,
            "observation": {
                "observer": observer,
                "at": seq,
                "self_location": None,
                "entities": [],
                "relations": [],
                "events": [],
                "affordances": [],
            },
        }
    )


def result(token: str, action_id: str) -> str:
    return json.dumps(
        {
            "t": "result",
            "token": token,
            "action_id": action_id,
            "result": {"accepted": {"events": []}},
        }
    )


class Script:
    """A socket whose server side is the test."""

    def __init__(self, *frames: str) -> None:
        self.inbound: asyncio.Queue[str] = asyncio.Queue()
        self.sent: list[JsonValue] = []
        self.closed = False
        for frame in frames:
            self.inbound.put_nowait(frame)

    async def send(self, message: str, /) -> None:
        self.sent.append(json.loads(message))

    async def recv(self) -> str:
        return await self.inbound.get()

    async def close(self) -> None:
        self.closed = True

    def say(self, frame: str) -> None:
        self.inbound.put_nowait(frame)


async def joined(script: Script) -> SeatSession:
    return await SeatSession.join(
        script, seat=EntityKey("visitor"), invite=Invite("not-a-secret"), nickname="tester"
    )


def request() -> ActionRequest:
    ring = ActionTypeId("ring")
    return ActionRequest(
        actor=ME, action_type=ring, payload=ActionRecord(action_type=ring, payload={})
    )


async def settle() -> None:
    for _ in range(5):
        await asyncio.sleep(0)


def run[T](coroutine: Coroutine[Any, Any, T]) -> T:
    return asyncio.run(asyncio.wait_for(coroutine, 5))


def test_answers_are_paired_by_token_not_by_order() -> None:
    async def scenario() -> list[Outcome]:
        script = Script(welcome())
        session = await joined(script)
        calls = [asyncio.create_task(session.submit(request())) for _ in range(3)]
        await settle()
        tokens = [frame["token"] for frame in script.sent[1:] if isinstance(frame, dict)]
        assert tokens == ["c1", "c2", "c3"]
        for token, action_id in (("c3", "30"), ("c2", "20"), ("c1", "10")):
            script.say(result(token, action_id))
        return list(await asyncio.gather(*calls))

    outcomes = run(scenario())
    assert [outcome.action_id for outcome in outcomes if isinstance(outcome, Answered)] == [
        "10",
        "20",
        "30",
    ]


def test_a_refusal_resolves_only_its_request_and_closing_fails_the_rest() -> None:
    async def scenario() -> None:
        script = Script(welcome())
        session = await joined(script)
        first, second, third = (asyncio.create_task(session.submit(request())) for _ in range(3))
        await settle()
        script.say(
            json.dumps({"t": "refused", "token": "c2", "code": "actor_not_observer", "detail": "x"})
        )
        assert await second == RefusedRequest("actor_not_observer", "x")
        assert not first.done() and not third.done()
        script.say(json.dumps({"t": "closing", "reason": "world_stopped"}))
        for call in (first, third):
            with pytest.raises(SessionClosed) as closed:
                await call
            assert closed.value.reason == "world_stopped"
        await settle()
        assert script.closed, "the client closes the socket on closing (PROTOCOL.md §5.6)"

    run(scenario())


def test_the_newest_observation_wins_and_a_stale_one_is_ignored() -> None:
    async def scenario() -> None:
        script = Script(welcome(), observation(2), observation(1))
        session = await joined(script)
        seen = await session.changed()
        await settle()
        assert seen.seq == 2
        assert session.newest is not None and session.newest.seq == 2
        later = asyncio.create_task(session.changed(since=2))
        await settle()
        assert not later.done()
        script.say(observation(3))
        assert (await later).seq == 3

    run(scenario())


def test_a_server_of_another_revision_is_refused() -> None:
    script = Script(welcome(protocol=3))
    with pytest.raises(ProtocolMismatch):
        run(joined(script))
    assert script.closed


def test_a_refused_join_raises_its_code_after_the_closing() -> None:
    script = Script(
        json.dumps({"t": "refused", "code": "unauthorized"}),
        json.dumps({"t": "closing", "reason": "unauthorized"}),
    )
    with pytest.raises(JoinRefused) as refused:
        run(joined(script))
    assert refused.value.code == "unauthorized"
    assert script.closed and script.inbound.empty()


def ended_by(*frames: str) -> MineWorldError:
    async def scenario() -> MineWorldError:
        script = Script(welcome(), *frames)
        session = await joined(script)
        pending = asyncio.create_task(session.submit(request()))
        with pytest.raises(MineWorldError) as ended:
            await session.changed(since=10**6)
        with pytest.raises(type(ended.value)):
            await pending
        assert script.closed
        return ended.value

    return run(scenario())


def test_an_observation_of_another_observer_ends_the_session() -> None:
    error = ended_by(observation(1, observer="102"))
    assert isinstance(error, ForeignObserver) and error.received == "102"


def test_an_answer_to_no_request_ends_the_session() -> None:
    assert isinstance(ended_by(result("c99", "1")), ProtocolViolation)


def test_a_refusal_naming_no_request_ends_the_session() -> None:
    assert isinstance(
        ended_by(json.dumps({"t": "refused", "code": "malformed_frame"})), ProtocolViolation
    )


def offering(available: bool) -> Observation:
    shoot = ActionTypeId("shoot")
    requirement = SpatialRequirement(
        place="any",
        within_range=None,
        requires_line_of_access=False,
        requires_target_available=True,
    )
    affordance = (
        Affordance(
            action_type=shoot,
            target=EntityId("7"),
            available=True,
            unavailable_reason=None,
            requirement=requirement,
        )
        if available
        else Affordance(
            action_type=shoot,
            target=EntityId("7"),
            available=False,
            unavailable_reason="busy",
            requirement=requirement,
        )
    )
    return Observation(
        observer=ME,
        at=0,
        self_location=None,
        entities=[],
        relations=[],
        events=(),
        affordances=[affordance],
    )


def test_offers_refuse_locally_what_the_observation_does_not_offer_and_send_nothing() -> None:
    # AP-6, the local half: no frame leaves the session for a request the verdict refuses.
    async def scenario() -> None:
        session = await joined(Script(welcome()))
        sent = session.sent_frames
        for seen, target in ((offering(False), "7"), (offering(True), "8")):
            with pytest.raises(offers.NotOffered):
                offers.request(seen, ActionTypeId("shoot"), target=EntityId(target), payload={})
        assert session.sent_frames == sent
        made = offers.request(
            offering(True), ActionTypeId("shoot"), target=EntityId("7"), payload={}
        )
        assert made.actor == ME and made.payload.action_type == "shoot"

    run(scenario())


def test_an_incomplete_or_foreign_affordance_cannot_be_attempted() -> None:
    seen = offering(True)
    with pytest.raises(offers.NotOffered, match="not a complete affordance"):
        offers.attempt(seen, seen.affordances[0])
    complete = seen.affordances[0].model_copy(update={"payload": {"aim": "low"}})
    with pytest.raises(offers.NotOffered, match="not among"):
        offers.attempt(seen, complete)
    owned = seen.model_copy(update={"affordances": [complete]})
    made = offers.attempt(owned, complete)
    assert made.payload.payload == {"aim": "low"} and made.target == "7"
