"""The inbox status `Player.inbox` reports (pr-s27d §5.3(b)), and the hook a handle calls when its inbox gains facts.

The status follows the adds, the bound and the reads. The hook fires once per batch that added facts, after the
facts are in the inbox (the hook reads `arrived`), and never for an empty batch. M-d-2 (the hook called before
`Inbox.add`) turns `the_hook_sees_the_new_fact` red."""

from __future__ import annotations

from typing import cast

from mineworld_sdk.perceived import CursorCell, PerceivedBatch
from mineworld_sdk.resuming import ResumingSeat
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EntityKey, EventId
from test_inbox import ME, fact

from mineworld_agent.handle import Handle
from mineworld_agent.inbox import Inbox
from mineworld_agent.refs import RefTable


def test_the_status_follows_the_adds_the_bound_and_the_reads() -> None:
    inbox = Inbox(RefTable(), EntityId(ME), bound=2)
    assert (inbox.unread, inbox.dropped, inbox.newest) == (0, 0, None)
    inbox.add([fact(1, "tick", {})])
    assert (inbox.unread, inbox.dropped, inbox.newest) == (1, 0, "F1")
    inbox.add([fact(2, "tick", {}), fact(3, "tick", {})])
    assert (inbox.unread, inbox.dropped, inbox.newest) == (2, 1, "F3")
    inbox.events(None, 10)
    assert (inbox.unread, inbox.dropped, inbox.newest) == (0, 1, "F3")


async def _no_sleep(_seconds: float) -> None:
    raise AssertionError("these tests never wait")


def _handle() -> Handle:
    refs = RefTable()
    return Handle(
        seat=cast(ResumingSeat, None),
        key=EntityKey("visitor"),
        refs=refs,
        inbox=Inbox(refs, EntityId(ME)),
        cursor=CursorCell(),
        wall=lambda: 0.0,
        sleep=_no_sleep,
        idle_release_s=0,
        states=None,
    )


def _batch(events: list[PerceivedEvent], through: str) -> PerceivedBatch:
    return PerceivedBatch(events=tuple(events), through=EventId(through), connection=1)


def _ingest(handle: Handle, batch: PerceivedBatch) -> None:
    """The drain's seam: the one place a batch enters the inbox, and the hook is observed there."""
    handle._ingest(batch)  # pyright: ignore[reportPrivateUsage]


def test_the_hook_sees_the_new_fact() -> None:
    seen: list[int] = []
    handle = _handle()
    handle.on_inbox = lambda: seen.append(handle.inbox.arrived)
    _ingest(handle, _batch([fact(1, "tick", {}), fact(2, "tick", {})], "102"))
    assert seen == [2]


def test_the_hook_is_called_once_per_batch_that_added_facts() -> None:
    calls: list[int] = []
    handle = _handle()
    handle.on_inbox = lambda: calls.append(handle.inbox.arrived)
    _ingest(handle, _batch([fact(1, "tick", {})], "101"))
    _ingest(handle, _batch([fact(2, "tick", {}), fact(3, "tick", {})], "103"))
    assert calls == [1, 3]


def test_an_empty_batch_does_not_call_the_hook() -> None:
    calls: list[int] = []
    handle = _handle()
    handle.on_inbox = lambda: calls.append(handle.inbox.arrived)
    _ingest(handle, _batch([], "100"))
    assert calls == []
