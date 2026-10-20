"""The anchor oracle (pr-s28d §6.2): test-only, importing no module of `mineworld_cognition`.

It reads perceived facts in their wire form (`PerceivedEvent.model_dump(mode="json")`, the JSON lines of
`mineworld perceived --json`) and re-implements, by itself, the payload scan of §5.2 from its numbered rules, the
anchored history of a person's export, the known counts over the whole export (F-S28d-9), and the objective place
history C-3 names. It is never an input to memory.
"""

from __future__ import annotations

from collections.abc import Iterable, Mapping, Sequence
from typing import cast

type Wire = Mapping[str, object]
KINDS = ("place", "item", "organization")
PER_FACT = 16


def _is_id(value: object) -> bool:
    if not isinstance(value, str) or not value or not value.isascii() or not value.isdigit():
        return False
    return value == "0" or not value.startswith("0")


def scan(payload: object) -> list[tuple[str, str]]:
    """§5.2 rules 1 to 5, written again: `(entity, kind)` pairs, first occurrence, at most 16."""
    out: list[tuple[str, str]] = []
    seen: set[str] = set()

    def walk(value: object) -> None:
        if len(out) >= PER_FACT:
            return
        if isinstance(value, dict):
            members = cast(dict[str, object], value)
            entity, kind = members.get("entity"), members.get("entity_type")
            if (
                set(members) == {"entity", "entity_type"}
                and isinstance(entity, str)
                and _is_id(entity)
                and isinstance(kind, str)
            ):
                if kind in KINDS and entity not in seen:
                    seen.add(entity)
                    out.append((entity, kind))
                return
            for key in sorted(members):
                walk(members[key])
        elif isinstance(value, list):
            for element in cast(list[object], value):
                walk(element)

    walk(payload)
    return out[:PER_FACT]


def _place(fact: Wire) -> str | None:
    place = fact.get("place")
    if isinstance(place, dict):
        return cast(str, cast(dict[str, object], place)["entity"])
    return None


def _payload(fact: Wire) -> object:
    return cast(dict[str, object], fact["payload"])["payload"]


def anchored_to(
    fact: Wire, entity: str, roles: Sequence[str], kinds: Sequence[str], me: str
) -> bool:
    if entity == me:
        return False
    if "at" in roles and "place" in kinds and _place(fact) == entity:
        return True
    if "named" in roles:
        return any(e == entity and k in kinds for e, k in scan(_payload(fact)))
    return False


def history(
    export: Iterable[Wire], entity: str, roles: Sequence[str], kinds: Sequence[str], me: str
) -> list[str]:
    """The ids of the facts anchored to `entity`, in id order."""
    facts = sorted(export, key=lambda f: int(cast(str, f["id"])))
    return [cast(str, f["id"]) for f in facts if anchored_to(f, entity, roles, kinds, me)]


def known(
    export: Iterable[Wire], entity: str, roles: Sequence[str], kinds: Sequence[str], me: str
) -> tuple[int, int, int] | None:
    """`(visits, first_at, last_at)` over the whole export, or `None` when no fact is anchored to it."""
    facts = sorted(export, key=lambda f: int(cast(str, f["id"])))
    times = [cast(int, f["at"]) for f in facts if anchored_to(f, entity, roles, kinds, me)]
    if not times:
        return None
    return len(times), times[0], times[-1]


def anchors_in(
    export: Iterable[Wire], roles: Sequence[str], kinds: Sequence[str], me: str
) -> list[str]:
    """Every entity the export anchors under these roles and kinds, in first-seen order."""
    seen: dict[str, None] = {}
    for fact in sorted(export, key=lambda f: int(cast(str, f["id"]))):
        if "at" in roles and "place" in kinds:
            place = _place(fact)
            if place is not None and place != me:
                seen.setdefault(place, None)
        if "named" in roles:
            for entity, kind in scan(_payload(fact)):
                if kind in kinds and entity != me:
                    seen.setdefault(entity, None)
    return list(seen)


def objective(exports: Iterable[Iterable[Wire]], place: str) -> set[str]:
    """The union of every person's facts whose envelope place is `place`: the test oracle C-3 names."""
    return {cast(str, f["id"]) for export in exports for f in export if _place(f) == place}
