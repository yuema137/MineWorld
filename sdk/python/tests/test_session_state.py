"""AP-4 and the session's error paths, with the server's frames given directly.

A `Script` stands in for the socket: it holds the frames the server would send and records what the
session sent. What these tests own is the session's own logic — pairing by token, newest-wins, and how
each error ends it. That the real server sends these frames, in this order, is AP-5's (real binary).
"""

from __future__ import annotations

import asyncio
import json
from collections.abc import Coroutine
from pathlib import Path
from typing import Any

import pytest
import support
from websockets.exceptions import ConnectionClosedError

from mineworld_sdk import offers
from mineworld_sdk.errors import MineWorldError
from mineworld_sdk.session import (
    Answered,
    ForeignObserver,
    JoinRefused,
    Outcome,
    Perceiving,
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
from mineworld_sdk.wire.frames import Invite, Perceived
from mineworld_sdk.wire.ids import ActionTypeId, EntityId, EntityKey, EventId, JsonValue

ME = EntityId("101")
WORLD: JsonValue = {
    "protocol": 2,
    "instance": "1a2b3c4d5e6f70819293a4b5c6d7e8f9",
    "at": 0,
    "time_scale": 1,
    "paused": False,
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
            "acted_through": None,
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
    """Runs one coroutine on the selector loop on every platform, within 5 s (`support.run`)."""
    return support.run(coroutine, timeout_s=5)


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


def clock(paused: bool) -> str:
    return json.dumps({"t": "clock", "at": 7, "time_scale": 1, "paused": paused})


def test_the_host_clock_is_kept_and_a_paused_submit_is_refused_not_fatal() -> None:
    # S11-D (PROTOCOL.md §5.9, §5.5): a clock frame follows welcome and every pause and resume; while
    # paused a submit is refused `paused`, which answers that request and leaves the session open.
    async def scenario() -> None:
        script = Script(welcome(), clock(False), observation(1))
        session = await joined(script)
        await session.changed()
        assert session.clock is not None and not session.clock.paused
        script.say(clock(True))
        await settle()
        assert session.clock.paused
        pending = asyncio.create_task(session.submit(request()))
        await settle()
        script.say(json.dumps({"t": "refused", "token": "c1", "code": "paused"}))
        assert await pending == RefusedRequest("paused", None)
        script.say(observation(2))
        assert (await session.changed(since=1)).seq == 2

    run(scenario())


def fact(event_id: str) -> JsonValue:
    """A fact as a `perceived` frame or `observation.events` carries it (`PROTOCOL.md` §5.2)."""
    return {
        "id": event_id,
        "at": 4,
        "event_type": "spoke",
        "subjects": ["5"],
        "participants": ["5", ME],
        "place": {"entity": "3", "entity_type": "place"},
        "caused_by": {"action": "41"},
        "payload": {"event_type": "spoke", "schema_version": 1, "payload": {"utterance": "hi"}},
        "visibility": {"place": {"entity": "3", "entity_type": "place"}},
        "provenance": {"emitted_by": "conversation", "controller_decision": "41"},
    }


def delta(seq: int, base: int, at: int) -> str:
    return json.dumps(
        {
            "t": "delta",
            "seq": seq,
            "base": base,
            "revision": None,
            "acted_through": "41",
            "delta": {"at": at, "events": [fact("9")]},
        }
    )


def test_a_delta_is_applied_to_the_observation_held_and_one_that_does_not_fit_ends_it() -> None:
    # S11-C (PROTOCOL.md §5.3): a caller only ever sees whole observations.
    async def scenario() -> None:
        script = Script(welcome(), observation(1))
        session = await joined(script)
        await session.changed()
        script.say(delta(2, 1, 9))
        seen = await session.changed(since=1)
        assert (seen.seq, seen.observation.at, seen.acted_through) == (2, 9, "41")
        assert [event.id for event in seen.observation.events] == ["9"]
        assert seen.observation.observer == ME
        script.say(delta(4, 2, 10).replace('"base": 2', '"base": 3'))
        with pytest.raises(ProtocolViolation, match="base 3"):
            await session.changed(since=2)

    run(scenario())


def perceived(through: str, *ids: str) -> str:
    return json.dumps({"t": "perceived", "through": through, "events": [fact(i) for i in ids]})


async def perceiving(
    script: Script, since: str | None = None
) -> tuple[SeatSession, list[Perceived]]:
    """A session that asked for the `perceived` stream, and the frames its sink was handed."""
    handed: list[Perceived] = []
    session = await SeatSession.join(
        script,
        seat=EntityKey("visitor"),
        invite=Invite("not-a-secret"),
        nickname="tester",
        perceiving=Perceiving(None if since is None else EventId(since), handed.append),
    )
    return session, handed


def test_perceived_frames_reach_the_sink_in_order_and_the_session_keeps_none() -> None:
    # S11-C (PROTOCOL.md §5.8): the join asks for the stream from its cursor; each frame goes to the
    # caller's sink, and the through received before an observation is recorded with it (§3.6).
    async def scenario() -> None:
        script = Script(welcome())
        session, handed = await perceiving(script, since="3")
        sent = script.sent[0]
        assert isinstance(sent, dict) and sent.get("perceived") == {"since": "3"}, sent
        script.say(perceived("12", "7", "12"))
        script.say(observation(1))
        script.say(perceived("20", "15"))
        await session.changed()
        await settle()
        assert [(f.through, [e.id for e in f.events]) for f in handed] == [
            ("12", ["7", "12"]),
            ("20", ["15"]),
        ]
        assert session.newest_through == "12"

    run(scenario())


FRAMES = Path(__file__).resolve().parents[3] / "server" / "tests" / "frames"


def golden_text(kind: str) -> str:
    """A reviewed server frame, byte for byte as `server/tests/frames.rs` checks the Rust types."""
    return (FRAMES / f"{kind}.json").read_text(encoding="utf-8")


def test_the_lagged_sequence_ends_the_session_as_lagged_not_as_a_violation() -> None:
    # AP3b-1 (F-P3b-1): `refused { lagged }` carries no token and is followed by `closing { lagged }`
    # (PROTOCOL.md §5.5, §5.8). The client rejoins and loses nothing; it never met a violation.
    async def scenario() -> MineWorldError:
        script = Script(welcome())
        session, handed = await perceiving(script)
        script.say(perceived("12", "7"))
        script.say(golden_text("refused-lagged"))
        script.say(golden_text("closing-lagged"))
        with pytest.raises(MineWorldError) as ended:
            await session.changed(since=10**6)
        await settle()
        assert script.closed, "the client closes the socket on closing (PROTOCOL.md §5.6)"
        assert [f.through for f in handed] == ["12"]
        return ended.value

    error = run(scenario())
    assert isinstance(error, SessionClosed) and error.reason == "lagged", repr(error)


class Dropped(Script):
    """A socket whose server side vanishes once its frames are read: `recv` then raises as `websockets`
    does for a connection closed without a close frame."""

    async def recv(self) -> str:
        if self.inbound.empty():
            raise ConnectionClosedError(None, None)
        return await super().recv()


def test_a_socket_dropped_between_lagged_and_its_closing_still_ends_as_lagged() -> None:
    # C2's failure case: the socket ends before `closing { lagged }` is read.
    async def scenario() -> MineWorldError:
        script = Dropped(welcome())
        session, _ = await perceiving(script)
        script.say(golden_text("refused-lagged"))
        with pytest.raises(MineWorldError) as ended:
            await session.changed(since=10**6)
        return ended.value

    error = run(scenario())
    assert isinstance(error, SessionClosed) and error.reason == "lagged", repr(error)


@pytest.mark.parametrize(
    ("frames", "wrong"),
    [
        ([perceived("12", "12", "7")], "strictly ascending"),
        ([perceived("10", "7", "10"), perceived("15", "10", "15")], "repeats a fact"),
        ([perceived("9", "7", "10")], "below its own last fact"),
        ([perceived("10", "7", "10"), perceived("8")], "below the previous"),
    ],
    ids=["descending", "duplicate", "through-below-fact", "through-goes-back"],
)
def test_a_broken_perceived_stream_fails_closed(frames: list[str], wrong: str) -> None:
    # AP3b-2 (design §4.4, checks 1-3): the session ends ProtocolViolation and hands the sink nothing
    # from the bad frame on. Through ResumingSeat that is SeatLost("protocol_violation") (AP3b-5 row 11).
    async def scenario() -> tuple[MineWorldError, list[Perceived]]:
        script = Script(welcome())
        session, handed = await perceiving(script)
        for frame in frames:
            script.say(frame)
        with pytest.raises(MineWorldError) as ended:
            await session.changed(since=10**6)
        return ended.value, handed

    error, handed = run(scenario())
    assert isinstance(error, ProtocolViolation) and wrong in str(error), repr(error)
    good = [json.loads(frame)["through"] for frame in frames[:-1]]
    assert [f.through for f in handed] == good, "nothing past the bad frame reached the sink"


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


def test_an_occupied_seat_is_refused_and_the_client_closes_its_socket() -> None:
    # S11-B: the server does not follow seat_occupied with closing; the connection stays, so the
    # session must close it itself rather than wait for a closing that never comes.
    script = Script(json.dumps({"t": "refused", "code": "seat_occupied"}))
    with pytest.raises(JoinRefused) as refused:
        run(joined(script))
    assert refused.value.code == "seat_occupied"
    assert script.closed


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
        events=[],
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
