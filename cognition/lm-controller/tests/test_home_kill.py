"""A killed home resumes to the uninterrupted dump (pr-s28a C7; AC-a-7, I-a-11; step-28 AS28-8, I-S28-4, I-S28-5).

Fifty seeded kills (three per seam per PR, all fifty each night: the rest are `slow`, D-a-15) of a child process (`home_kill.py kill`) driving the kill world into a file home: 17 inside
`SeatView.hold` after the record and before the access row, 17 inside `SeatView.release` after the access deletes
and before collection, 16 at the home's `COMMIT`. After each, the killed file is opened: its committed state holds
I-S28-4 (no record without a holder, no held key without its record). Then every seat is re-delivered from its own
persisted cursor, and the home's dump equals the uninterrupted run's, byte for byte.
"""

from __future__ import annotations

import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

import pytest
from home_kill import INSTANCE, POINTS, Point, drive, seam
from retention_stream import draw

from mineworld_cognition.memory.home import MemoryHome
from mineworld_cognition.memory.scopes import world

CHILD = Path(__file__).resolve().parent / "home_kill.py"
KILLS: dict[Point, int] = {"record": 17, "release": 17, "commit": 16}
SEED = 28


@dataclass(frozen=True)
class Uninterrupted:
    dump: bytes
    calls: dict[Point, int]


@pytest.fixture(scope="module")
def uninterrupted(tmp_path_factory: pytest.TempPathFactory) -> Uninterrupted:
    """The kill world driven once without a kill, counting every seam's calls."""
    counted = dict.fromkeys(POINTS, 0)
    path = tmp_path_factory.mktemp("uninterrupted") / "memory.sqlite"
    with pytest.MonkeyPatch.context() as patch:
        for point in POINTS:

            def count(point: Point = point) -> None:
                counted[point] += 1

            seam(point, count, patch.setattr)
        drive(path)
    with MemoryHome.open(path, INSTANCE) as home:
        return Uninterrupted(home.dump(), counted)


PER_PR = 3
"""Kills per seam a PR runs; the rest of the 50 are `slow` and run each night (`stability`; D-a-15)."""
CASES = [
    pytest.param(point, n, marks=pytest.mark.slow) if n >= PER_PR else pytest.param(point, n)
    for point in POINTS
    for n in range(KILLS[point])
]


@pytest.mark.parametrize(("point", "number"), CASES)
def test_a_killed_home_resumes_to_the_uninterrupted_dump(
    point: Point, number: int, uninterrupted: Uninterrupted, tmp_path: Path
) -> None:
    # AC-a-7. Its mutation (M-a-10) commits the record in a transaction of its own before the access row: a
    # kill between them leaves a committed record no seat holds.
    assert uninterrupted.calls[point] > KILLS[point]
    at = 1 + draw(SEED, POINTS.index(point) * 100 + number, 7) % uninterrupted.calls[point]
    path = tmp_path / "memory.sqlite"
    killed = subprocess.run(
        [sys.executable, str(CHILD), "kill", str(path), point, str(at)],
        capture_output=True,
        encoding="utf-8",
        timeout=300,
        check=False,
    )
    assert killed.returncode == 137, (killed.returncode, killed.stdout, killed.stderr)
    with MemoryHome.open(path, INSTANCE) as home:
        assert world.check_holders(home.connection) == [], (point, at)
    drive(path)
    with MemoryHome.open(path, INSTANCE) as home:
        assert home.dump() == uninterrupted.dump, (point, at)
