"""The world tiler (pr-s28g-scale-gates.md §5.2, AC-g-1 … AC-g-3).

The unit cases tile a small pack against small schemas written by the test, so each rule of the walk is seen
alone: a reference in a section, in a list, under `oneOf` / `anyOf`; renaming; the world lists expanded block by
block; a partial last block; the `--unique` sections; every refusal. The real-binary cases tile two shipped
worlds (social-cafe, bodies-yard) with the repository's own generated schemas and run them with `mineworld`.
"""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
from pathlib import Path, PurePosixPath
from typing import cast

import pytest
import yaml

from conftest import SCHEMAS, WORLDS
from tile import TileRefused, main, tile, write

KEY = {"$ref": "#/$defs/EntityKey"}
DEFS = {"EntityKey": {"type": "string", "pattern": "^[a-z0-9]([a-z0-9_-]*[a-z0-9])?$"}}


def _schema(path: Path, name: str, body: dict[str, object]) -> None:
    path.mkdir(parents=True, exist_ok=True)
    (path / f"{name}.schema.json").write_text(json.dumps({"$defs": DEFS, **body}), encoding="utf-8")


def _yaml(path: Path, data: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(yaml.safe_dump(data, sort_keys=False), encoding="utf-8")


def _keys() -> dict[str, object]:
    return {"type": "array", "items": KEY}


@pytest.fixture
def schemas(tmp_path: Path) -> Path:
    root = tmp_path / "schemas"
    _schema(
        root,
        "world",
        {
            "type": "object",
            "properties": {
                "world": {"type": "object"},
                "systems": {"type": "array", "items": {"type": "string"}},
                "configure": {"type": "array", "items": {"type": "string"}},
                "places": _keys(),
                "items": _keys(),
                "population": _keys(),
                "seats": _keys(),
                "home": {"anyOf": [KEY, {"type": "null"}]},
            },
        },
    )
    passage = {"type": "object", "properties": {"to": KEY}, "required": ["to"]}
    _schema(
        root,
        "place",
        {
            "type": "object",
            "properties": {
                "tags": {"type": "array"},
                "passages": {"type": "array", "items": passage},
            },
        },
    )
    located = {"type": "object", "properties": {"place": KEY}, "required": ["place"]}
    _schema(
        root,
        "person",
        {"type": "object", "properties": {"location": {"anyOf": [located, {"type": "null"}]}}},
    )
    _schema(root, "item", {"type": "object", "properties": {"tags": {"type": "array"}}})
    _schema(root, "section-name", {"type": "string"})
    _schema(
        root,
        "section-visit",
        {
            "oneOf": [
                {
                    "type": "object",
                    "additionalProperties": False,
                    "required": ["box"],
                    "properties": {"box": {"type": "integer"}},
                },
                {
                    "type": "object",
                    "additionalProperties": False,
                    "required": ["place"],
                    "properties": {"place": KEY},
                },
            ]
        },
    )
    _schema(root, "section-pals", _keys())
    _schema(
        root, "configure-quiet", {"type": "object", "properties": {"level": {"type": "integer"}}}
    )
    _schema(root, "configure-meet", {"type": "object", "properties": {"spot": KEY}})
    return root


@pytest.fixture
def pack(tmp_path: Path) -> Path:
    root = tmp_path / "mini"
    _yaml(
        root / "world.yaml",
        {
            "world": {"id": "mini", "name": "Mini"},
            "systems": ["presence"],
            "configure": ["quiet"],
            "places": ["hall", "yard"],
            "items": ["ball"],
            "population": ["ana", "ben"],
            "seats": ["ana", "ben"],
            "home": "hall",
        },
    )
    _yaml(root / "configure" / "quiet.yaml", {"level": 2})
    _yaml(root / "places" / "hall.yaml", {"tags": ["public"], "passages": [{"to": "yard"}]})
    _yaml(root / "places" / "yard.yaml", {"passages": [{"to": "hall"}]})
    _yaml(root / "items" / "ball.yaml", {"tags": ["toy"]})
    _yaml(
        root / "people" / "ana.yaml",
        {"name": "Ana Lee", "location": {"place": "hall"}, "visit": {"place": "yard"}},
    )
    _yaml(
        root / "people" / "ben.yaml",
        {"name": "Ben Ode", "location": {"place": "yard"}, "visit": {"box": 3}, "pals": ["ana"]},
    )
    return root


def _read(files: dict[PurePosixPath, str], path: str) -> dict[str, object]:
    return yaml.safe_load(files[PurePosixPath(path)])


def test_every_reference_is_renamed_to_its_own_block(pack: Path, schemas: Path) -> None:
    files = tile(pack, 4, "mini-4", schemas, unique=["name"])
    assert sorted(map(str, files)) == sorted(
        [
            "configure/quiet.yaml",
            "world.yaml",
            *(f"places/{k}-{b:02d}.yaml" for k in ("hall", "yard") for b in (1, 2)),
            *(f"items/ball-{b:02d}.yaml" for b in (1, 2)),
            *(f"people/{k}-{b:02d}.yaml" for k in ("ana", "ben") for b in (1, 2)),
        ]
    )
    assert _read(files, "places/hall-02.yaml")["passages"] == [{"to": "yard-02"}]
    ana = _read(files, "people/ana-02.yaml")
    assert ana["location"] == {"place": "hall-02"}  # under anyOf
    assert ana["visit"] == {"place": "yard-02"}  # the oneOf branch that fits
    assert ana["name"] == "Ana Lee 2"
    ben = _read(files, "people/ben-01.yaml")
    assert ben["visit"] == {"box": 3}  # the other branch: an integer, not a key
    assert ben["pals"] == ["ana-01"]  # a list of references in a section
    assert ben["name"] == "Ben Ode"  # block 1 keeps the source names


def test_the_world_lists_expand_block_by_block_and_a_partial_block_has_only_its_people(
    pack: Path, schemas: Path
) -> None:
    files = tile(pack, 3, "mini-3", schemas)
    world = _read(files, "world.yaml")
    assert world["world"] == {"id": "mini-3", "name": "Mini, tiled 2 times"}
    assert world["places"] == ["hall-01", "yard-01", "hall-02", "yard-02"]
    assert world["population"] == ["ana-01", "ben-01", "ana-02"]
    assert world["seats"] == ["ana-01", "ben-01", "ana-02"]
    assert world["home"] == "hall-01"  # a single reference names block 1's copy
    assert PurePosixPath("people/ben-02.yaml") not in files
    assert _read(files, "configure/quiet.yaml") == {"level": 2}


def test_the_same_arguments_give_the_same_bytes_with_lf_and_the_header(
    pack: Path, schemas: Path
) -> None:
    first = tile(pack, 4, "mini-4", schemas)
    assert first == tile(pack, 4, "mini-4", schemas)
    for text in first.values():
        assert text.startswith(
            "# Generated by tools/worldgen/tile.py from mini (people 4, blocks 2)."
        )
        assert text.endswith("\n") and "\r" not in text


def test_a_section_without_a_schema_is_refused_by_name(pack: Path, schemas: Path) -> None:
    _yaml(pack / "people" / "ana.yaml", {"name": "Ana Lee", "hobby": {"place": "yard"}})
    with pytest.raises(TileRefused) as refused:
        tile(pack, 2, "x", schemas)
    assert "'hobby'" in refused.value.what and "section-hobby.schema.json" in refused.value.why


def test_a_reference_to_an_undefined_key_is_refused_naming_the_file_and_the_key(
    pack: Path, schemas: Path
) -> None:
    _yaml(pack / "places" / "yard.yaml", {"passages": [{"to": "cellar"}]})
    with pytest.raises(TileRefused) as refused:
        tile(pack, 2, "x", schemas)
    assert "places/yard.yaml" in refused.value.what and "'cellar'" in refused.value.why


def test_a_reference_to_a_person_the_partial_block_lacks_is_refused(
    pack: Path, schemas: Path
) -> None:
    _yaml(pack / "people" / "ana.yaml", {"name": "Ana Lee", "pals": ["ben"]})
    with pytest.raises(TileRefused) as refused:
        tile(pack, 3, "x", schemas)
    assert "'ben'" in refused.value.why and "block 02" in refused.value.why


def test_a_configure_file_with_a_reference_is_refused(pack: Path, schemas: Path) -> None:
    _yaml(
        pack / "world.yaml",
        {**yaml.safe_load((pack / "world.yaml").read_text()), "configure": ["meet"]},
    )
    _yaml(pack / "configure" / "meet.yaml", {"spot": "hall"})
    with pytest.raises(TileRefused) as refused:
        tile(pack, 2, "x", schemas)
    assert refused.value.what == "configure/meet.yaml"


def test_organizations_and_an_unknown_world_key_are_refused(pack: Path, schemas: Path) -> None:
    world = yaml.safe_load((pack / "world.yaml").read_text())
    _yaml(pack / "world.yaml", {**world, "weather": {"kind": "rain"}})
    with pytest.raises(TileRefused) as refused:
        tile(pack, 2, "x", schemas)
    assert refused.value.what == "world.yaml weather"
    assert "weather" not in _read(tile(pack, 2, "x", schemas, drop=["weather"]), "world.yaml")
    (pack / "organizations").mkdir()
    with pytest.raises(TileRefused) as refused:
        tile(pack, 2, "x", schemas)
    assert refused.value.what == "organizations"


def test_a_key_too_long_to_suffix_is_refused(pack: Path, schemas: Path) -> None:
    long = "a" * 62
    world = yaml.safe_load((pack / "world.yaml").read_text())
    _yaml(pack / "world.yaml", {**world, "items": [long]})
    _yaml(pack / "items" / f"{long}.yaml", {})
    with pytest.raises(TileRefused) as refused:
        tile(pack, 2, "x", schemas)
    assert refused.value.what == long


def test_the_command_refuses_an_existing_output_and_writes_nothing(
    pack: Path, schemas: Path, tmp_path: Path
) -> None:
    out = tmp_path / "out"
    out.mkdir()
    arguments = [
        "--source",
        str(pack),
        "--people",
        "2",
        "--id",
        "out",
        "--out",
        str(out),
        "--schemas",
        str(schemas),
    ]
    assert main(arguments) == 2
    assert not any(out.iterdir())
    out.rmdir()
    assert main(arguments) == 0
    assert (out / "world.yaml").is_file()


# ── The shipped worlds, with the repository's schemas ────────────────────────────────────────────

CAFE = WORLDS / "social-cafe"
# The café's settlement names one station hall that every seat must reach; independent blocks cannot keep
# that (QS28g-1), so the tiled town drops it (the loader treats an absent settlement as optional).
CAFE_ARGS = {"unique": ["name"], "drop": ["settlement"]}


def _list(data: dict[str, object], kind: str) -> list[str]:
    found: object = data.get(kind) or []
    assert isinstance(found, list)
    return [str(one) for one in cast(list[object], found)]


def _manifest(world: Path) -> dict[str, object]:
    return yaml.safe_load((world / "world.yaml").read_text(encoding="utf-8"))


def _source_keys(world: Path) -> set[str]:
    manifest = _manifest(world)
    return {key for kind in ("places", "items", "population") for key in _list(manifest, kind)}


def test_the_cafe_tiled_to_a_hundred_people_has_every_copy_in_its_own_block() -> None:
    # AC-g-1's shape: 9 blocks (8 of 12 and one of 4), every place and item in every block, the copies of the 11
    # source seats that exist, and every reference naming its own block.
    files = tile(CAFE, 100, "town-100", SCHEMAS, **CAFE_ARGS)
    source = _manifest(CAFE)
    world = _read(files, "world.yaml")
    assert len(_list(world, "places")) == 9 * len(_list(source, "places"))
    assert len(_list(world, "items")) == 9 * len(_list(source, "items"))
    assert len(_list(world, "population")) == 100
    assert [key for key in _list(world, "population") if key.endswith("-09")] == [
        "alice-09",
        "bob-09",
        "carol-09",
        "dev-09",
    ]
    ninth = {"alice", "bob", "carol", "dev"}
    seats = _list(source, "seats")
    assert len(_list(world, "seats")) == 8 * len(seats) + len(ninth & set(seats))
    for path, text in files.items():
        block = re.search(r"-(\d\d)\.yaml$", path.name)
        if block is None:
            continue
        named = set(re.findall(r"\b[a-z0-9-]+-(\d\d)\b", text))
        assert named <= {block.group(1)}, (path, named)


def test_the_tiler_names_no_world_key() -> None:
    # AC-g-3's scan: no key of the shipped packs appears as a word in tile.py.
    text = (Path(__file__).resolve().parents[1] / "tile.py").read_text(encoding="utf-8")
    words = set(re.findall(r"[a-z0-9][a-z0-9_-]*", text))
    keys: set[str] = set()
    for world in WORLDS.iterdir():
        if (world / "world.yaml").is_file():
            keys |= _source_keys(world)
    assert not (words & keys), sorted(words & keys)


def test_market_town_is_refused_naming_organizations() -> None:
    with pytest.raises(TileRefused) as refused:
        tile(WORLDS / "market-town", 30, "x", SCHEMAS, **CAFE_ARGS)
    assert refused.value.what == "organizations"


def _digest(files: dict[PurePosixPath, str]) -> str:
    sha = hashlib.sha256()
    for path, text in sorted(files.items()):
        sha.update(str(path).encode() + b"\0" + text.encode("utf-8") + b"\0")
    return sha.hexdigest()


@pytest.mark.real_binary
@pytest.mark.parametrize(("world", "people"), [("social-cafe", 24), ("bodies-yard", 30)])
def test_a_tiled_world_loads_and_runs_a_day(
    binary: Path, tmp_path: Path, world: str, people: int
) -> None:
    # AC-g-1 / AC-g-3: the loader is the authority on the tiled pack. AC-g-2: two tilings, the same bytes.
    files = tile(WORLDS / world, people, f"{world}-{people}", SCHEMAS, **CAFE_ARGS)
    assert _digest(files) == _digest(
        tile(WORLDS / world, people, f"{world}-{people}", SCHEMAS, **CAFE_ARGS)
    )
    out = tmp_path / f"{world}-{people}"
    write(files, out)
    # ARC-341: a run takes at most stride - 1 seats; the smallest divisor of the default pace (900 s) that is at
    # least the default stride (30 s) and exceeds the tiled world's seats.
    seats = len(_list(_read(files, "world.yaml"), "seats"))
    stride = next(s for s in range(30, 901) if 900 % s == 0 and s > seats)
    result = subprocess.run(
        [
            str(binary),
            "run",
            str(out),
            "--headless",
            "--seed",
            "7",
            "--days",
            "1",
            "--pace",
            "900",
            "--stride",
            str(stride),
            "--save",
            str(tmp_path / "save"),
        ],
        capture_output=True,
        encoding="utf-8",
        timeout=600,
        check=False,
    )
    assert result.returncode == 0, result.stderr
