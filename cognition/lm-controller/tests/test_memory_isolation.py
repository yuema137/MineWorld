"""AP4-11: memory reads only what it is given (I-3) and holds no provider concept (I-10)."""

from __future__ import annotations

import re
from pathlib import Path

from memory_fixtures import ALICE, BOB, CAFE, alice_identity, named, spoke
from mineworld_sdk.wire.ids import EntityKey
from support import completion, run
from test_provider_scan import ALLOWED, PATTERN, SOURCE

from mineworld_cognition.backend.scripted import ScriptedBackend
from mineworld_cognition.budget import BudgetPolicy, MemoryLedger
from mineworld_cognition.compress.model_summarizer import ModelSummarizer, embellish
from mineworld_cognition.gateway import Budget, ModelGateway
from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.store import MemoryStore
from mineworld_cognition.record import RecordingBackend

PACKAGES = ("memory", "compress")
# What memory and compression may import from the rest of the package: the model types and the
# gateway (ModelSummarizer), and the cassette key it stores beside prose (ledger §14.3 X-3).
COGNITION_IMPORTS = {
    "mineworld_cognition.backend.canonical",
    "mineworld_cognition.backend.model",
    "mineworld_cognition.gateway",
}


def _sources() -> list[Path]:
    return [path for package in PACKAGES for path in sorted((SOURCE / package).rglob("*.py"))]


def test_memory_opens_no_save_runs_no_process_and_imports_no_configuration_or_adapter() -> None:
    # AP4-11 (a). `sqlite3.connect` appears once, in the store, at the path its caller gives.
    findings: list[str] = []
    imported: set[str] = set()
    for path in _sources():
        text = path.read_text(encoding="utf-8")
        for pattern in (
            r"\bsubprocess\b",
            r"world\.sqlite",
            r"(?<![\w.])(?<!def )open\(",
            r"\bos\.",
            r"\bPath\(",
        ):
            findings += [f"{path.name}: {m.group(0)}" for m in re.finditer(pattern, text)]
        # The store connects to its caller's path; the FTS5 probe to a throwaway in-memory database.
        for connect in re.findall(r"sqlite3\.connect\([^)]*\)", text):
            if path.name != "store.py" and connect != 'sqlite3.connect(":memory:")':
                findings.append(f"{path.name}: {connect}")
        imported.update(
            re.findall(r"^\s*(?:from|import) (mineworld_cognition\.\S+)", text, re.MULTILINE)
        )
    assert findings == []
    outside = {
        name
        for name in imported
        if not name.startswith(("mineworld_cognition.memory", "mineworld_cognition.compress"))
    }
    assert outside == COGNITION_IMPORTS


def test_memory_and_compression_are_inside_the_provider_scan() -> None:
    # AP4-11 (c): test_provider_scan.py scans every source file outside ALLOWED.
    assert _sources()
    assert not [path for path in _sources() if path in ALLOWED]


def test_an_embellished_store_holds_no_provider_concept(tmp_path: Path) -> None:
    # AP4-11 (b): embellished through a binding named `local` whose model is `qwen-test`, which the
    # cassette records. Its mutation stores the entry's `meta.model` beside the prose: `qwen` is found.
    facts = [
        named(1, BOB, "Bob Achterberg"),
        spoke(10, 9 * 3600, BOB, ALICE, "Morning.", CAFE),
        spoke(11, 3 * 86400, BOB, ALICE, "Again.", CAFE),
    ]
    cassette = tmp_path / "prose.jsonl"
    recorder = RecordingBackend(
        ScriptedBackend(lambda _r: completion("A quiet morning with Bob.")),
        cassette,
        binding="local",
        model="qwen-test",
    )
    gateway = ModelGateway(recorder, Budget.of(BudgetPolicy(), MemoryLedger(), _Clock()))
    with MemoryStore.open(":memory:", alice_identity()) as store:
        ingest(store, facts, facts[-1].id)
        run(embellish(store, ModelSummarizer(gateway, EntityKey("alice")), limit=100))
        run(gateway.aclose())
        dump = store.dump().decode("utf-8")
    assert "qwen-test" in cassette.read_text(encoding="utf-8")
    assert '"prose":"A quiet morning with Bob."' in dump
    assert PATTERN.findall(dump) == []


class _Clock:
    def now(self) -> int:
        return 1_760_000_000
