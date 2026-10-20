"""Tile a World Pack into a larger one of independent blocks (pr-s28g-scale-gates.md §5.2).

`tile(source, people, world_id, schemas)` copies the pack's places, items and people `blocks` times, with
`blocks = ceil(people / len(population))`. Every place, person and item of block *b* gets the key `<key>-<bb>`
(`bb` two digits from `01`), and every reference inside a block names that block's copy. Person *p* (0-based, in
the source population's order, repeated) goes to block `p // n + 1`, so the last block may hold only some people.

Which fields are references is never listed here: they are found from the repository's generated JSON Schemas
(`mineworld reference`, `docs/reference/schema/`). A value at a node that is `{"$ref": "#/$defs/EntityKey"}` is a
reference. A top-level key of a place, person or item file that its base schema does not define is a System Pack
section, read with `section-<key>.schema.json`; a section with no schema is refused, so a new section's references
are never silently missed.

In `world.yaml` a list of references becomes, block by block, the copies that exist (so a partial last block's
seats are only its people's); a single reference names block 1's copy.

Refused by name (`TileRefused`, exit 2, nothing written): organizations, a decision list named by the world, a
configure file that holds a reference or has no schema, a section with no schema, a reference to a key the source
does not define (or to a person the block does not have), a key too long to suffix, and an output that exists.
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from collections.abc import Callable, Sequence
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import cast

import yaml

type Json = bool | int | float | str | list[Json] | dict[str, Json] | None
type Schema = dict[str, Json]

KEY_MAX = 64
SUFFIX = 3
"""`-bb`: the length a key grows by."""
REFERENCE = "EntityKey"
"""The `$defs` name the generated schemas give every authoring key (`mineworld_authoring::schema::Key`)."""

# The pack format (docs/MODULE_SPEC.md §4.1): which world.yaml list names which directory, and the base schema of
# its files. This is the format, not a world.
KINDS: dict[str, tuple[str, str]] = {
    "places": ("places", "place"),
    "items": ("items", "item"),
    "population": ("people", "person"),
}
UNSUPPORTED = ("organizations", "decisions")


class TileRefused(Exception):
    """The source cannot be tiled: `what` names the file, key or section, `why` says why."""

    def __init__(self, what: str, why: str) -> None:
        self.what = what
        self.why = why
        super().__init__(f"tile: {what}: {why}")


@dataclass(frozen=True)
class _Source:
    root: Path
    world: dict[str, Json]
    keys: dict[str, tuple[str, ...]]
    """Each kind's keys, in world.yaml's order."""
    kind_of: dict[str, str]
    """Every defined key → its kind."""


class _Schemas:
    def __init__(self, directory: Path) -> None:
        self._directory = directory
        self._cache: dict[str, Schema | None] = {}

    def get(self, name: str) -> Schema | None:
        if name not in self._cache:
            path = self._directory / f"{name}.schema.json"
            self._cache[name] = (
                cast(Schema, json.loads(path.read_text(encoding="utf-8")))
                if path.is_file()
                else None
            )
        return self._cache[name]

    def need(self, name: str, what: str) -> Schema:
        found = self.get(name)
        if found is None:
            raise TileRefused(what, f"no schema {name}.schema.json in {self._directory}")
        return found


# ── The schema walk ─────────────────────────────────────────────────────────────────────────────


class _Rename:
    """What a reference becomes. `one` maps a single reference; `many` maps a list of them."""

    def __init__(
        self, one: Callable[[str, str], str], many: Callable[[list[str], str], list[str]]
    ) -> None:
        self.one = one
        self.many = many


def _is_reference(node: Schema) -> bool:
    ref = node.get("$ref")
    return isinstance(ref, str) and ref.rsplit("/", 1)[-1] == REFERENCE


def _resolved(root: Schema, node: Schema) -> Schema:
    seen = 0
    while isinstance(ref := node.get("$ref"), str) and not _is_reference(node):
        if not ref.startswith("#/$defs/"):
            raise TileRefused(ref, "only local $defs references are understood")
        defs = cast(dict[str, Schema], root.get("$defs", {}))
        node = defs[ref.removeprefix("#/$defs/")]
        seen += 1
        if seen > 64:
            raise TileRefused(ref, "a $ref cycle")
    return node


def _json_type(value: Json) -> str:
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "boolean"
    if isinstance(value, int):
        return "integer"
    if isinstance(value, float):
        return "number"
    if isinstance(value, str):
        return "string"
    if isinstance(value, list):
        return "array"
    return "object"


def _types(node: Schema) -> set[str]:
    declared = node.get("type")
    if isinstance(declared, str):
        return {declared}
    if isinstance(declared, list):
        return {str(one) for one in declared}
    return set()


def _matches(root: Schema, node: Schema, value: Json) -> bool:
    """Whether `value` fits `node` closely enough to pick it among `anyOf` / `oneOf` branches: its JSON type,
    and for an object its required keys and, when closed, no key the branch does not define."""
    if _is_reference(node):
        return isinstance(value, str)
    node = _resolved(root, node)
    branches = node.get("anyOf", node.get("oneOf"))
    if isinstance(branches, list):
        return any(_matches(root, cast(Schema, branch), value) for branch in branches)
    types = _types(node)
    kind = _json_type(value)
    if types and kind not in types and not (kind == "integer" and "number" in types):
        return False
    if isinstance(value, dict):
        required = node.get("required", [])
        if isinstance(required, list) and any(str(key) not in value for key in required):
            return False
        properties = cast(dict[str, Json], node.get("properties", {}))
        if node.get("additionalProperties") is False and "patternProperties" not in node:
            return all(key in properties for key in value)
    return True


def _child(node: Schema, key: str) -> Schema | None:
    properties = cast(dict[str, Schema], node.get("properties", {}))
    if key in properties:
        return properties[key]
    extra = node.get("additionalProperties")
    if isinstance(extra, dict):
        return cast(Schema, extra)
    patterns = cast(dict[str, Schema], node.get("patternProperties", {}))
    if len(patterns) == 1:
        return next(iter(patterns.values()))
    return None


def _walk(root: Schema, node: Schema, value: Json, rename: _Rename, where: str) -> Json:
    """`value` with every reference renamed, walked beside its schema node."""
    if _is_reference(node):
        if not isinstance(value, str):
            raise TileRefused(where, f"a reference that is not a key: {value!r}")
        return rename.one(value, where)
    node = _resolved(root, node)
    branches = node.get("anyOf", node.get("oneOf"))
    if isinstance(branches, list):
        for branch in branches:
            if _matches(root, cast(Schema, branch), value):
                return _walk(root, cast(Schema, branch), value, rename, where)
        return value
    if isinstance(value, list):
        items = node.get("items")
        if not isinstance(items, dict):
            return value
        items = cast(Schema, items)
        if _is_reference(items):
            keys = [one for one in value if isinstance(one, str)]
            if len(keys) != len(value):
                raise TileRefused(where, "a list of references holds something that is not a key")
            return cast(Json, rename.many(keys, where))
        return [_walk(root, items, one, rename, f"{where}[{n}]") for n, one in enumerate(value)]
    if isinstance(value, dict):
        out: dict[str, Json] = {}
        for key, one in value.items():
            child = _child(node, key)
            out[key] = one if child is None else _walk(root, child, one, rename, f"{where}.{key}")
        return out
    return value


def _holds_reference(root: Schema, value: Json, where: str) -> bool:
    found: list[str] = []

    def one(key: str, at: str) -> str:
        found.append(at)
        return key

    def many(keys: list[str], at: str) -> list[str]:
        if keys:
            found.append(at)
        return keys

    _walk(root, root, value, _Rename(one, many), where)
    return bool(found)


# ── Reading the source ──────────────────────────────────────────────────────────────────────────


def _load(path: Path) -> Json:
    try:
        return cast(Json, yaml.safe_load(path.read_text(encoding="utf-8")))
    except OSError as error:
        raise TileRefused(str(path), f"cannot read ({type(error).__name__})") from None


def _read_source(root: Path) -> _Source:
    world = _load(root / "world.yaml")
    if not isinstance(world, dict):
        raise TileRefused("world.yaml", "not a mapping")
    for key in UNSUPPORTED:
        if world.get(key):
            raise TileRefused(key, "the tiler does not copy this yet (pr-s28g §2.2)")
    if (root / "organizations").exists():
        raise TileRefused("organizations", "the tiler does not copy this yet (pr-s28g §2.2)")
    keys: dict[str, tuple[str, ...]] = {}
    kind_of: dict[str, str] = {}
    for kind in KINDS:
        listed = world.get(kind) or []
        if not isinstance(listed, list):
            raise TileRefused(f"world.yaml {kind}", "not a list")
        keys[kind] = tuple(str(one) for one in listed)
        for one in keys[kind]:
            if len(one) > KEY_MAX - SUFFIX:
                raise TileRefused(
                    one, f"longer than {KEY_MAX - SUFFIX} characters, so it cannot be suffixed"
                )
            kind_of[one] = kind
    if not keys["population"]:
        raise TileRefused("world.yaml population", "empty: there is nobody to tile")
    return _Source(root, world, keys, kind_of)


# ── Tiling ──────────────────────────────────────────────────────────────────────────────────────


def _copy(key: str, block: int) -> str:
    return f"{key}-{block:02d}"


def _dump(data: Json, header: str) -> str:
    text = yaml.safe_dump(
        data, sort_keys=False, allow_unicode=True, width=100, default_flow_style=None
    )
    return header + text


def tile(
    source: Path,
    people: int,
    world_id: str,
    schemas: Path,
    *,
    unique: Sequence[str] = (),
    drop: Sequence[str] = (),
) -> dict[PurePosixPath, str]:
    """The tiled pack's files, path → text (LF line ends). Pure: reads the source and the schemas only.

    `unique`: top-level string sections of a person file (such as a name) that get ` <b>` appended in block
    *b* ≥ 2, so they stay unique. `drop`: top-level world.yaml keys left out of the tiled world."""
    if people < 1:
        raise TileRefused("--people", "must be at least 1")
    found = _read_source(source)
    catalog = _Schemas(schemas)
    population = found.keys["population"]
    blocks = math.ceil(people / len(population))
    members: dict[int, list[str]] = {}
    for p in range(people):
        members.setdefault(p // len(population) + 1, []).append(population[p % len(population)])
    header = (
        f"# Generated by tools/worldgen/tile.py from {source.name} (people {people}, blocks {blocks}). "
        "Do not edit; regenerate.\n"
    )

    def exists(key: str, block: int) -> bool:
        return found.kind_of[key] != "population" or key in members[block]

    def defined(key: str, where: str) -> None:
        if key not in found.kind_of:
            raise TileRefused(where, f"names {key!r}, which the source does not define")

    files: dict[PurePosixPath, str] = {}
    for kind, (directory, base_name) in KINDS.items():
        base = catalog.need(base_name, f"{directory}/")
        for key in found.keys[kind]:
            relative = f"{directory}/{key}.yaml"
            data = _load(source / relative)
            if data is None:
                data = {}
            if not isinstance(data, dict):
                raise TileRefused(relative, "not a mapping")
            for block in range(1, blocks + 1):
                if not exists(key, block):
                    continue

                def one(ref: str, where: str, block: int = block) -> str:
                    defined(ref, where)
                    if not exists(ref, block):
                        raise TileRefused(
                            where, f"names {ref!r}, which block {block:02d} does not have"
                        )
                    return _copy(ref, block)

                def many(refs: list[str], where: str, block: int = block) -> list[str]:
                    return [one(ref, where, block) for ref in refs]

                rename = _Rename(one, many)
                out: dict[str, Json] = {}
                base_properties = cast(dict[str, Json], base.get("properties", {}))
                for section, value in data.items():
                    where = f"{relative} {section}"
                    if section in base_properties:
                        root = base
                        node = cast(Schema, base_properties[section])
                    else:
                        root = catalog.need(
                            f"section-{section}", f"{relative}: section {section!r}"
                        )
                        node = root
                    renamed = _walk(root, node, value, rename, where)
                    if kind == "population" and section in unique and block >= 2:
                        if not isinstance(renamed, str):
                            raise TileRefused(
                                where, "--unique names a section that is not a string"
                            )
                        renamed = f"{renamed} {block}"
                    out[section] = renamed
                files[PurePosixPath(directory, f"{_copy(key, block)}.yaml")] = _dump(out, header)

    world_schema = catalog.need("world", "world.yaml")

    def world_one(ref: str, where: str) -> str:
        defined(ref, where)
        return _copy(ref, 1)

    def world_many(refs: list[str], where: str) -> list[str]:
        for ref in refs:
            defined(ref, where)
        return [_copy(ref, b) for b in range(1, blocks + 1) for ref in refs if exists(ref, b)]

    world_properties = cast(dict[str, Json], world_schema.get("properties", {}))
    tiled: dict[str, Json] = {}
    for key, value in found.world.items():
        if key in drop:
            continue
        if key not in world_properties:
            raise TileRefused(
                f"world.yaml {key}",
                "the world schema does not define it (pass --drop to leave it out)",
            )
        tiled[key] = _walk(
            world_schema,
            cast(Schema, world_properties[key]),
            value,
            _Rename(world_one, world_many),
            f"world.yaml {key}",
        )
    identity = tiled.get("world")
    if not isinstance(identity, dict):
        raise TileRefused("world.yaml world", "missing")
    identity["id"] = world_id
    identity["name"] = f"{identity.get('name')}, tiled {blocks} times"
    files[PurePosixPath("world.yaml")] = _dump(tiled, header)

    configure = found.world.get("configure") or []
    if not isinstance(configure, list):
        raise TileRefused("world.yaml configure", "not a list")
    for key in configure:
        relative = f"configure/{key}.yaml"
        schema = catalog.need(f"configure-{key}", relative)
        data = _load(source / relative)
        if _holds_reference(schema, data, relative):
            raise TileRefused(
                relative, "holds a reference to a place, person or item (pr-s28g §2.2)"
            )
        files[PurePosixPath(relative)] = _dump(data, header)
    return dict(sorted(files.items()))


def write(files: dict[PurePosixPath, str], out: Path) -> None:
    """Writes the files under `out`, which must not exist; LF line ends on every OS."""
    if out.exists():
        raise TileRefused(str(out), "the output directory exists")
    for relative, text in sorted(files.items()):
        path = out.joinpath(*relative.parts)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(text.encode("utf-8"))


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Tile a World Pack into independent blocks (pr-s28g §5.2)."
    )
    parser.add_argument(
        "--source", type=Path, required=True, help="the World Pack directory to tile"
    )
    parser.add_argument(
        "--people", type=int, required=True, help="how many people the tiled world holds"
    )
    parser.add_argument("--id", required=True, help="the tiled world's id (its directory's name)")
    parser.add_argument(
        "--out", type=Path, required=True, help="the directory to write (must not exist)"
    )
    parser.add_argument(
        "--schemas", type=Path, default=Path("docs/reference/schema"), help="the generated schemas"
    )
    parser.add_argument(
        "--unique",
        action="append",
        default=[],
        help="a person-file string section suffixed per block (repeatable)",
    )
    parser.add_argument(
        "--drop", action="append", default=[], help="a world.yaml key left out (repeatable)"
    )
    arguments = parser.parse_args(argv)
    try:
        files = tile(
            arguments.source,
            arguments.people,
            arguments.id,
            arguments.schemas,
            unique=arguments.unique,
            drop=arguments.drop,
        )
        write(files, arguments.out)
    except TileRefused as refused:
        print(refused, file=sys.stderr)
        return 2
    print(f"tile: wrote {len(files)} files to {arguments.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
