extends SceneTree

## The station clock rule, headless (`ARC-108`; pr-s29e §5.2, AC-e-1's clock rows; M-e-1).
##
##   godot --headless --path clients/shared --script res://checks/station_clock_check.gd
##
## Prints `CLOCK <case> <value>` per case, `[FAIL]` for a case that differs, and
## `all station clock checks pass` or `<n> STATION CLOCK CHECKS FAILED`; exits non-zero on any failure.

const Clock := preload("res://station/station_clock.gd")

var _fails := 0


class Seen:
	extends RefCounted
	var frame := {}

	func _init(the_frame: Dictionary) -> void:
		frame = the_frame

	func at() -> int:
		return int(frame.get("at", 0))

	func place() -> String:
		return String(frame.get("self_location", {}).get("place", {}).get("entity", ""))

	func entity(id: String) -> Dictionary:
		for e in frame.get("entities", []):
			if String(e.get("id", "")) == id:
				return e
		return {}


func _init() -> void:
	# Social Café and Lakeside: no calendar disclosed, so no hands, whatever the world time.
	_case("no calendar -> no hands", Clock.of(PackedStringArray(["place-shape"]), 117000), -1)
	_case("nothing disclosed -> no hands", Clock.of(PackedStringArray(), 0), -1)
	# Market Town: a calendar, so the time of day of the observation, on any day.
	_case("calendar -> time of day", Clock.of(PackedStringArray(["calendar-day", "shop"]), 117000), 30600)
	_case("calendar, midnight", Clock.of(PackedStringArray(["calendar-day"]), 86400 * 3), 0)
	var frame := {"at": 90061, "self_location": {"place": {"entity": "5"}}, "entities": [
		{"id": "5", "entity_type": "place", "components": [
			{"component_type": "calendar-day", "payload": {"day": 2}}]}]}
	_case("of_observation calendar", Clock.from_observation(Seen.new(frame)), 3661)
	frame["entities"][0]["components"] = []
	_case("of_observation no calendar", Clock.from_observation(Seen.new(frame)), -1)
	_case("of_observation nowhere", Clock.from_observation(Seen.new({"at": 5})), -1)
	if _fails == 0:
		print("all station clock checks pass")
	else:
		print("%d STATION CLOCK CHECKS FAILED" % _fails)
	quit(1 if _fails > 0 else 0)


func _case(name: String, got: int, want: int) -> void:
	print("CLOCK %s %d" % [name, got])
	if got != want:
		_fails += 1
		print("[FAIL] %s: expected %d" % [name, want])
