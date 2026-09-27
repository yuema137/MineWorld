extends Node2D

const Iso := preload("res://scripts/Iso.gd")

## The ground plane: water, quay, paving and grass, drawn once.
##
## Flat vector fills in the reference palette. Nothing here is a tilemap; the
## plaza is one surface with a paving joint pattern laid over it, which is what
## keeps it looking drawn rather than tiled.

const C := {
	# Measured off the reference plates rather than eyeballed: a k-means over
	# their low-saturation pixels lands on #8D847D / #9F9893 / #BAB1AA. The
	# paving in those images is a warm *grey*. The cream this used to be ran
	# about two steps light and much too yellow, which is most of the reason
	# the plaza read as cardboard rather than stone.
	"stone_hi": Color("cfc7bc"),
	"stone": Color("bab1a6"),
	"stone_lo": Color("9c948a"),
	"joint": Color("8a8279"),
	"quay": Color("b3aba0"),
	"quay_lo": Color("8e867c"),
	"quay_line": Color("6e675f"),
	"grass": Color("6f9440"),
	"grass_lo": Color("4e6f35"),
	"grass_hi": Color("93af46"),
	"grass_deep": Color("36502a"),
	"water": Color("2f7f9e"),
	"water_deep": Color("1f5f7d"),
	"water_lo": Color("1a4f68"),
	"water_hi": Color("5aa3bd"),
	"sand": Color("c2b7a2"),
}

const FLOWER := [Color("f2a0b4"), Color("f4e07a"), Color("ffffff"), Color("c79be0")]

# world-space extents
const LAND := Rect2(-60.0, -60.0, 120.0, 120.0)   # fills the frame, no void
const PLAZA := Rect2(0.9, -0.8, 17.8, 8.7)
const WATER := Rect2(-40.0, -30.0, 40.9, 70.0)
const PARK := Rect2(11.9, 3.12, 5.7, 4.0)
const QUAY_X := 0.9
const STONE := 0.44   # paving module, world units (~0.9 m slabs)

var _rng := RandomNumberGenerator.new()


func _ready() -> void:
	_rng.seed = 20260926
	z_index = -1000


func _quad(a: Vector2, b: Vector2, c: Vector2, d: Vector2) -> PackedVector2Array:
	return PackedVector2Array([Iso.to_screen(a), Iso.to_screen(b),
		Iso.to_screen(c), Iso.to_screen(d)])


func _rect_poly(r: Rect2) -> PackedVector2Array:
	return _quad(r.position, Vector2(r.end.x, r.position.y), r.end,
		Vector2(r.position.x, r.end.y))


func _draw() -> void:
	_rng.seed = 20260926
	_draw_land()
	_draw_water()
	_draw_plaza()
	_draw_park()
	_draw_quay()


## Open ground beyond the square, so the composition never runs out into the
## clear colour. Cheap, and it is what makes the frame read as a place.
func _draw_land() -> void:
	draw_colored_polygon(_rect_poly(LAND), C["grass_lo"])
	for i in range(900):
		var p := Vector2(-6.0 + _rng.randf() * 30.0, -8.0 + _rng.randf() * 26.0)
		var s := Iso.to_screen(p)
		draw_line(s, s + Vector2(9 + _rng.randf() * 13, 0), Color(C["grass"], 0.85), 4.0, true)
	for i in range(240):
		var p2 := Vector2(-6.0 + _rng.randf() * 30.0, -8.0 + _rng.randf() * 26.0)
		var s2 := Iso.to_screen(p2)
		draw_line(s2, s2 + Vector2(7 + _rng.randf() * 9, 0), Color(C["grass_hi"], 0.5), 3.0, true)
	# a darker verge hugging the paving, so the plaza sits in the grass
	var verge := PLAZA.grow(0.34)
	draw_colored_polygon(_rect_poly(verge), Color(C["grass_lo"].darkened(0.18), 0.55))


func _draw_water() -> void:
	draw_colored_polygon(_rect_poly(WATER), C["water_deep"])
	# ramp from deep open water to bright shallows against the quay
	for i in range(14):
		var t := i / 13.0
		var band := Rect2(QUAY_X - 11.0 * (1.0 - t), WATER.position.y,
			11.0 * (1.0 - t), WATER.size.y)
		draw_colored_polygon(_rect_poly(band),
			Color(C["water_deep"].lerp(C["water_hi"], t), 0.34))
	var shore := Rect2(QUAY_X - 0.55, WATER.position.y, 0.55, WATER.size.y)
	draw_colored_polygon(_rect_poly(shore), Color(C["water_hi"], 0.55))
	# broad light bands, then short glints, both following the iso grid
	for i in range(26):
		var y := WATER.position.y + _rng.randf() * WATER.size.y
		var x0 := WATER.position.x + _rng.randf() * WATER.size.x * 0.6
		var w := 0.7 + _rng.randf() * 2.0
		var a := Iso.to_screen(Vector2(x0, y))
		var b := Iso.to_screen(Vector2(x0 + w, y))
		draw_line(a, b, Color(C["water_hi"], 0.30), 7.0, true)
	for i in range(60):
		var y2 := WATER.position.y + _rng.randf() * WATER.size.y
		var x2 := WATER.position.x + _rng.randf() * WATER.size.x
		var a2 := Iso.to_screen(Vector2(x2, y2))
		draw_line(a2, a2 + Vector2(14.0 + _rng.randf() * 20.0, 0), Color(1, 1, 1, 0.22), 3.0, true)
	# sunlit shimmer near the quay
	for i in range(18):
		var y3 := 1.0 + _rng.randf() * 11.0
		var x3 := QUAY_X - 0.2 - _rng.randf() * 1.6
		var p := Iso.to_screen(Vector2(x3, y3))
		draw_line(p, p + Vector2(18, 0), Color("fff0c0"), 4.0, true)


## Deterministic vertex wobble, shared between neighbouring stones so the
## paving never gains a gap.
func _vjit(ix: int, iy: int) -> Vector2:
	var hx := float(int(hash(Vector2i(ix, iy))) % 1000) / 1000.0
	var hy := float(int(hash(Vector2i(ix + 7717, iy - 313))) % 1000) / 1000.0
	return Vector2((hx - 0.5) * STONE * 0.11, (hy - 0.5) * STONE * 0.11)


func _draw_plaza() -> void:
	draw_colored_polygon(_rect_poly(PLAZA), C["joint"])   # mortar shows through
	var nx := int(PLAZA.size.x / STONE)
	var ny := int(PLAZA.size.y / STONE)

	# Slabs of mixed size. A perfect lattice of identical stones is the single
	# most "vector" thing a large flat surface can do, and the plaza is the
	# biggest surface in the frame. Cells are claimed greedily into 2x1, 1x2 or
	# 1x1 slabs from a deterministic hash, so the bond breaks up without ever
	# leaving a hole or an overlap.
	var taken := {}
	for iy in range(ny):
		for ix in range(nx):
			var key := Vector2i(ix, iy)
			if taken.has(key):
				continue
			var wx := PLAZA.position.x + ix * STONE
			var wy := PLAZA.position.y + iy * STONE
			if PARK.has_point(Vector2(wx + STONE * 0.5, wy + STONE * 0.5)):
				continue
			var sx := 1
			var sy := 1
			var roll := float(int(hash(Vector2i(ix * 5, iy * 13))) % 1000) / 1000.0
			if roll < 0.20 and ix + 1 < nx and not taken.has(Vector2i(ix + 1, iy)) \
					and not PARK.has_point(Vector2(wx + STONE * 1.5, wy + STONE * 0.5)):
				sx = 2
			elif roll < 0.38 and iy + 1 < ny and not taken.has(Vector2i(ix, iy + 1)) \
					and not PARK.has_point(Vector2(wx + STONE * 0.5, wy + STONE * 1.5)):
				sy = 2
			for dy in range(sy):
				for dx in range(sx):
					taken[Vector2i(ix + dx, iy + dy)] = true

			var c00 := Vector2(wx, wy) + _vjit(ix, iy)
			var c10 := Vector2(wx + STONE * sx, wy) + _vjit(ix + sx, iy)
			var c11 := Vector2(wx + STONE * sx, wy + STONE * sy) + _vjit(ix + sx, iy + sy)
			var c01 := Vector2(wx, wy + STONE * sy) + _vjit(ix, iy + sy)
			var pts := PackedVector2Array([Iso.to_screen(c00), Iso.to_screen(c10),
				Iso.to_screen(c11), Iso.to_screen(c01)])
			# pull each stone in from the joint
			var mid := (pts[0] + pts[1] + pts[2] + pts[3]) * 0.25
			for i in range(4):
				pts[i] = pts[i] + (mid - pts[i]).normalized() * 1.0
			# +/-5% value per stone, plus a slow large-scale drift so the
			# paving weathers across the square instead of only dithering
			var v := (float(int(hash(Vector2i(ix * 31, iy * 17))) % 1000) / 1000.0 - 0.5) * 0.055
			v += sin(wx * 0.9 + wy * 0.5) * 0.020 + sin(wx * 0.23 - wy * 0.41) * 0.026
			var col := C["stone"]
			col = Color(clampf(col.r + v, 0, 1), clampf(col.g + v * 0.97, 0, 1),
				clampf(col.b + v * 0.90, 0, 1))
			# Per-slab shading: a lit top-left edge and a shaded bottom-right.
			# Gouraud, via draw_polygon's per-vertex colours — smooth, and the
			# engine does it, so there is nothing to band.
			var cols := PackedColorArray([
				col.lightened(0.055), col.lightened(0.02),
				col.darkened(0.065), col.darkened(0.02)])
			draw_polygon(pts, cols)

	# a kerb, so the paving ends because someone built it that way
	var sw_c := Vector2(PLAZA.position.x, PLAZA.end.y)
	var se_c := PLAZA.end
	var ne_c := Vector2(PLAZA.end.x, PLAZA.position.y)
	for pair in [[sw_c, se_c], [se_c, ne_c]]:
		var a := Iso.to_screen(pair[0])
		var b := Iso.to_screen(pair[1])
		draw_line(a + Vector2(0, 5), b + Vector2(0, 5), C["stone_lo"], 13.0, true)
		draw_line(a, b, C["stone_hi"], 9.0, true)
		draw_line(a + Vector2(0, 11), b + Vector2(0, 11), Color(0.22, 0.19, 0.13, 0.22),
			7.0, true)

	var centre := Vector2(6.4, 3.24)
	for r in [2.65]:
		var ring := PackedVector2Array()
		for k in range(65):
			var a := TAU * k / 64.0
			ring.append(Iso.to_screen(centre + Vector2(cos(a), sin(a)) * r))
		draw_polyline(ring, Color(C["joint"], 0.35), 2.0, true)


func _draw_park() -> void:
	draw_colored_polygon(_rect_poly(PARK), C["grass_lo"])
	draw_colored_polygon(_rect_poly(PARK.grow(-0.06)), C["grass"])
	for i in range(140):
		var p := PARK.position + Vector2(_rng.randf() * PARK.size.x, _rng.randf() * PARK.size.y)
		var s := Iso.to_screen(p)
		draw_line(s, s + Vector2(8 + _rng.randf() * 10, 0), Color(C["grass_hi"], 0.55), 3.0, true)
	for i in range(70):
		var p2 := PARK.position + Vector2(_rng.randf() * PARK.size.x, _rng.randf() * PARK.size.y)
		draw_circle(Iso.to_screen(p2), 2.6, FLOWER[_rng.randi() % FLOWER.size()])
	# a paved path curving through the grass
	var path := PackedVector2Array()
	for k in range(31):
		var t := k / 30.0
		var w := Vector2(11.9 + t * 5.6, 3.9 + sin(t * PI) * 1.6)
		path.append(Iso.to_screen(w))
	draw_polyline(path, C["stone_lo"], 42.0, true)
	draw_polyline(path, C["stone_hi"], 34.0, true)


func _draw_quay() -> void:
	# the stone edge that holds the plaza above the water
	var top_a := Vector2(QUAY_X, WATER.position.y)
	var top_b := Vector2(QUAY_X, WATER.end.y)
	var a := Iso.to_screen(top_a)
	var b := Iso.to_screen(top_b)
	var drop := Vector2(0, 46)
	draw_colored_polygon(PackedVector2Array([a, b, b + drop, a + drop]), C["quay_lo"])
	draw_colored_polygon(PackedVector2Array([a, b, b + Vector2(0, 10), a + Vector2(0, 10)]),
		C["quay"])
	# block courses
	var n := int((top_b.y - top_a.y) / 0.34)
	for i in range(n + 1):
		var wy := top_a.y + i * 0.34
		var p := Iso.to_screen(Vector2(QUAY_X, wy))
		draw_line(p + Vector2(0, 8), p + drop, Color(C["quay_line"], 0.5), 1.6, true)
	draw_line(a + Vector2(0, 24), b + Vector2(0, 24), Color(C["quay_line"], 0.45), 1.6, true)
	draw_line(a, b, Color(C["quay_line"], 0.8), 2.4, true)
	draw_line(a + drop, b + drop, Color(0.18, 0.28, 0.34, 0.35), 5.0, true)
