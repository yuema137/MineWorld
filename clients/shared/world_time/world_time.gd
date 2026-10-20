## The world's time, date and sun between frames, as the server disclosed them
## (`clients/shared/WORLD_TIME.md`, `ARC-266`).
##
## Fed the `welcome` summary, every `clock` frame and every observation, each with the caller's monotonic
## wall clock in microseconds (`Time.get_ticks_usec()`); it reads no clock itself. The date and the sun
## come only from the `calendar` System Pack's `calendar-day` and `calendar-light` records: nothing here
## computes astronomy, and a world without `calendar` has no date (`INV-TW-6`). Integers until the last
## step (`INV-TW-5`).
class_name MineWorldWorldTime
extends RefCounted

const DAY := 86400
const SAMPLE_INTERVAL := 900
const SAMPLES := 97

## The phase each light event begins, in the pack's order (`DayEvents::light_changes`).
const LIGHT_EVENTS := [
	["astronomical_dawn", "astronomical-twilight"],
	["civil_dawn", "civil-twilight"],
	["sunrise", "day"],
	["sunset", "civil-twilight"],
	["civil_dusk", "astronomical-twilight"],
	["astronomical_dusk", "night"],
]

var _anchored := false
var _anchor_at := 0
var _anchor_usec := 0
var _scale := 1
var _paused := false
## The largest instant shown in this session; the display never goes below it (`INV-TW-8`).
var _last_display := -9223372036854775807

var _has_day := false
var _date := {}
var _weekday := 0
var _day_start := 0
var _track: Array = []
## (offset after day start, phase) in time order.
var _changes: Array = []
var _light_phase := ""
## The instant the disclosed phase was true at.
var _light_at := 0


## A new session: the first anchor. Resets the display rule (a killed host may resume earlier).
func on_welcome(world: Dictionary, now_usec: int) -> void:
	_scale = maxi(1, int(world.get("time_scale", 1)))
	_paused = bool(world.get("paused", false))
	_anchor(int(world.get("at", 0)), now_usec)
	_last_display = -9223372036854775807


## A `clock` frame (`PROTOCOL.md` §5.9): re-anchor with the new pacing.
func on_clock(at: int, time_scale: int, paused: bool, now_usec: int) -> void:
	_scale = maxi(1, time_scale)
	_paused = paused
	_anchor(at, now_usec)


## An observation: re-anchor at its instant, and read the day and light records about its place. A frame
## without them keeps the last record (an observer in transit is disclosed none).
## (`observation` is a `MineWorldObservation`; typed loosely so this module loads in a project that does
## not take the protocol module, such as `clients/shared`'s checks.)
func on_observation(observation: Object, now_usec: int) -> void:
	_anchor(observation.at(), now_usec)
	var place: String = observation.place()
	if place == "":
		return
	var day: Dictionary = observation.component(place, "calendar-day")
	if not day.is_empty():
		_read_day(day)
	var light: Dictionary = observation.component(place, "calendar-light")
	if not light.is_empty() and _has_day:
		_light_phase = String(light.get("phase", ""))
		_light_at = observation.at()


func anchored() -> bool:
	return _anchored


func paused() -> bool:
	return _paused


func time_scale() -> int:
	return _scale


func has_calendar() -> bool:
	return _has_day


## The estimate of the world's instant at `now_usec`, whole seconds.
func estimate(now_usec: int) -> int:
	if not _anchored:
		return 0
	if _paused:
		return _anchor_at
	# 64-bit: elapsed × 3 600 overflows only after some eighty years of one session.
	var elapsed := maxi(0, now_usec - _anchor_usec)
	return _anchor_at + elapsed * _scale / 1000000


## The instant to show: never smaller than one already shown in this session (§3.3).
func display_seconds(now_usec: int) -> int:
	var shown := maxi(_last_display, estimate(now_usec))
	_last_display = shown
	return shown


## `{year, month, day, weekday}` at `seconds`, or `{}` without a record. Past the record's day the date
## rolls by the civil successor, one day per whole day.
func date_at(seconds: int) -> Dictionary:
	if not _has_day:
		return {}
	var days := maxi(0, floori(float(seconds - _day_start) / DAY))
	var date: Dictionary = _date.duplicate()
	for _i in days:
		date = MineWorldCivil.next_date(date)
	date["weekday"] = (_weekday + days) % 7
	return date


func seconds_into_day(seconds: int) -> int:
	return posmod(seconds, DAY)


## The sun at `seconds`: `{elevation_deg, azimuth_deg, valid}`, interpolated over the record's track,
## azimuth unwrapped; outside the record's day the nearest sample, held, `valid` false.
func sun_at(seconds: int) -> Dictionary:
	if not _has_day or _track.size() < SAMPLES:
		return {"elevation_deg": -90.0, "azimuth_deg": 0.0, "valid": false}
	var offset := seconds - _day_start
	if offset < 0:
		return _sample(0, false)
	if offset >= (SAMPLES - 1) * SAMPLE_INTERVAL:
		return _sample(SAMPLES - 1, offset == (SAMPLES - 1) * SAMPLE_INTERVAL)
	var i := offset / SAMPLE_INTERVAL
	var f := float(offset % SAMPLE_INTERVAL) / SAMPLE_INTERVAL
	var e0 := int(_track[i].get("elevation", 0))
	var e1 := int(_track[i + 1].get("elevation", 0))
	var a0 := int(_track[i].get("azimuth", 0))
	var d := int(_track[i + 1].get("azimuth", 0)) - a0
	if d > 180000:
		d -= 360000
	elif d < -180000:
		d += 360000
	return {"elevation_deg": (e0 + (e1 - e0) * f) / 1000.0,
		"azimuth_deg": fposmod((a0 + d * f) / 1000.0, 360.0), "valid": true}


## The light phase at `seconds`: the disclosed phase, advanced across the record's own event instants.
func phase_at(seconds: int) -> String:
	if not _has_day or _light_phase == "":
		return ""
	var from := _light_at - _day_start
	var to := mini(seconds - _day_start, DAY)
	var phase := _light_phase
	for change in _changes:
		if change[0] > from and change[0] <= to:
			phase = change[1]
	return phase


func _anchor(at: int, now_usec: int) -> void:
	_anchored = true
	_anchor_at = at
	_anchor_usec = now_usec


func _sample(index: int, valid: bool) -> Dictionary:
	var s: Dictionary = _track[index]
	return {"elevation_deg": int(s.get("elevation", 0)) / 1000.0,
		"azimuth_deg": int(s.get("azimuth", 0)) / 1000.0, "valid": valid}


func _read_day(day: Dictionary) -> void:
	var date: Variant = day.get("date", {})
	var track: Variant = day.get("track", [])
	if typeof(date) != TYPE_DICTIONARY or typeof(track) != TYPE_ARRAY:
		return
	_has_day = true
	_date = {"year": int(date.get("year", 1)), "month": int(date.get("month", 1)),
		"day": int(date.get("day", 1))}
	_weekday = int(day.get("weekday", 0))
	_day_start = int(day.get("day_start", 0))
	_track = track
	_changes.clear()
	var events: Variant = day.get("events", {})
	if typeof(events) == TYPE_DICTIONARY:
		for pair in LIGHT_EVENTS:
			var at: Variant = events.get(pair[0])
			if at != null:
				_changes.append([int(at), pair[1]])
	# Stable by time: two events at one second keep the listed order.
	var ordered: Array = []
	for change in _changes:
		var k := ordered.size()
		while k > 0 and ordered[k - 1][0] > change[0]:
			k -= 1
		ordered.insert(k, change)
	_changes = ordered
