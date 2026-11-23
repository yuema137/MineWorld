extends SceneTree
## The 2D seated drawing over a fixture pack, headless (S34-e AC-e-3, AC-e-4):
##   godot --headless --path clients/2d --script res://scripts/harness/seated_sprite_check.gd
## Crop with and without a stature, a seated sheet and a four-view figure; prints PASS or each FAIL.
const People := preload("res://scripts/scene/people.gd")
const Town := preload("res://scripts/town.gd")
const PlanProjection := preload("res://scripts/projection.gd")
var failures := 0

## A one-figure pack: a white 40 x 500 frame anchored at (20, 100), optionally with a seated sheet.
class FixturePack:
	extends RefCounted
	var with_height := true
	var with_seated := false
	var four_views := false
	var texture: Texture2D

	func _init() -> void:
		var image := Image.create(40, 500, false, Image.FORMAT_RGBA8)
		image.fill(Color.WHITE)
		texture = ImageTexture.create_from_image(image)

	func is_bound(_role: String) -> bool:
		return false

	func cast_size() -> int:
		return 1

	func directional(_role: String) -> Dictionary:
		var sprite := {"texture": texture, "scale": 1.0, "anchor": Vector2(20, 100),
			"height_m": 2.0 if with_height else null, "ink": "none"}
		var sheet := sprite.duplicate()
		sheet["sheet"] = {"columns": 1, "views": ["down", "down_right", "right", "up_right", "up"],
			"frames_per_cycle": 0, "cycle_m": 1.0}
		var views := {"front": sprite, "front_b": {}, "back": sprite, "back_b": {},
			"walk": {} if four_views else sheet}
		if with_seated:
			var seated := sheet.duplicate()
			seated["seat_height_m"] = 0.40
			views["seated"] = seated
		return views


func _init() -> void:
	_run.call_deferred()


func _run() -> void:
	for mode in ["crop", "opaque", "sheet", "four"]:
		var pack := FixturePack.new()
		pack.with_height = mode != "opaque"
		pack.with_seated = mode == "sheet"
		pack.four_views = mode == "four"
		var world := Node2D.new()
		root.add_child(world)
		var people := People.new()
		root.add_child(people)
		people.setup(PlanProjection.new("plan", 50.0), pack, Town.new(), world)
		# One standing person at the origin, settled, before the seat is given.
		people._people["1"] = people._make("1", Vector2.ZERO, "")
		var person: Dictionary = people._people["1"]
		person["still_s"] = 1.0
		people._process(0.0)
		var pose := {"point": "fixture", "plan": Vector2(2, 3), "pelvis": Vector2(2000, -3000),
			"direction": Vector2.RIGHT, "seat_height_m": 0.45}
		people.set_seated("1", pose)
		var body: Node2D = person["body"]
		var seated: Sprite2D = body.get_node("seat")
		_check(people.drawn_plan("1") == Vector2(2, 3), mode + ": immediate pelvis placement")
		_check(person["seat"]["view"] == 2, mode + ": facing chooses profile")
		_check(people.pick(people.figure_at("1")) == "1", mode + ": seated figure remains pickable")
		if mode == "sheet":
			_check(seated.frame == 2 and is_equal_approx(seated.position.y, -2.5), "seated atlas row/height")
		else:
			_check(is_equal_approx(seated.position.y, -22.5), mode + ": hip sits on seat surface")
			if mode != "four":
				_check(is_equal_approx(seated.region_rect.size.y, 47.0), mode + ": approved hip crop")
		var region := seated.region_rect
		people.set_player(Vector2(99, 99))
		people._process(0.02)
		_check(seated.region_rect == region and body.rotation == 0.0 and body.position == Vector2.ZERO,
			mode + ": no gait or bob while seated")
		pose["direction"] = Vector2.UP
		pose["plan"] = Vector2(4, 5)
		people.set_seated("1", pose)
		_check(person["seat"]["view"] == 4 and people.drawn_plan("1") == Vector2(4, 5),
			mode + ": same point with changed pose refreshes")
		people.set_seated("1", null)
		_check(not body.has_node("seat") and person["seat"] == null, mode + ": stand removes seated drawing")
		var visible := 0
		for child in body.get_children():
			visible += int(child.visible)
		_check(visible == 1, mode + ": exactly one standing view restored")
		people.free()
		world.free()
	print("seated_sprite_check: %s" % ("PASS" if failures == 0 else "FAIL"))
	quit(0 if failures == 0 else 1)


func _check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		print("FAIL: " + message)
