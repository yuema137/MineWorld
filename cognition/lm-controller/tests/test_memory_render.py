"""AP4-4 (a) to (c), (e): what a perceived fact becomes in memory, and that an unknown pack is remembered."""

from __future__ import annotations

from memory_fixtures import (
    ALICE,
    BOB,
    CAFE,
    CAROL,
    DEV,
    arrived,
    entered,
    fact,
    names_of_everyone,
    relationship_changed,
    rows,
    seat_view,
    spoke,
)
from mineworld_sdk.wire.contract import Observation, PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EventId, JsonValue

from mineworld_cognition.controller.attributes import SelfAttributes
from mineworld_cognition.controller.situation import Situation
from mineworld_cognition.memory.ingest import IngestReport, ingest
from mineworld_cognition.memory.names import NameBook


def _rows(dump: bytes, table: str) -> list[dict[str, object]]:
    return rows(dump, table)


def _gists(facts: list[PerceivedEvent]) -> tuple[dict[str, str], IngestReport]:
    with seat_view() as store:
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
    with seat_view() as store:
        ingest(store, names_of_everyone(), EventId("4"))
        ingest(
            store,
            [arrived(10, 100, BOB, CAFE), entered(11, 101, ALICE, CAROL, CAFE)],
            EventId("11"),
        )
        held = {str(r["event_id"]): r for r in _rows(store.dump(), "l0")}
    assert (held["10"]["gist"], held["10"]["salience"]) == ("Bob Achterberg arrived", "ambient")
    assert (held["11"]["gist"], held["11"]["salience"], held["11"]["mine"]) == (
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
    with seat_view() as store:
        ingest(store, names_of_everyone(), EventId("4"))
        report = ingest(store, [fished, spoke(11, 4000, BOB, ALICE, "later", CAFE)], EventId("11"))
        dump = store.dump()
    (row,) = [r for r in _rows(dump, "l0") if r["event_id"] == 10]
    assert row["gist"] == "fished at somewhere, with Bob Achterberg, Dev Raman"
    assert row["salience"] == "notable"
    assert report.decode_fallbacks == 0
    episodes = [r for r in _rows(dump, "episodes") if r["closed"] == 1]
    assert any(int(str(e["first_key"])) <= 10 <= int(str(e["last_key"])) for e in episodes)


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


def test_an_appearance_is_never_remembered_as_a_name() -> None:
    # X-4 (ARC-91, AS26a-9): a look is a key a client draws. Memory never states it, and never learns a
    # name from it: the slug appears in no gist, and a stranger's look does not name them.
    stranger = EntityId("7999999999")
    bob_looks = fact(
        10,
        100,
        "appeared",
        {"person": {"entity": BOB, "entity_type": "person"}, "look": "bob-achterberg"},
        subjects=[BOB],
        participants=[BOB],
        place=None,
    )
    stranger_looks = fact(
        11,
        101,
        "appeared",
        {"person": {"entity": stranger, "entity_type": "person"}, "look": "stranger-look"},
        subjects=[stranger],
        participants=[stranger],
        place=None,
    )
    with seat_view() as store:
        ingest(store, names_of_everyone(), EventId("4"))
        report = ingest(store, [bob_looks, stranger_looks], EventId("11"))
        dump = store.dump()
    gists = {str(r["event_id"]): str(r["gist"]) for r in _rows(dump, "l0")}
    assert gists["10"] == "appeared at somewhere, with Bob Achterberg"
    assert "someone" in gists["11"]
    assert not any("bob-achterberg" in gist or "stranger-look" in gist for gist in gists.values())
    names = _rows(dump, "names")
    assert {str(row["entity"]) for row in names} == {str(ALICE), str(BOB), str(CAROL), str(DEV)}
    assert len(names) == 4
    assert report.decode_fallbacks == 0


def test_perceived_traits_are_never_a_memory_record_and_a_trait_change_is_generic() -> None:
    # AC-f-11 (S30-f, F-S30f-5; I-S30-9): attributes reach memory only as perceived facts. A hundred
    # observations of Alice's `traits` add no record; a `trait-changed` fact renders generically and is citable.
    block = {
        "schema": 2,
        "entries": [
            {
                "attribute": "test.mood",
                "kind": "category",
                "label": "Mood",
                "facets": ["observed", "prompt"],
                "rank": 10,
                "text": "cheerful",
            }
        ],
    }
    traits = {"entity": ALICE, "component_type": "traits", "schema_version": 1, "payload": block}
    with seat_view() as store:
        ingest(store, names_of_everyone(), EventId("4"))
        before = _rows(store.dump(), "l0")
        for at in range(100):
            observation = Observation.model_validate(
                {
                    "observer": ALICE,
                    "at": 4000 + at,
                    "self_location": None,
                    "entities": [
                        {
                            "id": ALICE,
                            "entity_type": "person",
                            "location": None,
                            "tags": [],
                            "components": [traits],
                        }
                    ],
                    "relations": [],
                    "events": [],
                    "affordances": [],
                }
            )
            assert SelfAttributes.of(observation).about == "cheerful"
            Situation.of(observation, NameBook.load(store))
        assert _rows(store.dump(), "l0") == before
        changed = fact(
            10,
            5000,
            "trait-changed",
            {
                "person": {"entity": ALICE, "entity_type": "person"},
                "attribute": "test.mood",
                "value": {"category": "calm"},
            },
            subjects=[ALICE],
            participants=[ALICE],
            place=CAFE,
        )
        report = ingest(store, [changed], EventId("10"))
        (row,) = [r for r in _rows(store.dump(), "l0") if r["event_id"] == 10]
    assert row["gist"] == "trait-changed at somewhere, with me"
    assert report.decode_fallbacks == 0 and report.records == 1
def _notice(
    event: int, at: int, publisher: EntityId, body: dict[str, JsonValue], *, until: int
) -> PerceivedEvent:
    """A `notice-published` as the notices pack states it (pr-s31b §5.8), delivered to its audience."""
    notice: dict[str, JsonValue] = {
        "id": event,
        "topic": "news.daily" if "text" in body else "prices.ingredients",
        "publisher": {"entity": publisher, "entity_type": "person"},
        "body": body,
        "issued_at": at,
        "valid_until": until,
    }
    return fact(
        event,
        at,
        "notice-published",
        {"notice": notice},
        subjects=[publisher],
        participants=[publisher],
        place=None,
    )


def test_a_notice_is_remembered_with_its_publisher_and_its_end() -> None:
    # AC-b-10: the gist names the publisher, the topic and when the notice stops being valid, in recall's
    # own `Day n HH:MM` (13 600 s is day 1, 03:46). A text body is quoted.
    gists, report = _gists(
        [
            _notice(10, 10_000, BOB, {"text": "The market opens late tomorrow."}, until=13_600),
            _notice(11, 10_100, ALICE, {"text": "Fresh bread at noon."}, until=97_200),
        ]
    )
    assert gists["10"] == (
        "Bob Achterberg posted news.daily (valid until Day 1 03:46): «The market opens late tomorrow.»"
    )
    assert gists["11"] == "I posted news.daily (valid until Day 2 03:00): «Fresh bread at noon.»"
    assert report.decode_fallbacks == 0


def test_a_price_notice_names_six_lines_and_counts_the_rest_without_any_id() -> None:
    # AC-b-10: eight lines, six shown, then how many more. No item has a name this seat knows, so each is
    # "an item": a rendered line never shows an entity id (P4-5; D-S31b-4).
    items = [f"71000000{index:02d}" for index in range(1, 9)]
    lines: list[JsonValue] = [
        {"item": item, "price": 100 * (index + 1), "basis": 3} for index, item in enumerate(items)
    ]
    gists, report = _gists([_notice(10, 10_000, BOB, {"prices": lines}, until=13_600)])
    assert gists["10"] == (
        "Bob Achterberg posted prices.ingredients (valid until Day 1 03:46): an item 100, an item 200, "
        "an item 300, an item 400, an item 500, an item 600 … and 2 more"
    )
    assert not any(item in gists["10"] for item in items)
    assert report.decode_fallbacks == 0


def test_a_notice_is_mine_only_when_i_posted_it() -> None:
    from mineworld_cognition.memory.names import NameBook
    from mineworld_cognition.memory.render import render

    with seat_view() as store:
        names = NameBook(store, {})
        theirs = render(_notice(10, 10, BOB, {"text": "x"}, until=7_200), ALICE, names)
        mine = render(_notice(11, 10, ALICE, {"text": "x"}, until=7_200), ALICE, names)
    assert (theirs.mine, mine.mine) == (False, True)
    assert (theirs.salience, theirs.importance) == ("notable", 400)
    assert theirs.counterparts == (BOB,)
    assert mine.counterparts == ()


def test_a_notice_that_does_not_decode_falls_back_and_is_counted() -> None:
    # A body that is both kinds, and one that is neither, are not notices this renderer can read.
    gists, report = _gists(
        [
            _notice(10, 10_000, BOB, {"text": "x", "prices": []}, until=13_600),
            _notice(11, 10_000, BOB, {}, until=13_600),
        ]
    )
    assert gists["10"].startswith("notice-published at somewhere")
    assert gists["11"].startswith("notice-published at somewhere")
    assert report.decode_fallbacks == 2
