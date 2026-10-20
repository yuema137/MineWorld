"""AC-c-2: the scope registry finds the installed scopes and refuses malformed ones by name (pr-s28c §5.3)."""

from __future__ import annotations

from importlib.metadata import distribution

import pytest
from scope_fixtures import Notes, point

from mineworld_cognition.memory.retention.registry import SourceSpan
from mineworld_cognition.memory.scopes.errors import (
    DuplicateScope,
    ScopeLoadError,
    ScopeParameterError,
    UnknownScope,
)
from mineworld_cognition.memory.scopes.person import PersonScope
from mineworld_cognition.memory.scopes.registry import GROUP, ScopeRegistry
from mineworld_cognition.memory.scopes.world import WorldScope


def test_the_installed_scopes_are_bundled_by_this_distribution() -> None:
    registry = ScopeRegistry.installed()
    assert registry.ids() == ("item", "person", "place", "world")
    assert {registry.distribution(scope) for scope in registry.ids()} == {"mineworld-cognition"}
    declared = {
        entry.name: entry.value
        for entry in distribution("mineworld-cognition").entry_points
        if entry.group == GROUP
    }
    assert declared == {
        "person": "mineworld_cognition.memory.scopes.person:PersonScope",
        "world": "mineworld_cognition.memory.scopes.world:WorldScope",
        "place": "mineworld_cognition.memory.scopes.anchored:PlaceScope",
        "item": "mineworld_cognition.memory.scopes.anchored:ItemScope",
    }


def test_the_person_scope_declares_todays_five_sections() -> None:
    bound = ScopeRegistry.installed().bind("person", {})
    assert bound.base
    assert [
        (spec.key, spec.header, spec.budget.lines, spec.budget.bytes) for spec in bound.sections
    ] == [
        ("with", "What {name} and I said to each other before:", 6, 1000),
        ("L3", "People I know:", 16, 1600),
        ("L2", "Recent weeks:", 4, 800),
        ("L1", "Episodes I remember:", 8, 1600),
        ("L0", "The last day:", 48, 1000),
    ]
    assert sum(spec.budget.bytes for spec in bound.sections) == 6000
    assert ScopeRegistry.installed().bind("world", {}).sections == ()


@pytest.mark.parametrize(
    ("name", "target", "why"),
    [
        ("not-a-class", "not_a_class", "it is not a class"),
        ("notes", "WrongId", "its id is 'something-else', not its entry name"),
        ("undefaulted", "Undefaulted", "its parameters needed have no default"),
        ("plain", "NotScopeParameters", "its Parameters is not a ScopeParameters model"),
        ("foreign", "ForeignTable", "its table 'notes_index' is not named foreign_…"),
        ("norecall", "NoRecall", "it has no recall"),
        ("needs-arguments", "NeedsArguments", "it cannot be constructed without arguments"),
        ("absent", "DoesNotExist", "its module failed to import"),
        ("with", "Notes", "its name is a person section's key"),
        ("Notes", "Notes", "its name is not a scope id"),
        ("person", "Notes", "the base scopes are the memory package's own"),
        ("world", "Notes", "the base scopes are the memory package's own"),
        # AC-d-6 (pr-s28d §5.5): the hosted tables' load checks.
        ("nokey", "NoKey", "its table 'nokey_seen' has no key column"),
        ("counttext", "CountOnText", "its table 'counttext_seen' column 'n' counts"),
        ("seatcol", "SeatColumn", "its table 'seatcol_seen' has a column whose name"),
        ("capmissing", "CapMissing", "its table 'capmissing_seen' caps by 'absent'"),
        ("capnoevict", "CapNoEvict", "its table 'capnoevict_seen' has a cap but no evict"),
        ("evictmissing", "EvictMissing", "its table 'evictmissing_seen' evicts by a column"),
        ("eventkeyed", "EventKeyed", "its table 'eventkeyed_seen' is not keyed by seat"),
    ],
)
def test_a_malformed_scope_is_refused_naming_its_distribution(
    name: str, target: str, why: str
) -> None:
    with pytest.raises(ScopeLoadError) as raised:
        ScopeRegistry.from_entry_points([point(name, target)])
    assert raised.value.name == name
    assert raised.value.distribution == "(unknown distribution)"
    assert raised.value.why.startswith(why), raised.value.why


def test_two_distributions_publishing_one_id_are_refused() -> None:
    with pytest.raises(DuplicateScope) as raised:
        ScopeRegistry.from_entry_points([point("notes", "Notes"), point("notes", "Notes")])
    assert raised.value.scope == "notes"
    assert len(raised.value.distributions) == 2


def test_a_given_base_scope_must_be_trusted_by_name() -> None:
    with pytest.raises(ScopeLoadError):
        ScopeRegistry({"person": PersonScope})
    registry = ScopeRegistry(
        {"person": PersonScope, "world": WorldScope, "notes": Notes}, base=("person", "world")
    )
    assert registry.ids() == ("notes", "person", "world")
    assert registry.bind("person", {}).base and not registry.bind("notes", {}).base


def test_binding_refuses_an_unknown_scope_and_bad_parameters_without_echoing_them() -> None:
    registry = ScopeRegistry({"notes": Notes})
    with pytest.raises(UnknownScope) as unknown:
        registry.bind("place", {})
    assert unknown.value.installed == ("notes",)
    assert "notes" in str(unknown.value)
    source = SourceSpan(file="decisions.yaml", line=4, column=9)
    with pytest.raises(ScopeParameterError) as extra:
        registry.bind("notes", {"weight": 1}, source=source)
    assert [(e.location, e.message) for e in extra.value.errors] == [
        ("weight", "unknown parameter")
    ]
    with pytest.raises(ScopeParameterError) as invalid:
        registry.bind("notes", {"secret": 42, "section_lines": "s3cr3t-0042"}, source=source)
    assert {e.location for e in invalid.value.errors} == {"secret", "section_lines"}
    assert invalid.value.source == source
    assert "decisions.yaml:4:9" in str(invalid.value)
    assert "s3cr3t-0042" not in str(invalid.value)
    assert "42" not in "".join(e.message for e in invalid.value.errors)
