extends Node2D

## Shadows the scenery casts onto the ground.
##
## One sun, fixed for the spike: high and to the upper left, so everything
## throws a soft shadow down and to the right. This sits above the ground and
## below the props, so props stand on their own shadows.
##
## The softness is a real radial ramp, not stacked hard ellipses. Godot builds
## a `GradientTexture2D` with a radial fill once, and every shadow in the scene
## is that one texture stretched — which is both smoother and cheaper than the
## three-polygon blur this used to fake.
##
## Trees additionally cast *dapple*: a scatter of small soft blobs of varied
## strength rather than one flat blob. Broken light under a canopy is the most
## recognisable thing about the reference plates' paving, and it is what stops
## a large flat ground reading as a sheet of card.

const SUN := Vector2(0.62, 0.40)   # screen-space direction shadows run in

var casters: Array = []            # {pos, rx, ry, len, a, dapple}
var _tex: GradientTexture2D
var _rng := RandomNumberGenerator.new()


func _ready() -> void:
	z_index = -900
	_tex = _soft_blob()


## One soft round blob, white and fading to transparent. Tinted per draw.
func _soft_blob() -> GradientTexture2D:
	var g := Gradient.new()
	g.offsets = PackedFloat32Array([0.0, 0.45, 0.78, 1.0])
	g.colors = PackedColorArray([
		Color(1, 1, 1, 1.0), Color(1, 1, 1, 0.86),
		Color(1, 1, 1, 0.34), Color(1, 1, 1, 0.0)])
	var t := GradientTexture2D.new()
	t.gradient = g
	t.fill = GradientTexture2D.FILL_RADIAL
	t.fill_from = Vector2(0.5, 0.5)
	t.fill_to = Vector2(1.0, 0.5)
	t.width = 128
	t.height = 128
	return t


func _blob(c: Vector2, rx: float, ry: float, col: Color) -> void:
	draw_texture_rect(_tex, Rect2(c - Vector2(rx, ry), Vector2(rx, ry) * 2.0),
		false, col)


func _draw() -> void:
	_rng.seed = 20260927
	for s in casters:
		var base: Vector2 = s["pos"]
		var l: float = s["len"]
		var c := base + SUN * l
		var a: float = s["a"]
		var rx: float = s["rx"] + l * 0.16
		var ry: float = s["ry"]
		var tint := Color(0.20, 0.16, 0.11, a)
		_blob(c, rx, ry, tint)
		if not s.get("dapple", false):
			continue
		# Broken light under a canopy: a handful of smaller, harder patches
		# inside the soft mass, plus a few gaps of light punched back out.
		for i in range(9):
			var ang := _rng.randf() * TAU
			var d := sqrt(_rng.randf())
			var p := c + Vector2(cos(ang) * rx * 0.74 * d, sin(ang) * ry * 0.74 * d)
			var r := rx * _rng.randf_range(0.14, 0.30)
			_blob(p, r, r * (ry / maxf(rx, 1.0)) * 1.15,
				Color(0.17, 0.14, 0.10, a * _rng.randf_range(0.30, 0.62)))


## A footprint shadow for a building: the base quad, pushed along the sun.
func add_quad(quad: PackedVector2Array, l: float, a := 0.30) -> void:
	var mid := Vector2.ZERO
	for p in quad:
		mid += p
	mid /= quad.size()
	var rx := 0.0
	var ry := 0.0
	for p in quad:
		rx = maxf(rx, absf(p.x - mid.x))
		ry = maxf(ry, absf(p.y - mid.y))
	casters.append({"pos": mid, "rx": rx * 0.92, "ry": ry * 0.92, "len": l, "a": a})


func add(pos: Vector2, rx: float, ry: float, l: float, a := 0.34,
		dapple := false) -> void:
	casters.append({"pos": pos, "rx": rx, "ry": ry, "len": l, "a": a,
		"dapple": dapple})
