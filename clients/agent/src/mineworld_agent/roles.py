"""Role tags: the few tags whose meaning the framework fixes (`docs/CORE_CONCEPTS.md` §6.6, `ARC-105`).

A role tag says what a place is, such as a station's ticket office or a town's landmark. It is authored
content, written among a place's tags like any other, and it grants nothing: what an agent can do in a place
still comes only from the offers the server sends. The view lists a place's role tags ahead of its labels so an
agent finds the station by what it is, never by its key. The list below is held equal to the specification's
line by `tests/test_roles.py`; a new role tag is added there first.
"""

from __future__ import annotations

from collections.abc import Sequence
from typing import Final, Literal

ROLE_TAGS: Final = ("station", "stop", "platform", "ticket-office", "landmark")
"""The role tags of `docs/CORE_CONCEPTS.md` §6.6, in its order (ARC-105)."""

type RoleTag = Literal["station", "stop", "platform", "ticket-office", "landmark"]
"""One role tag: a code from the framework's vocabulary, never world text."""


def roles_of(tags: Sequence[str]) -> list[RoleTag]:
    """The role tags among `tags`: exact, case-sensitive matches only, each once, in `ROLE_TAGS` order."""
    present = set(tags)
    return [tag for tag in ROLE_TAGS if tag in present]
