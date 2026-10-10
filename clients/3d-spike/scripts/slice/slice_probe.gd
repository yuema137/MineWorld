## The slice's evidence harness. Four scripted modes, none of them optional
## extras: each one produces a number that a claim in `docs/VISUAL_SLICE.md`
## rests on, and a claim without its number is not made.
##
##   --slice-shots      capture the sec.12 review views, from the real client
##   --slice-drive      walk the real controller in through the real door and
##                      out again, and report what the body actually did
##   --slice-measure    print measured dimensions against sec.3's ranges
##   --slice-threshold  the sec.7.1 indoor/outdoor measurement, at four points
##   --slice-perf       frame cost at eight viewpoints by a fixed protocol
##                      (`slice_perf.gd`; `--perf-breakdown` for categories)
##   --slice-skyline-check  the drawn skyline against the summit table (A-8)
##   --slice-character  the occupant animates (idle and walk, measured on its
##                      bones) and every camera mode, standing and walking,
##                      outdoors and in, as frames to inspect
##
## `--slice-drive` runs headless. The other three need pixels and run windowed.
##
## The modes that need a world (`--slice-link`, `--slice-conversation`, and the
## ones S14 adds) live in `slice_probe_world.gd`, which extends this harness and
## shares its walking and capture helpers (step-15 §19.4 D-16a-5).
class_name SliceProbe
extends Node

const OUT := "res://shots/slice"

var player: SlicePlayer
var slice: SliceMain

const FP := CameraRig.Mode.FIRST_PERSON
const REAR := CameraRig.Mode.THIRD_REAR
const FRONT := CameraRig.Mode.THIRD_FRONT

## The nine views `VISUAL_SLICE.md` sec.12 requires, then the diagnostics.
## yaw 0 faces north (-z), which is the frontage; -90 faces east along the
## street; 180 faces south, out across the carriageway.
var views := [
	["01_street_wide", Vector3(-20.0, 0.45, -5.20), -75.0, -1.0, FP],
	# 05's framing: across the carriageway from the opposite pavement, diagonal,
	# the cafe frontage the subject
	["01r_street_reference_framing", Vector3(-8.6, 0.45, 5.7), -46.0, 4.0, FP],
	["02_cafe_approach", Vector3(-2.00, 0.45, -5.40), -55.0, 0.0, FP],
	["03_cafe_exterior", Vector3(0.30, 0.45, -5.55), -52.0, 3.0, FP],
	# 03's own framing: 03_cafe_frontage.png looks WEST along the frontage with
	# the cafe on the right of frame and the terrace under its window; the view
	# above looks east and mirrors it. This one stands east of the cafe.
	["03r_reference_framing", Vector3(10.6, 0.45, -4.30), 40.0, 3.0, FP],
	["04_doorway", Vector3(3.45, 0.45, -6.90), 0.0, 0.0, FP],
	# the same crossing with the character in it: she stands on the threshold
	["04c_doorway_character", Vector3(3.45, 0.45, -8.05), 0.0, -2.0, REAR],
	["05_interior_wide", Vector3(2.75, 0.60, -10.60), -38.0, -3.0, FP],
	["06_interior_character", Vector3(4.60, 0.60, -12.60), -8.0, -2.0, REAR],
	["07_third_rear", Vector3(-4.00, 0.45, -5.60), -80.0, -3.0, REAR],
	# facing the road with the café behind her; at (4.2, -6.1) the boom put a
	# terrace chair across her legs
	["08_third_front", Vector3(3.00, 0.45, -5.20), 180.0, -1.0, FRONT],
	["09_character_in_place", Vector3(5.20, 0.60, -11.20), 155.0, -1.0, FRONT],
	# diagnostics, for the agent rather than for the operator
	["10_interior_counter", Vector3(7.40, 0.60, -12.80), -6.0, 0.0, FP],
	["11_interior_looking_out", Vector3(6.20, 0.60, -11.60), 178.0, 0.0, FP],
	["12_back_wall", Vector3(5.20, 0.60, -13.60), 4.0, 1.0, FP],
	["13_doorway_from_inside", Vector3(3.55, 0.60, -10.40), 179.0, -1.0, FP],
	["14_west_frontage", Vector3(-15.0, 0.45, -6.40), -40.0, 4.0, FP],
	["15_east_end", Vector3(17.0, 0.45, -5.40), -70.0, 2.0, FP],
	["16_south_side", Vector3(2.0, 0.45, -4.60), 165.0, 1.0, FP],
	["17_street_from_east", Vector3(22.0, 0.45, -5.30), 96.0, -1.0, FP],
	# connected (--world): the world's people where the world says they are
	["19_connected_people", Vector3(9.2, 0.60, -14.3), 135.0, -4.0, FP],
	["18_pavement_detail", Vector3(6.00, 0.45, -5.20), -20.0, -26.0, FP],
	# VISUAL_SLICE.md sec.12 view 10: the second enterable building, sec.4.1
	["20_florist_door", Vector3(9.60, 0.45, -4.20), -22.0, 3.0, FP],
	["21_florist_interior", Vector3(11.40, 0.45, -9.10), -38.0, -6.0, FP],
	["22_florist_looking_out", Vector3(17.40, 0.45, -13.30), 140.0, -3.0, FP],
	# RL-b's skyline preview (skyline.gd): east along the street to the mountains,
	# west from the east end, and south-west from the west end toward Point Loma
	["23_skyline_east", Vector3(-20.0, 0.45, -5.20), -90.0, 4.0, FP],
	["24_skyline_west", Vector3(22.0, 0.45, -5.30), 90.0, 2.0, FP],
	["25_skyline_southwest", Vector3(-33.0, 0.45, 0.0), 123.0, 2.0, FP],
]

## Compositions the player's camera rig cannot frame: a camera that stays put
## while the body stands where it is placed. Real runtime frames, the same scene
## and lighting; only the viewpoint is free of the boom.
##   [name, camera position, camera yaw, camera pitch, fov, body position, body yaw]
## Camera heights are floor + eye height: pavement 0.14, café floor 0.29.
var fixed_views := [
	# 03's framing (as 03r) with her where 03 has its walker: on the pavement,
	# walking away along the frontage
	["03s_reference_framing_character", Vector3(10.6, 0.14 + CameraRig.EYE_HEIGHT, -4.30),
		40.0, 3.0, 75.0, Vector3(7.5, 0.45, -5.60), 90.0],
	# VISUAL_SLICE.md sec.12 view 9 at a distance a person can be judged at: chest
	# up to full figure, 1.9 m away, the counter and shelving behind her
	# (camera a little above her eye: at 1.45 m a pendant behind sat on her bun)
	["09c_character_close", Vector3(5.2 - 0.423 * 1.9, 0.29 + 1.65, -11.2 + 0.906 * 1.9),
		-25.0, -12.0, 50.0, Vector3(5.20, 0.60, -11.20), 155.0],
]

## Frames cropped and enlarged beside the capture, because a claim decided at
## 60 px is the evidence defect `ARC-17` sec.6 records three times over.
const CROPS := {
	"03_cafe_exterior": ["03b_fascia_detail", Rect2i(520, 120, 520, 300), 2],
	"05_interior_wide": ["05b_backwall_detail", Rect2i(560, 260, 480, 340), 2],
	"09_character_in_place": ["09b_character_detail", Rect2i(660, 250, 300, 380), 2],
}

var _mode := ""
var _warm := 0
## Simulated time, counted on physics ticks -- immune to how long a frame
## capture took in wall-clock time.
var _phys_t := 0.0


func _physics_process(delta: float) -> void:
	_phys_t += delta


## The standalone modes; `SliceProbeWorld.WORLD_MODES` lists the connected ones.
const MODES := ["shots", "drive", "measure", "threshold", "perf", "hud", "jumpshots", "doors",
	"character", "skyline-check"]


## The mode of `modes` the command line asks for, or "".
static func requested_of(modes: Array) -> String:
	var a := OS.get_cmdline_user_args()
	for m in modes:
		if "--slice-" + m in a:
			return m
	return ""


static func scripted() -> bool:
	return requested_of(MODES) != ""


## The modes this probe runs; a sibling harness adds its own.
func _modes() -> Array:
	return MODES


func _ready() -> void:
	if player != null:
		player.scripted_look = true
		# this runner turns the body by setting its yaw; any mouse motion is
		# the real cursor over the window
		player.ignore_mouse_look = true
	# A scripted capture must not be paced by the display. macOS throttles an
	# unfocused or occluded window hard: with vsync on, a capture run that
	# should take twenty seconds sat waiting on the GPU present at well under
	# one frame per second, which profiling showed as the main thread parked in
	# `waitUntilSignaledValue`. It was never a rendering cost, and treating it
	# as one would have led to cutting content that is not expensive.
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
	Engine.max_fps = 0
	OS.low_processor_usage_mode = false
	_mode = requested_of(_modes())
	DirAccess.make_dir_recursive_absolute(OUT)


func _process(_d: float) -> void:
	# Let the sky radiance, the reflection probes, the VoxelGI bake and the
	# shadow splits settle before the first frame is trusted.
	_warm += 1
	if _warm < (8 if _mode == "drive" else 12):
		return
	set_process(false)
	await _run(_mode)
	get_tree().quit(0)


## Run one mode; a sibling harness runs its own and hands the rest here.
func _run(mode: String) -> void:
	match mode:
		"shots": await _capture()
		"drive": await _drive()
		"measure": _measure()
		"threshold": await _threshold()
		"perf": await _perf()
		"hud": await _hud_frames()
		"jumpshots": await _jump_frames()
		"doors": await _door_frames()
		"character": await _character_check()
		"skyline-check": SliceSkyline.check(slice.world.find_child("Skyline", false, false))


func _settle(frames := 10) -> void:
	for i in range(frames):
		await get_tree().process_frame
	await RenderingServer.frame_post_draw


# --- shots ---------------------------------------------------------------------

func _capture() -> void:
	# connected (--world/--server): wait for the world's people to be drawn
	if slice.link != null:
		var w := 0.0
		while w < 10.0 and slice.link.figures.is_empty():
			await get_tree().process_frame
			w += get_process_delta_time()
		print("link   %d perceived people drawn" % slice.link.figures.size())
	# --views=a,b captures only the named views (a prefix is enough)
	var only: PackedStringArray = []
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--views="):
			only = a.substr(8).split(",")
	for v in views:
		if not only.is_empty():
			var want := false
			for o in only:
				if str(v[0]).begins_with(o):
					want = true
			if not want:
				continue
		player.place(v[1], v[2], v[3])
		player.set_camera(v[4])
		await _settle()
		var img := get_viewport().get_texture().get_image()
		img.save_png("%s/%s.png" % [OUT, v[0]])
		if CROPS.has(v[0]):
			var spec: Array = CROPS[v[0]]
			var rect: Rect2i = (spec[1] as Rect2i).intersection(
				Rect2i(Vector2i.ZERO, img.get_size()))
			var c := img.get_region(rect)
			c.resize(rect.size.x * int(spec[2]), rect.size.y * int(spec[2]),
				Image.INTERPOLATE_CUBIC)
			c.save_png("%s/%s.png" % [OUT, spec[0]])
		print("shot %-26s at %s yaw %.0f  [%s]"
			% [v[0], v[1], v[2], player.rig.mode_name()])
		# The body's feet should be on a floor: 0.14 outside, 0.29 inside. Higher
		# means the view's start point was inside furniture and the body was
		# pushed up on top of it -- which is what made 05 look down from 3 m.
		if player.global_position.y > 0.40:
			print("  WARNING: body standing at y %.2f -- on top of something, not the floor"
				% player.global_position.y)
		_pick_report()
	for v in fixed_views:
		if not only.is_empty() and not Array(only).any(func(o): return str(v[0]).begins_with(o)):
			continue
		await _fixed_shot(v)
	print("\ngi mode: %s" % slice.gi_name())
	print("character slot: %s" % player.slot.describe())


func _fixed_shot(v: Array) -> void:
	player.place(v[5], v[6], 0.0)
	player.set_camera(REAR)   # any third-person mode: the body is drawn
	var cam := Camera3D.new()
	cam.fov = v[4]
	slice.add_child(cam)
	cam.global_transform = Transform3D(
		Basis.from_euler(Vector3(deg_to_rad(v[3]), deg_to_rad(v[2]), 0.0)), v[1])
	cam.make_current()
	await _settle(12)
	await _save(v[0])
	print("shot %-26s camera %s yaw %.0f, body at %s yaw %.0f  [fixed camera]"
		% [v[0], v[1], v[2], player.global_position, v[6]])
	cam.queue_free()
	player.set_camera(REAR)   # hand the view back to the rig


## `--pick=x,y`: what is drawn at that pixel of the frame just captured? Lists
## every visual instance whose world bounds the camera ray through the pixel
## crosses, nearest first -- so an object seen in a frame is identified by its
## node path rather than guessed at from its shape (`ARC-23`).
func _pick_report() -> void:
	for a in OS.get_cmdline_user_args():
		if not a.begins_with("--pick="):
			continue
		var xy := a.substr(7).split(",")
		var px := Vector2(float(xy[0]), float(xy[1]))
		var cam := get_viewport().get_camera_3d()
		var o := cam.project_ray_origin(px)
		var d := cam.project_ray_normal(px)
		var hits: Array = []
		_pick_walk(slice.world, o, d, hits)
		hits.sort_custom(func(p, q): return p[0] < q[0])
		print("  pick %s:" % px)
		for h in hits.slice(0, 8):
			print("    %6.2f m  %s  aabb %s" % [h[0], h[1], h[2]])


func _pick_walk(n: Node, o: Vector3, d: Vector3, hits: Array) -> void:
	for c in n.get_children():
		_pick_walk(c, o, d, hits)
	if n is GeometryInstance3D:
		var gi := n as GeometryInstance3D
		var bb := gi.global_transform * gi.get_aabb()
		var hit: Variant = bb.intersects_ray(o, d)
		if hit != null:
			hits.append([o.distance_to(hit as Vector3), str(gi.get_path()), bb])


## The controls HUD as a player meets it: attached, the camera key pressed,
## captured while the toast shows and again after it has faded.
func _hud_frames() -> void:
	var hud := ControlsHud.attach(slice, player, true)
	hud.add_line(MineWorldText.text("hud.place", {"place": SliceWorld.STREET_PLACE}))
	player.place(SliceWorld.SPAWN, SliceWorld.SPAWN_YAW, -2.0)
	await _settle(8)
	player.cycle_camera()
	await _settle(12)
	get_viewport().get_texture().get_image().save_png("%s/hud_toast.png" % OUT)
	var t := 0.0
	while t < 2.8:
		await get_tree().process_frame
		t += get_process_delta_time()
	await _settle(4)
	get_viewport().get_texture().get_image().save_png("%s/hud_faded.png" % OUT)
	print("hud    captured hud_toast.png and hud_faded.png, camera now '%s'"
		% player.rig.mode_name())


## Every street door, from the pavement 3 m out and 1.4 m to one side, so the
## reveal's depth shows: where a first-time player looks for the way in.
func _door_frames() -> void:
	var k := 0
	for d in SliceTerrace.doors:
		var p: Vector3 = d[0]
		var fy: float = d[1]
		var b := Basis(Vector3.UP, fy)
		var stand := p + b * Vector3(1.4, 0.0, 3.0)
		stand.y = 0.45
		player.place(stand, rad_to_deg(fy) + 25.0, 2.0)
		player.set_camera(FP)
		await _settle(10)
		var nm := "door_%02d" % k
		get_viewport().get_texture().get_image().save_png("%s/%s.png" % [OUT, nm])
		print("door   %s  %-30s at %s%s" % [nm, d[2], p,
			"  WARNING: body lifted" if player.global_position.y > 0.40 else ""])
		k += 1


## A jump as a short frame sequence, third person front, so the body is seen.
func _jump_frames() -> void:
	player.place(Vector3(-4.0, 0.30, 0.6), 90.0, 0.0)
	player.set_camera(FRONT)
	await _settle(12)
	await _hold(0.6)   # physics frames: on the floor before the jump, not still settling
	# A fixed camera, not the rig: every rig camera follows the body, and a jump
	# filmed by a camera that rises with it shows nothing.
	var fixed := Camera3D.new()
	fixed.fov = 50.0
	slice.add_child(fixed)
	fixed.global_position = player.global_position + Vector3(-3.6, 1.1, 0.0)
	fixed.look_at(player.global_position + Vector3(0, 0.95, 0), Vector3.UP)
	fixed.make_current()
	await _settle(4)
	print("jump   before: feet y %.3f, on floor %s" % [player.global_position.y, player.is_on_floor()])
	# Saving a PNG takes longer than several physics ticks, which would leave
	# three frames for a 0.4 s flight. Slow the clock for the capture only; the
	# physics per tick is unchanged.
	Engine.time_scale = 0.12
	_phys_t = 0.0
	Input.action_press("jump")
	await get_tree().physics_frame
	Input.action_release("jump")
	var i := 0
	while i < 60:
		await RenderingServer.frame_post_draw
		get_viewport().get_texture().get_image().save_png("%s/jump_%02d.png" % [OUT, i])
		print("jump   frame %02d  sim t %.3f s  feet y %.3f" % [i, _phys_t, player.global_position.y])
		i += 1
		if _phys_t > 0.1 and player.is_on_floor():
			break
	Engine.time_scale = 1.0


# --- the character in the slice ------------------------------------------------

## Thresholds from the claim, not from the character (`ARC-23` rule 2): an idle
## that is a held pose moves no bone at all, and a walk swings a foot through
## a stride of tens of centimetres. 3 mm and 15 cm separate those cleanly.
const IDLE_MIN_M := 0.003
const WALK_MIN_M := 0.15

## Where the integration frames are taken: the open pavement west of the café,
## between the trees at x -20.5 and -12.5 and on the frontage side of the bench,
## and the café's own floor, on the loop `--drive` walks.
const CHAR_SPOTS := [
	["street", Vector3(-19.0, 0.45, -6.40), -90.0],
	["interior", Vector3(3.54, 0.60, -10.30), -90.0],
]


## Does the occupant animate in the slice, and does every camera mode draw it
## and the scene without torn or missing geometry? The first is measured on the
## occupant's bones. This is a probe, so it may look at the skeleton; the
## environment never does (`character_slot.gd`). The skeleton is found by type
## and the bone that moved most is named, so the number is shown to come from a
## limb and not from a root that slid (`ARC-23`).
func _character_check() -> void:
	print("== the reference character in the slice ==\n")
	print("character slot: %s" % player.slot.describe())
	var fails := 0
	var skel := _occupant_skeleton()
	if skel == null:
		print("FAIL: no Skeleton3D under the slot's occupant")
		return
	print("skeleton %s, %d bones" % [skel.get_path(), skel.get_bone_count()])
	player.place(CHAR_SPOTS[0][1], CHAR_SPOTS[0][2], 0.0)
	player.set_camera(REAR)
	await _settle(12)
	await _hold(0.5)
	# 4.5 s: longer than one breath of human.gd's standing loop (4.2 s), so a
	# breath is inside the window whatever phase the sample starts at
	# Two consecutive windows: a one-off settle after the body was placed moves
	# bones in the first only; a looping idle moves them in both.
	for w in ["first", "second"]:
		var idle := await _bone_excursion(skel, 4.5, false)
		print("idle   %s 4.5 s standing: largest bone excursion %.4f m (%s)"
			% [w, idle[0], idle[1]])
		fails += _range("idle animates, %s window (m)" % w, idle[0], IDLE_MIN_M, 10.0)
	var walk := await _bone_excursion(skel, 3.0, true)
	print("walk   3.0 s walking:  largest bone excursion %.4f m (%s)" % [walk[0], walk[1]])
	fails += _range("walk animates (bone excursion m)", walk[0], WALK_MIN_M, 10.0)
	await _hold(0.8)

	# every camera mode, standing and walking, outdoors and in
	for spot in CHAR_SPOTS:
		for m in [REAR, FRONT, FP]:
			player.place(spot[1], spot[2], 0.0)
			player.set_camera(m)
			await _settle(12)
			await _hold(0.4)
			var tag := "char_%s_%s" % [spot[0], _mode_tag(m)]
			await _save(tag + "_standing")
			Input.action_press("move_forward")
			await _hold(1.2)
			await _save(tag + "_walking")
			Input.action_release("move_forward")
			player.place(spot[1], spot[2], 0.0)
			await _settle(6)
			Input.action_press("move_forward")
			Input.action_press("jog")
			await _hold(1.0)
			await _save(tag + "_jogging")
			Input.action_release("jog")
			Input.action_release("move_forward")
			print("frames %s_standing / _walking / _jogging  [%s]; body visible %s"
				% [tag, player.rig.mode_name(), player.slot.occupant.visible])
	await _walk_strip()
	await _crowd_contact()
	print("\n%s" % ("all character checks pass" if fails == 0
		else "%d CHARACTER CHECKS FAILED" % fails))


func _occupant_skeleton() -> Skeleton3D:
	var found := player.slot.occupant.find_children("*", "Skeleton3D", true, false)
	return found[0] as Skeleton3D if not found.is_empty() else null


## Over `secs` of rendered frames, the largest distance any bone travelled in
## the skeleton's own frame (scaled to metres), and that bone's name. The
## skeleton's frame moves with the body, so walking across the street does not
## count; only the limbs do.
func _bone_excursion(skel: Skeleton3D, secs: float, walking: bool) -> Array:
	var lo: Array[Vector3] = []
	var hi: Array[Vector3] = []
	var s := skel.global_transform.basis.get_scale().x
	if walking:
		Input.action_press("move_forward")
	var t := 0.0
	while t < secs:
		await get_tree().process_frame
		t += get_process_delta_time()
		for i in skel.get_bone_count():
			var p := skel.get_bone_global_pose(i).origin * s
			if lo.size() <= i:
				lo.append(p)
				hi.append(p)
			else:
				lo[i] = lo[i].min(p)
				hi[i] = hi[i].max(p)
	if walking:
		Input.action_release("move_forward")
	var all: Array = []
	for i in lo.size():
		all.append([(hi[i] - lo[i]).length(), skel.get_bone_name(i)])
	all.sort_custom(func(a, b): return a[0] > b[0])
	var top: PackedStringArray = []
	for k in mini(6, all.size()):
		top.append("%s %.4f" % [all[k][1], all[k][0]])
	for nm in ["Hips", "Spine", "Chest", "UpperChest", "Neck", "Head"]:
		var i := skel.find_bone(nm)
		if i >= 0:
			top.append("[%s %.4f]" % [nm, (hi[i] - lo[i]).length()])
	print("       bones that moved most: %s" % ", ".join(top))
	return [all[0][0], all[0][1]] if not all.is_empty() else [0.0, "-"]


## Eight frames of the walk from a camera that stays put beside the pavement,
## so the gait is seen against the street rather than carried along with it.
func _walk_strip() -> void:
	player.place(Vector3(-19.0, 0.45, -6.40), -90.0, 0.0)
	player.set_camera(REAR)
	await _settle(12)
	var fixed := Camera3D.new()
	fixed.fov = 40.0
	slice.add_child(fixed)
	fixed.global_position = player.global_position + Vector3(2.2, 1.0, 4.2)
	fixed.look_at(player.global_position + Vector3(2.2, 0.9, 0.0), Vector3.UP)
	fixed.make_current()
	Engine.time_scale = 0.25
	Input.action_press("move_forward")
	await _hold(0.6)
	for i in range(8):
		await _hold(0.09)
		await _save("char_walk_%02d" % i)
	Input.action_release("move_forward")
	Engine.time_scale = 1.0
	fixed.queue_free()
	print("frames char_walk_00..07, a fixed camera beside the pavement")


## She jogs west along the pavement into the standing townsperson at
## (-8.5, -5.95) (`SliceStreetscape._people`), seen from a fixed camera across
## the pavement. Nothing makes bodies collide yet: the figures are drawn, not
## simulated, so she passes through him. This frame records that known gap for
## the physics step. It is evidence, not a check.
func _crowd_contact() -> void:
	var other := Vector3(-8.5, 0.14, -5.95)
	player.place(Vector3(-5.6, 0.45, -5.95), 90.0, 0.0)
	player.set_camera(REAR)
	await _settle(8)
	var fixed := Camera3D.new()
	fixed.fov = 45.0
	slice.add_child(fixed)
	fixed.global_position = other + Vector3(0.3, 1.45, 3.4)
	fixed.look_at(other + Vector3(0.3, 1.0, 0.0), Vector3.UP)
	fixed.make_current()
	# a new camera needs frames before its image is trusted: captured at once,
	# this frame came out black
	await _settle(12)
	await _hold(0.6)
	Input.action_press("move_forward")
	Input.action_press("jog")
	var nearest := INF
	var t := 0.0
	while t < 2.0:
		await get_tree().physics_frame
		t += get_physics_process_delta_time()
		var d := Vector2(player.global_position.x - other.x, player.global_position.z - other.z).length()
		nearest = minf(nearest, d)
		if d < 0.12:
			break
	await _save("char_run_into_townsperson")
	Input.action_release("jog")
	Input.action_release("move_forward")
	fixed.queue_free()
	print("frames char_run_into_townsperson: closest approach %.2f m between body centres "
		% nearest + "(bodies interpenetrate below ~0.5 m; nothing collides yet -- the physics step)")


func _mode_tag(m: CameraRig.Mode) -> String:
	match m:
		REAR: return "rear"
		FRONT: return "front"
	return "first"


## Waits for a drawn frame first, so the image is the scene as it now is.
func _save(nm: String) -> void:
	await RenderingServer.frame_post_draw
	get_viewport().get_texture().get_image().save_png("%s/%s.png" % [OUT, nm])


# --- performance ---------------------------------------------------------------

## Frame cost: the fixed protocol, its views and its breakdown live in
## `slice_perf.gd` (RL-b §3), because "performance is reasonable" is a claim that
## needs a number, and a number taken without warm-up or repeats is noise.
func _perf() -> void:
	await SlicePerf.new(self).run()


# --- the threshold measurement -------------------------------------------------

## `VISUAL_SLICE.md` sec.7.1, as numbers. Four samples on one straight path
## through the door, each one a real rendered frame from the real camera.
##
## The thresholds are stated here, before the run, so that they cannot be
## chosen afterwards to fit whatever came out:
const EXPOSURE_STEP_MAX := 3.0    ## no single step may change mean luma by more than 3x
const INTERIOR_LUMA_MIN := 0.055  ## below this an interior frame is "black"
const CLIP_MAX := 0.045           ## at most 4.5% of an interior-looking-out frame may clip
const CONTRAST_MIN := 0.055       ## p90 - p10 inside, or the room is flat

var samples := [
	["outside_pavement", Vector3(3.45, 0.45, -5.60), 0.0, 0.0],
	["in_the_doorway", Vector3(3.45, 0.55, -8.35), 0.0, 0.0],
	["two_metres_in", Vector3(3.55, 0.60, -10.40), -6.0, 0.0],
	["deep_inside", Vector3(5.60, 0.60, -14.20), -4.0, 0.0],
]


## The same four points through the florist's door (sec.4.1: "the same
## threshold measurement ... reported separately"), and its looking-out pose.
var florist_samples := [
	["florist_outside", Vector3(11.84, 0.45, -5.60), 0.0, 0.0],
	["florist_doorway", Vector3(11.84, 0.45, -8.30), 0.0, 0.0],
	["florist_two_m_in", Vector3(11.84, 0.45, -10.40), -6.0, 0.0],
	["florist_deep", Vector3(13.20, 0.45, -13.40), -10.0, 0.0],
]


func _threshold() -> void:
	print("== VISUAL_SLICE.md sec.7.1 -- indoor/outdoor, measured ==")
	print("gi mode: %s" % slice.gi_name())
	print("\n-- The Daily Bean --")
	await _threshold_run(samples, [Vector3(6.20, 0.60, -11.60), 178.0], "th_looking_out")
	print("\n-- The Flower Room (sec.4.1) --")
	await _threshold_run(florist_samples, [Vector3(15.0, 0.45, -12.0), 175.0],
		"th_florist_looking_out")


func _threshold_run(samples: Array, out_pose: Array, out_name: String) -> void:
	print("%-20s %8s %8s %8s %8s %8s" % ["sample", "mean", "p10", "p50", "p90", "clip%"])
	var means: Array[float] = []
	var stats: Array[Dictionary] = []
	for s in samples:
		player.place(s[1], s[2], s[3])
		player.set_camera(FP)
		await _settle(18)
		var img := get_viewport().get_texture().get_image()
		img.save_png("%s/th_%s.png" % [OUT, s[0]])
		var st := _luma_stats(img)
		stats.append(st)
		means.append(st["mean"])
		print("%-20s %8.4f %8.4f %8.4f %8.4f %7.2f%%" % [s[0], st["mean"],
			st["p10"], st["p50"], st["p90"], st["clip"] * 100.0])

	# looking back out through the glazing, from inside: the blown-out test
	player.place(out_pose[0], out_pose[1], 0.0)
	await _settle(18)
	var out_img := get_viewport().get_texture().get_image()
	out_img.save_png("%s/%s.png" % [OUT, out_name])
	var out_st := _luma_stats(out_img)
	print("%-20s %8.4f %8.4f %8.4f %8.4f %7.2f%%" % ["looking_out", out_st["mean"],
		out_st["p10"], out_st["p50"], out_st["p90"], out_st["clip"] * 100.0])

	print("\n-- verdicts --")
	var worst := 1.0
	for i in range(1, means.size()):
		var lo := minf(means[i], means[i - 1])
		var hi := maxf(means[i], means[i - 1])
		var ratio := hi / maxf(lo, 1e-5)
		worst = maxf(worst, ratio)
		print("step %-18s x%.2f" % [samples[i][0], ratio])
	_verdict("no catastrophic exposure jump", worst <= EXPOSURE_STEP_MAX,
		"worst step x%.2f, limit x%.2f" % [worst, EXPOSURE_STEP_MAX])
	var dark := 1.0
	for i in range(1, stats.size()):
		dark = minf(dark, stats[i]["mean"])
	_verdict("no black interior", dark >= INTERIOR_LUMA_MIN,
		"darkest interior frame mean %.4f, floor %.4f" % [dark, INTERIOR_LUMA_MIN])
	_verdict("no blown-out exterior through the glazing",
		out_st["clip"] <= CLIP_MAX,
		"%.2f%% clipped, limit %.2f%%" % [out_st["clip"] * 100.0, CLIP_MAX * 100.0])
	var contrast: float = stats[stats.size() - 1]["p90"] - stats[stats.size() - 1]["p10"]
	_verdict("the interior has direction, not flat fill", contrast >= CONTRAST_MIN,
		"p90-p10 inside %.4f, floor %.4f" % [contrast, CONTRAST_MIN])


func _verdict(claim: String, ok: bool, detail: String) -> void:
	print("%-46s %s  (%s)" % [claim, "PASS" if ok else "FAIL", detail])


## Frame luminance statistics, in linear light. The frame is already tonemapped
## and in sRGB, so it is linearised first: averaging sRGB values would report a
## perceptual number and the claim here is about exposure.
func _luma_stats(img: Image) -> Dictionary:
	var w := img.get_width()
	var h := img.get_height()
	var step := 4
	var vals: Array[float] = []
	var total := 0.0
	var clipped := 0
	var n := 0
	for y in range(0, h, step):
		for x in range(0, w, step):
			var c := img.get_pixel(x, y)
			var l := 0.2126 * _lin(c.r) + 0.7152 * _lin(c.g) + 0.0722 * _lin(c.b)
			vals.append(l)
			total += l
			if c.r > 0.985 and c.g > 0.985 and c.b > 0.985:
				clipped += 1
			n += 1
	vals.sort()
	return {
		"mean": total / maxf(float(n), 1.0),
		"p10": vals[int(n * 0.10)],
		"p50": vals[int(n * 0.50)],
		"p90": vals[int(n * 0.90)],
		"clip": float(clipped) / maxf(float(n), 1.0),
	}


static func _lin(c: float) -> float:
	return c / 12.92 if c <= 0.04045 else pow((c + 0.055) / 1.055, 2.4)


# --- the scale audit -----------------------------------------------------------

## `VISUAL_SLICE.md` sec.3, checked rather than eyeballed. Every line prints the
## measured value beside the range it has to be in, and says which it is.
func _measure() -> void:
	print("== VISUAL_SLICE.md sec.3 -- scale ==\n")
	print("-- declared: the constants the geometry is built from --")
	var fails := 0
	var rows := [
		["standing eye height", CameraRig.EYE_HEIGHT, 1.60, 1.70],
		["character stature", Player.BODY_HEIGHT, 1.70, 1.80],
		["walking speed m/s", Player.WALK_SPEED, 1.30, 1.50],
		["jogging speed m/s", Player.JOG_SPEED, 2.60, 3.40],
		["collision capsule radius", 0.30, 0.30, 0.36],
		["cafe floor to ceiling", SliceCafe.CEIL_Y - SliceCafe.FLOOR_Y, 3.00, 3.60],
		["cafe door leaf width", SliceCafe.DOOR_W, 0.85, 1.00],
		["cafe door head height", SliceCafe.DOOR_H - SliceCafe.FLOOR_Y, 2.05, 2.20],
		["entrance step riser", SliceCafe.FLOOR_Y, 0.10, 0.18],
		["shopfront glazing head", SliceCafe.GLAZE_HEAD, 2.60, 3.00],
		["stallriser height", SliceCafe.STALL_H, 0.20, 0.45],
		["fascia band height", SliceCafe.FASCIA_H, 0.70, 1.10],
		["cafe frontage width", SliceCafe.W, 7.00, 11.00],
		["cafe interior depth", SliceCafe.DEPTH, 8.00, 13.00],
		["upper storey height", SliceCafe.STOREY, 3.00, 3.60],
		["north pavement width", -SliceStreet.HALF_ROAD - SliceStreet.NORTH_FACE, 3.00, 4.50],
		["south pavement width", SliceStreet.SOUTH_FACE - SliceStreet.HALF_ROAD, 3.00, 4.50],
		["carriageway width", SliceStreet.HALF_ROAD * 2.0, 6.00, 8.00],
		["frontage to frontage", SliceStreet.SOUTH_FACE - SliceStreet.NORTH_FACE, 14.0, 22.0],
		["kerb upstand", SliceStreet.KERB_H, 0.10, 0.16],
		["walkable street length", SliceStreet.X_MAX - SliceStreet.X_MIN, 45.0, 70.0],
		["counter working height", SliceCafeInterior.COUNTER_TOP, 1.00, 1.10],
		["window bench seat height", 0.45, 0.42, 0.47],
		["sun elevation deg", -SliceMain.SUN_ELEVATION, 12.0, 25.0],
	]
	for r in rows:
		var v: float = r[1]
		var ok: bool = v >= float(r[2]) - 1e-6 and v <= float(r[3]) + 1e-6
		if not ok:
			fails += 1
		print("%-28s %7.3f   [%.2f .. %.2f]  %s"
			% [r[0], v, r[2], r[3], "ok" if ok else "OUT OF RANGE"])

	fails += _measure_geometry()

	print("\n-- borrowed props, as measured from their own geometry --")
	for line in SliceProps.audit():
		print("  " + line)

	print("\n-- scene --")
	print("  nodes in the slice tree      %d" % _count(slice.world))
	print("  gi mode                      %s" % slice.gi_name())
	print("  character slot               %s" % player.slot.describe())
	print("\n%s" % ("all scale checks pass" if fails == 0
		else "%d SCALE CHECKS OUT OF RANGE" % fails))


## The same claims, measured from what was BUILT rather than read from the
## constants it was built from (`ARC-23`): rays against the real colliders, and
## the stand-in's stature from its own mesh. A constant can be right while the
## geometry using it is wrong; this is what would catch that.
func _measure_geometry() -> int:
	print("\n-- measured: rays against the built colliders, and the occupant's mesh --")
	var fails := 0
	var nf := SliceStreet.NORTH_FACE
	var floor_y := SliceStreet.WALK_Y + SliceCafe.FLOOR_Y
	var door_x := 6.0 + SliceCafe.DOOR_X
	var dz := nf - SliceCafe.WALL_T * 0.5

	# floor and the solid structure above it, in the middle of the room
	var mid := Vector3(4.0, 1.5, nf - 4.0)
	var f: Variant = _ray(mid, mid + Vector3.DOWN * 3.0)
	var c: Variant = _ray(mid, mid + Vector3.UP * 5.0)
	if f != null and c != null:
		print("  room floor y %.3f; first solid above it at %.3f m -- ceiling boards are"
			% [f.y, c.y - f.y] + " %.2f m thick, so floor to ceiling underside %.3f m"
			% [0.14, c.y - f.y - 0.14])
		fails += _range("floor to ceiling underside", c.y - f.y - 0.14, 3.00, 3.60)
	# door head: the first solid above the middle of the threshold
	var head: Variant = _ray(Vector3(door_x, floor_y + 0.5, dz), Vector3(door_x, floor_y + 4.0, dz))
	if head != null:
		fails += _range("door clear head above threshold", head.y - floor_y, 2.00, 2.40)
	# door clear width: rays sideways from the door's centre line at waist height
	# at the jambs' own plane, just proud of the façade
	var y1 := floor_y + 1.0
	var jz := nf + 0.045
	var l: Variant = _ray(Vector3(door_x, y1, jz), Vector3(door_x - 3.0, y1, jz))
	var r: Variant = _ray(Vector3(door_x, y1, jz), Vector3(door_x + 3.0, y1, jz))
	if l != null and r != null:
		print("  door opening at 1.0 m: solid at x %.3f and x %.3f" % [l.x, r.x])
		fails += _range("door clear width (walkable)", r.x - l.x, 0.85, 1.60)
	# the whole frontage: where can a body pass, at three heights? The only
	# opening that may exist is the door.
	for h in [0.30, 1.00, 1.60]:
		var gaps: Array[String] = []
		var open_from := INF
		var x := 6.0 - SliceCafe.W * 0.5 + 0.05
		while x <= 6.0 + SliceCafe.W * 0.5 - 0.05:
			# from just inside the wall's inner face to just outside its outer
			# face, so furniture or pots either side cannot mask a gap
			var hit: Variant = _ray(Vector3(x, floor_y + h, nf - 0.40), Vector3(x, floor_y + h, nf + 0.25))
			var open := hit == null
			if open and open_from == INF:
				open_from = x
			if not open and open_from != INF:
				gaps.append("x %.2f..%.2f" % [open_from, x])
				open_from = INF
			x += 0.05
		if open_from != INF:
			gaps.append("x %.2f..end" % open_from)
		print("  frontage open at %.2f m above the floor: %s" % [h, ", ".join(gaps)])
		if gaps.size() != 1:
			fails += 1
			print("    OUT OF RANGE: expected exactly one opening, the door")
	# Is the café frontage in sun at all? A ray from each point toward the sun:
	# blocked means in shadow, and the collider that blocks it is named. And the
	# angle the beam meets the façade at, which is how strongly sun lights it.
	var sun := slice.get_node("Sun") as DirectionalLight3D
	var to_sun := sun.global_transform.basis.z.normalized()
	var face_n := Vector3(0, 0, 1)
	print("  sun: toward-sun %s; meets the café façade at %.1f deg; irradiance factor %.2f"
		% [to_sun, rad_to_deg(asin(clampf(to_sun.dot(face_n), -1, 1))), maxf(to_sun.dot(face_n), 0.0)])
	for pt in [Vector3(4.5, 1.2, nf + 0.25), Vector3(7.5, 1.6, nf + 0.25), Vector3(6.0, 4.5, nf + 0.25),
			Vector3(6.0, 0.16, nf + 1.5), Vector3(6.0, 0.16, nf + 3.5)]:
		# the scene's geometry only: a perceived person's pick collider is not a
		# building (step-15 A16-4)
		var q := PhysicsRayQueryParameters3D.create(pt, pt + to_sun * 120.0, Build.LAYER_WORLD)
		q.exclude = [player.get_rid()]
		var hit := player.get_world_3d().direct_space_state.intersect_ray(q)
		print("  sun at %s: %s" % [pt, "LIT" if hit.is_empty()
			else "shadowed by %s at %s" % [(hit["collider"] as Node).get_path(), hit["position"]]])

	fails += _measure_occupant()
	return fails


## The occupant's stature, from its own meshes. `VISUAL_SLICE.md` sec.3.1's
## 1.70-1.80 m is the person's height, sole to crown, so the hair is left out of
## it: a bun is a hairstyle, not stature (`human.gd` CANONICAL_HEIGHT excludes it
## for the same reason). Every mesh is listed with its own extent first, so the
## number is shown to come from the body and not from whatever else is tallest
## (`ARC-23`, locate before counting); the check fails if no `Body` mesh was read.
func _measure_occupant() -> int:
	var occ := player.slot.occupant as Node3D
	if occ == null:
		return 0
	var parts: Array = []
	_mesh_parts(occ, parts)
	var person := AABB()
	var have := false
	var saw_body := false
	for p in parts:
		var nm: String = p[0]
		var bb: AABB = p[1]
		var counted := nm != STATURE_EXCLUDED
		print("  occupant mesh %-10s y %.3f .. %.3f%s"
			% [nm, bb.position.y, bb.end.y, "" if counted else "   (not stature: excluded)"])
		if not counted:
			continue
		saw_body = saw_body or nm == "Body"
		person = bb if not have else person.merge(bb)
		have = true
	var whole := _world_aabb(occ)
	print("  slot occupant, every mesh: %.3f m (feet y %.3f, top y %.3f)"
		% [whole.size.y, whole.position.y, whole.end.y])
	print("  slot occupant, without %s: %.3f m (feet y %.3f, crown y %.3f); Body mesh read: %s"
		% [STATURE_EXCLUDED, person.size.y, person.position.y, person.end.y, saw_body])
	var fails := _range("occupant stature, from its mesh", person.size.y, 1.70, 1.80)
	if not saw_body:
		fails += 1
		print("    OUT OF RANGE: no mesh named Body was measured")
	return fails


## Hair is not stature; see `_measure_occupant`.
const STATURE_EXCLUDED := "Hair"


func _mesh_parts(n: Node, out: Array) -> void:
	if n is MeshInstance3D and (n as MeshInstance3D).visible:
		var mi := n as MeshInstance3D
		out.append([String(mi.name), mi.global_transform * mi.get_aabb()])
	for ch in n.get_children():
		_mesh_parts(ch, out)


func _range(nm: String, v: float, lo: float, hi: float) -> int:
	var ok := v >= lo - 1e-6 and v <= hi + 1e-6
	print("%-34s %7.3f   [%.2f .. %.2f]  %s" % [nm, v, lo, hi, "ok" if ok else "OUT OF RANGE"])
	return 0 if ok else 1


func _ray(a: Vector3, b: Vector3) -> Variant:
	var q := PhysicsRayQueryParameters3D.create(a, b, Build.LAYER_WORLD)
	q.exclude = [player.get_rid()]
	var hit := player.get_world_3d().direct_space_state.intersect_ray(q)
	return null if hit.is_empty() else hit["position"]


func _world_aabb(n: Node) -> AABB:
	var out := AABB()
	var have := false
	if n is VisualInstance3D and not (n is Light3D):
		var vi := n as VisualInstance3D
		var bb := vi.global_transform * vi.get_aabb()
		out = bb
		have = true
	for ch in n.get_children():
		var sub := _world_aabb(ch)
		if sub.size == Vector3.ZERO:
			continue
		out = sub if not have else out.merge(sub)
		have = true
	return out


func _count(n: Node) -> int:
	var k := 1
	for c in n.get_children():
		k += _count(c)
	return k


# --- the drive test ------------------------------------------------------------

## Walk the real controller through the real door and back out, and report what
## the body did. `VISUAL_SLICE.md` sec.6.3 is not satisfied by a door-shaped
## hole; it is satisfied by a body that got through one.
func _drive() -> void:
	print("== VISUAL_SLICE.md sec.6.3 -- the walk-in, measured ==\n")
	var fails := 0

	# The engine every number below is measured on (`DECISIONS.md` DEP-20).
	var engine: String = PhysicsEngineProbe.running()
	print("engine %s" % engine)
	if not engine.begins_with("Jolt Physics"):
		fails += 1
		print("  FAIL: the slice is selected to run on Jolt Physics (DEP-20)")

	# 1. the kerb. Start in the carriageway and walk onto the pavement.
	player.place(Vector3(3.45, 0.30, -2.00), 0.0, 0.0)
	await _hold(0.4)
	var y0 := player.global_position.y
	await _walk(2.6)
	var on_kerb := player.global_position.y
	print("kerb   road y %.3f -> pavement y %.3f, climbed %.3f m"
		% [y0, on_kerb, on_kerb - y0])
	if on_kerb < SliceStreet.WALK_Y - 0.05:
		fails += 1
		print("  FAIL: the player did not get onto the pavement")

	# 2. the doorway. Walk from the pavement across the threshold.
	player.place(Vector3(3.45, 0.45, -6.20), 0.0, 0.0)
	await _hold(0.4)
	await _walk(4.0)
	var inside := player.global_position
	var place := SliceWorld.place_at(slice.world, inside)
	print("door   ended at (%.2f, %.2f, %.2f), place = '%s'"
		% [inside.x, inside.y, inside.z, place])
	if place != SliceWorld.CAFE_PLACE:
		fails += 1
		print("  FAIL: walking through the door did not change the semantic place")
	if inside.z > SliceStreet.NORTH_FACE - 0.8:
		fails += 1
		print("  FAIL: the player did not get through the opening -- %s" % _blocker())

	# 3. the loop. Four legs around the room, back to the door. Each leg is a
	# DISTANCE, not a duration: timed legs overshot, and the south leg walked
	# 6.5 m straight back out of the door and reported that as a failed loop.
	var legs := [
		[Vector3(3.55, 0.60, -10.20), -90.0, 4.6],   # east along the front
		[Vector3(0.0, 0.0, 0.0), 0.0, 2.0],          # north, deeper in
		[Vector3(0.0, 0.0, 0.0), 90.0, 4.6],         # west, back across
		[Vector3(0.0, 0.0, 0.0), 180.0, 2.0],        # south, back to the door
	]
	player.place(legs[0][0], legs[0][1], 0.0)
	await _hold(0.3)
	var loop_start := player.global_position
	var travelled := 0.0
	for i in range(legs.size()):
		player.rotation.y = deg_to_rad(float(legs[i][1]))
		var before := player.global_position
		await _walk_dist(float(legs[i][2]), 6.0)
		var leg := before.distance_to(player.global_position)
		travelled += leg
		var p := player.global_position
		print("loop   leg %d ended at (%.2f, %.2f, %.2f), %.2f of %.2f m%s" % [i + 1, p.x, p.y,
			p.z, leg, legs[i][2], "" if leg > float(legs[i][2]) - 0.1
			else "  SHORT -- stopped by %s" % _blocker()])
		if leg < float(legs[i][2]) - 0.1:
			fails += 1
		if SliceWorld.place_at(slice.world, p) != SliceWorld.CAFE_PLACE:
			fails += 1
			print("  FAIL: leg %d left the cafe" % (i + 1))
	var back := loop_start.distance_to(player.global_position)
	print("loop   travelled %.2f m, ended %.2f m from where the loop started"
		% [travelled, back])
	if travelled < 8.0:
		fails += 1
		print("  FAIL: the loop did not get anywhere -- something is blocking it")
	if back > 0.8:
		fails += 1
		print("  FAIL: the loop did not close -- ended %.2f m from its start" % back)

	# 4. walls. Push into the back wall and into the counter and go nowhere.
	# from the lane west of the counter (the counter spans x 5.70..10.16), so the
	# push has a clear run at the wall rather than starting wedged against the
	# end of the back bar, which is what x 5.60 did: 0.00 m and no contact
	# Each push names the plane it must not cross -- the obstacle's own face,
	# from the geometry constants -- and fails if the body's centre gets past
	# it. The first version failed on "moved more than 1.4 m", a distance proxy
	# that called a body stopped 9 cm short of the back wall a pass-through.
	var nf := SliceStreet.NORTH_FACE
	var back_face := nf - SliceCafe.DEPTH + SliceCafe.WALL_T
	var counter_face := nf + SliceCafeInterior.COUNTER_Z + SliceCafeInterior.COUNTER_D * 0.5
	fails += await _wall("back wall", Vector3(3.00, 0.60, -16.80), 0.0, back_face, -1.0)
	fails += await _wall("counter", Vector3(7.40, 0.60, -14.60), 0.0, counter_face, -1.0)
	fails += await _wall("shopfront glazing", Vector3(7.60, 0.60, -9.40), 180.0, nf, 1.0)

	# 5. the way out, and the place changing back.
	player.place(Vector3(3.55, 0.60, -10.60), 180.0, 0.0)
	await _hold(0.3)
	await _walk(4.5)
	var out := player.global_position
	var out_place := SliceWorld.place_at(slice.world, out)
	print("exit   ended at (%.2f, %.2f, %.2f), place = '%s'"
		% [out.x, out.y, out.z, out_place])
	if out_place != SliceWorld.STREET_PLACE:
		fails += 1
		print("  FAIL: back outside, the semantic place is '%s', not '%s'"
			% [out_place, SliceWorld.STREET_PLACE])

	# 6. cameras, indoors. A mode switch must change nothing the controller owns.
	player.place(Vector3(5.20, 0.60, -12.40), -20.0, 0.0)
	await _hold(0.3)
	for m in [REAR, FRONT, FP]:
		var before := player.global_position
		player.set_camera(m)
		await _hold(0.25)
		var moved := before.distance_to(player.global_position)
		var cam := player.rig.active().global_position
		var d := cam.distance_to(player.global_position + Vector3.UP * CameraRig.PIVOT_HEIGHT)
		print("camera %-20s body moved %.4f m, boom %.2f m" % [
			player.rig.mode_name(), moved, d])
		if moved > 0.02:
			fails += 1
			print("  FAIL: switching the camera moved the body")
		if not _inside_room(cam):
			fails += 1
			print("  FAIL: the camera left the room")

	fails += await _jumps()
	fails += await _florist_walk()

	print("\n%s" % ("all drive checks pass" if fails == 0 else "%d DRIVE CHECKS FAILED" % fails))


## VISUAL_SLICE.md sec.4.1 / 6.3 for the second building: in through the
## florist's door on foot, a closed loop round the room clear of its staging,
## table, buckets and counter, and out again.
func _florist_walk() -> int:
	print("\n-- the second enterable building: The Flower Room --")
	var fails := 0
	var door_x := 11.84
	player.place(Vector3(door_x, 0.45, -5.50), 0.0, 0.0)
	await _hold(0.4)
	# in; north up the west lane; east past the table; south down the bucket
	# side; west along behind the window staging
	var legs := [[0.0, 4.0], [0.0, 3.7], [-90.0, 4.46], [180.0, 3.30], [90.0, 4.46], [0.0, -1.0]]
	var path: Array[String] = []
	var loop_start := Vector3.INF
	for leg in legs:
		if leg[1] < 0.0:
			break
		player.rotation.y = deg_to_rad(leg[0])
		await _walk_dist(leg[1], 6.0)
		var p := player.global_position
		var pl := SliceWorld.place_at(slice.world, p)
		path.append(pl)
		print("  leg yaw %4.0f %.2f m -> at (%.2f, %.2f) in %s%s" % [leg[0], leg[1], p.x, p.z,
			pl, "" if _last_block == "" else "  [touched %s]" % _last_block])
		if loop_start == Vector3.INF:
			loop_start = p
	# close the loop: the last leg ends level with where it began, give or take
	var back_to := loop_start - player.global_position
	player.rotation.y = 0.0 if back_to.z < 0.0 else PI
	await _walk_dist(absf(back_to.z), 4.0)
	var close := Vector2(player.global_position.x - loop_start.x,
		player.global_position.z - loop_start.z).length()
	print("  loop closes within %.2f m of where it began" % close)
	if close > 0.60:
		fails += 1
		print("  FAIL: the loop round the room did not close -- something is in the way")
	if not path.has(SliceWorld.FLORIST_PLACE):
		fails += 1
		print("  FAIL: the body never entered the florist")
	# and out through the door again
	await _walk_to_x(door_x)
	player.rotation.y = PI
	await _walk_dist(4.5, 6.0)
	var out := SliceWorld.place_at(slice.world, player.global_position)
	print("  out through the door: at %s in %s" % [player.global_position, out])
	if out != SliceWorld.STREET_PLACE:
		fails += 1
		print("  FAIL: did not walk back out onto the street")
	return fails


## Space to jump, measured. Bounds are the requirement's, not the
## implementation's (ARC-23 rule 2): a human jump of 0.40-0.50 m, landing where
## it took off, and no second jump from mid-air.
func _jumps() -> int:
	var fails := 0
	# 1. standing jump in the open carriageway
	player.place(Vector3(0.0, 0.30, 0.0), 0.0, 0.0)
	await _hold(0.4)
	var start := player.global_position
	var n0 := player.jumps
	var peak := await _jump_and_track(false)
	var end := player.global_position
	var rise := peak - start.y
	var drift := Vector2(end.x - start.x, end.z - start.z).length()
	print("jump   standing: rose %.3f m, landed y %.3f (start %.3f), xy drift %.4f m, on floor %s"
		% [rise, end.y, start.y, drift, player.is_on_floor()])
	if rise < 0.40 or rise > 0.50:
		fails += 1
		print("  FAIL: jump height outside 0.40-0.50 m")
	if drift > 0.02 or absf(end.y - start.y) > 0.02 or not player.is_on_floor():
		fails += 1
		print("  FAIL: did not land where it took off")
	if player.jumps - n0 != 1:
		fails += 1
		print("  FAIL: %d jumps counted for one press" % (player.jumps - n0))

	# 2. the same jump with a second press near the apex: must not jump again
	player.place(Vector3(0.0, 0.30, 0.0), 0.0, 0.0)
	await _hold(0.4)
	n0 = player.jumps
	var y0 := player.global_position.y
	var peak2 := await _jump_and_track(true)
	print("jump   pressed again in mid-air: jumps counted %d, rose %.3f m"
		% [player.jumps - n0, peak2 - y0])
	if player.jumps - n0 != 1 or peak2 - y0 > 0.50:
		fails += 1
		print("  FAIL: a press in mid-air made a second jump")

	# 3. a running jump: forward motion continues, and it lands on the floor
	player.place(Vector3(-6.0, 0.30, 0.0), -90.0, 0.0)
	await _hold(0.3)
	Input.action_press("move_forward")
	await _hold(0.8)
	var run0 := player.global_position
	var peak3 := await _jump_and_track(false)
	Input.action_release("move_forward")
	await _hold(0.3)
	print("jump   running: rose %.3f m, travelled %.2f m, on floor %s"
		% [peak3 - run0.y, player.global_position.distance_to(run0), player.is_on_floor()])
	if not player.is_on_floor():
		fails += 1
		print("  FAIL: running jump did not land")
	return fails


## Press jump for one physics tick, follow the body until it is back on the
## floor, and return the highest y it reached. `again` presses jump a second
## time at the apex.
func _jump_and_track(again: bool) -> float:
	Input.action_press("jump")
	await get_tree().physics_frame
	Input.action_release("jump")
	var peak := player.global_position.y
	var pressed_again := false
	var t := 0.0
	while t < 2.0:
		await get_tree().physics_frame
		t += get_physics_process_delta_time()
		peak = maxf(peak, player.global_position.y)
		if again and not pressed_again and player.velocity.y <= 0.0 and t > 0.1:
			pressed_again = true
			Input.action_press("jump")
			await get_tree().physics_frame
			Input.action_release("jump")
		if t > 0.1 and player.is_on_floor():
			break
	await _hold(0.2)
	return peak


func _inside_room(p: Vector3) -> bool:
	var back := SliceStreet.NORTH_FACE - SliceCafe.DEPTH + SliceCafe.WALL_T
	return p.x > 6.0 - SliceCafe.W * 0.5 - 0.1 and p.x < 6.0 + SliceCafe.W * 0.5 + 0.1 \
		and p.z < SliceStreet.NORTH_FACE + 0.1 and p.z > back - 0.1 \
		and p.y > 0.0 and p.y < SliceCafe.CEIL_Y + 0.3


## Push into an obstacle along z. `face_z` is the obstacle's face; `dir` is the
## sign of z the push travels in. Fails if the body's centre crosses the face.
func _wall(nm: String, from: Vector3, yaw: float, face_z: float, dir: float) -> int:
	player.place(from, yaw, 0.0)
	await _hold(0.3)
	var before := player.global_position
	await _walk(2.2)
	var d := before.distance_to(player.global_position)
	var gap := (face_z - player.global_position.z) * dir
	print("wall   %-20s pushed 2.2 s, moved %.2f m; body centre %.2f m short of the face at z %.2f"
		% [nm, d, gap, face_z])
	if gap < 0.0:
		print("  FAIL: the player went through the %s, ended at %s" % [nm, player.global_position])
		return 1
	print("         stopped by %s" % _blocker())
	return 0


## What the body last collided with, by node path and position -- so a failure
## names the obstacle instead of leaving it to be guessed (`ARC-23`: locate
## before counting).
var _last_block := ""


func _blocker() -> String:
	return _last_block if _last_block != "" else "no wall contact (floor only)"


func _sample_block() -> void:
	var best := ""
	for i in range(player.get_slide_collision_count()):
		var c := player.get_slide_collision(i)
		var o := c.get_collider() as Node
		if o == null:
			continue
		var n := c.get_normal()
		if absf(n.y) > 0.7:
			continue  # the floor, not an obstacle
		best = "%s at %s, contact %s normal %s" % [
			o.get_path(), (o as Node3D).global_position if o is Node3D else Vector3.ZERO,
			c.get_position(), n]
	if best != "":
		_last_block = best


## Walk forward until the body has covered `metres` in the horizontal plane,
## or `max_secs` pass -- whichever is first. Stops dead, then settles.
func _walk_dist(metres: float, max_secs: float) -> void:
	_last_block = ""
	var start := player.global_position
	Input.action_press("move_forward")
	var t := 0.0
	while t < max_secs:
		await get_tree().physics_frame
		_sample_block()
		t += get_physics_process_delta_time()
		var d := player.global_position - start
		if Vector2(d.x, d.z).length() >= metres:
			break
	Input.action_release("move_forward")
	player.velocity = Vector3.ZERO
	await _hold(0.25)


## Turn east or west and walk until the body is on the line x = `x`.
func _walk_to_x(x: float) -> void:
	var dx := x - player.global_position.x
	if absf(dx) < 0.05:
		return
	player.rotation.y = -PI * 0.5 if dx > 0.0 else PI * 0.5
	await _walk_dist(absf(dx), 6.0)


func _walk(secs: float) -> void:
	_last_block = ""
	Input.action_press("move_forward")
	var t := 0.0
	while t < secs:
		await get_tree().physics_frame
		_sample_block()
		t += get_physics_process_delta_time()
	Input.action_release("move_forward")
	await _hold(0.25)


func _hold(secs: float) -> void:
	var t := 0.0
	while t < secs:
		await get_tree().physics_frame
		t += get_physics_process_delta_time()
