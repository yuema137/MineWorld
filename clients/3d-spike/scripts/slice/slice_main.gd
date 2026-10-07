## Entry point for the VIS-3D-GODOT-2 slice: lighting rig, then the street,
## then the player.
##
## The rig is built before anything is placed, deliberately, and every material
## is judged inside it or not at all (`DEP-8` coherence procedure, step 3).
## Structurally it is `ARC-13`'s rig unchanged -- one sky, one sun, one tonemap,
## one exposure -- because the structure was the part of it that was right. What
## is new is the global-illumination stage, the reflection probes and the
## interior fill, all of which exist because this scene has an inside and
## `ARC-13`'s did not really.
##
## THE SUN, and why it points where it does. 19 degrees of elevation, roughly
## west-south-west. Two things had to be true at once: the north pavement must
## stay in sun (a golden-hour frame with no golden hour in it is the defect
## `ARC-13` names), and the sun must reach through the cafe's glazing far enough
## to put a patch of light on its floor, because the threshold is the thing
## this slice exists to show. At this azimuth the beam reaches about 4.8 m into
## the room. `street.gd` records the massing consequence.
##
## GLOBAL ILLUMINATION is chosen, not assumed -- see `_gi()`. `--gi=<mode>` runs
## the alternatives so the choice rests on frames and numbers.
class_name SliceMain
extends Node3D

enum GI { NONE, SSIL, VOXEL, SDFGI }

const SUN_ELEVATION := -19.3
## -48, was -55.8. Measured with --measure's sun rays: at -68 the beam met the
## café façade at 20.7 deg (irradiance factor 0.35) -- raking, but too dim to
## read as sunlit. At -48 it meets it at ~42 deg (factor ~0.6), and at 19 deg
## elevation every table, A-board and pilaster still throws a long shadow.
const SUN_AZIMUTH := -48.0
const SUN_ENERGY := 4.4
## The sky's share, lowered against the sun's so sunlit and shaded planes part.
const SKY_AMBIENT := 1.0
const INTERIOR_AMBIENT := Color(0.46, 0.33, 0.22)
const INTERIOR_AMBIENT_ENERGY := 0.55

var player: SlicePlayer
var world: Node3D
var probe: SliceProbe = null
var gi_mode := GI.VOXEL

var _env: Environment
var hud: ControlsHud = null
var _hud_place: Label = null


func _ready() -> void:
	gi_mode = _gi_from_args()
	_environment()
	_sun()
	world = SliceWorld.build(self)
	_gi()
	_reflections()
	_spawn_player()
	_hud()
	_link()

	if SliceProbe.scripted():
		probe = SliceProbe.new()
		probe.player = player
		probe.slice = self
		add_child(probe)


## `Props.gltf` keeps one generated scene per slug as a template it duplicates
## from. Those templates are never in the tree, so nothing frees them, and every
## quit reported each one -- with its meshes, materials and textures -- as
## leaked. Freed here, from the slice's own exit, rather than by editing the
## shared `props.gd` the promenade scene also runs on.
func _exit_tree() -> void:
	for k in Props._scenes.keys():
		var n: Node = Props._scenes[k]
		if n != null and is_instance_valid(n):
			n.free()
	Props._scenes.clear()


static func _gi_from_args() -> GI:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--gi="):
			match a.substr(5):
				"none": return GI.NONE
				"ssil": return GI.SSIL
				"voxel": return GI.VOXEL
				"sdfgi": return GI.SDFGI
	return GI.VOXEL


func gi_name() -> String:
	return ["none", "ssil", "voxel", "sdfgi"][gi_mode]


# --- the rig -------------------------------------------------------------------

func _environment() -> void:
	var env := Environment.new()

	# One sky, and only one. An HDRI *and* a procedural sky double-lights
	# everything; the fallback below is only for a checkout whose HDRI is
	# missing, and it warns rather than pretending.
	var sky := Sky.new()
	var hdr_path := "res://assets/hdri/qwantani_puresky_2k.hdr"
	var pano_tex: Texture2D = null
	if ResourceLoader.exists(hdr_path):
		pano_tex = load(hdr_path) as Texture2D
	if pano_tex != null:
		var pano := PanoramaSkyMaterial.new()
		pano.panorama = pano_tex
		pano.energy_multiplier = 0.80
		sky.sky_material = pano
	else:
		push_warning("no HDRI at %s -- falling back to a procedural sky" % hdr_path)
		var ps := ProceduralSkyMaterial.new()
		ps.sky_top_color = Color(0.29, 0.47, 0.73)
		ps.sky_horizon_color = Color(0.86, 0.80, 0.70)
		ps.ground_bottom_color = Color(0.26, 0.26, 0.24)
		sky.sky_material = ps
	# 512 rather than 256: the radiance map is the only reflection source for
	# anything outside a probe, and shop glass at roughness 0.025 shows its
	# mip level as a visible blur band across the pane.
	sky.radiance_size = Sky.RADIANCE_SIZE_512
	env.sky = sky
	env.background_mode = Environment.BG_SKY
	env.ambient_light_source = Environment.AMBIENT_SOURCE_SKY
	env.ambient_light_sky_contribution = 1.0
	env.ambient_light_energy = SKY_AMBIENT
	env.reflected_light_source = Environment.REFLECTION_SOURCE_SKY

	env.tonemap_mode = Environment.TONE_MAPPER_AGX
	env.tonemap_exposure = 1.06
	env.tonemap_white = 6.0

	# Contact shadowing. The radius is smaller than the spike's 1.2 m because
	# this scene's detail is at the scale of a moulding and a mug, not a wall.
	env.ssao_enabled = true
	env.ssao_radius = 0.65
	env.ssao_intensity = 2.0
	env.ssao_power = 1.5
	env.ssao_detail = 1.0
	env.ssao_horizon = 0.06
	env.ssao_light_affect = 0.12

	env.glow_enabled = true
	env.glow_intensity = 0.40
	env.glow_bloom = 0.05
	env.glow_hdr_threshold = 1.10
	env.glow_blend_mode = Environment.GLOW_BLEND_MODE_SOFTLIGHT

	# Depth fog only, and only far away: the backdrop ridges are aerial
	# perspective and nothing else. Volumetric fog is evaluated separately and
	# is off by default -- see `_gi()`.
	env.fog_enabled = true
	env.fog_mode = Environment.FOG_MODE_DEPTH
	env.fog_light_color = Color(0.76, 0.80, 0.87)
	env.fog_light_energy = 1.0
	env.fog_sun_scatter = 0.22
	env.fog_density = 0.0
	env.fog_depth_begin = 70.0
	env.fog_depth_end = 1500.0
	env.fog_depth_curve = 0.42
	env.fog_aerial_perspective = 0.60

	env.adjustment_enabled = true
	env.adjustment_brightness = 1.0
	env.adjustment_contrast = 1.06
	env.adjustment_saturation = 1.14

	var we := WorldEnvironment.new()
	we.environment = env
	add_child(we)
	_env = env

	# Fixed exposure. Auto exposure would hide the very thing sec.7.1 measures:
	# whether the step across the threshold is survivable without the camera
	# quietly rescaling the frame for us.
	var ca := CameraAttributesPractical.new()
	ca.dof_blur_far_enabled = false
	ca.auto_exposure_enabled = false
	we.camera_attributes = ca


func _sun() -> void:
	var sun := DirectionalLight3D.new()
	sun.name = "Sun"
	sun.light_color = SlicePalette.SUN_WARM
	sun.light_energy = SUN_ENERGY
	sun.light_angular_distance = 0.55
	sun.shadow_enabled = true
	sun.directional_shadow_mode = DirectionalLight3D.SHADOW_PARALLEL_4_SPLITS
	sun.directional_shadow_max_distance = 95.0
	sun.directional_shadow_split_1 = 0.055
	sun.directional_shadow_split_2 = 0.16
	sun.directional_shadow_split_3 = 0.42
	sun.directional_shadow_blend_splits = true
	# A 19-degree sun rakes every flat surface, and at grazing angles a small
	# depth bias leaves dark blotches of self-shadowing on the paving. More
	# normal bias buys that back at the cost of a little contact tightness,
	# which SSAO then puts back.
	sun.shadow_bias = 0.024
	sun.shadow_normal_bias = 2.2
	sun.rotation_degrees = Vector3(SUN_ELEVATION, SUN_AZIMUTH, 0.0)
	add_child(sun)

	# A cool sky-side fill. Without it the shadow side of every wall goes muddy
	# and a sunny street reads as an overcast one.
	var fill := DirectionalLight3D.new()
	fill.name = "SkyFill"
	fill.light_color = Color(0.60, 0.71, 0.92)
	fill.light_energy = 0.26
	fill.shadow_enabled = false
	fill.rotation_degrees = Vector3(-46.0, 118.0, 0.0)
	add_child(fill)


# --- global illumination, chosen rather than assumed ---------------------------

## The brief's instruction is explicit: do not use SDFGI merely because it
## exists. The four modes here are the real alternatives for this scene and
## `--gi=<mode>` runs each one, so the choice is made on captured frames and on
## the numbers `--threshold` prints rather than on a preference.
##
## What each one actually is here:
##
##   none   sun, sky ambient, and the interior's own fittings. The baseline.
##   ssil   screen-space indirect light: short-range bounce, no cost when
##          nothing is on screen, and it disappears at a screen edge.
##   voxel  a VoxelGI baked at load over the cafe and its doorway only. Real
##          bounce inside a bounded volume, stable under a moving camera,
##          and it is the doorway that needs it.
##   sdfgi  whole-world real-time GI.
##
## LightmapGI is **not** among them, and the reason is structural rather than a
## preference: this scene is built from script at load, so there is no authored
## mesh to unwrap a UV2 for and no edit-time pass in which to bake one. Baked
## lightmapping would mean authoring the slice as a `.tscn` of hand-unwrapped
## meshes, which is a different project. Recorded here so the next contributor
## does not read "no lightmaps" as an oversight.
func _gi() -> void:
	match gi_mode:
		GI.NONE:
			_env.ssil_enabled = false
		GI.SSIL:
			_ssil()
		GI.VOXEL:
			_ssil()
			_voxel_gi()
		GI.SDFGI:
			_env.sdfgi_enabled = true
			_env.sdfgi_cascades = 4
			_env.sdfgi_min_cell_size = 0.20
			_env.sdfgi_use_occlusion = true
			_env.sdfgi_energy = 1.0
			_env.sdfgi_bounce_feedback = 0.5


func _ssil() -> void:
	_env.ssil_enabled = true
	_env.ssil_radius = 3.0
	_env.ssil_intensity = 1.35
	_env.ssil_sharpness = 0.98
	_env.ssil_normal_rejection = 1.0


## One VoxelGI over the cafe, its doorway and the pavement immediately outside.
## Bounded on purpose: the volume that needs real bounce is the room, and a
## VoxelGI large enough to cover the whole street would have cells too coarse to
## resolve a counter. The extents reach 4 m out past the facade so that the
## light spilling out of the door and the light bouncing in off the sunlit
## paving are both inside the bake.
func _voxel_gi() -> void:
	var v := VoxelGI.new()
	v.name = "CafeGI"
	var d := VoxelGIData.new()
	v.data = d
	v.subdiv = VoxelGI.SUBDIV_256
	var depth := SliceCafe.DEPTH + 9.0
	v.size = Vector3(SliceCafe.W + 5.0, 9.0, depth)
	v.position = Vector3(6.0, 4.0,
		SliceStreet.NORTH_FACE - SliceCafe.DEPTH * 0.5 + 4.5)
	add_child(v)
	# Baked at load, from script. The scene is fully built by this point, which
	# is why `_gi()` runs after `SliceWorld.build`.
	v.bake(self, false)
	# The tuning lives on the data resource, not on the node: VoxelGI is the
	# placement and VoxelGIData is the bake, and the bake is what `energy`,
	# `propagation` and the biases belong to.
	d = v.data
	d.energy = 1.15
	d.propagation = 0.62
	d.bias = 1.4
	d.normal_bias = 0.6


## Reflection probes: one inside the cafe and one over the pavement in front of
## it. `VISUAL_SLICE.md` sec.7.1 requires that glass, a polished counter top and
## metal reflect the room they are in rather than the sky through the ceiling,
## and sky-only reflection is exactly how an interior gets that wrong.
func _reflections() -> void:
	var inside := ReflectionProbe.new()
	inside.name = "CafeProbe"
	inside.size = Vector3(SliceCafe.W, 3.4, SliceCafe.DEPTH)
	inside.origin_offset = Vector3.ZERO
	inside.position = Vector3(6.0, SliceStreet.WALK_Y + 1.7,
		SliceStreet.NORTH_FACE - SliceCafe.DEPTH * 0.5)
	inside.box_projection = true
	inside.interior = true
	inside.intensity = 1.0
	inside.max_distance = 26.0
	inside.update_mode = ReflectionProbe.UPDATE_ONCE
	# The room's own ambient, in place of the sky's: inside an interior probe
	# Godot takes ambient light from the probe, so the café is lit by a warm,
	# dim fill instead of a cool sky it cannot see. This is what lets the room
	# read darker and warmer than the street, as in 03, without a black corner.
	inside.ambient_mode = ReflectionProbe.AMBIENT_COLOR
	inside.ambient_color = INTERIOR_AMBIENT
	inside.ambient_color_energy = INTERIOR_AMBIENT_ENERGY
	add_child(inside)

	var outside := ReflectionProbe.new()
	outside.name = "FrontageProbe"
	outside.size = Vector3(22.0, 9.0, 9.0)
	outside.position = Vector3(6.0, 3.2, SliceStreet.NORTH_FACE + 3.4)
	outside.box_projection = true
	outside.interior = false
	outside.intensity = 0.85
	outside.max_distance = 60.0
	outside.update_mode = ReflectionProbe.UPDATE_ONCE
	add_child(outside)


# --- player and HUD ------------------------------------------------------------

func _spawn_player() -> void:
	player = SlicePlayer.new()
	player.name = "Player"
	player.look_enabled = not SliceProbe.scripted()
	add_child(player)
	player.place(SliceWorld.SPAWN, SliceWorld.SPAWN_YAW, -2.0)


func _process(_d: float) -> void:
	if _hud_place != null and player != null:
		var p := SliceWorld.place_at(world, player.global_position)
		_hud_place.text = "place: %s" % (p if p != "" else "-")


func _hud() -> void:
	if SliceProbe.scripted():
		return
	hud = ControlsHud.attach(self, player)
	_hud_place = hud.add_line("place: -")
	_hud_world = hud.add_line("world: offline  (./mineworld-slice --server=host:port)")


## The connection to a running MineWorld world (`VISUAL_SLICE.md` sec.9), when
## `--server=` is given. Without it the slice runs offline and says so.
var link: SliceLink = null
var _hud_world: Label = null


func _link() -> void:
	var address := SliceLink.address_from_args()
	if address == "":
		return
	link = SliceLink.new()
	link.name = "SliceLink"
	link.player = player
	link.world_root = world
	add_child(link)
	link.said.connect(func(text: String) -> void:
		if _hud_world != null:
			_hud_world.text = "world: " + text
		if hud != null and (text.begins_with("seated") or text.begins_with("talk")
				or text.begins_with("refused") or text.begins_with("disconnected")
				or text.contains(" said: ")):
			hud.toast(text))
	link.start(address, SliceLink.seat_from_args())
