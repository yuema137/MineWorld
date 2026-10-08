extends Node

## Stills from the running client, behind `--capture` (needs a window: `frame_post_draw` never
## arrives headless, the spike's finding). Written to `--shots=<dir>`, else `clients/2d/shots/<set>/`.
## The camera is moved for a framing and then handed back to the player.

var app: Node


func _dir() -> String:
	var dir := String(app.options.get("shots", ""))
	if dir == "":
		var set_name: String = "plain" if app.presentation.is_plain() else app.presentation.variant
		dir = ProjectSettings.globalize_path("res://shots/%s" % set_name)
	DirAccess.make_dir_recursive_absolute(dir)
	return dir


## One still at the current framing.
func shoot(name: String, zoom := 0.0, focus: Variant = null) -> void:
	var camera: Camera2D = app.camera
	var keep_zoom := camera.zoom
	camera.position_smoothing_enabled = false
	app.set_process(false)
	if zoom > 0.0:
		camera.zoom = Vector2.ONE * zoom
	if focus != null:
		camera.position = app.projection.to_screen(focus)
	for i in range(6):
		await get_tree().process_frame
	await RenderingServer.frame_post_draw
	var image := get_viewport().get_texture().get_image()
	var path := _dir().path_join(name + ".png")
	var saved := image.save_png(path)
	print("EVIDENCE ", JSON.stringify({"still": path, "saved": saved == OK, "size": [image.get_width(), image.get_height()]}))
	camera.zoom = keep_zoom
	camera.position_smoothing_enabled = true
	app.set_process(true)


## The café sequence: the room from inside, and a framing at the reference plate's scale.
func shoot_interior() -> void:
	var body: Vector2 = app.walker.body_plan
	await shoot("03_interior", 1.6, body + Vector2(1.5, -2.0))
	await shoot("04_ref_framing", 2.3, body + Vector2(0.5, -1.0))
