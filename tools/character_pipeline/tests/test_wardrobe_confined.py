"""I-p-10: MPFB2 stays a hidden tool. No MIT module imports `bpy` or MPFB2, no CI command runs
`blender/mpfb`, and every MPFB2 stage is GPL-3.0-or-later beside its licence text. Also the stage driver's
refusals (I-p-8)."""

from __future__ import annotations

import ast
from collections.abc import Mapping, Sequence
from pathlib import Path

import pytest

from mwcast.wardrobe import stages

PIPELINE = Path(__file__).resolve().parents[1]
REPO = PIPELINE.parents[1]
FORBIDDEN = ("bpy", "bmesh", "mathutils", "bl_ext")


def _imports(path: Path) -> set[str]:
    tree = ast.parse(path.read_text(encoding="utf-8"))
    names: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            names.update(alias.name.split(".")[0] for alias in node.names)
        elif isinstance(node, ast.ImportFrom) and node.module:
            names.add(node.module.split(".")[0])
    return names


def test_no_mwcast_module_imports_blender_or_mpfb() -> None:
    offenders = [
        str(path.relative_to(PIPELINE))
        for path in sorted((PIPELINE / "mwcast").rglob("*.py"))
        if _imports(path) & set(FORBIDDEN)
    ]
    assert offenders == []


def test_no_ci_command_names_the_mpfb_stages() -> None:
    sources = [REPO / "scripts" / "ci_layer.py", *sorted((REPO / ".github" / "workflows").glob("*.yml"))]
    assert [str(p) for p in sources if "blender/mpfb" in p.read_text(encoding="utf-8")] == []


def test_every_mpfb_stage_is_gpl3_beside_its_licence() -> None:
    folder = PIPELINE / "blender" / "mpfb"
    assert (folder / "LICENSE").read_text(encoding="utf-8").lstrip().startswith("GNU GENERAL PUBLIC LICENSE")
    scripts = sorted(folder.glob("*.py"))
    assert scripts
    for script in scripts:
        first = script.read_text(encoding="utf-8").splitlines()[0]
        assert first == "# SPDX-License-Identifier: GPL-3.0-or-later", script.name


def test_a_stage_without_the_slot_wrapper_is_refused(tmp_path: Path) -> None:
    manifest = tmp_path / "blender-user" / "extensions" / "user_default" / "mpfb" / "blender_manifest.toml"
    manifest.parent.mkdir(parents=True)
    manifest.write_text('id = "mpfb"\nversion = "2.0.18"\n', encoding="utf-8")
    calls: list[list[str]] = []

    def executor(args: Sequence[str], env: Mapping[str, str], log: Path) -> int:
        calls.append(list(args))
        assert env[stages.USER_RESOURCES_ENV] == str(tmp_path / "blender-user")
        return 0

    with pytest.raises(stages.StageError, match="with-slot.sh"):
        stages.run_stage(
            "regions",
            {},
            scratch=tmp_path,
            run_dir=tmp_path / "r",
            environ={},
            blender="b",
            executor=executor,
        )
    stages.run_stage(
        "regions",
        {"out": "x"},
        scratch=tmp_path,
        run_dir=tmp_path / "r",
        environ={stages.SLOT_ENV: "/slots/with-slot.sh"},
        blender="b",
        executor=executor,
    )
    assert calls[0][:2] == ["/slots/with-slot.sh", "b"]
    assert "--factory-startup" not in calls[0]  # DEP-72 item 4: the extension must load
    assert calls[0][-1].endswith("regions.params.json")


def test_a_stage_needs_mpfb_2_0_18_installed(tmp_path: Path) -> None:
    with pytest.raises(stages.StageError, match="is not installed"):
        stages.run_stage(
            "regions",
            {},
            scratch=tmp_path,
            run_dir=tmp_path / "r",
            environ={stages.SLOT_ENV: "s"},
            blender="b",
            executor=lambda a, e, log: 0,
        )
