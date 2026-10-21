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
## SCALE: the authored figure measures 1.7670 m (sole to crown; the gathered
## hair reaches 1.8026 and is deliberately not counted -- see character_model.py). An instance is scaled by
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
## The route D+ candidate for the default character: one Meshy generation,
## made game-ready and rigged on the same 52-joint skeleton, so the bone map,
## the clips and this file's posing apply unchanged
## (docs/references/CHARACTER_ROUTE_D_PLUS.md). Only the player is built from
## it; the townspeople keep `SRC`.
const SRC_D := "res://assets/characters/meshy_d/meshy_d.glb"
## Its one material, which keeps its own baked texture.
const MESHY_MATERIAL := "MW_Meshy"
## The standing loop every body plays: the 100STYLE `Neutral` idle (CC BY 4.0, `DEP-146`, `LICENSE.100style.txt`),
## retargeted at import through the same bone map and humanoid profile as every body (PR S26-c-pose).
const STAND_LIB := "res://assets/characters/stand_lib.glb"
const CLIPS := "res://assets/characters/quaternius_ual.glb"
const TEX := "res://assets/characters/vitruvian/textures/"

## Which body a person is built from. `TOWN` is the CharMorph body every
## townsperson shares; `REFERENCE` is the default character's own slot.
enum Body { TOWN, REFERENCE }

## Measured from the baked GLB, not assumed. `tools/blender/character_model.py` prints it.
## It moved from 1.7799 when the body was re-baked through CharMorph's Ultra
## Feminine morph and the shoes stopped being a swept tube.
const CANONICAL_HEIGHT := 1.7670

## Ground speed each clip is authored at, **measured on this character** by
## `tools/measure_stride.gd`: it samples a foot relative to the hips across one
## cycle, takes the peak-to-peak travel as one stride, and a cycle holds two.
##
## These have to be measured and they have to be measured here, not on the
## animation rig. `Normalize Position Tracks` rescales *position* tracks; stride
## lives in the leg *rotations* applied to our limb lengths, so it survives
## normalisation untouched and differs from the Quaternius mannequin's
## (1.021 / 2.503 m/s on its own skeleton, 1.063 / 2.660 on ours).
##
## The first version of this file guessed 1.35 and 3.10. At the controller's
## real 1.45 m/s that guess ran the clip ~30% too slow, which is precisely the
## skating this constant exists to prevent.
const WALK_CLIP_MPS := 1.063
const JOG_CLIP_MPS := 2.660

## Body -> PackedScene, and Body -> AnimationLibrary.
## Whether bodies built from now on get runtime foot grounding (PR S36-P4). The pose-sheet probe builds them
## without it, so that S36-P2's G-4w judges the asset and not the IK that would hide its defects (F-P4-2).
static var foot_grounding := true
static var _scenes: Dictionary = {}
static var _libs: Dictionary = {}
static var _mats: Dictionary = {}

var skeleton: Skeleton3D
var _tree: AnimationTree
var _posture: Posture
## Keeps the soles on the floor (`ground_contact.gd`); stood down while seated.
var _ground: GroundContact
## Each planted foot on its own ground (`foot_grounding.gd`, PR S36-P4); null when [member foot_grounding] was off.
var _feet: FootGrounding
var _speed := 0.0
var _sitting := false
var _seat_y := 0.45
## The imported GLB instance, held typed: `get_child(0)` is a bare Node and
## reaching through it for `.position` silently loses the type, which once
## failed compilation and degraded the whole scene.
var _inst: Node3D
var _body := Body.TOWN
## The ground speed each locomotion clip is placed at in the blend space. The town and reference bodies
## keep the measured constants above; a cast character brings its own, measured on its own body
## (`build_character`, step-26 §4.5).
var _walk_mps := WALK_CLIP_MPS
var _jog_mps := JOG_CLIP_MPS

## Blinking. The rig has no lid bones, so the lids close through the body
## mesh's `Blink` shape key (CharMorph's L3 Eyes_Closed, baked by
## `character_model.py`). Driven here rather than keyed in `Stand`, so a person
## blinks while walking as well as standing; each person's rhythm is their own.
var _face: MeshInstance3D
var _blink_idx := -1
var _blink_t := 0.0
var _blink_next := 3.0
var _blink_double := false
var _blink_rng := RandomNumberGenerator.new()
## Diagnostic: hold the lids at this weight (>= 0) instead of blinking.
var blink_hold := -1.0
## The shape key's weight at the bottom of a blink. Past 1.0 because the
## export pushes the iris 5 mm proud of the cornea (it rendered as a blank
## ball otherwise), and at 1.0 CharMorph's lids stopped just short of it:
## a closed eye with a slit of iris showing.
const BLINK_PEAK := 1.3


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
	sclera.roughness = 0.46
	_mats["sclera"] = sclera
	var iris := StandardMaterial3D.new()
	iris.albedo_texture = _tex("iris.jpg", true)
	# Not glossy: the iris sits *behind* the cornea, and the cornea's highlight
	# is not the iris's to carry.  At 0.12 a brown iris mirrored the sky and
	# rendered silver-grey at every portrait framing.
	# Matte and barely specular: in shade at 0.5 the sky's reflection turned
	# the iris grey-blue in the three-quarter portrait.
	iris.roughness = 0.85
	iris.metallic_specular = 0.15
	# The CC0 iris map is amber; the reference's eyes are warm brown, and at
	# chest-up framing the amber read as yellow-gold.
	iris.albedo_color = Color(0.58, 0.40, 0.28)
	_mats["iris"] = iris
	# The pupil is its own 64-face disc in the CC0 mesh, sharing the sclera's UV
	# island; textured with the sclera map it reads as a second white spot.
	# And it is matte.  At roughness 0.10 the near-black disc was a mirror and
	# reflected the sky, so every portrait showed silver-grey eyes over a brown
	# iris.  It cannot simply be hidden: the iris mesh is a ring, and without
	# the disc the white sclera shows through its centre.
	var pupil := StandardMaterial3D.new()
	pupil.albedo_color = Color(0.03, 0.025, 0.02)
	pupil.roughness = 0.7
	_mats["pupil"] = pupil
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
	# Matte, not wet. The CC0 roughness map runs glossy across the T-zone, and
	# with the default specular every preview showed a wet sheen on the
	# forehead, nose and chin where the reference's skin is matte with a soft
	# glow. A constant, fairly rough surface and a low specular level give
	# skin's broad, dim highlight instead of a lacquer's. (`rough` is kept in
	# the signature: the map is still the right input once a material model
	# with a separate sheen lobe is used.)
	m.roughness = 0.66
	m.metallic = 0.0
	m.metallic_specular = 0.30
	# MakeHuman-family skins are diffuse-dominant and read waxy in Forward+
	# without subsurface scattering. Stronger than before, and with warm
	# transmittance, for the reference's "visible subsurface warmth at the ear,
	# the nose and the jaw edge where light passes through".
	m.subsurf_scatter_enabled = true
	m.subsurf_scatter_strength = 0.45
	m.subsurf_scatter_skin_mode = true
	m.subsurf_scatter_transmittance_enabled = true
	m.subsurf_scatter_transmittance_color = Color(0.95, 0.42, 0.28)
	m.subsurf_scatter_transmittance_depth = 0.12
	m.subsurf_scatter_transmittance_boost = 0.25
	return m


## The card atlas carries coverage in its alpha, which StandardMaterial3D cannot
## sample separately -- see shaders/hair_card.gdshader.
##
## One file feeds both slots. It is the strand atlas built by
## `tools/blender/hair_atlas.py --alphas` (since preview 4, cut from OwlishMedia's CC0
## strand maps; see presentation/mineworld-default/LICENSES/): eight columns of
## fine strands with transparent gaps,
## for the groomed cards of `tools/blender/hair_groom.py`. The previous atlas drew a
## dozen thick bars per lock, and a head of them read as stripes.
static func _hair(tint: Color, flip_back := true) -> ShaderMaterial:
	var m := ShaderMaterial.new()
	m.shader = Mats.shader("res://shaders/hair_card.gdshader")
	var atlas := _tex("hair_strands.png", true)
	m.set_shader_parameter("tex_diffuse", atlas)
	m.set_shader_parameter("tex_opacity", atlas)
	m.set_shader_parameter("tint", tint)
	# lower than preview 3's 1.45/1.35/1.20: with the CC0 strands and lit
	# back faces, sunlit loose locks read near-blonde
	m.set_shader_parameter("highlight", Color(tint.r * 1.30, tint.g * 1.20, tint.b * 1.08))
	m.set_shader_parameter("cutoff", 0.30)
	m.set_shader_parameter("roughness_v", 0.55)
	# The CC0 strand sheets carry brighter shading than our procedural atlas
	# (most strands near 1.0), and at gain 1.0 nearly every pixel took the
	# highlight colour: the hair read ginger. 0.72 keeps the caramel for the
	# brightest strands only.
	m.set_shader_parameter("diffuse_gain", 0.72)
	m.set_shader_parameter("root_shade", 0.72)
	m.set_shader_parameter("flip_back", flip_back)
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


## A printed garment: the albedo carries the artwork, so the tiling weave normal
## that `_cloth` applies would fight it and the UV scale must stay at 1:1.
## `tile` repeats a seamless material (denim) across a planar UV in metres.
static func _printed(file: String, tint: Color, rough: float, tile := 1.0) -> StandardMaterial3D:
	var m := StandardMaterial3D.new()
	m.albedo_texture = _tex(file, true)
	m.albedo_color = tint
	m.roughness = rough
	m.metallic = 0.0
	m.uv1_scale = Vector3(tile, tile, 1)
	return m


## The generated body's one material: its own texture, our surface response.
## Diagnostic for the hairline flecks (CHARACTER_ROUTE_D_PLUS.md §8.4): the
## imported material is glTF's default dielectric at roughness 0.5.
static func _meshy(src: Material) -> Material:
	var m := (src as StandardMaterial3D).duplicate() as StandardMaterial3D
	print("human.gd: MW_Meshy imported roughness %.2f metallic %.2f specular %.2f" % [
		m.roughness, m.metallic, m.metallic_specular])
	m.roughness = 0.9
	m.metallic = 0.0
	m.metallic_specular = 0.25
	return m


## Shoes get no weave -- leather is not fabric, and the tiled normal read as
## camouflage on a foot-sized surface.
static func _plain(c: Color, rough: float, spec := 0.5) -> StandardMaterial3D:
	var m := StandardMaterial3D.new()
	m.albedo_color = c
	m.roughness = rough
	m.metallic = 0.0
	m.specular_mode = BaseMaterial3D.SPECULAR_SCHLICK_GGX
	m.roughness = rough
	m.albedo_color = c
	m.metallic_specular = spec
	return m


## Colours come from `npc.gd`'s palettes so the crowd still reads as one town;
## `shoe` tints the foot surface, which is where upstream put the shoes.
## `hoodie` and `pack` are optional: pass a transparent colour for a person who
## is not wearing them. Only the reference character does, for now.
static func build(height_m: float, skin: Color, hair: Color,
		top: Color, legs: Color, shoe: Color,
		hoodie := Color(0, 0, 0, 0), pack := Color(0, 0, 0, 0), body := Body.TOWN) -> Human:
	if not _scenes.has(body):
		_scenes[body] = load(SRC_D if body == Body.REFERENCE else SRC) as PackedScene
	var h := Human.new()
	h.name = "Human"
	var inst := (_scenes[body] as PackedScene).instantiate() as Node3D
	h.add_child(inst)
	h._inst = inst
	h._body = body
	h.scale = Vector3.ONE * (height_m / CANONICAL_HEIGHT)

	h.skeleton = inst.find_children("*", "Skeleton3D", true, false)[0] as Skeleton3D
	var m := _shared()
	# Bound by the GLB's own material names rather than by surface index, so
	# re-running the bake with a different surface order cannot silently paint
	# the shirt with skin. `tools/character_bake.py` carries the names through.
	var by_name := {
		"MW_Body": _skin("body_bc.jpg", "body_n.jpg", "body_rough.jpg", skin),
		"MW_Face": _skin("face_bc.jpg", "face_n.jpg", "face_rough.jpg", skin),
		"MW_Mouth": m["mouth"],
		"MW_Sclera": m["sclera"], "MW_Iris": m["iris"], "MW_Pupil": m["pupil"],
		# the mountain-and-slogan print is the one unique object on the
		# character, so the tee's albedo is artwork rather than a flat colour
		"MW_Tee": _printed("tee_bc.jpg", top, 0.82),
		# the ribbed crew neck, a darker maroon-brown band than the tee body
		"MW_TeeRib": _cloth(Color(top.r * 0.46, top.g * 0.30, top.b * 0.27), 0.9),
		"MW_Jeans": _printed("denim_bc.jpg", legs, 0.86, 2.0),
		"MW_Denim_Trim": _printed("denim_bc.jpg", legs.darkened(0.10), 0.84, 2.0),
		"MW_Hoodie": _cloth(hoodie, 0.86),
		# the zip tape is a lighter woven strip, not metal: the teeth are below
		# the resolution this character is ever seen at
		"MW_Zip": _plain(Color(0.66, 0.64, 0.60), 0.55),
		"MW_Cord": _plain(Color(0.88, 0.84, 0.74), 0.72),
		"MW_Shoe": _plain(shoe, 0.55), "MW_Sole": _plain(shoe.darkened(0.35), 0.72),
		# the rucksack: grey-green canvas, grey-green webbing over a dark navy
		# lower section with a visible adjuster, as the reference shows
		"MW_Pack": _cloth(pack, 0.92),
		# the padded straps are the bag's own canvas; lightened, they read
		# slate-blue in the sky light of the chest-up frames
		"MW_Webbing": _cloth(pack, 0.88),
		"MW_StrapLow": _plain(Color(0.10, 0.11, 0.14), 0.80),
		"MW_Buckle": _plain(Color(0.16, 0.16, 0.15), 0.42, 0.35),
		"MW_Hair": _hair(hair),
		# darker and cooler than the hair, as the reference's brows are
		"MW_Brow": _hair(hair.darkened(0.42), false),
		# The opaque shell under the cards; alpha-scissored hair always leaks and
		# this is what stops scalp showing between strands. Only slightly darker
		# than the strands: at 45% darker its edge read as a black headband
		# across the forehead wherever the cards were thin.
		# 8% darker, not 18%: under the CC0 strands' wider gaps the darker cap
		# showed through as dark patches on the crown.
		"MW_HairCap": _plain(hair.darkened(0.08), 0.68),
	}
	for mi: MeshInstance3D in h.skeleton.find_children("*", "MeshInstance3D", true, false):
		if mi.name == "Hoodie" and hoodie.a <= 0.0:
			mi.visible = false
			continue
		if mi.name == "Pack" and pack.a <= 0.0:
			mi.visible = false
			continue
		for i in mi.mesh.get_surface_count():
			var src := mi.mesh.surface_get_material(i)
			var key := (src.resource_name if src else "").trim_suffix(".001")
			if by_name.has(key):
				mi.set_surface_override_material(i, by_name[key])
			elif key == MESHY_MATERIAL:
				# The generated body carries its own baked base colour; the
				# palette arguments do not apply to it. Its surface response is
				# ours: hair, skin and cotton are all rough and barely specular.
				mi.set_surface_override_material(i, _meshy(src))
			else:
				push_warning("human.gd: no material for surface '%s'" % key)
		if mi.name == "Body" and mi.find_blend_shape_by_name("Blink") >= 0:
			h._face = mi
			h._blink_idx = mi.find_blend_shape_by_name("Blink")
		# The hair is alpha-scissored and self-shadows badly at grazing angles.
		if mi.name == "Hair":
			mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_DOUBLE_SIDED

	# Layered after the AnimationTree; see posture.gd for why it exists. The
	# reference character also holds her backpack strap, which is a pose and not
	# a prop: the hand has to be on the webbing, so it is solved and locked.
	h._posture = Posture.holding_strap(body) if pack.a > 0.0 else Posture.natural_stance()
	h.skeleton.add_child(h._posture)
	h._stand_on_soles(inst)

	h._build_tree(inst, pack.a > 0.0, body)
	return h


## A cast character, built from its manifest: `presentation/<pack>/3D/characters/cast/<id>/character.yaml`
## (step-26 §4.5, `ARC-93`), already read and checked by `SliceCast.body_for`. Nothing here is per person:
##
## - its own scene, **authored at its true stature and drawn at instance scale 1** (`SD-S26-3`). The
##   caller's height does not stretch it; the manifest's `stature_m` is the person's height.
## - its one material (`material`, `MW_Cast` from the pipeline) keeps its own baked base colour and takes
##   the manifest's `surface` response, exactly as `_meshy` gives the reference body its own.
## - the natural stance (`Posture.natural_stance`): no strap, no grip (`grip` is null for every cast member).
## - the clips placed at the manifest's `clips` speeds, measured on this body (`tools/measure_stride.gd`).
##
## The town and reference bodies are untouched by this path: they keep `build`, their constants and their
## cache entries (keyed by `Body`; a cast body's are keyed by its scene path).
static func build_character(m: Dictionary) -> Human:
	var path := String(m["scene"])
	if not _scenes.has(path):
		_scenes[path] = load(path) as PackedScene
	var h := Human.new()
	h.name = "Human"
	var inst := (_scenes[path] as PackedScene).instantiate() as Node3D
	h.add_child(inst)
	h._inst = inst
	h._body = Body.TOWN
	h.scale = Vector3.ONE
	var clips: Dictionary = m["clips"]
	h._walk_mps = float(clips["walk_mps"])
	h._jog_mps = float(clips["jog_mps"])
	h.skeleton = inst.find_children("*", "Skeleton3D", true, false)[0] as Skeleton3D
	var material := String(m["material"])
	var surface: Variant = m.get("surface")
	for mi: MeshInstance3D in h.skeleton.find_children("*", "MeshInstance3D", true, false):
		for i in mi.mesh.get_surface_count():
			var src := mi.mesh.surface_get_material(i)
			if src is StandardMaterial3D and src.resource_name == material:
				mi.set_surface_override_material(i, _cast_surface(src, surface))
			else:
				push_warning("human.gd: %s: no material for surface '%s'" % [
					m.get("id", path), src.resource_name if src else ""])
	h._posture = Posture.natural_stance()
	h.skeleton.add_child(h._posture)
	h._stand_on_soles(inst)
	h._build_tree(inst, false, Body.TOWN, path)
	return h


## The body's ground contact, from its own geometry (`ground_contact.gd`, PR S26-c-ground): the scene
## instance is raised by its rest sole's depth (the cast exports' soles are at y = -0.0177, pipeline
## standard G-4), and a `GroundContact` modifier, after `Posture`, keeps the lowest sole on the floor
## through every clip. Every body takes the same path; nothing here is per person.
func _stand_on_soles(inst: Node3D) -> void:
	var chain := Transform3D.IDENTITY
	var n: Node = skeleton
	while n != inst and n != null:
		chain = (n as Node3D).transform * chain
		n = n.get_parent()
	_ground = GroundContact.for_body(skeleton, inst.scene_file_path, chain)
	_ground.floor_node = self
	skeleton.add_child(_ground)
	inst.position.y = -_ground.sole_y
	if foot_grounding:
		_feet = FootGrounding.for_body(self, skeleton, _ground)


## A cast body's material: its own baked texture, the manifest's surface response (rough and barely
## specular by default, the values `_meshy` gives the reference body).
static func _cast_surface(src: StandardMaterial3D, surface: Variant) -> StandardMaterial3D:
	var s: Dictionary = surface if typeof(surface) == TYPE_DICTIONARY else {}
	var m := src.duplicate() as StandardMaterial3D
	m.roughness = float(s.get("roughness", 0.9))
	m.metallic = 0.0
	m.metallic_specular = float(s.get("specular", 0.25))
	return m


## `lib_key` names the clip library's cache entry: the `Body` for the town and reference bodies, the scene
## path for a cast character.
func _build_tree(inst: Node, _grip: bool, body: Body, lib_key: Variant = null) -> void:
	if lib_key == null:
		lib_key = body
	if not _libs.has(lib_key):
		var clips := (load(CLIPS) as PackedScene).instantiate()
		var ap := clips.find_children("*", "AnimationPlayer", true, false)[0] as AnimationPlayer
		# duplicated because the imported library is shared and read-only, and
		# a clip of our own has to go into it
		var lib := ap.get_animation_library(ap.get_animation_library_list()[0]).duplicate()
		lib.add_animation("Sit", _sit_clip())
		# the recorded standing loop; the protagonist's strap grip is `Posture`'s additive layer on her left arm
		lib.add_animation("Stand", _stand_clip())
		clips.queue_free()
		_libs[lib_key] = lib

	var player := AnimationPlayer.new()
	player.name = "AnimationPlayer"
	inst.add_child(player)
	player.add_animation_library("", _libs[lib_key])
	player.root_node = player.get_path_to(skeleton)

	# Idle -> Walk -> Jog on one axis, driven by measured ground speed. Root
	# motion stays off: the controller and the NPC paths move the body, exactly
	# as they did with the mannequin.
	var space := AnimationNodeBlendSpace1D.new()
	space.min_space = 0.0
	space.max_space = _jog_mps
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
	# Standing is the recorded idle (`stand_lib.glb`), not the UAL library's `Idle`: that one is authored
	# for an action game -- feet wide, arms held clear, head back -- and the operator called it stiff.
	for pair in [["Stand", 0.0], ["Walk", _walk_mps], ["Jog_Fwd", _jog_mps]]:
		var n := AnimationNodeAnimation.new()
		n.animation = pair[0]
		n.resource_name = pair[0]
		space.add_blend_point(n, pair[1], -1, pair[0])

	var scaler := AnimationNodeTimeScale.new()
	var sit_node := AnimationNodeAnimation.new()
	sit_node.animation = "Sit"
	# Sitting is a state of the tree, not a correction applied to its output.
	# The obvious design -- let the tree run and override the legs in a
	# SkeletonModifier3D -- cannot work here: the AnimationTree writes every
	# bone after the modifier, so the override was silently discarded and the
	# figure sat with its legs straight. Measured as a thigh euler of
	# (0, 0, -180) where (82, 0, -4) had been set. Fighting the evaluation
	# order loses; the pose has to arrive through the tree.
	var mode := AnimationNodeTransition.new()
	mode.input_count = 2
	mode.set_input_name(0, "move")
	mode.set_input_name(1, "sit")
	mode.xfade_time = 0.0

	var tree_root := AnimationNodeBlendTree.new()
	tree_root.add_node("Locomotion", space, Vector2(0, 0))
	tree_root.add_node("Rate", scaler, Vector2(300, 0))
	tree_root.add_node("Sit", sit_node, Vector2(300, 200))
	tree_root.add_node("Mode", mode, Vector2(600, 0))
	tree_root.connect_node("Rate", 0, "Locomotion")
	tree_root.connect_node("Mode", 0, "Rate")
	tree_root.connect_node("Mode", 1, "Sit")
	tree_root.connect_node("output", 0, "Mode")

	_tree = AnimationTree.new()
	_tree.name = "AnimationTree"
	_tree.tree_root = tree_root
	# The node has to be in the tree before either path can be resolved.
	inst.add_child(_tree)
	_tree.anim_player = _tree.get_path_to(player)
	_tree.active = true
	set_gait(0.0)


## A seated pose, authored as a one-key looping clip so the AnimationTree can
## play it like any other.
##
## Quaternius ships Sitting_Enter/Idle/Exit in the paid tier only. A two-key
## pose is still a clip, and being a clip is the whole point: it goes through
## the same evaluation the locomotion does instead of trying to outrun it.
##
## Every bone the pose needs is keyed, including the arms. A track the clip
## does not carry falls back to the rest pose, and the retargeted rest pose has
## the arms horizontal -- so a seated figure with unkeyed arms sits in a T.
static func _sit_clip() -> Animation:
	var pose := {
		"Hips": Vector3(-6, 0, 0), "Spine": Vector3(4, 0, 0), "Chest": Vector3(3, 0, 0),
		"LeftUpperLeg": Vector3(82, 0, -4), "RightUpperLeg": Vector3(82, 0, 4),
		"LeftLowerLeg": Vector3(100, 0, 0), "RightLowerLeg": Vector3(100, 0, 0),
		# knee bend swept rather than guessed: the foot clearance bottoms out
		# near +0.06 m around 100 degrees and does not improve past it
		"LeftFoot": Vector3(-26, 0, 0), "RightFoot": Vector3(-26, 0, 0),
		# Brought all the way down: the rest pose these start from is a T, so
		# this is ~90 degrees of travel, not a nudge. The axis was swept rather
		# than assumed -- X lowers the arm, Z swings it out to the side, and
		# guessing Z first produced a figure sitting with its arms held out.
		"LeftUpperArm": Vector3(-76, 0, 9), "RightUpperArm": Vector3(-76, 0, -9),
		"LeftLowerArm": Vector3(-22, 0, 8), "RightLowerArm": Vector3(-22, 0, -8),
	}
	var a := Animation.new()
	a.length = 2.0
	a.loop_mode = Animation.LOOP_LINEAR
	for bone: String in pose:
		var ti := a.add_track(Animation.TYPE_ROTATION_3D)
		# the same addressing the imported clips use, so it resolves the same way
		a.track_set_path(ti, "%%GeneralSkeleton:%s" % bone)
		var e: Vector3 = pose[bone]
		var q := Quaternion(Basis.from_euler(Vector3(
			deg_to_rad(e.x), deg_to_rad(e.y), deg_to_rad(e.z))))
		a.rotation_track_insert_key(ti, 0.0, q)
	return a


static func euler_q(e: Vector3) -> Quaternion:
	return Quaternion(Basis.from_euler(Vector3(
		deg_to_rad(e.x), deg_to_rad(e.y), deg_to_rad(e.z))))


## The gripping arm, as rest-relative eulers, solved (by the since-deleted `tools/stand_pose.gd -- grip`, PR
## S26-c-pose keeps the recorded values) inside the old procedural stand: the strap rides the chest, so a grip solved
## against the old spine does not land on it here.
## Knuckle 0.3 mm from the strap; the elbow hangs 216 mm below the shoulder and
## 87 mm out, a little behind the side -- with this forearm a hand on the strap
## at chest height cannot have its elbow in front, and the reference's is down
## at her side.
const GRIP_UPPER := Vector3(62.8, 57.2, 39.4)
const GRIP_LOWER := Vector3(109.0, -16.9, -23.7)
## The same arm solved on the `REFERENCE` body by `tools/stand_pose.gd -- grip
## reference`. Its arms are ~20 % longer and its strap rides higher and further
## out (docs/references/CHARACTER_ROUTE_D_PLUS.md §8), so the town body's angles
## do not land on its strap.
## The wrist is solved too: turned up, so the knuckles sit on the webbing above
## it and the elbow can hang at her side.
## Re-solved 2026-10-07 for the strap point at y 1.36, on the padded strap where
## it lies on top of the hoodie's shoulder (§8.7): knuckle 0.1 mm from the
## target, wrist below it and in front of the cloth, elbow 255 mm below the
## shoulder, 87 mm out.
const GRIP_UPPER_D := Vector3(73.0, 31.6, 8.0)
const GRIP_LOWER_D := Vector3(139.2, 41.3, 19.1)
const GRIP_HAND_D := Vector3(63.5, 3.8, 71.0)


## The gripping arm's rest-relative eulers for a body: [upper, lower], plus the
## hand where the grip needs the wrist.
static func grip_angles(body: Body) -> Array[Vector3]:
	if body == Body.REFERENCE:
		return [GRIP_UPPER_D, GRIP_LOWER_D, GRIP_HAND_D]
	return [GRIP_UPPER, GRIP_LOWER]


## The standing loop, the recorded idle (`STAND_LIB`). One clip for every body: the import retargets it onto the
## humanoid profile, so it plays on any body the bone map covers, cast, protagonist and town alike.
static var _stand: Animation


static func _stand_clip() -> Animation:
	if _stand == null:
		var scene := (load(STAND_LIB) as PackedScene).instantiate()
		var ap := scene.find_children("*", "AnimationPlayer", true, false)[0] as AnimationPlayer
		var lib := ap.get_animation_library(ap.get_animation_library_list()[0])
		_stand = (lib.get_animation("Stand") as Animation).duplicate()
		_stand.loop_mode = Animation.LOOP_LINEAR
		scene.queue_free()
	return _stand


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
	if want > _walk_mps:
		var jog := want >= (_walk_mps + _jog_mps) * 0.5
		blend = _jog_mps if jog else _walk_mps
		rate = want / blend
	_tree.set("parameters/Locomotion/blend_position", blend)
	_tree.set("parameters/Rate/scale", rate)
	if _ground:
		# standing and walking, a foot is always on the floor: hold it there; a jog leaves the floor on purpose
		_ground.pin = 1.0 - smoothstep(_walk_mps, _jog_mps, blend)
	if _feet:
		_feet.body_speed = maxf(speed_mps, 0.0)
	if _posture:
		_posture.tweak_weight = clampf(blend / _walk_mps, 0.0, 1.0)
		if _body == Body.REFERENCE:
			# lets go of the strap as she sets off; see Posture.grip_weight
			_posture.grip_weight = 1.0 - smoothstep(0.0, 0.5 * _walk_mps, blend)


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


## Sit on something `seat_y` high.
##
## The pose comes from the tree (see `_sit_clip`); this only switches the tree
## into it, stands the standing-posture correction down, and puts the hips on
## the seat.
func sit(seat_y := 0.45) -> void:
	_sitting = true
	_seat_y = seat_y
	if _posture:
		# the standing correction tucks arms and thighs in and fights the pose
		_posture.active = false
	if _ground:
		# seated, the contact is the seat (`_place_on_seat`), not the floor
		_ground.active = false
	if _feet:
		_feet.set_enabled(false)
	if _tree:
		_tree.set("parameters/Mode/transition_request", "sit")
	if is_inside_tree():
		_settle_seat()


func _ready() -> void:
	if _sitting:
		_settle_seat()
	_blink_rng.seed = get_instance_id()
	_blink_next = _blink_rng.randf_range(0.5, 4.0)


## One blink: 70 ms closing, 30 ms shut, 120 ms opening -- the lid comes down
## faster than it goes up. Every 2.4-5.5 s, and one time in six a double.
func _process(delta: float) -> void:
	if _blink_idx < 0:
		return
	if blink_hold >= 0.0:
		_face.set_blend_shape_value(_blink_idx, blink_hold)
		return
	_blink_t += delta
	var w := 0.0
	var t := _blink_t - _blink_next
	if t >= 0.0:
		if t < 0.07:
			w = t / 0.07
		elif t < 0.10:
			w = 1.0
		elif t < 0.22:
			w = 1.0 - (t - 0.10) / 0.12
		else:
			_blink_t = 0.0
			if _blink_double:
				_blink_double = false
				_blink_next = 0.12
			else:
				_blink_double = _blink_rng.randf() < 0.16
				_blink_next = _blink_rng.randf_range(2.4, 5.5)
	_face.set_blend_shape_value(_blink_idx, w * BLINK_PEAK)


## The hips can only be measured once the tree has actually written the seated
## pose. A single deferred call is too early -- it measures the standing pose
## and places the body half a metre out, which is what the first version did.
func _settle_seat() -> void:
	await get_tree().process_frame
	await get_tree().process_frame
	_place_on_seat()


## Put the hips on the seat the caller says the person is sitting on.
##
## The first version solved for the feet instead -- drop the body until they
## reach y = 0 -- and let the hips land wherever the pose put them, about
## 0.39 m, which is below every chair in the scene. A seated person's contact
## with the world is the seat, not the floor.
func _place_on_seat() -> void:
	var hi := skeleton.find_bone("Hips")
	if hi < 0 or _inst == null:
		return
	var hips_y := skeleton.get_bone_global_pose(hi).origin.y * scale.y
	_inst.position.y = (_seat_y - hips_y) / maxf(scale.y, 0.01)
	# Report the foot clearance, so a seat height that does not suit the pose
	# shows up in the log rather than only in a screenshot from one angle.
	var lowest := INF
	for bone in ["LeftFoot", "RightFoot"]:
		var i := skeleton.find_bone(bone)
		if i >= 0:
			lowest = minf(lowest, skeleton.get_bone_global_pose(i).origin.y)
	if lowest < INF:
		var foot: float = (lowest + _inst.position.y) * scale.y
		if absf(foot) > 0.10:
			push_warning("human.gd: seated on a %.2f m seat leaves the feet %+.2f m off the floor"
				% [_seat_y, foot])
