extends Node

## The player's body: a click becomes a `walk-to`, the keys become `move` requests, and the body follows
## the server.
##
## Acquisition, not rule (`ARC-47`). Going somewhere — a click on the floor or on a doorway, "Walk to
## <name>" — is one `walk-to` request: the server plans the route round the place's walls, furniture and
## loose objects and keeps it as the player's own `walking` record (`ARC-75`, step-11 SD-N12). This
## client plans no route and splits nothing. While that record is disclosed it sends one `walk-step` (no
## payload) per wall second, the pacing `clients/protocol/ADOPTION.md` §4.2 states, and stops when the
## record is gone (arrived, stalled, no route, replaced). Each answer is the server's: a refused step or
## walk ends the walk, and the next observation puts the body where the world says (step-11 §7.1 rule
## 4). Nothing is retried; nothing is argued with.
##
## The keys are embodied input, not a walk: the body moves continuously and reports a `move` every
## REPORT_M or REPORT_TURN_DEG, never running more than a stride ahead of the last accepted position. A
## `move` of the player's own ends any walk on the server, so the keys end the paced walk here too.
##
## Positions are plan metres (`projection.gd`); requests are built in the place's own millimetres by
## `town.local_in` and sent by `intents.gd`, the only script that submits.

signal walk_ended(completed: bool)
## The world put the body somewhere the client did not ask for, and the body followed (rule 4).
signal reconciled(place: String, plan: Vector2)

## The longest stride the keys run ahead of the last accepted position, in metres: under the server's
## MAX_STRIDE of 2 m (`PROTOCOL.md` §6.2).
const STRIDE_M := 1.9
## How fast the body is drawn walking, metres per second: above the 1.34 m a paced walk-step carries,
## so the drawn body reaches each confirmed position before the next one arrives.
const WALK_SPEED := 1.6
## With the keys: report after this much travel or this much turn (the 3D slice's cadence).
const REPORT_M := 0.5
const REPORT_TURN_DEG := 20.0
## The server put the observer this far from the last accepted position: follow it (rule 4).
const RECONCILE_M := 0.15
## The body has reached the position it is drawn walking to (drawing only).
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
## One `walk-step` per this many wall milliseconds (ADOPTION.md §4.2): pacing, never a world rule.
const STEP_INTERVAL_MS := 1000
## A paced walk whose observations stop reflecting its last request is not walked blind: after this
## many wall milliseconds without one, the walk ends here (the SDK pacer's UNSEEN_PATIENCE).
const UNSEEN_PATIENCE_MS := 5000
## A position the world gives during a paced walk farther than this from the drawn body is jumped to,
## not walked to: the world put the body there some other way (two strides and more).
const JUMP_M := 3.0

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

## The request in flight, if any (one at a time), and what it was: `{kind, place?, plan?, crossing?}`,
## kind "walk" (the walk asked for), "pace" (one paced step of it) or "stride" (a key-driven move).
var _token := ""
var _step: Dictionary = {}
## A walk asked for while a request was in flight: its `walk-to` destination, sent once that is answered.
var _wanted: Dictionary = {}
## Whether a `walk-to` was accepted and the walk is being paced.
var _pacing := false
## The action of the walk's last accepted request: an observation reflects it once its
## `acted_through` has reached it (`PROTOCOL.md` §5).
var _awaited_action := ""
var _next_step_ms := 0
var _unseen_since_ms := -1
## Where the body is drawn walking to: the position the world last gave, or the keys' own point.
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
## The newest observation: whether my own walk is disclosed, and where the world says I am.
var _latest: MineWorldObservation = null


## The world's word on where the observer is: placed on first sight, followed when it differs.
func observe(observation: MineWorldObservation) -> void:
	_latest = observation
	var place := observation.place()
	var plan: Variant = town.to_plan(place, observation.self_location().get("local"))
	if plan == null:
		return
	if _pacing and placed and not _reconcile_next:
		_follow_walk(observation, place, plan)
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
		# Reconciliation on difference: one rule for a refusal, a nudge and a teleport. The body goes
		# where the world says; no request answers it.
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


## During a paced walk every position the world gives is where the walk has taken the body: it is drawn
## walking there. Frames computed before the walk's last request say nothing new and are skipped.
func _follow_walk(observation: MineWorldObservation, place: String, plan: Vector2) -> void:
	if not _reflects_last(observation):
		stale_ignored += 1
		return
	accepted_plan = plan
	accepted_place = place
	body_place = place
	_goal = plan
	if body_plan.distance_to(plan) > JUMP_M:
		body_plan = plan
		reconciled.emit(place, plan)


## Whether `observation` was computed after the walk's last accepted request.
func _reflects_last(observation: MineWorldObservation) -> bool:
	var acted: Variant = observation.acted_through()
	return acted != null and String(acted).to_int() >= _awaited_action.to_int()


## Walks to a point the player clicked (plan metres), through doorways when it lies in another place.
## A click on a disclosed doorway of the place the body is in means "go through it" (F-10): the walk
## ends at the doorway's `there`, in the place it leads to.
func walk_to_plan(plan: Vector2) -> void:
	var door := _doorway_near(plan, DOOR_PICK_M)
	if not door.is_empty():
		walk_to(door["to"], town.to_plan(door["to"], {"x": door["there"].x, "y": door["there"].y}))
		return
	walk_to(town.place_at(plan, body_place), plan)


## Walks to a perceived person: the menu's "walk to" entry (D-b-3). The server takes the walk to within
## its own approach distance of them, and follows them if they move (`ARC-75`).
func approach(id: String) -> void:
	_ask_walk({"person": {"entity": id, "entity_type": "person"}})


## The disclosed doorway of the body's place whose `here` lies within `radius` of `plan`, or {}.
func _doorway_near(plan: Vector2, radius: float) -> Dictionary:
	for p in town.passages.get(body_place, []):
		var here: Variant = town.to_plan(body_place, {"x": p["here"].x, "y": p["here"].y})
		if here != null and town.to_plan(p["to"], {"x": p["there"].x, "y": p["there"].y}) != null \
				and (here as Vector2).distance_to(plan) <= radius:
			return p
	return {}


## Walks to `plan` in `place`: one `walk-to`; the server finds the way. Ends any walk in progress.
func walk_to(place: String, plan: Vector2) -> void:
	if place == "":
		return
	_ask_walk({"place": MineWorldSpace.location(place, town.local_in(place, plan))})


func _ask_walk(destination: Dictionary) -> void:
	_pacing = false
	_wanted = destination


func is_walking() -> bool:
	return _token != "" or _pacing or not _wanted.is_empty() \
		or body_plan.distance_to(_goal) > ARRIVED_M


## Stops after the request in flight, if any. The world is not asked to undo anything: the walk is
## no longer paced, and the server's record of it lapses unwalked.
func stop() -> void:
	_wanted = {}
	_end_walk(true)


## A connection ended: abandon the walk, and do not replay what was in flight (§4.6 point 5).
func abandon() -> String:
	var unanswered := _token
	_wanted = {}
	_pacing = false
	_token = ""
	_step = {}
	_reconcile_next = true
	return unanswered


## The world answered a request. Only the one in flight matters to the walker.
func resolved(token: String, result: Dictionary, action_id: String = "") -> void:
	if token != _token:
		return
	_token = ""
	var step := _step
	_step = {}
	var accepted := result.has("accepted")
	match String(step.get("kind", "")):
		"walk":
			if accepted and _wanted.is_empty():
				_pacing = true
				_awaited_action = action_id
				_next_step_ms = Time.get_ticks_msec()
				_unseen_since_ms = -1
			elif not accepted:
				_refused_walk()
		"pace":
			if accepted:
				_awaited_action = action_id
			else:
				_refused_walk()
		_:
			if accepted:
				_stale_place = accepted_place
				_stale_plan = accepted_plan
				_stale_until_ms = Time.get_ticks_msec() + STALE_WINDOW_MS
				accepted_plan = step["plan"]
				accepted_place = step["place"]
				body_place = accepted_place
				if step.get("crossing", false):
					_doors_armed = false
					_crossed_at = accepted_plan
			else:
				_reconcile_next = true


## The frame was refused (not a request at all): the walk ends the same way.
func refused(token: String) -> void:
	if token != "" and token == _token:
		_token = ""
		var kind := String(_step.get("kind", ""))
		_step = {}
		if kind == "walk" or kind == "pace":
			_refused_walk()
		else:
			_reconcile_next = true


func _refused_walk() -> void:
	_reconcile_next = true
	_end_walk(false)


func _end_walk(completed: bool) -> void:
	if not _pacing:
		if not completed:
			walk_ended.emit(false)
		return
	_pacing = false
	walk_ended.emit(completed)


func _process(delta: float) -> void:
	if not placed:
		return
	var dt := minf(delta, 1.0 / 30.0)
	var keys := _keys()
	if keys != Vector2.ZERO:
		# A move of my own ends the walk on the server; the keys end its pacing here.
		_wanted = {}
		_end_walk(true)
		_walk_keys(keys, dt)
		return
	if _token == "":
		if not _wanted.is_empty():
			var destination := _wanted
			_wanted = {}
			_send({"kind": "walk"}, intents.walk_to(destination))
		elif _pacing:
			_pace()
	var next := body_plan.move_toward(_goal, WALK_SPEED * dt)
	if next != body_plan:
		facing = (next - body_plan).normalized()
	body_plan = next


## One `walk-step` per wall second while my own `walking` record is disclosed in an observation that
## reflects the walk's last request; the walk ends when that record is gone.
func _pace() -> void:
	var now := Time.get_ticks_msec()
	if now < _next_step_ms or _latest == null:
		return
	if not _reflects_last(_latest):
		if _unseen_since_ms < 0:
			_unseen_since_ms = now
		elif now - _unseen_since_ms >= UNSEEN_PATIENCE_MS:
			_end_walk(false)
		return
	_unseen_since_ms = -1
	if _latest.own_component("walking").is_empty():
		_end_walk(true)
		return
	_next_step_ms = now + STEP_INTERVAL_MS
	_send({"kind": "pace"}, intents.walk_step())


func _send(step: Dictionary, token: String) -> void:
	_token = token
	_step = step if token != "" else {}
	if token == "" and _pacing:
		_end_walk(false)


func _send_move(step: Dictionary) -> void:
	# Facing is the direction from the last position asked for, never from where the drawn body
	# happens to be this frame: a request must not depend on frame timing (step-13 I-5).
	var direction: Vector2 = (step["plan"] - _goal)
	if direction.length() > 0.001:
		_asked_facing = direction.normalized()
	var local: Dictionary = town.local_in(step["place"], step["plan"])
	step["kind"] = "stride"
	_send(step, intents.move(step["place"], local, MineWorldSpace.yaw_from_2d_radians(_asked_facing.angle())))
	if _token != "":
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
			_send_move({"place": door["to"], "plan": town.to_plan(door["to"], {"x": door["there"].x, "y": door["there"].y}), "crossing": true})
			return
	if next.distance_to(accepted_plan) <= STRIDE_M:
		var into: String = town.place_at(next, body_place)
		var through: Dictionary = town.passage(body_place, into) if into != body_place else {}
		if not through.is_empty() and _token == "" and _doors_armed:
			# Stepping into another place: ask for the crossing through its disclosed doorway.
			var door: Vector2 = town.to_plan(body_place, {"x": through["here"].x, "y": through["here"].y})
			if body_plan.distance_to(door) <= STRIDE_M:
				_send_move({"place": into, "plan": town.to_plan(into, {"x": through["there"].x, "y": through["there"].y}), "crossing": true})
				return
		body_plan = next
		_goal = next
	if _token == "":
		var travelled := body_plan.distance_to(accepted_plan)
		var turned := rad_to_deg(absf(facing.angle_to(_reported_facing)))
		if travelled >= REPORT_M or turned >= REPORT_TURN_DEG:
			_reported_facing = facing
			_send_move({"place": body_place, "plan": body_plan, "crossing": false})


func _keys() -> Vector2:
	return Vector2(
		Input.get_action_strength("move_right") - Input.get_action_strength("move_left"),
		Input.get_action_strength("move_down") - Input.get_action_strength("move_up"))
