"""R-S27c-INFO: the view is information-rich — my money, items, job, schedule, acquaintances and invitations,
what people are doing, time, weather, and what each offer is about and costs — every field typed, every value
read from the raw observation (`recorded.py`), and the paged list of acquaintances never silently cut.

Recorded fields (wallet, holdings, agenda, weather, calendar, shop prices) are checked against the real
market-town observation. Fields the genesis café does not disclose (a job, acquaintances, invitations,
participation) are planted in it, in the shapes their packs serialize (`systems/employment/src/component.rs`
`Employment`/`Job`/`OnShift`, `relationships` `Acquaintances`, `group-activity` `Invitations`/`Participation`);
the real-binary play-test sees Alice's job on the wire.
"""

from __future__ import annotations

import json
from typing import Any

from mineworld_sdk.wire.ids import EntityId
from recorded import Raw, component, crowd, decoded, entity, raw

from mineworld_agent.models import DoingLine, InboxCounts, View, WorldText, json_bytes, render_text
from mineworld_agent.refs import RefTable
from mineworld_agent.view import Caps, Moment, Rendered, delta_of, page_of, render, view_of

MOMENT = Moment(at=0, time_scale=1, paused=False, since=None)
INBOX = InboxCounts(unread=0, dropped=0)
ME, ALICE = "17", "7"
KNOWN_AT_MOST = 150
"""relationships' default `known_at_most` (FG-a, `systems/relationships/src/interactions.rs:87`)."""


def ref(value: str) -> dict[str, str]:
    return {"entity": value, "entity_type": "person"}


def typed(value: str, kind: str) -> dict[str, str]:
    return {"entity": value, "entity_type": kind}


def plant(value: Raw, owner: str, kind: str, payload: Any) -> None:  # noqa: ANN401 (a pack's JSON)
    entity(value, owner)["components"].append(
        {"entity": owner, "component_type": kind, "schema_version": 1, "payload": payload}
    )


def rich(acquaintances: int = 3, invitations: int = 0) -> Raw:
    value = raw("market-town-start")
    plant(
        value,
        ME,
        "employment",
        {
            "job": {
                "employer": typed("40", "organization"),
                "workplace": typed("2", "place"),
                "from": "05:30",
                "until": "14:00",
                "wage": 120,
                "produces": [{"item": typed("22", "item"), "per_shift": 4}],
            },
            "shift": {"present_since": 19_800, "worked": 600},
        },
    )
    known = [
        {
            "counterpart": ref(str(30_000 + n)),
            "values": {
                "familiarity": 100 + n,
                "regard": -n,
                "exchanges": n,
                "activities_shared": 1,
                "first_met": 5,
                "first_met_by": "900",
                "last_contact": 10 + n,
            },
        }
        for n in range(acquaintances)
    ]
    plant(value, ME, "acquaintances", {"known": known})
    pending = [{"from": ref(ALICE), "kind": "chess", "at": 3, "until": 900}]
    pending += [
        {"from": ref(str(40_000 + n)), "kind": f"walk-{n}", "at": 4, "until": 1_800}
        for n in range(invitations)
    ]
    plant(value, ME, "invitations", {"pending": pending})
    plant(value, ALICE, "participation", {"activity": "55", "kind": "chess", "since": 7})
    plant(value, ME, "participation", {"activity": "56", "kind": "reading", "since": 9})
    return value


def view(value: Raw) -> tuple[View, Rendered, RefTable]:
    refs = RefTable()
    rendered = render(decoded(value), refs, 1, True)
    return view_of(rendered, MOMENT, INBOX, Caps()), rendered, refs


def test_my_money_is_my_disclosed_wallet() -> None:
    value = raw("market-town-start")
    shown, _, _ = view(value)
    assert shown.me.wallet is not None
    assert shown.me.wallet == component(entity(value, ME), "wallet")["balance"]


def test_my_holdings_are_every_held_item_by_reference_and_count() -> None:
    value = raw("market-town-start")
    held = component(entity(value, ME), "holdings")["held"]
    shown, _, refs = view(value)
    assert held, "the recorded visitor holds something"
    assert [(h.item, h.count, h.name) for h in shown.me.holdings] == [
        (refs.ref(h["item"]["entity"]), h["count"], None) for h in held
    ], "items by reference and count; names None until a pack discloses them (F-S27c-9)"


def test_my_agenda_is_the_schedule_packs() -> None:
    value = raw("market-town-start")
    agenda = component(entity(value, ME), "agenda")
    shown, _, refs = view(value)
    assert shown.me.agenda is not None
    assert shown.me.agenda.label.text == agenda["label"] and shown.me.agenda.label.kind == "label"
    assert shown.me.agenda.place == refs.ref(agenda["place"]["entity"])
    assert (shown.me.agenda.since, shown.me.agenda.until) == (agenda["since"], agenda["until"])


def test_my_plans_are_the_next_part_and_the_minutes_of_the_way_at_the_time_scale() -> None:
    """AC-LE2-6, the agent half: `MeLine.plans` from schedule's `agenda.next` and movement's `ways`, the minutes
    worked by hand from the shared formula (52 000 mm, 3 doorways: 39 + 3 + 1 = 43 strides; at 12x, 516 s,
    about 9 minutes). A place with no disclosed way has no minutes."""
    value = raw("market-town-start")
    agenda = component(entity(value, ME), "agenda")
    agenda["next"] = {
        "place": typed("2", "place"),
        "label": "work",
        "from": 19_800,
        "punctual": True,
    }
    plant(value, ME, "ways", [{"place": typed("2", "place"), "doors": 3, "length": 52_000}])
    refs = RefTable()
    plans = render(decoded(value), refs, 1, True, 12).me.plans
    assert plans is not None
    assert (plans.label.text, plans.place, plans.from_at, plans.punctual, plans.walk_minutes) == (
        "work",
        refs.ref(EntityId("2")),
        19_800,
        True,
        9,
    )
    component(entity(value, ME), "ways").clear()
    unknown = render(decoded(value), RefTable(), 1, True, 12).me.plans
    assert unknown is not None and unknown.walk_minutes is None


def test_the_calendar_of_my_place() -> None:
    value = raw("market-town-start")
    place = entity(value, value["self_location"]["place"]["entity"])
    shown, _, _ = view(value)
    day = component(place, "calendar-day")
    assert shown.calendar is not None
    assert (shown.calendar.year, shown.calendar.month, shown.calendar.day) == (
        day["date"]["year"],
        day["date"]["month"],
        day["date"]["day"],
    )
    assert shown.calendar.time_of_day_s == value["at"] - day["day_start"]
    assert shown.calendar.light == component(place, "calendar-light")["phase"]


def test_the_weather_of_my_place_this_hour() -> None:
    value = raw("market-town-start")
    place = entity(value, value["self_location"]["place"]["entity"])
    shown, _, _ = view(value)
    now = component(place, "weather-now")
    day = component(place, "calendar-day")
    hour = component(place, "weather-today")["hours"][(value["at"] - day["day_start"]) // 3600]
    assert shown.weather is not None
    assert (shown.weather.condition, shown.weather.since_hour) == (now["condition"], now["hour"])
    assert (shown.weather.temperature_dc, shown.weather.cloud_oktas) == (
        hour["temperature_dc"],
        hour["cloud_oktas"],
    )
    assert (shown.weather.precipitation_tenth_mm, shown.weather.wind_dms) == (
        hour["precipitation_tenth_mm"],
        hour["wind_dms"],
    )
    assert shown.weather.wind_from_deg == hour["wind_from_deg"]


def item_offers(value: Raw) -> list[tuple[int, str]]:
    """(index, item id) of every recorded affordance whose complete payload names an item."""
    return [
        (n, a["payload"]["item"]["entity"])
        for n, a in enumerate(value["affordances"])
        if isinstance(a.get("payload"), dict) and "item" in a["payload"]
    ]


def test_each_offer_names_what_its_payload_is_about() -> None:
    value = raw("market-town-start")
    shown, _, refs = view(value)
    about = dict(item_offers(value))
    for n, line in enumerate(shown.offers):
        assert line.about == ([refs.ref(EntityId(about[n]))] if n in about else []), line.n
    buys = [o for o in shown.offers if o.action_type == "buy"]
    assert len(buys) == 6 and len({tuple(o.about) for o in buys}) == 6, "six distinct items"


def test_each_item_offer_carries_my_shops_price() -> None:
    value = raw("market-town-start")
    place = entity(value, value["self_location"]["place"]["entity"])
    prices = {line["item"]["entity"]: line["price"] for line in component(place, "shop")["listed"]}
    shown, _, _ = view(value)
    about = dict(item_offers(value))
    for n, line in enumerate(shown.offers):
        assert line.price == (prices.get(about[n]) if n in about else None), line.n
    assert all(o.price is not None for o in shown.offers if o.action_type == "buy")


def of(refs: RefTable, entity_id: str) -> str:
    return refs.ref(EntityId(entity_id))


def test_my_job_is_the_employment_packs() -> None:
    shown, _, refs = view(rich())
    job = shown.me.job
    assert job is not None
    assert (job.employer, job.workplace) == (of(refs, "40"), of(refs, "2"))
    assert (job.from_s, job.until_s, job.wage) == (5 * 3600 + 30 * 60, 14 * 3600, 120)
    assert [(p.item, p.count) for p in job.produces] == [(of(refs, "22"), 4)]
    assert (job.on_shift, job.present, job.worked_s) == (True, True, 600)


def test_invitations_to_me_name_the_inviter_and_the_activity() -> None:
    shown, _, refs = view(rich())
    (invitation,) = shown.me.invitations
    assert invitation.by == of(refs, ALICE) and invitation.kind.text == "chess"
    assert invitation.kind.by == invitation.by and (invitation.at, invitation.until) == (3, 900)


def test_what_people_and_i_are_doing() -> None:
    shown, _, refs = view(rich())
    alice = next(p for p in shown.here.people if p.ref == of(refs, ALICE))
    assert alice.doing is not None and (alice.doing.kind.text, alice.doing.since) == ("chess", 7)
    assert all(p.doing is None for p in shown.here.people if p.ref != alice.ref)
    assert shown.me.doing is not None
    assert (shown.me.doing.kind.text, shown.me.doing.since) == ("reading", 9)


def test_acquaintances_are_read_as_the_pack_wrote_them() -> None:
    value = rich(acquaintances=3)
    known = component(entity(value, ME), "acquaintances")["known"]
    shown, _, refs = view(value)
    assert [
        (a.person, a.familiarity, a.regard, a.exchanges, a.last_contact)
        for a in shown.me.acquaintances
    ] == [
        (
            refs.ref(k["counterpart"]["entity"]),
            k["values"]["familiarity"],
            k["values"]["regard"],
            k["values"]["exchanges"],
            k["values"]["last_contact"],
        )
        for k in known
    ]


def fits(shown: View) -> None:
    assert json_bytes(shown) <= 8192 and len(render_text(shown).encode("utf-8")) <= 8192


def test_a_self_with_the_most_acquaintances_fits_by_paging_and_page_two_is_reachable() -> None:
    # Ruling 3: 150 acquaintances (FG-a's cap) page like every other list, never silently cut.
    value = rich(acquaintances=KNOWN_AT_MOST)
    shown, rendered, _ = view(value)
    fits(shown)
    first = len(shown.me.acquaintances)
    assert 0 < first < KNOWN_AT_MOST
    (more,) = [p for p in shown.more if p.list == "acquaintances"]
    assert more.start == first
    seen = [a.person for a in shown.me.acquaintances]
    pending, pages = [more], 1
    while pending:
        page = page_of(rendered, pending.pop(0), MOMENT, INBOX, Caps())
        fits(page)
        assert page.me.acquaintances, "page two and on are reachable and carry acquaintances"
        seen += [a.person for a in page.me.acquaintances]
        pending += [p for p in page.more if p.list == "acquaintances"]
        pages += 1
    assert pages >= 2
    assert len(seen) == len(set(seen)) == KNOWN_AT_MOST, "every acquaintance exactly once"


def test_many_invitations_page_and_none_is_lost() -> None:
    # At most one per inviter (group-activity `Invitations`), so a crowded town can hold many: paged.
    value = rich(acquaintances=0, invitations=KNOWN_AT_MOST)
    shown, rendered, _ = view(value)
    fits(shown)
    seen = [i.by for i in shown.me.invitations]
    pending = [p for p in shown.more if p.list == "invitations"]
    assert pending and 0 < len(seen) < KNOWN_AT_MOST + 1
    while pending:
        page = page_of(rendered, pending.pop(0), MOMENT, INBOX, Caps())
        fits(page)
        assert page.me.invitations
        seen += [i.by for i in page.me.invitations]
        pending += [p for p in page.more if p.list == "invitations"]
    assert len(seen) == len(set(seen)) == KNOWN_AT_MOST + 1, "every invitation exactly once"


def test_what_a_crowd_is_doing_pages_with_the_people() -> None:
    # Ruling 3: participation is per person, so it travels on `PersonLine` and pages with `people`.
    value = crowd(people=KNOWN_AT_MOST, things=0, offers=0)
    for n in range(KNOWN_AT_MOST):
        plant(
            value, str(10_000 + n), "participation", {"activity": "1", "kind": f"k{n}", "since": n}
        )
    shown, rendered, refs = view(value)
    pages, pending = [shown], [p for p in shown.more if p.list == "people"]
    while pending:
        page = page_of(rendered, pending.pop(0), MOMENT, INBOX, Caps())
        pages.append(page)
        pending += [p for p in page.more if p.list == "people"]
    for page in pages:
        fits(page)
    doing = {p.ref: p.doing for page in pages for p in page.here.people if p.doing is not None}
    assert len(pages) >= 2
    assert doing == {
        refs.ref(EntityId(str(10_000 + n))): DoingLine(
            kind=WorldText(text=f"k{n}", kind="label"), since=n
        )
        for n in range(KNOWN_AT_MOST)
    }


def test_a_change_to_a_paged_list_of_mine_answers_a_whole_view() -> None:
    refs = RefTable()
    before = render(decoded(rich(acquaintances=3)), refs, 1, True)
    same = render(decoded(rich(acquaintances=3)), refs, 2, True)
    after = render(decoded(rich(acquaintances=4)), refs, 3, True)
    invited = render(decoded(rich(acquaintances=3, invitations=1)), refs, 4, True)
    assert delta_of(before, same, MOMENT, INBOX, Caps()) is not None
    assert delta_of(before, after, MOMENT, INBOX, Caps()) is None
    assert delta_of(before, invited, MOMENT, INBOX, Caps()) is None


# ── My attributes (S30-f §5.8): my own sheet, typed, paged, in rank order ───────────────────────────────


def attribute_block(count: int) -> dict[str, Any]:
    return {
        "schema": 2,
        "entries": [
            {
                "attribute": f"test.a{n:03d}",
                "kind": "level",
                "label": f"A{n:03d}",
                "facets": ["prompt"],
                "rank": n,
                "value": {"level": f"v{n}", "rank": n % 5},
                "text": f"phrase {n:03d}",
            }
            for n in range(count)
        ],
    }


def test_my_attributes_are_my_sheet_typed_as_world_text() -> None:
    # The holder's view of every kind: text when rendered, the typed value when not; nothing shown, no line.
    value = raw("market-town-start")
    entries: list[dict[str, Any]] = [
        {"attribute": "test.age", "kind": "level", "label": "Age", "facets": ["observed"],
         "rank": 10, "value": {"level": "sixties", "rank": 6}, "text": "in her sixties"},
        {"attribute": "test.height", "kind": "quantity", "label": "Height", "facets": [],
         "rank": 20, "value": {"quantity": "1.660"}},
        {"attribute": "test.tongues", "kind": "set", "label": "Languages", "facets": [],
         "rank": 30, "value": {"set": ["spanish", "english"]}},
        {"attribute": "test.void", "kind": "set", "label": "Pets", "facets": [],
         "rank": 40, "value": {"set": []}},
        {"attribute": "test.born", "kind": "instant", "label": "Birth", "facets": [],
         "value": {"instant": -1000}},
        {"attribute": "test.glasses", "kind": "flag", "label": "Glasses", "facets": [],
         "rank": 35, "value": {"flag": False}},
    ]  # fmt: skip
    plant(value, ME, "traits", {"schema": 2, "entries": entries})
    plant(value, ME, "employment-like", {"attributes": {"schema": 2, "entries": [
        {"attribute": "work.occupation", "kind": "text", "label": "Occupation", "facets": ["prompt"],
         "rank": 15, "value": {"text": "barista"}, "text": "works as barista"}]}})  # fmt: skip
    shown, _, _ = view(value)
    lines = shown.me.attributes
    assert [line.attribute.text for line in lines] == [
        "test.age",
        "work.occupation",
        "test.height",
        "test.tongues",
        "test.glasses",
        "test.born",
    ]
    assert all(
        text.kind == "label"
        for line in lines
        for text in (line.attribute, line.label, *([line.text] if line.text else []))
    )
    age, job, height, tongues, glasses, born = lines
    assert age.text is not None and age.text.text == "in her sixties"
    assert age.value is not None and age.value.slug is not None and age.value.slug.text == "sixties"
    assert age.value.rank == 6
    assert job.value is not None and job.value.text is not None and job.value.text.text == "barista"
    assert height.text is None and height.value is not None and height.value.quantity == "1.660"
    assert tongues.value is not None and tongues.value.members is not None
    assert [m.text for m in tongues.value.members] == ["english", "spanish"]
    assert glasses.value is not None and glasses.value.flag is False
    assert born.value is not None and born.value.instant == -1000
    dumped = json.loads(shown.model_dump_json())["me"]["attributes"]
    assert dumped[2] == {
        "attribute": {"text": "test.height", "by": None, "kind": "label"},
        "label": {"text": "Height", "by": None, "kind": "label"},
        "kind": "quantity",
        "value": {"quantity": "1.660"},
    }


def test_a_block_that_does_not_read_adds_no_attribute() -> None:
    value = raw("market-town-start")
    plant(value, ME, "traits", {"schema": 3, "entries": []})
    shown, _, _ = view(value)
    assert shown.me.attributes == [] and '"attributes"' not in shown.model_dump_json()


def test_many_attributes_page_by_thirty_two_in_rank_order_and_none_is_lost() -> None:
    value = raw("market-town-start")
    plant(value, ME, "traits", attribute_block(70))
    shown, rendered, _ = view(value)
    fits(shown)
    assert 0 < len(shown.me.attributes) <= 32, "the byte cap may shorten the first page"
    roomy = view_of(rendered, MOMENT, INBOX, Caps(max_bytes=1 << 20))
    assert len(roomy.me.attributes) == 32, "a page holds 32 attributes"
    seen = [line.attribute.text for line in shown.me.attributes]
    pending = [p for p in shown.more if p.list == "attributes"]
    while pending:
        page = page_of(rendered, pending.pop(0), MOMENT, INBOX, Caps())
        fits(page)
        seen += [line.attribute.text for line in page.me.attributes]
        pending += [p for p in page.more if p.list == "attributes"]
    assert seen == [f"test.a{n:03d}" for n in range(70)]


def test_a_change_to_my_attributes_answers_a_whole_view() -> None:
    refs = RefTable()
    with_two, with_three = raw("market-town-start"), raw("market-town-start")
    plant(with_two, ME, "traits", attribute_block(2))
    plant(with_three, ME, "traits", attribute_block(3))
    before = render(decoded(with_two), refs, 1, True)
    same = render(decoded(with_two), refs, 2, True)
    after = render(decoded(with_three), refs, 3, True)
    assert delta_of(before, same, MOMENT, INBOX, Caps()) is not None
    assert delta_of(before, after, MOMENT, INBOX, Caps()) is None
    assert before.me.attributes == [], "kept apart: a delta's me never carries them"
