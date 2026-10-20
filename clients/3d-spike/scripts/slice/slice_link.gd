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

## THE SPATIAL BINDING -- where each of the world's places sits in this scene --
## is `SliceLayout`'s (step-15 §20, D-16c-2): it learns every place, its
## identity and its doorways from what the server discloses, and binds each to
## the drawn door its doorway lands on. This file keeps no list of the world's
## places; it asks the layout where to draw and which place the body is in.
## Since S7 the movement system DISCLOSES the doorways: the observer's own
## place carries a `passages` component listing, for each way out, the place it
## leads to and the doorway's position on both sides.


## A status line for the player, as a catalog key and its arguments, already in
## display labels: never an entity id (the ids go to the log only). The HUD words
## it in the language selected (S20). `notice` marks the ones worth a toast.
signal said(key: String, args: Dictionary, notice: bool)
## One line of conversation, as a player reads it: `Barista: words`.
signal spoke(line: String)

## The kinds of answer a result can be, by their catalog keys (S20).
const RESULT_KINDS := {
	"rejected": "link.kind.rejected",
	"unavailable": "link.kind.unavailable",
}

## Tags that say what kind of thing an entity is in general rather than who it
## is: skipped when a role is read from the tags (`display_label`).
const GENERIC_TAGS := ["public", "staff", "outdoor", "residential"]

var client: MineWorldClient
var player: SlicePlayer
var world_root: Node3D

var layout := SliceLayout.new()    ## where each of the world's places is drawn
var here := ""                     ## the place the world last put the body in, as the world named it
## EntityId string -> the figure drawn for every person perceived now; the
## dictionary `SliceFigures` keeps, which builds and moves them (D-16c-8)
var figures := {}
var _figures: SliceFigures
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
var _reconcile := true             ## put the body where the next observation says (the first view)
## The current place's disclosed walls and solids, as invisible colliders
## (step-11 §22 SD-E4).
var bodies := SliceClientBodies.new()
## The drawn place's loose objects (SD-E7).
var objects: SliceObjects
## When the drawn body must take the server's position (SD-E8, ARC-50). No
## other code moves the body to a server position (I-S14-4).
var correction := SliceCorrection.new()
## How many reports carried the wished stride (press-through), for the log and the probe.
var press_reports := 0
var _unknown_said := false
## EntityId string -> the label last shown for it, so someone heard after they
## have left view is still named as they were seen
var _labels := {}
var intents := SliceIntents.new()  ## every interaction request, and what each token asked
var things: SliceThings            ## what the things the world mentions are called (PR 16d)
## The pack's cast binding: which body draws a person, from the look the world disclosed (ARC-92).
var cast: SliceCast = null


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
	cast = SliceCast.load_pack(
		ProjectSettings.globalize_path("res://").path_join(SliceMain.PACK_DIR).simplify_path())
	_figures = SliceFigures.new()
	_figures.name = "Figures"
	_figures.world_root = world_root
	_figures.layout = layout
	_figures.cast = cast
	add_child(_figures)
	figures = _figures.nodes
	objects = SliceObjects.new()
	objects.name = "Objects"
	objects.world_root = world_root
	objects.layout = layout
	add_child(objects)
	client = MineWorldClient.new()
	client.name = "MineWorldClient"
	add_child(client)
	client.submitted_request.connect(_on_submitted)
	client.welcomed.connect(_on_welcomed)
	client.observed.connect(_on_observed)
	client.resolved.connect(_on_resolved)
	client.refused.connect(_on_refused)
	client.disconnected.connect(func(reason: String) -> void:
		_say("disconnected: %s" % reason, "link.disconnected", {"reason": _shown_reason(reason)}, true))
	things = SliceThings.new(self)
	client.reconnecting.connect(func(_attempt: int) -> void: _forget_pending())
	client.closing.connect(func(reason: String, _detail: String) -> void:
		_say("the server closed the connection: %s" % reason, "link.server_closed",
			{"reason": _shown_reason(reason)}, true))
	client.connect_to_world(address, seat, invite, nickname)
	_say("connecting to %s as %s" % [address, seat], "link.connecting", {"address": address, "seat": seat})


## `log_line` is the English line the log prints, exactly as before S20
## (INV-SET-6); `key` and `args` are what the player reads, worded by the HUD in
## the language selected. `log_detail` adds the identities and raw values for the
## log, and is never put on screen.
func _say(log_line: String, key: String, args: Dictionary = {}, notice := false, log_detail := "") -> void:
	print("[link] " + log_line + ("  [%s]" % log_detail if log_detail != "" else ""))
	said.emit(key, args, notice)


func _on_welcomed(seat: String, observer: String, world: Dictionary) -> void:
	_say("seated as %s" % seat, "link.seated", {"seat": seat}, true,
		"observer %s, world %s" % [observer, world.get("instance", "?")])


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


## `display_label` as the player reads it, for a message's argument: a disclosed
## name or role as the world states it (never translated), or "You" / "Someone"
## as a catalog reference, worded in the language selected whenever the message
## is rendered. The log keeps `display_label`'s English.
func shown_label(id: String) -> Variant:
	if client != null and id == client.observer:
		return MineWorldText.ref("link.you")
	var label := display_label(id)
	return MineWorldText.ref("link.someone") if not _labels.has(id) else label


## `shown_label` worded now, for a node's text (a figure's label).
func shown_text(id: String) -> String:
	var said: Variant = shown_label(id)
	return MineWorldText.text(said.key) if said is MineWorldText.Ref else String(said)


## A server's code as the player reads it: `reason.<code>` (S20 §3.6), as a
## catalog reference, or the readable form when the code is not one a catalog
## could hold.
static func _shown_reason(code: Variant) -> Variant:
	if typeof(code) == TYPE_DICTIONARY and not (code as Dictionary).is_empty():
		code = (code as Dictionary).keys()[0]
	if typeof(code) != TYPE_STRING or String(code) == "":
		return _readable(code)
	for c in String(code):
		if not (c in "abcdefghijklmnopqrstuvwxyz0123456789_-"):
			return _readable(code)
	return MineWorldText.code_ref("reason", String(code))


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


## Reconcile the scene with the newest view. An observation is exhaustive, so a
## person no longer listed is no longer perceived and their figure goes.
func _on_observed(obs: MineWorldObservation) -> void:
	var place := obs.place()
	if place == "":
		return
	var misses_before := layout.misses.size()
	layout.learn(obs)
	things.learn(obs)
	if layout.misses.size() != misses_before:
		for p in layout.misses:
			var m: Dictionary = layout.misses[p]
			print("[link] the doorway to place %s binds nothing: %s; nearest drawn door %s at %.2f m"
				% [p, m["why"], m["nearest"], m["distance"]])
	if place != here:
		if here != "":
			place_changes.append([here, place])
			# the old place's people are no longer perceived, and not by a door
			_figures.clear()
		# A place the scene has not bound -- a seat that starts outside the
		# café, or a doorway that lands on no drawn door -- is said, not drawn.
		var drawn := layout.is_drawn(place)
		var where := "in the %s%s" % [display_label(place), "" if drawn else " (not drawn by this slice)"]
		_say(where, "link.in_place" if drawn else "link.in_place_undrawn",
			{"place": shown_label(place)}, false, "place %s" % place)
		here = place
	_hear(obs)
	_wear(obs)
	# Nothing can be drawn in a place the layout has not bound to the scene.
	if not layout.is_drawn(place):
		bodies.observe(obs, layout, world_root)
		objects.observe(obs)
		return
	bodies.observe(obs, layout, world_root)
	objects.observe(obs)
	_stopped(obs)
	_prev_auth = _mm(obs.self_location().get("local"))

	# The server is the authority on where the body is. On the first view the
	# body goes where the world says it is; after that, the correction rule
	# decides when the drawn body takes the server's position (ARC-50).
	var me: Variant = obs.self_location().get("local")
	if _reconcile:
		_reconcile = false
		if typeof(me) == TYPE_DICTIONARY and player != null:
			player.global_position = to_scene(place, me) + Vector3(0, 0.02, 0)
			player.velocity = Vector3.ZERO
			_last_pos = player.global_position
			correction.adopted(_mm(me), place)
			_say("placed where the world says you are", "link.placed", {}, false,
				"%s %s" % [place, JSON.stringify(me)])
	elif typeof(me) == TYPE_DICTIONARY and player != null:
		var drawn := _mm(to_world(place, player.global_position))
		var to := correction.observe(place, _mm(me), drawn, obs.acted_through())
		if to != Vector2.INF:
			var c: Dictionary = correction.history[correction.history.size() - 1]
			var snapped := player.correct_to(to_scene(place, me))
			c["snap"] = snapped
			# what stopped the body, if the world said so within the last second (for C-1's
			# classes, step-11 §22.4 as amended 2026-10-17)
			c["stop"] = {}
			for k in range(stops.size() - 1, -1, -1):
				if String(stops[k]["place"]) == place and float(c["at"]) - float(stops[k]["at"]) <= 1.0:
					c["stop"] = stops[k]
					break
			print("[link] corrected %.3f m (%s, %s form) to %s in %s; the request it reflects (%s) reported %s, the body was drawn at %s"
				% [c["size"], "snap" if snapped else "glide", c["form"], JSON.stringify(me), place,
				_tokens_kind(String(c["acted"])), c["reference"], c["from"]])

	var seen := {}
	for id in obs.ids():
		if id == obs.observer():
			continue
		var loc := obs.location_of(id)
		if loc.is_empty() or typeof(loc.get("place")) != TYPE_DICTIONARY:
			continue
		if String(loc["place"].get("entity", "")) != place:
			continue
		# a name disclosed later replaces the role on the label
		seen[id] = {"at": to_scene(place, loc.get("local")),
			"yaw": MineWorldSpace.yaw_to_3d_radians(loc.get("facing")) + PI, "label": shown_text(id)}
		seen[id]["look"] = obs.look(id)
		seen[id]["name"] = obs.display_name(id)
	var gone := {}
	for id in figures:
		if not seen.has(id):
			gone[id] = (figures[id] as Node3D).global_position
	var change := _figures.observe(seen)
	for id in change["arrived"]:
		_say("%s is here" % display_label(id), "link.arrived", {"who": shown_label(id)}, false,
			"perceive %s" % id)
	for id in change["left"]:
		_say("%s is no longer in view" % display_label(id), "link.left_view",
			{"who": shown_label(id)}, false, "lose %s at %s" % [id, gone[id]])


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
		_speak(who, e.get("utterance"), "heard from %s" % who)


## Put one line of conversation on screen and in the log. The words are shown
## as text, never as an encoded value: an utterance is a string, and quoting or
## escaping it is a transport's business, not the player's. The speaker is named
## in English in the log and in the language selected on screen (S20); the words
## are the world's and are never translated.
func _speak(who: String, utterance: Variant, log_detail: String) -> void:
	var words := (String(utterance) if typeof(utterance) == TYPE_STRING
		else str(utterance)).strip_edges()
	var logged := display_label(who) if who != "" else "Someone"
	var shown := shown_text(who) if who != "" else MineWorldText.text("link.someone")
	print("[link] %s  [%s]" % ["%s: %s" % [logged, words], log_detail])
	spoke.emit("%s: %s" % [shown, words])


## The player's own body follows the Person the world seated it as: the observer's
## disclosed look through the cast binding, exactly as every other figure
## (`SliceCast`, `INV-1`). Never a fixed default avatar.
func _wear(obs: MineWorldObservation) -> void:
	if cast == null or player == null or not player.has_method("wear"):
		return
	var me := obs.observer()
	var look := obs.look(me)
	var character := cast.character_for(look, me)
	var spec := cast.body_for(character)
	var key := SliceCast.body_key(character, spec)
	if player.slot == null or player.slot.body_key != key or not has_meta("worn"):
		set_meta("worn", true)
		player.wear(spec, key)
		print("[link] wearing %s  [observer %s, name %s, look %s]" % [
			character if character != "" else "(town figure)", me, obs.display_name(me), look])


## World millimetres in a bound place's frame -> a point in this scene.
func to_scene(place: String, local: Variant) -> Vector3:
	return layout.to_scene(place, local)


## A point in this scene -> world millimetres in a bound place's frame. Height is
## never reported (REPORT_HEIGHT).
func to_world(place: String, p: Vector3) -> Dictionary:
	return layout.to_world(place, p)


func _physics_process(delta: float) -> void:
	if client == null or not client.is_seated() or here == "" or player == null \
			or _reconcile or not layout.is_drawn(here):
		return
	_since += delta
	# A correction is being drawn, or its suspension holds: nothing is reported
	# until every request sent before it is answered (ARC-50).
	if not correction.may_report() or player.gliding():
		return
	# Which of the world's places the body is in, by this slice's volumes and the
	# places the layout bound them to. In the door's reveal it is in no volume,
	# and there is nothing to report.
	var volume := SliceWorld.place_at(world_root, player.global_position)
	var at := layout.place_of_volume(volume)
	if at == "":
		if volume != "" and not _unknown_said:
			# a room no disclosed doorway has bound (a world without one)
			_unknown_said = true
			_say("the world has disclosed no passage from here to %s; nothing to report" % volume,
				"link.no_passage", {"place": volume})
		return
	if at != here:
		# Across the threshold: report the body on the far side, now. Whether the
		# door may be crossed is the movement system's answer, not this client's.
		if _since >= REPORT_GAP:
			report_position(at)
		return
	var p := player.global_position
	var flat := Vector2(p.x - _last_pos.x, p.z - _last_pos.z).length() if _last_pos != Vector3.INF \
		else INF
	var turned := absf(angle_difference(player.rotation.y, _last_yaw))
	# Pressing into a person or an object, the body stands still; the travel the
	# input wished for counts instead (press-through, step-11 §22 SD-E5).
	flat = maxf(flat, player.pressed_travel)
	if _since < REPORT_GAP or (flat < REPORT_DIST and turned < REPORT_TURN):
		return
	report_position(at)


## Report where the body is now, in the place the slice draws it in -- or, while
## the input presses into a person or an object, the point it wished to reach
## (`Player.intended_point`, SD-E5). Prediction is local; the server is the
## authority and its next observation is the answer (`ADOPTION.md` sec.4).
func report_position(place := "") -> String:
	if place == "":
		place = here
	var p := player.global_position
	_last_pos = p
	_last_yaw = player.rotation.y
	_since = 0.0
	if player.press_dir != Vector3.ZERO:
		p = player.intended_point()
		press_reports += 1
		print("[link] pressing into someone or something: reporting the wished stride, %.2f m past the body"
			% Vector2(p.x - player.global_position.x, p.z - player.global_position.z).length())
	player.reported()
	var local := to_world(place, p)
	last_sent_local = local
	var tok := client.submit(MOVE_ACTION, null, { "to": _location(place, local) },
		_location(place, local))
	_tokens[tok] = MOVE_ACTION
	return tok


## Every request this connection sends, as the protocol module sent it: the
## position it reported goes to the correction rule, so an observation's
## `acted_through` can be compared with it (ARC-50's exact form).
func _on_submitted(token: String, request: Dictionary) -> void:
	var at: Variant = request.get("actor_location")
	var point := Vector2.INF
	var place := ""
	if typeof(at) == TYPE_DICTIONARY and typeof(at.get("local")) == TYPE_DICTIONARY:
		point = _mm(at["local"])
		if typeof(at.get("place")) == TYPE_DICTIONARY:
			place = String(at["place"].get("entity", ""))
	correction.sent(token, point, place, String(request.get("action_type", "")) == MOVE_ACTION)


## The facts that stopped this body short (presence's `stopped-short`), for the log: what the
## stride asked for, where it ended and what was in the way -- a person's id, or nothing for a wall
## or a solid. The log's answer to "why was I corrected".
func _stopped(obs: MineWorldObservation) -> void:
	for e in obs.events_of("stopped-short"):
		var p: Variant = e.get("payload", {}).get("payload") if typeof(e.get("payload")) == TYPE_DICTIONARY else null
		if typeof(p) != TYPE_DICTIONARY or SliceObjects.id_of(p.get("person")) != obs.observer():
			continue
		var by: Variant = p.get("by")
		stops.append({"at": Time.get_ticks_msec() / 1000.0, "place": obs.place(),
			"wanted": _mm((p.get("wanted", {}) as Dictionary).get("local")),
			"reached": _mm((p.get("reached", {}) as Dictionary).get("local")),
			"by": "" if by == null else SliceObjects.id_of(by), "start": _prev_auth})
		print("[link] stopped short: wanted %s, reached %s, by %s" % [
			JSON.stringify((p.get("wanted", {}) as Dictionary).get("local")),
			JSON.stringify((p.get("reached", {}) as Dictionary).get("local")),
			"a wall or a solid" if by == null else display_label(SliceObjects.id_of(by))])


## Every stopped-short fact of this body, in order: { at (wall s), place, wanted, reached (mm),
## by ("" for a wall or a solid, else the person's id), start (the server's position for the body
## in the observation before) }.
var stops: Array = []
var _prev_auth := Vector2.INF


## What kind of request an ActionId was, for the log ("move", "talk", …, or "?").
func _tokens_kind(action_id: String) -> String:
	for a in answers:
		if String(a.get("action_id", "")) == action_id and action_id != "":
			return "%s %s" % [a["action"], action_id]
	return "? %s" % action_id


## A world position `{x, y}` as millimetres on the floor.
static func _mm(local: Variant) -> Vector2:
	if typeof(local) != TYPE_DICTIONARY:
		return Vector2.INF
	return Vector2(float(local.get("x", 0)), float(local.get("y", 0)))


func _location(place: String, local: Dictionary) -> Dictionary:
	return MineWorldSpace.location(place, local, MineWorldSpace.yaw_from_3d_radians(player.rotation.y))


## Talk to whoever the camera is looking at. Targeting is presentation; whether
## a conversation may happen is the server's, and the request is sent regardless
## of what the affordance says (ADOPTION.md sec.3.3).
func talk_to_facing(utterance: String) -> String:
	var target := facing_person()
	if target == "":
		_say("nobody in view to talk to", "link.nobody", {}, true)
		return ""
	var obs := client.latest
	if not SliceIntents.talk_offered(obs, target):
		var reason: Variant = SliceIntents.talk_reason(obs, target)
		var logged := "the world says talking to %s is unavailable: %s" % [display_label(target),
			_readable(reason)]
		_say(logged, "link.talk_unavailable", {"who": shown_label(target), "reason": _shown_reason(reason)},
			false, "target %s" % target)
	return intents.talk(client, target, utterance,
		_location(here, to_world(here, player.global_position)))


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
	# A move answered other than accepted is not argued or retried: the next
	# observation shows where the world holds the body, and the correction rule
	# draws it there (`PROTOCOL.md` sec.6.2, ARC-50).
	correction.answered(token, action_id)
	if kind == "accepted" and what == MOVE_ACTION:
		return  # the steady stream of position reports; not worth a toast
	var log_detail := "%s %s -> %s %s" % [what, token, kind, JSON.stringify(result)]
	if req.has("words"):
		if kind == "accepted":
			# what the player said, echoed as a line of the conversation
			_say("talking to %s" % display_label(req["target"]), "link.talking_to",
				{"who": shown_label(req["target"])})
			_speak(client.observer, req["words"], log_detail)
			return
		var reason: Variant = (result as Dictionary).get(kind) \
			if typeof(result) == TYPE_DICTIONARY else result
		var logged := "can't %s %s: %s" % [SliceIntents.log_verb(what), display_label(req["target"]),
			_readable(reason)]
		_say(logged, "link.cant", {"verb": SliceIntents.verb(what), "who": shown_label(req["target"]),
			"reason": _shown_reason(reason)}, true, log_detail)
		return
	if req.has("about"):
		_offer_answered(req, kind, result, log_detail)
		return
	var detail: Variant = (result as Dictionary).get(kind) if typeof(result) == TYPE_DICTIONARY else result
	var accepted := kind == "accepted"
	var outcome := kind if accepted else "%s, %s" % [kind, _readable(detail)]
	_say("%s: %s" % [what, outcome], "link.result_accepted" if accepted else "link.result_other",
		{"action": what, "kind": MineWorldText.ref(RESULT_KINDS[kind]) if RESULT_KINDS.has(kind) else kind,
			"reason": _shown_reason(detail)}, not accepted, log_detail)


func _on_refused(code: String, token: String, _detail: String) -> void:
	var req := intents.take(token)
	answers.append({ "token": token, "action": req.get("action", _tokens.get(token, "?")),
		"result": "refused", "code": code })
	correction.answered(token, "")
	if req.has("about"):
		_say("refused (%s): %s" % [req["action"], code], "ui.result.refused",
			{"action": _asked(req, false), "code": code}, true, "token %s" % token)
		return
	var about := ""
	if req.has("target"):
		about = " (%s %s)" % [SliceIntents.log_verb(req["action"]), display_label(req["target"])]
	var shown := {"reason": _shown_reason(code)}
	if req.has("target"):
		shown["verb"] = SliceIntents.verb(req["action"])
		shown["who"] = shown_label(req["target"])
	_say("refused%s: %s" % [about, _readable(code)],
		"link.refused_about" if req.has("target") else "link.refused", shown, true, "token %s" % token)


## PR 16d (D-16d-7): the answer to an offer a menu sent, in words, with names -- 13b's D-b-6: done
## ("You bought Coffee"), or the world said no and why, or nothing here can do that. The log line keeps
## the action type and the identities (INV-SET-6); the screen never shows either.
func _offer_answered(req: Dictionary, kind: String, result: Variant, log_detail: String) -> void:
	var detail: Variant = (result as Dictionary).get(kind) if typeof(result) == TYPE_DICTIONARY else result
	var logged := "%s about %s: %s" % [req["action"], req["about"], kind]
	match kind:
		"accepted":
			_say(logged, "ui.result.accepted", {"done": _asked(req, true)}, true, log_detail)
		"rejected":
			_say(logged, "ui.result.rejected", {"action": _asked(req, false),
				"reason": SliceThings.reason_words(detail)}, true, log_detail)
		_:
			_say(logged, "ui.result.unavailable", {"action": _asked(req, false)}, true, log_detail)


## What an offer that was sent asked for, in words: its entry's wording, or its "done" wording.
func _asked(req: Dictionary, done: bool) -> Variant:
	return things.action_ref(String(req.get("action", "")), String(req.get("target", "")),
		String(req.get("about", "")), done)


## The connection dropped: every offer sent and not yet answered is said to be unknown, never resent
## (D-16d-7). A talk's words are dropped with it.
func _forget_pending() -> void:
	for req in intents.drop_pending():
		if req.has("about"):
			_say("%s about %s: unknown, the connection dropped" % [req["action"], req["about"]],
				"ui.result.unknown", {"action": _asked(req, false)}, true)


## The position the body reports with an interaction: where it stands now, in the place it is drawn
## in (`PROTOCOL.md` §6.1).
func actor_location() -> Dictionary:
	return _location(here, to_world(here, player.global_position))


## Tab with nobody under the reticle: said, not sent.
func nobody_to_offer() -> void:
	_say("nobody in view to offer anything to", "ui.menu.nobody", {}, true)


## What this client shows of the world, for the parity test (S12 13a's `SHOWN`
## report; step-15 D-16c-7): the layout's bound places and doorways, and every
## drawn figure with the observer, each at the position it is DRAWN at -- so the
## report is an instrument of the scene, not a copy of the observation.
func shown() -> Dictionary:
	var people := []
	if client != null and player != null:
		people.append({"id": client.observer, "label": display_label(client.observer),
			"at": player.global_position})
	for id in figures:
		people.append({"id": id, "label": display_label(id),
			"at": (figures[id] as Node3D).global_position})
	return layout.shown(here, people)


## The person the player is looking at, by the name the HUD shows, or "".
func looking_at() -> String:
	var id := facing_person()
	return shown_text(id) if id != "" else ""


## What the camera's ray meets first: `{ entity, kind, point }`, `kind` being
## "person" (a drawn figure), "object" (a drawn loose object) or "" (the scene,
## or nothing), `point` the hit point or null (`SliceTargeting`).
func aim() -> Dictionary:
	var hit := SliceTargeting.aim(player.get_viewport())
	var id := String(hit["entity"])
	var kind := ""
	if figures.has(id):
		kind = "person"
	elif objects != null and objects.nodes.has(id):
		kind = "object"
	else:
		id = ""
	return { "entity": id, "kind": kind, "point": hit["point"] }


## The object a throw is being aimed with (the first G picked it), or "".
var aiming := ""


## Kick the object the camera is looking at (F). Sent whatever the offers say;
## the server answers (step-11 §22 SD-E6).
func kick_facing() -> String:
	var a := aim()
	if a["kind"] != "object":
		_say("nothing in view to kick", "link.nothing_to", {"verb": SliceIntents.verb(SliceIntents.KICK)}, true)
		return ""
	return intents.kick(client, objects.ref_of(a["entity"]), _here_location())


## Shove the person the camera is looking at (R).
func shove_facing() -> String:
	var a := aim()
	if a["kind"] != "person":
		_say("nobody in view to shove", "link.nothing_to", {"verb": SliceIntents.verb(SliceIntents.SHOVE)}, true)
		return ""
	return intents.shove(client, a["entity"], _here_location())


## G: the first press picks the object looked at and starts an aim; the second
## throws it -- with `toward` null (the world's default line) while the aim
## still rests on the object, else toward the point the ray meets, in this
## place's frame, rounded to millimetres. The point is never clamped: how far a
## throw may go is the server's (step-15 §4.4).
func throw_press() -> String:
	var a := aim()
	if aiming == "":
		if a["kind"] != "object":
			_say("nothing in view to throw", "link.nothing_to", {"verb": SliceIntents.verb(SliceIntents.THROW)},
				true)
			return ""
		aiming = a["entity"]
		return ""
	var object := aiming
	aiming = ""
	var toward: Variant = null
	if a["entity"] != object and a["point"] != null:
		var w := to_world(here, a["point"])
		toward = { "x": int(w.get("x", 0)), "y": int(w.get("y", 0)) }
	return intents.throw(client, objects.ref_of(object), toward, _here_location())


## Esc during an aim drops it. Whether there was one to drop.
func cancel_aim() -> bool:
	var had := aiming != ""
	aiming = ""
	return had


## The server's offers about what the camera is looking at, for the HUD (SD-E10):
## for a person, every affordance directed at them; for an object, every complete
## affordance whose payload names it. Each `{ action, available, reason }`, in
## the server's order. Read, never computed.
func offers(a: Dictionary) -> Array:
	var out: Array = []
	var obs := client.latest if client != null else null
	if obs == null or a["entity"] == "":
		return out
	var listed: Array = obs.affordances("", a["entity"]) if a["kind"] == "person" \
		else obs.affordances_about(a["entity"]).filter(func(o): return MineWorldObservation.is_complete(o))
	for o in listed:
		out.append({ "action": String(o.get("action_type", "")), "available": bool(o.get("available", false)),
			"reason": o.get("unavailable_reason") })
	return out


func _here_location() -> Dictionary:
	return _location(here, to_world(here, player.global_position))


func _unhandled_input(event: InputEvent) -> void:
	if not (event is InputEventKey and event.pressed and not event.echo) or client == null:
		return
	match (event as InputEventKey).physical_keycode:
		KEY_E: talk_to_facing(SliceIntents.DEFAULT_UTTERANCE)
		KEY_F: kick_facing()
		KEY_G: throw_press()
		KEY_R: shove_facing()
