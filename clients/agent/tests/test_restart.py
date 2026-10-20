"""AC-c-8 (INV-S27-7): what an agent does is journaled and survives a server restart, as a human's does.

A Player with a `state_dir` plays the visitor — walks up to Alice and says hello — and is then abandoned
mid-play, as a crashed program is (no `leave`: its state file stays). The server is killed and started again
on the same save. A new Player with the same `state_dir` joins the visitor: the file's secret is dead after
a restart (`PROTOCOL.md` §4.2), so the seat is joined afresh (`hosted` under `--town`), from the file's
cursor; the Person is where the save left it, `mineworld biography` keeps the acquaintance from before, and
the new inbox starts after the facts the first Player already took — no repeat (M-c-7, a state file never
rewritten after its first welcome, turns this red). `--cadence 600` keeps the town's controllers from moving
the visitor while nobody holds it.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from conftest import Start
from mineworld_sdk.wire.frames import Invite
from realserver import INVITE, Wire, biography, market_town, perceived_export
from support import run

from mineworld_agent import (
    EventsArgs,
    JoinArgs,
    ObserveArgs,
    Player,
    PlayerConfig,
    SayArgs,
    View,
    WaitArgs,
    WalkToArgs,
)
from mineworld_agent.models import EventLine, ToPerson, WalkEnded

pytestmark = pytest.mark.real_server

WORLD = "market-town"


async def every_event(player: Player, handle: str) -> list[EventLine]:
    lines: list[EventLine] = []
    after: str | None = None
    while True:
        page = await player.events(EventsArgs(handle=handle, after=after, limit=20))
        lines += page.events
        if page.next is None:
            return lines
        after = page.next


type Where = tuple[tuple[str, ...], int, int] | None
"""A position as two handles can compare it: references are each handle's own, so the place is named by
its labels, and the point by its millimetres."""


def where(view: View) -> Where:
    me = view.me
    if me.place is None or me.position is None:
        return None
    return tuple(t.text for t in me.place.labels), me.position.x_mm, me.position.y_mm


def run_of(lines: list[EventLine], exported: list[tuple[int, str]]) -> int:
    """Where `lines` appear in `exported` as one unbroken run; fails when they do not."""
    mine = [(line.at, str(line.event_type)) for line in lines]
    found = [i for i in range(len(exported) - len(mine) + 1) if exported[i : i + len(mine)] == mine]
    assert found, (mine, exported)
    return found[0]


def test_a_player_rejoins_its_seat_after_a_server_restart(mineworld: Start, tmp_path: Path) -> None:
    save, state = tmp_path / "save", tmp_path / "state"
    flags = market_town("--save", str(save), "--hold", "10", "--cadence", "600")
    first = mineworld(WORLD, *flags)
    config = PlayerConfig(server=first.url, state_dir=state)

    async def before() -> tuple[Where, list[EventLine]]:
        player = Player(config, Invite(INVITE))  # never closed: the program "crashes" below
        joined = await player.join(JoinArgs(seat="visitor", nickname="before"))
        h = joined.handle
        alice = next(
            p.ref
            for p in joined.view.here.people
            if p.name is not None and p.name.text.startswith("Alice")
        )
        near = await player.walk_to(WalkToArgs(handle=h, to=ToPerson(person=alice)))
        assert near.outcome.kind == "accepted", near.outcome
        for _ in range(3):
            if (await player.wait(WaitArgs(handle=h, until=WalkEnded(), timeout_s=25))).woke == (
                "walk_ended"
            ):
                break
        said = await player.say(SayArgs(handle=h, to=alice, words="hello before the restart"))
        assert said.outcome.kind == "accepted", said.outcome
        lines = said.outcome.events + await every_event(player, h)
        seen = (await player.observe(ObserveArgs(handle=h))).view
        assert isinstance(seen, View)
        return where(seen), sorted(lines, key=lambda line: int(line.ref[1:]))

    left_at, earlier = run(before(), timeout_s=80)
    saved = json.loads((state / "visitor.json").read_text(encoding="utf-8"))
    first.stop()
    second = mineworld(WORLD, *flags)
    wire = Wire(second)

    async def after() -> tuple[str, Where, list[EventLine]]:
        again = PlayerConfig(server=second.url, state_dir=state)
        async with Player(again, Invite(INVITE), connector=wire) as player:
            joined = await player.join(JoinArgs(seat="visitor", nickname="after"))
            lines = await every_event(player, joined.handle)
            return joined.took_over, where(joined.view), lines

    took_over, found_at, later = run(after(), timeout_s=60)
    second.stop()
    joins = [frame for _, frame in wire.taps[0].sent if frame.get("t") == "join"]
    assert joins[0].get("resume") == saved["resume"], "the file's secret was presented"
    assert joins[0].get("perceived") == {"since": saved["cursor"]}, "from the file's cursor"
    assert took_over == "hosted", "secrets do not survive a restart: joined afresh"
    assert left_at is not None and found_at == left_at, "the Person is where the save left it"
    assert any(
        '"became-acquainted"' in line and '"alice"' in line
        for line in biography(WORLD, save, "visitor")
    ), "the acquaintance the hello began is in the visitor's biography"
    exported = [(int(e.at), str(e.event_type)) for e in perceived_export(WORLD, save, "visitor")]
    start = run_of(earlier, exported)
    print(f"AC-c-8: {len(earlier)} facts before the restart, {len(later)} after")
    if later:
        assert run_of(later, exported) >= start + len(earlier), "no fact taken before is repeated"
