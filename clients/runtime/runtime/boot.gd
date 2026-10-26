extends Node
## The shared runtime's one scene (step-23 PR S23-RT §4.1, ARC-80).
##
## A release bundle carries one Godot engine for both clients. Official release templates refuse
## `--main-pack` (step-23 probe P-3), so the engine starts this small pack instead, and this script mounts
## the client's pack with `ProjectSettings.load_resource_pack()`, which also refreshes the global script
## classes and the UID cache. That call does not re-read the client's `project.godot`, so the settings the
## engine reads only at start-up live in this project's `project.godot` (the packager checks they match),
## and the rest are applied here from `<pack>.settings.json`, which the packager writes from the client's
## `project.godot`. `appliers.json` is the one list of those keys; the packager reads it too.
##
## User arguments: `--client=<key>` and `--root=<runtime folder>`. `BUNDLE.toml` in the root names the
## client's pack (`client_<key>_pack`) and its first scene (`client_<key>_scene`). Every failure prints
## `[runtime] FAIL <what>` and quits with status 2. The client's own arguments pass through untouched; the
## clients ignore `--client=`.

const APPLIERS := "res://runtime/appliers.json"
const STRETCH_MODES := {"disabled": Window.CONTENT_SCALE_MODE_DISABLED,
	"canvas_items": Window.CONTENT_SCALE_MODE_CANVAS_ITEMS, "viewport": Window.CONTENT_SCALE_MODE_VIEWPORT}
const STRETCH_ASPECTS := {"ignore": Window.CONTENT_SCALE_ASPECT_IGNORE,
	"keep": Window.CONTENT_SCALE_ASPECT_KEEP, "keep_width": Window.CONTENT_SCALE_ASPECT_KEEP_WIDTH,
	"keep_height": Window.CONTENT_SCALE_ASPECT_KEEP_HEIGHT, "expand": Window.CONTENT_SCALE_ASPECT_EXPAND}


func _ready() -> void:
	var key := ""
	var root := ""
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--client="):
			key = a.substr(9)
		elif a.begins_with("--root="):
			root = a.substr(7)
	if key == "" or root == "":
		_fail("--client=<key> and --root=<runtime folder> are both required")
		return
	var bundle := read_bundle(root.path_join("BUNDLE.toml"))
	var pack: String = bundle.get("client_%s_pack" % key, "")
	var scene: String = bundle.get("client_%s_scene" % key, "")
	if pack == "" or scene == "":
		_fail("%s names no client_%s_pack or client_%s_scene" % [root.path_join("BUNDLE.toml"), key, key])
		return
	var settings_path := root.path_join(pack.get_basename() + ".settings.json")
	var settings: Variant = JSON.parse_string(FileAccess.get_file_as_string(settings_path))
	var appliers: Variant = JSON.parse_string(FileAccess.get_file_as_string(APPLIERS))
	if typeof(settings) != TYPE_DICTIONARY or typeof(appliers) != TYPE_DICTIONARY:
		_fail("cannot read %s or %s" % [settings_path, APPLIERS])
		return
	var run_time: Dictionary = appliers["run_time"]
	for setting in settings:
		if not run_time.has(setting):
			_fail("%s: %s is not a run-time setting this runtime applies" % [settings_path, setting])
			return
	if not ProjectSettings.load_resource_pack(root.path_join(pack), true):
		_fail("cannot load %s" % root.path_join(pack))
		return
	for setting in settings:
		call("_" + str(run_time[setting]), settings[setting])
	print("[runtime] client %s: %s mounted, %d setting(s) applied, starting %s"
		% [key, pack, settings.size(), scene])
	get_tree().change_scene_to_file.call_deferred(scene)


## `BUNDLE.toml`'s `key = "value"` lines (the file is the packager's and holds nothing else).
static func read_bundle(path: String) -> Dictionary:
	var out := {}
	var line_re := RegEx.create_from_string('^(\\w+) = "(.*)"$')
	for line in FileAccess.get_file_as_string(path).split("\n"):
		var m := line_re.search(line.strip_edges())
		if m:
			out[m.get_string(1)] = m.get_string(2)
	return out


func _fail(why: String) -> void:
	printerr("[runtime] FAIL ", why)
	get_tree().quit(2)


func _title(v: Variant) -> void:
	get_window().title = str(v)


func _quit_on_go_back(v: Variant) -> void:
	get_tree().quit_on_go_back = bool(v)


func _stretch_mode(v: Variant) -> void:
	get_window().content_scale_size = Vector2i(
		ProjectSettings.get_setting("display/window/size/viewport_width"),
		ProjectSettings.get_setting("display/window/size/viewport_height"))
	get_window().content_scale_mode = STRETCH_MODES[str(v)]


func _stretch_aspect(v: Variant) -> void:
	get_window().content_scale_aspect = STRETCH_ASPECTS[str(v)]


func _canvas_texture_filter(v: Variant) -> void:
	get_viewport().canvas_item_default_texture_filter = int(v)


func _msaa_2d(v: Variant) -> void:
	get_viewport().msaa_2d = int(v)


func _msaa_3d(v: Variant) -> void:
	get_viewport().msaa_3d = int(v)


func _screen_space_aa(v: Variant) -> void:
	get_viewport().screen_space_aa = int(v)


func _occlusion_culling(v: Variant) -> void:
	get_viewport().use_occlusion_culling = bool(v)


func _anisotropic_filtering(v: Variant) -> void:
	get_viewport().anisotropic_filtering_level = int(v)


func _directional_shadow_size(v: Variant) -> void:
	RenderingServer.directional_shadow_atlas_set_size(int(v), true)


func _directional_soft_shadow(v: Variant) -> void:
	RenderingServer.directional_soft_shadow_filter_set_quality(int(v))


func _positional_soft_shadow(v: Variant) -> void:
	RenderingServer.positional_soft_shadow_filter_set_quality(int(v))


func _clear_color(v: Variant) -> void:
	var c: Array = v
	RenderingServer.set_default_clear_color(Color(c[0], c[1], c[2], c[3]))
