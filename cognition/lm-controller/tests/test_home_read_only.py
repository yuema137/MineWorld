"""Opening a memory home read-only (pr-s28e §5.2, AC-e-1's open cases, AC-e-5's home totals; I-e-1).

The home is a file driven by `home_kill.drive` (4 seats, 3 days, `consolidate`); the held and hot-journal homes
come from `home_kill.py`'s children, the other process. Every case compares the file's bytes before and after.
"""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

import pytest
from home_kill import INSTANCE, drive
from memory_fixtures import OTHER_INSTANCE

from mineworld_cognition.memory.home import (
    HomeBusy,
    HomeIdentityMismatch,
    HomeNeedsRecovery,
    MemoryHome,
    NotAMemoryHome,
)
from mineworld_cognition.memory.home.home import READ_ONLY
from mineworld_cognition.memory.records import StoreIdentity

CHILD = Path(__file__).resolve().parent / "home_kill.py"


def _fingerprint(path: Path) -> tuple[str, int, int]:
    data = path.read_bytes()
    return hashlib.sha256(data).hexdigest(), len(data), path.stat().st_mtime_ns


def _journal(path: Path) -> Path:
    return path.with_name(path.name + "-journal")


@pytest.fixture(scope="module")
def driven(tmp_path_factory: pytest.TempPathFactory) -> Path:
    path = tmp_path_factory.mktemp("driven") / "memory.sqlite"
    drive(path)
    return path


def test_a_missing_path_is_not_created(tmp_path: Path) -> None:
    path = tmp_path / "absent.sqlite"
    with pytest.raises(NotAMemoryHome) as refused:
        MemoryHome.open_read_only(path)
    assert refused.value.missing
    assert not path.exists()
    assert list(tmp_path.iterdir()) == []


def test_reading_a_home_changes_nothing_and_sees_what_open_sees(driven: Path) -> None:
    with MemoryHome.open(driven, INSTANCE) as home:
        expected = home.dump()
        views = {key: home.view_of(key).dump() for key in home.seats()}
    before = _fingerprint(driven)
    with MemoryHome.open_read_only(driven) as home:
        assert home.world_instance == INSTANCE
        assert home.dump() == expected
        for key in home.seats():
            view = home.view_of(key)
            assert view.dump() == views[key]
            assert view.size() == len(views[key])
            assert view.content_size() == view.size() - sum(s for _, s in view.row_bytes("meta"))
        home.totals()
    assert _fingerprint(driven) == before
    assert not _journal(driven).exists()


def test_every_write_path_of_a_read_only_home_raises(driven: Path) -> None:
    before = _fingerprint(driven)
    with MemoryHome.open_read_only(driven) as home:
        key = home.seats()[0]
        view = home.view_of(key)
        identity = StoreIdentity(world_instance=INSTANCE, seat=key, self_entity=view.me)
        for write in (
            lambda: home.transaction().__enter__(),
            lambda: home.seat(identity),
            lambda: home.remove_seat(key),
            home.reclaim,
        ):
            with pytest.raises(ValueError, match=READ_ONLY):
                write()
        assert not home.connection.in_transaction
    assert _fingerprint(driven) == before


def test_the_totals_are_the_segments_of_the_dump(driven: Path) -> None:
    # AC-e-5's home half: the oracle is the dump, cut by the test at its line tags.
    with MemoryHome.open_read_only(driven) as home:
        dump = home.dump()
        totals = home.totals()
        page_size = home.connection.execute("PRAGMA page_size").fetchone()[0]
        pages = home.connection.execute("PRAGMA page_count").fetchone()[0]
    segments: dict[str, int] = {}
    seats: list[list[object]] = []
    current: list[object] | None = None
    for text in dump.decode("utf-8").splitlines():
        tag = json.loads(text)[0]
        size = len(text.encode("utf-8")) + 1
        if tag == "seat":
            current = [json.loads(text)[1], 0]
            seats.append(current)
        if current is None:
            segments[tag] = segments.get(tag, 0) + size
        else:
            current[1] = int(str(current[1])) + size
    assert totals.home_meta == segments["home_meta"]
    assert totals.fact == segments["fact"]
    assert totals.fact_who == segments["fact_who"]
    assert [[key, size] for key, size in totals.seats] == seats
    assert totals.dump == len(dump)
    assert totals.file == pages * page_size == driven.stat().st_size
    assert 0 <= totals.free < totals.file


def test_another_world_instance_is_read_from_the_file(tmp_path: Path) -> None:
    path = tmp_path / "other.sqlite"
    MemoryHome.open(path, OTHER_INSTANCE).close()
    with MemoryHome.open_read_only(path) as home:
        assert home.world_instance == OTHER_INSTANCE
    with pytest.raises(HomeIdentityMismatch):
        MemoryHome.open(path, INSTANCE)


def _read_child(path: Path) -> str:
    result = subprocess.run(
        [sys.executable, str(CHILD), "read", str(path)],
        capture_output=True,
        encoding="utf-8",
        timeout=60,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    return result.stdout.strip()


def test_a_held_home_is_busy(driven: Path) -> None:
    # The running process holds its home with SQLite's exclusive lock: inspection is refused by name, and
    # succeeds once the holder closes it.
    with MemoryHome.open(driven, INSTANCE):
        assert _read_child(driven) == HomeBusy.__name__
        with pytest.raises(HomeBusy):
            MemoryHome.open_read_only(driven)
    assert _read_child(driven) == "opened"


def _killed(path: Path, *arguments: str) -> None:
    killed = subprocess.run(
        [sys.executable, str(CHILD), arguments[0], str(path), *arguments[1:]],
        capture_output=True,
        encoding="utf-8",
        timeout=300,
        check=False,
    )
    assert killed.returncode == 137, (killed.returncode, killed.stderr)


def test_a_hot_journal_needs_recovery_and_is_left_alone(tmp_path: Path) -> None:
    # A process killed inside a transaction that had written pages into the file leaves a hot journal. Rolling
    # it back is a write, so the read-only open refuses it and changes neither the file nor its journal.
    path = tmp_path / "memory.sqlite"
    _killed(path, "hot")
    journal = _journal(path)
    assert journal.exists(), "the kill left no journal"
    before = (_fingerprint(path), _fingerprint(journal))
    with pytest.raises(HomeNeedsRecovery):
        MemoryHome.open_read_only(path)
    assert _read_child(path) == HomeNeedsRecovery.__name__
    assert (_fingerprint(path), _fingerprint(journal)) == before
    # The writable open recovers it; then it reads, and the killed transaction left nothing.
    with MemoryHome.open(path, INSTANCE) as home:
        recovered = home.dump()
    with MemoryHome.open_read_only(path) as home:
        assert home.dump() == recovered
        assert home.totals().fact > 0


def test_a_kill_before_any_page_is_written_reads_the_committed_state(tmp_path: Path) -> None:
    # A kill at the `record` seam leaves the uncommitted pages in the dead process's cache only: the journal on
    # disk is a committed one with a zeroed header (exclusive locking mode keeps it), not hot. The read-only
    # open reads the last committed state, the same one `MemoryHome.open` recovers to, and changes nothing.
    path = tmp_path / "memory.sqlite"
    _killed(path, "kill", "record", "400")
    before = _fingerprint(path)
    with MemoryHome.open_read_only(path) as home:
        seen = home.dump()
    assert _fingerprint(path) == before
    with MemoryHome.open(path, INSTANCE) as home:
        assert home.dump() == seen
