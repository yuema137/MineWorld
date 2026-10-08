@tool
extends SceneTree
## Solve the left arm's pose so the hand lands on the backpack strap.
##
## `CHARACTER_IDENTITY.md` §5 makes the grip a recognition cue -- "it fixes the
## pose and reads at any distance" -- and §6 lists it as one of the five things
## that decide whether a viewer says *that is the same character*.
##
## The numbers are searched, not guessed. A bone's local euler after retargeting
## is not something to reason about by eye: the rest pose is the profile's
## T-pose, the animation writes over it, and `Posture.absolute` replaces that
## again. So this poses the real skeleton, measures where the hand actually
## lands, and refines. The answer is pasted into `posture.gd` as a constant,
## which keeps the runtime free of a solver it would run once.
## In Godot's frame, not Blender's: +Y is up and the reference pose faces +Z, so
## the strap that sits at (x 0.088, y -0.150, z 1.255) in the modelling tools is
## at (0.088, 1.255, 0.150) here. Getting that wrong asks the solver for a point
## two metres away and it dutifully reports failure.
## The strap's own centre line at chest height. The bone that has to land here
## is the **knuckle**, not the wrist: `LeftHand` sits at the wrist, so solving
## for it puts the palm 55 mm past the webbing and the hand reads as hovering
## beside the strap rather than holding it.
const TARGET := Vector3(0.092, 1.250, 0.152)
const TOL := 0.012
## Two-bone IK has a whole circle of solutions -- the elbow can be anywhere on
## it -- and the unconstrained answer put the elbow up behind her shoulder.
## Asking for the elbow as well picks the one a person would actually use:
## upper arm hanging, forearm across the chest.
const ELBOW := Vector3(0.300, 1.010, 0.030)
const ELBOW_WEIGHT := 0.55

func _init() -> void:
	var h := Human.build(Human.CANONICAL_HEIGHT, NPC.REF_SKIN, NPC.REF_HAIR,
		NPC.REF_TEE, NPC.REF_JEANS, NPC.REF_SHOE, NPC.REF_HOODIE, NPC.REF_PACK)
	root.add_child(h)
	var sk := h.skeleton
	# `Posture` is a SkeletonModifier3D and it rewrites the arm bones on every
	# update, so a pose set here is gone again by the time the hand is measured.
	# It has to come off while the solver runs; the answer goes back into it.
	for c in sk.get_children():
		if c is Posture:
			sk.remove_child(c)
			c.queue_free()
	var up := sk.find_bone("LeftUpperArm")
	var lo := sk.find_bone("LeftLowerArm")
	var hand := sk.find_bone("LeftMiddleProximal")
	_elbow = lo
	var rest_up := sk.get_bone_rest(up)
	var rest_lo := sk.get_bone_rest(lo)
	print("bones: upper=%d lower=%d hand=%d" % [up, lo, hand])
	print("rest hand at ", _hand(sk, hand))

	# Random restart with local refinement. A grid over five angles is tens of
	# millions of poses; this finds the same answer in a few thousand and does
	# not care that the search space wraps.
	var rng := RandomNumberGenerator.new()
	rng.seed = 20260927
	var best_up := Vector3.ZERO
	var best_lo := Vector3.ZERO
	var best_err := 1e9
	for restart in range(40):
		var a := Vector3(rng.randf_range(-180, 180), rng.randf_range(-180, 180),
			rng.randf_range(-180, 180))
		var b := Vector3(rng.randf_range(-140, 140), 0.0, rng.randf_range(-140, 140))
		var err := _err(sk, h, up, lo, hand, rest_up, rest_lo, a, b)
		var step := 40.0
		while step > 0.25:
			var improved := false
			for axis in range(5):
				for sign in [1.0, -1.0]:
					var a2 := a
					var b2 := b
					match axis:
						0: a2.x += sign * step
						1: a2.y += sign * step
						2: a2.z += sign * step
						3: b2.x += sign * step
						4: b2.z += sign * step
					var e2 := _err(sk, h, up, lo, hand, rest_up, rest_lo, a2, b2)
					if e2 < err - 1e-6:
						err = e2
						a = a2
						b = b2
						improved = true
			if not improved:
				step *= 0.5
		if err < best_err:
			best_err = err
			best_up = a
			best_lo = b
	_pose(sk, up, rest_up, best_up)
	_pose(sk, lo, rest_lo, best_lo)
	var got := _hand(sk, hand)
	var got_elbow := _hand(sk, lo)
	print("elbow at %.4f, %.4f, %.4f  (wanted %.3f, %.3f, %.3f)"
		% [got_elbow.x, got_elbow.y, got_elbow.z, ELBOW.x, ELBOW.y, ELBOW.z])
	print("hand error %.1f mm" % [got.distance_to(TARGET) * 1000.0])
	print("LeftUpperArm  Vector3(%.1f, %.1f, %.1f)" % [_wrap(best_up.x), _wrap(best_up.y), _wrap(best_up.z)])
	print("LeftLowerArm  Vector3(%.1f, %.1f, %.1f)" % [_wrap(best_lo.x), _wrap(best_lo.y), _wrap(best_lo.z)])
	print("hand lands at %.4f, %.4f, %.4f   target %.4f, %.4f, %.4f   error %.1f mm"
		% [got.x, got.y, got.z, TARGET.x, TARGET.y, TARGET.z, best_err * 1000.0])
	var hand_err := got.distance_to(TARGET)
	print("RESULT ", "ok" if hand_err <= TOL else "TOO FAR")
	quit(0 if hand_err <= TOL else 1)


func _wrap(d: float) -> float:
	while d > 180.0:
		d -= 360.0
	while d < -180.0:
		d += 360.0
	return d


## The bone chain is multiplied out by hand.  `get_bone_global_pose()` reports
## the pose *after* the modifier stack, and the stack only runs inside the
## skeleton's own processing callback -- outside a frame it answers with the
## last value it computed, so a pose set from a script appears to do nothing and
## the solver sees a flat error surface.  `force_update_all_bone_transforms()`
## exists and does not help.
func _hand(sk: Skeleton3D, hand: int) -> Vector3:
	var t := Transform3D.IDENTITY
	var b := hand
	while b != -1:
		t = sk.get_bone_pose(b) * t
		b = sk.get_bone_parent(b)
	return sk.global_transform * t.origin


var _elbow := -1


func _err(sk: Skeleton3D, h: Human, up: int, lo: int, hand: int,
		rest_up: Transform3D, rest_lo: Transform3D, a: Vector3, b: Vector3) -> float:
	_pose(sk, up, rest_up, a)
	_pose(sk, lo, rest_lo, b)
	return (_hand(sk, hand).distance_to(TARGET)
		+ ELBOW_WEIGHT * _hand(sk, _elbow).distance_to(ELBOW))


## Exactly what `Posture._process_modification` does for an `absolute` entry:
## the euler *replaces* the bone's pose rotation and the rest rotation is not
## pre-multiplied. Solving against a different convention than the one the
## runtime applies is how the first answer came back with her hand over her
## face while the solver reported a 0.3 mm error.
func _pose(sk: Skeleton3D, b: int, _rest: Transform3D, deg: Vector3) -> void:
	sk.set_bone_pose_rotation(b, Quaternion(Basis.from_euler(Vector3(
		deg_to_rad(deg.x), deg_to_rad(deg.y), deg_to_rad(deg.z)))))
