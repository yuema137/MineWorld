"""How long a disclosed way probably takes, at the caller's own stride cadence (`ARC-281`, `ARC-282`).

The movement pack discloses to a person, about themself, a `ways` record: for each place they can reach,
the doorways the way crosses and its length in millimetres — a lower bound, never a duration
(`pr-le-leave-early.md` §0.3). A duration is the length divided by how often the walker strides, and that
cadence belongs to whoever drives the walk. A client that paces its walk at one `walk-step` per wall
second strides once per `time_scale` world seconds, so it converts with that:

    strides(way)  = ceil(length / WALK_STRIDE_MM) + doors + 1     one per doorway, one for the first step
    estimate(way) = strides(way) * stride_s                       stride_s = the driver's cadence, world s

The same arithmetic exists in Rust (`mineworld_rule_controller::commute::estimate`); both are checked
against one literal table (`cognition/rule-controller/tests/estimate-table.json`). This module only
reads facts and multiplies: it holds no departure rule and decides nothing (I-LE2-1).
"""

from __future__ import annotations

from dataclasses import dataclass

from mineworld_sdk.wire.contract import Observation, PerceivedEntity
from mineworld_sdk.wire.ids import EntityId, JsonValue

__all__ = ["WALK_STRIDE_MM", "WAYS", "Way", "estimate", "minutes", "of", "read", "strides"]

WAYS = "ways"
"""The component type movement discloses the ways under."""

WALK_STRIDE_MM = 1_340
"""movement's `WALK_STRIDE`: the most one `walk-step` moves inside a place, in millimetres."""


@dataclass(frozen=True, slots=True)
class Way:
    """The way to one place, as movement discloses it."""

    place: EntityId
    doors: int
    length_mm: int


def strides(way: Way) -> int:
    """The strides a way takes: one per `WALK_STRIDE_MM` of its length, rounded up, one per doorway, and
    one for the first step after `walk-to` is resolved."""
    length = max(way.length_mm, 0)
    return -(-length // WALK_STRIDE_MM) + way.doors + 1


def estimate(way: Way, stride_s: int) -> int:
    """How long a way probably takes, in world seconds, when the walker strides every `stride_s`."""
    return strides(way) * stride_s


def minutes(seconds: int) -> int:
    """World seconds as whole minutes, rounded up: "about N minutes"."""
    return -(-max(seconds, 0) // 60)


def read(payload: JsonValue) -> tuple[Way, ...]:
    """The ways of a disclosed `ways` payload, in the order disclosed. An entry that is not one is skipped,
    so a newer pack's shape is read as far as it is understood."""
    found: list[Way] = []
    for entry in payload if isinstance(payload, list) else []:
        if not isinstance(entry, dict):
            continue
        place, doors, length = (
            entry.get("place"),
            _int(entry.get("doors")),
            _int(entry.get("length")),
        )
        entity = place.get("entity") if isinstance(place, dict) else None
        if not isinstance(entity, str) or doors is None or length is None:
            continue
        found.append(Way(place=EntityId(entity), doors=doors, length_mm=length))
    return tuple(found)


def of(observation: Observation) -> tuple[Way, ...]:
    """The ways the observation discloses to its observer about themself; empty when none are."""
    me: PerceivedEntity | None = next(
        (entity for entity in observation.entities if entity.id == observation.observer), None
    )
    if me is None:
        return ()
    for record in me.components:
        if record.component_type == WAYS:
            return read(record.payload)
    return ()


def _int(value: JsonValue) -> int | None:
    return value if isinstance(value, int) and not isinstance(value, bool) else None
