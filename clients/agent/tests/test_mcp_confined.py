"""I-d-1 and I-d-5 (INV-S27-9, `ARC-96`): `mcp` is imported by one module of the member and is a dependency of
the member alone, and the adapter reaches the world only through `Player`.

S27-d C1's two SDK prototypes (P-1, P-2) were deleted by pr-s27d1 (R-6): they tested only the `mcp` SDK. What the
adapter relies on from them is checked through the adapter: P-1 by `test_the_published_schemas_are_the_models`,
P-2 by every row of `test_mcp_errors.py`.

Replaces S27-c's `test_no_mcp.py`, which forbade `mcp` everywhere; the scan now allows it in exactly one module.
"""

from __future__ import annotations

import ast
import os
import re
import tomllib
from pathlib import Path
from typing import cast

MEMBER = Path(__file__).resolve().parents[1]
REPO = Path(__file__).resolve().parents[3]
SOURCE = MEMBER / "src" / "mineworld_agent"
ADAPTER = SOURCE / "mcp_server.py"
PRUNED = {".git", ".venv", "target", "node_modules", "__pycache__"}
_NAME = re.compile(r"^\s*([A-Za-z0-9][A-Za-z0-9._-]*)")


def imported(source: Path) -> list[str]:
    tree = ast.parse(source.read_text(encoding="utf-8"), filename=str(source))
    names: list[str] = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            names += [alias.name for alias in node.names]
        elif isinstance(node, ast.ImportFrom) and node.module is not None:
            names.append(node.module)
    return names


def is_mcp(module: str) -> bool:
    return module == "mcp" or module.startswith("mcp.")


def files_under(root: Path, suffix: str) -> list[Path]:
    found: list[Path] = []
    for directory, subdirectories, names in os.walk(root):
        subdirectories[:] = [name for name in subdirectories if name not in PRUNED]
        found += [Path(directory) / name for name in names if name.endswith(suffix)]
    return sorted(found)


def requirement_names(node: object) -> list[str]:
    """Every requirement name in a parsed TOML value: list items that are strings, at any depth."""
    if isinstance(node, str):
        match = _NAME.match(node)
        return [match.group(1).lower()] if match else []
    if isinstance(node, list):
        return [name for item in cast("list[object]", node) for name in requirement_names(item)]
    if isinstance(node, dict):
        values = cast("dict[str, object]", node).values()
        return [name for value in values for name in requirement_names(value)]
    return []


def test_mcp_is_imported_only_by_the_adapter_module() -> None:
    assert ADAPTER.is_file()
    importers = [
        source for source in files_under(SOURCE, ".py") if any(is_mcp(m) for m in imported(source))
    ]
    assert importers == [ADAPTER]


def test_mcp_is_a_dependency_of_the_member_alone() -> None:
    member = tomllib.loads((MEMBER / "pyproject.toml").read_text(encoding="utf-8"))
    assert "mcp" in requirement_names(member["project"]["dependencies"])
    for manifest in files_under(REPO, "pyproject.toml"):
        if manifest == MEMBER / "pyproject.toml":
            continue
        declared = tomllib.loads(manifest.read_text(encoding="utf-8"))
        assert "mcp" not in requirement_names(declared), manifest


def test_no_file_outside_the_member_imports_mcp() -> None:
    for source in files_under(REPO, ".py"):
        if source.is_relative_to(MEMBER):
            continue
        assert not any(is_mcp(module) for module in imported(source)), source


def test_the_adapter_reaches_the_world_only_through_player() -> None:
    names = imported(ADAPTER)
    assert not any(name.startswith("mineworld_sdk.resuming") for name in names)
    assert not any(name.startswith("mineworld_sdk.offers") for name in names)
    assert "socket" not in names
