"""AP4-7: the memory section is bounded, structured first and lexical last, literal and deterministic."""

from __future__ import annotations

import re
from collections.abc import Generator
from contextlib import contextmanager
from pathlib import Path

import pytest
from memory_fixtures import (
    ALICE,
    BOB,
    CAFE,
    CAROL,
    DAY,
    DEV,
    PARK,
    named,
    names_of_everyone,
    seat_view,
    spoke,
)
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EventId

from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.retrieve import MemorySection, RecallQuery, retrieve

RETRIEVE = Path(__file__).resolve().parent.parent / "src/mineworld_cognition/memory/retrieve.py"
HOUR = 3600
# The literals of D-P4-9, written here rather than read from the code under test.
CEILING = 6000
LINE = 160
SECTIONS = {
    # pr-p6c-fgd §4.1: `with` takes 1 000 of L0's 2 000 bytes; the ceiling is unchanged.
    "with": ("What {name} and I said to each other before:", 6, 1000),
    "L3": ("People I know:", 16, 1600),
    "L2": ("Recent weeks:", 4, 800),
    "L1": ("Episodes I remember:", 8, 1600),
    "L0": ("The last day:", 48, 1000),
}


@contextmanager
def _store(facts: list[PerceivedEvent]) -> Generator[SeatView]:
    with seat_view() as store:
        for start in range(0, len(facts), 256):
            batch = facts[start : start + 256]
            ingest(store, batch, batch[-1].id)
        yield store


def _adversarial() -> list[PerceivedEvent]:
    """Sixty days of 480-byte, three-byte-per-character utterances from twenty long-named people."""
    people = [EntityId(f"71000000{n:02d}") for n in range(20)]
    facts = [named(n + 1, person, f"Person {'é' * 50} {n:02d}") for n, person in enumerate(people)]
    event = 100
    for day in range(60):
        for hour in range(8, 18):
            speaker = people[(day + hour) % 20]
            utterance = "雨" * 150 + f" umbrella {day} {hour}" + "雨" * 5
            facts.append(spoke(event, day * DAY + hour * HOUR, speaker, ALICE, utterance, CAFE))
            event += 1
    return facts


def _section_bytes(section: MemorySection, level: str) -> tuple[int, int]:
    header = SECTIONS[level][0].replace("{name}", section.speaker or "")
    lines = [line.render() for line in section.lines if line.level == level]
    if not lines:
        return 0, 0
    return len(lines), len(header.encode()) + 1 + sum(len(text.encode()) + 1 for text in lines)


def test_every_section_and_the_whole_stay_within_their_budgets() -> None:
    # AP4-7 (a). Its mutation drops the byte budget: the adversarial store exceeds 6 000 bytes.
    with _store(_adversarial()) as store:
        query = RecallQuery(
            at=59 * DAY + 17 * HOUR,
            counterparts=(EntityId("7100000003"),),
            words="umbrella 雨",
            speaker=EntityId("7100000003"),
        )
        section = retrieve(store, query)
    rendered = section.render()
    assert len(rendered.encode("utf-8")) <= CEILING
    for level, (header, most, budget) in SECTIONS.items():
        count, size = _section_bytes(section, level)
        assert count <= most, level
        assert size <= budget, level
        if count:
            assert header.replace("{name}", section.speaker or "") in rendered
    assert all(len(line.render().encode("utf-8")) <= LINE for line in section.lines)
    # The store holds far more than the section shows, and every section is used.
    assert {line.level for line in section.lines} == {"with", "L3", "L2", "L1", "L0"}


def _with_carol() -> list[PerceivedEvent]:
    """Carol and I on day 0 (#10, #11) and once a day from day 3 to day 9 (#20 … #26, #26 within the last day
    before day 10); a line between Bob and Carol I overheard (#12); Carol's line just heard (#40), and a name
    learned in the last day (#41)."""
    t = 9 * HOUR
    return [
        *names_of_everyone(),
        spoke(10, t, CAROL, ALICE, "I left a red umbrella here.", CAFE),
        spoke(11, t + 60, ALICE, CAROL, "I'll keep it behind the counter.", CAFE),
        spoke(12, t + 120, BOB, CAROL, "Is that your umbrella?", CAFE),
        *(spoke(17 + d, d * DAY + t, CAROL, ALICE, f"Day {d}.", CAFE) for d in range(3, 10)),
        spoke(40, 10 * DAY, CAROL, ALICE, "Hello again.", CAFE),
        named(41, DEV, "Dev Raman"),
    ]


def test_the_speakers_first_and_newest_exchanges_come_first() -> None:
    # AP6c-1 (pr-p6c-fgd §4.1): the `with` section is first; it holds Carol's first two exchanges and then her
    # newest, oldest first; neither the overheard line nor the line just heard; and what it shows is not repeated
    # under "The last day". Its mutation drops the first-words read (newest only): #10 and #11 are missing.
    with _store(_with_carol()) as store:
        section = retrieve(
            store,
            # No counterparts, so no episode of Carol's covers the last day's records: L0 shows them.
            RecallQuery(at=10 * DAY, speaker=CAROL, heard=EventId("40")),
        )
    shown = [line.citation.token() for line in section.lines if line.level == "with"]
    assert shown == ["[#10]", "[#11]", "[#23]", "[#24]", "[#25]", "[#26]"]
    assert section.lines[0].level == "with"
    rendered = section.render()
    assert rendered.startswith("What Carol Mensah and I said to each other before:\n")
    recent = [line.citation.token() for line in section.lines if line.level == "L0"]
    assert "[#26]" not in recent and "[#40]" in recent
    assert len(rendered.encode("utf-8")) <= CEILING


def test_without_a_speaker_there_is_no_with_section() -> None:
    # The section is the speaker's: a consult on an arrival without a known person, or a retrieval with no
    # speaker, shows none, and the L0 section is as before.
    with _store(_with_carol()) as store:
        section = retrieve(store, RecallQuery(at=10 * DAY, counterparts=(CAROL,)))
    assert all(line.level != "with" for line in section.lines)
    assert section.speaker is None


def _talks() -> list[PerceivedEvent]:
    """Bob long ago, Carol every day since; an umbrella said three times, rain once."""
    t = 9 * HOUR
    return [
        *names_of_everyone(),
        spoke(
            10, t, BOB, ALICE, "Bring an umbrella and a coat, it will rain.", CAFE
        ),  # E2 (2 terms)
        spoke(
            20, 3 * DAY + t, CAROL, ALICE, "I lost my umbrella near here, that's the gist.", CAFE
        ),
        spoke(30, 6 * DAY + t, CAROL, ALICE, "Umbrella weather, foo.", CAFE),  # E4, as E5's time
        spoke(31, 6 * DAY + t, CAROL, ALICE, "Umbrellas everywhere.", PARK),  # E5: no whole term
        spoke(32, 6 * DAY + t, CAROL, ALICE, "An umbrella!", PARK),  # E5 (one term)
        *(spoke(40 + d, d * DAY + t, CAROL, ALICE, "Nice day.", CAFE) for d in range(8, 20)),
    ]


def _l1(section: MemorySection) -> list[str]:
    return [line.citation.token() for line in section.lines if line.level == "L1"]


def test_a_counterpart_in_the_query_brings_their_episodes_first() -> None:
    # AP4-7 (b): Bob's one episode, months old, comes before Carol's recent ones.
    with _store(_talks()) as store:
        section = retrieve(store, RecallQuery(at=20 * DAY, counterparts=(BOB,)))
    assert _l1(section)[:2] == ["[#10]", "[#1-4]"]


def test_words_find_older_episodes_ranked_by_terms_then_recency_then_first_id() -> None:
    # AP4-7 (c): "umbrella" and "rain" match episode [#10] twice; then, by recency, the two episodes of
    # day 7 at the same hour — ties broken by first id, [#30] before [#31-32] — then [#20].
    with _store(_talks()) as store:
        section = retrieve(store, RecallQuery(at=20 * DAY, words="Umbrella? Rain!"))
    assert _l1(section) == ["[#10]", "[#30]", "[#31-32]", "[#20]"]


@pytest.mark.parametrize("words", ['umbrella" OR "x', "NEAR(a b)", "gist:foo", "*", "-x"])
def test_hostile_words_are_matched_as_literal_terms(words: str) -> None:
    # AP4-7 (d), P4-4. Its mutation passes the words unquoted to MATCH: OperationalError.
    with _store(_talks()) as store:
        section = retrieve(store, RecallQuery(at=20 * DAY, words=words))
    # Terms are the words' runs of letters and digits, three or more long: `umbrella` (one term each, so
    # recency ranks); `near`, not the NEAR operator; `gist` and `foo`, not a column filter; none for `*`
    # and `-x`.
    expected = {
        'umbrella" OR "x': ["[#30]", "[#31-32]", "[#20]", "[#10]"],
        "NEAR(a b)": ["[#20]"],
        "gist:foo": ["[#30]", "[#20]"],
        "*": [],
        "-x": [],
    }[words]
    assert _l1(section) == expected


def test_retrieval_is_deterministic_and_reads_no_clock_randomness_or_float_score() -> None:
    # AP4-7 (e), P4-3.
    with _store(_talks()) as store:
        query = RecallQuery(at=20 * DAY, counterparts=(CAROL,), words="umbrella")
        assert retrieve(store, query) == retrieve(store, query)
    source = RETRIEVE.read_text(encoding="utf-8")
    assert not re.search(r"^\s*(import|from)\s+(time|datetime|random)\b", source, re.MULTILINE)
    assert "bm25" not in source.replace("`bm25()` is never read", "")


def test_the_section_cites_what_its_lines_stand_for() -> None:
    with _store(_talks()) as store:
        section = retrieve(store, RecallQuery(at=20 * DAY, counterparts=(BOB,)))
    cited = {(r.first, r.last) for r in section.citations()}
    assert (EventId("10"), EventId("10")) in cited
    assert all(line.citation.token() in section.render() for line in section.lines)
