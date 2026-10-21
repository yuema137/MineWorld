extends Node2D

## Side-by-side 2D walk: left, the client's current gait (two stride frames per view, swapped every
## STRIDE_M of travel, people.gd `_animate`); right, the prototype walk sheet (N frames of one whole
## cycle, frame = phase of the distance walked over the body's own cycle length). Same bodies, same
## path, same speed, same projection (isometric, 32 px per metre, sprites at scale 0.5), zoomed.
## Args after `--`: --repo=<checkout> --sheets=<dir of the prototype sheets>

const PPM := 32.0
const ZOOM := 2.5
const STRIDE_M := 0.72
const STRIDE_HOLD_S := 0.4
const PATH_M := 3.0
const PAUSE_S := 0.9
const KEYS := ["vera-lindgren", "alice-moreau", "dario-russo"]

var _repo := ""
var _sheets := ""
var _walkers: Array = []
var _t := 0.0


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--repo="):
			_repo = a.substr(7)
		elif a.begins_with("--sheets="):
			_sheets = a.substr(9)
	RenderingServer.set_default_clear_color(Color("efe6d4"))
	var bindings: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(
		_repo.path_join("presentation/mineworld-default/2D/assets/asset_bindings.yaml")))["sprites"]
	for side in 2:
		var panel := Node2D.new()
		panel.position = Vector2(side * 800, 0)
		panel.scale = Vector2(ZOOM, ZOOM)
		add_child(panel)
		var grid := Grid.new()
		panel.add_child(grid)
		for i in KEYS.size():
			var w := _walker(KEYS[i], side == 1, bindings)
			w["origin"] = Vector2(60, 70 + i * 116)
			w["node"].position = w["origin"]
			grid.origins.append(w["origin"])
			panel.add_child(w["node"])
			_walkers.append(w)
		_label(Vector2(side * 800 + 20, 12), "BEFORE (main): 2 stride frames per view" if side == 0
			else "AFTER (prototype): 12-frame walk cycle, frame = distance walked", 22)
	var line := ColorRect.new()
	line.color = Color("3b2c1e")
	line.position = Vector2(798, 0)
	line.size = Vector2(4, 900)
	add_child(line)


func _label(at: Vector2, text: String, size: int) -> void:
	var l := Label.new()
	l.text = text
	l.position = at
	l.add_theme_font_size_override("font_size", size)
	l.add_theme_color_override("font_color", Color("3b2c1e"))
	add_child(l)


func _tex(path: String) -> Texture2D:
	return ImageTexture.create_from_image(Image.load_from_file(path))


func _walker(key: String, new: bool, bindings: Dictionary) -> Dictionary:
	var node := Node2D.new()
	var body := Node2D.new()
	node.add_child(body)
	var w := {"node": node, "body": body, "new": new, "views": {}, "walked": 0.0, "phase": 0.0, "still": 9.0}
	if new:
		var sheet: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(_sheets.path_join(key + "_walk.json")))
		w["cycle_m"] = float(sheet["cycle_m"])
		for view in ["front", "back"]:
			var frames: Array = []
			for name in sheet["views"][view]["frames"]:
				frames.append(_sprite(_tex(_sheets.path_join(name + ".png")), sheet["views"][view]["anchor"], 0.5, body))
			w["views"][view] = frames
	else:
		# the client's walk speed for the comparison: the same body's clip speed as the prototype uses
		var sheet: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(_sheets.path_join(key + "_walk.json")))
		w["cycle_m"] = float(sheet["cycle_m"])
		for view in ["front", "front_b", "back", "back_b"]:
			var b: Dictionary = bindings["cast_%s_%s" % [key, view]]
			w["views"][view] = [_sprite(_tex(_repo.path_join("presentation/mineworld-default/2D/art").path_join(
				String(b["file"]).trim_prefix("art/"))), b["anchor"], float(b["scale"]), body)]
	return w


func _sprite(t: Texture2D, anchor: Array, s: float, parent: Node2D) -> Sprite2D:
	var sp := Sprite2D.new()
	sp.texture = t
	sp.centered = false
	sp.offset = -Vector2(float(anchor[0]), float(anchor[1]))
	sp.scale = Vector2(s, s)
	sp.visible = false
	parent.add_child(sp)
	return sp


## Where on the path a walker is at time t (plan metres east of its start) and whether it is moving.
func _path(t: float, speed: float) -> Array:
	var leg := PATH_M / speed
	var period := 2.0 * (leg + PAUSE_S)
	var u := fmod(t, period)
	if u < leg:
		return [u * speed, 1.0]
	if u < leg + PAUSE_S:
		return [PATH_M, 0.0]
	if u < 2.0 * leg + PAUSE_S:
		return [PATH_M - (u - leg - PAUSE_S) * speed, -1.0]
	return [0.0, 0.0]


func _process(dt: float) -> void:
	_t += dt
	for w in _walkers:
		var speed: float = float(w["cycle_m"]) / (40.0 / 30.0)
		var p := _path(_t, speed)
		var east: float = p[0]
		var plan := Vector2(east, 0.0)
		var screen := Vector2((plan.x - plan.y) * PPM, (plan.x + plan.y) * PPM * 0.5)
		var prev: Vector2 = w["node"].position - w["origin"]
		w["node"].position = w["origin"] + screen
		var moved: float = (screen - prev).length() / Vector2(PPM, PPM * 0.5).length()
		var dir: float = p[1]
		if dir != 0.0:
			w["away"] = dir < 0.0
			w["body"].scale.x = 1.0 if dir > 0.0 else -1.0
		_draw_walker(w, moved if dir != 0.0 else 0.0, dt)


func _draw_walker(w: Dictionary, metres: float, dt: float) -> void:
	var away: bool = w.get("away", false)
	if metres > 0.0001:
		w["still"] = 0.0
		w["walked"] = float(w["walked"]) + metres
		w["phase"] = float(w["phase"]) + metres / STRIDE_M * PI
	else:
		w["still"] = float(w["still"]) + dt
	for v in w["views"].values():
		for s in v:
			s.visible = false
	var view := "back" if away else "front"
	if w["new"]:
		var frames: Array = w["views"][view]
		if float(w["still"]) > 0.0:
			frames[0].visible = true  # the standing frame
		else:
			var n := frames.size() - 1
			var k := int(floor(fposmod(float(w["walked"]) / float(w["cycle_m"]), 1.0) * n)) % n
			frames[1 + k].visible = true
	else:
		var striding := float(w["still"]) < STRIDE_HOLD_S
		var step_b := striding and int(floor(float(w["phase"]) / PI)) % 2 != 0
		w["views"][view + ("_b" if step_b else "")][0].visible = true


class Grid extends Node2D:
	var origins: Array = []

	func _draw() -> void:
		var c := Color(0.55, 0.45, 0.33, 0.35)
		for o: Vector2 in origins:
			for i in range(-2, 9):
				var a: float = i * 0.5
				# lines of constant east (north-south) and constant north (east-west), 0.5 m apart
				var p0 := o + _iso(Vector2(a, -1.5))
				var p1 := o + _iso(Vector2(a, 1.0))
				draw_line(p0, p1, c, 0.4)
			for j in range(-3, 3):
				var b: float = j * 0.5
				draw_line(o + _iso(Vector2(-1.0, b)), o + _iso(Vector2(4.0, b)), c, 0.4)

	func _iso(plan: Vector2) -> Vector2:
		return Vector2((plan.x - plan.y) * 32.0, (plan.x + plan.y) * 16.0)
