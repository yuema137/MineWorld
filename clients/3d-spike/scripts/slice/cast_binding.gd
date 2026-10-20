## The 3D Presentation Pack's cast binding, read as data (`clients/3d-spike/PRESENTATION.md` §§2-3,
## `docs/DECISIONS.md` `ARC-92`).
##
## THE ONE PLACE this client decides which body draws a person. Who the person is, what they are
## called and how they look come from the server's observation only (`MineWorldObservation.look`,
## `ARC-91`); this file turns that opaque look key into a character, and a character into a body. The
## player is resolved exactly like everyone else (`INV-1`): the observer is drawn as the Person it
## occupies, never as a fixed default avatar.
##
## Resolution, in order:
##
##     a disclosed look the pack binds        -> that character
##     otherwise, the pack binds extras        -> extras[posmod(hash(id string), len(extras))]
##     otherwise                               -> "" : the client's town figure
##     no world connected (standalone slice)   -> the pack's `default`
##
## `""` for an unbound look with no extras is a recorded deviation from PRESENTATION.md §3 (which
## names `default` there): drawing every unbound person as the default character would put the
## protagonist's body on the whole town. See `pr-vf-person-identity.md`.
##
## A character resolves to a body (`body_for`): the reference body when this client's resource index
## (`assets/characters/cast/index.json`, PRESENTATION.md §1) names it so; otherwise a character the pack carries a manifest for (`characters/cast/<id>/character.yaml`, step-26 §4.5,
## `ARC-93`) is drawn as its own body, built generically from that manifest (`Human.build_character`);
## see `body_for`, which also degrades a Git LFS pointer checkout to the town figure (PR S26-c1,
## `.structured-coding/plans/mvp0/pr-s26c1-cast-client.md`). Anything else is the town figure, with a
## warning, so a newer pack degrades.
class_name SliceCast
extends RefCounted

const PACK_FILE := "characters/cast.yaml"
const INDEX := "res://assets/characters/cast/index.json"
const CHARACTERS := "characters/cast"
const MANIFEST := "character.yaml"
const LFS_POINTER := "version https://git-lfs"

var looks: Dictionary = {}
var extras: Array = []
var default_character := ""
var _bodies: Dictionary = {}
var problems: PackedStringArray = []
var _pack_dir := ""
## character id -> the body spec `body_for` resolved, so a missing body warns once.
var _resolved: Dictionary = {}


## Reads `<pack_dir>/characters/cast.yaml` and this client's resource index. A missing or faulty file
## leaves the binding empty (everyone is the town figure) and is reported in [member problems].
static func load_pack(pack_dir: String) -> SliceCast:
	var c := SliceCast.new()
	c._pack_dir = pack_dir
	var pack: Variant = _read(pack_dir.path_join(PACK_FILE), c.problems)
	if typeof(pack) == TYPE_DICTIONARY:
		var l: Variant = pack.get("looks", {})
		c.looks = l if typeof(l) == TYPE_DICTIONARY else {}
		var e: Variant = pack.get("extras", [])
		c.extras = e if typeof(e) == TYPE_ARRAY else []
		c.default_character = String(pack.get("default", ""))
	var index: Variant = _read(ProjectSettings.globalize_path(INDEX), c.problems)
	if typeof(index) == TYPE_DICTIONARY and typeof(index.get("characters")) == TYPE_DICTIONARY:
		c._bodies = index["characters"]
	for p in c.problems:
		push_warning("cast: " + p)
	return c


## The character that draws a person: `look` is what the observation disclosed ("" for none) and `id`
## the entity id string. "" means the client's town figure.
func character_for(look: String, id: String) -> String:
	if look != "" and looks.has(look):
		return String(looks[look])
	if not extras.is_empty():
		return String(extras[posmod(id.hash(), extras.size())])
	return ""


## How a character is drawn: `{"kind": "cast", "manifest": {...}}` for a character whose manifest this
## pack carries (`characters/cast/<id>/character.yaml`, step-26 §4.5) and whose imported body is real;
## `{"kind": "reference"}` for the reference body this client's index names
## (`assets/characters/cast/index.json`: the protagonist, until her constants move into a manifest);
## `{}` — the town figure — otherwise.
##
## A manifest that is missing a field, or a body whose files are Git LFS pointers (a checkout without
## LFS, as CI's) or were never imported, is drawn as the town figure with **one** warning naming why, so
## a partial checkout or a newer pack degrades rather than fails. Resolved once per character.
func body_for(character: String) -> Dictionary:
	if character == "":
		return {}
	if _resolved.has(character):
		return _resolved[character]
	var spec := {}
	var entry: Variant = _bodies.get(character)
	if typeof(entry) == TYPE_DICTIONARY and String(entry.get("body", "")) == "reference":
		spec = {"kind": "reference"}
	else:
		var why := ""
		var path := _pack_dir.path_join(CHARACTERS).path_join(character).path_join(MANIFEST)
		var m: Variant = _read(path, PackedStringArray())
		if typeof(m) != TYPE_DICTIONARY:
			why = "no manifest at %s" % path
		else:
			why = manifest_problem(m, character)
			if why == "":
				why = asset_problem(String(m["scene"]))
		if why == "":
			spec = {"kind": "cast", "manifest": m}
		else:
			push_warning("cast: character '%s' drawn as the town figure: %s" % [character, why])
	_resolved[character] = spec
	return spec


## The key the player's slot compares to know whether its body must change: the character for a cast
## body, `reference` or `town` otherwise.
static func body_key(character: String, spec: Dictionary) -> String:
	match String(spec.get("kind", "")):
		"cast":
			return character
		"reference":
			return "reference"
	return "town"


## What is wrong with a manifest, naming the field; "" when nothing is (step-26 §4.5, X-14).
static func manifest_problem(m: Dictionary, character: String) -> String:
	if m.get("format") != 1.0:
		return "manifest format is not 1"
	if String(m.get("id", "")) != character:
		return "manifest id is %s, not %s" % [m.get("id"), character]
	for field in ["scene", "material"]:
		if typeof(m.get(field)) != TYPE_STRING or String(m[field]) == "":
			return "manifest field '%s' is missing" % field
	if not String(m["scene"]).begins_with("res://"):
		return "manifest field 'scene' is not a res:// path"
	for field in ["stature_m", "authored_height_m"]:
		if typeof(m.get(field)) != TYPE_FLOAT or float(m[field]) <= 0.0:
			return "manifest field '%s' is missing" % field
	if absf(float(m["stature_m"]) - float(m["authored_height_m"])) > 0.005:
		return "stature_m and authored_height_m differ: a cast body is drawn at scale 1"
	var clips: Variant = m.get("clips")
	if typeof(clips) != TYPE_DICTIONARY:
		return "manifest field 'clips' is missing"
	for field in ["walk_mps", "jog_mps"]:
		if typeof(clips.get(field)) != TYPE_FLOAT or float(clips[field]) <= 0.0:
			return "manifest field 'clips.%s' is missing" % field
	if m.get("grip") != null:
		return "manifest field 'grip' is set; this client has no grip for a cast body yet"
	return ""


## Why an imported body cannot be drawn; "" when it can. A glTF's buffer and base-colour texture that
## are still Git LFS pointer files (`version https://git-lfs…`) never imported into a mesh: loading the
## scene would only print the importer's errors again. Files the export does not carry as raw files are
## simply not checked, and the imported scene must exist.
static func asset_problem(scene: String) -> String:
	var stem := scene.get_basename()
	for file in [scene, stem + ".bin", stem + "_base_color.png"]:
		var f := FileAccess.open(file, FileAccess.READ)
		if f == null:
			continue
		var head := f.get_buffer(LFS_POINTER.length()).get_string_from_ascii()
		if head == LFS_POINTER:
			return "%s is a Git LFS pointer (fetch it with `git lfs pull`)" % file
	if not ResourceLoader.exists(scene):
		return "%s is not imported" % scene
	return ""


static func _read(path: String, problems: PackedStringArray) -> Variant:
	var f := FileAccess.open(path, FileAccess.READ)
	if f == null:
		problems.append("cannot read %s" % path)
		return null
	var parsed: Variant = JSON.parse_string(f.get_as_text())
	if typeof(parsed) != TYPE_DICTIONARY:
		problems.append("%s is not a JSON object" % path)
		return null
	return parsed
