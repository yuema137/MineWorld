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

var views := [
	["01_promenade_wide", Vector3(2.0, 0.2, -16.0), 200.0, -3.0],
	["02_street_mid", Vector3(17.0, 0.2, 34.0), 0.0, -2.0],
	["03_cafe_near", Vector3(7.6, 0.2, -16.6), 182.0, 1.0],
	["04_npc", Vector3(-33.0, 0.2, -18.6), 212.0, -2.0],
	["05_lake_scenic", Vector3(-70.0, 0.2, -22.0), 18.0, -1.0],
	["06_plaza_fountain", Vector3(0.0, 0.2, -22.0), 178.0, 0.0],
	["07_shopfront_detail", Vector3(-11.0, 0.2, -15.2), 170.0, 3.0],
	["08_trail_north", Vector3(-88.0, 0.2, -18.0), 300.0, -2.0],
]

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
		for i in range(12):
			await get_tree().process_frame
		await RenderingServer.frame_post_draw
		var img := get_viewport().get_texture().get_image()
		img.save_png("%s/%s.png" % [OUT, v[0]])
		print("shot %s at %s yaw %.0f" % [v[0], v[1], v[2]])


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
