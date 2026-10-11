"""The resuming seat under scripted connections: §4.5's table, backoff, observations and submits across
connections, and the seat-level halves of AP3b-1, AP3b-3 and AP3b-4.

What these tests own is the seat's decisions: which outcome rejoins, with which `join` frame, and which
ends the seat. That the real server sends these frames, and that a resumed stream equals the offline
export, is `test_real_resume.py`'s (real binary). Every test checks that no task outlives the seat.
"""

from __future__ import annotations

import asyncio
import logging
import re
from collections.abc import Callable, Coroutine
from pathlib import Path
from typing import Any

import pytest
import support
from scripted import (
    INVITE,
    FakeClock,
    Gate,
    Link,
    ScriptedServer,
    closing,
    delta,
    golden_text,
    observation,
    perceived,
    refused,
    result,
    secret,
    settle,
    until,
    welcome,
)

from mineworld_sdk import (
    AnswerLost,
    CursorCell,
    CursorSource,
    NotConnected,
    PerceivedBatch,
    ReconnectPolicy,
    ResumingSeat,
    SeatLost,
    SessionClosed,
)
from mineworld_sdk.wire.contract import ActionRecord, ActionRequest
from mineworld_sdk.wire.frames import Invite
from mineworld_sdk.wire.ids import ActionTypeId, EntityId, EntityKey, ResumeSecret

SECRETS = [INVITE, *(secret(n) for n in range(1, 8))]
SOURCE = Path(__file__).resolve().parents[1] / "src" / "mineworld_sdk"


def run[T](scenario: Callable[[], Coroutine[Any, Any, T]]) -> T:
    """Runs a scenario on the selector loop within 5 s, and fails if any task outlives it."""

    async def checked() -> T:
        before = asyncio.all_tasks()
        value = await scenario()
        await settle()
        leaked = asyncio.all_tasks() - before
        assert not leaked, f"tasks outlived the seat: {leaked}"
        return value

    return support.run(checked(), timeout_s=5)


async def opened(
    server: ScriptedServer,
    clock: FakeClock,
    *,
    cursor: CursorSource | None = None,
    policy: ReconnectPolicy | None = None,
    resume: str | None = None,
) -> ResumingSeat:
    return await ResumingSeat.open(
        server,
        seat=EntityKey("wanderer"),
        invite=Invite(INVITE),
        nickname="tester",
        cursor=cursor,
        resume=None if resume is None else ResumeSecret(resume),
        policy=policy or ReconnectPolicy(),
        sleep=clock.sleep,
        clock=clock.time,
    )


def no_secret(*things: object) -> None:
    """AP3b-17: neither the invite nor any resume secret in any `str` or `repr`."""
    for thing in things:
        for text in (str(thing), repr(thing)):
            for hidden in SECRETS:
                assert hidden not in text, f"a secret leaked through {type(thing).__name__}"


def request() -> ActionRequest:
    ring = ActionTypeId("ring")
    return ActionRequest(
        actor=EntityId("101"), action_type=ring, payload=ActionRecord(action_type=ring, payload={})
    )


def ids(batches: list[PerceivedBatch]) -> list[str]:
    return [event.id for batch in batches for event in batch.events]


# ── AP3b-1, AP3b-3, AP3b-4 at the seat ────────────────────────────────────────────────────────────


def test_after_lagged_the_seat_rejoins_with_resume_and_the_store_cursor() -> None:
    # AP3b-1: the golden lagged sequence; the next connection's only frame before its welcome is a
    # join carrying the first welcome's resume and the cell's value, never take_over.
    async def scenario() -> None:
        one = Link(welcome(resume=1), perceived("12", "7", "12"))
        two = Link(welcome(resume=2, took_over="held"), observation(1))
        cell = CursorCell()
        seat = await opened(ScriptedServer(one, two), FakeClock(), cursor=cell)
        cell.commit(await anext(seat.perceived()))
        one.say(golden_text("refused-lagged"), golden_text("closing-lagged"))
        seen = await seat.changed()
        assert seen.connection == 2 and seat.welcome.took_over == "held"
        assert two.kinds() == ["join"]
        join = two.join()
        assert join["resume"] == secret(1) and join["take_over"] is False
        assert join["perceived"] == {"since": "12"}
        assert one.kinds() == ["join"], "a lagged connection is not left: the seat is held"
        await seat.leave()
        assert two.kinds() == ["join", "leave"]

    run(scenario)


def test_the_store_not_the_receipt_decides_where_a_rejoin_resumes() -> None:
    # AP3b-3 through the seat: A and B taken, only A committed; the rejoin's since is A's through,
    # and the re-sent 12 and 15 are not delivered twice (22 for the design's 18: DV-P3b-4).
    async def scenario() -> list[PerceivedBatch]:
        one = Link(welcome(resume=1), perceived("10", "7", "10"), perceived("20", "12", "15"))
        two = Link(welcome(resume=2, took_over="held"), perceived("25", "12", "15", "22"))
        cell = CursorCell()
        seat = await opened(ScriptedServer(one, two), FakeClock(), cursor=cell)
        stream = seat.perceived()
        batches = [await anext(stream), await anext(stream)]
        cell.commit(batches[0])
        one.drop()
        batches.append(await anext(stream))
        assert two.join()["perceived"] == {"since": "10"}, "the rejoin presents the store's cursor"
        await seat.leave()
        return batches

    batches = run(scenario)
    assert ids(batches) == ["7", "10", "12", "15", "22"]
    assert [batch.connection for batch in batches] == [1, 1, 2]


def test_a_store_behind_what_was_delivered_never_sees_a_fact_twice() -> None:
    # F-P3b-R1 (P3b-1, D-P3b-3; P4's R-P3b-1): facts are delivered up to 15, the store commits only
    # up to 10, the socket drops. The rejoin presents 10, so the server re-sends 12 and 15 — once in
    # a frame whose through is exactly the delivered 15 (every fact a duplicate), then new facts. The
    # consumer receives only facts after 15, once, in order, and the all-duplicate frame is no batch.
    async def scenario() -> list[PerceivedBatch]:
        one = Link(welcome(resume=1), perceived("10", "7", "10"), perceived("15", "12", "15"))
        two = Link(
            welcome(resume=2, took_over="held"),
            perceived("15", "12", "15"),
            perceived("20", "18", "20"),
        )
        cell = CursorCell()
        seat = await opened(ScriptedServer(one, two), FakeClock(), cursor=cell)
        stream = seat.perceived()
        batches = [await anext(stream), await anext(stream)]
        cell.commit(batches[0])
        one.drop()
        batches.append(await anext(stream))
        assert two.join()["perceived"] == {"since": "10"}, "the rejoin presents the store's cursor"
        await seat.leave()
        batches.extend([batch async for batch in stream])
        return batches

    batches = run(scenario)
    assert [(b.through, [e.id for e in b.events], b.connection) for b in batches] == [
        ("10", ["7", "10"], 1),
        ("15", ["12", "15"], 1),
        ("20", ["18", "20"], 2),
    ], "after the rejoin: only facts past the delivered 15, and no batch for the duplicate frame"


def test_a_consumer_that_falls_behind_costs_a_reconnect_never_a_fact() -> None:
    # AP3b-4: buffer 4; six facts in three frames while nothing is read. The client sends no leave,
    # closes, and rejoins with resume and the cell's cursor; drained and committed, nothing is missing.
    async def scenario() -> list[PerceivedBatch]:
        frames = [perceived("8", "7", "8"), perceived("10", "9", "10"), perceived("12", "11", "12")]
        one = Link(welcome(resume=1))
        two = Link(welcome(resume=2, took_over="held"))
        server = ScriptedServer(one, two)
        cell = CursorCell()
        seat = await opened(
            server, FakeClock(), cursor=cell, policy=ReconnectPolicy(perceived_buffer=4)
        )
        stream = seat.perceived()
        one.say(*frames)
        await until(lambda: seat.connection == 2, "a rejoin after the local lag")
        assert one.kinds() == ["join"] and one.closed, "closed without leave"
        assert two.join()["resume"] == secret(1) and two.join()["perceived"] == {"since": None}
        delivered: list[PerceivedBatch] = []
        for frame in frames:
            two.say(frame)
            batch = await anext(stream)
            cell.commit(batch)
            delivered.append(batch)
        await seat.leave()
        return delivered

    assert ids(run(scenario)) == ["7", "8", "9", "10", "11", "12"]


# ── AP3b-5: the decision table, row by row ────────────────────────────────────────────────────────


def test_row_1_leave_ends_the_stream_and_changed() -> None:
    async def scenario() -> None:
        one = Link(welcome(), perceived("10", "7", "10"))
        server = ScriptedServer(one)
        seat = await opened(server, FakeClock(), cursor=CursorCell())
        stream = seat.perceived()
        await seat.leave()
        assert ids([batch async for batch in stream]) == ["7", "10"]
        with pytest.raises(SessionClosed) as closed:
            await seat.changed()
        assert closed.value.reason == "left"
        assert one.kinds() == ["join", "leave"] and server.attempts == 1

    run(scenario)


def test_row_2_detach_closes_without_leave_and_never_rejoins() -> None:
    async def scenario() -> None:
        one = Link(welcome())
        server = ScriptedServer(one)
        seat = await opened(server, FakeClock(), cursor=CursorCell())
        await seat.detach()
        assert [batch async for batch in seat.perceived()] == []
        assert one.kinds() == ["join"] and one.closed and server.attempts == 1

    run(scenario)


@pytest.mark.parametrize("how", ["leave", "detach"])
def test_leaving_while_a_rejoin_is_in_flight_stops_it_cleanly(how: str) -> None:
    # C4 review: leave and detach race a rejoin waiting on its connect; neither leaks a task, and the
    # connection that arrives late is never joined.
    async def scenario() -> None:
        one = Link(welcome(resume=1))
        gate = Gate(Link(welcome(resume=2)))
        seat = await opened(ScriptedServer(one, gate), FakeClock(), cursor=CursorCell())
        one.drop()
        await until(lambda: seat.connection == 1 and one.closed, "the loss")
        await settle()
        await (seat.leave() if how == "leave" else seat.detach())
        gate.opened.set()
        await settle()
        assert gate.link.sent == [], "no join after the seat was given up"
        with pytest.raises(SessionClosed):
            await seat.changed()

    run(scenario)


@pytest.mark.parametrize("reason", ["taken_over", "superseded", "kicked", "world_stopped"])
def test_row_3_a_final_closing_ends_the_seat(reason: str) -> None:
    async def scenario() -> None:
        one = Link(welcome())
        server = ScriptedServer(one, Link(welcome(resume=2)))
        seat = await opened(server, FakeClock(), cursor=CursorCell())
        pending = asyncio.create_task(seat.submit(request()))
        await settle()
        one.say(closing(reason))
        with pytest.raises(SeatLost) as lost:
            await seat.changed()
        assert lost.value.reason == reason and server.attempts == 1
        with pytest.raises(SeatLost):
            await pending
        with pytest.raises(SeatLost):
            await anext(seat.perceived())
        no_secret(lost.value, seat)

    run(scenario)


def held_rejoin(hold: int, first_delay: float, *, ending: str) -> tuple[list[str], dict[str, Any]]:
    """Row 6: the connection ends (`ending`), the first attempt fails to connect, the second (after
    `first_delay`, no jitter) is welcomed. Returns the sleeps and the rejoin's join frame."""

    async def scenario() -> tuple[list[str], dict[str, Any]]:
        one = Link(welcome(resume=1, hold=hold))
        two = Link(welcome(resume=2, took_over="held"))
        clock = FakeClock()
        policy = ReconnectPolicy(first_delay=first_delay, factor=1, ceiling=first_delay, jitter=0)
        seat = await opened(
            ScriptedServer(one, OSError("refused"), two), clock, cursor=CursorCell(), policy=policy
        )
        if ending == "drop":
            one.drop()
        else:
            one.say(closing(ending))
        await until(lambda: seat.connection == 2, "the rejoin")
        await seat.leave()
        return [f"{s:g}" for s in clock.sleeps], two.join()

    return run(scenario)


@pytest.mark.parametrize("ending", ["drop", "server_stopping"])
def test_row_6_a_rejoin_carries_resume_only_while_the_hold_lasts(ending: str) -> None:
    sleeps, join = held_rejoin(10, 9, ending=ending)
    assert sleeps == ["0", "9"] and join["resume"] == secret(1), "at loss + hold - 1 s"
    sleeps, join = held_rejoin(10, 11, ending=ending)
    assert sleeps == ["0", "11"] and join["resume"] is None, "at loss + hold + 1 s"
    _, join = held_rejoin(0, 0.5, ending=ending)
    assert join["resume"] is None, "a server that holds nothing is never shown a resume"
    assert join["take_over"] is False and join["perceived"] == {"since": None}


def test_row_7_a_refused_resume_is_retried_at_once_without_it() -> None:
    async def scenario() -> None:
        one = Link(welcome(resume=1))
        two = Link(refused("invalid_resume"))
        three = Link(welcome(resume=3))
        clock = FakeClock()
        seat = await opened(ScriptedServer(one, two, three), clock, cursor=CursorCell())
        one.drop()
        await until(lambda: seat.connection == 2, "the fresh join")
        assert two.join()["resume"] == secret(1) and two.closed
        assert three.join()["resume"] is None
        assert clock.sleeps == [0.0], "no backoff before the retry without resume"
        await seat.leave()

    run(scenario)


def test_row_8_cursor_unavailable_is_final_and_no_other_cursor_is_tried() -> None:
    # P3b-3: never answered by rejoining with since: null or a newer cursor.
    async def scenario() -> None:
        one = Link(welcome(resume=1))
        server = ScriptedServer(one, Link(refused("cursor_unavailable")), Link(welcome(resume=3)))
        cell = CursorCell()
        seat = await opened(server, FakeClock(), cursor=cell)
        one.say(perceived("10", "7", "10"))
        cell.commit(await anext(seat.perceived()))
        one.drop()
        try:
            seen = await asyncio.wait_for(seat.changed(), 1)
        except SeatLost as lost:
            assert lost.reason == "cursor_unavailable", lost
            no_secret(lost, seat)
        except TimeoutError:
            await seat.leave()
            pytest.fail("P3b-3: cursor_unavailable was answered with another join, not SeatLost")
        else:
            await seat.leave()
            pytest.fail(f"P3b-3: cursor_unavailable was answered by a rejoin ({seen.connection})")
        assert server.attempts == 2, "P3b-3: a refused cursor is never followed by another join"

    run(scenario)


@pytest.mark.parametrize(
    ("answer", "reason"),
    [
        ([refused("seat_occupied")], "seat_occupied"),
        ([refused("unknown_seat")], "unknown_seat"),
        ([refused("seat_not_in_world")], "seat_not_in_world"),
        ([refused("invalid_nickname")], "invalid_nickname"),
        ([refused("unauthorized"), closing("unauthorized")], "unauthorized"),
        ([refused("protocol_mismatch"), closing("protocol_mismatch")], "protocol_mismatch"),
        ([welcome(resume=2, protocol=3)], "protocol_mismatch"),
    ],
    ids=["occupied", "unknown", "not-in-world", "nickname", "invite-rotated", "refused", "welcome"],
)
def test_row_9_a_final_refusal_of_a_rejoin_ends_the_seat(answer: list[str], reason: str) -> None:
    async def scenario() -> None:
        one = Link(welcome(resume=1))
        server = ScriptedServer(one, Link(*answer))
        seat = await opened(server, FakeClock(), cursor=CursorCell())
        one.drop()
        with pytest.raises(SeatLost) as lost:
            await seat.changed()
        assert lost.value.reason == reason and server.attempts == 2
        no_secret(lost.value, seat)

    run(scenario)


def test_row_9_a_final_refusal_of_the_first_join_raises_from_open() -> None:
    async def scenario() -> None:
        with pytest.raises(SeatLost) as lost:
            await opened(ScriptedServer(Link(refused("seat_occupied"))), FakeClock())
        assert lost.value.reason == "seat_occupied"

    run(scenario)


def test_row_10_another_world_instance_is_left_and_ends_the_seat() -> None:
    async def scenario() -> None:
        one = Link(welcome(resume=1), perceived("10", "7", "10"))
        other = "0" * 31 + "1"
        two = Link(welcome(resume=2, instance=other), perceived("30", "28"))
        seat = await opened(ScriptedServer(one, two), FakeClock(), cursor=CursorCell())
        stream = seat.perceived()
        assert ids([await anext(stream)]) == ["7", "10"]
        one.drop()
        with pytest.raises(SeatLost) as lost:
            await seat.changed()
        assert lost.value.reason == "world_changed"
        assert two.kinds() == ["join", "leave"], "the other world's seat is given back"
        assert two.join()["perceived"] == {"since": None}
        with pytest.raises(SeatLost):
            await anext(stream)  # no fact of the other world reached the consumer

    run(scenario)


@pytest.mark.parametrize("where", ["frame", "welcome"])
def test_row_11_a_violation_on_any_connection_ends_the_seat(where: str) -> None:
    async def scenario() -> None:
        one = Link(welcome(resume=1))
        two = Link(welcome(resume=2, observer="102"))
        server = ScriptedServer(one, two)
        seat = await opened(server, FakeClock(), cursor=CursorCell())
        pending = asyncio.create_task(seat.submit(request()))
        await settle()
        if where == "frame":
            one.say(result("c99", "1"))
        else:
            one.drop()
        with pytest.raises(SeatLost) as lost:
            await seat.changed()
        assert lost.value.reason == "protocol_violation" and lost.value.cause is not None
        with pytest.raises(AnswerLost):
            await pending
        assert server.attempts == (1 if where == "frame" else 2)
        no_secret(lost.value, seat)

    run(scenario)


# ── AP3b-6: seeded, bounded backoff (row 12) ──────────────────────────────────────────────────────


def gave_up_after(seed: int) -> list[float]:
    async def scenario() -> list[float]:
        one = Link(welcome(resume=1))
        clock = FakeClock()
        policy = ReconnectPolicy(seed=seed, give_up_after=30)
        seat = await opened(ScriptedServer(one), clock, cursor=CursorCell(), policy=policy)
        one.drop()
        with pytest.raises(SeatLost) as lost:
            await seat.changed()
        assert lost.value.reason == "gave_up"
        no_secret(lost.value, seat, policy)
        return clock.sleeps

    return run(scenario)


def test_backoff_is_seeded_capped_and_gives_up_in_time() -> None:
    first, again, other = gave_up_after(7), gave_up_after(7), gave_up_after(8)
    assert first == again, "one seed, one sequence"
    assert first != other, "another seed, another sequence"
    assert first[0] == 0, "the first attempt after a loss is immediate"
    assert all(delay <= ReconnectPolicy().ceiling for delay in first)
    assert sum(first) <= 30
    print(f"AP3b-6 seed 7: {len(first)} sleeps, total {sum(first):.2f} s")


# ── AP3b-7, AP3b-8: observations and submits across connections ───────────────────────────────────


def test_observations_are_ordered_by_connection_then_seq() -> None:
    async def scenario() -> None:
        one = Link(welcome(resume=1), observation(1), *(delta(seq) for seq in range(2, 41)))
        two = Link(welcome(resume=2, took_over="held"), perceived("5"), observation(1))
        seat = await opened(ScriptedServer(one, two), FakeClock(), cursor=CursorCell())
        last = await seat.changed()
        while last.frame.seq < 40:
            last = await seat.changed(last)
        assert (last.connection, last.frame.seq) == (1, 40)
        one.drop()
        seen = await seat.changed(last)
        assert (seen.connection, seen.frame.seq) == (2, 1)
        assert seen.frame.acted_through is None
        assert seen.perceived_through == "5"
        no_secret(seen)
        await seat.leave()

    run(scenario)


def test_a_submit_is_sent_once_and_never_while_reconnecting() -> None:
    async def scenario() -> None:
        one = Link(welcome(resume=1))
        two = Link(welcome(resume=2, took_over="held"))
        gate = Gate(two)
        seat = await opened(ScriptedServer(one, gate), FakeClock(), cursor=None)
        pending = asyncio.create_task(seat.submit(request()))
        await settle()
        one.drop()
        with pytest.raises(AnswerLost) as lost:
            await pending
        with pytest.raises(NotConnected) as waiting:
            await seat.submit(request())
        no_secret(lost.value, waiting.value)
        gate.opened.set()
        await until(lambda: seat.connection == 2, "the rejoin")
        answered = asyncio.create_task(seat.submit(request()))
        await settle()
        two.say(result("c1", "77"))
        outcome = await answered
        assert getattr(outcome, "action_id", None) == "77"
        assert one.kinds() == ["join", "submit"]
        assert two.kinds() == ["join", "submit"], "nothing pending was re-sent"
        await seat.leave()

    run(scenario)


# ── The surface ───────────────────────────────────────────────────────────────────────────────────


def test_the_stream_has_one_consumer_and_a_seat_without_a_cursor_has_none() -> None:
    async def scenario() -> None:
        with_cursor = await opened(
            ScriptedServer(Link(welcome())), FakeClock(), cursor=CursorCell()
        )
        with_cursor.perceived()
        with pytest.raises(RuntimeError):
            with_cursor.perceived()
        await with_cursor.leave()
        one = Link(welcome())
        without = await opened(ScriptedServer(one), FakeClock(), cursor=None)
        assert "perceived" not in one.join()
        with pytest.raises(RuntimeError):
            without.perceived()
        await without.leave()

    run(scenario)


def test_an_unknown_connector_error_reaches_the_caller_unchanged() -> None:
    class Unexpected(Exception):
        pass

    async def scenario() -> None:
        one = Link(welcome())
        seat = await opened(ScriptedServer(one, Unexpected("boom")), FakeClock())
        one.drop()
        with pytest.raises(Unexpected):
            await seat.changed()

    run(scenario)


def test_nothing_is_logged(caplog: pytest.LogCaptureFixture) -> None:
    # AP3b-17's log half: the SDK logs nothing at all, so no secret can reach a log record.
    caplog.set_level(logging.DEBUG)
    test_row_7_a_refused_resume_is_retried_at_once_without_it()
    test_row_8_cursor_unavailable_is_final_and_no_other_cursor_is_tried()
    gave_up_after(7)
    assert not [r for r in caplog.records if r.name.startswith("mineworld")], caplog.records
    assert not any(s in r.getMessage() for r in caplog.records for s in SECRETS)


def test_the_sdk_uses_only_primitives_every_event_loop_has() -> None:
    # AP3b-13 (P3b-5, D-P3b-10): no loop-specific call, no loop class, no signals, no Unix sockets.
    forbidden = re.compile(
        r"add_reader|add_writer|add_signal_handler|set_event_loop_policy|SelectorEventLoop"
        r"|ProactorEventLoop|import signal|AF_UNIX"
    )
    found = [
        f"{path.relative_to(SOURCE)}:{number}: {line.strip()}"
        for path in sorted(SOURCE.rglob("*.py"))
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1)
        if forbidden.search(line)
    ]
    assert not found, "loop-specific code in the SDK:\n" + "\n".join(found)
