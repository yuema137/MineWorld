"""A client's project settings under the shared runtime (step-23 PR S23-RT §§3, 4.3, 4.4; ARC-80).

The release bundle's one engine starts `clients/runtime/` and loads a client's pack at run time, which does not
re-read the client's `project.godot`. So every key a client sets must be one of:

* a **start-up** setting whose value equals the runtime's own `project.godot` (the engine reads it once);
* a **run-time** setting the runtime's boot script applies (`clients/runtime/runtime/appliers.json`); its value is
  written to `<pack>.settings.json` beside the client's pack;
* **ignored**: metadata or import-only (the same file's `ignored` list).

Anything else fails the packaging, naming the key, so a client cannot ship a setting the runtime would drop.
Standard library only; imported by `scripts/package.py`.
"""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any, Union

Json = Union[bool, int, float, str, "list[float]"]

RUNTIME = "clients/runtime"
APPLIERS = "clients/runtime/runtime/appliers.json"
COLOR = re.compile(r"^Color\(([^)]*)\)$")
SECTION = re.compile(r"^\[([^\]]+)\]$")


def parse_project(text: str) -> dict[str, str]:
    """`project.godot` as {"<section>/<key>": raw value}. A value may span lines (an input action's
    dictionary); it ends where its brackets balance. Comments (`;`) and blank lines are skipped."""
    out: dict[str, str] = {}
    section = ""
    key, depth = "", 0
    value: list[str] = []
    for line in text.splitlines():
        if depth:
            value.append(line)
            depth += _depth(line)
            if depth <= 0:
                out[key] = "\n".join(value)
                depth = 0
            continue
        stripped = line.strip()
        if not stripped or stripped.startswith(";"):
            continue
        if m := SECTION.match(stripped):
            section = m.group(1)
            continue
        name, _, raw = line.partition("=")
        key = f"{section}/{name.strip()}" if section else name.strip()
        depth = _depth(raw)
        if depth > 0:
            value = [raw]
        else:
            out[key] = raw.strip()
            depth = 0
    return out


def _depth(text: str) -> int:
    """Open minus closed brackets outside quoted strings."""
    depth, quoted = 0, False
    for i, ch in enumerate(text):
        if ch == '"' and (i == 0 or text[i - 1] != "\\"):
            quoted = not quoted
        elif not quoted:
            depth += (ch in "{[(") - (ch in "}])")
    return depth


def to_json(raw: str) -> Json:
    """A `project.godot` scalar as JSON: bool, int, float, string, or `Color(r, g, b, a)` as four numbers."""
    if raw in ("true", "false"):
        return raw == "true"
    if raw.startswith('"') and raw.endswith('"'):
        text: str = json.loads(raw)
        return text
    if m := COLOR.match(raw):
        parts = [float(x) for x in m.group(1).split(",")]
        return parts + [1.0] * (4 - len(parts))
    try:
        return int(raw)
    except ValueError:
        pass
    try:
        return float(raw)
    except ValueError:
        raise ValueError(f"cannot convert {raw!r}") from None


def judge(
    client_text: str, runtime_text: str, appliers: dict[str, Any]
) -> tuple[list[str], dict[str, Json]]:
    """(problems, the client's run-time settings) for one client's `project.godot` text."""
    runtime = parse_project(runtime_text)
    run_time: dict[str, str] = appliers["run_time"]
    ignored: set[str] = set(appliers["ignored"])
    problems: list[str] = []
    settings: dict[str, Json] = {}
    for key, raw in parse_project(client_text).items():
        if key in ignored:
            continue
        if key in run_time:
            try:
                settings[key] = to_json(raw)
            except ValueError as error:
                problems.append(f"{key}: {error}")
        elif key in runtime and runtime[key] == raw:
            continue
        elif key in runtime:
            problems.append(
                f"{key}: {raw!r} differs from the shared runtime's {runtime[key]!r}"
            )
        else:
            problems.append(
                f"{key}: not supported by the shared runtime (step-23 PR S23-RT §3)"
            )
    return problems, settings


def client_settings(root: Path, project: str) -> dict[str, Json]:
    """The run-time settings of `<project>/project.godot`; raises ValueError naming every problem."""
    appliers: dict[str, Any] = json.loads((root / APPLIERS).read_text(encoding="utf-8"))
    problems, settings = judge(
        (root / project / "project.godot").read_text(encoding="utf-8"),
        (root / RUNTIME / "project.godot").read_text(encoding="utf-8"),
        appliers,
    )
    if problems:
        raise ValueError("\n  ".join(f"{project}/project.godot: {p}" for p in problems))
    return settings
