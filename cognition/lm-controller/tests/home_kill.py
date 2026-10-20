"""A child process for the home's lock and kill tests (pr-s28a §5.10; AC-a-7, AC-a-12). Test code: nothing in
production names a kill point; every seam is patched here, in this process only.

    python home_kill.py busy <home> <world-instance>      open a home another process holds; print the outcome
    python home_kill.py read <home>                       open a home read-only; print the outcome
    python home_kill.py hot <home>                        drive the kill world, then die leaving a hot journal
    python home_kill.py die <home> <world-instance>       open a home and die holding it (os._exit, no close)
    python home_kill.py kill <home> <point> <n>           drive the kill world into a file home and die with
                                                          `os._exit(137)` at the n-th call of a seam

The seams (`point`): `record` — inside `SeatView.hold`, after the fact record's insert-or-verify and before the
access row; `release` — inside `SeatView.release`, after the access deletes and before the records are collected;
`commit` — the home's `COMMIT`, before it runs. `drive` is also what the test runs in its own process, for the
uninterrupted run and for the resume: each seat is delivered what lies above its persisted cursor, as the server
re-delivers from a seat's cursor.
"""

from __future__ import annotations

import os
import sqlite3
import sys
from collections.abc import Callable, Iterable
from pathlib import Path
from typing import Literal

import home_streams
from home_streams import CAST, SEATS, interleave, seat_streams
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EventId, WorldInstanceId
from retention_driver import retention
from support import run

from mineworld_cognition.memory.home import MemoryHome
from mineworld_cognition.memory.home import home as home_module
from mineworld_cognition.memory.records import EventKey, StoreIdentity, event_key
from mineworld_cognition.memory.retention.absorb import absorb
from mineworld_cognition.memory.scopes import world

type Point = Literal["record", "release", "commit"]
POINTS: tuple[Point, ...] = ("record", "release", "commit")
INSTANCE = WorldInstanceId("0000000018dd1ad807f6bbd134630000")
KILL_SEATS = 4
KILL_DAYS = 3
FRAME = 64
INTERLEAVING = 5


def drive(path: Path) -> None:
    """The kill world (4 seats of `home_streams.world(7, 3)`, frames of 64, one interleaving) through `absorb`
    under `consolidate`, into the home at `path`, each seat from its persisted cursor on."""
    seats = SEATS[:KILL_SEATS]
    streams = {
        seat: facts
        for seat, facts in seat_streams(home_streams.world(7, KILL_DAYS)).items()
        if seat in seats
    }
    kept = retention("consolidate")
    with MemoryHome.open(path, INSTANCE) as home:
        views = {
            seat: home.seat(
                StoreIdentity(world_instance=INSTANCE, seat=seat, self_entity=CAST[seat])
            )
            for seat in seats
        }
        pending = {seat: _after(facts, views[seat].cursor()) for seat, facts in streams.items()}
        for seat, batch in interleave(pending, FRAME, INTERLEAVING):
            run(absorb(views[seat], batch, batch[-1].id, kept), timeout_s=600)


def _after(facts: list[PerceivedEvent], cursor: EventId | None) -> list[PerceivedEvent]:
    """What the server re-delivers to a seat whose persisted cursor is `cursor`."""
    if cursor is None:
        return facts
    return [fact for fact in facts if event_key(fact.id) > event_key(cursor)]


type Patch = Callable[[object, str, object], None]


def seam(point: Point, on_call: Callable[[], None], patch: Patch = setattr) -> None:
    """Calls `on_call` at every call of the seam `point` (`record`: after the record's insert-or-verify;
    `release` and `commit`: before the seam's own work). `patch` is `setattr`, or a test's
    `monkeypatch.setattr`, which undoes it."""
    if point == "record":
        real_record = world.insert_or_verify

        def record(db: sqlite3.Connection, fact: PerceivedEvent, digest: str) -> bool:
            inserted = real_record(db, fact, digest)
            on_call()
            return inserted

        patch(world, "insert_or_verify", record)
    elif point == "release":
        real_collect = world.collect

        def collect(db: sqlite3.Connection, keys: Iterable[EventKey]) -> None:
            on_call()
            real_collect(db, keys)

        patch(world, "collect", collect)
    else:
        real_commit = getattr(home_module, "_commit")  # noqa: B009 - the home's private seam

        def commit(connection: sqlite3.Connection) -> None:
            on_call()
            real_commit(connection)

        patch(home_module, "_commit", commit)


def kill(path: Path, point: Point, at: int) -> None:
    seen = [0]

    def on_call() -> None:
        seen[0] += 1
        if seen[0] == at:
            os._exit(137)

    seam(point, on_call)
    drive(path)
    print("finished", flush=True)


def busy(path: Path, instance: WorldInstanceId) -> None:
    try:
        home = MemoryHome.open(path, instance)
    except ValueError as error:
        print(type(error).__name__, flush=True)
        return
    home.close()
    print("opened", flush=True)


def read(path: Path) -> None:
    """Opens a home read-only (pr-s28e §5.2) and prints the outcome, as `busy` does."""
    try:
        home = MemoryHome.open_read_only(path)
    except ValueError as error:
        print(type(error).__name__, flush=True)
        return
    home.close()
    print("opened", flush=True)


def hot(path: Path) -> None:
    """Drives the kill world into `path`, then dies inside a transaction that has spilled pages into the file
    (a one-page cache): what leaves a **hot** journal. A kill at a seam does not, because SQLite keeps every
    uncommitted page in its cache until the commit, and in exclusive locking mode the journal of a committed
    transaction stays on disk with a zeroed header."""
    drive(path)
    connection = sqlite3.connect(path, isolation_level=None)
    connection.execute("PRAGMA cache_size = 1")
    connection.execute("BEGIN IMMEDIATE")
    connection.execute("DELETE FROM access")
    connection.execute("DELETE FROM fact")
    print("spilled", flush=True)
    os._exit(137)


def die(path: Path, instance: WorldInstanceId) -> None:
    MemoryHome.open(path, instance)
    print("holding", flush=True)
    os._exit(137)


def main(arguments: list[str]) -> None:
    scenario, path = arguments[0], Path(arguments[1])
    if scenario == "busy":
        busy(path, WorldInstanceId(arguments[2]))
    elif scenario == "read":
        read(path)
    elif scenario == "hot":
        hot(path)
    elif scenario == "die":
        die(path, WorldInstanceId(arguments[2]))
    elif scenario == "kill":
        chosen: list[Point] = [each for each in POINTS if each == arguments[2]]
        kill(path, chosen[0], int(arguments[3]))
    else:
        raise SystemExit(f"unknown scenario {scenario!r}")


if __name__ == "__main__":
    main(sys.argv[1:])
