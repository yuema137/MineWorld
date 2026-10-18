"""Re-location of an offer in the newest observation (step-27 §5.1, P6's rule): by position, then by
`(action_type, target)` with the same payload; gone or incomplete is `StaleOffer`."""

from __future__ import annotations

import copy

import pytest
from recorded import decoded, raw

from mineworld_agent.refs import RefTable
from mineworld_agent.view import Incomplete, Rendered, StaleOffer, relocate, render

START = raw("market-town-start")
BUY = next(i for i, a in enumerate(START["affordances"], 1) if a["action_type"] == "buy")
TALK = next(i for i, a in enumerate(START["affordances"], 1) if a["action_type"] == "talk")


def rendered(value: dict[str, object], number: int, refs: RefTable) -> Rendered:
    return render(decoded(value), refs, number, True)  # type: ignore[arg-type]


def test_an_unmoved_offer_is_found_at_its_position() -> None:
    refs = RefTable()
    shown, newest = rendered(START, 1, refs), rendered(START, 2, refs)
    found = relocate(shown, BUY, newest)
    assert found == newest.observation.affordances[BUY - 1]


def test_a_moved_offer_is_found_by_its_type_target_and_payload() -> None:
    later = copy.deepcopy(START)
    first = later["affordances"].pop(BUY - 1)
    later["affordances"].append(first)
    refs = RefTable()
    shown, newest = rendered(START, 1, refs), rendered(later, 2, refs)
    found = relocate(shown, BUY, newest)
    assert found.payload == START["affordances"][BUY - 1]["payload"]
    assert newest.observation.affordances.index(found) == len(later["affordances"]) - 1


def test_another_item_at_the_same_position_is_never_bought_in_its_place() -> None:
    later = copy.deepcopy(START)
    later["affordances"][BUY - 1]["payload"] = {"item": {"entity": "999", "entity_type": "item"}}
    refs = RefTable()
    with pytest.raises(StaleOffer):
        relocate(rendered(START, 1, refs), BUY, rendered(later, 2, refs))


def test_a_gone_offer_is_stale() -> None:
    later = copy.deepcopy(START)
    del later["affordances"][BUY - 1]
    later["affordances"] = [a for a in later["affordances"] if a != START["affordances"][BUY - 1]]
    refs = RefTable()
    with pytest.raises(StaleOffer):
        relocate(rendered(START, 1, refs), BUY, rendered(later, 2, refs))


def test_an_incomplete_offer_needs_its_own_operation_and_a_number_out_of_range_is_stale() -> None:
    refs = RefTable()
    shown, newest = rendered(START, 1, refs), rendered(START, 2, refs)
    with pytest.raises(Incomplete):
        relocate(shown, TALK, newest)
    with pytest.raises(StaleOffer):
        relocate(shown, len(START["affordances"]) + 1, newest)
