"""W-4w: C-2's layer order on every dumped frame of an outfit (S36-P3 commit 3; it retires W-4's slab
outline).

C-2 (`contact_gates.layer_order`) judges an assembled outfit at rest. W-4w runs the same rule, with the same
bounds (`garment.poke.body`, `garment.poke.layer` in the tolerance file), on every full frame of the pose
sheet's
frame dump (`MWCF`, `mwcast/frames.py`): the rest pose, each captured frame and every walk and jog frame
the dump
carries. So one rule and one bound judge the outfit standing still and in motion.

A frame's meshes come from the dump's labels: each vertex's mesh index names its mesh (`Body`,
`part_<id>`), and
a body vertex's region index is the assembly's region channel. A part vertex takes its nearest body vertex's
region, as in C-2. Godot skins; this module only reads and judges (I-S36-3). It names no outfit and no part.
"""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np

from mwcast.contact import ContactUnavailable
from mwcast.contact_gates import LayerArrays, OutfitManifest, layer_order
from mwcast.frames import Frame, Frames
from mwcast.gates import GateId, GateResult, Verdict
from mwcast.tolerances import Tolerances

BODY = "Body"  # the drawn body (`BodyCheck`, the unmasked one, is the assembly's own reference)


@dataclass(frozen=True)
class FrameVerdict:
    """W-4w on one frame: its tag ("walk:12"), and C-2's result there."""

    tag: str
    result: GateResult

    def check(self) -> dict[str, object]:
        """The frame's row in the pose sheet's `report.json` (`checks.poke_through`)."""
        return {
            "gate": GateId.W4W.value,
            "pass": self.result.verdict is Verdict.PASS,
            "verdict": self.result.verdict.value,
            "reasons": list(self.result.reasons),
            "looked_at": list(self.result.looked_at[1:]),
        }


def _meshes(frames: Frames, positions: np.ndarray, names: list[str]) -> dict[str, LayerArrays]:
    labels = frames.labels
    out: dict[str, LayerArrays] = {}
    for name in names:
        if name not in frames.mesh_names:
            continue
        index = frames.mesh_names.index(name)
        members = np.flatnonzero(labels["mesh"] == index)
        local = np.full(frames.vertex_count, -1, dtype=np.int64)
        local[members] = np.arange(members.size, dtype=np.int64)
        faces = local[frames.faces]
        faces = faces[(faces >= 0).all(axis=1)]
        out[name] = LayerArrays(
            np.ascontiguousarray(positions[members], dtype=np.float64),
            np.ascontiguousarray(faces, dtype=np.int64),
            labels["region"][members].astype(np.int64),
            labels["under"][members].astype(bool),
        )
    return out


def judge_frame(
    frames: Frames, frame: Frame, manifest: OutfitManifest, tolerances: Tolerances
) -> FrameVerdict:
    """C-2 on one full frame. A frame without the body or a manifest part fails "cannot look" (I-S36-4)."""
    looked = [f"tolerances sha256 {tolerances.sha256}"]
    names = [BODY, *manifest.parts]
    meshes = _meshes(frames, frames.full_positions(frame), names)
    missing = sorted(n for n in names if n not in meshes)
    if missing:
        why = f"cannot look: frame {frame.tag} has no mesh {missing}"
        return FrameVerdict(frame.tag, GateResult(GateId.W4W, Verdict.FAIL, tuple(looked), (why,)))
    skin = meshes.pop(BODY)
    try:
        result = layer_order(manifest, tolerances, BODY, skin, meshes, looked)
    except ContactUnavailable as exc:
        why = f"cannot look: {exc}"
        return FrameVerdict(frame.tag, GateResult(GateId.W4W, Verdict.FAIL, tuple(looked), (why,)))
    reasons = tuple(f"{r}, {frame.clip} frame {frame.index}" for r in result.reasons)
    return FrameVerdict(frame.tag, GateResult(GateId.W4W, result.verdict, result.looked_at, reasons))


def frame_verdicts(frames: Frames, manifest: OutfitManifest, tolerances: Tolerances) -> list[FrameVerdict]:
    """W-4w on every full frame of the dump, in the dump's order."""
    return [judge_frame(frames, f, manifest, tolerances) for f in frames.frames if f.full]


def gate_w4w(frames: Frames, manifest: OutfitManifest, tolerances: Tolerances) -> GateResult:
    """W-4w over the dump: PASS only when every full frame passes C-2's layer order; the reasons name part,
    region, depth, bound key, clip and frame ("part_vera-tee pokes 3.4 mm through part_vera-hoodie in torso
    (garment.poke.layer 2 mm), walk frame 12")."""
    per = frame_verdicts(frames, manifest, tolerances)
    looked = [f"tolerances sha256 {tolerances.sha256}", str(frames.path), f"{len(per)} full frames"]
    if not per:
        return GateResult(
            GateId.W4W, Verdict.FAIL, tuple(looked), ("cannot look: the dump has no full frame",)
        )
    failing = [v for v in per if v.result.verdict is not Verdict.PASS]
    looked.append(f"{len(per) - len(failing)} of {len(per)} frames pass")
    reasons = tuple(r for v in failing for r in v.result.reasons)
    return GateResult(GateId.W4W, Verdict.FAIL if failing else Verdict.PASS, tuple(looked), reasons)
