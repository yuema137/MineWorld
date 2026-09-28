## A constant corrective pose layered on top of whatever is animating.
##
## The retarget is faithful -- playing Quaternius's `A_TPose` on this character
## gives a textbook T-pose, which is the check that says the BoneMap, the axis
## overwrite and the silhouette fix are all correct. What it does not fix is
## *style*: the free Universal Animation Library is authored for an action game,
## so its idle stands with the feet wide and the arms held clear of the body,
## ready to swing a sword. A person waiting outside a cafe does not stand like
## that.
##
## So this is an art correction, not a bug fix, and it is deliberately small:
## a few degrees of adduction at the shoulders and hips. It is written against
## `SkeletonProfileHumanoid` bone names, which means it applies unchanged to any
## future character mapped to the profile -- which is the point of having
## standardised on one.
##
## It runs as a `SkeletonModifier3D` so it lands *after* the `AnimationTree` has
## written its poses; setting the same bones from `_process` would be overwritten
## on the next frame depending on order, which is the kind of bug that looks like
## a flicker and takes an afternoon.
class_name Posture
extends SkeletonModifier3D

## How far the gripping hand's fingers close, in degrees at the proximal joint.
const FINGER_CURL := 38.0

## Profile bone name -> local euler correction in degrees, post-multiplied onto
## whatever the animation produced.
var tweaks := {}

## Profile bone name -> local euler in degrees that *replaces* what the
## animation produced. Used to fold the legs of a seated person while the idle
## clip keeps driving the spine, arms and head, so a person on a bench still
## breathes and shifts instead of freezing into a mannequin.
var absolute := {}


static func natural_stance() -> Posture:
	var p := Posture.new()
	p.name = "Posture"
	p.tweaks = {
		# Arms in toward the body: the clip holds them about 12 degrees wide.
		"LeftUpperArm": Vector3(0, 0, 11.0),
		"RightUpperArm": Vector3(0, 0, -11.0),
		# A little elbow bend reads as relaxed rather than as a mannequin.
		"LeftLowerArm": Vector3(0, 0, 6.0),
		"RightLowerArm": Vector3(0, 0, -6.0),
		# Feet closer together than a combat stance.
		"LeftUpperLeg": Vector3(0, 0, -6.0),
		"RightUpperLeg": Vector3(0, 0, 6.0),
		# A slight glance off-axis. The reference character is looking away from
		# camera and it is a surprising amount of what makes her read as someone
		# rather than as a mannequin facing front. Kept small so it still looks
		# natural on a body that is walking.
		"Neck": Vector3(0, 7.0, 0),
		"Head": Vector3(0, 9.0, 0),
	}
	return p


## The reference character, who is holding her backpack strap.
##
## `CHARACTER_IDENTITY.md` §5 puts her **left** hand on the left strap at chest
## height, fingers over the webbing, and §6 lists that grip among the five
## things a viewer matches her by: "it fixes the pose and reads at any
## distance."
##
## The two arm angles are **solved, not authored** — `tools/solve_grip.gd`
## searches the real skeleton until the hand lands on the strap, with a second
## term that keeps the elbow hanging down and forward instead of behind the
## shoulder, which is where an unconstrained two-bone solution puts it. Re-run
## it if the strap moves; the hand lands within a millimetre of the webbing.
##
## They are `absolute`, not `tweaks`: a corrective offset would still let the
## walk cycle swing the arm, and a hand that swings through the strap it is
## supposed to be holding is worse than no grip at all. The right arm, the legs
## and the spine keep animating.
static func holding_strap() -> Posture:
	var p := natural_stance()
	p.tweaks.erase("LeftUpperArm")
	p.tweaks.erase("LeftLowerArm")
	p.absolute["LeftUpperArm"] = Vector3(5.4, 107.4, -38.5)
	p.absolute["LeftLowerArm"] = Vector3(113.7, 0.0, -15.7)
	# Fingers curled over the webbing, thumb behind it -- the contract is
	# specific about that and an open flat hand beside a strap reads as a hand
	# that happens to be there. Relative, because the clips barely move fingers.
	for f in ["Index", "Middle", "Ring", "Little"]:
		p.tweaks["Left%sProximal" % f] = Vector3(0, 0, FINGER_CURL)
		p.tweaks["Left%sIntermediate" % f] = Vector3(0, 0, FINGER_CURL * 1.25)
		p.tweaks["Left%sDistal" % f] = Vector3(0, 0, FINGER_CURL * 0.7)
	p.tweaks["LeftThumbProximal"] = Vector3(0, 0, -FINGER_CURL * 0.5)
	p.tweaks["LeftThumbDistal"] = Vector3(0, 0, -FINGER_CURL * 0.4)
	return p


## Seated: the legs are replaced outright, the arms keep a light correction so
## they hang rather than staying where an action-game idle holds them.
static func seated() -> Posture:
	var p := Posture.new()
	p.name = "Posture"
	p.absolute = {
		"LeftUpperLeg": Vector3(82, 0, -4.0), "RightUpperLeg": Vector3(82, 0, 4.0),
		"LeftLowerLeg": Vector3(78, 0, 0), "RightLowerLeg": Vector3(78, 0, 0),
	}
	p.tweaks = {
		"LeftUpperArm": Vector3(0, 0, 9.0), "RightUpperArm": Vector3(0, 0, -9.0),
		"LeftLowerArm": Vector3(0, 0, 10.0), "RightLowerArm": Vector3(0, 0, -10.0),
	}
	return p


static func _quat(e: Vector3) -> Quaternion:
	return Quaternion(Basis.from_euler(Vector3(
		deg_to_rad(e.x), deg_to_rad(e.y), deg_to_rad(e.z))))


func _process_modification() -> void:
	var sk := get_skeleton()
	if sk == null:
		return
	for bone: String in absolute:
		var i := sk.find_bone(bone)
		if i >= 0:
			sk.set_bone_pose_rotation(i, _quat(absolute[bone]))
	for bone: String in tweaks:
		var i := sk.find_bone(bone)
		if i < 0:
			continue
		var e: Vector3 = tweaks[bone]
		var q := Quaternion(Basis.from_euler(Vector3(
			deg_to_rad(e.x), deg_to_rad(e.y), deg_to_rad(e.z))))
		sk.set_bone_pose_rotation(i, sk.get_bone_pose_rotation(i) * q)
