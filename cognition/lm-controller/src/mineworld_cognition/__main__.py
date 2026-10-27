"""`python -m mineworld_cognition --config FILE`: the `lm` seats of one decision file, joined to one server (C12).

Design: `.structured-coding/plans/mvp0/pr-s10-p6-lm-controller.md` §§5.9, 5.16. The configuration names the server, the
store, the decision JSON (`mineworld decisions --json`) and the invite's environment variable. One `SeatRunner` serves
each selected `lm` seat. Ctrl-C (SIGINT) or SIGTERM leaves every seat within five seconds (`SeatRunner._leave_on_stop`), and a
stop that hangs is cut short (`shutdown.Stopper`, D-P6-43); the status line of each seat is read back from its decision log,
which every runner closes with its counters.
"""

from __future__ import annotations

import argparse
import asyncio
import json
import os
import sys
import time
from collections.abc import Coroutine, Sequence
from pathlib import Path
from typing import cast

from mineworld_sdk.session import open_socket
from mineworld_sdk.wire.frames import Invite

from mineworld_cognition.backend.model import ModelBackend
from mineworld_cognition.config import CognitionConfig, build_router, load
from mineworld_cognition.controller.assignment import LmSeat, read_assignment
from mineworld_cognition.controller.homes import SeatHomes
from mineworld_cognition.controller.seat import SeatRunner
from mineworld_cognition.errors import ConfigError
from mineworld_cognition.gateway import Router
from mineworld_cognition.shutdown import Stopper

LOG_KIND = "counters"


def main(argv: Sequence[str] | None = None, *, scripted: ModelBackend | None = None) -> int:
    """`scripted` is the backend a `recording.mode = "scripted"` configuration needs, supplied in code by a test's
    entry (`build_router`); the command line never supplies one (S10 P7, `pr-s10-p7-milestone-d.md` C1)."""
    parser = argparse.ArgumentParser(
        prog="python -m mineworld_cognition",
        description="Drive the lm seats of a decision file through a language model (S10 P6b).",
    )
    parser.add_argument("--config", required=True, type=Path, metavar="FILE")
    arguments = parser.parse_args(argv)
    try:
        config = load(arguments.config)
        server, decisions, store_dir = _required(config)
        seats = read_assignment(decisions, config.seats).seats
        invite = _invite(config)
        router = build_router(config, scripted=scripted)
    except ConfigError as error:
        print(f"[cognition] {error}", file=sys.stderr)
        return 2
    _earlier_layout(store_dir, seats)
    stopper = Stopper()
    try:
        asyncio.run(
            stopper.serve(
                _closing(
                    serve(
                        config,
                        seats,
                        router=router,
                        invite=invite,
                        url=server,
                        store_dir=store_dir,
                    ),
                    router,
                )
            )
        )
    except ConfigError as error:
        print(f"[cognition] {error}", file=sys.stderr)
        return 2
    except KeyboardInterrupt:
        pass
    finally:
        for seat in seats:
            print(_status(store_dir, seat), flush=True)
        stopper.disarm()
    return 0


async def serve(
    config: CognitionConfig,
    seats: Sequence[LmSeat],
    *,
    router: Router,
    invite: Invite,
    url: str,
    store_dir: Path,
) -> None:
    """Runs one `SeatRunner` per seat on the running loop until every runner ends or one raises. The seats share
    one `SeatHomes` (pr-s28f §5.6), closed however serving ends."""
    homes = SeatHomes(store_dir, config.memory)
    runners = [
        SeatRunner(
            config,
            seat,
            router=router,
            connector=lambda: open_socket(url),
            invite=invite,
            nickname=config.nickname,
            store_dir=store_dir,
            homes=homes,
            clock=time.monotonic,
            sleep=asyncio.sleep,
        )
        for seat in seats
    ]
    try:
        async with asyncio.TaskGroup() as group:
            for runner in runners:
                group.create_task(runner.run())
    except* ConfigError as refusals:
        raise refusals.exceptions[0] from None
    finally:
        homes.close()


def _earlier_layout(store_dir: Path, seats: Sequence[LmSeat]) -> None:
    """F-S28f-6: a `<store>/<seat>.sqlite` of S28-a's layout is named once and left in place (QS28-8)."""
    for seat in seats:
        path = store_dir / f"{seat.key}.sqlite"
        if path.exists():
            print(
                f"[cognition] {seat.key}: {path} is a memory file of the earlier layout; it is no longer "
                "read and may be deleted (memory is rebuilt from the perceived stream)",
                file=sys.stderr,
            )


async def _closing(serving: Coroutine[None, None, None], router: Router) -> None:
    """Serves, then closes the router however serving ended, Ctrl-C included (F-P7-1): closing is what turns a
    `record` session's `.partial` into its cassette and what closes the ledger."""
    try:
        await serving
    finally:
        await router.aclose()


def _required(config: CognitionConfig) -> tuple[str, Path, Path]:
    """The server URL, the decision JSON and the store directory, each refused by name when absent."""
    if config.server is None:
        raise ConfigError("server: not set in the configuration file")
    if config.decisions_json is None:
        raise ConfigError("decisions_json: not set in the configuration file")
    if config.store is None:
        raise ConfigError("store: not set in the configuration file")
    return config.server, Path(config.decisions_json), Path(config.store)


def _invite(config: CognitionConfig) -> Invite:
    """The invite, read from the environment variable NAMED by the configuration (R-S11-5). The value is never
    printed: an error names the variable only."""
    value = os.environ.get(config.invite_env)
    if not value:
        raise ConfigError(
            f"{config.invite_env}: the environment variable holding the invite is not set"
        )
    return Invite(value)


def _status(store_dir: Path, seat: LmSeat) -> str:
    """One status line per seat, from the last counters line of its decision log (§5.12)."""
    path = store_dir / f"{seat.key}.decisions.jsonl"
    last: dict[str, object] | None = None
    if path.is_file():
        for line in path.read_text(encoding="utf-8").splitlines():
            loaded: object = json.loads(line) if line else None
            if isinstance(loaded, dict) and cast(dict[str, object], loaded).get("kind") == LOG_KIND:
                last = cast(dict[str, object], loaded)
    if last is None:
        return f"[cognition] {seat.key}: no consults recorded"
    fields = ", ".join(f"{name} {value}" for name, value in last.items() if name != "kind")
    return f"[cognition] {seat.key}: {fields}"


if __name__ == "__main__":
    raise SystemExit(main())
