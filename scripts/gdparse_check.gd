# Loads every GDScript file of the project it runs in, so that Godot parses and compiles each one
# (scripts/ci_gdparse.py; docs/decisions/ARC-154.md, note of 2026-10-26). Run as
#
#     godot --headless --path <project> --script <this file>
#
# after one `--import` of the project has written its global class cache. Godot reports a script that does
# not compile as `SCRIPT ERROR: Parse Error: ...` and the load fails, yet the process exits 0, so the
# verdict is ci_gdparse.py's: it reads the log. This file only names what it loaded and what failed, and
# ends with one summary line that the reader requires (a crash or an early exit prints none).
extends SceneTree


func _initialize() -> void:
	var paths: Array[String] = []
	_collect("res://", paths)
	paths.sort()
	var failed := 0
	for path in paths:
		var script := ResourceLoader.load(path, "", ResourceLoader.CACHE_MODE_REUSE) as GDScript
		if script == null:
			failed += 1
			printerr("GDPARSE FAIL %s: did not load" % path)
	print("GDPARSE checked %d scripts, %d failed to load" % [paths.size(), failed])
	quit(1 if failed > 0 else 0)


# Every *.gd under `dir`, following the client's symlinked shared modules, skipping hidden folders (.godot)
# and any folder Godot itself ignores (.gdignore).
func _collect(dir: String, paths: Array[String]) -> void:
	if FileAccess.file_exists(dir.path_join(".gdignore")):
		return
	for sub in DirAccess.get_directories_at(dir):
		if not sub.begins_with("."):
			_collect(dir.path_join(sub), paths)
	for file in DirAccess.get_files_at(dir):
		if file.get_extension() == "gd":
			paths.append(dir.path_join(file))
