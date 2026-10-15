## A Presentation Pack's far field, read and refused by the same rules as `mineworld_packages`
## (`docs/MODULE_SPEC.md` §6.1.1; `docs/DECISIONS.md` ARC-102).
##
## Both reference clients take this module as `res://mineworld_environment`. It reads one
## `far_field/<world-key>.yaml` (YAML's JSON subset, DEP-16), validates it, and either holds it or
## names its refusal by code. It never writes a file, opens no connection and builds no request: the
## far field is decoration beyond the boundary, never a world rule.
##
## The checks run in the order the Rust validator keeps, so a file gets the same one code from both
## (RL-e0 AC-FF-3, `tools/cli/tests/client_far_field.rs`):
##
##   MALFORMED (or SLOT, while reading the shape) → SCOPE_MISMATCH → SECTION_MISSING
##   → SECTION_FOREIGN → UNCITED → OUT_OF_RANGE → INSIDE_BOUNDARY → PROFILE → GAP
class_name MineWorldFarField
extends RefCounted

## One list, in the order of `FarFieldCode` in `packages/src/far_field.rs`.
const CODES := [
	"FAR_FIELD_MISSING",
	"FAR_FIELD_WORLD_UNKNOWN",
	"FAR_FIELD_MALFORMED",
	"FAR_FIELD_SCOPE_MISMATCH",
	"FAR_FIELD_SECTION_MISSING",
	"FAR_FIELD_SECTION_FOREIGN",
	"FAR_FIELD_GAP",
	"FAR_FIELD_INSIDE_BOUNDARY",
	"FAR_FIELD_PROFILE",
	"FAR_FIELD_OUT_OF_RANGE",
	"FAR_FIELD_UNCITED",
	"FAR_FIELD_SLOT",
]
const DIR := "far_field"
const REGION_PREFIX := "region-"
const STEP_TENTHS := 3600

const THREE_D_SECTIONS := ["datum_m", "datum_source", "sky", "foreground", "terrain_silhouettes", "skyline"]
const TWO_D_SECTIONS := ["surround"]

## "" when the far field was read and holds; otherwise one of CODES.
var error_code := ""
## What was wrong, naming the field.
var error_detail := ""
## The file read (or looked for).
var path := ""

var _data: Dictionary = {}


## Reads `<pack_dir>/far_field/<world_key>.yaml` for a pack presenting `dimension` ("2d" or "3d").
static func load_for(pack_dir: String, world_key: String, dimension: String) -> MineWorldFarField:
	var field := MineWorldFarField.new()
	if world_key == "":
		field._refuse("FAR_FIELD_WORLD_UNKNOWN", "no world key: start the client with --world-key=<key>")
		return field
	field.path = pack_dir.path_join(DIR).path_join("%s.yaml" % world_key)
	if not FileAccess.file_exists(field.path):
		field._refuse("FAR_FIELD_MISSING",
			"the Presentation Pack %s has no far field for the world %s (%s/%s.yaml)"
			% [pack_dir, world_key, DIR, world_key])
		return field
	return load_file(field.path, PackedStringArray([dimension]))


## Reads one far-field file for a pack presenting `dimensions`; the scope must equal the file's stem.
static func load_file(file: String, dimensions: PackedStringArray) -> MineWorldFarField:
	var field := MineWorldFarField.new()
	field.path = file
	var text := FileAccess.get_file_as_string(file)
	if text == "" and FileAccess.get_open_error() != OK:
		field._refuse("FAR_FIELD_MISSING", "%s could not be read" % file)
		return field
	field._validate(text, file.get_file().get_basename(), dimensions)
	return field


## The line a client prints and shows on a refusal.
func refusal_line(client: String) -> String:
	return "[mineworld-%s] %s: %s: %s" % [client, error_code, path, error_detail]


func ok() -> bool:
	return error_code == ""


func boundary() -> Dictionary:
	return _data.get("boundary", {}).duplicate(true)


func water() -> Array:
	return _data.get("water", []).duplicate(true)


func sky() -> Dictionary:
	return _section("sky")


func foreground() -> Dictionary:
	return _section("foreground")


func silhouettes() -> Array:
	var bands = _data.get("terrain_silhouettes")
	return [] if bands == null else (bands as Array).duplicate(true)


func skyline() -> Dictionary:
	return _section("skyline")


func surround() -> Dictionary:
	return _section("surround")


func datum_m() -> float:
	var datum = _data.get("datum_m")
	return 0.0 if datum == null else float(datum)


func _section(name: String) -> Dictionary:
	var value = _data.get(name)
	return {} if value == null else (value as Dictionary).duplicate(true)


func _refuse(code: String, detail: String) -> void:
	if error_code == "":
		error_code = code
		error_detail = detail


# --- the rules ------------------------------------------------------------------------------------

func _validate(text: String, stem: String, dimensions: PackedStringArray) -> void:
	var json := JSON.new()
	if json.parse(text) != OK:
		_refuse("FAR_FIELD_MALFORMED", "line %d: %s" % [json.get_error_line(), json.get_error_message()])
		return
	var shape := _Shape.new()
	shape.root(json.data)
	if shape.code != "":
		_refuse(shape.code, shape.detail)
		return
	_data = json.data
	var checks := [
		_check_scope.bind(stem),
		_check_sections.bind(dimensions),
		_check_citations,
		_check_ranges,
		_check_boundary,
		_check_profiles,
		_check_coverage,
	]
	for check in checks:
		check.call()
		if error_code != "":
			_data = {}
			return


func _check_scope(stem: String) -> void:
	var scope: Dictionary = _data["scope"]
	var expected := ""
	if scope.has("world"):
		expected = scope["world"]
	else:
		expected = REGION_PREFIX + String(scope["region"])
	if expected != stem:
		_refuse("FAR_FIELD_SCOPE_MISMATCH", "scope %s belongs in %s.yaml, not %s.yaml"
			% [JSON.stringify(scope), expected, stem])


func _present(name: String) -> bool:
	return _data.get(name) != null


func _check_sections(dimensions: PackedStringArray) -> void:
	var required := {
		"3d": THREE_D_SECTIONS,
		"2d": TWO_D_SECTIONS,
	}
	for dimension in ["3d", "2d"]:
		if not dimensions.has(dimension):
			continue
		for name in required[dimension]:
			var present := _present(name)
			if name == "terrain_silhouettes" and present:
				present = not (_data[name] as Array).is_empty()
			if name == "skyline" and present:
				present = not (_data[name]["sectors"] as Array).is_empty()
			if not present:
				_refuse("FAR_FIELD_SECTION_MISSING", "%s is required of a %s pack" % [name, dimension])
				return
	for dimension in ["3d", "2d"]:
		if dimensions.has(dimension):
			continue
		for name in required[dimension]:
			if _present(name):
				_refuse("FAR_FIELD_SECTION_FOREIGN", "%s belongs to a %s pack" % [name, dimension])
				return


func _check_citations() -> void:
	var sources := []
	if _present("datum_source"):
		sources.append(["datum_source", _data["datum_source"]])
	if _present("sky"):
		sources.append(["sky.visibility_source", _data["sky"]["visibility_source"]])
	for band in _bands():
		sources.append(["silhouette %s" % band["name"], band["source"]])
	for body in _data["water"]:
		sources.append(["water %s" % body["name"], body["source"]])
	if _present("skyline"):
		var index := 0
		for sector in _data["skyline"]["sectors"]:
			sources.append(["skyline sector %d" % index, sector["source"]])
			index += 1
		for name in ["station", "landmark"]:
			var slot: Dictionary = _data["skyline"]["slots"][name]
			if slot["state"] == "reserved":
				sources.append(["slot %s" % name, slot["source"]])
	for entry in sources:
		if String(entry[1]).strip_edges() == "":
			_refuse("FAR_FIELD_UNCITED", "%s: the source is empty" % entry[0])
			return


func _bands() -> Array:
	var bands = _data.get("terrain_silhouettes")
	return [] if bands == null else bands


func _check_ranges() -> void:
	var r := _Ranges.new()
	r.above("boundary.reach_m", _data["boundary"]["reach_m"], 0.0)
	for body in _data["water"]:
		var name: String = body["name"]
		r.sector("water %s" % name, body["sector"])
		r.within("water %s level_m" % name, body["level_m"], -430.0, 5000.0)
		if body.get("width_m") != null:
			r.within("water %s width_m" % name, body["width_m"], 1.0, 5000.0)
		if body.get("course") != null:
			for point in body["course"]:
				r.bearing("water %s course" % name, point["bearing_deg"])
	if _present("datum_m"):
		r.within("datum_m", _data["datum_m"], -430.0, 5000.0)
	if _present("sky"):
		r.within("sky.visibility_m", _data["sky"]["visibility_m"], 1000.0, 300000.0)
	if _present("foreground"):
		r.within("foreground.drop_m", _data["foreground"]["drop_m"], 0.0, 500.0)
		r.within("foreground.run_m", _data["foreground"]["run_m"], 10.0, 2000.0)
	for band in _bands():
		var name := "silhouette %s" % band["name"]
		r.sector(name, band["sector"])
		for point in band["profile"]:
			r.bearing(name, point["bearing_deg"])
			r.within("%s height_m" % name, point["height_m"], -430.0, 9000.0)
	if _present("skyline"):
		var index := 0
		for sector in _data["skyline"]["sectors"]:
			var name := "skyline sector %d" % index
			r.sector(name, sector["sector"])
			if float(sector["to_m"]) <= float(sector["from_m"]):
				r.fail("%s: to_m %s is not beyond from_m %s" % [name, sector["to_m"], sector["from_m"]])
			var h: Dictionary = sector["height_m"]
			if not (h["min"] > 0.0 and h["min"] <= h["max"] and h["max"] <= 400.0):
				r.fail("%s: height_m %s–%s is not 0 < min ≤ max ≤ 400" % [name, h["min"], h["max"]])
			if not (sector["density"] > 0.0 and sector["density"] <= 1.0):
				r.fail("%s: density %s is not in (0, 1]" % [name, sector["density"]])
			index += 1
		for name in ["station", "landmark"]:
			var slot: Dictionary = _data["skyline"]["slots"][name]
			if slot["state"] == "reserved":
				r.bearing("slot %s" % name, slot["bearing_deg"])
				r.above("slot %s footprint_m" % name, slot["footprint_m"], 0.0)
	if _present("surround"):
		var haze: Dictionary = _data["surround"]["haze"]
		if not (haze["from_m"] >= 0.0 and haze["to_m"] > haze["from_m"]):
			r.fail("surround.haze: from_m %s and to_m %s are not 0 ≤ from_m < to_m"
				% [haze["from_m"], haze["to_m"]])
	if r.first != "":
		_refuse("FAR_FIELD_OUT_OF_RANGE", r.first)


func _check_boundary() -> void:
	var edge_m: float = _data["boundary"]["reach_m"]
	var distances := []
	for band in _bands():
		for point in band["profile"]:
			distances.append(["silhouette %s" % band["name"], point["distance_m"]])
	for body in _data["water"]:
		distances.append(["water %s shore_m" % body["name"], body["shore_m"]])
	if _present("skyline"):
		var index := 0
		for sector in _data["skyline"]["sectors"]:
			distances.append(["skyline sector %d from_m" % index, sector["from_m"]])
			index += 1
		for name in ["station", "landmark"]:
			var slot: Dictionary = _data["skyline"]["slots"][name]
			if slot["state"] == "reserved":
				distances.append(["slot %s distance_m" % name, slot["distance_m"]])
	for entry in distances:
		if float(entry[1]) <= edge_m:
			_refuse("FAR_FIELD_INSIDE_BOUNDARY", "%s %s is not beyond boundary.reach_m %s"
				% [entry[0], entry[1], edge_m])
			return


func _check_profiles() -> void:
	for band in _bands():
		var name := "silhouette %s" % band["name"]
		var sector: Dictionary = band["sector"]
		var span := clockwise(sector["from_deg"], sector["to_deg"])
		var profile: Array = band["profile"]
		if profile.size() < 2:
			_refuse("FAR_FIELD_PROFILE", "%s: a profile has at least two points" % name)
			return
		var offsets := []
		for point in profile:
			offsets.append(clockwise(sector["from_deg"], point["bearing_deg"]))
		for offset in offsets:
			if offset > span:
				_refuse("FAR_FIELD_PROFILE", "%s: a profile point lies outside the sector" % name)
				return
		for i in range(1, offsets.size()):
			if offsets[i] <= offsets[i - 1]:
				_refuse("FAR_FIELD_PROFILE", "%s: bearings do not increase strictly along the sector" % name)
				return
		if offsets[0] != 0.0 or offsets[offsets.size() - 1] != span:
			_refuse("FAR_FIELD_PROFILE", "%s: the profile does not start at from_deg and end at to_deg" % name)
			return


func _check_coverage() -> void:
	if _present("surround"):
		var extent: float = _data["surround"]["extent_m"]
		if is_nan(extent) or extent <= 0.0:
			_refuse("FAR_FIELD_GAP", "surround.extent_m %s is not positive" % _number(extent))
			return
	if not _present("terrain_silhouettes") and not _present("skyline"):
		return
	var sectors := []
	for band in _bands():
		sectors.append(band["sector"])
	if _present("skyline"):
		for sector in _data["skyline"]["sectors"]:
			sectors.append(sector["sector"])
	for body in _data["water"]:
		if body["horizon"] == "open":
			sectors.append(body["sector"])
	var gap := uncovered(sectors)
	if not gap.is_empty():
		_refuse("FAR_FIELD_GAP", "the far field leaves [%.1f, %.1f) open" % [gap[0], gap[1]])


## The clockwise angle from `a` to `b`, in [0, 360).
static func clockwise(a: float, b: float) -> float:
	return fposmod(b - a, 360.0)


static func sector_contains(sector: Dictionary, bearing: float) -> bool:
	return clockwise(sector["from_deg"], bearing) < clockwise(sector["from_deg"], sector["to_deg"])


## The first maximal run of bearings no sector covers, at every 0.1°, as [from, to]; [] when closed.
static func uncovered(sectors: Array) -> Array:
	var open := func(tenth: int) -> bool:
		var bearing := float(tenth % STEP_TENTHS) / 10.0
		for sector in sectors:
			if sector_contains(sector, bearing):
				return false
		return true
	var first := -1
	for tenth in STEP_TENTHS:
		if open.call(tenth):
			first = tenth
			break
	if first < 0:
		return []
	var end := first
	while end < STEP_TENTHS and open.call(end):
		end += 1
	if first == 0 and end == STEP_TENTHS:
		return [0.0, 360.0]
	var start := first
	if first == 0:
		var behind := 0
		while open.call(STEP_TENTHS - 1 - behind):
			behind += 1
		start = (STEP_TENTHS - behind) % STEP_TENTHS
	return [float(start) / 10.0, float(end) / 10.0]


## A number printed as Rust prints an f64: "400", "0", "0.5".
static func _number(value: float) -> String:
	return str(int(value)) if value == floor(value) else str(value)


## The first out-of-range number found.
class _Ranges:
	var first := ""

	func fail(detail: String) -> void:
		if first == "":
			first = detail

	func within(what: String, value: float, low: float, high: float) -> void:
		if not (value >= low and value <= high):
			fail("%s %s is not in [%s, %s]" % [what, MineWorldFarField._number(value),
				MineWorldFarField._number(low), MineWorldFarField._number(high)])

	func above(what: String, value: float, low: float) -> void:
		if is_nan(value) or value <= low:
			fail("%s %s is not > %s" % [what, MineWorldFarField._number(value), MineWorldFarField._number(low)])

	func bearing(what: String, value: float) -> void:
		if not (value >= 0.0 and value < 360.0):
			fail("%s: bearing %s is not in [0, 360)" % [what, MineWorldFarField._number(value)])

	func sector(what: String, s: Dictionary) -> void:
		bearing(what, s["from_deg"])
		bearing(what, s["to_deg"])
		if s["from_deg"] == s["to_deg"]:
			fail("%s: a sector from %s to %s is empty; use two sectors for a full ring"
				% [what, s["from_deg"], s["to_deg"]])


## The shape check: every type, key, enum and pattern of format 1, before any rule. Unknown keys
## are refused everywhere, as serde's `deny_unknown_fields` does.
class _Shape:
	var code := ""
	var detail := ""

	func bad(what: String) -> bool:
		if code == "":
			code = "FAR_FIELD_MALFORMED"
			detail = what
		return false

	func slot_fault(what: String) -> bool:
		if code == "":
			code = "FAR_FIELD_SLOT"
			detail = what
		return false

	## `fields` maps a key to [kind, required]; kinds are checked by `value`.
	func object(value, where: String, fields: Dictionary) -> bool:
		if typeof(value) != TYPE_DICTIONARY:
			return bad("%s: expected an object" % where)
		for key in value:
			if not fields.has(key):
				return bad("%s: unknown field `%s`" % [where, key])
		for key in fields:
			var spec: Array = fields[key]
			var present: bool = value.has(key) and value[key] != null
			if not present:
				if spec[1]:
					return bad("%s: missing field `%s`" % [where, key])
				continue
			if not field(value[key], "%s.%s" % [where, key], spec[0]):
				return false
		return true

	func field(value, where: String, kind) -> bool:
		if kind is Callable:
			return kind.call(value, where)
		if kind is Array:
			if typeof(value) != TYPE_STRING or not (kind as Array).has(value):
				return bad("%s: unknown variant `%s`, expected one of %s" % [where, value, kind])
			return true
		match kind:
			"number":
				if typeof(value) != TYPE_FLOAT and typeof(value) != TYPE_INT:
					return bad("%s: expected a number" % where)
			"string":
				if typeof(value) != TYPE_STRING:
					return bad("%s: expected a string" % where)
			"key":
				return pattern(value, where, "abcdefghijklmnopqrstuvwxyz0123456789-", 48, "a key (1-48 of a-z, 0-9, -)")
			"asset":
				return pattern(value, where, "abcdefghijklmnopqrstuvwxyz0123456789-_", 64,
					"an asset name (1-64 of a-z, 0-9, _, -)")
			"colour":
				if typeof(value) != TYPE_STRING or value.length() != 7 or not value.begins_with("#") \
						or not value.substr(1).is_valid_hex_number(false):
					return bad("%s: '%s' is not a colour (#rrggbb)" % [where, value])
			"u32":
				if (typeof(value) != TYPE_FLOAT and typeof(value) != TYPE_INT) or float(value) != floor(value) \
						or value < 0 or value > 4294967295:
					return bad("%s: expected a u32" % where)
		return true

	func pattern(value, where: String, allowed: String, longest: int, what: String) -> bool:
		if typeof(value) != TYPE_STRING or value.length() < 1 or value.length() > longest:
			return bad("%s: '%s' is not %s" % [where, value, what])
		for c in value:
			if not allowed.contains(c):
				return bad("%s: '%s' is not %s" % [where, value, what])
		return true

	func list(value, where: String, item: Callable) -> bool:
		if typeof(value) != TYPE_ARRAY:
			return bad("%s: expected a list" % where)
		var index := 0
		for element in value:
			if not item.call(element, "%s[%d]" % [where, index]):
				return false
			index += 1
		return true

	func root(value) -> void:
		var ok := object(value, "far field", {
			"format": ["u32", true],
			"scope": [scope, true],
			"boundary": [boundary, true],
			"water": [func(v, w): return list(v, w, water), true],
			"datum_m": ["number", false],
			"datum_source": ["string", false],
			"sky": [sky, false],
			"foreground": [foreground, false],
			"terrain_silhouettes": [func(v, w): return list(v, w, silhouette), false],
			"skyline": [skyline, false],
			"surround": [surround, false],
		})
		if not ok:
			return
		if value["format"] != 1:
			bad("format %s is not 1, the only format" % MineWorldFarField._number(value["format"]))
			return
		var s = value.get("sky")
		if s != null:
			var panorama: bool = s.get("panorama") != null
			var procedural: bool = s.get("procedural") != null
			if not ((s["kind"] == "panorama" and panorama and not procedural)
					or (s["kind"] == "procedural" and procedural and not panorama)):
				bad("sky: a panorama sky states `panorama` only, a procedural sky `procedural` only")
				return
		for body in value["water"]:
			var river: bool = body["kind"] == "river"
			var width: bool = body.get("width_m") != null
			var course: bool = body.get("course") != null
			if (river and not (width and course)) or (not river and (width or course)):
				bad("water %s: width_m and course are stated for a river, and only for a river" % body["name"])
				return
			if course and (body["course"] as Array).size() < 2:
				bad("water %s: a course has at least two points" % body["name"])
				return

	func scope(value, where: String) -> bool:
		if typeof(value) != TYPE_DICTIONARY or value.size() != 1:
			return bad("%s: a scope is {\"world\": key} or {\"region\": key}" % where)
		var kind: String = value.keys()[0]
		if kind != "world" and kind != "region":
			return bad("%s: unknown variant `%s`" % [where, kind])
		return field(value[kind], "%s.%s" % [where, kind], "key")

	func sector(value, where: String) -> bool:
		return object(value, where, {"from_deg": ["number", true], "to_deg": ["number", true]})

	func boundary(value, where: String) -> bool:
		return object(value, where, {
			"kind": [["invisible_wall", "natural"], true],
			"reach_m": ["number", true],
			"beyond": [["drop", "water", "scrub", "planting", "rough_ground"], true],
		})

	func water(value, where: String) -> bool:
		return object(value, where, {
			"name": ["key", true],
			"kind": [["sea", "bay", "lake", "river"], true],
			"sector": [sector, true],
			"shore_m": ["number", true],
			"level_m": ["number", true],
			"horizon": [["open", "closed"], true],
			"colour": [func(v, w): return object(v, w, {"deep": ["colour", true], "shallow": ["colour", true]}), true],
			"source": ["string", true],
			"width_m": ["number", false],
			"course": [func(v, w): return list(v, w, func(p, q): return object(p, q,
				{"bearing_deg": ["number", true], "distance_m": ["number", true]})), false],
		})

	func sky(value, where: String) -> bool:
		return object(value, where, {
			"kind": [["panorama", "procedural"], true],
			"panorama": ["asset", false],
			"procedural": [func(v, w): return object(v, w, {"top": ["colour", true], "horizon": ["colour", true]}), false],
			"visibility_m": ["number", true],
			"visibility_source": ["string", true],
		})

	func foreground(value, where: String) -> bool:
		return object(value, where, {
			"surface": [["scrub", "dry_grass", "grass", "rough_ground"], true],
			"drop_m": ["number", true],
			"run_m": ["number", true],
		})

	func silhouette(value, where: String) -> bool:
		return object(value, where, {
			"name": ["key", true],
			"layer": [["near", "mid", "far"], true],
			"sector": [sector, true],
			"albedo": ["colour", true],
			"profile": [func(v, w): return list(v, w, func(p, q): return object(p, q, {
				"bearing_deg": ["number", true], "height_m": ["number", true], "distance_m": ["number", true],
			})), true],
			"source": ["string", true],
		})

	func skyline(value, where: String) -> bool:
		return object(value, where, {
			"sectors": [func(v, w): return list(v, w, massing), true],
			"seed": ["u32", true],
			"slots": [slots, true],
		})

	func massing(value, where: String) -> bool:
		return object(value, where, {
			"sector": [sector, true],
			"from_m": ["number", true],
			"to_m": ["number", true],
			"height_m": [func(v, w): return object(v, w, {"min": ["number", true], "max": ["number", true]}), true],
			"density": ["number", true],
			"source": ["string", true],
		})

	func surround(value, where: String) -> bool:
		return object(value, where, {
			"extent_m": ["number", true],
			"base": [["land", "water"], true],
			"haze": [func(v, w): return object(v, w, {
				"colour": ["colour", true], "from_m": ["number", true], "to_m": ["number", true],
			}), true],
		})

	func slots(value, where: String) -> bool:
		if typeof(value) != TYPE_DICTIONARY:
			return bad("%s: expected an object" % where)
		for key in value:
			if key != "station" and key != "landmark":
				return slot_fault("'%s' is not a slot (station, landmark)" % key)
		for key in ["station", "landmark"]:
			if not value.has(key):
				return slot_fault("%s is missing" % key)
			if not slot(value[key], "%s.%s" % [where, key]):
				return false
		return true

	func slot(value, where: String) -> bool:
		var ok := object(value, where, {
			"state": ["string", true],
			"bearing_deg": ["number", false],
			"distance_m": ["number", false],
			"footprint_m": ["number", false],
			"source": ["string", false],
		})
		if not ok:
			return false
		var stated := 0
		for key in ["bearing_deg", "distance_m", "footprint_m", "source"]:
			if value.get(key) != null:
				stated += 1
		match value["state"]:
			"unplaced":
				if stated != 0:
					return slot_fault("an unplaced slot states no position")
			"reserved":
				if stated != 4:
					return slot_fault("a reserved slot states bearing_deg, distance_m, footprint_m and source")
			_:
				return slot_fault("'%s' is not a slot state (unplaced, reserved)" % value["state"])
		return true
