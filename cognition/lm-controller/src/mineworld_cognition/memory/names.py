"""`NameBook`: the display names this seat knows, and the generic labels it uses otherwise (P4-5).

A rendered line never shows an entity id. A person with no known name is `someone`, a place with no
known label `somewhere`. Names are learned from perceived `named` facts (by ingestion, through the
`named` renderer) and from observations (by P6, `NameBook.learn`); `learn` is the only write path.
"""

from __future__ import annotations

import re
from collections.abc import Iterable

from mineworld_sdk.wire.ids import EntityId, EventId

from mineworld_cognition.memory.records import event_key
from mineworld_cognition.memory.store import MemoryStore

SOMEONE = "someone"
SOMEWHERE = "somewhere"
MAX_NAME_LENGTH = 60
_SPACE = re.compile(r"\s+")


def one_line(text: str) -> str:
    """World text on one line: every run of whitespace, newlines included, becomes one space."""
    return _SPACE.sub(" ", text).strip()


class NameBook:
    """Names, read from the store and written through it."""

    def __init__(self, store: MemoryStore, names: dict[EntityId, str]) -> None:
        self._store = store
        self._names = names

    @staticmethod
    def load(store: MemoryStore) -> NameBook:
        rows: list[tuple[str, str]] = store.connection.execute(
            "SELECT entity, name FROM names"
        ).fetchall()
        return NameBook(store, {EntityId(entity): name for entity, name in rows})

    def learn(self, entity: EntityId, name: str, source: EventId | None = None) -> None:
        """Teaches a name; `source` is the perceived fact that disclosed it, if one did."""
        clean = one_line(name)[:MAX_NAME_LENGTH]
        if not clean or self._names.get(entity) == clean:
            return
        self._names[entity] = clean
        self._store.connection.execute(
            "INSERT INTO names (entity, name, source_key) VALUES (?, ?, ?) "
            "ON CONFLICT (entity) DO UPDATE SET name = excluded.name, "
            "source_key = excluded.source_key",
            (entity, clean, None if source is None else event_key(source)),
        )

    def name(self, entity: EntityId) -> str | None:
        return self._names.get(entity)

    def person(self, entity: EntityId) -> str:
        return self._names.get(entity, SOMEONE)

    def place(self, entity: EntityId | None) -> str:
        return SOMEWHERE if entity is None else self._names.get(entity, SOMEWHERE)

    def people(self, entities: Iterable[EntityId], *, most: int = 3) -> str:
        """`Bob Achterberg, Carol Mensah and others`: up to `most` known names in the given order; any
        unknown or further person is folded into `someone` or `others`, never into an id."""
        shown: list[str] = []
        rest = 0
        for entity in entities:
            known = self._names.get(entity)
            if known is not None and len(shown) < most and known not in shown:
                shown.append(known)
            else:
                rest += 1
        if not shown:
            return "" if rest == 0 else SOMEONE if rest == 1 else "others"
        if rest == 0:
            return ", ".join(shown)
        return ", ".join(shown) + (" and someone else" if rest == 1 else " and others")
