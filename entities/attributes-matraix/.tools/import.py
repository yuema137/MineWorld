#!/usr/bin/env python3
"""The MatrAIx schema, imported unchanged into this optional backend pack (S30-h1 design §5.2; DEP-66, DEP-69).

    python3 -I .tools/import.py --upstream <clone>   copy the pinned files from a clone of the upstream at the
                                                     pinned commit into .upstream/, verify them, regenerate
    python3 -I .tools/import.py                      regenerate the pack from .upstream/ and .tools/overlay.yaml
    python3 -I .tools/import.py --check              regenerate into a temporary directory and compare byte for
                                                     byte with the pack; exit 1 on any difference

Standard library only; offline; it never runs upstream code. `.upstream/` holds upstream's files byte for byte;
everything the loader reads is a pure function of them and the overlay (I-h-3). The overlay is written in YAML's
JSON-compatible form so the standard library reads it (S30-h1 §16 D-h1-2).

Steps (§5.2): 1 verify every SHA-256 against the overlay's `upstream` block (`ProvenanceMismatch`); 2 drop every
dimension whose `source_origin.license` is present and not MIT, and refuse a count other than the overlay's
(`ExcludedCount`); 3 give each value a slug (lower case; the overlay's transliteration table; any other run of
characters outside [a-z0-9] becomes `-`), refusing a collision (`ValueSlugCollision`), an untransliterated
non-ASCII character (`UntransliteratedValue`) or a slug over 64 bytes (`ValueSlugTooLong`); 4 write one
declaration per kept dimension into `attributes/<category-slug>.yaml` — its group from the overlay's category
table, its category as the facet `category.<slug>`, its label, phrase and every value's verbatim words, and no
`default:` (D-h-3); 5 write `backends/<backend>.yaml`. Output is sorted, LF, UTF-8 and float-free.
"""

from __future__ import annotations

import hashlib
import json
import re
import shutil
import sys
import tempfile
from pathlib import Path

PACK = Path(__file__).resolve().parent.parent
OVERLAY = PACK / ".tools" / "overlay.yaml"
UPSTREAM = ".upstream"
SCHEMA = f"{UPSTREAM}/dimensions.json"
GENERATED = ("attributes", "backends")
"""The pack directories this tool writes, every file in them generated."""
SLUG_MAX = 64
OTHER = re.compile(r"[^a-z0-9]+")


class ImportRefused(Exception):
    """A refusal by name: the first word is the refusal's name."""


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_overlay(path: Path = OVERLAY) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def slug(text: str, table: dict[str, str], where: str) -> str:
    """`Early bird` → `early-bird`: lower case, the table's replacements, every other run of characters
    outside [a-z0-9] → `-`, trimmed."""
    lowered = "".join(table.get(character, character) for character in text.lower())
    for character in lowered:
        if not character.isascii():
            raise ImportRefused(
                f"UntransliteratedValue: {where}: {text!r} holds {character!r}"
            )
    result = OTHER.sub("-", lowered).strip("-")
    if not result:
        raise ImportRefused(f"ValueSlugEmpty: {where}: {text!r}")
    if len(result.encode()) > SLUG_MAX:
        raise ImportRefused(f"ValueSlugTooLong: {where}: {text!r} → {result!r}")
    return result


def verify(pack: Path, overlay: dict) -> None:
    """Step 1: every file the overlay pins is in `.upstream/` with its digest."""
    for relative, expected in sorted(overlay["upstream"]["files"].items()):
        path = pack / relative
        found = sha256(path) if path.is_file() else "missing"
        if found != expected:
            raise ImportRefused(
                f"ProvenanceMismatch: {relative}: expected {expected}, found {found}"
            )


def kept_dimensions(pack: Path, overlay: dict) -> tuple[list[dict], dict[str, int]]:
    """Step 2: the dimensions upstream releases under its own MIT terms, and the count dropped per licence."""
    dimensions = json.loads((pack / SCHEMA).read_text(encoding="utf-8"))["dimensions"]
    kept, dropped = [], {}
    for dimension in dimensions:
        licence = dimension.get("source_origin", {}).get("license")
        if licence is not None and licence != "MIT":
            dropped[licence] = dropped.get(licence, 0) + 1
        else:
            kept.append(dimension)
    expect = overlay["expect"]
    if len(kept) != expect["dimensions"] or dropped != expect["excluded"]:
        raise ImportRefused(
            f"ExcludedCount: kept {len(kept)}, dropped {dropped}; the overlay expects "
            f"{expect['dimensions']} and {expect['excluded']}"
        )
    return kept, dropped


def group_of(category: str, table: dict[str, str]) -> str:
    """The overlay's group for `category`: its exact row, else the row of the part before ': '."""
    for key in (category, category.split(": ", 1)[0]):
        if key in table:
            return table[key]
    raise ImportRefused(
        f"UnplacedCategory: {category!r} has no row in the overlay's category table"
    )


def quoted(text: str) -> str:
    """A YAML double-quoted scalar (JSON's escaping is YAML's)."""
    return json.dumps(text, ensure_ascii=False)


def declaration(dimension: dict, overlay: dict) -> tuple[str, list[str]]:
    """Step 4: one dimension as `(category slug, its lines)`."""
    table = overlay["transliterate"]
    identifier = dimension["id"]
    category = slug(dimension["category"], table, f"{identifier} category")
    values: list[tuple[str, str]] = []
    seen: dict[str, str] = {}
    for value in dimension["values"]:
        value_slug = slug(value, table, identifier)
        if value_slug in seen:
            raise ImportRefused(
                f"ValueSlugCollision: {identifier}: {seen[value_slug]!r} and {value!r} are both {value_slug!r}"
            )
        seen[value_slug] = value
        values.append((value_slug, value))
    defaults = overlay["defaults"]
    lines = [
        f"  {identifier}:",
        f"    group: {group_of(dimension['category'], overlay['categories'])}",
        f"    facets: [category.{category}]",
        f"    kind: {{ category: [{', '.join(value_slug for value_slug, _ in values)}] }}",
        f"    visibility: {defaults['visibility']}",
        f"    mutability: {defaults['mutability']}",
        f"    label: {quoted(dimension['label'])}",
        f"    phrase: {quoted(dimension['phrase'])}",
        "    words:",
        *(f"      {value_slug}: {quoted(value)}" for value_slug, value in values),
        f"    since: {overlay['pack']['version']}",
    ]
    return category, lines


def backend(overlay: dict, catalogues: list[str]) -> str:
    """Step 5: the backend contract (§5.9)."""
    upstream = overlay["upstream"]
    files = "".join(
        f"    {relative}: {digest}\n"
        for relative, digest in sorted(upstream["files"].items())
    )
    return (
        "# Generated by .tools/import.py from .upstream/ and .tools/overlay.yaml; do not edit.\n"
        "format: 1\n"
        f"backend: {overlay['backend']}\n"
        f"version: {overlay['pack']['version']}\n"
        f"license: {overlay['pack']['license']}\n"
        f"attribution: {quoted(overlay['pack']['attribution'])}\n"
        f"catalogues: [{', '.join(catalogues)}]\n"
        f"masks: {{ default: {overlay['masks']['default']} }}\n"
        "sampling: [profile]\n"
        "provenance:\n"
        f"  source: {upstream['source']}\n"
        f"  version: {quoted(upstream['version'])}\n"
        f"  commit: {upstream['commit']}\n"
        "  files:\n"
        f"{files}"
    )


def generate(pack: Path, out: Path) -> dict[str, int]:
    """Steps 1–5 from `pack`'s `.upstream/` and overlay into `out`; the count dropped per licence."""
    overlay = read_overlay(pack / ".tools" / "overlay.yaml")
    verify(pack, overlay)
    kept, dropped = kept_dimensions(pack, overlay)
    files: dict[str, list[str]] = {}
    for dimension in sorted(kept, key=lambda dimension: dimension["id"]):
        category, lines = declaration(dimension, overlay)
        files.setdefault(category, []).extend(lines)
    for directory in GENERATED:
        (out / directory).mkdir(parents=True, exist_ok=True)
    for category, lines in sorted(files.items()):
        header = (
            "# Generated by .tools/import.py from .upstream/dimensions.json and .tools/overlay.yaml; do not edit.\n"
            "format: 1\nattributes:\n"
        )
        (out / "attributes" / f"{category}.yaml").write_bytes(
            f"{header}{chr(10).join(lines)}\n".encode()
        )
    (out / "backends" / f"{overlay['backend']}.yaml").write_bytes(
        backend(overlay, sorted(files)).encode()
    )
    return dropped


def generated_files(root: Path) -> dict[str, bytes]:
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for directory in GENERATED
        for path in sorted((root / directory).glob("*"))
        if path.is_file()
    }


def check(pack: Path) -> list[str]:
    """`--check`: each generated file that differs, is missing or is extra."""
    with tempfile.TemporaryDirectory(prefix="mineworld-import-check-") as scratch:
        generate(pack, Path(scratch))
        expected = generated_files(Path(scratch))
    found = generated_files(pack)
    return [
        f"{relative}: {'missing' if relative not in found else 'extra' if relative not in expected else 'differs'}"
        for relative in sorted(set(expected) | set(found))
        if expected.get(relative) != found.get(relative)
    ]


def copy_upstream(clone: Path, pack: Path, overlay: dict) -> None:
    """`--upstream`: the pinned files, from a clone at the pinned commit, into `.upstream/`."""
    for relative, source in sorted(overlay["upstream"]["from"].items()):
        target = pack / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(clone / source, target)


def main(arguments: list[str]) -> int:
    try:
        if arguments == ["--check"]:
            differences = check(PACK)
            for difference in differences:
                print(f"[import] FAIL {difference}")
            if differences:
                return 1
            print(
                "[import] PASS: the pack is the import of .upstream/ and the overlay, byte for byte"
            )
            return 0
        if len(arguments) == 2 and arguments[0] == "--upstream":
            copy_upstream(Path(arguments[1]), PACK, read_overlay())
        elif arguments:
            print(__doc__)
            return 2
        for directory in GENERATED:
            shutil.rmtree(PACK / directory, ignore_errors=True)
        dropped = generate(PACK, PACK)
        print(f"[import] dropped by licence: {json.dumps(dropped, sort_keys=True)}")
        return 0
    except ImportRefused as refusal:
        print(f"[import] FAIL {refusal}")
        return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
