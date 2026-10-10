"""Source and artefact scans: AP5-10 (a) (IC-10, P5's half), AP5-13 (f), AP5-14 (f), AP5-9's source half.

A provider concept — a provider's name, a model family, an endpoint, a key — may appear only in the
adapter, the registry, the preset table and the configuration (I-10). Every committed cassette's key and
request are scanned too, and so is the SDK, which must stay provider-free.
"""

from __future__ import annotations

import json
import re
from pathlib import Path

from mineworld_cognition.backend.providers import PRESETS

PACKAGE = Path(__file__).resolve().parent.parent
SOURCE = PACKAGE / "src" / "mineworld_cognition"
SDK_SOURCE = PACKAGE.parent.parent / "sdk" / "python" / "src"
CASSETTES = PACKAGE / "tests" / "cassettes"
ALLOWED = {
    SOURCE / "backend" / "openai_compatible.py",
    SOURCE / "backend" / "anthropic_messages.py",
    SOURCE / "backend" / "cli_bridge.py",  # its allowlist refuses OPENAI_* and ANTHROPIC_* by name
    SOURCE / "backend" / "registry.py",
    SOURCE / "backend" / "providers.py",
    SOURCE / "config.py",
}
TERMS = [
    "openai",
    "ollama",
    "anthropic",
    "llama",
    "qwen",
    "gemma",
    "mistral",
    "phi",
    "vllm",
    "lmstudio",
    "api_key",
    "http://",
    "https://",
    "11434",
]
# Letters around a term would make it another word ("phi" in "graphics"); a term that ends in a symbol
# matches anywhere.
PATTERN = re.compile(
    "|".join(
        rf"(?<![a-z]){re.escape(term)}(?![a-z])" if term[-1].isalnum() else re.escape(term)
        for term in TERMS
    ),
    re.IGNORECASE,
)


def _findings(path: Path, text: str) -> list[str]:
    return [
        f"{path}:{number}: {match.group(0)}"
        for number, line in enumerate(text.splitlines(), start=1)
        for match in PATTERN.finditer(line)
    ]


def test_no_provider_concept_outside_the_adapter_and_configuration() -> None:
    # AP5-10 (a); its mutation plants "ollama" in gateway.py and expects that file and line here.
    findings: list[str] = []
    for root in (SOURCE, SDK_SOURCE):
        for path in sorted(root.rglob("*.py")):
            if path not in ALLOWED:
                findings += _findings(path, path.read_text(encoding="utf-8"))
    for path in sorted(CASSETTES.glob("*.jsonl")):
        for line in path.read_text(encoding="utf-8").splitlines()[1:]:
            entry = json.loads(line)
            keyed = json.dumps({"key": entry["key"], "request": entry["request"]})
            findings += _findings(path, keyed)
    assert findings == []


def test_the_scan_sees_a_planted_term() -> None:
    # The scan is not vacuous: the pattern finds each term in a line, and not inside another word.
    assert _findings(Path("x.py"), "x = 'Ollama'\ny = graphics") == ["x.py:1: Ollama"]


def test_only_secrets_reads_a_key_file_and_nothing_loads_one_into_the_environment() -> None:
    # AP5-13 (f) and AP5-9's source half.
    importers: list[str] = []
    for path in sorted(SOURCE.rglob("*.py")):
        text = path.read_text(encoding="utf-8")
        assert not re.search(r"load_dotenv\s*\(|import[^\n]*\bload_dotenv", text), path
        assert not re.search(r"[\"'/\\]secrets\.env", text), path  # the operator's key file
        assert ".config/mineworld" not in text, path
        if re.search(r"^\s*(from|import) dotenv\b", text, re.MULTILINE):
            importers.append(path.name)
    assert importers == ["secrets.py"]


def test_only_the_adapter_imports_the_http_client() -> None:
    importers = [
        path.relative_to(SOURCE).as_posix()
        for path in sorted(SOURCE.rglob("*.py"))
        if re.search(r"^\s*(from|import) httpx2\b", path.read_text(encoding="utf-8"), re.MULTILINE)
    ]
    assert importers == ["backend/anthropic_messages.py", "backend/openai_compatible.py"]


def test_the_readme_lists_exactly_the_presets() -> None:
    # AP5-14 (f): the README's provider table names every preset and its key variable, and no other.
    readme = (PACKAGE / "README.md").read_text(encoding="utf-8")
    rows = re.findall(r"^\| `([a-z]+)` \| [^|]+ \| [^|]+ \| `([A-Z_]+)` \|", readme, re.MULTILINE)
    assert sorted(rows) == sorted((name, preset.key_env) for name, preset in PRESETS.items())
