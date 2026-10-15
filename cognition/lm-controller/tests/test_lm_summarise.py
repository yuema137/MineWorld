"""`lm-summarise` (pr-fg-c §§5.4-5.5; AFGc-1, -2, -3, -6, -7 (a), -8), scripted and deterministic.

Every answer comes from a `ScriptedBackend`. A group's importance is a pure function of its request text
(§7.2), so the decision table's three columns are all exercised on the synthetic stream. No model is reached
(QS10-19).
"""

from __future__ import annotations

import ast
import hashlib
import json
import re
from collections.abc import Sequence
from pathlib import Path

from memory_fixtures import alice_identity
from mineworld_sdk.wire.contract import PerceivedEvent
from retention_driver import built
from retention_oracle import check_coverage
from retention_stream import PEOPLE, PLACES, stream
from support import run

from mineworld_cognition.backend.model import (
    BackendFailure,
    Completion,
    CompletionRequest,
    ModelBackend,
    Usage,
)
from mineworld_cognition.backend.scripted import ScriptedBackend
from mineworld_cognition.budget import BudgetPolicy, MemoryLedger
from mineworld_cognition.compress.model_summarizer import INSTRUCTIONS
from mineworld_cognition.forgetting.consolidate import Consolidate, ConsolidateParameters
from mineworld_cognition.forgetting.lm_summarise import LmSummarise, LmSummariseParameters
from mineworld_cognition.gateway import Budget, ModelGateway
from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.retention import AssistedPlan, RetentionBudget, SummaryGateway
from mineworld_cognition.memory.retention.items import extract
from mineworld_cognition.memory.retention.registry import PolicyRegistry
from mineworld_cognition.memory.retention.selection import MemoryPolicyChoice, Retention, resolve
from mineworld_cognition.memory.store import MemoryStore

SEED = 11
DAYS = 12
SENTENCES = (
    "We talked about the rain and the market.",
    "Then we went on about the café and the letter.",
    "Later we spoke of bread, music and the garden.",
    "We agreed on a coffee and an umbrella.",
    "The river came up in a conversation.",
    "We ran into each other near the lamp.",
    "Someone mentioned a zeppelin over the park.",
    "I told them about the tomorrow plan.",
    "We kept the noon meeting short.",
    "We made up after a long quiet afternoon.",
)
"""One fixed sentence per importance: names only, no digit run, no citation mark (§7.2)."""
SOURCE = Path(__file__).resolve().parent.parent / "src" / "mineworld_cognition" / "forgetting"
ALLOWED_IMPORTS = (
    "__future__",
    "collections.abc",
    "dataclasses",
    "hashlib",
    "typing",
    "pydantic",
    "mineworld_sdk.wire.ids",
    "mineworld_cognition.backend.canonical",
    "mineworld_cognition.backend.model",
    "mineworld_cognition.compress.model_summarizer",
    "mineworld_cognition.forgetting.consolidate",
    "mineworld_cognition.gateway",
    "mineworld_cognition.memory.retention",
)
"""AFGc-7 (a): what `lm_summarise.py` may import, by module. The bundled policies import public names only
(FGb-4); `consolidate` is the policy's own grouping, and the summarise constants are P4's (ledger F-FGc-1)."""


class FakeClock:
    def now(self) -> int:
        return 1_760_000_000


def _usage() -> Usage:
    return Usage(input_tokens=1, output_tokens=1, estimated=False)


def _constant(importance: int) -> Completion:
    return Completion(
        text=json.dumps({"summary": SENTENCES[importance - 1], "importance": importance}),
        finish="complete",
        usage=_usage(),
    )


def _by_text(request: CompletionRequest) -> int:
    digest = hashlib.sha256(request.messages[-1].text.encode("utf-8")).digest()
    return digest[0] % 10 + 1


def scripted(seen: list[CompletionRequest] | None = None) -> ScriptedBackend:
    def answer(request: CompletionRequest) -> Completion:
        if seen is not None:
            seen.append(request)
        return _constant(_by_text(request))

    return ScriptedBackend(answer)


def gateway(backend: ModelBackend, **policy: int) -> ModelGateway:
    return ModelGateway(backend, Budget.of(BudgetPolicy(**policy), MemoryLedger(), FakeClock()))


GENEROUS = {"calls_per_wall_hour": 10_000_000, "tokens_per_wall_day": 1_000_000_000}


def retention(gateway: ModelGateway | None, **parameters: int) -> Retention:
    choice = MemoryPolicyChoice.model_validate({"policy": "lm-summarise", "parameters": parameters})
    return resolve(choice, PolicyRegistry.installed(), gateway)


def _consolidated() -> Retention:
    return resolve(MemoryPolicyChoice(policy="consolidate"), PolicyRegistry.installed())


def _facts() -> list[PerceivedEvent]:
    return list(stream(SEED, DAYS))


def _ids(facts: Sequence[PerceivedEvent]) -> list[str]:
    """Every id the generator emitted: its Event IDs and its entities' ids (the AFGc-1 oracle)."""
    return [str(fact.id) for fact in facts] + [str(each) for each in (*PEOPLE, *PLACES)]


def _rows(dump: bytes) -> list[dict[str, object]]:
    return [json.loads(line) for line in dump.decode("utf-8").splitlines()]


def _ingested(facts: Sequence[PerceivedEvent]) -> MemoryStore:
    """Every fact ingested and no pass run: the groups `consolidate` would form are all present, so the
    policy's plan can be read against them directly."""
    store = MemoryStore.open(":memory:", alice_identity())
    for start in range(0, len(facts), 256):
        batch = facts[start : start + 256]
        ingest(store, batch, batch[-1].id)
    return store


def _plan(
    store: MemoryStore,
    backend: ScriptedBackend,
    calls: int,
    **parameters: int,
) -> AssistedPlan:
    """The policy's plan over the store's items, asked through a real gateway over `backend`."""
    items = extract(store).items
    now = max(item.last_at for item in items)
    budget = RetentionBudget.model_construct(store_bytes=0, now=now, calls_left=calls)
    summary = SummaryGateway(gateway(backend, **GENEROUS), store.identity.seat, calls)
    fallback = Consolidate().plan(items, budget, ConsolidateParameters())
    params = LmSummariseParameters.model_validate(parameters)
    return run(LmSummarise().plan(items, budget, params, summary, fallback), timeout_s=120)


def test_the_installed_registry_lists_lm_summarise_as_assisted() -> None:
    # AFGc-2 (the registry half): assisted, with `consolidate` as its fallback, found by its entry point.
    bound = PolicyRegistry.installed().bind("lm-summarise", {})
    assert bound.assisted
    assert bound.fallback_id == "consolidate"
    assert not PolicyRegistry.installed().bind("consolidate", {}).assisted


def test_a_request_holds_names_only_and_is_bounded() -> None:
    # AFGc-1 over the 12-day stream: every request is a summary of at most 6 000 bytes that holds no id the
    # generator emitted; the truncation marker appears where the moments would not fit.
    facts = _facts()
    seen: list[CompletionRequest] = []
    built(facts, retention(gateway(scripted(seen), **GENEROUS), calls_per_sim_day=24), 256)
    assert seen
    assert all(request.purpose == "summarize" for request in seen)
    sizes = [sum(len(m.text.encode("utf-8")) for m in r.messages) for r in seen]
    assert max(sizes) <= 6_000
    ids = _ids(facts)
    for request in seen:
        text = " ".join(message.text for message in request.messages)
        assert not any(each in text for each in ids)
    assert any(re.search(r"\(\d+ more\)$", r.messages[-1].text) for r in seen)


def test_the_request_text_is_the_literal_of_section_5_4() -> None:
    # The system text is P4's instructions plus the importance question; the user text is the container's
    # line, then one `- ` line per moment; the schema is the integer-only shape of §5.4.
    seen: list[CompletionRequest] = []
    store = _ingested(_facts()[:400])
    _plan(store, scripted(seen), calls=24)
    store.close()
    assert seen
    first = seen[0]
    system, user = first.messages
    assert system.role == "system"
    assert system.text.startswith(INSTRUCTIONS + " Then rate from 1 to 10")
    assert user.role == "user"
    assert user.text.startswith("So far: ")
    assert "\nMoments:" in user.text
    assert first.output_schema == {
        "type": "object",
        "properties": {
            "summary": {"type": "string"},
            "importance": {"type": "integer", "minimum": 1, "maximum": 10},
        },
        "required": ["summary", "importance"],
        "additionalProperties": False,
    }


def test_the_decision_table_is_applied_cell_by_cell() -> None:
    # AFGc-3 (a) to (c) on §5.4's table, one constant importance per run over the same groups. 9 keeps every
    # group's container and members, with prose; 1 forgets a top-level container and merges a nested group,
    # with no prose; 5 keeps the fallback's merge, with prose on a surviving container.
    store = _ingested(_facts())
    for importance in (9, 1, 5):
        backend = ScriptedBackend(lambda _r, value=importance: _constant(value))
        answered = _plan(store, backend, calls=24)
        assert answered.answers, "the stream has groups to ask"
        assert answered.invalid == 0
        for answer in answered.answers:
            kinds = {type(decision).__name__ for decision in answer.decisions}
            if importance == 9:
                assert kinds == {"Keep"}
                assert answer.prose is not None
            elif importance == 1:
                assert kinds in ({"Drop"}, {"Merge"})
                assert answer.prose is None
            else:
                assert kinds == {"Merge"}
                assert answer.prose is not None
    store.close()


def test_importance_nine_keeps_detail_and_the_citations_are_unchanged() -> None:
    # AFGc-3 (a), (b), (d) over the stream: with every group rated 9, every record stays, the store still
    # covers every perceived fact, and no episode's or chapter's citation differs from the no-model store.
    facts = _facts()
    kept = built(
        facts, retention(gateway(_constant_backend(9), **GENEROUS), calls_per_sim_day=24), 256
    )
    plain = built(facts, _consolidated(), 256)
    check_coverage(kept, [each.id for each in facts])
    records = [row for row in _rows(kept) if row["table"] == "l0"]
    assert len(records) > len([row for row in _rows(plain) if row["table"] == "l0"])
    assert _citations(kept) == _citations(plain)


def _constant_backend(importance: int) -> ScriptedBackend:
    return ScriptedBackend(lambda _r: _constant(importance))


def _citations(dump: bytes) -> list[tuple[object, object, object]]:
    return [
        (row.get("episode", row.get("week")), row.get("citation"), row.get("text"))
        for row in _rows(dump)
        if row["table"] in ("episodes", "chapters")
    ]


def test_importance_one_forgets_top_level_units_and_keeps_coverage() -> None:
    # AFGc-3's second case over the same stream: a top-level unit rated 1 is dropped and its citation joins
    # the forgotten set; the store still covers every perceived fact (its forgotten ids stay resolvable).
    facts = _facts()
    dump = built(
        facts, retention(gateway(_constant_backend(1), **GENEROUS), calls_per_sim_day=24), 256
    )
    assert [row for row in _rows(dump) if row["table"] == "forgotten"]
    check_coverage(dump, [each.id for each in facts])


def test_a_call_allowance_is_exact_per_pass() -> None:
    # AFGc-6 with the policy's own allowance: with more groups than the allowance, exactly the allowance
    # is asked, and the plan says the allowance was exhausted.
    store = _ingested(_facts())
    backend = scripted()
    answered = _plan(store, backend, calls=24, calls_per_sim_day=24)
    store.close()
    assert backend.calls == 24
    assert answered.stopped == "exhausted"


def test_invalid_answers_each_keep_the_fallback_and_are_counted() -> None:
    # AFGc-8: a non-JSON answer, importance 11, a 200-byte summary, a summary with `#12`, and `finish`
    # "length". Each group is invalid, keeps the fallback, and stores no prose.
    bad = (
        Completion(text="not json", finish="complete", usage=_usage()),
        Completion(text='{"summary": "We talked.", "importance": 11}', finish="complete", usage=_usage()),
        Completion(text=json.dumps({"summary": "a" * 200, "importance": 5}), finish="complete", usage=_usage()),
        Completion(text=json.dumps({"summary": "see #12 here", "importance": 5}), finish="complete", usage=_usage()),
        Completion(text=json.dumps({"summary": "We talked.", "importance": 5}), finish="length", usage=_usage()),
    )  # fmt: skip
    asked = 0

    def answer(_request: CompletionRequest) -> Completion:
        nonlocal asked
        asked += 1
        return bad[(asked - 1) % len(bad)]

    store = _ingested(_facts())
    backend = ScriptedBackend(answer)
    answered = _plan(store, backend, calls=24, calls_per_sim_day=24)
    store.close()
    assert answered.answers == ()
    assert answered.invalid == backend.calls
    assert backend.calls >= 5
    assert answered.stopped == "exhausted"


def test_a_failing_backend_stops_the_pass() -> None:
    # AFGc-5 (failed): the first failure stops the remaining groups, which fall back whole.
    store = _ingested(_facts())
    backend = ScriptedBackend(lambda _r: BackendFailure(reason="unreachable"))
    answered = _plan(store, backend, calls=24, calls_per_sim_day=24)
    store.close()
    assert answered.stopped == "failed"
    assert backend.calls == 1


def test_the_policy_imports_only_the_allowed_surface() -> None:
    # AFGc-7 (a): by static reading, the module imports nothing of an adapter, config, record, secrets or
    # budget module.
    tree = ast.parse((SOURCE / "lm_summarise.py").read_text(encoding="utf-8"))
    modules: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.ImportFrom) and node.module is not None:
            modules.add(node.module)
        elif isinstance(node, ast.Import):
            modules.update(alias.name for alias in node.names)
    for module in modules:
        assert any(module == each or module.startswith(each + ".") for each in ALLOWED_IMPORTS), (
            module
        )
    for forbidden in ("mineworld_cognition.budget", "mineworld_cognition.record"):
        assert forbidden not in modules
    assert not any(module.startswith("mineworld_cognition.config") for module in modules)
    assert not any(module.startswith("mineworld_cognition.secrets") for module in modules)


def test_an_unbound_tier_falls_back_whole() -> None:
    # FGc-1 by behaviour: with no `summarize` gateway, a pass asks nothing and its store is consolidate's.
    facts = _facts()
    unbound = built(facts, retention(None, calls_per_sim_day=24), 256)
    plain = built(facts, _consolidated(), 256)
    assert [row for row in _rows(unbound) if row["table"] != "retention_passes"] == [
        row for row in _rows(plain) if row["table"] != "retention_passes"
    ]
