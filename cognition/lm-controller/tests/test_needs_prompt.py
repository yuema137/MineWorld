"""The LM seat reads how it feels and who is asleep (S35 N-e, AC-Ne-5).

The observations are the repository's recordings (`tests/fixtures/observations/needs/`, `SOURCE.md`), read as raw
JSON. The expected words and the `HH:MM` times are read from those files, or are the hand-written values of
`SOURCE.md`. A world without the needs and sleep packs renders byte-identically: `test_controller_choice.py` holds
that golden (`tests/fixtures/prompt_request.json`), and it runs unchanged.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from memory_fixtures import open_view
from mineworld_sdk.wire.contract import Observation
from mineworld_sdk.wire.ids import EntityKey, EventId

from mineworld_cognition.controller.persona import Persona
from mineworld_cognition.controller.prompt import Trigger, build_request
from mineworld_cognition.controller.situation import Situation
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.retrieve import MemorySection

NEEDS = Path(__file__).resolve().parents[3] / "tests" / "fixtures" / "observations" / "needs"
SEAT = EntityKey("seat")


def recorded(name: str) -> dict[str, Any]:
    value: dict[str, Any] = json.loads((NEEDS / f"{name}.json").read_text(encoding="utf-8"))
    return value


def component(raw: dict[str, Any], entity: str, kind: str) -> Any:  # noqa: ANN401 (a pack's raw JSON)
    found = next(e for e in raw["entities"] if e["id"] == entity)
    return next((c["payload"] for c in found["components"] if c["component_type"] == kind), None)


def user_text(raw: dict[str, Any]) -> str:
    observation = Observation.model_validate(raw)
    situation = Situation.of(observation, NameBook.load(open_view()))
    trigger = Trigger(event=EventId("9"), kind="arrived", speaker=None, words="")
    request = build_request(SEAT, Persona(), situation, MemorySection(lines=()), trigger, ())
    return request.messages[1].text


def feel_line(raw: dict[str, Any]) -> str:
    words = component(raw, raw["observer"], "needs")["words"]
    return "How I feel: " + "; ".join(f"{need}: {word}" for need, word in words.items())


def test_a_sleeper_at_night_feels_and_says_when_she_fell_asleep_and_when_the_alarm_rings() -> None:
    raw = recorded("market-town-night")
    text = user_text(raw)
    assert feel_line(raw) in text
    # SOURCE.md: asleep since 02:25 with the alarm at 09:11.
    assert "I am asleep since 02:25; my alarm is at 09:11.\n" in text
    assert text.index("How I feel: ") < text.index("I am asleep since ") < text.index("Offers:")


def test_an_employee_who_was_absent_feels_hungry_and_tired_and_is_not_asleep() -> None:
    raw = recorded("commute-late")
    text = user_text(raw)
    assert feel_line(raw) in text
    assert "famished" in text and "exhausted" in text
    assert "I am asleep" not in text


def test_a_sleeper_in_the_room_is_marked_asleep_and_the_observer_feels_his_own_needs() -> None:
    raw = recorded("commute-sleeper")
    sleeper = next(
        e["id"] for e in raw["entities"] if component(raw, e["id"], "asleep") is not None
    )
    assert component(raw, sleeper, "asleep")["asleep"] is True
    assert "alarm" not in component(raw, sleeper, "asleep")
    text = user_text(raw)
    people = next(line for line in text.splitlines() if line.startswith("People here: "))
    assert "(asleep)" in people
    assert feel_line(raw) in text
    assert "I am asleep" not in text, "the observer is awake; only another person is asleep"


def test_each_sleep_line_is_within_its_byte_bound_and_shows_no_entity_id() -> None:
    raw = recorded("market-town-night")
    text = user_text(raw)
    line = next(x for x in text.splitlines() if x.startswith("I am asleep since "))
    assert len(line.encode("utf-8")) <= 160
    assert raw["observer"] not in line.split()


def _food_energy(raw: dict[str, Any], schema: int, block: Any) -> dict[str, Any]:  # noqa: ANN401
    """The recording with my `needs` record restated as a schema-`schema` food-energy record (ARC-370)."""
    edited = json.loads(json.dumps(raw))
    me = next(e for e in edited["entities"] if e["id"] == edited["observer"])
    for record in me["components"]:
        if record["component_type"] == "needs":
            record["schema_version"] = schema
            record["payload"]["hunger_model"] = "food-energy"
            record["payload"]["food_energy"] = block
    return edited


RESERVE = {
    "unit": "kcal",
    "available_kcal": 1500,
    "capacity_kcal": 2000,
    "use_kcal_per_world_day": 2000,
    "bar": {"value": 750, "maximum": 1000},
    "replaces_need": "hunger",
}


def test_a_food_energy_seat_reads_its_reserve_in_one_bounded_line() -> None:
    text = user_text(_food_energy(recorded("market-town-night"), 2, RESERVE))
    line = "My food energy: 1500 of 2000 kcal; I use 2000 kcal a day."
    assert text.count(line) == 1, text
    assert len(line.encode("utf-8")) <= 160


def test_no_reserve_line_for_schema_1_an_unknown_schema_or_a_malformed_block() -> None:
    raw = recorded("market-town-night")
    for edited in [
        raw,
        _food_energy(raw, 3, RESERVE),
        _food_energy(raw, 2, {**RESERVE, "available_kcal": 2001}),
        _food_energy(raw, 2, {**RESERVE, "available_kcal": True}),
        _food_energy(raw, 2, {**RESERVE, "unit": "kJ"}),
    ]:
        assert "My food energy" not in user_text(edited)
