"""The nightly memory scale gates (pr-s28g §5.5; step-28 AS28-17, AS28-12's steady state, AS28-18's 300-day C-7 and
C-8): 12 people for 300 days on social-cafe, and the café tiled to 100 people for 300 days, through the real binary
and the shipped memory code. Marker `scale`: deselected by the member's `addopts`, run by the nightly
`memory-scale` layer on Linux, macOS and Windows. Each gate writes its measurements to
`artifacts/nightly/timings-memory-scale.json` and asserts `check(...) == []`; every S-n lives in `check`.
"""

from __future__ import annotations

import os
import sys
from pathlib import Path

import pytest
from binary import REPOSITORY
from scale_harness import Homes, ScaleSpec, check, run_scale, tiled_world, write_timings

pytestmark = [pytest.mark.scale, pytest.mark.real_binary]

DAYS = 300
HOMES: dict[str, Homes] = {"linux": "one", "darwin": "per-block", "win32": "per-block"}
"""QS28g-2: one 100-seat home on Linux (the step's one-file-for-many-seats question); one home per block in
parallel workers on macOS and Windows. Every seat's memory is the same either way (I-S28-2)."""
CAFE = REPOSITORY / "worlds" / "social-cafe"


def _release_binary() -> None:
    """The nightly builds the release binary (the layer's first command); the harness uses it when it exists."""
    suffix = ".exe" if sys.platform == "win32" else ""
    release = REPOSITORY / "target" / "release" / f"mineworld{suffix}"
    if "MINEWORLD_BIN" not in os.environ and release.is_file():
        os.environ["MINEWORLD_BIN"] = str(release)


def test_twelve_people_for_three_hundred_days(tmp_path: Path) -> None:
    _release_binary()
    spec = ScaleSpec(
        world=CAFE,
        days=DAYS,
        seed=7,
        frame=4096,
        homes="one",
        workers=1,
        sample=10,
        continuity=True,
    )
    run = run_scale(spec, tmp_path)
    write_timings(run, "memory-scale")
    assert check(run) == []


def test_a_hundred_people_for_three_hundred_days(tmp_path: Path) -> None:
    _release_binary()
    world = tiled_world(CAFE, 100, tmp_path / "worlds")
    spec = ScaleSpec(
        world=world,
        days=DAYS,
        seed=7,
        frame=4096,
        homes=HOMES.get(sys.platform, "per-block"),
        workers=os.cpu_count() or 2,
        sample=10,
        continuity=False,
    )
    run = run_scale(spec, tmp_path)
    write_timings(run, "memory-scale")
    assert check(run) == []
