## A depiction drawn from a world's depiction file (`DECISIONS.md` ARC-177; S33-d revision 2): the
## world's look is data in its 3D Presentation Pack, `depictions/<key>.yaml`, and this class and the
## generic builders beside it draw whatever the file says. A world drawn from data needs no script: the
## registry (`SliceWorlds.open`, 16f-c) builds this class when the pack has the world's file.
##
## The file is found, in order: `--slice-depiction-file=<path>` (a variant, or another world's file
## drawn on this world's server); `file_path`, when whoever built this depiction set it (the registry);
## else the selected pack's `depictions/<key>.yaml` (`SlicePack`). No world key is needed to construct
## it. A refused file refuses the run by name (`SliceMain._open_depiction`); there is no fallback.
##
## Registration (volumes, doors, entries) is `SliceBoxDepiction.register`, from the file's `street` and
## `places`. Until the builders for terrain, water, edges, shells, woods and light land (S33-d C4-C9),
## the places are drawn as the box depiction's labelled grey boxes, and the HUD says so.
class_name SliceDataDepiction
extends SliceBoxDepiction

const FILE_ARG := "--slice-depiction-file="
## The pack's folder of depiction files (`docs/MODULE_SPEC.md` sec.6.1.2).
const DIR := "depictions"

## The file to read, when set before the first read (otherwise found as above).
var file_path := ""
var _file: SliceDepictionFile = null
## The drawn ground, when the file has a `terrain` section (null: the stub's flat ground at y 0).
var terrain: DepictionTerrain = null
## The placed place art, when the file has a `models` section (null otherwise).
var models: DepictionModels = null


## Draws the file: the far field, the ground, the registration of every place, then each section the
## file has, in order. A section the file lacks draws nothing; places stay the stub's boxes until the
## file has `buildings`.
func build(root: Node3D, far_field: MineWorldFarField, connected: bool) -> Node3D:
	var g := Node3D.new()
	g.name = "Depiction"
	root.add_child(g)
	var d := _data()
	draw_far_field(g, far_field)
	if d.get("terrain") != null:
		terrain = DepictionTerrain.build(g, d["terrain"], d.get("water"), _section("paths"), frame_origin())
	elif _section("models").is_empty():
		# the place art carries its own ground (16f-d2); a file with neither gets the stub's
		draw_ground(g)
	if d.get("water") != null:
		DepictionWater.build(g, d["water"], far_field, frame_origin())
	if not _section("models").is_empty():
		models = DepictionModels.new()
		models.build(g, _section("models"), self)
		print("models %d of %d place assets loaded" % [models.loaded, _section("models").size()])
	register(g)
	if models != null:
		_register_model_doors()
	draw_boxes(g)
	if not _section("edges").is_empty():
		DepictionEdges.build(g, _section("edges"), self)
	if not _section("scatter").is_empty():
		DepictionScatter.build(g, _section("scatter"), self)
	if not connected:
		_draw_recorded_furnishings(g)
	if models != null:
		# the frame-budget pass the art was drawn under before it was a pack asset (RL-b SD-RLb-3)
		var b := SliceBudget.apply(g)
		print("budget %d instances ranged by screen size; widest at its range end %.2f px; %d over"
			% [b["ranged"], b["max_px"], b["violations"]])
	_root = g
	return g


## Offline, the furniture of the place the recorded observation is of (`SliceFurnishings.RECORDED`,
## ARC-307, Q-FD-5), when it is this world's: drawn in the place whose footprint is the recorded floor,
## the place frame's origin on the footprint's south-west corner at the floor's height (its door's
## in-point). Connected, furniture comes from the server's disclosure (`SliceLink`).
func _draw_recorded_furnishings(g: Node3D) -> void:
	var parsed := SliceFurnishings.read_recorded()
	if parsed.is_empty() or String(parsed.get("world", "")) != key:
		return
	var fl: Dictionary = parsed["place-shape"].get("floor", {})
	if fl.is_empty():
		return
	var lo := Vector2(float(fl["min"]["x"]), float(fl["min"]["y"]))
	var size := Vector2(float(fl["max"]["x"]), float(fl["max"]["y"])) - lo
	var i := 0
	for p in places():
		var r: Rect2 = p["footprint"]
		if r.size.is_equal_approx(size):
			var origin := to_scene(r.position - lo)
			var door: int = _entries[i + 1]["door"]
			if door >= 0:
				origin.y = SliceLayout.in_point(SliceDoors.rows[door]).y
			SliceFurnishings.load_pack(SlicePack.select().dir).draw_parsed(g, parsed, origin)
			return
		i += 1


## The height of the drawn ground under a plan point (mm), 0 with no terrain.
func ground_at(mm: Vector2) -> Vector3:
	return terrain.on_ground(mm) if terrain != null else to_scene(mm)


## The stub's boxes, while the file draws no shells: on terrain only their walls, since the ground is
## the floor.
## With `models`, only a place no model depicts is a box, and the street is the art's.
func draw_boxes(g: Node3D) -> void:
	if terrain == null and models == null:
		super.draw_boxes(g)
		return
	var wall := Mats.paint(Color(0.62, 0.62, 0.60), 0.85)
	var floor_mat := Mats.paint(Color(0.48, 0.47, 0.44), 0.9)
	var doorways := []
	for p in places():
		doorways.append(p["doorway"])
	for p in places():
		if models != null and not models.for_place(p["tags"]).is_empty():
			continue
		_walls(g, p["footprint"], doorways, wall)
		if terrain == null:
			_floor(g, p["footprint"], floor_mat)


## The place's door: with a model that depicts the place, the asset's own door (its entrance node, and
## the label, depth and floor height its `asset.yaml` states, D-16c-4); else the stub's.
func _door(g: Node3D, p: Dictionary, volume: String) -> int:
	var m: Dictionary = models.for_place(p["tags"]) if models != null else {}
	if m.is_empty() or (m["doors"] as Array).is_empty():
		return super._door(g, p, volume)
	var row: Dictionary = m["doors"][0]
	var e: Node3D = (m["entrances"] as Dictionary).get(str(row["name"]))
	if e == null:
		return super._door(g, p, volume)
	var yaw := e.global_rotation.y
	var out := Vector3(sin(yaw), 0.0, cos(yaw))
	var sill := e.global_position - out * SliceLayout.DOOR_INSET
	SliceDoors.rows.append([sill, yaw, str(row["label"]), volume, float(row["depth_m"]),
		float(row["floor_m"])])
	return SliceDoors.rows.size() - 1


## Every other door the place art draws (an entrance of a model that no depicted place's door used: a
## shop front, a house door), registered as a closed door, so the layout binds a disclosed doorway that
## lands on it as `door` by coincidence (16c's D-16c-3 rule), as when the client drew the art itself.
func _register_model_doors() -> void:
	for m in models.placed:
		var used := ""
		if not (m["depicts"] as PackedStringArray).is_empty() and not (m["doors"] as Array).is_empty():
			used = str(m["doors"][0]["name"])
		for row in m["doors"]:
			var e: Node3D = (m["entrances"] as Dictionary).get(str(row["name"]))
			if e == null or str(row["name"]) == used:
				continue
			var yaw := e.global_rotation.y
			var out := Vector3(sin(yaw), 0.0, cos(yaw))
			SliceDoors.rows.append([e.global_position - out * SliceLayout.DOOR_INSET, yaw, str(row["label"]), "",
				float(row["depth_m"]), float(row["floor_m"])])


## The doorway (mm) of the place whose tags include every one of `tags`, or null.
func doorway_of(tags: PackedStringArray) -> Variant:
	for p in places():
		var all := not tags.is_empty()
		for t in tags:
			all = all and t in p["tags"]
		if all:
			return p["doorway"]
	return null


## The file's `light` section in the scene: { gi: [{box: AABB, subdiv: int}], probes: [{box: AABB,
## interior: bool}] }. A box stands on the street's surface less the volumes' VOLUME_BELOW; a probe's
## centre is `centre_up_m` above it. A probe whose centre stands inside a place's footprint is that
## room's interior probe.
func light() -> Dictionary:
	var out := {"gi": [], "probes": []}
	var l: Variant = _data().get("light")
	if l == null:
		return out
	var base := frame_origin().y
	for v in l.get("gi", []):
		var b: Array = _box(_rect(v["box_mm"]))
		var size := Vector3((b[1] as Vector3).x, float(v["height_m"]), (b[1] as Vector3).z)
		var at := Vector3((b[0] as Vector3).x, base - VOLUME_BELOW + size.y * 0.5, (b[0] as Vector3).z)
		out["gi"].append({"box": AABB(at - size * 0.5, size), "subdiv": int(v["subdiv"])})
	for v in l.get("probes", []):
		var b: Array = _box(_rect(v["box_mm"]))
		var size := Vector3((b[1] as Vector3).x, float(v["height_m"]), (b[1] as Vector3).z)
		var at := Vector3((b[0] as Vector3).x, base + float(v["centre_up_m"]), (b[0] as Vector3).z)
		var inside := false
		for p in places():
			var r: Rect2 = p["footprint"]
			var c := _plan_of(at)
			inside = inside or r.has_point(c)
		out["probes"].append({"box": AABB(at - size * 0.5, size), "interior": inside})
	return out


## A scene point's plan millimetres in the file's frame.
func _plan_of(p: Vector3) -> Vector2:
	var rel := p - frame_origin()
	return Vector2(rel.x * 1000.0, -rel.z * 1000.0)


## A list section of the file, empty when absent.
func _section(name: String) -> Array:
	return _data()[name] if _data().get(name) != null else []


## The file this depiction draws, read and checked once.
func source() -> SliceDepictionFile:
	if _file == null:
		_file = SliceDepictionFile.load_file(resolved_path())
	return _file


## Where the file is read from (see above).
func resolved_path() -> String:
	for a in OS.get_cmdline_user_args():
		if a.begins_with(FILE_ARG):
			return a.trim_prefix(FILE_ARG)
	if file_path != "":
		return file_path
	return SlicePack.select().path(DIR).path_join("%s.yaml" % key)


## Whether the file was read and passed this client's checks.
func loaded() -> bool:
	return source().ok()


## The refusal, as printed and shown.
func refusal_line() -> String:
	return source().refusal_line(key)


func _data() -> Dictionary:
	return source().data


## Whether the file draws any building shell; until it does, places are the stub's boxes.
func _has_shells() -> bool:
	return not _section("buildings").is_empty()


# --- the data SliceBoxDepiction reads -------------------------------------------------------------

func frame_origin() -> Vector3:
	var o: Array = _data()["frame"]["origin_m"]
	return Vector3(o[0], o[1], o[2])


func street() -> Dictionary:
	var s: Dictionary = _data()[SliceDepictionFile.STREET]
	return {"tags": _tags(s["tags"]), "footprint": _rect(s["footprint_mm"])}


func places() -> Array:
	var out := []
	for p in _data()["places"]:
		out.append({"tags": _tags(p["tags"]), "label": p["label"], "footprint": _rect(p["footprint_mm"]),
			"doorway": _point(p["doorway_mm"])})
	return out


func spawn() -> Dictionary:
	var s: Dictionary = _data()["spawn"]
	# above the drawn ground, which is the street's surface when the file has no terrain
	var on := ground_at(_point(s["at_mm"]))
	return {"position": on + Vector3(0.0, float(s["up_m"]), 0.0), "yaw": float(s["yaw_deg"])}


func viewpoints() -> Dictionary:
	var v: Dictionary = _data()["viewpoints"]
	var farfield := []
	for vp in v["farfield"]:
		farfield.append([vp["id"], _at(vp["at_mm"], vp["eye_m"]), Array(vp.get("roles", []))])
	var route := []
	for step in v.get("probe_route", []):
		route.append(_tags(step))
	return {
		"farfield": farfield,
		"farfield_shots": _views(v.get("farfield_shots", [])),
		"perf": _views(v.get("perf", [])),
		"perf_breakdown": Array(v.get("perf_breakdown", [])),
		"probe_route": route,
	}


func hud_note() -> Dictionary:
	if _has_shells() or models != null:
		return {}
	return {"key": "hud.depiction.stub", "args": {"world": key.capitalize()}}


func door_label(label: String) -> String:
	return label if _has_shells() or models != null else super.door_label(label)


# --- conversions -----------------------------------------------------------------------------------

static func _tags(tags: Array) -> PackedStringArray:
	return PackedStringArray(tags)


static func _rect(r: Array) -> Rect2:
	return Rect2(r[0], r[1], r[2], r[3])


static func _point(p: Array) -> Vector2:
	return Vector2(p[0], p[1])


## A plan point (mm) `up` metres above the street's surface, in the scene.
func _at(p: Array, up: float) -> Vector3:
	return to_scene(_point(p)) + Vector3(0.0, up, 0.0)


func _views(views: Array) -> Array:
	var out := []
	for view in views:
		out.append([view["name"], _at(view["at_mm"], view["eye_m"]), float(view["yaw_deg"]),
			float(view["pitch_deg"])])
	return out
