"""The driver of the wardrobe's Godot scripts (design §4.3 R-12 b, §4.8 W-9). Stdlib only.

A scratch Godot project (never `clients/3d-spike`) holds `fit_at_load.gd` and the harness; it is run headless
through the machine-slot wrapper (I-p-8): `<with-slot.sh> godot --headless --path <project> --script res://<s>
-- <job.json>`. Godot is `MINEWORLD_GODOT`, else `godot` on `PATH`.
"""

from __future__ import annotations

import json
import shutil
from collections.abc import Mapping
from pathlib import Path

from mwcast.wardrobe import stages

GODOT_DIR = stages.PIPELINE_ROOT / "godot" / "wardrobe"
SCRIPTS = ("fit_at_load.gd", "fit_check.gd")


def stage_project(project: Path) -> Path:
    project.mkdir(parents=True, exist_ok=True)
    (project / "project.godot").write_text(
        'config_version=5\n\n[application]\nconfig/name="s32p-wardrobe"\n', encoding="utf-8"
    )
    for name in SCRIPTS:
        shutil.copyfile(GODOT_DIR / name, project / name)
    return project


def command(slot: str, godot: str, project: Path, script: str, job: Path) -> list[str]:
    return [slot, godot, "--headless", "--path", str(project), "--script", f"res://{script}", "--", str(job)]


def run(
    script: str,
    job: Mapping[str, object],
    *,
    project: Path,
    environ: Mapping[str, str],
    executor: stages.Executor = stages.run_logged,
) -> Path:
    stage_project(project)
    job_path = project / f"{Path(script).stem}.job.json"
    job_path.write_text(json.dumps(dict(job), indent=2) + "\n", encoding="utf-8")
    log = project / f"{Path(script).stem}.log"
    godot = environ.get("MINEWORLD_GODOT", "") or "godot"
    code = executor(command(stages.slot(environ), godot, project, script, job_path), dict(environ), log)
    if code != 0:
        raise stages.StageError(f"godot {script} exited {code}; log {log}")
    return log
