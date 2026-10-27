"""Two price clerks' observations, with the packs' real JSON encodings (S31-d AC-d-12, AC-d-13; F-S31d-11).

`dollar()`: a clerk in a store, offered `publish-notice`, whose `survey-book` holds bread, apple and juice notes;
the place discloses an `item-catalogue` naming the kinds, the clerk the world's `currency` (US dollar, exponent 2,
symbol before). `crown()`: a second world: a crown of exponent 0 written after, a book of two kinds and no
catalogue, so the kinds are shown as `kind 1`, `kind 2`. An opaque id is a decimal string; an `ItemId` or a
`PlaceId` is `{"entity": "<id>", "entity_type": "<type>"}` (`contracts/src/ids.rs`).
"""

from __future__ import annotations

from mineworld_sdk.wire.contract import Observation
from mineworld_sdk.wire.ids import EntityId, JsonValue

from mineworld_cognition.config import FillConfig

CLERK = EntityId("7000000011")
GRACE = EntityId("7000000012")
STORE = EntityId("5000000011")
CAFE = EntityId("5000000012")
CORNER_STORE = EntityId("6000000001")
CAFE_COMPANY = EntityId("6000000002")
BREAD = EntityId("41")
APPLE = EntityId("42")
JUICE = EntityId("43")
FLOUR = EntityId("51")
SALT = EntityId("52")
DAY = 86_400


def settings(topic: str = "prices.ingredients", valid_minutes: int = 360) -> FillConfig:
    return FillConfig.model_validate(
        {"publish-notice": {"topic": topic, "valid_minutes": valid_minutes}}
    )


def _item(entity: EntityId) -> JsonValue:
    return {"entity": str(entity), "entity_type": "item"}


def _place(entity: EntityId) -> JsonValue:
    return {"entity": str(entity), "entity_type": "place"}


def _note(
    at: int, place: EntityId, seller: EntityId, price: int, source: str, stock: int | None
) -> JsonValue:
    return {
        "at": at,
        "place": _place(place),
        "seller": str(seller),
        "price": price,
        "source": source,
        "in_stock": stock,
    }


def _record(entity: EntityId, component: str, payload: JsonValue) -> JsonValue:
    return {
        "entity": str(entity),
        "component_type": component,
        "schema_version": 1,
        "payload": payload,
    }


def _publish(available: bool = True) -> JsonValue:
    return {
        "action_type": "publish-notice",
        "target": None,
        "available": available,
        "unavailable_reason": None if available else "busy",
        "requirement": {
            "place": "any",
            "within_range": None,
            "requires_line_of_access": False,
            "requires_target_available": False,
        },
    }


def _observation(
    at: int,
    book: JsonValue,
    currency: JsonValue,
    places: list[JsonValue],
    affordances: list[JsonValue],
) -> Observation:
    mine: list[JsonValue] = [_record(CLERK, "survey-book", book)]
    if currency is not None:
        mine.append(_record(CLERK, "currency", currency))
    clerk: JsonValue = {
        "id": str(CLERK),
        "entity_type": "person",
        "location": None,
        "tags": [],
        "components": [*mine, _record(CLERK, "display-name", {"name": "Carol Mensah"})],
    }
    grace: JsonValue = {
        "id": str(GRACE),
        "entity_type": "person",
        "location": None,
        "tags": [],
        "components": [_record(GRACE, "display-name", {"name": "Grace Okafor"})],
    }
    return Observation.model_validate(
        {
            "observer": str(CLERK),
            "at": at,
            "self_location": None,
            "entities": [clerk, grace, *places],
            "relations": [],
            "events": [],
            "affordances": affordances,
        }
    )


def dollar_book() -> JsonValue:
    day = DAY
    return {
        "categories": ["food", "drink"],
        "depth": 16,
        "window": 7 * DAY,
        "resurvey_after": 3600,
        "kinds": [
            {
                "item": _item(BREAD),
                "notes": [
                    _note(day + 100, STORE, CORNER_STORE, 290, "ask", 2),
                    _note(day + 200, STORE, CORNER_STORE, 300, "sale", None),
                    _note(day + 300, STORE, CORNER_STORE, 320, "sale", None),
                    _note(day + 400, STORE, CORNER_STORE, 300, "ask", 1),
                    _note(day + 500, STORE, GRACE, 260, "ask", None),
                    _note(day + 600, STORE, CORNER_STORE, 300, "sale", None),
                ],
            },
            {
                "item": _item(APPLE),
                "notes": [_note(day + 700, STORE, CORNER_STORE, 100, "sale", None)],
            },
            {
                "item": _item(JUICE),
                "notes": [
                    _note(day + 800, STORE, CORNER_STORE, 250, "ask", 4),
                    _note(day + 900, STORE, CORNER_STORE, 250, "sale", None),
                    _note(day + 1000, STORE, CORNER_STORE, 250, "sale", None),
                    _note(day + 1100, CAFE, CAFE_COMPANY, 350, "ask", 9),
                    _note(day + 1200, CAFE, CAFE_COMPANY, 350, "sale", None),
                    _note(day + 1300, CAFE, CAFE_COMPANY, 350, "sale", None),
                    _note(day + 1400, CAFE, CAFE_COMPANY, 340, "sale", None),
                ],
            },
        ],
        "surveyed": [{"place": _place(STORE), "at": day + 400}],
    }


def dollar(*, offered: bool = True, available: bool = True) -> Observation:
    """The US-dollar clerk at 2 days 00:00:05, in the store, its catalogue naming the kinds."""
    catalogue: JsonValue = {
        "kinds": [
            {"item": _item(BREAD), "category": "food", "name": "Bread"},
            {"item": _item(APPLE), "category": "food", "name": "Apple"},
            {"item": _item(JUICE), "category": "drink", "name": "Juice"},
        ]
    }
    store: JsonValue = {
        "id": str(STORE),
        "entity_type": "place",
        "location": None,
        "tags": ["store"],
        "components": [_record(STORE, "item-catalogue", catalogue)],
    }
    cafe: JsonValue = {
        "id": str(CAFE),
        "entity_type": "place",
        "location": None,
        "tags": ["cafe"],
        "components": [],
    }
    currency: JsonValue = {
        "name": "US dollar",
        "symbol": "$",
        "exponent": 2,
        "symbol_at": "before",
        "references": [{"item": _item(BREAD), "value": 300}],
    }
    return _observation(
        2 * DAY + 5,
        dollar_book(),
        currency,
        [store, cafe],
        [_publish(available)] if offered else [],
    )


def crown() -> Observation:
    """The crown clerk at 3 days 00:00:02: exponent 0, symbol after, two kinds and no catalogue."""
    book: JsonValue = {
        "categories": ["goods"],
        "depth": 8,
        "window": 7 * DAY,
        "resurvey_after": 3600,
        "kinds": [
            {
                "item": _item(FLOUR),
                "notes": [
                    _note(2 * DAY + 10, STORE, GRACE, 3, "sale", None),
                    _note(2 * DAY + 20, STORE, GRACE, 3, "sale", None),
                    _note(2 * DAY + 30, STORE, GRACE, 4, "sale", None),
                ],
            },
            {
                "item": _item(SALT),
                "notes": [
                    _note(2 * DAY + 40, STORE, CORNER_STORE, 5, "ask", 2),
                    _note(2 * DAY + 50, STORE, CORNER_STORE, 6, "sale", None),
                ],
            },
        ],
        "surveyed": [],
    }
    currency: JsonValue = {
        "name": "crown",
        "symbol": "cr",
        "exponent": 0,
        "symbol_at": "after",
        "references": [],
    }
    market: JsonValue = {
        "id": str(STORE),
        "entity_type": "place",
        "location": None,
        "tags": ["market"],
        "components": [],
    }
    return _observation(3 * DAY + 2, book, currency, [market], [_publish()])
