"""AP4-9: `embellish` through a real `ModelGateway`, over scripted and replayed backends; no model.

Event ids here are thirteen digits and entity ids ten, so a request that leaked either would show it
as a substring; the structural text's own numbers (days, times, counts) are at most two digits.
"""

from __future__ import annotations

import json
from collections.abc import Callable
from pathlib import Path

import pytest
from memory_fixtures import ALICE, BOB, CAFE, CAROL, DAY, PARK, alice_identity, named, spoke
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityKey
from support import completion, run

from mineworld_cognition.backend.model import BackendFailure, Completion, CompletionRequest
from mineworld_cognition.backend.scripted import ScriptedBackend
from mineworld_cognition.budget import BudgetPolicy, MemoryLedger
from mineworld_cognition.compress.model_summarizer import ModelSummarizer, embellish
from mineworld_cognition.gateway import Budget, ModelGateway
from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.store import MemoryStore
from mineworld_cognition.record import Cassette, CassetteMiss, RecordingBackend, ReplayBackend

BASE = 10**12
SEAT = EntityKey("alice")
T = 9 * 3600


class FakeClock:
    def now(self) -> int:
        return 1_760_000_000


def _facts(last_words: str = "See you.") -> list[PerceivedEvent]:
    """Four closed episodes over two weeks (one chapter) and an open one."""
    b = BASE
    return [
        named(b + 1, BOB, "Bob Achterberg"),
        named(b + 2, CAROL, "Carol Mensah"),
        spoke(b + 10, T, BOB, ALICE, "Morning.", CAFE),
        spoke(b + 11, 2 * DAY + T, CAROL, ALICE, "Rain later?", PARK),
        spoke(b + 12, 8 * DAY + T, BOB, ALICE, last_words, CAFE),
        spoke(b + 13, 9 * DAY + T, CAROL, ALICE, "Bye.", CAFE),
    ]


def _store(facts: list[PerceivedEvent]) -> MemoryStore:
    store = MemoryStore.open(":memory:", alice_identity())
    ingest(store, facts, facts[-1].id)
    return store


def _gateway(
    backend: ScriptedBackend | RecordingBackend | ReplayBackend, **policy: int
) -> ModelGateway:
    return ModelGateway(backend, Budget.of(BudgetPolicy(**policy), MemoryLedger(), FakeClock()))


def _answer(text: str) -> Callable[[CompletionRequest], Completion | BackendFailure]:
    return lambda _request: completion(text)


def _rows(store: MemoryStore) -> list[dict[str, object]]:
    return [json.loads(line) for line in store.dump().decode("utf-8").splitlines()]


def _without_prose(store: MemoryStore) -> list[dict[str, object]]:
    return [
        {k: v for k, v in row.items() if k not in ("prose", "prose_key")} for row in _rows(store)
    ]


def _summaries(store: MemoryStore) -> list[dict[str, object]]:
    return [row for row in _rows(store) if row["table"] in ("episodes", "chapters")]


def test_prose_replaces_text_only_and_never_a_citation() -> None:
    # AP4-9 (a), (b), (f). The mutation of (b) parses a citation from the completion: citations change.
    asked: list[CompletionRequest] = []

    def script(request: CompletionRequest) -> Completion:
        asked.append(request)
        return completion("I spent a quiet morning with Bob at the café.")

    backend = ScriptedBackend(script)
    with _store(_facts()) as store:
        before = _without_prose(store)
        report = run(embellish(store, ModelSummarizer(_gateway(backend), SEAT), limit=100))
        assert _without_prose(store) == before
        summaries = _summaries(store)
    closed = [row for row in summaries if row.get("closed", 1) == 1]
    assert (report.asked, report.stored) == (len(closed), len(closed)) == (5, 5)
    assert backend.calls == 5
    assert all(row["prose"] == "I spent a quiet morning with Bob at the café." for row in closed)
    assert all(
        isinstance(row["prose_key"], str) and len(str(row["prose_key"])) == 64 for row in closed
    )
    ids = [str(BASE + n) for n in (1, 2, 10, 11, 12, 13)] + [ALICE, BOB, CAROL, CAFE, PARK]
    for request in asked:
        assert request.purpose == "summarize"
        text = "\n".join(message.text for message in request.messages)
        assert not [i for i in ids if i in text], text


def test_a_refusal_a_failure_or_no_model_stores_nothing_and_raises_nothing() -> None:
    # AP4-9 (c).
    with _store(_facts()) as store:
        before = store.dump()
        spent = MemoryLedger()
        spent.charge(SEAT, FakeClock().now(), calls=1, tokens=0, estimated=False)
        refusing = ModelGateway(
            ScriptedBackend(_answer("never")),
            Budget.of(BudgetPolicy(calls_per_wall_hour=1), spent, FakeClock()),
        )
        refused = run(embellish(store, ModelSummarizer(refusing, SEAT), limit=100))
        failing = _gateway(ScriptedBackend(lambda _r: BackendFailure(reason="timeout")))
        failed = run(embellish(store, ModelSummarizer(failing, SEAT), limit=100))
        unbound = run(embellish(store, ModelSummarizer(None, SEAT), limit=100))
        assert store.dump() == before
    assert (refused.refused, refused.stored) == (5, 0)
    assert (failed.failed, failed.stored) == (5, 0)
    assert (unbound.unbound, unbound.asked) == (True, 0)


@pytest.mark.parametrize(
    "prose",
    ["x" * 161, "We met on day 123 at the café.", "Bob said hi [#10].", ""],
)
def test_invalid_prose_is_discarded_and_counted(prose: str) -> None:
    # AP4-9 (d): over 160 bytes, a run of three digits, a citation mark, or nothing at all.
    with _store(_facts()) as store:
        before = store.dump()
        report = run(
            embellish(
                store, ModelSummarizer(_gateway(ScriptedBackend(_answer(prose))), SEAT), limit=2
            )
        )
        assert store.dump() == before
    assert (report.asked, report.discarded, report.stored) == (2, 2, 0)


def test_recorded_prose_replays_identically_and_a_changed_text_is_a_miss(tmp_path: Path) -> None:
    # AP4-9 (e). Its mutation catches CassetteMiss as a fallback: the replay passes silently.
    cassette = tmp_path / "prose.jsonl"
    texts = iter(f"A memory, number {word}." for word in ("one", "two", "three", "four", "five"))
    recorder = RecordingBackend(
        ScriptedBackend(lambda _r: completion(next(texts))), cassette, binding="local", model="m"
    )
    with _store(_facts()) as recorded:
        gateway = _gateway(recorder)
        run(embellish(recorded, ModelSummarizer(gateway, SEAT), limit=100))
        run(gateway.aclose())
        expected = _summaries(recorded)
    with _store(_facts()) as replayed:
        replay = _gateway(ReplayBackend(Cassette.load(cassette)))
        run(embellish(replayed, ModelSummarizer(replay, SEAT), limit=100))
        assert _summaries(replayed) == expected
    with _store(_facts(last_words="Something else entirely.")) as changed:
        replay = _gateway(ReplayBackend(Cassette.load(cassette)))
        with pytest.raises(CassetteMiss):
            run(embellish(changed, ModelSummarizer(replay, SEAT), limit=100))
