extends SceneTree

## The far-field validator over the shared corpus, headless (RL-e0 C4; AC-FF-3).
##
##   godot --headless --path clients/shared --script res://checks/far_field_check.gd
##
## Prints `FARFIELD <name> <code>` per file — `<code>` is a `MineWorldFarField.CODES` entry or `ok` —
## then `far_field_check: <n> files`. It judges nothing itself: `tools/cli/tests/client_far_field.rs`
## compares every line with what `mineworld_packages` says of the same file.
##
##   --corpus=<dir>       the corpus (default packages/tests/fixtures/far_field); a stem ending `-2d` is
##                        read as a 2D pack's, any other as a 3D pack's. One extra line, `missing`, asks
##                        for a key the corpus has no file for.
##   --pack=<dir>:<dim>   also every far_field/*.yaml of a Presentation Pack, as dimension 2d or 3d;
##                        named `<dim>/<stem>`

const DEFAULT_CORPUS := "../../packages/tests/fixtures/far_field"


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
		var stem := file.get_basename()
		var dimension := "2d" if stem.ends_with("-2d") else "3d"
		_report(stem, MineWorldFarField.load_file(corpus.path_join(file), PackedStringArray([dimension])))
		count += 1
	# The absence of a file is the MISSING case: a key with no file in a directory that has others.
	_report("missing", MineWorldFarField.load_for(corpus.get_base_dir(), "missing", "3d"))
	count += 1
	for spec in packs:
		var cut: int = spec.rfind(":")
		var dir: String = spec.substr(0, cut)
		var dimension: String = spec.substr(cut + 1)
		var fields := dir.path_join(MineWorldFarField.DIR)
		for file in _yaml_files(fields):
			_report("%s/%s" % [dimension, file.get_basename()],
				MineWorldFarField.load_file(fields.path_join(file), PackedStringArray([dimension])))
			count += 1
	print("far_field_check: %d files" % count)
	quit(0)


func _report(name: String, field: MineWorldFarField) -> void:
	print("FARFIELD %s %s" % [name, "ok" if field.ok() else field.error_code])
	if not field.ok():
		print("  %s" % field.error_detail)


func _yaml_files(dir: String) -> PackedStringArray:
	var files := PackedStringArray()
	for file in DirAccess.get_files_at(dir):
		if file.get_extension() == "yaml":
			files.append(file)
	files.sort()
	return files
