"""`NameBook`: the display names this seat knows, and the generic labels it uses otherwise (P4-5).

A rendered line never shows an entity id. A person with no known name is `someone`, a place with no
known label `somewhere`. Names are learned from perceived `named` facts (by ingestion, through the
`named` renderer) and from observations (by P6, `NameBook.learn`); `learn` is the only write path.

A place has no display name in the shipped packs: the world discloses what a place is by its tags (its own,
to whoever stands in it, and a doorway's `to_tags`, to whoever stands beside it; ARC-82). Such a place is
labelled by `tag_label`, its tags joined in their disclosed order (`cafe/public`), until a display name
replaces it (D-P6-43). No tag is singled out, so no world's vocabulary is built in.
"""

from __future__ import annotations

import re
from collections.abc import Iterable, Mapping
from types import MappingProxyType

from mineworld_sdk.wire.ids import EntityId, EventId

from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.records import event_key

SOMEONE = "someone"
SOMEWHERE = "somewhere"
MAX_NAME_LENGTH = 60
TAG_SEPARATOR = "/"
_SPACE = re.compile(r"\s+")


def one_line(text: str) -> str:
    """World text on one line: every run of whitespace, newlines included, becomes one space."""
    return _SPACE.sub(" ", text).strip()


def tag_label(tags: Iterable[str]) -> str | None:
    """A place's label from its disclosed tags, in their order: `cafe/public`. `None` without a tag."""
    words = [one_line(tag) for tag in tags]
    label = TAG_SEPARATOR.join(word for word in words if word)
    return label or None


class NameBook:
    """A seat's names, read from its view and written through it."""

    def __init__(self, store: SeatView, names: dict[EntityId, str]) -> None:
        self._store = store
        self._names = names

    @staticmethod
    def load(store: SeatView) -> NameBook:
        rows: list[tuple[str, str]] = store.connection.execute(
            "SELECT entity, name FROM names WHERE seat = ?", (store.seat_id,)
        ).fetchall()
        return NameBook(store, {EntityId(entity): name for entity, name in rows})

    def learn(self, entity: EntityId, name: str, source: EventId | None = None) -> None:
        """Teaches a name; `source` is the perceived fact that disclosed it, if one did."""
        clean = one_line(name)[:MAX_NAME_LENGTH]
        if not clean or self._names.get(entity) == clean:
            return
        self._names[entity] = clean
        self._store.connection.execute(
            "INSERT INTO names (seat, entity, name, source_key) VALUES (?, ?, ?, ?) "
            "ON CONFLICT (seat, entity) DO UPDATE SET name = excluded.name, "
            "source_key = excluded.source_key",
            (self._store.seat_id, entity, clean, None if source is None else event_key(source)),
        )

    def name(self, entity: EntityId) -> str | None:
        return self._names.get(entity)

    def known(self) -> Mapping[EntityId, str]:
        """Every name this seat knows, read-only (the prose name check reads them, pr-s28b §5.4)."""
        return MappingProxyType(self._names)

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
