"""The character records: `fit.yaml` and `generation_log.yaml` (PIPELINE.md §§4–5).

Records are written in the JSON-compatible subset of YAML and parsed with `json`. Every key is checked:
an unknown key or a wrong type is refused with its key path. Typed results are frozen dataclasses.
The defaults table is PIPELINE.md §4.1 verbatim: an override is a value that differs from its default.
"""

from __future__ import annotations

import json
import re
from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path
from typing import Literal, cast

FORMAT_VERSION = 1


class RecordError(Exception):
    """A record is not valid. The message names the file and the key path (or line and column)."""


def load_record(path: Path) -> object:
    """Read a record as UTF-8 and parse it. A JSON error is re-raised as `RecordError` with its position."""
    text = path.read_text(encoding="utf-8")
    try:
        return cast(object, json.loads(text))
    except json.JSONDecodeError as exc:
        raise RecordError(f"{path}: line {exc.lineno}, column {exc.colno}: {exc.msg}") from exc


class _Section:
    """Reads one object of a record, checking types and ranges, and refusing keys nobody reads."""

    def __init__(self, raw: object, path: str, source: Path) -> None:
        if not isinstance(raw, dict):
            raise RecordError(f"{source}: {path}: expected an object")
        self._raw = cast(dict[str, object], raw)
        self._path = path
        self._source = source
        self._read: set[str] = set()

    def _where(self, key: str) -> str:
        return f"{self._path}.{key}" if self._path else key

    def _take(self, key: str) -> object:
        self._read.add(key)
        if key not in self._raw:
            raise RecordError(f"{self._source}: {self._where(key)}: missing")
        return self._raw[key]

    def _fail(self, key: str, expected: str) -> RecordError:
        return RecordError(f"{self._source}: {self._where(key)}: expected {expected}")

    def number(
        self, key: str, *, low: float | None = None, high: float | None = None, positive: bool = False
    ) -> float:
        value = self._take(key)
        if isinstance(value, bool) or not isinstance(value, int | float):
            raise self._fail(key, "a number")
        if positive and value <= 0:
            raise self._fail(key, "a number greater than 0")
        if low is not None and value < low:
            raise self._fail(key, f"a number at least {low}")
        if high is not None and value > high:
            raise self._fail(key, f"a number at most {high}")
        return float(value)

    def integer(self, key: str, *, low: int | None = None, high: int | None = None) -> int:
        value = self._take(key)
        if isinstance(value, bool) or not isinstance(value, int):
            raise self._fail(key, "an integer")
        if low is not None and value < low:
            raise self._fail(key, f"an integer at least {low}")
        if high is not None and value > high:
            raise self._fail(key, f"an integer at most {high}")
        return value

    def text(self, key: str, *, pattern: str | None = None) -> str:
        value = self._take(key)
        if not isinstance(value, str):
            raise self._fail(key, "a string")
        if pattern is not None and re.fullmatch(pattern, value) is None:
            raise self._fail(key, f"a string matching {pattern}")
        return value

    def boolean(self, key: str) -> bool:
        value = self._take(key)
        if not isinstance(value, bool):
            raise self._fail(key, "true or false")
        return value

    def choice(self, key: str, allowed: tuple[str, ...]) -> str:
        value = self._take(key)
        if not isinstance(value, str) or value not in allowed:
            raise self._fail(key, f"one of {', '.join(allowed)}")
        return value

    def numbers(self, key: str, length: int) -> tuple[float, ...]:
        items = self.items(key)
        if len(items) != length:
            raise self._fail(key, f"a list of {length} numbers")
        out: list[float] = []
        for item in items:
            if isinstance(item, bool) or not isinstance(item, int | float):
                raise self._fail(key, f"a list of {length} numbers")
            out.append(float(item))
        return tuple(out)

    def pixel(self, key: str) -> tuple[int, int]:
        values = self.numbers(key, 2)
        if any(v != int(v) for v in values):
            raise self._fail(key, "two integers")
        return int(values[0]), int(values[1])

    def pixels_box(self, key: str) -> tuple[int, int, int, int]:
        values = self.numbers(key, 4)
        if any(v != int(v) for v in values):
            raise self._fail(key, "four integers")
        return int(values[0]), int(values[1]), int(values[2]), int(values[3])

    def section(self, key: str) -> _Section:
        return _Section(self._take(key), self._where(key), self._source)

    def items(self, key: str) -> list[object]:
        value = self._take(key)
        if not isinstance(value, list):
            raise RecordError(f"{self._source}: {self._where(key)}: expected a list")
        return cast(list[object], value)

    def optional_box(self, key: str) -> tuple[int, int, int, int] | None:
        if key not in self._raw:
            self._read.add(key)
            return None
        return self.pixels_box(key)

    def done(self) -> None:
        extra = sorted(set(self._raw) - self._read - {k for k in self._raw if k.startswith("_")})
        if extra:
            raise RecordError(f"{self._source}: {self._where(extra[0])}: unknown key")


@dataclass(frozen=True)
class Reference:
    path: str
    sha256: str
    generator: str


@dataclass(frozen=True)
class Source:
    meshy_task: str
    download_file: str
    download_sha256: str
    input_sha256: str
    reference: Reference


@dataclass(frozen=True)
class Mask:
    session: str
    crop_box: tuple[int, int, int, int] | None
    fg_ratio: float
    size_px: int
    alpha_threshold: int


@dataclass(frozen=True)
class Bake:
    size_px: int
    samples: int
    margin_px: int
    extrusion_m: float
    ray_m: float
    device: str


@dataclass(frozen=True)
class GameReady:
    target_triangles: int
    eye_z_raw: float
    centre_band_z_raw: float
    centre_band_half_raw: float
    sole_z_m: float
    eye_from_sole_m: float
    weld_m: float
    bake: Bake


@dataclass(frozen=True)
class Fit:
    rig_source: str
    charmorph_front_y_at_hips: float
    charmorph_landmarks_z: tuple[float, ...]
    frame: str
    view_px: int
    px_per_raw_unit: int
    picks: Mapping[str, tuple[int, int]]


@dataclass(frozen=True)
class HeadRigid:
    above_chin_m: float
    half_width_m: float


@dataclass(frozen=True)
class PackCore:
    behind_spine1_m: float
    above_hips_m: float
    above_spine2_m: float
    half_width_m: float


@dataclass(frozen=True)
class Smooth:
    factor: float
    repeat: int
    limit: int


@dataclass(frozen=True)
class Fingers:
    hand_share_min: float
    power: float
    keep: int


@dataclass(frozen=True)
class PackPanels:
    x_in: float
    x_out: float
    sleeve_share_max: float
    arm_zero: bool
    smooth: bool
    smooth_repeat: int


@dataclass(frozen=True)
class Weights:
    head_rigid: HeadRigid
    pack_core: PackCore
    smooth: Smooth
    fingers: Fingers
    pack_panels: PackPanels


@dataclass(frozen=True)
class Flecks:
    enabled: bool
    z_min_m: float
    dark: float
    lighter: float


@dataclass(frozen=True)
class Export:
    material: str


@dataclass(frozen=True)
class FitParams:
    """A validated `fit.yaml`. Every value is typed; every key is read."""

    character: str
    stature_m: float
    accessories: tuple[str, ...]
    source: Source
    mask: Mask
    gameready: GameReady
    fit: Fit
    weights: Weights
    flecks: Flecks
    export: Export


_SLUG = r"[a-z0-9-]+"
_SHA256 = r"[0-9a-f]{64}"
_UUID = r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}"
_PICK_KEYS: tuple[str, ...] = (
    "r_shoulder",
    "l_shoulder",
    "r_elbow",
    "l_elbow",
    "r_wrist",
    "l_wrist",
    "r_handtip",
    "l_handtip",
    "r_hip",
    "l_hip",
    "r_knee",
    "l_knee",
    "r_ankle",
    "l_ankle",
    "chin",
    "eye",
)


def parse_fit(raw: object, source: Path) -> FitParams:
    """Validate a `fit.yaml` object. Refuses a wrong format version, an unknown key, a wrong type or a range
    violation, naming the key path."""
    top = _Section(raw, "", source)
    if top.integer("format") != FORMAT_VERSION:
        raise RecordError(f"{source}: format: expected {FORMAT_VERSION}")
    character = top.text("character", pattern=_SLUG)
    stature_m = top.number("stature_m", positive=True)
    accessories: list[str] = []
    for item in top.items("accessories"):
        if item != "backpack":
            raise RecordError(f'{source}: accessories: expected a list of "backpack" (got {item!r})')
        accessories.append("backpack")
    src = top.section("source")
    source_model = _parse_source(src, source)
    src.done()
    mask_s = top.section("mask")
    mask = Mask(
        session=mask_s.text("session"),
        crop_box=mask_s.optional_box("crop_box"),
        fg_ratio=mask_s.number("fg_ratio", low=0.0, high=1.0, positive=True),
        size_px=mask_s.integer("size_px", low=1),
        alpha_threshold=mask_s.integer("alpha_threshold", low=0, high=255),
    )
    mask_s.done()
    gameready = _parse_gameready(top.section("gameready"), source)
    fit_s = top.section("fit")
    picks_s = fit_s.section("picks")
    picks: dict[str, tuple[int, int]] = {}
    fit_model = Fit(
        rig_source=fit_s.text("rig_source"),
        charmorph_front_y_at_hips=fit_s.number("charmorph_front_y_at_hips"),
        charmorph_landmarks_z=fit_s.numbers("charmorph_landmarks_z", 7),
        frame=picks_s.text("frame"),
        view_px=picks_s.integer("view_px", low=1),
        px_per_raw_unit=picks_s.integer("px_per_raw_unit", low=1),
        picks=picks,
    )
    for key in _PICK_KEYS:
        picks[key] = picks_s.pixel(key)
    picks_s.done()
    fit_s.done()
    weights_s = top.section("weights")
    weights = _parse_weights(weights_s, source)
    weights_s.done()
    flecks_s = top.section("flecks")
    flecks = Flecks(
        enabled=flecks_s.boolean("enabled"),
        z_min_m=flecks_s.number("z_min_m"),
        dark=flecks_s.number("dark"),
        lighter=flecks_s.number("lighter"),
    )
    flecks_s.done()
    export_s = top.section("export")
    export = Export(material=export_s.text("material"))
    export_s.done()
    top.done()
    return FitParams(
        character=character,
        stature_m=stature_m,
        accessories=tuple(accessories),
        source=source_model,
        mask=mask,
        gameready=gameready,
        fit=fit_model,
        weights=weights,
        flecks=flecks,
        export=export,
    )


def _parse_source(s: _Section, source: Path) -> Source:
    meshy_task = s.text("meshy_task", pattern=_UUID)
    download_file = s.text("download_file")
    download_sha256 = s.text("download_sha256", pattern=_SHA256)
    input_sha256 = s.text("input_sha256", pattern=_SHA256)
    ref = s.section("reference")
    reference = Reference(
        path=ref.text("path"),
        sha256=ref.text("sha256", pattern=_SHA256),
        generator=ref.text("generator"),
    )
    ref.done()
    return Source(meshy_task, download_file, download_sha256, input_sha256, reference)


def _parse_gameready(s: _Section, source: Path) -> GameReady:
    gameready = GameReady(
        target_triangles=s.integer("target_triangles", low=1),
        eye_z_raw=s.number("eye_z_raw"),
        centre_band_z_raw=s.number("centre_band_z_raw"),
        centre_band_half_raw=s.number("centre_band_half_raw"),
        sole_z_m=s.number("sole_z_m"),
        eye_from_sole_m=s.number("eye_from_sole_m"),
        weld_m=s.number("weld_m", low=0.0),
        bake=_parse_bake(s.section("bake"), source),
    )
    s.done()
    return gameready


def _parse_bake(s: _Section, source: Path) -> Bake:
    bake = Bake(
        size_px=s.integer("size_px", low=1),
        samples=s.integer("samples", low=1),
        margin_px=s.integer("margin_px", low=0),
        extrusion_m=s.number("extrusion_m", low=0.0),
        ray_m=s.number("ray_m", low=0.0),
        device=s.choice("device", ("auto", "metal", "cuda", "optix", "hip", "cpu")),
    )
    s.done()
    return bake


def _parse_weights(s: _Section, source: Path) -> Weights:
    head = s.section("head_rigid")
    head_rigid = HeadRigid(head.number("above_chin_m"), head.number("half_width_m"))
    head.done()
    core = s.section("pack_core")
    pack_core = PackCore(
        behind_spine1_m=core.number("behind_spine1_m"),
        above_hips_m=core.number("above_hips_m"),
        above_spine2_m=core.number("above_spine2_m"),
        half_width_m=core.number("half_width_m"),
    )
    core.done()
    smooth_s = s.section("smooth")
    smooth = Smooth(
        smooth_s.number("factor"), smooth_s.integer("repeat", low=0), smooth_s.integer("limit", low=0)
    )
    smooth_s.done()
    fingers_s = s.section("fingers")
    fingers = Fingers(
        hand_share_min=fingers_s.number("hand_share_min"),
        power=fingers_s.number("power"),
        keep=fingers_s.integer("keep", low=1),
    )
    fingers_s.done()
    panels_s = s.section("pack_panels")
    panels = PackPanels(
        x_in=panels_s.number("x_in"),
        x_out=panels_s.number("x_out"),
        sleeve_share_max=panels_s.number("sleeve_share_max"),
        arm_zero=panels_s.boolean("arm_zero"),
        smooth=panels_s.boolean("smooth"),
        smooth_repeat=panels_s.integer("smooth_repeat", low=0),
    )
    panels_s.done()
    return Weights(head_rigid, pack_core, smooth, fingers, panels)


@dataclass(frozen=True)
class RunParams:
    """How a Meshy run was requested (D+ §3): raw, or quad remesh to a target polycount."""

    mode: Literal["raw", "remesh"]
    target_polycount: int | None


@dataclass(frozen=True)
class GenerationRun:
    run: str
    task_id: str
    task_id_complete: bool
    params: RunParams
    credits: int
    verdict: Literal["rejected", "kept"]
    reason: str
    date: str


@dataclass(frozen=True)
class GenerationLog:
    runs: tuple[GenerationRun, ...]

    @property
    def total_credits(self) -> int:
        return sum(run.credits for run in self.runs)

    @property
    def kept(self) -> int:
        return sum(1 for run in self.runs if run.verdict == "kept")


def parse_generation_log(raw: object, source: Path) -> GenerationLog:
    """Validate a `generation_log.yaml` object: a `runs` list of entries with the keys of PIPELINE.md §5."""
    top = _Section(raw, "", source)
    runs: list[GenerationRun] = []
    for index, item in enumerate(top.items("runs")):
        entry = _Section(item, f"runs[{index}]", source)
        params_s = entry.section("params")
        mode = params_s.choice("mode", ("raw", "remesh"))
        target = params_s.integer("target_polycount", low=1) if mode == "remesh" else None
        params_s.done()
        runs.append(
            GenerationRun(
                run=entry.text("run"),
                task_id=entry.text("task_id"),
                task_id_complete=entry.boolean("task_id_complete"),
                params=RunParams(mode=cast(Literal["raw", "remesh"], mode), target_polycount=target),
                credits=entry.integer("credits", low=0),
                verdict=cast(Literal["rejected", "kept"], entry.choice("verdict", ("rejected", "kept"))),
                reason=entry.text("reason"),
                date=entry.text("date", pattern=r"\d{4}-\d{2}-\d{2}"),
            )
        )
        entry.done()
    top.done()
    return GenerationLog(runs=tuple(runs))


# PIPELINE.md §4.1: the defaults, as dotted keys. An override is a value that differs from its default.
# Keys in OVERRIDE_EXEMPT describe the character and are not counted (PIPELINE.md §4.2).
DEFAULTS: dict[str, object] = {
    "mask.session": "birefnet-portrait",
    "mask.crop_box": (230, 40, 860, 1086),
    "mask.fg_ratio": 0.85,
    "mask.size_px": 1024,
    "mask.alpha_threshold": 16,
    "gameready.target_triangles": 95000,
    "gameready.eye_z_raw": 0.779,
    "gameready.centre_band_z_raw": 0.05,
    "gameready.centre_band_half_raw": 0.03,
    "gameready.sole_z_m": -0.0177,
    "gameready.eye_from_sole_m": 1.655,
    "gameready.weld_m": 1e-06,
    "gameready.bake.size_px": 2048,
    "gameready.bake.samples": 1,
    "gameready.bake.margin_px": 8,
    "gameready.bake.extrusion_m": 0.015,
    "gameready.bake.ray_m": 0.04,
    "gameready.bake.device": "auto",
    "fit.rig_source": "clients/3d-spike/assets/characters/vitruvian/vitruvian.glb",
    "fit.charmorph_front_y_at_hips": -0.045356,
    "fit.charmorph_landmarks_z": (-0.0177, 0.083, 0.51, 0.947, 1.401, 1.531, 1.637),
    "fit.frame": "dplus-v1",
    "fit.view_px": 1024,
    "fit.px_per_raw_unit": 512,
    "weights.head_rigid.above_chin_m": 0.01,
    "weights.head_rigid.half_width_m": 0.16,
    "weights.pack_core.behind_spine1_m": 0.075,
    "weights.pack_core.above_hips_m": 0.02,
    "weights.pack_core.above_spine2_m": 0.10,
    "weights.pack_core.half_width_m": 0.12,
    "weights.smooth.factor": 0.5,
    "weights.smooth.repeat": 4,
    "weights.smooth.limit": 4,
    "weights.fingers.hand_share_min": 0.95,
    "weights.fingers.power": 6.0,
    "weights.fingers.keep": 2,
    "weights.pack_panels.x_in": 0.12,
    "weights.pack_panels.x_out": 0.21,
    "weights.pack_panels.sleeve_share_max": 0.5,
    "weights.pack_panels.arm_zero": True,
    "weights.pack_panels.smooth": True,
    "weights.pack_panels.smooth_repeat": 8,
    "flecks.enabled": True,
    "flecks.z_min_m": 1.65,
    "flecks.dark": 0.30,
    "flecks.lighter": 0.12,
    "export.material": "MW_Cast",
}
OVERRIDE_EXEMPT: tuple[str, ...] = ("character", "stature_m", "accessories", "source.", "fit.picks")


def flatten(fit: FitParams) -> dict[str, object]:
    """The defaultable values of a record, keyed as in `DEFAULTS`."""
    g = fit.gameready
    b = g.bake
    w = fit.weights
    values: dict[str, object] = {
        "mask.session": fit.mask.session,
        "mask.crop_box": fit.mask.crop_box,
        "mask.fg_ratio": fit.mask.fg_ratio,
        "mask.size_px": fit.mask.size_px,
        "mask.alpha_threshold": fit.mask.alpha_threshold,
        "gameready.target_triangles": g.target_triangles,
        "gameready.eye_z_raw": g.eye_z_raw,
        "gameready.centre_band_z_raw": g.centre_band_z_raw,
        "gameready.centre_band_half_raw": g.centre_band_half_raw,
        "gameready.sole_z_m": g.sole_z_m,
        "gameready.eye_from_sole_m": g.eye_from_sole_m,
        "gameready.weld_m": g.weld_m,
        "gameready.bake.size_px": b.size_px,
        "gameready.bake.samples": b.samples,
        "gameready.bake.margin_px": b.margin_px,
        "gameready.bake.extrusion_m": b.extrusion_m,
        "gameready.bake.ray_m": b.ray_m,
        "gameready.bake.device": b.device,
        "fit.rig_source": fit.fit.rig_source,
        "fit.charmorph_front_y_at_hips": fit.fit.charmorph_front_y_at_hips,
        "fit.charmorph_landmarks_z": fit.fit.charmorph_landmarks_z,
        "fit.frame": fit.fit.frame,
        "fit.view_px": fit.fit.view_px,
        "fit.px_per_raw_unit": fit.fit.px_per_raw_unit,
        "weights.head_rigid.above_chin_m": w.head_rigid.above_chin_m,
        "weights.head_rigid.half_width_m": w.head_rigid.half_width_m,
        "weights.pack_core.behind_spine1_m": w.pack_core.behind_spine1_m,
        "weights.pack_core.above_hips_m": w.pack_core.above_hips_m,
        "weights.pack_core.above_spine2_m": w.pack_core.above_spine2_m,
        "weights.pack_core.half_width_m": w.pack_core.half_width_m,
        "weights.smooth.factor": w.smooth.factor,
        "weights.smooth.repeat": w.smooth.repeat,
        "weights.smooth.limit": w.smooth.limit,
        "weights.fingers.hand_share_min": w.fingers.hand_share_min,
        "weights.fingers.power": w.fingers.power,
        "weights.fingers.keep": w.fingers.keep,
        "weights.pack_panels.x_in": w.pack_panels.x_in,
        "weights.pack_panels.x_out": w.pack_panels.x_out,
        "weights.pack_panels.sleeve_share_max": w.pack_panels.sleeve_share_max,
        "weights.pack_panels.arm_zero": w.pack_panels.arm_zero,
        "weights.pack_panels.smooth": w.pack_panels.smooth,
        "weights.pack_panels.smooth_repeat": w.pack_panels.smooth_repeat,
        "flecks.enabled": fit.flecks.enabled,
        "flecks.z_min_m": fit.flecks.z_min_m,
        "flecks.dark": fit.flecks.dark,
        "flecks.lighter": fit.flecks.lighter,
        "export.material": fit.export.material,
    }
    return values


def overrides(fit: FitParams) -> list[str]:
    """The dotted keys whose value differs from `DEFAULTS`, sorted. `OVERRIDE_EXEMPT` keys are not counted."""
    found: list[str] = []
    for key, value in flatten(fit).items():
        if key.startswith(OVERRIDE_EXEMPT):
            continue
        if DEFAULTS.get(key) != value:
            found.append(key)
    return sorted(found)
