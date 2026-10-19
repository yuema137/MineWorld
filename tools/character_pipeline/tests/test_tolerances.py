"""The tolerance file: shipped complete, refused when malformed, tightened but never loosened (S36-P1
commit 2)."""

from __future__ import annotations

import json
from collections.abc import Callable
from pathlib import Path
from typing import Any

import pytest

from mwcast.tolerances import KEYS, ToleranceError, Tolerances

Regions = dict[str, dict[str, Any]]


def _shipped() -> dict[str, Any]:
    document: dict[str, Any] = json.loads(Tolerances.load().path.read_text(encoding="utf-8"))
    return document


def _write(tmp_path: Path, name: str, document: object) -> Path:
    path = tmp_path / name
    path.write_text(json.dumps(document), encoding="utf-8")
    return path


def test_the_shipped_file_holds_every_key_with_a_source() -> None:
    tolerances = Tolerances.load()
    assert set(tolerances.regions) == KEYS
    assert all(region.source.strip() for region in tolerances.regions.values())
    assert tolerances.bound("floor.sink.foot") == 5.0
    assert len(tolerances.sha256) == 64


def test_a_file_with_a_missing_source_is_refused_by_name(tmp_path: Path) -> None:
    document = _shipped()
    regions: Regions = document["regions"]
    del regions["item.rest.floor"]["source"]
    with pytest.raises(ToleranceError, match=r"item\.rest\.floor has no 'source'"):
        Tolerances.load(_write(tmp_path, "bad.json", document))


def _unknown_key(r: Regions) -> None:
    r["garment.poke.typo"] = r["garment.poke.body"]


def _missing_key(r: Regions) -> None:
    del r["self.hand_in_trunk"]


def _zero_bound(r: Regions) -> None:
    r["item.overlap.parts"]["bound_mm"] = 0


def _unknown_status(r: Regions) -> None:
    r["item.overlap.parts"]["status"] = "guessed"


@pytest.mark.parametrize(
    ("change", "message"),
    [
        (_unknown_key, "unknown region"),
        (_missing_key, "missing region"),
        (_zero_bound, "not a positive number"),
        (_unknown_status, "is not one of"),
    ],
)
def test_a_malformed_file_is_refused(tmp_path: Path, change: Callable[[Regions], None], message: str) -> None:
    document = _shipped()
    change(document["regions"])
    with pytest.raises(ToleranceError, match=message):
        Tolerances.load(_write(tmp_path, "bad.json", document))


def _pack(bound: float) -> dict[str, object]:
    return {
        "format": 1,
        "regions": {
            "floor.sink.foot": {
                "bound_mm": bound,
                "rationale": "a pack's stricter look",
                "source": "test pack",
                "status": "inconclusive",
            }
        },
    }


def test_a_pack_may_tighten_a_bound_and_may_not_loosen_one(tmp_path: Path) -> None:
    base = Tolerances.load()
    tightened = base.tightened_by(_write(tmp_path, "tight.json", _pack(4)))
    assert tightened.bound("floor.sink.foot") == 4.0
    assert tightened.bound("floor.sink.shin") == base.bound("floor.sink.shin")
    assert tightened.sha256 != base.sha256
    with pytest.raises(ToleranceError, match=r"floor\.sink\.foot 8 mm loosens"):
        base.tightened_by(_write(tmp_path, "loose.json", _pack(8)))
