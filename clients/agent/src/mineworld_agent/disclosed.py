"""What the packs disclose about me and the people I perceive, read into typed lines (R-S27c-INFO).

The operator's principle for the agent interface — "给agent的接口是信息丰富的", information-rich — means that
whatever a human can see or infer on screen reaches the agent in structured form, within the same
perception (ledger §15, the gap table). Each function here reads one pack's own component, as that pack
writes it, and returns `None` (or nothing) for anything it does not recognise: a pack that is not installed,
a component not disclosed to this observer, or a shape a newer pack writes. Nothing here evaluates a rule.

| Component        | Pack           | Disclosed to         | Line                         |
| ---------------- | -------------- | -------------------- | ---------------------------- |
| `wallet`         | economy        | the holder           | `MeLine.wallet`              |
| `wealth`         | economy        | the holder           | `MeLine.wealth`              |
| `currency`       | economy        | every living person, about itself | `View.currency`; every `Amount.display`; `reference` |
| `asks`           | economy        | anyone perceiving the seller | `MeLine.asks`, `PersonLine.asks` |
| `subscriptions`  | notices        | the holder           | `MeLine.subscriptions`       |
| `notice-board`   | notices        | the holder           | `View.notices`               |
| `holdings`       | inventory      | the holder           | `MeLine.holdings`            |
| `employment`     | employment     | the employee         | `MeLine.job`                 |
| `agenda`         | schedule       | the person           | `MeLine.agenda`, `MeLine.plans` |
| `ways`           | movement       | the person           | `MeLine.plans.walk_minutes`  |
| `acquaintances`  | relationships  | the holder           | `MeLine.acquaintances`       |
| `invitations`    | group-activity | the invitee          | `MeLine.invitations`         |
| `participation`  | group-activity | anyone perceiving    | `MeLine.doing`, `PersonLine.doing` |
| `item-catalogue` | item           | anyone in the place  | `ItemCount.name`, `AskLine.name`, `OfferLine.about_names`, `inspect(item)` |
| `loose-objects`  | bodies         | anyone in the place  | `OfferLine.about_names`, `inspect(item)` (the object's tags) |
| `weather-now`, `weather-today` | weather | anyone in the place | `View.weather`        |
| `needs`          | needs          | the holder           | `MeLine.needs`               |
| `asleep`         | sleep          | the sleeper sees `alarm`; anyone perceiving sees `since` | `MeLine.asleep`, `PersonLine.asleep` |
| `traits`         | attributes     | the holder: every non-hidden entry; anyone else: perceivable ones | `MeLine.attributes`, `PersonLine.description` |
| an owner's `attributes` member | the owning pack | as `traits` | `MeLine.attributes`, `PersonLine.description` |

Attributes are read through `mineworld_sdk.attributes` (S30-f §§5.4, 5.8): selected by facet and cut by bytes,
never by attribute id; the phrases are the server's.

Items are never perceived as entities, so an item is a reference to the id a payload names. Its name is the
one the `item` pack discloses on my place, in the `item-catalogue` record (`[{ item, category, name }]` for every
declared kind, `ARC-37` note): [`item_kinds`] reads it, and every line that names an item takes its name from
there. A loose object (bodies' `loose-objects` listing on my place) is not a kind and is in no catalogue; it is
called by the tags its listing entry carries ([`loose_tags`]), as a place is. An item neither lists has no name.
The handle's `Lexicon` keeps every catalogue name and the newest `currency` it was disclosed (`lexicon.py`), so
a name learned in one place still names the kind elsewhere, and every amount is formatted from the world.

A price is read from one place only: the `buy` offer's own payload ([`buy_price`], pr-s31g SD-S31g-3), the
price the world settles at. The `shop` listing is not read for prices.
"""

from __future__ import annotations

import re
from dataclasses import dataclass

from mineworld_sdk import attributes, ways
from mineworld_sdk.wire.contract import PerceivedEntity
from mineworld_sdk.wire.ids import EntityId, JsonValue
from pydantic import ValidationError

from mineworld_agent.lexicon import Currency, Lexicon
from mineworld_agent.models import (
    NEED_LEVEL_MAX,
    TOPIC_PATTERN,
    AcquaintanceLine,
    AgendaLine,
    Amount,
    AskLine,
    AsleepLine,
    AttributeLine,
    AttributeValueLine,
    CurrencyLine,
    DoingLine,
    InvitationLine,
    ItemCount,
    JobLine,
    NeedLine,
    NeedsLine,
    NoticeLine,
    NoticePriceLine,
    PlansLine,
    WealthLine,
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


def any_entity(value: JsonValue) -> EntityId | None:
    """An id as packs write it: a typed reference (`{ entity, entity_type }`) or a bare id string (`EntityId`)."""
    if isinstance(value, str) and value:
        return EntityId(value)
    return entity_id(value)


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


def _minor(value: JsonValue) -> int | None:
    """An amount in minor units: a non-negative integer."""
    number = _int(value)
    return number if number is not None and number >= 0 else None


def wallet(me: PerceivedEntity | None, lexicon: Lexicon) -> Amount | None:
    found = _obj(payload(me, "wallet"))
    balance = None if found is None else _minor(found.get("balance"))
    return None if balance is None else lexicon.amount(balance)


def currency(me: PerceivedEntity | None) -> Currency | None:
    """economy's `currency` record on me: `{ name, symbol, exponent, symbol_at, references: [{ item, value }] }`."""
    found = _obj(payload(me, "currency"))
    if found is None:
        return None
    name, symbol, exponent, side = (
        found.get("name"),
        found.get("symbol"),
        _int(found.get("exponent")),
        found.get("symbol_at"),
    )
    if not isinstance(name, str) or not isinstance(symbol, str) or exponent is None:
        return None
    if exponent < 0 or side not in ("before", "after"):
        return None
    references: dict[EntityId, int] = {}
    for entry in _list(found.get("references")):
        line = _obj(entry)
        item = None if line is None else any_entity(line.get("item"))
        value = None if line is None else _minor(line.get("value"))
        if item is not None and value is not None:
            references[item] = value
    line = CurrencyLine(
        name=WorldText(text=name, kind="name"),
        symbol=WorldText(text=symbol, kind="label"),
        exponent=exponent,
        symbol_at="before" if side == "before" else "after",
    )
    return Currency(line=line, references=references)


def wealth(me: PerceivedEntity | None, lexicon: Lexicon) -> WealthLine | None:
    """economy's `wealth` on me: `{ amount, unvalued }`."""
    found = _obj(payload(me, "wealth"))
    if found is None:
        return None
    amount, unvalued = _minor(found.get("amount")), _minor(found.get("unvalued"))
    if amount is None or unvalued is None:
        return None
    return WealthLine(amount=lexicon.amount(amount), unvalued=unvalued)


def asks(refs: RefTable, entity: PerceivedEntity | None, lexicon: Lexicon) -> list[AskLine]:
    """economy's `asks` view of a seller: `{ asks: [{ item, price, available }] }`, in the record's order."""
    found = _obj(payload(entity, "asks"))
    out: list[AskLine] = []
    for entry in _list(None if found is None else found.get("asks")):
        line = _obj(entry)
        if line is None:
            continue
        item, price, available = (
            any_entity(line.get("item")),
            _minor(line.get("price")),
            line.get("available"),
        )
        if item is None or price is None or not isinstance(available, bool):
            continue
        out.append(
            AskLine(
                item=refs.ref(item),
                name=lexicon.name(item),
                price=lexicon.amount(price),
                available=available,
                reference=lexicon.reference(item),
            )
        )
    return out


_TOPIC = re.compile(TOPIC_PATTERN)


def topic(value: JsonValue) -> str | None:
    """A notice topic's name, when it is one (`TopicName`'s pattern, at most 64 bytes)."""
    if isinstance(value, str) and len(value.encode("utf-8")) <= 64 and _TOPIC.match(value):
        return value
    return None


def subscriptions(me: PerceivedEntity | None) -> list[str]:
    """The notices pack's `subscriptions` on me: `{ topics: [TopicId] }`, in the record's order."""
    found = _obj(payload(me, "subscriptions"))
    topics = (topic(entry) for entry in _list(None if found is None else found.get("topics")))
    return [name for name in topics if name is not None]


def notice_line(refs: RefTable, value: JsonValue, lexicon: Lexicon) -> NoticeLine | None:
    """One `Notice` (`{ id, topic, publisher, body, issued_at, valid_until }`, body `{"prices": […]}` or
    `{"text": "…"}`), as the board and `notice-published` carry it; `None` when it does not read."""
    found = _obj(value)
    if found is None:
        return None
    name, publisher = topic(found.get("topic")), any_entity(found.get("publisher"))
    issued, until, body = (
        _int(found.get("issued_at")),
        _int(found.get("valid_until")),
        _obj(found.get("body")),
    )
    if name is None or publisher is None or issued is None or until is None or body is None:
        return None
    by = refs.ref(publisher)
    said = body.get("text")
    if isinstance(said, str):
        text = WorldText(text=said, by=by, kind="speech")
        return NoticeLine(topic=name, publisher=by, issued_at=issued, valid_until=until, text=text)
    lines: list[NoticePriceLine] = []
    for entry in _list(body.get("prices")):
        line = _obj(entry)
        if line is None:
            return None
        item, price, basis = (
            any_entity(line.get("item")),
            _minor(line.get("price")),
            _minor(line.get("basis")),
        )
        if item is None or price is None or basis is None:
            return None
        lines.append(
            NoticePriceLine(
                item=refs.ref(item),
                name=lexicon.name(item),
                price=lexicon.amount(price),
                basis=basis,
            )
        )
    if not lines:
        return None
    return NoticeLine(topic=name, publisher=by, issued_at=issued, valid_until=until, prices=lines)


def notices(refs: RefTable, me: PerceivedEntity | None, lexicon: Lexicon) -> list[NoticeLine]:
    """The notices pack's `notice-board` on me: `{ notices: [Notice] }`, whole and in the record's order. The
    record holds only current notices (the pack judges that); nothing here compares a time."""
    found = _obj(payload(me, "notice-board"))
    lines = (
        notice_line(refs, entry, lexicon)
        for entry in _list(None if found is None else found.get("notices"))
    )
    return [line for line in lines if line is not None]


BUY = "buy"


def buy_price(action_type: str, value: JsonValue) -> tuple[EntityId, int] | None:
    """The item and price of economy's `buy` payload (`{ item, price }`), read by name; `None` for anything else."""
    found = _obj(value)
    if action_type != BUY or found is None:
        return None
    item, price = any_entity(found.get("item")), _minor(found.get("price"))
    return None if item is None or price is None else (item, price)


@dataclass(frozen=True, slots=True)
class Kind:
    """One declared item kind as the `item-catalogue` discloses it: what people call it, and its sort."""

    name: str
    category: str


type Kinds = dict[EntityId, Kind]

ITEM_CATALOGUE = "item-catalogue"


def item_kinds(place: PerceivedEntity | None) -> Kinds:
    """Item → its kind, from my place's `item-catalogue` record; empty when the place discloses none."""
    found = _obj(payload(place, ITEM_CATALOGUE))
    kinds: Kinds = {}
    for entry in _list(None if found is None else found.get("kinds")):
        line = _obj(entry)
        if line is None:
            continue
        item, name, category = entity_id(line.get("item")), line.get("name"), line.get("category")
        if item is not None and isinstance(name, str) and name:
            kinds[item] = Kind(name=name, category=category if isinstance(category, str) else "")
    return kinds


def loose_tags(place: PerceivedEntity | None) -> dict[EntityId, tuple[str, ...]]:
    """Loose object → its tags, from my place's `loose-objects` listing; an object without tags is left out."""
    tagged: dict[EntityId, tuple[str, ...]] = {}
    for entry in _list(payload(place, "loose-objects")):
        line = _obj(entry)
        item = None if line is None else entity_id(line.get("object"))
        tags = tuple(
            t for t in _list(None if line is None else line.get("tags")) if isinstance(t, str)
        )
        if item is not None and tags:
            tagged[item] = tags
    return tagged


def item_names(kinds: Kinds) -> dict[EntityId, str]:
    """Item → its name, from a place's catalogue as `item_kinds` read it: what the lexicon learns."""
    return {item: kind.name for item, kind in kinds.items()}


def _counts(
    refs: RefTable,
    entries: list[JsonValue],
    count_key: str,
    lexicon: Lexicon,
    *,
    valued: bool = False,
) -> list[ItemCount]:
    out: list[ItemCount] = []
    for entry in entries:
        line = _obj(entry)
        if line is None:
            continue
        item, count = entity_id(line.get("item")), _int(line.get(count_key))
        if item is not None and count is not None:
            out.append(
                ItemCount(
                    item=refs.ref(item),
                    name=lexicon.name(item),
                    count=count,
                    reference=lexicon.reference(item) if valued else None,
                )
            )
    return out


def holdings(refs: RefTable, me: PerceivedEntity | None, lexicon: Lexicon) -> list[ItemCount]:
    found = _obj(payload(me, "holdings"))
    if found is None:
        return []
    return _counts(refs, _list(found.get("held")), "count", lexicon, valued=True)


def job(refs: RefTable, me: PerceivedEntity | None, lexicon: Lexicon) -> JobLine | None:
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
        produces=_counts(refs, _list(held.get("produces")), "per_shift", lexicon),
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
