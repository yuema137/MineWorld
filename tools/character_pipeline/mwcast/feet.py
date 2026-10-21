"""The foot-grounding judge: every planted foot of every body on the ground under it (PR S36-P4 commit 3).

`clients/3d-spike/tools/feet_course.gd` walks every body the 3D client draws over a seeded course (a flat
stretch, a kerb up and down, a ramp, a flight of stairs, an uneven patch) or across the slice street's kerb,
with the runtime foot grounding on (or off, `--no-ik`), and writes one row per body, per frame, per foot to
`feet.json`. This module judges those rows with the bounds of the tolerance file (I-S36-2), never a literal:

- a **planted** foot (the probe's planted flag) **FAILs** when its lowest point is more than
  `floor.sink.foot` under the ground beneath it, or more than `floor.float.planted` above it;
- a swinging foot is not judged (it leaves the ground on purpose);
- a frame the probe flags `unreached` (a step higher than `max_step_m`, which no leg holds in place) is
  listed, not judged.

Every body, foot and course segment prints its worst sink and float and the margin to its bound. `run` stages
nothing: the probe runs in the client project itself (`--path clients/3d-spike`), which the nightly's slice
probes have imported already.

Standard library only.
"""

from __future__ import annotations

import json
from collections.abc import Mapping, Sequence
from dataclasses import dataclass, field
from pathlib import Path
from typing import cast

from mwcast import pose_sheet
from mwcast.tolerances import Tolerances

FORMAT = 1
SIDES = ("left", "right")
PROBE = "res://tools/feet_course.gd"
# A planted run lasts at least this many frames (0.1 s at 30 fps): S36-P2's reading of G-8
# (pr-s36-p2-gait-contact.md bounded correction 2), so a foot passing the drift rule for a frame in
# mid-swing is not judged as planted.
MIN_RUN = 3
IMPORT_TIMEOUT_S = 3600
PROBE_TIMEOUT_S = 1800


class FeetError(Exception):
    """The probe wrote nothing, or its file is malformed. The message names the field."""


@dataclass(frozen=True)
class Failure:
    body: str
    foot: str
    segment: str
    frame: int
    height_mm: float
    why: str

    def line(self) -> str:
        return (
            f"FAIL {self.body} {self.foot} foot, segment {self.segment}, frame {self.frame}: "
            f"{self.height_mm:+.1f} mm ({self.why})"
        )


@dataclass
class Extent:
    """The worst planted sink (most negative) and float (most positive) of one body, foot and segment."""

    sink_mm: float = 0.0
    float_mm: float = 0.0
    planted: int = 0
    unreached: int = 0
    # the least contact-point drift as a share of the body's speed, over the moving frames (-1: none moved)
    min_drift_share: float = -1.0

    def add(self, height_mm: float) -> None:
        self.planted += 1
        self.sink_mm = min(self.sink_mm, height_mm)
        self.float_mm = max(self.float_mm, height_mm)


@dataclass
class Verdict:
    sink_bound_mm: float
    float_bound_mm: float
    failures: list[Failure] = field(default_factory=lambda: list[Failure]())
    unreached: list[str] = field(default_factory=lambda: list[str]())
    extents: dict[tuple[str, str, str], Extent] = field(
        default_factory=lambda: dict[tuple[str, str, str], Extent]()
    )
    frames: int = 0

    @property
    def passed(self) -> bool:
        return not self.failures and self.frames > 0

    def table(self) -> str:
        lines = [
            f"bounds: sink {self.sink_bound_mm:.1f} mm (floor.sink.foot), "
            f"float {self.float_bound_mm:.1f} mm (floor.float.planted)",
            f"{'body':<20}{'foot':<7}{'segment':<14}{'planted':>8}{'sink':>9}{'margin':>8}"
            f"{'float':>9}{'margin':>8}{'unreached':>11}{'drift':>7}",
        ]
        for (body, foot, segment), e in sorted(self.extents.items()):
            lines.append(
                f"{body:<20}{foot:<7}{segment:<14}{e.planted:>8}{e.sink_mm:>9.1f}"
                f"{self.sink_bound_mm + e.sink_mm:>8.1f}{e.float_mm:>9.1f}"
                f"{self.float_bound_mm - e.float_mm:>8.1f}{e.unreached:>11}"
                + (f"{e.min_drift_share:>7.2f}" if e.min_drift_share >= 0.0 else f"{'-':>7}")
            )
        return "\n".join(lines)


def _number(row: Mapping[str, object], key: str, where: str) -> float:
    value = row.get(key)
    if isinstance(value, bool) or not isinstance(value, int | float):
        raise FeetError(f"{where}: {key!r} is not a number")
    return float(value)


def judge(report: Mapping[str, object], tolerances: Tolerances) -> Verdict:
    """Judge a `feet.json` document. Raises `FeetError` naming the first malformed field."""
    if report.get("format") != FORMAT:
        raise FeetError(f"feet.json: format {report.get('format')!r}, expected {FORMAT}")
    verdict = Verdict(tolerances.bound("floor.sink.foot"), tolerances.bound("floor.float.planted"))
    bodies = report.get("bodies")
    if not isinstance(bodies, list) or not bodies:
        raise FeetError("feet.json: 'bodies' is not a non-empty list")
    for body_row in cast(list[object], bodies):
        if not isinstance(body_row, dict):
            raise FeetError("feet.json: a body is not an object")
        body = cast(dict[str, object], body_row)
        name = str(body.get("id", "?"))
        frames = body.get("frames")
        if not isinstance(frames, list):
            raise FeetError(f"{name}: 'frames' is not a list")
        rows: list[tuple[int, str, dict[str, dict[str, object]]]] = []
        for index, frame_row in enumerate(cast(list[object], frames)):
            if not isinstance(frame_row, dict):
                raise FeetError(f"{name} frame {index}: not an object")
            frame = cast(dict[str, object], frame_row)
            number = int(_number(frame, "frame", f"{name} frame {index}"))
            feet = frame.get("feet")
            if not isinstance(feet, dict):
                raise FeetError(f"{name} frame {number}: 'feet' is not an object")
            sides: dict[str, dict[str, object]] = {}
            for side in SIDES:
                foot_row = cast(dict[str, object], feet).get(side)
                if not isinstance(foot_row, dict):
                    raise FeetError(f"{name} frame {number}: no {side} foot")
                sides[side] = cast(dict[str, object], foot_row)
            rows.append((number, str(frame.get("segment", "?")), sides))
        verdict.frames += len(rows)
        for side in SIDES:
            planted = _runs([(seg, bool(feet[side].get("planted", False))) for _, seg, feet in rows])
            for (number, segment, feet), is_planted in zip(rows, planted, strict=True):
                _judge_foot(verdict, name, side, number, segment, feet[side], is_planted)
    return verdict


def _runs(flags: Sequence[tuple[str, bool]]) -> list[bool]:
    """Each frame's planted flag, kept only inside a run of at least `MIN_RUN` planted frames of one
    segment."""
    out = [False] * len(flags)
    start = 0
    while start < len(flags):
        end = start
        while end < len(flags) and flags[end][1] and flags[end][0] == flags[start][0]:
            end += 1
        if end - start >= MIN_RUN:
            out[start:end] = [True] * (end - start)
        start = max(end, start + 1)
    return out


def _judge_foot(
    verdict: Verdict,
    name: str,
    side: str,
    number: int,
    segment: str,
    foot: Mapping[str, object],
    planted: bool,
) -> None:
    where = f"{name} frame {number} {side}"
    height = _number(foot, "height_mm", where)
    extent = verdict.extents.setdefault((name, side, segment), Extent())
    drift, body = foot.get("drift_mps"), foot.get("body_mps")
    if isinstance(drift, int | float) and isinstance(body, int | float) and drift >= 0.0 and body > 0.0:
        share = float(drift) / float(body)
        extent.min_drift_share = share if extent.min_drift_share < 0.0 else min(extent.min_drift_share, share)
    if not planted:
        return
    if bool(foot.get("unreached", False)):
        extent.unreached += 1
        verdict.unreached.append(f"{where}, segment {segment}: unreached ({height:+.1f} mm)")
        return
    extent.add(height)
    if height < -verdict.sink_bound_mm:
        why = f"sinks past {verdict.sink_bound_mm:.1f} mm"
        verdict.failures.append(Failure(name, side, segment, number, height, why))
    elif height > verdict.float_bound_mm:
        why = f"floats past {verdict.float_bound_mm:.1f} mm"
        verdict.failures.append(Failure(name, side, segment, number, height, why))


def load(path: Path) -> dict[str, object]:
    try:
        document: object = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        raise FeetError(f"{path}: {exc}") from exc
    if not isinstance(document, dict):
        raise FeetError(f"{path}: not a JSON object")
    return cast(dict[str, object], document)


def probe_command(
    godot: str, project: Path, bodies: Path, out: Path, seed: int, no_ik: bool, scene: str
) -> list[str]:
    command = [
        godot,
        "--headless",
        "--path",
        str(project),
        "--fixed-fps",
        "30",
        "--script",
        PROBE,
        "--",
        "--bodies",
        str(bodies.resolve()),
        "--seed",
        str(seed),
        "--out",
        str(out.resolve()),
        "--scene",
        scene,
    ]
    if no_ik:
        command.append("--no-ik")
    return command


def run(
    repo_root: Path,
    out: Path,
    listing: Sequence[Mapping[str, object]],
    environ: Mapping[str, str],
    seed: int,
    no_ik: bool,
    scene: str,
) -> dict[str, object]:
    """Import the client project (a no-op when it is current) and run the probe; return its `feet.json`."""
    godot = pose_sheet.find_godot(environ)
    project = repo_root / pose_sheet.CLIENT
    out.mkdir(parents=True, exist_ok=True)
    bodies = out / "bodies.json"
    bodies.write_text(json.dumps(list(listing), indent=2) + "\n", encoding="utf-8")
    result = out / "feet.json"
    result.unlink(missing_ok=True)
    pose_sheet.run_logged(
        [godot, "--headless", "--path", str(project), "--import"], out / "import.log", IMPORT_TIMEOUT_S
    )
    pose_sheet.run_logged(
        probe_command(godot, project, bodies, out, seed, no_ik, scene), out / "probe.log", PROBE_TIMEOUT_S
    )
    if not result.exists():
        raise FeetError(f"Godot wrote no {result}; see {out / 'probe.log'}")
    return load(result)


def self_test(repo_root: Path, out: Path, environ: Mapping[str, str]) -> tuple[bool, list[str]]:
    """The probe's commit-1 and commit-2 checks (`feet_course.gd --self-test`): its verdict lines and whether
    every check passed. The verdict is read from what the probe printed and wrote, never from its exit
    status."""
    godot = pose_sheet.find_godot(environ)
    project = repo_root / pose_sheet.CLIENT
    out.mkdir(parents=True, exist_ok=True)
    (out / "selftest.json").unlink(missing_ok=True)
    pose_sheet.run_logged(
        [godot, "--headless", "--path", str(project), "--import"], out / "import.log", IMPORT_TIMEOUT_S
    )
    log = out / "selftest.log"
    pose_sheet.run_logged(
        [
            godot,
            "--headless",
            "--path",
            str(project),
            "--fixed-fps",
            "30",
            "--script",
            PROBE,
            "--",
            "--self-test",
            "--out",
            str(out.resolve()),
        ],
        log,
        PROBE_TIMEOUT_S,
    )
    lines = log.read_text(encoding="utf-8", errors="replace").splitlines() if log.exists() else []
    verdicts = [line for line in lines if line.startswith(("PASS ", "FAIL ", "feet_course: cost"))]
    passed = "all feet checks pass" in lines and (out / "selftest.json").exists()
    return passed, verdicts or ["the self-test printed no verdict; see " + str(log)]
