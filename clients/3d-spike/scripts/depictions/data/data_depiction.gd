## A depiction drawn from a world's depiction file (`DECISIONS.md` ARC-177; S33-d revision 2): the
## world's look is data in its 3D Presentation Pack, `depictions/<key>.yaml`, and this class and the
## generic builders beside it draw whatever the file says. A world drawn from data needs only the one
## line `extends SliceDataDepiction` until 16f lets the registry find the file itself.
##
## The file is found, in order: `--slice-depiction-file=<path>` (a variant, or another world's file
## drawn on this world's server); `file_path`, when whoever built this depiction set it (16f's
## registry); else `<SliceMain.PACK_DIR>/depictions/<key>.yaml`. No world key is needed to construct
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
	return ProjectSettings.globalize_path("res://").path_join(SliceMain.PACK_DIR).simplify_path() \
		.path_join(DIR).path_join("%s.yaml" % key)


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
	return _data().get("buildings") != null and not (_data()["buildings"] as Array).is_empty()


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
	return {"position": _at(s["at_mm"], s["up_m"]), "yaw": float(s["yaw_deg"])}


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
	if _has_shells():
		return {}
	return {"key": "hud.depiction.stub", "args": {"world": key.capitalize()}}


func door_label(label: String) -> String:
	return label if _has_shells() else super.door_label(label)


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
