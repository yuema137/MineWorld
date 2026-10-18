#!/usr/bin/env python3
"""The cast binding of a 3D Presentation Pack is structurally valid (`docs/DECISIONS.md` `ARC-92`).

`characters/cast.yaml` says which character draws each person a client is told about (`ARC-91`,
`ARC-92`). Godot reads it with its built-in JSON parser, so it is written in YAML's JSON-compatible
subset (`DEP-16`). A file that only looks valid draws everybody as nobody, and nothing else reads it
until S26-c2, so this checker holds its structure now: the keys, their types and the shape of every
id. Whether every id names a character directory is S26-c1's rule, added there.

Usage:

    python3 scripts/check_cast.py [FILE ...]     default: presentation/mineworld-default/3D/characters/cast.yaml
    python3 scripts/check_cast.py --self-test

Each failure prints `<file>: <rule>: <detail>`. Exit 1 if any file fails, else each file prints one line
and the exit is 0. The rules, `CC-1` to `CC-6`, are specified in `pr-s26a-appearance.md` §5.7.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT = ROOT / "presentation" / "mineworld-default" / "3D" / "characters" / "cast.yaml"
SHAPE = re.compile(r"[a-z0-9-]{1,48}")
TOP_KEYS = {"format", "looks", "extras", "default"}
VALID = (
    '{"format": 1, "looks": {}, "extras": [], "default": "vera-lindgren"}'
)


class DuplicateKey(Exception):
    def __init__(self, key: str) -> None:
        super().__init__(key)
        self.key = key


def _objects_without_duplicates(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise DuplicateKey(key)
        result[key] = value
    return result


def _is_id(value: object) -> bool:
    return isinstance(value, str) and SHAPE.fullmatch(value) is not None


def check_text(text: str) -> tuple[list[tuple[str, str]], dict[str, object] | None]:
    """The failures in `text` as `(rule, detail)`, and the parsed object when it parsed."""
    try:
        data = json.loads(text, object_pairs_hook=_objects_without_duplicates)
    except DuplicateKey as error:
        return [("CC-1", f"the key {error.key!r} appears twice")], None
    except json.JSONDecodeError as error:
        return [("CC-1", f"not JSON: {error.msg} at line {error.lineno} column {error.colno}")], None
    if not isinstance(data, dict):
        return [("CC-2", "the top level is not an object")], None

    failures: list[tuple[str, str]] = []
    for key in sorted(set(data) - TOP_KEYS):
        failures.append(("CC-2", f"unknown top-level key {key!r}"))
    for key in sorted(TOP_KEYS - set(data)):
        failures.append(("CC-2", f"missing top-level key {key!r}"))

    if "format" in data:
        fmt = data["format"]
        if not (isinstance(fmt, int) and not isinstance(fmt, bool) and fmt == 1):
            failures.append(("CC-3", f"format is {fmt!r}, not the integer 1"))

    if "looks" in data:
        looks = data["looks"]
        if not isinstance(looks, dict):
            failures.append(("CC-4", "looks is not an object"))
        else:
            for key, value in looks.items():
                if not SHAPE.fullmatch(key):
                    failures.append(("CC-4", f"look key {key!r} is not 1 to 48 of a-z, 0-9 and -"))
                if not _is_id(value):
                    failures.append(("CC-4", f"look {key!r} names {value!r}, not a character id"))

    if "extras" in data:
        extras = data["extras"]
        if not isinstance(extras, list):
            failures.append(("CC-5", "extras is not an array"))
        else:
            listed: set[str] = set()
            for entry in extras:
                if not _is_id(entry):
                    failures.append(("CC-5", f"extra {entry!r} is not a character id"))
                elif entry in listed:
                    failures.append(("CC-5", f"extra {entry!r} is listed twice"))
                else:
                    listed.add(entry)

    if "default" in data and not _is_id(data["default"]):
        failures.append(("CC-6", f"default is {data['default']!r}, not a character id"))
    return failures, data


def self_test() -> int:
    cases: list[tuple[str, str | None]] = [
        (VALID, None),
        ("not json {", "CC-1"),
        ('{"format": 1, "format": 1, "looks": {}, "extras": [], "default": "a"}', "CC-1"),
        ('{"format": 1, "looks": {}, "extras": [], "default": "a", "colour": "red"}', "CC-2"),
        ('{"format": 1, "looks": {}, "extras": []}', "CC-2"),
        ('{"format": 2, "looks": {}, "extras": [], "default": "a"}', "CC-3"),
        ('{"format": true, "looks": {}, "extras": [], "default": "a"}', "CC-3"),
        ('{"format": 1, "looks": {"Alice": "a"}, "extras": [], "default": "a"}', "CC-4"),
        ('{"format": 1, "looks": {"alice": "alice moreau"}, "extras": [], "default": "a"}', "CC-4"),
        ('{"format": 1, "looks": {}, "extras": ["a", "a"], "default": "a"}', "CC-5"),
        ('{"format": 1, "looks": {}, "extras": "a", "default": "a"}', "CC-5"),
        ('{"format": 1, "looks": {}, "extras": [], "default": null}', "CC-6"),
    ]
    for number, (text, expected) in enumerate(cases, start=1):
        failures, _ = check_text(text)
        rules = {rule for rule, _ in failures}
        if expected is None and failures:
            print(f"self-test: case {number} should pass: {failures}")
            return 1
        if expected is not None and expected not in rules:
            print(f"self-test: case {number} should fail {expected}, got {sorted(rules) or 'nothing'}")
            return 1
    print(f"self-test: {len(cases)} cases, all as expected")
    return 0


def main(argv: list[str]) -> int:
    if argv == ["--self-test"]:
        return self_test()
    files = [Path(arg) for arg in argv] or [DEFAULT]
    status = 0
    for path in files:
        try:
            text = path.read_text(encoding="utf-8")
        except OSError as error:
            print(f"{path}: read: {error}")
            status = 1
            continue
        failures, data = check_text(text)
        if failures or data is None:
            status = 1
            for rule, detail in failures:
                print(f"{path}: {rule}: {detail}")
            continue
        looks, extras = data["looks"], data["extras"]
        print(
            f"{path}: format 1, {len(looks)} looks, {len(extras)} extras, "
            f"default {data['default']}: valid"
        )
    return status


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
