## First-person walker.
##
## Minecraft is the *control* reference and nothing else (ART_DIRECTION sec.9):
## WASD, mouse look, gravity, collision, and deliberately no jump -- this is a
## walking simulator. Speeds are real human speeds, not game speeds: 1.45 m/s
## stroll, 3.1 m/s jog. Eye height 1.66 m. Those three numbers do more for the
## "am I a person in a town" feeling than any amount of geometry.
class_name Player
extends CharacterBody3D

const WALK_SPEED := 1.45
const JOG_SPEED := 3.10
const ACCEL := 9.0
const DECEL := 12.0
const EYE_HEIGHT := 1.66
const MOUSE_SENS := 0.0016
const PITCH_LIMIT := deg_to_rad(84.0)

@export var look_enabled := true
## Set by the scripted runner: headless has no capturable mouse, so the
## capture check would otherwise swallow every synthetic motion event.
var scripted_look := false

var cam: Camera3D
var _pitch := 0.0
var _bob := 0.0
var _speed_smooth := 0.0


func _ready() -> void:
	collision_layer = 0
	collision_mask = Build.LAYER_WORLD
	floor_max_angle = deg_to_rad(52.0)
	floor_snap_length = 0.4

	var cs := CollisionShape3D.new()
	var cap := CapsuleShape3D.new()
	cap.radius = 0.30
	cap.height = 1.72
	cs.shape = cap
	cs.position = Vector3(0, 0.86, 0)
	add_child(cs)

	cam = Camera3D.new()
	cam.position = Vector3(0, EYE_HEIGHT, 0)
	cam.fov = 70.0
	cam.near = 0.05
	cam.far = 1600.0
	add_child(cam)

	if look_enabled:
		Input.mouse_mode = Input.MOUSE_MODE_CAPTURED


func _unhandled_input(event: InputEvent) -> void:
	if not (look_enabled or scripted_look):
		return
	var looking := scripted_look or Input.mouse_mode == Input.MOUSE_MODE_CAPTURED
	if event is InputEventMouseMotion and looking:
		var mm := event as InputEventMouseMotion
		rotate_y(-mm.relative.x * MOUSE_SENS)
		_pitch = clampf(_pitch - mm.relative.y * MOUSE_SENS, -PITCH_LIMIT, PITCH_LIMIT)
		cam.rotation.x = _pitch
	elif event.is_action_pressed("ui_cancel"):
		Input.mouse_mode = (Input.MOUSE_MODE_VISIBLE
			if Input.mouse_mode == Input.MOUSE_MODE_CAPTURED
			else Input.MOUSE_MODE_CAPTURED)
	elif event is InputEventMouseButton and Input.mouse_mode == Input.MOUSE_MODE_VISIBLE:
		Input.mouse_mode = Input.MOUSE_MODE_CAPTURED


func _physics_process(delta: float) -> void:
	if not is_on_floor():
		velocity.y -= 22.0 * delta
	else:
		velocity.y = -0.2

	var wish := Vector3.ZERO
	var driving := look_enabled or scripted_look
	if driving:
		var ix := Input.get_axis("move_left", "move_right")
		var iz := Input.get_axis("move_forward", "move_back")
		wish = (transform.basis * Vector3(ix, 0, iz))
		if wish.length() > 1.0:
			wish = wish.normalized()

	var target_speed := JOG_SPEED if (driving and Input.is_action_pressed("jog")) else WALK_SPEED
	var target := wish * target_speed
	var flat := Vector3(velocity.x, 0, velocity.z)
	var rate := ACCEL if wish.length() > 0.01 else DECEL
	flat = flat.move_toward(target, rate * delta)
	velocity.x = flat.x
	velocity.z = flat.z

	move_and_slide()

	# Head bob, kept small on purpose. Enough to say "you are walking",
	# not enough to read as a camera effect.
	_speed_smooth = lerpf(_speed_smooth, Vector3(velocity.x, 0, velocity.z).length(), delta * 8.0)
	_bob += delta * _speed_smooth * 4.3
	var amp := clampf(_speed_smooth / JOG_SPEED, 0.0, 1.0) * 0.035
	cam.position.y = EYE_HEIGHT + sin(_bob) * amp
	cam.position.x = cos(_bob * 0.5) * amp * 0.6


## Used by the screenshot runner to place the camera deterministically.
func place(pos: Vector3, yaw_deg: float, pitch_deg: float) -> void:
	global_position = pos
	rotation = Vector3(0, deg_to_rad(yaw_deg), 0)
	_pitch = deg_to_rad(pitch_deg)
	cam.rotation.x = _pitch
	cam.position = Vector3(0, EYE_HEIGHT, 0)
	velocity = Vector3.ZERO
