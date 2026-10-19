"""The key-shape list of the pipeline's G-14 and the repository's secrets scan are one rule (AC-S26-9).

The scanner is a script under `scripts/`, so its literal is read with `ast`, not imported.
"""

from __future__ import annotations

import ast
from pathlib import Path
from typing import cast

from mwcast import standard

SCANNER = Path(__file__).resolve().parents[3] / "scripts" / "check_secrets.py"


def _scanner_patterns() -> list[str]:
    tree = ast.parse(SCANNER.read_text(encoding="utf-8"))
    for node in tree.body:
        if (
            isinstance(node, ast.AnnAssign)
            and isinstance(node.target, ast.Name)
            and node.target.id == "PATTERNS"
        ):
            assert node.value is not None
            value: object = ast.literal_eval(node.value)
            assert isinstance(value, dict)
            patterns = cast(dict[str, str], value)
            return sorted(patterns.values())
    raise AssertionError("scripts/check_secrets.py has no PATTERNS literal")


def test_the_pipeline_and_the_scanner_share_one_key_shape_list() -> None:
    assert sorted(standard.KEY_PATTERNS) == _scanner_patterns()
