# S34-e read-only seat-surface measurement append

Bound implementation: `4ce72437c8206826a5d5b5eba9fc3471024032c6`. This external append does not modify the immutable implementation or the C5 packet at `9c41ba69`. Investigation only: no approval of format 2, vector API, changed acceptance, world edits, or full S34-e acceptance.

## Findings and limits

Both Café worlds have identical measured seat geometry and placement. Of 18 seat rows, four (table-2.2 and table-3.1 in each world) have a scalar-facing line that does not intersect the selected visual seat triangles: minimum lateral gaps 279.9 and 270.8 mm. This tests triangle support, not furnishing-center equality. The other 14 lines intersect; this does not establish body fit or validate any particular scalar offset. The earlier furnishing-center tolerance failures must not be reported as ten actual surface failures.

The round stool's candidate seat top is 0.6893 m above its floor-mounted lowest vertex, about 239 mm above a 0.45 m default. Chair candidate center-ray heights are 0.4532 m (dining) and 0.4471 m (painted). Collision-solid overall heights are not seat heights.

All 29 frozen Lakeside scalar outputs sample over a candidate visual seat surface. Bakery's five calibrated points are approximately 3.71 mm below the sampled curved mesh; pier's five and square's six coincide with the continuous primitive top; station's seven and platform's six are 2.5 mm above their actual slat tops. This supports leaving legacy scalar outputs unchanged. It is a point-support/height audit, not a body-contact acceptance result.

Five of those legacy samples also lie below an overhead bench armrest: station wall-bench.2/.4/.6 and platform platform-bench-s.2/platform-bench-n.2. Armrest tops are 0.69 m, with seat slats at 0.4475 m. The report does not infer rig penetration without a body/pose check. Preserve legacy output and frozen F-RC-7 behavior; actual clearance remains a separate S34-f concern if required.

## Geometry, coordinates, and reproducibility

`measure.py` reads the three bound original glTF files and binary buffers, applies their node hierarchy transforms, then subtracts the complete mesh AABB X/Z center and true lowest Y vertex. Input units are metres; import root scales are 1. The stool has an authored approximately 90-degree Y node rotation; the two chairs have identity node rotations. No content import, model rescaling, paid assets, Godot render, or production edit was performed.

`dressing.gd:77–105` defines centering, floor mounting, and later support settling; `props.gd:65–103` loads the model; `furnishings.gd:169–218,341–382` maps bindings and primitive dimensions. All three bindings use front_md=180000 (`furnishings.yaml:24–28`). Rotation is theta=(front_md-furnishing_facing)/1000 degrees. Local x,z map through Godot Y rotation to world east=x, north=-z. World coordinates are authored millimetres, converted to metres. For a point facing alpha, forward u=(sin(alpha),cos(alpha)); a scalar offset gives pelvis P - offset*u. Candidate B displacement below is target minus P in place-local east/north metres; these are two different conventions and must not be interchanged.

`surfaces.py` selects upward-facing triangles within 20 degrees of horizontal whose centroids lie in the manually interpreted seat bands: dining 0.40–0.48 m, painted 0.42–0.48 m, stool 0.67–0.71 m. These bands are evidenced by mesh area peaks and the attached top/side projection, but seat identification retains human visual interpretation uncertainty. `candidate-surfaces.json` contains exact selected triangle counts, bounds, area-weighted centroids and outer support envelopes. Original selected triangles are saved separately. The convex hull is only an outer envelope: it cannot establish support across gaps. Point support and scalar-line intersection use the original selected triangles. These results do not establish failure against every conceivable sloped edge or other unselected mesh component.

For a curved seat there is no single exact plane: the center ray samples the upper selected triangle, not the centroid's averaged Y. Dining selected Y spans 0.4167–0.4537 m; painted 0.4335–0.4508 m; stool 0.6862–0.6897 m. No precision claim beyond source geometry or engine equivalence is implied. Raw node quaternion rounding and engine import floating-point behavior remain unverified. Support settling (`settle.gd`) can translate an instantiated model onto nearby parent mesh support; no actual scene was instantiated here. Reported heights are relative to model floor mount, before any runtime support translation.

Primitive evidence: `station/pieces.gd:81–104` creates three seat slats, each 0.128 m deep with 0.012 m gaps, center Y=seat_h-0.02 and thickness 0.035, giving top=seat_h-0.0025. Armrests use center Y=seat_h+0.22, thickness 0.04. `furnishings.gd:270–291` builds continuous bench boxes directly from their world solid dimensions. Collision solids are used as builder parameters only where the actual primitive builder explicitly does so; they are not treated as visual chair seat surfaces.

Reproduce from the preserved checkout and existing Python NumPy/Matplotlib plus Ruby YAML:

```sh
/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh python3 /tmp/mineworld-s34e-seat-measure/measure.py
/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh python3 /tmp/mineworld-s34e-seat-measure/surfaces.py
/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh python3 /tmp/mineworld-s34e-seat-measure/placements.py
python3 /tmp/mineworld-s34e-seat-measure/report.py
```

Each final measurement command completed successfully in seconds under the existing slot budget. No acceptance tests were broadened. See `provenance.json` for source/helper/artifact SHA256 hashes and exact HEAD.

## Measured candidate B values — PROPOSED, not authored pelvis calibration

The target is the selected surface's geometric area centroid in X/Z, with its upper seat-ray height. These values quantify feasibility; they do not approve a human pelvis target, fit margin, or a schema. Both Café worlds share this table.

| Seat | Candidate displacement east, north (m) | Candidate height (m) | Scalar line intersects selected seat |
|---|---|---|---|
| window-stools.1 | +0.0000, -0.4771 | 0.6893 | yes |
| window-stools.2 | -0.0007, -0.4982 | 0.6893 | yes |
| window-stools.3 | -0.0014, -0.5163 | 0.6893 | yes |
| table-1.1 | +0.6521, +0.0135 | 0.4532 | yes |
| table-1.2 | -0.6521, -0.0135 | 0.4532 | yes |
| table-2.1 | +0.5940, -0.0095 | 0.4471 | yes |
| table-2.2 | -0.0480, -0.4985 | 0.4471 | no; 279.9 mm gap |
| table-3.1 | +0.1008, +0.4865 | 0.4532 | no; 270.8 mm gap |
| table-3.2 | -0.6408, -0.0065 | 0.4532 | yes |

Full precision, absolute targets, furnishing and point facings, projected scalar-ray heights, and per-seat legacy samples are in `placement-measurements.json`. Measured candidate polygons are in `candidate-surfaces.json`; `seat-surfaces.png` provides nonoverlapping top/side projections.

## Freeze-ready decisions still missing

Bounded presentation-only B is geometrically feasible for candidate point placement, while preserving semantic world points and all legacy scalar output. The separate existing PROPOSED contract remains unchanged: mutually exclusive optional place-local east/north displacement and legacy scalar; explicit new-format handling, strict finite validation, disclosed-point precedence, unchanged scalar signatures/absence behavior, and no S34-f rig/contact scope.

Before freezing production values, a reviewer must accept or revise the candidate seat-band interpretation and choose actual seat targets/support margins, confirm runtime mounting translation, and determine whether seat-height tolerance applies to a point or a body contact. A universal furniture-origin oracle is unsupported. An independent oracle should reference selected visual seat triangles/support polygons, explicitly handle curved surfaces and primitive slat gaps, and retain calibration provenance. Do not derive expected targets from the new displacement being tested. A 10 mm universal target tolerance and exact anatomical pelvis targets are not established by this investigation. No new implementation/API/acceptance changes are authorized by this append.
