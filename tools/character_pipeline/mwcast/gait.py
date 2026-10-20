"""The gait gates over the dumped frames (step-36 §4.2; pr-s36-p2-gait-contact.md).

- **G-4w** sole vs floor: per foot, per frame of rest, the `Stand` loop and one `Walk` and one `Jog_Fwd`
  cycle, with "planted" defined by the gait itself.
- **G-9w** arm vs torso: per full frame, each limb's query vertices against the trunk patch by winding
  number (`contact_gates.limb_depths`, the same filters as C-1).

Godot skinned every frame (`godot/pose_sheet.gd --dump-frames`, I-S36-3); this module only decides. Every
bound is a key of the tolerance file (I-S36-2). A gate that cannot look fails with the reason (I-S36-4). No
gate reads a body's name (I-S36-5): sides and limbs come from the skeleton's bone names.

**Planted** (G-8's rule, on the treadmill). The body stays put while the floor would move backward at the
gait speed `v`. A foot is planted in frame *k* when its `Foot` bone's horizontal velocity (a central
difference over the cyclic frames, × 30 fps) satisfies `|vel · fwd + v| ≤ 0.25 v` and `|vel × fwd| ≤ 0.25 v`.
Standing, every frame is planted for both feet.

Example. A heel that sinks 7 mm at heel strike in walk frame 3 fails as "G-4w left foot, walk frame 3,
planted, sink 7.0 mm (bound 5.0 mm, floor.sink.foot)"; a right foot hovering 12 mm over frames 18–24 while
planted fails as "G-4w right foot, walk frames 18–24, planted, float 12.0 mm (bound 10.0 mm,
floor.float.planted)".
"""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from dataclasses import dataclass

import numpy as np
from numpy.typing import NDArray

from mwcast import contact_gates
from mwcast.contact import ContactUnavailable
from mwcast.contact_gates import LIMB_GROUPS, NoLook, Skinning, bone_group, humanoid
from mwcast.frames import Frame, Frames
from mwcast.gates import GateId, GateResult, Verdict
from mwcast.tolerances import Tolerances

# The probe's fixed frame rate (`godot --fixed-fps 30`): the frames of a clip are this far apart in time.
FPS = 30.0
# G-8's planted rule (pr-s32p-wardrobe-pilot.md §4.8): a planted foot's drift, as a share of the gait speed.
PLANTED_DRIFT = 0.25
# Below this gait speed the body is standing: every frame is planted.
STANDING_MPS = 1e-3
SIDES = ("left", "right")
CLIPS = ("stand", "walk", "jog")
_MM = 1000.0

Bools = NDArray[np.bool_]
Floats = NDArray[np.float64]


def _side_bone(side: str, bone: str) -> str:
    return ("Left" if side == "left" else "Right") + bone


def _sets(frames: Frames) -> dict[str, NDArray[np.int64]]:
    """Positions into the ground set: each side's foot set (dominant bone that side's `Foot` or `Toes`), shin
    set (`LowerLeg`), and side set (any bone of that side, the lowest point per foot of S26-c-ground)."""
    dominant = frames.dominant()
    names = [humanoid(dominant[int(i)]) for i in frames.ground]
    out: dict[str, NDArray[np.int64]] = {}
    for side in SIDES:
        prefix = _side_bone(side, "")
        foot = {_side_bone(side, "Foot"), _side_bone(side, "Toes")}
        out[f"{side} foot"] = np.asarray([k for k, n in enumerate(names) if n in foot], dtype=np.int64)
        out[f"{side} shin"] = np.asarray(
            [k for k, n in enumerate(names) if n == _side_bone(side, "LowerLeg")], dtype=np.int64
        )
        out[f"{side} side"] = np.asarray(
            [k for k, n in enumerate(names) if n.startswith(prefix)], dtype=np.int64
        )
    return out


def _heights(frames: Frames, clip: Sequence[Frame], subset: NDArray[np.int64] | None) -> Floats:
    """Per frame, the lowest point of the ground set (or of `subset`, positions into it) above the floor,
    in mm."""
    lows: list[float] = []
    for frame in clip:
        ys = frames.ground_positions(frame)[:, 1]
        if subset is not None:
            ys = ys[subset]
        lows.append((float(ys.min()) - frame.floor_y) * _MM if ys.size else float("nan"))
    return np.asarray(lows, dtype=np.float64)


def planted(frames: Frames, clip: str, side: str) -> Bools:
    """Per frame of `clip`, whether the `side` foot is planted (G-8's rule; module docstring)."""
    found = frames.clip(clip)
    if not found:
        return np.zeros(0, dtype=bool)
    v = float(np.mean([f.speed_mps for f in found]))
    if v < STANDING_MPS:
        return np.ones(len(found), dtype=bool)
    bone = frames.bone(_side_bone(side, "Foot"))
    at = np.stack([f.bones[bone] for f in found])
    vel = (np.roll(at, -1, axis=0) - np.roll(at, 1, axis=0)) * (FPS / 2.0)
    fwd = np.asarray(found[0].fwd, dtype=np.float64)
    along = vel[:, 0] * fwd[0] + vel[:, 2] * fwd[1]
    across = vel[:, 0] * fwd[1] - vel[:, 2] * fwd[0]
    limit = PLANTED_DRIFT * v
    return (np.abs(along + v) <= limit) & (np.abs(across) <= limit)


def _runs(indices: Sequence[int]) -> list[tuple[int, int]]:
    runs: list[tuple[int, int]] = []
    for k in indices:
        if runs and k == runs[-1][1] + 1:
            runs[-1] = (runs[-1][0], k)
        else:
            runs.append((k, k))
    return runs


def _frames_text(clip: str, first: int, last: int) -> str:
    return f"{clip} frame {first}" if first == last else f"{clip} frames {first}–{last}"


def _r(value: float) -> float:
    return round(float(value), 1)


@dataclass(frozen=True)
class Ground:
    """G-4w's verdict and the numbers the recorded `ground` block carries (PIPELINE.md §15.1)."""

    result: GateResult
    block: Mapping[str, object]


@dataclass(frozen=True)
class _Bounds:
    sink_foot: float
    sink_shin: float
    float_planted: float
    float_frame: float

    @staticmethod
    def of(tolerances: Tolerances) -> _Bounds:
        return _Bounds(
            tolerances.bound("floor.sink.foot"),
            tolerances.bound("floor.sink.shin"),
            tolerances.bound("floor.float.planted"),
            tolerances.bound("floor.float.walk_frame"),
        )


def _span(clip: str, found: Sequence[Frame], first: int, last: int) -> str:
    return _frames_text(clip, found[first].index, found[last].index)


def _foot_clip(
    frames: Frames, sets: Mapping[str, NDArray[np.int64]], side: str, clip: str, b: _Bounds
) -> tuple[dict[str, object], str, list[str]]:
    """One foot over one clip: its block entry, its looked-at line and its failures (sink, planted float,
    shin sink)."""
    found = frames.clip(clip)
    foot = _heights(frames, found, sets[f"{side} foot"])
    down = planted(frames, clip, side)
    high = float(foot[down].max()) if bool(down.any()) else None
    entry: dict[str, object] = {
        "sink_mm": _r(foot.min()),
        "planted_float_mm": None if high is None else _r(high),
        "planted_share": round(float(down.mean()), 3),
    }
    low = float(foot.min())
    line = (
        f"{side} foot, {clip}: {len(found)} frames, {int(down.sum())} planted; lowest {low:+.1f} mm "
        f"(margin {b.sink_foot + low:.1f} mm, floor.sink.foot); planted highest "
    )
    line += (
        "—"
        if high is None
        else f"{high:+.1f} mm (margin {b.float_planted - high:.1f} mm, floor.float.planted)"
    )
    reasons: list[str] = []
    for first, last in _runs([k for k in range(len(found)) if foot[k] < -b.sink_foot]):
        worst = -float(foot[first : last + 1].min())
        state = "planted" if bool(down[first : last + 1].any()) else "swing"
        reasons.append(
            f"G-4w {side} foot, {_span(clip, found, first, last)}, {state}, sink {worst:.1f} mm "
            f"(bound {b.sink_foot:g} mm, floor.sink.foot)"
        )
    for first, last in _runs([k for k in range(len(found)) if down[k] and foot[k] > b.float_planted]):
        worst = float(foot[first : last + 1].max())
        reasons.append(
            f"G-4w {side} foot, {_span(clip, found, first, last)}, planted, float {worst:.1f} mm "
            f"(bound {b.float_planted:g} mm, floor.float.planted)"
        )
    if sets[f"{side} shin"].size:
        shin = _heights(frames, found, sets[f"{side} shin"])
        entry["shin_low_mm"] = float(shin.min())
        for first, last in _runs([k for k in range(len(found)) if shin[k] < -b.sink_shin]):
            worst = -float(shin[first : last + 1].min())
            reasons.append(
                f"G-4w {side} shin, {_span(clip, found, first, last)}, sink {worst:.1f} mm "
                f"(bound {b.sink_shin:g} mm, floor.sink.shin)"
            )
    return entry, line, reasons


def _frame_floats(lows: Mapping[str, Floats], jog_feet: Sequence[float], b: _Bounds) -> list[str]:
    """S26-c-ground's floating rules, kept: each walk frame's lowest point, the Stand loop's, and each foot's
    lowest point over the jog cycle, at most `floor.float.walk_frame` above the floor."""
    reasons: list[str] = []
    walk = lows["walk"]
    for first, last in _runs([k for k in range(walk.size) if walk[k] > b.float_frame]):
        reasons.append(
            f"G-4w {_frames_text('walk', first, last)}: the lowest point floats "
            f"{float(walk[first : last + 1].max()):.1f} mm "
            f"(bound {b.float_frame:g} mm, floor.float.walk_frame)"
        )
    stand_high = float(lows["stand"].max())
    if stand_high > b.float_frame:
        reasons.append(
            f"G-4w stand: the lowest point floats {stand_high:.1f} mm (bound {b.float_frame:g} mm, "
            "floor.float.walk_frame)"
        )
    for side, low in zip(SIDES, jog_feet, strict=True):
        if low > b.float_frame:
            reasons.append(
                f"G-4w {side} foot, jog: never within {b.float_frame:g} mm of the floor "
                f"(lowest {low:.1f} mm, floor.float.walk_frame)"
            )
    return reasons


def gate_g4w(frames: Frames, tolerances: Tolerances) -> Ground:
    """G-4w over one body's dump: rest, stand, walk and jog frames, per foot. Fails when, for any foot and
    frame, the foot is below −`floor.sink.foot`, the shin below −`floor.sink.shin`, or the foot is planted
    and above `floor.float.planted`; in a walk also when a frame's lowest point is above
    `floor.float.walk_frame` (S26-c-ground's rule, kept with its standing and jogging forms: the Stand loop's
    lowest point, and each foot's lowest point over the jog cycle). G-8's contact share is reported, not
    judged."""
    looked = [f"tolerances sha256 {tolerances.sha256}", str(frames.path)]
    clips = {name: frames.clip(name) for name in ("rest", *CLIPS)}
    missing = [name for name, found in clips.items() if not found]
    if missing:
        return Ground(_cannot_look(GateId.G4W, looked, f"the dump has no {missing} frames"), {})
    sets = _sets(frames)
    empty = [name for name, idx in sets.items() if idx.size == 0 and not name.endswith("shin")]
    if empty:
        return Ground(_cannot_look(GateId.G4W, looked, f"the ground set has no {empty} vertex"), {})
    b = _Bounds.of(tolerances)
    reasons: list[str] = []
    feet: dict[str, dict[str, object]] = {side: {} for side in SIDES}
    shin_lows: list[float] = []
    for side in SIDES:
        rest_foot = float(_heights(frames, clips["rest"], sets[f"{side} foot"]).min())
        if rest_foot < -b.sink_foot:
            reasons.append(
                f"G-4w {side} foot, rest, sink {-rest_foot:.1f} mm "
                f"(bound {b.sink_foot:g} mm, floor.sink.foot)"
            )
        for clip in CLIPS:
            entry, line, failed = _foot_clip(frames, sets, side, clip, b)
            shin = entry.pop("shin_low_mm", None)
            if isinstance(shin, float):
                shin_lows.append(shin)
            feet[side][clip] = entry
            looked.append(line)
            reasons += failed
    lows = {clip: _heights(frames, clips[clip], None) for clip in CLIPS}
    rest_low = float(_heights(frames, clips["rest"], None).min())
    jog_feet = [float(_heights(frames, clips["jog"], sets[f"{side} side"]).min()) for side in SIDES]
    reasons += _frame_floats(lows, jog_feet, b)
    walk_min = int(np.argmin(lows["walk"]))
    block: dict[str, object] = {
        "rest_mm": _r(rest_low),
        "stand": {
            "min_mm": _r(lows["stand"].min()),
            "max_mm": _r(lows["stand"].max()),
            "frames": len(clips["stand"]),
        },
        "walk": {
            "min_mm": _r(lows["walk"].min()),
            "min_at_frame": clips["walk"][walk_min].index,
            "frame_low_max_mm": _r(lows["walk"].max()),
            "frames": len(clips["walk"]),
        },
        "jog": {
            "min_mm": _r(lows["jog"].min()),
            "foot_contact_mm": [_r(v) for v in jog_feet],
            "frames": len(clips["jog"]),
        },
        "worst_sink_mm": _r(min(rest_low, *(float(lows[c].min()) for c in CLIPS))),
        "worst_float_mm": _r(max(float(lows["stand"].max()), float(lows["walk"].max()), *jog_feet)),
        "feet": feet,
        "shin_sink_mm": _r(min(shin_lows)) if shin_lows else None,
    }
    return Ground(_decide(GateId.G4W, looked, reasons), block)


@dataclass(frozen=True)
class LimbFrame:
    """G-9w on one full frame: every limb group's depth, and the frame's verdict."""

    tag: str
    depths: tuple[contact_gates.LimbDepth, ...]
    cannot_look: str | None

    @property
    def passed(self) -> bool:
        return self.cannot_look is None and all(d.passed for d in self.depths)

    @property
    def worst(self) -> contact_gates.LimbDepth | None:
        return max(self.depths, key=lambda d: d.worst_mm, default=None)

    def check(self) -> dict[str, object]:
        """The frame's `checks.torso_clip` in the pose sheet's `report.json`."""
        worst = self.worst
        by_group: dict[str, int] = {}
        for d in self.depths:
            group = d.key.split(" ", 1)[1]
            if d.over:
                by_group[group] = by_group.get(group, 0) + d.over
        return {
            "gate": GateId.G9W.value,
            "pass": self.passed,
            "max_mm": _r(worst.worst_mm) if worst else 0.0,
            "worst_group": worst.key if worst and worst.worst_mm > 0 else "",
            "vertices_over_bound": sum(d.over for d in self.depths),
            "by_group": by_group,
            **({"cannot_look": self.cannot_look} if self.cannot_look else {}),
        }


def _skinning(frames: Frames, positions: Floats) -> Skinning:
    dominant = tuple(humanoid(name) for name in frames.dominant())
    return Skinning(
        positions,
        dominant,
        frames.labels["trunk"].astype(np.float64),
        frames.labels["limb"].astype(np.float64),
    )


def outermost(frames: Frames) -> Bools:
    """The query vertices G-9w may ask about: a limb vertex only on the outermost layer that covers that limb
    (F-S32p-6's rule: the hoodie's sleeve, not the tee's under it). A fused body has one layer: all kept."""
    dominant = [humanoid(name) for name in frames.dominant()]
    layer = frames.labels["layer"].astype(np.int64)
    keys = [
        ("Left" if bone.startswith("Left") else "Right") + " " + bone_group(bone)
        if bone_group(bone) in LIMB_GROUPS
        else ""
        for bone in dominant
    ]
    top: dict[str, int] = {}
    for key, value in zip(keys, layer, strict=True):
        if key:
            top[key] = max(top.get(key, -1), int(value))
    return np.asarray(
        [not key or int(value) >= top[key] for key, value in zip(keys, layer, strict=True)], dtype=bool
    )


def limb_frames(frames: Frames, tolerances: Tolerances, tags: Sequence[str] | None = None) -> list[LimbFrame]:
    """G-9w per full frame (all full frames, or those tagged `tags`). The trunk patch is the union of every
    mesh's trunk faces; a limb is asked about on its outermost layer only."""
    keep = outermost(frames)
    chosen = [f for f in frames.frames if f.full and (tags is None or f.tag in tags)]
    out: list[LimbFrame] = []
    for frame in chosen:
        skin = _skinning(frames, frames.full_positions(frame))
        joints = {humanoid(name): at for name, at in frames.joints(frame).items()}
        try:
            _, depths = contact_gates.limb_depths(skin, frames.faces, joints, tolerances, keep)
        except (NoLook, ContactUnavailable, KeyError) as exc:
            out.append(LimbFrame(frame.tag, (), f"cannot look: {exc}"))
            continue
        out.append(LimbFrame(frame.tag, tuple(depths), None))
    return out


@dataclass(frozen=True)
class Arms:
    """G-9w's verdict over a set of frames, and the block numbers CC-9 re-judges."""

    result: GateResult
    block: Mapping[str, object]


def gate_g9w(frames: Frames, tolerances: Tolerances, clips: Sequence[str] = ("walk", "jog")) -> Arms:
    """G-9w over the full frames of `clips`: each frame's limb groups against the trunk patch. Fails beyond
    `self.limb_in_trunk` (`self.hand_in_trunk` for the hand) in any frame; names the frames, the group and the
    depth."""
    looked = [f"tolerances sha256 {tolerances.sha256}", str(frames.path)]
    tags = [f.tag for c in clips for f in frames.clip(c) if f.full]
    if not tags:
        return Arms(_cannot_look(GateId.G9W, looked, f"the dump has no full {list(clips)} frame"), {})
    per = limb_frames(frames, tolerances, tags)
    reasons: list[str] = []
    worst = {"limb": 0.0, "hand": 0.0}
    worst_at = {"limb": "", "hand": ""}
    failing: dict[str, list[int]] = {}
    for lf in per:
        if lf.cannot_look:
            reasons.append(f"G-9w {lf.tag}: {lf.cannot_look}")
            continue
        for d in lf.depths:
            kind = "hand" if d.key.endswith(" hand") else "limb"
            if d.worst_mm > worst[kind]:
                worst[kind] = d.worst_mm
                worst_at[kind] = f"{lf.tag} {d.key}"
            if not d.passed:
                clip, _, index = lf.tag.partition(":")
                failing.setdefault(f"{clip}|{d.key}|{d.bound_key}|{d.bound_mm:g}", []).append(int(index or 0))
    for key, indices in sorted(failing.items()):
        clip, limb, bound_key, bound = key.split("|")
        depth = max(
            d.worst_mm
            for lf in per
            for d in lf.depths
            if d.key == limb and lf.tag.partition(":")[0] == clip and not d.passed
        )
        for first, last in _runs(sorted(indices)):
            reasons.append(
                f"G-9w {limb}, {_frames_text(clip, first, last)}, {depth:.1f} mm inside the trunk "
                f"(bound {bound} mm, {bound_key})"
            )
    looked.append(f"{len(per)} full frames of {list(clips)}")
    for kind in ("limb", "hand"):
        looked.append(f"worst {kind} {worst[kind]:.1f} mm at {worst_at[kind] or '—'}")
    block = {
        "limb_in_trunk_mm": {kind: _r(worst[kind]) for kind in ("limb", "hand")},
        "limb_in_trunk_at": worst_at,
        "frames": len(per),
    }
    return Arms(_decide(GateId.G9W, looked, reasons), block)


def _cannot_look(gate: GateId, looked: list[str], why: str) -> GateResult:
    return GateResult(gate, Verdict.FAIL, tuple(looked), (f"cannot look: {why}",))


def _decide(gate: GateId, looked: list[str], reasons: list[str]) -> GateResult:
    return GateResult(gate, Verdict.PASS if not reasons else Verdict.FAIL, tuple(looked), tuple(reasons))
