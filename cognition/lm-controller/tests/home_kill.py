"""A child process for the home's lock and kill tests (pr-s28a §5.10). Test code: nothing in production names a
kill point; every seam is patched here, in this process only.

    python home_kill.py busy <home> <world-instance>      open a home another process holds; print the outcome
    python home_kill.py die <home> <world-instance>       open a home and die holding it (os._exit, no close)
"""

from __future__ import annotations

import os
import sys
from pathlib import Path

from mineworld_sdk.wire.ids import WorldInstanceId

from mineworld_cognition.memory.home import MemoryHome


def busy(path: Path, instance: WorldInstanceId) -> None:
    try:
        home = MemoryHome.open(path, instance)
    except ValueError as error:
        print(type(error).__name__, flush=True)
        return
    home.close()
    print("opened", flush=True)


def die(path: Path, instance: WorldInstanceId) -> None:
    MemoryHome.open(path, instance)
    print("holding", flush=True)
    os._exit(137)


def main(arguments: list[str]) -> None:
    scenario, path, instance = arguments[0], Path(arguments[1]), WorldInstanceId(arguments[2])
    if scenario == "busy":
        busy(path, instance)
    elif scenario == "die":
        die(path, instance)
    else:
        raise SystemExit(f"unknown scenario {scenario!r}")


if __name__ == "__main__":
    main(sys.argv[1:])
