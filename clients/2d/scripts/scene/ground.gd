extends Node2D

## Everything nobody stands in front of: open land, the paved hub, lawns, and the floors and walls of
## rooms (the cut-away). Drawn once per layout change from plan-metre rectangles `places.gd` hands
## over; the paving and walls are the spike's (`Ground.gd`, `Interior.gd`), generalized from its
## hard-coded square to any rectangle, coloured from the pack's palette or plainly.

var projection
var presentation
## Plan rectangles, set by places.gd before [method queue_redraw].
var land := Rect2()
var paved := Rect2()
## Each `{rect: Rect2, kind: "lawn"|"room", far: bool, lights: Array[Vector2], alpha: float}`.
var areas: Array = []

var _rng := RandomNumberGenerator.new()


func _c(name: String, plain: Color) -> Color:
	return presentation.colour(name, plain)


func _poly(points: Array) -> PackedVector2Array:
	var out := PackedVector2Array()
	for p in points:
		out.append(projection.to_screen(p))
	return out


func _rect_poly(r: Rect2) -> PackedVector2Array:
	return _poly([r.position, Vector2(r.end.x, r.position.y), r.end, Vector2(r.position.x, r.end.y)])


func _draw() -> void:
	_rng.seed = 20261008
	if land.has_area():
		draw_colored_polygon(_rect_poly(land), _c("grass_lo", Color("7d9a63")))
		if not presentation.is_plain():
			_grass_strokes(land.grow(-land.size.x * 0.3))
	if paved.has_area():
		_paving(paved)
	for area in areas:
		if area["kind"] == "lawn":
			_lawn(area["rect"])
	for area in areas:
		if area["kind"] == "room" and float(area.get("alpha", 1.0)) > 0.002:
			_room(area)


func _grass_strokes(r: Rect2) -> void:
	for i in range(700):
		var p := Vector2(r.position.x + _rng.randf() * r.size.x, r.position.y + _rng.randf() * r.size.y)
		var s: Vector2 = projection.to_screen(p)
		draw_line(s, s + Vector2(9 + _rng.randf() * 13, 0), Color(_c("grass", Color("8ab251")), 0.8), 4.0, true)


## Slabs of mixed size over mortar, each Gouraud-shaded (the spike's plaza, any rectangle).
func _paving(r: Rect2) -> void:
	draw_colored_polygon(_rect_poly(r), _c("joint", Color("b9b4ac")))
	if presentation.is_plain():
		draw_colored_polygon(_rect_poly(r.grow(-0.05)), Color("d8d3cb"))
		return
	var stone := 0.88
	var nx := int(r.size.x / stone)
	var ny := int(r.size.y / stone)
	var taken := {}
	var base := _c("stone", Color("c7bfb3"))
	for iy in range(ny):
		for ix in range(nx):
			if taken.has(Vector2i(ix, iy)):
				continue
			var sx := 1
			var sy := 1
			var roll := float(posmod(hash(Vector2i(ix * 5, iy * 13)), 1000)) / 1000.0
			if roll < 0.20 and ix + 1 < nx and not taken.has(Vector2i(ix + 1, iy)):
				sx = 2
			elif roll < 0.38 and iy + 1 < ny and not taken.has(Vector2i(ix, iy + 1)):
				sy = 2
			for dy in range(sy):
				for dx in range(sx):
					taken[Vector2i(ix + dx, iy + dy)] = true
			var o := r.position + Vector2(ix * stone, iy * stone)
			var pts := _poly([o, o + Vector2(stone * sx, 0), o + Vector2(stone * sx, stone * sy), o + Vector2(0, stone * sy)])
			var mid := (pts[0] + pts[1] + pts[2] + pts[3]) * 0.25
			for i in range(4):
				pts[i] = pts[i] + (mid - pts[i]).normalized() * 0.7
			var v := (float(posmod(hash(Vector2i(ix * 31, iy * 17)), 1000)) / 1000.0 - 0.5) * 0.055
			v += sin(o.x * 0.45 + o.y * 0.25) * 0.02 + sin(o.x * 0.12 - o.y * 0.2) * 0.026
			var col := Color(clampf(base.r + v, 0, 1), clampf(base.g + v * 0.97, 0, 1), clampf(base.b + v * 0.9, 0, 1))
			draw_polygon(pts, PackedColorArray([col.lightened(0.022), col.lightened(0.008), col.darkened(0.03), col.darkened(0.01)]))
	# Weather: stains and wear, so a large flat surface does not read as card.
	for i in range(int(r.get_area() / 6.0)):
		var wp := r.position + Vector2(_rng.randf() * r.size.x, _rng.randf() * r.size.y)
		var sp: Vector2 = projection.to_screen(wp)
		var rr := 14.0 + _rng.randf() * 40.0
		draw_colored_polygon(_blob(sp, rr, rr * 0.5), Color(0.36, 0.32, 0.26, 0.05) if _rng.randf() < 0.62 else Color(0.96, 0.94, 0.88, 0.05))


func _blob(c: Vector2, rx: float, ry: float) -> PackedVector2Array:
	var pts := PackedVector2Array()
	var ph := _rng.randf() * TAU
	for i in range(11):
		var a := TAU * i / 11.0
		var k := 0.68 + 0.21 * sin(a * 3.0 + ph) + _rng.randf() * 0.3
		pts.append(c + Vector2(cos(a) * rx * k, sin(a) * ry * k))
	return pts


func _lawn(r: Rect2) -> void:
	draw_colored_polygon(_rect_poly(r), _c("grass_lo", Color("7d9a63")))
	draw_colored_polygon(_rect_poly(r.grow(-0.15)), _c("grass", Color("9bb882")))
	if presentation.is_plain():
		return
	for i in range(int(r.get_area() * 0.6)):
		var p := r.position + Vector2(_rng.randf() * r.size.x, _rng.randf() * r.size.y)
		var s: Vector2 = projection.to_screen(p)
		draw_line(s, s + Vector2(8 + _rng.randf() * 10, 0), Color(_c("grass_hi", Color("aecc63")), 0.55), 3.0, true)


## A room opened up: floor, the two far walls standing, the near walls cut to stubs (the spike's
## roof-lift cut-away, any rectangle and any orientation).
## A disclosed doorway of a room (F-10): a mat on the ground just outside it, and the opening in the
## wall — standing, as tall as a door, in a far wall; a dark gap across a cut-away near wall.
func _door(door: Dictionary, wall_h: float, alpha: float) -> void:
	const HALF_WIDTH_M := 0.55
	var at: Vector2 = door["at"]
	var out: Vector2 = door["out"]
	var along := Vector2(-out.y, out.x)
	var a := at - along * HALF_WIDTH_M
	var b := at + along * HALF_WIDTH_M
	draw_colored_polygon(_poly([a, b, b + out * 0.9, a + out * 0.9]), Color(_c("door_mat", Color("b5523b")), alpha))
	var pa: Vector2 = projection.to_screen(a)
	var pb: Vector2 = projection.to_screen(b)
	var dark := Color(_c("door", Color("3a2a1c")), alpha)
	var far: bool = projection.plan_dir_to_screen(out).y < 0.0
	if far and wall_h > 0.0:
		var h := minf(wall_h, projection.height_px(2.1))
		draw_colored_polygon(PackedVector2Array([pa + Vector2(0, -h), pb + Vector2(0, -h), pb, pa]), dark)
	else:
		draw_colored_polygon(_poly([a, b, b - out * 0.2, a - out * 0.2]), dark)
		draw_line(pa, pb, dark, 5.0, true)


func _room(area: Dictionary) -> void:
	var r: Rect2 = area["rect"]
	var alpha := float(area.get("alpha", 1.0))
	var corners := [r.position, Vector2(r.end.x, r.position.y), r.end, Vector2(r.position.x, r.end.y)]
	draw_colored_polygon(_rect_poly(r.grow(0.3)), Color(_c("plinth", Color("b3aa9c")), alpha))
	var floor_tex: Texture2D = presentation.texture("texture:floor")
	var quad := _poly(corners)
	if floor_tex != null:
		var uv := PackedVector2Array()
		for p in corners:
			uv.append(p / 1.6)
		draw_polygon(quad, PackedColorArray([Color(1.02, 0.97, 0.9, alpha), Color(1.02, 0.97, 0.9, alpha), Color(1.02, 0.97, 0.9, alpha), Color(1.02, 0.97, 0.9, alpha)]), uv, floor_tex)
	else:
		draw_colored_polygon(quad, Color(_c("floor", Color("d2c2a8")), alpha))
	var centre := r.get_center()
	var wall_h: float = projection.height_px(2.8) if projection.kind == "isometric" else 0.0
	for i in range(4):
		var a: Vector2 = corners[i]
		var b: Vector2 = corners[(i + 1) % 4]
		var normal: Vector2 = projection.plan_dir_to_screen((a + b) * 0.5 - centre)
		var far := normal.y < 0.0
		var h := wall_h if far else minf(wall_h, projection.height_px(0.42))
		var pa: Vector2 = projection.to_screen(a)
		var pb: Vector2 = projection.to_screen(b)
		var top := _c("wall" if far else "cut", Color("e9e2d4")) if absf(normal.x) < absf(normal.y) * 2.0 else _c("wall_side" if far else "cut", Color("ddd4c2"))
		var bottom := _c("wall_lo" if far else "cut_edge", Color("c9c0b0"))
		if h > 0.0:
			draw_polygon(PackedVector2Array([pa + Vector2(0, -h), pb + Vector2(0, -h), pb, pa]),
				PackedColorArray([Color(top, alpha), Color(top, alpha), Color(bottom, alpha), Color(bottom, alpha)]))
			draw_line(pa + Vector2(0, -h), pb + Vector2(0, -h), Color(_c("cut_edge", Color("9c8c72")), alpha), 2.5, true)
		if far:
			draw_line(pa, pb, Color(_c("skirt", Color("8a6a46")), alpha), 4.0, true)
		else:
			draw_line(pa, pb, Color(_c("cut_edge", Color("9c8c72")), alpha), 2.0, true)
	for door in area.get("doors", []):
		_door(door, wall_h, alpha)
	for light in area.get("lights", []):
		var c: Vector2 = projection.to_screen(light) + Vector2(0, 6)
		for k in [[132.0, 52.0, 0.07], [96.0, 38.0, 0.09], [58.0, 23.0, 0.11]]:
			var pts := PackedVector2Array()
			for j in range(25):
				var t := TAU * j / 24.0
				pts.append(c + Vector2(cos(t) * k[0], sin(t) * k[1]))
			draw_colored_polygon(pts, Color(_c("light", Color(1.0, 0.86, 0.55)), k[2] * alpha))
