extends Node2D

const Iso := preload("res://scripts/Iso.gd")

## The ground plane: water, quay, paving and grass, drawn once.
##
## Flat vector fills in the reference palette. Nothing here is a tilemap; the
## plaza is one surface with a paving joint pattern laid over it, which is what
## keeps it looking drawn rather than tiled.

const C := {
	"stone_hi": Color("edE6d6"),
	"stone": Color("e4dbc6"),
	"stone_lo": Color("c8bea6"),
	"joint": Color("b3a78b"),
	"quay": Color("d3cab4"),
	"quay_lo": Color("b6ab90"),
	"quay_line": Color("9c9075"),
	"grass": Color("8cc260"),
	"grass_lo": Color("6ca648"),
	"grass_hi": Color("a9d579"),
	"water": Color("3fa8d6"),
	"water_lo": Color("2b85b6"),
	"water_hi": Color("7fcbe8"),
	"sand": Color("e2d6bb"),
}

const FLOWER := [Color("f2a0b4"), Color("f4e07a"), Color("ffffff"), Color("c79be0")]

# world-space extents
const LAND := Rect2(-60.0, -60.0, 120.0, 120.0)   # fills the frame, no void
const PLAZA := Rect2(0.9, -0.8, 17.8, 13.0)
const WATER := Rect2(-40.0, -30.0, 40.9, 70.0)
const PARK := Rect2(11.9, 4.4, 5.7, 6.6)
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
	draw_colored_polygon(_rect_poly(WATER), C["water_lo"])
	var inner := WATER.grow(-0.04)
	draw_colored_polygon(_rect_poly(inner), C["water"])
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
	for iy in range(ny):
		for ix in range(nx):
			var wx := PLAZA.position.x + ix * STONE
			var wy := PLAZA.position.y + iy * STONE
			if PARK.has_point(Vector2(wx + STONE * 0.5, wy + STONE * 0.5)):
				continue
			var c00 := Vector2(wx, wy) + _vjit(ix, iy)
			var c10 := Vector2(wx + STONE, wy) + _vjit(ix + 1, iy)
			var c11 := Vector2(wx + STONE, wy + STONE) + _vjit(ix + 1, iy + 1)
			var c01 := Vector2(wx, wy + STONE) + _vjit(ix, iy + 1)
			var pts := PackedVector2Array([Iso.to_screen(c00), Iso.to_screen(c10),
				Iso.to_screen(c11), Iso.to_screen(c01)])
			# pull each stone in from the joint
			var mid := (pts[0] + pts[1] + pts[2] + pts[3]) * 0.25
			for i in range(4):
				pts[i] = pts[i] + (mid - pts[i]).normalized() * 1.1
			# +/-8% value per stone, on a warm sunlit base
			var v := (float(int(hash(Vector2i(ix * 31, iy * 17))) % 1000) / 1000.0 - 0.5) * 0.085
			var col := C["stone"]
			col = Color(clampf(col.r + v, 0, 1), clampf(col.g + v * 0.97, 0, 1),
				clampf(col.b + v * 0.90, 0, 1))
			draw_colored_polygon(pts, col)
	# a broad warm sunlit wash, brighter to the upper left
	var centre := Vector2(6.4, 4.6)
	for r in [2.2, 2.7, 3.2]:
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
		var w := Vector2(11.9 + t * 5.6, 5.4 + sin(t * PI) * 2.6)
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
