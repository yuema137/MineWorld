"""The published agent reference is current (pr-s27f §5.3, I-f-4, AC-f-4).

`docs/reference/agent-tools.json` and `agent-tools.md` are exactly what `mineworld-agent schema` and `schema
--markdown` generate from the adapter's models now, and the package's copy of guide chapter 14 (served as
`mineworld://guide/play-as-an-agent`) is exactly the chapter. A failure names the file and the command that
regenerates it."""

from __future__ import annotations

from pathlib import Path

from mineworld_agent.mcp_server import GUIDE_FILE, catalogue_json, reference_markdown

REPOSITORY = Path(__file__).resolve().parents[3]
REFERENCE = REPOSITORY / "docs" / "reference"
CHAPTER = REPOSITORY / "docs" / "guide" / "14-play-as-an-agent.md"
PACKAGED = REPOSITORY / "clients" / "agent" / "src" / "mineworld_agent" / "resources" / GUIDE_FILE


def _current(path: Path, generated: str, command: str) -> None:
    published = path.read_bytes()
    assert published == generated.encode("utf-8"), (
        f"{path.relative_to(REPOSITORY).as_posix()} is not what the adapter generates now; regenerate it from the "
        f"repository root with `{command}`"
    )


def test_the_json_reference_is_current() -> None:
    _current(
        REFERENCE / "agent-tools.json",
        catalogue_json(),
        "uv run --locked mineworld-agent schema --out docs/reference/agent-tools.json",
    )


def test_the_markdown_reference_is_current() -> None:
    _current(
        REFERENCE / "agent-tools.md",
        reference_markdown(),
        "uv run --locked mineworld-agent schema --markdown --out docs/reference/agent-tools.md",
    )


def test_the_packaged_chapter_is_the_chapter() -> None:
    assert PACKAGED.read_bytes() == CHAPTER.read_bytes(), (
        "the adapter's copy of guide chapter 14 differs from the chapter; copy it again: "
        "cp docs/guide/14-play-as-an-agent.md clients/agent/src/mineworld_agent/resources/play-as-an-agent.md"
    )
