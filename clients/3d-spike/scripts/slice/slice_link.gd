## The slice as a MineWorld presentation: the one place it talks to a world.
##
## Design: `.structured-coding/plans/vis-3d-godot-2/pr-01a-slice.md` sec.7a, and
## sec.7b for the move to S6's `move`. Transport, frames, identities and axes are
## all `clients/protocol/mineworld` (adopted by symlink, `ADOPTION.md` sec.1).
## This file adds only what is the slice's own: where each of the world's places
## sits in this scene, which perceived people to draw, and when to report the
## player's body.
##
## The one action it names is `move`, the body's strides. Every interaction
## request is built by `intents.gd` (`SliceIntents`), and what the player is
## looking at is `targeting.gd`'s ray (`SliceTargeting`): this file connects
## them to the world (step-15 §19).
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


## A status line for the player, already in display labels: never an entity id
## (the ids go to the log only). `notice` marks the ones worth a toast.
signal said(text: String, notice: bool)
## One line of conversation, as a player reads it: `Barista: words`.
signal spoke(line: String)

## Tags that say what kind of thing an entity is in general rather than who it
## is: skipped when a role is read from the tags (`display_label`).
const GENERIC_TAGS := ["public", "staff", "outdoor", "residential"]

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
## EntityId string -> the label last shown for it, so someone heard after they
## have left view is still named as they were seen
var _labels := {}
var intents := SliceIntents.new()  ## every interaction request, and what each token asked


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


## The server's invite (`--invite=`), required on every server (`server/PROTOCOL.md` §4.1). Passed to
## the protocol module and never shown or logged.
static func invite_from_args() -> String:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--invite="):
			return a.substr(9)
	return ""


## The player's nickname (`--nickname=`), shown to this connection only.
static func nickname_from_args() -> String:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--nickname="):
			return a.substr(11)
	return "player"


func start(address: String, seat: String, invite: String, nickname: String) -> void:
	client = MineWorldClient.new()
	client.name = "MineWorldClient"
	add_child(client)
	client.welcomed.connect(_on_welcomed)
	client.observed.connect(_on_observed)
	client.resolved.connect(_on_resolved)
	client.refused.connect(_on_refused)
	client.disconnected.connect(func(reason: String) -> void:
		_say("disconnected: %s" % reason, "", true))
	client.closing.connect(func(reason: String, _detail: String) -> void:
		_say("the server closed the connection: %s" % reason, "", true))
	client.connect_to_world(address, seat, invite, nickname)
	_say("connecting to %s as %s" % [address, seat])


## `shown` is what the player reads, in display labels; `log_detail` adds the
## identities and raw values for the log, and is never put on screen.
func _say(shown: String, log_detail := "", notice := false) -> void:
	print("[link] " + shown + ("  [%s]" % log_detail if log_detail != "" else ""))
	said.emit(shown, notice)


func _on_welcomed(seat: String, observer: String, world: Dictionary) -> void:
	_say("seated as %s" % seat,
		"observer %s, world %s" % [observer, world.get("instance", "?")], true)


## THE ONE PLACE an entity is named for the player: every figure label, every
## HUD line and every conversation line comes from here, and none of them ever
## shows a bare entity id.
##
## The name the world disclosed comes first (the `naming` pack, S8 PR 10c). A
## world without it, or an entity nobody named, falls back to its role: the
## first tag that says who it is rather than what kind of thing it is,
## capitalised -- `barista` -> "Barista". The observer is "You"; an entity with
## nothing to go on is "Someone".
func display_label(id: String) -> String:
	if client != null and id == client.observer:
		return "You"
	var obs := client.latest if client != null else null
	if obs != null and not obs.entity(id).is_empty():
		var shown := _disclosed_name(obs, id)
		if shown == "":
			shown = _role(obs.entity(id).get("tags", []))
		if shown != "":
			_labels[id] = shown
	return _labels.get(id, "Someone")


## The name the world disclosed for a perceived entity, or "" when it disclosed
## none: the `naming` pack's `display-name`, read by the protocol module's own
## `display_name` (`ARC-31`) so the payload's shape lives in one place.
func _disclosed_name(obs: MineWorldObservation, id: String) -> String:
	return obs.display_name(id).strip_edges()


static func _role(tags: Variant) -> String:
	if typeof(tags) != TYPE_ARRAY:
		return ""
	for t in tags:
		var tag := String(t).strip_edges()
		if tag != "" and not GENERIC_TAGS.has(tag):
			return tag.capitalize()
	return ""


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
		_say("in the %s%s" % [display_label(place),
			"" if key != "" else " (not drawn by this slice)"], "place %s" % place)
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
			_say("placed where the world says you are", "%s %s" % [key, JSON.stringify(me)])

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
			fig = _figure(id)
			figures[id] = fig
		# a name disclosed later replaces the role on the label
		var label := fig.get_node_or_null("Label") as Label3D
		if label != null:
			label.text = display_label(id)
		fig.global_position = to_scene(key, loc.get("local"))
		fig.rotation.y = MineWorldSpace.yaw_to_3d_radians(loc.get("facing")) + PI
	for id in figures.keys():
		if not seen.has(id):
			_say("%s is no longer in view" % display_label(id),
				"lose %s at %s" % [id, (figures[id] as Node3D).global_position])
			(figures[id] as Node).queue_free()
			figures.erase(id)


## What the observer has been told. Protocol revision 1 sends no event bodies;
## what someone said to you arrives as your own disclosed `conversation-history`
## (`ADOPTION.md` sec.6). Each new entry is shown once, as a line of
## conversation: the speaker by `display_label`, the words as plain text,
## exactly as the world states them. The speaker's identity goes to the log.
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
		_speak(display_label(who) if who != "" else "Someone", e.get("utterance"),
			"heard from %s" % who)


## Put one line of conversation on screen and in the log. The words are shown
## as text, never as an encoded value: an utterance is a string, and quoting or
## escaping it is a transport's business, not the player's.
func _speak(speaker: String, utterance: Variant, log_detail: String) -> void:
	var words := (String(utterance) if typeof(utterance) == TYPE_STRING
		else str(utterance)).strip_edges()
	var line := "%s: %s" % [speaker, words]
	print("[link] %s  [%s]" % [line, log_detail])
	spoke.emit(line)


## A perceived person, drawn from the same stand-in mannequin as the street's
## figures. Appearance is chosen here from tags, which are world data.
func _figure(id: String) -> Node3D:
	var rng := RandomNumberGenerator.new()
	rng.seed = hash(id)
	var n := NPC.make(rng, NPC.Pose.STAND, 1.72, false)
	n.name = "Person_" + id
	n.set_meta("entity_id", id)       # a string, never a number (ADOPTION.md sec.3.1)
	# what the targeting ray meets; nothing collides with it (step-15 §19)
	n.add_child(SliceTargeting.person_collider())
	world_root.add_child(n)
	var label := Label3D.new()
	label.name = "Label"
	label.text = display_label(id)
	label.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	label.position = Vector3(0, 2.05, 0)
	label.font_size = 48
	label.pixel_size = 0.004
	n.add_child(label)
	_say("%s is here" % label.text, "perceive %s" % id)
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


## Talk to whoever the camera is looking at. Targeting is presentation; whether
## a conversation may happen is the server's, and the request is sent regardless
## of what the affordance says (ADOPTION.md sec.3.3).
func talk_to_facing(utterance: String) -> String:
	var target := facing_person()
	if target == "":
		_say("nobody in view to talk to", "", true)
		return ""
	var obs := client.latest
	if not SliceIntents.talk_offered(obs, target):
		_say("the world says talking to %s is unavailable: %s"
			% [display_label(target), _readable(SliceIntents.talk_reason(obs, target))],
			"target %s" % target)
	return intents.talk(client, target, utterance,
		_location(here_key, to_world(here_key, player.global_position)))


## A server's reason code, as words: `too_far_away` -> "too far away".
static func _readable(code: Variant) -> String:
	if typeof(code) == TYPE_DICTIONARY and not (code as Dictionary).is_empty():
		code = (code as Dictionary).keys()[0]
	return String(code).replace("_", " ") if typeof(code) == TYPE_STRING else str(code)


## The perceived person the camera's ray meets first, or "" -- a wall, a
## counter or nothing in the way of the eye (`SliceTargeting`).
func facing_person() -> String:
	var id := String(SliceTargeting.aim(player.get_viewport())["entity"])
	return id if figures.has(id) else ""


func _on_resolved(token: String, action_id: String, result: Variant) -> void:
	var kind := "?"
	if typeof(result) == TYPE_STRING:
		kind = String(result)
	elif typeof(result) == TYPE_DICTIONARY and not (result as Dictionary).is_empty():
		kind = String((result as Dictionary).keys()[0])
	var req := intents.take(token)
	var what: String = req.get("action", _tokens.get(token, "?"))
	answers.append({ "token": token, "action": what, "action_id": action_id, "result": kind,
		"detail": result, "sent": last_sent_local.duplicate() })
	if what == MOVE_ACTION and kind != "accepted":
		# Refused for moving: reconcile to where the world says the body is
		# (`PROTOCOL.md` sec.6.2) -- do not argue, do not retry.
		_reconcile = true
	if kind == "accepted" and what == MOVE_ACTION:
		return  # the steady stream of position reports; not worth a toast
	var log_detail := "%s %s -> %s %s" % [what, token, kind, JSON.stringify(result)]
	if req.has("words"):
		if kind == "accepted":
			# what the player said, echoed as a line of the conversation
			_say("talking to %s" % display_label(req["target"]))
			_speak(display_label(client.observer), req["words"], log_detail)
			return
		var reason: Variant = (result as Dictionary).get(kind) \
			if typeof(result) == TYPE_DICTIONARY else result
		_say("can't %s %s: %s" % [SliceIntents.verb(what), display_label(req["target"]),
			_readable(reason)],
			log_detail, true)
		return
	_say("%s: %s" % [what, kind if kind == "accepted" else "%s, %s" % [kind, _readable(
		(result as Dictionary).get(kind) if typeof(result) == TYPE_DICTIONARY else result)]],
		log_detail, kind != "accepted")


func _on_refused(code: String, token: String, _detail: String) -> void:
	var req := intents.take(token)
	answers.append({ "token": token, "action": req.get("action", _tokens.get(token, "?")),
		"result": "refused", "code": code })
	if _tokens.get(token, "") == MOVE_ACTION:
		_reconcile = true
	var about := ""
	if req.has("target"):
		about = " (%s %s)" % [SliceIntents.verb(req["action"]), display_label(req["target"])]
	_say("refused%s: %s" % [about, _readable(code)], "token %s" % token, true)


## The person the player is looking at, by the name the HUD shows, or "".
func looking_at() -> String:
	var id := facing_person()
	return display_label(id) if id != "" else ""


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo \
			and (event as InputEventKey).physical_keycode == KEY_E and client != null:
		talk_to_facing(SliceIntents.DEFAULT_UTTERANCE)
