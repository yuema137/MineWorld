"""The stage planner and runner: Blender subprocesses, parameter files, and the run manifest (PIPELINE.md §3).

Stages 6–10 and the review frames run in the order of `STAGES`. Each Blender stage is one subprocess with an
argument list. The driver writes every parameter the script reads into `params/<tag>.json` (nothing from the
environment), records each stage's log, and stops at the first non-zero exit, naming the stage and its log.
Before stage 6 it checks the download's SHA-256 against the fit; a mismatch stops the run before any
subprocess.

The executor is injected, so that tests can record the commands without running Blender.
"""

from __future__ import annotations

import hashlib
import json
import shutil
from collections.abc import Mapping, Sequence
from dataclasses import dataclass, field
from pathlib import Path
from typing import Protocol, cast

from mwcast import blender as blender_cli
from mwcast import standard
from mwcast.records import FitParams

PIPELINE_VERSION = "mwcast-1"
STAGES: tuple[str, ...] = (
    "gameready",
    "fit_bind",
    "weights_fingers",
    "weights_pack",
    "flecks",
    "export",
    "render_bones",
    "dump_rig",
)
SCRIPTS: dict[str, str] = {
    "gameready": "gameready.py",
    "fit_bind": "fit_bind.py",
    "weights_fingers": "weights_fingers.py",
    "weights_pack": "weights_pack.py",
    "flecks": "flecks.py",
    "export": "export.py",
    "render_bones": "render_bones.py",
    "dump_rig": "dump_rig.py",
}
# Each stage's own output blend, read by the next stage (PIPELINE.md §2.1).
BLEND_OF: dict[str, str] = {
    "gameready": "gameready.blend",
    "fit_bind": "fit_bind.blend",
    "weights_fingers": "weights_fingers.blend",
    "weights_pack": "weights_pack.blend",
    "flecks": "flecks.blend",
}
REPO_ROOT = Path(__file__).resolve().parents[3]


class PipelineError(Exception):
    """A stage could not run, or a precondition failed. The message names the stage, the file or the check."""


@dataclass(frozen=True)
class Plan:
    stages: tuple[str, ...]
    skipped: tuple[tuple[str, str], ...]


def plan(fit: FitParams, start: str | None = None, stop: str | None = None) -> Plan:
    """The stages to run for this fit, from `start` to `stop` (both inclusive), with each skip's reason."""
    skipped: list[tuple[str, str]] = []
    if "backpack" not in fit.accessories:
        skipped.append(("weights_pack", "no pack in fit.accessories"))
    if not fit.flecks.enabled:
        skipped.append(("flecks", "flecks.enabled is false"))
    skip_names = {name for name, _ in skipped}
    names = [s for s in STAGES if s not in skip_names]
    for bound in (start, stop):
        if bound is None:
            continue
        if bound not in STAGES:
            raise PipelineError(f"unknown stage {bound!r}; the stages are {', '.join(STAGES)}")
        if bound in skip_names:
            raise PipelineError(f"stage {bound!r} is skipped for this fit; choose another bound")
    if start is not None:
        names = names[names.index(start) :]
    if stop is not None:
        names = names[: names.index(stop) + 1]
    return Plan(stages=tuple(names), skipped=tuple(skipped))


@dataclass(frozen=True)
class GameReady:
    """The numbers stage 6 decided, read from `gameready.json` (PIPELINE.md §2.1)."""

    scale: float
    sole_raw: float
    triangles: int
    bake_device: str


def read_gameready(path: Path) -> GameReady:
    raw: object = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(raw, dict):
        raise PipelineError(f"{path}: not an object")
    data = cast(dict[str, object], raw)

    def number(key: str) -> float:
        value = data.get(key)
        if isinstance(value, bool) or not isinstance(value, int | float):
            raise PipelineError(f"{path}: '{key}' is not a number")
        return float(value)

    device = data.get("bake_device")
    if not isinstance(device, str):
        raise PipelineError(f"{path}: 'bake_device' is not a string")
    return GameReady(
        scale=number("scale"),
        sole_raw=number("sole_raw"),
        triangles=int(number("triangles")),
        bake_device=device,
    )


# The CharMorph mesh whose front surface the centre points read, in `charmorph_body` (D+ rig_meshy.py:99).
CM_FRONT_MESH = "cm_vitruvian"

# The precision at which stage 6's transform reaches the fit (F-B1-B10-3). D+ handed the scale and the raw
# sole to rig_meshy.py as gameready_mesh.py printed them, `f"{s:.4f}"` and `f"{sole:.4f}"`
# (gameready_mesh.py:45), typed into rig_meshy.py:26 as 0.9563 and -0.9516. The full-precision values move
# every joint by about 2.6e-5 m, which moves one pack-core vertex across its bound and changes the heat
# weights by up to 0.18. A method constant, not a per-character value: gameready.json keeps the full values,
# and only the fit_bind parameters are rounded.
GAMEREADY_HANDOFF_DECIMALS = 4


def _pack_core(fit: FitParams) -> dict[str, float]:
    core = fit.weights.pack_core
    return {
        "behind_spine1_m": core.behind_spine1_m,
        "above_hips_m": core.above_hips_m,
        "above_spine2_m": core.above_spine2_m,
        "half_width_m": core.half_width_m,
    }


def stage_params(
    stage: str,
    fit: FitParams,
    *,
    repo_root: Path,
    run_dir: Path,
    download_file: Path,
    gameready: GameReady | None,
    provenance: Mapping[str, object] | None,
) -> dict[str, object]:
    """Every value one stage reads, from the fit and the run. Nothing comes from the environment."""
    out = run_dir / stage
    g = fit.gameready
    w = fit.weights
    if stage == "gameready":
        return {
            "download_glb": str(download_file),
            "out_dir": str(out),
            "target_triangles": g.target_triangles,
            "eye_z_raw": g.eye_z_raw,
            "sole_z_m": g.sole_z_m,
            "eye_from_sole_m": g.eye_from_sole_m,
            "centre_band_z_raw": g.centre_band_z_raw,
            "centre_band_half_raw": g.centre_band_half_raw,
            "weld_m": g.weld_m,
            "bake": {
                "size_px": g.bake.size_px,
                "samples": g.bake.samples,
                "margin_px": g.bake.margin_px,
                "extrusion_m": g.bake.extrusion_m,
                "ray_m": g.bake.ray_m,
                "device": g.bake.device,
            },
        }
    if gameready is None:
        raise PipelineError(f"stage {stage} needs gameready.json; the gameready stage has not run here")
    if stage == "fit_bind":
        return {
            "rig_source": str((repo_root / fit.fit.rig_source).resolve()),
            "out_dir": str(out),
            "scale": round(gameready.scale, GAMEREADY_HANDOFF_DECIMALS),
            "sole_raw": round(gameready.sole_raw, GAMEREADY_HANDOFF_DECIMALS),
            "sole_z_m": g.sole_z_m,
            "picks": {key: list(value) for key, value in fit.fit.picks.items()},
            "px_per_raw_unit": fit.fit.px_per_raw_unit,
            "charmorph_landmarks_z": list(fit.fit.charmorph_landmarks_z),
            "charmorph_body": str((repo_root / fit.fit.charmorph_body).resolve()),
            "cm_front_mesh": CM_FRONT_MESH,
            "head_rigid": {
                "above_chin_m": w.head_rigid.above_chin_m,
                "half_width_m": w.head_rigid.half_width_m,
            },
            "pack_core": _pack_core(fit),
            "smooth": {"factor": w.smooth.factor, "repeat": w.smooth.repeat, "limit": w.smooth.limit},
            "accessories": list(fit.accessories),
        }
    if stage == "weights_fingers":
        return {
            "out_dir": str(out),
            "hand_share_min": w.fingers.hand_share_min,
            "power": w.fingers.power,
            "keep": w.fingers.keep,
        }
    if stage == "weights_pack":
        panels = w.pack_panels
        return {
            "out_dir": str(out),
            "x_in": panels.x_in,
            "x_out": panels.x_out,
            "arm_zero": panels.arm_zero,
            "smooth": panels.smooth,
            "smooth_repeat": panels.smooth_repeat,
            "sleeve_share_max": panels.sleeve_share_max,
            "pack_core": _pack_core(fit),
        }
    if stage == "flecks":
        return {
            "out_dir": str(out),
            "meshy_texture": str(download_file.parent / "tex0_base_color.png"),
            "z_min_m": fit.flecks.z_min_m,
            "dark": fit.flecks.dark,
            "lighter": fit.flecks.lighter,
        }
    if stage == "export":
        return {
            "character": fit.character,
            "out_dir": str(out),
            "material": fit.export.material,
            "provenance": dict(provenance or {}),
        }
    if stage == "render_bones":
        return {"out_prefix": str(out / "frames" / "rig"), "pose": {}}
    raise PipelineError(f"no parameters are defined for stage {stage}")


def meshy_generator_string(download_dir: Path) -> str:
    """The Meshy model and pose mode, read from the download's `request.json` (PIPELINE.md §8)."""
    raw: object = json.loads((download_dir / "request.json").read_text(encoding="utf-8"))
    if not isinstance(raw, dict):
        raise PipelineError("request.json is not an object")
    params = cast(dict[str, object], raw).get("params")
    if not isinstance(params, dict):
        raise PipelineError("request.json has no params object")
    values = cast(dict[str, object], params)
    model = values.get("ai_model")
    pose = values.get("pose_mode")
    if not isinstance(model, str) or not isinstance(pose, str):
        raise PipelineError("request.json params lack ai_model or pose_mode")
    return f"Meshy image-to-3D, ai_model {model}, pose_mode {pose}, paid plan"


def provenance_block(
    fit: FitParams,
    *,
    fit_sha256: str,
    generator: str,
    triangles: int,
    blender_version: str,
) -> dict[str, object]:
    """The `mineworld_provenance` block of PIPELINE.md §8."""
    return {
        "source_type": "generated",
        "ai_generated": True,
        "generator": generator,
        "meshy_task": fit.source.meshy_task,
        "references": [{"path": fit.source.reference.path, "sha256": fit.source.reference.sha256}],
        "reference_generator": fit.source.reference.generator,
        "input_sha256": fit.source.input_sha256,
        "postprocess": (
            f"scaled to stature; welded; decimated to {triangles} triangles; base colour re-baked; "
            "fitted on the CharMorph 52-joint skeleton; heat weights with head, pack and finger fixes"
        ),
        "pipeline": {"version": PIPELINE_VERSION, "fit_sha256": fit_sha256, "blender": blender_version},
    }


def check_stored_once(export_dir: Path, character: str) -> None:
    """The stored-once rule of PIPELINE.md §9: the separate export's only image is `<id>_base_color.png`, and
    that file exists beside `<id>.gltf`. Godot extracts a texture once under the name the glTF gives it, so
    any other name (Blender's default is the bake's file name, `base_color.png`) breaks the rule."""
    expected = f"{character}_base_color.png"
    gltf = export_dir / f"{character}.gltf"
    raw: object = json.loads(gltf.read_text(encoding="utf-8"))
    if not isinstance(raw, dict):
        raise PipelineError(f"{gltf}: not an object")
    images = cast(dict[str, object], raw).get("images")
    uris: list[object] = []
    if isinstance(images, list):
        for image in cast(list[object], images):
            uris.append(cast(dict[str, object], image).get("uri") if isinstance(image, dict) else None)
    if uris != [expected]:
        raise PipelineError(f"{gltf}: image uris {uris}; the stored-once rule needs ['{expected}']")
    if not (export_dir / expected).is_file():
        raise PipelineError(f"{export_dir / expected} is missing")


class ExecResult(Protocol):
    returncode: int
    stdout: str
    stderr: str


class Executor(Protocol):
    def __call__(self, args: Sequence[str]) -> ExecResult: ...


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


@dataclass
class RunResult:
    run_dir: Path
    stages_run: list[str] = field(default_factory=list[str])
    skipped: tuple[tuple[str, str], ...] = ()
    manifest: Path | None = None


def _previous_blend(run_dir: Path, stage: str) -> Path | None:
    """The most recent stage blend before `stage` that exists in this run (a `--from` run reuses it)."""
    index = STAGES.index(stage)
    for earlier in reversed(STAGES[:index]):
        if earlier in BLEND_OF:
            candidate = run_dir / earlier / BLEND_OF[earlier]
            if candidate.exists():
                return candidate
    return None


def run(
    *,
    fit: FitParams,
    fit_path: Path,
    download_dir: Path,
    scratch: Path,
    run_id: str,
    start: str | None,
    stop: str | None,
    blender: Path,
    blender_version: str,
    allowed_non_pin: bool,
    executor: Executor,
    repo_root: Path = REPO_ROOT,
) -> RunResult:
    """Run stages 6–10, the frames and the armature dumps for one fit. Stops before any subprocess when the
    download's SHA-256 does not match the fit."""
    download_file = download_dir / fit.source.download_file
    if not download_file.is_file():
        raise PipelineError(f"download not found: {download_file}")
    download_sha = _sha256(download_file)
    if download_sha != fit.source.download_sha256:
        raise PipelineError(
            f"the download's SHA-256 is {download_sha}; fit.source.download_sha256 is "
            f"{fit.source.download_sha256}; no stage was run"
        )
    fit_sha = _sha256(fit_path)
    run_dir = scratch / fit.character / run_id
    (run_dir / "params").mkdir(parents=True, exist_ok=True)
    stage_plan = plan(fit, start, stop)
    result = RunResult(run_dir=run_dir, skipped=stage_plan.skipped)
    gameready: GameReady | None = None
    if "gameready" not in stage_plan.stages and (run_dir / "gameready.json").exists():
        gameready = read_gameready(run_dir / "gameready.json")
    provenance: dict[str, object] | None = None
    for stage in stage_plan.stages:
        if stage == "export" and gameready is not None:
            provenance = provenance_block(
                fit,
                fit_sha256=fit_sha,
                generator=meshy_generator_string(download_dir),
                triangles=gameready.triangles,
                blender_version=blender_version,
            )
        for job in _stage_jobs(
            stage,
            fit,
            repo_root=repo_root,
            run_dir=run_dir,
            download_file=download_file,
            gameready=gameready,
            provenance=provenance,
        ):
            params_path = run_dir / "params" / f"{job.tag}.json"
            params_path.write_text(json.dumps(job.params, indent=1, sort_keys=True), encoding="utf-8")
            script = Path(repo_root) / "tools/character_pipeline/blender" / SCRIPTS[stage]
            args = blender_cli.command(blender, script, params_path, job.input_blend)
            outcome = executor(args)
            log = run_dir / f"{job.tag}.log"
            log.write_text(outcome.stdout + outcome.stderr, encoding="utf-8")
            if outcome.returncode != 0:
                raise PipelineError(f"stage {stage} exited {outcome.returncode}; its log is {log}")
        result.stages_run.append(stage)
        if stage == "gameready":
            gameready = read_gameready(run_dir / "gameready" / "gameready.json")
            shutil.copy2(run_dir / "gameready" / "gameready.json", run_dir / "gameready.json")
        if stage == "export":
            check_stored_once(run_dir / "export", fit.character)
        if stage == "dump_rig":
            shutil.copy2(run_dir / "dump_rig" / "dump_rig.json", run_dir / "dump_rig.json")
            shutil.copy2(run_dir / "reference" / "dump_rig.json", run_dir / "reference_dump_rig.json")
    result.manifest = _write_manifest(
        run_dir,
        fit=fit,
        fit_sha256=fit_sha,
        download_sha256=download_sha,
        blender_version=blender_version,
        allowed_non_pin=allowed_non_pin,
        gameready=gameready,
        stages_run=result.stages_run,
        skipped=stage_plan.skipped,
    )
    return result


@dataclass(frozen=True)
class Job:
    """One Blender subprocess.

    The tag names the params file and the log. `input_blend` is None when the script opens nothing (stage 6)
    or reads a file of its own (`dump_rig`)."""

    tag: str
    params: Mapping[str, object]
    input_blend: Path | None


def _stage_jobs(
    stage: str,
    fit: FitParams,
    *,
    repo_root: Path,
    run_dir: Path,
    download_file: Path,
    gameready: GameReady | None,
    provenance: Mapping[str, object] | None,
) -> list[Job]:
    """The subprocesses of one stage.

    `dump_rig` runs twice: the reference skeleton (G-5's reference rolls) and the export (the candidate's
    rolls).
    Every other stage is one job."""
    if stage == "dump_rig":
        reference = (repo_root / fit.fit.rig_source).resolve()
        return [
            Job("dump_rig_reference", {"glb": str(reference), "out_dir": str(run_dir / "reference")}, None),
            Job(
                "dump_rig",
                {
                    "glb": str(run_dir / "export" / f"{fit.character}.glb"),
                    "out_dir": str(run_dir / "dump_rig"),
                },
                None,
            ),
        ]
    params = stage_params(
        stage,
        fit,
        repo_root=repo_root,
        run_dir=run_dir,
        download_file=download_file,
        gameready=gameready,
        provenance=provenance,
    )
    input_blend = None if stage == "gameready" else _previous_blend(run_dir, stage)
    return [Job(stage, params, input_blend)]


def _write_manifest(
    run_dir: Path,
    *,
    fit: FitParams,
    fit_sha256: str,
    download_sha256: str,
    blender_version: str,
    allowed_non_pin: bool,
    gameready: GameReady | None,
    stages_run: list[str],
    skipped: tuple[tuple[str, str], ...],
) -> Path:
    outputs: dict[str, dict[str, str]] = {}
    for stage in STAGES:
        stage_dir = run_dir / stage
        if stage_dir.is_dir():
            outputs[stage] = {
                str(path.relative_to(run_dir)): _sha256(path)
                for path in sorted(stage_dir.rglob("*"))
                if path.is_file() and path.suffix != ".log"
            }
    manifest = {
        "pipeline": PIPELINE_VERSION,
        "character": fit.character,
        "pin": standard.BLENDER_PIN,
        "blender": blender_version,
        "allowed_non_pin": allowed_non_pin,
        "bake_device": gameready.bake_device if gameready else None,
        "inputs": {"fit_sha256": fit_sha256, "download_sha256": download_sha256},
        "stages_run": stages_run,
        "skipped": [{"stage": s, "reason": r} for s, r in skipped],
        "outputs": outputs,
    }
    path = run_dir / "run_manifest.json"
    path.write_text(json.dumps(manifest, indent=1, sort_keys=True), encoding="utf-8")
    return path
