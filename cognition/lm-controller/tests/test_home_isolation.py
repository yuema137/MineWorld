"""Persons are isolated, and the home is a function of its seats (pr-s28a C6; AC-a-4, I-S28-3, I-S28-5).

A seat's dump and sections are the same whatever its home-mates' policies forget, compared with the seat alone in
its own home. The shared home's own dump is the same for every interleaving of the seats' frames and every frame
size, although the seats join the home in a different order each time.
"""

from __future__ import annotations

import pytest
from home_harness import SYNTHETIC, alone, consolidate, home_arm, none_arm, synthetic
from home_streams import SEATS, queries, world
from memory_fixtures import INSTANCE, rows
from mineworld_sdk.wire.ids import EntityKey
from retention_driver import retention

from mineworld_cognition.memory import MemoryHome, RecallQuery, SeatView, ingest, retrieve
from mineworld_cognition.memory.retention.selection import Retention

ALICE = SEATS[0]


@pytest.mark.parametrize(
    ("policy", "parameters"),
    [
        ("keep-all", {}),
        ("decay", {"threshold": 1_000}),
        ("importance-recency-cap", {"k_l0": 0, "k_l1": 0, "k_l2": 0, "k_l3": 0}),
    ],
)
def test_a_seat_is_the_same_whatever_its_home_mates_forget(
    policy: str, parameters: dict[str, int]
) -> None:
    # AC-a-4: Alice under `consolidate`, her eleven home-mates under a policy that keeps everything, forgets
    # almost everything, or keeps nothing; Alice's dump and sections at every point equal Alice alone.
    mates = retention(policy, None, **parameters)

    def policies(seat: EntityKey) -> Retention:
        return retention("consolidate") if seat == ALICE else mates

    asked = frozenset({ALICE})
    by_herself = none_arm({ALICE: synthetic(256)[ALICE]}, 256, SYNTHETIC, consolidate, asked=asked)
    shared = home_arm(synthetic(256), 256, 1, SYNTHETIC, policies, asked=asked)
    assert len(by_herself.snapshots[ALICE]) >= 9
    assert shared.snapshots[ALICE] == by_herself.snapshots[ALICE]
    changed = [
        mate
        for mate in SEATS[1:]
        if shared.snapshots[mate][-1].dump != alone(256).snapshots[mate][-1].dump
    ]
    assert changed, "the home-mates' own policy changed their own memory"


def test_a_seat_view_shows_none_of_a_home_mates_rows() -> None:
    # I-S28-1, I-S28-3 by name: two seats of one home with disjoint streams (the world's facts alternately, so
    # each learns other names, holds other facts, closes other episodes). Each seat's dump and sections equal
    # the same seat alone in a home of its own, and neither dump names a fact only the other perceived. A
    # seat leak (a read or a write without its seat predicate) fails here, not only in the harness.
    heard = world(7, 3)
    two = SEATS[:2]
    streams = {
        seat: [f for n, (f, _) in enumerate(heard) if n % 2 == k] for k, seat in enumerate(two)
    }

    def sections(view: SeatView, seat: EntityKey) -> list[str]:
        now = streams[seat][-1].at
        return [
            retrieve(view, RecallQuery(at=at, counterparts=who, words=words)).render()
            for at, who, words in queries(7, SYNTHETIC.index(seat), 0, now, SYNTHETIC.people)[:40]
        ]

    alone: dict[EntityKey, tuple[bytes, list[str]]] = {}
    for seat in two:
        with MemoryHome.open(":memory:", INSTANCE) as home:
            view = home.seat(SYNTHETIC.identity(seat))
            ingest(view, streams[seat], streams[seat][-1].id)
            alone[seat] = (view.dump(), sections(view, seat))
    with MemoryHome.open(":memory:", INSTANCE) as home:
        views = {seat: home.seat(SYNTHETIC.identity(seat)) for seat in two}
        for seat in two:
            ingest(views[seat], streams[seat], streams[seat][-1].id)
        for seat, other in (two, two[::-1]):
            dump = views[seat].dump()
            assert (dump, sections(views[seat], seat)) == alone[seat], seat
            mine = {f.id for f in streams[seat]}
            theirs = [f.id for f in streams[other] if f.id not in mine]
            held = {str(row["event_id"]) for row in rows(dump, "l0")}
            assert held.isdisjoint(theirs), seat


def test_the_home_dump_is_the_same_for_every_interleaving_and_frame() -> None:
    # I-S28-5: the home's own dump is a function of each seat's perceived prefix only. Seats join the home at
    # their first frame, so its internal seat numbers differ between these runs. Its mutation (M-a-9) orders
    # the home dump's seats by that number instead of their key.
    homes = {
        home_arm(synthetic(frame), frame, seed, SYNTHETIC, consolidate, asked=frozenset()).home
        for frame, seed in ((256, 1), (256, 2), (4096, 3))
    }
    assert len(homes) == 1
