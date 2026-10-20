"""Disclosed attributes, read and composed (S30-f §§5.4, 5.5; `MODULE_SPEC.md` §4.1 "Disclosure").

```text
block       {schema: 2, entries} — the payload of a `traits` component, or the member `attributes`
            of any other component's payload; any other shape, or an entry that does not decode
            strictly, skips the whole block (counted as unreadable), never half-read
entries     every readable block of one entity, merged by (rank is None, rank, attribute)
compose     select by facet, join with ", ", cut at an entry boundary with " …"
inspect     every entry with text, one line each
sheet       "label: text", or "label: <typed value>" when it renders no text
```

The server renders every phrase; this module selects, merges, joins and cuts server text only. It
names no attribute, no backend and no facet other than as a parameter. The Rust reader
(`authoring/attributes/src/disclosed.rs`) and the GDScript one apply the same composition, and one
hand-written golden (`authoring/attributes/tests/fixtures/composition.json`) holds them equal.
"""

from __future__ import annotations

import re
import unicodedata
from collections.abc import Iterable, Sequence
from dataclasses import dataclass
from typing import Annotated, Literal, cast

from pydantic import AfterValidator, BeforeValidator, Field, ValidationError

from mineworld_sdk.wire.contract import I64, U32, PerceivedEntity, WireModel
from mineworld_sdk.wire.ids import JsonValue

SCHEMA = 2
"""The block schema this reader reads."""

TRAITS = "traits"
"""The component whose payload is a block."""

OWNER_MEMBER = "attributes"
"""The member of any other component's payload that carries its owner's block."""

SEPARATOR = ", "
"""Between the texts of a composition."""

CUT_MARK = " …"
"""What a composition cut at an entry boundary ends with."""

_ELLIPSIS = "…"
_TEXT_MAX_BYTES = 280
_MAX_SCALE = 6
_I64_MAX = 2**63 - 1

_QUALIFIED = re.compile(r"[a-z][a-z0-9-]{0,63}\.[a-z][a-z0-9_.-]{0,63}")
_VALUE_SLUG = re.compile(r"[a-z0-9][a-z0-9_.-]{0,63}")
_DECIMAL = re.compile(r"(-?)([0-9]+)(?:\.([0-9]+))?")

Kind = Literal["category", "level", "quantity", "flag", "set", "text", "instant"]
"""An attribute kind's name."""


def _qualified(text: str) -> str:
    if _QUALIFIED.fullmatch(text) is None:
        raise ValueError(f"'{text}' is not a qualified attribute id (<namespace>.<slug>)")
    return text


def _value_slug(text: str) -> str:
    if _VALUE_SLUG.fullmatch(text) is None:
        raise ValueError(f"'{text}' is not a value slug")
    return text


def _bounded(text: str) -> str:
    if len(text.encode()) > _TEXT_MAX_BYTES:
        raise ValueError(f"text over {_TEXT_MAX_BYTES} bytes")
    if any(unicodedata.category(char) == "Cc" for char in text):
        raise ValueError("text holds a control character")
    return text


def _decimal(text: str) -> str:
    """`text` as the exact decimal it denotes, written as the Rust reader writes it back."""
    match = _DECIMAL.fullmatch(text)
    if match is None:
        raise ValueError(f"'{text}' is not a decimal number")
    sign, whole, fraction = match.group(1), match.group(2), match.group(3) or ""
    if len(fraction) > _MAX_SCALE:
        raise ValueError(f"{text} has more than {_MAX_SCALE} decimals")
    magnitude = int(whole + fraction)
    if magnitude > _I64_MAX:
        raise ValueError(f"{text} is too large")
    negative = sign == "-" and magnitude != 0
    if not fraction:
        return f"{'-' if negative else ''}{magnitude}"
    unit = 10 ** len(fraction)
    return f"{'-' if negative else ''}{magnitude // unit}.{magnitude % unit:0{len(fraction)}d}"


def _as_tuple(value: object) -> object:
    """A JSON list as a tuple; anything else is left for strict validation."""
    return tuple(cast("list[object]", value)) if isinstance(value, list) else value


def _sorted_unique(value: object) -> object:
    """A JSON list of strings as the sorted set it denotes; anything else is left for strict
    validation."""
    if not isinstance(value, list):
        return value
    items = cast("list[object]", value)
    strings = [item for item in items if isinstance(item, str)]
    return tuple(sorted(set(strings))) if len(strings) == len(items) else tuple(items)


QualifiedId = Annotated[str, AfterValidator(_qualified)]
ValueSlug = Annotated[str, AfterValidator(_value_slug)]
Decimal = Annotated[str, AfterValidator(_decimal)]
BoundedText = Annotated[str, AfterValidator(_bounded)]
SortedSet = Annotated[tuple[ValueSlug, ...], BeforeValidator(_sorted_unique)]
Facets = Annotated[tuple[str, ...], BeforeValidator(_sorted_unique)]


class CategoryValue(WireModel):
    """`{"category": slug}`."""

    category: ValueSlug


class LevelValue(WireModel):
    """`{"level": slug, "rank": n}`: a level and its position in the declared order."""

    level: ValueSlug
    rank: Annotated[int, Field(ge=0, le=2**16 - 1)]


class QuantityValue(WireModel):
    """`{"quantity": "1.660"}`: an exact decimal, kept as its string (never a float)."""

    quantity: Decimal


class FlagValue(WireModel):
    """`{"flag": bool}`."""

    flag: bool


class SetValue(WireModel):
    """`{"set": [slug, ...]}`, sorted and without repeats."""

    set: SortedSet


class TextValue(WireModel):
    """`{"text": "..."}`: bounded prose."""

    text: BoundedText


class InstantValue(WireModel):
    """`{"instant": seconds}`: a world instant."""

    instant: I64


AttributeValue = (
    CategoryValue | LevelValue | QuantityValue | FlagValue | SetValue | TextValue | InstantValue
)
"""One attribute's value, externally tagged by kind; exactly one kind's key."""


class AttributeEntry(WireModel):
    """One disclosed attribute (schema 2)."""

    attribute: QualifiedId
    kind: Kind
    label: str
    facets: Facets
    rank: U32 | None = None
    value: AttributeValue | None = None
    text: str | None = None


class Block(WireModel):
    """A disclosed payload: `{schema: 2, entries}`."""

    schema_: U32 = Field(alias="schema")
    entries: Annotated[tuple[AttributeEntry, ...], BeforeValidator(_as_tuple)]


@dataclass(frozen=True)
class Disclosed:
    """The entries of every readable block of one entity, merged in rank order, and how many blocks
    were present but unreadable (a newer schema, or an entry that does not decode)."""

    entries: tuple[AttributeEntry, ...] = ()
    attributes_unreadable: int = 0


def _block(payload: JsonValue) -> tuple[AttributeEntry, ...] | None:
    if not isinstance(payload, dict):
        return None
    schema = payload.get("schema")
    if type(schema) is not int or schema != SCHEMA:
        return None
    try:
        return Block.model_validate(payload).entries
    except ValidationError:
        return None


def _rank_order(entry: AttributeEntry) -> tuple[bool, int, str]:
    return (entry.rank is None, entry.rank or 0, entry.attribute)


def read_payloads(components: Iterable[tuple[str, JsonValue]]) -> Disclosed:
    """The merged blocks of `(component_type, payload)` pairs; never fails."""
    entries: list[AttributeEntry] = []
    unreadable = 0
    for component_type, payload in components:
        if component_type == TRAITS:
            candidate: JsonValue = payload
        elif isinstance(payload, dict) and OWNER_MEMBER in payload:
            candidate = payload[OWNER_MEMBER]
        else:
            continue
        block = _block(candidate)
        if block is None:
            unreadable += 1
            continue
        entries.extend(block)
    return Disclosed(tuple(sorted(entries, key=_rank_order)), unreadable)


def read(entity: PerceivedEntity) -> Disclosed:
    """The merged blocks of `entity`'s components."""
    return read_payloads((record.component_type, record.payload) for record in entity.components)


def blocks(entity: PerceivedEntity) -> tuple[AttributeEntry, ...]:
    """The entries `entity` discloses, merged in rank order; unknown shapes skipped."""
    return read(entity).entries


def _texts(entries: Sequence[AttributeEntry], facet: str | None) -> list[str]:
    return [
        entry.text for entry in entries if (facet is None or facet in entry.facets) and entry.text
    ]


def compose(entries: Sequence[AttributeEntry], facet: str | None, budget: int | None) -> str:
    """Select the entries carrying `facet` (every entry when `None`) with a non-empty text, join them
    with ", ", and cut at `budget` UTF-8 bytes at an entry boundary with " …"; a first entry longer
    than the budget alone is cut at a character boundary and ends with "…"."""
    texts = _texts(entries, facet)
    joined = SEPARATOR.join(texts)
    if budget is None or len(joined.encode()) <= budget:
        return joined
    mark = len(CUT_MARK.encode())
    length = 0
    kept = 0
    for index, text in enumerate(texts):
        following = length + len(text.encode()) + (0 if index == 0 else len(SEPARATOR))
        if following + mark > budget:
            break
        length = following
        kept = index + 1
    if kept > 0:
        return SEPARATOR.join(texts[:kept]) + CUT_MARK
    room = max(0, budget - len(_ELLIPSIS.encode()))
    first = texts[0].encode()[:room].decode(errors="ignore")
    return first + _ELLIPSIS


def inspect(entries: Sequence[AttributeEntry]) -> tuple[str, ...]:
    """Every entry's non-empty text, one line each, in order."""
    return tuple(_texts(entries, None))


def typed(value: AttributeValue) -> str:
    """A value as the own sheet shows it when its entry renders no text."""
    match value:
        case CategoryValue(category=slug):
            return slug
        case LevelValue(level=slug):
            return slug
        case QuantityValue(quantity=quantity):
            return quantity
        case FlagValue(flag=flag):
            return "yes" if flag else "no"
        case SetValue(set=members):
            return SEPARATOR.join(members)
        case TextValue(text=text):
            return text
        case InstantValue(instant=seconds):
            return str(seconds)


def shown(entry: AttributeEntry) -> str | None:
    """What the own sheet shows for `entry`: its text, else its typed value; `None` when that is
    empty or absent (an empty set, an empty text value)."""
    if entry.text:
        return entry.text
    if entry.value is not None:
        return typed(entry.value) or None
    return None


def sheet(entries: Sequence[AttributeEntry]) -> tuple[str, ...]:
    """The own sheet: `label: text`, or `label: <typed value>`; an entry that shows nothing is left
    out."""
    return tuple(
        f"{entry.label}: {line}" for entry in entries if (line := shown(entry)) is not None
    )
