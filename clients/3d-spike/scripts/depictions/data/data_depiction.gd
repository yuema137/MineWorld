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
## How far below the terrain's lowest point the far field's flat near ground stands (schema default).
const NEAR_GROUND_BELOW := 0.5
## The station the file's `station_kit` sites, once built: { kit: StationKit, site, eaves }, or {}.
var _station := {}


## Draws the file: the far field, the ground, the registration of every place, then each section the
## file has, in order. A section the file lacks draws nothing; places stay the stub's boxes until the
## file has `buildings` or `models`.
func build(root: Node3D, far_field: MineWorldFarField, connected: bool) -> Node3D:
	var g := Node3D.new()
	g.name = "Depiction"
	root.add_child(g)
	var d := _data()
	var view: SliceFarFieldView = null
	if d.get("terrain") != null:
		terrain = DepictionTerrain.build(g, d["terrain"], d.get("water"), _section("paths"), frame_origin(),
			_section("decks"), DepictionStations.cuts(_section("stations"), self))
		# the far field's flat ground inside the boundary goes under the file's terrain (D-S33d-12)
		view = draw_far_field(g, far_field, SliceFarFieldView.GROUND_Y - (terrain.lowest() - NEAR_GROUND_BELOW))
	else:
		view = draw_far_field(g, far_field)
		if _section("models").is_empty():
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
	if _has_shells():
		DepictionInteriors.build(g, _section("buildings"), self)
	if not _section("edges").is_empty():
		DepictionEdges.build(g, _section("edges"), self)
	if not _section("scatter").is_empty():
		DepictionScatter.build(g, _section("scatter"), self)
	if not _section("decks").is_empty():
		DepictionDecks.build(g, _section("decks"), self)
	if not _section("stations").is_empty():
		DepictionStations.build(g, _section("stations"), self)
	if d.get("trees") != null:
		DepictionWoods.build(g, d["trees"], self)
	_build_station_kit(g, view)
	if not connected:
		_draw_recorded_furnishings(g)
		_draw_recorded_station(g)
	if models != null:
		# the frame-budget pass the art was drawn under before it was a pack asset (RL-b SD-RLb-3)
		var b := SliceBudget.apply(g)
		print("budget %d instances ranged by screen size; widest at its range end %.2f px; %d over"
			% [b["ranged"], b["max_px"], b["violations"]])
	_root = g
	return g


## The station the file's `station_kit` section sites (S29-d, `DECISIONS.md` ARC-192; moved from Social
## Café's depiction script into its file in 16f-d2): the pack's station kit `kit`, its arch's façade on
## the plan point `facade_mm`, facing `facing`, its floors `floor_m` above the frame's origin. The depot
## and its interior are built from the kit; the trackbed is cut from the far field's near ground. A kit
## the reader refuses refuses the run by its own code (`refusal`). Nothing here names a world.
func _build_station_kit(g: Node3D, view: SliceFarFieldView) -> void:
	var s: Variant = _data().get("station_kit")
	if s == null:
		return
	var at := to_scene(_point(s["facade_mm"]))
	var site := {"kit": String(s["kit"]), "facade_x": at.x, "axis_z": at.z, "facing": String(s["facing"]),
		"floor_y": frame_origin().y + float(s["floor_m"])}
	var opened := StationKit.open(site, ProjectSettings.globalize_path("res://").path_join(SliceMain.PACK_DIR))
	if opened.has("refusal"):
		refusal = opened["refusal"]
		return
	var eaves := []
	for e in s.get("eaves", []):
		eaves.append(to_scene(_point(e["corner_mm"])) + Vector3(0.0, float(e["eaves_m"]), 0.0))
	_station = {"kit": opened["kit"], "site": opened["site"], "eaves": eaves}
	var depot := StationDepot.build(g, opened["kit"], opened["site"])
	if depot == null:
		return
	StationDepotInterior.build(depot, opened["kit"])
	# the trackbed lies below the near ground (S29-e §5.5): the ground is cut where it is drawn
	if view != null:
		view.cut_near_ground(StationDepotInterior.report.get("trench", []))
	# the depot is built from thousands of primitives, as the street was: batched as the slice batched it
	var m := SliceBatch.merge(depot, SliceTrial.cell_size())
	print("station kit %s: %d primitive instances -> %d meshes" % [site["kit"], m["merged_from"], m["merged_into"]])


## Offline, the station's benches and kiosk come from the recorded observations of the recorded world's
## station places (S29-e), each drawn into the kit place its tags bind: the kit place whose tags the
## recorded place carries, the most specific first -- the layout's rule (`SliceLayout._candidate`).
func _draw_recorded_station(g: Node3D) -> void:
	if _station.is_empty():
		return
	var kit: StationKit = _station["kit"]
	var furnishings := SliceFurnishings.load_pack(SlicePack.select().dir)
	for path in SliceFurnishings.recorded_files(key, "station-"):
		var tags := SliceFurnishings.recorded_tags(path)
		var best := ""
		for p in kit.place_names():
			var fits: bool = StationDepot.report["origins"].has(p)
			for t in kit.tags(p):
				fits = fits and tags.has(t)
			if fits and (best == "" or kit.tags(p).size() > kit.tags(best).size()):
				best = p
		if best != "":
			furnishings.draw_recorded_file(path, g, StationDepot.report["origins"][best])


func station() -> Dictionary:
	return _station


## The place entries, and one per kit place a station kit's door leads to (`StationViews.entries`).
func depictions() -> Array[Dictionary]:
	var out: Array[Dictionary] = super.depictions().duplicate()
	out.append_array(StationViews.entries(_station))
	return out


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


## The height (scene y) a body stands at over scene point (x, z): a deck's surface where a deck covers it
## (ARC-178), else the drawn terrain, else the street's surface.
func ground_height(x: float, z: float) -> float:
	var o := frame_origin()
	var mm := Vector2((x - o.x) * 1000.0, (o.z - z) * 1000.0)
	for deck in _section("decks"):
		if DepictionDecks.rect_of(deck).has_point(mm):
			return o.y + DepictionDecks.surface_at(deck, mm)
	var platform: Variant = DepictionStations.surface_at(_section("stations"), self, mm)
	if platform != null:
		return o.y + float(platform)
	return terrain.height_at(x, z) if terrain != null else o.y


## The index in `places()` of the one place `tags` name (ARC-179): the place whose tags are exactly
## these, else the only place carrying them all; -1 when none does, or several do and none exactly.
func place_index(tags: Array) -> int:
	var carrying := []
	# the file's places as written, not `places()`, whose floor heights are read through this
	var all: Array = _data()["places"]
	for i in all.size():
		var have := _tags(all[i]["tags"])
		var fits := true
		for t in tags:
			fits = fits and have.has(String(t))
		if not fits:
			continue
		if have.size() == tags.size():
			return i
		carrying.append(i)
	return carrying[0] if carrying.size() == 1 else -1


## The footprint (plan mm) of the place at `place_index` `i`.
func place_footprint(i: int) -> Rect2:
	return _rect(_data()["places"][i]["footprint_mm"])


## The scene volume of the one place `tags` name (`place_index`), or "".
func volume_of(tags: Array) -> String:
	var i := place_index(tags)
	return place_volume(i) if i >= 0 else ""


## The near water's surface (scene y), or the street's surface with no water.
func water_level() -> float:
	var w: Variant = _data().get("water")
	return frame_origin().y + (float(w["level_m"]) if w != null else 0.0)


## The places' drawing: the file's shells when it has `buildings` (and then nothing of the stub); else
## the stub's boxes, on terrain only their walls, since the ground is the floor. With `models`, only a
## place no model depicts is a box, and the street is the art's (16f-d2).
func draw_boxes(g: Node3D) -> void:
	if _has_shells():
		DepictionShells.build(g, _section("buildings"), self)
		return
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



## The file's `light` section, or {} when it has none (then the slice keeps its own GI and probes).
func light() -> Dictionary:
	return _data()["light"] if _data().get("light") != null else {}


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
			"doorway": _point(p["doorway_mm"]), "floor_m": _floor_of(p["tags"], _rect(p["footprint_mm"]))})
	return out


## A place's floor height in the scene (metres above the street): the surface of the deck that depicts it
## (ARC-178), or of the station platform whose footprint it is (ARC-179), else the street's surface.
func _floor_of(tags: Array, footprint: Rect2) -> float:
	for deck in _section("decks"):
		var all := true
		for t in deck["depicts"]:
			if not (t in tags):
				all = false
		if all:
			return float(deck["surface_m"])
	var platform: Variant = DepictionStations.surface_at(_section("stations"), self, footprint.get_center())
	return float(platform) if platform != null else 0.0


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
		# a step that names a point inside its entry keeps it in the place's frame, millimetres
		route.append(_tags(step) if step is Array
			else {"tags": _tags(step["tags"]), "at_mm": Vector2(step["at_mm"][0], step["at_mm"][1])})
	var out := {
		"farfield": farfield,
		"farfield_shots": _views(v.get("farfield_shots", [])),
		# a perf view places the body (SlicePerf), so its eye_m is lowered to the feet by the eye height
		"perf": _views(v.get("perf", []), -CameraRig.EYE_HEIGHT),
		"perf_breakdown": Array(v.get("perf_breakdown", [])),
		"probe_route": route,
	}
	if not _station.is_empty():
		_station_viewpoints(out)
	return out


## A station kit's views (S29-d §§5.8-5.9), computed from the kit and the built depot (`StationViews`)
## plus what the file's `station_kit` names: its street `views` toward the station (st_1, st_2), the
## `landmark_shots` (names of `farfield_shots`) that are LM-2's footprint rows, and the `walk_via` points
## on the street the walk in goes through.
func _station_viewpoints(out: Dictionary) -> void:
	var s: Dictionary = _data()["station_kit"]
	var street := _views(s.get("views", []))
	var lm2: Array = out["farfield_shots"].filter(func(r: Array) -> bool:
		return Array(s.get("landmark_shots", [])).has(r[0]))
	var via := []
	for p in s.get("walk_via", []):
		via.append(_at(p["at_mm"], float(p["up_m"])))
	out["perf"] = out["perf"] + StationViews.perf(_station)
	out["landmark"] = StationViews.landmark(_station, street, lm2)
	out["station_shots"] = StationViews.shots(_station, street)
	out["station_walk"] = StationViews.walk(_station, via)


## The file's `extras_from` (S33-g §5.2), or "" for the pack's own extras.
func extras_from() -> String:
	return String(_data().get("extras_from", ""))


## The file's `decor` figures (S33-g §5.3) in the scene, each on the drawn ground.
func decor() -> Array:
	var out := []
	for d in _section("decor"):
		if d["pose"] == "walking":
			out.append({"pose": "walking", "from": ground_at(_point(d["from_mm"])),
				"to": ground_at(_point(d["to_mm"]))})
		else:
			out.append({"pose": "standing", "at": ground_at(_point(d["at_mm"])),
				"yaw": float(d.get("yaw_deg", 0.0))})
	return out


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


func _views(views: Array, lift := 0.0) -> Array:
	var out := []
	for view in views:
		out.append([view["name"], _at(view["at_mm"], float(view["eye_m"]) + lift), float(view["yaw_deg"]),
			float(view["pitch_deg"])])
	return out
