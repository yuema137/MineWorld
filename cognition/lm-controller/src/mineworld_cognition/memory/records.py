"""Memory's typed records: ids, citations, the four levels, and a store's identity (D-P4-2, D-P4-4).

**Ids stay strings.** An `EventId` is a decimal string up to u64; memory never converts one to a number.
It orders facts by `EventKey`, the id left-padded with `0` to 20 characters (u64's width), which sorts
lexicographically exactly as the integers do. A **citation** is a sorted tuple of disjoint ranges over
this seat's perceived sequence: a range stands for the perceived ids between its ends, not for every
world id between them.
"""

from __future__ import annotations

from typing import Annotated, Literal, NewType

from mineworld_sdk.wire.ids import (
    EntityId,
    EntityIdField,
    EntityKeyField,
    EventId,
    EventIdField,
    EventTypeIdField,
    WorldInstanceIdField,
)
from pydantic import Field, model_validator

from mineworld_cognition.backend.model import CognitionModel

SCHEMA_VERSION = 1
KEY_WIDTH = 20
"""Decimal digits of u64's maximum, 18446744073709551615."""

EventKey = NewType("EventKey", str)
"""An `EventId` left-padded with `0` to `KEY_WIDTH` characters: the store's ordering key."""

Salience = Literal["notable", "ambient"]
"""`ambient`: a record ingested, covered and citable, which never opens or splits an episode and is not
shown line by line (another person's comings and goings, F-P4-2)."""

Level = Literal["L3", "L2", "L1", "L0"]

NonNegative = Annotated[int, Field(ge=0)]


def event_key(event: EventId) -> EventKey:
    return EventKey(event.rjust(KEY_WIDTH, "0"))


def event_of(key: str) -> EventId:
    """The `EventId` an `EventKey` was made from."""
    return EventId(key.lstrip("0") or "0")


class EventRange(CognitionModel):
    """The perceived facts from `first` to `last`, both included, in this seat's perceived order."""

    first: EventIdField
    last: EventIdField

    @model_validator(mode="after")
    def _ordered(self) -> EventRange:
        if event_key(self.first) > event_key(self.last):
            raise ValueError(f"a range runs forward: {self.first} is after {self.last}")
        return self

    def token(self) -> str:
        return f"#{self.first}" if self.first == self.last else f"#{self.first}-{self.last}"


class Citation(CognitionModel):
    """The facts a line stands for: sorted, disjoint ranges, at least one."""

    ranges: Annotated[tuple[EventRange, ...], Field(min_length=1)]

    @model_validator(mode="after")
    def _sorted_and_disjoint(self) -> Citation:
        for before, after in zip(self.ranges, self.ranges[1:], strict=False):
            if event_key(before.last) >= event_key(after.first):
                raise ValueError(
                    f"citation ranges must be sorted and disjoint: {before.token()} then "
                    f"{after.token()}"
                )
        return self

    @staticmethod
    def of(*events: EventId) -> Citation:
        """Single-fact ranges for the distinct `events`, sorted."""
        distinct = sorted(set(events), key=event_key)
        return Citation(ranges=tuple(EventRange(first=e, last=e) for e in distinct))

    @staticmethod
    def span(first: EventId, last: EventId) -> Citation:
        return Citation(ranges=(EventRange(first=first, last=last),))

    def token(self) -> str:
        """`[#1890-1907]`, `[#1890]`, or several comma-separated: what a line ends with."""
        return "[" + ",".join(r.token() for r in self.ranges) + "]"


class L0Record(CognitionModel):
    """One perceived fact, as rendered for this seat."""

    event: EventIdField
    at: int
    kind: EventTypeIdField
    place: EntityIdField | None
    """The fact's own place, `None` when it named none."""
    salience: Salience
    mine: bool
    gist: str
    counterparts: tuple[EntityIdField, ...]


class Episode(CognitionModel):
    """L1: a maximal run of consecutive records at one place, within one day, with no gap over 30 minutes
    of world time (D-P4-7). Numbered in order; an open episode has no text yet."""

    number: Annotated[int, Field(ge=1)]
    day: NonNegative
    place: EntityIdField | None
    first: EventIdField
    last: EventIdField
    opened_at: int
    closed_at: int
    closed: bool
    records: Annotated[int, Field(ge=1)]
    counterparts: tuple[EntityIdField, ...]
    text: str
    prose: str | None

    def citation(self) -> Citation:
        return Citation.span(self.first, self.last)


class Chapter(CognitionModel):
    """L2: the closed episodes of one simulated week (`day // 7`)."""

    week: NonNegative
    first_episode: Annotated[int, Field(ge=1)]
    last_episode: Annotated[int, Field(ge=1)]
    first: EventIdField
    last: EventIdField
    text: str
    prose: str | None

    def citation(self) -> Citation:
        # Consecutive episodes are adjacent in the perceived sequence (D-P4-7), so a week is one range.
        return Citation.span(self.first, self.last)


class StableFact(CognitionModel):
    """L3: what this seat knows of one counterpart, with the facts that establish it."""

    entity: EntityIdField
    first: EventIdField
    """The first perceived fact naming them (as a participant)."""
    last: EventIdField
    times_met: NonNegative
    """Closed episodes they were part of."""
    exchanges: NonNegative
    """Notable facts with them in which I took part."""
    level: str | None
    level_event: EventIdField | None
    """The newest fact that set `level`."""

    def citation(self) -> Citation:
        cited = [self.first, self.last]
        if self.level_event is not None:
            cited.append(self.level_event)
        return Citation.of(*cited)


class StoreIdentity(CognitionModel):
    """Whose memory a store is: refused by name when it differs (D-P4-3)."""

    world_instance: WorldInstanceIdField
    seat: EntityKeyField
    self_entity: EntityIdField
    schema_version: int = SCHEMA_VERSION

    @property
    def me(self) -> EntityId:
        return self.self_entity
