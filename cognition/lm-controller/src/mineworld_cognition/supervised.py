"""`python -m mineworld_cognition --supervised …`: the cognition process as a launcher's child (design
`pr-playable-dialogue-memory.md` §§3.4, 4.8 to 4.11; specification `docs/reference/cognition-supervised.md`).

Supervised mode adds four things to the standalone command, and changes nothing without `--supervised`:

- **The supervisor's paths win.** `--server-url` (loopback only), `--store-dir` and `--assignment` replace the
  configuration's `server`, `store` and `decisions_json` as typed fields (`model_copy`), and the invite is read
  from `MINEWORLD_INVITE`, the variable the launcher sets. Every bound backend must be on a loopback host.
- **Standard output carries only status lines** (`Status`): one JSON object per line, `starting` → `ready` (the
  seat's welcome received and its memory home held) → `stopped` or `degraded` with a closed reason. Everything
  else the process prints goes to standard error.
- **The end of standard input stops it**, as a first SIGINT does (`shutdown.Stopper.stop`), so a launcher that
  closes the pipe — or dies — stops it on every platform.
- **The store is bound to one world instance** (`BoundHomes`): the first welcome writes `<store>/instance`; a later
  welcome of another instance is refused as `instance_mismatch` before any memory home is opened.

`--preflight MANIFEST` checks the runtime against its manifest (`runtime_manifest`) and the configuration's backends,
prints one status line and exits; `--write-runtime-manifest PATH` writes the manifest and exits.
"""

from __future__ import annotations

import argparse
import ipaddress
import json
import os
import sys
import threading
import urllib.error
import urllib.request
from collections.abc import Callable, Iterator, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import IO, Literal, cast
from urllib.parse import urlsplit

from mineworld_sdk.reconnect import SeatLost
from mineworld_sdk.wire.frames import Invite

from mineworld_cognition import runtime_manifest
from mineworld_cognition.config import CognitionConfig, is_loopback_host, load
from mineworld_cognition.controller.assignment import LmSeat, read_assignment
from mineworld_cognition.controller.homes import HeldSeat, SeatHomes
from mineworld_cognition.errors import ConfigError
from mineworld_cognition.memory.home.errors import (
    HomeBusy,
    HomeIdentityMismatch,
    NotAMemoryHome,
    UnsupportedStoreSchema,
)
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.shutdown import Stopper

type Serving = Callable[[CognitionConfig, Sequence[LmSeat], Invite, SeatHomes, Stopper], int]
"""The standalone command's serving loop (`__main__`), run with this mode's homes and stopper."""

STATUS_FORMAT = 1
INVITE_ENV = "MINEWORLD_INVITE"
"""The variable the launcher sets to the server's invite; never a flag, never printed."""
INSTANCE_FILE = "instance"
MODEL_CHECK_S = 3.0
"""How long the preflight waits for the local model service to list its models."""
EXIT_DEGRADED = 3
"""The exit status of a supervised run or a preflight that ended on a named reason."""

State = Literal["starting", "ready", "degraded", "stopped"]
Reason = Literal[
    "config_invalid",
    "assignment_invalid",
    "runtime_mismatch",
    "backend_not_loopback",
    "home_busy",
    "home_identity_mismatch",
    "instance_mismatch",
    "server_unreachable",
    "seat_unavailable",
    "model_unavailable",
    "stopping",
]


class Degraded(Exception):
    """Conversation cannot go on, for a named reason. The message is for the cognition log only; the status line
    carries the reason alone."""

    def __init__(self, reason: Reason, detail: str) -> None:
        super().__init__(detail)
        self.reason: Reason = reason


class InstanceMismatch(ValueError):
    """The store belongs to another world instance than the one the server welcomed the seat to."""


class Status:
    """The status channel: one JSON object per line on the process's original standard output (§2 of the spec)."""

    def __init__(self, out: IO[str]) -> None:
        self._out = out
        self._lock = threading.Lock()

    def send(self, state: State, seat: str | None = None, reason: Reason | None = None) -> None:
        line = json.dumps({"status": STATUS_FORMAT, "state": state, "seat": seat, "reason": reason})
        with self._lock:
            try:
                self._out.write(line + "\n")
                self._out.flush()
            except OSError:
                pass  # a launcher that died closed the pipe; its end of input stops this process anyway


def add_arguments(parser: argparse.ArgumentParser) -> None:
    group = parser.add_argument_group("supervised (a launcher's child)")
    group.add_argument("--supervised", action="store_true")
    group.add_argument("--server-url", metavar="URL")
    group.add_argument("--store-dir", type=Path, metavar="DIR")
    group.add_argument("--assignment", type=Path, metavar="FILE")
    group.add_argument("--preflight", type=Path, metavar="MANIFEST")
    group.add_argument("--write-runtime-manifest", type=Path, metavar="PATH")


def wants(arguments: argparse.Namespace) -> bool:
    """Whether the command line asks for anything this module answers."""
    return bool(
        arguments.supervised or arguments.preflight or arguments.write_runtime_manifest
    ) or any(
        value is not None
        for value in (arguments.server_url, arguments.store_dir, arguments.assignment)
    )


def dispatch(
    arguments: argparse.Namespace,
    serve: Serving,
) -> int:
    """Runs the asked mode; `serve` is the standalone command's serving loop, given this mode's homes."""
    if arguments.write_runtime_manifest is not None:
        return _write_manifest(arguments.write_runtime_manifest)
    if arguments.config is None:
        print("[cognition] --config is required", file=sys.stderr)
        return 2
    paths = (arguments.server_url, arguments.store_dir, arguments.assignment)
    if not arguments.supervised and any(value is not None for value in paths):
        print(
            "[cognition] --server-url, --store-dir and --assignment are given by a supervisor: add --supervised",
            file=sys.stderr,
        )
        return 2
    # From here on, the original standard output is the status channel and nothing else (§2).
    status = Status(sys.stdout)
    sys.stdout = sys.stderr
    if arguments.preflight is not None:
        return _preflight(arguments.config, arguments.preflight, status)
    if None in paths:
        print(
            "[cognition] --supervised needs --server-url, --store-dir and --assignment",
            file=sys.stderr,
        )
        return 2
    return _supervised(arguments, status, serve)


# ── Preflight and manifest ─────────────────────────────────────────────────────────────────────────


def _write_manifest(path: Path) -> int:
    try:
        manifest = runtime_manifest.write(path)
    except (runtime_manifest.ManifestError, OSError) as error:
        print(f"[cognition] cannot write the runtime manifest: {error}", file=sys.stderr)
        return 2
    print(
        f"[cognition] wrote {path}: Python {manifest.python}, revision {manifest.source_revision}, "
        f"{len(manifest.distributions)} distributions",
        file=sys.stderr,
    )
    return 0


def _preflight(config_path: Path, manifest_path: Path, status: Status) -> int:
    try:
        config = _loaded(config_path)
        _check_runtime(manifest_path)
        _check_backends(config)
        _check_model(config)
    except Degraded as degraded:
        print(f"[cognition] preflight: {degraded.reason}: {degraded}", file=sys.stderr)
        status.send("degraded", reason=degraded.reason)
        return EXIT_DEGRADED
    status.send("starting")
    return 0


def _check_runtime(manifest_path: Path) -> None:
    try:
        recorded = runtime_manifest.read(manifest_path)
        current = runtime_manifest.derive()
    except runtime_manifest.ManifestError as error:
        raise Degraded("runtime_mismatch", str(error)) from None
    changed = runtime_manifest.differences(recorded, current)
    if changed:
        raise Degraded(
            "runtime_mismatch",
            f"the runtime differs from {manifest_path} in {', '.join(changed)}; write the manifest again",
        )
    unresolved = runtime_manifest.unresolved_entry_points()
    if unresolved:
        raise Degraded(
            "runtime_mismatch", f"entry-point groups unresolved: {', '.join(unresolved)}"
        )


def _check_backends(config: CognitionConfig) -> None:
    for name in config.bound_backends():
        host = urlsplit(config.backends[name].base_url).hostname or ""
        if not is_loopback_host(host):
            raise Degraded("backend_not_loopback", f"backends.{name} is not on a loopback host")


def _check_model(config: CognitionConfig) -> None:
    """Asks each bound OpenAI-compatible local service which models it serves (`GET <base_url>/models`): no
    inference, no download, no start. Skipped in replay mode, which reaches no backend."""
    if config.recording.mode not in ("live", "record"):
        return
    for name in config.bound_backends():
        backend = config.backends[name]
        if backend.kind != "openai-compatible":
            continue
        url = backend.base_url.rstrip("/") + "/models"
        try:
            with urllib.request.urlopen(url, timeout=MODEL_CHECK_S) as response:
                listed = cast(dict[str, object], json.loads(response.read()))
        except (urllib.error.URLError, OSError, ValueError):
            raise Degraded(
                "model_unavailable", f"backends.{name}: the local service did not answer"
            ) from None
        models = cast(list[dict[str, object]], listed.get("data", []))
        if not any(entry.get("id") == backend.model for entry in models):
            raise Degraded(
                "model_unavailable", f"backends.{name}: the configured model is not served"
            )


# ── Supervised serving ─────────────────────────────────────────────────────────────────────────────


@dataclass(frozen=True)
class Ready:
    """What `BoundHomes` reports once a seat holds its home."""

    status: Status
    seat: str


class BoundHomes(SeatHomes):
    """The process's homes, with the store bound to one world instance (§4 of the spec) and `ready` sent once a seat
    holds its home. The binding is checked before any home is opened, so a refusal changes nothing in the store."""

    def __init__(self, store: Path, config: CognitionConfig, ready: Ready) -> None:
        super().__init__(store, config.memory)
        self._instance_file = store / INSTANCE_FILE
        self._ready = ready

    async def hold(self, identity: StoreIdentity) -> HeldSeat:
        self._bind(str(identity.world_instance))
        held = await super().hold(identity)
        self._ready.status.send("ready", seat=self._ready.seat)
        return held

    def _bind(self, instance: str) -> None:
        if self._instance_file.is_file():
            recorded = self._instance_file.read_text(encoding="utf-8").strip()
            if recorded != instance:
                raise InstanceMismatch(
                    f"{self._instance_file} binds this store to another world instance; it is left as it is"
                )
            return
        self._instance_file.parent.mkdir(parents=True, exist_ok=True)
        partial = self._instance_file.with_name(INSTANCE_FILE + ".partial")
        partial.write_text(instance + "\n", encoding="utf-8")
        partial.replace(self._instance_file)


def _supervised(
    arguments: argparse.Namespace,
    status: Status,
    serve: Serving,
) -> int:
    status.send("starting")
    stopper = Stopper()
    _watch_stdin(stopper)
    seat: str | None = None
    try:
        config = _overridden(
            _loaded(arguments.config),
            arguments.server_url,
            arguments.store_dir,
            arguments.assignment,
        )
        seats = _one_seat(Path(cast(str, config.decisions_json)), config.seats)
        seat = str(seats[0].key)
        status.send("starting", seat=seat)
        invite = _invite()
        homes = BoundHomes(Path(cast(str, config.store)), config, Ready(status, seat))
        code = serve(config, seats, invite, homes, stopper)
    except Degraded as degraded:
        print(f"[cognition] {degraded.reason}: {degraded}", file=sys.stderr)
        status.send("degraded", seat=seat, reason=degraded.reason)
        return EXIT_DEGRADED
    except BaseException as error:
        reason = reason_of(error)
        if reason is None:
            raise
        print(f"[cognition] {reason}: {error}", file=sys.stderr)
        status.send("degraded", seat=seat, reason=reason)
        return EXIT_DEGRADED
    status.send("stopped", seat=seat, reason="stopping")
    return code


def _loaded(path: Path) -> CognitionConfig:
    try:
        return load(path)
    except ConfigError as error:
        raise Degraded("config_invalid", str(error)) from None


def _overridden(
    config: CognitionConfig, server_url: str, store_dir: Path, assignment: Path
) -> CognitionConfig:
    """The supervisor's server, store and assignment, as typed fields; the invite's variable is the launcher's; the
    budget ledger is the store's unless the operator named one. The configuration cannot override any of them."""
    if not _loopback_ws(server_url):
        raise Degraded(
            "config_invalid",
            "--server-url must be ws://127.0.0.1:<port>/ws or ws://[::1]:<port>/ws",
        )
    if not store_dir.is_absolute() or not assignment.is_absolute():
        raise Degraded("config_invalid", "--store-dir and --assignment must be absolute paths")
    _check_backends(config)
    ledger = config.budgets.ledger or str(store_dir / "budget.sqlite")
    return config.model_copy(
        update={
            "server": server_url,
            "store": str(store_dir),
            "decisions_json": str(assignment),
            "invite_env": INVITE_ENV,
            "budgets": config.budgets.model_copy(update={"ledger": ledger}),
        }
    )


def _loopback_ws(url: str) -> bool:
    parts = urlsplit(url)
    if (
        parts.scheme != "ws"
        or parts.path != "/ws"
        or parts.query
        or parts.fragment
        or parts.username
    ):
        return False
    try:
        return parts.port is not None and ipaddress.ip_address(parts.hostname or "").is_loopback
    except ValueError:
        return False


def _one_seat(path: Path, narrowed: list[str] | None) -> list[LmSeat]:
    try:
        seats = read_assignment(path, narrowed).seats
    except ConfigError as error:
        raise Degraded("assignment_invalid", str(error)) from None
    if len(seats) != 1:
        raise Degraded(
            "assignment_invalid",
            f"{path}: {len(seats)} lm seats; a supervised run drives exactly one",
        )
    return seats


def _invite() -> Invite:
    value = os.environ.get(INVITE_ENV)
    if not value:
        raise Degraded("config_invalid", f"{INVITE_ENV}: the launcher did not set the invite")
    return Invite(value)


def _watch_stdin(stopper: Stopper) -> None:
    """A daemon thread that reads standard input to its end, then stops serving (§1 of the spec). A thread, not the
    event loop's reader: Windows' proactor loop cannot watch a pipe that way."""

    # The raw descriptor, never `sys.stdin.buffer`: a daemon thread blocked inside a buffered read holds the
    # reader's lock, and the interpreter aborts at exit when it cannot take that lock (found in a packaged run).
    descriptor = sys.stdin.fileno()

    def watch() -> None:
        try:
            while os.read(descriptor, 4096):
                pass
        except OSError:
            pass
        stopper.stop("standard input closed")

    threading.Thread(target=watch, name="supervisor-stdin", daemon=True).start()


def reason_of(error: BaseException) -> Reason | None:
    """The closed reason of what ended serving, looking inside exception groups; `None` for anything unnamed."""
    for leaf in _leaves(error):
        if isinstance(leaf, InstanceMismatch):
            return "instance_mismatch"
        if isinstance(leaf, HomeBusy):
            return "home_busy"
        if isinstance(leaf, HomeIdentityMismatch | UnsupportedStoreSchema | NotAMemoryHome):
            return "home_identity_mismatch"
        if isinstance(leaf, SeatLost):
            return (
                "server_unreachable"
                if leaf.reason in ("gave_up", "world_stopped")
                else "seat_unavailable"
            )
        if isinstance(leaf, ConfigError):
            return "config_invalid"
    return None


def _leaves(error: BaseException) -> Iterator[BaseException]:
    if isinstance(error, BaseExceptionGroup):
        for inner in cast(BaseExceptionGroup[BaseException], error).exceptions:
            yield from _leaves(inner)
    else:
        yield error
