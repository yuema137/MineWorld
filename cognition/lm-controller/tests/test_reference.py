"""The generated cognition reference (S25 PR 25b): no drift, every field described, every example loads.

The checked-in pages under `docs/reference/` are compared byte for byte with what the models generate now
(AC-25b-1). A field with no description and an example the loader refuses both stop generation (AC-25b-2,
AC-25b-3). The examples printed on the page are the same ones the loader is asked to accept.
"""

from __future__ import annotations

import importlib.util
import re
import sys
from pathlib import Path
from types import ModuleType

import pytest
from pydantic import BaseModel, Field

from mineworld_cognition import config
from mineworld_cognition.config import CognitionConfig

REPOSITORY = Path(__file__).resolve().parents[3]
REFERENCE = REPOSITORY / "docs" / "reference"
TOOL = Path(__file__).resolve().parent.parent / "tools" / "reference.py"
TOML_BLOCK = re.compile(r"```toml\n(.*?)```", re.DOTALL)


def _tool() -> ModuleType:
    spec = importlib.util.spec_from_file_location("reference_tool", TOOL)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules["reference_tool"] = module
    spec.loader.exec_module(module)
    return module


reference = _tool()


def test_checked_in_reference_matches_the_models() -> None:
    stale = [
        relative
        for relative, text in reference.generate().items()
        if (REFERENCE / relative).read_text(encoding="utf-8") != text
    ]
    assert stale == [], f"docs/reference is stale; regenerate it: {stale}"


def test_generation_is_deterministic() -> None:
    assert reference.generate() == reference.generate()


def test_every_field_of_the_configuration_models_has_a_description() -> None:
    assert reference.undescribed_fields(CognitionConfig) == []


def test_a_field_without_a_description_stops_generation_naming_it() -> None:
    class Undescribed(BaseModel):
        limit: int = 1

    class Outer(BaseModel):
        inner: Undescribed = Field(description="a nested model")

    with pytest.raises(reference.GenerationError, match=r"Undescribed\.limit"):
        reference.check_described(Outer)


def test_every_example_on_the_page_is_loaded_by_the_loader(tmp_path: Path) -> None:
    page = (REFERENCE / reference.PAGE).read_text(encoding="utf-8")
    blocks = TOML_BLOCK.findall(page)
    assert len(blocks) == len(reference.EXAMPLES) > 0
    for index, block in enumerate(blocks):
        path = tmp_path / f"example-{index}.toml"
        path.write_text(block, encoding="utf-8", newline="\n")
        config.load(path)


def test_an_example_the_loader_refuses_stops_generation(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    refused = reference.Example("refused", '[recording]\nmode = "live"\nunknown = 1\n')
    monkeypatch.setattr(reference, "EXAMPLES", (refused,))
    with pytest.raises(reference.GenerationError, match="refused by the loader"):
        reference.check_examples()
