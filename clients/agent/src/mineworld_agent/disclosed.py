"""What the packs disclose about me and the people I perceive, read into typed lines (R-S27c-INFO).

The operator's principle for the agent interface — "给agent的接口是信息丰富的", information-rich — means that
whatever a human can see or infer on screen reaches the agent in structured form, within the same
perception (ledger §15, the gap table). Each function here reads one pack's own component, as that pack
writes it, and returns `None` (or nothing) for anything it does not recognise: a pack that is not installed,
a component not disclosed to this observer, or a shape a newer pack writes. Nothing here evaluates a rule.

| Component        | Pack           | Disclosed to         | Line                         |
| ---------------- | -------------- | -------------------- | ---------------------------- |
| `wallet`         | economy        | the holder           | `MeLine.wallet`              |
| `holdings`       | inventory      | the holder           | `MeLine.holdings`            |
| `employment`     | employment     | the employee         | `MeLine.job`                 |
| `agenda`         | schedule       | the person           | `MeLine.agenda`, `MeLine.plans` |
| `ways`           | movement       | the person           | `MeLine.plans.walk_minutes`  |
| `acquaintances`  | relationships  | the holder           | `MeLine.acquaintances`       |
| `invitations`    | group-activity | the invitee          | `MeLine.invitations`         |
| `participation`  | group-activity | anyone perceiving    | `MeLine.doing`, `PersonLine.doing` |
| `shop`           | economy        | anyone in the place  | `OfferLine.price`            |
| `weather-now`, `weather-today` | weather | anyone in the place | `View.weather`        |
| `needs`          | needs          | the holder           | `MeLine.needs`               |
| `asleep`         | sleep          | the sleeper sees `alarm`; anyone perceiving sees `since` | `MeLine.asleep`, `PersonLine.asleep` |
| `traits`         | attributes     | the holder: every non-hidden entry; anyone else: perceivable ones | `MeLine.attributes`, `PersonLine.description` |
| an owner's `attributes` member | the owning pack | as `traits` | `MeLine.attributes`, `PersonLine.description` |

Attributes are read through `mineworld_sdk.attributes` (S30-f §§5.4, 5.8): selected by facet and cut by bytes,
never by attribute id; the phrases are the server's.

Items are never perceived (the item pack discloses nothing), so an item is a reference to the id a payload
names, and its name is `None` until a pack discloses item names (ruling 2 on the gap table).
"""

from __future__ import annotations

from mineworld_sdk import attributes, ways
from mineworld_sdk.wire.contract import PerceivedEntity
from mineworld_sdk.wire.ids import EntityId, JsonValue
from pydantic import ValidationError

from mineworld_agent.models import (
    NEED_LEVEL_MAX,
    AcquaintanceLine,
    AgendaLine,
    AsleepLine,
    AttributeLine,
    AttributeValueLine,
    DoingLine,
    InvitationLine,
    ItemCount,
    JobLine,
    NeedLine,
    NeedsLine,
    PlansLine,
    WeatherLine,
    WorldText,
)
from mineworld_agent.refs import RefTable

type Json = dict[str, JsonValue]


def payload(entity: PerceivedEntity | None, component: str) -> JsonValue:
    """The payload of `entity`'s component of that type, or `None`."""
    if entity is None:
        return None
    for record in entity.components:
        if record.component_type == component:
            return record.payload
    return None


def entity_id(value: JsonValue) -> EntityId | None:
    """The `entity` of a typed reference (`{ entity, entity_type }`), as packs write ids."""
    if isinstance(value, dict) and isinstance(entity := value.get("entity"), str):
        return EntityId(entity)
    return None


def _obj(value: JsonValue) -> Json | None:
    return value if isinstance(value, dict) else None


def _int(value: JsonValue) -> int | None:
    return value if isinstance(value, int) and not isinstance(value, bool) else None


def _list(value: JsonValue) -> list[JsonValue]:
    return value if isinstance(value, list) else []


def _seconds(hh_mm: JsonValue) -> int | None:
    """schedule's `TimeOfDay`, written `"HH:MM"`, as seconds since midnight."""
    if not isinstance(hh_mm, str) or len(hh_mm) != 5 or hh_mm[2] != ":":
        return None
    hours, minutes = hh_mm[:2], hh_mm[3:]
    if not (hours.isdigit() and minutes.isdigit()):
        return None
    return int(hours) * 3600 + int(minutes) * 60


def wallet(me: PerceivedEntity | None) -> int | None:
    found = _obj(payload(me, "wallet"))
    return None if found is None else _int(found.get("balance"))


def _counts(refs: RefTable, entries: list[JsonValue], count_key: str) -> list[ItemCount]:
    out: list[ItemCount] = []
    for entry in entries:
        line = _obj(entry)
        if line is None:
            continue
        item, count = entity_id(line.get("item")), _int(line.get(count_key))
        if item is not None and count is not None:
            out.append(ItemCount(item=refs.ref(item), count=count))
    return out


def holdings(refs: RefTable, me: PerceivedEntity | None) -> list[ItemCount]:
    found = _obj(payload(me, "holdings"))
    return [] if found is None else _counts(refs, _list(found.get("held")), "count")


def job(refs: RefTable, me: PerceivedEntity | None) -> JobLine | None:
    found = _obj(payload(me, "employment"))
    held = None if found is None else _obj(found.get("job"))
    if held is None:
        return None
    employer, workplace = entity_id(held.get("employer")), entity_id(held.get("workplace"))
    start, end, wage = (
        _seconds(held.get("from")),
        _seconds(held.get("until")),
        _int(held.get("wage")),
    )
    if employer is None or workplace is None or start is None or end is None or wage is None:
        return None
    shift = _obj(found.get("shift")) if found is not None else None
    return JobLine(
        employer=refs.ref(employer),
        workplace=refs.ref(workplace),
        from_s=start,
        until_s=end,
        wage=wage,
        produces=_counts(refs, _list(held.get("produces")), "per_shift"),
        on_shift=shift is not None,
        present=shift is not None and shift.get("present_since") is not None,
        worked_s=(_int(shift.get("worked")) or 0) if shift is not None else 0,
    )


def agenda(refs: RefTable, me: PerceivedEntity | None) -> AgendaLine | None:
    found = _obj(payload(me, "agenda"))
    if found is None:
        return None
    label, place = found.get("label"), entity_id(found.get("place"))
    since, until = _int(found.get("since")), _int(found.get("until"))
    if not isinstance(label, str) or place is None or since is None or until is None:
        return None
    return AgendaLine(
        label=WorldText(text=label, kind="label"), place=refs.ref(place), since=since, until=until
    )


def plans(refs: RefTable, me: PerceivedEntity | None, stride_s: int) -> PlansLine | None:
    """The next part of my day (`agenda.next`) and, from `ways`, about how long walking there takes when
    one stride is `stride_s` world seconds."""
    found = _obj(payload(me, "agenda"))
    following = None if found is None else _obj(found.get("next"))
    if following is None:
        return None
    label, place, start = (
        following.get("label"),
        entity_id(following.get("place")),
        following.get("from"),
    )
    begins = _int(start)
    if not isinstance(label, str) or place is None or begins is None:
        return None
    way = next((way for way in ways.read(payload(me, ways.WAYS)) if way.place == place), None)
    return PlansLine(
        label=WorldText(text=label, kind="label"),
        place=refs.ref(place),
        from_at=begins,
        punctual=following.get("punctual") is True,
        walk_minutes=None if way is None else ways.minutes(ways.estimate(way, stride_s)),
    )


def acquaintances(refs: RefTable, me: PerceivedEntity | None) -> list[AcquaintanceLine]:
    found = _obj(payload(me, "acquaintances"))
    out: list[AcquaintanceLine] = []
    for entry in _list(None if found is None else found.get("known")):
        line = _obj(entry)
        values = None if line is None else _obj(line.get("values"))
        person = None if line is None else entity_id(line.get("counterpart"))
        if values is None or person is None:
            continue
        numbers = [
            _int(values.get(key))
            for key in (
                "familiarity",
                "regard",
                "exchanges",
                "activities_shared",
                "first_met",
                "last_contact",
            )
        ]
        if any(number is None for number in numbers):
            continue
        familiarity, regard, exchanges, shared, first, last = (n or 0 for n in numbers)
        out.append(
            AcquaintanceLine(
                person=refs.ref(person),
                familiarity=familiarity,
                regard=regard,
                exchanges=exchanges,
                activities_shared=shared,
                first_met=first,
                last_contact=last,
            )
        )
    return out


def invitations(refs: RefTable, me: PerceivedEntity | None) -> list[InvitationLine]:
    found = _obj(payload(me, "invitations"))
    out: list[InvitationLine] = []
    for entry in _list(None if found is None else found.get("pending")):
        line = _obj(entry)
        if line is None:
            continue
        by, kind = entity_id(line.get("from")), line.get("kind")
        at, until = _int(line.get("at")), _int(line.get("until"))
        if by is None or not isinstance(kind, str) or at is None or until is None:
            continue
        who = refs.ref(by)
        out.append(
            InvitationLine(
                by=who, kind=WorldText(text=kind, by=who, kind="label"), at=at, until=until
            )
        )
    return out


def doing(entity: PerceivedEntity | None) -> DoingLine | None:
    found = _obj(payload(entity, "participation"))
    if found is None:
        return None
    kind, since = found.get("kind"), _int(found.get("since"))
    if not isinstance(kind, str) or since is None:
        return None
    return DoingLine(kind=WorldText(text=kind, kind="label"), since=since)


def needs(me: PerceivedEntity | None) -> NeedsLine | None:
    """My needs, as the `needs` record names them: each key of its `words` map, with the pack's word and the
    level stored under that key. A need whose word or level does not read is skipped; nothing is compared."""
    found = _obj(payload(me, "needs"))
    words = None if found is None else _obj(found.get("words"))
    if found is None or words is None:
        return None
    lines: list[NeedLine] = []
    for need, word in words.items():
        level = _int(found.get(need))
        if not isinstance(word, str) or level is None or not 0 <= level <= NEED_LEVEL_MAX:
            continue
        try:
            lines.append(NeedLine(need=need, word=word, level=level))
        except ValidationError:
            continue
    return NeedsLine(needs=lines)


def asleep(entity: PerceivedEntity | None) -> AsleepLine | None:
    """Whether `entity` sleeps, since when, and its alarm where the pack discloses one (to the sleeper only)."""
    found = _obj(payload(entity, "asleep"))
    since = None if found is None else _int(found.get("since"))
    if found is None or found.get("asleep") is not True or since is None:
        return None
    return AsleepLine(since=since, alarm=_int(found.get("alarm")))


DESCRIPTION_FACET = "observed"
DESCRIPTION_BYTES = 160
"""Who I see of a person: the facet and the byte budget every surface shares (S30-f §5.4)."""


def description(entity: PerceivedEntity | None) -> WorldText | None:
    """How `entity` looks to me, composed from its own disclosed attributes; `None` when there is nothing."""
    if entity is None:
        return None
    text = attributes.compose(attributes.blocks(entity), DESCRIPTION_FACET, DESCRIPTION_BYTES)
    return WorldText(text=text, kind="label") if text else None


def _label(text: str) -> WorldText:
    return WorldText(text=text, kind="label")


def _value(value: attributes.AttributeValue) -> AttributeValueLine:
    match value:
        case attributes.CategoryValue(category=slug):
            return AttributeValueLine(slug=_label(slug))
        case attributes.LevelValue(level=slug, rank=rank):
            return AttributeValueLine(slug=_label(slug), rank=rank)
        case attributes.QuantityValue(quantity=quantity):
            return AttributeValueLine(quantity=quantity)
        case attributes.FlagValue(flag=flag):
            return AttributeValueLine(flag=flag)
        case attributes.SetValue(set=members):
            return AttributeValueLine(members=[_label(member) for member in members])
        case attributes.TextValue(text=text):
            return AttributeValueLine(text=_label(text))
        case attributes.InstantValue(instant=seconds):
            return AttributeValueLine(instant=seconds)


def my_attributes(me: PerceivedEntity | None) -> list[AttributeLine]:
    """My own sheet, in rank order: every entry that shows a phrase or a value (`attributes.sheet`'s rule)."""
    if me is None:
        return []
    return [
        AttributeLine(
            attribute=_label(entry.attribute),
            label=_label(entry.label),
            kind=entry.kind,
            value=None if entry.value is None else _value(entry.value),
            text=_label(entry.text) if entry.text else None,
        )
        for entry in attributes.blocks(me)
        if attributes.shown(entry) is not None
    ]


def shop_prices(place: PerceivedEntity | None) -> dict[EntityId, int]:
    """Item → price, from my place's `shop` listing."""
    found = _obj(payload(place, "shop"))
    prices: dict[EntityId, int] = {}
    for entry in _list(None if found is None else found.get("listed")):
        line = _obj(entry)
        if line is None:
            continue
        item, price = entity_id(line.get("item")), _int(line.get("price"))
        if item is not None and price is not None:
            prices[item] = price
    return prices


def named_entities(value: JsonValue) -> list[EntityId]:
    """Every typed reference (`{ entity, entity_type }`) inside a payload, in order, once each."""
    found: list[EntityId] = []

    def walk(node: JsonValue) -> None:
        if isinstance(node, dict):
            if (entity := entity_id(node)) is not None and "entity_type" in node:
                if entity not in found:
                    found.append(entity)
                return
            for item in node.values():
                walk(item)
        elif isinstance(node, list):
            for item in node:
                walk(item)

    walk(value)
    return found


def weather(place: PerceivedEntity | None, time_of_day_s: int | None) -> WeatherLine | None:
    now = _obj(payload(place, "weather-now"))
    if now is None:
        return None
    condition, since = now.get("condition"), _int(now.get("hour"))
    if not isinstance(condition, str) or since is None:
        return None
    today = _obj(payload(place, "weather-today"))
    hours = _list(None if today is None else today.get("hours"))
    hour = None if time_of_day_s is None else time_of_day_s // 3600
    this = _obj(hours[hour]) if hour is not None and 0 <= hour < len(hours) else None
    return WeatherLine(
        condition=condition,
        since_hour=since,
        temperature_dc=None if this is None else _int(this.get("temperature_dc")),
        cloud_oktas=None if this is None else _int(this.get("cloud_oktas")),
        precipitation_tenth_mm=None if this is None else _int(this.get("precipitation_tenth_mm")),
        wind_dms=None if this is None else _int(this.get("wind_dms")),
        wind_from_deg=None if this is None else _int(this.get("wind_from_deg")),
    )
