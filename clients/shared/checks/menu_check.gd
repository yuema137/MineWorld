extends SceneTree

## The settings menu alone, headless (step-20 §12.5 C4): the live language switch over every text of
## the menu (AC-SET-1's walk), Apply writing the file, Revert, the display confirmation reverting when
## not kept and saving when kept, Close dropping what was not applied, `add_tab`, and the marker catalog
## (AC-SET-2's walk).
##
##   godot --headless --path clients/shared --script res://checks/menu_check.gd

const MARKER := "qaa"

var _fails := 0
var _scratch := ""


func _initialize() -> void:
	_run.call_deferred()


func _run() -> void:
	_scratch = OS.get_user_data_dir().path_join("menu-check-scratch")
	_clear(_scratch)
	DirAccess.make_dir_recursive_absolute(_scratch)
	MineWorldText.load_layers("")
	MineWorldText.set_language("en")
	var file := _scratch.path_join("menu.cfg")
	var store := MineWorldSettingsStore.open(PackedStringArray(["--settings=" + file]), PackedStringArray())
	var menu := MineWorldSettingsMenu.new()
	get_root().add_child(menu)
	menu.setup(store, PackedStringArray([MineWorldDisplay.CAP_RENDER_SCALE]), Theme.new())
	menu.open()
	await process_frame
	_check(menu.is_open() and menu._root.mouse_filter == Control.MOUSE_FILTER_STOP,
		"the open menu covers the screen with a control that takes the mouse")

	# AC-SET-1 on the menu alone: every text changes to Chinese, live, and comes back exactly.
	var english := _texts(menu)
	_check(english.size() >= 20, "the walk reads the menu", "%d texts" % english.size())
	menu._on_language(menu._locales.find("zh_Hans"))
	await process_frame
	var chinese := _texts(menu)
	_compare(english, chinese)
	menu._on_language(menu._locales.find("en"))
	await process_frame
	_check(_texts(menu) == english, "switching back to en reproduces every text exactly")

	# Apply saves general settings at once.
	menu._on_language(menu._locales.find("zh_Hans"))
	menu._on_clock(MineWorldSettings.ClockFormat.H24)
	menu.apply()
	var saved := _read(file)
	_check(saved.get_value("general", "language") == "zh_Hans" and saved.get_value("general", "clock") == "24h"
		and menu.confirm_left < 0.0, "Apply saves language and clock, with no display confirmation")
	# Revert brings back what is saved; Close drops what was not applied.
	menu._on_language(menu._locales.find("en"))
	menu.revert()
	_check(MineWorldText.language() == "zh_Hans" and menu.draft.language == "zh_Hans", "Revert restores the saved language")
	menu._on_language(menu._locales.find("en"))
	menu.close()
	_check(MineWorldText.language() == "zh_Hans", "Close drops an unapplied language")
	menu.open()

	# A display change waits to be kept, and reverts by itself.
	menu._on_max_fps(MineWorldSettings.FPS_CAPS.find(30))
	menu.apply()
	_check(menu.confirm_left > 14.0 and Engine.max_fps == 30 and menu._confirm.visible,
		"Apply applies a display change and asks to keep it", "%.1f s, cap %d" % [menu.confirm_left, Engine.max_fps])
	_check(int(_read(file).get_value("display", "max_fps", 0)) == 0, "an unkept display change is not saved")
	menu.tick(16.0)
	_check(menu.confirm_left < 0.0 and Engine.max_fps == 0 and not menu._confirm.visible and menu.draft.max_fps == 0,
		"no answer within 15 s reverts it")
	menu._on_max_fps(MineWorldSettings.FPS_CAPS.find(30))
	menu.apply()
	menu.keep_display()
	_check(int(_read(file).get_value("display", "max_fps", 0)) == 30 and Engine.max_fps == 30, "Keep saves it")
	menu._on_max_fps(MineWorldSettings.FPS_CAPS.find(0))
	menu.apply()
	menu.keep_display()

	# The tab API adds a tab and leaves the others alone.
	var before := [menu._tabs.get_tab_title(0), menu._tabs.get_tab_title(1)]
	var extra := Label.new()
	extra.name = "Later"
	menu.add_tab("ui.settings.tab.general", extra)
	_check(menu._tabs.get_tab_count() == 3 and [menu._tabs.get_tab_title(0), menu._tabs.get_tab_title(1)] == before
		and menu._tabs.get_tab_title(2) == MineWorldText.text("ui.settings.tab.general"), "add_tab adds one tab")
	menu._tabs.remove_child(extra)
	menu._tab_keys.pop_back()
	extra.free()

	# AC-SET-2 on the menu alone: under the marker catalog every text is a marked catalog entry.
	var marker_dir := _scratch.path_join("marker")
	_check(write_marker(marker_dir) > 50, "the marker catalog is generated from en.po")
	MineWorldText.load_layers("", PackedStringArray([marker_dir]))
	menu._on_language(menu._locales.find(MARKER) if menu._locales.has(MARKER) else 0)
	MineWorldText.set_language(MARKER)
	await process_frame
	var unmarked := PackedStringArray()
	for line in _texts(menu):
		if not line.contains("⟦"):
			unmarked.append(line)
	_check(unmarked.is_empty(), "under the marker catalog every text comes from a catalog", "\n".join(unmarked))

	MineWorldText.load_layers("")
	MineWorldText.set_language("en")
	menu.queue_free()
	_clear(_scratch)
	print("menu_check: %s" % ("PASS" if _fails == 0 else "FAIL"))
	quit(0 if _fails == 0 else 1)


## A marker catalog (locale `qaa`): every entry of the shared and 2D pack en.po wrapped as ⟦…⟧.
## Returns how many entries it wrote. Never in a shipped folder (R-SET-11).
static func write_marker(dir: String) -> int:
	DirAccess.make_dir_recursive_absolute(dir)
	var sources := [MineWorldText.module_dir().path_join("locale/en.po")]
	var pack := ProjectSettings.globalize_path("res://").path_join("../../presentation/mineworld-default/2D/i18n/en.po").simplify_path()
	if FileAccess.file_exists(pack):
		sources.append(pack)
	var entries := {}
	for source: String in sources:
		var catalog := ResourceLoader.load(source, "", ResourceLoader.CACHE_MODE_IGNORE) as Translation
		for key in catalog.get_message_list():
			if key != "":
				entries[key] = "⟦%s⟧" % String(catalog.get_message(key))
	var out := FileAccess.open(dir.path_join(MARKER + ".po"), FileAccess.WRITE)
	out.store_string("msgid \"\"\nmsgstr \"\"\n\"Language: %s\\n\"\n\"Content-Type: text/plain; charset=UTF-8\\n\"\n" % MARKER)
	for key: String in entries:
		out.store_string("\nmsgid \"%s\"\nmsgstr \"%s\"\n" % [key, String(entries[key]).c_escape()])
	out.close()
	return entries.size()


## The menu's texts, tab by tab (only the current tab is visible), without the language list: each
## language names itself, in every language.
func _texts(menu: MineWorldSettingsMenu) -> PackedStringArray:
	var out := PackedStringArray()
	var current := menu._tabs.current_tab
	for tab in menu._tabs.get_tab_count():
		menu._tabs.current_tab = tab
		for line in MineWorldText.visible_ui_texts(menu):
			if not line.contains(str(menu._language.get_path())) and not out.has(line):
				out.append(line)
	menu._tabs.current_tab = current
	return out


func _compare(english: PackedStringArray, chinese: PackedStringArray) -> void:
	var by_path := {}
	for line in chinese:
		by_path[line.get_slice("\t", 0)] = line.get_slice("\t", 1)
	var wrong := PackedStringArray()
	for line in english:
		var path := line.get_slice("\t", 0)
		var said: String = by_path.get(path, "")
		if said == line.get_slice("\t", 1) or not MineWorldText.has_cjk(said):
			wrong.append("%s: \"%s\" → \"%s\"" % [path, line.get_slice("\t", 1), said])
	_check(chinese.size() == english.size() and wrong.is_empty(),
		"every text of the menu turns Chinese at once", "\n".join(wrong))


func _read(file: String) -> ConfigFile:
	var config := ConfigFile.new()
	config.load(file)
	return config


func _check(ok: bool, what: String, detail: String = "") -> void:
	if not ok:
		_fails += 1
	print("[%s] %s  %s" % ["PASS" if ok else "FAIL", what, detail])


func _clear(path: String) -> void:
	var dir := DirAccess.open(path)
	if dir == null:
		return
	for sub in dir.get_directories():
		_clear(path.path_join(sub))
	for f in dir.get_files():
		dir.remove(f)
	DirAccess.remove_absolute(path)
