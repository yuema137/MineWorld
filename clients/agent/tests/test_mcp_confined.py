"""I-d-1 and I-d-5 (INV-S27-9, `ARC-96`): `mcp` is imported by one module of the member and is a dependency of
the member alone, and the adapter reaches the world only through `Player`. Also the two SDK prototypes of S27-d
C1 (P-1, P-2, `pr-s27d` §9): they stay as regression tests of the SDK contract the adapter relies on.

Replaces S27-c's `test_no_mcp.py`, which forbade `mcp` everywhere; the scan now allows it in exactly one module.
"""

from __future__ import annotations

import ast
import os
import re
import tomllib
from pathlib import Path
from typing import Any, cast

from mcp import types
from mcp.client.client import Client
from mcp.server.context import ServerRequestContext
from mcp.server.lowlevel import Server
from support import run

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


async def _list(
    _ctx: ServerRequestContext[Any], _params: types.PaginatedRequestParams | None
) -> types.ListToolsResult:
    return types.ListToolsResult(
        tools=[
            types.Tool(
                name="double",
                description="Doubles x.",
                input_schema={
                    "type": "object",
                    "properties": {"x": {"type": "integer"}},
                    "required": ["x"],
                },
                output_schema={"type": "object", "properties": {"doubled": {"type": "integer"}}},
            )
        ]
    )


def test_p1_the_low_level_server_publishes_the_declared_schemas() -> None:
    """P-1: the schemas given to `Tool` are the ones a client lists, unchanged (no nesting, no rewriting)."""

    async def scenario() -> types.Tool:
        server: Server[Any] = Server("prototype", on_list_tools=_list)
        async with Client(server) as client:
            listed = await client.list_tools()
        assert [tool.name for tool in listed.tools] == ["double"]
        return listed.tools[0]

    tool = run(scenario(), timeout_s=30)
    assert tool.input_schema == {
        "type": "object",
        "properties": {"x": {"type": "integer"}},
        "required": ["x"],
    }
    assert tool.output_schema == {"type": "object", "properties": {"doubled": {"type": "integer"}}}


def test_p2_an_error_result_without_structured_content_is_accepted_for_a_tool_with_an_output_schema() -> (
    None
):
    """P-2 (QS27d-1): an `isError` result with only text content is delivered, not rejected, by the official
    client, although the tool declares an `outputSchema` (the SDK skips output validation for `isError`)."""

    async def call(
        _ctx: ServerRequestContext[Any], _params: types.CallToolRequestParams
    ) -> types.CallToolResult:
        return types.CallToolResult(
            content=[types.TextContent(type="text", text='{"kind":"error","code":"seat_lost"}')],
            is_error=True,
        )

    async def scenario() -> types.CallToolResult:
        server: Server[Any] = Server("prototype", on_list_tools=_list, on_call_tool=call)
        async with Client(server) as client:
            return await client.call_tool("double", {"x": 1})

    result = run(scenario(), timeout_s=30)
    assert result.is_error is True
    assert result.structured_content is None
    assert isinstance(result.content[0], types.TextContent)
    assert result.content[0].text == '{"kind":"error","code":"seat_lost"}'
