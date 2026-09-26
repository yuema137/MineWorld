# MineWorld renderer-integration spike — 3D reference client.
#
# Embodied first person: a CharacterBody3D with gravity and a capsule collider, a Camera3D at eye
# height, and a RayCast3D that recovers the EntityId a body was built from. The player walks
# toward Alice, looks at her, and presses E. It contains no world rule: it never measures the
# distance to Alice, never decides she is available, never decides that "talk" exists. The prompt
# it draws is the server's Affordance, and the interact key only reports that it was pressed.
#
# Coordinates. MineWorld is x east / y north / z up in millimetres; Godot is +X right, +Y up,
# -Z forward in metres. The conversion is one function each way, and the y/z swap and the sign
# flip on the bearing are the price of the contract not stating a frame (FINDINGS.md F5). Note
# that the 2D client's conversion is a *different* one from the same contract.

extends Node3D

const SERVER := "ws://127.0.0.1:7878/ws"
const CLIENT_TAG := "3d"

const EYE_HEIGHT := 1.62
const WALK_SPEED := 2.4
const GRAVITY := 18.0
const RAY_REACH := 3.0

const VERB := {"talk": "Talk to", "pick-up": "Pick up", "move-to": "Walk to"}
const REASON := {
	"busy": "busy", "too_far_away": "too far away", "permission_denied": "not allowed",
	"no_supported_interaction": "nothing to do there", "target_unavailable": "not available",
	"unavailable": "refused", "precondition_failed": "cannot be done",
}
const TAG_COLOUR := {
	"barista": Color(0.95, 0.55, 0.20),
	"regular": Color(0.55, 0.57, 0.65),
	"crockery": Color(0.90, 0.88, 0.60),
}

var socket := WebSocketPeer.new()
var observation: Dictionary = {}

var player: CharacterBody3D
var camera: Camera3D
var ray: RayCast3D
var prompt: Label
var readout: Label
var built := false
# entity id (wire form, a decimal string) -> the node drawn for it. DD-14 in both directions.
var bodies: Dictionary = {}

var next_action_id := 8000
var last_result := ""
var step := 0
var settled := 0
var walking := false
var interact_pressed := false
var since_report := 0.0
var reported := false


func _ready() -> void:
	socket.connect_to_url(SERVER)
	note("3D client connecting to %s" % SERVER)


# ---------------------------------------------------------------------------------------------
# Coordinates
# ---------------------------------------------------------------------------------------------

func to_godot(mm_x: float, mm_y: float, mm_z: float) -> Vector3:
	return Vector3(mm_x / 1000.0, mm_z / 1000.0, -mm_y / 1000.0)


func to_world(point: Vector3) -> Vector3:
	# Back to millimetres, rounded to integers: the contract is integer-only and serde refuses
	# a float where an i32 is declared (FINDINGS.md F9).
	return Vector3(round(point.x * 1000.0), round(-point.z * 1000.0), round(point.y * 1000.0))


func bearing_to_godot_yaw(millidegrees: float) -> float:
	# Bearing: 0 faces +y (north), increasing toward +x. Godot: 0 faces -Z, increasing
	# counter-clockwise. Hence the negation.
	return -deg_to_rad(millidegrees / 1000.0)


func godot_yaw_to_bearing(yaw: float) -> int:
	return int(round(fposmod(-rad_to_deg(yaw), 360.0) * 1000.0))


# ---------------------------------------------------------------------------------------------
# Building the scene out of the observation
# ---------------------------------------------------------------------------------------------

func _build() -> void:
	built = true
	var environment := WorldEnvironment.new()
	var settings := Environment.new()
	settings.background_mode = Environment.BG_COLOR
	settings.background_color = Color(0.10, 0.12, 0.16)
	settings.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	settings.ambient_light_color = Color(0.55, 0.57, 0.62)
	settings.ambient_light_energy = 0.9
	environment.environment = settings
	add_child(environment)

	var sun := DirectionalLight3D.new()
	sun.rotation = Vector3(deg_to_rad(-55), deg_to_rad(35), 0)
	sun.light_energy = 1.4
	add_child(sun)

	_build_place()
	for entity in observation["entities"]:
		if entity["location"] == null:
			continue
		if _name_of(entity) == "You":
			_build_player(entity)
		else:
			_build_body(entity)
	_build_hud()


# The floor and four walls, from the place-extent component. No dimension is written here.
func _build_place() -> void:
	var place := _entity_named("Lakeside Cafe")
	var extent = _component(place, "place-extent")
	if extent == null:
		return
	var low := to_godot(float(extent["min"]["x"]), float(extent["min"]["y"]), float(extent["min"]["z"]))
	var high := to_godot(float(extent["max"]["x"]), float(extent["max"]["y"]), float(extent["max"]["z"]))
	var width: float = abs(high.x - low.x)
	var depth: float = abs(high.z - low.z)
	var height: float = abs(high.y - low.y)
	var centre := (low + high) * 0.5

	_slab(Vector3(centre.x, low.y - 0.1, centre.z), Vector3(width, 0.2, depth), Color(0.30, 0.28, 0.26))
	var thickness := 0.2
	_slab(Vector3(centre.x, height * 0.5, low.z - thickness * 0.5), Vector3(width, height, thickness), Color(0.42, 0.40, 0.44))
	_slab(Vector3(centre.x, height * 0.5, high.z + thickness * 0.5), Vector3(width, height, thickness), Color(0.42, 0.40, 0.44))
	_slab(Vector3(low.x - thickness * 0.5, height * 0.5, centre.z), Vector3(thickness, height, depth), Color(0.36, 0.35, 0.39))
	_slab(Vector3(high.x + thickness * 0.5, height * 0.5, centre.z), Vector3(thickness, height, depth), Color(0.36, 0.35, 0.39))


func _slab(at: Vector3, size: Vector3, colour: Color) -> void:
	var body := StaticBody3D.new()
	body.position = at
	var mesh := MeshInstance3D.new()
	var box := BoxMesh.new()
	box.size = size
	mesh.mesh = box
	var material := StandardMaterial3D.new()
	material.albedo_color = colour
	mesh.material_override = material
	body.add_child(mesh)
	var shape := CollisionShape3D.new()
	var collider := BoxShape3D.new()
	collider.size = size
	shape.shape = collider
	body.add_child(shape)
	add_child(body)


func _build_player(entity: Dictionary) -> void:
	var local = entity["location"]["local"]
	player = CharacterBody3D.new()
	player.position = to_godot(float(local["x"]), float(local["y"]), 0) + Vector3(0, 0.9, 0)
	player.rotation.y = bearing_to_godot_yaw(float(entity["location"]["facing"]["yaw"]))
	var body = _component(entity, "body")
	var shape := CollisionShape3D.new()
	var capsule := CapsuleShape3D.new()
	capsule.height = float(body["height_mm"]) / 1000.0
	capsule.radius = float(body["radius_mm"]) / 1000.0
	shape.shape = capsule
	player.add_child(shape)

	camera = Camera3D.new()
	camera.position = Vector3(0, EYE_HEIGHT - 0.9, 0)
	camera.fov = 72
	player.add_child(camera)

	ray = RayCast3D.new()
	ray.target_position = Vector3(0, 0, -RAY_REACH)
	ray.enabled = true
	camera.add_child(ray)
	add_child(player)


# A rendered body, with the EntityId attached as node metadata: DD-14, exactly as the earlier
# spike did it, and the raycast recovers it below.
func _build_body(entity: Dictionary) -> void:
	var local = entity["location"]["local"]
	var body = _component(entity, "body")
	var height: float = float(body["height_mm"]) / 1000.0
	var radius: float = float(body["radius_mm"]) / 1000.0

	var node := StaticBody3D.new()
	node.position = to_godot(float(local["x"]), float(local["y"]), float(local["z"])) + Vector3(0, height * 0.5, 0)
	node.rotation.y = bearing_to_godot_yaw(float(entity["location"]["facing"]["yaw"]))
	node.set_meta("entity_id", entity["id"])

	var mesh := MeshInstance3D.new()
	var capsule := CapsuleMesh.new()
	capsule.height = max(height, radius * 2.0 + 0.01)
	capsule.radius = radius
	mesh.mesh = capsule
	var material := StandardMaterial3D.new()
	material.albedo_color = _colour_of(entity)
	mesh.material_override = material
	node.add_child(mesh)

	var shape := CollisionShape3D.new()
	var collider := CapsuleShape3D.new()
	collider.height = max(height, radius * 2.0 + 0.01)
	collider.radius = radius
	shape.shape = collider
	node.add_child(shape)

	var label := Label3D.new()
	label.text = _name_of(entity)
	label.position = Vector3(0, height * 0.5 + 0.28, 0)
	label.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	label.font_size = 96
	label.pixel_size = 0.0022
	label.no_depth_test = true
	node.add_child(label)

	add_child(node)
	bodies[entity["id"]] = node


func _build_hud() -> void:
	var layer := CanvasLayer.new()
	prompt = Label.new()
	prompt.anchor_left = 0.0
	prompt.anchor_top = 0.0
	prompt.position = Vector2(0, 470)
	prompt.size = Vector2(1000, 60)
	prompt.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	prompt.add_theme_font_size_override("font_size", 22)
	layer.add_child(prompt)

	readout = Label.new()
	readout.position = Vector2(16, 12)
	readout.size = Vector2(620, 220)
	readout.add_theme_font_size_override("font_size", 13)
	layer.add_child(readout)

	var reticle := Label.new()
	reticle.position = Vector2(494, 306)
	reticle.text = "+"
	reticle.add_theme_font_size_override("font_size", 20)
	layer.add_child(reticle)
	add_child(layer)


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
			last_result = _describe_result(frame["result"])
			note("server answered: %s" % last_result)
		"error":
			last_result = "protocol error: %s" % str(frame.get("detail", ""))
			note(last_result)


# One encoding now: the contract's own. The corruption is produced here, from the string, rather
# than by asking the server for a second copy of the frame.
func _report_id_encoding() -> void:
	var alice := _entity_named("Alice")
	var mug := _entity_named("Chipped mug")
	if alice.is_empty() or mug.is_empty():
		return
	var alice_id: String = alice["id"]
	var mug_id: String = mug["id"]
	note("contract alice=%s mug=%s (exact, typeof=%d)" % [alice_id, mug_id, typeof(alice["id"])])
	note("as JSON numbers alice=%.0f mug=%.0f collide=%s"
		% [float(alice_id), float(mug_id), str(float(alice_id) == float(mug_id))])
	# F2: an EntityId inside a component payload, which no protocol-level encoder could reach.
	for component in mug.get("components", []):
		if component["component_type"] == "ownership":
			var owner = component["payload"]["owner"]
			note("F2 payload id: owner=%s protected=%s" % [str(owner), str(typeof(owner) == TYPE_STRING)])


func send(frame: Dictionary) -> void:
	socket.send_text(JSON.stringify(frame))


func note(text: String) -> void:
	print("[3d] ", text)
	if socket.get_ready_state() == WebSocketPeer.STATE_OPEN:
		send({"t": "note", "text": "[3d] " + text})


# The same fields the 2D client sends, with one deliberate difference: actor_location is present,
# because this client does model a continuous position and the contract says it may report one.
func submit(action_type: String, target, payload: Dictionary, with_location: bool) -> void:
	next_action_id += 1
	var location = null
	if with_location:
		var here := to_world(player.global_position - Vector3(0, 0.9, 0))
		location = {
			"place": observation["self_location"]["place"],
			"local": {"x": int(here.x), "y": int(here.y), "z": 0},
			"facing": {"yaw": godot_yaw_to_bearing(player.rotation.y), "pitch": null},
		}
	var intent := {
		"action_id": str(next_action_id),
		"actor": observation["observer"],
		"action_type": action_type,
		"target": target,
		"payload": {"action_type": action_type, "payload": payload},
		"issued_at": int(observation["at"]),
		"actor_location": location,
	}
	send({"t": "intent", "client": CLIENT_TAG, "intent": intent})


# ---------------------------------------------------------------------------------------------
# Reading the observation
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


func _affordance_for(target: String) -> Dictionary:
	for affordance in observation.get("affordances", []):
		if affordance["target"] == target:
			return affordance
	return {}


func _colour_of(entity: Dictionary) -> Color:
	for tag in entity.get("tags", []):
		if TAG_COLOUR.has(tag):
			return TAG_COLOUR[tag]
	return Color(0.7, 0.7, 0.7)


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
# Embodiment
# ---------------------------------------------------------------------------------------------

func _physics_process(delta: float) -> void:
	if player == null:
		return
	if not player.is_on_floor():
		player.velocity.y -= GRAVITY * delta
	else:
		player.velocity.y = 0.0
	var forward := -player.global_transform.basis.z
	var speed := WALK_SPEED if walking else 0.0
	player.velocity.x = forward.x * speed
	player.velocity.z = forward.z * speed
	player.move_and_slide()


# What the camera is pointing at, recovered from the collider's metadata (DD-14).
func _targeted() -> String:
	if ray == null or not ray.is_colliding():
		return ""
	var collider = ray.get_collider()
	if collider == null or not collider.has_meta("entity_id"):
		return ""
	return collider.get_meta("entity_id")


func _input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and event.keycode == KEY_E:
		interact_pressed = true


func _press_interact() -> void:
	var key := InputEventKey.new()
	key.keycode = KEY_E
	key.physical_keycode = KEY_E
	key.pressed = true
	Input.parse_input_event(key)


# ---------------------------------------------------------------------------------------------
# Frame
# ---------------------------------------------------------------------------------------------

func _process(delta: float) -> void:
	socket.poll()
	while socket.get_available_packet_count() > 0:
		_receive(socket.get_packet().get_string_from_utf8())
	if observation.is_empty():
		return
	if not built:
		_build()
		return

	_draw_hud()
	_report_position(delta)
	_advance(delta)

	if interact_pressed:
		interact_pressed = false
		_interact()


# Reports where this client's own body is. The server walks its authoritative body toward it at
# its own speed; this client never moves the world.
func _report_position(delta: float) -> void:
	since_report += delta
	if since_report < 0.1 or not walking:
		return
	since_report = 0.0
	var here := to_world(player.global_position - Vector3(0, 0.9, 0))
	submit("move-to", null, {
		"to": {"x": int(here.x), "y": int(here.y), "z": 0},
		"facing": {"yaw": godot_yaw_to_bearing(player.rotation.y), "pitch": null},
	}, false)


# Pressing interact reports one fact — "the player pressed interact while targeting this
# entity" — and asks. It does not check whether the answer will be yes.
func _interact() -> void:
	var target := _targeted()
	if target == "":
		note("interact pressed with nothing targeted")
		return
	note("interact pressed while targeting %s" % target)
	submit("talk", target, {"topic": "greeting"}, true)


func _draw_hud() -> void:
	var target := _targeted()
	if target == "":
		prompt.text = ""
	else:
		var entity := _entity(target)
		var affordance := _affordance_for(target)
		if affordance.is_empty():
			prompt.text = ""
		else:
			var kind: String = affordance["action_type"]
			var label: String = "%s %s" % [VERB.get(kind, kind), _name_of(entity)]
			if affordance["available"]:
				prompt.text = "[E] %s" % label
				prompt.add_theme_color_override("font_color", Color(0.55, 0.98, 0.60))
			else:
				var why: String = REASON.get(affordance["unavailable_reason"],
					str(affordance["unavailable_reason"]))
				var reach = affordance["requirement"]["within_range"]
				var needs := "" if reach == null else "  (needs %.1f m)" % (float(reach) / 1000.0)
				prompt.text = "%s — %s%s" % [label, why, needs]
				prompt.add_theme_color_override("font_color", Color(0.72, 0.70, 0.74))

	var lines := PackedStringArray()
	lines.append("3D client   observer %s   world time t%d" % [observation["observer"], int(observation["at"])])
	var self_local = observation["self_location"]["local"]
	lines.append("server body   x=%d y=%d mm   yaw=%d mdeg"
		% [int(self_local["x"]), int(self_local["y"]), int(observation["self_location"]["facing"]["yaw"])])
	var here := to_world(player.global_position - Vector3(0, 0.9, 0))
	lines.append("client body   x=%d y=%d mm   yaw=%d mdeg"
		% [int(here.x), int(here.y), godot_yaw_to_bearing(player.rotation.y)])
	lines.append("targeting     %s" % (_targeted() if _targeted() != "" else "—"))
	lines.append("")
	lines.append("affordances (server):")
	for affordance in observation["affordances"]:
		var who := "—"
		if affordance["target"] != null:
			who = _name_of(_entity(affordance["target"]))
		var state: String = "available" if affordance["available"] else REASON.get(
			affordance["unavailable_reason"], str(affordance["unavailable_reason"]))
		lines.append("   %-9s %-14s %s" % [affordance["action_type"], who, state])
	lines.append("")
	lines.append("last answer: %s" % (last_result if last_result != "" else "—"))
	readout.text = "\n".join(lines)


# ---------------------------------------------------------------------------------------------
# The scripted run: look at Alice, walk up, press E when the server says it is possible.
# ---------------------------------------------------------------------------------------------

func _advance(delta: float) -> void:
	settled += 1
	# The step is advanced *before* the capture in every branch: _shoot awaits a frame, and
	# _process keeps running during the await, so a branch that awaited first would re-enter.
	match step:
		0:
			if settled > 40:
				step = 1
				settled = 0
				await _shoot("3d-01-entered")
		1:
			# Turn toward Alice's rendered body. Aiming is acquisition, not a rule.
			var alice := _entity_named("Alice")
			var node: Node3D = bodies.get(alice["id"])
			var to_her: Vector3 = node.global_position - player.global_position
			var wanted := atan2(-to_her.x, -to_her.z)
			player.rotation.y = lerp_angle(player.rotation.y, wanted, min(1.0, delta * 6.0))
			if abs(angle_difference(player.rotation.y, wanted)) < 0.01 and settled > 30:
				step = 2
				settled = 0
				walking = true
				await _shoot("3d-02-looking-at-alice")
		2:
			# Walk until the SERVER says the talk affordance for whatever is targeted is
			# available. The client never measures the distance itself.
			var target := _targeted()
			var affordance := _affordance_for(target) if target != "" else {}
			if not affordance.is_empty() and affordance["available"]:
				step = 3
				settled = 0
				walking = false
				_press_interact()
				await _shoot("3d-03-prompt-available")
			elif settled > 500:
				step = 4
				settled = 0
				walking = false
				note("gave up walking: never targeted an available affordance")
		3:
			if settled > 25:
				step = 4
				settled = 0
				await _shoot("3d-04-talk-accepted")
		4:
			if settled > 15:
				step = 5
				socket.close()
				get_tree().quit(0)


func _shoot(name: String) -> void:
	await RenderingServer.frame_post_draw
	var image := get_viewport().get_texture().get_image()
	var error := image.save_png("res://%s.png" % name)
	print("[3d] captured %s.png (error=%d)" % [name, error])
