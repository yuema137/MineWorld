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
## MENU {subject, self, entries, offered, glance, inspect, t_ms} a menu opened or its entries changed
## PANELS {seq, panels: [...], t_ms}                             the panels changed
## TOAST {kind, action_type, target, about, text, button, t_ms}  a result was shown
## STEP {step, do}                                               a step starts
## STEP_DONE {step, after_ms, panels, people}                    after a choice was answered
## ```
##
## Steps (a JSON array):
## ```text
## {"open": "<entity id>" | "self"}               click the drawn figure; the menu must be about it
## {"open": {"offered": "<type>", "except": [ids], "among": [ids]}}  the first drawn person (not me,
##                                                none of `except`, one of `among` when given) the newest
##                                                observation offers that type against, waiting for one
##                                                (13f: a test chooses from disclosure)
## {"open": "last"}                               the person the latest `offered` open chose
## {"choose": {selector}, "input": "<text>"}      selector: action_type, about, target, walk: true, aim: true
## {"open_object": "<object id>" | "nearest"}     click the drawn object; the menu must be about it (13d)
## {"click_floor": {"x": mm, "y": mm}}            a left click on the floor of the observer's place
## {"key": "esc"}                                 a key press, through the viewport
## {"objects": {"count": n, "motions": m, "moved": true}}  until n drawn, m motions drawn (or, with
##                                                moved, one ended since the last choice), none moving
## body steps (walk_near, hold_into_solid, walk_into, self)  harness/bodies.gd
## {"await": {"offered": {action_type, target}}}  until the observation offers such an affordance
## {"await": {"component": c, "of": id|"self", "present": bool}}
## {"await": {"seated": true}}                    until the connection is seated and observed again
## {"mark": "<name>"} / {"after": "<name>"}       cross-client order, through files in --sync=<dir>
## {"sleep": seconds} / {"report": true}
## {"panels": "things" | "conversations"}         toggle a panel group, as I / H do
## {"shoot": "<name>"}                            a still under --capture (windowed), else nothing
## {"trade": "open", "of": "<id>" | "self", "absent": bool}   open the menu; the trade panel must (not) show
## {"trade": "shelf"}                             press "This shop's prices…"
## {"trade": "press", "row": {kind, item}, "button": n, "amount": "<typed>", "item": "<id>"}
##                                                press a row's button through the panel's one path (S31-e)
## {"board": "open"}                              press N
## {"hud": true}                                  report the money line, the badge, the tags and the boards
## ```
##
## S31-e adds, from the trade module's rows (`clients/shared/TRADE.md` §7):
## ```text
## TRADE {subject, rows, t_ms}                    the trade panel's rows changed
## BOARD {notices: [{id, topic, publisher, lines}], badge, t_ms}
## HUD {money, badge, tags: [{subject, text}], boards: [{place, lines, fresh}], t_ms}
## FIELD {prompt, t_ms}                           a press asked for typed input that was not an amount
## ```
## Each wait is bounded (STEP_LIMIT_S); a step that cannot complete fails the drive by name.

## How long one step may wait for the world (RK-b6).
const STEP_LIMIT_S := 90.0
## After a choice is answered: how long to wait for the panels to follow (AC-I2).
const FOLLOW_S := 2.0

const Bodies := preload("res://scripts/harness/bodies.gd")

var drive: Node
var app: Node
var _body_steps: Node = null
## How many object motions were drawn when the latest choice or floor click was made.
var _motions_mark := 0
var _requests: Array = []
var _observations := 0
var _seated_again := false
var _panels: Array = []
## The person the latest `{"open": {"offered": …}}` step chose (`{"open": "last"}`).
var _last_offered := ""


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
	app.trade_panel.shown.connect(func(subject: String, rows: Array) -> void:
		print("TRADE ", JSON.stringify({"subject": subject, "rows": MineWorldTrade.report(rows),
			"t_ms": Time.get_ticks_msec()})))
	app.board_panel.shown.connect(func(rows: Array, badge: int) -> void:
		print("BOARD ", JSON.stringify({"notices": rows.map(func(r: Dictionary) -> Dictionary:
			return {"id": r["id"], "topic": r["topic"], "publisher": r["publisher"], "lines": Array(r["lines"])}),
			"badge": badge, "t_ms": Time.get_ticks_msec()})))


## A menu was drawn: what it lists, the raw offers it was built from, and whether the one matches the
## other — every entry but "walk to" is the frame's affordance at the same position (AC-I3).
func _on_listed(subject: String, entries: Array, offered: Array) -> void:
	var plain := entries.map(func(e: Dictionary) -> Dictionary:
		var copy := e.duplicate()
		copy.erase("affordance")
		return copy)
	var self_menu: bool = app.latest != null and subject == app.latest.observer()
	print("MENU ", JSON.stringify({"subject": subject, "self": self_menu, "entries": plain,
		"offered": offered, "glance": app.menu.glance, "inspect": app.menu.inspect,
		"t_ms": Time.get_ticks_msec()}))
	var listed := entries.filter(func(e: Dictionary) -> bool: return e["kind"] != "walk" and e["kind"] != "aim")
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
		if typeof(step["open"]) == TYPE_DICTIONARY:
			return await _open_offered(step["open"])
		return await _open(_last_offered if String(step["open"]) == "last" else String(step["open"]))
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
	if step.has("objects"):
		return await _objects(step["objects"])
	if step.has("open_object"):
		return await _open_object(String(step["open_object"]))
	if step.has("click_floor"):
		return await _click_floor(step["click_floor"])
	if step.has("key"):
		var press := InputEventKey.new()
		press.keycode = KEY_ESCAPE if String(step["key"]) == "esc" else OS.find_keycode_from_string(String(step["key"]))
		press.pressed = true
		app.get_viewport().push_input(press)
		await drive._seconds(0.2)
		return ""
	if step.has("panels"):
		app.panels.toggle(String(step["panels"]))
		return ""
	if step.has("trade"):
		return await _trade(i, step)
	if step.has("board"):
		app.board_panel.toggle()
		app.status.show_state(app.latest, app.link, app.link.client.revision)
		await drive._seconds(0.3)
		return ""
	if step.has("hud"):
		print("HUD ", JSON.stringify({"money": app.status.money_text, "badge": app.status.badge_text,
			"tags": app.people.tags(), "boards": app.places.board_report(), "t_ms": Time.get_ticks_msec()}))
		return ""
	if step.has("shoot"):
		# A still, with the menu or panels as they stand (`--capture`, windowed); ignored headless.
		if drive._capture != null:
			await drive._seconds(0.6)
			await drive._capture.shoot(String(step["shoot"]))
		return ""
	var body: Variant = await _bodies().step(step)
	if body != null:
		return body
	return "unknown step %s" % JSON.stringify(step)


## The body steps (`harness/bodies.gd`), made on first use.
func _bodies() -> Node:
	if _body_steps == null:
		_body_steps = Bodies.new()
		_body_steps.drive = drive
		_body_steps.app = app
		add_child(_body_steps)
	return _body_steps


## Clicks the subject's drawn figure, as a player does; the menu that opens must be about it.
func _open(who: String) -> String:
	var id: String = app.latest.observer() if who == "self" else who
	if not await drive._until(func() -> bool: return app.people.figure_at(id) != null, STEP_LIMIT_S):
		return "%s is never drawn" % id
	await drive._seconds(0.3)
	app.menu.close()
	var at: Vector2 = _visible_point(id)
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


## The trade steps (S31-e §5.12): the panel opens with the menu; a press goes through `trade_panel.press`, the
## path a click takes, with the typed text and the picked kind a player would supply. Rows are selected by
## kind and item id, never by their words.
func _trade(i: int, step: Dictionary) -> String:
	match String(step["trade"]):
		"open":
			var who := String(step.get("of", "self"))
			if who == "self":
				app.menu.close()
				app.open_menu(app.latest.observer())
			else:
				var failed: String = await _open(who)
				if failed != "":
					return failed
			var absent: bool = step.get("absent", false)
			var shows := func() -> bool: return app.trade_panel.is_open()
			if absent:
				await drive._seconds(1.0)
				return "the trade panel opened" if app.trade_panel.is_open() else ""
			if not await drive._until(shows, 5.0):
				return "the trade panel did not open beside the menu of %s" % who
			return ""
		"shelf":
			app.trade_panel.toggle_shelf()
			return ""
		"press":
			return await _press(i, step)
	return "unknown trade step %s" % JSON.stringify(step)


func _press(i: int, step: Dictionary) -> String:
	var want: Dictionary = step.get("row", {})
	var r := -1
	for at in app.trade_panel.rows.size():
		var row: Dictionary = app.trade_panel.rows[at]
		if row["kind"] == String(want.get("kind", row["kind"])) and row["item"] == String(want.get("item", row["item"])):
			r = at
			break
	if r < 0:
		return "no trade row matches %s" % JSON.stringify(want)
	var given := {}
	if step.has("amount"):
		given["amount"] = String(step["amount"])
	if step.has("item"):
		for kind in MineWorldTrade.catalogue_of(app.latest):
			if kind["item"] == String(step["item"]):
				given["item"] = kind["ref"]
	var sent_before: int = _requests.size()
	app.trade_panel.press(r, int(step.get("button", 0)), given)
	await drive._seconds(0.2)
	if app.talk_line.is_open():
		# The text was not an amount: the field asks again and nothing was sent (S31-e AC-e-5).
		print("FIELD ", JSON.stringify({"prompt": app.talk_line._prompt.text, "t_ms": Time.get_ticks_msec()}))
		app.talk_line.cancel()
	if not await drive._until(func() -> bool: return app.intents.pending.is_empty(), STEP_LIMIT_S):
		return "the world did not answer within %d s" % STEP_LIMIT_S
	if _requests.size() > sent_before:
		var seen := _observations
		await drive._until(func() -> bool: return _observations >= seen + 2, FOLLOW_S)
	print("STEP_DONE ", JSON.stringify({"step": i, "sent": _requests.size() - sent_before,
		"panels": _plain_panels(app.panels.current), "people": _people()}))
	return ""


## Opens the first drawn person the newest observation offers `want.offered` against — not the observer,
## none of `want.except` — in drawing order, waiting up to STEP_LIMIT_S for one; prints `CHOSE`.
func _open_offered(want: Dictionary) -> String:
	var type := String(want.get("offered", ""))
	var skip: Array = want.get("except", [])
	var among: Array = want.get("among", [])
	var found := func() -> String:
		if app.latest == null:
			return ""
		for id in app.people.drawn_ids():
			if id != app.latest.observer() and not skip.has(id) and (among.is_empty() or among.has(id)) \
					and not app.latest.affordances(type, id).is_empty():
				return id
		return ""
	if not await drive._until(func() -> bool: return found.call() != "", STEP_LIMIT_S):
		return "nobody drawn is offered %s within %d s" % [type, STEP_LIMIT_S]
	_last_offered = found.call()
	print("CHOSE ", JSON.stringify({"offered": type, "subject": _last_offered}))
	return await _open(_last_offered)


## A point of `id`'s drawn figure that no figure in front of it covers — where a player would click
## when people stand close — else the figure's middle.
func _visible_point(id: String) -> Vector2:
	var middle: Vector2 = app.people.figure_at(id)
	for dy in [0.0, -10.0, -18.0, 8.0, 14.0]:
		for dx in [0.0, -11.0, 11.0, -14.0, 14.0]:
			var at: Vector2 = middle + Vector2(dx, dy)
			if app.people.pick(at) == id:
				return at
	return middle


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
	_motions_mark = app.objects.motions.size()
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
	if selector.get("aim", false) != (entry["kind"] == "aim"):
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


## Clicks a drawn object's centre, as a player does (`"nearest"`, `"ball"`, `"box"`: the drawn object of
## that shape nearest the player); the menu that opens must be about it.
func _open_object(which: String) -> String:
	var id := which
	if which in ["nearest", "ball", "box"]:
		id = _bodies().nearest(which)
	if not await drive._until(func() -> bool: return app.objects.drawn_ids().has(id), STEP_LIMIT_S):
		return "the object %s is never drawn" % id
	await drive._seconds(0.3)
	app.menu.close()
	_push_click(app.objects.drawn_screen(id))
	if not await drive._until(func() -> bool: return app.menu.subject != "", 5.0):
		return "a click on the object %s opened no menu" % id
	if app.menu.subject != id:
		return "a click on the object %s opened the menu of %s" % [id, app.menu.subject]
	return ""


## A left click on the floor at `where` (`{x, y}` mm of the observer's place frame, or `"centre"`: the
## disclosed floor's), then waits until whatever it sent was answered.
func _click_floor(where: Variant) -> String:
	var place: String = app.latest.place()
	var local: Dictionary = _bodies().floor_centre() if typeof(where) == TYPE_STRING else where
	if local.is_empty():
		return "%s discloses no floor" % place
	print("EVIDENCE ", JSON.stringify({"clicked_floor": {"place": place, "local": local}}))
	_motions_mark = app.objects.motions.size()
	var plan: Variant = app.town.to_plan(place, local)
	if plan == null:
		return "the place %s is not placed" % place
	_push_click(app.projection.to_screen(plan))
	await drive._seconds(0.2)
	if not await drive._until(func() -> bool: return app.intents.pending.is_empty(), STEP_LIMIT_S):
		return "the world did not answer within %d s" % STEP_LIMIT_S
	return ""


## A left click at a point of the world node's frame, through the viewport's input path.
func _push_click(at: Vector2) -> void:
	var screen: Vector2 = app.get_viewport().get_canvas_transform() * at
	var press := InputEventMouseButton.new()
	press.button_index = MOUSE_BUTTON_LEFT
	press.pressed = true
	press.position = screen
	press.global_position = screen
	app.get_viewport().push_input(press, true)


## Waits until `count` objects are drawn (any number when absent), at least `motions` motions were
## drawn (none when absent), and none is moving; then reports them (`EVIDENCE {"objects": …}`).
func _objects(want: Variant) -> String:
	var count: int = int(want.get("count", -1)) if typeof(want) == TYPE_DICTIONARY else -1
	var motions: int = int(want.get("motions", 0)) if typeof(want) == TYPE_DICTIONARY else 0
	if typeof(want) == TYPE_DICTIONARY and want.get("moved", false):
		# A motion drawn since the last choice or floor click, and ended.
		var mark := _motions_mark
		var ended := func() -> bool:
			var m: Array = app.objects.motions
			for i in range(mark, m.size()):
				if m[i]["ended"]:
					return true
			return false
		if not await drive._until(ended, STEP_LIMIT_S):
			return "nothing was drawn moving since step's choice (%d motions)" % app.objects.motions.size()
	var settled := func() -> bool:
		var ids: PackedStringArray = app.objects.drawn_ids()
		if count >= 0 and ids.size() != count:
			return false
		if app.objects.motions.size() < motions:
			return false
		for id in ids:
			if app.objects.moving(id):
				return false
		return true
	if not await drive._until(settled, STEP_LIMIT_S):
		return "the objects never settled at %d drawn (%d drawn)" % [count, app.objects.drawn_ids().size()]
	await drive._seconds(0.2)
	drive._report_objects()
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
