"""`mineworld-memory inspect` through its console script, in a subprocess (pr-s28e §5.4, §5.5, §5.8; AC-e-1,
AC-e-4, AC-e-5, AC-e-10).

The homes are the synthetic 12-seat world in a file (`inspect_fixtures.build`): under `consolidate`, and under a
forgetful `importance-recency-cap` so that forgotten ranges exist. The oracles are the seat view dumps and the
home dump, parsed here.
"""

from __future__ import annotations

import hashlib
import json
import shutil
import sqlite3
import subprocess
import sys
from pathlib import Path

import pytest
from home_streams import SEATS
from inspect_fixtures import FORGETFUL, build, streams
from memory_fixtures import rows

from mineworld_cognition.inspect.report import BytesReport
from mineworld_cognition.memory.home import MemoryHome
from mineworld_cognition.memory.home.schema import VIEW_TABLES

MODULE = "mineworld_cognition.inspect.cli"


def cli(*arguments: str | Path, script: bool = False) -> subprocess.CompletedProcess[bytes]:
    """The command in a subprocess: `python -m …`, or the installed console script."""
    if script:
        found = shutil.which("mineworld-memory", path=str(Path(sys.executable).parent))
        assert found is not None, "the console script mineworld-memory is not installed"
        command: list[str] = [found]
    else:
        command = [sys.executable, "-m", MODULE]
    return subprocess.run(
        [*command, *(str(each) for each in arguments)],
        capture_output=True,
        timeout=300,
        check=False,
    )


def ok(*arguments: str | Path) -> bytes:
    done = cli(*arguments)
    assert done.returncode == 0, done.stderr.decode("utf-8")
    return done.stdout


def report(*arguments: str | Path) -> dict[str, object]:
    loaded: dict[str, object] = json.loads(ok(*arguments, "--json"))
    return loaded


@pytest.fixture(scope="module")
def synthetic(tmp_path_factory: pytest.TempPathFactory) -> Path:
    path = tmp_path_factory.mktemp("synthetic") / "memory.sqlite"
    build(path)
    return path


@pytest.fixture(scope="module")
def forgetful(tmp_path_factory: pytest.TempPathFactory) -> Path:
    path = tmp_path_factory.mktemp("forgetful") / "memory.sqlite"
    build(path, "importance-recency-cap", **FORGETFUL)
    return path


def _fingerprint(path: Path) -> tuple[str, int, int]:
    data = path.read_bytes()
    return hashlib.sha256(data).hexdigest(), len(data), path.stat().st_mtime_ns


def _views(seat: str) -> list[list[str]]:
    return [
        ["--seat", seat, "access"],
        ["--seat", seat, "access", "--event", "1500"],
        ["--seat", seat, "forgotten"],
        ["--seat", seat, "bytes"],
        ["bytes"],
    ]


def test_every_view_reads_without_writing(forgetful: Path) -> None:
    # AC-e-1 for these views: the file's sha256, size and mtime are unchanged and no journal appears.
    before = _fingerprint(forgetful)
    listing = sorted(path.name for path in forgetful.parent.iterdir())
    for arguments in _views("alice"):
        ok("inspect", forgetful, *arguments)
        ok("inspect", forgetful, *arguments, "--json")
    assert _fingerprint(forgetful) == before
    assert sorted(path.name for path in forgetful.parent.iterdir()) == listing


def test_the_console_script_is_the_same_command(synthetic: Path) -> None:
    by_module = cli("inspect", synthetic, "--seat", "bob", "bytes", "--json")
    by_script = cli("inspect", synthetic, "--seat", "bob", "bytes", "--json", script=True)
    assert by_module.returncode == by_script.returncode == 0, by_script.stderr
    assert by_module.stdout == by_script.stdout


def test_reports_are_byte_identical_across_runs_with_newline_ends(synthetic: Path) -> None:
    # AC-e-10: two runs, every view, JSON and text; `\n` line ends, no `\r`, on every platform.
    for arguments in _views("carol"):
        for form in ([], ["--json"]):
            first = ok("inspect", synthetic, *arguments, *form)
            assert first == ok("inspect", synthetic, *arguments, *form)
            assert first.endswith(b"\n") and b"\r" not in first


def test_forgotten_is_the_seat_dumps_forgotten_and_pass_lines(forgetful: Path) -> None:
    # AC-e-4 (R-S28e-b-1): every attribution column, `forced` exactly for a `host:` policy, and the pass log
    # with `prose_contradictions`, newest first.
    with MemoryHome.open_read_only(forgetful) as home:
        dumps = {seat: home.view_of(seat).dump() for seat in SEATS}
    ranges_seen = 0
    for seat in SEATS:
        got = report("inspect", forgetful, "--seat", seat, "forgotten")
        expected = [
            {
                "first": str(row["first_key"]),
                "last": str(row["last_key"]),
                "first_day": row["first_day"],
                "first_policy": row["first_policy"],
                "last_day": row["last_day"],
                "last_policy": row["last_policy"],
                "passes": row["passes"],
                "forced": str(row["first_policy"]).startswith("host:")
                or str(row["last_policy"]).startswith("host:"),
            }
            for row in rows(dumps[seat], "forgotten")
        ]
        assert got["ranges"] == expected, seat
        ranges_seen += len(expected)
        passes = [
            {**row, "trigger": str(row["trigger"])} for row in rows(dumps[seat], "retention_passes")
        ]
        assert got["passes"] == list(reversed(passes)), seat
        assert all("prose_contradictions" in each for each in passes)
    assert ranges_seen > 0, "the forgetful home forgot nothing"


def test_bytes_are_the_dumps_lines(synthetic: Path) -> None:
    # AC-e-5: per table, the report's bytes are the sum of that table's line lengths in the seat view dump; their
    # sum is `size()`, less `meta` is `content_size()`; the home's totals are the home dump's segments.
    with MemoryHome.open_read_only(synthetic) as home:
        dumps = {seat: home.view_of(seat).dump() for seat in SEATS[:3]}
        home_dump = home.dump()
        page_size = home.connection.execute("PRAGMA page_size").fetchone()[0]
        pages = home.connection.execute("PRAGMA page_count").fetchone()[0]
    for seat, dump in dumps.items():
        got = BytesReport.model_validate_json(
            ok("inspect", synthetic, "--seat", seat, "bytes", "--json")
        )
        sized = got.seat
        assert sized is not None
        lines: dict[str, list[int]] = {table.name: [] for table in VIEW_TABLES}
        for text in dump.decode("utf-8").splitlines():
            lines[json.loads(text)[0]].append(len(text.encode("utf-8")) + 1)
        tables = {each.table: each for each in sized.tables}
        assert list(tables) == [table.name for table in VIEW_TABLES]
        for name, sizes in lines.items():
            assert (tables[name].rows, tables[name].bytes) == (len(sizes), sum(sizes)), name
            assert tables[name].scope == "person"
        assert sized.size == len(dump) == sum(sum(sizes) for sizes in lines.values())
        assert sized.content_size == len(dump) - sum(lines["meta"])
        largest = [each.bytes for each in sized.largest]
        assert largest == sorted((s for sizes in lines.values() for s in sizes), reverse=True)[:5]
        bound = sized.bound
        assert (bound.mib, bound.bound, bound.target, bound.source) == (
            20,
            20 * 1_048_576,
            15 * 1_048_576,
            "default",
        )
        assert bound.trigger == bound.bound - bound.bound // 16
    whole = BytesReport.model_validate_json(ok("inspect", synthetic, "bytes", "--json"))
    assert whole.seat is None
    totals = whole.home
    assert totals.dump == len(home_dump)
    assert totals.file == pages * page_size
    segments = totals.home_meta + totals.fact + totals.fact_who
    assert segments + sum(each.bytes for each in totals.seats) == len(home_dump)
    assert [each.seat for each in totals.seats] == sorted(SEATS)


def test_the_bound_comes_from_the_decision_json(synthetic: Path, tmp_path: Path) -> None:
    decisions = tmp_path / "decisions.json"
    document = {
        "format": 2,
        "world": "synthetic",
        "digest": "0" * 64,
        "seats": [
            {
                "seat": "alice",
                "model": {"id": "lm", "assigned": "lm"},
                "memory": {"policy": "consolidate", "store_bound_mib": 64},
            }
        ],
    }
    decisions.write_text(json.dumps(document), encoding="utf-8")
    got = report("inspect", synthetic, "--seat", "alice", "bytes", "--decisions", decisions)
    seat = got["seat"]
    assert isinstance(seat, dict)
    assert seat["bound"]["mib"] == 64 and seat["bound"]["source"] == "given"
    refused = cli("inspect", synthetic, "--seat", "bob", "bytes", "--decisions", decisions)
    assert refused.returncode == 2
    assert refused.stderr.decode("utf-8").startswith("inspect:")


def test_access_counts_and_one_id_with_the_perceived_stream(
    synthetic: Path, tmp_path: Path
) -> None:
    perceived = tmp_path / "alice.jsonl"
    perceived.write_text(
        "".join(one.model_dump_json() + "\n" for one in streams()[SEATS[0]]), encoding="utf-8"
    )
    counts = report("inspect", synthetic, "--seat", "alice", "access")
    assert counts["perceived"] == "default"
    held = counts["counts"]
    assert isinstance(held, dict) and held["total"] > 0
    first = streams()[SEATS[0]][0].id
    one = report(
        "inspect",
        synthetic,
        "--seat",
        "alice",
        "access",
        "--event",
        first,
        "--perceived",
        perceived,
    )
    assert one["perceived"] == "given"
    explanation = one["explanation"]
    assert isinstance(explanation, dict) and explanation["kind"] in ("held", "merged")
    assert f"--person alice --since {int(first) - 1}" in str(one["show_fact"])
    # Another seat's stream lacks records Alice holds: refused by name.
    other = tmp_path / "bob.jsonl"
    other.write_text(
        "".join(one.model_dump_json() + "\n" for one in streams()[SEATS[1]]), encoding="utf-8"
    )
    refused = cli(
        "inspect", synthetic, "--seat", "alice", "access", "--event", first, "--perceived", other
    )
    assert refused.returncode == 2 and b"is not seat 'alice'" in refused.stderr


def test_an_incoherent_id_exits_3_with_its_report(synthetic: Path, tmp_path: Path) -> None:
    # A unit deleted by hand in a copy: an id Alice perceived is now neither held, merged nor forgotten.
    copy = tmp_path / "broken.sqlite"
    copy.write_bytes(synthetic.read_bytes())
    perceived = tmp_path / "alice.jsonl"
    perceived.write_text(
        "".join(one.model_dump_json() + "\n" for one in streams()[SEATS[0]]), encoding="utf-8"
    )
    with MemoryHome.open_read_only(copy) as home:
        held = {str(row["event_id"]) for row in rows(home.view_of(SEATS[0]).dump(), "l0")}
    target = next(one.id for one in streams()[SEATS[0]] if one.id not in held)
    connection = sqlite3.connect(copy)
    seat = connection.execute("SELECT seat FROM seats WHERE key = 'alice'").fetchone()[0]
    key = target.rjust(20, "0")
    for table in ("episodes", "chapter_ranges"):
        connection.execute(
            f"DELETE FROM {table} WHERE seat = ? AND ? BETWEEN first_key AND last_key", (seat, key)
        )
    connection.commit()
    connection.close()
    done = cli(
        "inspect",
        copy,
        "--seat",
        "alice",
        "access",
        "--event",
        target,
        "--perceived",
        perceived,
        "--json",
    )
    assert done.returncode == 3, done.stderr
    assert json.loads(done.stdout)["explanation"]["kind"] == "incoherent"


def test_refusals_are_named_and_exit_2(synthetic: Path, tmp_path: Path) -> None:
    missing = tmp_path / "absent.sqlite"
    done = cli("inspect", missing, "--seat", "alice", "access")
    assert done.returncode == 2 and done.stderr.startswith(b"memory.home:")
    assert not missing.exists()
    done = cli("inspect", synthetic, "--seat", "nobody", "access")
    assert done.returncode == 2 and b"has no seat 'nobody'" in done.stderr
    done = cli("inspect", synthetic, "access")
    assert done.returncode == 2 and done.stderr.startswith(b"inspect:")
