"""The cover query of `assemble` (L-b, D-impl-22): which points lie inside a worn part, just under it.

`blender/mpfb/assemble.py` runs in Blender's Python, which cannot load libigl, so it writes the query to an
`.npz` file and runs this module with the pipeline's own Python:

    python -m mwcast.wardrobe.cover <query.npz> <flags.npy>

The query holds `points` (n, 3), `reach_m` (a scalar) and, per part k, `v<k>` (vertices) and `f<k>`
(triangles), all in one frame and in metres. A point is flagged when, for at least one part, its
generalized winding number exceeds 0.5 (`contact.inside`) and its distance to that part is under
`reach_m`. The answer is a boolean (n,) array.

Why a winding number. A garment shell is open (waist, cuffs, an open front) and, after decimation, its
flat triangles cut across curved skin. Skin that pokes through a triangle by a few millimetres is outside
the shell locally, so a ray along its normal meets nothing; one of its neighbours is inside. The winding
number answers "inside?" for open and holed shells alike (contact.py). The reach keeps the test local:
skin far inside a loose shell (a neck inside a hood's ring) is not flagged by this rule.
"""

from __future__ import annotations

import sys
from pathlib import Path

import numpy as np
from numpy.typing import NDArray

from mwcast import contact


def flags(points: NDArray[np.float64], parts: list[contact.TriMesh], reach_m: float) -> NDArray[np.bool_]:
    out = np.zeros(points.shape[0], dtype=np.bool_)
    if points.shape[0] == 0:
        return out
    for part in parts:
        found = contact.inside(part, points)
        close = found.depths_mm < reach_m * 1000.0
        out[found.indices[close]] = True
    return out


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: python -m mwcast.wardrobe.cover <query.npz> <flags.npy>", file=sys.stderr)
        return 2
    with np.load(Path(argv[0])) as query:
        points = np.asarray(query["points"], dtype=np.float64)
        reach_m = float(query["reach_m"])
        parts: list[contact.TriMesh] = []
        k = 0
        while f"v{k}" in query:
            parts.append(contact.TriMesh(query[f"v{k}"], query[f"f{k}"]))
            k += 1
    try:
        result = flags(points, parts, reach_m)
    except contact.ContactUnavailable as exc:
        print(f"cover: {exc}; install tools/character_pipeline/requirements.txt", file=sys.stderr)
        return 3
    np.save(Path(argv[1]), result)
    within = f"{reach_m * 1000:.0f} mm"
    print(f"cover: {int(result.sum())} of {points.shape[0]} points inside a part within {within}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
