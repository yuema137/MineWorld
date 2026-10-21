"""Economy's sale and ask facts (`ARC-112`; pr-s31g §5.9): `sold`, `listed`, `price-set` and `withdrawn`.

Payloads as the merged pack states them (`systems/economy/src/event.rs`): the seller and the buyer are bare
entity ids, the item a typed `ItemId`, the place a typed `PlaceId`. A sale or an ask change is remembered so a
person knows what things cost and who sold what (step-31 §7.3).

Amounts are minor units with no currency: the currency is economy's `currency` record, which memory does not
read (SD-S31b-11). An item this seat knows no name for is "an item": a rendered line never shows an entity id
(`names.py`, P4-5; the notices renderer's D-S31b-4). Anchors come from `render`'s generic payload scan.
"""

from __future__ import annotations

from typing import Literal

from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EntityIdField
from pydantic import Field

from mineworld_cognition.memory.names import NameBook, one_line
from mineworld_cognition.memory.render import Payload, Rendered, counterparts, decode, fallback

SALE_WITH_ME = 400
SALE_WITHOUT_ME = 300
"""Base importance of a sale or an ask change: 400 when I bought or sold (or it is my ask), 300 when I only saw
it — between the generic 300 and 500 (pr-s31g §5.9; structural constants beside pr-fg-b §5.4's table, never
fitted, ARC-23)."""

AN_ITEM = "an item"
"""What an item is called when this seat knows no name for it (never its id)."""


class ItemRef(Payload):
    entity: EntityIdField
    entity_type: Literal["item"]


class Sold(Payload):
    seller: EntityIdField
    buyer: EntityIdField
    item: ItemRef
    count: int
    price: int


class Listed(Payload):
    seller: EntityIdField
    item: ItemRef
    price: int


class PriceSet(Payload):
    seller: EntityIdField
    item: ItemRef
    from_: int = Field(alias="from")
    to: int


class Withdrawn(Payload):
    seller: EntityIdField
    item: ItemRef


def _item(item: ItemRef, names: NameBook) -> str:
    return one_line(names.name(item.entity) or AN_ITEM)


def _who(entity: EntityId, me: EntityId, names: NameBook) -> str:
    return "I" if entity == me else names.person(entity)


def _rendered(fact: PerceivedEvent, me: EntityId, gist: str, mine: bool) -> Rendered:
    return Rendered(
        gist, counterparts(fact, me), "notable", mine, SALE_WITH_ME if mine else SALE_WITHOUT_ME
    )


def sold(fact: PerceivedEvent, me: EntityId, names: NameBook) -> Rendered:
    """`I bought <item> from <seller> for <price>`, `I sold <item> to <buyer> for <price>`, or
    `<buyer> bought <item> from <seller> for <price>`."""
    payload = decode(Sold, fact)
    if payload is None:
        return fallback(fact, me, names)
    item = _item(payload.item, names)
    if payload.buyer == me:
        gist = f"I bought {item} from {names.person(payload.seller)} for {payload.price}"
    elif payload.seller == me:
        gist = f"I sold {item} to {names.person(payload.buyer)} for {payload.price}"
    else:
        buyer, seller = names.person(payload.buyer), names.person(payload.seller)
        gist = f"{buyer} bought {item} from {seller} for {payload.price}"
    return _rendered(fact, me, gist, me in (payload.buyer, payload.seller))


def listed(fact: PerceivedEvent, me: EntityId, names: NameBook) -> Rendered:
    """`<seller> put <item> on sale at <price>`."""
    payload = decode(Listed, fact)
    if payload is None:
        return fallback(fact, me, names)
    who = _who(payload.seller, me, names)
    gist = f"{who} put {_item(payload.item, names)} on sale at {payload.price}"
    return _rendered(fact, me, gist, payload.seller == me)


def price_set(fact: PerceivedEvent, me: EntityId, names: NameBook) -> Rendered:
    """`<seller> changed <item> from <from> to <to>`."""
    payload = decode(PriceSet, fact)
    if payload is None:
        return fallback(fact, me, names)
    who = _who(payload.seller, me, names)
    gist = f"{who} changed {_item(payload.item, names)} from {payload.from_} to {payload.to}"
    return _rendered(fact, me, gist, payload.seller == me)


def withdrawn(fact: PerceivedEvent, me: EntityId, names: NameBook) -> Rendered:
    """`<seller> took <item> off sale`."""
    payload = decode(Withdrawn, fact)
    if payload is None:
        return fallback(fact, me, names)
    who = _who(payload.seller, me, names)
    gist = f"{who} took {_item(payload.item, names)} off sale"
    return _rendered(fact, me, gist, payload.seller == me)
