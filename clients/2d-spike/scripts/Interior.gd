extends Node2D

## The inside of the café: floor, walls and the light coming off the pendants.
##
## The room is not a separate scene. It occupies real world coordinates behind
## the café's frontage, so walking in is ordinary movement north through the
## doorway — the same coordinate space, the same camera, no swap and no load.
## What makes it readable is that the building sprite fades out once the player
## is over the threshold, which is the roof-lift cutaway an isometric game
## normally uses; the room is always there underneath it.
##
## Furniture is not drawn here. It is placed as ordinary props in the y-sorted
## world, so the player walks in front of and behind it exactly as they do with
## a bench outside. Only the surfaces the player cannot occupy live in here.

const Iso := preload("res://scripts/Iso.gd")

## Must match Main.ROOM / Main.DOOR.
var room := Rect2(3.45, -4.15, 3.55, 2.85)
var door := Rect2(4.66, -1.35, 1.18, 1.75)

const WALL_H := 2.8                     # metres, floor to ceiling
const CUT_H := 0.42                     # the stub left where a wall is cut away
const C := {
	"floor_tint": Color(1.02, 0.97, 0.90),
	"wall": Color("efe4d0"),
	"wall_lo": Color("dccbb0"),
	"wall_shade": Color("c4b294"),
	"wall_side": Color("cfbc9e"),
	"wall_side_lo": Color("b2a083"),
	"skirt": Color("8a6a46"),
	"rail": Color("a98a62"),
	"cut": Color("d9cdb6"),
	"cut_edge": Color("9c8c72"),
	"plinth": Color("b3aa9c"),
	"board": Color("3a3a38"),
	"board_edge": Color("8a6a42"),
	"glass": Color("bcd6e0"),
	"glass_hi": Color("dceaf0"),
	"frame": Color("9c7a52"),
}

var floor_tex: Texture2D
var _lights: Array[Vector2] = []


func _ready() -> void:
	z_index = -850
	texture_repeat = CanvasItem.TEXTURE_REPEAT_ENABLED
	var p := "res://art/interior/floor_boards.png"
	if ResourceLoader.exists(p):
		floor_tex = load(p)


func add_light(at: Vector2) -> void:
	_lights.append(at)


func _px(w: Vector2) -> Vector2:
	return Iso.to_screen(w)


## A wall standing on the line a->b, drawn as the quad it projects to.
func _wall(a: Vector2, b: Vector2, top: Color, bot: Color, h := WALL_H) -> void:
	var up := Vector2(0, -h * Iso.PX_PER_M_Z)
	var pa := _px(a)
	var pb := _px(b)
	draw_polygon(
		PackedVector2Array([pa + up, pb + up, pb, pa]),
		PackedColorArray([top, top, bot, bot]))


## A flat thing fixed to the wall running a->b: `along` metres from a, `up`
## metres off the floor, `w` by `h` metres.
func _hang(a: Vector2, b: Vector2, along: float, up_m: float, w: float,
		h: float, fill: Color, edge) -> void:
	var pa := _px(a)
	var dir := (_px(b) - pa).normalized()
	var per_m := (_px(b) - pa).length() / ((b - a).length() * Iso.METRES_PER_UNIT)
	var up := Vector2(0, -Iso.PX_PER_M_Z)
	var o := pa + dir * (along * per_m) + up * up_m
	var quad := PackedVector2Array([
		o, o + dir * (w * per_m),
		o + dir * (w * per_m) + up * h, o + up * h])
	if edge != null:
		var pad := dir * (0.035 * per_m) + up * 0.035
		draw_colored_polygon(PackedVector2Array([
			quad[0] - pad, quad[1] + dir * (0.07 * per_m) - up * 0.035,
			quad[2] + pad, quad[3] - dir * (0.07 * per_m) + up * 0.035]), edge)
	draw_colored_polygon(quad, fill)


func _draw() -> void:
	var r := room
	var nw := Vector2(r.position.x, r.position.y)
	var ne := Vector2(r.end.x, r.position.y)
	var se := Vector2(r.end.x, r.end.y)
	var sw := Vector2(r.position.x, r.end.y)

	# ---- footprint --------------------------------------------------------
	# A stone plinth a little larger than the room, so the interior sits on a
	# building rather than on the lawn.
	var g := r.grow(0.16)
	draw_colored_polygon(PackedVector2Array([
		_px(Vector2(g.position.x, g.position.y)),
		_px(Vector2(g.end.x, g.position.y)),
		_px(g.end), _px(Vector2(g.position.x, g.end.y))]), C["plinth"])

	# ---- floor ------------------------------------------------------------
	var quad := PackedVector2Array([_px(nw), _px(ne), _px(se), _px(sw)])
	if floor_tex:
		# One world unit is two metres; tile the boards every 1.6 m so the
		# plank scale reads against a 1.75 m person standing on them.
		var s := 1.6 / Iso.METRES_PER_UNIT
		var uv := PackedVector2Array()
		for w in [nw, ne, se, sw]:
			uv.append(Vector2(w.x / s, w.y / s))
		draw_polygon(quad, PackedColorArray([C["floor_tint"], C["floor_tint"],
			C["floor_tint"], C["floor_tint"]]), uv, floor_tex)
	else:
		draw_colored_polygon(quad, Color("c8a97e"))

	# ---- walls ------------------------------------------------------------
	# In this projection the north and west walls are the ones facing us.
	_wall(nw, ne, C["wall"], C["wall_lo"])                 # back wall
	_wall(sw, nw, C["wall_side"], C["wall_side_lo"])       # left-hand wall
	# The two near walls are cut away so the room can be seen into; what is
	# left is the thickness of the cut, which is what tells the eye this is a
	# building opened up rather than a rug on the grass.
	_wall(se, sw, C["cut"], C["cut_edge"], CUT_H)          # front, cut
	_wall(ne, se, C["cut"], C["cut_edge"], CUT_H)          # right, cut

	var up := Vector2(0, -WALL_H * Iso.PX_PER_M_Z)
	# a crisp top edge, or the walls fade into whatever is behind them
	draw_line(_px(nw) + up, _px(ne) + up, C["cut_edge"], 3.0, true)
	draw_line(_px(sw) + up, _px(nw) + up, C["cut_edge"], 3.0, true)
	# skirting, and a picture rail, so the walls have a scale to read against
	draw_line(_px(nw), _px(ne), C["skirt"], 5.0, true)
	draw_line(_px(sw), _px(nw), C["skirt"], 5.0, true)
	var rail := Vector2(0, -2.25 * Iso.PX_PER_M_Z)
	draw_line(_px(nw) + rail, _px(ne) + rail, C["rail"], 2.6, true)
	draw_line(_px(sw) + rail, _px(nw) + rail, C["rail"], 2.6, true)
	# where the two walls meet, and where they stop
	draw_line(_px(nw), _px(nw) + up, Color(0, 0, 0, 0.10), 3.0, true)

	# Things hung on the back wall. These have to lie in the wall's plane, not
	# in screen space: along the wall the basis is the wall's own direction,
	# and up is screen-vertical, which is what vertical projects to here.
	_hang(nw, ne, 0.52, 1.02, 0.72, 0.62, C["board"], C["board_edge"])
	for i in range(4):
		var ty := 1.16 + i * 0.16
		_hang(nw, ne, 0.60, ty, 0.56, 0.035, Color(1, 1, 1, 0.30), null)
	_hang(nw, ne, 1.92, 1.05, 0.95, 1.05, C["glass"], C["frame"])
	_hang(nw, ne, 2.02, 1.15, 0.75, 0.85, C["glass_hi"], null)
	_hang(nw, ne, 2.39, 1.05, 0.02, 1.05, C["frame"], null)

	# the doorway, punched through the front so the opening reads from inside
	var dl := Vector2(door.position.x, r.end.y)
	var dr := Vector2(door.end.x, r.end.y)
	draw_line(_px(dl), _px(dr), Color(1.0, 0.94, 0.80, 0.55), 7.0, true)

	# ---- light ------------------------------------------------------------
	# A warm pool under each pendant. The room is lit by lamps, not by the sun,
	# and this is what says so.
	for l in _lights:
		var c := _px(l) + Vector2(0, 6)
		for k in [[132.0, 52.0, 0.07], [96.0, 38.0, 0.09], [58.0, 23.0, 0.11]]:
			var pts := PackedVector2Array()
			for i in range(25):
				var a := TAU * i / 24.0
				pts.append(c + Vector2(cos(a) * k[0], sin(a) * k[1]))
			draw_colored_polygon(pts, Color(1.0, 0.86, 0.55, k[2]))

	# a soft occlusion where the walls meet the floor
	for pair in [[nw, ne], [sw, nw]]:
		draw_line(_px(pair[0]), _px(pair[1]), Color(0.30, 0.22, 0.14, 0.16),
			14.0, true)
