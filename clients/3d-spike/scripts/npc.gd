## Ordinary people, built from primitives at real human proportions.
##
## Decision ARC-4 governs this file. 04_character_closeup.png is authoritative
## for body proportions, everyday clothing and how a person sits in the scene --
## and explicitly NOT for facial fidelity. So the budget goes into 1.72 m
## proportions, correct limb ratios, muted real clothing and believable posture,
## and stops well short of a face. A coherent mannequin at the right scale in
## the right clothes reads as "a person over there"; a detailed head from a
## mismatched asset pack would not.
##
## FACING CONVENTION, and read this before attaching a mannequin to anything:
## the body is authored facing its own local **+Z**, not Godot's -Z. That is the
## convention the walk code steers by (`rotation.y = atan2(facing.x, facing.z)`)
## and the one town.gd's hand-placed yaws are written against. A consumer that
## uses Godot's forward -- the player controller does -- has to turn the body
## 180 degrees when it attaches it, and player.gd says so at that line. Get it
## wrong and the character walks through the town backwards. `--drive` prints
## the resulting world-space facing and what each camera therefore sees, so the
## mistake is one line of output rather than a squint at a screenshot.
##
## This is presentation only. An NPC here knows where it walks. It knows
## nothing about what is allowed in the world.
class_name NPC
extends Node3D

## PUPPET is a body whose position and gait are driven from outside -- the
## player character (see `player.gd`), which is this same mannequin moved by the
## real controller instead of along a path.
enum Pose { WALK, STAND, SIT, LEAN, PUPPET }

const SKINS := [
	Color(0.78, 0.60, 0.47), Color(0.62, 0.44, 0.32), Color(0.88, 0.72, 0.60),
	Color(0.45, 0.31, 0.22), Color(0.82, 0.66, 0.52), Color(0.55, 0.38, 0.26),
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

var _hip: Node3D
var _leg_l: Node3D
var _leg_r: Node3D
var _arm_l: Node3D
var _arm_r: Node3D
var _t := 0.0
var _gait := 0.0
var _dir := 1.0


static func _limb(parent: Node3D, top: Vector3, length: float, r: float, mat: Material) -> Node3D:
	# pivot at the top of the limb so rotating the pivot swings it naturally
	var piv := Node3D.new()
	piv.position = top
	parent.add_child(piv)
	var cm := CapsuleMesh.new()
	cm.radius = r
	cm.height = maxf(length, r * 2.1)
	cm.radial_segments = 8
	cm.rings = 3
	var mi := MeshInstance3D.new()
	mi.mesh = cm
	mi.material_override = mat
	mi.position = Vector3(0, -length * 0.5, 0)
	piv.add_child(mi)
	return piv


static func _blob(parent: Node3D, pos: Vector3, size: Vector3, mat: Material) -> MeshInstance3D:
	var sm := SphereMesh.new()
	sm.radius = 0.5
	sm.height = 1.0
	sm.radial_segments = 12
	sm.rings = 7
	var mi := MeshInstance3D.new()
	mi.mesh = sm
	mi.material_override = mat
	mi.position = pos
	mi.scale = size
	parent.add_child(mi)
	return mi


static func make(rng: RandomNumberGenerator, p_pose: Pose, height := 0.0) -> NPC:
	var n := NPC.new()
	n.pose = p_pose
	var h: float = height if height > 0.0 else rng.randf_range(1.62, 1.83)
	var k := h / 1.72  # scale everything off a 1.72 m reference body

	var skin: Color = NPC.SKINS[rng.randi() % NPC.SKINS.size()]
	var hair: Color = NPC.HAIRS[rng.randi() % NPC.HAIRS.size()]
	var top: Color = NPC.TOPS[rng.randi() % NPC.TOPS.size()]
	var leg: Color = NPC.LEGS[rng.randi() % NPC.LEGS.size()]
	var shoe := Color(0.14, 0.13, 0.12) if rng.randf() < 0.6 else Color(0.86, 0.85, 0.82)

	var m_skin := Mats.paint(skin, 0.72)
	var m_hair := Mats.paint(hair, 0.85)
	var m_top := Mats.paint(top, 0.80)
	var m_leg := Mats.paint(leg, 0.84)
	var m_shoe := Mats.paint(shoe, 0.62)

	var slim := rng.randf_range(0.93, 1.09)

	# --- legs: hip at 0.92 m, thigh 0.46, shin 0.44 ------------------------
	n._hip = Node3D.new()
	n._hip.position = Vector3(0, 0.92 * k, 0)
	n.add_child(n._hip)
	for side in [-1.0, 1.0]:
		var thigh := NPC._limb(n._hip, Vector3(0.085 * k * side, 0, 0), 0.46 * k, 0.083 * k * slim, m_leg)
		var shin := NPC._limb(thigh, Vector3(0, -0.46 * k, 0), 0.44 * k, 0.067 * k * slim, m_leg)
		NPC._blob(shin, Vector3(0, -0.45 * k, 0.035 * k),
			Vector3(0.10, 0.075, 0.25) * k, m_shoe)
		if side < 0.0:
			n._leg_l = thigh
		else:
			n._leg_r = thigh

	# --- torso: hips to shoulders 0.50, jacket over it ---------------------
	var torso := Node3D.new()
	torso.position = Vector3(0, 0.92 * k, 0)
	n.add_child(torso)
	NPC._blob(torso, Vector3(0, 0.14 * k, 0), Vector3(0.33, 0.34, 0.21) * k * slim, m_leg)
	NPC._blob(torso, Vector3(0, 0.36 * k, 0), Vector3(0.38, 0.48, 0.235) * k * slim, m_top)
	NPC._blob(torso, Vector3(0, 0.50 * k, 0), Vector3(0.40, 0.20, 0.235) * k * slim, m_top)

	# --- arms: shoulder at 1.42 m, upper 0.31, fore 0.28 -------------------
	var sh_y := 0.50 * k
	var sh_x := 0.205 * k * slim
	for side2 in [-1.0, 1.0]:
		var upper := NPC._limb(torso, Vector3(sh_x * side2, sh_y, 0), 0.31 * k, 0.058 * k * slim, m_top)
		var fore := NPC._limb(upper, Vector3(0, -0.31 * k, 0), 0.28 * k, 0.049 * k * slim,
			m_top if rng.randf() < 0.45 else m_skin)
		NPC._blob(fore, Vector3(0, -0.29 * k, 0), Vector3(0.075, 0.11, 0.055) * k, m_skin)
		if side2 < 0.0:
			n._arm_l = upper
		else:
			n._arm_r = upper

	# --- head: neck 1.48, head centre ~1.60, 0.225 tall --------------------
	NPC._blob(torso, Vector3(0, 0.58 * k, 0), Vector3(0.10, 0.10, 0.10) * k, m_skin)
	var head := Node3D.new()
	head.position = Vector3(0, 0.70 * k, 0)
	torso.add_child(head)
	NPC._blob(head, Vector3.ZERO, Vector3(0.155, 0.205, 0.180) * k, m_skin)
	NPC._blob(head, Vector3(0, 0.005 * k, -0.012 * k), Vector3(0.168, 0.185, 0.178) * k, m_hair)
	# fringe/back only: leave the face clear of hair geometry
	NPC._blob(head, Vector3(0, 0.02 * k, -0.045 * k), Vector3(0.163, 0.20, 0.165) * k, m_hair)
	if rng.randf() < 0.35:
		NPC._blob(head, Vector3(0, -0.03 * k, -0.115 * k), Vector3(0.11, 0.17, 0.10) * k, m_hair)
	# minimal features -- eyes and brows only. Explicitly not a face model.
	var m_eye := Mats.paint(Color(0.10, 0.09, 0.09), 0.35)
	for ex in [-0.042, 0.042]:
		NPC._blob(head, Vector3(ex * k, 0.012 * k, 0.083 * k), Vector3(0.022, 0.016, 0.012) * k, m_eye)
		NPC._blob(head, Vector3(ex * k, 0.045 * k, 0.080 * k), Vector3(0.040, 0.010, 0.012) * k, m_hair)
	NPC._blob(head, Vector3(0, -0.018 * k, 0.086 * k), Vector3(0.028, 0.048, 0.030) * k, m_skin)

	if rng.randf() < 0.22:
		var cap_c := Color(0.30, 0.33, 0.30) if rng.randf() < 0.5 else Color(0.42, 0.24, 0.22)
		var m_cap := Mats.paint(cap_c, 0.80)
		NPC._blob(head, Vector3(0, 0.055 * k, -0.01 * k), Vector3(0.185, 0.145, 0.195) * k, m_cap)
		NPC._blob(head, Vector3(0, 0.020 * k, 0.095 * k), Vector3(0.175, 0.022, 0.13) * k, m_cap)

	if rng.randf() < 0.30:
		var m_bag := Mats.paint(Color(0.30, 0.28, 0.26).lerp(top, 0.2), 0.85)
		NPC._blob(torso, Vector3(0, 0.36 * k, -0.15 * k), Vector3(0.28, 0.36, 0.17) * k, m_bag)

	n._apply_pose(k)
	n.phase = rng.randf() * TAU
	return n


func _apply_pose(k: float) -> void:
	match pose:
		Pose.STAND:
			_arm_l.rotation.x = 0.06
			_arm_r.rotation.x = -0.06
			_arm_l.rotation.z = -0.06
			_arm_r.rotation.z = 0.06
		Pose.LEAN:
			_arm_l.rotation.x = -0.9
			_arm_r.rotation.x = -1.1
			_arm_l.get_child(1).rotation.x = -1.2 if _arm_l.get_child_count() > 1 else 0.0
		Pose.SIT:
			# hips drop to bench height; thighs forward, shins down
			position.y -= 0.0
			_hip.position.y = 0.46 * k
			for l in [_leg_l, _leg_r]:
				l.rotation.x = -1.50
				var shin: Node3D = l.get_child(1)
				shin.rotation.x = 1.45
			var torso2: Node3D = get_child(1)
			torso2.position.y = 0.46 * k
			torso2.rotation.x = 0.06
			_arm_l.rotation.x = -0.55
			_arm_r.rotation.x = -0.62
		Pose.WALK:
			pass


## Limb swing at gait phase `s`, scaled by `amount` (0 = standing still, 1 = a
## stroll). One implementation, shared by the walking NPCs and by the player
## character, which is the same mannequin.
func _swing(s: float, amount: float) -> void:
	var a := clampf(amount, 0.0, 1.35)
	_leg_l.rotation.x = sin(s) * 0.50 * a
	_leg_r.rotation.x = -sin(s) * 0.50 * a
	(_leg_l.get_child(1) as Node3D).rotation.x = maxf(0.0, -cos(s)) * 0.75 * a
	(_leg_r.get_child(1) as Node3D).rotation.x = maxf(0.0, cos(s)) * 0.75 * a
	_arm_l.rotation.x = -sin(s) * 0.40 * a
	_arm_r.rotation.x = sin(s) * 0.40 * a
	(_arm_l.get_child(1) as Node3D).rotation.x = (-0.25 - maxf(0.0, sin(s)) * 0.35) * a
	(_arm_r.get_child(1) as Node3D).rotation.x = (-0.25 - maxf(0.0, -sin(s)) * 0.35) * a


## Drive a PUPPET body from a ground speed measured somewhere else. Phase
## advances with distance covered, so the cadence follows the speed and a stroll
## and a jog do not need separate animation states.
func step(delta: float, speed_mps: float) -> void:
	_gait += delta * maxf(speed_mps, 0.0) * 2.3
	_swing(_gait + phase, speed_mps / 1.45)


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
		_swing(_t * speed * 3.3 + phase, 1.0)
	elif pose == Pose.STAND or pose == Pose.LEAN:
		_t += delta
		var b := sin(_t * 0.8 + phase) * 0.012
		_hip.position.y += 0.0
		rotation.y += sin(_t * 0.35 + phase) * 0.0006
		(get_child(1) as Node3D).rotation.z = b
