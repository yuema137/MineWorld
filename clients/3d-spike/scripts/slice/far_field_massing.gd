## The far field's massing geometry (`far_field_view.gd` builds and places it; RL-e0 sec.5.4):
## the box shapes, their palette, and the overlap test. Split from the view when the RL-e0b drawing
## rules took it past the ~800-line mark (RL-e0b sec.5.2).
class_name SliceFarFieldMassing
extends RefCounted

## Massing: footprints and the share of boxes with a pitched cap (sec.5.4).
const BOX_W := Vector2(8.0, 20.0)
const BOX_D := Vector2(8.0, 16.0)
const CAP_RISE_M := 2.2
const WALLS := [Color(0.84, 0.78, 0.67), Color(0.76, 0.66, 0.52), Color(0.56, 0.36, 0.28)]
const ROOF := Color(0.47, 0.30, 0.24)


## True when a box at `centre` (footprint radius `radius`) overlaps no placed box and no cleared slot.
static func free_at(centre: Vector3, radius: float, placed: Dictionary, clear: Array) -> bool:
	for c in clear:
		if centre.distance_to(c[0]) < radius + float(c[1]):
			return false
	var cell := Vector2i(floori(centre.x / 40.0), floori(centre.z / 40.0))
	for dx in range(-1, 2):
		for dz in range(-1, 2):
			for other in placed.get(cell + Vector2i(dx, dz), []):
				if centre.distance_to(other[0]) < radius + float(other[1]):
					return false
	return true


## A building facing the origin: its front, two sides and its roof (flat, or a low pitch with a
## front gable). The back faces away from every reachable eye and is not built.
static func box(st: SurfaceTool, base: Vector3, bearing: float, w: float, d: float, h: float,
		capped: bool, wall: Color) -> void:
	var out: Vector3 = SliceFarFieldView._dir(bearing)   # away from the street
	var side := Vector3(-out.z, 0.0, out.x)     # along the frontage
	var f := base - out * (d * 0.5)             # front face centre at ground
	var b := base + out * (d * 0.5)
	var hw := side * (w * 0.5)
	var up := Vector3(0, h, 0)
	var fl := f - hw
	var fr := f + hw
	var bl := b - hw
	var br := b + hw
	quad(st, fl, fr, fr + up, fl + up, -out, wall)                  # front
	quad(st, bl, fl, fl + up, bl + up, -side, wall.darkened(0.12))  # left
	quad(st, fr, br, br + up, fr + up, side, wall.darkened(0.12))   # right
	if not capped:
		quad(st, fl + up, fr + up, br + up, bl + up, Vector3.UP, ROOF)
		return
	var ridge := Vector3(0, h + CAP_RISE_M, 0)
	var mf := f + ridge
	var mb := b + ridge
	tri(st, fl + up, fr + up, mf, -out, wall)                       # front gable
	quad(st, fl + up, mf, mb, bl + up, -side + Vector3.UP, ROOF)
	quad(st, mf, fr + up, br + up, mb, side + Vector3.UP, ROOF)


static func quad(st: SurfaceTool, a: Vector3, b: Vector3, c: Vector3, d: Vector3, facing: Vector3,
		colour: Color) -> void:
	tri(st, a, b, c, facing, colour)
	tri(st, a, c, d, facing, colour)


## One triangle whose front face looks along `facing`: in Godot's convention the front is clockwise
## as seen from that side, i.e. (b - a) x (c - a) points away from it.
static func tri(st: SurfaceTool, a: Vector3, b: Vector3, c: Vector3, facing: Vector3,
		colour: Color) -> void:
	if (b - a).cross(c - a).dot(facing) > 0.0:
		var t := b
		b = c
		c = t
	for v in [a, b, c]:
		st.set_color(colour.srgb_to_linear())   # vertex colours are linear
		st.add_vertex(v)
