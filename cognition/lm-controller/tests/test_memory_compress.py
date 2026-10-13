"""AP4-5 (the L1 rule), AP4-6 (L2 and L3), AP4-8 (`StructuralSummarizer`), and batch independence.

Every expected value is a literal computed by hand from the facts written here.
"""

from __future__ import annotations

import json

from memory_fixtures import (
    ALICE,
    BOB,
    CAFE,
    CAROL,
    DAY,
    DEV,
    PARK,
    alice_identity,
    arrived,
    fact,
    names_of_everyone,
    relationship_changed,
    spoke,
)
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EventId, EventTypeId

from mineworld_cognition.compress.summarize import StructuralSummarizer
from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import Episode, L0Record
from mineworld_cognition.memory.store import MemoryStore

T = 10 * 3600  # 10:00 on day 1


def _dump(facts: list[PerceivedEvent], frame: int | None = None) -> bytes:
    with MemoryStore.open(":memory:", alice_identity()) as store:
        size = frame or len(facts)
        for start in range(0, len(facts), size):
            batch = facts[start : start + size]
            ingest(store, batch, batch[-1].id)
        return store.dump()


def _rows(dump: bytes, table: str) -> list[dict[str, object]]:
    rows = [json.loads(line) for line in dump.decode("utf-8").splitlines()]
    return [row for row in rows if row["table"] == table]


def _episodes(facts: list[PerceivedEvent]) -> list[tuple[str, str, int]]:
    """(first id, last id, closed) per episode: keys are 20-digit ids, so stripping zeros is exact."""
    return [
        (str(e["first_key"]).lstrip("0"), str(e["last_key"]).lstrip("0"), int(str(e["closed"])))
        for e in _rows(_dump(facts), "episodes")
    ]


def _nowhere(event: int, at: int) -> PerceivedEvent:
    return fact(event, at, "noticed", None, subjects=[BOB], participants=[BOB], place=None)


def test_a_gap_of_1800_seconds_joins_and_1801_splits() -> None:
    facts = [
        spoke(10, T, BOB, ALICE, "a", CAFE),
        spoke(11, T + 1800, BOB, ALICE, "b", CAFE),
        spoke(12, T + 1800 + 1801, BOB, ALICE, "c", CAFE),
    ]
    assert _episodes(facts) == [("10", "11", 1), ("12", "12", 0)]


def test_a_place_change_and_a_day_boundary_split() -> None:
    facts = [
        spoke(10, T, BOB, ALICE, "a", CAFE),
        spoke(11, T + 10, BOB, ALICE, "b", PARK),
        spoke(12, DAY - 100, BOB, ALICE, "c", PARK),
        spoke(13, DAY + 100, BOB, ALICE, "d", PARK),
    ]
    assert _episodes(facts) == [("10", "10", 1), ("11", "11", 1), ("12", "12", 1), ("13", "13", 0)]


def test_a_fact_with_no_place_joins_the_open_episode() -> None:
    facts = [
        spoke(10, T, BOB, ALICE, "a", CAFE),
        _nowhere(11, T + 10),
        spoke(12, T + 20, BOB, ALICE, "b", CAFE),
    ]
    assert _episodes(facts) == [("10", "12", 0)]


def test_others_comings_and_goings_never_split_an_episode() -> None:
    # Carol arriving at the park while Alice talks with Bob at the café joins; an ambient fact opens an
    # episode only when none is open.
    facts = [
        arrived(10, T, CAROL, PARK),
        spoke(11, T + 10, BOB, ALICE, "a", CAFE),
        arrived(12, T + 20, DEV, PARK),
        spoke(13, T + 30, BOB, ALICE, "b", CAFE),
    ]
    assert _episodes(facts) == [("10", "10", 1), ("11", "13", 0)]


def test_an_episodes_counterparts_are_the_union_of_its_records() -> None:
    # Its mutation requires an equal counterpart set (step-17's wording): the second line splits.
    facts = [
        spoke(10, T, BOB, ALICE, "a", CAFE),
        spoke(11, T + 10, CAROL, ALICE, "b", CAFE),
        spoke(12, T + 4000, BOB, ALICE, "c", CAFE),
    ]
    dump = _dump(facts)
    assert [(e["first_key"], e["closed"]) for e in _rows(dump, "episodes")] == [
        ("00000000000000000010", 1),
        ("00000000000000000012", 0),
    ]
    assert [(w["episode"], w["entity"]) for w in _rows(dump, "episode_who")] == [
        (1, BOB),
        (1, CAROL),
        (2, BOB),
    ]


def three_weeks() -> list[PerceivedEvent]:
    """Names, then three weeks at the café and the park; hand-computed episodes in the comments."""
    return [
        *names_of_everyone(),  # E1 [1-4], day 0, no place
        spoke(10, T, BOB, ALICE, "Morning.", CAFE),  # E2 [10-11], day 0
        spoke(11, T + 600, ALICE, BOB, "Hi.", CAFE),
        relationship_changed(12, 2 * DAY + T, ALICE, BOB, "acquaintance", "friendly", CAFE),  # E3
        spoke(13, 9 * DAY + T, CAROL, ALICE, "Hey.", PARK),  # E4 [13-14], day 9 (week 1)
        arrived(14, 9 * DAY + T + 100, BOB, PARK),
        spoke(15, 16 * DAY + T, BOB, ALICE, "Back again.", CAFE),  # E5 [15-16], day 16 (week 2)
        relationship_changed(16, 16 * DAY + T + 60, ALICE, BOB, "friendly", "friend", CAFE),
        spoke(17, 22 * DAY + T, DEV, CAROL, "Psst.", CAFE),  # E6 [17], day 22 (week 3)
        spoke(18, 22 * DAY + T + 4000, DEV, CAROL, "Later.", CAFE),  # E7 [18], open
    ]


def test_weeks_become_chapters_once_past_and_cite_their_episodes() -> None:
    # AP4-6, L2: weeks 0, 1 and 2 are fully past (a closed episode exists in week 3); week 3 is not.
    chapters = _rows(_dump(three_weeks()), "chapters")
    assert [(c["week"], c["first_episode"], c["last_episode"]) for c in chapters] == [
        (0, 1, 3),
        (1, 4, 4),
        (2, 5, 5),
    ]
    assert chapters[0]["text"] == (
        "Week 1: 3 episodes, mostly at somewhere; most often with Bob Achterberg (3), "
        "Carol Mensah (1), Dev Raman (1); first met Bob Achterberg, Carol Mensah, Dev Raman."
    )


def test_stable_facts_about_a_counterpart_match_the_hand_computed_values() -> None:
    # AP4-6, L3, for Bob: named (2) is the first fact naming him; 16 the last, and the newest change of
    # my level. Met in E1, E2, E3, E4 (arrived, ambient) and E5; exchanges are the notable facts with him
    # in which I took part: 10, 11, 12, 15, 16. Its mutation sets first_key on every record.
    stable = {s["entity"]: s for s in _rows(_dump(three_weeks()), "stable")}
    assert stable[BOB] == {
        "table": "stable",
        "entity": BOB,
        "first_key": "00000000000000000002",
        "last_key": "00000000000000000016",
        "times_met": 5,
        "exchanges": 5,
        "level": "friend",
        "level_key": "00000000000000000016",
    }
    assert ALICE not in stable


def test_the_store_does_not_depend_on_how_facts_were_batched() -> None:
    facts = three_weeks()
    whole = _dump(facts)
    assert _dump(facts, frame=1) == whole
    assert _dump(facts, frame=3) == whole


def _names() -> NameBook:
    store = MemoryStore.open(":memory:", alice_identity())
    names = NameBook.load(store)
    for entity, name in ((BOB, "Bob Achterberg"), (CAROL, "Carol Mensah"), (DEV, "Dev Raman")):
        names.learn(entity, name)
    names.learn(CAFE, "the café")
    return names


def _record(
    event: str, at: int, gist: str, who: EntityId, *, mine: bool = False, ambient: bool = False
) -> L0Record:
    return L0Record(
        event=EventId(event),
        at=at,
        kind=EventTypeId("arrived" if ambient else "spoke"),
        place=CAFE,
        salience="ambient" if ambient else "notable",
        mine=mine,
        gist=gist,
        counterparts=(who,),
    )


def test_the_structural_episode_and_chapter_texts_are_exact_and_repeatable() -> None:
    # AP4-8. Its mutation adds a wall-clock timestamp: the second call differs.
    day_12 = 11 * DAY + 9 * 3600
    records = [
        _record("10", day_12 + 600, "Bob Achterberg said to me: «Rain later?»", BOB),
        _record("11", day_12 + 1200, "I said to Bob Achterberg: «Maybe.»", BOB, mine=True),
        _record("12", day_12 + 1800, "Carol Mensah arrived", CAROL, ambient=True),
        _record("13", day_12 + 2400, "Bob Achterberg said to me: «Bye.»", BOB),
    ]
    episodes = [
        Episode(
            number=n,
            day=day,
            place=place,
            first=EventId(f"{n}0"),
            last=EventId(f"{n}9"),
            opened_at=day * DAY,
            closed_at=day * DAY,
            closed=True,
            records=3,
            counterparts=who,
            text="",
            prose=None,
        )
        for n, day, place, who in (
            (1, 14, CAFE, (BOB, CAROL)),
            (2, 15, CAFE, (BOB,)),
            (3, 20, PARK, (DEV,)),
        )
    ]
    summarizer, names = StructuralSummarizer(), _names()
    episode = summarizer.episode(records, names, place=CAFE)
    chapter = summarizer.chapter(episodes, names, newcomers=[DEV])
    assert episode == (
        "Day 12, 09:10\N{EN DASH}09:40, the café: with Bob Achterberg; 3 notable moments, 1 mine; "
        "1 other came and went; Bob Achterberg said to me: «Bye.»"
    )
    assert chapter == (
        "Week 3: 3 episodes, mostly at the café; most often with Bob Achterberg (2), Carol Mensah "
        "(1), Dev Raman (1); first met Dev Raman."
    )
    assert summarizer.episode(records, names, place=CAFE) == episode
    assert summarizer.chapter(episodes, names, newcomers=[DEV]) == chapter
