"""AFGb-7: retrieval after forgetting (pr-fg-b §5.10).

C2's half, (c) and (d), runs on hand-built stores whose rows the test deletes itself, as `Merge(into=1)`
would; the synthetic-stream half, (a), (b) and (e), follows with the bundled policies.
"""

from __future__ import annotations

from memory_fixtures import (
    ALICE,
    BOB,
    CAFE,
    CAROL,
    DAY,
    names_of_everyone,
    relationship_changed,
    seat_view,
    spoke,
)
from mineworld_sdk.wire.ids import EventId

from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.retrieve import RecallQuery, retrieve

T = 9 * 3600


def _delete_records(store: SeatView, *events: str) -> None:
    """What `Merge(into=1)` removes, for the given facts: the seat releases them (its access rows and index
    entries, and the records with them, since no other seat holds them)."""
    with store.transaction():
        store.release(event.rjust(20, "0") for event in events)


def test_a_stable_fact_renders_with_every_establishing_record_gone() -> None:
    # AFGb-7 (c). Bob: first named by fact 2 (day 1), last and level by fact 20 (day 3). Its mutation
    # restores the L3 line's read of `l0.at`: the deleted rows make it fail.
    facts = [
        *names_of_everyone(),
        spoke(10, T, BOB, ALICE, "Hello.", CAFE),
        relationship_changed(20, 2 * DAY + T, ALICE, BOB, "acquaintance", "friendly", CAFE),
        spoke(30, 3 * DAY + T, CAROL, ALICE, "Later.", CAFE),
    ]
    with seat_view() as store:
        ingest(store, facts, EventId("30"))
        _delete_records(store, "2", "20")
        section = retrieve(store, RecallQuery(at=3 * DAY + T))
    (bob,) = [line for line in section.lines if line.level == "L3" and "Bob" in line.text]
    assert bob.text == (
        "Bob Achterberg: met on day 1, last seen day 3, met 3 times, 2 exchanges, friendly."
    )
    assert bob.citation.token() == "[#2,#20]"


def test_a_word_only_in_a_merged_episodes_text_finds_that_episode() -> None:
    # AFGb-7 (d): episode 2's records are gone; its summary quotes "zeppelin". Its mutation matches only
    # records: the episode is not found.
    facts = [
        *names_of_everyone(),
        spoke(10, T, BOB, ALICE, "The zeppelin lands at noon.", CAFE),
        spoke(11, T + 60, ALICE, BOB, "Really?", CAFE),
        spoke(30, 3 * DAY + T, CAROL, ALICE, "Later.", CAFE),
    ]
    with seat_view() as store:
        ingest(store, facts, EventId("30"))
        _delete_records(store, "10", "11")
        (count,) = store.connection.execute(
            "SELECT count(*) FROM access WHERE seat = ? AND gist LIKE '%zeppelin%'",
            (store.seat_id,),
        ).fetchone()
        assert count == 0
        section = retrieve(store, RecallQuery(at=3 * DAY + T, words="zeppelin"))
    assert [line.citation.token() for line in section.lines if line.level == "L1"] == ["[#10-11]"]
