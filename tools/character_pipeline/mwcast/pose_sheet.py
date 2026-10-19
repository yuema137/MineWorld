"""The S26 pose sheet driver (step-26 §6.4, P-1…P-8; PIPELINE.md §13).

Builds a scratch Godot project that copies the 3D client's body code, clips, the protagonist (the quality-tier
control) and the café's chair and table, imports the subject there (never in `clients/3d-spike`), runs
`godot/pose_sheet.gd` windowed at a fixed frame rate, and lays the frames and their clipping flags out
in a static `sheet.html`. Godot runs only as a subprocess with an argument list.
"""

from __future__ import annotations

import hashlib
import html
import json
import shutil
import struct
import subprocess
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import cast

GODOT_ENV = "MINEWORLD_GODOT"
# A normal run imports in under a minute and renders in under another (Apple M5): these stop a stuck run.
IMPORT_TIMEOUT_S = 900
RENDER_TIMEOUT_S = 600
CLIENT = Path("clients/3d-spike")
VERA = "res://assets/characters/meshy_d/meshy_d.glb"
# The client files the sheet runs on, copied (not linked): the scratch import never writes into the client.
COPIED = (
    "scripts/human.gd",
    "scripts/posture.gd",
    "scripts/mats.gd",  # human.gd's shader loader (class Mats) since main's 2026-10-18 anisotropy change
    "shaders/hair_card.gdshader",
    "assets/characters/meshy_d",
    "assets/characters/quaternius_ual.glb",
    "assets/characters/quaternius_ual.glb.import",
    "assets/characters/quaternius_ual_bonemap.tres",
    "assets/characters/vitruvian/vitruvian_bonemap.tres",
    "assets/characters/vitruvian/textures",
    "assets/models/dining_chair_02",
    "assets/models/round_wooden_table_02",
    "assets/hdri",
)
PROJECT_GODOT = """config_version=5

[application]

config/name="MineWorld pose sheet (scratch)"
config/features=PackedStringArray("4.7", "Forward Plus")

[display]

window/size/viewport_width=320
window/size/viewport_height=240

[rendering]

renderer/rendering_method="forward_plus"
textures/default_filters/anisotropic_filtering_level=4
lights_and_shadows/directional_shadow/size=4096
lights_and_shadows/directional_shadow/soft_shadow_filter_quality=2
"""


class PoseSheetError(Exception):
    """The subject cannot be staged, Godot is missing, or a run produced no report."""


@dataclass(frozen=True)
class Subject:
    res_path: str
    armature: str


def find_godot(environ: Mapping[str, str]) -> str:
    explicit = environ.get(GODOT_ENV, "")
    if explicit:
        return explicit
    found = shutil.which("godot")
    if found:
        return found
    raise PoseSheetError(f"no Godot found: set {GODOT_ENV} or put godot (4.7) on PATH")


def _gltf_document(path: Path) -> Mapping[str, object]:
    data = path.read_bytes()
    if path.suffix.lower() == ".glb":
        if data[:4] != b"glTF":
            raise PoseSheetError(f"{path}: not a GLB")
        length = struct.unpack("<I", data[12:16])[0]
        return cast(dict[str, object], json.loads(data[20 : 20 + length]))
    return cast(dict[str, object], json.loads(data))


def _objects(value: object) -> list[dict[str, object]]:
    """The JSON objects in a JSON array; anything else is skipped."""
    if not isinstance(value, list):
        return []
    return [cast(dict[str, object], item) for item in cast(list[object], value) if isinstance(item, dict)]


def armature_name(document: Mapping[str, object]) -> str:
    """The name of the node that holds the first skin's root joint: Godot names the skeleton path after it."""
    skins = _objects(document.get("skins"))
    joints = skins[0].get("joints") if skins else None
    if not isinstance(joints, list) or not joints:
        raise PoseSheetError("the subject has no skin")
    root_joint = cast(list[object], joints)[0]
    for node in _objects(document.get("nodes")):
        children = node.get("children")
        if isinstance(children, list) and root_joint in cast(list[object], children):
            return str(node.get("name", ""))
    raise PoseSheetError("the subject's root joint has no parent node; Godot's skeleton path is unknown")


def _clean_import(text: str) -> str:
    """Drop the uid, the cached path and the [deps] block: the scratch project derives its own."""
    out: list[str] = []
    skip = False
    for line in text.splitlines():
        if line.startswith("["):
            skip = line.strip() == "[deps]"
        if skip or line.startswith("uid=") or line.startswith("path="):
            continue
        out.append(line)
    return "\n".join(out) + "\n"


def stage_glb(repo_root: Path, source: Path, staged: Path, name: str) -> Subject:
    """One more GLB (S32-p `--pair`: outfit B beside outfit A), with the protagonist's import settings."""
    armature = armature_name(_gltf_document(source))
    shutil.copy2(source, staged / f"{name}.glb")
    template = (repo_root / CLIENT / "assets/characters/meshy_d/meshy_d.glb.import").read_text()
    settings = _clean_import(template).replace(
        "PATH:mixamo_vitruvian/Skeleton3D", f"PATH:{armature}/Skeleton3D"
    )
    (staged / f"{name}.glb.import").write_text(settings)
    return Subject(res_path=f"res://cast/{name}.glb", armature=armature)


def stage_project(repo_root: Path, subject: Path, project: Path) -> Subject:
    """Copy the client files and the subject into `project`; write `project.godot` and the import settings."""
    project.mkdir(parents=True, exist_ok=True)
    client = repo_root / CLIENT
    for rel in COPIED:
        src = client / rel
        dst = project / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        if src.is_dir():
            shutil.copytree(src, dst, dirs_exist_ok=True)
        else:
            shutil.copy2(src, dst)
    for imp in project.rglob("*.import"):
        imp.write_text(_clean_import(imp.read_text()))
    shutil.copy2(repo_root / "tools/character_pipeline/godot/pose_sheet.gd", project / "pose_sheet.gd")
    (project / "project.godot").write_text(PROJECT_GODOT)

    document = _gltf_document(subject)
    armature = armature_name(document)
    staged = project / "cast"
    if staged.exists():
        shutil.rmtree(staged)
    staged.mkdir()
    name = "subject" + subject.suffix.lower()
    shutil.copy2(subject, staged / name)
    if subject.suffix.lower() == ".gltf":
        for key in ("buffers", "images"):
            for item in _objects(document.get(key)):
                uri = item.get("uri")
                if isinstance(uri, str) and uri and not uri.startswith("data:"):
                    shutil.copy2(subject.parent / uri, staged / uri)
    template = (client / "assets/characters/meshy_d/meshy_d.glb.import").read_text()
    settings = _clean_import(template).replace(
        "PATH:mixamo_vitruvian/Skeleton3D", f"PATH:{armature}/Skeleton3D"
    )
    (staged / f"{name}.import").write_text(settings)
    return Subject(res_path=f"res://cast/{name}", armature=armature)


def godot_commands(
    godot: str,
    project: Path,
    subject: Subject,
    out: Path,
    label: str,
    grip: bool,
    plant_seat_gap: float,
    outfit: Path | None = None,
    pair: Subject | None = None,
) -> tuple[list[str], list[str]]:
    importer = [godot, "--headless", "--path", str(project), "--import"]
    render = [
        godot,
        "--path",
        str(project),
        "--always-on-top",
        "--fixed-fps",
        "30",
        "--disable-vsync",
        "--script",
        "res://pose_sheet.gd",
        "--",
        "--subject",
        subject.res_path,
        "--vera",
        VERA,
        "--out",
        str(out.resolve()),
        "--label",
        label,
    ]
    if grip:
        render.append("--grip")
    if plant_seat_gap:
        render += ["--plant-seat-gap", repr(plant_seat_gap)]
    if outfit is not None:
        render += ["--outfit", str(outfit.resolve())]
    if pair is not None:
        render += ["--pair", pair.res_path]
    return importer, render


def write_sheet(out: Path) -> Path:
    """`sheet.html` beside `report.json`: one static page, every frame with its flags."""
    report = json.loads((out / "report.json").read_text())
    cards: list[str] = []
    for frame in report["frames"]:
        digest = hashlib.sha256((out / frame["file"]).read_bytes()).hexdigest()[:16]
        status = "PASS" if frame["pass"] else "FLAGGED"
        rows = "".join(
            f"<tr><th>{html.escape(name)}</th><td>{html.escape(json.dumps(value))}</td></tr>"
            for name, value in frame["checks"].items()
        )
        cards.append(
            f'<figure class="{status.lower()}"><a href="{html.escape(frame["file"])}">'
            f'<img src="{html.escape(frame["file"])}" alt="{html.escape(frame["id"])}"></a>'
            f'<figcaption><b>{html.escape(frame["id"])}</b> <span class="tag">{status}</span><br>'
            f"{html.escape(frame['what'])}<table>{rows}</table><code>sha256 {digest}…</code>"
            "</figcaption></figure>"
        )
    notes = "".join(f"<li>{html.escape(n)}</li>" for n in report["notes"])
    bounds = report["bounds"]
    flagged = sum(1 for f in report["frames"] if not f["pass"])
    page = f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>Pose sheet: {html.escape(report["label"])}</title>
<style>
body {{ font: 14px/1.4 system-ui, sans-serif; margin: 24px; background: #f4f2ee; color: #222; }}
.grid {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 16px; }}
figure {{ margin: 0; background: #fff; padding: 8px; border: 4px solid #3a7d44; }}
figure.flagged {{ border-color: #d81414; }}
img {{ width: 100%; display: block; }}
.tag {{ font-weight: bold; color: #3a7d44; }} .flagged .tag {{ color: #d81414; }}
table {{ font-size: 12px; border-collapse: collapse; }} th {{ text-align: left; padding-right: 6px; }}
code {{ font-size: 11px; color: #666; }}
</style></head><body>
<h1>Pose sheet: {html.escape(report["label"])}</h1>
<p>Subject <code>{html.escape(report["subject"])}</code>; control <code>{html.escape(report["vera"])}</code>
(Vera, step-26 §6.3). {len(report["frames"])} frames, <b>{flagged} flagged</b> (red border; torso
penetration over {bounds["torso_clip_mm"]} mm, seat contact off by more than {bounds["seat_contact_mm"]} mm).
Rendered by Godot {html.escape(report["godot"])} on {html.escape(report["renderer"])}
at a fixed {report["fps"]} fps.
</p>
<ul>{notes}</ul>
<div class="grid">{"".join(cards)}</div>
</body></html>
"""
    sheet = out / "sheet.html"
    sheet.write_text(page)
    return sheet


def run(
    repo_root: Path,
    subject: Path,
    out: Path,
    label: str,
    grip: bool,
    plant_seat_gap: float,
    environ: Mapping[str, str],
    project: Path | None = None,
    outfit: Path | None = None,
    pair: Path | None = None,
) -> Path:
    godot = find_godot(environ)
    project = project or out / ".project"
    staged = stage_project(repo_root, subject.resolve(), project)
    staged_pair = stage_glb(repo_root, pair.resolve(), project / "cast", "pair") if pair else None
    out.mkdir(parents=True, exist_ok=True)
    frames = out / "frames"
    if frames.exists():
        shutil.rmtree(frames)
    report = out / "report.json"
    report.unlink(missing_ok=True)
    importer, render = godot_commands(
        godot, project, staged, out, label, grip, plant_seat_gap, outfit=outfit, pair=staged_pair
    )
    _call(importer, out / "import.log", IMPORT_TIMEOUT_S)
    _call(render, None, RENDER_TIMEOUT_S)
    if not report.exists():
        raise PoseSheetError(f"Godot wrote no {report}; see its output above")
    return write_sheet(out)


def _call(args: Sequence[str], log: Path | None, timeout_s: int) -> None:
    print("pose-sheet: " + " ".join(args) + (f" > {log}" if log else ""), flush=True)
    try:
        if log is None:
            subprocess.run(list(args), check=False, timeout=timeout_s)
            return
        with log.open("w") as handle:
            subprocess.run(
                list(args), check=False, stdout=handle, stderr=subprocess.STDOUT, timeout=timeout_s
            )
    except subprocess.TimeoutExpired as exc:
        raise PoseSheetError(
            f"Godot did not finish in {timeout_s} s and was stopped. A window that is hidden, minimised or "
            "on another desktop stops drawing on macOS, and the sheet waits for each drawn frame."
        ) from exc
