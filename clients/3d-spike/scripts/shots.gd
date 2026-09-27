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
	_mode = "drive" if "--drive" in args else "shots"
	DirAccess.make_dir_recursive_absolute(OUT)


func _process(_d: float) -> void:
	# let the sky radiance, shadow splits and SSAO settle before the first frame
	_warm += 1
	if _warm < (8 if _mode == "drive" else 30):
		return
	set_process(false)
	if _mode == "drive":
		await _drive()
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
