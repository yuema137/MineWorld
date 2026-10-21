## A world's depiction file, read and checked by this client (`DECISIONS.md` ARC-177;
## `docs/MODULE_SPEC.md` sec.6.1.2; `PRESENTATION.md` sec.6).
##
## `mineworld packs validate` checks a file in full (`mineworld_packages::depiction`). This reader checks
## what it must before drawing, and refuses by the same names: DEPICTION_MALFORMED (types, required keys,
## unknown keys, the conditional shapes), DEPICTION_SCOPE_MISMATCH, and DEPICTION_REFERENCE for the names
## only this client knows -- texture slugs, tree forms and cards. The geometry codes are the validator's
## alone; a validated pack is trusted for them (S33-d Q-S33d-4 (A)).
##
## Nothing here names a world, a place key or a tag: the format's own section names are the only words.
class_name SliceDepictionFile
extends RefCounted

## The codes, in the validator's order (`DepictionCode::ALL`).
const CODES := ["DEPICTION_MALFORMED", "DEPICTION_SCOPE_MISMATCH", "DEPICTION_UNCITED",
	"DEPICTION_OUT_OF_RANGE", "DEPICTION_POLYGON", "DEPICTION_REFERENCE", "DEPICTION_DOOR",
	"DEPICTION_ON_FLOOR"]
const MALFORMED := "DEPICTION_MALFORMED"
const SCOPE_MISMATCH := "DEPICTION_SCOPE_MISMATCH"
const REFERENCE := "DEPICTION_REFERENCE"

## The section that binds the street every place opens onto: the format's word, not a place.
const STREET := "street"

## The tree builder forms and cards this client's library has (`Props`, `Procgen`). A later builder that
## adds a form or a card adds its name here.
const FORMS := ["conifer", "broadleaf"]
const CARDS := ["needle", "leaf"]

## The schema. A spec is a primitive name, or [kind, ...]:
##   int   an integral number          uint  a non-negative integral number
##   num   a number                    str   a string                bool  a boolean
##   key   1-48 of a-z 0-9 -           name  1-64 of a-z 0-9 _ -     colour  #rrggbb
##   ["list", spec]  ["tuple", n, spec]  ["obj", type]  ["enum", [values]]  ["null_or", spec]
## Each object type: { field: [spec, required] }.
const POINT := ["tuple", 2, "int"]
const RECT := ["tuple", 4, "int"]
const POLYGON := ["list", POINT]
const TYPES := {
	"file": {
		"format": ["uint", true], "scope": [["obj", "scope"], true], "frame": [["obj", "frame"], true],
		STREET: [["obj", "street"], true], "places": [["list", ["obj", "place"]], true],
		"spawn": [["obj", "spawn"], true], "terrain": [["obj", "terrain"], false],
		"water": [["obj", "water"], false], "edges": [["list", ["obj", "edge"]], false],
		"paths": [["list", ["obj", "path"]], false], "buildings": [["list", ["obj", "building"]], false],
		"trees": [["obj", "trees"], false], "scatter": [["list", ["obj", "scatter"]], false],
		"light": [["obj", "light"], false], "models": [["list", ["obj", "model"]], false],
		"viewpoints": [["obj", "viewpoints"], true],
	},
	# 16f-d1: a place asset of the pack (`assets/places/<asset>/`), placed by an anchor (ARC-177 amended).
	"model": {"asset": ["key", true], "depicts": [["null_or", ["list", "key"]], false],
		"anchor": [["obj", "anchor"], true]},
	"anchor": {"entrance": ["name", false], "at_doorway": ["bool", false], "at_mm": [POINT, false],
		"yaw_deg": ["num", false]},
	"scope": {"world": ["key", true]},
	"frame": {"origin_m": [["tuple", 3, "num"], true]},
	"street": {"tags": [["list", "key"], true], "footprint_mm": [RECT, true]},
	"place": {"tags": [["list", "key"], true], "label": ["str", true], "footprint_mm": [RECT, true],
		"doorway_mm": [POINT, true]},
	"spawn": {"at_mm": [POINT, true], "up_m": ["num", true], "yaw_deg": ["num", true]},
	"terrain": {
		"extent_mm": [RECT, true], "cell_m": ["num", true], "blend_m": ["num", true], "seed": ["uint", true],
		"layers": [["list", ["obj", "layer"]], true], "default_layer": ["key", true],
		"regions": [["list", ["obj", "region"]], true], "profiles": [["list", ["obj", "profile"]], false],
		"bed": [["obj", "bed"], false],
	},
	"layer": {"name": ["key", true], "material": ["name", true], "repeat_m": ["num", true],
		"tint": ["colour", false]},
	"region": {"name": ["key", true], "polygon_mm": [POLYGON, true], "height": [["obj", "height"], true],
		"layer": ["key", true], "source": ["str", false]},
	"height": {"flat_m": ["num", false], "crown_m": ["num", false],
		"crown_along": [["enum", ["x", "y"]], false], "ramp": [["obj", "ramp"], false]},
	"ramp": {"from_mm": [["tuple", 2, POINT], true], "from_m": ["num", true],
		"to_mm": [["tuple", 2, POINT], true], "to_m": ["num", true]},
	"profile": {"name": ["key", true], "bearing_deg": ["num", true],
		"samples": [["list", ["obj", "sample"]], true], "source": ["str", false]},
	"sample": {"distance_m": ["num", true], "height_m": ["num", true]},
	"bed": {"depth_m": ["num", true], "run_m": ["num", true], "source": ["str", false]},
	"water": {"level_m": ["num", true], "far_field_water": ["key", true], "extent_m": ["num", true],
		"polygon_mm": [POLYGON, true], "shader": [["obj", "shader"], false],
		"lilies": [["list", ["obj", "lilies"]], false], "source": ["str", false]},
	"shader": {"depth_fade_m": ["num", false], "shore_foam": ["num", false]},
	"lilies": {"polygon_mm": [POLYGON, true], "cover": ["num", true], "seed": ["uint", true]},
	"edge": {"name": ["key", true],
		"kind": [["enum", ["kerb", "quay", "revetment", "rail", "wall", "gate"]], true],
		"line_mm": [POLYGON, true], "height_m": ["num", true], "width_m": ["num", true],
		"material": ["name", true], "tint": ["colour", false], "open_at": [["list", "key"], false],
		"drop_m": ["num", false], "posts_m": ["num", false], "source": ["str", false]},
	"path": {"name": ["key", true], "line_mm": [POLYGON, true], "width_m": ["num", true],
		"layer": ["key", true]},
	"building": {"name": ["key", true], "depicts": [["null_or", ["list", "key"]], false],
		"footprint_mm": [POLYGON, true], "storeys": ["uint", true], "storey_m": ["num", true],
		"plinth_m": ["num", true], "walls": [["obj", "surface"], true], "roof": [["obj", "roof"], true],
		"openings": [["list", ["obj", "opening"]], false], "chimney": [["obj", "chimney"], false],
		"source": ["str", false]},
	"surface": {"material": ["name", true], "tint": ["colour", false]},
	"roof": {"form": [["enum", ["gable", "hip", "mono", "flat"]], true],
		"ridge": [["enum", ["x", "y"]], false], "falls": [["enum", ["n", "s", "e", "w"]], false],
		"pitch_deg": ["num", true], "overhang_m": ["num", false], "material": ["name", true],
		"tint": ["colour", false]},
	"opening": {"side": [["enum", ["n", "s", "e", "w"]], true],
		"kind": [["enum", ["door", "double_door", "window", "shop_window"]], true],
		"storey": ["uint", true], "offset_m": ["num", false], "at_doorway": ["bool", false],
		"width_m": ["num", true], "height_m": ["num", true], "sill_m": ["num", false],
		"state": [["enum", ["closed", "open", "dark"]], true]},
	"chimney": {"at_mm": [POINT, true], "size_m": [["tuple", 2, "num"], true],
		"above_ridge_m": ["num", true]},
	"trees": {"species": [["list", ["obj", "species"]], true],
		"stands": [["list", ["obj", "stand"]], false], "singles": [["list", ["obj", "single"]], false],
		"keep_clear": [["obj", "keep_clear"], true], "lod_m": ["num", true],
		"air_walls": [["list", ["obj", "air_wall"]], false], "source": ["str", false]},
	"species": {"name": ["key", true], "latin": ["str", true], "form": ["name", true],
		"card": ["name", true], "height_m": [["obj", "range"], true], "source": ["str", false]},
	"range": {"min": ["num", true], "max": ["num", true]},
	"stand": {"name": ["key", true], "area": [["obj", "area"], true], "spacing_m": ["num", true],
		"density": [["obj", "density"], true], "mix": [["list", ["obj", "mix"]], true],
		"seed": ["uint", true]},
	"area": {"ring": [["obj", "ring"], false], "polygon_mm": [POLYGON, false]},
	"ring": {"centre_mm": [POINT, true], "inner_m": ["num", true], "outer_m": ["num", true]},
	"density": {"from": ["num", true], "to": ["num", true], "rise_m": [["tuple", 2, "num"], true]},
	"mix": {"species": ["key", true], "weight": ["num", true],
		"prefer": [["enum", ["low", "rise"]], false]},
	"single": {"species": ["key", true], "at_mm": [POINT, true], "height_m": ["num", true]},
	"keep_clear": {"places": ["bool", true], "paths_m": ["num", true]},
	"air_wall": {"ring": [["obj", "air_ring"], false], "line_mm": [POLYGON, false]},
	"air_ring": {"centre_mm": [POINT, true], "radius_m": ["num", true]},
	"scatter": {"name": ["key", true],
		"kind": [["enum", ["boulder", "reeds", "grass", "wildflower"]], true],
		"area": [["obj", "scatter_area"], true], "count": ["uint", true],
		"size_m": [["obj", "range"], true], "seed": ["uint", true], "range_m": ["num", false],
		"source": ["str", false]},
	"scatter_area": {"polygon_mm": [POLYGON, false], "along_mm": [POLYGON, false],
		"width_m": ["num", false]},
	"light": {"gi": [["list", ["obj", "gi"]], false], "probes": [["list", ["obj", "probe"]], false]},
	"gi": {"box_mm": [RECT, true], "height_m": ["num", true], "subdiv": ["uint", true]},
	"probe": {"box_mm": [RECT, true], "height_m": ["num", true], "centre_up_m": ["num", true],
		"update": [["enum", ["once"]], true]},
	"viewpoints": {"farfield": [["list", ["obj", "farfield_viewpoint"]], true],
		"farfield_shots": [["list", ["obj", "view"]], false], "perf": [["list", ["obj", "view"]], false],
		"perf_breakdown": [["list", "str"], false], "probe_route": [["list", ["list", "key"]], false]},
	"farfield_viewpoint": {"id": ["str", true], "at_mm": [POINT, true], "eye_m": ["num", true],
		"roles": [["list", ["enum", ["protected", "judged", "high"]]], false]},
	"view": {"name": ["str", true], "at_mm": [POINT, true], "eye_m": ["num", true],
		"yaw_deg": ["num", true], "pitch_deg": ["num", true]},
}

## The file, parsed and checked: the root object of `TYPES.file`. Empty when refused.
var data: Dictionary = {}
var path := ""
var error_code := ""
var error_detail := ""


func ok() -> bool:
	return error_code == ""


## Reads and checks the file at `file_path`; its `scope.world` must equal the file's stem.
static func load_file(file_path: String) -> SliceDepictionFile:
	var f := SliceDepictionFile.new()
	f.path = file_path
	if not FileAccess.file_exists(file_path):
		return f._refuse(MALFORMED, "cannot read %s" % file_path)
	var text := FileAccess.get_file_as_string(file_path)
	var json := JSON.new()
	if json.parse(text) != OK:
		return f._refuse(MALFORMED, "not YAML's JSON subset at line %d: %s"
			% [json.get_error_line(), json.get_error_message()])
	if not json.data is Dictionary:
		return f._refuse(MALFORMED, "the file is not one object")
	var root: Dictionary = json.data
	var fault := _check(root, ["obj", "file"], "")
	if fault == "":
		fault = _shape(root)
	if fault != "":
		return f._refuse(MALFORMED, fault)
	var stem := file_path.get_file().get_basename()
	var world: String = root["scope"]["world"]
	if world != stem:
		return f._refuse(SCOPE_MISMATCH, "scope.world %s belongs in %s.yaml, not %s.yaml"
			% [world, world, stem])
	fault = _names(root)
	if fault != "":
		return f._refuse(REFERENCE, fault)
	f.data = root
	return f


## The refusal as printed and shown, RL-e0's line format.
func refusal_line(world: String) -> String:
	return "[mineworld-slice] %s: { world: %s }: %s: %s" % [error_code, world, path, error_detail]


func _refuse(code: String, detail: String) -> SliceDepictionFile:
	error_code = code
	error_detail = detail
	data = {}
	return self


# --- the schema ------------------------------------------------------------------------------------

static func _check(value: Variant, spec: Variant, at: String) -> String:
	var where := at if at != "" else "the file"
	if spec is String:
		return _primitive(value, spec, where)
	match spec[0]:
		"list":
			if not value is Array:
				return "%s: not a list" % where
			for i in (value as Array).size():
				var fault := _check(value[i], spec[1], "%s[%d]" % [at, i])
				if fault != "":
					return fault
			return ""
		"tuple":
			if not value is Array or (value as Array).size() != spec[1]:
				return "%s: not a list of %d" % [where, spec[1]]
			for i in (value as Array).size():
				var fault := _check(value[i], spec[2], "%s[%d]" % [at, i])
				if fault != "":
					return fault
			return ""
		"enum":
			if not value is String or not (value in spec[1]):
				return "%s: %s is not one of %s" % [where, str(value), ", ".join(PackedStringArray(spec[1]))]
			return ""
		"null_or":
			return "" if value == null else _check(value, spec[1], at)
		"obj":
			return _object(value, TYPES[spec[1]], at, where)
	return "%s: the schema has no spec %s" % [where, str(spec)]


static func _object(value: Variant, fields: Dictionary, at: String, where: String) -> String:
	if not value is Dictionary:
		return "%s: not an object" % where
	var d: Dictionary = value
	for k in d:
		if not fields.has(k):
			return "%s: unknown field `%s`" % [where, k]
	for k in fields:
		var spec: Array = fields[k]
		var field_at := ("%s.%s" % [at, k]) if at != "" else str(k)
		if not d.has(k):
			if spec[1]:
				return "%s: missing field `%s`" % [where, k]
			continue
		if d[k] == null and not (spec[0] is Array and spec[0][0] == "null_or"):
			if spec[1]:
				return "%s: null" % field_at
			continue
		var fault := _check(d[k], spec[0], field_at)
		if fault != "":
			return fault
	return ""


static func _primitive(value: Variant, kind: String, where: String) -> String:
	var number := value is float or value is int
	match kind:
		"num":
			return "" if number else "%s: not a number" % where
		"int", "uint":
			if not number or float(value) != floorf(float(value)):
				return "%s: not an integer" % where
			if kind == "uint" and float(value) < 0.0:
				return "%s: negative" % where
			return ""
		"str":
			return "" if value is String else "%s: not a string" % where
		"bool":
			return "" if value is bool else "%s: not a boolean" % where
		"key":
			return "" if value is String and _word(value, 48, false) else "%s: '%s' is not a key" % [where, str(value)]
		"name":
			return "" if value is String and _word(value, 64, true) else "%s: '%s' is not a name" % [where, str(value)]
		"colour":
			var hex := "0123456789abcdefABCDEF"
			if value is String and (value as String).length() == 7 and (value as String).begins_with("#"):
				for c in (value as String).substr(1):
					if not hex.contains(c):
						return "%s: not a colour (#rrggbb)" % where
				return ""
			return "%s: not a colour (#rrggbb)" % where
	return "%s: the schema has no primitive %s" % [where, kind]


static func _word(text: String, longest: int, underscore: bool) -> bool:
	if text.length() < 1 or text.length() > longest:
		return false
	for c in text:
		var fits := (c >= "a" and c <= "z") or (c >= "0" and c <= "9") or c == "-" or (underscore and c == "_")
		if not fits:
			return false
	return true


# --- the conditional shapes (the validator's check_shape, in its order) --------------------------

static func _shape(root: Dictionary) -> String:
	if int(root["format"]) != 1:
		return "format %d is not 1, the only format" % int(root["format"])
	var terrain: Dictionary = root.get("terrain", {}) if root.get("terrain") != null else {}
	var regions: Array = terrain.get("regions", [])
	for i in regions.size():
		var h: Dictionary = regions[i]["height"]
		var at := "terrain.regions[%d].height" % i
		if h.has("flat_m") and not h.has("ramp"):
			if h.has("crown_m") != h.has("crown_along"):
				return "%s: crown_m and crown_along are stated together" % at
		elif not (h.has("ramp") and not h.has("flat_m") and not h.has("crown_m") and not h.has("crown_along")):
			return "%s: exactly one of flat_m (with an optional crown) and ramp" % at
	var edges: Array = _list(root, "edges")
	for i in edges.size():
		var e: Dictionary = edges[i]
		var at := "edges[%d] \"%s\"" % [i, e["name"]]
		if (e["kind"] in ["quay", "revetment"]) != e.has("drop_m"):
			return "%s: drop_m is stated for a quay or a revetment, and only for them" % at
		if (e["kind"] == "rail") != e.has("posts_m"):
			return "%s: posts_m is stated for a rail, and only for a rail" % at
		if (e["line_mm"] as Array).size() < 2:
			return "%s.line_mm: a line has at least two points" % at
	var paths: Array = _list(root, "paths")
	for i in paths.size():
		if (paths[i]["line_mm"] as Array).size() < 2:
			return "paths[%d].line_mm: a line has at least two points" % i
	var buildings: Array = _list(root, "buildings")
	for i in buildings.size():
		var b: Dictionary = buildings[i]
		var at := "buildings[%d] \"%s\"" % [i, b["name"]]
		if (b["roof"]["form"] == "mono") != b["roof"].has("falls"):
			return "%s.roof: falls is stated for a mono roof, and only for it" % at
		var openings: Array = b.get("openings", [])
		for j in openings.size():
			if bool(openings[j].get("at_doorway", false)) == openings[j].has("offset_m"):
				return "%s.openings[%d]: exactly one of offset_m and at_doorway: true" % [at, j]
	if root.get("trees") != null:
		var stands: Array = root["trees"].get("stands", [])
		for i in stands.size():
			if stands[i]["area"].has("ring") == stands[i]["area"].has("polygon_mm"):
				return "trees.stands[%d].area: exactly one of ring and polygon_mm" % i
		var walls: Array = root["trees"].get("air_walls", [])
		for i in walls.size():
			var w: Dictionary = walls[i]
			if w.has("ring") == w.has("line_mm"):
				return "trees.air_walls[%d]: exactly one of ring and line_mm" % i
			if w.has("line_mm") and (w["line_mm"] as Array).size() < 2:
				return "trees.air_walls[%d].line_mm: a line has at least two points" % i
	var models: Array = _list(root, "models")
	for i in models.size():
		var a: Dictionary = models[i]["anchor"]
		var doorway := a.has("entrance") and bool(a.get("at_doorway", false)) and not a.has("at_mm") \
			and not a.has("yaw_deg")
		var placed := a.has("at_mm") and not a.has("entrance") and not bool(a.get("at_doorway", false))
		if not (doorway or placed):
			return "models[%d] \"%s\".anchor: exactly one of entrance with at_doorway: true, or at_mm (with an optional yaw_deg)" \
				% [i, models[i]["asset"]]
	var scatter: Array = _list(root, "scatter")
	for i in scatter.size():
		var a: Dictionary = scatter[i]["area"]
		var at := "scatter[%d] \"%s\".area" % [i, scatter[i]["name"]]
		var polygon := a.has("polygon_mm") and not a.has("along_mm") and not a.has("width_m")
		var strip := a.has("along_mm") and a.has("width_m") and not a.has("polygon_mm")
		if not (polygon or strip):
			return "%s: exactly one of polygon_mm, or along_mm with width_m" % at
		if strip and (a["along_mm"] as Array).size() < 2:
			return "%s: a line has at least two points" % at
	return ""


static func _list(root: Dictionary, section: String) -> Array:
	return root[section] if root.get(section) != null else []


# --- the names only this client knows -------------------------------------------------------------

static func _names(root: Dictionary) -> String:
	var materials := []
	if root.get("terrain") != null:
		var layers: Array = root["terrain"]["layers"]
		for i in layers.size():
			materials.append(["terrain.layers[%d].material" % i, layers[i]["material"]])
	var edges := _list(root, "edges")
	for i in edges.size():
		materials.append(["edges[%d].material" % i, edges[i]["material"]])
	var buildings := _list(root, "buildings")
	for i in buildings.size():
		materials.append(["buildings[%d].walls.material" % i, buildings[i]["walls"]["material"]])
		materials.append(["buildings[%d].roof.material" % i, buildings[i]["roof"]["material"]])
	for m in materials:
		if not has_texture(m[1]):
			return "%s: this client's texture library has no %s" % [m[0], m[1]]
	if root.get("trees") != null:
		var species: Array = root["trees"]["species"]
		for i in species.size():
			if not (species[i]["form"] in FORMS):
				return "trees.species[%d].form: this client has no tree form %s (it has %s)" \
					% [i, species[i]["form"], ", ".join(PackedStringArray(FORMS))]
			if not (species[i]["card"] in CARDS):
				return "trees.species[%d].card: this client has no card %s (it has %s)" \
					% [i, species[i]["card"], ", ".join(PackedStringArray(CARDS))]
	return ""


## Whether the client's texture library has the slug (`Mats.tex`'s path, imported or not).
static func has_texture(slug: String) -> bool:
	var file := "%s/%s/%s_diff_1k.jpg" % [Mats.TEX_DIR, slug, slug]
	return ResourceLoader.exists(file) or FileAccess.file_exists(file)
