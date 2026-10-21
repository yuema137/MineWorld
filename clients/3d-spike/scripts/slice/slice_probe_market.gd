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
## {"choose": {selector}, "input": "<text>"}  (PR 16e) a composed entry: the talk line it opens is
##                                     typed into and sent with Enter (16g1's line)
## {"walk_near": "<object or person id>"}     (PR 16e) walk the body until NEAR_M short of it
## {"open_object": "<object id>"}       (PR 16e) aim the reticle at the drawn object (it must meet it),
##                                     then press B: the slice has no menu of its own for an object,
##                                     so the next choice is made among your menu's entries about
##                                     it -- the offers 2D's object menu lists (13d)
## {"gate": true}                      O-4: with a menu open, W and the mouse do nothing; Esc closes the
##                                     menu, and only the next Esc opens the settings menu
## {"press_disabled": true}            O-6: press every entry the menu lists disabled
## {"say": "<text>", "to": "<id>"}     PR 16g1: aim at the person, press E, type the text into the talk
##   [, "turn_deg": d]                 line and press Enter; with turn_deg the body turns that far
##                                     after E and before Enter (the line stays pinned, D-3DS-3)
## {"typing_gate": "<id>"}             AC-3DS-3: E opens the line; typing "wasd e" with W held walks
##                                     nowhere and sends nothing; Esc closes it sending nothing; E opens
##                                     it again; Esc
## {"sleep": seconds} / {"report": true} / {"shoot": "<name>"} (a frame under --frames)
## ```
##
## Output, one line each: `MENU`, `PANELS`, `REQUEST`, `RESULT`, `TOAST` and `STEP_DONE` in 13b's
## keys, with the raw disclosure each was built from beside it (the oracle is the frame or the save,
## never the report alone); `SCREEN` -- every text on screen and every id this session has seen, for
## the no-id check (§22.4 N-2); `EVIDENCE`; `[FAIL]` and `market complete: PASS|FAIL`. Since PR 16e
## also `IDENTITY_WORLD` (once seated, after every `report`, and at the end: `ADOPTION.md` §7), and
## the harness flag `--drop-doorway=<from>,<to>` (Q-PAR-3): that doorway is left out of the scene's
## binding; one the world never disclosed by the end of the run fails it by name.
class_name SliceProbeMarket
extends SliceProbeWorld

const MARKET_MODES := ["market"]
## How long one step may wait for the world (13b's STEP_LIMIT_S).
const STEP_LIMIT_S := 90.0
## After a choice is answered: how long the panels may take to follow (§22.4 K-2).
const FOLLOW_S := 2.0
## How far short of a person the walk step stops: 13b's APPROACH_M, a destination, not a range.
const APPROACH_M := 1.2
## How far short of an object or a person a `walk_near` stops (PR 16e): clear of the body's 250 mm
## radius and the thing's own, inside a kick's 800 mm and a shove's 1 000 mm reach.
const NEAR_M := 0.6
const SHOTS := "res://shots/16d"
## The whole run's bound, in wall seconds.
const RUN_LIMIT_S := 420

var _requests: Array = []
var _observations := 0
var _failed := false
## Every id this session has seen: perceived entities, catalogued kinds, payload references (N-2).
var _seen := {}
## The object `open_object` aimed at: the next choice is among your entries about it ("" for none).
var _object := ""
## Every fact this client learned from `observation.events`, by EventId, in arrival order (AC-15).
var _events_seen: Array = []


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
	_print_identity(link)
	var dropped := _drop_doorway(link)
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
	if dropped.size() == 2 and not link.layout.knows_doorway(dropped[0], dropped[1]):
		_fail("drop-doorway", "%s -> %s is no doorway this client learned" % [dropped[0], dropped[1]])
	_print_identity(link)
	print("market complete: %s" % ("FAIL" if _failed else "PASS"))


## `--drop-doorway=<from>,<to>` (harness only, PR 16e Q-PAR-3): that doorway is no longer bound, so the
## scene's report leaves it out. Returns [from, to], or [] without the flag.
func _drop_doorway(link: SliceLink) -> PackedStringArray:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--drop-doorway="):
			var named := a.substr(15).split(",", false)
			if named.size() != 2:
				_fail("drop-doorway", "expected <from>,<to>, got %s" % a.substr(15))
				return PackedStringArray()
			link.layout.hide_doorway(named[0], named[1])
			print("drop-doorway %s -> %s" % [named[0], named[1]])
			return named
	return PackedStringArray()


## The 3D side of `AC-15` (PR 16e §4.2, 2D's `report_identity`): the world instance, the persisted
## revision of the newest frame, every person the newest observation lists by id and disclosed name,
## and every fact learned by EventId.
func _print_identity(link: SliceLink) -> void:
	var obs := link.client.latest
	var people := {}
	if obs != null:
		for entity in obs.entities():
			if String(entity.get("entity_type", "")) == "person":
				var id := String(entity.get("id", ""))
				people[id] = obs.display_name(id)
	print("IDENTITY_WORLD ", JSON.stringify({"instance": link.instance, "revision": link.client.revision,
		"people": people, "events_seen": _events_seen}))


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
		for event in obs.events():
			if typeof(event) == TYPE_DICTIONARY and event.has("id"):
				_events_seen.append(String(event["id"]))
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
	if step.has("open_object"):
		return await _open_object(String(step["open_object"]), link, menu)
	if step.has("walk_near"):
		return await _walk_near(String(step["walk_near"]), link)
	if step.has("choose"):
		return await _choose(i, step["choose"], link, menu, String(step.get("input", "")))
	if step.has("gate"):
		return await _gate(link, menu)
	if step.has("say"):
		return await _say_step(i, step, link)
	if step.has("typing_gate"):
		return await _typing_gate(String(step["typing_gate"]), link)
	if step.has("press_disabled"):
		return await _press_disabled(menu)
	if step.has("sleep"):
		await _hold(float(step["sleep"]))
		return ""
	if step.has("report"):
		_print_shown(link)
		_print_identity(link)
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
	_object = ""
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


## PR 16e: aims the reticle at the drawn object `id`, as a player looks at it, and opens your menu (B);
## the next choice is among its entries about the object. A ray that meets something else fails by
## name; nothing is moved for it.
func _open_object(id: String, link: SliceLink, menu: SliceOfferMenu) -> String:
	menu.close()
	_object = ""
	await _frames(2)
	if not await _until(func() -> bool: return link.objects.nodes.has(id), STEP_LIMIT_S):
		return "the object %s is never drawn" % id
	for lift in [0.0, 0.05, -0.05]:
		await _aim_at(link.objects.drawn_at(id) + Vector3.UP * lift)
		if link.aim()["entity"] == id:
			break
	var aimed: Dictionary = link.aim()
	print("aim      object %s -> %s (%s)" % [id, aimed["entity"], aimed["kind"]])
	if aimed["entity"] != id:
		return "the ray never meets the object %s (it meets %s)" % [id, aimed["entity"]]
	await _press(KEY_B)
	if not await _until(func() -> bool: return menu.subject == link.client.observer, 5.0):
		return "B opened no menu of yours"
	_object = id
	return ""


## PR 16e: walks the body on the real controller until NEAR_M short of the drawn object or person `id`.
func _walk_near(id: String, link: SliceLink) -> String:
	var at := Vector3.ZERO
	if link.figures.has(id):
		at = (link.figures[id] as Node3D).global_position
	elif link.objects.nodes.has(id):
		at = link.objects.drawn_at(id)
	else:
		return "%s is neither a drawn person nor a drawn object" % id
	var from := player.global_position
	var flat := Vector3(at.x - from.x, 0.0, at.z - from.z)
	var stop := at - flat.normalized() * NEAR_M
	await _walk_to(Vector3(stop.x, from.y, stop.z), 12.0)
	await _hold(1.0)
	var left := Vector2(at.x - player.global_position.x, at.z - player.global_position.z).length()
	print("walk     near %s: stopped %.2f m from it, from %s to %s" % [id, left, from, player.global_position])
	return ""


## Resolves a selector against the built entries by their data, presses that entry, and waits until
## the world has answered what it sent and the panels have had time to follow. After `open_object`,
## only the entries about that object are candidates. `input`: the words typed for a composed entry.
func _choose(i: int, selector: Dictionary, link: SliceLink, menu: SliceOfferMenu, input := "") -> String:
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
			if (_object == "" or entry["about"] == _object) and _matches(entry, selector):
				index = entry["index"]
				break
		if index < 0:
			return "no built entry%s matches %s" % ["" if _object == "" else " about %s" % _object,
				JSON.stringify(selector)]
		var line: SliceTalkLine = slice.talk_line
		if input != "" and line != null:
			line.last_token = ""
		var token := menu.choose(index)
		if input != "":
			# A composed talk entry opens the talk line for its person (16g1): the words are typed there
			# and sent with Enter, as a player does.
			if line == null or not await _until(func() -> bool: return line.is_open(), 5.0):
				return "choosing %s opened no talk line" % JSON.stringify(selector)
			line.line.type_and_send(input)
			token = line.last_token
			if token == "":
				return "Enter in the talk line sent nothing"
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
	# the drawing camera followed the body: its distance to the body's pivot, against the rig's boom
	var cam := get_viewport().get_camera_3d()
	if cam != null:
		var pivot := player.global_position + Vector3.UP * CameraRig.PIVOT_HEIGHT
		print("camera   %s %.2f m from the body's pivot (rear boom %.2f m)"
			% [cam.name, cam.global_position.distance_to(pivot), CameraRig.REAR_DISTANCE])


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


## Aims at `who` and presses E, as a player does; "" when the talk line opened for them, else why not.
func _open_line(who: String, link: SliceLink) -> String:
	slice.offer_menu.close()
	await _frames(2)
	if not await _until(func() -> bool: return link.figures.has(who), STEP_LIMIT_S):
		return "%s is never drawn" % who
	if not await _clear_aim(link, who):
		return "the ray never meets %s (it meets %s)" % [who, link.facing_person()]
	_report_aim(link, "say to %s" % who)
	var line := slice.talk_line
	await _press(KEY_E)
	if not await _until(func() -> bool: return line.is_open(), 5.0):
		return "E on %s opened no talk line" % who
	if line.target != who:
		return "E on %s opened the line for %s" % [who, line.target]
	return ""


## PR 16g1: E, the words typed into the shared line, Enter; waits for the world's answer.
func _say_step(i: int, step: Dictionary, link: SliceLink) -> String:
	var who := String(step.get("to", ""))
	var failure: String = await _open_line(who, link)
	if failure != "":
		return failure
	var line := slice.talk_line
	if step.has("turn_deg"):
		player.rotation.y += deg_to_rad(float(step["turn_deg"]))
		await _frames(3)
	var facing_at_enter := link.facing_person()
	var sent_before := _requests.size()
	line.last_token = ""
	line.line.type_and_send(String(step["say"]))
	var token := line.last_token
	if token == "":
		return "Enter in the talk line sent nothing"
	if not await _until(func() -> bool: return _answered(link, token), STEP_LIMIT_S):
		return "the world did not answer %s within %d s" % [token, STEP_LIMIT_S]
	print("STEP_DONE ", JSON.stringify({"step": i, "sent": _requests.size() - sent_before, "token": token,
		"pinned": who, "facing_at_enter": facing_at_enter, "line_open": line.is_open()}))
	return ""


## A key typed as a keyboard sends it: pressed with its character, through the engine's input.
func _type_key(keycode: Key, character: String, pressed: bool) -> void:
	var key := InputEventKey.new()
	key.keycode = keycode
	key.physical_keycode = keycode
	key.unicode = character.unicode_at(0) if pressed and character != "" else 0
	key.pressed = pressed
	Input.parse_input_event(key)


## AC-3DS-3: with the talk line open, the keys a player walks and acts with only type.
func _typing_gate(who: String, link: SliceLink) -> String:
	var failure: String = await _open_line(who, link)
	if failure != "":
		return failure
	var line := slice.talk_line
	var settings := slice.settings_menu
	var asked := _requests.size()
	var at := player.global_position
	# W held for a second -- the walk key -- then a, s, d, space (jump) and e (talk), each typed.
	_type_key(KEY_W, "w", true)
	await _hold(1.0)
	_type_key(KEY_W, "w", false)
	for pair in [[KEY_A, "a"], [KEY_S, "s"], [KEY_D, "d"], [KEY_SPACE, " "], [KEY_E, "e"]]:
		_type_key(pair[0], pair[1], true)
		await _frames(2)
		_type_key(pair[0], pair[1], false)
		await _frames(2)
	await _hold(0.5)
	var typed := line.line.typed()
	var moved := at.distance_to(player.global_position)
	var while_open := _requests.size() - asked
	var open_after_typing := line.is_open()
	await _press(KEY_ESCAPE)
	await _frames(2)
	var after_esc := {"line_open": line.is_open(), "settings_open": settings.is_open(),
		"requests": _requests.size() - asked}
	await _press(KEY_E)
	await _frames(2)
	var reopened := line.is_open() and line.target == who
	await _press(KEY_ESCAPE)
	await _frames(2)
	var closed_again := not line.is_open() and not settings.is_open()
	await _hold(0.5)
	print("EVIDENCE ", JSON.stringify({"typing_gate": {"typed": typed, "moved_mm": roundi(moved * 1000.0),
		"requests_while_open": while_open, "open_after_typing": open_after_typing, "after_esc": after_esc,
		"reopened": reopened, "closed_again": closed_again, "requests": _requests.size() - asked}}))
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
