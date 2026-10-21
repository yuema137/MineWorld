"""The named errors of `memory.scopes` (pr-s28c-memory-scopes.md §5.11; MODULE_SPEC §5.3).

Every message begins `memory.scopes:` and names the scope, the seat or the source position it is about; none
repeats a parameter's value or a line's text.
"""

from __future__ import annotations

from collections.abc import Sequence

from mineworld_sdk.wire.ids import EntityKey

from mineworld_cognition.memory.retention.registry import ParameterError, SourceSpan

PREFIX = "memory.scopes:"


def _at(source: SourceSpan | None) -> str:
    return f" ({source})" if source is not None else ""


class ScopeError(ValueError):
    """A memory scope could not be loaded, selected or run."""


class ScopeLoadError(ScopeError):
    def __init__(self, name: str, distribution: str, why: str) -> None:
        super().__init__(f"{PREFIX} scope {name!r} from {distribution} cannot be used: {why}")
        self.name = name
        self.distribution = distribution
        self.why = why


class DuplicateScope(ScopeError):
    def __init__(self, scope: str, distributions: Sequence[str]) -> None:
        super().__init__(
            f"{PREFIX} scope {scope!r} is published by more than one distribution: "
            + ", ".join(distributions)
        )
        self.scope = scope
        self.distributions = tuple(distributions)


class UnknownScope(ScopeError):
    def __init__(self, scope: str, installed: Sequence[str]) -> None:
        super().__init__(
            f"{PREFIX} no scope {scope!r} is installed; installed: {', '.join(installed) or 'none'}"
        )
        self.scope = scope
        self.installed = tuple(installed)


class ScopeParameterError(ScopeError):
    def __init__(
        self, scope: str, errors: Sequence[ParameterError], source: SourceSpan | None
    ) -> None:
        listed = "; ".join(f"{error.location}: {error.message}" for error in errors)
        super().__init__(f"{PREFIX} invalid parameters for scope {scope!r}{_at(source)}: {listed}")
        self.scope = scope
        self.errors = tuple(errors)
        self.source = source


class PersonScopeRequired(ScopeError):
    def __init__(self, source: SourceSpan | None) -> None:
        super().__init__(f"{PREFIX} a memory stack{_at(source)} must list the person scope first")
        self.source = source


class ScopeListedTwice(ScopeError):
    def __init__(self, scope: str, source: SourceSpan | None) -> None:
        super().__init__(f"{PREFIX} scope {scope!r}{_at(source)} is listed twice in one stack")
        self.scope = scope
        self.source = source


class ScopeNotSelectable(ScopeError):
    def __init__(self, scope: str, source: SourceSpan | None) -> None:
        super().__init__(
            f"{PREFIX} scope {scope!r}{_at(source)} is a base scope that no stack lists: its "
            "placement is cognition.toml's"
        )
        self.scope = scope
        self.source = source


class TotalSectionsOverCeiling(ScopeError):
    def __init__(self, total: int, ceiling: int, source: SourceSpan | None) -> None:
        super().__init__(
            f"{PREFIX} the memory stack{_at(source)} declares {total} bytes of sections, over the "
            f"{ceiling}-byte ceiling: a scope's section takes its bytes from the person scope's"
        )
        self.total = total
        self.ceiling = ceiling
        self.source = source


class ScopeSectionOverBudget(ScopeError):
    def __init__(
        self, scope: str, seat: EntityKey, section: str, what: str, size: int, limit: int
    ) -> None:
        super().__init__(
            f"{PREFIX} scope {scope!r} for seat {seat}: section {section!r} {what} {size} over its "
            f"limit {limit}"
            if what != "section"
            else f"{PREFIX} scope {scope!r} for seat {seat}: a line keyed to section {section!r}, "
            "which the scope does not declare"
        )
        self.scope = scope
        self.seat = seat
        self.section = section
        self.what = what
        self.bytes = size
        self.limit = limit


class AccessDenied(ScopeError):
    def __init__(self, scope: str, seat: EntityKey, event: str) -> None:
        super().__init__(
            f"{PREFIX} scope {scope!r} cited or read event {event}, which seat {seat} may not cite"
        )
        self.scope = scope
        self.seat = seat
        self.event = event


class ScopeRecallFailed(ScopeError):
    def __init__(self, scope: str, seat: EntityKey, error: str) -> None:
        super().__init__(f"{PREFIX} scope {scope!r} failed to recall for seat {seat}: {error}")
        self.scope = scope
        self.seat = seat
        self.error = error
