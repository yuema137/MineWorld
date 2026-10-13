"""`AC-10` (IC-4; pr-s10-p4 AP4-10), over the real binary and a real 100-day history.

"After 100 simulated days, character history does not require feeding all historical events to a
model; biography compression stays bounded while original event provenance is retained." (`MVP.md` §9)

One module-scoped run: `mineworld run` (100 days, seed 7) → `mineworld perceived` (Alice, Bob) → Alice's
store on disk, ingested in frames of 256, retrieved at a trigger on days 10, 30, 60 and 100 → (a) to (g),
and AP4-4 (d). The bounds are literals from step-17, never measured values. No model is involved.
"""

from __future__ import annotations

import json
import time
from dataclasses import dataclass

import pytest
from ac10_harness import (
    QUOTE_PREFIX,
    TRIGGER_DAYS,
    History,
    _entity,  # pyright: ignore[reportPrivateUsage]
    _export,  # pyright: ignore[reportPrivateUsage]
    _payload,  # pyright: ignore[reportPrivateUsage]
    expand,
    speaker,
    unlabelled_digits,
    utterance,
)
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityKey, EventId

from mineworld_cognition.memory import (
    MemorySection,
    MemoryStore,
    RecallQuery,
    StoreIdentity,
    ingest,
    retrieve,
)

pytestmark = pytest.mark.real_binary

CEILING = 6000
GROWTH = 1.10
FRAME = 256


@dataclass(frozen=True)
class Run:
    history: History
    sections: dict[int, MemorySection]
    dump: bytes
    fallbacks: int
    store_bytes: int
    whole: bytes
    """A second store, built from the whole export in one frame: shared by (e) and (g) (ledger §14.5)."""


def _identity(history: History) -> StoreIdentity:
    return StoreIdentity(
        world_instance=history.instance, seat=EntityKey("alice"), self_entity=history.me
    )


def _dump(history: History, frame: int) -> bytes:
    with MemoryStore.open(":memory:", _identity(history)) as store:
        for batch in history.frames(frame):
            ingest(store, batch, batch[-1].id)
        return store.dump()


@pytest.fixture(scope="module")
def run(tmp_path_factory: pytest.TempPathFactory) -> Run:
    scratch = tmp_path_factory.mktemp("ac10")
    history = History.build(scratch)
    triggers = {day: fact for day, fact in zip(TRIGGER_DAYS, history.triggers(), strict=True)}
    cuts = {fact.id: day for day, fact in triggers.items()}
    sections: dict[int, MemorySection] = {}
    fallbacks = 0
    path = scratch / "alice.sqlite"
    started = time.monotonic()
    with MemoryStore.open(path, _identity(history)) as store:
        for batch in history.frames(FRAME, tuple(cuts)):
            fallbacks += ingest(store, batch, batch[-1].id).decode_fallbacks
            day = cuts.get(batch[-1].id)
            if day is not None:
                trigger = triggers[day]
                query = RecallQuery(
                    at=trigger.at, counterparts=(speaker(trigger),), words=utterance(trigger)
                )
                sections[day] = retrieve(store, query)
        dump = store.dump()
    history.wall_s["ingest"] = time.monotonic() - started
    print(f"\n[ac10] wall {history.wall_s}; store {path.stat().st_size} bytes on disk")
    whole = _dump(history, len(history.alice))
    return Run(history, sections, dump, fallbacks, path.stat().st_size, whole)


def _rows(dump: bytes, table: str) -> list[dict[str, object]]:
    rows = [json.loads(line) for line in dump.decode("utf-8").splitlines()]
    return [row for row in rows if row["table"] == table]


def _id(key: object) -> str:
    return str(key).lstrip("0") or "0"


def test_a_bounded_every_section_and_no_growth_with_age(run: Run) -> None:
    # Its mutation removes D-P4-9's budgets: a section exceeds 6 000 bytes.
    sizes = {day: len(section.render().encode("utf-8")) for day, section in run.sections.items()}
    print(f"\n[ac10] section bytes {sizes}")
    assert set(sizes) == set(TRIGGER_DAYS)
    assert all(size <= CEILING for size in sizes.values()), sizes
    assert sizes[100] <= GROWTH * sizes[30], sizes


def _cited(dump: bytes, sections: dict[int, MemorySection]) -> set[str]:
    cited = {str(r["event_id"]) for r in _rows(dump, "l0")}
    for row in _rows(dump, "episodes"):
        cited |= {_id(row["first_key"]), _id(row["last_key"])}
    for row in _rows(dump, "stable"):
        cited |= {_id(row["first_key"]), _id(row["last_key"])}
        if row["level_key"] is not None:
            cited.add(_id(row["level_key"]))
    cited |= {_id(row["source_key"]) for row in _rows(dump, "names") if row["source_key"]}
    for section in sections.values():
        cited |= {r.first for r in section.citations()} | {r.last for r in section.citations()}
    return cited


def test_b_every_citation_resolves_to_a_fact_alice_perceived(run: Run) -> None:
    # Its mutation feeds the union of everyone's exports: an id outside Alice's export is cited.
    assert _cited(run.dump, run.sections) <= set(run.history.ids())


def test_c_coverage_is_exact_open_tail_and_closed_episodes(run: Run) -> None:
    # Its mutations disable the roll-up, or drop one closed episode's range: coverage breaks.
    ids = run.history.ids()
    order = {event: index for index, event in enumerate(ids)}
    episodes = sorted(_rows(run.dump, "episodes"), key=lambda row: int(str(row["episode"])))
    covered: list[EventId] = []
    for row in episodes:
        covered += expand(order, ids, _id(row["first_key"]), _id(row["last_key"]))
    assert covered == ids
    (open_episode,) = [row for row in episodes if row["closed"] == 0]
    assert open_episode is episodes[-1]
    tail = expand(order, ids, _id(open_episode["first_key"]), _id(open_episode["last_key"]))
    assert len({run.history.alice[order[e]].at // 86400 for e in tail}) == 1


def _texts(run: Run) -> list[str]:
    texts = [str(row["gist"]) for row in _rows(run.dump, "l0")]
    texts += [str(row["text"]) for row in _rows(run.dump, "episodes")]
    texts += [str(row["text"]) for row in _rows(run.dump, "chapters")]
    texts += [_uncut(line.text) for section in run.sections.values() for line in section.lines]
    return texts


def _uncut(text: str) -> str:
    """A section line without the `; `-separated part a 160-byte cut ended in (`… 1 other came an…`):
    a labelled field cut in half no longer reads as one. Its whole text is in the dump, checked whole."""
    return text.rsplit("; ", 1)[0] if text.endswith("…") and "; " in text else text


LISTENERS = ("carol", "dev", "erin", "felix", "grace", "hana", "ivan", "visitor", "wanderer")
"""Whose exports test_d searches after Bob's, in the pack's population order (Alice and Otto excepted:
Otto drives no seat and hears little)."""


LOCATED = 4
"""How many unperceived lines test_d locates: four, not five, since people walk (step-11 §21.15 M-6, the
primary session's ruling, 2026-10-10; pr-s10-p4-memory.md's note). Every one keeps both checks."""


def _unperceived(history: History, count: int = LOCATED) -> list[PerceivedEvent]:
    """Bob's lines first (the harness's own search), then other people's, by the same rule: a `spoke`
    fact in their export, absent from Alice's, at a place other than hers at that moment, whose words she
    never heard said by anyone. Deduplicated by fact id, since one line reaches several listeners.

    Widened on step-11 §21.15 M-6. Since people walk (12n-2), they talk more, and Alice hears more of the
    fixed lines. In the 100-day history Bob's export holds two such facts, and all ten listeners'
    exports together hold four. The rule is unchanged; only the exports searched grow."""
    located = history.unperceived(count)
    if len(located) == count:
        return located
    mine = set(history.ids())
    heard = "\n".join(utterance(f) for f in history.alice if f.event_type == "spoke")
    where = [
        (fact.at, fact.place.entity)
        for fact in history.alice
        if fact.event_type in ("arrived", "person-entered-place")
        and fact.place is not None
        and _entity(_payload(fact)["person"]) == history.me
    ]
    seen = {fact.id for fact in located}
    for person in LISTENERS:
        for fact in _export(history.save, person):
            if fact.event_type != "spoke" or fact.id in mine or fact.id in seen:
                continue
            if fact.place is None:
                continue
            hers = [place for at, place in where if at <= fact.at]
            if not hers or hers[-1] == fact.place.entity:
                continue
            if utterance(fact)[:QUOTE_PREFIX] in heard:
                continue
            located.append(fact)
            seen.add(fact.id)
            if len(located) == count:
                return located
    return located


def test_d_nothing_alice_did_not_perceive_appears_anywhere(run: Run) -> None:
    located = _unperceived(run.history)
    assert len(located) == LOCATED
    held = {str(row["event_id"]) for row in _rows(run.dump, "l0")}
    joined = "\n".join(_texts(run))
    for fact in located:
        assert fact.id not in held
        assert utterance(fact)[:QUOTE_PREFIX] not in joined


def test_e_two_stores_from_one_export_dump_the_same_bytes(run: Run) -> None:
    # The file store (frames of 256, cut at the triggers) and an in-memory one built separately.
    assert run.whole == run.dump


def test_f_each_counterpart_reaches_back_to_the_first_fact_naming_them(run: Run) -> None:
    # Its mutation computes L3 over the last 7 days only: first_key moves.
    earliest = run.history.earliest_naming()
    stable = {str(row["entity"]): _id(row["first_key"]) for row in _rows(run.dump, "stable")}
    assert stable == earliest
    by_name = {name: entity for entity, name in run.history.names.items()}
    lines = [line for line in run.sections[100].lines if line.level == "L3"]
    assert lines
    for line in lines:
        entity = by_name[line.text.split(":", 1)[0]]
        first = min(line.citation.ranges, key=lambda r: int(r.first)).first
        assert first == earliest[entity], line.text


def test_g_batch_boundaries_change_nothing(run: Run) -> None:
    # Its mutation closes the open episode at the end of each batch.
    # Frames of 1, of 256 (the file store) and of the whole export (the store (e) shares).
    assert _dump(run.history, 1) == run.dump
    assert run.whole == run.dump


def test_no_text_holds_an_entity_id_and_every_payload_decoded(run: Run) -> None:
    # AP4-4 (d), P4-5. Its mutation renders an unnamed person as `entity-<id>`: the line is named.
    # Entity ids here are short decimals ("7"), so the oracle is stricter than a token match: outside
    # the labelled fields of the templates, no digit at all may remain (ledger §14.3 X-4).
    leaks = [found for text in _texts(run) if (found := unlabelled_digits(text)) is not None]
    assert leaks == []
    assert run.fallbacks == 0
