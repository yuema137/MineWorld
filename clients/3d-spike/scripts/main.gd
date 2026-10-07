## Entry point: lighting rig first, then the town, then the player.
##
## The rig is fixed and built before anything is placed, deliberately. One HDRI
## sky (never an HDRI *and* a procedural sky -- that double-lights everything),
## one sun, one tonemap, one exposure. Judge every material inside it or not at
## all.
extends Node3D

const SHOT_DIR := "res://shots"

var player: Player
var _shots: Shots = null
var _mode_label: Label = null


func _ready() -> void:
	_environment()
	_sun()
	Town.build(self)
	_spawn_player()
	_hud()

	if _scripted():
		_shots = Shots.new()
		_shots.player = player
		add_child(_shots)


## True when this run is driven by a script rather than by a human.
static func _scripted() -> bool:
	var a := OS.get_cmdline_user_args()
	return ("--shots" in a or "--drive" in a or "--portrait" in a or "--bodycheck" in a
		or "--motion" in a or "--frametime" in a or "--sweep" in a or "--headtrace" in a)


func _environment() -> void:
	var env := Environment.new()

	var sky := Sky.new()
	# The sky is loaded as an *imported resource*, not read off the filesystem.
	# Image.load_from_file() works from a project directory and silently fails
	# once the project is exported to a binary -- the sky would simply vanish
	# from a release build, which is exactly when it would cost the most, and
	# ARC-6 has MVP-0 shipping mineworld-3d as something a person runs. Godot
	# imports .hdr as a CompressedTexture2D, which is what PanoramaSkyMaterial
	# wants, so this is also one object fewer than building an ImageTexture.
	var hdr_path := "res://assets/hdri/qwantani_puresky_2k.hdr"
	var pano_tex: Texture2D = null
	if ResourceLoader.exists(hdr_path):
		pano_tex = load(hdr_path) as Texture2D
	if pano_tex != null:
		var pano := PanoramaSkyMaterial.new()
		pano.panorama = pano_tex
		# Below 1.0 on purpose: qwantani's sun disc is very bright and at this
		# exposure it blew the sky to white in any frame that included it.
		pano.energy_multiplier = 0.80
		sky.sky_material = pano
	else:
		push_warning("no HDRI at %s -- falling back to a procedural sky" % hdr_path)
		var ps := ProceduralSkyMaterial.new()
		ps.sky_top_color = Color(0.29, 0.47, 0.73)
		ps.sky_horizon_color = Color(0.77, 0.78, 0.75)
		ps.ground_bottom_color = Color(0.26, 0.26, 0.24)
		ps.ground_horizon_color = Color(0.66, 0.66, 0.62)
		ps.sun_angle_max = 12.0
		sky.sky_material = ps
	sky.radiance_size = Sky.RADIANCE_SIZE_256
	env.sky = sky
	env.background_mode = Environment.BG_SKY
	env.ambient_light_source = Environment.AMBIENT_SOURCE_SKY
	env.ambient_light_sky_contribution = 1.0
	# A low sun leaves north-facing shopfronts in deep shade, and 07 was dark
	# enough that its own signage stopped being legible. Lifting sky ambient
	# fixes the shadow end without touching the key, which is what keeps the
	# golden-hour read.
	env.ambient_light_energy = 1.55
	env.reflected_light_source = Environment.REFLECTION_SOURCE_SKY

	# Warm late afternoon, moderate contrast. ART_DIRECTION sec.5 is explicit
	# that this must not read as a movie poster, so: no heavy vignette, modest
	# glow, a small warm lift rather than a teal-orange grade.
	env.tonemap_mode = Environment.TONE_MAPPER_AGX
	env.tonemap_exposure = 1.06
	env.tonemap_white = 6.0

	env.ssao_enabled = true
	env.ssao_radius = 1.2
	env.ssao_intensity = 1.5
	env.ssao_power = 1.4
	env.ssao_detail = 0.6

	env.ssil_enabled = false

	env.glow_enabled = true
	env.glow_intensity = 0.42
	env.glow_bloom = 0.06
	env.glow_hdr_threshold = 1.05
	env.glow_blend_mode = Environment.GLOW_BLEND_MODE_SOFTLIGHT

	env.fog_enabled = true
	env.fog_mode = Environment.FOG_MODE_DEPTH
	env.fog_light_color = Color(0.72, 0.79, 0.88)
	env.fog_light_energy = 1.0
	env.fog_sun_scatter = 0.18
	env.fog_density = 0.0
	env.fog_depth_begin = 90.0
	env.fog_depth_end = 1300.0
	env.fog_depth_curve = 0.45
	env.fog_aerial_perspective = 0.55

	env.adjustment_enabled = true
	env.adjustment_brightness = 1.0
	env.adjustment_contrast = 1.07
	env.adjustment_saturation = 1.16

	var we := WorldEnvironment.new()
	we.environment = env
	add_child(we)

	var ca := CameraAttributesPractical.new()
	ca.dof_blur_far_enabled = false
	ca.auto_exposure_enabled = false
	we.camera_attributes = ca


func _sun() -> void:
	var sun := DirectionalLight3D.new()
	sun.light_color = Color(1.0, 0.80, 0.56)
	sun.light_energy = 2.9
	sun.light_angular_distance = 0.6
	sun.shadow_enabled = true
	sun.directional_shadow_mode = DirectionalLight3D.SHADOW_PARALLEL_2_SPLITS
	sun.directional_shadow_max_distance = 115.0
	sun.directional_shadow_split_1 = 0.14
	sun.directional_shadow_blend_splits = true
	# A 17-degree sun rakes the pavement, and at grazing angles the old bias
	# left dark blotches of self-shadowing on flat ground. More normal bias and
	# less depth bias trades a little contact-shadow tightness for that.
	sun.shadow_bias = 0.028
	sun.shadow_normal_bias = 2.6
	# 17 degrees above the horizon, raking across the street from the west:
	# golden hour, long shadows, warm faces on the shopfronts. The plates are
	# warm and saturated and this rig was neither -- an overcast sky and a pale
	# sun made every frame read grey no matter what the geometry did.
	sun.rotation_degrees = Vector3(-17.0, 124.0, 0.0)
	add_child(sun)

	# A cool sky-side fill. Without it the shadow side of every wall goes muddy
	# and the whole street looks overcast instead of sunny.
	var fill := DirectionalLight3D.new()
	fill.light_color = Color(0.62, 0.72, 0.92)
	fill.light_energy = 0.28
	fill.shadow_enabled = false
	fill.rotation_degrees = Vector3(-48.0, -40.0, 0.0)
	add_child(fill)


func _spawn_player() -> void:
	player = Player.new()
	player.look_enabled = not _scripted()
	add_child(player)
	player.place(Vector3(2.0, 0.2, -17.0), 196.0, -4.0)


func _hud() -> void:
	if _scripted():
		return
	var c := CanvasLayer.new()
	add_child(c)
	c.add_child(_hud_line(
		"W/S walk   A/D strafe   mouse look   Shift jog   F5 camera   Esc release mouse", 14))
	_mode_label = _hud_line("camera: %s" % player.rig.mode_name(), 36)
	c.add_child(_mode_label)
	player.camera_mode_changed.connect(_on_camera_mode_changed)


func _hud_line(txt: String, y: float) -> Label:
	var l := Label.new()
	l.text = txt
	l.position = Vector2(18, y)
	l.add_theme_color_override("font_color", Color(1, 1, 1, 0.82))
	l.add_theme_color_override("font_shadow_color", Color(0, 0, 0, 0.7))
	l.add_theme_constant_override("shadow_offset_y", 1)
	l.add_theme_constant_override("shadow_offset_x", 1)
	return l


func _on_camera_mode_changed(mode_name: String) -> void:
	_mode_label.text = "camera: %s" % mode_name
