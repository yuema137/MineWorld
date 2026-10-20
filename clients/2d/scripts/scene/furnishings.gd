extends Node

## The furniture of every place the observer has stood in, drawn from the world's data (`ARC-307`;
## `clients/2d/PRESENTATION.md` §4.1).
##
## The `bodies` System Pack discloses a place's shape with the place: its solids, and the furnishings
## those solids belong to (key, kind, `at`, `facing`). This draws what it was told and nothing else:
##
##     a furnishing whose role `furnishing.<kind>` (or a `.`-prefix of it) the pack binds
##                         -> that sprite at `at`, over its solids' footprints, mirrored to its facing
##     a furnishing nothing binds, or plain drawing
##                         -> its solids' footprints as filled shapes
##     a solid that names no furnishing (the room's shell)
##                         -> its footprint as an outline
##
## So every box a body meets is shown, and no disclosed piece is left out (I-FD-5). It names no place,
## no world and no kind (I-FD-2), and decides nothing: footprints are drawing, never a rule.

const Sprites := preload("res://scripts/scene/sprites.gd")

## The role prefix a kind is drawn by: `furnishing.` + the kind, then each shorter `.`-prefix.
const ROLE_PREFIX := "furnishing."
## The colours of plain drawing: a furnishing's footprint, and a shell solid's outline.
const FOOTPRINT := Color(0.52, 0.40, 0.28, 0.85)
const SHELL := Color(0.30, 0.25, 0.20, 0.9)
## Footprints and outlines lie on the floor, under everyone.
const FLOOR_Z := -5

var projection
var presentation
var town
var sprites
var world: Node2D

## place id → the `place-shape` its drawing was built from.
var _built_from := {}
## place id → its nodes.
var _nodes := {}
## place id → one row per furnishing drawn: {key, kind, role, at (plan metres), plain}.
var rows := {}
var _version := -1


func setup(the_projection, the_presentation, the_town, the_world: Node2D) -> void:
	projection = the_projection
	presentation = the_presentation
	town = the_town
	world = the_world
	sprites = Sprites.new(presentation)


## Draws the observer's place's furnishings when its disclosed shape changed, or when the town placed
## a place anew (every drawing is redone then, since plan positions moved).
func reconcile(observation: MineWorldObservation) -> void:
	var here := observation.place()
	var shape: Variant = observation.component_value(here, "place-shape") if here != "" else null
	var changed: bool = typeof(shape) == TYPE_DICTIONARY and shape != _built_from.get(here)
	if changed:
		_built_from[here] = shape
	if town.version != _version:
		_version = town.version
		for place in _built_from.keys():
			_draw_place(place)
	elif changed:
		_draw_place(here)


func _draw_place(place: String) -> void:
	for node in _nodes.get(place, []):
		node.queue_free()
	_nodes[place] = []
	rows[place] = []
	var shape: Dictionary = _built_from[place]
	if town.to_plan(place, {"x": 0, "y": 0}) == null:
		return
	var solids: Array = shape.get("solids", []) if typeof(shape.get("solids")) == TYPE_ARRAY else []
	for s in solids:
		if typeof(s) == TYPE_DICTIONARY and String(s.get("furnishing", "")) == "":
			_outline(place, s)
	var list: Array = shape.get("furnishings", []) if typeof(shape.get("furnishings")) == TYPE_ARRAY else []
	for f in list:
		if typeof(f) != TYPE_DICTIONARY:
			continue
		var key := String(f.get("key", ""))
		var kind := String(f.get("kind", ""))
		var role := resolve(kind)
		var at: Vector2 = town.to_plan(place, f.get("at", {}))
		var sprite: Dictionary = presentation.sprite(role) if role != "" else {}
		for s in solids:
			if typeof(s) == TYPE_DICTIONARY and String(s.get("furnishing", "")) == key:
				_footprint(place, s, sprite.is_empty())
		if not sprite.is_empty():
			var facing := deg_to_rad(float(f.get("facing", 0)) / 1000.0)
			var screen_dir: Vector2 = projection.plan_dir_to_screen(Vector2(sin(facing), cos(facing)))
			var node: Node2D = sprites.make(sprite, false, screen_dir.x < 0.0)
			var holder := Node2D.new()
			holder.position = projection.to_screen(at)
			holder.add_child(node)
			world.add_child(holder)
			_nodes[place].append(holder)
		rows[place].append({"key": key, "kind": kind, "role": role, "at": at, "plain": sprite.is_empty()})


## The pack's role for `kind`: `furnishing.<kind>`, else each shorter `.`-prefix; "" when none is bound.
func resolve(kind: String) -> String:
	var k := kind
	while k != "":
		if presentation.is_bound(ROLE_PREFIX + k):
			return ROLE_PREFIX + k
		var cut := k.rfind(".")
		k = k.substr(0, cut) if cut > 0 else ""
	return ""


## A solid's footprint corners, on screen.
func _corners(place: String, s: Dictionary) -> PackedVector2Array:
	var lo: Dictionary = s.get("min", {})
	var hi: Dictionary = s.get("max", {})
	var out := PackedVector2Array()
	for c in [[lo.get("x", 0), lo.get("y", 0)], [hi.get("x", 0), lo.get("y", 0)],
			[hi.get("x", 0), hi.get("y", 0)], [lo.get("x", 0), hi.get("y", 0)]]:
		out.append(projection.to_screen(town.to_plan(place, {"x": int(c[0]), "y": int(c[1])})))
	return out


func _footprint(place: String, s: Dictionary, filled: bool) -> void:
	var poly := Polygon2D.new()
	poly.polygon = _corners(place, s)
	poly.color = FOOTPRINT if filled else Color(FOOTPRINT, 0.25)
	poly.z_index = FLOOR_Z
	world.add_child(poly)
	_nodes[place].append(poly)


func _outline(place: String, s: Dictionary) -> void:
	var line := Line2D.new()
	var corners := _corners(place, s)
	corners.append(corners[0])
	line.points = corners
	line.width = 1.5
	line.default_color = SHELL
	line.z_index = FLOOR_Z
	world.add_child(line)
	_nodes[place].append(line)
