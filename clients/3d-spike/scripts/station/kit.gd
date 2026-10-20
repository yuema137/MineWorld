## A station kit: the Presentation Pack's description of one station building, read and checked
## (`DECISIONS.md` ARC-192; `PRESENTATION.md` §5).
##
## The file is `presentation/<pack>/3D/stations/<kit>.yaml`, YAML's JSON subset (DEP-16), as the far
## field and the cast binding are. It states the building's dimensions in millimetres, each group with
## its `source`, and the places it depicts by ROLE TAG, never by key. Its frames are the world's place
## frames (x east, y north, `CORE_CONCEPTS.md` §6.1), so a door position in the kit is literally the
## passage point a world states, and the binding (`SliceLayout`) checks the two agree within 0.15 m.
##
## A kit is presentation only: it changes what is drawn, never which requests this client sends, and it
## decides no reachability or availability (`PRESENTATION.md` §4). Only this client reads it (Q-S29d-3).
##
## THE REFUSALS, checked in this order, each printed `[mineworld-slice] <CODE>: <path>: <detail>`:
##   STATION_KIT_MISSING       no file for the kit a depiction names
##   STATION_KIT_MALFORMED     not JSON, a wrong type, an unknown key, a door naming an unknown place,
##                             `format` other than 1
##   STATION_KIT_UNCITED       a place, a door or a number group with no `source`
##   STATION_KIT_OUT_OF_RANGE  a door leaf outside VISUAL_SLICE §3.2; a floor not positive; a doorway
##                             point not 400 mm inside its wall within 1 mm (12d's authoring rule)
##   STATION_SITE_FACING       a site whose arch faces anything but east: a binding only translates, so
##                             a world facing another way writes its frames for it (§5.5)
class_name StationKit
extends RefCounted

const DIR := "stations"
const MISSING := "STATION_KIT_MISSING"
const MALFORMED := "STATION_KIT_MALFORMED"
const UNCITED := "STATION_KIT_UNCITED"
const BEYOND := "STATION_KIT_OUT_OF_RANGE"
const SITE_FACING := "STATION_SITE_FACING"

## The keys a kit may have. `fitout` is S29-e's section, accepted absent or present.
const TOP_KEYS := ["format", "kit", "style", "source", "places", "doors", "shell", "arch", "towers",
	"arcade", "annex", "tile", "fitout"]
## The number groups and the numbers each must state (a scalar, or a pair when listed in PAIRS).
const GROUPS := {
	"shell": ["eaves_mm", "ridge_mm", "plinth_mm", "overhang_mm"],
	"arch": ["clear_mm", "screen_mm", "bays", "gable_mm"],
	"towers": ["plan_mm", "forward_mm", "inner_offset_mm", "shaft_mm", "belfry_mm", "drum_mm",
		"dome_mm", "lantern_mm", "finial_mm"],
	"arcade": ["depth_mm", "pier_mm", "pitch_mm", "length_mm", "spring_mm", "rise_mm", "eaves_mm"],
	"annex": ["eaves_mm"],
}
const PAIRS := ["clear_mm", "plan_mm", "belfry_mm", "drum_mm", "dome_mm", "lantern_mm", "size_mm",
	"leaf_mm", "at", "there", "min", "max"]
## How a kit place is drawn: the arched building, a walled room off one of its walls, or an open place
## under the arcade along one of its walls.
const SHELLS := ["main", "annex", "open"]
const WALLS := ["east", "west", "north", "south"]
## The arch's side of a door, for a door that joins the building to the street.
const OUTSIDE := "outside"
## VISUAL_SLICE.md §3.2: a door leaf's width and height, metres.
const LEAF_W := Vector2(0.85, 1.00)
const LEAF_H := Vector2(2.05, 2.20)
## 12d's authoring rule (step-11 SD-D3): a doorway point stands 400 mm inside its wall, within 1 mm.
const POINT_INSET_MM := 400.0
const POINT_TOLERANCE_MM := 1.0

var path := ""
var error_code := ""
var error_detail := ""
## The parsed file, once it passed every check.
var data: Dictionary = {}


## The kit `kit` of the pack at `pack_dir`.
static func load_kit(pack_dir: String, kit: String) -> StationKit:
	return load_file(pack_dir.path_join(DIR).path_join(kit + ".yaml"))


## A kit file by its path (`--slice-station-kit-file=`: a variant, or a planted refusal).
static func load_file(file: String) -> StationKit:
	var k := StationKit.new()
	k.path = file
	if not FileAccess.file_exists(file):
		k._fail(MISSING, "no station kit file")
		return k
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(file))
	if typeof(parsed) != TYPE_DICTIONARY:
		k._fail(MALFORMED, "not a JSON object")
		return k
	k.data = parsed
	if k._malformed() or k._uncited() or k._beyond_limits():
		k.data = {}
	return k


func ok() -> bool:
	return error_code == ""


## The refusal as printed and shown (RL-e0's line format).
func refusal_line(client := "slice") -> String:
	return "[mineworld-%s] %s: %s: %s" % [client, error_code, path, error_detail]


## A site's refusal line, or "" when the site can be drawn (`STATION_SITE_FACING`).
static func site_refusal(site: Dictionary, client := "slice") -> String:
	var facing := String(site.get("facing", ""))
	if facing == "east":
		return ""
	return "[mineworld-%s] %s: { kit: %s }: the arch faces \"%s\"; only \"east\" is drawn, because a binding only translates (ARC-192)" \
		% [client, SITE_FACING, site.get("kit", ""), facing]


## The station a depiction draws at `site`, as `SliceWorld.build` takes it: { kit, site }, or { refusal }.
## `--slice-station-kit-file=<path>` draws another kit file at the same site (a variant, or a planted
## refusal); `--slice-station-facing=<dir>` plants a site facing elsewhere (AC-d-1).
static func open(site: Dictionary, pack_dir: String) -> Dictionary:
	var s := site.duplicate()
	var file := ""
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--slice-station-kit-file="):
			file = a.trim_prefix("--slice-station-kit-file=")
		elif a.begins_with("--slice-station-facing="):
			s["facing"] = a.trim_prefix("--slice-station-facing=")
	var line := site_refusal(s)
	if line != "":
		return {"refusal": line}
	var kit := load_file(file) if file != "" else load_kit(pack_dir, String(s["kit"]))
	if not kit.ok():
		return {"refusal": kit.refusal_line()}
	if file != "":
		print("station kit from %s" % file)
	return {"kit": kit, "site": s}


# --- getters (metres unless stated) ------------------------------------------------------------------

func kit_name() -> String:
	return String(data.get("kit", ""))


## Every kit place name, in the file's order. The names are the kit's own, never a world's keys.
func place_names() -> PackedStringArray:
	return PackedStringArray((data["places"] as Dictionary).keys())


func place(name: String) -> Dictionary:
	return data["places"][name]


## The one place drawn as the arched building.
func main_place() -> String:
	for p in place_names():
		if place(p)["shell"] == "main":
			return p
	return ""


func tags(name: String) -> PackedStringArray:
	return PackedStringArray(place(name)["tags"])


## A place's floor rectangle in its own frame, metres: position = min, end = max (x east, y north).
func floor_rect(name: String) -> Rect2:
	var f: Dictionary = place(name)["floor"]
	var lo := Vector2(f["min"][0], f["min"][1]) / 1000.0
	var hi := Vector2(f["max"][0], f["max"][1]) / 1000.0
	return Rect2(lo, hi - lo)


func doors() -> Array:
	return data["doors"]


## The door that joins the building to the street.
func arch_door() -> Dictionary:
	for d in doors():
		if d["from"] == OUTSIDE:
			return d
	return {}


func group(name: String) -> Dictionary:
	return data.get(name, {})


## A scalar of a group, metres.
func m(group_name: String, key: String) -> float:
	return float(group(group_name)[key]) / 1000.0


## A pair of a group, metres.
func pair(group_name: String, key: String) -> Vector2:
	var a: Array = group(group_name)[key]
	return Vector2(a[0], a[1]) / 1000.0


func wall_t(wall: String) -> float:
	var w: Dictionary = group("shell")["walls_mm"]
	return float(w.get(wall, w["other"])) / 1000.0


func pattern(surface: String) -> String:
	return String(group("tile").get(surface, ""))


## Every number group of the kit as `KIT` lines (`--slice-station-kit`).
func lines() -> PackedStringArray:
	var out := PackedStringArray()
	for p in place_names():
		out.append("KIT place %s tags %s volume %s shell %s floor %s" % [p, ",".join(tags(p)),
			place(p)["volume"], place(p)["shell"], floor_rect(p)])
	for d in doors():
		out.append("KIT door %s %s -> %s at %s wall %s leaves %d x %s" % [d["name"], d["from"], d["to"],
			d["at"], d["wall"], int(d["leaves"]), d["leaf_mm"]])
	for g in GROUPS:
		var nums := []
		for key in GROUPS[g]:
			nums.append("%s=%s" % [key, group(g)[key]])
		out.append("KIT %s %s; source: %s" % [g, " ".join(nums), group(g)["source"]])
	return out


# --- the checks ----------------------------------------------------------------------------------------

func _fail(code: String, detail: String) -> bool:
	error_code = code
	error_detail = detail
	return true


static func _num(v: Variant) -> bool:
	return typeof(v) == TYPE_FLOAT or typeof(v) == TYPE_INT


static func _num_pair(v: Variant) -> bool:
	return typeof(v) == TYPE_ARRAY and (v as Array).size() == 2 and _num(v[0]) and _num(v[1])


func _malformed() -> bool:
	if int(data.get("format", 0)) != 1:
		return _fail(MALFORMED, "format is not 1")
	for key in data:
		if not (key in TOP_KEYS):
			return _fail(MALFORMED, "unknown key %s" % key)
	if typeof(data.get("kit")) != TYPE_STRING or typeof(data.get("places")) != TYPE_DICTIONARY \
			or typeof(data.get("doors")) != TYPE_ARRAY or typeof(data.get("tile")) != TYPE_DICTIONARY:
		return _fail(MALFORMED, "kit, places, doors and tile are required")
	var mains := 0
	for p in data["places"]:
		var e: Variant = data["places"][p]
		if typeof(e) != TYPE_DICTIONARY or typeof(e.get("tags")) != TYPE_ARRAY or (e["tags"] as Array).is_empty() \
				or typeof(e.get("volume")) != TYPE_STRING or not (e.get("shell") in SHELLS) \
				or typeof(e.get("floor")) != TYPE_DICTIONARY or not _num_pair(e["floor"].get("min")) \
				or not _num_pair(e["floor"].get("max")):
			return _fail(MALFORMED, "place %s: tags, volume, shell (%s) and floor {min, max} are required"
				% [p, "|".join(SHELLS)])
		for t in e["tags"]:
			if typeof(t) != TYPE_STRING:
				return _fail(MALFORMED, "place %s: a tag is not a string" % p)
		if e["shell"] == "main":
			mains += 1
	if mains != 1:
		return _fail(MALFORMED, "%d places are the main building; exactly one must be" % mains)
	var outside := 0
	for d in data["doors"]:
		if typeof(d) != TYPE_DICTIONARY or typeof(d.get("name")) != TYPE_STRING \
				or typeof(d.get("from")) != TYPE_STRING or typeof(d.get("to")) != TYPE_STRING \
				or not _num_pair(d.get("at")) or not (d.get("wall") in WALLS) or not _num(d.get("leaves")) \
				or not _num_pair(d.get("leaf_mm")):
			return _fail(MALFORMED, "a door needs name, from, to, at, wall (%s), leaves and leaf_mm"
				% "|".join(WALLS))
		if not data["places"].has(d["to"]):
			return _fail(MALFORMED, "door %s leads to an unknown place %s" % [d["name"], d["to"]])
		if d["from"] == OUTSIDE:
			outside += 1
		elif not data["places"].has(d["from"]):
			return _fail(MALFORMED, "door %s leads from an unknown place %s" % [d["name"], d["from"]])
		elif data["places"][d["from"]]["shell"] != "main" or not _num_pair(d.get("there")):
			return _fail(MALFORMED, "door %s: an inner door leads from the main building and states there"
				% d["name"])
	if outside != 1:
		return _fail(MALFORMED, "%d doors lead from outside; exactly one must" % outside)
	for g in GROUPS:
		if typeof(data.get(g)) != TYPE_DICTIONARY:
			return _fail(MALFORMED, "group %s is required" % g)
		for key in GROUPS[g]:
			var v: Variant = data[g].get(key)
			if not (_num_pair(v) if key in PAIRS else _num(v)):
				return _fail(MALFORMED, "%s.%s is not a %s" % [g, key, "pair of numbers" if key in PAIRS
					else "number"])
	var shell: Dictionary = data["shell"]
	if typeof(shell.get("walls_mm")) != TYPE_DICTIONARY or not _num(shell["walls_mm"].get("other")) \
			or typeof(shell.get("clerestory")) != TYPE_DICTIONARY \
			or not _num(shell["clerestory"].get("count")) or not _num_pair(shell["clerestory"].get("size_mm")) \
			or not _num(shell["clerestory"].get("sill_mm")):
		return _fail(MALFORMED, "shell.walls_mm.other and shell.clerestory {count, size_mm, sill_mm} are required")
	return false


func _uncited() -> bool:
	for p in data["places"]:
		if String(data["places"][p].get("source", "")).strip_edges() == "":
			return _fail(UNCITED, "place %s has no source" % p)
	for d in data["doors"]:
		if String(d.get("source", "")).strip_edges() == "":
			return _fail(UNCITED, "door %s has no source" % d["name"])
	for g in GROUPS:
		if String(data[g].get("source", "")).strip_edges() == "":
			return _fail(UNCITED, "%s has no source" % g)
	return false


func _beyond_limits() -> bool:
	for p in data["places"]:
		var r := floor_rect(p)
		if r.size.x <= 0.0 or r.size.y <= 0.0:
			return _fail(BEYOND, "place %s: the floor is not positive" % p)
	for g in GROUPS:
		for key in GROUPS[g]:
			var v: Variant = data[g][key]
			for x in (v as Array if typeof(v) == TYPE_ARRAY else [v]):
				if float(x) <= 0.0:
					return _fail(BEYOND, "%s.%s is not positive" % [g, key])
	if int(data["arch"]["bays"]) % 2 != 1:
		return _fail(BEYOND, "arch.bays is %d; it is odd, so one bay stands on the axis" % int(data["arch"]["bays"]))
	for d in data["doors"]:
		var leaf := Vector2(d["leaf_mm"][0], d["leaf_mm"][1]) / 1000.0
		if leaf.x < LEAF_W.x or leaf.x > LEAF_W.y or leaf.y < LEAF_H.x or leaf.y > LEAF_H.y:
			return _fail(BEYOND, "door %s: the leaf %s m is outside %s x %s m (VISUAL_SLICE §3.2)"
				% [d["name"], leaf, LEAF_W, LEAF_H])
		var host := String(d["to"] if d["from"] == OUTSIDE else d["from"])
		var miss := _inset_miss(floor_rect(host), d["wall"], Vector2(d["at"][0], d["at"][1]))
		if miss != "":
			return _fail(BEYOND, "door %s: %s" % [d["name"], miss])
		if d["from"] != OUTSIDE:
			# the far side of the wall is the target's opposite edge
			var opposite := {"east": "west", "west": "east", "north": "south", "south": "north"}
			miss = _inset_miss(floor_rect(d["to"]), opposite[d["wall"]], Vector2(d["there"][0], d["there"][1]))
			if miss != "":
				return _fail(BEYOND, "door %s (there): %s" % [d["name"], miss])
	return false


## "" when `at` (mm) stands POINT_INSET_MM inside `wall` of the floor `r` (metres), else why not.
static func _inset_miss(r: Rect2, wall: String, at: Vector2) -> String:
	var edge: float = {"east": r.end.x, "west": r.position.x, "north": r.end.y, "south": r.position.y}[wall] * 1000.0
	var v := at.x if wall in ["east", "west"] else at.y
	var inward := -1.0 if wall in ["east", "north"] else 1.0
	var off := (v - edge) * inward
	if absf(off - POINT_INSET_MM) > POINT_TOLERANCE_MM:
		return "the point %s is %.0f mm inside its %s wall, not %.0f" % [at, off, wall, POINT_INSET_MM]
	return ""
