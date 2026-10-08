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
## Scenarios: `seated` (default), `walk`, `strides`, `idle`, `click`. Arguments: `--walk-out=x,y`,
## `--walk-door=x,y`, `--walk-inside=x,y` (millimetres; market-town's street and café by default),
## `--strides=n`, `--hold=seconds`.

const Capture := preload("res://scripts/harness/capture.gd")

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


func _process(_delta: float) -> void:
	if _lift_s < 0.0 and _place_changes > 0 and app.places.facade_alpha(_place) <= 0.01 \
			and app.places.facade_anchor(_place).size() > 0:
		_lift_s = _now() - _place_changed_at


func _ready() -> void:
	app.link.submitted.connect(func(token: String, request: Dictionary) -> void:
		_requests.append(request)
		print("REQUEST ", JSON.stringify({"token": token, "request": request})))
	app.link.resolved.connect(func(token: String, _a: String, result: Dictionary) -> void:
		_results.append(result)
		print("RESULT ", JSON.stringify({"token": token, "result": result})))
	app.link.observed.connect(_on_observed)
	app.link.welcomed.connect(func(observer: String, world: Dictionary) -> void:
		print("EVIDENCE ", JSON.stringify({"welcome": {"observer": observer,
			"instance": world.get("instance", ""), "revision": world.get("revision")}})))
	if app.options.has("capture"):
		_capture = Capture.new()
		_capture.app = app
		add_child(_capture)
	_run.call_deferred()


func _on_observed(observation: MineWorldObservation) -> void:
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
	match scenario:
		"walk":
			await _walk()
		"strides":
			await _strides(int(app.options.get("strides", "5")))
		"idle":
			await _seconds(float(app.options.get("hold", "6")))
			_report_self()
		"click":
			await _click()
		_:
			await _seconds(SETTLE_S)
			_check_seated()
	_finish()


## AC-W1's walk, with AC-W2 and AC-W12 checked on the way.
func _walk() -> void:
	var out := _point("walk-out", Vector2i(-12000, 1500))
	var door := _point("walk-door", Vector2i(0, 3000))
	var inside := _point("walk-inside", Vector2i(2400, 3000))
	var start: String = app.walker.body_place
	var exits: Array = app.town.passages.get(start, [])
	if exits.is_empty():
		_check(false, "walk", "the starting place discloses no doorway")
		return
	var street: String = exits[0]["to"]
	app.walker.walk_to(street, app.town.to_plan(street, {"x": out.x, "y": out.y}))
	if not await _walk_done("out onto the street"):
		return
	await _seconds(SETTLE_S)
	_check_facades()
	var target := ""
	for p in app.town.passages.get(street, []):
		if p["here"] == door:
			target = p["to"]
	if target == "":
		_check(false, "walk", "no doorway at %s in the street" % door)
		return
	if _capture != null:
		await _capture.shoot("01_street_wide")
	app.walker.walk_to(target, app.town.to_plan(target, {"x": inside.x, "y": inside.y}))
	if not await _walk_done("into the place behind %s" % door):
		return
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


## `n` strides east in the current place, one after another (the stub scenarios, AC-W5 … AC-W7).
func _strides(n: int) -> void:
	var from: Vector2 = app.walker.body_plan
	app.walker.walk_to(app.walker.body_place, from + Vector2(1.9 * n, 0.0))
	await _until(func() -> bool: return not app.walker.is_walking(), TIMEOUT_S)
	await _seconds(SETTLE_S)
	_report_self()


## The click path (AC-W10): clicking where a point is drawn asks to walk to that point.
func _click() -> void:
	await _seconds(SETTLE_S)
	var target: Vector2 = app.walker.body_plan + Vector2(1.0, 0.5)
	var screen: Vector2 = app.get_viewport().get_canvas_transform() * app.projection.to_screen(target)
	var press := InputEventMouseButton.new()
	press.button_index = MOUSE_BUTTON_LEFT
	press.pressed = true
	press.position = screen
	press.global_position = screen
	# In the viewport's own coordinates (`in_local_coords`), as the canvas transform gives them.
	app.get_viewport().push_input(press, true)
	await _seconds(0.2)
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
	var missing := PackedStringArray()
	for role in app.presentation.bound_roles():
		var loaded: bool = not app.presentation.sprite(role).is_empty() or not app.presentation.directional(role).is_empty()
		if not loaded:
			missing.append(role)
	_check(missing.is_empty() and app.presentation.errors.is_empty(), "every bound role resolves",
		"%d roles, missing %s, errors %s" % [app.presentation.bound_roles().size(), missing, app.presentation.errors])
	_report_self()


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
		people.append({"id": id, "label": app.people.label_of(id),
			"local": null if drawn == null else app.town.local_in(here, drawn)})
	print("SHOWN ", JSON.stringify({"place": here, "places": places, "doorways": doorways, "people": people}))


func _report_self() -> void:
	_report_shown()
	var observation: MineWorldObservation = app.latest
	var me := observation.observer()
	var drawn: Variant = app.people.drawn_plan(me)
	print("EVIDENCE ", JSON.stringify({"self": {"place": observation.place(),
		"local": observation.self_location().get("local"),
		"drawn_local": null if drawn == null else app.town.local_in(observation.place(), drawn),
		"body_local": app.town.local_in(app.walker.body_place, app.walker.body_plan),
		"passages": app.town.passage_count(), "instance": app.link.instance}}))


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


func _point(name: String, fallback: Vector2i) -> Vector2i:
	var text := String(app.options.get(name, ""))
	var parts := text.split(",")
	return Vector2i(int(parts[0]), int(parts[1])) if parts.size() == 2 else fallback


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
