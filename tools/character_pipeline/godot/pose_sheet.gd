# SPDX-License-Identifier: MIT
# The S26 pose sheet (step-26 §6.4, P-1…P-8): one cast character rendered in fixed poses, at fixed cameras, in the
# 3D client's own lighting, with the client's own body code (`human.gd`, `posture.gd`) and clips.
#
# Never run in clients/3d-spike. `mwcast/pose_sheet.py` builds a scratch project that copies the client files this
# script needs, imports it, and runs (windowed: a headless Godot renders nothing):
#   godot --path <project> --fixed-fps 30 --disable-vsync --script res://pose_sheet.gd -- \
#       --subject res://cast/subject.glb --vera res://assets/characters/meshy_d/meshy_d.glb --out <abs dir>
#       [--label <name>] [--grip] [--plant-seat-gap <m>]
#
# Determinism: every pose is reached by counting frames under `--fixed-fps`, never by wall-clock time. Blinking is
# held open (`blink_hold = 0`) and the reference body's head low-pass is set to 0 (its converged value at a steady
# gait), because `posture.gd` times that filter with `Time.get_ticks_usec()`.
#
# Writes <out>/frames/<id>.png and <out>/report.json. A frame whose clipping check fails is written with a red border.
extends SceneTree

const W := 1000
const H := 1250
const FPS := 30.0
## The seat-contact bound (G-13). The arm–torso check (G-9w) and the ground check (G-4w) are decided in Python
## from the frame dumps (`mwcast/gait.py`, bounds in `contact_tolerances.json`; S36-P2).
const SEAT_MM := 20.0
## W-4's shell: slab height and angular bins (7.5 degrees each) of a garment's outline round a segment (until S36-P3).
const SLAB_M := 0.02
const POLAR_BINS := 48
## The café's loose seating, as `scripts/slice/cafe_interior.gd` `_loose_seating` places it: a round table with a
## dining chair 0.66 m from its centre, facing it.
const CHAIR := "res://assets/models/dining_chair_02/dining_chair_02.gltf"
const TABLE := "res://assets/models/round_wooden_table_02/round_wooden_table_02.gltf"
const CHAIR_FROM_TABLE_M := 0.66
## The knees' clearance in front of the table's base: the café's 0.66 m puts a seated person's shins through this
## table's pedestal cabinet, so the chair moves back until the base is this far ahead of the seat's middle.
const KNEE_CLEARANCE_M := 0.58
## How far above the seat surface the `Hips` joint sits when the body rests on it: the protagonist's (`meshy_d.glb`)
## settled value in the sheet's table pose (`_seated`, 2026-10-15). Each body is settled on its own seat surface and
## its value compared with this one: a body that would float or sink by more than G-13's 20 mm under one shared
## placement rule is flagged. (human.gd `sit(seat_y)` puts the `Hips` joint itself at `seat_y`; in its own `Sit`
## pose that leaves the protagonist's lowest buttock vertex 127 mm under the seat surface.)
const HIP_OVER_SEAT_M := 0.250
## The slice's sun and sky (`scripts/slice/slice_main.gd` `_environment`, `_sun`; `palette.gd` SUN_WARM).
const HDRI := "res://assets/hdri/qwantani_puresky_2k.hdr"
const SUN_ELEVATION := -19.3
const SUN_AZIMUTH := -48.0
const SUN_ENERGY := 4.4
const SUN_WARM := Color(1.0, 0.80, 0.56)
## The subject faces this far round from the sun's azimuth, so its front is lit at three-quarter.
const FACE_FROM_SUN_DEG := 25.0

## The ground probe (PIPELINE.md §15.1) visits rest, the Stand loop and one Walk and one Jog cycle; G-4w is judged
## in Python from the dump. Vertices that can reach the floor: those within this height of the body's lowest rest
## vertex (the feet and the shins). Measured on the rest pose; nothing above the knee comes near the floor in a gait.
const GROUND_SET_M := 0.30
## The frame dump (`--dump-frames <dir>`, MWCF v1, PIPELINE.md §15.1): what the probe skinned, for Python to judge.
const MWCF_VERSION := 1
## The Stand loop writes every vertex in every this-many-th frame and the ground set only in the others.
const STAND_FULL_EVERY := 10
## The stance-lift plant lifts the left foot while its ankle moves backward faster than this share of the gait speed
## (a plant's choice of frames, not a gate's rule; G-4w's planted rule is `mwcast/gait.py`'s).
const STANCE_SHARE := 0.25

const TORSO := ["Spine", "Chest", "UpperChest"]
const StandGates := preload("res://stand_gates.gd")
const SEAT := ["Hips", "LeftUpperLeg", "RightUpperLeg"]

var _args := {}
var _out := ""
var _vp: SubViewport
var _cam: Camera3D
var _stage: Node3D
var _frames: Array = []
var _facing := 0.0
var _subject_stature := 0.0
var _notes: Array[String] = []
## Skeleton -> its bones' global poses as last drawn. `get_bone_global_pose` outside `skeleton_updated` returns the
## pose without the SkeletonModifier3D results (posture.gd's grip and the sheet's own reach), so it is read there.
var _poses := {}
## The subject's ankle height standing, the seated legs' floor goal.
var _ankle_h := 0.09
## Outfit mode (S32-p, `--outfit <manifest.json>`): the parts' layers and covered regions, and the body's
## per-vertex regions. Empty for a single fused export.
var _outfit := {}
var _region_names := PackedStringArray()
## The open dump of the captured frames (pose-sheet mode with `--dump-frames`), else empty.
var _capture_dump := {}
## The plants of S36-P2 commit 4: the floor reference raised by this much in every dumped frame
## (`--plant-sole-offset <m>`), and the left foot raised in stance frames (`--plant-stance-lift <m>`).
var _sole_offset := 0.0
var _stance_lift := 0.0
## W-4: a vertex lying outside a garment's shell by more than this, within its covered regions, pokes through
## (step-32 §7.2 W-4: 2 mm, about the cloth's thickness).
const POKE_MM := 2.0
## W-4's segments: the bone group a vertex is dominated by, the regions that cover it.
const POKE_REGIONS := {"torso": ["torso", "hips"], "upper_arm": ["upper-arms"], "sleeve": ["forearms"],
	"thigh": ["thighs", "hips"], "leg": ["shins"]}


func _initialize() -> void:
	# the bodies are judged as assets: no runtime foot grounding, which would hide a floating planted foot
	# (S36-P4 F-P4-2; G-4w judges the asset, `feet_course.gd` judges the grounding)
	Human.foot_grounding = false
	var raw := OS.get_cmdline_user_args()
	var i := 0
	while i < raw.size():
		var key: String = raw[i]
		if key.begins_with("--") and i + 1 < raw.size() and not (raw[i + 1] as String).begins_with("--"):
			_args[key.substr(2)] = raw[i + 1]
			i += 2
		else:
			_args[key.substr(2)] = "true"
			i += 1
	_out = _args.get("out", "")
	if _out == "" or not (_args.has("subject") or (_args.has("ground-only") and _args.has("bodies"))):
		printerr("pose_sheet: --out and --subject (or --ground-only --bodies <json>) are required")
		quit(2)
		return
	_sole_offset = float(_args.get("plant-sole-offset", "0"))
	_stance_lift = float(_args.get("plant-stance-lift", "0"))
	if _args.has("dump-frames"):
		DirAccess.make_dir_recursive_absolute(_args["dump-frames"])
	if _args.has("outfit"):
		_outfit = JSON.parse_string(FileAccess.get_file_as_string(_args["outfit"]))
		for r in _outfit["regions"]:
			_region_names.append(r)
	DirAccess.make_dir_recursive_absolute(_out.path_join("frames"))
	root.size = Vector2i(320, 240)
	_vp = SubViewport.new()
	_vp.size = Vector2i(W, H)
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	_vp.msaa_3d = Viewport.MSAA_4X
	_vp.screen_space_aa = Viewport.SCREEN_SPACE_AA_DISABLED
	_vp.use_taa = false
	_vp.own_world_3d = true
	root.add_child(_vp)
	_stage = Node3D.new()
	_vp.add_child(_stage)
	_lights()
	_cam = Camera3D.new()
	_vp.add_child(_cam)
	_cam.current = true
	_run.call_deferred()


# --- the stage -------------------------------------------------------------------------------------------------------

func _lights() -> void:
	var env := Environment.new()
	var sky := Sky.new()
	var pano := PanoramaSkyMaterial.new()
	pano.panorama = load(HDRI) as Texture2D
	pano.energy_multiplier = 0.80
	sky.sky_material = pano
	sky.radiance_size = Sky.RADIANCE_SIZE_512
	env.sky = sky
	env.background_mode = Environment.BG_SKY
	env.ambient_light_source = Environment.AMBIENT_SOURCE_SKY
	env.ambient_light_sky_contribution = 1.0
	env.ambient_light_energy = 1.0
	env.reflected_light_source = Environment.REFLECTION_SOURCE_SKY
	env.tonemap_mode = Environment.TONE_MAPPER_AGX
	env.tonemap_exposure = 1.06
	env.tonemap_white = 6.0
	env.ssao_enabled = true
	env.ssao_radius = 0.65
	env.ssao_intensity = 2.0
	env.ssao_power = 1.5
	env.ssao_detail = 1.0
	env.ssao_horizon = 0.06
	env.ssao_light_affect = 0.12
	env.adjustment_enabled = true
	env.adjustment_contrast = 1.06
	env.adjustment_saturation = 1.14
	var we := WorldEnvironment.new()
	we.environment = env
	var ca := CameraAttributesPractical.new()
	ca.auto_exposure_enabled = false
	we.camera_attributes = ca
	_vp.add_child(we)
	var sun := DirectionalLight3D.new()
	sun.light_color = SUN_WARM
	sun.light_energy = SUN_ENERGY
	sun.light_angular_distance = 0.55
	sun.shadow_enabled = true
	sun.directional_shadow_mode = DirectionalLight3D.SHADOW_PARALLEL_2_SPLITS
	sun.directional_shadow_max_distance = 20.0
	sun.shadow_bias = 0.024
	sun.shadow_normal_bias = 2.2
	sun.rotation_degrees = Vector3(SUN_ELEVATION, SUN_AZIMUTH, 0.0)
	_vp.add_child(sun)
	var fill := DirectionalLight3D.new()
	fill.light_color = Color(0.60, 0.71, 0.92)
	fill.light_energy = 0.26
	fill.rotation_degrees = Vector3(-46.0, 118.0, 0.0)
	_vp.add_child(fill)
	# a neutral paving-grey floor, so the feet and the chair have a shadow to stand in
	var floor_mesh := PlaneMesh.new()
	floor_mesh.size = Vector2(40, 40)
	var floor_mat := StandardMaterial3D.new()
	floor_mat.albedo_color = Color(0.46, 0.45, 0.43)
	floor_mat.roughness = 0.9
	var floor := MeshInstance3D.new()
	floor.mesh = floor_mesh
	floor.material_override = floor_mat
	_vp.add_child(floor)
	# The body faces its local +Z (human.gd). Yaw it toward the sun, `FACE_FROM_SUN_DEG` round.
	var to_sun := sun.basis * Vector3(0, 0, 1)
	_facing = atan2(to_sun.x, to_sun.z) + deg_to_rad(FACE_FROM_SUN_DEG)


func _build(path: String, grip: bool) -> Human:
	# `--manifest`: the subject is a cast character, built from its manifest exactly as the client builds it
	# (`Human.build_character`: its clip speeds), its scene the staged export
	if _args.has("manifest") and path == _args.get("subject", ""):
		var m: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(_args["manifest"]))
		m["scene"] = path
		return _ready_body(Human.build_character(m))
	# human.gd caches one PackedScene and one clip library per Body slot; the sheet reuses the REFERENCE slot for
	# whichever body it builds, so the cache is reset first (the stand clips key the body's own rests).
	Human._scenes.erase(Human.Body.REFERENCE)
	Human._libs.erase(Human.Body.REFERENCE)
	Human._scenes[Human.Body.REFERENCE] = load(path) as PackedScene
	var pack := Color(0.24, 0.25, 0.21, 1.0) if grip else Color(0, 0, 0, 0)
	var h := Human.build(Human.CANONICAL_HEIGHT, Color(0.86, 0.70, 0.60), Color(0.30, 0.20, 0.13),
		Color(0.55, 0.20, 0.18), Color(0.22, 0.27, 0.38), Color(0.14, 0.13, 0.12),
		Color(0.36, 0.38, 0.33, 1.0), pack, Human.Body.REFERENCE)
	# A pipeline export names its one material `MW_Cast` (PIPELINE.md §4.1); human.gd binds only `MW_Meshy`, so the
	# same surface response is applied here.
	for mi: MeshInstance3D in h.skeleton.find_children("*", "MeshInstance3D", true, false):
		for s in mi.mesh.get_surface_count():
			var src := mi.mesh.surface_get_material(s)
			# A modular outfit's parts name theirs `MW_Cast_<part>` (S32-p F-S32p-5): the same response.
			if src != null and (src.resource_name == "MW_Cast" or src.resource_name.begins_with("MW_Cast_")) \
					and src is StandardMaterial3D:
				mi.set_surface_override_material(s, Human._meshy(src))
	for mi: MeshInstance3D in h.skeleton.find_children("BodyCheck", "MeshInstance3D", true, false):
		mi.visible = false  # an assembled outfit's W-4 reference, never drawn
	return _ready_body(h)


## The sheet's hold on a built body: lids open, the head filter off, its poses read as drawn.
func _ready_body(h: Human) -> Human:
	h.blink_hold = 0.0
	h.skeleton.skeleton_updated.connect(_on_skeleton_updated.bind(h.skeleton))
	if h._posture != null:
		h._posture.head_tau = 0.0
	return h


func _on_skeleton_updated(sk: Skeleton3D) -> void:
	var poses: Array[Transform3D] = []
	for i in sk.get_bone_count():
		poses.append(sk.get_bone_global_pose(i))
	_poses[sk] = poses


## Every bone's skeleton-space pose as last drawn.
func _drawn(sk: Skeleton3D) -> Array:
	if _poses.has(sk):
		return _poses[sk]
	var poses: Array = []
	for i in sk.get_bone_count():
		poses.append(sk.get_bone_global_pose(i))
	return poses


func _gpose(sk: Skeleton3D, bone: int) -> Transform3D:
	if _poses.has(sk):
		return (_poses[sk] as Array[Transform3D])[bone]
	return sk.get_bone_global_pose(bone)


func _place(h: Human, pos: Vector3, yaw: float) -> void:
	if h.get_parent() == null:
		_stage.add_child(h)
	h.position = pos
	h.rotation = Vector3(0, yaw, 0)


func _frames_n(n: int) -> void:
	for i in n:
		await process_frame


# --- the run ---------------------------------------------------------------------------------------------------------

func _run() -> void:
	if _args.has("ground-only"):
		await _run_ground_only()
		return
	var subject_path: String = _args["subject"]
	var vera_path: String = _args.get("vera", "")
	var grip: bool = _args.get("grip", "false") == "true"
	var s := _build(subject_path, grip)
	_place(s, Vector3.ZERO, _facing)
	if _args.has("plant-arm-into-torso"):
		# acceptance probe (step-26 X-11's shape): the right arm forced through the trunk's side
		var plant := Posture.new()
		plant.name = "PlantedArm"
		plant.rest_relative = {"RightUpperArm": Vector3(118.0, 0, 0), "RightLowerArm": Vector3(0, 0, 0)}
		s.skeleton.add_child(plant)
		_notes.append("PLANTED: right arm forced 28 degrees past hanging, into the trunk (--plant-arm-into-torso)")
	await _frames_n(30)
	# G-9w reads every captured frame from this dump (`mwcast/pose_sheet.py`, after the run)
	if _args.has("dump-frames"):
		_capture_dump = _dump_open(String(_args["dump-frames"]).path_join("capture.mwcf"), s)
	_subject_stature = _skinned(s)["top"]
	print("pose_sheet: subject %s, stature (skinned top) %.3f m" % [subject_path, _subject_stature])
	var stature := _subject_stature
	_ankle_h = minf(_bone_world(s, "LeftFoot").y, _bone_world(s, "RightFoot").y)

	# P-1: idle stand, front / three-quarter / rear; P-8: face at 2.5 m.
	for v in [["P-1_front", 0.0], ["P-1_three-quarter", 35.0], ["P-1_side", 90.0], ["P-1_rear", 180.0]]:
		_frame_full(s, v[1], stature)
		await _capture(v[0], "P-1", "idle stand", s, {})
	var head := _bone_world(s, "Head")
	_cam.fov = 2.0 * rad_to_deg(atan(0.30 / 2.5))
	var face := _facing + deg_to_rad(8.0)
	var eye := Vector3(head.x, head.y + 0.09, head.z)
	_cam.position = eye + Vector3(sin(face), 0, cos(face)) * 2.5
	_cam.look_at(eye, Vector3.UP)
	await _capture("P-8_face", "P-8", "face close-up, 2.5 m", s, {})

	# the quality-tier control (§6.3): Vera beside the subject, same light; then a conversation two-shot (P-6)
	var vera: Human = null
	if vera_path != "":
		vera = _build(vera_path, true)
		var side := Vector3(cos(_facing), 0, -sin(_facing))
		_place(vera, side * -0.85, _facing)
		_place(s, side * 0.0, _facing)
		await _frames_n(20)
		_cam.fov = 30.0
		var mid := side * -0.425
		var d := (stature * 0.62) / tan(deg_to_rad(15.0)) + 0.6
		_cam.position = mid + Vector3(sin(_facing), 0, cos(_facing)) * d + Vector3(0, stature * 0.55, 0)
		_cam.look_at(mid + Vector3(0, stature * 0.50, 0), Vector3.UP)
		await _capture("P-1_lineup-with-vera", "P-1", "idle stand beside Vera (quality-tier control), front", s, {})
		# P-6: facing each other at 1.1 m, a three-quarter camera on the subject's face
		var fwd := Vector3(sin(_facing), 0, cos(_facing))
		_place(vera, fwd * 1.1, _facing + PI)
		_place(s, Vector3.ZERO, _facing)
		await _frames_n(20)
		var a := _facing + deg_to_rad(62.0)
		_cam.fov = 32.0
		_cam.position = fwd * 0.55 + Vector3(sin(a), 0, cos(a)) * 4.8 + Vector3(0, 1.35, 0)
		_cam.look_at(fwd * 0.55 + Vector3(0, stature * 0.52, 0), Vector3.UP)
		await _capture("P-6_two-shot-with-vera", "P-6", "conversation two-shot with Vera (idle; no gesture clip)", s, {})
		if _args.has("pair"):
			# S32-p: the same person in outfit A (the subject) and outfit B, beside the fused Vera, front, same light
			var pair := _build(_args["pair"], false)
			var side2 := Vector3(cos(_facing), 0, -sin(_facing))
			_place(vera, side2 * -0.85, _facing)
			_place(s, Vector3.ZERO, _facing)
			_place(pair, side2 * 0.85, _facing)
			await _frames_n(20)
			_cam.fov = 30.0
			var d2 := maxf((stature * 0.62) / tan(deg_to_rad(15.0)) + 0.6, 1.5 / tan(deg_to_rad(15.0)) + 0.6)
			_cam.position = Vector3(sin(_facing), 0, cos(_facing)) * d2 + Vector3(0, stature * 0.55, 0)
			_cam.look_at(Vector3(0, stature * 0.50, 0), Vector3.UP)
			await _capture("outfit-pair_A-B-vera", "pair", "fused Vera | outfit A | outfit B, front", s, {})
			pair.visible = false
		vera.visible = false

	# P-2: walk, contact and passing; P-3: jog, contact. Treadmill: the body stays put, the clip plays at its speed.
	_place(s, Vector3.ZERO, _facing)
	# each clip at its blend point: the body's own speeds (a cast manifest's `clips`, else the reference's)
	for g in [["Walk", s._walk_mps, "P-2", "walk"], ["Jog_Fwd", s._jog_mps, "P-3", "jog"]]:
		s.set_gait(g[1] * s.scale.y)
		var length: float = s._player().get_animation(g[0]).length
		var period := roundi(length * FPS)
		await _frames_n(period)
		var seps: Array[float] = []
		for k in period:
			# sampled at the same point of the frame `_capture` reads, so index k is the frame it would capture
			await process_frame
			await RenderingServer.frame_post_draw
			seps.append(_stride(s))
		var k_contact := 0
		var k_pass := 0
		for k in period:
			if absf(seps[k]) > absf(seps[k_contact]):
				k_contact = k
			if absf(seps[k]) < absf(seps[k_pass]):
				k_pass = k
		_notes.append("%s: period %d frames (clip %.4f s); contact at frame %d (feet %.3f m apart), passing at %d (%.3f m)"
			% [g[0], period, length, k_contact, absf(seps[k_contact]), k_pass, absf(seps[k_pass])])
		var shots := [[k_contact, "%s_%s-contact" % [g[2], g[3]], "%s, contact" % g[3]]]
		if g[2] == "P-2":
			shots.append([k_pass, "%s_%s-passing" % [g[2], g[3]], "%s, passing" % g[3]])
		shots.sort_custom(func(x: Array, y: Array) -> bool: return x[0] < y[0])
		var at := 0
		for shot in shots:
			# the same phase one cycle on; `_capture` waits for the draw of the frame after this one
			await _frames_n(int(shot[0]) - at)
			at = int(shot[0]) + 1
			_frame_full(s, 90.0, stature)
			await _capture(shot[1], g[2], "%s, side" % shot[2], s, {})
		await _frames_n(period - at)
	s.set_gait(0.0)

	# the ground contact (§15.1): rest, the Stand loop, one Walk and one Jog cycle, every frame
	var gdump := {}
	if _args.has("dump-frames"):
		gdump = _dump_open(String(_args["dump-frames"]).path_join("ground.mwcf"), s)
	var ground: Dictionary = await _ground(s, s._walk_mps * s.scale.y, s._jog_mps * s.scale.y, null, gdump)
	_dump_close(gdump)
	print("pose_sheet: ground %s" % JSON.stringify(ground))
	s.set_gait(0.0)
	await _frames_n(10)

	# P-4: on the café's dining chair at its round table, hands on the table.
	await _seated(s, stature)

	var report := {
		"format": 1,
		"subject": subject_path,
		"label": _args.get("label", subject_path.get_file().get_basename()),
		"vera": vera_path,
		"stature_skinned_m": snappedf(stature, 0.001),
		"bounds": {"seat_contact_mm": SEAT_MM, "poke_through_mm": POKE_MM},
		"outfit": _outfit.get("name", null),
		"pair": _args.get("pair", null),
		"godot": Engine.get_version_info()["string"],
		"renderer": RenderingServer.get_video_adapter_name(),
		"fps": FPS,
		"notes": _notes,
		"frames": _frames,
		"ground": ground,
	}
	_dump_close(_capture_dump)
	var f := FileAccess.open(_out.path_join("report.json"), FileAccess.WRITE)
	f.store_string(JSON.stringify(report, "  ", false) + "\n")
	f.close()
	var failed := 0
	for fr: Dictionary in _frames:
		if not fr["pass"]:
			failed += 1
	print("pose_sheet: %d frames, %d flagged; %s" % [_frames.size(), failed, _out.path_join("report.json")])
	quit(0)


func _seated(s: Human, stature: float) -> void:
	var table := (load(TABLE) as PackedScene).instantiate() as Node3D
	var chair := (load(CHAIR) as PackedScene).instantiate() as Node3D
	_stage.add_child(table)
	_stage.add_child(chair)
	var fwd := Vector3(sin(_facing), 0, cos(_facing))
	# the table: centred on its footprint at the floor, in front of the chair
	var tb := _vertex_bounds(table, func(_p: Vector3) -> bool: return true)
	var base := _vertex_bounds(table, func(p: Vector3) -> bool: return p.y < tb.end.y - 0.12)
	var base_half := maxf(base.size.x, base.size.z) * 0.5
	var radius := maxf(tb.size.x, tb.size.z) * 0.5
	var chair_at := Vector3.ZERO
	var dist := maxf(CHAIR_FROM_TABLE_M, base_half + KNEE_CLEARANCE_M)
	_notes.append("P-4: table base half-width %.3f m, top radius %.3f m; chair %.3f m from the table centre (café: %.2f)"
		% [base_half, radius, dist, CHAIR_FROM_TABLE_M])
	var table_at := chair_at + fwd * dist
	table.position = table_at - Vector3(tb.get_center().x, tb.position.y, tb.get_center().z)
	# the chair: its back (the vertices above 0.65 m) away from the table
	var cb := _vertex_bounds(chair, func(_p: Vector3) -> bool: return true)
	var back := _vertex_bounds(chair, func(p: Vector3) -> bool: return p.y > cb.position.y + 0.65)
	var back_dir := Vector3(back.get_center().x - cb.get_center().x, 0, back.get_center().z - cb.get_center().z)
	var chair_front_local := atan2(-back_dir.x, -back_dir.z)
	chair.rotation.y = _facing - chair_front_local
	var cbw := _xf_aabb(Transform3D(chair.basis, Vector3.ZERO), cb)
	chair.position = chair_at - Vector3(cbw.get_center().x, cbw.position.y, cbw.get_center().z)
	# 45 frames: their materials' pipelines compile asynchronously on first sight, and a frame drawn before that ends
	# differs between runs
	await _frames_n(45)
	# the seat: the top of the chair's geometry over a 16 cm square at the seat's middle (bounds' middle moved 6 cm
	# toward the table, as streetscape.gd `_chair_seat` does), measured from the asset's vertices
	var seat_mid := chair_at + fwd * 0.06
	var seat_top := -INF
	var seat_rect := Rect2()
	var first := true
	for p: Vector3 in _vertices_world(chair):
		var dx := p.x - seat_mid.x
		var dz := p.z - seat_mid.z
		if absf(dx) < 0.08 and absf(dz) < 0.08 and p.y < 0.65:
			seat_top = maxf(seat_top, p.y)
	# the seat panel's footprint: chair vertices within 4 cm below the seat top, in the body frame (x side, y forward)
	for p: Vector3 in _vertices_world(chair):
		if p.y > seat_top - 0.04 and p.y <= seat_top + 0.001:
			var q := _body_xy(p - chair_at)
			if first:
				seat_rect = Rect2(q, Vector2.ZERO)
				first = false
			else:
				seat_rect = seat_rect.expand(q)
	var table_top_w := _vertex_bounds_world(table).end.y
	_notes.append("P-4: dining_chair_02 seat top %.3f m (measured), round_wooden_table_02 top %.3f m (measured)"
		% [seat_top, table_top_w])
	_place(s, seat_mid, _facing)
	var seat_info := {"seat_top": seat_top, "rect": seat_rect, "chair_at": chair_at, "table_top": table_top_w}
	# Seated on its own seat surface: from the protagonist's hips-over-seat height, the hips move until the lowest
	# buttock-or-thigh vertex over the seat touches it (three corrections; the table pose is re-solved each time,
	# since the legs' solution moves with the hips).
	var off := HIP_OVER_SEAT_M
	var reach: Posture = null
	var err := 0.0
	for it in 4:
		if reach != null:
			s.skeleton.remove_child(reach)
			reach.free()
		s.sit(seat_top + off)
		await _frames_n(6)
		reach = _table_pose(s, fwd, seat_top, table_at, chair_at, radius, table_top_w, it == 3)
		s.skeleton.add_child(reach)
		await _frames_n(4)
		var data := _skinned(s)
		err = _seat_low(s, data["p"], data["bone"], seat_info) - seat_top
		if it < 3:
			off -= err
	seat_info["hip_over_seat"] = off
	_notes.append("P-4: seated with the hips joint %.3f m over the seat (protagonist %.3f m); residual %+.1f mm"
		% [off, HIP_OVER_SEAT_M, err * 1000.0])
	var gap := float(_args.get("plant-seat-gap", "0"))
	if gap != 0.0:
		s.position.y += gap
		_notes.append("P-4: PLANTED seat gap %+.3f m (--plant-seat-gap, acceptance X-26)" % gap)
	await _frames_n(4)
	var a := _facing + deg_to_rad(40.0)
	_cam.fov = 34.0
	var look := seat_mid + fwd * 0.30 + Vector3(0, 0.80, 0)
	_cam.position = look + Vector3(sin(a), 0, cos(a)) * 3.1 + Vector3(0, 0.75, 0)
	_cam.look_at(look, Vector3.UP)
	await _capture("P-4_seated-three-quarter", "P-4", "seated on the café dining chair at its table, hands on the table",
		s, seat_info)
	a = _facing + deg_to_rad(90.0)
	_cam.fov = 30.0
	look = seat_mid + fwd * 0.25 + Vector3(0, 0.65, 0)
	_cam.position = look + Vector3(sin(a), 0, cos(a)) * 3.3 + Vector3(0, 0.10, 0)
	_cam.look_at(look, Vector3.UP)
	await _capture("P-4_seated-side", "P-4", "seated, side (seat contact)", s, seat_info)


## The table pose, solved on this body, all rest-relative (posture.gd's convention; the axes are human.gd
## `stand_key`'s), returned as a Posture to layer after the `Sit` clip.
func _table_pose(s: Human, fwd: Vector3, seat_top: float, table_at: Vector3, chair_at: Vector3, radius: float,
		table_top_w: float, note: bool) -> Posture:
	# The table pose, solved on this body, all rest-relative (posture.gd's convention; the axes are human.gd
	# `stand_key`'s): 1. the pelvis turned so the trunk (hips to neck) leans 10 degrees forward of vertical, sitting up
	# to the table; 2. each leg so its ankle rests on the floor below the knee; 3. each arm so the wrist lies on the
	# table top 5 cm in from its near edge. The `Sit` clip's own trunk reclines and its feet hang (human.gd).
	var sk := s.skeleton
	var left_axis := Vector3(cos(_facing), 0, -sin(_facing))
	var over := {}
	var rr := {}
	var hips_i := sk.find_bone("Hips")
	var neck_i := sk.find_bone("Neck")
	var goal := (Vector3.UP * cos(deg_to_rad(10.0)) + fwd * sin(deg_to_rad(10.0))).normalized()
	var trunk := _search(over, [[-40.0, 40.0, 1.0]],
		func(e: Array) -> Dictionary: return {hips_i: _rr(sk, hips_i, Vector3(e[0], 0, 0))},
		func(o: Dictionary) -> float:
			var d := (sk.global_transform.basis * (_g(sk, neck_i, o).origin - _g(sk, hips_i, o).origin)).normalized()
			return d.angle_to(goal))
	over[hips_i] = _rr(sk, hips_i, Vector3(trunk[0][0], 0, 0))
	rr["Hips"] = Vector3(trunk[0][0], 0, 0)
	if note:
		_notes.append("P-4: trunk upright by Hips X %+.2f deg (residual %.2f deg)" % [trunk[0][0], rad_to_deg(trunk[1])])
	for side in ["Left", "Right"]:
		var ul := sk.find_bone(side + "UpperLeg")
		var ll := sk.find_bone(side + "LowerLeg")
		var ft := sk.find_bone(side + "Foot")
		var hip := sk.global_transform * _g(sk, ul, over).origin
		var thigh := (_g(sk, ll, over).origin - _g(sk, ul, over).origin).length()
		var foot_goal := Vector3(hip.x, _ankle_h, hip.z) + fwd * (thigh * 0.95)
		var sgn := 1.0 if side == "Left" else -1.0
		foot_goal += left_axis * sgn * 0.03
		var leg := _search(over, [[40.0, 110.0, 5.0], [-15.0, 15.0, 5.0], [30.0, 130.0, 5.0]],
			func(e: Array) -> Dictionary: return {ul: _rr(sk, ul, Vector3(e[0], 0, e[1])), ll: _rr(sk, ll, Vector3(e[2], 0, 0))},
			func(o: Dictionary) -> float:
				var foot := sk.global_transform * _g(sk, ft, o).origin
				var knee := sk.global_transform * _g(sk, ll, o).origin
				# the thigh about level, as a thigh lies on a seat of the right height
				return foot.distance_to(foot_goal) + 0.5 * maxf(0.0, seat_top + 0.03 - knee.y) \
					+ 0.2 * absf(knee.y - hip.y))
		over[ul] = _rr(sk, ul, Vector3(leg[0][0], 0, leg[0][1]))
		over[ll] = _rr(sk, ll, Vector3(leg[0][2], 0, 0))
		rr[side + "UpperLeg"] = Vector3(leg[0][0], 0, leg[0][1])
		rr[side + "LowerLeg"] = Vector3(leg[0][2], 0, 0)
		if note:
			_notes.append("P-4: %s ankle %.1f mm from the floor goal" % [side, leg[1] * 1000.0])
	var edge := (table_at - chair_at).dot(fwd) - radius
	for side in ["Left", "Right"]:
		var ua := sk.find_bone(side + "UpperArm")
		var la := sk.find_bone(side + "LowerArm")
		var ha := sk.find_bone(side + "Hand")
		var sh := sk.global_transform * _g(sk, ua, over).origin
		var sgn := 1.0 if side == "Left" else -1.0
		# the wrist 5 cm in from the table's near edge, 10 cm inside the shoulder, at the top plus half a hand
		var target := Vector3(sh.x, table_top_w + 0.06, sh.z) + fwd * (edge + 0.05 - (sh - chair_at).dot(fwd)) \
			- left_axis * sgn * 0.10
		var mid := sk.global_transform * _g(sk, hips_i, over).origin
		var arm := _search(over, [[0.0, 96.0, 6.0], [-60.0, 60.0, 15.0], [0.0, 108.0, 6.0], [0.0, 144.0, 8.0]],
			func(e: Array) -> Dictionary: return {ua: _rr(sk, ua, Vector3(e[0], e[1], sgn * e[2])),
				la: _rr(sk, la, Vector3(e[3], 0, 0))},
			func(o: Dictionary) -> float:
				var hand := sk.global_transform * _g(sk, ha, o).origin
				var elbow := sk.global_transform * _g(sk, la, o).origin
				# the elbow below the shoulder and out to the side, never tucked into the trunk
				var out_side := absf((elbow - mid).dot(left_axis)) - absf((sh - mid).dot(left_axis))
				return hand.distance_to(target) + 0.5 * maxf(0.0, elbow.y - (sh.y - 0.12)) \
					+ 0.5 * maxf(0.0, -out_side))
		var e: Array = arm[0]
		rr[side + "UpperArm"] = Vector3(e[0], e[1], sgn * e[2])
		rr[side + "LowerArm"] = Vector3(e[3], 0, 0)
		if note:
			_notes.append("P-4: %s wrist %.1f mm from its table goal" % [side, arm[1] * 1000.0])
	var reach := Posture.new()
	reach.name = "TablePose"
	reach.rest_relative = rr
	return reach


## A bone's skeleton-space transform from the tree's local poses, with `over` (bone -> local rotation) replacing some.
func _g(sk: Skeleton3D, bone: int, over: Dictionary) -> Transform3D:
	var q: Quaternion = over[bone] if over.has(bone) else sk.get_bone_pose_rotation(bone)
	var t := Transform3D(Basis(q), sk.get_bone_pose_position(bone))
	var p := sk.get_bone_parent(bone)
	return t if p < 0 else _g(sk, p, over) * t


func _rr(sk: Skeleton3D, bone: int, e: Vector3) -> Quaternion:
	return sk.get_bone_rest(bone).basis.get_rotation_quaternion() * Human.euler_q(e)


## Minimise `cost(over + make(e))` over the parameter grid `ranges` ([lo, hi, step] each), then refine by coordinate
## descent. Deterministic. Returns [best parameters, best cost].
func _search(base: Dictionary, ranges: Array, make: Callable, cost: Callable) -> Array:
	var n := ranges.size()
	var cur: Array = []
	for r: Array in ranges:
		cur.append(float(r[0]))
	var best: Array = cur.duplicate()
	var best_c := INF
	while true:
		var o := base.duplicate()
		o.merge(make.call(cur), true)
		var c: float = cost.call(o)
		if c < best_c:
			best_c = c
			best = cur.duplicate()
		var j := 0
		while j < n:
			cur[j] += float(ranges[j][2])
			if cur[j] <= float(ranges[j][1]) + 1e-6:
				break
			cur[j] = float(ranges[j][0])
			j += 1
		if j == n:
			break
	for step: float in [2.0, 1.0, 0.5, 0.25]:
		var improved := true
		while improved:
			improved = false
			for j in n:
				for d: float in [-1.0, 1.0]:
					var e: Array = best.duplicate()
					e[j] += d * step
					var o := base.duplicate()
					o.merge(make.call(e), true)
					var c: float = cost.call(o)
					if c < best_c - 1e-7:
						best_c = c
						best = e
						improved = true
	return [best, best_c]


# --- the ground (PIPELINE.md §15.1) ----------------------------------------------------------------------------------

## The fast mode: no frames, no Vera beside the subject, no chair. `--bodies <json>` lists the bodies to measure,
## each `{"id": <id>, "manifest": <character.yaml path>}` (built by `Human.build_character`, exactly as the client
## builds a cast person) or `{"id": <id>, "reference": true}` (the protagonist on the `REFERENCE` slot, as the client
## builds her). Writes `<out>/ground.json`: one ground block per body.
func _run_ground_only() -> void:
	var listing: Variant = JSON.parse_string(FileAccess.get_file_as_string(_args["bodies"]))
	if typeof(listing) != TYPE_ARRAY:
		printerr("pose_sheet: --bodies is not a JSON array")
		quit(2)
		return
	var rows: Array = []
	for entry: Dictionary in listing:
		var h: Human
		var walk := Human.WALK_CLIP_MPS
		var jog := Human.JOG_CLIP_MPS
		if entry.get("reference", false):
			Human._scenes.erase(Human.Body.REFERENCE)
			Human._libs.erase(Human.Body.REFERENCE)
			h = Human.build(Human.CANONICAL_HEIGHT, Color(0.86, 0.70, 0.60), Color(0.30, 0.20, 0.13),
				Color(0.55, 0.20, 0.18), Color(0.22, 0.27, 0.38), Color(0.14, 0.13, 0.12),
				Color(0.36, 0.38, 0.33, 1.0), Color(0.24, 0.25, 0.21, 1.0), Human.Body.REFERENCE)
		else:
			var m: Variant = JSON.parse_string(FileAccess.get_file_as_string(String(entry["manifest"])))
			h = Human.build_character(m)
			walk = float(m["clips"]["walk_mps"])
			jog = float(m["clips"]["jog_mps"])
		h.blink_hold = 0.0
		if h._posture != null:
			h._posture.head_tau = 0.0
		h.skeleton.skeleton_updated.connect(_on_skeleton_updated.bind(h.skeleton))
		_place(h, Vector3.ZERO, 0.0)
		await _frames_n(30)
		# the standing-pose gates (§15.2) on every frame of the Stand loop; the protagonist's strap hand is held by
		# her grip, not by the clip, so only her free arm is gated (her index entry's `grip` names the held side)
		var gates := StandGates.new(h.skeleton, h, entry.get("pack_core", {}))
		var free: Array = ["Left", "Right"]
		free.erase(String(entry.get("grip", "")))
		_dump = {} if _args.has("dump-pose") else null
		var frames_dump := {}
		if _args.has("dump-frames"):
			frames_dump = _dump_open(String(_args["dump-frames"]).path_join("%s.mwcf" % entry["id"]), h)
		var g: Dictionary = await _ground(h, walk * h.scale.y, jog * h.scale.y, gates, frames_dump)
		_dump_close(frames_dump)
		if _dump != null:
			_dump.erase("_sep_a")
			_dump.erase("_sep_b")
			_dump["clip"] = {"stand": "stand_lib.glb Stand (100STYLE Neutral, frame 0)", "stride": "quaternius_ual.glb Walk (largest foot separations)"}
			var df := FileAccess.open(String(_args["dump-pose"]).path_join("%s.json" % entry["id"]), FileAccess.WRITE)
			df.store_string(JSON.stringify(_dump) + "\n")
			df.close()
		var sg: Dictionary = gates.block(free)
		g["arm_clearance_mm"] = [sg["sides"]["Left"]["clearance_mm"], sg["sides"]["Right"]["clearance_mm"]]
		g["stand_gates"] = sg
		g["id"] = entry["id"]
		rows.append(g)
		print("pose_sheet: ground %s %s" % [entry["id"], JSON.stringify(g)])
		_poses.erase(h.skeleton)
		h.queue_free()
		await _frames_n(2)
	var out := {"format": 1, "godot": Engine.get_version_info()["string"], "fps": FPS,
		"plants": {"sole_offset_m": _sole_offset, "stance_lift_m": _stance_lift}, "bodies": rows}
	var f := FileAccess.open(_out.path_join("ground.json"), FileAccess.WRITE)
	f.store_string(JSON.stringify(out, "  ", false) + "\n")
	f.close()
	quit(0)


## The vertices that can touch the floor, prepared once: per skinned surface, those within `GROUND_SET_M` of the
## lowest rest vertex, with their bones (skeleton indices), weights and the side of their dominant bone.
func _ground_set(h: Human) -> Array:
	var sk := h.skeleton
	var rest: Array[Transform3D] = []
	for i in sk.get_bone_count():
		rest.append(sk.get_bone_global_rest(i))
	var all: Array = []
	var floor_y := INF
	for mi: MeshInstance3D in sk.find_children("*", "MeshInstance3D", true, false):
		if not mi.visible or mi.skin == null:
			continue
		var skin := mi.skin
		var bones := PackedInt32Array()
		var binds: Array[Transform3D] = []
		for b in skin.get_bind_count():
			var bone := skin.get_bind_bone(b)
			if bone < 0:
				bone = sk.find_bone(skin.get_bind_name(b))
			bones.append(bone)
			binds.append(skin.get_bind_pose(b))
		for s in mi.mesh.get_surface_count():
			var arr := mi.mesh.surface_get_arrays(s)
			var v: PackedVector3Array = arr[Mesh.ARRAY_VERTEX]
			var bi: PackedInt32Array = arr[Mesh.ARRAY_BONES]
			var bw: PackedFloat32Array = arr[Mesh.ARRAY_WEIGHTS]
			var per := bi.size() / maxi(v.size(), 1)
			var ys := PackedFloat32Array()
			for i in v.size():
				var p := Vector3.ZERO
				for k in per:
					var w := bw[i * per + k]
					if w > 0.0:
						var b := bi[i * per + k]
						p += (rest[bones[b]] * binds[b] * v[i]) * w
				ys.append(p.y)
				floor_y = minf(floor_y, p.y)
			all.append([v, bi, bw, per, bones, binds, ys])
	var out: Array = []
	for e: Array in all:
		var v: PackedVector3Array = e[0]
		var bi: PackedInt32Array = e[1]
		var bw: PackedFloat32Array = e[2]
		var per: int = e[3]
		var bones: PackedInt32Array = e[4]
		var ys: PackedFloat32Array = e[6]
		var kv := PackedVector3Array()
		var kb := PackedInt32Array()
		var kw := PackedFloat32Array()
		var side := PackedInt32Array()
		var dom := PackedInt32Array()
		for i in v.size():
			if ys[i] > floor_y + GROUND_SET_M:
				continue
			kv.append(v[i])
			var best := -1.0
			var best_b := 0
			for k in per:
				kb.append(bones[bi[i * per + k]])
				kw.append(bw[i * per + k])
				if bw[i * per + k] > best:
					best = bw[i * per + k]
					best_b = bones[bi[i * per + k]]
			var name := sk.get_bone_name(best_b)
			dom.append(best_b)
			side.append(0 if name.begins_with("Left") else (1 if name.begins_with("Right") else 2))
		if kv.size() > 0:
			# the bind poses by skeleton bone: a surface's binds index its skin, its kept bones the skeleton
			var by_bone := {}
			for b in bones.size():
				by_bone[bones[b]] = e[5][b]
			out.append([kv, kb, kw, per, side, by_bone, dom])
	return out


## The lowest skinned vertex above the body's root, overall and per side (left, right), in metres, under the
## skeleton-space bone poses `poses` (Σ w · pose · bind, as `_skinned`).
func _ground_low(h: Human, gset: Array, poses: Array) -> Array:
	var sk := h.skeleton
	var base := h.global_position.y
	var low := [INF, INF, INF, -1]
	for e: Array in gset:
		var kv: PackedVector3Array = e[0]
		var kb: PackedInt32Array = e[1]
		var kw: PackedFloat32Array = e[2]
		var per: int = e[3]
		var side: PackedInt32Array = e[4]
		var by_bone: Dictionary = e[5]
		var dom: PackedInt32Array = e[6]
		var mats := {}
		for bone: int in by_bone:
			mats[bone] = sk.global_transform * (poses[bone] as Transform3D) * (by_bone[bone] as Transform3D)
		for i in kv.size():
			var p := Vector3.ZERO
			for k in per:
				var w := kw[i * per + k]
				if w > 0.0:
					p += ((mats[kb[i * per + k]] as Transform3D) * kv[i]) * w
			var y := p.y - base
			var sd := side[i]
			if y < low[0]:
				low[0] = y
				low[3] = dom[i]
			if sd < 2 and y < low[sd + 1]:
				low[sd + 1] = y
	return low


func _mm(m: float) -> float:
	return snappedf(m * 1000.0, 0.1)


## The ground diagnosis of one body, and its frames for G-4w: rest, the `Stand` loop, and every frame of one `Walk`
## and one `Jog_Fwd` cycle (each played at its clip speed, so at rate 1). The numbers here are Godot's own reading
## of the lowest vertex (kept so the dump can be checked against them); the verdict is Python's, from the frames
## written to `dump` (`mwcast/gait.py`, PIPELINE.md §15.1). `dump` is empty when no dump is written; `gates`, when
## given, are the standing-pose gates (§15.2), fed every Stand frame.
func _ground(h: Human, walk_mps: float, jog_mps: float, gates: RefCounted = null, dump: Dictionary = {}) -> Dictionary:
	var sk := h.skeleton
	var gset := _ground_set(h)
	var rest: Array[Transform3D] = []
	for i in sk.get_bone_count():
		rest.append(sk.get_bone_global_rest(i))
	var rest_low: float = _ground_low(h, gset, rest)[0]
	_dump_frame(dump, "rest:0", true, rest)
	var hips := sk.find_bone("Hips")
	var out := {
		"rest_mm": _mm(rest_low),
		"motion_scale": snappedf(sk.motion_scale, 0.0001),
		"rest_hips_m": snappedf((sk.global_transform * rest[hips]).origin.y - h.global_position.y, 0.0001),
		"rest_ankle_m": snappedf((sk.global_transform * rest[sk.find_bone("LeftFoot")]).origin.y
			- h.global_position.y, 0.0001),
	}
	var lift_plant: StanceLift = null
	if _stance_lift != 0.0:
		lift_plant = StanceLift.new()
		lift_plant.name = "PlantedStanceLift"
		lift_plant.body = h
		lift_plant.lift = _stance_lift
		sk.add_child(lift_plant)
	h.set_gait(0.0)
	await _frames_n(4)
	var n := roundi(Human._stand_clip().length * FPS)
	var lo := INF
	var hi := -INF
	var hips_lo := INF
	var lifts := PackedFloat32Array()
	for k in n:
		await process_frame
		var poses: Array = _poses[sk]
		var y: float = _ground_low(h, gset, poses)[0]
		lo = minf(lo, y)
		hi = maxf(hi, y)
		hips_lo = minf(hips_lo, (sk.global_transform * (poses[hips] as Transform3D)).origin.y - h.global_position.y)
		lifts.append(_last_lift(h))
		_dump_frame(dump, "stand:%d" % k, k % STAND_FULL_EVERY == 0, poses)
		if gates != null:
			gates.frame(poses, k)
		if k == 0 and _dump != null:
			_dump["stand"] = _deltas(h, poses)
	out["stand"] = {"min_mm": _mm(lo), "max_mm": _mm(hi), "hips_m": snappedf(hips_lo, 0.0001), "frames": n,
		"lift_mm": _lift_stats(lifts)}
	for g in [["walk", "Walk", walk_mps], ["jog", "Jog_Fwd", jog_mps]]:
		h.set_gait(g[2])
		var period := roundi((h._player().get_animation(g[1]).length) * FPS)
		await _frames_n(period)
		var g_lo := INF
		var g_lo_at := 0
		var g_lo_bone := ""
		var frame_hi := -INF
		var feet := [INF, INF]
		var h_lo := INF
		var h_hi := -INF
		lifts = PackedFloat32Array()
		for k in period:
			await process_frame
			var poses: Array = _poses[sk]
			var low := _ground_low(h, gset, poses)
			if low[0] < g_lo:
				g_lo = low[0]
				g_lo_at = k
				g_lo_bone = sk.get_bone_name(low[3]) if low[3] >= 0 else ""
			frame_hi = maxf(frame_hi, low[0])
			feet[0] = minf(feet[0], low[1])
			feet[1] = minf(feet[1], low[2])
			var hy := (sk.global_transform * (poses[hips] as Transform3D)).origin.y - h.global_position.y
			h_lo = minf(h_lo, hy)
			h_hi = maxf(h_hi, hy)
			lifts.append(_last_lift(h))
			_dump_frame(dump, "%s:%d" % [g[0], k], true, poses)
			if g[0] == "walk" and _dump != null:
				# the stride frames for the 2D sprites: the largest left-foot-forward and right-foot-forward separations
				var sep := ((poses[sk.find_bone("LeftFoot")] as Transform3D).origin - (poses[sk.find_bone("RightFoot")] as Transform3D).origin).z
				if sep > float(_dump.get("_sep_a", -INF)):
					_dump["_sep_a"] = sep
					_dump["stride_a"] = _deltas(h, poses)
				if sep < float(_dump.get("_sep_b", INF)):
					_dump["_sep_b"] = sep
					_dump["stride_b"] = _deltas(h, poses)
		out[g[0]] = {"min_mm": _mm(g_lo), "min_at_frame": g_lo_at, "min_bone": g_lo_bone,
			"frame_low_max_mm": _mm(frame_hi),
			"foot_contact_mm": [_mm(feet[0]), _mm(feet[1])], "hips_m": [snappedf(h_lo, 0.0001), snappedf(h_hi, 0.0001)],
			"frames": period, "lift_mm": _lift_stats(lifts)}
	h.set_gait(0.0)
	if lift_plant != null:
		out["planted_stance_lift_frames"] = lift_plant.lifted
		sk.remove_child(lift_plant)
		lift_plant.free()
	return out


## `GroundContact.last_lift` (metres) of the frame just drawn, 0 where the body has none (§6 of the S36-P2 design:
## reported, not judged; S36-P4 decides a bound).
func _last_lift(h: Human) -> float:
	return h._ground.last_lift if h._ground != null else 0.0


func _lift_stats(lifts: PackedFloat32Array) -> Dictionary:
	var top := 0.0
	var total := 0.0
	for v in lifts:
		top = maxf(top, absf(v))
		total += absf(v)
	return {"max_mm": _mm(top), "mean_mm": _mm(total / maxf(lifts.size(), 1.0))}


## The stance-lift plant (S36-P2 commit 4, `--plant-stance-lift <m>`): after every other modifier, the left `Foot`
## is raised by `lift` metres in each frame the gait calls stance, read from the foot's own speed against the body's
## gait speed (G-8's planted rule; standing, every frame). A gate that judges planted frames must then fail the left
## foot only, and only where it is planted.
class StanceLift extends SkeletonModifier3D:
	var body: Human
	var lift := 0.0
	var lifted := 0
	var _prev := Vector3.INF

	func _process_modification() -> void:
		var sk := get_skeleton()
		if sk == null or body == null:
			return
		var foot := sk.find_bone("LeftFoot")
		if foot < 0:
			return
		var pose := sk.get_bone_global_pose(foot)
		var at := sk.global_transform * pose.origin
		var v: float = body._speed
		var planted := v < 1e-3
		if not planted and _prev != Vector3.INF:
			var fwd := body.global_transform.basis.z
			fwd = Vector3(fwd.x, 0.0, fwd.z).normalized()
			# moving backward with the treadmill floor: stance, and a few early-swing frames, which the gate does
			# not judge (the ankle rolls over the foot in stance, so its speed alone does not mark stance exactly)
			planted = (at - _prev).dot(fwd) * FPS < -STANCE_SHARE * v
		_prev = at
		if not planted:
			return
		pose.origin += sk.global_transform.basis.inverse() * Vector3(0.0, lift, 0.0)
		sk.set_bone_global_pose(foot, pose)
		lifted += 1


# --- the frame dump (MWCF v1, PIPELINE.md §15.1) ---------------------------------------------------------------------

## Open a dump of `h` at `path`: the header, the bone names, the faces, the per-vertex labels, the mesh names and the
## ground set, written once. The vertices are `_skinned`'s, in its order (every visible skinned mesh, and an outfit's
## `BodyCheck`), so a global index is the running offset over meshes and surfaces. Returns the handle `_dump_frame`
## and `_dump_close` take.
func _dump_open(path: String, h: Human) -> Dictionary:
	var sk := h.skeleton
	var c := _skin_cache(h)
	var f := FileAccess.open(path, FileAccess.WRITE)
	if f == null:
		printerr("pose_sheet: cannot write the frame dump %s" % path)
		quit(1)
		return {}
	var n: int = (c["V"] as PackedVector3Array).size()
	var faces: PackedInt32Array = c["faces"]
	f.store_buffer("MWCF".to_ascii_buffer())
	f.store_32(MWCF_VERSION)
	f.store_32(n)
	f.store_32(faces.size() / 3)
	f.store_32(sk.get_bone_count())
	var count_at := f.get_position()
	f.store_32(0)  # the frame count, written on close
	for i in sk.get_bone_count():
		_store_text(f, sk.get_bone_name(i))
	f.store_buffer(faces.to_byte_array())
	f.store_buffer(c["labels"])
	var mesh_names: PackedStringArray = c["mesh_names"]
	f.store_16(mesh_names.size())
	for m in mesh_names:
		_store_text(f, m)
	# the ground set: the visible vertices within GROUND_SET_M of the lowest visible rest vertex
	var rest: Array = []
	for i in sk.get_bone_count():
		rest.append(sk.get_bone_global_rest(i))
	var at_rest := _skin_points(h, c, rest, c["all"])
	var visible: PackedByteArray = c["visible"]
	var floor_y := INF
	for i in n:
		if visible[i] == 1:
			floor_y = minf(floor_y, at_rest[i].y)
	var ground := PackedInt32Array()
	for i in n:
		if visible[i] == 1 and at_rest[i].y <= floor_y + GROUND_SET_M:
			ground.append(i)
	f.store_32(ground.size())
	f.store_buffer(ground.to_byte_array())
	print("pose_sheet: dump %s: %d vertices, %d faces, %d bones, ground set %d" % [path, n, faces.size() / 3,
		sk.get_bone_count(), ground.size()])
	return {"f": f, "h": h, "c": c, "ground": ground, "count_at": count_at, "count": 0}


## One frame: its tag ("walk:12"), the floor, the gait speed and the body's forward (horizontal), every bone's world
## origin, then every vertex (`full`) or the ground set only. `poses` are the skeleton-space bone poses drawn (the
## rest poses for "rest:0"). Nothing is written when `d` is empty.
func _dump_frame(d: Dictionary, tag: String, full: bool, poses: Array) -> void:
	if d.is_empty():
		return
	var f: FileAccess = d["f"]
	var h: Human = d["h"]
	var sk := h.skeleton
	_store_text(f, tag)
	f.store_float(h.global_position.y + _sole_offset)
	f.store_float(h._speed)
	var fwd := h.global_transform.basis.z
	fwd = Vector3(fwd.x, 0.0, fwd.z).normalized()
	f.store_float(fwd.x)
	f.store_float(fwd.z)
	var origins := PackedVector3Array()
	for i in sk.get_bone_count():
		origins.append(sk.global_transform * (poses[i] as Transform3D).origin)
	f.store_buffer(origins.to_byte_array())
	f.store_8(1 if full else 0)
	var pts := _skin_points(h, d["c"], poses, d["c"]["all"] if full else d["ground"])
	f.store_buffer(pts.to_byte_array())
	d["count"] = int(d["count"]) + 1


func _dump_close(d: Dictionary) -> void:
	if d.is_empty():
		return
	var f: FileAccess = d["f"]
	f.seek(int(d["count_at"]))
	f.store_32(int(d["count"]))
	f.close()
	d.clear()


func _store_text(f: FileAccess, text: String) -> void:
	var bytes := text.to_utf8_buffer()
	f.store_16(bytes.size())
	f.store_buffer(bytes)


## The body's skinning, prepared once for the dump, in `_skinned`'s visiting order: per vertex its bind-space
## position and up to `P` (slot, weight) pairs, where a slot is one (skin bind, skeleton bone) of one mesh; the
## per-vertex labels (MWCF: u16 dominant bone, f32 trunk share, f32 limb share, u16 mesh, i16 region, u8 under,
## u8 layer); the faces as global vertex indices; and which vertices are drawn.
func _skin_cache(h: Human) -> Dictionary:
	var sk := h.skeleton
	var slot_bone := PackedInt32Array()
	var slot_bind: Array[Transform3D] = []
	var surfaces: Array = []
	var mesh_names := PackedStringArray()
	var per_max := 1
	var n := 0
	for mi: MeshInstance3D in sk.find_children("*", "MeshInstance3D", true, false):
		if (not mi.visible and String(mi.name) != "BodyCheck") or mi.skin == null:
			continue
		var skin := mi.skin
		var base_slot := slot_bone.size()
		for b in skin.get_bind_count():
			var bone := skin.get_bind_bone(b)
			if bone < 0:
				bone = sk.find_bone(skin.get_bind_name(b))
			slot_bone.append(bone)
			slot_bind.append(skin.get_bind_pose(b))
		var mesh_index := mesh_names.size()
		mesh_names.append(String(mi.name))
		for s in mi.mesh.get_surface_count():
			var arr := mi.mesh.surface_get_arrays(s)
			var v: PackedVector3Array = arr[Mesh.ARRAY_VERTEX]
			var bi: PackedInt32Array = arr[Mesh.ARRAY_BONES]
			var per := bi.size() / maxi(v.size(), 1)
			per_max = maxi(per_max, per)
			surfaces.append([mi, mesh_index, base_slot, arr, per, n])
			n += v.size()
	var V := PackedVector3Array()
	V.resize(n)
	var S := PackedInt32Array()
	S.resize(n * per_max)
	var Wt := PackedFloat32Array()
	Wt.resize(n * per_max)
	var labels := PackedByteArray()
	labels.resize(n * 16)
	var visible := PackedByteArray()
	visible.resize(n)
	var faces := PackedInt32Array()
	for e: Array in surfaces:
		var mi: MeshInstance3D = e[0]
		var mesh_index: int = e[1]
		var base_slot: int = e[2]
		var arr: Array = e[3]
		var per: int = e[4]
		var first: int = e[5]
		var v: PackedVector3Array = arr[Mesh.ARRAY_VERTEX]
		var bi: PackedInt32Array = arr[Mesh.ARRAY_BONES]
		var bw: PackedFloat32Array = arr[Mesh.ARRAY_WEIGHTS]
		var uv2: PackedVector2Array = arr[Mesh.ARRAY_TEX_UV2] if arr[Mesh.ARRAY_TEX_UV2] != null \
			else PackedVector2Array()
		var layer := clampi(_layer(String(mi.name)), 0, 255)
		for i in v.size():
			var g := first + i
			V[g] = v[i]
			visible[g] = 1 if mi.visible else 0
			var best_w := -1.0
			var best_b := 0
			var tw := 0.0
			var lw := 0.0
			for k in per:
				var w := bw[i * per + k]
				S[g * per_max + k] = base_slot + bi[i * per + k]
				Wt[g * per_max + k] = w
				if w <= 0.0:
					continue
				var name := sk.get_bone_name(slot_bone[base_slot + bi[i * per + k]])
				if name in TORSO:
					tw += w
				elif _group(name) not in ["", "torso"]:
					lw += w
				if w > best_w:
					best_w = w
					best_b = slot_bone[base_slot + bi[i * per + k]]
			var o := g * 16
			labels.encode_u16(o, best_b)
			labels.encode_float(o + 2, tw)
			labels.encode_float(o + 6, lw)
			labels.encode_u16(o + 10, mesh_index)
			labels.encode_s16(o + 12, int(floor(uv2[i].x)) if i < uv2.size() else -1)
			labels.encode_u8(o + 14, 1 if i < uv2.size() and uv2[i].y < 0.0 else 0)
			labels.encode_u8(o + 15, layer)
		# Godot's front faces wind clockwise; the dump's wind counter-clockwise (glTF's and libigl's: outward by the
		# right-hand rule), so each triangle is written as (a, c, b). Read as Godot stores them, a closed body has a
		# generalized winding number of -1 inside, and nothing is ever "inside" it.
		var idx: PackedInt32Array = arr[Mesh.ARRAY_INDEX] if arr[Mesh.ARRAY_INDEX] != null else PackedInt32Array()
		if idx.is_empty():
			idx.resize(v.size() - v.size() % 3)
			for i in idx.size():
				idx[i] = i
		for t in idx.size() / 3:
			faces.append(first + idx[3 * t])
			faces.append(first + idx[3 * t + 2])
			faces.append(first + idx[3 * t + 1])
	var all := PackedInt32Array()
	all.resize(n)
	for i in n:
		all[i] = i
	return {"slot_bone": slot_bone, "slot_bind": slot_bind, "V": V, "S": S, "W": Wt, "P": per_max,
		"labels": labels, "visible": visible, "faces": faces, "mesh_names": mesh_names, "all": all}


## The world positions of the vertices `idx` under the skeleton-space bone poses `poses`, skinned on the CPU exactly
## as the GPU skins them (Σ w · pose(bone) · bind), as `_skinned` does.
func _skin_points(h: Human, c: Dictionary, poses: Array, idx: PackedInt32Array) -> PackedVector3Array:
	var sk := h.skeleton
	var slot_bone: PackedInt32Array = c["slot_bone"]
	var slot_bind: Array[Transform3D] = c["slot_bind"]
	var mats: Array[Transform3D] = []
	for s in slot_bone.size():
		mats.append(sk.global_transform * (poses[slot_bone[s]] as Transform3D) * slot_bind[s])
	var V: PackedVector3Array = c["V"]
	var S: PackedInt32Array = c["S"]
	var Wt: PackedFloat32Array = c["W"]
	var P: int = c["P"]
	var out := PackedVector3Array()
	out.resize(idx.size())
	for j in idx.size():
		var i := idx[j]
		var p := Vector3.ZERO
		for k in P:
			var w := Wt[i * P + k]
			if w > 0.0:
				p += (mats[S[i * P + k]] * V[i]) * w
		out[j] = p
	return out


# --- framing and capture ---------------------------------------------------------------------------------------------

## A full-body camera at `az` degrees round from the subject's front, framed from its stature so nothing crops.
func _frame_full(h: Human, az: float, stature: float) -> void:
	_cam.fov = 30.0
	var a := _facing + deg_to_rad(az)
	var at := h.global_position
	var d := (stature * 0.62) / tan(deg_to_rad(15.0))
	_cam.position = at + Vector3(sin(a), 0, cos(a)) * d + Vector3(0, stature * 0.55, 0)
	_cam.look_at(at + Vector3(0, stature * 0.50, 0), Vector3.UP)


func _capture(id: String, pose: String, what: String, h: Human, seat: Dictionary) -> void:
	await process_frame
	await RenderingServer.frame_post_draw
	var img := _vp.get_texture().get_image()
	if not _capture_dump.is_empty() and h == _capture_dump["h"]:
		_dump_frame(_capture_dump, id, true, _drawn(h.skeleton))
	# the red border is drawn for the checks decided here (G-13, W-4); G-9w is decided from the dump and shown on the
	# sheet's HTML card (`mwcast/pose_sheet.py`)
	var checks := _checks(h, seat)
	var ok: bool = checks["pass"]
	if not ok:
		var red := Color(0.86, 0.08, 0.08)
		var b := 14
		img.fill_rect(Rect2i(0, 0, W, b), red)
		img.fill_rect(Rect2i(0, H - b, W, b), red)
		img.fill_rect(Rect2i(0, 0, b, H), red)
		img.fill_rect(Rect2i(W - b, 0, b, H), red)
	var file := "frames/%s.png" % id
	img.save_png(_out.path_join(file))
	checks.erase("pass")
	_frames.append({"id": id, "pose": pose, "what": what, "file": file, "pass": ok, "checks": checks})
	print("pose_sheet: %s %s %s" % [id, "PASS" if ok else "FLAGGED", JSON.stringify(checks)])


# --- the checks ------------------------------------------------------------------------------------------------------

## Skinned vertex positions in world space, with each vertex's dominant bone, computed on the CPU from the posed
## skeleton exactly as the GPU skins them (Σ w · pose(bone) · bind).
func _skinned(h: Human) -> Dictionary:
	var sk := h.skeleton
	var pts := PackedVector3Array()
	var dom := PackedStringArray()
	# per vertex: the weight share of the trunk bones, and of the limbs (arms and legs)
	var trunk_w := PackedFloat32Array()
	var limb_w := PackedFloat32Array()
	var top := -INF
	# per vertex: the mesh it belongs to and its body region, or -1 (the outfit checks, S32-p)
	var mesh_of := PackedStringArray()
	var index_of := PackedInt32Array()
	# and whether it lies under an outer layer at rest (UV2 v > 1; written by the assembly)
	var under_of: Array[bool] = []
	for mi: MeshInstance3D in sk.find_children("*", "MeshInstance3D", true, false):
		# `BodyCheck` (an assembled outfit's unmasked body) is never drawn but is W-4's reference
		if (not mi.visible and String(mi.name) != "BodyCheck") or mi.skin == null:
			continue
		var skin := mi.skin
		var mats: Array[Transform3D] = []
		var names: Array[String] = []
		var is_trunk: Array[bool] = []
		var is_limb: Array[bool] = []
		for b in skin.get_bind_count():
			var bone := skin.get_bind_bone(b)
			if bone < 0:
				bone = sk.find_bone(skin.get_bind_name(b))
			mats.append(sk.global_transform * _gpose(sk, bone) * skin.get_bind_pose(b))
			names.append(sk.get_bone_name(bone))
			is_trunk.append(sk.get_bone_name(bone) in TORSO)
			is_limb.append(_group(sk.get_bone_name(bone)) not in ["", "torso"])
		for s in mi.mesh.get_surface_count():
			var arr := mi.mesh.surface_get_arrays(s)
			var v: PackedVector3Array = arr[Mesh.ARRAY_VERTEX]
			# S32-p: an assembled outfit's body carries each vertex's region in its second UV map (u = index + 0.5)
			var uv2: PackedVector2Array = arr[Mesh.ARRAY_TEX_UV2] if arr[Mesh.ARRAY_TEX_UV2] != null \
				else PackedVector2Array()
			var bi: PackedInt32Array = arr[Mesh.ARRAY_BONES]
			var bw: PackedFloat32Array = arr[Mesh.ARRAY_WEIGHTS]
			var per := bi.size() / maxi(v.size(), 1)
			for i in v.size():
				var p := Vector3.ZERO
				var best_w := -1.0
				var best_b := 0
				var tw := 0.0
				var lw := 0.0
				for k in per:
					var w := bw[i * per + k]
					if w <= 0.0:
						continue
					var b := bi[i * per + k]
					p += (mats[b] * v[i]) * w
					if is_trunk[b]:
						tw += w
					elif is_limb[b]:
						lw += w
					if w > best_w:
						best_w = w
						best_b = b
				pts.append(p)
				dom.append(names[best_b])
				trunk_w.append(tw)
				limb_w.append(lw)
				mesh_of.append(String(mi.name))
				index_of.append(int(floor(uv2[i].x)) if i < uv2.size() else -1)
				# Blender wrote v = 1.5 for "under"; glTF's v is 1 - Blender's, so it arrives as -0.5
				under_of.append(i < uv2.size() and uv2[i].y < 0.0)
				top = maxf(top, p.y)
	return {"p": pts, "bone": dom, "top": top, "trunk_w": trunk_w, "limb_w": limb_w, "mesh": mesh_of,
		"index": index_of, "under": under_of}


func _bone_world(h: Human, bone: String) -> Vector3:
	var sk := h.skeleton
	return sk.global_transform * _gpose(sk, sk.find_bone(bone)).origin


## Horizontal coordinates in the subject's frame: x to its left, y forward.
func _body_xy(p: Vector3) -> Vector2:
	var fwd := Vector3(sin(_facing), 0, cos(_facing))
	var left := Vector3(cos(_facing), 0, -sin(_facing))
	return Vector2(p.dot(left), p.dot(fwd))


## Foot separation along the facing direction, for picking contact and passing frames.
func _stride(h: Human) -> float:
	var fwd := Vector3(sin(_facing), 0, cos(_facing))
	return (_bone_world(h, "LeftFoot") - _bone_world(h, "RightFoot")).dot(fwd)


func _bin(q: Vector2) -> int:
	return int(floor((q.angle() + PI) / TAU * POLAR_BINS)) % POLAR_BINS


func _group(bone: String) -> String:
	if bone in TORSO:
		return "torso"
	for f in ["Thumb", "Index", "Middle", "Ring", "Little"]:
		if f in bone:
			return "hand"
	if bone.ends_with("Hand"):
		return "hand"
	if bone.ends_with("LowerArm"):
		return "sleeve"
	if bone.ends_with("UpperArm"):
		return "upper_arm"
	if bone.ends_with("UpperLeg"):
		return "thigh"
	if bone.ends_with("LowerLeg") or bone.ends_with("Foot") or bone.ends_with("Toes"):
		return "leg"
	return ""


## The checks still decided in Godot: W-4's poke-through in outfit mode (until S36-P3) and G-13's seat contact for
## P-4. G-9's arm–torso test is G-9w now, decided in Python from the captured frame's dump (`mwcast/gait.py`).
func _checks(h: Human, seat: Dictionary) -> Dictionary:
	var sk_data := _skinned(h)
	var pts: PackedVector3Array = sk_data["p"]
	var bones: PackedStringArray = sk_data["bone"]
	var out := {}
	var ok := true
	if not _outfit.is_empty():
		var poke := _poke_through(h, sk_data)
		out["poke_through"] = poke
		ok = ok and bool(poke["pass"])
	if not seat.is_empty():
		var seat_top: float = seat["seat_top"]
		var low := _seat_low(h, pts, bones, seat)
		var below := 0
		for i in pts.size():
			if bones[i] in SEAT and pts[i].y < seat_top - SEAT_MM / 1000.0 and _over_seat(h, pts[i], seat):
				below += 1
		var err := (low - seat_top) * 1000.0 if low < INF else INF
		# G-13 under one placement rule: how far this body would float (+) or sink (-) where the protagonist's
		# hips-over-seat height seats it
		var rule := (HIP_OVER_SEAT_M - float(seat.get("hip_over_seat", HIP_OVER_SEAT_M))) * 1000.0
		var seat_ok := absf(err) <= SEAT_MM and absf(rule) <= SEAT_MM
		out["seat_contact"] = {"pass": seat_ok, "seat_top_m": snappedf(seat_top, 0.001),
			"lowest_seat_vertex_m": snappedf(low, 0.001) if low < INF else null,
			"error_mm": snappedf(err, 0.1) if err < INF else null, "vertices_below_bound": below,
			"hip_over_seat_m": snappedf(float(seat.get("hip_over_seat", 0.0)), 0.001),
			"protagonist_rule_error_mm": snappedf(rule, 0.1)}
		ok = ok and seat_ok
		# not a gate: how far the hands are from the table top (negative = into it)
		var table_top: float = seat["table_top"]
		var hand_low := INF
		for i in pts.size():
			if _group(bones[i]) == "hand":
				hand_low = minf(hand_low, pts[i].y)
		out["hands_above_table_mm"] = snappedf((hand_low - table_top) * 1000.0, 0.1)
	out["pass"] = ok
	return out


func _layer(mesh_name: String) -> int:
	var parts: Dictionary = _outfit.get("parts", {})
	if parts.has(mesh_name):
		return int(parts[mesh_name]["layer"])
	return 0


func _covers(mesh_name: String) -> Array:
	var parts: Dictionary = _outfit.get("parts", {})
	return parts[mesh_name]["covers"] if parts.has(mesh_name) else []


## W-4 (S32-p): no visible body vertex lies outside a garment's shell by more than POKE_MM within the garment's
## covered regions, and no inner-layer vertex outside an outer layer. Per garment, per segment (the trunk along
## Hips->Neck, each upper arm, forearm, thigh and shin along its bone), the garment's shell is its outermost radius
## from the segment's axis in 2 cm slabs x 48 directions; an inner vertex (a body vertex of a covered region, or a
## vertex of a lower layer) further out than the smallest of the three neighbouring radii by more than POKE_MM is a
## poke-through, printed with its mesh, vertex index, segment and depth. Only directions where the garment has
## surface are judged (a backpack judges the back only).
func _poke_through(h: Human, sk_data: Dictionary) -> Dictionary:
	var pts: PackedVector3Array = sk_data["p"]
	var bones: PackedStringArray = sk_data["bone"]
	var meshes: PackedStringArray = sk_data["mesh"]
	var index: PackedInt32Array = sk_data["index"]
	var under: Array[bool] = sk_data["under"]
	var up := (_bone_world(h, "Neck") - _bone_world(h, "Hips")).normalized()
	var fwd0 := Vector3(sin(_facing), 0, cos(_facing))
	var axes := {}
	axes["torso"] = [_bone_world(h, "Hips") - up * 0.15, _bone_world(h, "Neck")]
	for side in ["Left", "Right"]:
		axes[side + "upper_arm"] = [_bone_world(h, side + "UpperArm"), _bone_world(h, side + "LowerArm")]
		axes[side + "sleeve"] = [_bone_world(h, side + "LowerArm"), _bone_world(h, side + "Hand")]
		axes[side + "thigh"] = [_bone_world(h, side + "UpperLeg"), _bone_world(h, side + "LowerLeg")]
		axes[side + "leg"] = [_bone_world(h, side + "LowerLeg"), _bone_world(h, side + "Foot")]
	# per vertex: its segment key, slab and bin and radius (or -1)
	var seg := PackedStringArray()
	var cell := PackedInt32Array()
	var rad := PackedFloat32Array()
	for i in pts.size():
		var g := _group(bones[i])
		if not POKE_REGIONS.has(g):
			seg.append("")
			cell.append(-1)
			rad.append(-1.0)
			continue
		var key: String = g if g == "torso" else ("Left" if bones[i].begins_with("Left") else "Right") + g
		var a0: Vector3 = axes[key][0]
		var a1: Vector3 = axes[key][1]
		var dir := (a1 - a0).normalized()
		var t := (pts[i] - a0).dot(dir)
		var q := pts[i] - (a0 + dir * t)
		var fwd := (fwd0 - dir * fwd0.dot(dir)).normalized()
		var left := dir.cross(fwd).normalized()
		var slab := int(floor(t / SLAB_M))
		seg.append(key)
		cell.append(slab * POLAR_BINS + _bin(Vector2(q.dot(left), q.dot(fwd))))
		rad.append(q.length())
	var worst := 0.0
	var worst_at := ""
	var count := 0
	var by_part := {}
	var tested := 0
	var closest := -INF  # the largest depth seen (negative: every tested vertex stays inside its shell)
	var parts: Dictionary = _outfit.get("parts", {})
	for part: String in parts:
		var layer := _layer(part)
		var covers := _covers(part)
		var shell := {}  # segment key -> {cell: radius}
		for i in pts.size():
			if meshes[i] != part or cell[i] < 0:
				continue
			var s: Dictionary = shell.get(seg[i], {})
			s[cell[i]] = maxf(float(s.get(cell[i], 0.0)), rad[i])
			shell[seg[i]] = s
		for i in pts.size():
			if cell[i] < 0 or not shell.has(seg[i]) or not under[i]:
				continue
			var inner := false
			if meshes[i] == "Body":
				continue  # the drawn, masked body: `BodyCheck` stands for it, whole
			if meshes[i] == "BodyCheck":
				var r := index[i]  # the body vertex's region (UV2), -1 where it has none
				# the region must be covered by the part and belong to the segment it is tested in (an armpit
				# vertex of the torso is not judged against the sleeve's radius round the upper arm)
				var seg_group := seg[i].trim_prefix("Left").trim_prefix("Right")
				inner = r >= 0 and r < _region_names.size() and _region_names[r] in covers \
					and _region_names[r] in POKE_REGIONS[seg_group]
			else:
				inner = _layer(meshes[i]) < layer
			if not inner:
				continue
			var s: Dictionary = shell[seg[i]]
			var c := cell[i]
			var slab_base := c - c % POLAR_BINS
			var b := c % POLAR_BINS
			var n1 := slab_base + (b + 1) % POLAR_BINS
			var n2 := slab_base + (b + POLAR_BINS - 1) % POLAR_BINS
			if not (s.has(c) and s.has(n1) and s.has(n2)):
				continue
			var depth := (rad[i] - minf(float(s[c]), minf(float(s[n1]), float(s[n2])))) * 1000.0
			tested += 1
			closest = maxf(closest, depth)
			if depth > POKE_MM:
				count += 1
				by_part[part] = int(by_part.get(part, 0)) + 1
			if depth > worst:
				worst = depth
				var where := _region_names[index[i]] if meshes[i] == "BodyCheck" and index[i] >= 0 else "layer"
				worst_at = "%s (%s) at %s through %s, segment %s, slab-bin %d" % [meshes[i], where,
					str(pts[i].snappedf(0.001)), part, seg[i], cell[i]]
	var body_n := 0
	var body_mapped := 0
	for i in meshes.size():
		if meshes[i] == "BodyCheck":
			body_n += 1
			body_mapped += int(index[i] >= 0)
	# Without the body's region channel the check cannot tell covered skin from bare skin: it is not passed.
	var mapped := body_n > 0 and body_mapped > body_n / 2
	# a check that tested nothing has not passed
	return {"pass": worst <= POKE_MM and mapped and tested > 0, "max_mm": snappedf(worst, 0.1), "worst": worst_at,
		"vertices_over_bound": count, "by_part": by_part, "body_vertices": body_n, "tested": tested,
		"closest_mm": snappedf(closest, 0.1) if tested > 0 else null,
		"body_vertices_with_region": body_mapped}


## The lowest vertex of the buttocks and the thighs' underside over the seat panel: seat-bone vertices from 10 cm
## behind the hips joint forward (a fused pack hangs further back, against the chair's back, and is not a seat contact).
func _seat_low(h: Human, pts: PackedVector3Array, bones: PackedStringArray, seat: Dictionary) -> float:
	var low := INF
	for i in pts.size():
		if bones[i] in SEAT and _over_seat(h, pts[i], seat):
			low = minf(low, pts[i].y)
	return low


func _over_seat(h: Human, p: Vector3, seat: Dictionary) -> bool:
	var at: Vector3 = seat["chair_at"]
	var q := _body_xy(p - at)
	return (seat["rect"] as Rect2).has_point(q) and q.y >= _body_xy(_bone_world(h, "Hips") - at).y - 0.10


# --- asset geometry --------------------------------------------------------------------------------------------------

func _vertices_world(n: Node3D) -> PackedVector3Array:
	var out := PackedVector3Array()
	for mi: MeshInstance3D in n.find_children("*", "MeshInstance3D", true, false):
		var xf := mi.global_transform
		for s in mi.mesh.get_surface_count():
			var v: PackedVector3Array = mi.mesh.surface_get_arrays(s)[Mesh.ARRAY_VERTEX]
			for p in v:
				out.append(xf * p)
	return out


func _vertex_bounds_world(n: Node3D) -> AABB:
	var pts := _vertices_world(n)
	var b := AABB(pts[0], Vector3.ZERO)
	for p in pts:
		b = b.expand(p)
	return b


## Bounds in `n`'s own frame (its transform ignored), of the vertices `keep` accepts.
func _vertex_bounds(n: Node3D, keep: Callable) -> AABB:
	var inv := n.global_transform.affine_inverse() if n.is_inside_tree() else Transform3D.IDENTITY
	var b := AABB()
	var first := true
	for mi: MeshInstance3D in n.find_children("*", "MeshInstance3D", true, false):
		var xf := inv * mi.global_transform if n.is_inside_tree() else _local_chain(n, mi)
		for s in mi.mesh.get_surface_count():
			var v: PackedVector3Array = mi.mesh.surface_get_arrays(s)[Mesh.ARRAY_VERTEX]
			for p in v:
				var q := xf * p
				if not keep.call(q):
					continue
				if first:
					b = AABB(q, Vector3.ZERO)
					first = false
				else:
					b = b.expand(q)
	return b


func _local_chain(n: Node3D, mi: Node3D) -> Transform3D:
	var xf := Transform3D.IDENTITY
	var c: Node = mi
	while c != n and c != null:
		xf = (c as Node3D).transform * xf
		c = c.get_parent()
	return xf


func _xf_aabb(xf: Transform3D, b: AABB) -> AABB:
	return xf * b


# --- the pose dump for the 2D sprites (PR S26-c-pose part F) ----------------------------------------------------------

## The pose being dumped for the current body (`--dump-pose <dir>`), or null.
var _dump = null


## Each bone's change from its rest, in the imported scene's own frame (the glTF's: Y-up, the body facing +Z; the
## sole lift and the instance placement left out), keyed by the export's own joint name through the client's bone map: the skinned mesh is `delta · rest-skinned`, so a renderer
## that applies the same deltas to the same export draws the same posed body.
func _deltas(h: Human, poses: Array) -> Dictionary:
	var sk := h.skeleton
	var to_scene := h._inst.global_transform.affine_inverse() * sk.global_transform
	# the skin's own bind poses: the import's rest fixer rewrites the rests (`overwrite_axis`) and compensates in the
	# binds, so the skinning matrix (pose * bind), not pose * rest^-1, is what moves the export's rest vertices
	var mi: MeshInstance3D = null
	for m: MeshInstance3D in sk.find_children("*", "MeshInstance3D", true, false):
		if m.skin != null and m.visible:
			mi = m
			break
	var mesh_in_scene := h._inst.global_transform.affine_inverse() * mi.global_transform
	var binds := {}
	for b in mi.skin.get_bind_count():
		var bone := mi.skin.get_bind_bone(b)
		bone = bone if bone >= 0 else sk.find_bone(mi.skin.get_bind_name(b))
		binds[bone] = mi.skin.get_bind_pose(b)
	var bm := load("res://assets/characters/vitruvian/vitruvian_bonemap.tres") as BoneMap
	var out := {}
	for i in sk.get_bone_count():
		if not binds.has(i):
			continue
		var src := String(bm.get_skeleton_bone_name(sk.get_bone_name(i)))
		if src == "":
			src = sk.get_bone_name(i)
		var d := to_scene * (poses[i] as Transform3D) * (binds[i] as Transform3D) * mesh_in_scene.affine_inverse()
		out[src.replace("mixamorig_", "mixamorig:")] = [d.basis.x.x, d.basis.y.x, d.basis.z.x, d.origin.x,
			d.basis.x.y, d.basis.y.y, d.basis.z.y, d.origin.y, d.basis.x.z, d.basis.y.z, d.basis.z.z, d.origin.z]
	return out
