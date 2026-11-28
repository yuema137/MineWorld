"""The attribute reader and composer against the shared hand-written golden (S30-f §§5.4, 5.5).

The golden is the one the Rust reader (`authoring/attributes/src/disclosed/tests.rs`) and the GDScript
check read; each case's expectations were written by hand, never produced by code under test.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, cast

import pytest

from mineworld_sdk import attributes
from mineworld_sdk.wire.contract import PerceivedEntity

GOLDEN = (
    Path(__file__).resolve().parents[3]
    / "authoring"
    / "attributes"
    / "tests"
    / "fixtures"
    / "composition.json"
)

CASES: list[dict[str, Any]] = json.loads(GOLDEN.read_text(encoding="utf-8"))["cases"]


def _disclosed(case: dict[str, Any]) -> attributes.Disclosed:
    return attributes.read_payloads(
        (component["type"], component["payload"]) for component in case["components"]
    )


def _assert_reading(
    entries: tuple[attributes.AttributeEntry, ...],
    reading: attributes.Reading,
    expected: dict[str, Any],
) -> None:
    for composition in expected["compose"]:
        text = attributes.compose(entries, composition["facet"], composition["budget"], reading)
        assert text == composition["text"], (composition["facet"], composition["budget"])
        if composition["budget"] is not None:
            assert len(text.encode()) <= composition["budget"]
    assert list(attributes.inspect(entries, reading)) == expected["inspect"]
    assert list(attributes.sheet(entries, reading)) == expected["sheet"]


@pytest.mark.parametrize("case", CASES, ids=[case["name"] for case in CASES])
def test_every_golden_case_merges_composes_and_lists_as_written(case: dict[str, Any]) -> None:
    disclosed = _disclosed(case)
    entries = disclosed.entries
    assert [entry.attribute for entry in entries] == case["merged"]
    # The case's own expectations are English: by default, and for a reader asking for `en` (I-k-3).
    _assert_reading(entries, attributes.ENGLISH_READING, case)
    _assert_reading(entries, disclosed.reading("en"), case)
    for expected in case.get("readings", []):
        _assert_reading(entries, disclosed.reading(expected["locale"]), expected)


@pytest.mark.parametrize(
    ("typed", "canonical"),
    [
        ("en", "en"),
        ("zh-Hans", "zh_Hans"),
        ("zh_Hans", "zh_Hans"),
        ("pt-BR", "pt_BR"),
        ("es_419", "es_419"),
    ],
)
def test_a_typed_locale_is_normalised(typed: str, canonical: str) -> None:
    assert attributes.parse_locale(typed) == canonical


@pytest.mark.parametrize("typed", ["", "EN", "zh_hans", "zh-Hans-", "chinese", "zh_Hans_cn"])
def test_a_malformed_locale_is_refused(typed: str) -> None:
    with pytest.raises(ValueError, match="is not a locale"):
        attributes.parse_locale(typed)


def test_a_language_with_two_offered_locales_reads_english() -> None:
    words = attributes.ReaderWords(join="+", more="~", pair="=", yes="y", no="n")
    offered = {"zh_Hans": words, "zh_Hant": words}
    assert attributes.reading("zh_CN", offered) == attributes.ENGLISH_READING
    assert attributes.reading("zh_Hant", offered).locale == "zh_Hant"


def _entry(**changes: object) -> dict[str, object]:
    entry: dict[str, object] = {
        "attribute": "t.mark",
        "kind": "category",
        "label": "Mark",
        "facets": ["observed"],
        "rank": 10,
        "value": {"category": "scar"},
        "text": "a small scar",
    }
    entry.update(changes)
    return entry


def _traits(*entries: dict[str, object], schema: object = 3) -> attributes.Disclosed:
    block = cast(Any, {"schema": schema, "entries": list(entries)})
    return attributes.read_payloads([("traits", block)])


def test_a_readable_block_decodes_its_values_exactly() -> None:
    disclosed = _traits(
        _entry(),
        _entry(attribute="t.height", kind="quantity", value={"quantity": "01.50"}, text=None),
        _entry(attribute="t.tongues", kind="set", value={"set": ["b", "a", "b"]}, text=None),
        _entry(attribute="t.void", kind="set", value={"set": []}, text=None),
    )
    assert disclosed.attributes_unreadable == 0
    # one rank, so ordered by id: t.height, t.mark, t.tongues; the empty set shows nothing
    assert attributes.sheet(disclosed.entries) == ("Mark: 1.50", "Mark: a small scar", "Mark: a, b")


@pytest.mark.parametrize(
    ("why", "block"),
    [
        ("schema 1", {"schema": 1, "entries": [_entry()]}),
        ("schema 2, before locales", {"schema": 2, "entries": [_entry()]}),
        ("schema as a float", {"schema": 3.0, "entries": [_entry()]}),
        ("no entries", {"schema": 3}),
        ("an unknown block member", {"schema": 3, "entries": [_entry()], "more": 1}),
        ("an unknown entry field", {"schema": 3, "entries": [_entry(), _entry(colour="red")]}),
        ("a float quantity", {"schema": 3, "entries": [_entry(value={"quantity": 1.5})]}),
        (
            "two kinds in one value",
            {"schema": 3, "entries": [_entry(value={"flag": True, "category": "x"})]},
        ),
        ("a level without its rank", {"schema": 3, "entries": [_entry(value={"level": "low"})]}),
        ("an id without a namespace", {"schema": 3, "entries": [_entry(attribute="mark")]}),
        ("an unknown kind", {"schema": 3, "entries": [_entry(kind="colour")]}),
        ("a rank as a string", {"schema": 3, "entries": [_entry(rank="10")]}),
        ("not an object", [_entry()]),
    ],
)
def test_a_block_that_does_not_decode_strictly_is_skipped_whole_and_counted(
    why: str, block: object
) -> None:
    good = {"schema": 3, "entries": [_entry(attribute="t.kept", text="kept")]}
    disclosed = attributes.read_payloads(
        [("traits", cast(Any, block)), ("employment", cast(Any, {"attributes": good}))]
    )
    assert [entry.attribute for entry in disclosed.entries] == ["t.kept"], why
    assert disclosed.attributes_unreadable == 1, why


def test_a_component_without_an_attributes_member_is_not_a_block() -> None:
    disclosed = attributes.read_payloads([("display-name", {"name": "Alice"}), ("x", 3)])
    assert disclosed == attributes.Disclosed()


def test_an_entity_is_read_through_its_perceived_components() -> None:
    case = next(case for case in CASES if case["name"] == "merge-and-order")
    entity = PerceivedEntity.model_validate(
        {
            "id": "7",
            "entity_type": "person",
            "location": None,
            "tags": [],
            "components": [
                {
                    "entity": "7",
                    "component_type": component["type"],
                    "schema_version": 1,
                    "payload": component["payload"],
                }
                for component in case["components"]
            ],
        }
    )
    assert [entry.attribute for entry in attributes.blocks(entity)] == case["merged"]
    assert attributes.read(entity).attributes_unreadable == 1
