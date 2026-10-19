"""Access, physical against logical, and efficiency (pr-s28a C8; AC-a-1, AC-a-8, AC-a-9; step-28 AS28-1, AS28-9,
AS28-12; I-S28-1, I-a-6 as ruled, QS28a-1).

- AC-a-8, on the synthetic world (frames of 256) and on the real 12-seat stream: after every frame (so at every
  pass point and between them), (a) the shared home's dump is no longer than the sum of the one-seat homes' dumps
  at the same progress of each seat; (b) each one-seat home's dump is at most its seat view's plus
  `K_ACCESS_OVERHEAD_BYTES` per held record; every seat view is within its 20 MiB bound. The largest per-record
  overhead measured is printed beside `K` for the ledger.
- AC-a-1, on the real stream: every key a seat's access index holds, and every citation of 200 seeded queries per
  seat, is in that seat's `mineworld perceived` export.
- AC-a-9, on the real stream before retention, as amended by the operator on 2026-10-18 (option (a), recorded in
  pr-s28a §AC-a-9 and step-11 §19.13 TD-D16 item 13): the seat views hold at most `PER_RECORD_BOUND` (180) bytes
  per held record, normalised to the history (145.5 B measured on 12d's café, plus a 24 % margin). The original
  rule, the shared home at most 0.30 times the twelve schema-3 stores' 11 712 772 B (measured at C1 on the
  schema-3 code and the pre-12d history: `tests/golden/SOURCE.md`), is printed as advisory only.
"""

from __future__ import annotations

import json
from collections.abc import Mapping

import pytest
from capture_golden_renders import GOLDEN
from home_harness import SYNTHETIC, Arm, Cast, Streams, consolidate, home_arm, none_arm, synthetic
from home_streams import Cafe12, queries
from mineworld_sdk.wire.ids import EntityKey

from mineworld_cognition.memory import MemoryHome, RecallQuery, ingest, retrieve
from mineworld_cognition.memory.home.schema import K_ACCESS_OVERHEAD_BYTES as K
from mineworld_cognition.memory.records import event_key

MIB = 1_048_576
BOUND = 20 * MIB
EFFICIENCY = 0.30
"""Advisory since 2026-10-18: the home against C1's schema-3 stores, a ratio of two different histories."""
PER_RECORD_BOUND = 180.0
"""AC-a-9 as amended (operator, 2026-10-18, option (a)): seat-view bytes per held record, normalised to the
history. Measured 145.5 B on 12d's café (bodies, persona, appearance; 4 174 666 B of home); the bound is that
plus a 24 % margin, so a history with more facts moves the measure only as far as its records grow."""


def physical_within_logical(streams: Streams, cast: Cast, frame: int) -> tuple[int, int]:
    """AC-a-8 (a) and (b) after every frame; returns (frames checked, the largest average overhead per held
    record of a one-seat home over its view, in bytes, rounded up)."""
    alone: Arm = none_arm(streams, frame, cast, consolidate, asked=frozenset(), lengths=True)
    largest = 0
    checked = 0
    for seat, measured in alone.lengths.items():
        for each in measured:
            assert each.view <= BOUND, seat
            assert each.home <= each.view + K * each.held, (seat, each)
            if each.held:
                largest = max(largest, (each.home - each.view + each.held - 1) // each.held)
            checked += 1

    def within(home: MemoryHome, done: Mapping[EntityKey, int]) -> None:
        shared = len(home.dump())
        separate = sum(alone.lengths[seat][count - 1].home for seat, count in done.items() if count)
        assert shared <= separate, (shared, separate, dict(done))

    home_arm(streams, frame, 1, cast, consolidate, asked=frozenset(), after_frame=within)
    return checked, largest


def test_sharing_never_costs_disk_and_disk_stays_bounded_on_the_synthetic_world() -> None:
    # Frames of 7, so that a frame ends within 7 facts of every pass point.
    checked, largest = physical_within_logical(synthetic(7), SYNTHETIC, 7)
    print(
        f"\n[AC-a-8 synthetic] {checked} frames; largest average overhead per held record "
        f"{largest} B; K = {K}"
    )
    assert checked > 12 * 30  # every seat, many frames: the checks ran


def _cast(cafe: Cafe12) -> Cast:
    return Cast(cafe.me, cafe.instance, sorted(cafe.me.values()))


@pytest.mark.real_binary
def test_sharing_never_costs_disk_and_disk_stays_bounded_on_the_real_stream(
    social_cafe_12: Cafe12,
) -> None:
    checked, largest = physical_within_logical(social_cafe_12.streams, _cast(social_cafe_12), 64)
    print(
        f"\n[AC-a-8 real] {checked} frames; largest average overhead per held record {largest} B; "
        f"K = {K}"
    )
    assert checked >= 12


@pytest.mark.real_binary
def test_a_seat_holds_and_cites_only_what_it_perceived(social_cafe_12: Cafe12) -> None:
    # AC-a-1 (I-S28-1): the access oracle is each seat's own export, from the binary.
    cast = _cast(social_cafe_12)
    streams = social_cafe_12.streams
    with MemoryHome.open(":memory:", cast.instance) as home:
        views = {seat: home.seat(cast.identity(seat)) for seat in sorted(streams)}
        for seat, facts in streams.items():
            for start in range(0, len(facts), 256):
                batch = facts[start : start + 256]
                ingest(views[seat], batch, batch[-1].id)
        exceptions: list[tuple[str, str]] = []
        for seat, view in views.items():
            perceived = {event_key(fact.id) for fact in streams[seat]}
            held: list[tuple[str]] = view.connection.execute(
                "SELECT event_key FROM access WHERE seat = ?", (view.seat_id,)
            ).fetchall()
            exceptions += [(seat, key) for (key,) in held if key not in perceived]
            now = streams[seat][-1].at
            for at, who, words in queries(7, cast.index(seat), 0, now, cast.people):
                section = retrieve(view, RecallQuery(at=at, counterparts=who, words=words))
                for cited in section.citations():
                    for end in (cited.first, cited.last):
                        if event_key(end) not in perceived:
                            exceptions.append((seat, end))
            assert held, seat
    assert exceptions == []


@pytest.mark.real_binary
def test_the_home_stores_the_world_day_in_under_a_third_of_the_per_seat_bytes(
    social_cafe_12: Cafe12,
) -> None:
    # AC-a-9 (AS28-12): plain ingest, before retention; the baseline is C1's schema-3 measurement.
    baseline = json.loads(GOLDEN.read_text(encoding="utf-8"))["cafe12_schema3"]
    cast = _cast(social_cafe_12)
    streams = social_cafe_12.streams
    with MemoryHome.open(":memory:", cast.instance) as home:
        views = {seat: home.seat(cast.identity(seat)) for seat in sorted(streams)}
        for seat, facts in streams.items():
            ingest(views[seat], facts, facts[-1].id)
        physical = len(home.dump())
        views_bytes = {seat: view.content_size() for seat, view in views.items()}
        held = {seat: len(view.row_bytes("l0")) for seat, view in views.items()}
    days = int(baseline["days"])
    per_record = sum(views_bytes.values()) / sum(held.values())
    per_day = max(views_bytes.values()) / days
    print(
        f"\n[AC-a-9] home {physical} B against {baseline['dump_bytes']} B for the schema-3 stores "
        f"(ratio {physical / int(baseline['dump_bytes']):.3f}); {physical // days} B per world-day; "
        f"seat-view bytes per held record {per_record:.1f}; the busiest seat's {per_day:.0f} B a day, "
        f"so 20 MiB holds {BOUND / per_day:.0f} days of it in full detail"
    )
    assert int(baseline["seats"]) == len(streams)
    # AC-a-9 as amended (operator, 2026-10-18): bytes per held record, not a ratio against C1's schema-3
    # stores, which were measured on another history (the pre-12d café); that ratio is printed above,
    # advisory, against EFFICIENCY.
    assert per_record <= PER_RECORD_BOUND, (per_record, PER_RECORD_BOUND)
