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


## The world's word on where the observer is: placed on first sight, followed when it differs.
func observe(observation: MineWorldObservation) -> void:
	var place := observation.place()
	var plan: Variant = town.to_plan(place, observation.self_location().get("local"))
	if plan == null:
		return
	if not placed or _reconcile_next or (_token == "" and (place != accepted_place
			or (plan as Vector2).distance_to(accepted_plan) > RECONCILE_M)):
		# Reconciliation on difference: one rule for a stride stopped short, a refusal, a nudge
		# and a teleport. The body goes where the world says; no request answers it.
		body_plan = plan
		_goal = plan
		body_place = place
		accepted_plan = plan
		accepted_place = place
		placed = true
		_reconcile_next = false


## Walks to a point the player clicked (plan metres), through doorways when it lies in another place.
func walk_to_plan(plan: Vector2) -> void:
	walk_to(town.place_at(plan, body_place), plan)


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
		accepted_plan = _step["plan"]
		accepted_place = _step["place"]
		body_place = accepted_place
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
	var direction: Vector2 = (step["plan"] - body_plan)
	if direction.length() > 0.001:
		facing = direction.normalized()
	var local: Dictionary = town.local_in(step["place"], step["plan"])
	_token = intents.move(step["place"], local, MineWorldSpace.yaw_from_2d_radians(facing.angle()))
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
	var next := body_plan + direction * WALK_SPEED * dt
	if next.distance_to(accepted_plan) <= STRIDE_M:
		var into: String = town.place_at(next, body_place)
		var through: Dictionary = town.passage(body_place, into) if into != body_place else {}
		if not through.is_empty() and _token == "":
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
	var n := int(ceil(a.distance_to(b) / STRIDE_M))
	for i in range(1, n + 1):
		out.append({"place": place, "plan": a.lerp(b, float(i) / n), "crossing": false})
	return out
