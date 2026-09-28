class_name MineWorldSpace
extends RefCounted

## The one conversion between MineWorld's axes and Godot's, in both directions and for both
## dimensions.
##
## The frame is fixed by `docs/CORE_CONCEPTS.md` §6.1, and it is stated there because two clients
## cannot guess it alike:
## [codeblock]
## +x  east        +y  north        +z  up        right-handed
## yaw  measured from +y toward +x, so 0 mdeg faces north and 90 000 mdeg faces east
## pitch  positive looks up
## [/codeblock]
##
## The renderer spike wrote this conversion twice, once per client, each with a sign flip nothing in
## the contract could have told the author about (`spike/FINDINGS.md` F5). The symptom of getting it
## wrong is NPCs facing backwards, and the fix is the kind that gets applied by negating something
## until it looks right. So it lives here, once, and both clients import it.
##
## Units: the world speaks integer millimetres and millidegrees, because those values reach the event
## log and floating-point arithmetic is not reproducible across platforms (`AC-12`). Godot speaks
## float metres and radians. Everything returned to the world is rounded and cast; everything
## returned to the engine is a float.

## Millimetres in a metre. The only scale factor in this file.
const MM_PER_METRE := 1000.0

## Millidegrees in a degree.
const MDEG_PER_DEGREE := 1000.0

## One full turn in millidegrees: the modulus a yaw is canonicalized against, as `Orientation` does.
const FULL_TURN_MDEG := 360_000


## A world position as a 2D top-down point in metres.
##
## `+y` is north and Godot's 2D `+y` is down the screen, so north is up: the sign flip is here and
## nowhere else. The result is in metres; how many pixels a metre is worth is the client's own art
## decision and none of this file's business.
static func to_2d(local: Variant) -> Vector2:
	if typeof(local) != TYPE_DICTIONARY:
		return Vector2.ZERO
	return Vector2(
		float(local.get("x", 0)) / MM_PER_METRE,
		-float(local.get("y", 0)) / MM_PER_METRE,
	)


## A world position as a 3D point in metres.
##
## `+x` east becomes Godot `+X`, `+z` up becomes Godot `+Y`, and `+y` north becomes Godot `-Z`,
## which is Godot's forward. Both frames are right-handed, so this is a rotation and nothing is
## mirrored.
static func to_3d(local: Variant) -> Vector3:
	if typeof(local) != TYPE_DICTIONARY:
		return Vector3.ZERO
	return Vector3(
		float(local.get("x", 0)) / MM_PER_METRE,
		float(local.get("z", 0)) / MM_PER_METRE,
		-float(local.get("y", 0)) / MM_PER_METRE,
	)


## A world heading as a 2D rotation in radians.
##
## Yaw 0 is north, which is up the screen, which is `-90°` in Godot 2D — where angle 0 points `+x`
## and angles increase clockwise because `+y` is down.
static func yaw_to_2d_radians(facing: Variant) -> float:
	return deg_to_rad(_yaw_degrees(facing) - 90.0)


## A world heading as a 3D `rotation.y` in radians.
##
## Yaw 0 is north is Godot `-Z` is `rotation.y == 0`; yaw increases toward east, which is Godot's
## `+X`, which is a *negative* rotation about `+Y`. Hence the single minus sign — the one the spike
## had to discover by turning a character round and looking at it.
static func yaw_to_3d_radians(facing: Variant) -> float:
	return -deg_to_rad(_yaw_degrees(facing))


## A pitch in radians, or 0.0 when the world does not model one.
static func pitch_to_radians(facing: Variant) -> float:
	if typeof(facing) != TYPE_DICTIONARY or facing.get("pitch") == null:
		return 0.0
	return deg_to_rad(float(facing["pitch"]) / MDEG_PER_DEGREE)


## A 2D point in metres as a world position: integer millimetres, `+y` north.
static func from_2d(position_metres: Vector2, height_metres: float = 0.0) -> Dictionary:
	return {
		"x": millimetres(position_metres.x),
		"y": millimetres(-position_metres.y),
		"z": millimetres(height_metres),
	}


## A 3D point in metres as a world position: integer millimetres, on the world's axes.
static func from_3d(position_metres: Vector3) -> Dictionary:
	return {
		"x": millimetres(position_metres.x),
		"y": millimetres(-position_metres.z),
		"z": millimetres(position_metres.y),
	}


## A 2D rotation in radians as a world yaw in canonical millidegrees.
static func yaw_from_2d_radians(radians: float) -> int:
	return millidegrees(rad_to_deg(radians) + 90.0)


## A 3D `rotation.y` in radians as a world yaw in canonical millidegrees.
static func yaw_from_3d_radians(radians: float) -> int:
	return millidegrees(-rad_to_deg(radians))


## A `Location` as the protocol carries one: a place, and optionally where inside it.
##
## `place` is the place's own identity **as a string**, never parsed as a number (see
## `MineWorldClient`). `local` and `facing` are omitted as `null` when a client does not model them,
## which is exactly what a 2D client does and what the contract intends.
static func location(place: String, local: Variant = null, facing_mdeg: Variant = null) -> Dictionary:
	var facing: Variant = null
	if facing_mdeg != null:
		facing = { "yaw": int(facing_mdeg), "pitch": null }
	return {
		"place": { "entity": place, "entity_type": "place" },
		"local": local,
		"facing": facing,
	}


## Metres as integer millimetres.
##
## Rounded and cast, because GDScript has one number type and `serde` refuses a float where an `i32`
## is declared — `1500.0` comes back as *invalid type: floating point 1500.0, expected i32*
## (`spike/FINDINGS.md` F9). That refusal is the contract working; this function is how a client
## stops earning it.
static func millimetres(metres: float) -> int:
	return int(round(metres * MM_PER_METRE))


## Degrees as canonical integer millidegrees, in `[0, 360000)` as `Orientation` stores them.
static func millidegrees(degrees: float) -> int:
	return posmod(int(round(degrees * MDEG_PER_DEGREE)), FULL_TURN_MDEG)


static func _yaw_degrees(facing: Variant) -> float:
	if typeof(facing) != TYPE_DICTIONARY:
		return 0.0
	return float(facing.get("yaw", 0)) / MDEG_PER_DEGREE
