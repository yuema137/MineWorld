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

## The player's settings (S20, `res://mineworld_settings`, `clients/shared/SETTINGS.md`): read and
## applied first, before anything is drawn; the menu opens with Esc.
var settings_store: MineWorldSettingsStore
var settings_menu: MineWorldSettingsMenu
## What the first `_process` found in effect (AC-SET-7), for a probe to print.
var first_frame := {}
var _look_before := [true, false]
var _holds := 0                    ## menus open now (`_hold_input`)
## Connected only: your menu (B) and the menu of the person you look at (Tab) (PR 16d).
var offer_menu: SliceOfferMenu = null
## The world clock, the sun rig and the host module (TW-e, SET-c; `slice_clock.gd`).
var clock: SliceClock


func _ready() -> void:
	var low := _low_render_cost()
	if low:
		MineWorldDisplay.render_scale_factor = LOW_RENDER_SCALE
	_settings()
	# the depiction before the far field, so a missing depiction is named as such (ARC-146)
	if not _open_depiction():
		return
	if not _load_far_field():
		return
	gi_mode = GI.NONE if low else _gi_from_args()
	_environment()
	_sun()
	# connected or not is known from the arguments before anything is built
	world = depiction.build(self, far_field, SliceLink.address_from_args() != "")
	_gi()
	_reflections()
	if low:
		_cost_effects_off()
	_spawn_player()
	clock = SliceClock.new()
	add_child(clock)
	clock.setup(settings_store, self, _env, _scripted() or SliceSun.fixed_requested())
	_hud()
	if not _wording():
		MineWorldText.load_layers("", PackedStringArray(), false)  # the HUD loads catalogs if none are
	_link()

	if _scripted():
		if SliceProbeMarket.requested():
			probe = SliceProbeMarket.new()
		elif SliceProbeBodies.requested():
			probe = SliceProbeBodies.new()
		elif SliceProbeGeometry.requested():
			probe = SliceProbeGeometry.new()
		elif SliceProbeTown.requested():
			probe = SliceProbeTown.new()
		elif SliceProbeWorld.requested():
			probe = SliceProbeWorld.new()
		else:
			probe = SliceProbe.new()
		probe.player = player
		probe.slice = self
		add_child(probe)


## The 3D Presentation Pack, from this project's folder. The only part of the pack the slice reads
## before 16f is `far_field/` (RL-e0 SD-FF-6) and the cast binding.
const PACK_DIR := "../../presentation/mineworld-default/3D"

## The world this scene presents, by its key (`SliceWorlds.key()`), and the depiction that draws it
## (ARC-144). "" when a join named no world: the far-field loader then refuses, unchanged.
var world_key := ""
var depiction: SliceDepiction = null
## The Presentation Pack's far field for this world (`docs/MODULE_SPEC.md` sec.6.1.1).
var far_field: MineWorldFarField = null


## Opens the world's depiction. A key with none is refused by name (PRESENTATION_WORLD_MISSING,
## ARC-146): printed, shown, and in a scripted or harness run the process exits 3. No fallback.
func _open_depiction() -> bool:
	world_key = SliceWorlds.key()
	if world_key == "":
		return true
	depiction = SliceWorlds.open(world_key)
	if depiction != null:
		print("[slice] world %s, drawn by %s" % [world_key, SliceWorlds.DEPICTION % world_key])
		return true
	_refuse(SliceWorlds.refusal_line(world_key))
	return false


## A refusal by name: printed, shown full-screen, and in a scripted or harness run the process exits 3.
func _refuse(line: String) -> void:
	printerr(line)
	print(line)
	var layer := CanvasLayer.new()
	var label := Label.new()
	label.text = line
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.set_anchors_preset(Control.PRESET_FULL_RECT)
	label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	layer.add_child(label)
	add_child(layer)
	var harness := _scripted()
	for arg in OS.get_cmdline_user_args():
		harness = harness or arg.begins_with("--slice-")
	if harness:
		get_tree().quit(3)


## Reads the far field before anything is drawn. A missing or faulty one is refused by name: printed,
## shown, and in a scripted or harness run the process exits 3. There is no fallback (ARC-102).
##
## `--slice-far-field-file=<path>` draws another far-field file on this street instead -- another
## world's, or a generated variant -- validated exactly as the pack's own (its scope must equal its
## file's stem). It is how the rendered check is proven not to depend on one world's numbers
## (RL-e0b; `scripts/far_field_variant.py` writes rotated variants).
func _load_far_field() -> bool:
	var dir := ProjectSettings.globalize_path("res://").path_join(PACK_DIR).simplify_path()
	var file := ""
	for arg in OS.get_cmdline_user_args():
		if arg.begins_with("--slice-far-field-file="):
			file = arg.trim_prefix("--slice-far-field-file=")
	if file == "":
		far_field = MineWorldFarField.load_for(dir, depiction.far_field_key() if depiction != null else "",
			"3d")
	else:
		far_field = MineWorldFarField.load_file(file, PackedStringArray(["3d"]))
		print("far field from %s" % file)
	if far_field.ok():
		return true
	_refuse(far_field.refusal_line("slice"))
	return false


## Whether a scripted probe mode, standalone or connected, runs this session.
static func _scripted() -> bool:
	return SliceProbe.scripted() or SliceProbeWorld.requested() or SliceProbeTown.requested() \
		or SliceProbeMarket.requested() \
		or SliceProbeGeometry.requested() or SliceProbeBodies.requested()


## Every probe mode is a harness flag: a scripted run never reads the player's settings (INV-SET-5).
static func harness_flags() -> PackedStringArray:
	var out := PackedStringArray()
	for mode in SliceProbe.MODES + SliceProbeWorld.WORLD_MODES + SliceProbeTown.TOWN_MODES \
			+ SliceProbeMarket.MARKET_MODES + SliceProbeGeometry.GEOMETRY_MODES + SliceProbeBodies.BODIES_MODES:
		out.append("slice-" + mode)
	return out


## The settings, the shared catalogs (the 3D client has no Presentation Pack yet, S14 16f), the CJK
## font, the clock, the language and the display, with the render scale this client offers.
func _settings() -> void:
	settings_store = MineWorldSettingsStore.open(OS.get_cmdline_user_args(), harness_flags())
	for problem in MineWorldText.load_layers("", settings_store.extra_locale_dirs, _wording()):
		push_warning(problem)
	settings_store.settle_language(MineWorldText.locales())
	MineWorldText.install_font_fallback()
	MineWorldText.set_clock(settings_store.settings.clock)
	MineWorldText.set_language(settings_store.settings.language)
	MineWorldDisplay.print_platform_once()
	MineWorldDisplay.apply(settings_store.settings, get_window(),
		PackedStringArray([MineWorldDisplay.CAP_RENDER_SCALE]))


## `--no-wording` plays with no catalog at all, every key shown as its readable form: the
## language-independence check (PR 16d P-3, as the 2D client's `--no-wording`).
static func _wording() -> bool:
	return not ("--no-wording" in OS.get_cmdline_user_args())


## The settings menu over everything; while it is open the body neither walks nor looks and E asks
## for nothing. Closing leaves the mouse released until the next click, as Esc always did.
func _menu() -> void:
	settings_menu = MineWorldSettingsMenu.new()
	add_child(settings_menu)
	var caps := PackedStringArray([MineWorldDisplay.CAP_RENDER_SCALE, MineWorldControlsTab.CAP_MOUSE_LOOK])
	if clock.hosted_here:
		caps.append(MineWorldSettingsMenu.CAP_HOSTED_HERE)
	settings_menu.setup(settings_store, caps, _menu_theme())
	settings_menu.add_tab("ui.settings.tab.controls", MineWorldControlsTab.new(settings_menu,
		PackedStringArray([MineWorldControlsTab.CAP_MOUSE_LOOK]), key_rows()))
	settings_menu.applied.connect(func(value: MineWorldSettings) -> void:
		player.set_look(value.look_scale(), value.invert_y))
	clock.attach_menu(settings_menu)
	settings_menu.opened.connect(func() -> void:
		Input.mouse_mode = Input.MOUSE_MODE_VISIBLE
		_hold_input(true)
		if offer_menu != null:
			offer_menu.set_process_unhandled_input(false))
	settings_menu.closed.connect(func() -> void:
		_hold_input(false)
		if offer_menu != null:
			offer_menu.set_process_unhandled_input(true))


## THE GATE for every menu of this client (SET-a's, generalised by PR 16d, D-16d-9): while any menu is
## open the body neither walks nor looks and E asks for nothing. Counted, so the settings menu opened
## over an interaction menu, and closed again, leaves the game held until both are closed.
func _hold_input(hold: bool) -> void:
	if hold:
		_holds += 1
		if _holds > 1:
			return
		_look_before = [player.look_enabled, player.scripted_look]
		player.look_enabled = false
		player.scripted_look = false
		if link != null:
			link.set_process_unhandled_input(false)
		return
	if _holds == 0:
		return
	_holds -= 1
	if _holds > 0:
		return
	player.look_enabled = _look_before[0]
	player.scripted_look = _look_before[1]
	if link != null:
		link.set_process_unhandled_input(true)


## The menu in this client's light-on-dark HUD colours.
static func _menu_theme() -> Theme:
	var theme := Theme.new()
	var panel := StyleBoxFlat.new()
	panel.bg_color = Color(0.08, 0.09, 0.11, 0.92)
	panel.set_corner_radius_all(10)
	panel.set_content_margin_all(22)
	theme.set_stylebox("panel", "PanelContainer", panel)
	theme.set_color("font_color", "Label", Color(1, 1, 1, 0.92))
	theme.set_font_size("font_size", "Label", 15)
	return theme


## The Controls tab's key map (`clients/shared/SETTINGS.md` §7.1; SET-b): this client's own InputMap actions,
## each with its catalog key, in the order a player meets them. Esc is Godot's `ui_cancel`, which this client
## takes in `_input` before the player does, so it is the menu's key.
const KEY_MAP := [
	["move_forward", "ui.settings.key.move_forward"],
	["move_left", "ui.settings.key.move_left"],
	["move_back", "ui.settings.key.move_back"],
	["move_right", "ui.settings.key.move_right"],
	["jump", "ui.settings.key.jump"],
	["jog", "ui.settings.key.jog"],
	["camera_cycle", "ui.settings.key.camera_cycle"],
	["inspect", "ui.settings.key.inspect"],
	["ui_cancel", "ui.settings.key.menu"],
]


## One row per KEY_MAP action, its physical keys read from InputMap at start; an action with no key shows
## none. Nothing is written back: the map is read-only (QSET-7).
static func key_rows() -> Array[MineWorldKeyRow]:
	var rows: Array[MineWorldKeyRow] = []
	for pair in KEY_MAP:
		var keys := PackedInt32Array()
		for event in InputMap.action_get_events(pair[0]):
			if event is InputEventKey:
				var key := event as InputEventKey
				keys.append(key.physical_keycode if key.physical_keycode != KEY_NONE else key.keycode)
		rows.append(MineWorldKeyRow.new(pair[1], keys))
	return rows


## Esc opens and closes the menu, before the player's own Esc (which released the mouse) sees it. An
## open interaction menu is closed first (PR 16d, D-16d-9; 12e's throw aim goes before it, A16d-18).
func _input(event: InputEvent) -> void:
	if not event.is_action_pressed("ui_cancel"):
		return
	# Esc during a throw's aim drops the aim (step-11 §22 SD-E6) and opens nothing.
	if link != null and link.cancel_aim():
		get_viewport().set_input_as_handled()
		return
	if offer_menu != null and offer_menu.is_open() and not settings_menu.is_open():
		offer_menu.close()
		get_viewport().set_input_as_handled()
	elif settings_menu != null:
		settings_menu.toggle()
		get_viewport().set_input_as_handled()


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


## `--render-cost=low` (ARC-291): a CI launch on a software rasteriser (lavapipe, D3D12 WARP) pays the
## full desktop cost per pixel and draws fewer than 12 frames in 240 s. It halves the render scale (the
## settings' value times this factor; the file is unchanged), takes GI none, and turns off directional
## shadows, SSAO and glow -- none of which a scripted check judges. Players never pass it.
const LOW_RENDER_SCALE := 0.5


static func _low_render_cost() -> bool:
	return "--render-cost=low" in OS.get_cmdline_user_args()


func _cost_effects_off() -> void:
	_env.ssao_enabled = false
	_env.glow_enabled = false
	for child in get_children():
		if child is DirectionalLight3D:
			(child as DirectionalLight3D).shadow_enabled = false
	print("[slice] render cost low: scale x%.2f, GI none, shadows, SSAO and glow off (ARC-291)" % LOW_RENDER_SCALE)


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
	# The sky is the far field's (RL-e0): a panorama is an asset of this client's catalogue.
	var sky := Sky.new()
	var declared := far_field.sky()
	var pano_tex: Texture2D = null
	var hdr_path := ""
	if declared["kind"] == "panorama":
		hdr_path = "res://assets/hdri/%s.hdr" % declared["panorama"]
		if ResourceLoader.exists(hdr_path):
			pano_tex = load(hdr_path) as Texture2D
	if pano_tex != null:
		var pano := PanoramaSkyMaterial.new()
		pano.panorama = pano_tex
		pano.energy_multiplier = 0.80
		sky.sky_material = pano
	else:
		var ps := ProceduralSkyMaterial.new()
		ps.sky_top_color = Color(0.29, 0.47, 0.73)
		ps.sky_horizon_color = Color(0.86, 0.80, 0.70)
		if declared["kind"] == "procedural":
			ps.sky_top_color = Color.html(declared["procedural"]["top"])
			ps.sky_horizon_color = Color.html(declared["procedural"]["horizon"])
		else:
			push_warning("no HDRI at %s -- falling back to a procedural sky" % hdr_path)
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
	# 128, was 256: the Default tier's frame budget (RL-b SD-RLb-6a). The visible
	# change -- a warmer, brighter florist, a small tone shift in the café -- was
	# accepted by the operator on 2026-10-09 and the frames re-baselined on it.
	v.subdiv = VoxelGI.SUBDIV_128
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
	player.look_enabled = not _scripted()
	add_child(player)
	var spawn := depiction.spawn()
	player.place(spawn["position"], spawn["yaw"], -2.0)
	player.set_look(settings_store.settings.look_scale(), settings_store.settings.invert_y)


## The HUD's composed lines are rendered every frame from their keys, so a language change shows at
## once; the world line keeps its last key and arguments and renders again on a change.
func _process(_d: float) -> void:
	if world == null:
		return   # the far field was refused: nothing was built (_load_far_field)
	if first_frame.is_empty():
		var window := get_window()
		first_frame = {"language": MineWorldText.language(), "clock": MineWorldText.clock(),
			"window": [window.size.x, window.size.y], "mode": window.mode, "max_fps": Engine.max_fps,
			"render_scale": window.scaling_3d_scale, "file": settings_store.path,
			"problems": settings_store.problems, "look_scale": player.look_scale,
			"invert_y": player.invert_look_y}
	if _hud_place != null and player != null:
		var p := SliceWorld.place_at(world, player.global_position)
		_hud_place.text = MineWorldText.text("hud.place", {"place": p if p != "" else "-"})
	if _hud_looking != null and link != null:
		_hud_looking.text = _looking_line()
		hud.set_aiming(link.aiming != "")
	if _hud_glance != null and link != null:
		_hud_glance.text = MineWorldAttributes.glance(_aimed_record())
	if _inspect != null and link != null:
		if Input.is_action_pressed("inspect"):
			var aimed := _aimed_person()
			_inspect.present(link.shown_text(aimed) if aimed != "" else MineWorldText.text("hud.inspect.self"),
				SliceInspectPanel.lines_for(link.client.latest, aimed))
		else:
			_inspect.hide_panel()


## The person the aim meets, or "".
func _aimed_person() -> String:
	var a := link.aim()
	return String(a["entity"]) if a["kind"] == "person" else ""


## The perceived record of the person the aim meets, or {} (S30-f: who I see).
func _aimed_record() -> Dictionary:
	var aimed := _aimed_person()
	var obs: MineWorldObservation = link.client.latest if link.client != null else null
	return {} if aimed == "" or obs == null else obs.entity(aimed)


## The key that sends each interaction (step-11 §22 SD-E6, QE-5): an input
## mapping, presentation only. An offer whose action has no key is listed by its
## verb alone.
const OFFER_KEYS := {
	SliceIntents.TALK: "E", SliceIntents.SHOVE: "R", SliceIntents.KICK: "F", SliceIntents.THROW: "G",
}
## A loose object is named by its disclosed shape, the only thing known of it.
const OBJECT_NAMES := { "ball": "hud.object.ball", "box": "hud.object.box" }


## The `looking at` line (SD-E10): whom or what the ray meets, and the server's
## offers for it by key and verb, greyed with the server's reason when it says
## not now. During a throw's aim, what the second G does. Read, never computed:
## nothing here decides what may be sent -- the keys send whatever is offered.
func _looking_line() -> String:
	var a := link.aim()
	if link.aiming != "":
		return MineWorldText.text("hud.aiming")
	var who := ""
	if a["kind"] == "person":
		who = link.shown_text(a["entity"])
	elif a["kind"] == "object":
		who = MineWorldText.text(OBJECT_NAMES.get(link.objects.shape_of(a["entity"]), "hud.object.box"))
	if who == "":
		return MineWorldText.text("hud.looking_at", {"who": "-"})
	var parts := PackedStringArray()
	for o in link.offers(a):
		var args := {"key": OFFER_KEYS.get(o["action"], ""), "verb": SliceIntents.verb(o["action"])}
		if o["available"]:
			parts.append(MineWorldText.text("hud.offer", args))
		else:
			args["reason"] = SliceLink._shown_reason(o["reason"])
			parts.append(MineWorldText.text("hud.offer_not_now", args))
	if parts.is_empty():
		return MineWorldText.text("hud.looking_at", {"who": who})
	return MineWorldText.text("hud.looking_at_offers", {"who": who, "offers": " · ".join(parts)})


func _notification(what: int) -> void:
	if what == NOTIFICATION_TRANSLATION_CHANGED:
		_render_world()
		if _hud_connected != null:
			_hud_connected.text = MineWorldText.text("hint.connected")


func _hud() -> void:
	if not (_scripted() and not (SliceProbeWorld.with_hud() or SliceProbeMarket.requested())):
		hud = ControlsHud.attach(self, player, true)
		# what this depiction is, when it is not the finished world (Q-S33c-1)
		var note := depiction.hud_note()
		if not note.is_empty():
			hud.add_line(MineWorldText.text(note["key"], note["args"]))
		_hud_place = hud.add_line(MineWorldText.text("hud.place", {"place": "-"}))
		_hud_world = hud.add_line("")
		_render_world()
		_needs = SliceNeedsHud.new()
		_needs.name = "NeedsHud"
		add_child(_needs)
		_needs.attach(hud, _env)
	_menu()


## What the packs disclose about me (S35 N-e); null without a HUD.
var _needs: SliceNeedsHud = null


## The connection to a running MineWorld world (`VISUAL_SLICE.md` sec.9), when
## `--server=` is given. Without it the slice runs offline and says so.
var link: SliceLink = null
var _hud_world: Label = null
var _hud_looking: Label = null
## Who I see, under the looking line, and the inspect panel (S30-f §5.9).
var _hud_glance: Label = null
var _inspect: SliceInspectPanel = null
var _hud_time: Label = null
var _hud_connected: Label = null
## The world line's last message, as a key and its arguments ("" while offline).
var _world_key := ""
var _world_args := {}


func _render_world() -> void:
	if _hud_world == null:
		return
	_hud_world.text = MineWorldText.text("hud.world.offline") if _world_key == "" \
		else MineWorldText.text("hud.world", {"status": MineWorldText.text(_world_key, _world_args)})


func _link() -> void:
	var address := SliceLink.address_from_args()
	if address == "":
		return
	link = SliceLink.new()
	link.name = "SliceLink"
	link.player = player
	link.world_root = world
	add_child(link)
	if _needs != null:
		_needs.bind_link(link)
	# Both arrive in display labels (`SliceLink.shown_label`); the link keeps
	# entity ids to its log. A status line is a key and its arguments, worded here.
	link.said.connect(func(key: String, args: Dictionary, notice: bool) -> void:
		_world_key = key
		_world_args = args
		_render_world()
		if hud != null and notice:
			hud.toast(MineWorldText.text(key, args)))
	link.spoke.connect(func(line: String) -> void:
		if hud != null:
			hud.caption(line))
	link.start(address, SliceLink.seat_from_args(), SliceLink.invite_from_args(),
		SliceLink.nickname_from_args())
	# Connected only: the player targets by aiming (`SliceTargeting`), so the
	# screen shows where the aim is and whom it meets (step-15 Q-16a-1, a 3D
	# visual default the operator judges in play). Offline nothing is targeted.
	if hud != null:
		hud.add_reticle()
		_hud_looking = hud.add_line(MineWorldText.text("hud.looking_at", {"who": "-"}))
		_hud_glance = hud.add_line("")
		_hud_glance.name = "Glance"
		_inspect = SliceInspectPanel.new()
		_inspect.name = "InspectPanel"
		add_child(_inspect)
		# The world's date and time, in the clock form the settings choose (S20 SD-SET-a-7, TW-e).
		_hud_time = hud.add_line("")
		_hud_time.name = "Clock"
		# The connected keys (PR 16d, D-16d-4a): a line of their own; the shared controls line is
		# the standalone slice's and is not edited.
		_hud_connected = hud.add_line(MineWorldText.text("hint.connected"))
	clock.attach_client(link.client, _hud_time)
	_offers()


## Your menu and the menu of the person you look at (PR 16d): rebuilt from every observation while
## open, and holding the game through the one gate.
func _offers() -> void:
	offer_menu = SliceOfferMenu.new()
	offer_menu.name = "OfferMenu"
	offer_menu.link = link
	offer_menu.things = link.things
	add_child(offer_menu)
	link.client.observed.connect(offer_menu.refresh)
	offer_menu.opened.connect(func() -> void: _hold_input(true))
	offer_menu.closed.connect(func() -> void: _hold_input(false))
