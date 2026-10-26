extends Node
## Measured on the *character*, not on the animation rig: stride comes from leg
## rotations applied to our limb lengths, so the Quaternius mannequin's numbers
## are not ours. Reported at scale 1.0; a shorter or taller instance scales
## linearly, which is what human.gd divides by.
##
## `-- --character <id>` measures a cast character instead, built from its
## manifest in the default pack exactly as the slice builds it (`SliceCast`,
## `Human.build_character`), at its own stature and instance scale 1: the
## numbers it prints are that manifest's `clips` (step-26 §4.5).
##
## Two readings per clip (PR S36-P4b, Q-S36-P4-5):
##
## - **stride**: the foot bone's peak-to-peak travel against the hips is one
##   stride and a cycle holds two, so the speed is 2 * p2p / length. That holds
##   only when each foot is planted for half the cycle; a jog's foot is planted
##   for about a third of it, so for a jog the stride reading is not the speed
##   the planted foot moves back at, and the foot slips.
## - **contact**: the speed the body must cover for its planted foot to stand
##   still. Per foot and frame, the contact point (the lowest of `GroundContact`'s
##   support points, the vertex S36-P2's and S36-P4's planted rule reads) and its
##   velocity against the body; the stance frames are those in which it is within
##   `floor.float.planted` (the tolerance file, I-S36-2) of its own lowest height
##   over the cycle; the speed is the median backward speed over the stance
##   frames of both feet. Printed with the planted share it gives under G-8's
##   drift rule (`foot_grounding.json` `planted_drift_share`).
##
## `walk_mps` is the stride reading (the walk is planted at it, S36-P4 §9.2);
## `jog_mps` is the contact reading. The last line is one JSON object with both
## readings of both clips, for whoever writes them into a manifest.
const PACK := "../../presentation/mineworld-default/3D"
const TOLERANCES := "../../tools/character_pipeline/contact_tolerances.json"
## Samples over one cycle.
const STEPS := 120


func _ready() -> void:
	# the asset, not the runtime IK (F-P4-2): the IK would plant a slipping foot
	Human.foot_grounding = false
	var h: Human
	var args := OS.get_cmdline_user_args()
	var at := args.find("--character")
	var who := "reference"
	if at >= 0 and at + 1 < args.size():
		var cast := SliceCast.load_pack(
			ProjectSettings.globalize_path("res://").path_join(PACK).simplify_path())
		var spec := cast.body_for(args[at + 1])
		if String(spec.get("kind", "")) != "cast":
			printerr("measure_stride: %s has no drawable cast body" % args[at + 1])
			get_tree().quit(1)
			return
		h = Human.build_character(spec["manifest"])
		who = args[at + 1]
		print("character  %s" % who)
	else:
		h = Human.build(Human.CANONICAL_HEIGHT, NPC.REF_SKIN, NPC.REF_HAIR,
			NPC.REF_TEE, NPC.REF_JEANS, NPC.REF_SHOE)
	if h == null:
		printerr("measure_stride: the body could not be built")
		get_tree().quit(1)
		return
	add_child(h)
	var float_m := _bound_mm("floor.float.planted") / 1000.0
	var share := float(FootGrounding.tunables()["planted_drift_share"])
	var sk := h.skeleton
	var ap: AnimationPlayer = null
	for c in h.get_child(0).get_children():
		if c is AnimationPlayer:
			ap = c
	var hips := sk.find_bone("Hips")
	var lf := sk.find_bone("LeftFoot")
	var rf := sk.find_bone("RightFoot")
	# the clip's own pose: GroundContact's lift would hold the lowest point on the floor through a jog's flight
	h._ground.active = false
	var out := {"body": who, "steps": STEPS}
	print("clip       len(s)  stride(m)  stride(m/s)  contact(m/s)  planted@contact  planted@stride")
	for name in ["Walk", "Jog_Fwd"]:
		h.debug_clip(name)
		ap.speed_scale = 0.0
		var a := ap.get_animation(name)
		var lo := [1e9, 1e9]
		var hi := [-1e9, -1e9]
		var frames: Array = []
		for i in STEPS:
			ap.seek(a.length * float(i) / STEPS, true)
			await get_tree().process_frame
			var hp := sk.get_bone_global_pose(hips).origin
			for k in 2:
				var fp := sk.get_bone_global_pose(lf if k == 0 else rf).origin - hp
				lo[k] = minf(lo[k], fp.z)
				hi[k] = maxf(hi[k], fp.z)
			var by := h._ground.points_by_side()
			frames.append([by["left"], by["right"]])
		var stride: float = ((hi[0] - lo[0]) + (hi[1] - lo[1])) * 0.5
		var stride_mps := 2.0 * stride / a.length
		var dt := a.length / STEPS
		if "--dump" in args:
			for k in frames.size():
				var row := "DUMP %s %3d" % [name, k]
				for side in 2:
					var pts: PackedVector3Array = frames[k][side]
					var v := _velocity(frames, k, side, dt)
					row += "  y %7.1f mm  v %6.2f %6.2f %6.2f" % [pts[_low(pts)].y * 1000.0, v.x, v.y, v.z]
				print(row)
		var contact := _contact_speed(frames, dt, float_m)
		var at_contact := _planted_share(frames, dt, contact, share)
		var at_stride := _planted_share(frames, dt, stride_mps, share)
		print("%-10s %6.3f  %8.3f  %10.3f  %12.3f  %15.3f  %14.3f" % [name, a.length, stride, stride_mps,
			contact, at_contact, at_stride])
		out[name] = {"length_s": snappedf(a.length, 0.001), "stride_mps": snappedf(stride_mps, 0.001),
			"contact_mps": snappedf(contact, 0.001), "planted_share_contact": snappedf(at_contact, 0.001),
			"planted_share_stride": snappedf(at_stride, 0.001)}
	print("MEASURE " + JSON.stringify(out))
	get_tree().quit()


## Per frame, per foot: the index of the lowest support point.
static func _low(pts: PackedVector3Array) -> int:
	var low := 0
	for i in pts.size():
		if pts[i].y < pts[low].y:
			low = i
	return low


## The backward velocity (+z is the body's forward) of the contact point in frame `k`, against the body, cyclic:
## the same vertex in frame `k - 1`.
static func _velocity(frames: Array, k: int, side: int, dt: float) -> Vector3:
	var now: PackedVector3Array = frames[k][side]
	var before: PackedVector3Array = frames[(k - 1 + frames.size()) % frames.size()][side]
	var low := _low(now)
	return (now[low] - before[low]) / dt


## The median backward speed of the contact point over both feet's stance frames (within `float_m` of that foot's
## lowest height over the cycle).
static func _contact_speed(frames: Array, dt: float, float_m: float) -> float:
	var speeds: Array[float] = []
	for side in 2:
		var floor_y := INF
		for f: Array in frames:
			var pts: PackedVector3Array = f[side]
			floor_y = minf(floor_y, pts[_low(pts)].y)
		for k in frames.size():
			var pts: PackedVector3Array = frames[k][side]
			if pts[_low(pts)].y <= floor_y + float_m:
				speeds.append(-_velocity(frames, k, side, dt).z)
	if speeds.is_empty():
		return 0.0
	speeds.sort()
	return speeds[speeds.size() / 2]


## The share of foot-frames planted under G-8's rule when the body covers `speed`: the contact point's drift against
## the ground, |velocity + speed * forward|, at most `share * speed`.
static func _planted_share(frames: Array, dt: float, speed: float, share: float) -> float:
	var n := 0
	for side in 2:
		for k in frames.size():
			if (_velocity(frames, k, side, dt) + Vector3(0, 0, speed)).length() <= share * speed:
				n += 1
	return float(n) / float(2 * frames.size())


static func _bound_mm(region: String) -> float:
	var path := ProjectSettings.globalize_path("res://").path_join(TOLERANCES).simplify_path()
	var doc: Variant = JSON.parse_string(FileAccess.get_file_as_string(path))
	return float(doc["regions"][region]["bound_mm"])
