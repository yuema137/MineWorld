extends Node2D

## MineWorld 2D presentation spike.
##
## Builds one small quayside square from flat vector props, drops a player and
## a handful of background people into it, and lets you walk around. There is
## no simulation here and nothing is authoritative: it exists so a human can
## look at it and say whether this is the 2D look.

const Iso := preload("res://scripts/Iso.gd")
const Demo := preload("res://scripts/Demo.gd")
const GroundScript := preload("res://scripts/Ground.gd")

const ART := "res://art/svg/%s.svg"

## Scale applied to each 2x-authored sprite. Derived from one rule: a person is
## 1.75 m and 1 m is Iso.PX_PER_M_Z pixels, so a person is ~56 px tall and
## everything else is sized against that.
const SCALE := {
	"shop": 0.5, "person": 0.28, "dog": 0.26, "bird": 0.24,
	"tree_a": 0.37, "tree_b": 0.37, "tree_c": 0.34,
	"bush_a": 0.30, "bush_b": 0.28, "hedge": 0.26,
	"lamppost": 0.27, "bench": 0.26, "bench_r": 0.26,
	"planter": 0.28, "planter_b": 0.28, "pot": 0.26, "pot_b": 0.26,
	"chalkboard": 0.26, "signpost": 0.30, "cafeset": 0.30,
	"bicycle": 0.24, "barrel": 0.24, "fountain": 0.34,
	"rail_x": 0.5, "rail_y": 0.5, "jetty": 0.5, "boat": 0.30,
}

var props: Dictionary = {}
var world: Node2D
var player: Node2D
var player_at := Vector2(6.6, 7.6)
var player_facing := 1.0
var walkers: Array = []
var cam: Camera2D
var _jit := RandomNumberGenerator.new()

const WALK_MIN := Vector2(1.5, 0.2)
const WALK_MAX := Vector2(17.2, 10.9)
const PLAYER_SPEED := 2.4   # world units/second (~4.8 m/s, a brisk walk in a demo)


func _ready() -> void:
	_jit.seed = 20260926
	props = _load_props()
	world = Node2D.new()
	world.y_sort_enabled = true
	add_child(world)

	var ground := Node2D.new()
	ground.set_script(GroundScript)
	add_child(ground)

	_build_places()
	_build_scenery()
	_build_people()

	cam = Camera2D.new()
	cam.zoom = Vector2(1.05, 1.05)
	cam.position_smoothing_enabled = true
	cam.position_smoothing_speed = 4.0
	cam.position = Iso.to_screen(player_at)
	add_child(cam)
	cam.make_current()

	_build_hud()

	var args := OS.get_cmdline_user_args()
	if args.has("--drive"):
		_drive.call_deferred()
	elif args.has("--shots"):
		_shots.call_deferred()


func _load_props() -> Dictionary:
	var f := FileAccess.open("res://art/props.json", FileAccess.READ)
	return JSON.parse_string(f.get_as_text())


func _scale_for(name: String) -> float:
	if SCALE.has(name):
		return SCALE[name]
	if name.begins_with("shop_"):
		return SCALE["shop"]
	if name.begins_with("seated_") or name.begins_with("npc_") or name.begins_with("player"):
		return SCALE["person"]
	return 0.3


## One prop, anchored on the ground so the painter's sort is correct.
func _prop(name: String, at: Vector2, flip := false, z := 0) -> Node2D:
	var holder := Node2D.new()
	holder.position = Iso.to_screen(at)
	holder.z_index = z
	var s := Sprite2D.new()
	s.texture = load(ART % name)
	s.centered = false
	var m: Dictionary = props[name]
	s.offset = Vector2(-float(m["ax"]), -float(m["ay"]))
	var k := _scale_for(name)
	# Vegetation gets a little size and hue jitter, so a dozen copies of one
	# tree do not read as a dozen copies of one tree.
	if name.begins_with("tree") or name.begins_with("bush") or name == "hedge":
		k *= 0.84 + _jit.randf() * 0.34
		s.modulate = Color(0.90 + _jit.randf() * 0.16,
			0.92 + _jit.randf() * 0.14, 0.86 + _jit.randf() * 0.18)
	s.scale = Vector2(-k if flip else k, k)
	if flip:
		s.offset.x = -s.offset.x - float(m["w"]) + 2.0 * float(m["ax"])
	holder.add_child(s)
	world.add_child(holder)
	return holder


## World position of a point offset in *screen* pixels from another — handy for
## sitting someone in a chair that was drawn as part of a table sprite.
func _at_screen(base: Vector2, off: Vector2) -> Vector2:
	return Iso.to_world(Iso.to_screen(base) + off)


func _build_places() -> void:
	# Structurally dumb: a place knows its sprite, its spot and its lettering.
	var places := [
		Demo.DemoPlace.new(&"cafe", &"shop_cafe", Vector2(5.2, -0.6), "Café"),
		Demo.DemoPlace.new(&"bakery", &"shop_bakery", Vector2(9.0, -0.6), "Bakery"),
		Demo.DemoPlace.new(&"books", &"shop_books", Vector2(12.8, -0.6), "Books"),
		Demo.DemoPlace.new(&"bloom", &"shop_bloom", Vector2(16.3, -0.6), "Bloom"),
	]
	for p in places:
		var n := _prop(String(p.sprite), p.at)
		var m: Dictionary = props[String(p.sprite)]
		if not m.has("sign"):
			continue
		var k := _scale_for(String(p.sprite))
		var sign_pos := (Vector2(m["sign"][0], m["sign"][1])
			- Vector2(float(m["ax"]), float(m["ay"]))) * k
		# the shopfront is sheared by the projection, so the lettering is too
		var skewed := Node2D.new()
		skewed.transform = Transform2D(Vector2(1, 0.5), Vector2(0, 1), sign_pos)
		n.add_child(skewed)
		var lab := Label.new()
		lab.text = p.label
		lab.add_theme_color_override("font_color", Color("4b3826"))
		lab.add_theme_font_size_override("font_size", 30)
		lab.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		lab.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		var sw: float = float(m["sign_w"]) * k
		lab.size = Vector2(sw, 40)
		lab.position = Vector2(-sw * 0.5, -20)
		skewed.add_child(lab)


func _build_scenery() -> void:
	# quay, jetty and water
	_prop("jetty", Vector2(0.6, 6.6), false, -2)
	_prop("boat", Vector2(-1.6, 7.9), false, -1)
	_prop("barrel", Vector2(-0.3, 5.1))
	_prop("bird", Vector2(-3.2, 3.2), false, -1)
	_prop("bird", Vector2(-1.2, 5.4), true)
	for i in range(1, 12):
		if i >= 4 and i <= 6:
			continue   # gap where the jetty meets the quay
		_prop("rail_y", Vector2(1.18, i + 0.5))

	_prop("fountain", Vector2(6.4, 4.6))

	# shopfront clutter
	var street := [
		["pot", 1.05, -0.28, false], ["chalkboard", 1.55, -0.20, false],
		["pot_b", 5.35, -0.30, false], ["chalkboard", 6.15, -0.22, false],
		["pot", 9.15, -0.30, false], ["chalkboard", 9.95, -0.22, true],
		["pot_b", 12.95, -0.30, false], ["pot", 16.4, -0.28, false],
		["cafeset", 2.30, 0.55, false], ["cafeset", 3.95, 1.05, false],
		["bicycle", 10.70, 1.15, false],
		["signpost", 8.10, 2.55, false],
		["planter", 4.55, 2.25, false], ["planter_b", 8.70, 1.65, false],
		["planter", 11.20, 3.10, false], ["planter_b", 2.10, 2.95, false],
		["bench_r", 2.15, 7.5, false], ["bench", 5.10, 9.6, false],
		["lamppost", 1.62, 2.2, false], ["lamppost", 1.62, 5.9, false],
		["lamppost", 1.62, 9.4, false], ["lamppost", 5.05, 0.55, false],
		["lamppost", 9.45, 0.55, false], ["lamppost", 13.45, 0.70, false],
		["lamppost", 11.55, 8.40, false], ["lamppost", 7.60, 9.9, false],
	]
	for it in street:
		_prop(it[0], Vector2(it[1], it[2]), it[3])

	# the park corner
	var park := [
		["tree_a", 10.55, 2.45], ["tree_b", 2.05, 1.05], ["tree_c", 6.85, 9.9],
		["tree_a", 14.10, 5.70], ["tree_b", 15.90, 8.55], ["tree_c", 12.70, 8.90],
		["tree_c", 16.90, 3.10],
		["bush_a", 12.45, 5.10], ["bush_b", 13.70, 8.05], ["bush_a", 15.30, 6.50],
		["bush_b", 12.10, 9.70], ["bush_a", 16.70, 9.10], ["bush_b", 15.05, 4.75],
		["hedge", 12.05, 6.10], ["hedge", 12.05, 6.80], ["hedge", 12.05, 7.50],
		["bench", 13.35, 6.20], ["bench_r", 15.65, 9.70],
	]
	for it in park:
		_prop(it[0], Vector2(it[1], it[2]))

	# treeline closing the south and east sides, so the square has edges
	var edge: Array = []
	var i := 0.0
	while i < 16.0:
		edge.append(["tree_b" if int(i) % 3 == 0 else "tree_c", 2.2 + i, 11.9 + sin(i) * 0.35])
		edge.append(["bush_a" if int(i) % 2 == 0 else "bush_b", 2.8 + i, 11.25])
		i += 1.45
	var j := 0.0
	while j < 12.0:
		edge.append(["tree_c" if int(j) % 2 == 0 else "tree_b", 18.3 + sin(j) * 0.3, 0.4 + j])
		edge.append(["bush_b", 17.75, 0.9 + j])
		j += 1.5
	# and along the shopfront row, behind the buildings
	var m := 0.0
	while m < 16.0:
		edge.append(["tree_b", 1.0 + m, -4.3])
		m += 1.6
	for it in edge:
		_prop(it[0], Vector2(it[1], it[2]))

	# fill the paving: planters, pots and low greenery break up the expanse
	var fill := [
		["planter", 3.10, 4.10], ["planter_b", 3.10, 5.30], ["planter", 3.10, 6.50],
		["planter_b", 9.90, 5.60], ["planter", 9.90, 6.80], ["planter_b", 9.90, 4.40],
		["pot", 6.40, 1.90], ["pot_b", 7.30, 1.95], ["pot", 5.50, 1.85],
		["bush_b", 4.60, 8.60], ["bush_a", 6.20, 8.30], ["bush_b", 8.20, 8.90],
		["tree_c", 4.30, 7.40], ["tree_b", 9.10, 9.10], ["tree_c", 1.95, 4.40],
		["pot_b", 1.60, 6.60], ["pot", 1.60, 7.70], ["pot_b", 1.60, 3.30],
		["bench", 7.90, 3.55], ["bench_r", 8.40, 6.90],
		["planter", 12.60, 1.60], ["planter_b", 14.20, 1.80],
		["pot", 13.30, 3.80], ["bush_a", 14.60, 3.40], ["pot_b", 15.60, 2.20],
		["barrel", 2.55, 9.10], ["bicycle", 4.85, 0.35],
		["chalkboard", 13.05, -0.22], ["chalkboard", 16.50, -0.20],
		["cafeset", 5.55, 0.95],
	]
	for it in fill:
		_prop(it[0], Vector2(it[1], it[2]))


func _build_people() -> void:
	# Seated people: the Sit affordance in 2D form, as the references show it.
	_prop("seated_a", _at_screen(Vector2(2.30, 0.55), Vector2(-33, 2)))
	_prop("seated_b", _at_screen(Vector2(3.95, 1.05), Vector2(34, 2)))

	var crowd := [
		Demo.DemoPerson.new(&"a", &"npc_a", Vector2(4.2, 6.4),
			PackedVector2Array([Vector2(4.2, 6.4), Vector2(9.6, 6.9)]), 0.80),
		Demo.DemoPerson.new(&"b", &"npc_b", Vector2(8.0, 8.6),
			PackedVector2Array([Vector2(8.0, 8.6), Vector2(3.4, 8.9)]), 0.70),
		Demo.DemoPerson.new(&"c", &"npc_c", Vector2(10.8, 4.1),
			PackedVector2Array([Vector2(10.8, 4.1), Vector2(10.9, 8.6)]), 0.75),
		Demo.DemoPerson.new(&"d", &"npc_d", Vector2(5.8, 2.1),
			PackedVector2Array([Vector2(5.8, 2.1), Vector2(12.8, 2.3)]), 0.62),
		Demo.DemoPerson.new(&"e", &"npc_e", Vector2(2.6, 9.6),
			PackedVector2Array([Vector2(2.6, 9.6), Vector2(2.7, 4.6)]), 0.55),
		Demo.DemoPerson.new(&"dog", &"dog", Vector2(6.4, 7.6),
			PackedVector2Array([Vector2(6.4, 7.6), Vector2(8.8, 7.0), Vector2(7.2, 9.0)]), 1.25),
	]
	for c in crowd:
		walkers.append(_make_actor(c))

	player = _make_actor(Demo.DemoPerson.new(&"player", &"player", player_at))
	player.set_meta("is_player", true)


func _make_actor(p) -> Node2D:
	var holder := Node2D.new()
	holder.position = Iso.to_screen(p.at)
	var body := Node2D.new()
	holder.add_child(body)
	var base := String(p.sprite)
	var has_back := props.has(base + "_back")
	for suffix in (["_front", "_back"] if has_back else [""]):
		var s := Sprite2D.new()
		var nm: String = base + suffix
		s.texture = load(ART % nm)
		s.centered = false
		var m: Dictionary = props[nm]
		s.offset = Vector2(-float(m["ax"]), -float(m["ay"]))
		var k := _scale_for(nm)
		s.scale = Vector2(k, k)
		s.name = "front" if suffix != "_back" else "back"
		body.add_child(s)
	if has_back:
		body.get_node("back").visible = false
	world.add_child(holder)
	holder.set_meta("data", p)
	holder.set_meta("leg", 0)
	holder.set_meta("phase", randf() * TAU)
	return holder


func _build_hud() -> void:
	var layer := CanvasLayer.new()
	add_child(layer)
	var panel := PanelContainer.new()
	var sb := StyleBoxFlat.new()
	sb.bg_color = Color(0.99, 0.97, 0.92, 0.82)
	sb.corner_radius_top_left = 10
	sb.corner_radius_top_right = 10
	sb.corner_radius_bottom_left = 10
	sb.corner_radius_bottom_right = 10
	sb.content_margin_left = 14
	sb.content_margin_right = 14
	sb.content_margin_top = 9
	sb.content_margin_bottom = 9
	panel.add_theme_stylebox_override("panel", sb)
	panel.position = Vector2(22, 20)
	var v := VBoxContainer.new()
	panel.add_child(v)
	for t in ["MineWorld — 2D presentation spike",
			"WASD or arrow keys to walk    ·    Esc to quit"]:
		var l := Label.new()
		l.text = t
		l.add_theme_color_override("font_color", Color("4b3826"))
		l.add_theme_font_size_override("font_size", 17 if t.begins_with("MineWorld") else 14)
		v.add_child(l)
	layer.add_child(panel)


func _unhandled_input(ev: InputEvent) -> void:
	if ev is InputEventKey and ev.pressed and ev.keycode == KEY_ESCAPE:
		get_tree().quit(0)


func _process(dt: float) -> void:
	var sdir := Vector2(
		Input.get_action_strength("move_right") - Input.get_action_strength("move_left"),
		Input.get_action_strength("move_down") - Input.get_action_strength("move_up"))
	var wdir := Iso.screen_dir_to_world(sdir)
	if wdir != Vector2.ZERO:
		player_at += wdir * PLAYER_SPEED * dt
		player_at.x = clamp(player_at.x, WALK_MIN.x, WALK_MAX.x)
		player_at.y = clamp(player_at.y, WALK_MIN.y, WALK_MAX.y)
		if absf(sdir.x) > 0.01:
			player_facing = signf(sdir.x)
	player.position = Iso.to_screen(player_at)
	_animate(player, wdir, dt, player_facing)
	if cam:
		cam.position = Iso.to_screen(player_at)

	for w in walkers:
		_step_walker(w, dt)


func dir_of(v: Vector2) -> Vector2:
	return v.normalized()


func _step_walker(w: Node2D, dt: float) -> void:
	var p = w.get_meta("data")
	if (p.route as PackedVector2Array).size() < 2:
		return
	var leg: int = w.get_meta("leg")
	var route: PackedVector2Array = p.route
	var target: Vector2 = route[(leg + 1) % route.size()]
	var d: Vector2 = target - p.at
	if d.length() < 0.08:
		w.set_meta("leg", (leg + 1) % route.size())
		return
	var dir: Vector2 = dir_of(d)
	p.at = (p.at as Vector2) + dir * float(p.speed) * dt
	w.position = Iso.to_screen(p.at)
	var sdir := Iso.to_screen(dir)
	_animate(w, dir, dt, signf(sdir.x) if absf(sdir.x) > 0.01 else 1.0)


## Walk cycle: a bob, a lean, and a swap to the back view when walking away.
func _animate(actor: Node2D, wdir: Vector2, dt: float, facing: float) -> void:
	var body: Node2D = actor.get_child(0)
	var moving := wdir != Vector2.ZERO
	var ph: float = actor.get_meta("phase")
	if moving:
		ph += dt * 9.0
		actor.set_meta("phase", ph)
	var bob := (absf(sin(ph)) * -3.0) if moving else 0.0
	body.position = Vector2(0, bob)
	body.scale = Vector2(facing, 1.0)
	body.rotation = (sin(ph) * 0.03) if moving else 0.0
	var back := body.get_node_or_null("back")
	if back:
		# screen-up movement means we see their back
		var away := Iso.to_screen(wdir).y < -0.5
		back.visible = moving and away
		body.get_node("front").visible = not (moving and away)


# ---------------------------------------------------------------------------
# verification harness
# ---------------------------------------------------------------------------

func _save(name: String) -> void:
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	img.save_png("res://shots/%s.png" % name)
	print("shot: ", name)


func _settle(n: int) -> void:
	for i in range(n):
		await get_tree().process_frame


func _shots() -> void:
	DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path("res://shots"))
	cam.position_smoothing_enabled = false
	set_process(false)

	# wide: the whole square
	cam.zoom = Vector2(0.62, 0.62)
	cam.position = Iso.to_screen(Vector2(8.0, 4.2))
	await _settle(8)
	await _save("01_wide")

	# mid: people on the plaza
	cam.zoom = Vector2(1.15, 1.15)
	cam.position = Iso.to_screen(Vector2(6.6, 6.4))
	await _settle(6)
	await _save("02_mid_npcs")

	# close: the cafe frontage and its terrace
	cam.zoom = Vector2(2.2, 2.2)
	cam.position = Iso.to_screen(Vector2(3.2, 0.2))
	await _settle(6)
	await _save("03_close_cafe")

	# close: quay, jetty and boat
	cam.zoom = Vector2(1.9, 1.9)
	cam.position = Iso.to_screen(Vector2(0.4, 6.4))
	await _settle(6)
	await _save("04_close_quay")

	# the park corner
	cam.zoom = Vector2(1.35, 1.35)
	cam.position = Iso.to_screen(Vector2(13.6, 6.8))
	await _settle(6)
	await _save("05_park")

	get_tree().quit(0)


## Presses each movement key for real, through the input map, and reports how
## far the player actually travelled. This is the movement check.
func _drive() -> void:
	DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path("res://shots"))
	await _settle(4)
	var keys := [
		["W", KEY_W], ["A", KEY_A], ["S", KEY_S], ["D", KEY_D],
		["Up", KEY_UP], ["Left", KEY_LEFT], ["Down", KEY_DOWN], ["Right", KEY_RIGHT],
	]
	var i := 0
	for k in keys:
		var before := player_at
		var samples: Array[Vector2] = []
		var ev := InputEventKey.new()
		ev.physical_keycode = k[1]
		ev.keycode = k[1]
		ev.pressed = true
		Input.parse_input_event(ev)
		for f in range(26):
			await get_tree().process_frame
			samples.append(player_at)
		var up := InputEventKey.new()
		up.physical_keycode = k[1]
		up.keycode = k[1]
		up.pressed = false
		Input.parse_input_event(up)
		await _settle(2)
		var moved := player_at - before
		# smoothness: every sampled step should be a small non-zero fraction
		var steps: Array[float] = []
		for n in range(1, samples.size()):
			steps.append((samples[n] - samples[n - 1]).length())
		steps.sort()
		print("key %-5s moved world %+.3f,%+.3f  screen %+.1f,%+.1f  step min/med/max %.4f/%.4f/%.4f"
			% [k[0], moved.x, moved.y,
			Iso.to_screen(player_at).x - Iso.to_screen(before).x,
			Iso.to_screen(player_at).y - Iso.to_screen(before).y,
			steps[0], steps[steps.size() / 2], steps[steps.size() - 1]])
		i += 1
		if i == 4:
			await _save("06_after_wasd")
	await _save("07_after_arrows")
	print("drive complete")
	get_tree().quit(0)
