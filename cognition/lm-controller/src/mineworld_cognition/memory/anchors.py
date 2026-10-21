"""Payload anchors: the places, items and organizations a perceived fact's payload names (pr-s28d §5.2).

The contract types every entity reference a payload carries as `{"entity": "<id>", "entity_type": "<kind>"}`
(`TypedEntityRef`), so one generic scan finds them for every pack, including packs this code has never seen. No
event type, entity id or world is named here (I-d-8). The scan is a function of the fact alone, so it is the same
for every seat that holds the fact.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal, cast, get_args

from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId

AnchorKind = Literal["place", "item", "organization"]
ANCHOR_KINDS: tuple[AnchorKind, ...] = get_args(AnchorKind)
ANCHORS_PER_FACT = 16
"""The most anchors kept per fact: a payload naming more is cut, in scan order (F-S28d-15)."""
_REFERENCE_KEYS = frozenset({"entity", "entity_type"})


@dataclass(frozen=True)
class Anchor:
    """An entity a fact is anchored to, with the kind the world disclosed for it."""

    entity: EntityId
    kind: AnchorKind


def payload_anchors(fact: PerceivedEvent) -> tuple[Anchor, ...]:
    """Every typed reference in the fact's payload whose kind is a place, an item or an organization: depth
    first, object keys in sorted order, list elements in order; the first occurrence of each entity, at most
    `ANCHORS_PER_FACT`. A payload that is not a JSON object or list gives `()`; the scan never raises."""
    found: dict[str, Anchor] = {}
    pending: list[object] = [fact.payload.payload]
    while pending and len(found) < ANCHORS_PER_FACT:
        value = pending.pop()
        if isinstance(value, dict):
            members = cast(dict[str, object], value)
            if members.keys() == _REFERENCE_KEYS:
                entity, kind = members["entity"], members["entity_type"]
                if isinstance(entity, str) and _decimal(entity) and isinstance(kind, str):
                    if kind in ANCHOR_KINDS and entity not in found:
                        found[entity] = Anchor(EntityId(entity), kind)
                    continue
            # Reversed onto the stack, so the smallest key is visited first.
            pending.extend(members[key] for key in sorted(members, reverse=True))
        elif isinstance(value, list):
            pending.extend(reversed(cast(list[object], value)))
    return tuple(found.values())


def _decimal(value: str) -> bool:
    """ASCII decimal digits with no leading zero: an `EntityId`."""
    return value.isascii() and value.isdigit() and (value == "0" or value[0] != "0")
