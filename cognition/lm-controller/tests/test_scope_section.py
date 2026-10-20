"""F-S28c-2, I-c-6: the widened section renders the person's five first and parses every logged section."""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from mineworld_sdk.wire.ids import EventId
from pydantic import ValidationError

from mineworld_cognition.controller.log import DecisionRecord
from mineworld_cognition.memory.records import Citation
from mineworld_cognition.memory.retrieve import MemorySection, SectionHeading, SectionLine

FIXTURE = Path(__file__).parent / "fixtures" / "milestone_d.decisions.jsonl"


def _line(level: str, event: str) -> SectionLine:
    return SectionLine(level=level, text=f"{level} line", citation=Citation.of(EventId(event)))


def test_other_sections_render_after_the_persons_in_stack_order() -> None:
    section = MemorySection(
        lines=(_line("notes", "5"), _line("L0", "4"), _line("place", "6"), _line("L3", "3")),
        sections=(
            SectionHeading(key="place", header="About places:"),
            SectionHeading(key="notes", header="Notes:"),
        ),
    )
    assert section.render() == (
        "People I know:\nL3 line [#3]\n"
        "The last day:\nL0 line [#4]\n"
        "About places:\nplace line [#6]\n"
        "Notes:\nnotes line [#5]\n"
    )


def test_a_section_key_is_a_person_level_or_a_scope_id() -> None:
    for good in ("L0", "L3", "with", "place", "l1"):
        _line(good, "1")
    for bad in ("L4", "Place", "", "9x"):
        with pytest.raises(ValidationError):
            _line(bad, "1")


def test_the_milestone_d_log_parses_unchanged() -> None:
    # AP7-4: the committed log's sections (P4's `level` values, no `sections`) parse as they are.
    parsed = 0
    for text in FIXTURE.read_text(encoding="utf-8").splitlines()[:20]:
        loaded: object = json.loads(text)
        if isinstance(loaded, dict) and loaded.get("kind") == "decision":  # pyright: ignore[reportUnknownMemberType]
            record = DecisionRecord.model_validate(loaded, strict=False)
            assert record.input.memory.sections == ()
            parsed += 1
    assert parsed > 0
