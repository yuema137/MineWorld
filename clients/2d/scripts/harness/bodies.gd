extends Node

## The body steps of the scripted checks (S12 PR 13d, ARC-342): what a Rust test against a real world
## with the `bodies` System Pack asks the player to do, added to `harness/interact.gd`'s steps.
##
## Every target is derived from what the world disclosed, never from a world's constants (I-13d-3): the
## nearest drawn object (of a shape, when one is named), the centre of the place's disclosed floor, the
## nearest disclosed shell solid. The harness names no action type: the test's selectors do.
##
## ```text
## {"walk_near": "nearest" | "ball" | "box" | "<object id>"}   walk to 0.5 m short of that object
##                                             (one walk-to); "<person id>": 0.6 m short of them
## {"click_floor": "centre"}                   (interact.gd) the floor's centre, from `place-shape`
## {"hold_into_solid": seconds}                hold the keys toward the nearest shell solid, then report
## {"self": true}                              report where I am drawn and observed
## {"walk_into": "<tag>"}                      walk into the place a doorway leads to with that tag
## ```
##
## Output: `EVIDENCE {"wall": {solid, observed, drawn, body}}` after a hold; `EVIDENCE {"self": …}`.

## How far short of an object a `walk_near` stops, in metres: clear of a body's 250 mm radius plus a
## ball's or a small box's half-width, within the pack's 800 mm reach.
const NEAR_M := 0.5
## How far short of a person a `walk_near` stops: two bodies' 250 mm radii and 100 mm between them,
## within a shove's 1 000 mm reach (PR 16e).
const PERSON_NEAR_M := 0.6
const SETTLE_S := 1.0

var drive: Node
var app: Node


## One step, or null when it is not a body step.
func step(s: Dictionary) -> Variant:
	if s.has("walk_near"):
		return await _walk_near(String(s["walk_near"]))
	if s.has("hold_into_solid"):
		return await _hold_into_solid(float(s["hold_into_solid"]))
	if s.has("self"):
		drive._report_self()
		return ""
	if s.has("walk_into"):
		return await _walk_into(String(s["walk_into"]))
	return null


## Walks into the place a doorway leads to whose tags hold `tag` (the test names the tag): directly when
## the starting place opens onto it, else out through the starting place's first doorway and in through
## that place's doorway to it — 2.5 m inside, one server-routed walk per leg (the `walk` scenario's way).
func _walk_into(tag: String) -> String:
	for leg in 2:
		var here: String = app.walker.body_place
		if tag in app.town.tags.get(here, PackedStringArray()):
			return ""
		var exits: Array = app.town.passages.get(here, [])
		if exits.is_empty():
			return "%s discloses no doorway" % here
		var chosen: Dictionary = exits[0]
		for p in exits:
			if tag in app.town.tags.get(p["to"], PackedStringArray()):
				chosen = p
		var to: String = chosen["to"]
		var there: Vector2 = app.town.to_plan(to, {"x": chosen["there"].x, "y": chosen["there"].y})
		if tag in app.town.tags.get(to, PackedStringArray()) and app.places.drawn.has(to):
			there += app.places.drawn[to]["in_dir"] * 2.5
		app.walker.walk_to(to, there)
		if not await drive._until(func() -> bool: return not app.walker.is_walking(), drive.TIMEOUT_S):
			return "the walk into %s did not end in %d s" % [to, drive.TIMEOUT_S]
		await drive._seconds(SETTLE_S)
	return "" if tag in app.town.tags.get(app.walker.body_place, PackedStringArray()) \
		else "never reached a place tagged %s (in %s)" % [tag, app.walker.body_place]


## The drawn object nearest the player's body, of `which` shape ("nearest": any), or "". A drawn
## object's id names that object (PR 16e: one step list for both clients names objects by id).
func nearest(which: String) -> String:
	if app.objects.drawn_ids().has(which):
		return which
	var best := ""
	var nearest_m := INF
	for id in app.objects.drawn_ids():
		if which != "nearest" and app.objects.shape_of(id) != which:
			continue
		var d: float = (app.objects.drawn_plan(id) as Vector2).distance_to(app.walker.body_plan)
		if d < nearest_m:
			nearest_m = d
			best = id
	return best


## The centre of the observer's place's disclosed floor, as local millimetres, or {}.
func floor_centre() -> Dictionary:
	var shape: Variant = app.latest.component_value(app.latest.place(), "place-shape")
	if typeof(shape) != TYPE_DICTIONARY or typeof(shape.get("floor")) != TYPE_DICTIONARY:
		return {}
	var lo: Dictionary = shape["floor"].get("min", {})
	var hi: Dictionary = shape["floor"].get("max", {})
	return {"x": int((int(lo.get("x", 0)) + int(hi.get("x", 0))) / 2),
		"y": int((int(lo.get("y", 0)) + int(hi.get("y", 0))) / 2)}


func _walk_near(which: String) -> String:
	if app.people.figure_at(which) != null:
		return await _walk_near_person(which)
	var id := nearest(which)
	if id == "":
		return "no %s object is drawn in %s" % [which, app.latest.place()]
	var at: Vector2 = app.objects.drawn_plan(id)
	var away: Vector2 = (app.walker.body_plan - at).normalized()
	app.walker.walk_to(app.walker.body_place, at + away * NEAR_M)
	if not await drive._until(func() -> bool: return not app.walker.is_walking(), drive.TIMEOUT_S):
		return "the walk to the object %s did not end in %d s" % [id, drive.TIMEOUT_S]
	await drive._seconds(SETTLE_S)
	print("EVIDENCE ", JSON.stringify({"walked_near": {"object": id, "shape": app.objects.shape_of(id),
		"object_local": app.objects.drawn_local(id),
		"body_local": app.town.local_in(app.walker.body_place, app.walker.body_plan)}}))
	return ""


## Walks to PERSON_NEAR_M short of the drawn person `id` (PR 16e: a shove's 1 m reach; the menu's own
## "walk to" stops 1.2 m short, a destination for talking).
func _walk_near_person(id: String) -> String:
	var at: Vector2 = app.people.drawn_plan(id)
	var away: Vector2 = (app.walker.body_plan - at).normalized()
	app.walker.walk_to(app.walker.body_place, at + away * PERSON_NEAR_M)
	if not await drive._until(func() -> bool: return not app.walker.is_walking(), drive.TIMEOUT_S):
		return "the walk to %s did not end in %d s" % [id, drive.TIMEOUT_S]
	await drive._seconds(SETTLE_S)
	print("EVIDENCE ", JSON.stringify({"walked_near": {"person": id,
		"person_local": app.town.local_in(app.walker.body_place, app.people.drawn_plan(id)),
		"body_local": app.town.local_in(app.walker.body_place, app.walker.body_plan)}}))
	return ""


## Holds the arrow keys whose screen direction best points the body at the nearest shell solid's
## centre (a solid naming no furnishing), for `seconds`; then reports the solid, the observed position
## and the drawn one. The test judges them by the plan's bounds.
func _hold_into_solid(seconds: float) -> String:
	var place: String = app.latest.place()
	var shape: Variant = app.latest.component_value(place, "place-shape")
	var solids: Array = shape.get("solids", []) if typeof(shape) == TYPE_DICTIONARY and typeof(shape.get("solids")) == TYPE_ARRAY else []
	var best: Dictionary = {}
	var best_m := INF
	for s in solids:
		if typeof(s) != TYPE_DICTIONARY or String(s.get("furnishing", "")) != "":
			continue
		var centre: Vector2 = _solid_centre(place, s)
		var d: float = centre.distance_to(app.walker.body_plan)
		if d < best_m:
			best_m = d
			best = s
	if best.is_empty():
		return "%s discloses no shell solid" % place
	var want: Vector2 = (_solid_centre(place, best) - app.walker.body_plan).normalized()
	var keys: Array = _keys_toward(want)
	for action in keys:
		Input.action_press(action)
	await drive._seconds(seconds)
	for action in keys:
		Input.action_release(action)
	await drive._until(func() -> bool: return not app.walker.is_walking(), drive.TIMEOUT_S)
	await drive._seconds(SETTLE_S)
	var drawn: Variant = app.people.drawn_plan(app.people.observer)
	print("EVIDENCE ", JSON.stringify({"wall": {"place": place, "solid": best,
		"observed": app.latest.self_location().get("local"),
		"drawn": null if drawn == null else app.town.local_in(place, drawn),
		"body": app.town.local_in(app.walker.body_place, app.walker.body_plan)}}))
	return ""


func _solid_centre(place: String, s: Dictionary) -> Vector2:
	var lo: Dictionary = s.get("min", {})
	var hi: Dictionary = s.get("max", {})
	return app.town.to_plan(place, {"x": (int(lo.get("x", 0)) + int(hi.get("x", 0))) / 2,
		"y": (int(lo.get("y", 0)) + int(hi.get("y", 0))) / 2})


## The walking actions (one or two) whose screen direction best points along `want` (plan).
func _keys_toward(want: Vector2) -> Array:
	var best: Array = []
	var best_dot := -2.0
	var screen_of := {"move_up": Vector2(0, -1), "move_down": Vector2(0, 1), "move_left": Vector2(-1, 0),
		"move_right": Vector2(1, 0)}
	for combo in [["move_up"], ["move_down"], ["move_left"], ["move_right"], ["move_up", "move_left"],
			["move_up", "move_right"], ["move_down", "move_left"], ["move_down", "move_right"]]:
		var screen := Vector2.ZERO
		for action in combo:
			screen += screen_of[action]
		var d: float = app.projection.screen_dir_to_plan(screen).dot(want)
		if d > best_dot:
			best_dot = d
			best = combo
	return best
