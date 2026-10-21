"""The contact tolerances: every bound of the contact checks, as data with its rationale (I-S36-2).

One file, `tools/character_pipeline/contact_tolerances.json`, holds them (step-36 §3.3). A gate asks
`Tolerances.bound(key)` and never holds a literal. A presentation pack may supply its own file that
tightens a bound and never loosens one (Q-S36-4). The file's SHA-256 is recorded with each measurement,
so a recorded number can be re-judged against the bounds it was measured with.

Standard library only.
"""

from __future__ import annotations

import hashlib
import json
from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path
from typing import cast

DEFAULT_PATH = Path(__file__).resolve().parents[1] / "contact_tolerances.json"
FORMAT = 1
STATUSES = frozenset({"cited", "inherited", "inconclusive"})
# The region keys of step-36 §3.3: every key the contact gates (P1) and the gait gates (P2, P5, CC10) read.
KEYS = frozenset(
    {
        "floor.sink.foot",
        "floor.sink.shin",
        "floor.float.planted",
        "floor.float.walk_frame",
        "self.limb_in_trunk",
        "self.hand_in_trunk",
        "self.arm_in_thigh",
        "garment.poke.body",
        "garment.poke.layer",
        "item.rest.floor",
        "item.overlap.parts",
        "scene.mesh_over_solid",
    }
)
_FIELDS = ("bound_mm", "rationale", "source", "status")


class ToleranceError(Exception):
    """A tolerance file is malformed, or a pack file loosens a bound. The message names the key."""


@dataclass(frozen=True)
class Region:
    bound_mm: float
    rationale: str
    source: str
    status: str


@dataclass(frozen=True)
class Tolerances:
    regions: Mapping[str, Region]
    sha256: str
    path: Path

    @staticmethod
    def load(path: Path = DEFAULT_PATH) -> Tolerances:
        """Read and validate the full tolerance file: every key of `KEYS`, and nothing else."""
        data = path.read_bytes()
        regions = _parse(path, data, partial=False)
        return Tolerances(regions, hashlib.sha256(data).hexdigest(), path)

    def bound(self, key: str) -> float:
        if key not in self.regions:
            raise ToleranceError(f"no tolerance named {key!r}")
        return self.regions[key].bound_mm

    def tightened_by(self, pack_file: Path) -> Tolerances:
        """These tolerances with a pack's file applied. The pack names a subset of the keys; each of its
        bounds must be at most the base bound. A higher bound is refused naming the key (Q-S36-4).

        The SHA-256 of the result covers both files, in order."""
        data = pack_file.read_bytes()
        overrides = _parse(pack_file, data, partial=True)
        merged = dict(self.regions)
        for key, region in overrides.items():
            base = self.regions[key].bound_mm
            if region.bound_mm > base:
                raise ToleranceError(
                    f"{pack_file}: {key} {region.bound_mm:g} mm loosens the base bound {base:g} mm; "
                    "a pack may only tighten"
                )
            merged[key] = region
        digest = hashlib.sha256(self.sha256.encode("ascii") + b"\n" + data).hexdigest()
        return Tolerances(merged, digest, pack_file)


def _parse(path: Path, data: bytes, *, partial: bool) -> dict[str, Region]:
    try:
        document: object = json.loads(data.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise ToleranceError(f"{path}: not valid UTF-8 JSON: {exc}") from exc
    if not isinstance(document, dict):
        raise ToleranceError(f"{path}: the document is not an object")
    top = cast(dict[str, object], document)
    if top.get("format") != FORMAT:
        raise ToleranceError(f"{path}: format is {top.get('format')!r}, expected {FORMAT}")
    raw = top.get("regions")
    if not isinstance(raw, dict):
        raise ToleranceError(f"{path}: 'regions' is not an object")
    entries = cast(dict[str, object], raw)
    unknown = sorted(set(entries) - KEYS)
    if unknown:
        raise ToleranceError(f"{path}: unknown region key(s) {unknown}")
    missing = sorted(KEYS - set(entries))
    if missing and not partial:
        raise ToleranceError(f"{path}: missing region key(s) {missing}")
    return {key: _region(path, key, value) for key, value in sorted(entries.items())}


def _region(path: Path, key: str, value: object) -> Region:
    if not isinstance(value, dict):
        raise ToleranceError(f"{path}: {key} is not an object")
    entry = cast(dict[str, object], value)
    for field in _FIELDS:
        if field not in entry:
            raise ToleranceError(f"{path}: {key} has no '{field}'")
    extra = sorted(set(entry) - set(_FIELDS))
    if extra:
        raise ToleranceError(f"{path}: {key} has unknown field(s) {extra}")
    bound = entry["bound_mm"]
    if isinstance(bound, bool) or not isinstance(bound, int | float) or not bound > 0:
        raise ToleranceError(f"{path}: {key} bound_mm {bound!r} is not a positive number")
    texts: dict[str, str] = {}
    for field in ("rationale", "source", "status"):
        text = entry[field]
        if not isinstance(text, str) or not text.strip():
            raise ToleranceError(f"{path}: {key} '{field}' is empty or not a string")
        texts[field] = text
    if texts["status"] not in STATUSES:
        raise ToleranceError(f"{path}: {key} status {texts['status']!r} is not one of {sorted(STATUSES)}")
    return Region(float(bound), texts["rationale"], texts["source"], texts["status"])
