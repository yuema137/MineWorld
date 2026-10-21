"""Places carry their role tags ahead of their labels, and the role tags change nothing else (S29-g, U-1 … U-6).

The oracles are literal values written here, the raw recorded or planted payloads, and `docs/CORE_CONCEPTS.md`'s
own line; never a value the adapter reports about itself (test rules §25).
"""

from __future__ import annotations

import copy
import json
import re
from pathlib import Path
from typing import Any, get_args

import pytest
from recorded import Raw, decoded, raw

from mineworld_agent.models import InboxCounts, View
from mineworld_agent.refs import RefTable
from mineworld_agent.roles import ROLE_TAGS, RoleTag, roles_of
from mineworld_agent.view import Caps, Moment, components_of, render, view_of

MOMENT = Moment(at=0, time_scale=1, paused=False, since=None)
INBOX = InboxCounts(unread=0, dropped=0)
CAPS = Caps()
REPOSITORY = Path(__file__).resolve().parents[3]


def view(value: Raw) -> View:
    return view_of(render(decoded(value), RefTable(), 1, True), MOMENT, INBOX, CAPS)


def place_of(value: Raw) -> Raw:
    here = value["self_location"]["place"]["entity"]
    found: Raw = next(e for e in value["entities"] if e["id"] == here)
    return found


def passages(value: Raw) -> list[Any]:
    place = place_of(value)
    for component in place["components"]:
        if component["component_type"] == "passages":
            leads: list[Any] = component["payload"]["leads_to"]
            return leads
    return []


def test_u1_exits_carry_their_role_tags_and_every_label() -> None:
    value = raw("market-town-street")
    rendered = view(value)
    leads = passages(value)
    assert len(rendered.here.exits) == len(leads)
    for line, passage in zip(rendered.here.exits, leads, strict=True):
        tags = [t for t in passage.get("to_tags", []) if isinstance(t, str)]
        assert line.leads_to.roles == [t for t in ROLE_TAGS if t in tags]
        assert [label.text for label in line.leads_to.labels] == tags
    # No reference moved: the same observation with every role tag removed gives the same refs.
    stripped = copy.deepcopy(value)
    for passage in passages(stripped):
        passage["to_tags"] = [t for t in passage.get("to_tags", []) if t not in ROLE_TAGS]
    plain = view(stripped)
    assert [e.leads_to.ref for e in plain.here.exits] == [
        e.leads_to.ref for e in rendered.here.exits
    ]
    if not any("station" in p.get("to_tags", []) for p in leads):
        pytest.skip(
            "market-town-street.json predates S29-a's station; S29-a2 re-records it (R-S29a-g-1)"
        )
    hall = next(
        e for e, p in zip(rendered.here.exits, leads, strict=True) if "station" in p["to_tags"]
    )
    assert hall.leads_to.roles == ["station", "landmark"]


def test_u2_roles_are_exact_once_and_in_vocabulary_order() -> None:
    value = raw("market-town-street")
    # `stations` and `station-hall` are near misses; a tag is lowercase by the contract, so case cannot differ.
    tags = [
        "outdoor",
        "landmark",
        "stations",
        "station-hall",
        "stop",
        "platform",
        "station",
        "station",
    ]
    place_of(value)["tags"] = tags
    place = view(value).me.place
    assert place is not None
    assert place.roles == ["station", "stop", "platform", "landmark"]
    assert [label.text for label in place.labels] == tags


def test_u3_role_tags_change_nothing_else() -> None:
    planted = raw("market-town-street")
    place_of(planted)["tags"] = [*place_of(planted)["tags"], "station", "landmark"]
    for passage in passages(planted):
        passage["to_tags"] = [*passage.get("to_tags", []), "stop", "platform"]
    stripped = copy.deepcopy(planted)
    place_of(stripped)["tags"] = [t for t in place_of(stripped)["tags"] if t not in ROLE_TAGS]
    for passage in passages(stripped):
        passage["to_tags"] = [t for t in passage["to_tags"] if t not in ROLE_TAGS]
    a, b = view(planted), view(stripped)
    assert a.offers == b.offers
    assert a.here.people == b.here.people
    assert a.here.things == b.here.things
    assert [e.door for e in a.here.exits] == [e.door for e in b.here.exits]
    assert [e.leads_to.ref for e in a.here.exits] == [e.leads_to.ref for e in b.here.exits]
    assert a.me.model_copy(update={"place": None}) == b.me.model_copy(update={"place": None})
    assert a.me.place is not None
    assert a.me.place.roles == ["station", "landmark"]


def test_u4_an_unknown_departures_record_is_carried_verbatim_never_read() -> None:
    value = raw("market-town-street")
    place_of(value)["tags"] = [*place_of(value)["tags"], "station"]
    without = view(value).model_dump_json()
    planted = copy.deepcopy(value)
    payload = {"lines": [{"line": "coast", "next": 30600}]}
    here = place_of(planted)
    here["components"].append(
        {
            "entity": here["id"],
            "component_type": "departures",
            "schema_version": 1,
            "payload": payload,
        }
    )
    assert view(planted).model_dump_json() == without
    observed = decoded(planted)
    entity = next(e for e in observed.entities if e.id == here["id"])
    # A budget large enough for every component, as `inspect` asked about this one place gives.
    carried = [c for c in components_of(entity, 1 << 20) if c.component == "departures"]
    assert len(carried) == 1
    assert carried[0].envelope.payload == payload
    assert not carried[0].envelope.truncated
    rendered = json.loads(without)
    text = json.dumps(
        {k: v for k, v in rendered.items() if k != "here"} | {"exits": rendered["here"]["exits"]}
    ).lower()
    for word in ("service", "departure", "closed", "timetable"):
        assert word not in text, word


def test_u5_the_vocabulary_is_the_specifications() -> None:
    concepts = (REPOSITORY / "docs" / "CORE_CONCEPTS.md").read_text(encoding="utf-8")
    line = next(line for line in concepts.splitlines() if "The role tags are:" in line)
    assert tuple(re.findall(r"`([^`]+)`", line.split("The role tags are:", 1)[1])) == ROLE_TAGS
    assert get_args(RoleTag.__value__) == ROLE_TAGS


def test_u6_roles_never_read_a_key_or_a_name() -> None:
    first = raw("market-town-street")
    place_of(first)["tags"] = ["station", "public", "landmark"]
    second = copy.deepcopy(first)
    place = place_of(second)
    for component in place["components"]:
        if component["component_type"] == "display-name":
            component["payload"]["name"] = "Somewhere Else"
    a, b = view(first).me.place, view(second).me.place
    assert a is not None and b is not None
    assert a.roles == b.roles == ["station", "landmark"]
    assert roles_of(["landmark", "station"]) == roles_of(["station", "landmark"])
