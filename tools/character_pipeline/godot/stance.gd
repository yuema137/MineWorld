# SPDX-License-Identifier: MIT
# The relaxed standing arms (PR S26-c-ground, bug 2; PIPELINE.md §15.2), solved per body from its own mesh.
#
# The `Stand` clip (`human.gd` `stand_key`) was authored on the town body: on the broader generated bodies its
# hanging arms press the forearms and hands into the hips (the weight-bearing right hip most), and several bodies'
# hands hang palm forward, because a generated body's finger joints need not lie as its mesh's hand does. This
# solves, per body, the arms of a relaxed stand, rest-relative (`stand_key`'s convention):
#
# - the upper arm's straight-down X found by measurement (a rest may be a T or an A), then a natural hang 6 degrees
#   out from it;
# - the forearm twisted about its own axis, after a slight elbow bend (`ELBOW_DEG`), so the hand faces the thigh as
#   its mesh shows it (`palm_error`: thin across the body, the thumb ahead of the palm);
# - a light finger curl (`CURL_DEG`) about whichever finger-bone axis closes the mesh's fingertips toward the thigh;
# - the upper arm then abducted (at most `MAX_ABDUCTION_DEG`) and swung forward (at most 12 degrees) by the least
#   that makes the lower half of the forearm and the hand clear the torso, the hips and the thighs by `CLEAR_MM`;
#   a side that cannot within those limits (its hip, pushed out by the weight shift, is the widest point) takes the
#   wider `STAGES[1]` search (more abduction, forward swing and elbow bend), and one that cannot even then keeps its
#   best and is reported short. `solve(base, sides)` solves only the sides given (the protagonist's free arm).
#
# Clearance: in 2 cm horizontal slabs, the body's outline (vertices dominated by the hips, the spine, the chest or
# the thighs) as its outermost radius from the slab's centroid in 48 directions; an arm vertex (one the forearm, the
# hand or a finger carries almost alone, lying on its bone segment) is `distance from the centroid - outline
# radius` clear of it. Negative is inside. The upper half of the forearm is left out: at the elbow a relaxed arm may
# rest against a broad flank.
#
# Everything is evaluated analytically from the skeleton's current local poses with the solved bones replaced
# (Σ w · pose · bind on the CPU), so the solve needs no frames and is deterministic.
extends RefCounted

const CLEAR_MM := 30.0
const ELBOW_DEG := 14.0
const CURL_DEG := {"Proximal": 16.0, "Intermediate": 22.0, "Distal": 12.0}
const FINGERS := ["Index", "Middle", "Ring", "Little"]
const BODY := ["Hips", "Spine", "Chest", "UpperChest", "LeftUpperLeg", "RightUpperLeg"]
## The share of the forearm, from the elbow, left out of the clearance (see the header).
const FOREARM_FROM := 0.5
## An arm vertex further than this from its bone segment (elbow to wrist; wrist to fingertips, `HAND_M` long) is
## not on the limb, and one whose weights are more than `MIX_W` off the arm is seam (pose_sheet.gd's G-9 rule).
const LIMB_R := 0.08
const HAND_M := 0.20
const MIX_W := 0.05
const SLAB_M := 0.02
const BINS := 48
## A relaxed arm hangs a little out from straight down; the search starts there and abducts no further than the max.
const HANG_ABDUCTION_DEG := 6.0
const MAX_ABDUCTION_DEG := 15.0
## The search limits, relaxed first and wider second: [abduction deg, forward swing deg, elbow bends deg].
const STAGES := [[MAX_ABDUCTION_DEG, 12, [ELBOW_DEG]], [24.0, 21, [ELBOW_DEG, 20.0, 26.0]]]

var sk: Skeleton3D
var root: Node3D
var _verts: Array = []  # per skinned surface: [v, bi, bw, per, bones, binds, dom, hand tag]


func _init(skeleton: Skeleton3D, floor_node: Node3D) -> void:
	sk = skeleton
	root = floor_node
	for mi: MeshInstance3D in sk.find_children("*", "MeshInstance3D", true, false):
		if not mi.visible or mi.skin == null:
			continue
		var skin := mi.skin
		var bones := PackedInt32Array()
		var binds: Array[Transform3D] = []
		for b in skin.get_bind_count():
			var bone := skin.get_bind_bone(b)
			bones.append(bone if bone >= 0 else sk.find_bone(skin.get_bind_name(b)))
			binds.append(skin.get_bind_pose(b))
		for s in mi.mesh.get_surface_count():
			var arr := mi.mesh.surface_get_arrays(s)
			var v: PackedVector3Array = arr[Mesh.ARRAY_VERTEX]
			var bi: PackedInt32Array = arr[Mesh.ARRAY_BONES]
			var bw: PackedFloat32Array = arr[Mesh.ARRAY_WEIGHTS]
			var per := bi.size() / maxi(v.size(), 1)
			var dom := PackedStringArray()
			var tag := PackedStringArray()
			for i in v.size():
				var best := -1.0
				var bb := 0
				for k in per:
					if bw[i * per + k] > best:
						best = bw[i * per + k]
						bb = bones[bi[i * per + k]]
				var grp := _group(sk.get_bone_name(bb))
				if grp.ends_with("hand") or grp.ends_with("forearm"):
					# a fused generated mesh bleeds arm weights into the flank and the hem: only a vertex the arm
					# carries almost alone is arm surface (the G-9 check's rule, pose_sheet.gd MIX_W)
					var arm_w := 0.0
					for k in per:
						if _group(sk.get_bone_name(bones[bi[i * per + k]])).length() > 1 \
								and _group(sk.get_bone_name(bones[bi[i * per + k]])) != "body":
							arm_w += bw[i * per + k]
					if arm_w < 1.0 - MIX_W:
						grp = ""
				dom.append(grp)
				tag.append(_hand_tag(sk.get_bone_name(bb)) if grp.ends_with("hand") else "")
			_verts.append([v, bi, bw, per, bones, binds, dom, tag])


static func _group(bone: String) -> String:
	if bone in BODY:
		return "body"
	var side := "L" if bone.begins_with("Left") else "R"
	for f in ["Thumb", "Index", "Middle", "Ring", "Little"]:
		if f in bone:
			return side + "hand"
	if bone.ends_with("Hand"):
		return side + "hand"
	if bone.ends_with("LowerArm"):
		return side + "forearm"
	return ""


## Skeleton-space global transforms of every bone, with `over` (bone index -> local rotation) replacing some.
func _globals(over: Dictionary) -> Array[Transform3D]:
	var g: Array[Transform3D] = []
	g.resize(sk.get_bone_count())
	var done := PackedByteArray()
	done.resize(sk.get_bone_count())
	for i in sk.get_bone_count():
		_global(i, over, g, done)
	return g


func _global(i: int, over: Dictionary, g: Array[Transform3D], done: PackedByteArray) -> Transform3D:
	if done[i] == 1:
		return g[i]
	var q: Quaternion = over[i] if over.has(i) else sk.get_bone_pose_rotation(i)
	var t := Transform3D(Basis(q), sk.get_bone_pose_position(i))
	var p := sk.get_bone_parent(i)
	g[i] = t if p < 0 else _global(p, over, g, done) * t
	done[i] = 1
	return g[i]


func _rr(bone: String, e: Vector3) -> Quaternion:
	return sk.get_bone_rest(sk.find_bone(bone)).basis.get_rotation_quaternion() * Human.euler_q(e)


## The overrides (bone index -> local rotation) of a set of rest-relative eulers (bone name -> degrees).
func overrides(eulers: Dictionary) -> Dictionary:
	var o := {}
	for b: String in eulers:
		var i := sk.find_bone(b)
		if i >= 0:
			o[i] = _rr(b, eulers[b])
	return o


func _world(g: Array[Transform3D], bone: String) -> Vector3:
	return root.global_transform.affine_inverse() * sk.global_transform * g[sk.find_bone(bone)].origin


## The arms' clearance from the torso, the hips and the thighs, in mm, per side: [left, right].
func clearance(eulers: Dictionary) -> Array:
	var g := _globals(overrides(eulers))
	var to_root := root.global_transform.affine_inverse() * sk.global_transform
	var body := {}
	var arm: Array = []
	for e: Array in _verts:
		var v: PackedVector3Array = e[0]
		var bi: PackedInt32Array = e[1]
		var bw: PackedFloat32Array = e[2]
		var per: int = e[3]
		var bones: PackedInt32Array = e[4]
		var binds: Array = e[5]
		var dom: PackedStringArray = e[6]
		var mats: Array[Transform3D] = []
		for b in bones.size():
			mats.append(to_root * g[bones[b]] * (binds[b] as Transform3D))
		for i in v.size():
			if dom[i] == "":
				continue
			var p := Vector3.ZERO
			for k in per:
				var w := bw[i * per + k]
				if w > 0.0:
					p += (mats[bi[i * per + k]] * v[i]) * w
			if dom[i] == "body":
				var s := int(floor(p.y / SLAB_M))
				var slab: PackedVector2Array = body.get(s, PackedVector2Array())
				slab.append(Vector2(p.x, p.z))
				body[s] = slab
			else:
				arm.append([p, dom[i]])
	var hulls := {}
	for s: int in body:
		var pts: PackedVector2Array = body[s]
		if pts.size() < 8:
			continue
		var c := Vector2.ZERO
		for q in pts:
			c += q
		c /= pts.size()
		var r := PackedFloat32Array()
		r.resize(BINS)
		for q in pts:
			var bin := _bin(q - c)
			r[bin] = maxf(r[bin], (q - c).length())
		hulls[s] = [c, r]
	var out := [INF, INF]
	# the forearm counts from FOREARM_FROM of the way down from the elbow: at the elbow a relaxed arm may rest
	# against a broad flank; the forearm's lower part and the hand must not
	var seg := {}
	for sd in ["L", "R"]:
		var nm := "Left" if sd == "L" else "Right"
		seg[sd] = [_world(g, nm + "LowerArm"), _world(g, nm + "Hand")]
	for a: Array in arm:
		var p: Vector3 = a[0]
		var grp: String = a[1]
		var ends: Array = seg[grp.left(1)]
		if grp.ends_with("forearm"):
			var e0: Vector3 = ends[0]
			var e1: Vector3 = ends[1]
			if (p - e0).dot(e1 - e0) / maxf((e1 - e0).length_squared(), 1e-9) < FOREARM_FROM:
				continue
			if p.distance_to(Geometry3D.get_closest_point_to_segment(p, e0, e1)) > LIMB_R:
				continue
		else:
			var w0: Vector3 = ends[1]
			var w1: Vector3 = w0 + (w0 - (ends[0] as Vector3)).normalized() * HAND_M
			if p.distance_to(Geometry3D.get_closest_point_to_segment(p, w0, w1)) > LIMB_R:
				continue
		var s := int(floor(p.y / SLAB_M))
		if not hulls.has(s):
			continue
		var c: Vector2 = hulls[s][0]
		var r: PackedFloat32Array = hulls[s][1]
		var q := Vector2(p.x, p.z) - c
		var bin := _bin(q)
		var edge := r[bin]
		var side := 0 if grp.begins_with("L") else 1
		out[side] = minf(out[side], (q.length() - edge) * 1000.0)
	return out


static func _bin(q: Vector2) -> int:
	return int(floor((q.angle() + PI) / TAU * BINS)) % BINS


## A hand vertex's part, for the hand's orientation as the mesh shows it: `<side>palm` (the hand bone),
## `<side>thumb`, `<side>tip` (a finger's distal bone).
static func _hand_tag(bone: String) -> String:
	var side := "Left" if bone.begins_with("Left") else "Right"
	if "Thumb" in bone:
		return side + "thumb"
	if bone.ends_with("Distal"):
		return side + "tip"
	if bone.ends_with("Hand"):
		return side + "palm"
	return ""


## The centroids of the hands' parts (`_hand_tag`), skinned under `eulers`, in the floor node's frame.
func hand_centroids(eulers: Dictionary) -> Dictionary:
	var g := _globals(overrides(eulers))
	var to_root := root.global_transform.affine_inverse() * sk.global_transform
	var sum := {}
	var n := {}
	for e: Array in _verts:
		var v: PackedVector3Array = e[0]
		var bi: PackedInt32Array = e[1]
		var bw: PackedFloat32Array = e[2]
		var per: int = e[3]
		var bones: PackedInt32Array = e[4]
		var binds: Array = e[5]
		var tag: PackedStringArray = e[7]
		for i in v.size():
			if tag[i] == "":
				continue
			var p := Vector3.ZERO
			for k in per:
				var w := bw[i * per + k]
				if w > 0.0:
					var b := bi[i * per + k]
					p += (to_root * g[bones[b]] * (binds[b] as Transform3D) * v[i]) * w
			sum[tag[i]] = (sum.get(tag[i], Vector3.ZERO) as Vector3) + p
			n[tag[i]] = int(n.get(tag[i], 0)) + 1
			if not tag[i].ends_with("thumb"):
				# the flat of the hand (palm and fingertips): its spread across (x) and along (z) the body
				var fl := tag[i].trim_suffix("palm").trim_suffix("tip") + "flat"
				sum[fl] = (sum.get(fl, Vector3.ZERO) as Vector3) + p
				sum[fl + "2"] = (sum.get(fl + "2", Vector3.ZERO) as Vector3) + p * p
				n[fl] = int(n.get(fl, 0)) + 1
				n[fl + "2"] = n[fl]
	var out := {}
	for t: String in sum:
		out[t] = (sum[t] as Vector3) / float(n[t])
	return out


## How far (degrees) a hand is turned from facing its thigh, as its mesh shows it. A hand facing its thigh is thin
## across the body (x) and wide along it (z); one turned to face forward is the other way round. Its turn is
## `atan(spread across / spread along)` (0: facing the thigh or away from it; 90: facing forward or back), and a
## hand whose thumb lies behind the palm (its back to the thigh) adds 90. The joints are not used: a generated
## body's finger joints need not lie as its mesh's hand does.
func palm_error(eulers: Dictionary, side: String) -> float:
	var c := hand_centroids(eulers)
	var fl := side + "flat"
	if not c.has(fl) or not c.has(side + "thumb") or not c.has(side + "palm"):
		return 180.0
	var mean: Vector3 = c[fl]
	var sq: Vector3 = c[fl + "2"]
	var var_x := maxf(sq.x - mean.x * mean.x, 0.0)
	var var_z := maxf(sq.z - mean.z * mean.z, 0.0)
	var turn := rad_to_deg(atan2(sqrt(var_x), sqrt(var_z)))
	if (c[side + "thumb"] as Vector3).z < (c[side + "palm"] as Vector3).z:
		turn += 90.0
	return turn


## The arm eulers for one side: the upper arm's lowering `x` and forward swing `z`, the forearm twist `twist`, and
## the finger curl about `curl` (a unit axis with its sign, in the finger bones' frame; zero for none).
func _arm(side: String, x: float, z: float, twist: float, curl: Vector3, elbow := ELBOW_DEG) -> Dictionary:
	var m := 1.0 if side == "Left" else -1.0
	var e := {
		side + "UpperArm": Vector3(x, 0, m * z),
		# the twist about the forearm's own axis after the bend (Rx then Ry), so it turns the hand without
		# swinging the forearm; expressed in `stand_key`'s euler order
		side + "LowerArm": _deg((Basis(Vector3.RIGHT, deg_to_rad(elbow)) * Basis(Vector3.UP, deg_to_rad(m * twist))).get_euler()),
	}
	if curl != Vector3.ZERO:
		for f in FINGERS:
			for j: String in CURL_DEG:
				e[side + f + j] = curl * float(CURL_DEG[j])
	return e


## The upper arm's abduction, degrees: how far the shoulder -> elbow line swings out from straight down, in the
## body's frontal plane (positive: away from the body).
func _abduction(g: Array[Transform3D], side: String) -> float:
	var d := _world(g, side + "LowerArm") - _world(g, side + "UpperArm")
	var out := d.x if side == "Left" else -d.x
	return rad_to_deg(atan2(out, -d.y))


## Solve the relaxed arms. `base` is the clip's own arm eulers (bone -> degrees, for the upper arms' forward swing).
## The upper arm's lowering X is searched against the measured abduction, not assumed, and the hand's turn and the
## fingers' curl against the mesh, not the joints (see `palm_error`).
## Returns {"eulers": {bone: [x, y, z]}, "clearance_mm": [left, right], "abduction_deg": [left, right],
## "palm_error_deg": [left, right], "notes": [...]}.
func solve(base: Dictionary, sides: Array = ["Left", "Right"]) -> Dictionary:
	var eulers := {}
	var notes: Array[String] = []
	var palm := [0.0, 0.0]
	var abd := [0.0, 0.0]
	for side: String in sides:
		var m := 1.0 if side == "Left" else -1.0
		var idx := 0 if side == "Left" else 1
		var z0: float = absf((base.get(side + "UpperArm", Vector3(87, 0, m * 6.0)) as Vector3).z)
		# the X that hangs the upper arm straight down (abduction 0), and which way X swings it out
		var x0 := 0.0
		var best_a := INF
		for x in range(-30, 181, 2):
			var a := absf(_abduction(_globals(overrides(_arm(side, float(x), z0, 0.0, Vector3.ZERO))), side))
			if a < best_a:
				best_a = a
				x0 = float(x)
		for step: float in [1.0, 0.5, 0.25]:
			for d: float in [-step, step]:
				var a := absf(_abduction(_globals(overrides(_arm(side, x0 + d, z0, 0.0, Vector3.ZERO))), side))
				if a < best_a:
					best_a = a
					x0 += d
		var out_sign := 1.0 if _abduction(_globals(overrides(_arm(side, x0 + 5.0, z0, 0.0, Vector3.ZERO))), side) \
			> 0.0 else -1.0
		var hang := x0 + out_sign * HANG_ABDUCTION_DEG
		# the forearm twist that turns the palm to the thigh (the mesh's thumb forward)
		var twist := 0.0
		var best := INF
		for t in range(-180, 180, 15):
			var err := palm_error(_arm(side, hang, z0, float(t), Vector3.ZERO), side)
			if err < best:
				best = err
				twist = float(t)
		for step: float in [6.0, 3.0, 1.0]:
			for d: float in [-step, step]:
				var err := palm_error(_arm(side, hang, z0, twist + d, Vector3.ZERO), side)
				if err < best:
					best = err
					twist += d
		# the finger curl: of the finger bones' X and Z axes, either way, the one that closes the fingertips
		# most toward the thigh (medially), as the mesh shows them
		var curl := Vector3.ZERO
		var c0: Dictionary = hand_centroids(_arm(side, hang, z0, twist, Vector3.ZERO))
		var best_in := 0.0
		for axis: Vector3 in [Vector3.RIGHT, Vector3.LEFT, Vector3.BACK, Vector3.FORWARD]:
			var c1: Dictionary = hand_centroids(_arm(side, hang, z0, twist, axis))
			if not c0.has(side + "tip") or not c1.has(side + "tip"):
				continue
			var inward := -m * ((c1[side + "tip"] as Vector3).x - (c0[side + "tip"] as Vector3).x)
			if inward > best_in:
				best_in = inward
				curl = axis
		# out from the natural hang, and a little forward of it, until the lower forearm and the hand clear the
		# body by CLEAR_MM: the least deviation that does. First within the relaxed limits (STAGES[0]: abduction,
		# forward swing, elbow bend); only a side that cannot clear within them (where the hip, pushed out by the
		# stand's weight shift, is the widest point) takes the wider limits of STAGES[1], which also let the elbow
		# bend further so the hand hangs ahead of the hip. A side that cannot clear even then keeps its best and is
		# reported short.
		var best_c := -INF
		var pick := [hang, z0, ELBOW_DEG]
		var cleared := false
		for stage: Array in STAGES:
			var x := hang
			for _i in 60:
				for elbow: float in stage[2]:
					for dz: float in range(0, int(stage[1]) + 1, 3):
						var trial := eulers.duplicate()
						trial.merge(_arm(side, x, z0 + dz, twist, curl, elbow), true)
						var c: float = clearance(trial)[idx]
						if c > best_c:
							best_c = c
							pick = [x, z0 + dz, elbow]
						if c >= CLEAR_MM:
							pick = [x, z0 + dz, elbow]
							cleared = true
							break
					if cleared:
						break
				if cleared:
					break
				var nxt := eulers.duplicate()
				nxt.merge(_arm(side, x + out_sign, z0, twist, curl), true)
				if _abduction(_globals(overrides(nxt)), side) > float(stage[0]):
					break
				x += out_sign
			if cleared:
				break
		var x: float = pick[0]
		var z: float = pick[1]
		var elbow_deg: float = pick[2]
		eulers.merge(_arm(side, x, z, twist, curl, elbow_deg), true)
		palm[idx] = palm_error(eulers, side)
		abd[idx] = _abduction(_globals(overrides(eulers)), side)
		notes.append("%s: upper arm X %.1f Z %.1f (straight down at X %.1f), abduction %.1f deg, elbow %.0f, forearm twist %+.1f, "
			% [side, x, z, x0, abd[idx], elbow_deg, twist] + "curl about %s, palm %.1f deg off" % [curl, palm[idx]])
	var out := {}
	for b: String in eulers:
		var e: Vector3 = eulers[b]
		out[b] = [snappedf(e.x, 0.1), snappedf(e.y, 0.1), snappedf(e.z, 0.1)]
	var cl := clearance(eulers)
	return {"eulers": out, "clearance_mm": [snappedf(cl[0], 0.1), snappedf(cl[1], 0.1)],
		"abduction_deg": [snappedf(abd[0], 0.1), snappedf(abd[1], 0.1)],
		"palm_error_deg": [snappedf(palm[0], 0.1), snappedf(palm[1], 0.1)], "notes": notes}


static func _deg(e: Vector3) -> Vector3:
	return Vector3(rad_to_deg(e.x), rad_to_deg(e.y), rad_to_deg(e.z))
