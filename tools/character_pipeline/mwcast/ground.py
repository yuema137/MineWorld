"""The ground contact probe over every body the 3D client draws (PIPELINE.md §15.1).

Stages one scratch Godot project that holds the client's body code, clips, the protagonist and every
committed cast body at its own `res://` path (never importing in `clients/3d-spike`), runs
`godot/pose_sheet.gd --ground-only --dump-frames` headless at a fixed frame rate, and reads back, per body,
Godot's diagnosis and the dump of every frame it skinned: rest, the `Stand` loop, and every frame of one
`Walk` and one `Jog_Fwd` cycle. Python decides (S36-P2): G-4w per foot and frame, G-9w per full rest, stand,
walk and jog frame (`mwcast/gait.py`; rest and stand judged, walk and jog recorded, PR S36-CC10), every bound
from `contact_tolerances.json`. `--write` records each block in the
body's manifest (`character.yaml`) or, for the reference body, in the client's resource index, where
`scripts/check_cast.py` (CC-9) re-judges every body from the recorded numbers and the tolerance file's hash.

numpy and libigl are imported inside `measure`, not at the top: `cast.py`'s stdlib-only subcommands import
this module too.
"""

from __future__ import annotations

import json
import shutil
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import cast

from mwcast import pose_sheet
from mwcast.tolerances import Tolerances

PACK_CAST = Path("presentation/mineworld-default/3D/characters/cast")
CLIENT_CAST = Path("assets/characters/cast")
INDEX = CLIENT_CAST / "index.json"
TOLERANCES = "tools/character_pipeline/contact_tolerances.json"
# The fields of a ground block that are recorded; the rest of the probe's output is diagnosis.
RECORDED = (
    "rest_mm",
    "stand",
    "walk",
    "jog",
    "worst_sink_mm",
    "worst_float_mm",
    "feet",
    "shin_sink_mm",
    "arms",
    "ground_contact_lift_mm",
    "gates",
    "pass",
    "tolerances_sha256",
)
# The numbers both Godot's diagnosis and the dump give; they agree within 0.1 mm (S36-P2 commit 1).
AGREE = (
    ("rest_mm",),
    ("stand", "min_mm"),
    ("stand", "max_mm"),
    ("walk", "min_mm"),
    ("walk", "frame_low_max_mm"),
    ("jog", "min_mm"),
)
CLIPS = ("stand", "walk", "jog")


class GroundError(Exception):
    """A body cannot be staged, Godot is missing, or the run wrote no measurement."""


@dataclass(frozen=True)
class Body:
    id: str
    manifest: Path | None  # None: the reference body (the client's index names it)
    grip: str = ""  # the side a reference body's strap grip holds (its index entry's `grip`), not gated


@dataclass(frozen=True)
class Plants:
    """The generated variants of S36-P2 commit 4 (`pose_sheet.gd --plant-*`). A planted run is never
    recorded."""

    sole_offset_m: float = 0.0
    stance_lift_m: float = 0.0

    @property
    def any(self) -> bool:
        return self.sole_offset_m != 0.0 or self.stance_lift_m != 0.0

    def args(self) -> list[str]:
        out: list[str] = []
        if self.sole_offset_m:
            out += ["--plant-sole-offset", repr(self.sole_offset_m)]
        if self.stance_lift_m:
            out += ["--plant-stance-lift", repr(self.stance_lift_m)]
        return out


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
    plants: Plants | None = None,
    keep_dumps: bool = False,
    tolerances: Tolerances | None = None,
) -> dict[str, object]:
    """Run the probe and judge every body from its dump. Each dump is deleted once read, unless kept."""
    plants = plants or Plants()
    godot = pose_sheet.find_godot(environ)
    project = out / ".project"
    stage(repo_root, project, chosen)
    out.mkdir(parents=True, exist_ok=True)
    dumps = out / "dumps"
    if dumps.exists():
        shutil.rmtree(dumps)
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
        "--dump-frames",
        str(dumps.resolve()),
        *plants.args(),
    ]
    if dump_pose:
        (out / "poses").mkdir(parents=True, exist_ok=True)
        probe += ["--dump-pose", str((out / "poses").resolve())]
    pose_sheet.run_logged(probe, out / "probe.log", pose_sheet.RENDER_TIMEOUT_S * 6)
    if not result.exists():
        raise GroundError(f"Godot wrote no {result}; see {out / 'probe.log'}")
    report = cast(dict[str, object], json.loads(result.read_text()))
    tol = tolerances or Tolerances.load(repo_root / TOLERANCES)
    for row in cast(list[dict[str, object]], report["bodies"]):
        dump = dumps / f"{row['id']}.mwcf"
        measure(row, dump, tol)
        if not keep_dumps:
            dump.unlink(missing_ok=True)
    return report


def measure(row: dict[str, object], dump: Path, tolerances: Tolerances) -> None:
    """Judge one body from its dump, in place: Godot's numbers stay as diagnosis (`probe`), the dump's replace
    them, and the gates, the per-foot readings and the agreement between the two are added. A missing or
    unreadable dump fails both gates "cannot look" (I-S36-4)."""
    from mwcast import gait
    from mwcast.contact import ContactUnavailable
    from mwcast.frames import FramesError, read

    probe = {key: row.get(key) for key in ("rest_mm", *CLIPS)}
    row["probe"] = probe
    row["tolerances_sha256"] = tolerances.sha256
    row["ground_contact_lift_mm"] = {
        clip: cast(dict[str, object], probe[clip]).get("lift_mm")
        for clip in CLIPS
        if isinstance(probe.get(clip), dict)
    }
    try:
        frames = read(dump)
    except (OSError, FramesError) as exc:
        why = f"cannot look: no readable dump ({exc})"
        row["gates"] = {"G-4w": "FAIL", "G-9w": "FAIL"}
        row["reasons"] = [why]
        row["pass"] = False
        return
    ground = gait.gate_g4w(frames, tolerances)
    arm_reason = ""
    try:
        arms = gait.gate_g9w(frames, tolerances)
    except ContactUnavailable as exc:
        arms = None
        arm_reason = f"G-9w cannot look: {exc}"
    for key, value in ground.block.items():
        if key in CLIPS and isinstance(probe.get(key), dict):
            row[key] = {**cast(dict[str, object], probe[key]), **cast(dict[str, object], value)}
        else:
            row[key] = value
    reasons = list(ground.result.reasons)
    if arms is None:
        reasons.append(arm_reason)
        g9w = "FAIL"
    else:
        row.update(arms.block)
        reasons += list(arms.result.reasons)
        g9w = arms.result.verdict.value
    row["gates"] = {"G-4w": ground.result.verdict.value, "G-9w": g9w}
    row["looked_at"] = list(ground.result.looked_at) + ([] if arms is None else list(arms.result.looked_at))
    row["reasons"] = reasons
    row["pass"] = ground.result.verdict.value == "PASS" and g9w == "PASS"
    row["probe_agreement_mm"] = agreement(probe, row)


def _get(node: object, key: str) -> object:
    return cast(Mapping[str, object], node).get(key) if isinstance(node, Mapping) else None


def _at(block: Mapping[str, object], path: Sequence[str]) -> float | None:
    value: object = block
    for key in path:
        value = _get(value, key)
    return float(value) if isinstance(value, int | float) and not isinstance(value, bool) else None


def agreement(probe: Mapping[str, object], row: Mapping[str, object]) -> float | None:
    """The largest difference, mm, between Godot's reading and the dump's on the numbers both give."""
    gaps: list[float] = []
    for path in AGREE:
        a, b = _at(probe, path), _at(row, path)
        if a is not None and b is not None:
            gaps.append(abs(a - b))
    jog_a = probe.get("jog")
    jog_b = row.get("jog")
    if isinstance(jog_a, dict) and isinstance(jog_b, dict):
        fa = cast(dict[str, object], jog_a).get("foot_contact_mm")
        fb = cast(dict[str, object], jog_b).get("foot_contact_mm")
        if isinstance(fa, list) and isinstance(fb, list):
            pairs = zip(cast(list[float], fa), cast(list[float], fb), strict=False)
            gaps += [abs(float(x) - float(y)) for x, y in pairs]
    return round(max(gaps), 2) if gaps else None


def _foot(row: Mapping[str, object], side: str, clip: str) -> str:
    value = _at(row, ("feet", side, clip, "planted_float_mm"))
    return f"{value:>6.1f}" if value is not None else f"{'—':>6}"


def table(report: Mapping[str, object]) -> str:
    """One line per body, millimetres: rest; Stand lowest/highest; Walk and Jog lowest; each foot's highest
    planted reading in the walk and the jog; the gates; Godot-vs-dump agreement."""
    lines = [
        f"{'body':<18}{'rest':>7}{'st lo':>7}{'st hi':>7}{'wk lo':>7}{'wk fl':>7}{'jg lo':>7}"
        f"{'wk pL':>7}{'wk pR':>7}{'jg pL':>7}{'jg pR':>7}{'st arm':>7}{'st hnd':>7}{'agree':>7}  G-4w  G-9w"
    ]
    for row in cast(list[dict[str, object]], report["bodies"]):
        gates = cast(dict[str, str], row.get("gates", {}))

        def num(*path: str, r: Mapping[str, object] = row) -> str:
            value = _at(r, path)
            return f"{value:>7.1f}" if value is not None else f"{'—':>7}"

        lines.append(
            f"{row['id']!s:<18}{num('rest_mm')}{num('stand', 'min_mm')}{num('stand', 'max_mm')}"
            f"{num('walk', 'min_mm')}{num('walk', 'frame_low_max_mm')}{num('jog', 'min_mm')}"
            f" {_foot(row, 'left', 'walk')} {_foot(row, 'right', 'walk')} {_foot(row, 'left', 'jog')}"
            f" {_foot(row, 'right', 'jog')}{num('arms', 'stand', 'limb_mm')}{num('arms', 'stand', 'hand_mm')}"
            f"{num('probe_agreement_mm')}  {gates.get('G-4w', '—'):<5} {gates.get('G-9w', '—')}"
        )
    return "\n".join(lines)


def reasons(report: Mapping[str, object]) -> list[str]:
    """Every failing body's reasons, prefixed with its id."""
    out: list[str] = []
    for row in cast(list[dict[str, object]], report["bodies"]):
        for reason in cast(list[str], row.get("reasons", [])):
            out.append(f"{row['id']}: {reason}")
    return out


def recorded(row: Mapping[str, object], godot: str) -> dict[str, object]:
    block: dict[str, object] = {key: row[key] for key in RECORDED if key in row}
    block["tolerances"] = TOLERANCES
    block["measured_by"] = (
        "tools/character_pipeline/cast.py ground (pose_sheet.gd --ground-only --dump-frames, "
        f"mwcast/gait.py, Godot {godot})"
    )
    return block


def write(repo_root: Path, report: Mapping[str, object], chosen: Sequence[Body]) -> list[Path]:
    """Record each body's ground block: in its manifest, or in the index entry of a reference body. A planted
    run is refused: its numbers describe a generated variant, not the body."""
    plants = report.get("plants")
    if isinstance(plants, dict) and any(float(v) != 0.0 for v in cast(dict[str, float], plants).values()):
        raise GroundError(f"a planted run ({plants}) is never recorded")
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
    """A body's `stand_gates` (PIPELINE.md §15.2): the worst palm, reach and joint angles over
    every frame of the Stand loop, the misses and the verdict, as the probe measured them."""
    block = dict(cast(Mapping[str, object], row.get("stand_gates", {})))
    block["measured_by"] = (
        f"tools/character_pipeline/cast.py ground (stand_gates.gd on every Stand frame, Godot {godot})"
    )
    return block

