"""Audit an onboarding transcript: is the run void, and where did the agent get stuck?

    python3 tools/onboarding/audit.py TRANSCRIPT --sandbox SANDBOX [--json]

TRANSCRIPT is the agent CLI's stream of JSON lines. `tools/onboarding/PROTOCOL.md` §6 is the authority for the
items. Exit 0 when the run is not void and has no stuck point, 1 when it has stuck points, 3 when it is void.
Standard library only.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import cast

HERE = Path(__file__).resolve().parent
GUESSES = HERE / "guess_phrases.txt"

# Tools that reach the network: any use voids the run (INV-S25-7).
WEB_TOOLS = {"WebFetch", "WebSearch"}
# File tools and the input keys that name a path.
PATH_KEYS = {
    "Read": "file_path",
    "Write": "file_path",
    "Edit": "file_path",
    "NotebookEdit": "notebook_path",
    "Glob": "path",
    "Grep": "path",
}
# Absolute paths a shell command may name without reading anything of the machine's.
HARMLESS = {"/dev/null", "/dev/stdout", "/dev/stderr", "/dev/stdin"}
# Shell words that read the environment or the home folder.
ESCAPES = re.compile(r"(^|[^\w])(~|\$HOME|\$\{HOME\}|\$USERPROFILE|%USERPROFILE%)")
# The terminal results of the agent CLI that mean it was stopped, not finished.
STOPPED = {"error_max_turns", "error_max_budget_usd", "error_during_execution"}


@dataclass
class Item:
    """One finding, with the transcript line it is on (1-based; 0 for a file of the sandbox)."""

    kind: str
    line: int
    text: str


@dataclass
class Audit:
    void: list[Item] = field(default_factory=list[Item])
    stuck: list[Item] = field(default_factory=list[Item])

    def render(self) -> str:
        head = (
            "VOID"
            if self.void
            else ("CLEAN" if not self.stuck else f"{len(self.stuck)} stuck point(s)")
        )
        lines = [f"audit: {head}"]
        lines += [f"  void   line {item.line:>5}  {item.kind}: {item.text}" for item in self.void]
        lines += [f"  stuck  line {item.line:>5}  {item.kind}: {item.text}" for item in self.stuck]
        return "\n".join(lines)


def inside(path: str, root: str) -> bool:
    """Whether `path`, already absolute and normalised, lies at or under `root`, on this machine's rules (a
    different drive on Windows is outside)."""
    try:
        return os.path.commonpath(
            [os.path.normcase(path), os.path.normcase(root)]
        ) == os.path.normcase(root)
    except ValueError:
        return False


def resolve(path: str, cwd: str) -> str:
    """`path` made absolute against `cwd` and normalised, without touching the file system. A symbolic link
    is caught by `real`."""
    return os.path.normpath(os.path.join(cwd, path))


def real(path: str) -> str:
    """The path with every symbolic link on it resolved, as far as the path exists: a link the agent made cannot
    point outside, and a spelling through a linked folder (macOS's `/var` for `/private/var`) is still inside."""
    return str(Path(path).resolve())


def outside(path: str, cwd: str, sandbox: str) -> bool:
    if path in HARMLESS or path.upper() == "NUL":
        return False
    return not inside(real(resolve(path, cwd)), real(sandbox))


# A shell word that names a path: absolute (`/x`, `C:\x`, `C:/x`), or one that climbs (`..`).
PATHLIKE = re.compile(r"^(/|[A-Za-z]:[\\/])|(^|[\\/])\.\.([\\/]|$)")


def shell_reads(command: str, cwd: str, sandbox: str) -> tuple[list[str], str]:
    """The words of a shell command that name a place outside the sandbox, and the working directory after it.
    `cd` is followed; any absolute path, `~`, `$HOME` or a `..` that leaves the sandbox counts."""
    found: list[str] = []
    if ESCAPES.search(command):
        found.append(ESCAPES.findall(command)[0][1])
    try:
        words = shlex.split(command, comments=True, posix=True)
    except ValueError:
        words = command.split()
    previous = ""
    for word in words:
        for piece in re.split(r"[=:]", word) if "=" in word else [word]:
            part = piece.strip(";&|()<>")
            if PATHLIKE.search(part) and outside(part, cwd, sandbox):
                found.append(part)
        if previous == "cd" and not word.startswith("-"):
            cwd = resolve(word.strip(";&|()<>"), cwd)
        previous = word
    return found, cwd


def fields(value: object) -> dict[str, object]:
    """A JSON object as a typed mapping; anything else as an empty one."""
    return cast("dict[str, object]", value) if isinstance(value, dict) else {}


def items(value: object) -> list[object]:
    """A JSON array as a typed list; anything else as an empty one."""
    return cast("list[object]", value) if isinstance(value, list) else []


def text_of(content: object) -> str:
    if isinstance(content, str):
        return content
    parts = [
        str(fields(block).get("text", ""))
        for block in items(content)
        if fields(block).get("type") == "text"
    ]
    return "\n".join(parts)


def audit(transcript: Path, sandbox: Path) -> Audit:
    root = str(sandbox.resolve())
    cwd = os.path.join(root, "work")
    phrases = [
        p.strip().lower()
        for p in GUESSES.read_text(encoding="utf-8").splitlines()
        if p.strip() and not p.startswith("#")
    ]
    result = Audit()
    refusals: dict[str, list[int]] = {}
    pending: dict[str, str] = {}
    for number, raw in enumerate(transcript.read_text(encoding="utf-8").splitlines(), 1):
        try:
            event: object = json.loads(raw)
        except json.JSONDecodeError:
            continue
        event = fields(event)
        kind = event.get("type")
        if kind == "result" and event.get("subtype") in STOPPED:
            result.stuck.append(Item("stopped", number, str(event.get("subtype"))))
        for value in items(fields(event.get("message")).get("content")):
            block = fields(value)
            if block.get("type") == "text" and kind == "assistant":
                said = str(block.get("text", "")).lower()
                for phrase in phrases:
                    if phrase in said:
                        result.stuck.append(Item("guess", number, phrase))
            elif block.get("type") == "tool_use":
                name, given = str(block.get("name")), fields(block.get("input"))
                if name in WEB_TOOLS:
                    result.void.append(Item("web", number, name))
                elif name in PATH_KEYS and given.get(PATH_KEYS[name]):
                    path = str(given[PATH_KEYS[name]])
                    if outside(path, cwd, root):
                        result.void.append(Item("outside", number, f"{name} {path}"))
                elif name == "Bash":
                    command = str(given.get("command", ""))
                    found, cwd = shell_reads(command, cwd, root)
                    for word in found:
                        result.void.append(
                            Item("outside", number, f"Bash {word} in {command[:120]!r}")
                        )
                    if "mineworld" in command:
                        pending[str(block.get("id"))] = command
            elif block.get("type") == "tool_result":
                command = pending.pop(str(block.get("tool_use_id")), None)
                output = text_of(block.get("content"))
                failed = bool(block.get("is_error")) or output.startswith("Exit code")
                for line in output.splitlines():
                    if command is not None and failed and "unknown field" in line:
                        result.stuck.append(Item("unknown-field", number, line.strip()[:200]))
                    # A refusal: a `mineworld` command that failed, and the line it explains why on.
                    if command is not None and failed and line.startswith("[mineworld]"):
                        key = re.sub(r"\d+", "N", line.strip())
                        refusals.setdefault(key, []).append(number)
    for refusal, lines in refusals.items():
        if len(lines) >= 3:
            result.stuck.append(
                Item("repeated-refusal", lines[2], f"{len(lines)}x {refusal[:200]}")
            )
    work = sandbox / "work"
    status = (
        (work / "STATUS").read_text(encoding="utf-8").strip() if (work / "STATUS").is_file() else ""
    )
    if not status.startswith("DONE"):
        result.stuck.append(Item("status", 0, status or "no STATUS written"))
    gaps = work / "GAPS.txt"
    if gaps.is_file():
        for line in gaps.read_text(encoding="utf-8").splitlines():
            if line.strip():
                result.stuck.append(Item("gap", 0, line.strip()))
    return result


def main(arguments: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Audit an onboarding transcript (PROTOCOL.md §6).")
    parser.add_argument("transcript", type=Path)
    parser.add_argument("--sandbox", type=Path, required=True)
    parser.add_argument("--json", action="store_true")
    options = parser.parse_args(arguments)
    result = audit(options.transcript, options.sandbox)
    if options.json:
        print(
            json.dumps(
                {
                    "void": [i.__dict__ for i in result.void],
                    "stuck": [i.__dict__ for i in result.stuck],
                }
            )
        )
    else:
        print(result.render())
    return 3 if result.void else (1 if result.stuck else 0)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
