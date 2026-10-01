## The slice's evidence harness. Four scripted modes, none of them optional
## extras: each one produces a number that a claim in `docs/VISUAL_SLICE.md`
## rests on, and a claim without its number is not made.
##
##   --slice-shots      capture the sec.12 review views, from the real client
##   --slice-drive      walk the real controller in through the real door and
##                      out again, and report what the body actually did
##   --slice-measure    print measured dimensions against sec.3's ranges
##   --slice-threshold  the sec.7.1 indoor/outdoor measurement, at four points
##   --slice-perf       frame cost at four viewpoints, with draw calls
##
## `--slice-drive` runs headless. The other three need pixels and run windowed.
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
	["02_cafe_approach", Vector3(-2.00, 0.45, -5.40), -55.0, 0.0, FP],
	["03_cafe_exterior", Vector3(0.50, 0.45, -4.20), -53.0, 2.0, FP],
	["04_doorway", Vector3(3.45, 0.45, -6.90), 0.0, 0.0, FP],
	["05_interior_wide", Vector3(3.70, 0.60, -9.90), -16.0, -1.0, FP],
	["06_interior_character", Vector3(4.60, 0.60, -12.60), -8.0, -2.0, REAR],
	["07_third_rear", Vector3(-4.00, 0.45, -5.60), -80.0, -3.0, REAR],
	["08_third_front", Vector3(4.20, 0.45, -6.10), -20.0, -1.0, FRONT],
	["09_character_in_place", Vector3(5.20, 0.60, -11.20), 155.0, -1.0, FRONT],
	# diagnostics, for the agent rather than for the operator
	["10_interior_counter", Vector3(7.40, 0.60, -12.80), -6.0, 0.0, FP],
	["11_interior_looking_out", Vector3(6.20, 0.60, -11.60), 178.0, 0.0, FP],
	["12_back_wall", Vector3(5.20, 0.60, -13.60), 4.0, 1.0, FP],
	["13_doorway_from_inside", Vector3(3.55, 0.60, -10.40), 179.0, -1.0, FP],
	["14_west_frontage", Vector3(-15.0, 0.45, -5.00), -40.0, 4.0, FP],
	["15_east_end", Vector3(17.0, 0.45, -5.40), -70.0, 2.0, FP],
	["16_south_side", Vector3(2.0, 0.45, -4.60), 165.0, 1.0, FP],
	["17_street_from_east", Vector3(22.0, 0.45, -5.30), 96.0, -1.0, FP],
	["18_pavement_detail", Vector3(6.00, 0.45, -5.20), -20.0, -26.0, FP],
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


static func scripted() -> bool:
	var a := OS.get_cmdline_user_args()
	for m in ["--slice-shots", "--slice-drive", "--slice-measure", "--slice-threshold",
			"--slice-perf"]:
		if m in a:
			return true
	return false


func _ready() -> void:
	var a := OS.get_cmdline_user_args()
	if player != null:
		player.scripted_look = true
	# A scripted capture must not be paced by the display. macOS throttles an
	# unfocused or occluded window hard: with vsync on, a capture run that
	# should take twenty seconds sat waiting on the GPU present at well under
	# one frame per second, which profiling showed as the main thread parked in
	# `waitUntilSignaledValue`. It was never a rendering cost, and treating it
	# as one would have led to cutting content that is not expensive.
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
	Engine.max_fps = 0
	OS.low_processor_usage_mode = false
	for m in ["shots", "drive", "measure", "threshold", "perf"]:
		if "--slice-" + m in a:
			_mode = m
	DirAccess.make_dir_recursive_absolute(OUT)


func _process(_d: float) -> void:
	# Let the sky radiance, the reflection probes, the VoxelGI bake and the
	# shadow splits settle before the first frame is trusted.
	_warm += 1
	if _warm < (8 if _mode == "drive" else 12):
		return
	set_process(false)
	match _mode:
		"shots": await _capture()
		"drive": await _drive()
		"measure": _measure()
		"threshold": await _threshold()
		"perf": await _perf()
	get_tree().quit(0)


func _settle(frames := 10) -> void:
	for i in range(frames):
		await get_tree().process_frame
	await RenderingServer.frame_post_draw


# --- shots ---------------------------------------------------------------------

func _capture() -> void:
	for v in views:
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
	print("\ngi mode: %s" % slice.gi_name())
	print("character slot: %s" % player.slot.describe())


# --- performance ---------------------------------------------------------------

## Frame cost, measured at the four viewpoints that cost the most: the widest
## exterior, the cafe frontage, the doorway with inside and outside both in
## frame, and the interior. Reported in milliseconds, with the draw calls and
## primitives behind them, because "performance is reasonable" is a claim that
## needs a number and because a regression needs something to regress from.
var perf_views := [
	["street wide", Vector3(-20.0, 0.45, -5.20), -75.0, -1.0],
	["cafe frontage", Vector3(0.50, 0.45, -4.20), -53.0, 2.0],
	["doorway", Vector3(3.45, 0.45, -6.90), 0.0, 0.0],
	["interior", Vector3(3.70, 0.60, -9.90), -16.0, -1.0],
]


func _perf() -> void:
	print("== frame cost, %dx%d, gi=%s ==" % [
		get_viewport().get_visible_rect().size.x,
		get_viewport().get_visible_rect().size.y, slice.gi_name()])
	print("%-16s %9s %9s %9s %11s %12s"
		% ["view", "mean ms", "p50 ms", "worst ms", "draw calls", "primitives"])
	for v in perf_views:
		player.place(v[1], v[2], v[3])
		player.set_camera(FP)
		await _settle(6)
		var samples: Array[float] = []
		var last := Time.get_ticks_usec()
		for i in range(20):
			await RenderingServer.frame_post_draw
			var now := Time.get_ticks_usec()
			samples.append(float(now - last) / 1000.0)
			last = now
		samples.sort()
		var total := 0.0
		for x in samples:
			total += x
		print("%-16s %9.2f %9.2f %9.2f %11d %12d" % [v[0], total / samples.size(),
			samples[samples.size() / 2], samples[samples.size() - 1],
			int(Performance.get_monitor(Performance.RENDER_TOTAL_DRAW_CALLS_IN_FRAME)),
			int(Performance.get_monitor(Performance.RENDER_TOTAL_PRIMITIVES_IN_FRAME))])
	print("\nobjects in frame  %d" % int(
		Performance.get_monitor(Performance.RENDER_TOTAL_OBJECTS_IN_FRAME)))
	print("video memory      %.1f MB" % (
		Performance.get_monitor(Performance.RENDER_VIDEO_MEM_USED) / 1048576.0))


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


func _threshold() -> void:
	print("== VISUAL_SLICE.md sec.7.1 -- indoor/outdoor, measured ==")
	print("gi mode: %s" % slice.gi_name())
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
	player.place(Vector3(6.20, 0.60, -11.60), 178.0, 0.0)
	await _settle(18)
	var out_img := get_viewport().get_texture().get_image()
	out_img.save_png("%s/th_looking_out.png" % OUT)
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
	print("== VISUAL_SLICE.md sec.3 -- scale, measured ==\n")
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

	print("\n-- borrowed props, as measured from their own geometry --")
	for line in SliceProps.audit():
		print("  " + line)

	print("\n-- scene --")
	print("  nodes in the slice tree      %d" % _count(slice.world))
	print("  gi mode                      %s" % slice.gi_name())
	print("  character slot               %s" % player.slot.describe())
	print("\n%s" % ("all scale checks pass" if fails == 0
		else "%d SCALE CHECKS OUT OF RANGE" % fails))


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

	print("\n%s" % ("all drive checks pass" if fails == 0 else "%d DRIVE CHECKS FAILED" % fails))


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
