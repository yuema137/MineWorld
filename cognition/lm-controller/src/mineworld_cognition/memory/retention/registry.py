"""`PolicyRegistry`: the installed retention policies, found through one entry-point group (pr-fg-b §5.3).

Every policy, bundled or a user's, is an entry point of the group `mineworld.forgetting` whose name is the
policy's `id` (FGb-4; standard metadata, `importlib.metadata`, no loader of our own). A loaded object is
checked before it is used: a class, with an `id` equal to its entry-point name, a strict `Parameters`
model whose every field has a default, and a `plan`. Anything else is refused by name, never skipped.

`bind` decodes a world's raw parameters into the policy's own `Parameters`, once, and refuses unknown keys
and invalid values naming the policy, the field and where the choice was written; it never repeats the
value it was given.
"""

from __future__ import annotations

import re
from collections.abc import Iterable, Mapping, Sequence
from dataclasses import dataclass
from importlib.metadata import EntryPoint, entry_points
from typing import Annotated

from mineworld_sdk.wire.ids import JsonValue
from pydantic import BaseModel, Field, ValidationError

from mineworld_cognition.backend.model import CognitionModel
from mineworld_cognition.memory.retention.policy import MemoryItem, RetentionBudget

GROUP = "mineworld.forgetting"
POLICY_ID = re.compile(r"[a-z][a-z0-9-]{0,47}")

type JsonObject = dict[str, JsonValue]
"""Raw JSON as transport only: decoded into the policy's own `Parameters` at `bind`."""


class SourceSpan(CognitionModel):
    """Where a world wrote the choice: the file, line and column the decision list's reader reports."""

    file: str
    line: Annotated[int, Field(ge=1)]
    column: Annotated[int, Field(ge=1)]

    def __str__(self) -> str:
        return f"{self.file}:{self.line}:{self.column}"


class PolicyError(ValueError):
    """A retention policy could not be loaded, found or bound."""


class PolicyLoadError(PolicyError):
    def __init__(self, name: str, distribution: str, why: str) -> None:
        super().__init__(f"retention policy {name!r} from {distribution} cannot be used: {why}")
        self.name = name
        self.distribution = distribution
        self.why = why


class DuplicatePolicy(PolicyError):
    def __init__(self, policy: str, distributions: Sequence[str]) -> None:
        super().__init__(
            f"retention policy {policy!r} is published by more than one distribution: "
            + ", ".join(distributions)
        )
        self.policy = policy
        self.distributions = tuple(distributions)


class UnknownPolicy(PolicyError):
    def __init__(self, policy: str, known: Sequence[str]) -> None:
        super().__init__(
            f"no retention policy {policy!r} is installed; installed: {', '.join(known) or 'none'}"
        )
        self.policy = policy
        self.known = tuple(known)


@dataclass(frozen=True)
class ParameterError:
    """One refused field: where it is and why, never the value given."""

    location: str
    message: str


class InvalidParameters(PolicyError):
    def __init__(
        self, policy: str, errors: Sequence[ParameterError], source: SourceSpan | None
    ) -> None:
        where = f" ({source})" if source is not None else ""
        listed = "; ".join(f"{error.location}: {error.message}" for error in errors)
        super().__init__(f"invalid parameters for retention policy {policy!r}{where}: {listed}")
        self.policy = policy
        self.errors = tuple(errors)
        self.source = source


@dataclass(frozen=True)
class _Checked:
    """A policy class that passed the checks: its parameters model and an instance."""

    id: str
    parameters: type[BaseModel]
    instance: object


class RegisteredPolicy:
    """A policy bound to its validated parameters: what the host runs."""

    def __init__(self, checked: _Checked, parameters: BaseModel) -> None:
        self._checked = checked
        self._parameters = parameters

    @property
    def id(self) -> str:
        return self._checked.id

    def plan(self, items: Sequence[MemoryItem], budget: RetentionBudget) -> object:
        """The policy's plan, unvalidated: the host validates it (a policy may be written without
        pyright, so its result is not trusted to be what its signature says)."""
        planner = getattr(self._checked.instance, "plan")  # noqa: B009 - checked callable at load
        result: object = planner(items, budget, self._parameters)
        return result


class PolicyRegistry:
    """Policies by id. `installed()` reads the entry-point group; tests and embedders may pass classes."""

    def __init__(self, policies: Mapping[str, object]) -> None:
        self._policies = {
            name: _check(name, candidate, "(given)") for name, candidate in policies.items()
        }

    @staticmethod
    def installed() -> PolicyRegistry:
        return PolicyRegistry.from_entry_points(entry_points(group=GROUP))

    @staticmethod
    def from_entry_points(points: Iterable[EntryPoint]) -> PolicyRegistry:
        """Every entry point loaded and checked; two distributions publishing one id are refused."""
        published: dict[str, list[EntryPoint]] = {}
        for point in points:
            published.setdefault(point.name, []).append(point)
        registry = PolicyRegistry({})
        for name, found in sorted(published.items()):
            if len(found) > 1:
                raise DuplicatePolicy(name, [_distribution(point) for point in found])
            (point,) = found
            try:
                loaded: object = point.load()
            except Exception as error:
                why = f"its module failed to import ({type(error).__name__}: {error})"
                raise PolicyLoadError(name, _distribution(point), why) from error
            registry._policies[name] = _check(name, loaded, _distribution(point))
        return registry

    def ids(self) -> tuple[str, ...]:
        return tuple(sorted(self._policies))

    def bind(
        self, policy: str, parameters: JsonObject, *, source: SourceSpan | None = None
    ) -> RegisteredPolicy:
        checked = self._policies.get(policy)
        if checked is None:
            raise UnknownPolicy(policy, self.ids())
        fields = checked.parameters.model_fields
        unknown = [
            ParameterError(key, "unknown parameter") for key in parameters if key not in fields
        ]
        if unknown:
            raise InvalidParameters(policy, unknown, source)
        try:
            decoded = checked.parameters.model_validate(parameters, strict=True)
        except ValidationError as error:
            errors = [
                ParameterError(".".join(str(part) for part in detail["loc"]), detail["msg"])
                for detail in error.errors(include_input=False, include_url=False)
            ]
            raise InvalidParameters(policy, errors, source) from None
        return RegisteredPolicy(checked, decoded)


def _distribution(point: EntryPoint) -> str:
    dist = point.dist
    return "(unknown distribution)" if dist is None else dist.name


def _check(name: str, candidate: object, distribution: str) -> _Checked:
    if not POLICY_ID.fullmatch(name):
        raise PolicyLoadError(name, distribution, "its name is not a policy id")
    if not isinstance(candidate, type):
        raise PolicyLoadError(name, distribution, "it is not a class")
    declared: object = getattr(candidate, "id", None)
    if declared != name:
        raise PolicyLoadError(name, distribution, f"its id is {declared!r}, not its entry name")
    parameters: object = getattr(candidate, "Parameters", None)
    if not (isinstance(parameters, type) and issubclass(parameters, BaseModel)):
        raise PolicyLoadError(name, distribution, "its Parameters is not a Pydantic model")
    missing = [field for field, info in parameters.model_fields.items() if info.is_required()]
    if missing:
        why = f"its parameters {', '.join(missing)} have no default"
        raise PolicyLoadError(name, distribution, why)
    if not callable(getattr(candidate, "plan", None)):
        raise PolicyLoadError(name, distribution, "it has no plan")
    try:
        instance: object = candidate()
    except Exception as error:
        why = f"it cannot be constructed without arguments ({type(error).__name__})"
        raise PolicyLoadError(name, distribution, why) from error
    return _Checked(name, parameters, instance)
