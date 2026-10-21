## Places drawn plainly from the server's disclosure (S14 16f-b, `docs/decisions/ARC-52.md`;
## `PRESENTATION.md` §6).
##
## A place the server discloses and no depiction entry draws is still drawn: `SliceLayout` glues it into
## the scene at its passage (`ARC-45` point 1) and binds it as a SHELL, and this file draws it from its
## disclosed `place-shape`:
##
##   the floor     a slab of the floor rectangle, its top at the place's floor -- ground the feet
##                 stand on (`Build.LAYER_WORLD`), because the plan has no height
##   the walls     four walls outside the floor, as thick as the disclosed colliders, each split at
##                 every doorway by the one span function (`SliceSpans`), a lintel over each opening
##   the solids    a box of each shell solid's footprint and height (a solid that names no furnishing,
##                 `CORE_CONCEPTS.md` §6.5); furnished solids are `SliceFurnishings`'
##   the volume    the floor as the place's scene volume, so the body is reported in it
##
## The walls and solids are drawing only: the colliders that stop the body stay `SliceClientBodies`',
## built from the same numbers. Every place drawn stays drawn while this world instance lasts (`ARC-45`
## point 2), and is dropped with the layout when the instance changes (`forget`).
##
## A shell place one doorway away that has not been stood in has no disclosed shape yet (the server
## discloses only the room you stand in). It gets a THRESHOLD: a volume just past the current floor's
## edge at the doorway's gap, so that walking through the gap reports the body in that place, and the
## server's answer discloses its shape.
##
## The look comes from the pack's `places/shells.yaml` (`MODULE_SPEC.md` §6.1, validated by
## `mineworld packs validate`): materials by slug, a default style, and styles by tag with `.`-prefix
## fallback, as the furnishings binding. A place's style is the default changed by the first of its
## disclosed tags, in disclosed order, that the file binds. A missing or faulty file draws the client's
## plain grey. Nothing here names a world, a place or a tag.
class_name SliceShells
extends RefCounted

## The pack's style file, relative to the 3D Presentation Pack's folder.
const BINDING_FILE := "places/shells.yaml"
## The node every shell's drawing hangs under.
const NODE_NAME := "Shells"
## Every scene volume this file makes is named with this prefix (`is_shell`).
const VOLUME_PREFIX := "shell."
## A shell's floor slab: its thickness below the floor (a drawing size).
const FLOOR_T := 0.10
## A place volume reaches this far below the floor and this high (the box stub's sizes).
const VOLUME_BELOW := 0.5
const VOLUME_H := 3.5
## How far past the current floor's edge a threshold reaches: one stride past the wall, well inside the
## stride the server accepts across a doorway (a size of a volume, not a rule).
const THRESHOLD_DEPTH := 1.0
## The parts a style has.
const PARTS := ["wall", "floor", "opening", "solid"]
## The client's plain grey, per part (the box stub's colours).
const PLAIN_COLOUR := {
	"wall": Color(0.62, 0.62, 0.60), "floor": Color(0.48, 0.47, 0.44), "solid": Color(0.55, 0.55, 0.53),
}
const PLAIN_ROUGH := 0.85

## The parsed file ({} when absent or faulty).
var file := {}
var problems: PackedStringArray = []
## place id -> its drawing node.
var nodes := {}
## place id -> [shape, origin] it was drawn from.
var _built := {}
## place id -> its scene volume: the floor once drawn, else a threshold.
var volumes := {}
var _next := 0
## The scene root the volumes hang directly under (the depiction's world node).
var _root: Node3D = null


## Reads the pack's style file. A missing or faulty file leaves every place plain grey, named in
## [member problems].
static func load_pack(pack_dir: String) -> SliceShells:
	var s := SliceShells.new()
	var path := pack_dir.path_join(BINDING_FILE)
	if not FileAccess.file_exists(path):
		return s
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(path))
	if typeof(parsed) == TYPE_DICTIONARY and int(parsed.get("format", 0)) == 1:
		s.file = parsed
	else:
		s.problems.append("%s is not a format-1 shells file" % path)
	for p in s.problems:
		push_warning("shells: " + p)
	return s


## Draw `obs`'s place when it is a shell and its disclosed shape changed, and give every shell place
## one doorway away that is not drawn yet its threshold. Returns true when anything was (re)built.
func observe(obs: MineWorldObservation, layout: SliceLayout, world_root: Node3D) -> bool:
	var here := obs.place()
	if here == "" or world_root == null or not layout.is_drawn(here):
		return false
	_root = world_root
	var shape: Variant = obs.component_value(here, "place-shape")
	if typeof(shape) != TYPE_DICTIONARY:
		return false
	var changed := false
	if layout.drawn_as.get(here, "") == SliceLayout.SHELL:
		changed = _draw(world_root, layout, here, shape)
	for p in layout.passages.get(here, []):
		var to: String = p["to"]
		if layout.drawn_as.get(to, "") == SliceLayout.SHELL and layout.is_drawn(to) and not volumes.has(to):
			_threshold(world_root, layout, here, shape, p)
			changed = true
	return changed


## Whether a scene volume is one of these shells' (a floor or a threshold).
static func is_shell(volume: String) -> bool:
	return volume.begins_with(VOLUME_PREFIX)


## The volume the body is reported in: `found` (`SliceWorld.place_at`), except that a shell's volume
## never takes a point from the volume of the place the body is in (`current`). Glued frames overlap at
## a doorway by the two doorway insets (`ARC-45`), so without this the body would be reported in the
## next place while it still stands inside this one's walls; with it, the place changes only when the
## body has left this place's floor, which the disclosed walls allow only through a doorway.
static func keep(root: Node3D, found: String, current: String, p: Vector3) -> String:
	if found == current or current == "" or not is_shell(found):
		return found
	return current if contains(root, current, p) else found


## Whether the scene volume `id` (an Area3D under `root` whose meta `place_id` is `id`) holds `p`.
static func contains(root: Node3D, id: String, p: Vector3) -> bool:
	for c in root.get_children():
		if not (c is Area3D) or not c.has_meta("place_id") or str(c.get_meta("place_id")) != id:
			continue
		var a := c as Area3D
		var box := ((a.get_child(0) as CollisionShape3D).shape as BoxShape3D).size
		var local := p - a.position
		if absf(local.x) <= box.x * 0.5 and absf(local.y) <= box.y * 0.5 and absf(local.z) <= box.z * 0.5:
			return true
	return false


## The outward normal (place frame: x east, y north) of the floor edge nearest a doorway point (mm).
static func outward(shape: Dictionary, doorway_mm: Variant) -> Vector2:
	var floor_d: Dictionary = shape.get("floor", {})
	var lo := _m(floor_d.get("min", {}))
	var hi := _m(floor_d.get("max", {}))
	var d := _m(doorway_mm)
	var gaps := [d.x - lo.x, hi.x - d.x, d.y - lo.y, hi.y - d.y]
	var normals := [Vector2(-1, 0), Vector2(1, 0), Vector2(0, -1), Vector2(0, 1)]
	var best := 0
	for i in gaps.size():
		if gaps[i] < gaps[best]:
			best = i
	return normals[best]


## Drop everything drawn: the world instance changed (`ARC-45` point 2).
func forget() -> void:
	for place in nodes:
		if is_instance_valid(nodes[place]):
			(nodes[place] as Node).queue_free()
	nodes.clear()
	_built.clear()
	for place in volumes.keys():
		_free_volume(place)
	volumes.clear()


## The style of a place with `tags`: the plain parts, changed by the file's default, then by the first
## tag (in disclosed order, each tried with its `.`-prefixes) the file binds. A part that is null is not
## drawn (`"wall": null`: a street's edges are its neighbours' fronts).
func style_for(tags: Array) -> Dictionary:
	var style := _merge(_plain(), file.get("default", {}))
	var by_tag: Variant = file.get("tags", {})
	if typeof(by_tag) != TYPE_DICTIONARY:
		return style
	for t in tags:
		for key in _prefixes(String(t)):
			if by_tag.has(key) and typeof(by_tag[key]) == TYPE_DICTIONARY:
				return _merge(style, by_tag[key])
	return style


# --- drawing -------------------------------------------------------------------------------------

func _draw(world_root: Node3D, layout: SliceLayout, place: String, shape: Dictionary) -> bool:
	var origin := layout.to_scene(place, {"x": 0, "y": 0})
	if _built.has(place) and _built[place][0] == shape and _built[place][1] == origin:
		return false
	if nodes.has(place) and is_instance_valid(nodes[place]):
		(nodes[place] as Node).queue_free()
	_free_volume(place)
	var root := world_root.get_node_or_null(NODE_NAME) as Node3D
	if root == null:
		root = Node3D.new()
		root.name = NODE_NAME
		world_root.add_child(root)
	var node := Node3D.new()
	node.name = place.validate_node_name()
	root.add_child(node)
	nodes[place] = node
	_built[place] = [shape, origin]
	build(node, shape, layout.passages.get(place, []), origin, style_for(layout.tags.get(place, [])))
	var floor_d: Dictionary = shape.get("floor", {})
	var lo := _m(floor_d.get("min", {}))
	var hi := _m(floor_d.get("max", {}))
	volumes[place] = _volume(world_root, origin, lo, hi)
	layout.set_volume(place, volumes[place])
	print("[shells] place %s drawn from its disclosure: floor %s..%s m, style %s"
		% [place, lo, hi, JSON.stringify(style_for(layout.tags.get(place, [])))])
	return true


## Draws one place's shell under `node`: `shape` its disclosed `place-shape`, `passages` as the layout
## keeps them, `origin` where its frame's (0, 0) stands in the scene, `style` from `style_for`.
func build(node: Node3D, shape: Dictionary, passages: Array, origin: Vector3, style: Dictionary) -> void:
	var floor_d: Dictionary = shape.get("floor", {})
	var lo := _m(floor_d.get("min", {}))
	var hi := _m(floor_d.get("max", {}))
	if style["floor"] != null:
		_box(node, origin, lo, hi, -FLOOR_T, 0.0, _material("floor", style["floor"]), true)
	if style["wall"] != null:
		var doors: Array = []
		for p in passages:
			if typeof(p) == TYPE_DICTIONARY and typeof(p.get("here")) == TYPE_DICTIONARY:
				doors.append(_m(p["here"]))
		var t := SliceClientBodies.WALL_THICKNESS
		var mat := _material("wall", style["wall"])
		var tall := float(style["wall"].get("height_m", 2.7))
		var opening: Dictionary = style["opening"] if style["opening"] != null else {}
		var gap := float(opening.get("width_m", SliceClientBodies.DOOR_GAP))
		var head := minf(float(opening.get("height_m", 2.1)), tall)
		# each edge: its outer box from `a` to `b`, which way it runs, and its line
		_wall(node, origin, Vector2(lo.x - t, lo.y - t), Vector2(hi.x + t, lo.y), doors, true, lo.y, tall, gap, head, mat)
		_wall(node, origin, Vector2(lo.x - t, hi.y), Vector2(hi.x + t, hi.y + t), doors, true, hi.y, tall, gap, head, mat)
		_wall(node, origin, Vector2(lo.x - t, lo.y), Vector2(lo.x, hi.y), doors, false, lo.x, tall, gap, head, mat)
		_wall(node, origin, Vector2(hi.x, lo.y), Vector2(hi.x + t, hi.y), doors, false, hi.x, tall, gap, head, mat)
	var solids: Variant = shape.get("solids", [])
	if style["solid"] != null and typeof(solids) == TYPE_ARRAY:
		var mat := _material("solid", style["solid"])
		for s in solids:
			if typeof(s) != TYPE_DICTIONARY or s.has("furnishing"):
				continue
			_box(node, origin, _m(s.get("min", {})), _m(s.get("max", {})), 0.0,
				float(s.get("height", 0)) / 1000.0, mat, false)


## One wall from `a` to `b` (metres, place frame), split at every doorway within the colliders'
## DOOR_EDGE of its line, with a lintel from `head` to `tall` over each opening.
func _wall(node: Node3D, origin: Vector3, a: Vector2, b: Vector2, doors: Array, along_x: bool, line: float,
		tall: float, gap: float, head: float, mat: Material) -> void:
	var centres := SliceSpans.openings(doors, along_x, line, SliceClientBodies.DOOR_EDGE)
	var start := a.x if along_x else a.y
	var stop := b.x if along_x else b.y
	for piece in SliceSpans.split(start, stop, centres, gap):
		_span(node, origin, a, b, along_x, piece[0], piece[1], 0.0, tall, mat)
	if head < tall:
		for c in centres:
			_span(node, origin, a, b, along_x, maxf(c - gap * 0.5, start), minf(c + gap * 0.5, stop), head, tall, mat)


func _span(node: Node3D, origin: Vector3, a: Vector2, b: Vector2, along_x: bool, from: float, to: float,
		bottom: float, top: float, mat: Material) -> void:
	var lo := Vector2(from, a.y) if along_x else Vector2(a.x, from)
	var hi := Vector2(to, b.y) if along_x else Vector2(b.x, to)
	_box(node, origin, lo, hi, bottom, top, mat, false)


## A box of footprint `lo`..`hi` (metres, place frame), from `bottom` to `top` above the floor; a floor
## the feet stand on collides on `Build.LAYER_WORLD`, nothing else here collides.
static func _box(node: Node3D, origin: Vector3, lo: Vector2, hi: Vector2, bottom: float, top: float,
		mat: Material, ground: bool) -> void:
	if hi.x - lo.x <= 0.0 or hi.y - lo.y <= 0.0 or top - bottom <= 0.0:
		return
	var a := _at(origin, lo)
	var b := _at(origin, hi)
	Build.box(node, Vector3((a.x + b.x) * 0.5, origin.y + (bottom + top) * 0.5, (a.z + b.z) * 0.5),
		Vector3(absf(b.x - a.x), top - bottom, absf(b.z - a.z)), mat, 0.0, ground, Build.LAYER_WORLD)


## A scene volume (an Area3D directly under `world_root`, as `SliceWorld.place_at` reads them) over the
## frame rectangle `lo`..`hi`; returns its id.
func _volume(world_root: Node3D, origin: Vector3, lo: Vector2, hi: Vector2) -> String:
	var id := "%s%d" % [VOLUME_PREFIX, _next]
	_next += 1
	var a := _at(origin, lo)
	var b := _at(origin, hi)
	SliceWorld._place(world_root, id, Vector3((a.x + b.x) * 0.5, origin.y + VOLUME_H * 0.5 - VOLUME_BELOW,
		(a.z + b.z) * 0.5), Vector3(absf(b.x - a.x), VOLUME_H, absf(b.z - a.z)))
	return id


## The threshold into `p["to"]`: past the current floor's edge at the doorway, a gap wide and
## THRESHOLD_DEPTH deep, in the current place's frame.
func _threshold(world_root: Node3D, layout: SliceLayout, here: String, shape: Dictionary, p: Dictionary) -> void:
	var floor_d: Dictionary = shape.get("floor", {})
	var lo := _m(floor_d.get("min", {}))
	var hi := _m(floor_d.get("max", {}))
	var d := _m(p["here"])
	var out := outward(shape, p["here"])
	var half := SliceClientBodies.DOOR_GAP * 0.5
	var a := Vector2.ZERO
	var b := Vector2.ZERO
	if out.x != 0.0:
		var edge := hi.x if out.x > 0.0 else lo.x
		a = Vector2(minf(edge, edge + out.x * THRESHOLD_DEPTH), d.y - half)
		b = Vector2(maxf(edge, edge + out.x * THRESHOLD_DEPTH), d.y + half)
	else:
		var edge := hi.y if out.y > 0.0 else lo.y
		a = Vector2(d.x - half, minf(edge, edge + out.y * THRESHOLD_DEPTH))
		b = Vector2(d.x + half, maxf(edge, edge + out.y * THRESHOLD_DEPTH))
	var to: String = p["to"]
	volumes[to] = _volume(world_root, layout.to_scene(here, {"x": 0, "y": 0}), a, b)
	layout.set_volume(to, volumes[to])
	print("[shells] place %s: a threshold past the doorway at %s m of place %s" % [to, d, here])


func _free_volume(place: String) -> void:
	if not volumes.has(place):
		return
	var id: String = volumes[place]
	volumes.erase(place)
	if _root == null or not is_instance_valid(_root):
		return
	for c in _root.get_children():
		if c is Area3D and c.has_meta("place_id") and str(c.get_meta("place_id")) == id:
			# out of the tree now, so `SliceWorld.place_at` no longer finds it this frame
			_root.remove_child(c)
			c.queue_free()


# --- style ---------------------------------------------------------------------------------------

func _material(part: String, spec: Variant) -> Material:
	var plain: Color = PLAIN_COLOUR.get(part, PLAIN_COLOUR["wall"])
	var slug := String(spec.get("material", "")) if typeof(spec) == TYPE_DICTIONARY else ""
	var table: Variant = file.get("materials", {})
	if slug == "" or typeof(table) != TYPE_DICTIONARY or typeof(table.get(slug)) != TYPE_DICTIONARY:
		return Mats.paint(plain, PLAIN_ROUGH)
	var m: Dictionary = table[slug]
	var c: Variant = m.get("colour", [])
	var colour := Color(float(c[0]), float(c[1]), float(c[2])) \
		if typeof(c) == TYPE_ARRAY and c.size() == 3 else plain
	var rough := float(m.get("roughness", PLAIN_ROUGH))
	var texture := String(m.get("texture", ""))
	if texture == "":
		return Mats.paint(colour, rough)
	return Mats.pbr(texture, float(m.get("metres", 2.0)), colour, rough)


static func _plain() -> Dictionary:
	return {
		"wall": {"height_m": 2.7},
		"floor": {},
		"opening": {"width_m": SliceClientBodies.DOOR_GAP, "height_m": 2.1},
		"solid": {},
	}


## `over`'s parts over `base`'s: a part `over` sets to null is null; a part it sets is merged key by key.
static func _merge(base: Dictionary, over: Variant) -> Dictionary:
	var out := base.duplicate(true)
	if typeof(over) != TYPE_DICTIONARY:
		return out
	for part in PARTS:
		if not over.has(part):
			continue
		if over[part] == null:
			out[part] = null
			continue
		if typeof(over[part]) != TYPE_DICTIONARY:
			continue
		var merged: Dictionary = (out[part] as Dictionary).duplicate() if out[part] != null else {}
		merged.merge(over[part], true)
		out[part] = merged
	return out


## `a.b.c` -> ["a.b.c", "a.b", "a"].
static func _prefixes(tag: String) -> PackedStringArray:
	var out := PackedStringArray()
	var t := tag
	while t != "":
		out.append(t)
		var cut := t.rfind(".")
		t = t.substr(0, cut) if cut > 0 else ""
	return out


## A point in the place frame (metres) in the scene, at the origin's height.
static func _at(origin: Vector3, v: Vector2) -> Vector3:
	var p := MineWorldSpace.to_3d({"x": v.x * 1000.0, "y": v.y * 1000.0})
	return origin + Vector3(p.x, 0.0, p.z)


## A disclosed `{x, y}` in millimetres -> metres, (east, north).
static func _m(v: Variant) -> Vector2:
	if typeof(v) != TYPE_DICTIONARY:
		return Vector2.ZERO
	return Vector2(float(v.get("x", 0)) / 1000.0, float(v.get("y", 0)) / 1000.0)
