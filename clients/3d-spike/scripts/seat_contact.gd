## Where a seated body touches its seat, from the body's own skinned geometry (S33-f; G-13's seat contact,
## `tools/character_pipeline/godot/pose_sheet.gd` `_seat_low`, restated for the running client).
##
## `Human.sit(seat_y)` puts the `Hips` joint at `seat_y`; how far the buttocks and the thighs' undersides
## hang below that joint depends on the body (its build, its clothes). This measures it, from the posed
## skeleton, skinning each seat vertex on the CPU exactly as the GPU does (Σ w · pose(bone) · bind), so a
## body is seated with its own lowest seat vertex on the seat -- one placement rule for every body,
## nothing per person (the charter's "tools first, no hand-tuning").
##
## The seat vertices: those whose strongest bone is the hips or an upper leg, from BEHIND_M behind the
## hips joint forward (a fused pack or a coat's tail hangs further back, against a backrest, and is not a
## seat contact), within AROUND_M of the joint sideways and forward.
class_name SeatContact
extends RefCounted

const SEAT_BONES := ["Hips", "LeftUpperLeg", "RightUpperLeg"]
const BEHIND_M := 0.10
const AROUND_M := 0.45


## The lowest seat vertex of `h`'s current pose, in world space, or INF when the body has no skinned
## seat vertex (a stand-in figure) or its skin did not read back whole (`SkinArrays`).
static func lowest(h: Human) -> float:
	var sk := h.skeleton
	if sk == null:
		return INF
	var hips := sk.find_bone("Hips")
	if hips < 0:
		return INF
	var hip_xf := sk.global_transform * sk.get_bone_global_pose(hips)
	var hip := hip_xf.origin
	var fwd := h.global_transform.basis.z.normalized()
	var side := h.global_transform.basis.x.normalized()
	var low := INF
	for mi: MeshInstance3D in sk.find_children("*", "MeshInstance3D", true, false):
		if not mi.visible or mi.skin == null or mi.mesh == null:
			continue
		var skin := mi.skin
		var mats: Array[Transform3D] = []
		var seat: Array[bool] = []
		for b in skin.get_bind_count():
			var bone := skin.get_bind_bone(b)
			if bone < 0:
				bone = sk.find_bone(skin.get_bind_name(b))
			mats.append(sk.global_transform * sk.get_bone_global_pose(bone) * skin.get_bind_pose(b))
			seat.append(sk.get_bone_name(bone) in SEAT_BONES)
		for s in mi.mesh.get_surface_count():
			var read := SkinArrays.read(mi.mesh, s, skin.get_bind_count(), "SeatContact: %s" % mi.name)
			if read.has("unreadable"):
				return INF   # a body whose skin did not read back whole is not measured at all
			if not read.has("v"):
				continue
			var v: PackedVector3Array = read["v"]
			var bi: PackedInt32Array = read["bi"]
			var bw: PackedFloat32Array = read["bw"]
			var per: int = read["per"]
			for i in v.size():
				var best_w := -1.0
				var best_b := 0
				for k in per:
					if bw[i * per + k] > best_w:
						best_w = bw[i * per + k]
						best_b = bi[i * per + k]
				if not seat[best_b]:
					continue
				var p := Vector3.ZERO
				for k in per:
					var w := bw[i * per + k]
					if w > 0.0:
						p += (mats[bi[i * per + k]] * v[i]) * w
				var d := p - hip
				var along := d.dot(fwd)
				if along < -BEHIND_M or along > AROUND_M or absf(d.dot(side)) > AROUND_M:
					continue
				low = minf(low, p.y)
	return low


## How far the `Hips` joint stands above the body's lowest seat vertex in its current pose, metres, or
## -1 when it cannot be measured.
static func hips_over_seat(h: Human) -> float:
	var sk := h.skeleton
	var low := lowest(h)
	if sk == null or low == INF:
		return -1.0
	var hips := sk.find_bone("Hips")
	return (sk.global_transform * sk.get_bone_global_pose(hips)).origin.y - low
