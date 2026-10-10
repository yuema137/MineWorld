extends Node

## The scripted checks (`./mineworld-2d --drive[=scenario]`): drive the real client against a real or
## stub server, print what happened, assert what step-13 §14.2 decided before measuring, exit
## non-zero on any failure. Headless by default; `--capture` (windowed) also writes stills.
##
## Output, one line each, for a person and for the Rust tests that read it:
## ```text
## REQUEST {token, request}      every request the client sent, as sent (the AC-W10 transcript)
## RESULT {token, result}        every answer
## PLACE <id> <tags>             the observer's place changed
## EVIDENCE {…}                  a measured fact a test compares against its own oracle
## SHOWN {…}                     what the client draws, as data (places, doorways, people)
## [PASS] / [FAIL] what  detail  an assertion, with a fixed bound from the requirement (ARC-23)
## drive complete: PASS|FAIL
## ```
##
## Scenarios: `seated` (default), `walk`, `street`, `strides`, `idle`, `click`, `home`, and 13b's
## `steps` and `panels` (`harness/interact.gd`, which adds `MENU`, `PANELS`, `TOAST`, `STEP` lines), and
## S20's `settings` and `display` (`harness/settings.gd`; `--marker` adds the marker-catalog walk,
## `--apply=<language>,<clock>,<W>x<H>` applies through the menu).
## Arguments: `--strides=n`, `--hold=seconds`, `--frame` (print the first observation as a `FRAME`
## line), `--steps=<file>`, `--requests=<file>`, `--sync=<dir>`. No scenario names a world's
## coordinates: every waypoint is derived from the disclosed passages.

const Capture := preload("res://scripts/harness/capture.gd")
const Interact := preload("res://scripts/harness/interact.gd")
const SettingsScenario := preload("res://scripts/harness/settings.gd")

## Fixed bounds, from the requirements, never from the quantity under test (ARC-23).
const POSITION_BOUND_M := 0.001
const DOOR_BOUND_PX := 2.0
const HEIGHT_BOUND_PX := 2.0
const CUTAWAY_BOUND_S := 1.0
const SETTLE_S := 1.2
const TIMEOUT_S := 90.0

var app: Node
var _fails := 0
var _place := ""
var _place_changes := 0
var _place_changed_at := 0.0
var _results: Array = []
var _requests: Array = []
var _capture: Node = null
## Seconds from the latest place change to the first frame its façade was fully lifted, or -1.
var _lift_s := -1.0
var _disconnects := 0
var _observed_since_seated := false
var _interact: Node = null


## A reconciliation being timed: where the world put the body, and frames drawn since.
var _reconcile: Dictionary = {}


func _on_reconciled(place: String, plan: Vector2) -> void:
	_reconcile = {"place": place, "plan": plan, "frames": 0}


func _process(_delta: float) -> void:
	if not _reconcile.is_empty():
		var drawn: Variant = app.people.drawn_plan(app.people.observer)
		if drawn != null and (drawn as Vector2).distance_to(_reconcile["plan"]) <= POSITION_BOUND_M:
			print("EVIDENCE ", JSON.stringify({"reconciled": {"frames": _reconcile["frames"],
				"local": app.town.local_in(_reconcile["place"], _reconcile["plan"])}}))
			_reconcile = {}
		else:
			_reconcile["frames"] += 1
	if _lift_s < 0.0 and _place_changes > 0 and app.places.facade_alpha(_place) <= 0.01 \
			and app.places.facade_anchor(_place).size() > 0:
		_lift_s = _now() - _place_changed_at


func _ready() -> void:
	app.link.submitted.connect(func(token: String, request: Dictionary) -> void:
		_requests.append(request)
		print("REQUEST ", JSON.stringify({"token": token, "request": request})))
	app.link.resolved.connect(func(token: String, _a: String, result: Dictionary) -> void:
		_results.append(result)
		print("RESULT ", JSON.stringify({"token": token, "result": result, "t_ms": Time.get_ticks_msec()})))
	app.link.observed.connect(_on_observed)
	app.walker.reconciled.connect(_on_reconciled)
	app.link.state_changed.connect(func(state: String, reason: String) -> void:
		if state == "reconnecting":
			_disconnects += 1
		_observed_since_seated = false
		print("STATE %s %s" % [state, reason]))
	app.link.welcomed.connect(func(observer: String, world: Dictionary) -> void:
		print("EVIDENCE ", JSON.stringify({"welcome": {"observer": observer,
			"instance": world.get("instance", ""), "revision": world.get("revision")}})))
	if app.options.has("capture"):
		_capture = Capture.new()
		_capture.app = app
		add_child(_capture)
	print("EVIDENCE ", SettingsScenario.evidence(app))
	_interact = Interact.new()
	_interact.drive = self
	_interact.app = app
	add_child(_interact)
	_run.call_deferred()


func _on_observed(observation: MineWorldObservation) -> void:
	_observed_since_seated = true
	var place := observation.place()
	if place != _place:
		if _place != "":
			_place_changes += 1
		_place = place
		_place_changed_at = _now()
		_lift_s = -1.0
		print("PLACE %s %s" % [place, ",".join(PackedStringArray(observation.entity(place).get("tags", [])))])


func _run() -> void:
	var scenario := String(app.options.get("drive", ""))
	if scenario == "":
		scenario = "seated"
	print("drive: scenario %s, presentation %s, variant %s" % [scenario,
		"none" if app.presentation.is_plain() else app.presentation.directory, app.presentation.variant])
	if not await _until(func() -> bool: return app.walker.placed, TIMEOUT_S):
		_check(false, "seated", "no observation placed the player within %d s" % TIMEOUT_S)
		return _finish()
	if app.options.has("frame"):
		# The raw observation, for pinning a component's exact JSON before a reader is written.
		print("FRAME ", JSON.stringify(app.latest.frame))
	match scenario:
		"walk":
			await _walk()
		"strides":
			await _strides(int(app.options.get("strides", "5")))
		"street":
			await _street(int(app.options.get("strides", "12")))
		"idle":
			await _seconds(float(app.options.get("hold", "6")))
			_report_self()
		"click":
			await _click()
		"home":
			await _home(String(app.options.get("exit", "click")))
		"steps":
			await _interact.steps(String(app.options.get("steps", "")))
		"panels":
			await _interact.panels()
		"settings", "display":
			var settings: Node = SettingsScenario.new()
			settings.drive = self
			settings.app = app
			add_child(settings)
			if scenario == "settings":
				await settings.run()
			else:
				await settings.display()
		_:
			await _seconds(SETTLE_S)
			_check_seated()
	_finish()


## AC-W1's walk, with AC-W2 and AC-W12 checked on the way.
## Every waypoint is derived from the disclosed passages (12d moves the café's doorway, QD-11): out
## through the starting place's doorway and 1.5 m toward the middle of the street; to the nearest other
## doorway of the street (in market-town, the café's); and 2.5 m into the place behind it.
func _walk() -> void:
	var start: String = app.walker.body_place
	var exits: Array = app.town.passages.get(start, [])
	if exits.is_empty():
		_check(false, "walk", "the starting place discloses no doorway")
		return
	var street: String = exits[0]["to"]
	var exit_there: Vector2 = app.town.to_plan(street, {"x": exits[0]["there"].x, "y": exits[0]["there"].y})
	app.walker.walk_to(street, exit_there)
	if not await _walk_done("out onto the street"):
		return
	var centre: Vector2 = app.town.hub_extent().get_center()
	app.walker.walk_to(street, exit_there + (centre - exit_there).normalized() * 1.5)
	if not await _walk_done("into the street"):
		return
	await _seconds(SETTLE_S)
	# From the street, before the café: what the doorways read and how the façades are drawn (R-PK-1).
	_report_self()
	_check_facades()
	var door := Vector2i.ZERO
	var target := ""
	var nearest := INF
	for p in app.town.passages.get(street, []):
		var at: Vector2 = app.town.to_plan(street, {"x": p["here"].x, "y": p["here"].y})
		if p["to"] != start and at.distance_to(exit_there) < nearest:
			nearest = at.distance_to(exit_there)
			target = p["to"]
			door = p["here"]
	if target == "":
		_check(false, "walk", "the street discloses no other doorway")
		return
	if _capture != null:
		await _capture.shoot("01_street_wide")
	var entry: Dictionary = app.town.passage(street, target)
	var inside: Vector2 = app.town.to_plan(target, {"x": entry["there"].x, "y": entry["there"].y}) \
		+ app.places.drawn[target]["in_dir"] * 2.5
	app.walker.walk_to(target, inside)
	if not await _walk_done("into the place behind %s" % door):
		return
	var tags: Array = app.latest.entity(app.latest.place()).get("tags", [])
	print("EVIDENCE ", JSON.stringify({"entered": {"place": target, "tags": tags}}))
	# Timed from the observation that changed the place to the first frame the façade is gone,
	# measured every frame by `_process`, not after the walk inside ends.
	_check(_lift_s >= 0.0 and _lift_s <= CUTAWAY_BOUND_S, "façade lifts on entering",
		"gone %.2f s after the place changed (bound %.1f s)" % [_lift_s, CUTAWAY_BOUND_S])
	await _seconds(SETTLE_S)
	var accepted := 0
	for result in _results:
		if result.has("accepted"):
			accepted += 1
	_check(accepted == _results.size() and accepted > 0, "every stride accepted", "%d of %d" % [accepted, _results.size()])
	_check(_place_changes == 2, "place changed twice", "%d changes" % _place_changes)
	print("EVIDENCE ", JSON.stringify({"walk": {"accepted": accepted, "results": _results.size(),
		"place_changes": _place_changes, "observer": app.latest.observer()}}))
	_check_seated()
	if _capture != null:
		await _capture.shoot_interior()
		# Back out to the street, now that the café's tags are known, for its façade and terrace.
		var door_plan: Vector2 = app.town.to_plan(street, {"x": door.x, "y": door.y})
		var outward: Vector2 = (centre - door_plan).normalized()
		app.walker.walk_to(street, door_plan + outward * 4.5 + Vector2(outward.y, -outward.x) * 1.5)
		await _until(func() -> bool: return not app.walker.is_walking(), TIMEOUT_S)
		await _seconds(1.0)
		await _capture.shoot("02_cafe_door", 1.9, door_plan - outward * 1.5)


## `n` strides east in the current place, one after another (the stub scenarios, AC-W5 … AC-W7).
func _strides(n: int) -> void:
	var from: Vector2 = app.walker.body_plan
	app.walker.walk_to(app.walker.body_place, from + Vector2(1.9 * n, 0.0))
	await _until(func() -> bool: return not app.walker.is_walking(), TIMEOUT_S)
	await _seconds(SETTLE_S)
	_report_self()


## Out onto the street and along it (AC-W4: a test restarts the server under this walk). If the
## connection dropped, waits for the client's own reconnect before reporting.
func _street(n: int) -> void:
	var start: String = app.walker.body_place
	var exits: Array = app.town.passages.get(start, [])
	if exits.is_empty():
		_check(false, "street", "the starting place discloses no doorway")
		return
	var street: String = exits[0]["to"]
	var exit_there: Vector2 = app.town.to_plan(street, {"x": exits[0]["there"].x, "y": exits[0]["there"].y})
	app.walker.walk_to(street, exit_there)
	await _until(func() -> bool: return not app.walker.is_walking(), TIMEOUT_S)
	var centre: Vector2 = app.town.hub_extent().get_center()
	app.walker.walk_to(street, exit_there + (centre - exit_there).normalized() * 1.5)
	await _until(func() -> bool: return not app.walker.is_walking(), TIMEOUT_S)
	await _seconds(0.4)
	_report_self()
	var from: Vector2 = app.walker.body_plan
	app.walker.walk_to(app.walker.body_place, from + Vector2(1.9 * n, 0.0))
	await _until(func() -> bool: return not app.walker.is_walking(), TIMEOUT_S)
	if _disconnects > 0:
		var back := await _until(func() -> bool: return app.link.state == "seated" and _observed_since_seated, float(app.options.get("hold", "30")))
		_check(back, "reconnected", "%d disconnect(s), state %s" % [_disconnects, app.link.state])
	await _seconds(SETTLE_S)
	_report_self()


## The click path (AC-W10): clicking where a point is drawn asks to walk to that point.
## The player's own first minute, as `./mineworld-2d` starts it: seated at home, with only the room's
## one disclosed doorway known. Played through the real input path (a click on the screen, or the
## keys), never through the walker's API.
## 1. Click well past the room's far wall: the drawn floor must not change (F-9). The world may well
##    accept the walk — it discloses no extent for the room — and that is reported, not corrected.
## 2. Leave by the door, with a click on it (`--exit=click`) or by walking onto it with the keys
##    (`--exit=keys`): the observer's place must become the one the doorway leads to (F-10).
func _home(exit: String) -> void:
	await _seconds(SETTLE_S)
	var here: String = app.latest.place()
	var room: Dictionary = app.places.drawn.get(here, {})
	var doorways: Array = app.town.passages.get(here, [])
	if room.is_empty() or doorways.size() != 1:
		_check(false, "home", "not seated in a drawn room with one doorway: %s, %d doorway(s)" % [here, doorways.size()])
		return
	var p: Dictionary = doorways[0]
	var door: Vector2 = app.town.to_plan(here, {"x": p["here"].x, "y": p["here"].y})
	var floor_before: Rect2 = room["rect"]
	_check(app.places.has_method("door_drawn") and app.places.door_drawn(here, door), "the doorway is drawn", "at %s in %s" % [door, here])
	var in_dir: Vector2 = room["in_dir"]
	var beyond: Vector2 = door + in_dir * (floor_before.size.length() + 3.0)
	await _press_at(beyond)
	await _until(func() -> bool: return not app.walker.is_walking(), TIMEOUT_S)
	await _seconds(0.6)
	var floor_after: Rect2 = app.places.drawn.get(here, {}).get("rect", Rect2())
	_check(floor_after == floor_before, "the drawn floor stays put", "before %s, after %s" % [floor_before, floor_after])
	var body: Vector2 = app.walker.body_plan
	print("EVIDENCE ", JSON.stringify({"beyond_the_floor": {"place": app.latest.place(),
		"local": app.latest.self_location().get("local"), "outside_drawn_floor": not floor_before.has_point(body),
		"accepted": _results.filter(func(r: Dictionary) -> bool: return r.has("accepted")).size(),
		"results": _results.size()}}))
	if _capture != null:
		await _capture.shoot("07_home_interior", 1.6, floor_before.get_center())
	if exit == "keys":
		await _press_at(door + in_dir * 1.0)
		await _until(func() -> bool: return not app.walker.is_walking(), TIMEOUT_S)
		await _hold_keys_toward(door, p["to"])
	else:
		await _press_at(door)
	var out := await _until(func() -> bool: return app.latest.place() == p["to"] and not app.walker.is_walking(), 20.0)
	_check(out, "out through the door (%s)" % exit, "now in %s, the doorway leads to %s" % [app.latest.place(), p["to"]])
	await _seconds(SETTLE_S)
	_check(app.latest.place() == p["to"] and _place_changes == 1, "stays out (no bounce back)",
		"in %s after %d place change(s)" % [app.latest.place(), _place_changes])
	if _capture != null and out:
		await _seconds(SETTLE_S)
		await _capture.shoot("08_home_door", 1.6, door)
	_report_self()


## Holds the arrow keys whose direction on the screen best points the body at `door`, until the
## observer is in `to` or ten seconds pass.
func _hold_keys_toward(door: Vector2, to: String) -> void:
	var want: Vector2 = (door - app.walker.body_plan).normalized()
	var best: Array = []
	var best_dot := -2.0
	for combo in [["move_up"], ["move_down"], ["move_left"], ["move_right"], ["move_up", "move_left"],
			["move_up", "move_right"], ["move_down", "move_left"], ["move_down", "move_right"]]:
		var screen := Vector2.ZERO
		for action in combo:
			screen += {"move_up": Vector2(0, -1), "move_down": Vector2(0, 1), "move_left": Vector2(-1, 0), "move_right": Vector2(1, 0)}[action]
		var d: float = app.projection.screen_dir_to_plan(screen).dot(want)
		if d > best_dot:
			best_dot = d
			best = combo
	for action in best:
		Input.action_press(action)
	await _until(func() -> bool: return app.latest.place() == to, 10.0)
	for action in best:
		Input.action_release(action)


## A left click on the screen point where `plan` is drawn, through the viewport's input path.
func _press_at(plan: Vector2) -> void:
	var screen: Vector2 = app.get_viewport().get_canvas_transform() * app.projection.to_screen(plan)
	var press := InputEventMouseButton.new()
	press.button_index = MOUSE_BUTTON_LEFT
	press.pressed = true
	press.position = screen
	press.global_position = screen
	# In the viewport's own coordinates (`in_local_coords`), as the canvas transform gives them.
	app.get_viewport().push_input(press, true)
	await _seconds(0.2)


func _click() -> void:
	await _seconds(SETTLE_S)
	var target: Vector2 = app.walker.body_plan + Vector2(1.0, 0.5)
	var screen: Vector2 = app.get_viewport().get_canvas_transform() * app.projection.to_screen(target)
	await _press_at(target)
	var asked: Variant = null
	if not _requests.is_empty():
		var to: Dictionary = _requests[-1]["payload"]["payload"]["to"]
		asked = app.town.to_plan(String(to["place"]["entity"]), to["local"])
	var miss: float = INF if asked == null else (asked as Vector2).distance_to(target)
	print("EVIDENCE ", JSON.stringify({"click": {"target": [target.x, target.y], "screen": [screen.x, screen.y],
		"asked": null if asked == null else [asked.x, asked.y], "requests": _requests.size()}}))
	_check(miss <= POSITION_BOUND_M, "a click asks for the clicked point", "miss %.4f m (bound %.3f)" % [miss, POSITION_BOUND_M])
	await _until(func() -> bool: return not app.walker.is_walking(), TIMEOUT_S)


## AC-W2 and AC-W12, checked against the newest observation.
func _check_seated() -> void:
	var observation: MineWorldObservation = app.latest
	_check(app.link.instance != "", "welcomed into a world", "instance %s" % app.link.instance)
	var listed := {}
	for entity in observation.entities():
		if String(entity.get("entity_type", "")) == "person":
			listed[String(entity.get("id", ""))] = entity
	for id in app.people.drawn_ids():
		_check(listed.has(id), "drawn only if perceived", "person %s" % id)
	for id in listed:
		var where: Variant = app.town.to_plan(observation.place(), listed[id].get("location", {}).get("local"))
		var drawn: Variant = app.people.drawn_plan(id)
		var miss: float = INF if where == null or drawn == null else (where as Vector2).distance_to(drawn)
		_check(miss <= POSITION_BOUND_M, "drawn where the server says", "%s miss %.4f m" % [id, miss])
		var name := observation.display_name(id)
		var label: String = app.people.label_of(id)
		_check(name != "" and label == name and label != id, "labelled by name", "%s → %s" % [id, label])
		var height_m: Variant = app.people.intended_height_m(id)
		if height_m != null:
			var px: float = app.people.drawn_height_px(id)
			var want: float = float(height_m) * app.projection.ppm
			_check(absf(px - want) <= HEIGHT_BOUND_PX, "drawn at its height", "%s %.1f px, want %.1f ± %.0f" % [id, px, want, HEIGHT_BOUND_PX])
	_check_painters_order(listed.keys())
	var missing := PackedStringArray()
	for role in app.presentation.bound_roles():
		var loaded: bool = not app.presentation.sprite(role).is_empty() or not app.presentation.directional(role).is_empty()
		if not loaded:
			missing.append(role)
	_check(missing.is_empty() and app.presentation.errors.is_empty(), "every bound role resolves",
		"%d roles, missing %s, errors %s" % [app.presentation.bound_roles().size(), missing, app.presentation.errors])
	_report_self()


## AC-W12's painter's order: of the two drawn people furthest apart down the screen, the nearer one
## (further south, lower on the screen) is drawn in front of the other.
func _check_painters_order(ids: Array) -> void:
	var near := ""
	var far := ""
	for id in ids:
		var plan: Variant = app.people.drawn_plan(id)
		if plan == null:
			continue
		var depth: float = app.projection.to_screen(plan).y
		if near == "" or depth > app.projection.to_screen(app.people.drawn_plan(near)).y:
			near = id
		if far == "" or depth < app.projection.to_screen(app.people.drawn_plan(far)).y:
			far = id
	if near == far:
		return  # one person in view (the apartments): nothing to order
	_check(app.people.drawn_behind(far, near) and not app.people.drawn_behind(near, far),
		"painter's order", "%s drawn in front of %s" % [near, far])


func _check_facades() -> void:
	var checked := 0
	for place in app.places.drawn:
		var anchor: Dictionary = app.places.facade_anchor(place)
		if anchor.is_empty():
			continue
		checked += 1
		var miss: float = (anchor["drawn"] as Vector2).distance_to(anchor["doorway"])
		_check(miss <= DOOR_BOUND_PX, "façade on its doorway", "%s miss %.2f px" % [place, miss])
	print("EVIDENCE ", JSON.stringify({"facades_checked": checked}))


## What the client shows, as data (the "one world, two views" rule): every place it draws and why,
## every doorway with its destination, every person with their label and position. 13f compares this
## with the 3D client's report of the same world.
func _report_shown() -> void:
	var observation: MineWorldObservation = app.latest
	var here := observation.place()
	var places := []
	for place in app.places.drawn:
		places.append({"place": place, "drawn_as": app.places.drawn[place]["kind"],
			"tags": app.town.tags.get(place, PackedStringArray())})
	var doorways := []
	for place in app.town.passages:
		for p in app.town.passages[place]:
			doorways.append({"from": place, "to": p["to"], "here": [p["here"].x, p["here"].y],
				"there": [p["there"].x, p["there"].y]})
	var people := []
	for id in app.people.drawn_ids():
		var drawn: Variant = app.people.drawn_plan(id)
		people.append({"id": id, "label": app.people.label_of(id), "activity": app.people.activity_of(id),
			"local": null if drawn == null else app.town.local_in(here, drawn)})
	print("SHOWN ", JSON.stringify({"place": here, "places": places, "doorways": doorways,
		"doorway_labels": app.places.door_labels(), "people": people}))


func _report_self() -> void:
	_report_shown()
	var observation: MineWorldObservation = app.latest
	var me := observation.observer()
	var drawn: Variant = app.people.drawn_plan(me)
	print("EVIDENCE ", JSON.stringify({"self": {"place": observation.place(),
		"local": observation.self_location().get("local"),
		"drawn_local": null if drawn == null else app.town.local_in(observation.place(), drawn),
		"body_local": app.town.local_in(app.walker.body_place, app.walker.body_plan),
		"passages": app.town.passage_count(), "instance": app.link.instance,
		"stale_frames_ignored": app.walker.stale_ignored}}))


func _walk_done(what: String) -> bool:
	var ended := await _until(func() -> bool: return not app.walker.is_walking(), TIMEOUT_S)
	if not ended:
		_check(false, "walk " + what, "did not finish in %d s" % TIMEOUT_S)
		return false
	for result in _results:
		if not result.has("accepted"):
			_check(false, "walk " + what, "the world refused a stride: %s" % JSON.stringify(result))
			return false
	await _seconds(0.4)
	return true




func _check(ok: bool, what: String, detail: String) -> void:
	if not ok:
		_fails += 1
	print("  [%s] %-30s %s" % ["PASS" if ok else "FAIL", what, detail])


func _finish() -> void:
	print("drive complete: %s" % ("PASS" if _fails == 0 else "FAIL"))
	get_tree().quit(0 if _fails == 0 else 1)


func _until(condition: Callable, limit_s: float) -> bool:
	var deadline := _now() + limit_s
	while not condition.call():
		if _now() > deadline:
			return false
		await get_tree().process_frame
	return true


func _seconds(s: float) -> void:
	await get_tree().create_timer(s).timeout


func _now() -> float:
	return Time.get_ticks_msec() / 1000.0
