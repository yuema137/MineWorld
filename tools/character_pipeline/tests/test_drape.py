"""The drape rule for open shapes (S36-P3b, Q-S36-P3-3): a hat brim over a scalp. A brim covers the scalp
without enclosing it, so the winding number reads the skin under it as outside; the recipe's `open_regions`
says so, and C-2 and the push-out's rule 5 then ask only that the skin lies on the brim's inner side."""

from __future__ import annotations

import numpy as np
from contact_shapes import hemisphere
from numpy.typing import NDArray

from mwcast.contact import TriMesh
from mwcast.contact_gates import LayerArrays, OutfitManifest, PartSpec, layer_order
from mwcast.gates import GateResult, Verdict
from mwcast.tolerances import Tolerances
from mwcast.wardrobe.pushout import push_out

Floats = NDArray[np.float64]
Ints = NDArray[np.int64]


def _brim(height_m: float, half_m: float = 0.15, cells: int = 30) -> TriMesh:
    """A flat square sheet at y = `height_m` (a brim with no crown), `cells` x `cells` quads."""
    steps = np.linspace(-half_m, half_m, cells + 1)
    vertices = np.array([(x, height_m, z) for z in steps for x in steps], dtype=np.float64)
    faces: list[tuple[int, int, int]] = []
    for row in range(cells):
        for col in range(cells):
            a = row * (cells + 1) + col
            b, c, d = a + 1, a + cells + 1, a + cells + 2
            faces += [(a, c, b), (b, c, d)]
    return TriMesh(vertices, np.array(faces, dtype=np.int64))


def _scalp() -> TriMesh:
    vertices, faces = hemisphere(0.100, rings=16, segments=32)
    return TriMesh(vertices, faces)


def _c2(brim: TriMesh, open_regions: tuple[str, ...]) -> GateResult:
    scalp = _scalp()
    n = scalp.vertices.shape[0]
    skin = LayerArrays(scalp.vertices, scalp.faces, np.zeros(n, dtype=np.int64), np.ones(n, dtype=bool))
    m = brim.vertices.shape[0]
    part = LayerArrays(brim.vertices, brim.faces, np.full(m, -1, dtype=np.int64), np.zeros(m, dtype=bool))
    manifest = OutfitManifest(
        ("scalp",), {"part_brim": PartSpec(1.0, ("scalp",), "head", open_regions=open_regions)}
    )
    return layer_order(manifest, Tolerances.load(), "Body", skin, {"part_brim": part}, [])


def test_a_brim_over_the_scalp_passes_by_the_drape_rule_and_fails_as_an_enclosing_shell() -> None:
    brim = _brim(0.120)  # 20 mm over the crown
    draped = _c2(brim, ("scalp",))
    assert draped.verdict is Verdict.PASS, draped.reasons
    assert any("by the drape rule" in line for line in draped.looked_at)
    enclosing = _c2(brim, ())
    assert enclosing.verdict is Verdict.FAIL
    assert (
        enclosing.reasons[0].startswith("Body pokes") and "through part_brim in scalp" in enclosing.reasons[0]
    )


def test_the_crown_poking_3_mm_through_the_brim_fails_the_drape_rule() -> None:
    result = _c2(_brim(0.097), ("scalp",))
    assert result.verdict is Verdict.FAIL
    assert result.reasons == ("Body pokes 3.0 mm through part_brim in scalp (garment.poke.body 2 mm)",)


def test_rule_5_leaves_a_draped_brim_alone_and_lifts_it_where_the_crown_pokes_through() -> None:
    scalp = _scalp()
    crown = np.flatnonzero(scalp.vertices[:, 1] > 0.09).astype(np.int64)
    clear = _brim(0.120)
    _moved, report = push_out(clear, scalp, probe=crown, draped=np.ones(crown.size, dtype=bool))
    assert report.moved == 0  # the skin lies on the brim's inner side: nothing to enclose
    _moved, enclosing = push_out(clear, scalp, probe=crown, draped=np.zeros(crown.size, dtype=bool))
    assert enclosing.moved > 0  # read as an enclosing shell, the skin 20 mm under it "pokes out"
    low = _brim(0.097)
    lifted, poked = push_out(low, scalp, probe=crown, draped=np.ones(crown.size, dtype=bool))
    assert poked.moved > 0 and poked.still_poking == 0
    assert lifted[:, 1].max() >= 0.100 + 0.0015  # over the crown by the gap, less rule 5's slack
