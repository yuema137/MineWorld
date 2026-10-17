"""AC-c-2 (INV-S27-2, the adapter half): two handles of one Player, on `visitor` and `wanderer` in different
places, share no reference, name or fact. Each handle's references are its own (`E1` is each one's self,
`E2` each one's place: M-c-2, a table shared across handles, turns this red), and neither result shows a
person or a fact only the other seat perceives.

The world has no save, so neither seat has the reliable stream from the beginning (`cursor_unavailable`):
the Player plays them as a client without the stream does, its inbox fed from each observation's own facts
(D-c-19), which this test also exercises.
"""

from __future__ import annotations

import pytest
from conftest import Start
from mineworld_sdk.wire.frames import Invite
from realserver import INVITE, Transcript, market_town
from support import run

from mineworld_agent import (
    EventsArgs,
    JoinArgs,
    ObserveArgs,
    Player,
    PlayerConfig,
    View,
    WaitArgs,
    WalkToArgs,
)
from mineworld_agent.models import ToPlace, WalkEnded, render_text

pytestmark = pytest.mark.real_server


def names(view: View) -> set[str]:
    return {p.name.text for p in view.here.people if p.name is not None}


def test_the_two_seats_share_no_reference_name_or_event(mineworld: Start) -> None:
    server = mineworld("market-town", *market_town("--pace", "600"))
    transcript = Transcript()

    async def play() -> tuple[View, View, list[str], list[str]]:
        async with Player(PlayerConfig(server=server.url), Invite(INVITE)) as player:
            visitor = transcript(await player.join(JoinArgs(seat="visitor", nickname="one")))
            wanderer = transcript(await player.join(JoinArgs(seat="wanderer", nickname="two")))
            v, w = visitor.handle, wanderer.handle
            for joined in (visitor, wanderer):
                assert (joined.me, joined.view.me.ref) == ("E1", "E1"), "each handle's own table"
                assert joined.view.me.place is not None and joined.view.me.place.ref == "E2"
            vera = visitor.view.me.name
            assert vera is not None and vera.text in names(wanderer.view), "both start in the café"
            street = next(
                e.leads_to.ref
                for e in visitor.view.here.exits
                if any(t.text == "street" for t in e.leads_to.labels)
            )
            went = transcript(await player.walk_to(WalkToArgs(handle=v, to=ToPlace(place=street))))
            assert went.outcome.kind == "accepted", went.outcome
            for _ in range(3):
                ended = transcript(
                    await player.wait(WaitArgs(handle=v, until=WalkEnded(), timeout_s=25))
                )
                if ended.woke == "walk_ended":
                    break
            out = transcript(await player.observe(ObserveArgs(handle=v))).view
            stayed = transcript(await player.observe(ObserveArgs(handle=w))).view
            assert isinstance(out, View) and isinstance(stayed, View)
            seen_by_wanderer = transcript(await player.events(EventsArgs(handle=w, limit=20)))
            seen_by_visitor = transcript(await player.events(EventsArgs(handle=v, limit=20)))
            return (
                out,
                stayed,
                [e.event_type for e in seen_by_visitor.events],
                [e.event_type for e in seen_by_wanderer.events],
            )

    out, stayed, visitor_facts, wanderer_facts = run(play(), timeout_s=80)
    assert out.me.place is not None and stayed.me.place is not None
    assert any(t.text == "street" for t in out.me.place.labels)
    assert any(t.text == "cafe" for t in stayed.me.place.labels)
    vera = out.me.name
    assert vera is not None and vera.text not in names(stayed), "the visitor has left the café"
    assert not names(out) & {"Alice Moreau", "Bob Achterberg"}, (
        "nobody of the café is in the street"
    )
    assert "walk-ended" in visitor_facts, "my walk's end is my fact"
    assert "walk-ended" not in wanderer_facts, (
        "the wanderer did not perceive the visitor's walk end"
    )
    assert transcript.largest() <= 8192
    for result in transcript.results:
        assert INVITE not in result.model_dump_json() and INVITE not in render_text(result)
