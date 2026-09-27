## The walker. One movement implementation, three cameras observing it.
##
## Minecraft is the *control* reference and nothing else (ART_DIRECTION sec.9):
## WASD, mouse look, gravity, collision, and deliberately no jump -- this is a
## walking simulator. Speeds are real human speeds, not game speeds: 1.45 m/s
## stroll, 3.1 m/s jog. Eye height 1.66 m. Those three numbers do more for the
## "am I a person in a town" feeling than any amount of geometry.
##
## Everything that decides where the body is lives in this file and nowhere
## else. The three cameras (`camera_rig.gd`) only read it. Pressing the camera
## key changes which camera the viewport draws from -- it does not touch
## position, velocity, rotation, the collision shape or the floor state, and
## `--drive` measures that rather than trusting it.
class_name Player
extends CharacterBody3D

const WALK_SPEED := 1.45
const JOG_SPEED := 3.10
const ACCEL := 9.0
const DECEL := 12.0
const EYE_HEIGHT := CameraRig.EYE_HEIGHT
const MOUSE_SENS := 0.0016
const PITCH_LIMIT := deg_to_rad(84.0)

## Fixed, so the player character looks the same on every run.
const BODY_SEED := 90210
const BODY_HEIGHT := 1.75

signal camera_mode_changed(mode_name: String)

@export var look_enabled := true
## Set by the scripted runner: headless has no capturable mouse, so the
## capture check would otherwise swallow every synthetic motion event.
var scripted_look := false

## The three cameras. Not a movement system -- a passive observer.
var rig: CameraRig
## The visible character. Hidden in first person.
var body: NPC
## The first-person camera under its old name, for callers that want the eye.
var cam: Camera3D

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

	rig = CameraRig.new()
	rig.name = "CameraRig"
	add_child(rig)
	cam = rig.first_person

	_make_body()
	_sync_body()

	if look_enabled:
		Input.mouse_mode = Input.MOUSE_MODE_CAPTURED


## The visible player character. The NPC mannequin *is* the character model
## (decision ARC-4 -- real proportions, ordinary clothing, no face): same
## palette, same limbs, same code. Two of the three camera modes exist to look
## at a character, and reusing that mannequin was the alternative to writing a
## second one, which would have been more work and less coherent. Here it is
## driven by the controller's measured ground speed instead of by a path.
func _make_body() -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = BODY_SEED
	body = NPC.make(rng, NPC.Pose.PUPPET, BODY_HEIGHT)
	body.name = "Body"
	# The mannequin is authored facing its local +Z, which is the convention
	# npc.gd steers by (`rotation.y = atan2(facing.x, facing.z)`). The
	# controller uses Godot's own forward, -Z. Without this the character walks
	# backwards through the town, which the third-person shots show instantly.
	body.rotation.y = PI
	add_child(body)


## First person deliberately shows no body. A convincing first-person body is a
## separate problem -- mesh culling, animation clipping, the camera sitting
## inside the head -- and is explicitly out of scope for this spike.
func _sync_body() -> void:
	body.visible = rig.mode != CameraRig.Mode.FIRST_PERSON


## first person -> third rear -> third front -> first person.
func cycle_camera() -> CameraRig.Mode:
	set_camera(rig.next_mode())
	return rig.mode


## Everything a mode switch does. Note what is absent: no teleport, no velocity
## change, no re-spawn, no reset of anything the controller owns.
func set_camera(m: CameraRig.Mode) -> void:
	rig.apply_mode(m)
	_sync_body()
	camera_mode_changed.emit(rig.mode_name())


func _unhandled_input(event: InputEvent) -> void:
	if not (look_enabled or scripted_look):
		return
	var looking := scripted_look or Input.mouse_mode == Input.MOUSE_MODE_CAPTURED
	if event is InputEventMouseMotion and looking:
		var mm := event as InputEventMouseMotion
		rotate_y(-mm.relative.x * MOUSE_SENS)
		rig.pitch = clampf(rig.pitch - mm.relative.y * MOUSE_SENS, -PITCH_LIMIT, PITCH_LIMIT)
	elif event.is_action_pressed("camera_cycle"):
		cycle_camera()
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

	_speed_smooth = lerpf(_speed_smooth, Vector3(velocity.x, 0, velocity.z).length(), delta * 8.0)

	# Head bob, kept small on purpose. Enough to say "you are walking", not
	# enough to read as a camera effect. First person only: a bobbing
	# third-person camera reads as a fault rather than as footsteps.
	_bob += delta * _speed_smooth * 4.3
	var amp := clampf(_speed_smooth / JOG_SPEED, 0.0, 1.0) * 0.035
	rig.bob = Vector2(cos(_bob * 0.5) * amp * 0.6, sin(_bob) * amp)

	# The same mannequin the NPCs use, walking at the speed the body is
	# actually moving.
	body.step(delta, _speed_smooth)

	# Cameras last: the body transform is final for this tick, and the
	# third-person collision ray is a space query, which is legal here.
	rig.update(delta)


## Used by the screenshot runner to place the player deterministically.
func place(pos: Vector3, yaw_deg: float, pitch_deg: float) -> void:
	global_position = pos
	rotation = Vector3(0, deg_to_rad(yaw_deg), 0)
	rig.pitch = deg_to_rad(pitch_deg)
	rig.bob = Vector2.ZERO
	rig.snap()
	velocity = Vector3.ZERO
	_speed_smooth = 0.0
