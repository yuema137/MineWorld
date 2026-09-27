extends Node
var cam: Camera3D
var anim := "Walk"
const SHOTS := [["front", 0.0, 1.05, 2.4], ["face", 0.0, 1.60, 0.62],
	["side", 90.0, 1.05, 2.4], ["back", 180.0, 1.05, 2.4]]

func _ready() -> void:
	for i in 30:
		await get_tree().process_frame
	for s in SHOTS:
		var yaw: float = deg_to_rad(s[1])
		cam.position = Vector3(sin(yaw) * float(s[3]), float(s[2]), cos(yaw) * float(s[3]))
		cam.look_at(Vector3(0, float(s[2]) - 0.02, 0))
		for i in 8:
			await get_tree().process_frame
		await RenderingServer.frame_post_draw
		get_viewport().get_texture().get_image().save_png("res://preview_%s_%s.png" % [anim, s[0]])
		print("saved ", s[0])
	get_tree().quit()
