"""AFGb-1 (interface and registry), AFGb-9 (purity and no privilege): pr-fg-b §5.2, §5.3, §6.

The registry is exercised the two ways a policy reaches it: entry points (`from_entry_points`, and the
installed distribution's own group) and classes given directly (`PolicyRegistry({...})`).
"""

from __future__ import annotations

from importlib.metadata import EntryPoint

import pytest
from retention_fixtures import KeepsNothingNamed, NoDefault, WrongId

from mineworld_cognition.memory.retention.registry import (
    GROUP,
    DuplicatePolicy,
    InvalidParameters,
    PolicyLoadError,
    PolicyRegistry,
    SourceSpan,
    UnknownPolicy,
)
from mineworld_cognition.memory.retention.selection import (
    FallbackNotAllowed,
    MemoryPolicyChoice,
    resolve,
)

SOURCE = SourceSpan(file="worlds/x/decisions.yaml", line=12, column=7)


def _given() -> PolicyRegistry:
    return PolicyRegistry({"user-policy": KeepsNothingNamed})


def test_an_unknown_policy_is_refused_naming_it_and_every_installed_id() -> None:
    # AFGb-1 (c), first case.
    with pytest.raises(UnknownPolicy) as raised:
        _given().bind("forget-everything", {})
    assert raised.value.known == ("user-policy",)
    assert "forget-everything" in str(raised.value)
    assert "user-policy" in str(raised.value)


def test_a_class_whose_id_is_not_its_entry_name_is_refused() -> None:
    # AFGb-1 (c): the entry name and the class's own id must agree, or two names would mean one policy.
    with pytest.raises(PolicyLoadError) as raised:
        PolicyRegistry({"wrong": WrongId})
    assert (raised.value.name, raised.value.distribution) == ("wrong", "(given)")
    assert "something-else" in raised.value.why


def test_parameters_without_a_default_are_refused() -> None:
    # AFGb-1 (c): a world that writes no parameters must still get a valid policy.
    with pytest.raises(PolicyLoadError) as raised:
        PolicyRegistry({"no-default": NoDefault})
    assert "hours" in raised.value.why


def test_two_distributions_publishing_one_id_are_refused() -> None:
    # AFGb-1 (c): entry points as two installed packages would publish them.
    points = [
        EntryPoint("user-policy", "retention_fixtures:KeepsNothingNamed", GROUP),
        EntryPoint("user-policy", "retention_fixtures:KeepsNothingNamed", GROUP),
    ]
    with pytest.raises(DuplicatePolicy) as raised:
        PolicyRegistry.from_entry_points(points)
    assert raised.value.policy == "user-policy"
    assert len(raised.value.distributions) == 2


def test_an_entry_point_that_cannot_be_imported_is_refused_by_name() -> None:
    point = EntryPoint("absent", "retention_fixtures_that_do_not_exist:Policy", GROUP)
    with pytest.raises(PolicyLoadError) as raised:
        PolicyRegistry.from_entry_points([point])
    assert raised.value.name == "absent"
    assert "ModuleNotFoundError" in raised.value.why


@pytest.mark.parametrize(
    ("parameters", "field"),
    [({"hours": 721}, "hours"), ({"hours": 3, "days": 1}, "days"), ({"hours": "3"}, "hours")],
)
def test_invalid_parameters_are_refused_naming_policy_field_and_source(
    parameters: dict[str, int | str], field: str
) -> None:
    # AFGb-1 (d) on a user's policy: out of bounds, an unknown key, a string for an integer. The given
    # value is never repeated (errors carry locations and messages only). Its mutation lets `bind` ignore
    # unknown keys: the second case binds silently.
    with pytest.raises(InvalidParameters) as raised:
        _given().bind("user-policy", dict(parameters), source=SOURCE)
    error = raised.value
    assert error.policy == "user-policy"
    assert [e.location for e in error.errors] == [field]
    assert error.source == SOURCE
    assert "worlds/x/decisions.yaml:12:7" in str(error)
    assert "721" not in str(error)


def test_a_choice_resolves_to_its_bound_policy_and_a_fallback_is_refused_on_a_plain_one() -> None:
    # §6: the JSON form DL-a emits, written here literally (QFGb-1).
    choice = MemoryPolicyChoice.model_validate_json(
        '{"policy": "user-policy", "parameters": {"hours": 5}, '
        '"source": {"file": "decisions.yaml", "line": 3, "column": 5}}'
    )
    assert resolve(choice, _given()).policy.id == "user-policy"
    with_fallback = choice.model_copy(update={"fallback": "consolidate"})
    with pytest.raises(FallbackNotAllowed) as raised:
        resolve(with_fallback, _given())
    assert "decisions.yaml:3:5" in str(raised.value)
