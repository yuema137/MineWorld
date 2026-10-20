"""The recall host: a seat's memory stack run, every line checked, one section assembled (pr-s28c §5.7).

For each bound scope, in stack order, the host builds the scope's view (`BaseView` for a base scope,
`GuardedView` otherwise), calls its `recall`, and checks every returned line before it can join the section:

1. its key is one of the scope's sections (`ScopeSectionOverBudget`, `what="section"`);
2. it is at most `LINE_BYTES` (`what="line"`);
3. each section keeps to its lines and its bytes, header and newlines included (`what="lines"`, `"bytes"`);
4. every range end it cites is in the seat's citable set (`AccessDenied`, QS28c-2).

A scope whose `recall` raises is `ScopeRecallFailed`. With `on_defect="raise"` the first error is raised; with
`"withhold"` that scope contributes nothing to this call and a `ScopeDefect` records why (QS28c-3). Every check
is a function of the view and the lines, so a replay withholds the same scope.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

from mineworld_sdk.wire.ids import EntityKey

from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import event_key
from mineworld_cognition.memory.retrieve import (
    LINE_BYTES,
    MemorySection,
    SectionHeading,
    header,
)
from mineworld_cognition.memory.scopes.errors import (
    AccessDenied,
    ScopeError,
    ScopeRecallFailed,
    ScopeSectionOverBudget,
)
from mineworld_cognition.memory.scopes.interface import RecallQuery, SectionLine
from mineworld_cognition.memory.scopes.registry import BoundScope
from mineworld_cognition.memory.scopes.stack import MemoryStack
from mineworld_cognition.memory.scopes.view import BaseView, GuardedView

type OnDefect = Literal["raise", "withhold"]


@dataclass(frozen=True)
class ScopeDefect:
    """A scope's contribution withheld from one recall: the error's class and message (no gist, no value)."""

    scope: str
    seat: EntityKey
    error: str


@dataclass(frozen=True)
class Recalled:
    section: MemorySection
    defects: tuple[ScopeDefect, ...]


def recall(
    view: SeatView, stack: MemoryStack, query: RecallQuery, *, on_defect: OnDefect
) -> Recalled:
    lines: list[SectionLine] = []
    headings: list[SectionHeading] = []
    speaker: str | None = None
    defects: list[ScopeDefect] = []
    seat = stack.seat
    for bound in stack.scopes:
        try:
            got = _checked(view, bound, query, seat)
        except ScopeError as error:
            if on_defect == "raise":
                raise
            defects.append(ScopeDefect(bound.id, seat, f"{type(error).__name__}: {error}"))
            continue
        if bound.id == "person":
            lines += got.lines
            speaker = got.speaker
        else:
            lines += got.lines
            headings += [
                SectionHeading(key=spec.key, header=spec.header) for spec in bound.sections
            ]
    section = MemorySection(lines=tuple(lines), speaker=speaker, sections=tuple(headings))
    return Recalled(section=section, defects=tuple(defects))


def accounted(view: SeatView, stack: MemoryStack) -> int:
    """The bytes the stack's scopes account for in `view` (each read as `recall` reads it): under `verify`, the
    seat checks this equals `view.content_size()` (R-S28f-c-1, F-S28c-16)."""
    total = 0
    for bound in stack.scopes:
        scoped: GuardedView = (
            BaseView(view, bound.id) if bound.base else GuardedView(view, bound.id)
        )
        total += bound.scope.account(scoped).total()
    return total


@dataclass(frozen=True)
class _Got:
    lines: tuple[SectionLine, ...]
    speaker: str | None


def _checked(view: SeatView, bound: BoundScope, query: RecallQuery, seat: EntityKey) -> _Got:
    scoped: GuardedView = BaseView(view, bound.id) if bound.base else GuardedView(view, bound.id)
    budgets = {spec.key: spec.budget for spec in bound.sections}
    try:
        returned: object = bound.scope.recall(scoped, query, budgets, bound.parameters)
    except ScopeError:
        raise
    except Exception as error:
        raise ScopeRecallFailed(bound.id, seat, type(error).__name__) from error
    lines = _lines(returned, bound.id, seat)
    speaker = _speaker(view, query) if bound.id == "person" else None
    used: dict[str, tuple[int, int]] = {}
    for line in lines:
        spec = next((s for s in bound.sections if s.key == line.level), None)
        if spec is None:
            raise ScopeSectionOverBudget(bound.id, seat, line.level, "section", 0, 0)
        size = len(line.render().encode("utf-8"))
        if size > LINE_BYTES:
            raise ScopeSectionOverBudget(bound.id, seat, spec.key, "line", size, LINE_BYTES)
        count, total = used.get(spec.key, (0, len(_header(spec.key, spec.header, speaker)) + 1))
        count, total = count + 1, total + size + 1
        if count > spec.budget.lines:
            raise ScopeSectionOverBudget(
                bound.id, seat, spec.key, "lines", count, spec.budget.lines
            )
        if total > spec.budget.bytes:
            raise ScopeSectionOverBudget(
                bound.id, seat, spec.key, "bytes", total, spec.budget.bytes
            )
        used[spec.key] = (count, total)
    ends = {
        event_key(end) for line in lines for r in line.citation.ranges for end in (r.first, r.last)
    }
    citable = view.citable(ends, units=bound.base)
    for line in lines:
        for cited in line.citation.ranges:
            for end in (cited.first, cited.last):
                if event_key(end) not in citable:
                    raise AccessDenied(bound.id, seat, end)
    said = any(line.level == "with" for line in lines)
    return _Got(lines=lines, speaker=speaker if said else None)


def _lines(returned: object, scope: str, seat: EntityKey) -> tuple[SectionLine, ...]:
    """A scope's result, which is not trusted to be what its signature says."""
    if not isinstance(returned, tuple) or not all(
        isinstance(line, SectionLine)
        for line in returned  # pyright: ignore[reportUnknownVariableType]
    ):
        raise ScopeRecallFailed(scope, seat, "its recall returned something other than lines")
    return returned  # pyright: ignore[reportUnknownVariableType]


def _speaker(view: SeatView, query: RecallQuery) -> str | None:
    return None if query.speaker is None else NameBook.load(view).person(query.speaker)


def _header(key: str, declared: str, speaker: str | None) -> bytes:
    """The header the section renders with: the person's `with` names the speaker (`retrieve.header`)."""
    if key == "with":
        return header("with", speaker).encode("utf-8")
    return declared.encode("utf-8")
