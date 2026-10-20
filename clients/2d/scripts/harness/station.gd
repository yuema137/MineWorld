extends Node

## The station drive (`./mineworld-2d --drive=station`; pr-s29f §7): walk to the station, into its
## hall, into the ticket office and back, onto the platform and back, and report what the client drew.
##
## Every place is found by its tags, never by a key: the hub's doorway to a place tagged `station`, then
## the hall's doorways to places tagged `ticket-office` and `platform`. Prints `EVIDENCE {"station_…"}`
## lines for the runner's oracle (`tools/cli/tests/client_2d_station.rs`) and ends with
## `all station checks pass` or `<n> STATION CHECKS FAILED`. With `--capture`, writes `st2d_approach`,
## `st2d_hall`, `st2d_tickets` and `st2d_platform`.

const State := preload("res://mineworld_station/station_state.gd")

## How far before the station's doorway the approach still is taken (pr-s29f §7).
const APPROACH_M := 18.0
## How far into a place a leg walks past its doorway.
const IN_M := 2.5

var drive: Node
var app: Node
var _fails := 0


func run() -> void:
	if not await drive._out_to_hub():
		_check(false, "station", "never reached the street")
		return _summary()
	await drive._seconds(drive.SETTLE_S)
	var hub: String = app.town.hub()
	var way := _doorway(hub, "station", ["ticket-office", "platform"])
	if way.is_empty():
		_check(false, "station doorway", "no hub doorway leads to a place tagged station")
		return _summary()
	var hall: String = way["to"]
	var entry: Dictionary = app.places.drawn.get(hall, {})
	var holder: Node = entry.get("node")
	var sprite_id := String(holder.get_meta("sprite_id", "")) if holder != null else ""
	var in_dir: Vector2 = entry.get("in_dir", Vector2.ZERO)
	_evidence("station_facade", {"place": hall, "drawn_as": entry.get("kind", ""), "sprite": sprite_id,
		"in_dir": [in_dir.x, in_dir.y], "tags": app.town.tags.get(hall, PackedStringArray())})
	# From the street nothing about the hall is disclosed: its fixtures are shut and blank (M-f-5).
	_evidence("station_from_street", app.places.fittings.state_of(hall))
	_check(app.places.fittings.state_of(hall)["kiosk"] == State.SHUT, "unentered hall's kiosk is shut",
		"standing in %s" % app.latest.place())
	var door: Vector2 = app.town.to_plan(hub, {"x": way["here"].x, "y": way["here"].y})
	if drive._capture != null:
		await drive._capture.shoot("st2d_approach", 1.0, door - in_dir * APPROACH_M)
	app.walker.walk_to(hub, door - in_dir * drive.FRONT_M)
	if not await drive._walk_done("to the station's front"):
		return _summary()
	app.walker.walk_to(hall, app.town.to_plan(hall, {"x": way["there"].x, "y": way["there"].y}) + in_dir * IN_M)
	if not await drive._walk_done("into the station"):
		return _summary()
	await drive._seconds(drive.SETTLE_S)
	_check(app.latest.place() == hall, "in the station", "standing in %s" % app.latest.place())
	_report_room(hall)
	if drive._capture != null:
		await drive._capture.shoot("st2d_hall", 0.8, app.walker.body_plan)
	for leg in [["ticket-office", "st2d_tickets"], ["platform", "st2d_platform"]]:
		await _visit(hall, leg[0], leg[1])
	_summary()


## Into the hall's neighbour tagged `tag`, a report and a still there, and back into the hall.
func _visit(hall: String, tag: String, shot: String) -> void:
	var way := _doorway(hall, tag, [])
	if way.is_empty():
		_check(false, "%s doorway" % tag, "the hall has no doorway to a place tagged %s" % tag)
		return
	var target: String = way["to"]
	var door: Vector2 = app.town.to_plan(hall, {"x": way["here"].x, "y": way["here"].y})
	var room: Rect2 = app.places.drawn[hall]["rect"]
	var out := _axis(door - room.get_center())
	app.walker.walk_to(hall, door - out * 1.0)
	if not await drive._walk_done("to the %s door" % tag):
		return
	app.walker.walk_to(target, app.town.to_plan(target, {"x": way["there"].x, "y": way["there"].y}) + out * IN_M)
	if not await drive._walk_done("into the %s" % tag):
		return
	await drive._seconds(drive.SETTLE_S)
	_check(app.latest.place() == target, "in the %s" % tag, "standing in %s" % app.latest.place())
	_report_room(target)
	if drive._capture != null:
		await drive._capture.shoot(shot, 0.9, app.walker.body_plan)
	app.walker.walk_to(hall, door - out * IN_M)
	await drive._walk_done("back into the hall from the %s" % tag)


## What the room the observer stands in was drawn as: its rectangle and whether the world shaped it,
## its fixture state, the words over its fittings, and the furnishings that follow a state.
func _report_room(place: String) -> void:
	var entry: Dictionary = app.places.drawn.get(place, {})
	var rect: Rect2 = entry.get("rect", Rect2())
	var words := []
	for row in app.places.fittings.rows:
		if row["place"] == place:
			words.append({"role": row["role"], "label": String(row["item"].get("label", "")),
				"text": app.places.fittings.word_of(row)})
	var stateful := []
	for row in app.furnishings.rows.get(place, []):
		if String(row["role"]).contains(":"):
			stateful.append({"key": row["key"], "role": row["role"]})
	_evidence("station_room", {"place": place, "tags": app.town.tags.get(place, PackedStringArray()),
		"size_m": [snappedf(rect.size.x, 0.001), snappedf(rect.size.y, 0.001)],
		"shaped": entry.get("shaped", false), "wall_m": entry.get("wall_m", 2.8),
		"state": State.of(app.latest), "words": words, "furnishings": stateful,
		"doorway_labels": app.places.door_labels()})


## The doorway of `from` to a place carrying `tag` and none of `without`; {} when none does.
func _doorway(from: String, tag: String, without: Array) -> Dictionary:
	for p in app.town.passages.get(from, []):
		var to_tags: PackedStringArray = p["to_tags"]
		if not to_tags.has(tag):
			continue
		var excluded := false
		for other in without:
			if to_tags.has(other):
				excluded = true
		if not excluded:
			return p
	return {}


static func _axis(v: Vector2) -> Vector2:
	if absf(v.x) > absf(v.y):
		return Vector2(signf(v.x), 0.0)
	return Vector2(0.0, signf(v.y) if v.y != 0.0 else -1.0)


func _evidence(key: String, value: Variant) -> void:
	print("EVIDENCE ", JSON.stringify({key: value}))


func _check(ok: bool, what: String, detail: String) -> void:
	if not ok:
		_fails += 1
	drive._check(ok, what, detail)


func _summary() -> void:
	print("all station checks pass" if _fails == 0 else "%d STATION CHECKS FAILED" % _fails)
