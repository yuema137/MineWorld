extends Node

## The `settings` scenario (`./mineworld-2d --drive=settings`, S20 SET-a): the shared settings menu in
## the connected client (step-20 §7). Run by `drive.gd`, whose helpers it uses, and printed the same way.
##
## ```text
## EVIDENCE {"settings": …}     the settings in effect at the first frame (AC-SET-7, AC-SET-9)
## EVIDENCE {"texts": …}        how many UI texts the walk read, in each language (AC-SET-1)
## [PASS]/[FAIL] …              every text turns to the other language at once and back exactly
##                              (AC-SET-1); under the marker catalog every text is a catalog entry
##                              (AC-SET-2, with --marker); an open menu takes gameplay input (AC-SET-12)
## ```
##
## A short walk first gives the stub's frame record requests to compare (AC-SET-5); the run ends by
## leaving the world, so the stub sees every frame the client sent.

const MARKER := "qaa"

var drive: Node
var app: Node


## The settings in effect, as data: the file, what loading it found wrong, and what was applied. Printed
## by `drive.gd` at the start of every scripted run (AC-SET-7, AC-SET-8, AC-SET-9).
static func evidence(app: Node) -> String:
	var window: Window = app.get_window()
	return JSON.stringify({"settings": {"file": app.settings_store.path,
		"problems": app.settings_store.problems, "language": MineWorldText.language(),
		"clock": MineWorldText.clock(), "window": [window.size.x, window.size.y], "mode": window.mode,
		"max_fps": Engine.max_fps, "vsync": DisplayServer.window_get_vsync_mode(window.get_window_id())}})


func run() -> void:
	await drive._strides(2)
	app.settings_menu.open()
	await _frames(3)
	if app.options.has("apply"):
		await _apply(String(app.options["apply"]))
	var start := MineWorldText.language()
	var other := "en" if start == "zh_Hans" else "zh_Hans"
	var first := _texts()
	await _still("settings_%s_general" % start)
	MineWorldText.set_language(other)
	await _frames(3)
	var second := _texts()
	await _still("settings_%s_general" % other)
	app.settings_menu._tabs.current_tab = 1
	await _still("settings_%s_display" % other)
	app.settings_menu._tabs.current_tab = 0
	print("EVIDENCE ", JSON.stringify({"texts": {start: first.size(), other: second.size()}}))
	_changed(first, second, start, other)
	MineWorldText.set_language(start)
	await _frames(3)
	var again := _texts()
	drive._check(_masked(again) == _masked(first), "switching back reproduces every text exactly",
		"%d texts" % again.size())
	if app.options.has("marker"):
		await _marker()
	await _input_taken()
	app.settings_menu.close()
	# Hold the connection, so a frame sent late is seen; then leave, so the stub records the end.
	await drive._seconds(2.0)
	app.link.client.leave_world()
	await drive._seconds(0.5)


## The `display` scenario (AC-SET-11, windowed): what the window and the engine say after the settings
## were applied, and the mean frame rate over five seconds, counted frame by frame.
func display() -> void:
	await drive._seconds(1.0)
	var frames := 0
	var started := Time.get_ticks_usec()
	while Time.get_ticks_usec() - started < 5_000_000:
		await get_tree().process_frame
		frames += 1
	var seconds := (Time.get_ticks_usec() - started) / 1_000_000.0
	var window := get_window()
	print("EVIDENCE ", JSON.stringify({"display": {"window": [window.size.x, window.size.y],
		"mode": window.mode, "vsync": DisplayServer.window_get_vsync_mode(window.get_window_id()),
		"max_fps": Engine.max_fps, "mean_fps": frames / seconds,
		"refresh": DisplayServer.screen_get_refresh_rate(window.current_screen)}}))


## `--apply=<language>,<clock>,<W>x<H>`: chooses them in the menu, applies and keeps them, as a player
## would, so the store writes its file (AC-SET-6, AC-SET-7).
func _apply(spec: String) -> void:
	var parts := spec.split(",")
	var menu: MineWorldSettingsMenu = app.settings_menu
	menu._on_language(menu._locales.find(parts[0]))
	menu._on_clock(["auto", "12h", "24h"].find(parts[1]))
	var size := parts[2].split("x")
	menu.draft.window_size = Vector2i(int(size[0]), int(size[1]))
	menu.apply()
	menu.keep_display()
	await _frames(2)
	print("EVIDENCE ", JSON.stringify({"applied": {"file": app.settings_store.absolute_path(),
		"language": MineWorldText.language()}}))


## AC-SET-1: every text changed, and the Chinese side of each pair has Chinese in it.
func _changed(first: PackedStringArray, second: PackedStringArray, a: String, b: String) -> void:
	var by_path := {}
	for line in second:
		by_path[line.get_slice("\t", 0)] = line.get_slice("\t", 1)
	var wrong := PackedStringArray()
	for line in first:
		var path := line.get_slice("\t", 0)
		var before := line.get_slice("\t", 1)
		var after: String = by_path.get(path, "")
		var chinese := after if b == "zh_Hans" else before
		print("EVIDENCE ", JSON.stringify({"text": {"path": path, a: before, b: after}}))
		if after == before or not MineWorldText.has_cjk(chinese):
			wrong.append("%s: \"%s\" → \"%s\"" % [path, before, after])
	drive._check(first.size() >= 30 and first.size() == second.size() and wrong.is_empty(),
		"every UI text turns from %s to %s at once" % [a, b], "%d texts; %s" % [first.size(), " | ".join(wrong)])


## AC-SET-2: a marker catalog made from the English the client loaded, every entry wrapped as ⟦…⟧;
## registered in memory only, never written to a shipped folder (R-SET-11).
func _marker() -> void:
	var english := TranslationServer.get_translation_object("en")
	var marked := Translation.new()
	marked.locale = MARKER
	for key in english.get_message_list():
		if key != "":
			marked.add_message(key, "⟦%s⟧" % String(english.get_message(key)))
	TranslationServer.add_translation(marked)
	var start := MineWorldText.language()
	MineWorldText.set_language(MARKER)
	await _frames(3)
	var unmarked := PackedStringArray()
	for line in _texts():
		if not line.contains("⟦"):
			unmarked.append(line)
	drive._check(unmarked.is_empty(), "under the marker catalog every UI text comes from a catalog",
		" | ".join(unmarked))
	MineWorldText.set_language(start)
	TranslationServer.remove_translation(marked)
	await _frames(2)


## AC-SET-12: with the menu open, two seconds of W held and a click on the floor ask for nothing.
func _input_taken() -> void:
	var before: int = drive._requests.size()
	Input.action_press("move_up")
	await drive._seconds(2.0)
	Input.action_release("move_up")
	await drive._press_at(app.walker.body_plan + Vector2(1.0, 0.5))
	await drive._seconds(0.5)
	print("EVIDENCE ", JSON.stringify({"menu_input": {"requests_before": before, "requests_after": drive._requests.size()}}))
	drive._check(drive._requests.size() == before, "an open menu takes gameplay input",
		"%d requests while open" % (drive._requests.size() - before))


## Every UI text of the client, the menu's tabs one by one; the language list (each language names
## itself) and the live frame-rate reading are left out.
func _texts() -> PackedStringArray:
	var menu: MineWorldSettingsMenu = app.settings_menu
	var skip := [str(menu._language.get_path()), str(menu._fps_value.get_path())]
	var out := PackedStringArray()
	var current := menu._tabs.current_tab
	for tab in menu._tabs.get_tab_count():
		menu._tabs.current_tab = tab
		for line in MineWorldText.visible_ui_texts(app):
			var path := line.get_slice("\t", 0)
			if not skip.any(func(s: String) -> bool: return path.begins_with(s)) and not out.has(line):
				out.append(line)
	menu._tabs.current_tab = current
	return out


## Digits masked: the clock and the revision move while the walk runs.
static func _masked(lines: PackedStringArray) -> PackedStringArray:
	var out := PackedStringArray()
	var digits := RegEx.create_from_string("[0-9]+")
	for line in lines:
		out.append(digits.sub(line, "#", true))
	return out


## A still, with `--capture` (windowed): the menu over the HUD, for the operator's review.
func _still(name: String) -> void:
	if drive._capture != null:
		await drive._capture.shoot(name)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame
