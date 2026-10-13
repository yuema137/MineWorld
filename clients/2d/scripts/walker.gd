extends Node

## The player's body: a click or the keys become `move` requests, and the body follows the server.
##
## Acquisition, not rule (`ARC-47`). The distances computed here are request sizes (a stride of at
## most STRIDE_M, so the server's MAX_STRIDE is never exceeded by construction), a route (straight
## strides, through the doorway the world disclosed) and drawing. Whether a stride is allowed is the
## server's answer: a rejected or refused one ends the walk, and the next observation puts the body
## where the world says (step-11 §7.1 rule 4). Nothing is retried; nothing is argued with.
##
## Positions are plan metres (`projection.gd`); requests are built in the place's own millimetres by
## `town.local_in` and sent by `intents.gd`, the only script that submits.

signal walk_ended(completed: bool)
## The world put the body somewhere the walk did not take it, and the body followed (rule 4).
signal reconciled(place: String, plan: Vector2)

## The longest stride asked for, in metres: under the server's MAX_STRIDE of 2 m (`PROTOCOL.md` §6.2).
const STRIDE_M := 1.9
## How fast the body is drawn walking, metres per second.
const WALK_SPEED := 1.6
## With the keys: report after this much travel or this much turn (the 3D slice's cadence).
const REPORT_M := 0.5
const REPORT_TURN_DEG := 20.0
## The server put the observer this far from the last accepted position: follow it (rule 4).
const RECONCILE_M := 0.15
## The body has reached the stride it asked for (drawing only).
const ARRIVED_M := 0.02
## A frame computed before an accepted move: the old position to the millimetre, within this window.
const STALE_M := 0.002
const STALE_WINDOW_MS := 600
## A click this close to a disclosed doorway is a click on the door (it is drawn 1.1 m wide).
const DOOR_PICK_M := 0.8
## Walking with the keys this close to a doorway steps through it; doorways arm again once the body
## is DOOR_REARM_M from where it last crossed, so arriving through one does not bounce back.
const DOOR_STEP_M := 0.6
const DOOR_REARM_M := 1.0
## "Walk to <person>" stops this far short of where they are drawn: "a pace away", a destination the
## player chose (step-13 D-b-3, QS13b-6). Not any pack's range: the server answers every stride, and
## the next action, whatever it is.
const APPROACH_M := 1.2

var town
var intents: Node
var projection

## Where the body is drawn, and in which place's frame it stands.
var body_plan := Vector2.ZERO
var body_place := ""
## The last position the world confirmed: by an accepted request or by an observation.
var accepted_plan := Vector2.ZERO
var accepted_place := ""
var facing := Vector2(0, 1)
var placed := false

## Each step: `{place, plan, crossing: bool}`. The head is in flight once `_token` is set.
var _route: Array = []
var _token := ""
var _step: Dictionary = {}
## Where the body is drawn walking to: the stride last asked for, or where the world put it.
var _goal := Vector2.ZERO
var _reconcile_next := false
var _reported_facing := Vector2(0, 1)
## The facing last asked for: a crossing (where `here` and `there` are one point) keeps it.
var _asked_facing := Vector2(0, 1)
## Where the body stood before the last accepted move, and until when a frame showing exactly that
## is taken as computed before the move (see [method observe]).
## How many such frames were ignored (reported by the drive, so the race is seen when it happens).
var stale_ignored := 0
var _stale_place := ""
var _stale_plan := Vector2.ZERO
var _stale_until_ms := 0
## Whether walking onto a doorway with the keys steps through it (see DOOR_REARM_M).
var _doors_armed := true
var _crossed_at := Vector2.ZERO
## The newest observation: where the people a walk may approach are.
var _latest: MineWorldObservation = null


## The world's word on where the observer is: placed on first sight, followed when it differs.
func observe(observation: MineWorldObservation) -> void:
	_latest = observation
	var place := observation.place()
	var plan: Variant = town.to_plan(place, observation.self_location().get("local"))
	if plan == null:
		return
	# An observation the server computed before applying the move it then accepted can arrive after
	# the result. It shows exactly the position the move started from; following it would snap the
	# body back. Such a frame is ignored for a short while after an accepted result — a real
	# displacement shows somewhere else, and is followed.
	if Time.get_ticks_msec() < _stale_until_ms and place == _stale_place \
			and (plan as Vector2).distance_to(_stale_plan) < STALE_M:
		stale_ignored += 1
		return
	if not placed or _reconcile_next or (_token == "" and (place != accepted_place
			or (plan as Vector2).distance_to(accepted_plan) > RECONCILE_M)):
		# Reconciliation on difference: one rule for a stride stopped short, a refusal, a nudge
		# and a teleport. The body goes where the world says; no request answers it.
		var was_placed := placed
		body_plan = plan
		_goal = plan
		body_place = place
		accepted_plan = plan
		accepted_place = place
		placed = true
		_reconcile_next = false
		if was_placed:
			reconciled.emit(place, plan)

## Walks to a point the player clicked (plan metres), through doorways when it lies in another place.
## A click on a disclosed doorway of the place the body is in means "go through it" (F-10): the
## route ends at the doorway's `there`, in the place it leads to.
func walk_to_plan(plan: Vector2) -> void:
	var door := _doorway_near(plan, DOOR_PICK_M)
	if not door.is_empty():
		walk_to(door["to"], town.to_plan(door["to"], {"x": door["there"].x, "y": door["there"].y}))
		return
	walk_to(town.place_at(plan, body_place), plan)


## Walks toward a perceived person, stopping APPROACH_M short of where the newest observation puts
## them: the menu's "walk to" entry (D-b-3). Ordinary strides; nothing else is decided here.
func approach(id: String) -> void:
	if _latest == null:
		return
	var location := _latest.location_of(id)
	var place := body_place
	if typeof(location.get("place")) == TYPE_DICTIONARY:
		place = String(location["place"].get("entity", body_place))
	var them: Variant = town.to_plan(place, location.get("local"))
	if them == null:
		return
	var from: Vector2 = _goal if place == body_place else body_plan
	var gap: Vector2 = (them as Vector2) - from
	if gap.length() <= APPROACH_M:
		return
	walk_to(place, (them as Vector2) - gap.normalized() * APPROACH_M)


## The disclosed doorway of the body's place whose `here` lies within `radius` of `plan`, or {}.
func _doorway_near(plan: Vector2, radius: float) -> Dictionary:
	for p in town.passages.get(body_place, []):
		var here: Variant = town.to_plan(body_place, {"x": p["here"].x, "y": p["here"].y})
		if here != null and town.to_plan(p["to"], {"x": p["there"].x, "y": p["there"].y}) != null \
				and (here as Vector2).distance_to(plan) <= radius:
			return p
	return {}


## Walks to `plan` in `place`. Ends any walk in progress after its request in flight is answered.
func walk_to(place: String, plan: Vector2) -> void:
	var route: Array = []
	var cur := body_place
	var pos: Vector2 = _goal
	for next in _places_between(cur, place):
		var p: Dictionary = town.passage(cur, next)
		var door: Vector2 = town.to_plan(cur, {"x": p["here"].x, "y": p["here"].y})
		route.append_array(_strides(cur, pos, door))
		route.append({"place": next, "plan": town.to_plan(next, {"x": p["there"].x, "y": p["there"].y}), "crossing": true})
		pos = door
		cur = next
	if cur != place:
		return
	route.append_array(_strides(cur, pos, plan))
	_route = route


func is_walking() -> bool:
	return _token != "" or not _route.is_empty() or body_plan.distance_to(_goal) > ARRIVED_M


## Stops after the request in flight, if any. The world is not asked to undo anything.
func stop() -> void:
	_route.clear()


## A connection ended: abandon the walk, and do not replay what was in flight (§4.6 point 5).
func abandon() -> String:
	var unanswered := _token
	_route.clear()
	_token = ""
	_step = {}
	_reconcile_next = true
	return unanswered


## The world answered a request. Only the one in flight matters to the walk.
func resolved(token: String, result: Dictionary) -> void:
	if token != _token:
		return
	_token = ""
	if result.has("accepted"):
		_stale_place = accepted_place
		_stale_plan = accepted_plan
		_stale_until_ms = Time.get_ticks_msec() + STALE_WINDOW_MS
		accepted_plan = _step["plan"]
		accepted_place = _step["place"]
		body_place = accepted_place
		if _step.get("crossing", false):
			_doors_armed = false
			_crossed_at = accepted_plan
		_step = {}
		if _route.is_empty():
			walk_ended.emit(true)
	else:
		_end_refused()


## The frame was refused (not a request at all): the walk ends the same way.
func refused(token: String) -> void:
	if token != "" and token == _token:
		_token = ""
		_end_refused()


func _end_refused() -> void:
	_route.clear()
	_step = {}
	_reconcile_next = true
	walk_ended.emit(false)


func _process(delta: float) -> void:
	if not placed:
		return
	var dt := minf(delta, 1.0 / 30.0)
	var keys := _keys()
	if keys != Vector2.ZERO:
		_route.clear()
		_walk_keys(keys, dt)
		return
	# The next stride is asked for once the last one is answered and the body has reached it, so each
	# request starts from the position the world last accepted.
	if _token == "" and not _route.is_empty() and body_plan.distance_to(_goal) < ARRIVED_M:
		_send(_route.pop_front())
	var next := body_plan.move_toward(_goal, WALK_SPEED * dt)
	if next != body_plan:
		facing = (next - body_plan).normalized()
	body_plan = next


func _send(step: Dictionary) -> void:
	# Facing is the direction from the last position asked for, never from where the drawn body
	# happens to be this frame: a request must not depend on frame timing (step-13 I-5).
	var direction: Vector2 = (step["plan"] - _goal)
	if direction.length() > 0.001:
		_asked_facing = direction.normalized()
	var local: Dictionary = town.local_in(step["place"], step["plan"])
	_token = intents.move(step["place"], local, MineWorldSpace.yaw_from_2d_radians(_asked_facing.angle()))
	_step = step if _token != "" else {}
	if _token == "":
		_route.clear()
	else:
		_goal = step["plan"]


## The keys: the body moves continuously and reports a `move` every REPORT_M or REPORT_TURN_DEG,
## never letting it run more than a stride ahead of the last accepted position (the reporting rule).
func _walk_keys(screen_dir: Vector2, dt: float) -> void:
	var direction: Vector2 = projection.screen_dir_to_plan(screen_dir)
	facing = direction
	_asked_facing = direction
	var next := body_plan + direction * WALK_SPEED * dt
	if not _doors_armed and body_plan.distance_to(_crossed_at) > DOOR_REARM_M:
		_doors_armed = true
	if _doors_armed and _token == "":
		# Walking onto a disclosed doorway: ask for the crossing to its `there` (F-10).
		var door := _doorway_near(next, DOOR_STEP_M)
		if not door.is_empty():
			_send({"place": door["to"], "plan": town.to_plan(door["to"], {"x": door["there"].x, "y": door["there"].y}), "crossing": true})
			return
	if next.distance_to(accepted_plan) <= STRIDE_M:
		var into: String = town.place_at(next, body_place)
		var through: Dictionary = town.passage(body_place, into) if into != body_place else {}
		if not through.is_empty() and _token == "" and _doors_armed:
			# Stepping into another place: ask for the crossing through its disclosed doorway.
			var door: Vector2 = town.to_plan(body_place, {"x": through["here"].x, "y": through["here"].y})
			if body_plan.distance_to(door) <= STRIDE_M:
				_send({"place": into, "plan": town.to_plan(into, {"x": through["there"].x, "y": through["there"].y}), "crossing": true})
				return
		body_plan = next
		_goal = next
	if _token == "":
		var travelled := body_plan.distance_to(accepted_plan)
		var turned := rad_to_deg(absf(facing.angle_to(_reported_facing)))
		if travelled >= REPORT_M or turned >= REPORT_TURN_DEG:
			_reported_facing = facing
			_send({"place": body_place, "plan": body_plan, "crossing": false})


func _keys() -> Vector2:
	return Vector2(
		Input.get_action_strength("move_right") - Input.get_action_strength("move_left"),
		Input.get_action_strength("move_down") - Input.get_action_strength("move_up"))


## The places a walk passes through to reach `to`, after `from`: [] (same place), [to] (a door
## between them), or [hub, to] (out onto the hub and in again). Unknown ways give no route.
func _places_between(from: String, to: String) -> Array:
	if from == to:
		return []
	if not town.passage(from, to).is_empty():
		return [to]
	var hub: String = town.hub()
	if hub != "" and not town.passage(from, hub).is_empty() and not town.passage(hub, to).is_empty():
		return [hub, to]
	return []


## Equal strides of at most STRIDE_M from `a` to `b` in `place`.
func _strides(place: String, a: Vector2, b: Vector2) -> Array:
	var out: Array = []
	# The tolerance keeps float noise from adding a stride: 3.8 m / 1.9 m is 2.0000000002 in floats.
	var n := int(ceil(a.distance_to(b) / STRIDE_M - 1e-6))
	for i in range(1, n + 1):
		out.append({"place": place, "plan": a.lerp(b, float(i) / n), "crossing": false})
	return out
