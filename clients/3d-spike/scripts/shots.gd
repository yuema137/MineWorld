## Reproducible capture and a headless drive test.
##
##   --shots   teleport through a fixed list of viewpoints, save a PNG at each
##   --drive   press the real input actions and print what the body actually
##             did, so "collision works" is a measurement and not a hope
##
## Both drive the same Player through the same code path a human would.
class_name Shots
extends Node

const OUT := "res://shots"

var player: Player

const FP := CameraRig.Mode.FIRST_PERSON
const REAR := CameraRig.Mode.THIRD_REAR
const FRONT := CameraRig.Mode.THIRD_FRONT

var views := [
	# name, position, yaw, pitch, camera mode.
	# yaw 0 faces the lake (-Z), 90 faces west, 180 faces the shops, 270 east.
	["01_promenade_wide", Vector3(11.0, 0.2, -19.0), 102.0, -2.0, FP],
	["02_street_mid", Vector3(17.0, 0.2, 42.0), 0.0, -1.0, FP],
	["03_cafe_near", Vector3(9.0, 0.2, -16.8), 180.0, 2.0, FP],
	["04_npc", Vector3(-32.0, 0.2, -20.5), 112.0, -2.0, FP],
	["05_lake_scenic", Vector3(-74.0, 0.2, -24.5), 352.0, -2.0, FP],
	["06_plaza_fountain", Vector3(0.0, 0.2, -19.0), 0.0, -1.0, FP],
	["07_shopfront_detail", Vector3(-12.0, 0.2, -15.6), 180.0, 3.0, FP],
	["08_lake_trail", Vector3(-95.0, 0.2, -20.0), 64.0, -1.0, FP],
	["09_quayside", Vector3(-20.0, 0.2, -23.0), 330.0, -3.0, FP],
	["10_street_corner", Vector3(24.0, 0.2, -16.0), 150.0, 1.0, FP],
	# The three camera modes, from one standing position so they compare.
	["11_mode1_first_person", Vector3(6.0, 0.2, -18.6), 104.0, -2.0, FP],
	["12_mode2_third_rear", Vector3(6.0, 0.2, -18.6), 104.0, -2.0, REAR],
	["13_mode3_third_front", Vector3(6.0, 0.2, -18.6), 104.0, -2.0, FRONT],
	# The character in the wider scene, and the rear boom pulled in by a wall.
	["14_mode2_scenic_wide", Vector3(-70.0, 0.2, -24.0), 348.0, -1.0, REAR],
	["15_mode2_camera_pull_in", Vector3(-12.0, 0.2, -12.9), 0.0, -2.0, REAR],
	# Interiors. These are real rooms -- see interior.gd -- so the same three
	# cameras work inside, and `--drive` walks in through the door rather than
	# teleporting, which is the only way to prove the opening is not decoration.
	["16_cafe_door", Vector3(4.6, 0.2, -17.6), 165.0, 1.0, FP],
	["17_cafe_counter", Vector3(6.6, 0.2, -8.6), 225.0, 1.0, FP],
	["18_cafe_window_out", Vector3(9.0, 0.2, -5.0), 18.0, -1.0, FP],
	["19_cafe_third_front", Vector3(8.0, 0.2, -7.0), 200.0, 0.0, FRONT],
	["20_mart_interior", Vector3(-1.0, 0.2, -10.0), 180.0, 1.0, FP],
	["21_street_golden", Vector3(17.0, 0.2, -15.8), 99.0, 2.0, FP],
	# seated figures, close: a person's contact with a chair is wrong from every
	# angle but the one it was tuned on, so it gets its own viewpoints
	["23_cafe_seated", Vector3(3.2, 0.2, -17.2), 128.0, -4.0, FP],
	["24_bench_seated", Vector3(-34.0, 0.2, -20.6), 118.0, -5.0, FP],
]

## Two of the frames also get a head-and-shoulders crop saved beside them. The
## mannequin's face is deliberately minimal (ARC-4), so at 1600x900 the head is
## about 40 px and the difference between mode 2 and mode 3 is not reliably
## readable at a glance -- which is exactly how a correct front view gets read
## as a back view. The crop is cut from the same captured frame, so it cannot
## disagree with it.
const HEAD_CROP := {
	"12_mode2_third_rear": "12b_mode2_head_detail",
	"13_mode3_third_front": "13b_mode3_head_detail",
}
const HEAD_RECT := Rect2i(680, 375, 240, 240)
## The crop is shown at 2x. Nothing is invented: it is the captured pixels,
## enlarged, because a 60 px head decides whether this is a front view or a
## back view and it should not take a squint.
const HEAD_ZOOM := 2

var _i := 0
var _warm := 0
var _mode := ""


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	if player != null:
		player.scripted_look = true
	_mode = "shots"
	if "--drive" in args:
		_mode = "drive"
	elif "--portrait" in args:
		_mode = "portrait"
	elif "--bodycheck" in args:
		_mode = "bodycheck"
	elif "--motion" in args:
		_mode = "motion"
	DirAccess.make_dir_recursive_absolute(OUT)


func _process(_d: float) -> void:
	# let the sky radiance, shadow splits and SSAO settle before the first frame
	_warm += 1
	if _warm < (8 if _mode == "drive" else 30):
		return
	set_process(false)
	if _mode == "drive":
		await _drive()
	elif _mode == "portrait":
		await _portrait()
	elif _mode == "bodycheck":
		await _bodycheck()
	elif _mode == "motion":
		await _motion()
	else:
		await _capture()
	get_tree().quit(0)


func _capture() -> void:
	for v in views:
		player.place(v[1], v[2], v[3])
		player.set_camera(v[4])
		for i in range(12):
			await get_tree().process_frame
		await RenderingServer.frame_post_draw
		var img := get_viewport().get_texture().get_image()
		img.save_png("%s/%s.png" % [OUT, v[0]])
		if HEAD_CROP.has(v[0]):
			var rect := HEAD_RECT.intersection(Rect2i(Vector2i.ZERO, img.get_size()))
			var head := img.get_region(rect)
			head.resize(rect.size.x * HEAD_ZOOM, rect.size.y * HEAD_ZOOM,
				Image.INTERPOLATE_CUBIC)
			head.save_png("%s/%s.png" % [OUT, HEAD_CROP[v[0]]])
		print("shot %s at %s yaw %.0f  [%s]" % [v[0], v[1], v[2], player.rig.mode_name()])
	await _stride_frames()


## Where the character stands for her own portrait session, and the five views
## `docs/VISUAL_FIDELITY.md` §10 asks a review submission to contain.
##
## The framing is the *reference's* framing, which is the rule §8 states: the
## canonical plate is a chest-up portrait, so the comparison frame is chest-up.
## Judging a face in a 60 px head inside a street screenshot is the mistake this
## project has already made three times.
## The window is 1600x900 and neither `--resolution` nor a runtime resize moves
## it, so the portrait frame is cut out of the centre of the captured image.
## `Camera3D.fov` is vertical, so cropping the width changes the aspect and not
## the framing: what these files show is exactly what the client rendered.
const PORTRAIT_CROP := Rect2i(490, 0, 620, 900)
## Clear of the street trees' canopy. The first spot put her directly beneath
## one, and its alpha-scissored leaves rendered as dark shards across her crown
## -- which was read, reasonably, as an artefact of her hair. The promenade is
## lined with them, so this is far enough along it to have sky overhead.
## x = 30 was not: the street trees stand at z -22.6 every 11.3 m with gaps
## at |x| < 6 and |x - 17| < 7, so x = 30 sat between the x = 22.8 and 34.1
## trees and the camera, looking along +x, framed the 34.1 canopy over her
## crown in every portrait.  x = 4 is in the promontory gap, 15 m from either.
const PORTRAIT_SPOT := Vector3(4.0, 0.2, -16.6)
const PORTRAIT_YAW := 104.0
## name, camera distance, camera height, look-at height, yaw offset from her
## front in degrees, field of view
const PORTRAIT_VIEWS := [
	["P1_portrait_front", 1.15, 1.50, 1.46, 8.0, 40.0],
	["P2_portrait_tq", 1.15, 1.50, 1.46, 34.0, 40.0],
	["P3_full_front", 3.10, 1.05, 0.95, 6.0, 42.0],
	["P4_full_tq", 3.10, 1.05, 0.95, 38.0, 42.0],
	["P5_full_rear", 3.10, 1.05, 0.95, 180.0, 42.0],  # the only one from behind
	["P6_head", 0.68, 1.58, 1.56, 12.0, 46.0],
]


## The five review frames, from the running client, in its own lighting.
func _portrait() -> void:
	# No look input at all for the stills.  `scripted_look` lets the drive
	# test's synthetic mouse turn the player -- and with it, any real mouse
	# moving over the window turned her between shots, so the "rear" frame
	# once came back showing her front and two runs never agreed on a pose.
	player.scripted_look = false
	Input.mouse_mode = Input.MOUSE_MODE_VISIBLE
	player.place(PORTRAIT_SPOT, PORTRAIT_YAW, 0.0)
	player.set_camera(CameraRig.Mode.THIRD_FRONT)
	await _settle(0.6)
	var cam := Camera3D.new()
	add_child(cam)
	var base := player.global_position
	# her own facing, in world space: `place` sets the body yaw from PORTRAIT_YAW
	var face := deg_to_rad(PORTRAIT_YAW)
	for v in PORTRAIT_VIEWS:
		player.place(PORTRAIT_SPOT, PORTRAIT_YAW, 0.0)
		var a: float = face + deg_to_rad(v[4])
		cam.fov = v[5]
		# Godot yaw θ puts forward at (-sin θ, 0, -cos θ); standing in front of
		# her means stepping along that, not against it.  The first pass had the
		# sign the other way and photographed the back of her head six times.
		cam.position = base + Vector3(-sin(a) * v[1], v[2], -cos(a) * v[1])
		cam.look_at(base + Vector3(0, v[3], 0), Vector3.UP)
		cam.current = true
		await _settle(0.25)
		await RenderingServer.frame_post_draw
		_save_portrait(v[0])
		print("portrait %s  dist %.2f m  fov %.0f" % [v[0], v[1], v[5]])
		if v[0] == "P6_head":
			# the same head with the lids held shut: the evidence that the blink
			# closes the eye rather than merely moving the lid
			var bodies := player.find_children("*", "Human", true, false)
			if not bodies.is_empty():
				(bodies[0] as Human).blink_hold = Human.BLINK_PEAK
				await RenderingServer.frame_post_draw
				await RenderingServer.frame_post_draw
				_save_portrait("P6b_blink")
				(bodies[0] as Human).blink_hold = -1.0
				print("portrait P6b_blink")
	# and one mid-stride, because a still figure hides everything about a walk
	await _portrait_walk(cam, base, face)


func _portrait_walk(cam: Camera3D, base: Vector3, face: float) -> void:
	# walking needs the scripted controls back; she is re-placed first
	player.scripted_look = true
	player.place(PORTRAIT_SPOT, PORTRAIT_YAW, 0.0)
	var a := face + deg_to_rad(52.0)
	cam.fov = 42.0
	cam.position = base + Vector3(-sin(a) * 3.2, 1.05, -cos(a) * 3.2)
	cam.look_at(base + Vector3(0, 0.95, 0), Vector3.UP)
	cam.current = true
	Input.action_press("move_forward")
	await _settle(0.85)
	# frame her where she has walked to, from the same side and distance
	var now := player.global_position
	cam.position = now + Vector3(-sin(a) * 3.2, 1.05, -cos(a) * 3.2)
	cam.look_at(now + Vector3(0, 0.95, 0), Vector3.UP)
	await get_tree().process_frame
	await RenderingServer.frame_post_draw
	_save_portrait("P7_walk")
	Input.action_release("move_forward")
	print("portrait P7_walk")


## Frame sequences, because stiffness is about motion and stills under-show
## it. `idle`: one whole standing loop at the reference's three-quarter
## chest-up framing, every 0.5 s. `walk`: from the operator's third-person
## front camera, two seconds walking, then stopping, then standing, every
## 0.1 s. Written to shots/motion/; the launcher's caller assembles them.
func _motion() -> void:
	DirAccess.make_dir_recursive_absolute(OUT + "/motion")
	player.scripted_look = false
	Input.mouse_mode = Input.MOUSE_MODE_VISIBLE
	player.place(PORTRAIT_SPOT, PORTRAIT_YAW, 0.0)
	player.set_camera(CameraRig.Mode.THIRD_FRONT)
	await _settle(0.6)
	var cam := Camera3D.new()
	add_child(cam)
	var base := player.global_position
	var a := deg_to_rad(PORTRAIT_YAW + 34.0)
	cam.fov = 40.0
	cam.position = base + Vector3(-sin(a) * 1.15, 1.50, -cos(a) * 1.15)
	cam.look_at(base + Vector3(0, 1.46, 0), Vector3.UP)
	cam.current = true
	for i in 26:
		await _settle(0.5)
		await RenderingServer.frame_post_draw
		_save_portrait("motion/idle_%02d" % i)
	print("motion idle: 26 frames")
	cam.current = false
	cam.queue_free()
	player.scripted_look = true
	player.place(PORTRAIT_SPOT, PORTRAIT_YAW, -8.0)
	player.set_camera(CameraRig.Mode.THIRD_FRONT)
	await _settle(0.3)
	for i in 45:
		if i == 0:
			Input.action_press("move_forward")
		if i == 20:
			Input.action_release("move_forward")
		await _settle(0.1)
		await RenderingServer.frame_post_draw
		get_viewport().get_texture().get_image().save_png("%s/motion/walk_%02d.png" % [OUT, i])
	print("motion walk: 45 frames")
	player.scripted_look = false


## The operator's own views of the player character, full window: the two
## third-person cameras, standing and mid-stride, and each with her turned
## both ways so every camera sees her front and her back.
##
## The portrait frames are composed for comparison with the reference; these
## are what a player actually sees, and a body that is incomplete from some
## angle in some pose shows up here and not there.
func _bodycheck() -> void:
	var modes := [["rear", CameraRig.Mode.THIRD_REAR], ["front", CameraRig.Mode.THIRD_FRONT]]
	for m in modes:
		for turn in [0.0, 180.0]:
			for walking in [false, true]:
				player.scripted_look = walking
				Input.mouse_mode = Input.MOUSE_MODE_VISIBLE
				player.place(PORTRAIT_SPOT, PORTRAIT_YAW + turn, -8.0)
				player.set_camera(m[1])
				if walking:
					Input.action_press("move_forward")
				await _settle(0.9 if walking else 0.5)
				await RenderingServer.frame_post_draw
				var name := "B_%s_%s_%s" % [m[0], "turned" if turn > 0.0 else "facing",
					"walk" if walking else "stand"]
				get_viewport().get_texture().get_image().save_png("%s/%s.png" % [OUT, name])
				print("bodycheck ", name)
				if walking:
					Input.action_release("move_forward")
	player.scripted_look = false
	# and the townspeople, who share the body but not the wardrobe: the nearest
	# standing one and the nearest walking one, front and back, close up
	var cam := Camera3D.new()
	add_child(cam)
	cam.fov = 40.0
	var people := get_tree().root.find_children("*", "NPC", true, false)
	people.sort_custom(func(a: Node3D, b: Node3D) -> bool:
		return a.global_position.distance_to(PORTRAIT_SPOT) < b.global_position.distance_to(PORTRAIT_SPOT))
	var n := 0
	for p: Node3D in people:
		if n >= 3:
			break
		n += 1
		for side in [["front", 0.0], ["back", PI]]:
			var body := p.global_transform.basis
			var fwd := (body * Vector3(0, 0, 1)).normalized()
			fwd = fwd.rotated(Vector3.UP, side[1])
			cam.global_position = p.global_position + fwd * 2.4 + Vector3(0, 1.2, 0)
			cam.look_at(p.global_position + Vector3(0, 1.0, 0), Vector3.UP)
			cam.current = true
			await RenderingServer.frame_post_draw
			await RenderingServer.frame_post_draw
			var name := "B_npc%d_%s" % [n, side[0]]
			get_viewport().get_texture().get_image().save_png("%s/%s.png" % [OUT, name])
			print("bodycheck ", name)


func _save_portrait(name: String) -> void:
	var img := get_viewport().get_texture().get_image()
	var rect := PORTRAIT_CROP.intersection(Rect2i(Vector2i.ZERO, img.get_size()))
	img.get_region(rect).save_png("%s/%s.png" % [OUT, name])


## Two frames a known distance apart, while walking, from a fixed camera.
##
## The numbers in `--drive` say the gait is distance-driven; these say what it
## looks like. Because the camera does not move between them, a planted foot
## that has stayed planted occupies the same pixels in both, and a foot that
## skated does not. The pair is the visual half of the foot-sliding check and
## the reason the camera is parked rather than chasing the body.
func _stride_frames() -> void:
	var start := Vector3(-30.0, 0.2, -22.5)
	player.place(start, 90.0, 0.0)
	# REAR, not FP: first person hides the body, so the pair came back as two
	# frames of empty pavement with the character standing invisibly in them.
	player.set_camera(REAR)
	await _settle(0.4)
	# a fixed observer, off to the side, looking at where the walk will pass
	var cam := Camera3D.new()
	cam.fov = 42.0
	get_tree().current_scene.add_child(cam)
	cam.global_position = Vector3(-31.3, 0.95, -20.1)
	cam.look_at(Vector3(-31.3, 0.50, -22.5))
	# Walk to each mark, release the key, let the body settle, then capture.
	# Holding the key through the capture lets the awaits advance the walk --
	# the first attempt asked for frames 0.35 m apart and got 1.08 m and 8.95 m.
	for mark in [1.00, 1.35]:
		var guard := 0
		Input.action_press("move_forward")
		while start.distance_to(player.global_position) < mark and guard < 600:
			guard += 1
			await get_tree().physics_frame
		Input.action_release("move_forward")
		# claimed here, not at creation: the player rig makes its own camera
		# current every frame, so an observer camera set up earlier is silently
		# replaced and the pair comes back as two frames of empty pavement.
		cam.make_current()
		for k in range(8):
			await get_tree().process_frame
		await RenderingServer.frame_post_draw
		var tag := "a" if mark < 1.2 else "b"
		get_viewport().get_texture().get_image().save_png("%s/22_stride_%s.png" % [OUT, tag])
		print("stride frame %s at %.2f m from the start, walking, same fixed camera"
			% [tag, start.distance_to(player.global_position)])
	cam.queue_free()


## Foot sliding is a defect class, not a style question, so it gets measured
## rather than eyeballed. Two things are reported:
##
## * **gait cycles per metre.** The animation must advance with distance, so at
##   a given gait this number cannot change with speed. It legitimately differs
##   between walk and jog -- a jog has a longer stride -- which is why walk is
##   measured at two different speeds and compared with itself.
## * **stance foot slip.** The real artefact. While a foot is planted its world
##   position should not move; whatever it does move is the skate, in mm.
##
## Cycles are counted from the foot's own motion (its fore/aft position relative
## to the hips crossing the midpoint), not from anything the animation system
## reports about itself.
func _measure_gait(action: String, secs: float) -> Dictionary:
	var human: Human = player.body.body
	var sk := human.skeleton
	var b_hips := sk.find_bone("Hips")
	var b_lf := sk.find_bone("LeftFoot")
	var fore: Array[float] = []          # left foot fore/aft, relative to the hips
	var along: Array[float] = []         # distance travelled at that sample
	# Per tick, how far the better-planted foot moved. Taking the minimum of the
	# two feet needs no stance detection and cannot be fooled by a foot swap,
	# which is what wrecked the first version of this measurement: it tracked
	# "the lower foot" and dutifully reported the 80 cm gap between one foot and
	# the other as an 80 cm slide.
	var planted: Array[float] = []
	var p0 := player.global_position
	var prev_l := Vector3.ZERO
	var prev_r := Vector3.ZERO
	var have_prev := false
	if action != "":
		Input.action_press(action)
	# Sampled on physics frames. Render frames would read a fresher skeleton
	# pose, but --drive runs headless where there is no frame pacing, so the
	# process delta collapses and every per-tick figure derived from it is
	# meaningless. A fixed 60 Hz tick is worth more than a fresher sample.
	var t := 0.0
	while t < secs:
		await get_tree().physics_frame
		t += get_physics_process_delta_time()
		var hips := sk.get_bone_global_pose(b_hips).origin
		fore.append((sk.get_bone_global_pose(b_lf).origin - hips).z)
		along.append(p0.distance_to(player.global_position))
		var wl := human.foot_position(true)
		var wr := human.foot_position(false)
		if have_prev:
			var dl := Vector2(wl.x - prev_l.x, wl.z - prev_l.z).length()
			var dr := Vector2(wr.x - prev_r.x, wr.z - prev_r.z).length()
			planted.append(minf(dl, dr))
		prev_l = wl
		prev_r = wr
		have_prev = true
	if action != "":
		Input.action_release(action)

	# Cycles are counted between the FIRST and LAST midpoint crossing of the
	# foot's own swing, with the crossing instants interpolated. Counting over
	# the whole window instead would fold in the acceleration ramp and quantise
	# to half a cycle, which at these durations is a 10% error -- big enough to
	# hide the thing being tested.
	var cycles := 0.0
	var span := 0.0
	var amp := 0.0
	if fore.size() > 8:
		var mean := 0.0
		for v in fore:
			mean += v
		mean /= fore.size()
		for v in fore:
			amp = maxf(amp, absf(v - mean))
		if amp > 0.05:   # below this the body is standing, not stepping
			var xs: Array[float] = []
			for i in range(1, fore.size()):
				var a0 := fore[i - 1] - mean
				var a1 := fore[i] - mean
				if (a0 <= 0.0 and a1 > 0.0) or (a0 >= 0.0 and a1 < 0.0):
					var f: float = absf(a0) / maxf(absf(a0) + absf(a1), 1e-6)
					xs.append(lerpf(along[i - 1], along[i], f))
			if xs.size() >= 2:
				cycles = (xs.size() - 1) * 0.5
				span = xs[xs.size() - 1] - xs[0]

	planted.sort()
	var med := planted[planted.size() / 2] if planted.size() > 0 else 0.0
	var hz := 1.0 / maxf(get_physics_process_delta_time(), 1e-5)
	var body_speed := p0.distance_to(player.global_position) / maxf(secs, 1e-5)
	return {
		"dist": p0.distance_to(player.global_position),
		"cycles": cycles, "span": span,
		"per_m": (cycles / span) if span > 0.05 else 0.0,
		# median per-tick movement of the planted foot, as a speed, and as a
		# fraction of how fast the body is going. Perfect footing is 0%;
		# a body sliding with static legs is 100%.
		"slip_mps": med * hz,
		"slip_pct": (med * hz / body_speed * 100.0) if body_speed > 0.05 else 0.0,
	}


func _gait_report() -> void:
	print("\n-- gait cadence: does the animation follow the ground, or the clock --")
	player.place(Vector3(-16.0, 0.2, -22.0), 90.0, 0.0)
	await _settle(0.4)

	var still := await _measure_gait("", 1.5)
	print("standing still:   %.2f m travelled, %.2f gait cycles  (must be ~0)"
		% [still["dist"], still["cycles"]])

	player.place(Vector3(-16.0, 0.2, -22.0), 90.0, 0.0)
	await _settle(0.4)
	var w1 := await _measure_gait("move_forward", 4.0)
	print("walk 4.0 s:       %.2f m, %.2f cycles over %.2f m, %.3f cycles/m, planted foot drifts %.2f m/s = %.0f%% of body speed"
		% [w1["dist"], w1["cycles"], w1["span"], w1["per_m"], w1["slip_mps"], w1["slip_pct"]])

	player.place(Vector3(-16.0, 0.2, -22.0), 90.0, 0.0)
	await _settle(0.4)
	var w2 := await _measure_gait("move_forward", 2.2)
	print("walk 2.2 s:       %.2f m, %.2f cycles over %.2f m, %.3f cycles/m  (same gait, less time:"
		% [w2["dist"], w2["cycles"], w2["span"], w2["per_m"]])
	print("                  cycles/m must match the 4.0 s walk -- %.3f vs %.3f, delta %.3f)"
		% [w1["per_m"], w2["per_m"], absf(w1["per_m"] - w2["per_m"])])

	player.place(Vector3(-16.0, 0.2, -22.0), 90.0, 0.0)
	await _settle(0.4)
	Input.action_press("jog")
	var j := await _measure_gait("move_forward", 4.0)
	Input.action_release("jog")
	print("jog 4.0 s:        %.2f m, %.2f cycles over %.2f m, %.3f cycles/m, planted foot drifts %.2f m/s = %.0f%% of body speed"
		% [j["dist"], j["cycles"], j["span"], j["per_m"], j["slip_mps"], j["slip_pct"]])
	print("                  (a jog has a longer stride, so fewer cycles/m than a walk is")
	print("                   correct; and a run has a flight phase where neither foot is")
	print("                   planted, so its drift figure is not comparable to the walk)")
	await _contact_report()


func _hold(action: String, secs: float) -> void:
	Input.action_press(action)
	var t := 0.0
	while t < secs:
		await get_tree().physics_frame
		t += get_physics_process_delta_time()
	Input.action_release(action)


func _settle(secs: float) -> void:
	var t := 0.0
	while t < secs:
		await get_tree().physics_frame
		t += get_physics_process_delta_time()


## Walk the real controller and report. Anything surprising here is a bug in
## the controller, not in the test.
func _drive() -> void:
	player.scripted_look = true
	print("\n=== drive test ===")
	print("physics tick %d Hz" % Engine.physics_ticks_per_second)

	player.place(Vector3(2.0, 6.0, -19.0), 180.0, 0.0)
	await _settle(1.5)
	print("gravity: dropped from y=6.0 to y=%.3f, on_floor=%s"
		% [player.global_position.y, player.is_on_floor()])

	player.place(Vector3(2.0, 0.2, -19.0), 180.0, 0.0)
	await _settle(0.3)
	var p0 := player.global_position
	await _hold("move_forward", 2.0)
	await _settle(0.2)
	var p1 := player.global_position
	print("forward 2.0 s: moved %.2f m  (%.2f m/s)" % [p0.distance_to(p1), p0.distance_to(p1) / 2.0])

	await _hold("move_back", 2.0)
	await _settle(0.2)
	print("back 2.0 s:    returned to %.2f m from start" % p0.distance_to(player.global_position))

	await _gait_report()

	# --- can you actually walk inside? -------------------------------------
	# The cafe door is at world x 5.9 on the shopfront line; the room runs back
	# to z = +2. Teleporting a camera inside would prove nothing, so this walks
	# the real controller through the opening and reports where it ended up.
	player.place(Vector3(5.9, 0.2, -14.6), 180.0, 0.0)
	await _settle(0.3)
	var before_in := player.global_position
	await _hold("move_forward", 4.0)
	await _settle(0.2)
	var after_in := player.global_position
	var inside := after_in.z > -11.7 and absf(after_in.x - 9.0) < 4.4
	print("walk into the cafe: z %.2f -> %.2f (door at -12.0), x %.2f, inside=%s"
		% [before_in.z, after_in.z, after_in.x, inside])
	await _hold("move_back", 4.0)
	await _settle(0.2)
	print("walk back out:      z %.2f, outside=%s"
		% [player.global_position.z, player.global_position.z < -12.2])

	player.place(Vector3(2.0, 0.2, -19.0), 180.0, 0.0)
	await _settle(0.3)
	var p2 := player.global_position
	await _hold("move_right", 1.5)
	await _settle(0.2)
	var d_strafe := p2.distance_to(player.global_position)
	var right := (player.global_position - p2).normalized().dot(player.global_basis.x)
	print("strafe right 1.5 s: %.2f m, along body +X: %.2f" % [d_strafe, right])
	await _hold("move_left", 1.5)
	await _settle(0.2)
	print("strafe left back:   %.2f m from strafe start" % p2.distance_to(player.global_position))

	Input.action_press("jog")
	var p3 := player.global_position
	await _hold("move_forward", 1.5)
	Input.action_release("jog")
	await _settle(0.2)
	print("jog 1.5 s: %.2f m (%.2f m/s)"
		% [p3.distance_to(player.global_position), p3.distance_to(player.global_position) / 1.5])

	# mouse look, through the real event path
	var yaw0 := player.rotation.y
	var pitch0 := player.cam.rotation.x
	for i in range(30):
		var ev := InputEventMouseMotion.new()
		ev.relative = Vector2(20, -6)
		Input.parse_input_event(ev)
		await get_tree().process_frame
	print("mouse look: 600 px right -> yaw %+.1f deg ; 180 px up -> pitch %+.1f deg"
		% [rad_to_deg(player.rotation.y - yaw0), rad_to_deg(player.cam.rotation.x - pitch0)])
	for i in range(60):
		var ev2 := InputEventMouseMotion.new()
		ev2.relative = Vector2(0, -40)
		Input.parse_input_event(ev2)
		await get_tree().process_frame
	print("pitch clamp after 2400 px up: %+.1f deg" % rad_to_deg(player.cam.rotation.x))

	await _facing_report()
	await _camera_continuity()
	await _camera_collision()

	# collision: walk straight into the shopfront row
	player.place(Vector3(-12.0, 0.2, -17.0), 180.0, 0.0)
	await _settle(0.3)
	var before := player.global_position
	await _hold("move_forward", 6.0)
	await _settle(0.3)
	var after := player.global_position
	print("into shopfront: start z=%.2f end z=%.2f (wall at z=-12.0) -> %s"
		% [before.z, after.z, "STOPPED" if after.z < -12.0 else "WENT THROUGH"])

	# collision: walk into the fountain kerb
	player.place(Vector3(0.0, 0.2, -23.0), 180.0, 0.0)
	await _settle(0.3)
	await _hold("move_forward", 6.0)
	await _settle(0.3)
	var fz := player.global_position.z
	print("into fountain (rim at z=-25.8): end z=%.2f -> %s"
		% [fz, "STOPPED" if fz > -26.2 else "WENT THROUGH"])

	# collision: walk into the quay railing, which guards the water
	player.place(Vector3(-30.0, 0.2, -23.0), 0.0, 0.0)
	await _settle(0.3)
	await _hold("move_forward", 6.0)
	await _settle(0.3)
	var qz := player.global_position.z
	print("into quay rail (rail at z=-26.0): end z=%.2f -> %s"
		% [qz, "STOPPED" if qz > -26.4 else "WENT THROUGH"])

	# a long stroll across the whole scene, checking we stay on the ground
	player.place(Vector3(40.0, 0.2, -18.0), 90.0, 0.0)
	await _settle(0.3)
	var min_y := 999.0
	var max_y := -999.0
	Input.action_press("move_forward")
	var t := 0.0
	while t < 55.0:
		await get_tree().physics_frame
		t += get_physics_process_delta_time()
		min_y = minf(min_y, player.global_position.y)
		max_y = maxf(max_y, player.global_position.y)
	Input.action_release("move_forward")
	print("55 s walk west: ended at %s, y stayed in [%.2f, %.2f], on_floor=%s"
		% [player.global_position, min_y, max_y, player.is_on_floor()])
	print("=== drive test done ===\n")


## Press and release the real camera key, through the real input path, so this
## measures what a person pressing F5 gets and not a private back door.
func _tap_camera_key() -> void:
	for down in [true, false]:
		var ev := InputEventKey.new()
		ev.physical_keycode = KEY_F5
		ev.pressed = down
		Input.parse_input_event(ev)


## One measurement window of `ticks` physics ticks on a walking body, with the
## camera key tapped at the start of the window when `switch`. Everything a
## camera switch could plausibly damage is sampled at tick resolution.
func _window(switch: bool, ticks: int) -> Dictionary:
	await get_tree().physics_frame
	var p0 := player.global_position
	var v0 := player.velocity
	var yaw0 := player.rotation.y
	var id0 := player.get_instance_id()
	var mode0 := player.rig.mode_name()
	if switch:
		_tap_camera_key()
	var prev := p0
	var max_step := 0.0
	var min_step := 1e9
	var min_speed := 1e9
	var floor_all := true
	for i in range(ticks):
		await get_tree().physics_frame
		var step := prev.distance_to(player.global_position)
		max_step = maxf(max_step, step)
		min_step = minf(min_step, step)
		min_speed = minf(min_speed, Vector3(player.velocity.x, 0, player.velocity.z).length())
		floor_all = floor_all and player.is_on_floor()
		prev = player.global_position
	return {
		"moved": p0.distance_to(player.global_position),
		"dv": (player.velocity - v0).length(),
		"dyaw": absf(player.rotation.y - yaw0),
		"max_step": max_step,
		"min_step": min_step,
		"min_speed": min_speed,
		"on_floor": floor_all,
		"same_node": id0 == player.get_instance_id(),
		"mode": "%s -> %s" % [mode0, player.rig.mode_name()],
	}


## The requirement most likely to break silently: one movement controller, three
## cameras, and a switch that leaves the body strictly alone. Measured on a body
## that is walking at the time, against a control window with no switch.
func _camera_continuity() -> void:
	print("\n-- camera switching: three cameras, one body --")
	print("cameras: %s" % [[
		player.rig.first_person.name, player.rig.third_rear.name,
		player.rig.third_front.name]])
	player.place(Vector3(24.0, 0.2, -18.0), 96.0, 0.0)
	await _settle(0.4)
	Input.action_press("move_forward")
	await _settle(2.5)  # up to steady walking speed before measuring

	var ticks := 6
	var dt := get_physics_process_delta_time()
	print("each pair below is two adjacent %d-tick windows on the same walk: the first"
		% ticks)
	print("with no switch, the second with the camera key tapped at the start of it.")
	for i in range(4):
		var control: Dictionary = await _window(false, ticks)
		var w: Dictionary = await _window(true, ticks)
		print("%-42s moved %.4f m vs %.4f m with no switch (diff %+.5f m), per-tick %.4f-%.4f m, min speed %.4f m/s, |dv| %.4f m/s, dyaw %.5f rad, on_floor=%s, same body node=%s"
			% [w["mode"], w["moved"], control["moved"], w["moved"] - control["moved"],
				w["min_step"], w["max_step"], w["min_speed"], w["dv"], w["dyaw"],
				w["on_floor"], w["same_node"]])
	Input.action_release("move_forward")
	await _settle(0.4)
	print("expected per-tick step at 1.45 m/s and %.4f s ticks: %.4f m" % [dt, 1.45 * dt])

	# And the same check with the body in mid-air, where a reset would be
	# unmissable: switch while falling.
	player.set_camera(CameraRig.Mode.FIRST_PERSON)
	player.place(Vector3(2.0, 6.0, -19.0), 180.0, 0.0)
	await _settle(0.25)
	var vy0 := player.velocity.y
	var y0 := player.global_position.y
	_tap_camera_key()
	await get_tree().physics_frame
	print("switch mid-fall: vy %+.3f -> %+.3f m/s, y %.3f -> %.3f (gravity 22 m/s^2 over one tick = %+.3f)"
		% [vy0, player.velocity.y, y0, player.global_position.y, -22.0 * dt])
	await _settle(1.2)
	print("landed at y=%.3f, on_floor=%s, camera now [%s]"
		% [player.global_position.y, player.is_on_floor(), player.rig.mode_name()])


## The third-person cameras must not sit inside the world. One ray, pulled in to
## the first hit -- so the measurement is simply: is the boom shorter when there
## is a building where the camera wanted to be?
func _camera_collision() -> void:
	print("\n-- third-person camera collision --")
	var full := Vector3(0, CameraRig.REAR_LIFT, CameraRig.REAR_DISTANCE).length()
	for c in [
			["open promenade", Vector3(-60.0, 0.2, -22.0), 90.0],
			["back to the shopfront row (wall at z=-12.0)", Vector3(-12.0, 0.2, -12.9), 0.0],
			["back to a side-street facade (wall at x=13.0)", Vector3(14.3, 0.2, 19.0), 270.0],
		]:
		player.place(c[1], c[2], 0.0)
		player.set_camera(CameraRig.Mode.THIRD_REAR)
		await _settle(0.5)
		var pivot := player.global_position + Vector3.UP * CameraRig.PIVOT_HEIGHT
		var boom := pivot.distance_to(player.rig.third_rear.global_position)
		print("rear boom, %-44s %.2f m of %.2f m -> %s"
			% [c[0], boom, full, "PULLED IN" if boom < full - 0.05 else "clear"])
	player.set_camera(CameraRig.Mode.FIRST_PERSON)


## Ground truth for "which way is the character facing", independent of what
## any screenshot looks like. The mannequin is authored facing its own local
## +Z, so `body.global_transform.basis.z` is the direction its face points.
##
## Note the invariant in the last two numbers: the two third-person cameras sit
## on opposite sides of the body along the view axis, so the face can only ever
## point toward exactly one of them. "Both views show the back of the head" is
## not a state this rig can be in -- if it looks that way, the head is being
## read at a scale where a 40 px face is not legible, which is what the head
## detail crops are for.
func _facing_report() -> void:
	print("\n-- which way is the character facing --")
	player.set_camera(CameraRig.Mode.FIRST_PERSON)
	player.place(Vector3(6.0, 0.2, -18.6), 104.0, -2.0)
	await _settle(0.4)

	var face := player.body.global_transform.basis.z   # authored +Z = the face
	var fwd := -player.global_transform.basis.z        # Godot forward = -Z
	var view := player.rig.view_direction()
	var pivot := player.global_position + Vector3.UP * CameraRig.PIVOT_HEIGHT
	print("player forward (-basis.z)      %s" % fwd)
	print("mesh face direction (+basis.z) %s" % face)
	print("mesh face . player forward     %+.4f  (+1 = the character faces where it walks)"
		% face.dot(fwd))

	for c in [["rear ", player.rig.third_rear], ["front", player.rig.third_front]]:
		var cam: Camera3D = c[1]
		var off := cam.global_position - pivot
		print("%s camera: offset along view axis %+.3f m (%s the body), mesh face . direction to camera %+.4f -> camera sees the %s"
			% [c[0], off.dot(view), "ahead of" if off.dot(view) > 0.0 else "behind",
				face.dot(off.normalized()),
				"FACE" if face.dot(off.normalized()) > 0.0 else "BACK OF THE HEAD"])


## Is either foot ever actually on the ground?
##
## A walk has a support phase: at least one foot in contact at all times. That
## is a different property from the drift measured above -- drift asks whether
## a planted foot slides, this asks whether anything is planted at all. A body
## hovering with its legs cycling below it would pass the drift test and fail
## this one, so the two are reported separately.
##
## Calibrated against the character standing still, where the sole is on the
## ground by construction, so the number means "how far off the ground the sole
## is" without needing to know where the sole sits inside the mesh.
func _contact_report() -> void:
	var human: Human = player.body.body
	player.place(Vector3(-16.0, 0.2, -22.0), 90.0, 0.0)
	await _settle(0.6)
	var rest := minf(human.foot_position(true).y, human.foot_position(false).y)
	var lo := 1e9
	var hi := -1e9
	var down := 0
	var n := 0
	Input.action_press("move_forward")
	var t := 0.0
	while t < 3.0:
		await get_tree().physics_frame
		t += get_physics_process_delta_time()
		if t < 0.8:
			continue     # let the walk reach steady state
		var m: float = minf(human.foot_position(true).y,
			human.foot_position(false).y) - rest
		lo = minf(lo, m)
		hi = maxf(hi, m)
		if m <= 0.015:
			down += 1
		n += 1
	Input.action_release("move_forward")
	print("\n-- ground contact: is a foot ever actually down --")
	print("lower foot vs its standing height: min %+.3f m, max %+.3f m" % [lo, hi])
	print("in contact (within 15 mm of the ground) on %d%% of sampled frames"
		% [int(round(100.0 * down / maxi(n, 1)))])
	print("   (negative is the sole clipping into the pavement, positive is hover;")
	print("    a walk needs a support phase, so this should sit near zero for a")
	print("    good part of every cycle rather than always positive)")
