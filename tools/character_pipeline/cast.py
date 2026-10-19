#!/usr/bin/env python3
"""Launcher for the character pipeline: `python3 tools/character_pipeline/cast.py <subcommand>`.

Puts this directory on `sys.path` so that `mwcast` is importable without installing anything.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from mwcast.cli import main  # noqa: E402  (imported after sys.path is set)

if __name__ == "__main__":
    raise SystemExit(main())
