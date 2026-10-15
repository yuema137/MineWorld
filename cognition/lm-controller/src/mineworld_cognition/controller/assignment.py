"""Reads `mineworld decisions --json` (format 2) and selects the `lm` seats (pr-s10-p6 §5.9, C3).

The file is the only record of the assignment this process uses; no YAML is parsed here. Models are strict
(`CognitionModel`), so an unknown key is refused. An unknown `format` (format 1 included: it predates FG-b2's
`store_bound_mib`) is refused by name, and every error names the file, never a value from it (I-16).
A seat's `memory.store_bound_mib` travels in its `MemoryPolicyChoice` to `resolve`, which binds it as the
retention host's store bound (D-P6b-39).
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Literal, cast

from mineworld_sdk.wire.ids import EntityKey
from pydantic import ValidationError

from mineworld_cognition.backend.model import CognitionModel
from mineworld_cognition.errors import ConfigError
from mineworld_cognition.memory.retention.selection import MemoryPolicyChoice

LM = "lm"
"""The model id that means the LM controller (DL-a `external "lm"`)."""

FORMAT = 2
"""The `decisions --json` version this reader reads (MODULE_SPEC `decisions`; FG-b2 added `store_bound_mib`)."""


class SeatModel(CognitionModel):
    id: str
    assigned: str | None = None
    chain: list[str] | None = None


class SeatEntry(CognitionModel):
    seat: str
    model: SeatModel
    memory: MemoryPolicyChoice | None = None


class ResolvedDecisions(CognitionModel):
    format: Literal[2]
    world: str
    digest: str
    seats: list[SeatEntry]


class LmSeat(CognitionModel):
    """One seat this process joins: its key and its memory choice (`None` means DL-a's default)."""

    key: EntityKey
    memory: MemoryPolicyChoice | None


class Assignment(CognitionModel):
    """The `lm` seats of one decision file, in roster order, for one world."""

    world: str
    seats: list[LmSeat]


def read_assignment(path: Path, narrowed: list[str] | None) -> Assignment:
    """Reads the file, selects every seat whose resolved model is `lm`, and narrows it when asked.

    Refused by name: an unreadable file, an unknown `format`, a malformed document, a narrowed key that is
    not an `lm` seat of the file, and a file with no `lm` seat.
    """
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as error:
        raise ConfigError(
            f"{path}: cannot read the decision JSON ({type(error).__name__})"
        ) from None
    try:
        raw: object = json.loads(text)
    except json.JSONDecodeError:
        raise ConfigError(f"{path}: not a JSON document") from None
    if not isinstance(raw, dict) or cast(dict[str, object], raw).get("format") != FORMAT:
        raise ConfigError(
            f"{path}: unknown decision JSON format; this reader reads format {FORMAT}"
        )
    try:
        document = ResolvedDecisions.model_validate(raw)
    except ValidationError as error:
        locations = "; ".join(
            ".".join(str(part) for part in item["loc"]) + ": " + item["msg"]
            for item in error.errors()
        )
        raise ConfigError(f"{path}: not a decision JSON document ({locations})") from None
    lm_seats = [
        LmSeat(key=EntityKey(entry.seat), memory=entry.memory)
        for entry in document.seats
        if entry.model.assigned == LM
    ]
    if narrowed is not None:
        known = {str(seat.key) for seat in lm_seats}
        for key in narrowed:
            if key not in known:
                raise ConfigError(f"seats: an entry is not an lm seat in {path}")
        lm_seats = [seat for seat in lm_seats if str(seat.key) in narrowed]
    if not lm_seats:
        raise ConfigError(f"no lm seat in {path}")
    return Assignment(world=document.world, seats=lm_seats)
