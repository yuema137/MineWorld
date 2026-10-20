"""`mineworld-agent`: the MCP adapter's command line (S27-d §5.8, S27-e §5.7) and the published tool list (`schema`).

The only module that reads the environment. Over stdio the credential is `MINEWORLD_GRANT`, else `MINEWORLD_INVITE`,
and it is never an argument and never printed (step-27 §6.3). With `--http` each request's `Authorization: Bearer` is
its credential, and the environment credentials must be unset: the adapter holds none of its own (F-S27e-7). stdout
carries MCP messages (stdio), or for `schema` the JSON alone; diagnostics go to stderr, one line each, naming a
code, a flag or a field and never a value.
"""

from __future__ import annotations

import argparse
import asyncio
import os
import re
import socket
import sys
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path

from mineworld_sdk.wire.frames import Invite
from pydantic import ValidationError

from mineworld_agent.mcp_server import (
    Adapter,
    catalogue_json,
    http_app,
    is_loopback,
    security_settings,
    serve_http,
    serve_stdio,
)
from mineworld_agent.player import Player, PlayerConfig
from mineworld_agent.pool import Credentials

GRANT_VARIABLE = "MINEWORLD_GRANT"
INVITE_VARIABLE = "MINEWORLD_INVITE"
USAGE_ERROR = 2
DEFAULT_CREDENTIALS = 8
MAX_CREDENTIALS = 64
_ORIGIN = re.compile(r"^[a-z][a-z0-9+.-]*://[^/\s?#@]+$")
"""An origin: `scheme://host[:port]`, no path, query, fragment or user."""
_HOST_PORT = re.compile(r"^(?:\[(?P<v6>[0-9A-Fa-f:.]+)\]|(?P<name>[^\s:\[\]]+)):(?P<port>\d{1,5})$")
_ALLOW_HOST = re.compile(r"^(?:\[[0-9A-Fa-f:.]+\]|[^\s:\[\]/]+)(?::\d{1,5})?$")


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="mineworld-agent",
        description="Play seats of a MineWorld world as an MCP server, over stdio or Streamable HTTP.",
    )
    commands = parser.add_subparsers(dest="command", required=True)

    mcp = commands.add_parser(
        "mcp",
        help="serve the nine tools over stdio (one player) or over HTTP (one player per bearer)",
    )
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
    mcp.add_argument(
        "--http",
        default=None,
        metavar="HOST:PORT",
        help="serve Streamable HTTP at http://HOST:PORT/mcp instead of stdio; loopback only by default",
    )
    mcp.add_argument(
        "--allow-remote",
        action="store_true",
        default=False,
        help="with --http, allow a non-loopback address (plain HTTP: put TLS in front of it)",
    )
    mcp.add_argument(
        "--allow-origin",
        action="append",
        default=None,
        metavar="ORIGIN",
        help="with --http, a browser origin allowed to call (repeatable; default none)",
    )
    mcp.add_argument(
        "--allow-host",
        action="append",
        default=None,
        metavar="HOST[:PORT]",
        help="with --http --allow-remote, a Host name a tunnel or gateway sends (repeatable)",
    )
    mcp.add_argument(
        "--max-credentials",
        type=int,
        default=None,
        help=f"with --http, at most this many credentials at once (1 to {MAX_CREDENTIALS}, default 8)",
    )

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


def _config(arguments: argparse.Namespace) -> PlayerConfig | None:
    try:
        return PlayerConfig(
            server=arguments.server,
            idle_release_s=arguments.idle_release,
            inbox=arguments.inbox,
            max_result_bytes=arguments.max_result_bytes,
            state_dir=arguments.state_dir,
        )
    except ValidationError as error:
        for item in error.errors(include_url=False):
            _stderr(f"invalid argument: {'.'.join(str(part) for part in item['loc'])}")
        return None


def _mcp(arguments: argparse.Namespace, environment: Mapping[str, str]) -> int:
    if arguments.http is not None:
        return _mcp_http(arguments, environment)
    for flag, given in (
        ("--allow-remote", arguments.allow_remote),
        ("--allow-origin", arguments.allow_origin is not None),
        ("--allow-host", arguments.allow_host is not None),
        ("--max-credentials", arguments.max_credentials is not None),
    ):
        if given:
            _stderr(f"{flag} needs --http")
            return USAGE_ERROR
    grant = environment.get(GRANT_VARIABLE, "")
    invite = environment.get(INVITE_VARIABLE, "")
    if grant and invite:
        _stderr(f"set one of {GRANT_VARIABLE} and {INVITE_VARIABLE}, not both")
        return USAGE_ERROR
    secret = grant or invite
    if not secret:
        _stderr(f"set {GRANT_VARIABLE} or {INVITE_VARIABLE} in the environment")
        return USAGE_ERROR
    config = _config(arguments)
    if config is None:
        return USAGE_ERROR
    return asyncio.run(_serve(config, Invite(secret)))


async def _serve(config: PlayerConfig, credential: Invite) -> int:
    adapter = Adapter()
    async with Player(config, credential, on_inbox=adapter.on_inbox) as player:
        await serve_stdio(adapter.server(player, config))
    return 0


def _address(value: str) -> tuple[str, int] | None:
    """`HOST:PORT` or `[V6]:PORT` as (host, port), or `None`."""
    match = _HOST_PORT.match(value)
    if match is None or int(match.group("port")) > 65535:
        return None
    return match.group("v6") or match.group("name"), int(match.group("port"))


def _mcp_http(arguments: argparse.Namespace, environment: Mapping[str, str]) -> int:
    if environment.get(GRANT_VARIABLE) or environment.get(INVITE_VARIABLE):
        _stderr(
            "HTTP mode takes each request's credential from its bearer; "
            f"unset {GRANT_VARIABLE} and {INVITE_VARIABLE}"
        )
        return USAGE_ERROR
    address = _address(arguments.http)
    if address is None:
        _stderr("invalid argument: --http (expected HOST:PORT)")
        return USAGE_ERROR
    host, port = address
    remote = not is_loopback(host)
    if remote and not arguments.allow_remote:
        _stderr(
            "--http HOST is not a loopback address; add --allow-remote, "
            "and put TLS in front of it (a tunnel or a gateway)"
        )
        return USAGE_ERROR
    allow_hosts: list[str] = arguments.allow_host or []
    if allow_hosts and not arguments.allow_remote:
        _stderr("--allow-host needs --allow-remote")
        return USAGE_ERROR
    if not all(_ALLOW_HOST.match(name) for name in allow_hosts):
        _stderr("invalid argument: --allow-host (expected HOST or HOST:PORT)")
        return USAGE_ERROR
    origins: list[str] = arguments.allow_origin or []
    if not all(_ORIGIN.match(origin) for origin in origins):
        _stderr("invalid argument: --allow-origin (expected scheme://host[:port])")
        return USAGE_ERROR
    limit = DEFAULT_CREDENTIALS if arguments.max_credentials is None else arguments.max_credentials
    if not 1 <= limit <= MAX_CREDENTIALS:
        _stderr(f"invalid argument: --max-credentials (1 to {MAX_CREDENTIALS})")
        return USAGE_ERROR
    config = _config(arguments)
    if config is None:
        return USAGE_ERROR
    family = socket.AF_INET6 if ":" in host else socket.AF_INET
    try:
        bound = socket.create_server((host, port), family=family)
    except OSError as error:
        _stderr(f"cannot listen on --http: {type(error).__name__}")
        return USAGE_ERROR
    real = bound.getsockname()[1]
    shown = f"[{host}]" if ":" in host else host
    where = (
        "plain HTTP on a non-loopback address: TLS belongs to a tunnel or a gateway in front of it"
        if remote
        else "loopback only"
    )
    _stderr(f"listening on http://{shown}:{real}/mcp ({where})")
    names = Names(host=host, port=real, allow_hosts=allow_hosts, origins=origins)
    return asyncio.run(_serve_http(config, names, limit, bound))


@dataclass(frozen=True, slots=True)
class Names:
    """What the `Host` and `Origin` checks allow (pr-s27e §5.5): the bound address, its real port, the extra hosts."""

    host: str
    port: int
    allow_hosts: list[str]
    origins: list[str]


async def _serve_http(config: PlayerConfig, names: Names, limit: int, bound: socket.socket) -> int:
    adapter = Adapter()
    credentials = Credentials(config, limit=limit, on_inbox=adapter.on_inbox)
    settings = security_settings((names.host, names.port), names.allow_hosts, names.origins)
    app = http_app(adapter, credentials, config, settings)
    try:
        await serve_http(app, credentials, bound)
    finally:
        bound.close()
    return 0


def main(argv: Sequence[str] | None = None) -> int:
    arguments = _parser().parse_args(None if argv is None else list(argv))
    if arguments.command == "schema":
        return _schema(arguments.out)
    return _mcp(arguments, os.environ)


if __name__ == "__main__":
    raise SystemExit(main())
