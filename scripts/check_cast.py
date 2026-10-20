#!/usr/bin/env python3
"""The cast binding of a 3D Presentation Pack is structurally valid (`docs/DECISIONS.md` `ARC-92`).

`characters/cast.yaml` says which character draws each person a client is told about (`ARC-91`,
`ARC-92`). Godot reads it with its built-in JSON parser, so it is written in YAML's JSON-compatible
subset (`DEP-16`). A file that only looks valid draws everybody as nobody, and nothing else reads it
until S26-c2, so this checker holds its structure now: the keys, their types and the shape of every
id. S26-c1 adds CC-7 (every character the binding names has its directory `cast/<id>/` beside the file)
and CC-8 (a `cast/<id>/character.yaml` manifest, where one exists, has the fields the 3D client builds a
body from, and its scene is a file of the 3D client; `pr-s26c1-cast-client.md` §3 D-1).

Usage:

    python3 scripts/check_cast.py [FILE ...]     default: presentation/mineworld-default/3D/characters/cast.yaml
    python3 scripts/check_cast.py --self-test

Each failure prints `<file>: <rule>: <detail>`. Exit 1 if any file fails, else each file prints one line
and the exit is 0. The rules `CC-1` to `CC-6` are specified in `pr-s26a-appearance.md` §5.7, `CC-7` and `CC-8` above.
"""

from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path
from typing import cast

ROOT = Path(__file__).resolve().parent.parent
DEFAULT = (
    ROOT / "presentation" / "mineworld-default" / "3D" / "characters" / "cast.yaml"
)
SHAPE = re.compile(r"[a-z0-9-]{1,48}")
TOP_KEYS = {"format", "looks", "extras", "default"}
VALID = '{"format": 1, "looks": {}, "extras": [], "default": "vera-lindgren"}'


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
        return [
            (
                "CC-1",
                f"not JSON: {error.msg} at line {error.lineno} column {error.colno}",
            )
        ], None
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
                    failures.append(
                        ("CC-4", f"look key {key!r} is not 1 to 48 of a-z, 0-9 and -")
                    )
                if not _is_id(value):
                    failures.append(
                        ("CC-4", f"look {key!r} names {value!r}, not a character id")
                    )

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


CLIENT_3D = ROOT / "clients" / "3d-spike"
MANIFEST_NUMBERS = ("stature_m", "authored_height_m")


def _number(value: object) -> bool:
    return isinstance(value, (int, float)) and not isinstance(value, bool) and value > 0


def manifest_failures(manifest: object, character: str) -> list[str]:
    """CC-8: a `character.yaml` (step-26 §4.5, `ARC-93`) has what the 3D client builds a body from."""
    if not isinstance(manifest, dict):
        return ["not a JSON object"]
    m = cast(dict[str, object], manifest)
    failures: list[str] = []
    if m.get("format") != 1:
        failures.append("format is not 1")
    if m.get("id") != character:
        failures.append(f"id is {m.get('id')!r}, not {character!r}")
    scene = m.get("scene")
    if not isinstance(scene, str) or not scene.startswith("res://"):
        failures.append("scene is not a res:// path")
    elif not (CLIENT_3D / scene.removeprefix("res://")).is_file():
        failures.append(f"scene {scene} is not a file of the 3D client")
    if not isinstance(m.get("material"), str) or not m.get("material"):
        failures.append("material is missing")
    for field in MANIFEST_NUMBERS:
        if not _number(m.get(field)):
            failures.append(f"{field} is missing or not a positive number")
    clips = m.get("clips")
    if not isinstance(clips, dict):
        failures.append("clips is missing")
    else:
        for field in ("walk_mps", "jog_mps"):
            if not _number(cast(dict[str, object], clips).get(field)):
                failures.append(f"clips.{field} is missing or not a positive number")
    return failures


INDEX_3D = CLIENT_3D / "assets" / "characters" / "cast" / "index.json"
# The contact tolerances (S36-P1/P2, I-S36-2): CC-9 takes its bounds from the file, never from a literal, and
# refuses a block measured under a file with another hash.
TOLERANCES = ROOT / "tools" / "character_pipeline" / "contact_tolerances.json"


class Bounds:
    """The tolerance file's bounds CC-9 reads, and its SHA-256 (the one a recorded block names)."""

    def __init__(self, data: bytes) -> None:
        regions = cast(
            dict[str, dict[str, object]], json.loads(data.decode("utf-8"))["regions"]
        )
        self.sha256 = hashlib.sha256(data).hexdigest()
        self._mm = {
            key: float(cast(float, entry["bound_mm"])) for key, entry in regions.items()
        }

    @staticmethod
    def load(path: Path = TOLERANCES) -> Bounds:
        return Bounds(path.read_bytes())

    def __getitem__(self, key: str) -> float:
        return self._mm[key]


def _mm(block: dict[str, object], *path: str) -> float | None:
    value: object = block
    for key in path:
        if not isinstance(value, dict):
            return None
        value = cast(dict[str, object], value).get(key)
    return (
        float(value)
        if isinstance(value, (int, float)) and not isinstance(value, bool)
        else None
    )


def ground_failures(ground: object, bounds: Bounds | None = None) -> list[str]:
    """CC-9: a body carries a ground measurement (`cast.py ground`, PIPELINE.md §15.1), measured under the current
    tolerance file (its SHA-256), within its bounds; recomputed from the recorded numbers, never taken from the
    block's own verdict.

    G-4w: the lowest skinned vertex at rest, over the Stand loop and every frame of one Walk and one Jog cycle,
    and each foot's lowest point in each clip, at most `floor.sink.foot` below the floor; the shins at most
    `floor.sink.shin`; each foot, where planted, at most `floor.float.planted` above it; the Stand loop's and
    each walk frame's lowest point, and each foot's lowest point in the Jog, at most `floor.float.walk_frame`.
    G-9w: no limb deeper than `self.limb_in_trunk` and no hand deeper than `self.hand_in_trunk` inside the trunk
    in any walk or jog frame."""
    if not isinstance(ground, dict):
        return [
            "no ground measurement (run `tools/character_pipeline/cast.py ground --write`)"
        ]
    b = bounds or Bounds.load()
    g = cast(dict[str, object], ground)
    if g.get("tolerances_sha256") != b.sha256:
        return [
            "ground: measured under other bounds; re-run `tools/character_pipeline/cast.py ground --write`"
        ]
    failures: list[str] = []
    sink_foot, sink_shin = b["floor.sink.foot"], b["floor.sink.shin"]
    float_planted, float_frame = b["floor.float.planted"], b["floor.float.walk_frame"]
    sinks = {
        "rest": _mm(g, "rest_mm"),
        "stand": _mm(g, "stand", "min_mm"),
        "walk": _mm(g, "walk", "min_mm"),
        "jog": _mm(g, "jog", "min_mm"),
    }
    floats = {
        "stand": _mm(g, "stand", "max_mm"),
        "walk": _mm(g, "walk", "frame_low_max_mm"),
    }
    jog = g.get("jog")
    feet_low = (
        cast(dict[str, object], jog).get("foot_contact_mm")
        if isinstance(jog, dict)
        else None
    )
    if isinstance(feet_low, list) and len(cast(list[object], feet_low)) == 2:
        for side, value in zip(
            ("jog left foot", "jog right foot"), cast(list[object], feet_low)
        ):
            floats[side] = float(value) if isinstance(value, (int, float)) else None
    else:
        floats["jog feet"] = None
    planted: dict[str, float | None] = {}
    for side in ("left", "right"):
        for clip in ("stand", "walk", "jog"):
            sinks[f"feet.{side}.{clip}"] = _mm(g, "feet", side, clip, "sink_mm")
            # a clip in which the foot is never planted (share 0) has no planted float to judge
            if _mm(g, "feet", side, clip, "planted_share") != 0.0:
                planted[f"feet.{side}.{clip}"] = _mm(
                    g, "feet", side, clip, "planted_float_mm"
                )
    for name, value in sinks.items():
        if value is None:
            failures.append(f"ground.{name}: no measurement")
        elif value < -sink_foot:
            failures.append(
                f"ground.{name}: {value} mm below the floor (floor.sink.foot {sink_foot:g} mm)"
            )
    for name, value in floats.items():
        if value is None:
            failures.append(f"ground.{name}: no measurement")
        elif value > float_frame:
            failures.append(
                f"ground.{name}: floats {value} mm (floor.float.walk_frame {float_frame:g} mm)"
            )
    for name, value in planted.items():
        if value is None:
            failures.append(f"ground.{name}: no planted measurement")
        elif value > float_planted:
            failures.append(
                f"ground.{name}: floats {value} mm while planted (floor.float.planted {float_planted:g} mm)"
            )
    shin = g.get("shin_sink_mm", "missing")
    if shin == "missing" or (shin is not None and not isinstance(shin, (int, float))):
        failures.append("ground.shin_sink_mm: no measurement")
    elif isinstance(shin, (int, float)) and shin < -sink_shin:
        failures.append(
            f"ground.shin_sink_mm: {shin} mm below the floor (floor.sink.shin {sink_shin:g} mm)"
        )
    for kind, key in (("limb", "self.limb_in_trunk"), ("hand", "self.hand_in_trunk")):
        value = _mm(g, "limb_in_trunk_mm", kind)
        if value is None:
            failures.append(f"ground.limb_in_trunk_mm.{kind}: no measurement")
        elif value > b[key]:
            failures.append(
                f"ground.limb_in_trunk_mm.{kind}: {value} mm inside the trunk in the gait ({key} {b[key]:g} mm)"
            )
    return failures


ARM_CLEAR_MM = 20.0


def stance_failures(stance: object, ground: object = None) -> list[str]:
    """CC-10: a body carries its relaxed standing arms (`cast.py ground --solve-stance`, PIPELINE.md §15.2):
    `stance.arms`, bone -> three rest-relative euler degrees, solved on its own mesh; and every arm the stance
    solves (its `<Side>UpperArm` is keyed) clears the torso, hips and thighs by at least 20 mm as built
    (`ground.arm_clearance_mm`, [left, right]). An arm the stance leaves alone (the protagonist's strap hand) is
    not held to it."""
    if not isinstance(stance, dict):
        return [
            "no stance (run `tools/character_pipeline/cast.py ground --solve-stance --write`)"
        ]
    arms = cast(dict[str, object], stance).get("arms")
    if not isinstance(arms, dict) or not arms:
        return ["stance.arms is missing"]
    bad = [
        bone
        for bone, value in cast(dict[str, object], arms).items()
        if not (isinstance(value, list) and len(cast(list[object], value)) == 3)
    ]
    failures = [f"stance.arms.{bone} is not three numbers" for bone in bad]
    clear = (
        cast(dict[str, object], ground).get("arm_clearance_mm")
        if isinstance(ground, dict)
        else None
    )
    for i, side in enumerate(("Left", "Right")):
        if f"{side}UpperArm" not in cast(dict[str, object], arms):
            continue
        value = (
            cast(list[object], clear)[i]
            if isinstance(clear, list) and len(clear) == 2
            else None
        )
        if not isinstance(value, (int, float)):
            failures.append(f"ground.arm_clearance_mm: no {side.lower()} measurement")
        elif value < ARM_CLEAR_MM:
            failures.append(
                f"{side.lower()} arm {value} mm from the body (bound {ARM_CLEAR_MM})"
            )
    return failures


def reference_failures() -> list[tuple[str, str]]:
    """CC-9 for the bodies the 3D client's index draws itself (the protagonist): the measurement is there."""
    if not INDEX_3D.is_file():
        return []
    index = json.loads(INDEX_3D.read_text(encoding="utf-8"))
    failures: list[tuple[str, str]] = []
    for character, entry in sorted(
        cast(dict[str, dict[str, object]], index.get("characters", {})).items()
    ):
        if entry.get("body") == "reference":
            for detail in ground_failures(entry.get("ground")):
                failures.append(("CC-9", f"{INDEX_3D}: {character}: {detail}"))
            if "stance" in entry:
                for detail in stance_failures(entry.get("stance"), entry.get("ground")):
                    failures.append(("CC-10", f"{INDEX_3D}: {character}: {detail}"))
    return failures


def character_failures(path: Path, data: dict[str, object]) -> list[tuple[str, str]]:
    """CC-7: every character the binding names has its directory beside the file; CC-8: its manifest, where
    it has one, is well formed (a character without one is drawn by the client's own index or as the town
    figure, PRESENTATION.md §3)."""
    named: set[str] = set(cast(dict[str, str], data["looks"]).values())
    named |= set(cast(list[str], data["extras"]))
    named.add(cast(str, data["default"]))
    failures: list[tuple[str, str]] = []
    for character in sorted(named):
        directory = path.parent / "cast" / character
        if not directory.is_dir():
            failures.append(
                ("CC-7", f"character {character!r} has no directory {directory}")
            )
            continue
        manifest = directory / "character.yaml"
        if manifest.is_file():
            try:
                parsed: object = json.loads(manifest.read_text(encoding="utf-8"))
            except json.JSONDecodeError as error:
                failures.append(("CC-8", f"{manifest}: not JSON: {error.msg}"))
                continue
            for detail in manifest_failures(parsed, character):
                failures.append(("CC-8", f"{manifest}: {detail}"))
            if isinstance(parsed, dict):
                for detail in ground_failures(
                    cast(dict[str, object], parsed).get("ground")
                ):
                    failures.append(("CC-9", f"{manifest}: {detail}"))
                for detail in stance_failures(
                    cast(dict[str, object], parsed).get("stance"),
                    cast(dict[str, object], parsed).get("ground"),
                ):
                    failures.append(("CC-10", f"{manifest}: {detail}"))
    return failures + reference_failures()


def self_test() -> int:
    cases: list[tuple[str, str | None]] = [
        (VALID, None),
        ("not json {", "CC-1"),
        (
            '{"format": 1, "format": 1, "looks": {}, "extras": [], "default": "a"}',
            "CC-1",
        ),
        (
            '{"format": 1, "looks": {}, "extras": [], "default": "a", "colour": "red"}',
            "CC-2",
        ),
        ('{"format": 1, "looks": {}, "extras": []}', "CC-2"),
        ('{"format": 2, "looks": {}, "extras": [], "default": "a"}', "CC-3"),
        ('{"format": true, "looks": {}, "extras": [], "default": "a"}', "CC-3"),
        (
            '{"format": 1, "looks": {"Alice": "a"}, "extras": [], "default": "a"}',
            "CC-4",
        ),
        (
            '{"format": 1, "looks": {"alice": "alice moreau"}, "extras": [], "default": "a"}',
            "CC-4",
        ),
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
            print(
                f"self-test: case {number} should fail {expected}, got {sorted(rules) or 'nothing'}"
            )
            return 1
    # CC-8 (X-14's shape): a manifest without its stature is refused by name; the shipped shape passes.
    good = {
        "format": 1,
        "id": "alice-moreau",
        "scene": "res://assets/characters/cast/alice-moreau/alice-moreau.gltf",
        "material": "MW_Cast",
        "stature_m": 1.61,
        "authored_height_m": 1.61,
        "clips": {"walk_mps": 1.016, "jog_mps": 2.573},
    }
    if manifest_failures(good, "alice-moreau"):
        print(
            f"self-test: the shipped manifest shape should pass: {manifest_failures(good, 'alice-moreau')}"
        )
        return 1
    missing = {k: v for k, v in good.items() if k != "stature_m"}
    if not any("stature_m" in f for f in manifest_failures(missing, "alice-moreau")):
        print("self-test: a manifest without stature_m should fail naming it")
        return 1
    # CC-9: a passing ground block passes; a body sinking 12 mm in the walk, one floating 30 mm standing, a planted
    # foot floating 12 mm, a hand 8 mm in the trunk, a block measured under other bounds, and one with no
    # measurement are refused by name, whatever their own `pass` says.
    bounds = Bounds.load()
    foot = {"sink_mm": -0.5, "planted_float_mm": 1.0, "planted_share": 0.45}
    ground: dict[str, object] = {
        "rest_mm": 0.0,
        "stand": {"min_mm": -0.2, "max_mm": -0.2},
        "walk": {"min_mm": -0.8, "frame_low_max_mm": 0.0},
        "jog": {"min_mm": -0.8, "foot_contact_mm": [-0.7, -0.1]},
        "feet": {
            side: {
                "stand": dict(foot, planted_share=1.0),
                "walk": dict(foot),
                "jog": dict(foot),
            }
            for side in ("left", "right")
        },
        "shin_sink_mm": 3.0,
        "limb_in_trunk_mm": {"limb": 0.0, "hand": 1.2},
        "tolerances_sha256": bounds.sha256,
        "pass": True,
    }
    if ground_failures(ground, bounds):
        print(
            f"self-test: a passing ground block should pass: {ground_failures(ground, bounds)}"
        )
        return 1
    sunk = json.loads(json.dumps(ground))
    sunk["walk"]["min_mm"] = -12.0
    floating = json.loads(json.dumps(ground))
    floating["stand"]["max_mm"] = 30.0
    hovering = json.loads(json.dumps(ground))
    hovering["feet"]["right"]["walk"]["planted_float_mm"] = 12.0
    hand = json.loads(json.dumps(ground))
    hand["limb_in_trunk_mm"]["hand"] = 8.0
    stale = json.loads(json.dumps(ground))
    stale["tolerances_sha256"] = "0" * 64
    for label, bad, word in (
        ("sunk", sunk, "ground.walk"),
        ("floating", floating, "ground.stand"),
        ("planted-float", hovering, "feet.right.walk: floats 12.0 mm while planted"),
        ("hand-in-trunk", hand, "limb_in_trunk_mm.hand"),
        ("stale-bounds", stale, "measured under other bounds"),
    ):
        if not any(word in f for f in ground_failures(bad, bounds)):
            print(
                f"self-test: a {label} ground block should fail naming {word}: {ground_failures(bad, bounds)}"
            )
            return 1
    if not stance_failures(
        {"arms": {"RightUpperArm": [80.0, 0.0, -15.0]}},
        {"arm_clearance_mm": [-20.0, 5.0]},
    ) or stance_failures(
        {"arms": {"RightUpperArm": [80.0, 0.0, -15.0]}},
        {"arm_clearance_mm": [-20.0, 25.0]},
    ):
        print(
            "self-test: CC-10 should hold a solved arm to 20 mm and leave an unsolved one alone"
        )
        return 1
    if not stance_failures(None) or stance_failures(
        {"arms": {"LeftUpperArm": [80.0, 0.0, 15.0]}},
        {"arm_clearance_mm": [30.0, 30.0]},
    ):
        print(
            "self-test: CC-10 should refuse a missing stance and pass a well-formed one"
        )
        return 1
    if not ground_failures(None):
        print("self-test: a manifest without a ground block should fail")
        return 1
    print(f"self-test: {len(cases) + 8} cases, all as expected")
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
        if not failures and data is not None:
            failures = character_failures(path, data)
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
