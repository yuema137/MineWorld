extends Node

## The loose objects of the observer's place, drawn from the world's data (`ARC-342`;
## `clients/2d/PRESENTATION.md` §3.1, §6).
##
## The `bodies` System Pack discloses, with the place, a listing of the objects lying in it:
## `[{object, shape, at, tags?}]`, a shape being `{ball: <radius mm>}` or `{box: {x, y, z}}` half-extents.
## This draws each listed object at its `at` and nothing else:
##
##     a shape whose role `object.<shape>` the pack binds  -> that sprite, scaled to the footprint
##     otherwise (plain drawing)                           -> a ball as a disc of its radius, a box as
##                                                            its footprint rectangle
##     a shape this file cannot read                       -> a 0.2 m outline square, logged once
##
## Motion is drawing, never prediction (QB-7): a drawing moves only toward a position an observation
## states. When the observation that moved an object carries the owner's `object-moved` fact for it with
## a `path`, the drawing plays the path keyframe by keyframe (Godot's `Tween`, one leg per keyframe,
## PATH_KEYFRAME wall seconds each) and ends exactly on the fact's `to`; otherwise it eases to the new
## `at`. A newer move kills the running tween and starts from where the object is drawn. Wall time
## only: the world's time scale is never read (step-11 SD-E11).
##
## It names no object, place or world (I-13d-3), and decides nothing: a footprint is drawing and
## picking, never a rule (I-13d-1).

const Sprites := preload("res://scripts/scene/sprites.gd")
const Walker := preload("res://scripts/walker.gd")
const Intents := preload("res://scripts/intents.gd")

## The disclosed component this file draws.
const LISTING := "loose-objects"
## The role prefix a shape is drawn by: `object.ball`, `object.box`.
const ROLE_PREFIX := "object."
## Wall seconds per `path` keyframe: the pack's keyframe spacing played back in real time (the 3D
## client's PATH_KEYFRAME, `clients/3d-spike/scripts/slice/object_view.gd`).
const PATH_KEYFRAME := 0.1
## The fallback ease: distance / OBJECT_SPEED, between the walker's glide for one report and 0.8 s
## (the 3D client's numbers).
const OBJECT_SPEED := 4.0
const EASE_SHORTEST := Walker.REPORT_M / Walker.WALK_SPEED
const EASE_LONGEST := 0.8
## The outline square an unreadable shape is drawn as, in metres.
const UNKNOWN_SIDE_M := 0.2
## A plain ball's and box's colours.
const BALL := Color(0.93, 0.93, 0.90)
const BOX := Color(0.60, 0.45, 0.29)
const INK := Color(0.2, 0.17, 0.14)
const DISC_SEGMENTS := 24

var projection
var presentation
var town
var sprites
var world: Node2D

## Every motion drawn, in order: {id, how ("path" | "ease"), from, to, points (the path's keyframes and
## `to`, local mm), visited (the drawn point at the end of each leg, local mm), ended}.
var motions: Array = []

var _place := ""
var _version := -1
## object id -> {node, shape (as disclosed), ref (the reference as received), at (local mm), half
## (footprint half-extents, metres), tween}
var _drawn := {}
var _logged_unknown := false


func setup(the_projection, the_presentation, the_town, the_world: Node2D) -> void:
	projection = the_projection
	presentation = the_presentation
	town = the_town
	world = the_world
	sprites = Sprites.new(presentation)


## One observation: build, move or remove each object the observer's place lists. A listing that is not
## an array draws nothing; a place change, or the town placing places anew, rebuilds every drawing.
func reconcile(observation: MineWorldObservation) -> void:
	var place := observation.place()
	if place != _place or town.version != _version:
		clear()
		_place = place
		_version = town.version
	if place == "" or town.to_plan(place, {"x": 0, "y": 0}) == null:
		return
	var listing: Variant = observation.component_value(place, LISTING)
	var seen := {}
	if typeof(listing) == TYPE_ARRAY:
		for entry in listing:
			if typeof(entry) != TYPE_DICTIONARY:
				continue
			var id := id_of(entry.get("object"))
			if id == "" or typeof(entry.get("at")) != TYPE_DICTIONARY:
				continue
			seen[id] = true
			var at := _mm(entry["at"])
			if not _drawn.has(id):
				_build(id, entry.get("shape"), entry.get("object"), at)
			elif _drawn[id]["at"] != at:
				_move(id, at, _path_for(observation, id))
	for id in _drawn.keys():
		if not seen.has(id):
			_free(id)


func clear() -> void:
	for id in _drawn.keys():
		_free(id)
	_place = ""


## The drawn object ids, in listing order of their first appearance.
func drawn_ids() -> PackedStringArray:
	return PackedStringArray(_drawn.keys())


## Where an object is drawn now, in plan metres, or null.
func drawn_plan(id: String) -> Variant:
	return projection.to_plan(_drawn[id]["node"].position) if _drawn.has(id) else null


## Where an object is drawn now, in the world node's frame (pixels), or Vector2.ZERO.
func drawn_screen(id: String) -> Vector2:
	return _drawn[id]["node"].position if _drawn.has(id) else Vector2.ZERO


## Where an object is drawn now, in integer millimetres of the place's frame, or {}.
func drawn_local(id: String) -> Dictionary:
	var plan: Variant = drawn_plan(id)
	return {} if plan == null else town.local_in(_place, plan)


## The disclosed `at` an object was last listed at (local mm, as a Vector3i), or null.
func disclosed_at(id: String) -> Variant:
	return _drawn[id]["at"] if _drawn.has(id) else null


## "ball", "box", or "" (not drawn, or a shape this file cannot read).
func shape_of(id: String) -> String:
	if not _drawn.has(id):
		return ""
	var shape: Variant = _drawn[id]["shape"]
	if typeof(shape) == TYPE_DICTIONARY:
		if shape.has("ball"):
			return "ball"
		if shape.has("box"):
			return "box"
	return ""


## The reference to put in a request about `id`, exactly as the listing gave it.
func ref_of(id: String) -> Variant:
	return _drawn[id]["ref"] if _drawn.has(id) else null


## Whether a drawing is still moving.
func moving(id: String) -> bool:
	if not _drawn.has(id):
		return false
	var tween: Variant = _drawn[id]["tween"]
	return tween != null and (tween as Tween).is_valid() and (tween as Tween).is_running()


## The object drawn under `at` (a point in the world node's frame), or "": the front-most whose drawn
## footprint holds it. Picking is drawing, never a rule.
func pick(at: Vector2) -> String:
	var plan: Vector2 = projection.to_plan(at)
	var picked := ""
	var front := -INF
	for id in _drawn:
		var d: Dictionary = _drawn[id]
		var centre: Vector2 = projection.to_plan(d["node"].position)
		var half: Vector2 = d["half"]
		var offset := plan - centre
		var inside: bool
		if shape_of(id) == "ball":
			inside = offset.length() <= half.x
		else:
			inside = absf(offset.x) <= half.x and absf(offset.y) <= half.y
		if inside and d["node"].position.y > front:
			picked = id
			front = d["node"].position.y
	return picked


## An object reference as an identity string: a typed reference's `entity`, or the value itself.
static func id_of(ref: Variant) -> String:
	if typeof(ref) == TYPE_DICTIONARY:
		return String((ref as Dictionary).get("entity", ""))
	return "" if ref == null else str(ref)


## The keyframes of the newest `object-moved` fact for `id` in this observation, as local mm (the
## fact's `to` last, its first keyframe — `from` — dropped), or [] when there is none or it states no
## path. An event about an object not listed never reaches here.
func _path_for(observation: MineWorldObservation, id: String) -> Array:
	var found: Array = []
	for e in observation.events_of(Intents.OBJECT_MOVED):
		var outer: Variant = e.get("payload")
		var p: Variant = outer.get("payload") if typeof(outer) == TYPE_DICTIONARY else null
		if typeof(p) != TYPE_DICTIONARY or id_of(p.get("object")) != id:
			continue
		found = []
		var path: Variant = p.get("path", [])
		if typeof(path) != TYPE_ARRAY or (path as Array).is_empty() or typeof(p.get("to")) != TYPE_DICTIONARY:
			continue
		for i in range(1, path.size()):
			if typeof(path[i]) == TYPE_DICTIONARY:
				found.append(_mm(path[i]))
		found.append(_mm(p["to"]))
	return found


func _move(id: String, to: Vector3i, path: Array) -> void:
	var d: Dictionary = _drawn[id]
	d["at"] = to
	var node: Node2D = d["node"]
	if d["tween"] != null and (d["tween"] as Tween).is_valid():
		(d["tween"] as Tween).kill()
	var from: Dictionary = drawn_local(id)
	var motion := {"id": id, "how": "path" if not path.is_empty() else "ease", "from": from,
		"to": _dict(to), "points": path.map(func(p: Vector3i) -> Dictionary: return _dict(p)),
		"visited": [], "ended": false}
	motions.append(motion)
	var tween := node.create_tween()
	tween.set_process_mode(Tween.TWEEN_PROCESS_PHYSICS)
	d["tween"] = tween
	if path.is_empty():
		var goal: Vector2 = _screen(to)
		var metres: float = projection.to_plan(node.position).distance_to(projection.to_plan(goal))
		tween.tween_property(node, "position", goal, clampf(metres / OBJECT_SPEED, EASE_SHORTEST, EASE_LONGEST))
		tween.tween_callback(_visited.bind(id, motion))
	else:
		for point in path:
			tween.tween_property(node, "position", _screen(point), PATH_KEYFRAME)
			tween.tween_callback(_visited.bind(id, motion))
	tween.tween_callback(func() -> void:
		node.position = _screen(to)
		motion["ended"] = true)


func _visited(id: String, motion: Dictionary) -> void:
	motion["visited"].append(drawn_local(id))


func _build(id: String, shape: Variant, ref: Variant, at: Vector3i) -> void:
	var node := Node2D.new()
	node.name = "object_%s" % id
	node.set_meta("object", id)
	node.position = _screen(at)
	var half := Vector2.ZERO
	var kind := ""
	if typeof(shape) == TYPE_DICTIONARY and (typeof(shape.get("ball")) == TYPE_INT or typeof(shape.get("ball")) == TYPE_FLOAT):
		kind = "ball"
		var r := float(shape["ball"]) / 1000.0
		half = Vector2(r, r)
	elif typeof(shape) == TYPE_DICTIONARY and typeof(shape.get("box")) == TYPE_DICTIONARY:
		kind = "box"
		half = Vector2(float(shape["box"].get("x", 0)), float(shape["box"].get("y", 0))) / 1000.0
	else:
		half = Vector2.ONE * UNKNOWN_SIDE_M * 0.5
		if not _logged_unknown:
			_logged_unknown = true
			push_warning("[mineworld-2d] a loose object's shape is not readable, drawn as an outline: %s" % JSON.stringify(shape))
	var role := ROLE_PREFIX + kind
	var sprite: Dictionary = presentation.sprite(role) if kind != "" and presentation.is_bound(role) else {}
	if not sprite.is_empty():
		node.add_child(_bound(sprite, half))
	else:
		node.add_child(_plain(kind, half))
	world.add_child(node)
	_drawn[id] = {"node": node, "shape": shape, "ref": ref, "at": at, "half": half, "tween": null}


## The bound sprite, scaled so its width spans the footprint's drawn width.
func _bound(sprite: Dictionary, half: Vector2) -> Node2D:
	var s: Sprite2D = sprites.make(sprite, false, false)
	var outline := _footprint("box", half)
	var width := 0.0
	for p in outline:
		width = maxf(width, absf(p.x) * 2.0)
	var texture_w: float = s.texture.get_width() if s.texture != null else 0.0
	if texture_w > 0.0:
		s.scale = Vector2.ONE * width / texture_w
	return s


## Plain drawing: the footprint as a filled shape with an ink edge; an unreadable shape as an outline.
func _plain(kind: String, half: Vector2) -> Node2D:
	var holder := Node2D.new()
	var outline := _footprint(kind if kind != "" else "box", half)
	if kind != "":
		var fill := Polygon2D.new()
		fill.polygon = outline
		fill.color = BALL if kind == "ball" else BOX
		holder.add_child(fill)
	var edge := Line2D.new()
	var closed := outline.duplicate()
	closed.append(outline[0])
	edge.points = closed
	edge.width = 1.5
	edge.default_color = INK
	holder.add_child(edge)
	return holder


## A footprint's outline, on screen, relative to its centre: a disc of `half.x` or a rectangle.
func _footprint(kind: String, half: Vector2) -> PackedVector2Array:
	var out := PackedVector2Array()
	var centre: Vector2 = projection.to_screen(Vector2.ZERO)
	if kind == "ball":
		for i in DISC_SEGMENTS:
			var a := TAU * i / DISC_SEGMENTS
			out.append(projection.to_screen(Vector2(cos(a), sin(a)) * half.x) - centre)
	else:
		for c in [Vector2(-1, -1), Vector2(1, -1), Vector2(1, 1), Vector2(-1, 1)]:
			out.append(projection.to_screen(c * half) - centre)
	return out


func _free(id: String) -> void:
	var d: Dictionary = _drawn[id]
	if d["tween"] != null and (d["tween"] as Tween).is_valid():
		(d["tween"] as Tween).kill()
	(d["node"] as Node).queue_free()
	_drawn.erase(id)


func _screen(at: Vector3i) -> Vector2:
	return projection.to_screen(town.to_plan(_place, {"x": at.x, "y": at.y}))


static func _mm(local: Dictionary) -> Vector3i:
	return Vector3i(int(local.get("x", 0)), int(local.get("y", 0)), int(local.get("z", 0)))


static func _dict(at: Vector3i) -> Dictionary:
	return {"x": at.x, "y": at.y, "z": at.z}
