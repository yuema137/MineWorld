## The correction rule: when the drawn body and the server's body part, and how
## the client gets back (step-11 §22 SD-E8, `docs/DECISIONS.md` ARC-50).
##
## The 3D client predicts exactly one thing -- that its own body is stopped by
## what stands in its way -- and reports where the body went (`PROTOCOL.md`
## §6.2). The server decides every stride. This file decides only when the
## client must stop believing its prediction and draw the server's answer:
##
##   reference  the point the client reported with the request the observation
##              reflects. EXACT form: the observation's `acted_through` names that
##              request (S11-C), and its reported point is looked up here. The
##              HEURISTIC form, for an `acted_through` this file did not see: the
##              point of the newest answered `move` (`baseline`), and every
##              point still in flight.
##   correct    when the server's position differs from the reference by more
##              than CORRECTION_MM (and, heuristically, from every point in
##              flight), and from where the body is drawn by more than
##              CORRECTION_MM too: the body goes to the server's position.
##   suspend    after a correction no `move` is reported until every request sent
##              before it has been answered and one further observation has
##              arrived; the rule is then applied once more, and reporting
##              resumes. A stride sent from the wrong position can still be
##              accepted after the correction; without the suspension it would
##              drag the body back, correction after correction.
##
## One number covers a stride the server stopped short, a nudge, a shove, a jam
## at a box and a refusal. The glide that draws a correction is `Player`'s; this
## file only says where to and how far. Every time is wall time (SD-E11): no
## clock of the world is read here.
class_name SliceCorrection
extends RefCounted

## ARC-50's one number, in millimetres (step-11 §7.1, §22.3.1): above the
## server's 1 mm quantization and its controller's 10 mm gap, below a visible
## jump; the same tolerance as the geometry probe and `SliceLayout.DOOR_MATCH`.
const CORRECTION_MM := 150.0

## token -> { point (mm, Vector2; Vector2.INF when none was reported), place,
## stride }, for every request not yet answered
var _in_flight := {}
## action id -> { point, place }: the point its request reported, or the
## server's position adopted for it by a correction
var _by_action := {}
## The point sent with the newest `move` whose answer has arrived, and its place.
var baseline := Vector2.INF
var baseline_place := ""
## Whether reporting is suspended, and what it waits for.
var suspended := false
var _waiting := {}
var _need_observation := false
## Every correction, in order: { at (wall s), size (m), from, to (mm), place,
## form ("exact" | "heuristic") }
var history: Array = []


## A request was sent, reporting the body at `point` (mm, `place`'s frame), or
## Vector2.INF when it reported no position. `is_move` marks the strides.
func sent(token: String, point: Vector2, place: String, is_move: bool) -> void:
	if token == "":
		return
	_in_flight[token] = {"point": point, "place": place, "stride": is_move}


## A request was answered: resolved with `action_id`, or refused before an
## identity was allocated (`action_id` "").
func answered(token: String, action_id: String) -> void:
	var entry: Variant = _in_flight.get(token)
	_in_flight.erase(token)
	_waiting.erase(token)
	if entry == null:
		return
	# Only a stride asks for a position. Any other request reports where the body
	# is but asks the server for nothing about it, so an observation that reflects
	# it is compared with the stride before it (step-11 §22.15 X12e-15).
	if not bool(entry["stride"]):
		if action_id != "" and baseline != Vector2.INF:
			_by_action[action_id] = {"point": baseline, "place": baseline_place}
		return
	if entry["point"] == Vector2.INF:
		return
	if action_id != "":
		_by_action[action_id] = {"point": entry["point"], "place": entry["place"]}
	baseline = entry["point"]
	baseline_place = entry["place"]


## The body was placed where the world says (the first view): every reference
## starts again from there.
func adopted(point: Vector2, place: String) -> void:
	baseline = point
	baseline_place = place
	_by_action.clear()
	suspended = false
	_waiting.clear()
	_need_observation = false


## May the client report a `move` now?
func may_report() -> bool:
	return not suspended


## One observation of `place`, the place the world puts the body in. `auth` is
## the server's position and `drawn` where the body is drawn (mm, `place`'s
## frame); `acted` the frame's `acted_through`, or null. Returns the point to
## correct to, or Vector2.INF.
func observe(place: String, auth: Vector2, drawn: Vector2, acted: Variant) -> Vector2:
	if suspended:
		if not _waiting.is_empty():
			return Vector2.INF
		if not _need_observation:
			_need_observation = true
			return Vector2.INF
		# every pre-correction request is answered and one more observation has
		# arrived: apply the rule once more, then resume
		suspended = false
		_need_observation = false
	var key := "" if acted == null else String(acted)
	var form := "heuristic"
	var differs := false
	var ref: Variant = _by_action.get(key)
	var compared := baseline
	if ref != null and String(ref["place"]) == place:
		form = "exact"
		compared = ref["point"]
		differs = auth.distance_to(ref["point"]) > CORRECTION_MM
	else:
		differs = baseline_place != place or auth.distance_to(baseline) > CORRECTION_MM
		if differs:
			for t in _in_flight:
				var e: Dictionary = _in_flight[t]
				if bool(e["stride"]) and e["point"] != Vector2.INF and String(e["place"]) == place \
						and auth.distance_to(e["point"]) <= CORRECTION_MM:
					differs = false
					break
	if not differs:
		return Vector2.INF
	# The world holds the body elsewhere than the request said: the reference is
	# the server's position from now on. Where the body is drawn already agrees
	# with it, there is nothing to draw and nothing is suspended (step-11 §22.15
	# X12e-3).
	if key != "":
		_by_action[key] = {"point": auth, "place": place}
	baseline = auth
	baseline_place = place
	if drawn.distance_to(auth) <= CORRECTION_MM:
		return Vector2.INF
	history.append({"at": Time.get_ticks_msec() / 1000.0, "size": drawn.distance_to(auth) / 1000.0,
		"from": drawn, "to": auth, "place": place, "form": form, "reference": compared, "acted": key})
	suspended = true
	_need_observation = false
	_waiting = {}
	for t in _in_flight:
		_waiting[t] = true
	return auth
