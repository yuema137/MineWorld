"""Finding Blender, pinning its version, and building its command line (PIPELINE.md §3, §11; design §5.2).

Blender runs only as a subprocess with an argument list, never through a shell. The version pin is
`standard.BLENDER_PIN`; another version is refused unless the operator names its first line with
`--allow-blender`, and then the run is a new parity baseline, never evidence of reproduction.
"""

from __future__ import annotations

import shutil
import subprocess
from collections.abc import Callable, Mapping, Sequence
from pathlib import Path

from mwcast import standard

BLENDER_ENV = "MINEWORLD_BLENDER"
MACOS_APPLICATION = Path("/Applications/Blender.app/Contents/MacOS/Blender")


class BlenderError(Exception):
    """Blender cannot be found, or it is not the pinned version."""


def find_blender(
    environ: Mapping[str, str],
    platform_name: str,
    which: Callable[[str], str | None] = shutil.which,
) -> Path:
    """`MINEWORLD_BLENDER`, else `blender` on PATH, else the macOS application. Refuses when none is found."""
    explicit = environ.get(BLENDER_ENV, "")
    if explicit:
        return Path(explicit)
    on_path = which("blender")
    if on_path:
        return Path(on_path)
    if platform_name == "darwin" and MACOS_APPLICATION.exists():
        return MACOS_APPLICATION
    raise BlenderError(
        f"no Blender found: set {BLENDER_ENV}, put blender on PATH, or install {standard.BLENDER_PIN}"
    )


def read_version_line(blender: Path) -> str:
    """The first line of `blender --version`, read through a subprocess argument list."""
    result = subprocess.run(
        [str(blender), "--version"],
        capture_output=True,
        text=True,
        check=False,
    )
    lines = result.stdout.strip().splitlines()
    if not lines:
        raise BlenderError(f"{blender} --version printed nothing")
    return lines[0].strip()


def check_pin(version: str, allow: str | None) -> bool:
    """Accept the pinned version, or the one the operator named. Returns True when the named version was used
    (a new parity baseline, to be recorded). Refuses any other version."""
    if version == standard.BLENDER_PIN:
        return False
    if allow is not None and version == allow:
        return True
    raise BlenderError(
        f"Blender reports {version!r}; the pin is {standard.BLENDER_PIN!r}. "
        "Pass --allow-blender with that first line to make this a new parity baseline."
    )


def command(blender: Path, script: Path, params: Path, in_blend: Path | None) -> list[str]:
    """The argument list of one stage (PIPELINE.md §2.2)."""
    args = [str(blender), "--background", "--factory-startup"]
    if in_blend is not None:
        args.append(str(in_blend))
    args.extend(["--python-exit-code", "1", "--python", str(script), "--", str(params)])
    return args


def run_command(args: Sequence[str]) -> subprocess.CompletedProcess[str]:
    """Run one stage's argument list. Used by the driver; tests inject a fake instead."""
    return subprocess.run(list(args), capture_output=True, text=True, check=False)
