"""The driver of the MPFB2 Blender stages (design §4.3; F-S32p-3; I-p-8, I-p-10). Stdlib only.

An MPFB2 stage is one Blender subprocess:

    <with-slot.sh> <blender> -b --python-exit-code 1 --python blender/mpfb/<stage>.py -- <params.json>

with `BLENDER_USER_RESOURCES=<scratch>/blender-user`, which holds only MPFB2 2.0.18 installed from its release
zip (SHA-256 checked here). It runs **without** `--factory-startup`, because the extension must load: the
documented exception of `DEP-72` item 4 to PIPELINE.md §11, for `blender/mpfb/` only. The Blender pin is still
checked (`Blender 5.2.2 LTS`). Every run goes through the machine-slot wrapper (`MINEWORLD_CAST_SLOT`, the
operator's `with-slot.sh`); without it the run is refused.

This module never imports the GPL scripts it runs.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import subprocess
from collections.abc import Callable, Mapping, Sequence
from pathlib import Path

PIPELINE_ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = PIPELINE_ROOT / "blender" / "mpfb"
REGIONS_JSON = PIPELINE_ROOT / "wardrobe" / "template" / "hm08_regions.json"
MPFB_VERSION = "2.0.18"
# P-1 (step-32 §5.1): the zip from extensions.blender.org, whose SHA-256 matched the published value.
MPFB_ZIP_SHA256 = "c075b3e9b94b8e5556a5f03f4653602d02ea306de08126e8f55ed6dee28a2dad"
SLOT_ENV = "MINEWORLD_CAST_SLOT"
USER_RESOURCES_ENV = "BLENDER_USER_RESOURCES"
STAGES = ("regions", "register", "clay", "garment_clean", "garment_bind", "assemble", "fixtures", "preview")

Executor = Callable[[Sequence[str], Mapping[str, str], Path], int]


class StageError(Exception):
    """A stage was refused, or its subprocess failed."""


def blender_user(scratch: Path) -> Path:
    return scratch / "blender-user"


def slot(environ: Mapping[str, str]) -> str:
    value = environ.get(SLOT_ENV, "")
    if not value:
        raise StageError(
            f"{SLOT_ENV} is not set: every Blender and Godot run goes through with-slot.sh (I-p-8)"
        )
    return value


def stage_environment(environ: Mapping[str, str], scratch: Path) -> dict[str, str]:
    env = dict(environ)
    env[USER_RESOURCES_ENV] = str(blender_user(scratch))
    return env


def check_zip(path: Path) -> None:
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    if digest != MPFB_ZIP_SHA256:
        raise StageError(
            f"{path}: SHA-256 {digest}; MPFB2 {MPFB_VERSION}'s published zip is {MPFB_ZIP_SHA256}"
        )


def install_command(slot_path: str, blender: str, zip_path: Path) -> list[str]:
    return [
        slot_path,
        blender,
        "-b",
        "--command",
        "extension",
        "install-file",
        "-r",
        "user_default",
        "-e",
        str(zip_path),
    ]


def installed_version(scratch: Path) -> str | None:
    manifest = blender_user(scratch) / "extensions" / "user_default" / "mpfb" / "blender_manifest.toml"
    if not manifest.is_file():
        return None
    match = re.search(r'^version\s*=\s*"([^"]+)"', manifest.read_text(encoding="utf-8"), re.MULTILINE)
    return match.group(1) if match else None


def require_installed(scratch: Path) -> None:
    version = installed_version(scratch)
    if version != MPFB_VERSION:
        raise StageError(
            f"MPFB2 {MPFB_VERSION} is not installed in {blender_user(scratch)} (found {version}); "
            "run `cast.py wardrobe install-mpfb --zip <mpfb-2.0.18.zip>` first"
        )


def stage_command(slot_path: str, blender: str, stage: str, params: Path) -> list[str]:
    if stage not in STAGES:
        raise StageError(f"unknown stage {stage!r}")
    script = SCRIPTS / f"{stage}.py"
    return [slot_path, blender, "-b", "--python-exit-code", "1", "--python", str(script), "--", str(params)]


def run_logged(args: Sequence[str], env: Mapping[str, str], log: Path) -> int:
    """The real executor: the command's output goes to `log`; the exit code is returned."""
    log.parent.mkdir(parents=True, exist_ok=True)
    with log.open("w", encoding="utf-8") as handle:
        return subprocess.run(
            list(args), env=dict(env), stdout=handle, stderr=subprocess.STDOUT, check=False
        ).returncode


def run_stage(
    stage: str,
    params: Mapping[str, object],
    *,
    scratch: Path,
    run_dir: Path,
    environ: Mapping[str, str],
    blender: str,
    executor: Executor = run_logged,
    label: str | None = None,
) -> Path:
    """Write `params.json`, run the stage through the slot wrapper and return its log; non-zero refuses.
    `label` names the params and log files when one stage runs more than once (the assembly's phases)."""
    require_installed(scratch)
    run_dir.mkdir(parents=True, exist_ok=True)
    name = label or stage
    params_path = run_dir / f"{name}.params.json"
    params_path.write_text(json.dumps(dict(params), indent=2, sort_keys=True) + "\n", encoding="utf-8")
    log = run_dir / f"{name}.log"
    args = stage_command(slot(environ), blender, stage, params_path)
    code = executor(args, stage_environment(environ, scratch), log)
    if code != 0:
        raise StageError(f"stage {stage} exited {code}; log {log}")
    return log


def default_blender(environ: Mapping[str, str]) -> str:
    return environ.get("MINEWORLD_BLENDER", "") or "blender"


def environment() -> Mapping[str, str]:
    return os.environ
