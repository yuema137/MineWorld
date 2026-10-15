"""The MCP adapter over stdio (S27-d, `docs/DECISIONS.md` `ARC-96`, `DEP-56`).

The only module of `mineworld-agent` that imports `mcp` (INV-S27-9). It holds no game logic: every call goes
through `Player`. This C1 file is a stub: an empty low-level `Server`, which the prototype tests drive. The
tools, errors, resources, prompt and subscription are added in C3.
"""

from __future__ import annotations

from typing import Any

from mcp.server.lowlevel import Server


def build_server() -> Server[Any]:
    """An empty low-level MCP server named for the adapter (C1 stub; C3 registers the tools)."""
    return Server("mineworld-agent")
