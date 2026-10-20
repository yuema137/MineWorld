"""The contact probe on synthetic meshes with known answers (S36-P1 commit 1)."""

from __future__ import annotations

import sys

import numpy as np
import pytest
from contact_shapes import box, hemisphere

from mwcast import contact
from mwcast.contact import ContactUnavailable, RigidPart, TriMesh


def _mesh(shape: tuple[object, object]) -> TriMesh:
    vertices, faces = shape
    return TriMesh(np.asarray(vertices), np.asarray(faces))


def _translation(x: float = 0.0, y: float = 0.0, z: float = 0.0) -> np.ndarray:
    matrix = np.eye(4)
    matrix[:3, 3] = (x, y, z)
    return matrix


def test_a_point_8_mm_inside_a_face_of_the_cube_is_8_mm_deep() -> None:
    cube = _mesh(box((0.0, 0.0, 0.0), (1.0, 1.0, 1.0)))
    found = contact.inside(cube, [[0.5, 0.5, 0.992], [0.5, 0.5, 1.001]])
    assert found.indices.tolist() == [0]
    assert found.depths_mm[0] == pytest.approx(8.0, abs=0.01)
    away = contact.outside(cube, [[0.5, 0.5, 0.992], [0.5, 0.5, 1.001]])
    assert away.indices.tolist() == [1]
    assert away.distances_mm[0] == pytest.approx(1.0, abs=0.01)


def test_a_point_under_an_open_dome_is_inside_where_ray_parity_would_not_say_so() -> None:
    # Directly below the pole, a ray straight down leaves through the open rim without crossing the surface,
    # so parity reads "outside"; the winding number is about 0.5 + a bit at the centre of a hemisphere and
    # well above it near the dome. A point under the dome, 20 mm below its top, reads inside.
    dome = _mesh(hemisphere(0.1))
    found = contact.inside(dome, [[0.0, 0.08, 0.0], [0.0, 0.15, 0.0], [0.3, 0.05, 0.0]])
    assert found.indices.tolist() == [0]
    assert found.depths_mm[0] == pytest.approx(20.0, abs=0.5)


def test_two_boxes_overlapping_by_4_mm_overlap_by_4_mm_and_two_1_mm_apart_do_not() -> None:
    unit = _mesh(box((0.0, 0.0, 0.0), (1.0, 1.0, 1.0)))
    near = contact.overlaps([RigidPart("a", unit, np.eye(4)), RigidPart("b", unit, _translation(x=0.996))])
    assert [(o.a, o.b) for o in near] == [("a", "b")]
    assert near[0].depth_mm == pytest.approx(4.0, abs=0.01)
    apart = contact.overlaps([RigidPart("a", unit, np.eye(4)), RigidPart("b", unit, _translation(x=1.001))])
    assert apart == []


def test_lowest_and_clearance_read_the_y_axis() -> None:
    points = [[0.0, 0.006, 0.0], [5.0, 0.2, -3.0]]
    assert contact.lowest(points) == pytest.approx(0.006)
    assert contact.clearance(points, 0.0) == pytest.approx(6.0)
    assert contact.clearance(points, 0.01) == pytest.approx(-4.0)


@pytest.mark.parametrize(
    ("vertices", "faces", "message"),
    [
        ([[0, 0, 0], [1, 0, 0], [0, 1, 0]], [[0, 1, 3]], "outside 0..2"),
        ([[0, 0, 0], [1, float("nan"), 0], [0, 1, 0]], [[0, 1, 2]], "not finite"),
        (np.zeros((0, 3)), np.zeros((0, 3)), "no vertices"),
        ([[0, 0, 0], [1, 0, 0], [0, 1, 0]], np.zeros((0, 3)), "non-empty"),
    ],
)
def test_a_malformed_mesh_is_refused(vertices: object, faces: object, message: str) -> None:
    with pytest.raises(ValueError, match=message):
        TriMesh(np.asarray(vertices, dtype=np.float64), np.asarray(faces, dtype=np.int64))


def test_a_missing_library_is_reported_by_its_distribution_name(monkeypatch: pytest.MonkeyPatch) -> None:
    # A None entry in sys.modules makes `import igl` raise ImportError, as an uninstalled package does.
    monkeypatch.setitem(sys.modules, "igl", None)
    cube = _mesh(box((0.0, 0.0, 0.0), (1.0, 1.0, 1.0)))
    with pytest.raises(ContactUnavailable) as raised:
        contact.inside(cube, [[0.5, 0.5, 0.5]])
    assert raised.value.library == "libigl"
