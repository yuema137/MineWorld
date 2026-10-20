extends SceneTree

## The world-time module and the date formatter, headless (`clients/shared/WORLD_TIME.md` §8;
## pr-tw-e-set-c §8 AC-WC-2, -4, -5, -6 and the formats of §5.3).
##
##   godot --headless --path clients/shared --script res://checks/world_time_check.gd
##
## Every input is scripted: the frames and the wall clock (`now_usec`). The calendar records are built
## the way the `calendar` System Pack discloses them, with an epoch (2031-03-14, a Friday) that is no
## machine's today, so a date taken from the OS clock fails. Prints `[PASS]`/`[FAIL]` per assertion and
## `world_time_check: PASS|FAIL`; exits non-zero on any failure.

const SECOND := 1000000

var _fails := 0


## A stand-in for `MineWorldObservation`: the three calls the module makes.
class Seen:
	extends RefCounted
	var _at := 0
	var _place := ""
	var _components := {}

	func _init(at: int, place: String, components: Dictionary) -> void:
		_at = at
		_place = place
		_components = components

	func at() -> int:
		return _at

	func place() -> String:
		return _place

	func component(_id: String, component_type: String) -> Dictionary:
		return _components.get(component_type, {})


func _init() -> void:
	for problem in MineWorldText.load_layers("", PackedStringArray()):
		push_warning(problem)
	_estimate_and_display()
	_date_and_roll_over()
	_sun()
	_phase()
	_formats()
	print("world_time_check: %s" % ("PASS" if _fails == 0 else "FAIL"))
	quit(0 if _fails == 0 else 1)


func _check(ok: bool, what: String) -> void:
	print("[%s] %s" % ["PASS" if ok else "FAIL", what])
	if not ok:
		_fails += 1


## A day record: `date`, a weekday, `day_start`, events and a track whose azimuth crosses north
## between samples 40 and 41 (359.9° → 0.2°).
static func _day(date: Dictionary, weekday: int, day_start: int) -> Dictionary:
	var track := []
	for i in 97:
		track.append({"elevation": -60000 + i * 1000, "azimuth": i * 3000})
	track[40] = {"elevation": 10000, "azimuth": 359900}
	track[41] = {"elevation": 11000, "azimuth": 200}
	return {"date": date, "weekday": weekday, "day_start": day_start,
		"events": {"astronomical_dawn": 18000, "civil_dawn": 21600, "sunrise": 23400, "solar_noon": 43200,
			"sunset": 63000, "civil_dusk": 64800, "astronomical_dusk": 68400}, "track": track}


func _estimate_and_display() -> void:
	var wt := MineWorldWorldTime.new()
	wt.on_welcome({"at": 1000, "time_scale": 12, "paused": false}, 0)
	_check(wt.display_seconds(10 * SECOND) == 1120, "10 wall s at 12x advance 120 world s")
	# AC-WC-2: a clock frame 120 s behind the running estimate holds the display, never sets it back.
	wt.on_clock(1000, 12, false, 10 * SECOND)
	var shown := []
	for k in range(0, 21):
		shown.append(wt.display_seconds(10 * SECOND + k * SECOND))
	var monotonic := true
	for k in range(1, shown.size()):
		monotonic = monotonic and shown[k] >= shown[k - 1]
	_check(monotonic, "AC-WC-2: no displayed value is smaller than the one before")
	_check(shown[5] == 1120, "AC-WC-2: the shown minute holds while the world catches up")
	_check(shown[20] == 1240, "AC-WC-2: then advances once the world passes it")
	wt.on_clock(1300, 12, true, 40 * SECOND)
	_check(wt.paused() and wt.display_seconds(90 * SECOND) == 1300, "paused: the estimate stands still")
	wt.on_clock(1300, 24, false, 90 * SECOND)
	_check(wt.display_seconds(100 * SECOND) == 1540, "a rescale to 24x advances at the new rate from its instant")
	wt.on_welcome({"at": 200, "time_scale": 12, "paused": false}, 100 * SECOND)
	_check(wt.display_seconds(100 * SECOND) == 200, "AC-WC-2: a new welcome earlier than the display is shown at once")


func _date_and_roll_over() -> void:
	var wt := MineWorldWorldTime.new()
	wt.on_welcome({"at": 0, "time_scale": 1}, 0)
	_check(wt.date_at(0).is_empty() and not wt.has_calendar(), "no record: no date (INV-TW-6)")
	wt.on_observation(Seen.new(0, "7", {"calendar-day": _day({"year": 2031, "month": 3, "day": 14}, 4, 0),
		"calendar-light": {"phase": "night"}}), 0)
	var d := wt.date_at(3600)
	_check(d == {"year": 2031, "month": 3, "day": 14, "weekday": 4}, "AC-WC-4: the date is the record's: %s" % d)
	wt.on_observation(Seen.new(5000, "", {}), 0)
	_check(wt.has_calendar(), "an observer in no place keeps the last record")
	var cases := [
		[{"year": 2026, "month": 10, "day": 8}, {"year": 2026, "month": 10, "day": 9}],
		[{"year": 2026, "month": 12, "day": 31}, {"year": 2027, "month": 1, "day": 1}],
		[{"year": 2028, "month": 2, "day": 28}, {"year": 2028, "month": 2, "day": 29}],
		[{"year": 2027, "month": 2, "day": 28}, {"year": 2027, "month": 3, "day": 1}],
	]
	for pair in cases:
		var rolled := MineWorldWorldTime.new()
		rolled.on_welcome({"at": 0}, 0)
		rolled.on_observation(Seen.new(0, "7", {"calendar-day": _day(pair[0], 6, 0)}), 0)
		var next := rolled.date_at(86400 + 60)
		_check(next.get("year") == pair[1]["year"] and next.get("month") == pair[1]["month"]
			and next.get("day") == pair[1]["day"] and next.get("weekday") == 0,
			"AC-WC-5: %s rolls to %s" % [pair[0], next])


func _sun() -> void:
	var wt := MineWorldWorldTime.new()
	wt.on_welcome({"at": 0}, 0)
	wt.on_observation(Seen.new(0, "7", {"calendar-day": _day({"year": 2031, "month": 3, "day": 14}, 4, 0)}), 0)
	var at_sample := wt.sun_at(10 * 900)
	_check(absf(at_sample["elevation_deg"] - (-50.0)) < 0.001 and absf(at_sample["azimuth_deg"] - 30.0) < 0.001,
		"AC-WC-6: a sample is the sample: %s" % at_sample)
	var mid := wt.sun_at(10 * 900 + 450)
	_check(absf(mid["elevation_deg"] - (-49.5)) < 0.001 and absf(mid["azimuth_deg"] - 31.5) < 0.001,
		"AC-WC-6: a midpoint is the mean: %s" % mid)
	var north := wt.sun_at(40 * 900 + 450)
	_check(absf(north["azimuth_deg"] - 0.05) < 0.001, "AC-WC-6: the midpoint across north is 0.05, not 180: %s" % north)
	_check(not wt.sun_at(86400 + 1)["valid"], "past the record's 24:00 the sun is held, not extrapolated")


func _phase() -> void:
	var wt := MineWorldWorldTime.new()
	wt.on_welcome({"at": 0}, 0)
	wt.on_observation(Seen.new(100, "7", {"calendar-day": _day({"year": 2031, "month": 3, "day": 14}, 4, 0),
		"calendar-light": {"phase": "night"}}), 0)
	_check(wt.phase_at(100) == "night", "the disclosed phase at its instant")
	_check(wt.phase_at(22000) == "civil-twilight", "advanced across civil dawn")
	_check(wt.phase_at(43200) == "day", "advanced across sunrise")
	_check(wt.phase_at(70000) == "night", "advanced across astronomical dusk")


func _formats() -> void:
	var thu := {"year": 2026, "month": 10, "day": 8, "weekday": 3}
	MineWorldText.set_language("en")
	MineWorldText.set_clock(MineWorldSettings.ClockFormat.AUTO)
	_check(MineWorldClockFormat.date_time(0, thu) == "Thu 8 Oct 2026, 12:00 AM",
		"en: %s" % MineWorldClockFormat.date_time(0, thu))
	_check(MineWorldClockFormat.date_time(3600 * 30, {}) == MineWorldClockFormat.day_time(3600 * 30),
		"no record: the day_time line unchanged (INV-TW-6)")
	MineWorldText.set_language("zh_Hans")
	_check(MineWorldClockFormat.date_time(0, thu) == "2026年10月8日 星期四 00:00",
		"zh_Hans automatic: %s" % MineWorldClockFormat.date_time(0, thu))
	MineWorldText.set_clock(MineWorldSettings.ClockFormat.H12)
	_check(MineWorldClockFormat.date_time(0, thu) == "2026年10月8日 星期四 上午 12:00",
		"zh_Hans 12-hour: %s" % MineWorldClockFormat.date_time(0, thu))
	MineWorldText.set_language("en")
	MineWorldText.set_clock(MineWorldSettings.ClockFormat.H24)
	_check(MineWorldClockFormat.date_time(0, thu) == "Thu 8 Oct 2026, 00:00",
		"en 24-hour: %s" % MineWorldClockFormat.date_time(0, thu))
	MineWorldText.set_clock(MineWorldSettings.ClockFormat.AUTO)
