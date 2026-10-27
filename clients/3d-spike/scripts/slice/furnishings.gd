## The furniture of every place this client has stood in, drawn from the world's data (`ARC-307`;
## `.structured-coding/plans/mvp1/pr-vf-furnishings-from-data.md` §3.5).
##
## The `bodies` System Pack discloses a place's furnishings inside its `place-shape`: each piece's key,
## kind, the centre of its drawn footprint (`at`, millimetres, place frame) and the way it faces
## (`facing`, millidegrees clockwise from north). The 3D Presentation Pack binds a kind to a model
## (`furniture/furnishings.yaml`, `PRESENTATION.md` §5). This file joins the two:
##
##     kind bound              -> the bound model, centred on `at`, yaw = front - facing; with
##                                `repeat: true` copied along its solids' longer side to fill it
##     kind bound to a `form`  -> a generic form (`table`, `hull`) built over its own solids in the
##                                binding's material, for a piece the model library has no model of
##     a `.`-prefix bound      -> that binding (`chair.painted` -> `chair`)
##     nothing bound           -> a plain box over each of its solids, and an `unbound` row in the
##                                `FURNISHED` report: a disclosed piece is never left undrawn,
##                                because an undrawn solid is an invisible wall (I-FD-5)
##
## It builds NO colliders: `client_bodies.gd` builds one from every disclosed solid, these included
## (I-FD-3). It names no place, no world and no kind (I-FD-2). A place stays furnished after the
## observer leaves it, as the layout keeps what it learned.
class_name SliceFurnishings
extends RefCounted

## The pack's binding file, relative to the 3D Presentation Pack's folder.
const BINDING_FILE := "furniture/furnishings.yaml"
## The node every place's furnishings hang under.
const NODE_NAME := "Furnishings"
## The recorded observation the standalone slice draws from (Q-FD-5): the anchor place's disclosed
## `place-shape`, written by `tools/cli/tests/furnishings_fixture.rs` and failing on drift.
const RECORDED := "../../tests/fixtures/observations/furnishings/social-cafe-cafe.json"
## The neutral timber of an unbound piece's plain boxes.
const PLAIN_TIMBER := Color(0.52, 0.40, 0.28)

## kind -> its binding, as the pack states it.
var kinds: Dictionary = {}
var problems: PackedStringArray = []
## place id -> the `place-shape` its node was built from.
var _built_from := {}
## place id -> its node.
var nodes := {}
## place id -> the FURNISHED rows of that place: one per furnishing, in disclosed order.
var rows := {}
## place id -> where its frame's origin lies in its node's parent, and the shape it was built from.
var _origins := {}
## The world the drawn places belong to, for the dump ("" when unknown).
var world_id := ""


## Reads the pack's binding. A missing or faulty file leaves every kind unbound (plain boxes), named
## in [member problems].
static func load_pack(pack_dir: String) -> SliceFurnishings:
	var f := SliceFurnishings.new()
	var path := pack_dir.path_join(BINDING_FILE)
	var file := FileAccess.open(path, FileAccess.READ)
	if file == null:
		f.problems.append("cannot read %s" % path)
	else:
		var parsed: Variant = JSON.parse_string(file.get_as_text())
		if typeof(parsed) == TYPE_DICTIONARY and int(parsed.get("format", 0)) == 1 \
				and typeof(parsed.get("kinds")) == TYPE_DICTIONARY:
			f.kinds = parsed["kinds"]
		else:
			f.problems.append("%s is not a format-1 furnishings binding" % path)
	for p in f.problems:
		push_warning("furnishings: " + p)
	return f


## Build `obs`'s place's furnishings when its disclosed `place-shape` changed. Returns true when it
## (re)built.
func observe(obs: MineWorldObservation, layout: SliceLayout, world_root: Node3D) -> bool:
	var here := obs.place()
	if here == "" or not layout.is_drawn(here):
		return false
	var shape: Variant = obs.component_value(here, "place-shape")
	if typeof(shape) != TYPE_DICTIONARY or shape == _built_from.get(here):
		return false
	_build_into(world_root, here, shape, layout.to_scene(here, {"x": 0, "y": 0}))
	return true


## The standalone slice (no server): the recorded observation's place, drawn under `parent` with
## the place's frame origin at `origin` (parent-local). Returns the place id drawn, or "".
func draw_recorded(parent: Node3D, origin: Vector3) -> String:
	var parsed := read_recorded()
	if parsed.is_empty():
		return ""
	return draw_parsed(parent, parsed, origin)


## The recorded observations beside RECORDED whose names begin `<world>-<prefix>`, as absolute paths
## (S29-e: `social-cafe-station-<place>.json`, written by `tools/cli/tests/station_furnishings_fixture.rs`).
static func recorded_files(world: String, prefix: String) -> PackedStringArray:
	var dir := ProjectSettings.globalize_path("res://").path_join(RECORDED).simplify_path().get_base_dir()
	var out := PackedStringArray()
	for f in DirAccess.get_files_at(dir):
		if f.begins_with("%s-%s" % [world, prefix]) and f.ends_with(".json"):
			out.append(dir.path_join(f))
	out.sort()
	return out


## A recorded file's place tags ([] when it states none).
static func recorded_tags(path: String) -> PackedStringArray:
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(path))
	var out := PackedStringArray()
	if typeof(parsed) == TYPE_DICTIONARY and typeof(parsed.get("tags")) == TYPE_ARRAY:
		for t in parsed["tags"]:
			out.append(String(t))
	return out


## `draw_recorded` for any recorded observation file.
func draw_recorded_file(path: String, parent: Node3D, origin: Vector3) -> String:
	var parsed := read_recorded_file(path)
	if parsed.is_empty():
		return ""
	return draw_parsed(parent, parsed, origin)


## The recorded observation ({world, place, place-shape}), or {} when it cannot be read.
static func read_recorded() -> Dictionary:
	return read_recorded_file(ProjectSettings.globalize_path("res://").path_join(RECORDED).simplify_path())


## A recorded observation file ({world, place, place-shape}), or {} when it cannot be read.
static func read_recorded_file(path: String) -> Dictionary:
	var file := FileAccess.open(path, FileAccess.READ)
	if file == null:
		push_warning("furnishings: cannot read the recorded observation %s" % path)
		return {}
	var parsed: Variant = JSON.parse_string(file.get_as_text())
	if typeof(parsed) != TYPE_DICTIONARY or typeof(parsed.get("place-shape")) != TYPE_DICTIONARY:
		push_warning("furnishings: %s holds no place-shape" % path)
		return {}
	return parsed


## Draws a recorded observation (`read_recorded`) under `parent`, the place's frame origin at `origin`.
func draw_parsed(parent: Node3D, parsed: Dictionary, origin: Vector3) -> String:
	var place := String(parsed.get("place", "recorded"))
	world_id = String(parsed.get("world", ""))
	_build_into(parent, place, parsed["place-shape"], origin)
	return place


func _build_into(parent: Node3D, place: String, shape: Dictionary, origin: Vector3) -> void:
	if nodes.has(place) and is_instance_valid(nodes[place]):
		(nodes[place] as Node).queue_free()
	var root := parent.get_node_or_null(NODE_NAME) as Node3D
	if root == null:
		root = Node3D.new()
		root.name = NODE_NAME
		parent.add_child(root)
	var node := Node3D.new()
	node.name = place.validate_node_name()
	root.add_child(node)
	nodes[place] = node
	_built_from[place] = shape
	_origins[place] = origin
	rows[place] = build(node, place, shape, origin)
	var dir := dump_dir()
	if dir != "":
		dump(dir, place)


## Draws every furnishing of `shape` under `node`, the place's frame origin at `origin`. Returns the
## FURNISHED rows: { key, kind, bound, model, at (scene), yaw, unbound }.
func build(node: Node3D, place: String, shape: Dictionary, origin: Vector3) -> Array:
	var out := []
	var list: Variant = shape.get("furnishings", [])
	if typeof(list) != TYPE_ARRAY:
		return out
	var solids: Variant = shape.get("solids", [])
	for f in list:
		if typeof(f) != TYPE_DICTIONARY:
			continue
		var key := String(f.get("key", ""))
		var kind := String(f.get("kind", ""))
		var bound := resolve(kind)
		var at := origin + MineWorldSpace.to_3d(f.get("at", {}))
		at.y = origin.y
		var row := {"key": key, "kind": kind, "bound": bound, "at": at, "unbound": bound == ""}
		if bound == "":
			_plain(node, key, solids if typeof(solids) == TYPE_ARRAY else [], origin)
			row["model"] = ""
			row["yaw"] = 0.0
			push_warning("furnishings: %s/%s: no binding for kind %s; drawn as plain boxes"
				% [place, key, kind])
		elif kinds[bound].has("piece"):
			var b: Dictionary = kinds[bound]
			var yaw := yaw_of(b, int(f.get("facing", 0)))
			var m := _piece(node, b, key, solids if typeof(solids) == TYPE_ARRAY else [], at, yaw)
			row["model"] = "piece:" + String(b.get("piece", ""))
			row["yaw"] = yaw
			if m != null and m.has_meta("mw_fixture"):
				StationFixtures.register(m, "", place)
		else:
			var b: Dictionary = kinds[bound]
			var yaw := yaw_of(b, int(f.get("facing", 0)))
			var own := _named(solids if typeof(solids) == TYPE_ARRAY else [], key)
			var m: Node3D = null
			if b.get("form") != null:
				m = _form(node, String(b["form"]), b, own, origin, yaw)
				row["model"] = "form:" + String(b["form"])
			else:
				m = SliceProps.put(node, String(b.get("model", "")), at, yaw, _tint(b.get("tint")),
					0.28, 0.86, b.get("parts", []) if typeof(b.get("parts")) == TYPE_ARRAY else [])
				row["model"] = String(b.get("model", ""))
				if bool(b.get("repeat", false)):
					_repeat(node, m, b, own, origin, yaw)
			m.name = key.validate_node_name()
			row["yaw"] = yaw
			row["dressing"] = _dress(node, b, place, key, at, yaw)
		out.append(row)
	# A piece is many primitives (a bench row is ~20 boxes): merged like the rest of the slice, by
	# material and cell. Models are not primitives, and a fixture's toggled parts carry surface
	# materials, so neither is touched (`batch.gd`).
	if out.any(func(r: Dictionary) -> bool: return String(r.get("model", "")).begins_with("piece:")):
		SliceBatch.merge(node)
	return out


## The solids that name furnishing `key`.
static func _named(solids: Array, key: String) -> Array:
	var out := []
	for s in solids:
		if typeof(s) == TYPE_DICTIONARY and String(s.get("furnishing", "")) == key:
			out.append(s)
	return out


## A solid's footprint in the scene: [centre (on the place's floor), size x, size z, height].
static func _solid_box(s: Dictionary, origin: Vector3) -> Array:
	var lo := MineWorldSpace.to_3d(s.get("min", {}))
	var hi := MineWorldSpace.to_3d(s.get("max", {}))
	var c := origin + (lo + hi) * 0.5
	return [Vector3(c.x, origin.y, c.z), absf(hi.x - lo.x), absf(hi.z - lo.z), float(s.get("height", 0)) / 1000.0]


## `repeat: true`: the model is copied along the longer side of the piece's solids until it fills it
## (a bench longer than the model, a run of shelving), evenly spaced; `m` is the first copy, moved.
func _repeat(node: Node3D, m: Node3D, b: Dictionary, own: Array, origin: Vector3, yaw: float) -> void:
	if own.is_empty():
		return
	var box: Array = _solid_box(own[0], origin)
	var along_x: bool = box[1] >= box[2]
	var span: float = maxf(box[1], box[2])
	var drawn := SliceProps.placed_box(m)
	var unit := maxf(drawn.size.x if along_x else drawn.size.z, 0.1)
	var n := maxi(1, roundi(span / unit))
	var step := span / float(n)
	var axis := Vector3(1, 0, 0) if along_x else Vector3(0, 0, 1)
	var first: Vector3 = (box[0] as Vector3) - axis * (span * 0.5 - step * 0.5)
	# `put` centred the first copy's box on the piece's `at`; it moves to the first slot
	var placed := drawn.position + drawn.size * 0.5
	m.position += Vector3(first.x - placed.x, 0.0, first.z - placed.z)
	for k in range(1, n):
		var copy := SliceProps.put(node, String(b.get("model", "")), first + axis * (step * k), yaw,
			_tint(b.get("tint")), 0.28, 0.86, b.get("parts", []) if typeof(b.get("parts")) == TYPE_ARRAY else [])
		copy.name = "%s_%d" % [m.name, k]


## A piece drawn by a generic form over its own solids, in the binding's `material` (a texture slug)
## and `tint`, instead of a model:
##   table  a top `top_m` thick at the solid's height on four legs `leg_m` square, inset `leg_m`
##   bench  the same, a backless seat: a sitter may face either side of it
##   hull   a boat's hull on a cradle: the solid's footprint tapered to a bow on the side it faces,
##          the cradle's two chocks `cradle_m` tall under it
## Returns the node the piece hangs under.
func _form(node: Node3D, form: String, b: Dictionary, own: Array, origin: Vector3, yaw: float) -> Node3D:
	var piece := Node3D.new()
	node.add_child(piece)
	var mat := Mats.pbr(String(b.get("material", "wood_table_worn")), 1.0, _tint(b.get("tint")))
	for s in own:
		var box: Array = _solid_box(s, origin)
		var c: Vector3 = box[0]
		var h: float = box[3]
		match form:
			"table", "bench":
				var top := float(b.get("top_m", 0.05))
				var leg := float(b.get("leg_m", 0.07))
				Build.box(piece, c + Vector3(0, h - top * 0.5, 0), Vector3(box[1], top, box[2]), mat)
				for sx in [-1.0, 1.0]:
					for sz in [-1.0, 1.0]:
						var at := c + Vector3(sx * (box[1] * 0.5 - leg * 1.5), (h - top) * 0.5,
							sz * (box[2] * 0.5 - leg * 1.5))
						Build.box(piece, at, Vector3(leg, h - top, leg), mat)
			"hull":
				_hull(piece, c, box[1], box[2], h, float(b.get("cradle_m", 0.3)), yaw, mat)
			_:
				Build.box(piece, c + Vector3(0, h * 0.5, 0), Vector3(box[1], h, box[2]), mat)
	return piece


## A hull over a footprint of `w` x `d` (scene x, z), `h` tall, standing on a cradle `cradle` tall; the
## bow toward `yaw`'s forward (the model-free equivalent of front_md 0).
static func _hull(g: Node3D, c: Vector3, w: float, d: float, h: float, cradle: float, yaw: float,
		mat: Material) -> void:
	var fwd := Vector3(sin(yaw), 0, cos(yaw))
	var long := maxf(w, d)
	var beam := minf(w, d)
	var side := Vector3(fwd.z, 0, -fwd.x)
	var chock := Mats.pbr("weathered_brown_planks", 1.0)
	for k in [-0.3, 0.3]:
		Build.box(g, c + fwd * (long * k) + Vector3(0, cradle * 0.5, 0), Vector3(beam, cradle, 0.2), chock,
			atan2(fwd.x, fwd.z))
	var y0 := c.y + cradle
	var y1 := c.y + h
	var stern := c - fwd * (long * 0.5)
	var bow := c + fwd * (long * 0.5)
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	var half := beam * 0.5
	# the gunwale: a stern transom, full beam to two thirds, then in to the stem
	var top := [stern - side * half * 0.8, c + side * half, bow, c - side * half, stern + side * half * 0.8]
	var ring := []
	for p in top:
		ring.append(Vector3(p.x, y1, p.z))
	# sides down to the keel line, narrowed to a third of the beam
	for k in ring.size():
		var a: Vector3 = ring[k]
		var e: Vector3 = ring[(k + 1) % ring.size()]
		var a2 := Vector3(lerpf(a.x, c.x, 0.66), y0, lerpf(a.z, c.z, 0.66))
		var e2 := Vector3(lerpf(e.x, c.x, 0.66), y0, lerpf(e.z, c.z, 0.66))
		for v in [a, e, e2, a, e2, a2, a, e2, e, a, a2, e2]:
			st.add_vertex(v)
	for k in range(1, ring.size() - 1):
		var a := Vector3((ring[0] as Vector3).x, y0, (ring[0] as Vector3).z).lerp(Vector3(c.x, y0, c.z), 0.66)
		var e := Vector3((ring[k] as Vector3).x, y0, (ring[k] as Vector3).z).lerp(Vector3(c.x, y0, c.z), 0.66)
		var f := Vector3((ring[k + 1] as Vector3).x, y0, (ring[k + 1] as Vector3).z).lerp(Vector3(c.x, y0, c.z), 0.66)
		for v in [a, e, f, a, f, e]:
			st.add_vertex(v)
	st.generate_normals()
	var mi := MeshInstance3D.new()
	mi.mesh = st.commit()
	mi.material_override = mat
	g.add_child(mi)
## A kind bound to a piece (`piece`: a `StationPieces` builder, PRESENTATION.md §5): built to the
## footprint and height of the solids that name the furnishing, so the drawn piece is the disclosed one.
func _piece(node: Node3D, b: Dictionary, key: String, solids: Array, at: Vector3, yaw: float) -> Node3D:
	var lo := Vector2(INF, INF)
	var hi := Vector2(-INF, -INF)
	var height := 0.0
	for s in solids:
		if typeof(s) != TYPE_DICTIONARY or String(s.get("furnishing", "")) != key:
			continue
		var a := Vector2(float(s.get("min", {}).get("x", 0)), float(s.get("min", {}).get("y", 0))) / 1000.0
		var c := Vector2(float(s.get("max", {}).get("x", 0)), float(s.get("max", {}).get("y", 0))) / 1000.0
		lo = Vector2(minf(lo.x, a.x), minf(lo.y, a.y))
		hi = Vector2(maxf(hi.x, c.x), maxf(hi.y, c.y))
		height = maxf(height, float(s.get("height", 0)) / 1000.0)
	if lo.x == INF or height <= 0.0:
		push_warning("furnishings: %s names no solid; a piece is drawn to its solids" % key)
		return null
	var size := hi - lo
	if absf(sin(yaw)) > 0.7:
		size = Vector2(size.y, size.x)
	var m := StationPieces.furnishing(node, String(b.get("piece", "")), size, height,
		StationPieces.style_of(String(b.get("style", ""))))
	if m == null:
		push_warning("furnishings: %s: no piece %s" % [key, b.get("piece", "")])
		return null
	m.name = key.validate_node_name()
	m.transform = Transform3D(Basis(Vector3.UP, yaw), at)
	return m


## The binding a kind resolves to: itself, else each shorter `.`-prefix; "" for none.
func resolve(kind: String) -> String:
	var k := kind
	while k != "":
		if kinds.has(k) and typeof(kinds[k]) == TYPE_DICTIONARY:
			return k
		var cut := k.rfind(".")
		k = k.substr(0, cut) if cut > 0 else ""
	return ""


## Scene yaw, radians about +Y (counter-clockwise from above), of a piece facing `facing_md`
## (clockwise from north) under binding `b`: front - facing.
static func yaw_of(b: Dictionary, facing_md: int) -> float:
	return deg_to_rad(float(int(b.get("front_md", 0)) - facing_md) / 1000.0)


## FNV-1a 32 of `text`: the stable choice of a piece's dressing set.
static func fnv1a32(text: String) -> int:
	var h := 0x811c9dc5
	for byte in text.to_utf8_buffer():
		h = ((h ^ byte) * 0x01000193) & 0xffffffff
	return h


## The hand-scale dressing that follows the piece: one set of the binding's, chosen by
## `fnv1a32(place + "/" + key)`. Each prop sits at its `at` in the model's frame, `up` above the floor.
func _dress(node: Node3D, b: Dictionary, place: String, key: String, at: Vector3, yaw: float) -> Array:
	var placed := []
	var sets: Variant = b.get("dressing", [])
	if typeof(sets) != TYPE_ARRAY or (sets as Array).is_empty():
		return placed
	var chosen: Variant = sets[fnv1a32(place + "/" + key) % (sets as Array).size()]
	if typeof(chosen) != TYPE_ARRAY:
		return placed
	var basis := Basis(Vector3.UP, yaw)
	for p in chosen:
		if typeof(p) != TYPE_DICTIONARY:
			continue
		var off: Array = p.get("at", [0.0, 0.0])
		var local := basis * Vector3(float(off[0]), 0.0, float(off[1]))
		var pos := at + local + Vector3(0.0, float(p.get("up", 0.0)), 0.0)
		var item_yaw := yaw + deg_to_rad(float(p.get("yaw_md", 0)) / 1000.0)
		SliceProps.put(node, String(p.get("model", "")), pos, item_yaw, _tint(p.get("tint")),
			0.28, 0.86, p.get("parts", []) if typeof(p.get("parts")) == TYPE_ARRAY else [])
		placed.append({"model": String(p.get("model", "")), "at": pos, "yaw": item_yaw,
			"parts": p.get("parts", [])})
	return placed


## An unbound piece: a plain box over each solid that names it.
func _plain(node: Node3D, key: String, solids: Array, origin: Vector3) -> void:
	var mat := Mats.paint(PLAIN_TIMBER, 0.8)
	for s in solids:
		if typeof(s) != TYPE_DICTIONARY or String(s.get("furnishing", "")) != key:
			continue
		var lo := MineWorldSpace.to_3d(s.get("min", {}))
		var hi := MineWorldSpace.to_3d(s.get("max", {}))
		var h := float(s.get("height", 0)) / 1000.0
		var centre := origin + (lo + hi) * 0.5
		Build.box(node, Vector3(centre.x, origin.y + h * 0.5, centre.z),
			Vector3(absf(hi.x - lo.x), h, absf(hi.z - lo.z)), mat)


static func _tint(v: Variant) -> Color:
	if typeof(v) == TYPE_ARRAY and (v as Array).size() >= 3:
		return Color(float(v[0]), float(v[1]), float(v[2]))
	return Color.WHITE


## The `FURNISHED` report: every place built, each furnishing with its kind, its binding and where it
## is DRAWN (the placed model's AABB centre on the floor, scene metres), so the report is an instrument
## of the scene, not a copy of the observation.
func furnished() -> Dictionary:
	var places := {}
	var total := 0
	var unbound := 0
	for place in rows:
		var list := []
		var node: Node3D = nodes.get(place)
		for r in rows[place]:
			var drawn: Variant = null
			if node != null and r["model"] != "":
				var m := node.get_node_or_null(NodePath(String(r["key"]).validate_node_name())) as Node3D
				if m != null:
					var box := SliceProps.placed_box(m)
					var c := box.position + box.size * 0.5
					drawn = [snappedf(c.x, 0.001), snappedf(c.z, 0.001)]
			list.append({"key": r["key"], "kind": r["kind"], "bound": r["bound"], "model": r["model"],
				"at": [snappedf((r["at"] as Vector3).x, 0.001), snappedf((r["at"] as Vector3).z, 0.001)],
				"drawn": drawn, "yaw": snappedf(float(r["yaw"]), 0.0001), "unbound": r["unbound"]})
			total += 1
			if r["unbound"]:
				unbound += 1
		places[place] = list
	return {"pieces": total, "unbound": unbound, "places": places}


## `--furnishings-dump=<dir>`, or "" when not asked.
static func dump_dir() -> String:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--furnishings-dump="):
			return a.trim_prefix("--furnishings-dump=")
	return ""


## S36-P5's input (design §3.7): `<dir>/<place>/furnishings.json`, each drawn piece with its glTF and
## the 4x4 (row-major) that takes the model's own metres to the place's frame (x east, y north, z up),
## the indices of the disclosed solids that name it, its dressing likewise, and the shell solids.
func dump(dir: String, place: String) -> void:
	var shape: Dictionary = _built_from[place]
	var origin: Vector3 = _origins[place]
	var node: Node3D = nodes[place]
	var solids: Array = shape.get("solids", []) if typeof(shape.get("solids")) == TYPE_ARRAY else []
	var pieces := []
	for r in rows[place]:
		var named := []
		for i in range(solids.size()):
			if typeof(solids[i]) == TYPE_DICTIONARY and String(solids[i].get("furnishing", "")) == r["key"]:
				named.append(i)
		var piece := {"key": r["key"], "kind": r["kind"], "bound": r["bound"], "model": r["model"],
			"solids": named, "dressing": []}
		var b: Dictionary = kinds.get(r["bound"], {}) if r["bound"] != "" else {}
		piece["parts"] = b.get("parts", [])
		var m := node.get_node_or_null(NodePath(String(r["key"]).validate_node_name())) as Node3D
		if m != null and not String(r["model"]).begins_with("form:") \
				and not String(r["model"]).begins_with("piece:"):
			piece["gltf"] = _gltf(String(r["model"]))
			piece["to_place"] = _to_place(m.transform, origin)
		for d in r.get("dressing", []):
			piece["dressing"].append({"model": d["model"], "parts": d["parts"], "gltf": _gltf(String(d["model"])),
				"at_place": _point(d["at"], origin)})
		pieces.append(piece)
	var shell := []
	for i in range(solids.size()):
		if typeof(solids[i]) == TYPE_DICTIONARY and String(solids[i].get("furnishing", "")) == "":
			shell.append({"solid": i})
	var out := {"format": 1, "world": world_id, "place": place, "furnishings": pieces, "shell": shell}
	var to := dir.path_join(place.validate_filename())
	DirAccess.make_dir_recursive_absolute(to)
	var f := FileAccess.open(to.path_join("furnishings.json"), FileAccess.WRITE)
	if f == null:
		push_warning("furnishings: cannot write the dump under %s" % to)
		return
	f.store_string(JSON.stringify(out, "  ") + "\n")
	print("furnishings dump %s: %d pieces" % [to.path_join("furnishings.json"), pieces.size()])


static func _gltf(slug: String) -> String:
	return "assets/models/%s/%s.gltf" % [slug, slug]


## Scene (parent frame, metres, +Y up, north -Z) -> the place's frame (x east, y north, z up).
static func _point(p: Vector3, origin: Vector3) -> Array:
	var d := p - origin
	return [d.x, -d.z, d.y]


## The model-local -> place-frame transform, as 16 floats row-major. Scene = B * m + o; the place frame
## is C * (scene - origin), C taking (x, y, z) to (x, -z, y).
static func _to_place(t: Transform3D, origin: Vector3) -> Array:
	var rows_of := [Vector3(1, 0, 0), Vector3(0, 0, -1), Vector3(0, 1, 0)]
	var o := t.origin - origin
	var out := []
	for c in rows_of:
		var row_basis := Vector3(c.dot(t.basis.x), c.dot(t.basis.y), c.dot(t.basis.z))
		out.append_array([row_basis.x, row_basis.y, row_basis.z, c.dot(o)])
	out.append_array([0.0, 0.0, 0.0, 1.0])
	return out
