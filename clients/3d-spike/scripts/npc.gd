## Ordinary people: where they walk and how they stand.
##
## This file used to build the body too, out of capsules and spheres, because
## `DEP-8`'s research had found no CC0 library shipping clothed, rigged,
## realistically proportioned modern people. One does -- see
## `docs/CHARACTER_ASSET_AUDIT.md` -- so the body moved to `human.gd` and this
## file kept what it was always really about: paths, poses and gait.
##
## `ARC-4` still governs the result. `04_character_closeup.png` is authoritative
## for body proportions, everyday clothing and how a person sits in the scene,
## and explicitly NOT for facial fidelity. The CC0 character has a real face,
## which `ARC-4` permits but does not require; what `ARC-4` rules out is making
## that face a *standard*. It costs nothing here because the face arrived with
## the asset rather than being budgeted for.
##
## FACING CONVENTION, and read this before attaching a body to anything:
## the body is authored facing its own local **+Z**, not Godot's -Z. That is the
## convention the walk code steers by (`rotation.y = atan2(facing.x, facing.z)`)
## and the one town.gd's hand-placed yaws are written against. A consumer that
## uses Godot's forward -- the player controller does -- has to turn the body
## 180 degrees when it attaches it, and player.gd says so at that line. Get it
## wrong and the character walks through the town backwards. `--drive` prints
## the resulting world-space facing and what each camera therefore sees, so the
## mistake is one line of output rather than a squint at a screenshot.
##
## The convention survived the swap to a rigged character unchanged: Godot's
## `SkeletonProfileHumanoid` also defines its reference pose facing +Z.
##
## This is presentation only. An NPC here knows where it walks. It knows
## nothing about what is allowed in the world.
class_name NPC
extends Node3D

## PUPPET is a body whose position and gait are driven from outside -- the
## player character (see `player.gd`), which is this same body moved by the
## real controller instead of along a path.
enum Pose { WALK, STAND, SIT, LEAN, PUPPET }

## Skin is a *tint over one photographic albedo*, so these stay close to neutral:
## pushed further they read as paint rather than as people. The honest limit is
## that the CC0 asset ships one body texture set; see `human.gd`.
const SKINS := [
	Color(1.00, 0.96, 0.92), Color(0.82, 0.68, 0.56), Color(1.00, 0.90, 0.82),
	Color(0.63, 0.49, 0.39), Color(0.93, 0.84, 0.75), Color(0.74, 0.58, 0.45),
]
const HAIRS := [
	Color(0.12, 0.09, 0.07), Color(0.26, 0.16, 0.09), Color(0.42, 0.29, 0.15),
	Color(0.58, 0.46, 0.28), Color(0.20, 0.13, 0.10), Color(0.66, 0.64, 0.61),
]
const TOPS := [
	Color(0.46, 0.20, 0.19), Color(0.30, 0.35, 0.29), Color(0.85, 0.83, 0.77),
	Color(0.23, 0.29, 0.38), Color(0.55, 0.52, 0.47), Color(0.36, 0.30, 0.26),
	Color(0.62, 0.58, 0.44), Color(0.28, 0.32, 0.36),
]
const LEGS := [
	Color(0.27, 0.32, 0.42), Color(0.21, 0.24, 0.31), Color(0.35, 0.33, 0.30),
	Color(0.18, 0.20, 0.24), Color(0.44, 0.40, 0.34),
]

var pose := Pose.STAND
var path_a := Vector3.ZERO
var path_b := Vector3.ZERO
var speed := 1.3
var phase := 0.0

var body: Human
var _t := 0.0


## The person in `3D/references/04_character_closeup.png`, read off the plate:
## a dark red / brick open zip hoodie, a cream tee with a mountain graphic, mid
## blue jeans, a grey-green backpack, short tousled warm brown hair. `ARC-4`
## scopes facial *fidelity* below the reference; it does not scope identity
## down, so wardrobe and hair are matched rather than approximated.
const REF_SKIN := Color(1.0, 0.95, 0.90)
const REF_HAIR := Color(0.26, 0.16, 0.10)
const REF_TEE := Color(0.84, 0.81, 0.73)
const REF_HOODIE := Color(0.44, 0.15, 0.14)
const REF_JEANS := Color(0.36, 0.45, 0.58)
const REF_SHOE := Color(0.30, 0.27, 0.24)
const REF_PACK := Color(0.36, 0.38, 0.32)


static func make(rng: RandomNumberGenerator, p_pose: Pose, height := 0.0,
		reference := false) -> NPC:
	var n := NPC.new()
	n.pose = p_pose
	var h: float = height if height > 0.0 else rng.randf_range(1.62, 1.83)
	if reference:
		n.body = Human.build(h, REF_SKIN, REF_HAIR, REF_TEE, REF_JEANS, REF_SHOE,
			REF_HOODIE, REF_PACK)
	else:
		n.body = Human.build(
			h,
			NPC.SKINS[rng.randi() % NPC.SKINS.size()],
			NPC.HAIRS[rng.randi() % NPC.HAIRS.size()],
			NPC.TOPS[rng.randi() % NPC.TOPS.size()],
			NPC.LEGS[rng.randi() % NPC.LEGS.size()],
			Color(0.14, 0.13, 0.12) if rng.randf() < 0.6 else Color(0.86, 0.85, 0.82))
	n.add_child(n.body)
	n.phase = rng.randf() * TAU
	# Idle and Walk are one blend space, so a standing person is simply a person
	# at zero ground speed. Giving each body a different start offset stops a
	# row of NPCs breathing in unison.
	n._t = n.phase
	if p_pose == Pose.SIT:
		n.body.sit()
	else:
		n.body.set_gait(0.0)
	return n


## Drive a PUPPET body from a ground speed measured somewhere else. Unchanged
## contract from the mannequin version: the caller still owns the movement and
## this only decides what the legs do about it.
func step(delta: float, speed_mps: float) -> void:
	body.set_gait(speed_mps)


func _process(delta: float) -> void:
	if pose == Pose.WALK:
		_t += delta
		var seg := path_b - path_a
		var len_seg := seg.length()
		if len_seg < 0.1:
			return
		var u := fmod(_t * speed, len_seg * 2.0)
		var back := u > len_seg
		var f: float = (len_seg * 2.0 - u) if back else u
		var p := path_a + seg.normalized() * f
		global_position = Vector3(p.x, global_position.y, p.z)
		var facing := -seg.normalized() if back else seg.normalized()
		rotation.y = atan2(facing.x, facing.z)
		body.set_gait(speed)
	elif pose == Pose.STAND or pose == Pose.LEAN:
		_t += delta
		# The Idle clip carries the weight shift now, so all that is left here is
		# a slow drift in heading -- people standing around do not hold a bearing.
		rotation.y += sin(_t * 0.35) * 0.0006
