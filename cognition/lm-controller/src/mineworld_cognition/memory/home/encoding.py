"""The positional canonical line (pr-s28a-memory-home.md §5.4; QS28-5, `ARC-101`): what the bound and determinism
are defined on.

A line is the compact JSON array `[table, c1, …, cn]` (no spaces, UTF-8 kept as it is), then a newline. Each cell
goes through its column's codec first (`schema.codec`): an `EventKey` or `EventId` column is written as the
unpadded decimal number (a u64 is exact in Python's `int` and in JSON), a digest as its first 32 hex characters
(128 bits; the full 64 stay in the table and are the re-delivery check), every other value as stored (`INTEGER` →
number, `TEXT` → string, `NULL` → `null`). No table holds a float. One encoder, built once: retention measures
lines at every write and every pass.
"""

from __future__ import annotations

import json
from collections.abc import Callable, Sequence

type Cell = int | str | None
type Codec = Callable[[Cell], Cell]

DIGEST_CHARS = 32
_ENCODER = json.JSONEncoder(ensure_ascii=False, separators=(",", ":"))


def raw(value: Cell) -> Cell:
    return value


def as_number(value: Cell) -> Cell:
    """An id or a key as its number; an unset one (`""` or `NULL`) as `null`. Only the dump converts: ids stay
    strings in every table and every API (P4's rule)."""
    if value is None or value == "":
        return None
    return int(value)


def short_digest(value: Cell) -> Cell:
    assert isinstance(value, str), "a digest is text"
    return value[:DIGEST_CHARS]


def line(table: str, cells: Sequence[Cell], codecs: Sequence[Codec]) -> str:
    """A row's canonical line, without its newline."""
    return _ENCODER.encode([table, *(code(cell) for code, cell in zip(codecs, cells, strict=True))])


def line_bytes(table: str, cells: Sequence[Cell], codecs: Sequence[Codec]) -> int:
    """The bytes a row's line takes in a dump, newline included."""
    return len(line(table, cells, codecs).encode("utf-8")) + 1


def encoded(table: str, cells: Sequence[Cell]) -> str:
    """A line whose cells are already written (the home dump's grouped `fact_who` line)."""
    return _ENCODER.encode([table, *cells])
