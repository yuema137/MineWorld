"""`mineworld-memory`: inspect a stopped memory home, and export or import one seat (pr-s28e §5.8).

    mineworld-memory inspect HOME --seat KEY access [--event ID] [--perceived FILE] [--json]
    mineworld-memory inspect HOME --seat KEY forgotten [--json]
    mineworld-memory inspect HOME [--seat KEY] bytes [--decisions FILE] [--json]
    mineworld-memory inspect HOME --seat KEY recall --at T [--with E]… [--words TEXT] [--trace [--trace-all]]
                             [--decisions FILE] [--config FILE] [--perceived FILE] [--json]
    mineworld-memory export HOME --seat KEY [--out FILE]
    mineworld-memory import HOME --world-instance ID FILE

Exit codes: 0 the report; 2 a refusal, its named error's message on stderr; 3 an `incoherent` explanation (the
report is still printed). Output is UTF-8 with `\\n` line ends on every platform.
"""

from __future__ import annotations

import argparse
import sys
from collections.abc import Callable, Sequence
from pathlib import Path

from mineworld_sdk.wire.ids import EntityKey, EventId, WorldInstanceId

from mineworld_cognition.errors import ConfigError
from mineworld_cognition.inspect import report
from mineworld_cognition.inspect.inputs import InspectRefusal, read_inputs
from mineworld_cognition.inspect.report import AccessReport, InspectReport, RecallReport
from mineworld_cognition.inspect.requests import (
    AccessRequest,
    BytesRequest,
    ForgottenRequest,
    InspectRequest,
    RecallRequest,
    ScopesRequest,
)
from mineworld_cognition.memory.home import (
    HomeBusy,
    HomeIdentityMismatch,
    HomeNeedsRecovery,
    MemoryHome,
    NotAMemoryHome,
    NotASeatExport,
    SeatAlreadyInHome,
    SeatNotInHome,
    UnsupportedExportFormat,
    UnsupportedStoreSchema,
    export_seat,
    import_seat,
)
from mineworld_cognition.memory.retention.registry import PolicyError
from mineworld_cognition.memory.scopes.world import ConflictingFact

PROG = "mineworld-memory"
REFUSALS: tuple[type[Exception], ...] = (
    ConfigError,
    ConflictingFact,
    HomeBusy,
    HomeIdentityMismatch,
    HomeNeedsRecovery,
    InspectRefusal,
    NotAMemoryHome,
    NotASeatExport,
    PolicyError,
    SeatAlreadyInHome,
    SeatNotInHome,
    UnsupportedExportFormat,
    UnsupportedStoreSchema,
)
"""Every named error a command refuses with (exit 2)."""

type Command = Callable[[argparse.Namespace], int]


def write(text: str) -> None:
    """UTF-8 and `\\n` on every platform: the bytes, not a text stream that would translate line ends."""
    sys.stdout.buffer.write(text.encode("utf-8"))
    sys.stdout.buffer.flush()


def _incoherent(found: InspectReport) -> bool:
    if isinstance(found, AccessReport):
        return found.explanation is not None and found.explanation.kind == "incoherent"
    if isinstance(found, RecallReport):
        return any(
            each is not None and each.kind == "incoherent"
            for line in found.trace
            for cited in line.ranges
            for each in (cited.first, cited.last, *cited.shown)
        )
    return False


def _request(arguments: argparse.Namespace) -> InspectRequest:
    seat: EntityKey | None = arguments.seat
    view: str = arguments.view
    if view == "bytes":
        return BytesRequest(seat=seat)
    if seat is None:
        raise InspectRefusal(f"the {view} view describes one seat: pass --seat KEY")
    if view == "access":
        event: EventId | None = arguments.event
        return AccessRequest(seat=seat, event=event)
    if view == "recall":
        counterparts: list[str] = arguments.counterparts or []
        return RecallRequest(
            seat=seat,
            at=arguments.at,
            counterparts=tuple(counterparts),
            words=arguments.words,
            trace=arguments.trace,
            trace_all=arguments.trace_all,
        )
    if view == "scopes":
        return ScopesRequest(seat=seat)
    return ForgottenRequest(seat=seat)


def _inspect(arguments: argparse.Namespace) -> int:
    request = _request(arguments)
    inputs = read_inputs(
        arguments.seat,
        perceived=getattr(arguments, "perceived", None),
        decisions=getattr(arguments, "decisions", None),
        config=getattr(arguments, "config", None),
    )
    with MemoryHome.open_read_only(arguments.home) as home:
        found = report(home, request, inputs)
    write(found.model_dump_json() + "\n" if arguments.json else found.text())
    return 3 if _incoherent(found) else 0


def _export(arguments: argparse.Namespace) -> int:
    """Reads the home read-only, as `inspect` does; writes the export to `--out` or to stdout."""
    with MemoryHome.open_read_only(arguments.home) as home:
        data = export_seat(home, arguments.seat)
    out: Path | None = arguments.out
    if out is None:
        sys.stdout.buffer.write(data)
        sys.stdout.buffer.flush()
    else:
        out.write_bytes(data)
    return 0


def _import(arguments: argparse.Namespace) -> int:
    """Opens the target home for writing (created when absent) and adds the exported seat to it."""
    path: Path = arguments.file
    try:
        data = path.read_bytes()
    except OSError as error:
        raise InspectRefusal(
            f"{path}: cannot read the seat export ({type(error).__name__})"
        ) from None
    with MemoryHome.open(arguments.home, WorldInstanceId(arguments.world_instance)) as home:
        seat = import_seat(home, data)
    print(f"imported seat {seat}", file=sys.stderr)
    return 0


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog=PROG,
        description="Inspect a stopped memory home, or move one seat between homes (S28-e).",
    )
    commands = parser.add_subparsers(dest="command", required=True)
    inspect = commands.add_parser(
        "inspect", help="report on a stopped home; reads only, never writes"
    )
    inspect.add_argument("home", type=Path, metavar="HOME", help="the memory home's file")
    inspect.add_argument("--seat", type=EntityKey, metavar="KEY", help="the seat's key")
    inspect.set_defaults(run=_inspect)
    views = inspect.add_subparsers(dest="view", required=True, metavar="VIEW")
    access = views.add_parser("access", help="what the seat holds and how; or why one id")
    access.add_argument("--event", type=EventId, metavar="ID", help="explain one event id")
    access.add_argument(
        "--perceived",
        type=Path,
        metavar="FILE",
        help="the seat's `mineworld perceived --json` output: makes the answer exact",
    )
    views.add_parser("forgotten", help="the forgotten ranges, their policies, and the pass log")
    sizes = views.add_parser("bytes", help="where the bytes are: tables, largest rows, the home")
    sizes.add_argument(
        "--decisions",
        type=Path,
        metavar="FILE",
        help="`mineworld decisions --json`: the seat's bound (default 20 MiB)",
    )
    recall = views.add_parser(
        "recall", help="the memory section a consult receives, every line traced to its facts"
    )
    recall.add_argument("--at", type=int, required=True, metavar="T", help="the world time")
    recall.add_argument(
        "--with",
        dest="counterparts",
        action="append",
        metavar="E",
        help="a person present: an entity id, or a name the seat knows exactly (repeatable)",
    )
    recall.add_argument("--words", default="", metavar="TEXT", help="the trigger's words")
    recall.add_argument("--trace", action="store_true", help="explain every cited range")
    recall.add_argument(
        "--trace-all", action="store_true", help="with --trace, every held key of a range, not 8"
    )
    recall.add_argument(
        "--decisions", type=Path, metavar="FILE", help="`mineworld decisions --json`"
    )
    recall.add_argument(
        "--config",
        type=Path,
        metavar="FILE",
        help="`cognition.toml`: the seat's recall_episodes (default 8)",
    )
    recall.add_argument(
        "--perceived", type=Path, metavar="FILE", help="the seat's `mineworld perceived --json`"
    )
    stacked = views.add_parser(
        "scopes", help="the installed scopes, the seat's stack and the scope tables it holds"
    )
    stacked.add_argument(
        "--decisions",
        type=Path,
        metavar="FILE",
        help="`mineworld decisions --json`: the seat's stack",
    )
    for each in (access, views.choices["forgotten"], sizes, recall, stacked):
        each.add_argument("--json", action="store_true", help="print the report as JSON")
    export = commands.add_parser(
        "export", help="write one seat as a seat export (reads the home only)"
    )
    export.add_argument("home", type=Path, metavar="HOME", help="the memory home's file")
    export.add_argument("--seat", type=EntityKey, required=True, metavar="KEY")
    export.add_argument(
        "--out", type=Path, metavar="FILE", help="the file to write (stdout by default)"
    )
    export.set_defaults(run=_export)
    load = commands.add_parser(
        "import", help="add an exported seat to a home of the same world instance"
    )
    load.add_argument("home", type=Path, metavar="HOME", help="the memory home (created if absent)")
    load.add_argument(
        "--world-instance", required=True, metavar="ID", help="the home's world instance"
    )
    load.add_argument("file", type=Path, metavar="FILE", help="the seat export")
    load.set_defaults(run=_import)
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    arguments = _parser().parse_args(argv)
    run: Command = arguments.run
    try:
        return run(arguments)
    except REFUSALS as error:
        print(str(error), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
