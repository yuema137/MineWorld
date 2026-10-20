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
## A character resolves to a body through this client's resource index
## (`assets/characters/cast/index.json`, PRESENTATION.md §1): which imported body draws it. A character
## the index does not list is drawn as the town figure, with a warning, so a newer pack degrades.
class_name SliceCast
extends RefCounted

const PACK_FILE := "characters/cast.yaml"
const INDEX := "res://assets/characters/cast/index.json"

var looks: Dictionary = {}
var extras: Array = []
var default_character := ""
var _bodies: Dictionary = {}
var problems: PackedStringArray = []


## Reads `<pack_dir>/characters/cast.yaml` and this client's resource index. A missing or faulty file
## leaves the binding empty (everyone is the town figure) and is reported in [member problems].
static func load_pack(pack_dir: String) -> SliceCast:
	var c := SliceCast.new()
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


## Whether a character is drawn by the reference body (the imported route D+ character) rather than the
## town figure.
func is_reference(character: String) -> bool:
	if character == "":
		return false
	var entry: Variant = _bodies.get(character)
	if typeof(entry) != TYPE_DICTIONARY:
		push_warning("cast: character '%s' has no body in %s; drawn as the town figure" % [character, INDEX])
		return false
	return String(entry.get("body", "")) == "reference"


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
