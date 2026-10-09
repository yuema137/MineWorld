extends Node

## The interaction scenarios of the scripted checks (step-13 §15.3): `--drive=steps --steps=<file>`
## plays a list of steps a Rust test wrote, and `--drive=panels` reports what the panels show.
##
## The drive never names an action type (both client scans forbid it). The **test** writes the
## selectors — `{"choose": {"action_type": "talk"}}`, `{"choose": {"action_type": "buy", "about":
## "24"}}` — and this script resolves them against the entries the menu actually rendered, then presses
## that entry through `menu.choose(i)`, the path a mouse and the Enter key take. Subjects are opened
## by a click on the drawn figure, through the app's own picking.
##
## Output, one line each, added to the drive's:
## ```text
## MENU {subject, self, entries: [...], offered: [...], t_ms}   a menu opened or its entries changed
## PANELS {seq, panels: [...], t_ms}                             the panels changed
## TOAST {kind, action_type, target, about, text, button, t_ms}  a result was shown
## STEP {step, do}                                               a step starts
## STEP_DONE {step, after_ms, panels, people}                    after a choice was answered
## ```
##
## Steps (a JSON array):
## ```text
## {"open": "<entity id>" | "self"}               click the drawn figure; the menu must be about it
## {"choose": {selector}, "input": "<text>"}      selector: action_type, about, target, walk: true
## {"await": {"offered": {action_type, target}}}  until the observation offers such an affordance
## {"await": {"component": c, "of": id|"self", "present": bool}}
## {"await": {"seated": true}}                    until the connection is seated and observed again
## {"mark": "<name>"} / {"after": "<name>"}       cross-client order, through files in --sync=<dir>
## {"sleep": seconds} / {"report": true}
## ```
## Each wait is bounded (STEP_LIMIT_S); a step that cannot complete fails the drive by name.

## How long one step may wait for the world (RK-b6).
const STEP_LIMIT_S := 90.0
## After a choice is answered: how long to wait for the panels to follow (AC-I2).
const FOLLOW_S := 2.0

var drive: Node
var app: Node
var _requests: Array = []
var _observations := 0
var _seated_again := false
var _panels: Array = []


func _ready() -> void:
	app.menu.listed.connect(_on_listed)
	app.panels.changed.connect(func(panels: Array) -> void:
		_panels = panels
		print("PANELS ", JSON.stringify({"seq": app.link.client.sequence, "panels": _plain_panels(panels),
			"t_ms": Time.get_ticks_msec()})))
	app.toasts.toasted.connect(func(record: Dictionary) -> void:
		var shown := record.duplicate()
		shown["t_ms"] = Time.get_ticks_msec()
		print("TOAST ", JSON.stringify(shown)))
	app.link.submitted.connect(func(token: String, request: Dictionary) -> void:
		_requests.append({"token": token, "flavour": "2d", "request": request}))
	app.link.observed.connect(func(_o: MineWorldObservation) -> void:
		_observations += 1
		_seated_again = true)
	app.link.state_changed.connect(func(state: String, _reason: String) -> void:
		if state != "seated":
			_seated_again = false)


## A menu was drawn: what it lists, the raw offers it was built from, and whether the one matches the
## other — every entry but "walk to" is the frame's affordance at the same position (AC-I3).
func _on_listed(subject: String, entries: Array, offered: Array) -> void:
	var plain := entries.map(func(e: Dictionary) -> Dictionary:
		var copy := e.duplicate()
		copy.erase("affordance")
		return copy)
	var self_menu: bool = app.latest != null and subject == app.latest.observer()
	print("MENU ", JSON.stringify({"subject": subject, "self": self_menu, "entries": plain,
		"offered": offered, "t_ms": Time.get_ticks_msec()}))
	var listed := entries.filter(func(e: Dictionary) -> bool: return e["kind"] != "walk")
	var same := listed.size() == offered.size()
	for i in mini(listed.size(), offered.size()):
		same = same and JSON.stringify(listed[i]["affordance"]) == JSON.stringify(offered[i])
	drive._check(same, "menu lists the frame's offers", "%s: %d entries, %d offers" % [subject, listed.size(), offered.size()])


## `--drive=panels`: hold for `--hold` seconds (default 3) and report the panels and the people.
func panels() -> void:
	await drive._seconds(float(app.options.get("hold", "3")))
	print("PANELS ", JSON.stringify({"seq": app.link.client.sequence, "panels": _plain_panels(app.panels.current),
		"t_ms": Time.get_ticks_msec(), "final": true}))
	drive._report_shown()


## `--drive=steps --steps=<file>`.
func steps(path: String) -> void:
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(path))
	if typeof(parsed) != TYPE_ARRAY:
		drive._check(false, "steps", "%s is not a JSON array of steps" % path)
		return
	for i in parsed.size():
		var step: Dictionary = parsed[i]
		print("STEP ", JSON.stringify({"step": i, "do": step}))
		var failure: String = await _step(i, step)
		if failure != "":
			drive._check(false, "step %d" % i, failure)
			break
	_write_requests()
	drive._report_shown()


## One step; "" when it completed, else why not.
func _step(i: int, step: Dictionary) -> String:
	if step.has("open"):
		return await _open(String(step["open"]))
	if step.has("choose"):
		return await _choose(i, step["choose"], step.get("input"))
	if step.has("await"):
		return await _await(step["await"])
	if step.has("mark"):
		var file := FileAccess.open(_sync(String(step["mark"])), FileAccess.WRITE)
		if file == null:
			return "cannot write the mark %s" % step["mark"]
		file.store_string("1")
		file.close()
		return ""
	if step.has("after"):
		var mark := _sync(String(step["after"]))
		if not await drive._until(func() -> bool: return FileAccess.file_exists(mark), STEP_LIMIT_S):
			return "the mark %s never appeared" % step["after"]
		return ""
	if step.has("sleep"):
		await drive._seconds(float(step["sleep"]))
		return ""
	if step.has("report"):
		drive._report_shown()
		return ""
	return "unknown step %s" % JSON.stringify(step)


## Clicks the subject's drawn figure, as a player does; the menu that opens must be about it.
func _open(who: String) -> String:
	var id: String = app.latest.observer() if who == "self" else who
	if not await drive._until(func() -> bool: return app.people.figure_at(id) != null, STEP_LIMIT_S):
		return "%s is never drawn" % id
	await drive._seconds(0.3)
	app.menu.close()
	var at: Vector2 = app.people.figure_at(id)
	var screen: Vector2 = app.get_viewport().get_canvas_transform() * at
	var press := InputEventMouseButton.new()
	press.button_index = MOUSE_BUTTON_LEFT
	press.pressed = true
	press.position = screen
	press.global_position = screen
	app.get_viewport().push_input(press, true)
	if not await drive._until(func() -> bool: return app.menu.subject != "", 5.0):
		return "a click on %s opened no menu" % id
	if app.menu.subject != id:
		return "a click on %s opened the menu of %s" % [id, app.menu.subject]
	return ""


## Resolves a selector against the rendered entries, presses that entry, supplies its input, and
## waits until the world has answered what it sent.
func _choose(i: int, selector: Dictionary, input: Variant) -> String:
	if not app.menu.is_open():
		return "no menu is open"
	var index := -1
	for entry in app.menu.entries:
		if _matches(entry, selector):
			index = entry["index"]
			break
	if index < 0:
		return "no rendered entry matches %s" % JSON.stringify(selector)
	var kind: String = app.menu.entries[index]["kind"]
	var sent_before: int = _requests.size()
	app.menu.choose(index)
	if input != null:
		if not await drive._until(func() -> bool: return app.talk_line.is_open(), 5.0):
			return "choosing %s asked for no input" % JSON.stringify(selector)
		app.talk_line.type_and_send(String(input))
	await drive._seconds(0.2)
	var answered := func() -> bool:
		return app.intents.pending.is_empty() and not app.walker.is_walking()
	if not await drive._until(answered, STEP_LIMIT_S):
		return "the world did not answer within %d s" % STEP_LIMIT_S
	var t0 := Time.get_ticks_msec()
	if _requests.size() > sent_before or kind == "walk":
		var seen := _observations
		await drive._until(func() -> bool: return _observations >= seen + 2, FOLLOW_S)
	print("STEP_DONE ", JSON.stringify({"step": i, "after_ms": Time.get_ticks_msec() - t0,
		"sent": _requests.size() - sent_before, "panels": _plain_panels(app.panels.current),
		"people": _people()}))
	return ""


func _matches(entry: Dictionary, selector: Dictionary) -> bool:
	if selector.get("walk", false):
		return entry["kind"] == "walk"
	if entry["kind"] == "walk":
		return false
	if selector.has("action_type") and entry["action_type"] != selector["action_type"]:
		return false
	if selector.has("about") and entry["about"] != String(selector["about"]):
		return false
	if selector.has("target") and entry["target"] != selector["target"]:
		return false
	return true


func _await(condition: Dictionary) -> String:
	var check: Callable
	if condition.has("offered"):
		var want: Dictionary = condition["offered"]
		check = func() -> bool:
			return app.latest != null and not app.latest.affordances(String(want.get("action_type", "")),
				want.get("target")).is_empty()
	elif condition.has("component"):
		var of := String(condition.get("of", "self"))
		var present: bool = condition.get("present", true)
		check = func() -> bool:
			if app.latest == null:
				return false
			var id: String = app.latest.observer() if of == "self" else of
			return (app.latest.component_value(id, String(condition["component"])) != null) == present
	elif condition.has("seated"):
		check = func() -> bool: return app.link.state == "seated" and _seated_again
	else:
		return "unknown condition %s" % JSON.stringify(condition)
	if not await drive._until(check, STEP_LIMIT_S):
		return "never true within %d s: %s" % [STEP_LIMIT_S, JSON.stringify(condition)]
	return ""


func _sync(name: String) -> String:
	return String(app.options.get("sync", OS.get_user_data_dir())).path_join(name)


## `--requests=<file>`: every request sent, in demo.gd's format (`[{token, flavour, request}]`).
func _write_requests() -> void:
	var path := String(app.options.get("requests", ""))
	if path == "":
		return
	var file := FileAccess.open(path, FileAccess.WRITE)
	if file == null:
		drive._check(false, "requests file", "cannot write %s" % path)
		return
	file.store_string(JSON.stringify(_requests, "  "))
	file.close()


func _people() -> Array:
	var out: Array = []
	for id in app.people.drawn_ids():
		out.append({"id": id, "label": app.people.label_of(id), "activity": app.people.activity_of(id)})
	return out


static func _plain_panels(panels: Array) -> Array:
	return panels.map(func(p: Dictionary) -> Dictionary:
		return {"component": p["component"], "title": p["title"], "rows": Array(p["rows"]),
			"payload": p["payload"], "known": p["known"]})
