"""AP4-11: memory reads only what it is given (I-3) and holds no provider concept (I-10)."""

from __future__ import annotations

import ast
import re
from pathlib import Path

from memory_fixtures import ALICE, BOB, CAFE, named, rows, seat_view, spoke
from mineworld_sdk.wire.ids import EntityKey
from support import completion, run
from test_provider_scan import ALLOWED, PATTERN, SOURCE

from mineworld_cognition.backend.scripted import ScriptedBackend
from mineworld_cognition.budget import BudgetPolicy, MemoryLedger
from mineworld_cognition.compress.model_summarizer import ModelSummarizer, embellish
from mineworld_cognition.gateway import Budget, ModelGateway
from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.record import RecordingBackend

PACKAGES = ("memory", "compress", "forgetting")
"""`memory/` includes the retention host (`memory/retention/`); `forgetting/` holds the bundled policies
(pr-fg-b I-1 … I-3 extended, §2.3)."""
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
    # AP4-11 (a). `sqlite3.connect` appears once, in the home, at the path its caller gives.
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
        # The home connects to its caller's path (pr-s28a I-a-7); the FTS5 probe to a throwaway in-memory
        # database.
        for connect in re.findall(r"sqlite3\.connect\([^)]*\)", text):
            home = path == SOURCE / "memory" / "home" / "home.py"
            if not home and connect != 'sqlite3.connect(":memory:")':
                findings.append(f"{path.name}: {connect}")
        imported.update(
            re.findall(r"^\s*(?:from|import) (mineworld_cognition\.\S+)", text, re.MULTILINE)
        )
    assert findings == []
    outside = {
        name
        for name in imported
        if not name.startswith(
            (
                "mineworld_cognition.memory",
                "mineworld_cognition.compress",
                "mineworld_cognition.forgetting",
            )
        )
    }
    assert outside == COGNITION_IMPORTS


WRITES = re.compile(
    r"\b(?:INSERT\s+INTO|DELETE\s+FROM|UPDATE)\s+(fact_who|fact|access_text|access)\b",
    re.IGNORECASE,
)
WRITERS = {
    SOURCE / "memory" / "home" / "home.py",
    SOURCE / "memory" / "home" / "view.py",
    SOURCE / "memory" / "home" / "export.py",
    SOURCE / "memory" / "scopes" / "world.py",
}
"""The only modules that write the shared records and the access index (pr-s28a §5.1; AC-a-11, AS28-14); the
seat import (`export.py`, pr-s28e §5.7) writes the access rows and their index through the home's transaction."""


def test_only_the_home_writes_the_shared_records_and_the_access_index() -> None:
    # AC-a-11 (AS28-14). Its mutation (M-a-11) adds an `INSERT INTO access` to compress/episodes.py.
    writers = {path for path in _sources() if WRITES.search(path.read_text(encoding="utf-8"))}
    assert writers <= WRITERS, sorted(str(path) for path in writers - WRITERS)
    assert SOURCE / "memory" / "scopes" / "world.py" in writers  # the scan sees a real writer


SQL_WRITES = re.compile(r"\b(?:INSERT|UPDATE|DELETE|CREATE|DROP)\b")


def test_the_inspector_holds_no_sql_write() -> None:
    # AC-e-11 (pr-s28e §5.1): `inspect/` reads a home and writes to one only through `import_seat`; no SQL
    # write appears in its text. The planted line shows the scan is not vacuous.
    assert SQL_WRITES.search('db.execute("DELETE FROM access")')
    sources = sorted((SOURCE / "inspect").rglob("*.py"))
    assert sources
    findings = [
        f"{path.name}: {match.group(0)}"
        for path in sources
        for match in SQL_WRITES.finditer(path.read_text(encoding="utf-8"))
    ]
    assert findings == []


def test_memory_reads_no_save_and_no_world_log() -> None:
    # AC-a-11 (AS28-13, I-S28-7): no module of memory, compression or forgetting imports the persistence layer
    # or names a save's files; the planted line shows the scan is not vacuous.
    pattern = re.compile(
        r"\bpersistence\b|\bmineworld_persistence\b|\.save\b|save\.sqlite|world_log"
    )
    assert pattern.search("from mineworld_persistence import Save")
    findings = [
        f"{path.name}: {match.group(0)}"
        for path in _sources()
        for match in pattern.finditer(path.read_text(encoding="utf-8"))
    ]
    assert findings == []


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
    with seat_view() as store:
        ingest(store, facts, facts[-1].id)
        run(embellish(store, ModelSummarizer(gateway, EntityKey("alice")), limit=100))
        run(gateway.aclose())
        dump = store.dump()
    assert "qwen-test" in cassette.read_text(encoding="utf-8")
    assert "A quiet morning with Bob." in {row["prose"] for row in rows(dump, "episodes")}
    assert PATTERN.findall(dump.decode("utf-8")) == []


class _Clock:
    def now(self) -> int:
        return 1_760_000_000


FORBIDDEN_IN_SCOPE_SIGNATURES = re.compile(
    r"\b(Connection|Path|Clock|ModelGateway|SeatView|sqlite3)\b"
)
"""What no memory scope's signature may name (pr-s28c AC-c-10; AS28-14's signature half, I-S28-8)."""


def _annotations(tree: ast.AST) -> list[str]:
    found: list[str] = []
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            arguments = [*node.args.posonlyargs, *node.args.args, *node.args.kwonlyargs]
            found += [ast.unparse(a.annotation) for a in arguments if a.annotation is not None]
            if node.returns is not None:
                found.append(ast.unparse(node.returns))
        elif isinstance(node, ast.AnnAssign):
            found.append(ast.unparse(node.annotation))
    return found


def test_no_scope_signature_names_a_connection_path_clock_or_gateway() -> None:
    # AC-c-10. Its mutation (M-c-13) gives `ScopeView.record` a `sqlite3.Connection` parameter.
    interface = SOURCE / "memory" / "scopes" / "interface.py"
    annotations = _annotations(ast.parse(interface.read_text(encoding="utf-8")))
    assert annotations, "the scan reads the interface's annotations"
    assert [text for text in annotations if FORBIDDEN_IN_SCOPE_SIGNATURES.search(text)] == []
    planted = "def record(self, db: sqlite3.Connection) -> None: ..."
    assert FORBIDDEN_IN_SCOPE_SIGNATURES.search(" ".join(_annotations(ast.parse(planted))))
