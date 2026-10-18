"""IC-1 through the real binary (AP3b-9 … AP3b-12): what a Python seat received, across a dropped socket,
a client crash and a server restart, concatenates to exactly `mineworld perceived` over the same save.

No `--town`: after genesis, every fact the wanderer perceives is caused by the test — the visitor says a
line to Alice in the café, and the wanderer, in the same place, overhears it (`spoke` is visible to the
Place). The oracle is the binary's own export, never the SDK's output (test rules §25). Every wait is
event-driven with a 30 s ceiling. The facts and the seats named here are social-cafe's; the SDK names
none of them.
"""

from __future__ import annotations

import asyncio
import json
import random
from collections.abc import Callable
from dataclasses import dataclass, field
from itertools import pairwise
from pathlib import Path

import pytest
import support
from conftest import Start
from realserver import INVITE, Server, perceived_export, seated
from test_real_server import TALK, walk_into_reach
from websockets.asyncio.client import ClientConnection
from websockets.asyncio.client import connect as websocket_connect

from mineworld_sdk import (
    CursorCell,
    PerceivedBatch,
    ResumingSeat,
    SeatLost,
    SeatSession,
    offers,
)
from mineworld_sdk.wire.contract import Accepted
from mineworld_sdk.wire.frames import Invite
from mineworld_sdk.wire.ids import EntityKey, EventId, JsonValue, ResumeSecret

pytestmark = pytest.mark.real_server

WORLD = "social-cafe"
PATIENCE_S = 30.0
LINES = 3


class Tap:
    """A real socket whose sent `join` frames are recorded, and whose transport the test can abort
    as a network would: no close frame, no `leave`."""

    def __init__(self, inner: ClientConnection) -> None:
        self.inner = inner
        self.joins: list[dict[str, JsonValue]] = []

    async def send(self, message: str, /) -> None:
        frame = json.loads(message)
        if frame.get("t") == "join":
            self.joins.append(frame)
        await self.inner.send(message)

    async def recv(self) -> str | bytes:
        return await self.inner.recv()

    async def close(self) -> None:
        await self.inner.close()

    def abort(self) -> None:
        self.inner.transport.abort()


@dataclass
class Wire:
    """The wanderer's connector: follows `server` (which a restart replaces), waits while `open` is
    clear, and records every socket with the cell's value when it was opened."""

    server: Server
    cell: CursorCell
    open: asyncio.Event = field(default_factory=asyncio.Event)
    taps: list[Tap] = field(default_factory=list[Tap])
    cursors: list[EventId | None] = field(default_factory=list[EventId | None])

    async def __call__(self) -> Tap:
        self.cursors.append(self.cell.cursor())
        await self.open.wait()
        tap = Tap(await websocket_connect(self.server.url, compression=None, max_size=2**24))
        self.taps.append(tap)
        return tap


@dataclass
class Consumer:
    """The single consumer: takes every batch and commits it, as P4's ingestion would (R-P6-P3b-1)."""

    cell: CursorCell
    batches: list[PerceivedBatch] = field(default_factory=list[PerceivedBatch])
    arrived: asyncio.Event = field(default_factory=asyncio.Event)
    limit: int | None = None

    async def run(self, seat: ResumingSeat) -> None:
        async for batch in seat.perceived():
            self.batches.append(batch)
            self.cell.commit(batch)
            self.arrived.set()
            if self.limit is not None and len(self.batches) >= self.limit:
                return

    def ids(self) -> list[str]:
        return [event.id for batch in self.batches for event in batch.events]

    async def until(self, what: str, ready: Callable[[], bool]) -> None:
        async def watch() -> None:
            while not ready():
                self.arrived.clear()
                await self.arrived.wait()

        try:
            await asyncio.wait_for(watch(), PATIENCE_S)
        except TimeoutError:
            pytest.fail(f"the wanderer never delivered {what} within {PATIENCE_S:.0f} s")


class Speaker:
    """The visitor, saying lines to Alice from within reach; each line's caused events are returned."""

    def __init__(self, visitor: SeatSession, alice: SeatSession) -> None:
        self.visitor, self.alice = visitor, alice

    @classmethod
    async def seated(cls, server: Server) -> Speaker:
        visitor, _ = await seated(server, "visitor")
        alice, _ = await seated(server, "alice")
        await walk_into_reach(visitor, alice)
        return cls(visitor, alice)

    async def say(self, line: str) -> set[str]:
        async def offered() -> None:
            while True:
                seen = (await self.visitor.changed()).observation
                if any(
                    a.action_type == TALK and a.target == self.alice.observer and a.available
                    for a in seen.affordances
                ):
                    return

        await asyncio.wait_for(offered(), PATIENCE_S)
        seen = (await self.visitor.changed()).observation
        outcome = await self.visitor.submit(
            offers.request(seen, TALK, target=self.alice.observer, payload={"utterance": line})
        )
        result = getattr(outcome, "result", None)
        assert isinstance(result, Accepted), f"the line {line!r} was not accepted: {outcome}"
        return {str(event) for event in result.accepted.events}

    async def leave(self) -> None:
        for session in (self.visitor, self.alice):
            await session.leave()


def heard(consumer: Consumer, caused: set[str]) -> Callable[[], bool]:
    """Whether the wanderer has been delivered the `spoke` fact among `caused`."""
    return lambda: any(
        event.id in caused and event.event_type == "spoke"
        for batch in consumer.batches
        for event in batch.events
    )


async def wanderer(wire: Wire, *, resume: str | None = None) -> ResumingSeat:
    wire.open.set()
    return await ResumingSeat.open(
        wire,
        seat=EntityKey("wanderer"),
        invite=Invite(INVITE),
        nickname="wanderer-sdk",
        cursor=wire.cell,
        resume=None if resume is None else ResumeSecret(resume),
    )


def exported_through(save: Path, cursor: EventId | None) -> list[str]:
    """The export, filtered to ids at or below the last committed cursor."""
    assert cursor is not None, "the wanderer committed nothing"
    limit = (len(cursor), cursor)
    return [
        event.id
        for event in perceived_export(WORLD, save, "wanderer")
        if (len(event.id), event.id) <= limit
    ]


def ascending(ids: list[str]) -> bool:
    keys = [(len(i), i) for i in ids]
    return all(a < b for a, b in pairwise(keys))


def test_a_dropped_socket_loses_nothing(mineworld: Start, tmp_path: Path) -> None:
    """AP3b-9, on the platform's default event loop (the proactor on Windows, F-P5b-1)."""
    server = mineworld(WORLD, save=tmp_path / "save", hold=10)

    async def scenario() -> tuple[Consumer, Wire, list[set[str]], int, str]:
        speaker = await Speaker.seated(server)
        cell = CursorCell()
        wire, consumer = Wire(server, cell), Consumer(cell)
        seat = await wanderer(wire)
        reading = asyncio.create_task(consumer.run(seat))
        for n in range(LINES):
            caused = await speaker.say(f"Before {n}.")
            await consumer.until(f"line {n}", heard(consumer, caused))
        wire.open.clear()
        wire.taps[-1].abort()
        away = [await speaker.say(f"While away {n}.") for n in range(LINES)]
        wire.open.set()
        seen = await asyncio.wait_for(seat.changed(), PATIENCE_S)
        while seen.connection < 2:
            seen = await asyncio.wait_for(seat.changed(seen), PATIENCE_S)
        took_over = seat.welcome.took_over
        for n, caused in enumerate(away):
            await consumer.until(f"away line {n}", heard(consumer, caused))
        for n in range(LINES):
            caused = await speaker.say(f"After {n}.")
            await consumer.until(f"after line {n}", heard(consumer, caused))
        await seat.leave()
        await reading
        await speaker.leave()
        return consumer, wire, away, seen.connection, took_over

    consumer, wire, away, connection, took_over = support.run(
        scenario(), timeout_s=120, loop="default"
    )
    server.stop()
    delivered = consumer.ids()
    exported = exported_through(tmp_path / "save", consumer.cell.cursor())
    print(
        f"AP3b-9: {len(delivered)} delivered over {connection} connections, {len(exported)} "
        f"exported; rejoin since {wire.taps[1].joins[0].get('perceived')}, took_over {took_over}"
    )
    assert connection == 2 and took_over == "held"
    assert delivered == exported, "the live stream equals the offline export"
    assert ascending(delivered)
    assert delivered[0] == exported[0], "since: null serves from this world's first fact"
    on_second = {e.id for b in consumer.batches if b.connection == 2 for e in b.events}
    assert all(caused & on_second for caused in away), "the lines said while away came on rejoin"
    rejoin = wire.taps[1].joins[0]
    assert rejoin.get("perceived") == {"since": wire.cursors[1]}, "the cell's value, at that moment"
    assert rejoin.get("resume") is not None and rejoin.get("take_over") is False


def test_a_crashed_client_resumes_from_its_store(mineworld: Start, tmp_path: Path) -> None:
    """AP3b-10: a seat abandoned mid-stream (detach: no leave, the seat held), then a new seat, cold,
    from the same store and the first seat's last resume secret."""
    server = mineworld(WORLD, save=tmp_path / "save", hold=10)
    seed = 20261010
    cut = random.Random(seed).randint(1, 4)
    print(f"AP3b-10: seed {seed}, cut after {cut} delivered batches")

    async def scenario() -> tuple[Consumer, Consumer, str]:
        speaker = await Speaker.seated(server)
        cell = CursorCell()
        first = Consumer(cell, limit=cut)
        seat = await wanderer(Wire(server, cell))
        reading = asyncio.create_task(first.run(seat))
        for n in range(LINES):
            await speaker.say(f"Before {n}.")
        await asyncio.wait_for(reading, PATIENCE_S)
        secret = seat.welcome.resume
        assert secret is not None
        await seat.detach()
        second = Consumer(cell)
        again = await wanderer(Wire(server, cell), resume=secret)
        took_over = again.welcome.took_over
        reading = asyncio.create_task(second.run(again))
        for n in range(LINES):
            caused = await speaker.say(f"After {n}.")
            await second.until(f"after line {n}", heard(second, caused))
        await again.leave()
        await reading
        await speaker.leave()
        return first, second, took_over

    first, second, took_over = support.run(scenario(), timeout_s=120)
    server.stop()
    delivered = first.ids() + second.ids()
    exported = exported_through(tmp_path / "save", second.cell.cursor())
    print(f"AP3b-10: {len(first.ids())} + {len(second.ids())} delivered, {len(exported)} exported")
    assert took_over == "held"
    assert len(set(delivered)) == len(delivered), "no fact twice across the two processes"
    assert delivered == exported


def test_a_restarted_server_is_rejoined_afresh(mineworld: Start, tmp_path: Path) -> None:
    """AP3b-11: the server is killed and started again on the same save, on a new port. Holds and
    secrets do not survive it: the rejoin meets invalid_resume and joins afresh, in the same world."""
    save = tmp_path / "save"
    servers = [mineworld(WORLD, save=save, hold=60)]

    async def scenario() -> tuple[Consumer, Wire, str, str, str]:
        speaker = await Speaker.seated(servers[0])
        cell = CursorCell()
        wire, consumer = Wire(servers[0], cell), Consumer(cell)
        seat = await wanderer(wire)
        instance = seat.instance
        reading = asyncio.create_task(consumer.run(seat))
        for n in range(LINES):
            caused = await speaker.say(f"Before {n}.")
            await consumer.until(f"line {n}", heard(consumer, caused))
        servers[0].stop()
        await asyncio.to_thread(lambda: servers.append(mineworld(WORLD, save=save, hold=60)))
        wire.server = servers[-1]
        seen = await asyncio.wait_for(seat.changed(), PATIENCE_S)
        while seen.connection < 2:
            seen = await asyncio.wait_for(seat.changed(seen), PATIENCE_S)
        took_over = seat.welcome.took_over
        speaker = await Speaker.seated(servers[-1])
        for n in range(LINES):
            caused = await speaker.say(f"After {n}.")
            await consumer.until(f"after line {n}", heard(consumer, caused))
        await seat.leave()
        await reading
        await speaker.leave()
        return consumer, wire, took_over, instance, seat.instance

    consumer, wire, took_over, before, after = support.run(scenario(), timeout_s=120)
    servers[-1].stop()
    delivered = consumer.ids()
    exported = exported_through(save, consumer.cell.cursor())
    resumes = [tap.joins[0].get("resume") is not None for tap in wire.taps]
    print(
        f"AP3b-11: joins with resume {resumes}; took_over {took_over}; {len(delivered)} delivered"
    )
    assert resumes[-2:] == [True, False], "the held secret was refused, then a fresh join"
    assert took_over == "none" and before == after
    assert delivered == exported and ascending(delivered)


def test_a_world_without_a_save_has_no_history_to_serve(mineworld: Start) -> None:
    """AP3b-12 (F-P3b-2): since: null is refused cursor_unavailable on an ephemeral world; the refusal
    is final, and grants nothing: the seat stays joinable."""
    server = mineworld(WORLD)

    async def scenario() -> None:
        with pytest.raises(SeatLost) as lost:
            await ResumingSeat.connect(
                server.url,
                seat=EntityKey("wanderer"),
                invite=Invite(INVITE),
                nickname="wanderer-sdk",
                cursor=CursorCell(),
            )
        assert lost.value.reason == "cursor_unavailable"
        plain = await SeatSession.connect(
            server.url, seat=EntityKey("wanderer"), invite=Invite(INVITE), nickname="plain"
        )
        async with plain:
            assert plain.welcome.took_over == "none"

    support.run(scenario(), timeout_s=60)
