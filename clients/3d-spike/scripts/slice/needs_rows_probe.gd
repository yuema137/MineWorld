extends SceneTree

## The rows the 3D slice's HUD shows for each recorded observation, in each locale (S35 N-e, AC-S35-10 /
## AC-Ne-7). Prints one line per fixture and locale, in the 2D harness's format:
##
##   ROWS <fixture> <locale> <json rows>
##
## The rows come from `SliceNeedsHud.rows`, the builder the HUD itself uses. The 2D harness
## (`clients/2d/scripts/harness/needs_rows.gd`) prints the same lines; the parity test compares them.
##
##   godot --headless --path clients/3d-spike --script res://scripts/slice/needs_rows_probe.gd -- --fixtures=<dir>

const NeedsHud := preload("res://scripts/slice/needs_hud.gd")

const FIXTURES := ["market-town-night", "commute-late", "commute-sleeper"]
const LOCALES := ["en", "zh_Hans"]


func _init() -> void:
	var directory := ""
	for argument in OS.get_cmdline_user_args():
		if argument.begins_with("--fixtures="):
			directory = argument.trim_prefix("--fixtures=")
	for locale in LOCALES:
		MineWorldText.load_layers("")
		MineWorldText.set_language(locale)
		for fixture in FIXTURES:
			var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(directory.path_join(fixture + ".json")))
			if typeof(parsed) != TYPE_DICTIONARY:
				printerr("ROWS %s: not an observation" % fixture)
				quit(1)
				return
			var observation := MineWorldObservation.new(parsed)
			var rows := NeedsHud.rows(observation, Callable(NeedsHud, "place_tags"))
			print("ROWS %s %s %s" % [fixture, locale, JSON.stringify(Array(rows))])
	quit(0)
