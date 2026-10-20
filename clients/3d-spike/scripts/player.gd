## The walker. One movement implementation, three cameras observing it.
##
## Minecraft is the *control* reference and nothing else (ART_DIRECTION sec.9):
## WASD, mouse look, gravity, collision, and Space to jump -- added at the
## operator's request after playing it, at a human height rather than
## Minecraft's (see JUMP_HEIGHT). Speeds are real human speeds, not game speeds: 1.45 m/s
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
## A person's collision capsule: the server's default person, bodies'
## `PERSON_RADIUS` 250 mm (the operator's ruling of 2026-10-08: adult shoulder
## breadth 40-46 cm, chest depth 25-30 cm; Fruin (1971) pedestrian body ellipse
## 0.61 x 0.46 m) and `PERSON_HEIGHT` 1 720 mm (step-11 SD-B2), which no
## observation discloses yet because every person is that capsule (QP-6, QE-7).
## The player's own body and every perceived person's collider use it
## (step-15 §4.2, QS14-3; step-11 §22 SD-E2); `--world --bodies` B-r holds it
## to what the server does, by behaviour. Once per-person shapes are disclosed,
## the client reads them instead.
const CAPSULE_RADIUS := 0.25
const CAPSULE_HEIGHT := 1.72
## What the body collides with: the scene, the scene's walk aids (the slopes that
## make a drawn step a walk, `Build.LAYER_WALK_AID`, PR S36-P4b), the current
## place's disclosed walls and solids, perceived people and loose objects
## (step-11 §22 SD-E3). Every contact is prediction only; the server decides
## every stride.
const COLLIDES_WITH := Build.LAYER_WORLD | Build.LAYER_WALK_AID | Build.LAYER_DISCLOSED \
	| Build.LAYER_BODIES | Build.LAYER_OBJECTS
## Press-through (step-11 §22 SD-E5): the input points into a person or an
## object when its dot with the contact normal is below this.
const PRESS_DOT := -0.5
## How far ahead along the input the press is tested when the body stands
## still against what it presses into (a size of a probe, not a rule).
const PRESS_PROBE := 0.02
## What a press-through stride may pass: people and objects are the server's to
## resolve, so the cast that bounds the wished stride meets the scene (with its
## walk aids, which the body rides) and walls only.
const PRESS_CAST_MASK := Build.LAYER_WORLD | Build.LAYER_WALK_AID | Build.LAYER_DISCLOSED
const MOUSE_SENS := 0.0016
const PITCH_LIMIT := deg_to_rad(84.0)

## The controller's gravity. Stronger than 9.81 on purpose and unchanged from
## the accepted movement feel: a walking body that drops off a kerb at real g
## floats.
const GRAVITY := 22.0
## How high a jump lifts the feet, in metres. 0.45, not Minecraft's 1.25: a
## standing adult's jump clears roughly 0.4-0.5 m, and this is a person in a
## town, not a block-climber. The body leaves the ground at sqrt(2 g h), so
## at GRAVITY the jump lasts 2v/g = 0.40 s.
##
## It is rendered movement, local to the client (CLAUDE.md sec.4 rule 14) --
## not a server action. A connected client reports position, not jumps, and a
## jump changes height for 0.4 s and xy not at all, so it cannot carry the body
## past anything the server measures in the plane.
const JUMP_HEIGHT := 0.45
const JUMP_VELOCITY := sqrt(2.0 * GRAVITY * JUMP_HEIGHT)

## Fixed, so the player character looks the same on every run.
const BODY_SEED := 90210
const BODY_HEIGHT := 1.75

signal camera_mode_changed(mode_name: String)

@export var look_enabled := true
## Set by the scripted runner: headless has no capturable mouse, so the
## capture check would otherwise swallow every synthetic motion event.
var scripted_look := false
## Set by a scripted runner that turns the body itself and sends no mouse
## motion. Then every mouse motion is the real cursor crossing the capture
## window, and it would turn the view between the runner's turn and its
## capture: the slice's frames and its talk target varied from run to run with
## every person in the same place (2026-10-07).
var ignore_mouse_look := false
## The player's look settings (SET-b; `clients/shared/SETTINGS.md` §3). The defaults are today's look, so a
## client that never calls `set_look` (the promenade) looks exactly as before.
var look_scale := 1.0
var invert_look_y := false

## The two look settings, from the settings value (`look_scale()`, `invert_y`): called at start and on every
## Apply. Nothing else reads a setting here.
func set_look(scale: float, invert_y: bool) -> void:
	look_scale = scale
	invert_look_y = invert_y

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
	collision_mask = COLLIDES_WITH
	floor_max_angle = deg_to_rad(52.0)
	floor_snap_length = 0.4

	var cs := CollisionShape3D.new()
	var cap := CapsuleShape3D.new()
	cap.radius = CAPSULE_RADIUS
	cap.height = CAPSULE_HEIGHT
	cs.shape = cap
	cs.position = Vector3(0, CAPSULE_HEIGHT * 0.5, 0)
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
	body = NPC.make(rng, NPC.Pose.PUPPET, BODY_HEIGHT, true)
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
	body.visible = rig.mode != CameraRig.Mode.FIRST_PERSON and not rig.too_close()


## Whether the active third-person camera was last found too close to the head
## (`CameraRig.too_close`), as last synced to the body's visibility.
var _too_close := false


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
	if event is InputEventMouseMotion and looking and not ignore_mouse_look:
		look((event as InputEventMouseMotion).relative)
	elif event.is_action_pressed("camera_cycle"):
		cycle_camera()
	elif event.is_action_pressed("ui_cancel"):
		Input.mouse_mode = (Input.MOUSE_MODE_VISIBLE
			if Input.mouse_mode == Input.MOUSE_MODE_CAPTURED
			else Input.MOUSE_MODE_CAPTURED)
	elif event is InputEventMouseButton and Input.mouse_mode == Input.MOUSE_MODE_VISIBLE:
		Input.mouse_mode = Input.MOUSE_MODE_CAPTURED


## Turns the view by one mouse motion of `relative` counts, scaled by the look settings (SET-b). Mouse
## motion reaches this through `_unhandled_input`; a probe drives it directly, so OS cursor motion cannot
## add to a measurement.
func look(relative: Vector2) -> void:
	var sense := MOUSE_SENS * look_scale
	var pitch_sign := -1.0 if invert_look_y else 1.0
	rotate_y(-relative.x * sense)
	rig.pitch = clampf(rig.pitch - relative.y * sense * pitch_sign, -PITCH_LIMIT, PITCH_LIMIT)


## Jumps made since start, for the drive test's "no jump in mid-air" check.
var jumps := 0


func _physics_process(delta: float) -> void:
	var driving := look_enabled or scripted_look
	if not is_on_floor():
		velocity.y -= GRAVITY * delta
	else:
		velocity.y = -0.2
		# Only from the floor: is_on_floor() is false for the whole flight, so a
		# second press in the air does nothing -- there is no double jump.
		if driving and Input.is_action_just_pressed("jump"):
			velocity.y = JUMP_VELOCITY
			jumps += 1

	var wish := Vector3.ZERO
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
	_press_through(wish, target_speed * delta)
	_glide_step(delta)

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
	# A boom pulled in to the head (a wall or a window right behind) would draw
	# from inside the character: the body is hidden while that lasts. Synced on
	# the change only, so a script that shows the body on purpose keeps it.
	var close := rig.too_close()
	if close != _too_close:
		_too_close = close
		_sync_body()


## PRESS-THROUGH (step-11 §22 SD-E5). A body stopped locally at a person never
## asks the server to nudge them, so when this frame's slide met a person or an
## object and the input points into it, the travel the input wished for is
## counted, and the link reports the intended point (`intended_point`) instead
## of the body's. The server decides what that stride achieves -- a nudge, a
## push, a jam, a stop -- and the correction rule draws its answer.
##
## The direction pressed into, while pressing (Vector3.ZERO otherwise), and the
## travel wished for along it since the link last reported.
var press_dir := Vector3.ZERO
var pressed_travel := 0.0


func _press_through(wish: Vector3, step: float) -> void:
	var into := Vector3.ZERO
	var flat_wish := Vector3(wish.x, 0, wish.z)
	if flat_wish.length() > 0.01:
		var dir := flat_wish.normalized()
		# This frame's slides, and -- because a body held against a capsule
		# reports the contact only on the frames it moves into it -- a test of
		# the next step along the input, which meets whatever the input presses
		# into first (a wall first means it presses into a wall).
		var hits: Array[KinematicCollision3D] = []
		for i in range(get_slide_collision_count()):
			hits.append(get_slide_collision(i))
		var ahead := KinematicCollision3D.new()
		if test_move(global_transform, dir * maxf(step, PRESS_PROBE), ahead):
			hits.append(ahead)
		for c in hits:
			var o := c.get_collider() as CollisionObject3D
			if o == null or (o.collision_layer & (Build.LAYER_BODIES | Build.LAYER_OBJECTS)) == 0:
				continue
			var n := c.get_normal()
			n.y = 0.0
			if n.length() > 0.01 and dir.dot(n.normalized()) < PRESS_DOT:
				into = dir
				break
	if into == Vector3.ZERO:
		press_dir = Vector3.ZERO
		pressed_travel = 0.0
		return
	press_dir = into
	pressed_travel += step


## The point the input wished the body at: a capsule cast from the body along
## `press_dir` for `pressed_travel`, against walls only (`PRESS_CAST_MASK`).
func intended_point() -> Vector3:
	if press_dir == Vector3.ZERO or pressed_travel <= 0.0:
		return global_position
	var shape := CapsuleShape3D.new()
	shape.radius = CAPSULE_RADIUS
	shape.height = CAPSULE_HEIGHT
	var q := PhysicsShapeQueryParameters3D.new()
	q.shape = shape
	q.transform = Transform3D(Basis(), global_position + Vector3(0, CAPSULE_HEIGHT * 0.5 + 0.05, 0))
	q.motion = press_dir * pressed_travel
	q.collision_mask = PRESS_CAST_MASK
	var safe := get_world_3d().direct_space_state.cast_motion(q)
	var frac: float = safe[0] if safe.size() > 0 else 1.0
	return global_position + press_dir * pressed_travel * frac


## The link reported: the wished travel starts again.
func reported() -> void:
	pressed_travel = 0.0


## THE GLIDE (step-11 §22 SD-E8, ARC-50). A correction up to GLIDE_UP_TO is
## drawn over GLIDE_SECONDS of wall time; a larger one snaps. The body is moved
## kinematically, on top of whatever the input does this frame, toward a point
## the server already holds free -- input stays live, and nothing here asks the
## server anything. Frame `delta` is wall time: the world's time scale is never
## read (SD-E11, ARC-67).
const GLIDE_SECONDS := 0.12
const GLIDE_UP_TO := 1.0

var _glide_left := Vector3.ZERO
var _glide_time := 0.0


## Move the body to `target` (only x and z; the floor keeps its height): glided
## when the move is at most GLIDE_UP_TO, snapped otherwise. True if it snapped.
func correct_to(target: Vector3) -> bool:
	var off := Vector3(target.x - global_position.x, 0.0, target.z - global_position.z)
	velocity = Vector3(0.0, velocity.y, 0.0)
	if off.length() > GLIDE_UP_TO:
		global_position = Vector3(target.x, maxf(global_position.y, target.y), target.z)
		_glide_left = Vector3.ZERO
		_glide_time = 0.0
		return true
	_glide_left = off
	_glide_time = GLIDE_SECONDS
	return false


## Whether a correction is still being drawn.
func gliding() -> bool:
	return _glide_time > 0.0


func _glide_step(delta: float) -> void:
	if _glide_time <= 0.0:
		return
	var frac := minf(delta / _glide_time, 1.0)
	# moved like any motion of the body, so it stops at what is drawn in its way --
	# a pushed box still drawn on its way out, say -- instead of passing through it;
	# what it could not move this frame is carried to the next (step-11 §22.15
	# X12e-24: added on top of the input's own slide, the glide pushed the body
	# 6.6 mm into a box)
	_glide_left -= _glide_move(_glide_left * frac)
	_glide_time -= delta
	if _glide_time <= 0.0:
		_glide_move(_glide_left)
		_glide_left = Vector3.ZERO
		_glide_time = 0.0


## Moves the body by `step` unless something drawn is in the way; returns how far it moved.
func _glide_move(step: Vector3) -> Vector3:
	if step.length() < 0.00001:
		return Vector3.ZERO
	var hit := move_and_collide(step)
	return step if hit == null else step - hit.get_remainder()


## Used by the screenshot runner to place the player deterministically.
func place(pos: Vector3, yaw_deg: float, pitch_deg: float) -> void:
	global_position = pos
	rotation = Vector3(0, deg_to_rad(yaw_deg), 0)
	rig.pitch = deg_to_rad(pitch_deg)
	rig.bob = Vector2.ZERO
	rig.snap()
	velocity = Vector3.ZERO
	_speed_smooth = 0.0
