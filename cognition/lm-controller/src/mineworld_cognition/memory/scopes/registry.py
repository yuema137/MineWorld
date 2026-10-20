"""`ScopeRegistry`: the installed memory scopes, found through one entry-point group (pr-s28c §5.3).

Every scope is an entry point of the group `mineworld.memory_scopes` whose name is the scope's `id`, read with
`importlib.metadata` exactly as `PolicyRegistry` reads `mineworld.forgetting` (`ARC-85`). A loaded object is
checked before it is used; anything malformed is refused by name, never skipped.

The **base scopes** `person` and `world` (QS28c-1) are named once, in `BASE_SCOPES`, and only the
`mineworld-cognition` distribution may publish them: they own the home's base tables and read the seat's
own view.
"""

from __future__ import annotations

from collections.abc import Iterable, Mapping
from dataclasses import dataclass
from importlib.metadata import EntryPoint, entry_points
from typing import cast

from pydantic import BaseModel, ValidationError

from mineworld_cognition.memory.retention.registry import (
    POLICY_ID,
    JsonObject,
    ParameterError,
    SourceSpan,
)
from mineworld_cognition.memory.scopes.errors import (
    DuplicateScope,
    ScopeLoadError,
    ScopeParameterError,
    UnknownScope,
)
from mineworld_cognition.memory.scopes.interface import (
    PERSON_SECTIONS,
    MemoryScope,
    ScopeParameters,
    SectionSpec,
    TableSpec,
)

GROUP = "mineworld.memory_scopes"
BASE_SCOPES = ("person", "world")
"""The home's base scopes: implemented by the memory package, given the seat's own view (QS28c-1)."""
BASE_DISTRIBUTION = "mineworld-cognition"
"""The only distribution that may publish a base scope."""
GIVEN = "(given)"
"""The distribution of a scope passed to `ScopeRegistry` directly (tests, embedders)."""

type AnyScope = MemoryScope[ScopeParameters]


@dataclass(frozen=True)
class _Checked:
    id: str
    distribution: str
    instance: AnyScope
    parameters: type[ScopeParameters]


@dataclass(frozen=True)
class BoundScope:
    """A scope bound to its decoded parameters: what the recall host runs."""

    id: str
    scope: AnyScope
    parameters: ScopeParameters
    sections: tuple[SectionSpec, ...]
    base: bool
    """A base scope (`BASE_SCOPES`): it reads the seat's own view."""


class ScopeRegistry:
    """Scopes by id. `installed()` reads the entry-point group; tests and embedders may pass classes, whose
    distribution is `(given)`; `base` names the given ones that are base scopes."""

    def __init__(self, scopes: Mapping[str, object], *, base: Iterable[str] = ()) -> None:
        trusted = set(base)
        self._scopes = {
            name: _check(name, candidate, BASE_DISTRIBUTION if name in trusted else GIVEN)
            for name, candidate in scopes.items()
        }

    @staticmethod
    def installed() -> ScopeRegistry:
        return ScopeRegistry.from_entry_points(entry_points(group=GROUP))

    @staticmethod
    def from_entry_points(points: Iterable[EntryPoint]) -> ScopeRegistry:
        """Every entry point loaded and checked; two distributions publishing one id are refused."""
        published: dict[str, list[EntryPoint]] = {}
        for point in points:
            published.setdefault(point.name, []).append(point)
        registry = ScopeRegistry({})
        for name, found in sorted(published.items()):
            if len(found) > 1:
                raise DuplicateScope(name, [_distribution(point) for point in found])
            (point,) = found
            distribution = _distribution(point)
            try:
                loaded: object = point.load()
            except Exception as error:
                why = f"its module failed to import ({type(error).__name__}: {error})"
                raise ScopeLoadError(name, distribution, why) from error
            registry._scopes[name] = _check(name, loaded, distribution)
        return registry

    def ids(self) -> tuple[str, ...]:
        return tuple(sorted(self._scopes))

    def distribution(self, scope: str) -> str:
        return self._scopes[scope].distribution

    def parameters_model(self, scope: str) -> type[ScopeParameters]:
        return self._scopes[scope].parameters

    def bind(
        self, scope: str, parameters: JsonObject, *, source: SourceSpan | None = None
    ) -> BoundScope:
        checked = self._scopes.get(scope)
        if checked is None:
            raise UnknownScope(scope, self.ids())
        fields = checked.parameters.model_fields
        unknown = [
            ParameterError(key, "unknown parameter") for key in parameters if key not in fields
        ]
        if unknown:
            raise ScopeParameterError(scope, unknown, source)
        try:
            decoded = checked.parameters.model_validate(parameters, strict=True)
        except ValidationError as error:
            errors = [
                ParameterError(".".join(str(part) for part in detail["loc"]), detail["msg"])
                for detail in error.errors(include_input=False, include_url=False)
            ]
            raise ScopeParameterError(scope, errors, source) from None
        sections = checked.instance.sections(decoded)
        return BoundScope(
            id=scope,
            scope=checked.instance,
            parameters=decoded,
            sections=tuple(sections),
            base=scope in BASE_SCOPES,
        )


def _distribution(point: EntryPoint) -> str:
    dist = point.dist
    return "(unknown distribution)" if dist is None else dist.name


def _check(name: str, candidate: object, distribution: str) -> _Checked:
    if not POLICY_ID.fullmatch(name):
        raise ScopeLoadError(name, distribution, "its name is not a scope id")
    if name in PERSON_SECTIONS:
        raise ScopeLoadError(name, distribution, "its name is a person section's key")
    if name in BASE_SCOPES and distribution != BASE_DISTRIBUTION:
        raise ScopeLoadError(name, distribution, "the base scopes are the memory package's own")
    if not isinstance(candidate, type):
        raise ScopeLoadError(name, distribution, "it is not a class")
    declared: object = getattr(candidate, "id", None)
    if declared != name:
        raise ScopeLoadError(name, distribution, f"its id is {declared!r}, not its entry name")
    parameters: object = getattr(candidate, "Parameters", None)
    if not (isinstance(parameters, type) and issubclass(parameters, ScopeParameters)):
        raise ScopeLoadError(name, distribution, "its Parameters is not a ScopeParameters model")
    model = cast(type[BaseModel], parameters)
    missing = [field for field, info in model.model_fields.items() if info.is_required()]
    if missing:
        why = f"its parameters {', '.join(missing)} have no default"
        raise ScopeLoadError(name, distribution, why)
    _check_tables(name, getattr(candidate, "tables", None), distribution)
    for member in ("sections", "index", "recall", "account"):
        if not callable(getattr(candidate, member, None)):
            raise ScopeLoadError(name, distribution, f"it has no {member}")
    try:
        instance: object = candidate()
    except Exception as error:
        why = f"it cannot be constructed without arguments ({type(error).__name__})"
        raise ScopeLoadError(name, distribution, why) from error
    return _Checked(name, distribution, cast(AnyScope, instance), parameters)


def _check_tables(name: str, tables: object, distribution: str) -> None:
    """`tables` is a tuple of `TableSpec`, each named `<id>_…`, the names unique; a base scope declares none
    (it owns the home's base tables)."""
    if not isinstance(tables, tuple):
        raise ScopeLoadError(name, distribution, "its tables is not a tuple of TableSpec")
    specs = cast(tuple[object, ...], tables)
    if not all(isinstance(spec, TableSpec) for spec in specs):
        raise ScopeLoadError(name, distribution, "its tables is not a tuple of TableSpec")
    names = [cast(TableSpec, spec).name for spec in specs]
    if name in BASE_SCOPES and names:
        raise ScopeLoadError(name, distribution, "a base scope declares no table of its own")
    for table in names:
        if not table.startswith(f"{name}_") or table == f"{name}_":
            raise ScopeLoadError(name, distribution, f"its table {table!r} is not named {name}_…")
    if len(set(names)) != len(names):
        raise ScopeLoadError(name, distribution, "its tables are not uniquely named")
