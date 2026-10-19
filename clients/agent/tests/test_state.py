"""The state file (step-27 §5.2; pr-s27c §5.1 `state.py`): an atomic rewrite that leaves no temporary
behind, mode 0600 on POSIX, a file of another world instance ignored, a file that does not parse treated as
absent, and deletion on leave. The real rejoin across a server restart is `test_restart.py`'s."""

from __future__ import annotations

import os
import stat
from pathlib import Path

from mineworld_sdk.wire.ids import EntityKey, WorldInstanceId

from mineworld_agent.state import SeatState, StateDir

SEAT = EntityKey("visitor")
WORLD = WorldInstanceId("1a2b3c4d5e6f70819293a4b5c6d7e8f9")
OTHER = WorldInstanceId("ffffffffffffffffffffffffffffffff")
SECRET = "5f0c2a9e8d7b6c5a4f3e2d1c0b9a8f7e"


def test_a_rewrite_replaces_the_whole_file_and_leaves_nothing_beside_it(tmp_path: Path) -> None:
    states = StateDir(tmp_path / "state")
    states.write(SEAT, SeatState(instance=WORLD, resume=SECRET, cursor="40"))
    states.write(SEAT, SeatState(instance=WORLD, resume="0" * 32, cursor="57"))
    read = states.read(SEAT, WORLD)
    assert read is not None and (read.secret, read.since) == ("0" * 32, "57")
    assert sorted(p.name for p in (tmp_path / "state").iterdir()) == ["visitor.json"]


def test_the_file_is_readable_by_its_owner_only(tmp_path: Path) -> None:
    # On Windows the directory's ACL (the user's profile) governs; POSIX modes are asserted where they
    # exist (R-c-5). Never skipped: the write itself is checked on every platform.
    states = StateDir(tmp_path)
    states.write(SEAT, SeatState(instance=WORLD, resume=SECRET, cursor=None))
    assert states.path(SEAT).is_file()
    if os.name == "posix":
        assert stat.S_IMODE(states.path(SEAT).stat().st_mode) == 0o600


def test_a_file_of_another_world_or_one_that_does_not_parse_is_ignored(tmp_path: Path) -> None:
    states = StateDir(tmp_path)
    states.write(SEAT, SeatState(instance=OTHER, resume=SECRET, cursor="9"))
    assert states.read(SEAT, WORLD) is None
    states.path(SEAT).write_text("{ not json", encoding="utf-8")
    assert states.read(SEAT, WORLD) is None
    assert states.read(EntityKey("wanderer"), WORLD) is None


def test_leave_deletes_it_and_deleting_twice_is_harmless(tmp_path: Path) -> None:
    states = StateDir(tmp_path)
    states.write(SEAT, SeatState(instance=WORLD, resume=SECRET, cursor="1"))
    states.delete(SEAT)
    states.delete(SEAT)
    assert not states.path(SEAT).exists()
