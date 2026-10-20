"""AC-LE2-6, the shared half: the Python estimate agrees with Rust's `commute::estimate` on one literal table,
and the disclosed `ways` record is read as movement writes it (`pr-le-leave-early.md` §0.3, LE-c C1)."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from mineworld_sdk import ways
from mineworld_sdk.wire.contract import Observation
from mineworld_sdk.wire.ids import EntityId

TABLE = (
    Path(__file__).resolve().parents[3]
    / "cognition"
    / "rule-controller"
    / "tests"
    / "estimate-table.json"
)


def test_strides_and_estimate_follow_the_table_rust_is_checked_against() -> None:
    rows: list[dict[str, Any]] = json.loads(TABLE.read_text(encoding="utf-8"))["rows"]
    assert len(rows) >= 6, "the table is not empty"
    for row in rows:
        way = ways.Way(place=EntityId("7"), doors=row["doors"], length_mm=row["length_mm"])
        assert ways.strides(way) == row["strides"], row
        assert ways.estimate(way, row["stride_s"]) == row["estimate_s"], row


def test_minutes_round_up() -> None:
    assert [ways.minutes(s) for s in (0, 1, 59, 60, 61, 516)] == [0, 1, 1, 1, 2, 9]


def _observation(components: list[dict[str, Any]]) -> Observation:
    return Observation.model_validate(
        {
            "observer": "101",
            "at": 0,
            "self_location": None,
            "entities": [
                {
                    "id": "101",
                    "entity_type": "person",
                    "location": None,
                    "tags": [],
                    "components": components,
                }
            ],
            "relations": [],
            "events": [],
            "affordances": [],
        }
    )


def test_the_disclosed_record_is_read_in_order_and_unknown_entries_are_skipped() -> None:
    payload = [
        {"place": {"entity": "3", "entity_type": "place"}, "doors": 1, "length": 13_400},
        {"place": {"entity": "4", "entity_type": "place"}, "doors": True, "length": 1},
        {"place": "5", "doors": 1, "length": 1},
        {"place": {"entity": "6", "entity_type": "place"}, "doors": 2, "length": 0},
    ]
    record = {"entity": "101", "component_type": "ways", "schema_version": 1, "payload": payload}
    found = ways.of(_observation([record]))
    assert found == (
        ways.Way(place=EntityId("3"), doors=1, length_mm=13_400),
        ways.Way(place=EntityId("6"), doors=2, length_mm=0),
    )
    assert ways.of(_observation([])) == ()
