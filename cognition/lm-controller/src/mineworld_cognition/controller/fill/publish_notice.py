"""The `publish-notice` filler: an LM price clerk posts suggested prices from its own price book (pr-s31d §5.8).

It lists itself when the world offers the seat `publish-notice` (available, no target), the seat's own
`survey-book` record holds at least one kind, and the operator configured `[fill.publish-notice] topic`. It shows
the book in labels only: kinds by the place's `item-catalogue` names (duplicates suffixed ` (2)`; an unnamed kind is
`kind 1`, `kind 2`, … in item order), sellers by the name book, never an entity id (step-17 I-10). The model answers
lines `{item: <label>, price, basis}`; the filler checks each against what it showed and builds the notices pack's
own payload `{topic, body: {prices: [{item, price, basis}]}, valid_for}`, lines in item order. The notices pack
judges it exactly as it judges the deterministic clerk's (`judge_publish`, AC-S31-11).

The record shapes are the packs' JSON encodings: an opaque id is a decimal string, a typed reference (`ItemId`,
`PlaceId`) is `{"entity": "<id>", "entity_type": "<type>"}` (`contracts/src/ids.rs`; F-S31d-11).
"""

from __future__ import annotations

from collections.abc import Mapping
from types import MappingProxyType

from mineworld_sdk.wire.contract import Observation
from mineworld_sdk.wire.ids import ActionTypeId, EntityId, JsonValue

from mineworld_cognition.backend.model import CognitionModel
from mineworld_cognition.config import FillConfig
from mineworld_cognition.controller.choice import ContentFailure
from mineworld_cognition.controller.fill.base import FillPayload, FillShown
from mineworld_cognition.memory.names import NameBook

PUBLISH_NOTICE = ActionTypeId("publish-notice")
SURVEY_BOOK = "survey-book"
ITEM_CATALOGUE = "item-catalogue"
CURRENCY = "currency"
LINES_MAX = 32
"""The notices pack's `PRICE_LINES_MAX`."""
PRICE_MAX = 1_000_000_000_000
"""The notices pack's `NOTICE_PRICE_MAX`."""
DAY_BEGAN = "day-began"
KEY = "notice"


class AskShown(CognitionModel):
    """One seller's newest ask of a kind, as shown: the seller's label, the price, a shop's stock count."""

    seller: str
    price: int
    stock: int | None


class KindShown(CognitionModel):
    """One kind of the book, as shown: its label and id, its sale prices oldest first, its asks."""

    label: str
    item: EntityId
    sales: tuple[int, ...]
    asks: tuple[AskShown, ...]

    @property
    def notes(self) -> int:
        """The notes a line on this kind may rest on: every sale and every seller's newest ask shown."""
        return len(self.sales) + len(self.asks)


class BookShown(FillShown):
    """What the filler showed: the topic and validity from the operator's table, the amounts' unit line, and
    every kind of the book in item order."""

    topic: str
    valid_for: int
    unit: str
    kinds: tuple[KindShown, ...]


class PublishNoticeFiller:
    """The price clerk's filler: suggested prices on the configured topic, from the seat's own book."""

    ACTION_TYPE = PUBLISH_NOTICE

    @property
    def action_type(self) -> ActionTypeId:
        return PUBLISH_NOTICE

    @property
    def key(self) -> str:
        return KEY

    @property
    def due_events(self) -> Mapping[str, str]:
        return MappingProxyType({DAY_BEGAN: "A new day began."})

    def listed(
        self, observation: Observation, names: NameBook, settings: FillConfig
    ) -> BookShown | None:
        table = settings.publish_notice
        if table is None:
            return None
        offered = any(
            affordance.action_type == PUBLISH_NOTICE
            and affordance.target is None
            and affordance.available
            for affordance in observation.affordances
        )
        if not offered:
            return None
        book = _own(observation, SURVEY_BOOK)
        kinds = _kinds(book, _catalogue(observation), names)
        if not kinds:
            return None
        return BookShown(
            topic=table.topic,
            valid_for=table.valid_minutes * 60,
            unit=_unit(_own(observation, CURRENCY)),
            kinds=kinds,
        )

    def section(self, number: str, shown: FillShown) -> tuple[str, ...]:
        book = _book(shown)
        to_fill = (
            f"{number}. publish-notice: post suggested prices on {book.topic}. Answer"
            f' "act": "fill" and "fill": {{"offer": "{number}", "{KEY}": {{"lines": [{{"item": <a kind'
            ' from MY PRICE BOOK>, "price": <a whole amount>, "basis": <how many of the notes shown'
            f" for that kind it rests on>}}]}}}}, at most {LINES_MAX} lines."
        )
        return (to_fill, "", f"MY PRICE BOOK ({book.unit})", *(_line(kind) for kind in book.kinds))

    def schema(self) -> dict[str, JsonValue]:
        return {
            "type": ["object", "null"],
            "additionalProperties": False,
            "required": ["lines"],
            "properties": {
                "lines": {
                    "type": "array",
                    "maxItems": LINES_MAX,
                    "items": {
                        "type": "object",
                        "additionalProperties": False,
                        "required": ["item", "price", "basis"],
                        "properties": {
                            "item": {"type": "string"},
                            "price": {"type": "integer", "minimum": 1},
                            "basis": {"type": "integer", "minimum": 1},
                        },
                    },
                }
            },
        }

    def check(self, content: JsonValue, shown: FillShown) -> FillPayload | ContentFailure:
        book = _book(shown)
        if content is None:
            return ContentFailure(reason="no_lines", detail="the notice has no lines")
        if not isinstance(content, dict) or not isinstance(content.get("lines"), list):
            return ContentFailure(reason="schema", detail=f"fill.{KEY}: lines is not a list")
        lines = content["lines"]
        assert isinstance(lines, list)
        if not 1 <= len(lines) <= LINES_MAX:
            return ContentFailure(reason="no_lines", detail=f"a notice has 1 to {LINES_MAX} lines")
        by_label = {kind.label: kind for kind in book.kinds}
        seen: set[str] = set()
        prices: list[tuple[int, EntityId, int, int]] = []
        for index, line in enumerate(lines):
            parsed = _parsed(line)
            if parsed is None:
                return ContentFailure(
                    reason="schema", detail=f"fill.{KEY}.lines.{index}: item, price and basis"
                )
            label, price, basis = parsed
            kind = by_label.get(label)
            if kind is None:
                return ContentFailure(
                    reason="unknown_item", detail="that kind is not in my price book"
                )
            if label in seen:
                return ContentFailure(reason="duplicate_item", detail=f"{label} is listed twice")
            seen.add(label)
            if not 1 <= price <= PRICE_MAX:
                return ContentFailure(
                    reason="price_out_of_range", detail=f"a price is from 1 to {PRICE_MAX}"
                )
            if not 1 <= basis <= kind.notes:
                return ContentFailure(
                    reason="basis_out_of_range",
                    detail=f"the basis for {label} is from 1 to {kind.notes}",
                )
            prices.append((int(kind.item), kind.item, price, basis))
        body: list[JsonValue] = [
            {"item": item, "price": price, "basis": basis}
            for _, item, price, basis in sorted(prices, key=lambda entry: entry[0])
        ]
        return FillPayload(
            action_type=PUBLISH_NOTICE,
            target=None,
            payload={"topic": book.topic, "body": {"prices": body}, "valid_for": book.valid_for},
        )


def _book(shown: FillShown) -> BookShown:
    if not isinstance(shown, BookShown):
        raise TypeError("the publish-notice filler checks only what it showed")
    return shown


def _parsed(line: JsonValue) -> tuple[str, int, int] | None:
    if not isinstance(line, dict):
        return None
    item, price, basis = line.get("item"), line.get("price"), line.get("basis")
    if not isinstance(item, str) or not _whole(price) or not _whole(basis):
        return None
    assert isinstance(price, int) and isinstance(basis, int)
    return item, price, basis


def _whole(value: JsonValue) -> bool:
    return isinstance(value, int) and not isinstance(value, bool)


def _line(kind: KindShown) -> str:
    sales = ", ".join(str(price) for price in kind.sales) or "none"
    asks = (
        ", ".join(
            f"{ask.seller} {ask.price}" + ("" if ask.stock is None else f" (stock {ask.stock})")
            for ask in kind.asks
        )
        or "none"
    )
    return f"{kind.label} ({kind.notes} notes): sales {sales}; asks {asks}"


def _own(observation: Observation, component: str) -> JsonValue:
    """A record the observer is disclosed about itself."""
    for entity in observation.entities:
        if entity.id != observation.observer:
            continue
        for record in entity.components:
            if record.component_type == component:
                return record.payload
    return None


def _catalogue(observation: Observation) -> dict[EntityId, str]:
    """Every kind's name from the first `item-catalogue` record perceived (a place's)."""
    for entity in observation.entities:
        for record in entity.components:
            if record.component_type != ITEM_CATALOGUE or not isinstance(record.payload, dict):
                continue
            kinds = record.payload.get("kinds")
            names: dict[EntityId, str] = {}
            for kind in kinds if isinstance(kinds, list) else []:
                item = _reference(kind.get("item")) if isinstance(kind, dict) else None
                name = kind.get("name") if isinstance(kind, dict) else None
                if item is not None and isinstance(name, str) and name:
                    names[item] = name
            return names
    return {}


def _reference(value: JsonValue) -> EntityId | None:
    """A typed reference's entity: `{"entity": "<id>", "entity_type": …}`."""
    if not isinstance(value, dict):
        return None
    entity = value.get("entity")
    return EntityId(entity) if isinstance(entity, str) and entity.isdigit() else None


def _kinds(
    book: JsonValue, catalogue: dict[EntityId, str], names: NameBook
) -> tuple[KindShown, ...]:
    """The book's kinds in item order, each with its label, sales and newest ask per (seller, place)."""
    kinds = book.get("kinds") if isinstance(book, dict) else None
    found: list[tuple[EntityId, tuple[int, ...], tuple[AskShown, ...]]] = []
    for kind in kinds if isinstance(kinds, list) else []:
        if not isinstance(kind, dict):
            continue
        item = _reference(kind.get("item"))
        notes = kind.get("notes")
        if item is None or not isinstance(notes, list):
            continue
        sales: list[int] = []
        asks: dict[tuple[str, str], AskShown] = {}
        for note in notes:
            parsed = _note(note, names)
            if parsed is None:
                continue
            source, key, ask = parsed
            if source == "sale":
                sales.append(ask.price)
            else:
                asks.pop(key, None)
                asks[key] = ask
        if sales or asks:
            found.append((item, tuple(sales), tuple(asks.values())))
    found.sort(key=lambda entry: int(entry[0]))
    labels = _labels([item for item, _, _ in found], catalogue)
    return tuple(
        KindShown(label=labels[item], item=item, sales=sales, asks=asks)
        for item, sales, asks in found
    )


def _note(note: JsonValue, names: NameBook) -> tuple[str, tuple[str, str], AskShown] | None:
    if not isinstance(note, dict):
        return None
    source, seller, price = note.get("source"), note.get("seller"), note.get("price")
    place = _reference(note.get("place"))
    stock = note.get("in_stock")
    if source not in ("ask", "sale") or not isinstance(seller, str) or not _whole(price):
        return None
    assert isinstance(price, int) and isinstance(source, str)
    label = names.name(EntityId(seller)) or f"{names.place(place)} shop"
    count = stock if isinstance(stock, int) and not isinstance(stock, bool) else None
    shown = AskShown(seller=label, price=price, stock=count)
    return source, (seller, "" if place is None else str(place)), shown


def _labels(items: list[EntityId], catalogue: dict[EntityId, str]) -> dict[EntityId, str]:
    """Each kind's label: its catalogue name, ` (2)` on a repeat; else `kind 1`, `kind 2`, … in item order."""
    labels: dict[EntityId, str] = {}
    seen: dict[str, int] = {}
    unnamed = 0
    for item in items:
        name = catalogue.get(item)
        if name is None:
            unnamed += 1
            labels[item] = f"kind {unnamed}"
            continue
        seen[name] = seen.get(name, 0) + 1
        labels[item] = name if seen[name] == 1 else f"{name} ({seen[name]})"
    return labels


def _unit(currency: JsonValue) -> str:
    """The amounts' unit line: minor units of the world's currency, with an example of how one reads."""
    if not isinstance(currency, dict):
        return "amounts in minor units"
    name, symbol = currency.get("name"), currency.get("symbol")
    exponent, symbol_at = currency.get("exponent"), currency.get("symbol_at")
    if not isinstance(name, str) or not isinstance(symbol, str) or not _whole(exponent):
        return "amounts in minor units"
    assert isinstance(exponent, int)
    amount = 3 * 10**exponent
    major = "3" if exponent == 0 else f"3.{'0' * exponent}"
    written = f"{symbol}{major}" if symbol_at == "before" else f"{major} {symbol}"
    return f"amounts in minor units of the {name}: {amount} = {written}"
