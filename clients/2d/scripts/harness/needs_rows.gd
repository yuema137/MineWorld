extends SceneTree

## The rows the 2D client's needs, sleep and day panels show for each recorded observation, in each
## locale (`pr-s35-ne-presentation.md` §4.3, AC-Ne-7). Prints one line per fixture and locale:
##
##   ROWS <fixture> <locale> <json rows>
##
## A person compares these lines with `tests/fixtures/observations/needs/rows.<locale>.txt`, which was
## written by hand from the fixtures. The 3D probe prints the same lines for the same fixtures.
##
##   godot --headless --path clients/2d --script res://scripts/harness/needs_rows.gd -- --fixtures=<dir>
##
## It reads only what the panels read: the rows come from `readers.gd`, and no wording or threshold is
## written here.

const Readers := preload("res://scripts/hud/readers.gd")
const Words := preload("res://scripts/hud/words.gd")

const FIXTURES := ["market-town-night", "commute-late", "commute-sleeper"]
const LOCALES := ["en", "zh_Hans"]


func _init() -> void:
	var directory := ""
	for argument in OS.get_cmdline_user_args():
		if argument.begins_with("--fixtures="):
			directory = argument.trim_prefix("--fixtures=")
	var pack := ProjectSettings.globalize_path("res://").path_join("../../presentation/mineworld-default/2D")
	for locale in LOCALES:
		Words.load_pack(pack, locale)
		for fixture in FIXTURES:
			var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(directory.path_join(fixture + ".json")))
			if typeof(parsed) != TYPE_DICTIONARY:
				printerr("ROWS %s: not an observation" % fixture)
				quit(1)
				return
			var observation := MineWorldObservation.new(parsed)
			var rows: Array = []
			for panel in Readers.new().panels(observation):
				if Readers.NEEDS_PANELS.has(panel["component"]):
					rows.append_array(panel["rows"])
			print("ROWS %s %s %s" % [fixture, locale, JSON.stringify(rows)])
	quit(0)
