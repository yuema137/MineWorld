# MineWorld renderer-integration spike — 2D reference client.
#
# It renders an Observation top-down and submits an ActionIntent from a click. It contains no
# world rule: it never measures a distance, never decides whether anybody is available, never
# decides whether an action exists. Availability, its reason and the reach an action needs all
# arrive in the Affordance the server computed; this file only draws them and reports clicks.
#
# The run is scripted so it is reproducible without a human at the keyboard, but the clicks are
# real InputEventMouseButtons fed through Input.parse_input_event, so the acquisition path is
# the one a player would take.

extends Node2D

const SERVER := "ws://127.0.0.1:7878/ws"
const CLIENT_TAG := "2d"

# Screen mapping. World +y is north and Godot 2D's +y points down, so the y axis is negated.
# That sign flip is the contract's silence about handedness, paid for once (FINDINGS.md F5).
const ROOM_CENTRE := Vector2(380, 300)
const MM_TO_PX := 0.06

# What a client is allowed to decide for itself: wording and colour.
const VERB := {"talk": "Talk to", "pick-up": "Pick up", "move-to": "Walk to"}
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
	"player": Color(0.30, 0.75, 1.00),
	"barista": Color(1.00, 0.62, 0.25),
	"regular": Color(0.62, 0.62, 0.68),
	"crockery": Color(0.85, 0.85, 0.55),
}

var socket := WebSocketPeer.new()
var observation: Dictionary = {}
var font: Font = ThemeDB.fallback_font

var next_action_id := 7000
var log_lines: PackedStringArray = []
var last_result := ""
var step := 0
var settled := 0
var shots := 0
var reported := false


func _ready() -> void:
	socket.connect_to_url(SERVER)
	note("2D client connecting to %s" % SERVER)


func _process(_delta: float) -> void:
	socket.poll()
	while socket.get_available_packet_count() > 0:
		_receive(socket.get_packet().get_string_from_utf8())
	if observation.is_empty():
		return
	queue_redraw()
	_advance()


# ---------------------------------------------------------------------------------------------
# Protocol
# ---------------------------------------------------------------------------------------------

func _receive(text: String) -> void:
	var frame = JSON.parse_string(text)
	if typeof(frame) != TYPE_DICTIONARY:
		return
	match frame.get("t", ""):
		"observation":
			observation = frame["observation"]
			if not reported:
				reported = true
				_report_id_encoding()
		"result":
			var result = frame["result"]
			last_result = _describe_result(result)
			note("server answered: %s" % last_result)
		"error":
			last_result = "protocol error: %s" % str(frame.get("detail", ""))
			note(last_result)


# The 64-bit id problem, checked from inside Godot rather than argued about on paper.
#
# There is one encoding now: the contract's own. Every id arrives as a decimal string, so the
# demonstration no longer needs a second copy of the frame — this client produces the corruption
# itself, by doing to the string what a JSON number would have forced it to do.
func _report_id_encoding() -> void:
	var alice := _entity_named("Alice")
	if alice.is_empty():
		return
	var alice_id: String = alice["id"]
	note("id from the contract  : %s  typeof=%d (2=int 4=string)" % [alice_id, typeof(alice["id"])])
	note("id -> int()           : %d" % int(alice_id))
	note("id if it had been a JSON number: %.0f" % float(alice_id))
	# The mug is a different entity whose id is two away from Alice's. Exact as strings; the same
	# value as doubles.
	var mug := _entity_named("Chipped mug")
	if not mug.is_empty():
		var mug_id: String = mug["id"]
		note("collision check: alice=%s mug=%s exact_collide=%s as_double_collide=%s"
			% [alice_id, mug_id, str(alice_id == mug_id),
			   str(float(alice_id) == float(mug_id))])
		# F2, the half a protocol-level encoder could not reach: an EntityId inside a component
		# payload. The contract protects it because the rule is on the type.
		for component in mug.get("components", []):
			if component["component_type"] == "ownership":
				var owner = component["payload"]["owner"]
				note("F2 payload id: owner=%s protected=%s (a component payload is opaque to any encoder)"
					% [str(owner), str(typeof(owner) == TYPE_STRING)])
	# And the other side of the same rule: a large integer that is *not* an id is still a number,
	# because nothing may guess. A client that needs it exactly must not read it as a number.
	var place := _entity_named("Lakeside Cafe")
	for component in place.get("components", []):
		if component["component_type"] == "signage":
			var catalogue = component["payload"]["catalogue_id"]
			note("not an id: sent 9007199254740999, parsed %.0f, protected=%s"
				% [float(catalogue), str(typeof(catalogue) == TYPE_STRING)])


func send(frame: Dictionary) -> void:
	socket.send_text(JSON.stringify(frame))


func note(text: String) -> void:
	print("[2d] ", text)
	log_lines.append(text)
	if log_lines.size() > 8:
		log_lines.remove_at(0)
	if socket.get_ready_state() == WebSocketPeer.STATE_OPEN:
		send({"t": "note", "text": "[2d] " + text})


# Builds a real ActionIntent in its wire form.
#
# action_id is invented here, which the contract says a client may not do (FINDINGS.md F4).
# issued_at is the world time of the last observation, not of this request (F4).
# actor_location is deliberately absent: a 2D client models no continuous position of its own,
# so it reports none, and the server evaluates against its own state (F6).
func submit(action_type: String, target, payload: Dictionary) -> void:
	next_action_id += 1
	var intent := {
		"action_id": str(next_action_id),
		"actor": observation["observer"],
		"action_type": action_type,
		"target": target,
		"payload": {"action_type": action_type, "payload": payload},
		"issued_at": int(observation["at"]),
		"actor_location": null,
	}
	send({"t": "intent", "client": CLIENT_TAG, "intent": intent})
	note("submitted %s -> %s" % [action_type, str(target)])


# ---------------------------------------------------------------------------------------------
# Reading the observation. Lookups only: nothing here computes a world fact.
# ---------------------------------------------------------------------------------------------

func _name_of(entity: Dictionary) -> String:
	for component in entity.get("components", []):
		if component["component_type"] == "display-name":
			return component["payload"]["name"]
	return ""


func _component(entity: Dictionary, type_name: String) -> Variant:
	for component in entity.get("components", []):
		if component["component_type"] == type_name:
			return component["payload"]
	return null


func _entity(id: String) -> Dictionary:
	for entity in observation.get("entities", []):
		if entity["id"] == id:
			return entity
	return {}


func _entity_named(wanted: String) -> Dictionary:
	for entity in observation.get("entities", []):
		if _name_of(entity) == wanted:
			return entity
	return {}


func _affordance(action_type: String, target: String) -> Dictionary:
	for affordance in observation.get("affordances", []):
		if affordance["action_type"] == action_type and affordance["target"] == target:
			return affordance
	return {}


func _describe_result(result) -> String:
	if typeof(result) == TYPE_STRING:
		return "unavailable — no system provides that action here"
	if result.has("accepted"):
		return "accepted (%d event(s))" % result["accepted"]["events"].size()
	if result.has("rejected"):
		var reason = result["rejected"]
		if typeof(reason) == TYPE_STRING:
			return "refused — %s" % REASON.get(reason, reason)
		return "refused — system said %s" % str(reason)
	return str(result)


# ---------------------------------------------------------------------------------------------
# The scripted run. Real mouse events, fed at reproducible moments.
# ---------------------------------------------------------------------------------------------

func _advance() -> void:
	settled += 1
	match step:
		0:
			if settled > 30:
				_shoot("2d-01-arrived")
				_click(_screen_of(_entity_named("Bob")))
				step = 1
				settled = 0
		1:
			# Bob is refused by the server, not by this client. Show the answer, then walk.
			if settled > 20:
				_shoot("2d-02-bob-refused")
				_click(_floor_point(1500, 0))
				step = 2
				settled = 0
		2:
			# Wait for the SERVER to say the talk affordance is available. The client never
			# measures the distance itself; it watches the affordance flip.
			var alice := _entity_named("Alice")
			if not alice.is_empty():
				var affordance := _affordance("talk", alice["id"])
				if not affordance.is_empty() and affordance["available"]:
					_shoot("2d-03-prompt-available")
					_click(_screen_of(alice))
					step = 3
					settled = 0
		3:
			if settled > 20:
				_shoot("2d-04-talk-accepted")
				step = 4
				settled = 0
		4:
			if settled > 5:
				socket.close()
				get_tree().quit(0)


func _click(where: Vector2) -> void:
	var press := InputEventMouseButton.new()
	press.button_index = MOUSE_BUTTON_LEFT
	press.pressed = true
	press.position = where
	Input.parse_input_event(press)


func _unhandled_input(event: InputEvent) -> void:
	if not (event is InputEventMouseButton and event.pressed):
		return
	var at: Vector2 = event.position
	var hit := _entity_under(at)
	if hit.is_empty():
		# Empty floor: ask to stand there. The server decides how, and how fast.
		var world := _world_of(at)
		# Every number sent back must be an explicit integer. Godot has one number type and
		# JSON.stringify writes 1500.0, which serde refuses for an i32 (FINDINGS.md F9).
		submit("move-to", null, {
			"to": {"x": int(round(world.x)), "y": int(round(world.y)), "z": 0},
			"facing": null,
		})
		return
	var kind: String = "talk" if hit["entity_type"] == "person" else "pick-up"
	submit(kind, hit["id"], {"topic": "greeting"} if kind == "talk" else {})


func _entity_under(at: Vector2) -> Dictionary:
	for entity in observation.get("entities", []):
		if entity["location"] == null:
			continue
		if _name_of(entity) == "You":
			continue
		var body = _component(entity, "body")
		var radius: float = 16.0
		if body != null:
			radius = max(14.0, float(body["radius_mm"]) * MM_TO_PX * 2.2)
		if _screen_of(entity).distance_to(at) <= radius:
			return entity
	return {}


# ---------------------------------------------------------------------------------------------
# Drawing
# ---------------------------------------------------------------------------------------------

func _screen_of(entity: Dictionary) -> Vector2:
	if entity.is_empty() or entity["location"] == null:
		return Vector2.ZERO
	var local = entity["location"]["local"]
	return _floor_point(float(local["x"]), float(local["y"]))


func _floor_point(mm_x: float, mm_y: float) -> Vector2:
	return ROOM_CENTRE + Vector2(mm_x * MM_TO_PX, -mm_y * MM_TO_PX)


func _world_of(at: Vector2) -> Vector2:
	var offset := at - ROOM_CENTRE
	return Vector2(offset.x / MM_TO_PX, -offset.y / MM_TO_PX)


func _colour_of(entity: Dictionary) -> Color:
	for tag in entity.get("tags", []):
		if TAG_COLOUR.has(tag):
			return TAG_COLOUR[tag]
	return Color(0.7, 0.7, 0.7)


func _draw() -> void:
	draw_rect(Rect2(Vector2.ZERO, get_viewport_rect().size), Color(0.10, 0.11, 0.13))
	if observation.is_empty():
		return

	# The room, built from the place-extent component. No dimension is hardcoded here.
	var place := _entity_named("Lakeside Cafe")
	if not place.is_empty():
		var extent = _component(place, "place-extent")
		if extent != null:
			var top_left := _floor_point(float(extent["min"]["x"]), float(extent["max"]["y"]))
			var bottom_right := _floor_point(float(extent["max"]["x"]), float(extent["min"]["y"]))
			var room := Rect2(top_left, bottom_right - top_left)
			draw_rect(room, Color(0.16, 0.17, 0.20))
			draw_rect(room, Color(0.45, 0.47, 0.55), false, 2.0)
		var sign_payload = _component(place, "signage")
		if sign_payload != null:
			draw_string(font, Vector2(24, 28), sign_payload["text"], HORIZONTAL_ALIGNMENT_LEFT,
				-1, 18, Color(0.85, 0.87, 0.95))

	for entity in observation["entities"]:
		if entity["location"] == null:
			continue
		_draw_entity(entity)

	_draw_panel()


func _draw_entity(entity: Dictionary) -> void:
	var at := _screen_of(entity)
	var body = _component(entity, "body")
	var radius: float = 10.0
	if body != null:
		radius = max(6.0, float(body["radius_mm"]) * MM_TO_PX * 2.2)
	var colour := _colour_of(entity)
	draw_circle(at, radius, colour)

	# Facing, from Orientation.yaw. Compass bearing: 0 is +y (north), increasing toward +x.
	var facing = entity["location"]["facing"]
	if facing != null:
		var yaw := deg_to_rad(float(facing["yaw"]) / 1000.0)
		var direction := Vector2(sin(yaw), -cos(yaw))
		draw_line(at, at + direction * (radius + 12.0), colour, 2.0)

	draw_string(font, at + Vector2(-28, -radius - 8), _name_of(entity),
		HORIZONTAL_ALIGNMENT_LEFT, -1, 14, Color(0.92, 0.93, 0.98))

	# The interaction prompt, straight out of the server's Affordance.
	var kind: String = "talk" if entity["entity_type"] == "person" else "pick-up"
	var affordance := _affordance(kind, entity["id"])
	if affordance.is_empty():
		return
	var label: String = "%s %s" % [VERB.get(kind, kind), _name_of(entity)]
	if affordance["available"]:
		draw_string(font, at + Vector2(-40, radius + 20), "[click] " + label,
			HORIZONTAL_ALIGNMENT_LEFT, -1, 13, Color(0.55, 0.95, 0.60))
	else:
		var why: String = REASON.get(affordance["unavailable_reason"],
			str(affordance["unavailable_reason"]))
		draw_string(font, at + Vector2(-40, radius + 20), "%s — %s" % [label, why],
			HORIZONTAL_ALIGNMENT_LEFT, -1, 13, Color(0.60, 0.60, 0.66))
		# What the action needs, shown without being evaluated here.
		var reach = affordance["requirement"]["within_range"]
		if reach != null:
			draw_string(font, at + Vector2(-40, radius + 34),
				"needs %.1f m" % (float(reach) / 1000.0), HORIZONTAL_ALIGNMENT_LEFT, -1, 12,
				Color(0.50, 0.50, 0.58))


func _draw_panel() -> void:
	var x := 780.0
	draw_rect(Rect2(Vector2(x - 12, 0), Vector2(240, 640)), Color(0.07, 0.08, 0.10))
	var y := 28.0
	draw_string(font, Vector2(x, y), "2D client", HORIZONTAL_ALIGNMENT_LEFT, -1, 16,
		Color(0.85, 0.87, 0.95))
	y += 22
	draw_string(font, Vector2(x, y), "observer %s" % observation["observer"],
		HORIZONTAL_ALIGNMENT_LEFT, -1, 12, Color(0.6, 0.62, 0.7))
	y += 16
	draw_string(font, Vector2(x, y), "world time t%d" % int(observation["at"]),
		HORIZONTAL_ALIGNMENT_LEFT, -1, 12, Color(0.6, 0.62, 0.7))
	y += 26
	draw_string(font, Vector2(x, y), "affordances (server)", HORIZONTAL_ALIGNMENT_LEFT, -1, 13,
		Color(0.8, 0.82, 0.9))
	y += 18
	for affordance in observation["affordances"]:
		var target_name := "—"
		if affordance["target"] != null:
			target_name = _name_of(_entity(affordance["target"]))
		var line := "%s %s" % [affordance["action_type"], target_name]
		var colour := Color(0.55, 0.95, 0.60) if affordance["available"] else Color(0.65, 0.5, 0.5)
		draw_string(font, Vector2(x, y), line, HORIZONTAL_ALIGNMENT_LEFT, -1, 12, colour)
		y += 14
		if not affordance["available"]:
			draw_string(font, Vector2(x + 10, y),
				REASON.get(affordance["unavailable_reason"], str(affordance["unavailable_reason"])),
				HORIZONTAL_ALIGNMENT_LEFT, -1, 11, Color(0.5, 0.5, 0.58))
			y += 14
	y += 16
	draw_string(font, Vector2(x, y), "last answer", HORIZONTAL_ALIGNMENT_LEFT, -1, 13,
		Color(0.8, 0.82, 0.9))
	y += 18
	draw_string(font, Vector2(x, y), last_result if last_result != "" else "—",
		HORIZONTAL_ALIGNMENT_LEFT, 220, 12, Color(0.9, 0.85, 0.6))
	y += 30
	for line in log_lines:
		draw_string(font, Vector2(x, y), line, HORIZONTAL_ALIGNMENT_LEFT, 220, 10,
			Color(0.45, 0.47, 0.55))
		y += 12


func _shoot(name: String) -> void:
	shots += 1
	await RenderingServer.frame_post_draw
	var image := get_viewport().get_texture().get_image()
	var error := image.save_png("res://%s.png" % name)
	print("[2d] captured %s.png (error=%d)" % [name, error])
