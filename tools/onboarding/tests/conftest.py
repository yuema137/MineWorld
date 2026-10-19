"""Shared fixtures: the harness's modules on the import path, and the `mineworld` binary the scorer runs."""

from __future__ import annotations

import os
import sys
from pathlib import Path

import pytest

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))

FIXTURES = HERE / "fixtures"


@pytest.fixture
def binary() -> Path:
    """The binary under test. Missing, the test fails rather than skips: `ci_layer.py` builds it first."""
    name = "mineworld.exe" if sys.platform == "win32" else "mineworld"
    path = Path(os.environ.get("MINEWORLD_BIN", HERE.parents[2] / "target" / "debug" / name))
    assert path.is_file(), f"build the binary first: {path}"
    return path
