"""The ground contact probe over every body the 3D client draws (PIPELINE.md §15.1).

Stages one scratch Godot project that holds the client's body code, clips, the protagonist and every
committed cast body at its own `res://` path (never importing in `clients/3d-spike`), runs
`godot/pose_sheet.gd --ground-only` headless at a fixed frame rate, and reads back one ground block per
body: the lowest skinned vertex against the floor at rest, over the `Stand` loop, and every frame of one
`Walk` and one `Jog_Fwd` cycle. `--write` records each block in the body's manifest (`character.yaml`)
or, for the reference body, in the client's resource index, where `scripts/check_cast.py` (CC-9) holds
every body to the bound.
"""

from __future__ import annotations

import json
import shutil
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import cast

from mwcast import pose_sheet

PACK_CAST = Path("presentation/mineworld-default/3D/characters/cast")
CLIENT_CAST = Path("assets/characters/cast")
INDEX = CLIENT_CAST / "index.json"
SINK_MM = 5.0
FLOAT_MM = 20.0
# The fields of a ground block that are recorded; the rest of the probe's output is diagnosis.
RECORDED = (
    "rest_mm",
    "stand",
    "walk",
    "jog",
    "worst_sink_mm",
    "worst_float_mm",
    "pass",
    "arm_clearance_mm",
)


class GroundError(Exception):
    """A body cannot be staged, Godot is missing, or the run wrote no measurement."""


@dataclass(frozen=True)
class Body:
    id: str
    manifest: Path | None  # None: the reference body (the client's index names it)
    grip: str = ""  # the side a reference body's strap grip holds (its index entry's `grip`), not gated


def bodies(repo_root: Path) -> list[Body]:
    """Every body the client draws: the index's reference bodies, then each manifest in the default pack."""
    found: list[Body] = []
    index = json.loads((repo_root / pose_sheet.CLIENT / INDEX).read_text())
    for character, entry in sorted(cast(dict[str, dict[str, object]], index["characters"]).items()):
        if entry.get("body") == "reference":
            found.append(Body(character, None, str(entry.get("grip") or "")))
    for manifest in sorted((repo_root / PACK_CAST).glob("*/character.yaml")):
        found.append(Body(manifest.parent.name, manifest))
    return found


def stage(repo_root: Path, project: Path, chosen: Sequence[Body]) -> None:
    """The scratch project: the pose sheet's client files, plus each cast body's directory at its own path."""
    project.mkdir(parents=True, exist_ok=True)
    client = repo_root / pose_sheet.CLIENT
    for rel in pose_sheet.COPIED:
        _copy(client / rel, project / rel)
    for body in chosen:
        if body.manifest is not None:
            scene = str(json.loads(body.manifest.read_text())["scene"]).removeprefix("res://")
            _copy(client / Path(scene).parent, project / Path(scene).parent)
    for imp in project.rglob("*.import"):
        imp.write_text(pose_sheet.clean_import(imp.read_text()))
    shutil.copy2(repo_root / "tools/character_pipeline/godot/pose_sheet.gd", project / "pose_sheet.gd")
    shutil.copy2(repo_root / "tools/character_pipeline/godot/stand_gates.gd", project / "stand_gates.gd")
    (project / "project.godot").write_text(pose_sheet.PROJECT_GODOT)


def _copy(src: Path, dst: Path) -> None:
    dst.parent.mkdir(parents=True, exist_ok=True)
    if src.is_dir():
        shutil.copytree(src, dst, dirs_exist_ok=True)
    else:
        shutil.copy2(src, dst)


def run(
    repo_root: Path,
    out: Path,
    chosen: Sequence[Body],
    environ: Mapping[str, str],
    dump_pose: bool = False,
) -> dict[str, object]:
    godot = pose_sheet.find_godot(environ)
    project = out / ".project"
    stage(repo_root, project, chosen)
    out.mkdir(parents=True, exist_ok=True)
    listing = [
        {"id": b.id, "reference": True, "grip": b.grip}
        if b.manifest is None
        else {"id": b.id, "manifest": str(b.manifest.resolve())}
        for b in chosen
    ]
    (out / "bodies.json").write_text(json.dumps(listing, indent=2) + "\n")
    result = out / "ground.json"
    result.unlink(missing_ok=True)
    pose_sheet.run_logged(
        [godot, "--headless", "--path", str(project), "--import"],
        out / "import.log",
        pose_sheet.IMPORT_TIMEOUT_S,
    )
    probe = [
        godot,
        "--headless",
        "--path",
        str(project),
        "--fixed-fps",
        "30",
        "--script",
        "res://pose_sheet.gd",
        "--",
        "--ground-only",
        "--bodies",
        str((out / "bodies.json").resolve()),
        "--out",
        str(out.resolve()),
    ]
    if dump_pose:
        (out / "poses").mkdir(parents=True, exist_ok=True)
        probe += ["--dump-pose", str((out / "poses").resolve())]
    pose_sheet.run_logged(probe, out / "probe.log", pose_sheet.RENDER_TIMEOUT_S * 3)
    if not result.exists():
        raise GroundError(f"Godot wrote no {result}; see {out / 'probe.log'}")
    return cast(dict[str, object], json.loads(result.read_text()))


def table(report: Mapping[str, object]) -> str:
    """One line per body, millimetres: rest; Stand lowest/highest; Walk and Jog lowest and stance float."""
    lines = [
        f"{'body':<18}{'rest':>8}{'stand lo':>10}{'stand hi':>10}{'walk lo':>9}{'walk fl':>9}"
        f"{'jog lo':>8}{'jog fl':>8}  verdict"
    ]
    for row in cast(list[dict[str, object]], report["bodies"]):
        stand = cast(dict[str, float], row["stand"])
        walk = cast(dict[str, object], row["walk"])
        jog = cast(dict[str, object], row["jog"])
        jog_feet = cast(list[float], jog["foot_contact_mm"])
        lines.append(
            f"{row['id']!s:<18}{row['rest_mm']:>8.1f}{stand['min_mm']:>10.1f}{stand['max_mm']:>10.1f}"
            f"{cast(float, walk['min_mm']):>9.1f}{cast(float, walk['frame_low_max_mm']):>9.1f}"
            f"{cast(float, jog['min_mm']):>8.1f}{max(jog_feet):>8.1f}  {'PASS' if row['pass'] else 'FAIL'}"
        )
    return "\n".join(lines)


def recorded(row: Mapping[str, object], godot: str) -> dict[str, object]:
    block: dict[str, object] = {key: row[key] for key in RECORDED if key in row}
    block["bounds_mm"] = {"sink": SINK_MM, "float": FLOAT_MM}
    block["measured_by"] = (
        f"tools/character_pipeline/cast.py ground (pose_sheet.gd --ground-only, Godot {godot})"
    )
    return block


def write(repo_root: Path, report: Mapping[str, object], chosen: Sequence[Body]) -> list[Path]:
    """Record each body's ground block: in its manifest, or in the index entry of a reference body."""
    godot = str(report["godot"])
    rows = {str(r["id"]): r for r in cast(list[dict[str, object]], report["bodies"])}
    written: list[Path] = []
    index_path = repo_root / pose_sheet.CLIENT / INDEX
    index = json.loads(index_path.read_text())
    for body in chosen:
        row = rows.get(body.id)
        if row is None:
            continue
        if body.manifest is None:
            index["characters"][body.id]["ground"] = recorded(row, godot)
            index["characters"][body.id]["stand_gates"] = gates_block(row, godot)
            continue
        manifest = json.loads(body.manifest.read_text())
        manifest["ground"] = recorded(row, godot)
        manifest["stand_gates"] = gates_block(row, godot)
        body.manifest.write_text(json.dumps(manifest, indent=2) + "\n")
        written.append(body.manifest)
    index_path.write_text(json.dumps(index, indent=2) + "\n")
    written.append(index_path)
    return written


def gates_block(row: Mapping[str, object], godot: str) -> dict[str, object]:
    """A body's `stand_gates` (PIPELINE.md §15.2): the worst palm, reach, clearance and joint angles over
    every frame of the Stand loop, the misses and the verdict, as the probe measured them."""
    block = dict(cast(Mapping[str, object], row.get("stand_gates", {})))
    block["measured_by"] = (
        f"tools/character_pipeline/cast.py ground (stand_gates.gd on every Stand frame, Godot {godot})"
    )
    return block
