"""AP3b-3 and AP3b-4 at the stream: delivery, suppression, the cursor's authority, the local bound.

`World` stands in for the server's contract (`PROTOCOL.md` §5.8): a join from cursor `since` is served
exactly the observer's facts after it, in frames, the last carrying the head as `through`. The stream
under test never sees `World`; it sees only the frames. What the seat does around these operations — the
join frame it sends, the socket it closes — is `test_resuming.py`'s.
"""

from __future__ import annotations

import pytest
import support

from mineworld_sdk.perceived import (
    CursorAhead,
    CursorCell,
    LocalLag,
    PerceivedBatch,
    PerceivedStream,
)
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.frames import Perceived
from mineworld_sdk.wire.ids import EventId, event_order


def event(event_id: str) -> PerceivedEvent:
    return PerceivedEvent.model_validate(
        {
            "id": event_id,
            "at": 4,
            "event_type": "spoke",
            "subjects": ["5"],
            "participants": ["5", "101"],
            "place": {"entity": "3", "entity_type": "place"},
            "caused_by": {"action": "41"},
            "payload": {"event_type": "spoke", "schema_version": 1, "payload": {"utterance": "hi"}},
            "visibility": {"place": {"entity": "3", "entity_type": "place"}},
            "provenance": {"emitted_by": "conversation", "controller_decision": "41"},
        }
    )


def frame(through: str, *ids: str) -> Perceived:
    return Perceived(t="perceived", through=EventId(through), events=[event(i) for i in ids])


class World:
    """The facts one observer learned, and what a join from a cursor is served (`PROTOCOL.md` §5.8)."""

    def __init__(self, *admitted: str) -> None:
        self.admitted = admitted

    def serve(self, since: EventId | None, head: str, per_frame: int = 3) -> list[Perceived]:
        after = [
            i
            for i in self.admitted
            if (since is None or event_order(EventId(i)) > event_order(since))
            and event_order(EventId(i)) <= event_order(EventId(head))
        ]
        chunks = [after[k : k + per_frame] for k in range(0, len(after), per_frame)] or [[]]
        return [
            frame(head if n == len(chunks) - 1 else chunk[-1], *chunk)
            for n, chunk in enumerate(chunks)
        ]


def ids(batches: list[PerceivedBatch]) -> list[str]:
    return [e.id for batch in batches for e in batch.events]


def take(stream: PerceivedStream, count: int) -> list[PerceivedBatch]:
    """The next `count` batches, each already waiting: nothing here waits for one to arrive."""

    async def taking() -> list[PerceivedBatch]:
        return [await anext(stream) for _ in range(count)]

    return support.run(taking(), timeout_s=5)


def test_each_fact_reaches_the_consumer_once_and_the_store_decides_the_cursor() -> None:
    # AP3b-3: the consumer takes A and B but commits only A; the rejoin presents A's through, the
    # server re-sends what followed it, and the consumer receives only what it was never handed.
    cell = CursorCell()
    stream = PerceivedStream(cell, bound=4096)
    stream.accept(frame("10", "7", "10"), connection=1)
    stream.accept(frame("20", "12", "15"), connection=1)
    first, second = take(stream, 2)
    cell.commit(first)
    stream.discard()  # the socket dropped
    since = stream.since()
    assert since == "10", "the rejoin presents the store's cursor, not the last through received"
    # The design's literal was 18 (DV-P3b-4): a fact at or below B's through 20 cannot follow it on the
    # server's contract (§5.8: through is the newest id considered), so 22 stands in for it.
    stream.accept(frame("25", "12", "15", "22"), connection=2)
    (third,) = take(stream, 1)
    assert [e.id for e in third.events] == ["22"] and third.through == "25"
    assert third.connection == 2
    delivered = ids([first, second, third])
    assert delivered == ["7", "10", "12", "15", "22"]
    assert sorted(delivered, key=lambda i: event_order(EventId(i))) == delivered


def test_a_cursor_one_id_late_loses_the_fact_after_it() -> None:
    # AP3b-3 (c), IC-1's literal mutation: fact 11 is admitted, so a rejoin from anything but the
    # store's cursor "10" misses it or repeats it. The served facts come from `World`, not the stream.
    world = World("7", "10", "11", "12", "15", "18")
    cell = CursorCell()
    stream = PerceivedStream(cell, bound=4096)
    for f in world.serve(None, head="10"):
        stream.accept(f, connection=1)
    (first,) = take(stream, 1)
    cell.commit(first)
    stream.discard()
    for f in world.serve(stream.since(), head="18"):
        stream.accept(f, connection=2)
    rest = take(stream, 2)
    assert ids([first, *rest]) == list(world.admitted)


def test_a_new_process_resumes_from_exactly_the_store_cursor() -> None:
    # AP3b-3 (c), the half no CursorAhead can catch: a new process has delivered nothing, so a late
    # cursor ("11" for the store's "10") would silently lose admitted fact 11.
    world = World("7", "10", "11", "12", "15", "18")
    cell = CursorCell(EventId("10"))
    stream = PerceivedStream(cell, bound=4096)
    served = world.serve(stream.since(), head="18")
    for f in served:
        stream.accept(f, connection=1)
    assert ids(take(stream, len(served))) == ["11", "12", "15", "18"]


def test_an_empty_batch_that_moves_the_cursor_is_delivered() -> None:
    # D-P3b-3: the end of a backfill with no fact for this observer still advances the cursor.
    stream = PerceivedStream(CursorCell(), bound=4096)
    stream.accept(frame("10", "7", "10"), connection=1)
    stream.accept(frame("40"), connection=1)
    stream.accept(frame("40"), connection=1)
    batches = take(stream, 2)
    assert [(b.through, len(b.events)) for b in batches] == [("10", 2), ("40", 0)]


def test_a_source_ahead_of_what_was_delivered_is_refused() -> None:
    # AP3b-3, second case: the caller claims to have ingested facts it was never given.
    class Ahead:
        def cursor(self) -> EventId | None:
            return EventId("30")

    stream = PerceivedStream(Ahead(), bound=4096)
    stream.accept(frame("25", "18", "25"), connection=1)
    take(stream, 1)
    with pytest.raises(CursorAhead):
        stream.since()


def test_more_waiting_than_the_bound_discards_and_the_retry_loses_nothing() -> None:
    # AP3b-4 at the stream: bound 4, three frames of two facts, nothing read; the third overflows, every
    # waiting batch is discarded, and the facts served again from the store's cursor arrive whole.
    world = World("7", "8", "9", "10", "11", "12")
    cell = CursorCell()
    stream = PerceivedStream(cell, bound=4)
    served = world.serve(cell.cursor(), head="12", per_frame=2)
    stream.accept(served[0], connection=1)
    stream.accept(served[1], connection=1)
    with pytest.raises(LocalLag):
        stream.accept(served[2], connection=1)
    delivered: list[PerceivedBatch] = []
    for f in world.serve(stream.since(), head="12", per_frame=2):
        stream.accept(f, connection=2)
        (batch,) = take(stream, 1)
        cell.commit(batch)
        delivered.append(batch)
    assert ids(delivered) == list(world.admitted)
    assert cell.cursor() == "12"


def test_the_stream_ends_after_what_was_accepted() -> None:
    # A terminal end delivers the batches already waiting, then raises; a leave ends it quietly.
    lost = PerceivedStream(CursorCell(), bound=4096)
    lost.accept(frame("10", "7", "10"), connection=1)
    lost.finish(RuntimeError("the seat is lost"))

    async def drain(stream: PerceivedStream) -> list[PerceivedBatch]:
        return [batch async for batch in stream]

    with pytest.raises(RuntimeError):
        support.run(drain(lost), timeout_s=5)
    left = PerceivedStream(CursorCell(), bound=4096)
    left.accept(frame("10", "7", "10"), connection=1)
    left.finish(None)
    assert ids(support.run(drain(left), timeout_s=5)) == ["7", "10"]


def test_a_cell_never_goes_back() -> None:
    cell = CursorCell(EventId("20"))
    with pytest.raises(ValueError, match="never goes back"):
        cell.commit(PerceivedBatch((), EventId("19"), 1))
