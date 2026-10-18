"""`mineworld-agent`: the MCP adapter's command line (S27-d §5.8) and the published tool list (`schema`).

The only module that reads the environment: the credential is `MINEWORLD_GRANT`, else `MINEWORLD_INVITE`, and it is
never an argument and never printed (step-27 §6.3). stdout carries MCP messages, or for `schema` the JSON alone;
diagnostics go to stderr, one line each, naming a code or a field and never a value.
"""

from __future__ import annotations

import argparse
import asyncio
import os
import sys
from collections.abc import Mapping, Sequence
from pathlib import Path

from mineworld_sdk.wire.frames import Invite
from pydantic import ValidationError

from mineworld_agent.mcp_server import Adapter, catalogue_json, serve_stdio
from mineworld_agent.player import Player, PlayerConfig

GRANT_VARIABLE = "MINEWORLD_GRANT"
INVITE_VARIABLE = "MINEWORLD_INVITE"
USAGE_ERROR = 2


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="mineworld-agent",
        description="Play one seat of a MineWorld world as an MCP server over stdio.",
    )
    commands = parser.add_subparsers(dest="command", required=True)

    mcp = commands.add_parser("mcp", help="serve the thirteen tools over stdio, for one player")
    mcp.add_argument(
        "--server", required=True, help="the world's WebSocket address, ws://HOST:PORT/ws"
    )
    mcp.add_argument(
        "--state-dir", type=Path, default=None, help="where resume secrets and cursors are kept"
    )
    mcp.add_argument(
        "--idle-release",
        type=int,
        default=600,
        help="seconds without a call before a seat is given back; 0 = never",
    )
    mcp.add_argument(
        "--max-result-bytes", type=int, default=8192, help="at most 8192 bytes per result"
    )
    mcp.add_argument("--inbox", type=int, default=512, help="facts the inbox holds, at most 512")

    schema = commands.add_parser("schema", help="print the tool list as JSON")
    schema.add_argument(
        "--out", type=Path, default=None, help="write to this file instead of stdout"
    )
    return parser


def _stderr(line: str) -> None:
    print(f"mineworld-agent: {line}", file=sys.stderr, flush=True)


def _schema(out: Path | None) -> int:
    if out is None:
        print(catalogue_json(), end="")
    else:
        out.write_text(catalogue_json(), encoding="utf-8", newline="\n")
    return 0


def _mcp(arguments: argparse.Namespace, environment: Mapping[str, str]) -> int:
    grant = environment.get(GRANT_VARIABLE, "")
    invite = environment.get(INVITE_VARIABLE, "")
    if grant and invite:
        _stderr(f"set one of {GRANT_VARIABLE} and {INVITE_VARIABLE}, not both")
        return USAGE_ERROR
    secret = grant or invite
    if not secret:
        _stderr(f"set {GRANT_VARIABLE} or {INVITE_VARIABLE} in the environment")
        return USAGE_ERROR
    try:
        config = PlayerConfig(
            server=arguments.server,
            idle_release_s=arguments.idle_release,
            inbox=arguments.inbox,
            max_result_bytes=arguments.max_result_bytes,
            state_dir=arguments.state_dir,
        )
    except ValidationError as error:
        for item in error.errors(include_url=False):
            _stderr(f"invalid argument: {'.'.join(str(part) for part in item['loc'])}")
        return USAGE_ERROR
    return asyncio.run(_serve(config, Invite(secret)))


async def _serve(config: PlayerConfig, credential: Invite) -> int:
    adapter = Adapter()
    async with Player(config, credential, on_inbox=adapter.on_inbox) as player:
        await serve_stdio(adapter.server(player, config))
    return 0


def main(argv: Sequence[str] | None = None) -> int:
    arguments = _parser().parse_args(None if argv is None else list(argv))
    if arguments.command == "schema":
        return _schema(arguments.out)
    return _mcp(arguments, os.environ)


if __name__ == "__main__":
    raise SystemExit(main())
