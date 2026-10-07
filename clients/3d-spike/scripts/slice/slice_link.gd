## The slice as a MineWorld presentation: the one place it talks to a world.
##
## Design: `.structured-coding/plans/vis-3d-godot-2/pr-01a-slice.md` sec.7a, and
## sec.7b for the move to S6's `move`. Transport, frames, identities and axes are
## all `clients/protocol/mineworld` (adopted by symlink, `ADOPTION.md` sec.1).
## This file adds only what is the slice's own: where each of the world's places
## sits in this scene, which perceived people to draw, and when to report the
## player's body.
##
## It is the ONLY slice file that names an action type.
##
## What it never does (`ADOPTION.md` sec.3.3): decide whether an action is
## allowed, compare positions to decide whether to submit, or keep a list of
## what actions exist. It reports intent; the server answers. In particular it
## does not decide whether the door may be crossed -- it reports the body on the
## far side, and the movement system answers.
class_name SliceLink
extends Node

## S6's one movement action (`DECISIONS.md` `ARC-26`; `server/PROTOCOL.md` sec.6.2).
const MOVE_ACTION := "move"

## THE REPORTING RULE (`PROTOCOL.md` sec.6.2): report a `move` before the body has
## travelled MAX_STRIDE (2 m) since the last position the server accepted. The
## body is reported every REPORT_DIST of travel, a quarter of that bound: at the
## controller's 3.10 m/s jog that is every ~0.16 s, against the rule's ~0.65 s.
## It is a request cadence, not a rule this client enforces -- the server decides
## every stride.
const REPORT_DIST := 0.50
const REPORT_TURN := deg_to_rad(20.0)
## The shortest gap between two reports, so a body standing still and turning
## does not flood the connection.
const REPORT_GAP := 0.08
## A jump moves the body in height only (`Player.JUMP_HEIGHT`). Height is never
## reported: the world's places are floors, and reporting a body 0.45 m in the
## air would be reporting a rendered moment as a world fact. So a jump straight
## up sends nothing, and a running jump sends the same strides a walk does.
const REPORT_HEIGHT := false

## THE SPATIAL BINDING -- where each of the world's places sits in this scene.
##
## World Packs author no geometry, but the world states its doorways, and since
## S7 the movement system DISCLOSES them: the observer's own place carries a
## `passages` component listing, for each way out, the place it leads to and
## the doorway's position on both sides. So this client learns from the world,
## not from a copy of the pack: which place the café's door opens onto (the
## place the slice draws as the street), and where that door is in each
## place's frame. Nothing about the pack is quoted here.
##
## The world frame is fixed (+x east, +y north, `CORE_CONCEPTS.md` sec.6.1), so a
## binding may only translate. Each place is bound so that its disclosed doorway
## lands on this slice's café door: the server decides a crossing by distance to
## that doorway, so it is the one point the binding must get right.
##
## Until S8 PR 10a (`main` `2f24eef`) the pack's café was the mirror of this
## one, and with the door aligned `alice` and `bob` were drawn just west of the
## café's west wall (design sec.7b). 10a authored the café to this slice's
## layout, so with the same binding they now stand inside the room.
##
## The two keys the slice draws. A place's key is learned from its tag (the
## world's data), and the street's also from being where the café's door leads.
const KEYS := ["cafe", "street"]
## The slice's own place volumes (`SliceWorld`) -> the pack's authoring keys,
## which the world also carries as each place's tag.
## The florist (`VISUAL_SLICE.md` sec.4.1) is a room the world does not model:
## `social-cafe` has no such place. So a body inside it is reported where the
## world can put it -- in the street's frame -- rather than not at all, which
## would leave the server's last position behind at the florist's door and
## refuse the first report after it.
const PLACE_KEY := {
	SliceWorld.CAFE_PLACE: "cafe",
	SliceWorld.STREET_PLACE: "street",
	SliceWorld.FLORIST_PLACE: "street",
}


## The slice's door, as the point each side of it the pack's doorway is bound
## to: 0.2 m into the room past the façade's inner face, and 0.2 m out onto the
## pavement past its outer face.
static func door_point(key: String) -> Vector3:
	var x := 6.0 + SliceCafe.DOOR_X
	if key == "cafe":
		return Vector3(x, SliceStreet.WALK_Y + SliceCafe.FLOOR_Y,
			SliceStreet.NORTH_FACE - SliceCafe.WALL_T - 0.2)
	return Vector3(x, SliceStreet.WALK_Y, SliceStreet.NORTH_FACE + 0.2)


## A place's origin in this scene: where its frame's (0, 0) lands, given the
## doorway the world disclosed for it.
func origin(key: String) -> Vector3:
	var d := MineWorldSpace.to_3d(doorway[key])
	return door_point(key) - Vector3(d.x, 0.0, d.z)


## Read the passages the world disclosed on the place the observer is in. The
## café's one passage names the street and both sides of the door; standing in
## the street, the street's names the café. Read, never computed: the frame is
## the world's (`MineWorldObservation.component`).
func _learn_passages(obs: MineWorldObservation, place: String, key: String) -> void:
	var leads: Variant = obs.component(place, "passages").get("leads_to", [])
	if typeof(leads) != TYPE_ARRAY:
		return
	for p in leads:
		if typeof(p) != TYPE_DICTIONARY or typeof(p.get("to")) != TYPE_DICTIONARY:
			continue
		var to := String(p["to"].get("entity", ""))
		var other := "street" if key == "cafe" else "cafe"
		if to == "" or (place_ids.has(other) and place_ids[other] != to):
			continue
		place_ids[other] = to
		if typeof(p.get("here")) == TYPE_DICTIONARY:
			doorway[key] = p["here"]
		if typeof(p.get("there")) == TYPE_DICTIONARY:
			doorway[other] = p["there"]


signal said(text: String)

var client: MineWorldClient
var player: SlicePlayer
var world_root: Node3D

var place_ids := {}                ## key -> the place's identity, as the world named it
## key -> the doorway between café and street in that place's frame, as the
## world disclosed it (world millimetres, exactly as received)
var doorway := {}
var cafe_place := ""               ## the café's identity, once known
var here_key := ""                 ## the pack key of the place the world last put the body in
var figures := {}                  ## EntityId string -> the figure drawn for it
var last_sent_local := {}          ## the last position reported, as sent
var answers: Array[Dictionary] = []
## Every entry of the observer's own disclosed conversation history, in order.
var heard: Array = []
## Every place change the server's observations showed, in order: [from, to].
var place_changes: Array = []

var _last_pos := Vector3.INF
var _last_yaw := 0.0
var _since := 0.0
var _tokens := {}                  ## token -> action type, for the transcript
var _reconcile := true             ## put the body where the next observation says
var _unknown_said := false


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


## The pack key of a place identity, if this client knows it.
func key_of(place: String) -> String:
	for k in place_ids:
		if place_ids[k] == place:
			return k
	return ""


## Reconcile the scene with the newest view. An observation is exhaustive, so a
## person no longer listed is no longer perceived and their figure goes.
func _on_observed(obs: MineWorldObservation) -> void:
	var place := obs.place()
	if place == "":
		return
	# learn the place's key from its tag, which is world data
	if key_of(place) == "":
		for tag in obs.entity(place).get("tags", []):
			if KEYS.has(String(tag)):
				place_ids[String(tag)] = place
	var key := key_of(place)
	if key != "":
		_learn_passages(obs, place, key)
	if place_ids.get("cafe", "") != "":
		cafe_place = place_ids["cafe"]
	if key != here_key:
		if here_key != "":
			place_changes.append([here_key, key])
		_say("in place %s (%s)" % [place, key if key != "" else "not drawn by this slice"])
		here_key = key
	_hear(obs)
	# Nothing can be drawn in a place whose doorway the world has not disclosed:
	# the doorway is what binds its frame to this scene.
	if key == "" or not doorway.has(key):
		return

	# The server is the authority on where the body is. On the first view, and
	# after any refused move, the body goes where the world says it is.
	if _reconcile:
		_reconcile = false
		var me: Variant = obs.self_location().get("local")
		if typeof(me) == TYPE_DICTIONARY and player != null:
			player.global_position = to_scene(key, me) + Vector3(0, 0.02, 0)
			player.velocity = Vector3.ZERO
			_last_pos = player.global_position
			_say("body placed where the world says: %s %s" % [key, JSON.stringify(me)])

	var seen := {}
	for id in obs.ids():
		if id == obs.observer():
			continue
		var loc := obs.location_of(id)
		if loc.is_empty() or typeof(loc.get("place")) != TYPE_DICTIONARY:
			continue
		if String(loc["place"].get("entity", "")) != place:
			continue
		seen[id] = true
		var fig: Node3D = figures.get(id)
		if fig == null:
			fig = _figure(id, obs)
			figures[id] = fig
		fig.global_position = to_scene(key, loc.get("local"))
		fig.rotation.y = MineWorldSpace.yaw_to_3d_radians(loc.get("facing")) + PI
	for id in figures.keys():
		if not seen.has(id):
			_say("lose %s at %s (no longer perceived here)"
				% [id, (figures[id] as Node3D).global_position])
			(figures[id] as Node).queue_free()
			figures.erase(id)


## What the observer has been told. Protocol revision 1 sends no event bodies;
## what someone said to you arrives as your own disclosed `conversation-history`
## (`ADOPTION.md` sec.6). Each new entry is shown once, as the world states it --
## the speaker by identity (and tags, which are world data), the words verbatim.
func _hear(obs: MineWorldObservation) -> void:
	var h: Variant = obs.own_component("conversation-history").get("heard", [])
	if typeof(h) != TYPE_ARRAY:
		return
	for i in range(heard.size(), (h as Array).size()):
		var e: Variant = h[i]
		if typeof(e) != TYPE_DICTIONARY:
			continue
		heard.append(e)
		var who := ""
		if typeof(e.get("speaker")) == TYPE_DICTIONARY:
			who = String(e["speaker"].get("entity", ""))
		var tags: Array = obs.entity(who).get("tags", []) if who != "" else []
		_say("%s%s said: %s" % [who, " (%s)" % ", ".join(tags) if not tags.is_empty() else "",
			JSON.stringify(e.get("utterance"))])


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


## World millimetres in a place's frame -> a point in this scene.
func to_scene(key: String, local: Variant) -> Vector3:
	var p := MineWorldSpace.to_3d(local)
	return origin(key) + Vector3(p.x, 0.0, p.z)


## A point in this scene -> world millimetres in a place's frame.
func to_world(key: String, p: Vector3) -> Dictionary:
	var rel := p - origin(key)
	if not REPORT_HEIGHT:
		rel.y = 0.0
	return MineWorldSpace.from_3d(rel)


func _physics_process(delta: float) -> void:
	if client == null or not client.is_seated() or here_key == "" or player == null \
			or _reconcile or not doorway.has(here_key):
		return
	_since += delta
	# Which of the world's places the body is in, by this slice's volumes. In the
	# door's reveal it is in neither, and there is nothing to report.
	var key: String = PLACE_KEY.get(SliceWorld.place_at(world_root, player.global_position), "")
	if key == "":
		return
	if key != here_key:
		# Across the threshold: report the body on the far side, now. Whether the
		# door may be crossed is the movement system's answer, not this client's.
		if not place_ids.has(key) or not doorway.has(key):
			if not _unknown_said:
				_unknown_said = true
				_say("the world has disclosed no passage from here to %s; nothing to report" % key)
			return
		if _since >= REPORT_GAP:
			report_position(key)
		return
	var p := player.global_position
	var flat := Vector2(p.x - _last_pos.x, p.z - _last_pos.z).length() if _last_pos != Vector3.INF \
		else INF
	var turned := absf(angle_difference(player.rotation.y, _last_yaw))
	if _since < REPORT_GAP or (flat < REPORT_DIST and turned < REPORT_TURN):
		return
	report_position(key)


## Report where the body is now, in the place the slice draws it in. Prediction
## is local; the server is the authority and its next observation is the answer
## (`ADOPTION.md` sec.4).
func report_position(key := "") -> String:
	if key == "":
		key = here_key
	var p := player.global_position
	_last_pos = p
	_last_yaw = player.rotation.y
	_since = 0.0
	var local := to_world(key, p)
	last_sent_local = local
	var tok := client.submit(MOVE_ACTION, null, { "to": _location(key, local) },
		_location(key, local))
	_tokens[tok] = MOVE_ACTION
	return tok


func _location(key: String, local: Dictionary) -> Dictionary:
	return MineWorldSpace.location(place_ids.get(key, ""), local,
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
		_location(here_key, to_world(here_key, player.global_position)))
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


func _on_resolved(token: String, action_id: String, result: Variant) -> void:
	var kind := "?"
	if typeof(result) == TYPE_STRING:
		kind = String(result)
	elif typeof(result) == TYPE_DICTIONARY and not (result as Dictionary).is_empty():
		kind = String((result as Dictionary).keys()[0])
	var what: String = _tokens.get(token, "?")
	answers.append({ "token": token, "action": what, "action_id": action_id, "result": kind,
		"detail": result, "sent": last_sent_local.duplicate() })
	if what == MOVE_ACTION and kind != "accepted":
		# Refused for moving: reconcile to where the world says the body is
		# (`PROTOCOL.md` sec.6.2) -- do not argue, do not retry.
		_reconcile = true
	if kind == "accepted" and what == MOVE_ACTION:
		return  # the steady stream of position reports; not worth a toast
	_say("%s %s -> %s %s" % [what, token, kind,
		"" if kind == "accepted" else JSON.stringify(result)])


func _on_refused(code: String, token: String, _detail: String) -> void:
	answers.append({ "token": token, "action": _tokens.get(token, "?"), "result": "refused",
		"code": code })
	if _tokens.get(token, "") == MOVE_ACTION:
		_reconcile = true
	_say("refused %s: %s" % [token, code])


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo \
			and (event as InputEventKey).physical_keycode == KEY_E and client != null:
		talk_to_facing("Hello! A coffee, please.")
