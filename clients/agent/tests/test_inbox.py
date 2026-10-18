"""The inbox: an exact bound and drop count, `spoke` and `walk-ended` read by name, every `WaitFor`, and the
facts an accepted act caused shown once. Facts are built in the contract's JSON (`PerceivedEvent`), the
payloads as the conversation and movement packs write them (`{ entity, entity_type }` persons)."""

from __future__ import annotations

import json

from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import ActionId, EntityId, JsonValue
from recorded import observation

from mineworld_agent.inbox import Inbox, offered
from mineworld_agent.models import AnyEvent, OfferAvailable, SpokenTo, ViewChanged, WalkEnded
from mineworld_agent.refs import RefTable

ME, ALICE, CAFE = "17", "7", "2"


def person(entity: str) -> JsonValue:
    return {"entity": entity, "entity_type": "person"}


def fact(n: int, kind: str, payload: JsonValue, *, action: str | None = None) -> PerceivedEvent:
    value: JsonValue = {
        "id": str(100 + n),
        "at": n,
        "event_type": kind,
        "subjects": [ALICE, ME],
        "participants": [ALICE, ME],
        "place": {"entity": CAFE, "entity_type": "place"},
        "caused_by": {"action": action} if action else "world_genesis",
        "payload": {"event_type": kind, "schema_version": 1, "payload": payload},
        "visibility": "public",
        "provenance": {"emitted_by": "conversation", "controller_decision": action},
    }
    return PerceivedEvent.model_validate_json(json.dumps(value))


def spoke(
    n: int, speaker: str, listener: str, words: str, action: str | None = None
) -> PerceivedEvent:
    payload = {"speaker": person(speaker), "listener": person(listener), "utterance": words}
    return fact(n, "spoke", payload, action=action)


def walk_ended(n: int, who: str) -> PerceivedEvent:
    destination: JsonValue = {"place": {"place": {"entity": CAFE, "entity_type": "place"}}}
    return fact(
        n, "walk-ended", {"person": person(who), "destination": destination, "outcome": "arrived"}
    )


def test_the_bound_is_exact_and_every_drop_is_counted() -> None:
    inbox = Inbox(RefTable(), EntityId(ME), bound=512)
    inbox.add(fact(n, "tick", {}) for n in range(1, 513))
    assert (inbox.unread, inbox.dropped) == (512, 0)
    inbox.add([fact(513, "tick", {}), fact(514, "tick", {})])
    assert (inbox.unread, inbox.dropped) == (512, 2)
    lines, following = inbox.events(None, 20)
    assert [line.ref for line in lines] == [f"F{n}" for n in range(3, 23)], "the oldest two dropped"
    assert following == "F22"
    assert inbox.unread == 512 - 20


def test_events_page_by_after_and_end_with_no_next() -> None:
    inbox = Inbox(RefTable(), EntityId(ME))
    inbox.add(fact(n, "tick", {}) for n in range(1, 26))
    first, after = inbox.events(None, 10)
    second, after2 = inbox.events(after, 10)
    third, after3 = inbox.events(after2, 10)
    assert [len(first), len(second), len(third)] == [10, 10, 5]
    assert (after, after2, after3) == ("F10", "F20", None)
    assert inbox.unread == 0


def test_a_spoke_fact_carries_its_words_as_speech_by_the_speaker() -> None:
    refs = RefTable()
    inbox = Inbox(refs, EntityId(ME))
    inbox.add([spoke(1, ALICE, ME, ">>> hello <<<")])
    (line,), _ = inbox.events(None, 10, detail=True)
    assert line.event_type == "spoke" and line.words is not None
    assert line.words.text == ">>> hello <<<" and line.words.kind == "speech"
    assert line.words.by == refs.ref(EntityId(ALICE))
    assert line.subjects == [refs.ref(EntityId(ALICE)), refs.ref(EntityId(ME))]
    assert line.place == refs.ref(EntityId(CAFE))
    assert line.detail is not None and line.detail.truncated is False


def test_other_facts_show_type_and_participants_only_and_detail_is_capped() -> None:
    inbox = Inbox(RefTable(), EntityId(ME))
    inbox.add([fact(1, "bought", {"note": "x" * 5000, "price": 300})])
    (line,), _ = inbox.events(None, 10, detail=True)
    assert line.words is None and line.event_type == "bought"
    assert line.detail is not None and line.detail.truncated and line.detail.bytes > 5000
    assert len(json.dumps(line.detail.payload)) <= 1024
    again = Inbox(RefTable(), EntityId(ME))
    again.add([fact(1, "bought", {"price": 300})])
    (plain,), _ = again.events(None, 1)
    assert plain.detail is None, "detail is off by default"


def test_every_fact_shaped_wait_condition() -> None:
    inbox = Inbox(RefTable(), EntityId(ME))
    inbox.add([spoke(1, ME, ALICE, "hi"), walk_ended(2, ALICE)])
    assert inbox.matches(AnyEvent(), after=0) and not inbox.matches(AnyEvent(), after=2)
    assert not inbox.matches(SpokenTo(), after=0), "I spoke; nobody spoke to me"
    assert not inbox.matches(WalkEnded(), after=0), "Alice's walk, not mine"
    inbox.add([spoke(3, ALICE, ME, "hello"), walk_ended(4, ME)])
    assert inbox.matches(SpokenTo(), after=2) and inbox.matches(WalkEnded(), after=2)
    assert not inbox.matches(SpokenTo(), after=3)
    assert not inbox.matches(ViewChanged(), after=0), "the observation's, not a fact's"


def test_offer_available_reads_the_newest_observation() -> None:
    seen = observation("market-town-start")
    refs = RefTable()
    alice = refs.ref(EntityId(ALICE))
    assert offered(OfferAvailable(action_type="walk-to"), seen, refs)
    assert not offered(OfferAvailable(action_type="talk", target=alice), seen, refs), "too far"
    assert not offered(OfferAvailable(action_type="walk-step"), seen, refs), "no walk at genesis"
    assert not offered(OfferAvailable(action_type="talk", target="E99"), seen, refs)


def test_what_an_act_caused_is_shown_once() -> None:
    inbox = Inbox(RefTable(), EntityId(ME))
    inbox.add([spoke(1, ME, ALICE, "hi", action="41"), spoke(2, ALICE, ME, "hello")])
    caused = inbox.caused_by(ActionId("41"))
    assert [line.ref for line in caused] == ["F1"]
    assert inbox.unread == 1
    lines, _ = inbox.events(None, 10)
    assert [line.ref for line in lines] == ["F2"], "the act's own fact is not shown again"
