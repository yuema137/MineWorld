"""A handle's state between processes: `<state_dir>/<seat>.json` (step-27 §5.2).

Only when the Player was configured with a `state_dir`. The file holds what lets a restarted program take
its seat back within the server's hold — the world instance, the newest welcome's `resume` secret, and the
perceived cursor — and nothing else: no invite, no observation, no fact. It is written atomically (a
temporary file in the same directory, then `os.replace`), readable only by its owner on POSIX (mode 0600;
on Windows the user profile's ACL governs the directory the user chose), rewritten after every welcome
(each carries a fresh `resume`) and as the cursor advances, and deleted on `leave`.

A file whose `instance` is not the server's is ignored: its secret and cursor belong to another world.
"""

from __future__ import annotations

import contextlib
import os
import tempfile
from pathlib import Path

from mineworld_sdk.wire.ids import EntityKey, EventId, ResumeSecret, WorldInstanceId
from pydantic import BaseModel, ConfigDict, Field, ValidationError

__all__ = ["SeatState", "StateDir"]


class SeatState(BaseModel):
    """What the file holds. The secret never appears in a `repr`."""

    model_config = ConfigDict(strict=True, frozen=True, extra="forbid")

    instance: str
    resume: str | None = Field(default=None, repr=False)
    cursor: str | None = None

    @property
    def world(self) -> WorldInstanceId:
        return WorldInstanceId(self.instance)

    @property
    def secret(self) -> ResumeSecret | None:
        return None if self.resume is None else ResumeSecret(self.resume)

    @property
    def since(self) -> EventId | None:
        return None if self.cursor is None else EventId(self.cursor)


class StateDir:
    """The directory of seat files, one per seat."""

    __slots__ = ("_root",)

    def __init__(self, root: Path) -> None:
        self._root = root

    def path(self, seat: EntityKey) -> Path:
        return self._root / f"{seat}.json"

    def read(self, seat: EntityKey, instance: WorldInstanceId) -> SeatState | None:
        """The seat's state, when there is one for this world instance; otherwise `None`. A file that
        does not parse is treated as absent (and overwritten at the next welcome)."""
        try:
            text = self.path(seat).read_text(encoding="utf-8")
        except FileNotFoundError:
            return None
        try:
            state = SeatState.model_validate_json(text)
        except ValidationError:
            return None
        return state if state.world == instance else None

    def write(self, seat: EntityKey, state: SeatState) -> None:
        """Replaces the seat's file atomically, mode 0600 on POSIX."""
        self._root.mkdir(parents=True, exist_ok=True)
        descriptor, temporary = tempfile.mkstemp(prefix=f".{seat}.", suffix=".tmp", dir=self._root)
        try:
            with os.fdopen(descriptor, "w", encoding="utf-8") as file:
                file.write(state.model_dump_json())
            if os.name == "posix":
                os.chmod(temporary, 0o600)
            os.replace(temporary, self.path(seat))
        except BaseException:
            with contextlib.suppress(FileNotFoundError):
                os.unlink(temporary)
            raise

    def delete(self, seat: EntityKey) -> None:
        with contextlib.suppress(FileNotFoundError):
            self.path(seat).unlink()

    def __repr__(self) -> str:
        return f"StateDir({str(self._root)!r})"
