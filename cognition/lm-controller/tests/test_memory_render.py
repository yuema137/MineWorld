"""AP4-4 (a) to (c), (e): what a perceived fact becomes in memory, and that an unknown pack is remembered."""

from __future__ import annotations

import json

from memory_fixtures import (
    ALICE,
    BOB,
    CAFE,
    CAROL,
    DEV,
    alice_identity,
    arrived,
    entered,
    fact,
    names_of_everyone,
    relationship_changed,
    spoke,
)
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EventId

from mineworld_cognition.memory.ingest import IngestReport, ingest
from mineworld_cognition.memory.store import MemoryStore


def _rows(dump: bytes, table: str) -> list[dict[str, object]]:
    rows = [json.loads(line) for line in dump.decode("utf-8").splitlines()]
    return [row for row in rows if row["table"] == table]


def _gists(facts: list[PerceivedEvent]) -> tuple[dict[str, str], IngestReport]:
    with MemoryStore.open(":memory:", alice_identity()) as store:
        ingest(store, names_of_everyone(), EventId("4"))
        report = ingest(store, facts, facts[-1].id)
        return {str(r["event_id"]): str(r["gist"]) for r in _rows(store.dump(), "l0")}, report


def test_speech_renders_as_heard_said_and_overheard() -> None:
    # AP4-4 (a): names come from the `named` facts ingested first.
    gists, report = _gists(
        [
            spoke(10, 100, BOB, ALICE, "Hello.", CAFE),
            spoke(11, 101, ALICE, BOB, "Hi, Bob.", CAFE),
            spoke(12, 102, BOB, CAROL, "Coffee?", CAFE),
            spoke(13, 103, EntityId("7999999999"), ALICE, "Who am I?", CAFE),
        ]
    )
    assert gists["10"] == "Bob Achterberg said to me: «Hello.»"
    assert gists["11"] == "I said to Bob Achterberg: «Hi, Bob.»"
    assert gists["12"] == "Bob Achterberg said to Carol Mensah: «Coffee?»"
    assert gists["13"] == "someone said to me: «Who am I?»"
    assert gists["1"] == "I am called Alice Moreau"
    assert gists["2"] == "I learned the name Bob Achterberg"
    assert report.decode_fallbacks == 0


def test_quoted_words_cannot_close_the_quote_or_break_the_line() -> None:
    # AP4-4 (b).
    gists, _ = _gists([spoke(10, 100, BOB, ALICE, "a » b « c\nd", CAFE)])
    assert gists["10"] == 'Bob Achterberg said to me: «a " b " c d»'


def test_presence_of_others_is_ambient_and_mine_is_notable() -> None:
    with MemoryStore.open(":memory:", alice_identity()) as store:
        ingest(store, names_of_everyone(), EventId("4"))
        ingest(
            store,
            [arrived(10, 100, BOB, CAFE), entered(11, 101, ALICE, CAROL, CAFE)],
            EventId("11"),
        )
        rows = {r["event_id"]: r for r in _rows(store.dump(), "l0")}
    assert (rows["10"]["gist"], rows["10"]["salience"]) == ("Bob Achterberg arrived", "ambient")
    assert (rows["11"]["gist"], rows["11"]["salience"], rows["11"]["mine"]) == (
        "I went to somewhere",
        "notable",
        1,
    )


def test_an_event_type_no_code_knows_is_remembered_generically() -> None:
    # AP4-4 (c), I-9: a `fished` fact from a pack this code has never seen is ingested, rendered,
    # covered by an episode and citable; no exception and no fallback warning. Its mutation raises on
    # an unknown type.
    fished = fact(
        10,
        100,
        "fished",
        {"catch": "trout", "rod": {"entity": "123", "entity_type": "item"}},
        subjects=[BOB],
        participants=[BOB, DEV],
        place=CAFE,
    )
    with MemoryStore.open(":memory:", alice_identity()) as store:
        ingest(store, names_of_everyone(), EventId("4"))
        report = ingest(store, [fished, spoke(11, 4000, BOB, ALICE, "later", CAFE)], EventId("11"))
        dump = store.dump()
    (row,) = [r for r in _rows(dump, "l0") if r["event_id"] == "10"]
    assert row["gist"] == "fished at somewhere, with Bob Achterberg, Dev Raman"
    assert row["salience"] == "notable"
    assert report.decode_fallbacks == 0
    episodes = [r for r in _rows(dump, "episodes") if r["closed"] == 1]
    assert any(
        str(e["first_key"]) <= "00000000000000000010" <= str(e["last_key"]) for e in episodes
    )


def test_a_payload_a_plugin_cannot_decode_falls_back_and_is_counted() -> None:
    # AP4-4 (e): a `spoke` without its utterance.
    broken = fact(
        10,
        100,
        "spoke",
        {"speaker": {"entity": BOB, "entity_type": "person"}},
        subjects=[ALICE],
        participants=[BOB, ALICE],
        place=CAFE,
    )
    gists, report = _gists([broken])
    assert gists["10"] == "spoke at somewhere, with me and Bob Achterberg"
    assert report.decode_fallbacks == 1


def test_a_change_of_my_level_is_rendered_from_my_side() -> None:
    gists, _ = _gists(
        [
            relationship_changed(10, 100, ALICE, BOB, "acquaintance", "friendly", CAFE),
            relationship_changed(11, 100, BOB, ALICE, "acquaintance", "friendly", CAFE),
        ]
    )
    assert gists["10"] == "I now feel friendly towards Bob Achterberg"
    assert gists["11"] == "Bob Achterberg now feels friendly towards me"
