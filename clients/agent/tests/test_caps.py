"""I-c-13 (step-27 §5.10): every rendering fits 8 KiB of JSON and 8 KiB of text; a crowd is paged, and a
page too large for the cap is shortened — never cut mid-field — and continued."""

from __future__ import annotations

import pytest
from recorded import NAMES, crowd, decoded, observation

from mineworld_agent.models import InboxCounts, View, json_bytes, render_text
from mineworld_agent.refs import RefTable
from mineworld_agent.view import Caps, Moment, Rendered, page_of, render, view_of

MOMENT = Moment(at=86_399, time_scale=12, paused=False, since=360)
INBOX = InboxCounts(unread=512, dropped=9_999)
CAP = 8192


def fits(view: View) -> None:
    assert json_bytes(view) <= CAP, json_bytes(view)
    assert len(render_text(view).encode("utf-8")) <= CAP


@pytest.mark.parametrize("name", NAMES)
def test_every_recorded_view_fits(name: str) -> None:
    fits(view_of(render(observation(name), RefTable(), 1, True), MOMENT, INBOX, Caps()))


def every_page(view: View, rendered: Rendered) -> list[View]:
    pages, pending = [view], list(view.more)
    while pending:
        page = page_of(rendered, pending.pop(0), MOMENT, INBOX, Caps())
        pages.append(page)
        pending += page.more
    return pages


def test_a_crowd_of_two_hundred_is_paged_under_the_cap() -> None:
    value = crowd(people=100, things=100, offers=200)
    rendered = render(decoded(value), RefTable(), 3, True)
    pages = every_page(view_of(rendered, MOMENT, INBOX, Caps()), rendered)
    for page in pages:
        fits(page)
    assert sum(len(p.offers) for p in pages) == len(value["affordances"])
    assert sum(len(p.here.people) for p in pages) == 100 + 3
    assert sum(len(p.here.things) for p in pages) == 100


def test_long_names_shorten_a_page_and_it_continues_where_it_stopped() -> None:
    # 24 people whose names are 300 bytes each cannot fit one page: the fit shows fewer and continues.
    value = crowd(people=24, things=0, offers=0, name="N" * 300)
    rendered = render(decoded(value), RefTable(), 1, True)
    first = view_of(rendered, MOMENT, INBOX, Caps())
    fits(first)
    shown = len(first.here.people)
    assert shown < 24 + 3
    (more,) = [p for p in first.more if p.list == "people"]
    assert more.start == shown
    pages = every_page(first, rendered)
    refs = [p.ref for page in pages for p in page.here.people]
    assert len(refs) == len(set(refs)) == 24 + 3
    names = [p.name.text for page in pages for p in page.here.people if p.name is not None]
    assert all(n.startswith("N" * 300) or len(n) < 40 for n in names), "never cut inside a name"
