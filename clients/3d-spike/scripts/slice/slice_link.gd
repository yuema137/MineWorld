## The slice as a MineWorld presentation: the one place it talks to a world.
##
## Design: `.structured-coding/plans/vis-3d-godot-2/pr-01a-slice.md` sec.7a.
## Transport, frames, identities and axes are all `clients/protocol/mineworld`
## (adopted by symlink, `ADOPTION.md` sec.1). This file adds only what is the
## slice's own: where the world's café frame sits in this scene, which perceived
## people to draw, and when to report the player's position.
##
## It is the ONLY slice file that names an action type. When S6 retires
## `arrive` for `move`, the change is `MOVE_ACTION` and `_move_payload` below.
##
## What it never does (`ADOPTION.md` sec.3.3): decide whether an action is
## allowed, compare positions to decide whether to submit, or keep a list of
## what actions exist. It reports intent; the server answers.
class_name SliceLink
extends Node

## The presence pack's action on main @ a594164. S6 replaces it with `move`.
const MOVE_ACTION := "arrive"
## Report the body when it has moved this far or turned this much, at most this
## often. S6's reporting rule (report before travelling MAX_STRIDE) will set the
## distance; 0.30 m is well inside any stride bound.
const MOVE_DIST := 0.30
const MOVE_TURN := deg_to_rad(20.0)
const MOVE_EVERY := 0.4
## A jump moves the body in height only (`Player.JUMP_HEIGHT`, 0.4 s). Height
## is not reported: the world's café frame is a floor, and reporting a body
## 0.45 m in the air would be reporting a rendered moment as a world fact.
const REPORT_HEIGHT := false

## The world's café frame in this scene: its origin is the room's inner
## front-west corner, +x east, +y into the room. A stated binding, and a
## recorded discrepancy -- see the design sec.7a: no single origin puts the World
## Pack's door on the slice's door while keeping its people inside the room.
static func cafe_origin() -> Vector3:
	return Vector3(6.0 - SliceCafe.W * 0.5 + SliceCafe.WALL_T,
		SliceStreet.WALK_Y + SliceCafe.FLOOR_Y,
		SliceStreet.NORTH_FACE - SliceCafe.WALL_T)

signal said(text: String)

var client: MineWorldClient
var player: SlicePlayer
var world_root: Node3D

var cafe_place := ""               ## the café's identity, as the server named it
var figures := {}                  ## EntityId string -> the figure drawn for it
var last_sent_local := {}          ## the last position reported, as sent
var answers: Array[Dictionary] = []

var _last_pos := Vector3.INF
var _last_yaw := 0.0
var _since := 0.0
var _tokens := {}                  ## token -> action type, for the transcript


static func address_from_args() -> String:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--server="):
			return a.substr(9)
	return ""


static func seat_from_args() -> String:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--seat="):
			return a.substr(7)
	return "visitor"


func start(address: String, seat: String) -> void:
	client = MineWorldClient.new()
	client.name = "MineWorldClient"
	add_child(client)
	client.welcomed.connect(_on_welcomed)
	client.observed.connect(_on_observed)
	client.resolved.connect(_on_resolved)
	client.refused.connect(_on_refused)
	client.disconnected.connect(func(reason: String) -> void: _say("disconnected: %s" % reason))
	client.connect_to_world(address, seat)
	_say("connecting to %s as %s" % [address, seat])


func _say(text: String) -> void:
	print("[link] " + text)
	said.emit(text)


func _on_welcomed(seat: String, observer: String, world: Dictionary) -> void:
	_say("seated as %s -- observer %s, world %s" % [seat, observer, world.get("instance", "?")])


## Reconcile the scene with the newest view: an observation is exhaustive, so
## a person no longer listed is no longer perceived and their figure goes.
func _on_observed(obs: MineWorldObservation) -> void:
	if cafe_place == "" and obs.place() != "":
		cafe_place = obs.place()
		_say("in place %s (drawn as the café)" % cafe_place)
	var seen := {}
	for id in obs.ids():
		if id == obs.observer():
			continue
		var loc := obs.location_of(id)
		if loc.is_empty() or typeof(loc.get("place")) != TYPE_DICTIONARY:
			continue
		if String(loc["place"].get("entity", "")) != cafe_place:
			continue
		seen[id] = true
		var fig: Node3D = figures.get(id)
		if fig == null:
			fig = _figure(id, obs)
			figures[id] = fig
		fig.global_position = to_scene(loc.get("local"))
		fig.rotation.y = MineWorldSpace.yaw_to_3d_radians(loc.get("facing")) + PI
	for id in figures.keys():
		if not seen.has(id):
			(figures[id] as Node).queue_free()
			figures.erase(id)


## A perceived person, drawn from the same stand-in mannequin as the street's
## figures. Appearance is chosen here from tags, which are world data.
func _figure(id: String, obs: MineWorldObservation) -> Node3D:
	var rng := RandomNumberGenerator.new()
	rng.seed = hash(id)
	var n := NPC.make(rng, NPC.Pose.STAND, 1.72, false)
	n.name = "Person_" + id
	n.set_meta("entity_id", id)       # a string, never a number (ADOPTION.md sec.3.1)
	world_root.add_child(n)
	var tags: Array = obs.entity(id).get("tags", [])
	var label := Label3D.new()
	label.text = ", ".join(tags) if not tags.is_empty() else id
	label.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	label.position = Vector3(0, 2.05, 0)
	label.font_size = 48
	label.pixel_size = 0.004
	n.add_child(label)
	_say("perceive %s (%s)" % [id, label.text])
	return n


## World millimetres in the café frame -> a point in this scene.
static func to_scene(local: Variant) -> Vector3:
	var p := MineWorldSpace.to_3d(local)
	return cafe_origin() + Vector3(p.x, 0.0, p.z)


## A point in this scene -> world millimetres in the café frame.
static func to_world(p: Vector3) -> Dictionary:
	var rel := p - cafe_origin()
	if not REPORT_HEIGHT:
		rel.y = 0.0
	return MineWorldSpace.from_3d(rel)


func _physics_process(delta: float) -> void:
	if client == null or not client.is_seated() or cafe_place == "" or player == null:
		return
	_since += delta
	# Only inside the café: social-cafe models no street, so out there there is
	# no world place to report a position in. This is "nothing to say", not a
	# rule about what is allowed.
	if SliceWorld.place_at(world_root, player.global_position) != SliceWorld.CAFE_PLACE:
		return
	var p := player.global_position
	var flat := Vector2(p.x - _last_pos.x, p.z - _last_pos.z).length() if _last_pos != Vector3.INF \
		else INF
	var turned := absf(angle_difference(player.rotation.y, _last_yaw))
	if _since < MOVE_EVERY or (flat < MOVE_DIST and turned < MOVE_TURN):
		return
	report_position()


## Report where the body is now. Prediction is local; the server is the
## authority and its next observation is the answer (`ADOPTION.md` sec.4).
func report_position() -> String:
	var p := player.global_position
	_last_pos = p
	_last_yaw = player.rotation.y
	_since = 0.0
	var local := to_world(p)
	last_sent_local = local
	var tok := client.submit(MOVE_ACTION, null, _move_payload(local), _location(local))
	_tokens[tok] = MOVE_ACTION
	return tok


func _move_payload(local: Dictionary) -> Dictionary:
	return { "location": _location(local) }


func _location(local: Dictionary) -> Dictionary:
	return MineWorldSpace.location(cafe_place, local,
		MineWorldSpace.yaw_from_3d_radians(player.rotation.y))


## Talk to whoever the camera is facing. Targeting is presentation; whether a
## conversation may happen is the server's, and the request is sent regardless
## of what the affordance says (ADOPTION.md sec.3.3).
func talk_to_facing(utterance: String) -> String:
	var target := facing_person()
	if target == "":
		_say("nobody in view to talk to")
		return ""
	var obs := client.latest
	if obs != null and not obs.may("talk", target):
		_say("the world says talk to %s is unavailable: %s"
			% [target, obs.unavailable_reason("talk", target)])
	var tok := client.submit("talk", target, { "utterance": utterance },
		_location(to_world(player.global_position)))
	_tokens[tok] = "talk"
	return tok


func facing_person() -> String:
	var cam := player.get_viewport().get_camera_3d()
	if cam == null:
		return ""
	var fwd := -cam.global_transform.basis.z
	var best := ""
	var best_dot := 0.80
	for id in figures:
		var to: Vector3 = (figures[id] as Node3D).global_position + Vector3(0, 1.4, 0) \
			- cam.global_position
		var d := fwd.dot(to.normalized())
		if d > best_dot:
			best_dot = d
			best = id
	return best


func _on_resolved(token: String, action_id: String, result: Dictionary) -> void:
	var kind: String = result.keys()[0] if not result.is_empty() else "?"
	var what: String = _tokens.get(token, "?")
	answers.append({ "token": token, "action": what, "action_id": action_id, "result": kind,
		"detail": result })
	if kind == "accepted" and what == MOVE_ACTION:
		return  # the steady stream of position reports; not worth a toast
	_say("%s %s -> %s %s" % [what, token, kind,
		"" if kind == "accepted" else JSON.stringify(result.get(kind))])


func _on_refused(code: String, token: String, _detail: String) -> void:
	answers.append({ "token": token, "action": _tokens.get(token, "?"), "result": "refused",
		"code": code })
	_say("refused %s: %s" % [token, code])


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo \
			and (event as InputEventKey).physical_keycode == KEY_E and client != null:
		talk_to_facing("Hello! A coffee, please.")
