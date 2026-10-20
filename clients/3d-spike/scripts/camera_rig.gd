## Three cameras observing one body.
##
## The architectural point of this file is what it does *not* contain: no
## movement, no gravity, no input-to-velocity, no state that a mode switch could
## reset. `Player` owns all of that, exactly once. This rig is a passive
## observer -- it reads the body's transform and the shared look pitch every
## frame and writes three camera transforms. Switching mode calls
## `make_current()` on one of three cameras that already exist and are already
## positioned; nothing is created, freed, reparented or re-initialised, so
## position, velocity, orientation and collision state cannot be disturbed by a
## switch. That is the property the drive test measures.
##
##   Mode 1  FIRST_PERSON  the eye. No visible body (deliberately -- see README).
##   Mode 2  THIRD_REAR    behind and above: body, direction of travel, street.
##   Mode 3  THIRD_FRONT   ahead looking back: the character and what is behind.
##
## Both third-person cameras orbit the same pivot along the *view* axis, so
## looking up swings the camera down and vice versa. That is Minecraft's
## behaviour and it is the one people already have in their hands.
class_name CameraRig
extends Node3D

enum Mode { FIRST_PERSON = 0, THIRD_REAR = 1, THIRD_FRONT = 2 }

const MODE_NAMES := ["first person", "third person rear", "third person front"]

const FOV := 70.0
const NEAR := 0.05
const FAR := 1600.0

## Eye height, first person. Unchanged from the original controller.
const EYE_HEIGHT := 1.66
## Orbit centre for both third-person cameras: roughly the shoulders.
const PIVOT_HEIGHT := 1.45

const REAR_DISTANCE := 3.40
const REAR_LIFT := 0.55
const FRONT_DISTANCE := 2.55
const FRONT_LIFT := 0.30

## Stop short of whatever the ray hit, so the near plane does not clip into it.
const COLLISION_MARGIN := 0.30
## Never pull closer than this to the pivot.
const MIN_DISTANCE := 0.45
## Pull in instantly, ease back out at this rate (m/s). Snapping outward pops.
const RETURN_SPEED := 7.0
## The boom is swept as a sphere of this radius, not a ray (what Godot's
## SpringArm3D does with a shape): a thin ray slips past a door jamb or a window
## mullion that the camera's near plane then cuts. Wider than the near plane's
## half-diagonal at NEAR and FOV.
const PROBE_RADIUS := 0.12
## A body that moved farther than this between two updates was put somewhere
## (a server placement, a correction, a probe), not walked: no walk or jog
## covers it in one physics tick. The boom then starts again from its full
## length at the new place, instead of easing out from wherever the old place
## had pulled it in -- which is how the first frame after the first placement
## started inside the character's head.
const JUMP_M := 1.0
## A boom shorter than this puts the camera inside the character's head and
## hair (the head's centre is ~0.25 m above the shoulder pivot, its radius with
## hair ~0.15 m, plus the near plane): the body is hidden while it lasts, the
## common third-person remedy for a camera pushed into its own character.
const HEAD_CLEAR_M := 0.75

## Look pitch in radians, shared by all three cameras -- one look direction,
## three viewpoints onto it.
var pitch := 0.0
## First-person head bob, written by the controller as (x, y) metres.
var bob := Vector2.ZERO

var mode: Mode = Mode.FIRST_PERSON

var first_person: Camera3D
var third_rear: Camera3D
var third_front: Camera3D

## Current (possibly pulled-in) boom length for the rear and front cameras.
var _rear_len := REAR_DISTANCE
var _front_len := FRONT_DISTANCE
## The pivot at the last update, to tell a placement from a walk (JUMP_M).
var _last_pivot := Vector3.INF
var _probe := SphereShape3D.new()


func _ready() -> void:
	first_person = _camera("FirstPersonCamera", false)
	third_rear = _camera("ThirdPersonBackCamera", true)
	third_front = _camera("ThirdPersonFrontCamera", true)
	apply_mode(mode)


func _camera(nm: String, world_space: bool) -> Camera3D:
	var c := Camera3D.new()
	c.name = nm
	c.fov = FOV
	c.near = NEAR
	c.far = FAR
	# The third-person cameras are positioned in world space: they trail the
	# body rather than being welded to it, so inheriting its transform would
	# only have to be undone.
	c.top_level = world_space
	add_child(c)
	return c


func active() -> Camera3D:
	match mode:
		Mode.THIRD_REAR:
			return third_rear
		Mode.THIRD_FRONT:
			return third_front
		_:
			return first_person


func mode_name() -> String:
	var nm: String = MODE_NAMES[int(mode)]
	return nm


## first person -> third rear -> third front -> first person. The rig owns the
## order; the controller owns what else a switch has to keep in step (the body's
## visibility, the HUD), so this only reports the answer.
func next_mode() -> Mode:
	return ((int(mode) + 1) % MODE_NAMES.size()) as Mode


## Forget the collision pull-in state. Used after a teleport, where easing the
## boom outward from wherever the old one ended up would be nonsense: INF makes
## the next update place each camera at its full length, or at whatever the
## collision ray allows there.
func snap() -> void:
	_rear_len = INF
	_front_len = INF


## Switch which of the three existing cameras the viewport draws from. This is
## the whole of a mode switch.
func apply_mode(m: Mode) -> void:
	mode = m
	active().make_current()


## Called by the controller from `_physics_process`, after `move_and_slide`:
## the body transform is then final for the tick, and a space query is legal.
## Reads the body, writes the cameras.
func update(delta: float) -> void:
	first_person.position = Vector3(bob.x, EYE_HEIGHT + bob.y, 0.0)
	first_person.rotation = Vector3(pitch, 0.0, 0.0)

	var pivot := global_position + Vector3.UP * PIVOT_HEIGHT
	if _last_pivot == Vector3.INF or pivot.distance_to(_last_pivot) > JUMP_M:
		snap()
	_last_pivot = pivot
	var view := view_direction()
	_rear_len = _place(third_rear, pivot, -view, REAR_DISTANCE, REAR_LIFT, view, _rear_len, delta)
	_front_len = _place(third_front, pivot, view, FRONT_DISTANCE, FRONT_LIFT, -view, _front_len, delta)


## Whether the active third-person camera sits so close to the pivot that it
## would draw from inside the character's head and shoulders: the boom pulled
## in below HEAD_CLEAR_M. First person is never "too close" -- it has no body.
func too_close() -> bool:
	match mode:
		Mode.THIRD_REAR:
			return _rear_len < HEAD_CLEAR_M
		Mode.THIRD_FRONT:
			return _front_len < HEAD_CLEAR_M
		_:
			return false


## Unit vector the player is looking along: body yaw combined with the shared
## pitch.
func view_direction() -> Vector3:
	var yaw_forward := -global_transform.basis.z
	return (yaw_forward * cos(pitch) + Vector3.UP * sin(pitch)).normalized()


## Put one third-person camera on a boom of length `want` pointing `away` from
## the pivot, pulled in to the first thing the boom hits, looking along `face`.
## Returns the boom length actually used, for the caller to carry forward.
func _place(cam: Camera3D, pivot: Vector3, away: Vector3, want: float, lift: float,
		face: Vector3, current: float, delta: float) -> float:
	# Fold the lift into the boom direction so one ray covers the real path.
	var to_ideal := away * want + Vector3.UP * lift
	var full := to_ideal.length()
	var dir := to_ideal / full

	var allowed := _clear_length(pivot, dir, full)
	# Pull in the instant something is in the way; ease back out.
	var used := allowed if allowed < current else minf(current + RETURN_SPEED * delta, allowed)

	cam.global_position = pivot + dir * used
	cam.look_at(cam.global_position + face, Vector3.UP)
	return used


## How far along `dir` the camera may sit before it would be inside the world:
## the engine's own swept-shape query (`cast_motion`, the query SpringArm3D
## runs), a PROBE_RADIUS sphere from the pivot along the boom.
func _clear_length(pivot: Vector3, dir: Vector3, full: float) -> float:
	var world := get_world_3d()
	if world == null:
		return full
	var space := world.direct_space_state
	if space == null:
		return full
	_probe.radius = PROBE_RADIUS
	var q := PhysicsShapeQueryParameters3D.new()
	q.shape = _probe
	q.transform = Transform3D(Basis(), pivot)
	q.motion = dir * full
	q.collision_mask = Build.LAYER_WORLD
	var safe := space.cast_motion(q)
	if safe.is_empty() or safe[0] >= 1.0:
		return full
	var d: float = full * float(safe[0]) - COLLISION_MARGIN
	return clampf(d, MIN_DISTANCE, full)
