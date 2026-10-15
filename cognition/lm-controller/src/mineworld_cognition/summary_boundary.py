"""The summary boundary (pr-fg-c §5.2, FGc-6): the seat's gateway as an assisted policy's host sees it.

A replay that holds no recording for a request raises `CassetteMiss` inside the backend. This gateway is
the one place that miss becomes a stop the memory package can name: `SummaryUnavailable`, with the miss as
its cause. Memory never imports `record`; this module sits above both, so the dependency runs one way.
Only the assisted seat's gateway is wrapped here. A compression path's `CassetteMiss` is unchanged
(AP4-9 (e)): it propagates as before.
"""

from __future__ import annotations

from mineworld_sdk.wire.ids import EntityKey

from mineworld_cognition.backend.model import CompletionRequest
from mineworld_cognition.gateway import GateOutcome, ModelGateway
from mineworld_cognition.memory.retention.assisted import SummaryUnavailable
from mineworld_cognition.record import CassetteMiss


class SummaryBoundaryGateway(ModelGateway):
    """A `ModelGateway` whose cassette miss is `SummaryUnavailable`, chained to the original miss."""

    async def complete(self, seat: EntityKey, request: CompletionRequest) -> GateOutcome:
        try:
            return await super().complete(seat, request)
        except CassetteMiss as miss:
            raise SummaryUnavailable(
                "the recording holds no answer for this summary request"
            ) from miss
