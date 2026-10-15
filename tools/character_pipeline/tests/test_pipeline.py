"""The stage 3 crop rule, the stage planner, the Blender command lines and the pin (design §9 B6 and B8)."""

from __future__ import annotations

import copy
from collections.abc import Sequence
from pathlib import Path
from typing import NoReturn

import pytest
from test_records import vera_fit_object

from mwcast import blender, pipeline
from mwcast.mask import alpha_bbox
from mwcast.records import FitParams, parse_fit
from mwcast.standard import BLENDER_PIN


def _grid(rows: list[list[int]]) -> list[bytes]:
    return [bytes(row) for row in rows]


def test_alpha_bbox_returns_the_hand_computed_box_on_a_6_by_5_grid() -> None:
    # Five rows of six pixels. Two foreground pixels: (x=1, y=1) and (x=4, y=3), both well above 16.
    grid = _grid(
        [
            [0, 0, 0, 0, 0, 0],
            [0, 200, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 200, 0],
            [0, 0, 0, 0, 0, 0],
        ]
    )

    assert alpha_bbox(grid, 16) == (1, 1, 5, 4)


def test_alpha_at_the_threshold_is_not_foreground() -> None:
    # The threshold is strict: 16 is background, 17 is foreground (D+ `prep_inputs.py` tests alpha > 16).
    grid = _grid([[16, 0, 0], [0, 0, 0], [0, 0, 17]])

    assert alpha_bbox(grid, 16) == (2, 2, 3, 3)


def test_a_fully_transparent_image_has_no_foreground_and_is_refused() -> None:
    grid = _grid([[0, 0, 0], [0, 0, 0]])

    with pytest.raises(ValueError, match="no pixel"):
        alpha_bbox(grid, 16)


def _fit(*, accessories: list[str], flecks: bool) -> FitParams:
    raw = copy.deepcopy(vera_fit_object())
    raw["accessories"] = accessories
    raw["flecks"] = {"enabled": flecks, "z_min_m": 1.65, "dark": 0.30, "lighter": 0.12}
    return parse_fit(raw, Path("fit.yaml"))


def test_vera_plans_every_stage_but_the_flecks_she_does_not_get() -> None:
    plan = pipeline.plan(_fit(accessories=["backpack"], flecks=False))

    assert plan.stages == (
        "gameready",
        "fit_bind",
        "weights_fingers",
        "weights_pack",
        "export",
        "render_bones",
        "dump_rig",
    )
    assert plan.skipped == (("flecks", "flecks.enabled is false"),)


def test_a_fit_without_a_pack_keeps_flecks_and_drops_the_pack_stage() -> None:
    plan = pipeline.plan(_fit(accessories=[], flecks=True))

    assert "weights_pack" not in plan.stages
    assert "flecks" in plan.stages
    assert plan.skipped == (("weights_pack", "no pack in fit.accessories"),)


def test_a_stage_bound_that_is_skipped_is_refused() -> None:
    with pytest.raises(pipeline.PipelineError, match="skipped"):
        pipeline.plan(_fit(accessories=["backpack"], flecks=False), start="flecks")


def test_the_blender_command_line_is_the_argument_list_of_the_design() -> None:
    binary = Path("/opt/blender")
    script = Path("/repo/blender/export.py")
    params = Path("/run/params.json")

    assert blender.command(binary, script, params, None) == [
        "/opt/blender",
        "--background",
        "--factory-startup",
        "--python-exit-code",
        "1",
        "--python",
        "/repo/blender/export.py",
        "--",
        "/run/params.json",
    ]
    with_blend = blender.command(binary, script, params, Path("/run/weights_fingers.blend"))
    assert with_blend[3] == "/run/weights_fingers.blend"


def test_the_pin_refuses_another_version_and_accepts_the_named_one() -> None:
    assert blender.check_pin(BLENDER_PIN, None) is False
    with pytest.raises(blender.BlenderError, match="5.2.1"):
        blender.check_pin("Blender 5.2.1", None)
    assert blender.check_pin("Blender 5.2.1", "Blender 5.2.1") is True


def test_a_download_whose_sha256_differs_stops_before_any_subprocess(tmp_path: Path) -> None:
    fit = _fit(accessories=["backpack"], flecks=False)
    download = tmp_path / "download"
    download.mkdir()
    (download / fit.source.download_file).write_bytes(b"not the accepted download")
    calls: list[Sequence[str]] = []

    class Recorder:
        def __call__(self, args: Sequence[str]) -> NoReturn:
            calls.append(args)
            raise AssertionError("no subprocess may run")

    with pytest.raises(pipeline.PipelineError, match="SHA-256"):
        pipeline.run(
            fit=fit,
            fit_path=tmp_path / "fit.yaml",
            download_dir=download,
            scratch=tmp_path / "scratch",
            run_id="t",
            start=None,
            stop=None,
            blender=Path("/opt/blender"),
            blender_version=BLENDER_PIN,
            allowed_non_pin=False,
            executor=Recorder(),
        )
    assert calls == []
