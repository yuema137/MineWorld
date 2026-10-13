"""AP-11: without the server binary, the real-server tests are an error that names the build — never a
skip that reads as green (D-P3-10).

Run as a child pytest with `MINEWORLD_BIN` pointing at nothing, so this test needs no binary itself and
runs in every job, the Windows smoke job included.
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

TESTS = Path(__file__).resolve().parent


def test_a_missing_binary_is_an_error_naming_the_build_never_a_skip() -> None:
    environment = dict(os.environ)
    environment["MINEWORLD_BIN"] = str(TESTS / "no-such-mineworld")
    child = subprocess.run(
        [
            sys.executable,
            "-m",
            "pytest",
            str(TESTS / "test_real_server.py"),
            "-p",
            "no:cacheprovider",
            "-q",
        ],
        cwd=TESTS.parent,
        env=environment,
        capture_output=True,
        text=True,
        encoding="utf-8",
        timeout=120,
    )
    summary = child.stdout.strip().splitlines()[-1]
    assert "error" in summary and "skipped" not in summary and "passed" not in summary, summary
    assert "cargo build -p mineworld-cli" in child.stdout, child.stdout
