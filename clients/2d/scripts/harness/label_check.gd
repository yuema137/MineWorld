extends SceneTree

# Overhead names in the 2D client never cover one another (promo v2: names overlapped when people
# stood close). The placement is the shared `MineWorldLabelPlacement` (`clients/shared/labels/`); this
# check seats a crowd of people close together through the client's own `people.gd`, lets the
# captions settle, and measures every shown caption's drawn rectangle on screen.
#
#   godot --headless --path clients/2d --script res://scripts/harness/label_check.gd
#   godot --path clients/2d --script res://scripts/harness/label_check.gd -- --still=<png> [--before]
#
# Prints `PASS <case>` / `FAIL <case>: <why>` and exits 1 on any failure. `--before` skips the
# declutter pass (the evidence still of the defect); `--still` needs a window.

const Presentation := preload("res://scripts/presentation.gd")
const Plan := preload("res://scripts/projection.gd")
const People := preload("res://scripts/scene/people.gd")
## Names of different lengths, one metre or less apart: the crowd of the defect.
const CROWD := {
	"p1": ["Wes Calloway", Vector2(0.0, 0.0)],
	"p2": ["Bob Achterberg", Vector2(0.5, 0.2)],
	"p3": ["Alice Moreau", Vector2(0.9, 0.0)],
	"p4": ["Vera Lindgren", Vector2(0.3, 0.7)],
	"p5": ["Dario Russo", Vector2(0.7, 0.9)],
}

var _fails := 0
var _people
var _world: Node2D


func _initialize() -> void:
	var args := OS.get_cmdline_user_args()
	var before := args.has("--before")
	var still := ""
	for a in args:
		if a.begins_with("--still="):
			still = a.substr(8)
	var presentation := Presentation.new()
	presentation.load_pack("", "")
	_world = Node2D.new()
	root.add_child(_world)
	var camera := Camera2D.new()
	camera.zoom = Vector2.ONE * 2.0
	camera.position = Plan.new().to_screen(Vector2(0.45, 0.45))
	root.add_child(camera)
	var projection := Plan.new()
	_people = People.new()
	root.add_child(_people)
	_people.setup(projection, presentation, null, _world)
	_people.observer = "p1"
	for id in CROWD:
		var person: Dictionary = _people._make(id, CROWD[id][1], "")
		_people._people[id] = person
		person["label"].text = CROWD[id][0]
		person["node"].position = projection.to_screen(CROWD[id][1])
	_people.set_process(false)
	# the camera takes effect on the first frames: measure with the zoom the player sees
	for _i in 4:
		await process_frame
	if not before:
		for _i in 60:
			_people._declutter(0.1)
	_report()
	if still != "":
		for _i in 4:
			await process_frame
		await RenderingServer.frame_post_draw
		root.get_viewport().get_texture().get_image().save_png(still)
	print("label_check: ", "FAIL" if _fails > 0 else "PASS")
	quit(1 if _fails > 0 else 0)


func _report() -> void:
	var rects := {}
	for id in CROWD:
		var l: Label = _people._people[id]["label"]
		if l.modulate.a < 0.5:
			continue
		rects[id] = _drawn(_people._people[id]["node"], l)
	var own: Label = _people._people["p1"]["label"]
	_check(not own.has_meta("base_y") or absf(own.position.y - float(own.get_meta("base_y"))) < 0.5,
		"the observer's name stays where it belongs", "it moved")
	var overlaps: Array = []
	for a in rects:
		for b in rects:
			if str(a) < str(b) and (rects[a] as Rect2).intersects(rects[b]):
				overlaps.append("%s/%s" % [a, b])
	_check(overlaps.is_empty(), "no two shown names overlap", str(overlaps))
	_check(rects.size() >= 2, "a crowd still shows the nearest names (%d of %d)" % [rects.size(), CROWD.size()], str(rects.keys()))


## The rectangle a label's text covers on screen, in pixels.
func _drawn(node: Node2D, l: Label) -> Rect2:
	var to_screen := node.get_global_transform_with_canvas()
	var font := l.get_theme_font("font")
	var text := font.get_string_size(l.text, HORIZONTAL_ALIGNMENT_LEFT, -1, l.get_theme_font_size("font_size"))
	var local := Rect2(Vector2(l.position.x + (l.size.x - text.x) * 0.5, l.position.y + (l.size.y - text.y) * 0.5), text)
	return to_screen * local


func _check(ok: bool, name: String, why: String) -> void:
	if ok:
		print("PASS ", name)
	else:
		_fails += 1
		print("FAIL %s: %s" % [name, why])
