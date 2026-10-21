"""Forgotten-range attribution (pr-s28b §§5.2, 5.3, 6.1; AC-b-1, AC-b-2; step-28 §5.6).

The literal cases build a seat view by hand — forgotten ranges and surviving episodes inserted row by row, as
`test_retention_plan.py` does — and run `normalize` as the passes would, one `PassScope` per pass; every expected
row is written in the test. The backstop case runs the host bound on a small ingested store with a test-only
tighter limit (`Tight`), so its forced drops happen without a 20 MiB stream. The pass-level cases run the synthetic
12-seat world in one shared home under the three policies of AC-b-1 (`capture_s28b_parity.shared`) and compare
each seat view, with S28-b's columns removed, with the digests frozen at C0 from S28-a's merged code.
"""

from __future__ import annotations

import hashlib
import json
from collections.abc import Callable
from dataclasses import dataclass

import pytest
from capture_s28b_parity import GOLDEN, POLICIES, parity_view, shared
from memory_fixtures import seat_view
from mineworld_sdk.wire.ids import EventId
from retention_driver import DAY
from retention_oracle import Dump
from retention_stream import stream

from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.retention.bound import StoreBounds, enforce
from mineworld_cognition.memory.retention.forgotten import (
    MAX_RANGES,
    Forgetter,
    ForgottenSet,
    PassScope,
    add,
    normalize,
)

HOST_STEPS = {"host:consolidate", "host:chapters", "host:chapterless-episodes"}


def _key(n: int) -> str:
    return str(n).rjust(20, "0")


def _episode(store: SeatView, number: int, first: int, last: int) -> None:
    store.connection.execute(
        "INSERT INTO episodes (seat, episode, day, place, first_key, last_key, opened_at, closed_at, "
        "closed, records, text, prose, prose_key, importance, mine) "
        "VALUES (?, ?, 0, NULL, ?, ?, 0, 0, 1, 1, '', NULL, NULL, 0, 0)",
        (store.seat_id, number, _key(first), _key(last)),
    )


def _pass(store: SeatView, forgetter: Forgetter, *spans: tuple[int, int]) -> PassScope:
    """One pass that forgets `spans` (as a `Drop` would) and normalises once."""
    scope = PassScope.begin(store)
    add(store, [(_key(a), _key(b)) for a, b in spans], forgetter)
    normalize(store, scope)
    return scope


def _rows(store: SeatView) -> list[tuple[object, ...]]:
    found: list[tuple[object, ...]] = store.connection.execute(
        "SELECT first_key, last_key, first_day, first_policy, last_day, last_policy, passes "
        "FROM forgotten WHERE seat = ? ORDER BY first_key",
        (store.seat_id,),
    ).fetchall()
    return [(int(str(r[0])), int(str(r[1])), *r[2:]) for r in found]


def test_a_merge_keeps_the_first_forgetter() -> None:
    # §6.1: a chapter dropped on day 12 by `consolidate` adds its two runs, which nothing separates (one range,
    # one pass); a later adjacent drop on day 40 by `decay` joins it. Its mutation M-b-1 (the newer part's
    # first attribution wins) reads (40, decay, 40, decay).
    with seat_view() as store:
        _episode(store, 1, 300, 310)
        _pass(store, Forgetter(12, "consolidate"), (100, 105), (106, 110))
        assert _rows(store) == [(100, 110, 12, "consolidate", 12, "consolidate", 1)]
        _pass(store, Forgetter(40, "decay"), (111, 120))
        assert _rows(store) == [(100, 120, 12, "consolidate", 40, "decay", 2)]
        (only,) = ForgottenSet.load(store).attributed()
        assert (only.first, only.last, only.passes, only.forced()) == ("100", "120", 2, False)


def test_one_pass_counts_once() -> None:
    # F-S28b-3: within one pass, `normalize` runs several times (after the plan, after each backstop step);
    # the pass adds 1 to the range it grows, however often. Its mutation M-b-3 (1 at every call) reads 3.
    with seat_view() as store:
        _pass(store, Forgetter(3, "decay"), (100, 110))
        scope = PassScope.begin(store)
        add(store, [(_key(111), _key(120))], Forgetter(9, "decay"))
        normalize(store, scope)
        add(store, [(_key(121), _key(130))], Forgetter(9, "host:chapters"))
        normalize(store, scope)
        normalize(store, scope)
        assert _rows(store) == [(100, 130, 3, "decay", 9, "host:chapters", 2)]
        assert ForgottenSet.load(store).attributed()[0].forced()


def test_a_range_left_alone_is_not_counted_again() -> None:
    # A pass that forgets elsewhere leaves an unrelated range's attribution exactly as it was.
    with seat_view() as store:
        _episode(store, 1, 150, 160)
        _pass(store, Forgetter(3, "decay"), (100, 110))
        _pass(store, Forgetter(5, "decay"), (200, 210))
        assert _rows(store) == [
            (100, 110, 3, "decay", 3, "decay", 1),
            (200, 210, 5, "decay", 5, "decay", 1),
        ]


def test_two_ranges_a_later_pass_joins_count_as_an_upper_bound() -> None:
    # F-S28b-2: pass 1 adds A and B, kept apart by a surviving episode; pass 2 drops that episode, and the three
    # ranges merge. `passes` is the documented upper bound: 1 + 1 + 1 = 3 for two distinct passes.
    with seat_view() as store:
        _episode(store, 1, 150, 160)
        _pass(store, Forgetter(3, "consolidate"), (100, 110), (200, 210))
        store.connection.execute("DELETE FROM episodes WHERE seat = ?", (store.seat_id,))
        _pass(store, Forgetter(7, "decay"), (150, 160))
        assert _rows(store) == [(100, 210, 3, "consolidate", 7, "decay", 3)]


def test_the_cap_moves_the_oldest_ranges_into_the_watermark_without_attribution() -> None:
    # §5.2: 300 ranges, each on its own day, kept apart by surviving episodes: the 44 oldest leave, the watermark
    # rises to the 44th's end, and the oldest range left keeps its own attribution (day 44).
    with seat_view() as store:
        for n in range(300):
            _episode(store, n + 1, 1000 * n + 500, 1000 * n + 600)
        scope = PassScope.begin(store)
        for n in range(300):
            add(store, [(_key(1000 * n), _key(1000 * n + 10))], Forgetter(n, "decay"))
        normalize(store, scope)
        forgotten = ForgottenSet.load(store)
    attributed = forgotten.attributed()
    assert len(attributed) == MAX_RANGES
    assert forgotten.watermark() == "43010"
    assert (attributed[0].first, attributed[0].first_day, attributed[0].last_day) == (
        "44000",
        44,
        44,
    )
    assert forgotten.range_of(EventId("43005")) is None
    assert forgotten.range_of(EventId("44005")) == attributed[0]
    assert not forgotten.contains(EventId("500"))


@dataclass(frozen=True)
class Tight(StoreBounds):
    """Test-only: the default bound with the backstop's limit set by the test (`cut`), so the host bound's
    forced drops run on a small store. The bound, the floor and every production figure are untouched."""

    cut: int = 0

    @property
    def target(self) -> int:
        return self.cut

    @property
    def limit(self) -> int:
        return self.cut


def _forced_ranges(days: int, cut: Callable[[int], int]) -> tuple[int, list[list[object]]]:
    """A store of `days` synthetic days, then the backstop at `cut(size)`; the pass day and the ranges."""
    facts = list(stream(11, days, per_day=60))
    with seat_view() as store:
        ingest(store, facts, facts[-1].id)
        now = (facts[-1].at // DAY + 1) * DAY
        bounds = Tight(cut=cut(store.size()))
        with store.transaction():
            enforce(store, now, bounds, PassScope.begin(store))
        return now // DAY, _full_rows(store.dump(), "forgotten")


@pytest.mark.parametrize(
    ("days", "share", "expected"),
    [
        (5, 8, {"host:chapterless-episodes"}),
        (16, 32, {"host:chapters", "host:chapterless-episodes"}),
    ],
)
def test_the_backstop_signs_its_own_drops(days: int, share: int, expected: set[str]) -> None:
    # F-S28b-4: a range the host bound drops says which step dropped it, and the pass's day. Five days hold no
    # chapter, so step 5 drops episodes; sixteen days hold two weeks' chapters, which step 4 drops before step 5
    # drops the third week's episodes. The limit is the store's size over `share` (the steps that merge alone do
    # not reach it). Its mutation M-b-2 (the backstop signs with the pass's policy) leaves no `host:` value.
    day, rows = _forced_ranges(days, lambda size: size // share)
    assert rows
    full = [row[2:] for row in rows]
    assert {policy for each in full for policy in (each[1], each[3])} == expected
    assert all(each[0] == each[2] == day and each[4] == 1 for each in full)


def test_a_pass_signs_with_its_policy_and_day_in_a_shared_home() -> None:
    # AC-b-1 at pass level, for every seat of the synthetic world under the three policies: every range's days
    # are ordered and at most the newest pass day, every policy value is the seat's policy or a backstop step,
    # and `passes` ≥ 1. `consolidate` forgets nothing in 30 days (it drops only chapters older than five years);
    # the other two forget at every seat.
    for policy, parameters in POLICIES.items():
        for seat, dump in shared(policy, parameters).items():
            parsed = Dump.parse(dump)
            newest = max(int(str(row["day"])) for row in parsed.rows("retention_passes"))
            for row in _full_rows(dump, "forgotten"):
                first_day, first_policy, last_day, last_policy, passes = row[2:]
                assert isinstance(first_day, int) and isinstance(last_day, int), (policy, seat)
                assert first_day <= last_day <= newest, (policy, seat, row)
                assert {first_policy, last_policy} <= {policy, *HOST_STEPS}, (policy, seat, row)
                assert isinstance(passes, int) and passes >= 1, (policy, seat, row)
            assert bool(parsed.rows("forgotten")) == (policy != "consolidate"), (policy, seat)


def _full_rows(dump: bytes, table: str) -> list[list[object]]:
    out: list[list[object]] = []
    for line in dump.decode("utf-8").splitlines():
        cells: list[object] = json.loads(line)
        if cells[0] == table:
            out.append(cells[1:])
    return out


@pytest.mark.parametrize("policy", sorted(POLICIES))
def test_attribution_never_changes_what_is_forgotten(policy: str) -> None:
    # AC-b-2 (I-b-2): every seat view's parity view (the dump without what schema 5 necessarily changes, D-b-2)
    # is byte for byte the one S28-a's merged code wrote on the same run (sha256 frozen in
    # `golden/s28b_parity.json`, captured from the unchanged code).
    frozen = json.loads(GOLDEN.read_text(encoding="utf-8"))["views"][policy]
    found = {
        seat: hashlib.sha256(parity_view(dump)).hexdigest()
        for seat, dump in shared(policy, POLICIES[policy]).items()
    }
    assert found == frozen
