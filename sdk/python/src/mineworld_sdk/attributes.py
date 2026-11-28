"""Disclosed attributes, read and composed (S30-f §§5.4, 5.5; `MODULE_SPEC.md` §4.1 "Disclosure").

```text
block       {schema: 3, words?, entries} — the payload of a `traits` component, or the member
            `attributes` of any other component's payload; any other shape, or an entry that does not
            decode strictly, skips the whole block (counted as unreadable), never half-read
entries     every readable block of one entity, merged by (rank is None, rank, attribute)
reading     the reader's locale among the offered ones (its own; else the one offered locale of its
            language; else English) and the words it composes with (S30-k §5.6, `ARC-129`)
compose     select by facet, join with the reading's `join`, cut at an entry boundary with `more`
inspect     every entry with text, one line each
sheet       "label<pair>text", or "label<pair><typed value>" when it renders no text
```

Localised text is the server's too: an entry carries `texts` (and, in the holder's view, `labels`) per
offered locale; an entry without the reader's locale is shown in English, whole.

The server renders every phrase; this module selects, merges, joins and cuts server text only. It
names no attribute, no backend and no facet other than as a parameter. The Rust reader
(`authoring/attributes/src/disclosed.rs`) and the GDScript one apply the same composition, and one
hand-written golden (`authoring/attributes/tests/fixtures/composition.json`) holds them equal.
"""

from __future__ import annotations

import re
import unicodedata
from collections.abc import Iterable, Mapping, Sequence
from dataclasses import dataclass, field
from typing import Annotated, Literal, cast

from pydantic import AfterValidator, BeforeValidator, Field, ValidationError

from mineworld_sdk.wire.contract import I64, U32, PerceivedEntity, WireModel
from mineworld_sdk.wire.ids import JsonValue

SCHEMA = 3
"""The block schema this reader reads."""

TRAITS = "traits"
"""The component whose payload is a block."""

OWNER_MEMBER = "attributes"
"""The member of any other component's payload that carries its owner's block."""

SEPARATOR = ", "
"""Between the texts of an English composition."""

CUT_MARK = " …"
"""What an English composition cut at an entry boundary ends with."""

ENGLISH = "en"
"""The locale of the declarations themselves: every entry's `label` and `text`."""

_ELLIPSIS = "…"
_TEXT_MAX_BYTES = 280
_MAX_SCALE = 6
_I64_MAX = 2**63 - 1

_LOCALE = re.compile(r"[a-z]{2,3}(?:_[A-Z][a-z]{3})?(?:_(?:[A-Z]{2}|[0-9]{3}))?")
_QUALIFIED = re.compile(r"[a-z][a-z0-9-]{0,63}\.[a-z][a-z0-9_.-]{0,63}")
_VALUE_SLUG = re.compile(r"[a-z0-9][a-z0-9_.-]{0,63}")
_DECIMAL = re.compile(r"(-?)([0-9]+)(?:\.([0-9]+))?")

Kind = Literal["category", "level", "quantity", "flag", "set", "text", "instant"]
"""An attribute kind's name."""


def _qualified(text: str) -> str:
    if _QUALIFIED.fullmatch(text) is None:
        raise ValueError(f"'{text}' is not a qualified attribute id (<namespace>.<slug>)")
    return text


def _canonical_locale(text: str) -> str:
    if _LOCALE.fullmatch(text) is None:
        raise ValueError(f"'{text}' is not a canonical locale (such as en, zh_Hans, pt_BR)")
    return text


def parse_locale(text: str) -> str:
    """A locale as a person types it, in canonical form: `zh-Hans` is read as `zh_Hans` (S30-k §5.1).

    Raises `ValueError` when the text is not a language of 2 or 3 lower-case letters, then optionally
    `_` and a 4-letter script in title case, then optionally `_` and a 2-letter region in upper case or
    3 digits."""
    normalised = text.replace("-", "_")
    if _LOCALE.fullmatch(normalised) is None:
        raise ValueError(
            f"'{text}' is not a locale: write a language of 2 or 3 lower-case letters, then optionally "
            "'_' and a 4-letter script in title case, then optionally '_' and a 2-letter region in "
            "upper case or 3 digits (such as en, pt_BR)"
        )
    return normalised


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


Locale = Annotated[str, AfterValidator(_canonical_locale)]
"""A locale in canonical form, as a block carries it (`zh_Hans`)."""


class AttributeEntry(WireModel):
    """One disclosed attribute (schema 3)."""

    attribute: QualifiedId
    kind: Kind
    label: str
    facets: Facets
    rank: U32 | None = None
    value: AttributeValue | None = None
    text: str | None = None
    labels: dict[Locale, str] = Field(default_factory=dict[Locale, str])
    """The label per offered locale; in the holder's view only."""
    texts: dict[Locale, str] = Field(default_factory=dict[Locale, str])
    """The text per offered locale the entry is rendered in, when it renders one."""


class ReaderWords(WireModel):
    """The words a reader composes with in one locale."""

    join: str
    more: str
    pair: str
    yes: str
    no: str


ENGLISH_WORDS = ReaderWords(join=SEPARATOR, more=CUT_MARK, pair=": ", yes="yes", no="no")
"""English, as S30-f's readers wrote it."""


class Block(WireModel):
    """A disclosed payload: `{schema: 3, words?, entries}`."""

    schema_: U32 = Field(alias="schema")
    words: dict[Locale, ReaderWords] = Field(default_factory=dict[Locale, ReaderWords])
    entries: Annotated[tuple[AttributeEntry, ...], BeforeValidator(_as_tuple)]


@dataclass(frozen=True)
class Disclosed:
    """The entries of every readable block of one entity, merged in rank order, how many blocks were
    present but unreadable (another schema, or an entry that does not decode), and the reader words
    per offered locale (the first readable block's: one server renders every block from one set of
    label packs)."""

    entries: tuple[AttributeEntry, ...] = ()
    attributes_unreadable: int = 0
    words: Mapping[str, ReaderWords] = field(default_factory=dict[str, ReaderWords])

    def reading(self, locale: str) -> Reading:
        """How a reader wanting `locale` reads these entries."""
        return reading(locale, self.words)


@dataclass(frozen=True)
class Reading:
    """How one reader reads: the offered locale it reads in (`None` for English) and its words."""

    locale: str | None = None
    words: ReaderWords = ENGLISH_WORDS

    def text(self, entry: AttributeEntry) -> str | None:
        """`entry`'s text in this reading: its localised text, else its English one."""
        if self.locale is not None and self.locale in entry.texts:
            return entry.texts[self.locale]
        return entry.text

    def label(self, entry: AttributeEntry) -> str | None:
        """`entry`'s localised label, when the holder's view carries one in this reading."""
        return None if self.locale is None else entry.labels.get(self.locale)


ENGLISH_READING = Reading()
"""English: S30-f's output, byte for byte."""


def reading(wanted: str, offered: Mapping[str, ReaderWords]) -> Reading:
    """The reading of a reader wanting `wanted` among the `offered` locales: `wanted` when offered;
    else the one offered locale with the same language, when exactly one has it; else English."""
    locale = parse_locale(wanted)
    if locale == ENGLISH:
        return ENGLISH_READING
    if locale in offered:
        return Reading(locale, offered[locale])
    language = locale.split("_")[0]
    same = [candidate for candidate in offered if candidate.split("_")[0] == language]
    if len(same) == 1:
        return Reading(same[0], offered[same[0]])
    return ENGLISH_READING


def _block(payload: JsonValue) -> Block | None:
    if not isinstance(payload, dict):
        return None
    schema = payload.get("schema")
    if type(schema) is not int or schema != SCHEMA:
        return None
    try:
        return Block.model_validate(payload)
    except ValidationError:
        return None


def _rank_order(entry: AttributeEntry) -> tuple[bool, int, str]:
    return (entry.rank is None, entry.rank or 0, entry.attribute)


def read_payloads(components: Iterable[tuple[str, JsonValue]]) -> Disclosed:
    """The merged blocks of `(component_type, payload)` pairs; never fails."""
    entries: list[AttributeEntry] = []
    unreadable = 0
    words: dict[str, ReaderWords] | None = None
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
        if words is None:
            words = dict(block.words)
        entries.extend(block.entries)
    return Disclosed(tuple(sorted(entries, key=_rank_order)), unreadable, words or {})


def read(entity: PerceivedEntity) -> Disclosed:
    """The merged blocks of `entity`'s components."""
    return read_payloads((record.component_type, record.payload) for record in entity.components)


def blocks(entity: PerceivedEntity) -> tuple[AttributeEntry, ...]:
    """The entries `entity` discloses, merged in rank order; unknown shapes skipped."""
    return read(entity).entries


def _texts(entries: Sequence[AttributeEntry], facet: str | None, reading: Reading) -> list[str]:
    return [
        text
        for entry in entries
        if (facet is None or facet in entry.facets) and (text := reading.text(entry))
    ]


def _utf8(text: str) -> int:
    return len(text.encode())


def compose(
    entries: Sequence[AttributeEntry],
    facet: str | None,
    budget: int | None,
    reading: Reading = ENGLISH_READING,
) -> str:
    """Select the entries carrying `facet` (every entry when `None`) with a non-empty text in
    `reading`, join them with its `join`, and cut at `budget` UTF-8 bytes at an entry boundary with its
    `more`; a first entry longer than the budget alone is cut at a character boundary and ends with
    "…"."""
    separator, cut_mark = reading.words.join, reading.words.more
    texts = _texts(entries, facet, reading)
    joined = separator.join(texts)
    if budget is None or _utf8(joined) <= budget:
        return joined
    mark = _utf8(cut_mark)
    length = 0
    kept = 0
    for index, text in enumerate(texts):
        following = length + _utf8(text) + (0 if index == 0 else _utf8(separator))
        if following + mark > budget:
            break
        length = following
        kept = index + 1
    if kept > 0:
        return separator.join(texts[:kept]) + cut_mark
    room = max(0, budget - _utf8(_ELLIPSIS))
    first = texts[0].encode()[:room].decode(errors="ignore")
    return first + _ELLIPSIS


def inspect(
    entries: Sequence[AttributeEntry], reading: Reading = ENGLISH_READING
) -> tuple[str, ...]:
    """Every entry's non-empty text in `reading`, one line each, in order."""
    return tuple(_texts(entries, None, reading))


def typed(value: AttributeValue, words: ReaderWords = ENGLISH_WORDS) -> str:
    """A value as the own sheet shows it when its entry renders no text, with `words`' flag words."""
    match value:
        case CategoryValue(category=slug):
            return slug
        case LevelValue(level=slug):
            return slug
        case QuantityValue(quantity=quantity):
            return quantity
        case FlagValue(flag=flag):
            return words.yes if flag else words.no
        case SetValue(set=members):
            return words.join.join(members)
        case TextValue(text=text):
            return text
        case InstantValue(instant=seconds):
            return str(seconds)


@dataclass(frozen=True)
class SheetLine:
    """One own-sheet line in a reading: the label, what it shows, and the words it is joined with.
    An entry the holder's view carries in the reading's locale is shown in it; any other in English,
    whole (D-k-2)."""

    label: str
    shown: str
    words: ReaderWords

    def __str__(self) -> str:
        return f"{self.label}{self.words.pair}{self.shown}"


def sheet_line(entry: AttributeEntry, reading: Reading = ENGLISH_READING) -> SheetLine | None:
    """`entry` on the own sheet in `reading`: its text, else its typed value; `None` when that is empty
    or absent (an empty set, an empty text value)."""
    label = reading.label(entry)
    if label is None:
        label, text, words = entry.label, entry.text, ENGLISH_WORDS
    else:
        text, words = reading.text(entry), reading.words
    if text:
        return SheetLine(label, text, words)
    if entry.value is not None and (value := typed(entry.value, words)):
        return SheetLine(label, value, words)
    return None


def shown(entry: AttributeEntry) -> str | None:
    """What the English own sheet shows for `entry`: its text, else its typed value; `None` when that
    is empty or absent (an empty set, an empty text value)."""
    line = sheet_line(entry)
    return None if line is None else line.shown


def sheet(entries: Sequence[AttributeEntry], reading: Reading = ENGLISH_READING) -> tuple[str, ...]:
    """The own sheet in `reading`: `label<pair>text`, or `label<pair><typed value>`; an entry that
    shows nothing is left out."""
    return tuple(str(line) for entry in entries if (line := sheet_line(entry, reading)) is not None)
