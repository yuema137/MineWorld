extends SceneTree

## The settings store and display application, headless (step-20 §12.5 C2; AC-SET-8, AC-SET-15).
##
##   godot --headless --path clients/shared --script res://checks/store_check.gd
##
## Prints one `[PASS]`/`[FAIL]` line per assertion and `store_check: PASS|FAIL`; exits non-zero on any
## failure. Its files live in a scratch folder under the user directory, removed at the end; it never
## touches `settings.cfg` itself.
##
##   --where           only print and check the settings folder (AC-SET-15), and nothing else
##   --write=<name>    save a probe value to user://<name> (for the cross-project check)
##   --read=<name>     read user://<name> and print it as an EVIDENCE line, then delete it

var _fails := 0
var _scratch := ""


func _init() -> void:
	var args := OS.get_cmdline_user_args()
	_where()
	if args.has("--where"):
		_finish()
		return
	for arg in args:
		if arg.begins_with("--write="):
			_write_probe(arg.substr(8))
			_finish()
			return
		if arg.begins_with("--read="):
			_read_probe(arg.substr(7))
			_finish()
			return
	_scratch = OS.get_user_data_dir().path_join("store-check-scratch")
	_remove_tree(_scratch)
	DirAccess.make_dir_recursive_absolute(_scratch)
	_harness_flags()
	_round_trip()
	_bad_files()
	_language_not_offered()
	_unknown_key()
	_atomic()
	_merge()
	_display_headless()
	_remove_tree(_scratch)
	_finish()


func _finish() -> void:
	print("store_check: %s" % ("PASS" if _fails == 0 else "FAIL"))
	quit(0 if _fails == 0 else 1)


func _check(ok: bool, what: String, detail: String = "") -> void:
	if not ok:
		_fails += 1
	print("[%s] %s  %s" % ["PASS" if ok else "FAIL", what, detail])


## AC-SET-15: the shared settings folder, on the OS this runs on.
func _where() -> void:
	var dir := OS.get_user_data_dir()
	print("EVIDENCE ", JSON.stringify({"where": {"os": OS.get_name(), "user_dir": dir,
		"driver": RenderingServer.get_current_rendering_driver_name(), "display": DisplayServer.get_name()}}))
	_check(dir.replace("\\", "/").get_file() == "MineWorld", "the settings folder is the shared MineWorld folder", dir)


func _write_probe(name: String) -> void:
	var value := MineWorldSettings.new()
	value.language = "zh_Hans"
	value.window_size = Vector2i(1280, 720)
	var store := MineWorldSettingsStore.open(PackedStringArray(["--settings=user://" + name]), PackedStringArray())
	_check(store.save(value) == OK, "probe saved", store.absolute_path())


func _read_probe(name: String) -> void:
	var store := MineWorldSettingsStore.open(PackedStringArray(["--settings=user://" + name]), PackedStringArray())
	print("EVIDENCE ", JSON.stringify({"probe": {"language": store.settings.language,
		"window_size": [store.settings.window_size.x, store.settings.window_size.y], "problems": store.problems.size()}}))
	DirAccess.remove_absolute(store.absolute_path())


func _file(name: String) -> String:
	return _scratch.path_join(name)


func _open(file: String, flags: PackedStringArray = PackedStringArray()) -> MineWorldSettingsStore:
	var args := PackedStringArray(["--settings=" + file])
	args.append_array(flags)
	return MineWorldSettingsStore.open(args, PackedStringArray(["drive", "capture"]))


func _harness_flags() -> void:
	var plain := MineWorldSettingsStore.open(PackedStringArray([]), PackedStringArray(["drive"]))
	_check(plain.path == MineWorldSettingsStore.DEFAULT_PATH, "no flag: the user's file", plain.path)
	var driven := MineWorldSettingsStore.open(PackedStringArray(["--drive=walk"]), PackedStringArray(["drive"]))
	_check(driven.path == "" and driven.settings.equals(MineWorldSettings.new()),
		"a harness flag means --settings=none", driven.path)
	var chosen := MineWorldSettingsStore.open(PackedStringArray(["--drive", "--settings=" + _file("x.cfg")]),
		PackedStringArray(["drive"]))
	_check(chosen.path == _file("x.cfg"), "a harness flag with --settings=<path> uses that path", chosen.path)
	var none := MineWorldSettingsStore.open(PackedStringArray(["--settings=none"]), PackedStringArray())
	var value := MineWorldSettings.new()
	value.language = "zh_Hans"
	_check(none.save(value) == OK and none.settings.language == "zh_Hans" and not FileAccess.file_exists(_file("none")),
		"--settings=none keeps a value in memory and writes nothing")


## Every field survives a save and a load.
func _round_trip() -> void:
	var file := _file("round.cfg")
	var value := MineWorldSettings.new()
	value.language = "zh_Hans"
	value.clock = MineWorldSettings.ClockFormat.H24
	value.window_mode = MineWorldSettings.WindowMode.BORDERLESS
	value.window_size = Vector2i(1280, 720)
	value.render_scale = 67
	value.vsync = MineWorldSettings.VSync.ADAPTIVE
	value.max_fps = -1
	_check(_open(file).save(value) == OK, "a value saves", file)
	var back := _open(file)
	_check(back.settings.equals(value) and back.problems.is_empty(), "every field round-trips",
		FileAccess.get_file_as_string(file).replace("\n", " | "))
	var config := ConfigFile.new()
	config.load(file)
	_check(config.get_value("meta", "version") == 1 and typeof(config.get_value("display", "max_fps")) == TYPE_INT,
		"the file is versioned and its integers stay integers")


## AC-SET-8: a bad file never breaks a client; it is left as it is until an Apply, which keeps it as .bak.
func _bad_files() -> void:
	var cases := {
		"unparsable": ["this is [not a = config\n\u0000\u0001", "", ""],
		"window_mode": ["[meta]\nversion=1\n[display]\nwindow_mode=7\nmax_fps=60\n", "display/window_mode", "max_fps"],
		"max_fps": ["[meta]\nversion=1\n[display]\nmax_fps=\"fast\"\nvsync=\"off\"\n", "display/max_fps", "vsync"],
		"version": ["[meta]\nversion=99\n[general]\nlanguage=\"zh_Hans\"\n", "meta/version", ""],
	}
	for name in cases:
		var bytes: String = cases[name][0]
		var key: String = cases[name][1]
		var file := _file(name + ".cfg")
		var out := FileAccess.open(file, FileAccess.WRITE)
		out.store_string(bytes)
		out.close()
		var before := FileAccess.get_file_as_bytes(file)
		var store := _open(file)
		var said := "\n".join(store.problems)
		_check(store.problems.size() >= 1 and said.contains(file) and (key == "" or said.contains(key)),
			"%s: a warning names the file and the key" % name, said)
		match name:
			"window_mode":
				_check(store.settings.window_mode == MineWorldSettings.WindowMode.WINDOWED and store.settings.max_fps == 60,
					"window_mode: the bad key takes its default, the good one is kept")
			"max_fps":
				_check(store.settings.max_fps == 0 and store.settings.vsync == MineWorldSettings.VSync.OFF,
					"max_fps: the bad key takes its default, the good one is kept")
			_:
				_check(store.settings.equals(MineWorldSettings.new()), "%s: the defaults" % name)
		_check(FileAccess.get_file_as_bytes(file) == before, "%s: the file is byte-identical after loading" % name)
		var applied := store.settings.copy()
		applied.clock = MineWorldSettings.ClockFormat.H12
		_check(store.save(applied) == OK, "%s: an apply saves" % name)
		_check(FileAccess.get_file_as_bytes(file + ".bak") == before, "%s: the old content is kept as .bak" % name)
		var again := _open(file)
		_check(again.problems.is_empty() and again.settings.clock == MineWorldSettings.ClockFormat.H12,
			"%s: the new file reads clean" % name)


## AC-SET-8's fifth case: a language no catalog offers.
func _language_not_offered() -> void:
	var file := _file("tlh.cfg")
	var out := FileAccess.open(file, FileAccess.WRITE)
	out.store_string("[meta]\nversion=1\n[general]\nlanguage=\"tlh\"\nclock=\"24h\"\n")
	out.close()
	var before := FileAccess.get_file_as_bytes(file)
	var store := _open(file)
	_check(store.problems.is_empty() and store.settings.language == "tlh", "tlh: a well-formed locale reads")
	store.settle_language(PackedStringArray(["en", "zh_Hans"]))
	var said := "\n".join(store.problems)
	_check(store.settings.language == "en" and store.settings.clock == MineWorldSettings.ClockFormat.H24
		and said.contains(file) and said.contains("general/language"), "tlh: default language, a warning naming file and key", said)
	_check(FileAccess.get_file_as_bytes(file) == before, "tlh: the file is byte-identical")
	store.save(store.settings)
	_check(FileAccess.get_file_as_bytes(file + ".bak") == before, "tlh: an apply keeps the old content as .bak")


func _unknown_key() -> void:
	var file := _file("unknown.cfg")
	var out := FileAccess.open(file, FileAccess.WRITE)
	out.store_string("[meta]\nversion=1\n[general]\nlanguage=\"en\"\nserver_address=\"x\"\n[launcher]\nkeep_running=true\n")
	out.close()
	var store := _open(file)
	_check(store.problems.size() == 1 and store.problems[0].contains("general/server_address"),
		"an unknown key of an owned section is named and dropped; another module's section is not judged", "\n".join(store.problems))
	store.save(store.settings)
	var config := ConfigFile.new()
	config.load(file)
	_check(not config.has_section_key("general", "server_address") and config.get_value("launcher", "keep_running", false) == true,
		"after an apply the unknown key is gone and the other module's section is kept")


func _atomic() -> void:
	var file := _file("atomic.cfg")
	var value := MineWorldSettings.new()
	_check(_open(file).save(value) == OK, "a first save creates the file")
	value.max_fps = 30
	_check(_open(file).save(value) == OK, "a second save replaces it")
	_check(not FileAccess.file_exists(file + ".tmp"), "no .tmp is left behind")
	_check(_open(file).settings.max_fps == 30, "the replacement is what is read")
	# The crash window of Godot's Windows rename (remove, then move): only the .tmp exists.
	DirAccess.rename_absolute(file, file + ".tmp")
	var recovered := _open(file)
	_check(recovered.settings.max_fps == 30, "a lone .tmp is recovered on load")


## Two clients open at once: each writes only the section it changed.
func _merge() -> void:
	var file := _file("merge.cfg")
	_open(file).save(MineWorldSettings.new())
	var two_d := _open(file)
	var three_d := _open(file)
	var a := two_d.settings.copy()
	a.language = "zh_Hans"
	two_d.save(a)
	var b := three_d.settings.copy()
	b.window_mode = MineWorldSettings.WindowMode.FULLSCREEN
	three_d.save(b)
	var after := _open(file)
	_check(after.settings.language == "zh_Hans" and after.settings.window_mode == MineWorldSettings.WindowMode.FULLSCREEN,
		"a display apply in one client keeps a language apply of the other")


## Applying every value headless is a no-op without error (P-11), and the defaults change nothing.
func _display_headless() -> void:
	var window := get_root()
	# A client's window opens at the launchers' size, the default (a bare --script root is 100×100).
	window.size = MineWorldSettings.DEFAULT_WINDOW
	var before := [window.mode, window.size, Engine.max_fps]
	MineWorldDisplay.apply(MineWorldSettings.new(), window, PackedStringArray([MineWorldDisplay.CAP_RENDER_SCALE]))
	_check([window.mode, window.size, Engine.max_fps] == before, "the defaults change nothing", str(before))
	for mode in [MineWorldSettings.WindowMode.BORDERLESS, MineWorldSettings.WindowMode.FULLSCREEN, MineWorldSettings.WindowMode.WINDOWED]:
		for vsync in [MineWorldSettings.VSync.OFF, MineWorldSettings.VSync.ADAPTIVE, MineWorldSettings.VSync.ON]:
			var value := MineWorldSettings.new()
			value.window_mode = mode
			value.vsync = vsync
			value.render_scale = 50
			value.max_fps = -1
			MineWorldDisplay.apply(value, window, PackedStringArray([MineWorldDisplay.CAP_RENDER_SCALE]))
	_check(is_equal_approx(window.scaling_3d_scale, 0.5), "render scale applies with the capability", str(window.scaling_3d_scale))
	_check(Engine.max_fps == 0, "match display with no known rate is no cap", str(Engine.max_fps))
	var capped := MineWorldSettings.new()
	capped.max_fps = 30
	MineWorldDisplay.apply(capped, window, PackedStringArray())
	_check(Engine.max_fps == 30, "the frame cap applies", str(Engine.max_fps))
	MineWorldDisplay.apply(MineWorldSettings.new(), window, PackedStringArray([MineWorldDisplay.CAP_RENDER_SCALE]))
	_check(Engine.max_fps == 0 and is_equal_approx(window.scaling_3d_scale, 1.0), "the defaults restore")
	print(MineWorldDisplay.platform_line())


func _remove_tree(path: String) -> void:
	var dir := DirAccess.open(path)
	if dir == null:
		return
	for f in dir.get_files():
		dir.remove(f)
	DirAccess.remove_absolute(path)
