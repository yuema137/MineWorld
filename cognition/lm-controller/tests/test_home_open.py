"""Opening a memory home (pr-s28a §5.5; AC-a-12, I-a-10): create, reopen, refuse by name, lock.

A P4/FG-b store is built here by hand, table by table, as schemas 2 and 3 wrote them (`meta` with the identity
rows), so the refusal is tested against the files those schemas left on disk; the file's bytes are compared
before and after. The lock is tested across processes: `home_kill.py` is the other process.
"""

from __future__ import annotations

import hashlib
import sqlite3
import subprocess
import sys
from pathlib import Path

import pytest
from memory_fixtures import ALICE, INSTANCE, OTHER_INSTANCE, alice_identity, names_of_everyone
from mineworld_sdk.wire.ids import EventId

from mineworld_cognition.memory.home import (
    HomeBusy,
    HomeIdentityMismatch,
    MemoryHome,
    NotAMemoryHome,
    UnsupportedStoreSchema,
)
from mineworld_cognition.memory.ingest import ingest

CHILD = Path(__file__).resolve().parent / "home_kill.py"


def _child(scenario: str, path: Path) -> str:
    result = subprocess.run(
        [sys.executable, str(CHILD), scenario, str(path), INSTANCE],
        capture_output=True,
        encoding="utf-8",
        timeout=60,
        check=False,
    )
    assert result.returncode in (0, 137), result.stderr
    return result.stdout.strip()


def _old_store(path: Path, version: int) -> str:
    """A schema-2 or schema-3 store as P4/FG-b left it: its `meta` identity rows and an `l0` row. Returns the
    file's sha256."""
    connection = sqlite3.connect(path)
    # As P4's `_create` did, before any table.
    connection.execute("PRAGMA auto_vacuum = INCREMENTAL")
    connection.execute("CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)")
    connection.execute(
        "CREATE TABLE l0 (event_key TEXT PRIMARY KEY, event_id TEXT NOT NULL, gist TEXT NOT NULL)"
    )
    connection.executemany(
        "INSERT INTO meta (key, value) VALUES (?, ?)",
        [
            ("world_instance", INSTANCE),
            ("seat", "alice"),
            ("self_entity", ALICE),
            ("schema", str(version)),
            ("cursor", "12"),
        ],
    )
    connection.execute("INSERT INTO l0 VALUES ('00000000000000000012', '12', 'Bob said hello')")
    connection.commit()
    connection.close()
    return hashlib.sha256(path.read_bytes()).hexdigest()


def test_a_new_home_is_created_and_reopened_with_its_identity(tmp_path: Path) -> None:
    path = tmp_path / "memory.sqlite"
    with MemoryHome.open(path, INSTANCE) as home:
        assert home.connection.execute("PRAGMA auto_vacuum").fetchone()[0] == 2  # incremental
        created = home.dump()
    with MemoryHome.open(path, INSTANCE) as home:
        assert home.dump() == created
    assert (
        created
        == f'["home_meta","schema","5"]\n["home_meta","world_instance","{INSTANCE}"]\n'.encode()
    )


def _marker_row(path: Path) -> list[tuple[str]]:
    connection = sqlite3.connect(path)
    try:
        return connection.execute("SELECT value FROM home_meta WHERE key = 'created'").fetchall()
    finally:
        connection.close()


def test_a_home_keeps_its_creation_marker_and_a_rebuilt_home_has_another(tmp_path: Path) -> None:
    # AC-m-1, AC-m-2 (ARC-152): two homes fed identically dump identically (I-S28-5), but their markers differ,
    # and the marker survives every reopen. Its mutation writes a constant marker, or rewrites it at open.
    first, second = tmp_path / "first.sqlite", tmp_path / "second.sqlite"
    dumps: list[bytes] = []
    for path in (first, second):
        with MemoryHome.open(path, INSTANCE) as home:
            view = home.seat(alice_identity())
            ingest(view, names_of_everyone(), EventId("4"))
            dumps.append(home.dump())
    assert dumps[0] == dumps[1]
    assert b"created" not in dumps[0]
    [(marker,)] = _marker_row(first)
    assert len(marker) == 32 and set(marker) <= set("0123456789abcdef")
    assert _marker_row(second) != [(marker,)]
    with MemoryHome.open(first, INSTANCE) as home:
        assert home.created == marker
        assert home.dump() == dumps[0]
        assert marker.encode() not in home.view_of(alice_identity().seat).dump()
    assert _marker_row(first) == [(marker,)]


def test_a_home_made_before_the_marker_is_marked_once_at_open(tmp_path: Path) -> None:
    # AC-m-3: a schema-4 home written by S28-a's code (no `created` row) receives one at its first open and
    # keeps it; nothing it dumps changes.
    path = tmp_path / "memory.sqlite"
    with MemoryHome.open(path, INSTANCE) as home:
        home.connection.execute("DELETE FROM home_meta WHERE key = 'created'")
        before = home.dump()
    assert _marker_row(path) == []
    with MemoryHome.open(path, INSTANCE) as home:
        marker = home.created
        assert home.dump() == before
    assert _marker_row(path) == [(marker,)]
    with MemoryHome.open(path, INSTANCE) as home:
        assert home.created == marker


@pytest.mark.parametrize("version", [2, 3])
def test_an_older_store_is_refused_by_name_and_left_unchanged(tmp_path: Path, version: int) -> None:
    # AC-a-12, I-a-10: QS28-8's rule; F-S28a-1's number. Its mutation opens the old file as a new home.
    path = tmp_path / "alice.sqlite"
    before = _old_store(path, version)
    with pytest.raises(UnsupportedStoreSchema) as raised:
        MemoryHome.open(path, INSTANCE)
    assert (raised.value.found, raised.value.wanted) == (str(version), 5)
    assert str(raised.value).startswith("memory.home:")
    assert "rebuild it from the perceived stream" in str(raised.value)
    assert "since: null" in str(raised.value)
    assert hashlib.sha256(path.read_bytes()).hexdigest() == before


def _home_of_schema(path: Path, version: str) -> str:
    """A home as another schema's code left it: S28-b's schema 5 differs from S28-a's schema 4 only in two
    tables' columns, and the home is refused by its `home_meta.schema` row before any table is read, so a home
    whose row says `version` stands for that schema's file. Returns the file's sha256."""
    with MemoryHome.open(path, INSTANCE) as home:
        home.connection.execute("UPDATE home_meta SET value = ? WHERE key = 'schema'", (version,))
    return hashlib.sha256(path.read_bytes()).hexdigest()


def test_a_schema_four_home_is_refused_by_name_and_left_unchanged(tmp_path: Path) -> None:
    # AC-b-3 (F-S28b-1, QS28-8): S28-a's schema-4 home is rebuilt, never read. Its mutation M-b-4 (`_check`
    # accepts schema 4) opens the file.
    path = tmp_path / "memory.sqlite"
    before = _home_of_schema(path, "4")
    with pytest.raises(UnsupportedStoreSchema) as raised:
        MemoryHome.open(path, INSTANCE)
    assert (raised.value.found, raised.value.wanted) == ("4", 5)
    assert "rebuild it from the perceived stream" in str(raised.value)
    assert hashlib.sha256(path.read_bytes()).hexdigest() == before


def test_a_newer_home_is_refused(tmp_path: Path) -> None:
    path = tmp_path / "memory.sqlite"
    _home_of_schema(path, "6")
    with pytest.raises(UnsupportedStoreSchema) as raised:
        MemoryHome.open(path, INSTANCE)
    assert raised.value.found == "6"


def test_a_database_that_is_not_memory_is_refused(tmp_path: Path) -> None:
    path = tmp_path / "world.sqlite"
    connection = sqlite3.connect(path)
    connection.execute("CREATE TABLE facts (id INTEGER)")
    connection.commit()
    connection.close()
    with pytest.raises(NotAMemoryHome):
        MemoryHome.open(path, INSTANCE)


def test_another_world_instances_home_is_refused_and_unchanged(tmp_path: Path) -> None:
    path = tmp_path / "memory.sqlite"
    with MemoryHome.open(path, INSTANCE) as home:
        before = home.dump()
    with pytest.raises(HomeIdentityMismatch) as raised:
        MemoryHome.open(path, OTHER_INSTANCE)
    assert (raised.value.field, raised.value.stored, raised.value.given) == (
        "world_instance",
        INSTANCE,
        OTHER_INSTANCE,
    )
    with MemoryHome.open(path, INSTANCE) as home:
        assert home.dump() == before


def test_a_held_home_is_busy(tmp_path: Path) -> None:
    # AC-a-12, DEP-61: a second process cannot open a held home, and can once it is closed. Its mutation
    # (M-a-1) connects with a timeout and takes no exclusive lock: the child opens the held home. Both a home
    # just created (its first write takes SQLite's write lock) and one reopened without a write (only the
    # explicit exclusive transaction locks it) are held.
    path = tmp_path / "memory.sqlite"
    with MemoryHome.open(path, INSTANCE):
        assert _child("busy", path) == HomeBusy.__name__
    with MemoryHome.open(path, INSTANCE):
        assert _child("busy", path) == HomeBusy.__name__
    assert _child("busy", path) == "opened"


def test_a_killed_holder_leaves_no_stale_lock(tmp_path: Path) -> None:
    # DEP-61's reason: the operating system releases SQLite's lock when the holder dies, with no side file.
    path = tmp_path / "memory.sqlite"
    with MemoryHome.open(path, INSTANCE):
        pass
    assert _child("die", path) == "holding"
    with MemoryHome.open(path, INSTANCE) as home:
        assert home.seats() == ()
