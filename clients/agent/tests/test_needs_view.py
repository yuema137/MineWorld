"""`view.py` shows my needs and who is asleep, over the recorded needs fixtures (S35 N-e, AC-Ne-3).

The needs observations are the repository's own recordings (`tests/fixtures/observations/needs/`, `SOURCE.md`), read
here as raw JSON. Expected values are read from those files, never from `disclosed.py`'s output. A pack-free
observation shows neither line.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import pytest
from recorded import NAMES, component, decoded, entity, observation

from mineworld_agent.models import AsleepLine
from mineworld_agent.refs import RefTable
from mineworld_agent.view import render

NEEDS = Path(__file__).resolve().parents[3] / "tests" / "fixtures" / "observations" / "needs"

type Raw = dict[str, Any]


def needs_raw(name: str) -> Raw:
    value: Raw = json.loads((NEEDS / f"{name}.json").read_text(encoding="utf-8"))
    return value


def rendered_of(value: Raw) -> tuple[Any, RefTable]:
    refs = RefTable()
    return render(decoded(value), refs, 1, True), refs


def expected_needs(value: Raw, who: str) -> list[tuple[str, str, int]]:
    record = component(entity(value, who), "needs")
    return [(need, word, record[need]) for need, word in record["words"].items()]


@pytest.mark.parametrize("name", ["market-town-night", "commute-late"])
def test_my_needs_are_the_words_and_levels_the_file_holds(name: str) -> None:
    value = needs_raw(name)
    rendered, _ = rendered_of(value)
    assert rendered.me.needs is not None
    shown = [(line.need, line.word, line.level) for line in rendered.me.needs.needs]
    assert shown == expected_needs(value, value["observer"])
    assert shown, "the recording discloses at least one need"


def test_the_sleeper_sees_its_own_alarm() -> None:
    value = needs_raw("market-town-night")
    record = component(entity(value, value["observer"]), "asleep")
    assert record["asleep"] is True and record["alarm"] is not None
    rendered, _ = rendered_of(value)
    assert rendered.me.asleep == AsleepLine(since=record["since"], alarm=record["alarm"])


def test_someone_who_sleeps_nearby_is_shown_without_an_alarm() -> None:
    value = needs_raw("commute-sleeper")
    sleeper = next(e["id"] for e in value["entities"] if component(e, "asleep") is not None)
    assert sleeper != value["observer"]
    record = component(entity(value, sleeper), "asleep")
    assert "alarm" not in record
    rendered, refs = rendered_of(value)
    line = next(p for p in rendered.people if p.ref == refs.ref(sleeper))
    assert line.asleep == AsleepLine(since=record["since"], alarm=None)


def test_the_observer_of_the_sleeper_recording_shows_its_own_needs() -> None:
    value = needs_raw("commute-sleeper")
    rendered, _ = rendered_of(value)
    assert rendered.me.needs is not None
    shown = [(line.need, line.word) for line in rendered.me.needs.needs]
    assert shown == [(need, word) for need, word, _ in expected_needs(value, value["observer"])]
    assert rendered.me.asleep is None


@pytest.mark.parametrize("name", NAMES)
def test_a_pack_free_observation_shows_neither_line(name: str) -> None:
    rendered = render(observation(name), RefTable(), 1, True)
    assert rendered.me.needs is None
    assert rendered.me.asleep is None
    assert all(person.asleep is None for person in rendered.people)


def test_a_word_that_is_not_an_identifier_is_skipped_not_fatal() -> None:
    value = needs_raw("market-town-night")
    observer = value["observer"]
    record = component(entity(value, observer), "needs")
    record["words"]["hunger"] = "Not A Word"
    rendered, _ = rendered_of(value)
    assert rendered.me.needs is not None
    shown = [line.need for line in rendered.me.needs.needs]
    assert "hunger" not in shown
    assert "tiredness" in shown
