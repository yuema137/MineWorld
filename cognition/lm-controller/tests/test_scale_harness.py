"""The scale harness's per-PR smoke run (pr-s28g §5.3, AC-g-4, AC-g-6): the café tiled to 24 people, 2 days, one
home per block in two workers, two seats rebuilt alone. Every check holds; then each S-n is shown to fire on a
copy of the results with one planted violation. The 300-day gates are the nightly's (`test_memory_scale.py`).
"""

from __future__ import annotations

import re
from dataclasses import replace
from pathlib import Path

import pytest
from binary import REPOSITORY
from scale_harness import ScaleRun, ScaleSpec, check, run_scale, tiled_world

HARNESS = Path(__file__).resolve().parent / "scale_harness.py"


@pytest.fixture(scope="module")
def smoke(tmp_path_factory: pytest.TempPathFactory) -> ScaleRun:
    scratch = tmp_path_factory.mktemp("scale")
    world = tiled_world(REPOSITORY / "worlds" / "social-cafe", 24, scratch)
    spec = ScaleSpec(
        world=world,
        days=2,
        seed=7,
        frame=4096,
        homes="per-block",
        workers=2,
        sample=2,
        continuity=False,
    )
    return run_scale(spec, scratch)


def _one(run: ScaleRun, prefix: str) -> list[str]:
    return [message for message in check(run) if message.startswith(prefix)]


@pytest.mark.real_binary
def test_the_smoke_run_holds_every_check(smoke: ScaleRun) -> None:
    assert len(smoke.homes) == 2
    assert sum(len(home.seats) for home in smoke.homes) == 24
    assert len(smoke.rebuilt) == 2
    assert all(seat.facts > 0 for home in smoke.homes for seat in home.seats)
    assert check(smoke) == []


@pytest.mark.real_binary
def test_each_check_fires_on_its_planted_violation(smoke: ScaleRun) -> None:
    home = smoke.homes[0]
    seat = home.seats[0]
    bounds = smoke.bounds

    def with_home(**changes: object) -> ScaleRun:
        return replace(smoke, homes=(replace(home, **changes), *smoke.homes[1:]))  # type: ignore[arg-type]

    def with_seat(**changes: object) -> ScaleRun:
        return with_home(seats=(replace(seat, **changes), *home.seats[1:]))  # type: ignore[arg-type]

    assert _one(with_seat(largest_size=bounds.bound + 1), "S-1")
    assert _one(with_seat(largest_after_pass=bounds.target + 1), "S-1")
    assert _one(with_home(physical=home.logical + 1), "S-2")
    assert _one(with_home(file_bytes=home.physical * 10 + 9 * 1_048_576), "S-3")
    assert _one(with_home(peak_rss=10 * 1024**3), "S-4")
    sampled = next(key for key in smoke.rebuilt)
    owner = next(one for one in smoke.homes if any(s.seat == sampled for s in one.seats))
    planted = tuple(
        replace(s, dump_sha256="0" * 64) if s.seat == sampled else s for s in owner.seats
    )
    others = tuple(one for one in smoke.homes if one is not owner)
    assert _one(replace(smoke, homes=(replace(owner, seats=planted), *others)), "S-5")
    assert _one(with_home(anchor_rows_max=home.anchors_max + 1), "S-6")
    cafe = replace(smoke, spec=replace(smoke.spec, continuity=True))
    dense = replace(cafe, homes=(replace(home, physical=home.logical), *smoke.homes[1:]))
    assert _one(dense, "S-7")
    marked = replace(
        seat, dump_sha256="0" * 64, l2_empty_days=(9,), most_met_in_l3=False, unexplained=3
    )
    messages = check(
        replace(cafe, homes=(replace(home, seats=(marked, *home.seats[1:])), *smoke.homes[1:]))
    )
    assert any(m.startswith("S-8") and "L2 empty" in m for m in messages)
    assert any(m.startswith("S-8") and "most-met" in m for m in messages)
    assert any(m.startswith("S-9") for m in messages)


def test_the_harness_writes_no_perceived_fact_and_imports_no_model() -> None:
    # AC-g-6: the only file the harness opens for writing is the timings file; no backend, gateway or record.
    text = HARNESS.read_text(encoding="utf-8")
    # A bare `open(` or `Path.open(` (never `MemoryHome.open(`, which opens the home) would be a file write.
    assert re.findall(r"\.write_text\(|\.write_bytes\(|(?<![\w.])open\(|\b\w+\.open\(", text) == [
        "MemoryHome.open(",
        "MemoryHome.open(",
        ".write_text(",
    ]
    assert not re.search(r"^(from|import) \S*(backend|gateway|record)\b", text, re.MULTILINE)
