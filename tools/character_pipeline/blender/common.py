# SPDX-License-Identifier: GPL-2.0-or-later
"""Shared helpers for the Blender stages of the character pipeline (PIPELINE.md §2.2–2.3).

These scripts run inside Blender's Python, so they import `bpy` and friends. They are GPL-2.0-or-later and
live only in this directory; the MIT driver never imports them, it only runs them as subprocesses.

Every stage reads its parameters from the JSON file after `--` (`params.json`, written by the driver). Nothing
here, and nothing in a stage, types a number that belongs to one character (I-B1-9).
"""

import json
import sys
from typing import Any

# The export settings of the D+ scripts (A-B1-8, `rig_meshy.py:259-262`), verbatim in substance: each stage's
# export calls `bpy.ops.export_scene.gltf(filepath=..., **EXPORT_SETTINGS)`.
EXPORT_SETTINGS: dict[str, Any] = {
    "export_format": "GLB",
    "use_selection": True,
    "export_yup": True,
    "export_animations": False,
    "export_morph": False,
    "export_skins": True,
    "export_materials": "EXPORT",
    "export_texcoords": True,
    "export_normals": True,
    "export_extras": True,
}


def params() -> dict[str, Any]:
    """The parameters the driver wrote: the JSON file named after `--`."""
    argv = sys.argv[sys.argv.index("--") + 1 :]
    with open(argv[0], encoding="utf-8") as handle:
        loaded = json.load(handle)
    if not isinstance(loaded, dict):
        raise SystemExit("params file is not a JSON object")
    return loaded


def write_result(name: str, data: dict[str, Any]) -> None:
    """Write `<run_dir>/<name>.json` and print each item, as the D+ scripts printed their numbers."""
    import os

    run_dir = params()["out_dir"]
    os.makedirs(run_dir, exist_ok=True)
    with open(os.path.join(run_dir, name + ".json"), "w", encoding="utf-8") as handle:
        json.dump(data, handle, indent=1, sort_keys=True)
    for key in sorted(data):
        print(f"{name}.{key} = {data[key]}")


def set_provenance(block: dict[str, Any]) -> None:
    """Store the provenance block on the scene (stage 10 writes it into the glTF extras)."""
    import bpy

    bpy.context.scene["mineworld_provenance"] = block
