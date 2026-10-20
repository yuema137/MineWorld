"""One observation of a seat, rendered compactly for an agent (step-27 §§5.3, 5.10).

A view is that one `Observation` and nothing else (INV-S27-2). Every entity it shows gets a short
reference from the handle's own `RefTable`; every affordance is listed, numbered in the observation's
order, never filtered (the step's adversarial: count equality); requirements are shown, never evaluated.
Names and tags arrive as `WorldText`. Lists are paged (people ≤ 24, things ≤ 24, offers ≤ 40) and a result
is fitted under the byte cap by shortening its largest list, never by cutting a field.

The only pack payloads read by name are the ones a client knows (step-27 §4.1; pr-s27c C-8): naming's
`display-name`, movement's `passages` and `walking`, and the calendar pack's `calendar-day` and
`calendar-light` for the clock. Anything else is carried, uninterpreted, by `inspect`.
"""

from __future__ import annotations

import json
from collections.abc import Sequence
from dataclasses import dataclass

from mineworld_sdk.wire.contract import (
    Affordance,
    Location,
    Observation,
    PerceivedEntity,
    Rejection,
    SpatialRequirement,
    SpecificPlace,
    SystemRejection,
)
from mineworld_sdk.wire.ids import EntityId, JsonValue

from mineworld_agent import disclosed
from mineworld_agent.models import (
    AcquaintanceLine,
    AttributeLine,
    CalendarLine,
    ComponentPayload,
    Destination,
    Envelope,
    ExitLine,
    Here,
    InboxCounts,
    InvitationLine,
    ListName,
    MeLine,
    OfferLine,
    PageRef,
    PersonLine,
    PlaceLine,
    Point,
    ReasonLine,
    RequirementLine,
    ThingLine,
    ToPerson,
    ToPlace,
    ToPoint,
    View,
    ViewDelta,
    WalkLine,
    WeatherLine,
    WorldText,
    json_bytes,
)
from mineworld_agent.refs import RefTable
from mineworld_agent.roles import roles_of

DISPLAY_NAME = "display-name"
PASSAGES = "passages"
WALKING = "walking"
CALENDAR_DAY = "calendar-day"
CALENDAR_LIGHT = "calendar-light"
INSPECT_BYTES = 4096
"""The most component payload one `inspect` shows, in bytes of compact JSON (step-27 §5.3)."""


@dataclass(frozen=True, slots=True)
class Caps:
    """Page sizes and the byte cap (step-27 §5.10); configuration may lower them, never raise them."""

    people: int = 24
    things: int = 24
    offers: int = 40
    acquaintances: int = 24
    invitations: int = 24
    attributes: int = 32
    max_bytes: int = 8192


@dataclass(frozen=True, slots=True)
class Moment:
    """The time fields of a result (step-27 §5.9)."""

    at: int
    time_scale: int
    paused: bool
    since: int | None


@dataclass(frozen=True, slots=True)
class Rendered:
    """One observation, rendered whole: what views, pages, deltas and re-location are made from."""

    number: int
    observation: Observation
    acted_through: bool
    me: MeLine
    people: tuple[PersonLine, ...]
    things: tuple[ThingLine, ...]
    exits: tuple[ExitLine, ...]
    offers: tuple[OfferLine, ...]
    acquaintances: tuple[AcquaintanceLine, ...] = ()
    """`me.acquaintances`, kept apart so that it pages like the other lists."""
    invitations: tuple[InvitationLine, ...] = ()
    """`me.invitations`, kept apart for the same reason."""
    attributes: tuple[AttributeLine, ...] = ()
    """`me.attributes`, kept apart for the same reason."""
    calendar: CalendarLine | None = None
    weather: WeatherLine | None = None


class StaleOffer(LookupError):
    """The offer is no longer in the newest observation as it was shown, or no longer complete."""


class Incomplete(LookupError):
    """The offer was shown incomplete: it carries no payload, so it needs its own operation (`say`,
    `walk_to`, `move`), never `attempt` (step-27 §5.5)."""


# ── Rendering ─────────────────────────────────────────────────────────────────────────────────────


def _payload(entity: PerceivedEntity | None, component: str) -> JsonValue:
    if entity is None:
        return None
    for record in entity.components:
        if record.component_type == component:
            return record.payload
    return None


def _name(entity: PerceivedEntity | None) -> WorldText | None:
    payload = _payload(entity, DISPLAY_NAME)
    if isinstance(payload, dict) and isinstance(name := payload.get("name"), str):
        return WorldText(text=name, kind="name")
    return None


def _labels(tags: Sequence[str]) -> list[WorldText]:
    return [WorldText(text=tag, kind="label") for tag in tags]


def place_line(refs: RefTable, place: EntityId, tags: Sequence[str]) -> PlaceLine:
    """A place by reference, with its role tags first and every tag as a label (step-29 §9.4)."""
    return PlaceLine(ref=refs.ref(place), roles=roles_of(tags), labels=_labels(tags))


def _entity_id(value: JsonValue) -> EntityId | None:
    """The `entity` of a typed reference (`{ entity, entity_type }`), as packs write `PlaceId`/`PersonId`."""
    if isinstance(value, dict) and isinstance(entity := value.get("entity"), str):
        return EntityId(entity)
    return None


def _point(refs: RefTable, place: EntityId, local: JsonValue) -> Point | None:
    if not isinstance(local, dict):
        return None
    x, y, z = local.get("x"), local.get("y"), local.get("z")
    if not isinstance(x, int) or not isinstance(y, int):
        return None
    return Point(place=refs.ref(place), x_mm=x, y_mm=y, z_mm=z if isinstance(z, int) else None)


def _located(refs: RefTable, location: Location | None) -> Point | None:
    if location is None or location.local is None:
        return None
    local = location.local
    return Point(place=refs.ref(location.place.entity), x_mm=local.x, y_mm=local.y, z_mm=local.z)


def _walk(refs: RefTable, entity: PerceivedEntity, here: EntityId | None) -> WalkLine | None:
    payload = _payload(entity, WALKING)
    if not isinstance(payload, dict):
        return None
    destination = payload.get("destination")
    to: Destination | None = None
    if isinstance(destination, dict):
        place = destination.get("place")
        person = _entity_id(destination.get("person"))
        if isinstance(place, dict) and (where := _entity_id(place.get("place"))) is not None:
            point = _point(refs, where, place.get("local"))
            to = ToPlace(place=refs.ref(where)) if point is None else ToPoint(point=point)
        elif person is not None:
            to = ToPerson(person=refs.ref(person))
    upcoming = payload.get("next")
    first = upcoming[0] if isinstance(upcoming, list) and upcoming else None
    return WalkLine(to=to, next=None if here is None else _point(refs, here, first))


def _reason(rejection: Rejection | None) -> ReasonLine | None:
    if rejection is None:
        return None
    if isinstance(rejection, SystemRejection):
        return ReasonLine(reason="system", system_code=rejection.system.code)
    return ReasonLine(reason=rejection)


def reason_line(rejection: Rejection) -> ReasonLine:
    """A rejection's kind and, for a System's own code, the code (also used for an act's answer)."""
    line = _reason(rejection)
    assert line is not None
    return line


def _requirement(refs: RefTable, requirement: SpatialRequirement) -> RequirementLine:
    place = requirement.place
    specific = refs.ref(place.specific.entity) if isinstance(place, SpecificPlace) else None
    return RequirementLine(
        place="specific" if specific else ("same" if place == "same_place_as_actor" else "any"),
        specific_place=specific,
        within_mm=requirement.within_range,
        line_of_access=requirement.requires_line_of_access,
        target_available=requirement.requires_target_available,
    )


def offer_line(
    refs: RefTable, n: int, affordance: Affordance, prices: dict[EntityId, int] | None = None
) -> OfferLine:
    """One offer. For a complete one, `about` names the entities its payload names (the item a `buy`
    sells) and `price` is my place's shop price for the one item it names, if the shop lists it."""
    about = disclosed.named_entities(affordance.payload)
    priced = [prices[e] for e in about if prices and e in prices]
    return OfferLine(
        n=n,
        action_type=affordance.action_type,
        target=None if affordance.target is None else refs.ref(affordance.target),
        available=affordance.available,
        unavailable=_reason(affordance.unavailable_reason),
        requirement=_requirement(refs, affordance.requirement),
        complete=affordance.payload is not None,
        about=[refs.ref(entity) for entity in about],
        price=priced[0] if len(priced) == 1 else None,
    )


def _exits(refs: RefTable, place: PerceivedEntity | None, here: EntityId) -> list[ExitLine]:
    payload = _payload(place, PASSAGES)
    leads = payload.get("leads_to") if isinstance(payload, dict) else None
    exits: list[ExitLine] = []
    for passage in leads if isinstance(leads, list) else []:
        if not isinstance(passage, dict) or (to := _entity_id(passage.get("to"))) is None:
            continue
        tags = passage.get("to_tags")
        labels = [t for t in tags if isinstance(t, str)] if isinstance(tags, list) else []
        exits.append(
            ExitLine(
                door=_point(refs, here, passage.get("here")),
                leads_to=place_line(refs, to, labels),
            )
        )
    return exits


def render(
    observation: Observation, refs: RefTable, number: int, acted_through: bool, stride_s: int = 1
) -> Rendered:
    """The whole rendering of one observation. References are assigned in a fixed order: me, my place,
    then every entity in the observation's order, then exits and offers' targets. `stride_s` is the
    world seconds of one paced stride (the session's `time_scale`), for `MeLine.plans`."""
    me_id = observation.observer
    refs.ref(me_id)
    here = observation.self_location.place.entity if observation.self_location else None
    if here is not None:
        refs.ref(here)
    by_id = {entity.id: entity for entity in observation.entities}
    for entity in observation.entities:
        refs.ref(entity.id)
    mine = by_id.get(me_id)
    place_entity = None if here is None else by_id.get(here)
    location = observation.self_location
    facing = None if location is None or location.facing is None else location.facing.yaw
    me = MeLine(
        ref=refs.ref(me_id),
        name=_name(mine),
        place=None
        if here is None
        else place_line(refs, here, place_entity.tags if place_entity is not None else ()),
        position=_located(refs, location),
        facing_deg=None if facing is None else round(facing / 1000),
        walking=None if mine is None else _walk(refs, mine, here),
        doing=disclosed.doing(mine),
        wallet=disclosed.wallet(mine),
        holdings=disclosed.holdings(refs, mine),
        job=disclosed.job(refs, mine),
        agenda=disclosed.agenda(refs, mine),
        plans=disclosed.plans(refs, mine, stride_s),
        needs=disclosed.needs(mine),
        asleep=disclosed.asleep(mine),
    )
    acquaintances = disclosed.acquaintances(refs, mine)
    invitations = disclosed.invitations(refs, mine)
    sheet = disclosed.my_attributes(mine)
    calendar = calendar_of(observation)
    people: list[PersonLine] = []
    things: list[ThingLine] = []
    for entity in observation.entities:
        if entity.id == me_id or entity.id == here:
            continue
        if entity.entity_type == "person":
            people.append(
                PersonLine(
                    ref=refs.ref(entity.id),
                    name=_name(entity),
                    labels=_labels(entity.tags),
                    position=_located(refs, entity.location),
                    walking=_walk(refs, entity, here),
                    doing=disclosed.doing(entity),
                    asleep=disclosed.asleep(entity),
                    description=disclosed.description(entity),
                )
            )
        else:
            things.append(
                ThingLine(
                    ref=refs.ref(entity.id),
                    entity_type=entity.entity_type,
                    name=_name(entity),
                    labels=_labels(entity.tags),
                    position=_located(refs, entity.location),
                )
            )
    exits = [] if here is None else _exits(refs, place_entity, here)
    prices = disclosed.shop_prices(place_entity)
    offers = [
        offer_line(refs, n, a, prices) for n, a in enumerate(observation.affordances, start=1)
    ]
    return Rendered(
        number=number,
        observation=observation,
        acted_through=acted_through,
        me=me,
        people=tuple(people),
        things=tuple(things),
        exits=tuple(exits),
        offers=tuple(offers),
        acquaintances=tuple(acquaintances),
        invitations=tuple(invitations),
        attributes=tuple(sheet),
        calendar=calendar,
        weather=disclosed.weather(
            place_entity, None if calendar is None else calendar.time_of_day_s
        ),
    )


# ── Views, pages and the cap ──────────────────────────────────────────────────────────────────────


@dataclass(slots=True)
class _Window:
    """How much of each list a result shows, from where."""

    people: tuple[int, int]
    things: tuple[int, int]
    offers: tuple[int, int]
    acquaintances: tuple[int, int]
    invitations: tuple[int, int]
    attributes: tuple[int, int] = (0, 0)
    exits: bool = True


def _totals(rendered: Rendered) -> dict[ListName, int]:
    return {
        "people": len(rendered.people),
        "things": len(rendered.things),
        "offers": len(rendered.offers),
        "acquaintances": len(rendered.acquaintances),
        "invitations": len(rendered.invitations),
        "attributes": len(rendered.attributes),
    }


def _windows(window: _Window) -> dict[ListName, tuple[int, int]]:
    return {
        "people": window.people,
        "things": window.things,
        "offers": window.offers,
        "acquaintances": window.acquaintances,
        "invitations": window.invitations,
        "attributes": window.attributes,
    }


def _limits(caps: Caps) -> dict[ListName, int]:
    return {
        "people": caps.people,
        "things": caps.things,
        "offers": caps.offers,
        "acquaintances": caps.acquaintances,
        "invitations": caps.invitations,
        "attributes": caps.attributes,
    }


def _more(rendered: Rendered, window: _Window) -> list[PageRef]:
    totals = _totals(rendered)
    return [
        PageRef(list=name, view=rendered.number, start=start + count)
        for name, (start, count) in _windows(window).items()
        if count > 0 and start + count < totals[name]
    ]


def _view(rendered: Rendered, moment: Moment, inbox: InboxCounts, window: _Window) -> View:
    (p0, pn), (t0, tn), (o0, on) = window.people, window.things, window.offers
    (a0, an), (i0, in_) = window.acquaintances, window.invitations
    s0, sn = window.attributes
    me = rendered.me.model_copy(
        update={
            "acquaintances": list(rendered.acquaintances[a0 : a0 + an]),
            "invitations": list(rendered.invitations[i0 : i0 + in_]),
            "attributes": list(rendered.attributes[s0 : s0 + sn]),
        }
    )
    return View(
        at=moment.at,
        time_scale=moment.time_scale,
        paused=moment.paused,
        world_seconds_since_last_call=moment.since,
        view=rendered.number,
        acted_through=rendered.acted_through,
        me=me,
        here=Here(
            people=list(rendered.people[p0 : p0 + pn]),
            things=list(rendered.things[t0 : t0 + tn]),
            exits=list(rendered.exits) if window.exits else [],
        ),
        offers=list(rendered.offers[o0 : o0 + on]),
        calendar=rendered.calendar,
        weather=rendered.weather,
        inbox=inbox,
        more=_more(rendered, window),
    )


def _shown(start: int, cap: int, total: int) -> tuple[int, int]:
    return start, max(0, min(cap, total - start))


def _fit(
    rendered: Rendered, moment: Moment, inbox: InboxCounts, window: _Window, caps: Caps
) -> View:
    """Shortens the largest shown list one entry at a time until the result fits `caps.max_bytes`."""
    while True:
        view = _view(rendered, moment, inbox, window)
        if json_bytes(view) <= caps.max_bytes:
            return view
        name, (start, count) = max(_windows(window).items(), key=lambda item: item[1][1])
        if count <= 1:
            raise OverflowError(f"a view of {json_bytes(view)} bytes cannot be paged under the cap")
        setattr(window, name, (start, count - 1))


def view_of(rendered: Rendered, moment: Moment, inbox: InboxCounts, caps: Caps) -> View:
    """The first page of every list, fitted under the cap."""
    totals, limits = _totals(rendered), _limits(caps)
    window = _Window(
        people=_shown(0, limits["people"], totals["people"]),
        things=_shown(0, limits["things"], totals["things"]),
        offers=_shown(0, limits["offers"], totals["offers"]),
        acquaintances=_shown(0, limits["acquaintances"], totals["acquaintances"]),
        invitations=_shown(0, limits["invitations"], totals["invitations"]),
        attributes=_shown(0, limits["attributes"], totals["attributes"]),
    )
    return _fit(rendered, moment, inbox, window, caps)


def page_of(
    rendered: Rendered, page: PageRef, moment: Moment, inbox: InboxCounts, caps: Caps
) -> View:
    """One page of one list of a held view; the other lists are empty, the exits left out."""
    totals, limits = _totals(rendered), _limits(caps)
    none = (0, 0)
    window = _Window(
        people=none, things=none, offers=none, acquaintances=none, invitations=none, exits=False
    )
    setattr(window, page.list, _shown(page.start, limits[page.list], totals[page.list]))
    return _fit(rendered, moment, inbox, window, caps)


def delta_of(
    base: Rendered, newest: Rendered, moment: Moment, inbox: InboxCounts, caps: Caps
) -> ViewDelta | None:
    """What changed from `base` to `newest`, or `None` when the delta would not fit the cap (the caller
    then answers a whole view). A change to my acquaintances, invitations or attributes, paged lists of
    `me`, also answers a whole view."""
    if (newest.acquaintances, newest.invitations, newest.attributes) != (
        base.acquaintances,
        base.invitations,
        base.attributes,
    ):
        return None
    people_before = {line.ref: line for line in base.people}
    people_now = {line.ref: line for line in newest.people}
    things_before = {line.ref: line for line in base.things}
    things_now = {line.ref: line for line in newest.things}
    delta = ViewDelta(
        at=moment.at,
        time_scale=moment.time_scale,
        paused=moment.paused,
        world_seconds_since_last_call=moment.since,
        view=newest.number,
        base_view=base.number,
        acted_through=newest.acted_through,
        me=None if newest.me == base.me else newest.me,
        people=[line for ref, line in people_now.items() if people_before.get(ref) != line],
        people_gone=[ref for ref in people_before if ref not in people_now],
        things=[line for ref, line in things_now.items() if things_before.get(ref) != line],
        things_gone=[ref for ref in things_before if ref not in things_now],
        exits=None if newest.exits == base.exits else list(newest.exits),
        offers=None if newest.offers == base.offers else list(newest.offers),
        calendar=None if newest.calendar == base.calendar else newest.calendar,
        weather=None if newest.weather == base.weather else newest.weather,
        inbox=inbox,
        more=[],
    )
    return delta if json_bytes(delta) <= caps.max_bytes else None


# ── Re-location (step-27 §5.1, P6's rule) ─────────────────────────────────────────────────────────


def _same(shown: Affordance, now: Affordance) -> bool:
    return (shown.action_type, shown.target, shown.payload) == (
        now.action_type,
        now.target,
        now.payload,
    )


def relocate(shown: Rendered, n: int, newest: Rendered) -> Affordance:
    """Offer `n` of view `shown`, found in the newest observation: at its position first, then by
    `(action_type, target)` — and, for a complete offer, the same payload, so that a different item is
    never bought in its place. Raises `StaleOffer` when it is gone or no longer complete."""
    if not 1 <= n <= len(shown.observation.affordances):
        raise StaleOffer(f"view {shown.number} has no offer {n}")
    wanted = shown.observation.affordances[n - 1]
    if wanted.payload is None:
        raise Incomplete(f"{wanted.action_type!r} needs its own operation")
    current = newest.observation.affordances
    found = current[n - 1] if n <= len(current) and _same(wanted, current[n - 1]) else None
    if found is None:
        found = next((a for a in current if _same(wanted, a)), None)
    if found is None or found.payload is None:
        raise StaleOffer(f"offer {n} of view {shown.number} is not offered as it was")
    return found


# ── Inspect and the clock ─────────────────────────────────────────────────────────────────────────


def _compact(value: JsonValue) -> int:
    return len(json.dumps(value, ensure_ascii=False, separators=(",", ":")).encode("utf-8"))


def cut(payload: JsonValue, budget: int) -> tuple[JsonValue, bool]:
    """`payload` if it fits `budget` bytes, else its largest whole prefix that does — the first items of
    an array, the first members of an object — never a value cut inside. A scalar that does not fit is
    dropped (`None`). Returns the value shown and whether anything was cut."""
    if _compact(payload) <= budget:
        return payload, False
    if isinstance(payload, list):
        kept: list[JsonValue] = []
        for item in payload:
            if _compact([*kept, item]) > budget:
                break
            kept.append(item)
        return kept, True
    if isinstance(payload, dict):
        members: dict[str, JsonValue] = {}
        for key, item in payload.items():
            if _compact({**members, key: item}) > budget:
                break
            members[key] = item
        return members, True
    return None, True


def envelope(payload: JsonValue, budget: int) -> Envelope:
    shown, truncated = cut(payload, budget)
    return Envelope(payload=shown, bytes=_compact(payload), truncated=truncated)


def components_of(entity: PerceivedEntity, budget: int = INSPECT_BYTES) -> list[ComponentPayload]:
    """Every component perception disclosed for `entity`, in order, sharing `budget` bytes of payload."""
    out: list[ComponentPayload] = []
    left = budget
    for record in entity.components:
        made = envelope(record.payload, max(left, 0))
        left -= _compact(made.payload)
        out.append(ComponentPayload(component=record.component_type, envelope=made))
    return out


def entity_name(entity: PerceivedEntity) -> WorldText | None:
    return _name(entity)


def entity_labels(entity: PerceivedEntity) -> list[WorldText]:
    return _labels(entity.tags)


def calendar_of(observation: Observation) -> CalendarLine | None:
    """The calendar pack's date and time of day, from the observer's place, when it discloses them."""
    here = observation.self_location.place.entity if observation.self_location else None
    place = next((e for e in observation.entities if e.id == here), None)
    day = _payload(place, CALENDAR_DAY)
    if not isinstance(day, dict):
        return None
    date, start = day.get("date"), day.get("day_start")
    if not isinstance(date, dict) or not isinstance(start, int):
        return None
    year, month, number = date.get("year"), date.get("month"), date.get("day")
    if not isinstance(year, int) or not isinstance(month, int) or not isinstance(number, int):
        return None
    light = _payload(place, CALENDAR_LIGHT)
    phase = light.get("phase") if isinstance(light, dict) else None
    return CalendarLine(
        year=year,
        month=month,
        day=number,
        time_of_day_s=observation.at - start,
        light=phase if isinstance(phase, str) else None,
    )
