"""The pose sheet driver's staging and page (PIPELINE.md §15). Godot runs only in the sheet itself."""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from conftest import REPO_ROOT

from mwcast import pose_sheet


def test_the_subject_is_imported_with_the_protagonists_retarget_under_its_own_armature(
    tmp_path: Path,
) -> None:
    subject = tmp_path / "someone.gltf"
    document = {
        "nodes": [{"name": "Hips"}, {"name": "rig_of_someone", "children": [0]}],
        "skins": [{"joints": [0]}],
        "buffers": [{"uri": "someone.bin"}],
        "images": [{"uri": "someone_base_color.png"}],
    }
    subject.write_text(json.dumps(document))
    (tmp_path / "someone.bin").write_bytes(b"\0")
    (tmp_path / "someone_base_color.png").write_bytes(b"\0")

    staged = pose_sheet.stage_project(REPO_ROOT, subject, tmp_path / "project")

    assert staged == pose_sheet.Subject(res_path="res://cast/subject.gltf", armature="rig_of_someone")
    settings = (tmp_path / "project/cast/subject.gltf.import").read_text()
    assert '"PATH:rig_of_someone/Skeleton3D"' in settings
    assert "vitruvian_bonemap.tres" in settings
    assert "uid=" not in settings and "[deps]" not in settings
    assert (tmp_path / "project/cast/someone.bin").exists()
    assert (tmp_path / "project/cast/someone_base_color.png").exists()
    assert (tmp_path / "project/scripts/human.gd").read_bytes() == (
        REPO_ROOT / "clients/3d-spike/scripts/human.gd"
    ).read_bytes()


def test_a_subject_without_a_skin_is_refused(tmp_path: Path) -> None:
    subject = tmp_path / "static.gltf"
    subject.write_text(json.dumps({"nodes": [{"name": "Mesh"}]}))

    with pytest.raises(pose_sheet.PoseSheetError, match="no skin"):
        pose_sheet.stage_project(REPO_ROOT, subject, tmp_path / "project")


def test_the_sheet_shows_a_flagged_frame_as_flagged(tmp_path: Path) -> None:
    (tmp_path / "frames").mkdir()
    for name in ("a", "b"):
        (tmp_path / f"frames/{name}.png").write_bytes(name.encode())
    report: dict[str, object] = {
        "label": "someone",
        "subject": "res://cast/subject.glb",
        "vera": pose_sheet.VERA,
        "bounds": {"torso_clip_mm": 5.0, "seat_contact_mm": 20.0},
        "godot": "4.7.2",
        "renderer": "gpu",
        "fps": 30.0,
        "notes": ["a note"],
        "frames": [
            {"id": "a", "what": "stand", "file": "frames/a.png", "pass": True, "checks": {}},
            {"id": "b", "what": "seated", "file": "frames/b.png", "pass": False, "checks": {"seat": 1}},
        ],
    }
    (tmp_path / "report.json").write_text(json.dumps(report))

    page = pose_sheet.write_sheet(tmp_path).read_text()

    assert '<figure class="pass"><a href="frames/a.png">' in page
    assert '<figure class="flagged"><a href="frames/b.png">' in page
    assert "<b>1 flagged</b>" in page
