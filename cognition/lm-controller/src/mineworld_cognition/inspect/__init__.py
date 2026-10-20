"""Memory inspection (pr-s28e-memory-inspect.md; step-28 §5.6; `AC-18`): one typed `InspectReport` per request,
from the API (`report`) or the command `mineworld-memory inspect` (`inspect/cli.py`), one implementation.

`report(home, request, inputs)` takes any open home: a stopped one opened with `MemoryHome.open_read_only`, or a
running process's own. It reads only; this package writes nothing to a home except through `import_seat`.
"""

from __future__ import annotations

from mineworld_cognition.inspect import views
from mineworld_cognition.inspect.inputs import Inputs, InspectRefusal, read_inputs
from mineworld_cognition.inspect.report import (
    AccessReport,
    BytesReport,
    ForgottenReport,
    InspectReport,
    RecallReport,
)
from mineworld_cognition.inspect.requests import (
    AccessRequest,
    BytesRequest,
    ForgottenRequest,
    InspectRequest,
    RecallRequest,
)
from mineworld_cognition.memory.home import MemoryHome


def report(
    home: MemoryHome, request: InspectRequest, inputs: Inputs | None = None
) -> InspectReport:
    """The one implementation of every view (§5.4)."""
    given = Inputs() if inputs is None else inputs
    match request:
        case AccessRequest():
            return views.access(home, request.seat, request.event, given)
        case ForgottenRequest():
            return views.forgotten(home, request.seat)
        case BytesRequest():
            return views.bytes_view(home, request.seat, given)
        case RecallRequest():
            return views.recall(home, request, given)


__all__ = [
    "AccessReport",
    "AccessRequest",
    "BytesReport",
    "BytesRequest",
    "ForgottenReport",
    "ForgottenRequest",
    "Inputs",
    "InspectRefusal",
    "InspectReport",
    "InspectRequest",
    "RecallReport",
    "RecallRequest",
    "read_inputs",
    "report",
]
