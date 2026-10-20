"""The seat export and import (pr-s28e §5.7; AC-e-8, AS28-11, I-S28-2; AC-e-1 for `export`).

The home is the synthetic 12-seat world in a file (`inspect_fixtures.build`). The oracles are the original home's
`MemoryHome.dump()` and each `SeatView.dump()`.
"""

from __future__ import annotations

import hashlib
import json
import random
import subprocess
import sys
from collections.abc import Callable
from pathlib import Path

import pytest
from home_streams import SEATS
from inspect_fixtures import build, streams
from memory_fixtures import INSTANCE, OTHER_INSTANCE
from mineworld_sdk.wire.ids import EntityKey

from mineworld_cognition.memory.home import (
    HomeIdentityMismatch,
    MemoryHome,
    NotASeatExport,
    SeatAlreadyInHome,
    UnsupportedExportFormat,
    UnsupportedStoreSchema,
    export_seat,
    import_seat,
)
from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.scopes.world import ConflictingFact

MODULE = "mineworld_cognition.inspect.cli"


@pytest.fixture(scope="module")
def synthetic(tmp_path_factory: pytest.TempPathFactory) -> Path:
    path = tmp_path_factory.mktemp("synthetic") / "memory.sqlite"
    build(path)
    return path


@pytest.fixture(scope="module")
def exported(synthetic: Path) -> dict[EntityKey, bytes]:
    with MemoryHome.open_read_only(synthetic) as home:
        return {seat: export_seat(home, seat) for seat in home.seats()}


@pytest.fixture(scope="module")
def original(synthetic: Path) -> tuple[bytes, dict[EntityKey, bytes]]:
    with MemoryHome.open_read_only(synthetic) as home:
        return home.dump(), {seat: home.view_of(seat).dump() for seat in home.seats()}


def test_export_reads_without_writing_and_is_deterministic(
    synthetic: Path, exported: dict[EntityKey, bytes]
) -> None:
    data = synthetic.read_bytes()
    with MemoryHome.open_read_only(synthetic) as home:
        again = export_seat(home, SEATS[0])
    assert again == exported[SEATS[0]]
    assert synthetic.read_bytes() == data
    header = json.loads(again.splitlines()[0])
    assert header == ["mineworld-seat-export", 1, "5", INSTANCE, "alice", header[5]]
    digests = [json.loads(line)[5] for line in again.splitlines() if line.startswith(b'["fact",')]
    assert digests and all(len(digest) == 64 for digest in digests)


@pytest.mark.parametrize("order", ["key", "shuffled"])
def test_every_seat_exported_and_imported_reproduces_the_home(
    order: str, exported: dict[EntityKey, bytes], original: tuple[bytes, dict[EntityKey, bytes]]
) -> None:
    # AS28-11 (AC-e-8): in key order and in a seeded shuffled order, into a fresh home of the same instance.
    seats = sorted(exported)
    if order == "shuffled":
        random.Random(28).shuffle(seats)
    home_dump, views = original
    with MemoryHome.open(":memory:", INSTANCE, verify="every-commit") as home:
        for seat in seats:
            assert import_seat(home, exported[seat]) == seat
        assert home.dump() == home_dump
        for seat, dump in views.items():
            assert home.view_of(seat).dump() == dump, seat
        # A re-delivery of each seat's last 50 facts is a no-op: the full digests match.
        for seat in SEATS:
            facts = streams()[seat][-50:]
            ingest(home.view_of(seat), facts, facts[-1].id)
        assert home.dump() == home_dump


def test_an_import_beside_other_seats_leaves_them_unchanged(
    exported: dict[EntityKey, bytes], original: tuple[bytes, dict[EntityKey, bytes]]
) -> None:
    _, views = original
    with MemoryHome.open(":memory:", INSTANCE) as home:
        for seat in SEATS[1:4]:
            import_seat(home, exported[seat])
        before = {seat: home.view_of(seat).dump() for seat in SEATS[1:4]}
        import_seat(home, exported[SEATS[0]])
        assert {seat: home.view_of(seat).dump() for seat in SEATS[1:4]} == before
        assert home.view_of(SEATS[0]).dump() == views[SEATS[0]]


def _edited(data: bytes, line: int, edit: Callable[[list[object]], list[object]]) -> bytes:
    lines = data.decode("utf-8").splitlines()
    lines[line] = json.dumps(
        edit(json.loads(lines[line])), ensure_ascii=False, separators=(",", ":")
    )
    return ("\n".join(lines) + "\n").encode("utf-8")


def test_every_refusal_is_named_and_changes_nothing(exported: dict[EntityKey, bytes]) -> None:
    alice, bob = exported[SEATS[0]], exported[SEATS[1]]

    def facts(data: bytes) -> dict[int, int]:
        return {
            json.loads(line)[1]: n
            for n, line in enumerate(data.splitlines())
            if line.startswith(b'["fact",')
        }

    held_by_both = min(set(facts(alice)) & set(facts(bob)))
    conflicting_line = facts(alice)[held_by_both]
    cases: list[tuple[bytes, type[Exception], str]] = [
        (b"\xff\n", NotASeatExport, "not UTF-8"),
        (alice[:-1], NotASeatExport, "no newline"),
        (alice.replace(b"\n", b"\n{oops\n", 1), NotASeatExport, "line 2"),
        (_edited(alice, 0, lambda h: [h[0], 2, *h[2:]]), UnsupportedExportFormat, "format 2"),
        (_edited(alice, 0, lambda h: [*h[:2], "4", *h[3:]]), UnsupportedStoreSchema, "schema 4"),
        (
            _edited(alice, 0, lambda h: [*h[:3], OTHER_INSTANCE, *h[4:]]),
            HomeIdentityMismatch,
            "world_instance",
        ),
        (bob, SeatAlreadyInHome, "'bob'"),
        (
            _edited(alice, conflicting_line, lambda f: [*f[:5], "0" * 64]),
            ConflictingFact,
            "different content",
        ),
    ]
    with MemoryHome.open(":memory:", INSTANCE) as home:
        import_seat(home, bob)
        before = home.dump()
        for data, error, words in cases:
            with pytest.raises(error, match=words):
                import_seat(home, data)
            assert home.dump() == before, error
        with pytest.raises(NotASeatExport) as refused:
            import_seat(home, cases[2][0])
        assert "oops" not in str(refused.value)


def _cli(*arguments: str | Path) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        [sys.executable, "-m", MODULE, *(str(each) for each in arguments)],
        capture_output=True,
        timeout=300,
        check=False,
    )


def test_the_commands_move_a_seat_between_file_homes(
    synthetic: Path, tmp_path: Path, exported: dict[EntityKey, bytes]
) -> None:
    before = hashlib.sha256(synthetic.read_bytes()).hexdigest()
    printed = _cli("export", synthetic, "--seat", "dev")
    assert printed.returncode == 0, printed.stderr
    assert printed.stdout == exported[EntityKey("dev")]
    out = tmp_path / "dev.seat"
    assert _cli("export", synthetic, "--seat", "dev", "--out", out).returncode == 0
    assert out.read_bytes() == exported[EntityKey("dev")]
    assert hashlib.sha256(synthetic.read_bytes()).hexdigest() == before
    target = tmp_path / "target.sqlite"
    done = _cli("import", target, "--world-instance", INSTANCE, out)
    assert done.returncode == 0, done.stderr
    with MemoryHome.open_read_only(target) as home, MemoryHome.open_read_only(synthetic) as source:
        assert home.view_of(EntityKey("dev")).dump() == source.view_of(EntityKey("dev")).dump()
    again = _cli("import", target, "--world-instance", INSTANCE, out)
    assert again.returncode == 2 and b"already has a seat 'dev'" in again.stderr
