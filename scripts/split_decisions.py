#!/usr/bin/env python3
"""Split a monolithic decision log into one file per decision record (`docs/decisions/`).

`docs/DECISIONS.md` used to be one long file that almost every pull request appended to, so almost
every merge conflicted on it. The operator ruled on 2026-10-18 to split it: one record per file,
`docs/decisions/<ID>.md`, with a generated `docs/decisions/INDEX.md`. The rules live in
`docs/DECISIONS.md`; this script is the mechanical migration, and it stays until every branch that
predates the split has been migrated (PR design `.structured-coding/plans/mvp0/pr-dsplit-decisions.md`).

A section of the log starts at a level-2 heading outside a code fence. It is either a record,
`## ARC-48 — Title`, or a later dated note on one, `## ARC-48 note — …`. Any other level-2 heading
is refused. The `---` rule and blank lines that separate a section from the next
are layout of the log, not part of the section. A record's file holds the record's section, then each
of its notes in the order they appear in the log, joined by the same `---` rule the log used. Every
section's bytes are copied unchanged; nothing is reworded.

Two modes:

  split   --source LOG --out DIR      every record of LOG, written to DIR (the initial migration)
  migrate --base LOG --theirs LOG --out DIR
          a branch made before the split: every record whose text differs between the branch's
          merge base (`--base`) and the branch (`--theirs`) is written to DIR. A record that DIR
          already holds with text other than `--base`'s was also changed on `main` after the branch
          forked; it is reported and left alone, for a person to merge by hand.

Both modes refuse a log they cannot parse completely, and `split` proves the parse by rebuilding the
log byte-for-byte from its pieces before writing anything.
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path

ID_HEADING = re.compile(r"^## ((?:ARC|DEP)-\d+)\b(.*)$")
# What a section ends with that belongs to the log rather than to the section.
TRAILER = re.compile(r"(?:\n+---)?\n*\Z")
JOIN = "\n---\n\n"


@dataclass
class Section:
    key: str  # ARC-48
    is_note: bool
    core: str  # the section's text, from its heading, ending in exactly one newline
    trailer: str  # what followed it in the log before the next heading


def parse(text: str, label: str) -> tuple[str, list[Section]]:
    """The preamble before the first record, and every section, in order."""
    lines = text.splitlines(keepends=True)
    starts: list[tuple[int, str, bool]] = []
    fenced = False
    for index, line in enumerate(lines):
        if line.startswith("```"):
            fenced = not fenced
            continue
        if fenced or not line.startswith("## "):
            continue
        match = ID_HEADING.match(line.rstrip("\n"))
        if not match:
            raise SystemExit(f"{label}:{index + 1}: a level-2 heading that names no decision: {line!r}")
        rest = match.group(2)
        is_note = rest.startswith(" note")
        if not is_note and not rest.startswith(" — "):
            raise SystemExit(f"{label}:{index + 1}: neither a record nor a note: {line!r}")
        starts.append((index, match.group(1), is_note))
    if fenced:
        raise SystemExit(f"{label}: a code fence is never closed")
    if not starts:
        raise SystemExit(f"{label}: no decision headings found")

    preamble = "".join(lines[: starts[0][0]])
    sections = []
    for position, (index, key, is_note) in enumerate(starts):
        end = starts[position + 1][0] if position + 1 < len(starts) else len(lines)
        chunk = "".join(lines[index:end])
        trailer_at = TRAILER.search(chunk).start()
        core = chunk[:trailer_at] + "\n"
        sections.append(Section(key, is_note, core, chunk[trailer_at + 1 :]))
    rebuilt = preamble + "".join(s.core + s.trailer for s in sections)
    if rebuilt != text:
        raise SystemExit(f"{label}: the parse does not rebuild the log byte-for-byte")
    return preamble, sections


def records(sections: list[Section], label: str) -> dict[str, str]:
    """Each record's file content, keyed by id: the record, then its notes in log order."""
    parts: dict[str, list[str]] = {}
    for section in sections:
        if section.is_note:
            if section.key not in parts:
                raise SystemExit(f"{label}: a note on {section.key} comes before {section.key} itself")
            parts[section.key].append(section.core)
        else:
            if section.key in parts:
                raise SystemExit(f"{label}: {section.key} is recorded twice")
            parts[section.key] = [section.core]
    return {key: JOIN.join(chunks) for key, chunks in parts.items()}


def write(out: Path, key: str, content: str) -> None:
    with open(out / f"{key}.md", "w", encoding="utf-8", newline="\n") as handle:
        handle.write(content)


def split(source: Path, out: Path) -> int:
    text = source.read_text(encoding="utf-8")
    _, sections = parse(text, str(source))
    files = records(sections, str(source))
    out.mkdir(parents=True, exist_ok=True)
    for key, content in files.items():
        write(out, key, content)
    notes = sum(1 for s in sections if s.is_note)
    print(f"{len(sections)} sections ({len(files)} records, {notes} notes) -> {len(files)} files in {out}")
    return 0


def migrate(base: Path, theirs: Path, out: Path) -> int:
    before = records(parse(base.read_text(encoding="utf-8"), str(base))[1], str(base))
    after = records(parse(theirs.read_text(encoding="utf-8"), str(theirs))[1], str(theirs))
    written, clashes = [], []
    for key in sorted(set(before) | set(after)):
        if before.get(key) == after.get(key):
            continue
        if key not in after:
            clashes.append(f"{key}: removed on the branch; removal is never automatic")
            continue
        target = out / f"{key}.md"
        current = target.read_text(encoding="utf-8") if target.is_file() else None
        if current is not None and current != before.get(key) and current != after[key]:
            clashes.append(f"{key}: changed on main since the branch forked; merge {target} by hand")
            continue
        write(out, key, after[key])
        written.append(key)
    for key in written:
        print(f"wrote {out / (key + '.md')}")
    for clash in clashes:
        print(f"NOT WRITTEN  {clash}", file=sys.stderr)
    print(f"{len(written)} records written, {len(clashes)} left for a person")
    return 1 if clashes else 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = parser.add_subparsers(dest="mode", required=True)
    one = sub.add_parser("split")
    one.add_argument("--source", type=Path, required=True)
    one.add_argument("--out", type=Path, required=True)
    two = sub.add_parser("migrate")
    two.add_argument("--base", type=Path, required=True)
    two.add_argument("--theirs", type=Path, required=True)
    two.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    if args.mode == "split":
        return split(args.source, args.out)
    return migrate(args.base, args.theirs, args.out)


if __name__ == "__main__":
    sys.exit(main())
