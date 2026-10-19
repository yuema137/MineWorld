"""Guide chapter 14 shows the recorded session verbatim, and holds no credential (pr-s27f §5.2 "The page", I-f-3's
page half, I-f-6; AC-f-2, AC-f-9).

The runner of the guide has no include directive (F-S27f-3), so the page marks the transcript with a plain HTML
comment, `<!-- agent-session: examples/agent-session.json -->`, and this test compares the next fenced block with
the fixture byte for byte, in the English page and in its mirror."""

from __future__ import annotations

import re
from pathlib import Path

import pytest

REPOSITORY = Path(__file__).resolve().parents[3]
GUIDE = REPOSITORY / "docs" / "guide"
PAGES = (GUIDE / "14-play-as-an-agent.md", GUIDE / "14-play-as-an-agent.zh-CN.md")
MARK = re.compile(r"^<!-- agent-session: (?P<path>\S+) -->$")
FENCE = re.compile(r"^(`{3,})")
SCANNED = (
    *PAGES,
    GUIDE / "examples" / "agent-session.json",
    REPOSITORY / "docs" / "reference" / "agent-tools.md",
    REPOSITORY / "docs" / "reference" / "agent-tools.json",
)
CREDENTIALS = (
    ("a 32-hex-character token", re.compile(r"(?<![0-9A-Fa-f])[0-9A-Fa-f]{32}(?![0-9A-Fa-f])")),
    ("an invite value other than [..]", re.compile(r"invite=(?!\[\.\.\])\S")),
    ("a bearer other than an environment expansion", re.compile(r"Bearer (?!\$\{)\S")),
)


def shown_session(page: Path) -> tuple[Path, str]:
    """The fixture the page names and the content of the fenced block after the mark."""
    lines = page.read_text(encoding="utf-8").split("\n")
    for index, line in enumerate(lines):
        mark = MARK.match(line)
        if mark is None:
            continue
        rest = lines[index + 1 :]
        opening = next(number for number, text in enumerate(rest) if FENCE.match(text))
        fence = FENCE.match(rest[opening])
        assert fence is not None
        closing = next(
            number
            for number, text in enumerate(rest[opening + 1 :], start=opening + 1)
            if text.startswith(fence.group(1)) and not text.strip("`")
        )
        content = "\n".join(rest[opening + 1 : closing]) + "\n"
        return page.parent / mark.group("path"), content
    raise AssertionError(f"{page.name} has no `<!-- agent-session: … -->` mark")


@pytest.mark.parametrize("page", PAGES, ids=lambda page: page.name)
def test_the_page_shows_the_fixture(page: Path) -> None:
    fixture, shown = shown_session(page)
    assert fixture.is_file(), fixture
    assert shown.encode("utf-8") == fixture.read_bytes(), (
        f"{page.name}: the transcript block differs from {fixture.relative_to(REPOSITORY).as_posix()}; "
        "paste the fixture's content into the block verbatim"
    )


@pytest.mark.parametrize("path", SCANNED, ids=lambda path: path.name)
def test_no_credential_is_written_down(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for what, pattern in CREDENTIALS:
        found = pattern.search(text)
        assert found is None, f"{path.name}: {what} at offset {found.start() if found else -1}"
