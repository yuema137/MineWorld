extends SceneTree

## AC-SET-13 / INV-SET-9: every character of every shipped catalog has a glyph in a bundled font —
## Godot's own default font or the bundled Noto Sans SC — with the system's fonts never asked.
##
##   godot --headless --path clients/shared --script res://checks/glyph_check.gd [-- --catalog=<dir>]...
##
## The shipped catalogs are the shared layer (`settings/locale/`) and every Presentation Pack layer
## (`presentation/*/*/i18n/`). `--catalog=<dir>` adds a folder (a mutation, or a user's catalog).

## Characters the formats put around catalog text: digits and the punctuation of numbers and times.
const FORMAT_CHARS := "0123456789:.,-–—/%+ ·…×"

var _fails := 0


func _init() -> void:
	var noto := MineWorldText.bundled_font()
	var engine_font: Font = ThemeDB.fallback_font
	_check(noto.has_char("设".unicode_at(0)), "the bundled font loads with system fallback off",
		noto.get_font_name())
	var dirs := PackedStringArray([MineWorldText.module_dir().path_join("locale")])
	var presentation := ProjectSettings.globalize_path("res://").path_join("../../presentation").simplify_path()
	for pack in DirAccess.get_directories_at(presentation):
		for view in DirAccess.get_directories_at(presentation.path_join(pack)):
			var i18n := presentation.path_join(pack).path_join(view).path_join("i18n")
			if DirAccess.dir_exists_absolute(i18n):
				dirs.append(i18n)
	for arg in OS.get_cmdline_user_args():
		if arg.begins_with("--catalog="):
			dirs.append(arg.substr(10))
	var files := 0
	var seen := {}
	for dir in dirs:
		for name in DirAccess.get_files_at(dir):
			if name.get_extension() != "po":
				continue
			files += 1
			var catalog := ResourceLoader.load(dir.path_join(name), "", ResourceLoader.CACHE_MODE_IGNORE) as Translation
			if catalog == null:
				_check(false, "%s parses" % dir.path_join(name))
				continue
			for key in catalog.get_message_list():
				if key == "":
					continue
				var said := String(catalog.get_message(key))
				for i in said.length():
					seen[said.unicode_at(i)] = "%s %s" % [dir.path_join(name), key]
	for i in FORMAT_CHARS.length():
		seen[FORMAT_CHARS.unicode_at(i)] = "a format character"
	var missing := PackedStringArray()
	for c: int in seen:
		if c < 32:
			continue
		if not (engine_font.has_char(c) or noto.has_char(c)):
			missing.append("U+%04X %s (%s)" % [c, String.chr(c), seen[c]])
	print("EVIDENCE ", JSON.stringify({"glyphs": {"catalogs": files, "characters": seen.size(),
		"missing": missing.size()}}))
	_check(files >= 4, "every shipped catalog is read", "%d files in %s" % [files, ", ".join(dirs)])
	_check(missing.is_empty(), "every shipped character has a bundled glyph", "\n".join(missing))
	print("glyph_check: %s" % ("PASS" if _fails == 0 else "FAIL"))
	quit(0 if _fails == 0 else 1)


func _check(ok: bool, what: String, detail: String = "") -> void:
	if not ok:
		_fails += 1
	print("[%s] %s  %s" % ["PASS" if ok else "FAIL", what, detail])
