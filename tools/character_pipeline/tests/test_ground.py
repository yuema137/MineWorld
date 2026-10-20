"""The ground probe's driver (PIPELINE.md §15.1): which bodies it measures and where it records them.

Godot runs only in the probe itself; these tests feed the driver a probe report."""

from __future__ import annotations

import json
from pathlib import Path

from conftest import REPO_ROOT

from mwcast import ground


def _row(body: str, walk_min: float) -> dict[str, object]:
    return {
        "id": body,
        "rest_mm": 0.0,
        "stand": {"min_mm": 0.0, "max_mm": 0.0},
        "walk": {"min_mm": walk_min, "frame_low_max_mm": 0.0},
        "jog": {"min_mm": -0.5, "foot_contact_mm": [-0.5, -0.2]},
        "worst_sink_mm": walk_min,
        "worst_float_mm": 0.0,
        "pass": walk_min >= -ground.SINK_MM,
        "motion_scale": 0.82,
    }


def _repo(tmp: Path) -> Path:
    index = tmp / "clients/3d-spike/assets/characters/cast/index.json"
    index.parent.mkdir(parents=True)
    index.write_text(json.dumps({"format": 1, "characters": {"vera-lindgren": {"body": "reference"}}}))
    manifest = tmp / ground.PACK_CAST / "elin-berg/character.yaml"
    manifest.parent.mkdir(parents=True)
    manifest.write_text(json.dumps({"format": 1, "id": "elin-berg", "scene": "res://x.gltf"}))
    (tmp / ground.PACK_CAST / "greta-novak").mkdir()  # a brief-only directory: no body yet
    return tmp


def test_every_committed_body_is_measured_the_protagonist_first() -> None:
    found = ground.bodies(REPO_ROOT)

    assert found[0] == ground.Body("vera-lindgren", None)
    assert len(found) == 1 + len(list((REPO_ROOT / ground.PACK_CAST).glob("*/character.yaml")))


def test_write_records_the_block_in_the_manifest_and_the_reference_in_the_index(tmp_path: Path) -> None:
    repo = _repo(tmp_path)
    chosen = ground.bodies(repo)
    report = {"godot": "4.7.2", "bodies": [_row("vera-lindgren", -0.8), _row("elin-berg", -12.0)]}

    ground.write(repo, report, chosen)

    elin = json.loads((repo / ground.PACK_CAST / "elin-berg/character.yaml").read_text())
    assert elin["scene"] == "res://x.gltf"
    assert elin["ground"]["walk"]["min_mm"] == -12.0 and elin["ground"]["pass"] is False
    assert "motion_scale" not in elin["ground"]
    index = json.loads((repo / "clients/3d-spike/assets/characters/cast/index.json").read_text())
    assert index["characters"]["vera-lindgren"]["ground"]["pass"] is True
    assert "FAIL" in ground.table(report).splitlines()[2]
