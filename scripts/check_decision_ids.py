#!/usr/bin/env python3
"""Every decision record has one identifier nobody else uses, and the index lists them all.

The decision log is `docs/decisions/`, one record per file (`docs/DECISIONS.md` states the rules;
operator ruling of 2026-10-18). This check exists because the old single-file log silently accepted a
duplicate: two agents branching from one `main` each allocated the next free `ARC-` number, their
entries landed in different regions of the file, and `git merge` reported no conflict — so `main`
gained two decisions with one identifier and every reference to either became ambiguous. With one
file per record two branches that pick the same number now collide as an add/add conflict, but a
record can still be written into the wrong file or the old log, so the rules are checked here:

1. A record file is `docs/decisions/<KIND>-<n>.md`, `KIND` being `ARC` or `DEP`, and its first line
   is `## <KIND>-<n> — <title>` naming that same identifier.
2. Every other decision heading in it is a later note on the same record, `## <KIND>-<n> note — …`.
   A heading for another identifier is a record in the wrong file.
3. `docs/DECISIONS.md` holds no record any more; an entry added there by a branch made before the
   split is reported, so it is moved instead of lost.
4. `docs/decisions/INDEX.md` is exactly what `--write-index` generates from the records.

A duplicate is not a formatting complaint. `CLAUDE.md` §2.1 makes the decision log the authority a
specification cites by identifier; an identifier that names two decisions makes the citation
unresolvable, which is the defect §2.1(4) describes.

Exits non-zero and names every problem. `--write-index` rewrites the index, then checks.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DIRECTORY = ROOT / "docs" / "decisions"
INDEX = DIRECTORY / "INDEX.md"
STUB = ROOT / "docs" / "DECISIONS.md"

# No leading zero, so `ARC-07.md` cannot stand beside `ARC-7.md` as a second spelling of one id.
NAME = re.compile(r"^((ARC|DEP)-([1-9]\d*))\.md$")
HEADING = re.compile(r"^## ((?:ARC|DEP)-\d+)\b(.*)$")
KINDS = {
    "ARC": "Architectural routes",
    "DEP": "Dependencies, and building our own instead",
}


def headings(path: Path) -> list[tuple[int, str, str]]:
    """Every decision heading outside a code fence: line number, identifier, the rest of the line."""
    found = []
    fenced = False
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if line.startswith("```"):
            fenced = not fenced
        elif not fenced and (match := HEADING.match(line)):
            found.append((number, match.group(1), match.group(2)))
    return found


def relative(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def collect(problems: list[str]) -> list[tuple[str, int, str]]:
    """(kind, number, title) for each well-formed record; every defect goes to `problems`."""
    entries = []
    for path in sorted(DIRECTORY.glob("*.md")):
        if path == INDEX:
            continue
        name = NAME.match(path.name)
        if not name:
            problems.append(f"{relative(path)}: not named <ARC|DEP>-<n>.md")
            continue
        key, kind, number = name.group(1), name.group(2), int(name.group(3))
        found = headings(path)
        first = path.read_text(encoding="utf-8").split("\n", 1)[0]
        if not found or found[0][0] != 1 or found[0][1] != key or not found[0][2].startswith(" — "):
            problems.append(f"{relative(path)}:1: the first line must be `## {key} — <title>`, not {first!r}")
            continue
        for line_number, other, rest in found[1:]:
            if other != key:
                problems.append(
                    f"{relative(path)}:{line_number}: {other} is a record of its own; "
                    f"it belongs in docs/decisions/{other}.md"
                )
            elif not rest.startswith(" note"):
                problems.append(f"{relative(path)}:{line_number}: {key} is recorded twice in one file")
        entries.append((kind, number, found[0][2][len(" — "):].strip()))
    return entries


def render(entries: list[tuple[str, int, str]]) -> str:
    """The index. One row per record and nothing that changes when another row does — no count, no
    note tally — so two branches that each add a record touch disjoint lines, and the `union` merge
    of `.gitattributes` can keep both."""
    lines = [
        "# Decision index",
        "",
        "**Generated** by `python3 scripts/check_decision_ids.py --write-index` from the records in this",
        "directory. Do not edit it by hand: CI's fast layer fails when it is stale. The rules for the log",
        "are [`../DECISIONS.md`](../DECISIONS.md).",
    ]
    for kind, heading in KINDS.items():
        rows = sorted((e for e in entries if e[0] == kind), key=lambda e: e[1])
        lines += ["", f"## {kind} — {heading}", "", "| Id | Title |", "| --- | --- |"]
        for _, number, title in rows:
            key = f"{kind}-{number}"
            cell = title.replace("|", r"\|")
            lines.append(f"| [{key}]({key}.md) | {cell} |")
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description="Check the decision records and their index.")
    parser.add_argument("--write-index", action="store_true", help="regenerate INDEX.md first")
    args = parser.parse_args()

    if not DIRECTORY.is_dir():
        print(f"{relative(DIRECTORY)}: not found", file=sys.stderr)
        return 2

    problems: list[str] = []
    entries = collect(problems)
    if not entries:
        print(f"{relative(DIRECTORY)}: no decision records found — has the layout changed?", file=sys.stderr)
        return 2

    if STUB.is_file():
        for line_number, key, _ in headings(STUB):
            problems.append(
                f"{relative(STUB)}:{line_number}: {key} is written in the old log; move it to "
                f"docs/decisions/{key}.md (scripts/split_decisions.py migrate)"
            )

    expected = render(entries)
    if args.write_index:
        with open(INDEX, "w", encoding="utf-8", newline="\n") as handle:
            handle.write(expected)
    elif not INDEX.is_file() or INDEX.read_text(encoding="utf-8") != expected:
        problems.append(
            f"{relative(INDEX)}: stale; run `python3 scripts/check_decision_ids.py --write-index` "
            "and commit the result"
        )

    if problems:
        for problem in problems:
            print(problem, file=sys.stderr)
        return 1
    print(f"{len(entries)} decision ids, all distinct; index current")
    return 0


if __name__ == "__main__":
    sys.exit(main())
