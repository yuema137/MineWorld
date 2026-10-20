"""AC-c-3, AC-c-7: a seat's memory stack is resolved when it starts, or refused by name (pr-s28c §5.5)."""

from __future__ import annotations

from pathlib import Path

import pytest
from binary import REPOSITORY, mineworld
from mineworld_sdk.wire.ids import EntityKey
from scope_fixtures import Notes

from mineworld_cognition.controller.assignment import read_assignment
from mineworld_cognition.memory.retention.registry import PolicyRegistry, SourceSpan
from mineworld_cognition.memory.retention.selection import (
    BoundBelowDefault,
    MemoryPolicyChoice,
    resolve,
)
from mineworld_cognition.memory.scopes.errors import (
    PersonScopeRequired,
    ScopeListedTwice,
    ScopeNotSelectable,
    ScopeParameterError,
    TotalSectionsOverCeiling,
    UnknownScope,
)
from mineworld_cognition.memory.scopes.person import PersonScope
from mineworld_cognition.memory.scopes.registry import ScopeRegistry
from mineworld_cognition.memory.scopes.stack import MemoryScopeChoice, resolve_stack
from mineworld_cognition.memory.scopes.world import WorldScope

ALICE = EntityKey("alice")
ENTRY = SourceSpan(file="decisions.yaml", line=2, column=3)
REGISTRY = ScopeRegistry(
    {"person": PersonScope, "world": WorldScope, "notes": Notes}, base=("person", "world")
)


def _at(line: int) -> SourceSpan:
    return SourceSpan(file="decisions.yaml", line=line, column=9)


def _choice(*scopes: MemoryScopeChoice) -> MemoryPolicyChoice:
    return MemoryPolicyChoice(policy="consolidate", scopes=scopes, source=ENTRY)


def _scope(scope: str, line: int, **parameters: object) -> MemoryScopeChoice:
    return MemoryScopeChoice.model_validate(
        {"id": scope, "parameters": parameters, "source": _at(line)}
    )


def test_no_stack_and_an_explicit_person_resolve_to_the_person_scope_alone() -> None:
    for choice in (
        MemoryPolicyChoice(policy="consolidate"),
        _choice(_scope("person", 3)),
    ):
        stack = resolve_stack(choice, ScopeRegistry.installed(), seat=ALICE)
        assert [scope.id for scope in stack.scopes] == ["person"]
        assert stack.section_bytes() == 6000


def test_a_stack_takes_its_section_bytes_from_the_persons() -> None:
    stack = resolve_stack(
        _choice(
            _scope("person", 3, sections={"L0": {"lines": 48, "bytes": 600}}),
            _scope("notes", 4, section_lines=4, section_bytes=400),
        ),
        REGISTRY,
        seat=ALICE,
    )
    assert [scope.id for scope in stack.scopes] == ["person", "notes"]
    assert stack.section_bytes() == 6000  # exactly the ceiling is allowed


def test_world_is_not_selectable() -> None:
    with pytest.raises(ScopeNotSelectable) as raised:
        resolve_stack(_choice(_scope("person", 3), _scope("world", 4)), REGISTRY, seat=ALICE)
    assert (raised.value.scope, raised.value.source) == ("world", _at(4))


@pytest.mark.parametrize(
    "scopes",
    [(), (MemoryScopeChoice.model_validate({"id": "notes", "source": _at(3)}),)],
)
def test_the_person_scope_is_required_first(scopes: tuple[MemoryScopeChoice, ...]) -> None:
    with pytest.raises(PersonScopeRequired) as raised:
        resolve_stack(_choice(*scopes), REGISTRY, seat=ALICE)
    assert raised.value.source == (_at(3) if scopes else ENTRY)


def test_a_scope_listed_twice_is_refused_at_its_second_entry() -> None:
    with pytest.raises(ScopeListedTwice) as raised:
        resolve_stack(
            _choice(_scope("person", 3), _scope("notes", 4), _scope("notes", 5)),
            REGISTRY,
            seat=ALICE,
        )
    assert (raised.value.scope, raised.value.source) == ("notes", _at(5))


def test_an_unknown_scope_names_every_installed_one() -> None:
    with pytest.raises(UnknownScope) as raised:
        resolve_stack(_choice(_scope("person", 3), _scope("place", 4)), REGISTRY, seat=ALICE)
    assert raised.value.installed == ("notes", "person", "world")
    assert all(scope in str(raised.value) for scope in ("notes", "person", "world"))


def test_invalid_parameters_are_named_by_field_and_source_never_by_value() -> None:
    with pytest.raises(ScopeParameterError) as raised:
        resolve_stack(_choice(_scope("person", 3, l0_window_s="s3cr3t-0042")), REGISTRY, seat=ALICE)
    assert [error.location for error in raised.value.errors] == ["l0_window_s"]
    assert raised.value.source == _at(3)
    assert "decisions.yaml:3:9" in str(raised.value)
    assert "s3cr3t-0042" not in str(raised.value)


@pytest.mark.parametrize(
    ("sections", "field"),
    [
        ({"L3": {"lines": 17, "bytes": 1600}}, "L3.lines"),
        ({"L2": {"lines": 4, "bytes": 801}}, "L2.bytes"),
        ({"with": {"lines": 7, "bytes": 1000}}, "with.lines"),
    ],
)
def test_the_person_sections_can_only_shrink(sections: dict[str, object], field: str) -> None:
    with pytest.raises(ScopeParameterError) as raised:
        resolve_stack(_choice(_scope("person", 3, sections=sections)), REGISTRY, seat=ALICE)
    assert field in str(raised.value)


def test_sections_over_the_ceiling_are_refused_with_their_total() -> None:
    with pytest.raises(TotalSectionsOverCeiling) as raised:
        resolve_stack(
            _choice(_scope("person", 3), _scope("notes", 4, section_lines=1, section_bytes=1)),
            REGISTRY,
            seat=ALICE,
        )
    assert (raised.value.total, raised.value.source) == (6001, ENTRY)


def test_a_bound_below_the_default_names_the_seat_and_its_source() -> None:
    choice = MemoryPolicyChoice(policy="consolidate", store_bound_mib=19, source=ENTRY)
    with pytest.raises(BoundBelowDefault) as raised:
        resolve(choice, PolicyRegistry.installed(), seat=ALICE)
    assert (raised.value.seat, raised.value.given_mib, raised.value.source) == (ALICE, 19, ENTRY)
    assert str(raised.value).startswith("memory.retention:")
    assert "alice" in str(raised.value) and "decisions.yaml:2:3" in str(raised.value)


@pytest.mark.real_binary
def test_a_stack_round_trips_through_the_real_reader_with_its_positions(tmp_path: Path) -> None:
    # AC-c-7 and AC-c-3's end-to-end case: YAML → `mineworld decisions --json` (format 3) → the assignment →
    # `resolve_stack`, a planted error named at the line and column of the scope's YAML entry.
    decisions = tmp_path / "stack.decisions.yaml"
    decisions.write_text(
        "models:\n"
        "  lm: { fallback: rule }\n"
        "assign:\n"
        "  - { seat: alice, model: lm }\n"
        "memory:\n"
        "  default:\n"
        "    policy: consolidate\n"
        "    scopes:\n"
        "      - id: person\n"
        "        parameters: { l0_window_s: 43200 }\n"
        "      - { id: person }\n",
        encoding="utf-8",
    )
    output = mineworld(
        "decisions", REPOSITORY / "worlds" / "social-cafe", "--decisions", decisions, "--json"
    )
    path = tmp_path / "seats.json"
    path.write_text(output, encoding="utf-8")
    (alice,) = read_assignment(path, None).seats
    assert alice.memory is not None and alice.memory.scopes is not None
    assert [(scope.id, scope.source) for scope in alice.memory.scopes] == [
        ("person", SourceSpan(file=str(decisions), line=9, column=9)),
        ("person", SourceSpan(file=str(decisions), line=11, column=9)),
    ]
    with pytest.raises(ScopeListedTwice) as raised:
        resolve_stack(alice.memory, ScopeRegistry.installed(), seat=alice.key)
    assert raised.value.source == SourceSpan(file=str(decisions), line=11, column=9)
