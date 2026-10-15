"""I-c-8 (INV-S27-9): this library imports no MCP module and depends on no MCP distribution.

MCP is S27-d's skin over `Player`, confined to one module it adds. Until then nothing here may name it: a
scan of every source file's imports and of the member's declared dependencies.
"""

from __future__ import annotations

import ast
import re
import tomllib
from pathlib import Path

MEMBER = Path(__file__).resolve().parents[1]
SOURCES = sorted((MEMBER / "src").rglob("*.py"))
_MCP = re.compile(r"^mcp([._-]|$)", re.IGNORECASE)


def imported(source: Path) -> list[str]:
    tree = ast.parse(source.read_text(encoding="utf-8"), filename=str(source))
    names: list[str] = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            names += [alias.name for alias in node.names]
        elif isinstance(node, ast.ImportFrom) and node.module is not None:
            names.append(node.module)
    return names


def test_no_source_imports_mcp() -> None:
    assert SOURCES, "the scan found no source to read"
    found = [(s.name, n) for s in SOURCES for n in imported(s) if _MCP.match(n)]
    assert not found, f"an MCP import in the core library: {found}"


def test_no_dependency_is_mcp() -> None:
    project = tomllib.loads((MEMBER / "pyproject.toml").read_text(encoding="utf-8"))
    declared: list[str] = list(project["project"]["dependencies"])
    for group in project.get("dependency-groups", {}).values():
        declared += [entry for entry in group if isinstance(entry, str)]
    names = [re.split(r"[\s\[<>=!~;]", entry, maxsplit=1)[0] for entry in declared]
    assert names, "the scan found no dependency to read"
    assert not [n for n in names if _MCP.match(n)], f"an MCP dependency: {names}"
