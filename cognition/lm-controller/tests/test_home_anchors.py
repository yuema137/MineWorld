"""AC-d-3: schema 6 keeps each held fact's payload anchors beside its access row (pr-s28d §5.3, §5.8).

The arm is `lakeside-like` (the second world): three seats in one home, a week of facts through `absorb` with
`consolidate`. The oracles are the anchor oracle's scan of each fact the seat was given, and the dumps.
"""

from __future__ import annotations

import json
from collections.abc import Generator

import anchor_oracle
import pytest
from anchor_streams import LAKE_PEOPLE, LAKE_SEATS, lakeside, streams
from memory_fixtures import INSTANCE
from retention_driver import days, feed, retention

from mineworld_cognition.memory.home import MemoryHome, SeatView
from mineworld_cognition.memory.records import StoreIdentity, event_key, event_of
from mineworld_cognition.memory.retention.items import extract

SEED, DAYS = 11, 7


GIVEN = streams(lakeside(SEED, DAYS))


@pytest.fixture(scope="module")
def home() -> Generator[MemoryHome]:
    with MemoryHome.open(":memory:", INSTANCE) as opened:
        for seat, person in zip(LAKE_SEATS, LAKE_PEOPLE, strict=True):
            view = opened.seat(
                StoreIdentity(world_instance=INSTANCE, seat=seat, self_entity=person)
            )
            feed(view, days(GIVEN[seat]), retention("consolidate"))
        yield opened


def _views(home: MemoryHome) -> list[SeatView]:
    return [home.view_of(seat) for seat in LAKE_SEATS]


def _tagged(dump: bytes, tag: str) -> list[str]:
    return [line for line in dump.decode().splitlines() if line.startswith(f'["{tag}",')]


def _home_section(home: MemoryHome, seat: str) -> bytes:
    lines = home.dump().decode().splitlines()
    start = lines.index(next(line for line in lines if line.startswith(f'["seat","{seat}",')))
    rest = lines[start + 1 :]
    end = next((i for i, line in enumerate(rest) if line.startswith('["seat",')), len(rest))
    return ("\n".join(rest[:end]) + "\n").encode()


def test_each_anchor_is_one_line_with_equal_bytes_in_the_view_and_the_home(
    home: MemoryHome,
) -> None:
    total = 0
    for view in _views(home):
        mine = _tagged(view.dump(), "access_anchor")
        assert mine == _tagged(_home_section(home, view.identity.seat), "access_anchor")
        total += len(mine)
    assert total > 0


def test_the_anchor_rows_are_the_oracles_scan_of_each_held_fact(home: MemoryHome) -> None:
    for view in _views(home):
        wire = {f.id: f.model_dump(mode="json") for f in GIVEN[view.identity.seat]}
        expected = {
            (int(event), entity, kind)
            for event in wire
            if view.holds(event_key(event))
            for entity, kind in anchor_oracle.scan(wire[event]["payload"]["payload"])
        }
        got = {
            (cells[1], cells[2], cells[3])
            for cells in (json.loads(line) for line in _tagged(view.dump(), "access_anchor"))
        }
        assert got == expected


def test_the_meter_and_the_extraction_equal_the_dump(home: MemoryHome) -> None:
    for view in _views(home):
        meta = sum(size for _, size in view.row_bytes("meta"))
        assert view.content_size() == len(view.dump()) - meta
        assert extract(view).total == len(view.dump())


def test_release_removes_the_anchors_with_the_access_row() -> None:
    given = streams(lakeside(SEED, 1))
    seat, person = LAKE_SEATS[0], LAKE_PEOPLE[0]
    with MemoryHome.open(":memory:", INSTANCE) as opened:
        view = opened.seat(StoreIdentity(world_instance=INSTANCE, seat=seat, self_entity=person))
        feed(view, [given[seat]], retention("keep-all"))
        before = view.content_size()
        rows: list[tuple[str]] = view.connection.execute(
            "SELECT DISTINCT event_key FROM access_anchor WHERE seat = ?", (view.seat_id,)
        ).fetchall()
        keys = [key for (key,) in rows]
        assert keys
        with view.transaction():
            view.release(keys)
        assert not view.connection.execute(
            "SELECT 1 FROM access_anchor WHERE seat = ?", (view.seat_id,)
        ).fetchall()
        assert all(not view.holds(event_key(event_of(key))) for key in keys)
        meta = sum(size for _, size in view.row_bytes("meta"))
        assert view.content_size() == len(view.dump()) - meta < before
