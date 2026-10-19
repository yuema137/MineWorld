#!/usr/bin/env python3
"""The sandbox's `mineworld`: refuses any path outside the sandbox, then runs the real binary (PROTOCOL.md §3.3).

The agent may run `mineworld …` and nothing else in its shell, but `mineworld` itself reads and writes the paths it is
given (`--save`, `--packs`, `reference --out`, `create`). This guard is installed as `SANDBOX/bin/mineworld`; the real
binary is `SANDBOX/bin/real/mineworld`, which the agent's allowlist cannot name. It refuses, before the binary starts,
a working directory outside the sandbox and every argument that names a path outside it. It must run on the system
`python3` of macOS (3.9), so it uses nothing newer.
"""

from __future__ import annotations

import os
import sys

HERE = os.path.dirname(os.path.realpath(__file__))
SANDBOX = os.path.dirname(HERE)
REAL = os.path.join(HERE, "real", "mineworld.exe" if os.name == "nt" else "mineworld")


def inside(path: str) -> bool:
    root = os.path.realpath(SANDBOX)
    target = os.path.realpath(path)
    return target == root or target.startswith(root + os.sep)


def pathlike(word: str) -> bool:
    return "/" in word or "\\" in word or word.startswith(("~", ".")) or ":" in word[1:3]


LOOPBACK = ("127.0.0.1", "localhost", "::1", "[::1]")


def listen_refusal(arguments: list[str]) -> str | None:
    """A server in the sandbox listens on this machine only: `--listen` must name a loopback address."""
    for index, argument in enumerate(arguments):
        if argument == "--listen":
            value = arguments[index + 1] if index + 1 < len(arguments) else ""
        elif argument.startswith("--listen="):
            value = argument.split("=", 1)[1]
        else:
            continue
        host = value.rsplit(":", 1)[0] if value.count(":") == 1 or value.startswith("[") else value
        if host not in LOOPBACK and not host.startswith("127."):
            return (
                "--listen "
                + value
                + " is not a loopback address; a server here listens on 127.0.0.1 only"
            )
    return None


def refusal(arguments: list[str], cwd: str) -> str | None:
    if not inside(cwd):
        return "the working directory " + cwd + " is outside the sandbox"
    listen = listen_refusal(arguments)
    if listen is not None:
        return listen
    for argument in arguments:
        for word in argument.split("=", 1)[-1:] if argument.startswith("--") else [argument]:
            if pathlike(word) and not inside(os.path.join(cwd, os.path.expanduser(word))):
                return "the path " + word + " is outside the sandbox"
    return None


def main() -> int:
    arguments = sys.argv[1:]
    reason = refusal(arguments, os.getcwd())
    if reason is not None:
        sys.stderr.write(
            "[sandbox] refused: " + reason + "; a world and its saves stay under " + SANDBOX + "\n"
        )
        return 126
    os.execv(REAL, [REAL, *arguments])
    return 0


if __name__ == "__main__":
    sys.exit(main())
