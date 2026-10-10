extends Node

## The people the newest observation lists, drawn where it puts them (`I-7`, AC-W2).
##
## Reconciliation, not simulation: a person appears when an observation lists them, moves toward
## where the latest one puts them, and is removed when one no longer lists them. Nobody is moved by
## this script except toward the server's position; the player's own body is the walker's
## prediction and is handed in by [method set_player].
##
## Gait is the spike's: driven by distance travelled, never by the clock (`STRIDE_M`), with two
## stride poses per view.

const Sprites := preload("res://scripts/scene/sprites.gd")
const Words := preload("res://scripts/hud/words.gd")

## A click this close to a drawn figure picks that person: half its width, in screen pixels at zoom 1
## (drawing only, step-13 D-b-1).
const PICK_HALF_WIDTH_PX := 15.0

## How far a person travels between one footfall and the next.
const STRIDE_M := 0.72
## Smoothing toward the server's position, per second; a bigger jump than SNAP_M is not smoothed.
const FOLLOW_RATE := 9.0
const SNAP_M := 4.0
## Below this, a person is where the server put them (AC-W2's 1 mm, with headroom for float).
const SETTLED_M := 0.0005

var projection
var presentation
var town
var sprites
## The node the people are added to: the y-sorted world.
var world: Node2D
var observer := ""

## id string → {node, body, label, plan, target, phase, view_away, flip}
var _people: Dictionary = {}


func setup(the_projection, the_presentation, the_town, the_world: Node2D) -> void:
	projection = the_projection
	presentation = the_presentation
	town = the_town
	world = the_world
	sprites = Sprites.new(presentation)


## Reconciles the drawn people with one observation.
func reconcile(observation: MineWorldObservation) -> void:
	var listed := {}
	var here := observation.place()
	for entity in observation.entities():
		if String(entity.get("entity_type", "")) != "person":
			continue
		var id := String(entity.get("id", ""))
		var location: Variant = entity.get("location")
		if typeof(location) != TYPE_DICTIONARY:
			continue
		var place := String(location.get("place", {}).get("entity", "")) if typeof(location.get("place")) == TYPE_DICTIONARY else here
		var plan: Variant = town.to_plan(place, location.get("local"))
		if plan == null:
			continue
		listed[id] = true
		if not _people.has(id):
			_people[id] = _make(id, plan)
		var person: Dictionary = _people[id]
		person["target"] = plan
		var name := observation.display_name(id)
		person["label"].text = name if name != "" else id
		# The participation marker: drawn from the disclosure about this person, never from what this
		# client did (step-13 D-b-7, AC-I3).
		var taking_part: Variant = observation.component_value(id, "participation")
		var kind: Variant = taking_part.get("kind") if typeof(taking_part) == TYPE_DICTIONARY else null
		person["activity"] = null if kind == null else str(kind)
		person["marker"].text = "" if kind == null else Words.text("ui.participation", {"kind": str(kind)})
	for id in _people.keys():
		if not listed.has(id):
			_people[id]["node"].queue_free()
			_people.erase(id)


## The walker's body for the observer: drawn there, whatever the last observation said.
func set_player(plan: Vector2) -> void:
	if _people.has(observer):
		_people[observer]["target"] = plan
		_people[observer]["snap"] = true


func _process(delta: float) -> void:
	var dt := minf(delta, 1.0 / 30.0)
	for id in _people:
		var person: Dictionary = _people[id]
		var from: Vector2 = person["plan"]
		var to: Vector2 = person["target"]
		var next := to
		if not person.get("snap", false) and from.distance_to(to) < SNAP_M:
			next = from.lerp(to, 1.0 - exp(-FOLLOW_RATE * dt))
			if next.distance_to(to) < SETTLED_M:
				next = to
		person["snap"] = false
		person["plan"] = next
		person["node"].position = projection.to_screen(next)
		_animate(person, next - from)


## Every drawn person's id.
func drawn_ids() -> PackedStringArray:
	return PackedStringArray(_people.keys())


## Where a person is drawn, read back from the node through the inverse projection (AC-W2).
func drawn_plan(id: String) -> Variant:
	if not _people.has(id):
		return null
	return projection.to_plan(_people[id]["node"].position)


func label_of(id: String) -> String:
	return _people[id]["label"].text if _people.has(id) else ""


## The activity kind a drawn person's marker shows, or null.
func activity_of(id: String) -> Variant:
	return _people[id].get("activity") if _people.has(id) else null


## The person drawn under `at` (a point in the world node's frame), or "": the front-most figure whose
## drawn outline holds it.
func pick(at: Vector2) -> String:
	var picked := ""
	var front := -INF
	for id in _people:
		var person: Dictionary = _people[id]
		var foot: Vector2 = person["node"].position
		var top: float = person.get("top", 34.0)
		if absf(at.x - foot.x) <= PICK_HALF_WIDTH_PX and at.y >= foot.y - top - 6.0 and at.y <= foot.y + 8.0 \
				and foot.y > front:
			picked = id
			front = foot.y
	return picked


## Where a drawn person's figure is, in the world node's frame: the middle of its body.
func figure_at(id: String) -> Variant:
	if not _people.has(id):
		return null
	var person: Dictionary = _people[id]
	return person["node"].position - Vector2(0, float(person.get("top", 34.0)) * 0.5)


## The real height a person's sprite was normalized to, or null (plain drawing, or not recorded).
func intended_height_m(id: String) -> Variant:
	return _people[id].get("height_m") if _people.has(id) else null


## A drawn person's height on screen at zoom 1, in pixels: the opaque rows of the sprite actually
## drawn, times its draw scale (AC-W12).
func drawn_height_px(id: String) -> float:
	if not _people.has(id):
		return 0.0
	var body: Node2D = _people[id]["body"]
	for child in body.get_children():
		if child is Sprite2D and child.visible:
			var trimmed := _opaque_height(child.texture)
			return trimmed * absf(child.scale.y)
	return 0.0


## Whether `a` is drawn behind `b`: in the y-sorted world, the one whose ground point is higher on
## the screen is drawn first.
func drawn_behind(a: String, b: String) -> bool:
	return world.y_sort_enabled and _people[a]["node"].position.y < _people[b]["node"].position.y


func _make(id: String, plan: Vector2) -> Dictionary:
	var node := Node2D.new()
	node.name = "person_%s" % id
	node.set_meta("person", id)
	var body := Node2D.new()
	body.name = "body"
	node.add_child(body)
	var is_player := id == observer
	var role := "player" if is_player else "person:%d" % _cast_index(id)
	var views: Dictionary = presentation.directional(role)
	var top := 34.0
	if views.is_empty():
		var shape: Node2D = Sprites.plain("player" if is_player else "person", Vector2(9, 9),
			Color(0.95, 0.6, 0.3) if is_player else _plain_colour(id))
		shape.name = "front"
		body.add_child(shape)
	else:
		for view in ["front", "front_b", "back", "back_b"]:
			var s: Sprite2D = sprites.make(views[view])
			if s == null:
				continue
			s.name = view
			s.visible = view == "front"
			body.add_child(s)
			if view == "front":
				top = float(s.texture.get_height()) * absf(s.scale.y)
		if is_player:
			node.add_child(Sprites.plain("ring", Vector2(26, 10)))
			node.get_child(node.get_child_count() - 1).z_index = -1
	var label := Label.new()
	label.name = "label"
	label.add_theme_font_size_override("font_size", 13)
	label.add_theme_color_override("font_color", Color("3b2c1e"))
	label.add_theme_color_override("font_outline_color", Color(1, 0.98, 0.93, 0.9))
	label.add_theme_constant_override("outline_size", 5)
	label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	label.size = Vector2(160, 18)
	label.position = Vector2(-80, -top - 22)
	# A name the world disclosed is world content: never translated, never UI text (S20 SD-SET-a-9).
	MineWorldText.mark_world_text(label)
	node.add_child(label)
	var marker := Label.new()
	marker.name = "marker"
	marker.add_theme_font_size_override("font_size", 12)
	marker.add_theme_color_override("font_color", Color("8a3f1c"))
	marker.add_theme_color_override("font_outline_color", Color(1, 0.98, 0.93, 0.9))
	marker.add_theme_constant_override("outline_size", 5)
	marker.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	marker.size = Vector2(160, 16)
	marker.position = Vector2(-80, -top - 38)
	marker.auto_translate_mode = Node.AUTO_TRANSLATE_MODE_DISABLED
	node.add_child(marker)
	world.add_child(node)
	var height_m: Variant = null if views.is_empty() else views["front"].get("height_m")
	return {"node": node, "body": body, "label": label, "marker": marker, "plan": plan, "target": plan,
		"phase": float(posmod(id.hash(), 628)) / 100.0, "away": false, "snap": true,
		"height_m": height_m, "top": top, "activity": null}


func _cast_index(id: String) -> int:
	var size: int = presentation.cast_size()
	return 0 if size == 0 else posmod(id.hash(), size)


func _plain_colour(id: String) -> Color:
	return Color.from_hsv(float(posmod(id.hash(), 360)) / 360.0, 0.35, 0.75)


## Walk cycle, driven by distance travelled (the spike's `_animate`).
func _animate(person: Dictionary, moved: Vector2) -> void:
	var body: Node2D = person["body"]
	var metres := moved.length()
	var moving := metres > 0.00001
	var phase: float = person["phase"]
	if moving:
		phase += (metres / STRIDE_M) * PI
		person["phase"] = phase
		var screen_dir: Vector2 = projection.plan_dir_to_screen(moved)
		person["away"] = screen_dir.y < -0.25 * screen_dir.length()
		if absf(screen_dir.x) > 0.01:
			body.scale.x = signf(screen_dir.x)
	body.position = Vector2(0, (absf(sin(phase)) * -3.2) if moving else 0.0)
	body.rotation = (sin(phase) * 0.035) if moving else 0.0
	var away: bool = person["away"] and moving
	var step_b := moving and int(floor(phase / PI)) % 2 != 0
	var has_back := body.has_node("back")
	for child in body.get_children():
		var view := String(child.name)
		var wants_back := away and has_back
		var is_back := view.begins_with("back")
		var is_b := view.ends_with("_b")
		var has_b := body.has_node(("back" if wants_back else "front") + "_b")
		child.visible = (is_back == wants_back) and (is_b == (step_b and has_b))
	if body.get_child_count() > 0 and body.get_child(0) is Node2D and body.get_child(0).get("facing") != null:
		body.get_child(0).facing = projection.plan_dir_to_screen(moved) if moving else body.get_child(0).facing


static func _opaque_height(texture: Texture2D) -> float:
	var image := texture.get_image()
	if image == null:
		return float(texture.get_height())
	var used := image.get_used_rect()
	return float(used.size.y)
