"""Schema v2 and ingestion under retention (pr-fg-b C2): base importance per renderer (§5.4), the movement
renderer (QFGb-6), AFGb-11 (a), (b) (schema and disk), AFGb-6 (b), (c) (re-delivery after forgetting).

Rows that retention would remove are removed here by the test itself, with plain SQL, so these criteria
hold before the host exists and do not depend on it. Every expected value is a literal.
"""

from __future__ import annotations

import sqlite3
from pathlib import Path

import pytest
from memory_fixtures import (
    ALICE,
    BOB,
    CAFE,
    CAROL,
    DAY,
    INSTANCE,
    arrived,
    became_acquainted,
    conversation_started,
    entered,
    fact,
    names_of_everyone,
    relationship_changed,
    rows,
    seat_view,
    spoke,
    walk_ended,
    walk_started,
)
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EventId

from mineworld_cognition.memory import ConflictingFact, OutOfOrder, ingest
from mineworld_cognition.memory.home import MemoryHome, SeatView, UnsupportedStoreSchema

T = 9 * 3600


def _rows(dump: bytes, table: str) -> list[dict[str, object]]:
    return rows(dump, table)


def _l0(facts: list[PerceivedEvent]) -> dict[str, dict[str, object]]:
    with seat_view() as store:
        ingest(store, names_of_everyone(), EventId("4"))
        ingest(store, facts, facts[-1].id)
        return {str(row["event_id"]): row for row in _rows(store.dump(), "l0")}


def test_each_renderer_sets_the_base_importance_of_the_design_table() -> None:
    # pr-fg-b §5.4's table, literal by literal. Its mutation gives overheard speech the addressed value.
    broken = fact(
        40,
        T + 40,
        "spoke",
        {"speaker": {"entity": BOB, "entity_type": "person"}},
        subjects=[ALICE],
        participants=[BOB, ALICE],
        place=CAFE,
    )
    facts = [
        spoke(10, T, BOB, ALICE, "To me.", CAFE),
        spoke(11, T + 1, ALICE, BOB, "By me.", CAFE),
        spoke(12, T + 2, BOB, CAROL, "Overheard.", CAFE),
        conversation_started(13, T + 3, BOB, ALICE, CAFE),
        conversation_started(14, T + 4, BOB, CAROL, CAFE),
        arrived(15, T + 5, ALICE, CAFE),
        arrived(16, T + 6, BOB, CAFE),
        entered(17, T + 7, ALICE, CAROL, CAFE),
        entered(18, T + 8, BOB, CAROL, CAFE),
        became_acquainted(19, T + 9, ALICE, BOB, CAFE),
        became_acquainted(20, T + 10, BOB, CAROL, CAFE),
        relationship_changed(21, T + 11, BOB, ALICE, "acquaintance", "friendly", CAFE),
        walk_started(22, T + 12, ALICE, CAFE),
        walk_started(23, T + 13, BOB, CAFE),
        walk_ended(24, T + 14, ALICE, "arrived", CAFE),
        walk_ended(25, T + 15, BOB, "stalled", CAFE),
        fact(26, T + 16, "fished", None, subjects=[BOB], participants=[BOB, ALICE], place=CAFE),
        fact(27, T + 17, "fished", None, subjects=[BOB], participants=[BOB], place=CAFE),
        broken,
    ]
    rows = _l0(facts)
    expected = {
        "1": 500,  # named
        "10": 700,
        "11": 600,
        "12": 200,
        "13": 400,
        "14": 150,
        "15": 100,
        "16": 50,
        "17": 100,
        "18": 50,
        "19": 800,
        "20": 300,
        "21": 800,
        "22": 80,
        "23": 30,
        "24": 80,
        "25": 30,
        "26": 500,  # generic, I took part
        "27": 300,  # generic, I did not
        "40": 500,  # a payload the plug-in cannot decode: the generic value
    }
    assert {event: rows[event]["importance"] for event in expected} == expected
    # Schema 4 derives involvement from the record (`me` among the participants) and stores none
    # (F-S28a-7): the column schema v2 added is gone from the view.
    assert all("involved" not in row for row in rows.values())


def test_other_peoples_walks_are_background_and_mine_are_notable() -> None:
    # QFGb-6. Its mutation removes the renderer: Bob's walk is generic and notable.
    rows = _l0(
        [
            walk_started(10, T, ALICE, CAFE),
            walk_started(11, T + 1, BOB, CAFE),
            walk_ended(12, T + 2, BOB, "arrived", CAFE),
            walk_ended(13, T + 3, BOB, "teleported", CAFE),
            walk_ended(14, T + 4, ALICE, "no-route", CAFE),
        ]
    )
    seen = {
        event: (rows[event]["gist"], rows[event]["salience"], rows[event]["mine"])
        for event in ("10", "11", "12", "13", "14")
    }
    assert seen == {
        "10": ("I set out walking", "notable", 1),
        "11": ("Bob Achterberg set out walking", "ambient", 0),
        "12": ("Bob Achterberg arrived after a walk", "ambient", 0),
        "13": ("Bob Achterberg stopped walking", "ambient", 0),
        "14": ("I found no way on a walk", "notable", 1),
    }


@pytest.mark.parametrize("old", ["1", "2"])
def test_an_older_schema_store_is_refused_by_name_and_left_unchanged(
    tmp_path: Path, old: str
) -> None:
    # AFGb-11 (a): a store whose `schema` is 1 (P4's) or 2 (FG-b's; pr-fg-b2 §4.5) is refused, with no
    # migration (QFGb-7); since S28-a by the home, as every per-seat store is (QS28-8; test_home_open.py
    # covers schema 3).
    path = tmp_path / "alice.sqlite"
    connection = sqlite3.connect(path)
    connection.execute("PRAGMA auto_vacuum = INCREMENTAL")
    connection.execute("CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)")
    connection.executemany(
        "INSERT INTO meta (key, value) VALUES (?, ?)",
        [("world_instance", INSTANCE), ("seat", "alice"), ("self_entity", ALICE), ("schema", old)],
    )
    connection.commit()
    connection.close()
    before = path.read_bytes()
    with pytest.raises(UnsupportedStoreSchema) as raised:
        MemoryHome.open(path, INSTANCE)
    assert (raised.value.found, raised.value.wanted) == (old, 4)
    assert path.read_bytes() == before


def test_a_new_store_file_reclaims_freed_pages_incrementally(tmp_path: Path) -> None:
    # AFGb-11 (b), on the home's file. Its mutation omits the pragma: SQLite reports 0 (none).
    path = tmp_path / "alice.sqlite"
    with seat_view(path=path) as store:
        (mode,) = store.connection.execute("PRAGMA auto_vacuum").fetchone()
    assert mode == 2


def _day_two() -> list[PerceivedEvent]:
    """Names (episode 1, day 0); a talk at the café on day 1 (episode 2: 10-12); one fact on day 2
    (episode 3, open), which closes episode 2."""
    return [
        *names_of_everyone(),
        spoke(10, DAY + T, BOB, ALICE, "Morning.", CAFE),
        spoke(11, DAY + T + 60, ALICE, BOB, "Hello.", CAFE),
        spoke(12, DAY + T + 120, BOB, ALICE, "Bye.", CAFE),
        spoke(20, 2 * DAY + T, CAROL, ALICE, "Next day.", CAFE),
    ]


def _merge_episode_records(store: SeatView, first: str, last: str) -> None:
    """What `Merge(into=1)` removes: the seat releases the records (access rows, index entries, and the
    records themselves, which no other seat holds)."""
    held: list[tuple[str]] = store.connection.execute(
        "SELECT event_key FROM access WHERE seat = ? AND event_key BETWEEN ? AND ?",
        (store.seat_id, first.rjust(20, "0"), last.rjust(20, "0")),
    ).fetchall()
    with store.transaction():
        store.release(key for (key,) in held)


def test_a_fact_merged_away_and_delivered_again_is_skipped_and_counted() -> None:
    # AFGb-6 (b), merged half. Its mutation keeps P4's rule: OutOfOrder.
    with seat_view() as store:
        facts = _day_two()
        ingest(store, facts, EventId("20"))
        _merge_episode_records(store, "10", "12")
        before = store.dump()
        report = ingest(store, [spoke(11, DAY + T + 60, ALICE, BOB, "Hello.", CAFE)], EventId("20"))
        assert (report.records, report.redeliveries, report.forgotten_redeliveries) == (0, 0, 1)
        assert store.dump() == before
        assert store.cursor() == "20"


def test_a_forgotten_fact_delivered_again_is_skipped_and_counted() -> None:
    # AFGb-6 (b), forgotten half: episode 2 dropped and its range in the forgotten set.
    with seat_view() as store:
        ingest(store, _day_two(), EventId("20"))
        _merge_episode_records(store, "10", "12")
        seat = store.seat_id
        store.connection.execute("DELETE FROM episode_who WHERE seat = ? AND episode = 2", (seat,))
        store.connection.execute("DELETE FROM episodes WHERE seat = ? AND episode = 2", (seat,))
        store.connection.execute(
            "INSERT INTO forgotten (seat, first_key, last_key) VALUES (?, ?, ?)",
            (seat, "10".rjust(20, "0"), "12".rjust(20, "0")),
        )
        before = store.dump()
        report = ingest(store, [spoke(12, DAY + T + 120, BOB, ALICE, "Bye.", CAFE)], EventId("20"))
        assert report.forgotten_redeliveries == 1
        assert store.dump() == before


def test_order_and_conflicts_are_still_refused() -> None:
    # AFGb-6 (c): [5, 3] in one call; a held id with other content; and a fact below the cursor that
    # lies in no range the store represents or forgot, which was never delivered (P4's rule, unchanged).
    with seat_view() as store:
        with pytest.raises(OutOfOrder):
            ingest(
                store,
                [spoke(5, 100, BOB, ALICE, "a", CAFE), spoke(3, 99, BOB, ALICE, "b", CAFE)],
                EventId("5"),
            )
        ingest(store, [spoke(5, 100, BOB, ALICE, "a", CAFE)], EventId("20"))
        with pytest.raises(ConflictingFact):
            ingest(store, [spoke(5, 100, BOB, ALICE, "changed", CAFE)], EventId("20"))
        with pytest.raises(OutOfOrder):
            ingest(store, [spoke(12, 101, BOB, ALICE, "c", CAFE)], EventId("20"))
        with pytest.raises(OutOfOrder):
            # Held facts out of order in one call are refused too.
            ingest(
                store,
                [spoke(5, 100, BOB, ALICE, "a", CAFE), spoke(5, 100, BOB, ALICE, "a", CAFE)],
                EventId("20"),
            )
