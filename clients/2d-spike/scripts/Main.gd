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
const ShadowScript := preload("res://scripts/Shadows.gd")
const GradeScript := preload("res://scripts/Grade.gd")
const RingScript := preload("res://scripts/Ring.gd")

const ART := "res://art/svg/%s.svg"
const ART_GEN := "res://art/generated/%s.png"

## Art variants. The spike now has two sources of sprites — the procedural SVG
## generator and a set generated from the reference plates — and the point of
## the variants is that a person can look at whole scenes and pick one, rather
## than judging assets on a contact sheet. `ARC-9` puts that call at the
## integrated-scene level, not per asset.
##
##   ./mineworld-2d --variant=procedural   everything from gen_art.py
##   ./mineworld-2d --variant=people       generated cast, procedural world
##   ./mineworld-2d --variant=full         generated cast, shopfronts and flora
const VARIANTS := ["procedural", "people", "full"]
var variant := "full"

## Scale applied to each 2x-authored sprite. Derived from one rule: a person is
## 1.75 m and 1 m is Iso.PX_PER_M_Z pixels, so a person is ~56 px tall and
## everything else is sized against that.
const SCALE := {
	"shop": 0.5, "person": 0.34, "dog": 0.30, "bird": 0.26,
	"lamppost": 0.27, "bench": 0.26, "bench_r": 0.26,
	"planter": 0.28, "planter_b": 0.28, "pot": 0.26, "pot_b": 0.26,
	"chalkboard": 0.26, "signpost": 0.30, "cafeset": 0.30,
	"bicycle": 0.24, "barrel": 0.24, "fountain": 0.34,
	"rail_x": 0.5, "rail_y": 0.5, "jetty": 0.5, "boat": 0.30,
}

var props: Dictionary = {}
var world: Node2D
var player: Node2D
var player_at := Vector2(6.6, 5.1)
var player_facing := 1.0
var walkers: Array = []
var cam: Camera2D
var _jit := RandomNumberGenerator.new()
var _ink_shader: Shader
var _ink_mats: Dictionary = {}   # texel width -> shared ShaderMaterial
var shadows: Node2D
var grade: CanvasLayer

const WALK_MIN := Vector2(1.7, 0.3)
const WALK_MAX := Vector2(16.8, 6.66)
const PLAYER_SPEED := 2.4   # world units/second (~4.8 m/s, a brisk walk in a demo)


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--variant="):
			var v := a.substr(10)
			if VARIANTS.has(v):
				variant = v
			else:
				push_warning("unknown variant '%s'; using '%s'" % [v, variant])
	print("variant: ", variant)
	_jit.seed = 20260926
	props = _load_props()
	_ink_shader = load("res://art/ink.gdshader")
	world = Node2D.new()
	world.y_sort_enabled = true
	add_child(world)

	var ground := Node2D.new()
	ground.set_script(GroundScript)
	add_child(ground)

	shadows = Node2D.new()
	shadows.set_script(ShadowScript)
	add_child(shadows)

	_build_places()
	_build_scenery()
	_build_people()

	cam = Camera2D.new()
	cam.zoom = Vector2(1.55, 1.55)
	cam.position_smoothing_enabled = true
	cam.position_smoothing_speed = 4.0
	cam.position = Iso.to_screen(player_at)
	add_child(cam)
	cam.make_current()

	grade = CanvasLayer.new()
	grade.set_script(GradeScript)
	add_child(grade)

	_build_hud()

	var args := OS.get_cmdline_user_args()
	if args.has("--drive"):
		_drive.call_deferred()
	elif args.has("--shots"):
		_shots.call_deferred()


func _load_props() -> Dictionary:
	var f := FileAccess.open("res://art/props.json", FileAccess.READ)
	var p: Dictionary = JSON.parse_string(f.get_as_text())
	# Generated sprites carry their own manifest, written by tools/normalize.gd
	# when it cuts each candidate to its bounding box. Merging rather than
	# replacing means a variant can mix the two sources freely.
	var g := FileAccess.open("res://art/generated/generated.json", FileAccess.READ)
	if g != null:
		var gen: Dictionary = JSON.parse_string(g.get_as_text())
		for k in gen:
			p[k] = gen[k]
	return p


func _is_generated(name: String) -> bool:
	return props.has(name) and props[name].get("source", "") == "generated"


func _art(name: String) -> String:
	return (ART_GEN if _is_generated(name) else ART) % name


## Which sprite actually gets drawn for a role, given the variant. Roles are
## the scene's vocabulary — "npc_b", "shop_cafe" — and stay put; only what they
## resolve to changes.
func _role(name: String) -> String:
	var m: Dictionary = ROLE_GEN.get(variant, {})
	var to: String = m.get(name, "")
	# A directional character resolves to a base name that is not itself a
	# prop: only <base>_front and <base>_back exist. Accept either shape, or
	# every walker silently falls back while seated figures do not.
	if to != "" and (props.has(to) or props.has(to + "_front")):
		return to
	# a generated stand-in for a role that has none falls back to the original
	if variant == "full" and (props.has("gen_" + name)
			or props.has("gen_" + name + "_front")):
		return "gen_" + name
	return name


const ROLE_PEOPLE := {
	"player": "gen_player", "npc_a": "gen_a", "npc_b": "gen_b",
	"npc_c": "gen_c", "npc_d": "gen_d", "npc_e": "gen_e",
	"npc_f": "gen_f", "npc_g": "gen_g", "npc_h": "gen_h",
	"npc_i": "gen_i", "npc_j": "gen_j",
	"seated_a": "gen_sit_a", "seated_b": "gen_sit_b",
	"dog": "gen_dog", "cat": "gen_cat",
}
## The procedural set has five walkers and no cat, but crowd density must not
## differ between variants or the comparison stops being about art. The extra
## roles reuse existing procedural sprites.
const ROLE_PROC := {
	"npc_f": "npc_a", "npc_g": "npc_b", "npc_h": "npc_c",
	"npc_i": "npc_d", "npc_j": "npc_e", "cat": "dog",
}
const ROLE_GEN := {
	"people": ROLE_PEOPLE, "full": ROLE_PEOPLE, "procedural": ROLE_PROC,
}


## Vegetation is scaled from its authored height to a target height in metres,
## rather than carrying a hand-tuned number per sprite. Nine tree sprites drawn
## at nine different pixel heights have to agree about how tall a tree is, and
## the only way that stays true as sprites are added is to derive it.
const VEG_M := {
	"tree_a": 6.4, "tree_b": 6.0, "tree_c": 5.2, "tree_d": 5.0, "tree_e": 7.0,
	"tree_f": 5.6, "tree_g": 7.4, "tree_h": 4.6, "tree_i": 6.2,
	"bush_a": 1.45, "bush_b": 1.20, "bush_c": 1.30, "bush_d": 1.75,
	"hedge": 1.05,
}

## The species pool the scenery draws from. Ordering is deliberate: the belts
## walk this list so neighbours differ in silhouette, not just in jitter.
const TREES := ["tree_a", "tree_d", "tree_b", "tree_g", "tree_c", "tree_e",
	"tree_h", "tree_f", "tree_i"]
const BUSHES := ["bush_a", "bush_c", "bush_b", "bush_d"]


func _scale_for(name: String) -> float:
	# Generated sprites were cut to their bounding box and resized so that
	# pixel height is height_m * Iso.PX_PER_M_Z * 2. That makes their draw
	# scale exactly one half, for every one of them, with no per-sprite tuning.
	if _is_generated(name):
		return 0.5
	if VEG_M.has(name):
		var m: Dictionary = props[name]
		return VEG_M[name] * Iso.PX_PER_M_Z / float(m["h"])
	if SCALE.has(name):
		return SCALE[name]
	if name.begins_with("shop_"):
		return SCALE["shop"]
	if name.begins_with("seated_") or name.begins_with("npc_") or name.begins_with("player"):
		return SCALE["person"]
	return 0.3


## The ink contour for a sprite drawn at scale `k`.
##
## The line should be about the same thickness on screen everywhere, so its
## width in texels is the screen width divided by the draw scale. Materials are
## shared between props that land on the same width, which collapses a few
## hundred sprites onto a handful of materials.
const INK_PX := 1.8        # target contour thickness, screen pixels


## How hard the contour bites, per prop class. Foliage and shopfronts already
## draw their own line work in the SVG — foliage a dark union silhouette,
## shopfronts a value break at every junction — so inking them at full
## strength draws the line twice and turns them to mud.
const INK_BITE := {"veg": 0.20, "shop": 0.26, "prop": 0.46, "none": 0.0}


func _ink_class(name: String) -> String:
	# Generated sprites are drawn with their own contour already. Inking them
	# again thickens every edge and loses the line's variation, which is one of
	# the things that makes them read as drawn rather than traced.
	if _is_generated(name):
		return "none"
	if name.begins_with("tree") or name.begins_with("bush") or name == "hedge":
		return "veg"
	if name.begins_with("shop_"):
		return "shop"
	return "prop"


func _ink_material(k: float, cls := "prop") -> ShaderMaterial:
	if cls == "none":
		return null
	var texels := clampf(INK_PX / maxf(k, 0.02), 1.5, 14.0)
	var key := "%s:%.1f" % [cls, roundf(texels * 2.0) / 2.0]
	if _ink_mats.has(key):
		return _ink_mats[key]
	var m := ShaderMaterial.new()
	m.shader = _ink_shader
	m.set_shader_parameter("width", roundf(texels * 2.0) / 2.0)
	m.set_shader_parameter("strength", INK_BITE.get(cls, 0.46))
	_ink_mats[key] = m
	return m


## One prop, anchored on the ground so the painter's sort is correct.
func _prop(name: String, at: Vector2, flip := false, z := 0) -> Node2D:
	var holder := Node2D.new()
	holder.position = Iso.to_screen(at)
	holder.z_index = z
	var sprite := _role(name)
	var s := Sprite2D.new()
	s.texture = load(_art(sprite))
	s.centered = false
	var m: Dictionary = props[sprite]
	s.offset = Vector2(-float(m["ax"]), -float(m["ay"]))
	var k := _scale_for(sprite)
	# Vegetation gets a little size and hue jitter, so a dozen copies of one
	# tree do not read as a dozen copies of one tree.
	if name.begins_with("tree") or name.begins_with("bush") or name == "hedge":
		k *= 0.82 + _jit.randf() * 0.40
		# Species now carry the variety, so the per-instance tint is a light
		# touch — enough to break identical neighbours, not enough to undo the
		# measured foliage ramp.
		s.modulate = Color(0.93 + _jit.randf() * 0.14,
			0.95 + _jit.randf() * 0.10, 0.90 + _jit.randf() * 0.16)
	s.scale = Vector2(-k if flip else k, k)
	s.material = _ink_material(k, _ink_class(sprite))
	if flip:
		s.offset.x = -s.offset.x - float(m["w"]) + 2.0 * float(m["ax"])
	holder.add_child(s)
	world.add_child(holder)
	_cast_shadow(name, holder.position, k)
	return holder


## Sun is fixed upper-left; everything with height throws a soft blob.
func _cast_shadow(name: String, at: Vector2, k: float) -> void:
	if shadows == null:
		return
	if name.begins_with("tree"):
		# One broad soft pool per tree, dappled, rather than four hard blobs.
		var r := 52.0 * k * 2.2
		shadows.add(at, r, r * 0.46, 64.0 * k * 2.2, 0.46, true)
		shadows.add(at + Vector2(r * 0.55, r * 0.16), r * 0.58, r * 0.28,
			50.0 * k * 2.2, 0.26, true)
	elif name.begins_with("bush") or name == "hedge":
		shadows.add(at, 46.0 * k * 2.0, 18.0 * k * 2.0, 26.0 * k * 2.0, 0.40)
	elif name == "lamppost":
		shadows.add(at, 13.0, 6.0, 150.0 * k * 2.0, 0.30)
	elif name.begins_with("shop_"):
		shadows.add(at + Vector2(-70, -18), 250.0 * k, 96.0 * k, 190.0 * k, 0.38)
	elif name in ["fountain", "cafeset", "bench", "bench_r", "planter", "planter_b",
			"barrel", "bicycle", "signpost", "chalkboard", "pot", "pot_b"]:
		shadows.add(at, 36.0 * k * 1.6, 14.0 * k * 1.6, 26.0 * k * 1.6, 0.38)


## Squash the square's depth. The references are streets, not fields: the
## shopfront row, its terrace and the greenery opposite should fill the frame
## with very little bare paving between them. Everything from the terrace
## backwards is pulled toward the shops by DEPTH.
const DEPTH := 0.60
const DEPTH_FROM := 1.2


func _wp(x: float, y: float) -> Vector2:
	return Vector2(x, y if y < DEPTH_FROM else DEPTH_FROM + (y - DEPTH_FROM) * DEPTH)


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
	_prop("jetty", _wp(0.6, 6.6), false, -2)
	_prop("boat", _wp(-1.6, 7.9), false, -1)
	_prop("barrel", _wp(-0.3, 5.1))
	_prop("bird", _wp(-3.2, 3.2), false, -1)
	_prop("bird", _wp(-1.2, 5.4), true)
	for i in range(1, 12):
		if i >= 4 and i <= 6:
			continue   # gap where the jetty meets the quay
		_prop("rail_y", _wp(1.18, i + 0.5))

	_prop("fountain", _wp(6.4, 4.6))

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
		_prop(it[0], _wp(it[1], it[2]), it[3])

	# the park corner
	var park := [
		["tree_d", 10.55, 2.45], ["tree_b", 2.05, 1.05], ["tree_c", 6.85, 9.9],
		["tree_a", 14.10, 5.70], ["tree_i", 15.90, 8.55], ["tree_h", 12.70, 8.90],
		["tree_g", 16.90, 3.10],
		["bush_a", 12.45, 5.10], ["bush_c", 13.70, 8.05], ["bush_d", 15.30, 6.50],
		["bush_b", 12.10, 9.70], ["bush_c", 16.70, 9.10], ["bush_a", 15.05, 4.75],
		["hedge", 12.05, 6.10], ["hedge", 12.05, 6.80], ["hedge", 12.05, 7.50],
		["bench", 13.35, 6.20], ["bench_r", 15.65, 9.70],
	]
	for it in park:
		_prop(it[0], _wp(it[1], it[2]))

	# Deep foliage belts on every side. The world has to run off the edge of
	# the frame; a visible map boundary is what made this read as a diorama.
	var edge: Array = []
	var b := 0.0
	while b < 3.0:                      # south
		var xx := -2.0
		while xx < 24.0:
			edge.append([TREES[int(xx * 3.0 + b * 2.0) % TREES.size()],
				xx + b * 0.7, 11.9 + b * 2.30 + sin(xx * 1.7) * 0.6])
			if b < 1.0:
				edge.append([BUSHES[int(xx) % BUSHES.size()],
					xx + 0.9, 11.3 + b * 2.30])
			xx += 1.95
		b += 1.0
	var e := 0.0
	while e < 3.0:                      # east
		var yy := -3.0
		while yy < 18.0:
			edge.append([TREES[int(yy * 2.0 + e * 5.0 + 4.0) % TREES.size()],
				18.4 + e * 1.45 + sin(yy) * 0.5, yy + e * 0.7])
			if e < 1.0:
				edge.append([BUSHES[int(yy + 1.0) % BUSHES.size()],
					17.9 + e * 1.45, yy + 0.9])
			yy += 1.85
		e += 1.0
	var n := 0.0
	while n < 3.0:                      # behind the shop row
		var xn := -1.0
		while xn < 20.0:
			edge.append([TREES[int(xn * 2.0 + n * 3.0) % TREES.size()],
				xn, -4.6 - n * 1.3])
			xn += 1.35
		n += 1.0
	for it in edge:
		_prop(it[0], _wp(it[1], it[2]))

	# Foreground foliage: trees nearer the camera than the player can walk,
	# so canopies break into the bottom of the frame.
	var fg := ["tree_a", "tree_d", "tree_i", "tree_b"]
	for i in range(4):
		var fgx: float = [1.8, 5.4, 10.2, 15.6][i]
		_prop(fg[i], _wp(fgx, 11.1 + sin(fgx) * 0.2))

	# Density near the buildings: clusters, not singles.
	var fill := [
		["planter", 3.10, 4.10], ["planter_b", 3.10, 5.30], ["planter", 3.10, 6.50],
		["planter_b", 9.90, 5.60], ["planter", 9.90, 6.80], ["planter_b", 9.90, 4.40],
		["bush_c", 4.60, 8.60], ["bush_a", 6.20, 8.30], ["bush_d", 8.20, 8.90],
		["tree_h", 4.30, 7.40], ["tree_f", 9.10, 9.10], ["tree_c", 1.95, 4.40],
		["pot_b", 1.60, 6.60], ["pot", 1.60, 7.70], ["pot_b", 1.60, 3.30],
		["bench", 7.90, 3.55], ["bench_r", 8.40, 6.90],
		["planter", 12.60, 1.60], ["planter_b", 14.20, 1.80],
		["pot", 13.30, 3.80], ["bush_d", 14.60, 3.40], ["pot_b", 15.60, 2.20],
		["barrel", 2.55, 9.10], ["bicycle", 4.85, 0.35],
		["cafeset", 5.55, 0.95], ["cafeset", 11.40, 0.95],
	]
	# clusters of pots and chalkboards flanking every shopfront
	for sx in [1.15, 5.30, 9.10, 12.85, 16.35]:
		fill.append(["pot", sx, -0.30])
		fill.append(["pot_b", sx + 0.32, -0.16])
		fill.append(["pot", sx + 0.12, -0.02])
		fill.append(["bush_b" if int(sx) % 2 == 0 else "bush_c", sx + 0.62, -0.26])
		fill.append(["chalkboard", sx + 1.00, -0.14])
		fill.append(["planter_b", sx + 1.48, -0.24])
	for it in fill:
		_prop(it[0], _wp(it[1], it[2]))


func _build_people() -> void:
	# Seated people: the Sit affordance in 2D form, as the references show it.
	_prop("seated_a", _at_screen(Vector2(2.30, 0.55), Vector2(-38, 2)))
	_prop("seated_b", _at_screen(Vector2(3.95, 1.05), Vector2(39, 2)))

	var crowd := [
		Demo.DemoPerson.new(&"a", &"npc_a", _wp(4.2, 6.4),
			PackedVector2Array([_wp(4.2, 6.4), _wp(9.6, 6.9)]), 0.80),
		Demo.DemoPerson.new(&"b", &"npc_b", _wp(8.0, 8.6),
			PackedVector2Array([_wp(8.0, 8.6), _wp(3.4, 8.9)]), 0.70),
		Demo.DemoPerson.new(&"c", &"npc_c", _wp(10.8, 4.1),
			PackedVector2Array([_wp(10.8, 4.1), _wp(10.9, 8.6)]), 0.75),
		Demo.DemoPerson.new(&"d", &"npc_d", _wp(5.8, 2.1),
			PackedVector2Array([_wp(5.8, 2.1), _wp(12.8, 2.3)]), 0.62),
		Demo.DemoPerson.new(&"e", &"npc_e", _wp(2.6, 9.6),
			PackedVector2Array([_wp(2.6, 9.6), _wp(2.7, 4.6)]), 0.55),
		Demo.DemoPerson.new(&"dog", &"dog", _wp(6.4, 7.6),
			PackedVector2Array([_wp(6.4, 7.6), _wp(8.8, 7.0), _wp(7.2, 9.0)]), 1.25),
		# The plates are busy. Five people in a square this size read as a town
		# that has been evacuated.
		Demo.DemoPerson.new(&"f", &"npc_f", _wp(13.6, 2.2),
			PackedVector2Array([_wp(13.6, 2.2), _wp(16.4, 4.4)]), 0.66),
		Demo.DemoPerson.new(&"g", &"npc_g", _wp(3.1, 3.4),
			PackedVector2Array([_wp(3.1, 3.4), _wp(3.3, 7.8)]), 0.52),
		Demo.DemoPerson.new(&"h", &"npc_h", _wp(7.4, 1.3),
			PackedVector2Array([_wp(7.4, 1.3), _wp(10.4, 1.4)]), 0.48),
		Demo.DemoPerson.new(&"i", &"npc_i", _wp(11.4, 6.6),
			PackedVector2Array([_wp(11.4, 6.6), _wp(8.2, 5.2), _wp(11.0, 4.0)]), 0.95),
		Demo.DemoPerson.new(&"j", &"npc_j", _wp(15.2, 7.4),
			PackedVector2Array([_wp(15.2, 7.4), _wp(11.8, 8.6)]), 0.58),
		Demo.DemoPerson.new(&"cat", &"cat", _wp(2.4, 6.2),
			PackedVector2Array([_wp(2.4, 6.2), _wp(2.6, 8.4), _wp(4.0, 7.4)]), 0.62),
	]
	for c in crowd:
		walkers.append(_make_actor(c))

	player = _make_actor(Demo.DemoPerson.new(&"player", &"player", player_at))
	player.set_meta("is_player", true)
	if _is_generated(_role("player") + "_front"):
		# The procedural player carries a warm ring drawn into its own sprite.
		# The generated one does not, so the ring is drawn here instead —
		# the player has to stay findable in a crowd of eleven.
		var ring := Node2D.new()
		ring.set_script(RingScript)
		player.add_child(ring)
		player.move_child(ring, 0)


func _make_actor(p) -> Node2D:
	var holder := Node2D.new()
	holder.position = Iso.to_screen(p.at)
	var body := Node2D.new()
	holder.add_child(body)
	var base := _role(String(p.sprite))
	var has_back := props.has(base + "_back")
	for suffix in (["_front", "_back"] if has_back else [""]):
		var s := Sprite2D.new()
		var nm: String = base + suffix
		s.texture = load(_art(nm))
		s.centered = false
		var m: Dictionary = props[nm]
		s.offset = Vector2(-float(m["ax"]), -float(m["ay"]))
		var k := _scale_for(nm)
		s.scale = Vector2(k, k)
		s.material = _ink_material(k, _ink_class(nm))
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
	if grade:
		grade.set_world_offset(cam.position if cam else Vector2.ZERO)

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
	if grade and cam:
		grade.set_world_offset(cam.position)
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	img.save_png("res://shots/%s/%s.png" % [variant, name])
	print("shot: ", name)


func _settle(n: int) -> void:
	for i in range(n):
		await get_tree().process_frame


func _shots() -> void:
	DirAccess.make_dir_recursive_absolute(
		ProjectSettings.globalize_path("res://shots/%s" % variant))
	cam.position_smoothing_enabled = false
	set_process(false)

	# wide: the whole square
	cam.zoom = Vector2(0.92, 0.92)
	cam.position = Iso.to_screen(Vector2(8.6, 3.0))
	await _settle(8)
	await _save("01_wide")

	# mid: people on the plaza
	cam.zoom = Vector2(1.5, 1.5)
	cam.position = Iso.to_screen(Vector2(6.8, 3.6))
	await _settle(6)
	await _save("02_mid_npcs")

	# close: the cafe frontage and its terrace
	cam.zoom = Vector2(1.6, 1.6)
	cam.position = Iso.to_screen(Vector2(4.1, 1.5))
	await _settle(6)
	await _save("03_close_cafe")

	# close: quay, jetty and boat
	cam.zoom = Vector2(1.9, 1.9)
	cam.position = Iso.to_screen(Vector2(0.8, 4.0))
	await _settle(6)
	await _save("04_close_quay")

	# the park corner
	cam.zoom = Vector2(1.45, 1.45)
	cam.position = Iso.to_screen(Vector2(13.8, 4.2))
	await _settle(6)
	await _save("05_park")

	# Framed to match references/02_cafe_street.png, so the comparison is
	# like for like. In that plate a person stands about 8% of the frame
	# width; at Iso.PX_PER_M_Z a 1.75 m person is 56 px, so 1600 px of
	# viewport needs zoom ~2.3 to agree. Every earlier shot is wider than
	# any reference plate, which flattered the art in some ways and
	# punished it in others.
	cam.zoom = Vector2(2.3, 2.3)
	cam.position = Iso.to_screen(Vector2(3.6, 1.1))
	await _settle(6)
	await _save("06_ref_framing")

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
