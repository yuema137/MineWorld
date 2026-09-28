## The rigged human body: CC0 geometry, MineWorld materials, Quaternius motion.
##
## This replaces the capsule mannequin that `npc.gd` used to build from
## primitives. `npc.gd` still owns *behaviour* -- where a person walks, which
## pose they hold -- and this file owns the *body*. The split matters because
## the body is now an imported asset with a licence trail
## (`docs/CHARACTER_ASSET_AUDIT.md`) while the behaviour is still ours.
##
## FACING: the character's rest pose faces its own local **+Z**, which is the
## convention `npc.gd` already steers by and the same one Godot's
## `SkeletonProfileHumanoid` defines for a humanoid reference pose. Nothing had
## to change for the swap, and `--drive` still prints world-space facing so a
## regression here is one line of output rather than a squint at a screenshot.
##
## SCALE: the authored figure measures 1.7688 m. An instance is scaled by
## `height / 1.7688`, which is the rule `docs/HUMANOID_PROFILE.md` states and
## the reason `height_mm` in the server means something visible here.
##
## Materials are re-authored rather than imported: the GLB ships bare materials
## and the textures are separate CC0 files. That is also what `DEP-8`'s
## coherence procedure asks for -- material authority stays with the project, so
## a borrowed asset stops announcing that it was borrowed.
class_name Human
extends Node3D

const SRC := "res://assets/characters/vitruvian/vitruvian.glb"
const CLIPS := "res://assets/characters/quaternius_ual.glb"
const TEX := "res://assets/characters/vitruvian/textures/"

## Measured from the baked GLB, not assumed. `tools/character_bake.py` prints it.
const CANONICAL_HEIGHT := 1.7688

## Ground speed each clip is authored at, **measured on this character** by
## `tools/measure_stride.gd`: it samples a foot relative to the hips across one
## cycle, takes the peak-to-peak travel as one stride, and a cycle holds two.
##
## These have to be measured and they have to be measured here, not on the
## animation rig. `Normalize Position Tracks` rescales *position* tracks; stride
## lives in the leg *rotations* applied to our limb lengths, so it survives
## normalisation untouched and differs from the Quaternius mannequin's
## (1.021 / 2.503 m/s on its own skeleton, 1.058 / 2.647 on ours).
##
## The first version of this file guessed 1.35 and 3.10. At the controller's
## real 1.45 m/s that guess ran the clip ~30% too slow, which is precisely the
## skating this constant exists to prevent.
const WALK_CLIP_MPS := 1.058
const JOG_CLIP_MPS := 2.647

static var _scene: PackedScene
static var _lib: AnimationLibrary
static var _mats: Dictionary = {}

var skeleton: Skeleton3D
var _tree: AnimationTree
var _posture: Posture
var _speed := 0.0
var _sitting := false


static func _tex(file: String, srgb: bool) -> Texture2D:
	var t := load(TEX + file) as Texture2D
	if t is CompressedTexture2D and not srgb:
		pass  # import flags carry the colour space; nothing to do at runtime
	return t


## Eyes and mouth are identical on everyone, so they are built once.
static func _shared() -> Dictionary:
	if not _mats.is_empty():
		return _mats
	var sclera := StandardMaterial3D.new()
	sclera.albedo_texture = _tex("sclera.jpg", true)
	sclera.roughness = 0.25
	_mats["sclera"] = sclera
	var iris := StandardMaterial3D.new()
	iris.albedo_texture = _tex("iris.jpg", true)
	iris.roughness = 0.12
	_mats["iris"] = iris
	var mouth := StandardMaterial3D.new()
	mouth.albedo_texture = _tex("mouth.jpg", true)
	mouth.roughness = 0.35
	_mats["mouth"] = mouth
	return _mats


## There is exactly one CC0 skin texture set, so a crowd built from it is a
## crowd of one person in different shirts. A per-instance albedo tint buys back
## some of that range: the map keeps all the photographic detail and the tint
## shifts its tone. It is a real limitation of a one-character asset, not a
## finished solution -- a second body texture set is the actual fix, and
## `docs/HUMANOID_PROFILE.md` records what person #2 would cost.
static func _skin(bc: String, nm: String, rough: String, tint: Color) -> StandardMaterial3D:
	var m := StandardMaterial3D.new()
	m.albedo_texture = _tex(bc, true)
	m.albedo_color = tint
	m.normal_enabled = true
	m.normal_texture = _tex(nm, false)
	m.normal_scale = 0.8
	m.roughness_texture = _tex(rough, false)
	m.roughness = 1.0
	m.metallic = 0.0
	# MakeHuman-family skins are diffuse-dominant and read waxy in Forward+
	# without this; it is the cheapest large step toward the reference's
	# material feel and costs nothing in the lighting rig.
	m.subsurf_scatter_enabled = true
	m.subsurf_scatter_strength = 0.28
	m.subsurf_scatter_skin_mode = true
	return m


## The groom's coverage lives in its own map, which StandardMaterial3D cannot
## sample -- see shaders/hair_card.gdshader.
static func _hair(tint: Color) -> ShaderMaterial:
	var m := ShaderMaterial.new()
	m.shader = load("res://shaders/hair_card.gdshader")
	m.set_shader_parameter("tex_diffuse", _tex("hair_bc.jpg", true))
	m.set_shader_parameter("tex_opacity", _tex("hair_opacity.png", false))
	m.set_shader_parameter("tint", tint)
	m.set_shader_parameter("cutoff", 0.42)
	m.set_shader_parameter("roughness_v", 0.55)
	return m


static func _cloth(c: Color, rough: float) -> StandardMaterial3D:
	var m := StandardMaterial3D.new()
	m.albedo_color = c
	m.roughness = rough
	m.metallic = 0.0
	m.normal_enabled = true
	m.normal_texture = _tex("fabric_n.jpg", false)
	# Fine and faint: a weave hint at conversation distance, not a quilted
	# pattern. The garment UVs are near 1:1 with metres, so the tiling is high.
	m.normal_scale = 0.12
	m.uv1_scale = Vector3(14, 14, 1)
	return m


## Shoes get no weave -- leather is not fabric, and the tiled normal read as
## camouflage on a foot-sized surface.
static func _plain(c: Color, rough: float) -> StandardMaterial3D:
	var m := StandardMaterial3D.new()
	m.albedo_color = c
	m.roughness = rough
	m.metallic = 0.0
	return m


## Colours come from `npc.gd`'s palettes so the crowd still reads as one town;
## `shoe` tints the foot surface, which is where upstream put the shoes.
## `hoodie` and `pack` are optional: pass a transparent colour for a person who
## is not wearing them. Only the reference character does, for now.
static func build(height_m: float, skin: Color, hair: Color,
		top: Color, legs: Color, shoe: Color,
		hoodie := Color(0, 0, 0, 0), pack := Color(0, 0, 0, 0)) -> Human:
	if _scene == null:
		_scene = load(SRC) as PackedScene
	var h := Human.new()
	h.name = "Human"
	var inst := _scene.instantiate()
	h.add_child(inst)
	h.scale = Vector3.ONE * (height_m / CANONICAL_HEIGHT)

	h.skeleton = inst.find_children("*", "Skeleton3D", true, false)[0] as Skeleton3D
	var m := _shared()
	# Bound by the GLB's own material names rather than by surface index, so
	# re-running the bake with a different surface order cannot silently paint
	# the shirt with skin. `tools/character_bake.py` carries the names through.
	var by_name := {
		"VitBody": _skin("body_bc.jpg", "body_n.jpg", "body_rough.jpg", skin),
		"VitShoes": _plain(shoe, 0.45),
		"VitPants": _cloth(legs, 0.85), "VitShirt": _cloth(top, 0.80),
		"VitSkin": _skin("face_bc.jpg", "face_n.jpg", "face_rough.jpg", skin),
		"VitMouth": m["mouth"],
		"VitSclera": m["sclera"], "VitIris": m["iris"], "VitHair": _hair(hair),
		"VitHoodie": _cloth(hoodie, 0.86),
	}
	for mi: MeshInstance3D in h.skeleton.find_children("*", "MeshInstance3D", true, false):
		if mi.name == "Hoodie" and hoodie.a <= 0.0:
			mi.visible = false
			continue
		for i in mi.mesh.get_surface_count():
			var src := mi.mesh.surface_get_material(i)
			var key := (src.resource_name if src else "").trim_suffix(".001")
			if by_name.has(key):
				mi.set_surface_override_material(i, by_name[key])
			else:
				push_warning("human.gd: no material for surface '%s'" % key)
		# The hair is alpha-scissored and self-shadows badly at grazing angles.
		if mi.name == "Hair":
			mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_DOUBLE_SIDED

	if pack.a > 0.0:
		h._backpack(pack)

	# Layered after the AnimationTree; see posture.gd for why it exists.
	h._posture = Posture.natural_stance()
	h.skeleton.add_child(h._posture)

	h._build_tree(inst)
	return h


## A backpack on one shoulder. Built from primitives and hung off the profile's
## UpperChest bone with a BoneAttachment3D, so it rides the spine and needs no
## skinning -- a rucksack is rigid anyway. The strap across the chest is most of
## what makes the reference silhouette recognisable, more than the bag itself.
## A backpack on the shoulders, built from primitives.
##
## Parented to the character rather than to a bone. A `BoneAttachment3D` on
## UpperChest is the textbook answer and it is what the first version did, but
## the bone's frame after retargeting is not character space and undoing it put
## the bag through the chest at an angle. A rucksack on a walking person barely
## moves relative to the torso, so the honest trade is fixed placement that is
## visibly right over rig-following that is visibly wrong. If the character ever
## needs to bend, this is the thing to revisit.
func _backpack(c: Color) -> void:
	var hold := Node3D.new()
	hold.name = "Pack"
	add_child(hold)

	var canvas := Mats.paint(c, 0.92)
	var webbing := Mats.paint(c.darkened(0.30), 0.88)
	var buckle := Mats.paint(Color(0.18, 0.18, 0.17), 0.45, 0.4)
	# the bag on the upper back, with a lid flap and a lower pocket so the
	# silhouette is not one plain box
	Build.box(hold, Vector3(0, 1.235, -0.185), Vector3(0.265, 0.34, 0.145), canvas)
	Build.box(hold, Vector3(0, 1.385, -0.185), Vector3(0.245, 0.10, 0.155),
		Mats.paint(c.lightened(0.05), 0.92))
	Build.box(hold, Vector3(0, 1.115, -0.205), Vector3(0.20, 0.11, 0.12),
		Mats.paint(c.darkened(0.14), 0.92))
	# straps over both shoulders and down the chest. The right-hand one is what
	# the reference character grips, and it carries a lot of the silhouette.
	for sx in [-1.0, 1.0]:
		Build.box(hold, Vector3(sx * 0.105, 1.445, -0.02), Vector3(0.065, 0.075, 0.28),
			webbing)
		Build.box(hold, Vector3(sx * 0.115, 1.29, 0.105), Vector3(0.06, 0.34, 0.05),
			webbing)
		Build.box(hold, Vector3(sx * 0.115, 1.135, 0.120), Vector3(0.055, 0.055, 0.035),
			buckle)


func _build_tree(inst: Node) -> void:
	if _lib == null:
		var clips := (load(CLIPS) as PackedScene).instantiate()
		var ap := clips.find_children("*", "AnimationPlayer", true, false)[0] as AnimationPlayer
		_lib = ap.get_animation_library(ap.get_animation_library_list()[0])
		clips.queue_free()

	var player := AnimationPlayer.new()
	player.name = "AnimationPlayer"
	inst.add_child(player)
	player.add_animation_library("", _lib)
	player.root_node = player.get_path_to(skeleton)

	# Idle -> Walk -> Jog on one axis, driven by measured ground speed. Root
	# motion stays off: the controller and the NPC paths move the body, exactly
	# as they did with the mannequin.
	var space := AnimationNodeBlendSpace1D.new()
	space.min_space = 0.0
	space.max_space = JOG_CLIP_MPS
	# Without this the two locomotion clips run on their own clocks (1.333 s and
	# 0.933 s), so a blend of them averages two footfall patterns that are out of
	# phase. The legs stop reaching and the feet skate -- measured at 49% of body
	# speed before this line existed.
	space.sync = true
	# Each clip sits at the speed it was authored for. That is what makes the
	# cadence distance-driven with no tuning constant: between two blend points
	# the blended stride interpolates exactly as the blend position does, so
	# playing at rate 1.0 covers exactly the ground the body is covering. Put a
	# clip at the wrong position and the whole band skates.
	for pair in [["Idle", 0.0], ["Walk", WALK_CLIP_MPS], ["Jog_Fwd", JOG_CLIP_MPS]]:
		var n := AnimationNodeAnimation.new()
		n.animation = pair[0]
		n.resource_name = pair[0]
		space.add_blend_point(n, pair[1], -1, pair[0])

	var scaler := AnimationNodeTimeScale.new()
	var tree_root := AnimationNodeBlendTree.new()
	tree_root.add_node("Locomotion", space, Vector2(0, 0))
	tree_root.add_node("Rate", scaler, Vector2(300, 0))
	tree_root.connect_node("Rate", 0, "Locomotion")
	tree_root.connect_node("output", 0, "Rate")

	_tree = AnimationTree.new()
	_tree.name = "AnimationTree"
	_tree.tree_root = tree_root
	# The node has to be in the tree before either path can be resolved.
	inst.add_child(_tree)
	_tree.anim_player = _tree.get_path_to(player)
	_tree.active = true
	set_gait(0.0)


## Drive the body from a ground speed measured elsewhere -- the same contract the
## capsule mannequin had, so `npc.gd` and `player.gd` did not have to change.
## The cadence rule, stated so it is testable: **animation phase advances with
## distance travelled, never with wall-clock time.** Standing still advances no
## phase; walking twice as fast takes steps twice as often, of the same length.
##
## The implementation falls out of putting each clip at its measured speed. A
## body scaled to `s` covers `s` times the ground per cycle, so the blend
## position is the speed expressed in the clip's own (unscaled) units, and the
## playback rate is then exactly 1.0. Above the fastest clip there is nothing
## left to blend toward, so the rate takes over.
func set_gait(speed_mps: float) -> void:
	if _tree == null:
		return
	var s := maxf(scale.y, 0.01)
	var want := maxf(speed_mps, 0.0) / s
	_speed = speed_mps

	# Below walking pace the blend is idle-to-walk, which scales the stride
	# amplitude with the blend weight -- speed and stride rise together, so the
	# clip plays at its own rate. Above it, the band is pinned to a single clip
	# and the rate carries the speed: cross-fading walk into jog blends two
	# different cadences and shortens the stride even with sync on, which is
	# skating by another route.
	var blend := want
	var rate := 1.0
	if want > WALK_CLIP_MPS:
		var jog := want >= (WALK_CLIP_MPS + JOG_CLIP_MPS) * 0.5
		blend = JOG_CLIP_MPS if jog else WALK_CLIP_MPS
		rate = want / blend
	_tree.set("parameters/Locomotion/blend_position", blend)
	_tree.set("parameters/Rate/scale", rate)


## World position of a foot. Used by `--drive` to measure foot sliding directly
## rather than inferring it from the numbers that were supposed to prevent it.
func foot_position(left: bool) -> Vector3:
	var i := skeleton.find_bone("LeftFoot" if left else "RightFoot")
	if i < 0:
		return global_position
	return skeleton.global_transform * skeleton.get_bone_global_pose(i).origin


## Play one clip straight, bypassing the blend space. Diagnostic only: playing
## `A_TPose` is how you tell a broken retarget from a clip you simply dislike.
func debug_clip(clip: String) -> void:
	if _tree:
		_tree.active = false
	var ap := _player()
	if ap:
		ap.play(clip)


func _player() -> AnimationPlayer:
	for c in get_child(0).get_children():
		if c is AnimationPlayer:
			return c
	return null


## Sit. The legs are replaced by `Posture.seated()` while the Idle clip keeps
## driving spine, arms and head, so a person on a bench still breathes. The clip
## set has Sitting_Idle_Loop in the paid tier only; this is the free-tier answer
## and it is written against profile bone names, so it works unchanged on any
## future character mapped to the profile.
func sit() -> void:
	_sitting = true
	set_gait(0.0)
	if _posture:
		_posture.tweaks = Posture.seated().tweaks
		_posture.absolute = Posture.seated().absolute
	_drop_to_ground.call_deferred()


func _ready() -> void:
	if _sitting:
		_drop_to_ground.call_deferred()


## Folding the legs does not move the hips, so a seated figure would hover with
## its feet underground. Drop the body until the lower foot rests on y = 0 and
## the hips land at bench height on their own -- measured from the posed
## skeleton, so it stays right if the pose or the character changes. It has to
## run after a frame: `get_bone_global_pose()` reports the rest pose for a
## skeleton that has never been processed.
func _drop_to_ground() -> void:
	var lowest := INF
	for bone in ["LeftFoot", "RightFoot"]:
		var i := skeleton.find_bone(bone)
		if i >= 0:
			lowest = minf(lowest, skeleton.get_bone_global_pose(i).origin.y)
	if lowest < INF:
		get_child(0).position.y = -lowest
