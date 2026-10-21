## The 3D Presentation Pack this client is given (`docs/decisions/ARC-51.md`; `PRESENTATION.md` §0).
##
##   --presentation=<dir>   a 3D pack directory, absolute or relative to the repository root, as the
##                          2D client's flag (`clients/2d/PRESENTATION.md` §1). Absent: DEFAULT.
##
## The pack is resolved once, before anything is drawn, and this class is the only place a pack path is
## built: every reader asks `SlicePack.current.path("<relative path>")`. A directory that is not a 3D
## Presentation Pack is refused by name (PRESENTATION_PACK_INVALID): it has no `pack.yaml`, its `type`
## is not `presentation-pack`, or its `manifest.yaml` lists no `3d` dimension. A pack that lacks an
## optional file (the cast, the furnishings) is not refused; that part is drawn plainly.
##
## Only the two identity fields are read here, from the YAML these two files are written in (top-level
## `key: value` lines and a flow or block list). `mineworld packs validate` validates a pack in full;
## the client checks only that it was given a 3D Presentation Pack.
class_name SlicePack
extends RefCounted

## The default pack, relative to the repository root.
const DEFAULT := "presentation/mineworld-default/3D"
const FLAG := "--presentation="
## The refusal's code.
const INVALID := "PRESENTATION_PACK_INVALID"

## The pack this run reads, once selected.
static var current: SlicePack = null

## The pack's directory, absolute and simplified.
var dir := ""
## Why the directory is not a 3D Presentation Pack; "" when it is one.
var why := ""


## The pack this run was given (`--presentation=`, else DEFAULT), resolved and checked once.
static func select() -> SlicePack:
	if current == null:
		current = open(requested())
	return current


## The directory named on the command line, as given; DEFAULT when none is.
static func requested() -> String:
	for a in OS.get_cmdline_user_args():
		if a.begins_with(FLAG):
			return a.trim_prefix(FLAG)
	return DEFAULT


## `given` as an absolute path: an absolute path stays; a relative one is read from the repository root.
## A path that leaves the repository is allowed (a user's own pack).
static func resolve(given: String) -> String:
	if given.is_absolute_path():
		return given.simplify_path()
	return ProjectSettings.globalize_path("res://").path_join("../..").path_join(given).simplify_path()


## Reads and checks the pack at `given`.
static func open(given: String) -> SlicePack:
	var pack := SlicePack.new()
	pack.dir = resolve(given)
	pack.why = _check(pack.dir)
	return pack


func ok() -> bool:
	return why == ""


## A file or folder of the pack, by its path relative to the pack's root.
func path(relative: String) -> String:
	return dir.path_join(relative)


## The refusal, as printed and shown (RL-e0's line format).
func refusal_line() -> String:
	return "[mineworld-slice] %s: { pack: %s, why: %s }" % [INVALID, dir, why]


static func _check(dir: String) -> String:
	if not DirAccess.dir_exists_absolute(dir):
		return "no such directory"
	var identity := dir.path_join("pack.yaml")
	if not FileAccess.file_exists(identity):
		return "no pack.yaml"
	var type := _scalar(FileAccess.get_file_as_string(identity), "type")
	if type != "presentation-pack":
		return "pack.yaml says type: %s, not presentation-pack" % (type if type != "" else "(none)")
	var manifest := dir.path_join("manifest.yaml")
	if not FileAccess.file_exists(manifest):
		return "no manifest.yaml"
	var dimensions := _list(FileAccess.get_file_as_string(manifest), "dimension")
	if not dimensions.has("3d"):
		return "manifest.yaml lists dimension %s, not 3d" % [dimensions]
	return ""


## A top-level `key: value`, unquoted, without its comment; "" when absent.
static func _scalar(text: String, key: String) -> String:
	for line in text.split("\n"):
		if line.begins_with(key + ":"):
			return _bare(line.substr(key.length() + 1))
	return ""


## A top-level list `key: [a, b]` or `key:` followed by `  - a` lines.
static func _list(text: String, key: String) -> PackedStringArray:
	var out := PackedStringArray()
	var lines := text.split("\n")
	for i in lines.size():
		if not lines[i].begins_with(key + ":"):
			continue
		var rest := _bare(lines[i].substr(key.length() + 1))
		if rest.begins_with("["):
			for item in rest.trim_prefix("[").trim_suffix("]").split(",", false):
				out.append(_bare(item))
			return out
		for j in range(i + 1, lines.size()):
			var item := lines[j].strip_edges()
			if item == "" or item.begins_with("#"):
				continue
			if not item.begins_with("- "):
				break
			out.append(_bare(item.substr(2)))
		return out
	return out


static func _bare(value: String) -> String:
	var comment := value.find(" #")
	if comment >= 0:
		value = value.substr(0, comment)
	return value.strip_edges().trim_prefix("\"").trim_suffix("\"").trim_prefix("'").trim_suffix("'")
