@tool
extends SceneTree
## Measure the standing pose instead of guessing it.
##
##     godot --headless --path clients/3d-spike --script tools/stand_pose.gd -- sweep
##     godot --headless --path clients/3d-spike --script tools/stand_pose.gd -- eval
##
## `sweep` turns each bone the `Stand` clip keys by +20 degrees about each of its
## local axes, from the rest pose, and reports which way the limb end actually
## moves in her own frame. The sit clip's arms were first keyed on a guessed
## axis and she sat with them held out; this is the measurement that replaces
## the guess.
##
## `eval` poses the skeleton exactly as the clip's first key does and reports
## what a viewer would check: both soles against the floor, where the weight
## sits over the feet, the gripping knuckle against the strap, the free hand
## against the thigh, and which way the face points.
##
## Godot frame: +Y up, she faces +Z, her own left is +X.
##
## Add `reference` to any mode to measure the default character's own body
## (`Human.Body.REFERENCE`) instead of the town body; `strap` then prints the
## front surface across her left chest, which is where `STRAP_D` was read.

## The strap's centre line at chest height -- `solve_grip.gd`'s target.
const STRAP := Vector3(0.092, 1.250, 0.152)
## On the reference body the target is where the knuckle sits, read from its
## rest mesh by `strap`: the strap's ridge runs at x 0.14-0.16, z 0.015-0.025
## from y 1.31 up to the shoulder (lower down it turns round her side), and the
## knuckle sits ~2.5 cm proud of it with the fingers wrapped round.
## Raised from y 1.31 to 1.36 (§8.7): at 1.31 the fist sat at the hoodie's open
## edge, where the strap runs under the cloth, and read as pushed into it; at
## 1.36 the padded strap lies on top of the hoodie's shoulder.
const STRAP_D := Vector3(0.150, 1.360, 0.045)

var sk: Skeleton3D
var body := Human.Body.TOWN
var mesh: MeshInstance3D


func _init() -> void:
	var args := OS.get_cmdline_user_args()
	if "reference" in args:
		body = Human.Body.REFERENCE
	var h := Human.build(Human.CANONICAL_HEIGHT, NPC.REF_SKIN, NPC.REF_HAIR,
		NPC.REF_TEE, NPC.REF_JEANS, NPC.REF_SHOE, NPC.REF_HOODIE, NPC.REF_PACK, body)
	root.add_child(h)
	sk = h.skeleton
	mesh = sk.find_children("*", "MeshInstance3D", true, false)[0] as MeshInstance3D
	for c in sk.get_children():
		if c is SkeletonModifier3D:
			sk.remove_child(c)
			c.queue_free()
	if "sweep" in args:
		_sweep()
	if "legs" in args:
		_legs()
	if "strap" in args:
		_strap_probe()
	if "grip" in args:
		_grip()
	if "eval" in args:
		_reset()
		var s := _skx()
		print("rest     ankles y %.3f / %.3f   toes y %.3f / %.3f" % [
			(s * _p("LeftFoot")).y, (s * _p("RightFoot")).y,
			(s * _p("LeftToes")).y, (s * _p("RightToes")).y])
		for t in [0.0, 3.15, 6.3, 9.45]:
			print("--- t = %.2f" % t)
			_eval(Human.stand_key(t, true, body))
	quit(0)


func _strap_target() -> Vector3:
	return STRAP_D if body == Human.Body.REFERENCE else STRAP


## The rest mesh's front-most surface (largest z) in 1 cm cells across her left
## chest, in the skeleton's parent frame -- the frame `STRAP` is written in.
## The strap stands proud of the hoodie, so it shows as a ridge in z.
func _strap_probe() -> void:
	_reset()
	var xf := _skx()
	# skinned by hand at rest: bone rest * bind pose, weighted -- the mesh
	# node's own transform is not what places a skinned surface
	var arr := mesh.mesh.surface_get_arrays(0)
	var verts: PackedVector3Array = arr[Mesh.ARRAY_VERTEX]
	var bones: PackedInt32Array = arr[Mesh.ARRAY_BONES]
	var wts: PackedFloat32Array = arr[Mesh.ARRAY_WEIGHTS]
	var skin := mesh.skin
	var bind := []
	for i in skin.get_bind_count():
		var bi := skin.get_bind_bone(i)
		if bi < 0:
			bi = sk.find_bone(skin.get_bind_name(i))
		bind.append(_g_rest(bi) * skin.get_bind_pose(i))
	var pts := PackedVector3Array()
	for vi in verts.size():
		var p := Vector3.ZERO
		for j in 4:
			var w := wts[vi * 4 + j]
			if w > 0.0:
				p += w * ((bind[bones[vi * 4 + j]] as Transform3D) * verts[vi])
		pts.append(xf * p)
	var cols := range(6, 25, 2)
	var line := "  y\\x  "
	for cx in cols:
		line += "%6.2f" % (cx / 100.0)
	print(line)
	for yi in range(140, 109, -3):
		var y := yi / 100.0
		line = "%5.2f  " % y
		for cx in cols:
			var best := -9.0
			for p in pts:
				if absf(p.x - cx / 100.0) < 0.006 and absf(p.y - y) < 0.006 and p.z > best:
					best = p.z
			line += ("%6.3f" % best) if best > -9.0 else "    - "
		print(line)
	for b in ["LeftShoulder", "LeftUpperArm", "LeftLowerArm", "LeftHand", "UpperChest"]:
		var p := xf * _p(b)
		print("rest %-13s (%.3f, %.3f, %.3f)" % [b, p.x, p.y, p.z])


## The skeleton's world transform, composed from its parents' local ones:
## `global_transform` is not available before the tree is entered, and it
## answered with identity -- silently, apart from an error line.
func _skx() -> Transform3D:
	var t := Transform3D.IDENTITY
	var n: Node = sk
	while n != null:
		if n is Node3D:
			t = (n as Node3D).transform * t
		n = n.get_parent()
	return t


## Bone index -> global transform, multiplied out by hand: outside a frame
## `get_bone_global_pose` answers with a stale value (see solve_grip.gd).
func _g(b: int) -> Transform3D:
	var t := Transform3D.IDENTITY
	while b != -1:
		t = sk.get_bone_pose(b) * t
		b = sk.get_bone_parent(b)
	return t


func _p(name: String) -> Vector3:
	return _g(sk.find_bone(name)).origin


func _reset() -> void:
	for i in sk.get_bone_count():
		sk.set_bone_pose_rotation(i, sk.get_bone_rest(i).basis.get_rotation_quaternion())
		sk.set_bone_pose_position(i, sk.get_bone_rest(i).origin)


## Exactly what the `Stand` clip keys: the rest rotation times the delta.
func _pose_bone(name: String, e: Vector3) -> void:
	var i := sk.find_bone(name)
	sk.set_bone_pose_rotation(i, sk.get_bone_rest(i).basis.get_rotation_quaternion() * Human.euler_q(e))


func _apply(pose: Dictionary) -> void:
	_reset()
	for bone: String in pose:
		if bone == "Hips@pos":
			continue
		_pose_bone(bone, pose[bone])
	if pose.has("Hips@pos"):
		var h := sk.find_bone("Hips")
		sk.set_bone_pose_position(h, sk.get_bone_rest(h).origin + pose["Hips@pos"])


## Global rest transform of a bone, for carrying a point with it.
func _g_rest(b: int) -> Transform3D:
	var t := Transform3D.IDENTITY
	while b != -1:
		t = sk.get_bone_rest(b) * t
		b = sk.get_bone_parent(b)
	return t


## The strap, carried by whatever the chest is doing in this pose.
func _strap_posed() -> Vector3:
	var c := sk.find_bone("UpperChest")
	return _skx() * (_g(c) * _g_rest(c).affine_inverse() * (_skx().affine_inverse() * _strap_target()))


## Solve the gripping arm inside the stand pose: knuckle on the posed strap,
## elbow hanging down and slightly out rather than winged up behind her.
func _grip() -> void:
	var pose := Human.stand_key(0.0, true, body)
	_apply(pose)
	var target := _strap_posed()
	var wrist := body == Human.Body.REFERENCE
	var best_a: Vector3 = pose["LeftUpperArm"]
	var best_b: Vector3 = pose["LeftLowerArm"]
	var best_c: Vector3 = pose.get("LeftHand", Vector3.ZERO) if wrist else Vector3.INF
	var best := _grip_err(best_a, best_b, target, best_c)
	var rng := RandomNumberGenerator.new()
	rng.seed = 20261001
	var dims := 9 if wrist else 6
	for restart in 30:
		var a := best_a if restart == 0 else Vector3(rng.randf_range(20, 110), rng.randf_range(-60, 60), rng.randf_range(-40, 80))
		var b := best_b if restart == 0 else Vector3(rng.randf_range(40, 150), rng.randf_range(-60, 60), rng.randf_range(-40, 40))
		var c := best_c if (restart == 0 or not wrist) else Vector3(rng.randf_range(-50, 50), rng.randf_range(-40, 40), rng.randf_range(-50, 50))
		var err := _grip_err(a, b, target, c)
		var step := 16.0
		while step > 0.2:
			var improved := false
			for axis in dims:
				for s in [1.0, -1.0]:
					var a2 := a
					var b2 := b
					var c2 := c
					if axis < 3:
						a2[axis] += s * step
					elif axis < 6:
						b2[axis - 3] += s * step
					else:
						# a wrist bends about 70 degrees, not a full turn
						c2[axis - 6] = clampf(c2[axis - 6] + s * step, -75.0, 75.0)
					var e2 := _grip_err(a2, b2, target, c2)
					if e2 < err - 1e-7:
						err = e2
						a = a2
						b = b2
						c = c2
						improved = true
			if not improved:
				step *= 0.5
		if err < best:
			best = err
			best_a = a
			best_b = b
			best_c = c
	_grip_err(best_a, best_b, target, best_c)
	if wrist:
		var wr := _skx() * _p("LeftHand")
		print("wrist (%.3f, %.3f, %.3f)" % [wr.x, wr.y, wr.z])
		print("const GRIP_HAND_D := Vector3(%.1f, %.1f, %.1f)" % [best_c.x, best_c.y, best_c.z])
	var s := _skx()
	var k := s * _p("LeftMiddleProximal")
	var el := s * _p("LeftLowerArm")
	var sh := s * _p("LeftUpperArm")
	print("strap posed at (%.3f, %.3f, %.3f)" % [target.x, target.y, target.z])
	print("knuckle off strap %.1f mm; elbow (%.3f, %.3f, %.3f), %.0f mm below the shoulder, %.0f mm out"
		% [k.distance_to(target) * 1000, el.x, el.y, el.z, (sh.y - el.y) * 1000, (el.x - sh.x) * 1000])
	print("const GRIP_UPPER := Vector3(%.1f, %.1f, %.1f)" % [best_a.x, best_a.y, best_a.z])
	print("const GRIP_LOWER := Vector3(%.1f, %.1f, %.1f)" % [best_b.x, best_b.y, best_b.z])


func _grip_err(a: Vector3, b: Vector3, target: Vector3, c := Vector3.INF) -> float:
	_pose_bone("LeftUpperArm", a)
	_pose_bone("LeftLowerArm", b)
	var s := _skx()
	var k := s * _p("LeftMiddleProximal")
	var el := s * _p("LeftLowerArm")
	var sh := s * _p("LeftUpperArm")
	if c != Vector3.INF:
		_pose_bone("LeftHand", c)
		k = s * _p("LeftMiddleProximal")
		var wr := s * _p("LeftHand")
		# The reference body (CHARACTER_ROUTE_D_PLUS.md §8.3): its arms are ~20 %
		# longer than the town body's, so a straight wrist on the strap at chest
		# height drives the elbow 30 cm behind her. In the reference the hand is
		# turned up at the wrist, fingers round the webbing, knuckles forward:
		# the wrist sits below the knuckle, which shortens the reach and lets the
		# elbow hang at her side, a little back and clear of the hoodie.
		var wrist_want := target + Vector3(0.012, -0.075, -0.010)
		var elbow_want_d := sh + Vector3(0.085, -0.255, -0.11)
		return (k.distance_to(target) + 0.5 * wr.distance_to(wrist_want)
			+ 0.4 * el.distance_to(elbow_want_d))
	# the elbow hangs: well below the shoulder, a little out to the side and
	# a little forward of the body line -- a person holding a strap, not
	# saluting it
	# (the first solve, without the forward term weighted, put it 128 mm behind
	# her back while the knuckle sat 0.4 mm from the strap)
	# A forearm cannot reach a strap high on the chest from an elbow in front
	# of the body; in the reference her elbow is down at her side by the lower
	# ribs, a little back. Not winged out behind her, which is where an
	# unweighted solve goes.
	var elbow_want := sh + Vector3(0.07, -0.23, -0.02)
	return k.distance_to(target) + 0.6 * el.distance_to(elbow_want)


## Solve the legs for the contrapposto the pelvis tilt implies. Rolling the
## pelvis swings both legs with it -- measured: the standing foot slid 198 mm
## out and the free foot's toes went 30 mm through the floor -- so the hip
## height, both thighs, the free knee and the free foot are solved together:
## standing foot flat under the hip, free foot a little forward and out, toes
## on the floor, heel just lifted.
const R_ANKLE := Vector3(-0.075, 0.083, 0.000)
const L_ANKLE := Vector3(0.120, 0.098, 0.060)


func _legs() -> void:
	var pose := Human.stand_key(0.0, true)
	# does the hip position axis do anything? (the first solve never moved it)
	for dy in [0.0, -0.02]:
		var p2 := pose.duplicate()
		p2["Hips@pos"] = Vector3(0, dy, 0)
		_apply(p2)
		print("  Hips@pos y %+.3f -> right ankle y %.4f" % [dy, (_skx() * _p("RightFoot")).y])
	var names := ["Hips@pos", "RightUpperLeg", "LeftUpperLeg", "LeftLowerLeg", "LeftFoot", "RightFoot"]
	# Joint random-direction steps, not one parameter at a time: lowering the
	# hips sinks the free foot until the free knee bends in the same move, so
	# coordinate descent sat at the start and never lowered them at all.
	var best := _leg_err(pose)
	var rng := RandomNumberGenerator.new()
	rng.seed = 20261001
	var scale := 6.0
	for it in 6000:
		var trial := pose.duplicate()
		for n: String in names:
			var v: Vector3 = trial[n]
			if n == "Hips@pos":
				v.y += rng.randfn() * scale * 0.002
			elif n in ["LeftLowerLeg", "LeftFoot", "RightFoot"]:
				v.x += rng.randfn() * scale
			else:
				v += Vector3(rng.randfn(), rng.randfn(), rng.randfn()) * scale
			trial[n] = v
		var e := _leg_err(trial)
		if e < best:
			best = e
			pose = trial
		if it % 1500 == 1499:
			scale *= 0.4
	_apply(pose)
	var s := _skx()
	print("legs solved, error %.1f mm" % [best * 1000])
	for n: String in names:
		var v: Vector3 = pose[n]
		print("  \"%s\": Vector3(%.3f, %.3f, %.3f)," % [n, v.x, v.y, v.z] if n == "Hips@pos"
			else "  \"%s\": Vector3(%.1f, %.1f, %.1f)," % [n, v.x, v.y, v.z])
	print("  ankles L (%.3f, %.3f, %.3f)  R (%.3f, %.3f, %.3f)   toes L y %.3f  R y %.3f" % [
		(s * _p("LeftFoot")).x, (s * _p("LeftFoot")).y, (s * _p("LeftFoot")).z,
		(s * _p("RightFoot")).x, (s * _p("RightFoot")).y, (s * _p("RightFoot")).z,
		(s * _p("LeftToes")).y, (s * _p("RightToes")).y])


func _leg_err(pose: Dictionary) -> float:
	_apply(pose)
	var s := _skx()
	var la := s * _p("LeftFoot")
	var ra := s * _p("RightFoot")
	var lt := s * _p("LeftToes")
	var rt := s * _p("RightToes")
	return (ra.distance_to(R_ANKLE) + la.distance_to(L_ANKLE)
		+ absf(rt.y + 0.001) + absf(lt.y + 0.001)
		# no bending the standing knee to get there
		+ 0.0005 * absf((pose["RightUpperLeg"] as Vector3).y))


func _dir(v: Vector3) -> String:
	var parts := []
	if absf(v.y) > 0.004: parts.append(("up %.0f" if v.y > 0 else "down %.0f") % [absf(v.y) * 1000])
	if absf(v.z) > 0.004: parts.append(("fwd %.0f" if v.z > 0 else "back %.0f") % [absf(v.z) * 1000])
	if absf(v.x) > 0.004: parts.append(("her-left %.0f" if v.x > 0 else "her-right %.0f") % [absf(v.x) * 1000])
	return ", ".join(parts) if parts else "-"


## The probe for each bone is the end of the limb it carries.
const SWEEP := {
	"Hips": "Head", "Spine": "Head", "Chest": "Head", "UpperChest": "Head",
	"Neck": "Head", "Head": "*face",
	"RightShoulder": "RightLowerArm", "RightUpperArm": "RightHand",
	"RightLowerArm": "RightHand", "RightHand": "RightMiddleDistal",
	"LeftShoulder": "LeftLowerArm", "LeftUpperArm": "LeftHand", "LeftLowerArm": "LeftHand",
	"LeftHand": "LeftMiddleDistal", "LeftIndexProximal": "LeftIndexDistal",
	"LeftMiddleProximal": "LeftMiddleDistal", "LeftThumbProximal": "LeftThumbDistal",
	"LeftUpperLeg": "LeftFoot", "LeftLowerLeg": "LeftFoot", "LeftFoot": "LeftToes",
	"RightUpperLeg": "RightFoot", "RightLowerLeg": "RightFoot",
}


func _probe(name: String) -> Vector3:
	if name == "*face":
		# a point 0.15 m in front of the face, carried by the head bone
		var hb := _g(sk.find_bone("Head"))
		var rest := Transform3D.IDENTITY
		var b := sk.find_bone("Head")
		while b != -1:
			rest = sk.get_bone_rest(b) * rest
			b = sk.get_bone_parent(b)
		return hb * (rest.affine_inverse() * (rest.origin + Vector3(0, 0, 0.15)))
	return _p(name)


func _sweep() -> void:
	var non_identity := []
	for i in sk.get_bone_count():
		if not sk.get_bone_rest(i).basis.get_rotation_quaternion().is_equal_approx(Quaternion.IDENTITY):
			non_identity.append(sk.get_bone_name(i))
	print("rest rotations that are not identity: ", non_identity)
	print("=== axis sweep: +20 deg about each local axis, from rest ===")
	for bone: String in SWEEP:
		if sk.find_bone(bone) < 0:
			print("%-15s (not in skeleton)" % bone)
			continue
		for axis in 3:
			_reset()
			var base := _probe(SWEEP[bone])
			var e := Vector3.ZERO
			e[axis] = 20.0
			var rq := sk.get_bone_rest(sk.find_bone(bone)).basis.get_rotation_quaternion()
			sk.set_bone_pose_rotation(sk.find_bone(bone), rq * Human.euler_q(e))
			var moved := _probe(SWEEP[bone]) - base
			print("%-15s %s+20  %s moves %s" % [bone, "XYZ"[axis], SWEEP[bone], _dir(moved)])
	_reset()


func _eval(pose: Dictionary) -> void:
	_apply(pose)
	var s := _skx()
	var lf := s * _p("LeftFoot")
	var rf := s * _p("RightFoot")
	var lt := s * _p("LeftToes")
	var rt := s * _p("RightToes")
	var hips := s * _p("Hips")
	var head := s * _p("Head")
	print("=== stand pose, first key ===")
	print("ankles   L y %.3f  R y %.3f   toes L y %.3f  R y %.3f" % [lf.y, rf.y, lt.y, rt.y])
	print("feet x   L %+.3f  R %+.3f   hips x %+.3f   head x %+.3f" % [lf.x, rf.x, hips.x, head.x])
	var knuckle := s * _p("LeftMiddleProximal")
	var strap := _strap_posed()
	print("grip     knuckle (%.3f, %.3f, %.3f)  strap (%.3f, %.3f, %.3f)  off %.0f mm"
		% [knuckle.x, knuckle.y, knuckle.z, strap.x, strap.y, strap.z,
		knuckle.distance_to(strap) * 1000])
	var rh := s * _p("RightHand")
	var rleg := s * _p("RightUpperLeg")
	print("free hand (%.3f, %.3f, %.3f); %.0f mm out from the hip joint, %.0f mm below it"
		% [rh.x, rh.y, rh.z, (rleg.x - rh.x) * 1000, (rleg.y - rh.y) * 1000])
	var face := s * _probe("*face")
	var fwd := (face - head)
	print("face points yaw %+.0f deg (+ = her left), pitch %+.0f deg (+ = up)"
		% [rad_to_deg(atan2(fwd.x, fwd.z)), rad_to_deg(atan2(fwd.y, Vector2(fwd.x, fwd.z).length()))])
