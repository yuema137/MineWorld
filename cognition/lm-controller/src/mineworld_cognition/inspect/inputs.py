"""The inputs the home does not store (pr-s28e F-S28e-5, §5.8), read from the files the caller names.

- `--perceived FILE`: `mineworld perceived … --json`'s output for the seat, one `PerceivedEvent` per line; read
  into the set of its event ids. The lines name no person, so a file is refused when it lacks a record the seat
  holds: every held record was perceived, so another person's stream is caught (D-e-4).
- `--decisions FILE`: `mineworld decisions --json` (format 2); the seat's `memory` choice gives its bound.
- `--config FILE`: `cognition.toml`; the seat's `recall_episodes` (its override, else `[controller]`).

Without a file the default is used, and every report says which values are defaults.
"""

from __future__ import annotations

from collections.abc import Set as AbstractSet
from dataclasses import dataclass
from pathlib import Path

from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityKey, EventId
from pydantic import ValidationError

from mineworld_cognition.config import controller_for, load
from mineworld_cognition.controller.assignment import FORMAT, ResolvedDecisions
from mineworld_cognition.controller.memory_view import EPISODES_MAX
from mineworld_cognition.errors import ConfigError
from mineworld_cognition.memory.retention.bound import FLOOR_MIB
from mineworld_cognition.memory.retention.selection import MemoryPolicyChoice

PREFIX = "inspect:"


class InspectRefusal(ValueError):
    """An input the inspector cannot use; the message names the file or the value's role, never its content."""

    def __init__(self, reason: str) -> None:
        super().__init__(f"{PREFIX} {reason}")
        self.reason = reason


@dataclass(frozen=True)
class Inputs:
    """One seat's inputs. `None` means the default, which the report labels."""

    perceived: AbstractSet[EventId] | None = None
    choice: MemoryPolicyChoice | None = None
    """The seat's memory choice from `--decisions`."""
    recall_episodes: int | None = None
    """The seat's `recall_episodes` from `--config`."""

    @property
    def bound_mib(self) -> int:
        if self.choice is None or self.choice.store_bound_mib is None:
            return FLOOR_MIB
        return self.choice.store_bound_mib

    @property
    def episodes(self) -> int:
        return EPISODES_MAX if self.recall_episodes is None else self.recall_episodes


def read_perceived(path: Path) -> frozenset[EventId]:
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as error:
        raise InspectRefusal(
            f"{path}: cannot read the perceived file ({type(error).__name__})"
        ) from None
    ids: set[EventId] = set()
    for number, line in enumerate(text.splitlines(), start=1):
        if not line:
            continue
        try:
            ids.add(PerceivedEvent.model_validate_json(line).id)
        except ValidationError:
            raise InspectRefusal(
                f"{path}: line {number} is not a perceived fact (`mineworld perceived --json`)"
            ) from None
    return frozenset(ids)


def check_perceived(perceived: AbstractSet[EventId], held: AbstractSet[EventId], seat: str) -> None:
    """A seat's perceived file holds every record the seat holds (D-e-4)."""
    missing = sorted(held - perceived, key=int)
    if missing:
        raise InspectRefusal(
            f"the perceived file is not seat {seat!r}'s: it lacks {len(missing)} records the seat holds "
            f"(first #{missing[0]})"
        )


def read_choice(path: Path, seat: EntityKey) -> MemoryPolicyChoice | None:
    """The seat's memory choice in a decision JSON; `None` when the seat's entry has none (the default)."""
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as error:
        raise InspectRefusal(
            f"{path}: cannot read the decision JSON ({type(error).__name__})"
        ) from None
    try:
        document = ResolvedDecisions.model_validate_json(text)
    except ValidationError:
        raise InspectRefusal(
            f"{path}: not a decision JSON document of format {FORMAT} (`mineworld decisions --json`)"
        ) from None
    for entry in document.seats:
        if entry.seat == seat:
            return entry.memory
    raise InspectRefusal(f"{path}: the decision JSON has no seat {seat!r}")


def read_recall_episodes(path: Path, seat: EntityKey) -> int:
    try:
        config = load(path)
    except ConfigError as error:
        raise InspectRefusal(str(error)) from None
    return controller_for(config, seat).recall_episodes


def read_inputs(
    seat: EntityKey | None,
    *,
    perceived: Path | None = None,
    decisions: Path | None = None,
    config: Path | None = None,
) -> Inputs:
    if seat is None:
        if decisions is not None or perceived is not None or config is not None:
            raise InspectRefusal(
                "--perceived, --decisions and --config describe one seat: pass --seat"
            )
        return Inputs()
    return Inputs(
        perceived=None if perceived is None else read_perceived(perceived),
        choice=None if decisions is None else read_choice(decisions, seat),
        recall_episodes=None if config is None else read_recall_episodes(config, seat),
    )
