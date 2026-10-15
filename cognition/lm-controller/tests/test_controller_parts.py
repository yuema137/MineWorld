"""The typed parts of the controller: persona, situation and memory view (pr-s10-p6 §5.2.1 to 5.2.3, C5).

The golden frame `server/tests/frames/observation.json` carries no entities and no affordances, so the labels,
collisions and numbering are tested over an observation built here in the same wire form. Every value comes
from a literal in the test, never from the code under test.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from memory_fixtures import ALICE, BOB, CAFE, DAY, PARK, alice_identity, names_of_everyone, spoke
from mineworld_sdk.wire.contract import Observation
from mineworld_sdk.wire.ids import EntityId, JsonValue

from mineworld_cognition.controller import Persona, Situation, memory_view
from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.retrieve import RecallQuery
from mineworld_cognition.memory.store import MemoryStore

REPOSITORY = Path(__file__).resolve().parents[3]
GOLDEN = REPOSITORY / "server" / "tests" / "frames" / "observation.json"
BOB_TWO = EntityId("7000000005")
DEV = EntityId("7000000006")
ANY_PLACE = "any"


def _requirement() -> dict[str, Any]:
    return {
        "place": ANY_PLACE,
        "within_range": None,
        "requires_line_of_access": False,
        "requires_target_available": True,
    }


def _place_ref(place: EntityId) -> dict[str, Any]:
    return {"entity": place, "entity_type": "place"}


def _entity(
    entity: EntityId, kind: str, place: EntityId | None, name: str | None
) -> dict[str, Any]:
    components: list[dict[str, Any]] = []
    if name is not None:
        components.append(
            {
                "entity": entity,
                "component_type": "display-name",
                "schema_version": 1,
                "payload": {"name": name},
            }
        )
    location = (
        None if place is None else {"place": _place_ref(place), "local": None, "facing": None}
    )
    return {
        "id": entity,
        "entity_type": kind,
        "location": location,
        "tags": [],
        "components": components,
    }


def _affordance(
    action: str, target: EntityId | None, available: bool, payload: JsonValue | None
) -> dict[str, Any]:
    wire: dict[str, Any] = {
        "action_type": action,
        "target": target,
        "available": available,
        "unavailable_reason": None if available else "too_far_away",
        "requirement": _requirement(),
    }
    if payload is not None:
        wire["payload"] = payload
    return wire


def _observation(
    entities: list[dict[str, Any]], affordances: list[dict[str, Any]], place: EntityId | None
) -> Observation:
    return Observation.model_validate(
        {
            "observer": ALICE,
            "at": 4112,
            "self_location": None
            if place is None
            else {"place": _place_ref(place), "local": None, "facing": None},
            "entities": entities,
            "relations": [],
            "events": [],
            "affordances": affordances,
        }
    )


def _crowded() -> Observation:
    """Alice at the café with two people named Bob (one talkable) and Dev at the park."""
    entities = [
        _entity(ALICE, "person", CAFE, "Alice"),
        _entity(BOB, "person", CAFE, "Bob"),
        _entity(BOB_TWO, "person", CAFE, "Bob"),
        _entity(DEV, "person", PARK, "Dev"),
    ]
    affordances = [
        _affordance("talk", BOB, True, None),
        _affordance("talk", BOB_TWO, False, None),
        _affordance("order", None, True, {"item": "tea"}),
        _affordance("wave", BOB, True, None),
    ]
    return _observation(entities, affordances, CAFE)


def test_the_golden_frame_has_no_one_to_label_and_no_offer() -> None:
    frame = Observation.model_validate(
        json.loads(GOLDEN.read_text(encoding="utf-8"))["observation"]
    )
    store = MemoryStore.open(":memory:", alice_identity())
    situation = Situation.of(frame, NameBook.load(store))
    assert situation.present == ()
    assert situation.talk_to == ()
    assert situation.offers == ()
    assert situation.at == 4112


def test_two_people_with_one_name_get_distinct_labels_in_id_order() -> None:
    store = MemoryStore.open(":memory:", alice_identity())
    situation = Situation.of(_crowded(), NameBook.load(store))
    assert [person.label for person in situation.present] == ["Bob", "Bob (2)"]
    assert [person.entity for person in situation.present] == [BOB, BOB_TWO]
    assert situation.me == "Alice"
    assert situation.place == "somewhere"  # no place name was learned


def test_an_unavailable_talk_is_not_talkable_and_offers_are_numbered_from_one() -> None:
    store = MemoryStore.open(":memory:", alice_identity())
    situation = Situation.of(_crowded(), NameBook.load(store))
    assert situation.talk_to == ("Bob",)
    assert [(offer.number, offer.action_type, offer.position) for offer in situation.offers] == [
        (1, "order", 2),
    ]
    assert situation.offers[0].target_label is None


def test_a_person_elsewhere_is_not_present() -> None:
    store = MemoryStore.open(":memory:", alice_identity())
    situation = Situation.of(_crowded(), NameBook.load(store))
    assert "Dev" not in [person.label for person in situation.present]


def _persona_observation(payload: JsonValue) -> Observation:
    entity = _entity(ALICE, "person", CAFE, "Alice")
    entity["components"].append(
        {"entity": ALICE, "component_type": "persona", "schema_version": 1, "payload": payload}
    )
    return _observation([entity], [], CAFE)


def test_the_persona_is_read_from_the_observers_own_component() -> None:
    payload: dict[str, JsonValue] = {
        "summary": "Runs the counter.",
        "traits": ["warm", "dry"],
        "speech": "short",
    }
    persona, absent = Persona.of(_persona_observation(payload))
    assert absent is False
    assert persona == Persona(summary="Runs the counter.", traits=("warm", "dry"), speech="short")


def test_a_missing_or_oversized_persona_is_empty_and_flagged() -> None:
    assert Persona.of(_observation([_entity(ALICE, "person", CAFE, "Alice")], [], CAFE)) == (
        Persona(),
        True,
    )
    oversized: dict[str, JsonValue] = {"summary": "x" * 400, "traits": [], "speech": ""}
    assert Persona.of(_persona_observation(oversized)) == (Persona(), True)


def _memory_store() -> MemoryStore:
    """Alice's store with names, and many lines of speech from Bob and her, so every level is non-empty."""
    store = MemoryStore.open(":memory:", alice_identity())
    facts = list(names_of_everyone())
    last = max(int(fact.id) for fact in facts)
    for index in range(40):
        speaker, listener = (BOB, ALICE) if index % 2 else (ALICE, BOB)
        facts.append(
            spoke(last + 1 + index, index * 600, speaker, listener, f"line number {index}", CAFE)
        )
    ingest(store, facts, facts[-1].id)
    return store


def test_the_episode_knob_shrinks_the_section_and_its_citations_follow() -> None:
    store = _memory_store()
    query = RecallQuery(at=DAY, counterparts=(BOB,), words="line")
    sizes: list[int] = []
    for recall_episodes in (8, 3, 0):
        section = memory_view(store, query, recall_episodes)
        sizes.append(len(section.render().encode("utf-8")))
        cited = {(r.first, r.last) for line in section.lines for r in line.citation.ranges}
        assert {(r.first, r.last) for r in section.citations()} == cited
        assert sum(1 for line in section.lines if line.level == "L1") <= recall_episodes
    assert sizes == sorted(sizes, reverse=True)
    assert sizes[0] >= sizes[-1]
