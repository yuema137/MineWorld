extends Node

## The places of the drawn town: the hub's paving, a façade or a room at every doorway, lawns, the
## room the observer stands in, and the dressing around them (`ARC-45`, `clients/2d/PRESENTATION.md`
## §4).
##
## Every position here is derived from a disclosed doorway or from the rectangle the hub's doorways
## span. No place is drawn by key or at an absolute coordinate, so the same pack dresses any world.
## Footprints are decoration: they are registered with the town only so a click can be routed into
## the place it points at, and are never used to refuse or shorten a stride.

const Ground := preload("res://scripts/scene/ground.gd")
const Sprites := preload("res://scripts/scene/sprites.gd")

## How fast a façade lifts away and a room fades in, per second (the spike's 3.4: 1 → 0 in 0.3 s).
const FADE_RATE := 3.4
## How far the hub's paving reaches past its outermost doorways when the pack does not say.
const DEFAULT_MARGIN_M := 5.0

var projection
var presentation
var town
var sprites
var world: Node2D
var ground: Node2D

## The place the observer stands in, and the hub, as last drawn.
var here := ""
## Where the observer stood in the latest observation, plan metres.
var self_plan := Vector2.ZERO
## place id → {kind: "facade"|"room"|"lawn", door: Vector2, in_dir: Vector2, rect: Rect2, node,
## fittings: Array[Node2D], fade: float}
var drawn: Dictionary = {}

var _version := -1
var _dressing: Array[Node2D] = []


func setup(the_projection, the_presentation, the_town, the_world: Node2D, parent: Node2D) -> void:
	projection = the_projection
	presentation = the_presentation
	town = the_town
	world = the_world
	sprites = Sprites.new(presentation)
	ground = Ground.new()
	ground.projection = projection
	ground.presentation = presentation
	ground.z_index = -1000
	ground.texture_repeat = CanvasItem.TEXTURE_REPEAT_ENABLED
	parent.add_child(ground)


## Rebuilds the drawing when the layout learned something; otherwise only notes where the observer is.
func reconcile(observation: MineWorldObservation) -> void:
	here = observation.place()
	var at: Variant = town.to_plan(here, observation.self_location().get("local"))
	if at != null:
		self_plan = at
	if town.version != _version:
		_version = town.version
		_rebuild()


func _process(delta: float) -> void:
	var dirty := false
	for place in drawn:
		var entry: Dictionary = drawn[place]
		var inside: bool = place == here
		var target := 1.0 if inside else 0.0
		var fade: float = entry.get("fade", 0.0)
		var next := move_toward(fade, target, delta * FADE_RATE)
		if next == fade:
			continue
		entry["fade"] = next
		if entry["kind"] == "facade" and entry.get("node") != null:
			entry["node"].modulate.a = 1.0 - next
		for node in entry["fittings"]:
			if entry["kind"] == "facade":
				node.modulate.a = next
				node.visible = next > 0.002
		dirty = true
	if dirty:
		_update_areas()


## How opaque a place's façade is drawn (AC-W12's cut-away check). 1.0 for a place with none.
func facade_alpha(place: String) -> float:
	var entry: Dictionary = drawn.get(place, {})
	if entry.get("node") == null:
		return 1.0
	return entry["node"].modulate.a


## Where a façade's door is drawn, and where its doorway is (both screen pixels), for AC-W12.
func facade_anchor(place: String) -> Dictionary:
	var entry: Dictionary = drawn.get(place, {})
	if entry.get("node") == null:
		return {}
	return {"drawn": entry["node"].position, "doorway": projection.to_screen(entry["door"])}


func _rebuild() -> void:
	for place in drawn:
		var entry: Dictionary = drawn[place]
		if entry.get("node") != null:
			entry["node"].queue_free()
		for node in entry["fittings"]:
			node.queue_free()
	for node in _dressing:
		node.queue_free()
	drawn.clear()
	_dressing.clear()
	town.footprints.clear()
	var hub: String = town.hub()
	var extent: Rect2 = town.hub_extent()
	var margin := float(presentation.setting(["ground", "root_margin_m"], DEFAULT_MARGIN_M))
	var paved := extent.grow(margin) if hub != "" else Rect2()
	if hub != "":
		for p in town.passages[hub]:
			_draw_doorway(hub, p, extent.get_center())
	# A place the observer stands in that the hub's doorways did not reach (the first place before
	# the street is known): drawn as a room around its own doorway.
	for place in town.passages:
		if place != hub and not drawn.has(place):
			for p in town.passages[place]:
				_draw_room_from_inside(place, p)
				break
	_edges(paved)
	ground.paved = paved
	var land_margin := float(presentation.setting(["ground", "land_margin_m"], 60.0))
	ground.land = (paved if paved.has_area() else Rect2(Vector2.ZERO, Vector2.ZERO)).grow(land_margin)
	_update_areas()


## One doorway out of the hub: its place drawn as a façade (far side), a room (near side) or a lawn.
func _draw_doorway(hub: String, p: Dictionary, centre: Vector2) -> void:
	var place: String = p["to"]
	var door: Vector2 = town.to_plan(hub, {"x": p["here"].x, "y": p["here"].y})
	var in_dir := _axis(door - centre)
	var right := Vector2(-in_dir.y, in_dir.x)
	var place_tags: PackedStringArray = town.tags.get(place, PackedStringArray())
	var outdoor := _by_tag("outdoor", place_tags)
	var far: bool = projection.plan_dir_to_screen(in_dir).y < 0.0
	var spec: Dictionary = outdoor if not outdoor.is_empty() else _by_tag("interiors", place_tags)
	var rect := _footprint(door, in_dir, right, spec)
	town.footprints[place] = rect
	var entry := {"door": door, "in_dir": in_dir, "rect": rect, "node": null, "fittings": [], "fade": 0.0,
		"lights": []}
	if not outdoor.is_empty():
		entry["kind"] = "lawn"
		for item in outdoor.get("dressing", []):
			_dressing.append(_prop(item, door, in_dir, right))
	elif far:
		entry["kind"] = "facade"
		entry["node"] = _facade(place, place_tags, door)
	else:
		entry["kind"] = "room"
	if entry["kind"] != "lawn":
		for item in spec.get("fittings", []):
			var node := _prop(item, door, in_dir, right)
			if node != null:
				entry["fittings"].append(node)
				if entry["kind"] == "facade":
					node.visible = false
		for light in spec.get("lights", []):
			entry["lights"].append(door + right * float(light.get("along_m", 0.0)) + in_dir * float(light.get("in_m", 0.0)))
	drawn[place] = entry
	for item in _facade_dressing(place_tags):
		_dressing.append(_prop(item, door, in_dir, right))


## The first place stood in before the hub is known: a room around its own doorway.
func _draw_room_from_inside(place: String, p: Dictionary) -> void:
	var door: Vector2 = town.to_plan(place, {"x": p["here"].x, "y": p["here"].y})
	# Which way the room runs from its door is not disclosed; the observer stands inside it, so the
	# room is drawn toward them. Presentation only, and replaced once the hub is known.
	var in_dir := _axis(self_plan - door) if self_plan.distance_to(door) > 0.01 else Vector2(0, -1)
	var right := Vector2(-in_dir.y, in_dir.x)
	var spec := _by_tag("interiors", town.tags.get(place, PackedStringArray()))
	var rect := _footprint(door, in_dir, right, spec)
	town.footprints[place] = rect
	var entry := {"door": door, "in_dir": in_dir, "rect": rect, "node": null, "fittings": [], "fade": 1.0,
		"kind": "room", "lights": []}
	for item in spec.get("fittings", []):
		var node := _prop(item, door, in_dir, right)
		if node != null:
			entry["fittings"].append(node)
	for light in spec.get("lights", []):
		entry["lights"].append(door + right * float(light.get("along_m", 0.0)) + in_dir * float(light.get("in_m", 0.0)))
	drawn[place] = entry


func _footprint(door: Vector2, in_dir: Vector2, right: Vector2, spec: Dictionary) -> Rect2:
	var width := float(spec.get("width_m", 9.0))
	var depth := float(spec.get("depth_m", 8.0))
	var left := float(spec.get("door_from_left_m", width * 0.5))
	var a := door - right * left
	var b := door + right * (width - left)
	var rect := Rect2(a, Vector2.ZERO)
	for corner in [a, b, a + in_dir * depth, b + in_dir * depth]:
		rect = rect.expand(corner)
	return rect


func _facade(place: String, place_tags: PackedStringArray, door: Vector2) -> Node2D:
	var holder := Node2D.new()
	holder.name = "facade_%s" % place
	holder.position = projection.to_screen(door)
	var sprite := {}
	for tag in place_tags:
		sprite = presentation.sprite("facade:%s" % tag)
		if not sprite.is_empty():
			break
	if sprite.is_empty():
		var generic := 0
		while presentation.is_bound("facade:unknown:%d" % generic):
			generic += 1
		if generic > 0:
			sprite = presentation.sprite("facade:unknown:%d" % posmod(place.hash(), generic))
	var node: Node2D = sprites.make(sprite, true)
	if node == null:
		node = Sprites.plain("block", Vector2(projection.ppm * 4.0, projection.ppm * 3.0), Color("c9b9a3"))
	holder.add_child(node)
	world.add_child(holder)
	return holder


func _facade_dressing(place_tags: PackedStringArray) -> Array:
	var out: Array = []
	out.append_array(presentation.setting(["facades", "*", "dressing"], []))
	for tag in place_tags:
		var own: Array = presentation.setting(["facades", tag, "dressing"], [])
		if not own.is_empty():
			out.append_array(own)
			break
	return out


func _prop(item: Dictionary, door: Vector2, in_dir: Vector2, right: Vector2) -> Node2D:
	var at := door + right * float(item.get("along_m", 0.0)) \
		+ in_dir * (float(item.get("in_m", 0.0)) - float(item.get("out_m", 0.0)))
	var sprite: Dictionary = presentation.sprite(String(item.get("role", "")))
	var node: Node2D = sprites.make(sprite)
	if node == null:
		if not presentation.is_plain():
			return null
		node = Sprites.plain("prop", Vector2(4, 4))
	var holder := Node2D.new()
	holder.position = projection.to_screen(at)
	node.position.y -= projection.height_px(float(item.get("lift_m", 0.0)))
	holder.add_child(node)
	world.add_child(holder)
	return holder


## Planting beyond the hub's paving, so the town does not end at the frame's edge.
func _edges(paved: Rect2) -> void:
	var roles: Array = presentation.setting(["edges", "roles"], [])
	if roles.is_empty() or not paved.has_area():
		return
	var rows := int(presentation.setting(["edges", "rows"], 2))
	var spacing := float(presentation.setting(["edges", "spacing_m"], 4.2))
	var gap := float(presentation.setting(["edges", "row_gap_m"], 3.0))
	var offset := float(presentation.setting(["edges", "offset_m"], 3.0))
	var n := 0
	for row in range(rows):
		var ring := paved.grow(offset + row * gap)
		var sides := [[ring.position, Vector2(ring.end.x, ring.position.y)],
			[Vector2(ring.end.x, ring.position.y), ring.end],
			[ring.end, Vector2(ring.position.x, ring.end.y)],
			[Vector2(ring.position.x, ring.end.y), ring.position]]
		for side in sides:
			var a: Vector2 = side[0]
			var b: Vector2 = side[1]
			var steps := int(a.distance_to(b) / spacing)
			for i in range(steps):
				var at := a.lerp(b, (i + 0.5) / float(steps)) + Vector2(sin(n * 1.7) * 0.6, cos(n * 2.3) * 0.6)
				n += 1
				if _in_any_footprint(at):
					continue
				var role := String(roles[(n * 7 + row * 3) % roles.size()])
				var node := _prop({"role": role}, at, Vector2(0, -1), Vector2(1, 0))
				if node != null:
					_dressing.append(node)


func _in_any_footprint(at: Vector2) -> bool:
	for place in town.footprints:
		if town.footprints[place].grow(1.5).has_point(at):
			return true
	return false


func _update_areas() -> void:
	var areas: Array = []
	for place in drawn:
		var entry: Dictionary = drawn[place]
		match entry["kind"]:
			"lawn":
				areas.append({"kind": "lawn", "rect": entry["rect"]})
			"room":
				areas.append({"kind": "room", "rect": entry["rect"], "lights": entry["lights"], "alpha": 1.0})
			"facade":
				if entry["fade"] > 0.002:
					areas.append({"kind": "room", "rect": entry["rect"], "lights": entry["lights"], "alpha": entry["fade"]})
	ground.areas = areas
	ground.queue_redraw()


## A tag-keyed block of the renderer parameters (`interiors`, `outdoor`): the first of the place's
## tags it has; for `interiors`, else its `*`.
func _by_tag(section: String, place_tags: PackedStringArray) -> Dictionary:
	var table: Dictionary = presentation.setting([section], {})
	for tag in place_tags:
		if table.has(tag):
			return table[tag]
	return table.get("*", {}) if section == "interiors" else {}


## The axis direction (east, west, north or south, as a plan unit vector) a vector mostly points.
static func _axis(v: Vector2) -> Vector2:
	if absf(v.x) > absf(v.y):
		return Vector2(signf(v.x), 0.0)
	return Vector2(0.0, signf(v.y) if v.y != 0.0 else -1.0)
