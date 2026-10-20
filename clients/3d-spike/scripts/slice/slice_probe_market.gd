## The slice's evidence harness for buying, handing over, eating and drinking (step-15 §22, PR 16d):
## the connected slice against a real `mineworld server`, played on the real controller and the real
## keys, never teleported (`ENGINEERING_RULES.md` §19).
##
##   --slice-market --steps=<file> [--requests=<file>] [--frames]
##
## The steps are 13b's (`clients/2d/scripts/harness/interact.gd`), written by the Rust test
## (`tools/cli/tests/client_3d_market.rs`); this file names no action type, no item and no person.
## Each selector is resolved against the entries the menu actually built, by its data (action type,
## the thing it is about, its target), never by its words, and the entry is pressed through
## `SliceOfferMenu.choose`, the path a click and Enter take.
##
## ```text
## {"open": "self"}                    press B; the menu must be yours
## {"open": "<entity id>"}             aim at the person's head (stepping aside if someone is in the
##                                     way), press Tab; the menu must be theirs
## {"choose": {selector}}              action_type, about, target -- or {"walk": true}: walk the body
##                                     straight toward the menu's person until 1.2 m short (13b's
##                                     APPROACH_M, a destination, not a range; D-16d-11)
## {"gate": true}                      O-4: with a menu open, W and the mouse do nothing; Esc closes the
##                                     menu, and only the next Esc opens the settings menu
## {"press_disabled": true}            O-6: press every entry the menu lists disabled
## {"sleep": seconds} / {"report": true} / {"shoot": "<name>"} (a frame under --frames)
## ```
##
## Output, one line each: `MENU`, `PANELS`, `REQUEST`, `RESULT`, `TOAST` and `STEP_DONE` in 13b's
## keys, with the raw disclosure each was built from beside it (the oracle is the frame or the save,
## never the report alone); `SCREEN` -- every text on screen and every id this session has seen, for
## the no-id check (§22.4 N-2); `EVIDENCE`; `[FAIL]` and `market complete: PASS|FAIL`.
class_name SliceProbeMarket
extends SliceProbeWorld

const MARKET_MODES := ["market"]
## How long one step may wait for the world (13b's STEP_LIMIT_S).
const STEP_LIMIT_S := 90.0
## After a choice is answered: how long the panels may take to follow (§22.4 K-2).
const FOLLOW_S := 2.0
## How far short of a person the walk step stops: 13b's APPROACH_M, a destination, not a range.
const APPROACH_M := 1.2
const SHOTS := "res://shots/16d"
## The whole run's bound, in wall seconds.
const RUN_LIMIT_S := 420

var _requests: Array = []
var _observations := 0
var _failed := false
## Every id this session has seen: perceived entities, catalogued kinds, payload references (N-2).
var _seen := {}


static func requested() -> bool:
	return requested_of(MARKET_MODES) != ""


func _modes() -> Array:
	return MARKET_MODES


func _run(mode: String) -> void:
	if mode == "market":
		await _market()
	else:
		await super(mode)


func _market() -> void:
	print("== step-15 §22 -- buying, handing over, eating and drinking, in 3D ==\n")
	# A run that cannot finish says so and ends, rather than holding a Godot slot.
	get_tree().create_timer(RUN_LIMIT_S).timeout.connect(func() -> void:
		print("[FAIL] the run took longer than %d s" % RUN_LIMIT_S)
		print("market complete: FAIL")
		get_tree().quit(1))
	var link := slice.link
	var menu := slice.offer_menu
	if link == null or menu == null:
		print("[FAIL] no --server= given")
		print("market complete: FAIL")
		return
	var waited := 0.0
	while waited < 20.0 and not link.has_first_view():
		await get_tree().process_frame
		waited += get_process_delta_time()
	await _hold(0.8)
	_wire(link, menu)
	var path := ""
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--steps="):
			path = a.substr(8)
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(path)) if path != "" else null
	if typeof(parsed) != TYPE_ARRAY:
		_fail("steps", "%s is not a JSON array of steps" % path)
	else:
		for i in parsed.size():
			var step: Dictionary = parsed[i]
			print("STEP ", JSON.stringify({"step": i, "do": step}))
			var failure: String = await _step(i, step, link, menu)
			_screen(i, link)
			if failure != "":
				_fail("step %d" % i, failure)
				break
	_write_requests()
	print("market complete: %s" % ("FAIL" if _failed else "PASS"))


func _fail(what: String, why: String) -> void:
	_failed = true
	print("[FAIL] %s  %s" % [what, why])


## The reports, each with the raw disclosure it was built from.
func _wire(link: SliceLink, menu: SliceOfferMenu) -> void:
	menu.listed.connect(func(subject: String, entries: Array, offered: Array) -> void:
		var obs := link.client.latest
		print("MENU ", JSON.stringify({"subject": subject, "self": subject == link.client.observer,
			"entries": entries.map(func(e: Dictionary) -> Dictionary:
				var copy := e.duplicate()
				copy.erase("affordance")
				return copy),
			"offered": offered, "disclosed": _disclosed(obs), "t_ms": Time.get_ticks_msec()})))
	menu.shown_panels.connect(func(panels: Array) -> void:
		print("PANELS ", JSON.stringify({"seq": link.client.sequence, "panels": _plain(panels),
			"disclosed": _disclosed(link.client.latest), "t_ms": Time.get_ticks_msec()})))
	link.client.submitted_request.connect(func(token: String, request: Dictionary) -> void:
		var entry := {"token": token, "flavour": "3d", "request": request}
		_requests.append(entry)
		print("REQUEST ", JSON.stringify(entry)))
	link.client.resolved.connect(func(token: String, _id: String, result: Dictionary) -> void:
		print("RESULT ", JSON.stringify({"token": token, "result": result})))
	link.client.refused.connect(func(code: String, token: String, _detail: String) -> void:
		print("RESULT ", JSON.stringify({"token": token, "refused": code})))
	link.said.connect(func(key: String, args: Dictionary, notice: bool) -> void:
		if notice:
			print("TOAST ", JSON.stringify({"key": key, "text": MineWorldText.text(key, args),
				"t_ms": Time.get_ticks_msec()})))
	link.client.observed.connect(func(obs: MineWorldObservation) -> void:
		_observations += 1
		_learn_ids(obs))
	_learn_ids(link.client.latest)


## What the frame disclosed that a report's words come from, raw: the place's item catalogue, every
## perceived entity's disclosed name, and the observer's place and clock.
func _disclosed(obs: MineWorldObservation) -> Dictionary:
	if obs == null:
		return {}
	var names := {}
	for id in obs.ids():
		var named: Variant = obs.component_value(id, "display-name")
		if named != null:
			names[id] = named
	return {"place": obs.place(), "at": obs.at(),
		"item_catalogue": obs.component_value(obs.place(), SliceThings.CATALOGUE), "display_names": names}


func _learn_ids(obs: MineWorldObservation) -> void:
	if obs == null:
		return
	for id in obs.ids():
		_seen[id] = true
	var catalogue: Variant = obs.component_value(obs.place(), SliceThings.CATALOGUE)
	if typeof(catalogue) == TYPE_DICTIONARY and typeof(catalogue.get("kinds")) == TYPE_ARRAY:
		for kind in catalogue["kinds"]:
			if typeof(kind) == TYPE_DICTIONARY:
				_seen[SliceThings.ref(kind.get("item"))] = true
	for offered in obs.affordances():
		var about := SliceOfferMenu.about_of(offered)
		if about != "":
			_seen[about] = true
		if offered.get("target") != null:
			_seen[String(offered["target"])] = true


## One step; "" when it completed, else why not.
func _step(i: int, step: Dictionary, link: SliceLink, menu: SliceOfferMenu) -> String:
	if step.has("open"):
		return await _open(String(step["open"]), link, menu)
	if step.has("choose"):
		return await _choose(i, step["choose"], link, menu)
	if step.has("gate"):
		return await _gate(link, menu)
	if step.has("press_disabled"):
		return await _press_disabled(menu)
	if step.has("sleep"):
		await _hold(float(step["sleep"]))
		return ""
	if step.has("report"):
		_print_shown(link)
		return ""
	if step.has("shoot"):
		if "--frames" in OS.get_cmdline_user_args():
			await _hold(0.6)
			await _settle(4)
			DirAccess.make_dir_recursive_absolute(SHOTS)
			var file := "%s/%s.png" % [SHOTS, String(step["shoot"])]
			get_viewport().get_texture().get_image().save_png(file)
			print("frame    %s" % ProjectSettings.globalize_path(file))
		return ""
	return "unknown step %s" % JSON.stringify(step)


## A key press and its release, through the engine's input, as a keyboard sends them.
func _press(keycode: Key) -> void:
	for pressed in [true, false]:
		var key := InputEventKey.new()
		key.keycode = keycode
		key.physical_keycode = keycode
		key.pressed = pressed
		Input.parse_input_event(key)
		await get_tree().process_frame


func _open(who: String, link: SliceLink, menu: SliceOfferMenu) -> String:
	menu.close()
	await _frames(2)
	if who == "self":
		await _press(KEY_B)
		if not await _until(func() -> bool: return menu.subject == link.client.observer, 5.0):
			return "B opened no menu of yours"
		return ""
	if not await _until(func() -> bool: return link.figures.has(who), STEP_LIMIT_S):
		return "%s is never drawn" % who
	if not await _clear_aim(link, who):
		return "the ray never meets %s (it meets %s)" % [who, link.facing_person()]
	_report_aim(link, "open %s" % who)
	await _press(KEY_TAB)
	if not await _until(func() -> bool: return menu.subject != "", 5.0):
		return "Tab on %s opened no menu" % who
	if menu.subject != who:
		return "Tab on %s opened the menu of %s" % [who, menu.subject]
	return ""


## Resolves a selector against the built entries by their data, presses that entry, and waits until
## the world has answered what it sent and the panels have had time to follow.
func _choose(i: int, selector: Dictionary, link: SliceLink, menu: SliceOfferMenu) -> String:
	if not menu.is_open():
		return "no menu is open"
	var sent_before := _requests.size()
	var t0 := Time.get_ticks_msec()
	if selector.get("walk", false):
		var who := menu.subject
		menu.close()
		await _approach(link, who)
	else:
		var index := -1
		for entry in menu.entries:
			if _matches(entry, selector):
				index = entry["index"]
				break
		if index < 0:
			return "no built entry matches %s" % JSON.stringify(selector)
		var token := menu.choose(index)
		if token != "" and not await _until(func() -> bool: return _answered(link, token), STEP_LIMIT_S):
			return "the world did not answer %s within %d s" % [token, STEP_LIMIT_S]
		t0 = Time.get_ticks_msec()
		if _requests.size() > sent_before:
			var seen := _observations
			await _until(func() -> bool: return _observations >= seen + 2, FOLLOW_S)
	print("STEP_DONE ", JSON.stringify({"step": i, "after_ms": Time.get_ticks_msec() - t0,
		"sent": _requests.size() - sent_before, "panels": _plain(link.things.panels(link.client.latest)),
		"disclosed": _disclosed(link.client.latest)}))
	return ""


static func _matches(entry: Dictionary, selector: Dictionary) -> bool:
	if selector.has("action_type") and entry["action_type"] != selector["action_type"]:
		return false
	if selector.has("about") and entry["about"] != String(selector["about"]):
		return false
	if selector.has("target") and entry["target"] != selector["target"]:
		return false
	return true


## Walks the body straight toward `who` until APPROACH_M short, on the real controller; the server
## answers every stride. Reports where the body stopped against where the world put the person.
func _approach(link: SliceLink, who: String) -> void:
	if not link.figures.has(who):
		print("walk     %s is not drawn; nothing to walk to" % who)
		return
	var at: Vector3 = (link.figures[who] as Node3D).global_position
	var from := player.global_position
	var flat := Vector3(at.x - from.x, 0.0, at.z - from.z)
	var stop := at - flat.normalized() * APPROACH_M
	await _walk_to(Vector3(stop.x, from.y, stop.z), 12.0)
	await _hold(1.0)
	var left := Vector2(at.x - player.global_position.x, at.z - player.global_position.z).length()
	print("walk     to %s (%s): stopped %.2f m from the figure, from %s to %s"
		% [link.display_label(who), who, left, from, player.global_position])


## O-4 (§22.4): with a menu open, two seconds of W and 200 px of mouse motion ask for nothing and move
## nothing; Esc closes the menu and leaves the settings menu closed; Esc again opens it.
func _gate(link: SliceLink, menu: SliceOfferMenu) -> String:
	if not menu.is_open():
		return "the gate step needs an open menu"
	var settings := slice.settings_menu
	var asked := _requests.size()
	var at := player.global_position
	var yaw := player.rotation.y
	player.ignore_mouse_look = false
	Input.action_press("move_forward")
	for k in 20:
		var motion := InputEventMouseMotion.new()
		motion.relative = Vector2(10, 0)
		Input.parse_input_event(motion)
		await _hold(0.1)
	Input.action_release("move_forward")
	await _hold(0.3)
	player.ignore_mouse_look = true
	var moved := at.distance_to(player.global_position)
	var turned := rad_to_deg(absf(angle_difference(yaw, player.rotation.y)))
	await _press(KEY_ESCAPE)
	var first := {"menu_open": menu.is_open(), "settings_open": settings.is_open()}
	await _press(KEY_ESCAPE)
	var second := {"settings_open": settings.is_open()}
	settings.close()
	await _frames(2)
	print("EVIDENCE ", JSON.stringify({"gate": {"requests": _requests.size() - asked, "moved_m": moved,
		"yaw_deg": turned, "after_first_esc": first, "after_second_esc": second}}))
	return ""


## O-6 (§22.4): every entry the menu lists disabled, pressed; the requests are counted by the test.
func _press_disabled(menu: SliceOfferMenu) -> String:
	if not menu.is_open():
		return "no menu is open"
	var before := _requests.size()
	var pressed := 0
	var subject := menu.subject
	for entry in menu.entries.duplicate():
		if not entry["enabled"]:
			pressed += 1
			menu.choose(entry["index"])
	await _hold(1.5)
	print("EVIDENCE ", JSON.stringify({"press_disabled": {"subject": subject, "pressed": pressed,
		"requests": _requests.size() - before}}))
	return ""


## Every text the client puts on screen -- HUD lines, menus, panels, toasts, the figures' labels --
## and every id seen. The scene's own signage is the drawn town, not the client speaking (§22.4 N-2).
func _screen(i: int, link: SliceLink) -> void:
	var texts: Array = []
	for root in [slice.hud, slice.offer_menu] + link.figures.values():
		if root != null:
			_texts(root, texts)
	print("SCREEN ", JSON.stringify({"step": i, "texts": texts, "seen": _seen.keys(),
		"disclosed": _disclosed(link.client.latest),
		"own": _own(link.client.latest)}))


func _own(obs: MineWorldObservation) -> Dictionary:
	if obs == null:
		return {}
	return {"wallet": obs.component_value(obs.observer(), "wallet"),
		"holdings": obs.component_value(obs.observer(), "holdings"),
		"shop": obs.component_value(obs.place(), "shop"),
		"offered": obs.affordances()}


func _texts(node: Node, out: Array) -> void:
	if node is CanvasItem and not (node as CanvasItem).is_visible_in_tree():
		return
	if node is CanvasLayer and not (node as CanvasLayer).visible:
		return
	if node is CanvasItem and (node as CanvasItem).modulate.a <= 0.01:
		return
	var said := ""
	if node is Button:
		said = (node as Button).text
	elif node is Label:
		said = (node as Label).text
	elif node is Label3D and (node as Label3D).visible:
		said = (node as Label3D).text
	if said.strip_edges() != "":
		out.append({"node": str(node.get_path()), "text": said})
	for child in node.get_children():
		_texts(child, out)


static func _plain(panels: Array) -> Array:
	return panels.map(func(p: Dictionary) -> Dictionary:
		return {"component": p["component"], "title": p["title"], "rows": Array(p["rows"]),
			"payload": p["payload"]})


## `n` process frames. Headless, `_settle`'s post-draw signal never comes; frames do.
func _frames(n: int) -> void:
	for k in n:
		await get_tree().process_frame


func _until(ready: Callable, limit: float) -> bool:
	var t := 0.0
	while t < limit:
		if ready.call():
			return true
		await get_tree().process_frame
		t += get_process_delta_time()
	return ready.call()


## `--requests=<file>`: every request sent, in demo.gd's format (`[{token, flavour, request}]`).
func _write_requests() -> void:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--requests="):
			var file := FileAccess.open(a.substr(11), FileAccess.WRITE)
			if file != null:
				file.store_string(JSON.stringify(_requests, "  "))
				file.close()
