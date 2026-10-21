"""Explaining why a seat can or cannot recall an event (pr-s28e §5.3; AC-e-2, the classes of AC-e-3; C-7).

The oracles are the seats' perceived streams and their seat view dumps, read back by the test; the explanation of
the code under test is compared with them, never with itself.
"""

from __future__ import annotations

from collections import Counter
from pathlib import Path

import pytest
from home_streams import CAST, SEATS
from inspect_fixtures import Oracle, build, streams
from memory_fixtures import ALICE, BOB, CAFE, INSTANCE, PARK, arrived, rows, spoke
from mineworld_sdk.wire.ids import EntityKey, EventId
from retention_stream import draw

from mineworld_cognition.memory.explain import explain, held_counts
from mineworld_cognition.memory.home import MemoryHome, SeatView
from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.records import StoreIdentity


@pytest.fixture(scope="module")
def synthetic(tmp_path_factory: pytest.TempPathFactory) -> Path:
    path = tmp_path_factory.mktemp("synthetic") / "memory.sqlite"
    build(path)
    return path


def _how(seat: EntityKey, subjects: list[str], participants: list[str]) -> str:
    me = CAST[seat]
    if me in subjects:
        return "subject"
    if me in participants:
        return "participant"
    return "witness"


def test_held_counts_recount_the_stream_by_role_and_salience(synthetic: Path) -> None:
    # AC-e-2 (R-S28e-a-1): every seat of the synthetic world, against the dump's `l0` and the stream's roles.
    with MemoryHome.open_read_only(synthetic) as home:
        for seat in SEATS:
            view = home.view_of(seat)
            facts = {one.id: one for one in streams()[seat]}
            expected: Counter[tuple[str, str]] = Counter()
            for row in rows(view.dump(), "l0"):
                one = facts[EventId(str(row["event_id"]))]
                how = _how(seat, list(one.subjects), list(one.participants))
                expected[(how, str(row["salience"]))] += 1
            counts = held_counts(view)
            got = {(c.how, c.salience): c.records for c in counts.counts if c.records}
            assert got == dict(expected), seat
            assert counts.total == sum(expected.values())
            meta = {str(row["key"]): row["value"] for row in rows(view.dump(), "meta")}
            assert counts.cursor == str(meta["cursor"])
            assert counts.watermark == (
                None if meta["forgotten_below"] is None else str(meta["forgotten_below"])
            )


def test_every_sampled_id_is_classed_as_the_oracle_classes_it(synthetic: Path) -> None:
    # C-7 on the synthetic world: per seat, 300 perceived ids and 100 it never perceived, drawn by SplitMix64.
    seen: Counter[str] = Counter()
    with MemoryHome.open_read_only(synthetic) as home:
        for index, seat in enumerate(SEATS):
            view = home.view_of(seat)
            oracle = Oracle.of(view.dump())
            mine = [one.id for one in streams()[seat]]
            perceived = frozenset(mine)
            everyone = sorted({one.id for facts in streams().values() for one in facts}, key=int)
            others = [event for event in everyone if event not in perceived]
            sample = [mine[draw(28, index * 1000 + n, 1) % len(mine)] for n in range(300)]
            sample += [others[draw(28, index * 1000 + n, 2) % len(others)] for n in range(100)]
            for event in sample:
                got = explain(view, event, perceived)
                assert got.kind == oracle.classify(event, perceived), (seat, event)
                seen[got.kind] += 1
    assert seen["incoherent"] == 0
    assert {"held", "merged", "never-perceived", "covered-not-perceived"} <= set(seen), seen


def _two_seats(home: MemoryHome) -> tuple[SeatView, SeatView]:
    alice = home.seat(
        StoreIdentity(world_instance=INSTANCE, seat=EntityKey("alice"), self_entity=ALICE)
    )
    bob = home.seat(StoreIdentity(world_instance=INSTANCE, seat=EntityKey("bob"), self_entity=BOB))
    return alice, bob


def test_each_class_on_a_hand_built_seat() -> None:
    facts = [
        spoke(10, 9 * 3600, BOB, ALICE, "Morning.", CAFE),
        arrived(12, 9 * 3600 + 60, ALICE, CAFE),
        spoke(14, 9 * 3600 + 120, ALICE, BOB, "Coffee?", CAFE),
        arrived(20, 15 * 3600, ALICE, PARK),
    ]
    with MemoryHome.open(":memory:", INSTANCE) as home:
        alice, _ = _two_seats(home)
        ingest(alice, facts, EventId("30"))
        perceived = frozenset(one.id for one in facts)
        held = explain(alice, EventId("10"), perceived)
        assert held.kind == "held"
        assert held.how == "subject"  # Bob spoke to Alice: she is the listener, the subject
        assert held.access is not None and held.access.salience == "notable"
        assert held.record is not None and [w.entity for w in held.record.who] == [BOB, ALICE]
        assert explain(alice, EventId("14"), perceived).how == "participant"
        assert explain(alice, EventId("31"), perceived).kind == "not-yet-delivered"
        assert explain(alice, EventId("25"), perceived).kind == "never-perceived"
        assert explain(alice, EventId("25")).kind == "never-perceived"
        # 11 lies inside the morning episode's range (10 … 14) and was never perceived.
        assert explain(alice, EventId("11"), perceived).kind == "covered-not-perceived"
        assert explain(alice, EventId("11")).kind == "merged"
        with alice.transaction():
            alice.release(["00000000000000000012"])
        merged = explain(alice, EventId("12"), perceived)
        assert merged.kind == "merged" and merged.units[0].unit == "episode"
        with alice.transaction() as db:
            alice.release(["00000000000000000020"])
            db.execute(
                "DELETE FROM episodes WHERE seat = ? AND first_key = ?",
                (alice.seat_id, "00000000000000000020"),
            )
            db.execute(
                "INSERT INTO forgotten VALUES (?, ?, ?, 0, 'consolidate', 0, 'host:consolidate', 2)",
                (alice.seat_id, "00000000000000000020", "00000000000000000020"),
            )
        forgotten = explain(alice, EventId("20"), perceived)
        assert forgotten.kind == "forgotten"
        assert forgotten.forgotten is not None and forgotten.forgotten.forced
        assert forgotten.forgotten.last_policy == "host:consolidate"
        with alice.transaction() as db:
            db.execute("DELETE FROM forgotten WHERE seat = ?", (alice.seat_id,))
            alice.set_meta("forgotten_below", "00000000000000000020")
        assert explain(alice, EventId("20"), perceived).kind == "forgotten-below-watermark"
        with alice.transaction():
            alice.set_meta("forgotten_below", "")
        assert explain(alice, EventId("20"), perceived).kind == "incoherent"


def test_a_home_mates_units_never_explain_this_seats_ids() -> None:
    # I-S28-1 in `covering`: Alice's episode covers 11; Bob, who perceived only 40, never perceived 11.
    with MemoryHome.open(":memory:", INSTANCE) as home:
        alice, bob = _two_seats(home)
        ingest(
            alice,
            [spoke(10, 9 * 3600, BOB, ALICE, "Hi.", CAFE), arrived(12, 9 * 3600 + 9, ALICE, CAFE)],
            EventId("12"),
        )
        ingest(bob, [arrived(40, 10 * 3600, BOB, PARK)], EventId("40"))
        assert explain(alice, EventId("11")).kind == "merged"
        assert explain(bob, EventId("11"), frozenset({EventId("40")})).kind == "never-perceived"
        assert explain(bob, EventId("11")).units == ()
        assert explain(bob, EventId("10")).kind == "never-perceived"
