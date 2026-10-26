"""Shared fixtures: the tiler on the import path, the repository's schemas, and the `mineworld` binary."""

from __future__ import annotations

import os
import sys
from pathlib import Path

import pytest

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))

REPOSITORY = HERE.parents[2]
SCHEMAS = REPOSITORY / "docs" / "reference" / "schema"
WORLDS = REPOSITORY / "worlds"


@pytest.fixture
def binary() -> Path:
    """The binary under test. Missing, the test fails rather than skips: `ci_layer.py` builds it first."""
    name = "mineworld.exe" if sys.platform == "win32" else "mineworld"
    path = Path(os.environ.get("MINEWORLD_BIN", REPOSITORY / "target" / "debug" / name))
    assert path.is_file(), f"build the binary first: {path}"
    return path
