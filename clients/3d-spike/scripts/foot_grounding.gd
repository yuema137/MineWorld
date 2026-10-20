## Each planted foot on its own ground: rays, one `TwoBoneIK3D` per leg, the sole turned to the slope (PR S36-P4,
## `.structured-coding/plans/mvp1/pr-s36-p4-foot-grounding.md`; `DEP-144`).
##
## Why it exists. `GroundContact` keeps the lowest sole on the floor by moving the `Hips`. A hips offset is one
## number, so it cannot put two feet on two heights: Bob at the kerb with his left foot on the pavement (0.14 m up)
## and his right foot on the road has the left foot 140 mm inside the kerb, or the right one 140 mm in the air.
##
## What it does, after the animation, `Posture` and `GroundContact`, and only in how the body is drawn:
##
## 1. Per foot, rays straight down through the foot's lowest support point and its toe, heel and edge extremes
##    ([method ray_points]), against `Build.LAYER_WORLD` only (the drawn floors; never the server's disclosed
##    solids, which are not floors, nor other bodies). The highest hit is the ground under that foot. No hit: the
##    floor node's `y = 0`, today's behaviour.
## 2. `GroundContact` gets both grounds and settles the hips on the lower one (`GroundContact.lift_for`).
## 3. A planted foot is moved by its leg's `TwoBoneIK3D` so its lowest support point lands on the ground under it
##    (raised onto higher ground, or lowered by at most `max_planted_lower_m` when the clip leaves it floating, as at
##    a touchdown or a lift-off), and
##    the foot is turned by the rotation that takes the vertical to the ground's normal (at most
##    `max_foot_tilt_deg`). The asset is still judged without it: S36-P2's G-4w measures bodies with grounding off
##    (F-P4-2), so the IK never hides an asset defect from the gate.
## 4. A foot is planted when its contact point drifts by at most `planted_drift_share` of the body's speed (G-8's
##    rule, read as S36-P2 reads it); standing, both are. Its IK influence moves toward 1 (planted) or 0 at
##    `influence_per_s`.
## 5. `LimitAngularVelocityModifier3D` after the IK holds the leg bones under `max_leg_deg_per_s`. It is reset when
##    the gait changes, so that it holds the IK's corrections and never the animation's own switch of clip.
## 6. Beyond `ik_distance_m` from the active camera the IK and the rays stand down (`GroundContact` keeps working).
## 7. Two grounds more than `max_step_m` apart: the higher foot is not lifted, and the frame is `unreached`.
##
## Nothing here is per body: the bone names are the retargeted skeleton's, the support points are the body's own
## (measured by `GroundContact` at runtime), and every tunable is in `foot_grounding.json` with its reason. It
## sends nothing and writes no authoritative state (I-S36-6); the root's position is never changed (I-P4-1).
class_name FootGrounding
extends Node

const DATA := "res://assets/characters/foot_grounding.json"
const KEYS := ["ray_up_m", "ray_down_m", "max_step_m", "max_foot_tilt_deg", "planted_drift_share",
	"influence_per_s", "max_leg_deg_per_s", "ik_distance_m", "max_planted_lower_m"]
const SIDES := ["left", "right"]
## Below this body speed a person is standing, and both feet are planted.
const STANDING_MPS := 0.05
## The most the next frame's raise carries of what the IK left under (or over) the ground.
const BIAS_MAX_M := 0.05

static var _tunables: Dictionary = {}

## The body this grounds; its origin stands on the floor (the floor node of `GroundContact`).
var human: Node3D
var skeleton: Skeleton3D
var contact: GroundContact
## Off while seated (`Human.sit`).
var enabled := true
## The body's speed over the ground, m/s (`Human.set_gait`).
var body_speed := 0.0:
	set(v):
		if absf(v - body_speed) > 1e-4 and _limit != null:
			(_limit as LimitAngularVelocityModifier3D).reset()
		body_speed = v

## Per side: the ground under the foot in the floor node's frame (`y`, the hit point `p`, the normal `n`), whether a
## ray hit, whether the foot is planted, its IK influence, whether its ground is out of reach, and the last raise.
var feet := {}
var near := true
var _ik := {}
var _target := {}
var _pole := {}
var _tilt: FootTilt
var _limit: SkeletonModifier3D
var _bones := {}
var _pre_basis := {}
var _rot := {}


## The tunables, read once; an unknown or a missing key is refused by name (I-P4-2).
static func tunables() -> Dictionary:
	if not _tunables.is_empty():
		return _tunables
	var doc: Variant = JSON.parse_string(FileAccess.get_file_as_string(DATA))
	assert(typeof(doc) == TYPE_DICTIONARY and int(doc.get("format", 0)) == 1, "foot_grounding.json: format 1")
	var rows: Dictionary = doc["tunables"]
	var out := {}
	for key: String in rows:
		assert(key in KEYS, "foot_grounding.json: unknown key '%s'" % key)
		out[key] = float(rows[key]["value"])
	for key: String in KEYS:
		assert(out.has(key), "foot_grounding.json: missing key '%s'" % key)
	_tunables = out
	return out


## A refusal that works in a release build too (where `assert` is stripped): the message names the key.
static func check(doc: Variant) -> String:
	if typeof(doc) != TYPE_DICTIONARY or int((doc as Dictionary).get("format", 0)) != 1:
		return "foot_grounding.json: not format 1"
	var rows: Variant = (doc as Dictionary).get("tunables")
	if typeof(rows) != TYPE_DICTIONARY:
		return "foot_grounding.json: no tunables"
	for key: String in rows:
		if not key in KEYS:
			return "foot_grounding.json: unknown key '%s'" % key
	for key: String in KEYS:
		if not (rows as Dictionary).has(key):
			return "foot_grounding.json: missing key '%s'" % key
	return ""


## Ground `h` (its skeleton and its `GroundContact`). Adds the IK, tilt and limit modifiers after the contact.
static func for_body(h: Node3D, sk: Skeleton3D, gc: GroundContact) -> FootGrounding:
	var problem := check(JSON.parse_string(FileAccess.get_file_as_string(DATA)))
	if problem != "":
		push_error(problem)
		return null
	var fg := FootGrounding.new()
	fg.name = "FootGrounding"
	fg.human = h
	fg.skeleton = sk
	fg.contact = gc
	for side: String in SIDES:
		fg.feet[side] = {"hit": false, "y": 0.0, "p": Vector3.ZERO, "n": Vector3.UP, "planted": true,
			"influence": 0.0, "unreached": false, "raise": 0.0, "last": Vector3.INF}
		var pre := "Left" if side == "left" else "Right"
		fg._bones[side] = [sk.find_bone(pre + "UpperLeg"), sk.find_bone(pre + "LowerLeg"), sk.find_bone(pre + "Foot")]
	if fg._bones["left"].has(-1) or fg._bones["right"].has(-1):
		return null
	h.add_child(fg)
	fg._build_modifiers()
	gc.settled = fg._on_settled
	return fg


func _build_modifiers() -> void:
	for side: String in SIDES:
		var pre := "Left" if side == "left" else "Right"
		var target := Node3D.new()
		target.name = pre + "FootTarget"
		skeleton.add_child(target)
		var pole := Node3D.new()
		pole.name = pre + "KneePole"
		skeleton.add_child(pole)
		var ik := TwoBoneIK3D.new()
		ik.name = pre + "LegIK"
		skeleton.add_child(ik)
		ik.setting_count = 1
		ik.set_root_bone_name(0, pre + "UpperLeg")
		ik.set_middle_bone_name(0, pre + "LowerLeg")
		ik.set_end_bone_name(0, pre + "Foot")
		ik.set_target_node(0, NodePath("../" + target.name))
		ik.set_pole_node(0, NodePath("../" + pole.name))
		ik.influence = 0.0
		_ik[side] = ik
		_target[side] = target
		_pole[side] = pole
	_tilt = FootTilt.new()
	_tilt.name = "FootTilt"
	_tilt.grounding = self
	skeleton.add_child(_tilt)
	var limit := LimitAngularVelocityModifier3D.new()
	limit.name = "LegAngularLimit"
	skeleton.add_child(limit)
	_configure_limit(limit)
	_limit = limit


func _configure_limit(limit: LimitAngularVelocityModifier3D) -> void:
	limit.max_angular_velocity = deg_to_rad(float(tunables()["max_leg_deg_per_s"]))
	limit.chain_count = 2
	for i in 2:
		var pre := "Left" if i == 0 else "Right"
		limit.set_root_bone_name(i, pre + "UpperLeg")
		limit.set_end_bone_name(i, pre + "Foot")


## Switch grounding on or off with `GroundContact` (seated, the seat is the contact).
func set_enabled(on: bool) -> void:
	enabled = on
	for side: String in SIDES:
		(_ik[side] as SkeletonModifier3D).active = on
	_tilt.active = on
	_limit.active = on
	if not on:
		contact.ground_left = 0.0
		contact.ground_right = 0.0


## The support points a foot's rays go through: its lowest, and its extremes in eight horizontal directions (the
## toe, the heel, both edges and the corners), indices into `GroundContact.last_points[side]`. A foot rests on the highest thing
## under any of them, so a foot half over a kerb's edge stands on the kerb, as a real one would.
static func ray_points(pts: PackedVector3Array) -> PackedInt32Array:
	var out := PackedInt32Array()
	if pts.is_empty():
		return out
	# the lowest, then the extremes in eight horizontal directions (forward, back, both sides and the diagonals)
	var best := [0, 0, 0, 0, 0, 0, 0, 0, 0]
	var dirs: Array[Vector2] = []
	for k in 8:
		dirs.append(Vector2.RIGHT.rotated(k * PI / 4.0))
	for i in pts.size():
		var p := pts[i]
		if p.y < pts[best[0]].y:
			best[0] = i
		for k in 8:
			var q := pts[best[k + 1]]
			if Vector2(p.x, p.z).dot(dirs[k]) > Vector2(q.x, q.z).dot(dirs[k]):
				best[k + 1] = i
	for i: int in best:
		if not out.has(i):
			out.append(i)
	return out


func _physics_process(_delta: float) -> void:
	if not enabled or contact == null or not contact.active:
		return
	var t := tunables()
	var cam := get_viewport().get_camera_3d() if is_inside_tree() else null
	near = cam == null or cam.global_position.distance_to(human.global_position) <= t["ik_distance_m"]
	var space := human.get_world_3d().direct_space_state
	var to_world := human.global_transform
	var to_floor := to_world.affine_inverse()
	var up := (to_world.basis * Vector3.UP).normalized()
	for side: String in SIDES:
		var f: Dictionary = feet[side]
		f["hit"] = false
		f["y"] = 0.0
		f["n"] = Vector3.UP
		var hits := {}
		var pts: PackedVector3Array = contact.last_points[side]
		var drawn: PackedVector3Array = contact.last_points_world[side]
		if near and drawn.size() == pts.size():
			var n_sum := Vector3.ZERO
			var top := -INF
			for i in ray_points(pts):
				# where the point was drawn, not where the body's frame has carried it since (it walks on)
				var at := drawn[i]
				var q := PhysicsRayQueryParameters3D.create(at + up * t["ray_up_m"], at - up * t["ray_down_m"],
					Build.LAYER_WORLD)
				var hit := space.intersect_ray(q)
				if hit.is_empty():
					continue
				var p := to_floor * (hit["position"] as Vector3)
				var n := (to_floor.basis * (hit["normal"] as Vector3)).normalized()
				# a ray that grazes a step's edge reports the riser's normal: not ground a sole can rest flat on
				if Vector3.UP.angle_to(n) > deg_to_rad(t["max_foot_tilt_deg"]):
					continue
				hits[i] = [hit["position"], hit["normal"]]
				n_sum += n
				top = maxf(top, p.y)
			if not hits.is_empty():
				f["hit"] = true
				f["y"] = top
				f["n"] = n_sum.normalized()
		f["hits"] = hits
	var gl: float = feet["left"]["y"]
	var gr: float = feet["right"]["y"]
	feet["left"]["unreached"] = gl - gr > t["max_step_m"]
	feet["right"]["unreached"] = gr - gl > t["max_step_m"]
	contact.ground_left = gl
	contact.ground_right = gr


## Planted and influence, once per drawn frame (the animation advances once per frame, not once per physics tick,
## so a foot's speed is measured between frames). G-8's rule as S36-P2 reads it: the foot's **contact point** (its
## lowest support point, the same vertex compared across frames) drifts by at most `planted_drift_share` of the
## body's speed; standing, both feet are planted. (The `Foot` bone is not the contact: in stance the ankle rolls
## over the foot at 0.6-0.7 of the gait speed.)
func _update_planted(delta: float) -> void:
	var t := tunables()
	var to_world := human.global_transform
	for side: String in SIDES:
		var f: Dictionary = feet[side]
		var pts: PackedVector3Array = contact.last_points[side]
		var speed := 0.0
		var prev: PackedVector3Array = f.get("prev", PackedVector3Array())
		var world := PackedVector3Array()
		var low := 0
		for i in pts.size():
			world.append(to_world * pts[i])
			if pts[i].y < pts[low].y:
				low = i
		if prev.size() == world.size() and not world.is_empty() and delta > 0.0:
			speed = (world[low] - prev[low]).length() / delta
		f["prev"] = world
		f["planted"] = body_speed < STANDING_MPS or speed <= t["planted_drift_share"] * body_speed
		var want := 1.0 if near and f["planted"] and not f["unreached"] else 0.0
		f["influence"] = move_toward(f["influence"], want, t["influence_per_s"] * delta)
		(_ik[side] as SkeletonModifier3D).influence = f["influence"]


## Called by `GroundContact` once per modification, after the hips' lift: aims each leg's IK at its foot's own
## ground, before the IK nodes (which follow `GroundContact` in the skeleton's modifier order) run.
func _on_settled() -> void:
	if not enabled:
		return
	_update_planted(get_process_delta_time())
	var t := tunables()
	var to_world := human.global_transform
	var up_w := (to_world.basis * Vector3.UP).normalized()
	var fwd_w := (to_world.basis * Vector3.BACK).normalized()
	var max_tilt := deg_to_rad(t["max_foot_tilt_deg"])
	var sk_world := skeleton.global_transform
	for side: String in SIDES:
		var f: Dictionary = feet[side]
		var b: Array = _bones[side]
		# bone poses are in the skeleton's space, which sits below the floor node by the body's sole depth
		var hip := sk_world * skeleton.get_bone_global_pose(b[0]).origin
		var knee := sk_world * skeleton.get_bone_global_pose(b[1]).origin
		var foot_pose := skeleton.get_bone_global_pose(b[2])
		var ankle := sk_world * foot_pose.origin
		_pre_basis[side] = foot_pose.basis
		# the turn that takes the vertical to the ground's normal, capped; the foot keeps its animated roll on top
		var n: Vector3 = f["n"]
		var axis := Vector3.UP.cross(n)
		var angle := minf(Vector3.UP.angle_to(n), max_tilt)
		var rot := Basis() if axis.length() < 1e-6 else Basis(axis.normalized(), angle)
		_rot[side] = rot
		# the raise: the most any support point, turned with the foot, is under the ground near it (negative: the foot
		# floats, and is lowered onto the ground). A point's ground is the highest of the hit planes within half the
		# foot's own width of it (the nearest hit's when none is that close): a foot over a kerb's edge rests on the
		# kerb, as a rigid sole does, and on smooth uneven ground each point meets the surface beside it.
		var raise := 0.0
		var hits: Dictionary = f.get("hits", {})
		var pts: PackedVector3Array = contact.last_points[side]
		if f["hit"] and not f["unreached"]:
			var inv := to_world.affine_inverse()
			var a := inv * ankle
			var planes: Array = []
			for i: int in hits:
				planes.append([inv * (hits[i][0] as Vector3), (inv.basis * (hits[i][1] as Vector3)).normalized()])
			var reach := _half_width(pts)
			f["planes"] = planes
			f["reach"] = reach
			var need := _need(pts, rot, a, planes, reach)
			# what the IK left under the ground last frame (the shin's skin moves with the knee as it bends)
			need += float(f.get("bias", 0.0))
			raise = need if need > -INF else 0.0
			# a foot that is leaving the ground (no longer planted) is never pulled back down while its influence
			# ramps out; it is only kept out of the ground
			if not f["planted"]:
				raise = maxf(raise, 0.0)
			# a foot higher than a swing foot's lowest clearance is swinging, whatever its drift says: never lowered more
			raise = maxf(raise, -float(t["max_planted_lower_m"]))
		if not f["hit"] or f["unreached"] or f["influence"] <= 0.0:
			f["bias"] = 0.0
		f["raise"] = raise
		(_target[side] as Node3D).global_position = ankle + up_w * raise
		# the pole: in front of the knee in the plane the animation bends it in, so a zero raise is the animated pose
		var line := ankle - hip
		var along := hip + line * clampf((knee - hip).dot(line) / maxf(line.length_squared(), 1e-6), 0.0, 1.0)
		var bend := knee - along
		var dir := bend.normalized() if bend.length() > 0.01 else fwd_w
		(_pole[side] as Node3D).global_position = knee + dir * 0.5


## How far `pts` (turned by `rot` about `a`) must rise so that none is under the ground near it (negative: they float).
## A point's ground is the highest of the hit planes within `reach` of it, or the nearest hit's when none is.
static func _need(pts: PackedVector3Array, rot: Basis, a: Vector3, planes: Array, reach: float) -> float:
	var need := -INF
	for p: Vector3 in pts:
		var tilted := a + rot * (p - a)
		var here := -INF
		var nearest := INF
		var near_need := -INF
		for pl: Array in planes:
			var hp: Vector3 = pl[0]
			var hn: Vector3 = pl[1]
			var g := -(tilted - hp).dot(hn) / maxf(hn.y, 0.2)
			var dist := Vector2(tilted.x - hp.x, tilted.z - hp.z).length()
			if dist <= reach:
				here = maxf(here, g)
			if dist < nearest:
				nearest = dist
				near_need = g
		need = maxf(need, here if here > -INF else near_need)
	return need


## Half the foot's width: half its support points' extent across the body (its local x), measured on this body.
static func _half_width(pts: PackedVector3Array) -> float:
	var lo := INF
	var hi := -INF
	for p in pts:
		lo = minf(lo, p.x)
		hi = maxf(hi, p.x)
	return maxf((hi - lo) * 0.5, 0.0) if hi > lo else 0.0


## After the IK: each foot's global orientation is its animated one turned to the ground, by its influence. The
## IK's own rotation of the foot (it turns with the shin) is replaced, so a zero raise on flat ground changes nothing.
func _apply_tilt() -> void:
	if not enabled:
		return
	var to_floor_basis := (human.global_transform.affine_inverse() * skeleton.global_transform).basis
	for side: String in SIDES:
		var f: Dictionary = feet[side]
		var w: float = f["influence"]
		if w <= 0.0 or not _pre_basis.has(side):
			continue
		var i: int = _bones[side][2]
		var pose := skeleton.get_bone_global_pose(i)
		# the turn is in the floor node's frame; the foot's basis is in the skeleton's
		var rot_sk: Basis = to_floor_basis.inverse() * (_rot[side] as Basis) * to_floor_basis
		var want: Basis = rot_sk * (_pre_basis[side] as Basis)
		var q := pose.basis.get_rotation_quaternion().slerp(want.get_rotation_quaternion(), w)
		pose.basis = Basis(q).scaled(pose.basis.get_scale())
		skeleton.set_bone_global_pose(i, pose)
	# the feet as drawn now: what is still under the ground is carried into the next frame's raise, so a standing
	# body's planted foot converges onto its ground within a frame or two
	var drawn := contact.points_by_side()
	for side: String in SIDES:
		var f: Dictionary = feet[side]
		if float(f["influence"]) <= 0.0 or not f.has("planes") or not f["hit"] or f["unreached"]:
			continue
		var left_under := _need(drawn[side], Basis(), Vector3.ZERO, f["planes"], f["reach"])
		# only for a body standing still: in motion last frame's residual belongs to another pose
		if body_speed >= STANDING_MPS:
			f["bias"] = 0.0
		elif left_under > -INF and (f["planted"] or left_under > 0.0):
			f["bias"] = clampf(float(f.get("bias", 0.0)) + left_under, -BIAS_MAX_M, BIAS_MAX_M)


## The modifier that turns the feet after the IK (see [method FootGrounding._apply_tilt]).
class FootTilt:
	extends SkeletonModifier3D
	var grounding: FootGrounding

	func _process_modification() -> void:
		if grounding != null:
			grounding._apply_tilt()
