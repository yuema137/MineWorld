extends Node2D

# The demonstration scene for `mineworld/`: the smallest thing that proves the module works against a
# real server.
#
#   godot --path clients/protocol                                  windowed, driven by the keyboard
#   godot --path clients/protocol -- --autopilot                   scripted, and prints a transcript
#   godot --headless --path clients/protocol -- --autopilot        the same, with nothing rendered
#
# It is deliberately not art. It draws the world as dots and text, because what it exists to show is
# that a client can obtain a world, render *only* what it was told, act, and be answered — and
# because the 2D and 3D clients that will adopt this module keep their own scene graphs, their own
# art and their own camera work. What changes for them is where the world comes from, and this is
# that, with nothing else attached.
#
# There is no world rule in this file. It never measures a distance, never decides whether anybody is
# available, never decides whether an action exists. Every verdict it draws arrived in an affordance
# the server computed (`docs/ENGINEERING_RULES.md` §§8-9).

const DEFAULT_ADDRESS := "127.0.0.1:7878"
const DEFAULT_SEAT := "visitor"

# What a client is allowed to decide for itself: wording, colour and layout.
const VERB := { "talk": "Talk to", "move": "Walk to" }
const REASON := {
	"busy": "busy",
	"too_far_away": "too far away",
	"permission_denied": "not allowed",
	"no_supported_interaction": "nothing to do there",
	"target_unavailable": "not available",
	"unavailable": "refused",
	"precondition_failed": "cannot be done",
}
const TAG_COLOUR := {
	"barista": Color(1.00, 0.62, 0.25),
	"staff": Color(1.00, 0.62, 0.25),
	"regular": Color(0.62, 0.62, 0.68),
	"visitor": Color(0.30, 0.75, 1.00),
	"cafe": Color(0.35, 0.35, 0.42),
}
const ROOM_CENTRE := Vector2(300.0, 330.0)
const PIXELS_PER_METRE := 60.0

# How close to stand, in metres on the screen, and what to say there.
#
# Beside somebody rather than at a fixed point, because that is what a client actually does with a
# click: it works in its own space and converts once. Whether the result is close *enough* is the
# server's to answer, and the autopilot waits for that answer rather than predicting it.
const STAND_BESIDE := 1.0
const GREETING := "hello Alice, this is the demonstration scene"

# The longest single `move` this client sends, in metres.
#
# The server takes at most 2 m per request and a client must report before its body has gone that
# far since the last position the server accepted (`server/PROTOCOL.md` §6.2, the reporting rule).
# This is a request size, not a rule the client enforces: the server still decides every stride. It
# is a little under 2 m so that rounding a stride to whole millimetres can never make it 2.001 m.
const STRIDE := 1.9

var _world := MineWorldClient.new()
var _address := DEFAULT_ADDRESS
var _seat := DEFAULT_SEAT
var _autopilot := false
# `2d` reports no position when it acts; `3d` reports the one it walked to. The only difference
# between the two clients at the protocol boundary, and the whole of `AC-13`'s permitted difference.
var _flavour := "2d"
var _requests_path := ""
var _shot_path := ""

var _font: Font = ThemeDB.fallback_font
var _last_answer := "—"
var _requests: Array = []
var _step := 0
var _settled := 0

# A walk in progress, in this client's own 2D metres: where it is going, where the server last
# accepted it to be, the stride it is waiting to hear about, and that stride's token.
var _walk_to: Variant = null
var _walk_from := Vector2.ZERO
var _stride_to := Vector2.ZERO
var _stride_token := ""
var _strides := 0


func _ready() -> void:
	_read_arguments()
	add_child(_world)
	_world.welcomed.connect(_on_welcomed)
	_world.observed.connect(_on_observed)
	_world.resolved.connect(_on_resolved)
	_world.refused.connect(_on_refused)
	_world.disconnected.connect(_on_disconnected)
	_world.submitted_request.connect(_on_submitted)
	_note("connecting to %s as seat '%s' (%s)" % [_address, _seat, _flavour])
	_world.connect_to_world(_address, _seat)


func _read_arguments() -> void:
	var arguments := OS.get_cmdline_user_args()
	var index := 0
	while index < arguments.size():
		var argument := String(arguments[index])
		var value := String(arguments[index + 1]) if index + 1 < arguments.size() else ""
		match argument:
			"--address":
				_address = value
				index += 1
			"--seat":
				_seat = value
				index += 1
			"--flavour":
				_flavour = value
				index += 1
			"--requests":
				_requests_path = value
				index += 1
			"--screenshot":
				_shot_path = value
				index += 1
			"--autopilot":
				_autopilot = true
			_:
				push_warning("[demo] ignoring unknown argument %s" % argument)
		index += 1


# ------------------------------------------------------------------------------------------------
# What the server says.
# ------------------------------------------------------------------------------------------------

func _on_welcomed(seat: String, observer: String, world: Dictionary) -> void:
	_note("seated: seat '%s' is observer %s" % [seat, observer])
	_note("world instance %s, %d entities, %d client(s)" % [
		_world.world_instance(), int(world.get("entities", 0)), int(world.get("clients", 0)),
	])


func _on_observed(observation: MineWorldObservation) -> void:
	if _step == 0:
		_note("observation %d: %s" % [_world.sequence, ", ".join(observation.ids())])
		_step = 1
	if _autopilot:
		_autopilot_step(observation)
	queue_redraw()


func _on_resolved(token: String, action_id: String, result: Dictionary) -> void:
	_last_answer = _describe(result)
	_note("%s -> action %s: %s" % [token, action_id, _last_answer])
	if token == _stride_token:
		_stride_answered(result.has("accepted"))


func _on_refused(code: String, token: String, detail: String) -> void:
	_last_answer = "refused (%s)" % code
	_note("%s refused: %s — %s" % [token, code, detail])
	if token == _stride_token:
		_stride_answered(false)


func _on_disconnected(reason: String) -> void:
	_note("disconnected: %s" % reason)
	if _autopilot:
		_finish()


func _on_submitted(token: String, request: Dictionary) -> void:
	# Recorded as it went out. `clients/protocol/evidence/` keeps two of these — one per flavour —
	# and the AC-13 test compares them with the server's own definition of a semantic core.
	_requests.append({ "token": token, "flavour": _flavour, "request": request })


## An `ActionResult` in this client's own words. The contract carries no display text, deliberately
## (`DD-13`), so every client maps the closed set to its own wording.
func _describe(result: Variant) -> String:
	if typeof(result) == TYPE_STRING:
		return "the world has no such action here" if result == "unavailable" else String(result)
	if typeof(result) != TYPE_DICTIONARY:
		return "?"
	if result.has("accepted"):
		var events: Variant = result["accepted"].get("events", [])
		return "accepted (%d fact(s))" % (events.size() if typeof(events) == TYPE_ARRAY else 0)
	if result.has("rejected"):
		var rejection: Variant = result["rejected"]
		if typeof(rejection) == TYPE_DICTIONARY:
			return "refused by a system (%s)" % String(rejection.get("system", {}).get("code", "?"))
		return "refused: %s" % REASON.get(String(rejection), String(rejection))
	return JSON.stringify(result)


# ------------------------------------------------------------------------------------------------
# What a player does, and what the autopilot does instead.
# ------------------------------------------------------------------------------------------------

func _unhandled_input(event: InputEvent) -> void:
	if _autopilot or not _world.is_seated() or _world.latest == null:
		return
	if event is InputEventKey and event.pressed and not event.echo:
		match event.keycode:
			KEY_1:
				_walk_beside(_barista())
			KEY_E, KEY_SPACE:
				_talk_to_the_barista()
			KEY_ESCAPE:
				_world.disconnect_from_world("closed by the player")
				get_tree().quit()


## Walks up to somebody: `move` requests, one stride at a time, which is how a client moves a person.
##
## The client works in its own space — where that person is *on the screen*, a metre below them — and
## converts once at the boundary. `MineWorldSpace` turns metres into the integer millimetres the
## contract declares, which is what keeps GDScript's single number type from sending `1500.0` where an
## `i32` is expected (`spike/FINDINGS.md` F9).
##
## It follows the reporting rule: each stride is at most [constant STRIDE] from the last position the
## server accepted, and the next is sent only once the server has answered the last. It does not ask
## whether a stride is allowed, nor whether the end of the walk is close enough to talk — the server
## answers both.
func _walk_beside(whom: String) -> void:
	var place := _world.latest.place()
	if place.is_empty() or whom.is_empty():
		_note("this world models no position for me, so there is nowhere to walk")
		return
	var them := MineWorldSpace.to_2d(_world.latest.location_of(whom).get("local"))
	_walk_from = MineWorldSpace.to_2d(_world.latest.self_location().get("local"))
	_walk_to = them + Vector2(0.0, STAND_BESIDE)
	_strides = 0
	_next_stride()


## Sends the next stride of the walk in progress, toward its end and at most [constant STRIDE] long.
func _next_stride() -> void:
	if _walk_to == null:
		return
	var remaining: Vector2 = _walk_to - _walk_from
	var step := remaining if remaining.length() <= STRIDE else remaining.normalized() * STRIDE
	_stride_to = _walk_from + step
	_strides += 1
	_stride_token = _world.submit("move", null, {
		"to": MineWorldSpace.location(_world.latest.place(), MineWorldSpace.from_2d(_stride_to)),
	})


## The server answered the stride in flight. Accepted: that is where the person now is, so the walk
## continues from there. Refused: the client stops and keeps to whatever the next observation shows,
## which is the reconciliation `server/PROTOCOL.md` §6.2 asks for — it does not argue or retry.
func _stride_answered(accepted: bool) -> void:
	_stride_token = ""
	if not accepted:
		_note("stride %d refused; stopping where the world says I am" % _strides)
		_walk_to = null
		return
	_walk_from = _stride_to
	if _walk_from.distance_to(_walk_to) < 0.001:
		_note("walked there in %d stride(s)" % _strides)
		_walk_to = null
		return
	_next_stride()


## Speaks to whoever the world says is the barista — found by tag, never by a hard-coded identity.
func _talk_to_the_barista() -> void:
	var barista := _barista()
	if barista.is_empty():
		_note("nobody here is a barista")
		return
	# Submitted whether or not the affordance says available, and that is on purpose: a client
	# reports intent and the server answers. Showing the prompt greyed out is presentation; refusing
	# to ask would be a client implementing the rule.
	_world.submit("talk", barista, { "utterance": GREETING }, _reported_location())


## What this client reports about where it is, which is the one field a 2D and a 3D client differ in.
##
## `null` for the 2D flavour: a client that models no continuous position has none to report, and the
## contract says so. The 3D flavour reports the position it walked to — a *report*, which the server
## is free to ignore because it evaluates its own authoritative state (`ENGINEERING_RULES.md` §8).
func _reported_location() -> Variant:
	if _flavour != "3d":
		return null
	var place := _world.latest.place()
	if place.is_empty():
		return null
	var here: Variant = _world.latest.self_location().get("local")
	var walked: Vector3 = MineWorldSpace.to_3d(here)
	return MineWorldSpace.location(
		place,
		MineWorldSpace.from_3d(walked),
		MineWorldSpace.yaw_from_3d_radians(0.0),
	)


func _barista() -> String:
	var found := _world.latest.tagged("barista")
	return "" if found.is_empty() else String(found[0])


## The scripted run: walk up, wait for the server to say the action is available, speak, and stop.
##
## It waits for the *server's* verdict rather than for a duration, which is the same discipline a
## player has: the prompt appears when the world says it may.
func _autopilot_step(observation: MineWorldObservation) -> void:
	var barista := _barista()
	if barista.is_empty():
		return
	match _step:
		1:
			_note("the server says talk to %s is %s%s" % [
				barista,
				"available" if observation.may("talk", barista) else "not available",
				"" if observation.may("talk", barista)
					else " (%s)" % _reason(observation, barista),
			])
			_walk_beside(barista)
			_step = 2
		2:
			if observation.may("talk", barista):
				_note("the server now says talk to %s is available" % barista)
				_talk_to_the_barista()
				_step = 3
		3:
			# A few frames, so the reply Alice may send has time to arrive and be rendered.
			_settled += 1
			if _settled > 30:
				_step = 4
				_report(observation)
				_capture_and_finish()


func _reason(observation: MineWorldObservation, target: String) -> String:
	var why: Variant = observation.unavailable_reason("talk", target)
	if typeof(why) == TYPE_STRING:
		return REASON.get(String(why), String(why))
	return JSON.stringify(why)


## What the run saw, printed so a transcript is readable, and the requests written where the AC-13
## test reads them.
func _report(observation: MineWorldObservation) -> void:
	_note("--- what this client was told ---")
	_note("world instance      %s" % _world.world_instance())
	_note("observer            %s" % _world.observer)
	_note("perceived           %s" % ", ".join(observation.ids()))
	for id in observation.ids():
		var tags: Variant = observation.entity(String(id)).get("tags", [])
		_note("  %s  tags %s  at %s m" % [
			id,
			JSON.stringify(tags),
			MineWorldSpace.to_2d(observation.location_of(String(id)).get("local")),
		])
		# Every record disclosed on this entity, by type: a record this client has no use for is
		# carried and ignored, never an error (step-08 C2, the far-side check of Q4).
		var types: Array = []
		for record in observation.entity(String(id)).get("components", []):
			types.append(String(record.get("component_type", "")))
		if not types.is_empty():
			_note("      components %s" % JSON.stringify(types))
		var passages: Dictionary = observation.component(String(id), "passages")
		if not passages.is_empty():
			_note("      doorways   %d (read through MineWorldObservation.component)" % [
				passages.get("leads_to", []).size(),
			])
	for offered in observation.affordances():
		_note("  affordance %s -> %s : %s" % [
			String(offered.get("action_type", "")),
			JSON.stringify(offered.get("target")),
			"available" if bool(offered.get("available", false))
				else JSON.stringify(offered.get("unavailable_reason")),
		])
	var heard: Variant = observation.own_component("conversation-history").get("heard", [])
	if typeof(heard) == TYPE_ARRAY and not heard.is_empty():
		_note("--- what I have been told (my own disclosed history) ---")
		for entry in heard:
			_note("  %s said %s" % [
				JSON.stringify(entry.get("speaker", {}).get("entity")),
				JSON.stringify(entry.get("utterance")),
			])
	else:
		_note("nobody has said anything to me yet")
	if not _requests_path.is_empty():
		var file := FileAccess.open(_requests_path, FileAccess.WRITE)
		if file == null:
			push_error("[demo] cannot write %s" % _requests_path)
		else:
			file.store_string(JSON.stringify(_requests, "  "))
			file.close()
			_note("wrote %d submitted request(s) to %s" % [_requests.size(), _requests_path])


## Saves what the scene is showing, when a windowed run was asked to.
##
## Headless renders nothing, so there is nothing to save: an empty image is worse than no image,
## because it looks like evidence.
func _capture() -> void:
	if _shot_path.is_empty():
		return
	if DisplayServer.get_name() == "headless":
		_note("headless renders nothing, so no screenshot was taken")
		return
	# One frame after the drawing that this run's last observation queued, so what is saved is what
	# was on the screen.
	await RenderingServer.frame_post_draw
	var image := get_viewport().get_texture().get_image()
	var saved := image.save_png(_shot_path)
	_note("screenshot %s (%s)" % [_shot_path, "saved" if saved == OK else "failed"])


## The end of a scripted run: save the picture if one was asked for, then stop.
func _capture_and_finish() -> void:
	await _capture()
	_finish()


func _finish() -> void:
	_world.disconnect_from_world("the scripted run is over")
	get_tree().quit()


# ------------------------------------------------------------------------------------------------
# Drawing what was said, and nothing that was not.
# ------------------------------------------------------------------------------------------------

func _draw() -> void:
	draw_rect(Rect2(Vector2.ZERO, get_viewport_rect().size), Color(0.09, 0.10, 0.12))
	_text(Vector2(16, 26), "MineWorld client protocol — demonstration scene", Color.WHITE, 18)
	_text(
		Vector2(16, 48),
		"seat '%s' = observer %s  ·  world %s  ·  frame %d" % [
			_world.seat, _world.observer, _world.world_instance(), _world.sequence,
		],
		Color(0.65, 0.70, 0.78),
	)
	if _world.latest == null:
		_text(Vector2(16, 76), "waiting for the first observation…", Color(0.8, 0.8, 0.5))
		return
	var observation := _world.latest
	_draw_people(observation)
	_draw_affordances(observation)
	_draw_history(observation)
	_text(
		Vector2(16, get_viewport_rect().size.y - 40),
		"[1] walk closer   [E] talk to the barista   [Esc] leave",
		Color(0.55, 0.60, 0.70),
	)
	_text(
		Vector2(16, get_viewport_rect().size.y - 20),
		"last answer: %s" % _last_answer,
		Color(0.85, 0.85, 0.60),
	)


func _draw_people(observation: MineWorldObservation) -> void:
	for id in observation.ids():
		var entity := observation.entity(String(id))
		if String(entity.get("entity_type", "")) != "person":
			continue
		var local := observation.location_of(String(id))
		if local.is_empty():
			continue
		var at := ROOM_CENTRE + MineWorldSpace.to_2d(local.get("local")) * PIXELS_PER_METRE
		var colour := _colour(entity)
		draw_circle(at, 9.0, colour)
		if String(id) == _world.observer:
			draw_arc(at, 14.0, 0.0, TAU, 24, Color(1, 1, 1, 0.5), 1.5)
		var facing: Variant = local.get("facing")
		if facing != null:
			var heading := MineWorldSpace.yaw_to_2d_radians(facing)
			draw_line(at, at + Vector2(cos(heading), sin(heading)) * 20.0, colour, 1.5)
		_text(at + Vector2(14, 4), String(id), colour)


func _draw_affordances(observation: MineWorldObservation) -> void:
	var y := 96.0
	_text(Vector2(560, y), "what the SERVER says I may do", Color(0.90, 0.90, 0.95))
	y += 22.0
	for offered in observation.affordances():
		var action := String(offered.get("action_type", ""))
		var target: Variant = offered.get("target")
		var who := "—" if target == null else String(target)
		var available := bool(offered.get("available", false))
		var line := "%s %s" % [VERB.get(action, action), who]
		var colour := Color(0.45, 0.90, 0.50) if available else Color(0.70, 0.45, 0.45)
		if not available:
			var why: Variant = offered.get("unavailable_reason")
			var named := String(why) if typeof(why) == TYPE_STRING else JSON.stringify(why)
			line += "  —  %s" % REASON.get(named, named)
		var reach: Variant = offered.get("requirement", {}).get("within_range")
		if reach != null:
			line += "  (needs %.1f m)" % (float(reach) / 1000.0)
		_text(Vector2(560, y), line, colour)
		y += 20.0


func _draw_history(observation: MineWorldObservation) -> void:
	var y := 96.0
	_text(Vector2(16, y), "what I have been told", Color(0.90, 0.90, 0.95))
	y += 22.0
	var heard: Variant = observation.own_component("conversation-history").get("heard", [])
	if typeof(heard) != TYPE_ARRAY or heard.is_empty():
		_text(Vector2(16, y), "nothing yet", Color(0.55, 0.60, 0.70))
		return
	for entry in heard:
		var speaker := String(entry.get("speaker", {}).get("entity", "?"))
		var said := String(entry.get("utterance", ""))
		_text(Vector2(16, y), "%s: %s" % [speaker, said.substr(0, 64)], Color(0.80, 0.85, 0.95))
		y += 18.0


func _colour(entity: Dictionary) -> Color:
	var tags: Variant = entity.get("tags", [])
	if typeof(tags) == TYPE_ARRAY:
		for tag in tags:
			if TAG_COLOUR.has(String(tag)):
				return TAG_COLOUR[String(tag)]
	return Color(0.75, 0.75, 0.80)


func _text(at: Vector2, line: String, colour: Color, size: int = 14) -> void:
	draw_string(_font, at, line, HORIZONTAL_ALIGNMENT_LEFT, -1, size, colour)


func _note(line: String) -> void:
	print("[demo] %s" % line)
