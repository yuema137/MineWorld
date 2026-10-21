## The current place's disclosed walls and solids, as invisible colliders the
## player's body meets (step-11 §22 SD-E4; step-15 §4.2).
##
## The `bodies` System Pack discloses each place's fixed geometry with the place
## (`place-shape`: the walkable floor and the solids standing on it, in the
## place's frame, millimetres). The scene's own walls stay -- they are the world
## the operator accepted -- and this geometry is added beside them, so the body
## is stopped where the server would stop it even where the scene and the server
## disagree. Where they disagree, that is a content defect for the probe
## (`--world --geometry`) to name, never something this file absorbs.
##
## Every number here is the server's (the floor, each solid's footprint and
## height) or a size of something built (the perimeter wall's thickness and
## height, the gap at a doorway). None is a rule: nothing here decides what may
## be done.
##
## A place without `place-shape` -- a world without bodies -- builds nothing.
class_name SliceClientBodies
extends RefCounted

## The perimeter walls, outside the floor: 200 mm thick and 3 000 mm high, the
## sizes bodies' own Rapier perimeter uses (step-11 SD-B11).
const WALL_THICKNESS := 0.20
const WALL_TALL := 3.0
## A gap cut in a perimeter wall at each disclosed doorway whose point lies
## within DOOR_EDGE of that edge (12d puts every doorway 0.4 m inside, SD-D3). A
## crossing is a placement, not a sweep (SD-B3): the server has no wall there to
## mirror, and the scene's own jambs still stop the body.
const DOOR_GAP := 2.0
const DOOR_EDGE := 0.5
## The node every disclosed collider hangs under.
const NODE_NAME := "DisclosedGeometry"

## The place the node was built for, and the value it was built from.
var place := ""
var _built_from: Variant = null
var node: Node3D = null


## Rebuild for `obs`'s place when the place or its disclosed `place-shape`
## changed. Returns true when it rebuilt.
func observe(obs: MineWorldObservation, layout: SliceLayout, world_root: Node3D) -> bool:
	var here := obs.place()
	var shape: Variant = obs.component_value(here, "place-shape")
	if here == place and shape == _built_from and (node != null or shape == null):
		return false
	clear()
	place = here
	_built_from = shape
	if typeof(shape) != TYPE_DICTIONARY or not layout.is_drawn(here):
		return true
	node = build(here, shape, layout.passages.get(here, []), layout)
	world_root.add_child(node)
	return true


func clear() -> void:
	if node != null:
		node.queue_free()
	node = null
	place = ""
	_built_from = null


## The colliders of one place's disclosed shape, in the scene (not yet in the
## tree). `passages` as `SliceLayout.passages` keeps them.
static func build(where: String, shape: Dictionary, passages: Array, layout: SliceLayout) -> Node3D:
	var root := Node3D.new()
	root.name = NODE_NAME
	var floor_d: Dictionary = shape.get("floor", {})
	var lo := _mm(floor_d.get("min", {}))
	var hi := _mm(floor_d.get("max", {}))
	var base_y := layout.to_scene(where, {"x": 0, "y": 0}).y
	var doors: Array[Vector2] = []
	for p in passages:
		if typeof(p) == TYPE_DICTIONARY and typeof(p.get("here")) == TYPE_DICTIONARY:
			doors.append(_mm(p["here"]))
	var t := WALL_THICKNESS
	# each edge: its line, its extent along the line, and which side is outside
	_edge(root, where, layout, base_y, Vector2(lo.x - t, lo.y - t), Vector2(hi.x + t, lo.y), doors, true, lo.y, "WallS")
	_edge(root, where, layout, base_y, Vector2(lo.x - t, hi.y), Vector2(hi.x + t, hi.y + t), doors, true, hi.y, "WallN")
	_edge(root, where, layout, base_y, Vector2(lo.x - t, lo.y), Vector2(lo.x, hi.y), doors, false, lo.x, "WallW")
	_edge(root, where, layout, base_y, Vector2(hi.x, lo.y), Vector2(hi.x + t, hi.y), doors, false, hi.x, "WallE")
	var solids: Variant = shape.get("solids", [])
	if typeof(solids) == TYPE_ARRAY:
		var i := 0
		for s in solids:
			if typeof(s) != TYPE_DICTIONARY:
				continue
			var height := float(s.get("height", 0)) / 1000.0
			_box(root, where, layout, base_y, _mm(s.get("min", {})), _mm(s.get("max", {})), 0.0, height,
				"Solid%d" % i)
			i += 1
	return root


## One perimeter wall: the box from `a` to `b` (metres, place frame), cut by a
## DOOR_GAP wherever a doorway lies within DOOR_EDGE of the edge at `line`
## (the shared span function, `SliceSpans`). `along_x` says the wall runs
## east-west.
static func _edge(root: Node3D, where: String, layout: SliceLayout, base_y: float, a: Vector2, b: Vector2,
		doors: Array[Vector2], along_x: bool, line: float, label: String) -> void:
	var centres := SliceSpans.openings(doors, along_x, line, DOOR_EDGE)
	var start := a.x if along_x else a.y
	var stop := b.x if along_x else b.y
	var n := 0
	for piece in SliceSpans.split(start, stop, centres, DOOR_GAP):
		var lo := Vector2(piece[0], a.y) if along_x else Vector2(a.x, piece[0])
		var hi := Vector2(piece[1], b.y) if along_x else Vector2(b.x, piece[1])
		_box(root, where, layout, base_y, lo, hi, 0.0, WALL_TALL, "%s%d" % [label, n])
		n += 1


## An invisible static box on `Build.LAYER_DISCLOSED` from footprint `lo`..`hi`
## (metres, place frame), from `bottom` to `top` above the floor.
static func _box(root: Node3D, where: String, layout: SliceLayout, base_y: float, lo: Vector2, hi: Vector2,
		bottom: float, top: float, nm: String) -> void:
	if hi.x - lo.x <= 0.0 or hi.y - lo.y <= 0.0 or top - bottom <= 0.0:
		return
	var body := StaticBody3D.new()
	body.name = nm
	body.collision_layer = Build.LAYER_DISCLOSED
	body.collision_mask = 0
	var cs := CollisionShape3D.new()
	var box := BoxShape3D.new()
	box.size = Vector3(hi.x - lo.x, top - bottom, hi.y - lo.y)
	cs.shape = box
	body.add_child(cs)
	var centre := layout.to_scene(where, {"x": int(round((lo.x + hi.x) * 500.0)),
		"y": int(round((lo.y + hi.y) * 500.0))})
	body.position = Vector3(centre.x, base_y + (bottom + top) * 0.5, centre.z)
	root.add_child(body)


## A disclosed `{x, y}` in millimetres -> metres, (east, north).
static func _mm(v: Variant) -> Vector2:
	if typeof(v) != TYPE_DICTIONARY:
		return Vector2.ZERO
	return Vector2(float(v.get("x", 0)) / 1000.0, float(v.get("y", 0)) / 1000.0)
