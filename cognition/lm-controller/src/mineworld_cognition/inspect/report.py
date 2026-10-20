"""`InspectReport`: what `mineworld-memory inspect` prints and `inspect.report` returns (pr-s28e §5.4).

Every model is strict and frozen (`CognitionModel`); `InspectReport` is a union discriminated by `view`. A report is
a function of the home's content and the inputs only: no clock, no path but those given, so its JSON is
byte-identical across runs and platforms (I-e-6). `text()` is the human form: a few aligned columns.
"""

from __future__ import annotations

from collections.abc import Sequence
from typing import Annotated, Literal

from pydantic import Field

from mineworld_cognition.backend.model import CognitionModel
from mineworld_cognition.memory.explain import Explanation, Forgotten, HeldCounts, Unit

Source = Literal["given", "default"]
"""Whether an input value came from a file the caller gave (`--decisions`, `--config`) or is the default."""


def _table(header: Sequence[str], rows: Sequence[Sequence[object]]) -> list[str]:
    cells = [[str(cell) for cell in row] for row in [header, *rows]]
    widths = [max(len(row[i]) for row in cells) for i in range(len(header))]
    return [
        "  ".join(cell.ljust(widths[i]) for i, cell in enumerate(row)).rstrip() for row in cells
    ]


def _explanation_lines(found: Explanation, indent: str = "") -> list[str]:
    out = [f"{indent}{found.event}: {found.kind}" + (f" ({found.note})" if found.note else "")]
    if found.record is not None:
        who = ", ".join(f"{row.entity} {row.role}" for row in found.record.who) or "nobody"
        out.append(
            f"{indent}  record  at {found.record.at}, {found.record.kind}, place "
            f"{found.record.place or '-'}; who: {who}"
        )
    if found.access is not None:
        out.append(
            f"{indent}  access  {found.access.salience}, mine {int(found.access.mine)}, importance "
            f"{found.access.importance}, how {found.how}; gist: {found.access.gist}"
        )
    if found.holders is not None:
        out.append(f"{indent}  holders {found.holders} seats of this home (diagnostic)")
    out += [f"{indent}  {_unit(unit)}" for unit in found.units]
    if found.forgotten is not None:
        out.append(f"{indent}  {_forgotten(found.forgotten)}")
    if found.watermark is not None:
        out.append(f"{indent}  watermark {found.watermark}")
    if found.cursor is not None:
        out.append(f"{indent}  cursor {found.cursor}")
    return out


def _unit(unit: Unit) -> str:
    if unit.unit == "episode":
        state = "closed" if unit.closed else "open"
        return (
            f"episode {unit.number} ({state}), day {unit.day}, place {unit.place or '-'}, "
            f"{unit.records} records, #{unit.first}-{unit.last}"
        )
    return f"chapter week {unit.number}, #{unit.first}-{unit.last}"


def _forgotten(found: Forgotten) -> str:
    forced = ", forced" if found.forced else ""
    return (
        f"forgotten #{found.first}-{found.last}: day {found.first_day} by {found.first_policy} … day "
        f"{found.last_day} by {found.last_policy}, {found.passes} passes{forced}"
    )


# ── access ─────────────────────────────────────────────────────────────────────────────────────────


class AccessReport(CognitionModel):
    view: Literal["access"] = "access"
    seat: str
    counts: HeldCounts | None = None
    """Without `--event`: what the seat holds, by how and salience."""
    explanation: Explanation | None = None
    """With `--event`: the id's one class."""
    perceived: Source
    """Whether the seat's perceived stream was given (`--perceived`)."""
    show_fact: str | None = None
    """The command that prints the fact itself (world and save are the user's; the home does not know them)."""

    def text(self) -> str:
        out = [f"seat {self.seat}"]
        if self.counts is not None:
            rows = [(c.how, c.salience, c.records) for c in self.counts.counts]
            out += _table(("how", "salience", "records"), rows)
            out.append(f"total {self.counts.total}")
            out.append(
                f"cursor {self.counts.cursor or '-'}, newest {self.counts.newest or '-'}, "
                f"watermark {self.counts.watermark or '-'}"
            )
        if self.explanation is not None:
            out += _explanation_lines(self.explanation)
        if self.show_fact is not None:
            out.append(f"the fact: {self.show_fact}")
        return "\n".join(out) + "\n"


# ── forgotten ──────────────────────────────────────────────────────────────────────────────────────


class PassRow(CognitionModel):
    """One `retention_passes` row, every column."""

    trigger: str
    day: int
    cause: str
    policy: str
    outcome: str
    kept: int
    merged: int
    dropped: int
    forced: int
    bytes_before: int
    bytes_after: int
    forgotten_ranges: int
    model_decisions: int
    fallback_decisions: int
    prose_contradictions: int


class ForgottenReport(CognitionModel):
    view: Literal["forgotten"] = "forgotten"
    seat: str
    ranges: tuple[Forgotten, ...]
    """The explicit ranges in key order, with their attribution."""
    watermark: str | None
    passes: tuple[PassRow, ...]
    """The seat's pass log (at most 32 rows), newest first."""

    def text(self) -> str:
        out = [f"seat {self.seat}", f"watermark {self.watermark or '-'}"]
        out += [_forgotten(found) for found in self.ranges] or ["no forgotten range"]
        columns = tuple(PassRow.model_fields)
        out += _table(columns, [[getattr(row, name) for name in columns] for row in self.passes])
        return "\n".join(out) + "\n"


# ── bytes ──────────────────────────────────────────────────────────────────────────────────────────


class TableBytes(CognitionModel):
    table: str
    scope: str
    rows: int
    bytes: int


class RowBytes(CognitionModel):
    table: str
    scope: str
    key: str
    """The row's ordering cells (its table's `ORDER BY` columns), comma-joined: an entity, an id, a number."""
    bytes: int


class Bound(CognitionModel):
    mib: int
    bound: int
    target: int
    trigger: int
    source: Source


class SeatBytes(CognitionModel):
    seat: str
    size: int
    content_size: int
    tables: tuple[TableBytes, ...]
    largest: tuple[RowBytes, ...]
    """The five largest rows of the view, largest first."""
    bound: Bound
    fill_per_mille: int
    """`content_size` per mille of the bound."""
    physical_share: int
    """Σ over held records of (the record's home-dump bytes / its holders), rounded down: diagnostic, never
    charged (QS28-4)."""


class SeatSection(CognitionModel):
    seat: str
    bytes: int


class HomeBytes(CognitionModel):
    home_meta: int
    fact: int
    fact_who: int
    seats: tuple[SeatSection, ...]
    dump: int
    file: int
    free: int


class BytesReport(CognitionModel):
    view: Literal["bytes"] = "bytes"
    seat: SeatBytes | None
    home: HomeBytes

    def text(self) -> str:
        out: list[str] = []
        if self.seat is not None:
            seat = self.seat
            out.append(
                f"seat {seat.seat}: size {seat.size}, content {seat.content_size}, bound {seat.bound.bound} "
                f"({seat.bound.mib} MiB, {seat.bound.source}), fill {seat.fill_per_mille}‰, "
                f"physical share {seat.physical_share} (diagnostic, never charged)"
            )
            out += _table(
                ("table", "scope", "rows", "bytes"),
                [(t.table, t.scope, t.rows, t.bytes) for t in seat.tables],
            )
            out.append("largest rows:")
            out += _table(
                ("table", "scope", "key", "bytes"),
                [(r.table, r.scope, r.key, r.bytes) for r in seat.largest],
            )
        home = self.home
        out.append(
            f"home: dump {home.dump} (home_meta {home.home_meta}, fact {home.fact}, fact_who "
            f"{home.fact_who}), file {home.file}, free {home.free}"
        )
        out += _table(("seat", "bytes"), [(s.seat, s.bytes) for s in home.seats])
        return "\n".join(out) + "\n"


# ── recall ─────────────────────────────────────────────────────────────────────────────────────────


class TraceRange(CognitionModel):
    """One cited range: its two ends explained, how many held keys lie inside, and the first of them."""

    first: Explanation
    last: Explanation | None
    """`None` when the range is one id."""
    held_inside: int
    shown: tuple[Explanation, ...]
    """Up to 8 held keys inside the range in key order (all with `--trace-all`), each with its access row and
    record."""


class TraceLine(CognitionModel):
    level: str
    scope: str
    text: str
    """The line as the section renders it, citation token included."""
    citation: str
    ranges: tuple[TraceRange, ...] = ()


class RecallReport(CognitionModel):
    view: Literal["recall"] = "recall"
    seat: str
    at: int
    counterparts: tuple[str, ...]
    words: str
    recall_episodes: int
    recall_source: Source
    stack: tuple[str, ...]
    stack_source: Source
    section: str
    """`MemorySection.render()`, byte for byte: what a consult with these inputs receives."""
    trace: tuple[TraceLine, ...] = ()

    def text(self) -> str:
        out = [
            f"seat {self.seat} at {self.at}; with {', '.join(self.counterparts) or '-'}; words "
            f"{self.words!r}; recall_episodes {self.recall_episodes} ({self.recall_source}); stack "
            f"{', '.join(self.stack)} ({self.stack_source})",
            "",
        ]
        if not self.trace:
            return "\n".join(out) + "\n" + self.section
        for line in self.trace:
            out.append(line.text)
            out.append(f"  {line.level} · scope {line.scope} · {line.citation}")
            for cited in line.ranges:
                out += _explanation_lines(cited.first, "    ")
                if cited.last is not None:
                    out += _explanation_lines(cited.last, "    ")
                out.append(f"    {cited.held_inside} held inside")
                for shown in cited.shown:
                    out += _explanation_lines(shown, "      ")
        return "\n".join(out) + "\n"


InspectReport = Annotated[
    AccessReport | ForgottenReport | BytesReport | RecallReport, Field(discriminator="view")
]
