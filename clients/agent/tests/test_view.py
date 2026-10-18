"""`view.py` over the recorded market-town observations: every affordance listed, numbered in order, the
requirement shown as the server wrote it, names as `WorldText`, exits from movement's disclosure, pages at
the limits, and a delta equal to the change. Expected values come from the raw fixtures (`recorded.py`).
"""

from __future__ import annotations

from typing import Any

import pytest
from recorded import NAMES, component, crowd, decoded, entity, observation, raw

from mineworld_agent.models import InboxCounts, View, json_bytes
from mineworld_agent.refs import RefTable
from mineworld_agent.view import Caps, Moment, delta_of, page_of, render, view_of

MOMENT = Moment(at=0, time_scale=1, paused=False, since=None)
INBOX = InboxCounts(unread=0, dropped=0)
CAPS = Caps()


def whole(name: str) -> View:
    return view_of(render(observation(name), RefTable(), 1, True), MOMENT, INBOX, CAPS)


def names_by_id(value: dict[str, Any]) -> dict[str, str]:
    return {
        e["id"]: payload["name"]
        for e in value["entities"]
        if (payload := component(e, "display-name")) is not None
    }


@pytest.mark.parametrize("name", NAMES)
def test_every_affordance_is_listed(name: str) -> None:
    # The step's adversarial (and M-c-5): the view lists every affordance, count for count, in order.
    value = raw(name)
    view = whole(name)
    affordances = value["affordances"]
    assert len(view.offers) == len(affordances) and not [p for p in view.more if p.list == "offers"]
    assert [o.n for o in view.offers] == list(range(1, len(affordances) + 1))
    for line, affordance in zip(view.offers, affordances, strict=True):
        assert line.action_type == affordance["action_type"]
        assert line.available == affordance["available"]
        assert line.complete == ("payload" in affordance)
        assert (line.unavailable is None) == (affordance["unavailable_reason"] is None)
        if isinstance(affordance["unavailable_reason"], str):
            assert line.unavailable is not None
            assert line.unavailable.reason == affordance["unavailable_reason"]


@pytest.mark.parametrize("name", NAMES)
def test_requirements_are_shown_as_the_server_wrote_them(name: str) -> None:
    value = raw(name)
    for line, affordance in zip(whole(name).offers, value["affordances"], strict=True):
        written = affordance["requirement"]
        place = written["place"]
        expected = (
            "specific"
            if isinstance(place, dict)
            else {"any": "any", "same_place_as_actor": "same"}[place]
        )
        assert line.requirement.place == expected
        assert (line.requirement.specific_place is None) == (expected != "specific")
        assert line.requirement.within_mm == written["within_range"]
        assert line.requirement.line_of_access == written["requires_line_of_access"]
        assert line.requirement.target_available == written["requires_target_available"]


@pytest.mark.parametrize("name", NAMES)
def test_people_are_named_as_world_text_and_offers_point_at_them(name: str) -> None:
    value = raw(name)
    names = names_by_id(value)
    view = whole(name)
    observer = value["observer"]
    assert view.me.name is not None and view.me.name.text == names[observer]
    assert view.me.name.kind == "name"
    others = {e["id"] for e in value["entities"] if e["entity_type"] == "person"} - {observer}
    assert sorted(p.name.text for p in view.here.people if p.name) == sorted(
        names[i] for i in others
    )
    ref_of_name = {p.name.text: p.ref for p in view.here.people if p.name}
    for line, affordance in zip(view.offers, value["affordances"], strict=True):
        if affordance["target"] in others:
            assert line.target == ref_of_name[names[affordance["target"]]]
    tags = {names[i]: entity(value, i)["tags"] for i in others}
    for person in view.here.people:
        assert person.name is not None
        assert [label.text for label in person.labels] == tags[person.name.text]
        assert {label.kind for label in person.labels} <= {"label"}


@pytest.mark.parametrize("name", NAMES)
def test_exits_are_movement_s_passages(name: str) -> None:
    value = raw(name)
    here = value["self_location"]["place"]["entity"]
    passages = component(entity(value, here), "passages")["leads_to"]
    view = whole(name)
    assert len(view.here.exits) == len(passages)
    for line, passage in zip(view.here.exits, passages, strict=True):
        assert line.door is not None
        assert (line.door.x_mm, line.door.y_mm) == (passage["here"]["x"], passage["here"]["y"])
        assert [label.text for label in line.leads_to.labels] == passage["to_tags"]
    assert view.me.place is not None and view.me.place.ref == view.me.position.place  # type: ignore[union-attr]


def test_my_own_walk_is_shown() -> None:
    value = raw("market-town-walking")
    walking = component(entity(value, value["observer"]), "walking")
    view = whole("market-town-walking")
    assert view.me.walking is not None and view.me.walking.next is not None
    first = walking["next"][0]
    assert (view.me.walking.next.x_mm, view.me.walking.next.y_mm) == (first["x"], first["y"])
    exits = {line.leads_to.ref for line in view.here.exits}
    assert getattr(view.me.walking.to, "place", None) in exits, (
        "walking to the street, by its exit's ref"
    )
    assert whole("market-town-start").me.walking is None


def test_lists_page_at_their_limits_and_every_entry_appears_once() -> None:
    value = crowd(people=30, things=30, offers=50)
    rendered = render(decoded(value), RefTable(), 1, True)
    first = view_of(rendered, MOMENT, INBOX, CAPS)
    people = len([e for e in value["entities"] if e["entity_type"] == "person"]) - 1
    things = len([e for e in value["entities"] if e["entity_type"] == "item"])
    seen = {
        "people": [p.ref for p in first.here.people],
        "things": [t.ref for t in first.here.things],
    }
    seen["offers"] = [str(o.n) for o in first.offers]
    assert len(first.here.people) <= 24 and len(first.here.things) <= 24 and len(first.offers) <= 40
    pending = list(first.more)
    while pending:
        page = pending.pop(0)
        shown = page_of(rendered, page, MOMENT, INBOX, CAPS)
        assert json_bytes(shown) <= CAPS.max_bytes
        seen["people"] += [p.ref for p in shown.here.people]
        seen["things"] += [t.ref for t in shown.here.things]
        seen["offers"] += [str(o.n) for o in shown.offers]
        assert not shown.here.exits
        pending += shown.more
    assert len(seen["people"]) == len(set(seen["people"])) == people
    assert len(seen["things"]) == len(set(seen["things"])) == things
    assert seen["offers"] == [str(n) for n in range(1, len(value["affordances"]) + 1)]


def test_a_delta_is_smaller_than_the_view_and_equals_the_change() -> None:
    before, after = raw("market-town-start"), raw("market-town-walking")
    refs = RefTable()
    base = render(decoded(before), refs, 1, True)
    newest = render(decoded(after), refs, 2, True)
    delta = delta_of(base, newest, MOMENT, INBOX, CAPS)
    assert delta is not None
    assert json_bytes(delta) < json_bytes(view_of(newest, MOMENT, INBOX, CAPS))
    changed = {
        e["id"]
        for e in after["entities"]
        if e["entity_type"] == "person"
        and e != next(b for b in before["entities"] if b["id"] == e["id"])
    } - {after["observer"]}
    assert len(delta.people) == len(changed) and not delta.people_gone
    assert (delta.me is None) == (
        before["self_location"] == after["self_location"]
        and entity(before, before["observer"]) == entity(after, after["observer"])
    )
    offers_changed = before["affordances"] != after["affordances"]
    assert (delta.offers is not None) == offers_changed
    if delta.offers is not None:
        assert len(delta.offers) == len(after["affordances"])
    same = delta_of(newest, newest, MOMENT, INBOX, CAPS)
    assert same is not None and same.me is None and same.offers is None and not same.people
