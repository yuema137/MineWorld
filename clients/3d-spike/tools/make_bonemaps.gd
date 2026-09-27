## Generate and validate the committed BoneMap resources.
##
##   godot --headless --path clients/3d-spike --script res://tools/make_bonemaps.gd
##
## One table, two maps. Both rigs are expressed against Godot's
## SkeletonProfileHumanoid, which is what lets a Quaternius clip drive a
## CharMorph character without either knowing about the other -- see
## docs/HUMANOID_PROFILE.md. Generating rather than hand-writing the .tres means
## every mapped bone is checked to exist in the skeleton it claims to map, so a
## typo fails here instead of showing up as one limb that does not animate.
extends SceneTree

const CHARACTER := "res://assets/characters/vitruvian/vitruvian.glb"
const ANIMATION := "res://assets/characters/quaternius_ual.glb"

## profile bone -> [Vitruvian (CharMorph "mixamo" preset), Quaternius (Rigify DEF)]
## "" means deliberately unmapped.
##
## Thumbs: both source rigs give three thumb joints starting at what the profile
## calls the metacarpal, so thumb1/2/3 map to Metacarpal/Proximal/Distal. The
## profile's alternative reading (start at Proximal, leave Distal empty) is
## equally defensible; what matters is that BOTH maps make the same choice, and
## they do. Eyes and Jaw stay unmapped: neither rig has them.
const MAP := {
	"Root": ["", "root"],
	"Hips": ["mixamorig_Hips", "DEF-hips"],
	"Spine": ["mixamorig_Spine", "DEF-spine.001"],
	"Chest": ["mixamorig_Spine1", "DEF-spine.002"],
	"UpperChest": ["mixamorig_Spine2", "DEF-spine.003"],
	"Neck": ["mixamorig_Neck", "DEF-neck"],
	"Head": ["mixamorig_Head", "DEF-head"],
	"LeftEye": ["", ""], "RightEye": ["", ""], "Jaw": ["", ""],

	"LeftShoulder": ["mixamorig_LeftShoulder", "DEF-shoulder.L"],
	"LeftUpperArm": ["mixamorig_LeftArm", "DEF-upper_arm.L"],
	"LeftLowerArm": ["mixamorig_LeftForeArm", "DEF-forearm.L"],
	"LeftHand": ["mixamorig_LeftHand", "DEF-hand.L"],
	"LeftThumbMetacarpal": ["mixamorig_LeftHandThumb1", "DEF-thumb.01.L"],
	"LeftThumbProximal": ["mixamorig_LeftHandThumb2", "DEF-thumb.02.L"],
	"LeftThumbDistal": ["mixamorig_LeftHandThumb3", "DEF-thumb.03.L"],
	"LeftIndexProximal": ["mixamorig_LeftHandIndex1", "DEF-f_index.01.L"],
	"LeftIndexIntermediate": ["mixamorig_LeftHandIndex2", "DEF-f_index.02.L"],
	"LeftIndexDistal": ["mixamorig_LeftHandIndex3", "DEF-f_index.03.L"],
	"LeftMiddleProximal": ["mixamorig_LeftHandMiddle1", "DEF-f_middle.01.L"],
	"LeftMiddleIntermediate": ["mixamorig_LeftHandMiddle2", "DEF-f_middle.02.L"],
	"LeftMiddleDistal": ["mixamorig_LeftHandMiddle3", "DEF-f_middle.03.L"],
	"LeftRingProximal": ["mixamorig_LeftHandRing1", "DEF-f_ring.01.L"],
	"LeftRingIntermediate": ["mixamorig_LeftHandRing2", "DEF-f_ring.02.L"],
	"LeftRingDistal": ["mixamorig_LeftHandRing3", "DEF-f_ring.03.L"],
	"LeftLittleProximal": ["mixamorig_LeftHandPinky1", "DEF-f_pinky.01.L"],
	"LeftLittleIntermediate": ["mixamorig_LeftHandPinky2", "DEF-f_pinky.02.L"],
	"LeftLittleDistal": ["mixamorig_LeftHandPinky3", "DEF-f_pinky.03.L"],

	"RightShoulder": ["mixamorig_RightShoulder", "DEF-shoulder.R"],
	"RightUpperArm": ["mixamorig_RightArm", "DEF-upper_arm.R"],
	"RightLowerArm": ["mixamorig_RightForeArm", "DEF-forearm.R"],
	"RightHand": ["mixamorig_RightHand", "DEF-hand.R"],
	"RightThumbMetacarpal": ["mixamorig_RightHandThumb1", "DEF-thumb.01.R"],
	"RightThumbProximal": ["mixamorig_RightHandThumb2", "DEF-thumb.02.R"],
	"RightThumbDistal": ["mixamorig_RightHandThumb3", "DEF-thumb.03.R"],
	"RightIndexProximal": ["mixamorig_RightHandIndex1", "DEF-f_index.01.R"],
	"RightIndexIntermediate": ["mixamorig_RightHandIndex2", "DEF-f_index.02.R"],
	"RightIndexDistal": ["mixamorig_RightHandIndex3", "DEF-f_index.03.R"],
	"RightMiddleProximal": ["mixamorig_RightHandMiddle1", "DEF-f_middle.01.R"],
	"RightMiddleIntermediate": ["mixamorig_RightHandMiddle2", "DEF-f_middle.02.R"],
	"RightMiddleDistal": ["mixamorig_RightHandMiddle3", "DEF-f_middle.03.R"],
	"RightRingProximal": ["mixamorig_RightHandRing1", "DEF-f_ring.01.R"],
	"RightRingIntermediate": ["mixamorig_RightHandRing2", "DEF-f_ring.02.R"],
	"RightRingDistal": ["mixamorig_RightHandRing3", "DEF-f_ring.03.R"],
	"RightLittleProximal": ["mixamorig_RightHandPinky1", "DEF-f_pinky.01.R"],
	"RightLittleIntermediate": ["mixamorig_RightHandPinky2", "DEF-f_pinky.02.R"],
	"RightLittleDistal": ["mixamorig_RightHandPinky3", "DEF-f_pinky.03.R"],

	"LeftUpperLeg": ["mixamorig_LeftUpLeg", "DEF-thigh.L"],
	"LeftLowerLeg": ["mixamorig_LeftLeg", "DEF-shin.L"],
	"LeftFoot": ["mixamorig_LeftFoot", "DEF-foot.L"],
	"LeftToes": ["mixamorig_LeftToeBase", "DEF-toe.L"],
	"RightUpperLeg": ["mixamorig_RightUpLeg", "DEF-thigh.R"],
	"RightLowerLeg": ["mixamorig_RightLeg", "DEF-shin.R"],
	"RightFoot": ["mixamorig_RightFoot", "DEF-foot.R"],
	"RightToes": ["mixamorig_RightToeBase", "DEF-toe.R"],
}

## Above Godot's own 18. UpperChest parents both shoulders and the neck, so a rig
## without it needs its shoulders reparented at map time -- requiring it instead
## keeps every consumer simpler. Toes are what stop feet reading as planks.
const ALSO_REQUIRED := ["Chest", "UpperChest", "Neck", "LeftToes", "RightToes"]


func _bones_of(path: String) -> PackedStringArray:
	var ps := load(path) as PackedScene
	if ps == null:
		push_error("cannot load " + path)
		return PackedStringArray()
	var root := ps.instantiate()
	var out := PackedStringArray()
	for n in root.find_children("*", "Skeleton3D", true, false):
		var s := n as Skeleton3D
		for i in s.get_bone_count():
			out.append(s.get_bone_name(i))
	root.free()
	return out


func _build(col: int, bones: PackedStringArray, label: String, out_path: String) -> bool:
	var profile := SkeletonProfileHumanoid.new()
	var bm := BoneMap.new()
	bm.profile = profile
	var ok := true
	var mapped := 0
	var unmapped: Array[String] = []
	for i in profile.bone_size:
		var pname := profile.get_bone_name(i)
		if not MAP.has(pname):
			push_error("%s: profile bone %s missing from MAP" % [label, pname])
			ok = false
			continue
		var target: String = MAP[pname][col]
		if target == "":
			unmapped.append(pname)
			if profile.is_required(i) or pname in ALSO_REQUIRED:
				# Root is the documented exception: the profile declares it, the
				# CharMorph rig roots at Hips, and nothing needs it.
				if pname != "Root":
					push_error("%s: REQUIRED bone %s is unmapped" % [label, pname])
					ok = false
			continue
		if not bones.has(target):
			push_error("%s: %s -> '%s' but that bone is not in the skeleton" % [label, pname, target])
			ok = false
			continue
		bm.set_skeleton_bone_name(pname, target)
		mapped += 1
	var used := {}
	for i in profile.bone_size:
		var t: String = MAP[profile.get_bone_name(i)][col]
		if t == "":
			continue
		if used.has(t):
			push_error("%s: '%s' mapped twice" % [label, t])
			ok = false
		used[t] = true
	var unused := []
	for b in bones:
		if not used.has(b):
			unused.append(b)
	print("%s: %d/%d profile bones mapped, %d skeleton bones (%d unused: %s)"
		% [label, mapped, profile.bone_size, bones.size(), unused.size(), str(unused)])
	print("   unmapped profile bones: ", str(unmapped))
	if ok:
		var err := ResourceSaver.save(bm, out_path)
		if err != OK:
			push_error("save failed: %d" % err)
			ok = false
		else:
			print("   wrote ", out_path)
	return ok


func _init() -> void:
	var cb := _bones_of(CHARACTER)
	var ab := _bones_of(ANIMATION)
	var ok := _build(0, cb, "vitruvian", "res://assets/characters/vitruvian/vitruvian_bonemap.tres")
	ok = _build(1, ab, "quaternius", "res://assets/characters/quaternius_ual_bonemap.tres") and ok
	print("RESULT ", "ok" if ok else "FAILED")
	quit(0 if ok else 1)
