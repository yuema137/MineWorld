extends SceneTree

## What the 2D client's readers call every item each recorded observation names, and every panel row they
## show (`tools/cli/tests/item_names.rs`). For each `<world>.json` under `--fixtures` (an array of
## observations) it prints, per observation `n`:
##
##   THING <world> <n> <id> <name>     for every item the observation names by a typed reference
##   ROW <world> <n> <row>             for every row of every panel
##
##   godot --headless --path clients/2d --script res://scripts/harness/item_names.gd -- --fixtures=<dir>
##
## It reads only what the client reads: names come from `readers.gd` after it learned the observation,
## exactly as the app does on each frame. No name is written here.

const Readers := preload("res://scripts/hud/readers.gd")
const Words := preload("res://scripts/hud/words.gd")


func _init() -> void:
	var directory := ""
	for argument in OS.get_cmdline_user_args():
		if argument.begins_with("--fixtures="):
			directory = argument.trim_prefix("--fixtures=")
	var pack := ProjectSettings.globalize_path("res://").path_join("../../presentation/mineworld-default/2D")
	Words.load_pack(pack, "en")
	var files := DirAccess.get_files_at(directory)
	files.sort()
	for file in files:
		if not file.ends_with(".json"):
			continue
		var world := file.trim_suffix(".json")
		var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(directory.path_join(file)))
		if typeof(parsed) != TYPE_ARRAY:
			printerr("ITEMS %s: not an array of observations" % file)
			quit(1)
			return
		for n in parsed.size():
			var observation := MineWorldObservation.new(parsed[n])
			var readers := Readers.new()
			readers.learn(observation)
			var items := {}
			_items(parsed[n], items)
			for id in items:
				print("THING %s %d %s %s" % [world, n, id, readers.thing(id)])
			for panel in readers.panels(observation):
				for row in panel["rows"]:
					print("ROW %s %d %s" % [world, n, row])
	quit(0)


## Every item `value` names by a typed reference (`{entity, entity_type: "item"}`), at any depth.
static func _items(value: Variant, found: Dictionary) -> void:
	match typeof(value):
		TYPE_DICTIONARY:
			if value.get("entity_type") == "item" and typeof(value.get("entity")) == TYPE_STRING:
				found[value["entity"]] = true
				return
			for key in value:
				_items(value[key], found)
		TYPE_ARRAY:
			for entry in value:
				_items(entry, found)
