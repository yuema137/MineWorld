## The loose objects of the drawn place: boxes and balls, drawn where the server
## says and moved the way it says (step-11 §22 SD-E7, SD-E9; step-15 §4.5).
##
## The source is the place's disclosed `loose-objects` listing -- each object's
## id, its shape and where its centre is (the `bodies` System Pack). One
## `AnimatableBody3D` per object, on `Build.LAYER_OBJECTS`, its collider exactly
## the disclosed shape, so what blocks the player is what is drawn is what the
## server has. Items are never perceived, so nothing but the shape is known: the
## look is chosen from the shape alone (QE-4: primitives in the slice's palette).
##
## Motion is drawing, never prediction (QB-7): nothing here moves an object
## except toward a position an observation states. When an observation carries
## the `object-moved` fact for it (S11-C), the object is drawn along the fact's
## `path`, one keyframe per PATH_KEYFRAME, ending exactly at its `to`; otherwise
## (a push, which states no path, or a fact the observer did not learn) it is
## tweened to the new position. A ball also rolls -- rotation is drawing; the
## server locks it (SD-O16). Every duration is wall time (SD-E11): the world's
## time scale is never read.
class_name SliceObjects
extends Node

## The fallback motion: distance / OBJECT_SPEED, within the two bounds -- about
## a firmly kicked ball's roll after friction; a push moves an object at most a
## stride and takes the shortest (step-15 §4.5).
const OBJECT_SPEED := 4.0
## The shortest is the player's glide (`Player.GLIDE_SECONDS`), not the 0.15 s
## step-15 §4.5 proposed: a push and the pusher's correction arrive in one
## observation, and a box drawn moving slower than the body that pushed it is
## drawn inside it -- measured, 40.8 mm (step-11 §22.15 X12e-16).
const TWEEN_SHORTEST := Player.GLIDE_SECONDS
const TWEEN_LONGEST := 0.8
## Wall seconds per `path` keyframe: bodies' keyframe spacing (PATH_EVERY 6
## sub-steps at 60 Hz), played back in real time.
const PATH_KEYFRAME := 0.1
## The fact this file reads (`bodies`' `ObjectMoved`), by its event type.
const OBJECT_MOVED := "object-moved"

var world_root: Node3D
var layout: SliceLayout
## object id -> the body drawn for it
var nodes := {}
## Every motion drawn, in order: { id, how ("path" | "tween"), from, to (scene),
## seconds (wall), started and, once drawn to its end, ended (wall s) }
var motions: Array = []

var _place := ""
## object id -> { body, shape (Dictionary), at (scene), radius, queue (Array of
## [point, seconds]), from, elapsed }
var _drawn := {}


## One observation of the drawn place: build, move or remove each listed object.
func observe(obs: MineWorldObservation) -> void:
	var place := obs.place()
	if place != _place:
		clear()
		_place = place
	if not layout.is_drawn(place):
		return
	var listing: Variant = obs.component_value(place, "loose-objects")
	var seen := {}
	if typeof(listing) == TYPE_ARRAY:
		for entry in listing:
			if typeof(entry) != TYPE_DICTIONARY:
				continue
			var id := id_of(entry.get("object"))
			if id == "":
				continue
			seen[id] = true
			var at := _scene_point(place, entry.get("at"))
			if not _drawn.has(id):
				_build(id, entry.get("shape"), at)
				_refs[id] = entry.get("object")
				continue
			var d: Dictionary = _drawn[id]
			if (d["goal"] as Vector3).distance_to(at) > 0.0005:
				_move(id, at, _path_for(obs, id, place))
	for id in _drawn.keys():
		if not seen.has(id):
			(_drawn[id]["body"] as Node).queue_free()
			_drawn.erase(id)
			nodes.erase(id)


func clear() -> void:
	for id in _drawn:
		(_drawn[id]["body"] as Node).queue_free()
	_drawn.clear()
	_refs.clear()
	nodes.clear()
	_place = ""


## Where an object is drawn now, or Vector3.INF.
func drawn_at(id: String) -> Vector3:
	return (_drawn[id]["body"] as Node3D).global_position if _drawn.has(id) else Vector3.INF


## object id -> the reference the listing gave for it, exactly as received (the
## contract's typed reference `{ entity, entity_type }`): what a request about
## the object carries, so the client never builds an identity's shape itself.
var _refs := {}


## The reference to put in a request about `id`, as the world disclosed it.
func ref_of(id: String) -> Variant:
	return _refs.get(id)


## An object reference as an identity string: a typed reference's `entity`, or
## the value itself when it is already one. Identities are strings (`PROTOCOL.md`
## §7).
static func id_of(ref: Variant) -> String:
	if typeof(ref) == TYPE_DICTIONARY:
		return String((ref as Dictionary).get("entity", ""))
	return "" if ref == null else str(ref)


## What an object's disclosed shape is, "ball" or "box" ("" if not drawn): all a
## client knows of it, since items are never perceived (F-O1).
func shape_of(id: String) -> String:
	if not _drawn.has(id):
		return ""
	return "ball" if float(_drawn[id]["radius"]) > 0.0 else "box"


## Whether an object is still being moved.
func moving(id: String) -> bool:
	return _drawn.has(id) and not (_drawn[id]["queue"] as Array).is_empty()


## The keyframes of the newest `object-moved` fact for `id` in this observation,
## as scene points (the fact's `to` last), or [] when there is none or it
## states no path.
func _path_for(obs: MineWorldObservation, id: String, place: String) -> Array:
	var found: Array = []
	for e in obs.events_of(OBJECT_MOVED):
		var outer: Variant = e.get("payload")
		var p: Variant = outer.get("payload") if typeof(outer) == TYPE_DICTIONARY else null
		if typeof(p) != TYPE_DICTIONARY or id_of(p.get("object")) != id:
			continue
		var path: Variant = p.get("path", [])
		if typeof(path) != TYPE_ARRAY or (path as Array).is_empty():
			found = []
			continue
		found = []
		for k in path:
			found.append(_scene_point(place, k))
		found.append(_scene_point(place, p.get("to")))
	return found


func _move(id: String, to: Vector3, path: Array) -> void:
	var d: Dictionary = _drawn[id]
	var body: Node3D = d["body"]
	var queue: Array = []
	var how := "tween"
	var seconds := 0.0
	if path.size() >= 2:
		how = "path"
		# the first keyframe is `from`: the drawn object starts where it is
		for i in range(1, path.size()):
			queue.append([path[i], PATH_KEYFRAME])
			seconds += PATH_KEYFRAME
	else:
		seconds = clampf(body.global_position.distance_to(to) / OBJECT_SPEED, TWEEN_SHORTEST, TWEEN_LONGEST)
		queue.append([to, seconds])
	d["queue"] = queue
	d["goal"] = to
	d["from"] = body.global_position
	d["elapsed"] = 0.0
	motions.append({"id": id, "how": how, "from": body.global_position, "to": to, "seconds": seconds,
		"started": Time.get_ticks_usec() / 1e6})


## On the physics tick, as the player's glide is: a push and the pusher's
## correction start from one observation and advance together, frame by frame
## (step-11 §22.15 X12e-16); on `_process` the box trailed the body by a frame.
func _physics_process(delta: float) -> void:
	if delta <= 0.0:
		return
	for id in _drawn:
		var d: Dictionary = _drawn[id]
		var queue: Array = d["queue"]
		if queue.is_empty():
			continue
		var body: Node3D = d["body"]
		var leg: Array = queue[0]
		var target: Vector3 = leg[0]
		var secs: float = leg[1]
		d["elapsed"] = float(d["elapsed"]) + delta
		var frac := clampf(float(d["elapsed"]) / maxf(secs, 0.0001), 0.0, 1.0)
		var before := body.global_position
		body.global_position = (d["from"] as Vector3).lerp(target, frac)
		_roll(d, body.global_position - before)
		if frac >= 1.0:
			queue.pop_front()
			d["from"] = target
			d["elapsed"] = 0.0
			if queue.is_empty():
				body.global_position = d["goal"]
				for m in range(motions.size() - 1, -1, -1):
					if motions[m]["id"] == id:
						motions[m]["ended"] = Time.get_ticks_usec() / 1e6
						break


## A ball rolls about the horizontal axis normal to its motion, by distance /
## radius. Rotation is drawing only.
func _roll(d: Dictionary, step: Vector3) -> void:
	var r: float = d["radius"]
	var flat := Vector3(step.x, 0.0, step.z)
	if r <= 0.0 or flat.length() < 0.00001:
		return
	var mesh: Node3D = d["mesh"]
	var axis := Vector3.UP.cross(flat.normalized())
	mesh.global_rotate(axis, flat.length() / r)


func _build(id: String, shape: Variant, at: Vector3) -> void:
	var body := AnimatableBody3D.new()
	body.name = "Object_" + id
	body.set_meta("entity_id", id)     # a string, never a number (ADOPTION.md §3.1)
	body.collision_layer = Build.LAYER_OBJECTS
	body.collision_mask = 0
	# moved from `_process` toward observed positions, never by physics (E16a-4)
	body.sync_to_physics = false
	var cs := CollisionShape3D.new()
	var mesh := Node3D.new()
	mesh.name = "Look"
	var radius := 0.0
	if typeof(shape) == TYPE_DICTIONARY and shape.has("ball"):
		radius = float(shape["ball"]) / 1000.0
		var sphere := SphereShape3D.new()
		sphere.radius = radius
		cs.shape = sphere
		_ball_look(mesh, radius)
		_contact_shadow(body, Vector2(radius, radius) * 2.0, radius * 2.0)
	else:
		var half: Variant = shape.get("box", {}) if typeof(shape) == TYPE_DICTIONARY else {}
		var size := Vector3(float(half.get("x", 200)), float(half.get("z", 200)), float(half.get("y", 200))) \
			* 2.0 / 1000.0
		var box := BoxShape3D.new()
		box.size = size
		cs.shape = box
		_box_look(mesh, size)
		_contact_shadow(body, Vector2(size.x, size.z), size.y)
	_layer_only(mesh, SHADOW_FREE_LAYER)
	body.add_child(cs)
	body.add_child(mesh)
	world_root.add_child(body)
	body.global_position = at
	nodes[id] = body
	_drawn[id] = {"body": body, "mesh": mesh, "radius": radius, "queue": [], "goal": at, "from": at,
		"elapsed": 0.0}


## The look of every disclosed ball shape: Poly Haven's CC0 `football` (ASSETS.md), inflated
## variant only, scaled to the disclosed radius and centred on the body's centre, so the model
## rolls about the point the collider is at. Bound by shape kind, never by item id.
const BALL_MODEL := "football"
const BALL_PART := "_inflated"


static func _ball_look(mesh: Node3D, r: float) -> void:
	var holder := Node3D.new()
	holder.name = "BallModel"
	mesh.add_child(holder)
	var model := Props.gltf(BALL_MODEL)
	holder.add_child(model)
	SliceProps.keep(model, [BALL_PART])
	var box := SliceProps.aabb_of(model)
	if box.size == Vector3.ZERO:
		return
	var s := r / (box.size.y * 0.5)
	holder.scale = Vector3.ONE * s
	holder.position = -(box.position + box.size * 0.5) * s


## The visual layer a prop with a contact shadow is drawn on, so its own decal skips it.
const SHADOW_FREE_LAYER := 2
const SHADOW_LAYER_MASK := 1
static var _blob: GradientTexture2D = null


static func _layer_only(n: Node, layer: int) -> void:
	if n is VisualInstance3D:
		(n as VisualInstance3D).layers = layer
	for c in n.get_children():
		_layer_only(c, layer)


## A soft contact shadow: Godot's built-in `Decal` with a radial-gradient blob, projected down
## onto the floor under a small dynamic prop (the sun's shadow map is too coarse for a 22 cm
## ball, so without this it reads as floating). The decal is a child of the body, so it follows
## the disclosed position and never rolls; the prop's own meshes sit on another layer.
static func _contact_shadow(body: Node3D, footprint: Vector2, height: float) -> void:
	if _blob == null:
		_blob = GradientTexture2D.new()
		_blob.fill = GradientTexture2D.FILL_RADIAL
		_blob.fill_from = Vector2(0.5, 0.5)
		_blob.fill_to = Vector2(0.5, 0.0)
		_blob.width = 128
		_blob.height = 128
		var g := Gradient.new()
		g.set_color(0, Color(0, 0, 0, 0.85))
		g.set_color(1, Color(0, 0, 0, 0.0))
		g.add_point(0.55, Color(0, 0, 0, 0.45))
		_blob.gradient = g
	var d := Decal.new()
	d.name = "ContactShadow"
	d.texture_albedo = _blob
	d.cull_mask = SHADOW_LAYER_MASK
	var span := maxf(footprint.x, footprint.y) * 1.5
	d.size = Vector3(span, height * 0.6 + 0.12, span)
	d.position = Vector3(0.0, -height * 0.5, 0.0)    # the floor under the body's centre
	d.upper_fade = 0.5
	d.lower_fade = 0.5
	body.add_child(d)


## A cardboard carton: kraft brown, a strip of tape across the top.
static func _box_look(mesh: Node3D, size: Vector3) -> void:
	Build.box(mesh, Vector3.ZERO, size, SlicePalette.painted(Color(0.60, 0.45, 0.29), 0.9))
	Build.box(mesh, Vector3(0, size.y * 0.5 + 0.001, 0), Vector3(size.x * 0.18, 0.002, size.z * 1.002),
		SlicePalette.painted(Color(0.72, 0.62, 0.46), 0.5))


func _scene_point(place: String, local: Variant) -> Vector3:
	var p := layout.to_scene(place, local)
	var up := float(local.get("z", 0)) / 1000.0 if typeof(local) == TYPE_DICTIONARY else 0.0
	return p + Vector3(0.0, up, 0.0)
