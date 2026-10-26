extends SceneTree

## The client's depiction reader over the shared corpus, headless (S33-d AC-d-10; ARC-177).
##
##   godot --headless --path clients/3d-spike --script res://scripts/checks/depiction_check.gd
##
## Prints `DEPICTION <name> <code>` per file -- `<code>` is a `SliceDepictionFile.CODES` entry or `ok` --
## then `depiction_check: <n> files`. It judges nothing itself: `tools/cli/tests/client_depiction.rs`
## compares every line with what `mineworld_packages` says of the same file.
##
##   --corpus=<dir>   the corpus (default packages/tests/fixtures/depiction)
##   --pack=<dir>     also every depictions/*.yaml of a 3D Presentation Pack, named `3d/<stem>`

const DEFAULT_CORPUS := "../../packages/tests/fixtures/depiction"


func _init() -> void:
	var corpus := ProjectSettings.globalize_path("res://").path_join(DEFAULT_CORPUS).simplify_path()
	var packs := []
	for arg in OS.get_cmdline_user_args():
		if arg.begins_with("--corpus="):
			corpus = arg.substr(9)
		elif arg.begins_with("--pack="):
			packs.append(arg.substr(7))
	var count := 0
	for file in _yaml_files(corpus):
		_report(file.get_basename(), SliceDepictionFile.load_file(corpus.path_join(file)))
		count += 1
	for dir in packs:
		var files: String = dir.path_join(SliceDataDepiction.DIR)
		for file in _yaml_files(files):
			_report("3d/%s" % file.get_basename(), SliceDepictionFile.load_file(files.path_join(file)))
			count += 1
	print("depiction_check: %d files" % count)
	quit(0)


func _report(name: String, file: SliceDepictionFile) -> void:
	print("DEPICTION %s %s" % [name, "ok" if file.ok() else file.error_code])
	if not file.ok():
		print("  %s" % file.error_detail)


func _yaml_files(dir: String) -> PackedStringArray:
	var files := PackedStringArray()
	for file in DirAccess.get_files_at(dir):
		if file.get_extension() == "yaml":
			files.append(file)
	files.sort()
	return files
