"""One step to the cause (pr-s28e §5.6, AS28-16 (a) and (b); AC-e-7, AC-e-9).

A defect is planted by hand in a copy of the synthetic home, then located with one command of `mineworld-memory`
in a subprocess. The oracle is the planted value itself.
"""

from __future__ import annotations

import sqlite3
import subprocess
import sys
from pathlib import Path

import pytest
from inspect_fixtures import build
from mineworld_sdk.wire.ids import EntityKey

from mineworld_cognition.inspect import RecallRequest, report
from mineworld_cognition.inspect.report import AccessReport, RecallReport
from mineworld_cognition.memory.explain import Explanation
from mineworld_cognition.memory.home import MemoryHome

MODULE = "mineworld_cognition.inspect.cli"
PLANTED = "Bob owes me forty crowns"
"""The wrong gist planted in seat B's access row."""


def cli(*arguments: str | Path) -> bytes:
    done = subprocess.run(
        [sys.executable, "-m", MODULE, *(str(each) for each in arguments)],
        capture_output=True,
        timeout=300,
        check=False,
    )
    assert done.returncode == 0, done.stderr.decode("utf-8")
    return done.stdout


@pytest.fixture(scope="module")
def synthetic(tmp_path_factory: pytest.TempPathFactory) -> Path:
    path = tmp_path_factory.mktemp("synthetic") / "memory.sqlite"
    build(path)
    return path


def _seat_number(connection: sqlite3.Connection, key: str) -> int:
    found: int = connection.execute("SELECT seat FROM seats WHERE key = ?", (key,)).fetchone()[0]
    return found


def _plant(path: Path, seat: str, key: str, gist: str) -> None:
    connection = sqlite3.connect(path)
    number = _seat_number(connection, seat)
    connection.execute(
        "UPDATE access SET gist = ? WHERE seat = ? AND event_key = ?", (gist, number, key)
    )
    connection.commit()
    connection.close()


def _shared_notable(path: Path, first: str, second: str) -> list[tuple[str, int]]:
    """Keys both seats hold as notable, newest first, with the record's time."""
    connection = sqlite3.connect(path)
    a, b = _seat_number(connection, first), _seat_number(connection, second)
    found: list[tuple[str, int]] = connection.execute(
        "SELECT x.event_key, f.at FROM access x JOIN access y ON y.event_key = x.event_key "
        "JOIN fact f ON f.event_key = x.event_key WHERE x.seat = ? AND y.seat = ? "
        "AND x.salience = 'notable' AND y.salience = 'notable' ORDER BY x.event_key DESC",
        (a, b),
    ).fetchall()
    connection.close()
    return found


def _explanations(found: RecallReport) -> list[tuple[str, Explanation]]:
    """Every explanation of the trace, with the text of the line it explains."""
    out: list[tuple[str, Explanation]] = []
    for line in found.trace:
        for cited in line.ranges:
            for each in (cited.first, cited.last, *cited.shown):
                if each is not None:
                    out.append((line.text, each))
    return out


def test_a_wrong_gist_in_one_seats_access_row_is_named_in_one_command(
    synthetic: Path, tmp_path: Path
) -> None:
    # AC-e-7 (AS28-16 (a)). The search below only chooses the input (a shared key that B's section shows on a
    # line of its own); the assertion is on one `inspect recall --trace --json` of the command.
    home_path = tmp_path / "planted.sqlite"
    home_path.write_bytes(synthetic.read_bytes())
    chosen: tuple[str, int] | None = None
    for key, at in _shared_notable(home_path, "alice", "bob")[:60]:
        _plant(home_path, "bob", key, PLANTED)
        with MemoryHome.open_read_only(home_path) as home:
            found = report(home, RecallRequest(seat=EntityKey("bob"), at=at + 60))
        assert isinstance(found, RecallReport)
        if PLANTED in found.section:
            chosen = (key, at)
            break
        original = sqlite3.connect(synthetic)
        number = _seat_number(original, "bob")
        (gist,) = original.execute(
            "SELECT gist FROM access WHERE seat = ? AND event_key = ?", (number, key)
        ).fetchone()
        original.close()
        _plant(home_path, "bob", key, gist)
    assert chosen is not None, "no shared record reaches a line of B's section"
    key, at = chosen
    event = key.lstrip("0")
    out = cli(
        "inspect", home_path, "--seat", "bob", "recall", "--at", str(at + 60), "--trace", "--json"
    )
    bob = RecallReport.model_validate_json(out)
    assert bob.seat == "bob"
    named = [(text, each) for text, each in _explanations(bob) if each.event == event]
    on_line = [(text, each) for text, each in named if PLANTED in text]
    assert on_line, "the line showing the planted gist does not explain its key"
    text, explained = on_line[0]
    assert explained.kind == "held"
    assert explained.access is not None and explained.access.gist == PLANTED
    line = next(line for line in bob.trace if line.text == text)
    assert line.scope == "person"
    # Seat A, for the same key: its own gist, and the same shared envelope.
    alice = AccessReport.model_validate_json(
        cli("inspect", home_path, "--seat", "alice", "access", "--event", event, "--json")
    )
    assert alice.explanation is not None and alice.explanation.access is not None
    assert alice.explanation.access.gist != PLANTED
    assert alice.explanation.record == explained.record
