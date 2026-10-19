extends Node

## `--drive=farfield`: the 2D client never shows its clear colour (RL-e0 sec.6.4, `ARC-103`).
##
## After the town is drawn, at each of the 8 reachable extremes -- the corners and edge midpoints of
## the hub's extent -- and each of 4 aspects, an offscreen viewport sharing the client's World2D is
## rendered twice: once with the clear colour set to a magenta sentinel and once to a green one. A
## pixel that differs between the two is a clear-colour pixel. PASS: zero such pixels in every render.
## The camera's zoom is what the `canvas_items`/`expand` stretch gives the pack's fixed zoom at that
## window size; it never clamps or moves anything the player's camera does.
##
## With `--capture`, the five regression views of sec.7 are written too (`ff2d_n`, `_e`, `_s`, `_w`
## at 1600 x 900; `ff2d_wide`, the east extreme at 4800 x 225), into `--shots=<dir>` or
## `clients/2d/shots/<world>/farfield/`.

## sec.6.4, fixed: 16:9, 32:9, the settings' 7680 x 360 (21.3 : 1) and 640 x 4320 (1 : 6.75).
const ASPECTS := [Vector2i(1600, 900), Vector2i(3200, 900), Vector2i(4800, 225), Vector2i(506, 3416)]
## The stretch's base size (`clients/2d/project.godot`): `expand` keeps at least this much of it.
const BASE := Vector2(1600, 900)
const SENTINELS := [Color(1, 0, 1), Color(0, 1, 0)]
const DIFF := 8
## [name, unit offset from the hub's centre, in half-extents; plan +x east, +y south]
const EXTREMES := [
	["N", Vector2(0, -1)], ["NE", Vector2(1, -1)], ["E", Vector2(1, 0)], ["SE", Vector2(1, 1)],
	["S", Vector2(0, 1)], ["SW", Vector2(-1, 1)], ["W", Vector2(-1, 0)], ["NW", Vector2(-1, -1)],
]
const SHOTS := [["ff2d_n", "N", 0], ["ff2d_e", "E", 0], ["ff2d_s", "S", 0], ["ff2d_w", "W", 0],
	["ff2d_wide", "E", 2]]
const SHOT_QUALITY := 0.85

var app: Node
var _sv: SubViewport
var _cam: Camera2D


## Returns the number of failed renders.
func run() -> int:
	var key := String(app.options.get("world-key", ""))
	var extent: Rect2 = app.town.hub_extent()
	# A world with no hub (no place with two doorways: bodies-yard's two rooms) is reached through its
	# drawn places: their footprints' union is the reachable extent.
	if not extent.has_area():
		for r in app.town.footprints.values():
			extent = r if not extent.has_area() else extent.merge(r)
	print("EVIDENCE ", JSON.stringify({"farfield2d": {"hub": app.town.hub(),
		"extent": [extent.position.x, extent.position.y, extent.size.x, extent.size.y],
		"surround": [app.places.ground.surround.position.x, app.places.ground.surround.position.y,
			app.places.ground.surround.size.x, app.places.ground.surround.size.y]}}))
	var pack_zoom: float = float(app.presentation.setting(["camera", "zoom"], 1.0))
	var clear := RenderingServer.get_default_clear_color()
	_sv = SubViewport.new()
	_sv.world_2d = app.get_viewport().world_2d
	_sv.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	_cam = Camera2D.new()
	_sv.add_child(_cam)
	add_child(_sv)
	_cam.make_current()
	var fails := 0
	for e in EXTREMES:
		var at := extent.get_center() + extent.size * 0.5 * (e[1] as Vector2)
		for size in ASPECTS:
			var images := []
			for sentinel in SENTINELS:
				RenderingServer.set_default_clear_color(sentinel)
				images.append(await _render(at, size, pack_zoom))
			var px := _differing(images[0], images[1])
			var label := "%s %s %dx%d" % [key, e[0], size.x, size.y]
			if px == 0:
				print("[farfield2d] %s: 0 clear-colour px   ok" % label)
			else:
				print("FAIL farfield2d %s: %d clear-colour px" % [label, px])
				fails += 1
	RenderingServer.set_default_clear_color(clear)
	if app.options.has("capture"):
		await _shots(key, extent, pack_zoom)
	_sv.queue_free()
	print("\n%s" % ("all farfield2d checks pass" if fails == 0 else "%d FARFIELD2D CHECKS FAILED" % fails))
	return fails


## The zoom the client's camera has at a window of `size`: `canvas_items` stretch with `expand`
## aspect keeps the whole base size visible, so the drawn world grows with the window's longer side.
static func zoom_for(size: Vector2i, pack_zoom: float) -> float:
	return pack_zoom * minf(size.x / BASE.x, size.y / BASE.y)


func _render(at: Vector2, size: Vector2i, pack_zoom: float) -> Image:
	_sv.size = size
	_cam.zoom = Vector2.ONE * zoom_for(size, pack_zoom)
	_cam.position = app.projection.to_screen(at)
	for i in 3:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw
	var img := _sv.get_texture().get_image()
	img.convert(Image.FORMAT_RGB8)
	return img


static func _differing(a: Image, b: Image) -> int:
	var ad := a.get_data()
	var bd := b.get_data()
	var n := 0
	for i in range(0, ad.size(), 3):
		if absi(ad[i] - bd[i]) > DIFF or absi(ad[i + 1] - bd[i + 1]) > DIFF \
				or absi(ad[i + 2] - bd[i + 2]) > DIFF:
			n += 1
	return n


func _shots(key: String, extent: Rect2, pack_zoom: float) -> void:
	var dir := String(app.options.get("shots", ""))
	if dir == "":
		dir = ProjectSettings.globalize_path("res://shots/%s/farfield" % key)
	DirAccess.make_dir_recursive_absolute(dir)
	for s in SHOTS:
		var unit: Vector2 = EXTREMES.filter(func(e: Array) -> bool: return e[0] == s[1])[0][1]
		var size: Vector2i = ASPECTS[s[2]]
		var img := await _render(extent.get_center() + extent.size * 0.5 * unit, size, pack_zoom)
		img.resize(size.x * 3 / 5 if s[2] == 0 else size.x * 2 / 5,
			size.y * 3 / 5 if s[2] == 0 else size.y * 2 / 5, Image.INTERPOLATE_LANCZOS)
		var path := dir.path_join("%s.jpg" % s[0])
		img.save_jpg(path, SHOT_QUALITY)
		print("EVIDENCE ", JSON.stringify({"still": path, "size": [img.get_width(), img.get_height()]}))
