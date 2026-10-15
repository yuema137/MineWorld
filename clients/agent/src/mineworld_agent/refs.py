"""Short references for one handle: `E1`, `E2`, … in first-seen order (step-27 §5.1).

A table belongs to one handle and is never shared (I-c-2): a reference names only an entity this handle's
own observations or facts listed, and resolving any other is `UnknownRef` — which a caller cannot tell
apart from "not perceived" (`CORE_CONCEPTS.md` §15.1). Raw `EntityId`s never leave this module's callers.
"""

from __future__ import annotations

from mineworld_sdk.wire.ids import EntityId


class UnknownRef(LookupError):
    """No entity this handle was shown has that reference."""


class RefTable:
    """`EntityId` ↔ `E<n>`, assigned once and stable for the handle's life."""

    __slots__ = ("_by_id", "_by_ref")

    def __init__(self) -> None:
        self._by_id: dict[EntityId, str] = {}
        self._by_ref: dict[str, EntityId] = {}

    def ref(self, entity: EntityId) -> str:
        """The entity's reference, assigning the next one the first time it is seen."""
        known = self._by_id.get(entity)
        if known is not None:
            return known
        made = f"E{len(self._by_id) + 1}"
        self._by_id[entity] = made
        self._by_ref[made] = entity
        return made

    def resolve(self, ref: str) -> EntityId:
        """The entity behind `ref`, or `UnknownRef`."""
        entity = self._by_ref.get(ref)
        if entity is None:
            raise UnknownRef(ref)
        return entity

    def __len__(self) -> int:
        return len(self._by_id)

    def __repr__(self) -> str:
        return f"RefTable({len(self._by_id)} references)"
