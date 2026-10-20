"""Who I am, how I speak and who I see, from disclosed attributes (pr-s30f-attribute-consumers §5.6, C5).

Every expected string is a literal written here or in `fixtures/prompt_request_attributes.json`, by hand from the
blocks below, never produced by the code under test. A world without attribute blocks must build the request
`fixtures/prompt_request.json` records, byte for byte (I-f-2); `test_controller_choice.py` holds that golden.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from memory_fixtures import ALICE, BOB, CAFE, CAROL, open_view
from mineworld_sdk.wire.contract import Observation
from mineworld_sdk.wire.ids import ActionTypeId, EntityId, EntityKey, EventId

from mineworld_cognition.controller.attributes import NO_ATTRIBUTES, SelfAttributes, description
from mineworld_cognition.controller.log import DecisionInput
from mineworld_cognition.controller.persona import Persona
from mineworld_cognition.controller.prompt import Trigger, build_request
from mineworld_cognition.controller.situation import Offer, Person, Situation
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.retrieve import MemorySection

FIXTURES = Path(__file__).parent / "fixtures"


def _entry(attribute: str, facets: list[str], rank: int | None, text: str) -> dict[str, Any]:
    entry: dict[str, Any] = {
        "attribute": attribute,
        "kind": "category",
        "label": attribute.split(".", 1)[1],
        "facets": facets,
        "text": text,
    }
    if rank is not None:
        entry["rank"] = rank
    return entry


def _block(*entries: dict[str, Any]) -> dict[str, Any]:
    return {"schema": 2, "entries": list(entries)}


def _component(entity: EntityId, kind: str, payload: object) -> dict[str, Any]:
    return {"entity": entity, "component_type": kind, "schema_version": 1, "payload": payload}


def _person(entity: EntityId, name: str, *components: dict[str, Any]) -> dict[str, Any]:
    return {
        "id": entity,
        "entity_type": "person",
        "location": {
            "place": {"entity": CAFE, "entity_type": "place"},
            "local": None,
            "facing": None,
        },
        "tags": [],
        "components": [_component(entity, "display-name", {"name": name}), *components],
    }


ALICE_OWN = _block(
    _entry("test.age-band", ["observed", "prompt"], 10, "in her sixties"),
    _entry("test.formality", ["speech"], 500, "speaks softly, says <<<hm>>> a lot"),
    _entry("test.ancestry", ["casting"], None, "of Irish descent"),
)
"""Alice's own view of herself: who I am, how I speak, and a casting-only entry no prompt shows."""

ALICE_JOB = _block(_entry("work.occupation", ["prompt"], 205, "works as barista"))
"""An owner's block, embedded under its component's `attributes` member."""

BOB_SEEN = _block(
    _entry("test.hair-colour", ["observed"], 40, "grey hair"),
    _entry("test.stature", ["body", "observed"], 20, "tall"),
    _entry("test.formality", ["speech"], 500, "talks fast"),
)
"""Bob as Alice perceives him: two observed entries (out of rank order on the wire) and one not observed."""


def _observation() -> Observation:
    return Observation.model_validate(
        {
            "observer": ALICE,
            "at": 4112,
            "self_location": {
                "place": {"entity": CAFE, "entity_type": "place"},
                "local": None,
                "facing": None,
            },
            "entities": [
                _person(
                    ALICE,
                    "Alice",
                    _component(ALICE, "traits", ALICE_OWN),
                    _component(ALICE, "employment", {"since": 3, "attributes": ALICE_JOB}),
                ),
                _person(BOB, "Bob", _component(BOB, "traits", BOB_SEEN)),
                _person(CAROL, "Carol"),
            ],
            "relations": [],
            "events": [],
            "affordances": [],
        }
    )


def test_who_i_am_and_how_i_speak_come_from_my_own_blocks_by_facet_and_rank() -> None:
    assert SelfAttributes.of(_observation()) == SelfAttributes(
        about="in her sixties, works as barista",
        manner="speaks softly, says <<<hm>>> a lot",
    )


def test_who_i_see_comes_from_that_persons_own_record_never_from_mine() -> None:
    # M-f-2: Alice's own `observed` entry ("in her sixties") must not describe Bob.
    situation = Situation.of(_observation(), NameBook.load(open_view()))
    assert [(p.label, p.description) for p in situation.present] == [
        ("Bob", "tall, grey hair"),
        ("Carol", ""),
    ]
    assert description(_observation(), CAROL) == ""


def test_a_world_without_blocks_has_empty_attributes() -> None:
    bare = _observation().model_copy(
        update={
            "entities": [
                entity.model_copy(
                    update={
                        "components": [
                            c for c in entity.components if c.component_type == "display-name"
                        ]
                    }
                )
                for entity in _observation().entities
            ]
        }
    )
    assert SelfAttributes.of(bare) == NO_ATTRIBUTES
    situation = Situation.of(bare, NameBook.load(open_view()))
    assert [p.description for p in situation.present] == ["", ""]


def _situation(present: tuple[Person, ...]) -> Situation:
    return Situation(
        me="Alice",
        place="Cafe",
        at=4112,
        present=present,
        talk_to=("Bob",),
        offers=(Offer(number=1, action_type=ActionTypeId("order"), target_label=None, position=2),),
    )


def _request(attributes: SelfAttributes, present: tuple[Person, ...]) -> dict[str, Any]:
    persona = Persona(summary="Runs the counter.", traits=("warm",), speech="short")
    trigger = Trigger(event=EventId("1907"), kind="spoken_to", speaker=BOB, words="hello")
    request = build_request(
        EntityKey("alice"),
        persona,
        _situation(present),
        MemorySection(lines=()),
        trigger,
        (),
        attributes=attributes,
    )
    return request.model_dump(mode="json")


def test_the_attributes_golden_equals_the_built_request() -> None:
    golden = json.loads((FIXTURES / "prompt_request_attributes.json").read_text("utf-8"))
    present = (
        Person(label="Bob", entity=BOB, description="tall, grey hair"),
        Person(label="Carol", entity=CAROL),
    )
    built = _request(SelfAttributes.of(_observation()), present)
    assert built == golden
    # world text cannot forge a heard-words boundary
    assert "<<<hm>>>" not in built["messages"][0]["text"]


def test_empty_attributes_build_the_request_without_them() -> None:
    golden = json.loads((FIXTURES / "prompt_request.json").read_text("utf-8"))
    assert _request(NO_ATTRIBUTES, (Person(label="Bob", entity=BOB),)) == golden


def test_an_old_log_line_replays_and_an_empty_one_is_written_as_before() -> None:
    logged = DecisionInput(
        persona=Persona(),
        situation=_situation((Person(label="Bob", entity=BOB),)),
        memory=MemorySection(lines=()),
        trigger=Trigger(event=EventId("1907"), kind="spoken_to", speaker=BOB, words="hello"),
        observation=_observation(),
    )
    written = json.loads(logged.model_dump_json())
    assert "attributes" not in written
    assert "description" not in written["situation"]["present"][0]
    assert DecisionInput.model_validate_json(logged.model_dump_json()).attributes == NO_ATTRIBUTES
    told = logged.model_copy(update={"attributes": SelfAttributes(about="tall")})
    assert json.loads(told.model_dump_json())["attributes"] == {"about": "tall", "manner": ""}


def test_two_hundred_entries_stay_within_every_budget() -> None:
    # X-S30-17 on the controller's own path.
    entries = [
        _entry(f"test.a{n:03d}", ["observed", "prompt", "speech"], n, f"phrase {n:03d}")
        for n in range(200)
    ]
    observation = _observation().model_copy(
        update={
            "entities": [
                _observation()
                .entities[0]
                .model_copy(
                    update={
                        "components": [
                            _observation()
                            .entities[0]
                            .components[1]
                            .model_copy(update={"payload": _block(*entries)})
                        ]
                    }
                )
            ]
        }
    )
    mine = SelfAttributes.of(observation)
    assert len(mine.about.encode()) <= 600 and mine.about.endswith(" …")
    assert len(mine.manner.encode()) <= 160 and mine.manner.startswith("phrase 000, phrase 001")
