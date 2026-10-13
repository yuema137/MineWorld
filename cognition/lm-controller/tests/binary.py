"""The real `mineworld` binary, for the `real_binary` tests: found, never built, and never skipped.

As `sdk/python/tests/realserver.py`: `MINEWORLD_BIN`, or the debug build under the repository's
`target/`, with the platform's executable suffix. A missing binary fails the test (D-P3-10).
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

REPOSITORY = Path(__file__).resolve().parents[3]
BUILD = "cargo build -p mineworld-cli"


def binary() -> Path:
    named = os.environ.get("MINEWORLD_BIN")
    if named:
        return Path(named)
    suffix = ".exe" if sys.platform == "win32" else ""
    return REPOSITORY / "target" / "debug" / f"mineworld{suffix}"


def mineworld(*arguments: str | Path) -> str:
    """Runs the binary from the repository root with an argument list (never a shell) and returns its
    standard output; a failure raises with the binary's own error output."""
    executable = binary()
    if not executable.is_file():
        raise FileNotFoundError(
            f"the mineworld binary is not at {executable}. Build it first with `{BUILD}` "
            "(or set MINEWORLD_BIN); this test runs the real binary and never skips"
        )
    result = subprocess.run(
        [str(executable), *(str(argument) for argument in arguments)],
        cwd=REPOSITORY,
        capture_output=True,
        encoding="utf-8",
        timeout=600,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"mineworld {' '.join(map(str, arguments))} exited {result.returncode}: {result.stderr}"
        )
    return result.stdout
