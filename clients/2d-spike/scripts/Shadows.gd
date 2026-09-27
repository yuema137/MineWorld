extends Node2D

## Shadows the scenery casts onto the ground.
##
## One sun, fixed for the spike: high and to the upper left, so everything
## throws a soft shadow down and to the right. This sits above the ground and
## below the props, so props stand on their own shadows.

const SUN := Vector2(0.62, 0.40)   # screen-space direction shadows run in

var casters: Array = []            # {pos, rx, ry, len, a}


func _ready() -> void:
	z_index = -900


func _ellipse(c: Vector2, rx: float, ry: float, tilt: float) -> PackedVector2Array:
	var pts := PackedVector2Array()
	var co := cos(tilt)
	var si := sin(tilt)
	for i in range(23):
		var t := TAU * i / 22.0
		var x := cos(t) * rx
		var y := sin(t) * ry
		pts.append(c + Vector2(x * co - y * si, x * si + y * co))
	return pts


func _draw() -> void:
	for s in casters:
		var base: Vector2 = s["pos"]
		var l: float = s["len"]
		var c := base + SUN * l
		var tilt := SUN.angle()
		# three passes standing in for a blur
		var a: float = s["a"]
		for k in [[1.30, 0.26], [1.08, 0.36], [0.88, 0.48]]:
			draw_colored_polygon(
				_ellipse(c, s["rx"] * k[0] + l * 0.18, s["ry"] * k[0], tilt),
				Color(0.22, 0.17, 0.11, a * k[1]))


## A footprint shadow for a building: the base quad, pushed along the sun.
func add_quad(quad: PackedVector2Array, l: float, a := 0.30) -> void:
	var soft := Node2D.new()   # nothing to add; quads are drawn as fat ellipses
	soft.queue_free()
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


func add(pos: Vector2, rx: float, ry: float, l: float, a := 0.34) -> void:
	casters.append({"pos": pos, "rx": rx, "ry": ry, "len": l, "a": a})
