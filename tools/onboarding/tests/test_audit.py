"""AC-25f-2: the auditor voids a run that read outside the sandbox or used the web, and lists every stuck point
(PROTOCOL.md §6). The transcripts are synthetic, in the agent CLI's stream-JSON shape."""

from __future__ import annotations

import json
import os
from pathlib import Path

import audit


def sandbox(tmp_path: Path, status: str = "DONE\n", gaps: str | None = None) -> Path:
    root = tmp_path / "sandbox"
    for folder in ("guide", "reference", "bin", "work"):
        (root / folder).mkdir(parents=True)
    (root / "work" / "STATUS").write_text(status, encoding="utf-8")
    if gaps is not None:
        (root / "work" / "GAPS.txt").write_text(gaps, encoding="utf-8")
    return root


def tool(name: str, number: int, **given: str) -> dict[str, object]:
    return {
        "type": "assistant",
        "message": {
            "content": [{"type": "tool_use", "id": f"t{number}", "name": name, "input": given}]
        },
    }


def said(text: str) -> dict[str, object]:
    return {"type": "assistant", "message": {"content": [{"type": "text", "text": text}]}}


def answer(number: int, text: str, error: bool = False) -> dict[str, object]:
    return {
        "type": "user",
        "message": {
            "content": [
                {
                    "type": "tool_result",
                    "tool_use_id": f"t{number}",
                    "content": text,
                    "is_error": error,
                }
            ]
        },
    }


def transcript(tmp_path: Path, events: list[dict[str, object]]) -> Path:
    path = tmp_path / "transcript.jsonl"
    path.write_text("".join(json.dumps(event) + "\n" for event in events), encoding="utf-8")
    return path


def test_a_clean_run_is_clean(tmp_path: Path) -> None:
    root = sandbox(tmp_path)
    events = [
        tool("Read", 1, file_path=str(root / "guide" / "README.md")),
        tool("Bash", 2, command="mineworld validate village"),
        answer(2, "village is a valid World Pack."),
    ]
    result = audit.audit(transcript(tmp_path, events), root)
    assert not result.void and not result.stuck, result.render()


def test_a_read_outside_the_sandbox_voids_the_run_by_any_spelling(tmp_path: Path) -> None:
    root = sandbox(tmp_path)
    (tmp_path / "secret.md").write_text("x", encoding="utf-8")
    try:
        os.symlink(tmp_path / "secret.md", root / "work" / "link.md")
        link = str(root / "work" / "link.md")
    except OSError:  # Windows without the symlink privilege: the link case cannot be built there
        link = str(tmp_path / "secret.md")
    events = [
        tool("Read", 1, file_path=str(tmp_path / "secret.md")),
        tool("Read", 2, file_path="../../secret.md"),
        tool("Read", 3, file_path=link),
        tool("Bash", 4, command="cat ~/.claude/CLAUDE.md"),
        tool("Bash", 5, command="cd .. && cd .. && ls"),
        tool("Grep", 6, path="/Users", pattern="world"),
        tool("WebFetch", 7, url="https://example.com"),
    ]
    result = audit.audit(transcript(tmp_path, events), root)
    lines = sorted({item.line for item in result.void})
    assert lines == [1, 2, 3, 4, 5, 6, 7], result.render()


def test_a_repeated_refusal_a_gap_a_guess_and_stuck_are_listed(tmp_path: Path) -> None:
    root = sandbox(
        tmp_path, status="STUCK the guide has no market\n", gaps="how to hold a market\n"
    )
    refusal = "Exit code 1\n[mineworld] village/world.yaml: unknown field `market`, expected one of `world`"
    events = [said("I'll assume a market is a place.")]
    for number in (1, 2, 3):
        events += [
            tool("Bash", number, command="mineworld validate village"),
            answer(number, refusal, True),
        ]
    events.append({"type": "result", "subtype": "error_max_turns"})
    result = audit.audit(transcript(tmp_path, events), root)
    kinds = sorted({item.kind for item in result.stuck})
    assert kinds == ["gap", "guess", "repeated-refusal", "status", "stopped", "unknown-field"], (
        result.render()
    )
    assert not result.void


def test_unknown_field_counts_only_in_a_failed_mineworld_command(tmp_path: Path) -> None:
    root = sandbox(tmp_path)
    events = [
        tool("Bash", 1, command="cat ../reference/world-pack.md"),
        answer(1, "Any other key is refused with `unknown field`, naming the key."),
    ]
    result = audit.audit(transcript(tmp_path, events), root)
    assert not result.stuck, result.render()


def test_absolute_paths_inside_the_sandbox_and_dev_null_are_allowed(tmp_path: Path) -> None:
    root = sandbox(tmp_path)
    events = [
        tool("Bash", 1, command=f"ls {root}/reference > /dev/null && cat {root}/guide/README.md"),
        tool("Bash", 3, command=f"B={root}; ls $B/guide; cd {root}/work; cat ../guide/README.md"),
        tool("Glob", 2, path=str(root / "guide"), pattern="*.md"),
    ]
    result = audit.audit(transcript(tmp_path, events), root)
    assert not result.void, result.render()


def test_a_spelling_through_a_linked_folder_is_still_inside(tmp_path: Path) -> None:
    # macOS gives temporary folders as /var/…, a link to /private/var/…: both spellings name the sandbox.
    root = sandbox(tmp_path)
    alias = tmp_path / "alias"
    try:
        os.symlink(root, alias)
    except OSError:  # Windows without the symlink privilege
        return
    events = [tool("Read", 1, file_path=str(alias / "guide" / "README.md"))]
    result = audit.audit(transcript(tmp_path, events), root)
    assert not result.void, result.render()
