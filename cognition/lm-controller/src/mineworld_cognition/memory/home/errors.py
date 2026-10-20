"""The memory home's named errors (pr-s28a-memory-home.md §5.5; step-28 §5.5). Each is a `ValueError`, carries its
fields, and starts its message with `memory.home:`. Nothing was changed when one is raised on opening."""

from __future__ import annotations

from collections.abc import Sequence
from typing import Literal

HomeField = Literal["world_instance", "self_entity"]
PREFIX = "memory.home:"


class NotAMemoryHome(ValueError):
    """The file is a SQLite database with tables, but neither a home nor an older memory store (a save); or, for
    a read-only open, no file exists at the path (nothing is created)."""

    def __init__(self, path: str, *, missing: bool = False) -> None:
        what = "does not exist" if missing else "holds tables but no memory home"
        super().__init__(f"{PREFIX} {path} {what}")
        self.path = path
        self.missing = missing


class UnsupportedStoreSchema(ValueError):
    """The file is memory of another schema: a P4/FG-b store (2, 3) or a newer home. It is never modified."""

    def __init__(self, path: str, found: str, wanted: int) -> None:
        super().__init__(
            f"{PREFIX} {path} is memory of schema {found}, and this code reads schema {wanted}: rebuild it "
            "from the perceived stream: delete the file and start the seat with a null cursor "
            "(`perceived: { since: null }`)"
        )
        self.path = path
        self.found = found
        self.wanted = wanted


class HomeIdentityMismatch(ValueError):
    """The home is another world instance's, or the seat's stored self entity differs."""

    def __init__(self, field: HomeField, stored: str, given: str) -> None:
        super().__init__(
            f"{PREFIX} the home's {field} is {stored!r}, and the caller's is {given!r}"
        )
        self.field: HomeField = field
        self.stored = stored
        self.given = given


class HomeBusy(ValueError):
    """Another connection (another process) holds the home's lock: one writer per home (`DEP-61`)."""

    def __init__(self, path: str) -> None:
        super().__init__(f"{PREFIX} {path} is held by another process; a home has one writer")
        self.path = path


class HomeNeedsRecovery(ValueError):
    """A read-only open found a hot journal: a process was killed inside a transaction, and rolling it back is a
    write (pr-s28e F-S28e-2). The file is left as it is."""

    def __init__(self, path: str) -> None:
        super().__init__(
            f"{PREFIX} {path} was left inside a transaction by a stopped process; start the cognition process "
            "for this home once, or open it with `MemoryHome.open`, then inspect it again"
        )
        self.path = path


class SeatNotInHome(ValueError):
    def __init__(self, seat: str) -> None:
        super().__init__(f"{PREFIX} the home has no seat {seat!r}")
        self.seat = seat


class SeatAlreadyInHome(ValueError):
    """An import of a seat the target home already has (pr-s28e §5.7): nothing is changed."""

    def __init__(self, seat: str) -> None:
        super().__init__(f"{PREFIX} the home already has a seat {seat!r}")
        self.seat = seat


class NotASeatExport(ValueError):
    """An import's input is not a seat export; the reason names a line number, never a line's content."""

    def __init__(self, reason: str) -> None:
        super().__init__(f"{PREFIX} not a seat export: {reason}")
        self.reason = reason


class UnsupportedExportFormat(ValueError):
    def __init__(self, found: str, wanted: int) -> None:
        super().__init__(
            f"{PREFIX} the seat export is format {found}, and this code reads format {wanted}"
        )
        self.found = found
        self.wanted = wanted


class HolderInvariantBroken(ValueError):
    """I-S28-4 or a seat's meter failed under `verify`: a defect, never a fallback; the transaction rolls back."""

    def __init__(self, keys: Sequence[str] = (), seat: str | None = None) -> None:
        if seat is not None:
            detail = f"seat {seat!r}'s measured size differs from its dump"
        else:
            detail = f"fact records and holders disagree at {', '.join(keys[:8])}"
        super().__init__(f"{PREFIX} {detail}")
        self.keys = tuple(keys)
        self.seat = seat


class FactRecordMissing(ValueError):
    """A seat holds a key whose fact record is gone: I-S28-4 broken in a committed state."""

    def __init__(self, key: str) -> None:
        super().__init__(f"{PREFIX} the record of held fact {key} is missing")
        self.key = key
