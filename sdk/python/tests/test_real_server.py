"""CP-P3: Python is a client of the real server, end to end (AP-2, AP-5, AP-6, AP-7, AP-11).

These tests may name `talk`, `move` and `conversation-history`: they are fixtures of the social-cafe
and market-town World Packs. The SDK itself names none of them (D-P3-2).
"""

from __future__ import annotations

import asyncio
import json
import logging
import math
import time
from collections.abc import Callable, Coroutine
from itertools import pairwise
from typing import Any

import pytest
from realserver import INVITE, Recording, Server, first_difference, seated
from websockets.asyncio.client import connect as websocket_connect

from mineworld_sdk import offers
from mineworld_sdk.session import Answered, JoinRefused, SeatSession, SessionClosed
from mineworld_sdk.wire import codec
from mineworld_sdk.wire.contract import Accepted, Observation
from mineworld_sdk.wire.frames import Invite, ObservationFrame
from mineworld_sdk.wire.ids import ActionTypeId, EntityId, EntityKey, JsonValue

TALK = ActionTypeId("talk")
MOVE = ActionTypeId("move")
SHOOT = ActionTypeId("shoot")
STRIDE_MM = 2_000
"""The longest stride the server accepts (`server/PROTOCOL.md` §6.2), a literal, as the Rust tests keep it."""
IN_REACH_MM = 1_800
"""How far short of Alice the visitor stops: inside the 3 m `talk` range the pack offers."""
CONVERSATION_BOUND_S = 10.0
"""AP-5: a line said, heard, answered, and the answer disclosed, within ten wall seconds."""
PATIENCE_S = 20.0
COVERAGE = {"welcome": 1, "observation": 50, "result": 2}
"""AP-2's minimum frames per session, fixed before anything was measured."""

pytestmark = pytest.mark.real_server


def run[T](coroutine: Coroutine[Any, Any, T]) -> T:
    return asyncio.run(asyncio.wait_for(coroutine, 120))


async def until(
    session: SeatSession,
    what: str,
    ready: Callable[[ObservationFrame], bool],
    within: float = PATIENCE_S,
) -> ObservationFrame:
    """The first observation satisfying `ready`, waited for on the stream, or a failure naming `what`."""

    async def watch() -> ObservationFrame:
        since: int | None = None
        while True:
            frame = await session.changed(since)
            if ready(frame):
                return frame
            since = frame.seq

    try:
        return await asyncio.wait_for(watch(), within)
    except TimeoutError:
        pytest.fail(f"no observation with {what} arrived within {within:.0f} s")


def heard(observation: Observation, listener: EntityId) -> list[tuple[str, str]]:
    """(speaker, utterance) for each line `listener`'s own `conversation-history` discloses."""
    for entity in observation.entities:
        if entity.id != listener:
            continue
        for record in entity.components:
            if record.component_type != "conversation-history":
                continue
            payload = record.payload
            entries = payload.get("heard") if isinstance(payload, dict) else None
            lines: list[tuple[str, str]] = []
            for entry in entries if isinstance(entries, list) else []:
                if isinstance(entry, dict):
                    speaker, said = entry.get("speaker"), entry.get("utterance")
                    if isinstance(speaker, dict) and isinstance(said, str):
                        lines.append((str(speaker.get("entity")), said))
            return lines
    return []


def position(observation: Observation) -> tuple[str, int, int]:
    where = observation.self_location
    assert where is not None and where.local is not None, "a seat in social-cafe has a position"
    return where.place.entity, where.local.x, where.local.y


def strides(start: tuple[int, int], end: tuple[int, int]) -> list[tuple[int, int]]:
    """The fewest equal strides of at most `STRIDE_MM` from `start` to `end`, integer arithmetic."""
    (x0, y0), (x1, y1) = start, end
    count = max(1, math.ceil(math.hypot(x1 - x0, y1 - y0) / STRIDE_MM))
    while True:
        points = [
            (x0 + (x1 - x0) * k // count, y0 + (y1 - y0) * k // count) for k in range(count + 1)
        ]
        if all(math.dist(a, b) <= STRIDE_MM for a, b in pairwise(points)):
            return points[1:]
        count += 1


def move_to(place: str, x: int, y: int) -> JsonValue:
    return {
        "to": {
            "place": {"entity": place, "entity_type": "place"},
            "local": {"x": x, "y": y, "z": 0},
            "facing": None,
        }
    }


async def walk_into_reach(visitor: SeatSession, alice: SeatSession) -> None:
    """Walks the visitor to within `IN_REACH_MM` of Alice, every stride offered and accepted."""
    mine = (await visitor.changed()).observation
    place, x, y = position(mine)
    alice_place, ax, ay = position((await alice.changed()).observation)
    assert place == alice_place, "the visitor and Alice start in one place"
    distance = math.hypot(ax - x, ay - y)
    goal = (
        ax - round((ax - x) * IN_REACH_MM / distance),
        ay - round((ay - y) * IN_REACH_MM / distance),
    )
    for step_x, step_y in strides((x, y), goal):
        request = offers.request(
            (await visitor.changed()).observation,
            MOVE,
            target=None,
            payload=move_to(place, step_x, step_y),
        )
        outcome = await visitor.submit(request)
        assert isinstance(outcome, Answered) and isinstance(outcome.result, Accepted), (
            f"a stride to ({step_x}, {step_y}) was not accepted: {outcome}"
        )


async def answer_new_lines(alice: SeatSession, answered: int) -> int:
    """Alice's echo: answers every line her own history shows beyond the first `answered`."""
    frame = await until(
        alice,
        "a new line in Alice's history",
        lambda f: len(heard(f.observation, alice.observer)) > answered,
    )
    for speaker, said in heard(frame.observation, alice.observer)[answered:]:
        request = offers.request(
            frame.observation,
            TALK,
            target=EntityId(speaker),
            payload={"utterance": f"You said: {said}"},
        )
        outcome = await alice.submit(request)
        assert isinstance(outcome, Answered) and isinstance(outcome.result, Accepted), (
            f"alice's reply was not accepted: {outcome}"
        )
        answered += 1
    return answered


def round_trips(recording: Recording, observer: EntityId) -> None:
    """AP-2: every frame received re-encodes to itself; INV-13: every observation is the seat's own."""
    for raw in recording.received:
        expected: JsonValue = json.loads(raw)
        frame = codec.decode(raw)
        difference = first_difference(expected, codec.to_json(frame))
        assert difference is None, f"a received {frame.t} frame does not round-trip at {difference}"
        if isinstance(frame, ObservationFrame):
            assert frame.observation.observer == observer, "a frame named another observer"


def covers_the_minimum(recording: Recording, who: str) -> None:
    counts = {
        kind: recording.count(kind) for kind in ("welcome", "observation", "result", "refused")
    }
    print(f"AP-2 coverage, {who}: {counts}, {len(recording.received)} frames round-tripped")
    for kind, least in COVERAGE.items():
        assert recording.count(kind) >= least, (
            f"{who} received {recording.count(kind)} {kind} frames"
        )


async def observations_at_least(session: SeatSession, recording: Recording, least: int) -> None:
    await until(session, f"{least} observations", lambda _: recording.count("observation") >= least)


def test_two_seats_hold_one_conversation_and_every_frame_round_trips(
    mineworld: Callable[[str], Server],
) -> None:
    """AP-5, AP-2 (social-cafe), and the world's half of AP-6."""
    server = mineworld("social-cafe")

    async def scenario() -> list[ObservationFrame]:
        visitor, visitor_frames = await seated(server, "visitor")
        alice, alice_frames = await seated(server, "alice")
        async with visitor, alice:
            await walk_into_reach(visitor, alice)
            await until(
                visitor,
                "talk to alice available",
                lambda f: any(
                    a.action_type == TALK and a.target == alice.observer and a.available
                    for a in f.observation.affordances
                ),
            )
            # S11-D: the server sent its clock right after welcome, and the world is running.
            for session in (visitor, alice):
                assert session.clock is not None and not session.clock.paused, session.clock
            answered = 0
            for line in ("Good morning.", "Is the coffee fresh?"):
                began = time.monotonic()
                said = offers.request(
                    (await visitor.changed()).observation,
                    TALK,
                    target=alice.observer,
                    payload={"utterance": line},
                )
                outcome = await visitor.submit(said)
                assert isinstance(outcome, Answered) and isinstance(outcome.result, Accepted), (
                    outcome
                )
                assert outcome.result.accepted.events, "an accepted talk causes at least one event"
                accepted = time.monotonic() - began
                answered = await answer_new_lines(alice, answered)
                replied = time.monotonic() - began
                reply = f"You said: {line}"
                disclosed = await until(
                    visitor,
                    f"Alice's reply {reply!r} disclosed to the visitor",
                    lambda f, reply=reply: (
                        (alice.observer, reply) in heard(f.observation, visitor.observer)
                    ),
                )
                elapsed = time.monotonic() - began
                print(
                    f"AP-5 exchange {line!r}: accepted {accepted * 1000:.0f} ms, answered "
                    f"{replied * 1000:.0f} ms, disclosed {elapsed * 1000:.0f} ms (seq {disclosed.seq}); "
                    f"visitor heard {heard(disclosed.observation, visitor.observer)}"
                )
                assert elapsed <= CONVERSATION_BOUND_S, f"one exchange took {elapsed:.1f} s"

            # AP-6, the world's half: the same request offers refuses is answered `unavailable`.
            seen = (await visitor.changed()).observation
            sent = visitor.sent_frames
            with pytest.raises(offers.NotOffered):
                offers.request(seen, SHOOT, target=alice.observer, payload={})
            assert visitor.sent_frames == sent, "a request offers refused sent no frame"
            raw = offers.request(seen, TALK, target=alice.observer, payload={})
            shoot = raw.model_copy(
                update={
                    "action_type": SHOOT,
                    "payload": raw.payload.model_copy(update={"action_type": SHOOT}),
                }
            )
            outcome = await visitor.submit(shoot)
            assert isinstance(outcome, Answered) and outcome.result == "unavailable", outcome

            await observations_at_least(visitor, visitor_frames, COVERAGE["observation"])
            await observations_at_least(alice, alice_frames, COVERAGE["observation"])
        for recording, session, who in (
            (visitor_frames, visitor, "visitor"),
            (alice_frames, alice, "alice"),
        ):
            covers_the_minimum(recording, who)
            round_trips(recording, session.observer)
        return [
            frame
            for raw in visitor_frames.received + alice_frames.received
            if isinstance(frame := codec.decode(raw), ObservationFrame)
        ]

    observed = [frame.observation for frame in run(scenario())]
    assert any(entity.components for o in observed for entity in o.entities), (
        "an entity with a component"
    )
    assert any(o.affordances for o in observed), "an affordance with its requirement"
    assert any(o.relations for o in observed), "a non-empty relations list"


def test_a_complete_affordance_is_attempted_unchanged_and_round_trips(
    mineworld: Callable[[str], Server],
) -> None:
    """AP-2 (market-town): bob starts in the café, a shop, and is offered complete `buy` affordances
    (`systems/economy/src/offer.rs`); he attempts two, unchanged."""
    server = mineworld("market-town")

    async def scenario() -> list[ObservationFrame]:
        bob, frames = await seated(server, "bob")
        async with bob:
            for _ in range(2):
                frame = await until(
                    bob,
                    "an available complete affordance",
                    lambda f: any(
                        a.available and a.payload is not None for a in f.observation.affordances
                    ),
                )
                offer = next(
                    a
                    for a in frame.observation.affordances
                    if a.available and a.payload is not None
                )
                outcome = await bob.submit(offers.attempt(frame.observation, offer))
                assert isinstance(outcome, Answered), outcome
            await observations_at_least(bob, frames, COVERAGE["observation"])
        covers_the_minimum(frames, "bob")
        round_trips(frames, bob.observer)
        return [
            frame
            for raw in frames.received
            if isinstance(frame := codec.decode(raw), ObservationFrame)
        ]

    observed = run(scenario())
    assert any(
        affordance.payload is not None
        for frame in observed
        for affordance in frame.observation.affordances
    ), "an observation with a complete affordance"


def test_an_occupied_seat_is_refused_unless_taken_over(mineworld: Callable[[str], Server]) -> None:
    """C6 (S11-B): a seat held by one connection is `seat_occupied` to a second, and `take_over`
    takes it — the first connection is told `closing { taken_over }`."""
    server = mineworld("social-cafe")

    async def scenario() -> None:
        first, _ = await seated(server, "visitor")
        with pytest.raises(JoinRefused) as refused:
            await seated(server, "visitor")
        assert refused.value.code == "seat_occupied"
        second = await SeatSession.connect(
            server.url,
            seat=EntityKey("visitor"),
            invite=Invite(INVITE),
            nickname="taker",
            take_over=True,
        )
        async with second:
            assert second.welcome.took_over == "connection"
            assert second.observer == first.observer
            with pytest.raises(SessionClosed) as closed:
                await first.changed(since=10**9)
            assert closed.value.reason == "taken_over"

    run(scenario())


def test_a_wrong_invite_is_refused_late_and_never_repeated(
    mineworld: Callable[[str], Server], caplog: pytest.LogCaptureFixture
) -> None:
    """AP-7: refused `unauthorized` no sooner than 500 ms; neither invite appears in any log record,
    exception or session `repr`."""
    server = mineworld("social-cafe")
    offered = "wrong-invite-0123456789abcdefghi"
    assert len(offered) == 32
    caplog.set_level(logging.DEBUG)

    async def scenario() -> tuple[float, JoinRefused, str, Recording, SeatSession, Recording]:
        refused_frames = Recording(await websocket_connect(server.url, compression=None))
        began = time.monotonic()
        with pytest.raises(JoinRefused) as refused:
            await SeatSession.join(
                refused_frames,
                seat=EntityKey("visitor"),
                invite=Invite(offered),
                nickname="intruder",
            )
        elapsed = time.monotonic() - began
        visitor, frames = await seated(server, "visitor")
        async with visitor:
            shown = repr(visitor) + str(visitor)
        return elapsed, refused.value, shown, refused_frames, visitor, frames

    elapsed, refusal, shown, refused_frames, visitor, frames = run(scenario())
    assert refusal.code == "unauthorized"
    assert elapsed >= 0.5, f"refused after {elapsed:.3f} s, before the server's fixed delay"
    exposed = [
        str(refusal),
        repr(refusal),
        shown,
        *(record.getMessage() for record in caplog.records),
    ]
    for secret in (offered, INVITE):
        assert not any(secret in text for text in exposed), "an invite leaked"
    # The refusal carries no token, and `closing` no detail: both omissions round-trip (M-3's owner).
    assert [json.loads(raw)["t"] for raw in refused_frames.received] == ["refused", "closing"]
    round_trips(refused_frames, visitor.observer)
    round_trips(frames, visitor.observer)
