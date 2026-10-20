"""The views a scope reads (pr-s28c-memory-scopes.md §5.2, QS28c-1).

- `GuardedView` is what every optional scope receives: it implements `ScopeView` and nothing else. It has no
  connection attribute; each of its methods runs the seat's access join itself (`access.seat = ?` bound
  first), and asking it for a key the seat does not hold raises `AccessDenied`.
- `BaseView` is what a base scope (`registry.BASE_SCOPES`) receives: the same guarded methods, plus the seat's
  own `SeatView`, whose seat-filtered queries the person scope's retrieval already runs through. This is the
  one privilege of the interface, named here and granted by the host from `BoundScope.base`.
"""

from __future__ import annotations

from typing import Literal, cast

from mineworld_sdk.wire.ids import EntityId, EntityKey, EventId

from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import Salience, event_key, event_of
from mineworld_cognition.memory.scopes.errors import AccessDenied
from mineworld_cognition.memory.scopes.interface import FactRecord, Reading


class GuardedView:
    """One seat's held facts for the scope `scope`, read through the seat's access join."""

    __slots__ = ("_names", "_scope", "_seat_view")

    def __init__(self, seat_view: SeatView, scope: str) -> None:
        self._seat_view = seat_view
        self._scope = scope
        self._names: NameBook | None = None

    @property
    def seat(self) -> EntityKey:
        return EntityKey(self._seat_view.identity.seat)

    @property
    def me(self) -> EntityId:
        return self._seat_view.me

    def holds(self, event: EventId) -> bool:
        return self._seat_view.holds(event_key(event))

    def record(self, event: EventId) -> tuple[FactRecord, Reading]:
        view = self._seat_view
        row: tuple[str, int, str, str | None, str, Salience, int, int] | None = (
            view.connection.execute(
                "SELECT f.event_id, f.at, f.kind, f.place, a.gist, a.salience, a.mine, a.importance "
                "FROM access a JOIN fact f ON f.event_key = a.event_key "
                "WHERE a.seat = ? AND a.event_key = ?",
                (view.seat_id, event_key(event)),
            ).fetchone()
        )
        if row is None:
            raise AccessDenied(self._scope, self.seat, event)
        event_id, at, kind, place, gist, salience, mine, importance = row
        who: list[tuple[str, int]] = view.connection.execute(
            "SELECT w.entity, w.role FROM access a JOIN fact_who w ON w.event_key = a.event_key "
            "WHERE a.seat = ? AND a.event_key = ? ORDER BY w.ordinal",
            (view.seat_id, event_key(event)),
        ).fetchall()
        fact = FactRecord.model_validate(
            {
                "event": event_id,
                "at": at,
                "kind": kind,
                "place": place,
                "who": tuple((entity, cast(Literal[1, 2, 3], role)) for entity, role in who),
            }
        )
        reading = Reading(gist=gist, salience=salience, mine=bool(mine), importance=importance)
        return fact, reading

    def held_between(self, first_at: int, last_at: int, limit: int) -> tuple[EventId, ...]:
        view = self._seat_view
        rows: list[tuple[str]] = view.connection.execute(
            "SELECT a.event_key FROM access a JOIN fact f ON f.event_key = a.event_key "
            "WHERE a.seat = ? AND f.at >= ? AND f.at <= ? ORDER BY a.event_key DESC LIMIT ?",
            (view.seat_id, first_at, last_at, max(0, limit)),
        ).fetchall()
        return tuple(event_of(key) for (key,) in rows)

    def name(self, entity: EntityId) -> str | None:
        if self._names is None:
            self._names = NameBook.load(self._seat_view)
        return self._names.name(entity)


class BaseView(GuardedView):
    """A base scope's view: the guarded methods and the seat's own `SeatView` (QS28c-1)."""

    __slots__ = ()

    @property
    def seat_view(self) -> SeatView:
        return self._seat_view
