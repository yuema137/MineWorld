# SPDX-License-Identifier: MIT
# The standing-pose gates (PR S26-c-pose, part E; PIPELINE.md §15.2): every frame of the `Stand` loop, on the
# CPU-skinned mesh and the posed skeleton as the client draws them. It measures; nothing here sets a pose.
#
# - ROM. Each arm joint's angle, from the clinical zero, inside the normal active range. The profile's own axes
#   carry the decomposition: SkeletonProfileHumanoid's reference pose is a T-pose facing +Z, +Y runs from a joint to
#   its child and +X flexes the joint (Godot, "Retargeting 3D Skeletons"). Ranges (degrees):
#     shoulder flexion 180, extension 60, abduction 180, adduction 50, internal rotation 70, external 90;
#     elbow flexion 0-150; forearm pronation 80, supination 80; wrist flexion 80, extension 70,
#     radial deviation 20, ulnar deviation 30 — AAOS normal values (Greene & Heckman, *The Clinical Measurement
#     of Joint Motion*, AAOS 1994); elbow and forearm agree with Zwerus et al., Shoulder & Elbow 2017 (flexion mean
#     146, pronation 80); where CDC's reference values (Soucie et al., Haemophilia 2011) are stricter for the
#     shoulder and elbow, theirs apply: none is (their adult shoulder flexion and elbow flexion means exceed AAOS's).
# - Forearm rotation (operator ruling 2026-10-19, replacing the palm-direction gate). No supination, and pronation
#   from the thumb-forward neutral inside the relaxed band (0-60 deg; AAOS normal 0-80 sits in `ROM`).
# - Recorded, not gated, until the 22-body table sets their bounds (the same ruling): the palm normal's angle from
#   facing the thigh (plane fit of the metacarpal region, signed to the thumb's side), the thumb's lead over the palm
#   centre, the reach (fingertips against the front of the same-side thigh) and the CC-10 clearance (the lower half
#   of the forearm and the hand against the torso, hips and thighs, a carried pack left out).
extends RefCounted

## AAOS normal active ranges (see the header), degrees.
const ROM := {
	"shoulder_flexion": [-60.0, 180.0],  # extension 60, flexion 180
	"shoulder_abduction": [-50.0, 180.0],  # adduction 50, abduction 180
	"shoulder_rotation": [-90.0, 70.0],  # external 90 (-), internal 70 (+)
	"elbow_flexion": [0.0, 150.0],
	"forearm_pronation": [-80.0, 80.0],  # supination 80 (-), pronation 80 (+)
	"wrist_flexion": [-70.0, 80.0],  # extension 70 (-), flexion 80 (+)
	"wrist_deviation": [-30.0, 20.0],  # ulnar 30 (-), radial 20 (+)
}
## The palm faces the thigh within this angle (the operator's bound, PR S26-c-pose part E).
const PALM_DEG := 30.0
## The forearm-rotation gate (operator ruling 2026-10-19, replacing the palm-direction gate): pronation from the
## thumb-forward neutral, no supination (0) and at most the relaxed band's 60 degrees, inside AAOS's 0-80 (`ROM`).
## The palm angle, the reach and the clearance are recorded, not gated, until the 22-body table sets their bounds.
const PRONATION_RELAXED := [0.0, 60.0]
## CC-10: the arm clears the body by at least this (PIPELINE §15.2).
const CLEAR_MM := 20.0
const BODY := ["Hips", "Spine", "Chest", "UpperChest", "LeftUpperLeg", "RightUpperLeg"]
const FINGERS := ["Index", "Middle", "Ring", "Little"]
const SLAB_M := 0.02
const BINS := 48
## The share of the forearm, from the elbow, the clearance leaves out (at the elbow a relaxed arm may rest
## against a broad flank) and the limb filters of PIPELINE §15 G-9.
const FOREARM_FROM := 0.5
const LIMB_R := 0.08
const HAND_M := 0.20
const MIX_W := 0.05

var sk: Skeleton3D
var root: Node3D
var _verts: Array = []  # per surface: [v, bi, bw, per, bones, binds, group, hand tag, side]
var worst := {}
var frames := 0
var misses: Array = []


## `pack_core`: a fused backpack's core, as the bind made it rigid (fit.yaml `weights.pack_core`, fit_bind.py):
## behind the Spine1 joint by more than `behind_spine1_m`, from `above_hips_m` over the hips to `above_spine2_m` over
## Spine2, within `half_width_m` of the middle. A pack is an accessory, not the body: its vertices (and a separate
## `Pack` mesh's) are left out of the clearance outline.
func _init(skeleton: Skeleton3D, floor_node: Node3D, pack_core: Dictionary = {}) -> void:
	sk = skeleton
	root = floor_node
	var rest_root := root.global_transform.affine_inverse() * sk.global_transform
	var j := {}
	for b in ["Hips", "Chest", "UpperChest"]:
		j[b] = (rest_root * sk.get_bone_global_rest(sk.find_bone(b))).origin
	for mi: MeshInstance3D in sk.find_children("*", "MeshInstance3D", true, false):
		if not mi.visible or mi.skin == null or "Pack" in String(mi.name):
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
			var grp := PackedStringArray()
			var tag := PackedStringArray()
			for i in v.size():
				var best := -1.0
				var bb := 0
				for k in per:
					if bw[i * per + k] > best:
						best = bw[i * per + k]
						bb = bones[bi[i * per + k]]
				var name := sk.get_bone_name(bb)
				var g := _group(name)
				var t := ""
				if g.ends_with("hand") or g.ends_with("forearm"):
					var arm_w := 0.0
					for k in per:
						var gk := _group(sk.get_bone_name(bones[bi[i * per + k]]))
						if gk.length() > 1 and gk != "body" and not gk.ends_with("thigh"):
							arm_w += bw[i * per + k]
					if arm_w < 1.0 - MIX_W:
						g = ""
					elif g.ends_with("hand"):
						t = _hand_tag(name)
				if g == "body" and not pack_core.is_empty():
					var pr := Vector3.ZERO
					for k in per:
						var wk := bw[i * per + k]
						if wk > 0.0:
							pr += (rest_root * sk.get_bone_global_rest(bones[bi[i * per + k]]) * binds[bi[i * per + k]] * v[i]) * wk
					if pr.z < (j["Chest"] as Vector3).z - float(pack_core["behind_spine1_m"]) \
							and pr.y > (j["Hips"] as Vector3).y + float(pack_core["above_hips_m"]) \
							and pr.y < (j["UpperChest"] as Vector3).y + float(pack_core["above_spine2_m"]) \
							and absf(pr.x) < float(pack_core["half_width_m"]):
						g = ""
				grp.append(g)
				tag.append(t)
			_verts.append([v, bi, bw, per, bones, binds, grp, tag])


static func _group(bone: String) -> String:
	var side := "L" if bone.begins_with("Left") else "R"
	if bone.ends_with("UpperLeg"):
		return side + "thigh"
	if bone in BODY:
		return "body"
	for f in ["Thumb", "Index", "Middle", "Ring", "Little"]:
		if f in bone:
			return side + "hand"
	if bone.ends_with("Hand"):
		return side + "hand"
	if bone.ends_with("LowerArm"):
		return side + "forearm"
	return ""


static func _hand_tag(bone: String) -> String:
	if "Thumb" in bone:
		return "thumb"
	if bone.ends_with("Distal"):
		return "tip"
	return "flat"


static func _bin(q: Vector2) -> int:
	return int(floor((q.angle() + PI) / TAU * BINS)) % BINS


## Symmetric 3x3 eigenvector of the smallest eigenvalue (Jacobi).
static func _thin_axis(c: Basis) -> Vector3:
	var a := [[c.x.x, c.y.x, c.z.x], [c.x.y, c.y.y, c.z.y], [c.x.z, c.y.z, c.z.z]]
	var v := [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
	for _sweep in 30:
		for p in 3:
			for q in range(p + 1, 3):
				if absf(a[p][q]) < 1e-15:
					continue
				var th := 0.5 * atan2(2.0 * a[p][q], a[q][q] - a[p][p])
				var cs := cos(th)
				var sn := sin(th)
				for k in 3:
					var akp: float = a[k][p]
					var akq: float = a[k][q]
					a[k][p] = cs * akp - sn * akq
					a[k][q] = sn * akp + cs * akq
				for k in 3:
					var apk: float = a[p][k]
					var aqk: float = a[q][k]
					a[p][k] = cs * apk - sn * aqk
					a[q][k] = sn * apk + cs * aqk
				for k in 3:
					var vkp: float = v[k][p]
					var vkq: float = v[k][q]
					v[k][p] = cs * vkp - sn * vkq
					v[k][q] = sn * vkp + cs * vkq
	var m := 0
	for i in 3:
		if a[i][i] < a[m][m]:
			m = i
	return Vector3(v[0][m], v[1][m], v[2][m]).normalized()


## Twist (degrees) of a rotation about a unit axis (swing-twist decomposition).
static func _twist(q: Quaternion, axis: Vector3) -> float:
	var p := Vector3(q.x, q.y, q.z).dot(axis)
	var t := Quaternion(axis.x * p, axis.y * p, axis.z * p, q.w)
	if t.length() < 1e-9:
		return 0.0
	t = t.normalized()
	var ang := 2.0 * atan2(Vector3(t.x, t.y, t.z).dot(axis), t.w)
	return rad_to_deg(wrapf(ang, -PI, PI))


## The swing of a rotation (its twist about `axis` removed) as an angle about `about` (degrees): the projection
## of the swung axis onto the plane perpendicular to `about`.
static func _swing_about(q: Quaternion, axis: Vector3, about: Vector3) -> float:
	var d := q * axis
	var a := about.normalized()
	var p0 := (axis - a * axis.dot(a)).normalized()
	var p1 := (d - a * d.dot(a)).normalized()
	return rad_to_deg(atan2(a.dot(p0.cross(p1)), p0.dot(p1)))


func _rel(poses: Array, bone: String) -> Quaternion:
	var i := sk.find_bone(bone)
	var p := sk.get_bone_parent(i)
	var parent_pose: Transform3D = poses[p] if p >= 0 else Transform3D.IDENTITY
	var parent_rest := sk.get_bone_global_rest(p) if p >= 0 else Transform3D.IDENTITY
	var local_pose := parent_pose.basis.inverse() * (poses[i] as Transform3D).basis
	var local_rest := parent_rest.basis.inverse() * sk.get_bone_global_rest(i).basis
	return (local_rest.get_rotation_quaternion().inverse() * local_pose.get_rotation_quaternion()).normalized()


## The arm joints' clinical angles on one side, from the posed skeleton.
func rom(poses: Array, side: String) -> Dictionary:
	var m := 1.0 if side == "Left" else -1.0
	var chest: Basis = (poses[sk.find_bone("UpperChest")] as Transform3D).basis.orthonormalized()
	var sh: Vector3 = (poses[sk.find_bone(side + "UpperArm")] as Transform3D).origin
	var el: Vector3 = (poses[sk.find_bone(side + "LowerArm")] as Transform3D).origin
	# the upper arm's direction in the chest's frame (x left, y up, z forward: the profile's body axes)
	var d := chest.inverse() * (el - sh).normalized()
	var down := -d.y
	var out := {}
	out["shoulder_flexion"] = rad_to_deg(atan2(d.z, down))
	out["shoulder_abduction"] = rad_to_deg(atan2(m * d.x, down))
	# rotation of the humerus about its own axis from the T-pose rest (lowering the arm in the frontal plane is a
	# pure swing, so the rest's twist is the clinical neutral); internal positive
	out["shoulder_rotation"] = m * _twist(_rel(poses, side + "UpperArm"), Vector3.UP)
	var lower := _rel(poses, side + "LowerArm")
	out["elbow_flexion"] = _swing_about(lower, Vector3.UP, Vector3.RIGHT)
	var hand := _rel(poses, side + "Hand")
	# the forearm's axial rotation from the T-pose rest (palm down, thumb forward: neutral); the hand's own axial
	# twist, which a wrist cannot make, is counted as forearm rotation; pronation positive
	out["forearm_pronation"] = m * (_twist(lower, Vector3.UP) + _twist(hand, Vector3.UP))
	out["wrist_flexion"] = _swing_about(hand, Vector3.UP, Vector3.RIGHT)
	out["wrist_deviation"] = m * _swing_about(hand, Vector3.UP, Vector3.BACK)
	return out


## One frame: skin the mesh, measure every gate, keep the worst values and every miss.
func frame(poses: Array, frame_index: int) -> void:
	frames += 1
	var to_root := root.global_transform.affine_inverse() * sk.global_transform
	var body := {}
	var arm: Array = []
	var hand := {"L": {}, "R": {}}
	var thigh_front := {"L": -INF, "R": -INF}
	for e: Array in _verts:
		var v: PackedVector3Array = e[0]
		var bi: PackedInt32Array = e[1]
		var bw: PackedFloat32Array = e[2]
		var per: int = e[3]
		var bones: PackedInt32Array = e[4]
		var binds: Array = e[5]
		var grp: PackedStringArray = e[6]
		var tag: PackedStringArray = e[7]
		var mats: Array[Transform3D] = []
		for b in bones.size():
			mats.append(to_root * (poses[bones[b]] as Transform3D) * (binds[b] as Transform3D))
		for i in v.size():
			var g := grp[i]
			if g == "":
				continue
			var p := Vector3.ZERO
			for k in per:
				var w := bw[i * per + k]
				if w > 0.0:
					p += (mats[bi[i * per + k]] * v[i]) * w
			if g == "body" or g.ends_with("thigh"):
				var s := int(floor(p.y / SLAB_M))
				var slab: PackedVector2Array = body.get(s, PackedVector2Array())
				slab.append(Vector2(p.x, p.z))
				body[s] = slab
				if g.ends_with("thigh"):
					thigh_front[g.left(1)] = maxf(thigh_front[g.left(1)], p.z)
			else:
				arm.append([p, g])
				if tag[i] != "":
					var hs: Dictionary = hand[g.left(1)]
					if not hs.has(tag[i]):
						hs[tag[i]] = PackedVector3Array()
					var arr: PackedVector3Array = hs[tag[i]]
					arr.append(p)
					hs[tag[i]] = arr
	var res := {}
	# clearance
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
	var clear := {"L": INF, "R": INF}
	for sd in ["L", "R"]:
		var side := "Left" if sd == "L" else "Right"
		var e0: Vector3 = to_root * (poses[sk.find_bone(side + "LowerArm")] as Transform3D).origin
		var e1: Vector3 = to_root * (poses[sk.find_bone(side + "Hand")] as Transform3D).origin
		for a: Array in arm:
			var g: String = a[1]
			if g.left(1) != sd:
				continue
			var p: Vector3 = a[0]
			if g.ends_with("forearm"):
				if (p - e0).dot(e1 - e0) / maxf((e1 - e0).length_squared(), 1e-9) < FOREARM_FROM:
					continue
				if p.distance_to(Geometry3D.get_closest_point_to_segment(p, e0, e1)) > LIMB_R:
					continue
			elif p.distance_to(Geometry3D.get_closest_point_to_segment(p, e1, e1 + (e1 - e0).normalized() * HAND_M)) \
					> LIMB_R:
				continue
			var s := int(floor(p.y / SLAB_M))
			if not hulls.has(s):
				continue
			var c: Vector2 = hulls[s][0]
			var r: PackedFloat32Array = hulls[s][1]
			var q := Vector2(p.x, p.z) - c
			clear[sd] = minf(clear[sd], (q.length() - r[_bin(q)]) * 1000.0)
	for sd in ["L", "R"]:
		var side := "Left" if sd == "L" else "Right"
		var m := 1.0 if sd == "L" else -1.0
		res[side] = {"clearance_mm": clear[sd]}
		# palm: plane fit of the metacarpal region alone (the hand's non-thumb vertices between the wrist and the
		# middle finger's knuckle, along that line), so curled fingers neither tilt the plane nor move its centre;
		# signed to the thumb's side
		var hs: Dictionary = hand[sd]
		var w0: Vector3 = to_root * (poses[sk.find_bone(side + "Hand")] as Transform3D).origin
		var w1: Vector3 = to_root * (poses[sk.find_bone(side + "MiddleProximal")] as Transform3D).origin
		var flat := PackedVector3Array()
		for key in ["flat", "tip"]:
			for p: Vector3 in hs.get(key, PackedVector3Array()):
				var t := (p - w0).dot(w1 - w0) / maxf((w1 - w0).length_squared(), 1e-9)
				if t >= 0.0 and t <= 1.0:
					flat.append(p)
		var thumb: PackedVector3Array = hs.get("thumb", PackedVector3Array())
		var tips: PackedVector3Array = hs.get("tip", PackedVector3Array())
		if flat.size() >= 8 and thumb.size() > 0:
			var c := Vector3.ZERO
			for p in flat:
				c += p
			c /= flat.size()
			var cov := Basis(Vector3.ZERO, Vector3.ZERO, Vector3.ZERO)
			for p in flat:
				var d := p - c
				cov.x += d * d.x
				cov.y += d * d.y
				cov.z += d * d.z
			var n := _thin_axis(cov)
			var tc := Vector3.ZERO
			for p in thumb:
				tc += p
			tc /= thumb.size()
			if (tc - c).dot(n) < 0.0:
				n = -n
			# 0: the palm faces the thigh (inward lateral, -x on the left, +x on the right)
			res[side]["palm_deg"] = rad_to_deg(n.angle_to(Vector3(-m, 0, 0)))
			res[side]["thumb_ahead_mm"] = (tc.z - c.z) * 1000.0
			var tip_front := -INF
			for p in tips:
				tip_front = maxf(tip_front, p.z)
			res[side]["reach_mm"] = (tip_front - float(thigh_front[sd])) * 1000.0
		res[side]["rom"] = rom(poses, side)
	_keep(res, frame_index)


func _keep(res: Dictionary, frame_index: int) -> void:
	for side in ["Left", "Right"]:
		var r: Dictionary = res[side]
		var w: Dictionary = worst.get(side, {})
		w["clearance_mm"] = minf(float(w.get("clearance_mm", INF)), float(r["clearance_mm"]))
		if r.has("palm_deg"):
			w["palm_deg"] = maxf(float(w.get("palm_deg", -INF)), float(r["palm_deg"]))
			w["thumb_ahead_mm"] = minf(float(w.get("thumb_ahead_mm", INF)), float(r["thumb_ahead_mm"]))
			w["reach_mm"] = maxf(float(w.get("reach_mm", -INF)), float(r["reach_mm"]))
		else:
			w["palm_deg"] = INF
		# the forearm-rotation gate (operator ruling 2026-10-19): no supination, pronation inside the relaxed band
		var pron: float = r["rom"]["forearm_pronation"]
		if pron < PRONATION_RELAXED[0] or pron > PRONATION_RELAXED[1]:
			_miss(side, "forearm rotation (supination below 0, or past the relaxed pronation band)", pron,
				"%s..%s deg" % [PRONATION_RELAXED[0], PRONATION_RELAXED[1]], frame_index)
		var rw: Dictionary = w.get("rom", {})
		for k: String in r["rom"]:
			var v: float = r["rom"][k]
			var lohi: Array = rw.get(k, [INF, -INF])
			rw[k] = [minf(lohi[0], v), maxf(lohi[1], v)]
			if v < float(ROM[k][0]) or v > float(ROM[k][1]):
				_miss(side, k, v, "%s..%s deg (AAOS)" % [ROM[k][0], ROM[k][1]], frame_index)
		w["rom"] = rw
		worst[side] = w


func _miss(side: String, what: String, value: float, bound: String, frame_index: int) -> void:
	if misses.size() < 40:
		misses.append("%s %s %.1f outside %s at stand frame %d" % [side, what, value, bound, frame_index])


## The block recorded per body: the worst value of every gate over the loop, the misses, and the verdict.
## `free` names the sides the gates apply to (the protagonist's strap hand is held by her grip, not the clip).
func block(free: Array) -> Dictionary:
	var out := {"frames": frames, "bounds": {"pronation_relaxed_deg": PRONATION_RELAXED, "rom_deg": ROM,
		"recorded_only": ["palm_deg", "thumb_ahead_mm", "reach_mm", "clearance_mm"]}, "sides": {}}
	var ok := true
	for side in ["Left", "Right"]:
		var w: Dictionary = worst.get(side, {})
		var rw := {}
		for k: String in w.get("rom", {}):
			rw[k] = [snappedf(w["rom"][k][0], 0.1), snappedf(w["rom"][k][1], 0.1)]
		out["sides"][side] = {
			"gated": side in free,
			"palm_deg": snappedf(float(w.get("palm_deg", INF)), 0.1) if w.get("palm_deg", INF) < INF else null,
			"thumb_ahead_mm": snappedf(float(w.get("thumb_ahead_mm", 0.0)), 0.1),
			"reach_mm": snappedf(float(w.get("reach_mm", 0.0)), 0.1),
			"clearance_mm": snappedf(float(w.get("clearance_mm", 0.0)), 0.1),
			"rom_deg": rw,
		}
	var gated_misses: Array = []
	for m: String in misses:
		if m.get_slice(" ", 0) in free:
			gated_misses.append(m)
	ok = gated_misses.is_empty()
	out["misses"] = gated_misses
	out["pass"] = ok
	return out
