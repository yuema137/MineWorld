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


# The recorded ground findings (S36-P2 §5 AC-4): a reading that fails G-4w or G-9w on a shipped body for a reason
# this PR cannot fix (a clip or a pose; the fix is owed by the lane named), with its measured value. A finding holds
# only while the reading stays within FINDING_SLACK_MM of it; deeper is a new failure, and a finding whose reading
# now passes is stale and fails until it is removed. `cast.py ground --write` never writes this file.
FINDINGS = ROOT / "tools" / "character_pipeline" / "ground_findings.json"
FINDING_SLACK_MM = 0.1


def load_findings(path: Path = FINDINGS) -> dict[str, dict[str, float]]:
    """Body id -> reading path ("feet.right.stand.planted_float_mm") -> recorded mm."""
    if not path.is_file():
        return {}
    document = cast(dict[str, object], json.loads(path.read_text(encoding="utf-8")))
    bodies = cast(dict[str, list[dict[str, object]]], document.get("findings", {}))
    return {
        body: {str(f["reading"]): float(cast(float, f["mm"])) for f in rows}
        for body, rows in bodies.items()
    }


def ground_failures(
    ground: object,
    bounds: Bounds | None = None,
    findings: dict[str, float] | None = None,
) -> list[str]:
    """CC-9: a body carries a ground measurement (`cast.py ground`, PIPELINE.md §15.1), measured under the current
    tolerance file (its SHA-256), within its bounds; recomputed from the recorded numbers, never taken from the
    block's own verdict. `findings` (reading path -> recorded mm) excuses a reading up to its recorded value.

    G-4w: the lowest skinned vertex at rest, over the Stand loop and every frame of one Walk and one Jog cycle,
    and each foot's lowest point in each clip, at most `floor.sink.foot` below the floor; the shins at most
    `floor.sink.shin`; each foot, where planted, at most `floor.float.planted` above it; the Stand loop's and
    each walk frame's lowest point, and each foot's lowest point in the Jog, at most `floor.float.walk_frame`.
    G-9w's arm readings are CC-10's (`arm_failures`); their findings (`arms.…`) are left to it."""
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
    # (reading path, value or None, how far over the bound it may go (+) or under (-), bound, key)
    checks: list[tuple[str, float | None, int, float, str]] = []
    for path in ("rest_mm", "stand.min_mm", "walk.min_mm", "jog.min_mm"):
        checks.append(
            (
                path,
                _mm(g, *path.split(".")),
                -1,
                b["floor.sink.foot"],
                "floor.sink.foot",
            )
        )
    for path in ("stand.max_mm", "walk.frame_low_max_mm"):
        checks.append(
            (
                path,
                _mm(g, *path.split(".")),
                1,
                b["floor.float.walk_frame"],
                "floor.float.walk_frame",
            )
        )
    jog = g.get("jog")
    feet_low = (
        cast(dict[str, object], jog).get("foot_contact_mm")
        if isinstance(jog, dict)
        else None
    )
    for i, side in enumerate(("left", "right")):
        value = (
            cast(list[object], feet_low)[i]
            if isinstance(feet_low, list) and len(cast(list[object], feet_low)) == 2
            else None
        )
        checks.append(
            (
                f"jog.foot_contact_mm.{side}",
                float(value) if isinstance(value, (int, float)) else None,
                1,
                b["floor.float.walk_frame"],
                "floor.float.walk_frame",
            )
        )
        for clip in ("stand", "walk", "jog"):
            base = f"feet.{side}.{clip}"
            checks.append(
                (
                    f"{base}.sink_mm",
                    _mm(g, "feet", side, clip, "sink_mm"),
                    -1,
                    b["floor.sink.foot"],
                    "floor.sink.foot",
                )
            )
            # a clip in which the foot is never planted (share 0) has no planted float to judge
            if _mm(g, "feet", side, clip, "planted_share") != 0.0:
                checks.append(
                    (
                        f"{base}.planted_float_mm",
                        _mm(g, "feet", side, clip, "planted_float_mm"),
                        1,
                        b["floor.float.planted"],
                        "floor.float.planted",
                    )
                )
    shin = g.get("shin_sink_mm", "missing")
    if shin is not None:
        checks.append(
            (
                "shin_sink_mm",
                float(shin) if isinstance(shin, (int, float)) else None,
                -1,
                b["floor.sink.shin"],
                "floor.sink.shin",
            )
        )
    known = {k: v for k, v in (findings or {}).items() if not k.startswith("arms.")}
    failures: list[str] = []
    for path, value, sign, bound, key in checks:
        if value is None:
            failures.append(f"ground.{path}: no measurement")
            continue
        over = value > bound if sign > 0 else value < -bound
        recorded = known.pop(path, None)
        if not over:
            if recorded is not None:
                failures.append(
                    f"ground.{path}: stale finding ({recorded} mm recorded, now {value} mm within {key}); "
                    "remove it from ground_findings.json"
                )
            continue
        if recorded is not None and abs(value) <= abs(recorded) + FINDING_SLACK_MM:
            continue
        what = "above" if sign > 0 else "below"
        failures.append(
            f"ground.{path}: {value} mm ({what} the {key} bound of {bound:g} mm)"
        )
    for path in sorted(known):
        failures.append(
            f"ground.{path}: a finding names a reading the block does not have"
        )
    return failures


# CC-10 (PR S36-CC10): the clips whose G-9w arm readings are judged, and those recorded only (operator question
# Q-S36P2-2: the walking arm pose is being replaced). Which clips are judged is policy, not a second rule.
ARMS_JUDGED = ("rest", "stand")
ARMS_RECORDED = ("walk", "jog")


def arm_failures(
    ground: object,
    bounds: Bounds | None = None,
    findings: dict[str, float] | None = None,
) -> list[str]:
    """CC-10: G-9w's arm readings (`ground.arms`, PIPELINE.md §15.1), measured under the current tolerance file,
    are within the bound of their recorded key in every judged clip (rest, stand): per clip, the worst limb and
    the worst hand reading (each arm vertex inside the body core by winding number and pseudonormal sign, its
    depth against its closest face's region key). The walk and jog readings must be present and are not judged.
    `findings` (`arms.<clip>.<kind>_mm` -> recorded mm) excuses a reading up to its recorded value + 0.1 mm; a
    finding whose reading now passes is stale and fails. Recomputed from the numbers, never the block's verdict."""
    if not isinstance(ground, dict):
        return ["no ground measurement (run `tools/character_pipeline/cast.py ground --write`)"]
    b = bounds or Bounds.load()
    g = cast(dict[str, object], ground)
    if g.get("tolerances_sha256") != b.sha256:
        return ["ground.arms: measured under other bounds; re-run `tools/character_pipeline/cast.py ground --write`"]
    arms = g.get("arms")
    if not isinstance(arms, dict):
        return ["ground.arms: no measurement"]
    known = {k: v for k, v in (findings or {}).items() if k.startswith("arms.")}
    failures: list[str] = []
    for clip in (*ARMS_JUDGED, *ARMS_RECORDED):
        entry = cast(dict[str, object], arms).get(clip)
        keys = cast(dict[str, object], entry).get("keys") if isinstance(entry, dict) else None
        for kind in ("limb", "hand"):
            path = f"arms.{clip}.{kind}_mm"
            value = _mm(g, "arms", clip, f"{kind}_mm")
            if value is None:
                failures.append(f"ground.{path}: no measurement")
                continue
            if clip not in ARMS_JUDGED:
                continue
            key = cast(dict[str, object], keys).get(kind) if isinstance(keys, dict) else None
            recorded = known.pop(path, None)
            if value == 0.0 and not key:
                key = "self.hand_in_trunk" if kind == "hand" else "self.limb_in_trunk"
            if not isinstance(key, str):
                failures.append(f"ground.arms.{clip}.keys.{kind}: no bound key")
                continue
            try:
                bound = b[key]
            except KeyError:
                failures.append(f"ground.arms.{clip}.keys.{kind}: {key!r} is not a key of the tolerance file")
                continue
            if value <= bound:
                if recorded is not None:
                    failures.append(
                        f"ground.{path}: stale finding ({recorded} mm recorded, now {value} mm within {key}); "
                        "remove it from ground_findings.json"
                    )
                continue
            if recorded is not None and value <= recorded + FINDING_SLACK_MM:
                continue
            failures.append(f"ground.{path}: {value} mm inside the body (the {key} bound of {bound:g} mm)")
    for path in sorted(known):
        failures.append(f"ground.{path}: a finding names a reading the block does not have")
    return failures


# PR S26-c-pose part E (PIPELINE.md §15.2): the bounds the probe's `stand_gates` block is re-checked against. AAOS
# normal active ranges (Greene & Heckman, The Clinical Measurement of Joint Motion, AAOS 1994; elbow and forearm
# agree with Zwerus et al. 2017), degrees from the clinical zero.
ROM_DEG = {
    "shoulder_flexion": (-60.0, 180.0),
    "shoulder_abduction": (-50.0, 180.0),
    "shoulder_rotation": (-90.0, 70.0),
    "elbow_flexion": (0.0, 150.0),
    "forearm_pronation": (-80.0, 80.0),
    "wrist_flexion": (-70.0, 80.0),
    "wrist_deviation": (-30.0, 20.0),
}
# The forearm-rotation gate (operator ruling 2026-10-19, replacing the palm-direction gate): no supination, and
# pronation from the thumb-forward neutral within the relaxed band (0-60 deg) inside AAOS's 0-80 (ROM_DEG).
PRONATION_RELAXED = (0.0, 60.0)


def stand_failures(gates: object) -> list[str]:
    """CC-11: every gated arm (both, except the side a reference body's strap grip holds) keeps, on every frame of
    the Stand loop, its joint angles inside the AAOS normal ranges and its forearm rotation inside the relaxed band
    (no supination; pronation from the thumb-forward neutral at most 60 deg; operator ruling 2026-10-19). The palm
    angle and the reach are recorded and must be present, but are not bounds until the 22-body table sets them
    (the same ruling). Arm-body contact is CC-10's (`arm_failures`). Recomputed from the recorded numbers, never from the block's own verdict."""
    if not isinstance(gates, dict):
        return [
            "no stand_gates block (run `tools/character_pipeline/cast.py ground --write`)"
        ]
    sides = cast(dict[str, object], gates).get("sides")
    if not isinstance(sides, dict):
        return ["stand_gates.sides is missing"]
    failures: list[str] = []
    for side in ("Left", "Right"):
        s = cast(dict[str, object], sides).get(side)
        if not isinstance(s, dict):
            failures.append(f"stand_gates.sides.{side} is missing")
            continue
        g = cast(dict[str, object], s)
        if not g.get("gated", False):
            continue
        for key in ("palm_deg", "thumb_ahead_mm", "reach_mm"):
            if not isinstance(g.get(key), (int, float)):
                failures.append(f"CC-11 {side} {key}: no measurement")
        rom = g.get("rom_deg")
        for joint, (lo, hi) in ROM_DEG.items():
            span = (
                cast(dict[str, object], rom).get(joint)
                if isinstance(rom, dict)
                else None
            )
            if not (isinstance(span, list) and len(cast(list[object], span)) == 2):
                failures.append(f"CC-11 {side} {joint}: no measurement")
                continue
            a, b = cast(list[float], span)
            if a < lo or b > hi:
                failures.append(
                    f"CC-11 {side} {joint} {a}..{b} deg outside {lo}..{hi} (AAOS)"
                )
        pron = (
            cast(dict[str, object], rom).get("forearm_pronation")
            if isinstance(rom, dict)
            else None
        )
        if isinstance(pron, list) and len(cast(list[object], pron)) == 2:
            a, b = cast(list[float], pron)
            lo, hi = PRONATION_RELAXED
            if a < lo or b > hi:
                failures.append(
                    f"CC-11 {side} forearm rotation {a}..{b} deg outside the relaxed band {lo}..{hi} "
                    "(no supination; pronation from the thumb-forward neutral)"
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
            for detail in ground_failures(
                entry.get("ground"), None, load_findings().get(character)
            ):
                failures.append(("CC-9", f"{INDEX_3D}: {character}: {detail}"))
            for detail in arm_failures(entry.get("ground"), None, load_findings().get(character)):
                failures.append(("CC-10", f"{INDEX_3D}: {character}: {detail}"))
            for detail in stand_failures(entry.get("stand_gates")):
                failures.append(
                    (
                        detail.split(" ")[0] if detail.startswith("CC-") else "CC-11",
                        f"{INDEX_3D}: {character}: {detail}",
                    )
                )
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
                    cast(dict[str, object], parsed).get("ground"),
                    None,
                    load_findings().get(character),
                ):
                    failures.append(("CC-9", f"{manifest}: {detail}"))
                for detail in arm_failures(
                    cast(dict[str, object], parsed).get("ground"),
                    None,
                    load_findings().get(character),
                ):
                    failures.append(("CC-10", f"{manifest}: {detail}"))
                for detail in stand_failures(cast(dict[str, object], parsed).get("stand_gates")):
                    rule = detail.split(" ")[0] if detail.startswith("CC-") else "CC-11"
                    failures.append((rule, f"{manifest}: {detail}"))
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
    # foot floating 12 mm, a block measured under other bounds, and one with no measurement are refused by name,
    # whatever their own `pass` says.
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
        "arms": {
            clip: {
                "limb_mm": 0.0,
                "hand_mm": 1.2,
                "keys": {"limb": "self.limb_in_trunk", "hand": "self.arm_in_thigh"},
            }
            for clip in ("rest", "stand", "walk", "jog")
        },
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
    stale = json.loads(json.dumps(ground))
    stale["tolerances_sha256"] = "0" * 64
    for label, bad, word in (
        ("sunk", sunk, "ground.walk"),
        ("floating", floating, "ground.stand"),
        ("planted-float", hovering, "feet.right.walk.planted_float_mm: 12.0 mm"),
        ("stale-bounds", stale, "measured under other bounds"),
    ):
        if not any(word in f for f in ground_failures(bad, bounds)):
            print(
                f"self-test: a {label} ground block should fail naming {word}: {ground_failures(bad, bounds)}"
            )
            return 1
    # a recorded finding excuses its reading up to the recorded value; deeper, or stale, fails
    reading = "feet.right.walk.planted_float_mm"
    if (
        ground_failures(hovering, bounds, {reading: 12.0})
        or not ground_failures(hovering, bounds, {reading: 11.0})
        or not any(
            "stale finding" in f
            for f in ground_failures(ground, bounds, {reading: 12.0})
        )
    ):
        print(
            "self-test: a finding should excuse 12 mm, not 11, and go stale once the reading passes"
        )
        return 1
    # CC-10: the passing block passes; a standing hand 7 mm over its bound fails naming its key, the same depth in
    # the walk passes (recorded only), a missing walk reading and a block under other bounds fail; a finding
    # excuses its reading up to + 0.1 mm, not 0.2 mm over it, and goes stale once the reading passes.
    if arm_failures(ground, bounds):
        print(f"self-test: a passing arms block should pass: {arm_failures(ground, bounds)}")
        return 1
    standing = json.loads(json.dumps(ground))
    standing["arms"]["stand"]["hand_mm"] = bounds["self.arm_in_thigh"] + 7.0
    walking = json.loads(json.dumps(ground))
    walking["arms"]["walk"]["hand_mm"] = bounds["self.arm_in_thigh"] + 7.0
    unrecorded = json.loads(json.dumps(ground))
    del unrecorded["arms"]["jog"]["limb_mm"]
    over = f"ground.arms.stand.hand_mm: {bounds['self.arm_in_thigh'] + 7.0} mm inside the body (the self.arm_in_thigh"
    finding = "arms.stand.hand_mm"
    deep = bounds["self.arm_in_thigh"] + 7.0
    if (
        not any(f.startswith(over) for f in arm_failures(standing, bounds))
        or arm_failures(walking, bounds)
        or not any("arms.jog.limb_mm: no measurement" in f for f in arm_failures(unrecorded, bounds))
        or not any("measured under other bounds" in f for f in arm_failures(stale, bounds))
        or arm_failures(standing, bounds, {finding: deep})
        or not arm_failures(standing, bounds, {finding: deep - 0.2})
        or not any("stale finding" in f for f in arm_failures(ground, bounds, {finding: deep}))
        or ground_failures(standing, bounds, {finding: deep})
    ):
        print("self-test: CC-10 should judge rest and stand, record walk and jog, and honour findings exactly")
        return 1
    # CC-11: a passing stand block passes; a supinated forearm, one pronated past the relaxed band and one twisted
    # past AAOS's range each fail by name; palm and reach are recorded only (a 150 deg palm passes), but a missing
    # measurement fails; an ungated (strap) side is left alone; no block fails.
    side_ok = {
        "gated": True,
        "palm_deg": 40.0,
        "thumb_ahead_mm": 8.0,
        "reach_mm": 20.0,
        "rom_deg": {
            k: [max(lo, 0.0), max(lo, 0.0) + 1.0] for k, (lo, _hi) in ROM_DEG.items()
        },
    }
    good_gates = {"sides": {"Left": side_ok, "Right": side_ok}}
    if stand_failures(good_gates):
        print(f"self-test: a passing stand block should pass: {stand_failures(good_gates)}")
        return 1
    rom = side_ok["rom_deg"]
    for label, change, word in (
        (
            "supinated",
            {"rom_deg": {**rom, "forearm_pronation": [-5.0, 20.0]}},
            "relaxed band",
        ),
        (
            "over-pronated",
            {"rom_deg": {**rom, "forearm_pronation": [10.0, 70.0]}},
            "relaxed band",
        ),
        (
            "over-twisted",
            {"rom_deg": {**rom, "forearm_pronation": [-120.0, 0.0]}},
            "forearm_pronation",
        ),
        ("unmeasured palm", {"palm_deg": None}, "palm_deg"),
    ):
        bad = {"sides": {"Left": {**side_ok, **change}, "Right": side_ok}}
        if not any(word in f for f in stand_failures(bad)):
            print(f"self-test: a {label} arm should fail naming {word}: {stand_failures(bad)}")
            return 1
    recorded_only = {
        "sides": {
            "Left": {**side_ok, "palm_deg": 150.0, "reach_mm": 90.0},
            "Right": side_ok,
        }
    }
    if stand_failures(recorded_only):
        print("self-test: palm and reach are recorded only until the 22-body table sets their bounds")
        return 1
    strap = {
        "sides": {
            "Left": {**side_ok, "gated": False, "palm_deg": 90.0},
            "Right": side_ok,
        }
    }
    if stand_failures(strap) or not stand_failures(None):
        print(
            "self-test: an ungated strap side should be left alone, and a missing block should fail"
        )
        return 1
    if not ground_failures(None):
        print("self-test: a manifest without a ground block should fail")
        return 1
    print(f"self-test: {len(cases) + 21} cases, all as expected")
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
