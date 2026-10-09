extends RefCounted

## A Presentation Pack, read at runtime from disk, answering "what does this role look like?"
##
## The core asks for roles and never for files (`ARC-46`); this is the one script that opens the
## pack's two files and its images, so the format is changed here and specified in
## `clients/2d/PRESENTATION.md`. With no pack, or for a role the active set does not bind, every
## answer is empty and the caller draws plainly.

const BINDINGS := "assets/asset_bindings.yaml"
const RENDERER := "renderer/godot.yaml"

## The pack directory, absolute, or "" when drawing plainly.
var directory := ""
## The binding set in use.
var variant := ""
## The pack's renderer parameters (`renderer/godot.yaml`), or {} when plain.
var renderer: Dictionary = {}
## Problems met while loading: a file that would not read, a role bound to nothing. Each names its
## cause; the drive prints them and AC-W12 requires there to be none.
var errors := PackedStringArray()

var _sprites: Dictionary = {}
var _roles: Dictionary = {}
var _textures: Dictionary = {}
var _shaders: Dictionary = {}


## Loads `dir`'s pack with binding set `set_name` ("" for its default). `dir` == "" draws plainly.
func load_pack(dir: String, set_name: String) -> bool:
	if dir == "":
		return true
	directory = dir.trim_suffix("/")
	var bindings := _read_json(BINDINGS)
	renderer = _read_json(RENDERER)
	if bindings.is_empty():
		directory = ""
		renderer = {}
		return false
	_sprites = bindings.get("sprites", {})
	var sets: Dictionary = bindings.get("sets", {})
	variant = set_name if set_name != "" else String(bindings.get("default_set", ""))
	if not sets.has(variant):
		errors.append("the pack has no binding set '%s' (it has: %s)" % [variant, ", ".join(sets.keys())])
		variant = String(bindings.get("default_set", ""))
	_roles = _resolve(sets, variant)
	return true


func is_plain() -> bool:
	return directory == ""


## The sprite a role resolves to: `{texture, scale, anchor, door, ink}`, or {} to draw plainly.
## `anchor` and `door` are in the texture's pixels; `scale` is the draw scale to apply.
func sprite(role: String) -> Dictionary:
	var id := String(_roles.get(role, ""))
	return sprite_by_id(id) if id != "" else {}


## A directional role (`player`, `person:n`): `{front, front_b, back, back_b}`, each a sprite or {}.
func directional(role: String) -> Dictionary:
	var base := String(_roles.get(role, ""))
	if base == "":
		return {}
	if _sprites.has(base):
		var one := sprite_by_id(base)
		return {} if one.is_empty() else {"front": one, "front_b": {}, "back": {}, "back_b": {}}
	var views := {}
	for view in ["front", "front_b", "back", "back_b"]:
		var id := "%s_%s" % [base, view]
		views[view] = sprite_by_id(id) if _sprites.has(id) else {}
	return {} if views["front"].is_empty() else views


## How many cast roles (`person:0`, `person:1`, …) the set binds contiguously from 0.
func cast_size() -> int:
	var n := 0
	while _roles.has("person:%d" % n):
		n += 1
	return n


func is_bound(role: String) -> bool:
	return _roles.has(role)


## Every role the active set binds, for the drive's "every role resolves" check (AC-W12).
func bound_roles() -> PackedStringArray:
	return PackedStringArray(_roles.keys())


## A sprite by id, loading its image the first time. {} when it is missing or will not load.
func sprite_by_id(id: String) -> Dictionary:
	if not _sprites.has(id):
		return {}
	if _textures.has(id):
		return _textures[id]
	var spec: Dictionary = _sprites[id]
	var file := String(spec.get("file", ""))
	var draw_scale := float(spec.get("scale", 1.0))
	var raster := 1.0
	var image: Image
	if file.ends_with(".svg"):
		# Rasterized at twice its draw scale and drawn down, so edges stay clean (the spike's rule).
		raster = clampf(draw_scale * 2.0, 0.05, 4.0)
		image = Image.new()
		var bytes := FileAccess.get_file_as_bytes(_path(file))
		if bytes.is_empty() or image.load_svg_from_buffer(bytes, raster) != OK:
			image = null
	else:
		image = Image.load_from_file(_path(file))
	if image == null or image.is_empty():
		errors.append("cannot load %s for sprite %s" % [file, id])
		_textures[id] = {}
		return {}
	image.generate_mipmaps()
	var anchor: Array = spec.get("anchor", [image.get_width() / 2.0, image.get_height()])
	var door: Variant = spec.get("door")
	var loaded := {
		"id": id,
		"texture": ImageTexture.create_from_image(image),
		"scale": draw_scale / raster,
		"anchor": Vector2(float(anchor[0]), float(anchor[1])) * raster,
		"door": null if door == null else Vector2(float(door[0]), float(door[1])) * raster,
		"ink": String(spec.get("ink", "none")),
		"height_m": spec.get("height_m"),
	}
	_textures[id] = loaded
	return loaded


## A texture role (`texture:floor`, `texture:paper`), or null.
func texture(role: String) -> Texture2D:
	var found := sprite(role)
	return found.get("texture") if not found.is_empty() else null


## A shader the renderer parameters name, compiled from its source text, or null.
func shader(file: String) -> Shader:
	if is_plain() or file == "":
		return null
	if _shaders.has(file):
		return _shaders[file]
	var source := FileAccess.get_file_as_string(_path(file))
	var compiled: Shader = null
	if source == "":
		errors.append("cannot read shader %s" % file)
	else:
		compiled = Shader.new()
		compiled.code = source
	_shaders[file] = compiled
	return compiled


## A named palette colour, or `fallback`.
func colour(name: String, fallback: Color) -> Color:
	var palette: Dictionary = renderer.get("palette", {})
	return Color(String(palette[name])) if palette.has(name) else fallback


## A value at a path of keys in the renderer parameters, or `fallback`.
func setting(path: Array, fallback: Variant) -> Variant:
	var at: Variant = renderer
	for key in path:
		if typeof(at) != TYPE_DICTIONARY or not at.has(key):
			return fallback
		at = at[key]
	return at


func _resolve(sets: Dictionary, name: String) -> Dictionary:
	var roles := {}
	var seen := {}
	while name != "" and sets.has(name) and not seen.has(name):
		seen[name] = true
		var own: Dictionary = sets[name].get("roles", {})
		for role in own:
			if not roles.has(role):
				roles[role] = own[role]
		name = String(sets[name].get("extends", ""))
	return roles


func _read_json(file: String) -> Dictionary:
	var text := FileAccess.get_file_as_string(_path(file))
	if text == "":
		errors.append("cannot read %s" % _path(file))
		return {}
	var parsed: Variant = JSON.parse_string(text)
	if typeof(parsed) != TYPE_DICTIONARY:
		errors.append("%s is not YAML's JSON subset (DEP-16)" % _path(file))
		return {}
	return parsed


func _path(file: String) -> String:
	return "%s/%s" % [directory, file]
