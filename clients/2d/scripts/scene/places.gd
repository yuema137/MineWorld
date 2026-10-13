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
const Words := preload("res://scripts/hud/words.gd")

## How fast a façade lifts away and a room fades in, per second (the spike's 3.4: 1 → 0 in 0.3 s).
const FADE_RATE := 3.4
## How far behind its door a façade is sorted (drawing only; see `_facade`).
const SORT_BEHIND_M := 0.6
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
## One label per drawn doorway of a room, naming where it leads (meta `to`).
var _door_labels: Array[Label] = []
## Which way a room seen only from inside runs from its door, fixed when it is first drawn: the
## world discloses no extent, so the guess is made once and never follows the player (F-9).
var _inside_dir: Dictionary = {}
var _observation: MineWorldObservation = null


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
	_observation = observation
	for label in _door_labels:
		label.text = _door_text(label.get_meta("to"))


## The wording key of each tag that names a place (R-PK-1, ARC-82). The keys are written out, not built,
## so the catalogue check (client_text, AC-SET-4) sees each one as used.
const PLACE_NAME_KEYS := {
	"cafe": "place.cafe", "park": "place.park", "store": "place.store",
	"apartments": "place.apartments", "workplace": "place.workplace", "street": "place.street",
}


## "door to <where it leads>". The name comes from the first tag the destination carries, in the order
## the world lists them, that the wording names; else the place's display name if the world gives one;
## else "outside". A tag the wording does not name (`public`) says nothing about where a door leads.
func _door_text(to: String) -> String:
	return Words.text("ui.door-to", {"place": _place_name(to)})


func _place_name(to: String) -> String:
	var to_tags: PackedStringArray = town.tags.get(to, PackedStringArray())
	for tag in to_tags:
		var key: String = PLACE_NAME_KEYS.get(tag, "")
		if key != "" and Words.has(key):
			return Words.text(key)
	var name := _observation.display_name(to) if _observation != null else ""
	return name if name != "" else Words.text("ui.outside")


## The doorways of the observer's place, each with the text it is named by: `{to, text}`. For the harness's
## report (R-PK-1). A label is drawn only at the doors of a room the observer stands in, so from the street
## this is the wording a doorway would carry, not a label on screen.
func door_labels() -> Array:
	var out: Array = []
	for p in town.passages.get(here, []):
		out.append({"to": p["to"], "text": _door_text(p["to"])})
	return out


## Whether a doorway of `place` is drawn at `at` (plan metres), on a floor that is showing (F-10).
func door_drawn(place: String, at: Vector2) -> bool:
	for area in ground.areas:
		if area.get("place") != place or float(area.get("alpha", 1.0)) <= 0.002:
			continue
		for door in area.get("doors", []):
			if (door["at"] as Vector2).distance_to(at) < 0.01:
				return true
	return false


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
	var holder: Node2D = entry["node"]
	return {"drawn": holder.position + holder.get_meta("door_offset", Vector2.ZERO),
		"doorway": projection.to_screen(entry["door"])}


func _rebuild() -> void:
	for place in drawn:
		var entry: Dictionary = drawn[place]
		if entry.get("node") != null:
			entry["node"].queue_free()
		for node in entry["fittings"]:
			node.queue_free()
	for node in _dressing:
		node.queue_free()
	for label in _door_labels:
		label.queue_free()
	drawn.clear()
	_dressing.clear()
	_door_labels.clear()
	town.footprints.clear()
	var hub: String = town.hub()
	var extent: Rect2 = town.hub_extent()
	var margin := float(presentation.setting(["ground", "root_margin_m"], DEFAULT_MARGIN_M))
	var paved := extent.grow(margin) if hub != "" else Rect2()
	if hub != "":
		for p in town.passages[hub]:
			_draw_doorway(hub, p, extent)
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
func _draw_doorway(hub: String, p: Dictionary, extent: Rect2) -> void:
	var place: String = p["to"]
	var door: Vector2 = town.to_plan(hub, {"x": p["here"].x, "y": p["here"].y})
	var in_dir := _inward(door, extent)
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
		entry["node"] = _facade(place, place_tags, door, in_dir)
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
	# room is drawn toward where they first stood. Presentation only, chosen once (F-9), and replaced
	# once the hub is known.
	if not _inside_dir.has(place):
		_inside_dir[place] = _axis(self_plan - door) if self_plan.distance_to(door) > 0.01 else Vector2(0, -1)
	var in_dir: Vector2 = _inside_dir[place]
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


func _facade(place: String, place_tags: PackedStringArray, door: Vector2, in_dir: Vector2) -> Node2D:
	var holder := Node2D.new()
	holder.name = "facade_%s" % place
	# A façade is sorted as if it stood SORT_BEHIND_M behind its door, and drawn exactly where it was:
	# anyone on its doorstep is drawn in front of it (sorted at the door's own depth, a player who had
	# just come out was hidden behind the building), while what stands behind it stays behind.
	var sort_at: Vector2 = projection.to_screen(door + in_dir * SORT_BEHIND_M)
	var offset: Vector2 = projection.to_screen(door) - sort_at
	holder.position = sort_at
	holder.set_meta("door_offset", offset)
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
	node.position += offset
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
				areas.append({"kind": "lawn", "rect": entry["rect"], "place": place})
			"room":
				areas.append({"kind": "room", "rect": entry["rect"], "lights": entry["lights"], "alpha": 1.0,
					"place": place, "doors": _doors(place, entry)})
			"facade":
				if entry["fade"] > 0.002:
					areas.append({"kind": "room", "rect": entry["rect"], "lights": entry["lights"],
						"alpha": entry["fade"], "place": place, "doors": _doors(place, entry)})
	ground.areas = areas
	ground.queue_redraw()
	_place_door_labels(areas)


## Every disclosed doorway of a room, where it is drawn: `{at, out, to}` in plan metres. The doorway
## the room was drawn from opens against `in_dir`; any other opens through the nearest wall.
func _doors(place: String, entry: Dictionary) -> Array:
	var out: Array = []
	var rect: Rect2 = entry["rect"]
	for p in town.passages.get(place, []):
		var at: Variant = town.to_plan(place, {"x": p["here"].x, "y": p["here"].y})
		if at == null:
			continue
		var outward: Vector2 = -entry["in_dir"] if (at as Vector2).distance_to(entry["door"]) < 0.01 \
			else _axis(at - rect.get_center())
		out.append({"at": at, "out": outward, "to": p["to"]})
	return out


## A label just outside each drawn doorway of the room the observer stands in, naming where it leads.
func _place_door_labels(areas: Array) -> void:
	for label in _door_labels:
		label.queue_free()
	_door_labels.clear()
	for area in areas:
		if area.get("place") != here:
			continue
		for door in area.get("doors", []):
			var label := Label.new()
			label.auto_translate_mode = Node.AUTO_TRANSLATE_MODE_DISABLED
			label.set_meta("to", door["to"])
			label.text = _door_text(door["to"])
			label.add_theme_font_size_override("font_size", 13)
			label.add_theme_color_override("font_color", Color("2e2418"))
			label.add_theme_color_override("font_outline_color", Color(1, 0.96, 0.82, 0.95))
			label.add_theme_constant_override("outline_size", 6)
			label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
			label.size = Vector2(160, 18)
			label.position = projection.to_screen(door["at"] + door["out"] * 1.0) - Vector2(80, 9)
			label.z_index = 50
			world.add_child(label)
			_door_labels.append(label)


## A tag-keyed block of the renderer parameters (`interiors`, `outdoor`): the first of the place's
## tags it has; for `interiors`, else its `*`.
func _by_tag(section: String, place_tags: PackedStringArray) -> Dictionary:
	var table: Dictionary = presentation.setting([section], {})
	for tag in place_tags:
		if table.has(tag):
			return table[tag]
	return table.get("*", {}) if section == "interiors" else {}


## Which way a place runs from its doorway on the hub: away from the hub, across the hub's long axis.
## Doorways line the long sides of a street, so a doorway at a corner of the extent still faces
## across it (the convenience store at the street's north-east corner opens north, not east).
static func _inward(door: Vector2, extent: Rect2) -> Vector2:
	var d := door - extent.get_center()
	if extent.size.x >= extent.size.y:
		return Vector2(0.0, signf(d.y) if d.y != 0.0 else -1.0)
	return Vector2(signf(d.x) if d.x != 0.0 else 1.0, 0.0)


## The axis direction (east, west, north or south, as a plan unit vector) a vector mostly points.
static func _axis(v: Vector2) -> Vector2:
	if absf(v.x) > absf(v.y):
		return Vector2(signf(v.x), 0.0)
	return Vector2(0.0, signf(v.y) if v.y != 0.0 else -1.0)
