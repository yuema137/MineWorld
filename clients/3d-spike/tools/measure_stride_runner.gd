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
const PACK := "../../presentation/mineworld-default/3D"


func _ready() -> void:
	var h: Human
	var args := OS.get_cmdline_user_args()
	var at := args.find("--character")
	if at >= 0 and at + 1 < args.size():
		var cast := SliceCast.load_pack(
			ProjectSettings.globalize_path("res://").path_join(PACK).simplify_path())
		var spec := cast.body_for(args[at + 1])
		if String(spec.get("kind", "")) != "cast":
			printerr("measure_stride: %s has no drawable cast body" % args[at + 1])
			get_tree().quit(1)
			return
		h = Human.build_character(spec["manifest"])
		print("character  %s" % args[at + 1])
	else:
		h = Human.build(Human.CANONICAL_HEIGHT, NPC.REF_SKIN, NPC.REF_HAIR,
			NPC.REF_TEE, NPC.REF_JEANS, NPC.REF_SHOE)
	add_child(h)
	var sk := h.skeleton
	var ap: AnimationPlayer = null
	for c in h.get_child(0).get_children():
		if c is AnimationPlayer:
			ap = c
	var hips := sk.find_bone("Hips")
	var lf := sk.find_bone("LeftFoot")
	var rf := sk.find_bone("RightFoot")
	print("clip       len(s)  stride(m)  ground(m/s)  @1.0 scale")
	for name in ["Walk", "Jog_Fwd"]:
		h.debug_clip(name)
		var a := ap.get_animation(name)
		var steps := 90
		var lo := [1e9, 1e9]
		var hi := [-1e9, -1e9]
		for i in steps:
			ap.seek(a.length * float(i) / steps, true)
			await get_tree().process_frame
			var hp := sk.get_bone_global_pose(hips).origin
			for k in 2:
				var fp := sk.get_bone_global_pose(lf if k == 0 else rf).origin - hp
				lo[k] = minf(lo[k], fp.z)
				hi[k] = maxf(hi[k], fp.z)
		var stride: float = ((hi[0] - lo[0]) + (hi[1] - lo[1])) * 0.5
		print("%-10s %6.3f  %8.3f  %8.3f" % [name, a.length, stride, 2.0 * stride / a.length])
	get_tree().quit()
