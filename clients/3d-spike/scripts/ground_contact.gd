## Keeps a body's soles on the floor, from the body's own geometry (PR S26-c-ground,
## `.structured-coding/plans/mvp0/pr-s26c-ground.md`).
##
## Why it exists. The locomotion clips are the Quaternius mannequin's, retargeted: the leg and foot
## *rotations* carry over, and the hips' *translation* is scaled by this body's hips height
## (`Normalize Position Tracks` sets `Skeleton3D.motion_scale`). That keeps a body whose feet are a
## scaled copy of the mannequin's on the floor, and no other. Heel strike and toe-off roll the foot about
## the ankle, so how far the heel, the ball and the toe cap drop below the ankle depends on where this
## body's ankle joint sits over its own sole and how far its shoe reaches in front of and behind it; and
## the generated bodies differ from the mannequin there (ankles picked from 0.049 m to 0.102 m above
## the sole). Measured before this existed: every cast body and the protagonist pushed a toe or heel
## 22 to 60 mm through the floor somewhere in the Walk or Jog cycle (pose-sheet ground probe,
## PIPELINE.md §15.1). No one hips-height scale fixes that, because the error follows the foot's roll.
##
## What it does. Once per body (cached by scene), it takes the vertices that can touch the floor -- the
## support points of the lower legs' and feet's rest geometry over a fan of pitches and rolls -- with
## their skin weights. Every frame, after the animation and `Posture`, it skins those points exactly as
## the GPU does (Σ w · pose · bind), finds the lowest, and moves the `Hips` vertically:
##
## - by as much as it sank below the floor, always (nothing goes through the floor);
## - by as much as it floats above it, scaled by [member pin]: 1 while standing or walking (a foot is
##   always on the floor), 0 at a jog (its flight phases leave the floor on purpose). `Human.set_gait`
##   sets it.
##
## It also gives `Human` the body's rest sole ([member sole_y]): the cast exports put their soles at
## y = -0.0177 (pipeline standard G-4, inherited from the protagonist's export), and the client stands a
## body's scene origin on the floor, so a body is raised by its own measured sole depth.
class_name GroundContact
extends SkeletonModifier3D

## The bones whose skin can reach the floor in a gait: the shins (a trouser hem), the feet and the toes.
const BONES := ["LeftLowerLeg", "LeftFoot", "LeftToes", "RightLowerLeg", "RightFoot", "RightToes"]
## How far above the lowest rest vertex a vertex is still a candidate (the shoe and the hem).
const BAND_M := 0.12
## The fan of foot orientations whose support points are kept: pitch (toe down +) and roll, degrees.
const PITCHES := [-70.0, -55.0, -40.0, -30.0, -20.0, -10.0, 0.0, 10.0, 20.0, 30.0, 40.0, 55.0, 70.0]
const ROLLS := [-20.0, -10.0, 0.0, 10.0, 20.0]

## scene path -> {"pts": PackedVector3Array (bind space), "bones": PackedInt32Array, "w": PackedFloat32Array,
## "per": int, "binds": {bone: Transform3D}, "sole_y": float}
static var _cache: Dictionary = {}

## The node whose origin stands on the floor (the `Human`); the floor is its local y = 0.
var floor_node: Node3D
## 1: the lowest point is held on the floor; 0: only kept from going below it. See the header.
var pin := 1.0
## The rest sole's height in the scene instance's frame (negative: below its origin), from the vertices.
var sole_y := 0.0
## The last correction applied, metres in the floor node's frame (diagnostics and the probe).
var last_lift := 0.0
var _d: Dictionary


## The contact data for `sk`, an instance of the scene at `key`; `chain` is the skeleton's transform in the
## scene instance's frame (for [member sole_y]).
static func for_body(sk: Skeleton3D, key: String, chain: Transform3D) -> GroundContact:
	var g := GroundContact.new()
	g.name = "GroundContact"
	if key == "":
		g._d = _prepare(sk, chain)
	else:
		if not _cache.has(key):
			_cache[key] = _prepare(sk, chain)
		g._d = _cache[key]
	g.sole_y = float(g._d["sole_y"])
	return g


static func _prepare(sk: Skeleton3D, chain: Transform3D) -> Dictionary:
	var wanted := {}
	for b in BONES:
		var i := sk.find_bone(b)
		if i >= 0:
			wanted[i] = sk.get_bone_name(i).begins_with("Left")
	# the floor's vertical and the body's forward and left, in skeleton space (the body faces its +Z)
	var inv := chain.basis.inverse()
	var up := (inv * Vector3.UP).normalized()
	var fwd := (inv * Vector3.BACK).normalized()
	var left_axis := up.cross(fwd).normalized()
	# the vertices a floor bone dominates, skinned at rest (Σ w · rest · bind): the floor can only meet these
	var cand: Array = []  # [rest position, bind-space vertex, bones, weights, binds, is_left]
	var sole := INF
	var low := INF
	for mi: MeshInstance3D in sk.find_children("*", "MeshInstance3D", true, false):
		if mi.skin == null or mi.mesh == null:
			continue
		var skin := mi.skin
		var bones := PackedInt32Array()
		var bind: Array[Transform3D] = []
		var rest_bind: Array[Transform3D] = []
		for b in skin.get_bind_count():
			var bone := skin.get_bind_bone(b)
			bone = bone if bone >= 0 else sk.find_bone(skin.get_bind_name(b))
			bones.append(bone)
			bind.append(skin.get_bind_pose(b))
			rest_bind.append(sk.get_bone_global_rest(bone) * skin.get_bind_pose(b))
		for s in mi.mesh.get_surface_count():
			var arr := mi.mesh.surface_get_arrays(s)
			var v: PackedVector3Array = arr[Mesh.ARRAY_VERTEX]
			var bi: PackedInt32Array = arr[Mesh.ARRAY_BONES]
			var bw: PackedFloat32Array = arr[Mesh.ARRAY_WEIGHTS]
			var per := bi.size() / maxi(v.size(), 1)
			for i in v.size():
				var best := -1.0
				var dom := -1
				for k in per:
					if bw[i * per + k] > best:
						best = bw[i * per + k]
						dom = bones[bi[i * per + k]]
				if not wanted.has(dom):
					continue
				var p := Vector3.ZERO
				var vb := PackedInt32Array()
				var vw := PackedFloat32Array()
				var vbind: Array[Transform3D] = []
				for k in per:
					var w := bw[i * per + k]
					if w > 0.0:
						var b := bi[i * per + k]
						p += (rest_bind[b] * v[i]) * w
						vb.append(bones[b])
						vw.append(w)
						vbind.append(bind[b])
				sole = minf(sole, (chain * p).y)
				low = minf(low, p.dot(up))
				cand.append([p, v[i], vb, vw, vbind, wanted[dom]])
	# keep the support points (the lowest vertex under each foot orientation of a fan), per side, of the
	# candidates near the floor
	var near: Array = cand.filter(func(c: Array) -> bool: return (c[0] as Vector3).dot(up) <= low + BAND_M)
	var keep := {}
	for is_left in [true, false]:
		var mine: Array = near.filter(func(c: Array) -> bool: return c[5] == is_left)
		for pitch: float in PITCHES:
			for roll: float in ROLLS:
				var down := (-up).rotated(left_axis, deg_to_rad(pitch)).rotated(fwd, deg_to_rad(roll))
				var best := -INF
				var at: Array = []
				for c: Array in mine:
					var d := (c[0] as Vector3).dot(down)
					if d > best:
						best = d
						at = c
				if not at.is_empty():
					keep[at] = true
	var out := {"pts": PackedVector3Array(), "bones": [], "w": [], "binds": [], "sole_y": sole if sole < INF else 0.0}
	for c: Array in keep:
		out["pts"].append(c[1])
		out["bones"].append(c[2])
		out["w"].append(c[3])
		out["binds"].append(c[4])
	return out


## The lowest contact point's height in the floor node's frame, under the skeleton's current poses.
func lowest() -> float:
	var sk := get_skeleton()
	var to_floor := floor_node.global_transform.affine_inverse() * sk.global_transform
	var poses := {}
	var low := INF
	var pts: PackedVector3Array = _d["pts"]
	for j in pts.size():
		var bones: PackedInt32Array = _d["bones"][j]
		var ws: PackedFloat32Array = _d["w"][j]
		var binds: Array = _d["binds"][j]
		var p := Vector3.ZERO
		for k in bones.size():
			var b := bones[k]
			if not poses.has(b):
				poses[b] = sk.get_bone_global_pose(b)
			p += ((poses[b] as Transform3D) * (binds[k] as Transform3D) * pts[j]) * ws[k]
		low = minf(low, (to_floor * p).y)
	return low


func _process_modification() -> void:
	var sk := get_skeleton()
	if sk == null or floor_node == null or (_d["pts"] as PackedVector3Array).is_empty():
		return
	var low := lowest()
	var lift := -low if low < 0.0 else -low * pin
	last_lift = lift
	if absf(lift) < 1e-5:
		return
	var hips := sk.find_bone("Hips")
	if hips < 0:
		return
	# the floor's vertical, in the space the Hips' position is expressed in (its parent's, or the skeleton's)
	var to_floor := floor_node.global_transform.affine_inverse() * sk.global_transform
	var up_sk := to_floor.basis.inverse() * Vector3(0, lift, 0)
	var parent := sk.get_bone_parent(hips)
	if parent >= 0:
		up_sk = sk.get_bone_global_pose(parent).basis.inverse() * up_sk
	sk.set_bone_pose_position(hips, sk.get_bone_pose_position(hips) + up_sk)
